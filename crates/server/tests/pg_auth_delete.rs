//! Account deletion over real Postgres (auth T36, requirements R17.1 to R17.5 and R16.4).
//!
//! Run with `DATABASE_URL` set: `cargo test -p counterspell-server --test pg_auth_delete -- --ignored`.

use std::{net::SocketAddr, sync::Arc, time::Duration as StdDuration};

use axum::{
    body::{to_bytes, Body},
    extract::connect_info::MockConnectInfo,
    http::{header, HeaderMap, Method, Request, StatusCode},
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
        AuthState,
    },
    client_ip::TrustedProxies,
    ratelimit::{Limiter, MemoryLimiter},
    testkit::ensure_problem,
};
use serde_json::{json, Value};
use sqlx::{types::Uuid, PgPool};
use time::OffsetDateTime;
use tower::ServiceExt;

const HANDLE: &str = "delete-me-handle";
const EMAIL: &str = "delete.me@example.co";
const OTHER_EMAIL: &str = "keep.me@example.co";

fn clock() -> Arc<ManualClock> {
    Arc::new(ManualClock::new(
        OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp"),
    ))
}

/// The auth router with no mail and no OAuth, which deletion does not need.
fn app(pool: &PgPool, clock: &Arc<ManualClock>) -> Router {
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
    );
    let peer = SocketAddr::from(([203, 0, 113, 9], 40_000));
    counterspell_server::auth::router(state).layer(MockConnectInfo(peer))
}

/// Sends one request with an optional session cookie. Returns the status, headers and body text.
async fn send(
    app: &Router,
    method: Method,
    path: &str,
    body: Option<Value>,
    cookie: Option<&str>,
) -> (StatusCode, HeaderMap, String) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(value) = cookie {
        builder = builder.header(header::COOKIE, format!("cs_session={value}"));
    }
    let body = match body {
        Some(json) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(json.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(builder.body(body).expect("valid request"))
        .await
        .expect("the router never fails");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("a small body");
    (
        status,
        headers,
        String::from_utf8_lossy(&bytes).into_owned(),
    )
}

/// Creates a verified user with an optional email, and returns its id.
async fn insert_user(pool: &PgPool, handle: &str, email: Option<&str>) -> UserId {
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (handle, display_name, email, email_verified_at)
         VALUES ($1, $1, $2, CASE WHEN $2::text IS NULL THEN NULL ELSE now() END)
         RETURNING id",
    )
    .bind(handle)
    .bind(email)
    .fetch_one(pool)
    .await
    .expect("insert user");
    UserId::from_uuid(id)
}

/// Creates a real session for a user and returns the raw cookie value.
async fn session_for(pool: &PgPool, clock: &Arc<ManualClock>, user: UserId) -> String {
    SessionService::new(pool.clone(), clock.clone())
        .create(user, AuthMethod::MagicLink)
        .await
        .expect("a session")
        .cookie_value
        .expose()
        .to_owned()
}

/// A 32-byte value unique to one user, for the hash columns that must not collide between users.
fn hash_for(user: UserId) -> Vec<u8> {
    let id: [u8; 16] = *user.as_uuid().as_bytes();
    let mut hash = id.to_vec();
    hash.extend_from_slice(&id);
    hash
}

/// Gives a user a GitHub identity, a link flow bound to its session, an unused email token for
/// `address`, and one finished run with its events.
async fn fill_account(pool: &PgPool, user: UserId, address: &str) {
    sqlx::query(
        "INSERT INTO identities (user_id, provider, provider_user_id) VALUES ($1, 'github', $2)",
    )
    .bind(user.as_uuid())
    .bind(format!("gh-{}", user.as_uuid()))
    .execute(pool)
    .await
    .expect("identity");
    // The caller already created the session; this reuses it, so the account has one session.
    let (session_key,): (String,) =
        sqlx::query_as("SELECT id FROM sessions WHERE user_id = $1 LIMIT 1")
            .bind(user.as_uuid())
            .fetch_one(pool)
            .await
            .expect("the session key");
    sqlx::query(
        "INSERT INTO oauth_flows (state_hash, provider, purpose, pkce_verifier, session_id, user_id,
                                  created_at, expires_at)
         VALUES ($1, 'github', 'link', $2, $3, $4, now(), now() + interval '5 minutes')",
    )
    .bind(hash_for(user))
    .bind("v".repeat(43))
    .bind(session_key)
    .bind(user.as_uuid())
    .execute(pool)
    .await
    .expect("a link flow");
    sqlx::query(
        "INSERT INTO email_tokens (token_hash, purpose, email, user_id, created_at, expires_at)
         VALUES ($1, 'magic_link', $2, NULL, now(), now() + interval '15 minutes')",
    )
    .bind(hash_for(user))
    .bind(address)
    .execute(pool)
    .await
    .expect("an unused token");
    add_run(pool, user, "completed", "succeeded").await;
}

/// Adds one run for a user and returns its submission id. The job state is set by the caller.
async fn add_run(pool: &PgPool, user: UserId, status: &str, job_state: &str) -> Uuid {
    ensure_problem(pool).await.expect("the test problem");
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status,
                                  verdict, finished_at)
         VALUES (gen_random_uuid(), $1, 'two-sum', 'python', 'run', 'print(1)', $2, $3,
                 CASE WHEN $3 = 'completed' THEN 'accepted' END,
                 CASE WHEN $3 = 'completed' THEN now() END)
         RETURNING id",
    )
    .bind(user.as_uuid())
    .bind(vec![7_u8; 32])
    .bind(status)
    .fetch_one(pool)
    .await
    .expect("a submission");
    sqlx::query("INSERT INTO jobs (submission_id, state) VALUES ($1, $2)")
        .bind(id)
        .bind(job_state)
        .execute(pool)
        .await
        .expect("a job");
    sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload)
         VALUES ($1, 1, 1, 'compiling', '{}'::jsonb)",
    )
    .bind(id)
    .execute(pool)
    .await
    .expect("a run event");
    id
}

async fn count_for_user(pool: &PgPool, sql: &str, user: UserId) -> i64 {
    sqlx::query_scalar(sql)
        .bind(user.as_uuid())
        .fetch_one(pool)
        .await
        .expect("a count")
}

async fn count_for_run(pool: &PgPool, sql: &str, run: Uuid) -> i64 {
    sqlx::query_scalar(sql)
        .bind(run)
        .fetch_one(pool)
        .await
        .expect("a count")
}

async fn count_unused_tokens(pool: &PgPool, address: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM email_tokens WHERE email = $1::citext AND used_at IS NULL",
    )
    .bind(address)
    .fetch_one(pool)
    .await
    .expect("a count")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_delete_removes_the_account_and_everything_that_depends_on_it(pool: PgPool) {
    let clock = clock();
    let app = app(&pool, &clock);
    let user = insert_user(&pool, HANDLE, Some(EMAIL)).await;
    let cookie = session_for(&pool, &clock, user).await;
    fill_account(&pool, user, EMAIL).await;
    let run = add_run(&pool, user, "completed", "succeeded").await;
    let other_user = insert_user(&pool, "keep-me-handle", Some(OTHER_EMAIL)).await;
    let other_cookie = session_for(&pool, &clock, other_user).await;
    fill_account(&pool, other_user, OTHER_EMAIL).await;
    assert_eq!(
        count_unused_tokens(&pool, EMAIL).await,
        1,
        "a token exists before deletion"
    );

    let (status, headers, body) = send(
        &app,
        Method::DELETE,
        "/api/me",
        Some(json!({ "confirm_handle": HANDLE })),
        Some(&cookie),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "the account is deleted: {body}"
    );
    let cleared = headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .any(|line| line.starts_with("cs_session=") && line.contains("Max-Age=0"));
    assert!(cleared, "the session cookie is cleared");

    assert_eq!(
        count_for_user(&pool, "SELECT count(*) FROM users WHERE id = $1", user).await,
        0
    );
    for sql in [
        "SELECT count(*) FROM identities WHERE user_id = $1",
        "SELECT count(*) FROM sessions WHERE user_id = $1",
        "SELECT count(*) FROM oauth_flows WHERE user_id = $1",
        "SELECT count(*) FROM submissions WHERE user_id = $1",
    ] {
        assert_eq!(
            count_for_user(&pool, sql, user).await,
            0,
            "gone after deletion: {sql}"
        );
    }
    for sql in [
        "SELECT count(*) FROM jobs WHERE submission_id = $1",
        "SELECT count(*) FROM run_events WHERE submission_id = $1",
    ] {
        assert_eq!(
            count_for_run(&pool, sql, run).await,
            0,
            "run rows are gone: {sql}"
        );
    }
    assert_eq!(
        count_unused_tokens(&pool, EMAIL).await,
        0,
        "the unused token for the address is gone"
    );
    assert_eq!(
        count_unused_tokens(&pool, OTHER_EMAIL).await,
        1,
        "a token for another address stays"
    );

    // The other user is untouched: the same cookie still works and its run still exists.
    let (status, _, _) = send(&app, Method::GET, "/api/me", None, Some(&other_cookie)).await;
    assert_eq!(status, StatusCode::OK, "another user's session still works");
    assert_eq!(
        count_for_user(
            &pool,
            "SELECT count(*) FROM submissions WHERE user_id = $1",
            other_user
        )
        .await,
        1
    );
    assert_eq!(
        count_for_user(
            &pool,
            "SELECT count(*) FROM identities WHERE user_id = $1",
            other_user
        )
        .await,
        1
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_delete_with_a_wrong_handle_changes_nothing(pool: PgPool) {
    let clock = clock();
    let app = app(&pool, &clock);
    let user = insert_user(&pool, HANDLE, Some(EMAIL)).await;
    let cookie = session_for(&pool, &clock, user).await;
    fill_account(&pool, user, EMAIL).await;

    for wrong in ["not-my-handle", "", "DELETE-ME-HANDLE"] {
        let (status, _, body) = send(
            &app,
            Method::DELETE,
            "/api/me",
            Some(json!({ "confirm_handle": wrong })),
            Some(&cookie),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "a wrong handle is refused");
        assert!(
            body.contains("confirmation_mismatch"),
            "the code says why: {body}"
        );
    }

    assert_eq!(
        count_for_user(&pool, "SELECT count(*) FROM users WHERE id = $1", user).await,
        1
    );
    assert_eq!(
        count_for_user(
            &pool,
            "SELECT count(*) FROM sessions WHERE user_id = $1",
            user
        )
        .await,
        1,
        "the session is kept"
    );
    assert_eq!(
        count_unused_tokens(&pool, EMAIL).await,
        1,
        "the token is kept"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_delete_is_refused_while_a_run_is_queued_or_claimed(pool: PgPool) {
    let clock = clock();
    let app = app(&pool, &clock);
    let user = insert_user(&pool, HANDLE, Some(EMAIL)).await;
    let cookie = session_for(&pool, &clock, user).await;
    let run = add_run(&pool, user, "queued", "queued").await;

    let (status, _, body) = send(
        &app,
        Method::DELETE,
        "/api/me",
        Some(json!({ "confirm_handle": HANDLE })),
        Some(&cookie),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "a queued run blocks deletion: {body}"
    );
    assert!(body.contains("runs_active"), "the code says why: {body}");

    sqlx::query(
        "UPDATE jobs SET state = 'claimed', lease_owner = 'worker-a',
                         lease_expires_at = now() + interval '30 seconds', claimed_at = now()
         WHERE submission_id = $1",
    )
    .bind(run)
    .execute(&pool)
    .await
    .expect("claim the job");
    let (status, _, body) = send(
        &app,
        Method::DELETE,
        "/api/me",
        Some(json!({ "confirm_handle": HANDLE })),
        Some(&cookie),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "a claimed run blocks deletion: {body}"
    );

    assert_eq!(
        count_for_user(&pool, "SELECT count(*) FROM users WHERE id = $1", user).await,
        1
    );
    assert_eq!(
        count_for_run(&pool, "SELECT count(*) FROM submissions WHERE id = $1", run).await,
        1
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_delete_refuses_the_demo_account(pool: PgPool) {
    sqlx::query("SELECT cs_seed_demo_user($1, 'placeholder-hash')")
        .bind("demo@example.co")
        .execute(&pool)
        .await
        .expect("seed the demo account");
    let (demo_id,): (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE is_demo")
        .fetch_one(&pool)
        .await
        .expect("the demo user");
    let demo = UserId::from_uuid(demo_id);
    let clock = clock();
    let app = app(&pool, &clock);
    let cookie = session_for(&pool, &clock, demo).await;

    let (status, _, body) = send(
        &app,
        Method::DELETE,
        "/api/me",
        Some(json!({ "confirm_handle": "anything" })),
        Some(&cookie),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "the demo account is refused: {body}"
    );
    assert!(
        body.contains("demo_account_restricted"),
        "the code says why: {body}"
    );
    assert_eq!(
        count_for_user(&pool, "SELECT count(*) FROM users WHERE id = $1", demo).await,
        1
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_delete_without_a_session_is_unauthenticated(pool: PgPool) {
    let clock = clock();
    let app = app(&pool, &clock);
    let user = insert_user(&pool, HANDLE, Some(EMAIL)).await;

    let (status, _, _) = send(
        &app,
        Method::DELETE,
        "/api/me",
        Some(json!({ "confirm_handle": HANDLE })),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        count_for_user(&pool, "SELECT count(*) FROM users WHERE id = $1", user).await,
        1
    );
}
