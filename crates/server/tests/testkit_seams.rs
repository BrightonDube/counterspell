//! Test versions of the seams. They are only compiled with the `testkit` feature, which the server's
//! own tests always enable.
#![cfg(feature = "testkit")]

mod support;

use axum::{
    http::{HeaderMap, HeaderValue},
    routing::get,
    Router,
};
use counterspell_common::ids::UserId;
use counterspell_server::{
    seams::{CurrentUser, ProblemLookup, UserResolver},
    testkit::{BearerTestResolver, InMemoryProblems},
    AppState,
};
use support::{get as http_get, serve, state_for, Fixture};
use uuid::Uuid;

fn bearer(value: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("authorization", HeaderValue::from_static(value));
    headers
}

#[test]
fn bearer_resolver_reads_numbered_test_users() {
    let resolver = BearerTestResolver;

    let seven = resolver.resolve(&bearer("Bearer test-user-7"));
    assert_eq!(seven, Some(UserId::from_uuid(Uuid::from_u128(7))));
    assert!(resolver.resolve(&bearer("Bearer test-user-x")).is_none());
    assert!(resolver.resolve(&bearer("Basic dXNlcjpwYXNz")).is_none());
    assert!(resolver.resolve(&HeaderMap::new()).is_none());
}

#[test]
fn in_memory_catalogue_publishes_only_listed_slugs() {
    let problems = InMemoryProblems::new(&["two-sum"]);

    assert_eq!(
        problems.published("two-sum").map(|p| p.slug),
        Some("two-sum".to_owned())
    );
    assert!(problems.published("three-sum").is_none());
}

#[tokio::test]
async fn extractor_accepts_a_bearer_test_user() {
    let fixture = Fixture::new();
    let state: AppState = state_for(&fixture.dir).with_user_resolver(BearerTestResolver);
    let router = Router::new().route("/probe", get(who)).with_state(state);
    let address = serve(router).await;

    let reply = http_get(
        address,
        "/probe",
        &[("Authorization", "Bearer test-user-7")],
    )
    .await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, Uuid::from_u128(7).to_string());
}

async fn who(user: CurrentUser) -> String {
    user.0.to_string()
}
