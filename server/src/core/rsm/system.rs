//! RSM system surface.
//!
//! The implementations live in `transitions.rs` to preserve the CI-enforced
//! `ResMut<RoundState>` single-writer file boundary.

pub use super::transitions::{on_session_ready, rsm_input_reader, tick_rsm_timers};
