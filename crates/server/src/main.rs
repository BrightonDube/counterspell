//! Start-up for `counterspell-server`. The logic lives in the library so tests can build the same router.

use std::{io, net::SocketAddr};

use counterspell_server::{
    build_router, config::ServerConfig, ensure_static_files, ratelimit::Limiter,
    ratelimit::ValkeyLimiter, AppState,
};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    // Logging must never print secrets or request bodies. Errors here are already safe messages.
    let _ = tracing_subscriber::fmt().with_writer(io::stderr).try_init();

    let config = ServerConfig::from_env()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    ensure_static_files(&config.static_dir)?;
    let address = config
        .address()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;

    // connect_lazy opens no connection yet, so a database outage does not stop the server from starting.
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect_lazy(&config.database_url)
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "DATABASE_URL is not a valid Postgres URL",
            )
        })?;

    let listener = TcpListener::bind(address).await.map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("could not bind counterspell-server to {address}: {error}"),
        )
    })?;

    let limiter = ValkeyLimiter::open(&config.valkey_url).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "VALKEY_URL is not a valid Valkey URL",
        )
    })?;

    axum::serve(
        listener,
        build_router(AppState::new(pool, config, Limiter::Valkey(limiter)))
            .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(|error| io::Error::other(format!("server stopped unexpectedly: {error}")))
}
