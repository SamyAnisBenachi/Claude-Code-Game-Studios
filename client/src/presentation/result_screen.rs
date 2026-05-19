use bevy::prelude::*;
use lightyear::prelude::{MessageReceiver, MessageSender};
use shared::card::ClassId;
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
use crate::ui::design_tokens::{overlays, typography, z_layers};
use crate::ui::settings::AccessibilityPreferences;

const OBJECTIVE_LANES: usize = 5;
const FOCUS_BORDER: Color = Color::srgb(0.93, 0.82, 0.28);
const IDLE_BORDER: Color = Color::srgba(0.82, 0.86, 0.9, 0.35);

// Outcome accent palette — Krosmaga-style hero affordance: a strongly tinted
// accent stripe + panel border that communicates outcome at a glance, distinct
// from the read-only data rows. Kept inside the result_screen module so the
// rest of the UI is not coupled to result-specific colours.
const OUTCOME_ACCENT_VICTORY: Color = Color::srgb(0.32, 0.78, 0.42);
const OUTCOME_ACCENT_DEFEAT: Color = Color::srgb(0.86, 0.32, 0.32);
const OUTCOME_ACCENT_DRAW: Color = Color::srgb(0.93, 0.78, 0.32);
const OUTCOME_ACCENT_NEUTRAL: Color = Color::srgb(0.62, 0.68, 0.76);

const CTA_PRIMARY_BG: Color = Color::srgb(0.86, 0.66, 0.22);
const CTA_PRIMARY_BORDER: Color = Color::srgba(0.96, 0.82, 0.36, 0.85);
const CTA_SECONDARY_BG: Color = Color::srgb(0.14, 0.22, 0.31);

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
            .init_resource::<ResultScreenStepState>()
            // S13-LATE-MSG-DEDUPE-001: ensure the dedupe ring exists even when
            // tests load this plugin in isolation (PresentationPlugin would
            // normally install ClientIdempotencyPlugin first).
            .init_resource::<ClientIdempotencyState>()
            .add_message::<PresentationGameSnapshotMessage>()
            .add_message::<ResultScreenActionRequest>()
            .add_message::<ResultScreenStepActionRequest>()
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
                    handle_result_screen_step_actions_system
                        .in_set(PresentationSet::StateSync)
                        .before(handle_result_screen_actions_system),
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

/// Two-step reveal action (V-P1-05): advance from the hero/outcome panel to
/// the accounting/details panel. Triggered by the Continue CTA or by Enter /
/// Space while the screen is on [`ResultScreenStep::Hero`].
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultScreenStepActionRequest {
    AdvanceToAccounting,
}

/// Which step of the two-step result reveal is currently presented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResultScreenStep {
    /// Step 1: hero panel with the outcome title and class/persona line.
    #[default]
    Hero,
    /// Step 2: accounting panel with the round/resource/objective breakdown.
    Accounting,
}

/// Persisted step state for the two-step result reveal. Reset to
/// [`ResultScreenStep::Hero`] each time the result screen is spawned, so a
/// fresh session always opens on the hero panel.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResultScreenStepState {
    pub current: ResultScreenStep,
}

impl ResultScreenStepState {
    pub fn reset_for_session(&mut self) {
        *self = Self::default();
    }

    pub fn advance_to_accounting(&mut self) {
        self.current = ResultScreenStep::Accounting;
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct ResultScreenEntities {
    pub root: Entity,
    pub panel: Entity,
    pub accent_stripe: Entity,
    pub round_chip: Entity,
    pub hero_panel: Entity,
    pub accounting_panel: Entity,
    pub headline: Entity,
    pub class_persona: Entity,
    pub cause: Entity,
    pub continue_hint: Entity,
    pub summary: Entity,
    pub resources_line: Entity,
    pub ledger_line: Entity,
    pub own_rows: [Entity; OBJECTIVE_LANES],
    pub opponent_rows: [Entity; OBJECTIVE_LANES],
    pub continue_button: Entity,
    pub return_button: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenRoot;

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenPanel;

/// Marker on the hero/outcome sub-panel rendered during
/// [`ResultScreenStep::Hero`].
#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenHeroPanel;

/// Marker on the accounting/details sub-panel rendered during
/// [`ResultScreenStep::Accounting`].
#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenAccountingPanel;

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenHeadline;

/// Marker on the class/persona text in the hero panel.
#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenClassPersona;

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

/// Marker on the Continue CTA. Shown only during [`ResultScreenStep::Hero`];
/// hidden when the screen has advanced to the accounting panel.
#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenContinueButton;

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenFocusIndicator;

#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenRematchButton;

/// Marker on the outcome-tinted accent stripe along the panel's left edge.
/// Background colour is updated each frame from the outcome accent palette
/// so VICTORY / DEFEAT / DRAW / pending read at a glance.
#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenAccentStripe;

/// Marker on the "Round N" chip rendered above the headline on the hero
/// panel. Hidden when no round number is known (no result and no snapshot).
#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenRoundChip;

/// Marker on the small caption rendered under the Continue CTA on the hero
/// step ("Press Enter to view details"). Hidden during the accounting step.
#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenContinueHint;

/// Marker on the explicit resources readout (Gold / Mana / Reserve) on the
/// accounting panel.
#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenResourcesLine;

/// Marker on the chunked "objectives lost" ledger line on the accounting
/// panel — short, scannable counts separate from the verbose summary.
#[derive(Component, Debug, Clone, Copy)]
pub struct ResultScreenLedgerLine;

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

/// Display label for the local player's class on the hero panel. Returns
/// `None` when no snapshot has cached a class for the local player; the hero
/// panel hides the class line in that case rather than inventing a value.
pub fn result_screen_class_persona_label(
    snapshot: Option<&S2CGameSnapshot>,
    local_player_id: Option<PlayerId>,
) -> Option<String> {
    let snapshot = snapshot?;
    let local = local_player_id.unwrap_or(snapshot.recipient_player_id);
    let player = snapshot
        .players
        .iter()
        .find(|player| player.player_id == local)?;
    Some(format!("Class: {}", class_id_label(player.class_id)))
}

fn class_id_label(class: ClassId) -> &'static str {
    match class {
        ClassId::Iop => "Iop",
        ClassId::Cra => "Cra",
        ClassId::Sacrier => "Sacrier",
        ClassId::Xelor => "Xelor",
        ClassId::Ecaflip => "Ecaflip",
        ClassId::Sadida => "Sadida",
        ClassId::Neutral => "Neutral",
    }
}

/// Maps an outcome headline ("VICTORY" / "DEFEAT" / "DRAW" / "RESULT PENDING"
/// / "NO RESULT") onto the outcome accent palette consumed by the hero
/// stripe and the panel border. Keeps colour selection in one place so the
/// accent stripe, border, and any future outcome-tinted chrome stay in
/// sync.
pub fn result_screen_outcome_accent(headline: &str) -> Color {
    match headline {
        "VICTORY" => OUTCOME_ACCENT_VICTORY,
        "DEFEAT" => OUTCOME_ACCENT_DEFEAT,
        "DRAW" => OUTCOME_ACCENT_DRAW,
        _ => OUTCOME_ACCENT_NEUTRAL,
    }
}

/// Round number label ("Round N") sourced from the authoritative result or
/// the cached snapshot, whichever is available. Returns `None` when no round
/// number is known — the round chip is hidden in that case rather than
/// rendering a placeholder.
pub fn result_screen_round_label(
    result: Option<&S2CGameOver>,
    snapshot: Option<&S2CGameSnapshot>,
) -> Option<String> {
    let round = result
        .map(|value| value.round)
        .or_else(|| snapshot.map(|value| value.round_number))?;
    Some(format!("Round {round}"))
}

/// Explicit resources line for the local player ("Gold N | Mana C/M |
/// Reserve R"). Returns `None` when the snapshot does not carry a player
/// entry matching the local recipient — the line is hidden in that case.
pub fn result_screen_resources_line(snapshot: Option<&S2CGameSnapshot>) -> Option<String> {
    let snapshot = snapshot?;
    let player = snapshot
        .players
        .iter()
        .find(|player| player.player_id == snapshot.recipient_player_id)?;
    Some(format!(
        "Gold {}  |  Mana {} / {}  |  Reserve {}",
        player.gold, player.current_mana, player.mana_cap, player.reserve_mana
    ))
}

/// Compact "objectives lost" ledger line for the accounting panel.
/// Surfaces own real / fake losses and opponent real / fake reveals as
/// short scannable counts — distinct from the verbose summary text which
/// remains the canonical contract for AC verification.
pub fn result_screen_ledger_line(snapshot: Option<&S2CGameSnapshot>) -> String {
    let summary = build_result_objective_summary(snapshot);
    let count = |rows: &[ResultObjectiveRow], identity: ResultObjectiveIdentity| {
        rows.iter()
            .filter(|row| {
                row.identity == identity && row.state == ResultObjectiveState::Destroyed
            })
            .count()
    };
    let own_real = count(&summary.own_rows, ResultObjectiveIdentity::Real);
    let own_fake = count(&summary.own_rows, ResultObjectiveIdentity::Fake);
    let opp_real = count(&summary.opponent_rows, ResultObjectiveIdentity::Real);
    let opp_fake = count(&summary.opponent_rows, ResultObjectiveIdentity::Fake);
    format!(
        "Objectives Lost — You: {own_real} real / {own_fake} fake   •   Opponent: {opp_real} real / {opp_fake} fake"
    )
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
    mut step_state: ResMut<ResultScreenStepState>,
) {
    return_state.reset_for_session();
    step_state.reset_for_session();

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
            BackgroundColor(Color::srgba(
                0.02,
                0.025,
                0.035,
                overlays::OVERLAY_SCRIM_ALPHA,
            )),
            Visibility::Hidden,
            z_layers::MODAL,
        ))
        .id();

    // Outer panel is now a Row container: a thin outcome-tinted accent
    // stripe along the left edge + the existing column of result content
    // along the right. The stripe makes the outcome legible at a glance
    // (Krosmaga-style hero affordance) without altering the data contract.
    let panel = commands
        .spawn((
            Name::new("Result screen panel"),
            ResultScreenPanel,
            ChildOf(root),
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                width: Val::Percent(88.0),
                max_width: Val::Px(860.0),
                max_height: Val::Percent(92.0),
                column_gap: Val::Px(18.0),
                padding: UiRect {
                    left: Val::Px(14.0),
                    right: Val::Px(26.0),
                    top: Val::Px(22.0),
                    bottom: Val::Px(22.0),
                },
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.055, 0.062, 0.078, 0.96)),
            BorderColor::all(Color::srgba(0.82, 0.86, 0.9, 0.32)),
        ))
        .id();

    let accent_stripe = commands
        .spawn((
            Name::new("Result outcome accent stripe"),
            ResultScreenAccentStripe,
            ChildOf(panel),
            Node {
                width: Val::Px(6.0),
                min_width: Val::Px(6.0),
                height: Val::Percent(100.0),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(OUTCOME_ACCENT_NEUTRAL),
        ))
        .id();

    let content = commands
        .spawn((
            Name::new("Result screen content"),
            ChildOf(panel),
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                row_gap: Val::Px(14.0),
                ..default()
            },
        ))
        .id();

    // Step 1: hero/outcome sub-panel — large title + class/persona line + cause.
    let hero_panel = commands
        .spawn((
            Name::new("Result hero panel"),
            ResultScreenHeroPanel,
            ChildOf(content),
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(10.0),
                width: Val::Percent(100.0),
                ..default()
            },
        ))
        .id();

    // Round chip — small outcome-tinted pill above the headline. Hidden when
    // no round number is available yet.
    let round_chip = commands
        .spawn((
            Name::new("Result round chip"),
            ResultScreenRoundChip,
            ChildOf(hero_panel),
            Node {
                display: Display::None,
                padding: UiRect {
                    left: Val::Px(10.0),
                    right: Val::Px(10.0),
                    top: Val::Px(3.0),
                    bottom: Val::Px(3.0),
                },
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.12, 0.16, 0.85)),
            BorderColor::all(Color::srgba(0.82, 0.86, 0.9, 0.42)),
            Text::new(""),
            TextFont {
                font_size: typography::CAPTION,
                ..default()
            },
            TextColor(Color::srgb(0.96, 0.97, 0.99)),
        ))
        .id();

    // Headline reads bigger than other surfaces — Krosmaga's outcome hero is
    // dominated by the result word. Promoted from H1 to DISPLAY so it reads
    // as the screen's hero element, distinct from headline-tier text
    // elsewhere in the client.
    let headline = spawn_result_text(
        &mut commands,
        hero_panel,
        "RESULT PENDING",
        typography::DISPLAY,
        Color::srgb(0.96, 0.97, 0.99),
        Some(ResultScreenHeadline),
    );
    let class_persona = spawn_result_text(
        &mut commands,
        hero_panel,
        "",
        typography::H2,
        Color::srgb(0.91, 0.93, 0.96),
        Some(ResultScreenClassPersona),
    );
    let cause = spawn_result_text(
        &mut commands,
        hero_panel,
        "Result data is unavailable. No winner or reason was declared.",
        typography::H3,
        Color::srgb(0.82, 0.86, 0.9),
        Some(ResultScreenCause),
    );
    let continue_hint = spawn_result_text(
        &mut commands,
        hero_panel,
        "Press Enter or Space to view round accounting.",
        typography::CAPTION,
        Color::srgba(0.74, 0.79, 0.84, 0.85),
        Some(ResultScreenContinueHint),
    );

    // Step 2: accounting sub-panel — round/resources/objectives accounting.
    let accounting_panel = commands
        .spawn((
            Name::new("Result accounting panel"),
            ResultScreenAccountingPanel,
            ChildOf(content),
            Node {
                display: Display::None,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                width: Val::Percent(100.0),
                ..default()
            },
        ))
        .id();

    // Accounting section header — readable label above the verbose summary.
    spawn_result_text(
        &mut commands,
        accounting_panel,
        "Match Accounting",
        typography::H2,
        Color::srgb(0.93, 0.95, 0.98),
        None::<ResultScreenSummaryText>,
    );

    let resources_line = spawn_result_text(
        &mut commands,
        accounting_panel,
        "",
        typography::H3,
        Color::srgb(0.93, 0.83, 0.42),
        Some(ResultScreenResourcesLine),
    );

    let ledger_line = spawn_result_text(
        &mut commands,
        accounting_panel,
        "",
        typography::BODY,
        Color::srgb(0.91, 0.93, 0.96),
        Some(ResultScreenLedgerLine),
    );

    let summary = spawn_result_text(
        &mut commands,
        accounting_panel,
        "Round Unknown | Resources Unknown | Objectives Unknown",
        typography::CAPTION,
        Color::srgb(0.74, 0.79, 0.84),
        Some(ResultScreenSummaryText),
    );

    let objective_grid = commands
        .spawn((
            Name::new("Result objective grid"),
            ChildOf(accounting_panel),
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
            ChildOf(content),
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                margin: UiRect {
                    top: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    // Continue is the primary CTA on the hero step — accent-tinted and a
    // wider hit target so the transition affordance dominates the action row.
    let continue_button = spawn_result_cta_button(
        &mut commands,
        actions,
        ResultScreenContinueButton,
        "Continue ▸",
        CTA_PRIMARY_BG,
        CTA_PRIMARY_BORDER,
        true,
    );

    // Return-to-Lobby CTA stays mounted in the actions row across both steps
    // so the dismiss path is always reachable per the V-P1-05 constraint
    // ("keep the return/dismiss CTA accessible and visible").
    let return_button = spawn_result_cta_button(
        &mut commands,
        actions,
        ResultScreenReturnToLobbyButton,
        "Return to Lobby",
        CTA_SECONDARY_BG,
        IDLE_BORDER,
        false,
    );

    commands.insert_resource(ResultScreenEntities {
        root,
        panel,
        accent_stripe,
        round_chip,
        hero_panel,
        accounting_panel,
        headline,
        class_persona,
        cause,
        continue_hint,
        summary,
        resources_line,
        ledger_line,
        own_rows,
        opponent_rows,
        continue_button,
        return_button,
    });
}

fn spawn_result_cta_button<M: Component>(
    commands: &mut Commands,
    parent: Entity,
    marker: M,
    label: &str,
    background: Color,
    border: Color,
    primary: bool,
) -> Entity {
    let (min_width, height, font_size) = if primary {
        (200.0, 50.0, typography::H2)
    } else {
        (176.0, 46.0, typography::H3)
    };
    commands
        .spawn((
            Name::new(format!("Result CTA {label}")),
            marker,
            ChildOf(parent),
            Button,
            Interaction::None,
            Node {
                min_width: Val::Px(min_width),
                height: Val::Px(height),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect {
                    left: Val::Px(20.0),
                    right: Val::Px(20.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                },
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(background),
            BorderColor::all(border),
            Text::new(label),
            TextFont {
                font_size,
                ..default()
            },
            TextColor(Color::srgb(0.96, 0.97, 0.99)),
        ))
        .id()
}

fn despawn_result_screen_system(
    mut commands: Commands,
    entities: Option<Res<ResultScreenEntities>>,
    mut view_state: ResMut<ResultScreenViewState>,
    mut focus_order: ResMut<ResultScreenFocusOrder>,
    mut step_state: ResMut<ResultScreenStepState>,
) {
    if let Some(entities) = entities {
        commands.entity(entities.root).despawn();
        commands.remove_resource::<ResultScreenEntities>();
    }
    view_state.clear_for_lobby();
    focus_order.set_entities(Vec::new());
    step_state.reset_for_session();
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
    step_state: Res<ResultScreenStepState>,
    mut actions: MessageWriter<ResultScreenActionRequest>,
    mut step_actions: MessageWriter<ResultScreenStepActionRequest>,
    return_interactions: Query<
        &Interaction,
        (Changed<Interaction>, With<ResultScreenReturnToLobbyButton>),
    >,
    continue_interactions: Query<
        &Interaction,
        (Changed<Interaction>, With<ResultScreenContinueButton>),
    >,
) {
    if !view_state.visible {
        return;
    }

    for interaction in &return_interactions {
        if *interaction == Interaction::Pressed {
            actions.write(ResultScreenActionRequest::ReturnToLobby);
        }
    }

    if step_state.current == ResultScreenStep::Hero {
        for interaction in &continue_interactions {
            if *interaction == Interaction::Pressed {
                step_actions.write(ResultScreenStepActionRequest::AdvanceToAccounting);
            }
        }
    }
}

fn result_screen_keyboard_input_system(
    input: Option<Res<ButtonInput<KeyCode>>>,
    view_state: Res<ResultScreenViewState>,
    step_state: Res<ResultScreenStepState>,
    mut focus_order: ResMut<ResultScreenFocusOrder>,
    mut actions: MessageWriter<ResultScreenActionRequest>,
    mut step_actions: MessageWriter<ResultScreenStepActionRequest>,
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
        match step_state.current {
            ResultScreenStep::Hero => {
                step_actions.write(ResultScreenStepActionRequest::AdvanceToAccounting);
            }
            ResultScreenStep::Accounting => {
                actions.write(ResultScreenActionRequest::ReturnToLobby);
            }
        }
    }
}

fn handle_result_screen_step_actions_system(
    mut step_actions: MessageReader<ResultScreenStepActionRequest>,
    mut step_state: ResMut<ResultScreenStepState>,
) {
    let mut advance_requested = false;
    for action in step_actions.read() {
        if *action == ResultScreenStepActionRequest::AdvanceToAccounting {
            advance_requested = true;
        }
    }

    if advance_requested && step_state.current == ResultScreenStep::Hero {
        step_state.advance_to_accounting();
        tracing::info!(
            from = "Hero",
            to = "Accounting",
            "result_screen_step_transition"
        );
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
    step_state: Res<ResultScreenStepState>,
    identity: Res<ClientSessionIdentity>,
    mut focus_order: ResMut<ResultScreenFocusOrder>,
    mut visibility_query: Query<&mut Visibility>,
    mut node_query: Query<&mut Node>,
    mut text_query: Query<&mut Text>,
    mut border_query: Query<&mut BorderColor>,
    mut background_query: Query<&mut BackgroundColor>,
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
    let class_persona =
        result_screen_class_persona_label(view_state.cached_snapshot.as_ref(), local_player_id);

    set_text(&mut text_query, entities.headline, &copy.headline);
    set_text(&mut text_query, entities.cause, &copy.cause);
    set_text(&mut text_query, entities.summary, &summary_text);
    set_text(
        &mut text_query,
        entities.class_persona,
        class_persona.as_deref().unwrap_or(""),
    );

    // Outcome accent stripe + panel border tint communicate VICTORY /
    // DEFEAT / DRAW at a glance. A neutral colour is applied when no result
    // is cached yet so the screen never reads as a real outcome by accident.
    let accent = result_screen_outcome_accent(&copy.headline);
    if let Ok(mut background) = background_query.get_mut(entities.accent_stripe) {
        *background = BackgroundColor(accent);
    }
    if let Ok(mut border) = border_query.get_mut(entities.panel) {
        *border = BorderColor::all(accent.with_alpha(0.42));
    }

    // Round chip — only mounted when a round number is known.
    let round_label =
        result_screen_round_label(view_state.cached_result.as_ref(), view_state.cached_snapshot.as_ref());
    set_text(
        &mut text_query,
        entities.round_chip,
        round_label.as_deref().unwrap_or(""),
    );
    set_node_display(
        &mut node_query,
        entities.round_chip,
        if round_label.is_some() {
            Display::Flex
        } else {
            Display::None
        },
    );

    // Resources line on the accounting panel — only mounted when the
    // snapshot carries a player entry for the local recipient.
    let resources_label = result_screen_resources_line(view_state.cached_snapshot.as_ref());
    set_text(
        &mut text_query,
        entities.resources_line,
        resources_label.as_deref().unwrap_or(""),
    );
    set_node_display(
        &mut node_query,
        entities.resources_line,
        if resources_label.is_some() {
            Display::Flex
        } else {
            Display::None
        },
    );

    let ledger_label = result_screen_ledger_line(view_state.cached_snapshot.as_ref());
    set_text(&mut text_query, entities.ledger_line, &ledger_label);

    // Hide the class line when the snapshot has not yet provided a class
    // rather than rendering a blank row.
    set_node_display(
        &mut node_query,
        entities.class_persona,
        if class_persona.is_some() {
            Display::Flex
        } else {
            Display::None
        },
    );

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

    // Toggle which sub-panel is mounted in the layout flow and which CTAs
    // appear in the action row based on the current step.
    let (hero_display, accounting_display, continue_display, focus_targets) =
        match step_state.current {
            ResultScreenStep::Hero => (
                Display::Flex,
                Display::None,
                Display::Flex,
                vec![entities.continue_button, entities.return_button],
            ),
            ResultScreenStep::Accounting => (
                Display::None,
                Display::Flex,
                Display::None,
                vec![entities.return_button],
            ),
        };

    set_node_display(&mut node_query, entities.hero_panel, hero_display);
    set_node_display(&mut node_query, entities.accounting_panel, accounting_display);
    set_node_display(&mut node_query, entities.continue_button, continue_display);
    // Continue hint only reads on the hero step — it advertises the Enter /
    // Space affordance for the Continue CTA.
    set_node_display(&mut node_query, entities.continue_hint, continue_display);

    focus_order.set_entities(focus_targets);
    sync_focus_indicator(
        &mut commands,
        focused_query,
        &mut border_query,
        focus_order.focused_entity(),
    );
}

fn set_node_display(query: &mut Query<&mut Node>, entity: Entity, display: Display) {
    if let Ok(mut node) = query.get_mut(entity) {
        if node.display != display {
            node.display = display;
        }
    }
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
        typography::BODY,
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
                font_size: typography::CAPTION,
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
