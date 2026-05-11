use std::collections::HashMap;
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
use server_crate::core::economy::{PlayerEconomies, PlayerEconomy};
use server_crate::core::session::PlayerConnectionMap;
use server_crate::feature::auction::{
    process_bid_batch, AuctionBid, AuctionNetworkOutbox, AuctionPhase, AuctionState,
};
use server_crate::foundation::config::GameConfig;
use server_crate::network::register_lightyear_protocol;
use shared::card::CardId;
use shared::protocol::{
    BidRejectedReason, C2SPlaceBid, ReliableChannel, RoundPhase, S2CAuctionCard, S2CPhaseChanged,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const TICK_HZ: f64 = 60.0;
const MAX_FRAMES: usize = 600;
const FRAME_SLEEP: Duration = Duration::from_millis(10);
const AUCTION_CARD_ID: CardId = CardId(7);
const AUCTION_STARTING_PRICE: u32 = 4;
const AUCTION_ROUND: u32 = 3;
const AUCTION_TIMER_MS: u32 = 20_000;
const DUPLICATE_BID_AMOUNT: u32 = AUCTION_STARTING_PRICE + 1;
const PLAYER_A: PlayerId = PlayerId(1);
const PLAYER_B: PlayerId = PlayerId(2);

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
    test_helpers::init_test_tracing();
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

#[test]
fn two_clients_same_window_duplicate_bid_first_valid_wins_on_reliable_channel() {
    test_helpers::init_test_tracing();
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");
    let flags = TwoClientBidFlags::new();

    let mut server_app = build_two_client_bid_server_app(port, flags.clone());
    for _ in 0..30 {
        server_app.update();
        thread::sleep(FRAME_SLEEP);
    }

    let mut client_a = build_two_client_bid_client_app(
        "Auction FIFO Bid Client A",
        url.clone(),
        flags.clone(),
        BidClient::A,
    );
    for _ in 0..MAX_FRAMES {
        server_app.update();
        client_a.update();

        if flags.first_client_ready() {
            break;
        }

        thread::sleep(FRAME_SLEEP);
    }
    assert!(
        flags.first_client_ready(),
        "first auction bid client did not connect before arming bids: {}",
        flags.report()
    );

    let mut client_b = build_two_client_bid_client_app(
        "Auction FIFO Bid Client B",
        url,
        flags.clone(),
        BidClient::B,
    );
    for _ in 0..MAX_FRAMES {
        server_app.update();
        client_a.update();
        client_b.update();

        if flags.two_clients_ready() {
            break;
        }

        thread::sleep(FRAME_SLEEP);
    }
    assert!(
        flags.two_clients_ready(),
        "two auction bid clients did not connect before arming bids: {}",
        flags.report()
    );

    flags.bids_armed.store(true, Ordering::SeqCst);
    for _ in 0..MAX_FRAMES {
        client_a.update();
        client_b.update();
        server_app.update();

        if flags.two_client_fifo_observed() {
            break;
        }

        thread::sleep(FRAME_SLEEP);
    }

    assert!(
        flags.client_a_sent.load(Ordering::SeqCst),
        "client A never queued C2SPlaceBid on ReliableChannel: {}",
        flags.report()
    );
    assert!(
        flags.client_b_sent.load(Ordering::SeqCst),
        "client B never queued C2SPlaceBid on ReliableChannel: {}",
        flags.report()
    );
    assert!(
        flags.server_processed_batch.load(Ordering::SeqCst),
        "server never processed the two-client bid batch: {}",
        flags.report()
    );

    let observations = flags
        .bid_observations
        .lock()
        .expect("two-client bid observations mutex should not be poisoned")
        .clone();
    assert_eq!(
        observations.received.len(),
        2,
        "server should record exactly two competing bid inputs: {:?}",
        observations
    );
    assert!(
        observations
            .received
            .iter()
            .all(|received| received.server_frame <= observations.processed_frame),
        "competing auction inputs should be collected before the single bid batch is processed: {:?}",
        observations
    );
    assert_eq!(
        observations
            .received
            .iter()
            .map(|received| received.amount)
            .collect::<Vec<_>>(),
        vec![DUPLICATE_BID_AMOUNT, DUPLICATE_BID_AMOUNT],
        "both clients should submit the same conflicting bid amount: {:?}",
        observations
    );

    let first_bidder = observations.received[0].player_id;
    let second_bidder = observations.received[1].player_id;
    assert_ne!(
        first_bidder, second_bidder,
        "two-client FIFO evidence requires two distinct bidders: {:?}",
        observations
    );
    assert_eq!(
        observations.accepted,
        vec![(first_bidder, DUPLICATE_BID_AMOUNT, AUCTION_TIMER_MS)],
        "first observed valid bid should be the only accepted bid: {:?}",
        observations
    );
    assert_eq!(
        observations.rejected,
        vec![(second_bidder, BidRejectedReason::AmountTooLow)],
        "later same-amount conflict should be rejected deterministically: {:?}",
        observations
    );
    assert_eq!(observations.leader_before_batch, None);
    assert_eq!(observations.price_before_batch, AUCTION_STARTING_PRICE);
    assert_eq!(observations.final_leader, Some(first_bidder));
    assert_eq!(observations.final_price, DUPLICATE_BID_AMOUNT);
    assert_eq!(
        reserved_gold_for(&observations, first_bidder),
        DUPLICATE_BID_AMOUNT
    );
    assert_eq!(reserved_gold_for(&observations, second_bidder), 0);
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

fn build_two_client_bid_server_app(port: u16, flags: TwoClientBidFlags) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    app.init_resource::<PeerMetadata>();
    register_lightyear_protocol(&mut app);
    app.insert_resource(flags);
    app.insert_resource(TwoClientConnectionAssignments::default());
    app.insert_resource(PendingTwoClientBids::default());
    app.insert_resource(TwoClientServerFrame::default());
    app.insert_resource(two_client_live_auction());
    app.insert_resource(two_client_economies());
    app.insert_resource(two_client_auction_config());
    app.add_systems(Startup, move |mut commands: Commands| {
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let config = ServerConfig::builder()
            .with_bind_address(bind_addr)
            .with_no_encryption();
        let server = commands
            .spawn((
                Name::new("Auction FIFO Two-Client Bid Server"),
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
            advance_two_client_server_frame,
            record_two_client_bid_server_state,
            collect_and_process_two_client_bids_once,
        )
            .chain(),
    );
    app.finish();
    app
}

fn build_two_client_bid_client_app(
    name: &'static str,
    url: String,
    flags: TwoClientBidFlags,
    role: BidClient,
) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    register_lightyear_protocol(&mut app);
    app.insert_resource(flags);
    app.insert_resource(BidClientRole(role));
    app.add_systems(Startup, move |mut commands: Commands| {
        let client = commands
            .spawn((
                Name::new(name),
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
            record_two_client_bid_client_connection_counts,
            send_two_client_bid_once,
        )
            .chain(),
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
        timer_duration_ms: AUCTION_TIMER_MS,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BidClient {
    A,
    B,
}

#[derive(Resource)]
struct BidClientRole(BidClient);

#[derive(Clone, Resource)]
struct TwoClientBidFlags {
    bids_armed: Arc<AtomicBool>,
    client_a_sent: Arc<AtomicBool>,
    client_b_sent: Arc<AtomicBool>,
    server_processed_batch: Arc<AtomicBool>,
    client_a_connected_count: Arc<AtomicUsize>,
    client_b_connected_count: Arc<AtomicUsize>,
    server_client_count: Arc<AtomicUsize>,
    server_linked_count: Arc<AtomicUsize>,
    server_bid_receiver_count: Arc<AtomicUsize>,
    server_connection_map_count: Arc<AtomicUsize>,
    bid_observations: Arc<Mutex<TwoClientBidObservation>>,
}

impl TwoClientBidFlags {
    fn new() -> Self {
        Self {
            bids_armed: Arc::new(AtomicBool::new(false)),
            client_a_sent: Arc::new(AtomicBool::new(false)),
            client_b_sent: Arc::new(AtomicBool::new(false)),
            server_processed_batch: Arc::new(AtomicBool::new(false)),
            client_a_connected_count: Arc::new(AtomicUsize::new(0)),
            client_b_connected_count: Arc::new(AtomicUsize::new(0)),
            server_client_count: Arc::new(AtomicUsize::new(0)),
            server_linked_count: Arc::new(AtomicUsize::new(0)),
            server_bid_receiver_count: Arc::new(AtomicUsize::new(0)),
            server_connection_map_count: Arc::new(AtomicUsize::new(0)),
            bid_observations: Arc::new(Mutex::new(TwoClientBidObservation::default())),
        }
    }

    fn first_client_ready(&self) -> bool {
        self.client_a_connected_count.load(Ordering::SeqCst) == 1
            && self.server_client_count.load(Ordering::SeqCst) == 1
            && self.server_connection_map_count.load(Ordering::SeqCst) == 1
    }

    fn two_clients_ready(&self) -> bool {
        self.client_a_connected_count.load(Ordering::SeqCst) == 1
            && self.client_b_connected_count.load(Ordering::SeqCst) == 1
            && self.server_client_count.load(Ordering::SeqCst) == 2
            && self.server_connection_map_count.load(Ordering::SeqCst) == 2
            && self.server_bid_receiver_count.load(Ordering::SeqCst) == 2
    }

    fn two_client_fifo_observed(&self) -> bool {
        self.client_a_sent.load(Ordering::SeqCst)
            && self.client_b_sent.load(Ordering::SeqCst)
            && self.server_processed_batch.load(Ordering::SeqCst)
    }

    fn report(&self) -> String {
        format!(
            "bids_armed={}, client_a_sent={}, client_b_sent={}, server_processed_batch={}, client_a_connected_count={}, client_b_connected_count={}, server_client_count={}, server_linked_count={}, server_bid_receiver_count={}, server_connection_map_count={}, bid_observations={:?}",
            self.bids_armed.load(Ordering::SeqCst),
            self.client_a_sent.load(Ordering::SeqCst),
            self.client_b_sent.load(Ordering::SeqCst),
            self.server_processed_batch.load(Ordering::SeqCst),
            self.client_a_connected_count.load(Ordering::SeqCst),
            self.client_b_connected_count.load(Ordering::SeqCst),
            self.server_client_count.load(Ordering::SeqCst),
            self.server_linked_count.load(Ordering::SeqCst),
            self.server_bid_receiver_count.load(Ordering::SeqCst),
            self.server_connection_map_count.load(Ordering::SeqCst),
            self.bid_observations
                .lock()
                .map(|observations| observations.clone())
                .unwrap_or_default(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TwoClientBidReceived {
    player_id: PlayerId,
    amount: u32,
    server_frame: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct TwoClientBidObservation {
    received: Vec<TwoClientBidReceived>,
    processed_frame: usize,
    leader_before_batch: Option<PlayerId>,
    price_before_batch: u32,
    accepted: Vec<(PlayerId, u32, u32)>,
    rejected: Vec<(PlayerId, BidRejectedReason)>,
    final_leader: Option<PlayerId>,
    final_price: u32,
    player_a_reserved_gold: u32,
    player_b_reserved_gold: u32,
}

#[derive(Clone, Copy, Debug)]
struct PendingTwoClientBid {
    received: TwoClientBidReceived,
    bid: AuctionBid,
}

#[derive(Resource, Default)]
struct PendingTwoClientBids {
    bids: Vec<PendingTwoClientBid>,
}

#[derive(Resource, Default)]
struct TwoClientServerFrame(usize);

#[derive(Resource, Default)]
struct TwoClientConnectionAssignments {
    map: HashMap<PeerId, PlayerId>,
}

fn advance_two_client_server_frame(mut frame: ResMut<TwoClientServerFrame>) {
    frame.0 = frame.0.saturating_add(1);
}

fn record_two_client_bid_server_state(
    mut commands: Commands,
    flags: Res<TwoClientBidFlags>,
    mut assignments: ResMut<TwoClientConnectionAssignments>,
    clients: Query<&RemoteId, With<ClientOf>>,
    linked_servers: Query<(), (With<RawServer>, With<Linked>)>,
    bid_receivers: Query<&MessageReceiver<C2SPlaceBid>>,
) {
    for remote in clients.iter() {
        if assignments.map.contains_key(&remote.0) {
            continue;
        }

        let player = match assignments.map.len() {
            0 => PLAYER_A,
            1 => PLAYER_B,
            extra => PlayerId(u64::try_from(extra + 1).unwrap_or(u64::MAX)),
        };
        assignments.map.insert(remote.0, player);
    }

    flags
        .server_client_count
        .store(clients.iter().count(), Ordering::SeqCst);
    flags
        .server_linked_count
        .store(linked_servers.iter().count(), Ordering::SeqCst);
    flags
        .server_bid_receiver_count
        .store(bid_receivers.iter().count(), Ordering::SeqCst);
    flags
        .server_connection_map_count
        .store(assignments.map.len(), Ordering::SeqCst);

    commands.insert_resource(PlayerConnectionMap(assignments.map.clone()));
}

fn collect_and_process_two_client_bids_once(
    flags: Res<TwoClientBidFlags>,
    frame: Res<TwoClientServerFrame>,
    mut pending: ResMut<PendingTwoClientBids>,
    mut bid_receivers: Query<(&RemoteId, &mut MessageReceiver<C2SPlaceBid>)>,
    connections: Option<Res<PlayerConnectionMap>>,
    mut auction: ResMut<AuctionState>,
    mut economies: ResMut<PlayerEconomies>,
    config: Res<GameConfig>,
) {
    if !flags.bids_armed.load(Ordering::SeqCst)
        || flags.server_processed_batch.load(Ordering::SeqCst)
    {
        return;
    }

    let Some(connections) = connections.as_deref() else {
        return;
    };

    for (remote, mut receiver) in bid_receivers.iter_mut() {
        for bid in receiver.receive() {
            let Some(player_id) = connections.0.get(&remote.0).copied() else {
                continue;
            };
            let auction_bid = AuctionBid {
                bidder: player_id,
                peer_id: Some(remote.0),
                amount: bid.amount,
            };
            pending.bids.push(PendingTwoClientBid {
                received: TwoClientBidReceived {
                    player_id,
                    amount: bid.amount,
                    server_frame: frame.0,
                },
                bid: auction_bid,
            });
        }
    }

    if pending.bids.len() < 2 {
        return;
    }

    let bids = pending
        .bids
        .iter()
        .map(|entry| entry.bid)
        .collect::<Vec<_>>();
    let mut outbox = AuctionNetworkOutbox::default();
    let mut gold_broadcasts = Vec::new();
    let leader_before_batch = auction.current_leader;
    let price_before_batch = auction.current_price;
    process_bid_batch(
        &mut auction,
        &mut economies,
        None,
        &config,
        bids,
        &mut outbox,
        &mut gold_broadcasts,
    );

    let mut observations = flags
        .bid_observations
        .lock()
        .expect("two-client bid observations mutex should not be poisoned");
    observations.received = pending.bids.iter().map(|entry| entry.received).collect();
    observations.processed_frame = frame.0;
    observations.leader_before_batch = leader_before_batch;
    observations.price_before_batch = price_before_batch;
    observations.accepted = outbox
        .accepted()
        .iter()
        .map(|dispatch| {
            (
                dispatch.player_id,
                dispatch.message.amount,
                dispatch.message.new_timer_ms,
            )
        })
        .collect();
    observations.rejected = outbox
        .rejected()
        .iter()
        .map(|dispatch| (dispatch.player_id, dispatch.message.reason))
        .collect();
    observations.final_leader = auction.current_leader;
    observations.final_price = auction.current_price;
    observations.player_a_reserved_gold = economies
        .0
        .get(&PLAYER_A)
        .map(|economy| economy.reserved_gold)
        .unwrap_or_default();
    observations.player_b_reserved_gold = economies
        .0
        .get(&PLAYER_B)
        .map(|economy| economy.reserved_gold)
        .unwrap_or_default();
    flags.server_processed_batch.store(true, Ordering::SeqCst);
}

fn record_two_client_bid_client_connection_counts(
    flags: Res<TwoClientBidFlags>,
    role: Res<BidClientRole>,
    connected: Query<(), (With<Client>, With<Connected>)>,
) {
    let count = connected.iter().count();
    match role.0 {
        BidClient::A => flags
            .client_a_connected_count
            .store(count, Ordering::SeqCst),
        BidClient::B => flags
            .client_b_connected_count
            .store(count, Ordering::SeqCst),
    }
}

fn send_two_client_bid_once(
    flags: Res<TwoClientBidFlags>,
    role: Res<BidClientRole>,
    mut senders: Query<&mut MessageSender<C2SPlaceBid>>,
) {
    if !flags.bids_armed.load(Ordering::SeqCst) {
        return;
    }

    let sent_flag = match role.0 {
        BidClient::A => &flags.client_a_sent,
        BidClient::B => &flags.client_b_sent,
    };
    if sent_flag.load(Ordering::SeqCst) {
        return;
    }

    let mut sent = false;
    for mut sender in senders.iter_mut() {
        sender.send::<ReliableChannel>(C2SPlaceBid {
            amount: DUPLICATE_BID_AMOUNT,
        });
        sent = true;
    }

    if sent {
        sent_flag.store(true, Ordering::SeqCst);
    }
}

fn two_client_live_auction() -> AuctionState {
    AuctionState {
        phase: AuctionPhase::LiveBidding,
        card_id: Some(AUCTION_CARD_ID),
        starting_price: AUCTION_STARTING_PRICE,
        current_price: AUCTION_STARTING_PRICE,
        current_leader: None,
        timer_remaining_ms: AUCTION_TIMER_MS,
    }
}

fn two_client_economies() -> PlayerEconomies {
    PlayerEconomies(HashMap::from([
        (PLAYER_A, bidder_economy(20, 0)),
        (PLAYER_B, bidder_economy(20, 0)),
    ]))
}

fn bidder_economy(gold: u32, reserved_gold: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold,
        current_mana: 0,
        reserve_mana: 0,
        mana_cap: 10,
        reserved_gold,
    }
}

fn two_client_auction_config() -> GameConfig {
    GameConfig(shared::config::GameConfig {
        auction_timer_seconds: AUCTION_TIMER_MS / 1000,
        ..shared::config::GameConfig::default()
    })
}

fn reserved_gold_for(observations: &TwoClientBidObservation, player: PlayerId) -> u32 {
    match player {
        PLAYER_A => observations.player_a_reserved_gold,
        PLAYER_B => observations.player_b_reserved_gold,
        _ => 0,
    }
}
