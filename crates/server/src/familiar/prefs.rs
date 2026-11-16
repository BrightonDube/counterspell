//! Mascot preferences: what a learner may set, how a request is checked, and how a patch merges into
//! the stored value. Pure: no database, no clock, no request types.
//!
//! The learner can set `muted`, `hidden`, `skin` and `speech`. Unknown keys, wrong types and unknown
//! enum values are refused, so the stored object only ever holds these four fields.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

/// The largest preferences request body, in bytes.
pub const MAX_PATCH_BYTES: usize = 1024;

/// The mascot skin. Only the jack-o'-lantern exists until the later skins ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Skin {
    /// The default jack-o'-lantern.
    #[serde(rename = "jack-o-lantern")]
    JackOLantern,
}

/// Whether the speech bubble asks the AI provider. `off` keeps the bubble to a template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Speech {
    /// Ask the hint providers, then fall back to the template.
    Auto,
    /// Never call a provider.
    Off,
}

/// The full set of mascot preferences, with defaults filled in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamiliarPrefs {
    /// No animation or sound; the words stay.
    pub muted: bool,
    /// Shows the 16 px dot instead of the figure.
    pub hidden: bool,
    /// The mascot skin.
    pub skin: Skin,
    /// Whether the speech bubble may call an AI provider.
    pub speech: Speech,
}

impl Default for FamiliarPrefs {
    fn default() -> Self {
        Self {
            muted: false,
            hidden: false,
            skin: Skin::JackOLantern,
            speech: Speech::Auto,
        }
    }
}

/// A validated request: each field is set only if the request named it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Patch {
    /// New `muted` value, when the request set it.
    pub muted: Option<bool>,
    /// New `hidden` value, when the request set it.
    pub hidden: Option<bool>,
    /// New `skin` value, when the request set it.
    pub skin: Option<Skin>,
    /// New `speech` value, when the request set it.
    pub speech: Option<Speech>,
}

/// Why a preferences request was refused. The messages name fixed fields only, never the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefsError {
    /// The body was not a JSON object.
    NotAnObject,
    /// The object had a key that is not a preference.
    UnknownField,
    /// A field had the wrong JSON type.
    WrongType(&'static str),
    /// A field had a value outside its allowed set.
    BadValue(&'static str),
    /// The body was larger than [`MAX_PATCH_BYTES`].
    TooLarge,
}

impl PrefsError {
    /// The message shown to the learner. It names the field, never the rejected value.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::NotAnObject => "Preferences must be a JSON object.".to_owned(),
            Self::UnknownField => "Preferences contain a field that is not allowed.".to_owned(),
            Self::WrongType(field) => format!("The field `{field}` has the wrong type."),
            Self::BadValue(field) => {
                format!("The field `{field}` has a value that is not allowed.")
            }
            Self::TooLarge => format!("Preferences must be at most {MAX_PATCH_BYTES} bytes."),
        }
    }
}

/// Refuses a body larger than [`MAX_PATCH_BYTES`] before it is parsed.
///
/// # Errors
///
/// Returns [`PrefsError::TooLarge`] when `len` is over the limit.
pub fn check_size(len: usize) -> Result<(), PrefsError> {
    if len > MAX_PATCH_BYTES {
        Err(PrefsError::TooLarge)
    } else {
        Ok(())
    }
}

/// Checks a parsed request body and returns the fields it sets. An empty object is a valid patch.
///
/// # Errors
///
/// Returns a [`PrefsError`] for a non-object body, an unknown key, a wrong type or a value outside
/// the allowed set.
pub fn validate_patch(value: &Value) -> Result<Patch, PrefsError> {
    let object = value.as_object().ok_or(PrefsError::NotAnObject)?;
    let mut patch = Patch::default();
    for (key, field) in object {
        match key.as_str() {
            "muted" => patch.muted = Some(bool_field(field, "muted")?),
            "hidden" => patch.hidden = Some(bool_field(field, "hidden")?),
            "skin" => patch.skin = Some(skin_field(field)?),
            "speech" => patch.speech = Some(speech_field(field)?),
            _ => return Err(PrefsError::UnknownField),
        }
    }
    Ok(patch)
}

fn bool_field(field: &Value, name: &'static str) -> Result<bool, PrefsError> {
    field.as_bool().ok_or(PrefsError::WrongType(name))
}

fn skin_field(field: &Value) -> Result<Skin, PrefsError> {
    match field.as_str() {
        Some("jack-o-lantern") => Ok(Skin::JackOLantern),
        Some(_) => Err(PrefsError::BadValue("skin")),
        None => Err(PrefsError::WrongType("skin")),
    }
}

fn speech_field(field: &Value) -> Result<Speech, PrefsError> {
    match field.as_str() {
        Some("auto") => Ok(Speech::Auto),
        Some("off") => Ok(Speech::Off),
        Some(_) => Err(PrefsError::BadValue("speech")),
        None => Err(PrefsError::WrongType("speech")),
    }
}

/// Reads the stored preferences. A missing or malformed field falls back to its default, so one bad
/// value never hides the others.
#[must_use]
pub fn from_stored(stored: &Value) -> FamiliarPrefs {
    let defaults = FamiliarPrefs::default();
    let Some(object) = stored.as_object() else {
        return defaults;
    };
    FamiliarPrefs {
        muted: stored_bool(object, "muted").unwrap_or(defaults.muted),
        hidden: stored_bool(object, "hidden").unwrap_or(defaults.hidden),
        skin: skin_field(object.get("skin").unwrap_or(&Value::Null)).unwrap_or(defaults.skin),
        speech: speech_field(object.get("speech").unwrap_or(&Value::Null))
            .unwrap_or(defaults.speech),
    }
}

fn stored_bool(object: &Map<String, Value>, key: &str) -> Option<bool> {
    object.get(key).and_then(Value::as_bool)
}

/// Applies a patch to the stored preferences. Fields the patch does not set keep their stored value.
#[must_use]
pub fn merge(stored: &Value, patch: Patch) -> FamiliarPrefs {
    let current = from_stored(stored);
    FamiliarPrefs {
        muted: patch.muted.unwrap_or(current.muted),
        hidden: patch.hidden.unwrap_or(current.hidden),
        skin: patch.skin.unwrap_or(current.skin),
        speech: patch.speech.unwrap_or(current.speech),
    }
}

/// The JSON object stored in `users.familiar_prefs`. It always holds all four fields.
#[must_use]
pub fn to_json(prefs: &FamiliarPrefs) -> Value {
    json!({
        "muted": prefs.muted,
        "hidden": prefs.hidden,
        "skin": "jack-o-lantern",
        "speech": match prefs.speech {
            Speech::Auto => "auto",
            Speech::Off => "off",
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accepted(value: &Value) -> Patch {
        validate_patch(value).expect("patch should be accepted")
    }

    #[test]
    fn defaults_are_the_jack_o_lantern_with_speech_on_and_nothing_muted() {
        let defaults = FamiliarPrefs::default();
        assert!(!defaults.muted);
        assert!(!defaults.hidden);
        assert_eq!(defaults.skin, Skin::JackOLantern);
        assert_eq!(defaults.speech, Speech::Auto);
    }

    #[test]
    fn an_empty_object_is_a_valid_patch_that_sets_nothing() {
        assert_eq!(accepted(&json!({})), Patch::default());
    }

    #[test]
    fn each_allowed_field_is_read_into_its_slot() {
        let patch = accepted(
            &json!({"muted": true, "hidden": true, "skin": "jack-o-lantern", "speech": "off"}),
        );
        assert_eq!(patch.muted, Some(true));
        assert_eq!(patch.hidden, Some(true));
        assert_eq!(patch.skin, Some(Skin::JackOLantern));
        assert_eq!(patch.speech, Some(Speech::Off));
    }

    #[test]
    fn an_unknown_key_is_refused() {
        assert_eq!(
            validate_patch(&json!({"volume": 3})),
            Err(PrefsError::UnknownField)
        );
    }

    #[test]
    fn a_wrong_type_is_refused_with_the_field_name() {
        assert_eq!(
            validate_patch(&json!({"muted": "yes"})),
            Err(PrefsError::WrongType("muted"))
        );
        assert_eq!(
            validate_patch(&json!({"speech": true})),
            Err(PrefsError::WrongType("speech"))
        );
    }

    #[test]
    fn a_value_outside_the_allowed_set_is_refused() {
        assert_eq!(
            validate_patch(&json!({"skin": "ghost"})),
            Err(PrefsError::BadValue("skin"))
        );
        assert_eq!(
            validate_patch(&json!({"speech": "on"})),
            Err(PrefsError::BadValue("speech"))
        );
    }

    #[test]
    fn a_body_that_is_not_an_object_is_refused() {
        assert_eq!(validate_patch(&json!([true])), Err(PrefsError::NotAnObject));
        assert_eq!(
            validate_patch(&json!("muted")),
            Err(PrefsError::NotAnObject)
        );
    }

    #[test]
    fn a_body_over_one_kibibyte_is_refused_before_parsing() {
        assert_eq!(check_size(MAX_PATCH_BYTES), Ok(()));
        assert_eq!(check_size(MAX_PATCH_BYTES + 1), Err(PrefsError::TooLarge));
    }

    #[test]
    fn merge_keeps_stored_fields_the_patch_does_not_name() {
        let stored =
            json!({"muted": true, "hidden": false, "skin": "jack-o-lantern", "speech": "auto"});
        let merged = merge(&stored, accepted(&json!({"hidden": true})));
        assert!(merged.muted, "muted should stay as stored");
        assert!(merged.hidden, "hidden should take the patch value");
        assert_eq!(merged.speech, Speech::Auto);
    }

    #[test]
    fn merge_fills_defaults_for_a_stored_empty_object() {
        let merged = merge(&json!({}), Patch::default());
        assert_eq!(merged, FamiliarPrefs::default());
    }

    #[test]
    fn a_malformed_stored_value_falls_back_to_its_default_without_hiding_the_others() {
        let stored = json!({"muted": "maybe", "speech": "off"});
        let prefs = from_stored(&stored);
        assert!(!prefs.muted, "a bad stored bool falls back to the default");
        assert_eq!(prefs.speech, Speech::Off, "a good stored value is kept");
    }

    #[test]
    fn to_json_writes_exactly_the_four_fields() {
        let value = to_json(&FamiliarPrefs {
            muted: true,
            ..FamiliarPrefs::default()
        });
        let object = value.as_object().expect("object");
        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["hidden", "muted", "skin", "speech"]);
        assert_eq!(value["muted"], json!(true));
        assert_eq!(value["skin"], json!("jack-o-lantern"));
    }
}
