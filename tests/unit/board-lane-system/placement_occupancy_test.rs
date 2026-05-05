use std::collections::HashMap;

use bevy::prelude::{Entity, World};
use server::core::economy::state::PlayerEconomy;
use server::core::session::SessionConfig;
use server::feature::board::{
    is_field_slot_available, is_minion_slot_available, is_structure_slot_available,
    is_trap_slot_available, BoardOccupancy, LaneId,
};
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn lane(id: u8) -> LaneId {
    id
}

fn entity(world: &mut World) -> Entity {
    world.spawn_empty().id()
}

fn session_config(mode: GameMode, teams: &[(PlayerId, u8)]) -> SessionConfig {
    let team_map = teams.iter().copied().collect::<HashMap<_, _>>();
    let class_map = teams
        .iter()
        .map(|(player, _)| (*player, ClassId::Iop))
        .collect::<HashMap<_, _>>();

    SessionConfig {
        mode,
        player_count: teams.len() as u8,
        team_map,
        class_map,
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn one_v_one_session() -> SessionConfig {
    session_config(GameMode::OneVOne, &[(player(1), 0), (player(2), 1)])
}

fn two_v_two_session() -> SessionConfig {
    session_config(
        GameMode::TwoVTwo,
        &[
            (player(1), 0),
            (player(2), 0),
            (player(3), 1),
            (player(4), 1),
        ],
    )
}

fn economy_with_gold(gold: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold,
        current_mana: 0,
        reserve_mana: 0,
        mana_cap: 0,
        reserved_gold: 0,
    }
}

#[test]
fn bl_8_duplicate_minion_in_same_lane_is_rejected() {
    let mut world = World::new();
    let mut occupancy = BoardOccupancy::default();
    occupancy
        .minion_slots
        .insert((player(1), lane(3)), entity(&mut world));

    assert!(!is_minion_slot_available(
        &occupancy,
        player(1),
        lane(3),
        &one_v_one_session()
    ));
}

#[test]
fn bl_8_minion_occupancy_is_independent_per_player_and_lane() {
    let mut world = World::new();
    let mut occupancy = BoardOccupancy::default();
    occupancy
        .minion_slots
        .insert((player(1), lane(3)), entity(&mut world));

    assert!(is_minion_slot_available(
        &occupancy,
        player(2),
        lane(3),
        &one_v_one_session()
    ));
    assert!(is_minion_slot_available(
        &occupancy,
        player(1),
        lane(4),
        &one_v_one_session()
    ));
}

#[test]
fn bl_9_duplicate_trap_same_cell_is_rejected_without_gold_mutation() {
    let mut world = World::new();
    let mut occupancy = BoardOccupancy::default();
    let economy = economy_with_gold(10);
    occupancy
        .traps
        .insert((player(1), lane(2), 3), entity(&mut world));

    assert!(!is_trap_slot_available(&occupancy, player(1), lane(2), 3));
    assert_eq!(economy.gold, 10);
}

#[test]
fn bl_9_trap_occupancy_is_independent_per_player_and_cell() {
    let mut world = World::new();
    let mut occupancy = BoardOccupancy::default();
    occupancy
        .traps
        .insert((player(1), lane(2), 3), entity(&mut world));

    assert!(is_trap_slot_available(&occupancy, player(1), lane(2), 4));
    assert!(is_trap_slot_available(&occupancy, player(2), lane(2), 3));
}

#[test]
fn duplicate_structure_same_cell_is_rejected() {
    let mut world = World::new();
    let mut occupancy = BoardOccupancy::default();
    occupancy
        .structures
        .insert((player(1), lane(4), 6), entity(&mut world));

    assert!(!is_structure_slot_available(
        &occupancy,
        player(1),
        lane(4),
        6
    ));
    assert!(is_structure_slot_available(
        &occupancy,
        player(2),
        lane(4),
        6
    ));
}

#[test]
fn bl_29_duplicate_field_in_same_lane_is_rejected() {
    let mut world = World::new();
    let mut occupancy = BoardOccupancy::default();
    occupancy
        .fields
        .insert((player(1), lane(2)), entity(&mut world));

    assert!(!is_field_slot_available(&occupancy, player(1), lane(2)));
    assert!(is_field_slot_available(&occupancy, player(1), lane(3)));
}

#[test]
fn bl_32_fields_are_independent_per_player_in_same_lane() {
    let mut world = World::new();
    let mut occupancy = BoardOccupancy::default();
    occupancy
        .fields
        .insert((player(1), lane(2)), entity(&mut world));

    assert!(is_field_slot_available(&occupancy, player(2), lane(2)));

    occupancy
        .fields
        .insert((player(2), lane(2)), entity(&mut world));
    let lane_two_field_count = occupancy
        .fields
        .keys()
        .filter(|(_, field_lane)| *field_lane == lane(2))
        .count();

    assert_eq!(lane_two_field_count, 2);
}

#[test]
fn bl_33_two_v_two_teammate_can_use_second_team_minion_slot() {
    let mut world = World::new();
    let mut occupancy = BoardOccupancy::default();
    occupancy
        .minion_slots
        .insert((player(1), lane(1)), entity(&mut world));

    assert!(is_minion_slot_available(
        &occupancy,
        player(2),
        lane(1),
        &two_v_two_session()
    ));
}

#[test]
fn bl_33_two_v_two_player_cannot_use_personal_slot_twice() {
    let mut world = World::new();
    let mut occupancy = BoardOccupancy::default();
    occupancy
        .minion_slots
        .insert((player(1), lane(1)), entity(&mut world));
    occupancy
        .minion_slots
        .insert((player(2), lane(1)), entity(&mut world));
    let economy = economy_with_gold(10);

    assert!(!is_minion_slot_available(
        &occupancy,
        player(2),
        lane(1),
        &two_v_two_session()
    ));
    assert_eq!(economy.gold, 10);
}

#[test]
fn unknown_player_minion_slot_is_rejected_silently() {
    assert!(!is_minion_slot_available(
        &BoardOccupancy::default(),
        player(99),
        lane(1),
        &one_v_one_session()
    ));
}
