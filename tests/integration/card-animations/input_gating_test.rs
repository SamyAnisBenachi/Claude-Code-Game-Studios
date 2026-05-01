use std::time::Duration;

use bevy::{math::curve::EaseFunction, prelude::*, time::TimeUpdateStrategy};
use bevy_tweening::{lens::TransformScaleLens, PlaybackState, Tween, TweenAnim, TweenState};
use client::{
    card_animations::{
        make_tween_anim, CardAnimationsPlugin, HandCard, HandCardDragStarted, HandCardHoverEntered,
        HandCardScaleAnimation, HandCardScaleDirection, HandDragSprite, InputGatingAnimationConfig,
        PlacementPhaseAnimator, TimerBar, TimerBarEaseRequested,
    },
    state::ClientPhaseView,
};
use shared::protocol::RoundPhase;

fn app_with_card_animations() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(CardAnimationsPlugin);
    app.insert_resource(ClientPhaseView {
        phase: RoundPhase::Placement,
        round_number: 1,
        timer_duration_ms: 10_000,
    });
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

fn hand_card_count_playing(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query::<(&TweenAnim, &HandCard)>();
    query
        .iter(world)
        .filter(|(animator, _)| active_playing(animator))
        .count()
}

fn hovering_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query::<&HandCardScaleAnimation>();
    query
        .iter(world)
        .filter(|animation| animation.direction == HandCardScaleDirection::Hovering)
        .count()
}

fn return_tween(start: f32, duration_ms: u64) -> Tween {
    Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(duration_ms),
        TransformScaleLens {
            start: Vec3::splat(start),
            end: Vec3::splat(1.0),
        },
    )
}

#[test]
fn timer_bar_ease_request_starts_animator_same_update() {
    let mut app = app_with_card_animations();
    let timer_bar = app
        .world_mut()
        .spawn((
            TimerBar,
            Node {
                width: Val::Percent(80.0),
                ..default()
            },
        ))
        .id();

    app.world_mut().write_message(TimerBarEaseRequested {
        target_width_percent: 40.0,
    });
    run_for(&mut app, Duration::ZERO);

    let animator = app
        .world()
        .get::<TweenAnim>(timer_bar)
        .expect("timer bar tween should be inserted in the same update");
    assert!(active_playing(animator));
}

#[test]
fn timer_bar_duplicate_requests_leave_one_replaced_animator() {
    let mut app = app_with_card_animations();
    let timer_bar = app
        .world_mut()
        .spawn((
            TimerBar,
            Node {
                width: Val::Percent(90.0),
                ..default()
            },
        ))
        .id();

    app.world_mut().write_message(TimerBarEaseRequested {
        target_width_percent: 60.0,
    });
    app.world_mut().write_message(TimerBarEaseRequested {
        target_width_percent: 25.0,
    });
    run_for(&mut app, Duration::ZERO);

    let animator = app
        .world()
        .get::<TweenAnim>(timer_bar)
        .expect("timer bar should keep one animator component");
    assert!(active_playing(animator));
    assert_eq!(
        animator.tweenable().cycle_duration(),
        Duration::from_millis(150)
    );
}

#[test]
fn timer_bar_missing_target_does_not_panic() {
    let mut app = app_with_card_animations();

    app.world_mut().write_message(TimerBarEaseRequested {
        target_width_percent: 50.0,
    });
    run_for(&mut app, Duration::ZERO);

    let world = app.world_mut();
    let mut query = world.query::<&TweenAnim>();
    assert_eq!(query.iter(world).count(), 0);
}

#[test]
fn drag_start_in_placement_starts_drag_sprite_animator_same_update() {
    let mut app = app_with_card_animations();
    let card = app.world_mut().spawn((HandCard, Transform::default())).id();
    let drag_sprite = app
        .world_mut()
        .spawn((
            HandDragSprite,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            ImageNode::new(Handle::<Image>::default()),
        ))
        .id();

    app.world_mut().write_message(HandCardDragStarted { card });
    run_for(&mut app, Duration::ZERO);

    let animator = app
        .world()
        .get::<TweenAnim>(drag_sprite)
        .expect("drag sprite tween should be inserted in the same update");
    assert!(active_playing(animator));
    assert!(app
        .world()
        .get::<PlacementPhaseAnimator>(drag_sprite)
        .is_some());
    assert!(app.world().get::<Sprite>(drag_sprite).is_none());
}

#[test]
fn drag_start_outside_placement_does_not_gate_or_animate() {
    let mut app = app_with_card_animations();
    app.world_mut().insert_resource(ClientPhaseView {
        phase: RoundPhase::DraftAuction,
        round_number: 1,
        timer_duration_ms: 20_000,
    });
    let card = app.world_mut().spawn((HandCard, Transform::default())).id();
    let drag_sprite = app
        .world_mut()
        .spawn((
            HandDragSprite,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            ImageNode::new(Handle::<Image>::default()),
        ))
        .id();

    app.world_mut().write_message(HandCardDragStarted { card });
    run_for(&mut app, Duration::ZERO);

    assert!(app.world().get::<TweenAnim>(drag_sprite).is_none());
}

#[test]
fn hover_enter_keeps_returning_card_playing_and_starts_new_hover() {
    let mut app = app_with_card_animations();
    app.world_mut().insert_resource(InputGatingAnimationConfig {
        hand_card_dehover_ms: 5,
        hand_card_dehover_min_ms: 40,
        ..default()
    });
    let card_a = app
        .world_mut()
        .spawn((
            HandCard,
            Transform::from_scale(Vec3::splat(1.08)),
            make_tween_anim(return_tween(1.08, 5)),
            HandCardScaleAnimation::returning(),
        ))
        .id();
    let card_b = app
        .world_mut()
        .spawn((HandCard, Transform::from_scale(Vec3::splat(1.0))))
        .id();
    let card_c = app
        .world_mut()
        .spawn((HandCard, Transform::from_scale(Vec3::splat(1.0))))
        .id();

    app.world_mut()
        .write_message(HandCardHoverEntered { card: card_b });
    run_for(&mut app, Duration::ZERO);

    let animator_a = app.world().get::<TweenAnim>(card_a).unwrap();
    let animator_b = app.world().get::<TweenAnim>(card_b).unwrap();
    assert!(active_playing(animator_a));
    assert!(active_playing(animator_b));
    assert!(animator_a.tweenable().cycle_duration() >= Duration::from_millis(40));

    let scale_a = app.world().get::<Transform>(card_a).unwrap().scale.x;
    assert!(scale_a > 1.0 && scale_a <= 1.12);
    assert_eq!(hand_card_count_playing(&mut app), 2);
    assert_eq!(hovering_count(&mut app), 1);
    assert_eq!(app.world().get::<Transform>(card_c).unwrap().scale.x, 1.0);
}
