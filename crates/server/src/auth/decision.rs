//! The two rules that decide sign-in and linking, as pure functions.
//!
//! No database, clock or network is used here, so the rules are cheap to test on every
//! combination of inputs. The "no automatic linking" rule is enforced by the types: an
//! [`OAuthDecision::SignIn`] can only name the owner of the provider identity, never the owner
//! of a matching email address.

use counterspell_common::ids::UserId;

use super::email::EmailAddress;

/// What the caller found out about the email address on the provider's profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileEmail {
    /// The provider gave no verified email address.
    Absent,
    /// A verified address that no account uses.
    Free(EmailAddress),
    /// A verified address that already belongs to an account.
    Taken,
}

/// What to do after a provider sign-in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuthDecision {
    /// The provider identity is already linked. Sign in to its owner.
    SignIn(UserId),
    /// Create a new account, with the verified email when there is one.
    CreateAccount {
        /// The verified email to store, if any.
        email: Option<EmailAddress>,
    },
    /// The email belongs to another account. Refuse, with no session and no write.
    RefuseEmailTaken,
}

/// Who is trying to link a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    /// The signed-in account.
    pub id: UserId,
    /// Whether this account is the judge demo account.
    pub is_demo: bool,
}

/// What to do when a signed-in user links a provider identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkDecision {
    /// Add the identity to the signed-in account.
    Link,
    /// The identity is already linked to this account. Nothing changes.
    AlreadyLinkedHere,
    /// The identity belongs to another account. Refuse.
    RefuseLinkedElsewhere,
    /// This account already has an identity for this provider. Refuse.
    RefuseProviderAlreadyLinked,
    /// The demo account may not link anything. Refuse.
    RefuseDemo,
}

/// Decides what a provider sign-in means. The provider identity alone can select an account.
#[must_use]
pub fn decide_oauth_login(identity_owner: Option<UserId>, email: ProfileEmail) -> OAuthDecision {
    match identity_owner {
        Some(user) => OAuthDecision::SignIn(user),
        None => match email {
            ProfileEmail::Taken => OAuthDecision::RefuseEmailTaken,
            ProfileEmail::Free(address) => OAuthDecision::CreateAccount {
                email: Some(address),
            },
            ProfileEmail::Absent => OAuthDecision::CreateAccount { email: None },
        },
    }
}

/// Decides whether a provider identity may be linked to the signed-in account.
///
/// The checks run in this order: demo first, then already linked here, then linked elsewhere,
/// then the account already has this provider, and otherwise link.
#[must_use]
pub fn decide_link(
    actor: &Actor,
    identity_owner: Option<UserId>,
    actor_has_provider: bool,
) -> LinkDecision {
    if actor.is_demo {
        return LinkDecision::RefuseDemo;
    }
    match identity_owner {
        Some(owner) if owner == actor.id => LinkDecision::AlreadyLinkedHere,
        Some(_) => LinkDecision::RefuseLinkedElsewhere,
        None if actor_has_provider => LinkDecision::RefuseProviderAlreadyLinked,
        None => LinkDecision::Link,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn email() -> EmailAddress {
        EmailAddress::parse("owner@example.co").expect("valid")
    }

    fn profile_emails() -> Vec<ProfileEmail> {
        vec![
            ProfileEmail::Absent,
            ProfileEmail::Free(email()),
            ProfileEmail::Taken,
        ]
    }

    #[test]
    fn a_known_identity_always_signs_in_to_its_owner_whatever_the_email_says() {
        let owner = UserId::new();
        for email in profile_emails() {
            assert_eq!(
                decide_oauth_login(Some(owner), email),
                OAuthDecision::SignIn(owner)
            );
        }
    }

    #[test]
    fn an_unknown_identity_with_a_taken_email_is_refused() {
        assert_eq!(
            decide_oauth_login(None, ProfileEmail::Taken),
            OAuthDecision::RefuseEmailTaken
        );
    }

    #[test]
    fn an_unknown_identity_with_a_free_or_missing_email_creates_an_account() {
        assert_eq!(
            decide_oauth_login(None, ProfileEmail::Free(email())),
            OAuthDecision::CreateAccount {
                email: Some(email())
            }
        );
        assert_eq!(
            decide_oauth_login(None, ProfileEmail::Absent),
            OAuthDecision::CreateAccount { email: None }
        );
    }

    #[test]
    fn link_checks_run_in_the_documented_order() {
        let me = UserId::new();
        let other = UserId::new();
        let demo = Actor {
            id: me,
            is_demo: true,
        };
        let normal = Actor {
            id: me,
            is_demo: false,
        };

        assert_eq!(decide_link(&demo, Some(me), true), LinkDecision::RefuseDemo);
        assert_eq!(
            decide_link(&normal, Some(me), true),
            LinkDecision::AlreadyLinkedHere
        );
        assert_eq!(
            decide_link(&normal, Some(other), false),
            LinkDecision::RefuseLinkedElsewhere
        );
        assert_eq!(
            decide_link(&normal, None, true),
            LinkDecision::RefuseProviderAlreadyLinked
        );
        assert_eq!(decide_link(&normal, None, false), LinkDecision::Link);
    }

    proptest! {
        #[test]
        fn sign_in_never_names_anyone_but_the_identity_owner(
            known in proptest::bool::ANY,
            taken in proptest::bool::ANY,
        ) {
            let owner = UserId::new();
            let identity = known.then_some(owner);
            let email = if taken { ProfileEmail::Taken } else { ProfileEmail::Absent };
            match decide_oauth_login(identity, email) {
                OAuthDecision::SignIn(user) => prop_assert_eq!(Some(user), identity),
                OAuthDecision::RefuseEmailTaken => {
                    prop_assert!(identity.is_none() && taken);
                }
                OAuthDecision::CreateAccount { .. } => {
                    prop_assert!(identity.is_none() && !taken);
                }
            }
        }

        #[test]
        fn a_demo_account_is_always_refused_when_linking(
            owner_is_actor in proptest::bool::ANY,
            actor_has_provider in proptest::bool::ANY,
        ) {
            let actor = Actor { id: UserId::new(), is_demo: true };
            let owner = owner_is_actor.then_some(actor.id);
            prop_assert_eq!(decide_link(&actor, owner, actor_has_provider), LinkDecision::RefuseDemo);
        }
    }
}
