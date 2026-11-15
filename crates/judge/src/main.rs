//! Entry point for the trusted judge worker.
//!
//! With no arguments it reads its settings from the environment, connects to Postgres and runs the
//! queue until it gets `SIGINT` or `SIGTERM`. Until the sandbox runner exists it uses `NoExecutor`,
//! so every run ends as an internal error and never as a verdict.
//!
//! With the argument `healthcheck` it reads the heartbeat file and exits 0 or 1. It never connects
//! to Postgres and never starts the worker.

use counterspell_judge::config::JudgeConfig;
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
    if std::env::args().nth(1).as_deref() == Some("healthcheck") {
        return healthcheck();
    }
    match run_worker() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("judge stopped: {error}");
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

#[tokio::main]
async fn run_worker() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = JudgeConfig::from_env()?;
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(POOL_SIZE)
        .connect(&database_url)
        .await?;

    let (stop, shutdown) = watch::channel(false);
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
