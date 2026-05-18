use std::time::Duration;

use bevy::ecs::change_detection::Mut;
use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy_tweening::{
    lens::Lens, AnimationSystem, PlaybackState, Tween, TweenAnim, TweenState, TweeningPlugin,
};
use lightyear::prelude::MessageReceiver;
use shared::card::ClassId;
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
use crate::state::{ClientPhaseView, ClientSessionIdentity, ClientState, CurrentClientPhase};
use crate::ui::design_tokens::{overlays, spacing, strips, typography, z_layers};
use crate::ui::lobby::LobbyViewState;
use crate::ui::shared::{BoardLayout, HudObjectiveUpdate};

pub const HUD_DOT_ROWS: usize = 2;
pub const HUD_DOTS_PER_ROW: usize = 5;
/// Total HUD entities carrying the `HudEntity` marker.
/// PAW-004: +2 for figurine + timer bar (19 → 21).
/// S10-POLISH-001: +1 for the RESOLUTION dim overlay (21 → 22).
/// S14-HUD-OPP-FIGURINE: +1 for opponent figurine (22 → 23).
/// S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139): +1 for the
/// HUD phase-timer **numeric** countdown text label (23 → 24). The bar
/// alone was not legible as a remaining-time signal (UI-1129-06 /
/// AUDIT-1131-12).
///
/// PROMPT 1027: the per-pill prefix labels (PHASE / ROUND / GOLD / OPP / MANA)
/// and their structural pill containers are intentionally **not** tagged
/// `HudEntity`. They are read-only decorations that ride on the
/// `Visibility::Inherited` chain from the HUD root and do not participate in
/// the prepooled-entity contract that downstream systems rely on. Keeping
/// them outside the `HudEntity` count preserves the 24-entity invariant
/// without inflating it.
pub const HUD_ENTITY_COUNT: usize = 24;
/// Alpha applied to the RESOLUTION dim overlay's BackgroundColor — visibly
/// dims the underlying HUD without obscuring gold/mana/phase readouts.
/// Recorded in production/qa/evidence/sprint-10-hud-chrome-evidence.md.
///
/// Sprint 14 story 006 (`S12-TD-UI-OVERLAY-ALPHA-TOKEN-001`) routes this
/// constant through the canonical [`overlays::OVERLAY_DIM_ALPHA`] token
/// so the value is owned by `client/src/ui/design_tokens/overlays.rs`.
/// The [`HUD_DIM_OVERLAY_ALPHA`] name is preserved as a grep-stable
/// alias for consumer code; the *value* is the design-token module's.
pub const HUD_DIM_OVERLAY_ALPHA: f32 = overlays::OVERLAY_DIM_ALPHA;
/// Max pixel width of the HUD phase timer bar fill (matches spawn dimensions).
/// `sync_hud_timer_bar_system` scales `Node.width` from 0 up to this value
/// based on `PhaseTimerState` remaining ratio.
pub const HUD_PHASE_TIMER_BAR_MAX_WIDTH_PX: f32 = 200.0;
pub const CURRENT_MANA_BAR_WIDTH_PX: f32 = 104.0;
pub const CURRENT_MANA_BAR_HEIGHT_PX: f32 = 28.0;
pub const RESERVE_MANA_DIAMOND_SIZE_PX: f32 = 74.0;
pub const RESERVE_MANA_DIAMOND_ROTATION_DEGREES: f32 = 45.0;
/// Accessibility floor for HUD secondary resource text (phase label,
/// round counter, current mana, reserve mana). Held intentionally
/// independent of the [`typography`] scale because the integration
/// test at `tests/integration/hud/text_size_contrast_accessibility_test.rs`
/// asserts the rendered font ≥ this floor — it is a regression *floor*,
/// not a typography token. Sprint 14 story 003 leaves this constant
/// unchanged.
pub const HUD_RESOURCE_TEXT_MIN_SIZE_PX: f32 = 20.0;
/// Accessibility floor for HUD primary gold readout. Same role as
/// [`HUD_RESOURCE_TEXT_MIN_SIZE_PX`] but for the gold readout; kept
/// independent of the typography scale for the same reason.
pub const HUD_GOLD_TEXT_MIN_SIZE_PX: f32 = 40.0;
/// HUD primary gold readout font size. Sprint 14 story 003 routes this
/// through the [`typography::DISPLAY`] design token (40 px); equal to
/// [`HUD_GOLD_TEXT_MIN_SIZE_PX`] so the accessibility regression test
/// remains satisfied. An inline `const_assert` in
/// `client/src/ui/design_tokens/typography.rs` guards this invariant.
pub const HUD_GOLD_FONT_SIZE_PX: f32 = typography::DISPLAY;
/// HUD reserved-gold readout font size. Sprint 14 story 003 routes this
/// through the [`typography::H1`] design token (30 px); previously a
/// bare 26 px literal. Reserved gold remains visibly smaller than the
/// primary gold readout while sitting one semantic level above the
/// resource readouts.
pub const HUD_RESERVED_GOLD_FONT_SIZE_PX: f32 = typography::H1;
/// HUD secondary readout font size (phase / round / current mana /
/// reserve mana). Sprint 14 story 003 routes this through the
/// [`typography::H2`] design token (22 px); previously aliased to
/// [`HUD_RESOURCE_TEXT_MIN_SIZE_PX`] (20 px). H2 sits 2 px above the
/// accessibility floor so the regression test continues to pass.
pub const HUD_SECONDARY_FONT_SIZE_PX: f32 = typography::H2;
/// HUD pill-prefix label font size (PROMPT 1027). Smaller than the value
/// font sitting next to it so the prefix reads as a label, not data.
/// Routed through [`typography::H3`] (18 px) — the canonical "subhead /
/// section label" semantic level. Stays below the secondary-readout font
/// size so the prefix never visually competes with the value; stays above
/// the accessibility floor for any future floor sweep that audits every
/// HUD readout.
pub const HUD_PILL_PREFIX_FONT_SIZE_PX: f32 = typography::H3;
pub const HUD_TEXT_BACKGROUND_COLOR: Color = Color::srgba(0.04, 0.07, 0.12, 1.0);
pub const HUD_PRIMARY_TEXT_COLOR: Color = Color::srgba(0.96, 0.98, 1.0, 1.0);
pub const HUD_GOLD_TEXT_COLOR: Color = Color::srgba(1.0, 0.82, 0.28, 1.0);
pub const HUD_RESERVED_GOLD_TEXT_COLOR: Color = Color::srgba(0.95, 0.90, 0.70, 0.65);
/// PROMPT 1027 — colour for the static pill-prefix labels ("PHASE",
/// "ROUND", "GOLD", "OPP", "MANA"). Softer than [`HUD_PRIMARY_TEXT_COLOR`]
/// so the prefix reads as a label, not data — the brighter value text
/// remains the visual anchor of each pill.
pub const HUD_PILL_PREFIX_TEXT_COLOR: Color = Color::srgba(0.70, 0.78, 0.90, 0.90);
// Sprint 14 story 004 (S11-TD-UI-FLEX-STRIPS) — per-module `_GAP_PX`
// magic constants `HUD_GOLD_ROW_GAP_PX = 48.0` and
// `HUD_SECONDARY_ROW_GAP_PX = 28.0` (PROMPT 802 §3.9 G2) are deleted in
// favour of the `spacing` design-token recompositions documented at
// each call site. AC7 grep guard at
// `tests/integration/ui_clean_pass/strips_test.rs` enforces no
// surviving `_GAP_PX` identifier.

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
    pub top_strip: Entity,
    pub bottom_strip: Entity,
    pub phase_pill: Entity,
    pub phase_prefix: Entity,
    pub phase_label: Entity,
    pub round_pill: Entity,
    pub round_prefix: Entity,
    pub round_counter: Entity,
    pub own_gold_pill: Entity,
    pub own_gold_prefix: Entity,
    pub own_gold_parent: Entity,
    pub own_gold_span: Entity,
    pub opponent_gold_pill: Entity,
    pub opponent_gold_prefix: Entity,
    pub opponent_gold_parent: Entity,
    pub opponent_gold_span: Entity,
    pub mana_pill: Entity,
    pub mana_prefix: Entity,
    pub mana_label: Entity,
    pub reserve_container: Entity,
    pub reserve_label: Entity,
    pub figurine: Entity,
    pub opponent_figurine: Entity,
    pub timer_bar: Entity,
    pub timer_countdown: Entity,
    pub dim_overlay: Entity,
    pub dots: [[Entity; HUD_DOTS_PER_ROW]; HUD_DOT_ROWS],
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudTopStrip;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudBottomStrip;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[deprecated(
    since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001",
    note = "Universal HUD marker is too coarse for QA snapshot counts (SOURCE-1077-08). \
            Use the per-sub-surface root markers — HudTopStripRoot, HudBottomStripRoot, \
            HudScoreboardDotRoot, HudDimOverlayRoot — for visibility-aware counting. \
            The deprecated marker remains on existing entities for one Sprint cycle so \
            historical PROMPT 1022 / 1034 / 1036 snapshot comparisons still resolve."
)]
pub struct HudEntity;

/// S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 — per-sub-surface root marker for the
/// HUD top strip (`HeaderBar` primitive at viewport top). Lives on the same
/// entity as [`HudTopStrip`] and rides the HUD root's `Visibility` chain via
/// `Visibility::Inherited`, so a `Visibility::Visible` filter in
/// [`crate::presentation::qa_snapshot::UiCountQueries`] reports `1` when the
/// HUD is shown and `0` otherwise.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudTopStripRoot;

/// S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 — per-sub-surface root marker for the
/// HUD bottom strip (`FooterBar` primitive at viewport bottom). Lives on the
/// same entity as [`HudBottomStrip`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudBottomStripRoot;

/// S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 — per-sub-surface root marker for the
/// HUD scoreboard dot row entities. Applied to each scoreboard dot so the
/// visible-count signal advances from 0 (HUD hidden) to
/// `HUD_DOT_ROWS * HUD_DOTS_PER_ROW` (HUD visible).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudScoreboardDotRoot;

/// S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 — per-sub-surface root marker for the
/// HUD RESOLUTION-phase dim overlay. Lives on the same entity as
/// [`HudDimOverlay`]; visibility flips per phase.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudDimOverlayRoot;

/// PROMPT 1027 — marker for HUD pill containers (flex parents that
/// group a prefix label with its value entity). Structural only —
/// intentionally NOT tagged `HudEntity` so the pre-pooled
/// `HUD_ENTITY_COUNT` contract is preserved.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudPillContainer;

/// PROMPT 1027 — marker for HUD pill-prefix text labels ("PHASE",
/// "ROUND", "GOLD", "OPP", "MANA"). Structural only — intentionally
/// NOT tagged `HudEntity` so the pre-pooled `HUD_ENTITY_COUNT`
/// contract is preserved.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudPillPrefixLabel;

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

/// Marker for HUD class figurine entities.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudFigurine;

/// Marker for the opponent HUD class figurine entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpponentFigurineMarker;

/// S17-UI-HUD-OPP-MANA-CLEANUP — HUD-local mirror of the revealed class
/// identity for each player. Populated by
/// [`sync_class_reveal_from_lobby_view_system`] (reads
/// `LobbyViewState.revealed_classes`) and by
/// [`sync_class_reveal_from_snapshot_system`] (reads
/// `PresentationGameSnapshotMessage`); consumed by
/// [`sync_class_reveal_hud_system`] in `HudSystemSet::StateSync` to re-skin
/// the opponent figurine `ImageNode` and the OPP value text after the server
/// broadcasts `S2CClassesRevealed` (AUDIT-1076-10 + AUDIT-1076-16). Honours
/// the FROZEN-on-GAME_OVER contract: while `HudMode::Frozen`, only the
/// snapshot path is allowed to overwrite this resource (matches Sprint 14
/// story 017 AC6 + TR-HUD-009 binding).
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudClassReveal {
    pub local: Option<ClassId>,
    pub opponent: Option<ClassId>,
}

/// Marker for the HUD phase timer bar fill entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudTimerBar;

/// S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139) — marker
/// for the HUD phase-timer **numeric** countdown text label sitting beside
/// the timer bar. Renders the remaining seconds (`"45s"` style) so the
/// player has a scannable time-left signal even when the bar fill is
/// difficult to read against the surrounding chrome (UI-1129-06).
///
/// Visibility is driven by [`sync_hud_timer_countdown_text_system`] off the
/// same [`PhaseTimerState`] that powers [`HudTimerBar`]: hidden while the
/// timer is inactive (duration_ms == 0), visible otherwise. The countdown
/// is a passive surface — it never writes to [`PhaseTimerState`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudTimerCountdown;

/// Display state for the HUD phase timer bar.
///
/// `duration_ms` is the phase budget echoed by `S2CPhaseChanged.timer_duration_ms`;
/// `elapsed_ms` advances each frame while `active` is true. The fill width is
/// `(1 - elapsed_ms / duration_ms) * HUD_PHASE_TIMER_BAR_MAX_WIDTH_PX`; when
/// `active` is false (duration_ms == 0) the bar is hidden.
///
/// Reset on every `ClientPhaseView` change by `reset_phase_timer_system`;
/// advanced by `tick_phase_timer_system`; reflected onto the entity by
/// `sync_hud_timer_bar_system`.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseTimerState {
    pub elapsed_ms: u32,
    pub duration_ms: u32,
    pub active: bool,
}

/// Marker for the HUD RESOLUTION-phase dim/freeze overlay root entity.
/// Visibility is governed solely by `sync_dim_overlay_for_resolution_system`
/// reading `Res<CurrentClientPhase>`. The overlay is pre-pooled at HUD
/// session entry — never spawned or despawned per phase transition.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudDimOverlay;

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
        tracing::info!("HudPlugin loaded");
        if !app.is_plugin_added::<TweeningPlugin>() {
            app.add_plugins(TweeningPlugin);
        }

        app.init_resource::<CurrentClientPhase>()
            .init_resource::<ClientPhaseView>()
            .init_resource::<PlayerEconomyView>()
            .init_resource::<HudConfig>()
            .init_resource::<HudMode>()
            .init_resource::<PhaseTimerState>()
            .init_resource::<HudClassReveal>()
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
                    reset_phase_timer_system.in_set(HudSystemSet::PhaseTransition),
                    tick_phase_timer_system
                        .in_set(HudSystemSet::MessageDrain)
                        .after(reset_phase_timer_system),
                    sync_hud_timer_bar_system
                        .in_set(HudSystemSet::StateSync)
                        .after(tick_phase_timer_system),
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
                    sync_dim_overlay_for_resolution_system.in_set(HudSystemSet::StateSync),
                    sync_class_reveal_from_lobby_view_system.in_set(HudSystemSet::MessageDrain),
                    sync_class_reveal_from_snapshot_system
                        .in_set(HudSystemSet::MessageDrain)
                        .after(handle_game_snapshot_system),
                    sync_class_reveal_hud_system
                        .in_set(HudSystemSet::StateSync)
                        .after(sync_gold_text_system)
                        .after(sync_figurine_image_system),
                    // S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR
                    // (PROMPT 1139) — the two new systems are nested
                    // into a sub-tuple so the outer tuple stays under
                    // the Bevy-0.18 20-element `IntoScheduleConfigs`
                    // arity ceiling. Bevy flattens nested tuples
                    // transparently; ordering constraints are identical
                    // to what a flat 21-element tuple would express.
                    (
                        sync_hud_timer_countdown_text_system
                            .in_set(HudSystemSet::StateSync)
                            .after(tick_phase_timer_system),
                        // Membership in `HudSystemSet::StateSync` is the
                        // only ordering constraint we publish for this
                        // system. An `.after(sync_dot_image_on_objective_destroyed_system)`
                        // would trip Bevy 0.18's "tried to order against
                        // a SystemTypeSet that has more than one
                        // instance" guard because the
                        // `hud_asset_wiring_test` integration suite
                        // re-registers the destroyed-asset sync inside
                        // its own test schedule. Both systems write
                        // identical assets when both fire on the same
                        // frame, so the ordering is correctness-neutral.
                        sync_scoreboard_dot_image_for_state_system
                            .in_set(HudSystemSet::StateSync)
                            .after(handle_game_snapshot_system),
                    ),
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
                GoldLabelOwner::Local,
                &gold_states,
                &mut gold_texts,
                &mut gold_spans,
            );
            sync_gold_label_for_mode(
                entities.opponent_gold_parent,
                entities.opponent_gold_span,
                HudMode::EconomyBasic,
                GoldLabelOwner::Opponent,
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
                GoldLabelOwner::Local,
                &gold_states,
                &mut gold_texts,
                &mut gold_spans,
            );
            sync_gold_label_for_mode(
                entities.opponent_gold_parent,
                entities.opponent_gold_span,
                HudMode::EconomyAuction,
                GoldLabelOwner::Opponent,
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
                GoldLabelOwner::Local,
                &gold_states,
                &mut gold_texts,
                &mut gold_spans,
            );
            sync_gold_label_for_mode(
                entities.opponent_gold_parent,
                entities.opponent_gold_span,
                render_mode,
                GoldLabelOwner::Opponent,
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
            z_layers::UI_BASE,
        ))
        .id();

    #[cfg(feature = "ui_picking")]
    commands.entity(root).insert(Pickable {
        should_block_lower: false,
        is_hoverable: false,
    });

    // Sprint 14 story 015: the HeaderBar primitive is the structural
    // HUD top-strip parent. It is not tagged `HudEntity`, so the
    // pre-pooled HUD entity-count contract remains aligned with the
    // gameplay readouts and downstream systems.
    let top_strip = commands
        .spawn((
            Name::new("HUD Top Strip"),
            HudTopStrip,
            HudTopStripRoot,
            strips::HeaderBar,
            hud_top_strip_node(),
            Visibility::Inherited,
            ChildOf(root),
            z_layers::UI_BASE,
        ))
        .id();
    let bottom_strip = commands
        .spawn((
            Name::new("HUD Bottom Strip"),
            HudBottomStrip,
            HudBottomStripRoot,
            strips::FooterBar,
            hud_bottom_strip_node(*config),
            Visibility::Inherited,
            ChildOf(root),
            z_layers::UI_BASE,
        ))
        .id();

    // PROMPT 1027 — wrap each top-strip readout in a pill container with a
    // short static prefix label so phase / round / gold / opp / mana are
    // each scannable as a labelled chunk instead of a row of bare values.
    // Pill containers and prefix labels are structural decorations: they
    // are NOT tagged `HudEntity`, so `HUD_ENTITY_COUNT` stays at 23 and the
    // prepooled-entity contract is preserved.
    let phase_pill = spawn_pill_container(&mut commands, top_strip, "HUD Phase Pill");
    let phase_prefix = spawn_pill_prefix(&mut commands, phase_pill, "HUD Phase Prefix", "PHASE");
    let phase_label = spawn_text_label(
        &mut commands,
        phase_pill,
        "HUD Phase Label",
        "",
        PhaseLabel,
        top_strip_text_node(),
    );
    let round_pill = spawn_pill_container(&mut commands, top_strip, "HUD Round Pill");
    let round_prefix = spawn_pill_prefix(&mut commands, round_pill, "HUD Round Prefix", "ROUND");
    let round_counter = spawn_text_label(
        &mut commands,
        round_pill,
        "HUD Round Counter",
        "",
        RoundCounter,
        top_strip_text_node(),
    );
    let own_gold_pill = spawn_pill_container(&mut commands, top_strip, "HUD Own Gold Pill");
    let own_gold_prefix =
        spawn_pill_prefix(&mut commands, own_gold_pill, "HUD Own Gold Prefix", "GOLD");
    let (own_gold_parent, own_gold_span) = spawn_gold_label(
        &mut commands,
        own_gold_pill,
        "HUD Own Gold",
        GoldLabelOwner::Local,
    );
    // The HudTopStrip flex parent owns inter-readout spacing through
    // `spacing::SPACING_XL + spacing::SPACING_MD`; individual gold
    // labels no longer carry per-line absolute offsets.
    let opponent_gold_pill =
        spawn_pill_container(&mut commands, top_strip, "HUD Opponent Gold Pill");
    let opponent_gold_prefix = spawn_pill_prefix(
        &mut commands,
        opponent_gold_pill,
        "HUD Opponent Gold Prefix",
        "OPP",
    );
    let (opponent_gold_parent, opponent_gold_span) = spawn_gold_label(
        &mut commands,
        opponent_gold_pill,
        "HUD Opponent Gold",
        GoldLabelOwner::Opponent,
    );
    let mana_pill = spawn_pill_container(&mut commands, top_strip, "HUD Mana Pill");
    let mana_prefix = spawn_pill_prefix(&mut commands, mana_pill, "HUD Mana Prefix", "MANA");
    let mana_label = spawn_mana_label(
        &mut commands,
        mana_pill,
        "HUD Mana Label",
        current_mana_bar_node(),
    );
    let (reserve_container, reserve_label) = spawn_reserve_mana_label(&mut commands, top_strip);

    // ── PAW-004: class figurine (own player) ──────────────────────────────────
    // Spawned with fallback; updated to the correct class asset in StateSync
    // when the first S2CGameSnapshot arrives and own ClassId is known.
    // Sprint 14 story 016: the figurine is a flex child of HudBottomStrip so
    // future bottom readouts can share the same structural parent.
    let figurine = commands
        .spawn((
            Name::new("HUD Class Figurine"),
            HudEntity,
            HudFigurine,
            bottom_strip_figurine_node(),
            ImageNode::new(fallback_handle.clone()),
            Visibility::Hidden,
            ChildOf(bottom_strip),
        ))
        .id();
    let opponent_figurine = commands
        .spawn((
            Name::new("HUD Opponent Class Figurine"),
            HudEntity,
            HudFigurine,
            OpponentFigurineMarker,
            bottom_strip_figurine_node(),
            ImageNode::new(fallback_handle.clone()),
            Visibility::Hidden,
            ChildOf(bottom_strip),
        ))
        .id();

    // ── PAW-004: phase timer bar fill ─────────────────────────────────────────
    // Image is static; only Node width changes to represent timer progress.
    let timer_bar_image = if let Some(server) = &asset_server {
        ImageNode::new(server.load(HUD_PHASE_TIMER_BAR_ASSET))
    } else {
        ImageNode::new(Handle::default())
    };
    // Sprint 14 story 004: previously `top: hud_margin + 48.0` magic
    // offset. Replaced with HeaderBar-relative anchoring: the timer bar
    // sits immediately below the canonical HeaderBar strip footprint,
    // so `top = HEADER_BAR_HEIGHT_PX` (60) expressed via the strip
    // token rather than a per-module magic. Default hud_margin (12) +
    // 48 = 60 = HEADER_BAR_HEIGHT_PX — same pixel value, now derived
    // from `docs/ux/global-ui-design-spec.md` §9.
    let timer_bar = commands
        .spawn((
            Name::new("HUD Phase Timer Bar"),
            HudEntity,
            HudTimerBar,
            top_strip_timer_bar_node(),
            timer_bar_image,
            Visibility::Hidden,
            ChildOf(top_strip),
        ))
        .id();

    // S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139) — numeric
    // remaining-seconds readout. Sits in the top strip as a structural
    // sibling of the timer bar so users have a scannable countdown for
    // every timed phase (Placement / DraftShop / DraftInitial). The bar
    // fill alone was not legible as a remaining-time signal
    // (UI-1129-06). Visibility is governed by
    // `sync_hud_timer_countdown_text_system` reading `PhaseTimerState`.
    let timer_countdown = commands
        .spawn((
            Name::new("HUD Phase Timer Countdown"),
            HudEntity,
            HudTimerCountdown,
            Text::new(""),
            hud_text_font(HUD_SECONDARY_FONT_SIZE_PX),
            TextColor(HUD_PRIMARY_TEXT_COLOR),
            BackgroundColor(HUD_TEXT_BACKGROUND_COLOR),
            top_strip_text_node(),
            Visibility::Hidden,
            ChildOf(top_strip),
        ))
        .id();

    // ── S10-POLISH-001: RESOLUTION dim/freeze overlay ─────────────────────────
    // Pre-pooled at session entry — visibility flips via
    // sync_dim_overlay_for_resolution_system reading Res<CurrentClientPhase>.
    // Full-viewport translucent Node, child of root, spawned hidden.
    let dim_overlay = commands
        .spawn((
            Name::new("HUD Resolution Dim Overlay"),
            HudEntity,
            HudDimOverlay,
            HudDimOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, HUD_DIM_OVERLAY_ALPHA)),
            Visibility::Hidden,
            ChildOf(root),
            z_layers::UI_OVERLAY,
        ))
        .id();

    #[cfg(feature = "ui_picking")]
    commands.entity(dim_overlay).insert(Pickable {
        should_block_lower: false,
        is_hoverable: false,
    });

    let dots = spawn_scoreboard_dots(&mut commands, asset_server.as_deref(), root, &config);

    commands.insert_resource(HudEntities {
        root,
        top_strip,
        bottom_strip,
        phase_pill,
        phase_prefix,
        phase_label,
        round_pill,
        round_prefix,
        round_counter,
        own_gold_pill,
        own_gold_prefix,
        own_gold_parent,
        own_gold_span,
        opponent_gold_pill,
        opponent_gold_prefix,
        opponent_gold_parent,
        opponent_gold_span,
        mana_pill,
        mana_prefix,
        mana_label,
        reserve_container,
        reserve_label,
        figurine,
        opponent_figurine,
        timer_bar,
        timer_countdown,
        dim_overlay,
        dots,
    });
}

fn despawn_hud(mut commands: Commands, entities: Option<Res<HudEntities>>) {
    if let Some(entities) = entities {
        commands.entity(entities.root).despawn();
        commands.remove_resource::<HudEntities>();
    }
}

/// PROMPT 1027 — spawn a structural flex-row pill container under
/// `parent`. The container groups a prefix label with its value entity
/// so each top-strip readout reads as a labelled chunk. Returns the
/// container entity so callers can spawn children into it.
fn spawn_pill_container(commands: &mut Commands, parent: Entity, name: &'static str) -> Entity {
    commands
        .spawn((
            Name::new(name),
            HudPillContainer,
            pill_container_node(),
            // No BackgroundColor — the container is a flex grouping only;
            // the value entity inside still carries its own pill background.
            Visibility::Inherited,
            ChildOf(parent),
        ))
        .id()
}

/// PROMPT 1027 — spawn a static short prefix label ("PHASE", "ROUND",
/// "GOLD", "OPP", "MANA") inside a pill container. Structural only,
/// not `HudEntity`-tagged so the pre-pooled `HUD_ENTITY_COUNT`
/// invariant is preserved. Visibility rides on
/// `Visibility::Inherited` so the prefix follows the HUD root
/// visibility chain without participating in `set_hud_visible`.
fn spawn_pill_prefix(
    commands: &mut Commands,
    parent: Entity,
    name: &'static str,
    text: &'static str,
) -> Entity {
    commands
        .spawn((
            Name::new(name),
            HudPillPrefixLabel,
            Text::new(text),
            hud_text_font(HUD_PILL_PREFIX_FONT_SIZE_PX),
            TextColor(HUD_PILL_PREFIX_TEXT_COLOR),
            pill_prefix_node(),
            Visibility::Inherited,
            ChildOf(parent),
        ))
        .id()
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

fn spawn_reserve_mana_label(commands: &mut Commands, parent: Entity) -> (Entity, Entity) {
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
            reserve_mana_diamond_node(),
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
) -> (Entity, Entity) {
    // PROMPT 1027 — the cold-start placeholder reflects whether this is
    // the local player's gold (loading: "--g") or the opponent's gold
    // (hidden-by-design: "?"). The PROMPT 1022 audit (F-P3-13) called out
    // that "--g" for the opponent reads as broken because it visually
    // matches the local "still loading" placeholder. A single-glyph "?"
    // clearly signals "hidden information" using existing text
    // primitives only.
    let placeholder = unpopulated_gold_placeholder(owner);
    let parent_entity = commands
        .spawn((
            Name::new(name),
            HudEntity,
            owner,
            GoldDisplayState::default(),
            GoldTweenTarget::default(),
            Text::new(placeholder),
            hud_text_font(HUD_GOLD_FONT_SIZE_PX),
            TextColor(HUD_GOLD_TEXT_COLOR),
            BackgroundColor(HUD_TEXT_BACKGROUND_COLOR),
            top_strip_gold_node(),
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
                    HudScoreboardDotRoot,
                    ScoreboardDot {
                        row: row_marker,
                        lane_index,
                    },
                    ScoreboardDotState::default(),
                    Node {
                        position_type: PositionType::Absolute,
                        // PROMPT 1027 — anchor scoreboard dots BELOW the
                        // HeaderBar instead of at `hud_margin_px` from the
                        // root top. Before this change the dots sat at
                        // y ∈ [12, 32] which is inside the 60 px HeaderBar
                        // footprint, so they rendered as small circular
                        // silhouettes behind / next to the top-strip text
                        // pills (PROMPT 1022 audit F-P1-03). Re-anchoring
                        // below the HeaderBar clears the overlap without
                        // disturbing the X projection driven by
                        // `BoardLayout::scoreboard_lane_center_x`.
                        top: Val::Px(
                            strips::HEADER_BAR_HEIGHT_PX + config.hud_margin_px + row as f32 * 20.0,
                        ),
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
            tracing::info!(
                player_id = ?message.player_id,
                gold = message.gold,
                reserved_gold = message.reserved_gold,
                msg_type = "S2CGoldBroadcast",
                "drain_gold_broadcast: recv"
            );
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

#[derive(Default)]
pub struct FigurineClassCache {
    own: Option<ClassId>,
    opponent: Option<ClassId>,
}

/// PAW-004 / S14-HUD-OPP-FIGURINE: StateSync — update the own and opponent
/// figurine `ImageNode`s from the authoritative snapshot class ids. Runs every
/// frame but only writes when the snapshot class differs from the last asset
/// written for that figurine.
pub fn sync_figurine_image_system(
    asset_server: Option<Res<AssetServer>>,
    mut figurines: Query<&mut ImageNode, With<HudFigurine>>,
    entities: Option<Res<HudEntities>>,
    mut last_classes: Local<FigurineClassCache>,
    mut snapshot_messages: MessageReader<PresentationGameSnapshotMessage>,
) {
    let mut latest_classes = None;
    for msg in snapshot_messages.read() {
        if let Some((own, opponent)) = snapshot_hud_players(&msg.0) {
            latest_classes = Some((own.class_id, opponent.class_id));
        }
    }

    let Some((own_class_id, opponent_class_id)) = latest_classes else {
        return;
    };
    let Some(entities) = entities else {
        return;
    };
    let Some(server) = asset_server else {
        return;
    };

    sync_one_figurine_image(
        &mut figurines,
        entities.figurine,
        own_class_id,
        &mut last_classes.own,
        &server,
    );
    sync_one_figurine_image(
        &mut figurines,
        entities.opponent_figurine,
        opponent_class_id,
        &mut last_classes.opponent,
        &server,
    );
}

fn sync_one_figurine_image(
    figurines: &mut Query<&mut ImageNode, With<HudFigurine>>,
    entity: Entity,
    class_id: ClassId,
    last_class: &mut Option<ClassId>,
    server: &AssetServer,
) {
    if *last_class == Some(class_id) {
        return;
    }

    if let Ok(mut img) = figurines.get_mut(entity) {
        img.image = server.load(hud_figurine_asset(class_id));
        *last_class = Some(class_id);
    }
}

/// S17-UI-HUD-OPP-MANA-CLEANUP — MessageDrain: project the canonical
/// lobby `revealed_classes` mirror into the HUD-local [`HudClassReveal`]
/// resource so the opponent figurine + OPP value text can re-skin without
/// the lobby reducer needing to gain a HUD-side dependency. Skips the
/// projection while `HudMode::Frozen` so the FROZEN-on-GAME_OVER contract
/// (Sprint 14 story 017 AC6 / TR-HUD-009) holds for incremental reveals;
/// snapshot rebuilds remain free to overwrite via
/// [`sync_class_reveal_from_snapshot_system`] (AC6 binding).
///
/// S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139,
/// UI-1129-03) — the lookup now prefers
/// [`ClientSessionIdentity::player_id`] (the handshake-assigned local
/// id) over [`LobbyViewState::local_player_id`]. The two should agree
/// in steady state; the explicit priority guards against a stale lobby
/// mirror after a reconnect handshake re-assigns the local id. If only
/// one source is populated, that source is used. If both are populated
/// and they disagree, the identity wins and a `tracing::warn!` fires.
pub fn sync_class_reveal_from_lobby_view_system(
    lobby: Option<Res<LobbyViewState>>,
    identity: Option<Res<ClientSessionIdentity>>,
    mode: Res<HudMode>,
    mut reveal: ResMut<HudClassReveal>,
) {
    if *mode == HudMode::Frozen {
        return;
    }
    let Some(lobby) = lobby else {
        return;
    };
    if lobby.revealed_classes.is_empty() {
        return;
    }
    let identity_id = identity.as_deref().and_then(|i| i.player_id);
    let lobby_id = lobby.local_player_id;
    if let (Some(identity_id), Some(lobby_id)) = (identity_id, lobby_id) {
        if identity_id != lobby_id {
            warn!(
                identity_local = ?identity_id,
                lobby_local = ?lobby_id,
                "HUD: lobby local_player_id differs from session identity; using identity for class projection (UI-1129-03 defence)"
            );
        }
    }
    let Some(local_id) = identity_id.or(lobby_id) else {
        return;
    };

    let mut local_class: Option<ClassId> = None;
    let mut opponent_class: Option<ClassId> = None;
    for (pid, class_id) in &lobby.revealed_classes {
        if *pid == local_id {
            local_class = Some(*class_id);
        } else {
            opponent_class = Some(*class_id);
        }
    }

    if let Some(c) = local_class {
        if reveal.local != Some(c) {
            reveal.local = Some(c);
        }
    }
    if let Some(c) = opponent_class {
        if reveal.opponent != Some(c) {
            reveal.opponent = Some(c);
        }
    }
}

/// S17-UI-HUD-OPP-MANA-CLEANUP — MessageDrain: project the snapshot
/// rebuild (ADR-011) class identity into [`HudClassReveal`]. Runs every
/// frame so reconnect rebuilds re-skin the opponent figurine and OPP
/// value text. Intentionally NOT gated by `HudMode::Frozen` — per AC6
/// only the snapshot path is allowed to overwrite class identity once
/// `GAME_OVER` freezes the HUD.
///
/// S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139,
/// UI-1129-03) — the projection now prefers the canonical
/// handshake-assigned local id ([`ClientSessionIdentity::player_id`],
/// falling back to [`LobbyViewState::local_player_id`]) over
/// `S2CGameSnapshot::recipient_player_id` when resolving "own" vs
/// "opponent". This defends against any flow where the snapshot's
/// recipient field would invert the mapping (the AUDIT-1129 mirror-bug
/// report observed `OPP` rendering the local player's class on both
/// clients); the snapshot recipient remains the fallback so the system
/// keeps working on test paths that never insert a
/// `ClientSessionIdentity`. A `tracing::warn!` fires on mismatch so
/// future routing regressions surface in logs.
pub fn sync_class_reveal_from_snapshot_system(
    mut messages: MessageReader<PresentationGameSnapshotMessage>,
    identity: Option<Res<ClientSessionIdentity>>,
    lobby: Option<Res<LobbyViewState>>,
    mut reveal: ResMut<HudClassReveal>,
) {
    let trusted_local_id = identity
        .as_deref()
        .and_then(|i| i.player_id)
        .or_else(|| lobby.as_deref().and_then(|l| l.local_player_id));

    for msg in messages.read() {
        let snapshot = &msg.0;
        let resolved_local = trusted_local_id.unwrap_or(snapshot.recipient_player_id);
        if let Some(trusted) = trusted_local_id {
            if trusted != snapshot.recipient_player_id {
                warn!(
                    trusted_local = ?trusted,
                    snapshot_recipient = ?snapshot.recipient_player_id,
                    "HUD: snapshot recipient differs from trusted local id; using trusted id for class projection (UI-1129-03 defence)"
                );
            }
        }

        let Some(own) = snapshot
            .players
            .iter()
            .find(|p| p.player_id == resolved_local)
        else {
            continue;
        };
        let Some(opponent) = snapshot
            .players
            .iter()
            .find(|p| p.player_id != resolved_local)
        else {
            continue;
        };

        if reveal.local != Some(own.class_id) {
            reveal.local = Some(own.class_id);
        }
        if reveal.opponent != Some(opponent.class_id) {
            reveal.opponent = Some(opponent.class_id);
        }
    }
}

/// S17-UI-HUD-OPP-MANA-CLEANUP — StateSync: apply the resolved class
/// identity from [`HudClassReveal`] to the HUD's two visible class
/// carriers: the opponent figurine `ImageNode` (AUDIT-1076-10) and the
/// opponent OPP **prefix** `Text` of the OPP pill, which previously
/// survived class reveal as the bare literal `"OPP"` alongside the
/// gold-placeholder `"?"` value (AUDIT-1076-16). The prefix is the
/// canonical place to surface class identity: the OPP pill's *value*
/// text remains the authoritative opponent-gold readout (the existing
/// `sync_gold_text_system` writes `"{gold}g"` or `"?"` there per
/// reconnect-snapshot contract `tests/integration/hud/
/// reconnect_snapshot_rebuild_test.rs`). Runs after
/// [`sync_gold_text_system`] and [`sync_figurine_image_system`] so the
/// final visible text/image reflects the revealed class. The own
/// figurine is also covered so direct-tests of the resource path do not
/// drift from the snapshot path.
///
/// FROZEN binding (AC6): this system runs every frame, but
/// [`HudClassReveal`] only changes from the lobby path when not Frozen.
/// Snapshot rebuilds still overwrite. The `Local<Option<ClassId>>`
/// caches short-circuit redundant writes.
pub fn sync_class_reveal_hud_system(
    asset_server: Option<Res<AssetServer>>,
    reveal: Res<HudClassReveal>,
    entities: Option<Res<HudEntities>>,
    mut figurines: Query<&mut ImageNode, With<HudFigurine>>,
    mut texts: Query<&mut Text>,
    mut last_local: Local<Option<ClassId>>,
    mut last_opponent: Local<Option<ClassId>>,
) {
    let Some(entities) = entities else {
        return;
    };

    if let Some(opp_class) = reveal.opponent {
        if let Some(server) = asset_server.as_deref() {
            if *last_opponent != Some(opp_class) {
                if let Ok(mut img) = figurines.get_mut(entities.opponent_figurine) {
                    img.image = server.load(hud_figurine_asset(opp_class));
                }
            }
        }
        if let Ok(mut text) = texts.get_mut(entities.opponent_gold_prefix) {
            let display = format_opp_class_display(opp_class);
            if text.0 != display {
                text.0 = display;
            }
        }
        *last_opponent = Some(opp_class);
    }

    if let Some(own_class) = reveal.local {
        if let Some(server) = asset_server.as_deref() {
            if *last_local != Some(own_class) {
                if let Ok(mut img) = figurines.get_mut(entities.figurine) {
                    img.image = server.load(hud_figurine_asset(own_class));
                }
            }
        }
        *last_local = Some(own_class);
    }
}

/// S17-UI-HUD-OPP-MANA-CLEANUP — combined OPP-pill prefix string used by
/// [`sync_class_reveal_hud_system`] once `S2CClassesRevealed` lands. The
/// format mirrors the Sprint 14 lobby class-picker display strings
/// (`{:?}` over `ClassId`) and embeds the static `OPP` lead so the
/// previous `OPP ?` shorthand (the prefix text alone before reveal)
/// remains scannable as e.g. `OPP Iop` after reveal even though the
/// gold value entity may still be unpopulated.
pub fn format_opp_class_display(class_id: ClassId) -> String {
    format!("OPP {:?}", class_id)
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

/// S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139,
/// UI-1129-11 / AUDIT-1131-03) — StateSync: refresh the scoreboard dot
/// `ImageNode` whenever `ScoreboardDotState` changes so the visible
/// image stays in step with the alive/destroyed split.
///
/// The opponent row is spawned with the Unknown / fog-of-war asset
/// (`HUD_OBJECTIVE_DOT_UNKNOWN_ASSET`) because no snapshot has landed
/// yet at session entry. After the first `S2CGameSnapshot`,
/// [`write_dot_destroyed`] flips `ScoreboardDotState.destroyed` to
/// `false` (alive) but historically did **not** rewrite the
/// `ImageNode`, leaving the opponent dots painted as grey skulls for
/// the entire run (UI-1129-11). This system closes that gap by
/// reading the change-detected `ScoreboardDotState` and binding the
/// image to the matching alive/destroyed asset.
///
/// `HudPlayerIds` gating: the spawn-time Unknown image must survive
/// until the *first snapshot has actually landed* (i.e. until
/// `handle_game_snapshot_system` has inserted `HudPlayerIds`).
/// `ScoreboardDotState::default()` is already `{ destroyed: false }`
/// at spawn so a naive `Changed` filter would repaint the dots to the
/// Alive asset on the very first update tick, defeating the
/// fog-of-war placeholder. Gating on `HudPlayerIds` (the snapshot-only
/// resource) keeps the placeholder intact until the snapshot writes
/// the real lane states.
pub fn sync_scoreboard_dot_image_for_state_system(
    asset_server: Option<Res<AssetServer>>,
    player_ids: Option<Res<HudPlayerIds>>,
    mut dots: Query<
        (&mut ImageNode, &ScoreboardDotState),
        (With<ScoreboardDot>, Changed<ScoreboardDotState>),
    >,
) {
    let Some(server) = asset_server else {
        return;
    };
    if player_ids.is_none() {
        return;
    }
    for (mut image, state) in &mut dots {
        let asset = if state.destroyed {
            HUD_OBJECTIVE_DOT_DESTROYED_ASSET
        } else {
            crate::asset_wiring::HUD_OBJECTIVE_DOT_ALIVE_ASSET
        };
        image.image = server.load(asset);
    }
}

/// S10-POLISH-001: StateSync — flip the pre-pooled `HudDimOverlay` entity's
/// `Visibility` to `Visible` while `Phase::Resolution`, `Hidden` otherwise.
///
/// Reads `Res<CurrentClientPhase>` only (the resource populated by the existing
/// single `phase_sink_system`); never reads the `S2CPhaseChanged` receiver
/// directly, never writes to `CurrentClientPhase`, never emits a synthetic
/// `S2CPhaseChanged`. Guarantees the single-source-of-phase-truth invariant
/// (TR-HUD-006 + ADR-002).
///
/// Visibility flips are instantaneous (HUD-12b BLOCKING) — no `TweenAnim`
/// or per-frame alpha mutation.
pub fn sync_dim_overlay_for_resolution_system(
    current: Res<CurrentClientPhase>,
    entities: Option<Res<HudEntities>>,
    mut visibility: Query<&mut Visibility>,
) {
    let Some(entities) = entities else {
        return;
    };

    let target = if current.phase == RoundPhase::Resolution {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    set_visibility(&mut visibility, entities.dim_overlay, target);
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
        // S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139,
        // UI-1129-18 / AUDIT-1131-12): the shop sub-phase was collapsed
        // to the bare `DRAFT` literal, which the user could not
        // distinguish from `DRAFT INITIAL` in the top strip. Promote
        // the shop label to the explicit `DRAFT SHOP` so the three
        // draft sub-phases each carry a distinct chrome string
        // (`DRAFT INITIAL` / `DRAFT SHOP` / `AUCTION`).
        RoundPhase::DraftShop => Some("DRAFT SHOP"),
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
        &GoldLabelOwner,
        &mut GoldTweenTarget,
        &mut Text,
        Option<&Children>,
        Option<&TweenAnim>,
    )>,
    mut spans: Query<&mut TextSpan>,
) {
    for (state, owner, mut target, mut text, children, animator) in &mut gold_labels {
        if !is_hud_tween_active(animator) {
            sync_gold_tween_target_to_authoritative(state, &mut target);
        }

        let display = gold_display_state_from_target(&target);
        text.0 = format_gold_text(&display, *owner);

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

fn format_gold_text(state: &GoldDisplayState, owner: GoldLabelOwner) -> String {
    if state.is_populated {
        format!("{}g", state.gold as u32)
    } else {
        unpopulated_gold_placeholder(owner).to_string()
    }
}

/// PROMPT 1027 — single source of truth for the unpopulated-state gold
/// placeholder. Local gold uses the legacy "--g" loading placeholder
/// (the value will be replaced as soon as the first `S2CGoldUpdate`
/// arrives). Opponent gold uses "?" to signal hidden information using
/// existing text primitives only — PROMPT 1022 audit F-P3-13 / REPAIR-A3
/// disposition.
fn unpopulated_gold_placeholder(owner: GoldLabelOwner) -> &'static str {
    match owner {
        GoldLabelOwner::Local => "--g",
        GoldLabelOwner::Opponent => "?",
    }
}

/// Format the small inline reserved-gold span that hangs off the primary
/// gold readout while the auction is live.
///
/// S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139,
/// UI-1129-15) — the bare `(0r)` short-hand from PROMPT 1027 read as
/// "parenthetical zero-r" without a legend explaining `r = reserve`.
/// Replace with the explicit `(+N reserve)` wording so the suffix is
/// self-describing and reads in step with the mana microbadge
/// (`+N reserve`). Empty state remains the canonical "no auction
/// reserve" signal so this never adds whitespace outside the auction
/// mode.
fn format_reserved_gold_span(state: &GoldDisplayState, mode: HudMode) -> String {
    if mode == HudMode::EconomyAuction && state.is_populated {
        format!(" (+{} reserve)", display_reserved_gold(state))
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
            text.0 = format_gold_text(&state, *owner);
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

/// Reset `PhaseTimerState` on every `ClientPhaseView` change.
///
/// `phase_sink_system` (PresentationSet::PhaseTransition) writes
/// `ClientPhaseView.timer_duration_ms` before this system runs in
/// `HudSystemSet::PhaseTransition`, so change detection on the resource is
/// sufficient — covers both per-phase transitions and snapshot rebuilds.
pub fn reset_phase_timer_system(
    phase_view: Res<ClientPhaseView>,
    mut timer: ResMut<PhaseTimerState>,
) {
    if !phase_view.is_changed() {
        return;
    }
    timer.elapsed_ms = 0;
    timer.duration_ms = phase_view.timer_duration_ms;
    timer.active = phase_view.timer_duration_ms > 0;
}

/// Advance `PhaseTimerState.elapsed_ms` by `Time::delta()` each frame while
/// the timer is active. Saturating-clamped at `duration_ms`.
pub fn tick_phase_timer_system(time: Res<Time>, mut timer: ResMut<PhaseTimerState>) {
    if !timer.active || timer.duration_ms == 0 {
        return;
    }
    let delta_ms = u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX);
    let new_elapsed = timer.elapsed_ms.saturating_add(delta_ms);
    timer.elapsed_ms = new_elapsed.min(timer.duration_ms);
}

/// Reflect `PhaseTimerState` onto the `HudTimerBar` `Node.width` and
/// `Visibility`. Hidden when the timer is inactive (duration_ms == 0).
pub fn sync_hud_timer_bar_system(
    timer: Res<PhaseTimerState>,
    mut query: Query<(&mut Node, &mut Visibility), With<HudTimerBar>>,
) {
    if !timer.is_changed() {
        return;
    }
    let (target_width_px, target_visibility) = if timer.active && timer.duration_ms > 0 {
        let remaining = timer.duration_ms.saturating_sub(timer.elapsed_ms) as f32;
        let pct = (remaining / timer.duration_ms as f32).clamp(0.0, 1.0);
        (HUD_PHASE_TIMER_BAR_MAX_WIDTH_PX * pct, Visibility::Visible)
    } else {
        (0.0, Visibility::Hidden)
    };
    for (mut node, mut visibility) in &mut query {
        node.width = Val::Px(target_width_px);
        if *visibility != target_visibility {
            *visibility = target_visibility;
        }
    }
}

/// S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139,
/// UI-1129-06) — reflect [`PhaseTimerState`] onto the
/// [`HudTimerCountdown`] text label as a remaining-seconds readout
/// (`"45s"` style). Mirrors the visibility gating of
/// [`sync_hud_timer_bar_system`]: hidden while the timer is inactive
/// (duration_ms == 0) or the HUD itself is hidden; visible otherwise
/// for every phase that publishes a non-zero `timer_duration_ms`
/// (DraftInitial / DraftShop / Placement / DraftAuction — the
/// auction modal also publishes its own internal timer in
/// `client/src/ui/shop_auction/mod.rs`; the HUD countdown stays the
/// canonical top-strip remaining-time signal for *any* phase with a
/// non-zero duration).
pub fn sync_hud_timer_countdown_text_system(
    timer: Res<PhaseTimerState>,
    mut query: Query<(&mut Text, &mut Visibility), With<HudTimerCountdown>>,
) {
    if !timer.is_changed() {
        return;
    }
    let (target_text, target_visibility) = if timer.active && timer.duration_ms > 0 {
        let remaining_ms = timer.duration_ms.saturating_sub(timer.elapsed_ms);
        // Round-up so the user never sees `0s` while time is still
        // remaining; the bar continues to drain smoothly underneath.
        let remaining_s = remaining_ms.div_ceil(1_000);
        (format!("{remaining_s}s"), Visibility::Visible)
    } else {
        (String::new(), Visibility::Hidden)
    };
    for (mut text, mut visibility) in &mut query {
        if text.0 != target_text {
            text.0 = target_text.clone();
        }
        if *visibility != target_visibility {
            *visibility = target_visibility;
        }
    }
}

fn set_hud_visible(entities: &HudEntities, visibility: &mut Query<&mut Visibility>) {
    for entity in [
        entities.root,
        entities.top_strip,
        entities.bottom_strip,
        entities.phase_label,
        entities.round_counter,
        entities.own_gold_parent,
        entities.own_gold_span,
        entities.opponent_gold_parent,
        entities.opponent_gold_span,
        entities.mana_label,
        entities.figurine,
        entities.opponent_figurine,
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
    owner: GoldLabelOwner,
    gold_states: &Query<&GoldDisplayState>,
    gold_texts: &mut Query<&mut Text>,
    gold_spans: &mut Query<&mut TextSpan>,
) {
    let state = gold_states.get(parent).ok();

    if let (Some(state), Ok(mut text)) = (state, gold_texts.get_mut(parent)) {
        text.0 = format_gold_text(state, owner);
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

fn hud_top_strip_node() -> Node {
    // PROMPT 1183 (HUD-OVERLAY-RESPONSIVE-LAYOUT-REPAIR) — the previous
    // `SPACING_XL + SPACING_MD` (48 px) inter-pill column gap deviated
    // from the canonical "cluster-to-cluster" spacing in
    // `docs/ux/global-ui-design-spec.md` §9 (which ratifies `SPACING_MD`
    // / 16 px for HUD strip readouts) and pushed the seven-pill strip
    // content past the 1280 px minimum-viewport budget. Reducing it to
    // `SPACING_MD` keeps every readout legible while freeing ~192 px of
    // horizontal slack so the strip fits at 1280×720 / 1366×768.
    //
    // `row_gap` stays at `SPACING_XL - SPACING_XS` (28 px) to honour the
    // pre-existing Sprint 14 story 004 / AC3 source-grep contract
    // (`tests/integration/ui_clean_pass/strips_test.rs::
    // ac3_hud_secondary_row_offset_resolves_through_spacing_tokens`).
    // It is a no-op on a single-row flex parent, so the recomposition
    // trail is preserved without affecting layout.
    let mut node = strips::header_bar_node();
    node.padding = UiRect::horizontal(Val::Px(spacing::SPACING_LG));
    node.column_gap = Val::Px(spacing::SPACING_MD);
    node.row_gap = Val::Px(spacing::SPACING_XL - spacing::SPACING_XS);
    node.min_height = Val::Px(strips::HEADER_BAR_HEIGHT_PX);
    node.overflow = Overflow::visible();
    node
}

fn hud_bottom_strip_node(config: HudConfig) -> Node {
    let mut node = strips::footer_bar_node();
    node.padding.left = Val::Px(config.hud_margin_px);
    node.padding.right = Val::Px(spacing::SPACING_LG);
    node.column_gap = Val::Px(spacing::SPACING_LG);
    node.row_gap = Val::Px(spacing::SPACING_SM);
    node.overflow = Overflow::visible();
    node
}

fn bottom_strip_figurine_node() -> Node {
    Node {
        width: Val::Px(64.0),
        height: Val::Px(64.0),
        min_width: Val::Px(64.0),
        min_height: Val::Px(64.0),
        flex_shrink: 0.0,
        ..default()
    }
}

fn top_strip_text_node() -> Node {
    Node {
        padding: UiRect::axes(Val::Px(spacing::SPACING_SM), Val::Px(spacing::SPACING_XS)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

/// PROMPT 1027 — flex-row container that groups a pill prefix label
/// ("PHASE" / "ROUND" / "GOLD" / "OPP" / "MANA") with its value entity.
/// The inter-element gap is `spacing::SPACING_SM` so the prefix reads
/// as part of the same chunk as the value while still being visually
/// distinguishable.
fn pill_container_node() -> Node {
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexStart,
        column_gap: Val::Px(spacing::SPACING_SM),
        ..default()
    }
}

/// PROMPT 1027 — node for the static prefix label inside a pill. Small
/// horizontal padding keeps it readable without crowding the value.
fn pill_prefix_node() -> Node {
    Node {
        padding: UiRect::horizontal(Val::Px(spacing::SPACING_XS)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

fn top_strip_gold_node() -> Node {
    Node {
        padding: UiRect::horizontal(Val::Px(spacing::SPACING_SM)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

fn current_mana_bar_node() -> Node {
    Node {
        width: Val::Px(CURRENT_MANA_BAR_WIDTH_PX),
        height: Val::Px(CURRENT_MANA_BAR_HEIGHT_PX),
        min_width: Val::Px(CURRENT_MANA_BAR_WIDTH_PX),
        min_height: Val::Px(CURRENT_MANA_BAR_HEIGHT_PX),
        padding: UiRect::axes(Val::Px(spacing::SPACING_SM), Val::Px(spacing::SPACING_XS)),
        border: UiRect::all(Val::Px(spacing::SPACING_XS / 2.0)),
        border_radius: BorderRadius::all(Val::Px(spacing::SPACING_XS)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

fn reserve_mana_diamond_node() -> Node {
    Node {
        width: Val::Px(RESERVE_MANA_DIAMOND_SIZE_PX),
        height: Val::Px(RESERVE_MANA_DIAMOND_SIZE_PX),
        min_width: Val::Px(RESERVE_MANA_DIAMOND_SIZE_PX),
        min_height: Val::Px(RESERVE_MANA_DIAMOND_SIZE_PX),
        border: UiRect::all(Val::Px(spacing::SPACING_XS / 2.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

fn top_strip_timer_bar_node() -> Node {
    Node {
        width: Val::Px(HUD_PHASE_TIMER_BAR_MAX_WIDTH_PX),
        height: Val::Px(spacing::SPACING_SM),
        min_height: Val::Px(spacing::SPACING_SM),
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
