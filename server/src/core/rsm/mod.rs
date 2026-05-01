pub mod events;
pub mod plugin;
pub mod state;
pub mod transitions;

pub use events::*;
pub use plugin::RsmPlugin;
// Scaffold API consumed by downstream stories.
#[allow(unused_imports)]
pub use state::{GameOverRequest, PhaseAdvanceRequest, RoundPhase, RoundState};
// Scaffold API consumed by downstream stories.
#[allow(unused_imports)]
pub use transitions::{advance_phase, is_auction_round};
