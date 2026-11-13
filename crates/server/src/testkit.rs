//! Test versions of the seams. Compiled only with the `testkit` feature, so release builds never
//! contain a way to sign in with a header.

use axum::http::{header::AUTHORIZATION, HeaderMap};
use counterspell_common::ids::UserId;
use uuid::Uuid;

use crate::seams::{ProblemLookup, ProblemRef, UserResolver};

/// Accepts `Authorization: Bearer test-user-<n>` and maps `<n>` to a fixed user id.
#[derive(Debug, Clone, Copy, Default)]
pub struct BearerTestResolver;

impl UserResolver for BearerTestResolver {
    fn resolve(&self, headers: &HeaderMap) -> Option<UserId> {
        let value = headers.get(AUTHORIZATION)?.to_str().ok()?;
        let number = value.strip_prefix("Bearer test-user-")?;
        let number: u128 = number.parse().ok()?;
        Some(UserId::from_uuid(Uuid::from_u128(number)))
    }
}

/// A fixed list of published problem slugs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InMemoryProblems {
    slugs: Vec<String>,
}

impl InMemoryProblems {
    /// Publishes exactly the given slugs.
    #[must_use]
    pub fn new(slugs: &[&str]) -> Self {
        Self {
            slugs: slugs.iter().map(|slug| (*slug).to_owned()).collect(),
        }
    }
}

impl ProblemLookup for InMemoryProblems {
    fn published(&self, slug: &str) -> Option<ProblemRef> {
        self.slugs
            .iter()
            .any(|known| known == slug)
            .then(|| ProblemRef {
                slug: slug.to_owned(),
            })
    }
}
