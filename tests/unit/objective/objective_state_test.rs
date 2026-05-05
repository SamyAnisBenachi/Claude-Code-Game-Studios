use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::ServerPlugins;
use lightyear::prelude::Replicate;
use server::core::board::UnitStats;
use server::core::rsm::DraftStarted;
use server::core::session::{SessionConfig, TeamId};
use server::feature::objective::{
    HiddenObjectives, ObjectiveCounters, ObjectiveHp, ObjectivePlugin, ObjectiveSlot,
};
use server::foundation::config::GameConfig;
use server::foundation::rng::ServerRng;
use shared::card::ClassId;
use shared::protocol::{DraftPhase, GameMode};
use shared::session::PlayerId;

fn session_config(players: [PlayerId; 2]) -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: players.len() as u8,
        team_map: HashMap::from([(players[0], 1 as TeamId), (players[1], 2 as TeamId)]),
        class_map: HashMap::from([(players[0], ClassId::Iop), (players[1], ClassId::Cra)]),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn config_with_objective_hp(objective_hp: u32) -> GameConfig {
    GameConfig(shared::config::GameConfig {
        objective_hp,
        ..shared::config::GameConfig::default()
    })
}

fn app_with_objectives(objective_hp: u32) -> App {
    let mut app = App::new();
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / 60.0),
    })
    .add_message::<DraftStarted>()
    .add_plugins(ObjectivePlugin)
    .insert_resource(session_config([PlayerId(1), PlayerId(2)]))
    .insert_resource(config_with_objective_hp(objective_hp))
    .insert_resource(ServerRng::from_seed(7));

    app.world_mut()
        .resource_mut::<Messages<DraftStarted>>()
        .write(DraftStarted {
            round: 1,
            phase: DraftPhase::Initial,
        });
    app.update();
    app
}

fn objective_rows(app: &mut App) -> Vec<(PlayerId, u8, u32, bool, bool)> {
    let mut query = app.world_mut().query::<(
        &ObjectiveSlot,
        &ObjectiveHp,
        Option<&UnitStats>,
        Option<&Replicate>,
    )>();
    let mut rows = query
        .iter(app.world())
        .map(|(slot, hp, stats, replicate)| {
            (
                slot.player,
                slot.lane,
                hp.hp,
                stats.is_some(),
                replicate.is_some(),
            )
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|(player, lane, _, _, _)| (player.0, *lane));
    rows
}

#[test]
fn plugin_registers_objective_resources() {
    let mut app = App::new();
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / 60.0),
    })
    .add_message::<DraftStarted>()
    .add_plugins(ObjectivePlugin);
    app.finish();
    app.cleanup();

    assert!(app.world().contains_resource::<HiddenObjectives>());
    assert!(app.world().contains_resource::<ObjectiveCounters>());
}

#[test]
fn test_draft_initial_creates_five_slots_per_player() {
    let mut app = app_with_objectives(5);
    let rows = objective_rows(&mut app);

    assert_eq!(rows.len(), 10);
    assert_eq!(
        rows.iter()
            .filter(|(player, _, _, _, _)| *player == PlayerId(1))
            .count(),
        5
    );
    assert_eq!(
        rows.iter()
            .filter(|(player, _, _, _, _)| *player == PlayerId(2))
            .count(),
        5
    );

    for player in [PlayerId(1), PlayerId(2)] {
        let lanes = rows
            .iter()
            .filter(|(row_player, _, _, _, _)| *row_player == player)
            .map(|(_, lane, _, _, _)| *lane)
            .collect::<Vec<_>>();
        assert_eq!(lanes, vec![1, 2, 3, 4, 5]);
    }
}

#[test]
fn test_objective_slots_use_configured_hp_and_no_armor_component() {
    for objective_hp in [3, 5, 8] {
        let mut app = app_with_objectives(objective_hp);
        let rows = objective_rows(&mut app);

        assert!(rows
            .iter()
            .all(|(_, _, hp, has_unit_stats, has_replicate)| {
                *hp == objective_hp && !has_unit_stats && *has_replicate
            }));
    }
}

#[test]
fn test_hidden_resources_exist_and_counters_start_zero() {
    let app = app_with_objectives(5);

    let hidden = app.world().resource::<HiddenObjectives>();
    assert_eq!(hidden.identities.len(), 10);

    let counters = app.world().resource::<ObjectiveCounters>();
    for player in [PlayerId(1), PlayerId(2)] {
        assert_eq!(counters.real_objectives_destroyed(player), 0);
        assert_eq!(counters.fake_objectives_destroyed(player), 0);
        assert_eq!(counters.real_destroyed.get(&player), Some(&0));
        assert_eq!(counters.fake_destroyed.get(&player), Some(&0));
    }
}
