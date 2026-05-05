use std::collections::HashMap;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitOwner, UnitStats};
use server::core::session::SessionConfig;
use server::feature::board::{
    apply_charge_movement, apply_standard_movement, commit_unit_destination, BoardConfig,
    BoardOccupancy, ChargeBonus, TrapTrigger,
};
use server::feature::prism::{PrismCollected, PrismLaneKey, PrismPresence};
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;

#[derive(Component)]
struct TeleportLanding {
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

fn spawn_prism(world: &mut World, owner: PlayerId, lane: u8, collected: bool) {
    world.spawn((
        PrismLaneKey {
            player: owner,
            lane,
        },
        PrismPresence { collected },
    ));
}

fn run_standard_movement(world: &mut World) {
    world
        .run_system_once(apply_standard_movement)
        .expect("standard movement system should run");
}

fn run_charge_movement(world: &mut World) {
    world
        .run_system_once(apply_charge_movement)
        .expect("charge movement system should run");
}

fn run_teleport_commit(world: &mut World) {
    world
        .run_system_once(commit_teleport_landings)
        .expect("teleport commit system should run");
}

fn commit_teleport_landings(
    mut commands: Commands,
    session_config: Res<SessionConfig>,
    mut occupancy: ResMut<BoardOccupancy>,
    mut trap_triggers: MessageWriter<TrapTrigger>,
    mut units: Query<(Entity, &UnitOwner, &mut BoardPosition, &TeleportLanding)>,
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

fn unit_cell(world: &World, entity: Entity) -> u8 {
    world
        .get::<BoardPosition>(entity)
        .expect("unit should have a board position")
        .cell
}

fn prism_collections(world: &World) -> Vec<PrismCollected> {
    let messages = world.resource::<Messages<PrismCollected>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).copied().collect()
}

#[test]
fn test_bl_12_wall_unit_at_own_prism_cell_emits_collection() {
    let mut world = world_with_board_config();
    spawn_prism(&mut world, player(1), 2, false);
    spawn_unit(&mut world, player(1), 2, 1, 0, None);

    run_standard_movement(&mut world);

    assert_eq!(
        prism_collections(&world),
        vec![PrismCollected {
            player_id: player(1),
            lane: 2,
        }]
    );
}

#[test]
fn test_bl_12_collected_prism_does_not_emit_duplicate_collection() {
    let mut world = world_with_board_config();
    spawn_prism(&mut world, player(1), 2, true);
    spawn_unit(&mut world, player(1), 2, 1, 0, None);

    run_standard_movement(&mut world);

    assert!(prism_collections(&world).is_empty());
}

#[test]
fn test_bl_13_player_b_at_cell_1_does_not_collect_player_a_prism() {
    let mut world = world_with_board_config();
    spawn_prism(&mut world, player(2), 1, false);
    spawn_unit(&mut world, player(2), 1, 1, 0, None);

    run_standard_movement(&mut world);

    assert!(prism_collections(&world).is_empty());
}

#[test]
fn test_bl_13_player_b_at_cell_8_collects_own_prism() {
    let mut world = world_with_board_config();
    spawn_prism(&mut world, player(2), 1, false);
    spawn_unit(&mut world, player(2), 1, 8, 0, None);

    run_standard_movement(&mut world);

    assert_eq!(
        prism_collections(&world),
        vec![PrismCollected {
            player_id: player(2),
            lane: 1,
        }]
    );
}

#[test]
fn test_bl_18_teleport_to_spawn_cell_does_not_collect_prism() {
    let mut world = world_with_board_config();
    spawn_prism(&mut world, player(1), 4, false);
    let unit = spawn_unit(&mut world, player(1), 4, 5, 0, None);
    world
        .entity_mut(unit)
        .insert(TeleportLanding { lane: 4, cell: 1 });

    run_teleport_commit(&mut world);

    assert_eq!(unit_cell(&world, unit), 1);
    assert!(prism_collections(&world).is_empty());
}

#[test]
fn test_bl_30_charge_plus_standard_movement_ending_away_from_prism_emits_nothing() {
    let mut world = world_with_board_config();
    spawn_prism(&mut world, player(1), 3, false);
    let unit = spawn_unit(&mut world, player(1), 3, 1, 2, Some(2));

    run_charge_movement(&mut world);
    run_standard_movement(&mut world);

    assert_eq!(unit_cell(&world, unit), 5);
    assert!(prism_collections(&world).is_empty());
}
