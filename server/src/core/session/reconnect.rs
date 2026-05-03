use std::collections::HashMap;

use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use lightyear::prelude::{
    Connected, Disconnect, MessageReceiver, NetworkTarget, PeerId, RemoteId, Server,
    ServerMultiMessageSender,
};
use shared::protocol::{
    C2SHello, ObjectiveReveal, ReliableChannel, S2CAuctionBidAccepted, S2CAuctionBidRejected,
    S2CCardAcquired, S2CDraftOffering, S2CGameOver, S2CGameSnapshot, S2CGoldUpdate, S2CHandshake,
    S2CHandshakeRejected, S2CObjectiveIdentities, S2COpponentReconnected, S2CPhaseChanged,
    S2CPrismRewardDropped, S2CSangMepriseReveal, S2CSessionCancelled, S2CShopSlots,
};
use shared::session::PlayerId;
use uuid::Uuid;

use crate::core::rsm::{PlayerReconnected, RoundPhase};
use crate::core::session::{
    build_game_snapshot, ActiveSessions, DeferredMessage, PendingHello, PlayerConnectionMap,
    ReconnectNetworkOutbox, ReconnectTracker, SessionConfig, SessionId, SessionSlot, SessionSlots,
    SessionToken,
};
use crate::feature::objective::{HiddenObjectives, OBJECTIVE_LANE_COUNT};
use crate::foundation::config::GameConfig;

const RECONNECT_REJECTION_REASON: &str = "reconnect rejected";
const RECONNECT_TIMEOUT_REASON: &str = "hello timeout";

#[derive(Debug, Clone)]
pub enum ReconnectDispatch {
    Handshake {
        peer_id: PeerId,
        message: S2CHandshake,
    },
    GameSnapshot {
        peer_id: PeerId,
        message: S2CGameSnapshot,
    },
    ObjectiveIdentities {
        peer_id: PeerId,
        message: S2CObjectiveIdentities,
    },
    PhaseChanged {
        peer_id: PeerId,
        message: S2CPhaseChanged,
    },
    SangMepriseReveal {
        peer_id: PeerId,
        message: S2CSangMepriseReveal,
    },
    OpponentReconnected {
        recipients: Vec<PeerId>,
        message: S2COpponentReconnected,
    },
    HandshakeRejected {
        peer_id: PeerId,
        message: S2CHandshakeRejected,
    },
    Deferred {
        peer_id: PeerId,
        message: DeferredMessage,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconnectClose {
    pub entity: Entity,
    pub peer_id: PeerId,
    pub reason: String,
}

#[derive(Debug, Default, Clone)]
pub struct ReconnectProcessResult {
    pub dispatches: Vec<ReconnectDispatch>,
    pub closes: Vec<ReconnectClose>,
}

#[allow(clippy::too_many_arguments)]
pub fn on_reconnect_connected(
    trigger: On<Add, Connected>,
    clients: Query<&RemoteId>,
    config: Option<Res<GameConfig>>,
    mut tracker: ResMut<ReconnectTracker>,
) {
    let Ok(remote) = clients.get(trigger.entity) else {
        return;
    };

    tracker.pending_hellos.insert(
        remote.0,
        PendingHello {
            entity: trigger.entity,
            remaining_ms: hello_timeout_ms(config.as_deref()),
        },
    );
}

pub fn initialise_reconnect_tracker(
    session: &SessionConfig,
    slots: Option<&SessionSlots>,
    active_sessions: Option<&ActiveSessions>,
) -> ReconnectTracker {
    let mut tracker = ReconnectTracker::default();
    for player in session.players() {
        tracker.snapshot_sent.insert(player, true);
        tracker.deferred_queue.entry(player).or_default();
        let session_id = session_id_for_player(player, slots, active_sessions);
        let token = token_for_player(session_id, player);
        tracker.token_map.insert(token, (session_id, player));
    }
    tracker
}

/// Sole drainer for `MessageReceiver<C2SHello>`.
pub fn handle_reconnect(world: &mut World) {
    let hellos = drain_hello_messages(world);
    if hellos.is_empty() {
        return;
    }

    let mut frame_result = ReconnectProcessResult::default();
    for hello in hellos {
        let result = process_reconnect_hello(world, hello.entity, hello.peer_id, hello.message);
        frame_result.dispatches.extend(result.dispatches);
        frame_result.closes.extend(result.closes);
    }

    log_reconnect_result(world, &frame_result);
    send_reconnect_dispatches(world, &frame_result.dispatches);
    close_reconnect_peers(world, &frame_result.closes);
}

pub fn process_reconnect_hello(
    world: &mut World,
    connection_entity: Entity,
    peer_id: PeerId,
    hello: C2SHello,
) -> ReconnectProcessResult {
    clear_pending_hello(world, peer_id);

    let Some(token) = hello.session_token else {
        return ReconnectProcessResult::default();
    };

    let Some((session_id, player_id)) = token_lookup(world, &token) else {
        let message = S2CHandshakeRejected {
            server_version: protocol_version(world),
            client_version: hello.protocol_version,
        };
        return ReconnectProcessResult {
            dispatches: vec![ReconnectDispatch::HandshakeRejected { peer_id, message }],
            closes: vec![ReconnectClose {
                entity: connection_entity,
                peer_id,
                reason: RECONNECT_REJECTION_REASON.to_string(),
            }],
        };
    };

    map_reconnect_peer(world, peer_id, player_id);
    set_snapshot_sent(world, player_id, false);

    let Some(snapshot) = build_game_snapshot(player_id, world) else {
        let message = S2CHandshakeRejected {
            server_version: protocol_version(world),
            client_version: hello.protocol_version,
        };
        return ReconnectProcessResult {
            dispatches: vec![ReconnectDispatch::HandshakeRejected { peer_id, message }],
            closes: vec![ReconnectClose {
                entity: connection_entity,
                peer_id,
                reason: RECONNECT_REJECTION_REASON.to_string(),
            }],
        };
    };

    let objective_identities = objective_identities_for_player(world, player_id);
    let phase_changed = phase_changed_from_snapshot(&snapshot);
    let sang_meprise = sang_meprise_reveal_message(world, player_id);
    let opponent_recipients = opponent_reconnected_recipients(world, player_id);

    let mut dispatches = vec![
        ReconnectDispatch::Handshake {
            peer_id,
            message: S2CHandshake {
                protocol_version: protocol_version(world),
                session_id: session_id_to_u64(session_id),
                session_token: token,
            },
        },
        ReconnectDispatch::GameSnapshot {
            peer_id,
            message: snapshot,
        },
        ReconnectDispatch::ObjectiveIdentities {
            peer_id,
            message: objective_identities,
        },
        ReconnectDispatch::PhaseChanged {
            peer_id,
            message: phase_changed,
        },
    ];

    if let Some(message) = sang_meprise {
        dispatches.push(ReconnectDispatch::SangMepriseReveal { peer_id, message });
    }

    set_snapshot_sent(world, player_id, true);

    if !opponent_recipients.is_empty() {
        dispatches.push(ReconnectDispatch::OpponentReconnected {
            recipients: opponent_recipients,
            message: S2COpponentReconnected { player_id },
        });
    }

    if let Some(mut messages) = world.get_resource_mut::<Messages<PlayerReconnected>>() {
        messages.write(PlayerReconnected { player: player_id });
    }

    ReconnectProcessResult {
        dispatches,
        closes: Vec::new(),
    }
}

pub fn hello_timeout_watchdog(world: &mut World) {
    let delta_ms = elapsed_millis(world.resource::<Time>().delta());
    let mut closes = Vec::new();

    if let Some(mut tracker) = world.get_resource_mut::<ReconnectTracker>() {
        let mut expired = Vec::new();
        for (peer_id, pending) in tracker.pending_hellos.iter_mut() {
            let before = pending.remaining_ms;
            pending.remaining_ms = pending.remaining_ms.saturating_sub(delta_ms);
            if delta_ms > before || pending.remaining_ms == 0 {
                expired.push(*peer_id);
            }
        }

        for peer_id in expired {
            if let Some(pending) = tracker.pending_hellos.remove(&peer_id) {
                closes.push(ReconnectClose {
                    entity: pending.entity,
                    peer_id,
                    reason: RECONNECT_TIMEOUT_REASON.to_string(),
                });
            }
        }
    }

    if closes.is_empty() {
        return;
    }

    if let Some(mut outbox) = world.get_resource_mut::<ReconnectNetworkOutbox>() {
        outbox.extend_closes(closes.iter().cloned());
    }
    close_reconnect_peers(world, &closes);
}

pub fn flush_deferred_queue(world: &mut World) {
    let dispatches = drain_deferred_dispatches(world);
    if dispatches.is_empty() {
        return;
    }

    if let Some(mut outbox) = world.get_resource_mut::<ReconnectNetworkOutbox>() {
        outbox.extend_dispatches(dispatches.iter().cloned());
    }
    send_reconnect_dispatches(world, &dispatches);
}

pub fn defer_unicast_for_reconnect(
    tracker: Option<&mut ReconnectTracker>,
    player_id: PlayerId,
    message: DeferredMessage,
) -> bool {
    let Some(tracker) = tracker else {
        return false;
    };

    if tracker
        .snapshot_sent
        .get(&player_id)
        .copied()
        .unwrap_or(false)
    {
        return false;
    }

    tracker
        .deferred_queue
        .entry(player_id)
        .or_default()
        .push(message);
    true
}

fn drain_hello_messages(world: &mut World) -> Vec<InboundHello> {
    let mut system_state: SystemState<Query<(Entity, &RemoteId, &mut MessageReceiver<C2SHello>)>> =
        SystemState::new(world);
    let mut receivers = system_state.get_mut(world);
    let mut hellos = Vec::new();

    for (entity, remote, mut receiver) in receivers.iter_mut() {
        for message in receiver.receive() {
            hellos.push(InboundHello {
                entity,
                peer_id: remote.0,
                message: message.clone(),
            });
        }
    }

    hellos
}

#[derive(Clone)]
struct InboundHello {
    entity: Entity,
    peer_id: PeerId,
    message: C2SHello,
}

fn log_reconnect_result(world: &mut World, result: &ReconnectProcessResult) {
    let Some(mut outbox) = world.get_resource_mut::<ReconnectNetworkOutbox>() else {
        return;
    };
    outbox.extend_dispatches(result.dispatches.iter().cloned());
    outbox.extend_closes(result.closes.iter().cloned());
}

fn send_reconnect_dispatches(world: &mut World, dispatches: &[ReconnectDispatch]) {
    let mut system_state: SystemState<(Query<&Server>, Option<ServerMultiMessageSender>)> =
        SystemState::new(world);
    let (server, mut sender) = system_state.get_mut(world);
    let (Ok(server), Some(sender)) = (server.single(), sender.as_mut()) else {
        return;
    };

    for dispatch in dispatches {
        match dispatch {
            ReconnectDispatch::Handshake { peer_id, message } => {
                let _ = sender.send::<S2CHandshake, ReliableChannel>(
                    message,
                    server,
                    &single(*peer_id),
                );
            }
            ReconnectDispatch::GameSnapshot { peer_id, message } => {
                let _ = sender.send::<S2CGameSnapshot, ReliableChannel>(
                    message,
                    server,
                    &single(*peer_id),
                );
            }
            ReconnectDispatch::ObjectiveIdentities { peer_id, message } => {
                let _ = sender.send::<S2CObjectiveIdentities, ReliableChannel>(
                    message,
                    server,
                    &single(*peer_id),
                );
            }
            ReconnectDispatch::PhaseChanged { peer_id, message } => {
                let _ = sender.send::<S2CPhaseChanged, ReliableChannel>(
                    message,
                    server,
                    &single(*peer_id),
                );
            }
            ReconnectDispatch::SangMepriseReveal { peer_id, message } => {
                let _ = sender.send::<S2CSangMepriseReveal, ReliableChannel>(
                    message,
                    server,
                    &single(*peer_id),
                );
            }
            ReconnectDispatch::OpponentReconnected {
                recipients,
                message,
            } => {
                let _ = sender.send::<S2COpponentReconnected, ReliableChannel>(
                    message,
                    server,
                    &NetworkTarget::Only(recipients.clone()),
                );
            }
            ReconnectDispatch::HandshakeRejected { peer_id, message } => {
                let _ = sender.send::<S2CHandshakeRejected, ReliableChannel>(
                    message,
                    server,
                    &single(*peer_id),
                );
            }
            ReconnectDispatch::Deferred { peer_id, message } => {
                send_deferred_message(sender, server, *peer_id, message);
            }
        }
    }
}

fn send_deferred_message(
    sender: &mut ServerMultiMessageSender,
    server: &Server,
    peer_id: PeerId,
    message: &DeferredMessage,
) {
    match message {
        DeferredMessage::GameOver(message) => {
            let _ = sender.send::<S2CGameOver, ReliableChannel>(message, server, &single(peer_id));
        }
        DeferredMessage::SessionCancelled(message) => {
            let _ = sender.send::<S2CSessionCancelled, ReliableChannel>(
                message,
                server,
                &single(peer_id),
            );
        }
        DeferredMessage::GoldUpdate(message) => {
            let _ =
                sender.send::<S2CGoldUpdate, ReliableChannel>(message, server, &single(peer_id));
        }
        DeferredMessage::ObjectiveIdentities(message) => {
            let _ = sender.send::<S2CObjectiveIdentities, ReliableChannel>(
                message,
                server,
                &single(peer_id),
            );
        }
        DeferredMessage::DraftOffering(message) => {
            let _ =
                sender.send::<S2CDraftOffering, ReliableChannel>(message, server, &single(peer_id));
        }
        DeferredMessage::ShopSlots(message) => {
            let _ = sender.send::<S2CShopSlots, ReliableChannel>(message, server, &single(peer_id));
        }
        DeferredMessage::AuctionBidRejected(message) => {
            let _ = sender.send::<S2CAuctionBidRejected, ReliableChannel>(
                message,
                server,
                &single(peer_id),
            );
        }
        DeferredMessage::AuctionBidAccepted(message) => {
            let _ = sender.send::<S2CAuctionBidAccepted, ReliableChannel>(
                message,
                server,
                &single(peer_id),
            );
        }
        DeferredMessage::CardAcquiredMessage(message) => {
            let _ =
                sender.send::<S2CCardAcquired, ReliableChannel>(message, server, &single(peer_id));
        }
        DeferredMessage::PrismRewardDroppedMessage(message) => {
            let _ = sender.send::<S2CPrismRewardDropped, ReliableChannel>(
                message,
                server,
                &single(peer_id),
            );
        }
        DeferredMessage::CardAcquired { card_id, source } => {
            let message = S2CCardAcquired {
                card_id: *card_id,
                source: *source,
            };
            let _ =
                sender.send::<S2CCardAcquired, ReliableChannel>(&message, server, &single(peer_id));
        }
        DeferredMessage::PrismRewardDropped { player_id, lane } => {
            let message = S2CPrismRewardDropped {
                player_id: *player_id,
                lane: *lane,
            };
            let _ = sender.send::<S2CPrismRewardDropped, ReliableChannel>(
                &message,
                server,
                &single(peer_id),
            );
        }
    }
}

fn close_reconnect_peers(world: &mut World, closes: &[ReconnectClose]) {
    for close in closes {
        world.trigger(Disconnect {
            entity: close.entity,
        });
    }
}

fn drain_deferred_dispatches(world: &mut World) -> Vec<ReconnectDispatch> {
    let peers_by_player = peers_by_player(world);
    let Some(mut tracker) = world.get_resource_mut::<ReconnectTracker>() else {
        return Vec::new();
    };

    let ready_players = tracker
        .snapshot_sent
        .iter()
        .filter_map(|(player, sent)| (*sent).then_some(*player))
        .collect::<Vec<_>>();

    let mut dispatches = Vec::new();
    for player in ready_players {
        let Some(peer_id) = peers_by_player.get(&player).copied() else {
            continue;
        };
        let Some(queue) = tracker.deferred_queue.get_mut(&player) else {
            continue;
        };
        for message in queue.drain(..) {
            dispatches.push(ReconnectDispatch::Deferred { peer_id, message });
        }
    }

    dispatches
}

fn clear_pending_hello(world: &mut World, peer_id: PeerId) {
    if let Some(mut tracker) = world.get_resource_mut::<ReconnectTracker>() {
        tracker.pending_hellos.remove(&peer_id);
    }
}

fn token_lookup(world: &World, token: &SessionToken) -> Option<(SessionId, PlayerId)> {
    world
        .get_resource::<ReconnectTracker>()?
        .token_map
        .get(token)
        .copied()
}

fn map_reconnect_peer(world: &mut World, peer_id: PeerId, player_id: PlayerId) {
    let Some(mut connections) = world.get_resource_mut::<PlayerConnectionMap>() else {
        return;
    };

    connections.0.retain(|existing_peer, mapped_player| {
        *existing_peer == peer_id || *mapped_player != player_id
    });
    connections.0.insert(peer_id, player_id);
}

fn set_snapshot_sent(world: &mut World, player_id: PlayerId, sent: bool) {
    if let Some(mut tracker) = world.get_resource_mut::<ReconnectTracker>() {
        tracker.snapshot_sent.insert(player_id, sent);
    }
}

fn objective_identities_for_player(world: &World, player_id: PlayerId) -> S2CObjectiveIdentities {
    let mut identities = world
        .get_resource::<HiddenObjectives>()
        .map(|hidden| {
            hidden
                .identities
                .iter()
                .filter_map(|((owner, lane), is_fake)| {
                    (*owner == player_id).then_some((*lane, *is_fake))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    identities.sort_by_key(|(lane, _)| *lane);
    S2CObjectiveIdentities { identities }
}

fn sang_meprise_reveal_message(world: &World, recipient: PlayerId) -> Option<S2CSangMepriseReveal> {
    let tracker = world.get_resource::<ReconnectTracker>()?;
    if !tracker.sang_meprise_sent_to.contains(&recipient) {
        return None;
    }

    let identities = active_sang_meprise_pairs(world, recipient);
    (!identities.is_empty()).then_some(S2CSangMepriseReveal { identities })
}

pub fn active_sang_meprise_reveals(
    world: &World,
    recipient: PlayerId,
) -> Option<Vec<ObjectiveReveal>> {
    let tracker = world.get_resource::<ReconnectTracker>()?;
    if !tracker.sang_meprise_sent_to.contains(&recipient) {
        return None;
    }

    let mut reveals = Vec::new();
    let hidden = world.get_resource::<HiddenObjectives>()?;
    for opponent in session_opponents(world, recipient) {
        for lane in 1..=OBJECTIVE_LANE_COUNT {
            if let Some(is_fake) = hidden.identities.get(&(opponent, lane)).copied() {
                reveals.push(ObjectiveReveal {
                    player_id: opponent,
                    lane,
                    is_fake,
                });
            }
        }
    }
    reveals.sort_by_key(|reveal| (reveal.player_id.0, reveal.lane));
    (!reveals.is_empty()).then_some(reveals)
}

fn active_sang_meprise_pairs(world: &World, recipient: PlayerId) -> Vec<(u8, bool)> {
    active_sang_meprise_reveals(world, recipient)
        .unwrap_or_default()
        .into_iter()
        .map(|reveal| (reveal.lane, reveal.is_fake))
        .collect()
}

fn phase_changed_from_snapshot(snapshot: &S2CGameSnapshot) -> S2CPhaseChanged {
    S2CPhaseChanged {
        phase: snapshot.phase,
        round_number: snapshot.round_number,
        timer_duration_ms: snapshot.timer_remaining_ms.unwrap_or(0),
    }
}

fn opponent_reconnected_recipients(world: &World, reconnecting_player: PlayerId) -> Vec<PeerId> {
    let Some(connections) = world.get_resource::<PlayerConnectionMap>() else {
        return Vec::new();
    };
    let players = session_players_from_world(world);
    let mut peers = connections
        .0
        .iter()
        .filter_map(|(peer, player)| {
            (*player != reconnecting_player && players.contains(player)).then_some(*peer)
        })
        .collect::<Vec<_>>();
    peers.sort_by_key(|peer| format!("{peer:?}"));
    peers
}

fn peers_by_player(world: &World) -> HashMap<PlayerId, PeerId> {
    world
        .get_resource::<PlayerConnectionMap>()
        .map(|connections| {
            connections
                .0
                .iter()
                .map(|(peer, player)| (*player, *peer))
                .collect()
        })
        .unwrap_or_default()
}

fn session_opponents(world: &World, recipient: PlayerId) -> Vec<PlayerId> {
    session_players_from_world(world)
        .into_iter()
        .filter(|player| *player != recipient)
        .collect()
}

fn session_players_from_world(world: &World) -> Vec<PlayerId> {
    if let Some(session) = world.get_resource::<SessionConfig>() {
        return session.players().collect();
    }

    world
        .get_resource::<SessionSlots>()
        .map(|slots| slots.0.iter().filter_map(|slot| slot.player).collect())
        .unwrap_or_default()
}

fn session_id_for_player(
    player: PlayerId,
    slots: Option<&SessionSlots>,
    active_sessions: Option<&ActiveSessions>,
) -> SessionId {
    if let Some(session_id) = active_sessions.and_then(|sessions| sessions.0.get(&player).copied())
    {
        return session_id;
    }

    let slot_index = slots
        .and_then(|slots| {
            slots
                .0
                .iter()
                .find_map(|slot: &SessionSlot| (slot.player == Some(player)).then_some(slot.index))
        })
        .unwrap_or(0);
    SessionId(Uuid::from_u128(u128::from(slot_index) + 1))
}

fn token_for_player(session_id: SessionId, player: PlayerId) -> SessionToken {
    let mut token = *session_id.0.as_bytes();
    let player_bytes = player.0.to_le_bytes();
    for (index, byte) in player_bytes.iter().enumerate() {
        token[index] ^= *byte;
    }
    token
}

fn protocol_version(world: &World) -> u32 {
    world
        .get_resource::<GameConfig>()
        .map(|config| config.protocol_version)
        .unwrap_or_else(|| shared::config::GameConfig::default().protocol_version)
}

fn hello_timeout_ms(config: Option<&GameConfig>) -> u32 {
    config
        .map(|config| config.hello_timeout_ms)
        .unwrap_or_else(|| shared::config::GameConfig::default().hello_timeout_ms)
}

fn session_id_to_u64(session_id: SessionId) -> u64 {
    let bytes = session_id.0.as_u128().to_le_bytes();
    u64::from_le_bytes(bytes[0..8].try_into().expect("slice length is 8"))
}

fn elapsed_millis(duration: std::time::Duration) -> u32 {
    u32::try_from(duration.as_millis()).unwrap_or(u32::MAX)
}

fn single(peer_id: PeerId) -> NetworkTarget {
    NetworkTarget::Single(peer_id)
}

#[allow(dead_code)]
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
