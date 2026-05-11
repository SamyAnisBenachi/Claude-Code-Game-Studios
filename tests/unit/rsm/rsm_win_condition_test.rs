use bevy::prelude::*;
use server::core::objective_contract::ObjectiveCounters;
use server::core::rsm::{
    AuctionSettled, BroadcastPhaseChanged, DraftStarted, GameOverEmitted, PhaseAdvanceRequest,
    ResolutionComplete, RoundPhase, RoundState, RsmPlugin,
};
use server::core::session::SessionConfig;
use server::foundation::config::GameConfig;
use shared::card::ClassId;
use shared::protocol::{DraftPhase, GameMode, GameOverReason};
use shared::session::PlayerId;
use std::collections::HashMap;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

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

fn app_with_resolution_state(round_number: u32, destroyed: &[(PlayerId, u32)]) -> App {
    let players = [player(1), player(2)];
    let mut app = App::new();
    app.add_plugins(RsmPlugin);
    app.add_message::<AuctionSettled>();
    app.add_message::<ResolutionComplete>();
    app.insert_resource(GameConfig(shared::config::GameConfig::default()));
    app.insert_resource(session_config(&players));
    app.insert_resource(Time::<()>::default());
    *app.world_mut().resource_mut::<RoundState>() = RoundState {
        phase: RoundPhase::Resolution,
        round_number,
        ..RoundState::new()
    };
    app.insert_resource(ObjectiveCounters {
        real_destroyed: destroyed.iter().copied().collect(),
        fake_destroyed: Default::default(),
    });
    app
}

fn process_resolution_complete(app: &mut App) {
    app.world_mut().write_message(ResolutionComplete);
    app.update();
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

#[test]
fn rsm_win_condition_single_loser_emits_objectives_destroyed_game_over() {
    let mut app = app_with_resolution_state(5, &[(player(1), 2), (player(2), 0)]);

    process_resolution_complete(&mut app);

    let rsm = app.world().resource::<RoundState>();
    let game_over = read_messages::<GameOverEmitted>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert_eq!(rsm.phase, RoundPhase::GameOver);
    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::ObjectivesDestroyed);
    assert_eq!(game_over[0].loser, Some(player(1)));
    assert_eq!(broadcasts.len(), 1);
    assert_eq!(broadcasts[0].phase, RoundPhase::GameOver);
    assert_eq!(broadcasts[0].timer_ms, 0);
}

#[test]
fn rsm_win_condition_above_threshold_keeps_single_loser_path() {
    let mut app = app_with_resolution_state(5, &[(player(1), 0), (player(2), 3)]);

    process_resolution_complete(&mut app);

    let game_over = read_messages::<GameOverEmitted>(&app);

    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::ObjectivesDestroyed);
    assert_eq!(game_over[0].loser, Some(player(2)));
}

#[test]
fn rsm_win_condition_no_loss_advances_to_next_draft_after_round_increment() {
    let mut app = app_with_resolution_state(2, &[(player(1), 1), (player(2), 0)]);

    process_resolution_complete(&mut app);

    let rsm = app.world().resource::<RoundState>();
    let game_over = read_messages::<GameOverEmitted>(&app);
    let drafts = read_messages::<DraftStarted>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);

    assert!(game_over.is_empty());
    assert_eq!(rsm.round_number, 3);
    assert_eq!(rsm.phase, RoundPhase::DraftAuction);
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].round, 3);
    assert_eq!(drafts[0].phase, DraftPhase::Auction);
    assert_eq!(broadcasts.len(), 1);
    assert_eq!(broadcasts[0].phase, RoundPhase::DraftAuction);
}

#[test]
fn rsm_win_condition_no_loss_below_threshold_for_both_players() {
    let mut app = app_with_resolution_state(1, &[(player(1), 1), (player(2), 1)]);

    process_resolution_complete(&mut app);

    let rsm = app.world().resource::<RoundState>();

    assert_eq!(rsm.round_number, 2);
    assert_eq!(rsm.phase, RoundPhase::DraftShop);
    assert!(read_messages::<GameOverEmitted>(&app).is_empty());
}

#[test]
fn rsm_win_condition_mutual_destruction_emits_one_draw_without_loser() {
    let mut app = app_with_resolution_state(5, &[(player(1), 2), (player(2), 2)]);

    process_resolution_complete(&mut app);

    let rsm = app.world().resource::<RoundState>();
    let game_over = read_messages::<GameOverEmitted>(&app);

    assert_eq!(rsm.phase, RoundPhase::GameOver);
    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::Draw);
    assert_eq!(game_over[0].loser, None);
}

#[test]
fn rsm_win_condition_mutual_destruction_draws_even_with_uneven_counts() {
    let mut app = app_with_resolution_state(5, &[(player(1), 3), (player(2), 2)]);

    process_resolution_complete(&mut app);

    let game_over = read_messages::<GameOverEmitted>(&app);

    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::Draw);
    assert_eq!(game_over[0].loser, None);
}

#[test]
fn rsm_win_condition_direct_game_over_request_preserves_non_objective_reason() {
    let mut app = app_with_resolution_state(5, &[]);
    app.insert_resource(PhaseAdvanceRequest::game_over(
        RoundPhase::Resolution,
        GameOverReason::ResolutionTimeout,
        None,
    ));

    app.update();

    let game_over = read_messages::<GameOverEmitted>(&app);

    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::ResolutionTimeout);
    assert_eq!(game_over[0].loser, None);
}
