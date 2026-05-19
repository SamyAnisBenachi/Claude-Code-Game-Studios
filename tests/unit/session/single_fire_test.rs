use std::collections::HashMap;

use bevy::prelude::*;
use server::core::rsm::{
    AuctionSettled, ResolutionComplete, RoundPhase, RoundState, RsmNetworkOutbox, RsmPlugin,
};
use server::core::session::{
    ClassSelections, GameSessionPlugin, LobbyDeadline, LobbyState, SessionConfig, SessionSlot,
    SessionSlots,
};
use shared::card::ClassId;
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
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
    app.insert_resource(SessionSlots(vec![
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
    ]));
    app.insert_resource(ClassSelections(HashMap::from([
        (player(1), ClassId::Iop),
        (player(2), ClassId::Cra),
    ])));
    app.insert_resource(LobbyDeadline(1.0));
    app
}

#[test]
fn test_session_ready_fires_once_after_lobby_state_becomes_game_active() {
    let mut app = ready_app();

    app.update();
    assert_eq!(
        *app.world().resource::<LobbyState>(),
        LobbyState::GameActive
    );
    assert_eq!(
        app.world()
            .resource::<RsmNetworkOutbox>()
            .phase_changed()
            .len(),
        1
    );

    app.update();
    app.update();
    app.update();

    assert_eq!(
        app.world()
            .resource::<RsmNetworkOutbox>()
            .phase_changed()
            .len(),
        1,
        "SessionReady must not re-trigger after LobbyState::GameActive"
    );
    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::DraftInitial
    );
    assert!(app.world().contains_resource::<SessionConfig>());
}
