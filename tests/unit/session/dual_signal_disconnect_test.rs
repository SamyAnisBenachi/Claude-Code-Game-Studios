use std::collections::HashMap;

use bevy::prelude::*;
use server::core::session::{
    cancel_lobby_for_player, create_room, join_room, tick_lobby_heartbeats, ActiveSessions,
    CreateRoomOutcome, JoinRoomOutcome, LobbyState, RoomCode, RoomSessions, SessionId,
    SessionNetworkOutbox,
};
use server::foundation::config::GameConfig;
use shared::protocol::{GameMode, SessionCancelledReason};
use shared::session::PlayerId;
use uuid::Uuid;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_id(value: u128) -> SessionId {
    SessionId(Uuid::from_u128(value))
}

fn two_player_lobby(
    last_seen_a: f64,
    last_seen_b: f64,
) -> (RoomSessions, ActiveSessions, SessionId, PlayerId, PlayerId) {
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let player_a = player(1);
    let player_b = player(2);
    let id = session_id(1);

    match create_room(
        &mut rooms,
        &mut active,
        player_a,
        GameMode::OneVOne,
        0.0,
        90,
        id,
        RoomCode("ABCDEF".to_string()),
    ) {
        CreateRoomOutcome::Created(_) => {}
        CreateRoomOutcome::Rejected(rejection) => {
            panic!("room creation should succeed, got {:?}", rejection.reason)
        }
    }
    match join_room(&mut rooms, &mut active, player_b, "ABCDEF", 1, 0.0) {
        JoinRoomOutcome::Joined { .. } => {}
        JoinRoomOutcome::Rejected(rejection) => {
            panic!("join should succeed, got {:?}", rejection.reason)
        }
    }

    let session = rooms.get_mut(id).expect("room exists");
    session.heartbeats.0 = HashMap::from([(player_a, last_seen_a), (player_b, last_seen_b)]);

    (rooms, active, id, player_a, player_b)
}

fn heartbeat_app(rooms: RoomSessions, active: ActiveSessions) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(rooms);
    app.insert_resource(active);
    app.insert_resource(SessionNetworkOutbox::default());
    app.insert_resource(GameConfig(shared::config::GameConfig::default()));
    app.add_systems(Update, tick_lobby_heartbeats);
    app
}

#[test]
fn on_disconnected_cancel_clears_all_active_session_entries() {
    let (mut rooms, mut active, id, player_a, player_b) = two_player_lobby(0.0, 0.0);

    let cancelled_players =
        cancel_lobby_for_player(&mut rooms, &mut active, player_a).expect("lobby cancels");

    assert_eq!(cancelled_players, vec![player_a, player_b]);
    assert_eq!(
        rooms.get(id).expect("room remains inspectable").state,
        LobbyState::LobbyCancelled
    );
    assert!(rooms
        .get(id)
        .expect("room remains inspectable")
        .heartbeats
        .0
        .is_empty());
    assert!(!active.0.contains_key(&player_a));
    assert!(!active.0.contains_key(&player_b));
}

#[test]
fn heartbeat_gap_cancels_once_with_player_disconnected_wire_reason() {
    let (rooms, active, id, player_a, player_b) = two_player_lobby(-20.0, 0.0);
    let mut app = heartbeat_app(rooms, active);

    app.update();

    let outbox = app.world().resource::<SessionNetworkOutbox>();
    assert_eq!(outbox.session_cancelled().len(), 1);
    assert_eq!(
        outbox.session_cancelled()[0].reason,
        SessionCancelledReason::PlayerDisconnected
    );

    let rooms = app.world().resource::<RoomSessions>();
    assert_eq!(
        rooms.get(id).expect("room remains inspectable").state,
        LobbyState::LobbyCancelled
    );
    let active = app.world().resource::<ActiveSessions>();
    assert!(!active.0.contains_key(&player_a));
    assert!(!active.0.contains_key(&player_b));
}

#[test]
fn delayed_second_disconnect_signal_is_noop_after_lobby_cancelled() {
    let (mut rooms, mut active, id, player_a, _) = two_player_lobby(-20.0, 0.0);
    let _ = cancel_lobby_for_player(&mut rooms, &mut active, player_a);

    let second = cancel_lobby_for_player(&mut rooms, &mut active, player_a);

    assert!(second.is_none());
    assert_eq!(
        rooms.get(id).expect("room remains inspectable").state,
        LobbyState::LobbyCancelled
    );
}

#[test]
fn fresh_heartbeat_gap_does_not_cancel_lobby() {
    let (rooms, active, id, _, _) = two_player_lobby(0.0, -5.0);
    let mut app = heartbeat_app(rooms, active);

    app.update();

    assert!(app
        .world()
        .resource::<SessionNetworkOutbox>()
        .session_cancelled()
        .is_empty());
    assert_eq!(
        app.world()
            .resource::<RoomSessions>()
            .get(id)
            .expect("room exists")
            .state,
        LobbyState::LobbyWaiting
    );
}
