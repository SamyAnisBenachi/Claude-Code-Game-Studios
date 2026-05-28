//! PROMPT 2036 — Placement drag now emits `GhostPlacementChanged` LIVE as
//! the cursor moves between board cells, so the targeting overlay, ghost
//! unit preview, and `BoardCellHighlighted` paint legality affordance under
//! the cursor BEFORE the drop resolves. Without this producer the overlay
//! only fired post-drop; in-flight invalid cells were visually accepted and
//! valid cells were not highlighted (UX-001..UX-005, P0-014 from
//! production/qa/bugs/current-unplayable-bug-register-2026-05-28.md).

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::{
    hand::{
        BoardSpawnEdge, FanSlotIndex, GhostPlacementChanged, HandCardCatalog, HandContents,
        HandPlacementTargetKind, HandUiPlacementCursorMoved, HandUiPlacementDragStarted,
        HandUiPlugin, PendingPlacements, PlacementBoardView, PlacementTargetKind,
    },
    shared::BoardLayout,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[test]
fn test_live_ghost_emits_board_cell_when_minion_drag_cursor_enters_cell() {
    // Arrange — a Minion drag whose cursor begins off-board.
    let mut app = app_with_hand_ui_in_placement([(CardId(10), CardType::Minion)]);
    set_hand(&mut app, [CardId(10)]);
    app.update();
    start_drag(&mut app, 0, PlayerId(1));
    drain_ghost_messages(&mut app);

    // Act — cursor moves to a valid board cell.
    move_cursor_to_cell(&mut app, 2, 1);

    // Assert — a GhostPlacementChanged with Some(BoardCell) is emitted live,
    // BEFORE any drop resolves.
    let messages = drain_ghost_messages(&mut app);
    assert!(
        messages.iter().any(|m| matches!(
            m.target,
            Some(PlayTarget::BoardCell { lane: 2, cell: 1 })
        )),
        "expected live GhostPlacementChanged for cell (2,1); got {messages:?}",
    );
}

#[test]
fn test_live_ghost_emits_none_when_minion_drag_cursor_leaves_board() {
    // Arrange — drag with cursor already over a valid cell so a Some target
    // is in the producer's last-emitted slot.
    let mut app = app_with_hand_ui_in_placement([(CardId(11), CardType::Minion)]);
    set_hand(&mut app, [CardId(11)]);
    app.update();
    start_drag(&mut app, 0, PlayerId(1));
    move_cursor_to_cell(&mut app, 3, 1);
    drain_ghost_messages(&mut app);

    // Act — cursor moves far off-board (well outside the envelope).
    move_cursor_world(&mut app, Vec2::new(99_999.0, 99_999.0));

    // Assert — the producer flushes a target=None so the overlay despawns.
    let messages = drain_ghost_messages(&mut app);
    assert!(
        messages.iter().any(|m| m.target.is_none() && m.card_id == Some(CardId(11))),
        "expected live GhostPlacementChanged with target=None after cursor left the board; got {messages:?}",
    );
}

#[test]
fn test_live_ghost_does_not_emit_for_target_unit_drag_kind() {
    // Arrange — TargetUnit drag kind; the live producer must stay silent
    // and let the existing target-unit hover pathway own the affordance so
    // we do not pollute consumers with stray None writes.
    let mut app = app_with_hand_ui_in_placement([(CardId(12), CardType::Spell)]);
    set_hand(&mut app, [CardId(12)]);
    set_slot_target_kind(&mut app, 0, PlacementTargetKind::TargetUnit);
    app.update();
    start_drag(&mut app, 0, PlayerId(1));
    drain_ghost_messages(&mut app);

    // Act — cursor moves over a board cell.
    move_cursor_to_cell(&mut app, 2, 1);

    // Assert — no live GhostPlacementChanged is produced by the new system.
    let messages = drain_ghost_messages(&mut app);
    assert!(
        messages.is_empty(),
        "live producer must stay silent for TargetUnit kind; got {messages:?}",
    );
}

#[test]
fn test_live_ghost_dedupes_repeated_moves_to_same_cell() {
    // Arrange — drag with cursor settled on a valid board cell.
    let mut app = app_with_hand_ui_in_placement([(CardId(13), CardType::Minion)]);
    set_hand(&mut app, [CardId(13)]);
    app.update();
    start_drag(&mut app, 0, PlayerId(1));
    move_cursor_to_cell(&mut app, 4, 1);
    drain_ghost_messages(&mut app);

    // Act — same cell again across multiple ticks.
    move_cursor_to_cell(&mut app, 4, 1);
    move_cursor_to_cell(&mut app, 4, 1);

    // Assert — producer suppresses duplicates so downstream ghost-unit
    // rebuild churn stays cheap.
    let messages = drain_ghost_messages(&mut app);
    assert!(
        messages.is_empty(),
        "expected dedupe when cursor stays on the same cell; got {messages:?}",
    );
}

// ─── helpers ────────────────────────────────────────────────────────────────

fn app_with_hand_ui_in_placement<const N: usize>(
    catalog_entries: [(CardId, CardType); N],
) -> App {
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
    let catalog: HashMap<CardId, CardData> = catalog_entries
        .into_iter()
        .map(|(card_id, card_type)| (card_id, test_card(card_id, card_type)))
        .collect();
    app.insert_resource(HandCardCatalog { cards: catalog });
    app.insert_resource(PlacementBoardView {
        local_player_id: PlayerId(1),
        opponent_player_id: PlayerId(2),
        spawn_edge: BoardSpawnEdge::LowCells,
        spawn_range_cells: 4,
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app.world_mut()
        .resource_mut::<CurrentClientPhase>()
        .phase = RoundPhase::Placement;
    app.update();
    app
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

fn set_hand<const N: usize>(app: &mut App, cards: [CardId; N]) {
    app.world_mut().resource_mut::<HandContents>().cards = cards.to_vec();
    let _ = app.world_mut().resource::<PendingPlacements>();
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}

fn set_slot_target_kind(app: &mut App, slot_index: u8, target_kind: PlacementTargetKind) {
    let slot = fan_slot(app, slot_index);
    app.world_mut()
        .entity_mut(slot)
        .insert(HandPlacementTargetKind(target_kind));
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
    move_cursor_world(app, position);
}

fn move_cursor_world(app: &mut App, world_position: Vec2) {
    app.world_mut().write_message(HandUiPlacementCursorMoved {
        world_position: Some(world_position),
        screen_position: None,
    });
    app.update();
}

fn drain_ghost_messages(app: &mut App) -> Vec<GhostPlacementChanged> {
    let mut messages = app
        .world_mut()
        .resource_mut::<Messages<GhostPlacementChanged>>();
    let drained: Vec<GhostPlacementChanged> = messages.drain().collect();
    drained
}
