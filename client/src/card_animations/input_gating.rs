use std::time::Duration;

use bevy::{ecs::change_detection::Mut, math::curve::EaseFunction, prelude::*};
use bevy_tweening::{
    lens::{Lens, TransformScaleLens, UiPositionLens},
    PlaybackState, Tween, TweenAnim, Tweenable,
};
use shared::protocol::RoundPhase;

use crate::state::ClientPhaseView;

use super::{
    events::{
        HandCardDragStarted, HandCardHoverEntered, HandCardHoverExited, TimerBarEaseRequested,
    },
    make_tween_anim, placement_phase_duration, replace_tweenable, PlacementPhaseAnimator,
};

const TIMER_BAR_EASE_MS: u64 = 150;
const DRAG_LIFT_MS: u64 = 120;
const HAND_CARD_HOVER_MS: u64 = 120;
const HAND_CARD_DEHOVER_MS: u64 = 120;
const HAND_CARD_DEHOVER_MIN_MS: u64 = 40;
const SNAP_BACK_MS: u64 = 220;
const CELL_HIGHLIGHT_MS: u64 = 120;
const REST_SCALE: f32 = 1.0;
const HOVER_SCALE: f32 = 1.12;
const DRAG_LIFT_OFFSET_PX: f32 = -4.0;

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct InputGatingAnimationConfig {
    pub timer_bar_ease_ms: u64,
    pub drag_lift_ms: u64,
    pub hand_card_hover_ms: u64,
    pub hand_card_dehover_ms: u64,
    pub hand_card_dehover_min_ms: u64,
    pub snap_back_duration_ms: u64,
    pub cell_highlight_ms: u64,
    pub rest_scale: f32,
    pub hover_scale: f32,
    pub drag_lift_offset_px: f32,
}

impl Default for InputGatingAnimationConfig {
    fn default() -> Self {
        Self {
            timer_bar_ease_ms: TIMER_BAR_EASE_MS,
            drag_lift_ms: DRAG_LIFT_MS,
            hand_card_hover_ms: HAND_CARD_HOVER_MS,
            hand_card_dehover_ms: HAND_CARD_DEHOVER_MS,
            hand_card_dehover_min_ms: HAND_CARD_DEHOVER_MIN_MS,
            snap_back_duration_ms: SNAP_BACK_MS,
            cell_highlight_ms: CELL_HIGHLIGHT_MS,
            rest_scale: REST_SCALE,
            hover_scale: HOVER_SCALE,
            drag_lift_offset_px: DRAG_LIFT_OFFSET_PX,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TimerBar;

#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HandCard;

#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HandDragSprite;

#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BidPresetButton;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandCardScaleDirection {
    Hovering,
    Returning,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct HandCardScaleAnimation {
    pub direction: HandCardScaleDirection,
}

impl HandCardScaleAnimation {
    pub const fn hovering() -> Self {
        Self {
            direction: HandCardScaleDirection::Hovering,
        }
    }

    pub const fn returning() -> Self {
        Self {
            direction: HandCardScaleDirection::Returning,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NodeWidthPercentLens {
    pub start: f32,
    pub end: f32,
}

impl Lens<Node> for NodeWidthPercentLens {
    fn lerp(&mut self, mut target: Mut<Node>, ratio: f32) {
        let width = self.start + (self.end - self.start) * ratio;
        target.width = Val::Percent(width.clamp(0.0, 100.0));
    }
}

pub fn timer_bar_ease_system(
    mut commands: Commands,
    config: Res<InputGatingAnimationConfig>,
    mut requests: MessageReader<TimerBarEaseRequested>,
    mut timer_bars: Query<(Entity, &Node, Option<&mut TweenAnim>), With<TimerBar>>,
) {
    for request in requests.read() {
        let mut found = false;
        for (entity, node, animator) in &mut timer_bars {
            found = true;
            let tween = timer_bar_tween(
                node_width_percent(node.width),
                request.target_width_percent,
                config.timer_bar_ease_ms,
            );
            install_tween(&mut commands, entity, animator, tween, false);
        }

        if !found {
            warn!("TimerBarEaseRequested received but no TimerBar entity exists");
        }
    }
}

pub fn hand_card_drag_start_system(
    mut commands: Commands,
    config: Res<InputGatingAnimationConfig>,
    phase: Option<Res<ClientPhaseView>>,
    mut requests: MessageReader<HandCardDragStarted>,
    hand_cards: Query<(), With<HandCard>>,
    mut drag_sprites: Query<(Entity, &Node, Option<&mut TweenAnim>), With<HandDragSprite>>,
) {
    if !is_placement_phase(phase.as_deref()) {
        for _ in requests.read() {}
        return;
    }

    for request in requests.read() {
        if hand_cards.get(request.card).is_err() {
            warn!(
                "HandCardDragStarted received for non-hand-card entity {:?}",
                request.card
            );
            continue;
        }

        let Some((drag_sprite, node, animator)) = drag_sprites.iter_mut().next() else {
            warn!("HandCardDragStarted received but no HandDragSprite entity exists");
            continue;
        };

        tracing::info!(
            target: "drag_lift_tween_install",
            request_card = ?request.card,
            drag_sprite_entity = ?drag_sprite,
            node_left = ?node.left,
            node_top = ?node.top,
            tween_offset_px = config.drag_lift_offset_px,
            tween_ms = config.drag_lift_ms,
            existing_animator = animator.is_some(),
            "drag lift tween install"
        );
        let tween = drag_lift_tween(node, config.drag_lift_ms, config.drag_lift_offset_px);
        install_tween(&mut commands, drag_sprite, animator, tween, true);
    }
}

pub fn hand_card_hover_exit_system(
    mut commands: Commands,
    config: Res<InputGatingAnimationConfig>,
    phase: Option<Res<ClientPhaseView>>,
    mut requests: MessageReader<HandCardHoverExited>,
    mut hand_cards: Query<(Entity, &Transform, Option<&mut TweenAnim>), With<HandCard>>,
) {
    let placement_phase = is_placement_phase(phase.as_deref());

    for request in requests.read() {
        let Ok((entity, transform, animator)) = hand_cards.get_mut(request.card) else {
            warn!(
                "HandCardHoverExited received for non-hand-card entity {:?}",
                request.card
            );
            continue;
        };

        let tween = hand_card_return_tween(transform.scale, &config);
        install_hand_card_scale_tween(
            &mut commands,
            entity,
            animator,
            tween,
            HandCardScaleAnimation::returning(),
            placement_phase,
        );
    }
}

pub fn hand_card_hover_enter_system(
    mut commands: Commands,
    config: Res<InputGatingAnimationConfig>,
    phase: Option<Res<ClientPhaseView>>,
    mut requests: MessageReader<HandCardHoverEntered>,
    mut hand_cards: Query<
        (
            Entity,
            &Transform,
            Option<&mut TweenAnim>,
            Option<&HandCardScaleAnimation>,
        ),
        With<HandCard>,
    >,
) {
    let placement_phase = is_placement_phase(phase.as_deref());

    for request in requests.read() {
        if hand_cards.get(request.card).is_err() {
            warn!(
                "HandCardHoverEntered received for non-hand-card entity {:?}",
                request.card
            );
            continue;
        }

        for (entity, transform, animator, scale_animation) in &mut hand_cards {
            if entity == request.card {
                let tween = hand_card_hover_tween(transform.scale, &config);
                install_hand_card_scale_tween(
                    &mut commands,
                    entity,
                    animator,
                    tween,
                    HandCardScaleAnimation::hovering(),
                    placement_phase,
                );
                continue;
            }

            if should_keep_or_start_return(transform, scale_animation) {
                let tween = hand_card_return_tween(transform.scale, &config);
                install_hand_card_scale_tween(
                    &mut commands,
                    entity,
                    animator,
                    tween,
                    HandCardScaleAnimation::returning(),
                    placement_phase,
                );
            }
        }
    }
}

fn timer_bar_tween(start_width_percent: f32, target_width_percent: f32, duration_ms: u64) -> Tween {
    Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(duration_ms.min(TIMER_BAR_EASE_MS)),
        NodeWidthPercentLens {
            start: start_width_percent.clamp(0.0, 100.0),
            end: target_width_percent.clamp(0.0, 100.0),
        },
    )
}

fn drag_lift_tween(node: &Node, duration_ms: u64, offset_px: f32) -> Tween {
    let start = node_position(node);
    let mut end = start;
    end.top = offset_val_px(end.top, offset_px);

    Tween::new(
        EaseFunction::QuadraticOut,
        placement_phase_duration(duration_ms),
        UiPositionLens { start, end },
    )
}

fn hand_card_hover_tween(start: Vec3, config: &InputGatingAnimationConfig) -> Tween {
    let target = Vec3::splat(config.hover_scale);
    Tween::new(
        EaseFunction::QuadraticOut,
        placement_phase_duration(config.hand_card_hover_ms),
        TransformScaleLens { start, end: target },
    )
}

fn hand_card_return_tween(start: Vec3, config: &InputGatingAnimationConfig) -> Tween {
    let target = Vec3::splat(config.rest_scale);
    let duration_ms = config
        .hand_card_dehover_ms
        .max(config.hand_card_dehover_min_ms)
        .min(super::PLACEMENT_ANIMATION_CAP_MS);
    Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(duration_ms),
        TransformScaleLens { start, end: target },
    )
}

fn install_hand_card_scale_tween(
    commands: &mut Commands,
    entity: Entity,
    animator: Option<Mut<TweenAnim>>,
    tween: Tween,
    scale_animation: HandCardScaleAnimation,
    placement_phase: bool,
) {
    install_tween(commands, entity, animator, tween, placement_phase);
    commands.entity(entity).insert(scale_animation);
}

fn install_tween(
    commands: &mut Commands,
    entity: Entity,
    animator: Option<Mut<TweenAnim>>,
    tweenable: impl Tweenable + 'static,
    placement_phase: bool,
) {
    if let Some(mut animator) = animator {
        if let Err(error) = replace_tweenable(&mut animator, tweenable) {
            warn!("Failed to replace tween on entity {entity:?}: {error}");
            return;
        }
        animator.playback_state = PlaybackState::Playing;
        animator.destroy_on_completion = false;
    } else if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.insert(make_tween_anim(tweenable));
    } else {
        warn!("Animation target entity {entity:?} no longer exists");
        return;
    }

    if placement_phase {
        commands.entity(entity).insert(PlacementPhaseAnimator);
    }
}

fn should_keep_or_start_return(
    transform: &Transform,
    scale_animation: Option<&HandCardScaleAnimation>,
) -> bool {
    transform.scale.x > REST_SCALE
        || matches!(
            scale_animation.map(|animation| animation.direction),
            Some(HandCardScaleDirection::Hovering | HandCardScaleDirection::Returning)
        )
}

fn node_position(node: &Node) -> UiRect {
    UiRect {
        left: node.left,
        right: node.right,
        top: node.top,
        bottom: node.bottom,
    }
}

fn node_width_percent(width: Val) -> f32 {
    match width {
        Val::Percent(width) => width,
        _ => 100.0,
    }
}

fn offset_val_px(value: Val, offset_px: f32) -> Val {
    match value {
        Val::Px(value) => Val::Px(value + offset_px),
        _ => value,
    }
}

fn is_placement_phase(phase: Option<&ClientPhaseView>) -> bool {
    matches!(phase.map(|phase| phase.phase), Some(RoundPhase::Placement))
}
