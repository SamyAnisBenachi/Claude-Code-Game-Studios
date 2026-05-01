use std::time::Duration;

use bevy::color::Alpha;
use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy_tweening::lens::Lens;
use bevy_tweening::{Tween, TweenAnim, TweeningPlugin};
use client::card_animations::{
    AnimQueue, AnimationTimingConfig, AuctionPanelTransitionRequested, AuraPulseRequested,
    BackgroundColorAlphaLens, BoardRebuildRequested, CardAcquiredAnimReady, CardAnimationsPlugin,
    DamageNumberSpawnRequested, DisplacementAnimRequested, GoldTickRequested, GroupDrainedSignal,
    HandHideRequested, HandShowRequested, ObjectiveDestroyedAnimReady,
    PendingObjectiveDestroyedEvents, PendingPhaseChange, PlacementCancelAllAnimsRequested,
    PlacementRevealAnimReady, SettlementOverlayRequested, SnapBackRequested, SpriteAlphaLens,
    SpriteColorLens, StagedObjectiveRevealQueue, TextColorLens, TimerBarEaseRequested,
    TransformScaleXLens, TrapFlipRequested,
};

#[test]
fn plugin_builds_and_registers_resources_and_messages() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(CardAnimationsPlugin);

    app.update();

    assert!(app
        .world()
        .get_resource::<StagedObjectiveRevealQueue>()
        .is_some());
    assert!(app
        .world()
        .get_resource::<AnimationTimingConfig>()
        .is_some());
    assert!(app.world().get_resource::<AnimQueue>().is_some());
    assert!(app.world().get_resource::<PendingPhaseChange>().is_some());
    assert!(app
        .world()
        .get_resource::<PendingObjectiveDestroyedEvents>()
        .is_some());
    assert!(app
        .world()
        .get_resource::<Messages<GroupDrainedSignal>>()
        .is_some());
    assert!(app
        .world()
        .get_resource::<Messages<PlacementRevealAnimReady>>()
        .is_some());
    assert!(app
        .world()
        .get_resource::<Messages<AuraPulseRequested>>()
        .is_some());
}

#[test]
fn plugin_does_not_readd_existing_tweening_plugin() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(TweeningPlugin);
    app.add_plugins(CardAnimationsPlugin);

    app.update();

    assert!(app
        .world()
        .get_resource::<StagedObjectiveRevealQueue>()
        .is_some());
}

#[test]
fn all_domain_messages_are_registered_and_writable() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(CardAnimationsPlugin);

    app.world_mut()
        .resource_mut::<Messages<PlacementRevealAnimReady>>()
        .write(PlacementRevealAnimReady);
    app.world_mut()
        .resource_mut::<Messages<ObjectiveDestroyedAnimReady>>()
        .write(ObjectiveDestroyedAnimReady);
    app.world_mut()
        .resource_mut::<Messages<DamageNumberSpawnRequested>>()
        .write(DamageNumberSpawnRequested);
    app.world_mut()
        .resource_mut::<Messages<BoardRebuildRequested>>()
        .write(BoardRebuildRequested);
    app.world_mut()
        .resource_mut::<Messages<PlacementCancelAllAnimsRequested>>()
        .write(PlacementCancelAllAnimsRequested);
    app.world_mut()
        .resource_mut::<Messages<CardAcquiredAnimReady>>()
        .write(CardAcquiredAnimReady);
    app.world_mut()
        .resource_mut::<Messages<SnapBackRequested>>()
        .write(SnapBackRequested);
    app.world_mut()
        .resource_mut::<Messages<HandHideRequested>>()
        .write(HandHideRequested);
    app.world_mut()
        .resource_mut::<Messages<HandShowRequested>>()
        .write(HandShowRequested);
    app.world_mut()
        .resource_mut::<Messages<AuctionPanelTransitionRequested>>()
        .write(AuctionPanelTransitionRequested);
    app.world_mut()
        .resource_mut::<Messages<TimerBarEaseRequested>>()
        .write(TimerBarEaseRequested);
    app.world_mut()
        .resource_mut::<Messages<GoldTickRequested>>()
        .write(GoldTickRequested);
    app.world_mut()
        .resource_mut::<Messages<SettlementOverlayRequested>>()
        .write(SettlementOverlayRequested);
    app.world_mut()
        .resource_mut::<Messages<DisplacementAnimRequested>>()
        .write(DisplacementAnimRequested);
    app.world_mut()
        .resource_mut::<Messages<TrapFlipRequested>>()
        .write(TrapFlipRequested);
    app.world_mut()
        .resource_mut::<Messages<AuraPulseRequested>>()
        .write(AuraPulseRequested);
    app.world_mut()
        .resource_mut::<Messages<GroupDrainedSignal>>()
        .write(GroupDrainedSignal);

    app.update();
}

#[test]
fn sprite_alpha_lens_constructs_and_lerps_alpha() {
    let mut world = World::new();
    let entity = world.spawn(Sprite::default()).id();
    let tween = Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(100),
        SpriteAlphaLens {
            start: 0.25,
            end: 0.75,
        },
    );
    world.entity_mut(entity).insert(TweenAnim::new(tween));

    assert!(world.get::<TweenAnim>(entity).is_some());

    let mut entity_mut = world.entity_mut(entity);
    let mut sprite = entity_mut.get_mut::<Sprite>().unwrap();
    SpriteAlphaLens {
        start: 0.25,
        end: 0.75,
    }
    .lerp(sprite.reborrow(), 0.0);
    assert_eq!(sprite.color.alpha(), 0.25);

    SpriteAlphaLens {
        start: 0.25,
        end: 0.75,
    }
    .lerp(sprite.reborrow(), 1.0);
    assert_eq!(sprite.color.alpha(), 0.75);
}

#[test]
fn background_color_alpha_lens_clamps_alpha() {
    let mut world = World::new();
    let entity = world
        .spawn(BackgroundColor(Color::srgba(0.1, 0.2, 0.3, 0.0)))
        .id();
    let tween = Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(100),
        BackgroundColorAlphaLens {
            start: 0.25,
            end: 0.75,
        },
    );
    world.entity_mut(entity).insert(TweenAnim::new(tween));

    assert!(world.get::<TweenAnim>(entity).is_some());

    let mut entity_mut = world.entity_mut(entity);
    let mut color = entity_mut.get_mut::<BackgroundColor>().unwrap();
    BackgroundColorAlphaLens {
        start: 0.25,
        end: 0.75,
    }
    .lerp(color.reborrow(), 1.5);
    assert_eq!(color.0.alpha(), 1.0);
}

#[test]
fn sprite_color_lens_constructs_and_lerps_rgba() {
    let mut world = World::new();
    let entity = world.spawn(Sprite::default()).id();
    let tween = Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(100),
        SpriteColorLens {
            start: Color::srgba(0.0, 0.2, 0.4, 0.6),
            end: Color::srgba(1.0, 0.8, 0.6, 0.4),
        },
    );
    world.entity_mut(entity).insert(TweenAnim::new(tween));

    assert!(world.get::<TweenAnim>(entity).is_some());

    let mut entity_mut = world.entity_mut(entity);
    let mut sprite = entity_mut.get_mut::<Sprite>().unwrap();
    SpriteColorLens {
        start: Color::srgba(0.0, 0.2, 0.4, 0.6),
        end: Color::srgba(1.0, 0.8, 0.6, 0.4),
    }
    .lerp(sprite.reborrow(), 0.5);

    let color = sprite.color.to_srgba();
    assert_eq!(color.red, 0.5);
    assert_eq!(color.green, 0.5);
    assert_eq!(color.blue, 0.5);
    assert_eq!(color.alpha, 0.5);
}

#[test]
fn transform_scale_x_lens_clamps_to_zero_and_preserves_yz() {
    let mut world = World::new();
    let entity = world
        .spawn(Transform::from_scale(Vec3::new(1.0, 2.0, 3.0)))
        .id();
    let tween = Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(100),
        TransformScaleXLens {
            start: 1.0,
            end: 0.0,
        },
    );
    world.entity_mut(entity).insert(TweenAnim::new(tween));

    assert!(world.get::<TweenAnim>(entity).is_some());

    let mut entity_mut = world.entity_mut(entity);
    let mut transform = entity_mut.get_mut::<Transform>().unwrap();
    TransformScaleXLens {
        start: 0.1,
        end: -1.0,
    }
    .lerp(transform.reborrow(), 1.0);
    assert_eq!(transform.scale, Vec3::new(0.0, 2.0, 3.0));
}

#[test]
fn text_color_lens_constructs_for_text_color_component() {
    let mut world = World::new();
    let entity = world.spawn(TextColor(Color::WHITE)).id();
    let tween = Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(100),
        TextColorLens {
            start: Color::srgba(1.0, 0.0, 0.0, 1.0),
            end: Color::srgba(0.0, 0.0, 1.0, 0.0),
        },
    );
    world.entity_mut(entity).insert(TweenAnim::new(tween));

    assert!(world.get::<TweenAnim>(entity).is_some());

    let mut entity_mut = world.entity_mut(entity);
    let mut text_color = entity_mut.get_mut::<TextColor>().unwrap();
    TextColorLens {
        start: Color::srgba(1.0, 0.0, 0.0, 1.0),
        end: Color::srgba(0.0, 0.0, 1.0, 0.0),
    }
    .lerp(text_color.reborrow(), 0.5);

    let color = text_color.0.to_srgba();
    assert_eq!(color.red, 0.5);
    assert_eq!(color.green, 0.0);
    assert_eq!(color.blue, 0.5);
    assert_eq!(color.alpha, 0.5);
}
