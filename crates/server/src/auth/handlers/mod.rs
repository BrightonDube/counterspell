//! HTTP handlers for the auth routes. Each handler takes the signed-in user from an extractor and
//! never from the request.

pub mod login;
pub mod me;
pub mod session;
