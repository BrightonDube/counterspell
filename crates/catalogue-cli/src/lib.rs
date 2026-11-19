// SPDX-License-Identifier: MIT
//! Catalogue operator tooling: the pack-tree readers and the sandbox fill that
//! sit behind the `counterspell-catalogue` binary (problem-catalogue C19).
//!
//! These modules are the authoring path — they depend on
//! `counterspell-grimoire`, which the trusted judge must not pull into its
//! sandbox path (the 15 Nov decision). The row model, the publish rules and the
//! database writes live in `counterspell_judge::catalogue`; the runner over the
//! rootless sandbox daemon is [`pack_runner`].

#![warn(missing_docs)]

pub mod load;
#[cfg(feature = "sandbox")]
pub mod pack_runner;
pub mod sandbox_fill;
