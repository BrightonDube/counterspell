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

#[async_trait]
impl ProblemLookup for InMemoryProblems {
    async fn published(&self, slug: &str) -> Option<ProblemRef> {
        self.slugs
            .iter()
            .any(|known| known == slug)
            .then(|| ProblemRef {
                slug: slug.to_owned(),
                version: 1,
            })
    }
}

/// One scripted answer for [`ScriptedTransport`].
pub enum Step {
    /// Answer with this result.
    Reply(Result<crate::hints::provider::ChatResponse, crate::hints::provider::ProviderError>),
    /// Never answer. The caller's timeout must end the call.
    Stall,
}

/// A chat transport that gives scripted answers in order and counts the calls. It makes no network
/// call. When the script runs out, it reports a transport failure.
pub struct ScriptedTransport {
    steps: std::sync::Mutex<std::collections::VecDeque<Step>>,
    calls: std::sync::atomic::AtomicUsize,
    prompts: std::sync::Mutex<Vec<String>>,
}

impl ScriptedTransport {
    /// Creates a transport that answers with `steps` in order.
    #[must_use]
    pub fn new(steps: Vec<Step>) -> Self {
        Self {
            steps: std::sync::Mutex::new(steps.into()),
            calls: std::sync::atomic::AtomicUsize::new(0),
            prompts: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Every prompt received so far, in order: the system rules, a blank line, then the user
    /// message. Tests use it to check what would have been sent to a provider.
    #[must_use]
    pub fn prompts(&self) -> Vec<String> {
        self.prompts
            .lock()
            .map(|prompts| prompts.clone())
            .unwrap_or_default()
    }

    /// How many calls the transport has received.
    #[must_use]
    pub fn calls(&self) -> usize {
        self.calls.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl crate::hints::provider::ChatTransport for ScriptedTransport {
    fn chat(
        &self,
        request: crate::hints::provider::ChatRequest<'_>,
    ) -> impl std::future::Future<
        Output = Result<
            crate::hints::provider::ChatResponse,
            crate::hints::provider::ProviderError,
        >,
    > + Send {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if let Ok(mut prompts) = self.prompts.lock() {
            prompts.push(format!("{}\n\n{}", request.system, request.user));
        }
        let step = self
            .steps
            .lock()
            .ok()
            .and_then(|mut steps| steps.pop_front());
        async move {
            match step {
                Some(Step::Reply(result)) => result,
                Some(Step::Stall) => std::future::pending().await,
                None => Err(crate::hints::provider::ProviderError::Transport),
            }
        }
    }
}
