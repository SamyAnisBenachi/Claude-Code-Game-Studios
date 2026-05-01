pub mod animators;
pub mod events;
pub mod lenses;
pub mod queue;

use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;

pub use animators::{make_tween_anim, replace_tweenable};
pub use events::*;
pub use lenses::{
    BackgroundColorAlphaLens, SpriteAlphaLens, SpriteColorLens, TextColorLens, TransformScaleXLens,
};
pub use queue::StagedObjectiveRevealQueue;

pub struct CardAnimationsPlugin;

impl Plugin for CardAnimationsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<TweeningPlugin>() {
            app.add_plugins(TweeningPlugin);
        }

        app.init_resource::<StagedObjectiveRevealQueue>()
            .add_message::<PlacementRevealAnimReady>()
            .add_message::<ObjectiveDestroyedAnimReady>()
            .add_message::<DamageNumberSpawnRequested>()
            .add_message::<BoardRebuildRequested>()
            .add_message::<PlacementCancelAllAnimsRequested>()
            .add_message::<CardAcquiredAnimReady>()
            .add_message::<SnapBackRequested>()
            .add_message::<HandHideRequested>()
            .add_message::<HandShowRequested>()
            .add_message::<AuctionPanelTransitionRequested>()
            .add_message::<TimerBarEaseRequested>()
            .add_message::<GoldTickRequested>()
            .add_message::<SettlementOverlayRequested>()
            .add_message::<DisplacementAnimRequested>()
            .add_message::<TrapFlipRequested>()
            .add_message::<AuraPulseRequested>()
            .add_message::<GroupDrainedSignal>();
    }
}
