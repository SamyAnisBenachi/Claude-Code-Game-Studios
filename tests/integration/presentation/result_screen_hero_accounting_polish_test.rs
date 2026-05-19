//! PROMPT 1481 -- Result screen hero/accounting Krosmaga polish.
//!
//! Focused tests for the new polish primitives added on top of the existing
//! S9-RS-002 / S9-RS-003 two-step contract:
//!
//! - Outcome accent palette is distinct per outcome and read by the panel
//!   chrome.
//! - Round chip text + visibility track the cached result/snapshot.
//! - Resources line and ledger line surface in the accounting panel and
//!   carry the local player's gold/mana/objectives summary.
//! - Continue hint is mounted only on the hero step.
//!
//! These tests poke the presentation helpers and the spawned entity graph;
//! they do not touch the data contract verified by the MVP test.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::result_screen::{
    result_screen_ledger_line, result_screen_outcome_accent, result_screen_outcome_copy,
    result_screen_resources_line, result_screen_round_label, ResultScreenContinueHint,
    ResultScreenEntities, ResultScreenLedgerLine, ResultScreenPlugin, ResultScreenResourcesLine,
    ResultScreenRoundChip, ResultScreenStep, ResultScreenStepActionRequest, ResultScreenStepState,
    ResultScreenViewState,
};
use client::presentation::PresentationGameSnapshotMessage;
use client::state::{ClientSessionIdentity, ClientState, CurrentClientPhase};
use shared::card::ClassId;
use shared::protocol::{
    BoardSnapshot, GameOverReason, ObjectiveSnapshot, OpponentObjectiveSnapshot,
    PlacementTimerMultiplier, PlayerSnapshot, RoundPhase, S2CGameOver, S2CGameSnapshot,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn outcome_accent_is_distinct_per_outcome_class() {
    test_helpers::init_test_tracing();
    let local = player(1);
    let opponent = player(2);

    let victory_headline =
        result_screen_outcome_copy(Some(&result(Some(opponent), GameOverReason::ObjectivesDestroyed)), Some(local)).headline;
    let defeat_headline =
        result_screen_outcome_copy(Some(&result(Some(local), GameOverReason::ObjectivesDestroyed)), Some(local)).headline;
    let draw_headline =
        result_screen_outcome_copy(Some(&result(None, GameOverReason::Draw)), Some(local)).headline;
    let pending_headline = result_screen_outcome_copy(None, Some(local)).headline;
    let no_result_headline =
        result_screen_outcome_copy(Some(&result(None, GameOverReason::ResolutionTimeout)), Some(local)).headline;

    let victory = result_screen_outcome_accent(&victory_headline);
    let defeat = result_screen_outcome_accent(&defeat_headline);
    let draw = result_screen_outcome_accent(&draw_headline);
    let pending = result_screen_outcome_accent(&pending_headline);
    let no_result = result_screen_outcome_accent(&no_result_headline);

    // Victory / Defeat / Draw must read as three distinct outcome classes so
    // the accent stripe never collides between win and loss at a glance.
    assert_ne!(victory, defeat, "VICTORY and DEFEAT must use distinct accents");
    assert_ne!(victory, draw, "VICTORY and DRAW must use distinct accents");
    assert_ne!(defeat, draw, "DEFEAT and DRAW must use distinct accents");

    // Pending and NO RESULT collapse to the neutral accent — they are not
    // real outcomes and must not look like VICTORY/DEFEAT/DRAW.
    assert_eq!(pending, no_result, "PENDING and NO RESULT share the neutral accent");
    assert_ne!(pending, victory);
    assert_ne!(pending, defeat);
    assert_ne!(pending, draw);
}

#[test]
fn round_label_prefers_result_round_over_snapshot_round() {
    let snapshot = game_over_snapshot();
    assert_eq!(snapshot.round_number, 9);

    let mut result_at_round_9 = result(Some(player(2)), GameOverReason::ObjectivesDestroyed);
    result_at_round_9.round = 9;
    assert_eq!(
        result_screen_round_label(Some(&result_at_round_9), Some(&snapshot)),
        Some("Round 9".to_string())
    );

    assert_eq!(
        result_screen_round_label(None, Some(&snapshot)),
        Some("Round 9".to_string())
    );

    assert_eq!(result_screen_round_label(None, None), None);
}

#[test]
fn resources_line_surfaces_local_player_gold_mana_reserve() {
    let snapshot = game_over_snapshot();
    let line = result_screen_resources_line(Some(&snapshot))
        .expect("snapshot carries a local player entry");
    // Verify all three readouts surface; exact format is the contract that the
    // accounting panel renders.
    assert!(line.contains("Gold 8"), "gold readout missing: {line}");
    assert!(line.contains("Mana 6 / 10"), "mana readout missing: {line}");
    assert!(line.contains("Reserve 2"), "reserve readout missing: {line}");

    assert_eq!(result_screen_resources_line(None), None);
}

#[test]
fn ledger_line_chunks_own_and_opponent_real_fake_losses() {
    let snapshot = game_over_snapshot();
    let line = result_screen_ledger_line(Some(&snapshot));
    // 1 real, 1 fake on own side per fixture; 1 fake + 1 real revealed on
    // opponent side. The line must read as scannable "Objectives Lost" copy.
    assert!(line.contains("Objectives Lost"), "ledger missing header: {line}");
    assert!(line.contains("You:"), "ledger missing own label: {line}");
    assert!(line.contains("Opponent:"), "ledger missing opponent label: {line}");
    assert!(line.contains("1 real"), "ledger missing real count: {line}");
    assert!(line.contains("1 fake"), "ledger missing fake count: {line}");
}

#[test]
fn round_chip_mounts_visible_with_text_on_hero_step() {
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );

    let entities = *app.world().resource::<ResultScreenEntities>();
    assert_eq!(
        node_display(&mut app, entities.round_chip),
        Display::Flex,
        "round chip must mount when a round number is known"
    );
    let text = read_text(&mut app, entities.round_chip);
    assert!(text.contains("Round 9"), "round chip text wrong: {text}");

    assert_eq!(query_count::<ResultScreenRoundChip>(&mut app), 1);
}

#[test]
fn continue_hint_mounts_on_hero_then_hides_on_accounting() {
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );

    let entities = *app.world().resource::<ResultScreenEntities>();
    assert_eq!(
        node_display(&mut app, entities.continue_hint),
        Display::Flex,
        "continue hint reads on hero step"
    );
    assert_eq!(query_count::<ResultScreenContinueHint>(&mut app), 1);

    advance_to_accounting(&mut app);

    assert_eq!(
        node_display(&mut app, entities.continue_hint),
        Display::None,
        "continue hint hides once the user is on the accounting step"
    );
}

#[test]
fn resources_and_ledger_lines_render_on_accounting_panel() {
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );

    let entities = *app.world().resource::<ResultScreenEntities>();
    let resources_text = read_text(&mut app, entities.resources_line);
    assert!(
        resources_text.contains("Gold 8") && resources_text.contains("Mana 6 / 10"),
        "resources line missing readouts: {resources_text}"
    );

    let ledger_text = read_text(&mut app, entities.ledger_line);
    assert!(
        ledger_text.contains("Objectives Lost"),
        "ledger line missing copy: {ledger_text}"
    );

    assert_eq!(query_count::<ResultScreenResourcesLine>(&mut app), 1);
    assert_eq!(query_count::<ResultScreenLedgerLine>(&mut app), 1);
}

fn read_text(app: &mut App, entity: Entity) -> String {
    app.world_mut()
        .get::<Text>(entity)
        .map(|text| text.0.clone())
        .unwrap_or_default()
}

fn query_count<C: Component>(app: &mut App) -> usize {
    app.world_mut().query::<&C>().iter(app.world()).count()
}

fn node_display(app: &mut App, entity: Entity) -> Display {
    app.world_mut()
        .get::<Node>(entity)
        .expect("result screen entity must have a Node")
        .display
}

fn result_screen_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ResultScreenPlugin);
    app.init_resource::<ButtonInput<KeyCode>>();
    app.world_mut()
        .resource_mut::<ClientSessionIdentity>()
        .player_id = Some(player(1));
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn open_result_screen(app: &mut App, result: Option<S2CGameOver>) {
    app.world_mut()
        .write_message(PresentationGameSnapshotMessage(game_over_snapshot()));
    {
        let mut view_state = app.world_mut().resource_mut::<ResultScreenViewState>();
        view_state.cached_result = result;
    }
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = RoundPhase::GameOver;
    app.update();
    app.update();
}

fn advance_to_accounting(app: &mut App) {
    app.world_mut()
        .write_message(ResultScreenStepActionRequest::AdvanceToAccounting);
    app.update();
    assert_eq!(
        app.world().resource::<ResultScreenStepState>().current,
        ResultScreenStep::Accounting
    );
}

fn result(loser: Option<PlayerId>, reason: GameOverReason) -> S2CGameOver {
    S2CGameOver {
        loser,
        round: 9,
        reason,
    }
}

fn game_over_snapshot() -> S2CGameSnapshot {
    let mut own = player_snapshot(player(1), 8, 6, 2, 10);
    own.objectives = vec![
        objective(1, 3, true, true),
        objective(2, 0, false, true),
        objective(3, 5, true, false),
        objective(4, 4, false, false),
        objective(5, 7, true, false),
    ];
    own.opponent_objectives = vec![
        opponent_objective(1, 6, false, None),
        opponent_objective(2, 0, true, Some(true)),
        opponent_objective(3, 0, true, Some(false)),
        opponent_objective(4, 5, false, None),
    ];

    let mut opponent = player_snapshot(player(2), 11, 4, 1, 10);
    opponent.objectives = vec![
        objective(1, 6, true, false),
        objective(2, 0, false, true),
        objective(3, 0, true, true),
        objective(4, 5, false, false),
        objective(5, 4, true, false),
    ];

    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: own.player_id,
        round_number: 9,
        phase: RoundPhase::GameOver,
        timer_remaining_ms: None,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players: vec![own, opponent],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn player_snapshot(
    player_id: PlayerId,
    gold: u32,
    current_mana: u32,
    reserve_mana: u32,
    mana_cap: u8,
) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id: ClassId::Iop,
        gold,
        reserved_gold: 0,
        current_mana,
        reserve_mana,
        spawn_range_cells: 1,
        mana_cap,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: Vec::new(),
        opponent_objectives: Vec::new(),
    }
}

fn objective(lane: u8, hp: u8, is_real: bool, is_destroyed: bool) -> ObjectiveSnapshot {
    ObjectiveSnapshot {
        lane,
        hp,
        is_real,
        is_destroyed,
    }
}

fn opponent_objective(
    lane: u8,
    hp: u8,
    is_destroyed: bool,
    was_fake: Option<bool>,
) -> OpponentObjectiveSnapshot {
    OpponentObjectiveSnapshot {
        lane,
        hp,
        is_destroyed,
        was_fake,
    }
}
