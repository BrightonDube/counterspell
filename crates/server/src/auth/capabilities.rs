//! What an account may do. The demo account's limits live only here.
//!
//! Every mutating handler calls [`Capabilities::require`] before it does any work. No other code
//! checks whether a user is the demo account.

use super::errors::AuthError;

/// One action that the demo account is not allowed to take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// Link a GitHub or Google identity to the account.
    LinkProvider,
    /// Add an email address to the account.
    LinkEmail,
    /// Set or change a password.
    ChangeCredentials,
    /// Delete the account.
    DeleteAccount,
    /// Sign out of every session at once.
    LogoutEverywhere,
}

/// The actions an account may take. The UI reads this same struct from `/api/me`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "each flag is one named action that the UI reads separately from /api/me"
)]
pub struct Capabilities {
    /// May link a provider identity.
    pub link_provider: bool,
    /// May add an email address.
    pub link_email: bool,
    /// May change its password.
    pub change_credentials: bool,
    /// May delete the account.
    pub delete_account: bool,
    /// May sign out everywhere.
    pub logout_everywhere: bool,
}

impl Capabilities {
    /// The capabilities of an account. The demo account has none of them.
    #[must_use]
    pub fn for_user(is_demo: bool) -> Self {
        let granted = !is_demo;
        Self {
            link_provider: granted,
            link_email: granted,
            change_credentials: granted,
            delete_account: granted,
            logout_everywhere: granted,
        }
    }

    /// Returns `Ok` when the capability is granted, otherwise the `403` demo error.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::DemoAccountRestricted`] when the capability is not granted.
    pub fn require(&self, capability: Capability) -> Result<(), AuthError> {
        let granted = match capability {
            Capability::LinkProvider => self.link_provider,
            Capability::LinkEmail => self.link_email,
            Capability::ChangeCredentials => self.change_credentials,
            Capability::DeleteAccount => self.delete_account,
            Capability::LogoutEverywhere => self.logout_everywhere,
        };
        if granted {
            Ok(())
        } else {
            Err(AuthError::DemoAccountRestricted)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Capability; 5] = [
        Capability::LinkProvider,
        Capability::LinkEmail,
        Capability::ChangeCredentials,
        Capability::DeleteAccount,
        Capability::LogoutEverywhere,
    ];

    #[test]
    fn the_demo_account_has_no_capabilities() {
        let demo = Capabilities::for_user(true);
        assert_eq!(
            [
                demo.link_provider,
                demo.link_email,
                demo.change_credentials,
                demo.delete_account,
                demo.logout_everywhere
            ],
            [false; 5]
        );
        for capability in ALL {
            assert_eq!(
                demo.require(capability),
                Err(AuthError::DemoAccountRestricted)
            );
        }
    }

    #[test]
    fn an_ordinary_account_has_every_capability() {
        let user = Capabilities::for_user(false);
        assert_eq!(
            [
                user.link_provider,
                user.link_email,
                user.change_credentials,
                user.delete_account,
                user.logout_everywhere
            ],
            [true; 5]
        );
        for capability in ALL {
            assert_eq!(user.require(capability), Ok(()));
        }
    }
}
