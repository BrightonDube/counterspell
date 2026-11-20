//! Postgres reads and writes for the speech bubble. This is the only module in `familiar` that writes
//! SQL for it.
//!
//! Every read filters on the owner, so a stranger's run, an unknown id and a malformed one all return
//! "nothing". The only write is one `hints` row with `source = 'familiar'`. Nothing here can touch the
//! mood columns: the server role has no write grant on them, and no statement here names them in a
//! write.

use counterspell_common::status::RunMode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::hints::cost::micros_to_sql_decimal;

/// What the speech bubble needs to know about one finished run the caller owns.
#[derive(Debug, Clone, PartialEq)]
pub struct RunFacts {
    /// The run id.
    pub run: Uuid,
    /// The submitted code, bounded by the intake limit.
    pub code: String,
    /// Run or Submit.
    pub mode: RunMode,
    /// The final verdict code, such as `wrong_answer`.
    pub verdict: Option<String>,
    /// Tests passed, when the run counted them.
    pub passed: Option<i32>,
    /// Tests counted, when the run counted them.
    pub total: Option<i32>,
    /// The stored mood from 1 to 5, when the run has one.
    pub mood: Option<i16>,
    /// The stored mood reason code.
    pub reason: Option<String>,
    /// The stored fit summary.
    pub fit: Option<serde_json::Value>,
    /// `true` once the run has a terminal result.
    pub finished: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct RunFactsRow {
    id: Uuid,
    code: String,
    mode: String,
    verdict: Option<String>,
    passed: Option<i32>,
    total: Option<i32>,
    fit: Option<serde_json::Value>,
    familiar_mood: Option<i16>,
    familiar_reason: Option<String>,
    finished: bool,
}

/// Loads a run only when `user` owns it. A run that is not theirs is `None`, the same as an unknown id.
///
/// # Errors
///
/// Returns the database error when the query fails, or when the stored mode is not a known mode.
pub async fn load_run_facts(
    pool: &PgPool,
    run: Uuid,
    user: Uuid,
) -> Result<Option<RunFacts>, sqlx::Error> {
    let row: Option<RunFactsRow> = sqlx::query_as(
        "SELECT id, code, mode, verdict, passed, total, fit, familiar_mood, familiar_reason,
                (finished_at IS NOT NULL) AS finished
           FROM submissions
          WHERE id = $1 AND user_id = $2",
    )
    .bind(run)
    .bind(user)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let mode = RunMode::parse(&row.mode)
        .ok_or_else(|| sqlx::Error::Decode("the stored run mode is unknown".into()))?;
    Ok(Some(RunFacts {
        run: row.id,
        code: row.code,
        mode,
        verdict: row.verdict,
        passed: row.passed,
        total: row.total,
        mood: row.familiar_mood,
        reason: row.familiar_reason,
        fit: row.fit,
        finished: row.finished,
    }))
}

/// A stored AI speech bubble.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredBubble {
    /// The guarded plain text.
    pub body: String,
    /// The provider that answered.
    pub provider: Option<String>,
    /// The model that answered.
    pub model: Option<String>,
}

/// The stored bubble for a run the caller owns, if one exists. It costs no quota.
///
/// # Errors
///
/// Returns the database error when the query fails.
pub async fn find_bubble(
    pool: &PgPool,
    run: Uuid,
    user: Uuid,
) -> Result<Option<StoredBubble>, sqlx::Error> {
    let row: Option<(String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT body, provider, model FROM hints
          WHERE submission_id = $1 AND user_id = $2 AND source = 'familiar'",
    )
    .bind(run)
    .bind(user)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(body, provider, model)| StoredBubble {
        body,
        provider,
        model,
    }))
}

/// A speech bubble ready to store with its cost record.
#[derive(Debug, Clone, Copy)]
pub struct NewBubble<'a> {
    /// The owner.
    pub user: Uuid,
    /// The run the bubble explains.
    pub run: Uuid,
    /// The provider that answered.
    pub provider: &'a str,
    /// The model that answered.
    pub model: &'a str,
    /// The guarded text.
    pub body: &'a str,
    /// Input tokens, reported or estimated.
    pub tokens_in: u32,
    /// Output tokens, reported or estimated.
    pub tokens_out: u32,
    /// The cost in micro-dollars.
    pub cost_micros: i64,
    /// `true` when the token counts were estimated.
    pub cost_estimated: bool,
    /// A 32-byte key for the run, required by the table's shape check.
    pub cache_key: [u8; 32],
}

/// Stores the bubble, or returns the one already stored for the run. `ON CONFLICT DO NOTHING` makes a
/// second concurrent insert keep the first text.
///
/// # Errors
///
/// Returns the database error when the transaction fails.
pub async fn insert_bubble(
    pool: &PgPool,
    new: &NewBubble<'_>,
) -> Result<StoredBubble, sqlx::Error> {
    let tokens_in = i32::try_from(new.tokens_in).unwrap_or(i32::MAX);
    let tokens_out = i32::try_from(new.tokens_out).unwrap_or(i32::MAX);
    let mut transaction = pool.begin().await?;
    sqlx::query(
        "INSERT INTO hints (user_id, submission_id, rung, source, provider, model, body,
                            tokens_in, tokens_out, cost_usd, cost_estimated, cache_key)
         VALUES ($1, $2, NULL, 'familiar', $3, $4, $5, $6, $7, $8::numeric, $9, $10)
         ON CONFLICT (submission_id) WHERE source = 'familiar' DO NOTHING",
    )
    .bind(new.user)
    .bind(new.run)
    .bind(new.provider)
    .bind(new.model)
    .bind(new.body)
    .bind(tokens_in)
    .bind(tokens_out)
    .bind(micros_to_sql_decimal(new.cost_micros))
    .bind(new.cost_estimated)
    .bind(new.cache_key.to_vec())
    .execute(&mut *transaction)
    .await?;
    let stored: (String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT body, provider, model FROM hints
          WHERE submission_id = $1 AND source = 'familiar'",
    )
    .bind(new.run)
    .fetch_one(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(StoredBubble {
        body: stored.0,
        provider: stored.1,
        model: stored.2,
    })
}
