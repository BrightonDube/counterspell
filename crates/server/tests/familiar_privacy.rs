//! Privacy of the Familiar's speech bubble (efficiency-familiar T43, requirements R11.4 and R12).
//!
//! The published fixture holds canaries in hidden inputs, hidden expected outputs, the reference
//! solution, a known ghost and the problem statement. None of them may reach the prompt sent to a
//! provider, the bubble, or the stored hint. A scripted transport records the prompts, so no test
//! reaches an AI provider.
//!
//! Run with `DATABASE_URL` and `VALKEY_URL` set:
//! `cargo test -p counterspell-server --test familiar_privacy -- --ignored pg_`.

mod support;

use std::collections::HashMap;

use counterspell_common::ids::UserId;
use counterspell_judge::catalogue::testkit::{
    CANARY_EXPECTED, CANARY_GHOST, CANARY_HIDDEN, CANARY_REFERENCE, CANARY_STATEMENT,
};
use counterspell_server::{
    familiar::{
        attempt::AttemptMarker,
        speech::{speak, Source, Speech},
        store::load_run_facts,
    },
    hints::{
        ai::Deps,
        config::HintConfig,
        provider::{ChatResponse, Usage},
        quota::QuotaStore,
    },
    testkit::{ensure_user, ScriptedTransport, Step},
};
use sqlx::{types::Uuid, PgPool};
use time::OffsetDateTime;

/// Written into the learner's own code, to prove the prompt really carries the code.
const CODE_MARKER: &str = "learner_marker_ok";

/// Inserts a completed Submit with a stored measured mood and returns its owner and id.
async fn measured_run(pool: &PgPool) -> (UserId, Uuid) {
    let user = UserId::new();
    ensure_user(pool, user).await.expect("user fixture");
    support::publish_two_sum(pool).await;
    let fit = serde_json::json!({
        "fit_version": "fit-1", "mode": "submit", "bench_status": "measured",
        "reference": "n", "class": "n^2", "confidence": "medium",
        "plausible": ["n^2"], "leaning": "n^2", "points": [], "slope": 2.0,
        "slope_se": 0.05, "exponents": [], "residuals": {}, "noise": null,
        "range_ratio": null, "ratio_vs_baseline": null, "reasons": []
    });
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
    .bind(format!(
        "def two_sum(nums, target):\n    # {CODE_MARKER}\n    return []\n"
    ))
    .bind(vec![9_u8; 32])
    .bind(fit)
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

fn hint_config() -> HintConfig {
    let vars = HashMap::from([
        ("OPENAI_API_KEY".to_owned(), "sk-test-openai".to_owned()),
        ("GROQ_API_KEY".to_owned(), "gsk-test-groq".to_owned()),
        ("AI_DAILY_LIMIT".to_owned(), "10".to_owned()),
    ]);
    HintConfig::from_map(&vars).expect("the fake test configuration is valid")
}

// R11.4, R12: breaks if a prompt, a bubble or a stored hint can carry hidden or reference data.
#[ignore = "needs DATABASE_URL and VALKEY_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_the_speech_prompt_bubble_and_stored_hint_carry_no_hidden_data(pool: PgPool) {
    let (user, run) = measured_run(&pool).await;
    let facts = load_run_facts(&pool, run, user.as_uuid())
        .await
        .expect("query")
        .expect("the owner finds the run");
    let transport = ScriptedTransport::new(vec![Step::Reply(Ok(ChatResponse {
        text: "Your nested scan looks quadratic, and the reference is linear.".to_owned(),
        usage: Some(Usage {
            prompt_tokens: 100,
            completion_tokens: 20,
        }),
    }))]);
    let config = hint_config();
    let valkey = std::env::var("VALKEY_URL").expect("VALKEY_URL is set for the ignored tests");
    let quota = QuotaStore::open(&valkey, config.daily_limit, i64::MAX / 4).expect("url");
    let marker = AttemptMarker::open(&valkey).expect("url");
    let ai = Deps {
        config: &config,
        quota: &quota,
        transport: &transport,
    };
    let speech = Speech {
        pool: &pool,
        ai: Some(&ai),
        marker: &marker,
    };

    let bubble = speak(&speech, user, &facts, false, OffsetDateTime::now_utc())
        .await
        .expect("a bubble");

    assert_eq!(bubble.source, Source::Ai, "the provider path ran");
    let prompts = transport.prompts();
    assert_eq!(prompts.len(), 1, "one provider call, one prompt");
    // Control: the prompt really carries the learner's code and the measured class, so the
    // absence checks below are not vacuous.
    assert!(
        prompts[0].contains(CODE_MARKER),
        "the prompt holds the code"
    );
    let stored: Vec<(String,)> = sqlx::query_as("SELECT body FROM hints WHERE submission_id = $1")
        .bind(run)
        .fetch_all(&pool)
        .await
        .expect("stored hints");
    assert_eq!(stored.len(), 1, "the bubble is stored once");
    for canary in [
        CANARY_HIDDEN,
        CANARY_EXPECTED,
        CANARY_REFERENCE,
        CANARY_GHOST,
        CANARY_STATEMENT,
    ] {
        assert!(!prompts[0].contains(canary), "{canary} reached the prompt");
        assert!(!bubble.body.contains(canary), "{canary} reached the bubble");
        assert!(
            !stored[0].0.contains(canary),
            "{canary} reached the stored hint"
        );
        assert!(
            !format!("{facts:?}").contains(canary),
            "{canary} reached the run facts"
        );
    }
}
