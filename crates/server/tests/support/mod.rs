//! Helpers shared by the HTTP integration tests. They run a real listener on 127.0.0.1 and send raw
//! HTTP/1.1, so no extra test dependency is needed. Nothing here opens a database connection.

#![allow(dead_code)]

use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use axum::Router;
use std::sync::Arc;

use counterspell_server::{
    config::ServerConfig,
    ratelimit::{Limiter, MemoryLimiter},
    AppState,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

/// A static directory with an index page and one asset, removed when dropped.
pub struct Fixture {
    pub dir: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "counterspell-server-test-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("fixture dir");
        std::fs::write(dir.join("index.html"), "<main>spa-entry</main>").expect("index");
        std::fs::write(dir.join("asset.txt"), "plain-asset").expect("asset");
        Self { dir }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

pub struct Reply {
    pub status: u16,
    pub headers: String,
    pub body: String,
}

/// A pool that never connects. Tests that need no database use it.
pub fn lazy_pool() -> PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgres://app@127.0.0.1:1/counterspell")
        .expect("lazy pool")
}

/// Settings that point static files at `static_dir` and need no real services.
pub fn config_for(static_dir: &std::path::Path) -> ServerConfig {
    let dir = static_dir.to_string_lossy().into_owned();
    ServerConfig::from_lookup(move |name| match name {
        "DATABASE_URL" => Some("postgres://app@127.0.0.1:1/counterspell".to_owned()),
        "VALKEY_URL" => Some("redis://127.0.0.1:1".to_owned()),
        "STATIC_DIR" => Some(dir.clone()),
        _ => None,
    })
    .expect("config loads")
}

/// Starts `router` on a random local port and returns its address.
pub async fn serve(router: Router) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    address
}

/// Sends a `GET` with extra headers and reads the whole reply.
pub async fn get(address: SocketAddr, path: &str, headers: &[(&str, &str)]) -> Reply {
    let mut stream = TcpStream::connect(address).await.expect("connect");
    let mut request = String::from("GET ");
    request.push_str(path);
    request.push_str(" HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n");
    for (name, value) in headers {
        request.push_str(name);
        request.push_str(": ");
        request.push_str(value);
        request.push_str("\r\n");
    }
    request.push_str("\r\n");
    stream.write_all(request.as_bytes()).await.expect("write");
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.expect("read");
    let text = String::from_utf8_lossy(&raw).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((text.as_str(), ""));
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    Reply {
        status,
        headers: head.to_owned(),
        body: body.to_owned(),
    }
}

/// Builds the app state for tests with no database and an in-memory rate limiter.
pub fn state_for(static_dir: &std::path::Path) -> AppState {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    AppState::new(lazy_pool(), config_for(static_dir), limiter)
}
