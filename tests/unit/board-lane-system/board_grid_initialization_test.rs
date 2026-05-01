use bevy::prelude::*;
use server::feature::board::{
    BoardConfig, BoardGrid, BoardOccupancy, BoardPlugin, PrismState, SpawnRangeState,
    BOARD_CELLS_PER_LANE, BOARD_LANE_COUNT, BOARD_PLAYER_COUNT,
};

#[test]
fn plugin_registers_board_resources() {
    let mut app = App::new();
    app.add_plugins(BoardPlugin);
    app.finish();
    app.cleanup();

    assert!(app.world().contains_resource::<BoardGrid>());
    assert!(app.world().contains_resource::<BoardOccupancy>());
    assert!(app.world().contains_resource::<SpawnRangeState>());
    assert!(app.world().contains_resource::<PrismState>());
    assert!(app.world().contains_resource::<BoardConfig>());
}

#[test]
fn new_001a_board_grid_has_five_lanes_and_eight_empty_cells() {
    let mut world = World::new();
    world.insert_resource(BoardGrid::default());

    let grid = world.resource::<BoardGrid>();

    assert_eq!(grid.lanes.len(), BOARD_LANE_COUNT);
    for lane in &grid.lanes {
        assert_eq!(lane.len(), BOARD_CELLS_PER_LANE);
        assert!(lane.iter().all(Option::is_none));
    }
}

#[test]
fn new_001b_board_config_exposes_direction_and_bounds_constants() {
    let mut world = World::new();
    world.insert_resource(BoardConfig::default());

    let config = world.resource::<BoardConfig>();

    assert_eq!(config.player_a_direction, 1i16);
    assert_eq!(config.player_b_direction, -1i16);
    assert_eq!(config.player_a_spawn_cell, 1);
    assert_eq!(config.player_b_spawn_cell, 8);
    assert_eq!(config.player_a_objective_cell, 8);
    assert_eq!(config.player_b_objective_cell, 1);
    assert_eq!(config.lane_count, BOARD_LANE_COUNT as u8);
    assert_eq!(config.cells_per_lane, BOARD_CELLS_PER_LANE as u8);
    assert_eq!(config.cell_min, 1);
    assert_eq!(config.cell_max, 8);
}

#[test]
fn bl_21_minion_slots_start_empty_for_both_players() {
    let mut world = World::new();
    world.insert_resource(BoardOccupancy::default());

    let occupancy = world.resource::<BoardOccupancy>();

    assert_eq!(occupancy.minion_slots.len(), BOARD_PLAYER_COUNT);
    for player_slots in &occupancy.minion_slots {
        assert_eq!(player_slots.len(), BOARD_LANE_COUNT);
        assert!(player_slots.iter().all(Option::is_none));
    }
    assert!(occupancy.traps.is_empty());
    assert!(occupancy.structures.is_empty());
    assert!(occupancy.fields.is_empty());
}
