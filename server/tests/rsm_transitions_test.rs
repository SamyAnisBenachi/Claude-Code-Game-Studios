use bevy::prelude::*;
use server::core::rsm::{
    advance_phase, AuctionPhaseEntered, BroadcastPhaseChanged, DraftStarted, GameOverEmitted,
    PhaseAdvanceRequest, PlacementPhaseEntered, ResolutionPhaseEntered, RoundPhase, RoundState,
    SessionConfig, ShopRefreshNeeded,
};
use server::foundation::config::GameConfig;
use shared::protocol::{DraftPhase, GameOverReason};
use shared::session::PlayerId;

fn test_app(phase: RoundPhase, round_number: u32) -> App {
    let mut app = App::new();
    app.add_message::<DraftStarted>()
        .add_message::<ShopRefreshNeeded>()
        .add_message::<AuctionPhaseEntered>()
        .add_message::<PlacementPhaseEntered>()
        .add_message::<ResolutionPhaseEntered>()
        .add_message::<GameOverEmitted>()
        .add_message::<BroadcastPhaseChanged>()
        .insert_resource(RoundState {
            phase,
            round_number,
            ..RoundState::new()
        })
        .insert_resource(SessionConfig::new(vec![PlayerId(1), PlayerId(2)]))
        .insert_resource(GameConfig(shared::config::GameConfig::default()))
        .add_systems(Update, advance_phase);
    app
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

#[test]
fn rsm_transitions_is_auction_round_matches_f1() {
    let auction_rounds: Vec<u32> = (1..=12)
        .filter(|round| server::core::rsm::is_auction_round(*round))
        .collect();

    assert_eq!(auction_rounds, vec![3, 6, 9, 12]);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "round_number must be initialized before auction routing")]
fn rsm_transitions_is_auction_round_rejects_zero_in_debug() {
    let _ = server::core::rsm::is_auction_round(0);
}

#[test]
fn rsm_transitions_lobby_to_draft_initial_emits_f2_order_payloads() {
    let mut app = test_app(RoundPhase::Lobby, 1);
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::Lobby));

    app.update();

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::DraftInitial);
    assert_eq!(rsm.round_number, 1);

    let drafts = read_messages::<DraftStarted>(&app);
    let refreshes = read_messages::<ShopRefreshNeeded>(&app);
    let auctions = read_messages::<AuctionPhaseEntered>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].round, 1);
    assert_eq!(drafts[0].phase, DraftPhase::Initial);
    assert_eq!(
        refreshes.iter().map(|e| e.player).collect::<Vec<_>>(),
        vec![PlayerId(1), PlayerId(2)]
    );
    assert!(auctions.is_empty());
    assert_eq!(broadcasts.len(), 1);
    assert_eq!(broadcasts[0].phase, RoundPhase::DraftInitial);
    assert_eq!(broadcasts[0].timer_ms, 45_000);
}

#[test]
fn rsm_transitions_resolution_to_draft_shop_increments_before_draft_started() {
    let mut app = test_app(RoundPhase::Resolution, 1);
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::Resolution));

    app.update();

    let rsm = app.world().resource::<RoundState>();
    let drafts = read_messages::<DraftStarted>(&app);
    let auctions = read_messages::<AuctionPhaseEntered>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(rsm.round_number, 2);
    assert_eq!(rsm.phase, RoundPhase::DraftShop);
    assert_eq!(drafts[0].round, rsm.round_number);
    assert_eq!(drafts[0].phase, DraftPhase::Shop);
    assert!(auctions.is_empty());
    assert_eq!(broadcasts[0].phase, RoundPhase::DraftShop);
    assert_eq!(broadcasts[0].timer_ms, 30_000);
}

#[test]
fn rsm_transitions_resolution_to_draft_auction_emits_auction_before_broadcast() {
    let mut app = test_app(RoundPhase::Resolution, 2);
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::Resolution));

    app.update();

    let rsm = app.world().resource::<RoundState>();
    let drafts = read_messages::<DraftStarted>(&app);
    let refreshes = read_messages::<ShopRefreshNeeded>(&app);
    let auctions = read_messages::<AuctionPhaseEntered>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(rsm.round_number, 3);
    assert_eq!(rsm.phase, RoundPhase::DraftAuction);
    assert_eq!(drafts[0].round, 3);
    assert_eq!(drafts[0].phase, DraftPhase::Auction);
    assert_eq!(refreshes.len(), 2);
    assert_eq!(auctions.len(), 1);
    assert_eq!(auctions[0].round, 3);
    assert_eq!(broadcasts[0].phase, RoundPhase::DraftAuction);
    assert_eq!(broadcasts[0].timer_ms, 0);
}

#[test]
fn rsm_transitions_draft_auction_to_draft_shop_emits_shop_entry() {
    let mut app = test_app(RoundPhase::DraftAuction, 3);
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::DraftAuction));

    app.update();

    let rsm = app.world().resource::<RoundState>();
    let drafts = read_messages::<DraftStarted>(&app);
    let refreshes = read_messages::<ShopRefreshNeeded>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(rsm.phase, RoundPhase::DraftShop);
    assert_eq!(drafts[0].phase, DraftPhase::Shop);
    assert_eq!(refreshes.len(), 2);
    assert_eq!(broadcasts[0].phase, RoundPhase::DraftShop);
    assert_eq!(broadcasts[0].timer_ms, 30_000);
}

#[test]
fn rsm_transitions_draft_entry_shop_refresh_fans_out_once_per_player() {
    let mut app = test_app(RoundPhase::Resolution, 3);
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::Resolution));

    app.update();

    let refreshes = read_messages::<ShopRefreshNeeded>(&app);
    assert_eq!(refreshes.len(), 2);
    assert_ne!(refreshes[0].player, refreshes[1].player);
}

#[test]
fn rsm_transitions_draft_initial_to_placement_clears_submissions_and_broadcasts_last_payload() {
    let mut app = test_app(RoundPhase::DraftInitial, 1);
    {
        let mut rsm = app.world_mut().resource_mut::<RoundState>();
        rsm.submissions_received.insert(PlayerId(1));
    }
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::DraftInitial));

    app.update();

    let rsm = app.world().resource::<RoundState>();
    let placements = read_messages::<PlacementPhaseEntered>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert!(rsm.submissions_received.is_empty());
    assert_eq!(placements.len(), 1);
    assert_eq!(placements[0].round, 1);
    assert_eq!(broadcasts[0].phase, RoundPhase::Placement);
    assert_eq!(broadcasts[0].timer_ms, 10_000);
}

#[test]
fn rsm_transitions_draft_shop_to_placement_clears_submissions() {
    let mut app = test_app(RoundPhase::DraftShop, 2);
    {
        let mut rsm = app.world_mut().resource_mut::<RoundState>();
        rsm.submissions_received.insert(PlayerId(2));
    }
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::DraftShop));

    app.update();

    let rsm = app.world().resource::<RoundState>();
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert!(rsm.submissions_received.is_empty());
    assert_eq!(broadcasts[0].phase, RoundPhase::Placement);
}

#[test]
fn rsm_transitions_placement_to_resolution_emits_resolution_then_broadcast_payload() {
    let mut app = test_app(RoundPhase::Placement, 2);
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::Placement));

    app.update();

    let rsm = app.world().resource::<RoundState>();
    let resolutions = read_messages::<ResolutionPhaseEntered>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(rsm.phase, RoundPhase::Resolution);
    assert_eq!(resolutions.len(), 1);
    assert_eq!(resolutions[0].round, 2);
    assert_eq!(broadcasts[0].phase, RoundPhase::Resolution);
    assert_eq!(broadcasts[0].timer_ms, 60_000);
}

#[test]
fn rsm_transitions_game_over_entry_emits_game_over_then_zero_timer_broadcast() {
    let mut app = test_app(RoundPhase::Resolution, 5);
    app.insert_resource(PhaseAdvanceRequest::game_over(
        RoundPhase::Resolution,
        GameOverReason::ObjectivesDestroyed,
        Some(PlayerId(2)),
    ));

    app.update();

    let rsm = app.world().resource::<RoundState>();
    let game_over = read_messages::<GameOverEmitted>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(rsm.phase, RoundPhase::GameOver);
    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::ObjectivesDestroyed);
    assert_eq!(game_over[0].loser, Some(PlayerId(2)));
    assert_eq!(broadcasts[0].phase, RoundPhase::GameOver);
    assert_eq!(broadcasts[0].timer_ms, 0);
}

#[test]
fn rsm_transitions_game_over_source_is_terminal_noop() {
    let mut app = test_app(RoundPhase::GameOver, 5);
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::GameOver));

    app.update();

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::GameOver);
    assert!(read_messages::<BroadcastPhaseChanged>(&app).is_empty());
    assert!(read_messages::<GameOverEmitted>(&app).is_empty());
}

#[test]
fn rsm_transitions_double_transition_guard_noops_after_first_advance() {
    let mut app = test_app(RoundPhase::Placement, 2);
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::Placement));

    app.update();
    app.update();

    let rsm = app.world().resource::<RoundState>();
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(rsm.phase, RoundPhase::Resolution);
    assert_eq!(broadcasts.len(), 1);
}

#[test]
fn rsm_transitions_wrong_expected_source_silently_noops() {
    let mut app = test_app(RoundPhase::Placement, 2);
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::DraftShop));

    app.update();

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert!(read_messages::<BroadcastPhaseChanged>(&app).is_empty());
}
