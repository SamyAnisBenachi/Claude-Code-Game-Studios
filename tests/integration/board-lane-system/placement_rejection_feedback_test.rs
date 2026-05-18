//! PROMPT 1244 — S18-PLACEMENT-SUBMISSION-REJECTION-FEEDBACK-001.
//!
//! Server placement rejections previously logged a `warn!` line with the
//! structured `PlacementSubmissionResult` reason (R2 observability fix from
//! PROMPT 1079) but never told the originating client. The optimistic
//! `Submitted` view stayed sticky until the round timer expired, hiding
//! actionable error states (spawn-range / occupancy / insufficient mana /
//! invalid target). This test pins the new feedback path:
//!
//! - `handle_placement_submission` writes one `PlacementRejectionDispatch`
//!   per rejected submission, carrying the submitter's `PlayerId`, the
//!   originating Lightyear `peer_id`, and a protocol-facing
//!   `PlacementRejectedReason` mapped from the internal
//!   `PlacementSubmissionResult`.
//! - `send_placement_rejection_dispatches` drains those dispatches and
//!   unicasts `S2CPlacementRejected` to the originating peer. The live
//!   server/client connection test asserts exactly ONE message arrives on
//!   the wire, with the expected reason, for the submitting client only.
//!
//! Authority remains server-side: nothing here weakens the validation
//! pipeline. Accepted submissions still produce zero rejection dispatches.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::client::{
    Client, ClientConfig, ClientPlugins, Connect, RawClient, WebSocketClientIo,
};
use lightyear::prelude::server::{
    ClientOf, RawServer, ServerConfig, ServerPlugins, Start, WebSocketServerIo,
};
use lightyear::prelude::{Connected, LocalAddr, MessageReceiver};
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::rsm::{
    PlacementPhaseEntered, PlacementSubmitted, ResolutionPhaseEntered, RoundPhase, RoundState,
};
use server::core::session::SessionConfig;
use server::feature::acquisition::PlayerHands;
use server::feature::board::{
    placement_rejection_reason, BoardPlugin, PlacementRejectionDispatch,
    PlacementSubmissionReceived, PlacementSubmissionResult,
};
use server::foundation::config::CardCatalog;
use server::network::register_lightyear_protocol;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{
    GameMode, PlacedCardSubmit, PlacementRejectedReason, PlayTarget, S2CPlacementRejected,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const TICK_HZ: f64 = 60.0;
const MAX_FRAMES: usize = 600;
const FRAME_SLEEP: Duration = Duration::from_millis(10);

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn card_id(id: u32) -> CardId {
    CardId(id)
}

fn submitted(card_id: CardId, target: PlayTarget, current_mana_spend: u32) -> PlacedCardSubmit {
    PlacedCardSubmit {
        card_id,
        target,
        current_mana_spend,
        reserve_mana_spend: 0,
    }
}

fn minion_card(id: u32, cost: u32) -> CardData {
    CardData {
        id: card_id(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: None,
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost,
        atk: 2,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: vec![],
        effect_text: String::new(),
        art_id: format!("test_{id}"),
        pool_copies_override: Some(1),
    }
}

fn catalog(cards: Vec<CardData>) -> CardCatalog {
    CardCatalog {
        cards: cards.into_iter().map(|card| (card.id, card)).collect(),
    }
}

fn economy(current_mana: u32, reserve_mana: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold: 0,
        current_mana,
        reserve_mana,
        mana_cap: 10,
        reserved_gold: 0,
    }
}

fn session_config() -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(player(1), 0), (player(2), 1)]),
        class_map: HashMap::from([(player(1), ClassId::Iop), (player(2), ClassId::Ecaflip)]),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn player_hands(catalog_ids: Vec<(PlayerId, Vec<CardId>)>) -> PlayerHands {
    PlayerHands {
        hands: catalog_ids.into_iter().collect(),
    }
}

fn app_with_placement_systems(
    catalog: CardCatalog,
    economies: PlayerEconomies,
    hands: PlayerHands,
) -> App {
    let mut app = App::new();
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    })
    .add_plugins(BoardPlugin)
    .add_message::<PlacementPhaseEntered>()
    .add_message::<ResolutionPhaseEntered>()
    .insert_resource(RoundState {
        phase: RoundPhase::Placement,
        round_number: 2,
        ..RoundState::new()
    })
    .insert_resource(session_config())
    .insert_resource(catalog)
    .insert_resource(economies)
    .insert_resource(hands);
    app
}

fn write_message<T: bevy::prelude::Message>(app: &mut App, message: T) {
    app.world_mut().resource_mut::<Messages<T>>().write(message);
}

fn read_messages<T: bevy::prelude::Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

// =============================================================================
// Mapping: `PlacementSubmissionResult` → `PlacementRejectedReason`.
//
// Every rejection variant must map to its protocol-facing counterpart so the
// client receives a structured reason instead of a generic blanket variant.
// =============================================================================

#[test]
fn placement_rejection_reason_maps_every_rejection_variant() {
    test_helpers::init_test_tracing();

    assert_eq!(
        placement_rejection_reason(PlacementSubmissionResult::DiscardedWrongPhase),
        PlacementRejectedReason::WrongPhase,
    );
    assert_eq!(
        placement_rejection_reason(PlacementSubmissionResult::DuplicateFinalSubmission),
        PlacementRejectedReason::DuplicateFinalSubmission,
    );
    assert_eq!(
        placement_rejection_reason(PlacementSubmissionResult::UnknownPlayer),
        PlacementRejectedReason::UnknownPlayer,
    );
    assert_eq!(
        placement_rejection_reason(PlacementSubmissionResult::MissingCatalog),
        PlacementRejectedReason::MissingCatalog,
    );
    assert_eq!(
        placement_rejection_reason(PlacementSubmissionResult::MissingEconomy),
        PlacementRejectedReason::MissingEconomy,
    );
    assert_eq!(
        placement_rejection_reason(PlacementSubmissionResult::CardMissingFromCatalog),
        PlacementRejectedReason::CardMissingFromCatalog,
    );
    assert_eq!(
        placement_rejection_reason(PlacementSubmissionResult::CardNotInHand),
        PlacementRejectedReason::CardNotInHand,
    );
    assert_eq!(
        placement_rejection_reason(PlacementSubmissionResult::DuplicateCardId),
        PlacementRejectedReason::DuplicateCardId,
    );
    assert_eq!(
        placement_rejection_reason(PlacementSubmissionResult::InvalidTarget),
        PlacementRejectedReason::InvalidTarget,
    );
    assert_eq!(
        placement_rejection_reason(PlacementSubmissionResult::SpawnRangeRejected),
        PlacementRejectedReason::SpawnRangeRejected,
    );
    assert_eq!(
        placement_rejection_reason(PlacementSubmissionResult::OccupancyRejected),
        PlacementRejectedReason::OccupancyRejected,
    );
    assert_eq!(
        placement_rejection_reason(PlacementSubmissionResult::InsufficientMana),
        PlacementRejectedReason::InsufficientMana,
    );
    assert_eq!(
        placement_rejection_reason(PlacementSubmissionResult::OwnerMismatch),
        PlacementRejectedReason::OwnerMismatch,
    );
}

// =============================================================================
// `handle_placement_submission` queue path: every server rejection logs a
// `PlacementRejectionDispatch` carrying the submitter's PlayerId + peer_id +
// protocol-facing reason. Accepted submissions produce zero dispatches.
//
// These tests drive the system through the live Bevy schedule (no Lightyear
// wire connection); the wire-level "exactly one S2C to submitter" assertion
// lives in the live server/client test further below.
// =============================================================================

#[test]
fn spawn_range_rejection_writes_one_rejection_dispatch_for_submitter() {
    test_helpers::init_test_tracing();

    let mut app = app_with_placement_systems(
        catalog(vec![minion_card(103, 2)]),
        PlayerEconomies(HashMap::from([(player(2), economy(5, 0))])),
        player_hands(vec![(player(2), vec![card_id(103)])]),
    );

    write_message(
        &mut app,
        PlacementSubmissionReceived {
            player: player(2),
            peer_id: None,
            placements: vec![submitted(
                card_id(103),
                PlayTarget::BoardCell { lane: 1, cell: 1 },
                2,
            )],
        },
    );
    app.update();

    let dispatches = read_messages::<PlacementRejectionDispatch>(&app);
    assert_eq!(
        dispatches.len(),
        1,
        "spawn-range rejection MUST queue exactly one PlacementRejectionDispatch \
         for the submitting player; got {} dispatches",
        dispatches.len(),
    );
    assert_eq!(dispatches[0].player, player(2));
    assert_eq!(dispatches[0].peer_id, None);
    assert_eq!(
        dispatches[0].reason,
        PlacementRejectedReason::SpawnRangeRejected,
    );
    assert!(
        read_messages::<PlacementSubmitted>(&app).is_empty(),
        "rejected submission must NOT also queue a PlacementSubmitted event",
    );
}

#[test]
fn insufficient_mana_rejection_writes_insufficient_mana_reason() {
    test_helpers::init_test_tracing();

    // Card costs 5 mana but the economy only has 1 current + 0 reserve.
    let mut app = app_with_placement_systems(
        catalog(vec![minion_card(207, 5)]),
        PlayerEconomies(HashMap::from([(player(1), economy(1, 0))])),
        player_hands(vec![(player(1), vec![card_id(207)])]),
    );

    write_message(
        &mut app,
        PlacementSubmissionReceived {
            player: player(1),
            peer_id: None,
            placements: vec![PlacedCardSubmit {
                card_id: card_id(207),
                target: PlayTarget::BoardCell { lane: 1, cell: 1 },
                current_mana_spend: 5,
                reserve_mana_spend: 0,
            }],
        },
    );
    app.update();

    let dispatches = read_messages::<PlacementRejectionDispatch>(&app);
    assert_eq!(dispatches.len(), 1);
    assert_eq!(dispatches[0].player, player(1));
    assert_eq!(
        dispatches[0].reason,
        PlacementRejectedReason::InsufficientMana,
    );
}

#[test]
fn accepted_submission_writes_no_rejection_dispatch() {
    test_helpers::init_test_tracing();

    let mut app = app_with_placement_systems(
        catalog(vec![minion_card(103, 2)]),
        PlayerEconomies(HashMap::from([(player(2), economy(5, 0))])),
        player_hands(vec![(player(2), vec![card_id(103)])]),
    );

    write_message(
        &mut app,
        PlacementSubmissionReceived {
            player: player(2),
            peer_id: None,
            placements: vec![submitted(
                card_id(103),
                PlayTarget::BoardCell { lane: 1, cell: 8 },
                2,
            )],
        },
    );
    app.update();

    assert!(
        read_messages::<PlacementRejectionDispatch>(&app).is_empty(),
        "accepted submission must NOT queue any PlacementRejectionDispatch",
    );
    assert_eq!(
        read_messages::<PlacementSubmitted>(&app).len(),
        1,
        "accepted submission MUST queue exactly one PlacementSubmitted event",
    );
}

// =============================================================================
// End-to-end live-server test covering the wire path: exactly one
// `S2CPlacementRejected` reaches the submitting client (and no other client)
// for a representative invalid reason (spawn-range out of bounds).
// =============================================================================

#[derive(Clone, Resource)]
struct ServerConnectionProbe {
    connected_clients: Arc<AtomicUsize>,
}

impl ServerConnectionProbe {
    fn new() -> Self {
        Self {
            connected_clients: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn connected_clients(&self) -> usize {
        self.connected_clients.load(Ordering::SeqCst)
    }
}

#[derive(Clone, Resource)]
struct PlacementRejectedProbe {
    messages: Arc<Mutex<Vec<S2CPlacementRejected>>>,
}

impl PlacementRejectedProbe {
    fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn messages(&self) -> Vec<S2CPlacementRejected> {
        self.messages
            .lock()
            .expect("placement rejected probe should not be poisoned")
            .clone()
    }
}

fn record_connected_clients(
    probe: Res<ServerConnectionProbe>,
    clients: Query<(), (With<ClientOf>, With<Connected>)>,
) {
    probe
        .connected_clients
        .store(clients.iter().count(), Ordering::SeqCst);
}

fn record_placement_rejections(
    probe: Res<PlacementRejectedProbe>,
    mut receivers: Query<&mut MessageReceiver<S2CPlacementRejected>>,
) {
    for mut receiver in receivers.iter_mut() {
        for message in receiver.receive() {
            probe
                .messages
                .lock()
                .expect("placement rejected probe should not be poisoned")
                .push(message);
        }
    }
}

fn reserve_ephemeral_port() -> u16 {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("ephemeral localhost port should be available for placement rejection test");
    listener
        .local_addr()
        .expect("ephemeral listener should expose a local address")
        .port()
}

fn build_live_server(
    port: u16,
    cat: CardCatalog,
    economies: PlayerEconomies,
    hands: PlayerHands,
    probe: ServerConnectionProbe,
) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    register_lightyear_protocol(&mut app);
    app.add_plugins(BoardPlugin)
        .add_message::<PlacementPhaseEntered>()
        .add_message::<ResolutionPhaseEntered>()
        .insert_resource(RoundState {
            phase: RoundPhase::Placement,
            round_number: 2,
            ..RoundState::new()
        })
        .insert_resource(session_config())
        .insert_resource(cat)
        .insert_resource(economies)
        .insert_resource(hands)
        .insert_resource(probe)
        .add_systems(Startup, move |mut commands: Commands| {
            let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
            let config = ServerConfig::builder()
                .with_bind_address(bind_addr)
                .with_no_encryption();
            let server = commands
                .spawn((
                    Name::new("Placement Rejection WebSocket Server"),
                    LocalAddr(bind_addr),
                    RawServer,
                    WebSocketServerIo { config },
                ))
                .id();
            commands.trigger(Start { entity: server });
        })
        .add_systems(Update, record_connected_clients);
    app.finish();
    app
}

fn build_live_client(url: String, probe: PlacementRejectedProbe) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    register_lightyear_protocol(&mut app);
    app.insert_resource(probe)
        .add_systems(Startup, move |mut commands: Commands| {
            let client = commands
                .spawn((
                    Name::new("Placement Rejection WebSocket Client"),
                    Client::default(),
                    RawClient,
                    LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
                    WebSocketClientIo::from_url(ClientConfig::default(), url.clone()),
                ))
                .id();
            commands.trigger(Connect { entity: client });
        })
        .add_systems(Update, record_placement_rejections);
    app.finish();
    app
}

fn connect_apps(server_app: &mut App, client_app: &mut App, probe: &ServerConnectionProbe) {
    for _ in 0..MAX_FRAMES {
        server_app.update();
        client_app.update();
        if probe.connected_clients() > 0 {
            return;
        }
        thread::sleep(FRAME_SLEEP);
    }
    panic!("client did not connect to live placement-rejection server");
}

fn connected_peer_id(server_app: &mut App) -> lightyear::prelude::PeerId {
    let mut query = server_app
        .world_mut()
        .query_filtered::<&lightyear::prelude::RemoteId, (With<ClientOf>, With<Connected>)>();
    query
        .iter(server_app.world())
        .next()
        .map(|remote| remote.0)
        .expect("server should observe at least one Connected client before peer_id lookup")
}

#[test]
fn rejection_unicasts_exactly_one_s2c_placement_rejected_to_submitter() {
    test_helpers::init_test_tracing();
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");
    let connection_probe = ServerConnectionProbe::new();
    let rejected_probe = PlacementRejectedProbe::new();

    // Card cost = 2, economy = (5, 0), card in hand. The placement targets
    // an out-of-spawn-range cell to force SpawnRangeRejected.
    let cat = catalog(vec![minion_card(103, 2)]);
    let economies = PlayerEconomies(HashMap::from([(player(2), economy(5, 0))]));
    let hands = player_hands(vec![(player(2), vec![card_id(103)])]);

    let mut server_app = build_live_server(port, cat, economies, hands, connection_probe.clone());
    for _ in 0..30 {
        server_app.update();
        thread::sleep(FRAME_SLEEP);
    }
    let mut client_app = build_live_client(url, rejected_probe.clone());
    connect_apps(&mut server_app, &mut client_app, &connection_probe);

    // Resolve the live PeerId for the only connected client and feed it
    // through PlacementSubmissionReceived so the dispatcher can unicast to
    // exactly that peer.
    let peer_id = connected_peer_id(&mut server_app);

    write_message(
        &mut server_app,
        PlacementSubmissionReceived {
            player: player(2),
            peer_id: Some(peer_id),
            placements: vec![submitted(
                card_id(103),
                PlayTarget::BoardCell { lane: 1, cell: 1 },
                2,
            )],
        },
    );
    server_app.update();

    // Wait for the wire dispatch to reach the client.
    for _ in 0..MAX_FRAMES {
        if !rejected_probe.messages().is_empty() {
            break;
        }
        server_app.update();
        client_app.update();
        thread::sleep(FRAME_SLEEP);
    }

    let received = rejected_probe.messages();
    assert_eq!(
        received.len(),
        1,
        "expected exactly ONE S2CPlacementRejected on the wire; got {}",
        received.len(),
    );
    assert_eq!(
        received[0].reason,
        PlacementRejectedReason::SpawnRangeRejected,
    );
}
