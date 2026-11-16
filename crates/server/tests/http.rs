//! HTTP behaviour of the router: health, the JSON 404 for unknown API paths, static files and the SPA fallback.

mod support;

use counterspell_server::build_router;
use support::{get, serve, state_for, Fixture};

#[tokio::test]
async fn health_reports_ok_as_json() {
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = get(address, "/api/health", &[]).await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, r#"{"status":"ok"}"#);
}

#[tokio::test]
async fn unknown_api_path_is_a_json_404_not_html() {
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    for path in ["/api", "/api/does-not-exist", "/api/runs/abc/unknown"] {
        let reply = get(address, path, &[]).await;

        assert_eq!(reply.status, 404, "{path}");
        assert!(
            reply.headers.to_lowercase().contains("application/json"),
            "{path}"
        );
        assert!(
            reply.body.contains(r#""code":"not_found""#),
            "{path}: {}",
            reply.body
        );
        assert!(!reply.body.contains("<html"), "{path} returned HTML");
    }
}

#[tokio::test]
async fn existing_static_file_is_served() {
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = get(address, "/asset.txt", &[]).await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, "plain-asset");
}

#[tokio::test]
async fn unknown_page_falls_back_to_the_spa_entry() {
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = get(address, "/runs/123", &[]).await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, "<main>spa-entry</main>");
}

#[tokio::test]
async fn root_serves_the_prerendered_landing_page_not_the_fallback() {
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = get(address, "/", &[]).await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, "<main>landing</main>");
}

#[tokio::test]
async fn every_reply_says_nosniff() {
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = get(address, "/api/health", &[]).await;

    assert!(
        reply
            .headers
            .to_lowercase()
            .contains("x-content-type-options: nosniff"),
        "{}",
        reply.headers
    );
}

#[test]
fn startup_check_requires_the_spa_fallback_file() {
    let fixture = Fixture::new();
    std::fs::remove_file(fixture.dir.join("200.html")).expect("remove fallback");

    let error = counterspell_server::ensure_static_files(&fixture.dir)
        .expect_err("a directory without 200.html must fail the start-up check");

    assert!(error.to_string().contains("200.html"), "{error}");
}
