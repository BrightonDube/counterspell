//! Readiness route `GET /api/ready` (deploy-ops T06, R10.1, R6.6, P12).
//!
//! The dependencies are fakes, so no test needs a real Postgres or Valkey. Each fake answers up,
//! fails at once, or never answers.

mod support;

use std::{sync::Arc, time::Instant};

use async_trait::async_trait;
use counterspell_server::{build_router, health::Probe, health::ProbeFailure};
use support::{get, serve, state_for, Fixture};

/// How a fake dependency behaves when it is pinged.
#[derive(Clone, Copy)]
enum Behaviour {
    Up,
    Down,
    Hangs,
}

struct FakeProbe {
    name: &'static str,
    behaviour: Behaviour,
}

#[async_trait]
impl Probe for FakeProbe {
    fn name(&self) -> &'static str {
        self.name
    }

    async fn ping(&self) -> Result<(), ProbeFailure> {
        match self.behaviour {
            Behaviour::Up => Ok(()),
            Behaviour::Down => Err(ProbeFailure),
            Behaviour::Hangs => {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                Ok(())
            }
        }
    }
}

fn fake(name: &'static str, behaviour: Behaviour) -> Arc<dyn Probe> {
    Arc::new(FakeProbe { name, behaviour })
}

/// Serves the router with the given Postgres and Valkey behaviours.
async fn serve_with(postgres: Behaviour, valkey: Behaviour) -> (Fixture, std::net::SocketAddr) {
    let fixture = Fixture::new();
    let state = state_for(&fixture.dir)
        .with_readiness(vec![fake("postgres", postgres), fake("valkey", valkey)]);
    let address = serve(build_router(state)).await;
    (fixture, address)
}

#[tokio::test]
async fn ready_returns_ok_when_postgres_and_valkey_answer() {
    let (_fixture, address) = serve_with(Behaviour::Up, Behaviour::Up).await;

    let reply = get(address, "/api/ready", &[]).await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, r#"{"status":"ok"}"#);
}

#[tokio::test]
async fn ready_returns_503_with_no_detail_when_postgres_is_down() {
    let (_fixture, address) = serve_with(Behaviour::Down, Behaviour::Up).await;

    let reply = get(address, "/api/ready", &[]).await;

    assert_eq!(reply.status, 503);
    assert_eq!(reply.body, r#"{"status":"unavailable"}"#);
    assert!(!reply.body.contains("postgres://"));
    assert!(!reply.body.contains("postgres"));
}

#[tokio::test]
async fn ready_returns_503_when_valkey_is_down() {
    let (_fixture, address) = serve_with(Behaviour::Up, Behaviour::Down).await;

    let reply = get(address, "/api/ready", &[]).await;

    assert_eq!(reply.status, 503);
    assert_eq!(reply.body, r#"{"status":"unavailable"}"#);
}

#[tokio::test]
async fn ready_gives_up_on_a_dependency_that_hangs_within_three_seconds() {
    let (_fixture, address) = serve_with(Behaviour::Up, Behaviour::Hangs).await;

    let started = Instant::now();
    let reply = get(address, "/api/ready", &[]).await;
    let elapsed = started.elapsed();

    assert_eq!(reply.status, 503);
    assert!(
        elapsed.as_secs_f64() < 3.0,
        "readiness took {elapsed:?}, longer than the 2 s probe timeout allows"
    );
}

#[tokio::test]
async fn health_stays_ok_even_when_a_dependency_is_down() {
    let (_fixture, address) = serve_with(Behaviour::Down, Behaviour::Down).await;

    let reply = get(address, "/api/health", &[]).await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, r#"{"status":"ok"}"#);
}
