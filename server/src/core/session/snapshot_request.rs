// Server handler for `C2SRequestSnapshot` (S13-PROTO-ORPHAN-DRAIN-001 Path A).
//
// network-protocol.md Table A `C2SRequestSnapshot`: client-initiated desync
// recovery. Server responds with `S2CGameSnapshot` unicast (same path as
// reconnect). Rate-limited by `GameConfig::snapshot_cooldown_ms` (default
// 5000ms). ADR-002: client message is advisory only — server stays
// authoritative on the snapshot contents and can silently ignore the
// request. ADR-011: reuse the existing `build_game_snapshot` builder; no
// new construction path lands here.

use std::collections::HashMap;

use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use lightyear::prelude::{
    MessageReceiver, NetworkTarget, PeerId, RemoteId, Server, ServerMultiMessageSender,
};
use shared::protocol::{C2SRequestSnapshot, ReliableChannel, S2CGameSnapshot};
use shared::session::PlayerId;

use crate::core::session::{build_game_snapshot, PlayerConnectionMap};
use crate::foundation::config::GameConfig;

const DEFAULT_SNAPSHOT_COOLDOWN_MS: u32 = 5000;

/// Tracks the last time a snapshot was honoured for each player, in `Time`-
/// elapsed milliseconds. The server consults this resource before responding
/// to a `C2SRequestSnapshot` and silently ignores requests inside the
/// cooldown window.
#[derive(Resource, Debug, Default)]
pub struct SnapshotRequestCooldowns {
    last_sent_ms: HashMap<PlayerId, u64>,
}

impl SnapshotRequestCooldowns {
    pub fn last_sent_ms(&self, player: PlayerId) -> Option<u64> {
        self.last_sent_ms.get(&player).copied()
    }

    pub fn record_sent(&mut self, player: PlayerId, now_ms: u64) {
        self.last_sent_ms.insert(player, now_ms);
    }

    pub fn is_within_cooldown(&self, player: PlayerId, now_ms: u64, cooldown_ms: u32) -> bool {
        match self.last_sent_ms.get(&player).copied() {
            Some(prev) => now_ms.saturating_sub(prev) < u64::from(cooldown_ms),
            None => false,
        }
    }
}

/// Sole drainer for `MessageReceiver<C2SRequestSnapshot>`.
///
/// Exclusive system because `build_game_snapshot` is a wide read that needs
/// `&mut World` — the reconnect path uses the same pattern.
pub fn handle_request_snapshot(world: &mut World) {
    let requests = drain_request_snapshot(world);
    if requests.is_empty() {
        return;
    }

    let cooldown_ms = world
        .get_resource::<GameConfig>()
        .map(|config| config.snapshot_cooldown_ms)
        .unwrap_or(DEFAULT_SNAPSHOT_COOLDOWN_MS);
    let now_ms = world
        .get_resource::<Time>()
        .map(|time| time.elapsed().as_millis() as u64)
        .unwrap_or(0);

    let peer_to_player: HashMap<PeerId, PlayerId> = world
        .get_resource::<PlayerConnectionMap>()
        .map(|connections| connections.0.clone())
        .unwrap_or_default();

    let mut honoured: Vec<(PeerId, PlayerId, S2CGameSnapshot)> = Vec::new();
    for request in requests {
        let Some(player_id) = peer_to_player.get(&request.peer_id).copied() else {
            tracing::debug!(
                target: "server::game",
                peer_id = ?request.peer_id,
                "C2SRequestSnapshot discarded because sender is not mapped to a player"
            );
            continue;
        };

        if let Some(cooldowns) = world.get_resource::<SnapshotRequestCooldowns>() {
            if cooldowns.is_within_cooldown(player_id, now_ms, cooldown_ms) {
                tracing::debug!(
                    target: "server::game",
                    peer_id = ?request.peer_id,
                    player_id = player_id.0,
                    cooldown_ms = cooldown_ms,
                    "C2SRequestSnapshot inside cooldown window; silently ignored"
                );
                continue;
            }
        }

        let Some(snapshot) = build_game_snapshot(player_id, world) else {
            tracing::debug!(
                target: "server::game",
                peer_id = ?request.peer_id,
                player_id = player_id.0,
                "C2SRequestSnapshot dropped: build_game_snapshot returned None"
            );
            continue;
        };

        if let Some(mut cooldowns) = world.get_resource_mut::<SnapshotRequestCooldowns>() {
            cooldowns.record_sent(player_id, now_ms);
        }
        honoured.push((request.peer_id, player_id, snapshot));
    }

    if honoured.is_empty() {
        return;
    }

    send_request_snapshot_responses(world, honoured);
}

fn drain_request_snapshot(world: &mut World) -> Vec<InboundRequestSnapshot> {
    let mut system_state: SystemState<
        Query<(&RemoteId, &mut MessageReceiver<C2SRequestSnapshot>)>,
    > = SystemState::new(world);
    let mut receivers = system_state.get_mut(world);
    let mut out = Vec::new();

    for (remote, mut receiver) in receivers.iter_mut() {
        for _msg in receiver.receive() {
            tracing::info!(
                target: "server::game",
                peer_id = ?remote.0,
                "c2s_request_snapshot: recv"
            );
            out.push(InboundRequestSnapshot { peer_id: remote.0 });
        }
    }

    out
}

fn send_request_snapshot_responses(
    world: &mut World,
    responses: Vec<(PeerId, PlayerId, S2CGameSnapshot)>,
) {
    let mut system_state: SystemState<(Query<&Server>, Option<ServerMultiMessageSender>)> =
        SystemState::new(world);
    let (server_query, mut sender) = system_state.get_mut(world);
    let (Ok(server), Some(sender)) = (server_query.single(), sender.as_mut()) else {
        return;
    };

    for (peer_id, player_id, snapshot) in responses {
        tracing::info!(
            target: "server::game",
            peer_id = ?peer_id,
            player_id = player_id.0,
            round_number = snapshot.round_number,
            phase = ?snapshot.phase,
            "send_request_snapshot: type=S2CGameSnapshot enter"
        );
        if let Err(e) = sender.send::<S2CGameSnapshot, ReliableChannel>(
            &snapshot,
            server,
            &NetworkTarget::Single(peer_id),
        ) {
            tracing::error!(
                target: "server::game",
                peer_id = ?peer_id,
                player_id = player_id.0,
                err = ?e,
                "S2C send failed: type=S2CGameSnapshot, handler=handle_request_snapshot"
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct InboundRequestSnapshot {
    peer_id: PeerId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooldown_blocks_inside_window_and_releases_after_threshold() {
        let mut cooldowns = SnapshotRequestCooldowns::default();
        let player = PlayerId(42);
        let cooldown_ms = 5_000;

        assert!(!cooldowns.is_within_cooldown(player, 0, cooldown_ms));

        cooldowns.record_sent(player, 10_000);

        assert!(cooldowns.is_within_cooldown(player, 10_000, cooldown_ms));
        assert!(cooldowns.is_within_cooldown(player, 14_999, cooldown_ms));
        assert!(!cooldowns.is_within_cooldown(player, 15_000, cooldown_ms));
        assert!(!cooldowns.is_within_cooldown(player, 20_000, cooldown_ms));
    }

    #[test]
    fn record_sent_overwrites_previous_timestamp() {
        let mut cooldowns = SnapshotRequestCooldowns::default();
        let player = PlayerId(7);

        cooldowns.record_sent(player, 1_000);
        assert_eq!(cooldowns.last_sent_ms(player), Some(1_000));

        cooldowns.record_sent(player, 9_500);
        assert_eq!(cooldowns.last_sent_ms(player), Some(9_500));
    }
}
