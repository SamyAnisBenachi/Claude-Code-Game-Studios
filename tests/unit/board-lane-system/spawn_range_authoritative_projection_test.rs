use std::collections::HashMap;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::{Messages, World};
use server::core::objective_contract::ObjectiveCounters;
use server::core::session::{build_snapshot, SessionConfig};
use server::feature::board::{
    expand_spawn_range, expand_spawn_range_from_objective_fact, spawn_range_cells_for_player,
    update_spawn_range, FakeObjectiveDestroyed, SpawnRangeState,
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
    }
}

#[test]
fn test_expand_spawn_range_clamps_at_three_cells() {
    let session = session_config();
    let mut spawn_ranges = SpawnRangeState::default();

    assert_eq!(
        spawn_range_cells_for_player(&spawn_ranges, PLAYER_A, &session),
        Some(1)
    );
    assert_eq!(
        expand_spawn_range(&mut spawn_ranges, PLAYER_A, &session)
            .map(|change| change.new_spawn_range_cells),
        Some(2)
    );
    assert_eq!(
        expand_spawn_range(&mut spawn_ranges, PLAYER_A, &session)
            .map(|change| change.new_spawn_range_cells),
        Some(3)
    );
    assert_eq!(
        expand_spawn_range(&mut spawn_ranges, PLAYER_A, &session)
            .map(|change| change.new_spawn_range_cells),
        None
    );

    assert_eq!(spawn_ranges.fakes_destroyed[0], 2);
    assert_eq!(
        spawn_range_cells_for_player(&spawn_ranges, PLAYER_A, &session),
        Some(3)
    );
}

#[test]
fn test_snapshot_spawn_range_uses_spawn_range_state_not_objective_counters() {
    let mut world = World::new();
    world.insert_resource(session_config());
    world.insert_resource(SpawnRangeState {
        fakes_destroyed: [0, 0],
        ..Default::default()
    });
    world.insert_resource(ObjectiveCounters {
        real_destroyed: HashMap::new(),
        fake_destroyed: HashMap::from([(PLAYER_A, 2)]),
    });

    let snapshot = build_snapshot(PLAYER_A, &mut world).expect("snapshot should build");
    let player_a = snapshot
        .players
        .iter()
        .find(|player| player.player_id == PLAYER_A)
        .expect("player A snapshot should exist");

    assert_eq!(player_a.spawn_range_cells, 1);
}

#[test]
fn test_scheduled_spawn_range_bridge_skips_fact_already_applied_to_projection() {
    let session = session_config();
    let mut spawn_ranges = SpawnRangeState::default();
    assert_eq!(
        expand_spawn_range_from_objective_fact(&mut spawn_ranges, PLAYER_A, &session)
            .map(|change| change.new_spawn_range_cells),
        Some(2)
    );

    let mut world = World::new();
    world.insert_resource(session);
    world.insert_resource(spawn_ranges);
    world.insert_resource(Messages::<FakeObjectiveDestroyed>::default());
    world
        .resource_mut::<Messages<FakeObjectiveDestroyed>>()
        .write(FakeObjectiveDestroyed {
            destroyed_by: PLAYER_A,
        });

    world
        .run_system_once(update_spawn_range)
        .expect("spawn range update bridge should run");

    let spawn_ranges = world.resource::<SpawnRangeState>();
    assert_eq!(spawn_ranges.fakes_destroyed[0], 1);
    assert_eq!(
        spawn_range_cells_for_player(spawn_ranges, PLAYER_A, &session_config()),
        Some(2)
    );
}
