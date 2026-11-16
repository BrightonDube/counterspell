//! Database tests for single-use email tokens (auth T13, requirements R3.1 to R3.8, R9.3).
//!
//! Time comes from a `ManualClock`. The reference rule for validity is `token_status`, which the
//! property test uses as its model.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server
//! --test pg_auth_tokens -- --ignored`.

use std::sync::Arc;

use counterspell_common::ids::UserId;
use counterspell_server::auth::{
    clock::{Clock, ManualClock},
    email::EmailAddress,
    errors::AuthError,
    tokens::{email_token_expiry, token_status, Purpose, TokenStatus, TokenStore},
};
use proptest::{collection::vec, prelude::*, test_runner::TestCaseError};
use sqlx::{postgres::PgPoolOptions, PgPool};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

fn start() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp")
}

fn address(text: &str) -> EmailAddress {
    EmailAddress::parse(text).expect("valid test address")
}

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

fn store(pool: &PgPool, clock: &Arc<ManualClock>) -> TokenStore {
    TokenStore::new(pool.clone(), clock.clone())
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_one_token_is_used_by_exactly_one_of_many_racers(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start()));
    let tokens = store(&pool, &clock);
    let raw = tokens
        .issue(
            Purpose::MagicLink,
            &address("racer@example.co"),
            None,
            serde_json::json!({}),
        )
        .await
        .expect("issue")
        .expose()
        .to_owned();

    let mut racers = Vec::new();
    for _ in 0..32 {
        let tokens = tokens.clone();
        let raw = raw.clone();
        racers.push(tokio::spawn(async move {
            tokens.consume(&raw, Purpose::MagicLink, None).await.is_ok()
        }));
    }
    let mut winners = 0;
    for racer in racers {
        if racer.await.expect("racer finished") {
            winners += 1;
        }
    }
    assert_eq!(winners, 1, "exactly one concurrent consume may succeed");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_token_works_until_exactly_fifteen_minutes(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start()));
    let tokens = store(&pool, &clock);
    let inside = tokens
        .issue(
            Purpose::MagicLink,
            &address("inside@example.co"),
            None,
            serde_json::json!({}),
        )
        .await
        .expect("issue")
        .expose()
        .to_owned();
    clock.advance(Duration::minutes(15) - Duration::seconds(1));
    assert!(
        tokens
            .consume(&inside, Purpose::MagicLink, None)
            .await
            .is_ok(),
        "one second before the 15-minute limit the token still works"
    );

    let late = tokens
        .issue(
            Purpose::MagicLink,
            &address("late@example.co"),
            None,
            serde_json::json!({}),
        )
        .await
        .expect("issue")
        .expose()
        .to_owned();
    clock.advance(Duration::minutes(15));
    assert_eq!(
        tokens.consume(&late, Purpose::MagicLink, None).await.err(),
        Some(AuthError::InvalidOrExpiredToken),
        "at exactly 15 minutes the token has expired"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_token_never_works_for_another_purpose(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start()));
    let tokens = store(&pool, &clock);
    let raw = tokens
        .issue(
            Purpose::Verify,
            &address("purpose@example.co"),
            None,
            serde_json::json!({}),
        )
        .await
        .expect("issue")
        .expose()
        .to_owned();

    assert_eq!(
        tokens.consume(&raw, Purpose::MagicLink, None).await.err(),
        Some(AuthError::InvalidOrExpiredToken),
        "a verify token is refused as a magic link"
    );
    assert!(
        tokens.consume(&raw, Purpose::Verify, None).await.is_ok(),
        "the refused attempt did not use up the token"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_issuing_again_supersedes_the_earlier_token(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start()));
    let tokens = store(&pool, &clock);
    let email = address("again@example.co");
    let first = tokens
        .issue(Purpose::MagicLink, &email, None, serde_json::json!({}))
        .await
        .expect("first")
        .expose()
        .to_owned();
    let second = tokens
        .issue(Purpose::MagicLink, &email, None, serde_json::json!({}))
        .await
        .expect("second")
        .expose()
        .to_owned();

    let (live,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM email_tokens WHERE email = $1 AND purpose = 'magic_link' AND used_at IS NULL",
    )
    .bind(email.as_str())
    .fetch_one(&pool)
    .await
    .expect("count live tokens");
    assert_eq!(live, 1, "one live token per address and purpose");

    assert!(
        tokens
            .consume(&first, Purpose::MagicLink, None)
            .await
            .is_err(),
        "the superseded token no longer works"
    );
    assert!(tokens
        .consume(&second, Purpose::MagicLink, None)
        .await
        .is_ok());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_no_table_stores_the_raw_token(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start()));
    let tokens = store(&pool, &clock);
    let raw = tokens
        .issue(
            Purpose::Reset,
            &address("hashed-token@example.co"),
            None,
            serde_json::json!({}),
        )
        .await
        .expect("issue")
        .expose()
        .to_owned();

    let (as_raw,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM email_tokens WHERE token_hash = convert_to($1, 'UTF8')",
    )
    .bind(&raw)
    .fetch_one(&pool)
    .await
    .expect("count by raw value");
    let (as_hash,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM email_tokens WHERE token_hash = $1")
            .bind(
                counterspell_server::auth::tokens::TokenHash::of(&raw)
                    .as_bytes()
                    .as_slice(),
            )
            .fetch_one(&pool)
            .await
            .expect("count by hash");
    assert_eq!(as_raw, 0, "the raw token is never stored");
    assert_eq!(as_hash, 1, "the stored value is the hash of the raw token");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_link_token_only_works_for_the_user_it_was_issued_to(pool: PgPool) {
    let owner = verified_user(&pool, "link-owner").await;
    let stranger = verified_user(&pool, "link-stranger").await;
    let clock = Arc::new(ManualClock::new(start()));
    let tokens = store(&pool, &clock);
    let raw = tokens
        .issue(
            Purpose::LinkEmail,
            &address("attach@example.co"),
            Some(owner),
            serde_json::json!({}),
        )
        .await
        .expect("issue")
        .expose()
        .to_owned();

    let peeked = tokens.peek(&raw).await.expect("peek").expect("live token");
    assert_eq!(peeked.purpose, Purpose::LinkEmail);
    assert_eq!(peeked.user_id, Some(owner));

    assert!(
        tokens
            .consume(&raw, Purpose::LinkEmail, Some(stranger))
            .await
            .is_err(),
        "another user's session cannot use the link"
    );
    let used = tokens
        .consume(&raw, Purpose::LinkEmail, Some(owner))
        .await
        .expect("the owner can still use it, because the failed attempt did not use it up");
    assert_eq!(used.user_id, Some(owner));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_used_token_is_refused_even_inside_its_window(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start()));
    let tokens = store(&pool, &clock);
    let raw = tokens
        .issue(
            Purpose::MagicLink,
            &address("once@example.co"),
            None,
            serde_json::json!({}),
        )
        .await
        .expect("issue")
        .expose()
        .to_owned();
    clock.advance(Duration::minutes(1));

    assert!(tokens.consume(&raw, Purpose::MagicLink, None).await.is_ok());
    assert_eq!(
        tokens.consume(&raw, Purpose::MagicLink, None).await.err(),
        Some(AuthError::InvalidOrExpiredToken),
        "a second use is refused while the token is still inside its lifetime"
    );
}

proptest! {
    #[test]
    #[ignore = "needs DATABASE_URL"]
    fn pg_consume_agrees_with_the_reference_rule(
        steps in vec((0i64..=1200, any::<bool>()), 1..12),
    ) {
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
            let outcome = check_consume_model(&pool, steps).await;
            pool.close().await;
            outcome
        })?;
    }
}

/// Issues one token, then consumes it at random times, sometimes with the wrong purpose. After each
/// attempt the database must agree with `token_status` and the purpose rule.
async fn check_consume_model(pool: &PgPool, steps: Vec<(i64, bool)>) -> Result<(), TestCaseError> {
    let clock = Arc::new(ManualClock::new(start()));
    let tokens = TokenStore::new(pool.clone(), clock.clone());
    let issued = clock.now();
    let expires = email_token_expiry(issued);
    let raw = tokens
        .issue(
            Purpose::MagicLink,
            &address("model@example.co"),
            None,
            serde_json::json!({}),
        )
        .await
        .map_err(|e| TestCaseError::fail(format!("{e:?}")))?
        .expose()
        .to_owned();

    let mut used: Option<OffsetDateTime> = None;
    for (seconds, right_purpose) in steps {
        clock.advance(Duration::seconds(seconds));
        let now = clock.now();
        let purpose = if right_purpose {
            Purpose::MagicLink
        } else {
            Purpose::Reset
        };
        let expected =
            right_purpose && token_status(issued, expires, used, now) == TokenStatus::Valid;
        let got = tokens.consume(&raw, purpose, None).await.is_ok();
        prop_assert_eq!(
            got,
            expected,
            "consume at {} must match the reference rule",
            now
        );
        if got {
            used = Some(now);
        }
    }
    Ok(())
}
