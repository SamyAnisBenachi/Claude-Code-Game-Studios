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
use shared::card::CardId;
use shared::protocol::{ReliableChannel, RoundPhase, S2CAuctionCard, S2CPhaseChanged};

const TICK_HZ: f64 = 60.0;
const MAX_FRAMES: usize = 600;
const FRAME_SLEEP: Duration = Duration::from_millis(10);
const AUCTION_CARD_ID: CardId = CardId(7);
const AUCTION_STARTING_PRICE: u32 = 4;
const AUCTION_ROUND: u32 = 3;
const AUCTION_TIMER_MS: u32 = 20_000;

#[derive(Clone, Resource)]
struct FifoFlags {
    server_sent_pair: Arc<AtomicBool>,
    client_observed_card: Arc<AtomicBool>,
    client_observed_phase: Arc<AtomicBool>,
    client_connected_count: Arc<AtomicUsize>,
    client_linked_count: Arc<AtomicUsize>,
    server_client_count: Arc<AtomicUsize>,
    server_linked_count: Arc<AtomicUsize>,
    observations: Arc<Mutex<ObservationState>>,
}

impl FifoFlags {
    fn new() -> Self {
        Self {
            server_sent_pair: Arc::new(AtomicBool::new(false)),
            client_observed_card: Arc::new(AtomicBool::new(false)),
            client_observed_phase: Arc::new(AtomicBool::new(false)),
            client_connected_count: Arc::new(AtomicUsize::new(0)),
            client_linked_count: Arc::new(AtomicUsize::new(0)),
            server_client_count: Arc::new(AtomicUsize::new(0)),
            server_linked_count: Arc::new(AtomicUsize::new(0)),
            observations: Arc::new(Mutex::new(ObservationState::default())),
        }
    }

    fn all_observed(&self) -> bool {
        self.server_sent_pair.load(Ordering::SeqCst)
            && self.client_observed_card.load(Ordering::SeqCst)
            && self.client_observed_phase.load(Ordering::SeqCst)
    }

    fn report(&self) -> String {
        format!(
            "server_sent_pair={}, client_observed_card={}, client_observed_phase={}, client_connected_count={}, client_linked_count={}, server_client_count={}, server_linked_count={}, observations={:?}",
            self.server_sent_pair.load(Ordering::SeqCst),
            self.client_observed_card.load(Ordering::SeqCst),
            self.client_observed_phase.load(Ordering::SeqCst),
            self.client_connected_count.load(Ordering::SeqCst),
            self.client_linked_count.load(Ordering::SeqCst),
            self.server_client_count.load(Ordering::SeqCst),
            self.server_linked_count.load(Ordering::SeqCst),
            self.observations
                .lock()
                .map(|observations| observations.clone())
                .unwrap_or_default(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ObservedKind {
    AuctionCard,
    DraftAuctionPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MessageObservation {
    message_id: u16,
}

#[derive(Clone, Debug, Default)]
struct ObservationState {
    card: Option<MessageObservation>,
    phase: Option<MessageObservation>,
    card_payload: Option<(CardId, u32)>,
    phase_payload: Option<(RoundPhase, u32, u32)>,
    observed_order: Vec<ObservedKind>,
}

#[test]
fn auction_card_precedes_draft_auction_phase_on_reliable_channel() {
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");
    let flags = FifoFlags::new();

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
        flags.server_sent_pair.load(Ordering::SeqCst),
        "server never queued S2CAuctionCard then S2CPhaseChanged(DraftAuction): {}",
        flags.report()
    );
    assert!(
        flags.client_observed_card.load(Ordering::SeqCst),
        "client never observed S2CAuctionCard: {}",
        flags.report()
    );
    assert!(
        flags.client_observed_phase.load(Ordering::SeqCst),
        "client never observed S2CPhaseChanged(DraftAuction): {}",
        flags.report()
    );

    let observations = flags
        .observations
        .lock()
        .expect("FIFO observations mutex should not be poisoned")
        .clone();
    let card = observations
        .card
        .expect("auction card observation should be recorded");
    let phase = observations
        .phase
        .expect("draft auction phase observation should be recorded");

    assert_eq!(
        observations.card_payload,
        Some((AUCTION_CARD_ID, AUCTION_STARTING_PRICE))
    );
    assert_eq!(
        observations.phase_payload,
        Some((RoundPhase::DraftAuction, AUCTION_ROUND, AUCTION_TIMER_MS))
    );
    assert!(
        card.message_id < phase.message_id,
        "ReliableChannel message ids should preserve enqueue order: {:?}",
        observations
    );

    let card_position = observations
        .observed_order
        .iter()
        .position(|kind| *kind == ObservedKind::AuctionCard)
        .expect("auction card should be in observed order");
    let phase_position = observations
        .observed_order
        .iter()
        .position(|kind| *kind == ObservedKind::DraftAuctionPhase)
        .expect("draft auction phase should be in observed order");
    assert!(
        card_position <= phase_position,
        "client observed S2CPhaseChanged(DraftAuction) before S2CAuctionCard: {:?}",
        observations
    );
}

fn reserve_ephemeral_port() -> u16 {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("ephemeral localhost port should be available for FIFO websocket test");
    listener
        .local_addr()
        .expect("ephemeral listener should expose a local address")
        .port()
}

fn build_server_app(port: u16, flags: FifoFlags) -> App {
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
                Name::new("Auction FIFO WebSocket Server"),
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
            send_auction_card_then_phase_once,
        )
            .chain(),
    );
    app.finish();
    app
}

fn build_client_app(url: String, flags: FifoFlags) -> App {
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
                Name::new("Auction FIFO WebSocket Client"),
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
        (record_client_connection_counts, record_client_fifo_messages).chain(),
    );
    app.finish();
    app
}

fn record_server_connection_counts(
    flags: Res<FifoFlags>,
    clients: Query<(), With<ClientOf>>,
    linked_servers: Query<(), (With<RawServer>, With<Linked>)>,
) {
    flags
        .server_client_count
        .store(clients.iter().count(), Ordering::SeqCst);
    flags
        .server_linked_count
        .store(linked_servers.iter().count(), Ordering::SeqCst);
}

fn send_auction_card_then_phase_once(
    flags: Res<FifoFlags>,
    mut sender: ServerMultiMessageSender,
    server: Query<&Server>,
    clients: Query<&RemoteId, With<ClientOf>>,
) {
    if flags.server_sent_pair.load(Ordering::SeqCst) {
        return;
    }

    let Ok(server) = server.single() else {
        return;
    };
    let Some(remote) = clients.iter().next() else {
        return;
    };
    let target = NetworkTarget::Single(remote.0);

    let auction_card = S2CAuctionCard {
        card_id: AUCTION_CARD_ID,
        starting_price: AUCTION_STARTING_PRICE,
    };
    let phase_changed = S2CPhaseChanged {
        phase: RoundPhase::DraftAuction,
        round_number: AUCTION_ROUND,
        timer_duration_ms: AUCTION_TIMER_MS,
    };

    let sent_card = sender
        .send::<S2CAuctionCard, ReliableChannel>(&auction_card, server, &target)
        .is_ok();
    let sent_phase = sender
        .send::<S2CPhaseChanged, ReliableChannel>(&phase_changed, server, &target)
        .is_ok();

    if sent_card && sent_phase {
        flags.server_sent_pair.store(true, Ordering::SeqCst);
    }
}

fn record_client_connection_counts(
    flags: Res<FifoFlags>,
    connected: Query<(), (With<Client>, With<Connected>)>,
    linked: Query<(), (With<Client>, With<Linked>)>,
) {
    flags
        .client_connected_count
        .store(connected.iter().count(), Ordering::SeqCst);
    flags
        .client_linked_count
        .store(linked.iter().count(), Ordering::SeqCst);
}

fn record_client_fifo_messages(
    flags: Res<FifoFlags>,
    mut receivers: Query<(
        &mut MessageReceiver<S2CAuctionCard>,
        &mut MessageReceiver<S2CPhaseChanged>,
    )>,
) {
    for (mut auction_cards, mut phases) in receivers.iter_mut() {
        for received in auction_cards.receive_with_tick() {
            let message_id = received
                .message_id
                .expect("S2CAuctionCard should arrive on OrderedReliable with a message id");

            let mut observations = flags
                .observations
                .lock()
                .expect("FIFO observations mutex should not be poisoned");
            observations.card = Some(MessageObservation {
                message_id: message_id.0,
            });
            observations.card_payload = Some((received.data.card_id, received.data.starting_price));
            observations.observed_order.push(ObservedKind::AuctionCard);
            flags.client_observed_card.store(true, Ordering::SeqCst);
        }

        for received in phases.receive_with_tick() {
            if received.data.phase != RoundPhase::DraftAuction {
                continue;
            }

            let message_id = received
                .message_id
                .expect("S2CPhaseChanged should arrive on OrderedReliable with a message id");

            let mut observations = flags
                .observations
                .lock()
                .expect("FIFO observations mutex should not be poisoned");
            observations.phase = Some(MessageObservation {
                message_id: message_id.0,
            });
            observations.phase_payload = Some((
                received.data.phase,
                received.data.round_number,
                received.data.timer_duration_ms,
            ));
            observations
                .observed_order
                .push(ObservedKind::DraftAuctionPhase);
            flags.client_observed_phase.store(true, Ordering::SeqCst);
        }
    }
}
