use std::collections::{BTreeMap, BTreeSet, HashMap};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::{
    hand::{
        BoardCellHighlighted, BoardCellOccupied, BoardSpawnEdge, FanSlotIndex, FanSlotState,
        GhostPlacementChanged, HandCardCatalog, HandContents, HandPlacementTargetKind,
        HandUiPlacementCursorMoved, HandUiPlacementDragStarted, HandUiPlacementDropResolved,
        HandUiPlugin, NoValidTargetsOverlay, ObjectiveAlive, ObjectiveCell, PendingPlacements,
        PlacementBoardView, PlacementTargetKind, PlacementTargetUnit, TargetUnitHover,
    },
    shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT},
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{EntityId, PlacedCardSubmit, PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[test]
fn hu_12_minion_highlights_spawn_cells_minus_occupied_and_staged_minions() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([
        (CardId(10), CardType::Minion),
        (CardId(20), CardType::Minion),
    ]));
    let board_cells = spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(20)]);
    app.world_mut()
        .resource_mut::<PlacementBoardView>()
        .spawn_range_cells = 2;
    app.world_mut()
        .entity_mut(board_cells[&(1, 1)])
        .insert(BoardCellOccupied);
    app.world_mut()
        .resource_mut::<PendingPlacements>()
        .placements
        .push(PlacedCardSubmit {
            card_id: CardId(10),
            target: PlayTarget::BoardCell { lane: 1, cell: 2 },
            current_mana_spend: 0,
            reserve_mana_spend: 0,
        });
    app.update();

    start_drag(&mut app, 0, PlayerId(1));

    let expected = (1..=BOARD_LANE_COUNT)
        .flat_map(|lane| (1..=2).map(move |cell| (lane, cell)))
        .filter(|cell| !matches!(cell, (1, 1) | (1, 2)))
        .collect::<BTreeSet<_>>();
    assert_eq!(highlighted_lane_cells(&mut app), expected);
}

#[test]
fn hu_12b_target_obj_highlights_only_surviving_opponent_objectives() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(30), CardType::Spell)]));
    set_hand(&mut app, [CardId(30)]);
    set_slot_target_kind(&mut app, 0, PlacementTargetKind::TargetObj);
    spawn_objectives(&mut app, PlayerId(2), [true, true, false, true, true]);
    app.update();

    start_drag(&mut app, 0, PlayerId(1));

    assert_eq!(
        highlighted_objective_lanes(&mut app, PlayerId(2)),
        BTreeSet::from([1, 2, 4, 5])
    );
}

#[test]
fn hu_12c_lane_wide_highlights_all_non_objective_board_cells() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(40), CardType::Field)]));
    spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(40)]);
    app.update();

    start_drag(&mut app, 0, PlayerId(1));

    assert_eq!(highlighted_lane_cells(&mut app).len(), 40);
    assert_eq!(
        highlighted_lane_cells(&mut app),
        (1..=BOARD_LANE_COUNT)
            .flat_map(|lane| (1..=BOARD_CELL_COUNT).map(move |cell| (lane, cell)))
            .collect::<BTreeSet<_>>()
    );
}

#[test]
fn hu_12d_target_unit_hover_moves_between_units_without_cell_highlights() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(50), CardType::Spell)]));
    let board_cells = spawn_board_cells(&mut app);
    app.world_mut()
        .entity_mut(board_cells[&(1, 1)])
        .insert(BoardCellHighlighted);
    set_hand(&mut app, [CardId(50)]);
    set_slot_target_kind(&mut app, 0, PlacementTargetKind::TargetUnit);
    let unit_a = spawn_target_unit(&mut app, PlayerId(2), 100, 1, 1);
    let unit_b = spawn_target_unit(&mut app, PlayerId(2), 200, 3, 3);
    app.update();

    start_drag(&mut app, 0, PlayerId(1));
    move_cursor_to_cell(&mut app, 1, 1);

    assert!(app.world().get::<TargetUnitHover>(unit_a).is_some());
    assert!(app.world().get::<TargetUnitHover>(unit_b).is_none());
    assert_eq!(count_with::<BoardCellHighlighted>(&mut app), 0);

    move_cursor_to_cell(&mut app, 3, 3);

    assert!(app.world().get::<TargetUnitHover>(unit_a).is_none());
    assert!(app.world().get::<TargetUnitHover>(unit_b).is_some());
    assert_eq!(count_with::<BoardCellHighlighted>(&mut app), 0);
}

#[test]
fn hu_20_target_unit_without_valid_targets_shows_overlay_and_invalid_drop_cleans_up() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(60), CardType::Spell)]));
    spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(60)]);
    set_slot_target_kind(&mut app, 0, PlacementTargetKind::TargetUnit);
    app.update();

    start_drag(&mut app, 0, PlayerId(1));
    move_cursor_to_cell(&mut app, 1, 1);

    assert_eq!(count_with::<BoardCellHighlighted>(&mut app), 0);
    let overlay = overlay(&mut app);
    assert_eq!(
        app.world().get::<Visibility>(overlay),
        Some(&Visibility::Visible)
    );

    let slot = fan_slot(&mut app, 0);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id: PlayerId(1),
        target: None,
    });
    app.update();

    assert_eq!(
        app.world().get::<FanSlotState>(slot),
        Some(&FanSlotState::Active)
    );
    assert_eq!(
        app.world().get::<Visibility>(overlay),
        Some(&Visibility::Hidden)
    );
    assert!(ghost_messages(&app).is_empty());
    assert!(app
        .world()
        .resource::<PendingPlacements>()
        .placements
        .is_empty());
}

fn app_with_hand_ui_in_placement(catalog: HashMap<CardId, CardData>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
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

fn spawn_objectives(app: &mut App, player_id: PlayerId, alive: [bool; 5]) {
    for (index, alive) in alive.into_iter().enumerate() {
        let mut entity = app.world_mut().spawn(ObjectiveCell {
            player_id,
            lane: index as u8 + 1,
        });
        if alive {
            entity.insert(ObjectiveAlive);
        }
    }
}

fn spawn_target_unit(
    app: &mut App,
    owner_id: PlayerId,
    unit_id: EntityId,
    lane: u8,
    cell: u8,
) -> Entity {
    let position = app
        .world()
        .resource::<BoardLayout>()
        .cell_to_world(lane, cell);
    let transform = Transform::from_xyz(position.x, position.y, 0.0);
    app.world_mut()
        .spawn((
            PlacementTargetUnit { owner_id, unit_id },
            transform,
            GlobalTransform::from(transform),
        ))
        .id()
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
    });
    app.update();
}

fn set_slot_target_kind(app: &mut App, slot_index: u8, target_kind: PlacementTargetKind) {
    let slot = fan_slot(app, slot_index);
    app.world_mut()
        .entity_mut(slot)
        .insert(HandPlacementTargetKind(target_kind));
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}

fn overlay(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<NoValidTargetsOverlay>>();
    query.single(app.world()).expect("overlay should exist")
}

fn highlighted_lane_cells(app: &mut App) -> BTreeSet<(u8, u8)> {
    let mut query = app
        .world_mut()
        .query_filtered::<&LaneCell, With<BoardCellHighlighted>>();
    query
        .iter(app.world())
        .map(|lane_cell| (lane_cell.lane, lane_cell.cell))
        .collect()
}

fn highlighted_objective_lanes(app: &mut App, player_id: PlayerId) -> BTreeSet<u8> {
    let mut query = app
        .world_mut()
        .query_filtered::<&ObjectiveCell, With<BoardCellHighlighted>>();
    query
        .iter(app.world())
        .filter_map(|objective| (objective.player_id == player_id).then_some(objective.lane))
        .collect()
}

fn ghost_messages(app: &App) -> Vec<GhostPlacementChanged> {
    let messages = app.world().resource::<Messages<GhostPlacementChanged>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}
