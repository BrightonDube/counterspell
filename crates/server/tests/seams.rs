//! Production seams: the defaults that the server uses until auth and the catalogue are built.
//! They must never identify a user or find a problem.

mod support;

use axum::{
    http::{HeaderMap, HeaderValue},
    routing::get,
    Router,
};
use counterspell_server::seams::{
    EmptyCatalogue, LanguageCatalog, ProblemLookup, Unauthenticated, UserResolver,
};
use support::{get as http_get, serve, state_for, Fixture};

fn bearer(value: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("authorization", HeaderValue::from_static(value));
    headers
}

#[test]
fn production_resolver_never_identifies_anyone() {
    assert!(Unauthenticated.resolve(&HeaderMap::new()).is_none());
    assert!(Unauthenticated
        .resolve(&bearer("Bearer test-user-1"))
        .is_none());
}

#[test]
fn production_catalogue_publishes_no_problems() {
    assert!(EmptyCatalogue.published("two-sum").is_none());
}

#[test]
fn launch_catalogue_offers_python_and_javascript_only() {
    let catalogue = LanguageCatalog::launch(65_536);

    let python = catalogue.find("python").expect("python is enabled");
    assert_eq!(python.max_source_bytes, 65_536);
    assert!(catalogue.find("javascript").is_some());
    assert!(
        catalogue.find("java").is_none(),
        "stretch languages are off"
    );
    assert!(
        catalogue.find("PYTHON").is_none(),
        "names are case sensitive"
    );
}

#[tokio::test]
async fn current_user_extractor_rejects_anonymous_requests() {
    let fixture = Fixture::new();
    let address = serve(probe_router(&fixture)).await;

    let reply = http_get(address, "/probe", &[]).await;

    assert_eq!(reply.status, 401);
    assert!(
        reply.body.contains(r#""code":"unauthorized""#),
        "{}",
        reply.body
    );
}

/// A route that needs a signed-in user, so the extractor is exercised through a real request.
fn probe_router(fixture: &Fixture) -> Router {
    async fn who(user: counterspell_server::seams::CurrentUser) -> String {
        user.0.to_string()
    }
    Router::new()
        .route("/probe", get(who))
        .with_state(state_for(&fixture.dir))
}
