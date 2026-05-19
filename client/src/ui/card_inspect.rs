//! Shared card inspect / zoom presentation primitive.
//!
//! This module owns the reusable enlarged-card shell only. It does not wire
//! hover, hand, shop, auction, or board consumers. Future consumers can spawn
//! the primitive under their own bounded overlay parent and update its marked
//! children by querying the deterministic marker components below.

use bevy::prelude::*;
use bevy::ui::widget::{ImageNode, NodeImageMode};
use bevy::ui::{
    AlignItems, BorderColor, BorderRadius, Display, FlexDirection, GlobalZIndex, JustifyContent,
    Node, Overflow, PositionType, UiRect, Val,
};

use crate::ui::design_tokens::{
    card_slot::{
        card_slot_art_image_component, card_slot_label_strip_background_color,
        CARD_SLOT_ART_IMAGE_MODE,
    },
    spacing, text_fit, z_layers,
};

/// Outer shell width for the inspect card. Fits comfortably in 1280x720 with
/// room for caller-owned chrome around the bounded primitive.
pub const CARD_INSPECT_WIDTH_PX: f32 = 320.0;

/// Outer shell height for the inspect card. Kept below the 720px viewport floor
/// with caller-owned margins.
pub const CARD_INSPECT_HEIGHT_PX: f32 = 520.0;

/// Responsive width ceiling. Prevents cropped side edges when a future caller
/// presents the primitive in a narrow overlay or split panel.
pub const CARD_INSPECT_MAX_WIDTH_PERCENT: f32 = 92.0;

/// Responsive height ceiling. Prevents full-screen assumptions at the 720px
/// viewport floor.
pub const CARD_INSPECT_MAX_HEIGHT_PERCENT: f32 = 92.0;

pub const CARD_INSPECT_BORDER_PX: f32 = 2.0;
pub const CARD_INSPECT_RADIUS_PX: f32 = 8.0;
pub const CARD_INSPECT_ART_HEIGHT_PX: f32 = 270.0;
pub const CARD_INSPECT_TEXT_STRIP_MIN_HEIGHT_PX: f32 = 132.0;
pub const CARD_INSPECT_BADGE_SIZE_PX: f32 = 42.0;
pub const CARD_INSPECT_STAT_BADGE_SIZE_PX: f32 = 38.0;
pub const CARD_INSPECT_TITLE_FONT_PX: f32 = 20.0;
pub const CARD_INSPECT_RULES_FONT_PX: f32 = 14.0;
pub const CARD_INSPECT_BADGE_FONT_PX: f32 = 18.0;
pub const CARD_INSPECT_KEYWORD_FONT_PX: f32 = 12.0;

const CARD_INSPECT_BACKGROUND: Color = Color::srgba(0.075, 0.090, 0.125, 0.98);
const CARD_INSPECT_ART_FALLBACK: Color = Color::srgb(0.145, 0.180, 0.225);
const CARD_INSPECT_BORDER: Color = Color::srgba(0.92, 0.78, 0.42, 0.88);
const CARD_INSPECT_TEXT: Color = Color::srgb(0.94, 0.96, 0.98);
const CARD_INSPECT_MUTED_TEXT: Color = Color::srgb(0.76, 0.82, 0.88);
const CARD_INSPECT_COST: Color = Color::srgb(0.98, 0.78, 0.30);
const CARD_INSPECT_ATTACK: Color = Color::srgb(0.88, 0.36, 0.32);
const CARD_INSPECT_HEALTH: Color = Color::srgb(0.34, 0.82, 0.48);

/// Data passed into [`spawn_card_inspect`] for initial presentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardInspectView {
    pub title: String,
    pub cost: Option<String>,
    pub attack: Option<String>,
    pub health: Option<String>,
    pub keyword: Option<String>,
    pub rules_text: String,
}

impl Default for CardInspectView {
    fn default() -> Self {
        Self {
            title: "Unknown card".to_string(),
            cost: None,
            attack: None,
            health: None,
            keyword: None,
            rules_text: "No card text available.".to_string(),
        }
    }
}

/// Entity ids returned by [`spawn_card_inspect`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardInspectEntities {
    pub root: Entity,
    pub art: Entity,
    pub title: Entity,
    pub cost: Entity,
    pub attack: Entity,
    pub health: Entity,
    pub keyword: Entity,
    pub rules_text: Entity,
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CardInspectRoot;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CardInspectArtArea;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CardInspectTextStrip;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CardInspectTitleText;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CardInspectRulesText;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CardInspectKeywordText;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CardInspectCostBadge;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CardInspectAttackBadge;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CardInspectHealthBadge;

/// Builds the bounded outer shell Node for the inspect card.
pub fn card_inspect_root_node() -> Node {
    Node {
        width: Val::Px(CARD_INSPECT_WIDTH_PX),
        height: Val::Px(CARD_INSPECT_HEIGHT_PX),
        max_width: Val::Percent(CARD_INSPECT_MAX_WIDTH_PERCENT),
        max_height: Val::Percent(CARD_INSPECT_MAX_HEIGHT_PERCENT),
        padding: UiRect::all(Val::Px(spacing::SPACING_MD)),
        border: UiRect::all(Val::Px(CARD_INSPECT_BORDER_PX)),
        row_gap: Val::Px(spacing::SPACING_SM),
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        overflow: Overflow::clip(),
        ..default()
    }
}

pub fn card_inspect_art_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Px(CARD_INSPECT_ART_HEIGHT_PX),
        min_height: Val::Px(CARD_INSPECT_ART_HEIGHT_PX),
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(CARD_INSPECT_RADIUS_PX)),
        overflow: Overflow::clip(),
        position_type: PositionType::Relative,
        ..default()
    }
}

pub fn card_inspect_text_strip_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        min_height: Val::Px(CARD_INSPECT_TEXT_STRIP_MIN_HEIGHT_PX),
        padding: UiRect::all(Val::Px(spacing::SPACING_SM)),
        row_gap: Val::Px(spacing::SPACING_XS),
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        overflow: Overflow::clip(),
        ..default()
    }
}

pub fn card_inspect_badge_node(size_px: f32) -> Node {
    Node {
        width: Val::Px(size_px),
        height: Val::Px(size_px),
        min_width: Val::Px(size_px),
        min_height: Val::Px(size_px),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(size_px * 0.5)),
        ..default()
    }
}

pub fn card_inspect_title_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        min_height: Val::Px(26.0),
        overflow: Overflow::clip_x(),
        ..default()
    }
}

pub fn card_inspect_rules_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        min_height: Val::Px(58.0),
        overflow: Overflow::clip(),
        ..default()
    }
}

pub fn card_inspect_stats_row_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Px(CARD_INSPECT_STAT_BADGE_SIZE_PX),
        column_gap: Val::Px(spacing::SPACING_SM),
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::FlexEnd,
        align_items: AlignItems::Center,
        ..default()
    }
}

pub fn card_inspect_art_image_node() -> ImageNode {
    ImageNode {
        image_mode: CARD_SLOT_ART_IMAGE_MODE,
        ..card_slot_art_image_component()
    }
}

pub fn spawn_card_inspect(
    parent: &mut ChildSpawnerCommands,
    view: CardInspectView,
) -> CardInspectEntities {
    let mut entities = CardInspectEntities {
        root: Entity::PLACEHOLDER,
        art: Entity::PLACEHOLDER,
        title: Entity::PLACEHOLDER,
        cost: Entity::PLACEHOLDER,
        attack: Entity::PLACEHOLDER,
        health: Entity::PLACEHOLDER,
        keyword: Entity::PLACEHOLDER,
        rules_text: Entity::PLACEHOLDER,
    };

    let root = parent
        .spawn((
            CardInspectRoot,
            Name::new("Card Inspect Root"),
            card_inspect_root_node(),
            GlobalZIndex(z_layers::UI_OVERLAY.0),
            BackgroundColor(CARD_INSPECT_BACKGROUND),
            BorderColor::all(CARD_INSPECT_BORDER),
        ))
        .with_children(|root| {
            entities.cost = root
                .spawn((
                    CardInspectCostBadge,
                    Name::new("Card Inspect Cost Badge"),
                    card_inspect_badge_node(CARD_INSPECT_BADGE_SIZE_PX),
                    BackgroundColor(CARD_INSPECT_COST),
                    BorderColor::all(CARD_INSPECT_BORDER),
                    Text::new(view.cost.as_deref().unwrap_or("-")),
                    TextFont {
                        font_size: CARD_INSPECT_BADGE_FONT_PX,
                        ..default()
                    },
                    TextColor(Color::srgb(0.08, 0.06, 0.02)),
                    text_fit::single_line_centered(),
                ))
                .id();

            entities.art = root
                .spawn((
                    CardInspectArtArea,
                    Name::new("Card Inspect Art Area"),
                    card_inspect_art_node(),
                    card_inspect_art_image_node(),
                    BackgroundColor(CARD_INSPECT_ART_FALLBACK),
                    BorderColor::all(Color::srgba(0.92, 0.94, 0.96, 0.28)),
                ))
                .id();

            root.spawn((
                CardInspectTextStrip,
                Name::new("Card Inspect Text Strip"),
                card_inspect_text_strip_node(),
                BackgroundColor(card_slot_label_strip_background_color()),
            ))
            .with_children(|strip| {
                entities.title = strip
                    .spawn((
                        CardInspectTitleText,
                        Name::new("Card Inspect Title Text"),
                        card_inspect_title_node(),
                        Text::new(view.title),
                        TextFont {
                            font_size: CARD_INSPECT_TITLE_FONT_PX,
                            ..default()
                        },
                        TextColor(CARD_INSPECT_TEXT),
                        text_fit::text_layout(text_fit::TextFitPolicy::SingleLineNoWrap),
                    ))
                    .id();

                entities.keyword = strip
                    .spawn((
                        CardInspectKeywordText,
                        Name::new("Card Inspect Keyword Text"),
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(18.0),
                            overflow: Overflow::clip_x(),
                            ..default()
                        },
                        Text::new(view.keyword.as_deref().unwrap_or("")),
                        TextFont {
                            font_size: CARD_INSPECT_KEYWORD_FONT_PX,
                            ..default()
                        },
                        TextColor(CARD_INSPECT_MUTED_TEXT),
                        text_fit::text_layout(text_fit::TextFitPolicy::SingleLineNoWrap),
                    ))
                    .id();

                entities.rules_text = strip
                    .spawn((
                        CardInspectRulesText,
                        Name::new("Card Inspect Rules Text"),
                        card_inspect_rules_node(),
                        Text::new(view.rules_text),
                        TextFont {
                            font_size: CARD_INSPECT_RULES_FONT_PX,
                            ..default()
                        },
                        TextColor(CARD_INSPECT_TEXT),
                        text_fit::wrap_body_left(),
                    ))
                    .id();

                strip
                    .spawn((
                        Name::new("Card Inspect Stat Row"),
                        card_inspect_stats_row_node(),
                    ))
                    .with_children(|stats| {
                        entities.attack = stats
                            .spawn((
                                CardInspectAttackBadge,
                                Name::new("Card Inspect Attack Badge"),
                                card_inspect_badge_node(CARD_INSPECT_STAT_BADGE_SIZE_PX),
                                BackgroundColor(CARD_INSPECT_ATTACK),
                                BorderColor::all(Color::srgba(0.98, 0.92, 0.88, 0.45)),
                                Text::new(view.attack.as_deref().unwrap_or("-")),
                                TextFont {
                                    font_size: CARD_INSPECT_BADGE_FONT_PX,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.94, 0.90)),
                                text_fit::single_line_centered(),
                            ))
                            .id();

                        entities.health = stats
                            .spawn((
                                CardInspectHealthBadge,
                                Name::new("Card Inspect Health Badge"),
                                card_inspect_badge_node(CARD_INSPECT_STAT_BADGE_SIZE_PX),
                                BackgroundColor(CARD_INSPECT_HEALTH),
                                BorderColor::all(Color::srgba(0.88, 1.0, 0.90, 0.45)),
                                Text::new(view.health.as_deref().unwrap_or("-")),
                                TextFont {
                                    font_size: CARD_INSPECT_BADGE_FONT_PX,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.04, 0.10, 0.05)),
                                text_fit::single_line_centered(),
                            ))
                            .id();
                    });
            });
        })
        .id();

    entities.root = root;
    entities
}

/// Defensive check used by tests and future consumers that need to assert the
/// primitive remains bounded at the minimum supported live viewport.
pub const fn card_inspect_fits_1280x720() -> bool {
    CARD_INSPECT_WIDTH_PX <= 1280.0 * (CARD_INSPECT_MAX_WIDTH_PERCENT / 100.0)
        && CARD_INSPECT_HEIGHT_PX <= 720.0 * (CARD_INSPECT_MAX_HEIGHT_PERCENT / 100.0)
}

pub const fn card_inspect_art_image_mode() -> NodeImageMode {
    CARD_SLOT_ART_IMAGE_MODE
}
