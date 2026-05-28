//! PROMPT 2046 — draft grid container layout + drag ghost UI-coord tests.
//!
//! Asserts the two PROMPT 2040 architecture audit findings stay fixed:
//!
//!   1. The 9 draft-grid slots are children of a single
//!      `HandDraftGridRoot` container that is centred horizontally in
//!      `HandFanRoot` with stable dimensions (no per-slot hand-coded
//!      `left: 96 + col * 132` / `top: 28 + row * 66` offsets).
//!
//!   2. The drag ghost `HandDragSprite` Node carries the BAKED ghost size
//!      (`HAND_CARD_DISPLAY_WIDTH_PX * 1.10` etc.) directly on `Node.width`
//!      / `Node.height` and DOES NOT rely on a world-space
//!      `Transform::from_scale(splat(1.10))` to render scaled. The
//!      Transform's `scale` field must therefore be the identity vector.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::card_animations::HandDragSprite;
use client::state::ClientState;
use client::ui::hand::{
    GridSlotIndex, HandDraftGridRoot, HandUiEntities, HandUiPlugin, DRAFT_INITIAL_GRID_SLOT_COUNT,
};

const HAND_CARD_DISPLAY_WIDTH_PX: f32 = 108.0;
const HAND_CARD_DISPLAY_HEIGHT_PX: f32 = 150.0;
const HAND_DRAG_SPRITE_SCALE: f32 = 1.10;
const EPSILON: f32 = 0.001;

#[test]
fn test_draft_grid_root_is_unique_pre_pooled_container_parent_of_all_grid_slots() {
    // Arrange — boot the hand UI in InSession so HandUiPlugin pre-spawns
    // the draft-grid container and its 9 child slots.
    let mut app = app_with_hand_ui_in_session();

    // Act — count `HandDraftGridRoot`-tagged entities and verify each grid
    // slot's parent chain reaches the container.
    let mut grid_root_query = app.world_mut().query::<(Entity, &HandDraftGridRoot)>();
    let grid_root_entities: Vec<Entity> = grid_root_query
        .iter(app.world())
        .map(|(entity, _)| entity)
        .collect();

    let mut grid_slot_query = app
        .world_mut()
        .query::<(Entity, &GridSlotIndex, &ChildOf)>();
    let grid_slot_parents: Vec<(u8, Entity)> = grid_slot_query
        .iter(app.world())
        .map(|(_, index, child_of)| (index.0, child_of.parent()))
        .collect();

    // Assert — exactly one container, and all 9 slots parent into it.
    assert_eq!(
        grid_root_entities.len(),
        1,
        "exactly one HandDraftGridRoot must be pre-pooled per session"
    );
    let container = grid_root_entities[0];
    assert_eq!(
        grid_slot_parents.len(),
        DRAFT_INITIAL_GRID_SLOT_COUNT,
        "9 grid slots must be spawned"
    );
    for (slot_index, parent) in &grid_slot_parents {
        assert_eq!(
            *parent, container,
            "grid slot {slot_index} parent must be HandDraftGridRoot — \
             hand-coded per-slot pixel offsets have been replaced by \
             the container's flex layout"
        );
    }

    // Assert — entities resource exposes the container.
    let entities = app.world().resource::<HandUiEntities>();
    assert_eq!(entities.draft_grid_root, container);
}

#[test]
fn test_draft_grid_slot_node_has_no_hand_coded_absolute_pixel_offsets() {
    // Arrange — boot hand UI; capture each grid slot's spawn-time Node.
    let mut app = app_with_hand_ui_in_session();

    let mut slot_query = app.world_mut().query::<(&GridSlotIndex, &Node)>();
    let slot_nodes: Vec<(u8, Node)> = slot_query
        .iter(app.world())
        .map(|(index, node)| (index.0, node.clone()))
        .collect();

    // Assert — every slot is non-absolute and has Auto left/top. This is
    // the architecture guard: a regression that re-introduces
    // `position_type: Absolute` + `left: Val::Px(...)` per slot will fail
    // this test loudly.
    assert_eq!(slot_nodes.len(), DRAFT_INITIAL_GRID_SLOT_COUNT);
    for (index, node) in &slot_nodes {
        assert_eq!(
            node.position_type,
            PositionType::Relative,
            "grid slot {index} must be Relative inside the flex container; \
             Absolute positioning brings back the per-slot pixel-offset bug"
        );
        assert!(
            matches!(node.left, Val::Auto),
            "grid slot {index} left must stay Auto; flex layout owns the X offset (was Val::Px(...))"
        );
        assert!(
            matches!(node.top, Val::Auto),
            "grid slot {index} top must stay Auto; flex layout owns the Y offset (was Val::Px(...))"
        );
        assert!(
            matches!(node.width, Val::Px(w) if (w - 120.0).abs() < EPSILON),
            "grid slot {index} width must stay 120 px (HAND_DRAFT_GRID_CARD_WIDTH_PX)"
        );
        assert!(
            matches!(node.height, Val::Px(h) if (h - 56.0).abs() < EPSILON),
            "grid slot {index} height must stay 56 px (HAND_DRAFT_GRID_CARD_HEIGHT_PX)"
        );
    }
}

#[test]
fn test_draft_grid_root_is_centered_in_fan_root_with_stable_dimensions() {
    // Arrange.
    let app = app_with_hand_ui_in_session();
    let entities = *app.world().resource::<HandUiEntities>();

    // Act — read the container's Node.
    let node = app
        .world()
        .get::<Node>(entities.draft_grid_root)
        .expect("HandDraftGridRoot must have a Node");

    // Width: 3 cards x 120 + 2 gaps x 12 + 2 paddings x 16 = 360 + 24 + 32 = 416.
    let expected_width = 3.0 * 120.0 + 2.0 * 12.0 + 2.0 * 16.0;
    let expected_height = 3.0 * 56.0 + 2.0 * 10.0 + 2.0 * 16.0;

    // Assert — Absolute placement anchored at left:50% with margin_left =
    // -width/2, so the container is centred regardless of viewport width.
    assert_eq!(node.position_type, PositionType::Absolute);
    assert!(
        matches!(node.left, Val::Percent(p) if (p - 50.0).abs() < EPSILON),
        "container must anchor at left:50% (got {:?})",
        node.left
    );
    assert!(
        matches!(node.margin.left, Val::Px(m) if (m + expected_width / 2.0).abs() < EPSILON),
        "container must apply negative margin_left = -width/2 to centre on the 50% anchor (got {:?})",
        node.margin.left
    );

    // Assert — stable px dimensions (1280x720 or any viewport width
    // resolves the same 416x208 container).
    assert!(
        matches!(node.width, Val::Px(w) if (w - expected_width).abs() < EPSILON),
        "container width must be {expected_width} px (got {:?})",
        node.width
    );
    assert!(
        matches!(node.height, Val::Px(h) if (h - expected_height).abs() < EPSILON),
        "container height must be {expected_height} px (got {:?})",
        node.height
    );

    // Assert — flex layout configuration: row-wrap so 3 slots/row.
    assert_eq!(node.display, Display::Flex);
    assert_eq!(node.flex_direction, FlexDirection::Row);
    assert_eq!(node.flex_wrap, FlexWrap::Wrap);
}

#[test]
fn test_drag_ghost_node_uses_baked_scaled_dimensions_and_identity_transform() {
    // Arrange — boot hand UI.
    let mut app = app_with_hand_ui_in_session();

    // Act — read the drag sprite's Node + Transform.
    let mut query = app
        .world_mut()
        .query_filtered::<(&Node, &Transform), With<HandDragSprite>>();
    let (node, transform) = query
        .iter(app.world())
        .next()
        .expect("HandDragSprite entity must exist");

    let expected_width = HAND_CARD_DISPLAY_WIDTH_PX * HAND_DRAG_SPRITE_SCALE;
    let expected_height = HAND_CARD_DISPLAY_HEIGHT_PX * HAND_DRAG_SPRITE_SCALE;

    // Assert — Node carries the baked scaled dimensions (UI coord space).
    assert!(
        matches!(node.width, Val::Px(w) if (w - expected_width).abs() < EPSILON),
        "drag ghost Node.width must be {expected_width} px (baked scale) — \
         got {:?}. The 1.10x scale must not return to Transform::from_scale.",
        node.width
    );
    assert!(
        matches!(node.height, Val::Px(h) if (h - expected_height).abs() < EPSILON),
        "drag ghost Node.height must be {expected_height} px (baked scale) — got {:?}",
        node.height
    );

    // Assert — Transform.scale is identity (the world-space scale path is
    // gone). Any `Vec3::splat(1.10)` here brings back ADR-021 R2 drift.
    assert!(
        (transform.scale.x - 1.0).abs() < EPSILON
            && (transform.scale.y - 1.0).abs() < EPSILON
            && (transform.scale.z - 1.0).abs() < EPSILON,
        "drag ghost Transform.scale must be identity — got {:?}. \
         Scale belongs on Node.width/Node.height (UI coords), not on \
         Transform (world coords).",
        transform.scale
    );
}

fn app_with_hand_ui_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HandUiPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}
