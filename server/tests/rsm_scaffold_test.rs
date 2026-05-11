use bevy::prelude::*;
use server::core::rsm::{
    AbortAuction, AuctionPhaseEntered, AuctionSettled, BroadcastPhaseChanged, DraftStarted,
    GameOverEmitted, LobbyComplete, PlacementPhaseEntered, ResolutionComplete,
    ResolutionPhaseEntered, RoundPhase, RoundState, RsmPlugin, ShopRefreshTriggered,
};

#[test]
fn test_round_state_resource_initializes_to_lobby() {
    let mut app = App::new();
    app.init_resource::<RoundState>();

    let state = app.world().resource::<RoundState>();
    assert_eq!(state.phase, RoundPhase::Lobby);
    assert_eq!(state.round_number, 0);
    assert!(state.placement_timer.is_none());
    assert!(state.draft_shop_timer.is_none());
    assert!(state.draft_initial_timer.is_none());
    assert!(state.auction_safety_timer.is_none());
    assert!(state.resolution_safety_timer.is_none());
    assert!(state.submissions_received.is_empty());
    assert!(state.disconnect_trackers.is_empty());
}

#[test]
fn test_rsm_plugin_registers_round_state_and_messages() {
    let mut app = App::new();
    app.add_plugins(RsmPlugin);
    app.add_message::<AuctionSettled>();
    app.add_message::<ResolutionComplete>();
    app.finish();
    app.cleanup();

    let world = app.world();
    assert_eq!(world.resource::<RoundState>().phase, RoundPhase::Lobby);
    assert!(world.get_resource::<Messages<LobbyComplete>>().is_some());
    assert!(world.get_resource::<Messages<DraftStarted>>().is_some());
    assert!(world
        .get_resource::<Messages<ShopRefreshTriggered>>()
        .is_some());
    assert!(world
        .get_resource::<Messages<AuctionPhaseEntered>>()
        .is_some());
    assert!(world.get_resource::<Messages<AbortAuction>>().is_some());
    assert!(world
        .get_resource::<Messages<PlacementPhaseEntered>>()
        .is_some());
    assert!(world
        .get_resource::<Messages<ResolutionPhaseEntered>>()
        .is_some());
    assert!(world.get_resource::<Messages<GameOverEmitted>>().is_some());
    assert!(world
        .get_resource::<Messages<BroadcastPhaseChanged>>()
        .is_some());
    assert!(world.get_resource::<Messages<AuctionSettled>>().is_some());
    assert!(world
        .get_resource::<Messages<ResolutionComplete>>()
        .is_some());
}
