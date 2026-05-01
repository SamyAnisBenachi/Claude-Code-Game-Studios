use std::collections::HashMap;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitOwner, UnitStats};
use server::core::session::SessionConfig;
use server::feature::board::{apply_f1, apply_standard_movement, BoardConfig, BoardOccupancy};
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_config() -> SessionConfig {
    let player_a = player(1);
    let player_b = player(2);

    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(player_a, 0), (player_b, 1)]),
        class_map: HashMap::from([(player_a, ClassId::Iop), (player_b, ClassId::Cra)]),
    }
}

fn world_with_board_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoardConfig::default());
    world.insert_resource(session_config());
    world.insert_resource(BoardOccupancy::default());
    world
}

fn spawn_unit(world: &mut World, owner: PlayerId, lane: u8, cell: u8, mp: u8) -> Entity {
    world
        .spawn((
            BoardPosition { lane, cell },
            UnitStats::new(1, 1, mp, 0),
            UnitOwner(owner),
        ))
        .id()
}

fn run_standard_movement(world: &mut World) {
    world
        .run_system_once(apply_standard_movement)
        .expect("standard movement system should run");
}

fn unit_cell(world: &World, entity: Entity) -> u8 {
    world
        .get::<BoardPosition>(entity)
        .expect("unit should have a board position")
        .cell
}

#[test]
fn test_bl_1_player_a_standard_movement_advances_to_cell_4() {
    let mut world = world_with_board_config();
    let unit = spawn_unit(&mut world, player(1), 1, 1, 3);

    run_standard_movement(&mut world);

    let position = world
        .get::<BoardPosition>(unit)
        .expect("unit should have a board position");
    assert_eq!(position.lane, 1);
    assert_eq!(position.cell, 4);
}

#[test]
fn test_bl_2_player_a_standard_movement_clamps_at_cell_8() {
    let mut world = world_with_board_config();
    let unit = spawn_unit(&mut world, player(1), 1, 6, 3);
    let boundary_unit = spawn_unit(&mut world, player(1), 1, 8, 1);

    run_standard_movement(&mut world);

    assert_eq!(unit_cell(&world, unit), 8);
    assert_eq!(unit_cell(&world, boundary_unit), 8);
}

#[test]
fn test_bl_3_player_b_standard_movement_uses_negative_direction() {
    let mut world = world_with_board_config();
    let unit = spawn_unit(&mut world, player(2), 1, 5, 2);

    run_standard_movement(&mut world);

    assert_eq!(unit_cell(&world, unit), 3);
}

#[test]
fn test_bl_3_player_b_movement_uses_i16_intermediate_before_clamping() {
    assert_eq!(apply_f1(3, -1, 5, 1, 8), 1);
    assert_eq!(apply_f1(1, -1, 1, 1, 8), 1);
}

#[test]
fn test_bl_4_wall_units_with_zero_movement_points_do_not_move() {
    let mut world = world_with_board_config();
    let player_a_wall = spawn_unit(&mut world, player(1), 1, 1, 0);
    let player_b_wall = spawn_unit(&mut world, player(2), 1, 8, 0);

    run_standard_movement(&mut world);

    assert_eq!(unit_cell(&world, player_a_wall), 1);
    assert_eq!(unit_cell(&world, player_b_wall), 8);
}

#[test]
fn test_bl_27_standard_movement_skips_intermediate_trap_cell() {
    let mut world = world_with_board_config();
    let trap = world.spawn_empty().id();
    world
        .resource_mut::<BoardOccupancy>()
        .traps
        .insert((player(2), 1, 3), trap);
    let unit = spawn_unit(&mut world, player(1), 1, 1, 3);

    run_standard_movement(&mut world);

    assert_eq!(unit_cell(&world, unit), 4);
    assert_eq!(
        world
            .resource::<BoardOccupancy>()
            .traps
            .get(&(player(2), 1, 3))
            .copied(),
        Some(trap)
    );
}
