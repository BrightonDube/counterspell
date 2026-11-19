//! `counterspell-testserver`: a real Axum server plus a scripted judge for
//! web and browser verification. It runs the production router against a real
//! Postgres database, accepts `Bearer test-user-N` identities, publishes the
//! `two-sum` fixture problem and executes queued runs through
//! `ScriptedExecutor`, so clients see only real JSON envelopes and SSE frames.
//!
//! Only builds with `--features testkit`; release builds never contain it.
//!
//! ```sh
//! DATABASE_URL=postgres://... CS_TEST_PORT=5198 \
//!   cargo run -p counterspell-testserver
//! ```

use std::{env, net::SocketAddr, sync::Arc, time::Duration};

use counterspell_common::ids::UserId;
use counterspell_judge::{
    catalogue::testkit::insert_published_fixture,
    config::JudgeConfig,
    testkit::{Script, ScriptedExecutor},
    worker::Worker,
};
use counterspell_server::{
    build_router,
    config::ServerConfig,
    ratelimit::{Limiter, MemoryLimiter},
    runs::wake::RunWaker,
    testkit::{ensure_user, BearerTestResolver},
    AppState,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::{net::TcpListener, sync::watch};
use uuid::Uuid;

const TEST_USERS: u32 = 256;
const SCRIPTED_TESTS: u32 = 3;
const SCRIPT_DELAY: Duration = Duration::from_millis(30);

fn owner(number: u32) -> UserId {
    UserId::from_uuid(Uuid::from_u128(u128::from(number)))
}

fn server_config(database_url: &str) -> ServerConfig {
    let url = database_url.to_owned();
    ServerConfig::from_lookup(move |name| match name {
        "DATABASE_URL" => Some(url.clone()),
        // The fixture uses the in-memory limiter; a URL is still required by config.
        "VALKEY_URL" => Some("redis://127.0.0.1:1".to_owned()),
        "STATIC_DIR" => Some(env::var("STATIC_DIR").unwrap_or_else(|_| "web/build".to_owned())),
        _ => env::var(name).ok(),
    })
    .expect("server config")
}

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("connect to Postgres");
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("migrations");

    for number in 1..=TEST_USERS {
        ensure_user(&pool, owner(number))
            .await
            .expect("user fixture");
    }
    let published: Option<Option<i32>> =
        sqlx::query_scalar("SELECT published_version FROM problems WHERE slug = 'two-sum'")
            .fetch_optional(&pool)
            .await
            .expect("problem lookup");
    if !matches!(published, Some(Some(_))) {
        insert_published_fixture(&pool, "two-sum", 1)
            .await
            .expect("published two-sum fixture");
    }

    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let state = AppState::new(pool.clone(), server_config(&database_url), limiter)
        .with_user_resolver(BearerTestResolver)
        .with_run_waker(RunWaker::start(pool.clone()));

    let (stop_tx, stop_rx) = watch::channel(false);
    let judge_pool = PgPoolOptions::new()
        .max_connections(4)
        .connect_with((*pool.connect_options()).clone())
        .await
        .expect("judge pool");
    let worker = Arc::new(Worker::new(
        judge_pool,
        JudgeConfig::from_lookup(|_| None::<String>).expect("judge defaults"),
        ScriptedExecutor::new(Script::Pass {
            tests: SCRIPTED_TESTS,
            delay: SCRIPT_DELAY,
        }),
        "fixture-judge".to_owned(),
    ));
    tokio::spawn(worker.run(stop_rx));

    let port = env::var("CS_TEST_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(5198);
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
        .await
        .expect("bind test port");
    let address = listener.local_addr().expect("address");
    println!("testserver listening on http://{address}");
    axum::serve(
        listener,
        build_router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("serve");
    let _keep = stop_tx;
}
