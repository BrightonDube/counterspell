//! `GET /api/auth/config`: which sign-in options this deployment offers. It carries flags only,
//! never a secret, address or password.

use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::auth::config::AuthConfig;

/// The public flags the sign-in page needs. The bools are independent feature flags.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "each bool is an independent sign-in flag reported to the page"
)]
pub struct PublicSettings {
    github: bool,
    google: bool,
    demo: bool,
    /// The demo account's address, shown so the sign-in page can pre-fill it. `None` when the
    /// demo account is disabled.
    demo_email: Option<String>,
    /// Always `false`: email sign-in is retired and no email method is ever offered.
    email_enabled: bool,
}

impl PublicSettings {
    /// Builds the flags with email and demo details off. A provider is offered only when it has
    /// credentials; use [`PublicSettings::from_config`] for the real deployment.
    #[must_use]
    pub const fn new(github: bool, google: bool, demo: bool) -> Self {
        Self {
            github,
            google,
            demo,
            demo_email: None,
            email_enabled: false,
        }
    }

    /// The configured demo address, when the demo account is offered. The password route accepts
    /// only this address.
    #[must_use]
    pub fn demo_email(&self) -> Option<&str> {
        self.demo_email.as_deref()
    }

    /// Sets the demo address, for tests that exercise the password route.
    #[cfg(any(test, feature = "testkit"))]
    #[must_use]
    pub fn with_demo_email(mut self, email: &str) -> Self {
        self.demo_email = Some(email.to_owned());
        self
    }

    /// Reads the flags from the loaded configuration. Email sign-in is retired: the sign-in page
    /// offers providers, passkeys and the demo account, whatever the mail configuration holds.
    #[must_use]
    pub fn from_config(config: &AuthConfig) -> Self {
        Self {
            github: config.github.is_some(),
            google: config.google.is_some(),
            demo: config.demo.is_some(),
            demo_email: config.demo.as_ref().map(|demo| demo.email.clone()),
            email_enabled: false,
        }
    }
}

#[derive(Serialize)]
struct Providers {
    github: bool,
    google: bool,
}

#[derive(Serialize)]
struct EmailFlags {
    magic_link: bool,
    password: bool,
}

#[derive(Serialize)]
struct DemoFlag {
    enabled: bool,
    email: Option<String>,
}

#[derive(Serialize)]
struct ConfigView {
    providers: Providers,
    email: EmailFlags,
    demo: DemoFlag,
}

/// `GET /api/auth/config`.
async fn config(State(settings): State<PublicSettings>) -> Json<ConfigView> {
    Json(ConfigView {
        providers: Providers {
            github: settings.github,
            google: settings.google,
        },
        email: EmailFlags {
            magic_link: settings.email_enabled,
            // Email sign-in is retired, so no email method is ever advertised.
            password: false,
        },
        demo: DemoFlag {
            enabled: settings.demo,
            email: settings.demo_email,
        },
    })
}

/// The router for the public config route, with its flags fixed at start-up.
pub fn config_router(settings: PublicSettings) -> Router {
    Router::new()
        .route("/api/auth/config", get(config))
        .with_state(settings)
}
