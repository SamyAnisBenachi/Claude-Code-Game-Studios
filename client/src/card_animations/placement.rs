use bevy::{ecs::change_detection::Mut, math::curve::EaseFunction, prelude::*};
use bevy_tweening::{
    lens::UiPositionLens, AnimTarget, AnimTargetKind, PlaybackState, Tween, TweenAnim, Tweenable,
};

use crate::ui::shared::{BoardLayout, LaneCell};

use super::{
    cancel_tween_anim_in_place,
    events::{
        CellHighlightRequested, PlacementCancelAllAnimsRequested, PlacementRevealAnimReady,
        SnapBackRequested,
    },
    make_tween_anim, placement_phase_duration, replace_tweenable, InputGatingAnimationConfig,
    PlacementPhaseAnimator, SpriteColorLens, TransformScaleXLens,
};

const PLACEMENT_REVEAL_MS: u64 = 90;
const CELL_HIGHLIGHT_COLOR: Color = Color::srgba(0.88, 0.98, 1.0, 0.85);

pub fn placement_reveal_system(
    mut commands: Commands,
    board_layout: Option<Res<BoardLayout>>,
    mut requests: MessageReader<PlacementRevealAnimReady>,
    mut units: Query<(Entity, &mut Transform, Option<&mut TweenAnim>)>,
) {
    for request in requests.read() {
        for entry in &request.entries {
            let Ok((entity, mut transform, animator)) = units.get_mut(entry.unit) else {
                warn!(
                    "PlacementRevealAnimReady entry for missing unit {:?} ignored",
                    entry.unit
                );
                continue;
            };

            let end_scale_x = transform.scale.x;
            let tween = Tween::new(
                EaseFunction::QuadraticOut,
                placement_phase_duration(PLACEMENT_REVEAL_MS),
                TransformScaleXLens {
                    start: 0.0,
                    end: end_scale_x,
                },
            );

            install_tween(&mut commands, entity, animator, tween);
            snap_transform_xy_to_entry(&board_layout, &mut transform, entry.lane, entry.cell);
        }
    }
}

pub fn placement_snap_back_system(
    mut commands: Commands,
    config: Res<InputGatingAnimationConfig>,
    mut requests: MessageReader<SnapBackRequested>,
    mut targets: Query<(Entity, &Node, Option<&mut TweenAnim>)>,
) {
    for request in requests.read() {
        let Ok((entity, node, animator)) = targets.get_mut(request.target) else {
            warn!(
                "SnapBackRequested received for missing Node target {:?}",
                request.target
            );
            continue;
        };

        let tween = Tween::new(
            EaseFunction::QuadraticOut,
            placement_phase_duration(config.snap_back_duration_ms),
            UiPositionLens {
                start: node_position(node),
                end: request.end_position,
            },
        );

        install_placement_tween(&mut commands, entity, animator, tween);
    }
}

pub fn placement_cell_highlight_system(
    mut commands: Commands,
    config: Res<InputGatingAnimationConfig>,
    mut requests: MessageReader<CellHighlightRequested>,
    mut cells: Query<(Entity, &Sprite, Option<&mut TweenAnim>)>,
) {
    for request in requests.read() {
        let Ok((entity, sprite, animator)) = cells.get_mut(request.target) else {
            warn!(
                "CellHighlightRequested received for missing Sprite target {:?}",
                request.target
            );
            continue;
        };

        let tween = Tween::new(
            EaseFunction::QuadraticOut,
            placement_phase_duration(config.cell_highlight_ms),
            SpriteColorLens {
                start: sprite.color,
                end: CELL_HIGHLIGHT_COLOR,
            },
        );

        install_placement_tween(&mut commands, entity, animator, tween);
    }
}

pub fn placement_cancel_all_anims_system(
    board_layout: Option<Res<BoardLayout>>,
    mut requests: MessageReader<PlacementCancelAllAnimsRequested>,
    mut animators: Query<
        (Entity, &mut TweenAnim, Option<&AnimTarget>),
        With<PlacementPhaseAnimator>,
    >,
    mut targets: Query<(&LaneCell, &mut Transform)>,
) {
    let mut requested = false;
    for _request in requests.read() {
        requested = true;
    }
    if !requested {
        return;
    }

    for (controller_entity, mut animator, anim_target) in &mut animators {
        let target_entity = animation_target_entity(controller_entity, anim_target);

        if let Err(error) = cancel_tween_anim_in_place(&mut animator) {
            warn!("Failed to cancel PLACEMENT tween on entity {controller_entity:?}: {error}");
        }
        animator.playback_state = PlaybackState::Paused;

        snap_target_to_committed_cell(&board_layout, target_entity, &mut targets);
    }
}

fn install_placement_tween(
    commands: &mut Commands,
    entity: Entity,
    animator: Option<Mut<TweenAnim>>,
    tweenable: impl Tweenable + 'static,
) {
    if install_tween(commands, entity, animator, tweenable) {
        commands.entity(entity).insert(PlacementPhaseAnimator);
    }
}

fn install_tween(
    commands: &mut Commands,
    entity: Entity,
    animator: Option<Mut<TweenAnim>>,
    tweenable: impl Tweenable + 'static,
) -> bool {
    if let Some(mut animator) = animator {
        if let Err(error) = replace_tweenable(&mut animator, tweenable) {
            warn!("Failed to replace tween on entity {entity:?}: {error}");
            return false;
        }
        animator.playback_state = PlaybackState::Playing;
        animator.destroy_on_completion = false;
    } else if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.insert(make_tween_anim(tweenable));
    } else {
        warn!("Animation target entity {entity:?} no longer exists");
        return false;
    }

    true
}

fn snap_transform_xy_to_entry(
    board_layout: &Option<Res<BoardLayout>>,
    transform: &mut Transform,
    lane: u8,
    cell: u8,
) {
    let Some(board_layout) = board_layout.as_deref() else {
        warn!("BoardLayout missing; placement reveal snap skipped");
        return;
    };
    let world_xy = board_layout.cell_to_world(lane, cell);

    transform.translation.x = world_xy.x;
    transform.translation.y = world_xy.y;
}

fn snap_target_to_committed_cell(
    board_layout: &Option<Res<BoardLayout>>,
    target_entity: Entity,
    targets: &mut Query<(&LaneCell, &mut Transform)>,
) {
    let Some(board_layout) = board_layout.as_deref() else {
        warn!("BoardLayout missing; PLACEMENT cancellation snap skipped");
        return;
    };
    let Ok((lane_cell, mut transform)) = targets.get_mut(target_entity) else {
        warn!(
            "PLACEMENT cancellation target {:?} has no LaneCell/Transform; snap skipped",
            target_entity
        );
        return;
    };
    let world_xy = board_layout.cell_to_world(lane_cell.lane, lane_cell.cell);

    transform.translation.x = world_xy.x;
    transform.translation.y = world_xy.y;
}

fn animation_target_entity(controller_entity: Entity, anim_target: Option<&AnimTarget>) -> Entity {
    match anim_target.map(|target| target.kind) {
        Some(AnimTargetKind::Component { entity }) => entity,
        _ => controller_entity,
    }
}

fn node_position(node: &Node) -> UiRect {
    UiRect {
        left: node.left,
        right: node.right,
        top: node.top,
        bottom: node.bottom,
    }
}
