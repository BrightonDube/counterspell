//! `counterspell-mcp`: a Model Context Protocol stdio server for Kiro.
//!
//! The server never runs code on the developer's machine. Pure tools call the engine
//! and Grimoire libraries. Code-running tools need a trusted runner that uses only
//! the separate rootless sandbox daemon.

#![warn(missing_docs)]

pub mod config;
pub mod errors;
pub mod judge_adapter;
pub mod paths;
pub mod runner;
pub mod schemas;
pub mod server;
pub mod tools;
