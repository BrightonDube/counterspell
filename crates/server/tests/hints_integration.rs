//! Integration tests for the hints spec (H-test). They run against a real database, and the quota
//! test also needs Valkey. Run them with `-- --ignored`. No test reaches an AI provider: the daily
//! limit is zero, so the cap refuses the call before any transport is used. Fallback, the spend
//! cap and the prompt canary are covered by the unit tests in `hints::ai` and `hints::guard`.

mod support;

use std::{collections::HashMap, sync::Arc};

use counterspell_common::ids::UserId;
use counterspell_server::{
    build_router,
    hints::{
        config::HintConfig,
        handlers::HintState,
        provider::ReqwestTransport,
        quota::QuotaStore,
        store::{self, NewAiHint},
    },
    ratelimit::{Limiter, MemoryLimiter},
    testkit::{ensure_user, BearerTestResolver},
    AppState,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use support::{config_for, post, publish_two_sum, serve, Fixture};
use uuid::Uuid;

/// Fake provider keys and the given overrides. The keys are never sent anywhere in these tests.
fn fake_vars(overrides: &[(&str, &str)]) -> HashMap<String, String> {
    let mut vars = HashMap::from([
        ("OPENAI_API_KEY".to_owned(), "sk-test-openai".to_owned()),
        ("GROQ_API_KEY".to_owned(), "gsk-test-groq".to_owned()),
    ]);
    for (name, value) in overrides {
        vars.insert((*name).to_owned(), (*value).to_owned());
    }
    vars
}

fn app(pool: PgPool, static_dir: &std::path::Path, hints: HintState) -> AppState {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    AppState::new(pool, config_for(static_dir), limiter)
        .with_user_resolver(BearerTestResolver)
        .with_hints(hints)
}

/// The bearer header for the test identity `test-user-<number>`.
fn auth(number: u128) -> &'static str {
    Box::leak(format!("Bearer test-user-{number}").into_boxed_str())
}

/// Creates a finished submission for `user` on the published `two-sum` problem.
async fn finished_submission(pool: &PgPool, user: Uuid) -> Uuid {
    ensure_user(pool, UserId::from_uuid(user))
        .await
        .expect("user fixture");
    publish_two_sum(pool).await;
    let submission = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status, verdict, finished_at)
         VALUES ($1, $2, 'two-sum', 'python', 'run', 'print(1)', $3, 'completed', 'wrong_answer', now())",
    )
    .bind(submission)
    .bind(user)
    .bind(vec![0u8; 32])
    .execute(pool)
    .await
    .expect("a finished submission can be created");
    submission
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
async fn pg_an_ai_hint_over_the_daily_limit_is_429_and_names_the_next_rung(pool: PgPool) {
    let valkey_url = std::env::var("VALKEY_URL").expect("VALKEY_URL is set for the ignored tests");
    let fixture = Fixture::new();
    let learner = Uuid::from_u128(5101);
    let submission = finished_submission(&pool, learner).await;
    // A daily limit of zero refuses every AI call before the provider transport is used.
    let config = HintConfig::from_map(&fake_vars(&[("AI_DAILY_LIMIT", "0")]))
        .expect("the fake test configuration is valid");
    let quota = QuotaStore::open(&valkey_url, config.daily_limit, config.spend_cap_micros)
        .expect("VALKEY_URL is a valid URL");
    let hints = HintState {
        pool: pool.clone(),
        config,
        quota: Some(quota),
        transport: ReqwestTransport::new(),
    };
    let address = serve(build_router(app(pool.clone(), &fixture.dir, hints))).await;
    let headers = [
        ("authorization", auth(5101)),
        ("content-type", "application/json"),
    ];
    let reveal = json!({ "submission_id": submission, "rung": 1, "ai": false }).to_string();
    let ai = json!({ "submission_id": submission, "rung": 2, "ai": true }).to_string();

    let revealed = post(address, "/api/hints", &headers, &reveal).await;
    let refused = post(address, "/api/hints", &headers, &ai).await;

    assert_eq!(revealed.status, 200, "rung 1 is authored and free");
    assert_eq!(refused.status, 429);
    let body: Value = serde_json::from_str(&refused.body).expect("the refusal is JSON");
    assert_eq!(body["code"], "ai_daily_cap_reached");
    assert_eq!(
        body["next_rung"], 2,
        "the learner is told the next authored hint is still open"
    );
    assert!(
        !refused.body.contains("sk-test") && !refused.body.contains("gsk-test"),
        "no key appears in the refusal"
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_demo_reset_removes_the_demo_accounts_hints(pool: PgPool) {
    publish_two_sum(&pool).await;
    let demo = Uuid::new_v4();
    // Only the seeding path may create the demo account, so the test sets the same seed flag
    // that `seed_demo` sets, for this transaction only.
    let mut seeding = pool.begin().await.expect("a transaction starts");
    sqlx::query("SELECT set_config('counterspell.demo_seed', 'on', true)")
        .execute(&mut *seeding)
        .await
        .expect("the seed flag is set for this transaction");
    sqlx::query(
        "INSERT INTO users (id, email, email_verified_at, handle, display_name, is_demo)
         VALUES ($1, $2, now(), $3, 'Demo', true)",
    )
    .bind(demo)
    .bind(format!("{demo}@example.co"))
    .bind(demo.simple().to_string())
    .execute(&mut *seeding)
    .await
    .expect("the demo user can be created while seeding");
    seeding.commit().await.expect("the demo user is committed");
    let submission = finished_submission(&pool, demo).await;
    store::insert_ai(
        &pool,
        &NewAiHint {
            user: demo,
            submission,
            rung: 2,
            provider: "openai",
            model: "gpt-6-luna",
            body: "A demo nudge.",
            tokens_in: 100,
            tokens_out: 20,
            cost_micros: 10,
            cost_estimated: false,
            cache_key: [7u8; 32],
        },
    )
    .await
    .expect("the demo hint is stored");
    let before: (i64,) = sqlx::query_as("SELECT count(*) FROM hints WHERE user_id = $1")
        .bind(demo)
        .fetch_one(&pool)
        .await
        .expect("the count runs");

    sqlx::query("SELECT cs_reset_demo_account()")
        .execute(&pool)
        .await
        .expect("the demo reset runs");
    let after: (i64,) = sqlx::query_as("SELECT count(*) FROM hints WHERE user_id = $1")
        .bind(demo)
        .fetch_one(&pool)
        .await
        .expect("the count runs");

    assert_eq!(before.0, 1, "the demo hint exists before the reset");
    assert_eq!(after.0, 0, "the reset removes the demo hint");
}
