//! The terminal write (design Path 5).
//!
//! A run ends in one transaction: the fence, the result columns, one row per test,
//! one terminal event and the job state. Three barriers stop a second ending: the
//! fence here, the frozen-row trigger, and the unique index on terminal events.
//! The `done` payload and the stored columns come from the same `FinalResult`.

use super::emit::{append_event_locked, read_status_locked, store_event_row};
use super::fence::lock_claim;
use super::{ClaimToken, QueueError};
use counterspell_common::events::{
    DonePayload, ErrorPayload, PublicEvent, PublicTest, SampleDetail,
};
use counterspell_common::ids::RunId;
use counterspell_common::run_state::status_after_event;
use counterspell_common::status::{FailureReason, TestVerdict, Verdict};
use sqlx::{PgPool, Postgres, Transaction};

/// One test's result, as the terminal write stores it.
///
/// A hidden test has no field that could hold its input, expected output or
/// program output, so none can be stored.
#[derive(Debug, Clone, PartialEq)]
pub enum TestRecord {
    /// A sample test. The user may see its input, expected output and what the program printed.
    Sample {
        /// Position in the suite, starting at 1.
        ordinal: u32,
        /// The judge's verdict for this test.
        verdict: TestVerdict,
        /// Time the test took, measured by the judge.
        time_ms: u32,
        /// The text shown to the user for this sample.
        detail: SampleDetail,
    },
    /// A hidden test. Only the verdict and the time are kept.
    Hidden {
        /// Position in the suite, starting at 1.
        ordinal: u32,
        /// The judge's verdict for this test.
        verdict: TestVerdict,
        /// Time the test took, measured by the judge.
        time_ms: u32,
    },
}

/// The final result of a run that the judge finished.
#[derive(Debug, Clone, PartialEq)]
pub struct FinalResult {
    /// The judge's verdict for the whole run.
    pub verdict: Verdict,
    /// Tests that passed.
    pub passed: u32,
    /// Tests in the suite.
    pub total: u32,
    /// The score shown to the user.
    pub score: u32,
    /// The slowest time the run used, in milliseconds.
    pub runtime_ms: u32,
    /// Compiler output, cut to a fixed size when the `done` payload is built.
    pub compile_output: Option<String>,
    /// One record per test.
    pub tests: Vec<TestRecord>,
}

/// Ends the run held by `token` with `result`, in one fenced transaction.
///
/// # Errors
///
/// Returns `LeaseLost` when the claim no longer matches (nothing is written),
/// `EventRejected` when the run cannot finish from its current status,
/// `InvalidResult` when a count does not fit the database, or a database error.
/// Any error rolls the whole transaction back.
pub async fn finish(
    pool: &PgPool,
    token: &ClaimToken,
    result: &FinalResult,
) -> Result<(), QueueError> {
    let done = PublicEvent::Done(DonePayload::new(
        result.verdict,
        result.passed,
        result.total,
        result.score,
        result.compile_output.as_deref(),
    ));
    let passed = to_i32(result.passed)?;
    let total = to_i32(result.total)?;
    let score = to_i32(result.score)?;
    let runtime_ms = to_i32(result.runtime_ms)?;
    let run = token.run_id();
    let attempt = token.attempts().saturating_add(1);

    let mut tx = pool.begin().await?;
    lock_claim(&mut tx, token).await?;
    let current = read_status_locked(&mut tx, run).await?;
    status_after_event(current, &done).ok_or(QueueError::EventRejected)?;

    let seq: Option<i32> = sqlx::query_scalar(
        "UPDATE submissions \
         SET last_event_seq = last_event_seq + 1, status = 'completed', verdict = $2, \
             passed = $3, total = $4, score = $5, runtime_ms = $6, finished_at = now() \
         WHERE id = $1 AND finished_at IS NULL \
         RETURNING last_event_seq",
    )
    .bind(run.as_uuid())
    .bind(result.verdict.as_str())
    .bind(passed)
    .bind(total)
    .bind(score)
    .bind(runtime_ms)
    .fetch_optional(&mut *tx)
    .await?;
    let seq = seq.ok_or(QueueError::EventRejected)?;

    for test in &result.tests {
        insert_test_row(&mut tx, run, test).await?;
    }
    store_event_row(&mut tx, run, seq, attempt, &done).await?;
    sqlx::query(
        "UPDATE jobs SET state = 'succeeded', lease_owner = NULL, lease_expires_at = NULL, \
         updated_at = now() WHERE id = $1",
    )
    .bind(token.job_id())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Ends the run with a fixed reason that carries no verdict, such as a cancellation.
///
/// Only the reasons that do not come from a finished program are accepted here:
/// `cancelled`, `problem_unavailable` and `language_unavailable`. Retry exhaustion
/// and expiry have their own paths.
///
/// # Errors
///
/// Returns `EventRejected` for any other reason or when the run cannot end from its
/// status, `LeaseLost` when the claim no longer matches, or a database error.
pub async fn finish_failed(
    pool: &PgPool,
    token: &ClaimToken,
    reason: FailureReason,
) -> Result<(), QueueError> {
    if !matches!(
        reason,
        FailureReason::Cancelled
            | FailureReason::ProblemUnavailable
            | FailureReason::LanguageUnavailable
    ) {
        return Err(QueueError::EventRejected);
    }
    let mut tx = pool.begin().await?;
    lock_claim(&mut tx, token).await?;
    let ended = PublicEvent::Error(ErrorPayload::for_reason(reason));
    append_event_locked(
        &mut tx,
        token.run_id(),
        token.attempts().saturating_add(1),
        &ended,
    )
    .await?;
    sqlx::query(
        "UPDATE jobs SET state = 'failed', lease_owner = NULL, lease_expires_at = NULL, \
         updated_at = now() WHERE id = $1",
    )
    .bind(token.job_id())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Stores one test row. Samples keep the same public text the user sees on the stream.
/// Hidden tests store the verdict and time only.
async fn insert_test_row(
    tx: &mut Transaction<'_, Postgres>,
    run: RunId,
    test: &TestRecord,
) -> Result<(), QueueError> {
    let (ordinal, verdict, time_ms, is_sample, detail) = match test {
        TestRecord::Sample {
            ordinal,
            verdict,
            time_ms,
            detail,
        } => {
            let public = PublicEvent::Test(PublicTest::Sample {
                ordinal: *ordinal,
                verdict: *verdict,
                time_ms: *time_ms,
                detail: detail.clone(),
            });
            (*ordinal, *verdict, *time_ms, true, Some(public.payload()))
        }
        TestRecord::Hidden {
            ordinal,
            verdict,
            time_ms,
        } => (*ordinal, *verdict, *time_ms, false, None),
    };
    sqlx::query(
        "INSERT INTO test_results (submission_id, ordinal, is_sample, verdict, time_ms, public_detail) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(run.as_uuid())
    .bind(to_i32(ordinal)?)
    .bind(is_sample)
    .bind(verdict.as_str())
    .bind(to_i32(time_ms)?)
    .bind(detail)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Converts a count to the database's `int`, refusing values that do not fit.
fn to_i32(value: u32) -> Result<i32, QueueError> {
    i32::try_from(value).map_err(|_| QueueError::InvalidResult)
}
