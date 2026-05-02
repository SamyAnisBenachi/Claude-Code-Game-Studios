use std::{any::TypeId, time::Duration};

use bevy::{math::curve::EaseFunction, prelude::*};
use bevy_tweening::{PlaybackState, Tween, TweenAnim, Tweenable, TweeningError};

use super::lenses::{SpriteAlphaLens, TransformScaleXLens};

pub const PLACEMENT_ANIMATION_CAP_MS: u64 = 250;

#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlacementPhaseAnimator;

pub fn placement_phase_duration(duration_ms: u64) -> Duration {
    Duration::from_millis(duration_ms.min(PLACEMENT_ANIMATION_CAP_MS))
}

pub fn make_tween_anim(tweenable: impl Tweenable + 'static) -> TweenAnim {
    TweenAnim::new(tweenable).with_destroy_on_completed(false)
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
