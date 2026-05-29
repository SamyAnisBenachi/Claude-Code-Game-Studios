// PROMPT 2056 — Placement drag/drop client-side legality gate.
//
// `handle_placement_drag_ended_system` previously accepted any in-bounds
// `(lane, cell)` resolved from the cursor world position. The server then
// rejected occupied / objective / out-of-spawn-range / already-staged cells
// with `PlacementRejectedReason::InvalidTarget`, after a round-trip during
// which the card visually committed. PROMPT 2056 adds a client-side gate
// that mirrors the green valid-cell highlight predicate
// (`minion_highlight_cells`) so an illegal release resolves to
// `target = None` and the card snaps straight back to the fan without
// reaching the server.
//
// Tests cover each rejection branch of the gate plus the happy path.

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::{
    hand::{
        BoardCellOccupied, BoardSpawnEdge, FanSlotIndex, HandCardCatalog, HandContents,
        HandUiPlacementCursorMoved, HandUiPlacementDragEnded, HandUiPlacementDragStarted,
        HandUiPlacementDropResolved, HandUiPlugin, ObjectiveCell, PendingPlacements,
        PlacementBoardView,
    },
    shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT},
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlacedCardSubmit, PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[test]
fn test_minion_drop_on_valid_spawn_cell_resolves_to_board_cell_target() {
    // Arrange — fresh board, spawn_range_cells=2 so cell (3, 1) is a valid
    // spawn cell with no occupant, no objective, no staged minion.
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(10), CardType::Minion)]));
    spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(10)]);
    app.world_mut()
        .resource_mut::<PlacementBoardView>()
        .spawn_range_cells = 2;
    start_drag(&mut app, 0, PlayerId(1));
    move_cursor_to_cell(&mut app, 3, 1);

    // Act — release.
    end_drag(&mut app);

    // Assert — drop message carries the resolved board cell.
    assert_eq!(
        drop_targets(&app),
        vec![Some(PlayTarget::BoardCell { lane: 3, cell: 1 })]
    );
}

#[test]
fn test_minion_drop_on_occupied_cell_resolves_to_none() {
    // Arrange — make cell (3, 1) occupied.
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(10), CardType::Minion)]));
    let board_cells = spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(10)]);
    app.world_mut()
        .resource_mut::<PlacementBoardView>()
        .spawn_range_cells = 2;
    app.world_mut()
        .entity_mut(board_cells[&(3, 1)])
        .insert(BoardCellOccupied);
    start_drag(&mut app, 0, PlayerId(1));
    move_cursor_to_cell(&mut app, 3, 1);

    // Act.
    end_drag(&mut app);

    // Assert — gate rejects the cell.
    assert_eq!(drop_targets(&app), vec![None]);
}

#[test]
fn test_minion_drop_on_objective_cell_resolves_to_none() {
    // Arrange — mark (3, 1) as an objective cell.
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(10), CardType::Minion)]));
    let board_cells = spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(10)]);
    app.world_mut()
        .resource_mut::<PlacementBoardView>()
        .spawn_range_cells = 2;
    app.world_mut()
        .entity_mut(board_cells[&(3, 1)])
        .insert(ObjectiveCell {
            player_id: PlayerId(1),
            lane: 3,
        });
    start_drag(&mut app, 0, PlayerId(1));
    move_cursor_to_cell(&mut app, 3, 1);

    // Act.
    end_drag(&mut app);

    // Assert.
    assert_eq!(drop_targets(&app), vec![None]);
}

#[test]
fn test_minion_drop_on_out_of_spawn_range_cell_resolves_to_none() {
    // Arrange — spawn_range_cells=1 means only cell index 1 is a spawn cell
    // for the LowCells edge. Cell index 3 must be rejected.
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(10), CardType::Minion)]));
    spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(10)]);
    app.world_mut()
        .resource_mut::<PlacementBoardView>()
        .spawn_range_cells = 1;
    start_drag(&mut app, 0, PlayerId(1));
    move_cursor_to_cell(&mut app, 3, 3);

    // Act.
    end_drag(&mut app);

    // Assert.
    assert_eq!(drop_targets(&app), vec![None]);
}

#[test]
fn test_minion_drop_on_already_staged_minion_cell_resolves_to_none() {
    // Arrange — a sibling Minion card already staged on (3, 1) this turn.
    let mut app = app_with_hand_ui_in_placement(test_catalog([
        (CardId(10), CardType::Minion),
        (CardId(20), CardType::Minion),
    ]));
    spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(10)]);
    app.world_mut()
        .resource_mut::<PlacementBoardView>()
        .spawn_range_cells = 2;
    app.world_mut()
        .resource_mut::<PendingPlacements>()
        .placements
        .push(PlacedCardSubmit {
            card_id: CardId(20),
            target: PlayTarget::BoardCell { lane: 3, cell: 1 },
            current_mana_spend: 0,
            reserve_mana_spend: 0,
        });
    start_drag(&mut app, 0, PlayerId(1));
    move_cursor_to_cell(&mut app, 3, 1);

    // Act.
    end_drag(&mut app);

    // Assert.
    assert_eq!(drop_targets(&app), vec![None]);
}

// ---------------------------------------------------------------------------
// Test harness — mirrors placement_drag_highlights_test.rs so the two suites
// stay drift-free as the hand UI plugin evolves.
// ---------------------------------------------------------------------------

fn app_with_hand_ui_in_placement(catalog: HashMap<CardId, CardData>) -> App {
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
    app.insert_resource(HandCardCatalog { cards: catalog });
    app.insert_resource(PlacementBoardView {
        local_player_id: PlayerId(1),
        opponent_player_id: PlayerId(2),
        spawn_edge: BoardSpawnEdge::LowCells,
        spawn_range_cells: 1,
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_phase(&mut app, RoundPhase::Placement);
    app.update();
    app
}

fn test_catalog<const N: usize>(entries: [(CardId, CardType); N]) -> HashMap<CardId, CardData> {
    entries
        .into_iter()
        .map(|(card_id, card_type)| (card_id, test_card(card_id, card_type)))
        .collect()
}

fn test_card(card_id: CardId, card_type: CardType) -> CardData {
    CardData {
        id: card_id,
        name_fr: format!("Carte {}", card_id.0),
        name_en: format!("Card {}", card_id.0),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type,
        unit_type: UnitType::Blade,
        cost: 1,
        atk: 1,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: format!("test_{}", card_id.0),
        pool_copies_override: None,
    }
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn set_hand<const N: usize>(app: &mut App, cards: [CardId; N]) {
    app.world_mut().resource_mut::<HandContents>().cards = cards.to_vec();
}

fn spawn_board_cells(app: &mut App) -> BTreeMap<(u8, u8), Entity> {
    (1..=BOARD_LANE_COUNT)
        .flat_map(|lane| (1..=BOARD_CELL_COUNT).map(move |cell| (lane, cell)))
        .map(|(lane, cell)| {
            let entity = app.world_mut().spawn(LaneCell { lane, cell }).id();
            ((lane, cell), entity)
        })
        .collect()
}

fn start_drag(app: &mut App, slot_index: u8, owner_id: PlayerId) {
    let slot = fan_slot(app, slot_index);
    app.world_mut().write_message(HandUiPlacementDragStarted {
        card: slot,
        owner_id,
    });
    app.update();
}

fn move_cursor_to_cell(app: &mut App, lane: u8, cell: u8) {
    let position = app
        .world()
        .resource::<BoardLayout>()
        .cell_to_world(lane, cell);
    app.world_mut().write_message(HandUiPlacementCursorMoved {
        world_position: Some(position),
        screen_position: None,
    });
    app.update();
}

fn end_drag(app: &mut App) {
    app.world_mut().write_message(HandUiPlacementDragEnded);
    app.update();
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}

fn drop_targets(app: &App) -> Vec<Option<PlayTarget>> {
    let messages = app
        .world()
        .resource::<Messages<HandUiPlacementDropResolved>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).map(|msg| msg.target.clone()).collect()
}
