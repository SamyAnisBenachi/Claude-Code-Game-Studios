use std::time::Duration;

use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::{AnimTarget, PlaybackState, Tween, TweenAnim, TweenState};
use client::card_animations::{
    make_tween_anim, replace_tweenable, BoardRebuildRequested, CardAnimationsPlugin,
    PlacementPhaseAnimator, SpriteAlphaLens,
};

#[derive(Component)]
struct EntityIdentity;

fn app_with_card_animations() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(CardAnimationsPlugin);
    app.update();
    app
}

fn run_for(app: &mut App, duration: Duration) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(duration);
    app.update();
}

fn transform_tween(duration_ms: u64) -> Tween {
    Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(duration_ms),
        TransformPositionLens {
            start: Vec3::ZERO,
            end: Vec3::new(10.0, 0.0, 0.0),
        },
    )
}

fn sprite_tween(duration_ms: u64) -> Tween {
    Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(duration_ms),
        SpriteAlphaLens {
            start: 1.0,
            end: 0.0,
        },
    )
}

fn active_playing(animator: &TweenAnim) -> bool {
    animator.playback_state == PlaybackState::Playing
        && animator.tween_state() == TweenState::Active
}

#[test]
fn board_rebuild_cancels_playing_animators_and_keeps_components() {
    let mut app = app_with_card_animations();
    let transform_entity = app
        .world_mut()
        .spawn((
            Transform::default(),
            make_tween_anim(transform_tween(1_000)),
        ))
        .id();
    let sprite_entity = app
        .world_mut()
        .spawn((Sprite::default(), make_tween_anim(sprite_tween(1_000))))
        .id();

    assert!(active_playing(
        app.world().get::<TweenAnim>(transform_entity).unwrap()
    ));
    assert!(active_playing(
        app.world().get::<TweenAnim>(sprite_entity).unwrap()
    ));

    app.world_mut().write_message(BoardRebuildRequested);
    run_for(&mut app, Duration::ZERO);

    let transform_animator = app.world().get::<TweenAnim>(transform_entity).unwrap();
    let sprite_animator = app.world().get::<TweenAnim>(sprite_entity).unwrap();

    assert!(!active_playing(transform_animator));
    assert!(!active_playing(sprite_animator));
    assert_eq!(transform_animator.playback_state, PlaybackState::Paused);
    assert_eq!(sprite_animator.playback_state, PlaybackState::Paused);
}

#[test]
fn board_rebuild_cancels_placement_animator_same_frame_and_keeps_marker() {
    let mut app = app_with_card_animations();
    let entity = app
        .world_mut()
        .spawn((
            Transform::default(),
            PlacementPhaseAnimator,
            make_tween_anim(transform_tween(250)),
        ))
        .id();

    app.world_mut().write_message(BoardRebuildRequested);
    run_for(&mut app, Duration::ZERO);

    let animator = app.world().get::<TweenAnim>(entity).unwrap();
    assert!(!active_playing(animator));
    assert_eq!(animator.playback_state, PlaybackState::Paused);
    assert!(app.world().get::<PlacementPhaseAnimator>(entity).is_some());
}

#[test]
fn completed_animator_replacement_preserves_entity_id_and_restarts() {
    let mut app = app_with_card_animations();
    let entity = app
        .world_mut()
        .spawn((
            EntityIdentity,
            Transform::default(),
            make_tween_anim(transform_tween(10)),
        ))
        .id();

    run_for(&mut app, Duration::from_millis(20));
    assert_eq!(
        app.world().get::<TweenAnim>(entity).unwrap().tween_state(),
        TweenState::Completed
    );

    {
        let mut entity_mut = app.world_mut().entity_mut(entity);
        let mut animator = entity_mut.get_mut::<TweenAnim>().unwrap();
        replace_tweenable(&mut animator, transform_tween(100)).unwrap();
    }

    run_for(&mut app, Duration::from_millis(1));

    assert!(app.world().get_entity(entity).is_ok());
    assert!(app.world().get::<EntityIdentity>(entity).is_some());
    assert!(active_playing(
        app.world().get::<TweenAnim>(entity).unwrap()
    ));
}

#[test]
fn transform_and_sprite_tweens_can_run_for_same_target_entity() {
    let mut app = app_with_card_animations();
    let unit = app
        .world_mut()
        .spawn((Transform::default(), Sprite::default()))
        .id();

    app.world_mut()
        .entity_mut(unit)
        .insert(make_tween_anim(transform_tween(600)));
    let sprite_anim = app
        .world_mut()
        .spawn((
            make_tween_anim(sprite_tween(200)),
            AnimTarget::component::<Sprite>(unit),
        ))
        .id();

    run_for(&mut app, Duration::from_millis(1));

    assert!(app.world().get_entity(unit).is_ok());
    assert!(active_playing(app.world().get::<TweenAnim>(unit).unwrap()));
    assert!(active_playing(
        app.world().get::<TweenAnim>(sprite_anim).unwrap()
    ));
}

#[test]
fn completed_transform_animator_remains_without_new_tween() {
    let mut app = app_with_card_animations();
    let entity = app
        .world_mut()
        .spawn((Transform::default(), make_tween_anim(transform_tween(10))))
        .id();

    run_for(&mut app, Duration::from_millis(10));
    assert_eq!(
        app.world().get::<TweenAnim>(entity).unwrap().tween_state(),
        TweenState::Completed
    );
    assert!(app.world().get::<TweenAnim>(entity).is_some());

    run_for(&mut app, Duration::from_millis(10));
    assert!(app.world().get::<TweenAnim>(entity).is_some());
}
