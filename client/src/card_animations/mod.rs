pub mod animators;
pub mod events;
pub mod input_gating;
pub mod lenses;
pub mod queue;

use bevy::prelude::*;
use bevy_tweening::{AnimationSystem, TweenAnim, TweeningPlugin};

pub use animators::{
    cancel_tween_anim_in_place, make_tween_anim, replace_tweenable, spawn_sprite_tween_controller,
    spawn_transform_tween_controller, PlacementPhaseAnimator,
};
pub use events::*;
pub use input_gating::{
    hand_card_drag_start_system, hand_card_hover_enter_system, hand_card_hover_exit_system,
    timer_bar_ease_system, BidPresetButton, HandCard, HandCardScaleAnimation,
    HandCardScaleDirection, HandDragSprite, InputGatingAnimationConfig, NodeWidthPercentLens,
    TimerBar,
};
pub use lenses::{
    BackgroundColorAlphaLens, SpriteAlphaLens, SpriteColorLens, TextColorLens, TransformScaleXLens,
    TransformTranslationXLens, TransformTranslationYLens,
};
pub use queue::{
    resolution_executing_system, resolution_objective_reveal_system, AnimGroup, AnimQueue,
    AnimQueueEvent, AnimationTimingConfig, PendingObjectiveDestroyedEvent,
    PendingObjectiveDestroyedEvents, PendingPhaseChange, StagedObjectiveReveal,
    StagedObjectiveRevealQueue,
};

pub struct CardAnimationsPlugin;

impl Plugin for CardAnimationsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<TweeningPlugin>() {
            app.add_plugins(TweeningPlugin);
        }

        app.init_resource::<AnimationTimingConfig>()
            .init_resource::<InputGatingAnimationConfig>()
            .init_resource::<AnimQueue>()
            .init_resource::<PendingPhaseChange>()
            .init_resource::<PendingObjectiveDestroyedEvents>()
            .init_resource::<StagedObjectiveRevealQueue>()
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
            .add_message::<HandCardDragStarted>()
            .add_message::<HandCardHoverEntered>()
            .add_message::<HandCardHoverExited>()
            .add_message::<GoldTickRequested>()
            .add_message::<SettlementOverlayRequested>()
            .add_message::<DisplacementAnimRequested>()
            .add_message::<TrapFlipRequested>()
            .add_message::<AuraPulseRequested>()
            .add_message::<GroupDrainedSignal>()
            .add_systems(
                Update,
                (
                    (
                        timer_bar_ease_system,
                        hand_card_drag_start_system,
                        hand_card_hover_exit_system,
                        hand_card_hover_enter_system,
                    )
                        .chain()
                        .before(AnimationSystem::AnimationUpdate),
                    cancel_board_rebuild_tweens
                        .before(resolution_executing_system)
                        .before(AnimationSystem::AnimationUpdate),
                    resolution_executing_system.before(AnimationSystem::AnimationUpdate),
                    resolution_objective_reveal_system
                        .after(resolution_executing_system)
                        .after(AnimationSystem::AnimationUpdate),
                ),
            );
    }
}

fn cancel_board_rebuild_tweens(
    mut rebuilds: MessageReader<BoardRebuildRequested>,
    mut animators: Query<&mut TweenAnim>,
    mut queue: ResMut<AnimQueue>,
    mut staged_objectives: ResMut<StagedObjectiveRevealQueue>,
    mut pending_objectives: ResMut<PendingObjectiveDestroyedEvents>,
) {
    if rebuilds.read().next().is_none() {
        return;
    }

    queue.reset();
    staged_objectives.clear();
    pending_objectives.clear();

    for mut animator in &mut animators {
        if let Err(error) = cancel_tween_anim_in_place(&mut animator) {
            warn!("Failed to cancel tween during board rebuild: {error}");
        }
    }
}
