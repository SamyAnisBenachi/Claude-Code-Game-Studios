use std::{any::TypeId, time::Duration};

use bevy::{math::curve::EaseFunction, prelude::*, time::TimeUpdateStrategy};
use bevy_tweening::{AnimTarget, AnimTargetKind, PlaybackState, Tween, TweenAnim, TweenState};
use client::card_animations::{
    spawn_sprite_tween_controller, spawn_transform_tween_controller, CardAnimationsPlugin,
    SpriteColorLens, TransformScaleXLens, TransformTranslationXLens, TransformTranslationYLens,
};

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

fn active_playing(playback_state: PlaybackState, tween_state: TweenState) -> bool {
    playback_state == PlaybackState::Playing && tween_state == TweenState::Active
}

fn targeted_controller_states<T: Component>(
    app: &mut App,
    target: Entity,
) -> Vec<(PlaybackState, TweenState)> {
    let world = app.world_mut();
    let mut query = world.query::<(&TweenAnim, &AnimTarget)>();

    query
        .iter(world)
        .filter(|(animator, anim_target)| {
            animator.tweenable().target_type_id() == Some(TypeId::of::<T>())
                && matches!(
                    anim_target.kind,
                    AnimTargetKind::Component { entity } if entity == target
                )
        })
        .map(|(animator, _)| (animator.playback_state, animator.tween_state()))
        .collect()
}

fn assert_approx_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= f32::EPSILON,
        "expected {expected}, got {actual}"
    );
}

fn x_translation_tween(start: f32, end: f32, duration_ms: u64) -> Tween {
    Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(duration_ms),
        TransformTranslationXLens { start, end },
    )
}

fn y_translation_tween(start: f32, end: f32, duration_ms: u64) -> Tween {
    Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(duration_ms),
        TransformTranslationYLens { start, end },
    )
}

fn scale_x_tween(start: f32, end: f32, duration_ms: u64) -> Tween {
    Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(duration_ms),
        TransformScaleXLens { start, end },
    )
}

fn sprite_color_tween(start: Color, end: Color, duration_ms: u64) -> Tween {
    Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(duration_ms),
        SpriteColorLens { start, end },
    )
}

#[test]
fn repel_displacement_and_placement_reveal_controllers_play_simultaneously() {
    let mut app = app_with_card_animations();
    let unit = app
        .world_mut()
        .spawn((Transform::IDENTITY, Sprite::default()))
        .id();

    {
        let mut commands = app.world_mut().commands();
        spawn_transform_tween_controller(&mut commands, unit, x_translation_tween(0.0, 48.0, 600));
        spawn_sprite_tween_controller(
            &mut commands,
            unit,
            sprite_color_tween(
                Color::srgba(0.2, 0.2, 0.2, 1.0),
                Color::srgba(1.0, 1.0, 1.0, 1.0),
                100,
            ),
        );
        spawn_transform_tween_controller(&mut commands, unit, scale_x_tween(1.0, 0.2, 100));
    }
    app.world_mut().flush();

    run_for(&mut app, Duration::from_millis(1));

    let transform_controllers = targeted_controller_states::<Transform>(&mut app, unit);
    let sprite_controllers = targeted_controller_states::<Sprite>(&mut app, unit);

    assert_eq!(transform_controllers.len(), 2);
    assert_eq!(sprite_controllers.len(), 1);
    assert!(transform_controllers
        .into_iter()
        .all(|(playback, tween)| active_playing(playback, tween)));
    assert!(sprite_controllers
        .into_iter()
        .all(|(playback, tween)| active_playing(playback, tween)));
    assert!(app.world().get::<TweenAnim>(unit).is_none());
}

#[test]
fn parallel_transform_controllers_advance_x_and_y_in_same_update() {
    let mut app = app_with_card_animations();
    let unit = app.world_mut().spawn(Transform::IDENTITY).id();

    {
        let mut commands = app.world_mut().commands();
        spawn_transform_tween_controller(&mut commands, unit, x_translation_tween(0.0, 100.0, 600));
        spawn_transform_tween_controller(&mut commands, unit, y_translation_tween(0.0, 60.0, 600));
    }
    app.world_mut().flush();

    run_for(&mut app, Duration::from_millis(16));

    let transform_controllers = targeted_controller_states::<Transform>(&mut app, unit);
    let transform = app.world().get::<Transform>(unit).unwrap();

    assert_eq!(transform_controllers.len(), 2);
    assert!(transform_controllers
        .into_iter()
        .all(|(playback, tween)| active_playing(playback, tween)));
    assert!(transform.translation.x > 0.0 && transform.translation.x < 100.0);
    assert!(transform.translation.y > 0.0 && transform.translation.y < 60.0);
}

#[test]
fn parallel_transform_controllers_do_not_move_before_time_advances() {
    let mut app = app_with_card_animations();
    let unit = app.world_mut().spawn(Transform::IDENTITY).id();

    {
        let mut commands = app.world_mut().commands();
        spawn_transform_tween_controller(&mut commands, unit, x_translation_tween(0.0, 100.0, 600));
        spawn_transform_tween_controller(&mut commands, unit, y_translation_tween(0.0, 60.0, 600));
    }
    app.world_mut().flush();

    run_for(&mut app, Duration::ZERO);

    let transform = app.world().get::<Transform>(unit).unwrap();
    assert_approx_eq(transform.translation.x, 0.0);
    assert_approx_eq(transform.translation.y, 0.0);
}

#[test]
fn parallel_transform_controllers_reach_final_x_and_y_values() {
    let mut app = app_with_card_animations();
    let unit = app.world_mut().spawn(Transform::IDENTITY).id();

    {
        let mut commands = app.world_mut().commands();
        spawn_transform_tween_controller(&mut commands, unit, x_translation_tween(0.0, 100.0, 600));
        spawn_transform_tween_controller(&mut commands, unit, y_translation_tween(0.0, 60.0, 600));
    }
    app.world_mut().flush();

    run_for(&mut app, Duration::from_millis(600));

    let transform = app.world().get::<Transform>(unit).unwrap();
    assert_approx_eq(transform.translation.x, 100.0);
    assert_approx_eq(transform.translation.y, 60.0);
}
