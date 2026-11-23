//! HTTP behaviour of the router: health, the JSON 404 for unknown API paths, static files and the SPA fallback.

mod support;

use counterspell_server::build_router;
use support::{get, serve, state_for, Fixture};

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

    for path in ["/runs/123", "/no-such-route-xyz", "/problems/a/b"] {
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
async fn missing_static_resources_are_an_honest_404() {
    // test-change: PROD-FOUND-404
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    for path in [
        "/definitely-missing-asset.png",
        "/sitemap.xml",
        "/manifest.webmanifest",
        "/.well-known/security.txt",
        "/security.txt",
    ] {
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
async fn declared_spa_routes_still_fall_back_to_the_spa_entry() {
    // test-change: PROD-FOUND-404
    let fixture = Fixture::new();
    let address = serve(build_router(state_for(&fixture.dir))).await;

    for path in ["/dashboard", "/problems/two-sum", "/dashboard/"] {
        let reply = get(address, path, &[]).await;

        assert_eq!(reply.status, 200, "{path}");
        assert_eq!(reply.body, "<main>spa-entry</main>", "{path}");
    }
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
