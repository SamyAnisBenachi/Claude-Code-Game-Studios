//! RSM system surface.
//!
//! The implementations live in `transitions.rs` to preserve the CI-enforced
//! `ResMut<RoundState>` single-writer file boundary.

pub use super::transitions::{
    on_lightyear_connected, on_lightyear_disconnected, on_session_ready, rsm_input_reader,
    tick_disconnect_timers, tick_rsm_timers,
};
