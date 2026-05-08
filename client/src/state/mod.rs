// State layer: client-side read-only view of server state

use bevy::prelude::*;
use shared::protocol::{
    PlacementTimerMultiplier, RoundPhase, S2CGameSnapshot, S2CHandshake, S2CPhaseChanged,
    S2CSessionSettingsUpdated, SessionToken,
};
use shared::session::PlayerId;

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

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionSettingsView {
    pub placement_timer_multiplier_effective: PlacementTimerMultiplier,
}

impl Default for SessionSettingsView {
    fn default() -> Self {
        Self {
            placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        }
    }
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

/// Server-confirmed identity assigned by the fresh hello/reconnect handshake.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientSessionIdentity {
    pub player_id: Option<PlayerId>,
    pub session_id: Option<u64>,
    pub session_token: Option<SessionToken>,
}

pub fn apply_handshake_message(msg: &S2CHandshake, identity: &mut ClientSessionIdentity) {
    identity.player_id = Some(msg.player_id);
    identity.session_id = Some(msg.session_id);
    identity.session_token = Some(msg.session_token);
}

pub fn should_enter_session_from_phase(
    identity: &ClientSessionIdentity,
    phase: RoundPhase,
) -> bool {
    identity.player_id.is_some() && !matches!(phase, RoundPhase::Handshaking | RoundPhase::Lobby)
}

pub fn should_enter_session_from_snapshot(
    identity: &ClientSessionIdentity,
    snapshot: &S2CGameSnapshot,
) -> bool {
    identity.player_id == Some(snapshot.recipient_player_id)
        && should_enter_session_from_phase(identity, snapshot.phase)
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

pub fn apply_phase_view_message(msg: &S2CPhaseChanged, phase_view: &mut ClientPhaseView) {
    phase_view.phase = msg.phase;
    phase_view.round_number = msg.round_number;
    phase_view.timer_duration_ms = msg.timer_duration_ms;
}

pub fn apply_session_settings_updated_message(
    msg: &S2CSessionSettingsUpdated,
    settings_view: &mut SessionSettingsView,
) {
    settings_view.placement_timer_multiplier_effective = msg.placement_timer_multiplier_effective;
}

pub fn apply_snapshot_to_session_settings_view(
    snapshot: &S2CGameSnapshot,
    settings_view: &mut SessionSettingsView,
) {
    settings_view.placement_timer_multiplier_effective =
        snapshot.placement_timer_multiplier_effective;
}

/// Applies the phase and round from a reconnect snapshot to `CurrentClientPhase`.
///
/// Called by `game_snapshot_sink_system` so that `CurrentClientPhase` is always
/// written by the presentation layer's sink systems only, never by sub-plugins
/// (ADR-021 R5). `S2CGameSnapshot` carries no `timer_duration_ms`; timer-bearing
/// resources (`ClientPhaseView`) are not updated here.
pub fn apply_snapshot_to_current_phase(
    snapshot: &S2CGameSnapshot,
    current: &mut CurrentClientPhase,
) {
    current.phase = snapshot.phase;
    current.round = snapshot.round_number;
}

#[derive(Message, Debug, Clone)]
pub struct ClientGameSnapshotMessage(pub S2CGameSnapshot);
