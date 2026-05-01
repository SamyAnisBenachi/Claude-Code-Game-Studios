// State layer: client-side read-only view of server state

use bevy::prelude::*;
use shared::protocol::RoundPhase;

/// Client presentation lifecycle. It gates session-scoped UI pools.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientState {
    #[default]
    Lobby,
    InSession,
}

/// Updated only by S2CPhaseChanged handler - never drives server transitions.
#[derive(Resource, Default)]
pub struct ClientPhaseView {
    pub phase: RoundPhase,
    pub round_number: u32,
    pub timer_duration_ms: u32,
}
