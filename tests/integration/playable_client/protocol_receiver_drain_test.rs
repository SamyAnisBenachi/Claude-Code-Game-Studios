//! PROMPT 1246 — S18-PROTOCOL-RECEIVER-DRAIN-SMOKE-TESTS-001.
//!
//! Closes the false-confidence gap from PROMPT 1203 R17 / B-1203-X-01.
//!
//! Every existing shop/auction/hand UI test bypasses the real
//! `MessageReceiver<S2CFoo>` → drain → internal-UI-event path by
//! directly writing the internal UI events (e.g. `ShopAuctionShopSlotsReceived`).
//! If any of the wired drain systems were removed, renamed, or pointed at
//! the wrong writer, those tests would all still pass.
//!
//! These smoke tests use a live two-app WebSocket loopback (the same
//! topology proven in `placement_buffer_test.rs` and
//! `auction_fifo_ordering_test.rs`) so that the client truly receives bytes
//! through the lightyear stack, the `MessageReceiver<S2CFoo>` is populated
//! by the lightyear message plugin, and the **production** drain function
//! is the one that turns those into internal UI events.
//!
//! For each covered message type the assertion is the same shape that
//! direct-mutation tests already assume — i.e. *the internal UI event was
//! produced from the receiver-side message exactly once with the expected
//! payload*. If that property ever breaks, this test fails before the
//! direct-mutation suite ever runs.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use bevy::prelude::*;
use client::network::register_lightyear_protocol;
use client::presentation::board_rendering::{
    drain_placement_reveal_system, BoardLocalPlayer, BoardRenderState, PlacementRevealCollectState,
    ResolutionRevealWait,
};
use client::presentation::draft_shop_hand_bridge_fanout_system;
use client::state::{ClientIdempotencyState, CurrentClientPhase};
use client::ui::hand::HandUiCardAcquiredReceived;
use client::ui::shop_auction::{
    drain_auction_card_receiver_system, ShopAuctionAuctionCardReceived,
    ShopAuctionCardAcquiredReceived, ShopAuctionShopCardAcquiredReceived,
    ShopAuctionShopSlotsReceived,
};
use lightyear::prelude::client::{
    Client, ClientConfig, ClientPlugins, Connect, RawClient, WebSocketClientIo,
};
use lightyear::prelude::server::{
    ClientOf, RawServer, Server, ServerConfig, ServerPlugins, Start, WebSocketServerIo,
};
use lightyear::prelude::{Connected, LocalAddr, NetworkTarget, RemoteId, ServerMultiMessageSender};
use shared::card::CardId;
use shared::protocol::{
    CardSource, PlacedCardReveal, PlayTarget, ReliableChannel, S2CAuctionCard, S2CCardAcquired,
    S2CPlacementReveal, S2CShopSlots,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const TICK_HZ: f64 = 60.0;
const MAX_FRAMES: usize = 600;
const FRAME_SLEEP: Duration = Duration::from_millis(10);

// --------------------------------------------------------------------------
// Connection probe — server-side. Read by the polling helper to detect
// "client is linked, can send now".
// --------------------------------------------------------------------------

#[derive(Clone, Resource)]
struct ConnectionProbe {
    server_seen_client: Arc<AtomicUsize>,
}

impl ConnectionProbe {
    fn new() -> Self {
        Self {
            server_seen_client: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn client_count(&self) -> usize {
        self.server_seen_client.load(Ordering::SeqCst)
    }
}

fn record_server_connection_count(
    probe: Res<ConnectionProbe>,
    clients: Query<(), (With<ClientOf>, With<Connected>)>,
) {
    probe
        .server_seen_client
        .store(clients.iter().count(), Ordering::SeqCst);
}

// --------------------------------------------------------------------------
// `ServerSendOnce` is the once-flag used by the per-test server send
// systems. The send system runs every Update and short-circuits after the
// first successful send.
// --------------------------------------------------------------------------

#[derive(Resource, Default)]
struct ServerSendOnce {
    fired: AtomicBool,
}

// --------------------------------------------------------------------------
// Common scaffolding
// --------------------------------------------------------------------------

fn reserve_ephemeral_port() -> u16 {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("ephemeral localhost port should be available for protocol drain smoke test");
    listener
        .local_addr()
        .expect("ephemeral listener should expose a local address")
        .port()
}

fn install_server_core(app: &mut App, port: u16, probe: ConnectionProbe) {
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    register_lightyear_protocol(app);
    app.insert_resource(probe);
    app.init_resource::<ServerSendOnce>();
    app.add_systems(Startup, move |mut commands: Commands| {
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let config = ServerConfig::builder()
            .with_bind_address(bind_addr)
            .with_no_encryption();
        let server = commands
            .spawn((
                Name::new("Protocol Receiver Drain WebSocket Server"),
                LocalAddr(bind_addr),
                RawServer,
                WebSocketServerIo { config },
            ))
            .id();
        commands.trigger(Start { entity: server });
    });
    app.add_systems(Update, record_server_connection_count);
}

fn install_client_core(app: &mut App, url: String) {
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    register_lightyear_protocol(app);
    app.add_systems(Startup, move |mut commands: Commands| {
        let client = commands
            .spawn((
                Name::new("Protocol Receiver Drain WebSocket Client"),
                Client::default(),
                RawClient,
                LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
                WebSocketClientIo::from_url(ClientConfig::default(), url.clone()),
            ))
            .id();
        commands.trigger(Connect { entity: client });
    });
}

fn pump_until<F: FnMut(&App) -> bool>(server_app: &mut App, client_app: &mut App, mut done: F) {
    for _ in 0..MAX_FRAMES {
        server_app.update();
        client_app.update();

        if done(client_app) {
            return;
        }

        thread::sleep(FRAME_SLEEP);
    }

    panic!("client never observed the expected drained UI event in MAX_FRAMES");
}

fn read_messages<T: bevy::prelude::Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn warm_up_server(server_app: &mut App) {
    for _ in 0..30 {
        server_app.update();
        thread::sleep(FRAME_SLEEP);
    }
}

// --------------------------------------------------------------------------
// 1. S2CShopSlots -> draft_shop_hand_bridge_fanout_system
//                 -> ShopAuctionShopSlotsReceived
// --------------------------------------------------------------------------

#[test]
fn s2c_shop_slots_drains_to_shop_auction_shop_slots_received() {
    test_helpers::init_test_tracing();
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");
    let probe = ConnectionProbe::new();

    let mut server_app = App::new();
    install_server_core(&mut server_app, port, probe.clone());
    server_app.add_systems(Update, send_shop_slots_once);
    server_app.finish();

    warm_up_server(&mut server_app);

    let mut client_app = App::new();
    install_client_core(&mut client_app, url);
    // Register the production drain we are validating, plus all the
    // internal UI events it can write. The system under test is the same
    // function `PresentationPlugin` registers.
    client_app.add_message::<HandUiCardAcquiredReceived>();
    client_app.add_message::<ShopAuctionShopSlotsReceived>();
    client_app.add_message::<ShopAuctionCardAcquiredReceived>();
    client_app.add_message::<ShopAuctionShopCardAcquiredReceived>();
    client_app.add_message::<client::ui::hand::HandUiDraftOfferingReceived>();
    client_app.add_message::<client::ui::shop_auction::ShopAuctionDraftOfferingReceived>();
    client_app.add_systems(Update, draft_shop_hand_bridge_fanout_system);
    client_app.finish();

    pump_until(&mut server_app, &mut client_app, |app| {
        !read_messages::<ShopAuctionShopSlotsReceived>(app).is_empty()
    });

    let observed = read_messages::<ShopAuctionShopSlotsReceived>(&client_app);
    assert_eq!(
        observed.len(),
        1,
        "exactly one ShopAuctionShopSlotsReceived must be produced from one S2CShopSlots drain"
    );
    assert_eq!(
        observed[0].slots,
        vec![Some(CardId(7)), None, Some(CardId(11))],
        "drained slots must match the bytes the server sent"
    );

    // Sanity: card-acquired writers were not spuriously written.
    assert!(read_messages::<HandUiCardAcquiredReceived>(&client_app).is_empty());
    assert!(read_messages::<ShopAuctionCardAcquiredReceived>(&client_app).is_empty());
    assert!(read_messages::<ShopAuctionShopCardAcquiredReceived>(&client_app).is_empty());
}

fn send_shop_slots_once(
    flag: Res<ServerSendOnce>,
    mut sender: ServerMultiMessageSender,
    server: Query<&Server>,
    clients: Query<&RemoteId, With<ClientOf>>,
) {
    if flag.fired.load(Ordering::SeqCst) {
        return;
    }
    let Ok(server) = server.single() else {
        return;
    };
    let Some(remote) = clients.iter().next() else {
        return;
    };
    let target = NetworkTarget::Single(remote.0);

    let message = S2CShopSlots {
        slots: vec![Some(CardId(7)), None, Some(CardId(11))],
    };

    if sender
        .send::<S2CShopSlots, ReliableChannel>(&message, server, &target)
        .is_ok()
    {
        flag.fired.store(true, Ordering::SeqCst);
    }
}

// --------------------------------------------------------------------------
// 2. S2CAuctionCard -> drain_auction_card_receiver_system
//                  -> ShopAuctionAuctionCardReceived
// --------------------------------------------------------------------------

#[test]
fn s2c_auction_card_drains_to_shop_auction_auction_card_received() {
    test_helpers::init_test_tracing();
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");
    let probe = ConnectionProbe::new();

    let mut server_app = App::new();
    install_server_core(&mut server_app, port, probe.clone());
    server_app.add_systems(Update, send_auction_card_once);
    server_app.finish();

    warm_up_server(&mut server_app);

    let mut client_app = App::new();
    install_client_core(&mut client_app, url);
    client_app.add_message::<ShopAuctionAuctionCardReceived>();
    client_app.add_systems(Update, drain_auction_card_receiver_system);
    client_app.finish();

    pump_until(&mut server_app, &mut client_app, |app| {
        !read_messages::<ShopAuctionAuctionCardReceived>(app).is_empty()
    });

    let observed = read_messages::<ShopAuctionAuctionCardReceived>(&client_app);
    assert_eq!(observed.len(), 1);
    assert_eq!(
        observed[0],
        ShopAuctionAuctionCardReceived {
            card_id: CardId(42),
            starting_price: 6,
            timer_duration_ms: 18_000,
        }
    );
}

fn send_auction_card_once(
    flag: Res<ServerSendOnce>,
    mut sender: ServerMultiMessageSender,
    server: Query<&Server>,
    clients: Query<&RemoteId, With<ClientOf>>,
) {
    if flag.fired.load(Ordering::SeqCst) {
        return;
    }
    let Ok(server) = server.single() else {
        return;
    };
    let Some(remote) = clients.iter().next() else {
        return;
    };
    let target = NetworkTarget::Single(remote.0);

    let message = S2CAuctionCard {
        card_id: CardId(42),
        starting_price: 6,
        timer_duration_ms: 18_000,
    };

    if sender
        .send::<S2CAuctionCard, ReliableChannel>(&message, server, &target)
        .is_ok()
    {
        flag.fired.store(true, Ordering::SeqCst);
    }
}

// --------------------------------------------------------------------------
// 3a. S2CCardAcquired { source: DraftInitial }
//                 -> draft_shop_hand_bridge_fanout_system
//                 -> HandUiCardAcquiredReceived + ShopAuctionCardAcquiredReceived
//                 ; ShopAuctionShopCardAcquiredReceived must NOT fire on draft path.
// --------------------------------------------------------------------------

#[test]
fn s2c_card_acquired_draft_initial_drains_to_hand_and_draft_initial_writers() {
    test_helpers::init_test_tracing();
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");
    let probe = ConnectionProbe::new();

    let mut server_app = App::new();
    install_server_core(&mut server_app, port, probe.clone());
    server_app.add_systems(Update, send_card_acquired_draft_initial_once);
    server_app.finish();

    warm_up_server(&mut server_app);

    let mut client_app = App::new();
    install_client_core(&mut client_app, url);
    client_app.add_message::<HandUiCardAcquiredReceived>();
    client_app.add_message::<ShopAuctionShopSlotsReceived>();
    client_app.add_message::<ShopAuctionCardAcquiredReceived>();
    client_app.add_message::<ShopAuctionShopCardAcquiredReceived>();
    client_app.add_message::<client::ui::hand::HandUiDraftOfferingReceived>();
    client_app.add_message::<client::ui::shop_auction::ShopAuctionDraftOfferingReceived>();
    client_app.add_systems(Update, draft_shop_hand_bridge_fanout_system);
    client_app.finish();

    pump_until(&mut server_app, &mut client_app, |app| {
        !read_messages::<HandUiCardAcquiredReceived>(app).is_empty()
            && !read_messages::<ShopAuctionCardAcquiredReceived>(app).is_empty()
    });

    let hand = read_messages::<HandUiCardAcquiredReceived>(&client_app);
    let draft = read_messages::<ShopAuctionCardAcquiredReceived>(&client_app);
    let shop_purchase = read_messages::<ShopAuctionShopCardAcquiredReceived>(&client_app);
    assert_eq!(
        hand,
        vec![HandUiCardAcquiredReceived { card_id: CardId(3) }]
    );
    assert_eq!(
        draft,
        vec![ShopAuctionCardAcquiredReceived { card_id: CardId(3) }]
    );
    assert!(
        shop_purchase.is_empty(),
        "DraftInitial path must not produce the ShopPurchase writer"
    );
}

fn send_card_acquired_draft_initial_once(
    flag: Res<ServerSendOnce>,
    mut sender: ServerMultiMessageSender,
    server: Query<&Server>,
    clients: Query<&RemoteId, With<ClientOf>>,
) {
    if flag.fired.load(Ordering::SeqCst) {
        return;
    }
    let Ok(server) = server.single() else {
        return;
    };
    let Some(remote) = clients.iter().next() else {
        return;
    };
    let target = NetworkTarget::Single(remote.0);

    let message = S2CCardAcquired {
        card_id: CardId(3),
        source: CardSource::DraftInitial,
    };

    if sender
        .send::<S2CCardAcquired, ReliableChannel>(&message, server, &target)
        .is_ok()
    {
        flag.fired.store(true, Ordering::SeqCst);
    }
}

// --------------------------------------------------------------------------
// 3b. S2CCardAcquired { source: ShopPurchase }
//                  -> HandUiCardAcquiredReceived + ShopAuctionShopCardAcquiredReceived
//                  ; ShopAuctionCardAcquiredReceived (draft-initial) must NOT fire.
// --------------------------------------------------------------------------

#[test]
fn s2c_card_acquired_shop_purchase_drains_to_hand_and_shop_purchase_writers() {
    test_helpers::init_test_tracing();
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");
    let probe = ConnectionProbe::new();

    let mut server_app = App::new();
    install_server_core(&mut server_app, port, probe.clone());
    server_app.add_systems(Update, send_card_acquired_shop_purchase_once);
    server_app.finish();

    warm_up_server(&mut server_app);

    let mut client_app = App::new();
    install_client_core(&mut client_app, url);
    client_app.add_message::<HandUiCardAcquiredReceived>();
    client_app.add_message::<ShopAuctionShopSlotsReceived>();
    client_app.add_message::<ShopAuctionCardAcquiredReceived>();
    client_app.add_message::<ShopAuctionShopCardAcquiredReceived>();
    client_app.add_message::<client::ui::hand::HandUiDraftOfferingReceived>();
    client_app.add_message::<client::ui::shop_auction::ShopAuctionDraftOfferingReceived>();
    client_app.add_systems(Update, draft_shop_hand_bridge_fanout_system);
    client_app.finish();

    pump_until(&mut server_app, &mut client_app, |app| {
        !read_messages::<HandUiCardAcquiredReceived>(app).is_empty()
            && !read_messages::<ShopAuctionShopCardAcquiredReceived>(app).is_empty()
    });

    let hand = read_messages::<HandUiCardAcquiredReceived>(&client_app);
    let shop_purchase = read_messages::<ShopAuctionShopCardAcquiredReceived>(&client_app);
    let draft = read_messages::<ShopAuctionCardAcquiredReceived>(&client_app);
    assert_eq!(
        hand,
        vec![HandUiCardAcquiredReceived { card_id: CardId(9) }]
    );
    assert_eq!(
        shop_purchase,
        vec![ShopAuctionShopCardAcquiredReceived { card_id: CardId(9) }]
    );
    assert!(
        draft.is_empty(),
        "ShopPurchase path must not produce the DraftInitial writer"
    );
}

fn send_card_acquired_shop_purchase_once(
    flag: Res<ServerSendOnce>,
    mut sender: ServerMultiMessageSender,
    server: Query<&Server>,
    clients: Query<&RemoteId, With<ClientOf>>,
) {
    if flag.fired.load(Ordering::SeqCst) {
        return;
    }
    let Ok(server) = server.single() else {
        return;
    };
    let Some(remote) = clients.iter().next() else {
        return;
    };
    let target = NetworkTarget::Single(remote.0);

    let message = S2CCardAcquired {
        card_id: CardId(9),
        source: CardSource::ShopPurchase,
    };

    if sender
        .send::<S2CCardAcquired, ReliableChannel>(&message, server, &target)
        .is_ok()
    {
        flag.fired.store(true, Ordering::SeqCst);
    }
}

// --------------------------------------------------------------------------
// 4. S2CPlacementReveal -> drain_placement_reveal_system
//                 -> BoardRenderState::ResolutionReveal
//                 -> ResolutionRevealWait::is_active() == true
//                 -> PlacementRevealCollectState becomes pending with the
//                    opponent-only target captured.
// --------------------------------------------------------------------------

#[test]
fn s2c_placement_reveal_drains_to_resolution_reveal_state() {
    test_helpers::init_test_tracing();
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");
    let probe = ConnectionProbe::new();

    let mut server_app = App::new();
    install_server_core(&mut server_app, port, probe.clone());
    server_app.add_systems(Update, send_placement_reveal_once);
    server_app.finish();

    warm_up_server(&mut server_app);

    let mut client_app = App::new();
    install_client_core(&mut client_app, url);
    client_app
        .init_resource::<BoardLocalPlayer>()
        .init_resource::<CurrentClientPhase>()
        .init_resource::<ClientIdempotencyState>()
        .init_resource::<PlacementRevealCollectState>()
        .init_resource::<ResolutionRevealWait>()
        .init_resource::<BoardRenderState>();
    client_app
        .world_mut()
        .resource_mut::<BoardLocalPlayer>()
        .player_id = Some(PlayerId(1));
    client_app
        .world_mut()
        .resource_mut::<CurrentClientPhase>()
        .round = 4;
    client_app.add_systems(Update, drain_placement_reveal_system);
    client_app.finish();

    pump_until(&mut server_app, &mut client_app, |app| {
        *app.world().resource::<BoardRenderState>() == BoardRenderState::ResolutionReveal
    });

    assert_eq!(
        *client_app.world().resource::<BoardRenderState>(),
        BoardRenderState::ResolutionReveal
    );
    assert!(client_app
        .world()
        .resource::<ResolutionRevealWait>()
        .is_active());
    let collect = client_app.world().resource::<PlacementRevealCollectState>();
    assert!(collect.is_pending());
    // Only the opponent (PlayerId(2)) placement contributes a target —
    // local-player reveals are skipped by `reveal_target`.
    assert_eq!(collect.pending_target_count(), 1);
}

fn send_placement_reveal_once(
    flag: Res<ServerSendOnce>,
    mut sender: ServerMultiMessageSender,
    server: Query<&Server>,
    clients: Query<&RemoteId, With<ClientOf>>,
) {
    if flag.fired.load(Ordering::SeqCst) {
        return;
    }
    let Ok(server) = server.single() else {
        return;
    };
    let Some(remote) = clients.iter().next() else {
        return;
    };
    let target = NetworkTarget::Single(remote.0);

    let message = S2CPlacementReveal {
        placements: vec![
            // local player's own placement — must be filtered out
            PlacedCardReveal {
                owner_id: PlayerId(1),
                card_id: CardId(100),
                target: PlayTarget::BoardCell { lane: 2, cell: 3 },
            },
            // opponent's placement — must produce one collect target
            PlacedCardReveal {
                owner_id: PlayerId(2),
                card_id: CardId(101),
                target: PlayTarget::BoardCell { lane: 5, cell: 7 },
            },
        ],
    };

    if sender
        .send::<S2CPlacementReveal, ReliableChannel>(&message, server, &target)
        .is_ok()
    {
        flag.fired.store(true, Ordering::SeqCst);
    }
}
