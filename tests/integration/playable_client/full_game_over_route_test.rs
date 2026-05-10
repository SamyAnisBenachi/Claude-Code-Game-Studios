#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

extern crate server as server_crate;

use bevy::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use server_crate::core::objective_contract::ObjectiveCounters;
use server_crate::core::rsm::{
    advance_phase, rsm_input_reader, RoundPhase as ServerRoundPhase, RoundState, RsmPlugin,
};
use server_crate::core::session::{
    EndedSessionResultState, GameSessionPlugin, LobbyState, PlayerConnectionMap, RoomSessions,
    ServerRngFactory, ServerRngInitError, SessionConfig,
};
use server_crate::core::{economy::EconomyPlugin, pool::CardPoolPlugin};
use server_crate::feature::{
    acquisition::{CardAcquisitionPlugin, PlayerHands},
    auction::AuctionPlugin,
    board::{BoardPlugin, PendingPlacements},
    combat::CombatPlugin,
    objective::ObjectivePlugin,
};
use server_crate::foundation::{config::CardCatalog, rng::ServerRng};
use server_crate::network::{
    drain_signal_ready_messages, drain_submit_placement_messages,
    economy_dispatch::EconomyNetworkPlugin, register_lightyear_protocol,
    rsm_dispatch::dispatch_phase_changed,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{
    C2SAcknowledgeResult, C2SConfirmClass, C2SCreateRoom, C2SHello, C2SJoinRoom, C2SPlaceBid,
    C2SPurchaseCard, C2SSelectClass, C2SSignalReady, C2SSubmitPlacement, CardSource, GameMode,
    GameOverReason, PlacedCardSubmit, PlayTarget, ReliableChannel, ResolutionEvent,
    RoundPhase as ProtocolRoundPhase, S2CAuctionBidAccepted, S2CAuctionBidRejected, S2CAuctionCard,
    S2CAuctionSettled, S2CCardAcquired, S2CClassLocked, S2CClassesRevealed, S2CDraftOffering,
    S2CGameOver, S2CGoldUpdate, S2CHandshake, S2CJoinAck, S2CObjectiveIdentities, S2CPhaseChanged,
    S2CPlacementReveal, S2CResolutionEvent, S2CRoomCreated, S2CSlotUpdated,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const TICK_HZ: f64 = 60.0;
const ROOM_FLOW_MAX_FRAMES: usize = 5_400;
const FRAME_SLEEP: Duration = Duration::from_millis(10);

/// Full end-to-end route: lobby create → both clients join → class confirm →
/// DRAFT_INITIAL → DRAFT_SHOP → DRAFT_AUCTION → placement → resolution →
/// GAME_OVER → both clients send C2SAcknowledgeResult → server clears EndedSessionResultState.
///
/// This is the automated leg of the S9-QA-001 hybrid evidence plan.
/// The visual leg (result screen renders, Return to Lobby button) is covered by MANUAL-FG-001.
#[test]
fn full_game_over_route_including_acknowledgement_handshake() {
    test_helpers::init_test_tracing();
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");
    let flags = RoomSessionFlags::new();

    let mut server_app = build_room_flow_server_app(port, flags.clone());
    for _ in 0..30 {
        server_app.update();
        thread::sleep(FRAME_SLEEP);
    }

    let mut host_app = build_room_flow_client_app(url.clone(), ClientRole::Host, flags.clone());
    let mut joiner_app = build_room_flow_client_app(url, ClientRole::Joiner, flags.clone());
    for _ in 0..ROOM_FLOW_MAX_FRAMES {
        server_app.update();
        host_app.update();
        joiner_app.update();

        if flags.full_route_complete() {
            break;
        }

        thread::sleep(FRAME_SLEEP);
    }

    let evidence_dir = workspace_root()
        .join("target")
        .join("test-evidence")
        .join("e2e-game-over");
    fs::create_dir_all(&evidence_dir)
        .expect("test-evidence directory should be creatable under target/");
    let log_path = evidence_dir.join("run.log");
    let log = format!(
        "=== S9-QA-001 E2E GAME_OVER Route — automated log ===\n\
         host_received_game_over: {}\n\
         joiner_received_game_over: {}\n\
         host_sent_acknowledge: {}\n\
         joiner_sent_acknowledge: {}\n\
         server_ended_state_cleared: {}\n\
         server_round_game_over: {}\n\
         server_lobby_game_over: {}\n\
         server_session_config_removed: {}\n\
         server_rng_removed: {}\n\
         game_over_loser: {}\n\
         game_over_reason_draw: {}\n\
         full_route: {}\n\
         route_detail: {}\n",
        flags.host_received_game_over.load(Ordering::SeqCst),
        flags.joiner_received_game_over.load(Ordering::SeqCst),
        flags.host_sent_acknowledge.load(Ordering::SeqCst),
        flags.joiner_sent_acknowledge.load(Ordering::SeqCst),
        flags.server_ended_state_cleared.load(Ordering::SeqCst),
        flags.server_round_game_over.load(Ordering::SeqCst),
        flags.server_lobby_game_over.load(Ordering::SeqCst),
        flags.server_session_config_removed.load(Ordering::SeqCst),
        flags.server_rng_removed.load(Ordering::SeqCst),
        flags.game_over_loser.load(Ordering::SeqCst),
        flags.game_over_reason_draw.load(Ordering::SeqCst),
        flags.full_route_complete(),
        flags.report(),
    );
    fs::write(&log_path, &log).expect("test evidence log should be writable");

    assert!(
        flags.host_received_game_over.load(Ordering::SeqCst)
            && flags.joiner_received_game_over.load(Ordering::SeqCst),
        "both clients should receive authoritative S2CGameOver: {}",
        flags.report()
    );
    assert!(
        flags.host_received_game_over_phase.load(Ordering::SeqCst)
            && flags.joiner_received_game_over_phase.load(Ordering::SeqCst),
        "both clients should receive S2CPhaseChanged(GameOver): {}",
        flags.report()
    );
    assert!(
        flags.server_round_game_over.load(Ordering::SeqCst)
            && flags.server_lobby_game_over.load(Ordering::SeqCst)
            && flags.server_session_config_removed.load(Ordering::SeqCst)
            && flags.server_rng_removed.load(Ordering::SeqCst),
        "server should tear down session resources after game-over: {}",
        flags.report()
    );
    assert!(
        flags.host_sent_acknowledge.load(Ordering::SeqCst),
        "host client should send C2SAcknowledgeResult after receiving S2CGameOver: {}",
        flags.report()
    );
    assert!(
        flags.joiner_sent_acknowledge.load(Ordering::SeqCst),
        "joiner client should send C2SAcknowledgeResult after receiving S2CGameOver: {}",
        flags.report()
    );
    assert!(
        flags.server_ended_state_cleared.load(Ordering::SeqCst),
        "server EndedSessionResultState should be removed after both clients acknowledge: {}",
        flags.report()
    );
}

const NO_GAME_OVER_LOSER: u64 = u64::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClientRole {
    Host,
    Joiner,
}

#[derive(Clone, Resource)]
struct RoomSessionFlags {
    host_sent_hello: Arc<AtomicBool>,
    joiner_sent_hello: Arc<AtomicBool>,
    host_received_handshake: Arc<AtomicBool>,
    joiner_received_handshake: Arc<AtomicBool>,
    host_player_id: Arc<AtomicU64>,
    joiner_player_id: Arc<AtomicU64>,
    server_connection_count: Arc<AtomicUsize>,
    server_room_count: Arc<AtomicUsize>,
    host_sent_create_room: Arc<AtomicBool>,
    host_received_room_created: Arc<AtomicBool>,
    joiner_sent_join_room: Arc<AtomicBool>,
    joiner_received_join_ack: Arc<AtomicBool>,
    host_received_slot_update: Arc<AtomicBool>,
    host_sent_select_class: Arc<AtomicBool>,
    joiner_sent_select_class: Arc<AtomicBool>,
    host_sent_confirm_class: Arc<AtomicBool>,
    joiner_sent_confirm_class: Arc<AtomicBool>,
    host_received_class_locked: Arc<AtomicBool>,
    joiner_received_class_locked: Arc<AtomicBool>,
    host_received_classes_revealed: Arc<AtomicBool>,
    joiner_received_classes_revealed: Arc<AtomicBool>,
    reveal_player_count: Arc<AtomicUsize>,
    server_room_game_active: Arc<AtomicBool>,
    server_round_draft_initial: Arc<AtomicBool>,
    host_received_draft_initial: Arc<AtomicBool>,
    joiner_received_draft_initial: Arc<AtomicBool>,
    host_received_draft_offering: Arc<AtomicBool>,
    joiner_received_draft_offering: Arc<AtomicBool>,
    host_draft_offering_count: Arc<AtomicUsize>,
    joiner_draft_offering_count: Arc<AtomicUsize>,
    host_purchase_card_id: Arc<AtomicU64>,
    joiner_purchase_card_id: Arc<AtomicU64>,
    host_sent_purchase_card: Arc<AtomicBool>,
    joiner_sent_purchase_card: Arc<AtomicBool>,
    host_received_card_acquired: Arc<AtomicBool>,
    joiner_received_card_acquired: Arc<AtomicBool>,
    host_card_acquired_source_draft_initial: Arc<AtomicBool>,
    joiner_card_acquired_source_draft_initial: Arc<AtomicBool>,
    host_received_purchase_gold_update: Arc<AtomicBool>,
    joiner_received_purchase_gold_update: Arc<AtomicBool>,
    server_host_hand_size: Arc<AtomicUsize>,
    server_joiner_hand_size: Arc<AtomicUsize>,
    server_host_ready_seen: Arc<AtomicBool>,
    host_sent_ready_initial: Arc<AtomicBool>,
    host_sent_ready_retract: Arc<AtomicBool>,
    host_sent_ready_final: Arc<AtomicBool>,
    joiner_sent_ready: Arc<AtomicBool>,
    server_retract_path_observed: Arc<AtomicBool>,
    server_round_placement: Arc<AtomicBool>,
    host_received_placement: Arc<AtomicBool>,
    joiner_received_placement: Arc<AtomicBool>,
    host_placement_phase_count: Arc<AtomicUsize>,
    joiner_placement_phase_count: Arc<AtomicUsize>,
    host_sent_placement_submit: Arc<AtomicBool>,
    joiner_sent_placement_submit: Arc<AtomicBool>,
    host_sent_non_empty_placement_submit: Arc<AtomicBool>,
    joiner_sent_non_empty_placement_submit: Arc<AtomicBool>,
    host_sent_post_auction_placement_submit: Arc<AtomicBool>,
    joiner_sent_post_auction_placement_submit: Arc<AtomicBool>,
    server_host_non_empty_placement_accepted: Arc<AtomicBool>,
    server_joiner_non_empty_placement_accepted: Arc<AtomicBool>,
    server_round_resolution: Arc<AtomicBool>,
    host_received_resolution: Arc<AtomicBool>,
    joiner_received_resolution: Arc<AtomicBool>,
    host_resolution_phase_count: Arc<AtomicUsize>,
    joiner_resolution_phase_count: Arc<AtomicUsize>,
    host_received_placement_reveal: Arc<AtomicBool>,
    joiner_received_placement_reveal: Arc<AtomicBool>,
    host_received_non_empty_placement_reveal: Arc<AtomicBool>,
    joiner_received_non_empty_placement_reveal: Arc<AtomicBool>,
    host_non_empty_placement_reveal_count: Arc<AtomicUsize>,
    joiner_non_empty_placement_reveal_count: Arc<AtomicUsize>,
    host_received_resolution_event: Arc<AtomicBool>,
    joiner_received_resolution_event: Arc<AtomicBool>,
    host_resolution_event_count: Arc<AtomicUsize>,
    joiner_resolution_event_count: Arc<AtomicUsize>,
    host_received_unit_placed_resolution_event: Arc<AtomicBool>,
    joiner_received_unit_placed_resolution_event: Arc<AtomicBool>,
    server_round_draft_auction: Arc<AtomicBool>,
    host_received_draft_auction: Arc<AtomicBool>,
    joiner_received_draft_auction: Arc<AtomicBool>,
    host_received_auction_card: Arc<AtomicBool>,
    joiner_received_auction_card: Arc<AtomicBool>,
    auction_starting_price: Arc<AtomicUsize>,
    host_sent_auction_bid: Arc<AtomicBool>,
    host_received_auction_bid_accepted: Arc<AtomicBool>,
    joiner_received_auction_bid_accepted: Arc<AtomicBool>,
    host_received_auction_bid_rejected: Arc<AtomicBool>,
    joiner_received_auction_bid_rejected: Arc<AtomicBool>,
    host_received_auction_settled: Arc<AtomicBool>,
    joiner_received_auction_settled: Arc<AtomicBool>,
    host_received_card_acquired_source_auction: Arc<AtomicBool>,
    joiner_received_card_acquired_source_auction: Arc<AtomicBool>,
    server_round_draft_shop: Arc<AtomicBool>,
    host_received_draft_shop: Arc<AtomicBool>,
    joiner_received_draft_shop: Arc<AtomicBool>,
    host_draft_shop_phase_count: Arc<AtomicUsize>,
    joiner_draft_shop_phase_count: Arc<AtomicUsize>,
    host_sent_draft_shop_ready: Arc<AtomicBool>,
    joiner_sent_draft_shop_ready: Arc<AtomicBool>,
    host_sent_post_auction_draft_shop_ready: Arc<AtomicBool>,
    joiner_sent_post_auction_draft_shop_ready: Arc<AtomicBool>,
    host_sent_result_endpoint_draft_shop_ready: Arc<AtomicBool>,
    joiner_sent_result_endpoint_draft_shop_ready: Arc<AtomicBool>,
    host_sent_result_endpoint_placement_submit: Arc<AtomicBool>,
    joiner_sent_result_endpoint_placement_submit: Arc<AtomicBool>,
    host_received_objective_identities: Arc<AtomicBool>,
    joiner_received_objective_identities: Arc<AtomicBool>,
    host_real_lane_mask: Arc<AtomicUsize>,
    joiner_real_lane_mask: Arc<AtomicUsize>,
    host_received_objective_destroyed_resolution_event: Arc<AtomicBool>,
    joiner_received_objective_destroyed_resolution_event: Arc<AtomicBool>,
    host_received_game_over: Arc<AtomicBool>,
    joiner_received_game_over: Arc<AtomicBool>,
    host_received_game_over_phase: Arc<AtomicBool>,
    joiner_received_game_over_phase: Arc<AtomicBool>,
    host_game_over_phase_after_resolution: Arc<AtomicBool>,
    joiner_game_over_phase_after_resolution: Arc<AtomicBool>,
    game_over_loser: Arc<AtomicU64>,
    game_over_round: Arc<AtomicUsize>,
    game_over_reason_draw: Arc<AtomicBool>,
    server_round_game_over: Arc<AtomicBool>,
    server_lobby_game_over: Arc<AtomicBool>,
    server_session_config_removed: Arc<AtomicBool>,
    server_rng_removed: Arc<AtomicBool>,
    server_host_real_destroyed: Arc<AtomicUsize>,
    server_joiner_real_destroyed: Arc<AtomicUsize>,
    // Acknowledgement handshake — new for this test
    host_sent_acknowledge: Arc<AtomicBool>,
    joiner_sent_acknowledge: Arc<AtomicBool>,
    server_ended_state_cleared: Arc<AtomicBool>,
    room_code: Arc<Mutex<Option<String>>>,
}

impl RoomSessionFlags {
    fn new() -> Self {
        Self {
            host_sent_hello: Arc::new(AtomicBool::new(false)),
            joiner_sent_hello: Arc::new(AtomicBool::new(false)),
            host_received_handshake: Arc::new(AtomicBool::new(false)),
            joiner_received_handshake: Arc::new(AtomicBool::new(false)),
            host_player_id: Arc::new(AtomicU64::new(0)),
            joiner_player_id: Arc::new(AtomicU64::new(0)),
            server_connection_count: Arc::new(AtomicUsize::new(0)),
            server_room_count: Arc::new(AtomicUsize::new(0)),
            host_sent_create_room: Arc::new(AtomicBool::new(false)),
            host_received_room_created: Arc::new(AtomicBool::new(false)),
            joiner_sent_join_room: Arc::new(AtomicBool::new(false)),
            joiner_received_join_ack: Arc::new(AtomicBool::new(false)),
            host_received_slot_update: Arc::new(AtomicBool::new(false)),
            host_sent_select_class: Arc::new(AtomicBool::new(false)),
            joiner_sent_select_class: Arc::new(AtomicBool::new(false)),
            host_sent_confirm_class: Arc::new(AtomicBool::new(false)),
            joiner_sent_confirm_class: Arc::new(AtomicBool::new(false)),
            host_received_class_locked: Arc::new(AtomicBool::new(false)),
            joiner_received_class_locked: Arc::new(AtomicBool::new(false)),
            host_received_classes_revealed: Arc::new(AtomicBool::new(false)),
            joiner_received_classes_revealed: Arc::new(AtomicBool::new(false)),
            reveal_player_count: Arc::new(AtomicUsize::new(0)),
            server_room_game_active: Arc::new(AtomicBool::new(false)),
            server_round_draft_initial: Arc::new(AtomicBool::new(false)),
            host_received_draft_initial: Arc::new(AtomicBool::new(false)),
            joiner_received_draft_initial: Arc::new(AtomicBool::new(false)),
            host_received_draft_offering: Arc::new(AtomicBool::new(false)),
            joiner_received_draft_offering: Arc::new(AtomicBool::new(false)),
            host_draft_offering_count: Arc::new(AtomicUsize::new(0)),
            joiner_draft_offering_count: Arc::new(AtomicUsize::new(0)),
            host_purchase_card_id: Arc::new(AtomicU64::new(0)),
            joiner_purchase_card_id: Arc::new(AtomicU64::new(0)),
            host_sent_purchase_card: Arc::new(AtomicBool::new(false)),
            joiner_sent_purchase_card: Arc::new(AtomicBool::new(false)),
            host_received_card_acquired: Arc::new(AtomicBool::new(false)),
            joiner_received_card_acquired: Arc::new(AtomicBool::new(false)),
            host_card_acquired_source_draft_initial: Arc::new(AtomicBool::new(false)),
            joiner_card_acquired_source_draft_initial: Arc::new(AtomicBool::new(false)),
            host_received_purchase_gold_update: Arc::new(AtomicBool::new(false)),
            joiner_received_purchase_gold_update: Arc::new(AtomicBool::new(false)),
            server_host_hand_size: Arc::new(AtomicUsize::new(0)),
            server_joiner_hand_size: Arc::new(AtomicUsize::new(0)),
            server_host_ready_seen: Arc::new(AtomicBool::new(false)),
            host_sent_ready_initial: Arc::new(AtomicBool::new(false)),
            host_sent_ready_retract: Arc::new(AtomicBool::new(false)),
            host_sent_ready_final: Arc::new(AtomicBool::new(false)),
            joiner_sent_ready: Arc::new(AtomicBool::new(false)),
            server_retract_path_observed: Arc::new(AtomicBool::new(false)),
            server_round_placement: Arc::new(AtomicBool::new(false)),
            host_received_placement: Arc::new(AtomicBool::new(false)),
            joiner_received_placement: Arc::new(AtomicBool::new(false)),
            host_placement_phase_count: Arc::new(AtomicUsize::new(0)),
            joiner_placement_phase_count: Arc::new(AtomicUsize::new(0)),
            host_sent_placement_submit: Arc::new(AtomicBool::new(false)),
            joiner_sent_placement_submit: Arc::new(AtomicBool::new(false)),
            host_sent_non_empty_placement_submit: Arc::new(AtomicBool::new(false)),
            joiner_sent_non_empty_placement_submit: Arc::new(AtomicBool::new(false)),
            host_sent_post_auction_placement_submit: Arc::new(AtomicBool::new(false)),
            joiner_sent_post_auction_placement_submit: Arc::new(AtomicBool::new(false)),
            server_host_non_empty_placement_accepted: Arc::new(AtomicBool::new(false)),
            server_joiner_non_empty_placement_accepted: Arc::new(AtomicBool::new(false)),
            server_round_resolution: Arc::new(AtomicBool::new(false)),
            host_received_resolution: Arc::new(AtomicBool::new(false)),
            joiner_received_resolution: Arc::new(AtomicBool::new(false)),
            host_resolution_phase_count: Arc::new(AtomicUsize::new(0)),
            joiner_resolution_phase_count: Arc::new(AtomicUsize::new(0)),
            host_received_placement_reveal: Arc::new(AtomicBool::new(false)),
            joiner_received_placement_reveal: Arc::new(AtomicBool::new(false)),
            host_received_non_empty_placement_reveal: Arc::new(AtomicBool::new(false)),
            joiner_received_non_empty_placement_reveal: Arc::new(AtomicBool::new(false)),
            host_non_empty_placement_reveal_count: Arc::new(AtomicUsize::new(0)),
            joiner_non_empty_placement_reveal_count: Arc::new(AtomicUsize::new(0)),
            host_received_resolution_event: Arc::new(AtomicBool::new(false)),
            joiner_received_resolution_event: Arc::new(AtomicBool::new(false)),
            host_resolution_event_count: Arc::new(AtomicUsize::new(0)),
            joiner_resolution_event_count: Arc::new(AtomicUsize::new(0)),
            host_received_unit_placed_resolution_event: Arc::new(AtomicBool::new(false)),
            joiner_received_unit_placed_resolution_event: Arc::new(AtomicBool::new(false)),
            server_round_draft_auction: Arc::new(AtomicBool::new(false)),
            host_received_draft_auction: Arc::new(AtomicBool::new(false)),
            joiner_received_draft_auction: Arc::new(AtomicBool::new(false)),
            host_received_auction_card: Arc::new(AtomicBool::new(false)),
            joiner_received_auction_card: Arc::new(AtomicBool::new(false)),
            auction_starting_price: Arc::new(AtomicUsize::new(0)),
            host_sent_auction_bid: Arc::new(AtomicBool::new(false)),
            host_received_auction_bid_accepted: Arc::new(AtomicBool::new(false)),
            joiner_received_auction_bid_accepted: Arc::new(AtomicBool::new(false)),
            host_received_auction_bid_rejected: Arc::new(AtomicBool::new(false)),
            joiner_received_auction_bid_rejected: Arc::new(AtomicBool::new(false)),
            host_received_auction_settled: Arc::new(AtomicBool::new(false)),
            joiner_received_auction_settled: Arc::new(AtomicBool::new(false)),
            host_received_card_acquired_source_auction: Arc::new(AtomicBool::new(false)),
            joiner_received_card_acquired_source_auction: Arc::new(AtomicBool::new(false)),
            server_round_draft_shop: Arc::new(AtomicBool::new(false)),
            host_received_draft_shop: Arc::new(AtomicBool::new(false)),
            joiner_received_draft_shop: Arc::new(AtomicBool::new(false)),
            host_draft_shop_phase_count: Arc::new(AtomicUsize::new(0)),
            joiner_draft_shop_phase_count: Arc::new(AtomicUsize::new(0)),
            host_sent_draft_shop_ready: Arc::new(AtomicBool::new(false)),
            joiner_sent_draft_shop_ready: Arc::new(AtomicBool::new(false)),
            host_sent_post_auction_draft_shop_ready: Arc::new(AtomicBool::new(false)),
            joiner_sent_post_auction_draft_shop_ready: Arc::new(AtomicBool::new(false)),
            host_sent_result_endpoint_draft_shop_ready: Arc::new(AtomicBool::new(false)),
            joiner_sent_result_endpoint_draft_shop_ready: Arc::new(AtomicBool::new(false)),
            host_sent_result_endpoint_placement_submit: Arc::new(AtomicBool::new(false)),
            joiner_sent_result_endpoint_placement_submit: Arc::new(AtomicBool::new(false)),
            host_received_objective_identities: Arc::new(AtomicBool::new(false)),
            joiner_received_objective_identities: Arc::new(AtomicBool::new(false)),
            host_real_lane_mask: Arc::new(AtomicUsize::new(0)),
            joiner_real_lane_mask: Arc::new(AtomicUsize::new(0)),
            host_received_objective_destroyed_resolution_event: Arc::new(AtomicBool::new(false)),
            joiner_received_objective_destroyed_resolution_event: Arc::new(AtomicBool::new(false)),
            host_received_game_over: Arc::new(AtomicBool::new(false)),
            joiner_received_game_over: Arc::new(AtomicBool::new(false)),
            host_received_game_over_phase: Arc::new(AtomicBool::new(false)),
            joiner_received_game_over_phase: Arc::new(AtomicBool::new(false)),
            host_game_over_phase_after_resolution: Arc::new(AtomicBool::new(false)),
            joiner_game_over_phase_after_resolution: Arc::new(AtomicBool::new(false)),
            game_over_loser: Arc::new(AtomicU64::new(NO_GAME_OVER_LOSER)),
            game_over_round: Arc::new(AtomicUsize::new(0)),
            game_over_reason_draw: Arc::new(AtomicBool::new(false)),
            server_round_game_over: Arc::new(AtomicBool::new(false)),
            server_lobby_game_over: Arc::new(AtomicBool::new(false)),
            server_session_config_removed: Arc::new(AtomicBool::new(false)),
            server_rng_removed: Arc::new(AtomicBool::new(false)),
            server_host_real_destroyed: Arc::new(AtomicUsize::new(0)),
            server_joiner_real_destroyed: Arc::new(AtomicUsize::new(0)),
            host_sent_acknowledge: Arc::new(AtomicBool::new(false)),
            joiner_sent_acknowledge: Arc::new(AtomicBool::new(false)),
            server_ended_state_cleared: Arc::new(AtomicBool::new(false)),
            room_code: Arc::new(Mutex::new(None)),
        }
    }

    fn room_code(&self) -> Option<String> {
        self.room_code
            .lock()
            .expect("room code capture lock should not be poisoned")
            .clone()
    }

    fn session_entry_observed(&self) -> bool {
        self.host_received_classes_revealed.load(Ordering::SeqCst)
            && self.joiner_received_classes_revealed.load(Ordering::SeqCst)
            && self.server_round_draft_initial.load(Ordering::SeqCst)
            && self.host_received_draft_initial.load(Ordering::SeqCst)
            && self.joiner_received_draft_initial.load(Ordering::SeqCst)
    }

    fn draft_shop_observed(&self) -> bool {
        self.session_entry_observed()
            && self.host_received_draft_offering.load(Ordering::SeqCst)
            && self.joiner_received_draft_offering.load(Ordering::SeqCst)
            && self.host_sent_purchase_card.load(Ordering::SeqCst)
            && self.joiner_sent_purchase_card.load(Ordering::SeqCst)
            && self.host_received_card_acquired.load(Ordering::SeqCst)
            && self.joiner_received_card_acquired.load(Ordering::SeqCst)
            && self
                .host_received_purchase_gold_update
                .load(Ordering::SeqCst)
            && self
                .joiner_received_purchase_gold_update
                .load(Ordering::SeqCst)
            && self.host_sent_ready_retract.load(Ordering::SeqCst)
            && self.server_retract_path_observed.load(Ordering::SeqCst)
            && self.host_sent_ready_final.load(Ordering::SeqCst)
            && self.joiner_sent_ready.load(Ordering::SeqCst)
            && self.host_sent_placement_submit.load(Ordering::SeqCst)
            && self.joiner_sent_placement_submit.load(Ordering::SeqCst)
            && self.server_round_draft_shop.load(Ordering::SeqCst)
            && self.host_received_draft_shop.load(Ordering::SeqCst)
            && self.joiner_received_draft_shop.load(Ordering::SeqCst)
    }

    fn sprint7_endpoint_reproduced(&self) -> bool {
        self.draft_shop_observed()
            && self.host_sent_draft_shop_ready.load(Ordering::SeqCst)
            && self.joiner_sent_draft_shop_ready.load(Ordering::SeqCst)
            && self
                .host_sent_non_empty_placement_submit
                .load(Ordering::SeqCst)
            && self
                .joiner_sent_non_empty_placement_submit
                .load(Ordering::SeqCst)
            && self
                .host_received_non_empty_placement_reveal
                .load(Ordering::SeqCst)
            && self
                .joiner_received_non_empty_placement_reveal
                .load(Ordering::SeqCst)
            && self
                .host_non_empty_placement_reveal_count
                .load(Ordering::SeqCst)
                >= 2
            && self
                .joiner_non_empty_placement_reveal_count
                .load(Ordering::SeqCst)
                >= 2
            && self
                .server_host_non_empty_placement_accepted
                .load(Ordering::SeqCst)
            && self
                .server_joiner_non_empty_placement_accepted
                .load(Ordering::SeqCst)
            && self.server_round_draft_auction.load(Ordering::SeqCst)
            && self.host_received_draft_auction.load(Ordering::SeqCst)
            && self.joiner_received_draft_auction.load(Ordering::SeqCst)
            && self.host_received_auction_card.load(Ordering::SeqCst)
            && self.joiner_received_auction_card.load(Ordering::SeqCst)
            && self.host_sent_auction_bid.load(Ordering::SeqCst)
            && self
                .host_received_auction_bid_accepted
                .load(Ordering::SeqCst)
            && self
                .joiner_received_auction_bid_accepted
                .load(Ordering::SeqCst)
            && self.host_received_auction_settled.load(Ordering::SeqCst)
            && self.joiner_received_auction_settled.load(Ordering::SeqCst)
            && self
                .host_received_card_acquired_source_auction
                .load(Ordering::SeqCst)
            && self.host_draft_shop_phase_count.load(Ordering::SeqCst) >= 3
            && self.joiner_draft_shop_phase_count.load(Ordering::SeqCst) >= 3
            && self
                .host_sent_post_auction_draft_shop_ready
                .load(Ordering::SeqCst)
            && self
                .joiner_sent_post_auction_draft_shop_ready
                .load(Ordering::SeqCst)
            && self
                .host_sent_post_auction_placement_submit
                .load(Ordering::SeqCst)
            && self
                .joiner_sent_post_auction_placement_submit
                .load(Ordering::SeqCst)
    }

    fn game_over_result_observed(&self) -> bool {
        self.sprint7_endpoint_reproduced()
            && self
                .host_sent_result_endpoint_draft_shop_ready
                .load(Ordering::SeqCst)
            && self
                .joiner_sent_result_endpoint_draft_shop_ready
                .load(Ordering::SeqCst)
            && self
                .host_sent_result_endpoint_placement_submit
                .load(Ordering::SeqCst)
            && self
                .joiner_sent_result_endpoint_placement_submit
                .load(Ordering::SeqCst)
            && self
                .host_received_objective_destroyed_resolution_event
                .load(Ordering::SeqCst)
            && self
                .joiner_received_objective_destroyed_resolution_event
                .load(Ordering::SeqCst)
            && self.host_received_game_over.load(Ordering::SeqCst)
            && self.joiner_received_game_over.load(Ordering::SeqCst)
            && self.host_received_game_over_phase.load(Ordering::SeqCst)
            && self.joiner_received_game_over_phase.load(Ordering::SeqCst)
            && self
                .host_game_over_phase_after_resolution
                .load(Ordering::SeqCst)
            && self
                .joiner_game_over_phase_after_resolution
                .load(Ordering::SeqCst)
            && self.server_round_game_over.load(Ordering::SeqCst)
            && self.server_lobby_game_over.load(Ordering::SeqCst)
            && self.server_session_config_removed.load(Ordering::SeqCst)
            && self.server_rng_removed.load(Ordering::SeqCst)
    }

    fn full_route_complete(&self) -> bool {
        self.game_over_result_observed()
            && self.host_sent_acknowledge.load(Ordering::SeqCst)
            && self.joiner_sent_acknowledge.load(Ordering::SeqCst)
            && self.server_ended_state_cleared.load(Ordering::SeqCst)
    }

    fn report(&self) -> String {
        format!(
            "host_hello={}, joiner_hello={}, host_hs={}, joiner_hs={}, host_pid={}, joiner_pid={}, \
             conns={}, rooms={}, host_create={}, host_room_created={}, room_code={}, \
             joiner_join={}, joiner_ack={}, host_slot={}, host_select={}, joiner_select={}, \
             host_confirm={}, joiner_confirm={}, host_locked={}, joiner_locked={}, \
             host_revealed={}, joiner_revealed={}, reveal_count={}, server_active={}, \
             server_draft_initial={}, host_draft_initial={}, joiner_draft_initial={}, \
             host_offering={}, joiner_offering={}, host_purchase_id={}, joiner_purchase_id={}, \
             host_purchase={}, joiner_purchase={}, host_acquired={}, joiner_acquired={}, \
             host_gold_update={}, joiner_gold_update={}, \
             host_ready_init={}, host_ready_retract={}, host_ready_final={}, joiner_ready={}, \
             retract_observed={}, server_placement={}, host_placement={}, joiner_placement={}, \
             host_placement_count={}, joiner_placement_count={}, \
             host_submit={}, joiner_submit={}, host_non_empty_submit={}, joiner_non_empty_submit={}, \
             host_post_auction_submit={}, joiner_post_auction_submit={}, \
             server_host_accepted={}, server_joiner_accepted={}, \
             server_resolution={}, host_resolution={}, joiner_resolution={}, \
             host_resolution_count={}, joiner_resolution_count={}, \
             host_non_empty_reveal={}, joiner_non_empty_reveal={}, \
             host_non_empty_reveal_count={}, joiner_non_empty_reveal_count={}, \
             host_resolution_event={}, joiner_resolution_event={}, \
             host_resolution_event_count={}, joiner_resolution_event_count={}, \
             host_unit_placed={}, joiner_unit_placed={}, \
             server_auction={}, host_auction={}, joiner_auction={}, \
             host_auction_card={}, joiner_auction_card={}, auction_price={}, \
             host_bid_sent={}, host_bid_accepted={}, joiner_bid_accepted={}, \
             host_bid_rejected={}, joiner_bid_rejected={}, \
             host_auction_settled={}, joiner_auction_settled={}, \
             host_auction_acquired={}, joiner_auction_acquired={}, \
             server_draft_shop={}, host_draft_shop={}, joiner_draft_shop={}, \
             host_draft_shop_count={}, joiner_draft_shop_count={}, \
             host_draft_shop_ready={}, joiner_draft_shop_ready={}, \
             host_post_auction_draft_shop_ready={}, joiner_post_auction_draft_shop_ready={}, \
             host_result_endpoint_draft_shop_ready={}, joiner_result_endpoint_draft_shop_ready={}, \
             host_result_endpoint_placement_submit={}, joiner_result_endpoint_placement_submit={}, \
             host_objective_identities={}, joiner_objective_identities={}, \
             host_real_lane_mask={:#07b}, joiner_real_lane_mask={:#07b}, \
             host_objective_destroyed={}, joiner_objective_destroyed={}, \
             host_game_over={}, joiner_game_over={}, \
             host_game_over_phase={}, joiner_game_over_phase={}, \
             host_game_over_after_resolution={}, joiner_game_over_after_resolution={}, \
             game_over_loser={}, game_over_round={}, game_over_reason_draw={}, \
             server_round_game_over={}, server_lobby_game_over={}, \
             server_config_removed={}, server_rng_removed={}, \
             server_host_real_destroyed={}, server_joiner_real_destroyed={}, \
             host_sent_acknowledge={}, joiner_sent_acknowledge={}, server_ended_state_cleared={}",
            self.host_sent_hello.load(Ordering::SeqCst),
            self.joiner_sent_hello.load(Ordering::SeqCst),
            self.host_received_handshake.load(Ordering::SeqCst),
            self.joiner_received_handshake.load(Ordering::SeqCst),
            self.host_player_id.load(Ordering::SeqCst),
            self.joiner_player_id.load(Ordering::SeqCst),
            self.server_connection_count.load(Ordering::SeqCst),
            self.server_room_count.load(Ordering::SeqCst),
            self.host_sent_create_room.load(Ordering::SeqCst),
            self.host_received_room_created.load(Ordering::SeqCst),
            self.room_code().is_some(),
            self.joiner_sent_join_room.load(Ordering::SeqCst),
            self.joiner_received_join_ack.load(Ordering::SeqCst),
            self.host_received_slot_update.load(Ordering::SeqCst),
            self.host_sent_select_class.load(Ordering::SeqCst),
            self.joiner_sent_select_class.load(Ordering::SeqCst),
            self.host_sent_confirm_class.load(Ordering::SeqCst),
            self.joiner_sent_confirm_class.load(Ordering::SeqCst),
            self.host_received_class_locked.load(Ordering::SeqCst),
            self.joiner_received_class_locked.load(Ordering::SeqCst),
            self.host_received_classes_revealed.load(Ordering::SeqCst),
            self.joiner_received_classes_revealed.load(Ordering::SeqCst),
            self.reveal_player_count.load(Ordering::SeqCst),
            self.server_room_game_active.load(Ordering::SeqCst),
            self.server_round_draft_initial.load(Ordering::SeqCst),
            self.host_received_draft_initial.load(Ordering::SeqCst),
            self.joiner_received_draft_initial.load(Ordering::SeqCst),
            self.host_received_draft_offering.load(Ordering::SeqCst),
            self.joiner_received_draft_offering.load(Ordering::SeqCst),
            self.host_purchase_card_id.load(Ordering::SeqCst),
            self.joiner_purchase_card_id.load(Ordering::SeqCst),
            self.host_sent_purchase_card.load(Ordering::SeqCst),
            self.joiner_sent_purchase_card.load(Ordering::SeqCst),
            self.host_received_card_acquired.load(Ordering::SeqCst),
            self.joiner_received_card_acquired.load(Ordering::SeqCst),
            self.host_received_purchase_gold_update.load(Ordering::SeqCst),
            self.joiner_received_purchase_gold_update.load(Ordering::SeqCst),
            self.host_sent_ready_initial.load(Ordering::SeqCst),
            self.host_sent_ready_retract.load(Ordering::SeqCst),
            self.host_sent_ready_final.load(Ordering::SeqCst),
            self.joiner_sent_ready.load(Ordering::SeqCst),
            self.server_retract_path_observed.load(Ordering::SeqCst),
            self.server_round_placement.load(Ordering::SeqCst),
            self.host_received_placement.load(Ordering::SeqCst),
            self.joiner_received_placement.load(Ordering::SeqCst),
            self.host_placement_phase_count.load(Ordering::SeqCst),
            self.joiner_placement_phase_count.load(Ordering::SeqCst),
            self.host_sent_placement_submit.load(Ordering::SeqCst),
            self.joiner_sent_placement_submit.load(Ordering::SeqCst),
            self.host_sent_non_empty_placement_submit.load(Ordering::SeqCst),
            self.joiner_sent_non_empty_placement_submit.load(Ordering::SeqCst),
            self.host_sent_post_auction_placement_submit.load(Ordering::SeqCst),
            self.joiner_sent_post_auction_placement_submit.load(Ordering::SeqCst),
            self.server_host_non_empty_placement_accepted.load(Ordering::SeqCst),
            self.server_joiner_non_empty_placement_accepted.load(Ordering::SeqCst),
            self.server_round_resolution.load(Ordering::SeqCst),
            self.host_received_resolution.load(Ordering::SeqCst),
            self.joiner_received_resolution.load(Ordering::SeqCst),
            self.host_resolution_phase_count.load(Ordering::SeqCst),
            self.joiner_resolution_phase_count.load(Ordering::SeqCst),
            self.host_received_non_empty_placement_reveal.load(Ordering::SeqCst),
            self.joiner_received_non_empty_placement_reveal.load(Ordering::SeqCst),
            self.host_non_empty_placement_reveal_count.load(Ordering::SeqCst),
            self.joiner_non_empty_placement_reveal_count.load(Ordering::SeqCst),
            self.host_received_resolution_event.load(Ordering::SeqCst),
            self.joiner_received_resolution_event.load(Ordering::SeqCst),
            self.host_resolution_event_count.load(Ordering::SeqCst),
            self.joiner_resolution_event_count.load(Ordering::SeqCst),
            self.host_received_unit_placed_resolution_event.load(Ordering::SeqCst),
            self.joiner_received_unit_placed_resolution_event.load(Ordering::SeqCst),
            self.server_round_draft_auction.load(Ordering::SeqCst),
            self.host_received_draft_auction.load(Ordering::SeqCst),
            self.joiner_received_draft_auction.load(Ordering::SeqCst),
            self.host_received_auction_card.load(Ordering::SeqCst),
            self.joiner_received_auction_card.load(Ordering::SeqCst),
            self.auction_starting_price.load(Ordering::SeqCst),
            self.host_sent_auction_bid.load(Ordering::SeqCst),
            self.host_received_auction_bid_accepted.load(Ordering::SeqCst),
            self.joiner_received_auction_bid_accepted.load(Ordering::SeqCst),
            self.host_received_auction_bid_rejected.load(Ordering::SeqCst),
            self.joiner_received_auction_bid_rejected.load(Ordering::SeqCst),
            self.host_received_auction_settled.load(Ordering::SeqCst),
            self.joiner_received_auction_settled.load(Ordering::SeqCst),
            self.host_received_card_acquired_source_auction.load(Ordering::SeqCst),
            self.joiner_received_card_acquired_source_auction.load(Ordering::SeqCst),
            self.server_round_draft_shop.load(Ordering::SeqCst),
            self.host_received_draft_shop.load(Ordering::SeqCst),
            self.joiner_received_draft_shop.load(Ordering::SeqCst),
            self.host_draft_shop_phase_count.load(Ordering::SeqCst),
            self.joiner_draft_shop_phase_count.load(Ordering::SeqCst),
            self.host_sent_draft_shop_ready.load(Ordering::SeqCst),
            self.joiner_sent_draft_shop_ready.load(Ordering::SeqCst),
            self.host_sent_post_auction_draft_shop_ready.load(Ordering::SeqCst),
            self.joiner_sent_post_auction_draft_shop_ready.load(Ordering::SeqCst),
            self.host_sent_result_endpoint_draft_shop_ready.load(Ordering::SeqCst),
            self.joiner_sent_result_endpoint_draft_shop_ready.load(Ordering::SeqCst),
            self.host_sent_result_endpoint_placement_submit.load(Ordering::SeqCst),
            self.joiner_sent_result_endpoint_placement_submit.load(Ordering::SeqCst),
            self.host_received_objective_identities.load(Ordering::SeqCst),
            self.joiner_received_objective_identities.load(Ordering::SeqCst),
            self.host_real_lane_mask.load(Ordering::SeqCst),
            self.joiner_real_lane_mask.load(Ordering::SeqCst),
            self.host_received_objective_destroyed_resolution_event.load(Ordering::SeqCst),
            self.joiner_received_objective_destroyed_resolution_event.load(Ordering::SeqCst),
            self.host_received_game_over.load(Ordering::SeqCst),
            self.joiner_received_game_over.load(Ordering::SeqCst),
            self.host_received_game_over_phase.load(Ordering::SeqCst),
            self.joiner_received_game_over_phase.load(Ordering::SeqCst),
            self.host_game_over_phase_after_resolution.load(Ordering::SeqCst),
            self.joiner_game_over_phase_after_resolution.load(Ordering::SeqCst),
            self.game_over_loser.load(Ordering::SeqCst),
            self.game_over_round.load(Ordering::SeqCst),
            self.game_over_reason_draw.load(Ordering::SeqCst),
            self.server_round_game_over.load(Ordering::SeqCst),
            self.server_lobby_game_over.load(Ordering::SeqCst),
            self.server_session_config_removed.load(Ordering::SeqCst),
            self.server_rng_removed.load(Ordering::SeqCst),
            self.server_host_real_destroyed.load(Ordering::SeqCst),
            self.server_joiner_real_destroyed.load(Ordering::SeqCst),
            self.host_sent_acknowledge.load(Ordering::SeqCst),
            self.joiner_sent_acknowledge.load(Ordering::SeqCst),
            self.server_ended_state_cleared.load(Ordering::SeqCst),
        )
    }
}

#[derive(Resource)]
struct RoomClientProbe {
    role: ClientRole,
    flags: RoomSessionFlags,
}

fn playable_card(id: u32, class: ClassId, cost: u32) -> CardData {
    playable_card_with_rarity(id, class, cost, Rarity::Common)
}

fn playable_card_with_rarity(id: u32, class: ClassId, cost: u32, rarity: Rarity) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class,
        family: Some("E2E GameOver".to_string()),
        rarity,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost,
        atk: 1,
        hp: 1,
        mp: 7,
        ar: 0,
        keywords: vec![],
        effect_text: String::new(),
        art_id: format!("e2e_game_over_{id}"),
        pool_copies_override: Some(2),
    }
}

fn playable_e2e_catalog() -> CardCatalog {
    CardCatalog {
        cards: (1..=14)
            .map(|id| playable_card(id, ClassId::Iop, 1))
            .chain((101..=114).map(|id| playable_card(id, ClassId::Cra, 1)))
            .chain(
                (201..=214)
                    .map(|id| playable_card_with_rarity(id, ClassId::Neutral, 1, Rarity::Rare)),
            )
            .map(|card| (card.id, card))
            .collect::<HashMap<_, _>>(),
    }
}

fn playable_e2e_config() -> server_crate::foundation::config::GameConfig {
    let mut config = shared::config::GameConfig::default();
    config.auction_timer_seconds = 1;
    config.auction_timer_reset_seconds = 0;
    config.objective_hp = 2;
    config.fake_count = 1;
    server_crate::foundation::config::GameConfig(config)
}

fn deterministic_server_rng() -> Result<ServerRng, ServerRngInitError> {
    Ok(ServerRng::from_seed(0x504C_4159_B005))
}

fn reserve_ephemeral_port() -> u16 {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("ephemeral localhost port should be available for full game-over route test");
    listener
        .local_addr()
        .expect("ephemeral listener should expose a local address")
        .port()
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("server crate should live under workspace root")
        .to_path_buf()
}

fn build_room_flow_server_app(port: u16, flags: RoomSessionFlags) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    app.init_resource::<PeerMetadata>();
    app.add_plugins(RsmPlugin);
    app.add_plugins(GameSessionPlugin);
    app.add_plugins(EconomyPlugin);
    app.add_plugins(CardPoolPlugin);
    app.add_plugins(CardAcquisitionPlugin);
    app.add_plugins(AuctionPlugin);
    app.add_plugins(BoardPlugin);
    app.add_plugins(CombatPlugin);
    app.add_plugins(ObjectivePlugin);
    register_lightyear_protocol(&mut app);
    app.add_plugins(EconomyNetworkPlugin);
    app.insert_resource(playable_e2e_config());
    app.insert_resource(playable_e2e_catalog());
    app.insert_resource(ServerRngFactory::new(deterministic_server_rng));
    app.insert_resource(ServerRng::new());
    app.insert_resource(flags);
    app.add_systems(Startup, move |mut commands: Commands| {
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let config = ServerConfig::builder()
            .with_bind_address(bind_addr)
            .with_no_encryption();
        let server = commands
            .spawn((
                Name::new("E2E GameOver Route Server"),
                LocalAddr(bind_addr),
                RawServer,
                WebSocketServerIo { config },
            ))
            .id();
        commands.trigger(Start { entity: server });
    });
    app.add_systems(
        Update,
        (
            drain_signal_ready_messages.before(rsm_input_reader),
            drain_submit_placement_messages
                .before(server_crate::feature::board::BoardSystemSet::PlacementSubmission),
            record_room_flow_server_state,
            dispatch_phase_changed.after(advance_phase),
        ),
    );
    app.finish();
    app
}

fn build_room_flow_client_app(url: String, role: ClientRole, flags: RoomSessionFlags) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    register_lightyear_protocol(&mut app);
    app.insert_resource(RoomClientProbe { role, flags });
    app.add_systems(Startup, move |mut commands: Commands| {
        let client = commands
            .spawn((
                Name::new(format!("E2E GameOver Route {role:?} Client")),
                Client::default(),
                RawClient,
                LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
                WebSocketClientIo::from_url(ClientConfig::default(), url.clone()),
            ))
            .id();
        commands.trigger(Connect { entity: client });
    });
    app.add_systems(
        Update,
        (
            send_room_flow_hello_until_handshake,
            record_room_flow_handshake,
            send_room_flow_lobby_actions,
            record_result_endpoint_s2c_messages,
            record_room_flow_s2c_messages,
            send_acknowledge_on_game_over,
        )
            .chain(),
    );
    app.finish();
    app
}

#[allow(clippy::too_many_arguments)]
fn record_room_flow_server_state(
    flags: Res<RoomSessionFlags>,
    connections: Option<Res<PlayerConnectionMap>>,
    rooms: Option<Res<RoomSessions>>,
    lobby_state: Option<Res<LobbyState>>,
    round_state: Option<Res<RoundState>>,
    session_config: Option<Res<SessionConfig>>,
    server_rng: Option<Res<ServerRng>>,
    objective_counters: Option<Res<ObjectiveCounters>>,
    hands: Option<Res<PlayerHands>>,
    pending: Option<Res<PendingPlacements>>,
    ended_state: Option<Res<EndedSessionResultState>>,
) {
    if let Some(connections) = connections {
        flags
            .server_connection_count
            .store(connections.0.len(), Ordering::SeqCst);
    }

    if let Some(rooms) = rooms {
        flags.server_room_count.store(rooms.len(), Ordering::SeqCst);
        if rooms
            .session_ids()
            .into_iter()
            .filter_map(|session_id| rooms.get(session_id))
            .any(|session| session.state == LobbyState::GameActive)
        {
            flags.server_room_game_active.store(true, Ordering::SeqCst);
        }
    }

    let host = player_from_atomic(&flags.host_player_id);
    let joiner = player_from_atomic(&flags.joiner_player_id);
    if matches!(lobby_state.as_deref(), Some(LobbyState::GameOver)) {
        flags.server_lobby_game_over.store(true, Ordering::SeqCst);
    }
    if let Some(counters) = objective_counters.as_deref() {
        if let Some(host) = host {
            flags.server_host_real_destroyed.store(
                counters.real_objectives_destroyed(host) as usize,
                Ordering::SeqCst,
            );
        }
        if let Some(joiner) = joiner {
            flags.server_joiner_real_destroyed.store(
                counters.real_objectives_destroyed(joiner) as usize,
                Ordering::SeqCst,
            );
        }
    }
    if flags.server_round_game_over.load(Ordering::SeqCst) {
        if session_config.is_none() {
            flags
                .server_session_config_removed
                .store(true, Ordering::SeqCst);
        }
        if server_rng.is_none() {
            flags.server_rng_removed.store(true, Ordering::SeqCst);
        }
    }

    if let Some(hands) = hands {
        if let Some(host) = host {
            flags
                .server_host_hand_size
                .store(hands.hand_len(host), Ordering::SeqCst);
        }
        if let Some(joiner) = joiner {
            flags
                .server_joiner_hand_size
                .store(hands.hand_len(joiner), Ordering::SeqCst);
        }
    }

    if let Some(pending) = pending {
        if let Some(host) = host {
            if pending
                .submissions
                .get(&host)
                .is_some_and(|submission| submission.is_final && !submission.placements.is_empty())
            {
                flags
                    .server_host_non_empty_placement_accepted
                    .store(true, Ordering::SeqCst);
            }
        }
        if let Some(joiner) = joiner {
            if pending
                .submissions
                .get(&joiner)
                .is_some_and(|submission| submission.is_final && !submission.placements.is_empty())
            {
                flags
                    .server_joiner_non_empty_placement_accepted
                    .store(true, Ordering::SeqCst);
            }
        }
    }

    if let Some(state) = round_state.as_deref() {
        match state.phase {
            ServerRoundPhase::DraftInitial => flags
                .server_round_draft_initial
                .store(true, Ordering::SeqCst),
            ServerRoundPhase::Placement => {
                flags.server_round_placement.store(true, Ordering::SeqCst)
            }
            ServerRoundPhase::Resolution => {
                flags.server_round_resolution.store(true, Ordering::SeqCst)
            }
            ServerRoundPhase::DraftAuction => flags
                .server_round_draft_auction
                .store(true, Ordering::SeqCst),
            ServerRoundPhase::DraftShop => {
                flags.server_round_draft_shop.store(true, Ordering::SeqCst)
            }
            ServerRoundPhase::GameOver => {
                flags.server_round_game_over.store(true, Ordering::SeqCst)
            }
            _ => {}
        }

        if let Some(host) = host {
            if state.draft_ready_players.contains(&host) {
                flags.server_host_ready_seen.store(true, Ordering::SeqCst);
            }
            if state.phase == ServerRoundPhase::DraftInitial
                && flags.host_sent_ready_retract.load(Ordering::SeqCst)
                && !state.draft_ready_players.contains(&host)
            {
                flags
                    .server_retract_path_observed
                    .store(true, Ordering::SeqCst);
            }
        }
    }

    // Check EndedSessionResultState cleared after both clients acknowledge
    if flags.server_lobby_game_over.load(Ordering::SeqCst)
        && flags.host_sent_acknowledge.load(Ordering::SeqCst)
        && flags.joiner_sent_acknowledge.load(Ordering::SeqCst)
        && ended_state.is_none()
    {
        flags
            .server_ended_state_cleared
            .store(true, Ordering::SeqCst);
    }
}

fn player_from_atomic(value: &AtomicU64) -> Option<PlayerId> {
    match value.load(Ordering::SeqCst) {
        0 => None,
        id => Some(PlayerId(id)),
    }
}

fn send_room_flow_hello_until_handshake(
    probe: Res<RoomClientProbe>,
    mut senders: Query<&mut MessageSender<C2SHello>>,
) {
    let handshake_received = match probe.role {
        ClientRole::Host => probe.flags.host_received_handshake.load(Ordering::SeqCst),
        ClientRole::Joiner => probe.flags.joiner_received_handshake.load(Ordering::SeqCst),
    };
    if handshake_received {
        return;
    }

    let mut sent = false;
    for mut sender in &mut senders {
        sender.send::<ReliableChannel>(C2SHello {
            protocol_version: shared::config::GameConfig::default().protocol_version,
            session_token: None,
        });
        sent = true;
    }

    if sent {
        match probe.role {
            ClientRole::Host => probe.flags.host_sent_hello.store(true, Ordering::SeqCst),
            ClientRole::Joiner => probe.flags.joiner_sent_hello.store(true, Ordering::SeqCst),
        }
    }
}

fn record_room_flow_handshake(
    probe: Res<RoomClientProbe>,
    mut receivers: Query<&mut MessageReceiver<S2CHandshake>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            match probe.role {
                ClientRole::Host => {
                    probe
                        .flags
                        .host_received_handshake
                        .store(true, Ordering::SeqCst);
                    probe
                        .flags
                        .host_player_id
                        .store(message.player_id.0, Ordering::SeqCst);
                }
                ClientRole::Joiner => {
                    probe
                        .flags
                        .joiner_received_handshake
                        .store(true, Ordering::SeqCst);
                    probe
                        .flags
                        .joiner_player_id
                        .store(message.player_id.0, Ordering::SeqCst);
                }
            }
        }
    }
}

fn send_room_flow_lobby_actions(
    probe: Res<RoomClientProbe>,
    mut create_room: Query<&mut MessageSender<C2SCreateRoom>>,
    mut join_room: Query<&mut MessageSender<C2SJoinRoom>>,
    mut select_class: Query<&mut MessageSender<C2SSelectClass>>,
    mut confirm_class: Query<&mut MessageSender<C2SConfirmClass>>,
    mut purchase_card: Query<&mut MessageSender<C2SPurchaseCard>>,
    mut signal_ready: Query<&mut MessageSender<C2SSignalReady>>,
    mut submit_placement: Query<&mut MessageSender<C2SSubmitPlacement>>,
    mut place_bid: Query<&mut MessageSender<C2SPlaceBid>>,
) {
    match probe.role {
        ClientRole::Host => {
            if probe.flags.host_received_handshake.load(Ordering::SeqCst)
                && !probe.flags.host_sent_create_room.load(Ordering::SeqCst)
            {
                if let Some(mut sender) = create_room.iter_mut().next() {
                    sender.send::<ReliableChannel>(C2SCreateRoom {
                        mode: GameMode::OneVOne,
                    });
                    probe
                        .flags
                        .host_sent_create_room
                        .store(true, Ordering::SeqCst);
                }
            }

            if probe.flags.host_received_slot_update.load(Ordering::SeqCst) {
                send_class_selection_and_confirm(
                    ClassId::Iop,
                    &probe.flags.host_sent_select_class,
                    &probe.flags.host_sent_confirm_class,
                    &mut select_class,
                    &mut confirm_class,
                );
            }
        }
        ClientRole::Joiner => {
            if probe.flags.joiner_received_handshake.load(Ordering::SeqCst)
                && !probe.flags.joiner_sent_join_room.load(Ordering::SeqCst)
            {
                if let (Some(room_code), Some(mut sender)) =
                    (probe.flags.room_code(), join_room.iter_mut().next())
                {
                    sender.send::<ReliableChannel>(C2SJoinRoom {
                        room_code,
                        requested_slot: 1,
                    });
                    probe
                        .flags
                        .joiner_sent_join_room
                        .store(true, Ordering::SeqCst);
                }
            }

            if probe.flags.joiner_received_join_ack.load(Ordering::SeqCst) {
                send_class_selection_and_confirm(
                    ClassId::Cra,
                    &probe.flags.joiner_sent_select_class,
                    &probe.flags.joiner_sent_confirm_class,
                    &mut select_class,
                    &mut confirm_class,
                );
            }
        }
    }

    send_draft_initial_purchase(&probe, &mut purchase_card);
    send_draft_ready_path(&probe, &mut signal_ready);
    send_draft_shop_ready_path(&probe, &mut signal_ready);
    send_placement_submits(&probe, &mut submit_placement);
    send_auction_bid(&probe, &mut place_bid);
}

fn send_draft_initial_purchase(
    probe: &RoomClientProbe,
    purchase_card: &mut Query<&mut MessageSender<C2SPurchaseCard>>,
) {
    let (offering_received, card_id, sent) = match probe.role {
        ClientRole::Host => (
            &probe.flags.host_received_draft_offering,
            probe.flags.host_purchase_card_id.load(Ordering::SeqCst),
            &probe.flags.host_sent_purchase_card,
        ),
        ClientRole::Joiner => (
            &probe.flags.joiner_received_draft_offering,
            probe.flags.joiner_purchase_card_id.load(Ordering::SeqCst),
            &probe.flags.joiner_sent_purchase_card,
        ),
    };

    if !offering_received.load(Ordering::SeqCst) || card_id == 0 || sent.load(Ordering::SeqCst) {
        return;
    }

    if let Some(mut sender) = purchase_card.iter_mut().next() {
        sender.send::<ReliableChannel>(C2SPurchaseCard {
            card_id: CardId(card_id as u32),
        });
        sent.store(true, Ordering::SeqCst);
    }
}

fn send_draft_ready_path(
    probe: &RoomClientProbe,
    signal_ready: &mut Query<&mut MessageSender<C2SSignalReady>>,
) {
    if !draft_purchases_observed(&probe.flags) {
        return;
    }

    match probe.role {
        ClientRole::Host => {
            if !probe.flags.host_sent_ready_initial.load(Ordering::SeqCst) {
                send_ready(signal_ready, false, &probe.flags.host_sent_ready_initial);
                return;
            }

            if probe.flags.server_host_ready_seen.load(Ordering::SeqCst)
                && !probe.flags.host_sent_ready_retract.load(Ordering::SeqCst)
            {
                send_ready(signal_ready, true, &probe.flags.host_sent_ready_retract);
                return;
            }

            if probe
                .flags
                .server_retract_path_observed
                .load(Ordering::SeqCst)
                && !probe.flags.host_sent_ready_final.load(Ordering::SeqCst)
            {
                send_ready(signal_ready, false, &probe.flags.host_sent_ready_final);
            }
        }
        ClientRole::Joiner => {
            if probe
                .flags
                .server_retract_path_observed
                .load(Ordering::SeqCst)
                && probe.flags.host_sent_ready_final.load(Ordering::SeqCst)
                && !probe.flags.joiner_sent_ready.load(Ordering::SeqCst)
            {
                send_ready(signal_ready, false, &probe.flags.joiner_sent_ready);
            }
        }
    }
}

fn draft_purchases_observed(flags: &RoomSessionFlags) -> bool {
    flags.host_received_card_acquired.load(Ordering::SeqCst)
        && flags.joiner_received_card_acquired.load(Ordering::SeqCst)
        && flags
            .host_received_purchase_gold_update
            .load(Ordering::SeqCst)
        && flags
            .joiner_received_purchase_gold_update
            .load(Ordering::SeqCst)
}

fn send_ready(
    signal_ready: &mut Query<&mut MessageSender<C2SSignalReady>>,
    retract: bool,
    sent: &AtomicBool,
) {
    if let Some(mut sender) = signal_ready.iter_mut().next() {
        sender.send::<ReliableChannel>(C2SSignalReady { retract });
        sent.store(true, Ordering::SeqCst);
    }
}

fn send_draft_shop_ready_path(
    probe: &RoomClientProbe,
    signal_ready: &mut Query<&mut MessageSender<C2SSignalReady>>,
) {
    let (draft_shop_count, first_sent, post_auction_sent, result_endpoint_sent) = match probe.role {
        ClientRole::Host => (
            probe
                .flags
                .host_draft_shop_phase_count
                .load(Ordering::SeqCst),
            &probe.flags.host_sent_draft_shop_ready,
            &probe.flags.host_sent_post_auction_draft_shop_ready,
            &probe.flags.host_sent_result_endpoint_draft_shop_ready,
        ),
        ClientRole::Joiner => (
            probe
                .flags
                .joiner_draft_shop_phase_count
                .load(Ordering::SeqCst),
            &probe.flags.joiner_sent_draft_shop_ready,
            &probe.flags.joiner_sent_post_auction_draft_shop_ready,
            &probe.flags.joiner_sent_result_endpoint_draft_shop_ready,
        ),
    };

    if draft_shop_count >= 3 && !result_endpoint_sent.load(Ordering::SeqCst) {
        send_ready(signal_ready, false, result_endpoint_sent);
        return;
    }

    if draft_shop_count >= 2 && !post_auction_sent.load(Ordering::SeqCst) {
        send_ready(signal_ready, false, post_auction_sent);
        return;
    }

    if draft_shop_count >= 1 && !first_sent.load(Ordering::SeqCst) {
        send_ready(signal_ready, false, first_sent);
    }
}

fn send_placement_submits(
    probe: &RoomClientProbe,
    submit_placement: &mut Query<&mut MessageSender<C2SSubmitPlacement>>,
) {
    let placement_count = match probe.role {
        ClientRole::Host => probe
            .flags
            .host_placement_phase_count
            .load(Ordering::SeqCst),
        ClientRole::Joiner => probe
            .flags
            .joiner_placement_phase_count
            .load(Ordering::SeqCst),
    };

    match placement_count {
        0 => {}
        1 => send_empty_placement_submit(probe, submit_placement),
        2 => send_planned_objective_placement_submit(probe, submit_placement, 0, false),
        3 => send_planned_objective_placement_submit(probe, submit_placement, 1, true),
        _ => send_result_endpoint_empty_placement_submit(probe, submit_placement),
    }
}

fn send_empty_placement_submit(
    probe: &RoomClientProbe,
    submit_placement: &mut Query<&mut MessageSender<C2SSubmitPlacement>>,
) {
    let sent = match probe.role {
        ClientRole::Host => &probe.flags.host_sent_placement_submit,
        ClientRole::Joiner => &probe.flags.joiner_sent_placement_submit,
    };

    if sent.load(Ordering::SeqCst) {
        return;
    }

    if let Some(mut sender) = submit_placement.iter_mut().next() {
        sender.send::<ReliableChannel>(C2SSubmitPlacement {
            placements: Vec::new(),
        });
        sent.store(true, Ordering::SeqCst);
    }
}

fn send_non_empty_placement_submit(
    probe: &RoomClientProbe,
    submit_placement: &mut Query<&mut MessageSender<C2SSubmitPlacement>>,
    lane: u8,
    post_auction: bool,
) {
    let sent = match (probe.role, post_auction) {
        (ClientRole::Host, false) => &probe.flags.host_sent_non_empty_placement_submit,
        (ClientRole::Joiner, false) => &probe.flags.joiner_sent_non_empty_placement_submit,
        (ClientRole::Host, true) => &probe.flags.host_sent_post_auction_placement_submit,
        (ClientRole::Joiner, true) => &probe.flags.joiner_sent_post_auction_placement_submit,
    };
    if sent.load(Ordering::SeqCst) {
        return;
    }

    let Some(placement) = non_empty_placement(probe, lane) else {
        return;
    };

    if let Some(mut sender) = submit_placement.iter_mut().next() {
        sender.send::<ReliableChannel>(C2SSubmitPlacement {
            placements: vec![placement],
        });
        sent.store(true, Ordering::SeqCst);
    }
}

fn send_planned_objective_placement_submit(
    probe: &RoomClientProbe,
    submit_placement: &mut Query<&mut MessageSender<C2SSubmitPlacement>>,
    pair_index: usize,
    post_auction: bool,
) {
    let Some((host_lane, joiner_lane)) = planned_objective_lanes(&probe.flags, pair_index) else {
        return;
    };
    let lane = match probe.role {
        ClientRole::Host => host_lane,
        ClientRole::Joiner => joiner_lane,
    };

    send_non_empty_placement_submit(probe, submit_placement, lane, post_auction);
}

fn send_result_endpoint_empty_placement_submit(
    probe: &RoomClientProbe,
    submit_placement: &mut Query<&mut MessageSender<C2SSubmitPlacement>>,
) {
    let sent = match probe.role {
        ClientRole::Host => &probe.flags.host_sent_result_endpoint_placement_submit,
        ClientRole::Joiner => &probe.flags.joiner_sent_result_endpoint_placement_submit,
    };

    if sent.load(Ordering::SeqCst) {
        return;
    }

    if let Some(mut sender) = submit_placement.iter_mut().next() {
        sender.send::<ReliableChannel>(C2SSubmitPlacement {
            placements: Vec::new(),
        });
        sent.store(true, Ordering::SeqCst);
    }
}

fn planned_objective_lanes(flags: &RoomSessionFlags, pair_index: usize) -> Option<(u8, u8)> {
    let host_real_lanes = real_lanes(flags.host_real_lane_mask.load(Ordering::SeqCst));
    let joiner_real_lanes = real_lanes(flags.joiner_real_lane_mask.load(Ordering::SeqCst));

    for host_first in &joiner_real_lanes {
        for joiner_first in &host_real_lanes {
            if host_first == joiner_first {
                continue;
            }
            for host_second in joiner_real_lanes.iter().filter(|lane| *lane != host_first) {
                for joiner_second in host_real_lanes.iter().filter(|lane| *lane != joiner_first) {
                    if host_second == joiner_second {
                        continue;
                    }
                    return match pair_index {
                        0 => Some((*host_first, *joiner_first)),
                        1 => Some((*host_second, *joiner_second)),
                        _ => None,
                    };
                }
            }
        }
    }

    None
}

fn real_lanes(mask: usize) -> Vec<u8> {
    (1..=5)
        .filter(|lane| mask & lane_bit(*lane) != 0)
        .collect::<Vec<_>>()
}

fn lane_bit(lane: u8) -> usize {
    1usize << usize::from(lane)
}

fn non_empty_placement(probe: &RoomClientProbe, lane: u8) -> Option<PlacedCardSubmit> {
    let card_id = match probe.role {
        ClientRole::Host => probe.flags.host_purchase_card_id.load(Ordering::SeqCst),
        ClientRole::Joiner => probe.flags.joiner_purchase_card_id.load(Ordering::SeqCst),
    };
    if card_id == 0 {
        return None;
    }

    let cell = match probe.role {
        ClientRole::Host => 1,
        ClientRole::Joiner => 8,
    };

    Some(PlacedCardSubmit {
        card_id: CardId(card_id as u32),
        target: PlayTarget::BoardCell { lane, cell },
        current_mana_spend: 1,
        reserve_mana_spend: 0,
    })
}

fn send_auction_bid(
    probe: &RoomClientProbe,
    place_bid: &mut Query<&mut MessageSender<C2SPlaceBid>>,
) {
    if probe.role != ClientRole::Host
        || !probe
            .flags
            .host_received_auction_card
            .load(Ordering::SeqCst)
        || probe.flags.host_sent_auction_bid.load(Ordering::SeqCst)
    {
        return;
    }

    let starting_price = probe.flags.auction_starting_price.load(Ordering::SeqCst);
    if starting_price == 0 {
        return;
    }

    if let Some(mut sender) = place_bid.iter_mut().next() {
        sender.send::<ReliableChannel>(C2SPlaceBid {
            amount: (starting_price as u32).saturating_add(1),
        });
        probe
            .flags
            .host_sent_auction_bid
            .store(true, Ordering::SeqCst);
    }
}

fn send_class_selection_and_confirm(
    class_id: ClassId,
    select_sent: &AtomicBool,
    confirm_sent: &AtomicBool,
    select_class: &mut Query<&mut MessageSender<C2SSelectClass>>,
    confirm_class: &mut Query<&mut MessageSender<C2SConfirmClass>>,
) {
    if !select_sent.load(Ordering::SeqCst) {
        if let Some(mut sender) = select_class.iter_mut().next() {
            sender.send::<ReliableChannel>(C2SSelectClass { class_id });
            select_sent.store(true, Ordering::SeqCst);
        }
    }

    if !confirm_sent.load(Ordering::SeqCst) {
        if let Some(mut sender) = confirm_class.iter_mut().next() {
            sender.send::<ReliableChannel>(C2SConfirmClass { class_id });
            confirm_sent.store(true, Ordering::SeqCst);
        }
    }
}

/// Sends C2SAcknowledgeResult once per client after S2CGameOver is received.
/// This is the automated equivalent of the player pressing "Return to Lobby".
fn send_acknowledge_on_game_over(
    probe: Res<RoomClientProbe>,
    mut senders: Query<&mut MessageSender<C2SAcknowledgeResult>>,
) {
    let (game_over_received, sent_flag) = match probe.role {
        ClientRole::Host => (
            probe.flags.host_received_game_over.load(Ordering::SeqCst),
            &probe.flags.host_sent_acknowledge,
        ),
        ClientRole::Joiner => (
            probe.flags.joiner_received_game_over.load(Ordering::SeqCst),
            &probe.flags.joiner_sent_acknowledge,
        ),
    };

    if !game_over_received || sent_flag.load(Ordering::SeqCst) {
        return;
    }

    if let Some(mut sender) = senders.iter_mut().next() {
        sender.send::<ReliableChannel>(C2SAcknowledgeResult {});
        sent_flag.store(true, Ordering::SeqCst);
    }
}

fn record_result_endpoint_s2c_messages(
    probe: Res<RoomClientProbe>,
    mut objective_identities: Query<&mut MessageReceiver<S2CObjectiveIdentities>>,
    mut game_over: Query<&mut MessageReceiver<S2CGameOver>>,
) {
    for mut receiver in &mut objective_identities {
        for message in receiver.receive() {
            record_objective_identities(&probe, &message);
        }
    }

    for mut receiver in &mut game_over {
        for message in receiver.receive() {
            record_game_over(&probe, &message);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn record_room_flow_s2c_messages(
    probe: Res<RoomClientProbe>,
    mut room_created: Query<&mut MessageReceiver<S2CRoomCreated>>,
    mut join_ack: Query<&mut MessageReceiver<S2CJoinAck>>,
    mut slot_updates: Query<&mut MessageReceiver<S2CSlotUpdated>>,
    mut class_locked: Query<&mut MessageReceiver<S2CClassLocked>>,
    mut classes_revealed: Query<&mut MessageReceiver<S2CClassesRevealed>>,
    mut phase_changed: Query<&mut MessageReceiver<S2CPhaseChanged>>,
    mut draft_offering: Query<&mut MessageReceiver<S2CDraftOffering>>,
    mut card_acquired: Query<&mut MessageReceiver<S2CCardAcquired>>,
    mut gold_update: Query<&mut MessageReceiver<S2CGoldUpdate>>,
    mut placement_reveal: Query<&mut MessageReceiver<S2CPlacementReveal>>,
    mut resolution_event: Query<&mut MessageReceiver<S2CResolutionEvent>>,
    mut auction_card: Query<&mut MessageReceiver<S2CAuctionCard>>,
    mut auction_bid_accepted: Query<&mut MessageReceiver<S2CAuctionBidAccepted>>,
    mut auction_bid_rejected: Query<&mut MessageReceiver<S2CAuctionBidRejected>>,
    mut auction_settled: Query<&mut MessageReceiver<S2CAuctionSettled>>,
) {
    for mut receiver in &mut room_created {
        for message in receiver.receive() {
            if probe.role == ClientRole::Host {
                *probe
                    .flags
                    .room_code
                    .lock()
                    .expect("room code capture lock should not be poisoned") =
                    Some(message.room_code.clone());
                probe
                    .flags
                    .host_received_room_created
                    .store(true, Ordering::SeqCst);
            }
        }
    }

    for mut receiver in &mut join_ack {
        for _ in receiver.receive() {
            if probe.role == ClientRole::Joiner {
                probe
                    .flags
                    .joiner_received_join_ack
                    .store(true, Ordering::SeqCst);
            }
        }
    }

    for mut receiver in &mut slot_updates {
        for _ in receiver.receive() {
            if probe.role == ClientRole::Host {
                probe
                    .flags
                    .host_received_slot_update
                    .store(true, Ordering::SeqCst);
            }
        }
    }

    for mut receiver in &mut class_locked {
        for _ in receiver.receive() {
            match probe.role {
                ClientRole::Host => probe
                    .flags
                    .host_received_class_locked
                    .store(true, Ordering::SeqCst),
                ClientRole::Joiner => probe
                    .flags
                    .joiner_received_class_locked
                    .store(true, Ordering::SeqCst),
            }
        }
    }

    for mut receiver in &mut classes_revealed {
        for message in receiver.receive() {
            probe
                .flags
                .reveal_player_count
                .store(message.player_class_map.len(), Ordering::SeqCst);
            match probe.role {
                ClientRole::Host => probe
                    .flags
                    .host_received_classes_revealed
                    .store(true, Ordering::SeqCst),
                ClientRole::Joiner => probe
                    .flags
                    .joiner_received_classes_revealed
                    .store(true, Ordering::SeqCst),
            }
        }
    }

    for mut receiver in &mut placement_reveal {
        for message in receiver.receive() {
            match probe.role {
                ClientRole::Host => {
                    probe
                        .flags
                        .host_received_placement_reveal
                        .store(true, Ordering::SeqCst);
                    if !message.placements.is_empty() {
                        probe
                            .flags
                            .host_received_non_empty_placement_reveal
                            .store(true, Ordering::SeqCst);
                        probe
                            .flags
                            .host_non_empty_placement_reveal_count
                            .fetch_add(1, Ordering::SeqCst);
                    }
                }
                ClientRole::Joiner => {
                    probe
                        .flags
                        .joiner_received_placement_reveal
                        .store(true, Ordering::SeqCst);
                    if !message.placements.is_empty() {
                        probe
                            .flags
                            .joiner_received_non_empty_placement_reveal
                            .store(true, Ordering::SeqCst);
                        probe
                            .flags
                            .joiner_non_empty_placement_reveal_count
                            .fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
        }
    }

    for mut receiver in &mut resolution_event {
        for message in receiver.receive() {
            record_resolution_event(&probe, &message);
        }
    }

    for mut receiver in &mut phase_changed {
        for message in receiver.receive() {
            record_phase_changed(&probe, message.phase);
        }
    }

    for mut receiver in &mut draft_offering {
        for message in receiver.receive() {
            if let Some(card_id) = message.card_ids.first().copied() {
                match probe.role {
                    ClientRole::Host => {
                        probe
                            .flags
                            .host_draft_offering_count
                            .store(message.card_ids.len(), Ordering::SeqCst);
                        probe
                            .flags
                            .host_purchase_card_id
                            .store(u64::from(card_id.0), Ordering::SeqCst);
                        probe
                            .flags
                            .host_received_draft_offering
                            .store(true, Ordering::SeqCst);
                    }
                    ClientRole::Joiner => {
                        probe
                            .flags
                            .joiner_draft_offering_count
                            .store(message.card_ids.len(), Ordering::SeqCst);
                        probe
                            .flags
                            .joiner_purchase_card_id
                            .store(u64::from(card_id.0), Ordering::SeqCst);
                        probe
                            .flags
                            .joiner_received_draft_offering
                            .store(true, Ordering::SeqCst);
                    }
                }
            }
        }
    }

    for mut receiver in &mut card_acquired {
        for message in receiver.receive() {
            match probe.role {
                ClientRole::Host => {
                    probe
                        .flags
                        .host_received_card_acquired
                        .store(true, Ordering::SeqCst);
                    if message.source == CardSource::DraftInitial {
                        probe
                            .flags
                            .host_card_acquired_source_draft_initial
                            .store(true, Ordering::SeqCst);
                    }
                    if message.source == CardSource::AuctionWon {
                        probe
                            .flags
                            .host_received_card_acquired_source_auction
                            .store(true, Ordering::SeqCst);
                    }
                }
                ClientRole::Joiner => {
                    probe
                        .flags
                        .joiner_received_card_acquired
                        .store(true, Ordering::SeqCst);
                    if message.source == CardSource::DraftInitial {
                        probe
                            .flags
                            .joiner_card_acquired_source_draft_initial
                            .store(true, Ordering::SeqCst);
                    }
                    if message.source == CardSource::AuctionWon {
                        probe
                            .flags
                            .joiner_received_card_acquired_source_auction
                            .store(true, Ordering::SeqCst);
                    }
                }
            }
        }
    }

    for mut receiver in &mut gold_update {
        for message in receiver.receive() {
            let purchase_gold = message.gold < shared::config::GameConfig::default().starting_gold;
            match probe.role {
                ClientRole::Host => {
                    if probe.flags.host_sent_purchase_card.load(Ordering::SeqCst) && purchase_gold {
                        probe
                            .flags
                            .host_received_purchase_gold_update
                            .store(true, Ordering::SeqCst);
                    }
                }
                ClientRole::Joiner => {
                    if probe.flags.joiner_sent_purchase_card.load(Ordering::SeqCst) && purchase_gold
                    {
                        probe
                            .flags
                            .joiner_received_purchase_gold_update
                            .store(true, Ordering::SeqCst);
                    }
                }
            }
        }
    }

    for mut receiver in &mut auction_card {
        for message in receiver.receive() {
            probe
                .flags
                .auction_starting_price
                .store(message.starting_price as usize, Ordering::SeqCst);
            match probe.role {
                ClientRole::Host => probe
                    .flags
                    .host_received_auction_card
                    .store(true, Ordering::SeqCst),
                ClientRole::Joiner => probe
                    .flags
                    .joiner_received_auction_card
                    .store(true, Ordering::SeqCst),
            }
        }
    }

    for mut receiver in &mut auction_bid_accepted {
        for _message in receiver.receive() {
            match probe.role {
                ClientRole::Host => probe
                    .flags
                    .host_received_auction_bid_accepted
                    .store(true, Ordering::SeqCst),
                ClientRole::Joiner => probe
                    .flags
                    .joiner_received_auction_bid_accepted
                    .store(true, Ordering::SeqCst),
            }
        }
    }

    for mut receiver in &mut auction_bid_rejected {
        for _message in receiver.receive() {
            match probe.role {
                ClientRole::Host => probe
                    .flags
                    .host_received_auction_bid_rejected
                    .store(true, Ordering::SeqCst),
                ClientRole::Joiner => probe
                    .flags
                    .joiner_received_auction_bid_rejected
                    .store(true, Ordering::SeqCst),
            }
        }
    }

    for mut receiver in &mut auction_settled {
        for _message in receiver.receive() {
            match probe.role {
                ClientRole::Host => probe
                    .flags
                    .host_received_auction_settled
                    .store(true, Ordering::SeqCst),
                ClientRole::Joiner => probe
                    .flags
                    .joiner_received_auction_settled
                    .store(true, Ordering::SeqCst),
            }
        }
    }
}

fn record_objective_identities(probe: &RoomClientProbe, message: &S2CObjectiveIdentities) {
    let real_lane_mask = message
        .identities
        .iter()
        .filter_map(|(lane, is_fake)| (!*is_fake).then_some(lane_bit(*lane)))
        .fold(0usize, |mask, bit| mask | bit);

    match probe.role {
        ClientRole::Host => {
            probe
                .flags
                .host_real_lane_mask
                .store(real_lane_mask, Ordering::SeqCst);
            probe
                .flags
                .host_received_objective_identities
                .store(true, Ordering::SeqCst);
        }
        ClientRole::Joiner => {
            probe
                .flags
                .joiner_real_lane_mask
                .store(real_lane_mask, Ordering::SeqCst);
            probe
                .flags
                .joiner_received_objective_identities
                .store(true, Ordering::SeqCst);
        }
    }
}

fn record_resolution_event(probe: &RoomClientProbe, message: &S2CResolutionEvent) {
    let has_unit_placed = message
        .events
        .iter()
        .any(|event| matches!(event.event, ResolutionEvent::UnitPlaced { .. }));
    let has_objective_destroyed = message
        .events
        .iter()
        .any(|event| matches!(event.event, ResolutionEvent::ObjectiveDestroyed { .. }));

    match probe.role {
        ClientRole::Host => {
            probe
                .flags
                .host_received_resolution_event
                .store(true, Ordering::SeqCst);
            probe
                .flags
                .host_resolution_event_count
                .fetch_add(1, Ordering::SeqCst);
            if has_unit_placed {
                probe
                    .flags
                    .host_received_unit_placed_resolution_event
                    .store(true, Ordering::SeqCst);
            }
            if has_objective_destroyed {
                probe
                    .flags
                    .host_received_objective_destroyed_resolution_event
                    .store(true, Ordering::SeqCst);
            }
        }
        ClientRole::Joiner => {
            probe
                .flags
                .joiner_received_resolution_event
                .store(true, Ordering::SeqCst);
            probe
                .flags
                .joiner_resolution_event_count
                .fetch_add(1, Ordering::SeqCst);
            if has_unit_placed {
                probe
                    .flags
                    .joiner_received_unit_placed_resolution_event
                    .store(true, Ordering::SeqCst);
            }
            if has_objective_destroyed {
                probe
                    .flags
                    .joiner_received_objective_destroyed_resolution_event
                    .store(true, Ordering::SeqCst);
            }
        }
    }
}

fn record_game_over(probe: &RoomClientProbe, message: &S2CGameOver) {
    probe.flags.game_over_loser.store(
        message.loser.map_or(NO_GAME_OVER_LOSER, |player| player.0),
        Ordering::SeqCst,
    );
    probe
        .flags
        .game_over_round
        .store(message.round as usize, Ordering::SeqCst);
    if message.reason == GameOverReason::Draw {
        probe
            .flags
            .game_over_reason_draw
            .store(true, Ordering::SeqCst);
    }

    match probe.role {
        ClientRole::Host => probe
            .flags
            .host_received_game_over
            .store(true, Ordering::SeqCst),
        ClientRole::Joiner => probe
            .flags
            .joiner_received_game_over
            .store(true, Ordering::SeqCst),
    }
}

fn record_phase_changed(probe: &RoomClientProbe, phase: ProtocolRoundPhase) {
    match (probe.role, phase) {
        (ClientRole::Host, ProtocolRoundPhase::DraftInitial) => probe
            .flags
            .host_received_draft_initial
            .store(true, Ordering::SeqCst),
        (ClientRole::Joiner, ProtocolRoundPhase::DraftInitial) => probe
            .flags
            .joiner_received_draft_initial
            .store(true, Ordering::SeqCst),
        (ClientRole::Host, ProtocolRoundPhase::Placement) => probe
            .flags
            .host_received_placement
            .store(true, Ordering::SeqCst),
        (ClientRole::Joiner, ProtocolRoundPhase::Placement) => probe
            .flags
            .joiner_received_placement
            .store(true, Ordering::SeqCst),
        (ClientRole::Host, ProtocolRoundPhase::Resolution) => probe
            .flags
            .host_received_resolution
            .store(true, Ordering::SeqCst),
        (ClientRole::Joiner, ProtocolRoundPhase::Resolution) => probe
            .flags
            .joiner_received_resolution
            .store(true, Ordering::SeqCst),
        (ClientRole::Host, ProtocolRoundPhase::DraftAuction) => probe
            .flags
            .host_received_draft_auction
            .store(true, Ordering::SeqCst),
        (ClientRole::Joiner, ProtocolRoundPhase::DraftAuction) => probe
            .flags
            .joiner_received_draft_auction
            .store(true, Ordering::SeqCst),
        (ClientRole::Host, ProtocolRoundPhase::DraftShop) => probe
            .flags
            .host_received_draft_shop
            .store(true, Ordering::SeqCst),
        (ClientRole::Joiner, ProtocolRoundPhase::DraftShop) => probe
            .flags
            .joiner_received_draft_shop
            .store(true, Ordering::SeqCst),
        (ClientRole::Host, ProtocolRoundPhase::GameOver) => {
            probe
                .flags
                .host_received_game_over_phase
                .store(true, Ordering::SeqCst);
            if probe
                .flags
                .host_resolution_event_count
                .load(Ordering::SeqCst)
                >= 4
            {
                probe
                    .flags
                    .host_game_over_phase_after_resolution
                    .store(true, Ordering::SeqCst);
            }
        }
        (ClientRole::Joiner, ProtocolRoundPhase::GameOver) => {
            probe
                .flags
                .joiner_received_game_over_phase
                .store(true, Ordering::SeqCst);
            if probe
                .flags
                .joiner_resolution_event_count
                .load(Ordering::SeqCst)
                >= 4
            {
                probe
                    .flags
                    .joiner_game_over_phase_after_resolution
                    .store(true, Ordering::SeqCst);
            }
        }
        _ => {}
    }

    match (probe.role, phase) {
        (ClientRole::Host, ProtocolRoundPhase::Placement) => {
            probe
                .flags
                .host_placement_phase_count
                .fetch_add(1, Ordering::SeqCst);
        }
        (ClientRole::Joiner, ProtocolRoundPhase::Placement) => {
            probe
                .flags
                .joiner_placement_phase_count
                .fetch_add(1, Ordering::SeqCst);
        }
        (ClientRole::Host, ProtocolRoundPhase::Resolution) => {
            probe
                .flags
                .host_resolution_phase_count
                .fetch_add(1, Ordering::SeqCst);
        }
        (ClientRole::Joiner, ProtocolRoundPhase::Resolution) => {
            probe
                .flags
                .joiner_resolution_phase_count
                .fetch_add(1, Ordering::SeqCst);
        }
        (ClientRole::Host, ProtocolRoundPhase::DraftShop) => {
            probe
                .flags
                .host_draft_shop_phase_count
                .fetch_add(1, Ordering::SeqCst);
        }
        (ClientRole::Joiner, ProtocolRoundPhase::DraftShop) => {
            probe
                .flags
                .joiner_draft_shop_phase_count
                .fetch_add(1, Ordering::SeqCst);
        }
        _ => {}
    }
}
