use std::collections::HashMap;

use bevy::prelude::*;
use server::core::rsm::{
    AuctionSettled, ResolutionComplete, RoundPhase, RoundState, RsmNetworkOutbox, RsmPlugin,
};
use server::core::session::{
    confirm_class, create_room, f4_session_ready, join_room, ActiveSessions, ClassSelections,
    CreateRoomOutcome, GameSessionPlugin, JoinRoomOutcome, LobbyDeadline, LobbyHeartbeats,
    LobbyState, RoomCode, RoomSessions, SessionConfig, SessionId, SessionSlot, SessionSlots,
};
use server::foundation::rng::ServerRng;
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;
use uuid::Uuid;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_id(value: u128) -> SessionId {
    SessionId(Uuid::from_u128(value))
}

fn ready_slots() -> SessionSlots {
    SessionSlots(vec![
        SessionSlot {
            index: 0,
            team: 0,
            player: Some(player(1)),
            class: Some(ClassId::Iop),
            is_bot: false,
        },
        SessionSlot {
            index: 1,
            team: 1,
            player: Some(player(2)),
            class: Some(ClassId::Cra),
            is_bot: false,
        },
    ])
}

fn ready_selections() -> ClassSelections {
    ClassSelections(HashMap::from([
        (player(1), ClassId::Iop),
        (player(2), ClassId::Cra),
    ]))
}

fn ready_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RsmPlugin);
    app.add_plugins(GameSessionPlugin);
    app.add_message::<AuctionSettled>();
    app.add_message::<ResolutionComplete>();
    app.insert_resource(server::foundation::config::GameConfig(
        shared::config::GameConfig::default(),
    ));
    app.insert_resource(LobbyState::LobbyWaiting);
    app.insert_resource(ready_slots());
    app.insert_resource(ready_selections());
    app.insert_resource(LobbyDeadline(1.0));
    app.insert_resource(LobbyHeartbeats(HashMap::from([
        (player(1), 0.0),
        (player(2), 0.0),
    ])));
    app
}

#[test]
fn test_session_ready_f4_true_on_exact_lobby_deadline() {
    let slots = ready_slots();
    let selections = ready_selections();

    assert!(f4_session_ready(
        &slots,
        &selections,
        1.0,
        LobbyDeadline(1.0)
    ));
}

#[test]
fn test_session_ready_f4_true_inserts_config_rng_and_enters_draft_initial() {
    let mut app = ready_app();

    app.update();

    let config = app.world().resource::<SessionConfig>();
    assert_eq!(config.mode, GameMode::OneVOne);
    assert_eq!(config.player_count, 2);
    assert_eq!(config.team_map.get(&player(1)), Some(&0));
    assert_eq!(config.team_map.get(&player(2)), Some(&1));
    assert_eq!(config.class_map.get(&player(1)), Some(&ClassId::Iop));
    assert_eq!(config.class_map.get(&player(2)), Some(&ClassId::Cra));
    assert_eq!(
        config.placement_timer_multiplier_effective,
        shared::protocol::PlacementTimerMultiplier::X1
    );

    assert!(app.world().contains_resource::<ServerRng>());
    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::DraftInitial
    );
    assert_eq!(app.world().resource::<RoundState>().round_number, 1);
    assert_eq!(
        *app.world().resource::<LobbyState>(),
        LobbyState::GameActive
    );
    assert!(!app.world().contains_resource::<LobbyHeartbeats>());
}

#[test]
fn test_room_session_ready_promotes_ready_room_into_draft_initial() {
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let mut selections = ClassSelections::default();
    let room_id = session_id(7);
    let player_a = player(1);
    let player_b = player(2);

    match create_room(
        &mut rooms,
        &mut active,
        player_a,
        GameMode::OneVOne,
        0.0,
        90,
        room_id,
        RoomCode("ABCDEF".to_string()),
    ) {
        CreateRoomOutcome::Created(_) => {}
        CreateRoomOutcome::Rejected(rejection) => {
            panic!("room creation should succeed, got {:?}", rejection.reason)
        }
    }
    match join_room(&mut rooms, &mut active, player_b, "ABCDEF", 1, 1.0) {
        JoinRoomOutcome::Joined { .. } => {}
        JoinRoomOutcome::Rejected(rejection) => {
            panic!("join should succeed, got {:?}", rejection.reason)
        }
    }
    let _ = confirm_class(&mut rooms, &active, &mut selections, player_a, ClassId::Iop);
    let _ = confirm_class(&mut rooms, &active, &mut selections, player_b, ClassId::Cra);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RsmPlugin);
    app.add_plugins(GameSessionPlugin);
    app.add_message::<AuctionSettled>();
    app.add_message::<ResolutionComplete>();
    app.insert_resource(server::foundation::config::GameConfig(
        shared::config::GameConfig::default(),
    ));
    app.insert_resource(rooms);
    app.insert_resource(active);
    app.insert_resource(selections);

    app.update();

    let config = app.world().resource::<SessionConfig>();
    assert_eq!(config.mode, GameMode::OneVOne);
    assert_eq!(config.player_count, 2);
    assert_eq!(config.team_map.get(&player_a), Some(&0));
    assert_eq!(config.team_map.get(&player_b), Some(&1));
    assert_eq!(config.class_map.get(&player_a), Some(&ClassId::Iop));
    assert_eq!(config.class_map.get(&player_b), Some(&ClassId::Cra));
    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::DraftInitial
    );
    assert_eq!(
        *app.world().resource::<LobbyState>(),
        LobbyState::GameActive
    );
    assert_eq!(
        app.world()
            .resource::<RoomSessions>()
            .get(room_id)
            .expect("ready room should remain tracked")
            .state,
        LobbyState::GameActive
    );
}

#[test]
fn test_session_ready_f4_false_does_not_trigger_when_class_missing() {
    let mut app = ready_app();
    app.world_mut().resource_mut::<SessionSlots>().0[1].class = None;
    app.world_mut()
        .resource_mut::<ClassSelections>()
        .0
        .remove(&player(2));

    app.update();

    assert!(!app.world().contains_resource::<SessionConfig>());
    assert!(!app.world().contains_resource::<ServerRng>());
    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::Lobby
    );
    assert_eq!(
        *app.world().resource::<LobbyState>(),
        LobbyState::LobbyWaiting
    );
    assert!(app
        .world()
        .resource::<RsmNetworkOutbox>()
        .phase_changed()
        .is_empty());
}
