//! The break room: a small, isolated, ungraded game area.
//!
//! Everything lives inside this module plus the `/break` web routes and the
//! `puzzles`/`game_scores` tables. Cutting the feature deletes the module, the
//! one `.nest("/api/breakroom", ...)` line in `lib.rs`, the nav entry, and the
//! web paths listed in `.kiro/specs/break-room/design.md`.

mod daily;
mod handlers;
mod scores;

use axum::{routing::get, routing::post, Router};

use crate::AppState;

/// The whole server surface of the feature.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/puzzle/daily", get(handlers::daily_puzzle))
        .route(
            "/scores",
            post(handlers::save_score).get(handlers::list_scores),
        )
        .fallback(handlers::breakroom_not_found)
}
