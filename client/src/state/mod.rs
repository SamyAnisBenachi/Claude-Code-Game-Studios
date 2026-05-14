// State layer: client-side read-only view of server state

use bevy::prelude::*;
use shared::protocol::{
    PlacementTimerMultiplier, RoundPhase, S2CGameSnapshot, S2CHandshake, S2CObjectiveIdentities,
    S2COpponentDisconnected, S2COpponentReconnected, S2CPhaseChanged, S2CPrismRespawned,
    S2CPrismRewardDropped, S2CSessionCancelled, S2CSessionSettingsUpdated, SessionCancelledReason,
    SessionToken,
};
use shared::session::PlayerId;

pub mod idempotency;
pub use idempotency::{
    reset_client_idempotency_on_session_exit_system, ClassLockedDedupeKey, ClientIdempotencyPlugin,
    ClientIdempotencyState, DedupeRing, GameOverDedupeKey, PlacementRevealDedupeKey, DEDUPE_BOUND,
};

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
    tracing::info!(
        player_id = ?msg.player_id,
        session_id = msg.session_id,
        "client_apply_handshake",
    );
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
    tracing::info!(
        from = ?current.phase,
        to = ?phase,
        round = round_number,
        "client_apply_phase_changed",
    );
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

#[derive(Message, Debug, Clone)]
pub struct ClientGameSnapshotMessage(pub S2CGameSnapshot);

/// Local player's objective identities, unicast at DRAFT_INITIAL (ADR-001).
/// Each entry pairs a lane id with `is_fake` (true = fake objective for the
/// local player). Empty until the unicast lands.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct ClientObjectiveIdentities {
    pub identities: Vec<(u8, bool)>,
}

pub fn apply_objective_identities_message(
    msg: &S2CObjectiveIdentities,
    identities: &mut ClientObjectiveIdentities,
) {
    let count = msg.identities.len();
    let fakes = msg
        .identities
        .iter()
        .filter(|(_, is_fake)| *is_fake)
        .count();
    tracing::info!(count, fakes, "client_apply_objective_identities",);
    identities.identities = msg.identities.clone();
}

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct OpponentConnectionView {
    pub disconnected: Option<OpponentDisconnectIndicator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpponentDisconnectIndicator {
    pub player_id: PlayerId,
    pub grace_remaining_ms: u32,
}

pub fn apply_opponent_disconnected_message(
    msg: &S2COpponentDisconnected,
    view: &mut OpponentConnectionView,
) {
    view.disconnected = Some(OpponentDisconnectIndicator {
        player_id: msg.player_id,
        grace_remaining_ms: msg.grace_remaining_ms,
    });
}

pub fn apply_opponent_reconnected_message(
    _msg: &S2COpponentReconnected,
    view: &mut OpponentConnectionView,
) {
    view.disconnected = None;
}

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct PrismLifecycleView {
    pub last_respawn: Option<PrismRespawnEvent>,
    pub pending_rewards_lost: Vec<PrismRewardDroppedEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrismRespawnEvent {
    pub player_id: PlayerId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrismRewardDroppedEvent {
    pub player_id: PlayerId,
    pub lane: u8,
}

pub fn apply_prism_respawned_message(msg: &S2CPrismRespawned, view: &mut PrismLifecycleView) {
    view.last_respawn = Some(PrismRespawnEvent {
        player_id: msg.player_id,
    });
}

pub fn apply_prism_reward_dropped_message(
    msg: &S2CPrismRewardDropped,
    view: &mut PrismLifecycleView,
) {
    view.pending_rewards_lost.push(PrismRewardDroppedEvent {
        player_id: msg.player_id,
        lane: msg.lane,
    });
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SessionLifecycleView {
    pub cancellation: Option<SessionCancelledReason>,
}

pub fn apply_session_cancelled_message(msg: &S2CSessionCancelled, view: &mut SessionLifecycleView) {
    view.cancellation = Some(msg.reason);
}
