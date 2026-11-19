//! Baseline storage for the Efficiency Familiar (efficiency-familiar T21).
//!
//! This module holds every SQL statement for `problem_baselines`. Rows are insert-only:
//! a second measurement for the same version, language, digest and size is ignored, so a
//! baseline can change only through an admin delete.

use sqlx::PgPool;
use thiserror::Error;

/// Why a baseline call failed.
#[derive(Debug, Error)]
pub enum StoreError {
    /// The database refused the statement or the connection failed.
    #[error("the baseline database call failed: {0}")]
    Database(#[from] sqlx::Error),
    /// A size or time does not fit the stored integer type.
    #[error("a baseline value is out of range")]
    OutOfRange,
}

/// One raw bench sample for `run_timings`: a single timed call at one size.
/// `repeat` counts the timed repeats at that size, starting at 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimingRow {
    /// The input size that was timed.
    pub n: u64,
    /// The repeat index at that size, starting at 1.
    pub repeat: u32,
    /// The measured call time in nanoseconds.
    pub ns: u64,
}

/// Inserts raw bench samples for one attempt, fenced by `attempt` so a retry can
/// never collide with another attempt's rows. A duplicate inside one attempt is a
/// primary-key error, which is honest: the judge cannot delete, so it must not double-write.
///
/// # Errors
///
/// Returns `OutOfRange` when a value does not fit the column, or `Database` on a database
/// error. Any error rolls the whole batch back.
pub async fn insert_run_timings(
    pool: &PgPool,
    submission_id: uuid::Uuid,
    attempt: u16,
    rows: &[TimingRow],
) -> Result<u64, StoreError> {
    let attempt = i16::try_from(attempt).map_err(|_| StoreError::OutOfRange)?;
    let mut tx = pool.begin().await?;
    let mut inserted = 0_u64;
    for row in rows {
        let n = i32::try_from(row.n).map_err(|_| StoreError::OutOfRange)?;
        let repeat = i16::try_from(row.repeat).map_err(|_| StoreError::OutOfRange)?;
        let ns = i64::try_from(row.ns).map_err(|_| StoreError::OutOfRange)?;
        let result = sqlx::query(
            "INSERT INTO run_timings (submission_id, attempt, n, repeat, ns) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(submission_id)
        .bind(attempt)
        .bind(n)
        .bind(repeat)
        .bind(ns)
        .execute(&mut *tx)
        .await?;
        inserted += result.rows_affected();
    }
    tx.commit().await?;
    Ok(inserted)
}

/// One reference timing for a problem version, language and runtime image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineRow {
    /// The problem the timing belongs to.
    pub problem_slug: String,
    /// The problem version. A new version makes old timings stale.
    pub problem_version: u32,
    /// The language the reference ran in.
    pub lang: String,
    /// The input size.
    pub n: u64,
    /// The median time in nanoseconds.
    pub median_ns: u64,
    /// The image digest the program ran in.
    pub image_digest: String,
    /// The engine version that made the measurement.
    pub engine_version: String,
    /// The fit version used to classify it.
    pub fit_version: String,
}

/// Inserts baseline rows and returns how many were new. Existing rows are left unchanged.
///
/// # Errors
///
/// Returns `OutOfRange` when a value does not fit the column, or `Database` on a database error.
/// Any error rolls the whole batch back.
pub async fn insert_baselines(pool: &PgPool, rows: &[BaselineRow]) -> Result<u64, StoreError> {
    let mut tx = pool.begin().await?;
    let mut inserted = 0_u64;
    for row in rows {
        let version = i32::try_from(row.problem_version).map_err(|_| StoreError::OutOfRange)?;
        let n = i32::try_from(row.n).map_err(|_| StoreError::OutOfRange)?;
        let median = i64::try_from(row.median_ns).map_err(|_| StoreError::OutOfRange)?;
        let result = sqlx::query(
            "INSERT INTO problem_baselines \
                 (problem_slug, problem_version, lang, n, median_ns, image_digest, \
                  engine_version, fit_version) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT DO NOTHING",
        )
        .bind(&row.problem_slug)
        .bind(version)
        .bind(&row.lang)
        .bind(n)
        .bind(median)
        .bind(&row.image_digest)
        .bind(&row.engine_version)
        .bind(&row.fit_version)
        .execute(&mut *tx)
        .await?;
        inserted += result.rows_affected();
    }
    tx.commit().await?;
    Ok(inserted)
}

/// Returns the baseline points for one problem version, language and image digest, sorted by
/// size. A different version or digest finds nothing.
///
/// # Errors
///
/// Returns `OutOfRange` when a stored value cannot be read as an unsigned number, or
/// `Database` on a database error.
pub async fn find_baseline(
    pool: &PgPool,
    problem_slug: &str,
    problem_version: u32,
    lang: &str,
    image_digest: &str,
) -> Result<Vec<(u64, u64)>, StoreError> {
    let version = i32::try_from(problem_version).map_err(|_| StoreError::OutOfRange)?;
    let rows: Vec<(i32, i64)> = sqlx::query_as(
        "SELECT n, median_ns FROM problem_baselines \
         WHERE problem_slug = $1 AND problem_version = $2 AND lang = $3 AND image_digest = $4 \
         ORDER BY n",
    )
    .bind(problem_slug)
    .bind(version)
    .bind(lang)
    .bind(image_digest)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|(n, median)| {
            Ok((
                u64::try_from(n).map_err(|_| StoreError::OutOfRange)?,
                u64::try_from(median).map_err(|_| StoreError::OutOfRange)?,
            ))
        })
        .collect()
}
