//! PROMPT 1086 — Client placement perspective + submit feedback repair.
//!
//! Pins the new `apply_placement_board_view_from_snapshot_system` and the
//! follow-on click-to-stage flow against the run-7 manual-test pattern:
//!
//! - Player A (team 0) sees `BoardSpawnEdge::LowCells` and the click-stage
//!   default target lands at `BoardCell { lane: 1, cell: 1 }`.
//! - Player B (team 1) — the case that fired the audit — sees
//!   `BoardSpawnEdge::HighCells` and the click-stage default target lands
//!   at `BoardCell { lane: 1, cell: 8 }`, i.e. the server's `player_b_spawn_cell`.
//!
//! Without the new system, `PlacementBoardView` stays pinned to its Default
//! (`LowCells`, range 1) and the default click target is always cell 1 —
//! the exact symptom observed in `client-b.log:169`:
//!     fan_active_default_drop card_id=CardId(103) default_target=BoardCell { lane: 1, cell: 1 }
//! for a Player B placement that the server later rejects with
//! `SpawnRangeRejected` (PROMPT 1079).

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::board_rendering::PlayerTeamMap;
use client::presentation::PresentationGameSnapshotMessage;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    BoardSpawnEdge, FanSlotIndex, HandCardCatalog, HandContents, HandFanCardClicked, HandUiPlugin,
    PendingPlacements, PlacementBoardView,
};
use client::ui::shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{
    BoardSnapshot, PlacementTimerMultiplier, PlayTarget, PlayerSnapshot, RoundPhase,
    S2CGameSnapshot,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const MINION_CARD_ID: CardId = CardId(103);

// ── apply_placement_board_view_from_snapshot_system ─────────────────────────

#[test]
fn snapshot_for_player_b_updates_placement_board_view_to_high_cells() {
    test_helpers::init_test_tracing();
    let mut app = app_for_snapshot_only(PlayerId(2), 1);

    write_minimal_snapshot(
        &mut app,
        PlayerId(2),
        1,
        &[(PlayerId(1), 0), (PlayerId(2), 1)],
    );
    app.update();

    let view = *app.world().resource::<PlacementBoardView>();
    assert_eq!(view.local_player_id, PlayerId(2));
    assert_eq!(view.opponent_player_id, PlayerId(1));
    assert_eq!(view.spawn_edge, BoardSpawnEdge::HighCells);
    assert_eq!(view.spawn_range_cells, 1);
}

#[test]
fn snapshot_for_player_a_keeps_placement_board_view_at_low_cells() {
    test_helpers::init_test_tracing();
    let mut app = app_for_snapshot_only(PlayerId(1), 0);

    write_minimal_snapshot(
        &mut app,
        PlayerId(1),
        1,
        &[(PlayerId(1), 0), (PlayerId(2), 1)],
    );
    app.update();

    let view = *app.world().resource::<PlacementBoardView>();
    assert_eq!(view.local_player_id, PlayerId(1));
    assert_eq!(view.opponent_player_id, PlayerId(2));
    assert_eq!(view.spawn_edge, BoardSpawnEdge::LowCells);
    assert_eq!(view.spawn_range_cells, 1);
}

#[test]
fn snapshot_spawn_range_cells_updates_after_destruction() {
    // Mirrors `spawn_range_cells_from_fakes_destroyed` advancing from 1 to 2
    // to 8: the client view must track those changes so subsequent
    // click-stage defaults pick the latest legal cell instead of the
    // round-start cell.
    test_helpers::init_test_tracing();
    let mut app = app_for_snapshot_only(PlayerId(2), 1);

    write_minimal_snapshot(
        &mut app,
        PlayerId(2),
        1,
        &[(PlayerId(1), 0), (PlayerId(2), 1)],
    );
    app.update();
    assert_eq!(
        app.world()
            .resource::<PlacementBoardView>()
            .spawn_range_cells,
        1
    );

    write_minimal_snapshot_with_range(
        &mut app,
        PlayerId(2),
        2,
        &[(PlayerId(1), 0), (PlayerId(2), 1)],
        2,
    );
    app.update();
    assert_eq!(
        app.world()
            .resource::<PlacementBoardView>()
            .spawn_range_cells,
        2
    );
}

// ── default click-to-stage target follows perspective ───────────────────────

#[test]
fn player_b_click_stage_default_target_lands_on_cell_eight() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_for_player(PlayerId(2), 1);

    let slot = fan_slot(&mut app, 0);
    app.world_mut()
        .write_message(HandFanCardClicked { card: slot });
    app.update();

    let placements = &app.world().resource::<PendingPlacements>().placements;
    assert_eq!(placements.len(), 1, "exactly one card should be staged");
    assert_eq!(placements[0].card_id, MINION_CARD_ID);
    assert_eq!(
        placements[0].target,
        PlayTarget::BoardCell { lane: 1, cell: 8 },
        "Player B's default click-stage target must land on the high-cell spawn row (server's player_b_spawn_cell == 8), \
         not on lane 1 cell 1 (Player A's spawn). Run-7 client-b.log:169 captured the broken default."
    );
}

#[test]
fn player_a_click_stage_default_target_lands_on_cell_one() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_for_player(PlayerId(1), 0);

    let slot = fan_slot(&mut app, 0);
    app.world_mut()
        .write_message(HandFanCardClicked { card: slot });
    app.update();

    let placements = &app.world().resource::<PendingPlacements>().placements;
    assert_eq!(placements.len(), 1);
    assert_eq!(placements[0].card_id, MINION_CARD_ID);
    assert_eq!(
        placements[0].target,
        PlayTarget::BoardCell { lane: 1, cell: 1 },
        "Player A's default click-stage target must land on the low-cell spawn row (server's player_a_spawn_cell == 1)."
    );
}

// ── App builders ────────────────────────────────────────────────────────────

fn app_for_snapshot_only(local_player_id: PlayerId, local_team: u8) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HandUiPlugin);
    seed_player_team_map(&mut app, &[(PlayerId(1), 0), (PlayerId(2), 1)]);
    let _ = (local_player_id, local_team);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn app_with_hand_ui_for_player(local_player_id: PlayerId, local_team: u8) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HandUiPlugin);
    app.insert_resource(BoardLayout {
        board_origin: Vec2::ZERO,
        cell_width: 64.0,
        lane_height: 80.0,
    });
    app.insert_resource(HandCardCatalog {
        cards: HashMap::from([(MINION_CARD_ID, minion_card())]),
    });
    seed_player_team_map(&mut app, &[(PlayerId(1), 0), (PlayerId(2), 1)]);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    spawn_board_cells(&mut app);
    set_hand(&mut app, &[MINION_CARD_ID]);
    write_minimal_snapshot(
        &mut app,
        local_player_id,
        local_team,
        &[(PlayerId(1), 0), (PlayerId(2), 1)],
    );
    set_phase(&mut app, RoundPhase::Placement);
    app.update();
    app
}

fn seed_player_team_map(app: &mut App, teams: &[(PlayerId, u8)]) {
    // Insert the resource explicitly so this test does not depend on the
    // plugin-order in which `BoardRenderingPlugin` registers `PlayerTeamMap`
    // — HandUiPlugin mirrors the registration but we want the test to fail
    // loudly if either side stops providing it.
    let mut map = PlayerTeamMap::default();
    for (player_id, team) in teams {
        map.insert(*player_id, *team);
    }
    app.insert_resource(map);
}

fn write_minimal_snapshot(
    app: &mut App,
    local_player_id: PlayerId,
    _local_team: u8,
    teams: &[(PlayerId, u8)],
) {
    write_minimal_snapshot_with_range(app, local_player_id, 1, teams, 1);
}

fn write_minimal_snapshot_with_range(
    app: &mut App,
    local_player_id: PlayerId,
    round_number: u32,
    teams: &[(PlayerId, u8)],
    spawn_range_cells: u8,
) {
    let players: Vec<PlayerSnapshot> = teams
        .iter()
        .map(|(player_id, _team)| PlayerSnapshot {
            player_id: *player_id,
            class_id: ClassId::Iop,
            gold: 0,
            reserved_gold: 0,
            current_mana: 10,
            reserve_mana: 0,
            spawn_range_cells,
            mana_cap: 10,
            submitted: false,
            hand: if *player_id == local_player_id {
                vec![MINION_CARD_ID]
            } else {
                Vec::new()
            },
            shop_slots: Vec::new(),
            pool_snapshot: Vec::new(),
            objectives: Vec::new(),
            opponent_objectives: Vec::new(),
        })
        .collect();

    let snapshot = S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: local_player_id,
        round_number,
        phase: RoundPhase::Placement,
        timer_remaining_ms: Some(10_000),
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players,
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    };

    app.world_mut()
        .write_message(PresentationGameSnapshotMessage(snapshot));
}

fn minion_card() -> CardData {
    CardData {
        id: MINION_CARD_ID,
        name_fr: "Coureur du Marché".to_string(),
        name_en: "Market Runner".to_string(),
        class: ClassId::Iop,
        family: Some("PROMPT-1086".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: 2,
        atk: 2,
        hp: 3,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: "market_runner".to_string(),
        pool_copies_override: None,
    }
}

fn spawn_board_cells(app: &mut App) {
    for lane in 1..=BOARD_LANE_COUNT {
        for cell in 1..=BOARD_CELL_COUNT {
            app.world_mut().spawn(LaneCell { lane, cell });
        }
    }
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn set_hand(app: &mut App, cards: &[CardId]) {
    app.world_mut().resource_mut::<HandContents>().cards = cards.to_vec();
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}
