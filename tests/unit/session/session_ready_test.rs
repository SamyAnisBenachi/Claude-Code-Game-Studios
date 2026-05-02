use std::collections::HashMap;

use bevy::prelude::*;
use server::core::rsm::{RoundPhase, RoundState, RsmNetworkOutbox, RsmPlugin};
use server::core::session::{
    f4_session_ready, ClassSelections, GameSessionPlugin, LobbyDeadline, LobbyHeartbeats,
    LobbyState, SessionConfig, SessionSlot, SessionSlots,
};
use server::foundation::rng::ServerRng;
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn ready_slots() -> SessionSlots {
    SessionSlots(vec![
        SessionSlot {
            index: 0,
            team: 0,
            player: Some(player(1)),
            class: Some(ClassId::Iop),
        },
        SessionSlot {
            index: 1,
            team: 1,
            player: Some(player(2)),
            class: Some(ClassId::Cra),
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
