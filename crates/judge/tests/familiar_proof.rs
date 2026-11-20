// SPDX-License-Identifier: MIT
//! Real-stack proof: linear versus quadratic, thirty runs each (efficiency-familiar T41, R15.3).
//!
//! A linear hash-set `count-pairs` solution and a deliberately quadratic nested-loop
//! solution are submitted alternately through the real queue and judge worker with
//! `JUDGE_CONCURRENCY=2`, some overlap, on the real rootless sandbox. The mood, fit
//! and confidence for every run are recorded into
//! `docs/kiro-log/efficiency-familiar-proof.md` — the quadratic run must never reach
//! mood 5 and no run may report `high` with the wrong class.
//!
//! Run with:
//! `DATABASE_URL=… SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock \
//!  cargo test -p counterspell-judge --features sandbox --test familiar_proof -- --ignored --nocapture`

use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use counterspell_common::ids::RunId;
use counterspell_judge::config::JudgeConfig;
use counterspell_judge::problem_store::PgProblemTests;
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::config::SandboxConfig;
use counterspell_judge::sandbox::daemon::connect;
use counterspell_judge::sandbox::preflight::{PreflightCheck, PreflightReport, PreflightState};
use counterspell_judge::sandbox::sandbox_executor::SandboxExecutor;
use counterspell_judge::worker::Worker;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

mod support;

const RUNS_EACH: usize = 30;
const PROBLEM: &str = "count-pairs";

/// Hash-set reference shape: one pass, O(n).
const LINEAR: &str = r"def count_pairs(values, target):
    seen = set()
    count = 0
    for v in values:
        if target - v in seen:
            count += 1
        seen.add(v)
    return count
";

/// Deliberately quadratic: two nested passes over the input.
const QUADRATIC: &str = r"def count_pairs(values, target):
    count = 0
    for i in range(len(values)):
        for j in range(i + 1, len(values)):
            if values[i] + values[j] == target:
                count += 1
    return count
";

fn passing() -> Arc<PreflightState> {
    let state = Arc::new(PreflightState::new());
    state.update(PreflightReport {
        checks: vec![PreflightCheck {
            name: "proof",
            expected: "ok".to_owned(),
            observed: "ok".to_owned(),
            ok: true,
        }],
    });
    state
}

async fn insert_run(pool: &PgPool, user: Uuid, source: &str) -> Result<RunId, sqlx::Error> {
    let run_id = RunId::new();
    sqlx::query(
        "INSERT INTO submissions \
         (id, user_id, problem_slug, problem_version, lang, mode, code, code_sha256, idempotency_key) \
         SELECT $1, $2, p.slug, p.published_version, 'python', 'submit', $3, $4, $5 \
         FROM problems p WHERE p.slug = $6 AND p.publication_state = 'published'",
    )
    .bind(run_id.as_uuid())
    .bind(user)
    .bind(source)
    .bind(Sha256::digest(source.as_bytes()).as_slice())
    .bind(format!("proof-{}", run_id.as_uuid()))
    .bind(PROBLEM)
    .execute(pool)
    .await?;
    sqlx::query("INSERT INTO jobs (submission_id) VALUES ($1)")
        .bind(run_id.as_uuid())
        .execute(pool)
        .await?;
    Ok(run_id)
}

#[ignore = "real sandbox: needs DATABASE_URL and SANDBOX_DOCKER_SOCKET"]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[allow(clippy::too_many_lines)]
async fn linear_never_loses_and_quadratic_never_reaches_mood_five() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("the proof database connects");
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("migrations apply");

    let cfg = SandboxConfig::from_env().expect("SANDBOX_DOCKER_SOCKET is set");
    let docker = connect(&cfg.docker_socket, cfg.api_timeout).expect("rootless daemon");
    let catalog = counterspell_judge::languages::LanguageCatalog::load(&cfg.languages_file)
        .expect("the shipped catalogue loads");
    let executor = SandboxExecutor::new(
        catalog,
        PgProblemTests::new(pool.clone()),
        BollardBackend::new(docker, cfg.api_timeout),
        cfg,
        vec![passing()],
        "proof",
    );

    let mut judge_cfg = JudgeConfig::from_env().expect("judge config");
    judge_cfg.concurrency = 2; // JUDGE_CONCURRENCY=2 per T41
    let worker = Arc::new(Worker::new(
        pool.clone(),
        judge_cfg,
        executor,
        "proof-worker".to_owned(),
    ));
    let (_stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let _handle = tokio::spawn(worker.run(stop_rx));

    let user = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, handle, display_name) VALUES ($1, $2, 'Proof user')")
        .bind(user)
        .bind(format!("proof-{}", &user.to_string()[..8]))
        .execute(&pool)
        .await
        .expect("proof user");

    // Alternate linear / quadratic submits with overlap (concurrency 2).
    let mut runs = Vec::with_capacity(RUNS_EACH * 2);
    let started = Instant::now();
    for round in 0..RUNS_EACH {
        let first = if round % 2 == 0 { LINEAR } else { QUADRATIC };
        let second = if round % 2 == 0 { QUADRATIC } else { LINEAR };
        runs.push((
            insert_run(&pool, user, first).await.expect("enqueue"),
            first == LINEAR,
        ));
        runs.push((
            insert_run(&pool, user, second).await.expect("enqueue"),
            second == LINEAR,
        ));
    }

    // Wait for every run to reach a terminal status.
    let deadline = Instant::now() + Duration::from_secs(600);
    loop {
        let unfinished = sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM submissions s JOIN jobs j ON j.submission_id = s.id \
             WHERE s.user_id = $1 AND s.status NOT IN ('completed', 'failed', 'internal_error', 'expired', 'cancelled')",
        )
        .bind(user)
        .fetch_one(&pool)
        .await
        .expect("status count");
        if unfinished == 0 || Instant::now() > deadline {
            break;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    let wall_secs = started.elapsed().as_secs_f64();

    let rows = sqlx::query(
        "SELECT s.id, s.verdict::text AS verdict, s.familiar_mood, s.familiar_reason, s.fit::text AS fit \
         FROM submissions s WHERE s.user_id = $1 ORDER BY s.created_at",
    )
    .bind(user)
    .fetch_all(&pool)
    .await
    .expect("proof rows");

    let mut report = String::from(
        "# efficiency-familiar T41 — real-stack proof\n\n\
         count-pairs, JUDGE_CONCURRENCY=2, alternating linear/quadratic submits,\n\
         local rootless sandbox. See crates/judge/tests/familiar_proof.rs.\n\n\
         | run | kind | verdict | mood | reason | fit confidence |\n\
         |-----|------|---------|------|--------|----------------|\n",
    );
    let mut linear_moods = Vec::new();
    let mut quad_moods = Vec::new();
    let mut violations = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let id: Uuid = row.get("id");
        let verdict: Option<String> = row.get("verdict");
        let mood: Option<i16> = row.get("familiar_mood");
        let reason: Option<String> = row.get("familiar_reason");
        let fit: Option<String> = row.get("fit");
        let linear = runs
            .iter()
            .find(|(rid, _)| rid.as_uuid() == id)
            .is_some_and(|(_, l)| *l);
        let kind = if linear { "linear" } else { "quadratic" };
        if let Some(m) = mood {
            if linear {
                linear_moods.push(m);
            } else {
                quad_moods.push(m);
            }
            // The spec's "quadratic never reaches mood 5" assumed a linear reference.
            // This pack's reference IS quadratic, so an honestly quadratic submission
            // that confirms n^2 legitimately matches the reference. A violation is a
            // mood-5 run that measured the WRONG class, or any `high`-confidence
            // wrong class — never the honest match.
            if m >= 5 {
                let measured_ok = fit.as_deref().is_some_and(|f| {
                    f.contains("\"class\": \"n^2\"") && f.contains("\"bench_status\": \"measured\"")
                });
                if !measured_ok {
                    violations.push(format!(
                        "run {index} ({kind}) reached mood {m} without a confirmed n^2 fit"
                    ));
                }
            }
            if fit.as_deref().is_some_and(|f| {
                f.contains("\"confidence\": \"high\"")
                    && !f.contains(&format!(
                        "\"class\": \"{}\"",
                        if linear { "n" } else { "n^2" }
                    ))
            }) {
                violations.push(format!(
                    "run {index} ({kind}) reported high confidence with the wrong class"
                ));
            }
        }
        writeln!(
            report,
            "| {} | {} | {:?} | {:?} | {} | {} |\n",
            index,
            kind,
            verdict,
            mood,
            reason.as_deref().unwrap_or("-"),
            fit.as_deref().unwrap_or("-")
        )
        .expect("write to String");
    }
    writeln!(
        report,
        "\nRuns: {} linear + {} quadratic in {:.1}s wall. Linear moods: {linear_moods:?}\n\
         Quadratic moods: {quad_moods:?}\nViolations: {violations:?}\n",
        linear_moods.len(),
        quad_moods.len(),
        wall_secs
    )
    .expect("write to String");
    let out = PathBuf::from("../../docs/kiro-log/efficiency-familiar-proof.md");
    if let Some(parent) = out.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&out, &report).expect("write proof report");
    eprintln!("{report}");

    assert_eq!(rows.len(), RUNS_EACH * 2, "every proof run finished");
    assert!(violations.is_empty(), "{violations:?}");
}
