//! HTTP handlers for the auth routes. Each handler takes the signed-in user from an extractor and
//! never from the request.

pub mod config;
pub mod delete;
pub mod link;
pub mod login;
pub mod me;
pub mod oauth;
pub mod session;
