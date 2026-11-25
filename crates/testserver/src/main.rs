//! `counterspell-testserver`: a real Axum server plus a scripted judge for
//! web and browser verification. It runs the production router against a real
//! Postgres database, accepts `Bearer test-user-N` identities, publishes the
//! `two-sum` fixture problem and executes queued runs through
//! `ScriptedExecutor`, so clients see only real JSON envelopes and SSE frames.
//!
//! Only builds with `--features testkit`; release builds never contain it.
//!
//! ```sh
//! DATABASE_URL=postgres://... CS_TEST_PORT=5198 \
//!   cargo run -p counterspell-testserver
//! ```

use std::{env, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use counterspell_common::ids::UserId;
use counterspell_judge::{
    catalogue::testkit::insert_published_fixture,
    config::JudgeConfig,
    testkit::{Script, ScriptedExecutor},
    worker::Worker,
};
use counterspell_server::{
    build_router,
    config::ServerConfig,
    ratelimit::{Limiter, MemoryLimiter},
    runs::wake::RunWaker,
    seams::UserResolver,
    testkit::{ensure_user, BearerTestResolver},
    AppState,
};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tokio::{net::TcpListener, sync::watch};
use uuid::Uuid;

/// Seeded identities. Each web integration suite run draws fresh users from
/// this pool, and the per-user run limit counts over a 600 s window, so the
/// pool must outlast that window when suites are rerun back to back.
const TEST_USERS: u32 = 4096;
const SCRIPTED_TESTS: u32 = 3;
const SCRIPT_DELAY: Duration = Duration::from_millis(30);

fn owner(number: u32) -> UserId {
    UserId::from_uuid(Uuid::from_u128(u128::from(number)))
}

fn server_config(database_url: &str) -> ServerConfig {
    let url = database_url.to_owned();
    ServerConfig::from_lookup(move |name| match name {
        "DATABASE_URL" => Some(url.clone()),
        // The fixture uses the in-memory limiter; a URL is still required by config.
        "VALKEY_URL" => Some("redis://127.0.0.1:1".to_owned()),
        "STATIC_DIR" => Some(env::var("STATIC_DIR").unwrap_or_else(|_| "web/build".to_owned())),
        _ => env::var(name).ok(),
    })
    .expect("server config")
}

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("connect to Postgres");
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("migrations");

    for number in 1..=TEST_USERS {
        ensure_user(&pool, owner(number))
            .await
            .expect("user fixture");
    }
    let published: Option<Option<i32>> =
        sqlx::query_scalar("SELECT published_version FROM problems WHERE slug = 'two-sum'")
            .fetch_optional(&pool)
            .await
            .expect("problem lookup");
    if !matches!(published, Some(Some(_))) {
        insert_published_fixture(&pool, "two-sum", 1)
            .await
            .expect("published two-sum fixture");
    }

    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let state = AppState::new(pool.clone(), server_config(&database_url), limiter)
        .with_user_resolver(BearerTestResolver)
        .with_run_waker(RunWaker::start(pool.clone()));

    let (stop_tx, stop_rx) = watch::channel(false);
    let judge_pool = PgPoolOptions::new()
        .max_connections(4)
        .connect_with((*pool.connect_options()).clone())
        .await
        .expect("judge pool");
    let worker = Arc::new(Worker::new(
        judge_pool,
        JudgeConfig::from_lookup(|_| None::<String>).expect("judge defaults"),
        ScriptedExecutor::new(Script::Pass {
            tests: SCRIPTED_TESTS,
            delay: SCRIPT_DELAY,
        }),
        "fixture-judge".to_owned(),
    ));
    tokio::spawn(worker.run(stop_rx));

    // The auth router is not mounted (no AuthState), so the session probe and
    // `/api/me` mirror the production anonymous contract over the Bearer fixture
    // identities: the probe always 200s, `/api/me` 401s for signed-out visitors.
    // Without them a signed-out e2e visit would see 404s and double probes.
    let app = build_router(state).merge(bearer_session_routes(pool.clone()));

    let port = env::var("CS_TEST_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(5198);
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
        .await
        .expect("bind test port");
    let address = listener.local_addr().expect("address");
    println!("testserver listening on http://{address}");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("serve");
    let _keep = stop_tx;
}

/// State for the session/me shim routes.
#[derive(Clone)]
struct BearerAuthShim {
    pool: PgPool,
}

/// The auth module's anonymous contract over `Bearer test-user-N` identities.
/// Mounted by the testserver in place of the real auth router: `GET /api/auth/session`
/// always answers 200 (the probe public pages use so signed-out visits never
/// 401) and `GET /api/me` answers the fixture user's own row or 401.
fn bearer_session_routes(pool: PgPool) -> Router {
    Router::new()
        .route("/api/auth/session", get(session_status))
        .route("/api/auth/config", get(auth_config))
        .route("/api/me", get(bearer_me))
        .with_state(BearerAuthShim { pool })
}

/// `GET /api/auth/config`: the testserver signs in via `Bearer` headers, so it
/// advertises no interactive methods — the same shape production returns.
async fn auth_config() -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({
            "providers": { "github": false, "google": false },
            "email": { "magic_link": false, "password": false },
            "demo": { "enabled": false, "email": null }
        })),
    )
}

async fn session_status(headers: HeaderMap) -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({
            "signed_in": BearerTestResolver.resolve(&headers).await.is_some()
        })),
    )
}

async fn bearer_me(
    State(shim): State<BearerAuthShim>,
    headers: HeaderMap,
) -> axum::response::Response {
    let no_store = [(header::CACHE_CONTROL, "no-store")];
    let Some(user_id) = BearerTestResolver.resolve(&headers).await else {
        return (
            StatusCode::UNAUTHORIZED,
            no_store,
            Json(json!({ "code": "unauthenticated", "message": "Sign in to continue" })),
        )
            .into_response();
    };
    let row = sqlx::query(
        "SELECT handle::text, display_name, email::text, country,
                profile_completed_at IS NOT NULL, is_demo
         FROM users WHERE id = $1",
    )
    .bind(user_id.as_uuid())
    .fetch_optional(&shim.pool)
    .await;
    let Ok(Some(row)) = row else {
        return (
            StatusCode::UNAUTHORIZED,
            no_store,
            Json(json!({ "code": "unauthenticated", "message": "Sign in to continue" })),
        )
            .into_response();
    };
    let providers: Vec<String> = sqlx::query_scalar(
        "SELECT provider::text FROM identities WHERE user_id = $1 ORDER BY provider",
    )
    .bind(user_id.as_uuid())
    .fetch_all(&shim.pool)
    .await
    .unwrap_or_default();
    (
        no_store,
        Json(json!({
            "user": {
                "id": user_id,
                "handle": row.get::<String, _>("handle"),
                "display_name": row.get::<String, _>("display_name"),
                "email": row.get::<Option<String>, _>("email"),
                "country": row.get::<Option<String>, _>("country"),
                "profile_complete": row.get::<bool, _>(4),
                "avatar_url": "/art/creatures/skull-green-eyes.webp",
                "avatar_editable": false,
                "is_demo": row.get::<bool, _>(5),
                "providers": providers,
            },
            "capabilities": {
                "link_provider": true,
                "change_credentials": true,
                "delete_account": true,
                "logout_everywhere": true,
                "edit_profile": true,
            },
        })),
    )
        .into_response()
}
