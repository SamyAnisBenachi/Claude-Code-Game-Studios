use bevy::prelude::*;
use lightyear::prelude::{MessageReceiver, MessageSender};
use shared::card::{CardCatalog, CardId, Rarity};
use shared::protocol::{
    AuctionSnapshot, BidRejectedReason, C2SPlaceBid, C2SPurchaseCard, C2SRefreshShop,
    C2SSignalReady, ReliableChannel, RoundPhase, S2CAuctionBidAccepted, S2CAuctionBidRejected,
    S2CAuctionCard, S2CAuctionSettled,
};
use shared::session::PlayerId;

use crate::asset_wiring::{
    bid_button_asset, default_client_card_catalog, resolve_card_display_art, BidButtonChromeState,
    CardDisplayArtAsset, CardDisplayArtFallback, SHOP_PANEL_CHROME_ASSET,
    SHOP_SLOT_WELL_IDLE_ASSET,
};
use crate::card_animations::{
    AuctionPanelTransitionRequested, CardAcquiredAnimReady, SettlementOverlayRequested,
};
use crate::presentation::{PlayerEconomyView, PresentationGameSnapshotMessage};
use crate::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use crate::ui::hud::{HudGoldBroadcastMessage, HudPlayerIds};
use crate::ui::settings::AccessibilityPreferences;

pub const SHOP_AUCTION_UI_PANEL_ROOT_COUNT: usize = 6;
pub const SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT: usize = 9;
pub const SHOP_AUCTION_UI_SHOP_SLOT_COUNT: usize = 3;
pub const AUCTION_PREPARING_TIMEOUT_MS: u32 = 10_000;
pub const AUCTION_AWAITING_SERVER_DELAY_MS: u32 = 1_500;
pub const AUCTION_TOAST_FADE_IN_MS: u32 = 120;
pub const AUCTION_TOAST_HOLD_MS: u32 = 2_000;
pub const AUCTION_TOAST_FADE_OUT_MS: u32 = 120;
pub const AUCTION_SETTLEMENT_TRANSITION_MS: u32 = 350;
pub const DRAFT_INITIAL_OBJECTIVE_COPY: &str = "Select up to 9 cards to keep. You have 45 seconds.";
pub const AUCTION_BID_TARGET_WIDTH_PX: f32 = 108.0;
pub const AUCTION_BID_TARGET_HEIGHT_PX: f32 = 44.0;
pub const AUCTION_BID_FOCUS_RING_WIDTH_PX: f32 = 2.0;

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
        self.panel_state = ShopAuctionAuctionPanelState::Preparing;
        self.preparing_elapsed_ms = 0;
        self.timer_duration_ms = 0;
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
        self.timer_remaining_ms = if self.timer_duration_ms == 0 {
            message.new_timer_ms
        } else {
            message.new_timer_ms.min(self.timer_duration_ms)
        };
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
    pub draft_initial_slots: [Entity; SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT],
    pub draft_initial_bought_overlays: [Entity; SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT],
    pub draft_initial_ready_button: Entity,
    pub draft_initial_ready_status: Entity,
    pub draft_initial_hand_full_banner: Entity,
    pub draft_initial_objective_overlay: Entity,
    pub draft_initial_objective_copy: Entity,
    pub draft_initial_objective_dismiss_button: Entity,
    pub draft_initial_objective_retrieval_button: Entity,
    pub shop_panel: Entity,
    pub shop_slots: [Entity; SHOP_AUCTION_UI_SHOP_SLOT_COUNT],
    pub shop_refresh_button: Entity,
    pub shop_ready_button: Entity,
    pub shop_ready_status: Entity,
    pub shop_hand_full_banner: Entity,
    pub auction_panel: Entity,
    pub auction_featured_card: Entity,
    pub auction_status_text: Entity,
    pub auction_timer_bar: Entity,
    pub auction_bid_status_text: Entity,
    pub auction_bid_buttons: [Entity; 3],
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
pub struct ShopAuctionUiEntity;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopAuctionUiRoot;

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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionBidStatusText;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuctionFeaturedCard;

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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftInitialSlotIndex(pub u8);

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
            .init_resource::<ShopAuctionRefreshConfig>()
            .init_resource::<AuctionBidKeyboardFocus>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<PlayerEconomyView>()
            .init_resource::<ClientPhaseView>()
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
            .add_systems(OnEnter(ClientState::InSession), spawn_shop_auction_ui)
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
                    )
                        .chain()
                        .in_set(ShopAuctionUiSystemSet::Input),
                    (
                        tick_auction_preparing_timeout_system,
                        tick_auction_countdown_system,
                        tick_auction_settlement_transition_system,
                        tick_auction_toast_system,
                        sync_draft_initial_panel_system,
                        sync_shop_panel_system,
                        sync_auction_panel_system,
                        sync_settlement_overlay_system,
                        sync_auction_toast_system,
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
    mut visibility: Query<&mut Visibility>,
) {
    if !current.is_changed() {
        return;
    }

    let previous_mode = *mode;
    let mut next_mode = ShopAuctionUiMode::from_phase(current.phase);
    let settlement_active = settlement_state.transition_active;

    clear_auction_feedback_state(&mut toast_state, &mut timer_target, &mut keyboard_focus);

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
        ShopAuctionUiMode::Shop => shop_state.slots_loaded,
    };

    set_visibility(&mut visibility, entities.root, visibility_for(root_visible));
    set_visibility(
        &mut visibility,
        entities.draft_offering_panel,
        visibility_for(
            next_mode == ShopAuctionUiMode::DraftOffering && draft_state.offering_loaded,
        ),
    );
    set_visibility(
        &mut visibility,
        entities.shop_panel,
        visibility_for(next_mode == ShopAuctionUiMode::Shop && shop_state.slots_loaded),
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
) {
    toast_state.clear();
    *timer_target = AuctionTimerTargetFill::default();
    keyboard_focus.focused_button = None;
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
        clear_auction_feedback_state(&mut toast_state, &mut timer_target, &mut keyboard_focus);

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
    let local_player_id = player_ids
        .as_deref()
        .map(|ids| ids.local_id)
        .or(local_gold.player_id);

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
    let local_player_id = player_ids
        .as_deref()
        .map(|ids| ids.local_id)
        .or(local_gold.player_id);

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
    mut card_anim_messages: Option<ResMut<Messages<CardAcquiredAnimReady>>>,
    mut overlay_messages: Option<ResMut<Messages<SettlementOverlayRequested>>>,
    mut panel_transition_messages: Option<ResMut<Messages<AuctionPanelTransitionRequested>>>,
) {
    let local_player_id = player_ids
        .as_deref()
        .map(|ids| ids.local_id)
        .or(local_gold.player_id);

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
            if let Some(messages) = card_anim_messages.as_deref_mut() {
                messages.write(CardAcquiredAnimReady);
            }
        }
        if let Some(messages) = overlay_messages.as_deref_mut() {
            messages.write(SettlementOverlayRequested);
        }
        if let Some(messages) = panel_transition_messages.as_deref_mut() {
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
) {
    for message in auction_cards.read() {
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
        // p0: slot container — index + text-child link + visibility
        Query<(
            &DraftInitialSlotIndex,
            &DraftInitialSlotText,
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
                let Ok((slot_index, slot_text, mut visibility)) = slots.get_mut(slot_entity) else {
                    continue;
                };
                let text_entity = slot_text.0;

                let Some(card_id) = sorted_card_ids.get(slot_index.0 as usize).copied() else {
                    // Clear: hide slot, wipe text child, remove state components.
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

                if let Ok(mut text) = text_query.get_mut(text_entity) {
                    text.0.clear();
                    text.0
                        .push_str(&format!("{}\n{}g", card_name.as_str(), cost));
                }
                commands.entity(slot_entity).insert((
                    DraftInitialSlotCard(card_id),
                    DraftInitialSlotCardName(card_name),
                    DraftInitialSlotGoldCost(cost),
                    DraftInitialSlotRarity(rarity),
                    DraftInitialSlotState::Available,
                ));
                apply_card_display_art(&mut commands, slot_entity, card, asset_server.as_deref());
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
                Option<&ShopSlotCard>,
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
    {
        let mut visibility = shop_ui.p0();
        if *mode == ShopAuctionUiMode::Shop {
            set_visibility(&mut visibility, entities.root, visibility_for(active));
        }
        set_visibility(&mut visibility, entities.shop_panel, visibility_for(active));
        set_visibility(
            &mut visibility,
            entities.shop_refresh_button,
            visibility_for(active),
        );
        set_visibility(
            &mut visibility,
            entities.shop_ready_button,
            visibility_for(active),
        );
        set_visibility(
            &mut visibility,
            entities.shop_ready_status,
            visibility_for(active && shop_state.ready_signalled),
        );
        set_visibility(
            &mut visibility,
            entities.shop_hand_full_banner,
            visibility_for(active && hand_view.hand_size >= 10),
        );
    }

    {
        let mut slots = shop_ui.p1();
        for (slot_entity, card, mut slot_state, mut visibility, mut text) in &mut slots {
            if !active {
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
        }
    }

    let refresh_cost = displayed_refresh_cost(
        refresh_config.refresh_base_cost,
        refresh_config.refresh_cap,
        shop_state.refresh_count_this_draft,
    );
    let refresh_enabled = active
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
            text.0.push_str(&format!("REFRESH · {refresh_cost}g"));
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
    let local_leading = local_player_is_leading(&auction_state, &local_gold);
    let hand_full = hand_view.hand_size >= 10;
    let bid_status_visible = footer_visible && (local_leading || hand_full);

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
            entities.shop_footer,
            visibility_for(footer_visible),
        );
    }

    {
        let mut texts = auction_ui.p1();
        if let Ok(mut text) = texts.get_mut(entities.auction_featured_card) {
            text.0.clear();
            if let Some(card_id) = auction_state.card_id {
                let card = catalog.cards.get(&card_id);
                apply_card_display_art(
                    &mut commands,
                    entities.auction_featured_card,
                    card,
                    asset_server.as_deref(),
                );
                let name = card
                    .map(|card| card.name_en.as_str())
                    .unwrap_or("Unknown card");
                let rarity = card.map_or(Rarity::Common, |card| card.rarity);
                text.0.push_str(&format!(
                    "{name}\n{:?} - {}g",
                    rarity, auction_state.current_price
                ));
            } else {
                clear_card_display_art(&mut commands, entities.auction_featured_card);
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
            let target_width = if active_countdown {
                auction_timer_width_percent(
                    auction_state.timer_remaining_ms,
                    auction_state.timer_duration_ms,
                )
            } else {
                100.0
            };

            node.width = Val::Percent(target_width);
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
        let local_free_gold = local_gold.free_gold(&economy);
        let has_gold_source = economy.initialized || local_gold.initialized;
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
            let next_state = auction_bid_button_state(
                amount,
                local_free_gold,
                has_gold_source,
                hand_full,
                local_leading,
                auction_state.waiting_for_local_gold_after_opponent_bid(),
                &auction_state,
            );

            *visibility = visibility_for(
                footer_visible && next_state != AuctionBidButtonState::HiddenLeading,
            );
            *state = next_state;
            if let Some(ref server) = asset_server {
                image_node.image =
                    server.load(bid_button_asset(auction_bid_chrome_state(next_state)));
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
            text.0.push_str(settlement_state.overlay_text());
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
        ))
        .id();

    #[cfg(feature = "ui_picking")]
    commands
        .entity(root)
        .insert(bevy::picking::Pickable::IGNORE);

    let draft_offering_panel = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::DraftOffering,
        "Shop Auction Draft Offering Root",
        bottom_panel_node(),
    );
    let (draft_initial_slots, draft_initial_bought_overlays) =
        spawn_draft_initial_grid(&mut commands, draft_offering_panel);
    let draft_initial_ready_button =
        spawn_draft_initial_ready_button(&mut commands, draft_offering_panel);
    let draft_initial_ready_status =
        spawn_draft_initial_status_text(&mut commands, draft_offering_panel);
    let draft_initial_hand_full_banner =
        spawn_draft_initial_hand_full_banner(&mut commands, draft_offering_panel);
    let (
        draft_initial_objective_overlay,
        draft_initial_objective_copy,
        draft_initial_objective_dismiss_button,
    ) = spawn_draft_initial_objective_overlay(&mut commands, draft_offering_panel);
    let draft_initial_objective_retrieval_button =
        spawn_draft_initial_objective_retrieval_button(&mut commands, draft_offering_panel);
    let shop_panel = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::Shop,
        "Shop Auction Shop Root",
        bottom_panel_node(),
    );
    commands
        .entity(shop_panel)
        .insert(ImageNode::new(asset_server.load(SHOP_PANEL_CHROME_ASSET)));
    let shop_slots = spawn_shop_slots(&mut commands, &asset_server, shop_panel);
    let shop_refresh_button = spawn_shop_refresh_button(&mut commands, shop_panel);
    let shop_ready_button = spawn_shop_ready_button(&mut commands, shop_panel);
    let shop_ready_status = spawn_shop_ready_status(&mut commands, shop_panel);
    let shop_hand_full_banner = spawn_shop_hand_full_banner(&mut commands, shop_panel);
    let auction_panel = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::Auction,
        "Shop Auction Auction Root",
        auction_panel_node(),
    );
    // Reuses SHOP_PANEL_CHROME_ASSET as a placeholder until an auction-specific
    // chrome constant lands (PAW-TD-003-a is accept-risk for friend-game scope).
    commands
        .entity(auction_panel)
        .insert(ImageNode::new(asset_server.load(SHOP_PANEL_CHROME_ASSET)));
    let (
        auction_featured_card,
        auction_status_text,
        auction_timer_bar,
        auction_bid_status_text,
        auction_bid_buttons,
    ) = spawn_auction_contents(
        &mut commands,
        &asset_server,
        &refresh_config.bid_increments,
        auction_panel,
    );
    let shop_footer = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::ShopFooter,
        "Shop Auction Footer Root",
        footer_node(),
    );
    let shop_footer_slots = spawn_shop_footer_slots(&mut commands, shop_footer);
    let toast_root = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::Toast,
        "Shop Auction Toast Root",
        toast_node(),
    );
    let toast_text = spawn_auction_toast_text(&mut commands, toast_root);
    let settlement_overlay = spawn_panel_root(
        &mut commands,
        root,
        ShopAuctionPanelRoot::SettlementOverlay,
        "Shop Auction Settlement Overlay Root",
        overlay_node(),
    );
    commands
        .entity(settlement_overlay)
        .insert(BackgroundColor(Color::srgba(0.02, 0.05, 0.08, 0.58)));
    let settlement_overlay_text = spawn_settlement_overlay_text(&mut commands, settlement_overlay);

    commands.insert_resource(ShopAuctionUiEntities {
        root,
        draft_offering_panel,
        draft_initial_slots,
        draft_initial_bought_overlays,
        draft_initial_ready_button,
        draft_initial_ready_status,
        draft_initial_hand_full_banner,
        draft_initial_objective_overlay,
        draft_initial_objective_copy,
        draft_initial_objective_dismiss_button,
        draft_initial_objective_retrieval_button,
        shop_panel,
        shop_slots,
        shop_refresh_button,
        shop_ready_button,
        shop_ready_status,
        shop_hand_full_banner,
        auction_panel,
        auction_featured_card,
        auction_status_text,
        auction_timer_bar,
        auction_bid_status_text,
        auction_bid_buttons,
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
        shop_auction_text_font(18.0),
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
            shop_auction_text_font(15.0),
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
            shop_auction_text_font(24.0),
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
                draft_initial_slot_node(index),
                BackgroundColor(Color::srgba(0.08, 0.12, 0.16, 0.9)),
                Visibility::Hidden,
                ChildOf(parent),
            ))
            .id();

        // Child text entity — holds card name + cost display.
        let text_entity = commands
            .spawn((
                Name::new(format!("Shop Auction Draft Slot {index} Text")),
                ShopAuctionUiEntity,
                DraftInitialSlotTextLabel,
                Text::new(""),
                shop_auction_text_font(14.0),
                TextColor(Color::srgb(0.92, 0.94, 0.96)),
                ChildOf(slot),
            ))
            .id();

        // Store the text child's id on the slot so systems can reach it directly.
        commands
            .entity(slot)
            .insert(DraftInitialSlotText(text_entity));

        let overlay = commands
            .spawn((
                Name::new(format!("Shop Auction Draft Slot {index} Bought Overlay")),
                ShopAuctionUiEntity,
                DraftInitialBoughtOverlay {
                    slot_index: index as u8,
                },
                Text::new("BOUGHT"),
                shop_auction_text_font(14.0),
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
    commands
        .spawn((
            Name::new("Shop Auction Draft Ready Button"),
            ShopAuctionUiEntity,
            DraftInitialReadyButton,
            Button,
            Interaction::None,
            Text::new("Ready"),
            shop_auction_text_font(16.0),
            TextColor(Color::srgb(0.98, 0.93, 0.72)),
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
            shop_auction_text_font(13.0),
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
            shop_auction_text_font(14.0),
            TextColor(Color::srgb(1.0, 0.78, 0.55)),
            draft_initial_hand_full_banner_node(),
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
            shop_auction_text_font(14.0),
            TextColor(Color::srgb(0.96, 0.98, 1.0)),
            draft_initial_objective_copy_node(),
            Visibility::Hidden,
            ChildOf(overlay),
        ))
        .id();

    let dismiss = commands
        .spawn((
            Name::new("Shop Auction Draft Objective Dismiss"),
            ShopAuctionUiEntity,
            DraftInitialObjectiveDismissButton,
            Button,
            Interaction::None,
            Text::new("Dismiss"),
            shop_auction_text_font(13.0),
            TextColor(Color::srgb(0.98, 0.93, 0.72)),
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
    commands
        .spawn((
            Name::new("Shop Auction Draft Objective Retrieval"),
            ShopAuctionUiEntity,
            DraftInitialObjectiveRetrievalButton,
            Button,
            Interaction::None,
            Text::new("Objective"),
            shop_auction_text_font(13.0),
            TextColor(Color::srgb(0.74, 0.92, 0.92)),
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
) -> [Entity; SHOP_AUCTION_UI_SHOP_SLOT_COUNT] {
    std::array::from_fn(|index| {
        commands
            .spawn((
                Name::new(format!("Shop Auction Shop Slot {index}")),
                ShopAuctionUiEntity,
                ShopSlotIndex(index as u8),
                ShopSlotState::Empty,
                Button,
                Interaction::None,
                shop_slot_node(index),
                ImageNode::new(asset_server.load(SHOP_SLOT_WELL_IDLE_ASSET)),
                Text::new("Empty"),
                shop_auction_text_font(14.0),
                TextColor(Color::srgb(0.92, 0.94, 0.96)),
                Visibility::Hidden,
                ChildOf(parent),
            ))
            .id()
    })
}

fn spawn_shop_refresh_button(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Name::new("Shop Auction Refresh Button"),
            ShopAuctionUiEntity,
            ShopRefreshButton,
            Button,
            Interaction::None,
            ShopRefreshButtonState { enabled: false },
            Text::new("REFRESH · 1g"),
            shop_auction_text_font(15.0),
            TextColor(Color::srgb(0.74, 0.92, 0.92)),
            shop_refresh_button_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_shop_ready_button(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Name::new("Shop Auction Shop Ready Button"),
            ShopAuctionUiEntity,
            ShopReadyButton,
            Button,
            Interaction::None,
            Text::new("Ready"),
            shop_auction_text_font(16.0),
            TextColor(Color::srgb(0.98, 0.93, 0.72)),
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
            shop_auction_text_font(13.0),
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
            shop_auction_text_font(14.0),
            TextColor(Color::srgb(1.0, 0.78, 0.55)),
            shop_hand_full_banner_node(),
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
) -> (Entity, Entity, Entity, Entity, [Entity; 3]) {
    let featured_card = commands
        .spawn((
            Name::new("Shop Auction Featured Auction Card"),
            ShopAuctionUiEntity,
            AuctionFeaturedCard,
            Text::new(""),
            shop_auction_text_font(26.0),
            TextColor(Color::srgb(0.98, 0.94, 0.80)),
            auction_featured_card_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();

    let status_text = commands
        .spawn((
            Name::new("Shop Auction Auction Status"),
            ShopAuctionUiEntity,
            AuctionStatusText,
            Text::new(""),
            shop_auction_text_font(18.0),
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
            shop_auction_text_font(18.0),
            TextColor(Color::srgb(0.98, 0.88, 0.40)),
            auction_bid_status_text_node(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id();

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
                Text::new(""),
                shop_auction_text_font(17.0),
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

    (
        featured_card,
        status_text,
        timer_bar,
        bid_status_text,
        bid_buttons,
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
                shop_auction_text_font(13.0),
                TextColor(Color::srgba(0.92, 0.94, 0.96, 0.30)),
                shop_footer_slot_node(index),
                Visibility::Hidden,
                ChildOf(parent),
            ))
            .id()
    })
}

fn bottom_panel_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        bottom: Val::Px(0.0),
        height: Val::Px(260.0),
        ..default()
    }
}

fn draft_initial_slot_node(index: usize) -> Node {
    let column = index % 3;
    let row = index / 3;
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(96.0 + column as f32 * 132.0),
        top: Val::Px(30.0 + row as f32 * 66.0),
        width: Val::Px(120.0),
        height: Val::Px(56.0),
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

fn draft_initial_ready_button_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(96.0),
        top: Val::Px(58.0),
        width: Val::Px(132.0),
        height: Val::Px(36.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn draft_initial_status_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(96.0),
        top: Val::Px(100.0),
        width: Val::Px(180.0),
        height: Val::Px(28.0),
        ..default()
    }
}

fn draft_initial_hand_full_banner_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(96.0),
        top: Val::Px(138.0),
        width: Val::Px(260.0),
        height: Val::Px(30.0),
        ..default()
    }
}

fn draft_initial_objective_overlay_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(88.0),
        top: Val::Px(2.0),
        width: Val::Px(640.0),
        height: Val::Px(28.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn draft_initial_objective_copy_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(10.0),
        top: Val::Px(5.0),
        width: Val::Px(500.0),
        height: Val::Px(18.0),
        ..default()
    }
}

fn draft_initial_objective_dismiss_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(8.0),
        top: Val::Px(4.0),
        width: Val::Px(88.0),
        height: Val::Px(20.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn draft_initial_objective_retrieval_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(96.0),
        top: Val::Px(12.0),
        width: Val::Px(116.0),
        height: Val::Px(28.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn shop_slot_node(index: usize) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(92.0 + index as f32 * 154.0),
        top: Val::Px(44.0),
        width: Val::Px(136.0),
        height: Val::Px(78.0),
        border: UiRect::all(Val::Px(1.0)),
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

fn auction_panel_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(80.0),
        bottom: Val::Px(140.0),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    }
}

fn auction_featured_card_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(34.0),
        top: Val::Px(48.0),
        width: Val::Px(300.0),
        height: Val::Px(120.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn auction_status_text_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(34.0),
        top: Val::Px(176.0),
        width: Val::Px(360.0),
        height: Val::Px(32.0),
        ..default()
    }
}

fn auction_timer_bar_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(34.0),
        top: Val::Px(216.0),
        height: Val::Px(10.0),
        ..default()
    }
}

fn auction_bid_status_text_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(34.0),
        top: Val::Px(292.0),
        width: Val::Px(360.0),
        height: Val::Px(40.0),
        ..default()
    }
}

fn auction_bid_button_node(index: usize) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(34.0 + index as f32 * 9.0),
        top: Val::Px(248.0),
        width: Val::Px(AUCTION_BID_TARGET_WIDTH_PX),
        height: Val::Px(AUCTION_BID_TARGET_HEIGHT_PX),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn footer_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        bottom: Val::Px(100.0),
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

fn toast_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(24.0),
        bottom: Val::Px(220.0),
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

/// Maps an [`AuctionBidButtonState`] to the corresponding [`BidButtonChromeState`]
/// for asset selection. Only `Enabled` maps to `Normal`; all other states use `Disabled`.
fn auction_bid_chrome_state(state: AuctionBidButtonState) -> BidButtonChromeState {
    match state {
        AuctionBidButtonState::Enabled => BidButtonChromeState::Normal,
        _ => BidButtonChromeState::Disabled,
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

fn shop_active(mode: &ShopAuctionUiMode, state: &ShopAuctionShopState) -> bool {
    *mode == ShopAuctionUiMode::Shop && state.slots_loaded
}

fn should_buffer_shop_slots(phase: RoundPhase) -> bool {
    matches!(
        phase,
        RoundPhase::DraftInitial | RoundPhase::DraftAuction | RoundPhase::Resolution
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

    text.0.clear();
    text.0
        .push_str(&format!("{}\n{:?} · {}g", card_name.as_str(), rarity, cost));
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

fn apply_card_display_art(
    commands: &mut Commands,
    entity: Entity,
    card: Option<&shared::card::CardData>,
    asset_server: Option<&AssetServer>,
) {
    match resolve_card_display_art(card) {
        Ok(path) => {
            let mut entity_commands = commands.entity(entity);
            entity_commands.insert(CardDisplayArtAsset { path });
            entity_commands.remove::<CardDisplayArtFallback>();
            if let Some(asset_server) = asset_server {
                entity_commands.insert(ImageNode::new(asset_server.load(path)));
            }
        }
        Err(reason) => {
            let mut entity_commands = commands.entity(entity);
            entity_commands.insert(CardDisplayArtFallback { reason });
            entity_commands.remove::<(CardDisplayArtAsset, ImageNode)>();
        }
    }
}

fn clear_card_display_art(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .remove::<(CardDisplayArtAsset, CardDisplayArtFallback, ImageNode)>();
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
