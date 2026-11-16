//! Pack authoring pipeline logic (`pack-authoring-pipeline` spec, design §4 and §10).
//!
//! Everything here is pure. It parses and checks text that is passed in, and it
//! never reads files, runs processes or opens sockets. Hidden values are passed in
//! memory and never returned. The scripts under `scripts/` move the files.

pub mod state;
