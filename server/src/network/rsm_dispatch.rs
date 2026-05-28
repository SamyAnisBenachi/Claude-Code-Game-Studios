use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, PeerId, Server, ServerMultiMessageSender};
use shared::protocol::{ReliableChannel, S2COpponentDisconnected, S2CPhaseChanged};
use shared::session::PlayerId;

use crate::core::rsm::{
    BroadcastPhaseChanged, OpponentDisconnectNotice, RoundPhase, RsmNetworkOutbox,
};
use crate::core::session::{PlayerConnectionMap, SessionConfig};

/// Structured counter of RSM dispatch drops. Existing as a queryable resource
/// makes the "missing sender" failure mode observable (instead of silent) and
/// gives tests a deterministic assertion surface. PROMPT 2043 hardening.
#[derive(Resource, Default, Clone, Debug, PartialEq, Eq)]
pub struct RsmDispatchDiagnostics {
    pub phase_changed_dropped_missing_sender: u64,
    pub phase_changed_dropped_missing_server: u64,
    pub opponent_disconnected_dropped_missing_sender: u64,
    pub opponent_disconnected_dropped_missing_server: u64,
}

/// Classifies whether the lightyear send dependencies are wired up. Pure;
/// extracted so the silent-skip branches in dispatch_* are unit-testable
/// without spinning a full lightyear runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchReadiness {
    Ready,
    MissingSender,
    MissingServer,
    Headless,
}

pub fn classify_dispatch_readiness(
    server_present: bool,
    sender_present: bool,
) -> DispatchReadiness {
    match (server_present, sender_present) {
        (true, true) => DispatchReadiness::Ready,
        (true, false) => DispatchReadiness::MissingSender,
        (false, true) => DispatchReadiness::MissingServer,
        (false, false) => DispatchReadiness::Headless,
    }
}

/// Sends one reliable `S2CPhaseChanged` broadcast for each RSM phase-change message.
///
/// PROMPT 2043: the previous implementation silently skipped the send when
/// `ServerMultiMessageSender` was absent. That made server-side dispatch
/// failures look like client/UI issues (see PROMPT 2030). The path now emits
/// a structured `tracing::warn!` and increments
/// [`RsmDispatchDiagnostics`] so the missing-sender mode is never silent.
pub fn dispatch_phase_changed(
    mut phase_events: MessageReader<BroadcastPhaseChanged>,
    mut outbox: Option<ResMut<RsmNetworkOutbox>>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
    mut diagnostics: Option<ResMut<RsmDispatchDiagnostics>>,
) {
    let server_handle = server.single().ok();
    let readiness = classify_dispatch_readiness(server_handle.is_some(), sender.is_some());

    for event in phase_events.read() {
        let message = S2CPhaseChanged {
            phase: protocol_round_phase(event.phase),
            round_number: event.round,
            timer_duration_ms: event.timer_ms,
        };

        if let Some(outbox) = outbox.as_deref_mut() {
            outbox.push_phase_changed(message.clone());
        }

        match readiness {
            DispatchReadiness::Ready => {
                let (server_handle, sender) = (server_handle.unwrap(), sender.as_mut().unwrap());
                if let Err(e) = sender.send::<S2CPhaseChanged, ReliableChannel>(
                    &message,
                    server_handle,
                    &NetworkTarget::All,
                ) {
                    tracing::error!(
                        target: "server::game",
                        phase = ?event.phase,
                        round = event.round,
                        timer_ms = event.timer_ms,
                        err = ?e,
                        "S2C send failed: type=S2CPhaseChanged, handler=dispatch_phase_changed"
                    );
                }
            }
            DispatchReadiness::MissingSender => {
                if let Some(diagnostics) = diagnostics.as_deref_mut() {
                    diagnostics.phase_changed_dropped_missing_sender =
                        diagnostics.phase_changed_dropped_missing_sender.saturating_add(1);
                }
                tracing::warn!(
                    target: "server::game",
                    phase = ?event.phase,
                    round = event.round,
                    timer_ms = event.timer_ms,
                    readiness = ?readiness,
                    "S2C dropped: type=S2CPhaseChanged, handler=dispatch_phase_changed — \
                     ServerMultiMessageSender absent while Server entity exists. \
                     Likely missing ReplicationSender wiring or plugin order regression."
                );
            }
            DispatchReadiness::MissingServer => {
                if let Some(diagnostics) = diagnostics.as_deref_mut() {
                    diagnostics.phase_changed_dropped_missing_server =
                        diagnostics.phase_changed_dropped_missing_server.saturating_add(1);
                }
                tracing::warn!(
                    target: "server::game",
                    phase = ?event.phase,
                    round = event.round,
                    timer_ms = event.timer_ms,
                    readiness = ?readiness,
                    "S2C dropped: type=S2CPhaseChanged, handler=dispatch_phase_changed — \
                     Server entity missing while ServerMultiMessageSender present. \
                     Lightyear server not started or entity despawned mid-run."
                );
            }
            DispatchReadiness::Headless => {
                // Both absent: expected for headless/test apps that drive the RSM
                // without spinning the lightyear runtime. The outbox (if present)
                // already captured this message for the test side. No counter is
                // incremented; this branch is not a regression.
                tracing::debug!(
                    target: "server::game",
                    phase = ?event.phase,
                    round = event.round,
                    "dispatch_phase_changed: headless (no Server, no sender) — outbox-only path"
                );
            }
        }
    }
}

/// Sends one reliable `S2COpponentDisconnected` per RSM-emitted
/// `OpponentDisconnectNotice`, unicast to every surviving session player
/// (i.e. every session-occupying player other than the disconnected one).
/// PROMPT 1211 repair: the protocol message existed and the client drain
/// existed, but the server had no send site for it.
///
/// PROMPT 2043 hardening: the previous "no runtime" branch silently dropped
/// the notice. We now classify the dispatch readiness and emit a structured
/// `tracing::warn!` + counter when the runtime is partially wired (server
/// present but sender missing, or vice versa), so the silent failure mode
/// is observable in production logs and in tests via
/// [`RsmDispatchDiagnostics`].
pub fn dispatch_opponent_disconnected(
    mut notices: MessageReader<OpponentDisconnectNotice>,
    session: Option<Res<SessionConfig>>,
    connections: Option<Res<PlayerConnectionMap>>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
    mut diagnostics: Option<ResMut<RsmDispatchDiagnostics>>,
) {
    let server_handle = server.single().ok();
    let readiness = classify_dispatch_readiness(server_handle.is_some(), sender.is_some());

    for notice in notices.read() {
        let Some(session) = session.as_deref() else {
            tracing::debug!(
                target: "server::network::disconnect_notice",
                player_id = ?notice.player_id,
                "dispatch_opponent_disconnected: skipped (no SessionConfig)"
            );
            continue;
        };
        let Some(connections) = connections.as_deref() else {
            tracing::debug!(
                target: "server::network::disconnect_notice",
                player_id = ?notice.player_id,
                "dispatch_opponent_disconnected: skipped (no PlayerConnectionMap)"
            );
            continue;
        };

        let recipients = opponent_disconnect_recipients(notice.player_id, session, connections);
        if recipients.is_empty() {
            tracing::debug!(
                target: "server::network::disconnect_notice",
                disconnected_player_id = ?notice.player_id,
                "dispatch_opponent_disconnected: no surviving recipients with mapped PeerId"
            );
            continue;
        }

        let message = S2COpponentDisconnected {
            player_id: notice.player_id,
            grace_remaining_ms: notice.grace_remaining_ms,
        };

        match readiness {
            DispatchReadiness::Ready => {
                let (server_handle, sender) = (server_handle.unwrap(), sender.as_mut().unwrap());
                for peer_id in recipients {
                    if let Err(e) = sender.send::<S2COpponentDisconnected, ReliableChannel>(
                        &message,
                        server_handle,
                        &NetworkTarget::Single(peer_id),
                    ) {
                        tracing::error!(
                            target: "server::network::disconnect_notice",
                            peer_id = ?peer_id,
                            disconnected_player_id = ?notice.player_id,
                            grace_remaining_ms = notice.grace_remaining_ms,
                            err = ?e,
                            "S2C send failed: type=S2COpponentDisconnected, handler=dispatch_opponent_disconnected"
                        );
                    }
                }
            }
            DispatchReadiness::MissingSender => {
                if let Some(diagnostics) = diagnostics.as_deref_mut() {
                    diagnostics.opponent_disconnected_dropped_missing_sender =
                        diagnostics.opponent_disconnected_dropped_missing_sender.saturating_add(1);
                }
                tracing::warn!(
                    target: "server::network::disconnect_notice",
                    disconnected_player_id = ?notice.player_id,
                    grace_remaining_ms = notice.grace_remaining_ms,
                    recipient_count = recipients.len(),
                    readiness = ?readiness,
                    "S2C dropped: type=S2COpponentDisconnected, handler=dispatch_opponent_disconnected \
                     — ServerMultiMessageSender absent while Server entity exists. \
                     Likely missing ReplicationSender wiring or plugin order regression."
                );
            }
            DispatchReadiness::MissingServer => {
                if let Some(diagnostics) = diagnostics.as_deref_mut() {
                    diagnostics.opponent_disconnected_dropped_missing_server =
                        diagnostics.opponent_disconnected_dropped_missing_server.saturating_add(1);
                }
                tracing::warn!(
                    target: "server::network::disconnect_notice",
                    disconnected_player_id = ?notice.player_id,
                    grace_remaining_ms = notice.grace_remaining_ms,
                    recipient_count = recipients.len(),
                    readiness = ?readiness,
                    "S2C dropped: type=S2COpponentDisconnected, handler=dispatch_opponent_disconnected \
                     — Server entity missing while ServerMultiMessageSender present."
                );
            }
            DispatchReadiness::Headless => {
                tracing::debug!(
                    target: "server::network::disconnect_notice",
                    disconnected_player_id = ?notice.player_id,
                    "dispatch_opponent_disconnected: headless (no Server, no sender) — no send"
                );
            }
        }
    }
}

/// Pure helper computing the surviving session players' `PeerId`s for a
/// given `disconnected` player. Exposed for unit testing of the recipient
/// rule (PROMPT 1211: recipient must be the other occupied player(s), not
/// the disconnected one).
pub fn opponent_disconnect_recipients(
    disconnected: PlayerId,
    session: &SessionConfig,
    connections: &PlayerConnectionMap,
) -> Vec<PeerId> {
    session
        .players()
        .filter(|player| *player != disconnected)
        .filter_map(|player| peer_for_player(connections, player))
        .collect()
}

fn peer_for_player(connections: &PlayerConnectionMap, player_id: PlayerId) -> Option<PeerId> {
    connections
        .0
        .iter()
        .find_map(|(peer_id, mapped_player)| (*mapped_player == player_id).then_some(*peer_id))
}

fn protocol_round_phase(phase: RoundPhase) -> shared::protocol::RoundPhase {
    match phase {
        RoundPhase::Lobby => shared::protocol::RoundPhase::Lobby,
        RoundPhase::DraftInitial => shared::protocol::RoundPhase::DraftInitial,
        RoundPhase::DraftAuction => shared::protocol::RoundPhase::DraftAuction,
        RoundPhase::DraftShop => shared::protocol::RoundPhase::DraftShop,
        RoundPhase::Placement => shared::protocol::RoundPhase::Placement,
        RoundPhase::Resolution => shared::protocol::RoundPhase::Resolution,
        RoundPhase::GameOver => shared::protocol::RoundPhase::GameOver,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_dispatch_readiness_ready_when_both_present() {
        // Arrange / Act
        let readiness = classify_dispatch_readiness(true, true);
        // Assert
        assert_eq!(readiness, DispatchReadiness::Ready);
    }

    #[test]
    fn test_classify_dispatch_readiness_missing_sender_flagged() {
        let readiness = classify_dispatch_readiness(true, false);
        assert_eq!(readiness, DispatchReadiness::MissingSender);
    }

    #[test]
    fn test_classify_dispatch_readiness_missing_server_flagged() {
        let readiness = classify_dispatch_readiness(false, true);
        assert_eq!(readiness, DispatchReadiness::MissingServer);
    }

    #[test]
    fn test_classify_dispatch_readiness_headless_when_both_absent() {
        let readiness = classify_dispatch_readiness(false, false);
        assert_eq!(readiness, DispatchReadiness::Headless);
    }
}
