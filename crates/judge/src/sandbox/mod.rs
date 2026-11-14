//! The sandbox: runs untrusted code in throwaway containers on the separate rootless
//! Docker daemon, and turns what it observes into verdicts. The trusted worker owns
//! the tests, the expected outputs and the final verdict. User code only receives
//! inputs and prints raw answers.
//!
//! Modules marked "pure" in `design.md` section 5 do not touch Docker, the clock or
//! the filesystem, so their tests run anywhere.

pub mod backend;
pub mod compare;
pub mod config;
pub mod driver_protocol;
pub mod policy;
pub mod precheck;
pub mod records;
pub mod sanitize;
pub mod session;
pub mod spec;
pub mod stream;
#[cfg(feature = "testkit")]
pub mod testkit;
pub mod verdict;
