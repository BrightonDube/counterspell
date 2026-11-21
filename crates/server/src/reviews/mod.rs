//! Review scheduling primitives.
//!
//! The scheduler lives behind `FsrsScheduler` so callers work with Counterspell-owned
//! card, grade, and calculation types rather than the third-party API.

pub mod fsrs_adapter;

pub use fsrs_adapter::{
    CardState, FsrsScheduler, PersistedCard, ReviewCalculation, ReviewError, ReviewGrade,
    SchedulerConfig,
};
