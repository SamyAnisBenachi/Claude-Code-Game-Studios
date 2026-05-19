use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::board_rendering::rendering_constants::{
    Z_BOARD_CAMERA, Z_CELL_NODES, Z_FIELD_WASH, Z_GHOST_UNIT, Z_GRID_OVERLAY, Z_HEALTH_BARS,
    Z_LANE_RAILS, Z_LANE_SURFACE, Z_OBJECTIVES, Z_TRAPS_STRUCTURES, Z_UNITS,
};
use client::presentation::board_rendering::{
    BoardCamera, BoardCellNode, BoardGridOverlayLine, BoardGridOverlayState,
    BoardGridOverlayToggleButton, BoardLaneRail, BoardLaneSurface, BoardRenderingPlugin,
    SpawnHighlightState,
};
use client::presentation::{BoardLayout, LaneCell};
use client::state::ClientState;
use client::ui::shared::{BOARD_CELL_COUNT, BOARD_LANE_COUNT};
use shared::session::PlayerId;

#[test]
fn test_board_camera_spawns_single_fixed_orthographic_view() {
    let mut app = app_in_session();
    let world = app.world_mut();

    let mut cameras =
        world.query_filtered::<(&Transform, &Projection), (With<Camera2d>, With<BoardCamera>)>();
    let camera_rows: Vec<_> = cameras.iter(world).collect();

    assert_eq!(camera_rows.len(), 1);
    let (transform, projection) = camera_rows[0];
    assert!(matches!(projection, Projection::Orthographic(_)));
    assert!(!projection.is_perspective());
    assert_eq!(transform.translation.z, Z_BOARD_CAMERA);
    assert!(transform.translation.z > Z_GHOST_UNIT);

    let mut all_camera_projections = world.query_filtered::<&Projection, With<Camera2d>>();
    assert!(all_camera_projections
        .iter(world)
        .all(|projection| !projection.is_perspective()));
}

#[test]
fn test_board_grid_spawns_all_lane_cell_nodes() {
    let mut app = app_in_session();
    let world = app.world_mut();
    let layout = *world
        .get_resource::<BoardLayout>()
        .expect("BoardLayout should exist during session");

    let mut cells = world.query_filtered::<
        (&LaneCell, &Transform, &Sprite, &SpawnHighlightState),
        With<BoardCellNode>,
    >();
    let mut seen = BTreeSet::new();

    for (lane_cell, transform, sprite, highlight_state) in cells.iter(world) {
        seen.insert((lane_cell.lane, lane_cell.cell));

        let world_xy = layout.cell_to_world(lane_cell.lane, lane_cell.cell);
        assert_eq!(transform.translation.x, world_xy.x);
        assert_eq!(transform.translation.y, world_xy.y);
        assert_eq!(transform.translation.z, Z_CELL_NODES);
        assert_eq!(sprite.color, highlight_state.tint());
    }

    assert_eq!(seen.len(), 40);
    assert_eq!(
        seen,
        (1..=5)
            .flat_map(|lane| (1..=8).map(move |cell| (lane, cell)))
            .collect::<BTreeSet<_>>()
    );
}

#[test]
fn test_board_cells_are_world_space_sprites_not_ui_nodes() {
    let mut app = app_in_session();
    let world = app.world_mut();

    let mut cells = world.query_filtered::<
        (&Sprite, &Transform, Option<&Node>, &SpawnHighlightState),
        With<BoardCellNode>,
    >();
    let cell_rows: Vec<_> = cells.iter(world).collect();

    assert_eq!(cell_rows.len(), 40);
    for (sprite, transform, node, highlight_state) in cell_rows {
        assert!(node.is_none());
        assert_eq!(sprite.color, highlight_state.tint());
        assert_eq!(transform.translation.z, Z_CELL_NODES);
    }
}

#[test]
fn test_board_lane_surfaces_add_physical_lanes_without_picking() {
    let mut app = app_in_session();
    let world = app.world_mut();
    let layout = *world
        .get_resource::<BoardLayout>()
        .expect("BoardLayout should exist during session");

    let mut surfaces = world.query_filtered::<
        (&BoardLaneSurface, &Sprite, &Transform, Option<&Pickable>),
        With<BoardLaneSurface>,
    >();
    let surface_rows: Vec<_> = surfaces.iter(world).collect();
    assert_eq!(surface_rows.len(), usize::from(BOARD_LANE_COUNT));

    for (surface, sprite, transform, pickable) in surface_rows {
        let lane_start = layout.cell_to_world(surface.lane, 1);
        let lane_end = layout.cell_to_world(surface.lane, BOARD_CELL_COUNT);
        let center = (lane_start + lane_end) * 0.5;
        assert_eq!(transform.translation.x, center.x);
        assert_eq!(transform.translation.y, center.y);
        assert_eq!(transform.translation.z, Z_LANE_SURFACE);
        assert_eq!(
            sprite
                .custom_size
                .expect("lane surface size should be fixed")
                .y,
            layout.lane_height * 0.82
        );
        assert!(
            pickable.is_none(),
            "lane surfaces must not participate in placement picking"
        );
    }

    let mut rails = world.query_filtered::<(&Transform, Option<&Pickable>), With<BoardLaneRail>>();
    let rail_rows: Vec<_> = rails.iter(world).collect();
    assert_eq!(rail_rows.len(), usize::from(BOARD_LANE_COUNT + 1));
    for (transform, pickable) in rail_rows {
        assert_eq!(transform.translation.z, Z_LANE_RAILS);
        assert!(
            pickable.is_none(),
            "lane rails must not participate in placement picking"
        );
    }
}

#[test]
fn test_spawn_highlight_state_uses_cell_tint_not_extra_z_layer() {
    assert_eq!(
        SpawnHighlightState::Inactive.tint(),
        Color::srgba(0.17, 0.29, 0.33, 0.72)
    );
    assert_eq!(
        SpawnHighlightState::ValidSpawn {
            player_id: PlayerId(1),
        }
        .tint(),
        Color::srgba(1.0, 0.76, 0.18, 0.94)
    );

    let constants = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("presentation")
            .join("board_rendering")
            .join("rendering_constants.rs"),
    )
    .expect("rendering constants should be readable");

    assert!(!constants.contains("Z_SPAWN"));
}

#[test]
fn qa_grid_overlay_toggle_spawns_visible_button_disabled_by_default() {
    let mut app = app_in_session();
    let world = app.world_mut();

    assert!(
        !world.resource::<BoardGridOverlayState>().enabled,
        "QA grid overlay should default off"
    );

    let mut buttons =
        world.query_filtered::<(&Text, &Interaction), With<BoardGridOverlayToggleButton>>();
    let button_rows: Vec<_> = buttons.iter(world).collect();

    assert_eq!(button_rows.len(), 1);
    let (label, interaction) = button_rows[0];
    assert_eq!(label.0, "QA Grid: OFF");
    assert_eq!(*interaction, Interaction::None);
}

#[test]
fn qa_grid_overlay_toggle_flips_state_and_renders_non_pickable_lines() {
    let mut app = app_in_session();
    let button = qa_grid_overlay_button_entity(&mut app);

    set_button_interaction(&mut app, button, Interaction::Pressed);
    app.update();

    assert!(app.world().resource::<BoardGridOverlayState>().enabled);
    let world = app.world_mut();
    let mut lines =
        world.query_filtered::<(&Transform, Option<&Pickable>), With<BoardGridOverlayLine>>();
    let line_rows: Vec<_> = lines.iter(world).collect();

    assert_eq!(
        line_rows.len(),
        usize::from(BOARD_CELL_COUNT + 1 + BOARD_LANE_COUNT + 1)
    );
    for (transform, pickable) in line_rows {
        assert_eq!(transform.translation.z, Z_GRID_OVERLAY);
        assert!(
            pickable.is_none(),
            "QA grid overlay lines must not participate in picking"
        );
    }

    let mut labels = world.query_filtered::<&Text, With<BoardGridOverlayToggleButton>>();
    let label = labels
        .single(world)
        .expect("QA grid toggle button should exist");
    assert_eq!(label.0, "QA Grid: ON");
}

#[test]
fn qa_grid_overlay_toggle_second_press_removes_lines() {
    let mut app = app_in_session();
    let button = qa_grid_overlay_button_entity(&mut app);

    set_button_interaction(&mut app, button, Interaction::Pressed);
    app.update();
    set_button_interaction(&mut app, button, Interaction::None);
    app.update();
    set_button_interaction(&mut app, button, Interaction::Pressed);
    app.update();

    assert!(!app.world().resource::<BoardGridOverlayState>().enabled);
    let world = app.world_mut();
    let mut lines = world.query_filtered::<Entity, With<BoardGridOverlayLine>>();
    assert_eq!(lines.iter(world).count(), 0);
}

#[test]
fn test_board_z_layers_are_named_constants() {
    assert_eq!(Z_FIELD_WASH, 0.0);
    assert_eq!(Z_LANE_SURFACE, 0.45);
    assert_eq!(Z_LANE_RAILS, 0.55);
    assert_eq!(Z_CELL_NODES, 1.0);
    assert_eq!(Z_TRAPS_STRUCTURES, 2.0);
    assert_eq!(Z_OBJECTIVES, 2.5);
    assert_eq!(Z_UNITS, 3.0);
    assert_eq!(Z_HEALTH_BARS, 3.1);
    assert_eq!(Z_GHOST_UNIT, 3.5);
    assert_eq!(Z_GRID_OVERLAY, 3.6);

    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("presentation")
            .join("board_rendering.rs"),
    )
    .expect("board rendering source should be readable");

    assert!(source.contains("rendering_constants::Z_BOARD_CAMERA"));
    assert!(source.contains("rendering_constants::Z_CELL_NODES"));

    let violations = inline_z_literal_violations(&source);
    assert!(
        violations.is_empty(),
        "board rendering spawn code must use named Z constants; inline Z literals found:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_board_entities_are_removed_on_session_exit() {
    let mut app = app_in_session();

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Lobby);
    app.update();

    let world = app.world_mut();
    let mut cameras = world.query_filtered::<Entity, With<BoardCamera>>();
    let mut cells = world.query_filtered::<Entity, With<BoardCellNode>>();

    assert_eq!(cameras.iter(world).count(), 0);
    assert_eq!(cells.iter(world).count(), 0);
}

fn app_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(BoardRenderingPlugin);

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    app
}

fn qa_grid_overlay_button_entity(app: &mut App) -> Entity {
    let world = app.world_mut();
    let mut buttons = world.query_filtered::<Entity, With<BoardGridOverlayToggleButton>>();
    buttons
        .single(world)
        .expect("QA board grid overlay toggle button should exist")
}

fn set_button_interaction(app: &mut App, button: Entity, interaction: Interaction) {
    *app.world_mut()
        .entity_mut(button)
        .get_mut::<Interaction>()
        .expect("QA grid toggle button should have Interaction") = interaction;
}

fn inline_z_literal_violations(source: &str) -> Vec<String> {
    let mut violations = Vec::new();
    collect_z_arg_violations(source, "Transform::from_xyz", &mut violations);
    collect_z_arg_violations(source, "translation: Vec3::new", &mut violations);
    violations
}

fn collect_z_arg_violations(source: &str, marker: &str, violations: &mut Vec<String>) {
    let mut search_start = 0;
    while let Some(marker_offset) = source[search_start..].find(marker) {
        let marker_start = search_start + marker_offset;
        let Some(open_offset) = source[marker_start..].find('(') else {
            break;
        };
        let open = marker_start + open_offset;
        let Some(close) = matching_paren(source, open) else {
            violations.push(format!(
                "line {}: `{marker}` has no matching `)`",
                line_number(source, marker_start)
            ));
            break;
        };
        let args = split_top_level_args(&source[open + 1..close]);
        let Some(z_arg) = args.get(2).map(|arg| arg.trim()) else {
            violations.push(format!(
                "line {}: `{marker}` has fewer than three arguments",
                line_number(source, marker_start)
            ));
            search_start = close + 1;
            continue;
        };

        if has_numeric_literal(z_arg) {
            violations.push(format!(
                "line {}: `{marker}` z argument `{z_arg}` contains an inline numeric literal",
                line_number(source, marker_start)
            ));
        }
        search_start = close + 1;
    }
}

fn matching_paren(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, byte) in source.bytes().enumerate().skip(open) {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_top_level_args(args: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;

    for (index, byte) in args.bytes().enumerate() {
        match byte {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                parts.push(&args[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }

    parts.push(&args[start..]);
    parts
}

fn has_numeric_literal(expression: &str) -> bool {
    let bytes = expression.as_bytes();
    for (index, byte) in bytes.iter().copied().enumerate() {
        let starts_number = byte.is_ascii_digit()
            || (byte == b'.'
                && bytes
                    .get(index + 1)
                    .is_some_and(|next| next.is_ascii_digit()));
        if !starts_number {
            continue;
        }

        let previous_is_identifier = index
            .checked_sub(1)
            .and_then(|previous| bytes.get(previous))
            .is_some_and(|previous| previous.is_ascii_alphanumeric() || *previous == b'_');
        if !previous_is_identifier {
            return true;
        }
    }
    false
}

fn line_number(source: &str, byte_index: usize) -> usize {
    source[..byte_index]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}
