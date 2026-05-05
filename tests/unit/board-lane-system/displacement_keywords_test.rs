use std::collections::HashMap;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitOwner};
use server::core::economy::AwardGold;
use server::core::objective_contract::ObjectiveCounters;
use server::core::session::SessionConfig;
use server::feature::board::{
    apply_attract, apply_attract_displacements, apply_change_lane, apply_change_lane_displacements,
    apply_repel, apply_repel_displacements, update_spawn_range, AttractDisplacement, BoardConfig,
    BoardOccupancy, ChangeLaneDisplacement, FakeObjectiveDestroyed, Irremovable, LaneId,
    RepelDisplacement, SpawnRangeState, TrapTrigger,
};
use server::feature::objective::{
    apply_consequence_path, HiddenObjectives, ObjectiveHp, ObjectiveSlot, PendingObjectiveEvents,
};
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;

const PLAYER_A: PlayerId = PlayerId(1);
const PLAYER_B: PlayerId = PlayerId(2);

fn session_config() -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(PLAYER_A, 0), (PLAYER_B, 1)]),
        class_map: HashMap::from([(PLAYER_A, ClassId::Iop), (PLAYER_B, ClassId::Cra)]),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn world_with_board() -> World {
    let mut world = World::new();
    world.insert_resource(BoardConfig::default());
    world.insert_resource(session_config());
    world.insert_resource(BoardOccupancy::default());
    world.insert_resource(SpawnRangeState::default());
    world.insert_resource(Messages::<TrapTrigger>::default());
    world.insert_resource(Messages::<RepelDisplacement>::default());
    world.insert_resource(Messages::<AttractDisplacement>::default());
    world.insert_resource(Messages::<ChangeLaneDisplacement>::default());
    world.insert_resource(Messages::<FakeObjectiveDestroyed>::default());
    world
}

fn spawn_unit(world: &mut World, owner: PlayerId, lane: LaneId, cell: u8) -> Entity {
    world
        .spawn((BoardPosition { lane, cell }, UnitOwner(owner)))
        .id()
}

fn unit_position(world: &World, entity: Entity) -> BoardPosition {
    *world
        .get::<BoardPosition>(entity)
        .expect("unit should have a board position")
}

fn trap_trigger_count(world: &World) -> usize {
    let messages = world.resource::<Messages<TrapTrigger>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).count()
}

#[test]
fn test_bl_15_repel_clamps_player_a_spawn_boundary() {
    let config = BoardConfig::default();

    assert_eq!(
        apply_repel(2, PLAYER_A, 3, &session_config(), &config),
        Some(1)
    );
    assert_eq!(
        apply_repel(1, PLAYER_A, 1, &session_config(), &config),
        Some(1)
    );
    assert_eq!(
        apply_repel(5, PLAYER_A, 2, &session_config(), &config),
        Some(3)
    );
}

#[test]
fn test_new_010b_repel_clamps_player_b_spawn_boundary() {
    let config = BoardConfig::default();

    assert_eq!(
        apply_repel(7, PLAYER_B, 3, &session_config(), &config),
        Some(8)
    );
    assert_eq!(
        apply_repel(8, PLAYER_B, 1, &session_config(), &config),
        Some(8)
    );
}

#[test]
fn test_bl_24_and_new_010a_attract_uses_caster_relative_direction() {
    let config = BoardConfig::default();

    assert_eq!(apply_attract(8, 5, 5, true, &config), 7);
    assert_eq!(apply_attract(2, 6, 3, true, &config), 3);
    assert_eq!(apply_attract(3, 3, 10, true, &config), 3);
    assert_eq!(apply_attract(1, 6, 10, false, &config), 1);
}

#[test]
fn test_bl_19_bl_20_bl_28_change_lane_no_ops_at_boundaries_and_full_slots() {
    let mut world = World::new();
    let mut occupancy = BoardOccupancy::default();
    let config = BoardConfig::default();
    occupancy
        .minion_slots
        .insert((PLAYER_A, 2), world.spawn_empty().id());
    occupancy
        .minion_slots
        .insert((PLAYER_A, 4), world.spawn_empty().id());

    assert_eq!(apply_change_lane(1, -1, &occupancy, PLAYER_A, &config), 1);
    assert_eq!(apply_change_lane(5, 1, &occupancy, PLAYER_A, &config), 5);
    assert_eq!(apply_change_lane(3, -1, &occupancy, PLAYER_A, &config), 3);
    assert_eq!(apply_change_lane(3, 1, &occupancy, PLAYER_A, &config), 3);
    assert_eq!(
        apply_change_lane(3, 1, &BoardOccupancy::default(), PLAYER_A, &config),
        4
    );
}

#[test]
fn test_bl_23_irremovable_repel_silently_keeps_cell() {
    let mut world = world_with_board();
    let unit = spawn_unit(&mut world, PLAYER_A, 1, 4);
    world.entity_mut(unit).insert(Irremovable);
    world
        .resource_mut::<Messages<RepelDisplacement>>()
        .write(RepelDisplacement {
            target: unit,
            amount: 3,
        });

    world
        .run_system_once(apply_repel_displacements)
        .expect("repel displacement system should run");

    assert_eq!(unit_position(&world, unit).cell, 4);
    assert_eq!(trap_trigger_count(&world), 0);
}

#[test]
fn test_irremovable_attract_and_change_lane_silently_keep_position() {
    let mut world = world_with_board();
    let caster = spawn_unit(&mut world, PLAYER_A, 1, 8);
    let target = spawn_unit(&mut world, PLAYER_B, 3, 5);
    world.entity_mut(target).insert(Irremovable);
    world
        .resource_mut::<Messages<AttractDisplacement>>()
        .write(AttractDisplacement {
            caster,
            target,
            amount: 3,
        });
    world
        .resource_mut::<Messages<ChangeLaneDisplacement>>()
        .write(ChangeLaneDisplacement { target, delta: 1 });

    world
        .run_system_once(apply_attract_displacements)
        .expect("attract displacement system should run");
    world
        .run_system_once(apply_change_lane_displacements)
        .expect("change lane displacement system should run");

    assert_eq!(
        unit_position(&world, target),
        BoardPosition { lane: 3, cell: 5 }
    );
}

#[test]
fn test_change_lane_displacement_commits_valid_lane_and_occupancy_slot() {
    let mut world = world_with_board();
    let unit = spawn_unit(&mut world, PLAYER_A, 3, 4);
    world
        .resource_mut::<BoardOccupancy>()
        .minion_slots
        .insert((PLAYER_A, 3), unit);
    world
        .resource_mut::<Messages<ChangeLaneDisplacement>>()
        .write(ChangeLaneDisplacement {
            target: unit,
            delta: 1,
        });

    world
        .run_system_once(apply_change_lane_displacements)
        .expect("change lane displacement system should run");

    assert_eq!(
        unit_position(&world, unit),
        BoardPosition { lane: 4, cell: 4 }
    );
    assert!(!world
        .resource::<BoardOccupancy>()
        .minion_slots
        .contains_key(&(PLAYER_A, 3)));
    assert_eq!(
        world
            .resource::<BoardOccupancy>()
            .minion_slots
            .get(&(PLAYER_A, 4))
            .copied(),
        Some(unit)
    );
}

#[test]
fn test_fake_objective_consequence_emits_spawn_range_fact_and_records_counter() {
    let mut world = world_with_board();
    world.insert_resource(Messages::<AwardGold>::default());
    world.insert_resource(ObjectiveCounters::default());
    world.insert_resource(PendingObjectiveEvents::default());
    world.insert_resource(HiddenObjectives {
        identities: HashMap::from([((PLAYER_B, 2), true)]),
    });
    world.spawn((
        ObjectiveSlot {
            lane: 2,
            player: PLAYER_B,
            destroyed: false,
        },
        ObjectiveHp { hp: 1 },
    ));

    apply_consequence_path(&mut world, 2, PLAYER_A, PLAYER_B);

    let messages = world.resource::<Messages<FakeObjectiveDestroyed>>();
    let mut cursor = messages.get_cursor();
    let events = cursor.read(messages).cloned().collect::<Vec<_>>();
    assert_eq!(
        events,
        vec![FakeObjectiveDestroyed {
            destroyed_by: PLAYER_A
        }]
    );
    assert_eq!(
        world
            .resource::<PendingObjectiveEvents>()
            .queue
            .iter()
            .map(|event| (event.target_player_id, event.lane, event.was_fake))
            .collect::<Vec<_>>(),
        vec![(PLAYER_B, 2, true)]
    );
    assert_eq!(
        world
            .resource::<ObjectiveCounters>()
            .fake_objectives_destroyed(PLAYER_A),
        1
    );
}

#[test]
fn test_new_010c_fake_objective_destroyed_expands_next_spawn_range() {
    let mut world = world_with_board();
    {
        let mut messages = world.resource_mut::<Messages<FakeObjectiveDestroyed>>();
        messages.write(FakeObjectiveDestroyed {
            destroyed_by: PLAYER_A,
        });
        messages.write(FakeObjectiveDestroyed {
            destroyed_by: PLAYER_A,
        });
        messages.write(FakeObjectiveDestroyed {
            destroyed_by: PLAYER_A,
        });
    }

    world
        .run_system_once(update_spawn_range)
        .expect("spawn range update system should run");

    let spawn_range = world.resource::<SpawnRangeState>();
    assert_eq!(spawn_range.fakes_destroyed[0], 2);
    assert!(server::feature::board::validate_spawn_range(
        2,
        PLAYER_A,
        spawn_range.fakes_destroyed[0],
        &session_config(),
        &BoardConfig::default(),
    ));
    assert!(!server::feature::board::validate_spawn_range(
        2,
        PLAYER_A,
        0,
        &session_config(),
        &BoardConfig::default(),
    ));
    assert!(!server::feature::board::validate_spawn_range(
        4,
        PLAYER_A,
        spawn_range.fakes_destroyed[0],
        &session_config(),
        &BoardConfig::default(),
    ));
}
