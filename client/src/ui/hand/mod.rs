use std::collections::BTreeSet;
use std::time::Duration;

use bevy::ecs::query::QueryFilter;
use bevy::ecs::system::SystemParam;
use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_tweening::{lens::TransformPositionLens, Tween, TweenAnim};
use lightyear::prelude::{MessageReceiver, MessageSender};
use shared::card::{CardCatalog, CardId, CardType, ClassId, Rarity};
use shared::protocol::{
    C2SActivateCard, C2SPurchaseCard, C2SSubmitPlacement, EntityId, PlacedCardSubmit,
    PlacementRejectedReason, PlayTarget, ReliableChannel, RoundPhase, S2CPlacementRejected,
};
use shared::session::PlayerId;

use crate::asset_wiring::{
    apply_card_display_art, clear_card_display_art, default_client_card_catalog,
    insert_placeholder_assets, CardDisplayArtFallback, PlaceholderAssets,
};
use crate::ui::design_tokens::card_slot::{
    card_slot_art_image_component, card_slot_art_image_node, CardSlotArtImage, CardSlotKind,
};
use crate::card_animations::{
    cancel_tween_anim_in_place, make_tween_anim, replace_tweenable, HandCard, HandDragSprite,
};
use crate::presentation::board_rendering::PlayerTeamMap;
use crate::presentation::{PlayerEconomyView, PresentationGameSnapshotMessage};
use crate::state::{ClientPhaseView, ClientSessionIdentity, ClientState, CurrentClientPhase};
use crate::ui::design_tokens::{spacing, strips, typography, z_layers};
use crate::ui::lobby::PlayerTeamMapUpdated;
use crate::ui::shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT};

pub mod drag_state_visuals;

pub const HAND_FAN_SLOT_COUNT: usize = 10;
/// Height of the absolute-positioned `HandFanRoot` strip anchored to the bottom
/// of the viewport. `metrics_for_viewport` produces fan child positions in this
/// strip's LOCAL coord space (origin = top-left of the strip), so `fan_base_y`
/// is offset down from the strip top, not from the viewport top.
pub const HAND_FAN_STRIP_HEIGHT_PX: f32 = 260.0;
pub const DRAFT_INITIAL_GRID_SLOT_COUNT: usize = 9;
pub const RESERVE_STRIP_ENTITY_COUNT: usize = 4;
// Sprint 14 story 004 (S11-TD-UI-FLEX-STRIPS): the trailing `+ 1` accounts
// for the canonical `HandBar` strip primitive spawned by `spawn_hand_ui`
// as the viewport-edge-anchored parent of `HandFanRoot`. The strip is
// tagged with `HandUiEntity` so it is despawned with the hand UI tree on
// session exit.
//
// Sprint 15 story 020 (S12-UX-HAND-DRAG-STATE-VISUALS-001): the trailing
// `+ HAND_FAN_SLOT_COUNT * 2 + 1` accounts for the per-slot drag-state
// overlay child nodes (`FanSlotDimOverlay` + `FanSlotHoverOverlay`, two
// per slot) plus the single `FanPlateDropTargetOverlay` spawned under
// `HandFanRoot`. Each overlay is tagged with `HandUiEntity` so it is
// despawned with the hand UI tree on session exit. All overlays are
// children of pre-existing pre-pooled entities (slots / fan_root); no
// new top-level pre-pool entries are introduced (ADR-021 Impl
// Guideline 5 preserved).
//
// PROMPT 1043 (Placement Action Panel + Submit Affordance P1 repair):
// the trailing `+ 4` accounts for the four new tagged entities introduced
// by the bordered placement action panel:
//   1. `PlacementActionPanel` container (parent of the disclosure /
//      timer / placed-count / submit children),
//   2. `PlacementActionPanelHeader` text label ("Placement"),
//   3. the timer row that hosts the countdown + submitted-checkmark
//      side-by-side,
//   4. `PlacedCountReadout` text ("X placed") wired to
//      `PendingPlacements.staged_count()`.
// All four are tagged with `HandUiEntity` so they are despawned with
// the hand-UI tree on session exit (ADR-021 Impl Guideline 5 preserved).
//
// PROMPT 1239 (S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001): the trailing
// `+ HAND_FAN_SLOT_COUNT * 2` accounts for the two new per-slot idle
// playable-affordance overlay children (Playable + Unaffordable, mutually
// exclusive visibility). Both are children of pre-pooled `FanSlotIndex`
// entities — no new top-level pre-pool entries (ADR-021 Impl Guideline 5).
pub const HAND_UI_ENTITY_COUNT: usize = HAND_FAN_SLOT_COUNT
    + DRAFT_INITIAL_GRID_SLOT_COUNT
    + 8
    + HAND_FAN_SLOT_COUNT * RESERVE_STRIP_ENTITY_COUNT
    + 1
    + HAND_FAN_SLOT_COUNT * 2
    + 1
    + 4
    + HAND_FAN_SLOT_COUNT * 2
    // Sprint 18 story-022 (S18-UI-CARD-ART-AND-LABEL-STRIP-001):
    // per-fan-slot `CardSlotArtImage` child. The child is parented
    // into the existing pre-pooled `FanSlotIndex` entity (ADR-021 Impl
    // Guideline 5 preserved — no new top-level pre-pool entries).
    + HAND_FAN_SLOT_COUNT;
const HAND_CARD_DISPLAY_WIDTH_PX: f32 = 96.0;
const HAND_CARD_DISPLAY_HEIGHT_PX: f32 = 136.0;
const HAND_DRAFT_GRID_CARD_WIDTH_PX: f32 = 120.0;
const HAND_DRAFT_GRID_CARD_HEIGHT_PX: f32 = 56.0;
const HAND_DRAG_SPRITE_SCALE: f32 = 1.10;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct HandFanLayoutConfig {
    pub fan_base_margin_px: f32,
    pub fan_half_spread_px: f32,
    pub arc_height_px: f32,
    pub max_rotation_deg: f32,
}

impl Default for HandFanLayoutConfig {
    fn default() -> Self {
        Self {
            fan_base_margin_px: 100.0,
            fan_half_spread_px: 280.0,
            arc_height_px: 10.0,
            max_rotation_deg: 10.0,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct HandFanViewport {
    pub width_px: f32,
    pub height_px: f32,
}

impl Default for HandFanViewport {
    fn default() -> Self {
        Self {
            width_px: 800.0,
            height_px: 600.0,
        }
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandFanLayoutState {
    pub hand_count: usize,
}

#[derive(Resource, Debug, Clone)]
pub struct HandCardCatalog {
    pub cards: CardCatalog,
}

impl Default for HandCardCatalog {
    fn default() -> Self {
        Self {
            cards: default_client_card_catalog(),
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandUiTimingConfig {
    pub card_draw_animation_ms: u64,
    pub purchase_timeout_ms: u64,
    pub hand_full_notification_duration_ms: u64,
}

impl Default for HandUiTimingConfig {
    fn default() -> Self {
        Self {
            card_draw_animation_ms: 280,
            purchase_timeout_ms: 3_000,
            hand_full_notification_duration_ms: 2_000,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementTimerConfig {
    pub placement_duration_ms: u32,
    pub urgency_threshold_ms: u32,
    pub grace_window_ms: u32,
}

impl Default for PlacementTimerConfig {
    fn default() -> Self {
        Self {
            placement_duration_ms: 10_000,
            urgency_threshold_ms: 5_000,
            grace_window_ms: 200,
        }
    }
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct HandContents {
    pub cards: Vec<CardId>,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandUiMode {
    #[default]
    Hidden,
    Grid,
    Passive,
    PassiveLocked,
    Staging,
}

impl HandUiMode {
    pub fn from_phase(phase: RoundPhase) -> Self {
        match phase {
            RoundPhase::DraftInitial => Self::Grid,
            RoundPhase::DraftShop => Self::Passive,
            RoundPhase::DraftAuction => Self::PassiveLocked,
            RoundPhase::Placement => Self::Staging,
            RoundPhase::Lobby
            | RoundPhase::Resolution
            | RoundPhase::GameOver
            | RoundPhase::Handshaking => Self::Hidden,
        }
    }

    fn shows_fan_root(self) -> bool {
        matches!(
            self,
            Self::Grid | Self::Passive | Self::PassiveLocked | Self::Staging
        )
    }

    fn shows_fan_slots(self) -> bool {
        self.shows_fan_root()
    }

    fn allows_activation(self) -> bool {
        self == Self::Passive
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct HandUiOutboundMessages {
    pub activate_cards: Vec<C2SActivateCard>,
    pub purchase_cards: Vec<C2SPurchaseCard>,
    pub submit_placements: Vec<C2SSubmitPlacement>,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PendingPlacements {
    pub placements: Vec<PlacedCardSubmit>,
}

impl PendingPlacements {
    pub fn staged_count(&self) -> usize {
        self.placements.len()
    }

    fn clear(&mut self) {
        self.placements.clear();
    }

    fn stage_or_update(&mut self, placement: PlacedCardSubmit) {
        if let Some(existing) = self
            .placements
            .iter_mut()
            .find(|existing| existing.card_id == placement.card_id)
        {
            let current_mana_spend = existing.current_mana_spend;
            let reserve_mana_spend = existing.reserve_mana_spend;
            *existing = placement;
            existing.current_mana_spend = current_mana_spend;
            existing.reserve_mana_spend = reserve_mana_spend;
            return;
        }

        self.placements.push(placement);
    }

    fn target_for(&self, card_id: CardId) -> Option<&PlayTarget> {
        self.placements
            .iter()
            .find(|placement| placement.card_id == card_id)
            .map(|placement| &placement.target)
    }

    fn remove_staged(&mut self, card_id: CardId) -> Option<PlacedCardSubmit> {
        let index = self
            .placements
            .iter()
            .position(|placement| placement.card_id == card_id)?;
        Some(self.placements.remove(index))
    }

    fn reserve_amount_for(&self, card_id: CardId) -> Option<u32> {
        self.placements
            .iter()
            .find(|placement| placement.card_id == card_id)
            .map(|placement| placement.reserve_mana_spend)
    }

    fn mana_spend_for(&self, card_id: CardId) -> Option<(u32, u32)> {
        self.placements
            .iter()
            .find(|placement| placement.card_id == card_id)
            .map(|placement| (placement.reserve_mana_spend, placement.current_mana_spend))
    }

    fn reserve_committed_by_other_cards(&self, card_id: CardId) -> u32 {
        self.placements
            .iter()
            .filter(|placement| placement.card_id != card_id)
            .map(|placement| placement.reserve_mana_spend)
            .sum()
    }

    fn increment_reserve_amount(&mut self, card_id: CardId, ceiling: u32) -> bool {
        let Some(placement) = self
            .placements
            .iter_mut()
            .find(|placement| placement.card_id == card_id)
        else {
            return false;
        };

        if placement.reserve_mana_spend >= ceiling {
            return false;
        }

        placement.reserve_mana_spend += 1;
        placement.current_mana_spend = placement.current_mana_spend.saturating_sub(1);
        true
    }

    fn decrement_reserve_amount(&mut self, card_id: CardId) -> bool {
        let Some(placement) = self
            .placements
            .iter_mut()
            .find(|placement| placement.card_id == card_id)
        else {
            return false;
        };

        if placement.reserve_mana_spend == 0 {
            return false;
        }

        placement.reserve_mana_spend -= 1;
        placement.current_mana_spend = placement.current_mana_spend.saturating_add(1);
        true
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementTimer {
    pub remaining_ms: u32,
    pub urgency_fired: bool,
    pub in_grace_window: bool,
    pub grace_remaining_ms: u32,
    pub submitted: bool,
}

impl Default for PlacementTimer {
    fn default() -> Self {
        Self {
            remaining_ms: 0,
            urgency_fired: false,
            in_grace_window: false,
            grace_remaining_ms: 0,
            submitted: false,
        }
    }
}

impl PlacementTimer {
    fn reset_for_placement(&mut self, duration_ms: u32) {
        self.remaining_ms = duration_ms;
        self.urgency_fired = false;
        self.in_grace_window = false;
        self.grace_remaining_ms = 0;
        self.submitted = false;
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementBoardView {
    pub local_player_id: PlayerId,
    pub opponent_player_id: PlayerId,
    pub spawn_edge: BoardSpawnEdge,
    pub spawn_range_cells: u8,
}

impl Default for PlacementBoardView {
    fn default() -> Self {
        Self {
            local_player_id: PlayerId(1),
            opponent_player_id: PlayerId(2),
            spawn_edge: BoardSpawnEdge::LowCells,
            spawn_range_cells: 1,
        }
    }
}

impl PlacementBoardView {
    fn is_spawn_cell(self, lane: u8, cell: u8) -> bool {
        if !(1..=BOARD_LANE_COUNT).contains(&lane) {
            return false;
        }

        let range = self.spawn_range_cells.clamp(1, BOARD_CELL_COUNT);
        match self.spawn_edge {
            BoardSpawnEdge::LowCells => (1..=range).contains(&cell),
            BoardSpawnEdge::HighCells => {
                let first_cell = BOARD_CELL_COUNT - range + 1;
                (first_cell..=BOARD_CELL_COUNT).contains(&cell)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardSpawnEdge {
    LowCells,
    HighCells,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq)]
pub struct ActivePlacementDrag {
    pub card: Option<Entity>,
    pub card_id: Option<CardId>,
    pub owner_id: Option<PlayerId>,
    pub target_kind: Option<PlacementTargetKind>,
    // Cursor coordinates carried as two coexisting frames. PROMPT 1210 split:
    // `cursor_world_position` is world-space (Y-up, origin matches
    // `BoardLayout::board_origin`) and is the only coordinate that may feed
    // `cursor_to_lane_cell` / `cursor_over_unit`. `cursor_screen_position` is
    // viewport-space pixels (Y-down, origin = window top-left) and is the only
    // coordinate that may drive UI `Node.left`/`Node.top` or rectangle-in-
    // viewport checks like `cursor_over_fan_plate`. Mixing them was the source
    // of P0 B-1203-PLA-01 — drops over the board never resolved a `BoardCell`.
    pub cursor_world_position: Option<Vec2>,
    pub cursor_screen_position: Option<Vec2>,
}

impl ActivePlacementDrag {
    fn start(
        &mut self,
        card: Entity,
        card_id: CardId,
        owner_id: PlayerId,
        target_kind: PlacementTargetKind,
    ) {
        self.card = Some(card);
        self.card_id = Some(card_id);
        self.owner_id = Some(owner_id);
        self.target_kind = Some(target_kind);
        self.cursor_world_position = None;
        self.cursor_screen_position = None;
    }

    fn clear(&mut self) {
        *self = Self::default();
    }

    fn is_active(self) -> bool {
        self.card.is_some() && self.target_kind.is_some()
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq)]
pub struct ActiveGhostUnstageDrag {
    pub card_id: Option<CardId>,
    pub cursor_screen_position: Option<Vec2>,
}

impl ActiveGhostUnstageDrag {
    fn start(&mut self, card_id: CardId) {
        self.card_id = Some(card_id);
        self.cursor_screen_position = None;
    }

    fn clear(&mut self) {
        *self = Self::default();
    }

    fn is_active(self) -> bool {
        self.card_id.is_some()
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct FanZoneBounds {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

impl Default for FanZoneBounds {
    fn default() -> Self {
        Self {
            x_min: 0.0,
            x_max: 800.0,
            y_min: 340.0,
            y_max: 600.0,
        }
    }
}

impl FanZoneBounds {
    fn contains(self, position: Vec2) -> bool {
        let x_min = self.x_min.min(self.x_max);
        let x_max = self.x_min.max(self.x_max);
        let y_min = self.y_min.min(self.y_max);
        let y_max = self.y_min.max(self.y_max);

        (x_min..=x_max).contains(&position.x) && (y_min..=y_max).contains(&position.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementTargetKind {
    Minion,
    TargetObj,
    LaneWide,
    TargetUnit,
    Instant,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandPlacementTargetKind(pub PlacementTargetKind);

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandUiPlacementDragStarted {
    pub card: Entity,
    pub owner_id: PlayerId,
}

#[derive(Message, Debug, Default, Clone, Copy, PartialEq)]
pub struct HandUiPlacementCursorMoved {
    /// World-space cursor position (Y-up, origin matches
    /// `BoardLayout::board_origin`). Required for `cursor_to_lane_cell` and
    /// `cursor_over_unit`. `None` when no active 2D camera is available to
    /// resolve the viewport → world conversion.
    pub world_position: Option<Vec2>,
    /// Viewport-space cursor position in pixels (Y-down, origin = window
    /// top-left). Required for `cursor_over_fan_plate` and the UI drag-sprite
    /// `Node.left`/`Node.top` follow.
    pub screen_position: Option<Vec2>,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandUiPlacementDragEnded;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandFanCardClicked {
    pub card: Entity,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandGridCardClicked {
    pub card: Entity,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct HandUiDraftOfferingReceived {
    pub card_ids: Vec<CardId>,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandUiCardAcquiredReceived {
    pub card_id: CardId,
}

/// PROMPT 1244 — internal hand-UI message produced by
/// [`drain_placement_rejected_receiver_system`] from each wire
/// `S2CPlacementRejected` so the rejection handler can run on a Bevy
/// `Messages` queue that tests can drive directly.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandUiPlacementRejectedReceived {
    pub reason: PlacementRejectedReason,
}

impl From<S2CPlacementRejected> for HandUiPlacementRejectedReceived {
    fn from(message: S2CPlacementRejected) -> Self {
        Self {
            reason: message.reason,
        }
    }
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct HandUiPlacementDropResolved {
    pub card: Entity,
    pub owner_id: PlayerId,
    pub target: Option<PlayTarget>,
}

#[derive(Message, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimerUrgencyAudio;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandSubmitButtonClicked {
    pub button: Entity,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct GhostPlacementChanged {
    pub target: Option<PlayTarget>,
    pub card_id: Option<CardId>,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GhostClickedEvent {
    pub card_id: CardId,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GhostDragStartEvent {
    pub card_id: CardId,
}

/// PROMPT 1149 — Bridge from resolution-event `SpawnRangeChanged` (consumed
/// by `consume_pending_resolution_script_system` in `board_rendering`) to
/// `PlacementBoardView.spawn_range_cells` (owned by hand UI). Avoids
/// cross-module resource coupling: the board renderer writes the message
/// only when the changed range belongs to the local player; the hand
/// consumer applies it to `PlacementBoardView`.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalPlayerSpawnRangeChanged {
    pub new_spawn_range_cells: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FanLayoutMetrics {
    pub fan_center_x: f32,
    pub fan_base_y: f32,
    pub fan_half_spread: f32,
    pub arc_height: f32,
    pub max_rotation_deg: f32,
}

impl HandFanLayoutConfig {
    /// Compute fan layout metrics in the `HandFanRoot` LOCAL coord space.
    ///
    /// The fan_root strip spans `left:0 right:0` (full viewport width) and is
    /// anchored to `bottom:0` with `height: HAND_FAN_STRIP_HEIGHT_PX`. Fan slot
    /// `Node.left`/`Node.top` values are interpreted relative to the strip
    /// origin (top-left of the strip), not the viewport origin:
    ///
    /// - `fan_center_x = viewport.width_px / 2.0` — the strip is full-width, so
    ///   its local X axis matches the viewport X axis with no offset.
    /// - `fan_base_y = HAND_FAN_STRIP_HEIGHT_PX - fan_base_margin_px` — offset
    ///   measured DOWN from the strip's top edge. With defaults (260 − 100)
    ///   that places the card center at local y=160, well inside the 260px
    ///   strip regardless of viewport height.
    ///
    /// Earlier revisions returned `viewport.height_px - fan_base_margin_px`
    /// (viewport-coords). Because each fan slot is `ChildOf(fan_root)`, the
    /// child was effectively positioned at `viewport.height + (viewport.height
    /// − margin − strip_height)`, i.e. off-screen at 1080p. See HU-02 Verdict A
    /// reconciliation block.
    pub fn metrics_for_viewport(&self, viewport: HandFanViewport) -> FanLayoutMetrics {
        FanLayoutMetrics {
            fan_center_x: viewport.width_px / 2.0,
            fan_base_y: HAND_FAN_STRIP_HEIGHT_PX - self.fan_base_margin_px,
            fan_half_spread: self.fan_half_spread_px,
            arc_height: self.arc_height_px,
            max_rotation_deg: self.max_rotation_deg,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FanSlotLayout {
    pub t: f32,
    pub card_x: f32,
    pub card_y: f32,
    pub card_rotation_deg: f32,
}

impl FanSlotLayout {
    pub fn bevy_rotation_radians(&self) -> f32 {
        -self.card_rotation_deg.to_radians()
    }

    pub fn bevy_rotation(&self) -> Quat {
        Quat::from_rotation_z(self.bevy_rotation_radians())
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandSubmitButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmitValidationError {
    ReserveOverdrawn,
    ManaOverdrawn,
    /// PROMPT 1244 — the server rejected the most recent
    /// `C2SSubmitPlacement` batch via `S2CPlacementRejected`. The submit
    /// affordance is re-enabled and the disclosure step surfaces the
    /// rejection reason via the existing `Correction { error }` variant.
    ServerRejected {
        reason: PlacementRejectedReason,
    },
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementDisclosureState {
    pub step: PlacementDisclosureStep,
}

impl Default for PlacementDisclosureState {
    fn default() -> Self {
        Self {
            step: PlacementDisclosureStep::Hidden,
        }
    }
}

impl PlacementDisclosureState {
    fn set_for_staged_count(&mut self, staged_count: usize) {
        self.step = if staged_count == 0 {
            PlacementDisclosureStep::CardSelection
        } else {
            PlacementDisclosureStep::StagedCard
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementDisclosureStep {
    Hidden,
    CardSelection,
    TargetSelection {
        target_kind: PlacementTargetKind,
    },
    StagedCard,
    /// Surfaces a corrective hint: client-side mana misallocation
    /// ([`SubmitValidationError::ReserveOverdrawn`] /
    /// [`SubmitValidationError::ManaOverdrawn`]) OR a server-authoritative
    /// rejection of the last submitted batch
    /// ([`SubmitValidationError::ServerRejected`] — PROMPT 1244).
    Correction {
        error: SubmitValidationError,
    },
    Submitted,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmationModal;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandFanRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandTimer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Normal,
    Urgent,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerSubmittedCheckmark;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandFullNotification;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandSubmitInteractionState {
    Active,
    Inactive,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanSlotState {
    Active,
    Ghost,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCellHighlighted;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanPlateDropZone;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanPlateHighlighted;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCellOccupied;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectiveCell {
    pub player_id: PlayerId,
    pub lane: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectiveAlive;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementTargetUnit {
    pub owner_id: PlayerId,
    pub unit_id: EntityId,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetUnitHover;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoValidTargetsOverlay;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementDisclosureGuidance;

// PROMPT 1043 — Placement Action Panel container. Wraps the disclosure
// guidance, countdown timer, placed-count readout, submit button, and
// submitted checkmark so the placement-phase action surface reads as one
// bordered panel instead of a left-column of floating text fragments.
// Visibility tracks `HandUiMode::Staging` (see
// `hand_ui_phase_transition_system`); chrome paints only while the player
// is committing placements.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementActionPanel;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementActionPanelHeader;

// PROMPT 1043 — "X / Y placed" readout next to the submit button so the
// player can see how many slots are committed against their hand size at
// a glance.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacedCountReadout;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReserveStripForFanSlot(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReserveStripAction {
    Decrement,
    Increment,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReserveStripButton {
    pub slot_index: u8,
    pub action: ReserveStripAction,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReserveStripButtonDisabled;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReserveStripValueText(pub u8);

#[derive(Resource, Debug, Clone, Copy)]
pub struct HandUiEntities {
    /// Sprint 14 story 004 (S11-TD-UI-FLEX-STRIPS): canonical
    /// `strips::HandBar` primitive — viewport-edge-anchored 180 px
    /// flex container that wraps `fan_root`. Owns the despawn root
    /// for the hand-UI entity tree on session exit.
    pub hand_bar: Entity,
    pub fan_root: Entity,
    pub fan_slots: [Entity; HAND_FAN_SLOT_COUNT],
    pub grid_slots: [Entity; DRAFT_INITIAL_GRID_SLOT_COUNT],
    pub reserve_strips: [Entity; HAND_FAN_SLOT_COUNT],
    pub drag_sprite: Entity,
    pub submit_button: Entity,
    pub timer: Entity,
    pub submitted_checkmark: Entity,
    pub hand_full_notification: Entity,
    pub no_valid_targets_overlay: Entity,
    pub placement_disclosure_guidance: Entity,
    // PROMPT 1043 — bordered container that owns the placement action UI
    // (disclosure guidance, timer, placed-count readout, submit button,
    // submitted checkmark). Visibility tracks `HandUiMode::Staging`.
    pub placement_action_panel: Entity,
    // PROMPT 1043 — "X / Y placed" readout child of the action panel.
    pub placed_count_readout: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[deprecated(
    since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001",
    note = "Universal hand marker is too coarse for QA snapshot counts (SOURCE-1077-08). \
            Use per-sub-surface root markers — HandBarRoot, HandFanRoot (existing), \
            HandDraftGridSlotRoot, PlacementActionPanelRoot — for visibility-aware counting. \
            The deprecated marker stays on existing entities for one Sprint cycle so \
            historical PROMPT 1022 / 1034 / 1036 snapshot comparisons still resolve."
)]
pub struct HandUiEntity;

/// S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 — per-sub-surface root marker for the
/// canonical `strips::HandBar` viewport-edge strip. Lives on the
/// `hand_bar` entity; the existing [`HandFanRoot`] marker tracks the fan
/// area inside this strip. Counted under a `Visibility::Visible` filter
/// in [`crate::presentation::qa_snapshot::UiCountQueries`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandBarRoot;

/// S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 — per-sub-surface root marker
/// applied to each `Hand UI Draft Grid Slot` entity. The DraftInitial /
/// DraftShop grid surfaces these 9 entities; the visible-count signal is
/// 0 outside DraftInitial / DraftShop and 9 when the grid is shown. No
/// new wrapper entity is introduced — the slots themselves carry the
/// marker so layout (`hand_draft_grid_slot_node`) is unchanged.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandDraftGridSlotRoot;

/// S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 — per-sub-surface root marker for
/// the bordered placement action panel. Lives on the same entity as
/// [`PlacementActionPanel`]; visibility tracks `HandUiMode::Staging` via
/// `hand_ui_phase_transition_system`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementActionPanelRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanSlotIndex(pub u8);

/// Sprint 18 story-022 (`S18-UI-CARD-ART-AND-LABEL-STRIP-001`) —
/// stable reference from a `FanSlotIndex` slot to its per-slot
/// [`CardSlotArtImage`] child entity.
///
/// `sync_hand_fan_card_art_system` attaches per-card art via
/// `apply_card_display_art` against the art child (not the slot
/// root) so the PROMPT 1117 chrome-preservation contract keeps the
/// slot's spawn-time `BackgroundColor` intact while the per-card
/// `ImageNode` swaps onto the dedicated child.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanSlotArt(pub Entity);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSlotIndex(pub u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandSlotCard(pub CardId);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSlotCard(pub CardId);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct GridSlotCardName(pub String);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSlotManaCost(pub u32);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridSlotState {
    Available,
    Pending,
    HandFullLocked,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingPurchaseTimer {
    pub remaining_ms: u64,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationTimer {
    pub remaining_ms: u64,
}

// ── Hand card chrome markers (PAW-002) ────────────────────────────────────────

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandCardFrame;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatBadgeAtk;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatBadgeHp;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatBadgeMp;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatBadgeAr;

// PROMPT 1029 — numeric text labels overlaid on the four stat-badge images.
// QA captured the stat badges rendering as value-less diamond icons; the
// existing chrome only painted the badge background, never a number.
// Labels are children of their badge so badge layout/despawn flows
// recursively cover them.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatBadgeAtkLabel;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatBadgeHpLabel;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatBadgeMpLabel;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatBadgeArLabel;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandRarityIcon;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandTypeIcon;

// ── PROMPT 1239 — Idle hand playable-affordance overlays ─────────────────────
//
// Sibling pathway to the Sprint 15 Story 020 drag-state overlays in
// `drag_state_visuals.rs`. While the hand is idle (no drag in flight) and the
// PLACEMENT phase / interactive HandUiMode preconditions hold, every
// populated local fan slot receives one of two mutually-exclusive
// visibility states surfaced via per-slot child overlays:
//
//   - `FanSlotPlayableAffordanceOverlay`         — `BorderColor` ACCENT;
//     paints when `current_mana + reserve_mana >= card.cost` (Minion) or
//     for non-Minion cards (mana-free).
//   - `FanSlotPlayableAffordanceUnaffordableOverlay` — `BackgroundColor`
//     at `OVERLAY_DIM_ALPHA`; paints when the affordability check fails.
//
// The overlays are CHILDREN of pre-pooled `FanSlotIndex` entities — no new
// top-level pre-pool entries (ADR-021 Impl Guideline 5 preserved). They do
// NOT carry the `drag_state_visuals::DragStateOverlay` marker, so Story 020
// AC2's `Query<&FanSlotIndex, Without<DragStateOverlay>>` semantics are
// preserved by construction (per story-023 "Story 020 AC2 Reconciliation").
//
// The sync system reads `Res<CurrentClientPhase>`, `Res<HandUiMode>`,
// `Res<ActivePlacementDrag>`, `Res<PendingPlacements>`,
// `Res<PlayerEconomyView>`, `Res<HandCardCatalog>` read-only. ADR-002 +
// ADR-012 + ADR-021 binding preserved; no Lightyear message; no
// server-authoritative state.

/// Marker on the per-slot idle playable-affordance overlay child node.
/// Visible when the slot's card is affordable AND the hand is idle in
/// `Phase::Placement` with `HandUiMode ∈ { Passive, Staging }`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanSlotPlayableAffordanceOverlay;

/// Marker on the per-slot idle unaffordable-affordance overlay child node.
/// Visible when the slot's card is NOT affordable under the same idle
/// preconditions as [`FanSlotPlayableAffordanceOverlay`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanSlotPlayableAffordanceUnaffordableOverlay;

/// Per-slot active state for the idle affordance treatment. Inserted onto
/// the `FanSlotIndex` entity when the slot carries a card and the idle
/// preconditions hold; removed otherwise. Mutually exclusive with itself
/// (never both variants on the same slot in the same frame).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanSlotPlayableAffordanceActive {
    Playable,
    Unaffordable,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandUiSystemSet {
    PhaseTransition,
    MessageDrain,
    Input,
    StateSync,
}

pub struct HandUiPlugin;

impl Plugin for HandUiPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!(target: "client::ui::hand", "HandUiPlugin loaded");
        app.init_resource::<CurrentClientPhase>()
            .init_resource::<ClientPhaseView>()
            .init_resource::<HandFanLayoutConfig>()
            .init_resource::<HandFanViewport>()
            .init_resource::<HandFanLayoutState>()
            .init_resource::<HandCardCatalog>()
            .init_resource::<HandUiTimingConfig>()
            .init_resource::<PlacementTimerConfig>()
            .init_resource::<PlayerEconomyView>()
            .init_resource::<HandContents>()
            .init_resource::<HandUiMode>()
            .init_resource::<HandUiOutboundMessages>()
            .init_resource::<PendingPlacements>()
            .init_resource::<PlacementTimer>()
            .init_resource::<PlacementBoardView>()
            // PROMPT 1086: PlacementBoardView is now driven from
            // PresentationGameSnapshotMessage + PlayerTeamMap inside
            // `apply_placement_board_view_from_snapshot_system`. The team
            // map is owned by BoardRenderingPlugin, but Bevy resource
            // init is idempotent so we mirror the registration here to
            // keep test apps built on `MinimalPlugins + HandUiPlugin`
            // self-contained (see PROMPT 735 pattern for the cross-plugin
            // message-registration analogue).
            .init_resource::<PlayerTeamMap>()
            // PROMPT 1149: same pattern — the normal-flow team-map bootstrap
            // reads `ClientSessionIdentity` (owned by `LobbyUiPlugin`) so we
            // mirror the registration so HandUiPlugin tests can drive it
            // without also loading LobbyUiPlugin. `init_resource` is
            // idempotent.
            .init_resource::<ClientSessionIdentity>()
            .init_resource::<ActivePlacementDrag>()
            .init_resource::<ActiveGhostUnstageDrag>()
            .init_resource::<PlacementDisclosureState>()
            .init_resource::<FanZoneBounds>()
            .add_message::<HandFanCardClicked>()
            .add_message::<HandGridCardClicked>()
            .add_message::<HandUiDraftOfferingReceived>()
            .add_message::<HandUiCardAcquiredReceived>()
            // PROMPT 1244 — surface S2CPlacementRejected feedback to the
            // submit affordance + disclosure step. Registered idempotently so
            // HandUiPlugin tests on `MinimalPlugins` can drive the handler
            // without a live Lightyear server.
            .add_message::<HandUiPlacementRejectedReceived>()
            .add_message::<PresentationGameSnapshotMessage>()
            // PROMPT 1149: PlayerTeamMapUpdated is registered by LobbyUiPlugin
            // and by BoardRenderingPlugin already, but `add_message` is
            // idempotent (matches PROMPT 696 / PROMPT 1086 mirror pattern)
            // and HandUiPlugin tests do not necessarily load either of the
            // other plugins. LocalPlayerSpawnRangeChanged is brand-new and
            // owned by hand-ui; board_rendering writes it from
            // `consume_pending_resolution_script_system`.
            .add_message::<PlayerTeamMapUpdated>()
            .add_message::<LocalPlayerSpawnRangeChanged>()
            .add_message::<HandUiPlacementDragStarted>()
            .add_message::<HandUiPlacementCursorMoved>()
            .add_message::<HandUiPlacementDragEnded>()
            .add_message::<HandUiPlacementDropResolved>()
            .add_message::<HandSubmitButtonClicked>()
            .add_message::<TimerUrgencyAudio>()
            .add_message::<GhostPlacementChanged>()
            .add_message::<GhostClickedEvent>()
            .add_message::<GhostDragStartEvent>()
            // PROMPT 696: bevy_picking's `DefaultPickingPlugins` already calls
            // `add_message` for these in real gameplay. We re-declare them here
            // so tests built on `MinimalPlugins + HandUiPlugin` can drive the
            // producer systems via `write_message(Pointer::<E>::new(...))`.
            // `add_message` is idempotent (see bevy_app `SubApp::add_message`).
            .add_message::<Pointer<Press>>()
            .add_message::<Pointer<Move>>()
            .add_message::<Pointer<Release>>()
            .configure_sets(
                Update,
                (
                    HandUiSystemSet::PhaseTransition,
                    HandUiSystemSet::MessageDrain,
                    HandUiSystemSet::Input,
                    HandUiSystemSet::StateSync,
                )
                    .chain()
                    .run_if(in_state(ClientState::InSession)),
            )
            .add_systems(
                OnEnter(ClientState::InSession),
                // Sprint 18 story 020 (S18-UI-PLAY-AREA-CONTAINER-001):
                // chain after `PlayAreaSpawnSet` so the `PlayAreaRoot`
                // resource (when `PlayAreaPlugin` is registered) is
                // available before `spawn_hand_ui` parents the placement
                // action panel into `PlayArea`. Harness apps without
                // `PlayAreaPlugin` fall back to the historical
                // `HandFanRoot` parent via the `unwrap_or(fan_root)`
                // branch inside `spawn_hand_ui`.
                spawn_hand_ui
                    .after(insert_placeholder_assets)
                    .after(crate::ui::PlayAreaSpawnSet),
            )
            .add_systems(OnExit(ClientState::InSession), despawn_hand_ui)
            .add_systems(
                Update,
                (
                    hand_ui_phase_transition_system.in_set(HandUiSystemSet::PhaseTransition),
                    (
                        handle_game_snapshot_system,
                        apply_placement_board_view_from_snapshot_system,
                        // PROMPT 1149 — normal-flow PlacementBoardView bootstrap
                        // (NEW-1130-01) and resolution-event spawn-range
                        // mirror (latent NEW-1130-02). Ordered after the
                        // reconnect-snapshot system so a reconnect snapshot
                        // (authoritative) wins when both fire in the same tick.
                        apply_placement_board_view_from_team_map_system,
                        apply_placement_board_view_spawn_range_system,
                        handle_draft_offering_system,
                        handle_card_acquired_system,
                        // PROMPT 1244 — drain wire S2CPlacementRejected into
                        // the internal HandUiPlacementRejectedReceived queue,
                        // then revert stale Submitted state in the same set
                        // so the affordance is actionable again on the same
                        // tick the rejection arrives.
                        drain_placement_rejected_receiver_system,
                        handle_placement_rejected_system,
                        handle_ghost_clicked_unstage_system,
                        handle_ghost_drag_started_system,
                    )
                        .chain()
                        .in_set(HandUiSystemSet::MessageDrain),
                    (
                        handle_hand_control_interactions_system,
                        // PROMPT 696: producers run BEFORE their consumers so
                        // start/move/end messages flow into the existing
                        // `handle_placement_drag_*_system` consumers the same
                        // tick the pointer event was buffered.
                        produce_fan_slot_drag_started_from_pointer_press_system,
                        produce_drag_cursor_moved_from_pointer_move_system,
                        // PROMPT 1410 — explicit window→world cursor
                        // producer that runs alongside the Pointer<Move>
                        // producer so dragging over the board (no picking
                        // backend) still feeds `cursor_world_position`.
                        produce_drag_cursor_moved_from_window_system,
                        produce_drag_ended_from_pointer_release_system,
                        handle_placement_drag_started_system,
                        handle_placement_cursor_moved_system,
                        handle_placement_drag_ended_system,
                        handle_ghost_drag_ended_system,
                        handle_grid_card_click_system,
                        handle_hand_fan_card_click_system,
                        handle_hand_fan_activate_click_system,
                        handle_placement_drop_resolved_system,
                        handle_reserve_strip_button_interactions_system,
                    )
                        .chain()
                        .in_set(HandUiSystemSet::Input),
                    (
                        sync_submit_validation_error_system,
                        handle_submit_button_click_system,
                        tick_placement_timer_system,
                        apply_placement_drag_highlights_system,
                        // PROMPT 696 / HU-DRAG-04: HandDragSprite Node trails the
                        // cursor every frame while the drag is live. Placed in
                        // StateSync so it runs after the Input set has already
                        // updated `active_drag.cursor_screen_position` from the
                        // produced `HandUiPlacementCursorMoved` messages
                        // (viewport-space; PROMPT 1210 split).
                        sync_hand_drag_sprite_position_system,
                        sync_placement_disclosure_guidance_system,
                        // PROMPT 1043 — keep the "X placed" readout in
                        // step with `PendingPlacements.staged_count()` so
                        // unstaging, restaging, and phase resets all
                        // refresh the visible budget.
                        sync_placed_count_readout_system,
                        tick_pending_purchase_timeouts_system,
                        apply_fan_layout_system,
                        sync_hand_fan_card_art_system,
                        sync_fan_slot_chrome_system,
                        // PROMPT 1029: fills numeric ATK/HP/MP/AR labels overlaid
                        // on the stat-badge images so cards in the hand no longer
                        // read as value-less diamonds.
                        sync_fan_slot_stat_labels_system,
                        apply_reserve_strip_layout_system,
                        sync_reserve_strip_state_system,
                        tick_hand_full_notification_system,
                        // Sprint 15 story 020 (S12-UX-HAND-DRAG-STATE-VISUALS-001):
                        // read-only over `ActivePlacementDrag`, `HandUiMode`,
                        // `PendingPlacements`, `PlayerEconomyView`. Patches
                        // per-slot dim / hover overlays + the fan-plate
                        // drop-target overlay. ADR-002 + ADR-012 preserved.
                        drag_state_visuals::sync_hand_drag_state_visuals_system,
                        // PROMPT 1239 (S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001):
                        // read-only over `CurrentClientPhase`, `HandUiMode`,
                        // `ActivePlacementDrag`, `PendingPlacements`,
                        // `PlayerEconomyView`, `HandCardCatalog`. Surfaces the
                        // idle Playable / Unaffordable hint per local fan slot
                        // when no drag is in flight. Distinct marker pathway
                        // from Story 020 (no `DragStateOverlay` carry).
                        sync_hand_idle_playable_affordance_system,
                    )
                        .chain()
                        .in_set(HandUiSystemSet::StateSync),
                ),
            )
            .add_systems(
                Update,
                sync_hand_fan_viewport_from_window_system
                    .before(HandUiSystemSet::StateSync)
                    .run_if(in_state(ClientState::InSession)),
            );
    }
}

pub fn compute_fan_slot_layout(
    index: usize,
    count: usize,
    metrics: FanLayoutMetrics,
) -> Option<FanSlotLayout> {
    if count == 0 || count > HAND_FAN_SLOT_COUNT || index >= count {
        return None;
    }

    let t = if count == 1 {
        0.0
    } else {
        let half_span = (count - 1) as f32 / 2.0;
        (index as f32 - half_span) / half_span
    };

    Some(FanSlotLayout {
        t,
        card_x: metrics.fan_center_x + t * metrics.fan_half_spread,
        card_y: metrics.fan_base_y - metrics.arc_height * t * t,
        card_rotation_deg: metrics.max_rotation_deg * t,
    })
}

/// Syncs `HandFanViewport` to the current `PrimaryWindow` size each frame.
///
/// Without this writer, `HandFanViewport` keeps its 800×600 default forever, so
/// `metrics_for_viewport` anchors the fan to the wrong base on every actual
/// window size. Reads every frame and `set_if_neq`s — change detection on the
/// viewport resource only fires when the window actually resizes.
pub fn sync_hand_fan_viewport_from_window_system(
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    mut viewport: ResMut<HandFanViewport>,
) {
    let Some(window) = window else {
        return;
    };
    viewport.set_if_neq(HandFanViewport {
        width_px: window.width(),
        height_px: window.height(),
    });
}

pub fn apply_fan_layout_system(
    layout_state: Res<HandFanLayoutState>,
    config: Res<HandFanLayoutConfig>,
    viewport: Res<HandFanViewport>,
    mut fan_slots: Query<
        (&FanSlotIndex, &mut Visibility, &mut Transform, &mut Node),
        Without<HandSubmitButton>,
    >,
) {
    let hand_count = layout_state.hand_count.min(HAND_FAN_SLOT_COUNT);
    let metrics = config.metrics_for_viewport(*viewport);

    for (slot_index, mut visibility, mut transform, mut node) in &mut fan_slots {
        let Some(layout) = compute_fan_slot_layout(slot_index.0 as usize, hand_count, metrics)
        else {
            *visibility = Visibility::Hidden;
            tracing::debug!(
                target: "client::ui::hand",
                slot_idx = slot_index.0,
                hand_count,
                visibility = "Hidden",
                "hand_ui_apply_fan_layout_slot",
            );
            continue;
        };

        *visibility = Visibility::Visible;
        transform.translation.x = layout.card_x;
        transform.translation.y = layout.card_y;
        transform.rotation = layout.bevy_rotation();
        node.left = Val::Px(layout.card_x);
        node.top = Val::Px(layout.card_y);
        node.width = Val::Px(HAND_CARD_DISPLAY_WIDTH_PX);
        node.height = Val::Px(HAND_CARD_DISPLAY_HEIGHT_PX);
        tracing::debug!(
            target: "client::ui::hand",
            slot_idx = slot_index.0,
            hand_count,
            card_x = layout.card_x,
            card_y = layout.card_y,
            visibility = "Visible",
            "hand_ui_apply_fan_layout_slot",
        );
    }
}

pub fn apply_reserve_strip_layout_system(
    viewport: Res<HandFanViewport>,
    fan_slots: Query<(&FanSlotIndex, &Node), With<FanSlotIndex>>,
    mut reserve_strips: Query<(&ReserveStripForFanSlot, &mut Node), Without<FanSlotIndex>>,
) {
    for (reserve_slot, mut reserve_node) in &mut reserve_strips {
        let Some((_slot_index, fan_node)) = fan_slots
            .iter()
            .find(|(slot_index, _fan_node)| slot_index.0 == reserve_slot.0)
        else {
            continue;
        };

        if let Some(left) = val_px(fan_node.left) {
            reserve_node.left = Val::Px(left);
        }

        if let Some(top) = val_px(fan_node.top) {
            reserve_node.bottom = Val::Px((viewport.height_px - top + 8.0).max(0.0));
        }
    }
}

pub fn sync_reserve_strip_state_system(
    mode: Res<HandUiMode>,
    economy: Res<PlayerEconomyView>,
    catalog: Res<HandCardCatalog>,
    pending_placements: Res<PendingPlacements>,
    mut commands: Commands,
    fan_slots: Query<(&FanSlotIndex, Option<&HandSlotCard>, Option<&FanSlotState>)>,
    mut reserve_strips: Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    buttons: Query<(
        Entity,
        &ReserveStripButton,
        Option<&ReserveStripButtonDisabled>,
    )>,
    mut value_texts: Query<(&ReserveStripValueText, &mut Text)>,
) {
    sync_reserve_strip_entities(
        *mode,
        &economy,
        &catalog,
        &pending_placements,
        &mut commands,
        &fan_slots,
        &mut reserve_strips,
        &buttons,
        &mut value_texts,
    );
}

/// Bundles the entity-modifying queries used by [`hand_ui_phase_transition_system`]
/// into a single `SystemParam` slot so the system stays under Bevy 0.18's
/// 16-param limit after the story 022 idempotency `Local` was added.
#[derive(SystemParam)]
pub struct HandUiPhaseTransitionQueries<'w, 's> {
    submit_buttons: Query<
        'w,
        's,
        (&'static mut Text, &'static mut HandSubmitInteractionState),
        With<HandSubmitButton>,
    >,
    animators: Query<'w, 's, (Entity, &'static mut TweenAnim), With<HandUiEntity>>,
    timer_states: Query<'w, 's, &'static mut TimerState, With<HandTimer>>,
}

/// PROMPT 1226 — auto-submit context for `hand_ui_phase_transition_system`.
///
/// Bundles the senders/resources required to fire one final
/// `C2SSubmitPlacement` on the Placement → Resolution transition into a single
/// `SystemParam` slot, keeping the host system at Bevy 0.18's 16-param ceiling.
#[derive(SystemParam)]
pub struct HandUiAutoSubmitParams<'w, 's> {
    submit_senders: Query<'w, 's, &'static mut MessageSender<C2SSubmitPlacement>>,
    outbound: ResMut<'w, HandUiOutboundMessages>,
    economy: Res<'w, PlayerEconomyView>,
    identity: Res<'w, ClientSessionIdentity>,
    disclosure_state: ResMut<'w, PlacementDisclosureState>,
}

/// PROMPT 1226 — reason a phase-transition auto-submit short-circuited.
///
/// Each variant maps to one of the structured tracing branches required by
/// the task. `Submitted` indicates the late `C2SSubmitPlacement` was queued
/// before the existing `pending_placements.clear()` reset path ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseTransitionAutoSubmitOutcome {
    Submitted,
    NotPlacementToResolution,
    NoLocalPlayer,
    NoPendingPlacements,
    AlreadySubmitted,
    InvalidSubmitState,
}

impl PhaseTransitionAutoSubmitOutcome {
    fn as_str(self) -> &'static str {
        match self {
            Self::Submitted => "submitted",
            Self::NotPlacementToResolution => "not_placement_to_resolution",
            Self::NoLocalPlayer => "no_local_player",
            Self::NoPendingPlacements => "no_pending_placements",
            Self::AlreadySubmitted => "already_submitted",
            Self::InvalidSubmitState => "invalid_submit_state",
        }
    }
}

pub fn hand_ui_phase_transition_system(
    current: Res<CurrentClientPhase>,
    phase_view: Res<ClientPhaseView>,
    hand_contents: Res<HandContents>,
    timer_config: Res<PlacementTimerConfig>,
    mut mode: ResMut<HandUiMode>,
    mut layout_state: ResMut<HandFanLayoutState>,
    mut pending_placements: ResMut<PendingPlacements>,
    mut placement_timer: ResMut<PlacementTimer>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    mut active_ghost_drag: ResMut<ActiveGhostUnstageDrag>,
    entities: Option<Res<HandUiEntities>>,
    mut commands: Commands,
    mut visibility_query: Query<&mut Visibility>,
    queries: HandUiPhaseTransitionQueries,
    mut auto_submit_params: HandUiAutoSubmitParams,
    mut last_observed_phase: Local<Option<RoundPhase>>,
) {
    let HandUiPhaseTransitionQueries {
        mut submit_buttons,
        mut animators,
        mut timer_states,
    } = queries;
    // Phase idempotency (story 022 / DC-5): the `phase_sink_system` upstream
    // takes `ResMut<CurrentClientPhase>` and dereferences it mutably every
    // frame (when passing it through to `apply_phase_changed_messages_with_resolution_gate`),
    // which makes `current.is_changed()` fire at 60Hz even when no real phase
    // transition occurred. Compare the just-observed phase against the
    // previous frame's observed phase instead, so `phase_changed=true` only
    // fires on actual `RoundPhase` inequality.
    let observed_phase = current.phase;
    let phase_changed = match *last_observed_phase {
        Some(prev) => prev != observed_phase,
        None => true,
    };
    if !phase_changed && !hand_contents.is_changed() {
        return;
    }

    let Some(entities) = entities else {
        return;
    };

    let prev_mode = *mode;
    let next_mode = HandUiMode::from_phase(current.phase);
    let entering_staging = phase_changed && next_mode == HandUiMode::Staging;
    tracing::info!(
        target: "client::ui::hand",
        from = ?prev_mode,
        to = ?next_mode,
        phase = ?current.phase,
        phase_changed,
        entering_staging,
        round = current.round,
        hand_len = hand_contents.cards.len(),
        "hand_ui_phase_transition",
    );
    *mode = next_mode;
    let prev_hand_count = layout_state.hand_count;
    layout_state.hand_count = if next_mode.shows_fan_slots() {
        hand_contents.cards.len().min(HAND_FAN_SLOT_COUNT)
    } else {
        0
    };
    if prev_hand_count != layout_state.hand_count {
        tracing::info!(
            target: "client::ui::hand",
            before = prev_hand_count,
            after = layout_state.hand_count,
            shows_fan_slots = next_mode.shows_fan_slots(),
            source = "phase_transition",
            "hand_ui_hand_count_set",
        );
    }

    if phase_changed {
        // PROMPT 1226 — Placement → Resolution auto-submit. Must run BEFORE
        // the `pending_placements.clear()` / timer reset below so the final
        // `C2SSubmitPlacement` carries the staged set rather than an empty
        // payload. Server already accepts late submissions inside the 250 ms
        // grace window added by PROMPT 1209 (f48583d). The helper performs
        // structured tracing for every short-circuit branch and remains
        // server-authoritative (no optimistic local apply).
        let _ = try_auto_submit_on_phase_transition(
            *last_observed_phase,
            &current,
            &auto_submit_params.identity,
            &pending_placements,
            &mut placement_timer,
            &*entities,
            &mut submit_buttons,
            &mut auto_submit_params.submit_senders,
            &mut auto_submit_params.outbound,
            &mut visibility_query,
            &auto_submit_params.economy,
            &mut commands,
            &mut auto_submit_params.disclosure_state,
        );
        let pending_before = pending_placements.staged_count();
        pending_placements.clear();
        tracing::info!(
            target: "client::ui::hand",
            before = pending_before,
            after = pending_placements.staged_count(),
            source = "phase_transition",
            "hand_ui_pending_placements_cleared",
        );
        placement_timer.in_grace_window = false;
        placement_timer.grace_remaining_ms = 0;
        placement_timer.submitted = false;
        active_drag.clear();
        active_ghost_drag.clear();
        commands.insert_resource(PlacementDisclosureState {
            step: if next_mode == HandUiMode::Staging {
                PlacementDisclosureStep::CardSelection
            } else {
                PlacementDisclosureStep::Hidden
            },
        });
        commands
            .entity(entities.fan_root)
            .remove::<FanPlateHighlighted>();

        if next_mode != HandUiMode::Staging {
            placement_timer.remaining_ms = 0;
            placement_timer.urgency_fired = false;
            commands
                .entity(entities.submit_button)
                .remove::<SubmitValidationError>();
            if let Ok(mut timer_state) = timer_states.get_mut(entities.timer) {
                *timer_state = TimerState::Normal;
            }
        }
    }

    if next_mode == HandUiMode::Hidden {
        cancel_hand_ui_tweens(&mut commands, &mut animators);
    }

    set_visibility(
        entities.fan_root,
        visibility_for(next_mode.shows_fan_root()),
        &mut visibility_query,
    );
    set_visibility(
        entities.submit_button,
        visibility_for(next_mode == HandUiMode::Staging),
        &mut visibility_query,
    );
    set_visibility(
        entities.timer,
        visibility_for(next_mode == HandUiMode::Staging),
        &mut visibility_query,
    );
    if phase_changed {
        set_visibility(
            entities.submitted_checkmark,
            Visibility::Hidden,
            &mut visibility_query,
        );
    }
    set_visibility(
        entities.drag_sprite,
        Visibility::Hidden,
        &mut visibility_query,
    );
    set_visibility(
        entities.no_valid_targets_overlay,
        Visibility::Hidden,
        &mut visibility_query,
    );
    set_visibility(
        entities.placement_disclosure_guidance,
        visibility_for(next_mode == HandUiMode::Staging),
        &mut visibility_query,
    );
    // PROMPT 1043 — the bordered action panel itself paints only while
    // staging, so the chrome does not leak into Lobby/Draft/Auction.
    set_visibility(
        entities.placement_action_panel,
        visibility_for(next_mode == HandUiMode::Staging),
        &mut visibility_query,
    );

    for entity in entities.grid_slots.iter().copied() {
        if next_mode != HandUiMode::Grid {
            set_visibility(entity, Visibility::Hidden, &mut visibility_query);
            clear_grid_slot(&mut commands, entity);
        }
    }

    if next_mode != HandUiMode::Grid {
        set_visibility(
            entities.hand_full_notification,
            Visibility::Hidden,
            &mut visibility_query,
        );
        commands
            .entity(entities.hand_full_notification)
            .remove::<NotificationTimer>();
    }

    for (index, entity) in entities.fan_slots.iter().copied().enumerate() {
        if next_mode.shows_fan_slots() {
            if let Some(card_id) = hand_contents.cards.get(index).copied() {
                commands.entity(entity).insert(HandSlotCard(card_id));
                if next_mode == HandUiMode::Staging {
                    commands.entity(entity).insert(FanSlotState::Active);
                }
            } else {
                clear_card_display_art(&mut commands, entity);
                commands
                    .entity(entity)
                    .remove::<(HandSlotCard, FanSlotState)>();
            }
        } else {
            set_visibility(entity, Visibility::Hidden, &mut visibility_query);
            clear_card_display_art(&mut commands, entity);
            commands
                .entity(entity)
                .remove::<(HandSlotCard, FanSlotState)>();
        }
    }

    if entering_staging {
        let duration_ms = server_placement_timer_duration_ms(
            &current,
            &phase_view,
            timer_config.placement_duration_ms,
        );
        placement_timer.reset_for_placement(duration_ms);
        commands
            .entity(entities.submit_button)
            .remove::<SubmitValidationError>();
        if let Ok(mut timer_state) = timer_states.get_mut(entities.timer) {
            *timer_state = TimerState::Normal;
        }

        for (mut text, mut interaction_state) in &mut submit_buttons {
            text.0.clear();
            text.0.push_str("Submit (0 cards)");
            *interaction_state = HandSubmitInteractionState::Active;
        }
    }

    *last_observed_phase = Some(observed_phase);
}

pub fn sync_hand_fan_card_art_system(
    catalog: Res<HandCardCatalog>,
    asset_server: Option<Res<AssetServer>>,
    mut commands: Commands,
    fan_slots: Query<(Option<&HandSlotCard>, &FanSlotArt), With<FanSlotIndex>>,
) {
    // Sprint 18 story-022 (AC4) — per-card art binds onto the
    // `CardSlotArtImage` child entity (sized to the
    // `CardSlotKind::HandFan` image-inset rectangle) rather than the
    // slot root. The slot root keeps its spawn-time
    // `HAND_CARD_SLOT_BACKGROUND` floor untouched; the art ImageNode
    // attaches to the child via `apply_card_display_art`, which
    // already produces `ImageNode::new(handle)` with
    // `NodeImageMode::Auto` (UI-1129-05 banner-stretch resolved
    // structurally).
    for (card, art) in &fan_slots {
        let art_entity = art.0;
        let Some(card) = card else {
            clear_card_display_art(&mut commands, art_entity);
            continue;
        };

        apply_card_display_art(
            &mut commands,
            art_entity,
            catalog.cards.get(&card.0),
            asset_server.as_deref(),
        );
    }
}

/// PROMPT 1029 — populates the numeric stat labels overlaid on each hand-card
/// stat badge. Each label is a grandchild of the fan slot (slot → badge → label),
/// so we resolve the slot via two `ChildOf` hops. When the slot has no card or
/// the catalog lookup fails, the label is left empty (no diamond-without-number
/// glyph rendered). Runs in [`HandUiSystemSet::StateSync`] alongside the chrome
/// image sync so atk/hp/mp/ar text stays consistent with the rendered badges.
#[allow(clippy::type_complexity)]
pub fn sync_fan_slot_stat_labels_system(
    catalog: Res<HandCardCatalog>,
    slots: Query<&HandSlotCard, With<FanSlotIndex>>,
    badge_parents: Query<
        &ChildOf,
        Or<(
            With<StatBadgeAtk>,
            With<StatBadgeHp>,
            With<StatBadgeMp>,
            With<StatBadgeAr>,
        )>,
    >,
    mut atk_labels: Query<(&mut Text, &ChildOf), With<StatBadgeAtkLabel>>,
    mut hp_labels: Query<
        (&mut Text, &ChildOf),
        (With<StatBadgeHpLabel>, Without<StatBadgeAtkLabel>),
    >,
    mut mp_labels: Query<
        (&mut Text, &ChildOf),
        (
            With<StatBadgeMpLabel>,
            Without<StatBadgeAtkLabel>,
            Without<StatBadgeHpLabel>,
        ),
    >,
    mut ar_labels: Query<
        (&mut Text, &ChildOf),
        (
            With<StatBadgeArLabel>,
            Without<StatBadgeAtkLabel>,
            Without<StatBadgeHpLabel>,
            Without<StatBadgeMpLabel>,
        ),
    >,
) {
    let resolve = |child_of: &ChildOf| -> Option<&shared::card::CardData> {
        let badge = child_of.parent();
        let slot = badge_parents.get(badge).ok()?.parent();
        let slot_card = slots.get(slot).ok()?;
        catalog.cards.get(&slot_card.0)
    };

    for (mut text, child_of) in &mut atk_labels {
        write_stat_label_text(&mut text, resolve(child_of).map(|c| c.atk));
    }
    for (mut text, child_of) in &mut hp_labels {
        write_stat_label_text(&mut text, resolve(child_of).map(|c| c.hp));
    }
    for (mut text, child_of) in &mut mp_labels {
        write_stat_label_text(&mut text, resolve(child_of).map(|c| c.mp));
    }
    for (mut text, child_of) in &mut ar_labels {
        write_stat_label_text(&mut text, resolve(child_of).map(|c| c.ar));
    }
}

fn write_stat_label_text(text: &mut Text, value: Option<u8>) {
    text.0.clear();
    if let Some(value) = value {
        text.0.push_str(&value.to_string());
    }
}

/// Syncs ImageNode handles on the card chrome child entities (frame, badges, icons)
/// for each fan slot based on current card data and fallback state. Runs after
/// `sync_hand_fan_card_art_system` so `CardDisplayArtFallback` is already set.
pub fn sync_fan_slot_chrome_system(
    slots: Query<
        (
            Entity,
            Option<&HandSlotCard>,
            Option<&CardDisplayArtFallback>,
        ),
        With<FanSlotIndex>,
    >,
    catalog: Res<HandCardCatalog>,
    placeholder: Option<Res<PlaceholderAssets>>,
    mut frames: Query<(&mut ImageNode, &ChildOf), With<HandCardFrame>>,
    mut rarity_icons: Query<
        (&mut ImageNode, &ChildOf),
        (With<HandRarityIcon>, Without<HandCardFrame>),
    >,
    mut type_icons: Query<
        (&mut ImageNode, &ChildOf),
        (
            With<HandTypeIcon>,
            Without<HandCardFrame>,
            Without<HandRarityIcon>,
        ),
    >,
) {
    let Some(placeholder) = placeholder.as_ref() else {
        return;
    };
    for (mut img, child_of) in &mut frames {
        let Ok((_, slot_card, fallback)) = slots.get(child_of.parent()) else {
            continue;
        };
        img.image = chrome_frame_handle(slot_card, fallback, &catalog, placeholder);
    }
    for (mut img, child_of) in &mut rarity_icons {
        let Ok((_, slot_card, _)) = slots.get(child_of.parent()) else {
            continue;
        };
        img.image = chrome_rarity_icon_handle(slot_card, &catalog, &placeholder);
    }
    for (mut img, child_of) in &mut type_icons {
        let Ok((_, slot_card, _)) = slots.get(child_of.parent()) else {
            continue;
        };
        img.image = chrome_type_icon_handle(slot_card, &catalog, &placeholder);
    }
}

fn chrome_frame_handle(
    slot_card: Option<&HandSlotCard>,
    fallback: Option<&CardDisplayArtFallback>,
    catalog: &HandCardCatalog,
    placeholder: &PlaceholderAssets,
) -> Handle<Image> {
    if fallback.is_some() {
        return placeholder.fallback.clone();
    }
    let Some(card) = slot_card.and_then(|sc| catalog.cards.get(&sc.0)) else {
        return placeholder.card_frame_common.clone();
    };
    match card.rarity {
        Rarity::Common | Rarity::Uncommon => placeholder.card_frame_common.clone(),
        Rarity::Rare => placeholder.card_frame_rare.clone(),
        Rarity::Epic => placeholder.card_frame_epic.clone(),
        Rarity::Legendary => placeholder.card_frame_legendary.clone(),
    }
}

fn chrome_rarity_icon_handle(
    slot_card: Option<&HandSlotCard>,
    catalog: &HandCardCatalog,
    placeholder: &PlaceholderAssets,
) -> Handle<Image> {
    let Some(card) = slot_card.and_then(|sc| catalog.cards.get(&sc.0)) else {
        return placeholder.rarity_icon_common.clone();
    };
    match card.rarity {
        Rarity::Common | Rarity::Uncommon => placeholder.rarity_icon_common.clone(),
        Rarity::Rare => placeholder.rarity_icon_rare.clone(),
        Rarity::Epic => placeholder.rarity_icon_epic.clone(),
        Rarity::Legendary => placeholder.rarity_icon_legendary.clone(),
    }
}

fn chrome_type_icon_handle(
    slot_card: Option<&HandSlotCard>,
    catalog: &HandCardCatalog,
    placeholder: &PlaceholderAssets,
) -> Handle<Image> {
    let Some(card) = slot_card.and_then(|sc| catalog.cards.get(&sc.0)) else {
        return placeholder.class_type_icon_neutral.clone();
    };
    match card.class {
        ClassId::Iop => placeholder.class_type_icon_iop.clone(),
        ClassId::Cra => placeholder.class_type_icon_cra.clone(),
        ClassId::Sacrier => placeholder.class_type_icon_sacrier.clone(),
        ClassId::Xelor => placeholder.class_type_icon_xelor.clone(),
        ClassId::Ecaflip => placeholder.class_type_icon_ecaflip.clone(),
        ClassId::Sadida => placeholder.class_type_icon_sadida.clone(),
        ClassId::Neutral => placeholder.class_type_icon_neutral.clone(),
    }
}

fn server_placement_timer_duration_ms(
    current: &CurrentClientPhase,
    phase_view: &ClientPhaseView,
    fallback_ms: u32,
) -> u32 {
    if current.phase == RoundPhase::Placement
        && phase_view.phase == RoundPhase::Placement
        && phase_view.round_number == current.round
        && phase_view.timer_duration_ms > 0
    {
        phase_view.timer_duration_ms
    } else {
        fallback_ms
    }
}

pub fn tick_placement_timer_system(
    mode: Res<HandUiMode>,
    time: Res<Time<Virtual>>,
    timer_config: Res<PlacementTimerConfig>,
    entities: Option<Res<HandUiEntities>>,
    mut placement_timer: ResMut<PlacementTimer>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    mut disclosure_state: ResMut<PlacementDisclosureState>,
    pending_placements: Res<PendingPlacements>,
    economy: Res<PlayerEconomyView>,
    mut urgency_writer: MessageWriter<TimerUrgencyAudio>,
    mut commands: Commands,
    mut visibility_query: Query<&mut Visibility>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<HandTimer>>,
        Query<(&mut Text, &mut HandSubmitInteractionState), With<HandSubmitButton>>,
    )>,
    mut timer_states: Query<&mut TimerState, With<HandTimer>>,
    mut submit_senders: Query<&mut MessageSender<C2SSubmitPlacement>>,
    mut outbound: ResMut<HandUiOutboundMessages>,
) {
    let Some(entities) = entities else {
        return;
    };

    if *mode != HandUiMode::Staging {
        return;
    }

    let mut elapsed_ms = u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX);
    let previous_remaining_ms = placement_timer.remaining_ms;

    if placement_timer.remaining_ms > 0 {
        let consumed_ms = placement_timer.remaining_ms.min(elapsed_ms);
        placement_timer.remaining_ms -= consumed_ms;
        elapsed_ms -= consumed_ms;
    }

    if previous_remaining_ms > timer_config.urgency_threshold_ms
        && placement_timer.remaining_ms <= timer_config.urgency_threshold_ms
        && !placement_timer.urgency_fired
    {
        placement_timer.urgency_fired = true;
        if let Ok(mut timer_state) = timer_states.get_mut(entities.timer) {
            *timer_state = TimerState::Urgent;
        }
        urgency_writer.write(TimerUrgencyAudio);
    }

    if placement_timer.remaining_ms == 0
        && !placement_timer.submitted
        && !placement_timer.in_grace_window
    {
        if active_drag.is_active() {
            placement_timer.in_grace_window = true;
            placement_timer.grace_remaining_ms = timer_config.grace_window_ms;
        } else {
            let mut submit_buttons = text_sets.p1();
            submit_pending_placements(
                &pending_placements,
                entities.submit_button,
                entities.submitted_checkmark,
                &mut submit_buttons,
                &mut submit_senders,
                &mut outbound,
                &mut placement_timer,
                &mut visibility_query,
                &economy,
                &mut commands,
                &mut disclosure_state,
            );
        }
    }

    if placement_timer.in_grace_window && !placement_timer.submitted {
        placement_timer.grace_remaining_ms = placement_timer
            .grace_remaining_ms
            .saturating_sub(elapsed_ms);

        if placement_timer.grace_remaining_ms == 0 {
            cancel_active_placement_drag(
                &mut active_drag,
                &mut commands,
                entities.drag_sprite,
                entities.fan_root,
                &mut visibility_query,
                &mut disclosure_state,
            );

            let mut submit_buttons = text_sets.p1();
            submit_pending_placements(
                &pending_placements,
                entities.submit_button,
                entities.submitted_checkmark,
                &mut submit_buttons,
                &mut submit_senders,
                &mut outbound,
                &mut placement_timer,
                &mut visibility_query,
                &economy,
                &mut commands,
                &mut disclosure_state,
            );
        }
    }

    {
        let mut timer_texts = text_sets.p0();
        set_timer_text(&mut timer_texts, placement_timer.remaining_ms);
    }
}

pub fn handle_game_snapshot_system(
    mut snapshots: MessageReader<PresentationGameSnapshotMessage>,
    mut hand_contents: ResMut<HandContents>,
) {
    for snapshot in snapshots.read().map(|message| &message.0) {
        let Some(local_player) = snapshot
            .players
            .iter()
            .find(|player| player.player_id == snapshot.recipient_player_id)
        else {
            warn!(
                "Hand UI: snapshot for {:?} does not contain local player",
                snapshot.recipient_player_id
            );
            continue;
        };

        let before_len = hand_contents.cards.len();
        hand_contents.cards = local_player.hand.clone();
        tracing::info!(
            target: "client::ui::hand",
            player_id = ?snapshot.recipient_player_id,
            before_len,
            after_len = hand_contents.cards.len(),
            source = "game_snapshot",
            "hand_ui_hand_contents_set",
        );
    }
}

// PROMPT 1086 — Update PlacementBoardView from each snapshot so the
// click-to-stage default target (and any other consumer of
// PlacementBoardView) reflects the local player's actual perspective.
//
// Before this system existed, PlacementBoardView stayed pinned to its
// Default value (local_player_id=1, spawn_edge=LowCells, range=1), which
// caused `default_click_stage_target` to always emit
// `BoardCell { lane: 1, cell: 1 }` — even for Player B, whose server-side
// spawn cell is 8. The server correctly rejects those placements
// (PROMPT 1079 / 1084), but the user sees a card stage at the wrong
// cell and no actionable feedback. See AUDIT-1076-09 and PROMPT 1079
// residual risk #2.
pub fn apply_placement_board_view_from_snapshot_system(
    mut snapshots: MessageReader<PresentationGameSnapshotMessage>,
    player_team_map: Res<PlayerTeamMap>,
    mut board_view: ResMut<PlacementBoardView>,
) {
    for snapshot in snapshots.read().map(|message| &message.0) {
        let local_player_id = snapshot.recipient_player_id;
        let Some(local_snapshot) = snapshot
            .players
            .iter()
            .find(|player| player.player_id == local_player_id)
        else {
            continue;
        };

        let opponent_player_id = snapshot
            .players
            .iter()
            .map(|player| player.player_id)
            .find(|player_id| *player_id != local_player_id)
            .unwrap_or(board_view.opponent_player_id);

        let spawn_edge = spawn_edge_for_local_player(local_player_id, &player_team_map);
        let spawn_range_cells = local_snapshot.spawn_range_cells;

        let next = PlacementBoardView {
            local_player_id,
            opponent_player_id,
            spawn_edge,
            spawn_range_cells,
        };

        if *board_view != next {
            tracing::info!(
                target: "client::ui::hand::placement_board_view",
                local_player_id = ?next.local_player_id,
                opponent_player_id = ?next.opponent_player_id,
                spawn_edge = ?next.spawn_edge,
                spawn_range_cells = next.spawn_range_cells,
                source = "game_snapshot",
                "placement_board_view_updated"
            );
            *board_view = next;
        }
    }
}

fn spawn_edge_for_local_player(
    local_player_id: PlayerId,
    player_team_map: &PlayerTeamMap,
) -> BoardSpawnEdge {
    // Mirrors `presentation::board_rendering::spawn_range_edge_for_player`
    // for the local player: team 0 spawns from the low-cell edge, every
    // other team spawns from the high-cell edge. When the team map is not
    // yet populated, fall back to the historical Player A convention
    // (PlayerId(1) == LowCells) so behaviour matches the existing
    // assumption rather than silently flipping.
    if let Some(team) = player_team_map.team_for(local_player_id) {
        return if team == 0 {
            BoardSpawnEdge::LowCells
        } else {
            BoardSpawnEdge::HighCells
        };
    }

    if local_player_id == PlayerId(1) {
        BoardSpawnEdge::LowCells
    } else {
        BoardSpawnEdge::HighCells
    }
}

// PROMPT 1149 — Drive `PlacementBoardView` perspective fields from
// `PlayerTeamMapUpdated` + `ClientSessionIdentity` during normal play.
//
// Background (NEW-1130-01, AUDIT-1126-01): `S2CGameSnapshot` (the only
// trigger for `apply_placement_board_view_from_snapshot_system`) is sent
// by the server **only** on reconnect or explicit `C2SRequestSnapshot`.
// In a clean session start no snapshot ever fires, so `PlacementBoardView`
// stays pinned to its `Default`: `(PlayerId(1), LowCells, range=1)`.
// Player A coincidentally matches that default; Player B does not, so
// the click-to-stage default at `default_click_stage_target` routes every
// drop to `BoardCell { lane: 1, cell: 1 }` and the server correctly
// rejects every Player B submission with `SpawnRangeRejected` (server log
// line `WARN handle_placement_submission: submission rejected
// reason=SpawnRangeRejected` — PROMPT 1079 audit line).
//
// This system listens for `PlayerTeamMapUpdated`, which is broadcast by
// `drain_lobby_s2c_system` on every `S2CRoomCreated` / `S2CJoinAck` /
// `S2CSlotUpdated` AND re-broadcast by
// `broadcast_player_team_map_on_session_enter_system` on
// `OnEnter(ClientState::InSession)`. It reads the local player from
// `ClientSessionIdentity` (populated by `apply_handshake_message` on
// `S2CHandshake`) and resolves the spawn edge from the message's slots —
// self-contained, no dependency on the `PlayerTeamMap` resource update
// order. `spawn_range_cells` is preserved so the reconnect snapshot
// system and the resolution-event consumer remain authoritative for that
// field.
pub fn apply_placement_board_view_from_team_map_system(
    mut updates: MessageReader<PlayerTeamMapUpdated>,
    identity: Res<ClientSessionIdentity>,
    mut board_view: ResMut<PlacementBoardView>,
) {
    let Some(latest_slots) = updates.read().last().map(|update| update.slots.clone()) else {
        return;
    };

    let Some(local_player_id) = identity.player_id else {
        return;
    };

    let local_team = latest_slots
        .iter()
        .find(|slot| slot.player_id == Some(local_player_id))
        .map(|slot| slot.team);

    let spawn_edge = match local_team {
        Some(0) => BoardSpawnEdge::LowCells,
        Some(_) => BoardSpawnEdge::HighCells,
        None => {
            if local_player_id == PlayerId(1) {
                BoardSpawnEdge::LowCells
            } else {
                BoardSpawnEdge::HighCells
            }
        }
    };

    let opponent_player_id = latest_slots
        .iter()
        .filter_map(|slot| slot.player_id)
        .find(|player_id| *player_id != local_player_id)
        .unwrap_or(board_view.opponent_player_id);

    let next = PlacementBoardView {
        local_player_id,
        opponent_player_id,
        spawn_edge,
        spawn_range_cells: board_view.spawn_range_cells,
    };

    if *board_view != next {
        tracing::info!(
            target: "client::ui::hand::placement_board_view",
            local_player_id = ?next.local_player_id,
            opponent_player_id = ?next.opponent_player_id,
            spawn_edge = ?next.spawn_edge,
            spawn_range_cells = next.spawn_range_cells,
            source = "team_map",
            "placement_board_view_updated"
        );
        *board_view = next;
    }
}

// PROMPT 1149 — Apply local-player spawn-range expansion to
// `PlacementBoardView.spawn_range_cells` (latent NEW-1130-02).
//
// Background: `apply_resolution_spawn_range_changes` in
// `presentation::board_rendering` updates the visual highlight set when
// a `ResolutionEvent::SpawnRangeChanged` fires (after a fake objective
// is destroyed), but the hand UI's `PlacementBoardView` was never told.
// `default_click_stage_target` reads `PlacementBoardView.spawn_range_cells`
// to decide which cells are legal staging targets — so after a fake
// destruction the click-to-stage default would still target the
// pre-expansion cells, and any expansion past the first row would be
// invisible to the staging path.
//
// `consume_pending_resolution_script_system` in `board_rendering` writes
// a `LocalPlayerSpawnRangeChanged` for every `SpawnRangeChanged` event
// whose `player_id` matches the local player. This consumer mirrors the
// new value into `PlacementBoardView.spawn_range_cells`.
pub fn apply_placement_board_view_spawn_range_system(
    mut updates: MessageReader<LocalPlayerSpawnRangeChanged>,
    mut board_view: ResMut<PlacementBoardView>,
) {
    let mut latest = None;
    for update in updates.read() {
        latest = Some(update.new_spawn_range_cells);
    }
    let Some(new_spawn_range_cells) = latest else {
        return;
    };

    if board_view.spawn_range_cells == new_spawn_range_cells {
        return;
    }

    tracing::info!(
        target: "client::ui::hand::placement_board_view",
        local_player_id = ?board_view.local_player_id,
        opponent_player_id = ?board_view.opponent_player_id,
        spawn_edge = ?board_view.spawn_edge,
        spawn_range_cells = new_spawn_range_cells,
        source = "resolution_spawn_range_changed",
        "placement_board_view_updated"
    );
    board_view.spawn_range_cells = new_spawn_range_cells;
}

pub fn handle_draft_offering_system(
    mode: Res<HandUiMode>,
    catalog: Res<HandCardCatalog>,
    asset_server: Option<Res<AssetServer>>,
    entities: Option<Res<HandUiEntities>>,
    mut offerings: MessageReader<HandUiDraftOfferingReceived>,
    mut commands: Commands,
    mut visibility_query: Query<&mut Visibility>,
) {
    let Some(entities) = entities else {
        for _offering in offerings.read() {}
        return;
    };

    for offering in offerings.read() {
        for (index, entity) in entities.grid_slots.iter().copied().enumerate() {
            let Some(card_id) = offering.card_ids.get(index).copied() else {
                set_visibility(entity, Visibility::Hidden, &mut visibility_query);
                clear_grid_slot(&mut commands, entity);
                continue;
            };

            let Some(card) = catalog.cards.get(&card_id) else {
                warn!("Draft offering referenced unknown card id {card_id:?}");
                set_visibility(entity, Visibility::Hidden, &mut visibility_query);
                clear_grid_slot(&mut commands, entity);
                continue;
            };

            commands.entity(entity).insert((
                GridSlotCard(card_id),
                GridSlotCardName(card.name_en.clone()),
                GridSlotManaCost(card.cost),
                GridSlotState::Available,
            ));
            apply_card_display_art(&mut commands, entity, Some(card), asset_server.as_deref());
            commands.entity(entity).remove::<PendingPurchaseTimer>();
            set_visibility(
                entity,
                visibility_for(*mode == HandUiMode::Grid),
                &mut visibility_query,
            );
        }
    }
}

pub fn handle_card_acquired_system(
    mode: Res<HandUiMode>,
    catalog: Res<HandCardCatalog>,
    config: Res<HandFanLayoutConfig>,
    viewport: Res<HandFanViewport>,
    timing: Res<HandUiTimingConfig>,
    asset_server: Option<Res<AssetServer>>,
    entities: Option<Res<HandUiEntities>>,
    mut acquisitions: MessageReader<HandUiCardAcquiredReceived>,
    mut hand_contents: ResMut<HandContents>,
    mut layout_state: ResMut<HandFanLayoutState>,
    mut commands: Commands,
    mut grid_slots: ParamSet<(
        Query<(Entity, &GridSlotCard, &mut Visibility), With<GridSlotIndex>>,
        Query<(Entity, &Visibility, Option<&GridSlotCard>), With<GridSlotIndex>>,
    )>,
    mut fan_slots: Query<
        (
            &mut Visibility,
            &mut Transform,
            &mut Node,
            Option<&mut TweenAnim>,
            &FanSlotArt,
        ),
        (With<FanSlotIndex>, Without<GridSlotIndex>),
    >,
    mut notification: Query<
        &mut Visibility,
        (
            With<HandFullNotification>,
            Without<GridSlotIndex>,
            Without<FanSlotIndex>,
        ),
    >,
) {
    let Some(entities) = entities else {
        for _acquisition in acquisitions.read() {}
        return;
    };

    for acquisition in acquisitions.read() {
        hide_acquired_grid_slot(&mut commands, &mut grid_slots.p0(), acquisition.card_id);

        let before_len = hand_contents.cards.len();
        if hand_contents.cards.len() < HAND_FAN_SLOT_COUNT {
            hand_contents.cards.push(acquisition.card_id);
        }
        tracing::info!(
            target: "client::ui::hand",
            card_id = ?acquisition.card_id,
            before_len,
            after_len = hand_contents.cards.len(),
            source = "card_acquired",
            "hand_ui_hand_contents_set",
        );

        let hand_count = hand_contents.cards.len().min(HAND_FAN_SLOT_COUNT);
        let prev_hand_count = layout_state.hand_count;
        layout_state.hand_count = if mode.shows_fan_slots() {
            hand_count
        } else {
            0
        };
        if prev_hand_count != layout_state.hand_count {
            tracing::info!(
                target: "client::ui::hand",
                before = prev_hand_count,
                after = layout_state.hand_count,
                shows_fan_slots = mode.shows_fan_slots(),
                source = "card_acquired",
                "hand_ui_hand_count_set",
            );
        }

        if hand_count > 0 {
            let fan_index = hand_count - 1;
            let fan_entity = entities.fan_slots[fan_index];
            if let Ok((mut visibility, mut transform, mut node, animator, fan_art)) =
                fan_slots.get_mut(fan_entity)
            {
                let metrics = config.metrics_for_viewport(*viewport);
                if let Some(layout) = compute_fan_slot_layout(fan_index, hand_count, metrics) {
                    *visibility = Visibility::Visible;
                    transform.rotation = layout.bevy_rotation();
                    node.left = Val::Px(layout.card_x);
                    node.top = Val::Px(layout.card_y);
                    node.width = Val::Px(HAND_CARD_DISPLAY_WIDTH_PX);
                    node.height = Val::Px(HAND_CARD_DISPLAY_HEIGHT_PX);
                    commands
                        .entity(fan_entity)
                        .insert(HandSlotCard(acquisition.card_id));
                    // Sprint 18 story-022 (AC4): per-card art binds onto
                    // the `CardSlotArtImage` child, not the slot root.
                    apply_card_display_art(
                        &mut commands,
                        fan_art.0,
                        catalog.cards.get(&acquisition.card_id),
                        asset_server.as_deref(),
                    );
                    tracing::info!(
                        target: "client::ui::hand",
                        slot_idx = fan_index,
                        card_id = ?acquisition.card_id,
                        duration_ms = timing.card_draw_animation_ms,
                        "hand_ui_install_card_draw_animation",
                    );
                    install_card_draw_animation(
                        &mut commands,
                        fan_entity,
                        animator,
                        transform.translation,
                        Vec3::new(layout.card_x, layout.card_y, transform.translation.z),
                        timing.card_draw_animation_ms,
                    );
                }
            }
        }

        if hand_contents.cards.len() >= HAND_FAN_SLOT_COUNT {
            lock_visible_grid_slots(&mut commands, &mut grid_slots.p1());
            activate_hand_full_notification(
                &mut commands,
                entities.hand_full_notification,
                &mut notification,
                timing.hand_full_notification_duration_ms,
            );
        }
    }
}

/// PROMPT 1244 — drains wire `S2CPlacementRejected` messages into the
/// internal [`HandUiPlacementRejectedReceived`] queue so the handler system
/// can run on a Bevy `Messages` resource that tests on `MinimalPlugins +
/// HandUiPlugin` can drive directly without a live Lightyear server.
///
/// Mirrors the auction pattern in
/// `client::ui::shop_auction::drain_auction_bid_rejected_receiver_system`.
pub fn drain_placement_rejected_receiver_system(
    mut receivers: Query<&mut MessageReceiver<S2CPlacementRejected>>,
    mut writer: MessageWriter<HandUiPlacementRejectedReceived>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::info!(
                target: "client::ui::hand",
                reason = ?message.reason,
                msg_type = "S2CPlacementRejected",
                "drain_placement_rejected: recv"
            );
            writer.write(message.into());
        }
    }
}

/// PROMPT 1244 — turns each [`HandUiPlacementRejectedReceived`] into a
/// visible correction state.
///
/// Reverts the optimistic Submitted view the click handler installed (per
/// `submit_pending_placements`):
/// - `placement_timer.submitted` -> `false`
/// - submit button text -> `Submit (N cards)` reflecting `staged_count()`
/// - submit button interaction -> `Active`
/// - submitted checkmark -> `Hidden`
/// - disclosure step -> [`PlacementDisclosureStep::RejectedByServer`]
///
/// Authority remains server-side: this system never accepts placements or
/// mutates `PendingPlacements` — the rejection only re-enables the local
/// submit affordance so the player can adjust their batch and retry.
#[allow(clippy::too_many_arguments)]
pub fn handle_placement_rejected_system(
    mut rejections: MessageReader<HandUiPlacementRejectedReceived>,
    entities: Option<Res<HandUiEntities>>,
    pending_placements: Res<PendingPlacements>,
    mut placement_timer: ResMut<PlacementTimer>,
    mut disclosure_state: ResMut<PlacementDisclosureState>,
    mut commands: Commands,
    mut submit_buttons: Query<(&mut Text, &mut HandSubmitInteractionState), With<HandSubmitButton>>,
    mut visibility_query: Query<&mut Visibility>,
) {
    let Some(entities) = entities else {
        for _ in rejections.read() {}
        return;
    };

    for rejection in rejections.read() {
        let staged_count = pending_placements.staged_count();
        tracing::warn!(
            target: "client::ui::hand",
            reason = ?rejection.reason,
            staged_count,
            placement_timer_submitted_before = placement_timer.submitted,
            "hand_ui_placement_rejection_received"
        );

        placement_timer.submitted = false;

        if let Ok((mut text, mut interaction_state)) =
            submit_buttons.get_mut(entities.submit_button)
        {
            text.0.clear();
            text.0.push_str(&format!("Submit ({staged_count} cards)"));
            *interaction_state = HandSubmitInteractionState::Active;
        }

        commands
            .entity(entities.submit_button)
            .remove::<SubmitValidationError>();

        set_visibility(
            entities.submitted_checkmark,
            Visibility::Hidden,
            &mut visibility_query,
        );

        disclosure_state.step = PlacementDisclosureStep::Correction {
            error: SubmitValidationError::ServerRejected {
                reason: rejection.reason,
            },
        };
    }
}

pub fn handle_ghost_clicked_unstage_system(
    mode: Res<HandUiMode>,
    mut clicks: MessageReader<GhostClickedEvent>,
    mut pending_placements: ResMut<PendingPlacements>,
    mut disclosure_state: ResMut<PlacementDisclosureState>,
    mut ghost_writer: MessageWriter<GhostPlacementChanged>,
    mut commands: Commands,
    fan_slots: Query<(Entity, &FanSlotIndex, &HandSlotCard), With<FanSlotIndex>>,
    mut reserve_strips: Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    mut submit_buttons: Query<&mut Text, With<HandSubmitButton>>,
) {
    for click in clicks.read() {
        if *mode != HandUiMode::Staging {
            continue;
        }

        let should_unstage = pending_placements
            .target_for(click.card_id)
            .map(is_board_ghost_target)
            .unwrap_or(false);

        if should_unstage {
            unstage_card(
                click.card_id,
                &mut pending_placements,
                &mut ghost_writer,
                &mut commands,
                &fan_slots,
                &mut reserve_strips,
                &mut submit_buttons,
                &mut disclosure_state,
            );
        }
    }
}

pub fn handle_ghost_drag_started_system(
    mode: Res<HandUiMode>,
    mut starts: MessageReader<GhostDragStartEvent>,
    pending_placements: Res<PendingPlacements>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    mut active_ghost_drag: ResMut<ActiveGhostUnstageDrag>,
) {
    for start in starts.read() {
        let should_track = *mode == HandUiMode::Staging
            && pending_placements
                .target_for(start.card_id)
                .map(is_board_ghost_target)
                .unwrap_or(false);

        if should_track {
            active_drag.clear();
            active_ghost_drag.start(start.card_id);
        } else {
            active_ghost_drag.clear();
        }
    }
}

pub fn handle_hand_control_interactions_system(
    mut interactions: Query<
        (
            Entity,
            &Interaction,
            Option<&GridSlotIndex>,
            Option<&FanSlotIndex>,
            Option<&HandSubmitButton>,
        ),
        (
            Changed<Interaction>,
            Or<(
                With<GridSlotIndex>,
                With<FanSlotIndex>,
                With<HandSubmitButton>,
            )>,
        ),
    >,
    mut grid_clicks: MessageWriter<HandGridCardClicked>,
    mut fan_clicks: MessageWriter<HandFanCardClicked>,
    mut submit_clicks: MessageWriter<HandSubmitButtonClicked>,
) {
    for (entity, interaction, grid_slot, fan_slot, submit) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if grid_slot.is_some() {
            grid_clicks.write(HandGridCardClicked { card: entity });
        } else if fan_slot.is_some() {
            fan_clicks.write(HandFanCardClicked { card: entity });
        } else if submit.is_some() {
            submit_clicks.write(HandSubmitButtonClicked { button: entity });
        }
    }
}

pub fn handle_grid_card_click_system(
    mode: Res<HandUiMode>,
    timing: Res<HandUiTimingConfig>,
    mut clicks: MessageReader<HandGridCardClicked>,
    mut grid_cards: Query<(&GridSlotCard, Option<&GridSlotState>), With<GridSlotIndex>>,
    mut commands: Commands,
    mut outbound: ResMut<HandUiOutboundMessages>,
    mut senders: Query<&mut MessageSender<C2SPurchaseCard>>,
) {
    for click in clicks.read() {
        info!(
            "DRAFT_INITIAL click received — handler=handle_grid_card_click_system, click_entity={:?}",
            click.card
        );
        if *mode != HandUiMode::Grid {
            continue;
        }

        let Ok((card, state)) = grid_cards.get_mut(click.card) else {
            continue;
        };

        if state != Some(&GridSlotState::Available) {
            continue;
        }

        let message = C2SPurchaseCard { card_id: card.0 };
        match senders.single_mut() {
            Ok(mut sender) => {
                tracing::info!(
                    target: "client::ui::hand",
                    msg_type = "C2SPurchaseCard",
                    card_id = ?message.card_id,
                    handler = "handle_grid_card_click_system",
                    "c2s_send: enter"
                );
                sender.send::<ReliableChannel>(message.clone());
            }
            Err(e) => {
                error!(
                    "C2S send failed: type=C2SPurchaseCard, handler=handle_grid_card_click_system, query_err={:?}",
                    e
                );
            }
        }
        outbound.purchase_cards.push(message);
        commands.entity(click.card).insert((
            GridSlotState::Pending,
            PendingPurchaseTimer {
                remaining_ms: timing.purchase_timeout_ms,
            },
        ));
    }
}

pub fn handle_hand_fan_card_click_system(
    mode: Res<HandUiMode>,
    catalog: Res<HandCardCatalog>,
    board_view: Res<PlacementBoardView>,
    mut clicks: MessageReader<HandFanCardClicked>,
    mut pending_placements: ResMut<PendingPlacements>,
    mut disclosure_state: ResMut<PlacementDisclosureState>,
    mut ghost_writer: MessageWriter<GhostPlacementChanged>,
    mut drop_writer: MessageWriter<HandUiPlacementDropResolved>,
    mut commands: Commands,
    hand_cards: Query<(Entity, &FanSlotIndex, &HandSlotCard, Option<&FanSlotState>)>,
    fan_slots: Query<(Entity, &FanSlotIndex, &HandSlotCard), With<FanSlotIndex>>,
    board_cells: Query<(
        &LaneCell,
        Option<&BoardCellOccupied>,
        Option<&ObjectiveCell>,
    )>,
    objectives: Query<(&ObjectiveCell, Option<&ObjectiveAlive>)>,
    mut reserve_strips: Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    mut submit_buttons: Query<&mut Text, With<HandSubmitButton>>,
) {
    for click in clicks.read() {
        let Ok((_entity, _slot_index, card, slot_state)) = hand_cards.get(click.card) else {
            continue;
        };

        if *mode != HandUiMode::Staging {
            continue;
        }

        if slot_state == Some(&FanSlotState::Active) {
            if let Some(target) =
                default_click_stage_target(card.0, &catalog, *board_view, &board_cells, &objectives)
            {
                tracing::info!(
                    target: "client::ui::hand::fan_active_default_drop",
                    card_entity = ?click.card,
                    card_id = ?card.0,
                    default_target = ?target,
                    mode = ?*mode,
                    "fan active default drop"
                );
                drop_writer.write(HandUiPlacementDropResolved {
                    card: click.card,
                    owner_id: board_view.local_player_id,
                    target: Some(target),
                });
            }
            continue;
        }

        if slot_state != Some(&FanSlotState::Ghost) {
            continue;
        }

        let is_instant_ghost = pending_placements
            .target_for(card.0)
            .map(|target| matches!(target, PlayTarget::Instant))
            .unwrap_or(false);

        if is_instant_ghost {
            unstage_card(
                card.0,
                &mut pending_placements,
                &mut ghost_writer,
                &mut commands,
                &fan_slots,
                &mut reserve_strips,
                &mut submit_buttons,
                &mut disclosure_state,
            );
        }
    }
}

pub fn handle_hand_fan_activate_click_system(
    mode: Res<HandUiMode>,
    mut clicks: MessageReader<HandFanCardClicked>,
    hand_cards: Query<&HandSlotCard, With<FanSlotIndex>>,
    mut outbound: ResMut<HandUiOutboundMessages>,
    mut activate_senders: Query<&mut MessageSender<C2SActivateCard>>,
) {
    for click in clicks.read() {
        if !mode.allows_activation() {
            continue;
        }

        let Ok(card) = hand_cards.get(click.card) else {
            continue;
        };

        let message = C2SActivateCard { card_id: card.0 };
        match activate_senders.single_mut() {
            Ok(mut sender) => {
                tracing::info!(
                    target: "client::ui::hand",
                    msg_type = "C2SActivateCard",
                    card_id = ?message.card_id,
                    handler = "handle_hand_fan_activate_click_system",
                    "c2s_send: enter"
                );
                sender.send::<ReliableChannel>(message.clone());
            }
            Err(e) => {
                error!(
                    "C2S send failed: type=C2SActivateCard, handler=handle_hand_fan_activate_click_system, query_err={:?}",
                    e
                );
            }
        }
        outbound.activate_cards.push(message);
    }
}

pub fn handle_placement_drag_started_system(
    mode: Res<HandUiMode>,
    catalog: Res<HandCardCatalog>,
    asset_server: Option<Res<AssetServer>>,
    entities: Option<Res<HandUiEntities>>,
    mut starts: MessageReader<HandUiPlacementDragStarted>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    mut disclosure_state: ResMut<PlacementDisclosureState>,
    mut commands: Commands,
    hand_cards: Query<(&HandSlotCard, Option<&HandPlacementTargetKind>), With<FanSlotIndex>>,
    mut visibility_query: Query<&mut Visibility>,
) {
    for start in starts.read() {
        if *mode != HandUiMode::Staging {
            active_drag.clear();
            disclosure_state.step = PlacementDisclosureStep::Hidden;
            continue;
        }

        let Some(entities) = &entities else {
            active_drag.clear();
            disclosure_state.step = PlacementDisclosureStep::CardSelection;
            continue;
        };

        let Ok((card, target_kind)) = hand_cards.get(start.card) else {
            active_drag.clear();
            disclosure_state.step = PlacementDisclosureStep::CardSelection;
            continue;
        };

        let Some(target_kind) = resolve_placement_target_kind(card.0, target_kind, &catalog) else {
            active_drag.clear();
            disclosure_state.step = PlacementDisclosureStep::CardSelection;
            continue;
        };

        active_drag.start(start.card, card.0, start.owner_id, target_kind);
        disclosure_state.step = PlacementDisclosureStep::TargetSelection { target_kind };
        let prior_visibility = visibility_query.get(entities.drag_sprite).ok().copied();
        tracing::info!(
            target: "client::ui::hand::drag_sprite_visible_flip",
            card_entity = ?start.card,
            card_id = ?card.0,
            prior_visibility = ?prior_visibility,
            drag_sprite_entity = ?entities.drag_sprite,
            target_kind = ?target_kind,
            "drag sprite Visibility flip"
        );
        set_visibility(
            entities.drag_sprite,
            Visibility::Visible,
            &mut visibility_query,
        );
        apply_card_display_art(
            &mut commands,
            entities.drag_sprite,
            catalog.cards.get(&card.0),
            asset_server.as_deref(),
        );
    }
}

pub fn handle_placement_cursor_moved_system(
    mut moves: MessageReader<HandUiPlacementCursorMoved>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    mut active_ghost_drag: ResMut<ActiveGhostUnstageDrag>,
) {
    for cursor_move in moves.read() {
        tracing::debug!(
            target: "client::ui::hand::placement_cursor_move",
            cursor_world_position = ?cursor_move.world_position,
            cursor_screen_position = ?cursor_move.screen_position,
            active_drag_is_active = active_drag.is_active(),
            active_drag_card = ?active_drag.card,
            "placement cursor move"
        );
        if active_drag.is_active() {
            active_drag.cursor_world_position = cursor_move.world_position;
            active_drag.cursor_screen_position = cursor_move.screen_position;
        }

        // PROMPT 1210 — ghost-unstage hit-test reads viewport pixels (Y-down),
        // so it must consume `screen_position`, not the world-space conversion.
        if active_ghost_drag.is_active() {
            active_ghost_drag.cursor_screen_position = cursor_move.screen_position;
        }
    }
}

pub fn handle_placement_drag_ended_system(
    mode: Res<HandUiMode>,
    viewport: Res<HandFanViewport>,
    board_layout: Option<Res<BoardLayout>>,
    entities: Option<Res<HandUiEntities>>,
    mut ends: MessageReader<HandUiPlacementDragEnded>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    mut disclosure_state: ResMut<PlacementDisclosureState>,
    fan_plates: Query<&Node, With<FanPlateDropZone>>,
    objectives: Query<(&ObjectiveCell, Option<&ObjectiveAlive>)>,
    target_units: Query<(&PlacementTargetUnit, &GlobalTransform)>,
    mut drops: MessageWriter<HandUiPlacementDropResolved>,
) {
    for _end in ends.read() {
        if *mode == HandUiMode::Staging {
            // Resolve the drop target by target_kind. PROMPT 683 Phase 4 proved
            // the prior Instant-only gate dropped Minion/TargetObj/LaneWide/
            // TargetUnit drag-ends on the floor before reaching stage_or_update.
            let target = match active_drag.target_kind {
                Some(PlacementTargetKind::Instant) => entities
                    .as_ref()
                    .and_then(|entities| fan_plates.get(entities.fan_root).ok())
                    .and_then(|node| {
                        // PROMPT 1210 — `cursor_over_fan_plate` checks a
                        // viewport-pixel rectangle (Y-down), so it must read
                        // `cursor_screen_position` rather than the world-space
                        // sibling.
                        active_drag
                            .cursor_screen_position
                            .filter(|cursor| cursor_over_fan_plate(*cursor, node, *viewport))
                    })
                    .map(|_cursor| PlayTarget::Instant),
                Some(PlacementTargetKind::Minion) => board_layout
                    .as_deref()
                    .zip(active_drag.cursor_world_position)
                    .and_then(|(layout, cursor)| cursor_to_lane_cell(cursor, layout))
                    .map(|(lane, cell)| PlayTarget::BoardCell { lane, cell }),
                Some(PlacementTargetKind::LaneWide) => board_layout
                    .as_deref()
                    .zip(active_drag.cursor_world_position)
                    .and_then(|(layout, cursor)| cursor_to_lane_cell(cursor, layout))
                    .map(|(lane, _cell)| PlayTarget::LaneWide { lane }),
                Some(PlacementTargetKind::TargetObj) => board_layout
                    .as_deref()
                    .zip(active_drag.cursor_world_position)
                    .and_then(|(layout, cursor)| cursor_to_lane_cell(cursor, layout))
                    .and_then(|(lane, _cell)| {
                        objectives.iter().find_map(|(objective, alive)| {
                            (objective.lane == lane && alive.is_some()).then_some(
                                PlayTarget::TargetObj {
                                    player_id: objective.player_id,
                                    lane: objective.lane,
                                },
                            )
                        })
                    }),
                Some(PlacementTargetKind::TargetUnit) => board_layout
                    .as_deref()
                    .zip(active_drag.cursor_world_position)
                    .and_then(|(layout, cursor)| {
                        target_units.iter().find_map(|(unit, transform)| {
                            if cursor_over_unit(cursor, transform, layout) {
                                let unit_position = transform.translation().truncate();
                                cursor_to_lane_cell(unit_position, layout).map(|(lane, _cell)| {
                                    PlayTarget::TargetUnit {
                                        lane,
                                        unit_id: unit.unit_id,
                                    }
                                })
                            } else {
                                None
                            }
                        })
                    }),
                None => None,
            };

            if let (Some(card), Some(owner_id)) = (active_drag.card, active_drag.owner_id) {
                drops.write(HandUiPlacementDropResolved {
                    card,
                    owner_id,
                    target,
                });
            }
        }

        active_drag.clear();
        disclosure_state.step = PlacementDisclosureStep::CardSelection;
    }
}

/// PROMPT 696 / HU-DRAG-01 — Producer for `HandUiPlacementDragStarted`.
///
/// Reads `bevy_picking`'s buffered `Pointer<Press>` messages, filters to the
/// primary mouse button targeting a `FanSlotIndex` entity during
/// `HandUiMode::Staging`, and emits the start message that the existing
/// `handle_placement_drag_started_system` already consumes. The producer was
/// the proven feature-gap in PROMPT 683 Phase 2: the drag sprite, consumers,
/// and resources were all wired, but nothing was emitting these starts during
/// gameplay.
pub fn produce_fan_slot_drag_started_from_pointer_press_system(
    mode: Res<HandUiMode>,
    board_view: Res<PlacementBoardView>,
    mut presses: MessageReader<Pointer<Press>>,
    fan_slots: Query<(), (With<FanSlotIndex>, With<HandSlotCard>)>,
    mut writer: MessageWriter<HandUiPlacementDragStarted>,
) {
    if *mode != HandUiMode::Staging {
        for _ in presses.read() {}
        return;
    }
    for press in presses.read() {
        if press.button != PointerButton::Primary {
            continue;
        }
        if fan_slots.get(press.entity).is_err() {
            continue;
        }
        writer.write(HandUiPlacementDragStarted {
            card: press.entity,
            owner_id: board_view.local_player_id,
        });
    }
}

/// PROMPT 696 / HU-DRAG-02 — Producer for `HandUiPlacementCursorMoved`.
///
/// While an `ActivePlacementDrag` is live, forwards every buffered `Pointer<Move>`
/// position to the cursor-moved message. Entity-agnostic on purpose: cursor
/// position must continue to update once the cursor leaves the fan slot and
/// passes over the board, which is the entire point of the drag flow.
///
/// PROMPT 1210 — converts `pointer_location.position` (viewport pixels, Y-down)
/// into world-space (Y-up) via `Camera::viewport_to_world_2d` so the downstream
/// `cursor_to_lane_cell` / `cursor_over_unit` math runs in the coordinate frame
/// it actually expects. The raw viewport position is preserved on
/// `screen_position` for the drag sprite and the fan-plate hit test. Defensive
/// against a missing or inactive camera and projection edge cases — no panics
/// and no `unwrap` in the runtime path.
pub fn produce_drag_cursor_moved_from_pointer_move_system(
    active_drag: Res<ActivePlacementDrag>,
    mut moves: MessageReader<Pointer<Move>>,
    mut writer: MessageWriter<HandUiPlacementCursorMoved>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    if !active_drag.is_active() {
        for _ in moves.read() {}
        return;
    }
    let active_camera = cameras.iter().find(|(camera, _)| camera.is_active);
    for ev in moves.read() {
        let screen_position = ev.pointer_location.position;
        let world_position = active_camera.and_then(|(camera, transform)| {
            camera.viewport_to_world_2d(transform, screen_position).ok()
        });
        writer.write(HandUiPlacementCursorMoved {
            world_position,
            screen_position: Some(screen_position),
        });
    }
}

/// PROMPT 1410 / S18-BOARD-PICKING-BACKEND-DRAG-TO-CELL-001 — explicit
/// cursor-to-board-cell producer.
///
/// Reads the primary `Window`'s cursor position every frame the drag is
/// active and emits `HandUiPlacementCursorMoved` independent of the
/// `bevy_picking` `Pointer<Move>` stream. Required because `ui_picking`
/// only generates `Pointer<Move>` events while the cursor is over a UI
/// node — once the cursor leaves the hand-fan into the board area (which
/// has no picking backend), `Pointer<Move>` stops firing and
/// `ActivePlacementDrag.cursor_world_position` goes stale. At drag-end the
/// downstream `cursor_to_lane_cell` then returns `None`, the drop is
/// resolved as `target=None`, the card flips back to `FanSlotState::Active`,
/// and the next click hits the `fan_active_default_drop` fallback that
/// AUDIT-1392-P02 surfaced — never picking the cell under the cursor.
///
/// Conversion math matches `produce_drag_cursor_moved_from_pointer_move_system`:
/// the raw viewport pixel is preserved on `screen_position` and the
/// world-space sibling is computed via `Camera::viewport_to_world_2d` on
/// the first active 2D camera. The producer is a no-op while no drag is
/// active and during the same tick a `Pointer<Move>` already fed the
/// resource (the consumer `handle_placement_cursor_moved_system` simply
/// overwrites with the latest value — last writer wins).
pub fn produce_drag_cursor_moved_from_window_system(
    active_drag: Res<ActivePlacementDrag>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut writer: MessageWriter<HandUiPlacementCursorMoved>,
) {
    if !active_drag.is_active() {
        return;
    }
    // `Query<&Window, With<PrimaryWindow>>::iter().next()` is used rather
    // than `Option<Single<...>>` because the latter declines to match a
    // PrimaryWindow that was spawned *after* HandUiPlugin init in test
    // harnesses driven by raw `App::new()` / `MinimalPlugins`. The
    // production app always spawns its primary window before
    // HandUiPlugin runs, so either pattern works there — Query is the
    // safe lower-bound for both production and test entry orders.
    let Some(window) = windows.iter().next() else {
        return;
    };
    let Some(screen_position) = window.cursor_position() else {
        return;
    };

    let world_position = cameras
        .iter()
        .find(|(camera, _)| camera.is_active)
        .and_then(|(camera, transform)| {
            camera.viewport_to_world_2d(transform, screen_position).ok()
        });

    writer.write(HandUiPlacementCursorMoved {
        world_position,
        screen_position: Some(screen_position),
    });
}

/// PROMPT 696 / HU-DRAG-03 — Producer for `HandUiPlacementDragEnded`.
///
/// Closes the drag on the first primary-button `Pointer<Release>` while an
/// `ActivePlacementDrag` is live. Entity-agnostic so that releases over the
/// board, fan plate, or empty viewport space all terminate the drag — the
/// downstream `handle_placement_drag_ended_system` decides whether the drop
/// resolves to an Instant fan-plate target or no-op (PROMPT 697 handles board
/// cell drops in a follow-up scope).
pub fn produce_drag_ended_from_pointer_release_system(
    active_drag: Res<ActivePlacementDrag>,
    mut releases: MessageReader<Pointer<Release>>,
    mut writer: MessageWriter<HandUiPlacementDragEnded>,
) {
    if !active_drag.is_active() {
        for _ in releases.read() {}
        return;
    }
    let mut emitted = false;
    for ev in releases.read() {
        if ev.button != PointerButton::Primary {
            continue;
        }
        if emitted {
            continue;
        }
        writer.write(HandUiPlacementDragEnded);
        emitted = true;
    }
}

/// PROMPT 696 / HU-DRAG-02, HU-DRAG-04 — Per-frame follow for the drag sprite.
///
/// Mirrors `ActivePlacementDrag::cursor_screen_position` (viewport pixels,
/// Y-down) onto the `HandDragSprite` UI node. `Node.left`/`Node.top` are
/// viewport-space, so the screen-space sibling — not `cursor_world_position`
/// — is the only correct source after the PROMPT 1210 coord-space split.
/// `handle_placement_drag_started_system` flips visibility to `Visible` on
/// drag start and `handle_placement_drag_ended_system` flips it back to
/// `Hidden`; this system only touches `Node.left` / `Node.top` so the sprite
/// trails the cursor for as long as the drag is active.
pub fn sync_hand_drag_sprite_position_system(
    active_drag: Res<ActivePlacementDrag>,
    mut drag_sprite: Query<&mut Node, With<HandDragSprite>>,
) {
    if !active_drag.is_active() {
        return;
    }
    let Some(position) = active_drag.cursor_screen_position else {
        return;
    };
    for mut node in &mut drag_sprite {
        node.left = Val::Px(position.x);
        node.top = Val::Px(position.y);
    }
}

pub fn handle_ghost_drag_ended_system(
    mode: Res<HandUiMode>,
    fan_zone_bounds: Res<FanZoneBounds>,
    mut ends: MessageReader<HandUiPlacementDragEnded>,
    mut active_ghost_drag: ResMut<ActiveGhostUnstageDrag>,
    mut pending_placements: ResMut<PendingPlacements>,
    mut disclosure_state: ResMut<PlacementDisclosureState>,
    mut ghost_writer: MessageWriter<GhostPlacementChanged>,
    mut commands: Commands,
    fan_slots: Query<(Entity, &FanSlotIndex, &HandSlotCard), With<FanSlotIndex>>,
    mut reserve_strips: Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    mut submit_buttons: Query<&mut Text, With<HandSubmitButton>>,
) {
    for _end in ends.read() {
        let should_unstage = *mode == HandUiMode::Staging
            && active_ghost_drag
                .cursor_screen_position
                .map(|position| fan_zone_bounds.contains(position))
                .unwrap_or(false);

        if let (true, Some(card_id)) = (should_unstage, active_ghost_drag.card_id) {
            unstage_card(
                card_id,
                &mut pending_placements,
                &mut ghost_writer,
                &mut commands,
                &fan_slots,
                &mut reserve_strips,
                &mut submit_buttons,
                &mut disclosure_state,
            );
        }

        active_ghost_drag.clear();
    }
}

pub fn handle_placement_drop_resolved_system(
    mode: Res<HandUiMode>,
    catalog: Res<HandCardCatalog>,
    economy: Res<PlayerEconomyView>,
    entities: Option<Res<HandUiEntities>>,
    mut drops: MessageReader<HandUiPlacementDropResolved>,
    mut pending_placements: ResMut<PendingPlacements>,
    mut placement_timer: ResMut<PlacementTimer>,
    mut active_drag: ResMut<ActivePlacementDrag>,
    mut disclosure_state: ResMut<PlacementDisclosureState>,
    mut ghost_writer: MessageWriter<GhostPlacementChanged>,
    mut commands: Commands,
    mut visibility_sets: ParamSet<(
        Query<&mut Visibility>,
        Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    )>,
    fan_slots: Query<(&FanSlotIndex, &HandSlotCard), With<FanSlotIndex>>,
    mut submit_button_sets: ParamSet<(
        Query<&mut Text, With<HandSubmitButton>>,
        Query<(&mut Text, &mut HandSubmitInteractionState), With<HandSubmitButton>>,
    )>,
    mut submit_senders: Query<&mut MessageSender<C2SSubmitPlacement>>,
    mut outbound: ResMut<HandUiOutboundMessages>,
) {
    let Some(entities) = entities else {
        for _drop in drops.read() {}
        return;
    };

    for drop in drops.read() {
        if *mode != HandUiMode::Staging {
            disclosure_state.step = PlacementDisclosureStep::Hidden;
            continue;
        }

        active_drag.clear();
        set_visibility(
            entities.drag_sprite,
            Visibility::Hidden,
            &mut visibility_sets.p0(),
        );
        clear_card_display_art(&mut commands, entities.drag_sprite);
        commands
            .entity(entities.fan_root)
            .remove::<FanPlateHighlighted>();

        let Ok((slot_index, card)) = fan_slots.get(drop.card) else {
            continue;
        };

        let Some(target) = drop.target.clone() else {
            commands.entity(drop.card).insert(FanSlotState::Active);
            disclosure_state.step = PlacementDisclosureStep::CardSelection;
            continue;
        };

        let cost = card_cost_or_default(&catalog, card.0);
        let placement = PlacedCardSubmit {
            card_id: card.0,
            target: target.clone(),
            current_mana_spend: cost,
            reserve_mana_spend: 0,
        };
        let pending_before = pending_placements.staged_count();
        pending_placements.stage_or_update(placement);
        tracing::info!(
            target: "client::ui::hand",
            before = pending_before,
            after = pending_placements.staged_count(),
            card_id = ?card.0,
            cost,
            source = "placement_drop",
            "hand_ui_pending_placement_staged",
        );
        disclosure_state.step = PlacementDisclosureStep::StagedCard;
        ghost_writer.write(GhostPlacementChanged {
            target: Some(target),
            card_id: Some(card.0),
        });
        commands.entity(drop.card).insert(FanSlotState::Ghost);
        {
            let mut submit_texts = submit_button_sets.p0();
            set_submit_count_text(&mut submit_texts, pending_placements.staged_count());
        }
        set_reserve_strip_visibility(
            &mut visibility_sets.p1(),
            slot_index.0,
            visibility_for(cost > 0),
        );

        if placement_timer.in_grace_window && !placement_timer.submitted {
            let mut submit_buttons = submit_button_sets.p1();
            submit_pending_placements(
                &pending_placements,
                entities.submit_button,
                entities.submitted_checkmark,
                &mut submit_buttons,
                &mut submit_senders,
                &mut outbound,
                &mut placement_timer,
                &mut visibility_sets.p0(),
                &economy,
                &mut commands,
                &mut disclosure_state,
            );
        }
    }
}

pub fn handle_reserve_strip_button_interactions_system(
    mode: Res<HandUiMode>,
    economy: Res<PlayerEconomyView>,
    catalog: Res<HandCardCatalog>,
    mut commands: Commands,
    mut interactions: Query<
        (
            Entity,
            &Interaction,
            &ReserveStripButton,
            Option<&ReserveStripButtonDisabled>,
        ),
        Changed<Interaction>,
    >,
    mut pending_placements: ResMut<PendingPlacements>,
    fan_slots: Query<(&FanSlotIndex, Option<&HandSlotCard>, Option<&FanSlotState>)>,
    mut reserve_strips: Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    buttons: Query<(
        Entity,
        &ReserveStripButton,
        Option<&ReserveStripButtonDisabled>,
    )>,
    mut value_texts: Query<(&ReserveStripValueText, &mut Text)>,
) {
    for (_entity, interaction, button, disabled) in &mut interactions {
        if *interaction != Interaction::Pressed
            || *mode != HandUiMode::Staging
            || disabled.is_some()
        {
            continue;
        }

        let Some(card_id) =
            staged_card_for_slot(button.slot_index, &fan_slots, &pending_placements)
        else {
            continue;
        };

        let cost = card_cost_or_default(&catalog, card_id);
        if cost == 0 {
            continue;
        }

        match button.action {
            ReserveStripAction::Increment => {
                let ceiling = reserve_ceiling_for_card(
                    &pending_placements,
                    card_id,
                    cost,
                    economy.reserve_mana,
                );
                pending_placements.increment_reserve_amount(card_id, ceiling);
            }
            ReserveStripAction::Decrement => {
                pending_placements.decrement_reserve_amount(card_id);
            }
        }

        sync_reserve_strip_entities(
            *mode,
            &economy,
            &catalog,
            &pending_placements,
            &mut commands,
            &fan_slots,
            &mut reserve_strips,
            &buttons,
            &mut value_texts,
        );
    }
}

pub fn handle_submit_button_click_system(
    mode: Res<HandUiMode>,
    entities: Option<Res<HandUiEntities>>,
    mut clicks: MessageReader<HandSubmitButtonClicked>,
    pending_placements: Res<PendingPlacements>,
    economy: Res<PlayerEconomyView>,
    mut commands: Commands,
    mut submit_buttons: Query<(&mut Text, &mut HandSubmitInteractionState), With<HandSubmitButton>>,
    mut submit_senders: Query<&mut MessageSender<C2SSubmitPlacement>>,
    mut outbound: ResMut<HandUiOutboundMessages>,
    mut placement_timer: ResMut<PlacementTimer>,
    mut disclosure_state: ResMut<PlacementDisclosureState>,
    mut visibility_query: Query<&mut Visibility>,
) {
    let Some(entities) = entities else {
        for _click in clicks.read() {}
        return;
    };

    for click in clicks.read() {
        if *mode != HandUiMode::Staging || click.button != entities.submit_button {
            continue;
        }

        let submit_is_active = {
            let Ok((_text, interaction_state)) = submit_buttons.get_mut(entities.submit_button)
            else {
                continue;
            };
            *interaction_state == HandSubmitInteractionState::Active
        };

        if !submit_is_active {
            continue;
        }

        submit_pending_placements(
            &pending_placements,
            entities.submit_button,
            entities.submitted_checkmark,
            &mut submit_buttons,
            &mut submit_senders,
            &mut outbound,
            &mut placement_timer,
            &mut visibility_query,
            &economy,
            &mut commands,
            &mut disclosure_state,
        );
    }
}

pub fn sync_submit_validation_error_system(
    entities: Option<Res<HandUiEntities>>,
    pending_placements: Res<PendingPlacements>,
    economy: Res<PlayerEconomyView>,
    submit_errors: Query<&SubmitValidationError, With<HandSubmitButton>>,
    mut disclosure_state: ResMut<PlacementDisclosureState>,
    mut commands: Commands,
) {
    let Some(entities) = entities else {
        return;
    };

    let Ok(error) = submit_errors.get(entities.submit_button) else {
        return;
    };

    if validate_submit_placement_spend(&pending_placements, &economy).is_ok() {
        commands
            .entity(entities.submit_button)
            .remove::<SubmitValidationError>();
        disclosure_state.set_for_staged_count(pending_placements.staged_count());
    } else {
        disclosure_state.step = PlacementDisclosureStep::Correction { error: *error };
    }
}

// PROMPT 1043 — keeps the "X placed" readout in the placement action panel
// in sync with `PendingPlacements.staged_count()`. The readout displays
// "0 placed" while staging is empty and "{N} placed" otherwise; it goes
// blank outside `HandUiMode::Staging` so the chrome does not paint
// orphaned numbers in DraftShop / Auction / DraftInitial.
pub fn sync_placed_count_readout_system(
    mode: Res<HandUiMode>,
    pending_placements: Res<PendingPlacements>,
    entities: Option<Res<HandUiEntities>>,
    mut readouts: Query<&mut Text, With<PlacedCountReadout>>,
) {
    let Some(entities) = entities else {
        return;
    };

    let Ok(mut text) = readouts.get_mut(entities.placed_count_readout) else {
        return;
    };

    text.0.clear();
    if *mode == HandUiMode::Staging {
        let count = pending_placements.staged_count();
        text.0.push_str(&format!("{count} placed"));
    }
}

pub fn sync_placement_disclosure_guidance_system(
    mode: Res<HandUiMode>,
    disclosure_state: Res<PlacementDisclosureState>,
    entities: Option<Res<HandUiEntities>>,
    mut guidance: Query<(&mut Text, &mut Visibility), With<PlacementDisclosureGuidance>>,
) {
    let Some(entities) = entities else {
        return;
    };

    let Ok((mut text, mut visibility)) = guidance.get_mut(entities.placement_disclosure_guidance)
    else {
        return;
    };

    let step = if *mode == HandUiMode::Staging {
        disclosure_state.step
    } else {
        PlacementDisclosureStep::Hidden
    };

    let label = placement_disclosure_label(step);
    text.0.clear();
    text.0.push_str(label);
    *visibility = visibility_for(step != PlacementDisclosureStep::Hidden);
}

pub fn apply_placement_drag_highlights_system(
    mode: Res<HandUiMode>,
    board_layout: Option<Res<BoardLayout>>,
    board_view: Res<PlacementBoardView>,
    catalog: Res<HandCardCatalog>,
    active_drag: Res<ActivePlacementDrag>,
    pending_placements: Res<PendingPlacements>,
    mut commands: Commands,
    board_cells: Query<(
        Entity,
        &LaneCell,
        Option<&BoardCellOccupied>,
        Option<&ObjectiveCell>,
    )>,
    highlighted_cells: Query<Entity, With<BoardCellHighlighted>>,
    highlighted_fan_plates: Query<Entity, With<FanPlateHighlighted>>,
    objectives: Query<(Entity, &ObjectiveCell, Option<&ObjectiveAlive>)>,
    target_units: Query<(Entity, &PlacementTargetUnit, &GlobalTransform)>,
    hovered_units: Query<Entity, With<TargetUnitHover>>,
    mut overlays: Query<&mut Visibility, With<NoValidTargetsOverlay>>,
    entities: Option<Res<HandUiEntities>>,
) {
    if *mode != HandUiMode::Staging || !active_drag.is_active() {
        cleanup_placement_highlights(
            &mut commands,
            &highlighted_cells,
            &highlighted_fan_plates,
            &hovered_units,
            &mut overlays,
        );
        return;
    }

    if active_drag.target_kind == Some(PlacementTargetKind::Instant) {
        set_no_valid_targets_overlay(&mut overlays, Visibility::Hidden);
        sync_target_unit_hover(&mut commands, &hovered_units, None);
        sync_board_cell_highlights(&mut commands, &highlighted_cells, &BTreeSet::new());
        sync_fan_plate_highlight(
            &mut commands,
            &highlighted_fan_plates,
            entities.as_ref().map(|entities| entities.fan_root),
        );
        return;
    }

    let Some(board_layout) = board_layout else {
        cleanup_placement_highlights(
            &mut commands,
            &highlighted_cells,
            &highlighted_fan_plates,
            &hovered_units,
            &mut overlays,
        );
        warn!("BoardLayout missing; placement drag highlights skipped");
        return;
    };

    let desired_highlights = match active_drag.target_kind {
        Some(PlacementTargetKind::Minion) => {
            sync_fan_plate_highlight(&mut commands, &highlighted_fan_plates, None);
            set_no_valid_targets_overlay(&mut overlays, Visibility::Hidden);
            sync_target_unit_hover(&mut commands, &hovered_units, None);
            minion_highlight_cells(
                &board_layout,
                *board_view,
                &catalog,
                &pending_placements,
                &board_cells,
            )
        }
        Some(PlacementTargetKind::TargetObj) => {
            sync_fan_plate_highlight(&mut commands, &highlighted_fan_plates, None);
            set_no_valid_targets_overlay(&mut overlays, Visibility::Hidden);
            sync_target_unit_hover(&mut commands, &hovered_units, None);
            target_objective_highlight_cells(*board_view, &objectives)
        }
        Some(PlacementTargetKind::LaneWide) => {
            sync_fan_plate_highlight(&mut commands, &highlighted_fan_plates, None);
            set_no_valid_targets_overlay(&mut overlays, Visibility::Hidden);
            sync_target_unit_hover(&mut commands, &hovered_units, None);
            lane_wide_highlight_cells(&board_layout, &board_cells)
        }
        Some(PlacementTargetKind::TargetUnit) => {
            sync_fan_plate_highlight(&mut commands, &highlighted_fan_plates, None);
            sync_board_cell_highlights(&mut commands, &highlighted_cells, &BTreeSet::new());
            sync_target_unit_highlights(
                &mut commands,
                &board_layout,
                active_drag.cursor_world_position,
                &target_units,
                &hovered_units,
                &mut overlays,
            );
            return;
        }
        Some(PlacementTargetKind::Instant) => BTreeSet::new(),
        None => BTreeSet::new(),
    };

    sync_fan_plate_highlight(&mut commands, &highlighted_fan_plates, None);
    sync_board_cell_highlights(&mut commands, &highlighted_cells, &desired_highlights);
}

pub fn tick_pending_purchase_timeouts_system(
    mode: Res<HandUiMode>,
    time: Res<Time<Virtual>>,
    mut pending_slots: Query<(Entity, &mut GridSlotState, &mut PendingPurchaseTimer)>,
    mut commands: Commands,
) {
    if *mode != HandUiMode::Grid {
        return;
    }

    let delta_ms = elapsed_ms(time.delta());
    if delta_ms == 0 {
        return;
    }

    for (entity, mut state, mut timer) in &mut pending_slots {
        if *state != GridSlotState::Pending {
            commands.entity(entity).remove::<PendingPurchaseTimer>();
            continue;
        }

        timer.remaining_ms = timer.remaining_ms.saturating_sub(delta_ms);
        if timer.remaining_ms == 0 {
            *state = GridSlotState::Available;
            commands.entity(entity).remove::<PendingPurchaseTimer>();
        }
    }
}

pub fn tick_hand_full_notification_system(
    time: Res<Time<Virtual>>,
    mut notifications: Query<
        (Entity, &mut Visibility, &mut NotificationTimer),
        With<HandFullNotification>,
    >,
    mut commands: Commands,
) {
    let delta_ms = elapsed_ms(time.delta());
    if delta_ms == 0 {
        return;
    }

    for (entity, mut visibility, mut timer) in &mut notifications {
        timer.remaining_ms = timer.remaining_ms.saturating_sub(delta_ms);
        if timer.remaining_ms == 0 {
            *visibility = Visibility::Hidden;
            commands.entity(entity).remove::<NotificationTimer>();
        }
    }
}

pub fn spawn_hand_ui(
    mut commands: Commands,
    existing: Option<Res<HandUiEntities>>,
    placeholder: Option<Res<PlaceholderAssets>>,
    // Sprint 18 story 020 (S18-UI-PLAY-AREA-CONTAINER-001) AC6: the
    // placement action panel parents into `PlayArea` when
    // `PlayAreaPlugin` is registered, instead of `fan_root` inside
    // `HandBar`. Harness apps without the plugin
    // (`client/src/hand_ui_*_harness.rs`) keep parenting the panel
    // into `fan_root` via the `unwrap_or(fan_root)` fallback below.
    play_area_root: Option<Res<crate::ui::PlayAreaRoot>>,
) {
    if existing.is_some() {
        return;
    }
    let Some(placeholder) = placeholder.as_ref() else {
        return;
    };

    // Sprint 14 story 004 (S11-TD-UI-FLEX-STRIPS) — canonical HandBar
    // strip primitive (180 px footprint at viewport bottom edge) wraps
    // the existing `HandFanRoot` (260 px local height). The strip is
    // the viewport-edge-anchored layout box that the responsive matrix
    // invariant (story 005 deterministic strip height) reads against;
    // `HandFanRoot` retains its `f190cc7` chrome contract verbatim and
    // its 260 px height so the existing fan layout — 7 chrome children
    // at 100×100% / 20×20% / 15×15% — is preserved unchanged. The fan
    // extends 80 px above the HandBar footprint via `overflow: visible`
    // on the strip parent (set by `strips::hand_bar_node()`). See
    // `docs/ux/global-ui-design-spec.md` §9 "HandBar vs.
    // HAND_FAN_STRIP_HEIGHT_PX reconciliation".
    // S17-UI-HAND-B0004-CLEANUP-001 Strategy A: HandBar carries `Transform`
    // so the Bevy 0.18 Required Components API derives `GlobalTransform`
    // on the parent. `bevy_ui` `Node` requires `UiTransform` but NOT
    // `Transform`/`GlobalTransform` (verified against bevy_ui-0.18.1
    // `src/ui_node.rs` Node `#[require(...)]` set), so without this
    // insert the `HandFanRoot` child — which explicitly carries
    // `GlobalTransform` for fan-layout queries — would emit the engine
    // `B0004` hierarchy warning every `InSession` entry. `Transform`
    // auto-requires `GlobalTransform` (bevy_transform-0.18.1) so no
    // explicit `GlobalTransform` insert is needed on `HandBar`. Fan
    // layout, drag-state visuals, placement staging, and the Sprint 15
    // story 020 `closed-with-conditions / cannot-reproduce` disposition
    // are all preserved verbatim — this row is ECS hierarchy hygiene
    // only.
    let hand_bar = commands
        .spawn((
            Name::new("Hand UI HandBar"),
            HandUiEntity,
            HandBarRoot,
            strips::HandBar,
            strips::hand_bar_node(),
            Transform::default(),
            Visibility::Inherited,
            z_layers::UI_BASE,
        ))
        .id();

    let fan_root = commands
        .spawn((
            Name::new("Hand UI Fan Root"),
            HandUiEntity,
            HandFanRoot,
            FanPlateDropZone,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                height: Val::Px(HAND_FAN_STRIP_HEIGHT_PX),
                ..default()
            },
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Hidden,
            ChildOf(hand_bar),
        ))
        .id();

    #[cfg(feature = "ui_picking")]
    commands
        .entity(fan_root)
        .insert(bevy::picking::Pickable::IGNORE);

    // Sprint 15 story 020 (S12-UX-HAND-DRAG-STATE-VISUALS-001): fan-plate
    // Instant drop-target overlay — full-cover child of `fan_root`. Hidden
    // by default; flips Visible while an Instant drag is in flight.
    drag_state_visuals::spawn_fan_plate_drop_target_overlay(&mut commands, fan_root);

    let fan_slots = std::array::from_fn(|index| {
        let slot = commands
            .spawn((
                Name::new(format!("Hand UI Fan Slot {index}")),
                HandUiEntity,
                HandCard,
                FanSlotIndex(index as u8),
                FanSlotState::Active,
                Button,
                Interaction::None,
                hidden_slot_node(),
                BackgroundColor(HAND_CARD_SLOT_BACKGROUND),
                Transform::default(),
                Visibility::Hidden,
                ChildOf(fan_root),
            ))
            .id();

        // Sprint 18 story-022 (`S18-UI-CARD-ART-AND-LABEL-STRIP-001`) —
        // canonical `CardSlotArtImage` child sized to
        // `CardSlotKind::HandFan` image inset (4 / 4 / 4 / 28). The
        // per-card art handle binds onto this child (via
        // `sync_hand_fan_card_art_system`) instead of the slot root,
        // structurally enforcing PROMPT 1117 chrome preservation; the
        // attached `ImageNode` carries `NodeImageMode::Auto`
        // (UI-1129-05 banner-stretch resolved). Child of the
        // pre-pooled `FanSlotIndex` entity (ADR-021 Impl Guideline 5
        // preserved — no new top-level pre-pool entry).
        let (art_node, art_z) = card_slot_art_image_node(CardSlotKind::HandFan);
        let art_entity = commands
            .spawn((
                Name::new(format!("Fan Slot {index} Card Art")),
                HandUiEntity,
                CardSlotArtImage,
                art_node,
                art_z,
                card_slot_art_image_component(),
                Visibility::Inherited,
                ChildOf(slot),
            ))
            .id();
        commands.entity(slot).insert(FanSlotArt(art_entity));

        // Chrome children — PAW-002 (sized + positioned absolutely within the
        // fan slot's local box; see HU-card-slot-chrome-layout story for the
        // chosen percent layout. Slot intrinsic box is
        // HAND_CARD_DISPLAY_WIDTH_PX × HAND_CARD_DISPLAY_HEIGHT_PX so % values
        // resolve against a real, non-zero containing block once the slot is
        // promoted to Active by `apply_fan_layout_system`.)
        commands.spawn((
            Name::new(format!("Fan Slot {index} Card Frame")),
            HandCardFrame,
            fan_slot_card_frame_node(),
            ImageNode::new(placeholder.card_frame_common.clone()),
            Visibility::Inherited,
            ChildOf(slot),
        ));
        let atk_badge = commands
            .spawn((
                Name::new(format!("Fan Slot {index} Stat Badge ATK")),
                StatBadgeAtk,
                fan_slot_stat_badge_node(StatBadgeCorner::BottomLeft),
                ImageNode::new(placeholder.stat_badge_atk.clone()),
                Visibility::Inherited,
                ChildOf(slot),
            ))
            .id();
        commands.spawn((
            Name::new(format!("Fan Slot {index} Stat Badge ATK Label")),
            StatBadgeAtkLabel,
            stat_badge_label_node(),
            Text::new(""),
            stat_badge_label_text_font(),
            TextColor(STAT_BADGE_LABEL_COLOR),
            TextLayout::new_with_justify(Justify::Center),
            Visibility::Inherited,
            ChildOf(atk_badge),
        ));
        let hp_badge = commands
            .spawn((
                Name::new(format!("Fan Slot {index} Stat Badge HP")),
                StatBadgeHp,
                fan_slot_stat_badge_node(StatBadgeCorner::BottomRight),
                ImageNode::new(placeholder.stat_badge_hp.clone()),
                Visibility::Inherited,
                ChildOf(slot),
            ))
            .id();
        commands.spawn((
            Name::new(format!("Fan Slot {index} Stat Badge HP Label")),
            StatBadgeHpLabel,
            stat_badge_label_node(),
            Text::new(""),
            stat_badge_label_text_font(),
            TextColor(STAT_BADGE_LABEL_COLOR),
            TextLayout::new_with_justify(Justify::Center),
            Visibility::Inherited,
            ChildOf(hp_badge),
        ));
        let mp_badge = commands
            .spawn((
                Name::new(format!("Fan Slot {index} Stat Badge MP")),
                StatBadgeMp,
                fan_slot_stat_badge_node(StatBadgeCorner::TopLeft),
                ImageNode::new(placeholder.stat_badge_mp.clone()),
                Visibility::Inherited,
                ChildOf(slot),
            ))
            .id();
        commands.spawn((
            Name::new(format!("Fan Slot {index} Stat Badge MP Label")),
            StatBadgeMpLabel,
            stat_badge_label_node(),
            Text::new(""),
            stat_badge_label_text_font(),
            TextColor(STAT_BADGE_LABEL_COLOR),
            TextLayout::new_with_justify(Justify::Center),
            Visibility::Inherited,
            ChildOf(mp_badge),
        ));
        let ar_badge = commands
            .spawn((
                Name::new(format!("Fan Slot {index} Stat Badge AR")),
                StatBadgeAr,
                fan_slot_stat_badge_node(StatBadgeCorner::TopRight),
                ImageNode::new(placeholder.stat_badge_ar.clone()),
                Visibility::Inherited,
                ChildOf(slot),
            ))
            .id();
        commands.spawn((
            Name::new(format!("Fan Slot {index} Stat Badge AR Label")),
            StatBadgeArLabel,
            stat_badge_label_node(),
            Text::new(""),
            stat_badge_label_text_font(),
            TextColor(STAT_BADGE_LABEL_COLOR),
            TextLayout::new_with_justify(Justify::Center),
            Visibility::Inherited,
            ChildOf(ar_badge),
        ));
        commands.spawn((
            Name::new(format!("Fan Slot {index} Rarity Icon")),
            HandRarityIcon,
            fan_slot_icon_node(SlotIconAnchor::TopCenter),
            ImageNode::new(placeholder.rarity_icon_common.clone()),
            Visibility::Inherited,
            ChildOf(slot),
        ));
        commands.spawn((
            Name::new(format!("Fan Slot {index} Type Icon")),
            HandTypeIcon,
            fan_slot_icon_node(SlotIconAnchor::BottomCenter),
            ImageNode::new(placeholder.class_type_icon_neutral.clone()),
            Visibility::Inherited,
            ChildOf(slot),
        ));

        // Sprint 15 story 020 (S12-UX-HAND-DRAG-STATE-VISUALS-001): per-slot
        // drag-state overlay child nodes (dim + hover) spawn here so they
        // sit on top of the chrome children in paint order. Overlays start
        // Hidden; `drag_state_visuals::sync_hand_drag_state_visuals_system`
        // flips them Visible per the resolved drag state.
        drag_state_visuals::spawn_fan_slot_drag_state_overlays(&mut commands, slot, index as u8);

        // PROMPT 1239 — idle playable-affordance overlays. Sibling pathway
        // to Story 020: a Playable border + an Unaffordable dim cover. Both
        // start Hidden and are flipped by `sync_hand_idle_playable_affordance_system`.
        // Distinct marker set from `DragStateOverlay` so Story 020 AC2 query
        // semantics are preserved by construction.
        spawn_fan_slot_playable_affordance_overlays(&mut commands, slot, index as u8);

        slot
    });

    let grid_slots = std::array::from_fn(|index| {
        commands
            .spawn((
                Name::new(format!("Hand UI Draft Grid Slot {index}")),
                HandUiEntity,
                HandDraftGridSlotRoot,
                GridSlotIndex(index as u8),
                Button,
                Interaction::None,
                hand_draft_grid_slot_node(index),
                BackgroundColor(HAND_CARD_SLOT_BACKGROUND),
                Visibility::Hidden,
                ChildOf(fan_root),
            ))
            .id()
    });

    let reserve_strips =
        std::array::from_fn(|index| spawn_reserve_strip(&mut commands, fan_root, index as u8));

    let drag_sprite = commands
        .spawn((
            Name::new("Hand UI Drag Sprite"),
            HandUiEntity,
            HandDragSprite,
            hand_drag_sprite_node(),
            Transform::from_scale(Vec3::splat(HAND_DRAG_SPRITE_SCALE)),
            Visibility::Hidden,
            ChildOf(fan_root),
            z_layers::UI_OVERLAY,
        ))
        .id();

    // PROMPT 1043 — bordered container that hosts the placement-phase
    // action surface (header, disclosure guidance, countdown, placed-count
    // readout, submit button, submitted checkmark). Replaces the previous
    // four-floating-text-fragments layout (`Select a card` / timer digit /
    // `Submit` / `(0` clipped) — reports/PROMPT-1034 §2.3 + §3 D3/D4 and
    // reports/PROMPT-1036 §4.5 documented the affordance as visually
    // unrecognisable, which left every Placement round closing with
    // `committed_players=0`.
    // Sprint 18 story 020 (S18-UI-PLAY-AREA-CONTAINER-001) AC6:
    // placement action panel parents into `PlayArea` instead of
    // `fan_root` when the `PlayAreaRoot` resource is present. This
    // lifts the panel out of the 180 px `HandBar` footprint so it
    // sits above the fan inside the canonical middle band. Harness
    // apps without `PlayAreaPlugin` keep the historical `fan_root`
    // parent so the existing presentation tests preserve their
    // hierarchy invariants.
    let placement_action_panel_parent = play_area_root.as_ref().map(|p| p.0).unwrap_or(fan_root);
    let placement_action_panel = commands
        .spawn((
            Name::new("Hand UI Placement Action Panel"),
            HandUiEntity,
            PlacementActionPanel,
            PlacementActionPanelRoot,
            placement_action_panel_node(),
            BackgroundColor(PLACEMENT_ACTION_PANEL_BACKGROUND),
            BorderColor::all(PLACEMENT_ACTION_PANEL_BORDER),
            Visibility::Hidden,
            ChildOf(placement_action_panel_parent),
        ))
        .id();

    commands.spawn((
        Name::new("Hand UI Placement Action Panel Header"),
        HandUiEntity,
        PlacementActionPanelHeader,
        Text::new("Placement"),
        TextColor(PLACEMENT_ACTION_PANEL_HEADER_COLOR),
        TextFont {
            font_size: typography::H3,
            ..default()
        },
        action_panel_flex_label_node(),
        Visibility::Inherited,
        ChildOf(placement_action_panel),
    ));

    let placement_disclosure_guidance = commands
        .spawn((
            Name::new("Hand UI Placement Disclosure Guidance"),
            HandUiEntity,
            PlacementDisclosureGuidance,
            Text::new(""),
            TextColor(PLACEMENT_ACTION_PANEL_BODY_COLOR),
            TextFont {
                font_size: typography::BODY,
                ..default()
            },
            action_panel_flex_label_node(),
            Visibility::Hidden,
            ChildOf(placement_action_panel),
        ))
        .id();

    // PROMPT 1043 — timer + submitted checkmark live on a single row so
    // the countdown reads as a labelled pill ("Time: 8s") and the OK badge
    // sits adjacent to it after submit.
    let timer_row = commands
        .spawn((
            Name::new("Hand UI Placement Timer Row"),
            HandUiEntity,
            action_panel_row_node(),
            Visibility::Inherited,
            ChildOf(placement_action_panel),
        ))
        .id();

    let timer = commands
        .spawn((
            Name::new("Hand UI Placement Timer"),
            HandUiEntity,
            HandTimer,
            TimerState::Normal,
            Text::new(""),
            TextColor(PLACEMENT_ACTION_PANEL_BODY_COLOR),
            TextFont {
                font_size: typography::H3,
                ..default()
            },
            action_panel_flex_label_node(),
            Visibility::Hidden,
            ChildOf(timer_row),
        ))
        .id();

    let submitted_checkmark = commands
        .spawn((
            Name::new("Hand UI Timer Submitted Checkmark"),
            HandUiEntity,
            TimerSubmittedCheckmark,
            Text::new("OK"),
            TextColor(PLACEMENT_ACTION_PANEL_OK_COLOR),
            TextFont {
                font_size: typography::H3,
                ..default()
            },
            action_panel_flex_label_node(),
            Visibility::Hidden,
            ChildOf(timer_row),
        ))
        .id();

    // PROMPT 1043 — "X / Y placed" readout. Replaces the truncated `(0`
    // micro-copy that the previous 96 px submit button overflowed into.
    let placed_count_readout = commands
        .spawn((
            Name::new("Hand UI Placed Count Readout"),
            HandUiEntity,
            PlacedCountReadout,
            Text::new("0 placed"),
            TextColor(PLACEMENT_ACTION_PANEL_BODY_COLOR),
            TextFont {
                font_size: typography::BODY,
                ..default()
            },
            action_panel_flex_label_node(),
            Visibility::Inherited,
            ChildOf(placement_action_panel),
        ))
        .id();

    // PROMPT 1043 — Submit button promoted from a 96×28 bare-text node to
    // a 200×40 chromed button. Background + border + border-radius make
    // the affordance read as a button at all; the wider intrinsic width
    // also stops the existing "Submit (X cards)" string from wrapping into
    // the `(0` fragment that the audit captured.
    let submit_button = commands
        .spawn((
            Name::new("Hand UI Submit Button"),
            HandUiEntity,
            HandSubmitButton,
            HandSubmitInteractionState::Inactive,
            Button,
            Interaction::None,
            Text::new("Submit (0 cards)"),
            TextColor(PLACEMENT_ACTION_PANEL_BUTTON_TEXT_COLOR),
            TextFont {
                font_size: typography::BODY,
                ..default()
            },
            submit_button_node(),
            BackgroundColor(PLACEMENT_ACTION_PANEL_BUTTON_BACKGROUND),
            BorderColor::all(PLACEMENT_ACTION_PANEL_BUTTON_BORDER),
            Visibility::Hidden,
            ChildOf(placement_action_panel),
        ))
        .id();

    let hand_full_notification = commands
        .spawn((
            Name::new("Hand UI Hand Full Notification"),
            HandUiEntity,
            HandFullNotification,
            Text::new("Hand full"),
            hidden_control_node(120.0, 28.0, 168.0),
            Visibility::Hidden,
            ChildOf(fan_root),
        ))
        .id();

    let no_valid_targets_overlay = commands
        .spawn((
            Name::new("Hand UI No Valid Targets Overlay"),
            HandUiEntity,
            NoValidTargetsOverlay,
            hidden_control_node(180.0, 28.0, 236.0),
            Visibility::Hidden,
            ChildOf(fan_root),
        ))
        .id();

    commands.insert_resource(HandUiEntities {
        hand_bar,
        fan_root,
        fan_slots,
        grid_slots,
        reserve_strips,
        drag_sprite,
        submit_button,
        timer,
        submitted_checkmark,
        hand_full_notification,
        no_valid_targets_overlay,
        placement_disclosure_guidance,
        placement_action_panel,
        placed_count_readout,
    });
}

fn spawn_reserve_strip(commands: &mut Commands, fan_root: Entity, slot_index: u8) -> Entity {
    let strip = commands
        .spawn((
            Name::new(format!("Hand UI Reserve Strip {slot_index}")),
            HandUiEntity,
            ReserveStripForFanSlot(slot_index),
            reserve_strip_node(),
            Visibility::Hidden,
            ChildOf(fan_root),
        ))
        .id();

    spawn_reserve_strip_button(
        commands,
        strip,
        slot_index,
        ReserveStripAction::Decrement,
        "-",
        0.0,
    );

    // PROMPT 1175 (S17-HAND-RESERVE-STRIP-MICROBADGE-CLEANUP / AUDIT-1076-17
    // AC3 carry-forward): the per-staged-card reserve allocation text used to
    // read "Reserve N Current N", which duplicated the canonical HUD
    // `MANA n / N` strip. It now spawns empty and is populated by
    // `set_reserve_value_text` with a bare reserve-mana-spend integer
    // (e.g. `"2"`) when a card is staged. The bare integer is not a
    // duplicate of the HUD mana strip — that strip shows the player's
    // current/cap mana pool; this number is the editable per-card reserve
    // allocation driven by the adjacent `-` / `+` buttons.
    commands.spawn((
        Name::new(format!("Hand UI Reserve Strip Value {slot_index}")),
        HandUiEntity,
        ReserveStripValueText(slot_index),
        Text::new(""),
        reserve_strip_child_node(28.0, 124.0),
        Visibility::Inherited,
        ChildOf(strip),
    ));

    let plus = spawn_reserve_strip_button(
        commands,
        strip,
        slot_index,
        ReserveStripAction::Increment,
        "+",
        156.0,
    );
    commands.entity(plus).insert(ReserveStripButtonDisabled);

    strip
}

fn spawn_reserve_strip_button(
    commands: &mut Commands,
    parent: Entity,
    slot_index: u8,
    action: ReserveStripAction,
    label: &'static str,
    left_px: f32,
) -> Entity {
    commands
        .spawn((
            Name::new(format!("Hand UI Reserve Strip {action:?} {slot_index}")),
            HandUiEntity,
            ReserveStripButton { slot_index, action },
            Button,
            Interaction::None,
            Text::new(label),
            reserve_strip_child_node(left_px, 24.0),
            BackgroundColor(HAND_RESERVE_STRIP_BUTTON_BACKGROUND),
            Visibility::Inherited,
            ChildOf(parent),
        ))
        .id()
}

// ── PROMPT 1239 — Idle playable-affordance overlay spawn + sync ──────────────

fn playable_affordance_overlay_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(0.0),
        top: Val::Percent(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    }
}

fn unaffordable_affordance_overlay_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(0.0),
        top: Val::Percent(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

/// Spawn the two idle-affordance overlay child nodes for a single pre-pooled
/// fan slot. Both overlays start `Visibility::Hidden`; the sync system flips
/// them per the resolved affordance state. The Playable overlay reuses §7
/// `ACCENT` via `drag_state_visuals::accent_color()` as a border; the
/// Unaffordable overlay reuses the existing `OVERLAY_DIM_ALPHA` dim color
/// from `drag_state_visuals::dim_overlay_color()`. No new design tokens are
/// authored.
fn spawn_fan_slot_playable_affordance_overlays(
    commands: &mut Commands,
    slot: Entity,
    slot_index: u8,
) {
    commands.spawn((
        Name::new(format!(
            "Fan Slot {slot_index} Idle Playable Affordance Overlay"
        )),
        #[allow(deprecated)]
        HandUiEntity,
        FanSlotPlayableAffordanceOverlay,
        playable_affordance_overlay_node(),
        BorderColor::all(drag_state_visuals::accent_color()),
        Visibility::Hidden,
        ChildOf(slot),
    ));

    commands.spawn((
        Name::new(format!(
            "Fan Slot {slot_index} Idle Unaffordable Affordance Overlay"
        )),
        #[allow(deprecated)]
        HandUiEntity,
        FanSlotPlayableAffordanceUnaffordableOverlay,
        unaffordable_affordance_overlay_node(),
        BackgroundColor(drag_state_visuals::dim_overlay_color()),
        Visibility::Hidden,
        ChildOf(slot),
    ));
}

/// Returns true iff the player's `current + reserve` mana suffices for the
/// card's mana cost. Mirrors the conservative behaviour of
/// `drag_state_visuals::slot_is_affordable`: cards missing from the catalog
/// default to affordable, and non-Minion card types bypass the mana cost
/// check (matching the current pool, where only Minions cost mana).
fn slot_card_is_affordable(
    card_id: CardId,
    catalog: &HandCardCatalog,
    economy: &PlayerEconomyView,
) -> bool {
    let Some(card) = catalog.cards.get(&card_id) else {
        return true;
    };
    if card.card_type != CardType::Minion {
        return true;
    }
    let available = economy.current_mana.saturating_add(economy.reserve_mana);
    available >= card.cost
}

/// Sync the idle playable-affordance overlays. Read-only over
/// [`CurrentClientPhase`], [`HandUiMode`], [`ActivePlacementDrag`],
/// [`PendingPlacements`], [`PlayerEconomyView`], [`HandCardCatalog`].
/// ADR-002 + ADR-012 binding preserved.
pub fn sync_hand_idle_playable_affordance_system(
    phase: Res<CurrentClientPhase>,
    mode: Res<HandUiMode>,
    active_drag: Res<ActivePlacementDrag>,
    pending_placements: Res<PendingPlacements>,
    economy: Res<PlayerEconomyView>,
    catalog: Res<HandCardCatalog>,
    slots: Query<(Entity, &FanSlotIndex, Option<&HandSlotCard>)>,
    mut playable_overlays: Query<
        (&ChildOf, &mut Visibility),
        With<FanSlotPlayableAffordanceOverlay>,
    >,
    mut unaffordable_overlays: Query<
        (&ChildOf, &mut Visibility),
        (
            With<FanSlotPlayableAffordanceUnaffordableOverlay>,
            Without<FanSlotPlayableAffordanceOverlay>,
        ),
    >,
    mut commands: Commands,
) {
    let phase_ok = phase.phase == RoundPhase::Placement;
    let mode_ok = matches!(*mode, HandUiMode::Passive | HandUiMode::Staging);
    let drag_inactive = !active_drag.is_active();
    let idle_active = phase_ok && mode_ok && drag_inactive;

    let staged_ids: Vec<CardId> = pending_placements
        .placements
        .iter()
        .map(|p| p.card_id)
        .collect();

    let mut slot_states: std::collections::HashMap<Entity, FanSlotPlayableAffordanceActive> =
        std::collections::HashMap::new();

    if idle_active {
        for (slot_entity, _slot_index, slot_card) in slots.iter() {
            let Some(card) = slot_card else {
                continue;
            };
            if staged_ids.contains(&card.0) {
                continue;
            }
            let state = if slot_card_is_affordable(card.0, &catalog, &economy) {
                FanSlotPlayableAffordanceActive::Playable
            } else {
                FanSlotPlayableAffordanceActive::Unaffordable
            };
            slot_states.insert(slot_entity, state);
        }
    }

    for (child_of, mut visibility) in &mut playable_overlays {
        let parent = child_of.parent();
        *visibility = if matches!(
            slot_states.get(&parent),
            Some(FanSlotPlayableAffordanceActive::Playable),
        ) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (child_of, mut visibility) in &mut unaffordable_overlays {
        let parent = child_of.parent();
        *visibility = if matches!(
            slot_states.get(&parent),
            Some(FanSlotPlayableAffordanceActive::Unaffordable),
        ) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (slot_entity, _, _) in slots.iter() {
        match slot_states.get(&slot_entity) {
            Some(state) => {
                commands.entity(slot_entity).insert(*state);
            }
            None => {
                commands
                    .entity(slot_entity)
                    .remove::<FanSlotPlayableAffordanceActive>();
            }
        }
    }
}

fn despawn_hand_ui(mut commands: Commands, entities: Option<Res<HandUiEntities>>) {
    let Some(entities) = entities else {
        return;
    };

    // Sprint 14 story 004: despawn `hand_bar` (the canonical `HandBar`
    // strip primitive at the new top of the hand-UI tree) instead of
    // `fan_root`. `fan_root` is a child of `hand_bar` and is despawned
    // recursively with its parent, so this preserves the previous
    // despawn behaviour while also reclaiming the strip primitive
    // entity (PROMPT 915).
    commands.entity(entities.hand_bar).despawn();
    commands.remove_resource::<HandUiEntities>();
}

fn hidden_slot_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Px(0.0),
        height: Val::Px(0.0),
        ..default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatBadgeCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotIconAnchor {
    TopCenter,
    BottomCenter,
}

/// Stat badge footprint inside a fan slot — kept symmetric so badges read at
/// the four corners (MP top-left, AR top-right, ATK bottom-left, HP bottom-right).
const FAN_SLOT_STAT_BADGE_PERCENT: f32 = 20.0;
/// Rarity / type icon footprint — smaller than stat badges so the top-center
/// rarity glyph stays visually distinct from the corner badges flanking it.
const FAN_SLOT_ICON_PERCENT: f32 = 15.0;
/// Horizontal offset that centers a `FAN_SLOT_ICON_PERCENT`-wide element
/// inside the slot: `(100 - icon_width) / 2`.
const FAN_SLOT_ICON_CENTER_LEFT_PERCENT: f32 = (100.0 - FAN_SLOT_ICON_PERCENT) / 2.0;

fn fan_slot_card_frame_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(0.0),
        top: Val::Percent(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

fn fan_slot_stat_badge_node(corner: StatBadgeCorner) -> Node {
    let (left, right, top, bottom) = match corner {
        StatBadgeCorner::TopLeft => (Val::Percent(0.0), Val::Auto, Val::Percent(0.0), Val::Auto),
        StatBadgeCorner::TopRight => (Val::Auto, Val::Percent(0.0), Val::Percent(0.0), Val::Auto),
        StatBadgeCorner::BottomLeft => (Val::Percent(0.0), Val::Auto, Val::Auto, Val::Percent(0.0)),
        StatBadgeCorner::BottomRight => {
            (Val::Auto, Val::Percent(0.0), Val::Auto, Val::Percent(0.0))
        }
    };
    Node {
        position_type: PositionType::Absolute,
        left,
        right,
        top,
        bottom,
        width: Val::Percent(FAN_SLOT_STAT_BADGE_PERCENT),
        height: Val::Percent(FAN_SLOT_STAT_BADGE_PERCENT),
        ..default()
    }
}

/// PROMPT 1029 — text-label child of each stat badge. Fills the badge's local
/// box so the numeric value (e.g. "3") renders centered over the diamond icon
/// at every viewport. Inheriting visibility keeps the label visible only while
/// the badge itself is visible.
fn stat_badge_label_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(0.0),
        top: Val::Percent(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

/// PROMPT 1029 — stat badge labels share the BODY size token because the
/// numbers must read at 1280×720 inside a `FAN_SLOT_STAT_BADGE_PERCENT` ×
/// `FAN_SLOT_STAT_BADGE_PERCENT` corner of a `HAND_CARD_DISPLAY_WIDTH_PX` ×
/// `HAND_CARD_DISPLAY_HEIGHT_PX` card. Using the token instead of a literal
/// font size keeps typography review in one place.
fn stat_badge_label_text_font() -> TextFont {
    TextFont {
        font_size: typography::BODY,
        ..default()
    }
}

const STAT_BADGE_LABEL_COLOR: Color = Color::srgb(0.98, 0.98, 0.98);

fn fan_slot_icon_node(anchor: SlotIconAnchor) -> Node {
    let (top, bottom) = match anchor {
        SlotIconAnchor::TopCenter => (Val::Percent(0.0), Val::Auto),
        SlotIconAnchor::BottomCenter => (Val::Auto, Val::Percent(0.0)),
    };
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(FAN_SLOT_ICON_CENTER_LEFT_PERCENT),
        top,
        bottom,
        width: Val::Percent(FAN_SLOT_ICON_PERCENT),
        height: Val::Percent(FAN_SLOT_ICON_PERCENT),
        ..default()
    }
}

fn hand_draft_grid_slot_node(index: usize) -> Node {
    let column = index % 3;
    let row = index / 3;
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(96.0 + column as f32 * 132.0),
        top: Val::Px(28.0 + row as f32 * 66.0),
        width: Val::Px(HAND_DRAFT_GRID_CARD_WIDTH_PX),
        height: Val::Px(HAND_DRAFT_GRID_CARD_HEIGHT_PX),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn hand_drag_sprite_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Px(HAND_CARD_DISPLAY_WIDTH_PX),
        height: Val::Px(HAND_CARD_DISPLAY_HEIGHT_PX),
        ..default()
    }
}

fn hidden_control_node(width_px: f32, height_px: f32, bottom_px: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Px(width_px),
        height: Val::Px(height_px),
        bottom: Val::Px(bottom_px),
        ..default()
    }
}

// PROMPT 1043 — Placement Action Panel layout + chrome constants.
//
// Replaces the previous four floating `hidden_control_node(w, h, bottom)`
// absolute placements (reports/PROMPT-1034 §2.3 / §3 D4) with a single
// bordered container anchored to the bottom-right of the viewport, sitting
// outside the centered card-fan footprint so the panel does not occlude
// the cards.
//
// Width / height are chosen so "Submit (0 cards)" — the longest string
// the existing label cycles through — fits the 200 px button without
// wrapping, and the column-flex layout has visual room for the header,
// disclosure caption, timer row, placed-count readout, and submit button
// stacked vertically with `SPACING_SM` gaps.
const PLACEMENT_ACTION_PANEL_WIDTH_PX: f32 = 240.0;
const PLACEMENT_ACTION_PANEL_RIGHT_PX: f32 = 16.0;
const PLACEMENT_ACTION_PANEL_BOTTOM_PX: f32 = 16.0;
const PLACEMENT_ACTION_PANEL_BORDER_PX: f32 = 1.0;
const PLACEMENT_ACTION_SUBMIT_BUTTON_WIDTH_PX: f32 = 200.0;
const PLACEMENT_ACTION_SUBMIT_BUTTON_HEIGHT_PX: f32 = 40.0;

// Panel chrome colors — surface-elevated dark fill with a light translucent
// border so the panel reads as a distinct surface against the dark
// playfield without introducing a fresh palette entry. Mirrors the
// `Color::srgb(0.086, 0.106, 0.153)` SURFACE_ELEVATED pattern already used
// by the lobby class-picker panel (`client/src/ui/lobby.rs`).
const PLACEMENT_ACTION_PANEL_BACKGROUND: Color = Color::srgba(0.086, 0.106, 0.153, 0.94);
const PLACEMENT_ACTION_PANEL_BORDER: Color = Color::srgba(0.82, 0.86, 0.9, 0.40);
const PLACEMENT_ACTION_PANEL_HEADER_COLOR: Color = Color::srgb(0.96, 0.96, 0.98);
const PLACEMENT_ACTION_PANEL_BODY_COLOR: Color = Color::srgb(0.85, 0.88, 0.93);
const PLACEMENT_ACTION_PANEL_OK_COLOR: Color = Color::srgb(0.55, 0.85, 0.55);
const PLACEMENT_ACTION_PANEL_BUTTON_BACKGROUND: Color = Color::srgba(0.20, 0.42, 0.74, 0.95);
const PLACEMENT_ACTION_PANEL_BUTTON_BORDER: Color = Color::srgb(0.40, 0.62, 0.92);
const PLACEMENT_ACTION_PANEL_BUTTON_TEXT_COLOR: Color = Color::srgb(0.98, 0.99, 1.0);
const HAND_CARD_SLOT_BACKGROUND: Color = Color::srgba(0.07, 0.10, 0.14, 0.95);
const HAND_RESERVE_STRIP_BUTTON_BACKGROUND: Color = Color::srgba(0.08, 0.12, 0.16, 0.90);

/// Placement-action-panel `Node` builder. `pub` so the
/// `tests/integration/ui_clean_pass/play_area_budget_test.rs`
/// integration bin can assert the migrated Node shape (AC6 + AC7) at
/// the canonical 1280×720 / 1366×768 / 1920×1080 viewport matrix.
pub fn placement_action_panel_node() -> Node {
    // Sprint 18 story 020 (S18-UI-PLAY-AREA-CONTAINER-001) AC6: placement
    // action panel parents into `PlayArea` (was a child of `fan_root`
    // inside `HandBar` — its `bottom: 16` anchor placed the panel inside
    // the 180 px `HandBar` footprint and produced overlap F-03 per
    // PROMPT 1180 §2 RC-1). AC6 requires `max_height` + scroll-y (or
    // `flex_wrap` / pagination); we adopt the scroll-y branch with
    // `max_height: 100%` so the panel never exceeds the `PlayArea`
    // middle band even when the placement timer + disclosure body row
    // text grows. `overflow: scroll_y()` keeps overflow content reachable
    // without breaking the panel's bottom-right anchor inside `PlayArea`.
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(PLACEMENT_ACTION_PANEL_RIGHT_PX),
        bottom: Val::Px(PLACEMENT_ACTION_PANEL_BOTTOM_PX),
        width: Val::Px(PLACEMENT_ACTION_PANEL_WIDTH_PX),
        max_height: Val::Percent(100.0),
        overflow: Overflow::scroll_y(),
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        justify_content: JustifyContent::FlexStart,
        padding: UiRect::all(Val::Px(spacing::SPACING_MD)),
        row_gap: Val::Px(spacing::SPACING_SM),
        border: UiRect::all(Val::Px(PLACEMENT_ACTION_PANEL_BORDER_PX)),
        border_radius: BorderRadius::all(Val::Px(spacing::SPACING_SM)),
        ..default()
    }
}

fn action_panel_flex_label_node() -> Node {
    Node {
        // Relative flow inside the column-flex panel — no absolute
        // positioning, so each row stacks naturally with the panel's
        // `row_gap`. AC: the chrome composition test guards against
        // accidental `position_type: Absolute` regressions on these
        // children (which would re-introduce the floating-fragment bug).
        position_type: PositionType::Relative,
        width: Val::Percent(100.0),
        ..default()
    }
}

fn action_panel_row_node() -> Node {
    Node {
        position_type: PositionType::Relative,
        width: Val::Percent(100.0),
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexStart,
        column_gap: Val::Px(spacing::SPACING_SM),
        ..default()
    }
}

fn submit_button_node() -> Node {
    Node {
        position_type: PositionType::Relative,
        width: Val::Px(PLACEMENT_ACTION_SUBMIT_BUTTON_WIDTH_PX),
        height: Val::Px(PLACEMENT_ACTION_SUBMIT_BUTTON_HEIGHT_PX),
        display: Display::Flex,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border: UiRect::all(Val::Px(PLACEMENT_ACTION_PANEL_BORDER_PX)),
        border_radius: BorderRadius::all(Val::Px(spacing::SPACING_XS)),
        ..default()
    }
}

fn reserve_strip_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        bottom: Val::Px(0.0),
        width: Val::Px(180.0),
        height: Val::Px(24.0),
        ..default()
    }
}

fn reserve_strip_child_node(left_px: f32, width_px: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(left_px),
        bottom: Val::Px(0.0),
        width: Val::Px(width_px),
        height: Val::Px(24.0),
        ..default()
    }
}

fn visibility_for(visible: bool) -> Visibility {
    if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    }
}

fn set_visibility(
    entity: Entity,
    visibility: Visibility,
    visibility_query: &mut Query<&mut Visibility>,
) {
    if let Ok(mut current) = visibility_query.get_mut(entity) {
        *current = visibility;
    }
}

fn set_submit_count_text(
    submit_buttons: &mut Query<&mut Text, With<HandSubmitButton>>,
    count: usize,
) {
    for mut text in submit_buttons.iter_mut() {
        text.0.clear();
        text.0.push_str(&format!("Submit ({count} cards)"));
    }
}

fn set_timer_text(timer_texts: &mut Query<&mut Text, With<HandTimer>>, remaining_ms: u32) {
    let seconds = if remaining_ms == 0 {
        0
    } else {
        remaining_ms.saturating_add(999) / 1000
    };

    for mut text in timer_texts.iter_mut() {
        text.0.clear();
        text.0.push_str(&seconds.to_string());
    }
}

/// PROMPT 1226 — auto-submit the staged placements when the client observes
/// a Placement → Resolution phase transition.
///
/// **Why:** Without this, a player who staged placements but never clicked
/// Submit would have their `PendingPlacements` cleared by the phase reset
/// below with no `C2SSubmitPlacement` ever leaving the client. PROMPT 1209
/// (f48583d) added a 250 ms server-side grace window for late submissions,
/// but the grace window is moot if the client never sends. This helper
/// fires one final submit before the clear path runs.
///
/// Returns the outcome variant so callers can inspect it; structured tracing
/// covers every short-circuit branch (see `PhaseTransitionAutoSubmitOutcome`).
/// Authority remains server-side — no local state is treated as
/// optimistically accepted.
#[allow(clippy::too_many_arguments)]
fn try_auto_submit_on_phase_transition(
    prev_phase: Option<RoundPhase>,
    current: &CurrentClientPhase,
    identity: &ClientSessionIdentity,
    pending_placements: &PendingPlacements,
    placement_timer: &mut PlacementTimer,
    entities: &HandUiEntities,
    submit_buttons: &mut Query<
        (&mut Text, &mut HandSubmitInteractionState),
        With<HandSubmitButton>,
    >,
    submit_senders: &mut Query<&mut MessageSender<C2SSubmitPlacement>>,
    outbound: &mut HandUiOutboundMessages,
    visibility_query: &mut Query<&mut Visibility>,
    economy: &PlayerEconomyView,
    commands: &mut Commands,
    disclosure_state: &mut PlacementDisclosureState,
) -> PhaseTransitionAutoSubmitOutcome {
    // Only react to the specific Placement → Resolution edge. Every other
    // transition (Placement → Auction during a reconnect rewind, Lobby →
    // Placement at session entry, Resolution → DraftShop next-round, etc.)
    // is a no-op for auto-submit. This branch is intentionally quiet (no
    // tracing) because it fires on every non-target phase change and would
    // otherwise spam the log on healthy round progression.
    if !(current.phase == RoundPhase::Resolution && prev_phase == Some(RoundPhase::Placement)) {
        return PhaseTransitionAutoSubmitOutcome::NotPlacementToResolution;
    }

    let outcome = if identity.player_id.is_none() {
        PhaseTransitionAutoSubmitOutcome::NoLocalPlayer
    } else if pending_placements.placements.is_empty() {
        PhaseTransitionAutoSubmitOutcome::NoPendingPlacements
    } else if placement_timer.submitted {
        PhaseTransitionAutoSubmitOutcome::AlreadySubmitted
    } else {
        let staged_count = pending_placements.staged_count();
        let sent = submit_pending_placements(
            pending_placements,
            entities.submit_button,
            entities.submitted_checkmark,
            submit_buttons,
            submit_senders,
            outbound,
            placement_timer,
            visibility_query,
            economy,
            commands,
            disclosure_state,
        );
        if sent {
            tracing::info!(
                target: "client::ui::hand",
                msg_type = "C2SSubmitPlacement",
                placements_len = staged_count,
                player_id = ?identity.player_id,
                round = current.round,
                handler = "try_auto_submit_on_phase_transition",
                reason = PhaseTransitionAutoSubmitOutcome::Submitted.as_str(),
                "hand_ui_phase_transition_auto_submit"
            );
            return PhaseTransitionAutoSubmitOutcome::Submitted;
        }
        PhaseTransitionAutoSubmitOutcome::InvalidSubmitState
    };

    tracing::warn!(
        target: "client::ui::hand",
        reason = outcome.as_str(),
        staged_count = pending_placements.staged_count(),
        placement_timer_submitted = placement_timer.submitted,
        player_id = ?identity.player_id,
        round = current.round,
        from_phase = ?prev_phase,
        to_phase = ?current.phase,
        "hand_ui_phase_transition_auto_submit_short_circuit"
    );
    outcome
}

fn submit_pending_placements(
    pending_placements: &PendingPlacements,
    submit_button: Entity,
    submitted_checkmark: Entity,
    submit_buttons: &mut Query<
        (&mut Text, &mut HandSubmitInteractionState),
        With<HandSubmitButton>,
    >,
    submit_senders: &mut Query<&mut MessageSender<C2SSubmitPlacement>>,
    outbound: &mut HandUiOutboundMessages,
    placement_timer: &mut PlacementTimer,
    visibility_query: &mut Query<&mut Visibility>,
    economy: &PlayerEconomyView,
    commands: &mut Commands,
    disclosure_state: &mut PlacementDisclosureState,
) -> bool {
    if placement_timer.submitted {
        return false;
    }

    let Ok((mut text, mut interaction_state)) = submit_buttons.get_mut(submit_button) else {
        return false;
    };

    if *interaction_state != HandSubmitInteractionState::Active {
        return false;
    }

    if let Err(error) = validate_submit_placement_spend(pending_placements, economy) {
        commands.entity(submit_button).insert(error);
        disclosure_state.step = PlacementDisclosureStep::Correction { error };
        return false;
    }

    commands
        .entity(submit_button)
        .remove::<SubmitValidationError>();

    let msg = C2SSubmitPlacement {
        placements: pending_placements.placements.clone(),
    };
    match submit_senders.single_mut() {
        Ok(mut sender) => {
            tracing::info!(
                target: "client::ui::hand",
                msg_type = "C2SSubmitPlacement",
                placements_len = msg.placements.len(),
                handler = "submit_pending_placements",
                "c2s_send: enter"
            );
            sender.send::<ReliableChannel>(msg.clone());
        }
        Err(e) => {
            error!(
                "C2S send failed: type=C2SSubmitPlacement, handler=submit_pending_placements, query_err={:?}",
                e
            );
        }
    }
    outbound.submit_placements.push(msg);
    placement_timer.submitted = true;
    placement_timer.in_grace_window = false;
    placement_timer.grace_remaining_ms = 0;
    *interaction_state = HandSubmitInteractionState::Inactive;
    text.0.clear();
    text.0.push_str("Submitted");
    set_visibility(submitted_checkmark, Visibility::Visible, visibility_query);
    disclosure_state.step = PlacementDisclosureStep::Submitted;
    true
}

fn placement_disclosure_label(step: PlacementDisclosureStep) -> &'static str {
    match step {
        PlacementDisclosureStep::Hidden => "",
        PlacementDisclosureStep::CardSelection => "Select a card",
        PlacementDisclosureStep::TargetSelection {
            target_kind: PlacementTargetKind::Minion,
        } => "Choose a lane and cell",
        PlacementDisclosureStep::TargetSelection {
            target_kind: PlacementTargetKind::TargetObj,
        } => "Choose an objective",
        PlacementDisclosureStep::TargetSelection {
            target_kind: PlacementTargetKind::LaneWide,
        } => "Choose a lane",
        PlacementDisclosureStep::TargetSelection {
            target_kind: PlacementTargetKind::TargetUnit,
        } => "Choose a unit",
        PlacementDisclosureStep::TargetSelection {
            target_kind: PlacementTargetKind::Instant,
        } => "Drop on fan plate",
        PlacementDisclosureStep::StagedCard => "Review staged card and mana split",
        PlacementDisclosureStep::Correction { error } => match error {
            SubmitValidationError::ReserveOverdrawn | SubmitValidationError::ManaOverdrawn => {
                "Adjust reserve/current mana"
            }
            // PROMPT 1244 — single corrective hint per server rejection reason.
            SubmitValidationError::ServerRejected { reason } => match reason {
                PlacementRejectedReason::SpawnRangeRejected => {
                    "Server rejected placement: pick a cell inside your spawn range"
                }
                PlacementRejectedReason::OccupancyRejected => {
                    "Server rejected placement: that slot is already taken"
                }
                PlacementRejectedReason::InsufficientMana => {
                    "Server rejected placement: not enough mana for this batch"
                }
                PlacementRejectedReason::InvalidTarget => {
                    "Server rejected placement: pick a valid target"
                }
                PlacementRejectedReason::CardNotInHand
                | PlacementRejectedReason::DuplicateCardId
                | PlacementRejectedReason::CardMissingFromCatalog => {
                    "Server rejected placement: card no longer playable, retry"
                }
                PlacementRejectedReason::WrongPhase
                | PlacementRejectedReason::DuplicateFinalSubmission => {
                    "Server rejected placement: try again next round"
                }
                PlacementRejectedReason::UnknownPlayer
                | PlacementRejectedReason::OwnerMismatch
                | PlacementRejectedReason::MissingCatalog
                | PlacementRejectedReason::MissingEconomy => {
                    "Server rejected placement: contact support"
                }
            },
        },
        PlacementDisclosureStep::Submitted => "Placement submitted",
    }
}

fn validate_submit_placement_spend(
    pending_placements: &PendingPlacements,
    economy: &PlayerEconomyView,
) -> Result<(), SubmitValidationError> {
    let reserve_spend = pending_placements
        .placements
        .iter()
        .fold(0_u32, |sum, placement| {
            sum.saturating_add(placement.reserve_mana_spend)
        });
    if reserve_spend > economy.reserve_mana {
        return Err(SubmitValidationError::ReserveOverdrawn);
    }

    let current_spend = pending_placements
        .placements
        .iter()
        .fold(0_u32, |sum, placement| {
            sum.saturating_add(placement.current_mana_spend)
        });
    if current_spend > economy.current_mana {
        return Err(SubmitValidationError::ManaOverdrawn);
    }

    Ok(())
}

fn cancel_active_placement_drag(
    active_drag: &mut ActivePlacementDrag,
    commands: &mut Commands,
    drag_sprite: Entity,
    fan_root: Entity,
    visibility_query: &mut Query<&mut Visibility>,
    disclosure_state: &mut PlacementDisclosureState,
) {
    if let Some(card) = active_drag.card {
        commands.entity(card).insert(FanSlotState::Active);
    }

    active_drag.clear();
    set_visibility(drag_sprite, Visibility::Hidden, visibility_query);
    commands.entity(fan_root).remove::<FanPlateHighlighted>();
    disclosure_state.step = PlacementDisclosureStep::CardSelection;
}

fn set_reserve_strip_visibility(
    reserve_strips: &mut Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    slot_index: u8,
    visibility: Visibility,
) {
    for (reserve_slot, mut reserve_visibility) in reserve_strips.iter_mut() {
        if reserve_slot.0 == slot_index {
            *reserve_visibility = visibility;
        }
    }
}

fn sync_reserve_strip_entities(
    mode: HandUiMode,
    economy: &PlayerEconomyView,
    catalog: &HandCardCatalog,
    pending_placements: &PendingPlacements,
    commands: &mut Commands,
    fan_slots: &Query<(&FanSlotIndex, Option<&HandSlotCard>, Option<&FanSlotState>)>,
    reserve_strips: &mut Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    buttons: &Query<(
        Entity,
        &ReserveStripButton,
        Option<&ReserveStripButtonDisabled>,
    )>,
    value_texts: &mut Query<(&ReserveStripValueText, &mut Text)>,
) {
    for (reserve_slot, mut visibility) in reserve_strips.iter_mut() {
        let slot_index = reserve_slot.0;
        let card_id = (mode == HandUiMode::Staging)
            .then(|| staged_card_for_slot(slot_index, fan_slots, pending_placements))
            .flatten();
        let cost = card_id
            .map(|card_id| card_cost_or_default(catalog, card_id))
            .unwrap_or(0);
        let (reserve_amount, current_amount) = card_id
            .and_then(|card_id| pending_placements.mana_spend_for(card_id))
            .unwrap_or((0, 0));
        let ceiling = card_id
            .map(|card_id| {
                reserve_ceiling_for_card(pending_placements, card_id, cost, economy.reserve_mana)
            })
            .unwrap_or(0);
        let is_visible = card_id.is_some() && cost > 0;

        *visibility = visibility_for(is_visible);
        set_reserve_value_text(
            value_texts,
            slot_index,
            reserve_amount,
            current_amount,
            cost,
        );
        set_reserve_button_disabled(
            commands,
            buttons,
            slot_index,
            ReserveStripAction::Increment,
            !is_visible || reserve_amount >= ceiling,
        );
        set_reserve_button_disabled(
            commands,
            buttons,
            slot_index,
            ReserveStripAction::Decrement,
            false,
        );
    }
}

fn set_reserve_value_text(
    value_texts: &mut Query<(&ReserveStripValueText, &mut Text)>,
    slot_index: u8,
    reserve_amount: u32,
    _current_amount: u32,
    cost: u32,
) {
    // PROMPT 1175: bare reserve allocation only. No "Reserve" / "Current"
    // wording — that read as a second mana display and duplicated the HUD
    // `MANA n / N` strip (AUDIT-1076-17). Empty when no card is staged.
    for (value_slot, mut text) in value_texts.iter_mut() {
        if value_slot.0 == slot_index {
            if cost == 0 {
                text.0.clear();
            } else {
                text.0 = reserve_amount.to_string();
            }
        }
    }
}

fn set_reserve_button_disabled(
    commands: &mut Commands,
    buttons: &Query<(
        Entity,
        &ReserveStripButton,
        Option<&ReserveStripButtonDisabled>,
    )>,
    slot_index: u8,
    action: ReserveStripAction,
    disabled: bool,
) {
    for (entity, button, disabled_marker) in buttons.iter() {
        if button.slot_index != slot_index || button.action != action {
            continue;
        }

        if disabled && disabled_marker.is_none() {
            commands.entity(entity).insert(ReserveStripButtonDisabled);
        } else if !disabled && disabled_marker.is_some() {
            commands
                .entity(entity)
                .remove::<ReserveStripButtonDisabled>();
        }
    }
}

fn staged_card_for_slot(
    slot_index: u8,
    fan_slots: &Query<(&FanSlotIndex, Option<&HandSlotCard>, Option<&FanSlotState>)>,
    pending_placements: &PendingPlacements,
) -> Option<CardId> {
    fan_slots.iter().find_map(|(fan_slot, card, slot_state)| {
        if fan_slot.0 != slot_index || slot_state != Some(&FanSlotState::Ghost) {
            return None;
        }

        let card_id = card?.0;
        pending_placements
            .reserve_amount_for(card_id)
            .is_some()
            .then_some(card_id)
    })
}

fn card_cost_or_default(catalog: &HandCardCatalog, card_id: CardId) -> u32 {
    catalog
        .cards
        .get(&card_id)
        .map(|card| card.cost)
        .unwrap_or(1)
}

fn reserve_ceiling_for_card(
    pending_placements: &PendingPlacements,
    card_id: CardId,
    cost: u32,
    reserve_mana: u32,
) -> u32 {
    let other_committed = pending_placements.reserve_committed_by_other_cards(card_id);
    cost.min(reserve_mana.saturating_sub(other_committed))
}

fn unstage_card(
    card_id: CardId,
    pending_placements: &mut PendingPlacements,
    ghost_writer: &mut MessageWriter<GhostPlacementChanged>,
    commands: &mut Commands,
    fan_slots: &Query<(Entity, &FanSlotIndex, &HandSlotCard), With<FanSlotIndex>>,
    reserve_strips: &mut Query<(&ReserveStripForFanSlot, &mut Visibility)>,
    submit_buttons: &mut Query<&mut Text, With<HandSubmitButton>>,
    disclosure_state: &mut PlacementDisclosureState,
) -> bool {
    let Some((slot_entity, slot_index)) = fan_slot_for_card(fan_slots, card_id) else {
        return false;
    };

    let pending_before = pending_placements.staged_count();
    if pending_placements.remove_staged(card_id).is_none() {
        return false;
    }
    tracing::info!(
        target: "client::ui::hand",
        before = pending_before,
        after = pending_placements.staged_count(),
        card_id = ?card_id,
        source = "unstage_card",
        "hand_ui_pending_placement_removed",
    );

    ghost_writer.write(GhostPlacementChanged {
        target: None,
        card_id: Some(card_id),
    });
    commands.entity(slot_entity).insert(FanSlotState::Active);
    set_submit_count_text(submit_buttons, pending_placements.staged_count());
    set_reserve_strip_visibility(reserve_strips, slot_index, Visibility::Hidden);
    disclosure_state.set_for_staged_count(pending_placements.staged_count());
    true
}

fn fan_slot_for_card(
    fan_slots: &Query<(Entity, &FanSlotIndex, &HandSlotCard), With<FanSlotIndex>>,
    card_id: CardId,
) -> Option<(Entity, u8)> {
    fan_slots.iter().find_map(|(entity, slot_index, card)| {
        (card.0 == card_id).then_some((entity, slot_index.0))
    })
}

fn is_board_ghost_target(target: &PlayTarget) -> bool {
    matches!(
        target,
        PlayTarget::BoardCell { .. }
            | PlayTarget::TargetUnit { .. }
            | PlayTarget::TargetObj { .. }
            | PlayTarget::LaneWide { .. }
    )
}

fn resolve_placement_target_kind(
    card_id: CardId,
    target_kind: Option<&HandPlacementTargetKind>,
    catalog: &HandCardCatalog,
) -> Option<PlacementTargetKind> {
    if let Some(target_kind) = target_kind {
        return Some(target_kind.0);
    }

    let card = catalog.cards.get(&card_id)?;
    match card.card_type {
        CardType::Minion => Some(PlacementTargetKind::Minion),
        CardType::Field => Some(PlacementTargetKind::LaneWide),
        CardType::Order => Some(PlacementTargetKind::Instant),
        CardType::Spell | CardType::Trap | CardType::Structure | CardType::DoubleFace => None,
    }
}

fn default_click_stage_target(
    card_id: CardId,
    catalog: &HandCardCatalog,
    board_view: PlacementBoardView,
    board_cells: &Query<(
        &LaneCell,
        Option<&BoardCellOccupied>,
        Option<&ObjectiveCell>,
    )>,
    objectives: &Query<(&ObjectiveCell, Option<&ObjectiveAlive>)>,
) -> Option<PlayTarget> {
    match resolve_placement_target_kind(card_id, None, catalog)? {
        PlacementTargetKind::Minion => {
            let (lane, cell) = first_available_spawn_cell(board_view, board_cells)
                .unwrap_or_else(|| fallback_spawn_cell(board_view));
            Some(PlayTarget::BoardCell { lane, cell })
        }
        PlacementTargetKind::LaneWide => Some(PlayTarget::LaneWide { lane: 1 }),
        PlacementTargetKind::TargetObj => objectives
            .iter()
            .filter(|(objective, alive)| {
                objective.player_id == board_view.opponent_player_id && alive.is_some()
            })
            .map(|(objective, _alive)| (objective.lane, objective.player_id))
            .min_by_key(|(lane, _player_id)| *lane)
            .map(|(lane, player_id)| PlayTarget::TargetObj { player_id, lane })
            .or(Some(PlayTarget::TargetObj {
                player_id: board_view.opponent_player_id,
                lane: 1,
            })),
        PlacementTargetKind::TargetUnit => None,
        PlacementTargetKind::Instant => Some(PlayTarget::Instant),
    }
}

fn first_available_spawn_cell(
    board_view: PlacementBoardView,
    board_cells: &Query<(
        &LaneCell,
        Option<&BoardCellOccupied>,
        Option<&ObjectiveCell>,
    )>,
) -> Option<(u8, u8)> {
    board_cells
        .iter()
        .filter_map(|(lane_cell, occupied, objective)| {
            let available = board_view.is_spawn_cell(lane_cell.lane, lane_cell.cell)
                && occupied.is_none()
                && objective.is_none();
            available.then_some((lane_cell.lane, lane_cell.cell))
        })
        .min()
}

fn fallback_spawn_cell(board_view: PlacementBoardView) -> (u8, u8) {
    let cell = match board_view.spawn_edge {
        BoardSpawnEdge::LowCells => 1,
        BoardSpawnEdge::HighCells => BOARD_CELL_COUNT,
    };
    (1, cell)
}

fn minion_highlight_cells(
    board_layout: &BoardLayout,
    board_view: PlacementBoardView,
    catalog: &HandCardCatalog,
    pending_placements: &PendingPlacements,
    board_cells: &Query<(
        Entity,
        &LaneCell,
        Option<&BoardCellOccupied>,
        Option<&ObjectiveCell>,
    )>,
) -> BTreeSet<Entity> {
    let staged_minion_cells = staged_minion_cells(catalog, pending_placements);
    board_cells
        .iter()
        .filter_map(|(entity, lane_cell, occupied, objective)| {
            let _world_xy = board_layout.cell_to_world(lane_cell.lane, lane_cell.cell);
            let valid_cell = board_view.is_spawn_cell(lane_cell.lane, lane_cell.cell)
                && occupied.is_none()
                && objective.is_none()
                && !staged_minion_cells.contains(&(lane_cell.lane, lane_cell.cell));

            valid_cell.then_some(entity)
        })
        .collect()
}

fn target_objective_highlight_cells(
    board_view: PlacementBoardView,
    objectives: &Query<(Entity, &ObjectiveCell, Option<&ObjectiveAlive>)>,
) -> BTreeSet<Entity> {
    objectives
        .iter()
        .filter_map(|(entity, objective, alive)| {
            (objective.player_id == board_view.opponent_player_id && alive.is_some())
                .then_some(entity)
        })
        .collect()
}

fn lane_wide_highlight_cells(
    board_layout: &BoardLayout,
    board_cells: &Query<(
        Entity,
        &LaneCell,
        Option<&BoardCellOccupied>,
        Option<&ObjectiveCell>,
    )>,
) -> BTreeSet<Entity> {
    board_cells
        .iter()
        .filter_map(|(entity, lane_cell, _occupied, objective)| {
            let _world_xy = board_layout.cell_to_world(lane_cell.lane, lane_cell.cell);
            let valid_cell = objective.is_none();

            valid_cell.then_some(entity)
        })
        .collect()
}

fn staged_minion_cells(
    catalog: &HandCardCatalog,
    pending_placements: &PendingPlacements,
) -> BTreeSet<(u8, u8)> {
    pending_placements
        .placements
        .iter()
        .filter_map(|placement| {
            let card = catalog.cards.get(&placement.card_id)?;
            let PlayTarget::BoardCell { lane, cell } = &placement.target else {
                return None;
            };

            (card.card_type == CardType::Minion).then_some((*lane, *cell))
        })
        .collect()
}

fn cleanup_placement_highlights(
    commands: &mut Commands,
    highlighted_cells: &Query<Entity, With<BoardCellHighlighted>>,
    highlighted_fan_plates: &Query<Entity, With<FanPlateHighlighted>>,
    hovered_units: &Query<Entity, With<TargetUnitHover>>,
    overlays: &mut Query<&mut Visibility, With<NoValidTargetsOverlay>>,
) {
    sync_board_cell_highlights(commands, highlighted_cells, &BTreeSet::new());
    sync_fan_plate_highlight(commands, highlighted_fan_plates, None);
    sync_target_unit_hover(commands, hovered_units, None);
    set_no_valid_targets_overlay(overlays, Visibility::Hidden);
}

fn sync_board_cell_highlights(
    commands: &mut Commands,
    highlighted_cells: &Query<Entity, With<BoardCellHighlighted>>,
    desired: &BTreeSet<Entity>,
) {
    let current = highlighted_cells.iter().collect::<BTreeSet<_>>();

    for entity in current.difference(desired) {
        commands.entity(*entity).remove::<BoardCellHighlighted>();
    }

    for entity in desired.difference(&current) {
        commands.entity(*entity).insert(BoardCellHighlighted);
    }
}

fn sync_fan_plate_highlight(
    commands: &mut Commands,
    highlighted_fan_plates: &Query<Entity, With<FanPlateHighlighted>>,
    desired: Option<Entity>,
) {
    for entity in highlighted_fan_plates.iter() {
        if Some(entity) != desired {
            commands.entity(entity).remove::<FanPlateHighlighted>();
        }
    }

    if let Some(entity) = desired {
        commands.entity(entity).insert(FanPlateHighlighted);
    }
}

fn cursor_over_fan_plate(
    cursor_screen_position: Vec2,
    node: &Node,
    viewport: HandFanViewport,
) -> bool {
    let Some(left) = val_px(node.left) else {
        return false;
    };
    let Some(right) = val_px(node.right) else {
        return false;
    };
    let Some(bottom) = val_px(node.bottom) else {
        return false;
    };
    let Some(height) = val_px(node.height) else {
        return false;
    };

    if viewport.width_px <= left + right || viewport.height_px <= bottom || height <= 0.0 {
        return false;
    }

    let min_x = left;
    let max_x = viewport.width_px - right;
    let max_y = viewport.height_px - bottom;
    let min_y = (max_y - height).max(0.0);

    (min_x..=max_x).contains(&cursor_screen_position.x)
        && (min_y..=max_y).contains(&cursor_screen_position.y)
}

fn val_px(value: Val) -> Option<f32> {
    match value {
        Val::Px(px) => Some(px),
        _ => None,
    }
}

fn cursor_to_lane_cell(cursor: Vec2, layout: &BoardLayout) -> Option<(u8, u8)> {
    if layout.cell_width <= 0.0 || layout.lane_height <= 0.0 {
        return None;
    }
    let cell = ((cursor.x - layout.board_origin.x) / layout.cell_width).round() as i32 + 1;
    let lane = ((layout.board_origin.y - cursor.y) / layout.lane_height).round() as i32 + 1;
    if (1..=i32::from(BOARD_LANE_COUNT)).contains(&lane)
        && (1..=i32::from(BOARD_CELL_COUNT)).contains(&cell)
    {
        Some((lane as u8, cell as u8))
    } else {
        None
    }
}

fn sync_target_unit_highlights(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    cursor_world_position: Option<Vec2>,
    target_units: &Query<(Entity, &PlacementTargetUnit, &GlobalTransform)>,
    hovered_units: &Query<Entity, With<TargetUnitHover>>,
    overlays: &mut Query<&mut Visibility, With<NoValidTargetsOverlay>>,
) {
    let mut hovered = None;
    let mut valid_count = 0;

    for (entity, _target_unit, transform) in target_units.iter() {
        valid_count += 1;
        if hovered.is_none()
            && cursor_world_position
                .map(|cursor| cursor_over_unit(cursor, transform, board_layout))
                .unwrap_or(false)
        {
            hovered = Some(entity);
        }
    }

    if valid_count == 0 {
        sync_target_unit_hover(commands, hovered_units, None);
        set_no_valid_targets_overlay(overlays, Visibility::Visible);
        return;
    }

    set_no_valid_targets_overlay(overlays, Visibility::Hidden);
    sync_target_unit_hover(commands, hovered_units, hovered);
}

fn cursor_over_unit(
    cursor_world_position: Vec2,
    transform: &GlobalTransform,
    board_layout: &BoardLayout,
) -> bool {
    let unit_position = transform.translation().truncate();
    let half_width = board_layout.cell_width * 0.5;
    let half_height = board_layout.lane_height * 0.5;

    (cursor_world_position.x - unit_position.x).abs() <= half_width
        && (cursor_world_position.y - unit_position.y).abs() <= half_height
}

fn sync_target_unit_hover(
    commands: &mut Commands,
    hovered_units: &Query<Entity, With<TargetUnitHover>>,
    desired: Option<Entity>,
) {
    for entity in hovered_units.iter() {
        if Some(entity) != desired {
            commands.entity(entity).remove::<TargetUnitHover>();
        }
    }

    if let Some(entity) = desired {
        commands.entity(entity).insert(TargetUnitHover);
    }
}

fn set_no_valid_targets_overlay(
    overlays: &mut Query<&mut Visibility, With<NoValidTargetsOverlay>>,
    visibility: Visibility,
) {
    for mut overlay_visibility in overlays.iter_mut() {
        *overlay_visibility = visibility;
    }
}

fn clear_grid_slot(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).remove::<(
        GridSlotCard,
        GridSlotCardName,
        GridSlotManaCost,
        GridSlotState,
        PendingPurchaseTimer,
    )>();
    clear_card_display_art(commands, entity);
}

fn hide_acquired_grid_slot(
    commands: &mut Commands,
    grid_slots: &mut Query<(Entity, &GridSlotCard, &mut Visibility), With<GridSlotIndex>>,
    card_id: CardId,
) {
    for (entity, card, mut visibility) in grid_slots.iter_mut() {
        if card.0 != card_id {
            continue;
        }

        *visibility = Visibility::Hidden;
        clear_grid_slot(commands, entity);
        break;
    }
}

fn install_card_draw_animation(
    commands: &mut Commands,
    entity: Entity,
    animator: Option<Mut<TweenAnim>>,
    start: Vec3,
    end: Vec3,
    duration_ms: u64,
) {
    let tween = Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(duration_ms),
        TransformPositionLens { start, end },
    );

    if let Some(mut animator) = animator {
        if let Err(error) = replace_tweenable(&mut animator, tween) {
            warn!("Failed to replace Hand UI card draw tween on entity {entity:?}: {error}");
        }
    } else {
        commands.entity(entity).insert(make_tween_anim(tween));
    }
}

fn lock_visible_grid_slots(
    commands: &mut Commands,
    grid_slots: &mut Query<(Entity, &Visibility, Option<&GridSlotCard>), With<GridSlotIndex>>,
) {
    for (entity, visibility, card) in grid_slots.iter_mut() {
        if *visibility != Visibility::Visible || card.is_none() {
            continue;
        }

        commands
            .entity(entity)
            .insert(GridSlotState::HandFullLocked);
        commands.entity(entity).remove::<PendingPurchaseTimer>();
    }
}

fn activate_hand_full_notification<F: QueryFilter>(
    commands: &mut Commands,
    entity: Entity,
    notifications: &mut Query<&mut Visibility, F>,
    duration_ms: u64,
) {
    if let Ok(mut visibility) = notifications.get_mut(entity) {
        *visibility = Visibility::Visible;
    }
    commands.entity(entity).insert(NotificationTimer {
        remaining_ms: duration_ms,
    });
}

fn elapsed_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn cancel_hand_ui_tweens(
    commands: &mut Commands,
    animators: &mut Query<(Entity, &mut TweenAnim), With<HandUiEntity>>,
) {
    for (entity, mut animator) in animators.iter_mut() {
        if let Err(error) = cancel_tween_anim_in_place(&mut animator) {
            warn!("Failed to cancel Hand UI tween on entity {entity:?}: {error}");
        }
        commands.entity(entity).remove::<TweenAnim>();
    }
}
