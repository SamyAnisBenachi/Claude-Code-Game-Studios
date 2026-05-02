use std::collections::HashMap;

use bevy::prelude::*;
use server::core::session::{
    build_session_config, ClassSelections, GameSessionPlugin, LobbyDeadline, LobbyHeartbeats,
    LobbyState, RoomCode, SessionCancelled, SessionCancelledReason, SessionConfig, SessionId,
    SessionReady, SessionSlot, SessionSlots, SessionToken,
};
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;
use uuid::Uuid;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn occupied_slot(index: u8, team: u8, player: PlayerId, class: ClassId) -> SessionSlot {
    SessionSlot {
        index,
        team,
        player: Some(player),
        class: Some(class),
    }
}

#[test]
fn test_session_scaffold_constructs_all_new_types() {
    let p1 = player(1);
    let slot = occupied_slot(0, 0, p1, ClassId::Iop);
    let session_id = SessionId(Uuid::nil());
    let room_code = RoomCode("ABCDEF".to_string());
    let token: SessionToken = [7; 16];
    let slots = SessionSlots(vec![slot.clone()]);
    let selections = ClassSelections(HashMap::from([(p1, ClassId::Iop)]));
    let deadline = LobbyDeadline(90.0);
    let heartbeats = LobbyHeartbeats(HashMap::from([(p1, 12.5)]));
    let cancelled = SessionCancelled {
        reason: SessionCancelledReason::HeartbeatTimeout,
    };

    assert_eq!(slot.index, 0);
    assert_eq!(slot.player, Some(p1));
    assert_eq!(session_id.0, Uuid::nil());
    assert_eq!(room_code.0, "ABCDEF");
    assert_eq!(token, [7; 16]);
    assert_eq!(slots.0.len(), 1);
    assert_eq!(selections.0.get(&p1), Some(&ClassId::Iop));
    assert_eq!(deadline.0, 90.0);
    assert_eq!(heartbeats.0.get(&p1), Some(&12.5));
    assert_eq!(cancelled.reason, SessionCancelledReason::HeartbeatTimeout);
}

#[test]
fn test_lobby_state_waiting_and_active_are_distinct() {
    assert_ne!(LobbyState::LobbyWaiting, LobbyState::GameActive);
}

#[test]
fn test_build_session_config_valid_two_player_setup() {
    let p1 = player(1);
    let p2 = player(2);
    let slots = SessionSlots(vec![
        occupied_slot(0, 0, p1, ClassId::Iop),
        occupied_slot(1, 1, p2, ClassId::Cra),
    ]);
    let selections = ClassSelections(HashMap::from([(p1, ClassId::Iop), (p2, ClassId::Cra)]));

    let config = build_session_config(&slots, &selections);

    assert_eq!(config.mode, GameMode::OneVOne);
    assert_eq!(config.player_count, 2);
    assert_eq!(config.team_map.get(&p1), Some(&0));
    assert_eq!(config.team_map.get(&p2), Some(&1));
    assert_eq!(config.class_map.get(&p1), Some(&ClassId::Iop));
    assert_eq!(config.class_map.get(&p2), Some(&ClassId::Cra));
    assert_eq!(config.players().count(), 2);
}

#[test]
#[should_panic(expected = "slot 1")]
fn test_build_session_config_panics_when_occupied_slot_has_no_class() {
    let p1 = player(1);
    let slots = SessionSlots(vec![SessionSlot {
        index: 1,
        team: 0,
        player: Some(p1),
        class: None,
    }]);
    let selections = ClassSelections(HashMap::new());

    let _ = build_session_config(&slots, &selections);
}

#[test]
fn test_session_ready_is_zero_sized_observer_trigger() {
    assert_eq!(std::mem::size_of::<SessionReady>(), 0);
}

#[test]
fn test_session_ready_doc_comment_keeps_single_observer_registration_literal() {
    let source = include_str!("../src/core/session/events.rs");

    assert!(source.contains("app.observe(on_session_ready)"));
    assert!(!source.contains("EventReader<SessionReady>"));
}

#[test]
fn test_game_session_plugin_registers_cleanly() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GameSessionPlugin);

    app.update();
}

#[test]
fn test_session_config_resource_constructs_directly() {
    let p1 = player(1);
    let config = SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 1,
        team_map: HashMap::from([(p1, 0)]),
        class_map: HashMap::from([(p1, ClassId::Sacrier)]),
    };

    assert_eq!(config.players().collect::<Vec<_>>(), vec![p1]);
}

#[test]
fn test_session_config_players_iterate_in_ascending_player_id_order() {
    let p1 = player(1);
    let p2 = player(2);
    let config = SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(p2, 1), (p1, 0)]),
        class_map: HashMap::from([(p2, ClassId::Cra), (p1, ClassId::Iop)]),
    };

    assert_eq!(config.players().collect::<Vec<_>>(), vec![p1, p2]);
}
