//! Identifiers for users and runs. Ids are UUID values, shown as text.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Declares a UUID-backed id type with random creation, text parsing and display.
macro_rules! uuid_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Makes a new random (version 4) id.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Wraps an existing UUID.
            #[must_use]
            pub const fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            /// Returns the underlying UUID.
            #[must_use]
            pub const fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(text).map(Self)
            }
        }
    };
}

uuid_id! {
    /// Identifies one user account.
    UserId
}

uuid_id! {
    /// Identifies one run, which is one saved submission.
    RunId
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_id_round_trips_through_text() {
        let text = "3f2b8c1e-9d4a-4b7e-8a61-2c5d9e0f1a2b";
        let id: RunId = text.parse().expect("valid uuid text");
        assert_eq!(id.to_string(), text);
    }

    #[test]
    fn run_id_serialises_as_a_plain_string() {
        let id: RunId = "3f2b8c1e-9d4a-4b7e-8a61-2c5d9e0f1a2b"
            .parse()
            .expect("valid uuid text");
        let json = serde_json::to_string(&id).expect("serialise run id");
        assert_eq!(json, "\"3f2b8c1e-9d4a-4b7e-8a61-2c5d9e0f1a2b\"");
    }

    #[test]
    fn new_run_id_is_a_version_four_uuid() {
        let id = RunId::new();
        assert_eq!(id.as_uuid().get_version_num(), 4);
    }

    #[test]
    fn user_id_refuses_text_that_is_not_a_uuid() {
        assert!("not-a-uuid".parse::<UserId>().is_err());
    }
}
