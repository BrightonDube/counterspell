//! The small profile module owned by auth: validated input and the single write path.

use counterspell_common::ids::UserId;
use sqlx::PgPool;
use time::OffsetDateTime;

use super::errors::AuthError;

/// The static placeholder until real avatar storage exists.
pub const AVATAR_URL: &str = "/art/creatures/skull-green-eyes.webp";

/// The validated profile fields the user may change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfilePatch {
    /// Display name after trimming, 1 to 80 characters.
    pub display_name: String,
    /// Uppercase two-letter country code, or `None`.
    pub country: Option<String>,
}

impl ProfilePatch {
    /// Normalises and validates browser input.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::InvalidJson`] when the display name or country is outside the table's
    /// public contract.
    pub fn try_new(display_name: &str, country: Option<&str>) -> Result<Self, AuthError> {
        let display_name = display_name.trim();
        if display_name.is_empty() || display_name.chars().count() > 80 {
            return Err(AuthError::InvalidJson);
        }
        let country = country
            .map(str::trim)
            .filter(|code| !code.is_empty())
            .map(str::to_ascii_uppercase);
        if country
            .as_deref()
            .is_some_and(|code| !code.bytes().all(|byte| byte.is_ascii_uppercase()))
            || country.as_deref().is_some_and(|code| code.len() != 2)
        {
            return Err(AuthError::InvalidJson);
        }
        Ok(Self {
            display_name: display_name.to_owned(),
            country,
        })
    }
}

/// Writes the profile and marks onboarding complete.
///
/// # Errors
///
/// Returns the database error when the update fails.
pub async fn update_profile(
    pool: &PgPool,
    user: UserId,
    patch: &ProfilePatch,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users
         SET display_name = $2,
             country = $3,
             profile_completed_at = $4,
             updated_at = $4
         WHERE id = $1",
    )
    .bind(user.as_uuid())
    .bind(&patch.display_name)
    .bind(&patch.country)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // test-change: PO-1
    #[test]
    fn profile_input_is_trimmed_and_country_is_uppercased() {
        assert_eq!(
            ProfilePatch::try_new("  Brighton  ", Some(" za ")).expect("patch"),
            ProfilePatch {
                display_name: "Brighton".to_owned(),
                country: Some("ZA".to_owned())
            }
        );
    }

    // test-change: PO-1
    #[test]
    fn blank_country_means_no_country() {
        assert_eq!(
            ProfilePatch::try_new("Brighton", Some("   "))
                .expect("patch")
                .country,
            None
        );
    }

    // test-change: PO-1
    #[test]
    fn malformed_profile_input_is_rejected() {
        for (name, country) in [
            ("", None),
            ("   ", None),
            ("Brighton", Some("South Africa")),
            ("Brighton", Some("Z1")),
            ("Brighton", Some("Z")),
        ] {
            assert_eq!(
                ProfilePatch::try_new(name, country),
                Err(AuthError::InvalidJson)
            );
        }
        assert_eq!(
            ProfilePatch::try_new(&"x".repeat(81), None),
            Err(AuthError::InvalidJson)
        );
    }
}
