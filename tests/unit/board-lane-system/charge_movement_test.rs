use std::collections::HashMap;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitOwner, UnitStats};
use server::core::session::SessionConfig;
use server::feature::board::{
    apply_charge_movement, apply_standard_movement, BoardConfig, BoardOccupancy, ChargeBonus,
    TrapTrigger,
};
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
    world.insert_resource(Messages::<TrapTrigger>::default());
    world
}

fn spawn_unit(
    world: &mut World,
    owner: PlayerId,
    lane: u8,
    cell: u8,
    mp: u8,
    charge: Option<u8>,
) -> Entity {
    let mut entity = world.spawn((
        BoardPosition { lane, cell },
        UnitStats::new(1, 1, mp, 0),
        UnitOwner(owner),
    ));

    if let Some(charge) = charge {
        entity.insert(ChargeBonus(charge));
    }

    entity.id()
}

fn run_charge_movement(world: &mut World) {
    world
        .run_system_once(apply_charge_movement)
        .expect("charge movement system should run");
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

fn trap_trigger_count(world: &World) -> usize {
    let messages = world.resource::<Messages<TrapTrigger>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).count()
}

#[test]
fn test_bl_22_charge_movement_runs_before_standard_movement() {
    let mut world = world_with_board_config();
    let unit = spawn_unit(&mut world, player(1), 1, 1, 3, Some(2));

    run_charge_movement(&mut world);

    assert_eq!(unit_cell(&world, unit), 3);

    run_standard_movement(&mut world);

    assert_eq!(unit_cell(&world, unit), 6);
}

#[test]
fn test_bl_22_charge_and_standard_movement_clamp_independently() {
    let mut world = world_with_board_config();
    let charge_then_standard = spawn_unit(&mut world, player(1), 1, 1, 3, Some(5));
    let charge_to_boundary = spawn_unit(&mut world, player(1), 1, 1, 3, Some(7));

    run_charge_movement(&mut world);

    assert_eq!(unit_cell(&world, charge_then_standard), 6);
    assert_eq!(unit_cell(&world, charge_to_boundary), 8);

    run_standard_movement(&mut world);

    assert_eq!(unit_cell(&world, charge_then_standard), 8);
    assert_eq!(unit_cell(&world, charge_to_boundary), 8);
}

#[test]
fn test_units_without_charge_bonus_skip_sub_step_2() {
    let mut world = world_with_board_config();
    let charged = spawn_unit(&mut world, player(1), 1, 1, 3, Some(2));
    let uncharged = spawn_unit(&mut world, player(1), 2, 1, 3, None);

    run_charge_movement(&mut world);

    assert_eq!(unit_cell(&world, charged), 3);
    assert_eq!(unit_cell(&world, uncharged), 1);
}

#[test]
fn test_bl_27b_charge_movement_skips_intermediate_trap_cell() {
    let mut world = world_with_board_config();
    let trap = world.spawn_empty().id();
    world
        .resource_mut::<BoardOccupancy>()
        .traps
        .insert((player(2), 1, 2), trap);
    let unit = spawn_unit(&mut world, player(1), 1, 1, 0, Some(3));

    run_charge_movement(&mut world);

    assert_eq!(unit_cell(&world, unit), 4);
    assert_eq!(trap_trigger_count(&world), 0);
    assert_eq!(
        world
            .resource::<BoardOccupancy>()
            .traps
            .get(&(player(2), 1, 2))
            .copied(),
        Some(trap)
    );
}
