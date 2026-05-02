use std::{any::TypeId, time::Duration};

use bevy::{math::curve::EaseFunction, prelude::*};
use bevy_tweening::{AnimTarget, PlaybackState, Tween, TweenAnim, Tweenable, TweeningError};

use super::lenses::{SpriteAlphaLens, TransformScaleXLens};

#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlacementPhaseAnimator;

pub fn make_tween_anim(tweenable: impl Tweenable + 'static) -> TweenAnim {
    TweenAnim::new(tweenable).with_destroy_on_completed(false)
}

pub fn spawn_transform_tween_controller(
    commands: &mut Commands,
    target: Entity,
    tweenable: impl Tweenable + 'static,
) -> Entity {
    commands
        .spawn((
            make_tween_anim(tweenable),
            AnimTarget::component::<Transform>(target),
        ))
        .id()
}

pub fn spawn_sprite_tween_controller(
    commands: &mut Commands,
    target: Entity,
    tweenable: impl Tweenable + 'static,
) -> Entity {
    commands
        .spawn((
            make_tween_anim(tweenable),
            AnimTarget::component::<Sprite>(target),
        ))
        .id()
}

pub fn replace_tweenable(
    animator: &mut TweenAnim,
    tweenable: impl Tweenable + 'static,
) -> Result<(), TweeningError> {
    animator.set_tweenable(tweenable).map(|_| ())
}

pub fn cancel_tween_anim_in_place(animator: &mut TweenAnim) -> Result<(), TweeningError> {
    animator.destroy_on_completion = false;

    match animator.tweenable().target_type_id() {
        Some(target) if target == TypeId::of::<Transform>() => {
            replace_tweenable(animator, cancelled_transform_tween())?;
        }
        Some(target) if target == TypeId::of::<Sprite>() => {
            replace_tweenable(animator, cancelled_sprite_tween())?;
        }
        _ => {}
    }

    animator.playback_state = PlaybackState::Paused;
    Ok(())
}

fn cancelled_transform_tween() -> Tween {
    Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(1),
        TransformScaleXLens {
            start: 1.0,
            end: 1.0,
        },
    )
}

fn cancelled_sprite_tween() -> Tween {
    Tween::new(
        EaseFunction::Linear,
        Duration::from_millis(1),
        SpriteAlphaLens {
            start: 1.0,
            end: 1.0,
        },
    )
}
