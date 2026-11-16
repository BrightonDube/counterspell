//! AI and authored hints: the ladder, the AI hint client, the quota and the cost record.
//!
//! The design is in `.kiro/specs/hints/design.md`. Only `provider.rs` knows HTTP, only `store.rs`
//! knows SQL, and only `quota.rs` knows Valkey.

pub mod cache;
pub mod chain;
pub mod config;
pub mod cost;
pub mod guard;
pub mod ladder;
