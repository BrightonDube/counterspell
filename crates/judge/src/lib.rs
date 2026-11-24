//! Trusted judge worker: claims queued runs from Postgres, runs them through the
//! sandbox runner and records the verdict. This crate holds the tests, the
//! expected outputs and the final verdict. User code never reaches this library's
//! decisions; it only returns raw answers.
//!
//! The judge never depends on the web server, and it has no inbound port.

#![warn(missing_docs)]

pub mod catalogue;
pub mod config;
pub mod executor;
pub mod familiar;
pub mod health;
// The catalogue file lives in counterspell-common so the server can read the enabled-driver
// contract without reaching the sandbox boundary (language-consistency LC03).
pub use counterspell_common::languages;
pub mod problem_store;
pub mod queue;
pub mod sandbox;
pub mod service;
#[cfg(feature = "testkit")]
pub mod testkit;
pub mod worker;

pub use config::{ConfigError, JudgeConfig};
