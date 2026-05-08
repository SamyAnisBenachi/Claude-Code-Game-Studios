use std::time::Duration;

use bevy::ecs::change_detection::Mut;
use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy_tweening::{
    lens::Lens, AnimationSystem, PlaybackState, Tween, TweenAnim, TweenState, TweeningPlugin,
};
use lightyear::prelude::MessageReceiver;
use shared::protocol::{
    OpponentObjectiveSnapshot, PlayerSnapshot, RoundPhase, S2CGameSnapshot, S2CGoldBroadcast,
};
use shared::session::PlayerId;

use crate::asset_wiring::{
    hud_figurine_asset, hud_objective_dot_asset, ObjectiveDotState, PlaceholderAssets,
    HUD_OBJECTIVE_DOT_DESTROYED_ASSET, HUD_PHASE_TIMER_BAR_ASSET,
};
use crate::card_animations::cancel_tween_anim_in_place;
use crate::presentation::{PlayerEconomyView, PresentationGameSnapshotMessage};
use crate::state::{ClientState, CurrentClientPhase};
use crate::ui::shared::{BoardLayout, HudObjectiveUpdate};

pub const HUD_DOT_ROWS: usize = 2;
pub const HUD_DOTS_PER_ROW: usize = 5;
/// Total HUD entities carrying the `HudEntity` marker (PAW-004: +2 for figurine + timer bar).
pub const HUD_ENTITY_COUNT: usize = 21;
pub const CURRENT_MANA_BAR_WIDTH_PX: f32 = 104.0;
pub const CURRENT_MANA_BAR_HEIGHT_PX: f32 = 28.0;
pub const RESERVE_MANA_DIAMOND_SIZE_PX: f32 = 74.0;
pub const RESERVE_MANA_DIAMOND_ROTATION_DEGREES: f32 = 45.0;
pub const HUD_RESOURCE_TEXT_MIN_SIZE_PX: f32 = 20.0;
pub const HUD_GOLD_TEXT_MIN_SIZE_PX: f32 = 40.0;
pub const HUD_GOLD_FONT_SIZE_PX: f32 = HUD_GOLD_TEXT_MIN_SIZE_PX;
pub const HUD_RESERVED_GOLD_FONT_SIZE_PX: f32 = 26.0;
pub const HUD_SECONDARY_FONT_SIZE_PX: f32 = HUD_RESOURCE_TEXT_MIN_SIZE_PX;
pub const HUD_TEXT_BACKGROUND_COLOR: Color = Color::srgba(0.04, 0.07, 0.12, 1.0);
pub const HUD_PRIMARY_TEXT_COLOR: Color = Color::srgba(0.96, 0.98, 1.0, 1.0);
pub const HUD_GOLD_TEXT_COLOR: Color = Color::srgba(1.0, 0.82, 0.28, 1.0);
pub const HUD_RESERVED_GOLD_TEXT_COLOR: Color = Color::srgba(0.95, 0.90, 0.70, 0.65);
const HUD_GOLD_ROW_GAP_PX: f32 = 48.0;
const HUD_SECONDARY_ROW_GAP_PX: f32 = 28.0;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HudSystemSet {
    PhaseTransition,
    MessageDrain,
    StateSync,
}

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

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudPlayerIds {
    pub local_id: PlayerId,
    pub opponent_id: PlayerId,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudMode {
    #[default]
    Hidden,
    EconomyBasic,
    EconomyAuction,
    Frozen,
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
    pub reserve_container: Entity,
    pub reserve_label: Entity,
    pub figurine: Entity,
    pub timer_bar: Entity,
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
pub struct CurrentManaShape;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReserveManaShape;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManaShapeKind {
    Bar,
    Diamond,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ManaShapeGeometry {
    pub kind: ManaShapeKind,
    pub width_px: f32,
    pub height_px: f32,
    pub rotation_degrees: f32,
}

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

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GoldTweenTarget {
    pub gold: f32,
    pub reserved_gold: f32,
    pub is_populated: bool,
}

impl Default for GoldTweenTarget {
    fn default() -> Self {
        Self {
            gold: 0.0,
            reserved_gold: 0.0,
            is_populated: false,
        }
    }
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManaDisplayState {
    pub current_mana: u32,
    pub mana_cap: u32,
    pub reserve_mana: u32,
    pub is_populated: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ManaTweenTarget {
    pub current_mana: f32,
    pub mana_cap: f32,
    pub reserve_mana: f32,
    pub is_populated: bool,
}

impl Default for ManaTweenTarget {
    fn default() -> Self {
        Self {
            current_mana: 0.0,
            mana_cap: 0.0,
            reserve_mana: 0.0,
            is_populated: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GoldTweenLens {
    pub start_gold: f32,
    pub end_gold: f32,
    pub start_reserved_gold: f32,
    pub end_reserved_gold: f32,
}

impl Lens<GoldTweenTarget> for GoldTweenLens {
    fn lerp(&mut self, mut target: Mut<GoldTweenTarget>, ratio: f32) {
        target.gold = lerp_f32(self.start_gold, self.end_gold, ratio);
        target.reserved_gold = lerp_f32(self.start_reserved_gold, self.end_reserved_gold, ratio);
    }
}

#[derive(Clone, Debug)]
pub struct ManaTweenLens {
    pub start_current_mana: f32,
    pub end_current_mana: f32,
    pub start_mana_cap: f32,
    pub end_mana_cap: f32,
    pub start_reserve_mana: f32,
    pub end_reserve_mana: f32,
}

impl Lens<ManaTweenTarget> for ManaTweenLens {
    fn lerp(&mut self, mut target: Mut<ManaTweenTarget>, ratio: f32) {
        target.current_mana = lerp_f32(self.start_current_mana, self.end_current_mana, ratio);
        target.mana_cap = lerp_f32(self.start_mana_cap, self.end_mana_cap, ratio);
        target.reserve_mana = lerp_f32(self.start_reserve_mana, self.end_reserve_mana, ratio);
    }
}

#[derive(Message, Debug, Clone)]
pub struct HudGoldBroadcastMessage(pub S2CGoldBroadcast);

/// Marker for the HUD class figurine entity (own player's class portrait).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudFigurine;

/// Marker for the HUD phase timer bar fill entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudTimerBar;

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
        if !app.is_plugin_added::<TweeningPlugin>() {
            app.add_plugins(TweeningPlugin);
        }

        app.init_state::<ClientState>()
            .init_resource::<CurrentClientPhase>()
            .init_resource::<PlayerEconomyView>()
            .init_resource::<HudConfig>()
            .init_resource::<HudMode>()
            .add_message::<HudObjectiveUpdate>()
            .add_message::<HudGoldBroadcastMessage>()
            .add_message::<PresentationGameSnapshotMessage>()
            .configure_sets(
                Update,
                (
                    HudSystemSet::PhaseTransition,
                    HudSystemSet::MessageDrain,
                    HudSystemSet::StateSync,
                )
                    .chain()
                    .run_if(in_state(ClientState::InSession)),
            )
            .add_systems(OnEnter(ClientState::InSession), spawn_hud)
            .add_systems(OnExit(ClientState::InSession), despawn_hud)
            .add_systems(
                Update,
                (
                    hud_phase_transition_system
                        .in_set(HudSystemSet::PhaseTransition)
                        .before(update_phase_label_round_counter_system),
                    update_phase_label_round_counter_system.in_set(HudSystemSet::PhaseTransition),
                    handle_game_snapshot_system
                        .in_set(HudSystemSet::MessageDrain)
                        .before(handle_gold_broadcast_system)
                        .before(sync_hud_economy_view_system)
                        .before(handle_hud_objective_update_system),
                    drain_gold_broadcast_receiver_system
                        .in_set(HudSystemSet::MessageDrain)
                        .before(handle_gold_broadcast_system),
                    handle_gold_broadcast_system
                        .in_set(HudSystemSet::MessageDrain)
                        .before(sync_hud_economy_view_system),
                    sync_hud_economy_view_system.in_set(HudSystemSet::MessageDrain),
                    handle_hud_objective_update_system.in_set(HudSystemSet::MessageDrain),
                    sync_gold_text_system
                        .in_set(HudSystemSet::StateSync)
                        .after(AnimationSystem::AnimationUpdate),
                    sync_mana_text_system
                        .in_set(HudSystemSet::StateSync)
                        .after(AnimationSystem::AnimationUpdate),
                    sync_scoreboard_dot_layout_system.in_set(HudSystemSet::StateSync),
                    sync_figurine_image_system.in_set(HudSystemSet::StateSync),
                    sync_dot_image_on_objective_destroyed_system.in_set(HudSystemSet::StateSync),
                ),
            );
    }
}

pub fn hud_phase_transition_system(
    current: Res<CurrentClientPhase>,
    entities: Option<Res<HudEntities>>,
    mut mode: ResMut<HudMode>,
    mut commands: Commands,
    mut visibility: Query<&mut Visibility>,
    gold_states: Query<&GoldDisplayState>,
    mut gold_texts: Query<&mut Text>,
    mut gold_spans: Query<&mut TextSpan>,
    mut numeric_animators: Query<
        (Entity, &mut TweenAnim),
        Or<(With<GoldLabelOwner>, With<ManaLabel>)>,
    >,
    mut gold_tween_targets: Query<&mut GoldTweenTarget>,
    mana_states: Query<&ManaDisplayState, With<ManaLabel>>,
    mut mana_tween_targets: Query<&mut ManaTweenTarget, With<ManaLabel>>,
) {
    if !current.is_changed() {
        return;
    }

    let Some(entities) = entities else {
        return;
    };

    match current.phase {
        RoundPhase::Lobby | RoundPhase::Handshaking => {
            *mode = HudMode::Hidden;
            set_visibility(&mut visibility, entities.root, Visibility::Hidden);
        }
        RoundPhase::DraftInitial
        | RoundPhase::DraftShop
        | RoundPhase::Placement
        | RoundPhase::Resolution => {
            *mode = HudMode::EconomyBasic;
            set_hud_visible(&entities, &mut visibility);
            sync_gold_label_for_mode(
                entities.own_gold_parent,
                entities.own_gold_span,
                HudMode::EconomyBasic,
                &gold_states,
                &mut gold_texts,
                &mut gold_spans,
            );
            sync_gold_label_for_mode(
                entities.opponent_gold_parent,
                entities.opponent_gold_span,
                HudMode::EconomyBasic,
                &gold_states,
                &mut gold_texts,
                &mut gold_spans,
            );
        }
        RoundPhase::DraftAuction => {
            *mode = HudMode::EconomyAuction;
            set_hud_visible(&entities, &mut visibility);
            sync_gold_label_for_mode(
                entities.own_gold_parent,
                entities.own_gold_span,
                HudMode::EconomyAuction,
                &gold_states,
                &mut gold_texts,
                &mut gold_spans,
            );
            sync_gold_label_for_mode(
                entities.opponent_gold_parent,
                entities.opponent_gold_span,
                HudMode::EconomyAuction,
                &gold_states,
                &mut gold_texts,
                &mut gold_spans,
            );
        }
        RoundPhase::GameOver => {
            let render_mode = if *mode == HudMode::EconomyAuction {
                HudMode::EconomyAuction
            } else {
                HudMode::EconomyBasic
            };
            *mode = HudMode::Frozen;
            set_hud_visible(&entities, &mut visibility);
            snap_numeric_tween_targets(
                &entities,
                &gold_states,
                &mut gold_tween_targets,
                &mana_states,
                &mut mana_tween_targets,
            );
            cancel_hud_numeric_tweens(&mut commands, &mut numeric_animators);
            sync_gold_label_for_mode(
                entities.own_gold_parent,
                entities.own_gold_span,
                render_mode,
                &gold_states,
                &mut gold_texts,
                &mut gold_spans,
            );
            sync_gold_label_for_mode(
                entities.opponent_gold_parent,
                entities.opponent_gold_span,
                render_mode,
                &gold_states,
                &mut gold_texts,
                &mut gold_spans,
            );
        }
    }
}

fn spawn_hud(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    config: Res<HudConfig>,
    placeholder_assets: Option<Res<PlaceholderAssets>>,
    existing: Option<Res<HudEntities>>,
) {
    if existing.is_some() {
        return;
    }

    // Use fallback handle when PlaceholderAssets not yet available (test contexts).
    // When AssetServer is not present (minimal test setup), use a default handle.
    let fallback_handle = if let Some(pa) = &placeholder_assets {
        pa.fallback.clone()
    } else if let Some(server) = &asset_server {
        server.load(crate::asset_wiring::PLACEHOLDER_FALLBACK_ASSET)
    } else {
        Handle::default()
    };

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
        HUD_GOLD_ROW_GAP_PX,
    );
    let mana_label = spawn_mana_label(
        &mut commands,
        root,
        "HUD Mana Label",
        current_mana_bar_node(config.hud_margin_px),
    );
    let (reserve_container, reserve_label) =
        spawn_reserve_mana_label(&mut commands, root, config.hud_margin_px);

    // ── PAW-004: class figurine (own player) ──────────────────────────────────
    // Spawned with fallback; updated to the correct class asset in StateSync
    // when the first S2CGameSnapshot arrives and own ClassId is known.
    let figurine = commands
        .spawn((
            Name::new("HUD Class Figurine"),
            HudEntity,
            HudFigurine,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(config.hud_margin_px),
                bottom: Val::Px(config.hud_margin_px + 60.0),
                width: Val::Px(64.0),
                height: Val::Px(64.0),
                ..default()
            },
            ImageNode::new(fallback_handle.clone()),
            Visibility::Hidden,
            ChildOf(root),
        ))
        .id();

    // ── PAW-004: phase timer bar fill ─────────────────────────────────────────
    // Image is static; only Node width changes to represent timer progress.
    let timer_bar_image = if let Some(server) = &asset_server {
        ImageNode::new(server.load(HUD_PHASE_TIMER_BAR_ASSET))
    } else {
        ImageNode::new(Handle::default())
    };
    let timer_bar = commands
        .spawn((
            Name::new("HUD Phase Timer Bar"),
            HudEntity,
            HudTimerBar,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(config.hud_margin_px),
                top: Val::Px(config.hud_margin_px + 48.0),
                width: Val::Px(200.0),
                height: Val::Px(8.0),
                ..default()
            },
            timer_bar_image,
            Visibility::Hidden,
            ChildOf(root),
        ))
        .id();

    let dots = spawn_scoreboard_dots(&mut commands, asset_server.as_deref(), root, &config);

    commands.insert_resource(HudEntities {
        root,
        phase_label,
        round_counter,
        own_gold_parent,
        own_gold_span,
        opponent_gold_parent,
        opponent_gold_span,
        mana_label,
        reserve_container,
        reserve_label,
        figurine,
        timer_bar,
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
            hud_text_font(HUD_SECONDARY_FONT_SIZE_PX),
            TextColor(HUD_PRIMARY_TEXT_COLOR),
            BackgroundColor(HUD_TEXT_BACKGROUND_COLOR),
            node,
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_mana_label(
    commands: &mut Commands,
    parent: Entity,
    name: &'static str,
    node: Node,
) -> Entity {
    commands
        .spawn((
            Name::new(name),
            HudEntity,
            ManaLabel,
            CurrentManaShape,
            ManaShapeGeometry {
                kind: ManaShapeKind::Bar,
                width_px: CURRENT_MANA_BAR_WIDTH_PX,
                height_px: CURRENT_MANA_BAR_HEIGHT_PX,
                rotation_degrees: 0.0,
            },
            ManaDisplayState::default(),
            ManaTweenTarget::default(),
            Text::new("-- / --"),
            hud_text_font(HUD_SECONDARY_FONT_SIZE_PX),
            TextColor(HUD_PRIMARY_TEXT_COLOR),
            BackgroundColor(current_mana_bar_fill()),
            BorderColor::all(current_mana_bar_border()),
            node,
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_reserve_mana_label(
    commands: &mut Commands,
    parent: Entity,
    margin_px: f32,
) -> (Entity, Entity) {
    let container = commands
        .spawn((
            Name::new("HUD Reserve Mana Diamond"),
            HudEntity,
            ReserveManaShape,
            ManaShapeGeometry {
                kind: ManaShapeKind::Diamond,
                width_px: RESERVE_MANA_DIAMOND_SIZE_PX,
                height_px: RESERVE_MANA_DIAMOND_SIZE_PX,
                rotation_degrees: RESERVE_MANA_DIAMOND_ROTATION_DEGREES,
            },
            reserve_mana_diamond_node(margin_px),
            UiTransform::from_rotation(Rot2::degrees(RESERVE_MANA_DIAMOND_ROTATION_DEGREES)),
            BackgroundColor(reserve_mana_diamond_fill()),
            BorderColor::all(reserve_mana_diamond_border()),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();
    let label = commands
        .spawn((
            Name::new("HUD Reserve Mana Label"),
            HudEntity,
            ReserveManaLabel,
            Text::new(""),
            hud_text_font(HUD_SECONDARY_FONT_SIZE_PX),
            TextColor(HUD_PRIMARY_TEXT_COLOR),
            reserve_mana_label_node(),
            UiTransform::from_rotation(Rot2::degrees(-RESERVE_MANA_DIAMOND_ROTATION_DEGREES)),
            Visibility::Hidden,
            ChildOf(container),
        ))
        .id();

    (container, label)
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
            GoldTweenTarget::default(),
            Text::new("--g"),
            hud_text_font(HUD_GOLD_FONT_SIZE_PX),
            TextColor(HUD_GOLD_TEXT_COLOR),
            BackgroundColor(HUD_TEXT_BACKGROUND_COLOR),
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
            hud_text_font(HUD_RESERVED_GOLD_FONT_SIZE_PX),
            TextColor(HUD_RESERVED_GOLD_TEXT_COLOR),
            Visibility::Hidden,
            ChildOf(parent_entity),
        ))
        .id();

    (parent_entity, span_entity)
}

fn spawn_scoreboard_dots(
    commands: &mut Commands,
    asset_server: Option<&AssetServer>,
    parent: Entity,
    config: &HudConfig,
) -> [[Entity; HUD_DOTS_PER_ROW]; HUD_DOT_ROWS] {
    std::array::from_fn(|row| {
        std::array::from_fn(|lane_index| {
            let row_marker = match row {
                0 => ScoreboardRow::Opponent,
                _ => ScoreboardRow::Local,
            };

            // Own row starts Alive; opponent row starts Unknown.
            let initial_dot_state = match row_marker {
                ScoreboardRow::Local => ObjectiveDotState::Alive,
                ScoreboardRow::Opponent => ObjectiveDotState::Unknown,
            };
            let dot_image_path = hud_objective_dot_asset(initial_dot_state);
            let dot_image = if let Some(server) = asset_server {
                ImageNode::new(server.load(dot_image_path))
            } else {
                ImageNode::new(Handle::default())
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
                        left: Val::Px(0.0),
                        width: Val::Px(config.hud_dot_diameter_px),
                        height: Val::Px(config.hud_dot_diameter_px),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(config.hud_dot_diameter_px * 0.5)),
                        ..default()
                    },
                    dot_image,
                    BackgroundColor(alive_dot_fill()),
                    BorderColor::all(alive_dot_border()),
                    Visibility::Hidden,
                    ChildOf(parent),
                ))
                .id()
        })
    })
}

pub fn drain_gold_broadcast_receiver_system(
    mut receivers: Query<&mut MessageReceiver<S2CGoldBroadcast>>,
    mut writer: MessageWriter<HudGoldBroadcastMessage>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            writer.write(HudGoldBroadcastMessage(message));
        }
    }
}

pub fn handle_game_snapshot_system(
    mut commands: Commands,
    mut messages: MessageReader<PresentationGameSnapshotMessage>,
    entities: Option<Res<HudEntities>>,
    mut current: ResMut<CurrentClientPhase>,
    mut mode: ResMut<HudMode>,
    mut visibility: Query<&mut Visibility>,
    mut gold_labels: Query<(
        Entity,
        &GoldLabelOwner,
        &mut GoldDisplayState,
        &mut GoldTweenTarget,
        Option<&Children>,
        Option<&mut TweenAnim>,
    )>,
    mut mana_labels: Query<
        (
            Entity,
            &mut ManaDisplayState,
            &mut ManaTweenTarget,
            Option<&mut TweenAnim>,
        ),
        (With<ManaLabel>, Without<GoldDisplayState>),
    >,
    mut texts: Query<&mut Text>,
    mut spans: Query<&mut TextSpan>,
    mut dots: Query<(
        &mut ScoreboardDotState,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    let mut last_snapshot = None;
    for message in messages.read().map(|message| &message.0) {
        last_snapshot = Some(message);
    }

    let Some(snapshot) = last_snapshot else {
        return;
    };
    let Some(entities) = entities else {
        return;
    };
    let Some((own, opponent)) = snapshot_hud_players(snapshot) else {
        warn!(
            "HUD: snapshot for {:?} does not contain exactly one local and one opponent player",
            snapshot.recipient_player_id
        );
        return;
    };

    commands.insert_resource(HudPlayerIds {
        local_id: own.player_id,
        opponent_id: opponent.player_id,
    });

    current.phase = snapshot.phase;
    current.round = snapshot.round_number;

    let next_mode = hud_mode_for_phase(snapshot.phase);
    *mode = next_mode;
    if next_mode == HudMode::Hidden {
        set_visibility(&mut visibility, entities.root, Visibility::Hidden);
    } else {
        set_hud_visible(&entities, &mut visibility);
    }

    write_phase_label_and_round(&entities, snapshot.phase, snapshot.round_number, &mut texts);
    write_snapshot_gold_states(
        own,
        opponent,
        &mut commands,
        &mut gold_labels,
        next_mode,
        &mut texts,
        &mut spans,
    );
    write_snapshot_mana_state(own, &entities, &mut commands, &mut mana_labels);
    write_snapshot_dot_states(own, opponent, &entities, &mut dots);
}

pub fn handle_gold_broadcast_system(
    mut commands: Commands,
    mode: Res<HudMode>,
    config: Res<HudConfig>,
    player_ids: Option<Res<HudPlayerIds>>,
    mut messages: MessageReader<HudGoldBroadcastMessage>,
    mut gold_labels: Query<(
        Entity,
        &GoldLabelOwner,
        &mut GoldDisplayState,
        &mut GoldTweenTarget,
        Option<&mut TweenAnim>,
    )>,
) {
    if *mode == HudMode::Frozen {
        drain_hud_gold_broadcast_messages(&mut messages);
        return;
    }

    let Some(player_ids) = player_ids else {
        drain_hud_gold_broadcast_messages(&mut messages);
        return;
    };

    for message in messages.read().map(|message| &message.0) {
        let reserved_gold = clamped_reserved_gold(message);
        for (entity, owner, mut state, mut target, animator) in &mut gold_labels {
            match (*owner, message.player_id) {
                (GoldLabelOwner::Opponent, player_id) if player_id == player_ids.opponent_id => {
                    state.gold = message.gold as f32;
                    state.reserved_gold = reserved_gold;
                    state.is_populated = true;
                    start_gold_tween(
                        &mut commands,
                        entity,
                        &config,
                        &state,
                        &mut target,
                        animator,
                    );
                }
                (GoldLabelOwner::Local, player_id) if player_id == player_ids.local_id => {
                    state.reserved_gold = reserved_gold;
                    start_gold_tween(
                        &mut commands,
                        entity,
                        &config,
                        &state,
                        &mut target,
                        animator,
                    );
                }
                _ => {}
            }
        }
    }
}

pub fn handle_hud_objective_update_system(
    mode: Res<HudMode>,
    mut updates: MessageReader<HudObjectiveUpdate>,
    entities: Option<Res<HudEntities>>,
    player_ids: Option<Res<HudPlayerIds>>,
    mut dots: Query<(
        &mut ScoreboardDotState,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    if *mode == HudMode::Frozen {
        for _update in updates.read() {}
        return;
    }

    let Some(entities) = entities else {
        for _update in updates.read() {}
        return;
    };
    let Some(player_ids) = player_ids else {
        for _update in updates.read() {}
        return;
    };

    for update in updates.read() {
        if !(1..=HUD_DOTS_PER_ROW as u8).contains(&update.lane) {
            warn!(
                "HUD: OOB lane {} in HudObjectiveUpdate - ignored",
                update.lane
            );
            continue;
        }

        let Some(row_index) = scoreboard_row_index(update.target_player_id, &player_ids) else {
            warn!(
                "HUD: unknown player {:?} in HudObjectiveUpdate - ignored",
                update.target_player_id
            );
            continue;
        };

        let lane_index = usize::from(update.lane - 1);
        let dot = entities.dots[row_index][lane_index];
        if let Ok((mut state, mut background, mut border)) = dots.get_mut(dot) {
            state.destroyed = true;
            *background = BackgroundColor(Color::NONE);
            *border = BorderColor::all(destroyed_dot_border());
        }
    }
}

pub fn sync_scoreboard_dot_layout_system(
    layout: Option<Res<BoardLayout>>,
    config: Res<HudConfig>,
    mut warned_missing_layout: Local<bool>,
    mut dots: Query<(&ScoreboardDot, &mut Node)>,
) {
    let Some(layout) = layout else {
        if !*warned_missing_layout {
            warn!("HUD: BoardLayout missing; scoreboard dot alignment skipped");
            *warned_missing_layout = true;
        }
        return;
    };
    *warned_missing_layout = false;

    for (dot, mut node) in &mut dots {
        let lane = dot.lane_index as u8 + 1;
        let Some(center_x) = layout.scoreboard_lane_center_x(lane) else {
            warn!("HUD: invalid scoreboard lane {} - ignored", lane);
            continue;
        };
        node.left = Val::Px(center_x - config.hud_dot_diameter_px * 0.5);
    }
}

/// PAW-004: StateSync — update the figurine `ImageNode` to the own player's
/// class asset. Runs every frame but only writes when the snapshot class
/// differs from what the figurine currently shows.
pub fn sync_figurine_image_system(
    asset_server: Option<Res<AssetServer>>,
    mut figurines: Query<&mut ImageNode, With<HudFigurine>>,
    hud_player_ids: Option<Res<HudPlayerIds>>,
    entities: Option<Res<HudEntities>>,
    mut last_class: Local<Option<shared::card::ClassId>>,
    mut snapshot_messages: MessageReader<PresentationGameSnapshotMessage>,
) {
    // Drain messages to find the latest own class_id.
    let mut latest_class = None;
    for msg in snapshot_messages.read() {
        if let Some(own) = msg
            .0
            .players
            .iter()
            .find(|p| p.player_id == msg.0.recipient_player_id)
        {
            latest_class = Some(own.class_id);
        }
    }

    let Some(class_id) = latest_class else {
        return;
    };

    // Only update if the class changed since last sync.
    if *last_class == Some(class_id) {
        return;
    }
    *last_class = Some(class_id);

    let Some(entities) = entities else {
        return;
    };
    let Some(server) = asset_server else {
        return;
    };

    if let Ok(mut img) = figurines.get_mut(entities.figurine) {
        img.image = server.load(hud_figurine_asset(class_id));
    }

    let _ = hud_player_ids; // used for future class-change detection
}

/// PAW-004: StateSync — when a `HudObjectiveUpdate` message marks a dot as
/// destroyed, update that dot's `ImageNode` to the destroyed asset.
pub fn sync_dot_image_on_objective_destroyed_system(
    asset_server: Option<Res<AssetServer>>,
    mode: Res<HudMode>,
    mut updates: MessageReader<HudObjectiveUpdate>,
    entities: Option<Res<HudEntities>>,
    player_ids: Option<Res<HudPlayerIds>>,
    mut dot_images: Query<&mut ImageNode, With<ScoreboardDot>>,
    dot_states: Query<&ScoreboardDotState, With<ScoreboardDot>>,
) {
    if *mode == HudMode::Frozen {
        for _u in updates.read() {}
        return;
    }

    let Some(entities) = entities else {
        for _u in updates.read() {}
        return;
    };
    let Some(player_ids) = player_ids else {
        for _u in updates.read() {}
        return;
    };

    for update in updates.read() {
        if !(1..=HUD_DOTS_PER_ROW as u8).contains(&update.lane) {
            warn!(
                "HUD(PAW-004): OOB lane {} in HudObjectiveUpdate image sync - ignored",
                update.lane
            );
            continue;
        }

        let Some(row_index) = scoreboard_row_index(update.target_player_id, &player_ids) else {
            warn!(
                "HUD(PAW-004): unknown player {:?} in HudObjectiveUpdate image sync - ignored",
                update.target_player_id
            );
            continue;
        };

        let lane_index = usize::from(update.lane - 1);
        let dot_entity = entities.dots[row_index][lane_index];

        // Check current dot state to pick the correct asset.
        let is_already_destroyed = dot_states
            .get(dot_entity)
            .map(|s| s.destroyed)
            .unwrap_or(false);

        if !is_already_destroyed {
            // The ScoreboardDotState is updated by handle_hud_objective_update_system
            // which runs before StateSync. We pick the destroyed asset unconditionally
            // because this system is only triggered when an objective is destroyed.
        }

        if let Some(server) = &asset_server {
            if let Ok(mut img) = dot_images.get_mut(dot_entity) {
                img.image = server.load(HUD_OBJECTIVE_DOT_DESTROYED_ASSET);
            }
        }
    }
}

pub fn update_phase_label_round_counter_system(
    current: Res<CurrentClientPhase>,
    entities: Option<Res<HudEntities>>,
    mut texts: Query<&mut Text>,
) {
    if !current.is_changed() {
        return;
    }

    let Some(entities) = entities else {
        return;
    };
    let Some(label) = phase_label_text(current.phase) else {
        if current.phase != RoundPhase::Lobby {
            warn!("HUD: unsupported phase label for {:?}", current.phase);
        }
        return;
    };

    if let Ok(mut phase_text) = texts.get_mut(entities.phase_label) {
        phase_text.0.clear();
        phase_text.0.push_str(label);
    }

    if let Ok(mut round_text) = texts.get_mut(entities.round_counter) {
        round_text.0 = format!("R{}", current.round);
    }
}

pub fn phase_label_text(phase: RoundPhase) -> Option<&'static str> {
    match phase {
        RoundPhase::Lobby => None,
        RoundPhase::DraftInitial => Some("DRAFT INITIAL"),
        RoundPhase::DraftShop => Some("DRAFT"),
        RoundPhase::DraftAuction => Some("AUCTION"),
        RoundPhase::Placement => Some("PLACEMENT"),
        RoundPhase::Resolution => Some("RESOLUTION"),
        RoundPhase::GameOver => Some("GAME OVER"),
        RoundPhase::Handshaking => None,
    }
}

pub fn sync_hud_economy_view_system(
    mut commands: Commands,
    mode: Res<HudMode>,
    config: Res<HudConfig>,
    economy_view: Res<PlayerEconomyView>,
    mut gold_labels: Query<
        (
            Entity,
            &GoldLabelOwner,
            &mut GoldDisplayState,
            &mut GoldTweenTarget,
            Option<&mut TweenAnim>,
        ),
        Without<ManaLabel>,
    >,
    mut mana_labels: Query<
        (
            Entity,
            &mut ManaDisplayState,
            &mut ManaTweenTarget,
            Option<&mut TweenAnim>,
        ),
        (With<ManaLabel>, Without<GoldDisplayState>),
    >,
) {
    if !economy_view.initialized || *mode == HudMode::Frozen {
        return;
    }

    if economy_view.mana_cap == 0 {
        warn!("HUD: mana_cap=0 received - server invariant violated");
    }

    let Ok((mana_entity, mut mana_state, mut mana_target, mana_animator)) =
        mana_labels.single_mut()
    else {
        return;
    };

    let mut mana_needs_tween = false;
    for (entity, owner, mut state, mut target, animator) in &mut gold_labels {
        if *owner == GoldLabelOwner::Local {
            let gold_needs_tween = !state.is_populated || state.gold != economy_view.gold as f32;
            mana_needs_tween = mana_display_differs_from_view(&mana_state, &economy_view);
            if !gold_needs_tween && !mana_needs_tween {
                return;
            }

            apply_player_economy_view(&economy_view, &mut state, &mut mana_state);
            if gold_needs_tween {
                start_gold_tween(
                    &mut commands,
                    entity,
                    &config,
                    &state,
                    &mut target,
                    animator,
                );
            }
        }
    }

    if mana_needs_tween {
        start_mana_tween(
            &mut commands,
            mana_entity,
            &config,
            &mana_state,
            &mut mana_target,
            mana_animator,
        );
    }
}

fn drain_hud_gold_broadcast_messages(messages: &mut MessageReader<HudGoldBroadcastMessage>) {
    for _message in messages.read() {}
}

pub fn apply_player_economy_view(
    economy_view: &PlayerEconomyView,
    own_gold: &mut GoldDisplayState,
    mana_state: &mut ManaDisplayState,
) {
    own_gold.gold = economy_view.gold as f32;
    own_gold.is_populated = true;
    mana_state.current_mana = economy_view.current_mana;
    mana_state.mana_cap = economy_view.mana_cap as u32;
    mana_state.reserve_mana = economy_view.reserve_mana;
    mana_state.is_populated = true;
}

fn mana_display_differs_from_view(
    state: &ManaDisplayState,
    economy_view: &PlayerEconomyView,
) -> bool {
    !state.is_populated
        || state.current_mana != economy_view.current_mana
        || state.mana_cap != u32::from(economy_view.mana_cap)
        || state.reserve_mana != economy_view.reserve_mana
}

pub fn sync_gold_text_system(
    mode: Res<HudMode>,
    mut gold_labels: Query<(
        &GoldDisplayState,
        &mut GoldTweenTarget,
        &mut Text,
        Option<&Children>,
        Option<&TweenAnim>,
    )>,
    mut spans: Query<&mut TextSpan>,
) {
    for (state, mut target, mut text, children, animator) in &mut gold_labels {
        if !is_hud_tween_active(animator) {
            sync_gold_tween_target_to_authoritative(state, &mut target);
        }

        let display = gold_display_state_from_target(&target);
        text.0 = format_gold_text(&display);

        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut span) = spans.get_mut(child) {
                    span.0 = format_reserved_gold_span(&display, *mode);
                }
            }
        }
    }
}

pub fn sync_mana_text_system(
    entities: Option<Res<HudEntities>>,
    mut mana_labels: Query<
        (
            &ManaDisplayState,
            &mut ManaTweenTarget,
            &mut Text,
            Option<&TweenAnim>,
        ),
        With<ManaLabel>,
    >,
    mut reserve_labels: Query<&mut Text, (With<ReserveManaLabel>, Without<ManaLabel>)>,
    mut visibility: Query<&mut Visibility>,
) {
    let Ok((state, mut target, mut mana_text, animator)) = mana_labels.single_mut() else {
        return;
    };
    let Ok(mut reserve_text) = reserve_labels.single_mut() else {
        return;
    };

    if !is_hud_tween_active(animator) {
        sync_mana_tween_target_to_authoritative(state, &mut target);
    }

    if !target.is_populated {
        mana_text.0 = "-- / --".to_string();
        reserve_text.0.clear();
        set_reserve_mana_visibility(&entities, &mut visibility, Visibility::Hidden);
        return;
    }

    mana_text.0 = format!(
        "{} / {}",
        display_numeric_value(target.current_mana),
        display_numeric_value(target.mana_cap)
    );

    if state.reserve_mana > 0 {
        let reserve_value = display_numeric_value(target.reserve_mana).max(1);
        reserve_text.0 = format!("+{} reserve", reserve_value);
        set_reserve_mana_visibility(&entities, &mut visibility, Visibility::Visible);
    } else {
        reserve_text.0.clear();
        set_reserve_mana_visibility(&entities, &mut visibility, Visibility::Hidden);
    }
}

fn format_gold_text(state: &GoldDisplayState) -> String {
    if state.is_populated {
        format!("{}g", state.gold as u32)
    } else {
        "--g".to_string()
    }
}

fn format_reserved_gold_span(state: &GoldDisplayState, mode: HudMode) -> String {
    if mode == HudMode::EconomyAuction && state.is_populated {
        format!(" ({}r)", display_reserved_gold(state))
    } else {
        String::new()
    }
}

fn snapshot_hud_players(snapshot: &S2CGameSnapshot) -> Option<(&PlayerSnapshot, &PlayerSnapshot)> {
    let own = snapshot
        .players
        .iter()
        .find(|player| player.player_id == snapshot.recipient_player_id)?;
    let opponent = snapshot
        .players
        .iter()
        .find(|player| player.player_id != snapshot.recipient_player_id)?;

    Some((own, opponent))
}

fn hud_mode_for_phase(phase: RoundPhase) -> HudMode {
    match phase {
        RoundPhase::Lobby | RoundPhase::Handshaking => HudMode::Hidden,
        RoundPhase::DraftAuction => HudMode::EconomyAuction,
        RoundPhase::GameOver => HudMode::Frozen,
        RoundPhase::DraftInitial
        | RoundPhase::DraftShop
        | RoundPhase::Placement
        | RoundPhase::Resolution => HudMode::EconomyBasic,
    }
}

fn write_phase_label_and_round(
    entities: &HudEntities,
    phase: RoundPhase,
    round_number: u32,
    texts: &mut Query<&mut Text>,
) {
    if let Some(label) = phase_label_text(phase) {
        if let Ok(mut phase_text) = texts.get_mut(entities.phase_label) {
            phase_text.0.clear();
            phase_text.0.push_str(label);
        }
    }

    if let Ok(mut round_text) = texts.get_mut(entities.round_counter) {
        round_text.0 = format!("R{round_number}");
    }
}

fn write_snapshot_gold_states(
    own: &PlayerSnapshot,
    opponent: &PlayerSnapshot,
    commands: &mut Commands,
    gold_labels: &mut Query<(
        Entity,
        &GoldLabelOwner,
        &mut GoldDisplayState,
        &mut GoldTweenTarget,
        Option<&Children>,
        Option<&mut TweenAnim>,
    )>,
    mode: HudMode,
    texts: &mut Query<&mut Text>,
    spans: &mut Query<&mut TextSpan>,
) {
    for (entity, owner, mut state, mut target, children, animator) in gold_labels.iter_mut() {
        let player = match *owner {
            GoldLabelOwner::Local => own,
            GoldLabelOwner::Opponent => opponent,
        };
        state.gold = player.gold as f32;
        state.reserved_gold =
            clamped_reserved_gold_fields(player.player_id, player.gold, player.reserved_gold);
        state.is_populated = true;
        sync_gold_tween_target_to_authoritative(&state, &mut target);
        remove_hud_tween(commands, entity, animator);

        if let Ok(mut text) = texts.get_mut(entity) {
            text.0 = format_gold_text(&state);
        }
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut span) = spans.get_mut(child) {
                    span.0 = format_reserved_gold_span(&state, mode);
                }
            }
        }
    }
}

fn write_snapshot_mana_state(
    own: &PlayerSnapshot,
    entities: &HudEntities,
    commands: &mut Commands,
    mana_labels: &mut Query<
        (
            Entity,
            &mut ManaDisplayState,
            &mut ManaTweenTarget,
            Option<&mut TweenAnim>,
        ),
        (With<ManaLabel>, Without<GoldDisplayState>),
    >,
) {
    let Ok((entity, mut state, mut target, animator)) = mana_labels.get_mut(entities.mana_label)
    else {
        return;
    };

    state.current_mana = own.current_mana;
    state.mana_cap = own.mana_cap as u32;
    state.reserve_mana = own.reserve_mana;
    state.is_populated = true;
    sync_mana_tween_target_to_authoritative(&state, &mut target);
    remove_hud_tween(commands, entity, animator);
}

fn write_snapshot_dot_states(
    own: &PlayerSnapshot,
    opponent: &PlayerSnapshot,
    entities: &HudEntities,
    dots: &mut Query<(
        &mut ScoreboardDotState,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    write_player_objective_dots(entities.dots[1], &own.objectives, dots);
    write_opponent_objective_dots(entities.dots[0], opponent, own, dots);
}

fn write_player_objective_dots(
    row: [Entity; HUD_DOTS_PER_ROW],
    objectives: &[shared::protocol::ObjectiveSnapshot],
    dots: &mut Query<(
        &mut ScoreboardDotState,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    reset_dot_row(row, dots);
    for objective in objectives {
        if !(1..=HUD_DOTS_PER_ROW as u8).contains(&objective.lane) {
            warn!(
                "HUD: OOB lane {} in snapshot objective - ignored",
                objective.lane
            );
            continue;
        }
        write_dot_destroyed(
            row[usize::from(objective.lane - 1)],
            objective.is_destroyed,
            dots,
        );
    }
}

fn write_opponent_objective_dots(
    row: [Entity; HUD_DOTS_PER_ROW],
    opponent: &PlayerSnapshot,
    own: &PlayerSnapshot,
    dots: &mut Query<(
        &mut ScoreboardDotState,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    reset_dot_row(row, dots);

    if !own.opponent_objectives.is_empty() {
        for objective in &own.opponent_objectives {
            write_opponent_dot(row, objective, dots);
        }
        return;
    }

    for objective in &opponent.objectives {
        if !(1..=HUD_DOTS_PER_ROW as u8).contains(&objective.lane) {
            warn!(
                "HUD: OOB lane {} in opponent snapshot objective - ignored",
                objective.lane
            );
            continue;
        }
        write_dot_destroyed(
            row[usize::from(objective.lane - 1)],
            objective.is_destroyed,
            dots,
        );
    }
}

fn write_opponent_dot(
    row: [Entity; HUD_DOTS_PER_ROW],
    objective: &OpponentObjectiveSnapshot,
    dots: &mut Query<(
        &mut ScoreboardDotState,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    if !(1..=HUD_DOTS_PER_ROW as u8).contains(&objective.lane) {
        warn!(
            "HUD: OOB lane {} in snapshot opponent objective - ignored",
            objective.lane
        );
        return;
    }
    write_dot_destroyed(
        row[usize::from(objective.lane - 1)],
        objective.is_destroyed,
        dots,
    );
}

fn reset_dot_row(
    row: [Entity; HUD_DOTS_PER_ROW],
    dots: &mut Query<(
        &mut ScoreboardDotState,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    for dot in row {
        write_dot_destroyed(dot, false, dots);
    }
}

fn write_dot_destroyed(
    entity: Entity,
    destroyed: bool,
    dots: &mut Query<(
        &mut ScoreboardDotState,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    if let Ok((mut state, mut background, mut border)) = dots.get_mut(entity) {
        state.destroyed = destroyed;
        if destroyed {
            *background = BackgroundColor(Color::NONE);
            *border = BorderColor::all(destroyed_dot_border());
        } else {
            *background = BackgroundColor(alive_dot_fill());
            *border = BorderColor::all(alive_dot_border());
        }
    }
}

fn remove_hud_tween(commands: &mut Commands, entity: Entity, animator: Option<Mut<TweenAnim>>) {
    if let Some(mut animator) = animator {
        if let Err(error) = cancel_tween_anim_in_place(&mut animator) {
            warn!("Failed to cancel HUD snapshot tween on entity {entity:?}: {error}");
        }
        commands.entity(entity).remove::<TweenAnim>();
    }
}

fn clamped_reserved_gold_fields(player_id: PlayerId, gold: u32, reserved_gold: u32) -> f32 {
    if reserved_gold > gold {
        warn!(
            "HUD: snapshot reserved_gold {} exceeds gold {} for {:?}; clamping display value",
            reserved_gold, gold, player_id
        );
        gold as f32
    } else {
        reserved_gold as f32
    }
}

fn display_reserved_gold(state: &GoldDisplayState) -> u32 {
    let gold = state.gold.max(0.0) as u32;
    let reserved_gold = state.reserved_gold.max(0.0) as u32;
    reserved_gold.min(gold)
}

fn display_numeric_value(value: f32) -> u32 {
    value.max(0.0) as u32
}

fn clamped_reserved_gold(message: &S2CGoldBroadcast) -> f32 {
    if message.reserved_gold > message.gold {
        warn!(
            "HUD: reserved_gold {} exceeds gold {} for {:?}; clamping display value",
            message.reserved_gold, message.gold, message.player_id
        );
        message.gold as f32
    } else {
        message.reserved_gold as f32
    }
}

fn set_hud_visible(entities: &HudEntities, visibility: &mut Query<&mut Visibility>) {
    for entity in [
        entities.root,
        entities.phase_label,
        entities.round_counter,
        entities.own_gold_parent,
        entities.own_gold_span,
        entities.opponent_gold_parent,
        entities.opponent_gold_span,
        entities.mana_label,
    ] {
        set_visibility(visibility, entity, Visibility::Visible);
    }

    for row in entities.dots {
        for dot in row {
            set_visibility(visibility, dot, Visibility::Visible);
        }
    }
}

fn set_reserve_mana_visibility(
    entities: &Option<Res<HudEntities>>,
    visibility: &mut Query<&mut Visibility>,
    target_visibility: Visibility,
) {
    let Some(entities) = entities else {
        return;
    };

    set_visibility(visibility, entities.reserve_container, target_visibility);
    set_visibility(visibility, entities.reserve_label, target_visibility);
}

fn cancel_hud_numeric_tweens(
    commands: &mut Commands,
    animators: &mut Query<(Entity, &mut TweenAnim), Or<(With<GoldLabelOwner>, With<ManaLabel>)>>,
) {
    for (entity, mut animator) in animators.iter_mut() {
        if let Err(error) = cancel_tween_anim_in_place(&mut animator) {
            warn!("Failed to cancel HUD numeric tween on entity {entity:?}: {error}");
        }
        commands.entity(entity).remove::<TweenAnim>();
    }
}

fn set_visibility(
    visibility: &mut Query<&mut Visibility>,
    entity: Entity,
    target_visibility: Visibility,
) {
    if let Ok(mut current_visibility) = visibility.get_mut(entity) {
        *current_visibility = target_visibility;
    }
}

fn scoreboard_row_index(target_player_id: PlayerId, player_ids: &HudPlayerIds) -> Option<usize> {
    if target_player_id == player_ids.opponent_id {
        Some(0)
    } else if target_player_id == player_ids.local_id {
        Some(1)
    } else {
        None
    }
}

fn alive_dot_fill() -> Color {
    Color::srgba(0.84, 0.88, 0.92, 0.88)
}

fn alive_dot_border() -> Color {
    Color::srgba(0.96, 0.98, 1.0, 0.95)
}

fn destroyed_dot_border() -> Color {
    Color::srgba(0.30, 0.32, 0.35, 0.70)
}

fn current_mana_bar_fill() -> Color {
    Color::srgba(0.05, 0.18, 0.24, 1.0)
}

fn current_mana_bar_border() -> Color {
    Color::srgba(0.72, 0.94, 1.0, 0.92)
}

fn reserve_mana_diamond_fill() -> Color {
    Color::srgba(0.07, 0.13, 0.30, 1.0)
}

fn reserve_mana_diamond_border() -> Color {
    Color::srgba(0.68, 0.78, 1.0, 0.92)
}

fn sync_gold_label_for_mode(
    parent: Entity,
    span: Entity,
    mode: HudMode,
    gold_states: &Query<&GoldDisplayState>,
    gold_texts: &mut Query<&mut Text>,
    gold_spans: &mut Query<&mut TextSpan>,
) {
    let state = gold_states.get(parent).ok();

    if let (Some(state), Ok(mut text)) = (state, gold_texts.get_mut(parent)) {
        text.0 = format_gold_text(state);
    }

    if let Ok(mut span_text) = gold_spans.get_mut(span) {
        span_text.0 = state
            .map(|state| format_reserved_gold_span(state, mode))
            .unwrap_or_default();
    }
}

fn snap_numeric_tween_targets(
    entities: &HudEntities,
    gold_states: &Query<&GoldDisplayState>,
    gold_targets: &mut Query<&mut GoldTweenTarget>,
    mana_states: &Query<&ManaDisplayState, With<ManaLabel>>,
    mana_targets: &mut Query<&mut ManaTweenTarget, With<ManaLabel>>,
) {
    for entity in [entities.own_gold_parent, entities.opponent_gold_parent] {
        if let (Ok(state), Ok(mut target)) = (gold_states.get(entity), gold_targets.get_mut(entity))
        {
            sync_gold_tween_target_to_authoritative(state, &mut target);
        }
    }

    if let (Ok(state), Ok(mut target)) = (
        mana_states.get(entities.mana_label),
        mana_targets.get_mut(entities.mana_label),
    ) {
        sync_mana_tween_target_to_authoritative(state, &mut target);
    }
}

fn sync_gold_tween_target_to_authoritative(state: &GoldDisplayState, target: &mut GoldTweenTarget) {
    target.gold = state.gold;
    target.reserved_gold = state.reserved_gold;
    target.is_populated = state.is_populated;
}

fn sync_mana_tween_target_to_authoritative(state: &ManaDisplayState, target: &mut ManaTweenTarget) {
    target.current_mana = state.current_mana as f32;
    target.mana_cap = state.mana_cap as f32;
    target.reserve_mana = state.reserve_mana as f32;
    target.is_populated = state.is_populated;
}

fn gold_display_state_from_target(target: &GoldTweenTarget) -> GoldDisplayState {
    GoldDisplayState {
        gold: target.gold,
        reserved_gold: target.reserved_gold,
        is_populated: target.is_populated,
    }
}

fn start_gold_tween(
    commands: &mut Commands,
    entity: Entity,
    config: &HudConfig,
    state: &GoldDisplayState,
    target: &mut GoldTweenTarget,
    animator: Option<Mut<TweenAnim>>,
) {
    if !state.is_populated || !target.is_populated {
        sync_gold_tween_target_to_authoritative(state, target);
        return;
    }

    let tween = gold_tween(config, target, state);
    target.is_populated = state.is_populated;
    start_or_replace_hud_tween(commands, entity, animator, tween);
}

fn start_mana_tween(
    commands: &mut Commands,
    entity: Entity,
    config: &HudConfig,
    state: &ManaDisplayState,
    target: &mut ManaTweenTarget,
    animator: Option<Mut<TweenAnim>>,
) {
    if !state.is_populated || !target.is_populated {
        sync_mana_tween_target_to_authoritative(state, target);
        return;
    }

    let tween = mana_tween(config, target, state);
    target.is_populated = state.is_populated;
    start_or_replace_hud_tween(commands, entity, animator, tween);
}

fn gold_tween(config: &HudConfig, target: &GoldTweenTarget, state: &GoldDisplayState) -> Tween {
    Tween::new(
        EaseFunction::CubicOut,
        hud_tween_duration(config),
        GoldTweenLens {
            start_gold: target.gold,
            end_gold: state.gold,
            start_reserved_gold: target.reserved_gold,
            end_reserved_gold: state.reserved_gold,
        },
    )
}

fn mana_tween(config: &HudConfig, target: &ManaTweenTarget, state: &ManaDisplayState) -> Tween {
    Tween::new(
        EaseFunction::CubicOut,
        hud_tween_duration(config),
        ManaTweenLens {
            start_current_mana: target.current_mana,
            end_current_mana: state.current_mana as f32,
            start_mana_cap: target.mana_cap,
            end_mana_cap: state.mana_cap as f32,
            start_reserve_mana: target.reserve_mana,
            end_reserve_mana: state.reserve_mana as f32,
        },
    )
}

fn start_or_replace_hud_tween(
    commands: &mut Commands,
    entity: Entity,
    animator: Option<Mut<TweenAnim>>,
    tween: Tween,
) {
    if let Some(mut animator) = animator {
        animator.destroy_on_completion = false;
        animator.playback_state = PlaybackState::Playing;
        if let Err(error) = animator.set_tweenable(tween) {
            warn!("Failed to replace HUD tween on entity {entity:?}: {error}");
        }
        return;
    }

    commands
        .entity(entity)
        .insert(TweenAnim::new(tween).with_destroy_on_completed(false));
}

fn hud_tween_duration(config: &HudConfig) -> Duration {
    Duration::from_millis(u64::from(config.hud_tween_duration_ms.min(300).max(1)))
}

fn is_hud_tween_active(animator: Option<&TweenAnim>) -> bool {
    animator
        .map(|animator| {
            animator.playback_state == PlaybackState::Playing
                && animator.tween_state() == TweenState::Active
        })
        .unwrap_or(false)
}

fn lerp_f32(start: f32, end: f32, ratio: f32) -> f32 {
    start + (end - start) * ratio
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
        top: Val::Px(margin_px + HUD_SECONDARY_ROW_GAP_PX),
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

fn current_mana_bar_node(margin_px: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(margin_px),
        bottom: Val::Px(margin_px),
        width: Val::Px(CURRENT_MANA_BAR_WIDTH_PX),
        height: Val::Px(CURRENT_MANA_BAR_HEIGHT_PX),
        min_width: Val::Px(CURRENT_MANA_BAR_WIDTH_PX),
        min_height: Val::Px(CURRENT_MANA_BAR_HEIGHT_PX),
        padding: UiRect::axes(Val::Px(10.0), Val::Px(3.0)),
        border: UiRect::all(Val::Px(2.0)),
        border_radius: BorderRadius::all(Val::Px(4.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

fn reserve_mana_diamond_node(margin_px: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(margin_px + 9.0),
        bottom: Val::Px(margin_px + 42.0),
        width: Val::Px(RESERVE_MANA_DIAMOND_SIZE_PX),
        height: Val::Px(RESERVE_MANA_DIAMOND_SIZE_PX),
        min_width: Val::Px(RESERVE_MANA_DIAMOND_SIZE_PX),
        min_height: Val::Px(RESERVE_MANA_DIAMOND_SIZE_PX),
        border: UiRect::all(Val::Px(2.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

fn reserve_mana_label_node() -> Node {
    Node {
        width: Val::Px(104.0),
        height: Val::Px(24.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}
