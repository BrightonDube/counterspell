//! Efficiency Familiar server pieces: the mascot preferences (`GET` and `PATCH /api/me/familiar`).
//! The measured mood never comes from here; it is computed by the judge.

pub mod api;
pub mod claims;
pub mod prefs;
