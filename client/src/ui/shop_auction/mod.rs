use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use lightyear::prelude::{MessageReceiver, MessageSender};
use shared::card::{CardCatalog, CardData, CardId, CardType, Rarity};
use shared::protocol::{
    AuctionSnapshot, BidRejectedReason, C2SPlaceBid, C2SPurchaseCard, C2SRefreshShop,
    C2SSignalReady, ReliableChannel, RoundPhase, S2CAuctionBidAccepted, S2CAuctionBidRejected,
    S2CAuctionCard, S2CAuctionSettled,
};
use shared::session::PlayerId;

use crate::asset_wiring::{
    apply_card_display_art, bid_button_asset, clear_card_display_art, default_client_card_catalog,
    BidButtonChromeState, SHOP_PANEL_CHROME_ASSET, SHOP_SLOT_WELL_IDLE_ASSET,
};
use crate::card_animations::{
    AuctionPanelTransitionRequested, CardAcquiredAnimReady, SettlementOverlayRequested,
};
use crate::presentation::{PlayerEconomyView, PresentationGameSnapshotMessage};
use crate::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use crate::ui::design_tokens::card_slot::{
    card_slot_art_image_component, card_slot_art_image_node, card_slot_label_strip_background_color,
    card_slot_label_strip_node, card_slot_node, CardSlotArtImage, CardSlotKind, CardSlotLabelStrip,
};
use crate::ui::design_tokens::{overlays, spacing, typography, z_layers};
use crate::ui::hud::{HudGoldBroadcastMessage, HudPlayerIds, PhaseTimerState};
use crate::ui::settings::AccessibilityPreferences;
// PROMPT 1347 / S18-AUCTION-WON-CARD-DISPOSITION-001 — disposition state
// reads from the hand-side PlacementTimer + PendingPlacements + FanSlot
// queries so the AC4 banner + AC5 marker + AC11 snapshot lifecycle stays
// in sync with the existing drag/stage/submit pipeline. No reverse
// coupling: hand_ui still does not import from shop_auction.
use crate::ui::hand::{FanSlotIndex, HandSlotCard, PendingPlacements, PlacementTimer};

pub const SHOP_AUCTION_UI_PANEL_ROOT_COUNT: usize = 6;
pub const SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT: usize = 9;
pub const SHOP_AUCTION_UI_SHOP_SLOT_COUNT: usize = 3;
pub const AUCTION_PREPARING_TIMEOUT_MS: u32 = 10_000;
pub const AUCTION_AWAITING_SERVER_DELAY_MS: u32 = 1_500;
pub const AUCTION_TOAST_FADE_IN_MS: u32 = 120;
pub const AUCTION_TOAST_HOLD_MS: u32 = 2_000;
pub const AUCTION_TOAST_FADE_OUT_MS: u32 = 120;
pub const AUCTION_SETTLEMENT_TRANSITION_MS: u32 = 350;
// PROMPT 1116 — pending-state label rendered on the three bid buttons
// between `DraftAuction` phase entry and `S2CAuctionCard` arrival. Before
// this, the buttons spawned with `Text::new("")` and tracked a chrome
// handle that carried the baked-`?` `ui_bid_button_disabled.png` glyph
// — `SOURCE-1077-10`. Keeping it as a single const documents the spawn-
// state contract and lets every per-button update site share the string.
pub const AUCTION_BID_BUTTON_LOADING_LABEL: &str = "Loading…";
pub const DRAFT_INITIAL_OBJECTIVE_COPY: &str = "Select up to 9 cards to keep. You have 45 seconds.";
pub const DRAFT_INITIAL_MODAL_WIDTH_PERCENT: f32 = 88.0;
pub const DRAFT_INITIAL_MODAL_MAX_WIDTH_PX: f32 = 860.0;
// PROMPT 1051 — modal height extended from 300px to 360px to house the
// new keep-decision footer (Ready + Retract Ready + Waiting status) while
// preserving the 3×3 grid band above it. At 360px the modal still fits
// inside the 720px viewport (margin = (720 - 360)/2 = 180px top/bottom)
// and stays inside the 92% max-height guard.
pub const DRAFT_INITIAL_MODAL_HEIGHT_PX: f32 = 360.0;
pub const DRAFT_INITIAL_MODAL_MAX_HEIGHT_PERCENT: f32 = 92.0;
pub const DRAFT_INITIAL_MODAL_PADDING_PX: f32 = spacing::SPACING_LG;
// PROMPT 1051 — footer band height inside the modal. Hosts the Ready /
// Retract Ready button and the waiting-for-opponent status line as a
// single visually-grouped decision row anchored to the modal bottom.
pub const DRAFT_INITIAL_MODAL_FOOTER_HEIGHT_PX: f32 = 64.0;
pub const DRAFT_INITIAL_GRID_COLUMN_WIDTH_PX: f32 = 120.0;
pub const DRAFT_INITIAL_GRID_ROW_HEIGHT_PX: f32 = 56.0;
pub const DRAFT_INITIAL_GRID_COLUMN_GAP_PX: f32 = spacing::SPACING_MD;
pub const DRAFT_INITIAL_GRID_ROW_GAP_PX: f32 = spacing::SPACING_MD;
pub const DRAFT_INITIAL_GRID_WIDTH_PX: f32 =
    DRAFT_INITIAL_GRID_COLUMN_WIDTH_PX * 3.0 + DRAFT_INITIAL_GRID_COLUMN_GAP_PX * 2.0;
pub const DRAFT_INITIAL_GRID_HEIGHT_PX: f32 =
    DRAFT_INITIAL_GRID_ROW_HEIGHT_PX * 3.0 + DRAFT_INITIAL_GRID_ROW_GAP_PX * 2.0;
pub const DRAFT_INITIAL_GRID_LEFT_PX: f32 = spacing::SPACING_XL + spacing::SPACING_XL;
pub const DRAFT_INITIAL_GRID_TOP_PX: f32 = spacing::SPACING_XL + spacing::SPACING_XL;
pub const AUCTION_BID_TARGET_WIDTH_PX: f32 = 108.0;
pub const AUCTION_BID_TARGET_HEIGHT_PX: f32 = 44.0;
pub const AUCTION_BID_FOCUS_RING_WIDTH_PX: f32 = 2.0;
pub const AUCTION_READABILITY_CARD_LEFT_PX: f32 = 112.0;
pub const AUCTION_READABILITY_INFO_LEFT_PX: f32 = 552.0;
pub const AUCTION_READABILITY_INFO_WIDTH_PX: f32 = 468.0;
pub const AUCTION_READABILITY_CONTROL_GAP_PX: f32 = spacing::SPACING_MD;
pub const AUCTION_FREE_GOLD_COUNTER_COUNT: usize = 2;
pub const AUCTION_FREE_GOLD_COUNTER_ANCHOR_LEFT_PERCENT: f32 = 0.0;
pub const AUCTION_FREE_GOLD_COUNTER_LEFT_GAP_PX: f32 = spacing::SPACING_MD;
pub const AUCTION_FREE_GOLD_COUNTER_LEFT_OFFSET_PX: f32 = AUCTION_READABILITY_INFO_LEFT_PX;
pub const AUCTION_FREE_GOLD_COUNTER_BOTTOM_PX: f32 = 132.0;
pub const AUCTION_FREE_GOLD_COUNTER_GROUP_WIDTH_PX: f32 = 240.0;
pub const AUCTION_FREE_GOLD_COUNTER_GROUP_HEIGHT_PX: f32 = 48.0;
pub const AUCTION_FREE_GOLD_COUNTER_WIDTH_PX: f32 = 104.0;
pub const AUCTION_FREE_GOLD_COUNTER_PADDING_PX: f32 = spacing::SPACING_XS + 2.0;
pub const AUCTION_FREE_GOLD_COUNTER_LABEL_FONT_PX: f32 = typography::CAPTION;
pub const AUCTION_FREE_GOLD_COUNTER_VALUE_FONT_PX: f32 = typography::H2;

/// Featured-card pixel footprint — Sprint 14 story 016
/// (`S11-UX-AUCTION-FEATURED-CARD`). Width × height are each strictly
/// larger than any shop slot well (`shop_slot_node` = 136 × 78 px) so
/// the featured auction-up surface reads as the visually dominant card
/// at every canonical viewport (`docs/ux/global-ui-design-spec.md` §8).
pub const AUCTION_FEATURED_CARD_WIDTH_PX: f32 = 380.0;
pub const AUCTION_FEATURED_CARD_HEIGHT_PX: f32 = 280.0;
/// Featured-card frame stroke thickness — chosen from spec §4 spacing
/// scale (`SPACING_XS / 2 ≈ 2 px`, rounded up). Story 016 Implementation
/// Notes line 230-232 binds the *intent* (frame primitive) rather than
/// the exact pixel value; story 018 may extend this constant for the
/// leading / losing state without re-authoring geometry.
pub const AUCTION_FEATURED_CARD_FRAME_THICKNESS_PX: f32 = 3.0;

/// PROMPT 1182 — visible button chrome for primary-action affordances.
/// AUDIT-1129 UI-1129-08 (lobby) and the recurring shop/auction "looks
/// like a label, not a button" reports observed that several primary
/// actions (`DraftInitialReadyButton`, `ShopRefreshButton`,
/// `ShopReadyButton`, `DraftInitialObjectiveDismissButton`,
/// `DraftInitialObjectiveRetrievalButton`) spawned with only a 1px
/// border and no fill — visually indistinguishable from inert status
/// text under the auction / shop chrome's near-black panel background.
/// These constants give every primary-action button a non-transparent
/// `BackgroundColor` and a non-transparent `BorderColor` so it reads
/// unambiguously as an interactive button at every canonical viewport.
///
/// `PRIMARY_ACTION_BG` paints a dark amber-tinted fill that matches the
/// auction pass-button chrome (`auction_pass_button` already shipped
/// with this exact pair). `PRIMARY_ACTION_BORDER` is the same warm
/// off-white outline. Friend-game placeholder palette — final-art
/// replacement remains a separate scope under `PAW-TD-*-a`.
pub fn primary_action_button_background_color() -> Color {
    Color::srgba(0.12, 0.14, 0.18, 0.75)
}

pub fn primary_action_button_border_color() -> Color {
    Color::srgba(0.92, 0.94, 0.96, 0.55)
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShopAuctionUiSystemSet {
    PhaseTransition,
    MessageDrain,
    Input,
    StateSync,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopAuctionUiMode {
    #[default]
    Inactive,
    DraftOffering,
    AuctionPreparing,
    Auction,
    AuctionSettling,
    Shop,
}

impl ShopAuctionUiMode {
    pub fn from_phase(phase: RoundPhase) -> Self {
        match phase {
            RoundPhase::DraftInitial => Self::DraftOffering,
            RoundPhase::DraftAuction => Self::Inactive,
            RoundPhase::DraftShop => Self::Shop,
            RoundPhase::Lobby
            | RoundPhase::Placement
            | RoundPhase::Resolution
            | RoundPhase::GameOver
            | RoundPhase::Handshaking => Self::Inactive,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct ShopAuctionCardCatalog {
    pub cards: CardCatalog,
}

impl Default for ShopAuctionCardCatalog {
    fn default() -> Self {
        Self {
            cards: default_client_card_catalog(),
        }
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionDraftHandView {
    pub hand_size: usize,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ShopAuctionUiOutboundMessages {
    pub purchase_cards: Vec<C2SPurchaseCard>,
    pub refresh_shops: Vec<C2SRefreshShop>,
    pub ready_signals: Vec<C2SSignalReady>,
    pub place_bids: Vec<C2SPlaceBid>,
    pub gold_counter_flash_requests: u32,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionRefreshConfig {
    pub refresh_base_cost: u32,
    pub refresh_cap: u32,
    pub bid_increments: [u32; 3],
}

impl Default for ShopAuctionRefreshConfig {
    fn default() -> Self {
        let cfg = shared::config::GameConfig::default();
        Self {
            refresh_base_cost: cfg.refresh_base_cost,
            refresh_cap: cfg.refresh_cap,
            bid_increments: cfg.bid_increments,
        }
    }
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct ShopAuctionDraftInitialState {
    pub offering_loaded: bool,
    pub ready_signalled: bool,
    pub objective_overlay_visible: bool,
    pub objective_overlay_dismissed: bool,
    pub objective_focus_target: DraftInitialObjectiveFocusTarget,
    pending_confirmed_purchases: Vec<CardId>,
}

impl ShopAuctionDraftInitialState {
    fn reset_phase_state(&mut self) {
        self.offering_loaded = false;
        self.ready_signalled = false;
        self.objective_overlay_visible = false;
        self.objective_overlay_dismissed = false;
        self.objective_focus_target = DraftInitialObjectiveFocusTarget::None;
        self.pending_confirmed_purchases.clear();
    }

    fn show_objective_overlay(&mut self) {
        if !self.offering_loaded {
            return;
        }

        self.objective_overlay_visible = true;
        self.objective_overlay_dismissed = false;
        self.objective_focus_target = DraftInitialObjectiveFocusTarget::DismissButton;
    }

    fn dismiss_objective_overlay(&mut self) {
        if !self.objective_overlay_visible {
            return;
        }

        self.objective_overlay_visible = false;
        self.objective_overlay_dismissed = true;
        self.objective_focus_target = DraftInitialObjectiveFocusTarget::RetrievalAffordance;
    }

    fn queue_purchase_confirmation(&mut self, card_id: CardId) {
        self.pending_confirmed_purchases.push(card_id);
    }
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct ShopAuctionShopState {
    pub slots_loaded: bool,
    pub ready_signalled: bool,
    pub refresh_count_this_draft: u32,
    pub refresh_in_flight: bool,
    pub footer_slots_loaded: bool,
    footer_slots: [Option<CardId>; SHOP_AUCTION_UI_SHOP_SLOT_COUNT],
    buffered_slots: Option<Vec<Option<CardId>>>,
    pending_confirmed_purchases: Vec<CardId>,
}

impl ShopAuctionShopState {
    fn enter_shop_phase(&mut self) {
        self.slots_loaded = false;
        self.ready_signalled = false;
        self.refresh_count_this_draft = 0;
        self.refresh_in_flight = false;
        self.pending_confirmed_purchases.clear();
    }

    fn enter_auction_phase(&mut self) {
        self.clear_phase_state();
    }

    fn clear_phase_state(&mut self) {
        self.slots_loaded = false;
        self.ready_signalled = false;
        self.refresh_count_this_draft = 0;
        self.refresh_in_flight = false;
        self.pending_confirmed_purchases.clear();
    }

    fn clear_all(&mut self) {
        self.clear_phase_state();
        self.buffered_slots = None;
        self.footer_slots_loaded = false;
        self.footer_slots = [None; SHOP_AUCTION_UI_SHOP_SLOT_COUNT];
    }

    fn queue_slots(&mut self, slots: Vec<Option<CardId>>) {
        self.footer_slots = normalized_shop_slots(&slots);
        self.footer_slots_loaded = true;
        self.buffered_slots = Some(slots);
    }

    fn take_buffered_slots(&mut self) -> Option<Vec<Option<CardId>>> {
        self.buffered_slots.take()
    }

    fn queue_purchase_confirmation(&mut self, card_id: CardId) {
        self.pending_confirmed_purchases.push(card_id);
    }

    pub fn footer_slots(&self) -> [Option<CardId>; SHOP_AUCTION_UI_SHOP_SLOT_COUNT] {
        self.footer_slots
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopAuctionAuctionPanelState {
    Hidden,
    Preparing,
    Active,
    Settling,
    ConnectionError,
}

impl Default for ShopAuctionAuctionPanelState {
    fn default() -> Self {
        Self::Hidden
    }
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct ShopAuctionAuctionState {
    pub panel_state: ShopAuctionAuctionPanelState,
    pub card_id: Option<CardId>,
    pub starting_price: u32,
    pub current_price: u32,
    pub current_leader: Option<PlayerId>,
    pub timer_duration_ms: u32,
    pub timer_remaining_ms: u32,
    pub preparing_elapsed_ms: u32,
    pub locally_expired_elapsed_ms: u32,
    pub in_flight_bid_amount: Option<u32>,
    pub pending_bid_accepted: bool,
    pub pending_gold_broadcast_seen: bool,
    pub opponent_bid_gate_satisfied: bool,
    local_gold_broadcast_generation: u32,
    last_completed_gate_generation: u32,
}

impl ShopAuctionAuctionState {
    /// PROMPT 1397 / S18-AUCTION-LEADER-RESET-ON-CARD-001 (AUDIT-1392-P03).
    /// Single chokepoint that ingests an incoming `S2CAuctionCard` and resets
    /// every per-auction field that the previous auction's lifecycle may have
    /// left non-default. The wire message itself carries only `card_id`,
    /// `starting_price`, and `timer_duration_ms`; nothing on the wire clears
    /// `current_leader` (it is set only by `S2CAuctionBidAccepted`). Without
    /// an explicit local reset the previous winner's identity would leak into
    /// the new auction's empty-bid window, painting "OPPONENT LEADING" /
    /// "YOU ARE LEADING" before any bid lands (the bug PROMPT 1392's audit
    /// observed across rounds R3 → R6 in the 2026-05-18 capture).
    ///
    /// Fields reset here:
    /// - `card_id`, `starting_price`, `current_price` — bind to the new card.
    /// - `current_leader = None` — the audit's "leader stickiness" closure.
    /// - `timer_duration_ms` — new wall-clock budget for the countdown.
    /// - `timer_remaining_ms = 0` — Preparing-state default; `enter_active`
    ///   bumps this to `timer_duration_ms` when the live-bidding panel opens.
    /// - `locally_expired_elapsed_ms = 0` — drops any prior local-expiry
    ///   countup so the new auction starts unmarked.
    /// - `clear_bid_resolution_state()` — drops `in_flight_bid_amount` /
    ///   `pending_bid_accepted` / gold-broadcast gate counters so the prior
    ///   auction's in-flight bid + opponent-bid-gate state cannot bleed
    ///   into the new auction's bid-button decision logic.
    ///
    /// `AuctionFeaturedCardLeadLossState` is derived from `current_leader`
    /// in [`auction_featured_card_lead_loss_state`], so clearing
    /// `current_leader` here transitively resets the lead-loss colour band
    /// before `sync_auction_panel_system` reads the resource. The
    /// `MessageDrain` → `StateSync` set ordering in [`ShopAuctionUiPlugin`]
    /// guarantees this happens in the same frame the card arrives.
    fn buffer_card(&mut self, message: &S2CAuctionCard) {
        self.card_id = Some(message.card_id);
        self.starting_price = message.starting_price;
        self.current_price = message.starting_price;
        self.current_leader = None;
        self.timer_duration_ms = message.timer_duration_ms;
        self.timer_remaining_ms = 0;
        self.locally_expired_elapsed_ms = 0;
        self.clear_bid_resolution_state();
    }

    fn enter_preparing(&mut self) {
        // Preserve `timer_duration_ms` — both call sites invoke `buffer_card`
        // immediately before this, populating the live-bidding countdown
        // duration from `S2CAuctionCard.timer_duration_ms`. When the card
        // arrives before the DraftAuction phase change, the transition system
        // reads this field to seed `enter_active`; clearing it here would
        // strand the countdown at 0 (regression seen in
        // `sau_004_card_first_then_phase_activates_countdown`).
        self.panel_state = ShopAuctionAuctionPanelState::Preparing;
        self.preparing_elapsed_ms = 0;
        self.timer_remaining_ms = 0;
        self.locally_expired_elapsed_ms = 0;
        self.clear_bid_resolution_state();
    }

    fn enter_active(&mut self, timer_duration_ms: u32) {
        self.panel_state = ShopAuctionAuctionPanelState::Active;
        self.preparing_elapsed_ms = 0;
        self.timer_duration_ms = timer_duration_ms;
        self.timer_remaining_ms = timer_duration_ms;
        self.locally_expired_elapsed_ms = 0;
    }

    fn restore_from_snapshot(&mut self, snapshot: &AuctionSnapshot, phase_timer_duration_ms: u32) {
        self.clear();
        self.panel_state = ShopAuctionAuctionPanelState::Active;
        self.card_id = Some(snapshot.card_id);
        self.starting_price = snapshot.starting_price;
        self.current_price = if snapshot.last_accepted_bid == 0 {
            snapshot.starting_price
        } else {
            snapshot.last_accepted_bid
        };
        self.current_leader = snapshot.current_leader;
        self.timer_duration_ms = phase_timer_duration_ms.max(snapshot.timer_remaining_ms);
        self.timer_remaining_ms = snapshot.timer_remaining_ms.min(self.timer_duration_ms);
        self.preparing_elapsed_ms = 0;
        self.locally_expired_elapsed_ms = 0;
        self.clear_bid_resolution_state();
    }

    fn enter_settling(&mut self, amount: u32) {
        self.panel_state = ShopAuctionAuctionPanelState::Settling;
        self.preparing_elapsed_ms = 0;
        self.timer_duration_ms = 0;
        self.timer_remaining_ms = 0;
        self.locally_expired_elapsed_ms = 0;
        if amount > 0 {
            self.current_price = amount;
        }
        self.clear_bid_resolution_state();
    }

    fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn record_local_gold_broadcast(&mut self) {
        self.local_gold_broadcast_generation =
            self.local_gold_broadcast_generation.saturating_add(1);
        if self.panel_state != ShopAuctionAuctionPanelState::Active {
            return;
        }

        self.pending_gold_broadcast_seen = true;
        if self.pending_bid_accepted && !self.opponent_bid_gate_satisfied {
            self.opponent_bid_gate_satisfied = true;
            self.last_completed_gate_generation = self.local_gold_broadcast_generation;
        }
    }

    pub fn clear_bid_resolution_state(&mut self) {
        self.in_flight_bid_amount = None;
        self.pending_bid_accepted = false;
        self.pending_gold_broadcast_seen = false;
        self.opponent_bid_gate_satisfied = false;
        self.last_completed_gate_generation = self.local_gold_broadcast_generation;
    }

    fn begin_bid_accepted_gate(&mut self) {
        self.pending_bid_accepted = false;
        self.pending_gold_broadcast_seen =
            self.local_gold_broadcast_generation > self.last_completed_gate_generation;
        self.opponent_bid_gate_satisfied = false;
    }

    fn apply_bid_accepted(
        &mut self,
        message: &S2CAuctionBidAccepted,
        local_player: Option<PlayerId>,
    ) {
        self.current_price = message.amount;
        self.current_leader = Some(message.bidder);
        // PROMPT 1245 — server-anchor the visible remaining time. When the
        // server's `new_timer_ms` exceeds the prior `timer_duration_ms`
        // (e.g. an extension on a late bid), bump the duration so the
        // visual bar can fit it; otherwise the client would clamp the
        // remaining time down and either:
        //   (a) understate remaining seconds, or
        //   (b) leave bid buttons in a phantom-disabled state once the
        //       clamped value ticks back to 0 even though the server
        //       still considers the auction live.
        if message.new_timer_ms > self.timer_duration_ms {
            self.timer_duration_ms = message.new_timer_ms;
        }
        self.timer_remaining_ms = message.new_timer_ms;
        self.locally_expired_elapsed_ms = 0;
        self.in_flight_bid_amount = None;
        self.begin_bid_accepted_gate();

        if Some(message.bidder) == local_player {
            self.pending_gold_broadcast_seen = false;
            self.last_completed_gate_generation = self.local_gold_broadcast_generation;
            return;
        }

        self.pending_bid_accepted = true;
        if self.pending_gold_broadcast_seen {
            self.opponent_bid_gate_satisfied = true;
            self.last_completed_gate_generation = self.local_gold_broadcast_generation;
        }
    }

    pub fn waiting_for_local_gold_after_opponent_bid(&self) -> bool {
        self.pending_bid_accepted && !self.opponent_bid_gate_satisfied
    }

    pub fn panel_visible(&self) -> bool {
        matches!(
            self.panel_state,
            ShopAuctionAuctionPanelState::Preparing
                | ShopAuctionAuctionPanelState::Active
                | ShopAuctionAuctionPanelState::Settling
                | ShopAuctionAuctionPanelState::ConnectionError
        )
    }

    pub fn countdown_active(&self) -> bool {
        self.panel_state == ShopAuctionAuctionPanelState::Active
            && self.timer_duration_ms > 0
            && self.timer_remaining_ms > 0
    }

    pub fn locally_expired(&self) -> bool {
        self.panel_state == ShopAuctionAuctionPanelState::Active
            && self.timer_duration_ms > 0
            && self.timer_remaining_ms == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopAuctionSettlementOutcome {
    LocalWinner,
    OpponentWinner,
    NoBid,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ShopAuctionSettlementState {
    pub outcome: Option<ShopAuctionSettlementOutcome>,
    pub winner: Option<PlayerId>,
    pub amount: u32,
    pub card_id: Option<CardId>,
    pub elapsed_ms: u32,
    pub transition_duration_ms: u32,
    pub transition_active: bool,
    pub local_card_feedback_requests: u32,
    pub overlay_requests: u32,
    pub panel_transition_requests: u32,
}

impl Default for ShopAuctionSettlementState {
    fn default() -> Self {
        Self {
            outcome: None,
            winner: None,
            amount: 0,
            card_id: None,
            elapsed_ms: 0,
            transition_duration_ms: AUCTION_SETTLEMENT_TRANSITION_MS,
            transition_active: false,
            local_card_feedback_requests: 0,
            overlay_requests: 0,
            panel_transition_requests: 0,
        }
    }
}

impl ShopAuctionSettlementState {
    fn begin(
        &mut self,
        outcome: ShopAuctionSettlementOutcome,
        winner: Option<PlayerId>,
        amount: u32,
        card_id: Option<CardId>,
        local_card_feedback: bool,
    ) {
        self.outcome = Some(outcome);
        self.winner = winner;
        self.amount = amount;
        self.card_id = card_id;
        self.elapsed_ms = 0;
        self.transition_duration_ms = AUCTION_SETTLEMENT_TRANSITION_MS;
        self.transition_active = true;
        self.overlay_requests = self.overlay_requests.saturating_add(1);
        self.panel_transition_requests = self.panel_transition_requests.saturating_add(1);
        if local_card_feedback {
            self.local_card_feedback_requests = self.local_card_feedback_requests.saturating_add(1);
        }
    }

    fn clear(&mut self) {
        *self = Self::default();
    }

    fn finish_transition(&mut self) {
        self.transition_active = false;
        self.elapsed_ms = self.transition_duration_ms;
    }

    pub fn transition_progress(&self) -> f32 {
        if self.transition_duration_ms == 0 {
            return 1.0;
        }

        (self.elapsed_ms as f32 / self.transition_duration_ms as f32).min(1.0)
    }

    pub fn overlay_text(&self) -> &'static str {
        match self.outcome {
            Some(ShopAuctionSettlementOutcome::LocalWinner) => "Auction won - card moving to hand",
            Some(ShopAuctionSettlementOutcome::OpponentWinner) => "Opponent won the auction",
            Some(ShopAuctionSettlementOutcome::NoBid) => "No bids - card returned",
            None => "",
        }
    }

    /// PROMPT 1347 / AC7 — dynamic overlay text that names the price for
    /// the loser-side toast. The static [`Self::overlay_text`] is preserved
    /// for callers that prefer the price-free string; the dynamic variant
    /// is rendered by [`sync_settlement_overlay_system`] so the loser sees
    /// the bid commitment alongside the outcome.
    ///
    /// Card-name resolution is deferred to the caller (which has the
    /// catalog handy); this method returns price-aware copy for every
    /// outcome and the caller decorates with the card name when known.
    pub fn dynamic_overlay_text(&self) -> String {
        match self.outcome {
            Some(ShopAuctionSettlementOutcome::LocalWinner) => {
                "Auction won - card moving to hand".to_string()
            }
            Some(ShopAuctionSettlementOutcome::OpponentWinner) => {
                format!("Opponent won for {}g", self.amount)
            }
            Some(ShopAuctionSettlementOutcome::NoBid) => "No bids - card returned".to_string(),
            None => String::new(),
        }
    }
}

/// PROMPT 1347 / S18-AUCTION-WON-CARD-DISPOSITION-001 AC4 / AC5 / AC9 /
/// AC11 — client-side, presentation-only state that records the
/// most-recent auction the local player won and tracks its disposition
/// across the auction-followup PLACEMENT window. Drives:
///
/// - AC4: the "Auction won" affordance banner during PLACEMENT.
/// - AC5: the newly-acquired hand-fan marker.
/// - AC11: the QA snapshot `auction_won_pending` block.
///
/// Not authoritative. The server-side disposition contract
/// (`server/src/feature/auction/system.rs` `award_auction_card` →
/// `S2CCardAcquired { source: CardSource::AuctionWon }`) is unchanged.
/// This resource exists only so the client can render discoverability and
/// snapshot the disposition for observability. ADR-002 + ADR-013 are
/// preserved (AC21).
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionWonPending {
    pub state: Option<AuctionWonPendingState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionWonPendingState {
    /// Card ID of the auction-won card (one-shot per auction settle; the
    /// same `card_id` may appear again later in the same hand from a
    /// non-auction source — that does NOT re-engage this state).
    pub card_id: CardId,
    /// `phase_view.round_number` captured at auction settle. Surfaced on
    /// the snapshot block for cross-correlation with server logs.
    pub settle_round: u32,
    /// `true` once the won card appears in `PendingPlacements` for the
    /// current PLACEMENT phase. Drives AC4 banner + AC5 marker hide; the
    /// snapshot block continues to surface this flag until the state
    /// clears.
    pub staged_yet: bool,
    /// `true` once `PlacementTimer::submitted` has fired while the won
    /// card was staged. The state clears (becomes `None`) on the same
    /// frame the submit lands so the AC11 block becomes absent per the
    /// "submit clears the block" rule.
    pub submitted_yet: bool,
}

impl AuctionWonPending {
    /// AC4 / AC5: visible UI affordance gate. Banner + marker render only
    /// while the won card is in the hand AND has not yet been staged AND
    /// the local player is in the auction-followup PLACEMENT phase.
    pub fn affordance_visible(self, current_phase: RoundPhase) -> bool {
        matches!(current_phase, RoundPhase::Placement)
            && self
                .state
                .map(|s| !s.staged_yet)
                .unwrap_or(false)
    }

    /// AC11: snapshot-block gate. The block is emitted while the
    /// disposition is pending AND the local player is in PLACEMENT. The
    /// block reflects `staged_yet` / `submitted_yet` inside itself but
    /// does NOT depend on them for presence.
    pub fn snapshot_block_active(self, current_phase: RoundPhase) -> bool {
        matches!(current_phase, RoundPhase::Placement) && self.state.is_some()
    }

    pub fn card_id(self) -> Option<CardId> {
        self.state.map(|s| s.card_id)
    }

    /// Sets a fresh pending state on auction settle. Overwrites any prior
    /// pending state — the contract is one-shot-per-settle.
    pub fn arm(&mut self, card_id: CardId, settle_round: u32) {
        self.state = Some(AuctionWonPendingState {
            card_id,
            settle_round,
            staged_yet: false,
            submitted_yet: false,
        });
    }

    /// One-shot clear. After clearing the marker does NOT re-appear in a
    /// later PLACEMENT phase even if the same card is re-staged (AC9).
    pub fn clear(&mut self) {
        self.state = None;
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionShopTimerState {
    pub duration_ms: u32,
    pub remaining_ms: u32,
    pub started: bool,
    pub deferred: bool,
}

impl ShopAuctionShopTimerState {
    fn defer(&mut self, duration_ms: u32) {
        self.duration_ms = duration_ms;
        self.remaining_ms = duration_ms;
        self.started = false;
        self.deferred = true;
    }

    fn start(&mut self, duration_ms: u32) {
        self.duration_ms = duration_ms;
        self.remaining_ms = duration_ms;
        self.started = true;
        self.deferred = false;
    }

    fn restore_from_snapshot(&mut self, phase_duration_ms: u32, remaining_ms: Option<u32>) {
        let remaining_ms = remaining_ms.unwrap_or(phase_duration_ms);
        self.duration_ms = phase_duration_ms.max(remaining_ms);
        self.remaining_ms = remaining_ms.min(self.duration_ms);
        self.started = true;
        self.deferred = false;
    }

    fn stop(&mut self) {
        *self = Self::default();
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionLocalGoldView {
    pub player_id: Option<PlayerId>,
    pub gold: u32,
    pub reserved_gold: u32,
    pub initialized: bool,
}

impl ShopAuctionLocalGoldView {
    pub fn free_gold(self, economy: &PlayerEconomyView) -> u32 {
        let gold = if self.initialized {
            self.gold
        } else {
            economy.gold
        };
        let reserved_gold = if self.initialized {
            self.reserved_gold
        } else {
            0
        };
        local_free_gold(gold, reserved_gold)
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct ShopAuctionUiEntities {
    pub root: Entity,
    pub draft_offering_panel: Entity,
    pub draft_initial_modal_panel: Entity,
    pub draft_initial_modal_footer: Entity,
    pub draft_initial_grid: Entity,
    pub draft_initial_slots: [Entity; SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT],
    pub draft_initial_bought_overlays: [Entity; SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT],
    pub draft_initial_ready_button: Entity,
    pub draft_initial_ready_status: Entity,
    pub draft_initial_hand_full_banner: Entity,
    pub draft_initial_objective_overlay: Entity,
    pub draft_initial_objective_copy: Entity,
    pub draft_initial_objective_dismiss_button: Entity,
    pub draft_initial_objective_retrieval_button: Entity,
    pub draft_initial_countdown_label: Entity,
    pub shop_panel: Entity,
    pub shop_phase_title: Entity,
    pub shop_empty_state: Entity,
    pub shop_slots: [Entity; SHOP_AUCTION_UI_SHOP_SLOT_COUNT],
    /// PROMPT 1085 — child affordance label per shop slot. Index matches
    /// the parent slot entity in `shop_slots`.
    pub shop_slot_affordance_labels: [Entity; SHOP_AUCTION_UI_SHOP_SLOT_COUNT],
    pub shop_refresh_button: Entity,
    pub shop_ready_button: Entity,
    pub shop_ready_status: Entity,
    pub shop_hand_full_banner: Entity,
    pub auction_panel: Entity,
    pub auction_featured_card: Entity,
    pub auction_featured_card_frame: Entity,
    /// Sprint 18 story-022 — canonical `CardSlotArtImage` child of the
    /// featured card. The per-card art handle binds onto this entity
    /// (via `apply_card_display_art`) instead of the slot root.
    pub auction_featured_card_art: Entity,
    /// Sprint 18 story-022 — canonical `CardSlotLabelStrip` child of
    /// the featured card. Parent of the four text readouts (stats /
    /// keyword / price / timer) per AC9.
    pub auction_featured_card_label_strip: Entity,
    pub auction_featured_card_stats: Entity,
    pub auction_featured_card_keyword: Entity,
    /// PROMPT 1085 — explicit current-price readout on the featured card.
    pub auction_featured_card_price_label: Entity,
    /// PROMPT 1085 — numeric time-left readout on the featured card.
    pub auction_featured_card_timer_label: Entity,
    pub auction_status_text: Entity,
    pub auction_timer_bar: Entity,
    pub auction_bid_status_text: Entity,
    pub auction_free_gold_counter_group: Entity,
    pub auction_free_gold_counters: [Entity; AUCTION_FREE_GOLD_COUNTER_COUNT],
    pub auction_free_gold_counter_labels: [Entity; AUCTION_FREE_GOLD_COUNTER_COUNT],
    pub auction_free_gold_counter_values: [Entity; AUCTION_FREE_GOLD_COUNTER_COUNT],
    pub auction_bid_buttons: [Entity; 3],
    pub auction_pass_button: Entity,
    pub shop_footer: Entity,
    pub shop_footer_slots: [Entity; SHOP_AUCTION_UI_SHOP_SLOT_COUNT],
    pub toast_root: Entity,
    pub toast_text: Entity,
    pub settlement_overlay: Entity,
    pub settlement_overlay_text: Entity,
}

impl ShopAuctionUiEntities {
    pub fn panel_roots(self) -> [Entity; SHOP_AUCTION_UI_PANEL_ROOT_COUNT] {
        [
            self.draft_offering_panel,
            self.shop_panel,
            self.auction_panel,
            self.shop_footer,
            self.toast_root,
            self.settlement_overlay,
        ]
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[deprecated(
    since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001",
    note = "Universal shop/auction marker is too coarse for QA snapshot counts \
            (SOURCE-1077-08: snapshots showed shop_auction_entities = 78 across \
            every phase). Use per-sub-surface root markers via the existing \
            ShopAuctionPanelRoot enum (DraftOffering / Shop / Auction / ShopFooter / \
            Toast / SettlementOverlay) for visibility-aware counting. The deprecated \
            marker stays on existing entities for one Sprint cycle so historical \
            PROMPT 1022 / 1034 / 1036 snapshot comparisons still resolve."
)]
pub struct ShopAuctionUiEntity;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionUiRoot;

/// Per-sub-surface root marker for the shop/auction UI panels. Each variant
/// is applied to exactly one panel-root entity by [`spawn_panel_root`].
///
/// S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 — this enum is the canonical
/// per-sub-surface marker consumed by
/// [`crate::presentation::qa_snapshot::UiCountQueries`] for visibility-aware
/// counting (replaces the deprecated [`ShopAuctionUiEntity`] universal
/// marker for that purpose).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopAuctionPanelRoot {
    DraftOffering,
    Shop,
    Auction,
    ShopFooter,
    Toast,
    SettlementOverlay,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialModalPanel;

// PROMPT 1051 — marker for the keep-decision footer band anchored to the
// bottom of the keep-9 modal panel. Parents the Ready / Retract Ready
// button and the waiting-for-opponent status text so they read as a
// single footer-attached decision row instead of a detached side action.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialModalFooter;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialGrid;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionBidButton {
    pub increment: u32,
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionBidButtonState {
    #[default]
    GenericDisabled,
    Enabled,
    Unaffordable,
    HandFullLocked,
    InFlight,
    LocallyExpired,
    HiddenLeading,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct AuctionBidTargetBounds {
    pub width_px: f32,
    pub height_px: f32,
}

impl AuctionBidTargetBounds {
    pub const fn meets_minimum_target(self) -> bool {
        self.width_px >= 44.0 && self.height_px >= 44.0
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct AuctionBidFocusState {
    pub order: u8,
    pub focusable: bool,
    pub focused: bool,
    pub focus_ring_visible: bool,
    pub focus_ring_width_px: f32,
}

impl AuctionBidFocusState {
    pub const fn inactive(order: u8) -> Self {
        Self {
            order,
            focusable: false,
            focused: false,
            focus_ring_visible: false,
            focus_ring_width_px: 0.0,
        }
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionBidKeyboardFocus {
    pub focused_button: Option<Entity>,
}

/// PROMPT 1042 — local-only "I am passing this auction" state. The auction
/// protocol has no server-side Pass message; the auction simply expires if
/// the player does not bid. The Pass affordance documents that intent
/// visually: clicking Pass sets this resource which dims the bid buttons
/// and labels the Pass button "PASSED" until the next auction card arrives
/// or the phase changes.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionLocallyPassed {
    pub passed: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionPassButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionBidStatusText;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionFeaturedCard;

/// Sprint 14 story 016 (`S11-UX-AUCTION-FEATURED-CARD`) — stable marker
/// for the explicit visual frame surrounding the featured auction-up
/// card. Story 018 (`S12-UX-AUCTION-LEAD-LOSS-STATE-001`) extends this
/// primitive by re-coloring the frame border; the geometry is owned
/// here.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionFeaturedCardFrame;

/// Sprint 14 story 018 (`S12-UX-AUCTION-LEAD-LOSS-STATE-001`) -
/// strict, test-observable visual state for the featured-card frame.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionFeaturedCardLeadLossState {
    Neutral,
    Leading,
    Losing,
}

/// Sprint 14 story 016 — stable marker for the ATK / HP / cost stats
/// readout sitting inside the featured card. Carries `TextFont`
/// `H2 = 22 px` so AC4's numeric hierarchy assertion (name > stats >
/// keyword) is observable without touching layout.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionFeaturedCardStats;

/// Sprint 14 story 016 — stable marker for the keyword / rarity text
/// sitting beneath the stats readout. Carries `TextFont` `BODY = 15 px`
/// so AC4's numeric hierarchy assertion is observable.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionFeaturedCardKeyword;

/// PROMPT 1085 — prominent current-price readout overlaid on the featured
/// auction card. Audit AUDIT-1076-04: the prior layout encoded the price
/// inline with the card name on the featured-card root `Text`, which
/// collapsed against the card's `ImageNode` background and was unreadable.
/// The dedicated price label is the canonical surface so `"Bid: Ng"` is
/// always visible while the auction is active or settling.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionFeaturedCardPriceLabel;

/// PROMPT 1085 — numeric time-left readout overlaid on the featured
/// auction card. Audit AUDIT-1076-04 §State / Value Correlation Audit
/// noted "no visible countdown for auction" — the timer-bar width was the
/// only countdown surface and the bid row hid it. The numeric label sits
/// inside the card so the remaining bid window is legible regardless of
/// the bar's pixel state.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionFeaturedCardTimerLabel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuctionFreeGoldCounterKind {
    Interest,
    RefundedBid,
}

impl AuctionFreeGoldCounterKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Interest => "INTEREST",
            Self::RefundedBid => "BID REFUND",
        }
    }
}

pub const AUCTION_FREE_GOLD_COUNTER_KINDS: [AuctionFreeGoldCounterKind;
    AUCTION_FREE_GOLD_COUNTER_COUNT] = [
    AuctionFreeGoldCounterKind::Interest,
    AuctionFreeGoldCounterKind::RefundedBid,
];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionFreeGoldCounterGroup;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionFreeGoldCounter {
    pub kind: AuctionFreeGoldCounterKind,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionFreeGoldCounterLabel {
    pub kind: AuctionFreeGoldCounterKind,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionFreeGoldCounterValue {
    pub kind: AuctionFreeGoldCounterKind,
    pub amount: u32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionStatusText;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionTimerBar;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionTimerBarState {
    pub greyed: bool,
    pub countdown_active: bool,
    pub connection_error: bool,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq)]
pub struct AuctionTimerTargetFill {
    pub fill_pct: f32,
    pub new_timer_ms: u32,
    pub duration_ms: u32,
    pub updated: bool,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct ShopAuctionToastState {
    pub text: String,
    pub elapsed_ms: u32,
    pub active: bool,
}

impl ShopAuctionToastState {
    pub fn show(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.elapsed_ms = 0;
        self.active = true;
    }

    fn clear(&mut self) {
        self.text.clear();
        self.elapsed_ms = 0;
        self.active = false;
    }

    fn tick(&mut self, elapsed_ms: u32) {
        if !self.active {
            return;
        }

        self.elapsed_ms = self.elapsed_ms.saturating_add(elapsed_ms);
        if self.elapsed_ms >= auction_toast_total_ms() {
            self.active = false;
            self.text.clear();
            self.elapsed_ms = 0;
        }
    }

    pub fn alpha(&self) -> f32 {
        if !self.active {
            return 0.0;
        }

        if self.elapsed_ms < AUCTION_TOAST_FADE_IN_MS {
            return self.elapsed_ms as f32 / AUCTION_TOAST_FADE_IN_MS as f32;
        }

        let fade_out_start = AUCTION_TOAST_FADE_IN_MS.saturating_add(AUCTION_TOAST_HOLD_MS);
        if self.elapsed_ms < fade_out_start {
            return 1.0;
        }

        let fade_elapsed = self.elapsed_ms.saturating_sub(fade_out_start);
        1.0 - (fade_elapsed as f32 / AUCTION_TOAST_FADE_OUT_MS as f32).min(1.0)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionToastText;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionSettlementOverlayText;

/// PROMPT 1347 / S18-AUCTION-WON-CARD-DISPOSITION-001 AC4 — marker placed on
/// the post-settle "Auction won: <card-name>" banner spawned during the
/// auction-followup PLACEMENT window. Spawned lazily by
/// [`sync_auction_won_affordance_system`] when [`AuctionWonPending`] is
/// `Pending` and the current phase is `Placement`; despawned when the won
/// card is staged via drag-drop, the PLACEMENT phase ends, or
/// [`AuctionWonPending`] otherwise transitions back to `Idle`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionWonAffordanceBanner;

/// PROMPT 1347 / S18-AUCTION-WON-CARD-DISPOSITION-001 AC4 — marker placed on
/// the child text entity inside [`AuctionWonAffordanceBanner`]. Separating
/// the marker from the parent lets the sync system update text without
/// re-querying the banner root.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionWonAffordanceText;

/// PROMPT 1347 / S18-AUCTION-WON-CARD-DISPOSITION-001 AC5 — newly-acquired
/// visual marker attached as a child of the hand-fan slot whose
/// [`crate::ui::hand::HandSlotCard`] matches the [`AuctionWonPending`] card.
/// Spawned lazily by [`sync_auction_won_hand_marker_system`] when the won
/// card is in the fan during the auction-followup PLACEMENT window;
/// despawned on stage / phase-end / `Idle` transition. One-shot per auction
/// settle: once cleared by [`AuctionWonPending::clear`], the marker does
/// NOT re-appear even if the same card is re-staged in a later PLACEMENT
/// phase (AC9).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionWonHandMarker;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialSlotIndex(pub u8);

/// Sprint 18 story-022 (`S18-UI-CARD-ART-AND-LABEL-STRIP-001`) —
/// stable reference from a `DraftInitialSlotIndex` slot to its
/// per-slot [`CardSlotArtImage`] child entity.
///
/// `handle_draft_offering_system` attaches per-card art via
/// `apply_card_display_art` against the art child (not the slot
/// root) so the chrome-preservation contract from PROMPT 1117 keeps
/// the slot's spawn-time `BackgroundColor` intact while the per-card
/// `ImageNode` swaps on the child instead. The component therefore
/// stores the [`Entity`] published when the slot is spawned.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialSlotArt(pub Entity);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialSlotCard(pub CardId);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct DraftInitialSlotCardName(pub String);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialSlotGoldCost(pub u32);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialSlotRarity(pub Rarity);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftInitialSlotState {
    Available,
    Pending,
    HandFullLocked,
    Purchased,
}

/// Marker placed on the child text entity that displays card name + cost inside a draft slot.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialSlotTextLabel;

/// Stored on the slot entity; holds the `Entity` id of the child text node.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialSlotText(pub Entity);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialBoughtOverlay {
    pub slot_index: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialReadyButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialReadyStatus;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialHandFullBanner;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialObjectiveOverlay;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialObjectiveCopy;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialObjectiveDismissButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialObjectiveRetrievalButton;

/// PROMPT 1230 — live numeric countdown rendered inside the keep-9 modal
/// panel for the DraftInitial phase only.
///
/// The HUD top-strip already paints a remaining-seconds readout for every
/// phase that publishes a non-zero `S2CPhaseChanged.timer_duration_ms`
/// (see [`crate::ui::hud::HudTimerCountdown`] /
/// `sync_hud_timer_countdown_text_system`), but the keep-9 modal covers most
/// of the viewport during DraftInitial and the player reading the modal
/// cannot easily glance at the HUD strip. This modal-local label sits on the
/// modal itself so the budget is visible without leaving the modal mentally.
///
/// Reflected onto by [`sync_draft_initial_countdown_label_system`] off the
/// canonical [`PhaseTimerState`] resource. Visibility is gated by
/// [`draft_initial_active`] so the label cannot leak into the shop / auction
/// / placement / resolution UIs that share the same `ShopAuctionUiPlugin`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialCountdownLabel;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DraftInitialObjectiveFocusTarget {
    #[default]
    None,
    DismissButton,
    RetrievalAffordance,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopSlotIndex(pub u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopSlotCard(pub CardId);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct ShopSlotCardName(pub String);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopSlotGoldCost(pub u32);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopSlotRarity(pub Rarity);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopSlotState {
    Empty,
    Available,
    PendingPurchase,
    HandFullLocked,
    Refreshing,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopRefreshButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopReadyButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopReadyStatus;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopHandFullBanner;

/// PROMPT 1042 — explicit DraftShop phase title rendered at the top of the
/// shop panel. The HUD top-bar collapses DraftInitial / DraftShop /
/// DraftAuction into the single word "DRAFT" (separate HUD scope), so the
/// shop surface needs its own in-panel title so the player can distinguish
/// the shop from Placement at a glance.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopPhaseTitle;

/// PROMPT 1042 — explicit empty-state copy shown when the DraftShop panel
/// is mounted but the server has not yet delivered offer slots (race
/// window between `S2CPhaseChanged{DraftShop}` and `S2CShopSlots`, or a
/// dropped slot broadcast). Keeps the phase legible instead of rendering
/// a blank rectangle that looks like Placement.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopEmptyState;

/// PROMPT 1085 — explicit per-slot affordance label rendered beneath each
/// shop tile. Carries the human-readable buy / locked-reason copy so the
/// player can tell at a glance whether a slot is purchasable and, if not,
/// why ("Need Ng", "Hand full", "Refreshing…", etc.). Audit AUDIT-1076-04
/// + AUDIT-1076-13: shop tiles previously painted a single Text node that
/// collapsed with the slot's `ImageNode` background so the user could not
/// see purchase intent at all. The child label lives below the well and is
/// the canonical surface for the disabled-reason feedback path.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopSlotAffordanceLabel {
    pub slot_index: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopRefreshButtonState {
    pub enabled: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopFooterSlotIndex(pub u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopFooterSlotCard(pub CardId);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopFooterSlotState {
    EmptyLocked,
    Locked,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct ShopAuctionDraftOfferingReceived {
    pub card_ids: Vec<CardId>,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionAuctionCardReceived {
    pub card_id: CardId,
    pub starting_price: u32,
    pub timer_duration_ms: u32,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionCardAcquiredReceived {
    pub card_id: CardId,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionShopCardAcquiredReceived {
    pub card_id: CardId,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct ShopAuctionShopSlotsReceived {
    pub slots: Vec<Option<CardId>>,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionBidAcceptedReceived {
    pub bidder: PlayerId,
    pub amount: u32,
    pub new_timer_ms: u32,
}

impl From<S2CAuctionBidAccepted> for ShopAuctionBidAcceptedReceived {
    fn from(message: S2CAuctionBidAccepted) -> Self {
        Self {
            bidder: message.bidder,
            amount: message.amount,
            new_timer_ms: message.new_timer_ms,
        }
    }
}

impl From<ShopAuctionBidAcceptedReceived> for S2CAuctionBidAccepted {
    fn from(message: ShopAuctionBidAcceptedReceived) -> Self {
        Self {
            bidder: message.bidder,
            amount: message.amount,
            new_timer_ms: message.new_timer_ms,
        }
    }
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionBidRejectedReceived {
    pub reason: BidRejectedReason,
}

impl From<S2CAuctionBidRejected> for ShopAuctionBidRejectedReceived {
    fn from(message: S2CAuctionBidRejected) -> Self {
        Self {
            reason: message.reason,
        }
    }
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionSettledReceived {
    pub winner: Option<PlayerId>,
    pub amount: u32,
}

impl From<S2CAuctionSettled> for ShopAuctionSettledReceived {
    fn from(message: S2CAuctionSettled) -> Self {
        Self {
            winner: message.winner,
            amount: message.amount,
        }
    }
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionDraftSlotClicked {
    pub slot: Entity,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionDraftReadyButtonClicked {
    pub button: Entity,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionDraftObjectiveDismissClicked {
    pub button: Entity,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionDraftObjectiveRetrievalClicked {
    pub button: Entity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftInitialObjectivePanelClickTarget {
    NonActionablePanel,
    Overlay,
    CardSlot(Entity),
    ReadyButton,
    Timer,
    RetrievalAffordance,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionDraftObjectivePanelClicked {
    pub target: DraftInitialObjectivePanelClickTarget,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionDraftObjectiveEscPressed;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionDraftObjectiveEnterPressed;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionShopSlotClicked {
    pub slot: Entity,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionShopRefreshClicked {
    pub button: Entity,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionShopReadyButtonClicked {
    pub button: Entity,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionAuctionPassButtonClicked {
    pub button: Entity,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionBidButtonClicked {
    pub button: Entity,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionGoldCounterFlashRequested;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BidButtonLabel {
    pub total_commitment: u32,
    pub increment: u32,
}

impl BidButtonLabel {
    pub fn text(self) -> String {
        format!("{}g\n(+{})", self.total_commitment, self.increment)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionBorderColorTier {
    PaleInkBlue,
    AuctionAmber,
    DeepAmber,
    CrimsonAmber,
}

impl AuctionBorderColorTier {
    pub fn color(self) -> Color {
        match self {
            Self::PaleInkBlue => Color::srgb_u8(0x2A, 0x4D, 0x8A),
            Self::AuctionAmber => Color::srgb_u8(0xE8, 0x7C, 0x1E),
            Self::DeepAmber => Color::srgb_u8(0xC2, 0x63, 0x0E),
            Self::CrimsonAmber => Color::srgb_u8(0x9C, 0x20, 0x00),
        }
    }
}

pub struct ShopAuctionUiPlugin;

impl Plugin for ShopAuctionUiPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("ShopAuctionUiPlugin loaded");
        app.init_resource::<CurrentClientPhase>()
            .init_resource::<ShopAuctionUiMode>()
            .init_resource::<ShopAuctionCardCatalog>()
            .init_resource::<ShopAuctionDraftHandView>()
            .init_resource::<ShopAuctionUiOutboundMessages>()
            .init_resource::<ShopAuctionDraftInitialState>()
            .init_resource::<ShopAuctionShopState>()
            .init_resource::<ShopAuctionAuctionState>()
            .init_resource::<ShopAuctionSettlementState>()
            .init_resource::<ShopAuctionShopTimerState>()
            .init_resource::<ShopAuctionLocalGoldView>()
            .init_resource::<AuctionTimerTargetFill>()
            .init_resource::<ShopAuctionToastState>()
            // PROMPT 1347 / S18-AUCTION-WON-CARD-DISPOSITION-001 — presentation-
            // layer state for AC4 banner + AC5 hand-fan marker + AC11 snapshot
            // block. Not authoritative; derived from the existing
            // `S2CAuctionSettled` + `S2CCardAcquired { source: AuctionWon }`
            // wire-level pair (which is unchanged by this row).
            .init_resource::<AuctionWonPending>()
            .init_resource::<ShopAuctionRefreshConfig>()
            .init_resource::<AuctionBidKeyboardFocus>()
            .init_resource::<AuctionLocallyPassed>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<PlayerEconomyView>()
            .init_resource::<ClientPhaseView>()
            // PROMPT 1230 — DraftInitial modal-local countdown label reflects
            // the canonical phase timer owned by HudPlugin. Init here too so
            // the keep-9 countdown system works in shop-auction-only tests
            // that don't register HudPlugin; the production app loads both
            // plugins and `init_resource` is idempotent.
            .init_resource::<PhaseTimerState>()
            .add_message::<PresentationGameSnapshotMessage>()
            .add_message::<HudGoldBroadcastMessage>()
            .add_message::<ShopAuctionDraftOfferingReceived>()
            .add_message::<ShopAuctionAuctionCardReceived>()
            .add_message::<ShopAuctionCardAcquiredReceived>()
            .add_message::<ShopAuctionShopCardAcquiredReceived>()
            .add_message::<ShopAuctionShopSlotsReceived>()
            .add_message::<ShopAuctionBidAcceptedReceived>()
            .add_message::<ShopAuctionBidRejectedReceived>()
            .add_message::<ShopAuctionSettledReceived>()
            .add_message::<ShopAuctionDraftSlotClicked>()
            .add_message::<ShopAuctionDraftReadyButtonClicked>()
            .add_message::<ShopAuctionDraftObjectiveDismissClicked>()
            .add_message::<ShopAuctionDraftObjectiveRetrievalClicked>()
            .add_message::<ShopAuctionDraftObjectivePanelClicked>()
            .add_message::<ShopAuctionDraftObjectiveEscPressed>()
            .add_message::<ShopAuctionDraftObjectiveEnterPressed>()
            .add_message::<ShopAuctionShopSlotClicked>()
            .add_message::<ShopAuctionShopRefreshClicked>()
            .add_message::<ShopAuctionShopReadyButtonClicked>()
            .add_message::<ShopAuctionBidButtonClicked>()
            .add_message::<ShopAuctionAuctionPassButtonClicked>()
            .add_message::<ShopAuctionGoldCounterFlashRequested>()
            .configure_sets(
                Update,
                (
                    ShopAuctionUiSystemSet::PhaseTransition,
                    ShopAuctionUiSystemSet::MessageDrain,
                    ShopAuctionUiSystemSet::Input,
                    ShopAuctionUiSystemSet::StateSync,
                )
                    .chain()
                    .run_if(in_state(ClientState::InSession)),
            )
            .add_systems(
                OnEnter(ClientState::InSession),
                // Sprint 18 story 020 (S18-UI-PLAY-AREA-CONTAINER-001):
                // chain after `PlayAreaSpawnSet` so the `PlayAreaRoot`
                // resource (when `PlayAreaPlugin` is registered) is
                // available before the four migrated panels parent into
                // it. Harness apps without `PlayAreaPlugin` fall back to
                // the historical `ShopAuctionUiRoot` parent inside
                // `spawn_shop_auction_ui`.
                spawn_shop_auction_ui.after(crate::ui::PlayAreaSpawnSet),
            )
            .add_systems(OnExit(ClientState::InSession), despawn_shop_auction_ui)
            .add_systems(
                Update,
                (
                    shop_auction_ui_phase_transition_system
                        .in_set(ShopAuctionUiSystemSet::PhaseTransition),
                    (
                        drain_auction_card_receiver_system,
                        drain_auction_bid_accepted_receiver_system,
                        drain_auction_bid_rejected_receiver_system,
                        drain_auction_settled_receiver_system,
                        handle_auction_snapshot_system,
                        handle_auction_gold_broadcast_system,
                        handle_auction_bid_accepted_system,
                        handle_auction_bid_rejected_system,
                        handle_auction_settled_system,
                        handle_draft_offering_system,
                        handle_auction_card_system,
                        handle_card_acquired_system,
                        handle_shop_card_acquired_system,
                        apply_draft_initial_purchase_confirmations_system,
                        apply_shop_purchase_confirmations_system,
                        handle_shop_slots_system,
                    )
                        .chain()
                        .in_set(ShopAuctionUiSystemSet::MessageDrain),
                    (
                        handle_shop_auction_control_interactions_system,
                        handle_draft_initial_objective_message_input_system,
                        handle_draft_initial_objective_keyboard_system,
                        handle_draft_initial_objective_button_interactions_system,
                        handle_draft_initial_slot_click_system,
                        handle_draft_initial_ready_click_system,
                        handle_shop_slot_click_system,
                        handle_shop_refresh_click_system,
                        handle_shop_ready_click_system,
                        handle_auction_bid_button_interactions_system,
                        handle_auction_bid_keyboard_focus_system,
                        handle_auction_bid_button_click_system,
                        handle_auction_pass_button_interactions_system,
                        handle_auction_pass_button_click_system,
                    )
                        .chain()
                        .in_set(ShopAuctionUiSystemSet::Input),
                    (
                        tick_auction_preparing_timeout_system,
                        tick_auction_countdown_system,
                        tick_auction_settlement_transition_system,
                        tick_auction_toast_system,
                        sync_draft_initial_panel_system,
                        sync_draft_initial_countdown_label_system,
                        sync_shop_panel_system,
                        sync_auction_panel_system,
                        sync_settlement_overlay_system,
                        sync_auction_toast_system,
                        // PROMPT 1347 / AC4 + AC5 + AC9 — disposition state
                        // update must run before the affordance + marker
                        // sync systems so staged_yet / submitted_yet / phase-
                        // exit clearing reflects the current frame.
                        update_auction_won_pending_system,
                        sync_auction_won_affordance_system,
                        sync_auction_won_hand_marker_system,
                    )
                        .chain()
                        .in_set(ShopAuctionUiSystemSet::StateSync),
                ),
            );
    }
}

pub fn local_free_gold(gold: u32, reserved_gold: u32) -> u32 {
    gold.saturating_sub(reserved_gold)
}

pub fn bid_button_labels(current_price: u32) -> [BidButtonLabel; 3] {
    shared::config::GameConfig::default()
        .bid_increments
        .map(|increment| BidButtonLabel {
            total_commitment: current_price.saturating_add(increment),
            increment,
        })
}

pub fn bid_button_label_texts(current_price: u32) -> [String; 3] {
    bid_button_labels(current_price).map(BidButtonLabel::text)
}

pub fn displayed_refresh_cost(
    refresh_base_cost: u32,
    refresh_cap: u32,
    refresh_count_this_draft: u32,
) -> u32 {
    refresh_base_cost.saturating_add(refresh_count_this_draft.min(refresh_cap))
}

pub fn auction_border_color_tier(current_price: u32) -> AuctionBorderColorTier {
    match current_price {
        0..=3 => AuctionBorderColorTier::PaleInkBlue,
        4..=6 => AuctionBorderColorTier::AuctionAmber,
        7..=9 => AuctionBorderColorTier::DeepAmber,
        _ => AuctionBorderColorTier::CrimsonAmber,
    }
}

pub fn sort_draft_offering_card_ids(card_ids: &[CardId], catalog: &CardCatalog) -> Vec<CardId> {
    let mut indexed_cards = card_ids
        .iter()
        .copied()
        .take(SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT)
        .enumerate()
        .collect::<Vec<_>>();

    indexed_cards.sort_by(|(left_index, left_id), (right_index, right_id)| {
        let left = catalog.get(left_id);
        let right = catalog.get(right_id);
        let left_rank = left.map_or(0, |card| rarity_sort_rank(card.rarity));
        let right_rank = right.map_or(0, |card| rarity_sort_rank(card.rarity));
        let left_cost = left.map_or(0, |card| card.cost);
        let right_cost = right.map_or(0, |card| card.cost);

        right_rank
            .cmp(&left_rank)
            .then_with(|| right_cost.cmp(&left_cost))
            .then_with(|| left_index.cmp(right_index))
    });

    indexed_cards
        .into_iter()
        .map(|(_index, card_id)| card_id)
        .collect()
}

pub fn shop_auction_ui_phase_transition_system(
    current: Res<CurrentClientPhase>,
    phase_view: Res<ClientPhaseView>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    mut mode: ResMut<ShopAuctionUiMode>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
    mut shop_state: ResMut<ShopAuctionShopState>,
    mut auction_state: ResMut<ShopAuctionAuctionState>,
    mut settlement_state: ResMut<ShopAuctionSettlementState>,
    mut shop_timer: ResMut<ShopAuctionShopTimerState>,
    mut toast_state: ResMut<ShopAuctionToastState>,
    mut timer_target: ResMut<AuctionTimerTargetFill>,
    mut keyboard_focus: ResMut<AuctionBidKeyboardFocus>,
    mut locally_passed: ResMut<AuctionLocallyPassed>,
    mut visibility: Query<&mut Visibility>,
) {
    if !current.is_changed() {
        return;
    }

    let previous_mode = *mode;
    let mut next_mode = ShopAuctionUiMode::from_phase(current.phase);
    let settlement_active = settlement_state.transition_active;

    clear_auction_feedback_state(
        &mut toast_state,
        &mut timer_target,
        &mut keyboard_focus,
        &mut locally_passed,
    );

    if settlement_active {
        match current.phase {
            RoundPhase::DraftShop => {
                shop_state.enter_shop_phase();
                shop_timer.defer(phase_view.timer_duration_ms);
                next_mode = ShopAuctionUiMode::AuctionSettling;
            }
            RoundPhase::DraftAuction => {
                next_mode = ShopAuctionUiMode::AuctionSettling;
            }
            RoundPhase::Placement
            | RoundPhase::Resolution
            | RoundPhase::GameOver
            | RoundPhase::DraftInitial
            | RoundPhase::Lobby
            | RoundPhase::Handshaking => {
                settlement_state.clear();
                auction_state.clear();
                shop_timer.stop();
            }
        }
    } else if current.phase == RoundPhase::DraftAuction {
        shop_state.enter_auction_phase();
        shop_timer.stop();
        if auction_state.card_id.is_some() {
            let timer_duration_ms = auction_state.timer_duration_ms;
            auction_state.enter_active(timer_duration_ms);
            next_mode = ShopAuctionUiMode::Auction;
        } else {
            auction_state.panel_state = ShopAuctionAuctionPanelState::Hidden;
        }
    } else if current.phase == RoundPhase::DraftShop
        && auction_state.card_id.is_some()
        && matches!(
            auction_state.panel_state,
            ShopAuctionAuctionPanelState::Preparing
                | ShopAuctionAuctionPanelState::Active
                | ShopAuctionAuctionPanelState::ConnectionError
        )
    {
        auction_state.clear_bid_resolution_state();
        shop_state.enter_shop_phase();
        shop_timer.defer(phase_view.timer_duration_ms);
        next_mode = ShopAuctionUiMode::AuctionSettling;
    } else if matches!(
        auction_state.panel_state,
        ShopAuctionAuctionPanelState::Preparing
            | ShopAuctionAuctionPanelState::Active
            | ShopAuctionAuctionPanelState::Settling
            | ShopAuctionAuctionPanelState::ConnectionError
    ) {
        auction_state.clear();
    }

    *mode = next_mode;

    if next_mode != ShopAuctionUiMode::DraftOffering {
        draft_state.reset_phase_state();
    } else if previous_mode != ShopAuctionUiMode::DraftOffering && draft_state.offering_loaded {
        draft_state.show_objective_overlay();
    }

    match next_mode {
        ShopAuctionUiMode::Shop => {
            shop_state.enter_shop_phase();
            shop_timer.start(phase_view.timer_duration_ms);
        }
        ShopAuctionUiMode::DraftOffering => {
            shop_state.clear_all();
            shop_timer.stop();
        }
        ShopAuctionUiMode::Inactive if current.phase != RoundPhase::DraftAuction => {
            shop_state.clear_all();
            shop_timer.stop();
        }
        ShopAuctionUiMode::Inactive => {}
        ShopAuctionUiMode::AuctionPreparing
        | ShopAuctionUiMode::Auction
        | ShopAuctionUiMode::AuctionSettling => {}
    }

    let Some(entities) = entities else {
        return;
    };

    let root_visible = match next_mode {
        ShopAuctionUiMode::Inactive => false,
        ShopAuctionUiMode::DraftOffering => draft_state.offering_loaded,
        ShopAuctionUiMode::AuctionPreparing | ShopAuctionUiMode::Auction => {
            auction_state.panel_visible()
        }
        ShopAuctionUiMode::AuctionSettling => auction_state.panel_visible(),
        // PROMPT 1042 — root visible immediately on entering DraftShop so
        // the explicit shop surface (phase title + empty state + chrome)
        // renders even before `S2CShopSlots` arrives.
        ShopAuctionUiMode::Shop => true,
    };

    set_visibility(&mut visibility, entities.root, visibility_for(root_visible));
    set_visibility(
        &mut visibility,
        entities.draft_offering_panel,
        visibility_for(
            next_mode == ShopAuctionUiMode::DraftOffering && draft_state.offering_loaded,
        ),
    );
    // PROMPT 1042 — shop_panel visibility no longer gated on slots_loaded;
    // empty-state copy + phase title carry the surface during the race
    // window between phase change and `S2CShopSlots` delivery.
    set_visibility(
        &mut visibility,
        entities.shop_panel,
        visibility_for(next_mode == ShopAuctionUiMode::Shop),
    );
    set_visibility(
        &mut visibility,
        entities.auction_panel,
        visibility_for(
            matches!(
                next_mode,
                ShopAuctionUiMode::AuctionPreparing
                    | ShopAuctionUiMode::Auction
                    | ShopAuctionUiMode::AuctionSettling
            ) && auction_state.panel_visible(),
        ),
    );
    set_visibility(
        &mut visibility,
        entities.shop_footer,
        visibility_for(next_mode == ShopAuctionUiMode::Auction),
    );
    set_visibility(&mut visibility, entities.toast_root, Visibility::Hidden);
    set_visibility(&mut visibility, entities.toast_text, Visibility::Hidden);
    set_visibility(
        &mut visibility,
        entities.settlement_overlay,
        Visibility::Hidden,
    );
    set_visibility(
        &mut visibility,
        entities.settlement_overlay_text,
        Visibility::Hidden,
    );
}

fn clear_auction_feedback_state(
    toast_state: &mut ShopAuctionToastState,
    timer_target: &mut AuctionTimerTargetFill,
    keyboard_focus: &mut AuctionBidKeyboardFocus,
    locally_passed: &mut AuctionLocallyPassed,
) {
    toast_state.clear();
    *timer_target = AuctionTimerTargetFill::default();
    keyboard_focus.focused_button = None;
    // PROMPT 1042 — clear the local Pass intent on any phase change so a
    // pass in round N never leaks into round N+1's auction.
    locally_passed.passed = false;
}

pub fn drain_auction_card_receiver_system(
    mut receivers: Query<&mut MessageReceiver<S2CAuctionCard>>,
    mut writer: MessageWriter<ShopAuctionAuctionCardReceived>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::info!(
                card_id = ?message.card_id,
                starting_price = message.starting_price,
                timer_duration_ms = message.timer_duration_ms,
                msg_type = "S2CAuctionCard",
                "drain_auction_card: recv"
            );
            writer.write(ShopAuctionAuctionCardReceived {
                card_id: message.card_id,
                starting_price: message.starting_price,
                timer_duration_ms: message.timer_duration_ms,
            });
        }
    }
}

pub fn drain_auction_bid_accepted_receiver_system(
    mut receivers: Query<&mut MessageReceiver<S2CAuctionBidAccepted>>,
    mut writer: MessageWriter<ShopAuctionBidAcceptedReceived>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::info!(
                bidder = ?message.bidder,
                amount = message.amount,
                new_timer_ms = message.new_timer_ms,
                msg_type = "S2CAuctionBidAccepted",
                "drain_auction_bid_accepted: recv"
            );
            writer.write(message.into());
        }
    }
}

pub fn drain_auction_bid_rejected_receiver_system(
    mut receivers: Query<&mut MessageReceiver<S2CAuctionBidRejected>>,
    mut writer: MessageWriter<ShopAuctionBidRejectedReceived>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::info!(
                reason = ?message.reason,
                msg_type = "S2CAuctionBidRejected",
                "drain_auction_bid_rejected: recv"
            );
            writer.write(message.into());
        }
    }
}

pub fn drain_auction_settled_receiver_system(
    mut receivers: Query<&mut MessageReceiver<S2CAuctionSettled>>,
    mut writer: MessageWriter<ShopAuctionSettledReceived>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::info!(
                winner = ?message.winner,
                amount = message.amount,
                msg_type = "S2CAuctionSettled",
                "drain_auction_settled: recv"
            );
            writer.write(message.into());
        }
    }
}

pub fn handle_auction_snapshot_system(
    phase_view: Res<ClientPhaseView>,
    mut snapshots: MessageReader<PresentationGameSnapshotMessage>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
    mut shop_state: ResMut<ShopAuctionShopState>,
    mut auction_state: ResMut<ShopAuctionAuctionState>,
    mut settlement_state: ResMut<ShopAuctionSettlementState>,
    mut shop_timer: ResMut<ShopAuctionShopTimerState>,
    mut toast_state: ResMut<ShopAuctionToastState>,
    mut timer_target: ResMut<AuctionTimerTargetFill>,
    mut keyboard_focus: ResMut<AuctionBidKeyboardFocus>,
    mut locally_passed: ResMut<AuctionLocallyPassed>,
    mut local_gold: ResMut<ShopAuctionLocalGoldView>,
    mut hand_view: ResMut<ShopAuctionDraftHandView>,
    mut mode: ResMut<ShopAuctionUiMode>,
    mut shop_slots_writer: MessageWriter<ShopAuctionShopSlotsReceived>,
) {
    for snapshot in snapshots.read().map(|message| &message.0) {
        draft_state.reset_phase_state();
        shop_state.clear_all();
        auction_state.clear();
        settlement_state.clear();
        shop_timer.stop();
        clear_auction_feedback_state(
            &mut toast_state,
            &mut timer_target,
            &mut keyboard_focus,
            &mut locally_passed,
        );

        local_gold.player_id = Some(snapshot.recipient_player_id);
        let mut local_shop_slots = None;
        if let Some(local_player) = snapshot
            .players
            .iter()
            .find(|player| player.player_id == snapshot.recipient_player_id)
        {
            local_gold.gold = local_player.gold;
            local_gold.reserved_gold = local_player.reserved_gold;
            local_gold.initialized = true;
            hand_view.hand_size = local_player.hand.len();
            if snapshot.phase == RoundPhase::DraftShop {
                local_shop_slots = Some(local_player.shop_slots.clone());
            }
        } else {
            local_gold.gold = 0;
            local_gold.reserved_gold = 0;
            local_gold.initialized = false;
            hand_view.hand_size = 0;
            warn!(
                "Shop/Auction UI: snapshot for {:?} does not contain local player",
                snapshot.recipient_player_id
            );
        }

        match snapshot.phase {
            RoundPhase::DraftAuction => {
                if let Some(snapshot_auction) = snapshot.auction_state {
                    auction_state
                        .restore_from_snapshot(&snapshot_auction, phase_view.timer_duration_ms);
                    *mode = ShopAuctionUiMode::Auction;
                } else {
                    *mode = ShopAuctionUiMode::Inactive;
                }
            }
            RoundPhase::DraftShop => {
                shop_state.enter_shop_phase();
                shop_timer.restore_from_snapshot(
                    phase_view.timer_duration_ms,
                    snapshot.timer_remaining_ms,
                );
                *mode = ShopAuctionUiMode::Shop;
                if let Some(slots) = local_shop_slots {
                    shop_slots_writer.write(ShopAuctionShopSlotsReceived { slots });
                }
            }
            RoundPhase::DraftInitial => {
                *mode = ShopAuctionUiMode::DraftOffering;
            }
            RoundPhase::Lobby
            | RoundPhase::Placement
            | RoundPhase::Resolution
            | RoundPhase::GameOver
            | RoundPhase::Handshaking => {
                *mode = ShopAuctionUiMode::Inactive;
            }
        }
    }
}

pub fn handle_auction_gold_broadcast_system(
    player_ids: Option<Res<HudPlayerIds>>,
    mut messages: MessageReader<HudGoldBroadcastMessage>,
    mut local_gold: ResMut<ShopAuctionLocalGoldView>,
    mut auction_state: ResMut<ShopAuctionAuctionState>,
) {
    let local_player_id = auction_local_player_id(&local_gold, player_ids.as_deref());

    if let Some(player_id) = local_player_id {
        local_gold.player_id = Some(player_id);
    }

    for message in messages.read().map(|message| &message.0) {
        if Some(message.player_id) != local_player_id {
            continue;
        }

        local_gold.gold = message.gold;
        local_gold.reserved_gold = message.reserved_gold;
        local_gold.initialized = true;
        auction_state.record_local_gold_broadcast();
    }
}

pub fn handle_auction_bid_accepted_system(
    player_ids: Option<Res<HudPlayerIds>>,
    local_gold: Res<ShopAuctionLocalGoldView>,
    current: Res<CurrentClientPhase>,
    mode: Res<ShopAuctionUiMode>,
    mut messages: MessageReader<ShopAuctionBidAcceptedReceived>,
    mut auction_state: ResMut<ShopAuctionAuctionState>,
    mut timer_target: ResMut<AuctionTimerTargetFill>,
) {
    let local_player_id = auction_local_player_id(&local_gold, player_ids.as_deref());

    for message in messages.read() {
        if current.phase != RoundPhase::DraftAuction
            || *mode != ShopAuctionUiMode::Auction
            || auction_state.panel_state != ShopAuctionAuctionPanelState::Active
        {
            continue;
        }

        let accepted = S2CAuctionBidAccepted {
            bidder: message.bidder,
            amount: message.amount,
            new_timer_ms: message.new_timer_ms,
        };
        auction_state.apply_bid_accepted(&accepted, local_player_id);
        let duration_ms = auction_state.timer_duration_ms;
        let fill_pct = if duration_ms == 0 {
            0.0
        } else {
            accepted.new_timer_ms.min(duration_ms) as f32 / duration_ms as f32
        };
        *timer_target = AuctionTimerTargetFill {
            fill_pct,
            new_timer_ms: accepted.new_timer_ms,
            duration_ms,
            updated: true,
        };
    }
}

pub fn handle_auction_bid_rejected_system(
    current: Res<CurrentClientPhase>,
    mode: Res<ShopAuctionUiMode>,
    mut messages: MessageReader<ShopAuctionBidRejectedReceived>,
    mut auction_state: ResMut<ShopAuctionAuctionState>,
    mut toast_state: ResMut<ShopAuctionToastState>,
) {
    for message in messages.read() {
        if current.phase != RoundPhase::DraftAuction
            || *mode != ShopAuctionUiMode::Auction
            || auction_state.panel_state != ShopAuctionAuctionPanelState::Active
        {
            continue;
        }

        let toast = rejection_toast_text(message.reason, auction_state.current_price);
        auction_state.clear_bid_resolution_state();
        toast_state.show(toast);
    }
}

/// PROMPT 1347 — SystemParam bundle for the three optional Messages
/// resources written by [`handle_auction_settled_system`]. Bevy 0.18
/// caps system arity at 16 SystemParams; adding `AuctionWonPending` would
/// push the function to 17. Bundling the optional Messages resources
/// preserves the existing 16-param ceiling without changing what is
/// written.
#[derive(SystemParam)]
pub struct AuctionSettledAnimMessages<'w> {
    pub card_acquired_anim: Option<ResMut<'w, Messages<CardAcquiredAnimReady>>>,
    pub settlement_overlay: Option<ResMut<'w, Messages<SettlementOverlayRequested>>>,
    pub panel_transition: Option<ResMut<'w, Messages<AuctionPanelTransitionRequested>>>,
}

#[allow(clippy::too_many_arguments)]
pub fn handle_auction_settled_system(
    player_ids: Option<Res<HudPlayerIds>>,
    local_gold: Res<ShopAuctionLocalGoldView>,
    current: Res<CurrentClientPhase>,
    phase_view: Res<ClientPhaseView>,
    mut messages: MessageReader<ShopAuctionSettledReceived>,
    mut auction_state: ResMut<ShopAuctionAuctionState>,
    mut settlement_state: ResMut<ShopAuctionSettlementState>,
    mut shop_state: ResMut<ShopAuctionShopState>,
    mut shop_timer: ResMut<ShopAuctionShopTimerState>,
    mut hand_view: ResMut<ShopAuctionDraftHandView>,
    mut mode: ResMut<ShopAuctionUiMode>,
    mut toast_state: ResMut<ShopAuctionToastState>,
    mut timer_target: ResMut<AuctionTimerTargetFill>,
    mut auction_won_pending: ResMut<AuctionWonPending>,
    mut anim_messages: AuctionSettledAnimMessages,
) {
    let local_player_id = auction_local_player_id(&local_gold, player_ids.as_deref());

    for message in messages.read() {
        if !auction_settlement_can_start(current.phase, &auction_state) {
            continue;
        }

        let outcome = match message.winner {
            Some(winner) if Some(winner) == local_player_id => {
                ShopAuctionSettlementOutcome::LocalWinner
            }
            Some(_) => ShopAuctionSettlementOutcome::OpponentWinner,
            None => ShopAuctionSettlementOutcome::NoBid,
        };
        let local_card_feedback = outcome == ShopAuctionSettlementOutcome::LocalWinner;

        auction_state.enter_settling(message.amount);
        settlement_state.begin(
            outcome,
            message.winner,
            message.amount,
            auction_state.card_id,
            local_card_feedback,
        );
        toast_state.clear();
        *timer_target = AuctionTimerTargetFill::default();

        if local_card_feedback {
            hand_view.hand_size = hand_view.hand_size.saturating_add(1).min(10);
            if let Some(messages) = anim_messages.card_acquired_anim.as_deref_mut() {
                messages.write(CardAcquiredAnimReady);
            }
            // PROMPT 1347 / AC4 / AC5 / AC11 — arm the auction-won
            // disposition state so the affordance banner, hand-fan marker,
            // and QA snapshot block become active when the auction-followup
            // PLACEMENT phase begins. `auction_state.card_id` is the won
            // card (the auction state still holds the card at this point;
            // it is cleared shortly after by `enter_settling`). One-shot
            // per auction settle.
            if let Some(card_id) = auction_state.card_id {
                auction_won_pending.arm(card_id, phase_view.round_number);
                tracing::info!(
                    target: "client::ui::shop_auction",
                    card_id = ?card_id,
                    settle_round = phase_view.round_number,
                    "auction_won_pending: armed for AC4/AC5/AC11"
                );
            }
        }
        if let Some(messages) = anim_messages.settlement_overlay.as_deref_mut() {
            messages.write(SettlementOverlayRequested);
        }
        if let Some(messages) = anim_messages.panel_transition.as_deref_mut() {
            messages.write(AuctionPanelTransitionRequested);
        }

        if current.phase == RoundPhase::DraftShop {
            shop_state.enter_shop_phase();
            shop_timer.defer(phase_view.timer_duration_ms);
        }
        *mode = ShopAuctionUiMode::AuctionSettling;
    }
}

fn auction_settlement_can_start(
    current_phase: RoundPhase,
    auction_state: &ShopAuctionAuctionState,
) -> bool {
    matches!(
        current_phase,
        RoundPhase::DraftAuction | RoundPhase::DraftShop
    ) && auction_state.card_id.is_some()
        && matches!(
            auction_state.panel_state,
            ShopAuctionAuctionPanelState::Preparing
                | ShopAuctionAuctionPanelState::Active
                | ShopAuctionAuctionPanelState::ConnectionError
        )
}

pub fn handle_auction_card_system(
    current: Res<CurrentClientPhase>,
    mut auction_cards: MessageReader<ShopAuctionAuctionCardReceived>,
    mut auction_state: ResMut<ShopAuctionAuctionState>,
    mut mode: ResMut<ShopAuctionUiMode>,
    mut locally_passed: ResMut<AuctionLocallyPassed>,
) {
    for message in auction_cards.read() {
        // PROMPT 1042 — clear local Pass intent whenever a new auction
        // card is buffered. Pass is per-card, not per-phase.
        locally_passed.passed = false;
        // Placement is gameplay-active and never transitions directly into
        // DraftAuction; auction cards arriving mid-placement are out-of-band.
        if current.phase == RoundPhase::Placement {
            info!(
                target: "shop_auction",
                phase = ?current.phase,
                card_id = ?message.card_id,
                starting_price = message.starting_price,
                "handle_auction_card: dropped (Placement phase, no deferred activation path)"
            );
            continue;
        }

        if auction_state.panel_state == ShopAuctionAuctionPanelState::Settling {
            info!(
                target: "shop_auction",
                phase = ?current.phase,
                card_id = ?message.card_id,
                starting_price = message.starting_price,
                "handle_auction_card: dropped (panel Settling)"
            );
            continue;
        }

        // S11-SAU-AUCTION-CARD-DROP-ON-PHASE-LAG-001: when a DraftAuction
        // S2CPhaseChanged is held in PendingPhaseChange (Stage A buffering
        // during BoardRenderState::ResolutionExecuting), the matching
        // S2CAuctionCard arrives while CurrentClientPhase is still
        // DraftShop/Resolution/GameOver. Previously the card was silently
        // dropped via `continue`, leaving the auction panel hidden when the
        // phase change eventually drained into DraftAuction. Buffer the card
        // and stay in Preparing so the phase transition system promotes the
        // panel to Active when the deferred phase change applies.
        if matches!(
            current.phase,
            RoundPhase::DraftShop | RoundPhase::Resolution | RoundPhase::GameOver
        ) {
            if auction_state.card_id.is_some() {
                info!(
                    target: "shop_auction",
                    phase = ?current.phase,
                    card_id = ?message.card_id,
                    starting_price = message.starting_price,
                    "handle_auction_card: dropped (card already buffered)"
                );
                continue;
            }
            info!(
                target: "shop_auction",
                phase = ?current.phase,
                card_id = ?message.card_id,
                starting_price = message.starting_price,
                "handle_auction_card: buffering during transitional phase (defer activation)"
            );
            auction_state.buffer_card(&S2CAuctionCard {
                card_id: message.card_id,
                starting_price: message.starting_price,
                timer_duration_ms: message.timer_duration_ms,
            });
            auction_state.enter_preparing();
            *mode = ShopAuctionUiMode::AuctionPreparing;
            continue;
        }

        auction_state.buffer_card(&S2CAuctionCard {
            card_id: message.card_id,
            starting_price: message.starting_price,
            timer_duration_ms: message.timer_duration_ms,
        });

        if current.phase == RoundPhase::DraftAuction {
            let timer_duration_ms = auction_state.timer_duration_ms;
            auction_state.enter_active(timer_duration_ms);
            *mode = ShopAuctionUiMode::Auction;
        } else {
            auction_state.enter_preparing();
            *mode = ShopAuctionUiMode::AuctionPreparing;
        }
    }
}

pub fn handle_draft_offering_system(
    mode: Res<ShopAuctionUiMode>,
    catalog: Res<ShopAuctionCardCatalog>,
    asset_server: Option<Res<AssetServer>>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    mut offerings: MessageReader<ShopAuctionDraftOfferingReceived>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
    mut commands: Commands,
    mut draft_ui: ParamSet<(
        // p0: slot container — index + text-child link + art-child link + visibility
        Query<(
            &DraftInitialSlotIndex,
            &DraftInitialSlotText,
            &DraftInitialSlotArt,
            &mut Visibility,
        )>,
        // p1: bought overlays
        Query<(&DraftInitialBoughtOverlay, &mut Visibility)>,
    )>,
    // Separate query so p0 and the text mutation don't alias
    mut text_query: Query<&mut Text, With<DraftInitialSlotTextLabel>>,
) {
    let Some(entities) = entities else {
        for _offering in offerings.read() {}
        return;
    };

    for offering in offerings.read() {
        draft_state.offering_loaded = true;
        draft_state.ready_signalled = false;
        if *mode == ShopAuctionUiMode::DraftOffering {
            draft_state.show_objective_overlay();
        }
        draft_state.pending_confirmed_purchases.clear();

        let sorted_card_ids = sort_draft_offering_card_ids(&offering.card_ids, &catalog.cards);

        {
            let mut slots = draft_ui.p0();
            for slot_entity in entities.draft_initial_slots {
                let Ok((slot_index, slot_text, slot_art, mut visibility)) =
                    slots.get_mut(slot_entity)
                else {
                    continue;
                };
                let text_entity = slot_text.0;
                let art_entity = slot_art.0;

                let Some(card_id) = sorted_card_ids.get(slot_index.0 as usize).copied() else {
                    // Clear: hide slot, wipe text child, drop the art handle on
                    // the `CardSlotArtImage` child (story-022 AC5 / AC7), and
                    // remove state components.
                    clear_card_display_art(&mut commands, art_entity);
                    clear_draft_initial_slot(
                        &mut commands,
                        slot_entity,
                        text_entity,
                        &mut text_query,
                        &mut visibility,
                    );
                    continue;
                };

                let card = catalog.cards.get(&card_id);
                let card_name = card
                    .map(|card| card.name_en.clone())
                    .unwrap_or_else(|| format!("Card {}", card_id.0));
                let cost = card.map_or(0, |card| card.cost);
                let rarity = card.map_or(Rarity::Common, |card| card.rarity);
                // PROMPT 1029: append ATK/HP for minion-shaped cards so players
                // can read combat stats from the keep-9 grid. Empty string for
                // spells / traps keeps existing layout unchanged.
                let stats = card.map(format_card_combat_stats).unwrap_or_default();

                if let Ok(mut text) = text_query.get_mut(text_entity) {
                    text.0.clear();
                    if stats.is_empty() {
                        text.0
                            .push_str(&format!("{}\n{}g", card_name.as_str(), cost));
                    } else {
                        text.0
                            .push_str(&format!("{}\n{}g · {}", card_name.as_str(), cost, stats));
                    }
                }
                commands.entity(slot_entity).insert((
                    DraftInitialSlotCard(card_id),
                    DraftInitialSlotCardName(card_name),
                    DraftInitialSlotGoldCost(cost),
                    DraftInitialSlotRarity(rarity),
                    DraftInitialSlotState::Available,
                ));
                // Sprint 18 story-022 AC5: per-card art now binds onto the
                // `CardSlotArtImage` child instead of the slot root.
                apply_card_display_art(&mut commands, art_entity, card, asset_server.as_deref());
                *visibility = visibility_for(draft_initial_active(&mode, &draft_state));
            }
        }

        let mut overlays = draft_ui.p1();
        for (overlay, mut visibility) in &mut overlays {
            let is_known_overlay =
                (overlay.slot_index as usize) < SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT;
            if is_known_overlay {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

pub fn handle_shop_slots_system(
    current: Res<CurrentClientPhase>,
    mode: Res<ShopAuctionUiMode>,
    catalog: Res<ShopAuctionCardCatalog>,
    asset_server: Option<Res<AssetServer>>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    mut slot_messages: MessageReader<ShopAuctionShopSlotsReceived>,
    mut shop_state: ResMut<ShopAuctionShopState>,
    mut commands: Commands,
    mut shop_slots: Query<(Entity, &ShopSlotIndex, &mut Text, &mut Visibility)>,
) {
    let mut slots_to_apply = None;
    let can_apply_shop_slots = current.phase == RoundPhase::DraftShop
        && matches!(
            *mode,
            ShopAuctionUiMode::Shop | ShopAuctionUiMode::AuctionSettling
        );
    for message in slot_messages.read() {
        if can_apply_shop_slots {
            slots_to_apply = Some(message.slots.clone());
        } else if should_buffer_shop_slots(current.phase) {
            shop_state.queue_slots(message.slots.clone());
        } else {
            shop_state.refresh_in_flight = false;
        }
    }

    if can_apply_shop_slots && slots_to_apply.is_none() {
        slots_to_apply = shop_state.take_buffered_slots();
    }

    let Some(slots) = slots_to_apply else {
        return;
    };

    if !can_apply_shop_slots {
        shop_state.queue_slots(slots);
        return;
    }

    if shop_state.refresh_in_flight {
        shop_state.refresh_count_this_draft = shop_state.refresh_count_this_draft.saturating_add(1);
    }
    shop_state.refresh_in_flight = false;
    shop_state.slots_loaded = true;
    shop_state.pending_confirmed_purchases.clear();

    let Some(entities) = entities else {
        return;
    };

    for slot_entity in entities.shop_slots {
        let Ok((entity, slot_index, mut text, mut visibility)) = shop_slots.get_mut(slot_entity)
        else {
            continue;
        };
        let card_id = slots.get(slot_index.0 as usize).copied().flatten();
        apply_shop_slot(
            &mut commands,
            entity,
            card_id,
            &catalog.cards,
            asset_server.as_deref(),
            &mut text,
            &mut visibility,
        );
    }
}

pub fn handle_card_acquired_system(
    mut acquisitions: MessageReader<ShopAuctionCardAcquiredReceived>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
) {
    for acquisition in acquisitions.read() {
        draft_state.queue_purchase_confirmation(acquisition.card_id);
    }
}

pub fn handle_shop_card_acquired_system(
    current: Res<CurrentClientPhase>,
    mut acquisitions: MessageReader<ShopAuctionShopCardAcquiredReceived>,
    mut shop_state: ResMut<ShopAuctionShopState>,
) {
    if current.phase != RoundPhase::DraftShop {
        for _acquisition in acquisitions.read() {}
        shop_state.pending_confirmed_purchases.clear();
        return;
    }

    for acquisition in acquisitions.read() {
        shop_state.queue_purchase_confirmation(acquisition.card_id);
    }
}

pub fn apply_draft_initial_purchase_confirmations_system(
    mode: Res<ShopAuctionUiMode>,
    economy: Res<PlayerEconomyView>,
    mut hand_view: ResMut<ShopAuctionDraftHandView>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
    mut commands: Commands,
    mut slots: Query<(
        Entity,
        &DraftInitialSlotIndex,
        &DraftInitialSlotCard,
        &mut DraftInitialSlotState,
    )>,
    mut overlays: Query<(&DraftInitialBoughtOverlay, &mut Visibility)>,
) {
    if !draft_initial_active(&mode, &draft_state) || !economy.initialized {
        return;
    }

    let pending_confirmations = std::mem::take(&mut draft_state.pending_confirmed_purchases);
    if pending_confirmations.is_empty() {
        return;
    }

    let mut unapplied_confirmations = Vec::new();
    for card_id in pending_confirmations {
        let Some((slot_entity, slot_index)) =
            mark_confirmed_purchase(card_id, &mut commands, &mut slots)
        else {
            unapplied_confirmations.push(card_id);
            continue;
        };

        commands
            .entity(slot_entity)
            .remove::<PendingDraftInitialPurchase>();
        set_bought_overlay_visibility(slot_index, Visibility::Visible, &mut overlays);
        hand_view.hand_size = hand_view
            .hand_size
            .saturating_add(1)
            .min(SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT + 1);
    }

    draft_state.pending_confirmed_purchases = unapplied_confirmations;
}

pub fn apply_shop_purchase_confirmations_system(
    mode: Res<ShopAuctionUiMode>,
    economy: Res<PlayerEconomyView>,
    mut hand_view: ResMut<ShopAuctionDraftHandView>,
    mut shop_state: ResMut<ShopAuctionShopState>,
    mut commands: Commands,
    mut slots: Query<(Entity, &ShopSlotCard, &mut ShopSlotState, &mut Text)>,
) {
    if !shop_active(&mode, &shop_state) {
        shop_state.pending_confirmed_purchases.clear();
        return;
    }

    if !economy.initialized {
        return;
    }

    let pending_confirmations = std::mem::take(&mut shop_state.pending_confirmed_purchases);
    if pending_confirmations.is_empty() {
        return;
    }

    let mut unapplied_confirmations = Vec::new();
    for card_id in pending_confirmations {
        if !mark_confirmed_shop_purchase(card_id, &mut commands, &mut slots) {
            unapplied_confirmations.push(card_id);
        } else {
            hand_view.hand_size = hand_view.hand_size.saturating_add(1).min(10);
        }
    }

    shop_state.pending_confirmed_purchases = unapplied_confirmations;
}

pub fn handle_shop_auction_control_interactions_system(
    mut interactions: Query<
        (
            Entity,
            &Interaction,
            Option<&DraftInitialSlotIndex>,
            Option<&DraftInitialReadyButton>,
            Option<&ShopSlotIndex>,
            Option<&ShopRefreshButton>,
            Option<&ShopReadyButton>,
        ),
        (
            Changed<Interaction>,
            Or<(
                With<DraftInitialSlotIndex>,
                With<DraftInitialReadyButton>,
                With<ShopSlotIndex>,
                With<ShopRefreshButton>,
                With<ShopReadyButton>,
            )>,
        ),
    >,
    mut draft_slots: MessageWriter<ShopAuctionDraftSlotClicked>,
    mut draft_ready: MessageWriter<ShopAuctionDraftReadyButtonClicked>,
    mut shop_slots: MessageWriter<ShopAuctionShopSlotClicked>,
    mut shop_refresh: MessageWriter<ShopAuctionShopRefreshClicked>,
    mut shop_ready: MessageWriter<ShopAuctionShopReadyButtonClicked>,
) {
    for (entity, interaction, draft_slot, draft_button, shop_slot, refresh_button, ready_button) in
        &mut interactions
    {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if draft_slot.is_some() {
            draft_slots.write(ShopAuctionDraftSlotClicked { slot: entity });
        } else if draft_button.is_some() {
            draft_ready.write(ShopAuctionDraftReadyButtonClicked { button: entity });
        } else if shop_slot.is_some() {
            shop_slots.write(ShopAuctionShopSlotClicked { slot: entity });
        } else if refresh_button.is_some() {
            shop_refresh.write(ShopAuctionShopRefreshClicked { button: entity });
        } else if ready_button.is_some() {
            shop_ready.write(ShopAuctionShopReadyButtonClicked { button: entity });
        }
    }
}

pub fn handle_draft_initial_slot_click_system(
    mode: Res<ShopAuctionUiMode>,
    economy: Res<PlayerEconomyView>,
    hand_view: Res<ShopAuctionDraftHandView>,
    draft_state: Res<ShopAuctionDraftInitialState>,
    mut clicks: MessageReader<ShopAuctionDraftSlotClicked>,
    mut slots: Query<(
        &DraftInitialSlotCard,
        &DraftInitialSlotGoldCost,
        &DraftInitialSlotState,
    )>,
    mut senders: Query<&mut MessageSender<C2SPurchaseCard>>,
    mut outbound: ResMut<ShopAuctionUiOutboundMessages>,
    mut commands: Commands,
    mut flash_writer: MessageWriter<ShopAuctionGoldCounterFlashRequested>,
) {
    for click in clicks.read() {
        info!(
            "DRAFT_INITIAL click received — handler=handle_draft_initial_slot_click_system, click_entity={:?}",
            click.slot
        );
        if !draft_initial_active(&mode, &draft_state) {
            continue;
        }

        let Ok((card, cost, slot_state)) = slots.get_mut(click.slot) else {
            continue;
        };

        if *slot_state != DraftInitialSlotState::Available {
            continue;
        }

        if hand_view.hand_size >= 10 {
            commands
                .entity(click.slot)
                .insert(DraftInitialSlotState::HandFullLocked);
            continue;
        }

        if cost.0 > economy.gold {
            outbound.gold_counter_flash_requests =
                outbound.gold_counter_flash_requests.saturating_add(1);
            flash_writer.write(ShopAuctionGoldCounterFlashRequested);
            continue;
        }

        let message = C2SPurchaseCard { card_id: card.0 };
        match senders.single_mut() {
            Ok(mut sender) => {
                tracing::info!(
                    msg_type = "C2SPurchaseCard",
                    card_id = ?message.card_id,
                    handler = "handle_draft_initial_slot_click_system",
                    "c2s_send: enter"
                );
                sender.send::<ReliableChannel>(message.clone());
            }
            Err(e) => {
                error!(
                    "C2S send failed: type=C2SPurchaseCard, handler=handle_draft_initial_slot_click_system, query_err={:?}",
                    e
                );
            }
        }
        outbound.purchase_cards.push(message);
        commands
            .entity(click.slot)
            .insert((DraftInitialSlotState::Pending, PendingDraftInitialPurchase));
    }
}

pub fn handle_draft_initial_ready_click_system(
    entities: Option<Res<ShopAuctionUiEntities>>,
    mode: Res<ShopAuctionUiMode>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
    mut clicks: MessageReader<ShopAuctionDraftReadyButtonClicked>,
    mut senders: Query<&mut MessageSender<C2SSignalReady>>,
    mut outbound: ResMut<ShopAuctionUiOutboundMessages>,
) {
    let Some(entities) = entities else {
        for _click in clicks.read() {}
        return;
    };

    for click in clicks.read() {
        if click.button != entities.draft_initial_ready_button
            || !draft_initial_active(&mode, &draft_state)
        {
            continue;
        }

        let message = C2SSignalReady {
            retract: draft_state.ready_signalled,
        };
        match senders.single_mut() {
            Ok(mut sender) => {
                tracing::info!(
                    msg_type = "C2SSignalReady",
                    retract = message.retract,
                    handler = "handle_draft_initial_ready_click_system",
                    "c2s_send: enter"
                );
                sender.send::<ReliableChannel>(message.clone());
            }
            Err(e) => {
                error!(
                    "C2S send failed: type=C2SSignalReady, handler=handle_draft_initial_ready_click_system, query_err={:?}",
                    e
                );
            }
        }
        outbound.ready_signals.push(message);
        draft_state.ready_signalled = !draft_state.ready_signalled;
    }
}

pub fn handle_draft_initial_objective_message_input_system(
    entities: Option<Res<ShopAuctionUiEntities>>,
    mode: Res<ShopAuctionUiMode>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
    mut dismiss_clicks: MessageReader<ShopAuctionDraftObjectiveDismissClicked>,
    mut retrieval_clicks: MessageReader<ShopAuctionDraftObjectiveRetrievalClicked>,
    mut panel_clicks: MessageReader<ShopAuctionDraftObjectivePanelClicked>,
) {
    let Some(entities) = entities else {
        for _click in dismiss_clicks.read() {}
        for _click in retrieval_clicks.read() {}
        for _click in panel_clicks.read() {}
        return;
    };

    for click in dismiss_clicks.read() {
        if click.button == entities.draft_initial_objective_dismiss_button
            && draft_initial_active(&mode, &draft_state)
            && draft_state.objective_overlay_visible
        {
            draft_state.dismiss_objective_overlay();
        }
    }

    for click in panel_clicks.read() {
        if draft_initial_active(&mode, &draft_state)
            && draft_state.objective_overlay_visible
            && click.target == DraftInitialObjectivePanelClickTarget::NonActionablePanel
        {
            draft_state.dismiss_objective_overlay();
        }
    }

    for click in retrieval_clicks.read() {
        if click.button == entities.draft_initial_objective_retrieval_button
            && draft_initial_active(&mode, &draft_state)
            && draft_state.objective_overlay_dismissed
        {
            draft_state.show_objective_overlay();
        }
    }
}

pub fn handle_draft_initial_objective_keyboard_system(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mode: Res<ShopAuctionUiMode>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
    mut esc_presses: MessageReader<ShopAuctionDraftObjectiveEscPressed>,
    mut enter_presses: MessageReader<ShopAuctionDraftObjectiveEnterPressed>,
) {
    let esc_requested = esc_presses.read().next().is_some()
        || keys
            .as_ref()
            .is_some_and(|keys| keys.just_pressed(KeyCode::Escape));
    let enter_requested = enter_presses.read().next().is_some()
        || keys
            .as_ref()
            .is_some_and(|keys| keys.just_pressed(KeyCode::Enter));

    if !draft_initial_active(&mode, &draft_state) {
        return;
    }

    if esc_requested
        && draft_state.objective_overlay_visible
        && draft_state.objective_focus_target == DraftInitialObjectiveFocusTarget::DismissButton
    {
        draft_state.dismiss_objective_overlay();
        return;
    }

    if !enter_requested {
        return;
    }

    match draft_state.objective_focus_target {
        DraftInitialObjectiveFocusTarget::DismissButton
            if draft_state.objective_overlay_visible =>
        {
            draft_state.dismiss_objective_overlay();
        }
        DraftInitialObjectiveFocusTarget::RetrievalAffordance
            if draft_state.objective_overlay_dismissed =>
        {
            draft_state.show_objective_overlay();
        }
        _ => {}
    }
}

pub fn handle_draft_initial_objective_button_interactions_system(
    mode: Res<ShopAuctionUiMode>,
    mut draft_state: ResMut<ShopAuctionDraftInitialState>,
    mut interactions: Query<
        (
            Entity,
            &Interaction,
            Option<&DraftInitialObjectiveDismissButton>,
            Option<&DraftInitialObjectiveRetrievalButton>,
        ),
        (
            Changed<Interaction>,
            Or<(
                With<DraftInitialObjectiveDismissButton>,
                With<DraftInitialObjectiveRetrievalButton>,
            )>,
        ),
    >,
) {
    for (_entity, interaction, dismiss, retrieval) in &mut interactions {
        if *interaction != Interaction::Pressed || !draft_initial_active(&mode, &draft_state) {
            continue;
        }

        if dismiss.is_some() && draft_state.objective_overlay_visible {
            draft_state.dismiss_objective_overlay();
        } else if retrieval.is_some() && draft_state.objective_overlay_dismissed {
            draft_state.show_objective_overlay();
        }
    }
}

pub fn handle_shop_slot_click_system(
    current: Res<CurrentClientPhase>,
    mode: Res<ShopAuctionUiMode>,
    economy: Res<PlayerEconomyView>,
    hand_view: Res<ShopAuctionDraftHandView>,
    shop_state: Res<ShopAuctionShopState>,
    mut clicks: MessageReader<ShopAuctionShopSlotClicked>,
    mut slots: Query<(&ShopSlotCard, &ShopSlotGoldCost, &mut ShopSlotState)>,
    mut senders: Query<&mut MessageSender<C2SPurchaseCard>>,
    mut outbound: ResMut<ShopAuctionUiOutboundMessages>,
    mut commands: Commands,
    mut flash_writer: MessageWriter<ShopAuctionGoldCounterFlashRequested>,
) {
    for click in clicks.read() {
        if current.phase != RoundPhase::DraftShop || !shop_active(&mode, &shop_state) {
            continue;
        }

        let Ok((card, cost, mut slot_state)) = slots.get_mut(click.slot) else {
            continue;
        };

        if *slot_state != ShopSlotState::Available {
            continue;
        }

        if hand_view.hand_size >= 10 {
            *slot_state = ShopSlotState::HandFullLocked;
            continue;
        }

        if !economy.initialized || cost.0 > economy.gold {
            outbound.gold_counter_flash_requests =
                outbound.gold_counter_flash_requests.saturating_add(1);
            flash_writer.write(ShopAuctionGoldCounterFlashRequested);
            continue;
        }

        let message = C2SPurchaseCard { card_id: card.0 };
        match senders.single_mut() {
            Ok(mut sender) => {
                tracing::info!(
                    msg_type = "C2SPurchaseCard",
                    card_id = ?message.card_id,
                    handler = "handle_shop_slot_click_system",
                    "c2s_send: enter"
                );
                sender.send::<ReliableChannel>(message.clone());
            }
            Err(e) => {
                error!(
                    "C2S send failed: type=C2SPurchaseCard, handler=handle_shop_slot_click_system, query_err={:?}",
                    e
                );
            }
        }
        outbound.purchase_cards.push(message);
        *slot_state = ShopSlotState::PendingPurchase;
        commands.entity(click.slot).insert(PendingShopPurchase);
    }
}

pub fn handle_shop_refresh_click_system(
    entities: Option<Res<ShopAuctionUiEntities>>,
    current: Res<CurrentClientPhase>,
    mode: Res<ShopAuctionUiMode>,
    economy: Res<PlayerEconomyView>,
    refresh_config: Res<ShopAuctionRefreshConfig>,
    mut shop_state: ResMut<ShopAuctionShopState>,
    mut clicks: MessageReader<ShopAuctionShopRefreshClicked>,
    mut senders: Query<&mut MessageSender<C2SRefreshShop>>,
    mut outbound: ResMut<ShopAuctionUiOutboundMessages>,
    mut slots: Query<(&mut ShopSlotState, &mut Text), With<ShopSlotIndex>>,
) {
    let Some(entities) = entities else {
        for _click in clicks.read() {}
        return;
    };

    for click in clicks.read() {
        if current.phase != RoundPhase::DraftShop
            || click.button != entities.shop_refresh_button
            || !shop_active(&mode, &shop_state)
        {
            continue;
        }

        if shop_state.refresh_in_flight {
            continue;
        }

        let refresh_cost = displayed_refresh_cost(
            refresh_config.refresh_base_cost,
            refresh_config.refresh_cap,
            shop_state.refresh_count_this_draft,
        );
        if !economy.initialized || economy.gold < refresh_cost {
            continue;
        }

        let message = C2SRefreshShop {};
        match senders.single_mut() {
            Ok(mut sender) => {
                tracing::info!(
                    msg_type = "C2SRefreshShop",
                    handler = "handle_shop_refresh_click_system",
                    "c2s_send: enter"
                );
                sender.send::<ReliableChannel>(message.clone());
            }
            Err(e) => {
                error!(
                    "C2S send failed: type=C2SRefreshShop, handler=handle_shop_refresh_click_system, query_err={:?}",
                    e
                );
            }
        }
        outbound.refresh_shops.push(message);
        shop_state.refresh_in_flight = true;

        for slot_entity in entities.shop_slots {
            let Ok((mut slot_state, mut text)) = slots.get_mut(slot_entity) else {
                continue;
            };
            *slot_state = ShopSlotState::Refreshing;
            text.0.clear();
            text.0.push_str("Refreshing...");
        }
    }
}

pub fn handle_shop_ready_click_system(
    entities: Option<Res<ShopAuctionUiEntities>>,
    current: Res<CurrentClientPhase>,
    mode: Res<ShopAuctionUiMode>,
    mut shop_state: ResMut<ShopAuctionShopState>,
    mut clicks: MessageReader<ShopAuctionShopReadyButtonClicked>,
    mut senders: Query<&mut MessageSender<C2SSignalReady>>,
    mut outbound: ResMut<ShopAuctionUiOutboundMessages>,
) {
    let Some(entities) = entities else {
        for _click in clicks.read() {}
        return;
    };

    for click in clicks.read() {
        if current.phase != RoundPhase::DraftShop
            || click.button != entities.shop_ready_button
            || !shop_active(&mode, &shop_state)
        {
            continue;
        }

        let message = C2SSignalReady {
            retract: shop_state.ready_signalled,
        };
        match senders.single_mut() {
            Ok(mut sender) => {
                tracing::info!(
                    msg_type = "C2SSignalReady",
                    retract = message.retract,
                    handler = "handle_shop_ready_click_system",
                    "c2s_send: enter"
                );
                sender.send::<ReliableChannel>(message.clone());
            }
            Err(e) => {
                error!(
                    "C2S send failed: type=C2SSignalReady, handler=handle_shop_ready_click_system, query_err={:?}",
                    e
                );
            }
        }
        outbound.ready_signals.push(message);
        shop_state.ready_signalled = !shop_state.ready_signalled;
    }
}

pub fn handle_auction_bid_button_click_system(
    entities: Option<Res<ShopAuctionUiEntities>>,
    current: Res<CurrentClientPhase>,
    mode: Res<ShopAuctionUiMode>,
    economy: Res<PlayerEconomyView>,
    local_gold: Res<ShopAuctionLocalGoldView>,
    hand_view: Res<ShopAuctionDraftHandView>,
    mut auction_state: ResMut<ShopAuctionAuctionState>,
    mut clicks: MessageReader<ShopAuctionBidButtonClicked>,
    buttons: Query<&AuctionBidButton>,
    mut senders: Query<&mut MessageSender<C2SPlaceBid>>,
    mut outbound: ResMut<ShopAuctionUiOutboundMessages>,
) {
    let Some(entities) = entities else {
        for _click in clicks.read() {}
        return;
    };

    for click in clicks.read() {
        if current.phase != RoundPhase::DraftAuction
            || !entities.auction_bid_buttons.contains(&click.button)
            || !auction_active(&mode, &auction_state)
        {
            continue;
        }

        if auction_state.in_flight_bid_amount.is_some()
            || auction_state.locally_expired()
            || hand_view.hand_size >= 10
            || local_player_is_leading(&auction_state, &local_gold)
        {
            continue;
        }

        let Ok(button) = buttons.get(click.button) else {
            continue;
        };
        let amount = auction_bid_amount(auction_state.current_price, button.increment);
        if (!economy.initialized && !local_gold.initialized)
            || local_gold.free_gold(&economy) < amount
        {
            continue;
        }

        let message = C2SPlaceBid { amount };
        match senders.single_mut() {
            Ok(mut sender) => {
                tracing::info!(
                    msg_type = "C2SPlaceBid",
                    amount = message.amount,
                    handler = "handle_auction_bid_button_click_system",
                    "c2s_send: enter"
                );
                sender.send::<ReliableChannel>(message.clone());
            }
            Err(e) => {
                error!(
                    "C2S send failed: type=C2SPlaceBid, handler=handle_auction_bid_button_click_system, query_err={:?}",
                    e
                );
            }
        }
        outbound.place_bids.push(message);
        auction_state.in_flight_bid_amount = Some(amount);
    }
}

pub fn handle_auction_bid_button_interactions_system(
    mut interactions: Query<(Entity, &Interaction), (Changed<Interaction>, With<AuctionBidButton>)>,
    mut clicks: MessageWriter<ShopAuctionBidButtonClicked>,
) {
    for (entity, interaction) in &mut interactions {
        if *interaction == Interaction::Pressed {
            clicks.write(ShopAuctionBidButtonClicked { button: entity });
        }
    }
}

/// PROMPT 1042 — converts a `Pressed` interaction on the Pass affordance
/// to a `ShopAuctionAuctionPassButtonClicked` message so the click handler
/// can run after input is drained.
pub fn handle_auction_pass_button_interactions_system(
    mut interactions: Query<
        (Entity, &Interaction),
        (Changed<Interaction>, With<AuctionPassButton>),
    >,
    mut clicks: MessageWriter<ShopAuctionAuctionPassButtonClicked>,
) {
    for (entity, interaction) in &mut interactions {
        if *interaction == Interaction::Pressed {
            clicks.write(ShopAuctionAuctionPassButtonClicked { button: entity });
        }
    }
}

/// PROMPT 1042 — toggles the local "I am passing" state on Pass-button
/// click. Pure UI state — the server has no Pass protocol message; the
/// auction simply expires if the player does not bid. Clicking Pass a
/// second time un-passes so the player can change their mind before the
/// auction ends.
pub fn handle_auction_pass_button_click_system(
    entities: Option<Res<ShopAuctionUiEntities>>,
    current: Res<CurrentClientPhase>,
    mode: Res<ShopAuctionUiMode>,
    auction_state: Res<ShopAuctionAuctionState>,
    hand_view: Res<ShopAuctionDraftHandView>,
    local_gold: Res<ShopAuctionLocalGoldView>,
    mut clicks: MessageReader<ShopAuctionAuctionPassButtonClicked>,
    mut locally_passed: ResMut<AuctionLocallyPassed>,
) {
    let Some(entities) = entities else {
        for _click in clicks.read() {}
        return;
    };

    for click in clicks.read() {
        if click.button != entities.auction_pass_button
            || current.phase != RoundPhase::DraftAuction
            || !auction_active(&mode, &auction_state)
        {
            continue;
        }

        // Pass is meaningless when the player is already locked out of
        // bidding (leading, hand full, in-flight bid).
        if local_player_is_leading(&auction_state, &local_gold)
            || hand_view.hand_size >= 10
            || auction_state.in_flight_bid_amount.is_some()
        {
            continue;
        }

        locally_passed.passed = !locally_passed.passed;
    }
}

pub fn handle_auction_bid_keyboard_focus_system(
    entities: Option<Res<ShopAuctionUiEntities>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    focus_states: Query<&AuctionBidFocusState, With<AuctionBidButton>>,
    mut keyboard_focus: ResMut<AuctionBidKeyboardFocus>,
    mut clicks: MessageWriter<ShopAuctionBidButtonClicked>,
) {
    let Some(entities) = entities else {
        keyboard_focus.focused_button = None;
        return;
    };

    let buttons = entities.auction_bid_buttons;
    if !keyboard_focus
        .focused_button
        .is_some_and(|focused| bid_button_focusable(focused, &focus_states))
    {
        keyboard_focus.focused_button = None;
    }

    if keyboard.just_pressed(KeyCode::Tab) {
        let start_index = keyboard_focus
            .focused_button
            .and_then(|focused| buttons.iter().position(|button| *button == focused))
            .map_or(0, |index| (index + 1) % buttons.len());

        keyboard_focus.focused_button =
            next_focusable_bid_button(&buttons, start_index, &focus_states);
    }

    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        if let Some(button) = keyboard_focus.focused_button {
            if bid_button_focusable(button, &focus_states) {
                clicks.write(ShopAuctionBidButtonClicked { button });
            }
        }
    }
}

pub fn sync_draft_initial_panel_system(
    mode: Res<ShopAuctionUiMode>,
    hand_view: Res<ShopAuctionDraftHandView>,
    draft_state: Res<ShopAuctionDraftInitialState>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    mut visibility_sets: ParamSet<(
        Query<&mut Visibility>,
        Query<
            (
                Entity,
                Option<&DraftInitialSlotCard>,
                &mut DraftInitialSlotState,
                &mut Visibility,
                &mut BackgroundColor,
            ),
            With<DraftInitialSlotIndex>,
        >,
        Query<(&DraftInitialBoughtOverlay, &mut Visibility)>,
    )>,
    mut texts: Query<&mut Text>,
    mut commands: Commands,
) {
    let Some(entities) = entities else {
        return;
    };

    let active = draft_initial_active(&mode, &draft_state);
    {
        let mut visibility = visibility_sets.p0();
        if *mode == ShopAuctionUiMode::DraftOffering {
            set_visibility(&mut visibility, entities.root, visibility_for(active));
        }
        set_visibility(
            &mut visibility,
            entities.draft_offering_panel,
            visibility_for(active),
        );
        set_visibility(
            &mut visibility,
            entities.draft_initial_ready_button,
            visibility_for(active),
        );
        set_visibility(
            &mut visibility,
            entities.draft_initial_ready_status,
            visibility_for(active && draft_state.ready_signalled),
        );
        set_visibility(
            &mut visibility,
            entities.draft_initial_hand_full_banner,
            visibility_for(active && hand_view.hand_size >= 10),
        );
        let objective_overlay_visible = active && draft_state.objective_overlay_visible;
        set_visibility(
            &mut visibility,
            entities.draft_initial_objective_overlay,
            visibility_for(objective_overlay_visible),
        );
        set_visibility(
            &mut visibility,
            entities.draft_initial_objective_copy,
            visibility_for(objective_overlay_visible),
        );
        set_visibility(
            &mut visibility,
            entities.draft_initial_objective_dismiss_button,
            visibility_for(objective_overlay_visible),
        );
        set_visibility(
            &mut visibility,
            entities.draft_initial_objective_retrieval_button,
            visibility_for(active && draft_state.objective_overlay_dismissed),
        );
    }

    {
        let mut slots = visibility_sets.p1();
        for (slot_entity, card, mut slot_state, mut visibility, mut background) in &mut slots {
            if !active || card.is_none() {
                *visibility = Visibility::Hidden;
                continue;
            }

            *visibility = Visibility::Visible;
            if hand_view.hand_size >= 10 && *slot_state != DraftInitialSlotState::Purchased {
                *slot_state = DraftInitialSlotState::HandFullLocked;
                commands
                    .entity(slot_entity)
                    .remove::<PendingDraftInitialPurchase>();
            }
            *background = match *slot_state {
                DraftInitialSlotState::Pending => {
                    BackgroundColor(Color::srgba(0.18, 0.28, 0.40, 0.95))
                }
                _ => BackgroundColor(Color::srgba(0.08, 0.12, 0.16, 0.9)),
            };
        }
    }

    if !active {
        let mut overlays = visibility_sets.p2();
        for (_overlay, mut visibility) in &mut overlays {
            *visibility = Visibility::Hidden;
        }
    }

    if let Ok(mut text) = texts.get_mut(entities.draft_initial_ready_button) {
        text.0.clear();
        if draft_state.ready_signalled {
            text.0.push_str("Retract Ready");
        } else {
            text.0.push_str("Ready");
        }
    }

    if let Ok(mut text) = texts.get_mut(entities.draft_initial_ready_status) {
        text.0.clear();
        if draft_state.ready_signalled {
            text.0.push_str("Waiting for opponent...");
        }
    }
}

/// PROMPT 1230 — reflect [`PhaseTimerState`] onto the
/// [`DraftInitialCountdownLabel`] text and visibility while the keep-9 modal
/// is active.
///
/// Mirrors the rounding convention of
/// [`crate::ui::hud::sync_hud_timer_countdown_text_system`]: remaining
/// milliseconds are rounded *up* to the next whole second so the player
/// never sees `0s` while time still remains. The label is hidden whenever
/// DraftInitial is not active so it cannot leak into the shop / auction /
/// placement / resolution UIs that share this plugin, and the text is
/// cleared on hide so a stale "1s" cannot reappear if the modal is shown
/// again later in the same session.
///
/// Reads the canonical timer; never writes to it. The HUD top-strip
/// countdown stays untouched — this label is additive modal-local clarity,
/// not a replacement.
pub fn sync_draft_initial_countdown_label_system(
    mode: Res<ShopAuctionUiMode>,
    draft_state: Res<ShopAuctionDraftInitialState>,
    timer: Res<PhaseTimerState>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    mut query: Query<(&mut Text, &mut Visibility), With<DraftInitialCountdownLabel>>,
) {
    let Some(entities) = entities else {
        return;
    };
    let Ok((mut text, mut visibility)) = query.get_mut(entities.draft_initial_countdown_label)
    else {
        return;
    };

    let active = draft_initial_active(&mode, &draft_state);
    let (target_text, target_visibility) = if active && timer.active && timer.duration_ms > 0 {
        let remaining_ms = timer.duration_ms.saturating_sub(timer.elapsed_ms);
        let remaining_s = remaining_ms.div_ceil(1_000);
        (format!("{remaining_s}s"), Visibility::Visible)
    } else {
        (String::new(), Visibility::Hidden)
    };

    if text.0 != target_text {
        text.0 = target_text;
    }
    if *visibility != target_visibility {
        *visibility = target_visibility;
    }
}

pub fn sync_shop_panel_system(
    mode: Res<ShopAuctionUiMode>,
    economy: Res<PlayerEconomyView>,
    hand_view: Res<ShopAuctionDraftHandView>,
    refresh_config: Res<ShopAuctionRefreshConfig>,
    shop_state: Res<ShopAuctionShopState>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    mut commands: Commands,
    mut shop_ui: ParamSet<(
        Query<&mut Visibility>,
        Query<
            (
                Entity,
                &ShopSlotIndex,
                Option<&ShopSlotCard>,
                Option<&ShopSlotGoldCost>,
                &mut ShopSlotState,
                &mut Visibility,
                &mut Text,
            ),
            With<ShopSlotIndex>,
        >,
        Query<&mut Text>,
        Query<&mut ShopRefreshButtonState>,
    )>,
) {
    let Some(entities) = entities else {
        return;
    };

    let active = shop_active(&mode, &shop_state);
    // PROMPT 1042 — when in DraftShop but no slots have arrived yet, the
    // shop panel still renders explicit phase chrome (title + empty-state)
    // while interactive affordances (refresh / ready) stay hidden so the
    // player cannot click a ghost button before offers exist.
    let interactive = active && shop_state.slots_loaded;
    {
        let mut visibility = shop_ui.p0();
        if *mode == ShopAuctionUiMode::Shop {
            set_visibility(&mut visibility, entities.root, visibility_for(active));
        }
        set_visibility(&mut visibility, entities.shop_panel, visibility_for(active));
        set_visibility(
            &mut visibility,
            entities.shop_phase_title,
            visibility_for(active),
        );
        set_visibility(
            &mut visibility,
            entities.shop_empty_state,
            visibility_for(active && !shop_state.slots_loaded),
        );
        set_visibility(
            &mut visibility,
            entities.shop_refresh_button,
            visibility_for(interactive),
        );
        set_visibility(
            &mut visibility,
            entities.shop_ready_button,
            visibility_for(interactive),
        );
        set_visibility(
            &mut visibility,
            entities.shop_ready_status,
            visibility_for(interactive && shop_state.ready_signalled),
        );
        set_visibility(
            &mut visibility,
            entities.shop_hand_full_banner,
            visibility_for(interactive && hand_view.hand_size >= 10),
        );
    }

    // PROMPT 1085 — collect per-slot affordance copy in the same pass that
    // computes slot visibility / state, then apply via `p2` (the generic
    // `Query<&mut Text>` already used by refresh / ready labels) so we
    // don't grow the ParamSet beyond the four queries already wired.
    let mut affordance_text: [String; SHOP_AUCTION_UI_SHOP_SLOT_COUNT] = Default::default();

    {
        let mut slots = shop_ui.p1();
        for (slot_entity, slot_index, card, cost, mut slot_state, mut visibility, mut text) in
            &mut slots
        {
            // PROMPT 1042 — slot wells stay hidden until `S2CShopSlots`
            // delivers offers; while waiting, the shop_empty_state copy
            // ("Waiting for shop offers...") is the placeholder so 3
            // "Empty" wells do not look like a loaded-empty shop.
            if !interactive {
                *visibility = Visibility::Hidden;
                if *slot_state == ShopSlotState::PendingPurchase {
                    *slot_state = ShopSlotState::Available;
                    commands.entity(slot_entity).remove::<PendingShopPurchase>();
                }
                continue;
            }

            *visibility = Visibility::Visible;
            if hand_view.hand_size >= 10 && card.is_some() && *slot_state != ShopSlotState::Empty {
                *slot_state = ShopSlotState::HandFullLocked;
                commands.entity(slot_entity).remove::<PendingShopPurchase>();
            }
            if *slot_state == ShopSlotState::Refreshing {
                text.0.clear();
                text.0.push_str("Refreshing...");
            }

            // PROMPT 1085 — compute the affordance / disabled-reason copy
            // surfaced by the child `ShopSlotAffordanceLabel` so the player
            // can read purchase intent at a glance.
            let idx = slot_index.0 as usize;
            if idx < affordance_text.len() {
                affordance_text[idx] = shop_slot_affordance_copy(
                    *slot_state,
                    card.map(|c| c.0),
                    cost.map(|c| c.0),
                    hand_view.hand_size,
                    &economy,
                );
            }
        }
    }

    {
        let mut texts = shop_ui.p2();
        for (index, label_entity) in entities.shop_slot_affordance_labels.iter().enumerate() {
            if let Ok(mut text) = texts.get_mut(*label_entity) {
                text.0.clear();
                if interactive {
                    if let Some(copy) = affordance_text.get(index) {
                        text.0.push_str(copy);
                    }
                }
            }
        }
    }

    let refresh_cost = displayed_refresh_cost(
        refresh_config.refresh_base_cost,
        refresh_config.refresh_cap,
        shop_state.refresh_count_this_draft,
    );
    // PROMPT 1042 — refresh enabled only once the first `S2CShopSlots`
    // batch has been applied, so the player cannot click an affordance
    // that has nothing to refresh.
    let refresh_enabled = interactive
        && !shop_state.refresh_in_flight
        && economy.initialized
        && economy.gold >= refresh_cost;

    {
        let mut refresh_buttons = shop_ui.p3();
        if let Ok(mut button_state) = refresh_buttons.get_mut(entities.shop_refresh_button) {
            button_state.enabled = refresh_enabled;
        }
    }

    let mut texts = shop_ui.p2();
    if let Ok(mut text) = texts.get_mut(entities.shop_refresh_button) {
        text.0.clear();
        if shop_state.refresh_in_flight {
            text.0.push_str("Refreshing...");
        } else {
            // PROMPT 1245 — surface the next refresh cost in parens directly
            // on the button label so the player sees the price before
            // clicking. Cost is derived from `displayed_refresh_cost`, which
            // mirrors the server-side formula via `ShopAuctionRefreshConfig`.
            text.0.push_str(&format!("Refresh ({refresh_cost}g)"));
        }
    }

    if let Ok(mut text) = texts.get_mut(entities.shop_ready_button) {
        text.0.clear();
        if shop_state.ready_signalled {
            text.0.push_str("Retract Ready");
        } else {
            text.0.push_str("Ready");
        }
    }

    if let Ok(mut text) = texts.get_mut(entities.shop_ready_status) {
        text.0.clear();
        if shop_state.ready_signalled {
            text.0.push_str("Waiting for opponent...");
        }
    }
}

pub fn tick_auction_preparing_timeout_system(
    time: Res<Time>,
    mut auction_state: ResMut<ShopAuctionAuctionState>,
    mut mode: ResMut<ShopAuctionUiMode>,
) {
    if auction_state.panel_state != ShopAuctionAuctionPanelState::Preparing {
        return;
    }

    let elapsed_ms = u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX);
    auction_state.preparing_elapsed_ms = auction_state
        .preparing_elapsed_ms
        .saturating_add(elapsed_ms);

    if auction_state.preparing_elapsed_ms >= AUCTION_PREPARING_TIMEOUT_MS {
        auction_state.panel_state = ShopAuctionAuctionPanelState::ConnectionError;
        *mode = ShopAuctionUiMode::AuctionPreparing;
    }
}

pub fn tick_auction_countdown_system(
    time: Res<Time>,
    mut auction_state: ResMut<ShopAuctionAuctionState>,
) {
    if auction_state.panel_state != ShopAuctionAuctionPanelState::Active {
        return;
    }

    let elapsed_ms = u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX);
    let was_expired = auction_state.timer_remaining_ms == 0;
    auction_state.timer_remaining_ms = auction_state.timer_remaining_ms.saturating_sub(elapsed_ms);
    if auction_state.timer_remaining_ms == 0 {
        if was_expired {
            auction_state.locally_expired_elapsed_ms = auction_state
                .locally_expired_elapsed_ms
                .saturating_add(elapsed_ms);
        } else {
            auction_state.locally_expired_elapsed_ms = 0;
        }
    } else {
        auction_state.locally_expired_elapsed_ms = 0;
    }
}

pub fn tick_auction_settlement_transition_system(
    time: Res<Time>,
    current: Res<CurrentClientPhase>,
    phase_view: Res<ClientPhaseView>,
    preferences: Option<Res<AccessibilityPreferences>>,
    mut mode: ResMut<ShopAuctionUiMode>,
    mut auction_state: ResMut<ShopAuctionAuctionState>,
    mut settlement_state: ResMut<ShopAuctionSettlementState>,
    mut shop_timer: ResMut<ShopAuctionShopTimerState>,
) {
    if auction_state.panel_state != ShopAuctionAuctionPanelState::Settling
        || !settlement_state.transition_active
    {
        return;
    }

    if matches!(
        current.phase,
        RoundPhase::Placement | RoundPhase::Resolution | RoundPhase::GameOver
    ) {
        auction_state.clear();
        settlement_state.clear();
        shop_timer.stop();
        *mode = ShopAuctionUiMode::Inactive;
        return;
    }

    let reduced_motion = preferences
        .as_deref()
        .is_some_and(|preferences| preferences.reduced_motion);
    settlement_state.transition_duration_ms = if reduced_motion {
        0
    } else {
        AUCTION_SETTLEMENT_TRANSITION_MS
    };

    let elapsed_ms = u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX);
    settlement_state.elapsed_ms = settlement_state.elapsed_ms.saturating_add(elapsed_ms);

    if current.phase != RoundPhase::DraftShop {
        return;
    }

    if settlement_state.elapsed_ms >= settlement_state.transition_duration_ms {
        settlement_state.finish_transition();
        auction_state.clear();
        shop_timer.start(phase_view.timer_duration_ms);
        *mode = ShopAuctionUiMode::Shop;
    }
}

pub fn tick_auction_toast_system(time: Res<Time>, mut toast_state: ResMut<ShopAuctionToastState>) {
    let elapsed_ms = u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX);
    toast_state.tick(elapsed_ms);
}

pub fn sync_auction_panel_system(
    mode: Res<ShopAuctionUiMode>,
    economy: Res<PlayerEconomyView>,
    local_gold: Res<ShopAuctionLocalGoldView>,
    hand_view: Res<ShopAuctionDraftHandView>,
    catalog: Res<ShopAuctionCardCatalog>,
    asset_server: Option<Res<AssetServer>>,
    auction_state: Res<ShopAuctionAuctionState>,
    settlement_state: Res<ShopAuctionSettlementState>,
    shop_state: Res<ShopAuctionShopState>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    locally_passed: Res<AuctionLocallyPassed>,
    mut keyboard_focus: ResMut<AuctionBidKeyboardFocus>,
    mut commands: Commands,
    mut auction_ui: ParamSet<(
        Query<&mut Visibility>,
        Query<&mut Text>,
        Query<(&mut AuctionTimerBarState, &mut Node, &mut BackgroundColor), With<AuctionTimerBar>>,
        Query<(Entity, &ShopFooterSlotIndex, &mut Visibility, &mut Text)>,
        Query<(
            Entity,
            &AuctionBidButton,
            &mut AuctionBidButtonState,
            &mut Visibility,
            &mut Text,
            &mut TextColor,
            &mut Node,
            &mut BorderColor,
            &mut BackgroundColor,
            &mut AuctionBidTargetBounds,
            &mut AuctionBidFocusState,
            &mut ImageNode,
        )>,
        Query<&mut AuctionFreeGoldCounterValue>,
        Query<
            (&mut AuctionFeaturedCardLeadLossState, &mut BorderColor),
            With<AuctionFeaturedCardFrame>,
        >,
    )>,
) {
    let Some(entities) = entities else {
        return;
    };

    let auction_visible = matches!(
        *mode,
        ShopAuctionUiMode::AuctionPreparing
            | ShopAuctionUiMode::Auction
            | ShopAuctionUiMode::AuctionSettling
    ) && auction_state.panel_visible();
    let footer_visible = *mode == ShopAuctionUiMode::Auction && auction_visible;
    let featured_card_state = auction_featured_card_lead_loss_state(&auction_state, &local_gold);
    let local_leading = local_player_is_leading(&auction_state, &local_gold);
    let hand_full = hand_view.hand_size >= 10;
    let opponent_leading = featured_card_state == AuctionFeaturedCardLeadLossState::Losing;
    let bid_status_visible = footer_visible && (local_leading || hand_full || opponent_leading);
    let local_free_gold = local_gold.free_gold(&economy);
    let has_gold_source = economy.initialized || local_gold.initialized;

    {
        let mut visibility = auction_ui.p0();
        if matches!(
            *mode,
            ShopAuctionUiMode::AuctionPreparing
                | ShopAuctionUiMode::Auction
                | ShopAuctionUiMode::AuctionSettling
        ) {
            set_visibility(
                &mut visibility,
                entities.root,
                visibility_for(auction_visible),
            );
        }
        set_visibility(
            &mut visibility,
            entities.auction_panel,
            visibility_for(auction_visible),
        );
        set_visibility(
            &mut visibility,
            entities.auction_featured_card,
            visibility_for(auction_visible),
        );
        set_visibility(
            &mut visibility,
            entities.auction_featured_card_frame,
            visibility_for(auction_visible),
        );
        set_visibility(
            &mut visibility,
            entities.auction_featured_card_stats,
            visibility_for(auction_visible),
        );
        set_visibility(
            &mut visibility,
            entities.auction_featured_card_keyword,
            visibility_for(auction_visible),
        );
        // PROMPT 1085 — price + timer labels follow the featured-card
        // visibility so they paint together with the card.
        set_visibility(
            &mut visibility,
            entities.auction_featured_card_price_label,
            visibility_for(auction_visible),
        );
        set_visibility(
            &mut visibility,
            entities.auction_featured_card_timer_label,
            visibility_for(auction_visible),
        );
        set_visibility(
            &mut visibility,
            entities.auction_status_text,
            visibility_for(auction_visible),
        );
        set_visibility(
            &mut visibility,
            entities.auction_timer_bar,
            visibility_for(auction_visible),
        );
        set_visibility(
            &mut visibility,
            entities.auction_bid_status_text,
            visibility_for(bid_status_visible),
        );
        set_visibility(
            &mut visibility,
            entities.auction_free_gold_counter_group,
            visibility_for(auction_visible),
        );
        for entity in entities.auction_free_gold_counters {
            set_visibility(&mut visibility, entity, visibility_for(auction_visible));
        }
        for entity in entities.auction_free_gold_counter_labels {
            set_visibility(&mut visibility, entity, visibility_for(auction_visible));
        }
        for entity in entities.auction_free_gold_counter_values {
            set_visibility(&mut visibility, entity, visibility_for(auction_visible));
        }
        set_visibility(
            &mut visibility,
            entities.shop_footer,
            visibility_for(footer_visible),
        );
        // PROMPT 1042 — Pass affordance is visible whenever the bid row is
        // visible. It stays visible after a local pass so the player keeps
        // the "PASSED" feedback (it just disables itself + the bid row).
        set_visibility(
            &mut visibility,
            entities.auction_pass_button,
            visibility_for(footer_visible),
        );
    }

    {
        let mut featured_card_frames = auction_ui.p6();
        if let Ok((mut state, mut border_color)) =
            featured_card_frames.get_mut(entities.auction_featured_card_frame)
        {
            *state = featured_card_state;
            *border_color =
                BorderColor::all(auction_featured_card_lead_loss_color(featured_card_state));
        }
    }

    {
        let mut texts = auction_ui.p1();
        if let Ok(mut text) = texts.get_mut(entities.auction_featured_card) {
            text.0.clear();
            if let Some(card_id) = auction_state.card_id {
                let card = catalog.cards.get(&card_id);
                // Sprint 18 story-022 (AC6 / AC9): per-card art binds onto
                // the `CardSlotArtImage` child entity (16 / 16 / 16 / 96
                // image-inset rectangle of the 380 × 280 card) instead
                // of the slot root. The slot root preserves its
                // spawn-time `BackgroundColor` floor without an
                // `ImageNode` overlay; the art child carries
                // `NodeImageMode::Auto` so the source aspect ratio is
                // honoured (UI-1129-05 resolved).
                apply_card_display_art(
                    &mut commands,
                    entities.auction_featured_card_art,
                    card,
                    asset_server.as_deref(),
                );
                // PROMPT 1182 — render only the card name on the parent
                // featured-card entity. The prior `"{name}\n{rarity} - {N}g"`
                // payload painted a second H1 line directly under the name
                // that overlapped both the dedicated price label child
                // (`AuctionFeaturedCardPriceLabel` carries "Bid: {N}g"
                // at the top of the card) and the stats child (rarity
                // surfaced via card_type / cost). AUDIT-1129 UI-1129-02
                // observed the resulting two-line ghosting (`Vault·Sentry /
                // Rare – 3g` overlapping the price band). The dedicated
                // child labels are the single source of truth for price
                // and rarity now; the parent contributes the name only.
                let name = card
                    .map(|card| card.name_en.as_str())
                    .unwrap_or("Unknown card");
                text.0.push_str(name);
            } else {
                clear_card_display_art(&mut commands, entities.auction_featured_card_art);
            }
        }

        // PROMPT 1085 — current-price + numeric time-left labels. Always
        // re-written each frame while auction_visible so we don't leak the
        // prior auction's copy when a new card arrives. When no card is
        // buffered, both labels are cleared.
        if let Ok(mut text) = texts.get_mut(entities.auction_featured_card_price_label) {
            text.0.clear();
            if auction_visible && auction_state.card_id.is_some() {
                text.0
                    .push_str(&format!("Bid: {}g", auction_state.current_price));
            }
        }
        if let Ok(mut text) = texts.get_mut(entities.auction_featured_card_timer_label) {
            text.0.clear();
            if auction_visible && auction_state.card_id.is_some() {
                text.0
                    .push_str(&auction_featured_timer_label(&auction_state));
            }
        }

        // PROMPT 1029: featured-card stats node was spawned with `Text::new("")`
        // (story 016 reserved the typography slot but never wired content). Bind
        // ATK/HP + mana cost for the current auction card so the player can read
        // combat values on the most prominent card surface.
        if let Ok(mut text) = texts.get_mut(entities.auction_featured_card_stats) {
            text.0.clear();
            if let Some(card_id) = auction_state.card_id {
                if let Some(card) = catalog.cards.get(&card_id) {
                    let stats = format_card_combat_stats(card);
                    if stats.is_empty() {
                        text.0.push_str(&format!("Cost {}g", card.cost));
                    } else {
                        text.0
                            .push_str(&format!("ATK/HP {stats} · Cost {}g", card.cost));
                    }
                }
            }
        }

        if let Ok(mut text) = texts.get_mut(entities.auction_status_text) {
            text.0.clear();
            match auction_state.panel_state {
                ShopAuctionAuctionPanelState::Preparing => {
                    text.0.push_str("Auction starting...");
                }
                ShopAuctionAuctionPanelState::ConnectionError => {
                    text.0.push_str("Connection error - awaiting server...");
                }
                ShopAuctionAuctionPanelState::Active => {
                    if auction_state.locally_expired() {
                        text.0.push_str(auction_expired_status_text(
                            auction_state.locally_expired_elapsed_ms,
                        ));
                    } else {
                        text.0.push_str("Auction live");
                    }
                }
                ShopAuctionAuctionPanelState::Settling => {
                    text.0.push_str(settlement_state.overlay_text());
                }
                ShopAuctionAuctionPanelState::Hidden => {}
            }
        }

        if let Ok(mut text) = texts.get_mut(entities.auction_bid_status_text) {
            text.0.clear();
            if local_leading {
                text.0.push_str("YOU ARE LEADING");
            } else if hand_full {
                text.0.push_str("Hand full - no bids possible this auction");
            } else if opponent_leading {
                text.0.push_str("OPPONENT LEADING");
            }
        }

        let free_gold_text = format!("{local_free_gold}g");
        for entity in entities.auction_free_gold_counter_values {
            if let Ok(mut text) = texts.get_mut(entity) {
                text.0.clear();
                text.0.push_str(&free_gold_text);
            }
        }
    }

    {
        let mut counter_values = auction_ui.p5();
        for entity in entities.auction_free_gold_counter_values {
            if let Ok(mut value) = counter_values.get_mut(entity) {
                value.amount = local_free_gold;
            }
        }
    }

    {
        let mut timer_bars = auction_ui.p2();
        if let Ok((mut state, mut node, mut background)) =
            timer_bars.get_mut(entities.auction_timer_bar)
        {
            let active_countdown = auction_state.panel_state
                == ShopAuctionAuctionPanelState::Active
                && auction_state.timer_duration_ms > 0;
            let target_width_percent = if active_countdown {
                auction_timer_width_percent(
                    auction_state.timer_remaining_ms,
                    auction_state.timer_duration_ms,
                )
            } else {
                100.0
            };

            node.width = Val::Px(AUCTION_READABILITY_INFO_WIDTH_PX * target_width_percent / 100.0);
            *state = AuctionTimerBarState {
                greyed: !active_countdown,
                countdown_active: active_countdown,
                connection_error: auction_state.panel_state
                    == ShopAuctionAuctionPanelState::ConnectionError,
            };
            *background = BackgroundColor(if active_countdown {
                Color::srgb(0.25, 0.72, 0.43)
            } else {
                Color::srgb(0.36, 0.38, 0.42)
            });
        }
    }

    {
        let focused_button = keyboard_focus.focused_button;
        let mut focused_button_is_focusable = false;
        let mut bid_buttons = auction_ui.p4();
        for (index, button_entity) in entities.auction_bid_buttons.into_iter().enumerate() {
            let Ok((
                entity,
                button,
                mut state,
                mut visibility,
                mut text,
                mut text_color,
                mut node,
                mut border_color,
                mut background_color,
                mut target_bounds,
                mut focus_state,
                mut image_node,
            )) = bid_buttons.get_mut(button_entity)
            else {
                continue;
            };

            let amount = auction_bid_amount(auction_state.current_price, button.increment);
            let base_state = auction_bid_button_state(
                amount,
                local_free_gold,
                has_gold_source,
                hand_full,
                local_leading,
                auction_state.waiting_for_local_gold_after_opponent_bid(),
                &auction_state,
            );
            // PROMPT 1042 — local Pass dims the bid row to communicate the
            // player's intent. Pass is purely local — auction settles
            // server-side based on actual bids — so we only adjust the
            // visual state, never short-circuit a bid that is already in
            // flight (`InFlight` wins over `Pass`).
            let next_state =
                if locally_passed.passed && base_state != AuctionBidButtonState::InFlight {
                    AuctionBidButtonState::GenericDisabled
                } else {
                    base_state
                };

            *visibility = visibility_for(
                footer_visible && next_state != AuctionBidButtonState::HiddenLeading,
            );
            *state = next_state;
            // PROMPT 1116 — `card_id.is_none()` is the canonical signal
            // for "we haven't drained `S2CAuctionCard` yet". The
            // entity-level text is reset to the pending label so the
            // row never advertises the misleading numeric
            // `BidButtonLabel` ("0g\n(+1)") during the phase-entry race
            // window even though `Visibility::Hidden` keeps the row
            // off-screen for normal flow. The chrome image override is
            // intentionally narrower: `auction_bid_chrome_state` only
            // returns `None` for `HiddenLeading` so we don't strip the
            // existing chrome handle from every InSession-idle bid
            // button — `chrome_wiring_test` exercises that wiring.
            let card_not_ready = auction_state.card_id.is_none();
            if let Some(ref server) = asset_server {
                image_node.image = match auction_bid_chrome_state(next_state) {
                    Some(chrome) => server.load(bid_button_asset(chrome)),
                    None => Handle::default(),
                };
            }
            node.width = Val::Px(AUCTION_BID_TARGET_WIDTH_PX);
            node.height = Val::Px(AUCTION_BID_TARGET_HEIGHT_PX);
            *target_bounds = AuctionBidTargetBounds {
                width_px: AUCTION_BID_TARGET_WIDTH_PX,
                height_px: AUCTION_BID_TARGET_HEIGHT_PX,
            };

            let focusable =
                auction_bid_state_focusable(next_state) && *visibility == Visibility::Visible;
            let focused = focused_button == Some(entity) && focusable;
            if focused {
                focused_button_is_focusable = true;
            }
            *focus_state = AuctionBidFocusState {
                order: (index + 1) as u8,
                focusable,
                focused,
                focus_ring_visible: focused,
                focus_ring_width_px: if focused {
                    AUCTION_BID_FOCUS_RING_WIDTH_PX
                } else {
                    0.0
                },
            };
            node.border = UiRect::all(Val::Px(if focused {
                AUCTION_BID_FOCUS_RING_WIDTH_PX
            } else {
                1.0
            }));
            *border_color = BorderColor::all(auction_bid_border_color(next_state, focused));
            *background_color = BackgroundColor(auction_bid_background_color(next_state));

            text.0.clear();
            if next_state == AuctionBidButtonState::InFlight {
                text.0.push_str("BIDDING...");
            } else if card_not_ready {
                // PROMPT 1116 — before `S2CAuctionCard` arrives the
                // numeric `BidButtonLabel` ("0g\n(+1)") is meaningless
                // because the auction starting price has not yet been
                // sent by the server. Surface the spawn-state pending
                // label instead so the entity contract is "Loading…"
                // until `card_id.is_some()`. AC2 then asserts the swap
                // to numeric on `S2CAuctionCard` drain.
                text.0.push_str(AUCTION_BID_BUTTON_LOADING_LABEL);
            } else {
                text.0.push_str(
                    &BidButtonLabel {
                        total_commitment: amount,
                        increment: button.increment,
                    }
                    .text(),
                );
            }
            *text_color = TextColor(auction_bid_text_color(next_state));
        }
        if focused_button.is_some() && !focused_button_is_focusable {
            keyboard_focus.focused_button = None;
        }
    }

    // PROMPT 1042 — Pass button text reflects local pass state. Server
    // never sees this; auction settles based on actual bids placed.
    {
        let mut texts = auction_ui.p1();
        if let Ok(mut text) = texts.get_mut(entities.auction_pass_button) {
            text.0.clear();
            if locally_passed.passed {
                text.0.push_str("PASSED");
            } else {
                text.0.push_str("PASS");
            }
        }
    }

    {
        let footer_cards = shop_state.footer_slots();
        let mut footer_slots = auction_ui.p3();
        for slot_entity in entities.shop_footer_slots {
            let Ok((entity, slot_index, mut visibility, mut text)) =
                footer_slots.get_mut(slot_entity)
            else {
                continue;
            };

            if !footer_visible {
                *visibility = Visibility::Hidden;
                continue;
            }

            *visibility = Visibility::Visible;
            let card_id = if shop_state.footer_slots_loaded {
                footer_cards[slot_index.0 as usize]
            } else {
                None
            };
            apply_shop_footer_slot(
                &mut commands,
                entity,
                card_id,
                &catalog.cards,
                asset_server.as_deref(),
                &mut text,
            );
        }
    }
}

pub fn sync_settlement_overlay_system(
    mode: Res<ShopAuctionUiMode>,
    settlement_state: Res<ShopAuctionSettlementState>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    mut visibility: Query<&mut Visibility>,
    mut texts: Query<&mut Text, With<AuctionSettlementOverlayText>>,
    mut text_colors: Query<&mut TextColor, With<AuctionSettlementOverlayText>>,
) {
    let Some(entities) = entities else {
        return;
    };

    let visible = *mode == ShopAuctionUiMode::AuctionSettling
        && settlement_state.transition_active
        && settlement_state.outcome.is_some();

    if visible {
        set_visibility(&mut visibility, entities.root, Visibility::Visible);
    }
    set_visibility(
        &mut visibility,
        entities.settlement_overlay,
        visibility_for(visible),
    );
    set_visibility(
        &mut visibility,
        entities.settlement_overlay_text,
        visibility_for(visible),
    );

    if let Ok(mut text) = texts.get_mut(entities.settlement_overlay_text) {
        text.0.clear();
        if visible {
            // PROMPT 1347 / AC7 — loser-side toast names the price. The
            // dynamic copy includes `for {amount}g` for OpponentWinner so
            // the loser sees the bid commitment.
            text.0.push_str(&settlement_state.dynamic_overlay_text());
        }
    }

    if let Ok(mut color) = text_colors.get_mut(entities.settlement_overlay_text) {
        *color = TextColor(Color::srgba(
            0.98,
            0.94,
            0.80,
            0.72 + settlement_state.transition_progress() * 0.28,
        ));
    }
}

pub fn sync_auction_toast_system(
    mode: Res<ShopAuctionUiMode>,
    toast_state: Res<ShopAuctionToastState>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    mut visibility: Query<&mut Visibility>,
    mut texts: Query<&mut Text, With<AuctionToastText>>,
    mut text_colors: Query<&mut TextColor, With<AuctionToastText>>,
) {
    let Some(entities) = entities else {
        return;
    };

    let visible = *mode == ShopAuctionUiMode::Auction && toast_state.active;
    set_visibility(
        &mut visibility,
        entities.toast_root,
        visibility_for(visible),
    );
    set_visibility(
        &mut visibility,
        entities.toast_text,
        visibility_for(visible),
    );

    if let Ok(mut text) = texts.get_mut(entities.toast_text) {
        text.0.clear();
        if visible {
            text.0.push_str(&toast_state.text);
        }
    }

    if let Ok(mut color) = text_colors.get_mut(entities.toast_text) {
        *color = TextColor(Color::srgba(0.98, 0.92, 0.72, toast_state.alpha()));
    }
}

pub fn spawn_shop_auction_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    refresh_config: Res<ShopAuctionRefreshConfig>,
    existing: Option<Res<ShopAuctionUiEntities>>,
    // Sprint 18 story 020 (S18-UI-PLAY-AREA-CONTAINER-001): when
    // `PlayAreaPlugin` is registered, the four migrated panels
    // (`bottom_panel_node` / `auction_panel_node` / `footer_node` /
    // `toast_node`) parent into the `PlayAreaRoot` entity instead of
    // the full-viewport `ShopAuctionUiRoot`. Harness apps without the
    // plugin (e.g. `client/src/shop_auction_*_harness.rs`) keep
    // parenting into the local `ShopAuctionUiRoot` via the
    // `unwrap_or(root)` fallback below.
    play_area_root: Option<Res<crate::ui::PlayAreaRoot>>,
) {
    if existing.is_some() {
        return;
    }

    let root = commands
        .spawn((
            Name::new("Shop Auction UI Root"),
            ShopAuctionUiEntity,
            ShopAuctionUiRoot,
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

    let play_area_parent = play_area_root.as_ref().map(|p| p.0).unwrap_or(root);

    #[cfg(feature = "ui_picking")]
    commands
        .entity(root)
        .insert(bevy::picking::Pickable::IGNORE);

    let draft_offering_panel = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::DraftOffering,
        "Shop Auction Draft Offering Root",
        draft_initial_centering_root_node(),
    );
    // PROMPT 1051 — the centering root doubles as the modal scrim. A
    // near-black background painted at the canonical OVERLAY_SCRIM_ALPHA
    // dims and blocks the layer beneath so any pre-session warning text
    // (photosensitivity body copy, etc.) cannot bleed through the modal
    // edge during DraftInitial. Pairs with the fully-opaque modal panel
    // body below so the keep-9 picker reads as a true modal layer.
    commands.entity(draft_offering_panel).insert((
        z_layers::MODAL,
        BackgroundColor(Color::srgba(
            0.02,
            0.05,
            0.08,
            overlays::OVERLAY_SCRIM_ALPHA,
        )),
    ));
    let draft_initial_modal_panel =
        spawn_draft_initial_modal_panel(&mut commands, draft_offering_panel);
    let draft_initial_grid =
        spawn_draft_initial_grid_container(&mut commands, draft_initial_modal_panel);
    let (draft_initial_slots, draft_initial_bought_overlays) =
        spawn_draft_initial_grid(&mut commands, draft_initial_grid);
    // PROMPT 1051 — footer band anchored to the modal bottom; Ready and
    // Retract Ready / waiting status now parent into the footer instead
    // of floating absolutely in the modal's top-right column.
    let draft_initial_modal_footer =
        spawn_draft_initial_modal_footer(&mut commands, draft_initial_modal_panel);
    let draft_initial_ready_button =
        spawn_draft_initial_ready_button(&mut commands, draft_initial_modal_footer);
    let draft_initial_ready_status =
        spawn_draft_initial_status_text(&mut commands, draft_initial_modal_footer);
    let draft_initial_hand_full_banner =
        spawn_draft_initial_hand_full_banner(&mut commands, draft_initial_modal_panel);
    let (
        draft_initial_objective_overlay,
        draft_initial_objective_copy,
        draft_initial_objective_dismiss_button,
    ) = spawn_draft_initial_objective_overlay(&mut commands, draft_initial_modal_panel);
    let draft_initial_objective_retrieval_button =
        spawn_draft_initial_objective_retrieval_button(&mut commands, draft_initial_modal_panel);
    let draft_initial_countdown_label =
        spawn_draft_initial_countdown_label(&mut commands, draft_initial_modal_panel);
    let shop_panel = spawn_panel_root(
        &mut commands,
        play_area_parent,
        ShopAuctionPanelRoot::Shop,
        "Shop Auction Shop Root",
        bottom_panel_node(),
    );
    commands.entity(shop_panel).insert(z_layers::UI_BASE);
    commands
        .entity(shop_panel)
        .insert(ImageNode::new(asset_server.load(SHOP_PANEL_CHROME_ASSET)));
    let shop_phase_title = spawn_shop_phase_title(&mut commands, shop_panel);
    let shop_empty_state = spawn_shop_empty_state(&mut commands, shop_panel);
    let (shop_slots, shop_slot_affordance_labels) =
        spawn_shop_slots(&mut commands, &asset_server, shop_panel);
    let shop_refresh_button = spawn_shop_refresh_button(&mut commands, shop_panel);
    let shop_ready_button = spawn_shop_ready_button(&mut commands, shop_panel);
    let shop_ready_status = spawn_shop_ready_status(&mut commands, shop_panel);
    let shop_hand_full_banner = spawn_shop_hand_full_banner(&mut commands, shop_panel);
    let auction_panel = spawn_panel_root(
        &mut commands,
        play_area_parent,
        ShopAuctionPanelRoot::Auction,
        "Shop Auction Auction Root",
        auction_panel_node(),
    );
    commands.entity(auction_panel).insert(z_layers::UI_BASE);
    // Reuses SHOP_PANEL_CHROME_ASSET as a placeholder until an auction-specific
    // chrome constant lands (PAW-TD-003-a is accept-risk for friend-game scope).
    commands
        .entity(auction_panel)
        .insert(ImageNode::new(asset_server.load(SHOP_PANEL_CHROME_ASSET)));
    let AuctionContents {
        featured_card: auction_featured_card,
        featured_card_frame: auction_featured_card_frame,
        featured_card_art: auction_featured_card_art,
        featured_card_label_strip: auction_featured_card_label_strip,
        featured_card_stats: auction_featured_card_stats,
        featured_card_keyword: auction_featured_card_keyword,
        featured_card_price_label: auction_featured_card_price_label,
        featured_card_timer_label: auction_featured_card_timer_label,
        status_text: auction_status_text,
        timer_bar: auction_timer_bar,
        bid_status_text: auction_bid_status_text,
        free_gold_counter_group: auction_free_gold_counter_group,
        free_gold_counters: auction_free_gold_counters,
        free_gold_counter_labels: auction_free_gold_counter_labels,
        free_gold_counter_values: auction_free_gold_counter_values,
        bid_buttons: auction_bid_buttons,
        pass_button: auction_pass_button,
    } = spawn_auction_contents(
        &mut commands,
        &asset_server,
        &refresh_config.bid_increments,
        auction_panel,
    );
    let shop_footer = spawn_panel_root(
        &mut commands,
        play_area_parent,
        ShopAuctionPanelRoot::ShopFooter,
        "Shop Auction Footer Root",
        footer_node(),
    );
    commands.entity(shop_footer).insert(z_layers::UI_BASE);
    let shop_footer_slots = spawn_shop_footer_slots(&mut commands, shop_footer);
    let toast_root = spawn_panel_root(
        &mut commands,
        play_area_parent,
        ShopAuctionPanelRoot::Toast,
        "Shop Auction Toast Root",
        toast_node(),
    );
    commands.entity(toast_root).insert(z_layers::TOAST);
    let toast_text = spawn_auction_toast_text(&mut commands, toast_root);
    let settlement_overlay = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::SettlementOverlay,
        "Shop Auction Settlement Overlay Root",
        overlay_node(),
    );
    commands.entity(settlement_overlay).insert((
        BackgroundColor(Color::srgba(
            0.02,
            0.05,
            0.08,
            overlays::OVERLAY_SCRIM_ALPHA,
        )),
        z_layers::UI_OVERLAY,
    ));
    let settlement_overlay_text = spawn_settlement_overlay_text(&mut commands, settlement_overlay);

    commands.insert_resource(ShopAuctionUiEntities {
        root,
        draft_offering_panel,
        draft_initial_modal_panel,
        draft_initial_modal_footer,
        draft_initial_grid,
        draft_initial_slots,
        draft_initial_bought_overlays,
        draft_initial_ready_button,
        draft_initial_ready_status,
        draft_initial_hand_full_banner,
        draft_initial_objective_overlay,
        draft_initial_objective_copy,
        draft_initial_objective_dismiss_button,
        draft_initial_objective_retrieval_button,
        draft_initial_countdown_label,
        shop_panel,
        shop_phase_title,
        shop_empty_state,
        shop_slots,
        shop_slot_affordance_labels,
        shop_refresh_button,
        shop_ready_button,
        shop_ready_status,
        shop_hand_full_banner,
        auction_panel,
        auction_featured_card,
        auction_featured_card_frame,
        auction_featured_card_art,
        auction_featured_card_label_strip,
        auction_featured_card_stats,
        auction_featured_card_keyword,
        auction_featured_card_price_label,
        auction_featured_card_timer_label,
        auction_status_text,
        auction_timer_bar,
        auction_bid_status_text,
        auction_free_gold_counter_group,
        auction_free_gold_counters,
        auction_free_gold_counter_labels,
        auction_free_gold_counter_values,
        auction_bid_buttons,
        auction_pass_button,
        shop_footer,
        shop_footer_slots,
        toast_root,
        toast_text,
        settlement_overlay,
        settlement_overlay_text,
    });
}

fn despawn_shop_auction_ui(mut commands: Commands, entities: Option<Res<ShopAuctionUiEntities>>) {
    let Some(entities) = entities else {
        return;
    };

    commands.entity(entities.root).despawn();
    commands.remove_resource::<ShopAuctionUiEntities>();
}

fn spawn_panel_root(
    commands: &mut Commands,
    parent: Entity,
    marker: ShopAuctionPanelRoot,
    name: &'static str,
    node: Node,
) -> Entity {
    let root = commands
        .spawn((
            Name::new(name),
            ShopAuctionUiEntity,
            marker,
            node,
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();

    commands.spawn((
        Name::new(format!("{name} Label")),
        ShopAuctionUiEntity,
        Text::new(""),
        shop_auction_text_font(typography::H3),
        TextColor(Color::srgb(0.92, 0.94, 0.96)),
        panel_label_node(),
        Visibility::Hidden,
        ChildOf(root),
    ));

    root
}

fn spawn_auction_toast_text(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Name::new("Shop Auction Toast Text"),
            ShopAuctionUiEntity,
            AuctionToastText,
            Text::new(""),
            shop_auction_text_font(typography::BODY),
            TextColor(Color::srgba(0.98, 0.92, 0.72, 0.0)),
            auction_toast_text_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_settlement_overlay_text(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Name::new("Shop Auction Settlement Text"),
            ShopAuctionUiEntity,
            AuctionSettlementOverlayText,
            Text::new(""),
            shop_auction_text_font(typography::H2),
            TextColor(Color::srgba(0.98, 0.94, 0.80, 0.0)),
            settlement_overlay_text_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct PendingDraftInitialPurchase;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct PendingShopPurchase;

fn spawn_draft_initial_modal_panel(commands: &mut Commands, parent: Entity) -> Entity {
    // PROMPT 1051 — modal body is fully opaque (alpha 1.0). The previous
    // 0.94-alpha let any text painted at the same z-layer (notably the
    // pre-session photosensitivity warning body) bleed through the modal
    // edges. The fully-opaque body combined with the centering-root scrim
    // gives the keep-9 modal a proper modal contract: nothing beneath it
    // is visible.
    //
    // PROMPT 1080 — `Visibility::Inherited` (not `Visible`). Bevy 0.18
    // treats `Visibility::Visible` as unconditional, overriding any
    // ancestor `Hidden`. With `Visible` here, the opaque modal body kept
    // painting after DraftInitial closed (DraftShop/Placement/Auction/
    // Resolution) even though the `draft_offering_panel` parent was
    // correctly hidden — the user saw a black slab covering the board.
    // `Inherited` lets the parent's per-phase visibility govern the modal.
    commands
        .spawn((
            Name::new("Shop Auction Draft Initial Modal Panel"),
            ShopAuctionUiEntity,
            DraftInitialModalPanel,
            draft_initial_modal_panel_node(),
            BackgroundColor(Color::srgb(0.055, 0.062, 0.078)),
            BorderColor::all(Color::srgba(0.82, 0.86, 0.90, 0.26)),
            Visibility::Inherited,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_draft_initial_modal_footer(commands: &mut Commands, parent: Entity) -> Entity {
    // PROMPT 1051 — flex row anchored to the modal panel bottom, hosting
    // the Ready / Retract Ready button and the waiting-for-opponent status
    // text. The top border visually separates the footer from the grid
    // band above so the keep decision reads as a single grouped action.
    //
    // PROMPT 1080 — `Visibility::Inherited` so the footer follows the
    // modal panel (and the `draft_offering_panel` scrim) when DraftInitial
    // ends. See `spawn_draft_initial_modal_panel` for the full rationale.
    commands
        .spawn((
            Name::new("Shop Auction Draft Initial Modal Footer"),
            ShopAuctionUiEntity,
            DraftInitialModalFooter,
            draft_initial_modal_footer_node(),
            BorderColor::all(Color::srgba(0.82, 0.86, 0.90, 0.20)),
            Visibility::Inherited,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_draft_initial_grid_container(commands: &mut Commands, parent: Entity) -> Entity {
    // PROMPT 1080 — `Visibility::Inherited` so the grid container hides
    // with the modal panel when leaving DraftInitial. See
    // `spawn_draft_initial_modal_panel` for the full rationale.
    commands
        .spawn((
            Name::new("Shop Auction Draft Initial Grid"),
            ShopAuctionUiEntity,
            DraftInitialGrid,
            draft_initial_grid_node(),
            Visibility::Inherited,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_draft_initial_grid(
    commands: &mut Commands,
    parent: Entity,
) -> (
    [Entity; SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT],
    [Entity; SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT],
) {
    let mut overlays = Vec::with_capacity(SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT);
    let slots = std::array::from_fn(|index| {
        // Spawn the slot container WITHOUT Text so it doesn't render as a white dot
        // when no text is set. Text lives in a dedicated child entity instead.
        let slot = commands
            .spawn((
                Name::new(format!("Shop Auction Draft Slot {index}")),
                ShopAuctionUiEntity,
                DraftInitialSlotIndex(index as u8),
                Button,
                Interaction::None,
                draft_initial_slot_node(),
                BackgroundColor(Color::srgba(0.08, 0.12, 0.16, 0.9)),
                Visibility::Hidden,
                ChildOf(parent),
            ))
            .id();

        // Sprint 18 story-022 — card-art child sized to the canonical
        // `CardSlotKind::DraftGrid` image inset. Carries the
        // `CardSlotArtImage` marker so the per-card art handle binds
        // here (via `apply_card_display_art` in
        // `handle_draft_offering_system`) rather than on the slot root,
        // structurally enforcing the PROMPT 1117 chrome-preservation
        // contract (AC4 / AC5 / AC7).
        let (art_node, art_z) = card_slot_art_image_node(CardSlotKind::DraftGrid);
        let art_entity = commands
            .spawn((
                Name::new(format!("Shop Auction Draft Slot {index} Art")),
                ShopAuctionUiEntity,
                CardSlotArtImage,
                art_node,
                art_z,
                card_slot_art_image_component(),
                Visibility::Inherited,
                ChildOf(slot),
            ))
            .id();

        // Child text entity — holds card name + cost display. Sprint
        // 18 story-022 leaves this child parented to the slot directly
        // (not under a `CardSlotLabelStrip`) because the draft grid
        // slot is the simplest case: the slot already paints an
        // opaque dark `BackgroundColor`, the text already sits in
        // legible contrast against it, and the `CardSlotKind::DraftGrid`
        // text-inset rectangle is in the right-half landscape region
        // where the existing default-flex Node places the text.
        let text_entity = commands
            .spawn((
                Name::new(format!("Shop Auction Draft Slot {index} Text")),
                ShopAuctionUiEntity,
                DraftInitialSlotTextLabel,
                Text::new(""),
                shop_auction_text_font(typography::CAPTION),
                TextColor(Color::srgb(0.92, 0.94, 0.96)),
                ChildOf(slot),
            ))
            .id();

        // Store the text child's id on the slot so systems can reach it directly.
        commands.entity(slot).insert((
            DraftInitialSlotText(text_entity),
            DraftInitialSlotArt(art_entity),
        ));

        let overlay = commands
            .spawn((
                Name::new(format!("Shop Auction Draft Slot {index} Bought Overlay")),
                ShopAuctionUiEntity,
                DraftInitialBoughtOverlay {
                    slot_index: index as u8,
                },
                Text::new("BOUGHT"),
                shop_auction_text_font(typography::CAPTION),
                TextColor(Color::srgb(1.0, 0.94, 0.78)),
                overlay_text_node(),
                Visibility::Hidden,
                ChildOf(slot),
            ))
            .id();
        overlays.push(overlay);
        slot
    });

    let overlays = overlays
        .try_into()
        .expect("draft grid should always create exactly 9 overlays");
    (slots, overlays)
}

fn spawn_draft_initial_ready_button(commands: &mut Commands, parent: Entity) -> Entity {
    // PROMPT 1182 — explicit fill + border so the Ready CTA reads
    // unambiguously as an interactive button against the modal panel
    // background, instead of as a bare label.
    commands
        .spawn((
            Name::new("Shop Auction Draft Ready Button"),
            ShopAuctionUiEntity,
            DraftInitialReadyButton,
            Button,
            Interaction::None,
            Text::new("Ready"),
            shop_auction_text_font(typography::BODY),
            TextColor(Color::srgb(0.98, 0.93, 0.72)),
            BackgroundColor(primary_action_button_background_color()),
            BorderColor::all(primary_action_button_border_color()),
            draft_initial_ready_button_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_draft_initial_status_text(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Name::new("Shop Auction Draft Ready Status"),
            ShopAuctionUiEntity,
            DraftInitialReadyStatus,
            Text::new(""),
            shop_auction_text_font(typography::CAPTION),
            TextColor(Color::srgb(0.80, 0.86, 0.94)),
            draft_initial_status_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_draft_initial_hand_full_banner(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Name::new("Shop Auction Draft Hand Full Banner"),
            ShopAuctionUiEntity,
            DraftInitialHandFullBanner,
            Text::new("Hand full - cannot buy more cards."),
            shop_auction_text_font(typography::CAPTION),
            TextColor(Color::srgb(1.0, 0.78, 0.55)),
            draft_initial_hand_full_banner_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_draft_initial_countdown_label(commands: &mut Commands, parent: Entity) -> Entity {
    // PROMPT 1230 — modal-local countdown anchored to the top-right of the
    // keep-9 modal panel. Spawned `Visibility::Hidden`; the sync system
    // promotes it to `Visible` only when DraftInitial is active *and* the
    // canonical `PhaseTimerState` is actively counting down.
    commands
        .spawn((
            Name::new("Shop Auction Draft Initial Countdown"),
            ShopAuctionUiEntity,
            DraftInitialCountdownLabel,
            Text::new(""),
            shop_auction_text_font(typography::H2),
            TextColor(Color::srgb(0.98, 0.93, 0.72)),
            draft_initial_countdown_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_draft_initial_objective_overlay(
    commands: &mut Commands,
    parent: Entity,
) -> (Entity, Entity, Entity) {
    let overlay = commands
        .spawn((
            Name::new("Shop Auction Draft Objective Overlay"),
            ShopAuctionUiEntity,
            DraftInitialObjectiveOverlay,
            draft_initial_objective_overlay_node(),
            BackgroundColor(Color::srgba(0.02, 0.05, 0.08, 0.92)),
            BorderColor::all(Color::srgb(0.74, 0.92, 0.92)),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();

    let copy = commands
        .spawn((
            Name::new("Shop Auction Draft Objective Copy"),
            ShopAuctionUiEntity,
            DraftInitialObjectiveCopy,
            Text::new(DRAFT_INITIAL_OBJECTIVE_COPY),
            shop_auction_text_font(typography::CAPTION),
            TextColor(Color::srgb(0.96, 0.98, 1.0)),
            draft_initial_objective_copy_node(),
            Visibility::Hidden,
            ChildOf(overlay),
        ))
        .id();

    // PROMPT 1182 — Dismiss is a button; add the shared primary-action
    // chrome so the affordance reads as a button against the dark
    // objective-overlay scrim.
    let dismiss = commands
        .spawn((
            Name::new("Shop Auction Draft Objective Dismiss"),
            ShopAuctionUiEntity,
            DraftInitialObjectiveDismissButton,
            Button,
            Interaction::None,
            Text::new("Dismiss"),
            shop_auction_text_font(typography::CAPTION),
            TextColor(Color::srgb(0.98, 0.93, 0.72)),
            BackgroundColor(primary_action_button_background_color()),
            BorderColor::all(primary_action_button_border_color()),
            draft_initial_objective_dismiss_node(),
            Visibility::Hidden,
            ChildOf(overlay),
        ))
        .id();

    (overlay, copy, dismiss)
}

fn spawn_draft_initial_objective_retrieval_button(
    commands: &mut Commands,
    parent: Entity,
) -> Entity {
    // PROMPT 1182 — visible button chrome so the Objective retrieval
    // affordance reads as an interactive button instead of inert text.
    commands
        .spawn((
            Name::new("Shop Auction Draft Objective Retrieval"),
            ShopAuctionUiEntity,
            DraftInitialObjectiveRetrievalButton,
            Button,
            Interaction::None,
            Text::new("Objective"),
            shop_auction_text_font(typography::CAPTION),
            TextColor(Color::srgb(0.74, 0.92, 0.92)),
            BackgroundColor(primary_action_button_background_color()),
            BorderColor::all(primary_action_button_border_color()),
            draft_initial_objective_retrieval_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_shop_slots(
    commands: &mut Commands,
    asset_server: &AssetServer,
    parent: Entity,
) -> (
    [Entity; SHOP_AUCTION_UI_SHOP_SLOT_COUNT],
    [Entity; SHOP_AUCTION_UI_SHOP_SLOT_COUNT],
) {
    let mut affordance_labels = [Entity::PLACEHOLDER; SHOP_AUCTION_UI_SHOP_SLOT_COUNT];
    let slots = std::array::from_fn(|index| {
        // PROMPT 1085 — `BackgroundColor` is the visual floor so the tile
        // is always distinguishable from the panel chrome, even if the
        // `SHOP_SLOT_WELL_IDLE_ASSET` ImageNode handle is still loading
        // (the asset is tiny and load-cheap, but the player should never
        // see a black void during the load race window — AUDIT-1076-04).
        let slot = commands
            .spawn((
                Name::new(format!("Shop Auction Shop Slot {index}")),
                ShopAuctionUiEntity,
                ShopSlotIndex(index as u8),
                ShopSlotState::Empty,
                Button,
                Interaction::None,
                shop_slot_node(index),
                ImageNode::new(asset_server.load(SHOP_SLOT_WELL_IDLE_ASSET)),
                BackgroundColor(Color::srgba(0.10, 0.13, 0.18, 0.95)),
                BorderColor::all(Color::srgba(0.86, 0.90, 0.96, 0.40)),
                Text::new("Empty"),
                shop_auction_text_font(typography::CAPTION),
                TextColor(Color::srgb(0.92, 0.94, 0.96)),
                Visibility::Hidden,
                ChildOf(parent),
            ))
            .id();

        // PROMPT 1085 — child affordance label, parented to the slot so it
        // inherits visibility from the parent well. Text is set by
        // `sync_shop_panel_system` based on `ShopSlotState` + economy.
        let affordance = commands
            .spawn((
                Name::new(format!("Shop Auction Shop Slot {index} Affordance")),
                ShopAuctionUiEntity,
                ShopSlotAffordanceLabel {
                    slot_index: index as u8,
                },
                Text::new(""),
                shop_auction_text_font(typography::CAPTION),
                TextColor(Color::srgb(1.0, 0.94, 0.78)),
                shop_slot_affordance_label_node(),
                Visibility::Inherited,
                ChildOf(slot),
            ))
            .id();

        affordance_labels[index] = affordance;
        slot
    });
    (slots, affordance_labels)
}

fn spawn_shop_refresh_button(commands: &mut Commands, parent: Entity) -> Entity {
    // PROMPT 1182 — visible button chrome. The refresh affordance
    // previously spawned with only a 1px border and no fill, which made
    // it look like static text overlaid on the shop panel chrome.
    commands
        .spawn((
            Name::new("Shop Auction Refresh Button"),
            ShopAuctionUiEntity,
            ShopRefreshButton,
            Button,
            Interaction::None,
            ShopRefreshButtonState { enabled: false },
            Text::new("Refresh (1g)"),
            shop_auction_text_font(typography::BODY),
            TextColor(Color::srgb(0.74, 0.92, 0.92)),
            BackgroundColor(primary_action_button_background_color()),
            BorderColor::all(primary_action_button_border_color()),
            shop_refresh_button_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_shop_ready_button(commands: &mut Commands, parent: Entity) -> Entity {
    // PROMPT 1182 — visible button chrome. Same root cause as
    // `spawn_shop_refresh_button` — the Ready CTA was an unstyled text
    // affordance under the shop panel chrome.
    commands
        .spawn((
            Name::new("Shop Auction Shop Ready Button"),
            ShopAuctionUiEntity,
            ShopReadyButton,
            Button,
            Interaction::None,
            Text::new("Ready"),
            shop_auction_text_font(typography::BODY),
            TextColor(Color::srgb(0.98, 0.93, 0.72)),
            BackgroundColor(primary_action_button_background_color()),
            BorderColor::all(primary_action_button_border_color()),
            shop_ready_button_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_shop_ready_status(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Name::new("Shop Auction Shop Ready Status"),
            ShopAuctionUiEntity,
            ShopReadyStatus,
            Text::new(""),
            shop_auction_text_font(typography::CAPTION),
            TextColor(Color::srgb(0.80, 0.86, 0.94)),
            shop_ready_status_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_shop_hand_full_banner(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Name::new("Shop Auction Shop Hand Full Banner"),
            ShopAuctionUiEntity,
            ShopHandFullBanner,
            Text::new("Hand full - play cards during PLACEMENT to free space."),
            shop_auction_text_font(typography::CAPTION),
            TextColor(Color::srgb(1.0, 0.78, 0.55)),
            shop_hand_full_banner_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_shop_phase_title(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Name::new("Shop Auction Shop Phase Title"),
            ShopAuctionUiEntity,
            ShopPhaseTitle,
            Text::new("SHOP"),
            shop_auction_text_font(typography::H2),
            TextColor(Color::srgb(0.98, 0.88, 0.45)),
            shop_phase_title_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_shop_empty_state(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Name::new("Shop Auction Shop Empty State"),
            ShopAuctionUiEntity,
            ShopEmptyState,
            Text::new("Waiting for shop offers..."),
            shop_auction_text_font(typography::BODY),
            TextColor(Color::srgb(0.86, 0.90, 0.96)),
            shop_empty_state_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_auction_contents(
    commands: &mut Commands,
    asset_server: &AssetServer,
    bid_increments: &[u32; 3],
    parent: Entity,
) -> AuctionContents {
    // PROMPT 1085 — solid `BackgroundColor` is the visual floor for the
    // featured card. Audit AUDIT-1076-04 observed an "empty black modal"
    // during DraftAuction even when `S2CAuctionCard` had arrived; the prior
    // root node had no background, so when the card-art `ImageNode` was
    // still loading or absent (missing-asset path), nothing painted at all
    // and the player saw a black slab. The fallback color paints a dark
    // neutral so the card outline is always discernible.
    let featured_card = commands
        .spawn((
            Name::new("Shop Auction Featured Auction Card"),
            ShopAuctionUiEntity,
            AuctionFeaturedCard,
            Text::new(""),
            shop_auction_text_font(typography::H1),
            TextColor(Color::srgb(0.98, 0.94, 0.80)),
            auction_featured_card_node(),
            BackgroundColor(Color::srgba(0.07, 0.10, 0.14, 0.95)),
            BorderColor::all(auction_featured_card_accent_color()),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();

    // Sprint 14 story 016 AC2: explicit visual frame primitive painted as
    // a transparent sub-node overlapping the featured card. The frame
    // marker is observable via `AuctionFeaturedCardFrame` queries; story
    // 018 (lead / loss state) extends this primitive by recoloring the
    // border without re-authoring geometry.
    let featured_card_frame = commands
        .spawn((
            Name::new("Shop Auction Featured Card Frame"),
            ShopAuctionUiEntity,
            AuctionFeaturedCardFrame,
            AuctionFeaturedCardLeadLossState::Neutral,
            auction_featured_card_frame_node(),
            BorderColor::all(auction_featured_card_accent_color()),
            BackgroundColor(Color::NONE),
            Visibility::Hidden,
            ChildOf(featured_card),
        ))
        .id();

    // Sprint 18 story-022 (`S18-UI-CARD-ART-AND-LABEL-STRIP-001`) —
    // canonical card-art child sized to
    // `CardSlotKind::AuctionFeatured` image inset (16 / 16 / 16 / 96).
    // Carries the `CardSlotArtImage` marker so the per-card art handle
    // binds onto this child (via `apply_card_display_art` in the
    // `S2CAuctionCard` handler) instead of the slot root. Empty image
    // handle at spawn time keeps the chrome-preservation contract
    // intact; `NodeImageMode::Auto` (from
    // `card_slot_art_image_component`) prevents the UI-1129-05
    // banner-stretch defect.
    let (art_node, art_z) = card_slot_art_image_node(CardSlotKind::AuctionFeatured);
    let featured_card_art = commands
        .spawn((
            Name::new("Shop Auction Featured Card Art"),
            ShopAuctionUiEntity,
            CardSlotArtImage,
            art_node,
            art_z,
            card_slot_art_image_component(),
            Visibility::Inherited,
            ChildOf(featured_card),
        ))
        .id();

    // Sprint 18 story-022 (AC3 / AC9): opaque label strip sized to the
    // `CardSlotKind::AuctionFeatured` text inset (16 / 16 / 200 / 16).
    // Carries the `CardSlotLabelStrip` marker, an opaque
    // `BackgroundColor` (alpha ≥ 0.85), a `min_width` clamp, and
    // `Overflow::clip_x()`. The four featured-card text children
    // (stats / keyword / price / timer) re-parent into the strip so
    // the per-card readouts paint against an opaque label background
    // rather than the underlying card art (UI-1129-02 / S-04 closed
    // structurally).
    let (strip_node, strip_z) = card_slot_label_strip_node(CardSlotKind::AuctionFeatured);
    let featured_card_label_strip = commands
        .spawn((
            Name::new("Shop Auction Featured Card Label Strip"),
            ShopAuctionUiEntity,
            CardSlotLabelStrip,
            strip_node,
            strip_z,
            BackgroundColor(card_slot_label_strip_background_color()),
            Visibility::Inherited,
            ChildOf(featured_card),
        ))
        .id();

    // Sprint 14 story 016 AC4: typography hierarchy markers. The stats
    // and keyword readouts carry `H2` and `BODY` font sizes so that the
    // numeric hierarchy assertion (name `H1` > stats `H2` > keyword
    // `BODY`) is observable via stable marker queries. Authored as
    // hidden sub-nodes (test-observable UI state) — story 016 is
    // layout / composition / hierarchy scope only and does not author
    // new visible content; future content rows may set their `Text`.
    //
    // Sprint 18 story-022 AC9: stats / keyword / price / timer reparent
    // from `featured_card` to `featured_card_label_strip`. Their
    // absolute-position offsets now resolve against the strip's
    // rectangle (the bottom text band of the card), keeping all four
    // readouts inside the opaque strip background.
    let featured_card_stats = commands
        .spawn((
            Name::new("Shop Auction Featured Card Stats"),
            ShopAuctionUiEntity,
            AuctionFeaturedCardStats,
            Text::new(""),
            shop_auction_text_font(typography::H2),
            TextColor(Color::srgb(0.92, 0.94, 0.96)),
            auction_featured_card_stats_node(),
            Visibility::Hidden,
            ChildOf(featured_card_label_strip),
        ))
        .id();

    let featured_card_keyword = commands
        .spawn((
            Name::new("Shop Auction Featured Card Keyword"),
            ShopAuctionUiEntity,
            AuctionFeaturedCardKeyword,
            Text::new(""),
            shop_auction_text_font(typography::BODY),
            TextColor(Color::srgb(0.86, 0.90, 0.96)),
            auction_featured_card_keyword_node(),
            Visibility::Hidden,
            ChildOf(featured_card_label_strip),
        ))
        .id();

    // PROMPT 1085 — current-price + numeric time-left readouts anchored
    // inside the featured card so the bid economics are always legible.
    // Sprint 18 story-022 AC9: re-parented under the canonical
    // `CardSlotLabelStrip` child alongside `stats` / `keyword`.
    let featured_card_price_label = commands
        .spawn((
            Name::new("Shop Auction Featured Card Price"),
            ShopAuctionUiEntity,
            AuctionFeaturedCardPriceLabel,
            Text::new(""),
            shop_auction_text_font(typography::H2),
            TextColor(Color::srgb(0.98, 0.93, 0.40)),
            auction_featured_card_price_label_node(),
            Visibility::Inherited,
            ChildOf(featured_card_label_strip),
        ))
        .id();

    let featured_card_timer_label = commands
        .spawn((
            Name::new("Shop Auction Featured Card Timer"),
            ShopAuctionUiEntity,
            AuctionFeaturedCardTimerLabel,
            Text::new(""),
            shop_auction_text_font(typography::CAPTION),
            TextColor(Color::srgb(0.86, 0.90, 0.96)),
            auction_featured_card_timer_label_node(),
            Visibility::Inherited,
            ChildOf(featured_card_label_strip),
        ))
        .id();

    let status_text = commands
        .spawn((
            Name::new("Shop Auction Auction Status"),
            ShopAuctionUiEntity,
            AuctionStatusText,
            Text::new(""),
            shop_auction_text_font(typography::H3),
            TextColor(Color::srgb(0.86, 0.90, 0.96)),
            auction_status_text_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();

    let timer_bar = commands
        .spawn((
            Name::new("Shop Auction Auction Timer Bar"),
            ShopAuctionUiEntity,
            AuctionTimerBar,
            AuctionTimerBarState {
                greyed: true,
                countdown_active: false,
                connection_error: false,
            },
            Node {
                width: Val::Percent(100.0),
                ..auction_timer_bar_node()
            },
            BackgroundColor(Color::srgb(0.36, 0.38, 0.42)),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();

    let bid_status_text = commands
        .spawn((
            Name::new("Shop Auction Bid Status"),
            ShopAuctionUiEntity,
            AuctionBidStatusText,
            Text::new(""),
            shop_auction_text_font(typography::H3),
            TextColor(Color::srgb(0.98, 0.88, 0.40)),
            auction_bid_status_text_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();

    let (
        free_gold_counter_group,
        free_gold_counters,
        free_gold_counter_labels,
        free_gold_counter_values,
    ) = spawn_auction_free_gold_counter_group(commands, parent);

    let bid_buttons = std::array::from_fn(|index| {
        commands
            .spawn((
                Name::new(format!("Shop Auction Bid Button {index}")),
                ShopAuctionUiEntity,
                AuctionBidButton {
                    increment: bid_increments[index],
                },
                AuctionBidButtonState::GenericDisabled,
                AuctionBidTargetBounds {
                    width_px: AUCTION_BID_TARGET_WIDTH_PX,
                    height_px: AUCTION_BID_TARGET_HEIGHT_PX,
                },
                AuctionBidFocusState::inactive((index + 1) as u8),
                Interaction::None,
                // PROMPT 1116 — spawn-state pending text. The bid-button
                // text was empty until `S2CAuctionCard` arrived, leaving
                // a zero-content `Text` component on the entity during
                // the phase-entry race window even though the row was
                // `Visibility::Hidden`. Spawning with `Loading…` makes
                // the entity-level contract non-empty from frame zero so
                // any inspector / regression assertion sees a meaningful
                // pending string before the auction card resolves.
                Text::new(AUCTION_BID_BUTTON_LOADING_LABEL),
                shop_auction_text_font(typography::H3),
                TextColor(Color::srgb(0.98, 0.93, 0.72)),
                BackgroundColor(auction_bid_background_color(
                    AuctionBidButtonState::GenericDisabled,
                )),
                BorderColor::all(auction_bid_border_color(
                    AuctionBidButtonState::GenericDisabled,
                    false,
                )),
                auction_bid_button_node(index),
                Visibility::Hidden,
                ChildOf(parent),
            ))
            .insert(ImageNode::new(
                asset_server.load(bid_button_asset(BidButtonChromeState::Disabled)),
            ))
            .id()
    });

    let pass_button = commands
        .spawn((
            Name::new("Shop Auction Pass Button"),
            ShopAuctionUiEntity,
            AuctionPassButton,
            Button,
            Interaction::None,
            Text::new("PASS"),
            shop_auction_text_font(typography::H3),
            TextColor(Color::srgb(0.92, 0.94, 0.96)),
            BackgroundColor(Color::srgba(0.12, 0.14, 0.18, 0.75)),
            BorderColor::all(Color::srgba(0.92, 0.94, 0.96, 0.55)),
            auction_pass_button_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();

    AuctionContents {
        featured_card,
        featured_card_frame,
        featured_card_art,
        featured_card_label_strip,
        featured_card_stats,
        featured_card_keyword,
        featured_card_price_label,
        featured_card_timer_label,
        status_text,
        timer_bar,
        bid_status_text,
        free_gold_counter_group,
        free_gold_counters,
        free_gold_counter_labels,
        free_gold_counter_values,
        bid_buttons,
        pass_button,
    }
}

/// PROMPT 1085 — bundle of auction-panel sub-entities returned by
/// [`spawn_auction_contents`]. Replaced the previous 13-tuple return so
/// new sub-nodes (price / timer labels) can land without churning every
/// caller.
///
/// Sprint 18 story-022 (`S18-UI-CARD-ART-AND-LABEL-STRIP-001`) adds
/// `featured_card_art` and `featured_card_label_strip` to the bundle
/// so the runtime per-card art-binding system can target the
/// `CardSlotArtImage` child instead of the slot root.
struct AuctionContents {
    featured_card: Entity,
    featured_card_frame: Entity,
    featured_card_art: Entity,
    featured_card_label_strip: Entity,
    featured_card_stats: Entity,
    featured_card_keyword: Entity,
    featured_card_price_label: Entity,
    featured_card_timer_label: Entity,
    status_text: Entity,
    timer_bar: Entity,
    bid_status_text: Entity,
    free_gold_counter_group: Entity,
    free_gold_counters: [Entity; AUCTION_FREE_GOLD_COUNTER_COUNT],
    free_gold_counter_labels: [Entity; AUCTION_FREE_GOLD_COUNTER_COUNT],
    free_gold_counter_values: [Entity; AUCTION_FREE_GOLD_COUNTER_COUNT],
    bid_buttons: [Entity; 3],
    pass_button: Entity,
}

fn spawn_auction_free_gold_counter_group(
    commands: &mut Commands,
    parent: Entity,
) -> (
    Entity,
    [Entity; AUCTION_FREE_GOLD_COUNTER_COUNT],
    [Entity; AUCTION_FREE_GOLD_COUNTER_COUNT],
    [Entity; AUCTION_FREE_GOLD_COUNTER_COUNT],
) {
    let group = commands
        .spawn((
            Name::new("Shop Auction Free Gold Counter Group"),
            ShopAuctionUiEntity,
            AuctionFreeGoldCounterGroup,
            auction_free_gold_counter_group_node(),
            BackgroundColor(Color::srgba(0.05, 0.07, 0.10, 0.72)),
            BorderColor::all(Color::srgba(0.98, 0.78, 0.30, 0.42)),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();

    let counter_triplets = AUCTION_FREE_GOLD_COUNTER_KINDS.map(|kind| {
        let counter = commands
            .spawn((
                Name::new(format!("Shop Auction Free Gold Counter {:?}", kind)),
                ShopAuctionUiEntity,
                AuctionFreeGoldCounter { kind },
                auction_free_gold_counter_node(),
                Visibility::Hidden,
                ChildOf(group),
            ))
            .id();

        let label = commands
            .spawn((
                Name::new(format!("Shop Auction Free Gold {:?} Label", kind)),
                ShopAuctionUiEntity,
                AuctionFreeGoldCounterLabel { kind },
                Text::new(kind.label()),
                shop_auction_text_font(AUCTION_FREE_GOLD_COUNTER_LABEL_FONT_PX),
                TextColor(Color::srgb(0.78, 0.84, 0.92)),
                auction_free_gold_counter_label_node(),
                Visibility::Hidden,
                ChildOf(counter),
            ))
            .id();

        let value = commands
            .spawn((
                Name::new(format!("Shop Auction Free Gold {:?} Value", kind)),
                ShopAuctionUiEntity,
                AuctionFreeGoldCounterValue { kind, amount: 0 },
                Text::new("0g"),
                shop_auction_text_font(AUCTION_FREE_GOLD_COUNTER_VALUE_FONT_PX),
                TextColor(Color::srgb(0.98, 0.78, 0.30)),
                auction_free_gold_counter_value_node(),
                Visibility::Hidden,
                ChildOf(counter),
            ))
            .id();

        (counter, label, value)
    });

    (
        group,
        counter_triplets.map(|(counter, _, _)| counter),
        counter_triplets.map(|(_, label, _)| label),
        counter_triplets.map(|(_, _, value)| value),
    )
}

fn spawn_shop_footer_slots(
    commands: &mut Commands,
    parent: Entity,
) -> [Entity; SHOP_AUCTION_UI_SHOP_SLOT_COUNT] {
    std::array::from_fn(|index| {
        commands
            .spawn((
                Name::new(format!("Shop Auction Footer Slot {index}")),
                ShopAuctionUiEntity,
                ShopFooterSlotIndex(index as u8),
                ShopFooterSlotState::EmptyLocked,
                Text::new("Locked"),
                shop_auction_text_font(typography::CAPTION),
                TextColor(Color::srgba(0.92, 0.94, 0.96, 0.30)),
                shop_footer_slot_node(index),
                Visibility::Hidden,
                ChildOf(parent),
            ))
            .id()
    })
}

/// Shop-bottom-panel `Node` builder. `pub` so the
/// `tests/integration/ui_clean_pass/play_area_budget_test.rs`
/// integration bin can assert the migrated Node shape (AC2 + AC7) at
/// the canonical 1280×720 / 1366×768 / 1920×1080 viewport matrix.
pub fn bottom_panel_node() -> Node {
    // Sprint 18 story 020 (S18-UI-PLAY-AREA-CONTAINER-001) AC2: shop
    // panel parents into `PlayArea` and fills the middle band instead of
    // anchoring `bottom: 0, height: 260` against the viewport (the
    // viewport-anchored literal that overlapped `HandBar` and produced
    // overlap S-01 per PROMPT 1180 §2 RC-1). Within `PlayArea` the panel
    // occupies the canonical `viewport − HeaderBar − FooterBar −
    // HandBar` middle band, with the shop slots / refresh / ready
    // buttons still positioned via their existing per-child `Absolute`
    // anchors relative to the panel box.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(0.0),
        bottom: Val::Px(0.0),
        ..default()
    }
}

fn draft_initial_centering_root_node() -> Node {
    Node {
        display: Display::Flex,
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(0.0),
        bottom: Val::Px(0.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        padding: UiRect::all(Val::Px(DRAFT_INITIAL_MODAL_PADDING_PX)),
        ..default()
    }
}

/// PROMPT 1349 (Sprint 18 story 026 / Lane J) — draft-initial modal
/// panel node. The pre-1349 layout declared `height: 360 px` *and*
/// `max_height: 92 %`, a fixed-px-plus-percent conflict (PROMPT 1180
/// §1.4 S-08) that pinned the modal to 360 px at every resolution.
///
/// Story 026 §5 C-5 requires every in-scope modal to declare
/// `max_height: 92 %` + `Overflow::scroll_y()` and forbids the literal
/// `height: 360 px`. The literal is now expressed as `min_height` so
/// the panel keeps the visual floor at small viewports while the
/// `max_height: 92 %` ceiling scales with the viewport up to
/// ~1987 px at 3840×2160 (AC7).
pub fn draft_initial_modal_panel_node() -> Node {
    Node {
        display: Display::Flex,
        position_type: PositionType::Relative,
        width: Val::Percent(DRAFT_INITIAL_MODAL_WIDTH_PERCENT),
        max_width: Val::Px(DRAFT_INITIAL_MODAL_MAX_WIDTH_PX),
        min_height: Val::Px(DRAFT_INITIAL_MODAL_HEIGHT_PX),
        max_height: Val::Percent(DRAFT_INITIAL_MODAL_MAX_HEIGHT_PERCENT),
        overflow: Overflow::scroll_y(),
        border: UiRect::all(Val::Px(spacing::SPACING_XS / 2.0)),
        border_radius: BorderRadius::all(Val::Px(spacing::SPACING_SM)),
        padding: UiRect::all(Val::Px(DRAFT_INITIAL_MODAL_PADDING_PX)),
        ..default()
    }
}

/// PROMPT 1349 (Sprint 18 story 026 / Lane J) — draft-initial card-grid
/// container. The pre-1349 layout placed each of the nine slots
/// absolutely via per-index `left = column * (width + gap)` / `top = row *
/// (height + gap)` offsets (PROMPT 1180 §1.4 S-09). Story 026 §5 C-5
/// requires `Display::Grid` (or `FlexWrap::Wrap`) with absolute offsets
/// removed. The grid container itself keeps its `position_type:
/// Absolute` anchor inside the modal panel so the surrounding
/// absolutely-positioned overlays (objective banner, countdown, footer)
/// retain their existing positions; only slot placement is migrated
/// from manual offsets to Bevy 0.18 grid auto-placement.
pub fn draft_initial_grid_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(DRAFT_INITIAL_GRID_LEFT_PX),
        top: Val::Px(DRAFT_INITIAL_GRID_TOP_PX),
        width: Val::Px(DRAFT_INITIAL_GRID_WIDTH_PX),
        height: Val::Px(DRAFT_INITIAL_GRID_HEIGHT_PX),
        display: Display::Grid,
        grid_template_columns: RepeatedGridTrack::px(3, DRAFT_INITIAL_GRID_COLUMN_WIDTH_PX),
        grid_template_rows: RepeatedGridTrack::px(3, DRAFT_INITIAL_GRID_ROW_HEIGHT_PX),
        column_gap: Val::Px(DRAFT_INITIAL_GRID_COLUMN_GAP_PX),
        row_gap: Val::Px(DRAFT_INITIAL_GRID_ROW_GAP_PX),
        ..default()
    }
}

/// PROMPT 1349 — draft-initial card slot. Migrated off the pre-1349
/// per-index absolute offset (`position_type: Absolute`, `left = col *
/// (width + gap)`, `top = row * (height + gap)`) — story 026 §5 C-5
/// "absolute offsets removed". The slot is now placed by the grid
/// container's `Display::Grid` auto-placement.
pub fn draft_initial_slot_node() -> Node {
    Node {
        width: Val::Px(DRAFT_INITIAL_GRID_COLUMN_WIDTH_PX),
        height: Val::Px(DRAFT_INITIAL_GRID_ROW_HEIGHT_PX),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn overlay_text_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(22.0),
        top: Val::Px(18.0),
        ..default()
    }
}

fn draft_initial_modal_footer_node() -> Node {
    // PROMPT 1051 — footer anchored to modal-panel bottom, full width
    // minus the modal padding. Flex row with status text on the left
    // and the Ready CTA on the right so the keep decision reads as a
    // single grouped action band rather than two floating widgets.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(DRAFT_INITIAL_MODAL_PADDING_PX),
        right: Val::Px(DRAFT_INITIAL_MODAL_PADDING_PX),
        bottom: Val::Px(DRAFT_INITIAL_MODAL_PADDING_PX),
        height: Val::Px(DRAFT_INITIAL_MODAL_FOOTER_HEIGHT_PX),
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        padding: UiRect::horizontal(Val::Px(spacing::SPACING_MD)),
        column_gap: Val::Px(spacing::SPACING_MD),
        border: UiRect::top(Val::Px(1.0)),
        ..default()
    }
}

fn draft_initial_ready_button_node() -> Node {
    // PROMPT 1051 — flex child of the footer, pushed to the right end
    // by the footer's SpaceBetween distribution. No absolute positioning
    // so the button stays inside the footer band and reads as the
    // primary CTA of the keep-9 decision.
    Node {
        position_type: PositionType::Relative,
        width: Val::Px(132.0),
        height: Val::Px(36.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn draft_initial_status_node() -> Node {
    // PROMPT 1051 — flex child of the footer, sits on the left and
    // hosts the "Waiting for opponent..." copy when Ready is engaged.
    Node {
        position_type: PositionType::Relative,
        width: Val::Px(220.0),
        height: Val::Px(28.0),
        align_items: AlignItems::Center,
        ..default()
    }
}

fn draft_initial_hand_full_banner_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(spacing::SPACING_XL + spacing::SPACING_XL),
        top: Val::Px(150.0),
        width: Val::Px(260.0),
        height: Val::Px(30.0),
        ..default()
    }
}

fn draft_initial_countdown_node() -> Node {
    // PROMPT 1230 — top-right corner of the modal panel, sized so a
    // canonical "45s" / "9s" readout reads clearly without overlapping the
    // objective overlay (left=DRAFT_INITIAL_GRID_LEFT_PX, width=640 →
    // right edge at 704px; modal max-width=860px). The label is right-
    // aligned inside its box so the seconds value visually anchors to the
    // modal corner regardless of digit count.
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(DRAFT_INITIAL_MODAL_PADDING_PX),
        top: Val::Px(spacing::SPACING_MD),
        width: Val::Px(88.0),
        height: Val::Px(32.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexEnd,
        ..default()
    }
}

fn draft_initial_objective_overlay_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(DRAFT_INITIAL_GRID_LEFT_PX),
        top: Val::Px(spacing::SPACING_MD),
        width: Val::Px(640.0),
        height: Val::Px(32.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn draft_initial_objective_copy_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(10.0),
        top: Val::Px(7.0),
        width: Val::Px(500.0),
        height: Val::Px(18.0),
        ..default()
    }
}

fn draft_initial_objective_dismiss_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(8.0),
        top: Val::Px(6.0),
        width: Val::Px(88.0),
        height: Val::Px(20.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn draft_initial_objective_retrieval_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(spacing::SPACING_XL + spacing::SPACING_XL),
        top: Val::Px(spacing::SPACING_LG),
        width: Val::Px(116.0),
        height: Val::Px(28.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn shop_slot_node(index: usize) -> Node {
    // Sprint 16 story 009 (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`) Phase 1
    // migration: the slot's outer rectangle / border now flow from the
    // shared card-slot primitive so layout drift across hand / draft /
    // shop / auction surfaces cannot recur. Per-index horizontal
    // positioning remains a shop-panel concern and is overlaid on the
    // primitive's `position_type: Absolute` Node.
    let mut node = card_slot_node(CardSlotKind::ShopSlot);
    node.left = Val::Px(92.0 + index as f32 * 154.0);
    node.top = Val::Px(44.0);
    node
}

fn shop_slot_affordance_label_node() -> Node {
    // PROMPT 1085 — affordance copy band anchored to the bottom inside-edge
    // of the parent shop slot well. Caption-height row so the buy /
    // disabled-reason copy reads as a short tag below the card art.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(spacing::SPACING_XS),
        right: Val::Px(spacing::SPACING_XS),
        bottom: Val::Px(spacing::SPACING_XS / 2.0),
        height: Val::Px(typography::CAPTION * typography::LINE_HEIGHT_DEFAULT_RATIO),
        ..default()
    }
}

fn shop_refresh_button_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(92.0),
        top: Val::Px(148.0),
        width: Val::Px(148.0),
        height: Val::Px(36.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn shop_ready_button_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(96.0),
        top: Val::Px(148.0),
        width: Val::Px(132.0),
        height: Val::Px(36.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn shop_ready_status_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(96.0),
        top: Val::Px(190.0),
        width: Val::Px(180.0),
        height: Val::Px(28.0),
        ..default()
    }
}

fn shop_hand_full_banner_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(260.0),
        top: Val::Px(152.0),
        width: Val::Px(300.0),
        height: Val::Px(30.0),
        ..default()
    }
}

fn shop_phase_title_node() -> Node {
    // PROMPT 1042 — anchored to the top of the 260px shop_panel so the
    // word "SHOP" sits above the slot row at every captured viewport. Wide
    // enough to host an optional " — ROUND N" suffix without re-laying-out.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(spacing::SPACING_LG),
        top: Val::Px(spacing::SPACING_SM),
        width: Val::Px(280.0),
        height: Val::Px(28.0),
        ..default()
    }
}

fn shop_empty_state_node() -> Node {
    // PROMPT 1042 — centered above the row of shop_slot wells so the
    // message lands where the cards will eventually render. Width matches
    // the slot strip (3 × 136 + 2 × 18 = 444 px) so the text reads as the
    // placeholder for the offer row.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(92.0),
        top: Val::Px(70.0),
        width: Val::Px(444.0),
        height: Val::Px(28.0),
        ..default()
    }
}

/// Auction-panel `Node` builder. `pub` so the
/// `tests/integration/ui_clean_pass/play_area_budget_test.rs`
/// integration bin can assert the migrated Node shape (AC3 + AC7) at
/// the canonical 1280×720 / 1366×768 / 1920×1080 viewport matrix.
pub fn auction_panel_node() -> Node {
    // Sprint 18 story 020 (S18-UI-PLAY-AREA-CONTAINER-001) AC3: auction
    // panel parents into `PlayArea` (was `top: 80, bottom: 140` against
    // the viewport — the mis-tuned viewport-anchored literal overlapped
    // `HandBar` and `FooterBar` per PROMPT 1180 §2 RC-1 / overlap S-02).
    // The 50% × 50% featured-card centering anchor inside the panel now
    // resolves against `PlayArea`'s middle band, which is the strip-
    // budget-correct container shape.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(0.0),
        bottom: Val::Px(0.0),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    }
}

fn auction_featured_card_node() -> Node {
    // Sprint 14 story 016 AC1 + AC3: width × height each strictly larger
    // than `shop_slot_node` (136 × 78 px); center-of-panel anchor via the
    // canonical bevy_ui centering trick (left/top = 50% with a negative
    // margin = half the size). The auction panel inhabits the full
    // viewport width (`left: 0, right: 0`) and a fixed vertical band
    // (`top: 80, bottom: 140`), so the relative percent anchor resolves
    // to the panel's geometric center at every viewport in the canonical
    // matrix (`docs/ux/global-ui-design-spec.md` §8).
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(AUCTION_READABILITY_CARD_LEFT_PX),
        top: Val::Percent(50.0),
        margin: UiRect {
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(-AUCTION_FEATURED_CARD_HEIGHT_PX / 2.0),
            bottom: Val::Px(0.0),
        },
        width: Val::Px(AUCTION_FEATURED_CARD_WIDTH_PX),
        height: Val::Px(AUCTION_FEATURED_CARD_HEIGHT_PX),
        border: UiRect::all(Val::Px(AUCTION_FEATURED_CARD_FRAME_THICKNESS_PX)),
        padding: UiRect::all(Val::Px(spacing::SPACING_LG)),
        ..default()
    }
}

fn auction_featured_card_frame_node() -> Node {
    // Sprint 14 story 016 AC2: explicit visual frame overlay. Anchored
    // to the featured-card parent's full extent so the frame paints
    // exactly the perimeter of the card without occluding the inline
    // card content. Border thickness matches `auction_featured_card_node`
    // so the frame primitive stays visually flush even when story 018
    // (lead / loss state) recolors it.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(0.0),
        bottom: Val::Px(0.0),
        border: UiRect::all(Val::Px(AUCTION_FEATURED_CARD_FRAME_THICKNESS_PX)),
        ..default()
    }
}

fn auction_featured_card_stats_node() -> Node {
    // Sprint 14 story 016 AC4: stats readout sub-node. Anchored beneath
    // the name region inside the featured card; height is sized off the
    // H2 line-height-ratio so the typography hierarchy reads
    // unambiguously.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(spacing::SPACING_LG),
        right: Val::Px(spacing::SPACING_LG),
        bottom: Val::Px(
            spacing::SPACING_LG + typography::BODY * typography::LINE_HEIGHT_DEFAULT_RATIO,
        ),
        height: Val::Px(typography::H2 * typography::LINE_HEIGHT_DEFAULT_RATIO),
        ..default()
    }
}

fn auction_featured_card_keyword_node() -> Node {
    // Sprint 14 story 016 AC4: keyword readout sub-node. Anchored at the
    // card's bottom-inside edge per the read-order (name → stats →
    // keyword) per story 016 §Scope line 138-143.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(spacing::SPACING_LG),
        right: Val::Px(spacing::SPACING_LG),
        bottom: Val::Px(spacing::SPACING_SM),
        height: Val::Px(typography::BODY * typography::LINE_HEIGHT_DEFAULT_RATIO),
        ..default()
    }
}

fn auction_featured_card_price_label_node() -> Node {
    // PROMPT 1085 — prominent current-price line anchored to the top of
    // the featured card, opposite the name banner so the player can read
    // "Bid: Ng" without scanning down to the bid row. Width is the card
    // interior so longer prices wrap cleanly inside the frame.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(spacing::SPACING_LG),
        right: Val::Px(spacing::SPACING_LG),
        top: Val::Px(spacing::SPACING_SM),
        height: Val::Px(typography::H2 * typography::LINE_HEIGHT_DEFAULT_RATIO),
        ..default()
    }
}

fn auction_featured_card_timer_label_node() -> Node {
    // PROMPT 1085 — numeric time-left readout anchored just under the
    // price line. Caption-height row keeps the typography subordinate to
    // the price line above so the read order is `name → price → timer →
    // stats → keyword` per `docs/ux/global-ui-design-spec.md` §8.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(spacing::SPACING_LG),
        right: Val::Px(spacing::SPACING_LG),
        top: Val::Px(spacing::SPACING_SM + typography::H2 * typography::LINE_HEIGHT_DEFAULT_RATIO),
        height: Val::Px(typography::CAPTION * typography::LINE_HEIGHT_DEFAULT_RATIO),
        ..default()
    }
}

/// Sprint 14 story 016 — featured-card frame fill color. ACCENT token
/// `#F2C94C` per `docs/ux/global-ui-design-spec.md` §7. Friend-game
/// placeholder palette; `PAW-TD-*-a` accept-risk preserved.
pub fn auction_featured_card_accent_color() -> Color {
    Color::srgb(0.949, 0.788, 0.298)
}

/// Sprint 14 story 018 leading token, `SEMANTIC_SUCCESS #27AE60`.
pub fn auction_featured_card_leading_color() -> Color {
    Color::srgb(0.153, 0.682, 0.376)
}

/// Sprint 14 story 018 losing token, `SEMANTIC_ERROR #EB5757`.
pub fn auction_featured_card_losing_color() -> Color {
    Color::srgb(0.922, 0.341, 0.341)
}

pub fn auction_featured_card_lead_loss_color(state: AuctionFeaturedCardLeadLossState) -> Color {
    match state {
        AuctionFeaturedCardLeadLossState::Neutral => auction_featured_card_accent_color(),
        AuctionFeaturedCardLeadLossState::Leading => auction_featured_card_leading_color(),
        AuctionFeaturedCardLeadLossState::Losing => auction_featured_card_losing_color(),
    }
}

fn auction_status_text_node() -> Node {
    // Sprint 14 story 016: status text anchored near the panel top so it
    // sits clear of the panel-centered featured card at both the 1080p
    // and 768p viewports. Story 004 contract (status visibility &
    // content) is unchanged; only the absolute offset moves.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(AUCTION_READABILITY_INFO_LEFT_PX),
        top: Val::Px(spacing::SPACING_XL),
        width: Val::Px(AUCTION_READABILITY_INFO_WIDTH_PX),
        height: Val::Px(32.0),
        ..default()
    }
}

fn auction_timer_bar_node() -> Node {
    // Sprint 14 story 016: timer bar anchored near the panel top.
    // Story 004 contract (timer visibility / fill animation) is
    // unchanged; only the absolute offset moves.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(AUCTION_READABILITY_INFO_LEFT_PX),
        top: Val::Px(spacing::SPACING_XL + spacing::SPACING_XL + spacing::SPACING_MD),
        width: Val::Px(AUCTION_READABILITY_INFO_WIDTH_PX),
        height: Val::Px(10.0),
        ..default()
    }
}

fn auction_bid_status_text_node() -> Node {
    // Sprint 14 story 016: bid-status text anchored to the panel bottom
    // so it sits clear of the featured card (panel-centered, 380×280).
    // Story 005 + 006 contracts (status text visibility) are unchanged;
    // only the absolute offset moves.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(AUCTION_READABILITY_INFO_LEFT_PX),
        bottom: Val::Px(24.0),
        width: Val::Px(AUCTION_READABILITY_INFO_WIDTH_PX),
        height: Val::Px(40.0),
        ..default()
    }
}

fn auction_free_gold_counter_group_node() -> Node {
    // Sprint 14 story 017: a single shared container adjacent to the
    // bid cluster. It anchors from the final +5 bid-button x-position,
    // then adds one spacing-token gap so the counters read as part of
    // the decision cluster without overlapping button targets.
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(AUCTION_FREE_GOLD_COUNTER_LEFT_GAP_PX),
        position_type: PositionType::Absolute,
        left: Val::Percent(AUCTION_FREE_GOLD_COUNTER_ANCHOR_LEFT_PERCENT),
        bottom: Val::Px(AUCTION_FREE_GOLD_COUNTER_BOTTOM_PX),
        margin: UiRect {
            left: Val::Px(AUCTION_FREE_GOLD_COUNTER_LEFT_OFFSET_PX),
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
        },
        width: Val::Px(AUCTION_FREE_GOLD_COUNTER_GROUP_WIDTH_PX),
        height: Val::Px(AUCTION_FREE_GOLD_COUNTER_GROUP_HEIGHT_PX),
        padding: UiRect::all(Val::Px(AUCTION_FREE_GOLD_COUNTER_PADDING_PX)),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn auction_free_gold_counter_node() -> Node {
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexStart,
        width: Val::Px(AUCTION_FREE_GOLD_COUNTER_WIDTH_PX),
        height: Val::Percent(100.0),
        row_gap: Val::Px(0.0),
        ..default()
    }
}

fn auction_free_gold_counter_label_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Px(
            AUCTION_FREE_GOLD_COUNTER_LABEL_FONT_PX * typography::LINE_HEIGHT_DEFAULT_RATIO,
        ),
        ..default()
    }
}

fn auction_free_gold_counter_value_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Px(
            AUCTION_FREE_GOLD_COUNTER_VALUE_FONT_PX * typography::LINE_HEIGHT_DEFAULT_RATIO,
        ),
        ..default()
    }
}

fn auction_bid_button_node(index: usize) -> Node {
    // Sprint 14 story 016: bid buttons anchored to the panel bottom so
    // they read alongside (not on top of) the panel-centered featured
    // card. Bid target 44 × 44 CSS px (story 011) and focus-ring width
    // (story 011) are unchanged; only the absolute offset moves.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(
            AUCTION_READABILITY_INFO_LEFT_PX
                + index as f32 * (AUCTION_BID_TARGET_WIDTH_PX + AUCTION_READABILITY_CONTROL_GAP_PX),
        ),
        bottom: Val::Px(72.0),
        width: Val::Px(AUCTION_BID_TARGET_WIDTH_PX),
        height: Val::Px(AUCTION_BID_TARGET_HEIGHT_PX),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn auction_pass_button_node() -> Node {
    // PROMPT 1042 — Pass affordance anchored after the 3rd bid button at
    // index 3 (`34% + 3 × 9% = 61%`) so the auction decision cluster reads
    // [Bid +1] [Bid +3] [Bid +5] [Pass] in left-to-right scan order.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(
            AUCTION_READABILITY_INFO_LEFT_PX
                + 3.0 * (AUCTION_BID_TARGET_WIDTH_PX + AUCTION_READABILITY_CONTROL_GAP_PX),
        ),
        bottom: Val::Px(72.0),
        width: Val::Px(AUCTION_BID_TARGET_WIDTH_PX),
        height: Val::Px(AUCTION_BID_TARGET_HEIGHT_PX),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

/// Shop-footer `Node` builder. `pub` so the
/// `tests/integration/ui_clean_pass/play_area_budget_test.rs`
/// integration bin can assert the migrated Node shape (AC4 + AC7) at
/// the canonical 1280×720 / 1366×768 / 1920×1080 viewport matrix.
pub fn footer_node() -> Node {
    // Sprint 18 story 020 (S18-UI-PLAY-AREA-CONTAINER-001) AC4: shop
    // footer parents into `PlayArea` (was `bottom: 100, height: 96`
    // against the viewport — the viewport-anchored `bottom: 100`
    // literal was a hand-computed offset above the strip column and
    // overlapped the bottom strips per PROMPT 1180 §2 RC-1 / overlap
    // S-06). The footer now sits flush at `PlayArea`'s bottom edge so
    // the 96 px footer band stays inside the middle-band budget and the
    // four shop-footer slots remain horizontally aligned with the shop
    // panel above.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        bottom: Val::Px(0.0),
        height: Val::Px(96.0),
        ..default()
    }
}

fn shop_footer_slot_node(index: usize) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(92.0 + index as f32 * 154.0),
        top: Val::Px(16.0),
        width: Val::Px(136.0),
        height: Val::Px(64.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

/// Shop-toast `Node` builder. `pub` so the
/// `tests/integration/ui_clean_pass/play_area_budget_test.rs`
/// integration bin can assert the migrated Node shape (AC5 + AC7) at
/// the canonical 1280×720 / 1366×768 / 1920×1080 viewport matrix.
pub fn toast_node() -> Node {
    // Sprint 18 story 020 (S18-UI-PLAY-AREA-CONTAINER-001) AC5: toast
    // parents into `PlayArea` (was `bottom: 220` against the viewport —
    // the literal `HAND_BAR_HEIGHT_PX + FOOTER_BAR_HEIGHT_PX = 220`
    // offset above the bottom strip column is structurally what
    // `PlayArea` now provides). AC5 explicitly allows the toast to stay
    // `position_type: Absolute` within `PlayArea`; the toast sits flush
    // at `PlayArea`'s bottom-right corner with a small inset margin so
    // it does not clip the shop / auction panels' affordances.
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(24.0),
        bottom: Val::Px(0.0),
        width: Val::Px(260.0),
        height: Val::Px(48.0),
        ..default()
    }
}

fn auction_toast_text_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(10.0),
        right: Val::Px(10.0),
        top: Val::Px(8.0),
        bottom: Val::Px(8.0),
        ..default()
    }
}

fn overlay_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(80.0),
        bottom: Val::Px(140.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn settlement_overlay_text_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(34.0),
        top: Val::Px(140.0),
        width: Val::Px(420.0),
        height: Val::Px(48.0),
        ..default()
    }
}

fn panel_label_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        top: Val::Px(0.0),
        ..default()
    }
}

fn shop_auction_text_font(font_size: f32) -> TextFont {
    TextFont {
        font_size,
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

/// PROMPT 1085 — format the numeric time-left readout displayed on the
/// child [`AuctionFeaturedCardTimerLabel`]. The label mirrors the timer
/// bar's runtime state in copy: live countdown shows `{N}s left`, the
/// preparing window shows `Auction starting...`, settling shows
/// `Auction ending`, and the connection-error fallback surfaces a single
/// reassuring line so the player is not left without context.
pub fn auction_featured_timer_label(state: &ShopAuctionAuctionState) -> String {
    match state.panel_state {
        ShopAuctionAuctionPanelState::Hidden => String::new(),
        ShopAuctionAuctionPanelState::Preparing => "Auction starting...".to_string(),
        ShopAuctionAuctionPanelState::Settling => "Auction ending".to_string(),
        ShopAuctionAuctionPanelState::ConnectionError => "Awaiting server...".to_string(),
        ShopAuctionAuctionPanelState::Active => {
            if state.timer_remaining_ms == 0 {
                return "Auction ending".to_string();
            }
            let seconds = state.timer_remaining_ms.div_ceil(1_000);
            format!("{seconds}s left")
        }
    }
}

/// PROMPT 1085 — compute the human-readable affordance copy displayed on
/// the child [`ShopSlotAffordanceLabel`] beneath a shop tile. Empty string
/// means the parent slot is in a state that should not advertise a buy
/// affordance (`Empty` wells in particular). Audit AUDIT-1076-13 noted that
/// the player saw no disabled-reason feedback when clicking unaffordable
/// slots; this helper is the single source of that copy so every state
/// renders a deterministic, non-empty string for the visible cases.
pub fn shop_slot_affordance_copy(
    state: ShopSlotState,
    card_id: Option<CardId>,
    cost: Option<u32>,
    hand_size: usize,
    economy: &PlayerEconomyView,
) -> String {
    match state {
        ShopSlotState::Empty => String::new(),
        ShopSlotState::PendingPurchase => "PENDING...".to_string(),
        ShopSlotState::Refreshing => "REFRESHING...".to_string(),
        ShopSlotState::HandFullLocked => "LOCKED · Hand full".to_string(),
        ShopSlotState::Available => {
            if card_id.is_none() {
                return String::new();
            }
            if hand_size >= 10 {
                return "LOCKED · Hand full".to_string();
            }
            let cost = cost.unwrap_or(0);
            if !economy.initialized || economy.gold < cost {
                return format!("LOCKED · Need {cost}g");
            }
            format!("BUY · {cost}g")
        }
    }
}

fn auction_active(mode: &ShopAuctionUiMode, state: &ShopAuctionAuctionState) -> bool {
    *mode == ShopAuctionUiMode::Auction
        && state.panel_state == ShopAuctionAuctionPanelState::Active
        && state.card_id.is_some()
}

fn local_player_is_leading(
    state: &ShopAuctionAuctionState,
    local_gold: &ShopAuctionLocalGoldView,
) -> bool {
    state.current_leader.is_some() && state.current_leader == local_gold.player_id
}

fn auction_local_player_id(
    local_gold: &ShopAuctionLocalGoldView,
    player_ids: Option<&HudPlayerIds>,
) -> Option<PlayerId> {
    local_gold
        .player_id
        .or_else(|| player_ids.map(|ids| ids.local_id))
}

pub fn auction_featured_card_lead_loss_state(
    state: &ShopAuctionAuctionState,
    local_gold: &ShopAuctionLocalGoldView,
) -> AuctionFeaturedCardLeadLossState {
    match state.current_leader {
        Some(leader) if Some(leader) == local_gold.player_id => {
            AuctionFeaturedCardLeadLossState::Leading
        }
        Some(_) => AuctionFeaturedCardLeadLossState::Losing,
        None => AuctionFeaturedCardLeadLossState::Neutral,
    }
}

pub fn auction_bid_amount(current_price: u32, increment: u32) -> u32 {
    current_price.saturating_add(increment)
}

pub fn auction_bid_button_state(
    amount: u32,
    local_free_gold: u32,
    has_gold_source: bool,
    hand_full: bool,
    local_leading: bool,
    waiting_for_local_gold_after_opponent_bid: bool,
    auction_state: &ShopAuctionAuctionState,
) -> AuctionBidButtonState {
    if auction_state.panel_state != ShopAuctionAuctionPanelState::Active {
        return AuctionBidButtonState::GenericDisabled;
    }

    if local_leading {
        return AuctionBidButtonState::HiddenLeading;
    }

    if let Some(in_flight_amount) = auction_state.in_flight_bid_amount {
        return if in_flight_amount == amount {
            AuctionBidButtonState::InFlight
        } else {
            AuctionBidButtonState::GenericDisabled
        };
    }

    if waiting_for_local_gold_after_opponent_bid {
        return AuctionBidButtonState::GenericDisabled;
    }

    if auction_state.locally_expired() {
        return AuctionBidButtonState::LocallyExpired;
    }

    if hand_full {
        return AuctionBidButtonState::HandFullLocked;
    }

    if !has_gold_source || local_free_gold < amount {
        return AuctionBidButtonState::Unaffordable;
    }

    AuctionBidButtonState::Enabled
}

fn auction_bid_state_focusable(state: AuctionBidButtonState) -> bool {
    state == AuctionBidButtonState::Enabled
}

fn bid_button_focusable(
    button: Entity,
    focus_states: &Query<&AuctionBidFocusState, With<AuctionBidButton>>,
) -> bool {
    focus_states
        .get(button)
        .is_ok_and(|focus_state| focus_state.focusable)
}

fn next_focusable_bid_button(
    buttons: &[Entity; 3],
    start_index: usize,
    focus_states: &Query<&AuctionBidFocusState, With<AuctionBidButton>>,
) -> Option<Entity> {
    for offset in 0..buttons.len() {
        let index = (start_index + offset) % buttons.len();
        let button = buttons[index];
        if bid_button_focusable(button, focus_states) {
            return Some(button);
        }
    }

    None
}

/// Maps an [`AuctionBidButtonState`] to the corresponding
/// [`BidButtonChromeState`] for asset selection. `Enabled` maps to
/// `Normal`; `HiddenLeading` maps to `None` (no chrome image — the row
/// is `Visibility::Hidden` while the local player leads, so the baked-
/// `?` `Disabled` PNG must not be loaded onto the entity); every other
/// state maps to `Disabled`.
///
/// PROMPT 1116 — added the `HiddenLeading => None` branch so the
/// `ui_bid_button_disabled.png` chrome (which carries a baked-`?`
/// glyph per `PAW-TD-*-a` accept-risk) is no longer kept on the
/// entity's `ImageNode` during the local-leading window. Callers fall
/// back to `Handle<Image>::default()` when this returns `None`. The
/// `Normal` and `Disabled` mappings for every other variant are
/// unchanged.
fn auction_bid_chrome_state(state: AuctionBidButtonState) -> Option<BidButtonChromeState> {
    match state {
        AuctionBidButtonState::HiddenLeading => None,
        AuctionBidButtonState::Enabled => Some(BidButtonChromeState::Normal),
        _ => Some(BidButtonChromeState::Disabled),
    }
}

fn auction_bid_text_color(state: AuctionBidButtonState) -> Color {
    match state {
        AuctionBidButtonState::Enabled => Color::srgb(0.98, 0.93, 0.72),
        AuctionBidButtonState::InFlight => Color::srgba(0.98, 0.93, 0.72, 0.80),
        _ => Color::srgba(0.92, 0.94, 0.96, 0.30),
    }
}

fn auction_bid_border_color(state: AuctionBidButtonState, focused: bool) -> Color {
    if focused {
        return Color::srgb(1.0, 1.0, 1.0);
    }

    match state {
        AuctionBidButtonState::Enabled => Color::srgb(0.98, 0.73, 0.30),
        AuctionBidButtonState::InFlight => Color::srgb(0.98, 0.88, 0.40),
        AuctionBidButtonState::Unaffordable
        | AuctionBidButtonState::HandFullLocked
        | AuctionBidButtonState::GenericDisabled
        | AuctionBidButtonState::LocallyExpired => Color::srgba(0.92, 0.94, 0.96, 0.35),
        AuctionBidButtonState::HiddenLeading => Color::srgba(0.92, 0.94, 0.96, 0.0),
    }
}

fn auction_bid_background_color(state: AuctionBidButtonState) -> Color {
    match state {
        AuctionBidButtonState::Enabled => Color::srgba(0.22, 0.18, 0.11, 0.90),
        AuctionBidButtonState::InFlight => Color::srgba(0.34, 0.28, 0.12, 0.92),
        AuctionBidButtonState::HiddenLeading => Color::srgba(0.0, 0.0, 0.0, 0.0),
        _ => Color::srgba(0.12, 0.14, 0.18, 0.55),
    }
}

fn auction_expired_status_text(locally_expired_elapsed_ms: u32) -> &'static str {
    if locally_expired_elapsed_ms >= AUCTION_AWAITING_SERVER_DELAY_MS {
        "Awaiting server..."
    } else {
        "Auction ending..."
    }
}

pub fn rejection_toast_text(reason: BidRejectedReason, current_price: u32) -> String {
    match reason {
        BidRejectedReason::InsufficientGold => "Not enough gold".to_string(),
        BidRejectedReason::AmountTooLow => {
            format!("Bid must be at least {}g", current_price.saturating_add(1))
        }
        BidRejectedReason::AlreadyLeader => "You are already leading".to_string(),
        BidRejectedReason::HandFull => "Hand full — no bids possible this auction".to_string(),
        BidRejectedReason::AuctionExpired => "Auction has ended".to_string(),
    }
}

fn auction_toast_total_ms() -> u32 {
    AUCTION_TOAST_FADE_IN_MS
        .saturating_add(AUCTION_TOAST_HOLD_MS)
        .saturating_add(AUCTION_TOAST_FADE_OUT_MS)
}

fn draft_initial_active(mode: &ShopAuctionUiMode, state: &ShopAuctionDraftInitialState) -> bool {
    *mode == ShopAuctionUiMode::DraftOffering && state.offering_loaded
}

/// PROMPT 1042 — return true whenever the client is in DraftShop mode,
/// regardless of whether shop slots have arrived. The shop surface now
/// renders explicit chrome + an empty-state copy while waiting for
/// `S2CShopSlots`, so the player never sees a blank panel that looks like
/// Placement (PROMPT 1034 finding F-shop / §2.4).
fn shop_active(mode: &ShopAuctionUiMode, _state: &ShopAuctionShopState) -> bool {
    *mode == ShopAuctionUiMode::Shop
}

fn should_buffer_shop_slots(phase: RoundPhase) -> bool {
    matches!(
        phase,
        RoundPhase::DraftInitial
            | RoundPhase::DraftAuction
            | RoundPhase::Placement
            | RoundPhase::Resolution
    )
}

fn normalized_shop_slots(
    slots: &[Option<CardId>],
) -> [Option<CardId>; SHOP_AUCTION_UI_SHOP_SLOT_COUNT] {
    std::array::from_fn(|index| slots.get(index).copied().flatten())
}

fn auction_timer_width_percent(remaining_ms: u32, duration_ms: u32) -> f32 {
    if duration_ms == 0 {
        return 0.0;
    }

    (remaining_ms.min(duration_ms) as f32 / duration_ms as f32) * 100.0
}

fn rarity_sort_rank(rarity: Rarity) -> u8 {
    match rarity {
        Rarity::Common => 0,
        Rarity::Uncommon => 1,
        Rarity::Rare => 2,
        Rarity::Epic => 3,
        Rarity::Legendary => 4,
    }
}

/// PROMPT 1029 — formats the combat-stat readout shown on draft / shop / auction
/// card tiles. Returns `"ATK/HP"` for `Minion` / `Structure` cards (where the
/// numeric stat pair is gameplay-relevant) and an empty string for spells /
/// traps / fields / orders / double-face (where ATK/HP carry no meaning per
/// `shared/src/card.rs` `CardType` doc comment). Tiles use this to extend their
/// existing `name + cost` label without disturbing layout for non-minion cards.
pub fn format_card_combat_stats(card: &CardData) -> String {
    match card.card_type {
        CardType::Minion | CardType::Structure => format!("{}/{}", card.atk, card.hp),
        CardType::Spell
        | CardType::Trap
        | CardType::Field
        | CardType::Order
        | CardType::DoubleFace => String::new(),
    }
}

fn clear_draft_initial_slot(
    commands: &mut Commands,
    entity: Entity,
    text_entity: Entity,
    text_query: &mut Query<&mut Text, With<DraftInitialSlotTextLabel>>,
    visibility: &mut Visibility,
) {
    if let Ok(mut text) = text_query.get_mut(text_entity) {
        text.0.clear();
    }
    *visibility = Visibility::Hidden;
    commands.entity(entity).remove::<(
        DraftInitialSlotCard,
        DraftInitialSlotCardName,
        DraftInitialSlotGoldCost,
        DraftInitialSlotRarity,
        DraftInitialSlotState,
        PendingDraftInitialPurchase,
    )>();
    clear_card_display_art(commands, entity);
}

fn apply_shop_slot(
    commands: &mut Commands,
    entity: Entity,
    card_id: Option<CardId>,
    catalog: &CardCatalog,
    asset_server: Option<&AssetServer>,
    text: &mut Text,
    visibility: &mut Visibility,
) {
    *visibility = Visibility::Visible;

    let Some(card_id) = card_id else {
        clear_shop_slot(commands, entity, text);
        return;
    };

    let card = catalog.get(&card_id);
    let card_name = card
        .map(|card| card.name_en.clone())
        .unwrap_or_else(|| format!("Card {}", card_id.0));
    let cost = card.map_or(0, |card| card.cost);
    let rarity = card.map_or(Rarity::Common, |card| card.rarity);
    // PROMPT 1029: append ATK/HP on the shop slot label so the player can
    // compare stats against the 3 offered cards without hovering.
    let stats = card.map(format_card_combat_stats).unwrap_or_default();

    text.0.clear();
    if stats.is_empty() {
        text.0
            .push_str(&format!("{}\n{:?} · {}g", card_name.as_str(), rarity, cost));
    } else {
        text.0.push_str(&format!(
            "{}\n{:?} · {}g · {}",
            card_name.as_str(),
            rarity,
            cost,
            stats
        ));
    }
    commands.entity(entity).insert((
        ShopSlotCard(card_id),
        ShopSlotCardName(card_name),
        ShopSlotGoldCost(cost),
        ShopSlotRarity(rarity),
        ShopSlotState::Available,
    ));
    apply_card_display_art(commands, entity, card, asset_server);
    commands.entity(entity).remove::<PendingShopPurchase>();
}

fn clear_shop_slot(commands: &mut Commands, entity: Entity, text: &mut Text) {
    text.0.clear();
    text.0.push_str("Empty");
    commands.entity(entity).insert(ShopSlotState::Empty);
    commands.entity(entity).remove::<(
        ShopSlotCard,
        ShopSlotCardName,
        ShopSlotGoldCost,
        ShopSlotRarity,
        PendingShopPurchase,
    )>();
    clear_card_display_art(commands, entity);
}

fn apply_shop_footer_slot(
    commands: &mut Commands,
    entity: Entity,
    card_id: Option<CardId>,
    catalog: &CardCatalog,
    asset_server: Option<&AssetServer>,
    text: &mut Text,
) {
    let Some(card_id) = card_id else {
        text.0.clear();
        text.0.push_str("Locked empty");
        commands
            .entity(entity)
            .insert(ShopFooterSlotState::EmptyLocked);
        commands.entity(entity).remove::<ShopFooterSlotCard>();
        clear_card_display_art(commands, entity);
        return;
    };

    let card = catalog.get(&card_id);
    let card_name = card
        .map(|card| card.name_en.clone())
        .unwrap_or_else(|| format!("Card {}", card_id.0));
    let cost = card.map_or(0, |card| card.cost);

    text.0.clear();
    text.0
        .push_str(&format!("{}\n{}g - locked", card_name.as_str(), cost));
    commands
        .entity(entity)
        .insert((ShopFooterSlotCard(card_id), ShopFooterSlotState::Locked));
    apply_card_display_art(commands, entity, card, asset_server);
}

fn mark_confirmed_purchase(
    card_id: CardId,
    commands: &mut Commands,
    slots: &mut Query<(
        Entity,
        &DraftInitialSlotIndex,
        &DraftInitialSlotCard,
        &mut DraftInitialSlotState,
    )>,
) -> Option<(Entity, u8)> {
    for (entity, slot_index, slot_card, mut slot_state) in slots.iter_mut() {
        if slot_card.0 != card_id || *slot_state == DraftInitialSlotState::Purchased {
            continue;
        }

        *slot_state = DraftInitialSlotState::Purchased;
        commands
            .entity(entity)
            .remove::<PendingDraftInitialPurchase>();
        return Some((entity, slot_index.0));
    }

    None
}

fn mark_confirmed_shop_purchase(
    card_id: CardId,
    commands: &mut Commands,
    slots: &mut Query<(Entity, &ShopSlotCard, &mut ShopSlotState, &mut Text)>,
) -> bool {
    for (entity, slot_card, mut slot_state, mut text) in slots.iter_mut() {
        if slot_card.0 != card_id || *slot_state == ShopSlotState::Empty {
            continue;
        }

        *slot_state = ShopSlotState::Empty;
        clear_shop_slot(commands, entity, &mut text);
        return true;
    }

    false
}

fn set_bought_overlay_visibility(
    slot_index: u8,
    target_visibility: Visibility,
    overlays: &mut Query<(&DraftInitialBoughtOverlay, &mut Visibility)>,
) {
    for (overlay, mut visibility) in overlays.iter_mut() {
        if overlay.slot_index == slot_index {
            *visibility = target_visibility;
            return;
        }
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

// =========================================================================
// PROMPT 1347 — S18-AUCTION-WON-CARD-DISPOSITION-001
//
// Disposition contract: auction-won card → winner hand on settle → manual
// placement during the auction-followup PLACEMENT phase → persists if not
// staged. The wire-level contract is unchanged (server is the only writer
// of `PlayerHands` and `S2CCardAcquired { source: AuctionWon }`); the
// presentation layer below adds discoverability + observability per
// AUDIT-1131-02 (Lane B1 + B2 + Lane D3).
// =========================================================================

/// PROMPT 1347 — AC4 affordance banner copy. The implementing worker chose
/// a stable English string for the banner: "Auction won — place your
/// new card!" The copy avoids naming a specific card (the card name lives
/// on the hand fan via the existing card display art surface; the banner
/// is the discoverability prompt). Localisable: callers can swap this
/// const for a token lookup once the localisation pipeline lands; the
/// AC4 contract names "or equivalent localizable token" explicitly.
pub const AUCTION_WON_AFFORDANCE_TEXT: &str = "Auction won — place your new card!";

fn auction_won_affordance_root_node() -> Node {
    // PROMPT 1347 / AC4 — top-anchored banner that sits above the hand
    // bar so the winner sees it without competing for attention with the
    // shop / auction panels (both hidden during PLACEMENT). The 1366×768
    // minimum-supported viewport puts this band well clear of both the
    // hand fan strip (260px from bottom) and the HUD header.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(28.0),
        right: Val::Percent(28.0),
        top: Val::Px(96.0),
        height: Val::Px(56.0),
        padding: UiRect::all(Val::Px(12.0)),
        border: UiRect::all(Val::Px(2.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

fn auction_won_affordance_text_node() -> Node {
    Node {
        position_type: PositionType::Relative,
        ..default()
    }
}

fn auction_won_hand_marker_node() -> Node {
    // PROMPT 1347 / AC5 — overlay child sized to the parent fan slot's
    // intrinsic bounds. The amber border + ambient glow signal "newly
    // acquired" without changing the underlying chrome layout — the
    // existing drag-state and idle-playable overlays continue to layer
    // on top.
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(-4.0),
        right: Val::Px(-4.0),
        top: Val::Px(-4.0),
        bottom: Val::Px(-4.0),
        border: UiRect::all(Val::Px(3.0)),
        ..default()
    }
}

/// PROMPT 1347 / AC4 / AC5 / AC9 / AC11 — single-source-of-truth update
/// for the auction-won disposition state. Runs every frame in the
/// `StateSync` system set so banner / marker / snapshot derivations are
/// consistent within a frame.
///
/// State machine:
/// - `None` (Idle): no auction has been won, or the previous one was
///   cleared. Stays here until [`handle_auction_settled_system`] arms a
///   fresh `Some(_)` for the LocalWinner outcome.
/// - `Some(_)`: armed. Each frame:
///   - On phase change to `Resolution` / `GameOver` / `Lobby` /
///     `Handshaking`: clear to `None`.
///   - Else, recompute `staged_yet` from `PendingPlacements` and
///     `submitted_yet` from `PlacementTimer.submitted` (only if previously
///     staged). On `submitted_yet`, clear to `None` (AC11: block becomes
///     absent on submit).
///
/// AC9 binding: once cleared, the marker does NOT re-arm even if the same
/// card is re-staged in a future PLACEMENT phase, because `arm()` is only
/// called from `handle_auction_settled_system` on a fresh
/// `S2CAuctionSettled { winner: Some(local), .. }` (one-shot per settle).
pub fn update_auction_won_pending_system(
    current: Res<CurrentClientPhase>,
    pending_placements: Option<Res<PendingPlacements>>,
    placement_timer: Option<Res<PlacementTimer>>,
    mut auction_won_pending: ResMut<AuctionWonPending>,
) {
    let Some(state) = auction_won_pending.state else {
        return;
    };

    // AC9 + AC11 first half: clear on phase exit. The auction-followup
    // PLACEMENT is the only PLACEMENT scoped to this state (since arm()
    // happens during DRAFT_AUCTION → PLACEMENT). Any later PLACEMENT in
    // the same session re-uses the same `AuctionWonPending` only if a
    // fresh auction is won (arm() is the only writer of `Some`).
    match current.phase {
        RoundPhase::Resolution
        | RoundPhase::GameOver
        | RoundPhase::Lobby
        | RoundPhase::Handshaking => {
            tracing::info!(
                target: "client::ui::shop_auction",
                phase = ?current.phase,
                card_id = ?state.card_id,
                "auction_won_pending: clearing on phase exit (AC9 / AC11)"
            );
            auction_won_pending.clear();
            return;
        }
        RoundPhase::DraftInitial
        | RoundPhase::DraftShop
        | RoundPhase::DraftAuction
        | RoundPhase::Placement => {}
    }

    // AC4 / AC5: staged_yet derived from PendingPlacements. The won card
    // is uniquely identified by `card_id`; PendingPlacements may stage
    // other cards in parallel — only the won card's stage affects this
    // flag.
    let pending_has_won_card = pending_placements
        .as_deref()
        .map(|p| {
            p.placements
                .iter()
                .any(|placement| placement.card_id == state.card_id)
        })
        .unwrap_or(false);

    // AC11 second half: submitted_yet derived from PlacementTimer once
    // the won card has been staged. Without the stage gate a different
    // submit (e.g. submitting an empty batch) could incorrectly clear
    // this state.
    let timer_submitted = placement_timer
        .as_deref()
        .map(|t| t.submitted)
        .unwrap_or(false);
    let submitted_yet = state.submitted_yet || (state.staged_yet && timer_submitted);

    let new_state = AuctionWonPendingState {
        card_id: state.card_id,
        settle_round: state.settle_round,
        staged_yet: state.staged_yet || pending_has_won_card,
        submitted_yet,
    };

    if submitted_yet {
        // AC11: clear immediately on submit so the snapshot block becomes
        // absent on the same frame. AC4 / AC5 are already cleared via
        // `staged_yet` rendering rule one frame earlier.
        tracing::info!(
            target: "client::ui::shop_auction",
            card_id = ?state.card_id,
            "auction_won_pending: clearing on successful submit (AC11)"
        );
        auction_won_pending.clear();
        return;
    }

    if new_state != state {
        auction_won_pending.state = Some(new_state);
    }
}

/// PROMPT 1347 / AC4 / AC9 — spawn-or-despawn the "Auction won" affordance
/// banner. One-shot per pending state: at most one banner exists at any
/// time. The banner is a top-level UI entity (no ChildOf) so it survives
/// the per-phase visibility flips on `ShopAuctionUiRoot`.
pub fn sync_auction_won_affordance_system(
    current: Res<CurrentClientPhase>,
    auction_won_pending: Res<AuctionWonPending>,
    catalog: Res<ShopAuctionCardCatalog>,
    mut commands: Commands,
    banners: Query<Entity, With<AuctionWonAffordanceBanner>>,
    mut texts: Query<&mut Text, With<AuctionWonAffordanceText>>,
) {
    let should_be_visible = auction_won_pending.affordance_visible(current.phase);
    let banner_count = banners.iter().count();

    if should_be_visible && banner_count == 0 {
        let card_name = auction_won_pending
            .card_id()
            .and_then(|card_id| catalog.cards.get(&card_id).map(|c| c.name_en.clone()));
        let text = match card_name {
            Some(name) => format!("Auction won — place {name}!"),
            None => AUCTION_WON_AFFORDANCE_TEXT.to_string(),
        };
        let banner = commands
            .spawn((
                Name::new("Shop Auction — Auction Won Affordance Banner"),
                AuctionWonAffordanceBanner,
                auction_won_affordance_root_node(),
                BackgroundColor(Color::srgba(0.10, 0.07, 0.04, 0.92)),
                BorderColor::all(Color::srgb(0.95, 0.66, 0.18)),
                Visibility::Visible,
                z_layers::UI_OVERLAY,
            ))
            .id();
        commands.spawn((
            Name::new("Shop Auction — Auction Won Affordance Text"),
            AuctionWonAffordanceText,
            Text::new(text),
            shop_auction_text_font(typography::H3),
            TextColor(Color::srgb(0.98, 0.94, 0.78)),
            TextLayout::new_with_justify(Justify::Center),
            auction_won_affordance_text_node(),
            Visibility::Inherited,
            ChildOf(banner),
        ));
        tracing::info!(
            target: "client::ui::shop_auction",
            card_id = ?auction_won_pending.card_id(),
            "auction_won_affordance: banner spawned (AC4)"
        );
        return;
    }

    if !should_be_visible && banner_count > 0 {
        for entity in &banners {
            commands.entity(entity).despawn();
        }
        tracing::info!(
            target: "client::ui::shop_auction",
            card_id = ?auction_won_pending.card_id(),
            phase = ?current.phase,
            "auction_won_affordance: banner despawned (AC4 / AC9)"
        );
        return;
    }

    // Banner already exists and should remain — refresh the text in case
    // the catalog populated between spawn and this frame.
    if should_be_visible && banner_count > 0 {
        if let Some(card_id) = auction_won_pending.card_id() {
            if let Some(card) = catalog.cards.get(&card_id) {
                let new_text = format!("Auction won — place {}!", card.name_en);
                for mut text in &mut texts {
                    if text.0 != new_text {
                        text.0 = new_text.clone();
                    }
                }
            }
        }
    }
}

/// PROMPT 1347 / AC5 / AC9 — spawn-or-despawn the newly-acquired marker
/// child on the hand fan slot whose `HandSlotCard` matches the pending
/// won card. The marker is a child of the fan slot so it inherits the
/// slot's transform and visibility — when the slot is hidden by the hand
/// layout (e.g. mode transitions), the marker is invisible without
/// requiring its own visibility flip.
pub fn sync_auction_won_hand_marker_system(
    current: Res<CurrentClientPhase>,
    auction_won_pending: Res<AuctionWonPending>,
    mut commands: Commands,
    fan_slots: Query<(Entity, &HandSlotCard), With<FanSlotIndex>>,
    existing_markers: Query<(Entity, &ChildOf), With<AuctionWonHandMarker>>,
) {
    let should_be_visible = auction_won_pending.affordance_visible(current.phase);
    let target_card_id = auction_won_pending.card_id();

    // Determine which fan slot (if any) should host the marker this frame.
    let target_slot = if should_be_visible {
        target_card_id.and_then(|card_id| {
            fan_slots
                .iter()
                .find(|(_, slot_card)| slot_card.0 == card_id)
                .map(|(entity, _)| entity)
        })
    } else {
        None
    };

    // Remove markers that no longer belong on their parent slot. Tolerate
    // the case where a marker's parent slot got despawned by the hand
    // layout (Bevy auto-despawns the child).
    for (entity, child_of) in &existing_markers {
        if Some(child_of.parent()) != target_slot {
            commands.entity(entity).despawn();
        }
    }

    // Spawn the marker on the target slot if one is needed and none
    // exists yet.
    let already_attached = existing_markers
        .iter()
        .any(|(_, child_of)| Some(child_of.parent()) == target_slot);
    if let Some(slot) = target_slot {
        if !already_attached {
            commands.spawn((
                Name::new("Hand Fan Slot — Auction Won Newly-Acquired Marker"),
                AuctionWonHandMarker,
                auction_won_hand_marker_node(),
                BackgroundColor(Color::srgba(0.95, 0.66, 0.18, 0.08)),
                BorderColor::all(Color::srgb(0.98, 0.78, 0.30)),
                Visibility::Inherited,
                ChildOf(slot),
            ));
            tracing::info!(
                target: "client::ui::shop_auction",
                card_id = ?target_card_id,
                "auction_won_hand_marker: marker spawned (AC5)"
            );
        }
    }
}
