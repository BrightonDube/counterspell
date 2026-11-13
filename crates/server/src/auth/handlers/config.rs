//! `GET /api/auth/config`: which sign-in options this deployment offers. It carries flags only,
//! never a secret, address or password.

use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::auth::config::AuthConfig;

/// The public flags the sign-in page needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicSettings {
    github: bool,
    google: bool,
    demo: bool,
}

impl PublicSettings {
    /// Builds the flags. A provider is offered only when it has credentials.
    #[must_use]
    pub const fn new(github: bool, google: bool, demo: bool) -> Self {
        Self {
            github,
            google,
            demo,
        }
    }

    /// Reads the flags from the loaded configuration.
    #[must_use]
    pub fn from_config(config: &AuthConfig) -> Self {
        Self::new(
            config.github.is_some(),
            config.google.is_some(),
            config.demo.is_some(),
        )
    }
}

#[derive(Serialize)]
struct Providers {
    github: bool,
    google: bool,
}

#[derive(Serialize)]
struct DemoFlag {
    enabled: bool,
}

#[derive(Serialize)]
struct ConfigView {
    providers: Providers,
    password: bool,
    magic_link: bool,
    demo: DemoFlag,
}

/// `GET /api/auth/config`.
async fn config(State(settings): State<PublicSettings>) -> Json<ConfigView> {
    Json(ConfigView {
        providers: Providers {
            github: settings.github,
            google: settings.google,
        },
        password: true,
        magic_link: true,
        demo: DemoFlag {
            enabled: settings.demo,
        },
    })
}

/// The router for the public config route, with its flags fixed at start-up.
pub fn config_router(settings: PublicSettings) -> Router {
    Router::new()
        .route("/api/auth/config", get(config))
        .with_state(settings)
}
