//! Story 025 -- Lobby class-picker layout and hierarchy.
//!
//! Guards the Sprint 14 class-picker migration:
//!
//! - one marked class-picker block owns the heading and class grid;
//! - every selectable [`ClassId`] pairs its portrait and button in the same
//!   stable cell;
//! - the grid resolves to seven fixed columns at the canonical 1366x768 and
//!   1920x1080 viewports;
//! - selected-class affordance is present on first spawn and refreshes from
//!   [`LobbyInputState`] changes without respawning the lobby.
//!
//! Friend-game scope only. This test does not claim Standard-tier
//! accessibility (`QA-COND-0005`), playtest validation (`QA-COND-0006`),
//! placeholder-art completion (`PAW-TD-*-a`), or `S8-QA-001-W1` closure.

use bevy::asset::AssetPlugin;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::ClientState;
use client::ui::design_tokens::spacing::{SPACING_LG, SPACING_SM};
use client::ui::design_tokens::typography;
use client::ui::lobby::{
    lobby_all_class_ids, lobby_class_options, LobbyClassButton, LobbyClassPickerBlock,
    LobbyClassPickerCell, LobbyClassPickerGrid, LobbyClassPickerHeading, LobbyClassPortrait,
    LobbyInputState, LobbyPanel, LobbyUiPlugin, LOBBY_CLASS_PICKER_BUTTON_WIDTH_PX,
    LOBBY_CLASS_PICKER_CELL_HEIGHT_PX, LOBBY_CLASS_PICKER_CELL_WIDTH_PX,
    LOBBY_CLASS_PICKER_GRID_COLUMNS, LOBBY_CLASS_PICKER_SELECTABLE_COUNT, LOBBY_PANEL_MAX_WIDTH_PX,
    LOBBY_PANEL_WIDTH_PERCENT,
};
use shared::card::ClassId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const VIEWPORT_MIN: (f32, f32) = (1366.0, 768.0);
const VIEWPORT_HD: (f32, f32) = (1920.0, 1080.0);

fn spawn_lobby_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(LobbyUiPlugin);

    app.update();
    app.update();

    app
}

fn child_entities(world: &World, entity: Entity) -> Vec<Entity> {
    world
        .entity(entity)
        .get::<Children>()
        .map(|children| children.iter().collect::<Vec<_>>())
        .unwrap_or_default()
}

fn sorted_class_cell_entities(world: &mut World) -> Vec<Entity> {
    let mut query = world.query_filtered::<Entity, With<LobbyClassPickerCell>>();
    let mut entities = query.iter(world).collect::<Vec<_>>();
    entities.sort_by_key(|entity| entity.index());
    entities
}

fn cell_entity_for(world: &mut World, class_id: ClassId) -> Entity {
    let mut query = world.query::<(Entity, &LobbyClassPickerCell)>();
    query
        .iter(world)
        .find_map(|(entity, cell)| (cell.class_id == class_id).then_some(entity))
        .unwrap_or_else(|| panic!("expected LobbyClassPickerCell for {class_id:?}"))
}

fn cell_colors(world: &World, entity: Entity) -> (BackgroundColor, BorderColor) {
    let entity_ref = world.entity(entity);
    (
        *entity_ref
            .get::<BackgroundColor>()
            .expect("class-picker cell carries BackgroundColor"),
        *entity_ref
            .get::<BorderColor>()
            .expect("class-picker cell carries BorderColor"),
    )
}

fn sampled_cell_dimensions(app: &mut App) -> Vec<(ClassId, bool, f32, f32)> {
    let world = app.world_mut();
    let mut query = world.query::<(&LobbyClassPickerCell, &Node)>();
    let mut dimensions = query
        .iter(world)
        .map(|(cell, node)| {
            let width = match node.width {
                Val::Px(value) => value,
                other => panic!("cell {:?} width must be px, got {other:?}", cell.class_id),
            };
            let height = match node.height {
                Val::Px(value) => value,
                other => panic!("cell {:?} height must be px, got {other:?}", cell.class_id),
            };
            (cell.class_id, cell.selectable, width, height)
        })
        .collect::<Vec<_>>();
    dimensions.sort_by_key(|(class_id, selectable, _, _)| (format!("{class_id:?}"), *selectable));
    dimensions
}

#[test]
fn ac1_class_picker_is_one_hierarchical_block() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let panel_entity = {
        let mut panels = world.query_filtered::<Entity, With<LobbyPanel>>();
        panels
            .single(world)
            .expect("single LobbyPanel exists after lobby spawn")
    };

    let block_entities = {
        let mut blocks = world.query_filtered::<Entity, With<LobbyClassPickerBlock>>();
        blocks.iter(world).collect::<Vec<_>>()
    };
    assert_eq!(
        block_entities.len(),
        1,
        "AC1: exactly one hierarchical class-picker block must exist"
    );
    let block_entity = block_entities[0];
    assert!(
        child_entities(world, panel_entity).contains(&block_entity),
        "AC1: LobbyClassPickerBlock must be a direct child of LobbyPanel"
    );

    let block_children = child_entities(world, block_entity);
    let heading_count = block_children
        .iter()
        .filter(|entity| {
            world
                .entity(**entity)
                .get::<LobbyClassPickerHeading>()
                .is_some()
        })
        .count();
    let grid_count = block_children
        .iter()
        .filter(|entity| {
            world
                .entity(**entity)
                .get::<LobbyClassPickerGrid>()
                .is_some()
        })
        .count();

    assert_eq!(heading_count, 1, "AC1: class-picker block owns one heading");
    assert_eq!(grid_count, 1, "AC1: class-picker block owns one grid");
}

#[test]
fn ac2_each_selectable_class_pairs_portrait_and_button_in_one_cell() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let options = lobby_class_options();
    let all_portrait_ids = lobby_all_class_ids();
    assert_eq!(options.len(), LOBBY_CLASS_PICKER_SELECTABLE_COUNT);
    assert_eq!(all_portrait_ids.len(), LOBBY_CLASS_PICKER_GRID_COLUMNS);
    assert!(
        !options.contains(&ClassId::Neutral) && all_portrait_ids.contains(&ClassId::Neutral),
        "AC2: Neutral is portrait-only reconciliation; selectable classes are the six class options"
    );

    let mut cells = world.query::<(Entity, &LobbyClassPickerCell)>();
    let cell_rows = cells.iter(world).map(|(e, c)| (e, *c)).collect::<Vec<_>>();
    assert_eq!(
        cell_rows.len(),
        all_portrait_ids.len(),
        "AC2: one class-picker cell per lobby_all_class_ids() entry"
    );

    let mut portrait_query = world.query::<&LobbyClassPortrait>();
    assert_eq!(
        portrait_query.iter(world).count(),
        all_portrait_ids.len(),
        "AC2: LobbyClassPortrait marker count remains one per lobby_all_class_ids()"
    );

    let mut button_query = world.query::<&LobbyClassButton>();
    assert_eq!(
        button_query.iter(world).count(),
        options.len(),
        "AC2: LobbyClassButton marker count remains one per selectable lobby_class_options()"
    );

    for (cell_entity, cell) in &cell_rows {
        let children = child_entities(world, *cell_entity);
        let has_matching_portrait = children.iter().any(|child| {
            world
                .entity(*child)
                .get::<LobbyClassPortrait>()
                .is_some_and(|portrait| portrait.class_id == cell.class_id)
        });
        let has_matching_button = children.iter().any(|child| {
            world
                .entity(*child)
                .get::<LobbyClassButton>()
                .is_some_and(|button| button.class_id == cell.class_id)
        });

        assert!(
            has_matching_portrait,
            "AC2: cell {:?} must directly own its matching LobbyClassPortrait",
            cell.class_id
        );
        assert_eq!(
            has_matching_button, cell.selectable,
            "AC2: selectable cell {:?} must own a matching LobbyClassButton; \
             non-selectable Neutral cell must not synthesize one",
            cell.class_id
        );
    }
}

#[test]
fn ac3_ac4_grid_columns_fit_minimum_and_hd_viewports() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    let grid_node = {
        let world = app.world_mut();
        let mut grids = world.query_filtered::<&Node, With<LobbyClassPickerGrid>>();
        grids
            .single(world)
            .expect("single LobbyClassPickerGrid exists")
            .clone()
    };
    assert_eq!(grid_node.flex_direction, FlexDirection::Row);
    assert_eq!(grid_node.flex_wrap, FlexWrap::NoWrap);
    assert_eq!(grid_node.column_gap, Val::Px(SPACING_SM));
    assert_eq!(grid_node.justify_content, JustifyContent::Center);

    let required_width = LOBBY_CLASS_PICKER_GRID_COLUMNS as f32 * LOBBY_CLASS_PICKER_CELL_WIDTH_PX
        + (LOBBY_CLASS_PICKER_GRID_COLUMNS - 1) as f32 * SPACING_SM;
    for (label, (viewport_width, _viewport_height)) in
        [("1366x768", VIEWPORT_MIN), ("1920x1080", VIEWPORT_HD)]
    {
        let panel_width =
            (LOBBY_PANEL_WIDTH_PERCENT / 100.0 * viewport_width).min(LOBBY_PANEL_MAX_WIDTH_PX);
        let content_width = panel_width - (2.0 * SPACING_LG);
        assert!(
            required_width <= content_width + 1.0,
            "AC3/AC4: seven-column class grid needs {required_width}px but \
             {label} content width is {content_width}px"
        );
    }

    {
        let world = app.world_mut();
        let mut button_nodes = world.query_filtered::<&Node, With<LobbyClassButton>>();
        for node in button_nodes.iter(world) {
            assert_eq!(node.width, Val::Px(LOBBY_CLASS_PICKER_BUTTON_WIDTH_PX));
            assert_eq!(node.height, Val::Px(30.0));
        }
    }

    let dimensions = sampled_cell_dimensions(&mut app);
    assert_eq!(dimensions.len(), LOBBY_CLASS_PICKER_GRID_COLUMNS);
    for (class_id, _selectable, width, height) in dimensions {
        assert_eq!(
            width, LOBBY_CLASS_PICKER_CELL_WIDTH_PX,
            "AC5: {:?} cell width must be stable and pixel-fixed",
            class_id
        );
        assert_eq!(
            height, LOBBY_CLASS_PICKER_CELL_HEIGHT_PX,
            "AC5: {:?} cell height must be stable and pixel-fixed",
            class_id
        );
    }

    let longest_selected_label = lobby_class_options()
        .iter()
        .map(|class_id| format!("{class_id:?} *").chars().count())
        .max()
        .expect("class options are non-empty");
    let estimated_label_width = longest_selected_label as f32 * typography::BODY * 0.52;
    let inner_button_width = LOBBY_CLASS_PICKER_BUTTON_WIDTH_PX - 16.0;
    assert!(
        estimated_label_width <= inner_button_width,
        "AC4: longest selected class label must fit without ellipsis; \
         estimate {estimated_label_width:.1}px exceeds inner width {inner_button_width:.1}px"
    );
}

#[test]
fn ac5_repeat_lobby_spawns_preserve_cell_dimensions_within_one_pixel() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let first = sampled_cell_dimensions(&mut app);

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app.update();

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Lobby);
    app.update();
    app.update();

    let second = sampled_cell_dimensions(&mut app);
    assert_eq!(first.len(), second.len());
    for ((class_a, selectable_a, width_a, height_a), (class_b, selectable_b, width_b, height_b)) in
        first.iter().zip(second.iter())
    {
        assert_eq!(class_a, class_b);
        assert_eq!(selectable_a, selectable_b);
        assert!(
            (width_a - width_b).abs() <= 1.0,
            "AC5: {:?} width drifted from {width_a} to {width_b}",
            class_a
        );
        assert!(
            (height_a - height_b).abs() <= 1.0,
            "AC5: {:?} height drifted from {height_a} to {height_b}",
            class_a
        );
    }
}

#[test]
fn ac6_selected_cell_affordance_exists_and_refreshes_without_respawn() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    let (initial_entities, iop_before, cra_before) = {
        let world = app.world_mut();
        let entities = sorted_class_cell_entities(world);
        let iop = cell_entity_for(world, ClassId::Iop);
        let cra = cell_entity_for(world, ClassId::Cra);
        (entities, cell_colors(world, iop), cell_colors(world, cra))
    };
    assert_ne!(
        iop_before, cra_before,
        "AC6: default selected Iop cell must have a distinct first-spawn affordance"
    );

    app.world_mut()
        .resource_mut::<LobbyInputState>()
        .selected_class = ClassId::Xelor;
    app.update();

    let (after_entities, iop_after, xelor_after, cra_after) = {
        let world = app.world_mut();
        let entities = sorted_class_cell_entities(world);
        let iop = cell_entity_for(world, ClassId::Iop);
        let xelor = cell_entity_for(world, ClassId::Xelor);
        let cra = cell_entity_for(world, ClassId::Cra);
        (
            entities,
            cell_colors(world, iop),
            cell_colors(world, xelor),
            cell_colors(world, cra),
        )
    };

    assert_eq!(
        initial_entities, after_entities,
        "AC6: selected affordance refresh must not require class-picker respawn"
    );
    assert_ne!(
        xelor_after, cra_after,
        "AC6: newly selected Xelor cell must become visually distinct"
    );
    assert_eq!(
        iop_after, cra_after,
        "AC6: previously selected Iop cell must return to the non-selected affordance"
    );
}
