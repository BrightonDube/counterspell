//! Start-up for `counterspell-server`. The logic lives in the library so tests can build the same router.

use std::{io, net::SocketAddr};

use counterspell_server::{
    auth::{
        config::AuthConfig,
        setup::{build_state, start_background},
    },
    build_router,
    config::ServerConfig,
    ensure_static_files,
    ratelimit::Limiter,
    ratelimit::ValkeyLimiter,
    AppState,
};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    // Logging must never print secrets or request bodies. Errors here are already safe messages.
    let _ = tracing_subscriber::fmt().with_writer(io::stderr).try_init();

    let config = ServerConfig::from_env()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    // Auth settings are checked before anything listens, so a bad deploy refuses to start.
    let auth_config = AuthConfig::from_env()
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

    let limiter = Limiter::Valkey(ValkeyLimiter::open(&config.valkey_url).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "VALKEY_URL is not a valid Valkey URL",
        )
    })?);

    let auth = build_state(
        pool.clone(),
        &auth_config,
        limiter.clone(),
        config.trusted_proxies.clone(),
    )
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    // The cleanup and demo-reset tasks run for the life of the process.
    let _background_tasks = start_background(&auth, &auth_config).await;
    let hints =
        counterspell_server::hints::handlers::hint_state_from_env(pool.clone(), &config.valkey_url)
            .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message))?;
    let attempt = counterspell_server::familiar::attempt::AttemptMarker::open(&config.valkey_url)
        .map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "VALKEY_URL is not a valid Valkey URL",
        )
    })?;
    let state = AppState::new(pool, config, limiter)
        .with_hints(hints)
        .with_familiar_attempt(attempt)
        .with_auth(auth, &auth_config.public_base_url)
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "PUBLIC_BASE_URL is not an origin",
            )
        })?;

    axum::serve(
        listener,
        build_router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(|error| io::Error::other(format!("server stopped unexpectedly: {error}")))
}
