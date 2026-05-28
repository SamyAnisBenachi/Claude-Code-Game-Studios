use std::collections::HashMap;

use bevy::prelude::*;
use server::core::rsm::{
    advance_phase, AuctionPhaseEntered, BeginResolution, BroadcastPhaseChanged, DraftStarted,
    GameOverEmitted, LobbyComplete, PhaseAdvanceRequest, PlacementPhaseEntered,
    ResolutionPhaseEntered, RoundPhase, RoundState, ShopRefreshTrigger, ShopRefreshTriggered,
};
use server::core::session::{PlayerSessionData, PlayerSessions, SessionConfig};
use server::foundation::config::GameConfig;
use shared::card::ClassId;
use shared::protocol::{DraftPhase, GameMode, GameOverReason};
use shared::session::PlayerId;

fn session_config(players: &[PlayerId]) -> SessionConfig {
    let mut team_map = HashMap::new();
    let mut class_map = HashMap::new();

    for (index, player) in players.iter().copied().enumerate() {
        team_map.insert(player, index as u8);
        class_map.insert(player, ClassId::Iop);
    }

    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: players.len() as u8,
        team_map,
        class_map,
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn player_sessions(players: &[PlayerId]) -> PlayerSessions {
    let mut sessions = PlayerSessions::default();
    for (index, player) in players.iter().copied().enumerate() {
        sessions.players.insert(
            player,
            PlayerSessionData {
                class: if index == 0 {
                    ClassId::Iop
                } else {
                    ClassId::Cra
                },
                class_locked: false,
            },
        );
    }
    sessions
}

fn test_app(phase: RoundPhase, round_number: u32) -> App {
    let mut app = App::new();
    let players = [PlayerId(1), PlayerId(2)];
    app.add_message::<LobbyComplete>()
        .add_message::<DraftStarted>()
        .add_message::<ShopRefreshTriggered>()
        .add_message::<AuctionPhaseEntered>()
        .add_message::<PlacementPhaseEntered>()
        .add_message::<ResolutionPhaseEntered>()
        .add_message::<BeginResolution>()
        .add_message::<GameOverEmitted>()
        .add_message::<BroadcastPhaseChanged>()
        .insert_resource(RoundState {
            phase,
            round_number,
            ..RoundState::new()
        })
        .insert_resource(session_config(&players))
        .insert_resource(player_sessions(&players))
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
    let lobby_complete = read_messages::<LobbyComplete>(&app);
    let refreshes = read_messages::<ShopRefreshTriggered>(&app);
    let auctions = read_messages::<AuctionPhaseEntered>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(lobby_complete.len(), 1);
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].round, 1);
    assert_eq!(drafts[0].phase, DraftPhase::Initial);
    assert_eq!(
        refreshes.iter().map(|e| e.player_id).collect::<Vec<_>>(),
        vec![PlayerId(1), PlayerId(2)]
    );
    assert!(refreshes
        .iter()
        .all(|e| e.trigger == ShopRefreshTrigger::DraftInitial));
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
    let refreshes = read_messages::<ShopRefreshTriggered>(&app);
    let auctions = read_messages::<AuctionPhaseEntered>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(rsm.round_number, 3);
    assert_eq!(rsm.phase, RoundPhase::DraftAuction);
    assert_eq!(drafts[0].round, 3);
    assert_eq!(drafts[0].phase, DraftPhase::Auction);
    assert_eq!(refreshes.len(), 2);
    assert!(refreshes
        .iter()
        .all(|e| e.trigger == ShopRefreshTrigger::AuctionLock));
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
    let refreshes = read_messages::<ShopRefreshTriggered>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(rsm.phase, RoundPhase::DraftShop);
    assert_eq!(drafts[0].phase, DraftPhase::Shop);
    assert_eq!(refreshes.len(), 2);
    assert!(refreshes
        .iter()
        .all(|e| e.trigger == ShopRefreshTrigger::ShopUnlock));
    assert_eq!(broadcasts[0].phase, RoundPhase::DraftShop);
    assert_eq!(broadcasts[0].timer_ms, 30_000);
}

#[test]
fn rsm_transitions_draft_entry_shop_refresh_fans_out_once_per_player() {
    let mut app = test_app(RoundPhase::Resolution, 3);
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::Resolution));

    app.update();

    let refreshes = read_messages::<ShopRefreshTriggered>(&app);
    assert_eq!(refreshes.len(), 2);
    assert_ne!(refreshes[0].player_id, refreshes[1].player_id);
    assert!(refreshes
        .iter()
        .all(|e| e.trigger == ShopRefreshTrigger::ShopOpen));
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
    let begin_resolution = read_messages::<BeginResolution>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(rsm.phase, RoundPhase::Resolution);
    assert_eq!(resolutions.len(), 1);
    assert_eq!(resolutions[0].round, 2);
    assert_eq!(begin_resolution, vec![BeginResolution { round: 2 }]);
    assert_eq!(broadcasts[0].phase, RoundPhase::Resolution);
    assert_eq!(broadcasts[0].timer_ms, 0);
    assert!(rsm.resolution_safety_timer.is_some());
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

// ── Regression tests for PROMPT-2033 vacuous-flow P0 bugs ────────────────────

/// BUG-17 regression: submissions_received must be cleared when Resolution
/// advances to DraftShop so round-N+1 placement cannot falsely detect
/// all-players-submitted from stale round-N data.
#[test]
fn test_rsm_resolution_to_draftshop_clears_submissions_received() {
    // Arrange: round 1 Resolution with player 1's submission still present.
    let mut app = test_app(RoundPhase::Resolution, 1);
    {
        let mut rsm = app.world_mut().resource_mut::<RoundState>();
        rsm.submissions_received.insert(PlayerId(1));
    }
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::Resolution));

    // Act: advance Resolution → DraftShop (round 2 is not an auction round).
    app.update();

    // Assert: submissions_received is empty; phase advanced; round incremented.
    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::DraftShop);
    assert_eq!(rsm.round_number, 2);
    assert!(
        rsm.submissions_received.is_empty(),
        "BUG-17: submissions_received from round 1 must be cleared on Resolution->DraftShop"
    );
}

/// BUG-17 regression (auction path): same invariant must hold when Resolution
/// advances to DraftAuction (round 2 → round 3 would be auction).
#[test]
fn test_rsm_resolution_to_draftauction_clears_submissions_received() {
    // Arrange: round 2 Resolution (next round = 3, which is an auction round).
    let mut app = test_app(RoundPhase::Resolution, 2);
    {
        let mut rsm = app.world_mut().resource_mut::<RoundState>();
        rsm.submissions_received.insert(PlayerId(1));
        rsm.submissions_received.insert(PlayerId(2));
    }
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::Resolution));

    // Act: advance Resolution → DraftAuction (round 3).
    app.update();

    // Assert: submissions_received cleared; phase is DraftAuction.
    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::DraftAuction);
    assert_eq!(rsm.round_number, 3);
    assert!(
        rsm.submissions_received.is_empty(),
        "BUG-17: submissions_received must be cleared on Resolution->DraftAuction"
    );
}

/// BUG-05 regression: a normal (non-game-over) advance from Resolution must
/// not emit GameOverEmitted — it should advance to the next DraftShop round.
/// This represents the path taken by rsm_input_reader when
/// evaluate_objective_win_condition returns None (all objectives intact).
#[test]
fn test_rsm_resolution_normal_advance_does_not_emit_game_over() {
    // Arrange: Resolution phase, round 1; PhaseAdvanceRequest carries no game-over.
    let mut app = test_app(RoundPhase::Resolution, 1);
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::Resolution));

    // Act: advance_phase processes the non-game-over Resolution request.
    app.update();

    // Assert: game proceeds to DraftShop — NOT GameOver — and GameOverEmitted
    // is absent, proving the vacuous-GameOver path requires an explicit
    // game_over field in PhaseAdvanceRequest.
    let rsm = app.world().resource::<RoundState>();
    assert_eq!(
        rsm.phase,
        RoundPhase::DraftShop,
        "BUG-05: Resolution with no game-over request must advance to DraftShop"
    );
    let game_over = read_messages::<GameOverEmitted>(&app);
    assert!(
        game_over.is_empty(),
        "BUG-05: GameOverEmitted must not fire on normal Resolution advance"
    );
}
