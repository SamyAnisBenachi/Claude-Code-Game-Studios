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
//!       S2CPhaseChanged(Placement)    → C2SSubmitPlacement([])
//!       S2CAuctionCard                → C2SPlaceBid (starting_price + 1)
//!   → S2CGameOver → done
//!
//! Product-rule compliance (per PROMPT 1672): every C2S message goes through
//! the production Lightyear WebSocket transport and the same server handlers
//! that a GUI client would reach. Nothing here mutates server state directly.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use lightyear::prelude::*;
use shared::card::{CardId, ClassId};
use shared::protocol::{
    BotKind, C2SConfirmClass, C2SCreateBotRoom, C2SHello, C2SPlaceBid, C2SPurchaseCard,
    C2SSelectClass, C2SSignalReady, C2SSubmitPlacement, CardSource, GameMode, ReliableChannel,
    RoundPhase as ProtocolRoundPhase, S2CAuctionCard, S2CCardAcquired, S2CDraftOffering,
    S2CGameOver, S2CHandshake, S2CObjectiveIdentities, S2CPhaseChanged, S2CRoomCreated,
};

/// Shared state for the single-client bot soak trigger route.
/// All fields are Arc-wrapped atomics so they can be cloned across the App
/// boundary and read in the main tick-loop without locking.
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
    pub initial_card_id: Arc<AtomicU64>,        // 0 = not yet offered
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
                    sender.send::<ReliableChannel>(C2SSubmitPlacement {
                        placements: Vec::new(),
                    });
                    route.placements_sent.store(placement_count, Ordering::SeqCst);
                    tracing::info!(
                        placement_count,
                        "bot_soak_trigger: C2SSubmitPlacement sent (empty)"
                    );
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

pub fn record_draft_offering(
    route: Res<BotSoakRoute>,
    mut receivers: Query<&mut MessageReceiver<S2CDraftOffering>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            if let Some(card_id) = message.card_ids.first().copied() {
                if route.initial_card_id.load(Ordering::SeqCst) == 0 {
                    route
                        .initial_card_id
                        .store(u64::from(card_id.0), Ordering::SeqCst);
                    tracing::info!(
                        card_id = card_id.0,
                        "bot_soak_trigger: S2CDraftOffering received (initial)"
                    );
                }
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
