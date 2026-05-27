//! Single-client bot soak trigger route systems.
//!
//! Drives a headless Bevy client through the full bot-room protocol path so
//! `BotLobbyPlugin` + `BotActionLoopPlugin` can run an autonomous bot-vs-bot
//! soak on the server without any GUI involvement.
//!
//! State machine (sequential gates, enforced by atomic flags):
//!   Connect → C2SHello → S2CHandshake
//!   → C2SCreateBotRoom → S2CRoomCreated
//!   → C2SSelectClass + C2SConfirmClass  (bot's class auto-confirmed by server)
//!   → S2CDraftOffering → C2SPurchaseCard → S2CCardAcquired(DraftInitial) → C2SSignalReady
//!   → Loop per round:
//!       S2CPhaseChanged(DraftShop)    → C2SSignalReady
//!       S2CPhaseChanged(Placement)    → C2SSubmitPlacement (non-empty when card affordable)
//!       S2CAuctionCard                → C2SPlaceBid (starting_price + 1)
//!   → S2CGameOver → done
//!
//! PROMPT 1692: non-empty placement realism.
//!   The trigger now picks the cheapest affordable Minion from the DraftInitial
//!   offering and submits a real placement (lane 1, cell 1 — Player A spawn) when
//!   the card's cost is covered by the tracked mana budget.  Empty placements are
//!   still submitted when the hand is empty (PROMPT 1678 contract preserved).
//!
//! Product-rule compliance (per PROMPT 1672): every C2S message goes through
//! the production Lightyear WebSocket transport and the same server handlers
//! that a GUI client would reach. Nothing here mutates server state directly.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use lightyear::prelude::*;
use shared::card::{CardId, ClassId};
use shared::protocol::{
    BotKind, C2SConfirmClass, C2SCreateBotRoom, C2SHello, C2SPlaceBid, C2SPurchaseCard,
    C2SSelectClass, C2SSignalReady, C2SSubmitPlacement, CardSource, GameMode, PlacedCardSubmit,
    PlayTarget, ReliableChannel, RoundPhase as ProtocolRoundPhase, S2CAuctionCard, S2CCardAcquired,
    S2CDraftOffering, S2CGoldUpdate, S2CGameOver, S2CHandshake, S2CObjectiveIdentities,
    S2CPhaseChanged, S2CRoomCreated,
};

/// Card metadata extracted from the card catalog JSON at startup.
/// Only the fields the trigger needs (cost and whether it is a Minion).
#[derive(Debug, Clone)]
pub struct TriggerCardEntry {
    pub cost: u32,
    pub is_minion: bool,
}

/// Shared state for the single-client bot soak trigger route.
/// All atomics are Arc-wrapped so they can be cloned across the App boundary
/// and read in the main tick-loop without locking.
#[derive(Clone, Resource)]
pub struct BotSoakRoute {
    // handshake
    pub received_handshake: Arc<AtomicBool>,
    // bot room
    pub sent_create_bot_room: Arc<AtomicBool>,
    pub received_room_created: Arc<AtomicBool>,
    // class selection (pre-game lobby)
    pub sent_select_class: Arc<AtomicBool>,
    pub sent_confirm_class: Arc<AtomicBool>,
    // draft initial purchase
    pub initial_card_id: Arc<AtomicU64>,     // 0 = not yet offered
    pub initial_card_cost: Arc<AtomicU32>,   // 0 = unknown (card_info not loaded)
    pub sent_initial_purchase: Arc<AtomicBool>,
    pub received_initial_card: Arc<AtomicBool>,
    pub sent_initial_ready: Arc<AtomicBool>,
    // per-round loop counters
    pub placement_count: Arc<AtomicUsize>,
    pub draft_shop_count: Arc<AtomicUsize>,
    pub auction_count: Arc<AtomicUsize>,
    pub placements_sent: Arc<AtomicUsize>,
    pub draft_shop_ready_sent: Arc<AtomicUsize>,
    pub auction_bid_sent: Arc<AtomicUsize>,
    pub auction_starting_price: Arc<AtomicUsize>,
    pub last_phase: Arc<Mutex<Option<ProtocolRoundPhase>>>,
    // placement realism (PROMPT 1692)
    pub initial_card_placed: Arc<AtomicBool>,
    pub tracked_current_mana: Arc<AtomicU32>,
    pub tracked_reserve_mana: Arc<AtomicU32>,
    // static card cost/type lookup loaded at startup — read-only after init
    pub card_info: Arc<HashMap<u32, TriggerCardEntry>>,
    // terminal
    pub received_game_over: Arc<AtomicBool>,
}

impl Default for BotSoakRoute {
    fn default() -> Self {
        Self {
            received_handshake: Arc::new(AtomicBool::new(false)),
            sent_create_bot_room: Arc::new(AtomicBool::new(false)),
            received_room_created: Arc::new(AtomicBool::new(false)),
            sent_select_class: Arc::new(AtomicBool::new(false)),
            sent_confirm_class: Arc::new(AtomicBool::new(false)),
            initial_card_id: Arc::new(AtomicU64::new(0)),
            initial_card_cost: Arc::new(AtomicU32::new(0)),
            sent_initial_purchase: Arc::new(AtomicBool::new(false)),
            received_initial_card: Arc::new(AtomicBool::new(false)),
            sent_initial_ready: Arc::new(AtomicBool::new(false)),
            placement_count: Arc::new(AtomicUsize::new(0)),
            draft_shop_count: Arc::new(AtomicUsize::new(0)),
            auction_count: Arc::new(AtomicUsize::new(0)),
            placements_sent: Arc::new(AtomicUsize::new(0)),
            draft_shop_ready_sent: Arc::new(AtomicUsize::new(0)),
            auction_bid_sent: Arc::new(AtomicUsize::new(0)),
            auction_starting_price: Arc::new(AtomicUsize::new(0)),
            last_phase: Arc::new(Mutex::new(None)),
            initial_card_placed: Arc::new(AtomicBool::new(false)),
            tracked_current_mana: Arc::new(AtomicU32::new(0)),
            tracked_reserve_mana: Arc::new(AtomicU32::new(0)),
            card_info: Arc::new(HashMap::new()),
            received_game_over: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl BotSoakRoute {
    pub fn is_done(&self) -> bool {
        self.received_game_over.load(Ordering::SeqCst)
    }

    pub fn rounds_observed(&self) -> usize {
        self.placement_count.load(Ordering::SeqCst)
    }
}

// ---- Pure helpers (testable without Bevy) -----------------------------------

/// Select the best card for the trigger to purchase from a DraftInitial offering.
///
/// Preference order:
///   1. Cheapest Minion whose cost ≤ `mana_budget` (placeable in the same round)
///   2. Cheapest Minion regardless of cost (will become placeable as budget grows)
///   3. Cheapest non-Minion (fallback when no Minion in pool)
///   4. First card (fallback when `card_info` is empty / card unknown)
///
/// Returns `(card_id, cost)` or `None` if the offering is empty.
pub fn pick_best_trigger_card(
    offering: &[u32],
    card_info: &HashMap<u32, TriggerCardEntry>,
    mana_budget: u32,
) -> Option<(u32, u32)> {
    if offering.is_empty() {
        return None;
    }

    let mut best_affordable_minion: Option<(u32, u32)> = None; // (cost, card_id)
    let mut best_any_minion: Option<(u32, u32)> = None;
    let mut best_non_minion: Option<(u32, u32)> = None;

    for &card_id in offering {
        if let Some(entry) = card_info.get(&card_id) {
            if entry.is_minion {
                if entry.cost <= mana_budget
                    && (best_affordable_minion.is_none()
                        || entry.cost < best_affordable_minion.unwrap().0)
                {
                    best_affordable_minion = Some((entry.cost, card_id));
                }
                if best_any_minion.is_none() || entry.cost < best_any_minion.unwrap().0 {
                    best_any_minion = Some((entry.cost, card_id));
                }
            } else if best_non_minion.is_none() || entry.cost < best_non_minion.unwrap().0 {
                best_non_minion = Some((entry.cost, card_id));
            }
        }
    }

    // Ranked preference: affordable minion → any minion → non-minion → first card
    best_affordable_minion
        .or(best_any_minion)
        .or(best_non_minion)
        .map(|(cost, id)| (id, cost))
        .or_else(|| Some((offering[0], 0)))
}

/// Build a `PlacedCardSubmit` for the trigger's initial card, if affordable now.
///
/// Player A spawn position: lane 1, cell 1 (BoardConfig::default player_a_spawn_cell).
/// Returns `None` when the card has already been placed, has no known cost,
/// or when total mana is insufficient to cover the placement cost.
pub fn build_trigger_placement(
    card_id: u32,
    card_cost: u32,
    current_mana: u32,
    reserve_mana: u32,
    already_placed: bool,
) -> Option<PlacedCardSubmit> {
    if already_placed || card_id == 0 || card_cost == 0 {
        return None;
    }

    let total_mana = current_mana.saturating_add(reserve_mana);
    if total_mana < card_cost {
        return None;
    }

    // Pay from current mana first, spill into reserve — mirrors build_bot_placements logic.
    let from_current = card_cost.min(current_mana);
    let from_reserve = card_cost.saturating_sub(from_current);

    if from_reserve > reserve_mana {
        // Reserve insufficient even though total looked OK (shouldn't happen in practice).
        return None;
    }

    Some(PlacedCardSubmit {
        card_id: CardId(card_id),
        // Player A always spawns at lane 1, cell 1 (BoardConfig default spawn cell).
        target: PlayTarget::BoardCell { lane: 1, cell: 1 },
        current_mana_spend: from_current,
        reserve_mana_spend: from_reserve,
    })
}

// ---- Outbound (C2S) systems -------------------------------------------------

pub fn send_hello_until_handshake(
    route: Res<BotSoakRoute>,
    mut senders: Query<&mut MessageSender<C2SHello>>,
) {
    if route.received_handshake.load(Ordering::SeqCst) {
        return;
    }
    for mut sender in &mut senders {
        sender.send::<ReliableChannel>(C2SHello {
            protocol_version: shared::config::GameConfig::default().protocol_version,
            session_token: None,
        });
    }
}

pub fn send_create_bot_room(
    route: Res<BotSoakRoute>,
    mut senders: Query<&mut MessageSender<C2SCreateBotRoom>>,
) {
    if !route.received_handshake.load(Ordering::SeqCst)
        || route.sent_create_bot_room.load(Ordering::SeqCst)
    {
        return;
    }
    for mut sender in &mut senders {
        sender.send::<ReliableChannel>(C2SCreateBotRoom {
            mode: GameMode::OneVOne,
            bot_kind: BotKind::Default,
        });
        route.sent_create_bot_room.store(true, Ordering::SeqCst);
        tracing::info!("bot_soak_trigger: C2SCreateBotRoom sent (OneVOne / Default)");
    }
}

pub fn send_class_selection(
    route: Res<BotSoakRoute>,
    mut select: Query<&mut MessageSender<C2SSelectClass>>,
    mut confirm: Query<&mut MessageSender<C2SConfirmClass>>,
) {
    if !route.received_room_created.load(Ordering::SeqCst) {
        return;
    }
    if !route.sent_select_class.load(Ordering::SeqCst) {
        for mut sender in &mut select {
            sender.send::<ReliableChannel>(C2SSelectClass {
                class_id: ClassId::Iop,
            });
            route.sent_select_class.store(true, Ordering::SeqCst);
            tracing::info!("bot_soak_trigger: C2SSelectClass sent (Iop)");
        }
    }
    if !route.sent_confirm_class.load(Ordering::SeqCst) {
        for mut sender in &mut confirm {
            sender.send::<ReliableChannel>(C2SConfirmClass {
                class_id: ClassId::Iop,
            });
            route.sent_confirm_class.store(true, Ordering::SeqCst);
            tracing::info!("bot_soak_trigger: C2SConfirmClass sent (Iop)");
        }
    }
}

pub fn send_initial_purchase(
    route: Res<BotSoakRoute>,
    mut senders: Query<&mut MessageSender<C2SPurchaseCard>>,
) {
    let card_id = route.initial_card_id.load(Ordering::SeqCst);
    if card_id == 0 || route.sent_initial_purchase.load(Ordering::SeqCst) {
        return;
    }
    for mut sender in &mut senders {
        sender.send::<ReliableChannel>(C2SPurchaseCard {
            card_id: CardId(card_id as u32),
        });
        route.sent_initial_purchase.store(true, Ordering::SeqCst);
        tracing::info!(card_id, "bot_soak_trigger: C2SPurchaseCard sent (initial)");
    }
}

pub fn send_initial_ready(
    route: Res<BotSoakRoute>,
    mut senders: Query<&mut MessageSender<C2SSignalReady>>,
) {
    if !route.received_initial_card.load(Ordering::SeqCst)
        || route.sent_initial_ready.load(Ordering::SeqCst)
    {
        return;
    }
    for mut sender in &mut senders {
        sender.send::<ReliableChannel>(C2SSignalReady { retract: false });
        route.sent_initial_ready.store(true, Ordering::SeqCst);
        tracing::info!("bot_soak_trigger: C2SSignalReady sent (draft initial)");
    }
}

pub fn send_loop_actions(
    route: Res<BotSoakRoute>,
    mut placement_senders: Query<&mut MessageSender<C2SSubmitPlacement>>,
    mut ready_senders: Query<&mut MessageSender<C2SSignalReady>>,
    mut bid_senders: Query<&mut MessageSender<C2SPlaceBid>>,
) {
    let last_phase = route
        .last_phase
        .lock()
        .expect("last_phase mutex must not be poisoned")
        .clone();
    let placement_count = route.placement_count.load(Ordering::SeqCst);
    let draft_shop_count = route.draft_shop_count.load(Ordering::SeqCst);
    let auction_count = route.auction_count.load(Ordering::SeqCst);

    match last_phase {
        Some(ProtocolRoundPhase::Placement) => {
            if route.placements_sent.load(Ordering::SeqCst) < placement_count {
                if let Some(mut sender) = placement_senders.iter_mut().next() {
                    let card_id = route.initial_card_id.load(Ordering::SeqCst) as u32;
                    let card_cost = route.initial_card_cost.load(Ordering::SeqCst);
                    let current_mana = route.tracked_current_mana.load(Ordering::SeqCst);
                    let reserve_mana = route.tracked_reserve_mana.load(Ordering::SeqCst);
                    let already_placed = route.initial_card_placed.load(Ordering::SeqCst);

                    let placement_entry = build_trigger_placement(
                        card_id,
                        card_cost,
                        current_mana,
                        reserve_mana,
                        already_placed,
                    );

                    let is_non_empty = placement_entry.is_some();
                    let placements: Vec<PlacedCardSubmit> =
                        placement_entry.into_iter().collect();

                    sender.send::<ReliableChannel>(C2SSubmitPlacement {
                        placements,
                    });
                    route.placements_sent.store(placement_count, Ordering::SeqCst);

                    if is_non_empty {
                        route.initial_card_placed.store(true, Ordering::SeqCst);
                        tracing::info!(
                            placement_count,
                            card_id,
                            card_cost,
                            current_mana,
                            reserve_mana,
                            "bot_soak_trigger: C2SSubmitPlacement sent (non-empty)"
                        );
                    } else {
                        tracing::info!(
                            placement_count,
                            card_id,
                            card_cost,
                            current_mana,
                            reserve_mana,
                            already_placed,
                            "bot_soak_trigger: C2SSubmitPlacement sent (empty)"
                        );
                    }
                }
            }
        }
        Some(ProtocolRoundPhase::DraftShop) => {
            if route.draft_shop_ready_sent.load(Ordering::SeqCst) < draft_shop_count {
                if let Some(mut sender) = ready_senders.iter_mut().next() {
                    sender.send::<ReliableChannel>(C2SSignalReady { retract: false });
                    route
                        .draft_shop_ready_sent
                        .store(draft_shop_count, Ordering::SeqCst);
                    tracing::info!(
                        draft_shop_count,
                        "bot_soak_trigger: C2SSignalReady sent (draft shop)"
                    );
                }
            }
        }
        Some(ProtocolRoundPhase::DraftAuction) => {
            let starting = route.auction_starting_price.load(Ordering::SeqCst);
            if route.auction_bid_sent.load(Ordering::SeqCst) < auction_count && starting > 0 {
                if let Some(mut sender) = bid_senders.iter_mut().next() {
                    sender.send::<ReliableChannel>(C2SPlaceBid {
                        amount: (starting as u32).saturating_add(1),
                    });
                    route.auction_bid_sent.store(auction_count, Ordering::SeqCst);
                    tracing::info!(
                        starting,
                        "bot_soak_trigger: C2SPlaceBid sent (starting+1)"
                    );
                }
            }
        }
        _ => {}
    }
}

// ---- Inbound (S2C) systems --------------------------------------------------

pub fn record_handshake(
    route: Res<BotSoakRoute>,
    mut receivers: Query<&mut MessageReceiver<S2CHandshake>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::info!(
                player_id = message.player_id.0,
                "bot_soak_trigger: S2CHandshake received"
            );
            route.received_handshake.store(true, Ordering::SeqCst);
        }
    }
}

pub fn record_room_created(
    route: Res<BotSoakRoute>,
    mut receivers: Query<&mut MessageReceiver<S2CRoomCreated>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            let bot_slots = message.slots.iter().filter(|s| s.is_bot).count();
            tracing::info!(
                room_code = %message.room_code,
                total_slots = message.slots.len(),
                bot_slots,
                "bot_soak_trigger: S2CRoomCreated received"
            );
            route.received_room_created.store(true, Ordering::SeqCst);
        }
    }
}

/// Track per-round mana budget from S2CGoldUpdate so placement decisions
/// can correctly split current_mana_spend / reserve_mana_spend (PROMPT 1692).
pub fn record_gold_update(
    route: Res<BotSoakRoute>,
    mut receivers: Query<&mut MessageReceiver<S2CGoldUpdate>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            route
                .tracked_current_mana
                .store(message.current_mana, Ordering::SeqCst);
            route
                .tracked_reserve_mana
                .store(message.reserve_mana, Ordering::SeqCst);
            tracing::debug!(
                current_mana = message.current_mana,
                reserve_mana = message.reserve_mana,
                gold = message.gold,
                "bot_soak_trigger: S2CGoldUpdate recorded"
            );
        }
    }
}

/// Pick the cheapest affordable Minion from the DraftInitial offering
/// and store its card_id + cost for placement (PROMPT 1692).
pub fn record_draft_offering(
    route: Res<BotSoakRoute>,
    mut receivers: Query<&mut MessageReceiver<S2CDraftOffering>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            if route.initial_card_id.load(Ordering::SeqCst) != 0 {
                continue; // already picked (idempotent)
            }

            let current_mana = route.tracked_current_mana.load(Ordering::SeqCst);
            // Default to 1 when S2CGoldUpdate hasn't arrived yet: round 1 mana
            // ramp yields current_mana = 1, making this a safe soak fallback.
            let mana_budget = current_mana.max(1);

            let offering: Vec<u32> = message.card_ids.iter().map(|c| c.0).collect();
            let (picked_id, picked_cost) =
                pick_best_trigger_card(&offering, &route.card_info, mana_budget)
                    .unwrap_or((0, 0));

            if picked_id != 0 {
                route
                    .initial_card_id
                    .store(u64::from(picked_id), Ordering::SeqCst);
                route
                    .initial_card_cost
                    .store(picked_cost, Ordering::SeqCst);
                tracing::info!(
                    card_id = picked_id,
                    card_cost = picked_cost,
                    mana_budget,
                    offering_len = offering.len(),
                    "bot_soak_trigger: S2CDraftOffering — picked cheapest Minion"
                );
            }
        }
    }
}

pub fn record_card_acquired(
    route: Res<BotSoakRoute>,
    mut receivers: Query<&mut MessageReceiver<S2CCardAcquired>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            if message.source == CardSource::DraftInitial {
                route.received_initial_card.store(true, Ordering::SeqCst);
                tracing::info!("bot_soak_trigger: S2CCardAcquired (DraftInitial)");
            }
        }
    }
}

pub fn record_phase_and_auction(
    route: Res<BotSoakRoute>,
    mut phase_receivers: Query<&mut MessageReceiver<S2CPhaseChanged>>,
    mut auction_receivers: Query<&mut MessageReceiver<S2CAuctionCard>>,
    mut obj_receivers: Query<&mut MessageReceiver<S2CObjectiveIdentities>>,
) {
    for mut receiver in &mut phase_receivers {
        for message in receiver.receive() {
            tracing::info!(
                phase = ?message.phase,
                "bot_soak_trigger: S2CPhaseChanged"
            );
            *route
                .last_phase
                .lock()
                .expect("last_phase mutex must not be poisoned") = Some(message.phase);
            match message.phase {
                ProtocolRoundPhase::Placement => {
                    route.placement_count.fetch_add(1, Ordering::SeqCst);
                }
                ProtocolRoundPhase::DraftShop => {
                    route.draft_shop_count.fetch_add(1, Ordering::SeqCst);
                }
                ProtocolRoundPhase::DraftAuction => {
                    route.auction_count.fetch_add(1, Ordering::SeqCst);
                }
                _ => {}
            }
        }
    }
    for mut receiver in &mut auction_receivers {
        for message in receiver.receive() {
            route
                .auction_starting_price
                .store(message.starting_price as usize, Ordering::SeqCst);
        }
    }
    for mut receiver in &mut obj_receivers {
        for _ in receiver.receive() {
            // drain — objective identity hints not needed by the trigger client
        }
    }
}

pub fn record_game_over(
    route: Res<BotSoakRoute>,
    mut receivers: Query<&mut MessageReceiver<S2CGameOver>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::info!(
                round = message.round,
                reason = ?message.reason,
                "bot_soak_trigger: S2CGameOver received — trigger complete"
            );
            route.received_game_over.store(true, Ordering::SeqCst);
        }
    }
}

// ---- Unit tests (pure helpers only — no Bevy runtime required) --------------

#[cfg(test)]
mod tests {
    use super::*;

    fn card_info_fixture() -> HashMap<u32, TriggerCardEntry> {
        [
            (101, TriggerCardEntry { cost: 1, is_minion: true }),   // Tofu Scout
            (102, TriggerCardEntry { cost: 2, is_minion: true }),   // Wabbit Guard
            (105, TriggerCardEntry { cost: 2, is_minion: false }),  // Guild Errand (Order)
            (107, TriggerCardEntry { cost: 4, is_minion: true }),   // Vault Sentry
        ]
        .into_iter()
        .collect()
    }

    // ---- pick_best_trigger_card -------------------------------------------------

    #[test]
    fn test_pick_cheapest_affordable_minion() {
        let info = card_info_fixture();
        // Offering: 107 (cost 4), 101 (cost 1), 105 (non-minion). Budget = 1.
        let result = pick_best_trigger_card(&[107, 101, 105], &info, 1);
        assert_eq!(result, Some((101, 1)));
    }

    #[test]
    fn test_pick_cheapest_minion_fallback_when_no_affordable() {
        let info = card_info_fixture();
        // Budget = 1, only 107 (cost 4) is a Minion → falls back to cheapest Minion.
        let result = pick_best_trigger_card(&[107, 105], &info, 1);
        assert_eq!(result, Some((107, 4)));
    }

    #[test]
    fn test_pick_non_minion_when_no_minion_in_offering() {
        let info = card_info_fixture();
        let result = pick_best_trigger_card(&[105], &info, 2);
        assert_eq!(result, Some((105, 2)));
    }

    #[test]
    fn test_pick_first_card_when_card_info_empty() {
        // No cost data → fall through to first-card fallback (cost 0 = unknown).
        let result = pick_best_trigger_card(&[107, 101], &HashMap::new(), 5);
        assert_eq!(result, Some((107, 0)));
    }

    #[test]
    fn test_pick_returns_none_for_empty_offering() {
        let result = pick_best_trigger_card(&[], &card_info_fixture(), 5);
        assert_eq!(result, None);
    }

    #[test]
    fn test_pick_prefers_minion_over_cheaper_non_minion() {
        let info = card_info_fixture();
        // 105 (non-minion cost 2) vs 102 (minion cost 2, affordable at budget 2).
        let result = pick_best_trigger_card(&[105, 102], &info, 2);
        assert_eq!(result, Some((102, 2)));
    }

    // ---- build_trigger_placement -----------------------------------------------

    #[test]
    fn test_non_empty_placement_when_affordable() {
        let p = build_trigger_placement(101, 1, 1, 0, false)
            .expect("cost 1 affordable with current_mana 1");
        assert_eq!(p.card_id.0, 101);
        assert_eq!(p.current_mana_spend + p.reserve_mana_spend, 1);
        assert_eq!(p.target, PlayTarget::BoardCell { lane: 1, cell: 1 });
    }

    #[test]
    fn test_placement_spills_cost_into_reserve() {
        // cost 3, current = 2, reserve = 1 → from_current=2, from_reserve=1
        let p = build_trigger_placement(102, 3, 2, 1, false)
            .expect("total mana 3 covers cost 3");
        assert_eq!(p.current_mana_spend, 2);
        assert_eq!(p.reserve_mana_spend, 1);
    }

    #[test]
    fn test_empty_when_already_placed() {
        let result = build_trigger_placement(101, 1, 5, 0, true);
        assert!(result.is_none(), "no second placement after card is placed");
    }

    #[test]
    fn test_empty_when_unaffordable() {
        // cost 4, total mana 1 (current=1, reserve=0)
        let result = build_trigger_placement(107, 4, 1, 0, false);
        assert!(result.is_none(), "insufficient mana for cost-4 card");
    }

    #[test]
    fn test_empty_when_card_id_zero() {
        let result = build_trigger_placement(0, 1, 5, 0, false);
        assert!(result.is_none(), "card_id 0 means nothing was ever purchased");
    }

    #[test]
    fn test_empty_when_cost_zero() {
        // cost 0 means card_info was not loaded; treat as unknown → empty.
        let result = build_trigger_placement(101, 0, 5, 0, false);
        assert!(result.is_none(), "cost 0 is sentinel for unknown — skip placement");
    }

    #[test]
    fn test_mana_split_draws_current_first() {
        // cost 2, current 1, reserve 3 → from_current=1, from_reserve=1
        let p = build_trigger_placement(102, 2, 1, 3, false)
            .expect("total mana 4 covers cost 2");
        assert_eq!(p.current_mana_spend, 1);
        assert_eq!(p.reserve_mana_spend, 1);
    }
}
