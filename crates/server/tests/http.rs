//! HTTP behaviour of the router: health, the JSON 404 for unknown API paths, static files and the SPA fallback.

mod support;

use counterspell_server::build_router;
use support::{get, head, serve, state_for, Fixture};

fn header_value<'a>(headers: &'a str, name: &str) -> Option<&'a str> {
    headers.lines().find_map(|line| {
        let (header_name, value) = line.split_once(':')?;
        header_name.eq_ignore_ascii_case(name).then(|| value.trim())
    })
}

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
async fn immutable_build_assets_receive_a_year_cache_policy() {
    // test-change: OPS-PERF-STATIC-CACHE
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = get(address, "/_app/immutable/app.js", &[]).await;

    assert_eq!(reply.status, 200);
    assert!(
        reply
            .headers
            .to_lowercase()
            .contains("cache-control: public, max-age=31536000, immutable"),
        "{}",
        reply.headers
    );
}

#[tokio::test]
async fn named_site_assets_receive_a_week_cache_policy() {
    // test-change: OPS-PERF-STATIC-CACHE
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    for path in ["/art/ghost.png", "/fonts/coven.woff2", "/landing/hero.webp"] {
        let reply = get(address, path, &[]).await;

        assert_eq!(reply.status, 200, "{path}");
        assert!(
            reply
                .headers
                .to_lowercase()
                .contains("cache-control: public, max-age=604800"),
            "{path}: {}",
            reply.headers
        );
    }
}

#[tokio::test]
async fn robots_receives_a_short_cache_policy() {
    // test-change: OPS-PERF-STATIC-CACHE
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = get(address, "/robots.txt", &[]).await;

    assert_eq!(reply.status, 200);
    assert!(
        reply
            .headers
            .to_lowercase()
            .contains("cache-control: public, max-age=3600"),
        "{}",
        reply.headers
    );
}

#[tokio::test]
async fn missing_immutable_asset_is_a_404_and_is_never_cached() {
    // test-change: OPS-PERF-STATIC-CACHE, PROD-FOUND-404
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = get(address, "/_app/immutable/missing.js", &[]).await;

    assert_eq!(reply.status, 404);
    assert_eq!(reply.body, "<main>spa-entry</main>");
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
async fn immutable_cache_policy_survives_revalidation_without_caching_the_fallback() {
    // test-change: OPS-PERF-STATIC-CACHE
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let asset = get(address, "/_app/immutable/app.js", &[]).await;
    let asset_etag = header_value(&asset.headers, "etag").expect("asset etag");
    let revalidated_asset = get(
        address,
        "/_app/immutable/app.js",
        &[("If-None-Match", asset_etag)],
    )
    .await;

    assert_eq!(revalidated_asset.status, 304);
    assert!(
        revalidated_asset
            .headers
            .to_lowercase()
            .contains("cache-control: public, max-age=31536000, immutable"),
        "{}",
        revalidated_asset.headers
    );

    let fallback = get(address, "/_app/immutable/missing.js", &[]).await;
    let fallback_etag = header_value(&fallback.headers, "etag").expect("fallback etag");
    let revalidated_fallback = get(
        address,
        "/_app/immutable/missing.js",
        &[("If-None-Match", fallback_etag)],
    )
    .await;

    assert_eq!(revalidated_fallback.status, 404);
    assert!(
        revalidated_fallback
            .headers
            .to_lowercase()
            .contains("cache-control: no-store"),
        "{}",
        revalidated_fallback.headers
    );
}

#[tokio::test]
async fn traversal_path_cannot_probe_files_outside_the_static_directory() {
    // test-change: OPS-PERF-STATIC-CACHE
    let fixture = Fixture::new();
    let parent = fixture.dir.parent().expect("fixture parent");
    let secret_name = format!(
        "{}-outside.txt",
        fixture
            .dir
            .file_name()
            .expect("fixture name")
            .to_string_lossy()
    );
    let secret_path = parent.join(&secret_name);
    std::fs::write(&secret_path, "outside").expect("outside fixture");
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = get(
        address,
        &format!("/_app/immutable/../../../{secret_name}"),
        &[],
    )
    .await;
    std::fs::remove_file(secret_path).expect("remove outside fixture");

    assert_eq!(reply.status, 404);
    assert_eq!(reply.body, "<main>spa-entry</main>");
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
async fn percent_encoded_asset_names_still_serve_the_real_file() {
    // test-change: PROD-FOUND-404
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = get(address, "/landing/hero%20image.webp", &[]).await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, "spaced-landing");
}

#[tokio::test]
async fn directory_index_html_is_served_like_the_file() {
    // test-change: PROD-FOUND-404
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = get(address, "/docs/", &[]).await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, "<main>docs-index</main>");
}

#[tokio::test]
async fn unknown_route_is_an_honest_404_with_the_spa_entry_body() {
    // test-change: PROD-FOUND-404
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    for path in [
        "/runs/123",
        "/no-such-route-xyz",
        "/problems/a/b",
        // Router semantics keep encoded separators and interior empties honest:
        // none of these are declared routes, files or API paths.
        "/%61pi/health",
        "/break%2Fmemory",
        "/dashboard//",
        "/%2Flanding/moon.svg",
    ] {
        let reply = get(address, path, &[]).await;

        assert_eq!(reply.status, 404, "{path}");
        assert_eq!(reply.body, "<main>spa-entry</main>", "{path}");
        assert!(
            reply
                .headers
                .to_lowercase()
                .contains("cache-control: no-store"),
            "{path}: {}",
            reply.headers
        );
    }
}

#[tokio::test]
async fn conditional_requests_on_a_miss_still_get_the_full_shell_404() {
    // test-change: PROD-FOUND-404
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    for headers in [
        ("if-none-match", "*"),
        ("if-modified-since", "Wed, 01 Jan 2031 00:00:00 GMT"),
        ("range", "bytes=0-9"),
    ] {
        let reply = get(address, "/no-such-route-xyz", &[headers]).await;

        assert_eq!(reply.status, 404, "{headers:?}");
        assert_eq!(reply.body, "<main>spa-entry</main>", "{headers:?}");
    }

    let reply = get(address, "/dashboard", &[("if-none-match", "*")]).await;
    assert_ne!(reply.status, 404);
}

#[tokio::test]
async fn missing_static_resources_are_an_honest_404() {
    // test-change: PROD-FOUND-404
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    for path in ["/definitely-missing-asset.png", "/security.txt"] {
        let reply = get(address, path, &[]).await;

        assert_eq!(reply.status, 404, "{path}");
        assert!(
            reply
                .headers
                .to_lowercase()
                .contains("cache-control: no-store"),
            "{path}: {}",
            reply.headers
        );
    }
}

#[tokio::test]
async fn published_web_resources_are_real_files_not_fallback_404s() {
    // test-change: PROD-FOUND-RESOURCES
    let fixture = Fixture::new();
    let web_static = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/static");
    for resource in [
        "sitemap.xml",
        "manifest.webmanifest",
        ".well-known/security.txt",
    ] {
        let target = fixture.dir.join(resource);
        std::fs::create_dir_all(target.parent().expect("resource dir")).expect("mkdir");
        std::fs::copy(web_static.join(resource), &target)
            .unwrap_or_else(|error| panic!("copy {resource}: {error}"));
    }
    let address = serve(build_router(state_for(&fixture.dir))).await;

    for (path, type_marker, body_marker) in [
        ("/sitemap.xml", "application/xml", "<urlset"),
        (
            "/manifest.webmanifest",
            "application/manifest+json",
            "\"name\"",
        ),
        ("/.well-known/security.txt", "text/plain", "Contact:"),
    ] {
        let reply = get(address, path, &[]).await;

        assert_eq!(reply.status, 200, "{path}");
        assert!(
            reply
                .headers
                .to_lowercase()
                .contains(&format!("content-type: {type_marker}")),
            "{path}: {}",
            reply.headers
        );
        assert!(reply.body.contains(body_marker), "{path}: {}", reply.body);
    }
}

#[tokio::test]
async fn declared_spa_routes_still_fall_back_to_the_spa_entry() {
    // test-change: PROD-FOUND-404
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    for path in [
        "/dashboard",
        "/problems/two-sum",
        "/dashboard/",
        "/problems/a%2Fb",
    ] {
        let reply = get(address, path, &[]).await;

        assert_eq!(reply.status, 200, "{path}");
        assert_eq!(reply.body, "<main>spa-entry</main>", "{path}");
    }
}

#[tokio::test]
async fn liveness_is_documented_json_and_never_spa_html() {
    // test-change: PROD-FOUND-HEALTH
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = get(address, "/api/health", &[]).await;
    assert_eq!(reply.status, 200);
    assert!(
        reply
            .headers
            .to_lowercase()
            .contains("content-type: application/json"),
        "{}",
        reply.headers
    );
    assert!(reply.body.contains("\"status\":\"ok\""), "{}", reply.body);

    // Anything outside /api must never pass for a health reply: these paths are
    // not declared SPA routes, so they are honest 404s, not the app shell at 200.
    for path in ["/health", "/ready", "/healthz", "/readiness"] {
        let reply = get(address, path, &[]).await;
        assert_eq!(reply.status, 404, "{path}");
    }
}

#[tokio::test]
async fn anonymous_catalogue_gets_are_publicly_cacheable() {
    // test-change: PROD-FOUND-CACHE
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let reply = get(address, "/api/languages", &[]).await;
    assert_eq!(reply.status, 200);
    let headers = reply.headers.to_lowercase();
    assert!(
        headers.contains("cache-control: public, max-age=300"),
        "{headers}"
    );
    assert!(headers.contains("vary: cookie"), "{headers}");
    let etag = header_value(&reply.headers, "etag").expect("catalogue etag");

    let revalidated = get(address, "/api/languages", &[("If-None-Match", etag)]).await;
    assert_eq!(revalidated.status, 304);
    assert!(
        revalidated
            .headers
            .to_lowercase()
            .contains("cache-control: public, max-age=300"),
        "{}",
        revalidated.headers
    );

    // `*` and list forms of If-None-Match revalidate the same way; a foreign tag does not.
    let wildcard = get(address, "/api/languages", &[("If-None-Match", "*")]).await;
    assert_eq!(wildcard.status, 304);
    let list = format!("\"deadbeef\", {etag}");
    let listed = get(address, "/api/languages", &[("If-None-Match", &list)]).await;
    assert_eq!(listed.status, 304);
    let stale = get(
        address,
        "/api/languages",
        &[("If-None-Match", "\"deadbeef\"")],
    )
    .await;
    assert_eq!(stale.status, 200);

    // A session cookie means per-user data could ride along: never cache it.
    let authed = get(
        address,
        "/api/languages",
        &[("Cookie", "__Host-cs_session=abc123")],
    )
    .await;
    assert_eq!(authed.status, 200);
    assert!(
        authed
            .headers
            .to_lowercase()
            .contains("cache-control: no-store"),
        "{}",
        authed.headers
    );
}

#[tokio::test]
async fn catalogue_head_mirrors_the_get_cache_policy() {
    // test-change: PROD-FOUND-CACHE-HEAD — a HEAD must carry the same caching headers
    // as the GET it mirrors, or shared caches and probes see a dishonest `no-store`.
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    let get_reply = get(address, "/api/languages", &[]).await;
    let head_reply = head(address, "/api/languages", &[]).await;

    assert_eq!(head_reply.status, 200);
    assert_eq!(
        header_value(&head_reply.headers, "cache-control"),
        header_value(&get_reply.headers, "cache-control"),
        "HEAD and GET cache policies diverged: {:?} vs {:?}",
        head_reply.headers,
        get_reply.headers
    );
    assert_eq!(
        header_value(&head_reply.headers, "etag"),
        header_value(&get_reply.headers, "etag"),
    );
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

#[test]
fn startup_check_requires_the_spa_route_manifest() {
    // test-change: PROD-FOUND-404
    let fixture = Fixture::new();
    std::fs::remove_file(fixture.dir.join("routes.json")).expect("remove manifest");

    let error = counterspell_server::ensure_static_files(&fixture.dir)
        .expect_err("a directory without routes.json must fail the start-up check");

    assert!(error.to_string().contains("routes.json"), "{error}");
}

#[tokio::test]
async fn languages_endpoint_lists_only_enabled_drivers() {
    // LC03: the accepted catalogue derives from the judge file — a disabled driver is unlisted.
    let fixture = Fixture::new();
    std::fs::write(fixture.dir.join("driver.py"), "# driver\n").expect("driver file");
    let text = r#"
[python]
enabled = true
image = "python:3.13-slim@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90"
file = "sol.py"
driver = "driver.py"
compile = []
run = ["python3", "driver.py"]
bench_args = []
mem_mb = 64
time_ms_per_test = 1000
max_time_ms_per_test = 5000
compile_budget_ms = 5000
max_source_bytes = 65536
exec_scratch = false
protocol_version = 1

[java]
enabled = false
image = "eclipse-temurin:21-jdk@sha256:0a3f329821dcadf54b2a3033a15e2e9f007069a9db43d7f51897e6b80e726b07"
file = "Sol.java"
driver = "Driver.java"
compile = []
run = ["java", "Driver.java"]
bench_args = []
mem_mb = 256
time_ms_per_test = 1000
max_time_ms_per_test = 5000
compile_budget_ms = 10000
max_source_bytes = 65536
exec_scratch = true
protocol_version = 1
"#;
    let judge = counterspell_common::languages::LanguageCatalog::parse(text, &fixture.dir)
        .expect("catalogue parses");
    let state = state_for(&fixture.dir).with_languages(
        counterspell_server::seams::LanguageCatalog::from_enabled(&judge),
    );
    let address = serve(build_router(state)).await;

    let reply = get(address, "/api/languages", &[]).await;

    assert_eq!(reply.status, 200);
    assert!(reply.body.contains(r#""id":"python""#), "{}", reply.body);
    assert!(
        !reply.body.contains(r#""id":"java""#),
        "disabled drivers must not be advertised: {}",
        reply.body
    );
}
