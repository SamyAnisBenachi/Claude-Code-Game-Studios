use std::collections::HashMap;

use bevy::prelude::*;
use server::core::economy::{AwardGold, ManaCapIncreased};
use server::core::objective_contract::ObjectiveCounters;
use server::core::session::SessionConfig;
use server::feature::board::LaneId;
use server::feature::objective::{
    apply_consequence_path, take_damage, HiddenObjectives, ObjectiveDestroyed, ObjectiveHp,
    ObjectiveSlot, PendingObjectiveEvents,
};
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;

const PLAYER_A: PlayerId = PlayerId(1);
const PLAYER_B: PlayerId = PlayerId(2);
const LANE_1: LaneId = 1;
const LANE_2: LaneId = 2;
const LANE_3: LaneId = 3;

fn session_config() -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(PLAYER_A, 0), (PLAYER_B, 1)]),
        class_map: HashMap::from([(PLAYER_A, ClassId::Iop), (PLAYER_B, ClassId::Cra)]),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn app_with_objective(owner: PlayerId, lane: LaneId, was_fake: bool) -> App {
    let mut app = App::new();
    app.add_message::<AwardGold>();
    app.add_message::<ManaCapIncreased>();
    app.insert_resource(session_config());
    app.insert_resource(HiddenObjectives {
        identities: HashMap::from([((owner, lane), was_fake)]),
    });
    app.insert_resource(ObjectiveCounters::default());
    app.insert_resource(PendingObjectiveEvents::default());
    spawn_objective(&mut app, owner, lane, 1, false);
    app
}

fn spawn_objective(app: &mut App, owner: PlayerId, lane: LaneId, hp: u32, destroyed: bool) {
    app.world_mut().spawn((
        ObjectiveSlot {
            lane,
            player: owner,
            destroyed,
        },
        ObjectiveHp { hp },
    ));
}

fn set_identity(app: &mut App, owner: PlayerId, lane: LaneId, was_fake: bool) {
    app.world_mut()
        .resource_mut::<HiddenObjectives>()
        .identities
        .insert((owner, lane), was_fake);
}

fn queued_events(app: &App) -> Vec<ObjectiveDestroyed> {
    app.world()
        .resource::<PendingObjectiveEvents>()
        .queue
        .clone()
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn objective_destroyed(app: &mut App, owner: PlayerId, lane: LaneId) -> bool {
    let world = app.world_mut();
    let mut query = world.query::<&ObjectiveSlot>();
    query
        .iter(world)
        .find_map(|slot| (slot.player == owner && slot.lane == lane).then_some(slot.destroyed))
        .expect("test objective should exist")
}

#[test]
fn test_os7_opponent_destruction_emits_one_award_gold_for_real_and_fake() {
    for was_fake in [false, true] {
        let mut app = app_with_objective(PLAYER_B, LANE_1, was_fake);

        apply_consequence_path(app.world_mut(), LANE_1, PLAYER_A, PLAYER_B);

        assert_eq!(
            read_messages::<AwardGold>(&app),
            vec![AwardGold {
                player: PLAYER_A,
                amount: 3,
            }]
        );
    }
}

#[test]
fn test_os9_second_real_objective_increments_real_destroyed_count_to_two() {
    let mut app = app_with_objective(PLAYER_B, LANE_1, false);
    app.world_mut()
        .resource_mut::<ObjectiveCounters>()
        .real_destroyed
        .insert(PLAYER_B, 1);

    apply_consequence_path(app.world_mut(), LANE_1, PLAYER_A, PLAYER_B);

    assert_eq!(
        app.world()
            .resource::<ObjectiveCounters>()
            .real_objectives_destroyed(PLAYER_B),
        2
    );
}

#[test]
fn test_os10_both_players_real_counts_reach_two_in_same_sequence() {
    let mut app = app_with_objective(PLAYER_B, LANE_1, false);
    spawn_objective(&mut app, PLAYER_A, LANE_3, 1, false);
    set_identity(&mut app, PLAYER_A, LANE_3, false);
    {
        let mut counters = app.world_mut().resource_mut::<ObjectiveCounters>();
        counters.real_destroyed.insert(PLAYER_A, 1);
        counters.real_destroyed.insert(PLAYER_B, 1);
    }

    apply_consequence_path(app.world_mut(), LANE_1, PLAYER_A, PLAYER_B);
    apply_consequence_path(app.world_mut(), LANE_3, PLAYER_B, PLAYER_A);

    let counters = app.world().resource::<ObjectiveCounters>();
    assert_eq!(counters.real_objectives_destroyed(PLAYER_A), 2);
    assert_eq!(counters.real_objectives_destroyed(PLAYER_B), 2);
}

#[test]
fn test_os13a_objective_destroyed_is_queued_with_payload() {
    let mut app = app_with_objective(PLAYER_B, LANE_3, false);

    apply_consequence_path(app.world_mut(), LANE_3, PLAYER_A, PLAYER_B);

    assert_eq!(
        queued_events(&app),
        vec![ObjectiveDestroyed {
            target_player_id: PLAYER_B,
            lane: LANE_3,
            was_fake: false,
        }]
    );
}

#[test]
fn test_os14_self_destruction_real_objective_skips_rewards_and_increments_real_count() {
    let mut app = app_with_objective(PLAYER_A, LANE_1, false);

    apply_consequence_path(app.world_mut(), LANE_1, PLAYER_A, PLAYER_A);

    assert!(read_messages::<AwardGold>(&app).is_empty());
    assert!(read_messages::<ManaCapIncreased>(&app).is_empty());
    assert_eq!(
        app.world()
            .resource::<ObjectiveCounters>()
            .real_objectives_destroyed(PLAYER_A),
        1
    );
}

#[test]
fn test_os18a_take_damage_preserves_lane_ascending_consequence_order() {
    let mut app = app_with_objective(PLAYER_B, LANE_1, false);
    spawn_objective(&mut app, PLAYER_B, LANE_3, 1, false);
    set_identity(&mut app, PLAYER_B, LANE_3, false);

    take_damage(app.world_mut(), LANE_1, PLAYER_A, 1);
    take_damage(app.world_mut(), LANE_3, PLAYER_A, 1);

    let events = queued_events(&app);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].lane, LANE_1);
    assert_eq!(events[1].lane, LANE_3);
}

#[test]
fn test_os21_self_destruction_fake_marks_destroyed_without_rewards_or_counter() {
    let mut app = app_with_objective(PLAYER_A, LANE_2, true);
    app.world_mut()
        .resource_mut::<ObjectiveCounters>()
        .fake_destroyed
        .insert(PLAYER_A, 4);

    apply_consequence_path(app.world_mut(), LANE_2, PLAYER_A, PLAYER_A);

    assert!(objective_destroyed(&mut app, PLAYER_A, LANE_2));
    assert!(read_messages::<AwardGold>(&app).is_empty());
    assert!(read_messages::<ManaCapIncreased>(&app).is_empty());

    let counters = app.world().resource::<ObjectiveCounters>();
    assert_eq!(counters.fake_objectives_destroyed(PLAYER_A), 4);
    assert_eq!(counters.real_objectives_destroyed(PLAYER_A), 0);
    assert_eq!(
        queued_events(&app),
        vec![ObjectiveDestroyed {
            target_player_id: PLAYER_A,
            lane: LANE_2,
            was_fake: true,
        }]
    );
}
