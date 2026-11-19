//! Entry point for the trusted judge worker.
//!
//! With no arguments it reads its settings from the environment, connects to Postgres and runs the
//! queue until it gets `SIGINT` or `SIGTERM`. Until the sandbox runner exists it uses `NoExecutor`,
//! so every run ends as an internal error and never as a verdict.
//!
//! With the argument `healthcheck` it reads the heartbeat file and exits 0 or 1. It never connects
//! to Postgres and never starts the worker.
//!
//! With the argument `preflight` it runs the sandbox probe against every enabled language, prints
//! the JSON report and exits non-zero when any check fails. There is no skip flag.
//!
//! With the `sandbox` feature the worker runs submissions inside sandbox containers through
//! `SandboxExecutor`; without it the `NoExecutor` placeholder fails every run as an internal
//! error, never as a verdict.

use counterspell_judge::config::JudgeConfig;
#[cfg(not(feature = "sandbox"))]
use counterspell_judge::executor::NoExecutor;
use counterspell_judge::health;
use counterspell_judge::service::serve;
use sqlx::postgres::PgPoolOptions;
use std::process::ExitCode;
use std::time::SystemTime;
use tokio::sync::watch;

/// The most database connections the judge keeps open.
const POOL_SIZE: u32 = 8;

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("healthcheck") => return healthcheck(),
        #[cfg(feature = "sandbox")]
        Some("preflight") => return preflight(),
        _ => {}
    }
    match run_worker() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("judge stopped: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Prints the JSON preflight report for every enabled language and exits non-zero when
/// any check fails. A probe that cannot run at all counts as a failure, never a pass.
#[cfg(feature = "sandbox")]
#[tokio::main]
async fn preflight() -> ExitCode {
    use counterspell_judge::languages::LanguageCatalog;
    use counterspell_judge::sandbox::bollard_backend::BollardBackend;
    use counterspell_judge::sandbox::config::SandboxConfig;
    use counterspell_judge::sandbox::daemon::{connect, verify};
    use counterspell_judge::sandbox::preflight::{
        probe_failed_report, probe_spec, run_probe, PreflightReport,
    };
    use std::collections::BTreeMap;

    let outcome = async {
        let cfg = SandboxConfig::from_env()?;
        let docker = connect(&cfg.docker_socket, cfg.api_timeout)?;
        verify(&docker, cfg.runtime, cfg.api_timeout).await?;
        let catalog = LanguageCatalog::load(&cfg.languages_file)?;
        let backend = BollardBackend::new(docker, cfg.api_timeout);
        let mut reports: BTreeMap<String, PreflightReport> = BTreeMap::new();
        let mut all_passed = true;
        for name in catalog.names() {
            let Some(lang) = catalog.get(name).filter(|lang| lang.is_enabled()) else {
                continue;
            };
            let spec = probe_spec(lang, cfg.run_wall_cap, cfg.runtime)?;
            let report = match run_probe(&backend, &spec).await {
                Ok(report) => report,
                Err(error) => probe_failed_report(&error),
            };
            all_passed &= report.passed();
            reports.insert(name.to_owned(), report);
        }
        let json = serde_json::to_string_pretty(&reports)?;
        Ok::<_, Box<dyn std::error::Error>>((json, all_passed))
    };
    match outcome.await {
        Ok((json, true)) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Ok((json, false)) => {
            println!("{json}");
            eprintln!("preflight: one or more sandbox checks failed");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("preflight could not run: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Prints one line saying whether the heartbeat is fresh, and returns the matching exit code.
fn healthcheck() -> ExitCode {
    match health::report(&health::heartbeat_path(), SystemTime::now()) {
        Ok(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(line) => {
            println!("{line}");
            ExitCode::FAILURE
        }
    }
}

/// How often each language's sandbox preflight is re-probed while the worker runs.
#[cfg(feature = "sandbox")]
const PREFLIGHT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60);

/// Builds the real executor: the rootless sandbox daemon, the language catalogue, the
/// sweeper and one preflight loop per enabled language. Returns `Err` when the sandbox
/// cannot be verified — a judge that cannot sandbox must not start.
#[cfg(feature = "sandbox")]
async fn sandbox_executor(
    stop: &watch::Sender<bool>,
    pool: sqlx::PgPool,
) -> Result<
    counterspell_judge::sandbox::sandbox_executor::SandboxExecutor<
        counterspell_judge::problem_store::PgProblemTests,
        counterspell_judge::sandbox::bollard_backend::BollardBackend,
    >,
    Box<dyn std::error::Error>,
> {
    use counterspell_judge::languages::LanguageCatalog;
    use counterspell_judge::problem_store::PgProblemTests;
    use counterspell_judge::sandbox::bollard_backend::BollardBackend;
    use counterspell_judge::sandbox::config::SandboxConfig;
    use counterspell_judge::sandbox::daemon::{connect, verify};
    use counterspell_judge::sandbox::preflight::{self, PreflightState};
    use counterspell_judge::sandbox::reaper;
    use counterspell_judge::sandbox::sandbox_executor::SandboxExecutor;
    use std::sync::Arc;

    let cfg = SandboxConfig::from_env()?;
    let docker = connect(&cfg.docker_socket, cfg.api_timeout)?;
    verify(&docker, cfg.runtime, cfg.api_timeout).await?;
    let catalog = LanguageCatalog::load(&cfg.languages_file)?;

    tokio::spawn(reaper::run_sweeper(
        BollardBackend::new(docker.clone(), cfg.api_timeout),
        cfg.reaper_interval,
        cfg.orphan_age,
        stop.subscribe(),
    ));

    // One preflight state per enabled language; the executor refuses every run while any
    // state reports a failure, and an absent report counts as failing.
    let mut preflights = Vec::new();
    for name in catalog.names() {
        let Some(lang) = catalog.get(name).filter(|lang| lang.is_enabled()) else {
            continue;
        };
        let lang = lang.clone();
        let state = Arc::new(PreflightState::new());
        preflights.push(state.clone());
        let (wall, runtime) = (cfg.run_wall_cap, cfg.runtime);
        tokio::spawn(preflight::run(
            state,
            BollardBackend::new(docker.clone(), cfg.api_timeout),
            move || preflight::probe_spec(&lang, wall, runtime),
            PREFLIGHT_INTERVAL,
            stop.subscribe(),
        ));
    }

    Ok(SandboxExecutor::new(
        catalog,
        PgProblemTests::new(pool),
        BollardBackend::new(docker, cfg.api_timeout),
        cfg,
        preflights,
        "worker",
    ))
}

#[tokio::main]
async fn run_worker() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = JudgeConfig::from_env()?;
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(POOL_SIZE)
        .connect(&database_url)
        .await?;

    let (stop, shutdown) = watch::channel(false);
    #[cfg(feature = "sandbox")]
    let serving = tokio::spawn(serve(
        pool.clone(),
        cfg,
        sandbox_executor(&stop, pool).await?,
        shutdown,
    ));
    #[cfg(not(feature = "sandbox"))]
    let serving = tokio::spawn(serve(pool, cfg, NoExecutor, shutdown));
    wait_for_stop_signal().await?;
    // The receivers live inside `serve`, so this only fails after it has already stopped.
    let _ = stop.send(true);
    serving.await?;
    Ok(())
}

/// Waits until the process gets `SIGINT` (Ctrl-C) or, on Unix, `SIGTERM`.
async fn wait_for_stop_signal() -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => result,
            _ = terminate.recv() => Ok(()),
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await
    }
}
