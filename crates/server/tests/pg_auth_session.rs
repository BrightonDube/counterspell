//! Database tests for server-side sessions (auth T10, requirements R10.2 to R10.5, R10.7 to R10.9,
//! R11.2, R11.6).
//!
//! Time comes from a `ManualClock`, so expiry and renewal are checked without sleeping.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server -- --ignored pg_auth_session`.

use std::sync::Arc;

use counterspell_common::ids::UserId;
use counterspell_server::auth::{
    clock::ManualClock,
    session::{AuthMethod, SessionKey, SessionService},
};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// Inserts a verified user and returns its id.
async fn verified_user(pool: &PgPool, handle: &str) -> UserId {
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, email_verified_at, handle, display_name)
         VALUES ($1, now(), $2, $2) RETURNING id",
    )
    .bind(format!("{handle}@example.co"))
    .bind(handle)
    .fetch_one(pool)
    .await
    .expect("insert user");
    UserId::from_uuid(id)
}

fn start() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp")
}

/// A service whose clock the test can move.
fn service(pool: &PgPool, clock: &Arc<ManualClock>) -> SessionService {
    SessionService::new(pool.clone(), clock.clone())
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_created_sessions_store_only_the_hash(pool: PgPool) {
    let user = verified_user(&pool, "hashed").await;
    let clock = Arc::new(ManualClock::new(start()));
    let sessions = service(&pool, &clock);

    let created = sessions
        .create(user, AuthMethod::MagicLink)
        .await
        .expect("create");
    let raw = created.cookie_value.expose().to_owned();

    let (stored_by_hash,): (i64,) = sqlx::query_as("SELECT count(*) FROM sessions WHERE id = $1")
        .bind(SessionKey::of(&raw).as_str())
        .fetch_one(&pool)
        .await
        .expect("count by hash");
    let (stored_by_raw,): (i64,) = sqlx::query_as("SELECT count(*) FROM sessions WHERE id = $1")
        .bind(&raw)
        .fetch_one(&pool)
        .await
        .expect("count by raw value");
    assert_eq!(
        stored_by_hash, 1,
        "the row is keyed by the hash of the cookie"
    );
    assert_eq!(
        stored_by_raw, 0,
        "the raw cookie value must never be stored"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_session_authenticates_until_its_idle_limit(pool: PgPool) {
    let user = verified_user(&pool, "idle").await;
    let clock = Arc::new(ManualClock::new(start()));
    let sessions = service(&pool, &clock);
    let raw = sessions
        .create(user, AuthMethod::Password)
        .await
        .expect("create")
        .cookie_value
        .expose()
        .to_owned();

    // `is_valid` does not renew, so these checks measure from creation.
    let key = SessionKey::of(&raw);
    clock.advance(Duration::days(30) - Duration::seconds(1));
    assert!(
        sessions.is_valid(&key).await.expect("is_valid"),
        "one second before 30 idle days the session still works"
    );

    clock.advance(Duration::seconds(2));
    assert!(
        !sessions.is_valid(&key).await.expect("is_valid"),
        "after 30 idle days with no renewal the session is refused"
    );
    let idle = sessions.authenticate(&raw).await.expect("authenticate");
    assert!(
        idle.is_none(),
        "an idle-expired session cannot be revived by authenticate"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_continuous_use_is_still_capped_at_ninety_days(pool: PgPool) {
    let user = verified_user(&pool, "absolute").await;
    let clock = Arc::new(ManualClock::new(start()));
    let sessions = service(&pool, &clock);
    let raw = sessions
        .create(user, AuthMethod::OauthGithub)
        .await
        .expect("create")
        .cookie_value
        .expose()
        .to_owned();

    for _ in 0..3 {
        clock.advance(Duration::days(29));
        let kept = sessions.authenticate(&raw).await.expect("authenticate");
        assert!(
            kept.is_some(),
            "renewal keeps the session alive inside 90 days"
        );
    }

    clock.advance(Duration::days(3));
    let past_cap = sessions.authenticate(&raw).await.expect("authenticate");
    assert!(
        past_cap.is_none(),
        "the 90-day absolute limit ends the session even when used"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_renewal_happens_at_most_once_an_hour(pool: PgPool) {
    let user = verified_user(&pool, "throttle").await;
    let clock = Arc::new(ManualClock::new(start()));
    let sessions = service(&pool, &clock);
    let created = sessions
        .create(user, AuthMethod::Register)
        .await
        .expect("create");
    let raw = created.cookie_value.expose().to_owned();

    clock.advance(Duration::minutes(30));
    let early = sessions
        .authenticate(&raw)
        .await
        .expect("authenticate")
        .expect("valid");
    assert!(!early.renewed, "no renewal within the first hour");

    clock.advance(Duration::minutes(31));
    let later = sessions
        .authenticate(&raw)
        .await
        .expect("authenticate")
        .expect("valid");
    assert!(later.renewed, "a renewal happens once an hour has passed");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_unknown_wrong_case_and_revoked_values_do_not_authenticate(pool: PgPool) {
    let user = verified_user(&pool, "refused").await;
    let clock = Arc::new(ManualClock::new(start()));
    let sessions = service(&pool, &clock);
    let raw = sessions
        .create(user, AuthMethod::MagicLink)
        .await
        .expect("create")
        .cookie_value
        .expose()
        .to_owned();

    let valid = sessions.authenticate(&raw).await.expect("authenticate");
    assert!(valid.is_some(), "the real value works first");

    let swapped: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_lowercase() {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect();
    assert_ne!(
        swapped, raw,
        "the case-swapped value must differ from the original"
    );
    assert!(sessions
        .authenticate(&swapped)
        .await
        .expect("authenticate")
        .is_none());
    assert!(sessions
        .authenticate("unknown-value")
        .await
        .expect("authenticate")
        .is_none());

    sessions
        .revoke(&SessionKey::of(&raw))
        .await
        .expect("revoke");
    assert!(
        sessions
            .authenticate(&raw)
            .await
            .expect("authenticate")
            .is_none(),
        "a revoked session is refused on the next request"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_revoke_all_deletes_only_one_users_sessions(pool: PgPool) {
    let first = verified_user(&pool, "owner-a").await;
    let second = verified_user(&pool, "owner-b").await;
    let clock = Arc::new(ManualClock::new(start()));
    let sessions = service(&pool, &clock);

    sessions
        .create(first, AuthMethod::MagicLink)
        .await
        .expect("a1");
    sessions
        .create(first, AuthMethod::Password)
        .await
        .expect("a2");
    let other = sessions
        .create(second, AuthMethod::MagicLink)
        .await
        .expect("b1")
        .cookie_value
        .expose()
        .to_owned();

    let removed = sessions.revoke_all(first).await.expect("revoke all");
    assert_eq!(
        removed, 2,
        "exactly the first user's two sessions are removed"
    );
    assert!(
        sessions
            .authenticate(&other)
            .await
            .expect("authenticate")
            .is_some(),
        "the second user's session is untouched"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_sessions_of_a_deleted_user_are_gone(pool: PgPool) {
    let user = verified_user(&pool, "deleted").await;
    let clock = Arc::new(ManualClock::new(start()));
    let sessions = service(&pool, &clock);
    let raw = sessions
        .create(user, AuthMethod::MagicLink)
        .await
        .expect("create")
        .cookie_value
        .expose()
        .to_owned();

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.as_uuid())
        .execute(&pool)
        .await
        .expect("delete user");

    assert!(sessions
        .authenticate(&raw)
        .await
        .expect("authenticate")
        .is_none());
    assert!(!sessions
        .is_valid(&SessionKey::of(&raw))
        .await
        .expect("is_valid"));
}
