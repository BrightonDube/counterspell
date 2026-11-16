//! Postgres reads and writes for hints (hints spec H11). This is the only module that writes SQL
//! for hints.
//!
//! Every read of a learner's submission filters on the owner and on `finished_at`, so a stranger,
//! an unknown id and an unfinished run all return the same "nothing". A reveal is one transaction:
//! the insert and the read-back happen together, and `ON CONFLICT DO NOTHING` makes a second
//! concurrent reveal return the first stored text.

use std::collections::BTreeSet;

use sqlx::PgPool;
use uuid::Uuid;

use crate::hints::cost::micros_to_sql_decimal;

/// A finished submission the learner owns, with the fields a hint needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submission {
    /// The submission id.
    pub id: Uuid,
    /// The learner who made it.
    pub user_id: Uuid,
    /// The problem it was made for.
    pub problem_slug: String,
    /// The language name, for example `python`.
    pub lang: String,
    /// The learner's source code.
    pub code: String,
}

/// A hint text that is stored and returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredText {
    /// The stored hint body.
    pub body: String,
}

/// A stored AI hint, with the label fields the client shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiHint {
    /// The hint body.
    pub body: String,
    /// The provider that wrote it.
    pub provider: Option<String>,
    /// The model that wrote it.
    pub model: Option<String>,
}

/// One AI generation to store, with its cost record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAiHint<'a> {
    /// The learner who asked.
    pub user: Uuid,
    /// The submission the hint is about.
    pub submission: Uuid,
    /// The rung, from 2 to 4.
    pub rung: u8,
    /// The provider name.
    pub provider: &'a str,
    /// The model name.
    pub model: &'a str,
    /// The guarded hint text.
    pub body: &'a str,
    /// Input tokens counted or estimated for the call.
    pub tokens_in: u32,
    /// Output tokens counted or estimated for the call.
    pub tokens_out: u32,
    /// The cost in micro-dollars.
    pub cost_micros: i64,
    /// `true` when the token counts were estimated, not reported.
    pub cost_estimated: bool,
    /// The cache key, 32 bytes.
    pub cache_key: [u8; 32],
}

/// The failing input for a submission, as JSON text. Only the ghost columns are read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhostRow {
    /// The failing input.
    pub input: String,
    /// The expected output.
    pub expected: String,
    /// What the learner's code returned.
    pub got: String,
    /// The failure kind: `wrong`, `crash` or `timeout`.
    pub kind: String,
}

/// Loads a submission the learner owns and has finished. Any other case returns `None`.
///
/// # Errors
///
/// Returns the database error when the query fails.
pub async fn load_submission(
    pool: &PgPool,
    user: Uuid,
    submission: Uuid,
) -> Result<Option<Submission>, sqlx::Error> {
    let row: Option<(Uuid, Uuid, String, String, String)> = sqlx::query_as(
        "SELECT id, user_id, problem_slug, lang, code FROM submissions
         WHERE id = $1 AND user_id = $2 AND finished_at IS NOT NULL",
    )
    .bind(submission)
    .bind(user)
    .fetch_optional(pool)
    .await?;
    Ok(
        row.map(|(id, user_id, problem_slug, lang, code)| Submission {
            id,
            user_id,
            problem_slug,
            lang,
            code,
        }),
    )
}

/// The ladder rungs already revealed for a submission.
///
/// # Errors
///
/// Returns the database error when the query fails.
pub async fn revealed_rungs(pool: &PgPool, submission: Uuid) -> Result<BTreeSet<u8>, sqlx::Error> {
    let rungs: Vec<i16> = sqlx::query_scalar(
        "SELECT rung FROM hints WHERE submission_id = $1 AND source = 'ladder' AND rung IS NOT NULL",
    )
    .bind(submission)
    .fetch_all(pool)
    .await?;
    Ok(rungs
        .into_iter()
        .filter_map(|rung| u8::try_from(rung).ok())
        .collect())
}

/// The authored text for a problem's rung in its latest version. `None` when the problem has no
/// text for that rung.
///
/// # Errors
///
/// Returns the database error when the query fails.
pub async fn authored_body(
    pool: &PgPool,
    problem_slug: &str,
    rung: u8,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT body FROM problem_hints WHERE problem_slug = $1 AND rung = $2
         ORDER BY version DESC LIMIT 1",
    )
    .bind(problem_slug)
    .bind(i32::from(rung))
    .fetch_optional(pool)
    .await
}

/// Stores the ladder reveal for a rung, or returns the text already stored for it.
///
/// The insert and the read-back run in one transaction. If another reveal of the same rung wins
/// the race, this returns that reveal's text and stores nothing.
///
/// # Errors
///
/// Returns the database error when the transaction fails.
pub async fn reveal_ladder(
    pool: &PgPool,
    user: Uuid,
    submission: Uuid,
    rung: u8,
    body: &str,
) -> Result<StoredText, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    sqlx::query(
        "INSERT INTO hints (user_id, submission_id, rung, source, body)
         VALUES ($1, $2, $3, 'ladder', $4)
         ON CONFLICT (submission_id, rung) WHERE source = 'ladder' DO NOTHING",
    )
    .bind(user)
    .bind(submission)
    .bind(i16::from(rung))
    .bind(body)
    .execute(&mut *transaction)
    .await?;
    let stored: String = sqlx::query_scalar(
        "SELECT body FROM hints WHERE submission_id = $1 AND rung = $2 AND source = 'ladder'",
    )
    .bind(submission)
    .bind(i16::from(rung))
    .fetch_one(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(StoredText { body: stored })
}

/// Stores an AI hint with its cost record, or returns the hint already stored for that rung.
///
/// # Errors
///
/// Returns the database error when the transaction fails.
pub async fn insert_ai(pool: &PgPool, new: &NewAiHint<'_>) -> Result<AiHint, sqlx::Error> {
    let tokens_in = i32::try_from(new.tokens_in).unwrap_or(i32::MAX);
    let tokens_out = i32::try_from(new.tokens_out).unwrap_or(i32::MAX);
    let mut transaction = pool.begin().await?;
    sqlx::query(
        "INSERT INTO hints (user_id, submission_id, rung, source, provider, model, body,
                            tokens_in, tokens_out, cost_usd, cost_estimated, cache_key)
         VALUES ($1, $2, $3, 'ai', $4, $5, $6, $7, $8, $9::numeric, $10, $11)
         ON CONFLICT (submission_id, rung) WHERE source = 'ai' DO NOTHING",
    )
    .bind(new.user)
    .bind(new.submission)
    .bind(i16::from(new.rung))
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
         WHERE submission_id = $1 AND rung = $2 AND source = 'ai'",
    )
    .bind(new.submission)
    .bind(i16::from(new.rung))
    .fetch_one(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(AiHint {
        body: stored.0,
        provider: stored.1,
        model: stored.2,
    })
}

/// The stored AI hint for a submission and rung, if one exists.
///
/// # Errors
///
/// Returns the database error when the query fails.
pub async fn find_ai(
    pool: &PgPool,
    submission: Uuid,
    rung: u8,
) -> Result<Option<AiHint>, sqlx::Error> {
    let row: Option<(String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT body, provider, model FROM hints
         WHERE submission_id = $1 AND rung = $2 AND source = 'ai'",
    )
    .bind(submission)
    .bind(i16::from(rung))
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(body, provider, model)| AiHint {
        body,
        provider,
        model,
    }))
}

/// The ghost row for a submission. It returns `None` while no `ghosts` table exists, because the
/// owner of that table is still open (see `docs/kiro-log/hints-blockers.md`).
///
/// # Errors
///
/// Returns the database error when the query fails.
pub async fn ghost_for(pool: &PgPool, submission: Uuid) -> Result<Option<GhostRow>, sqlx::Error> {
    let table_exists: bool = sqlx::query_scalar("SELECT to_regclass('public.ghosts') IS NOT NULL")
        .fetch_one(pool)
        .await?;
    if !table_exists {
        return Ok(None);
    }
    let row: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT input::text, expected::text, got::text, kind FROM ghosts WHERE submission_id = $1",
    )
    .bind(submission)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(input, expected, got, kind)| GhostRow {
        input,
        expected,
        got,
        kind,
    }))
}
