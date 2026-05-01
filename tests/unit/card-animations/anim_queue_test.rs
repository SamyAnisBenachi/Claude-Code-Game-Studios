use std::time::Duration;

use bevy::{prelude::*, time::TimeUpdateStrategy};
use bevy_tweening::{PlaybackState, TweenAnim, TweenState};
use client::card_animations::{
    AnimGroup, AnimQueue, AnimQueueEvent, CardAnimationsPlugin, GroupDrainedSignal,
    PendingObjectiveDestroyedEvents, PendingPhaseChange,
};
use shared::protocol::RoundPhase;

fn app_with_card_animations() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(CardAnimationsPlugin);
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

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn two_group_queue(target: Entity) -> AnimQueue {
    AnimQueue::from_groups(vec![
        AnimGroup::new(1, 600, Vec::new()),
        AnimGroup::new(
            2,
            600,
            vec![AnimQueueEvent::transform_tween(
                target,
                Vec3::ZERO,
                Vec3::new(20.0, 0.0, 0.0),
                600,
            )],
        ),
    ])
}

fn active_playing(animator: &TweenAnim) -> bool {
    animator.playback_state == PlaybackState::Playing
        && animator.tween_state() == TweenState::Active
}

fn tween_anim_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query::<&TweenAnim>();
    query.iter(world).count()
}

#[test]
fn game_over_skip_holds_current_index_and_skips_next_group_tween() {
    let mut app = app_with_card_animations();
    let entity_1 = app.world_mut().spawn(Transform::default()).id();

    *app.world_mut().resource_mut::<AnimQueue>() = two_group_queue(entity_1);
    app.world_mut()
        .resource_mut::<PendingPhaseChange>()
        .set_phase(RoundPhase::GameOver);

    run_for(&mut app, Duration::from_millis(600));

    assert_eq!(app.world().resource::<AnimQueue>().current_index, 0);
    assert!(app.world().get::<TweenAnim>(entity_1).is_none());
    assert_eq!(read_messages::<GroupDrainedSignal>(&app).len(), 1);
}

#[test]
fn game_over_skip_runs_objective_reveal_for_pending_destroyed_objective() {
    let mut app = app_with_card_animations();
    let entity_1 = app.world_mut().spawn(Transform::default()).id();
    let objective_entity = app.world_mut().spawn(Transform::default()).id();

    *app.world_mut().resource_mut::<AnimQueue>() = two_group_queue(entity_1);
    app.world_mut()
        .resource_mut::<PendingPhaseChange>()
        .set_phase(RoundPhase::GameOver);
    app.world_mut()
        .resource_mut::<PendingObjectiveDestroyedEvents>()
        .push(2, objective_entity);

    run_for(&mut app, Duration::from_millis(600));

    let animator = app
        .world()
        .get::<TweenAnim>(objective_entity)
        .expect("objective reveal tween should be inserted");
    assert!(active_playing(animator));
    assert!(app.world().get::<TweenAnim>(entity_1).is_none());
}

#[test]
fn game_over_skip_fires_at_exact_group_duration_boundary() {
    let mut app = app_with_card_animations();
    let entity_1 = app.world_mut().spawn(Transform::default()).id();
    let objective_entity = app.world_mut().spawn(Transform::default()).id();

    *app.world_mut().resource_mut::<AnimQueue>() = two_group_queue(entity_1);
    app.world_mut()
        .resource_mut::<PendingPhaseChange>()
        .set_phase(RoundPhase::GameOver);
    app.world_mut()
        .resource_mut::<PendingObjectiveDestroyedEvents>()
        .push(1, objective_entity);

    run_for(&mut app, Duration::from_millis(599));

    assert_eq!(app.world().resource::<AnimQueue>().current_index, 0);
    assert!(app.world().get::<TweenAnim>(objective_entity).is_none());
    assert!(app.world().get::<TweenAnim>(entity_1).is_none());

    run_for(&mut app, Duration::from_millis(1));

    assert_eq!(app.world().resource::<AnimQueue>().current_index, 0);
    assert!(active_playing(
        app.world().get::<TweenAnim>(objective_entity).unwrap()
    ));
    assert!(app.world().get::<TweenAnim>(entity_1).is_none());
}

#[test]
fn empty_anim_queue_drains_pending_phase_after_pre_animation_pause() {
    let mut app = app_with_card_animations();
    let initial_animators = tween_anim_count(&mut app);

    app.world_mut()
        .resource_mut::<PendingPhaseChange>()
        .set_phase(RoundPhase::DraftShop);

    run_for(&mut app, Duration::from_millis(399));
    assert!(!app.world().resource::<PendingPhaseChange>().is_none());
    assert_eq!(tween_anim_count(&mut app), initial_animators);

    run_for(&mut app, Duration::from_millis(1));
    assert!(app.world().resource::<PendingPhaseChange>().is_none());
    assert_eq!(tween_anim_count(&mut app), initial_animators);
}
