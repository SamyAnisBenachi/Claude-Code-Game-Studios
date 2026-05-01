use std::time::Duration;

use bevy::{prelude::*, time::TimeUpdateStrategy};
use bevy_tweening::{PlaybackState, TweenAnim, TweenState};
use client::card_animations::{
    AnimGroup, AnimQueue, AnimationTimingConfig, CardAnimationsPlugin,
    PendingObjectiveDestroyedEvents,
};

fn app_with_card_animations(stagger_cadence_ms: u64) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(CardAnimationsPlugin);
    app.world_mut().insert_resource(AnimationTimingConfig {
        stagger_cadence_ms,
        ..Default::default()
    });
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    app.update();
    app
}

fn run_for(app: &mut App, duration: Duration) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(duration);
    app.update();
}

fn active_playing(app: &App, entity: Entity) -> bool {
    app.world()
        .get::<TweenAnim>(entity)
        .is_some_and(|animator| {
            animator.playback_state == PlaybackState::Playing
                && animator.tween_state() == TweenState::Active
        })
}

fn stage_destroyed_objectives(app: &mut App, lanes: &[u8]) -> Vec<Entity> {
    let entities = lanes
        .iter()
        .map(|_| app.world_mut().spawn(Transform::default()).id())
        .collect::<Vec<_>>();

    *app.world_mut().resource_mut::<AnimQueue>() =
        AnimQueue::from_groups(vec![AnimGroup::new(1, 0, Vec::new())]);

    {
        let mut pending = app
            .world_mut()
            .resource_mut::<PendingObjectiveDestroyedEvents>();
        for (&lane, &entity) in lanes.iter().zip(entities.iter()) {
            pending.push(lane, entity);
        }
    }

    run_for(app, Duration::ZERO);
    entities
}

#[test]
fn objective_reveals_stagger_by_sorted_lane_at_100_ms_cadence() {
    let mut app = app_with_card_animations(100);
    let entities = stage_destroyed_objectives(&mut app, &[3, 5]);
    let lane_3 = entities[0];
    let lane_5 = entities[1];

    assert!(active_playing(&app, lane_3));
    assert!(!active_playing(&app, lane_5));

    run_for(&mut app, Duration::from_millis(99));
    assert!(!active_playing(&app, lane_5));

    run_for(&mut app, Duration::from_millis(1));
    assert!(active_playing(&app, lane_5));
    assert!(app.world().get::<TweenAnim>(lane_3).is_some());
}

#[test]
fn objective_reveals_with_zero_cadence_start_same_frame() {
    let mut app = app_with_card_animations(0);
    let entities = stage_destroyed_objectives(&mut app, &[3, 5, 1, 4]);

    for entity in entities {
        assert!(active_playing(&app, entity));
    }
}

#[test]
fn objective_reveal_fourth_lane_starts_at_360_ms_with_120_ms_cadence() {
    let mut app = app_with_card_animations(120);
    let entities = stage_destroyed_objectives(&mut app, &[1, 2, 4, 5]);
    let fourth_lane = entities[3];

    assert!(!active_playing(&app, fourth_lane));

    run_for(&mut app, Duration::from_millis(344));
    assert!(!active_playing(&app, fourth_lane));

    run_for(&mut app, Duration::from_millis(16));
    assert!(active_playing(&app, fourth_lane));
}
