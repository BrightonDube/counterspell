//! `GET /api/languages` (problem-catalogue C32, CAT-A7, CAT-A8).
//!
//! The response is cloned out of the server's catalogue: entries are copied
//! into the DTO, so a request can never mutate the shared `Arc<LanguageCatalog>`
//! — a shared reference into the response would tie the catalogue's lifetime
//! to the request and let a handler change accepted limits mid-flight.

use axum::{extract::State, Json};

use super::dto::{LanguageListResponse, LanguageSummary};
use crate::AppState;

/// `GET /api/languages`. Public; anonymous callers see the same list.
pub async fn get_languages(State(state): State<AppState>) -> Json<LanguageListResponse> {
    let languages = state
        .languages
        .entries()
        .iter()
        .map(|language| LanguageSummary {
            id: language.name.clone(),
            display_name: language.display_name.clone(),
            max_source_bytes: language.max_source_bytes,
        })
        .collect();
    Json(LanguageListResponse { languages })
}
