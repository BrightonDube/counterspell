//! Email addresses as a type that can only hold valid, normalised values.
//!
//! The parser is the only way to make an [`EmailAddress`]. Mail senders and handlers receive
//! an already-checked address, so no code builds an address string by hand.

use std::fmt;

use thiserror::Error;

/// The longest address the parser accepts, in bytes.
pub const MAX_EMAIL_LEN: usize = 254;

/// The address is not one we accept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("the email address is not valid")]
pub struct EmailError;

/// A valid, lower-cased email address. It is never printed in full by `Debug`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Checks and normalises an address: trims, lower-cases, and enforces the rules of R4.3.
    ///
    /// Dots and `+tags` are kept, so two different mailboxes stay different (R4.4).
    ///
    /// # Errors
    ///
    /// Returns [`EmailError`] when the input breaks any rule.
    pub fn parse(input: &str) -> Result<Self, EmailError> {
        let normalised = input.trim().to_ascii_lowercase();
        if normalised.len() > MAX_EMAIL_LEN || !input.trim().is_ascii() {
            return Err(EmailError);
        }
        let forbidden = normalised.chars().any(|c| {
            c.is_whitespace() || c.is_control() || matches!(c, '<' | '>' | ',' | ';' | '"')
        });
        if forbidden {
            return Err(EmailError);
        }
        let (local, domain) = normalised.split_once('@').ok_or(EmailError)?;
        let one_at_only = !domain.contains('@');
        let domain_ok = domain.contains('.') && domain.split('.').all(|label| !label.is_empty());
        if local.is_empty() || !one_at_only || !domain_ok {
            return Err(EmailError);
        }
        Ok(Self(normalised))
    }

    /// The normalised address text. Use it only where the address is needed (for example in a
    /// database row or a mail `to` field), never in a log line.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for EmailAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EmailAddress([redacted])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn accepted(input: &str) -> String {
        EmailAddress::parse(input)
            .unwrap_or_else(|_| panic!("expected {input:?} to be accepted"))
            .as_str()
            .to_owned()
    }

    fn refused(input: &str) {
        assert_eq!(EmailAddress::parse(input), Err(EmailError), "{input:?}");
    }

    #[test]
    fn a_plain_address_is_trimmed_and_lower_cased() {
        assert_eq!(accepted("  Thandi@Example.CO  "), "thandi@example.co");
        assert_eq!(accepted("A@B.CO"), "a@b.co");
    }

    #[test]
    fn plus_tags_and_dots_are_kept_so_mailboxes_stay_distinct() {
        assert_eq!(accepted("a+x@b.co"), "a+x@b.co");
        assert_ne!(accepted("a.b@x.co"), accepted("ab@x.co"));
    }

    #[test]
    fn the_hostile_corpus_is_refused() {
        for input in [
            "a@b",
            "a@@b.co",
            "a b@c.co",
            "a@b.co\r\nBcc: x@y.z",
            "\"a\"@b.co",
            "a@b.co,c@d.co",
            "a<b>@c.co",
            "a@b;c.co",
            "a@.co",
            "a@b..co",
            "@b.co",
            "a@",
            "ünï@b.co",
            "",
            "   ",
        ] {
            refused(input);
        }
    }

    #[test]
    fn the_length_limit_is_254_bytes() {
        let at_limit = format!("{}@b.co", "a".repeat(249));
        assert_eq!(accepted(&at_limit).len(), 254);
        refused(&format!("{}@b.co", "a".repeat(250)));
    }

    proptest! {
        #[test]
        fn parsing_is_idempotent(input in "\\PC{0,300}") {
            if let Ok(first) = EmailAddress::parse(&input) {
                let second = EmailAddress::parse(first.as_str());
                prop_assert_eq!(second, Ok(first));
            }
        }

        #[test]
        fn accepted_values_follow_every_rule(input in "[ -~]{0,80}") {
            if let Ok(email) = EmailAddress::parse(&input) {
                let text = email.as_str();
                prop_assert!(text.len() <= MAX_EMAIL_LEN);
                prop_assert!(text.is_ascii());
                prop_assert_eq!(text.matches('@').count(), 1);
                prop_assert!(!text.chars().any(|c| c.is_whitespace() || c.is_control()));
                prop_assert!(!text.contains(['<', '>', ',', ';', '"']));
                prop_assert_eq!(text, text.to_ascii_lowercase());
                let (local, domain) = text.split_once('@').expect("one @");
                prop_assert!(!local.is_empty());
                prop_assert!(domain.contains('.'));
                prop_assert!(domain.split('.').all(|label| !label.is_empty()));
            }
        }
    }
}
