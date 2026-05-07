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
    advance_phase, RoundPhase as ServerRoundPhase, RoundState, RsmPlugin,
};
use server_crate::core::session::{
    GameSessionPlugin, LobbyState, PlayerConnectionMap, RoomSessions,
};
use server_crate::network::{register_lightyear_protocol, rsm_dispatch::dispatch_phase_changed};
use shared::card::ClassId;
use shared::protocol::{
    C2SConfirmClass, C2SCreateRoom, C2SHello, C2SJoinRoom, C2SSelectClass, GameMode,
    ReliableChannel, RoundPhase as ProtocolRoundPhase, S2CClassLocked, S2CClassesRevealed,
    S2CHandshake, S2CJoinAck, S2CPhaseChanged, S2CRoomCreated, S2CSlotUpdated,
};
use shared::session::PlayerId;

const TICK_HZ: f64 = 60.0;
const MAX_FRAMES: usize = 600;
const ROOM_FLOW_MAX_FRAMES: usize = 1_200;
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

    fn report(&self) -> String {
        format!(
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
    register_lightyear_protocol(&mut app);
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

    if round_state
        .as_deref()
        .map(|state| state.phase == ServerRoundPhase::DraftInitial)
        .unwrap_or(false)
    {
        flags
            .server_round_draft_initial
            .store(true, Ordering::SeqCst);
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
            if message.phase != ProtocolRoundPhase::DraftInitial {
                continue;
            }

            match probe.role {
                ClientRole::Host => probe
                    .flags
                    .host_received_draft_initial
                    .store(true, Ordering::SeqCst),
                ClientRole::Joiner => probe
                    .flags
                    .joiner_received_draft_initial
                    .store(true, Ordering::SeqCst),
            }
        }
    }
}
