//! Tests for the auth rate limits (auth T15, requirements R14.1 to R14.4, R14.6, R14.7, R4.8).
//!
//! Most tests use the in-memory limiter. The Valkey test is ignored unless it is run with
//! `VALKEY_URL=redis://localhost:6379 cargo test -p counterspell-server -- --ignored valkey_auth`.

use std::sync::Arc;

use counterspell_server::{
    auth::{
        config::{Redacted, DEFAULT_EMAIL_DAILY_CAP},
        errors::AuthError,
        limits::{Action, AuthLimiter, Scope},
    },
    ratelimit::{Limiter, MemoryLimiter, ValkeyLimiter},
};
use time::{Date, Month};
use uuid::Uuid;

fn limiter_in_memory() -> AuthLimiter {
    AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![7_u8; 32]),
    )
}

fn today() -> Date {
    Date::from_calendar_date(2026, Month::November, 13).expect("valid date")
}

#[tokio::test]
async fn the_request_after_the_limit_is_refused_with_a_wait() {
    let limits = limiter_in_memory();
    for _ in 0..3 {
        limits
            .hit(Action::Magic, Scope::Email, "thandi@example.co")
            .await
            .expect("the first three magic links in an hour are allowed");
    }
    let refused = limits
        .hit(Action::Magic, Scope::Email, "thandi@example.co")
        .await;
    match refused {
        Err(AuthError::RateLimited {
            retry_after_seconds,
        }) => assert!(retry_after_seconds >= 1, "the client gets a real wait"),
        other => panic!("the fourth request must be limited, got {other:?}"),
    }
}

#[tokio::test]
async fn limits_on_one_address_do_not_affect_another() {
    let limits = limiter_in_memory();
    for _ in 0..4 {
        let _ = limits
            .hit(Action::Magic, Scope::Email, "busy@example.co")
            .await;
    }
    assert!(
        limits
            .hit(Action::Magic, Scope::Email, "quiet@example.co")
            .await
            .is_ok(),
        "a different address has its own counter"
    );
}

#[tokio::test]
async fn login_failures_count_per_address_and_ip_pair() {
    let limits = limiter_in_memory();
    for _ in 0..5 {
        limits
            .hit(
                Action::LoginFailure,
                Scope::EmailAndIp,
                "pair@example.co|203.0.113.7",
            )
            .await
            .expect("five failures are allowed");
    }
    assert!(matches!(
        limits
            .hit(
                Action::LoginFailure,
                Scope::EmailAndIp,
                "pair@example.co|203.0.113.7"
            )
            .await,
        Err(AuthError::RateLimited { .. })
    ));
}

#[test]
fn keys_hold_no_email_address_or_ip() {
    let limits = limiter_in_memory();
    let key = limits
        .key(Action::Magic, Scope::Email, "Thandi@Example.co")
        .expect("key");
    assert!(key.starts_with("rl:auth:magic:email:"), "got {key}");
    assert!(!key.contains("thandi"), "no email text in the key");
    assert!(!key.contains("example"), "no domain text in the key");

    let ip_key = limits
        .key(Action::OAuthStart, Scope::Ip, "203.0.113.7")
        .expect("key");
    assert!(!ip_key.contains("203.0.113.7"), "no IP text in the key");
}

#[test]
fn the_same_identity_gives_the_same_key_and_a_different_secret_does_not() {
    let first = AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![1_u8; 32]),
    );
    let second = AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![2_u8; 32]),
    );
    let a = first
        .key(Action::Reset, Scope::Email, "x@example.co")
        .expect("key");
    let b = first
        .key(Action::Reset, Scope::Email, "x@example.co")
        .expect("key");
    let c = second
        .key(Action::Reset, Scope::Email, "x@example.co")
        .expect("key");
    assert_eq!(a, b, "the same secret and identity give the same key");
    assert_ne!(
        a, c,
        "the key depends on the secret, so it cannot be guessed from the address"
    );
}

#[tokio::test]
async fn the_daily_mail_cap_stops_at_eighty_and_resets_on_the_next_utc_day() {
    let limits = limiter_in_memory();
    for _ in 0..DEFAULT_EMAIL_DAILY_CAP {
        limits
            .hit_daily_mail_cap(today(), DEFAULT_EMAIL_DAILY_CAP)
            .await
            .expect("up to the cap, mail is allowed");
    }
    assert_eq!(
        limits
            .hit_daily_mail_cap(today(), DEFAULT_EMAIL_DAILY_CAP)
            .await,
        Err(AuthError::EmailUnavailable),
        "the 81st mail of the day is refused"
    );
    let tomorrow = today().next_day().expect("next day");
    assert!(
        limits
            .hit_daily_mail_cap(tomorrow, DEFAULT_EMAIL_DAILY_CAP)
            .await
            .is_ok(),
        "the next UTC day starts with a full allowance"
    );
}

#[tokio::test]
async fn mail_fails_closed_and_logins_fail_open_when_valkey_is_down() {
    // Nothing listens on port 1, so every Valkey call fails at once.
    let down = ValkeyLimiter::open("redis://127.0.0.1:1").expect("a well-formed URL");
    let limits = AuthLimiter::new(Limiter::Valkey(down), &Redacted::new(vec![3_u8; 32]));

    assert_eq!(
        limits
            .hit(Action::Magic, Scope::Email, "down@example.co")
            .await,
        Err(AuthError::EmailUnavailable),
        "mail stops when its counter store is down"
    );
    assert_eq!(
        limits
            .hit_daily_mail_cap(today(), DEFAULT_EMAIL_DAILY_CAP)
            .await,
        Err(AuthError::EmailUnavailable),
        "the daily cap also fails closed"
    );
    assert!(
        limits
            .hit(
                Action::LoginFailure,
                Scope::EmailAndIp,
                "down@example.co|203.0.113.7"
            )
            .await
            .is_ok(),
        "password attempts still work while the counter store is down"
    );
}

#[ignore = "needs VALKEY_URL"]
#[tokio::test]
async fn valkey_auth_limits_count_in_real_valkey() {
    let url = std::env::var("VALKEY_URL").expect("VALKEY_URL must be set for this test");
    let limiter = ValkeyLimiter::open(&url).expect("valid VALKEY_URL");
    let limits = AuthLimiter::new(Limiter::Valkey(limiter), &Redacted::new(vec![9_u8; 32]));
    let identity = format!("{}@example.co", Uuid::new_v4());
    for _ in 0..3 {
        limits
            .hit(Action::Magic, Scope::Email, &identity)
            .await
            .expect("three magic links are allowed");
    }
    assert!(matches!(
        limits.hit(Action::Magic, Scope::Email, &identity).await,
        Err(AuthError::RateLimited { .. })
    ));
}
