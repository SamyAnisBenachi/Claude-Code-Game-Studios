//! Scripted friend-game route systems for the two-client runtime harness.
//!
//! Mirrors the proven driving pattern from
//! `tests/integration/playable_client/full_game_over_route_test.rs` but
//! pared back to a leaner state machine: the harness exits on either an
//! `S2CGameOver` broadcast or a round-count cutoff, whichever comes first.
//!
//! AC9 binding (no optimistic client-side authority): every state mutation
//! observed here happens server-side; the client systems only **emit C2S
//! intents** and **record S2C broadcasts** into shared atomics. No client
//! mirror is mutated outside the message receivers.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use lightyear::prelude::*;
use shared::card::{CardId, ClassId};
use shared::protocol::{
    C2SConfirmClass, C2SCreateRoom, C2SHello, C2SJoinRoom, C2SPlaceBid, C2SPurchaseCard,
    C2SSelectClass, C2SSignalReady, C2SSubmitPlacement, CardSource, GameMode, GameOverReason,
    PlacedCardSubmit, ReliableChannel, RoundPhase as ProtocolRoundPhase, S2CAuctionCard,
    S2CCardAcquired, S2CClassesRevealed, S2CDraftOffering, S2CGameOver, S2CHandshake, S2CJoinAck,
    S2CObjectiveIdentities, S2CPhaseChanged, S2CRoomCreated, S2CSlotUpdated,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientRole {
    Host,
    Joiner,
}

/// Shared cross-app state used to coordinate scripting between the two client
/// `App`s and to observe server-broadcast S2C events. All fields are
/// `Arc`-wrapped atomics or mutexes so they can be cloned across apps.
#[derive(Clone, Resource)]
pub struct RouteState {
    pub host_sent_hello: Arc<AtomicBool>,
    pub joiner_sent_hello: Arc<AtomicBool>,
    pub host_received_handshake: Arc<AtomicBool>,
    pub joiner_received_handshake: Arc<AtomicBool>,
    pub host_player_id: Arc<AtomicU64>,
    pub joiner_player_id: Arc<AtomicU64>,

    pub host_sent_create_room: Arc<AtomicBool>,
    pub host_received_room_created: Arc<AtomicBool>,
    pub room_code: Arc<Mutex<Option<String>>>,
    pub joiner_sent_join_room: Arc<AtomicBool>,
    pub joiner_received_join_ack: Arc<AtomicBool>,
    pub host_received_slot_update: Arc<AtomicBool>,

    pub host_sent_select_class: Arc<AtomicBool>,
    pub host_sent_confirm_class: Arc<AtomicBool>,
    pub joiner_sent_select_class: Arc<AtomicBool>,
    pub joiner_sent_confirm_class: Arc<AtomicBool>,
    pub host_received_classes_revealed: Arc<AtomicBool>,
    pub joiner_received_classes_revealed: Arc<AtomicBool>,

    pub host_purchase_card_id: Arc<AtomicU64>,
    pub joiner_purchase_card_id: Arc<AtomicU64>,
    pub host_sent_purchase_card: Arc<AtomicBool>,
    pub joiner_sent_purchase_card: Arc<AtomicBool>,
    pub host_received_card_acquired: Arc<AtomicBool>,
    pub joiner_received_card_acquired: Arc<AtomicBool>,
    pub host_sent_draft_initial_ready: Arc<AtomicBool>,
    pub joiner_sent_draft_initial_ready: Arc<AtomicBool>,

    pub placement_phase_count: Arc<AtomicUsize>,
    pub draft_shop_phase_count: Arc<AtomicUsize>,
    pub resolution_phase_count: Arc<AtomicUsize>,
    pub auction_phase_count: Arc<AtomicUsize>,
    pub host_placements_sent: Arc<AtomicUsize>,
    pub joiner_placements_sent: Arc<AtomicUsize>,
    pub host_draft_shop_ready_sent: Arc<AtomicUsize>,
    pub joiner_draft_shop_ready_sent: Arc<AtomicUsize>,
    pub host_auction_bid_sent: Arc<AtomicUsize>,
    pub auction_starting_price: Arc<AtomicUsize>,

    pub host_received_game_over: Arc<AtomicBool>,
    pub joiner_received_game_over: Arc<AtomicBool>,
    pub game_over_round: Arc<AtomicUsize>,
    pub game_over_reason_draw: Arc<AtomicBool>,
    pub game_over_loser: Arc<AtomicU64>,

    pub last_observed_phase_host: Arc<Mutex<Option<ProtocolRoundPhase>>>,
    pub last_observed_phase_joiner: Arc<Mutex<Option<ProtocolRoundPhase>>>,
}

const NO_LOSER: u64 = u64::MAX;

impl Default for RouteState {
    fn default() -> Self {
        Self {
            host_sent_hello: Arc::new(AtomicBool::new(false)),
            joiner_sent_hello: Arc::new(AtomicBool::new(false)),
            host_received_handshake: Arc::new(AtomicBool::new(false)),
            joiner_received_handshake: Arc::new(AtomicBool::new(false)),
            host_player_id: Arc::new(AtomicU64::new(0)),
            joiner_player_id: Arc::new(AtomicU64::new(0)),
            host_sent_create_room: Arc::new(AtomicBool::new(false)),
            host_received_room_created: Arc::new(AtomicBool::new(false)),
            room_code: Arc::new(Mutex::new(None)),
            joiner_sent_join_room: Arc::new(AtomicBool::new(false)),
            joiner_received_join_ack: Arc::new(AtomicBool::new(false)),
            host_received_slot_update: Arc::new(AtomicBool::new(false)),
            host_sent_select_class: Arc::new(AtomicBool::new(false)),
            host_sent_confirm_class: Arc::new(AtomicBool::new(false)),
            joiner_sent_select_class: Arc::new(AtomicBool::new(false)),
            joiner_sent_confirm_class: Arc::new(AtomicBool::new(false)),
            host_received_classes_revealed: Arc::new(AtomicBool::new(false)),
            joiner_received_classes_revealed: Arc::new(AtomicBool::new(false)),
            host_purchase_card_id: Arc::new(AtomicU64::new(0)),
            joiner_purchase_card_id: Arc::new(AtomicU64::new(0)),
            host_sent_purchase_card: Arc::new(AtomicBool::new(false)),
            joiner_sent_purchase_card: Arc::new(AtomicBool::new(false)),
            host_received_card_acquired: Arc::new(AtomicBool::new(false)),
            joiner_received_card_acquired: Arc::new(AtomicBool::new(false)),
            host_sent_draft_initial_ready: Arc::new(AtomicBool::new(false)),
            joiner_sent_draft_initial_ready: Arc::new(AtomicBool::new(false)),
            placement_phase_count: Arc::new(AtomicUsize::new(0)),
            draft_shop_phase_count: Arc::new(AtomicUsize::new(0)),
            resolution_phase_count: Arc::new(AtomicUsize::new(0)),
            auction_phase_count: Arc::new(AtomicUsize::new(0)),
            host_placements_sent: Arc::new(AtomicUsize::new(0)),
            joiner_placements_sent: Arc::new(AtomicUsize::new(0)),
            host_draft_shop_ready_sent: Arc::new(AtomicUsize::new(0)),
            joiner_draft_shop_ready_sent: Arc::new(AtomicUsize::new(0)),
            host_auction_bid_sent: Arc::new(AtomicUsize::new(0)),
            auction_starting_price: Arc::new(AtomicUsize::new(0)),
            host_received_game_over: Arc::new(AtomicBool::new(false)),
            joiner_received_game_over: Arc::new(AtomicBool::new(false)),
            game_over_round: Arc::new(AtomicUsize::new(0)),
            game_over_reason_draw: Arc::new(AtomicBool::new(false)),
            game_over_loser: Arc::new(AtomicU64::new(NO_LOSER)),
            last_observed_phase_host: Arc::new(Mutex::new(None)),
            last_observed_phase_joiner: Arc::new(Mutex::new(None)),
        }
    }
}

impl RouteState {
    pub fn both_connected(&self) -> bool {
        self.host_received_handshake.load(Ordering::SeqCst)
            && self.joiner_received_handshake.load(Ordering::SeqCst)
    }

    pub fn either_game_over(&self) -> bool {
        self.host_received_game_over.load(Ordering::SeqCst)
            || self.joiner_received_game_over.load(Ordering::SeqCst)
    }

    pub fn rounds_observed(&self) -> usize {
        self.placement_phase_count.load(Ordering::SeqCst)
    }
}

#[derive(Resource, Clone)]
pub struct RouteProbe {
    pub role: ClientRole,
    pub state: RouteState,
}

pub fn send_hello_until_handshake(
    probe: Res<RouteProbe>,
    mut senders: Query<&mut MessageSender<C2SHello>>,
) {
    let already = match probe.role {
        ClientRole::Host => probe.state.host_received_handshake.load(Ordering::SeqCst),
        ClientRole::Joiner => probe.state.joiner_received_handshake.load(Ordering::SeqCst),
    };
    if already {
        return;
    }

    for mut sender in &mut senders {
        sender.send::<ReliableChannel>(C2SHello {
            protocol_version: shared::config::GameConfig::default().protocol_version,
            session_token: None,
        });
        match probe.role {
            ClientRole::Host => probe.state.host_sent_hello.store(true, Ordering::SeqCst),
            ClientRole::Joiner => probe.state.joiner_sent_hello.store(true, Ordering::SeqCst),
        }
    }
}

pub fn record_handshake(
    probe: Res<RouteProbe>,
    mut receivers: Query<&mut MessageReceiver<S2CHandshake>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            let role_label = match probe.role {
                ClientRole::Host => "host",
                ClientRole::Joiner => "joiner",
            };
            tracing::info!(
                target = "harness::probe",
                role = role_label,
                player_id = message.player_id.0,
                "S2CHandshake received"
            );
            match probe.role {
                ClientRole::Host => {
                    probe
                        .state
                        .host_received_handshake
                        .store(true, Ordering::SeqCst);
                    probe
                        .state
                        .host_player_id
                        .store(message.player_id.0, Ordering::SeqCst);
                }
                ClientRole::Joiner => {
                    probe
                        .state
                        .joiner_received_handshake
                        .store(true, Ordering::SeqCst);
                    probe
                        .state
                        .joiner_player_id
                        .store(message.player_id.0, Ordering::SeqCst);
                }
            }
        }
    }
}

pub fn send_lobby_actions(
    probe: Res<RouteProbe>,
    mut create_room: Query<&mut MessageSender<C2SCreateRoom>>,
    mut join_room: Query<&mut MessageSender<C2SJoinRoom>>,
    mut select_class: Query<&mut MessageSender<C2SSelectClass>>,
    mut confirm_class: Query<&mut MessageSender<C2SConfirmClass>>,
) {
    match probe.role {
        ClientRole::Host => {
            if probe.state.host_received_handshake.load(Ordering::SeqCst)
                && !probe.state.host_sent_create_room.load(Ordering::SeqCst)
            {
                if let Some(mut sender) = create_room.iter_mut().next() {
                    sender.send::<ReliableChannel>(C2SCreateRoom {
                        mode: GameMode::OneVOne,
                    });
                    probe
                        .state
                        .host_sent_create_room
                        .store(true, Ordering::SeqCst);
                }
            }
            if probe.state.host_received_slot_update.load(Ordering::SeqCst) {
                send_class_and_confirm(
                    ClassId::Iop,
                    &probe.state.host_sent_select_class,
                    &probe.state.host_sent_confirm_class,
                    &mut select_class,
                    &mut confirm_class,
                );
            }
        }
        ClientRole::Joiner => {
            if probe.state.joiner_received_handshake.load(Ordering::SeqCst)
                && !probe.state.joiner_sent_join_room.load(Ordering::SeqCst)
            {
                let code = probe
                    .state
                    .room_code
                    .lock()
                    .expect("room_code mutex must not be poisoned")
                    .clone();
                if let (Some(room_code), Some(mut sender)) = (code, join_room.iter_mut().next()) {
                    sender.send::<ReliableChannel>(C2SJoinRoom {
                        room_code,
                        requested_slot: 1,
                    });
                    probe
                        .state
                        .joiner_sent_join_room
                        .store(true, Ordering::SeqCst);
                }
            }
            if probe.state.joiner_received_join_ack.load(Ordering::SeqCst) {
                send_class_and_confirm(
                    ClassId::Cra,
                    &probe.state.joiner_sent_select_class,
                    &probe.state.joiner_sent_confirm_class,
                    &mut select_class,
                    &mut confirm_class,
                );
            }
        }
    }
}

fn send_class_and_confirm(
    class_id: ClassId,
    select_flag: &AtomicBool,
    confirm_flag: &AtomicBool,
    select_class: &mut Query<&mut MessageSender<C2SSelectClass>>,
    confirm_class: &mut Query<&mut MessageSender<C2SConfirmClass>>,
) {
    if !select_flag.load(Ordering::SeqCst) {
        if let Some(mut sender) = select_class.iter_mut().next() {
            sender.send::<ReliableChannel>(C2SSelectClass { class_id });
            select_flag.store(true, Ordering::SeqCst);
        }
    }
    if !confirm_flag.load(Ordering::SeqCst) {
        if let Some(mut sender) = confirm_class.iter_mut().next() {
            sender.send::<ReliableChannel>(C2SConfirmClass { class_id });
            confirm_flag.store(true, Ordering::SeqCst);
        }
    }
}

pub fn send_draft_initial_purchase(
    probe: Res<RouteProbe>,
    mut purchase_card: Query<&mut MessageSender<C2SPurchaseCard>>,
) {
    let (card_id, sent_flag) = match probe.role {
        ClientRole::Host => (
            probe.state.host_purchase_card_id.load(Ordering::SeqCst),
            &probe.state.host_sent_purchase_card,
        ),
        ClientRole::Joiner => (
            probe.state.joiner_purchase_card_id.load(Ordering::SeqCst),
            &probe.state.joiner_sent_purchase_card,
        ),
    };
    if card_id == 0 || sent_flag.load(Ordering::SeqCst) {
        return;
    }
    if let Some(mut sender) = purchase_card.iter_mut().next() {
        sender.send::<ReliableChannel>(C2SPurchaseCard {
            card_id: CardId(card_id as u32),
        });
        sent_flag.store(true, Ordering::SeqCst);
    }
}

pub fn send_draft_initial_ready(
    probe: Res<RouteProbe>,
    mut signal_ready: Query<&mut MessageSender<C2SSignalReady>>,
) {
    let acquired = match probe.role {
        ClientRole::Host => probe
            .state
            .host_received_card_acquired
            .load(Ordering::SeqCst),
        ClientRole::Joiner => probe
            .state
            .joiner_received_card_acquired
            .load(Ordering::SeqCst),
    };
    let sent_flag = match probe.role {
        ClientRole::Host => &probe.state.host_sent_draft_initial_ready,
        ClientRole::Joiner => &probe.state.joiner_sent_draft_initial_ready,
    };
    if !acquired || sent_flag.load(Ordering::SeqCst) {
        return;
    }
    if let Some(mut sender) = signal_ready.iter_mut().next() {
        sender.send::<ReliableChannel>(C2SSignalReady { retract: false });
        sent_flag.store(true, Ordering::SeqCst);
    }
}

pub fn send_loop_actions(
    probe: Res<RouteProbe>,
    mut submit_placement: Query<&mut MessageSender<C2SSubmitPlacement>>,
    mut signal_ready: Query<&mut MessageSender<C2SSignalReady>>,
    mut place_bid: Query<&mut MessageSender<C2SPlaceBid>>,
) {
    let last_phase = match probe.role {
        ClientRole::Host => probe
            .state
            .last_observed_phase_host
            .lock()
            .expect("last_observed_phase_host mutex must not be poisoned")
            .clone(),
        ClientRole::Joiner => probe
            .state
            .last_observed_phase_joiner
            .lock()
            .expect("last_observed_phase_joiner mutex must not be poisoned")
            .clone(),
    };

    let (placements_sent, draft_shop_ready_sent) = match probe.role {
        ClientRole::Host => (
            &probe.state.host_placements_sent,
            &probe.state.host_draft_shop_ready_sent,
        ),
        ClientRole::Joiner => (
            &probe.state.joiner_placements_sent,
            &probe.state.joiner_draft_shop_ready_sent,
        ),
    };
    let placement_count = probe.state.placement_phase_count.load(Ordering::SeqCst);
    let draft_shop_count = probe.state.draft_shop_phase_count.load(Ordering::SeqCst);

    match last_phase {
        Some(ProtocolRoundPhase::Placement) => {
            if placements_sent.load(Ordering::SeqCst) < placement_count {
                if let Some(mut sender) = submit_placement.iter_mut().next() {
                    sender.send::<ReliableChannel>(C2SSubmitPlacement {
                        placements: scripted_placement(probe.role, &probe.state),
                    });
                    placements_sent.store(placement_count, Ordering::SeqCst);
                }
            }
        }
        Some(ProtocolRoundPhase::DraftShop) => {
            if draft_shop_ready_sent.load(Ordering::SeqCst) < draft_shop_count {
                if let Some(mut sender) = signal_ready.iter_mut().next() {
                    sender.send::<ReliableChannel>(C2SSignalReady { retract: false });
                    draft_shop_ready_sent.store(draft_shop_count, Ordering::SeqCst);
                }
            }
        }
        Some(ProtocolRoundPhase::DraftAuction) => {
            if probe.role == ClientRole::Host {
                let auction_count = probe.state.auction_phase_count.load(Ordering::SeqCst);
                let bid_sent = probe.state.host_auction_bid_sent.load(Ordering::SeqCst);
                let starting = probe.state.auction_starting_price.load(Ordering::SeqCst);
                if bid_sent < auction_count && starting > 0 {
                    if let Some(mut sender) = place_bid.iter_mut().next() {
                        sender.send::<ReliableChannel>(C2SPlaceBid {
                            amount: (starting as u32).saturating_add(1),
                        });
                        probe
                            .state
                            .host_auction_bid_sent
                            .store(auction_count, Ordering::SeqCst);
                    }
                }
            }
        }
        _ => {}
    }
}

fn scripted_placement(role: ClientRole, _state: &RouteState) -> Vec<PlacedCardSubmit> {
    // Per the existing full-game-over route harness pattern: empty placement is
    // a valid, accepted C2SSubmitPlacement and is what drives the route through
    // the placement phase on the first loop. Subsequent loops still send empty
    // placements — sufficient to traverse Placement → Resolution → DraftShop.
    // The harness does not optimise for fastest GAME_OVER (deferred follow-on).
    let _ = role;
    Vec::new()
}

pub fn record_s2c_handshake_chain(
    probe: Res<RouteProbe>,
    mut room_created: Query<&mut MessageReceiver<S2CRoomCreated>>,
    mut join_ack: Query<&mut MessageReceiver<S2CJoinAck>>,
    mut slot_updates: Query<&mut MessageReceiver<S2CSlotUpdated>>,
    mut classes_revealed: Query<&mut MessageReceiver<S2CClassesRevealed>>,
) {
    for mut receiver in &mut room_created {
        for message in receiver.receive() {
            if probe.role == ClientRole::Host {
                *probe
                    .state
                    .room_code
                    .lock()
                    .expect("room_code mutex must not be poisoned") =
                    Some(message.room_code.clone());
                probe
                    .state
                    .host_received_room_created
                    .store(true, Ordering::SeqCst);
            }
        }
    }
    for mut receiver in &mut join_ack {
        for _ in receiver.receive() {
            if probe.role == ClientRole::Joiner {
                probe
                    .state
                    .joiner_received_join_ack
                    .store(true, Ordering::SeqCst);
            }
        }
    }
    for mut receiver in &mut slot_updates {
        for _ in receiver.receive() {
            if probe.role == ClientRole::Host {
                probe
                    .state
                    .host_received_slot_update
                    .store(true, Ordering::SeqCst);
            }
        }
    }
    for mut receiver in &mut classes_revealed {
        for _ in receiver.receive() {
            match probe.role {
                ClientRole::Host => probe
                    .state
                    .host_received_classes_revealed
                    .store(true, Ordering::SeqCst),
                ClientRole::Joiner => probe
                    .state
                    .joiner_received_classes_revealed
                    .store(true, Ordering::SeqCst),
            }
        }
    }
}

pub fn record_s2c_draft_and_phase(
    probe: Res<RouteProbe>,
    mut draft_offering: Query<&mut MessageReceiver<S2CDraftOffering>>,
    mut card_acquired: Query<&mut MessageReceiver<S2CCardAcquired>>,
    mut phase_changed: Query<&mut MessageReceiver<S2CPhaseChanged>>,
    mut auction_card: Query<&mut MessageReceiver<S2CAuctionCard>>,
    mut objective_identities: Query<&mut MessageReceiver<S2CObjectiveIdentities>>,
) {
    for mut receiver in &mut draft_offering {
        for message in receiver.receive() {
            if let Some(card_id) = message.card_ids.first().copied() {
                match probe.role {
                    ClientRole::Host => {
                        probe
                            .state
                            .host_purchase_card_id
                            .store(u64::from(card_id.0), Ordering::SeqCst);
                    }
                    ClientRole::Joiner => {
                        probe
                            .state
                            .joiner_purchase_card_id
                            .store(u64::from(card_id.0), Ordering::SeqCst);
                    }
                }
            }
        }
    }
    for mut receiver in &mut card_acquired {
        for message in receiver.receive() {
            if message.source == CardSource::DraftInitial {
                match probe.role {
                    ClientRole::Host => probe
                        .state
                        .host_received_card_acquired
                        .store(true, Ordering::SeqCst),
                    ClientRole::Joiner => probe
                        .state
                        .joiner_received_card_acquired
                        .store(true, Ordering::SeqCst),
                }
            }
        }
    }
    for mut receiver in &mut auction_card {
        for message in receiver.receive() {
            probe
                .state
                .auction_starting_price
                .store(message.starting_price as usize, Ordering::SeqCst);
        }
    }
    for mut receiver in &mut objective_identities {
        for _ in receiver.receive() {
            // Observed but harness ignores the lane mask — the empty-placement
            // path through Placement does not need targeted lanes. The story's
            // AC3 max-round cutoff is the canonical termination path; the
            // GAME_OVER fast-path is a follow-on enhancement.
        }
    }
    for mut receiver in &mut phase_changed {
        for message in receiver.receive() {
            let role_label = match probe.role {
                ClientRole::Host => "host",
                ClientRole::Joiner => "joiner",
            };
            tracing::info!(
                target = "harness::probe",
                role = role_label,
                phase = ?message.phase,
                "S2CPhaseChanged received"
            );
            let store_phase = || match probe.role {
                ClientRole::Host => {
                    *probe
                        .state
                        .last_observed_phase_host
                        .lock()
                        .expect("last_observed_phase_host mutex must not be poisoned") =
                        Some(message.phase)
                }
                ClientRole::Joiner => {
                    *probe
                        .state
                        .last_observed_phase_joiner
                        .lock()
                        .expect("last_observed_phase_joiner mutex must not be poisoned") =
                        Some(message.phase)
                }
            };
            store_phase();
            match message.phase {
                ProtocolRoundPhase::Placement => {
                    if probe.role == ClientRole::Host {
                        probe
                            .state
                            .placement_phase_count
                            .fetch_add(1, Ordering::SeqCst);
                    }
                }
                ProtocolRoundPhase::Resolution => {
                    if probe.role == ClientRole::Host {
                        probe
                            .state
                            .resolution_phase_count
                            .fetch_add(1, Ordering::SeqCst);
                    }
                }
                ProtocolRoundPhase::DraftShop => {
                    if probe.role == ClientRole::Host {
                        probe
                            .state
                            .draft_shop_phase_count
                            .fetch_add(1, Ordering::SeqCst);
                    }
                }
                ProtocolRoundPhase::DraftAuction => {
                    if probe.role == ClientRole::Host {
                        probe
                            .state
                            .auction_phase_count
                            .fetch_add(1, Ordering::SeqCst);
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn record_game_over(
    probe: Res<RouteProbe>,
    mut game_over: Query<&mut MessageReceiver<S2CGameOver>>,
) {
    for mut receiver in &mut game_over {
        for message in receiver.receive() {
            probe
                .state
                .game_over_round
                .store(message.round as usize, Ordering::SeqCst);
            probe.state.game_over_loser.store(
                message.loser.map_or(NO_LOSER, |player| player.0),
                Ordering::SeqCst,
            );
            if matches!(message.reason, GameOverReason::Draw) {
                probe
                    .state
                    .game_over_reason_draw
                    .store(true, Ordering::SeqCst);
            }
            match probe.role {
                ClientRole::Host => probe
                    .state
                    .host_received_game_over
                    .store(true, Ordering::SeqCst),
                ClientRole::Joiner => probe
                    .state
                    .joiner_received_game_over
                    .store(true, Ordering::SeqCst),
            }
        }
    }
}
