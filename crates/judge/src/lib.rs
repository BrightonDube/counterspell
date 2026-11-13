//! Trusted judge worker: claims queued runs from Postgres, runs them through the
//! sandbox runner and records the verdict. This crate holds the tests, the
//! expected outputs and the final verdict. User code never reaches this library's
//! decisions; it only returns raw answers.
//!
//! The judge never depends on the web server, and it has no inbound port.

#![warn(missing_docs)]

pub mod config;
pub mod executor;
pub mod queue;
pub mod service;
#[cfg(feature = "testkit")]
pub mod testkit;
pub mod worker;

pub use config::{ConfigError, JudgeConfig};
