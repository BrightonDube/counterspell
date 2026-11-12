use std::{
    env, io,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get},
    Json, Router,
};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Debug)]
struct Config {
    host: String,
    port: u16,
    static_dir: PathBuf,
}

impl Config {
    fn from_env() -> Result<Self, io::Error> {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
        let port = parse_port(env::var("PORT").unwrap_or_else(|_| "8080".to_owned()))?;
        let static_dir =
            PathBuf::from(env::var("STATIC_DIR").unwrap_or_else(|_| "web/build".to_owned()));

        Ok(Self {
            host,
            port,
            static_dir,
        })
    }

    fn address(&self) -> Result<SocketAddr, io::Error> {
        let address = format!("{}:{}", self.host, self.port);
        SocketAddr::from_str(&address).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid server address {address}: {error}"),
            )
        })
    }
}

fn parse_port(value: String) -> Result<u16, io::Error> {
    value.parse::<u16>().map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("PORT must be a number from 0 to 65535: {error}"),
        )
    })
}

fn app(static_dir: PathBuf) -> Router {
    let index_file = static_dir.join("index.html");
    let static_files = ServeDir::new(static_dir).fallback(ServeFile::new(index_file));

    Router::new()
        .route("/api/health", get(health))
        .route("/api", any(api_not_found))
        .route("/api/{*path}", any(api_not_found))
        .fallback_service(static_files)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn api_not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "code": "not_found",
            "message": "API route not found"
        })),
    )
}

fn ensure_static_files(static_dir: &Path) -> Result<(), io::Error> {
    let metadata = std::fs::metadata(static_dir).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "static directory {} is unavailable: {error}",
                static_dir.display()
            ),
        )
    })?;
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("static path {} is not a directory", static_dir.display()),
        ));
    }

    let index_file = static_dir.join("index.html");
    let index_metadata = std::fs::metadata(&index_file).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "SPA entry point {} is unavailable: {error}",
                index_file.display()
            ),
        )
    })?;
    if !index_metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("SPA entry point {} is not a file", index_file.display()),
        ));
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let config = Config::from_env()?;
    ensure_static_files(&config.static_dir)?;
    let address = config.address()?;
    let listener = TcpListener::bind(address).await.map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("could not bind counterspell-server to {address}: {error}"),
        )
    })?;

    axum::serve(listener, app(config.static_dir))
        .await
        .map_err(|error| io::Error::other(format!("server stopped unexpectedly: {error}")))
}
