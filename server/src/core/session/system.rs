// server/src/core/session/system.rs -- Room create/join handlers.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use lightyear::prelude::*;
use shared::card::ClassId;
use shared::protocol::{
    self, C2SAcknowledgeResult, C2SConfirmClass, C2SCreateRoom, C2SHeartbeat, C2SJoinRoom,
    C2SSelectClass, C2SSetPlacementTimerMultiplier, ConfirmClassRejectedReason,
    CreateRoomRejectedReason, GameMode, JoinRejectedReason, PlacementTimerMultiplier,
    ReliableChannel, RoundPhase as ProtocolRoundPhase, S2CClassLocked, S2CClassesRevealed,
    S2CConfirmClassRejected, S2CCreateRoomRejected, S2CGameOver, S2CJoinAck, S2CJoinRejected,
    S2CRoomCreated, S2CSessionCancelled, S2CSessionSettingsUpdated, S2CSlotUpdated,
};
use shared::session::PlayerId;
use uuid::Uuid;

use crate::core::rsm::{GameOverEmitted, PlayerHeartbeat, RoundPhase, RoundState};
use crate::core::session::{
    build_game_snapshot, build_session_config_with_settings, effective_placement_timer_multiplier,
    ActiveSessions, ClassPreviews, ClassSelections, EndedSessionResultState, LobbyDeadline,
    LobbyHeartbeats, LobbyState, PlacementTimerMultiplierRequests, PlayerConnectionMap,
    PlayerSessionData, PlayerSessions, ReconnectTracker, RoomCode, RoomSession, RoomSessions,
    SessionCancelledReason, SessionConfig, SessionId, SessionNetworkOutbox, SessionReady,
    SessionSlot, SessionSlots,
};
use crate::foundation::config::GameConfig;
use crate::foundation::rng::ServerRng;

pub const ROOM_CODE_LEN: usize = 6;
const ROOM_CODE_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionSystemSet {
    LobbyEval,
    ReconnectHandshake,
    LiveMessages,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerRngInitError;

#[derive(Resource, Clone, Copy)]
pub struct ServerRngFactory {
    factory: fn() -> Result<ServerRng, ServerRngInitError>,
}

impl ServerRngFactory {
    pub const fn new(factory: fn() -> Result<ServerRng, ServerRngInitError>) -> Self {
        Self { factory }
    }

    pub fn create(self) -> Result<ServerRng, ServerRngInitError> {
        (self.factory)()
    }
}

impl Default for ServerRngFactory {
    fn default() -> Self {
        Self::new(default_server_rng)
    }
}

fn default_server_rng() -> Result<ServerRng, ServerRngInitError> {
    Ok(ServerRng::new())
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectClassOutcome {
    PreviewUpdated,
    Ignored,
}

#[derive(Debug, Clone)]
pub enum ConfirmClassOutcome {
    Locked {
        locked: S2CClassLocked,
        revealed: Option<S2CClassesRevealed>,
        reveal_recipients: Vec<PlayerId>,
    },
    Rejected(S2CConfirmClassRejected),
    Ignored,
}

pub fn evaluate_session_ready(
    mut commands: Commands,
    time: Res<Time>,
    lobby_state: Option<Res<LobbyState>>,
    slots: Option<Res<SessionSlots>>,
    selections: Option<Res<ClassSelections>>,
    deadline: Option<Res<LobbyDeadline>>,
    rng_factory: Option<Res<ServerRngFactory>>,
    placement_timer_requests: Option<Res<PlacementTimerMultiplierRequests>>,
    connections: Option<Res<PlayerConnectionMap>>,
    active_sessions: Option<Res<ActiveSessions>>,
    mut player_sessions: ResMut<PlayerSessions>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
    mut outbox: Option<ResMut<SessionNetworkOutbox>>,
) {
    let (Some(lobby_state), Some(slots), Some(selections), Some(deadline)) =
        (lobby_state, slots, selections, deadline)
    else {
        return;
    };

    if *lobby_state != LobbyState::LobbyWaiting {
        return;
    }

    if !f4_session_ready(&slots, &selections, time.elapsed().as_secs_f64(), *deadline) {
        return;
    }

    let session_config = build_session_config_with_settings(
        &slots,
        &selections,
        placement_timer_requests.as_deref(),
    );
    let rng_factory = rng_factory.as_deref().copied().unwrap_or_default();
    let Ok(server_rng) = rng_factory.create() else {
        commands.insert_resource(LobbyState::LobbyCancelled);
        let recipients = occupied_players(&slots);
        broadcast_session_cancelled(
            &server,
            sender.as_mut(),
            connections.as_deref(),
            outbox.as_deref_mut(),
            SessionCancelledReason::RngInitFailure,
            &recipients,
            None,
        );
        return;
    };

    let reconnect_tracker = crate::core::session::initialise_reconnect_tracker(
        &session_config,
        Some(slots.as_ref()),
        active_sessions.as_deref(),
    );
    populate_player_sessions_from_config(&mut player_sessions, &session_config);
    commands.insert_resource(session_config);
    commands.insert_resource(reconnect_tracker);
    commands.insert_resource(server_rng);
    commands.trigger(SessionReady);
    commands.remove_resource::<LobbyHeartbeats>();
    commands.insert_resource(LobbyState::GameActive);
}

pub fn evaluate_room_session_ready(
    mut commands: Commands,
    time: Res<Time>,
    mut rooms: ResMut<RoomSessions>,
    selections: Option<Res<ClassSelections>>,
    rng_factory: Option<Res<ServerRngFactory>>,
    placement_timer_requests: Option<Res<PlacementTimerMultiplierRequests>>,
    active_sessions: Option<Res<ActiveSessions>>,
    session_config: Option<Res<SessionConfig>>,
    mut player_sessions: ResMut<PlayerSessions>,
) {
    let Some(selections) = selections else {
        return;
    };

    if session_config.is_some() {
        return;
    }

    let now = time.elapsed().as_secs_f64();
    let Some(session_id) = find_room_session_ready(&rooms, &selections, now) else {
        return;
    };

    let Some((slots, deadline)) = rooms
        .get(session_id)
        .map(|session| (session.slots.clone(), session.lobby_deadline))
    else {
        return;
    };

    let rng_factory = rng_factory.as_deref().copied().unwrap_or_default();
    let Ok(server_rng) = rng_factory.create() else {
        if let Some(session) = rooms.get_mut(session_id) {
            session.state = LobbyState::LobbyCancelled;
            session.heartbeats.0.clear();
        }
        commands.insert_resource(LobbyState::LobbyCancelled);
        return;
    };

    let session_config = build_session_config_with_settings(
        &slots,
        &selections,
        placement_timer_requests.as_deref(),
    );
    let reconnect_tracker = crate::core::session::initialise_reconnect_tracker(
        &session_config,
        Some(&slots),
        active_sessions.as_deref(),
    );
    populate_player_sessions_from_config(&mut player_sessions, &session_config);

    if let Some(session) = rooms.get_mut(session_id) {
        session.state = LobbyState::GameActive;
        session.heartbeats.0.clear();
    }

    commands.insert_resource(slots);
    commands.insert_resource(deadline);
    commands.insert_resource(session_config);
    commands.insert_resource(reconnect_tracker);
    commands.insert_resource(server_rng);
    commands.trigger(SessionReady);
    commands.insert_resource(LobbyState::GameActive);
}

fn populate_player_sessions_from_config(
    player_sessions: &mut PlayerSessions,
    session_config: &SessionConfig,
) {
    player_sessions.players.clear();
    for player_id in session_config.players() {
        let class = session_config
            .class_map
            .get(&player_id)
            .copied()
            .unwrap_or(ClassId::Neutral);
        player_sessions.players.insert(
            player_id,
            PlayerSessionData {
                class,
                class_locked: false,
            },
        );
    }
}

fn find_room_session_ready(
    rooms: &RoomSessions,
    selections: &ClassSelections,
    now: f64,
) -> Option<SessionId> {
    rooms.session_ids().into_iter().find(|session_id| {
        rooms
            .get(*session_id)
            .map(|session| {
                session.state == LobbyState::LobbyWaiting
                    && f4_session_ready(&session.slots, selections, now, session.lobby_deadline)
            })
            .unwrap_or(false)
    })
}

/// Sole drainer for `MessageReceiver<C2SSetPlacementTimerMultiplier>`.
#[allow(clippy::too_many_arguments)]
pub fn handle_placement_timer_multiplier_requests(
    lobby_state: Option<Res<LobbyState>>,
    session_config: Option<Res<SessionConfig>>,
    slots: Option<Res<SessionSlots>>,
    connections: Res<PlayerConnectionMap>,
    mut placement_timer_requests: Option<ResMut<PlacementTimerMultiplierRequests>>,
    mut receivers: Query<(
        &RemoteId,
        &mut MessageReceiver<C2SSetPlacementTimerMultiplier>,
    )>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
    mut outbox: Option<ResMut<SessionNetworkOutbox>>,
) {
    let Some(mut placement_timer_requests) = placement_timer_requests.take() else {
        for (_, mut receiver) in receivers.iter_mut() {
            for _ in receiver.receive() {}
        }
        return;
    };

    let mut requests = Vec::new();
    for (remote, mut receiver) in receivers.iter_mut() {
        for msg in receiver.receive() {
            tracing::info!(
                peer_id = ?remote.0,
                multiplier = ?msg.multiplier,
                "c2s_set_placement_timer_multiplier: recv"
            );
            let Some(player_id) = connections.0.get(&remote.0).copied() else {
                continue;
            };
            requests.push((player_id, Some(msg.multiplier)));
        }
    }

    let Some(update) = apply_placement_timer_multiplier_request_batch(
        lobby_state.as_deref(),
        session_config.as_deref(),
        slots.as_deref(),
        &mut placement_timer_requests,
        requests,
    ) else {
        return;
    };

    let recipients = slots.as_deref().map(occupied_players).unwrap_or_default();
    broadcast_session_settings_updated(
        &server,
        sender.as_mut(),
        Some(&connections),
        outbox.as_deref_mut(),
        update,
        &recipients,
    );
}

pub fn apply_placement_timer_multiplier_request_batch(
    lobby_state: Option<&LobbyState>,
    session_config: Option<&SessionConfig>,
    slots: Option<&SessionSlots>,
    placement_timer_requests: &mut PlacementTimerMultiplierRequests,
    requests: impl IntoIterator<Item = (PlayerId, Option<PlacementTimerMultiplier>)>,
) -> Option<S2CSessionSettingsUpdated> {
    if lobby_state != Some(&LobbyState::LobbyWaiting) || session_config.is_some() {
        return None;
    }

    let slots = slots?;
    let before = effective_placement_timer_multiplier(slots, Some(placement_timer_requests));
    let mut accepted_any = false;

    for (player_id, multiplier) in requests {
        if !session_has_player(slots, player_id) {
            continue;
        }

        let Some(multiplier) = multiplier else {
            continue;
        };

        accepted_any = true;
        if multiplier == PlacementTimerMultiplier::X1 {
            placement_timer_requests.0.remove(&player_id);
        } else {
            placement_timer_requests.0.insert(player_id, multiplier);
        }
    }

    if !accepted_any {
        return None;
    }

    let after = effective_placement_timer_multiplier(slots, Some(placement_timer_requests));
    (after != before).then_some(S2CSessionSettingsUpdated {
        placement_timer_multiplier_effective: after,
    })
}

pub fn f4_session_ready(
    slots: &SessionSlots,
    selections: &ClassSelections,
    now: f64,
    lobby_deadline: LobbyDeadline,
) -> bool {
    all_slots_filled(slots) && all_classes_confirmed(slots, selections) && now <= lobby_deadline.0
}

pub fn all_slots_filled(slots: &SessionSlots) -> bool {
    !slots.0.is_empty() && slots.0.iter().all(|slot| slot.player.is_some())
}

pub fn all_classes_confirmed(slots: &SessionSlots, selections: &ClassSelections) -> bool {
    !slots.0.is_empty()
        && slots.0.iter().all(|slot| match (slot.player, slot.class) {
            (Some(player), Some(class_id)) => selections.0.get(&player) == Some(&class_id),
            _ => false,
        })
}

pub fn handle_game_over_teardown(world: &mut World) {
    if matches!(
        world.get_resource::<LobbyState>().copied(),
        Some(LobbyState::GameOver)
    ) {
        return;
    }

    let Some(game_over) = first_game_over_message(world) else {
        return;
    };

    let players = session_players(
        world.get_resource::<SessionConfig>(),
        world.get_resource::<SessionSlots>(),
    );
    let message = S2CGameOver {
        loser: game_over.loser,
        round: game_over.round,
        reason: game_over.reason,
    };
    let session_ids = session_ids_for_players_from_world(world, &players);
    let ended_state =
        build_ended_session_result_state(world, message.clone(), &players, &session_ids);

    broadcast_game_over_from_world(world, message, &players);
    mark_rooms_game_over(world, &session_ids);
    remove_active_sessions_for_players(world, &players);
    world.insert_resource(ended_state);
    remove_live_session_resources(world);
    world.insert_resource(LobbyState::GameOver);
}

/// Sole drainer for `MessageReceiver<C2SAcknowledgeResult>`.
pub fn handle_result_acknowledgements(
    round_state: Option<Res<RoundState>>,
    connections: Res<PlayerConnectionMap>,
    mut ended_state: Option<ResMut<EndedSessionResultState>>,
    mut reconnect_tracker: Option<ResMut<ReconnectTracker>>,
    mut receivers: Query<(&RemoteId, &mut MessageReceiver<C2SAcknowledgeResult>)>,
    mut commands: Commands,
) {
    if !matches!(
        round_state.as_deref().map(|state| state.phase),
        Some(RoundPhase::GameOver)
    ) {
        drain_result_ack_receivers(&mut receivers);
        return;
    }

    let Some(ended_state) = ended_state.as_deref_mut() else {
        drain_result_ack_receivers(&mut receivers);
        return;
    };

    let mut terminal_cleanup = false;
    for (remote, mut receiver) in receivers.iter_mut() {
        for _message in receiver.receive() {
            tracing::info!(
                peer_id = ?remote.0,
                "c2s_acknowledge_result: recv"
            );
            if resolve_result_acknowledgement(
                round_state.as_deref().map(|state| state.phase),
                Some(&mut *ended_state),
                connections.0.get(&remote.0).copied(),
            ) == ResultAcknowledgementOutcome::AllAcknowledged
            {
                terminal_cleanup = true;
            }
        }
    }

    if terminal_cleanup {
        cleanup_ended_session_reconnect_state(reconnect_tracker.as_deref_mut(), ended_state);
        commands.remove_resource::<EndedSessionResultState>();
    }
}

pub fn tick_ended_session_result_timeout(
    time: Res<Time>,
    ended_state: Option<Res<EndedSessionResultState>>,
    mut reconnect_tracker: Option<ResMut<ReconnectTracker>>,
    mut commands: Commands,
) {
    let Some(ended_state) = ended_state.as_deref() else {
        return;
    };

    if elapsed_millis_u64(time.elapsed()) < ended_state.expires_at_ms {
        return;
    }

    cleanup_ended_session_reconnect_state(reconnect_tracker.as_deref_mut(), ended_state);
    commands.remove_resource::<EndedSessionResultState>();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultAcknowledgementOutcome {
    Discarded,
    Acknowledged,
    Duplicate,
    AllAcknowledged,
}

pub fn apply_result_acknowledgement(
    ended_state: Option<&mut EndedSessionResultState>,
    player_id: PlayerId,
) -> ResultAcknowledgementOutcome {
    let Some(ended_state) = ended_state else {
        return ResultAcknowledgementOutcome::Discarded;
    };

    if !ended_state.participants.contains(&player_id) {
        return ResultAcknowledgementOutcome::Discarded;
    }

    if !ended_state.acknowledge(player_id) {
        return ResultAcknowledgementOutcome::Duplicate;
    }

    if ended_state.all_acknowledged() {
        ResultAcknowledgementOutcome::AllAcknowledged
    } else {
        ResultAcknowledgementOutcome::Acknowledged
    }
}

pub fn resolve_result_acknowledgement(
    round_phase: Option<RoundPhase>,
    ended_state: Option<&mut EndedSessionResultState>,
    sender: Option<PlayerId>,
) -> ResultAcknowledgementOutcome {
    if round_phase != Some(RoundPhase::GameOver) {
        return ResultAcknowledgementOutcome::Discarded;
    }

    let Some(player_id) = sender else {
        return ResultAcknowledgementOutcome::Discarded;
    };

    apply_result_acknowledgement(ended_state, player_id)
}

pub fn cleanup_ended_session_reconnect_state(
    tracker: Option<&mut ReconnectTracker>,
    ended_state: &EndedSessionResultState,
) {
    cleanup_reconnect_tracker(
        tracker,
        &ended_state.session_ids,
        &participants_sorted(ended_state),
    );
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
            tracing::info!(
                peer_id = ?remote.0,
                mode = ?msg.mode,
                "c2s_create_room: recv"
            );
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
            tracing::info!(
                peer_id = ?remote.0,
                room_code = ?msg.room_code,
                requested_slot = ?msg.requested_slot,
                "c2s_join_room: recv"
            );
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

/// Sole drainer for `MessageReceiver<C2SSelectClass>`.
pub fn handle_select_class(
    connections: Res<PlayerConnectionMap>,
    rooms: Res<RoomSessions>,
    active_sessions: Res<ActiveSessions>,
    previews: Option<ResMut<ClassPreviews>>,
    mut receivers: Query<(&RemoteId, &mut MessageReceiver<C2SSelectClass>)>,
) {
    let Some(mut previews) = previews else {
        for (_, mut receiver) in receivers.iter_mut() {
            for _ in receiver.receive() {}
        }
        return;
    };

    for (remote, mut receiver) in receivers.iter_mut() {
        for msg in receiver.receive() {
            tracing::info!(
                peer_id = ?remote.0,
                class_id = ?msg.class_id,
                "c2s_select_class: recv"
            );
            let Some(player_id) = connections.0.get(&remote.0).copied() else {
                continue;
            };

            let _ = select_class(
                &rooms,
                &active_sessions,
                &mut previews,
                player_id,
                msg.class_id,
            );
        }
    }
}

/// Sole drainer for `MessageReceiver<C2SConfirmClass>`.
pub fn handle_confirm_class(
    connections: Res<PlayerConnectionMap>,
    active_sessions: Res<ActiveSessions>,
    mut rooms: ResMut<RoomSessions>,
    selections: Option<ResMut<ClassSelections>>,
    mut receivers: Query<(&RemoteId, &mut MessageReceiver<C2SConfirmClass>)>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
) {
    let Some(mut selections) = selections else {
        for (_, mut receiver) in receivers.iter_mut() {
            for _ in receiver.receive() {}
        }
        return;
    };

    let server = server.single().ok();

    for (remote, mut receiver) in receivers.iter_mut() {
        for msg in receiver.receive() {
            tracing::info!(
                peer_id = ?remote.0,
                class_id = ?msg.class_id,
                "c2s_confirm_class: recv"
            );
            let Some(player_id) = connections.0.get(&remote.0).copied() else {
                tracing::warn!(
                    peer_id = ?remote.0,
                    class_id = ?msg.class_id,
                    reason = "no_player_for_peer",
                    "c2s_confirm_class: silent discard prevented"
                );
                continue;
            };

            let outcome = confirm_class(
                &mut rooms,
                &active_sessions,
                &mut selections,
                player_id,
                msg.class_id,
            );

            if matches!(outcome, ConfirmClassOutcome::Ignored) {
                let reason =
                    confirm_class_ignored_reason(&rooms, &active_sessions, player_id, msg.class_id);
                tracing::warn!(
                    peer_id = ?remote.0,
                    player_id = ?player_id,
                    class_id = ?msg.class_id,
                    reason,
                    "c2s_confirm_class: silent discard prevented"
                );
            }

            if let (Some(server), Some(sender)) = (server, sender.as_mut()) {
                send_confirm_class_outcome(sender, server, &connections.0, remote.0, &outcome);
            }
        }
    }
}

fn confirm_class_ignored_reason(
    rooms: &RoomSessions,
    active_sessions: &ActiveSessions,
    player_id: PlayerId,
    class_id: ClassId,
) -> &'static str {
    if class_id == ClassId::Neutral {
        return "class_neutral_disallowed";
    }
    let Some(session_id) = active_sessions.0.get(&player_id).copied() else {
        return "no_active_session";
    };
    let Some(session) = rooms.get(session_id) else {
        return "session_id_not_in_rooms";
    };
    if session.state != LobbyState::LobbyWaiting {
        return "session_not_in_lobby_waiting";
    }
    if !session.slots.0.iter().any(|s| s.player == Some(player_id)) {
        return "slot_not_found_for_player";
    }
    "unknown"
}

pub fn handle_lobby_disconnect(
    trigger: On<Add, Disconnected>,
    mut commands: Commands,
    clients: Query<&RemoteId>,
    connections: Res<PlayerConnectionMap>,
    mut rooms: ResMut<RoomSessions>,
    mut active_sessions: ResMut<ActiveSessions>,
    mut lobby_state: Option<ResMut<LobbyState>>,
    slots: Option<Res<SessionSlots>>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
    mut outbox: Option<ResMut<SessionNetworkOutbox>>,
) {
    // Lightyear 0.26.4 reports disconnects by adding the `Disconnected`
    // marker component to the connection entity; `RemoteId` stores the `PeerId`.
    let Ok(remote) = clients.get(trigger.entity) else {
        return;
    };
    let Some(player_id) = connections.0.get(&remote.0).copied() else {
        return;
    };

    let players = if rooms.len() == 0 {
        cancel_global_lobby(
            lobby_state.as_deref_mut(),
            slots.as_deref(),
            &mut active_sessions,
            Some(player_id),
        )
    } else {
        cancel_lobby_for_player(&mut rooms, &mut active_sessions, player_id)
    };

    let Some(players) = players else {
        return;
    };

    cancel_lobby_resources(&mut commands, lobby_state.as_deref_mut());
    broadcast_session_cancelled(
        &server,
        sender.as_mut(),
        Some(&connections),
        outbox.as_deref_mut(),
        SessionCancelledReason::PlayerDisconnected,
        &players,
        Some(player_id),
    );
}

/// Sole drainer for `MessageReceiver<C2SHeartbeat>`.
pub fn handle_lobby_heartbeat(
    time: Res<Time>,
    connections: Res<PlayerConnectionMap>,
    active_sessions: Res<ActiveSessions>,
    mut rooms: ResMut<RoomSessions>,
    mut lobby_heartbeats: Option<ResMut<LobbyHeartbeats>>,
    mut receivers: Query<(&RemoteId, &mut MessageReceiver<C2SHeartbeat>)>,
    mut heartbeats: MessageWriter<PlayerHeartbeat>,
) {
    let now = time.elapsed().as_secs_f64();

    for (remote, mut receiver) in receivers.iter_mut() {
        for msg in receiver.receive() {
            tracing::info!(
                peer_id = ?remote.0,
                "c2s_heartbeat: recv"
            );
            debug!("Received C2SHeartbeat: {:?}", msg);
            let Some(player_id) = connections.0.get(&remote.0).copied() else {
                continue;
            };

            heartbeats.write(PlayerHeartbeat { player: player_id });

            if let Some(heartbeats) = lobby_heartbeats.as_deref_mut() {
                heartbeats.0.insert(player_id, now);
            }

            let Some(session_id) = active_sessions.0.get(&player_id).copied() else {
                continue;
            };
            let Some(session) = rooms.get_mut(session_id) else {
                continue;
            };
            if matches!(
                session.state,
                LobbyState::LobbyWaiting | LobbyState::LobbyReady
            ) {
                session.heartbeats.0.insert(player_id, now);
            }
        }
    }
}

pub fn tick_lobby_heartbeats(
    mut commands: Commands,
    time: Res<Time>,
    config: Option<Res<GameConfig>>,
    mut rooms: ResMut<RoomSessions>,
    mut active_sessions: ResMut<ActiveSessions>,
    mut lobby_state: Option<ResMut<LobbyState>>,
    slots: Option<Res<SessionSlots>>,
    heartbeats: Option<Res<LobbyHeartbeats>>,
    connections: Option<Res<PlayerConnectionMap>>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
    mut outbox: Option<ResMut<SessionNetworkOutbox>>,
) {
    let now = time.elapsed().as_secs_f64();
    let timeout = config
        .as_ref()
        .map(|config| config.lobby_heartbeat_timeout_seconds)
        .unwrap_or_else(|| shared::config::GameConfig::default().lobby_heartbeat_timeout_seconds);

    let Some((players, timed_out_player)) = find_lobby_heartbeat_timeout(
        &rooms,
        &active_sessions,
        lobby_state.as_deref(),
        slots.as_deref(),
        heartbeats.as_deref(),
        now,
        f64::from(timeout),
    )
    .and_then(|timed_out_player| {
        if rooms.len() == 0 {
            cancel_global_lobby(
                lobby_state.as_deref_mut(),
                slots.as_deref(),
                &mut active_sessions,
                Some(timed_out_player),
            )
            .map(|players| (players, timed_out_player))
        } else {
            cancel_lobby_for_player(&mut rooms, &mut active_sessions, timed_out_player)
                .map(|players| (players, timed_out_player))
        }
    }) else {
        return;
    };

    cancel_lobby_resources(&mut commands, lobby_state.as_deref_mut());
    broadcast_session_cancelled(
        &server,
        sender.as_mut(),
        connections.as_deref(),
        outbox.as_deref_mut(),
        SessionCancelledReason::HeartbeatTimeout,
        &players,
        Some(timed_out_player),
    );
}

pub fn lobby_timeout_check(
    mut commands: Commands,
    time: Res<Time>,
    mut rooms: ResMut<RoomSessions>,
    mut active_sessions: ResMut<ActiveSessions>,
    mut lobby_state: Option<ResMut<LobbyState>>,
    slots: Option<Res<SessionSlots>>,
    selections: Option<Res<ClassSelections>>,
    deadline: Option<Res<LobbyDeadline>>,
    connections: Option<Res<PlayerConnectionMap>>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
    mut outbox: Option<ResMut<SessionNetworkOutbox>>,
) {
    let now = time.elapsed().as_secs_f64();

    let players = if rooms.len() == 0 {
        if !global_lobby_deadline_expired(
            lobby_state.as_deref(),
            slots.as_deref(),
            selections.as_deref(),
            deadline.as_deref().copied(),
            now,
        ) {
            return;
        }

        cancel_global_lobby(
            lobby_state.as_deref_mut(),
            slots.as_deref(),
            &mut active_sessions,
            None,
        )
    } else {
        find_lobby_timeout(&rooms, selections.as_deref(), now).and_then(|session_id| {
            cancel_lobby_by_session(&mut rooms, &mut active_sessions, session_id)
        })
    };

    let Some(players) = players else {
        return;
    };

    cancel_lobby_resources(&mut commands, lobby_state.as_deref_mut());
    broadcast_session_cancelled(
        &server,
        sender.as_mut(),
        connections.as_deref(),
        outbox.as_deref_mut(),
        SessionCancelledReason::LobbyTimeout,
        &players,
        None,
    );
}

pub fn cancel_lobby_for_player(
    rooms: &mut RoomSessions,
    active_sessions: &mut ActiveSessions,
    player_id: PlayerId,
) -> Option<Vec<PlayerId>> {
    let session_id = active_sessions.0.get(&player_id).copied()?;
    cancel_lobby_by_session(rooms, active_sessions, session_id)
}

pub fn cancel_lobby_by_session(
    rooms: &mut RoomSessions,
    active_sessions: &mut ActiveSessions,
    session_id: SessionId,
) -> Option<Vec<PlayerId>> {
    let session = rooms.get_mut(session_id)?;
    if !matches!(
        session.state,
        LobbyState::LobbyWaiting | LobbyState::LobbyReady
    ) {
        return None;
    }

    let players = occupied_players(&session.slots);
    if players.is_empty() {
        return None;
    }

    session.state = LobbyState::LobbyCancelled;
    session.heartbeats.0.clear();
    for player in &players {
        active_sessions.0.remove(player);
    }

    Some(players)
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

pub fn select_class(
    rooms: &RoomSessions,
    active_sessions: &ActiveSessions,
    previews: &mut ClassPreviews,
    player_id: PlayerId,
    class_id: ClassId,
) -> SelectClassOutcome {
    if class_id == ClassId::Neutral {
        return SelectClassOutcome::Ignored;
    }

    let Some(session_id) = active_sessions.0.get(&player_id).copied() else {
        return SelectClassOutcome::Ignored;
    };
    let Some(session) = rooms.get(session_id) else {
        return SelectClassOutcome::Ignored;
    };

    if session.state != LobbyState::LobbyWaiting || !session_has_player(&session.slots, player_id) {
        return SelectClassOutcome::Ignored;
    }

    previews.0.insert(player_id, class_id);
    SelectClassOutcome::PreviewUpdated
}

pub fn confirm_class(
    rooms: &mut RoomSessions,
    active_sessions: &ActiveSessions,
    selections: &mut ClassSelections,
    player_id: PlayerId,
    class_id: ClassId,
) -> ConfirmClassOutcome {
    if class_id == ClassId::Neutral {
        return ConfirmClassOutcome::Ignored;
    }

    if let Some(confirmed) = selections.0.get(&player_id).copied() {
        return if confirmed == class_id {
            class_lock_ack(&rooms, active_sessions, player_id, class_id)
        } else {
            class_lock_rejected()
        };
    }

    let Some(session_id) = active_sessions.0.get(&player_id).copied() else {
        return ConfirmClassOutcome::Ignored;
    };
    let Some(session) = rooms.get_mut(session_id) else {
        return ConfirmClassOutcome::Ignored;
    };

    if session.state != LobbyState::LobbyWaiting {
        return ConfirmClassOutcome::Ignored;
    }

    {
        let Some(slot) = session
            .slots
            .0
            .iter_mut()
            .find(|slot| slot.player == Some(player_id))
        else {
            return ConfirmClassOutcome::Ignored;
        };

        if let Some(confirmed) = slot.class {
            selections.0.insert(player_id, confirmed);
            return if confirmed == class_id {
                class_lock_ack(&rooms, active_sessions, player_id, class_id)
            } else {
                class_lock_rejected()
            };
        }

        slot.class = Some(class_id);
    }

    selections.0.insert(player_id, class_id);

    let revealed =
        all_slots_locked(&session.slots).then(|| classes_revealed_message(&session.slots));
    let reveal_recipients = if revealed.is_some() {
        occupied_players(&session.slots)
    } else {
        Vec::new()
    };

    ConfirmClassOutcome::Locked {
        locked: S2CClassLocked { class_id },
        revealed,
        reveal_recipients,
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
        GameMode::TwoVTwo => SessionSlots(vec![
            SessionSlot {
                index: 0,
                team: 0,
                player: Some(creator),
                class: None,
            },
            SessionSlot {
                index: 1,
                team: 0,
                player: None,
                class: None,
            },
            SessionSlot {
                index: 2,
                team: 1,
                player: None,
                class: None,
            },
            SessionSlot {
                index: 3,
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

fn first_game_over_message(world: &World) -> Option<GameOverEmitted> {
    let messages = world.get_resource::<Messages<GameOverEmitted>>()?;
    let mut cursor = messages.get_cursor();
    cursor.read(messages).copied().next()
}

fn build_ended_session_result_state(
    world: &mut World,
    result: S2CGameOver,
    players: &[PlayerId],
    session_ids: &HashSet<SessionId>,
) -> EndedSessionResultState {
    let mut final_snapshots = HashMap::new();
    for player in players {
        if let Some(mut snapshot) = build_game_snapshot(*player, world) {
            snapshot.phase = ProtocolRoundPhase::GameOver;
            snapshot.round_number = result.round;
            snapshot.timer_remaining_ms = None;
            final_snapshots.insert(*player, snapshot);
        }
    }

    let expires_at_ms = elapsed_millis_u64(
        world
            .get_resource::<Time>()
            .map(|time| time.elapsed())
            .unwrap_or_default(),
    )
    .saturating_add(u64::from(ack_timeout_ms(world)));

    EndedSessionResultState {
        result,
        participants: players.iter().copied().collect(),
        acknowledged: HashSet::new(),
        final_snapshots,
        expires_at_ms,
        session_ids: session_ids.clone(),
    }
}

fn session_players(
    session_config: Option<&SessionConfig>,
    slots: Option<&SessionSlots>,
) -> Vec<PlayerId> {
    let mut players = session_config
        .map(|config| config.players().collect::<Vec<_>>())
        .unwrap_or_default();

    if players.is_empty() {
        if let Some(slots) = slots {
            players = occupied_players(slots);
        }
    }

    players.sort_by_key(|player| player.0);
    players.dedup();
    players
}

fn session_ids_for_players(rooms: &RoomSessions, players: &[PlayerId]) -> HashSet<SessionId> {
    rooms
        .session_ids()
        .into_iter()
        .filter(|session_id| {
            rooms
                .get(*session_id)
                .map(|session| {
                    occupied_players(&session.slots)
                        .into_iter()
                        .any(|player| players.contains(&player))
                })
                .unwrap_or(false)
        })
        .collect()
}

fn session_ids_for_players_from_world(world: &World, players: &[PlayerId]) -> HashSet<SessionId> {
    let mut session_ids = world
        .get_resource::<ActiveSessions>()
        .map(|active_sessions| {
            players
                .iter()
                .filter_map(|player| active_sessions.0.get(player).copied())
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();

    if session_ids.is_empty() {
        if let Some(rooms) = world.get_resource::<RoomSessions>() {
            session_ids.extend(session_ids_for_players(rooms, players));
        }
    }

    session_ids
}

fn remove_active_sessions_for_players(world: &mut World, players: &[PlayerId]) {
    let Some(mut active_sessions) = world.get_resource_mut::<ActiveSessions>() else {
        return;
    };

    for player in players {
        active_sessions.0.remove(player);
    }
}

fn mark_rooms_game_over(world: &mut World, session_ids: &HashSet<SessionId>) {
    let Some(mut rooms) = world.get_resource_mut::<RoomSessions>() else {
        return;
    };

    for session_id in session_ids {
        if let Some(session) = rooms.get_mut(*session_id) {
            session.state = LobbyState::GameOver;
            session.heartbeats.0.clear();
        }
    }
}

fn remove_live_session_resources(world: &mut World) {
    world.remove_resource::<SessionConfig>();
    world.remove_resource::<ServerRng>();
    world.remove_resource::<SessionSlots>();
    world.remove_resource::<ClassSelections>();
    world.remove_resource::<ClassPreviews>();
    if let Some(mut requests) = world.get_resource_mut::<PlacementTimerMultiplierRequests>() {
        requests.0.clear();
    }
    world.remove_resource::<LobbyDeadline>();
    world.remove_resource::<LobbyHeartbeats>();
}

fn broadcast_game_over_from_world(
    world: &mut World,
    message: S2CGameOver,
    recipients: &[PlayerId],
) {
    tracing::info!(
        recipient_count = recipients.len(),
        round = message.round,
        reason = ?message.reason,
        "broadcast_game_over_from_world enter"
    );

    if let Some(mut outbox) = world.get_resource_mut::<SessionNetworkOutbox>() {
        outbox.push_game_over(message.clone());
    }

    let target_peers = world
        .get_resource::<PlayerConnectionMap>()
        .map(|connections| {
            recipients
                .iter()
                .filter_map(|player_id| peer_for_player(&connections.0, *player_id))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if target_peers.is_empty() {
        tracing::warn!(
            recipient_count = recipients.len(),
            round = message.round,
            "broadcast_game_over_from_world DROPPED — target_peers empty; no PlayerConnectionMap or all recipients unresolved"
        );
        return;
    }

    let mut system_state: bevy::ecs::system::SystemState<(
        Query<&Server>,
        Option<ServerMultiMessageSender>,
    )> = bevy::ecs::system::SystemState::new(world);
    let (server, mut sender) = system_state.get_mut(world);
    let (Ok(server), Some(sender)) = (server.single(), sender.as_mut()) else {
        tracing::warn!(
            recipient_count = recipients.len(),
            round = message.round,
            "broadcast_game_over_from_world DROPPED — Server query empty or ServerMultiMessageSender missing"
        );
        return;
    };

    if let Err(e) = sender.send::<S2CGameOver, ReliableChannel>(
        &message,
        server,
        &NetworkTarget::Only(target_peers),
    ) {
        tracing::error!(
            round = message.round,
            err = ?e,
            "S2C send failed: type=S2CGameOver, handler=broadcast_game_over_from_world"
        );
    }
}

fn participants_sorted(ended_state: &EndedSessionResultState) -> Vec<PlayerId> {
    let mut players = ended_state.participants.iter().copied().collect::<Vec<_>>();
    players.sort_by_key(|player| player.0);
    players
}

fn drain_result_ack_receivers(
    receivers: &mut Query<(&RemoteId, &mut MessageReceiver<C2SAcknowledgeResult>)>,
) {
    for (_, mut receiver) in receivers.iter_mut() {
        for _message in receiver.receive() {}
    }
}

fn cleanup_reconnect_tracker(
    tracker: Option<&mut ReconnectTracker>,
    session_ids: &HashSet<SessionId>,
    players: &[PlayerId],
) {
    let Some(tracker) = tracker else {
        return;
    };

    tracker.token_map.retain(|_, (session_id, player)| {
        !session_ids.contains(session_id) && !players.contains(player)
    });
    tracker
        .deferred_queue
        .retain(|player, _| !players.contains(player));
    for player in players {
        tracker.snapshot_sent.remove(player);
        tracker.sang_meprise_sent_to.remove(player);
    }
}

fn ack_timeout_ms(world: &World) -> u32 {
    world
        .get_resource::<GameConfig>()
        .map(|config| config.ack_timeout_ms)
        .unwrap_or_else(|| shared::config::GameConfig::default().ack_timeout_ms)
}

fn elapsed_millis_u64(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn broadcast_game_over(
    server: &Query<&Server>,
    sender: Option<&mut ServerMultiMessageSender>,
    connections: Option<&PlayerConnectionMap>,
    outbox: Option<&mut SessionNetworkOutbox>,
    message: S2CGameOver,
    recipients: &[PlayerId],
) {
    tracing::info!(
        recipient_count = recipients.len(),
        round = message.round,
        reason = ?message.reason,
        "broadcast_game_over enter"
    );

    if let Some(outbox) = outbox {
        outbox.push_game_over(message.clone());
    }

    let Some(sender) = sender else {
        tracing::warn!(
            recipient_count = recipients.len(),
            round = message.round,
            "broadcast_game_over DROPPED — ServerMultiMessageSender missing"
        );
        return;
    };
    let Some(server) = server.single().ok() else {
        tracing::warn!(
            recipient_count = recipients.len(),
            round = message.round,
            "broadcast_game_over DROPPED — Server query empty"
        );
        return;
    };
    let Some(connections) = connections else {
        tracing::warn!(
            recipient_count = recipients.len(),
            round = message.round,
            "broadcast_game_over DROPPED — PlayerConnectionMap missing"
        );
        return;
    };

    let target_peers = recipients
        .iter()
        .filter_map(|player_id| peer_for_player(&connections.0, *player_id))
        .collect::<Vec<_>>();
    if target_peers.is_empty() {
        tracing::warn!(
            recipient_count = recipients.len(),
            round = message.round,
            "broadcast_game_over DROPPED — target_peers empty; all recipients unresolved in PlayerConnectionMap"
        );
        return;
    }

    if let Err(e) = sender.send::<S2CGameOver, ReliableChannel>(
        &message,
        server,
        &NetworkTarget::Only(target_peers),
    ) {
        tracing::error!(
            round = message.round,
            err = ?e,
            "S2C send failed: type=S2CGameOver, handler=broadcast_game_over"
        );
    }
}

fn broadcast_session_settings_updated(
    server: &Query<&Server>,
    sender: Option<&mut ServerMultiMessageSender>,
    connections: Option<&PlayerConnectionMap>,
    outbox: Option<&mut SessionNetworkOutbox>,
    message: S2CSessionSettingsUpdated,
    recipients: &[PlayerId],
) {
    tracing::info!(
        recipient_count = recipients.len(),
        placement_timer_multiplier = ?message.placement_timer_multiplier_effective,
        "broadcast_session_settings_updated enter"
    );

    if let Some(outbox) = outbox {
        outbox.push_session_settings_updated(message.clone());
    }

    let Some(sender) = sender else {
        tracing::warn!(
            recipient_count = recipients.len(),
            "broadcast_session_settings_updated DROPPED — ServerMultiMessageSender missing"
        );
        return;
    };
    let Some(server) = server.single().ok() else {
        tracing::warn!(
            recipient_count = recipients.len(),
            "broadcast_session_settings_updated DROPPED — Server query empty"
        );
        return;
    };
    let Some(connections) = connections else {
        tracing::warn!(
            recipient_count = recipients.len(),
            "broadcast_session_settings_updated DROPPED — PlayerConnectionMap missing"
        );
        return;
    };

    let target_peers = recipients
        .iter()
        .filter_map(|player_id| peer_for_player(&connections.0, *player_id))
        .collect::<Vec<_>>();
    if target_peers.is_empty() {
        tracing::warn!(
            recipient_count = recipients.len(),
            "broadcast_session_settings_updated DROPPED — target_peers empty; all recipients unresolved in PlayerConnectionMap"
        );
        return;
    }

    if let Err(e) = sender.send::<S2CSessionSettingsUpdated, ReliableChannel>(
        &message,
        server,
        &NetworkTarget::Only(target_peers),
    ) {
        tracing::error!(
            err = ?e,
            "S2C send failed: type=S2CSessionSettingsUpdated, handler=broadcast_session_settings_updated"
        );
    }
}

fn find_lobby_heartbeat_timeout(
    rooms: &RoomSessions,
    active_sessions: &ActiveSessions,
    lobby_state: Option<&LobbyState>,
    slots: Option<&SessionSlots>,
    heartbeats: Option<&LobbyHeartbeats>,
    now: f64,
    timeout_seconds: f64,
) -> Option<PlayerId> {
    if rooms.len() == 0 {
        if lobby_state != Some(&LobbyState::LobbyWaiting) {
            return None;
        }
        let slots = slots?;
        let heartbeats = heartbeats?;
        return occupied_players(slots).into_iter().find(|player| {
            active_sessions.0.contains_key(player)
                && heartbeats
                    .0
                    .get(player)
                    .map(|last_seen| now - *last_seen > timeout_seconds)
                    .unwrap_or(true)
        });
    }

    rooms.session_ids().into_iter().find_map(|session_id| {
        let session = rooms.get(session_id)?;
        if session.state != LobbyState::LobbyWaiting {
            return None;
        }
        occupied_players(&session.slots).into_iter().find(|player| {
            session
                .heartbeats
                .0
                .get(player)
                .map(|last_seen| now - *last_seen > timeout_seconds)
                .unwrap_or(true)
        })
    })
}

fn find_lobby_timeout(
    rooms: &RoomSessions,
    selections: Option<&ClassSelections>,
    now: f64,
) -> Option<SessionId> {
    let selections = selections?;
    rooms.session_ids().into_iter().find(|session_id| {
        rooms
            .get(*session_id)
            .map(|session| {
                session.state == LobbyState::LobbyWaiting
                    && now > session.lobby_deadline.0
                    && !f4_session_ready(&session.slots, selections, now, session.lobby_deadline)
            })
            .unwrap_or(false)
    })
}

fn global_lobby_deadline_expired(
    lobby_state: Option<&LobbyState>,
    slots: Option<&SessionSlots>,
    selections: Option<&ClassSelections>,
    deadline: Option<LobbyDeadline>,
    now: f64,
) -> bool {
    let (Some(LobbyState::LobbyWaiting), Some(slots), Some(selections), Some(deadline)) =
        (lobby_state, slots, selections, deadline)
    else {
        return false;
    };

    now > deadline.0 && !f4_session_ready(slots, selections, now, deadline)
}

fn cancel_global_lobby(
    lobby_state: Option<&mut LobbyState>,
    slots: Option<&SessionSlots>,
    active_sessions: &mut ActiveSessions,
    required_player: Option<PlayerId>,
) -> Option<Vec<PlayerId>> {
    let Some(lobby_state) = lobby_state else {
        return None;
    };
    if !matches!(
        *lobby_state,
        LobbyState::LobbyWaiting | LobbyState::LobbyReady
    ) {
        return None;
    }

    let slots = slots?;
    let players = occupied_players(slots);
    if players.is_empty() {
        return None;
    }
    if let Some(required_player) = required_player {
        if !players.contains(&required_player) || !active_sessions.0.contains_key(&required_player)
        {
            return None;
        }
    }

    *lobby_state = LobbyState::LobbyCancelled;
    for player in &players {
        active_sessions.0.remove(player);
    }

    Some(players)
}

fn cancel_lobby_resources(commands: &mut Commands, lobby_state: Option<&mut LobbyState>) {
    if let Some(lobby_state) = lobby_state {
        *lobby_state = LobbyState::LobbyCancelled;
    }
    commands.remove_resource::<LobbyHeartbeats>();
    commands.remove_resource::<ClassPreviews>();
    commands.remove_resource::<ClassSelections>();
    commands.remove_resource::<LobbyDeadline>();
    commands.remove_resource::<SessionSlots>();
}

fn session_has_player(slots: &SessionSlots, player_id: PlayerId) -> bool {
    slots.0.iter().any(|slot| slot.player == Some(player_id))
}

fn all_slots_locked(slots: &SessionSlots) -> bool {
    slots
        .0
        .iter()
        .all(|slot| slot.player.is_some() && slot.class.is_some())
}

fn classes_revealed_message(slots: &SessionSlots) -> S2CClassesRevealed {
    let mut player_class_map = slots
        .0
        .iter()
        .filter_map(|slot| Some((slot.player?, slot.class?)))
        .collect::<Vec<_>>();
    player_class_map.sort_by_key(|(player_id, _)| player_id.0);
    S2CClassesRevealed { player_class_map }
}

fn class_lock_rejected() -> ConfirmClassOutcome {
    ConfirmClassOutcome::Rejected(S2CConfirmClassRejected {
        reason: ConfirmClassRejectedReason::ClassAlreadyConfirmed,
    })
}

fn class_lock_ack(
    rooms: &RoomSessions,
    active_sessions: &ActiveSessions,
    player_id: PlayerId,
    class_id: ClassId,
) -> ConfirmClassOutcome {
    let revealed = active_sessions
        .0
        .get(&player_id)
        .and_then(|session_id| rooms.get(*session_id))
        .and_then(|session| {
            all_slots_locked(&session.slots).then(|| classes_revealed_message(&session.slots))
        });
    let reveal_recipients = active_sessions
        .0
        .get(&player_id)
        .and_then(|session_id| rooms.get(*session_id))
        .map(|session| occupied_players(&session.slots))
        .unwrap_or_default();

    ConfirmClassOutcome::Locked {
        locked: S2CClassLocked { class_id },
        revealed,
        reveal_recipients,
    }
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
            tracing::info!(
                peer_id = ?peer_id,
                room_code = %msg.room_code,
                session_id = %msg.session_id,
                "send_create_room_outcome enter (Created)"
            );
            // Lightyear 0.26 unicast is NetworkTarget::Single(PeerId), verified in
            // tests/evidence/lightyear-026-verification.md item 7.
            if let Err(e) = sender.send::<S2CRoomCreated, ReliableChannel>(
                msg,
                server,
                &NetworkTarget::Single(peer_id),
            ) {
                tracing::error!(
                    peer_id = ?peer_id,
                    room_code = %msg.room_code,
                    err = ?e,
                    "S2C send failed: type=S2CRoomCreated, handler=send_create_room_outcome"
                );
            }
        }
        CreateRoomOutcome::Rejected(msg) => {
            tracing::info!(
                peer_id = ?peer_id,
                reason = ?msg.reason,
                "send_create_room_outcome enter (Rejected)"
            );
            if let Err(e) = sender.send::<S2CCreateRoomRejected, ReliableChannel>(
                msg,
                server,
                &NetworkTarget::Single(peer_id),
            ) {
                tracing::error!(
                    peer_id = ?peer_id,
                    reason = ?msg.reason,
                    err = ?e,
                    "S2C send failed: type=S2CCreateRoomRejected, handler=send_create_room_outcome"
                );
            }
        }
    }
}

fn send_confirm_class_outcome(
    sender: &mut ServerMultiMessageSender,
    server: &Server,
    connections: &HashMap<PeerId, PlayerId>,
    peer_id: PeerId,
    outcome: &ConfirmClassOutcome,
) {
    match outcome {
        ConfirmClassOutcome::Locked {
            locked,
            revealed,
            reveal_recipients,
        } => {
            tracing::info!(
                peer_id = ?peer_id,
                class_id = ?locked.class_id,
                "send_confirm_class_outcome enter (Locked)"
            );
            if let Err(e) = sender.send::<S2CClassLocked, ReliableChannel>(
                locked,
                server,
                &NetworkTarget::Single(peer_id),
            ) {
                tracing::error!(
                    peer_id = ?peer_id,
                    class_id = ?locked.class_id,
                    err = ?e,
                    "S2C send failed: type=S2CClassLocked, handler=send_confirm_class_outcome"
                );
            }

            if let Some(revealed) = revealed {
                let target_peers = reveal_recipients
                    .iter()
                    .filter_map(|player_id| peer_for_player(connections, *player_id))
                    .collect::<Vec<_>>();

                if target_peers.is_empty() {
                    tracing::warn!(
                        peer_id = ?peer_id,
                        recipient_count = reveal_recipients.len(),
                        "send_confirm_class_outcome (Revealed) DROPPED — target_peers empty; all reveal recipients unresolved in connections map"
                    );
                } else if let Err(e) = sender.send::<S2CClassesRevealed, ReliableChannel>(
                    revealed,
                    server,
                    &NetworkTarget::Only(target_peers),
                ) {
                    tracing::error!(
                        peer_id = ?peer_id,
                        recipient_count = reveal_recipients.len(),
                        err = ?e,
                        "S2C send failed: type=S2CClassesRevealed, handler=send_confirm_class_outcome"
                    );
                }
            }
        }
        ConfirmClassOutcome::Rejected(msg) => {
            tracing::info!(
                peer_id = ?peer_id,
                reason = ?msg.reason,
                "send_confirm_class_outcome enter (Rejected)"
            );
            if let Err(e) = sender.send::<S2CConfirmClassRejected, ReliableChannel>(
                msg,
                server,
                &NetworkTarget::Single(peer_id),
            ) {
                tracing::error!(
                    peer_id = ?peer_id,
                    reason = ?msg.reason,
                    err = ?e,
                    "S2C send failed: type=S2CConfirmClassRejected, handler=send_confirm_class_outcome"
                );
            }
        }
        ConfirmClassOutcome::Ignored => {}
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
            tracing::info!(
                peer_id = ?peer_id,
                session_id = %ack.session_id,
                "send_join_room_outcome enter (Joined)"
            );
            if let Err(e) = sender.send::<S2CJoinAck, ReliableChannel>(
                ack,
                server,
                &NetworkTarget::Single(peer_id),
            ) {
                tracing::error!(
                    peer_id = ?peer_id,
                    session_id = %ack.session_id,
                    err = ?e,
                    "S2C send failed: type=S2CJoinAck, handler=send_join_room_outcome"
                );
            }

            let target_peers = slot_update_recipients
                .iter()
                .filter_map(|player_id| peer_for_player(connections, *player_id))
                .collect::<Vec<_>>();

            if target_peers.is_empty() {
                tracing::warn!(
                    peer_id = ?peer_id,
                    recipient_count = slot_update_recipients.len(),
                    "send_join_room_outcome (SlotUpdated) DROPPED — target_peers empty; all slot_update recipients unresolved in connections map"
                );
            } else if let Err(e) = sender.send::<S2CSlotUpdated, ReliableChannel>(
                slot_update,
                server,
                &NetworkTarget::Only(target_peers),
            ) {
                tracing::error!(
                    peer_id = ?peer_id,
                    recipient_count = slot_update_recipients.len(),
                    err = ?e,
                    "S2C send failed: type=S2CSlotUpdated, handler=send_join_room_outcome"
                );
            }
        }
        JoinRoomOutcome::Rejected(msg) => {
            tracing::info!(
                peer_id = ?peer_id,
                reason = ?msg.reason,
                "send_join_room_outcome enter (Rejected)"
            );
            if let Err(e) = sender.send::<S2CJoinRejected, ReliableChannel>(
                msg,
                server,
                &NetworkTarget::Single(peer_id),
            ) {
                tracing::error!(
                    peer_id = ?peer_id,
                    reason = ?msg.reason,
                    err = ?e,
                    "S2C send failed: type=S2CJoinRejected, handler=send_join_room_outcome"
                );
            }
        }
    }
}

fn broadcast_session_cancelled(
    server: &Query<&Server>,
    sender: Option<&mut ServerMultiMessageSender>,
    connections: Option<&PlayerConnectionMap>,
    outbox: Option<&mut SessionNetworkOutbox>,
    reason: SessionCancelledReason,
    recipients: &[PlayerId],
    exclude_player: Option<PlayerId>,
) {
    let message = S2CSessionCancelled {
        reason: match reason {
            SessionCancelledReason::LobbyTimeout => {
                shared::protocol::SessionCancelledReason::LobbyTimeout
            }
            SessionCancelledReason::PlayerDisconnected
            | SessionCancelledReason::HeartbeatTimeout => {
                shared::protocol::SessionCancelledReason::PlayerDisconnected
            }
            SessionCancelledReason::RngInitFailure => {
                shared::protocol::SessionCancelledReason::ServerRngFail
            }
        },
    };

    tracing::info!(
        recipient_count = recipients.len(),
        reason = ?reason,
        exclude_player = ?exclude_player,
        "broadcast_session_cancelled enter"
    );

    if let Some(outbox) = outbox {
        outbox.push_session_cancelled(message.clone());
    }

    let Some(sender) = sender else {
        tracing::warn!(
            recipient_count = recipients.len(),
            reason = ?reason,
            "broadcast_session_cancelled DROPPED — ServerMultiMessageSender missing"
        );
        return;
    };
    let Some(server) = server.single().ok() else {
        tracing::warn!(
            recipient_count = recipients.len(),
            reason = ?reason,
            "broadcast_session_cancelled DROPPED — Server query empty"
        );
        return;
    };
    let Some(connections) = connections else {
        tracing::warn!(
            recipient_count = recipients.len(),
            reason = ?reason,
            "broadcast_session_cancelled DROPPED — PlayerConnectionMap missing"
        );
        return;
    };

    let target_peers = connections
        .0
        .iter()
        .filter_map(|(peer_id, player_id)| {
            (recipients.contains(player_id) && Some(*player_id) != exclude_player)
                .then_some(*peer_id)
        })
        .collect::<Vec<_>>();
    if target_peers.is_empty() {
        tracing::warn!(
            recipient_count = recipients.len(),
            reason = ?reason,
            "broadcast_session_cancelled DROPPED — target_peers empty; all recipients unresolved or excluded in PlayerConnectionMap"
        );
        return;
    }

    if let Err(e) = sender.send::<S2CSessionCancelled, ReliableChannel>(
        &message,
        server,
        &NetworkTarget::Only(target_peers),
    ) {
        tracing::error!(
            reason = ?reason,
            err = ?e,
            "S2C send failed: type=S2CSessionCancelled, handler=broadcast_session_cancelled"
        );
    }
}

fn peer_for_player(connections: &HashMap<PeerId, PlayerId>, player_id: PlayerId) -> Option<PeerId> {
    connections
        .iter()
        .find_map(|(peer_id, mapped_player)| (*mapped_player == player_id).then_some(*peer_id))
}
