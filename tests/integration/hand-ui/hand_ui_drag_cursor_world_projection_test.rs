//! PROMPT 1210 — B-1203-PLA-01 drag-cursor coord-space repair.
//!
//! Drives the full producer / consumer / drop path the UI runs in production:
//!
//!   `Pointer<Move>` → `produce_drag_cursor_moved_from_pointer_move_system`
//!     → `HandUiPlacementCursorMoved`
//!     → `handle_placement_cursor_moved_system`
//!     → `ActivePlacementDrag.cursor_world_position`
//!     → `handle_placement_drag_ended_system` → `cursor_to_lane_cell`
//!     → `HandUiPlacementDropResolved { target: Some(BoardCell { ... }) }`.
//!
//! Pins the fix that the producer must convert
//! `Pointer<Move>.pointer_location.position` (viewport pixels, Y-down) into
//! world-space (Y-up, origin aligned to `BoardLayout::board_origin`) via
//! `Camera::viewport_to_world_2d` before `cursor_to_lane_cell` consumes it.
//! Before the fix this path silently misrouted every drop: the viewport
//! pixel `(viewport.x, viewport.y)` was treated as if it were
//! `(world.x, world.y)`, so a release deep on the board produced
//! `None` or the wrong cell.
//!
//! Scenario: viewport `1280 × 720`, camera at world `(0, 0, 0)`, default 2D
//! orthographic projection. Cursor is placed at the viewport pixel that the
//! camera projects to `BoardLayout::cell_to_world(lane=2, cell=3)`. The drag
//! must resolve to `BoardCell { lane: 2, cell: 3 }`.

use std::collections::HashMap;

use bevy::camera::{
    Camera, Camera2d, CameraProjection, OrthographicProjection, Projection, RenderTargetInfo,
};
use bevy::ecs::message::MessageCursor;
use bevy::math::UVec2;
use bevy::picking::{
    backend::HitData,
    pointer::{Location, PointerId},
};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    ActivePlacementDrag, BoardSpawnEdge, FanSlotIndex, HandCardCatalog, HandContents,
    HandUiPlacementCursorMoved, HandUiPlacementDropResolved, HandUiPlugin, PendingPlacements,
    PlacementBoardView,
};
use client::ui::shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const VIEWPORT_PHYSICAL_SIZE: UVec2 = UVec2::new(1280, 720);
const TARGET_LANE: u8 = 2;
const TARGET_CELL: u8 = 3;
const LOCAL_PLAYER_ID: PlayerId = PlayerId(7);

#[test]
fn drag_to_board_cell_projects_viewport_cursor_into_world_space() {
    test_helpers::init_test_tracing();

    let (mut app, camera) = app_with_board_and_camera();
    set_hand(&mut app, [CardId(100)]);
    spawn_board_cells(&mut app);
    app.update();

    let slot = fan_slot(&mut app, 0);

    // The world-space target the cursor must land on.
    let target_world = app
        .world()
        .resource::<BoardLayout>()
        .cell_to_world(TARGET_LANE, TARGET_CELL);

    // Project that back through the very camera + transform the producer sees,
    // so the test exercises the exact `viewport_to_world_2d` pair (not a
    // hardcoded inverse). If the producer skips the conversion the drop falls
    // on the wrong cell or none at all.
    let target_viewport = camera_world_to_viewport(&app, camera, target_world);

    let mut drop_cursor = drained_cursor::<HandUiPlacementDropResolved>(&app);

    // Press on the fan slot at any viewport coordinate (start_drag uses the
    // press target entity, not its position).
    app.world_mut()
        .write_message(pointer_press(slot, camera, Vec2::new(640.0, 600.0)));
    app.update();

    // Move the cursor to the viewport pixel that projects to the target cell.
    app.world_mut()
        .write_message(pointer_move(slot, camera, target_viewport));
    app.update();

    // World-space conversion was applied: the drag state must now hold the
    // world coordinate of the cell, not the raw viewport position.
    let drag = *app.world().resource::<ActivePlacementDrag>();
    let world_position = drag
        .cursor_world_position
        .expect("producer must populate world-space cursor with an active 2D camera");
    assert!(
        (world_position - target_world).length() < 0.5,
        "cursor_world_position {:?} should match cell_to_world {:?} within sub-pixel tolerance",
        world_position,
        target_world,
    );
    assert_ne!(
        drag.cursor_world_position, drag.cursor_screen_position,
        "world-space and screen-space cursors must not collapse onto the same value — that is exactly the B-1203-PLA-01 bug",
    );

    // Release closes the drag and runs the BoardCell branch of
    // `handle_placement_drag_ended_system` (Minion target_kind).
    app.world_mut()
        .write_message(pointer_release(slot, camera, target_viewport));
    app.update();

    let drops: Vec<_> = messages_since(&app, &mut drop_cursor);
    assert_eq!(
        drops,
        vec![HandUiPlacementDropResolved {
            card: slot,
            owner_id: LOCAL_PLAYER_ID,
            target: Some(PlayTarget::BoardCell {
                lane: TARGET_LANE,
                cell: TARGET_CELL,
            }),
        }],
        "release over viewport pixel for lane 2 / cell 3 must resolve to that BoardCell after viewport → world conversion",
    );

    let pending = &app.world().resource::<PendingPlacements>().placements;
    assert_eq!(pending.len(), 1, "drop must reach stage_or_update");
    assert_eq!(
        pending[0].target,
        PlayTarget::BoardCell {
            lane: TARGET_LANE,
            cell: TARGET_CELL,
        },
        "staged placement target must match the projected cell",
    );
}

// ── App / camera setup ───────────────────────────────────────────────────────

fn app_with_board_and_camera() -> (App, Entity) {
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
        cards: test_catalog([(CardId(100), CardType::Minion)]),
    });
    app.insert_resource(PlacementBoardView {
        local_player_id: LOCAL_PLAYER_ID,
        opponent_player_id: PlayerId(8),
        spawn_edge: BoardSpawnEdge::LowCells,
        spawn_range_cells: BOARD_CELL_COUNT,
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_phase(&mut app, RoundPhase::Placement);
    app.update();
    let camera = spawn_world_space_camera(&mut app, VIEWPORT_PHYSICAL_SIZE);
    app.update();
    (app, camera)
}

/// Spawns a `Camera2d` with `Camera.computed` populated so the producer's
/// `viewport_to_world_2d` resolves a usable conversion in this MinimalPlugins
/// test app. The render pipeline never runs — we only need the projection
/// matrix and target info that the conversion math reads.
fn spawn_world_space_camera(app: &mut App, viewport_size: UVec2) -> Entity {
    let mut projection = OrthographicProjection::default_2d();
    let logical_size = viewport_size.as_vec2();
    projection.update(logical_size.x, logical_size.y);
    let clip_from_view = projection.get_clip_from_view();

    let mut camera = Camera::default();
    camera.computed.target_info = Some(RenderTargetInfo {
        physical_size: viewport_size,
        scale_factor: 1.0,
    });
    camera.computed.clip_from_view = clip_from_view;

    let transform = Transform::from_xyz(0.0, 0.0, 0.0);
    app.world_mut()
        .spawn((
            camera,
            Camera2d,
            Projection::Orthographic(projection),
            transform,
            GlobalTransform::from(transform),
        ))
        .id()
}

fn camera_world_to_viewport(app: &App, camera: Entity, world_position: Vec2) -> Vec2 {
    let camera_component = app
        .world()
        .get::<Camera>(camera)
        .expect("camera entity must carry Camera");
    let transform = app
        .world()
        .get::<GlobalTransform>(camera)
        .expect("camera entity must carry GlobalTransform");
    camera_component
        .world_to_viewport(transform, world_position.extend(0.0))
        .expect("world target should project into the viewport for this fixture")
}

// ── Resource + hand wiring ───────────────────────────────────────────────────

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

fn spawn_board_cells(app: &mut App) {
    for lane in 1..=BOARD_LANE_COUNT {
        for cell in 1..=BOARD_CELL_COUNT {
            app.world_mut().spawn(LaneCell { lane, cell });
        }
    }
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}

// ── bevy_picking event factories ─────────────────────────────────────────────

fn pointer_location(position: Vec2) -> Location {
    Location {
        target: bevy::camera::NormalizedRenderTarget::None {
            width: VIEWPORT_PHYSICAL_SIZE.x,
            height: VIEWPORT_PHYSICAL_SIZE.y,
        },
        position,
    }
}

fn hit_data(camera: Entity) -> HitData {
    HitData::new(camera, 0.0, None, None)
}

fn pointer_press(target: Entity, camera: Entity, position: Vec2) -> Pointer<Press> {
    Pointer::new(
        PointerId::Mouse,
        pointer_location(position),
        Press {
            button: PointerButton::Primary,
            hit: hit_data(camera),
        },
        target,
    )
}

fn pointer_move(target: Entity, camera: Entity, position: Vec2) -> Pointer<Move> {
    Pointer::new(
        PointerId::Mouse,
        pointer_location(position),
        Move {
            hit: hit_data(camera),
            delta: Vec2::ZERO,
        },
        target,
    )
}

fn pointer_release(target: Entity, camera: Entity, position: Vec2) -> Pointer<Release> {
    Pointer::new(
        PointerId::Mouse,
        pointer_location(position),
        Release {
            button: PointerButton::Primary,
            hit: hit_data(camera),
        },
        target,
    )
}

// ── Message-cursor helpers ────────────────────────────────────────────────────

fn drained_cursor<M: Message + Clone>(app: &App) -> MessageCursor<M> {
    let messages = app.world().resource::<Messages<M>>();
    let mut cursor = messages.get_cursor();
    let _ = cursor.read(messages).count();
    cursor
}

fn messages_since<M: Message + Clone>(app: &App, cursor: &mut MessageCursor<M>) -> Vec<M> {
    let messages = app.world().resource::<Messages<M>>();
    cursor.read(messages).cloned().collect()
}

// Confirm at compile time that HandUiPlacementCursorMoved is in scope for use
// sites — kept as a compile-time check after the PROMPT 1210 field split.
#[allow(dead_code)]
fn assert_cursor_moved_imported() -> std::marker::PhantomData<HandUiPlacementCursorMoved> {
    std::marker::PhantomData
}
