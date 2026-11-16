//! Hint settings, read once at start-up.
//!
//! [`HintConfig::from_map`] does all parsing and checks and does no I/O, so tests pass a map and
//! never touch the process environment. [`HintConfig::from_env`] only collects the map.

use std::{collections::HashMap, fmt, ops::RangeInclusive};

use thiserror::Error;

/// Default `HINT_PROVIDERS`: `openai` first, `groq` as the fallback.
const DEFAULT_PROVIDERS: &str = "openai:gpt-6-luna,groq:openai/gpt-oss-20b";
/// Default OpenAI-compatible base URL.
const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
/// Default Groq OpenAI-compatible base URL.
const DEFAULT_GROQ_BASE_URL: &str = "https://api.groq.com/openai/v1";
/// Default timeout for one provider attempt, in milliseconds.
const DEFAULT_TIMEOUT_MS: u64 = 8_000;
/// Smallest and largest accepted per-attempt timeout, in milliseconds.
const TIMEOUT_RANGE_MS: RangeInclusive<u64> = 1_000..=15_000;
/// Default cap on provider output, in tokens.
const DEFAULT_MAX_OUTPUT_TOKENS: u64 = 300;
/// Default AI generations per user per UTC day.
const DEFAULT_DAILY_LIMIT: u64 = 10;
/// Default global spend cap per UTC day, in US dollars.
const DEFAULT_SPEND_CAP_DOLLARS: &str = "5.00";
/// Default hint requests per user per window.
const DEFAULT_WINDOW_LIMIT: u64 = 30;

/// A value that must never be printed. `Debug` and `Display` both print `[redacted]`.
#[derive(Clone, PartialEq, Eq)]
pub struct Redacted<T>(T);

impl<T> Redacted<T> {
    /// Wraps a secret value.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// The wrapped value. Call this only where the secret is used, never for logging.
    #[must_use]
    pub fn expose(&self) -> &T {
        &self.0
    }
}

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[redacted]")
    }
}

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[redacted]")
    }
}

/// The providers this build knows how to call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderName {
    /// `OpenAI`, the primary provider by default.
    OpenAi,
    /// `Groq`, the fallback provider by default.
    Groq,
}

impl ProviderName {
    /// The name used in `HINT_PROVIDERS` and in responses.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Groq => "groq",
        }
    }

    /// The environment variable that holds this provider's API key.
    #[must_use]
    pub fn key_variable(self) -> &'static str {
        match self {
            Self::OpenAi => "OPENAI_API_KEY",
            Self::Groq => "GROQ_API_KEY",
        }
    }

    /// The variables that hold this provider's input and output prices.
    fn price_variables(self) -> (&'static str, &'static str) {
        match self {
            Self::OpenAi => ("HINT_PRICE_OPENAI_IN_PER_M", "HINT_PRICE_OPENAI_OUT_PER_M"),
            Self::Groq => ("HINT_PRICE_GROQ_IN_PER_M", "HINT_PRICE_GROQ_OUT_PER_M"),
        }
    }

    /// Default input and output prices, in US dollars per million tokens.
    fn default_prices(self) -> (&'static str, &'static str) {
        match self {
            Self::OpenAi => ("0.10", "0.50"),
            Self::Groq => ("0.075", "0.30"),
        }
    }

    /// Reads a provider name from `HINT_PROVIDERS`. Names are case sensitive.
    fn parse(name: &str) -> Result<Self, HintConfigError> {
        match name {
            "openai" => Ok(Self::OpenAi),
            "groq" => Ok(Self::Groq),
            other => Err(HintConfigError::UnknownProvider {
                name: other.to_owned(),
            }),
        }
    }
}

/// Prices for one provider, in micro-dollars (millionths of a US dollar) per million tokens.
/// Integer units keep every cost sum exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Price {
    /// Price of one million input tokens.
    pub input_per_million_micros: i64,
    /// Price of one million output tokens.
    pub output_per_million_micros: i64,
}

/// One configured provider, with its key kept out of every log and debug print.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSpec {
    /// Which provider this is.
    pub name: ProviderName,
    /// The model name sent in each request.
    pub model: String,
    /// The OpenAI-compatible base URL, without a trailing `/chat/completions`.
    pub base_url: String,
    /// The API key. Redacted in every print.
    pub api_key: Redacted<String>,
    /// The prices used to record cost.
    pub price: Price,
}

/// All hint settings, parsed and checked. Build it with [`HintConfig::from_map`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HintConfig {
    /// The ordered provider list. The first entry is the primary. Empty means AI is off.
    pub providers: Vec<ProviderSpec>,
    /// Timeout for each provider attempt, in milliseconds.
    pub timeout_ms: u64,
    /// The `max_tokens` sent to each provider.
    pub max_output_tokens: u32,
    /// AI generations allowed per user per UTC day. The familiar shares this limit.
    pub daily_limit: u32,
    /// Global spend cap per UTC day, in micro-dollars.
    pub spend_cap_micros: i64,
    /// Hint requests allowed per user in each rate-limit window.
    pub window_limit: u64,
}

/// A setting is missing or unusable. Messages name the variable and never print its value.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HintConfigError {
    /// A provider is listed but its key variable is not set, and strict mode is on.
    #[error("environment variable {name} is required because a hint provider is listed")]
    MissingKey {
        /// The key variable that is missing.
        name: &'static str,
    },
    /// `HINT_PROVIDERS` names a provider this build does not know.
    #[error("HINT_PROVIDERS names an unknown provider: {name}")]
    UnknownProvider {
        /// The provider name as written.
        name: String,
    },
    /// A variable is set but cannot be used.
    #[error("environment variable {name} is invalid: {reason}")]
    Invalid {
        /// The variable name.
        name: &'static str,
        /// What is wrong, in plain words.
        reason: String,
    },
}

impl HintConfig {
    /// Settings with AI off: no providers, and the documented defaults for every limit.
    /// The spend cap default is `5.00` dollars, matching `DEFAULT_SPEND_CAP_DOLLARS`.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            providers: Vec::new(),
            timeout_ms: DEFAULT_TIMEOUT_MS,
            max_output_tokens: u32::try_from(DEFAULT_MAX_OUTPUT_TOKENS).unwrap_or(u32::MAX),
            daily_limit: u32::try_from(DEFAULT_DAILY_LIMIT).unwrap_or(u32::MAX),
            spend_cap_micros: 5_000_000,
            window_limit: DEFAULT_WINDOW_LIMIT,
        }
    }

    /// Reads the hint variables from the process environment.
    ///
    /// Only the variables this module uses are collected, so no other secret is copied.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`HintConfig::from_map`].
    pub fn from_env() -> Result<Self, HintConfigError> {
        let vars: HashMap<String, String> = std::env::vars()
            .filter(|(name, _)| {
                name.starts_with("HINT_")
                    || name.starts_with("AI_")
                    || matches!(
                        name.as_str(),
                        "OPENAI_API_KEY" | "GROQ_API_KEY" | "OPENAI_BASE_URL" | "GROQ_BASE_URL"
                    )
            })
            .collect();
        Self::from_map(&vars)
    }

    /// Parses and checks the hint settings from a map of variable names to values.
    ///
    /// Missing variables take their defaults. With `AI_STRICT_CONFIG=true` (the default), a
    /// listed provider without a key is an error. With `false`, that provider is left out and a
    /// warning is logged.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown provider, a missing key in strict mode, or a value outside
    /// its allowed range.
    pub fn from_map(vars: &HashMap<String, String>) -> Result<Self, HintConfigError> {
        let list = vars
            .get("HINT_PROVIDERS")
            .map_or(DEFAULT_PROVIDERS, String::as_str);
        let strict = read_bool(vars, "AI_STRICT_CONFIG", true)?;
        let mut providers = Vec::new();
        for entry in list.split(',').map(str::trim).filter(|e| !e.is_empty()) {
            let (name, model) = entry.split_once(':').ok_or(HintConfigError::Invalid {
                name: "HINT_PROVIDERS",
                reason: "each entry must be name:model".to_owned(),
            })?;
            let name = ProviderName::parse(name.trim())?;
            let model = model.trim();
            if model.is_empty() {
                return Err(HintConfigError::Invalid {
                    name: "HINT_PROVIDERS",
                    reason: "each entry needs a model name after the colon".to_owned(),
                });
            }
            let key = vars
                .get(name.key_variable())
                .map(|value| value.trim())
                .filter(|value| !value.is_empty());
            let Some(key) = key else {
                if strict {
                    return Err(HintConfigError::MissingKey {
                        name: name.key_variable(),
                    });
                }
                tracing::warn!(
                    provider = name.as_str(),
                    "hint provider left out: its API key variable is not set"
                );
                continue;
            };
            providers.push(ProviderSpec {
                name,
                model: model.to_owned(),
                base_url: read_base_url(vars, name)?,
                api_key: Redacted::new(key.to_owned()),
                price: read_price(vars, name)?,
            });
        }

        Ok(Self {
            providers,
            timeout_ms: read_number(
                vars,
                "HINT_TIMEOUT_MS",
                DEFAULT_TIMEOUT_MS,
                TIMEOUT_RANGE_MS,
            )?,
            max_output_tokens: to_u32(
                "HINT_MAX_OUTPUT_TOKENS",
                read_number(
                    vars,
                    "HINT_MAX_OUTPUT_TOKENS",
                    DEFAULT_MAX_OUTPUT_TOKENS,
                    1..=300,
                )?,
            )?,
            daily_limit: to_u32(
                "AI_DAILY_LIMIT",
                read_number(vars, "AI_DAILY_LIMIT", DEFAULT_DAILY_LIMIT, 0..=10_000)?,
            )?,
            spend_cap_micros: read_micro_dollars(
                vars,
                "AI_DAILY_SPEND_CAP_USD",
                DEFAULT_SPEND_CAP_DOLLARS,
            )?,
            window_limit: read_number(vars, "HINT_WINDOW_LIMIT", DEFAULT_WINDOW_LIMIT, 1..=1_000)?,
        })
    }
}

/// Reads a non-negative decimal dollar amount and returns micro-dollars. `"0.075"` gives `75_000`.
/// At most six decimal places are accepted, so the result is exact.
fn parse_micro_dollars(text: &str, name: &'static str) -> Result<i64, HintConfigError> {
    let invalid = |reason: &str| HintConfigError::Invalid {
        name,
        reason: reason.to_owned(),
    };
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if fraction.len() > 6 {
        return Err(invalid("use at most six decimal places"));
    }
    let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    if !digits(whole) || !digits(fraction) || (whole.is_empty() && fraction.is_empty()) {
        return Err(invalid("use a non-negative decimal number"));
    }
    let whole_value: i64 = if whole.is_empty() {
        0
    } else {
        whole
            .parse()
            .map_err(|_| invalid("the whole part is too large"))?
    };
    // Pad the fraction on the right to six digits, so "075" means 0.075000.
    let fraction_value: i64 = format!("{fraction:0<6}")
        .parse()
        .map_err(|_| invalid("the fraction is not a number"))?;
    whole_value
        .checked_mul(1_000_000)
        .and_then(|micros| micros.checked_add(fraction_value))
        .ok_or_else(|| invalid("the amount is too large"))
}

fn read_micro_dollars(
    vars: &HashMap<String, String>,
    name: &'static str,
    default: &str,
) -> Result<i64, HintConfigError> {
    parse_micro_dollars(vars.get(name).map_or(default, String::as_str), name)
}

fn read_price(
    vars: &HashMap<String, String>,
    name: ProviderName,
) -> Result<Price, HintConfigError> {
    let (default_in, default_out) = name.default_prices();
    let (input_name, output_name) = name.price_variables();
    Ok(Price {
        input_per_million_micros: read_micro_dollars(vars, input_name, default_in)?,
        output_per_million_micros: read_micro_dollars(vars, output_name, default_out)?,
    })
}

fn read_base_url(
    vars: &HashMap<String, String>,
    name: ProviderName,
) -> Result<String, HintConfigError> {
    let (variable, default) = match name {
        ProviderName::OpenAi => ("OPENAI_BASE_URL", DEFAULT_OPENAI_BASE_URL),
        ProviderName::Groq => ("GROQ_BASE_URL", DEFAULT_GROQ_BASE_URL),
    };
    let value = vars.get(variable).map_or(default, String::as_str).trim();
    if !value.starts_with("https://") {
        return Err(HintConfigError::Invalid {
            name: variable,
            reason: "the base URL must start with https://".to_owned(),
        });
    }
    Ok(value.trim_end_matches('/').to_owned())
}

fn read_number(
    vars: &HashMap<String, String>,
    name: &'static str,
    default: u64,
    range: RangeInclusive<u64>,
) -> Result<u64, HintConfigError> {
    let value = match vars.get(name) {
        None => return Ok(default),
        Some(text) => text
            .trim()
            .parse::<u64>()
            .map_err(|_| HintConfigError::Invalid {
                name,
                reason: "use a whole number".to_owned(),
            })?,
    };
    if range.contains(&value) {
        Ok(value)
    } else {
        Err(HintConfigError::Invalid {
            name,
            reason: format!("use a value from {} to {}", range.start(), range.end()),
        })
    }
}

fn to_u32(name: &'static str, value: u64) -> Result<u32, HintConfigError> {
    u32::try_from(value).map_err(|_| HintConfigError::Invalid {
        name,
        reason: "the value is too large".to_owned(),
    })
}

fn read_bool(
    vars: &HashMap<String, String>,
    name: &'static str,
    default: bool,
) -> Result<bool, HintConfigError> {
    match vars.get(name).map(|value| value.trim()) {
        None => Ok(default),
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(_) => Err(HintConfigError::Invalid {
            name,
            reason: "use true or false".to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect()
    }

    fn names(config: &HintConfig) -> Vec<&'static str> {
        config.providers.iter().map(|p| p.name.as_str()).collect()
    }

    #[test]
    fn defaults_list_openai_then_groq_with_their_models() {
        let config = HintConfig::from_map(&map(&[
            ("OPENAI_API_KEY", "sk-test-openai"),
            ("GROQ_API_KEY", "gsk-test-groq"),
        ]))
        .expect("the default list is valid when both keys are set");

        assert_eq!(names(&config), ["openai", "groq"]);
        assert_eq!(config.providers[0].model, "gpt-6-luna");
        assert_eq!(config.providers[1].model, "openai/gpt-oss-20b");
        assert_eq!(config.timeout_ms, 8_000);
        assert_eq!(config.max_output_tokens, 300);
        assert_eq!(config.daily_limit, 10);
        assert_eq!(config.spend_cap_micros, 5_000_000);
    }

    #[test]
    fn empty_provider_list_disables_ai() {
        let config =
            HintConfig::from_map(&map(&[("HINT_PROVIDERS", "")])).expect("an empty list is valid");

        assert_eq!(config.providers.len(), 0);
    }

    #[test]
    fn listed_provider_without_a_key_fails_when_strict() {
        let error = HintConfig::from_map(&map(&[("HINT_PROVIDERS", "openai:gpt-6-luna")]))
            .expect_err("strict mode refuses a provider with no key");

        assert_eq!(
            error,
            HintConfigError::MissingKey {
                name: "OPENAI_API_KEY"
            }
        );
    }

    #[test]
    fn keyless_provider_is_dropped_when_not_strict() {
        let config = HintConfig::from_map(&map(&[
            (
                "HINT_PROVIDERS",
                "openai:gpt-6-luna,groq:openai/gpt-oss-20b",
            ),
            ("GROQ_API_KEY", "gsk-test-groq"),
            ("AI_STRICT_CONFIG", "false"),
        ]))
        .expect("non-strict mode drops the provider without a key");

        assert_eq!(names(&config), ["groq"]);
    }

    #[test]
    fn unknown_provider_name_is_rejected() {
        let error = HintConfig::from_map(&map(&[("HINT_PROVIDERS", "gemini:flash-lite")]))
            .expect_err("only openai and groq are built");

        assert!(matches!(error, HintConfigError::UnknownProvider { .. }));
    }

    #[test]
    fn debug_output_never_shows_the_key() {
        let config = HintConfig::from_map(&map(&[
            ("HINT_PROVIDERS", "openai:gpt-6-luna"),
            ("OPENAI_API_KEY", "sk-secret-value-123"),
        ]))
        .expect("a provider with a key is valid");

        let printed = format!("{config:?}");

        assert!(!printed.contains("sk-secret-value-123"));
        assert!(printed.contains("[redacted]"));
    }

    #[test]
    fn timeout_outside_one_to_fifteen_seconds_is_rejected() {
        for value in ["500", "20000"] {
            let result =
                HintConfig::from_map(&map(&[("HINT_PROVIDERS", ""), ("HINT_TIMEOUT_MS", value)]));

            assert!(
                matches!(
                    result,
                    Err(HintConfigError::Invalid {
                        name: "HINT_TIMEOUT_MS",
                        ..
                    })
                ),
                "timeout {value} should be refused"
            );
        }
    }

    #[test]
    fn prices_parse_to_micro_dollars_per_million_tokens() {
        assert_eq!(parse_micro_dollars("0.075", "X"), Ok(75_000));
        assert_eq!(parse_micro_dollars("0.10", "X"), Ok(100_000));
        assert_eq!(parse_micro_dollars("5", "X"), Ok(5_000_000));
        assert!(parse_micro_dollars("-1", "X").is_err());
        assert!(parse_micro_dollars("0.0000001", "X").is_err());
    }

    #[test]
    fn prices_come_from_configuration_with_the_documented_defaults() {
        let config = HintConfig::from_map(&map(&[
            ("OPENAI_API_KEY", "sk-test-openai"),
            ("HINT_PROVIDERS", "openai:gpt-6-luna"),
            ("HINT_PRICE_OPENAI_IN_PER_M", "0.20"),
        ]))
        .expect("an overridden price is valid");

        assert_eq!(config.providers[0].price.input_per_million_micros, 200_000);
        assert_eq!(config.providers[0].price.output_per_million_micros, 500_000);
    }
}
