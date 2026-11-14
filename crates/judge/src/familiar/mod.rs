//! Efficiency Familiar, judge side: pure assessment helpers and, later, the bench orchestration.
//!
//! Nothing in this module runs code or touches the database. The sandbox and the storage live in
//! their own modules and call into these functions.

pub mod assess;
pub mod ratio;
