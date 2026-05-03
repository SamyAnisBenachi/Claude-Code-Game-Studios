pub mod events;
pub mod plugin;
pub mod state;
pub mod system;
pub mod transitions;

pub use events::*;
pub use plugin::{RsmPlugin, RsmSet};
// Scaffold API consumed by downstream stories.
#[allow(unused_imports)]
pub use state::{
    GameOverRequest, PendingPhaseAdvance, PhaseAdvanceRequest, RoundPhase, RoundState,
};
// Scaffold API consumed by downstream stories.
#[allow(unused_imports)]
pub use system::{
    on_lightyear_connected, on_lightyear_disconnected, on_session_ready, rsm_input_reader,
    tick_disconnect_timers, tick_rsm_timers,
};
// Scaffold API consumed by downstream stories.
#[allow(unused_imports)]
pub use transitions::{advance_phase, is_auction_round};
