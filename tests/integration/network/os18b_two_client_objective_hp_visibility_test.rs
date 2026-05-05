use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::thread;
use std::time::Duration;

extern crate server as server_crate;

use bevy::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use server_crate::core::rsm::{DraftStarted, ResolutionComplete};
use server_crate::core::session::{SessionConfig, TeamId};
use server_crate::feature::board::LaneId;
use server_crate::feature::objective::{
    take_damage, HiddenObjectives, ObjectiveCounters, ObjectiveDestroyed, ObjectiveHp,
    ObjectivePlugin, ObjectiveSlot, PendingObjectiveEvents, OBJECTIVE_LANE_COUNT,
};
use server_crate::foundation::config::GameConfig;
use server_crate::foundation::rng::ServerRng;
use server_crate::network::register_lightyear_protocol;
use shared::card::ClassId;
use shared::protocol::{DraftPhase, GameMode};
use shared::session::PlayerId;

const TICK_HZ: f64 = 60.0;
const MAX_FRAMES: usize = 900;
const SETTLE_FRAMES: usize = 12;
const FRAME_SLEEP: Duration = Duration::from_millis(10);
const PLAYER_A: PlayerId = PlayerId(1);
const PLAYER_B: PlayerId = PlayerId(2);
const TARGET_LANE: LaneId = 3;
const INITIAL_HP: u32 = 3;
const EXPECTED_OBJECTIVE_COUNT: usize = OBJECTIVE_LANE_COUNT as usize * 2;

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
struct ObjectiveHpObservations {
    armed: bool,
    values: Vec<u32>,
}

#[test]
fn test_os18b_two_clients_observe_only_final_objective_hp_after_same_substep_damage() {
    let port = reserve_ephemeral_port();
    let url = format!("ws://127.0.0.1:{port}");

    let mut server_app = build_server_app(port);
    for _ in 0..30 {
        server_app.update();
        thread::sleep(FRAME_SLEEP);
    }

    let mut client_a = build_client_app("OS-18b Client A", url.clone());
    let mut client_b = build_client_app("OS-18b Client B", url);

    pump_until(
        &mut server_app,
        &mut client_a,
        &mut client_b,
        live_connections_ready,
        "two clients connected and replication endpoints installed",
    );

    initialize_objective_fixture(&mut server_app);
    pump_until(
        &mut server_app,
        &mut client_a,
        &mut client_b,
        initial_objectives_replicated,
        "initial replicated ObjectiveHp state visible to both clients",
    );

    arm_observation(&mut client_a);
    arm_observation(&mut client_b);

    take_damage(server_app.world_mut(), TARGET_LANE, PLAYER_A, 2);
    take_damage(server_app.world_mut(), TARGET_LANE, PLAYER_A, 2);

    let server_hp = objective_hp(&mut server_app, PLAYER_B, TARGET_LANE);
    let queued = server_app
        .world()
        .resource::<PendingObjectiveEvents>()
        .queue
        .clone();
    let counters = server_app.world().resource::<ObjectiveCounters>().clone();

    server_app.world_mut().write_message(ResolutionComplete);
    server_app.update();

    let emitted = read_messages::<ObjectiveDestroyed>(&server_app);
    for _ in 0..SETTLE_FRAMES {
        client_a.update();
        client_b.update();
        server_app.update();
        thread::sleep(FRAME_SLEEP);
    }
    pump_until(
        &mut server_app,
        &mut client_a,
        &mut client_b,
        final_hp_observed_by_both_clients,
        "final ObjectiveHp observed by both clients",
    );
    for _ in 0..SETTLE_FRAMES {
        server_app.update();
        client_a.update();
        client_b.update();
        thread::sleep(FRAME_SLEEP);
    }

    let client_a_observations = observations(&client_a);
    let client_b_observations = observations(&client_b);

    assert_eq!(
        server_hp, 0,
        "server ObjectiveHp must saturate both damage calls"
    );
    assert_eq!(
        queued,
        vec![ObjectiveDestroyed {
            target_player_id: PLAYER_B,
            lane: TARGET_LANE,
            was_fake: false,
        }],
        "consequence path must queue exactly one ObjectiveDestroyed event"
    );
    assert_eq!(
        counters.real_objectives_destroyed(PLAYER_B),
        1,
        "real objective destruction counter should increment exactly once"
    );
    assert_eq!(
        emitted,
        vec![ObjectiveDestroyed {
            target_player_id: PLAYER_B,
            lane: TARGET_LANE,
            was_fake: false,
        }],
        "RESOLUTION-end sync must emit exactly one ObjectiveDestroyed message"
    );
    assert_eq!(
        client_a_observations,
        vec![0],
        "client A must observe final ObjectiveHp only; no intermediate or duplicate: {client_a_observations:?}"
    );
    assert_eq!(
        client_b_observations,
        vec![0],
        "client B must observe final ObjectiveHp only; no intermediate or duplicate: {client_b_observations:?}"
    );
    assert!(
        !client_a_observations.contains(&1) && !client_b_observations.contains(&1),
        "intermediate HP=1 must not be client-visible"
    );
}

fn reserve_ephemeral_port() -> u16 {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("ephemeral localhost port should be available for OS-18b test");
    listener
        .local_addr()
        .expect("ephemeral listener should expose a local address")
        .port()
}

fn build_server_app(port: u16) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    app.init_resource::<PeerMetadata>();
    register_lightyear_protocol(&mut app);
    app.add_message::<DraftStarted>();
    app.add_plugins(ObjectivePlugin);
    app.add_observer(add_replication_sender_to_connected_client);
    app.add_systems(Update, record_player_connections);
    app.add_systems(Startup, move |mut commands: Commands| {
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let config = ServerConfig::builder()
            .with_bind_address(bind_addr)
            .with_no_encryption();
        let server = commands
            .spawn((
                Name::new("OS-18b ObjectiveHp WebSocket Server"),
                LocalAddr(bind_addr),
                RawServer,
                WebSocketServerIo { config },
            ))
            .id();
        commands.trigger(Start { entity: server });
    });
    app.finish();
    app
}

fn build_client_app(name: &'static str, url: String) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    register_lightyear_protocol(&mut app);
    app.register_component::<ObjectiveHp>();
    app.init_resource::<ObjectiveHpObservations>();
    app.add_systems(Update, record_objective_hp_observations);
    app.add_systems(Startup, move |mut commands: Commands| {
        let client = commands
            .spawn((
                Name::new(name),
                Client::default(),
                RawClient,
                ReplicationReceiver::default(),
                LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
                WebSocketClientIo::from_url(ClientConfig::default(), url.clone()),
            ))
            .id();
        commands.trigger(Connect { entity: client });
    });
    app.finish();
    app
}

fn add_replication_sender_to_connected_client(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands.entity(trigger.entity).insert((
        ReplicationSender::new(Duration::ZERO, SendUpdatesMode::SinceLastAck, false),
        Name::new("OS-18b Replication Sender"),
    ));
}

fn record_player_connections(
    mut commands: Commands,
    peers: Query<&RemoteId, (With<ClientOf>, With<ReplicationSender>)>,
) {
    let mut connections = peers.iter().map(|remote| remote.0).collect::<Vec<_>>();
    connections.sort_by_key(|peer| format!("{peer:?}"));

    let mut map = HashMap::new();
    if let Some(peer) = connections.first() {
        map.insert(*peer, PLAYER_A);
    }
    if let Some(peer) = connections.get(1) {
        map.insert(*peer, PLAYER_B);
    }

    if !map.is_empty() {
        commands.insert_resource(server_crate::core::session::PlayerConnectionMap(map));
    }
}

fn record_objective_hp_observations(
    mut observations: ResMut<ObjectiveHpObservations>,
    hp: Query<&ObjectiveHp, Changed<ObjectiveHp>>,
) {
    if !observations.armed {
        return;
    }

    observations.values.extend(
        hp.iter()
            .filter_map(|hp| (hp.hp != INITIAL_HP).then_some(hp.hp)),
    );
}

fn initialize_objective_fixture(server_app: &mut App) {
    server_app.insert_resource(session_config());
    server_app.insert_resource(GameConfig(shared::config::GameConfig {
        objective_hp: INITIAL_HP,
        fake_count: 1,
        ..Default::default()
    }));
    server_app.insert_resource(ServerRng::from_seed(18));
    server_app.insert_resource(HiddenObjectives::default());
    server_app.insert_resource(PendingObjectiveEvents::default());

    server_app.world_mut().write_message(DraftStarted {
        round: 1,
        phase: DraftPhase::Initial,
    });
    server_app.update();

    server_app
        .world_mut()
        .resource_mut::<HiddenObjectives>()
        .identities
        .insert((PLAYER_B, TARGET_LANE), false);
}

fn session_config() -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(PLAYER_A, 0 as TeamId), (PLAYER_B, 1 as TeamId)]),
        class_map: HashMap::from([(PLAYER_A, ClassId::Iop), (PLAYER_B, ClassId::Cra)]),
    }
}

fn pump_until(
    server_app: &mut App,
    client_a: &mut App,
    client_b: &mut App,
    condition: fn(&mut App, &mut App, &mut App) -> bool,
    label: &str,
) {
    for _ in 0..MAX_FRAMES {
        server_app.update();
        client_a.update();
        client_b.update();

        if condition(server_app, client_a, client_b) {
            return;
        }

        thread::sleep(FRAME_SLEEP);
    }

    panic!("timed out waiting for {label}");
}

fn live_connections_ready(server_app: &mut App, client_a: &mut App, client_b: &mut App) -> bool {
    server_replication_sender_count(server_app) == 2
        && client_connected(client_a)
        && client_connected(client_b)
}

fn initial_objectives_replicated(
    _server_app: &mut App,
    client_a: &mut App,
    client_b: &mut App,
) -> bool {
    client_objective_hps(client_a) == vec![INITIAL_HP; EXPECTED_OBJECTIVE_COUNT]
        && client_objective_hps(client_b) == vec![INITIAL_HP; EXPECTED_OBJECTIVE_COUNT]
}

fn final_hp_observed_by_both_clients(
    _server_app: &mut App,
    client_a: &mut App,
    client_b: &mut App,
) -> bool {
    observations(client_a).contains(&0) && observations(client_b).contains(&0)
}

fn server_replication_sender_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query_filtered::<(), (With<ClientOf>, With<ReplicationSender>)>();
    query.iter(world).count()
}

fn client_connected(app: &mut App) -> bool {
    let world = app.world_mut();
    let mut query = world.query_filtered::<(), (With<Client>, With<Connected>)>();
    query.iter(world).next().is_some()
}

fn client_objective_hps(app: &mut App) -> Vec<u32> {
    let world = app.world_mut();
    let mut query = world.query::<&ObjectiveHp>();
    let mut values = query.iter(world).map(|hp| hp.hp).collect::<Vec<_>>();
    values.sort_unstable();
    values
}

fn arm_observation(client: &mut App) {
    let mut observations = client.world_mut().resource_mut::<ObjectiveHpObservations>();
    observations.armed = true;
    observations.values.clear();
}

fn observations(client: &App) -> Vec<u32> {
    client
        .world()
        .resource::<ObjectiveHpObservations>()
        .values
        .clone()
}

fn objective_hp(app: &mut App, player: PlayerId, lane: LaneId) -> u32 {
    let world = app.world_mut();
    let mut query = world.query::<(&ObjectiveSlot, &ObjectiveHp)>();
    query
        .iter(world)
        .find_map(|(slot, hp)| (slot.player == player && slot.lane == lane).then_some(hp.hp))
        .expect("target objective should exist")
}

fn read_messages<T: bevy::prelude::Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}
