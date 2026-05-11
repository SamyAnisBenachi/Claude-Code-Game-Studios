use std::collections::HashMap;

use bevy::prelude::*;
use server::core::rsm::{
    AuctionSettled, ResolutionComplete, RoundPhase, RoundState, RsmNetworkOutbox, RsmPlugin,
};
use server::core::session::{
    ClassSelections, GameSessionPlugin, LobbyDeadline, LobbyState, ServerRngFactory,
    ServerRngInitError, SessionConfig, SessionNetworkOutbox, SessionSlot, SessionSlots,
};
use server::foundation::rng::ServerRng;
use shared::card::ClassId;
use shared::protocol::SessionCancelledReason;
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn failing_rng() -> Result<ServerRng, ServerRngInitError> {
    Err(ServerRngInitError)
}

fn ready_app_with_failing_rng() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RsmPlugin);
    app.add_plugins(GameSessionPlugin);
    app.add_message::<AuctionSettled>();
    app.add_message::<ResolutionComplete>();
    app.insert_resource(server::foundation::config::GameConfig(
        shared::config::GameConfig::default(),
    ));
    app.insert_resource(ServerRngFactory::new(failing_rng));
    app.insert_resource(LobbyState::LobbyWaiting);
    app.insert_resource(SessionSlots(vec![
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
    ]));
    app.insert_resource(ClassSelections(HashMap::from([
        (player(1), ClassId::Iop),
        (player(2), ClassId::Cra),
    ])));
    app.insert_resource(LobbyDeadline(1.0));
    app
}

#[test]
fn test_rng_init_failure_cancels_lobby_and_does_not_trigger_session_ready() {
    let mut app = ready_app_with_failing_rng();

    app.update();

    assert!(!app.world().contains_resource::<SessionConfig>());
    assert!(!app.world().contains_resource::<ServerRng>());
    assert_eq!(
        *app.world().resource::<LobbyState>(),
        LobbyState::LobbyCancelled
    );
    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::Lobby
    );
    assert!(app
        .world()
        .resource::<RsmNetworkOutbox>()
        .phase_changed()
        .is_empty());

    let cancellations = app
        .world()
        .resource::<SessionNetworkOutbox>()
        .session_cancelled();
    assert_eq!(cancellations.len(), 1);
    assert_eq!(
        cancellations[0].reason,
        SessionCancelledReason::ServerRngFail
    );
}
