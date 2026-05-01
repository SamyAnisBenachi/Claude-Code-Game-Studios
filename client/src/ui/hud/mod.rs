use bevy::prelude::*;

use crate::state::ClientState;

pub const HUD_DOT_ROWS: usize = 2;
pub const HUD_DOTS_PER_ROW: usize = 5;
pub const HUD_ENTITY_COUNT: usize = 18;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct HudConfig {
    pub hud_margin_px: f32,
    pub hud_dot_diameter_px: f32,
    pub hud_tween_duration_ms: u32,
}

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            hud_margin_px: 12.0,
            hud_dot_diameter_px: 16.0,
            hud_tween_duration_ms: 300,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct HudEntities {
    pub root: Entity,
    pub phase_label: Entity,
    pub round_counter: Entity,
    pub own_gold_parent: Entity,
    pub own_gold_span: Entity,
    pub opponent_gold_parent: Entity,
    pub opponent_gold_span: Entity,
    pub mana_label: Entity,
    pub reserve_label: Entity,
    pub dots: [[Entity; HUD_DOTS_PER_ROW]; HUD_DOT_ROWS],
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudEntity;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseLabel;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoundCounter;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManaLabel;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReserveManaLabel;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoldLabelOwner {
    Local,
    Opponent,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GoldDisplayState {
    pub gold: f32,
    pub reserved_gold: f32,
    pub is_populated: bool,
}

impl Default for GoldDisplayState {
    fn default() -> Self {
        Self {
            gold: 0.0,
            reserved_gold: 0.0,
            is_populated: false,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreboardDot {
    pub row: ScoreboardRow,
    pub lane_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreboardRow {
    Opponent,
    Local,
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreboardDotState {
    pub destroyed: bool,
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ClientState>()
            .init_resource::<HudConfig>()
            .add_systems(OnEnter(ClientState::InSession), spawn_hud)
            .add_systems(OnExit(ClientState::InSession), despawn_hud);
    }
}

fn spawn_hud(mut commands: Commands, config: Res<HudConfig>, existing: Option<Res<HudEntities>>) {
    if existing.is_some() {
        return;
    }

    let root = commands
        .spawn((
            Name::new("HUD Root"),
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                ..default()
            },
            Visibility::Hidden,
        ))
        .id();

    #[cfg(feature = "ui_picking")]
    commands.entity(root).insert(Pickable {
        should_block_lower: false,
        is_hoverable: false,
    });

    let phase_label = spawn_text_label(
        &mut commands,
        root,
        "HUD Phase Label",
        "",
        PhaseLabel,
        top_left_node(config.hud_margin_px),
    );
    let round_counter = spawn_text_label(
        &mut commands,
        root,
        "HUD Round Counter",
        "",
        RoundCounter,
        top_left_second_line_node(config.hud_margin_px),
    );
    let (own_gold_parent, own_gold_span) = spawn_gold_label(
        &mut commands,
        root,
        "HUD Own Gold",
        GoldLabelOwner::Local,
        config.hud_margin_px,
        0.0,
    );
    let (opponent_gold_parent, opponent_gold_span) = spawn_gold_label(
        &mut commands,
        root,
        "HUD Opponent Gold",
        GoldLabelOwner::Opponent,
        config.hud_margin_px,
        22.0,
    );
    let mana_label = spawn_text_label(
        &mut commands,
        root,
        "HUD Mana Label",
        "",
        ManaLabel,
        bottom_left_node(config.hud_margin_px, 0.0),
    );
    let reserve_label = spawn_text_label(
        &mut commands,
        root,
        "HUD Reserve Mana Label",
        "",
        ReserveManaLabel,
        bottom_left_node(config.hud_margin_px, 22.0),
    );
    let dots = spawn_scoreboard_dots(&mut commands, root, &config);

    commands.insert_resource(HudEntities {
        root,
        phase_label,
        round_counter,
        own_gold_parent,
        own_gold_span,
        opponent_gold_parent,
        opponent_gold_span,
        mana_label,
        reserve_label,
        dots,
    });
}

fn despawn_hud(mut commands: Commands, entities: Option<Res<HudEntities>>) {
    if let Some(entities) = entities {
        commands.entity(entities.root).despawn();
        commands.remove_resource::<HudEntities>();
    }
}

fn spawn_text_label<M: Component>(
    commands: &mut Commands,
    parent: Entity,
    name: &'static str,
    text: &'static str,
    marker: M,
    node: Node,
) -> Entity {
    commands
        .spawn((
            Name::new(name),
            HudEntity,
            marker,
            Text::new(text),
            hud_text_font(18.0),
            TextColor(Color::srgb(0.92, 0.94, 0.96)),
            node,
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_gold_label(
    commands: &mut Commands,
    parent: Entity,
    name: &'static str,
    owner: GoldLabelOwner,
    margin_px: f32,
    top_offset_px: f32,
) -> (Entity, Entity) {
    let parent_entity = commands
        .spawn((
            Name::new(name),
            HudEntity,
            owner,
            GoldDisplayState::default(),
            Text::new("--g"),
            hud_text_font(18.0),
            TextColor(Color::srgb(0.95, 0.90, 0.70)),
            top_right_node(margin_px, top_offset_px),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();
    let span_entity = commands
        .spawn((
            Name::new(format!("{name} Reserved Span")),
            HudEntity,
            TextSpan::new(""),
            hud_text_font(12.0),
            TextColor(Color::srgba(0.95, 0.90, 0.70, 0.65)),
            Visibility::Hidden,
            ChildOf(parent_entity),
        ))
        .id();

    (parent_entity, span_entity)
}

fn spawn_scoreboard_dots(
    commands: &mut Commands,
    parent: Entity,
    config: &HudConfig,
) -> [[Entity; HUD_DOTS_PER_ROW]; HUD_DOT_ROWS] {
    std::array::from_fn(|row| {
        std::array::from_fn(|lane_index| {
            let row_marker = match row {
                0 => ScoreboardRow::Opponent,
                _ => ScoreboardRow::Local,
            };

            commands
                .spawn((
                    Name::new(format!(
                        "HUD {:?} Scoreboard Dot {}",
                        row_marker,
                        lane_index + 1
                    )),
                    HudEntity,
                    ScoreboardDot {
                        row: row_marker,
                        lane_index,
                    },
                    ScoreboardDotState::default(),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(config.hud_margin_px + row as f32 * 20.0),
                        left: Val::Percent(42.0 + lane_index as f32 * 4.0),
                        width: Val::Px(config.hud_dot_diameter_px),
                        height: Val::Px(config.hud_dot_diameter_px),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(config.hud_dot_diameter_px * 0.5)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.84, 0.88, 0.92, 0.88)),
                    BorderColor(Color::srgba(0.96, 0.98, 1.0, 0.95)),
                    Visibility::Hidden,
                    ChildOf(parent),
                ))
                .id()
        })
    })
}

fn hud_text_font(font_size: f32) -> TextFont {
    TextFont {
        font_size,
        ..default()
    }
}

fn top_left_node(margin_px: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(margin_px),
        top: Val::Px(margin_px),
        ..default()
    }
}

fn top_left_second_line_node(margin_px: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(margin_px),
        top: Val::Px(margin_px + 22.0),
        ..default()
    }
}

fn top_right_node(margin_px: f32, top_offset_px: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(margin_px),
        top: Val::Px(margin_px + top_offset_px),
        ..default()
    }
}

fn bottom_left_node(margin_px: f32, bottom_offset_px: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(margin_px),
        bottom: Val::Px(margin_px + bottom_offset_px),
        ..default()
    }
}
