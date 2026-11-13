//! Sign-in, sessions, email links, OAuth sign-in and linking, passwords, and the judge demo account.
//!
//! Everything in this module is server-side. Postgres is the only store for users, identities,
//! sessions, email tokens and OAuth flows. Valkey holds counters only.

pub mod clock;
pub mod config;
pub mod errors;
