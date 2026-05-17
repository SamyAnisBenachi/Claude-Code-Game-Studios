use std::time::Duration;

use bevy::camera::NormalizedRenderTarget;
use bevy::color::Alpha;
use bevy::ecs::message::MessageCursor;
use bevy::picking::{
    backend::HitData,
    pointer::{Location, PointerId},
};
use bevy::prelude::*;
use client::{
    asset_wiring::PlaceholderAssets,
    presentation::{
        board_rendering::{
            rendering_constants, BoardCamera, BoardGhostInteraction, BoardRuntimeAssets, GhostUnit,
            LaneGhostWash, ObjectiveTargetGhost, TargetUnitGhost, GHOST_PREVIEW_ALPHA,
        },
        BoardLayout,
    },
    ui::hand::{
        GhostClickedEvent, GhostDragStartEvent, GhostPlacementChanged, HandCardCatalog,
        ObjectiveCell, PlacementTargetUnit,
    },
};
use shared::{card::CardId, protocol::PlayTarget, session::PlayerId};

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[path = "../../helpers/production_app_factory.rs"]
mod production_app_factory;

#[test]
fn br_8_board_cell_ghost_replaces_existing_card_preview() {
    test_helpers::init_test_tracing();
    let mut app = app_with_board_rendering();
    let layout = *app.world().resource::<BoardLayout>();

    stage_ghost(
        &mut app,
        CardId(10),
        PlayTarget::BoardCell { lane: 2, cell: 4 },
    );

    let first = single_ghost_unit(&mut app, CardId(10));
    assert_eq!(
        app.world().get::<Transform>(first).unwrap().translation,
        layout
            .cell_to_world(2, 4)
            .extend(rendering_constants::Z_GHOST_UNIT)
    );
    assert_eq!(app.world().get::<Sprite>(first).unwrap().color.alpha(), 0.5);
    assert!(app.world().get::<Children>(first).is_none());

    stage_ghost(
        &mut app,
        CardId(10),
        PlayTarget::BoardCell { lane: 4, cell: 7 },
    );

    let second = single_ghost_unit(&mut app, CardId(10));
    assert_eq!(
        app.world().get::<Transform>(second).unwrap().translation,
        layout
            .cell_to_world(4, 7)
            .extend(rendering_constants::Z_GHOST_UNIT)
    );
    assert_eq!(ghost_unit_count(&mut app, CardId(10)), 1);
}

// PROMPT 1028 regression: the BoardCell ghost preview previously spawned
// as a flat half-alpha colour rectangle (`Sprite::from_color`), which
// reads as a uniform grey square on a dark board and was the visible
// symptom of "card drop only produces a grey square". Per
// `docs/ux/board-rendering-spec.md` §7 GHOST_PREVIEW_ALPHA, the ghost
// must reuse the real unit's class / placeholder image with a 0.5 alpha
// tint, falling back to the generic unit placeholder when the catalog
// has no class for the card.
#[test]
fn br_8_ghost_preview_uses_class_placeholder_image_for_known_card() {
    test_helpers::init_test_tracing();
    let mut app = app_with_board_rendering();

    // CardId(1) -> Iop Knight in `assets/data/cards.json`, loaded into
    // HandCardCatalog via the HandUiPlugin default-resource init.
    stage_ghost(
        &mut app,
        CardId(1),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );

    let entity = single_ghost_unit(&mut app, CardId(1));
    let sprite = app.world().get::<Sprite>(entity).unwrap().clone();
    assert_eq!(
        sprite.color.alpha(),
        GHOST_PREVIEW_ALPHA,
        "ghost preview must keep the canonical GHOST_PREVIEW_ALPHA tint",
    );

    let placeholders = app.world().resource::<PlaceholderAssets>();
    assert_eq!(
        sprite.image,
        placeholders.board_unit_iop.clone(),
        "ghost preview must reuse the Iop class placeholder image for \
         CardId(1) (Iop Knight) instead of a flat colour rectangle",
    );
    assert_eq!(
        sprite.custom_size,
        Some(rendering_constants::UNIT_SPRITE_SIZE),
        "ghost preview must use UNIT_SPRITE_SIZE, not CELL_NODE_SIZE",
    );

    // The catalog has no atlas frame for CardId(1) at runtime, so the
    // sprite must not carry a texture_atlas slice (atlas-frame path is
    // covered by the perf harness; this branch covers the class
    // placeholder path).
    assert!(
        sprite.texture_atlas.is_none(),
        "no atlas frame is registered for CardId(1) at runtime; ghost \
         must use a direct image handle, not a texture_atlas slice",
    );
    assert!(
        app.world().get::<Children>(entity).is_none(),
        "ghost preview entity must remain childless (BR-8 contract)",
    );
}

#[test]
fn br_8_ghost_preview_falls_back_to_generic_placeholder_for_unknown_card() {
    test_helpers::init_test_tracing();
    let mut app = app_with_board_rendering();

    // CardId(99_999) is intentionally not present in cards.json /
    // HandCardCatalog. With no source_class lookup, the ghost falls
    // through to the generic BoardRuntimeAssets unit placeholder so the
    // player still sees a placement preview rather than nothing.
    stage_ghost(
        &mut app,
        CardId(99_999),
        PlayTarget::BoardCell { lane: 2, cell: 4 },
    );

    let entity = single_ghost_unit(&mut app, CardId(99_999));
    let sprite = app.world().get::<Sprite>(entity).unwrap().clone();
    assert_eq!(sprite.color.alpha(), GHOST_PREVIEW_ALPHA);

    let board_assets = app.world().resource::<BoardRuntimeAssets>();
    assert_eq!(
        sprite.image,
        board_assets.unit_placeholder.clone(),
        "unknown CardId must fall back to BoardRuntimeAssets unit \
         placeholder image (not a flat colour rectangle)",
    );
    assert_eq!(
        sprite.custom_size,
        Some(rendering_constants::UNIT_SPRITE_SIZE),
    );
    assert!(sprite.texture_atlas.is_none());
    assert!(app.world().get::<Children>(entity).is_none());
}

// Stronger sanity check on the HandCardCatalog wiring: the production
// fixture must actually load the cards.json catalogue (this confirms the
// regression test above is exercising the real catalog path, not a
// silent default-empty resource).
#[test]
fn br_8_hand_card_catalog_loads_iop_class_for_card_id_1() {
    test_helpers::init_test_tracing();
    let app = app_with_board_rendering();
    let catalog = app.world().resource::<HandCardCatalog>();
    let card = catalog
        .cards
        .get(&CardId(1))
        .expect("CardId(1) (Iop Knight) must be present in default HandCardCatalog");
    assert_eq!(card.class, shared::card::ClassId::Iop);
}

#[test]
fn br_8_variant_matrix_marks_or_spawns_expected_board_ghosts() {
    test_helpers::init_test_tracing();
    let mut app = app_with_board_rendering();
    let unit = app
        .world_mut()
        .spawn(PlacementTargetUnit {
            owner_id: PlayerId(2),
            unit_id: 77,
        })
        .id();
    let objective = app
        .world_mut()
        .spawn(ObjectiveCell {
            player_id: PlayerId(2),
            lane: 3,
        })
        .id();

    stage_ghost(
        &mut app,
        CardId(20),
        PlayTarget::TargetUnit {
            lane: 1,
            unit_id: 77,
        },
    );
    assert_eq!(
        app.world().get::<TargetUnitGhost>(unit),
        Some(&TargetUnitGhost {
            card_id: CardId(20)
        })
    );
    assert_eq!(ghost_unit_count(&mut app, CardId(20)), 0);

    stage_ghost(
        &mut app,
        CardId(21),
        PlayTarget::TargetObj {
            player_id: PlayerId(2),
            lane: 3,
        },
    );
    assert_eq!(
        app.world().get::<ObjectiveTargetGhost>(objective),
        Some(&ObjectiveTargetGhost {
            card_id: CardId(21)
        })
    );
    assert_eq!(ghost_unit_count(&mut app, CardId(21)), 0);

    stage_ghost(&mut app, CardId(22), PlayTarget::LaneWide { lane: 5 });
    assert_eq!(lane_wash_lanes(&mut app, CardId(22)), vec![5]);
    assert_eq!(ghost_unit_count(&mut app, CardId(22)), 0);

    stage_ghost(&mut app, CardId(23), PlayTarget::Instant);
    assert_eq!(ghost_interaction_count(&mut app, CardId(23)), 0);
}

#[test]
fn br_10_clear_none_removes_matching_card_ghosts_without_spawn_range_edits() {
    test_helpers::init_test_tracing();
    let mut app = app_with_board_rendering();
    stage_ghost(
        &mut app,
        CardId(30),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );
    stage_ghost(&mut app, CardId(31), PlayTarget::LaneWide { lane: 2 });

    clear_ghost(&mut app, CardId(30));
    assert_eq!(ghost_unit_count(&mut app, CardId(30)), 0);
    assert_eq!(lane_wash_lanes(&mut app, CardId(31)), vec![2]);

    clear_ghost(&mut app, CardId(999));
    assert_eq!(lane_wash_lanes(&mut app, CardId(31)), vec![2]);
}

#[test]
fn br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui() {
    // PROMPT 812 (story 015 B1.a): BoardRenderingPlugin registers
    // `on_ghost_drag_start` and `on_ghost_clicked` as observers
    // (`add_observer(...)` at presentation::board_rendering, not as
    // MessageReader systems). Under `MinimalPlugins`, bevy_picking's
    // `DefaultPickingPlugins` is absent, so the observers are not
    // fired by buffered `Pointer<E>` messages. The fixture drives the
    // producer via `trigger_targets(...)`, which is what
    // `DefaultPickingPlugins` does in real gameplay.
    test_helpers::init_test_tracing();
    let mut app = app_with_board_rendering();
    stage_ghost(
        &mut app,
        CardId(40),
        PlayTarget::BoardCell { lane: 2, cell: 2 },
    );
    let ghost = single_ghost_unit(&mut app, CardId(40));
    let camera = board_camera(&mut app);
    let mut click_cursor = drained_cursor::<GhostClickedEvent>(&app);
    let mut drag_cursor = drained_cursor::<GhostDragStartEvent>(&app);

    let press = pointer_press(ghost, camera);
    let click = pointer_click(ghost, camera);
    app.world_mut().trigger(press);
    app.world_mut().trigger(click);
    app.update();

    assert_eq!(
        messages_since(&app, &mut drag_cursor),
        vec![GhostDragStartEvent {
            card_id: CardId(40)
        }]
    );
    assert_eq!(
        messages_since(&app, &mut click_cursor),
        vec![GhostClickedEvent {
            card_id: CardId(40)
        }]
    );
    assert_eq!(ghost_unit_count(&mut app, CardId(40)), 1);
}

// S13-FIXTURE-FACTORY-001: this fixture is now a thin wrapper over the
// canonical production-faithful test app factory. Previously the fixture used
// `MinimalPlugins + StatesPlugin + ClientState + BoardRenderingPlugin` only,
// which omitted `HandUiPlugin` (and the broader `PresentationPlugin` sub-plugin
// set). PROMPT 803 §3 DC-7 / §4 Lane D cluster B1 incident shows the producer-
// system gap that the factory closes. The pointer Message types are still
// registered manually because they are picking-backend types added explicitly
// here for the click/press-driven ghost interaction tests below; they are not
// part of the production plugin set.
fn app_with_board_rendering() -> App {
    let mut app = production_app_factory::production_client_app_in_session();
    app.add_message::<bevy::picking::events::Pointer<bevy::picking::events::Press>>();
    app.add_message::<bevy::picking::events::Pointer<bevy::picking::events::Click>>();
    app
}

fn stage_ghost(app: &mut App, card_id: CardId, target: PlayTarget) {
    app.world_mut().write_message(GhostPlacementChanged {
        target: Some(target),
        card_id: Some(card_id),
    });
    app.update();
}

fn clear_ghost(app: &mut App, card_id: CardId) {
    app.world_mut().write_message(GhostPlacementChanged {
        target: None,
        card_id: Some(card_id),
    });
    app.update();
}

fn single_ghost_unit(app: &mut App, card_id: CardId) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &GhostUnit)>();
    let matches = query
        .iter(app.world())
        .filter_map(|(entity, ghost)| (ghost.card_id == card_id).then_some(entity))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1);
    matches[0]
}

fn ghost_unit_count(app: &mut App, card_id: CardId) -> usize {
    let mut query = app.world_mut().query::<&GhostUnit>();
    query
        .iter(app.world())
        .filter(|ghost| ghost.card_id == card_id)
        .count()
}

fn lane_wash_lanes(app: &mut App, card_id: CardId) -> Vec<u8> {
    let mut query = app.world_mut().query::<&LaneGhostWash>();
    query
        .iter(app.world())
        .filter_map(|wash| (wash.card_id == card_id).then_some(wash.lane))
        .collect()
}

fn ghost_interaction_count(app: &mut App, card_id: CardId) -> usize {
    let mut query = app.world_mut().query::<&BoardGhostInteraction>();
    query
        .iter(app.world())
        .filter(|ghost| ghost.card_id == card_id)
        .count()
}

fn board_camera(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<BoardCamera>>();
    query
        .single(app.world())
        .expect("board camera should exist")
}

fn pointer_location() -> Location {
    Location {
        target: NormalizedRenderTarget::None {
            width: 1,
            height: 1,
        },
        position: Vec2::ZERO,
    }
}

fn hit_data(camera: Entity) -> HitData {
    HitData::new(camera, 0.0, None, None)
}

fn pointer_press(target: Entity, camera: Entity) -> Pointer<Press> {
    Pointer::new(
        PointerId::Mouse,
        pointer_location(),
        Press {
            button: PointerButton::Primary,
            hit: hit_data(camera),
        },
        target,
    )
}

fn pointer_click(target: Entity, camera: Entity) -> Pointer<Click> {
    Pointer::new(
        PointerId::Mouse,
        pointer_location(),
        Click {
            button: PointerButton::Primary,
            hit: hit_data(camera),
            duration: Duration::ZERO,
        },
        target,
    )
}

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
