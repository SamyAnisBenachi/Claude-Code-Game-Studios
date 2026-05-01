// State layer: client-side read-only view of server state

use bevy::prelude::*;
use shared::protocol::{RoundPhase, S2CPhaseChanged};

/// Client presentation lifecycle. It gates session-scoped UI pools.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientState {
    #[default]
    Lobby,
    InSession,
}

/// Timer-bearing client view used by timer/animation presentation systems.
/// This never drives server transitions.
#[derive(Resource, Default)]
pub struct ClientPhaseView {
    pub phase: RoundPhase,
    pub round_number: u32,
    pub timer_duration_ms: u32,
}

/// Canonical phase state for presentation systems.
///
/// Timer data is intentionally not stored here. Timer-bearing UIs own their own
/// display state; the HUD only needs the phase label and round counter.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentClientPhase {
    pub phase: RoundPhase,
    pub round: u32,
}

impl Default for CurrentClientPhase {
    fn default() -> Self {
        Self {
            phase: RoundPhase::Lobby,
            round: 0,
        }
    }
}

pub fn apply_phase_changed_message(msg: S2CPhaseChanged, current: &mut CurrentClientPhase) {
    let S2CPhaseChanged {
        phase,
        round_number,
        ..
    } = msg;
    current.phase = phase;
    current.round = round_number;
}
