//! Stateful property test for sessions (auth T12, requirements R10.1, R10.3 to R10.6, R11, R12.1,
//! R12.5).
//!
//! Random sequences of operations run against the real database and against a small pure model.
//! After each step the database answer must match the model: a session works only inside its
//! limits, belongs only to its own user, is removed by logout, and a new value is issued at every
//! sign-in. When a sequence fails, proptest prints the shortest sequence it could find.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> PROPTEST_CASES=256 cargo test -p counterspell-server
//! --test pg_auth_session_props -- --ignored`.

use std::{collections::HashSet, sync::Arc};

use counterspell_common::ids::UserId;
use counterspell_server::auth::{
    clock::{Clock, ManualClock},
    cookies::CookieSettings,
    errors::AuthError,
    session::{AuthMethod, SessionKey, SessionService},
};
use proptest::{collection::vec, prelude::*, test_runner::TestCaseError};
use sqlx::{postgres::PgPoolOptions, PgPool};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// One step of the random sequence. Session and user indexes wrap around, so every step is
/// meaningful once something exists.
#[derive(Debug, Clone)]
enum Op {
    Create(usize),
    Authenticate(usize),
    Logout(usize),
    LogoutAll(usize),
    IsValid(usize),
    Advance(i64),
}

fn op_strategy() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => (0usize..3).prop_map(Op::Create),
        4 => (0usize..16).prop_map(Op::Authenticate),
        1 => (0usize..16).prop_map(Op::Logout),
        1 => (0usize..3).prop_map(Op::LogoutAll),
        2 => (0usize..16).prop_map(Op::IsValid),
        // Short steps test the hourly renewal. Long steps test the 30-day idle and 90-day caps.
        3 => (1i64..=180).prop_map(Op::Advance),
        1 => (1i64..=(60 * 24 * 40)).prop_map(Op::Advance),
    ]
}

/// What the model knows about one session it created.
#[derive(Debug, Clone)]
struct ModelSession {
    raw: String,
    user: usize,
    created: OffsetDateTime,
    last_renewed: OffsetDateTime,
    revoked: bool,
}

impl ModelSession {
    /// The moment the session stops working: 30 days after its last renewal, and never more than
    /// 90 days after its creation.
    fn expires_at(&self) -> OffsetDateTime {
        let idle = self.last_renewed + Duration::days(30);
        let absolute = self.created + Duration::days(90);
        idle.min(absolute)
    }

    fn works_at(&self, now: OffsetDateTime) -> bool {
        !self.revoked && now < self.expires_at()
    }
}

fn start() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp")
}

#[allow(clippy::needless_pass_by_value, reason = "used as map_err(fail)")]
fn fail(error: AuthError) -> TestCaseError {
    TestCaseError::fail(format!("{error:?}"))
}

async fn insert_user(pool: &PgPool) -> Result<UserId, TestCaseError> {
    let handle = format!("p{}", &Uuid::new_v4().simple().to_string()[..12]);
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, email_verified_at, handle, display_name)
         VALUES ($1, now(), $2, $2) RETURNING id",
    )
    .bind(format!("{handle}@example.co"))
    .bind(&handle)
    .fetch_one(pool)
    .await
    .map_err(|error| TestCaseError::fail(error.to_string()))?;
    Ok(UserId::from_uuid(id))
}

/// Runs one sequence against the database and the model. Returns the first mismatch.
#[allow(
    clippy::too_many_lines,
    reason = "one flat match over the six operations reads best in one place"
)]
async fn check(pool: &PgPool, ops: Vec<Op>) -> Result<(), TestCaseError> {
    let clock = Arc::new(ManualClock::new(start()));
    let sessions = SessionService::new(pool.clone(), clock.clone());
    let cookies = CookieSettings::new(true);
    let mut users = Vec::new();
    for _ in 0..3 {
        users.push(insert_user(pool).await?);
    }
    let mut model: Vec<ModelSession> = Vec::new();
    let mut issued: HashSet<String> = HashSet::new();

    for op in ops {
        match op {
            Op::Create(user) => {
                let created = sessions
                    .create(users[user], AuthMethod::MagicLink)
                    .await
                    .map_err(fail)?;
                let raw = created.cookie_value.expose().to_owned();
                prop_assert!(
                    issued.insert(raw.clone()),
                    "every sign-in issues a new value"
                );
                let header = cookies.session_value(&raw).map_err(fail)?;
                let text = header
                    .to_str()
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                prop_assert!(
                    text.contains("HttpOnly")
                        && text.contains("Secure")
                        && text.contains("SameSite=Lax")
                        && text.contains("Path=/")
                        && !text.contains("Domain"),
                    "the session cookie has every required attribute: {text}"
                );
                let now = clock.now();
                model.push(ModelSession {
                    raw,
                    user,
                    created: now,
                    last_renewed: now,
                    revoked: false,
                });
            }
            Op::Authenticate(index) => {
                if model.is_empty() {
                    continue;
                }
                let at = index % model.len();
                let now = clock.now();
                let expected_user = model[at].user;
                let got = sessions.authenticate(&model[at].raw).await.map_err(fail)?;
                if model[at].works_at(now) {
                    let found = got.ok_or_else(|| {
                        TestCaseError::fail(
                            "the model says the session works, the database refused",
                        )
                    })?;
                    prop_assert_eq!(
                        found.user_id,
                        users[expected_user],
                        "a session belongs to its user"
                    );
                    let renews = now - model[at].last_renewed >= Duration::hours(1);
                    prop_assert_eq!(
                        found.renewed,
                        renews,
                        "renewal happens once an hour at most"
                    );
                    if renews {
                        model[at].last_renewed = now;
                    }
                } else {
                    prop_assert!(
                        got.is_none(),
                        "an expired or revoked session must be refused"
                    );
                }
            }
            Op::Logout(index) => {
                if model.is_empty() {
                    continue;
                }
                let at = index % model.len();
                let key = SessionKey::of(&model[at].raw);
                sessions.revoke(&key).await.map_err(fail)?;
                model[at].revoked = true;
            }
            Op::LogoutAll(user) => {
                let expected: u64 = model
                    .iter()
                    .filter(|s| s.user == user && !s.revoked)
                    .count()
                    .try_into()
                    .unwrap_or(u64::MAX);
                let removed = sessions.revoke_all(users[user]).await.map_err(fail)?;
                prop_assert_eq!(
                    removed,
                    expected,
                    "logout everywhere removes exactly that user's sessions"
                );
                for session in model.iter_mut().filter(|s| s.user == user) {
                    session.revoked = true;
                }
            }
            Op::IsValid(index) => {
                if model.is_empty() {
                    continue;
                }
                let at = index % model.len();
                let now = clock.now();
                let key = SessionKey::of(&model[at].raw);
                let got = sessions.is_valid(&key).await.map_err(fail)?;
                prop_assert_eq!(
                    got,
                    model[at].works_at(now),
                    "is_valid agrees with the model"
                );
            }
            Op::Advance(minutes) => clock.advance(Duration::minutes(minutes)),
        }
    }
    Ok(())
}

proptest! {
    #[test]
    #[ignore = "needs DATABASE_URL"]
    fn pg_session_model_matches_database(ops in vec(op_strategy(), 1..40)) {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for this test");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        runtime.block_on(async {
            let pool = PgPoolOptions::new()
                .max_connections(2)
                .connect(&url)
                .await
                .expect("connect to the throwaway database");
            sqlx::migrate!("../../migrations")
                .run(&pool)
                .await
                .expect("migrations apply");
            let outcome = check(&pool, ops).await;
            pool.close().await;
            outcome
        })?;
    }
}
