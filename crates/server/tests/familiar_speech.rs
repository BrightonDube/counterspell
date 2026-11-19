//! The speech bubble orchestration over real Postgres and Valkey (efficiency-familiar T37, requirements
//! R11.3 and R11.6 to R11.10, property P20). No test reaches an AI provider: a scripted transport gives
//! the replies and counts the calls.
//!
//! Run with `DATABASE_URL` and `VALKEY_URL` set:
//! `cargo test -p counterspell-server --test familiar_speech -- --ignored pg_`.

mod support;

use std::{collections::BTreeMap, collections::HashMap};

use counterspell_common::{
    familiar::{
        class::{ClassId, Confidence},
        payload::{BenchStatus, FitSummary},
    },
    ids::UserId,
    status::RunMode,
};
use counterspell_server::{
    familiar::{
        attempt::AttemptMarker,
        speech::{speak, Bubble, Source, Speech, TemplateReason},
        store::{load_run_facts, RunFacts},
    },
    hints::{
        ai::Deps,
        config::HintConfig,
        provider::{ChatResponse, ProviderError, Usage},
        quota::QuotaStore,
    },
    testkit::{ensure_user, ScriptedTransport, Step},
};
use sqlx::{types::Uuid, PgPool};
use time::OffsetDateTime;

/// A quadratic result against a linear reference: mood 2, "two classes worse".
fn quadratic_fit_json() -> serde_json::Value {
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

/// Inserts a completed Submit with a stored measured mood and returns its owner and id.
async fn measured_run(pool: &PgPool, code: &str) -> (UserId, Uuid) {
    let user = UserId::new();
    ensure_user(pool, user).await.expect("user fixture");
    support::publish_two_sum(pool).await;
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status,
                                  verdict, passed, total, finished_at, measured_complexity,
                                  complexity_confidence, fit, familiar_mood, familiar_reason,
                                  last_event_seq)
         VALUES (gen_random_uuid(), $1, 'two-sum', 'python', 'submit', $2, $3, 'completed',
                 'accepted', 12, 12, now(), 'n^2', 'medium', $4, 2, 'two_classes_worse', 1)
         RETURNING id",
    )
    .bind(user.as_uuid())
    .bind(code)
    .bind(vec![9_u8; 32])
    .bind(quadratic_fit_json())
    .fetch_one(pool)
    .await
    .expect("a measured submission");
    sqlx::query("INSERT INTO jobs (submission_id, state) VALUES ($1, 'succeeded')")
        .bind(id)
        .execute(pool)
        .await
        .expect("a job");
    (user, id)
}

fn hint_config(daily_limit: &str) -> HintConfig {
    let vars = HashMap::from([
        ("OPENAI_API_KEY".to_owned(), "sk-test-openai".to_owned()),
        ("GROQ_API_KEY".to_owned(), "gsk-test-groq".to_owned()),
        ("AI_DAILY_LIMIT".to_owned(), daily_limit.to_owned()),
    ]);
    HintConfig::from_map(&vars).expect("the fake test configuration is valid")
}

fn reply(text: &str) -> Step {
    Step::Reply(Ok(ChatResponse {
        text: text.to_owned(),
        usage: Some(Usage {
            prompt_tokens: 100,
            completion_tokens: 20,
        }),
    }))
}

fn failure() -> Step {
    Step::Reply(Err(ProviderError::Transport))
}

fn valkey_url() -> String {
    std::env::var("VALKEY_URL").expect("VALKEY_URL is set for the ignored tests")
}

/// Runs one `speak` call against the shared fixtures.
async fn speak_once(
    pool: &PgPool,
    transport: &ScriptedTransport,
    config: &HintConfig,
    marker: &AttemptMarker,
    user: UserId,
    facts: &RunFacts,
    speech_off: bool,
) -> Bubble {
    let quota = QuotaStore::open(&valkey_url(), config.daily_limit, i64::MAX / 4).expect("url");
    let ai = Deps {
        config,
        quota: &quota,
        transport,
    };
    let speech = Speech {
        pool,
        ai: Some(&ai),
        marker,
    };
    speak(&speech, user, facts, speech_off, OffsetDateTime::now_utc())
        .await
        .expect("a bubble")
}

async fn facts_for(pool: &PgPool, user: UserId, run: Uuid) -> RunFacts {
    load_run_facts(pool, run, user.as_uuid())
        .await
        .expect("query")
        .expect("the owner finds the run")
}

const LINEAR_SENTENCE: &str = "Your nested scan looks quadratic here, and the reference is linear.";

// Breaks if a stranger can read another learner's run facts.
#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_only_the_owner_can_load_the_run_facts(pool: PgPool) {
    let (owner, run) = measured_run(&pool, "print(1)").await;

    let stranger = load_run_facts(&pool, run, Uuid::new_v4())
        .await
        .expect("query");
    let own = load_run_facts(&pool, run, owner.as_uuid())
        .await
        .expect("query");

    assert_eq!(stranger, None);
    assert_eq!(own.map(|facts| facts.mood), Some(Some(2)));
}

// P20: N concurrent calls make one provider call, and no caller gets different AI text.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_concurrent_calls_make_exactly_one_provider_call(pool: PgPool) {
    let (user, run) = measured_run(&pool, "print(1)").await;
    let facts = facts_for(&pool, user, run).await;
    let transport = ScriptedTransport::new(vec![reply(LINEAR_SENTENCE)]);
    let config = hint_config("10");
    let marker = AttemptMarker::open(&valkey_url()).expect("url");

    let calls =
        (0..6).map(|_| speak_once(&pool, &transport, &config, &marker, user, &facts, false));
    let bubbles = futures_util::future::join_all(calls).await;

    assert_eq!(transport.calls(), 1);
    let ai: Vec<&Bubble> = bubbles.iter().filter(|b| b.source == Source::Ai).collect();
    assert_eq!(ai.len(), 1, "exactly one caller generated the text");
    assert_eq!(ai[0].body, LINEAR_SENTENCE);
    for bubble in bubbles.iter().filter(|b| b.source == Source::Template) {
        assert_eq!(bubble.reason, Some(TemplateReason::AlreadyAttempted));
        assert_eq!(
            bubble.body,
            "Looks quadratic, medium confidence. The reference is linear."
        );
    }
}

// Breaks if a second request after the first finished pays for another generation.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_the_stored_bubble_is_returned_without_a_provider_call_or_quota(pool: PgPool) {
    let (user, run) = measured_run(&pool, "print(1)").await;
    let facts = facts_for(&pool, user, run).await;
    let first_transport = ScriptedTransport::new(vec![reply(LINEAR_SENTENCE)]);
    let marker = AttemptMarker::open(&valkey_url()).expect("url");
    speak_once(
        &pool,
        &first_transport,
        &hint_config("10"),
        &marker,
        user,
        &facts,
        false,
    )
    .await;

    // A zero limit would refuse any new generation, so only the stored row can answer.
    let second_transport = ScriptedTransport::new(vec![]);
    let again = speak_once(
        &pool,
        &second_transport,
        &hint_config("0"),
        &marker,
        user,
        &facts,
        false,
    )
    .await;

    assert_eq!(second_transport.calls(), 0);
    assert_eq!(again.source, Source::Ai);
    assert_eq!(again.body, LINEAR_SENTENCE);
}

// Breaks if the cap reached still calls a provider, or shows an empty bubble.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_the_cap_reached_gives_the_template_and_no_provider_call(pool: PgPool) {
    let (user, run) = measured_run(&pool, "print(1)").await;
    let facts = facts_for(&pool, user, run).await;
    let transport = ScriptedTransport::new(vec![reply(LINEAR_SENTENCE)]);
    let marker = AttemptMarker::open(&valkey_url()).expect("url");

    let bubble = speak_once(
        &pool,
        &transport,
        &hint_config("0"),
        &marker,
        user,
        &facts,
        false,
    )
    .await;

    assert_eq!(bubble.reason, Some(TemplateReason::AiCapReached));
    assert_eq!(bubble.source, Source::Template);
    assert_eq!(transport.calls(), 0);
}

// Breaks if both providers failing shows nothing, or stores a row.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_both_providers_failing_gives_the_template(pool: PgPool) {
    let (user, run) = measured_run(&pool, "print(1)").await;
    let facts = facts_for(&pool, user, run).await;
    let transport = ScriptedTransport::new(vec![failure(), failure()]);
    let marker = AttemptMarker::open(&valkey_url()).expect("url");

    let bubble = speak_once(
        &pool,
        &transport,
        &hint_config("10"),
        &marker,
        user,
        &facts,
        false,
    )
    .await;

    assert_eq!(bubble.reason, Some(TemplateReason::AiUnavailable));
    assert!(bubble.sent_to_provider, "the code did go to the providers");
    let stored: i64 = sqlx::query_scalar("SELECT count(*) FROM hints WHERE source = 'familiar'")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(stored, 0);
}

// Breaks if "speech off" still sends the learner's code to a provider.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_speech_off_gives_the_template_and_sends_nothing(pool: PgPool) {
    let (user, run) = measured_run(&pool, "print(1)").await;
    let facts = facts_for(&pool, user, run).await;
    let transport = ScriptedTransport::new(vec![reply(LINEAR_SENTENCE)]);
    let marker = AttemptMarker::open(&valkey_url()).expect("url");

    let bubble = speak_once(
        &pool,
        &transport,
        &hint_config("10"),
        &marker,
        user,
        &facts,
        true,
    )
    .await;

    assert_eq!(bubble.reason, Some(TemplateReason::SpeechOff));
    assert!(!bubble.sent_to_provider);
    assert_eq!(transport.calls(), 0);
}

// Breaks if over-size code reaches a provider.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_oversized_code_gives_the_template_and_sends_nothing(pool: PgPool) {
    let code = "x".repeat(8 * 1024 + 1);
    let (user, run) = measured_run(&pool, &code).await;
    let facts = facts_for(&pool, user, run).await;
    let transport = ScriptedTransport::new(vec![reply(LINEAR_SENTENCE)]);
    let marker = AttemptMarker::open(&valkey_url()).expect("url");

    let bubble = speak_once(
        &pool,
        &transport,
        &hint_config("10"),
        &marker,
        user,
        &facts,
        false,
    )
    .await;

    assert_eq!(bubble.reason, Some(TemplateReason::CodeTooLarge));
    assert_eq!(transport.calls(), 0);
}

// Breaks if AI text that names an unmeasured class is shown or stored.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_claim_naming_an_unmeasured_class_is_replaced_by_the_template(pool: PgPool) {
    let (user, run) = measured_run(&pool, "print(1)").await;
    let facts = facts_for(&pool, user, run).await;
    let transport = ScriptedTransport::new(vec![reply("This looks cubic to me.")]);
    let marker = AttemptMarker::open(&valkey_url()).expect("url");

    let bubble = speak_once(
        &pool,
        &transport,
        &hint_config("10"),
        &marker,
        user,
        &facts,
        false,
    )
    .await;

    assert_eq!(bubble.reason, Some(TemplateReason::ClaimMismatch));
    assert_eq!(bubble.source, Source::Template);
    let stored: i64 = sqlx::query_scalar("SELECT count(*) FROM hints WHERE source = 'familiar'")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(stored, 0, "rejected text is never stored");
}

// Breaks if a run can spend a second generation after its attempt was already taken.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_an_attempt_already_taken_gives_the_template(pool: PgPool) {
    let (user, run) = measured_run(&pool, "print(1)").await;
    let facts = facts_for(&pool, user, run).await;
    let marker = AttemptMarker::open(&valkey_url()).expect("url");
    assert_eq!(marker.claim(run).await, Ok(true));
    let transport = ScriptedTransport::new(vec![reply(LINEAR_SENTENCE)]);

    let bubble = speak_once(
        &pool,
        &transport,
        &hint_config("10"),
        &marker,
        user,
        &facts,
        false,
    )
    .await;

    assert_eq!(bubble.reason, Some(TemplateReason::AlreadyAttempted));
    assert_eq!(transport.calls(), 0);
}

// Breaks if Valkey being down lets a generation through, or breaks the bubble.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_valkey_down_gives_the_template(pool: PgPool) {
    let (user, run) = measured_run(&pool, "print(1)").await;
    let facts = facts_for(&pool, user, run).await;
    let down = AttemptMarker::open("redis://127.0.0.1:1").expect("a well-formed url");
    let transport = ScriptedTransport::new(vec![reply(LINEAR_SENTENCE)]);

    let bubble = speak_once(
        &pool,
        &transport,
        &hint_config("10"),
        &down,
        user,
        &facts,
        false,
    )
    .await;

    assert_eq!(bubble.reason, Some(TemplateReason::AiUnavailable));
    assert_eq!(transport.calls(), 0);
}
