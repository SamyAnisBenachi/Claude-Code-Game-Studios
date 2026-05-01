use bevy_tweening::{TweenAnim, Tweenable, TweeningError};

pub fn make_tween_anim(tweenable: impl Tweenable + 'static) -> TweenAnim {
    TweenAnim::new(tweenable).with_destroy_on_completed(false)
}

pub fn replace_tweenable(
    animator: &mut TweenAnim,
    tweenable: impl Tweenable + 'static,
) -> Result<(), TweeningError> {
    animator.set_tweenable(tweenable).map(|_| ())
}
