use std::collections::HashMap;

use bevy::prelude::*;
use server::core::session::{SessionConfig, TeamId};
use server::feature::board::LaneId;
use server::feature::objective::{
    take_damage, HiddenObjectives, ObjectiveDestroyed, ObjectiveHp, ObjectiveSlot,
    PendingObjectiveEvents,
};
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;

const ATTACKER: PlayerId = PlayerId(1);
const DEFENDER: PlayerId = PlayerId(2);
const LANE: LaneId = 3;

fn session_config() -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(ATTACKER, 1 as TeamId), (DEFENDER, 2 as TeamId)]),
        class_map: HashMap::from([(ATTACKER, ClassId::Iop), (DEFENDER, ClassId::Cra)]),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn hidden_objectives(was_fake: bool) -> HiddenObjectives {
    HiddenObjectives {
        identities: HashMap::from([((DEFENDER, LANE), was_fake)]),
    }
}

fn world_with_objective(hp: u32, destroyed: bool, was_fake: bool) -> World {
    let mut world = World::new();
    world.insert_resource(session_config());
    world.insert_resource(hidden_objectives(was_fake));
    world.insert_resource(PendingObjectiveEvents::default());
    world.spawn((
        ObjectiveSlot {
            lane: LANE,
            player: DEFENDER,
            destroyed,
        },
        ObjectiveHp { hp },
    ));
    world
}

fn objective_state(world: &mut World) -> (u32, bool) {
    let mut query = world.query::<(&ObjectiveSlot, &ObjectiveHp)>();
    query
        .iter(world)
        .find_map(|(slot, hp)| {
            (slot.player == DEFENDER && slot.lane == LANE).then_some((hp.hp, slot.destroyed))
        })
        .expect("test objective should exist")
}

fn queued_events(world: &World) -> Vec<ObjectiveDestroyed> {
    world.resource::<PendingObjectiveEvents>().queue.clone()
}

#[test]
fn test_os3_non_lethal_damage_reduces_hp_without_event() {
    let mut world = world_with_objective(3, false, false);

    take_damage(&mut world, LANE, ATTACKER, 2);

    assert_eq!(objective_state(&mut world), (1, false));
    assert!(queued_events(&world).is_empty());
}

#[test]
fn test_os4_lethal_damage_saturates_to_zero_and_queues_once() {
    let mut world = world_with_objective(2, false, false);

    take_damage(&mut world, LANE, ATTACKER, 5);

    assert_eq!(objective_state(&mut world), (0, true));
    assert_eq!(
        queued_events(&world),
        vec![ObjectiveDestroyed {
            target_player_id: DEFENDER,
            lane: LANE,
            was_fake: false,
        }]
    );
}

#[test]
fn test_os5_destroyed_objective_is_noop() {
    let mut world = world_with_objective(0, true, false);

    take_damage(&mut world, LANE, ATTACKER, 3);

    assert_eq!(objective_state(&mut world), (0, true));
    assert!(queued_events(&world).is_empty());
}

#[test]
fn test_os6_take_damage_amount_is_unsigned() {
    fn assert_signature(_: fn(&mut World, LaneId, PlayerId, u32)) {}

    assert_signature(take_damage);
}

#[test]
fn test_os16_zero_damage_short_circuits_without_event() {
    let mut world = world_with_objective(3, false, false);

    take_damage(&mut world, LANE, ATTACKER, 0);

    assert_eq!(objective_state(&mut world), (3, false));
    assert!(queued_events(&world).is_empty());
}

#[test]
fn test_os20_sequential_lethal_damage_triggers_once() {
    let mut world = world_with_objective(3, false, false);

    take_damage(&mut world, LANE, ATTACKER, 5);
    take_damage(&mut world, LANE, ATTACKER, 5);

    assert_eq!(objective_state(&mut world), (0, true));
    assert_eq!(queued_events(&world).len(), 1);
}

#[test]
fn test_os25_garde_temps_routes_objective_hp_as_standard_damage() {
    let mut world = world_with_objective(3, false, true);
    let config = shared::config::GameConfig {
        objective_hp: 5,
        ..shared::config::GameConfig::default()
    };

    take_damage(&mut world, LANE, ATTACKER, config.objective_hp);

    assert_eq!(objective_state(&mut world), (0, true));
    assert_eq!(
        queued_events(&world),
        vec![ObjectiveDestroyed {
            target_player_id: DEFENDER,
            lane: LANE,
            was_fake: true,
        }]
    );
}
