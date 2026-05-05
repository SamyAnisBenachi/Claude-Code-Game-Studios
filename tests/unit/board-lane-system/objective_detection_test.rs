use std::collections::HashMap;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitOwner, UnitStats};
use server::core::session::SessionConfig;
use server::feature::board::{
    detect_objective_presence, is_at_objective, BoardConfig, BoardPlugin, UnitAtObjective,
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
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn world_with_objective_detection() -> World {
    let mut world = World::new();
    world.insert_resource(BoardConfig::default());
    world.insert_resource(session_config());
    world.insert_resource(Messages::<UnitAtObjective>::default());
    world
}

fn spawn_unit(world: &mut World, owner: PlayerId, lane: u8, cell: u8) -> Entity {
    world
        .spawn((
            BoardPosition { lane, cell },
            UnitOwner(owner),
            UnitStats::new(1, 1, 0, 0),
        ))
        .id()
}

fn run_objective_detection(world: &mut World) {
    world
        .run_system_once(detect_objective_presence)
        .expect("objective detection system should run");
}

fn objective_hits(world: &World) -> Vec<UnitAtObjective> {
    let messages = world.resource::<Messages<UnitAtObjective>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).copied().collect()
}

#[test]
fn plugin_registers_unit_at_objective_message() {
    let mut app = App::new();
    app.add_plugins(BoardPlugin);
    app.finish();
    app.cleanup();

    assert!(app.world().contains_resource::<Messages<UnitAtObjective>>());
}

#[test]
fn test_bl_10_player_a_cell_8_emits_once_and_ignores_non_objectives() {
    let mut world = world_with_objective_detection();
    let objective_unit = spawn_unit(&mut world, player(1), 1, 8);
    spawn_unit(&mut world, player(1), 2, 7);
    spawn_unit(&mut world, player(2), 3, 8);

    run_objective_detection(&mut world);

    assert_eq!(
        objective_hits(&world),
        vec![UnitAtObjective {
            unit_id: objective_unit,
            lane: 1,
        }]
    );
}

#[test]
fn test_bl_11_surviving_unit_at_objective_emits_again_next_round() {
    let mut world = world_with_objective_detection();
    let unit = spawn_unit(&mut world, player(1), 1, 8);

    run_objective_detection(&mut world);
    run_objective_detection(&mut world);

    let position = world
        .get::<BoardPosition>(unit)
        .expect("unit should still have a board position");
    assert_eq!(position.cell, 8);
    assert_eq!(
        objective_hits(&world),
        vec![
            UnitAtObjective {
                unit_id: unit,
                lane: 1,
            },
            UnitAtObjective {
                unit_id: unit,
                lane: 1,
            },
        ]
    );
}

#[test]
fn test_bl_25_player_b_cell_1_emits_once_and_ignores_player_a_cell_1() {
    let mut world = world_with_objective_detection();
    let objective_unit = spawn_unit(&mut world, player(2), 3, 1);
    spawn_unit(&mut world, player(1), 2, 1);
    spawn_unit(&mut world, player(2), 4, 2);

    run_objective_detection(&mut world);

    assert_eq!(
        objective_hits(&world),
        vec![UnitAtObjective {
            unit_id: objective_unit,
            lane: 3,
        }]
    );
}

#[test]
fn is_at_objective_uses_configured_objective_cells() {
    let session = session_config();
    let board = BoardConfig {
        player_a_objective_cell: 6,
        player_b_objective_cell: 2,
        ..BoardConfig::default()
    };

    assert!(is_at_objective(player(1), 6, &session, &board));
    assert!(is_at_objective(player(2), 2, &session, &board));
    assert!(!is_at_objective(player(1), 8, &session, &board));
    assert!(!is_at_objective(player(2), 1, &session, &board));
}
