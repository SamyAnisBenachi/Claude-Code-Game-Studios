use std::{any::TypeId, time::Duration};

use bevy::{math::curve::EaseFunction, prelude::*, time::TimeUpdateStrategy};
use bevy_tweening::{
    lens::{TransformPositionLens, TransformScaleLens},
    AnimTarget, AnimTargetKind, PlaybackState, Tween, TweenAnim, TweenState,
};
use client::{
    card_animations::{
        make_tween_anim, CardAnimationsPlugin, CellHighlightRequested, HandCard,
        HandCardDragStarted, HandCardHoverEntered, InputGatingAnimationConfig,
        PlacementCancelAllAnimsRequested, PlacementPhaseAnimator, PlacementRevealAnimReady,
        PlacementRevealEntry, SnapBackRequested,
    },
    state::ClientPhaseView,
    ui::shared::{BoardLayout, LaneCell},
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
    app.insert_resource(BoardLayout {
        board_origin: Vec2::new(10.0, 20.0),
        cell_width: 64.0,
        lane_height: 80.0,
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

fn animator_duration(app: &App, entity: Entity) -> Duration {
    app.world()
        .get::<TweenAnim>(entity)
        .expect("entity should have TweenAnim")
        .tweenable()
        .cycle_duration()
}

fn transform_tween(duration_ms: u64) -> Tween {
    Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(duration_ms),
        TransformPositionLens {
            start: Vec3::ZERO,
            end: Vec3::new(100.0, 100.0, 0.0),
        },
    )
}

fn active_placement_animator_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&TweenAnim, With<PlacementPhaseAnimator>>();
    query
        .iter(world)
        .filter(|animator| active_playing(animator))
        .count()
}

#[test]
fn placement_reveal_payload_starts_all_entries_same_update() {
    let mut app = app_with_card_animations();
    let entries = (1..=5)
        .map(|lane| {
            let unit = app
                .world_mut()
                .spawn((
                    Transform::from_xyz(-999.0, -999.0, lane as f32),
                    LaneCell { lane, cell: lane },
                ))
                .id();
            PlacementRevealEntry {
                unit,
                lane,
                cell: lane,
            }
        })
        .collect::<Vec<_>>();

    app.world_mut().write_message(PlacementRevealAnimReady {
        entries: entries.clone(),
    });
    run_for(&mut app, Duration::ZERO);

    let layout = *app.world().resource::<BoardLayout>();
    for entry in entries {
        let animator = app
            .world()
            .get::<TweenAnim>(entry.unit)
            .expect("reveal tween should be inserted");
        assert!(active_playing(animator));
        assert_eq!(
            animator.tweenable().target_type_id(),
            Some(TypeId::of::<Transform>())
        );
        assert_eq!(
            animator.tweenable().cycle_duration(),
            Duration::from_millis(250)
        );
        assert_eq!(
            app.world()
                .get::<Transform>(entry.unit)
                .expect("unit should keep Transform")
                .translation
                .truncate(),
            layout.cell_to_world(entry.lane, entry.cell)
        );
    }
}

#[test]
fn placement_reveal_empty_payload_is_noop() {
    let mut app = app_with_card_animations();

    app.world_mut().write_message(PlacementRevealAnimReady {
        entries: Vec::new(),
    });
    run_for(&mut app, Duration::ZERO);

    let world = app.world_mut();
    let mut query = world.query::<&TweenAnim>();
    assert_eq!(query.iter(world).count(), 0);
}

#[test]
fn drag_lift_duration_clamps_to_placement_cap() {
    let mut app = app_with_card_animations();
    app.world_mut().insert_resource(InputGatingAnimationConfig {
        drag_lift_ms: 300,
        ..default()
    });
    let card = app.world_mut().spawn((HandCard, Transform::default())).id();
    let drag_sprite = app
        .world_mut()
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                ..default()
            },
            ImageNode::new(Handle::<Image>::default()),
        ))
        .id();

    app.world_mut().write_message(HandCardDragStarted { card });
    app.world_mut()
        .entity_mut(drag_sprite)
        .insert(client::card_animations::HandDragSprite);
    run_for(&mut app, Duration::ZERO);

    assert_eq!(
        animator_duration(&app, drag_sprite),
        Duration::from_millis(250)
    );
    assert!(app
        .world()
        .get::<PlacementPhaseAnimator>(drag_sprite)
        .is_some());
}

#[test]
fn snap_back_duration_clamps_to_placement_cap() {
    let mut app = app_with_card_animations();
    app.world_mut().insert_resource(InputGatingAnimationConfig {
        snap_back_duration_ms: 300,
        ..default()
    });
    let target = app
        .world_mut()
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(24.0),
            ..default()
        })
        .id();

    app.world_mut().write_message(SnapBackRequested {
        target,
        end_position: UiRect {
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            ..default()
        },
    });
    run_for(&mut app, Duration::ZERO);

    assert_eq!(animator_duration(&app, target), Duration::from_millis(250));
    assert!(app.world().get::<PlacementPhaseAnimator>(target).is_some());
}

#[test]
fn hover_duration_clamps_to_placement_cap() {
    let mut app = app_with_card_animations();
    app.world_mut().insert_resource(InputGatingAnimationConfig {
        hand_card_hover_ms: 300,
        ..default()
    });
    let card = app
        .world_mut()
        .spawn((HandCard, Transform::from_scale(Vec3::splat(1.0))))
        .id();

    app.world_mut().write_message(HandCardHoverEntered { card });
    run_for(&mut app, Duration::ZERO);

    assert_eq!(animator_duration(&app, card), Duration::from_millis(250));
    assert!(app.world().get::<PlacementPhaseAnimator>(card).is_some());
}

#[test]
fn cell_highlight_duration_clamps_to_placement_cap() {
    let mut app = app_with_card_animations();
    app.world_mut().insert_resource(InputGatingAnimationConfig {
        cell_highlight_ms: 300,
        ..default()
    });
    let cell = app.world_mut().spawn(Sprite::default()).id();

    app.world_mut()
        .write_message(CellHighlightRequested { target: cell });
    run_for(&mut app, Duration::ZERO);

    assert_eq!(animator_duration(&app, cell), Duration::from_millis(250));
    assert!(app.world().get::<PlacementPhaseAnimator>(cell).is_some());
}

#[test]
fn placement_duration_below_cap_is_preserved() {
    let mut app = app_with_card_animations();
    app.world_mut().insert_resource(InputGatingAnimationConfig {
        snap_back_duration_ms: 200,
        ..default()
    });
    let target = app.world_mut().spawn(Node::default()).id();

    app.world_mut().write_message(SnapBackRequested {
        target,
        end_position: UiRect::default(),
    });
    run_for(&mut app, Duration::ZERO);

    assert_eq!(animator_duration(&app, target), Duration::from_millis(200));
}

#[test]
fn placement_cancel_stops_animators_and_snaps_targets_to_lane_cells() {
    let mut app = app_with_card_animations();
    let direct_target = app
        .world_mut()
        .spawn((
            Transform::from_xyz(111.0, 222.0, 3.5),
            LaneCell { lane: 1, cell: 2 },
            PlacementPhaseAnimator,
            make_tween_anim(transform_tween(250)),
        ))
        .id();
    let explicit_target = app
        .world_mut()
        .spawn((
            Transform::from_xyz(333.0, 444.0, 4.5),
            LaneCell { lane: 2, cell: 3 },
        ))
        .id();
    let explicit_controller = app
        .world_mut()
        .spawn((
            PlacementPhaseAnimator,
            make_tween_anim(transform_tween(250)),
            AnimTarget::component::<Transform>(explicit_target),
        ))
        .id();

    app.world_mut()
        .write_message(PlacementCancelAllAnimsRequested);
    run_for(&mut app, Duration::ZERO);

    assert_eq!(active_placement_animator_count(&mut app), 0);
    let layout = *app.world().resource::<BoardLayout>();
    assert_eq!(
        app.world()
            .get::<Transform>(direct_target)
            .unwrap()
            .translation,
        layout.cell_to_world(1, 2).extend(3.5)
    );
    assert_eq!(
        app.world()
            .get::<Transform>(explicit_target)
            .unwrap()
            .translation,
        layout.cell_to_world(2, 3).extend(4.5)
    );

    let explicit_animator = app.world().get::<TweenAnim>(explicit_controller).unwrap();
    assert_eq!(explicit_animator.playback_state, PlaybackState::Paused);
    let explicit_anim_target = app.world().get::<AnimTarget>(explicit_controller).unwrap();
    assert!(matches!(
        explicit_anim_target.kind,
        AnimTargetKind::Component { entity } if entity == explicit_target
    ));
    assert!(app
        .world()
        .get::<PlacementPhaseAnimator>(direct_target)
        .is_some());
    assert!(app
        .world()
        .get::<PlacementPhaseAnimator>(explicit_controller)
        .is_some());
}

#[test]
fn placement_cancel_without_lane_cell_does_not_panic() {
    let mut app = app_with_card_animations();
    let entity = app
        .world_mut()
        .spawn((
            Transform::default(),
            PlacementPhaseAnimator,
            make_tween_anim(Tween::new(
                EaseFunction::QuadraticOut,
                Duration::from_millis(250),
                TransformScaleLens {
                    start: Vec3::splat(1.0),
                    end: Vec3::splat(1.2),
                },
            )),
        ))
        .id();

    app.world_mut()
        .write_message(PlacementCancelAllAnimsRequested);
    run_for(&mut app, Duration::ZERO);

    assert_eq!(
        app.world().get::<TweenAnim>(entity).unwrap().playback_state,
        PlaybackState::Paused
    );
}
