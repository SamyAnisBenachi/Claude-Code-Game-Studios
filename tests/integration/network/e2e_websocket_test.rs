use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

extern crate server as server_crate;

use bevy::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use server_crate::network::register_lightyear_protocol;
use shared::protocol::{C2SHeartbeat, ReliableChannel, S2CHandshakeRejected, UnreliableChannel};

#[path = "../../test_helpers.rs"]
mod test_helpers;

const TICK_HZ: f64 = 60.0;
const MAX_FRAMES: usize = 600;
const FRAME_SLEEP: Duration = Duration::from_millis(10);

#[derive(Clone, Resource)]
struct RoundTripFlags {
    client_sent_heartbeat: Arc<AtomicBool>,
    server_received_heartbeat: Arc<AtomicBool>,
    server_sent_roundtrip_messages: Arc<AtomicBool>,
    client_received_reliable_message: Arc<AtomicBool>,
    client_connected_count: Arc<AtomicUsize>,
    client_linking_count: Arc<AtomicUsize>,
    client_linked_count: Arc<AtomicUsize>,
    client_unlinked_count: Arc<AtomicUsize>,
    client_transport_count: Arc<AtomicUsize>,
    client_unreliable_sender_count: Arc<AtomicUsize>,
    client_link_send_len: Arc<AtomicUsize>,
    client_link_recv_len: Arc<AtomicUsize>,
    server_client_count: Arc<AtomicUsize>,
    server_linked_count: Arc<AtomicUsize>,
    server_link_of_count: Arc<AtomicUsize>,
    server_link_of_linked_count: Arc<AtomicUsize>,
    server_transport_count: Arc<AtomicUsize>,
    server_unreliable_receiver_count: Arc<AtomicUsize>,
    server_link_send_len: Arc<AtomicUsize>,
    server_link_recv_len: Arc<AtomicUsize>,
    server_heartbeat_receiver_count: Arc<AtomicUsize>,
    client_unlinked_reason: Arc<Mutex<String>>,
}

impl RoundTripFlags {
    fn new() -> Self {
        Self {
            client_sent_heartbeat: Arc::new(AtomicBool::new(false)),
            server_received_heartbeat: Arc::new(AtomicBool::new(false)),
            server_sent_roundtrip_messages: Arc::new(AtomicBool::new(false)),
            client_received_reliable_message: Arc::new(AtomicBool::new(false)),
            client_connected_count: Arc::new(AtomicUsize::new(0)),
            client_linking_count: Arc::new(AtomicUsize::new(0)),
            client_linked_count: Arc::new(AtomicUsize::new(0)),
            client_unlinked_count: Arc::new(AtomicUsize::new(0)),
            client_transport_count: Arc::new(AtomicUsize::new(0)),
            client_unreliable_sender_count: Arc::new(AtomicUsize::new(0)),
            client_link_send_len: Arc::new(AtomicUsize::new(0)),
            client_link_recv_len: Arc::new(AtomicUsize::new(0)),
            server_client_count: Arc::new(AtomicUsize::new(0)),
            server_linked_count: Arc::new(AtomicUsize::new(0)),
            server_link_of_count: Arc::new(AtomicUsize::new(0)),
            server_link_of_linked_count: Arc::new(AtomicUsize::new(0)),
            server_transport_count: Arc::new(AtomicUsize::new(0)),
            server_unreliable_receiver_count: Arc::new(AtomicUsize::new(0)),
            server_link_send_len: Arc::new(AtomicUsize::new(0)),
            server_link_recv_len: Arc::new(AtomicUsize::new(0)),
            server_heartbeat_receiver_count: Arc::new(AtomicUsize::new(0)),
            client_unlinked_reason: Arc::new(Mutex::new(String::new())),
        }
    }

    fn all_observed(&self) -> bool {
        self.client_sent_heartbeat.load(Ordering::SeqCst)
            && self.server_received_heartbeat.load(Ordering::SeqCst)
            && self.server_sent_roundtrip_messages.load(Ordering::SeqCst)
            && self.client_received_reliable_message.load(Ordering::SeqCst)
    }

    fn report(&self) -> String {
        format!(
            "client_sent_heartbeat={}, server_received_heartbeat={}, server_sent_roundtrip_messages={}, client_received_reliable_message={}, client_connected_count={}, client_linking_count={}, client_linked_count={}, client_unlinked_count={}, client_transport_count={}, client_unreliable_sender_count={}, client_link_send_len={}, client_link_recv_len={}, client_unlinked_reason={:?}, server_client_count={}, server_linked_count={}, server_link_of_count={}, server_link_of_linked_count={}, server_transport_count={}, server_unreliable_receiver_count={}, server_link_send_len={}, server_link_recv_len={}, server_heartbeat_receiver_count={}",
            self.client_sent_heartbeat.load(Ordering::SeqCst),
            self.server_received_heartbeat.load(Ordering::SeqCst),
            self.server_sent_roundtrip_messages.load(Ordering::SeqCst),
            self.client_received_reliable_message.load(Ordering::SeqCst),
            self.client_connected_count.load(Ordering::SeqCst),
            self.client_linking_count.load(Ordering::SeqCst),
            self.client_linked_count.load(Ordering::SeqCst),
            self.client_unlinked_count.load(Ordering::SeqCst),
            self.client_transport_count.load(Ordering::SeqCst),
            self.client_unreliable_sender_count.load(Ordering::SeqCst),
            self.client_link_send_len.load(Ordering::SeqCst),
            self.client_link_recv_len.load(Ordering::SeqCst),
            self.client_unlinked_reason
                .lock()
                .map(|reason| reason.clone())
                .unwrap_or_else(|_| "<poisoned>".to_string()),
            self.server_client_count.load(Ordering::SeqCst),
            self.server_linked_count.load(Ordering::SeqCst),
            self.server_link_of_count.load(Ordering::SeqCst),
            self.server_link_of_linked_count.load(Ordering::SeqCst),
            self.server_transport_count.load(Ordering::SeqCst),
            self.server_unreliable_receiver_count.load(Ordering::SeqCst),
            self.server_link_send_len.load(Ordering::SeqCst),
            self.server_link_recv_len.load(Ordering::SeqCst),
            self.server_heartbeat_receiver_count.load(Ordering::SeqCst),
        )
    }
}

#[test]
fn e2e_websocket_heartbeat_roundtrip_and_reliable_channel() {
    test_helpers::init_test_tracing();
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");
    let flags = RoundTripFlags::new();

    let mut server_app = build_server_app(port, flags.clone());
    for _ in 0..30 {
        server_app.update();
        thread::sleep(FRAME_SLEEP);
    }

    let mut client_app = build_client_app(url, flags.clone());

    for _ in 0..MAX_FRAMES {
        server_app.update();
        client_app.update();

        if flags.all_observed() {
            break;
        }

        thread::sleep(FRAME_SLEEP);
    }

    assert!(
        flags.client_sent_heartbeat.load(Ordering::SeqCst),
        "client never queued C2SHeartbeat on UnreliableChannel: {}",
        flags.report()
    );
    assert!(
        flags.server_received_heartbeat.load(Ordering::SeqCst),
        "server did not receive C2SHeartbeat through MessageReceiver<C2SHeartbeat>: {}",
        flags.report()
    );
    assert!(
        flags.server_sent_roundtrip_messages.load(Ordering::SeqCst),
        "server did not send S2CHandshakeRejected: {}",
        flags.report()
    );
    assert!(
        flags
            .client_received_reliable_message
            .load(Ordering::SeqCst),
        "client did not receive S2CHandshakeRejected on ReliableChannel: {}",
        flags.report()
    );
}

fn reserve_ephemeral_port() -> u16 {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("ephemeral localhost port should be available for e2e websocket test");
    listener
        .local_addr()
        .expect("ephemeral listener should expose a local address")
        .port()
}

fn build_server_app(port: u16, flags: RoundTripFlags) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    app.init_resource::<PeerMetadata>();
    register_lightyear_protocol(&mut app);
    app.insert_resource(flags);
    app.add_systems(Startup, move |mut commands: Commands| {
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let config = ServerConfig::builder()
            .with_bind_address(bind_addr)
            .with_no_encryption();
        let server = commands
            .spawn((
                Name::new("E2E WebSocket Server"),
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
            record_server_connection_counts,
            record_server_heartbeat_received,
            send_server_roundtrip_messages,
        )
            .chain(),
    );
    app.finish();
    app
}

fn build_client_app(url: String, flags: RoundTripFlags) -> App {
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
                Name::new("E2E WebSocket Client"),
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
            record_client_connection_counts,
            send_client_heartbeat_until_received,
            record_client_reliable_message,
        )
            .chain(),
    );
    app.finish();
    app
}

fn send_client_heartbeat_until_received(
    flags: Res<RoundTripFlags>,
    mut senders: Query<&mut MessageSender<C2SHeartbeat>>,
) {
    if flags.server_received_heartbeat.load(Ordering::SeqCst) {
        return;
    }

    let mut sent = false;
    for mut sender in senders.iter_mut() {
        sender.send::<UnreliableChannel>(C2SHeartbeat {});
        sent = true;
    }

    if sent {
        flags.client_sent_heartbeat.store(true, Ordering::SeqCst);
    }
}

fn record_client_connection_counts(
    flags: Res<RoundTripFlags>,
    connected: Query<(), (With<Client>, With<Connected>)>,
    linking: Query<(), (With<Client>, With<Linking>)>,
    linked: Query<(), (With<Client>, With<Linked>)>,
    unlinked: Query<&Unlinked, With<Client>>,
    transports: Query<&Transport, With<Client>>,
    links: Query<&Link, With<Client>>,
) {
    flags
        .client_connected_count
        .store(connected.iter().count(), Ordering::SeqCst);
    flags
        .client_linking_count
        .store(linking.iter().count(), Ordering::SeqCst);
    flags
        .client_linked_count
        .store(linked.iter().count(), Ordering::SeqCst);
    flags
        .client_unlinked_count
        .store(unlinked.iter().count(), Ordering::SeqCst);
    flags
        .client_transport_count
        .store(transports.iter().count(), Ordering::SeqCst);
    flags.client_unreliable_sender_count.store(
        transports
            .iter()
            .filter(|transport| transport.has_sender::<UnreliableChannel>())
            .count(),
        Ordering::SeqCst,
    );
    let (send_len, recv_len) = links.iter().fold((0, 0), |(send, recv), link| {
        (send + link.send.len(), recv + link.recv.len())
    });
    flags.client_link_send_len.store(send_len, Ordering::SeqCst);
    flags.client_link_recv_len.store(recv_len, Ordering::SeqCst);
    if let Some(reason) = unlinked
        .iter()
        .next()
        .map(|unlinked| unlinked.reason.clone())
    {
        if let Ok(mut stored_reason) = flags.client_unlinked_reason.lock() {
            *stored_reason = reason;
        }
    }
}

fn record_server_connection_counts(
    flags: Res<RoundTripFlags>,
    clients: Query<(), With<ClientOf>>,
    linked_servers: Query<(), (With<RawServer>, With<Linked>)>,
    link_of: Query<(), With<LinkOf>>,
    linked_link_of: Query<(), (With<LinkOf>, With<Linked>)>,
    transports: Query<&Transport, With<ClientOf>>,
    links: Query<&Link, With<ClientOf>>,
    heartbeat_receivers: Query<&MessageReceiver<C2SHeartbeat>>,
) {
    flags
        .server_client_count
        .store(clients.iter().count(), Ordering::SeqCst);
    flags
        .server_linked_count
        .store(linked_servers.iter().count(), Ordering::SeqCst);
    flags
        .server_link_of_count
        .store(link_of.iter().count(), Ordering::SeqCst);
    flags
        .server_link_of_linked_count
        .store(linked_link_of.iter().count(), Ordering::SeqCst);
    flags
        .server_transport_count
        .store(transports.iter().count(), Ordering::SeqCst);
    flags.server_unreliable_receiver_count.store(
        transports
            .iter()
            .filter(|transport| transport.has_receiver::<UnreliableChannel>())
            .count(),
        Ordering::SeqCst,
    );
    let (send_len, recv_len) = links.iter().fold((0, 0), |(send, recv), link| {
        (send + link.send.len(), recv + link.recv.len())
    });
    flags.server_link_send_len.store(send_len, Ordering::SeqCst);
    flags.server_link_recv_len.store(recv_len, Ordering::SeqCst);
    flags
        .server_heartbeat_receiver_count
        .store(heartbeat_receivers.iter().count(), Ordering::SeqCst);
}

fn record_server_heartbeat_received(
    flags: Res<RoundTripFlags>,
    mut receivers: Query<&mut MessageReceiver<C2SHeartbeat>>,
) {
    for mut receiver in receivers.iter_mut() {
        for _msg in receiver.receive() {
            flags
                .server_received_heartbeat
                .store(true, Ordering::SeqCst);
        }
    }
}

fn send_server_roundtrip_messages(
    flags: Res<RoundTripFlags>,
    mut sender: ServerMultiMessageSender,
    server: Query<&Server>,
    clients: Query<(), With<ClientOf>>,
) {
    if flags.server_sent_roundtrip_messages.load(Ordering::SeqCst)
        || !flags.server_received_heartbeat.load(Ordering::SeqCst)
        || clients.is_empty()
    {
        return;
    }

    let Ok(server) = server.single() else {
        return;
    };

    let reliable = S2CHandshakeRejected {
        server_version: 1,
        client_version: 0,
    };

    let sent_reliable = sender
        .send::<S2CHandshakeRejected, ReliableChannel>(&reliable, server, &NetworkTarget::All)
        .is_ok();

    if sent_reliable {
        flags
            .server_sent_roundtrip_messages
            .store(true, Ordering::SeqCst);
    }
}

fn record_client_reliable_message(
    flags: Res<RoundTripFlags>,
    mut receivers: Query<&mut MessageReceiver<S2CHandshakeRejected>>,
) {
    for mut receiver in receivers.iter_mut() {
        for _msg in receiver.receive() {
            flags
                .client_received_reliable_message
                .store(true, Ordering::SeqCst);
        }
    }
}
