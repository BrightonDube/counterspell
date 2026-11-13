//! Rate limits for run submissions. Valkey holds counters only. These tests need no Valkey server
//! unless they are marked `valkey_` and run with `VALKEY_URL` set.

use std::net::IpAddr;

use counterspell_common::ids::UserId;
use counterspell_server::ratelimit::{
    ip_key, user_key, Decision, MemoryLimiter, RateLimiter, ValkeyLimiter,
};
use uuid::Uuid;

#[tokio::test]
async fn memory_limiter_allows_up_to_the_limit_then_limits() {
    let limiter = MemoryLimiter::default();

    assert_eq!(
        limiter.hit("k", 2, 600).await,
        Decision::Allowed { remaining: 1 }
    );
    assert_eq!(
        limiter.hit("k", 2, 600).await,
        Decision::Allowed { remaining: 0 }
    );
    match limiter.hit("k", 2, 600).await {
        Decision::Limited {
            retry_after_seconds,
        } => assert!(retry_after_seconds >= 1),
        other => panic!("third hit should be limited, got {other:?}"),
    }
}

#[tokio::test]
async fn memory_limiter_counts_each_key_separately() {
    let limiter = MemoryLimiter::default();
    assert_eq!(
        limiter.hit("user", 1, 600).await,
        Decision::Allowed { remaining: 0 }
    );

    assert_eq!(
        limiter.hit("ip", 1, 600).await,
        Decision::Allowed { remaining: 0 }
    );
}

#[test]
fn user_key_is_namespaced_by_user_id() {
    let id = UserId::from_uuid(Uuid::from_u128(7));

    assert_eq!(
        user_key(id),
        "rl:runs:user:00000000-0000-0000-0000-000000000007"
    );
}

#[test]
fn ip_key_is_a_hash_and_never_contains_the_address() {
    let address: IpAddr = "203.0.113.9".parse().expect("valid ip");

    let key = ip_key(address);

    assert!(key.starts_with("rl:runs:ip:"), "{key}");
    assert!(
        !key.contains("203.0.113.9"),
        "raw address leaked into {key}"
    );
    assert_eq!(key, ip_key(address), "the same address gives the same key");
}

#[tokio::test]
async fn valkey_outage_fails_open() {
    // Nothing listens on port 1, so every command fails. The limiter must allow the run and say so.
    let limiter = ValkeyLimiter::open("redis://127.0.0.1:1").expect("the URL is well formed");

    assert_eq!(
        limiter.hit("rl:runs:user:outage", 1, 600).await,
        Decision::SkippedFailOpen
    );
}

#[tokio::test]
#[ignore = "needs VALKEY_URL"]
async fn valkey_window_limits() {
    let limiter = valkey_from_env();
    let key = unique_key("window");

    assert_eq!(
        limiter.hit(&key, 2, 600).await,
        Decision::Allowed { remaining: 1 }
    );
    assert_eq!(
        limiter.hit(&key, 2, 600).await,
        Decision::Allowed { remaining: 0 }
    );
    match limiter.hit(&key, 2, 600).await {
        Decision::Limited {
            retry_after_seconds,
        } => {
            assert!(
                (1..=600).contains(&retry_after_seconds),
                "{retry_after_seconds}"
            );
        }
        other => panic!("third hit should be limited, got {other:?}"),
    }
}

#[tokio::test]
#[ignore = "needs VALKEY_URL"]
async fn valkey_window_resets_real_time() {
    // Real time: the window is one second, so this test sleeps past it on purpose.
    let limiter = valkey_from_env();
    let key = unique_key("reset");

    assert_eq!(
        limiter.hit(&key, 1, 1).await,
        Decision::Allowed { remaining: 0 }
    );
    assert!(matches!(
        limiter.hit(&key, 1, 1).await,
        Decision::Limited { .. }
    ));
    tokio::time::sleep(std::time::Duration::from_millis(1_300)).await;

    assert_eq!(
        limiter.hit(&key, 1, 1).await,
        Decision::Allowed { remaining: 0 }
    );
}

#[tokio::test]
#[ignore = "needs VALKEY_URL"]
async fn valkey_keys_hold_only_counters() {
    let limiter = valkey_from_env();
    let key = unique_key("counters");
    limiter.hit(&key, 5, 600).await;
    limiter.hit(&key, 5, 600).await;

    let client =
        redis::Client::open(std::env::var("VALKEY_URL").expect("VALKEY_URL")).expect("client");
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .expect("connect");
    let mut cursor: u64 = 0;
    let mut checked = 0;
    loop {
        let (next, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg("rl:*")
            .arg("COUNT")
            .arg(500)
            .query_async(&mut connection)
            .await
            .expect("scan");
        for name in keys {
            let kind: String = redis::cmd("TYPE")
                .arg(&name)
                .query_async(&mut connection)
                .await
                .expect("type");
            let value: String = redis::cmd("GET")
                .arg(&name)
                .query_async(&mut connection)
                .await
                .expect("get");
            assert_eq!(kind, "string", "{name} is not a counter");
            assert!(
                value.parse::<u64>().is_ok(),
                "{name} holds {value:?}, not a count"
            );
            checked += 1;
        }
        if next == 0 {
            break;
        }
        cursor = next;
    }

    assert!(
        checked >= 1,
        "the test's own counter should be in the namespace"
    );
    let kind: String = redis::cmd("TYPE")
        .arg(&key)
        .query_async(&mut connection)
        .await
        .expect("type");
    assert_eq!(kind, "string");
}

fn valkey_from_env() -> ValkeyLimiter {
    let url = std::env::var("VALKEY_URL").expect("VALKEY_URL is set for ignored tests");
    ValkeyLimiter::open(&url).expect("VALKEY_URL is well formed")
}

/// A key no other test uses, so reruns and parallel tests cannot share counters.
fn unique_key(purpose: &str) -> String {
    format!("rl:test:{purpose}:{}", Uuid::new_v4())
}
