use bevy::prelude::*;
use lightyear::prelude::{MessageReceiver, MessageSender};
use shared::protocol::{
    C2SAcknowledgeResult, GameOverReason, ObjectiveSnapshot, OpponentObjectiveSnapshot,
    ReliableChannel, RoundPhase, S2CGameOver, S2CGameSnapshot,
};
use shared::session::PlayerId;

use crate::presentation::{PresentationGameSnapshotMessage, PresentationSet};
use crate::state::{
    ClientIdempotencyState, ClientSessionIdentity, ClientState, CurrentClientPhase,
    GameOverDedupeKey,
};
use crate::ui::settings::AccessibilityPreferences;

const OBJECTIVE_LANES: usize = 5;
const FOCUS_BORDER: Color = Color::srgb(0.93, 0.82, 0.28);
const IDLE_BORDER: Color = Color::srgba(0.82, 0.86, 0.9, 0.35);

pub struct ResultScreenPlugin;

impl Plugin for ResultScreenPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("ResultScreenPlugin loaded");
        app.init_resource::<CurrentClientPhase>()
            .init_resource::<ClientSessionIdentity>()
            .init_resource::<AccessibilityPreferences>()
            .init_resource::<ResultScreenViewState>()
            .init_resource::<ResultScreenReturnToLobbyState>()
            .init_resource::<ResultScreenFocusOrder>()
            .init_resource::<ResultScreenMotionState>()
            .init_resource::<ResultScreenOutboundMessages>()
            // S13-LATE-MSG-DEDUPE-001: ensure the dedupe ring exists even when
            // tests load this plugin in isolation (PresentationPlugin would
            // normally install ClientIdempotencyPlugin first).
            .init_resource::<ClientIdempotencyState>()
            .add_message::<PresentationGameSnapshotMessage>()
            .add_message::<ResultScreenActionRequest>()
            .add_systems(OnEnter(ClientState::InSession), spawn_result_screen_system)
            .add_systems(OnExit(ClientState::InSession), despawn_result_screen_system)
            .add_systems(
                Update,
                (
                    result_screen_phase_transition_system
                        .in_set(PresentationSet::PhaseTransition)
                        .after(super::phase_sink_system),
                    drain_result_screen_game_over_receiver_system
                        .in_set(PresentationSet::MessageDrain),
                    cache_result_screen_snapshot_system
                        .in_set(PresentationSet::MessageDrain)
                        .after(super::game_snapshot_sink_system),
                    result_screen_button_interaction_system,
                    result_screen_keyboard_input_system,
                    handle_result_screen_actions_system
                        .in_set(PresentationSet::StateSync)
                        .before(sync_result_screen_ui_system),
                    sync_result_screen_ui_system.in_set(PresentationSet::StateSync),
                )
                    .run_if(in_state(ClientState::InSession)),
            );
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ResultScreenViewState {
    pub cached_result: Option<S2CGameOver>,
    pub cached_snapshot: Option<S2CGameSnapshot>,
    pub visible: bool,
    pub snapshot_game_over_seen: bool,
}

impl ResultScreenViewState {
    pub fn clear_for_lobby(&mut self) {
        *self = Self::default();
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ResultScreenOutboundMessages {
    pub acknowledgements: Vec<C2SAcknowledgeResult>,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResultScreenReturnToLobbyState {
    pub return_requested: bool,
    pub acknowledgement_sent: bool,
    pub local_cleanup_completed: bool,
}

impl ResultScreenReturnToLobbyState {
    pub fn reset_for_session(&mut self) {
        *self = Self::default();
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ResultScreenFocusOrder {
    entities: Vec<Entity>,
    focused_index: usize,
}

impl Default for ResultScreenFocusOrder {
    fn default() -> Self {
        Self {
            entities: Vec::new(),
            focused_index: 0,
        }
    }
}

impl ResultScreenFocusOrder {
    pub fn set_entities(&mut self, entities: Vec<Entity>) {
        self.entities = entities;
        if self.focused_index >= self.entities.len() {
            self.focused_index = 0;
        }
    }

    pub fn focus_next(&mut self) {
        if !self.entities.is_empty() {
            self.focused_index = (self.focused_index + 1) % self.entities.len();
        }
    }

    pub fn focus_return_to_lobby(&mut self) {
        self.focused_index = 0;
    }

    pub fn focused_entity(&self) -> Option<Entity> {
        self.entities.get(self.focused_index).copied()
    }

    pub fn is_focused(&self, entity: Entity) -> bool {
        self.focused_entity() == Some(entity)
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResultScreenMotionState {
    pub reduced_motion: bool,
    pub entry_duration_ms: u16,
    pub row_sequencing_enabled: bool,
    pub flash_count_per_second: u8,
}

impl Default for ResultScreenMotionState {
    fn default() -> Self {
        Self {
            reduced_motion: false,
            entry_duration_ms: 140,
            row_sequencing_enabled: false,
            flash_count_per_second: 0,
        }
    }
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultScreenActionRequest {
    ReturnToLobby,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct ResultScreenEntities {
    pub root: Entity,
    pub headline: Entity,
    pub cause: Entity,
    pub summary: Entity,
    pub own_rows: [Entity; OBJECTIVE_LANES],
    pub opponent_rows: [Entity; OBJECTIVE_LANES],
    pub return_button: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenRoot;

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenPanel;

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenHeadline;

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenCause;

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenSummaryText;

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenOwnObjectiveRow {
    pub lane: u8,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenOpponentObjectiveRow {
    pub lane: u8,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenReturnToLobbyButton;

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenFocusIndicator;

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenRematchButton;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultScreenOutcomeCopy {
    pub headline: String,
    pub cause: String,
    pub has_result: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultObjectiveIdentity {
    Real,
    Fake,
    Unknown,
    Unavailable,
}

impl ResultObjectiveIdentity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Real => "Real",
            Self::Fake => "Fake",
            Self::Unknown => "Unknown",
            Self::Unavailable => "Unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultObjectiveState {
    Alive,
    Destroyed,
    Unavailable,
}

impl ResultObjectiveState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Alive => "Alive",
            Self::Destroyed => "Destroyed",
            Self::Unavailable => "Unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResultObjectiveRow {
    pub lane: u8,
    pub hp: Option<u8>,
    pub state: ResultObjectiveState,
    pub identity: ResultObjectiveIdentity,
}

impl ResultObjectiveRow {
    pub fn unavailable(lane: u8) -> Self {
        Self {
            lane,
            hp: None,
            state: ResultObjectiveState::Unavailable,
            identity: ResultObjectiveIdentity::Unavailable,
        }
    }

    pub fn label(self) -> String {
        let hp = self
            .hp
            .map(|value| value.to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        format!(
            "L{} {} {} HP {}",
            self.lane,
            self.identity.label(),
            self.state.label(),
            hp
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultObjectiveSummary {
    pub own_rows: [ResultObjectiveRow; OBJECTIVE_LANES],
    pub opponent_rows: [ResultObjectiveRow; OBJECTIVE_LANES],
}

pub fn result_screen_outcome_copy(
    result: Option<&S2CGameOver>,
    local_player_id: Option<PlayerId>,
) -> ResultScreenOutcomeCopy {
    let Some(result) = result else {
        return ResultScreenOutcomeCopy {
            headline: "RESULT PENDING".to_string(),
            cause: "Result data is unavailable. No winner or reason was declared.".to_string(),
            has_result: false,
        };
    };

    match result.reason {
        GameOverReason::ObjectivesDestroyed => match result.loser {
            Some(loser) if Some(loser) == local_player_id => ResultScreenOutcomeCopy {
                headline: "DEFEAT".to_string(),
                cause: "Two of your real objectives were destroyed.".to_string(),
                has_result: true,
            },
            Some(_) if local_player_id.is_some() => ResultScreenOutcomeCopy {
                headline: "VICTORY".to_string(),
                cause: "Opponent lost two real objectives.".to_string(),
                has_result: true,
            },
            None => ResultScreenOutcomeCopy {
                headline: "DRAW".to_string(),
                cause: "Both players lost real objectives in the same resolution.".to_string(),
                has_result: true,
            },
            Some(_) => ResultScreenOutcomeCopy {
                headline: "RESULT PENDING".to_string(),
                cause: "Result received, but local player identity is unavailable.".to_string(),
                has_result: true,
            },
        },
        GameOverReason::Disconnect => match result.loser {
            Some(loser) if Some(loser) == local_player_id => ResultScreenOutcomeCopy {
                headline: "DEFEAT".to_string(),
                cause: "Your connection was lost beyond the grace window.".to_string(),
                has_result: true,
            },
            Some(_) if local_player_id.is_some() => ResultScreenOutcomeCopy {
                headline: "VICTORY".to_string(),
                cause: "Opponent connection was lost beyond the grace window.".to_string(),
                has_result: true,
            },
            None => ResultScreenOutcomeCopy {
                headline: "DRAW".to_string(),
                cause: "Both players lost connection; no winner was declared.".to_string(),
                has_result: true,
            },
            Some(_) => ResultScreenOutcomeCopy {
                headline: "RESULT PENDING".to_string(),
                cause: "Result received, but local player identity is unavailable.".to_string(),
                has_result: true,
            },
        },
        GameOverReason::Draw => ResultScreenOutcomeCopy {
            headline: "DRAW".to_string(),
            cause: "No winner was declared.".to_string(),
            has_result: true,
        },
        GameOverReason::ResolutionTimeout => ResultScreenOutcomeCopy {
            headline: "NO RESULT".to_string(),
            cause: "Resolution timed out without declaring a winner.".to_string(),
            has_result: true,
        },
    }
}

pub fn build_result_objective_summary(
    snapshot: Option<&S2CGameSnapshot>,
) -> ResultObjectiveSummary {
    let Some(snapshot) = snapshot else {
        return empty_objective_summary();
    };

    let local_player = snapshot
        .players
        .iter()
        .find(|player| player.player_id == snapshot.recipient_player_id);
    let opponent_player = snapshot
        .players
        .iter()
        .find(|player| player.player_id != snapshot.recipient_player_id);

    let own_rows = std::array::from_fn(|index| {
        let lane = (index + 1) as u8;
        local_player
            .and_then(|player| {
                player
                    .objectives
                    .iter()
                    .find(|objective| objective.lane == lane)
            })
            .map(own_objective_row)
            .unwrap_or_else(|| ResultObjectiveRow::unavailable(lane))
    });

    let opponent_rows = std::array::from_fn(|index| {
        let lane = (index + 1) as u8;
        let public_objective = local_player.and_then(|player| {
            player
                .opponent_objectives
                .iter()
                .find(|objective| objective.lane == lane)
        });
        let fallback_objective = opponent_player.and_then(|player| {
            player
                .objectives
                .iter()
                .find(|objective| objective.lane == lane)
        });

        opponent_objective_row(lane, public_objective, fallback_objective)
    });

    ResultObjectiveSummary {
        own_rows,
        opponent_rows,
    }
}

pub fn result_screen_summary_text(
    result: Option<&S2CGameOver>,
    snapshot: Option<&S2CGameSnapshot>,
) -> String {
    let summary = build_result_objective_summary(snapshot);
    let own_real_lost = summary
        .own_rows
        .iter()
        .filter(|row| {
            row.identity == ResultObjectiveIdentity::Real
                && row.state == ResultObjectiveState::Destroyed
        })
        .count();
    let own_fake_lost = summary
        .own_rows
        .iter()
        .filter(|row| {
            row.identity == ResultObjectiveIdentity::Fake
                && row.state == ResultObjectiveState::Destroyed
        })
        .count();
    let opponent_real_revealed = summary
        .opponent_rows
        .iter()
        .filter(|row| {
            row.identity == ResultObjectiveIdentity::Real
                && row.state == ResultObjectiveState::Destroyed
        })
        .count();
    let opponent_fake_revealed = summary
        .opponent_rows
        .iter()
        .filter(|row| {
            row.identity == ResultObjectiveIdentity::Fake
                && row.state == ResultObjectiveState::Destroyed
        })
        .count();

    let round = result
        .map(|value| value.round)
        .or_else(|| snapshot.map(|value| value.round_number))
        .map(|value| format!("R{value}"))
        .unwrap_or_else(|| "Unknown".to_string());

    let resources = snapshot
        .and_then(|value| {
            value
                .players
                .iter()
                .find(|player| player.player_id == value.recipient_player_id)
        })
        .map(|player| {
            format!(
                "Gold {} | Mana {} / {} | Reserve {}",
                player.gold, player.current_mana, player.mana_cap, player.reserve_mana
            )
        })
        .unwrap_or_else(|| "Resources Unknown".to_string());

    format!(
        "Round {round} | {resources} | Own real lost {own_real_lost} | Own fake lost {own_fake_lost} | Opponent real revealed {opponent_real_revealed} | Opponent fake revealed {opponent_fake_revealed}"
    )
}

pub fn result_screen_motion_state(
    preferences: &AccessibilityPreferences,
) -> ResultScreenMotionState {
    if preferences.reduced_motion {
        ResultScreenMotionState {
            reduced_motion: true,
            entry_duration_ms: 0,
            row_sequencing_enabled: false,
            flash_count_per_second: 0,
        }
    } else {
        ResultScreenMotionState::default()
    }
}

fn spawn_result_screen_system(
    mut commands: Commands,
    mut return_state: ResMut<ResultScreenReturnToLobbyState>,
) {
    return_state.reset_for_session();

    let root = commands
        .spawn((
            Name::new("Result screen root"),
            ResultScreenRoot,
            Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.025, 0.035, 0.46)),
            Visibility::Hidden,
            GlobalZIndex(100),
        ))
        .id();

    let panel = commands
        .spawn((
            Name::new("Result screen panel"),
            ResultScreenPanel,
            ChildOf(root),
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(88.0),
                max_width: Val::Px(860.0),
                max_height: Val::Percent(92.0),
                row_gap: Val::Px(14.0),
                padding: UiRect::all(Val::Px(26.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.055, 0.062, 0.078, 0.94)),
            BorderColor::all(Color::srgba(0.82, 0.86, 0.9, 0.26)),
        ))
        .id();

    let headline = spawn_result_text(
        &mut commands,
        panel,
        "RESULT PENDING",
        36.0,
        Color::srgb(0.96, 0.97, 0.99),
        Some(ResultScreenHeadline),
    );
    let cause = spawn_result_text(
        &mut commands,
        panel,
        "Result data is unavailable. No winner or reason was declared.",
        18.0,
        Color::srgb(0.82, 0.86, 0.9),
        Some(ResultScreenCause),
    );

    let summary = spawn_result_text(
        &mut commands,
        panel,
        "Round Unknown | Resources Unknown | Objectives Unknown",
        15.0,
        Color::srgb(0.74, 0.79, 0.84),
        Some(ResultScreenSummaryText),
    );

    let objective_grid = commands
        .spawn((
            Name::new("Result objective grid"),
            ChildOf(panel),
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(16.0),
                row_gap: Val::Px(12.0),
                ..default()
            },
        ))
        .id();

    let own_column = spawn_objective_column(&mut commands, objective_grid, "Your Objectives");
    let opponent_column =
        spawn_objective_column(&mut commands, objective_grid, "Opponent Objectives");

    let own_rows = std::array::from_fn(|index| {
        let lane = (index + 1) as u8;
        spawn_objective_row(
            &mut commands,
            own_column,
            ResultScreenOwnObjectiveRow { lane },
            &format!("L{lane} Unavailable Unavailable HP Unknown"),
        )
    });

    let opponent_rows = std::array::from_fn(|index| {
        let lane = (index + 1) as u8;
        spawn_objective_row(
            &mut commands,
            opponent_column,
            ResultScreenOpponentObjectiveRow { lane },
            &format!("L{lane} Unavailable Unavailable HP Unknown"),
        )
    });

    let actions = commands
        .spawn((
            Name::new("Result screen actions"),
            ChildOf(panel),
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                column_gap: Val::Px(10.0),
                margin: UiRect {
                    top: Val::Px(8.0),
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    let return_button = commands
        .spawn((
            Name::new("Return to lobby button"),
            ResultScreenReturnToLobbyButton,
            ChildOf(actions),
            Button,
            Interaction::None,
            Node {
                min_width: Val::Px(176.0),
                height: Val::Px(46.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect {
                    left: Val::Px(18.0),
                    right: Val::Px(18.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                },
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.14, 0.22, 0.31)),
            BorderColor::all(IDLE_BORDER),
            Text::new("Return to Lobby"),
            TextFont {
                font_size: 17.0,
                ..default()
            },
            TextColor(Color::srgb(0.96, 0.97, 0.99)),
        ))
        .id();

    commands.insert_resource(ResultScreenEntities {
        root,
        headline,
        cause,
        summary,
        own_rows,
        opponent_rows,
        return_button,
    });
}

fn despawn_result_screen_system(
    mut commands: Commands,
    entities: Option<Res<ResultScreenEntities>>,
    mut view_state: ResMut<ResultScreenViewState>,
    mut focus_order: ResMut<ResultScreenFocusOrder>,
) {
    if let Some(entities) = entities {
        commands.entity(entities.root).despawn();
        commands.remove_resource::<ResultScreenEntities>();
    }
    view_state.clear_for_lobby();
    focus_order.set_entities(Vec::new());
}

fn drain_result_screen_game_over_receiver_system(
    mut receivers: Query<&mut MessageReceiver<S2CGameOver>>,
    mut view_state: ResMut<ResultScreenViewState>,
    mut idempotency: ResMut<ClientIdempotencyState>,
) {
    for mut receiver in &mut receivers {
        for result in receiver.receive() {
            apply_game_over_drain(&mut idempotency, &mut view_state, result);
        }
    }
}

/// Idempotent apply for `S2CGameOver` per S13-LATE-MSG-DEDUPE-001.
///
/// On a fresh `(round, reason, loser)` key the message is logged and cached
/// into [`ResultScreenViewState::cached_result`]. On a duplicate
/// (reconnect-replay re-send) the message is logged at DEBUG and discarded
/// without mutating view state — matching the `C2SAcknowledgeResult`
/// idempotency precedent.
pub fn apply_game_over_drain(
    idempotency: &mut ClientIdempotencyState,
    view_state: &mut ResultScreenViewState,
    result: S2CGameOver,
) {
    let key = GameOverDedupeKey::from_message(&result);
    if !idempotency.game_over.check_and_insert(key) {
        tracing::debug!(
            loser = ?result.loser,
            round = result.round,
            reason = ?result.reason,
            msg_type = "S2CGameOver",
            "drain_result_screen_game_over: duplicate; no-op"
        );
        return;
    }

    tracing::info!(
        loser = ?result.loser,
        round = result.round,
        reason = ?result.reason,
        msg_type = "S2CGameOver",
        "drain_result_screen_game_over: recv"
    );
    view_state.cached_result = Some(result);
}

fn cache_result_screen_snapshot_system(
    mut reader: MessageReader<PresentationGameSnapshotMessage>,
    mut view_state: ResMut<ResultScreenViewState>,
) {
    for snapshot in reader.read() {
        if snapshot.0.phase == RoundPhase::GameOver {
            view_state.snapshot_game_over_seen = true;
        }
        view_state.cached_snapshot = Some(snapshot.0.clone());
    }
}

fn result_screen_phase_transition_system(
    current_phase: Res<CurrentClientPhase>,
    preferences: Res<AccessibilityPreferences>,
    return_state: Res<ResultScreenReturnToLobbyState>,
    mut view_state: ResMut<ResultScreenViewState>,
    mut motion_state: ResMut<ResultScreenMotionState>,
) {
    if return_state.return_requested {
        view_state.visible = false;
        return;
    }

    let snapshot_game_over = view_state
        .cached_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.phase == RoundPhase::GameOver);
    let should_show = current_phase.phase == RoundPhase::GameOver
        || view_state.snapshot_game_over_seen
        || snapshot_game_over;

    if should_show {
        view_state.visible = true;
        *motion_state = result_screen_motion_state(&preferences);
    } else {
        view_state.visible = false;
    }
}

fn result_screen_button_interaction_system(
    view_state: Res<ResultScreenViewState>,
    mut actions: MessageWriter<ResultScreenActionRequest>,
    interactions: Query<
        &Interaction,
        (Changed<Interaction>, With<ResultScreenReturnToLobbyButton>),
    >,
) {
    if !view_state.visible {
        return;
    }

    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            actions.write(ResultScreenActionRequest::ReturnToLobby);
        }
    }
}

fn result_screen_keyboard_input_system(
    input: Option<Res<ButtonInput<KeyCode>>>,
    view_state: Res<ResultScreenViewState>,
    mut focus_order: ResMut<ResultScreenFocusOrder>,
    mut actions: MessageWriter<ResultScreenActionRequest>,
) {
    if !view_state.visible {
        return;
    }

    let Some(input) = input else {
        return;
    };

    if input.just_pressed(KeyCode::Tab) {
        focus_order.focus_next();
    }

    if input.just_pressed(KeyCode::Escape) {
        focus_order.focus_return_to_lobby();
    }

    if input.just_pressed(KeyCode::Enter) || input.just_pressed(KeyCode::Space) {
        actions.write(ResultScreenActionRequest::ReturnToLobby);
    }
}

fn handle_result_screen_actions_system(
    mut actions: MessageReader<ResultScreenActionRequest>,
    mut senders: Query<&mut MessageSender<C2SAcknowledgeResult>>,
    mut outbound_messages: ResMut<ResultScreenOutboundMessages>,
    mut view_state: ResMut<ResultScreenViewState>,
    mut return_state: ResMut<ResultScreenReturnToLobbyState>,
    mut focus_order: ResMut<ResultScreenFocusOrder>,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    let mut return_requested = false;
    for action in actions.read() {
        if *action == ResultScreenActionRequest::ReturnToLobby {
            return_requested = true;
        }
    }

    if !return_requested {
        return;
    }

    return_state.return_requested = true;

    if !return_state.acknowledgement_sent {
        let acknowledgement = C2SAcknowledgeResult {};
        for mut sender in &mut senders {
            tracing::info!(
                msg_type = "C2SAcknowledgeResult",
                handler = "drain_result_screen_action_requests_system",
                "c2s_send: enter"
            );
            sender.send::<ReliableChannel>(acknowledgement.clone());
        }
        outbound_messages.acknowledgements.push(acknowledgement);
        return_state.acknowledgement_sent = true;
    }

    if !return_state.local_cleanup_completed {
        view_state.clear_for_lobby();
        focus_order.set_entities(Vec::new());
        return_state.local_cleanup_completed = true;
    }

    next_state.set(ClientState::Lobby);
}

fn sync_result_screen_ui_system(
    mut commands: Commands,
    entities: Option<Res<ResultScreenEntities>>,
    view_state: Res<ResultScreenViewState>,
    identity: Res<ClientSessionIdentity>,
    mut focus_order: ResMut<ResultScreenFocusOrder>,
    mut visibility_query: Query<&mut Visibility>,
    mut text_query: Query<&mut Text>,
    mut border_query: Query<&mut BorderColor>,
    focused_query: Query<Entity, With<ResultScreenFocusIndicator>>,
) {
    let Some(entities) = entities else {
        return;
    };

    if let Ok(mut visibility) = visibility_query.get_mut(entities.root) {
        *visibility = if view_state.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if !view_state.visible {
        focus_order.set_entities(Vec::new());
        clear_focus_indicators(&mut commands, focused_query, &mut border_query);
        return;
    }

    let local_player_id = identity.player_id.or_else(|| {
        view_state
            .cached_snapshot
            .as_ref()
            .map(|snapshot| snapshot.recipient_player_id)
    });
    let copy = result_screen_outcome_copy(view_state.cached_result.as_ref(), local_player_id);
    let summary = build_result_objective_summary(view_state.cached_snapshot.as_ref());
    let summary_text = result_screen_summary_text(
        view_state.cached_result.as_ref(),
        view_state.cached_snapshot.as_ref(),
    );

    set_text(&mut text_query, entities.headline, &copy.headline);
    set_text(&mut text_query, entities.cause, &copy.cause);
    set_text(&mut text_query, entities.summary, &summary_text);

    for (entity, row) in entities.own_rows.iter().zip(summary.own_rows.iter()) {
        set_text(&mut text_query, *entity, &row.label());
    }
    for (entity, row) in entities
        .opponent_rows
        .iter()
        .zip(summary.opponent_rows.iter())
    {
        set_text(&mut text_query, *entity, &row.label());
    }

    focus_order.set_entities(vec![entities.return_button]);
    sync_focus_indicator(
        &mut commands,
        focused_query,
        &mut border_query,
        focus_order.focused_entity(),
    );
}

fn spawn_result_text<M: Component>(
    commands: &mut Commands,
    parent: Entity,
    value: &str,
    font_size: f32,
    color: Color,
    marker: Option<M>,
) -> Entity {
    let mut entity_commands = commands.spawn((
        ChildOf(parent),
        Text::new(value),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
        Node {
            width: Val::Percent(100.0),
            ..default()
        },
    ));

    if let Some(marker) = marker {
        entity_commands.insert(marker);
    }

    entity_commands.id()
}

fn spawn_objective_column(commands: &mut Commands, parent: Entity, title: &str) -> Entity {
    let column = commands
        .spawn((
            Name::new(format!("Result {title}")),
            ChildOf(parent),
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                min_width: Val::Px(270.0),
                flex_grow: 1.0,
                row_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .id();

    spawn_result_text(
        commands,
        column,
        title,
        16.0,
        Color::srgb(0.91, 0.93, 0.96),
        None::<ResultScreenSummaryText>,
    );

    column
}

fn spawn_objective_row<M: Component>(
    commands: &mut Commands,
    parent: Entity,
    marker: M,
    value: &str,
) -> Entity {
    commands
        .spawn((
            ChildOf(parent),
            marker,
            Text::new(value),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.78, 0.83, 0.87)),
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(24.0),
                ..default()
            },
        ))
        .id()
}

fn own_objective_row(objective: &ObjectiveSnapshot) -> ResultObjectiveRow {
    ResultObjectiveRow {
        lane: objective.lane,
        hp: Some(objective.hp),
        state: if objective.is_destroyed {
            ResultObjectiveState::Destroyed
        } else {
            ResultObjectiveState::Alive
        },
        identity: if objective.is_real {
            ResultObjectiveIdentity::Real
        } else {
            ResultObjectiveIdentity::Fake
        },
    }
}

fn opponent_objective_row(
    lane: u8,
    public_objective: Option<&OpponentObjectiveSnapshot>,
    fallback_objective: Option<&ObjectiveSnapshot>,
) -> ResultObjectiveRow {
    if let Some(objective) = public_objective {
        return ResultObjectiveRow {
            lane,
            hp: Some(objective.hp),
            state: if objective.is_destroyed {
                ResultObjectiveState::Destroyed
            } else {
                ResultObjectiveState::Alive
            },
            identity: if objective.is_destroyed {
                objective
                    .was_fake
                    .map(|was_fake| {
                        if was_fake {
                            ResultObjectiveIdentity::Fake
                        } else {
                            ResultObjectiveIdentity::Real
                        }
                    })
                    .unwrap_or(ResultObjectiveIdentity::Unknown)
            } else {
                ResultObjectiveIdentity::Unknown
            },
        };
    }

    if let Some(objective) = fallback_objective {
        return ResultObjectiveRow {
            lane,
            hp: Some(objective.hp),
            state: if objective.is_destroyed {
                ResultObjectiveState::Destroyed
            } else {
                ResultObjectiveState::Alive
            },
            identity: ResultObjectiveIdentity::Unknown,
        };
    }

    ResultObjectiveRow::unavailable(lane)
}

fn empty_objective_summary() -> ResultObjectiveSummary {
    ResultObjectiveSummary {
        own_rows: std::array::from_fn(|index| ResultObjectiveRow::unavailable((index + 1) as u8)),
        opponent_rows: std::array::from_fn(|index| {
            ResultObjectiveRow::unavailable((index + 1) as u8)
        }),
    }
}

fn set_text(query: &mut Query<&mut Text>, entity: Entity, value: &str) {
    if let Ok(mut text) = query.get_mut(entity) {
        text.0.clear();
        text.0.push_str(value);
    }
}

fn clear_focus_indicators(
    commands: &mut Commands,
    focused_query: Query<Entity, With<ResultScreenFocusIndicator>>,
    border_query: &mut Query<&mut BorderColor>,
) {
    for entity in &focused_query {
        commands
            .entity(entity)
            .remove::<ResultScreenFocusIndicator>();
        if let Ok(mut border) = border_query.get_mut(entity) {
            *border = BorderColor::all(IDLE_BORDER);
        }
    }
}

fn sync_focus_indicator(
    commands: &mut Commands,
    focused_query: Query<Entity, With<ResultScreenFocusIndicator>>,
    border_query: &mut Query<&mut BorderColor>,
    focused_entity: Option<Entity>,
) {
    for entity in &focused_query {
        if Some(entity) != focused_entity {
            commands
                .entity(entity)
                .remove::<ResultScreenFocusIndicator>();
            if let Ok(mut border) = border_query.get_mut(entity) {
                *border = BorderColor::all(IDLE_BORDER);
            }
        }
    }

    if let Some(entity) = focused_entity {
        commands.entity(entity).insert(ResultScreenFocusIndicator);
        if let Ok(mut border) = border_query.get_mut(entity) {
            *border = BorderColor::all(FOCUS_BORDER);
        }
    }
}
