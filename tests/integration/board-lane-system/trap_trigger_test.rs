use std::collections::HashMap;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitOwner, UnitStats};
use server::core::session::SessionConfig;
use server::feature::board::{
    apply_charge_movement, apply_standard_movement, commit_lane_change_destinations,
    commit_unit_destination, BoardConfig, BoardOccupancy, ChargeBonus, LaneChangeDestination,
    TrapTrigger,
};
use server::feature::prism::PrismCollected;
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;

#[derive(Component)]
struct RepelLanding {
    lane: u8,
    cell: u8,
}

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
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn world_with_board_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoardConfig::default());
    world.insert_resource(session_config());
    world.insert_resource(BoardOccupancy::default());
    world.insert_resource(Messages::<TrapTrigger>::default());
    world.insert_resource(Messages::<PrismCollected>::default());
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

fn spawn_trap(world: &mut World, owner: PlayerId, lane: u8, cell: u8) -> Entity {
    let trap = world.spawn_empty().id();
    world
        .resource_mut::<BoardOccupancy>()
        .traps
        .insert((owner, lane, cell), trap);
    trap
}

fn run_standard_movement(world: &mut World) {
    world
        .run_system_once(apply_standard_movement)
        .expect("standard movement should run");
}

fn run_charge_movement(world: &mut World) {
    world
        .run_system_once(apply_charge_movement)
        .expect("charge movement should run");
}

fn run_repel_displacement(world: &mut World) {
    world
        .run_system_once(commit_repel_landings)
        .expect("repel displacement commit should run");
}

fn run_lane_change_commit(world: &mut World) {
    world
        .run_system_once(commit_lane_change_destinations)
        .expect("lane change commit should run");
}

fn commit_repel_landings(
    mut commands: Commands,
    session_config: Res<SessionConfig>,
    mut occupancy: ResMut<BoardOccupancy>,
    mut trap_triggers: MessageWriter<TrapTrigger>,
    mut units: Query<(Entity, &UnitOwner, &mut BoardPosition, &RepelLanding)>,
) {
    for (unit_entity, owner, mut position, landing) in &mut units {
        commit_unit_destination(
            &mut commands,
            &mut occupancy,
            &session_config,
            &mut trap_triggers,
            unit_entity,
            owner.0,
            &mut *position,
            landing.lane,
            landing.cell,
        );
    }
}

fn unit_position(world: &World, entity: Entity) -> BoardPosition {
    *world
        .get::<BoardPosition>(entity)
        .expect("unit should have a board position")
}

fn trap_triggers(world: &World) -> Vec<TrapTrigger> {
    let messages = world.resource::<Messages<TrapTrigger>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).copied().collect()
}

fn assert_trap_removed(world: &World, owner: PlayerId, lane: u8, cell: u8, trap: Entity) {
    assert!(!world
        .resource::<BoardOccupancy>()
        .traps
        .contains_key(&(owner, lane, cell)));
    assert!(!world.entities().contains(trap));
}

#[test]
fn bl_16_standard_movement_into_enemy_trap_triggers_and_removes_it() {
    let mut world = world_with_board_config();
    let trap = spawn_trap(&mut world, player(2), 1, 4);
    let unit = spawn_unit(&mut world, player(1), 1, 1, 3, None);

    run_standard_movement(&mut world);

    assert_eq!(
        unit_position(&world, unit),
        BoardPosition { lane: 1, cell: 4 }
    );
    assert_trap_removed(&world, player(2), 1, 4, trap);
    assert_eq!(
        trap_triggers(&world),
        vec![TrapTrigger {
            trap_entity: trap,
            unit_entity: unit,
            trap_owner: player(2),
            unit_owner: player(1),
            lane: 1,
            cell: 4,
        }]
    );
}

#[test]
fn bl_17_repel_displacement_into_enemy_trap_triggers_and_removes_it() {
    let mut world = world_with_board_config();
    let trap = spawn_trap(&mut world, player(2), 1, 5);
    let unit = spawn_unit(&mut world, player(1), 1, 8, 0, None);
    world
        .entity_mut(unit)
        .insert(RepelLanding { lane: 1, cell: 5 });

    run_repel_displacement(&mut world);

    assert_eq!(
        unit_position(&world, unit),
        BoardPosition { lane: 1, cell: 5 }
    );
    assert_trap_removed(&world, player(2), 1, 5, trap);
    assert_eq!(
        trap_triggers(&world),
        vec![TrapTrigger {
            trap_entity: trap,
            unit_entity: unit,
            trap_owner: player(2),
            unit_owner: player(1),
            lane: 1,
            cell: 5,
        }]
    );
}

#[test]
fn bl_31_simultaneous_change_lane_triggers_trap_once_by_lower_original_lane() {
    let mut world = world_with_board_config();
    let trap = spawn_trap(&mut world, player(1), 2, 3);
    let lower_lane_unit = spawn_unit(&mut world, player(2), 1, 3, 0, None);
    let higher_lane_unit = spawn_unit(&mut world, player(2), 3, 3, 0, None);
    world
        .entity_mut(lower_lane_unit)
        .insert(LaneChangeDestination {
            original_lane: 1,
            destination_lane: 2,
        });
    world
        .entity_mut(higher_lane_unit)
        .insert(LaneChangeDestination {
            original_lane: 3,
            destination_lane: 2,
        });

    run_lane_change_commit(&mut world);

    assert_eq!(
        unit_position(&world, lower_lane_unit),
        BoardPosition { lane: 2, cell: 3 }
    );
    assert_eq!(
        unit_position(&world, higher_lane_unit),
        BoardPosition { lane: 2, cell: 3 }
    );
    assert_trap_removed(&world, player(1), 2, 3, trap);
    assert_eq!(
        trap_triggers(&world),
        vec![TrapTrigger {
            trap_entity: trap,
            unit_entity: lower_lane_unit,
            trap_owner: player(1),
            unit_owner: player(2),
            lane: 2,
            cell: 3,
        }]
    );
}

#[test]
fn new_007a_charge_landing_on_enemy_trap_triggers_and_removes_it() {
    let mut world = world_with_board_config();
    let trap = spawn_trap(&mut world, player(2), 1, 4);
    let unit = spawn_unit(&mut world, player(1), 1, 2, 0, Some(2));

    run_charge_movement(&mut world);

    assert_eq!(
        unit_position(&world, unit),
        BoardPosition { lane: 1, cell: 4 }
    );
    assert_trap_removed(&world, player(2), 1, 4, trap);
    assert_eq!(
        trap_triggers(&world),
        vec![TrapTrigger {
            trap_entity: trap,
            unit_entity: unit,
            trap_owner: player(2),
            unit_owner: player(1),
            lane: 1,
            cell: 4,
        }]
    );
}
