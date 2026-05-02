use bevy::prelude::*;
use bevy_tweening::TweenAnim;
use lightyear::prelude::MessageReceiver;
use shared::protocol::{RoundPhase, S2CGoldBroadcast, S2CGoldUpdate};
use shared::session::PlayerId;

use crate::card_animations::cancel_tween_anim_in_place;
use crate::state::{ClientState, CurrentClientPhase};
use crate::ui::shared::{BoardLayout, HudObjectiveUpdate};

pub const HUD_DOT_ROWS: usize = 2;
pub const HUD_DOTS_PER_ROW: usize = 5;
pub const HUD_ENTITY_COUNT: usize = 18;

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

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManaDisplayState {
    pub current_mana: u32,
    pub mana_cap: u32,
    pub reserve_mana: u32,
    pub is_populated: bool,
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
            .init_resource::<CurrentClientPhase>()
            .init_resource::<HudConfig>()
            .init_resource::<HudMode>()
            .add_message::<HudObjectiveUpdate>()
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
                    handle_gold_broadcast_system
                        .in_set(HudSystemSet::MessageDrain)
                        .before(handle_gold_update_system),
                    handle_gold_update_system.in_set(HudSystemSet::MessageDrain),
                    handle_hud_objective_update_system.in_set(HudSystemSet::MessageDrain),
                    sync_gold_text_system.in_set(HudSystemSet::StateSync),
                    sync_mana_text_system.in_set(HudSystemSet::StateSync),
                    sync_scoreboard_dot_layout_system.in_set(HudSystemSet::StateSync),
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
    mut gold_animators: Query<(Entity, &mut TweenAnim), With<GoldLabelOwner>>,
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
            cancel_hud_gold_tweens(&mut commands, &mut gold_animators);
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
    let mana_label = spawn_mana_label(
        &mut commands,
        root,
        "HUD Mana Label",
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
            ManaDisplayState::default(),
            Text::new("-- / --"),
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
            hud_text_font(18.0 * 0.65),
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
                        left: Val::Px(0.0),
                        width: Val::Px(config.hud_dot_diameter_px),
                        height: Val::Px(config.hud_dot_diameter_px),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(config.hud_dot_diameter_px * 0.5)),
                        ..default()
                    },
                    BackgroundColor(alive_dot_fill()),
                    BorderColor::all(alive_dot_border()),
                    Visibility::Hidden,
                    ChildOf(parent),
                ))
                .id()
        })
    })
}

pub fn handle_gold_broadcast_system(
    mode: Res<HudMode>,
    player_ids: Option<Res<HudPlayerIds>>,
    mut receivers: Query<&mut MessageReceiver<S2CGoldBroadcast>>,
    mut gold_labels: Query<(&GoldLabelOwner, &mut GoldDisplayState)>,
) {
    if *mode == HudMode::Frozen {
        drain_gold_broadcasts(&mut receivers);
        return;
    }

    let Some(player_ids) = player_ids else {
        drain_gold_broadcasts(&mut receivers);
        return;
    };

    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            let reserved_gold = clamped_reserved_gold(&message);
            for (owner, mut state) in &mut gold_labels {
                match (*owner, message.player_id) {
                    (GoldLabelOwner::Opponent, player_id)
                        if player_id == player_ids.opponent_id =>
                    {
                        state.gold = message.gold as f32;
                        state.reserved_gold = reserved_gold;
                        state.is_populated = true;
                    }
                    (GoldLabelOwner::Local, player_id) if player_id == player_ids.local_id => {
                        state.reserved_gold = reserved_gold;
                    }
                    _ => {}
                }
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

pub fn handle_gold_update_system(
    mode: Res<HudMode>,
    mut receivers: Query<&mut MessageReceiver<S2CGoldUpdate>>,
    mut gold_labels: Query<(&GoldLabelOwner, &mut GoldDisplayState)>,
    mut mana_labels: Query<&mut ManaDisplayState, With<ManaLabel>>,
) {
    if *mode == HudMode::Frozen {
        drain_gold_updates(&mut receivers);
        return;
    }

    let mut last_update = None;
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            last_update = Some(message);
        }
    }

    let Some(message) = last_update else {
        return;
    };

    if message.mana_cap == 0 {
        warn!("HUD: mana_cap=0 received - server invariant violated");
    }

    let Ok(mut mana_state) = mana_labels.single_mut() else {
        return;
    };

    for (owner, mut state) in &mut gold_labels {
        if *owner == GoldLabelOwner::Local {
            apply_gold_update_message(&message, &mut state, &mut mana_state);
        }
    }
}

fn drain_gold_broadcasts(receivers: &mut Query<&mut MessageReceiver<S2CGoldBroadcast>>) {
    for mut receiver in receivers.iter_mut() {
        for _message in receiver.receive() {}
    }
}

fn drain_gold_updates(receivers: &mut Query<&mut MessageReceiver<S2CGoldUpdate>>) {
    for mut receiver in receivers.iter_mut() {
        for _message in receiver.receive() {}
    }
}

pub fn apply_gold_update_batch<I>(
    messages: I,
    own_gold: &mut GoldDisplayState,
    mana_state: &mut ManaDisplayState,
) -> Option<S2CGoldUpdate>
where
    I: IntoIterator<Item = S2CGoldUpdate>,
{
    let message = messages.into_iter().last()?;
    apply_gold_update_message(&message, own_gold, mana_state);
    Some(message)
}

pub fn apply_gold_update_message(
    message: &S2CGoldUpdate,
    own_gold: &mut GoldDisplayState,
    mana_state: &mut ManaDisplayState,
) {
    own_gold.gold = message.gold as f32;
    own_gold.is_populated = true;
    mana_state.current_mana = message.current_mana;
    mana_state.mana_cap = message.mana_cap as u32;
    mana_state.reserve_mana = message.reserve_mana;
    mana_state.is_populated = true;
}

pub fn sync_gold_text_system(
    mode: Res<HudMode>,
    mut gold_labels: Query<
        (&GoldDisplayState, &mut Text, Option<&Children>),
        Changed<GoldDisplayState>,
    >,
    mut spans: Query<&mut TextSpan>,
) {
    for (state, mut text, children) in &mut gold_labels {
        text.0 = format_gold_text(state);

        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut span) = spans.get_mut(child) {
                    span.0 = format_reserved_gold_span(state, *mode);
                }
            }
        }
    }
}

pub fn sync_mana_text_system(
    mut mana_labels: Query<
        (&ManaDisplayState, &mut Text),
        (With<ManaLabel>, Changed<ManaDisplayState>),
    >,
    mut reserve_labels: Query<
        (&mut Text, &mut Visibility),
        (With<ReserveManaLabel>, Without<ManaLabel>),
    >,
) {
    let Ok((state, mut mana_text)) = mana_labels.single_mut() else {
        return;
    };
    let Ok((mut reserve_text, mut reserve_visibility)) = reserve_labels.single_mut() else {
        return;
    };

    if !state.is_populated {
        mana_text.0 = "-- / --".to_string();
        reserve_text.0.clear();
        *reserve_visibility = Visibility::Hidden;
        return;
    }

    mana_text.0 = format!("{} / {}", state.current_mana, state.mana_cap);

    if state.reserve_mana > 0 {
        reserve_text.0 = format!("+{} reserve", state.reserve_mana);
        *reserve_visibility = Visibility::Visible;
    } else {
        reserve_text.0.clear();
        *reserve_visibility = Visibility::Hidden;
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

fn display_reserved_gold(state: &GoldDisplayState) -> u32 {
    let gold = state.gold.max(0.0) as u32;
    let reserved_gold = state.reserved_gold.max(0.0) as u32;
    reserved_gold.min(gold)
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

fn cancel_hud_gold_tweens(
    commands: &mut Commands,
    animators: &mut Query<(Entity, &mut TweenAnim), With<GoldLabelOwner>>,
) {
    for (entity, mut animator) in animators.iter_mut() {
        if let Err(error) = cancel_tween_anim_in_place(&mut animator) {
            warn!("Failed to cancel HUD gold tween on entity {entity:?}: {error}");
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
