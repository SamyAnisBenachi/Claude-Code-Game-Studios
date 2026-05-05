use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::board_rendering::rendering_constants::{
    Z_BOARD_CAMERA, Z_CELL_NODES, Z_FIELD_WASH, Z_GHOST_UNIT, Z_HEALTH_BARS, Z_OBJECTIVES,
    Z_TRAPS_STRUCTURES, Z_UNITS,
};
use client::presentation::board_rendering::{
    BoardCamera, BoardCellNode, BoardRenderingPlugin, SpawnHighlightState,
};
use client::presentation::{BoardLayout, LaneCell};
use client::state::ClientState;

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
fn test_spawn_highlight_state_uses_cell_tint_not_extra_z_layer() {
    assert_eq!(
        SpawnHighlightState::Inactive.tint(),
        Color::srgba(0.12, 0.24, 0.30, 0.55)
    );
    assert_eq!(
        SpawnHighlightState::ValidSpawn.tint(),
        Color::srgba(1.0, 0.82, 0.24, 0.88)
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
fn test_board_z_layers_are_named_constants() {
    assert_eq!(Z_FIELD_WASH, 0.0);
    assert_eq!(Z_CELL_NODES, 1.0);
    assert_eq!(Z_TRAPS_STRUCTURES, 2.0);
    assert_eq!(Z_OBJECTIVES, 2.5);
    assert_eq!(Z_UNITS, 3.0);
    assert_eq!(Z_HEALTH_BARS, 3.1);
    assert_eq!(Z_GHOST_UNIT, 3.5);

    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("presentation")
            .join("board_rendering.rs"),
    )
    .expect("board rendering source should be readable");

    assert!(source.contains("rendering_constants::Z_BOARD_CAMERA"));
    assert!(source.contains("rendering_constants::Z_CELL_NODES"));
    assert!(!source.contains("Transform::from_xyz(0.0, 0.0, 999.0)"));
    assert!(!source.contains("Transform::from_xyz(world_xy.x, world_xy.y, 1.0)"));
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
    app.add_plugins(BoardRenderingPlugin);

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    app
}
