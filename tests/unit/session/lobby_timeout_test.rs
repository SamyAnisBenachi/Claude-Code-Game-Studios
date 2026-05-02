use bevy::prelude::*;
use server::core::session::{
    create_room, lobby_timeout_check, ActiveSessions, CreateRoomOutcome, LobbyState, RoomCode,
    RoomSessions, SessionId, SessionNetworkOutbox,
};
use shared::protocol::{GameMode, SessionCancelledReason};
use shared::session::PlayerId;
use uuid::Uuid;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_id(value: u128) -> SessionId {
    SessionId(Uuid::from_u128(value))
}

fn one_player_lobby(deadline: f64) -> (RoomSessions, ActiveSessions, SessionId, PlayerId) {
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let player_a = player(1);
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
    rooms.get_mut(id).expect("room exists").lobby_deadline.0 = deadline;

    (rooms, active, id, player_a)
}

fn timeout_app(rooms: RoomSessions, active: ActiveSessions) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(rooms);
    app.insert_resource(active);
    app.insert_resource(SessionNetworkOutbox::default());
    app.add_systems(Update, lobby_timeout_check);
    app
}

#[test]
fn expired_lobby_deadline_cancels_unready_lobby() {
    let (rooms, active, id, player_a) = one_player_lobby(-1.0);
    let mut app = timeout_app(rooms, active);

    app.update();

    let outbox = app.world().resource::<SessionNetworkOutbox>();
    assert_eq!(outbox.session_cancelled().len(), 1);
    assert_eq!(
        outbox.session_cancelled()[0].reason,
        SessionCancelledReason::LobbyTimeout
    );

    assert_eq!(
        app.world()
            .resource::<RoomSessions>()
            .get(id)
            .expect("room remains inspectable")
            .state,
        LobbyState::LobbyCancelled
    );
    assert!(!app
        .world()
        .resource::<ActiveSessions>()
        .0
        .contains_key(&player_a));
}

#[test]
fn unexpired_lobby_deadline_does_not_cancel() {
    let (rooms, active, id, player_a) = one_player_lobby(90.0);
    let mut app = timeout_app(rooms, active);

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
    assert!(app
        .world()
        .resource::<ActiveSessions>()
        .0
        .contains_key(&player_a));
}

#[test]
fn timeout_path_is_noop_after_first_cancel_signal_wins() {
    let (mut rooms, mut active, id, player_a) = one_player_lobby(-1.0);
    let _ = server::core::session::cancel_lobby_for_player(&mut rooms, &mut active, player_a);
    let mut app = timeout_app(rooms, active);

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
            .expect("room remains inspectable")
            .state,
        LobbyState::LobbyCancelled
    );
}
