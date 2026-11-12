use std::{env, io, net::SocketAddr, path::PathBuf, str::FromStr};

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;

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
        let static_dir = PathBuf::from(env::var("STATIC_DIR").unwrap_or_else(|_| "web/build".to_owned()));

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

fn app() -> Router {
    Router::new().route("/api/health", get(health))
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let config = Config::from_env()?;
    let address = config.address()?;
    let _static_dir = config.static_dir;
    let listener = TcpListener::bind(address).await.map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("could not bind counterspell-server to {address}: {error}"),
        )
    })?;

    axum::serve(listener, app())
        .await
        .map_err(|error| io::Error::other(format!("server stopped unexpectedly: {error}")))
}
