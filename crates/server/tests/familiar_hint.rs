//! `POST /api/runs/{id}/familiar-hint` over real Postgres and Valkey (efficiency-familiar T38,
//! requirements R11.1, R11.2, R11.8, R11.11 and R12.1, property P21).
//!
//! The route's transport is the real HTTP client, so these tests never reach a provider: they cover the
//! refusals, the template shapes and the stored AI bubble. Generation, fallback and the claim guard are
//! covered in `familiar_speech.rs` with a scripted transport.
//!
//! Run with `DATABASE_URL` and `VALKEY_URL` set:
//! `cargo test -p counterspell-server --test familiar_hint -- --ignored pg_`.

mod support;

use std::{collections::BTreeMap, collections::HashMap, sync::Arc};

use counterspell_common::{
    familiar::{
        class::{ClassId, Confidence},
        payload::{BenchStatus, FitSummary},
    },
    ids::UserId,
    status::RunMode,
};
use counterspell_server::{
    build_router,
    familiar::attempt::AttemptMarker,
    hints::{
        config::HintConfig, handlers::HintState, provider::ReqwestTransport, quota::QuotaStore,
    },
    ratelimit::{Limiter, MemoryLimiter},
    testkit::{ensure_user, BearerTestResolver},
    AppState,
};
use serde_json::Value;
use sqlx::{types::Uuid, PgPool};
use support::{config_for, post, publish_two_sum, serve, Fixture};

fn valkey_url() -> String {
    std::env::var("VALKEY_URL").expect("VALKEY_URL is set for the ignored tests")
}

fn auth(number: u128) -> &'static str {
    Box::leak(format!("Bearer test-user-{number}").into_boxed_str())
}

/// A hint state with fake keys and the given daily limit. A limit of zero refuses every generation
/// before the transport is used, so no test reaches a provider.
fn hint_state(pool: &PgPool, daily_limit: &str) -> HintState {
    let vars = HashMap::from([
        ("OPENAI_API_KEY".to_owned(), "sk-test-openai".to_owned()),
        ("GROQ_API_KEY".to_owned(), "gsk-test-groq".to_owned()),
        ("AI_DAILY_LIMIT".to_owned(), daily_limit.to_owned()),
    ]);
    let config = HintConfig::from_map(&vars).expect("the fake configuration is valid");
    let quota = QuotaStore::open(&valkey_url(), config.daily_limit, config.spend_cap_micros)
        .expect("a valid URL");
    HintState {
        pool: pool.clone(),
        config,
        quota: Some(quota),
        transport: ReqwestTransport::new(),
    }
}

fn app(pool: &PgPool, dir: &std::path::Path, hints: HintState) -> AppState {
    AppState::new(
        pool.clone(),
        config_for(dir),
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
    )
    .with_user_resolver(BearerTestResolver)
    .with_hints(hints)
    .with_familiar_attempt(AttemptMarker::open(&valkey_url()).expect("a valid URL"))
}

fn quadratic_fit_json() -> Value {
    let summary = FitSummary {
        fit_version: "fit-1".to_owned(),
        mode: RunMode::Submit,
        bench_status: BenchStatus::Measured,
        reference: ClassId::Linear,
        class: Some(ClassId::Quadratic),
        confidence: Some(Confidence::Medium),
        plausible: vec![ClassId::Quadratic],
        leaning: Some(ClassId::Quadratic),
        points: Vec::new(),
        slope: Some(2.0),
        slope_se: Some(0.05),
        exponents: Vec::new(),
        residuals: BTreeMap::new(),
        noise: None,
        range_ratio: None,
        ratio_vs_baseline: None,
        reasons: Vec::new(),
    };
    serde_json::to_value(summary).expect("a fit summary always serialises")
}

/// A run for `user`. `finished` and `mood` control which refusal it triggers.
async fn insert_run(pool: &PgPool, user: u128, finished: bool, mood: bool) -> Uuid {
    ensure_user(pool, UserId::from_uuid(Uuid::from_u128(user)))
        .await
        .expect("user fixture");
    publish_two_sum(pool).await;
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status,
                                  verdict, passed, total, finished_at, measured_complexity,
                                  complexity_confidence, fit, familiar_mood, familiar_reason,
                                  last_event_seq)
         VALUES (gen_random_uuid(), $1, 'two-sum', 'python', 'submit', 'print(1)', $2, $3,
                 $4, 12, 12, $5, $6, $7, $8, $9, $10, 1)
         RETURNING id",
    )
    .bind(Uuid::from_u128(user))
    .bind(vec![9_u8; 32])
    .bind(if finished { "completed" } else { "running" })
    .bind(if finished { Some("accepted") } else { None })
    .bind(if finished {
        Some(time::OffsetDateTime::now_utc())
    } else {
        None
    })
    .bind(if mood { Some("n^2") } else { None })
    .bind(if mood { Some("medium") } else { None })
    .bind(if mood {
        Some(quadratic_fit_json())
    } else {
        None
    })
    .bind(if mood { Some(2_i16) } else { None })
    .bind(if mood {
        Some("two_classes_worse")
    } else {
        None
    })
    .fetch_one(pool)
    .await
    .expect("a submission");
    sqlx::query("INSERT INTO jobs (submission_id, state) VALUES ($1, $2)")
        .bind(id)
        .bind(if finished { "succeeded" } else { "queued" })
        .execute(pool)
        .await
        .expect("a job");
    id
}

fn path(run: impl std::fmt::Display) -> String {
    format!("/api/runs/{run}/familiar-hint")
}

async fn call(
    address: std::net::SocketAddr,
    run: impl std::fmt::Display,
    bearer: &'static str,
) -> support::Reply {
    post(
        address,
        &path(run),
        &[
            ("authorization", bearer),
            ("content-type", "application/json"),
        ],
        "{}",
    )
    .await
}

fn json_of(reply: &support::Reply) -> Value {
    serde_json::from_str(&reply.body).expect("a JSON body")
}

// P21: a stranger, an unknown id and a malformed id are indistinguishable.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_stranger_an_unknown_and_a_malformed_id_get_identical_404s(pool: PgPool) {
    let fixture = Fixture::new();
    let run = insert_run(&pool, 7101, true, true).await;
    let address = serve(build_router(app(
        &pool,
        &fixture.dir,
        hint_state(&pool, "10"),
    )))
    .await;

    let stranger = call(address, run, auth(7102)).await;
    let unknown = call(address, Uuid::new_v4(), auth(7101)).await;
    let malformed = call(address, "not-a-run-id", auth(7101)).await;

    assert_eq!(stranger.status, 404);
    assert_eq!(unknown.status, 404);
    assert_eq!(malformed.status, 404);
    assert_eq!(stranger.body, unknown.body);
    assert_eq!(unknown.body, malformed.body);
}

// Breaks if a missing session is answered with anything but 401 unauthenticated.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_no_session_is_401_unauthenticated(pool: PgPool) {
    let fixture = Fixture::new();
    let run = insert_run(&pool, 7111, true, true).await;
    let address = serve(build_router(app(
        &pool,
        &fixture.dir,
        hint_state(&pool, "10"),
    )))
    .await;

    let reply = post(
        address,
        &path(run),
        &[("content-type", "application/json")],
        "{}",
    )
    .await;

    assert_eq!(reply.status, 401);
    assert_eq!(json_of(&reply)["code"], "unauthenticated");
}

// Breaks if a bubble is made for a run that has not finished.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_an_unfinished_run_of_the_caller_is_409(pool: PgPool) {
    let fixture = Fixture::new();
    let run = insert_run(&pool, 7121, false, false).await;
    let address = serve(build_router(app(
        &pool,
        &fixture.dir,
        hint_state(&pool, "10"),
    )))
    .await;

    let reply = call(address, run, auth(7121)).await;

    assert_eq!(reply.status, 409);
    assert_eq!(json_of(&reply)["code"], "run_not_finished");
}

// Breaks if a finished run with no mood (cancelled, internal error) gets a bubble about nothing.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_finished_run_without_a_mood_is_422(pool: PgPool) {
    let fixture = Fixture::new();
    let run = insert_run(&pool, 7131, true, false).await;
    let address = serve(build_router(app(
        &pool,
        &fixture.dir,
        hint_state(&pool, "10"),
    )))
    .await;

    let reply = call(address, run, auth(7131)).await;

    assert_eq!(reply.status, 422);
    assert_eq!(json_of(&reply)["code"], "familiar_not_applicable");
}

// Breaks if the template shape changes, a mood leaks into the response, or a template is labelled AI.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_the_cap_reached_returns_an_unlabelled_template_with_no_mood(pool: PgPool) {
    let fixture = Fixture::new();
    let run = insert_run(&pool, 7141, true, true).await;
    let address = serve(build_router(app(
        &pool,
        &fixture.dir,
        hint_state(&pool, "0"),
    )))
    .await;

    let reply = call(address, run, auth(7141)).await;

    assert_eq!(reply.status, 200);
    let body = json_of(&reply);
    assert_eq!(body["run_id"], run.to_string());
    assert_eq!(body["source"], "template");
    assert_eq!(body["label"], Value::Null);
    assert_eq!(body["reason"], "ai_cap_reached");
    assert_eq!(body["sent_to_provider"], false);
    assert_eq!(
        body["body"],
        "Looks quadratic, medium confidence. The reference is linear."
    );
    assert_eq!(body["quota"]["limit"], 0);
    assert!(body.get("mood").is_none(), "the response has no mood field");
    assert!(!reply.body.contains("mood"), "no mood text anywhere");
}

// Breaks if a stored AI bubble loses its label or provider when read back.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_stored_ai_bubble_is_labelled_ai_hint_and_costs_nothing(pool: PgPool) {
    let fixture = Fixture::new();
    let run = insert_run(&pool, 7151, true, true).await;
    sqlx::query(
        "INSERT INTO hints (user_id, submission_id, rung, source, provider, model, body, cache_key)
         VALUES ($1, $2, NULL, 'familiar', 'openai', 'gpt-6-luna', 'The nested scan looks quadratic.', $3)",
    )
    .bind(Uuid::from_u128(7151))
    .bind(run)
    .bind(vec![1_u8; 32])
    .execute(&pool)
    .await
    .expect("a stored bubble");
    // A zero limit would refuse a new generation, so only the stored row can answer.
    let address = serve(build_router(app(
        &pool,
        &fixture.dir,
        hint_state(&pool, "0"),
    )))
    .await;

    let reply = call(address, run, auth(7151)).await;

    assert_eq!(reply.status, 200);
    let body = json_of(&reply);
    assert_eq!(body["source"], "ai");
    assert_eq!(body["label"], "AI hint");
    assert_eq!(body["provider"], "openai");
    assert_eq!(body["model"], "gpt-6-luna");
    assert_eq!(body["body"], "The nested scan looks quadratic.");
    assert_eq!(body["sent_to_provider"], true);
    assert_eq!(body["reason"], Value::Null);
}

// Breaks if speech "off" in the learner's preferences still sends their code anywhere.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_speech_off_in_the_preferences_returns_the_template(pool: PgPool) {
    let fixture = Fixture::new();
    let run = insert_run(&pool, 7161, true, true).await;
    sqlx::query(
        "UPDATE users SET familiar_prefs = familiar_prefs || '{\"speech\":\"off\"}'::jsonb WHERE id = $1",
    )
    .bind(Uuid::from_u128(7161))
    .execute(&pool)
    .await
    .expect("prefs");
    let address = serve(build_router(app(
        &pool,
        &fixture.dir,
        hint_state(&pool, "10"),
    )))
    .await;

    let reply = call(address, run, auth(7161)).await;

    assert_eq!(reply.status, 200);
    let body = json_of(&reply);
    assert_eq!(body["source"], "template");
    assert_eq!(body["reason"], "speech_off");
    assert_eq!(body["sent_to_provider"], false);
}

// Breaks if AI turned off at the server (no providers) is reported as a provider failure.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_ai_off_at_the_server_returns_the_ai_disabled_template(pool: PgPool) {
    let fixture = Fixture::new();
    let run = insert_run(&pool, 7171, true, true).await;
    let address = serve(build_router(app(
        &pool,
        &fixture.dir,
        HintState::disabled(pool.clone()),
    )))
    .await;

    let reply = call(address, run, auth(7171)).await;

    assert_eq!(reply.status, 200);
    assert_eq!(json_of(&reply)["reason"], "ai_disabled");
}
