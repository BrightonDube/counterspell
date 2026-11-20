// SPDX-License-Identifier: MIT
//! The sandbox pack runner now lives in `counterspell-judge`, so `counterspell-mcp` can share it
//! without depending on this operator crate. This module keeps the old import path working.

pub use counterspell_judge::sandbox::pack_runner::{
    PackRunnerFactory, PackSandboxRunner, REFERENCE_SOURCE,
};
