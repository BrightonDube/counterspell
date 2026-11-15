// test-change: CAT-F1. Setup only: runs now need a catalogue problem row (submissions_problem_fk). No assertion changed.
//! Database tests for the efficiency-familiar columns (efficiency-familiar T16, P16).
//! They check the CHECK constraint, the freeze trigger and the role grants from
//! migration `20261115130000_efficiency_familiar.sql`.
//!
//! Run with `cargo test -p counterspell-judge --test familiar_pg -- --ignored pg_`.

use counterspell_judge::familiar::store::{find_baseline, insert_baselines, BaselineRow};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

const CHECK_VIOLATION: &str = "23514";
const INSUFFICIENT_PRIVILEGE: &str = "42501";

/// One submission row with its familiar columns, as the database owner would insert it.
struct Row {
    status: &'static str,
    verdict: Option<&'static str>,
    finished: bool,
    mood: Option<i16>,
    reason: Option<&'static str>,
    measured: Option<&'static str>,
    confidence: Option<&'static str>,
    fit: Option<String>,
}

impl Row {
    /// A finished, accepted run with a laughing mood. Every constraint is satisfied.
    fn valid() -> Self {
        Self {
            status: "completed",
            verdict: Some("accepted"),
            finished: true,
            mood: Some(5),
            reason: Some("matches_reference"),
            measured: Some("n"),
            confidence: Some("medium"),
            fit: Some(r#"{"fit_version":"fit-1"}"#.to_owned()),
        }
    }
}

/// Inserts a `users` row as the database owner and returns its id.
async fn insert_user(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, handle, display_name) VALUES ($1, $2, 'Test user')")
        .bind(id)
        .bind(id.simple().to_string())
        .execute(pool)
        .await
        .unwrap();
    id
}

/// Inserts one submission with the given familiar columns. Returns the database error, if any.
async fn insert_row(pool: &PgPool, row: &Row) -> Result<Uuid, sqlx::Error> {
    let run = Uuid::new_v4();
    let owner = insert_user(pool).await;
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, \
             status, verdict, finished_at, familiar_mood, familiar_reason, \
             measured_complexity, complexity_confidence, fit) \
         VALUES ($1, $2, 'two-sum', 'python', 'submit', 'print(1)', decode('00', 'hex'), \
             $3, $4, CASE WHEN $5 THEN now() END, $6, $7, $8, $9, $10::jsonb)",
    )
    .bind(run)
    .bind(owner)
    .bind(row.status)
    .bind(row.verdict)
    .bind(row.finished)
    .bind(row.mood)
    .bind(row.reason)
    .bind(row.measured)
    .bind(row.confidence)
    .bind(row.fit.as_deref())
    .execute(pool)
    .await?;
    Ok(run)
}

/// Inserts a submission that is still running, as the database owner.
async fn seed_running(pool: &PgPool) -> Uuid {
    counterspell_judge::testkit::ensure_problem(pool)
        .await
        .expect("the test problem must exist");
    let owner = insert_user(pool).await;
    let run = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, \
             status, started_at) \
         VALUES ($1, $2, 'two-sum', 'python', 'submit', 'print(1)', decode('00', 'hex'), \
             'running', now())",
    )
    .bind(run)
    .bind(owner)
    .execute(pool)
    .await
    .unwrap();
    run
}

/// Returns the SQLSTATE code of a database error, or panics if the error is not from the database.
fn database_code(error: &sqlx::Error) -> String {
    error
        .as_database_error()
        .and_then(|db| db.code().map(std::borrow::Cow::into_owned))
        .unwrap_or_else(|| panic!("not a database error: {error}"))
}

/// Runs `sql` (with the run id as `$1`) and requires success.
async fn allowed(tx: &mut Transaction<'_, Postgres>, sql: &str, run: Uuid) {
    sqlx::query(sql)
        .bind(run)
        .execute(&mut **tx)
        .await
        .unwrap_or_else(|error| panic!("expected to be allowed: {sql}: {error}"));
}

/// Runs `sql` with no bind parameters and requires success.
async fn allowed_unbound(tx: &mut Transaction<'_, Postgres>, sql: &str) {
    sqlx::query(sql)
        .execute(&mut **tx)
        .await
        .unwrap_or_else(|error| panic!("expected to be allowed: {sql}: {error}"));
}

/// Runs `sql` (with the run id as `$1`) inside a savepoint and requires a privilege denial.
async fn denied(tx: &mut Transaction<'_, Postgres>, sql: &str, run: Uuid) {
    sqlx::query("SAVEPOINT probe")
        .execute(&mut **tx)
        .await
        .unwrap();
    let result = sqlx::query(sql).bind(run).execute(&mut **tx).await;
    sqlx::query("ROLLBACK TO SAVEPOINT probe")
        .execute(&mut **tx)
        .await
        .unwrap();
    match result {
        Ok(_) => panic!("expected to be denied, but it succeeded: {sql}"),
        Err(error) => assert_eq!(database_code(&error), INSUFFICIENT_PRIVILEGE, "{sql}"),
    }
}

/// Runs `sql` with no bind parameters inside a savepoint and requires a privilege denial.
async fn denied_unbound(tx: &mut Transaction<'_, Postgres>, sql: &str) {
    sqlx::query("SAVEPOINT probe")
        .execute(&mut **tx)
        .await
        .unwrap();
    let result = sqlx::query(sql).execute(&mut **tx).await;
    sqlx::query("ROLLBACK TO SAVEPOINT probe")
        .execute(&mut **tx)
        .await
        .unwrap();
    match result {
        Ok(_) => panic!("expected to be denied, but it succeeded: {sql}"),
        Err(error) => assert_eq!(database_code(&error), INSUFFICIENT_PRIVILEGE, "{sql}"),
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_familiar_constraints(pool: PgPool) {
    counterspell_judge::testkit::ensure_problem(&pool)
        .await
        .expect("the test problem must exist");
    // A valid row is accepted, and so is a furious run that failed.
    insert_row(&pool, &Row::valid()).await.unwrap();
    let furious = Row {
        status: "completed",
        verdict: Some("wrong_answer"),
        finished: true,
        mood: Some(1),
        reason: Some("wrong_answer"),
        measured: None,
        confidence: None,
        fit: None,
    };
    insert_row(&pool, &furious).await.unwrap();

    // A wrong answer can never carry a happy mood.
    let wrong_but_happy = Row {
        verdict: Some("wrong_answer"),
        mood: Some(3),
        ..Row::valid()
    };
    let error = insert_row(&pool, &wrong_but_happy).await.unwrap_err();
    assert_eq!(database_code(&error), CHECK_VIOLATION);

    // A class needs medium or high confidence.
    let low_class = Row {
        confidence: Some("low"),
        ..Row::valid()
    };
    let error = insert_row(&pool, &low_class).await.unwrap_err();
    assert_eq!(database_code(&error), CHECK_VIOLATION);

    // A class with no confidence is also rejected.
    let class_without_confidence = Row {
        confidence: None,
        ..Row::valid()
    };
    let error = insert_row(&pool, &class_without_confidence)
        .await
        .unwrap_err();
    assert_eq!(database_code(&error), CHECK_VIOLATION);

    // A mood is only allowed on a completed run.
    let queued_with_mood = Row {
        status: "queued",
        verdict: None,
        finished: false,
        ..Row::valid()
    };
    let error = insert_row(&pool, &queued_with_mood).await.unwrap_err();
    assert_eq!(database_code(&error), CHECK_VIOLATION);

    // A mood without a reason is rejected, and so is a reason without a mood.
    let mood_no_reason = Row {
        reason: None,
        ..Row::valid()
    };
    let error = insert_row(&pool, &mood_no_reason).await.unwrap_err();
    assert_eq!(database_code(&error), CHECK_VIOLATION);

    let reason_no_mood = Row {
        mood: None,
        ..Row::valid()
    };
    let error = insert_row(&pool, &reason_no_mood).await.unwrap_err();
    assert_eq!(database_code(&error), CHECK_VIOLATION);

    // A reason must be a short machine code, not text.
    let bad_reason = Row {
        reason: Some("Matches reference!"),
        ..Row::valid()
    };
    let error = insert_row(&pool, &bad_reason).await.unwrap_err();
    assert_eq!(database_code(&error), CHECK_VIOLATION);

    // The fit summary is capped at 4 KiB. Random hex does not compress, so the stored size
    // is the real size.
    let big: String = (0..600)
        .map(|_| Uuid::new_v4().simple().to_string())
        .collect();
    let oversize = Row {
        fit: Some(format!(r#"{{"blob":"{big}"}}"#)),
        ..Row::valid()
    };
    let error = insert_row(&pool, &oversize).await.unwrap_err();
    assert_eq!(database_code(&error), CHECK_VIOLATION);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_familiar_frozen(pool: PgPool) {
    counterspell_judge::testkit::ensure_problem(&pool)
        .await
        .expect("the test problem must exist");
    let run = seed_running(&pool).await;

    // The terminal write sets the verdict, the familiar columns and finished_at together.
    sqlx::query(
        "UPDATE submissions SET status = 'completed', verdict = 'accepted', finished_at = now(), \
             familiar_mood = 5, familiar_reason = 'matches_reference', \
             measured_complexity = 'n', complexity_confidence = 'medium', \
             fit = '{\"fit_version\":\"fit-1\"}'::jsonb \
         WHERE id = $1",
    )
    .bind(run)
    .execute(&pool)
    .await
    .unwrap();

    // Once finished, each of the five columns is frozen.
    let changes = [
        "familiar_mood = 4, familiar_reason = 'pleased_slower'",
        "familiar_reason = 'matches_reference_x'",
        "measured_complexity = 'n^2', complexity_confidence = 'medium'",
        "complexity_confidence = 'high'",
        "fit = '{\"fit_version\":\"fit-2\"}'::jsonb",
    ];
    for change in changes {
        let sql = format!("UPDATE submissions SET {change} WHERE id = $1");
        let error = sqlx::query(&sql)
            .bind(run)
            .execute(&pool)
            .await
            .expect_err(&sql);
        assert_eq!(database_code(&error), CHECK_VIOLATION, "{sql}");
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_familiar_roles(pool: PgPool) {
    counterspell_judge::testkit::ensure_problem(&pool)
        .await
        .expect("the test problem must exist");
    let run = seed_running(&pool).await;

    // The server may read everything and create runs, but it cannot write any familiar
    // column, a timing or a baseline.
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE counterspell_server")
        .execute(&mut *tx)
        .await
        .unwrap();
    allowed(
        &mut tx,
        "SELECT count(*) FROM submissions WHERE id = $1",
        run,
    )
    .await;
    denied(
        &mut tx,
        "UPDATE submissions SET familiar_mood = 5 WHERE id = $1",
        run,
    )
    .await;
    denied(
        &mut tx,
        "UPDATE submissions SET fit = '{}'::jsonb WHERE id = $1",
        run,
    )
    .await;
    denied(
        &mut tx,
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, \
             familiar_mood) \
         VALUES (gen_random_uuid(), $1, 'two-sum', 'python', 'run', 'x', decode('00', 'hex'), 5)",
        run,
    )
    .await;
    denied(
        &mut tx,
        "INSERT INTO run_timings (submission_id, attempt, n, repeat, ns) VALUES ($1, 1, 4, 1, 100)",
        run,
    )
    .await;
    denied_unbound(
        &mut tx,
        "INSERT INTO problem_baselines (problem_slug, problem_version, lang, n, median_ns, \
             image_digest, engine_version, fit_version) \
         VALUES ('two-sum', 1, 'python', 4, 100, 'sha256:x', 'engine-1', 'fit-1')",
    )
    .await;
    tx.rollback().await.unwrap();

    // The judge may write the five columns on finish, record timings and store baselines.
    // It cannot delete any of them.
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE counterspell_judge")
        .execute(&mut *tx)
        .await
        .unwrap();
    allowed(
        &mut tx,
        "UPDATE submissions SET status = 'completed', verdict = 'accepted', \
             finished_at = now(), familiar_mood = 5, familiar_reason = 'matches_reference', \
             measured_complexity = 'n', complexity_confidence = 'medium', \
             fit = '{\"fit_version\":\"fit-1\"}'::jsonb \
         WHERE id = $1",
        run,
    )
    .await;
    allowed(
        &mut tx,
        "INSERT INTO run_timings (submission_id, attempt, n, repeat, ns) VALUES ($1, 1, 4, 1, 100)",
        run,
    )
    .await;
    allowed_unbound(
        &mut tx,
        "INSERT INTO problem_baselines (problem_slug, problem_version, lang, n, median_ns, \
             image_digest, engine_version, fit_version) \
         VALUES ('two-sum', 1, 'python', 4, 100, 'sha256:x', 'engine-1', 'fit-1')",
    )
    .await;
    denied(
        &mut tx,
        "DELETE FROM run_timings WHERE submission_id = $1",
        run,
    )
    .await;
    denied(
        &mut tx,
        "UPDATE run_timings SET ns = 1 WHERE submission_id = $1",
        run,
    )
    .await;
    denied_unbound(&mut tx, "DELETE FROM problem_baselines").await;
    denied_unbound(&mut tx, "UPDATE problem_baselines SET median_ns = 1").await;
    tx.rollback().await.unwrap();
}

/// A baseline row for the two-sum problem in Python, at the given size and digest.
fn baseline(version: u32, n: u64, median_ns: u64, digest: &str) -> BaselineRow {
    BaselineRow {
        problem_slug: "two-sum".to_owned(),
        problem_version: version,
        lang: "python".to_owned(),
        n,
        median_ns,
        image_digest: digest.to_owned(),
        engine_version: "engine-1".to_owned(),
        fit_version: "fit-1".to_owned(),
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_baseline_insert_then_find_returns_sorted_points(pool: PgPool) {
    counterspell_judge::testkit::ensure_problem(&pool)
        .await
        .expect("the test problem must exist");
    let rows = [
        baseline(1, 1024, 300, "sha256:a"),
        baseline(1, 256, 100, "sha256:a"),
        baseline(1, 512, 200, "sha256:a"),
    ];

    assert_eq!(insert_baselines(&pool, &rows).await.unwrap(), 3);

    let found = find_baseline(&pool, "two-sum", 1, "python", "sha256:a")
        .await
        .unwrap();
    assert_eq!(found, vec![(256, 100), (512, 200), (1024, 300)]);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_baseline_reinsert_keeps_the_first_measurement(pool: PgPool) {
    counterspell_judge::testkit::ensure_problem(&pool)
        .await
        .expect("the test problem must exist");
    insert_baselines(&pool, &[baseline(1, 256, 100, "sha256:a")])
        .await
        .unwrap();

    let inserted = insert_baselines(&pool, &[baseline(1, 256, 999, "sha256:a")])
        .await
        .unwrap();

    assert_eq!(inserted, 0, "an existing measurement is not overwritten");
    let found = find_baseline(&pool, "two-sum", 1, "python", "sha256:a")
        .await
        .unwrap();
    assert_eq!(found, vec![(256, 100)]);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_baseline_new_version_is_a_separate_set(pool: PgPool) {
    counterspell_judge::testkit::ensure_problem(&pool)
        .await
        .expect("the test problem must exist");
    insert_baselines(
        &pool,
        &[
            baseline(1, 256, 100, "sha256:a"),
            baseline(2, 256, 150, "sha256:a"),
        ],
    )
    .await
    .unwrap();

    let old = find_baseline(&pool, "two-sum", 1, "python", "sha256:a")
        .await
        .unwrap();
    let new = find_baseline(&pool, "two-sum", 2, "python", "sha256:a")
        .await
        .unwrap();
    assert_eq!(old, vec![(256, 100)]);
    assert_eq!(new, vec![(256, 150)]);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_baseline_digest_mismatch_finds_nothing(pool: PgPool) {
    counterspell_judge::testkit::ensure_problem(&pool)
        .await
        .expect("the test problem must exist");
    insert_baselines(&pool, &[baseline(1, 256, 100, "sha256:a")])
        .await
        .unwrap();

    let found = find_baseline(&pool, "two-sum", 1, "python", "sha256:b")
        .await
        .unwrap();
    assert!(
        found.is_empty(),
        "a baseline from another image is not usable"
    );
}
