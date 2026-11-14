//! Efficiency Familiar types shared by the judge, the server and the browser.
//!
//! Nothing here runs code, touches a database or draws anything.

pub mod class;
pub mod facts;
pub mod mood;

pub use class::{ClassId, Confidence, Effect, Mood, MoodOutOfRange, UnknownClass};
pub use facts::{BenchFacts, MeasuredFacts, NoMeasurement};
pub use mood::{decide, MoodDecision, MoodReason};
