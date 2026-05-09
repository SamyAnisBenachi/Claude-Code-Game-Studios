use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::ServerPlugins;
use lightyear::prelude::Replicate;
use server::core::economy::PlayerEconomies;
use server::core::rsm::{DraftStarted, GameOverEmitted};
use server::core::session::SessionConfig;
use server::feature::prism::{
    AuditLog, DiscardLog, PrismCollected, PrismLaneKey, PrismPlugin, PrismPresence, PrismState,
    PRISM_LANE_COUNT,
};
use shared::card::ClassId;
use shared::protocol::{DraftPhase, GameMode, GameOverReason};
use shared::session::PlayerId;

fn session_config() -> SessionConfig {
    let player_a = PlayerId(1);
    let player_b = PlayerId(2);

    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(player_a, 0), (player_b, 1)]),
        class_map: HashMap::from([(player_a, ClassId::Iop), (player_b, ClassId::Cra)]),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn add_prism_resources(app: &mut App) {
    app.world_mut().insert_resource(PrismState::default());
    app.world_mut().insert_resource(DiscardLog::default());
    app.world_mut().insert_resource(AuditLog::default());
}

fn app_with_prism() -> App {
    let mut app = App::new();
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / 60.0),
    })
    .add_plugins(PrismPlugin)
    .add_message::<GameOverEmitted>();
    app
}

#[test]
fn default_prism_state_starts_uncollected_with_empty_logs() {
    let state = PrismState::default();
    let discard_log = DiscardLog::default();
    let audit_log = AuditLog::default();

    assert!(state
        .collected
        .iter()
        .flatten()
        .all(|collected| !*collected));
    assert!(state.pending_respawn.iter().all(|pending| !*pending));
    assert!(discard_log.entries.is_empty());
    assert!(audit_log.entries.is_empty());
}

#[test]
fn plugin_spawns_public_prism_presence_on_initial_draft_entry() {
    let mut app = app_with_prism();
    app.world_mut().insert_resource(session_config());

    app.world_mut()
        .resource_mut::<Messages<DraftStarted>>()
        .write(DraftStarted {
            round: 1,
            phase: DraftPhase::Initial,
        });

    app.update();

    assert!(app.world().contains_resource::<PrismState>());
    assert!(app.world().contains_resource::<DiscardLog>());
    assert!(app.world().contains_resource::<AuditLog>());

    let mut query = app
        .world_mut()
        .query::<(&PrismLaneKey, &PrismPresence, &Replicate)>();
    let entries = query
        .iter(app.world())
        .map(|(key, presence, _)| (key.player, key.lane, presence.collected))
        .collect::<Vec<_>>();

    assert_eq!(entries.len(), 2 * PRISM_LANE_COUNT);
    for player in [PlayerId(1), PlayerId(2)] {
        for lane in 1..=PRISM_LANE_COUNT as u8 {
            assert!(entries.contains(&(player, lane, false)));
        }
    }
}

#[test]
fn resolve_prism_draws_with_empty_buffer_leaves_uncollected_state_unchanged() {
    let mut app = app_with_prism();
    add_prism_resources(&mut app);

    app.update();

    let state = app.world().resource::<PrismState>();
    assert!(state
        .collected
        .iter()
        .flatten()
        .all(|collected| !*collected));
    assert!(app.world().resource::<DiscardLog>().entries.is_empty());
    assert!(app.world().resource::<AuditLog>().entries.is_empty());
}

#[test]
fn collected_lane_state_persists_without_new_collection_message() {
    let mut app = app_with_prism();
    add_prism_resources(&mut app);
    app.world_mut().resource_mut::<PrismState>().collected[0][2] = true;

    app.update();

    let state = app.world().resource::<PrismState>();
    assert!(state.collected[0][2]);
    assert!(!state.collected[0][0]);
    assert!(app.world().resource::<DiscardLog>().entries.is_empty());
    assert!(app.world().resource::<AuditLog>().entries.is_empty());
}

#[test]
fn prism_collection_scaffold_does_not_require_or_mutate_economy() {
    let player = PlayerId(1);
    let mut app = app_with_prism();
    add_prism_resources(&mut app);
    app.world_mut().insert_resource(PlayerEconomies::default());

    app.world_mut()
        .resource_mut::<Messages<PrismCollected>>()
        .write(PrismCollected {
            player_id: player,
            lane: 1,
        });

    app.update();

    assert!(app.world().resource::<PlayerEconomies>().0.is_empty());
    assert!(app.world().resource::<DiscardLog>().entries.is_empty());
    assert!(app.world().resource::<AuditLog>().entries.is_empty());
}

#[test]
fn game_over_cleans_prism_resources_and_presence_entities() {
    let mut app = app_with_prism();
    app.world_mut().insert_resource(session_config());

    app.world_mut()
        .resource_mut::<Messages<DraftStarted>>()
        .write(DraftStarted {
            round: 1,
            phase: DraftPhase::Initial,
        });
    app.update();

    app.world_mut()
        .resource_mut::<Messages<GameOverEmitted>>()
        .write(GameOverEmitted {
            reason: GameOverReason::Draw,
            loser: None,
            round: 1,
        });
    app.update();

    assert!(!app.world().contains_resource::<PrismState>());
    assert!(!app.world().contains_resource::<DiscardLog>());
    assert!(!app.world().contains_resource::<AuditLog>());

    let mut query = app.world_mut().query::<&PrismPresence>();
    assert_eq!(query.iter(app.world()).count(), 0);
}
