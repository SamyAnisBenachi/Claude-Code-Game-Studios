use bevy::color::Alpha;
use bevy::ecs::change_detection::Mut;
use bevy::prelude::*;
use bevy_tweening::lens::Lens;

#[derive(Clone, Debug)]
pub struct SpriteAlphaLens {
    pub start: f32,
    pub end: f32,
}

impl Lens<Sprite> for SpriteAlphaLens {
    fn lerp(&mut self, mut target: Mut<Sprite>, ratio: f32) {
        let alpha = lerp_f32(self.start, self.end, ratio).clamp(0.0, 1.0);
        target.color = target.color.with_alpha(alpha);
    }
}

#[derive(Clone, Debug)]
pub struct BackgroundColorAlphaLens {
    pub start: f32,
    pub end: f32,
}

impl Lens<BackgroundColor> for BackgroundColorAlphaLens {
    fn lerp(&mut self, mut target: Mut<BackgroundColor>, ratio: f32) {
        let alpha = lerp_f32(self.start, self.end, ratio).clamp(0.0, 1.0);
        target.0 = target.0.with_alpha(alpha);
    }
}

#[derive(Clone, Debug)]
pub struct SpriteColorLens {
    pub start: Color,
    pub end: Color,
}

impl Lens<Sprite> for SpriteColorLens {
    fn lerp(&mut self, mut target: Mut<Sprite>, ratio: f32) {
        target.color = lerp_color(self.start, self.end, ratio);
    }
}

#[derive(Clone, Debug)]
pub struct TransformScaleXLens {
    pub start: f32,
    pub end: f32,
}

impl Lens<Transform> for TransformScaleXLens {
    fn lerp(&mut self, mut target: Mut<Transform>, ratio: f32) {
        target.scale.x = lerp_f32(self.start, self.end, ratio).max(0.0);
    }
}

#[derive(Clone, Debug)]
pub struct TransformTranslationXLens {
    pub start: f32,
    pub end: f32,
}

impl Lens<Transform> for TransformTranslationXLens {
    fn lerp(&mut self, mut target: Mut<Transform>, ratio: f32) {
        target.translation.x = lerp_f32(self.start, self.end, ratio);
    }
}

#[derive(Clone, Debug)]
pub struct TransformTranslationYLens {
    pub start: f32,
    pub end: f32,
}

impl Lens<Transform> for TransformTranslationYLens {
    fn lerp(&mut self, mut target: Mut<Transform>, ratio: f32) {
        target.translation.y = lerp_f32(self.start, self.end, ratio);
    }
}

#[derive(Clone, Debug)]
pub struct TextColorLens {
    pub start: Color,
    pub end: Color,
}

impl Lens<TextColor> for TextColorLens {
    fn lerp(&mut self, mut target: Mut<TextColor>, ratio: f32) {
        target.0 = lerp_color(self.start, self.end, ratio);
    }
}

fn lerp_f32(start: f32, end: f32, ratio: f32) -> f32 {
    start + (end - start) * ratio
}

fn lerp_color(start: Color, end: Color, ratio: f32) -> Color {
    let start = start.to_srgba();
    let end = end.to_srgba();

    Color::srgba(
        lerp_f32(start.red, end.red, ratio),
        lerp_f32(start.green, end.green, ratio),
        lerp_f32(start.blue, end.blue, ratio),
        lerp_f32(start.alpha, end.alpha, ratio).clamp(0.0, 1.0),
    )
}
