//! Mail senders, templates and the link format (R4.1, R4.2, R4.5 to R4.7, R4.9).
//!
//! No test here touches the network. The Resend sender is tested with a mock server once its
//! HTTP client is available (task T14, blocked on the licence decision in `auth-blockers.md`).

use std::sync::Arc;

use counterspell_server::auth::{
    clock::{Clock, ManualClock},
    config::{AppEnv, Redacted},
    email::{
        compose, magic_link_url, EmailAddress, EmailSender, GuardedSender, LogSender, MailError,
        MemorySender,
    },
    limits::AuthLimiter,
    tokens::Purpose,
};
use counterspell_server::ratelimit::{Limiter, MemoryLimiter};
use time::macros::datetime;

fn address(text: &str) -> EmailAddress {
    EmailAddress::parse(text).expect("the test address is valid")
}

fn limiter() -> AuthLimiter {
    AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![7; 32]),
    )
}

#[test]
fn the_link_uses_the_configured_origin_and_the_fragment() {
    assert_eq!(
        magic_link_url("https://counterspell.example/", "abc123"),
        "https://counterspell.example/auth/verify#token=abc123"
    );
    assert_eq!(
        magic_link_url("https://counterspell.example", "abc123"),
        "https://counterspell.example/auth/verify#token=abc123"
    );
}

#[test]
fn the_mail_carries_the_link_in_text_and_html_and_escapes_html() {
    let link = "https://counterspell.example/auth/verify#token=t1";
    let mail = compose(
        Purpose::MagicLink,
        address("Reader@Example.com"),
        link,
        "key-1",
    );
    assert_eq!(mail.to.as_str(), "reader@example.com");
    assert_eq!(mail.idempotency_key, "key-1");
    assert!(mail.text.contains(link));
    assert!(mail.html.contains(&format!("href=\"{link}\"")));
    assert!(mail.text.contains("15 minutes"));

    let hostile = compose(
        Purpose::Reset,
        address("reader@example.com"),
        "https://counterspell.example/<script>alert(1)</script>",
        "key-2",
    );
    assert!(!hostile.html.contains("<script>"));
    assert!(hostile.html.contains("&lt;script&gt;"));
}

#[test]
fn each_purpose_has_its_own_subject() {
    let subjects: Vec<String> = [
        Purpose::MagicLink,
        Purpose::Verify,
        Purpose::Reset,
        Purpose::LinkEmail,
    ]
    .into_iter()
    .map(|purpose| compose(purpose, address("a@b.co"), "https://x/y", "k").subject)
    .collect();
    let mut unique = subjects.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), 4);
}

#[tokio::test]
async fn memory_sender_keeps_the_message_and_its_idempotency_key() {
    let sender = MemorySender::default();
    let mail = compose(
        Purpose::MagicLink,
        address("reader@example.com"),
        "https://x/auth/verify#token=t9",
        "key-9",
    );
    sender.send(mail).await.expect("memory send succeeds");

    let sent = sender.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to.as_str(), "reader@example.com");
    assert_eq!(sent[0].idempotency_key, "key-9");
}

#[test]
fn a_mail_debug_print_hides_the_link_and_the_body() {
    let mail = compose(
        Purpose::MagicLink,
        address("reader@example.com"),
        "https://x/auth/verify#token=secret-token-value",
        "key-3",
    );
    let shown = format!("{mail:?}");
    assert!(!shown.contains("secret-token-value"));
    assert!(!shown.contains("reader@example.com"));
}

#[test]
fn the_log_sender_is_refused_in_production_and_allowed_in_development() {
    assert_eq!(
        LogSender::new(AppEnv::Production).err(),
        Some(MailError::NotAllowed)
    );
    assert!(LogSender::new(AppEnv::Development).is_ok());
}

#[tokio::test]
async fn the_guard_refuses_to_mail_the_demo_address() {
    let inner = Arc::new(MemorySender::default());
    let clock: Arc<dyn Clock> = Arc::new(ManualClock::new(datetime!(2026-11-13 10:00 UTC)));
    let guard = GuardedSender::new(
        inner.clone(),
        limiter(),
        80,
        Some(address("demo@counterspell.example")),
        clock,
    );

    let demo = compose(
        Purpose::MagicLink,
        address("demo@counterspell.example"),
        "https://x/y",
        "k",
    );
    assert_eq!(guard.send(demo).await, Err(MailError::Suppressed));
    assert!(inner.sent().is_empty());

    let other = compose(
        Purpose::MagicLink,
        address("reader@example.com"),
        "https://x/y",
        "k",
    );
    guard.send(other).await.expect("ordinary mail is forwarded");
    assert_eq!(inner.sent().len(), 1);
}

#[tokio::test]
async fn the_daily_cap_stops_mail_and_resets_on_the_next_utc_day() {
    let inner = Arc::new(MemorySender::default());
    let manual = Arc::new(ManualClock::new(datetime!(2026-11-13 23:30 UTC)));
    let clock: Arc<dyn Clock> = manual.clone();
    let guard = GuardedSender::new(inner.clone(), limiter(), 2, None, clock);

    let mail = || {
        compose(
            Purpose::Verify,
            address("reader@example.com"),
            "https://x/y",
            "k",
        )
    };
    guard.send(mail()).await.expect("first mail of the day");
    guard.send(mail()).await.expect("second mail of the day");
    assert_eq!(guard.send(mail()).await, Err(MailError::Unavailable));
    assert_eq!(inner.sent().len(), 2, "the refused mail is not sent");

    manual.advance(time::Duration::hours(2));
    guard
        .send(mail())
        .await
        .expect("the cap resets on the next UTC day");
    assert_eq!(inner.sent().len(), 3);
}
