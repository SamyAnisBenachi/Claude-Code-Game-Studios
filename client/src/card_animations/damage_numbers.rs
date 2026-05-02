use bevy::{math::curve::EaseFunction, prelude::*, text::LineHeight};
use bevy_tweening::{lens::TransformPositionLens, AnimTarget, Tween};

use super::{
    events::DamageNumberSpawnRequested, make_tween_anim, AnimationTimingConfig, TextColorLens,
};

const DAMAGE_NUMBER_FLOAT_OFFSET_PX: f32 = 60.0;
const DAMAGE_NUMBER_FONT_SIZE: f32 = 24.0;
const DAMAGE_NUMBER_LINE_HEIGHT: f32 = 1.0;
const DAMAGE_NUMBER_COLOR: Color = Color::srgba(1.0, 0.22, 0.12, 1.0);
const DAMAGE_NUMBER_FADE_COLOR: Color = Color::srgba(1.0, 0.22, 0.12, 0.0);

pub const DAMAGE_NUMBER_JITTER_TABLE: [Vec2; 8] = [
    Vec2::new(0.0, 0.0),
    Vec2::new(14.0, 6.0),
    Vec2::new(-14.0, 6.0),
    Vec2::new(8.0, 18.0),
    Vec2::new(-8.0, 18.0),
    Vec2::new(20.0, -2.0),
    Vec2::new(-20.0, -2.0),
    Vec2::new(0.0, 24.0),
];

#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DamageNumber;

#[derive(Component, Debug)]
pub struct DespawnAfter(pub Timer);

pub fn spawn_damage_numbers(
    mut commands: Commands,
    timings: Res<AnimationTimingConfig>,
    mut requests: MessageReader<DamageNumberSpawnRequested>,
    targets: Query<&Transform>,
) {
    let timings = *timings;
    timings.assert_damage_number_budget();

    for request in requests.read() {
        let Ok(target_transform) = targets.get(request.target) else {
            warn!(
                "DamageNumberSpawnRequested received for missing target entity {:?}",
                request.target
            );
            continue;
        };

        let origin = damage_number_origin(target_transform, request.event_id);
        let float_end = origin + Vec3::new(0.0, DAMAGE_NUMBER_FLOAT_OFFSET_PX, 0.0);
        let despawn_after = DespawnAfter(Timer::new(
            timings.damage_number_despawn_delay(),
            TimerMode::Once,
        ));
        let damage_number = commands
            .spawn((
                Text2d::new(request.damage_value.to_string()),
                TextFont {
                    font_size: DAMAGE_NUMBER_FONT_SIZE,
                    ..default()
                },
                TextColor(DAMAGE_NUMBER_COLOR),
                LineHeight::RelativeToFont(DAMAGE_NUMBER_LINE_HEIGHT),
                Transform::from_translation(origin),
                DamageNumber,
                despawn_after,
            ))
            .id();

        commands.spawn((
            make_tween_anim(Tween::new(
                EaseFunction::CubicOut,
                timings.damage_number_float_duration(),
                TransformPositionLens {
                    start: origin,
                    end: float_end,
                },
            )),
            AnimTarget::component::<Transform>(damage_number),
            ChildOf(damage_number),
        ));

        commands.spawn((
            make_tween_anim(Tween::new(
                EaseFunction::CubicOut,
                timings.damage_number_fade_duration(),
                TextColorLens {
                    start: DAMAGE_NUMBER_COLOR,
                    end: DAMAGE_NUMBER_FADE_COLOR,
                },
            )),
            AnimTarget::component::<TextColor>(damage_number),
            ChildOf(damage_number),
        ));
    }
}

pub fn despawn_damage_numbers_after_timer(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut damage_numbers: Query<(Entity, &mut DespawnAfter), With<DamageNumber>>,
) {
    for (entity, mut despawn_after) in &mut damage_numbers {
        despawn_after.0.tick(time.delta());
        if despawn_after.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn damage_number_jitter(event_id: u32) -> Vec2 {
    let index = event_id as usize % DAMAGE_NUMBER_JITTER_TABLE.len();
    DAMAGE_NUMBER_JITTER_TABLE[index]
}

fn damage_number_origin(target_transform: &Transform, event_id: u32) -> Vec3 {
    let base = target_transform.translation;
    let jitter = damage_number_jitter(event_id);

    Vec3::new(base.x + jitter.x, base.y + jitter.y, base.z)
}
