// State layer: client-side read-only view of server state

use bevy::prelude::*;
use shared::protocol::RoundPhase;

/// Updated only by S2CPhaseChanged handler - never drives server transitions.
#[derive(Resource, Default)]
pub struct ClientPhaseView {
    pub phase: RoundPhase,
    pub round_number: u32,
    pub timer_duration_ms: u32,
}
