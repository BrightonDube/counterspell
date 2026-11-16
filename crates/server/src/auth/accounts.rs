//! Account helpers shared by the sign-in handlers.

use uuid::Uuid;

/// A random handle that matches the database rule: lower case, starts with a letter or digit.
/// Handles are never shown as proof of identity; users can rename their display name later.
#[must_use]
pub fn new_handle() -> String {
    let simple = Uuid::new_v4().simple().to_string();
    format!("p{}", &simple[..12])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_handles_match_the_database_rule() {
        let handle = new_handle();
        assert_eq!(handle.len(), 13);
        assert!(handle
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }
}
