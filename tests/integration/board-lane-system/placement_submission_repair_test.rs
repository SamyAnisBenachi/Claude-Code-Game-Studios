//! Regression coverage for PROMPT 1079 — Server Placement Buffer + Spawn Repair.
//!
//! These tests pin two failure modes observed in run-7 of the 2026-05-17
//! manual-friend evidence batch (see AUDIT-1076-02 and AUDIT-1076-03 in
//! `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md`):
//!
//! * **R2 lost-submission**: a Player B minion submission targeting cell=1 was
//!   silently rejected by spawn-range validation. `close_placement_phase` then
//!   reported `pending_submissions=0` / `spawned_units=0` with no log line
//!   explaining why. The fix is observability: rejections must emit a `warn!`
//!   with the structured `PlacementSubmissionResult` reason, and the buffer
//!   must NOT carry rejected submissions.
//! * **R3 commit-without-spawn**: an Order/Spell card with `PlayTarget::Instant`
//!   was accepted, committed, and broadcast in the placement reveal — but
//!   produced no server-side entity. Clients then rendered a ghost placement
//!   ("grey square") for a non-existent unit. The fix is to filter the reveal
//!   so effect-only placements never appear in `S2CPlacementReveal`. The
//!   internal `PlacementCommitted` event still carries the full committed
//!   sequence so future spell handling can process them.
//!
//! These tests run against the real Bevy `World` / `App` and exercise the live
//! placement system pipeline (`handle_placement_submission` →
//! `close_placement_phase`) end-to-end.

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
    process_placement_submission, BoardConfig, BoardOccupancy, BoardPlugin, PendingPlacements,
    PlacementCommitted, PlacementSubmissionReceived, PlacementSubmissionResult, PlayerSubmission,
    SpawnRangeState, LANE_WIDE_CELL_SENTINEL,
};
use server::foundation::config::CardCatalog;
use server::network::register_lightyear_protocol;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{GameMode, PlacedCardSubmit, PlayTarget, S2CPlacementReveal};
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

fn order_card(id: u32, cost: u32) -> CardData {
    CardData {
        id: card_id(id),
        name_fr: format!("Ordre {id}"),
        name_en: format!("Order {id}"),
        class: ClassId::Ecaflip,
        family: None,
        rarity: Rarity::Rare,
        card_type: CardType::Order,
        unit_type: UnitType::Neutral,
        cost,
        atk: 0,
        hp: 0,
        mp: 0,
        ar: 0,
        keywords: vec![],
        effect_text: String::new(),
        art_id: format!("test_order_{id}"),
        pool_copies_override: Some(1),
    }
}

fn field_card(id: u32, cost: u32) -> CardData {
    CardData {
        id: card_id(id),
        name_fr: format!("Champ {id}"),
        name_en: format!("Field {id}"),
        class: ClassId::Sadida,
        family: None,
        rarity: Rarity::Common,
        card_type: CardType::Field,
        unit_type: UnitType::Neutral,
        cost,
        atk: 0,
        hp: 0,
        mp: 0,
        ar: 0,
        keywords: vec![],
        effect_text: String::new(),
        art_id: format!("test_field_{id}"),
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
// R2 — Buffer write/read path: process_placement_submission must drop INVALID
// submissions silently and retain VALID submissions. The buffer state observed
// by close_placement_phase must match the result enum reported here.
// =============================================================================

#[test]
fn r2_invalid_minion_cell_for_player_b_is_rejected_with_spawn_range_reason() {
    test_helpers::init_test_tracing();

    // Player 2 = Player B. BoardConfig::default() puts Player B at spawn cell 8;
    // cell=1 is on the opposing side of the lane and must be rejected.
    let cat = catalog(vec![minion_card(103, 2)]);
    let economies = PlayerEconomies(HashMap::from([(player(2), economy(5, 0))]));
    let hands = player_hands(vec![(player(2), vec![card_id(103)])]);
    let mut pending = PendingPlacements::default();

    let result = process_placement_submission(
        &mut pending,
        player(2),
        vec![submitted(
            card_id(103),
            PlayTarget::BoardCell { lane: 1, cell: 1 },
            2,
        )],
        Some(RoundPhase::Placement),
        Some(&session_config()),
        &BoardConfig::default(),
        &SpawnRangeState::default(),
        &BoardOccupancy::default(),
        Some(&cat),
        Some(&economies),
        Some(&hands),
    );

    assert_eq!(result, PlacementSubmissionResult::SpawnRangeRejected);
    assert!(
        pending.submissions.is_empty(),
        "rejected submissions must NOT enter the pending buffer"
    );
}

#[test]
fn r2_valid_minion_for_player_b_is_accepted_and_retained_in_buffer() {
    test_helpers::init_test_tracing();

    // Same player, same card — but at cell=8 which is inside Player B's spawn
    // range when fakes_destroyed=0.
    let cat = catalog(vec![minion_card(103, 2)]);
    let economies = PlayerEconomies(HashMap::from([(player(2), economy(5, 0))]));
    let hands = player_hands(vec![(player(2), vec![card_id(103)])]);
    let mut pending = PendingPlacements::default();

    let result = process_placement_submission(
        &mut pending,
        player(2),
        vec![submitted(
            card_id(103),
            PlayTarget::BoardCell { lane: 1, cell: 8 },
            2,
        )],
        Some(RoundPhase::Placement),
        Some(&session_config()),
        &BoardConfig::default(),
        &SpawnRangeState::default(),
        &BoardOccupancy::default(),
        Some(&cat),
        Some(&economies),
        Some(&hands),
    );

    assert_eq!(result, PlacementSubmissionResult::Accepted);
    let submission = pending
        .submissions
        .get(&player(2))
        .expect("valid submission must be present in the pending buffer");
    assert!(submission.is_final);
    assert_eq!(submission.placements.len(), 1);
    assert_eq!(submission.placements[0].card_id, card_id(103));
    assert!(matches!(
        submission.placements[0].target,
        PlayTarget::BoardCell { lane: 1, cell: 8 }
    ));
}

#[test]
fn r2_handle_placement_submission_system_drops_invalid_and_writes_no_submitted_event() {
    // Drives the full system through Bevy's Update schedule so the public
    // observability contract (no PlacementSubmitted on rejection) is enforced.
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
            placements: vec![submitted(
                card_id(103),
                PlayTarget::BoardCell { lane: 1, cell: 1 },
                2,
            )],
        },
    );
    app.update();

    let submitted_msgs = read_messages::<PlacementSubmitted>(&app);
    assert!(
        submitted_msgs.is_empty(),
        "rejected submission must not produce a PlacementSubmitted event ({} written)",
        submitted_msgs.len()
    );
    assert!(
        app.world()
            .resource::<PendingPlacements>()
            .submissions
            .is_empty(),
        "rejected submission must not populate PendingPlacements"
    );
}

#[test]
fn r2_handle_placement_submission_system_accepts_valid_and_writes_submitted_event() {
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
            placements: vec![submitted(
                card_id(103),
                PlayTarget::BoardCell { lane: 1, cell: 8 },
                2,
            )],
        },
    );
    app.update();

    let submitted_msgs = read_messages::<PlacementSubmitted>(&app);
    assert_eq!(
        submitted_msgs.len(),
        1,
        "valid submission must produce exactly one PlacementSubmitted event"
    );
    assert_eq!(submitted_msgs[0].player, player(2));
    let submission = app
        .world()
        .resource::<PendingPlacements>()
        .submissions
        .get(&player(2))
        .cloned()
        .expect("valid submission must populate PendingPlacements before close runs");
    assert!(submission.is_final);
    assert_eq!(submission.placements.len(), 1);
}

// =============================================================================
// R3 — Effect-only placements (Order / Spell with PlayTarget::Instant) MUST NOT
// appear in S2CPlacementReveal. They are still accepted into the buffer (so
// future spell-effect handlers can process them) but spawned_units==0 for an
// Instant target is now matched by reveal_placements_len==0 to keep the wire
// state internally consistent.
// =============================================================================

#[test]
fn r3_order_with_instant_target_is_accepted_into_buffer() {
    test_helpers::init_test_tracing();

    let cat = catalog(vec![order_card(7, 1)]);
    let economies = PlayerEconomies(HashMap::from([(player(2), economy(5, 0))]));
    let hands = player_hands(vec![(player(2), vec![card_id(7)])]);
    let mut pending = PendingPlacements::default();

    let result = process_placement_submission(
        &mut pending,
        player(2),
        vec![submitted(card_id(7), PlayTarget::Instant, 1)],
        Some(RoundPhase::Placement),
        Some(&session_config()),
        &BoardConfig::default(),
        &SpawnRangeState::default(),
        &BoardOccupancy::default(),
        Some(&cat),
        Some(&economies),
        Some(&hands),
    );

    assert_eq!(result, PlacementSubmissionResult::Accepted);
    let submission = pending
        .submissions
        .get(&player(2))
        .expect("Order/Instant submission must be retained for future spell handling");
    assert_eq!(submission.placements.len(), 1);
    assert!(matches!(
        submission.placements[0].target,
        PlayTarget::Instant
    ));
}

// =============================================================================
// End-to-end live-server test covering R2 valid + R3 effect-only reveal filter.
//
// Drives a real Bevy `App` with a Lightyear `Server` entity, seeds the pending
// buffer with one BoardCell Minion (R2 valid path, must spawn + appear in
// reveal) and one Instant Order (R3 path, must commit but be filtered out of
// the reveal), then fires `ResolutionPhaseEntered` to invoke
// `close_placement_phase`. Asserts:
//
//   * spawned_units = 1 (only the BoardCell placement spawns a unit).
//   * PlacementCommitted carries BOTH commits (the Instant Order is still
//     available to future spell-effect handlers via the internal event).
//   * The wire S2CPlacementReveal contains exactly 1 entry — the BoardCell
//     placement — and the Instant Order is filtered out so clients never paint
//     a ghost placement.
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
struct PlacementRevealProbe {
    messages: Arc<Mutex<Vec<S2CPlacementReveal>>>,
}

impl PlacementRevealProbe {
    fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn messages(&self) -> Vec<S2CPlacementReveal> {
        self.messages
            .lock()
            .expect("placement reveal probe should not be poisoned")
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

fn record_placement_reveals(
    probe: Res<PlacementRevealProbe>,
    mut receivers: Query<&mut MessageReceiver<S2CPlacementReveal>>,
) {
    for mut receiver in receivers.iter_mut() {
        for message in receiver.receive() {
            probe
                .messages
                .lock()
                .expect("placement reveal probe should not be poisoned")
                .push(message);
        }
    }
}

fn reserve_ephemeral_port() -> u16 {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("ephemeral localhost port should be available for placement repair test");
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
                    Name::new("Placement Repair WebSocket Server"),
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

fn build_live_client(url: String, probe: PlacementRevealProbe) -> App {
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
                    Name::new("Placement Repair WebSocket Client"),
                    Client::default(),
                    RawClient,
                    LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
                    WebSocketClientIo::from_url(ClientConfig::default(), url.clone()),
                ))
                .id();
            commands.trigger(Connect { entity: client });
        })
        .add_systems(Update, record_placement_reveals);
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
    panic!("client did not connect to live placement-repair server");
}

#[test]
fn r3_close_placement_phase_filters_instant_orders_from_reveal_but_still_commits_them() {
    test_helpers::init_test_tracing();
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");
    let connection_probe = ServerConnectionProbe::new();
    let reveal_probe = PlacementRevealProbe::new();

    let cat = catalog(vec![minion_card(103, 2), order_card(7, 1)]);
    let economies = PlayerEconomies(HashMap::from([
        (player(1), economy(5, 0)),
        (player(2), economy(5, 0)),
    ]));
    let hands = player_hands(vec![
        (player(1), vec![card_id(103)]),
        (player(2), vec![card_id(7)]),
    ]);

    let mut server_app = build_live_server(port, cat, economies, hands, connection_probe.clone());
    for _ in 0..30 {
        server_app.update();
        thread::sleep(FRAME_SLEEP);
    }
    let mut client_app = build_live_client(url, reveal_probe.clone());
    connect_apps(&mut server_app, &mut client_app, &connection_probe);

    // Seed pending placements directly: one BoardCell minion (spawnable),
    // one Instant order (effect-only). Validation is bypassed by writing
    // directly to PendingPlacements; this test focuses on the
    // close_placement_phase boundary, not the submission validation path.
    {
        let mut pending = server_app.world_mut().resource_mut::<PendingPlacements>();
        pending.submissions.insert(
            player(1),
            PlayerSubmission {
                placements: vec![server::feature::board::AcceptedPlacement {
                    owner_id: player(1),
                    card_id: card_id(103),
                    target: PlayTarget::BoardCell { lane: 1, cell: 1 },
                    current_mana_spend: 2,
                    reserve_mana_spend: 0,
                }],
                submitted_at: Duration::ZERO,
                is_final: true,
            },
        );
        pending.submissions.insert(
            player(2),
            PlayerSubmission {
                placements: vec![server::feature::board::AcceptedPlacement {
                    owner_id: player(2),
                    card_id: card_id(7),
                    target: PlayTarget::Instant,
                    current_mana_spend: 1,
                    reserve_mana_spend: 0,
                }],
                submitted_at: Duration::ZERO,
                is_final: true,
            },
        );
    }

    write_message(&mut server_app, ResolutionPhaseEntered { round: 3 });
    server_app.update();
    client_app.update();

    // Internal PlacementCommitted carries BOTH commits so combat / future spell
    // handlers can still process the Instant Order.
    let committed_events = read_messages::<PlacementCommitted>(&server_app);
    assert_eq!(committed_events.len(), 1);
    let event = &committed_events[0];
    assert_eq!(event.round_number, 3);
    assert_eq!(
        event.committed_placements.len(),
        2,
        "PlacementCommitted must include both the BoardCell Minion and the Instant Order"
    );
    assert_eq!(
        event.spawned_units.len(),
        1,
        "Instant Order must not produce a spawned entity; only the BoardCell Minion spawns"
    );
    let spawned = &event.spawned_units[0];
    assert_eq!(spawned.player, player(1));
    assert_eq!(spawned.card_id, card_id(103));
    assert_eq!(spawned.lane, 1);
    assert_eq!(spawned.cell, 1);

    // Wait for the wire reveal to reach the client; the filter must keep the
    // Order/Instant entry off the wire entirely.
    for _ in 0..MAX_FRAMES {
        if !reveal_probe.messages().is_empty() {
            break;
        }
        server_app.update();
        client_app.update();
        thread::sleep(FRAME_SLEEP);
    }

    let received = reveal_probe.messages();
    assert_eq!(received.len(), 1, "expected one S2CPlacementReveal");
    assert_eq!(
        received[0].placements.len(),
        1,
        "reveal must contain exactly the BoardCell placement; the Instant Order \
         must be filtered out so clients never paint a ghost grey square"
    );
    assert_eq!(received[0].placements[0].card_id, card_id(103));
    assert!(matches!(
        received[0].placements[0].target,
        PlayTarget::BoardCell { lane: 1, cell: 1 }
    ));
}

// =============================================================================
// LaneWide / Field spawn-accounting fix: previously a Field placement spawned
// an entity + registered occupancy but `spawn_committed_placement` returned
// `None`, causing `PlacementCommitted::spawned_units` to under-count Fields.
// =============================================================================

#[test]
fn lane_wide_field_placement_is_counted_in_spawned_units() {
    test_helpers::init_test_tracing();
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");
    let connection_probe = ServerConnectionProbe::new();
    let reveal_probe = PlacementRevealProbe::new();

    let cat = catalog(vec![field_card(200, 1)]);
    let economies = PlayerEconomies(HashMap::from([(player(1), economy(5, 0))]));
    let hands = player_hands(vec![(player(1), vec![card_id(200)])]);

    let mut server_app = build_live_server(port, cat, economies, hands, connection_probe.clone());
    for _ in 0..30 {
        server_app.update();
        thread::sleep(FRAME_SLEEP);
    }
    let mut client_app = build_live_client(url, reveal_probe.clone());
    connect_apps(&mut server_app, &mut client_app, &connection_probe);

    {
        let mut pending = server_app.world_mut().resource_mut::<PendingPlacements>();
        pending.submissions.insert(
            player(1),
            PlayerSubmission {
                placements: vec![server::feature::board::AcceptedPlacement {
                    owner_id: player(1),
                    card_id: card_id(200),
                    target: PlayTarget::LaneWide { lane: 3 },
                    current_mana_spend: 1,
                    reserve_mana_spend: 0,
                }],
                submitted_at: Duration::ZERO,
                is_final: true,
            },
        );
    }

    write_message(&mut server_app, ResolutionPhaseEntered { round: 4 });
    server_app.update();
    client_app.update();

    let committed_events = read_messages::<PlacementCommitted>(&server_app);
    assert_eq!(committed_events.len(), 1);
    let event = &committed_events[0];
    assert_eq!(
        event.spawned_units.len(),
        1,
        "Field placement must count toward spawned_units"
    );
    let spawned = &event.spawned_units[0];
    assert_eq!(spawned.player, player(1));
    assert_eq!(spawned.card_id, card_id(200));
    assert_eq!(spawned.lane, 3);
    assert_eq!(
        spawned.cell, LANE_WIDE_CELL_SENTINEL,
        "LaneWide placements must use the cell sentinel since they occupy the whole lane"
    );

    for _ in 0..MAX_FRAMES {
        if !reveal_probe.messages().is_empty() {
            break;
        }
        server_app.update();
        client_app.update();
        thread::sleep(FRAME_SLEEP);
    }

    let received = reveal_probe.messages();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].placements.len(), 1);
    assert!(matches!(
        received[0].placements[0].target,
        PlayTarget::LaneWide { lane: 3 }
    ));
}
