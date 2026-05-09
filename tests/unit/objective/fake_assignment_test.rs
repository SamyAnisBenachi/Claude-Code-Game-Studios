use std::collections::HashSet;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::ServerPlugins;
use server::core::rsm::{DraftStarted, ResolutionComplete, ResolutionPhaseEntered};
use server::core::session::{SessionConfig, TeamId};
use server::feature::objective::system::{assign_fake_objectives, validate_objective_config};
use server::feature::objective::{
    HiddenObjectives, ObjectiveCounters, ObjectiveHp, ObjectivePlugin, ObjectiveSlot,
    OBJECTIVE_LANE_COUNT,
};
use server::foundation::config::GameConfig;
use server::foundation::rng::{RngEvent, ServerRng};
use shared::card::ClassId;
use shared::protocol::{DraftPhase, GameMode};
use shared::session::PlayerId;

fn session_config(players: &[PlayerId]) -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: players.len() as u8,
        team_map: players
            .iter()
            .enumerate()
            .map(|(index, player)| (*player, (index + 1) as TeamId))
            .collect(),
        class_map: players
            .iter()
            .map(|player| (*player, ClassId::Iop))
            .collect(),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn config(fake_count: u32, objective_hp: u32) -> shared::config::GameConfig {
    shared::config::GameConfig {
        fake_count,
        objective_hp,
        ..shared::config::GameConfig::default()
    }
}

fn app_with_objectives(config: shared::config::GameConfig, seed: u64) -> App {
    let mut app = App::new();
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / 60.0),
    })
    .add_message::<DraftStarted>()
    .add_message::<ResolutionPhaseEntered>()
    .add_message::<ResolutionComplete>()
    .add_plugins(ObjectivePlugin)
    .insert_resource(session_config(&[PlayerId(2), PlayerId(1)]))
    .insert_resource(GameConfig(config))
    .insert_resource(ServerRng::from_seed(seed));

    app.world_mut()
        .resource_mut::<Messages<DraftStarted>>()
        .write(DraftStarted {
            round: 1,
            phase: DraftPhase::Initial,
        });
    app.update();
    app
}

fn fake_lanes_for(hidden: &HiddenObjectives, player: PlayerId) -> Vec<u8> {
    let mut lanes = hidden
        .identities
        .iter()
        .filter_map(|((owner, lane), is_fake)| {
            if *owner == player && *is_fake {
                Some(*lane)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    lanes.sort_unstable();
    lanes
}

fn identity_counts(hidden: &HiddenObjectives, player: PlayerId) -> (usize, usize, usize) {
    let values = hidden
        .identities
        .iter()
        .filter_map(|((owner, _), is_fake)| (*owner == player).then_some(*is_fake))
        .collect::<Vec<_>>();
    let fake_count = values.iter().filter(|is_fake| **is_fake).count();
    let real_count = values.iter().filter(|is_fake| !**is_fake).count();
    (values.len(), fake_count, real_count)
}

#[test]
fn fake_lanes_are_distinct_for_supported_fake_counts() {
    for fake_count in [1, 2, 3] {
        for seed in 0..100 {
            let mut rng = ServerRng::from_seed(seed);
            let mut hidden = HiddenObjectives::default();

            assign_fake_objectives(&mut rng, &[PlayerId(1)], fake_count, &mut hidden);

            let fake_lanes = fake_lanes_for(&hidden, PlayerId(1));
            let distinct = fake_lanes.iter().copied().collect::<HashSet<_>>();
            assert_eq!(fake_lanes.len(), fake_count);
            assert_eq!(distinct.len(), fake_count);
        }
    }
}

#[test]
fn fake_assignment_records_all_five_lanes_per_player() {
    let mut rng = ServerRng::from_seed(42);
    let mut hidden = HiddenObjectives::default();

    assign_fake_objectives(&mut rng, &[PlayerId(2), PlayerId(1)], 2, &mut hidden);

    for player in [PlayerId(1), PlayerId(2)] {
        let (total, fake_count, real_count) = identity_counts(&hidden, player);
        assert_eq!(total, usize::from(OBJECTIVE_LANE_COUNT));
        assert_eq!(fake_count, 2);
        assert_eq!(real_count, 3);
    }

    let log = rng.audit_log();
    assert_eq!(log.len(), 5);
    assert_eq!(
        log[1].event_type,
        RngEvent::AssignFakeObjectives { player_id: 1 }
    );
    assert_eq!(
        log[2].event_type,
        RngEvent::AssignFakeObjectives { player_id: 1 }
    );
    assert_eq!(
        log[3].event_type,
        RngEvent::AssignFakeObjectives { player_id: 2 }
    );
    assert_eq!(
        log[4].event_type,
        RngEvent::AssignFakeObjectives { player_id: 2 }
    );
}

#[test]
fn invalid_objective_config_values_are_refused() {
    let fake_count_too_high = validate_objective_config(&config(4, 5))
        .expect_err("fake_count above lane_count - loss_threshold should fail");
    assert!(fake_count_too_high.contains("fake_count"));
    assert!(fake_count_too_high.contains("lane_count - loss_threshold"));

    let fake_count_zero =
        validate_objective_config(&config(0, 5)).expect_err("fake_count zero should fail");
    assert!(fake_count_zero.contains("fake_count"));
    assert!(fake_count_zero.contains(">= 1"));

    let objective_hp_zero =
        validate_objective_config(&config(2, 0)).expect_err("objective_hp zero should fail");
    assert!(objective_hp_zero.contains("objective_hp"));
    assert!(objective_hp_zero.contains(">= 1"));
}

#[test]
fn invalid_config_does_not_populate_objective_state_on_draft_initial() {
    let mut app = app_with_objectives(config(4, 5), 7);

    let hidden = app.world().resource::<HiddenObjectives>();
    assert!(hidden.identities.is_empty());

    let mut query = app.world_mut().query::<&ObjectiveSlot>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn fake_count_one_initializes_one_fake_and_zeroed_counters() {
    let mut app = app_with_objectives(config(1, 5), 9);

    let hidden = app.world().resource::<HiddenObjectives>();
    for player in [PlayerId(1), PlayerId(2)] {
        let (total, fake_count, real_count) = identity_counts(hidden, player);
        assert_eq!(total, usize::from(OBJECTIVE_LANE_COUNT));
        assert_eq!(fake_count, 1);
        assert_eq!(real_count, 4);
    }

    let counters = app.world().resource::<ObjectiveCounters>();
    for player in [PlayerId(1), PlayerId(2)] {
        assert_eq!(counters.real_objectives_destroyed(player), 0);
        assert_eq!(counters.fake_objectives_destroyed(player), 0);
    }

    let mut query = app.world_mut().query::<(&ObjectiveSlot, &ObjectiveHp)>();
    let rows = query.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(rows.len(), 10);
    assert!(rows.iter().all(|(_, hp)| hp.hp == 5));
}
