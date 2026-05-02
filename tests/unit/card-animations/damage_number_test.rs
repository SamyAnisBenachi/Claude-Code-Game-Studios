use std::{any::TypeId, time::Duration};

use bevy::{math::curve::EaseFunction, prelude::*, text::LineHeight, time::TimeUpdateStrategy};
use bevy_tweening::{
    lens::TransformPositionLens, AnimTarget, AnimTargetKind, PlaybackState, Tween, TweenAnim,
    TweenState,
};
use client::card_animations::{
    damage_number_jitter, make_tween_anim, AnimationTimingConfig, CardAnimationsPlugin,
    DamageNumber, DamageNumberSpawnRequested, DespawnAfter, TextColorLens,
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

fn active_playing(animator: &TweenAnim) -> bool {
    animator.playback_state == PlaybackState::Playing
        && animator.tween_state() == TweenState::Active
}

fn float_tween(duration_ms: u64) -> Tween {
    Tween::new(
        EaseFunction::CubicOut,
        Duration::from_millis(duration_ms),
        TransformPositionLens {
            start: Vec3::ZERO,
            end: Vec3::new(0.0, 60.0, 0.0),
        },
    )
}

fn fade_tween(duration_ms: u64) -> Tween {
    Tween::new(
        EaseFunction::CubicOut,
        Duration::from_millis(duration_ms),
        TextColorLens {
            start: Color::WHITE,
            end: Color::srgba(1.0, 1.0, 1.0, 0.0),
        },
    )
}

fn spawn_prebuilt_damage_number(
    app: &mut App,
    despawn_after_ms: u64,
    float_duration_ms: u64,
    fade_duration_ms: u64,
) -> Entity {
    let damage_number = app
        .world_mut()
        .spawn((
            Text2d::new("15"),
            TextFont::default(),
            TextColor(Color::WHITE),
            LineHeight::default(),
            Transform::default(),
            DamageNumber,
            DespawnAfter(Timer::new(
                Duration::from_millis(despawn_after_ms),
                TimerMode::Once,
            )),
        ))
        .id();

    app.world_mut().spawn((
        make_tween_anim(float_tween(float_duration_ms)),
        AnimTarget::component::<Transform>(damage_number),
        ChildOf(damage_number),
    ));
    app.world_mut().spawn((
        make_tween_anim(fade_tween(fade_duration_ms)),
        AnimTarget::component::<TextColor>(damage_number),
        ChildOf(damage_number),
    ));

    damage_number
}

fn damage_number_entities(app: &mut App) -> Vec<Entity> {
    let world = app.world_mut();
    let mut query = world.query_filtered::<Entity, With<DamageNumber>>();
    query.iter(world).collect()
}

fn controller_count_for(app: &mut App, entity: Entity, component: TypeId) -> usize {
    let world = app.world_mut();
    let mut query = world.query::<(&TweenAnim, &AnimTarget)>();
    query
        .iter(world)
        .filter(|(animator, target)| {
            active_playing(animator)
                && animator.tweenable().target_type_id() == Some(component)
                && matches!(
                    target.kind,
                    AnimTargetKind::Component { entity: target_entity }
                        if target_entity == entity
                )
        })
        .count()
}

#[test]
fn damage_number_despawns_at_symmetric_f2_delay() {
    let mut app = app_with_card_animations();
    let entity = spawn_prebuilt_damage_number(&mut app, 500, 500, 500);

    run_for(&mut app, Duration::from_millis(499));
    assert!(app.world().get_entity(entity).is_ok());

    run_for(&mut app, Duration::from_millis(1));
    assert!(app.world().get_entity(entity).is_err());
}

#[test]
fn damage_number_waits_for_later_asymmetric_f2_delay() {
    let mut app = app_with_card_animations();
    let entity = spawn_prebuilt_damage_number(&mut app, 600, 400, 600);

    run_for(&mut app, Duration::from_millis(400));
    assert!(app.world().get_entity(entity).is_ok());

    run_for(&mut app, Duration::from_millis(200));
    assert!(app.world().get_entity(entity).is_err());
}

#[test]
fn damage_number_spawn_request_creates_world_space_text_and_controllers() {
    let mut app = app_with_card_animations();
    let target_transform = Transform::from_xyz(32.0, 48.0, 2.0);
    let target = app.world_mut().spawn(target_transform).id();

    app.world_mut().write_message(DamageNumberSpawnRequested {
        target,
        damage_value: 15,
        event_id: 0,
    });
    run_for(&mut app, Duration::ZERO);

    let entities = damage_number_entities(&mut app);
    assert_eq!(entities.len(), 1);
    let damage_number = entities[0];

    assert_eq!(app.world().get::<Text2d>(damage_number).unwrap().0, "15");
    assert!(app.world().get::<TextFont>(damage_number).is_some());
    assert!(app.world().get::<TextColor>(damage_number).is_some());
    assert!(app.world().get::<LineHeight>(damage_number).is_some());
    assert_eq!(
        app.world()
            .get::<Transform>(damage_number)
            .unwrap()
            .translation,
        target_transform.translation
    );

    let timing = *app.world().resource::<AnimationTimingConfig>();
    assert_eq!(
        app.world()
            .get::<DespawnAfter>(damage_number)
            .unwrap()
            .0
            .duration(),
        timing.damage_number_despawn_delay()
    );
    assert_eq!(
        controller_count_for(&mut app, damage_number, TypeId::of::<Transform>()),
        1
    );
    assert_eq!(
        controller_count_for(&mut app, damage_number, TypeId::of::<TextColor>()),
        1
    );
}

#[test]
fn damage_number_jitter_matches_gdd_f3_table() {
    let expected = [
        Vec2::new(0.0, 0.0),
        Vec2::new(14.0, 6.0),
        Vec2::new(-14.0, 6.0),
        Vec2::new(8.0, 18.0),
        Vec2::new(-8.0, 18.0),
        Vec2::new(20.0, -2.0),
        Vec2::new(-20.0, -2.0),
        Vec2::new(0.0, 24.0),
    ];

    for (event_id, expected_offset) in expected.iter().copied().enumerate() {
        assert_eq!(damage_number_jitter(event_id as u32), expected_offset);
    }

    assert_eq!(damage_number_jitter(8), expected[0]);
    assert_eq!(damage_number_jitter(15), expected[7]);
}

#[test]
fn simultaneous_damage_numbers_use_distinct_entities_and_deterministic_jitter() {
    let mut app = app_with_card_animations();
    let target_transform = Transform::from_xyz(10.0, 20.0, 0.0);
    let target = app.world_mut().spawn(target_transform).id();

    app.world_mut().write_message(DamageNumberSpawnRequested {
        target,
        damage_value: 0,
        event_id: 0,
    });
    app.world_mut().write_message(DamageNumberSpawnRequested {
        target,
        damage_value: 5,
        event_id: 1,
    });
    run_for(&mut app, Duration::ZERO);

    let entities = damage_number_entities(&mut app);
    assert_eq!(entities.len(), 2);

    let positions = entities
        .iter()
        .map(|entity| {
            app.world()
                .get::<Transform>(*entity)
                .expect("damage number should have Transform")
                .translation
        })
        .collect::<Vec<_>>();

    let expected_0 = target_transform.translation;
    let expected_1 = target_transform.translation + Vec3::new(14.0, 6.0, 0.0);
    assert!(positions.contains(&expected_0));
    assert!(positions.contains(&expected_1));
}
