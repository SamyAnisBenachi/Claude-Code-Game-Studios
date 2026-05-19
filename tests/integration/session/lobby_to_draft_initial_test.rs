use std::collections::HashMap;

use bevy::prelude::*;
use server::core::rsm::{
    AuctionSettled, ResolutionComplete, RoundPhase, RoundState, RsmNetworkOutbox, RsmPlugin,
};
use server::core::session::{
    ClassSelections, GameSessionPlugin, LobbyDeadline, LobbyState, SessionConfig, SessionSlot,
    SessionSlots,
};
use server::foundation::rng::ServerRng;
use shared::card::ClassId;
use shared::protocol;
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn full_session_app() -> App {
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
fn test_lobby_to_draft_initial_emits_phase_changed_same_update() {
    test_helpers::init_test_tracing();
    let mut app = full_session_app();

    app.update();

    assert!(app.world().contains_resource::<SessionConfig>());
    assert!(app.world().contains_resource::<ServerRng>());

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::DraftInitial);
    assert_eq!(rsm.round_number, 1);

    let phase_changes = app.world().resource::<RsmNetworkOutbox>().phase_changed();
    assert_eq!(phase_changes.len(), 1);
    assert_eq!(phase_changes[0].phase, protocol::RoundPhase::DraftInitial);
    assert_eq!(phase_changes[0].round_number, 1);
    assert_eq!(phase_changes[0].timer_duration_ms, 45_000);
}
