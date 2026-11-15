//! Test versions of the seams. Compiled only with the `testkit` feature, so release builds never
//! contain a way to sign in with a header.

use axum::http::{header::AUTHORIZATION, HeaderMap};
use counterspell_common::ids::UserId;
use uuid::Uuid;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::seams::{ProblemLookup, ProblemRef, UserResolver};

// test-change: CAT-F1. Test-only seeding for submissions_problem_fk.
// test-change: CAT-F1. Test-only seeding for submissions_problem_fk.
/// Makes sure the `two-sum` test problem exists in the catalogue, because submissions must reference a
/// real problem (CAT-F1). Calling it again does nothing.
///
/// # Errors
/// Returns the database error if an insert fails.
pub async fn ensure_problem(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO tracks (slug, name) VALUES ('arrays', 'Arrays') ON CONFLICT DO NOTHING",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO problems (slug, title, track_slug, difficulty) VALUES ('two-sum', 'Two Sum', 'arrays', 'easy') \
         ON CONFLICT DO NOTHING",
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Makes sure a `users` row exists for `user`, so that tests can save runs for it. The row has no
/// email and no password. Calling it again for the same id does nothing.
///
/// # Errors
/// Returns the database error if the insert fails for a reason other than the row already existing.
pub async fn ensure_user(pool: &PgPool, user: UserId) -> Result<(), sqlx::Error> {
    ensure_problem(pool).await?;
    let simple = user.as_uuid().simple().to_string();
    sqlx::query(
        "INSERT INTO users (id, handle, display_name) VALUES ($1, $2, 'Test user')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(user.as_uuid())
    .bind(simple)
    .execute(pool)
    .await
    .map(|_| ())
}

/// Accepts `Authorization: Bearer test-user-<n>` and maps `<n>` to a fixed user id.
#[derive(Debug, Clone, Copy, Default)]
pub struct BearerTestResolver;

#[async_trait]
impl UserResolver for BearerTestResolver {
    async fn resolve(&self, headers: &HeaderMap) -> Option<UserId> {
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
