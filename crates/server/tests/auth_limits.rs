//! Tests for the auth rate limits (auth T15, requirements R14.1 to R14.4, R14.6, R14.7).
//!
//! Most tests use the in-memory limiter. The Valkey test is ignored unless it is run with
//! `VALKEY_URL=redis://localhost:6379 cargo test -p counterspell-server -- --ignored valkey_auth`.

use std::sync::Arc;

use counterspell_server::{
    auth::{
        config::Redacted,
        errors::AuthError,
        limits::{Action, AuthLimiter, Scope},
    },
    ratelimit::{Limiter, MemoryLimiter, ValkeyLimiter},
};
use uuid::Uuid;

fn limiter_in_memory() -> AuthLimiter {
    AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![7_u8; 32]),
    )
}

#[tokio::test]
// test-change: ACC-1
async fn the_request_after_the_limit_is_refused_with_a_wait() {
    let limits = limiter_in_memory();
    for _ in 0..5 {
        limits
            .hit(
                Action::LoginFailure,
                Scope::EmailAndIp,
                "thandi@example.co|203.0.113.7",
            )
            .await
            .expect("under the limit");
    }
    assert!(matches!(
        limits
            .hit(
                Action::LoginFailure,
                Scope::EmailAndIp,
                "thandi@example.co|203.0.113.7"
            )
            .await,
        Err(AuthError::RateLimited {
            retry_after_seconds
        }) if retry_after_seconds > 0
    ));
    // The same address from another IP is a different counter.
    limits
        .hit(
            Action::LoginFailure,
            Scope::EmailAndIp,
            "thandi@example.co|203.0.113.9",
        )
        .await
        .expect("a different pair has its own counter");
}

#[tokio::test]
// test-change: ACC-1
async fn every_identity_has_its_own_counter() {
    let limits = limiter_in_memory();
    for _ in 0..20 {
        limits
            .hit(Action::OAuthStart, Scope::Ip, "203.0.113.7")
            .await
            .expect("under the ip limit");
    }
    assert!(matches!(
        limits
            .hit(Action::OAuthStart, Scope::Ip, "203.0.113.7")
            .await,
        Err(AuthError::RateLimited { .. })
    ));
    limits
        .hit(Action::OAuthStart, Scope::Ip, "203.0.113.8")
        .await
        .expect("a second IP has a fresh counter");
}

#[test]
// test-change: ACC-1
fn keys_hold_no_email_address_or_ip() {
    let limits = limiter_in_memory();
    let key = limits
        .key(
            Action::LoginFailure,
            Scope::EmailAndIp,
            "Thandi@Example.co|203.0.113.7",
        )
        .expect("a limited pair makes a key");
    assert!(key.starts_with("rl:auth:login:email_ip:"), "got {key}");
    assert!(!key.contains("thandi"), "no email text in the key");
    assert!(!key.contains("203.0.113.7"), "no ip text in the key");
    let other = limits
        .key(Action::OAuthStart, Scope::Ip, "203.0.113.7")
        .expect("a limited pair makes a key");
    assert!(other.starts_with("rl:auth:oauth_start:ip:"), "got {other}");
    assert!(!other.contains("203.0.113.7"), "no ip text in the key");
}

#[test]
fn the_same_key_answers_for_the_same_inputs() {
    let limits = limiter_in_memory();
    // test-change: ACC-1
    let first = limits
        .key(Action::OAuthStart, Scope::Ip, "203.0.113.7")
        .expect("key");
    let second = limits
        .key(Action::OAuthStart, Scope::Ip, "203.0.113.7")
        .expect("key");
    assert_eq!(first, second);
}

#[test]
fn an_unknown_pair_is_refused() {
    let limits = limiter_in_memory();
    // test-change: ACC-1
    assert_eq!(
        limits.key(
            Action::OAuthStart,
            Scope::EmailAndIp,
            "x@example.co|203.0.113.7"
        ),
        Err(AuthError::Internal)
    );
}

#[tokio::test]
// test-change: ACC-1
async fn logins_fail_open_when_valkey_is_down() {
    // Nothing listens on port 1, so every Valkey call fails at once.
    let down = ValkeyLimiter::open("redis://127.0.0.1:1").expect("valid url");
    let limits = AuthLimiter::new(Limiter::Valkey(down), &Redacted::new(vec![3_u8; 32]));
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
    assert!(
        matches!(
            limits.hit(Action::Security, Scope::Ip, "203.0.113.7").await,
            Err(AuthError::SecurityUnavailable)
        ),
        "credential enrollment stops when the counter store is down"
    );
}

#[ignore = "needs VALKEY_URL"]
#[tokio::test]
// test-change: ACC-1
async fn valkey_auth_limits_count_in_real_valkey() {
    let url = std::env::var("VALKEY_URL").expect("VALKEY_URL must be set for this test");
    let limiter = ValkeyLimiter::open(&url).expect("valid VALKEY_URL");
    let limits = AuthLimiter::new(Limiter::Valkey(limiter), &Redacted::new(vec![9_u8; 32]));
    let identity = format!("{}@example.co|{}", Uuid::new_v4(), Uuid::new_v4());
    for _ in 0..5 {
        limits
            .hit(Action::LoginFailure, Scope::EmailAndIp, &identity)
            .await
            .expect("five login failures are allowed");
    }
    assert!(matches!(
        limits
            .hit(Action::LoginFailure, Scope::EmailAndIp, &identity)
            .await,
        Err(AuthError::RateLimited { .. })
    ));
}
