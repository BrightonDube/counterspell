//! The efficiency-familiar sections of `GET /api/runs/{id}` over real Postgres (efficiency-familiar T23,
//! requirements R7.4, R7.8 and R7.9). Each test writes the stored measured columns directly, as the judge
//! would, and checks what the read model returns.
//!
//! Run with `DATABASE_URL` set: `cargo test -p counterspell-server --test familiar_read -- --ignored pg_`.

mod support;

use std::{collections::BTreeMap, sync::Arc};

use counterspell_common::{
    familiar::{
        class::{ClassId, Confidence},
        payload::{BenchStatus, FitSummary},
    },
    ids::UserId,
    status::RunMode,
};
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
    testkit::{ensure_user, InMemoryProblems},
    AppState,
};
use serde_json::Value;
use sqlx::{types::Uuid, PgPool};
use support::{config_for, get, serve, Fixture, Reply};
use time::OffsetDateTime;

const ORIGIN: &str = "https://counterspell.example";

/// The full application state with real auth, a one-problem catalogue and a fixed clock.
fn state(pool: &PgPool, static_dir: &std::path::Path, clock: Arc<ManualClock>) -> AppState {
    let limits = AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![5_u8; 32]),
    );
    let auth = AuthState::new(
        pool.clone(),
        clock,
        CookieSettings::new(false),
        limits,
        HashGate::new(4, std::time::Duration::from_secs(2)),
        TrustedProxies::parse("10.0.0.0/8").expect("valid trusted proxies"),
    );
    AppState::new(
        pool.clone(),
        config_for(static_dir),
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
    )
    .with_problems(InMemoryProblems::new(&["two-sum"]))
    .with_auth(auth, ORIGIN)
    .expect("valid origin")
}

fn clock() -> Arc<ManualClock> {
    Arc::new(ManualClock::new(
        OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp"),
    ))
}

/// Makes a real account for the caller and returns its id and a session cookie value.
async fn signed_in(pool: &PgPool, clock: &Arc<ManualClock>) -> (UserId, String) {
    let user = UserId::new();
    ensure_user(pool, user).await.expect("user fixture");
    support::publish_two_sum(pool).await;
    let cookie = SessionService::new(pool.clone(), clock.clone())
        .create(user, AuthMethod::MagicLink)
        .await
        .expect("create session")
        .cookie_value
        .expose()
        .to_owned();
    (user, cookie)
}

async fn read_run(address: std::net::SocketAddr, run_id: &str, cookie: &str) -> Reply {
    let cookie = format!("cs_session={cookie}");
    get(
        address,
        &format!("/api/runs/{run_id}"),
        &[("Cookie", cookie.as_str())],
    )
    .await
}

/// A fitted linear result, written the way the judge writes its `fit` column.
fn linear_fit_json() -> Value {
    let summary = FitSummary {
        fit_version: "fit-1".to_owned(),
        mode: RunMode::Submit,
        bench_status: BenchStatus::Measured,
        reference: ClassId::Linear,
        class: Some(ClassId::Linear),
        confidence: Some(Confidence::Medium),
        plausible: vec![ClassId::Linear],
        leaning: Some(ClassId::Linear),
        points: Vec::new(),
        slope: Some(1.02),
        slope_se: Some(0.04),
        exponents: Vec::new(),
        residuals: BTreeMap::new(),
        noise: None,
        range_ratio: None,
        ratio_vs_baseline: None,
        reasons: Vec::new(),
    };
    serde_json::to_value(summary).expect("a fit summary always serialises")
}

/// Inserts a completed submit for `user` with the given measured columns, and returns its id. The run
/// records one event (sequence 1), so a finished run with no later events answers `204` and is not replayed.
async fn insert_completed(
    pool: &PgPool,
    user: UserId,
    fit: Option<Value>,
    mood: Option<i16>,
    reason: Option<&str>,
) -> Uuid {
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status,
                                  verdict, finished_at, measured_complexity, complexity_confidence,
                                  fit, familiar_mood, familiar_reason, last_event_seq)
         VALUES (gen_random_uuid(), $1, 'two-sum', 'python', 'submit', 'print(1)', $2, 'completed',
                 'accepted', now(), $3, $4, $5, $6, $7, 1)
         RETURNING id",
    )
    .bind(user.as_uuid())
    .bind(vec![9_u8; 32])
    .bind(fit.as_ref().map(|_| "n"))
    .bind(fit.as_ref().map(|_| "medium"))
    .bind(fit)
    .bind(mood)
    .bind(reason)
    .fetch_one(pool)
    .await
    .expect("a completed submission with its measured columns");
    sqlx::query("INSERT INTO jobs (submission_id, state) VALUES ($1, 'succeeded')")
        .bind(id)
        .execute(pool)
        .await
        .expect("a job");
    id
}

fn body_of(reply: &Reply) -> Value {
    serde_json::from_str(&reply.body).expect("json body")
}

// Breaks if a reload shows a different result from the live event, or drops the familiar section.
#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_read_model_returns_the_stored_familiar_and_complexity_sections(pool: PgPool) {
    let fixture = Fixture::new();
    let clock = clock();
    let address = serve(counterspell_server::build_router(state(
        &pool,
        &fixture.dir,
        clock.clone(),
    )))
    .await;
    let (user, cookie) = signed_in(&pool, &clock).await;
    let id = insert_completed(
        &pool,
        user,
        Some(linear_fit_json()),
        Some(5),
        Some("matches_reference"),
    )
    .await;

    let reply = read_run(address, &id.to_string(), &cookie).await;

    assert_eq!(reply.status, 200);
    let body = body_of(&reply);
    assert_eq!(body["familiar"]["mood"], 5);
    assert_eq!(body["familiar"]["reason"], "matches_reference");
    assert_eq!(
        body["familiar"]["headline"],
        "Matches reference complexity: looks linear, medium confidence."
    );
    assert_eq!(body["complexity"]["class"], "n");
    assert_eq!(body["complexity"]["confidence"], "medium");
    assert_eq!(body["complexity"]["reference"], "n");
    assert!(
        body["familiar"].get("effect").is_none(),
        "a reload carries no effect"
    );
}

// Breaks if a run with no measurement shows an empty or invented section, which would look like a result.
#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_read_model_omits_both_sections_when_nothing_was_measured(pool: PgPool) {
    let fixture = Fixture::new();
    let clock = clock();
    let address = serve(counterspell_server::build_router(state(
        &pool,
        &fixture.dir,
        clock.clone(),
    )))
    .await;
    let (user, cookie) = signed_in(&pool, &clock).await;
    let id = insert_completed(&pool, user, None, None, None).await;

    let reply = read_run(address, &id.to_string(), &cookie).await;

    assert_eq!(reply.status, 200);
    let body = body_of(&reply);
    assert!(
        body.get("familiar").is_none(),
        "no mood is stored, so no familiar section"
    );
    assert!(
        body.get("complexity").is_none(),
        "no fit is stored, so no complexity section"
    );
}

// Breaks if the live `familiar` event and the reloaded read model disagree about the mood or the sentence,
// so a learner would see one result live and another after a reload.
#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_live_familiar_event_matches_the_reloaded_read_model(pool: PgPool) {
    let fixture = Fixture::new();
    let clock = clock();
    let address = serve(counterspell_server::build_router(state(
        &pool,
        &fixture.dir,
        clock.clone(),
    )))
    .await;
    let (user, cookie) = signed_in(&pool, &clock).await;
    let id = insert_completed(
        &pool,
        user,
        Some(linear_fit_json()),
        Some(5),
        Some("matches_reference"),
    )
    .await;
    // The event the judge writes for the same stored facts. Its effect is live-only and is not stored.
    let event = serde_json::json!({
        "mood": 5,
        "reason": "matches_reference",
        "layer": "measured",
        "effect": "laugh_burst",
        "headline": "Matches reference complexity: looks linear, medium confidence.",
        "badge": "Matches reference complexity",
        "mode": "submit"
    });
    sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload)
         VALUES ($1, 1, 1, 'familiar', $2)",
    )
    .bind(id)
    .bind(&event)
    .execute(&pool)
    .await
    .expect("a familiar event row");

    let cookie_value = format!("cs_session={cookie}");
    let mut stream = support::sse_client::SseResponse::open(
        address,
        &format!("/api/runs/{id}/events"),
        &[("Cookie", cookie_value.as_str())],
    )
    .await;
    let frame = stream
        .next_frame_within(std::time::Duration::from_secs(5))
        .await
        .expect("the stored familiar event is replayed");
    let live = frame.envelope();
    let reload = body_of(&read_run(address, &id.to_string(), &cookie).await);

    assert_eq!(live["payload"]["mood"], reload["familiar"]["mood"]);
    assert_eq!(live["payload"]["headline"], reload["familiar"]["headline"]);
    assert_eq!(live["payload"]["badge"], reload["familiar"]["badge"]);
    assert!(reload["familiar"].get("effect").is_none());
}
