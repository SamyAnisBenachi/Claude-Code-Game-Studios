//! PROMPT 697 — drag-ended Instant-only gate fix regression coverage.
//!
//! Before PROMPT 697, `handle_placement_drag_ended_system` short-circuited the
//! `HandUiPlacementDropResolved` write for every `PlacementTargetKind` other
//! than `Instant`. `pending_placements.stage_or_update` was therefore never
//! invoked through the drag pipeline for `Minion`, `LaneWide`, `TargetObj`, or
//! `TargetUnit` cards. These tests drive each non-Instant kind through the
//! drag pipeline and assert (a) the drop message is emitted with the resolved
//! board target and (b) the placement reaches `PendingPlacements`.

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::{
    hand::{
        BoardSpawnEdge, FanSlotIndex, HandCardCatalog, HandContents, HandPlacementTargetKind,
        HandUiPlacementCursorMoved, HandUiPlacementDragEnded, HandUiPlacementDragStarted,
        HandUiPlacementDropResolved, HandUiPlugin, ObjectiveAlive, ObjectiveCell,
        PendingPlacements, PlacementBoardView, PlacementTargetKind, PlacementTargetUnit,
    },
    shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT},
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{EntityId, PlayTarget, RoundPhase};
use shared::session::PlayerId;

const DROP_LANE: u8 = 2;
const DROP_CELL: u8 = 4;

#[test]
fn drag_end_minion_drops_on_board_cell_and_stages_placement() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(10), CardType::Minion)]));
    spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(10)]);
    app.update();

    let slot = fan_slot(&mut app, 0);
    start_drag(&mut app, slot, PlayerId(7));
    move_cursor_to_cell(&mut app, DROP_LANE, DROP_CELL);
    end_drag(&mut app);

    assert_eq!(
        drops_for_card(&app, slot),
        vec![HandUiPlacementDropResolved {
            card: slot,
            owner_id: PlayerId(7),
            target: Some(PlayTarget::BoardCell {
                lane: DROP_LANE,
                cell: DROP_CELL,
            }),
        }],
        "Minion drag-end on a valid board cell must emit a BoardCell drop",
    );
    let pending = &app.world().resource::<PendingPlacements>().placements;
    assert_eq!(pending.len(), 1, "drop must reach stage_or_update");
    assert_eq!(pending[0].card_id, CardId(10));
    assert_eq!(
        pending[0].target,
        PlayTarget::BoardCell {
            lane: DROP_LANE,
            cell: DROP_CELL,
        },
    );
}

#[test]
fn drag_end_lane_wide_drops_on_lane_and_stages_placement() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(20), CardType::Field)]));
    spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(20)]);
    app.update();

    let slot = fan_slot(&mut app, 0);
    start_drag(&mut app, slot, PlayerId(7));
    move_cursor_to_cell(&mut app, DROP_LANE, DROP_CELL);
    end_drag(&mut app);

    assert_eq!(
        drops_for_card(&app, slot),
        vec![HandUiPlacementDropResolved {
            card: slot,
            owner_id: PlayerId(7),
            target: Some(PlayTarget::LaneWide { lane: DROP_LANE }),
        }],
        "LaneWide drag-end inside a lane must emit a LaneWide drop",
    );
    let pending = &app.world().resource::<PendingPlacements>().placements;
    assert_eq!(pending.len(), 1, "drop must reach stage_or_update");
    assert_eq!(pending[0].card_id, CardId(20));
    assert_eq!(pending[0].target, PlayTarget::LaneWide { lane: DROP_LANE });
}

#[test]
fn drag_end_target_obj_drops_on_alive_objective_and_stages_placement() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(30), CardType::Spell)]));
    spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(30)]);
    let slot = fan_slot(&mut app, 0);
    app.world_mut()
        .entity_mut(slot)
        .insert(HandPlacementTargetKind(PlacementTargetKind::TargetObj));
    spawn_objectives(&mut app, PlayerId(2), [true, true, true, true, true]);
    app.update();

    start_drag(&mut app, slot, PlayerId(7));
    move_cursor_to_cell(&mut app, DROP_LANE, DROP_CELL);
    end_drag(&mut app);

    assert_eq!(
        drops_for_card(&app, slot),
        vec![HandUiPlacementDropResolved {
            card: slot,
            owner_id: PlayerId(7),
            target: Some(PlayTarget::TargetObj {
                player_id: PlayerId(2),
                lane: DROP_LANE,
            }),
        }],
        "TargetObj drag-end on an alive objective must emit a TargetObj drop",
    );
    let pending = &app.world().resource::<PendingPlacements>().placements;
    assert_eq!(pending.len(), 1, "drop must reach stage_or_update");
    assert_eq!(pending[0].card_id, CardId(30));
    assert_eq!(
        pending[0].target,
        PlayTarget::TargetObj {
            player_id: PlayerId(2),
            lane: DROP_LANE,
        },
    );
}

#[test]
fn drag_end_target_unit_drops_on_unit_and_stages_placement() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(40), CardType::Spell)]));
    spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(40)]);
    let slot = fan_slot(&mut app, 0);
    app.world_mut()
        .entity_mut(slot)
        .insert(HandPlacementTargetKind(PlacementTargetKind::TargetUnit));
    let unit_id: EntityId = 555;
    spawn_target_unit(&mut app, PlayerId(2), unit_id, DROP_LANE, DROP_CELL);
    app.update();

    start_drag(&mut app, slot, PlayerId(7));
    move_cursor_to_cell(&mut app, DROP_LANE, DROP_CELL);
    end_drag(&mut app);

    assert_eq!(
        drops_for_card(&app, slot),
        vec![HandUiPlacementDropResolved {
            card: slot,
            owner_id: PlayerId(7),
            target: Some(PlayTarget::TargetUnit {
                lane: DROP_LANE,
                unit_id,
            }),
        }],
        "TargetUnit drag-end over a placement target unit must emit a TargetUnit drop",
    );
    let pending = &app.world().resource::<PendingPlacements>().placements;
    assert_eq!(pending.len(), 1, "drop must reach stage_or_update");
    assert_eq!(pending[0].card_id, CardId(40));
    assert_eq!(
        pending[0].target,
        PlayTarget::TargetUnit {
            lane: DROP_LANE,
            unit_id,
        },
    );
}

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

fn start_drag(app: &mut App, slot: Entity, owner_id: PlayerId) {
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
    // PROMPT 1210 — board-cell drop resolution runs on `cursor_world_position`;
    // the screen-space sibling is not consulted on this path.
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

fn drops_for_card(app: &App, card: Entity) -> Vec<HandUiPlacementDropResolved> {
    let messages = app
        .world()
        .resource::<Messages<HandUiPlacementDropResolved>>();
    let mut cursor = messages.get_cursor();
    cursor
        .read(messages)
        .filter(|drop| drop.card == card)
        .cloned()
        .collect()
}
