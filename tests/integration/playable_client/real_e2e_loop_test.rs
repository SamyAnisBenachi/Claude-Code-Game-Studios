use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

extern crate server as server_crate;

use bevy::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use server_crate::core::session::{GameSessionPlugin, PlayerConnectionMap};
use server_crate::network::register_lightyear_protocol;
use shared::protocol::{C2SHello, ReliableChannel, S2CHandshake};
use shared::session::PlayerId;

const TICK_HZ: f64 = 60.0;
const MAX_FRAMES: usize = 600;
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

#[derive(Clone, Resource)]
struct HandshakeFlags {
    client_sent_hello: Arc<AtomicBool>,
    server_mapped_player: Arc<AtomicBool>,
    client_received_handshake: Arc<AtomicBool>,
    client_connected_count: Arc<AtomicUsize>,
    server_connection_count: Arc<AtomicUsize>,
    client_handshake_player: Arc<AtomicU64>,
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
    app.add_systems(Update, record_server_fresh_player_mapping);
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
