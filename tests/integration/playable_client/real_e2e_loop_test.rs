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
use server_crate::core::rsm::{
    advance_phase, rsm_input_reader, RoundPhase as ServerRoundPhase, RoundState, RsmPlugin,
};
use server_crate::core::session::{
    GameSessionPlugin, LobbyState, PlayerConnectionMap, RoomSessions,
};
use server_crate::core::{economy::EconomyPlugin, pool::CardPoolPlugin};
use server_crate::feature::{
    acquisition::{CardAcquisitionPlugin, PlayerHands},
    board::BoardPlugin,
    combat::CombatPlugin,
};
use server_crate::foundation::{config::CardCatalog, rng::ServerRng};
use server_crate::network::{
    drain_signal_ready_messages, drain_submit_placement_messages,
    economy_dispatch::EconomyNetworkPlugin, register_lightyear_protocol,
    rsm_dispatch::dispatch_phase_changed,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{
    C2SConfirmClass, C2SCreateRoom, C2SHello, C2SJoinRoom, C2SPurchaseCard, C2SSelectClass,
    C2SSignalReady, C2SSubmitPlacement, CardSource, GameMode, ReliableChannel,
    RoundPhase as ProtocolRoundPhase, S2CCardAcquired, S2CClassLocked, S2CClassesRevealed,
    S2CDraftOffering, S2CGoldUpdate, S2CHandshake, S2CJoinAck, S2CPhaseChanged, S2CPlacementReveal,
    S2CResolutionEvent, S2CRoomCreated, S2CSlotUpdated,
};
use shared::session::PlayerId;

const TICK_HZ: f64 = 60.0;
const MAX_FRAMES: usize = 600;
const ROOM_FLOW_MAX_FRAMES: usize = 2_400;
const FRAME_SLEEP: Duration = Duration::from_millis(10);

#[test]
fn client_server_lightyear_protocol_features_cover_replication_metadata() {
    let root = workspace_root();
    let client_manifest = fs::read_to_string(root.join("client").join("Cargo.toml"))
        .expect("client Cargo.toml should be readable");
    let server_manifest = fs::read_to_string(root.join("server").join("Cargo.toml"))
        .expect("server Cargo.toml should be readable");

    assert_lightyear_feature(&client_manifest, "client", "client");
    assert_lightyear_feature(&client_manifest, "client", "websocket");
    assert_lightyear_feature(&client_manifest, "client", "raw_connection");
    assert_lightyear_feature(&client_manifest, "client", "replication");
    assert_lightyear_feature(&server_manifest, "server", "server");
    assert_lightyear_feature(&server_manifest, "server", "websocket");
    assert_lightyear_feature(&server_manifest, "server", "raw_connection");
    assert_lightyear_feature(&server_manifest, "server", "replication");
}

#[test]
fn real_lightyear_smoke_reaches_fresh_hello_handshake() {
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");
    let flags = HandshakeFlags::new();

    let mut server_app = build_server_app(port, flags.clone());
    for _ in 0..30 {
        server_app.update();
        thread::sleep(FRAME_SLEEP);
    }

    let mut client_app = build_client_app(url, flags.clone());
    for _ in 0..MAX_FRAMES {
        server_app.update();
        client_app.update();

        if flags.handshake_observed() {
            break;
        }

        thread::sleep(FRAME_SLEEP);
    }

    assert!(
        flags.client_sent_hello.load(Ordering::SeqCst),
        "client never queued C2SHello: {}",
        flags.report()
    );
    assert!(
        flags.server_mapped_player.load(Ordering::SeqCst),
        "server never mapped fresh hello to a player: {}",
        flags.report()
    );
    assert!(
        flags.client_received_handshake.load(Ordering::SeqCst),
        "client never received S2CHandshake: {}",
        flags.report()
    );
    assert_eq!(
        flags.client_handshake_player.load(Ordering::SeqCst),
        1,
        "fresh hello should assign PlayerId(1): {}",
        flags.report()
    );
}

#[test]
fn real_lightyear_two_client_room_session_reaches_class_reveal_and_session_entry() {
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

        if flags.session_entry_observed() {
            break;
        }

        thread::sleep(FRAME_SLEEP);
    }

    let room_code = flags
        .room_code()
        .expect("room code should be captured from S2CRoomCreated");
    assert_eq!(
        room_code.len(),
        server_crate::core::session::ROOM_CODE_LEN,
        "room code should use the server room-code length: {}",
        flags.report()
    );
    assert!(
        flags.host_sent_create_room.load(Ordering::SeqCst),
        "host never sent C2SCreateRoom: {}",
        flags.report()
    );
    assert!(
        flags.host_received_room_created.load(Ordering::SeqCst),
        "host never received S2CRoomCreated: {}",
        flags.report()
    );
    assert!(
        flags.joiner_sent_join_room.load(Ordering::SeqCst),
        "joiner never sent C2SJoinRoom: {}",
        flags.report()
    );
    assert!(
        flags.joiner_received_join_ack.load(Ordering::SeqCst),
        "joiner never received S2CJoinAck: {}",
        flags.report()
    );
    assert!(
        flags.host_sent_select_class.load(Ordering::SeqCst)
            && flags.joiner_sent_select_class.load(Ordering::SeqCst),
        "both clients should send C2SSelectClass: {}",
        flags.report()
    );
    assert!(
        flags.host_sent_confirm_class.load(Ordering::SeqCst)
            && flags.joiner_sent_confirm_class.load(Ordering::SeqCst),
        "both clients should send C2SConfirmClass: {}",
        flags.report()
    );
    assert!(
        flags.host_received_classes_revealed.load(Ordering::SeqCst)
            && flags
                .joiner_received_classes_revealed
                .load(Ordering::SeqCst),
        "both clients should receive S2CClassesRevealed: {}",
        flags.report()
    );
    assert!(
        flags.server_room_game_active.load(Ordering::SeqCst)
            && flags.server_round_draft_initial.load(Ordering::SeqCst),
        "server should promote the ready room into DraftInitial: {}",
        flags.report()
    );
    assert!(
        flags.host_received_draft_initial.load(Ordering::SeqCst)
            && flags.joiner_received_draft_initial.load(Ordering::SeqCst),
        "both clients should receive S2CPhaseChanged(DraftInitial): {}",
        flags.report()
    );
}

#[test]
fn real_lightyear_two_client_draft_initial_purchase_ready_reaches_draft_shop() {
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

        if flags.draft_shop_observed() {
            break;
        }

        thread::sleep(FRAME_SLEEP);
    }

    assert!(
        flags.host_received_draft_offering.load(Ordering::SeqCst)
            && flags.joiner_received_draft_offering.load(Ordering::SeqCst),
        "both clients should receive server-authored S2CDraftOffering: {}",
        flags.report()
    );
    assert_eq!(
        flags.host_draft_offering_count.load(Ordering::SeqCst),
        server_crate::feature::acquisition::DRAFT_INITIAL_OFFERING_COUNT as usize,
        "host offering should contain the server-authored draft count: {}",
        flags.report()
    );
    assert_eq!(
        flags.joiner_draft_offering_count.load(Ordering::SeqCst),
        server_crate::feature::acquisition::DRAFT_INITIAL_OFFERING_COUNT as usize,
        "joiner offering should contain the server-authored draft count: {}",
        flags.report()
    );
    assert!(
        flags.host_sent_purchase_card.load(Ordering::SeqCst)
            && flags.joiner_sent_purchase_card.load(Ordering::SeqCst),
        "both clients should send C2SPurchaseCard through Lightyear: {}",
        flags.report()
    );
    assert!(
        flags.host_received_card_acquired.load(Ordering::SeqCst)
            && flags.joiner_received_card_acquired.load(Ordering::SeqCst)
            && flags
                .host_card_acquired_source_draft_initial
                .load(Ordering::SeqCst)
            && flags
                .joiner_card_acquired_source_draft_initial
                .load(Ordering::SeqCst),
        "clients should receive authoritative S2CCardAcquired(DraftInitial): {}",
        flags.report()
    );
    assert!(
        flags
            .host_received_purchase_gold_update
            .load(Ordering::SeqCst)
            && flags
                .joiner_received_purchase_gold_update
                .load(Ordering::SeqCst),
        "clients should receive authoritative economy updates after purchase: {}",
        flags.report()
    );
    assert!(
        flags.server_host_hand_size.load(Ordering::SeqCst) >= 1
            && flags.server_joiner_hand_size.load(Ordering::SeqCst) >= 1,
        "server authoritative PlayerHands should contain purchased cards: {}",
        flags.report()
    );
    assert!(
        flags.host_sent_ready_initial.load(Ordering::SeqCst)
            && flags.host_sent_ready_retract.load(Ordering::SeqCst)
            && flags.host_sent_ready_final.load(Ordering::SeqCst)
            && flags.joiner_sent_ready.load(Ordering::SeqCst)
            && flags.server_retract_path_observed.load(Ordering::SeqCst),
        "ready/retract/ready path should be observed by server RSM: {}",
        flags.report()
    );
    assert!(
        flags.server_round_placement.load(Ordering::SeqCst)
            && flags.host_received_placement.load(Ordering::SeqCst)
            && flags.joiner_received_placement.load(Ordering::SeqCst),
        "DraftInitial all-ready should leave the server-owned phase through Placement: {}",
        flags.report()
    );
    assert!(
        flags.host_sent_placement_submit.load(Ordering::SeqCst)
            && flags.joiner_sent_placement_submit.load(Ordering::SeqCst)
            && flags.server_round_resolution.load(Ordering::SeqCst),
        "both clients should submit real C2SSubmitPlacement and reach Resolution: {}",
        flags.report()
    );
    assert!(
        flags.host_received_placement_reveal.load(Ordering::SeqCst)
            && flags
                .joiner_received_placement_reveal
                .load(Ordering::SeqCst)
            && flags.host_received_resolution_event.load(Ordering::SeqCst)
            && flags
                .joiner_received_resolution_event
                .load(Ordering::SeqCst),
        "clients should receive server-authored placement reveal and resolution event: {}",
        flags.report()
    );
    assert!(
        flags.server_round_draft_shop.load(Ordering::SeqCst)
            && flags.host_received_draft_shop.load(Ordering::SeqCst)
            && flags.joiner_received_draft_shop.load(Ordering::SeqCst),
        "server should progress to DraftShop and notify both clients: {}",
        flags.report()
    );
}

#[derive(Clone, Resource)]
struct HandshakeFlags {
    client_sent_hello: Arc<AtomicBool>,
    server_mapped_player: Arc<AtomicBool>,
    client_received_handshake: Arc<AtomicBool>,
    client_connected_count: Arc<AtomicUsize>,
    server_connection_count: Arc<AtomicUsize>,
    client_handshake_player: Arc<AtomicU64>,
}

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
    host_sent_placement_submit: Arc<AtomicBool>,
    joiner_sent_placement_submit: Arc<AtomicBool>,
    server_round_resolution: Arc<AtomicBool>,
    host_received_resolution: Arc<AtomicBool>,
    joiner_received_resolution: Arc<AtomicBool>,
    host_received_placement_reveal: Arc<AtomicBool>,
    joiner_received_placement_reveal: Arc<AtomicBool>,
    host_received_resolution_event: Arc<AtomicBool>,
    joiner_received_resolution_event: Arc<AtomicBool>,
    server_round_draft_shop: Arc<AtomicBool>,
    host_received_draft_shop: Arc<AtomicBool>,
    joiner_received_draft_shop: Arc<AtomicBool>,
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
            host_sent_placement_submit: Arc::new(AtomicBool::new(false)),
            joiner_sent_placement_submit: Arc::new(AtomicBool::new(false)),
            server_round_resolution: Arc::new(AtomicBool::new(false)),
            host_received_resolution: Arc::new(AtomicBool::new(false)),
            joiner_received_resolution: Arc::new(AtomicBool::new(false)),
            host_received_placement_reveal: Arc::new(AtomicBool::new(false)),
            joiner_received_placement_reveal: Arc::new(AtomicBool::new(false)),
            host_received_resolution_event: Arc::new(AtomicBool::new(false)),
            joiner_received_resolution_event: Arc::new(AtomicBool::new(false)),
            server_round_draft_shop: Arc::new(AtomicBool::new(false)),
            host_received_draft_shop: Arc::new(AtomicBool::new(false)),
            joiner_received_draft_shop: Arc::new(AtomicBool::new(false)),
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

    fn report(&self) -> String {
        let base = format!(
            "host_hello_sent={}, joiner_hello_sent={}, host_handshake={}, joiner_handshake={}, host_player_id={}, joiner_player_id={}, server_connections={}, server_rooms={}, host_create_sent={}, host_room_created={}, room_code_captured={}, joiner_join_sent={}, joiner_join_ack={}, host_slot_update={}, host_select_sent={}, joiner_select_sent={}, host_confirm_sent={}, joiner_confirm_sent={}, host_locked={}, joiner_locked={}, host_revealed={}, joiner_revealed={}, reveal_player_count={}, server_room_game_active={}, server_round_draft_initial={}, host_draft_initial={}, joiner_draft_initial={}",
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
        );
        format!(
            "{base}, host_offering={}, joiner_offering={}, host_offering_count={}, joiner_offering_count={}, host_purchase_card_id={}, joiner_purchase_card_id={}, host_purchase_sent={}, joiner_purchase_sent={}, host_acquired={}, joiner_acquired={}, host_acquired_source_draft_initial={}, joiner_acquired_source_draft_initial={}, host_purchase_gold_update={}, joiner_purchase_gold_update={}, server_host_hand_size={}, server_joiner_hand_size={}, server_host_ready_seen={}, host_ready_initial={}, host_ready_retract={}, host_ready_final={}, joiner_ready={}, server_retract_path_observed={}, server_placement={}, host_placement={}, joiner_placement={}, host_placement_submit={}, joiner_placement_submit={}, server_resolution={}, host_resolution={}, joiner_resolution={}, host_placement_reveal={}, joiner_placement_reveal={}, host_resolution_event={}, joiner_resolution_event={}, server_draft_shop={}, host_draft_shop={}, joiner_draft_shop={}",
            self.host_received_draft_offering.load(Ordering::SeqCst),
            self.joiner_received_draft_offering.load(Ordering::SeqCst),
            self.host_draft_offering_count.load(Ordering::SeqCst),
            self.joiner_draft_offering_count.load(Ordering::SeqCst),
            self.host_purchase_card_id.load(Ordering::SeqCst),
            self.joiner_purchase_card_id.load(Ordering::SeqCst),
            self.host_sent_purchase_card.load(Ordering::SeqCst),
            self.joiner_sent_purchase_card.load(Ordering::SeqCst),
            self.host_received_card_acquired.load(Ordering::SeqCst),
            self.joiner_received_card_acquired.load(Ordering::SeqCst),
            self.host_card_acquired_source_draft_initial.load(Ordering::SeqCst),
            self.joiner_card_acquired_source_draft_initial.load(Ordering::SeqCst),
            self.host_received_purchase_gold_update.load(Ordering::SeqCst),
            self.joiner_received_purchase_gold_update.load(Ordering::SeqCst),
            self.server_host_hand_size.load(Ordering::SeqCst),
            self.server_joiner_hand_size.load(Ordering::SeqCst),
            self.server_host_ready_seen.load(Ordering::SeqCst),
            self.host_sent_ready_initial.load(Ordering::SeqCst),
            self.host_sent_ready_retract.load(Ordering::SeqCst),
            self.host_sent_ready_final.load(Ordering::SeqCst),
            self.joiner_sent_ready.load(Ordering::SeqCst),
            self.server_retract_path_observed.load(Ordering::SeqCst),
            self.server_round_placement.load(Ordering::SeqCst),
            self.host_received_placement.load(Ordering::SeqCst),
            self.joiner_received_placement.load(Ordering::SeqCst),
            self.host_sent_placement_submit.load(Ordering::SeqCst),
            self.joiner_sent_placement_submit.load(Ordering::SeqCst),
            self.server_round_resolution.load(Ordering::SeqCst),
            self.host_received_resolution.load(Ordering::SeqCst),
            self.joiner_received_resolution.load(Ordering::SeqCst),
            self.host_received_placement_reveal.load(Ordering::SeqCst),
            self.joiner_received_placement_reveal.load(Ordering::SeqCst),
            self.host_received_resolution_event.load(Ordering::SeqCst),
            self.joiner_received_resolution_event.load(Ordering::SeqCst),
            self.server_round_draft_shop.load(Ordering::SeqCst),
            self.host_received_draft_shop.load(Ordering::SeqCst),
            self.joiner_received_draft_shop.load(Ordering::SeqCst),
        )
    }
}

#[derive(Clone, Resource)]
struct RoomClientProbe {
    role: ClientRole,
    flags: RoomSessionFlags,
}

impl HandshakeFlags {
    fn new() -> Self {
        Self {
            client_sent_hello: Arc::new(AtomicBool::new(false)),
            server_mapped_player: Arc::new(AtomicBool::new(false)),
            client_received_handshake: Arc::new(AtomicBool::new(false)),
            client_connected_count: Arc::new(AtomicUsize::new(0)),
            server_connection_count: Arc::new(AtomicUsize::new(0)),
            client_handshake_player: Arc::new(AtomicU64::new(0)),
        }
    }

    fn handshake_observed(&self) -> bool {
        self.client_sent_hello.load(Ordering::SeqCst)
            && self.server_mapped_player.load(Ordering::SeqCst)
            && self.client_received_handshake.load(Ordering::SeqCst)
    }

    fn report(&self) -> String {
        format!(
            "client_sent_hello={}, server_mapped_player={}, client_received_handshake={}, client_connected_count={}, server_connection_count={}, client_handshake_player={}",
            self.client_sent_hello.load(Ordering::SeqCst),
            self.server_mapped_player.load(Ordering::SeqCst),
            self.client_received_handshake.load(Ordering::SeqCst),
            self.client_connected_count.load(Ordering::SeqCst),
            self.server_connection_count.load(Ordering::SeqCst),
            self.client_handshake_player.load(Ordering::SeqCst),
        )
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("server crate should live under workspace root")
        .to_path_buf()
}

fn assert_lightyear_feature(manifest: &str, package: &str, feature: &str) {
    let dependency = manifest
        .lines()
        .find(|line| line.trim_start().starts_with("lightyear = "))
        .unwrap_or_else(|| panic!("{package} manifest should declare lightyear dependency"));
    assert!(
        dependency.contains(&format!("\"{feature}\"")),
        "{package} lightyear dependency must include protocol-affecting feature {feature:?}: {dependency}"
    );
}

fn playable_card(id: u32, class: ClassId, cost: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class,
        family: Some("Playable E2E".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost,
        atk: 1,
        hp: 1,
        mp: 1,
        ar: 0,
        keywords: vec![],
        effect_text: String::new(),
        art_id: format!("playable_e2e_{id}"),
        pool_copies_override: Some(2),
    }
}

fn playable_e2e_catalog() -> CardCatalog {
    CardCatalog {
        cards: (1..=14)
            .map(|id| playable_card(id, ClassId::Iop, 1))
            .chain((101..=114).map(|id| playable_card(id, ClassId::Cra, 1)))
            .chain((201..=214).map(|id| playable_card(id, ClassId::Neutral, 1)))
            .map(|card| (card.id, card))
            .collect::<HashMap<_, _>>(),
    }
}

fn reserve_ephemeral_port() -> u16 {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("ephemeral localhost port should be available for real e2e smoke test");
    listener
        .local_addr()
        .expect("ephemeral listener should expose a local address")
        .port()
}

fn build_server_app(port: u16, flags: HandshakeFlags) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    app.init_resource::<PeerMetadata>();
    app.add_plugins(RsmPlugin);
    app.add_plugins(GameSessionPlugin);
    register_lightyear_protocol(&mut app);
    app.insert_resource(flags);
    app.add_systems(Startup, move |mut commands: Commands| {
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let config = ServerConfig::builder()
            .with_bind_address(bind_addr)
            .with_no_encryption();
        let server = commands
            .spawn((
                Name::new("PLAYABLE-003 Real E2E Smoke Server"),
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
            record_server_fresh_player_mapping,
            dispatch_phase_changed.after(advance_phase),
        ),
    );
    app.finish();
    app
}

fn build_client_app(url: String, flags: HandshakeFlags) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    register_lightyear_protocol(&mut app);
    app.insert_resource(flags);
    app.add_systems(Startup, move |mut commands: Commands| {
        let client = commands
            .spawn((
                Name::new("PLAYABLE-003 Real E2E Smoke Client"),
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
            record_client_connection_count,
            send_fresh_hello_until_handshake,
            record_client_handshake,
        )
            .chain(),
    );
    app.finish();
    app
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
    app.add_plugins(BoardPlugin);
    app.add_plugins(CombatPlugin);
    register_lightyear_protocol(&mut app);
    app.add_plugins(EconomyNetworkPlugin);
    app.insert_resource(server_crate::foundation::config::GameConfig(
        shared::config::GameConfig::default(),
    ));
    app.insert_resource(playable_e2e_catalog());
    app.insert_resource(ServerRng::new());
    app.insert_resource(flags);
    app.add_systems(Startup, move |mut commands: Commands| {
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let config = ServerConfig::builder()
            .with_bind_address(bind_addr)
            .with_no_encryption();
        let server = commands
            .spawn((
                Name::new("PLAYABLE-003 Room Session Server"),
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
                Name::new(format!("PLAYABLE-003 Room Session {role:?} Client")),
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
            record_room_flow_s2c_messages,
        )
            .chain(),
    );
    app.finish();
    app
}

fn record_server_fresh_player_mapping(
    flags: Res<HandshakeFlags>,
    connections: Option<Res<PlayerConnectionMap>>,
) {
    let Some(connections) = connections else {
        return;
    };

    flags
        .server_connection_count
        .store(connections.0.len(), Ordering::SeqCst);
    if connections
        .0
        .values()
        .any(|player_id| *player_id == PlayerId(1))
    {
        flags.server_mapped_player.store(true, Ordering::SeqCst);
    }
}

fn record_client_connection_count(
    flags: Res<HandshakeFlags>,
    connected: Query<(), (With<Client>, With<Connected>)>,
) {
    flags
        .client_connected_count
        .store(connected.iter().count(), Ordering::SeqCst);
}

fn send_fresh_hello_until_handshake(
    flags: Res<HandshakeFlags>,
    mut senders: Query<&mut MessageSender<C2SHello>>,
) {
    if flags.client_received_handshake.load(Ordering::SeqCst) {
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
        flags.client_sent_hello.store(true, Ordering::SeqCst);
    }
}

fn record_client_handshake(
    flags: Res<HandshakeFlags>,
    mut receivers: Query<&mut MessageReceiver<S2CHandshake>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            flags
                .client_received_handshake
                .store(true, Ordering::SeqCst);
            flags
                .client_handshake_player
                .store(message.player_id.0, Ordering::SeqCst);
        }
    }
}

fn record_room_flow_server_state(
    flags: Res<RoomSessionFlags>,
    connections: Option<Res<PlayerConnectionMap>>,
    rooms: Option<Res<RoomSessions>>,
    round_state: Option<Res<RoundState>>,
    hands: Option<Res<PlayerHands>>,
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
            ServerRoundPhase::DraftShop => {
                flags.server_round_draft_shop.store(true, Ordering::SeqCst)
            }
            _ => {}
        }

        if let Some(host) = host {
            if state.draft_ready_players.contains(&host) {
                flags.server_host_ready_seen.store(true, Ordering::SeqCst);
            }
            if flags.host_sent_ready_retract.load(Ordering::SeqCst)
                && !state.draft_ready_players.contains(&host)
                && joiner
                    .map(|joiner| state.draft_ready_players.contains(&joiner))
                    .unwrap_or(false)
            {
                flags
                    .server_retract_path_observed
                    .store(true, Ordering::SeqCst);
            }
        }
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
    send_empty_placement_submit(&probe, &mut submit_placement);
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
            if probe.flags.host_sent_ready_retract.load(Ordering::SeqCst)
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

fn send_empty_placement_submit(
    probe: &RoomClientProbe,
    submit_placement: &mut Query<&mut MessageSender<C2SSubmitPlacement>>,
) {
    let (placement_received, sent) = match probe.role {
        ClientRole::Host => (
            &probe.flags.host_received_placement,
            &probe.flags.host_sent_placement_submit,
        ),
        ClientRole::Joiner => (
            &probe.flags.joiner_received_placement,
            &probe.flags.joiner_sent_placement_submit,
        ),
    };

    if !placement_received.load(Ordering::SeqCst) || sent.load(Ordering::SeqCst) {
        return;
    }

    if let Some(mut sender) = submit_placement.iter_mut().next() {
        sender.send::<ReliableChannel>(C2SSubmitPlacement {
            placements: Vec::new(),
        });
        sent.store(true, Ordering::SeqCst);
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

    for mut receiver in &mut placement_reveal {
        for _message in receiver.receive() {
            match probe.role {
                ClientRole::Host => probe
                    .flags
                    .host_received_placement_reveal
                    .store(true, Ordering::SeqCst),
                ClientRole::Joiner => probe
                    .flags
                    .joiner_received_placement_reveal
                    .store(true, Ordering::SeqCst),
            }
        }
    }

    for mut receiver in &mut resolution_event {
        for _message in receiver.receive() {
            match probe.role {
                ClientRole::Host => probe
                    .flags
                    .host_received_resolution_event
                    .store(true, Ordering::SeqCst),
                ClientRole::Joiner => probe
                    .flags
                    .joiner_received_resolution_event
                    .store(true, Ordering::SeqCst),
            }
        }
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
        (ClientRole::Host, ProtocolRoundPhase::DraftShop) => probe
            .flags
            .host_received_draft_shop
            .store(true, Ordering::SeqCst),
        (ClientRole::Joiner, ProtocolRoundPhase::DraftShop) => probe
            .flags
            .joiner_received_draft_shop
            .store(true, Ordering::SeqCst),
        _ => {}
    }
}
