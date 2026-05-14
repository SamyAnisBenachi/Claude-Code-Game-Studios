use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, Server, ServerMultiMessageSender};
use shared::protocol::{ReliableChannel, S2CPhaseChanged};

use crate::core::rsm::{BroadcastPhaseChanged, RoundPhase, RsmNetworkOutbox};

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
