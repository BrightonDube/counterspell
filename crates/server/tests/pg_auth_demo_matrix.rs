//! The demo account's restrictions, checked on every mutating auth route (auth T27, properties
//! P17, requirements R16.4 to R16.6).
//!
//! Run with `DATABASE_URL` set: `cargo test -p counterspell-server pg_auth_demo_matrix -- --include-ignored`.

use std::{net::SocketAddr, sync::Arc, time::Duration as StdDuration};

use axum::{
    body::{to_bytes, Body},
    extract::connect_info::MockConnectInfo,
    http::{header, HeaderMap, Method, Request, StatusCode},
    response::Response,
    Router,
};
use counterspell_common::ids::UserId;
use counterspell_server::{
    auth::{
        clock::ManualClock,
        config::Redacted,
        cookies::CookieSettings,
        limits::AuthLimiter,
        password::HashGate,
        session::{AuthMethod, SessionService},
        AuthState, MUTATING_ROUTES,
    },
    client_ip::TrustedProxies,
    ratelimit::{Limiter, MemoryLimiter},
};
use serde_json::{json, Value};
use sqlx::{types::Uuid, PgPool};
use time::OffsetDateTime;
use tower::ServiceExt;

const DEMO: &str = "demo@example.co";

// test-change: AUTH-EMAIL-REMOVAL
/// The router under test.
struct Harness {
    app: Router,
    clock: Arc<ManualClock>,
}

fn harness(pool: &PgPool) -> Harness {
    let clock = Arc::new(ManualClock::new(
        OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp"),
    ));
    let limits = AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![5_u8; 32]),
    );
    let state = AuthState::new(
        pool.clone(),
        clock.clone(),
        CookieSettings::new(false),
        limits,
        HashGate::new(4, StdDuration::from_secs(2)),
        TrustedProxies::parse("10.0.0.0/8").expect("valid trusted proxies"),
    )
    .with_oauth(
        "https://counterspell.example".to_owned(),
        vec![Arc::new(
            counterspell_server::auth::oauth::fake::FakeProvider::new(
                counterspell_server::auth::oauth::ProviderId::GitHub,
                vec![],
            )
            .expect("the fake URL parses"),
        )],
    );
    let peer = SocketAddr::from(([203, 0, 113, 9], 40_000));
    Harness {
        app: counterspell_server::auth::router(state).layer(MockConnectInfo(peer)),
        clock,
    }
}

/// Sends one request with an optional JSON body and session cookie.
async fn call(
    app: &Router,
    method: Method,
    path: &str,
    body: Option<Value>,
    session: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(raw) = session {
        builder = builder.header(header::COOKIE, format!("cs_session={raw}"));
    }
    let body = match body {
        Some(json) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(json.to_string())
        }
        None => Body::empty(),
    };
    let response: Response = app
        .clone()
        .oneshot(builder.body(body).expect("valid request"))
        .await
        .expect("the router never fails");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("a small body");
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, headers, json)
}

/// Creates the demo account through the database seed function, as start-up does.
async fn seed_demo(pool: &PgPool) -> UserId {
    sqlx::query("SELECT cs_seed_demo_user($1, 'placeholder-hash')")
        .bind(DEMO)
        .execute(pool)
        .await
        .expect("seed the demo account");
    let (id,): (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE is_demo")
        .fetch_one(pool)
        .await
        .expect("the demo user");
    UserId::from_uuid(id)
}

async fn session_for(pool: &PgPool, clock: &Arc<ManualClock>, user: UserId) -> String {
    SessionService::new(pool.clone(), clock.clone())
        .create(user, AuthMethod::Password)
        .await
        .expect("a session")
        .cookie_value
        .expose()
        .to_owned()
}

/// The rows that a restricted action could change. Equal snapshots mean nothing changed.
async fn snapshot(pool: &PgPool) -> (i64, i64, i64, i64, i64, Option<String>, Option<String>) {
    let (users,): (i64,) = sqlx::query_as("SELECT count(*) FROM users")
        .fetch_one(pool)
        .await
        .expect("count users");
    let (identities,): (i64,) = sqlx::query_as("SELECT count(*) FROM identities")
        .fetch_one(pool)
        .await
        .expect("count identities");
    let (sessions,): (i64,) = sqlx::query_as("SELECT count(*) FROM sessions")
        .fetch_one(pool)
        .await
        .expect("count sessions");
    let (tokens,): (i64,) = sqlx::query_as("SELECT count(*) FROM email_tokens")
        .fetch_one(pool)
        .await
        .expect("count tokens");
    let (flows,): (i64,) = sqlx::query_as("SELECT count(*) FROM oauth_flows")
        .fetch_one(pool)
        .await
        .expect("count flows");
    let (email, hash): (Option<String>, Option<String>) =
        sqlx::query_as("SELECT email::text, password_hash FROM users WHERE is_demo")
            .fetch_one(pool)
            .await
            .expect("the demo account");
    (users, identities, sessions, tokens, flows, email, hash)
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_demo_matrix_restricted_actions_give_403_and_change_nothing(pool: PgPool) {
    let demo = seed_demo(&pool).await;
    let h = harness(&pool);
    let session = session_for(&pool, &h.clock, demo).await;

    let before = snapshot(&pool).await;
    let mut answers = Vec::new();
    // test-change: AUTH-EMAIL-REMOVAL
    for (path, body) in [
        ("/api/auth/logout-all", None),
        ("/api/auth/link/github/start", None),
    ] {
        let (status, _, json) = call(&h.app, Method::POST, path, body, Some(&session)).await;
        answers.push((status, json["code"].clone()));
    }

    assert_eq!(
        answers,
        vec![
            (StatusCode::FORBIDDEN, json!("demo_account_restricted")),
            (StatusCode::FORBIDDEN, json!("demo_account_restricted")),
        ]
    );
    assert_eq!(snapshot(&pool).await, before, "the database is unchanged");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_demo_matrix_logout_works_for_the_demo_session(pool: PgPool) {
    let demo = seed_demo(&pool).await;
    let h = harness(&pool);
    let session = session_for(&pool, &h.clock, demo).await;

    let (logout, _, _) = call(
        &h.app,
        Method::POST,
        "/api/auth/logout",
        None,
        Some(&session),
    )
    .await;
    let (me, _, _) = call(&h.app, Method::GET, "/api/me", None, Some(&session)).await;

    assert_eq!(logout, StatusCode::NO_CONTENT);
    assert_eq!(me, StatusCode::UNAUTHORIZED, "the session ends at once");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_demo_matrix_triggers_refuse_direct_changes(pool: PgPool) {
    seed_demo(&pool).await;

    let email = sqlx::query("UPDATE users SET email = 'other@example.co' WHERE is_demo")
        .execute(&pool)
        .await;
    let password = sqlx::query("UPDATE users SET password_hash = 'x' WHERE is_demo")
        .execute(&pool)
        .await;
    let identity = sqlx::query(
        "INSERT INTO identities (user_id, provider, provider_user_id)
         SELECT id, 'github', 'demo-link' FROM users WHERE is_demo",
    )
    .execute(&pool)
    .await;
    let deleted = sqlx::query("DELETE FROM users WHERE is_demo")
        .execute(&pool)
        .await;

    assert!(email.is_err(), "the demo address cannot change");
    assert!(password.is_err(), "the demo password cannot change");
    assert!(
        identity.is_err(),
        "no provider can be linked to the demo account"
    );
    assert!(deleted.is_err(), "the demo account cannot be deleted");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_demo_matrix_every_listed_mutating_route_exists_for_a_session(pool: PgPool) {
    let demo = seed_demo(&pool).await;
    let h = harness(&pool);
    let session = session_for(&pool, &h.clock, demo).await;

    for path in MUTATING_ROUTES {
        let (status, _, _) =
            call(&h.app, Method::POST, path, Some(json!({})), Some(&session)).await;
        assert_ne!(
            status,
            StatusCode::NOT_FOUND,
            "{path} is listed but the router does not serve it"
        );
        assert_ne!(
            status,
            StatusCode::METHOD_NOT_ALLOWED,
            "{path} must accept POST"
        );
    }
}
