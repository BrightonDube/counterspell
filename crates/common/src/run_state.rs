//! The run state machine: which status may follow which, and how statuses
//! map to job states. This is the only Rust copy of the table. The database
//! function `cs_status_transition_allowed` mirrors it and a test compares them.
