//! Problem catalogue: the importer, the publish rules and the test kit that builds catalogue rows.
//!
//! The database triggers and grants (migrations `20261116100000` to `…200`) enforce the rules.
//! The Rust code in this module builds rows through the same rules and never bypasses them.

pub mod approval;
pub mod discover;
pub mod manifest;
pub mod plan;
pub mod private_tree;
pub mod report;
pub mod retired;
pub mod starter;
pub mod statement;
pub mod taxonomy;
#[cfg(feature = "testkit")]
pub mod testkit;
pub mod write;
