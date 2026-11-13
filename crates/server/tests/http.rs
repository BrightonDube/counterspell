//! HTTP behaviour of the router: health, the JSON 404 for unknown API paths, static files and the SPA fallback.
//! These run a real listener on 127.0.0.1 and send raw HTTP/1.1 so no extra test dependency is needed.

use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use counterspell_server::{build_router, config::ServerConfig, AppState};
use sqlx::postgres::PgPoolOptions;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

/// A static directory with an index page and one asset, removed when dropped.
struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new() -> Self {
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

struct Reply {
    status: u16,
    headers: String,
    body: String,
}

async fn spawn(dir: PathBuf) -> SocketAddr {
    let config = ServerConfig::from_lookup(|name| match name {
        "DATABASE_URL" => Some("postgres://app@127.0.0.1:1/counterspell".to_owned()),
        "VALKEY_URL" => Some("redis://127.0.0.1:1".to_owned()),
        "STATIC_DIR" => Some(dir.to_string_lossy().into_owned()),
        _ => None,
    })
    .expect("config loads");
    // connect_lazy does not open a connection, so these tests need no database.
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://app@127.0.0.1:1/counterspell")
        .expect("lazy pool");
    let router = build_router(AppState::new(pool, config));

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    address
}

async fn get(address: SocketAddr, path: &str) -> Reply {
    let mut stream = TcpStream::connect(address).await.expect("connect");
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
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

#[tokio::test]
async fn health_reports_ok_as_json() {
    let fixture = Fixture::new();
    let address = spawn(fixture.dir.clone()).await;

    let reply = get(address, "/api/health").await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, r#"{"status":"ok"}"#);
}

#[tokio::test]
async fn unknown_api_path_is_a_json_404_not_html() {
    let fixture = Fixture::new();
    let address = spawn(fixture.dir.clone()).await;

    for path in ["/api", "/api/does-not-exist", "/api/runs/abc/unknown"] {
        let reply = get(address, path).await;

        assert_eq!(reply.status, 404, "{path}");
        assert!(
            reply.headers.to_lowercase().contains("application/json"),
            "{path}"
        );
        assert!(
            reply.body.contains(r#""code":"not_found""#),
            "{path}: {}",
            reply.body
        );
        assert!(!reply.body.contains("<html"), "{path} returned HTML");
    }
}

#[tokio::test]
async fn existing_static_file_is_served() {
    let fixture = Fixture::new();
    let address = spawn(fixture.dir.clone()).await;

    let reply = get(address, "/asset.txt").await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, "plain-asset");
}

#[tokio::test]
async fn unknown_page_falls_back_to_the_spa_entry() {
    let fixture = Fixture::new();
    let address = spawn(fixture.dir.clone()).await;

    let reply = get(address, "/runs/123").await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, "<main>spa-entry</main>");
}
