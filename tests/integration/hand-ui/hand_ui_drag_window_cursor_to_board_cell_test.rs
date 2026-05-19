//! PROMPT 1410 / S18-BOARD-PICKING-BACKEND-DRAG-TO-CELL-001 — explicit
//! window→world cursor producer regression coverage.
//!
//! Drives the production input/picking/resolution path as far as a
//! `MinimalPlugins` harness permits, with `PrimaryWindow.cursor_position`
//! as the *only* source of cursor updates during the drag (mirroring real
//! gameplay over the board, which has no bevy_picking backend). Before the
//! PROMPT 1410 producer landed, the `Pointer<Move>` stream silently dried
//! up as soon as the cursor left the UI, leaving
//! `ActivePlacementDrag.cursor_world_position` at its drag-start `None`.
//! `handle_placement_drag_ended_system` then fell through with
//! `target=None`, the card flipped back to `FanSlotState::Active`, and the
//! next click routed through the `fan_active_default_drop` fallback that
//! AUDIT-1392-P02 surfaced — never picking the cell under the cursor.
//!
//! Test scenario: viewport 1280×720, `Camera2d` at world origin, default
//! orthographic projection. Drag is started with a `Pointer<Press>`, then
//! the cursor is moved by writing into `Window.set_cursor_position` only
//! (no `Pointer<Move>` messages). On `Pointer<Release>` the drag-end must
//! resolve to the `BoardCell { lane: 2, cell: 5 }` cell the cursor lands
//! on, and the staged placement target must match.
//!
//! Includes a regression assertion for the no-cursor fallback: when no
//! cursor position is ever available (e.g. cursor-left-window scenario),
//! the drag-end must still close with `target=None` so the click-to-stage
//! default at `default_click_stage_target` remains reachable on the next
//! click — the original fallback contract for unresolved drops is
//! preserved.

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
use bevy::window::{PrimaryWindow, WindowResolution};
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
const TARGET_CELL: u8 = 5;
const FIRST_LANE: u8 = 1;
const FIRST_CELL: u8 = 2;
const SECOND_LANE: u8 = 3;
const SECOND_CELL: u8 = 6;
const LOCAL_PLAYER_ID: PlayerId = PlayerId(7);

#[test]
fn window_cursor_during_drag_resolves_drop_to_hovered_board_cell() {
    test_helpers::init_test_tracing();

    let (mut app, camera) = app_with_board_camera_and_window();
    set_hand(&mut app, [CardId(100)]);
    spawn_board_cells(&mut app);
    app.update();

    let slot = fan_slot(&mut app, 0);

    let target_world = app
        .world()
        .resource::<BoardLayout>()
        .cell_to_world(TARGET_LANE, TARGET_CELL);
    let target_viewport = camera_world_to_viewport(&app, camera, target_world);

    let mut drop_cursor = drained_cursor::<HandUiPlacementDropResolved>(&app);
    let mut cursor_moved_cursor = drained_cursor::<HandUiPlacementCursorMoved>(&app);

    // Press on the fan slot — the producer reads only the entity, not the
    // position, so the press cursor coordinate is irrelevant for this case.
    app.world_mut()
        .write_message(pointer_press(slot, camera, Vec2::new(640.0, 600.0)));
    app.update();
    assert!(
        active_drag_active(&app),
        "ActivePlacementDrag must be live after Press is consumed",
    );

    // Move the cursor by *only* updating Window.cursor_position — no
    // Pointer<Move> messages. The PROMPT 1410 window producer must drive
    // the cursor_world_position update from here.
    set_window_cursor(&mut app, target_viewport);
    app.update();

    let cursor_moves = messages_since(&app, &mut cursor_moved_cursor);
    assert!(
        !cursor_moves.is_empty(),
        "PROMPT 1410 producer must emit at least one HandUiPlacementCursorMoved while the drag \
         is live and Window.cursor_position is populated (no Pointer<Move> backend covers the \
         board area in production)",
    );

    let drag = *app.world().resource::<ActivePlacementDrag>();
    let world_position = drag.cursor_world_position.expect(
        "PROMPT 1410: Window.cursor_position must feed ActivePlacementDrag.cursor_world_position \
         while the drag is live, even with no Pointer<Move> stream over the board",
    );
    assert!(
        (world_position - target_world).length() < 0.5,
        "window→world cursor must project onto cell_to_world {:?} (got {:?})",
        target_world,
        world_position,
    );

    // Release closes the drag. The producer pipeline runs in the same
    // tick so `handle_placement_drag_ended_system` reads the freshly
    // populated `cursor_world_position` and resolves a BoardCell.
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
        "Window cursor over cell ({}, {}) must resolve the drop to that BoardCell, \
         not the default-spawn fallback (AUDIT-1392-P02 fan_active_default_drop)",
        TARGET_LANE,
        TARGET_CELL,
    );

    let pending = &app.world().resource::<PendingPlacements>().placements;
    assert_eq!(
        pending.len(),
        1,
        "the resolved drop must stage exactly one placement",
    );
    assert_eq!(
        pending[0].target,
        PlayTarget::BoardCell {
            lane: TARGET_LANE,
            cell: TARGET_CELL,
        },
        "staged placement target must match the cell under the window cursor",
    );
}

#[test]
fn no_cursor_during_drag_leaves_drop_target_none_so_click_fallback_can_take_over() {
    test_helpers::init_test_tracing();

    // Same app shape as the happy-path test, but the window's cursor
    // position is never populated. Regression guard: the existing
    // click-to-stage fallback (`default_click_stage_target`) must remain
    // reachable when the drag never produced a board-cell hover — the
    // PROMPT 1410 producer is additive, not a replacement.
    let (mut app, camera) = app_with_board_camera_and_window();
    set_hand(&mut app, [CardId(100)]);
    spawn_board_cells(&mut app);
    app.update();

    let slot = fan_slot(&mut app, 0);
    let mut drop_cursor = drained_cursor::<HandUiPlacementDropResolved>(&app);

    app.world_mut()
        .write_message(pointer_press(slot, camera, Vec2::new(640.0, 600.0)));
    app.update();

    // No cursor — neither Pointer<Move> nor Window.cursor_position.
    app.world_mut()
        .write_message(pointer_release(slot, camera, Vec2::new(640.0, 600.0)));
    app.update();

    let drops: Vec<_> = messages_since(&app, &mut drop_cursor);
    assert_eq!(
        drops,
        vec![HandUiPlacementDropResolved {
            card: slot,
            owner_id: LOCAL_PLAYER_ID,
            target: None,
        }],
        "with no cursor available, drag-end must close with target=None so the \
         next click can route through default_click_stage_target",
    );

    let pending = &app.world().resource::<PendingPlacements>().placements;
    assert!(
        pending.is_empty(),
        "target=None must NOT stage a placement — that path is reserved for click-to-stage",
    );
}

#[test]
fn interaction_press_drag_tracks_window_cursor_cell_changes_and_drops_last_cell() {
    test_helpers::init_test_tracing();

    let (mut app, camera) = app_with_board_camera_and_window();
    set_hand(&mut app, [CardId(100)]);
    spawn_board_cells(&mut app);
    app.update();

    let slot = fan_slot(&mut app, 0);
    let first_world = app
        .world()
        .resource::<BoardLayout>()
        .cell_to_world(FIRST_LANE, FIRST_CELL);
    let second_world = app
        .world()
        .resource::<BoardLayout>()
        .cell_to_world(SECOND_LANE, SECOND_CELL);
    let first_viewport = camera_world_to_viewport(&app, camera, first_world);
    let second_viewport = camera_world_to_viewport(&app, camera, second_world);

    let mut drop_cursor = drained_cursor::<HandUiPlacementDropResolved>(&app);
    let mut cursor_moved_cursor = drained_cursor::<HandUiPlacementCursorMoved>(&app);

    // Real UI clicks first surface as `Interaction::Pressed`, not always as a
    // bevy_picking Pointer<Press>. In staging, that must begin a drag instead
    // of immediately taking the default-click staging path.
    press_fan_slot_interaction(&mut app, slot);
    app.update();
    assert!(
        active_drag_active(&app),
        "Interaction::Pressed on an active staging fan card must start ActivePlacementDrag",
    );

    set_window_cursor(&mut app, first_viewport);
    app.update();
    let first_drag = *app.world().resource::<ActivePlacementDrag>();
    assert!(
        (first_drag
            .cursor_world_position
            .expect("first cursor world")
            - first_world)
            .length()
            < 0.5,
        "first window cursor update must resolve to the first board cell",
    );

    set_window_cursor(&mut app, second_viewport);
    app.update();
    let second_drag = *app.world().resource::<ActivePlacementDrag>();
    assert!(
        (second_drag
            .cursor_world_position
            .expect("second cursor world")
            - second_world)
            .length()
            < 0.5,
        "second window cursor update must replace the first board-cell target",
    );

    let cursor_moves = messages_since(&app, &mut cursor_moved_cursor);
    assert!(
        cursor_moves.len() >= 2,
        "window cursor movement during an Interaction-started drag must emit cursor-moved messages",
    );
    assert_eq!(
        cursor_moves.last().and_then(|m| m.screen_position),
        Some(second_viewport),
        "the last cursor-moved message must carry the final screen position",
    );

    release_primary_mouse(&mut app);
    app.update();

    let drops: Vec<_> = messages_since(&app, &mut drop_cursor);
    assert_eq!(
        drops,
        vec![HandUiPlacementDropResolved {
            card: slot,
            owner_id: LOCAL_PLAYER_ID,
            target: Some(PlayTarget::BoardCell {
                lane: SECOND_LANE,
                cell: SECOND_CELL,
            }),
        }],
        "mouse release must drop on the latest cursor-derived cell, not the default fallback",
    );

    let pending = &app.world().resource::<PendingPlacements>().placements;
    assert_eq!(pending.len(), 1);
    assert_eq!(
        pending[0].target,
        PlayTarget::BoardCell {
            lane: SECOND_LANE,
            cell: SECOND_CELL,
        },
        "staged placement must consume the latest cursor-derived cell",
    );
}

// ── App / camera / window setup ──────────────────────────────────────────────

fn app_with_board_camera_and_window() -> (App, Entity) {
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

    spawn_primary_window(&mut app, VIEWPORT_PHYSICAL_SIZE);
    let camera = spawn_world_space_camera(&mut app, VIEWPORT_PHYSICAL_SIZE);
    app.update();

    (app, camera)
}

fn spawn_primary_window(app: &mut App, viewport_size: UVec2) {
    app.world_mut().spawn((
        Window {
            resolution: WindowResolution::new(viewport_size.x, viewport_size.y),
            ..default()
        },
        PrimaryWindow,
    ));
}

fn set_window_cursor(app: &mut App, logical_position: Vec2) {
    let mut query = app
        .world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>();
    let mut window = query
        .single_mut(app.world_mut())
        .expect("PrimaryWindow must exist before set_window_cursor is called");
    window.set_cursor_position(Some(logical_position));
}

fn press_fan_slot_interaction(app: &mut App, slot: Entity) {
    *app.world_mut()
        .entity_mut(slot)
        .get_mut::<Interaction>()
        .expect("fan slot must carry Interaction") = Interaction::Pressed;
}

fn release_primary_mouse(app: &mut App) {
    let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    buttons.press(MouseButton::Left);
    buttons.release(MouseButton::Left);
}

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

fn active_drag_active(app: &App) -> bool {
    let drag = app.world().resource::<ActivePlacementDrag>();
    drag.card.is_some() && drag.target_kind.is_some()
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

// ── Message-cursor helpers ───────────────────────────────────────────────────

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
