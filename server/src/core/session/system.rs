// server/src/core/session/system.rs -- Room create/join handlers.

use std::collections::HashMap;

use bevy::prelude::*;
use lightyear::prelude::*;
use shared::protocol::{
    self, C2SCreateRoom, C2SJoinRoom, CreateRoomRejectedReason, GameMode, JoinRejectedReason,
    ReliableChannel, S2CCreateRoomRejected, S2CJoinAck, S2CJoinRejected, S2CRoomCreated,
    S2CSlotUpdated,
};
use shared::session::PlayerId;
use uuid::Uuid;

use crate::core::session::{
    ActiveSessions, LobbyDeadline, LobbyHeartbeats, LobbyState, PlayerConnectionMap, RoomCode,
    RoomSession, RoomSessions, SessionId, SessionSlot, SessionSlots,
};
use crate::foundation::config::GameConfig;

pub const ROOM_CODE_LEN: usize = 6;
const ROOM_CODE_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

#[derive(Debug, Clone)]
pub enum CreateRoomOutcome {
    Created(S2CRoomCreated),
    Rejected(S2CCreateRoomRejected),
}

#[derive(Debug, Clone)]
pub enum JoinRoomOutcome {
    Joined {
        ack: S2CJoinAck,
        slot_update: S2CSlotUpdated,
        slot_update_recipients: Vec<PlayerId>,
    },
    Rejected(S2CJoinRejected),
}

/// Sole drainer for `MessageReceiver<C2SCreateRoom>`.
pub fn handle_create_room(
    time: Res<Time>,
    config: Option<Res<GameConfig>>,
    connections: Res<PlayerConnectionMap>,
    mut rooms: ResMut<RoomSessions>,
    mut active_sessions: ResMut<ActiveSessions>,
    mut receivers: Query<(&RemoteId, &mut MessageReceiver<C2SCreateRoom>)>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
) {
    let now = time.elapsed().as_secs_f64();
    let lobby_timeout_seconds = config
        .as_ref()
        .map(|config| config.lobby_timeout_seconds)
        .unwrap_or_else(|| shared::config::GameConfig::default().lobby_timeout_seconds);
    let server = server.single().ok();

    for (remote, mut receiver) in receivers.iter_mut() {
        for msg in receiver.receive() {
            let Some(player_id) = connections.0.get(&remote.0).copied() else {
                continue;
            };

            let session_id = SessionId(Uuid::new_v4());
            let room_code = generate_unique_room_code(&rooms);
            let outcome = create_room(
                &mut rooms,
                &mut active_sessions,
                player_id,
                msg.mode,
                now,
                lobby_timeout_seconds,
                session_id,
                room_code,
            );

            if let (Some(server), Some(sender)) = (server, sender.as_mut()) {
                send_create_room_outcome(sender, server, remote.0, &outcome);
            }
        }
    }
}

/// Sole drainer for `MessageReceiver<C2SJoinRoom>`.
pub fn handle_join_room(
    time: Res<Time>,
    connections: Res<PlayerConnectionMap>,
    mut rooms: ResMut<RoomSessions>,
    mut active_sessions: ResMut<ActiveSessions>,
    mut receivers: Query<(&RemoteId, &mut MessageReceiver<C2SJoinRoom>)>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
) {
    let now = time.elapsed().as_secs_f64();
    let server = server.single().ok();

    for (remote, mut receiver) in receivers.iter_mut() {
        for msg in receiver.receive() {
            let Some(player_id) = connections.0.get(&remote.0).copied() else {
                continue;
            };

            let outcome = join_room(
                &mut rooms,
                &mut active_sessions,
                player_id,
                &msg.room_code,
                msg.requested_slot,
                now,
            );

            if let (Some(server), Some(sender)) = (server, sender.as_mut()) {
                send_join_room_outcome(sender, server, &connections.0, remote.0, &outcome);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn create_room(
    rooms: &mut RoomSessions,
    active_sessions: &mut ActiveSessions,
    player_id: PlayerId,
    mode: GameMode,
    now: f64,
    lobby_timeout_seconds: u32,
    session_id: SessionId,
    room_code: RoomCode,
) -> CreateRoomOutcome {
    if let Some(existing_session_id) = active_sessions.0.get(&player_id).copied() {
        let Some(existing) = rooms.get(existing_session_id) else {
            return CreateRoomOutcome::Rejected(S2CCreateRoomRejected {
                reason: CreateRoomRejectedReason::AlreadyInSession,
            });
        };

        if existing.owner == player_id && existing.state == LobbyState::LobbyWaiting {
            return CreateRoomOutcome::Created(room_created_message(existing));
        }

        return CreateRoomOutcome::Rejected(S2CCreateRoomRejected {
            reason: CreateRoomRejectedReason::AlreadyInSession,
        });
    }

    let slots = initialise_slots(mode, player_id);
    let session = RoomSession {
        session_id,
        room_code: room_code.clone(),
        owner: player_id,
        mode,
        state: LobbyState::LobbyWaiting,
        slots,
        lobby_deadline: LobbyDeadline(now + f64::from(lobby_timeout_seconds)),
        heartbeats: LobbyHeartbeats(HashMap::from([(player_id, now)])),
    };

    active_sessions.0.insert(player_id, session_id);
    let message = room_created_message(&session);
    rooms.insert(session);

    CreateRoomOutcome::Created(message)
}

pub fn join_room(
    rooms: &mut RoomSessions,
    active_sessions: &mut ActiveSessions,
    player_id: PlayerId,
    room_code: &str,
    requested_slot: u8,
    now: f64,
) -> JoinRoomOutcome {
    if active_sessions.0.contains_key(&player_id) {
        return join_rejected(JoinRejectedReason::AlreadyInSession);
    }

    let room_code = normalise_room_code(room_code);
    let Some(session) = rooms.get_mut_by_code(&room_code) else {
        return join_rejected(JoinRejectedReason::RoomNotFound);
    };

    match session.state {
        LobbyState::LobbyWaiting => {}
        LobbyState::GameActive | LobbyState::GameOver => {
            return join_rejected(JoinRejectedReason::SessionInProgress);
        }
        LobbyState::LobbyReady | LobbyState::LobbyCancelled => {
            return join_rejected(JoinRejectedReason::SessionNotJoinable);
        }
    }

    if session.slots.0.iter().all(|slot| slot.player.is_some()) {
        return join_rejected(JoinRejectedReason::SessionFull);
    }

    let slot_update_recipients = occupied_players(&session.slots);

    let Some(slot) = session
        .slots
        .0
        .iter_mut()
        .find(|slot| slot.index == requested_slot)
    else {
        return join_rejected(JoinRejectedReason::InvalidSlot);
    };

    if slot.player.is_some() {
        return join_rejected(JoinRejectedReason::SlotOccupied);
    }

    slot.player = Some(player_id);
    slot.class = None;
    session.heartbeats.0.insert(player_id, now);
    active_sessions.0.insert(player_id, session.session_id);

    let slots = protocol_slots(&session.slots);

    JoinRoomOutcome::Joined {
        ack: S2CJoinAck {
            session_id: session.session_id.0.to_string(),
            mode: session.mode,
            slots: slots.clone(),
        },
        slot_update: S2CSlotUpdated { slots },
        slot_update_recipients,
    }
}

pub fn initialise_slots(mode: GameMode, creator: PlayerId) -> SessionSlots {
    match mode {
        GameMode::OneVOne => SessionSlots(vec![
            SessionSlot {
                index: 0,
                team: 0,
                player: Some(creator),
                class: None,
            },
            SessionSlot {
                index: 1,
                team: 1,
                player: None,
                class: None,
            },
        ]),
    }
}

pub fn protocol_slots(slots: &SessionSlots) -> Vec<protocol::SessionSlot> {
    slots
        .0
        .iter()
        .map(|slot| protocol::SessionSlot {
            slot: slot.index,
            team: slot.team,
            player_id: slot.player,
            class_id: slot.class,
            class_confirmed: slot.class.is_some(),
        })
        .collect()
}

pub fn normalise_room_code(room_code: &str) -> RoomCode {
    RoomCode(room_code.trim().to_ascii_uppercase())
}

pub fn room_code_from_bytes(bytes: &[u8; 16]) -> RoomCode {
    let mut code = String::with_capacity(ROOM_CODE_LEN);
    for byte in bytes.iter().take(ROOM_CODE_LEN) {
        let index = usize::from(*byte) % ROOM_CODE_ALPHABET.len();
        code.push(char::from(ROOM_CODE_ALPHABET[index]));
    }
    RoomCode(code)
}

pub fn generate_unique_room_code(rooms: &RoomSessions) -> RoomCode {
    loop {
        let id = Uuid::new_v4();
        let code = room_code_from_bytes(id.as_bytes());
        if !rooms.contains_room_code(&code) {
            return code;
        }
    }
}

fn room_created_message(session: &RoomSession) -> S2CRoomCreated {
    S2CRoomCreated {
        session_id: session.session_id.0.to_string(),
        room_code: session.room_code.0.clone(),
        mode: session.mode,
        slots: protocol_slots(&session.slots),
    }
}

fn occupied_players(slots: &SessionSlots) -> Vec<PlayerId> {
    slots.0.iter().filter_map(|slot| slot.player).collect()
}

fn join_rejected(reason: JoinRejectedReason) -> JoinRoomOutcome {
    JoinRoomOutcome::Rejected(S2CJoinRejected { reason })
}

fn send_create_room_outcome(
    sender: &mut ServerMultiMessageSender,
    server: &Server,
    peer_id: PeerId,
    outcome: &CreateRoomOutcome,
) {
    match outcome {
        CreateRoomOutcome::Created(msg) => {
            // Lightyear 0.26 unicast is NetworkTarget::Single(PeerId), verified in
            // tests/evidence/lightyear-026-verification.md item 7.
            let _ = sender.send::<S2CRoomCreated, ReliableChannel>(
                msg,
                server,
                &NetworkTarget::Single(peer_id),
            );
        }
        CreateRoomOutcome::Rejected(msg) => {
            let _ = sender.send::<S2CCreateRoomRejected, ReliableChannel>(
                msg,
                server,
                &NetworkTarget::Single(peer_id),
            );
        }
    }
}

fn send_join_room_outcome(
    sender: &mut ServerMultiMessageSender,
    server: &Server,
    connections: &HashMap<PeerId, PlayerId>,
    peer_id: PeerId,
    outcome: &JoinRoomOutcome,
) {
    match outcome {
        JoinRoomOutcome::Joined {
            ack,
            slot_update,
            slot_update_recipients,
        } => {
            let _ = sender.send::<S2CJoinAck, ReliableChannel>(
                ack,
                server,
                &NetworkTarget::Single(peer_id),
            );

            let target_peers = slot_update_recipients
                .iter()
                .filter_map(|player_id| peer_for_player(connections, *player_id))
                .collect::<Vec<_>>();

            if !target_peers.is_empty() {
                let _ = sender.send::<S2CSlotUpdated, ReliableChannel>(
                    slot_update,
                    server,
                    &NetworkTarget::Only(target_peers),
                );
            }
        }
        JoinRoomOutcome::Rejected(msg) => {
            let _ = sender.send::<S2CJoinRejected, ReliableChannel>(
                msg,
                server,
                &NetworkTarget::Single(peer_id),
            );
        }
    }
}

fn peer_for_player(connections: &HashMap<PeerId, PlayerId>, player_id: PlayerId) -> Option<PeerId> {
    connections
        .iter()
        .find_map(|(peer_id, mapped_player)| (*mapped_player == player_id).then_some(*peer_id))
}
