//! The CSP report intake (PF08 stage 0): a bounded, rate-limited data sink that browsers post
//! violation reports to. It must never weaken the CSRF/origin rules around it.

mod support;

use counterspell_server::build_router;
use support::{post, serve, state_for, Fixture};

fn report_body() -> &'static str {
    r#"{"csp-report":{"document-uri":"https://counterspel.online/problems","violated-directive":"script-src-elem","effective-directive":"script-src-elem","blocked-uri":"inline","disposition":"report","status-code":200}}"#
}

#[tokio::test]
async fn a_valid_report_is_accepted_and_never_cached() {
    // test-change: PROD-FOUND-CSP-REPORT
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = post(
        address,
        "/api/csp-report",
        &[("Content-Type", "application/csp-report")],
        report_body(),
    )
    .await;

    assert_eq!(reply.status, 204, "{} {}", reply.headers, reply.body);
    assert!(
        reply
            .headers
            .to_lowercase()
            .contains("cache-control: no-store"),
        "{}",
        reply.headers
    );
}

#[tokio::test]
async fn a_modern_reports_json_batch_is_accepted() {
    // The newer report-to mechanism posts an array of {"type","url","body"} objects.
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let body = r#"[{"type":"csp-violation","url":"https://counterspel.online/","body":{"effectiveDirective":"script-src-elem","blockedURL":"inline","disposition":"report"}}]"#;
    let reply = post(
        address,
        "/api/csp-report",
        &[("Content-Type", "application/reports+json")],
        body,
    )
    .await;

    assert_eq!(reply.status, 204, "{} {}", reply.headers, reply.body);
}

#[tokio::test]
async fn an_unreadable_body_is_a_400() {
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = post(
        address,
        "/api/csp-report",
        &[("Content-Type", "application/csp-report")],
        "not json at all",
    )
    .await;

    assert_eq!(reply.status, 400, "{} {}", reply.headers, reply.body);
    assert!(reply.body.contains(r#""code":"invalid_report""#), "{}", reply.body);
}

#[tokio::test]
async fn an_oversized_body_is_a_413() {
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let huge = format!(
        r#"{{"csp-report":{{"blocked-uri":"{}"}}}}"#,
        "x".repeat(9 * 1024)
    );
    let reply = post(
        address,
        "/api/csp-report",
        &[("Content-Type", "application/csp-report")],
        &huge,
    )
    .await;

    assert_eq!(reply.status, 413, "{} {}", reply.headers, reply.body);
}

#[tokio::test]
async fn a_get_is_not_allowed() {
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = support::get(address, "/api/csp-report", &[]).await;

    assert_eq!(reply.status, 405);
}

#[tokio::test]
async fn bursts_past_the_limit_are_throttled() {
    // A public write endpoint must bound its log volume: one shared bucket allows a
    // burst, then refuses with 429 until the window resets.
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    for _ in 0..60 {
        let reply = post(
            address,
            "/api/csp-report",
            &[("Content-Type", "application/csp-report")],
            report_body(),
        )
        .await;
        assert_eq!(reply.status, 204);
    }
    let reply = post(
        address,
        "/api/csp-report",
        &[("Content-Type", "application/csp-report")],
        report_body(),
    )
    .await;
    assert_eq!(reply.status, 429);
}
