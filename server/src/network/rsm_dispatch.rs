use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, PeerId, Server, ServerMultiMessageSender};
use shared::protocol::{ReliableChannel, S2COpponentDisconnected, S2CPhaseChanged};
use shared::session::PlayerId;

use crate::core::rsm::{
    BroadcastPhaseChanged, OpponentDisconnectNotice, RoundPhase, RsmNetworkOutbox,
};
use crate::core::session::{PlayerConnectionMap, SessionConfig};

/// Sends one reliable `S2CPhaseChanged` broadcast for each RSM phase-change message.
pub fn dispatch_phase_changed(
    mut phase_events: MessageReader<BroadcastPhaseChanged>,
    mut outbox: Option<ResMut<RsmNetworkOutbox>>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
) {
    let server = server.single().ok();

    for event in phase_events.read() {
        let message = S2CPhaseChanged {
            phase: protocol_round_phase(event.phase),
            round_number: event.round,
            timer_duration_ms: event.timer_ms,
        };

        if let Some(outbox) = outbox.as_deref_mut() {
            outbox.push_phase_changed(message.clone());
        }

        if let (Some(server), Some(sender)) = (server, sender.as_mut()) {
            if let Err(e) = sender.send::<S2CPhaseChanged, ReliableChannel>(
                &message,
                server,
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
    }
}

/// Sends one reliable `S2COpponentDisconnected` per RSM-emitted
/// `OpponentDisconnectNotice`, unicast to every surviving session player
/// (i.e. every session-occupying player other than the disconnected one).
/// PROMPT 1211 repair: the protocol message existed and the client drain
/// existed, but the server had no send site for it.
pub fn dispatch_opponent_disconnected(
    mut notices: MessageReader<OpponentDisconnectNotice>,
    session: Option<Res<SessionConfig>>,
    connections: Option<Res<PlayerConnectionMap>>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
) {
    let server_handle = server.single().ok();

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

        let (Some(server_handle), Some(sender)) = (server_handle, sender.as_mut()) else {
            // Runtime not wired (test/headless without lightyear); the notice
            // has still been observed and is testable via MessageReader on
            // the rsm-side. Skip the actual send.
            continue;
        };

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
