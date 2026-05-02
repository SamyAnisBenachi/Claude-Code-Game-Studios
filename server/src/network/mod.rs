use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use crate::core::rsm::{advance_phase, tick_disconnect_timers, PlayerHeartbeat};
use crate::core::session::PlayerConnectionMap;
use crate::lobby::handler::handle_class_choice;
use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use shared::protocol::{
    self, C2SAcknowledgeResult, C2SActivateCard, C2SHeartbeat, C2SHello, C2SPlaceBid,
    C2SSignalReady, C2SSubmitPlacement, ProtocolChannel, ProtocolDirection, ProtocolRegistry,
    ReliableChannel, S2CObjectiveIdentities,
};

pub mod rsm_dispatch;

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / 60.0),
        });

        register_lightyear_protocol(app);

        app.init_resource::<PlayerConnectionMap>()
            .add_message::<PlayerHeartbeat>()
            .add_systems(Startup, open_websocket_server)
            .add_systems(
                Update,
                (
                    receive_c2s_messages.before(tick_disconnect_timers),
                    handle_class_choice,
                    rsm_dispatch::dispatch_phase_changed.after(advance_phase),
                ),
            )
            .add_observer(log_client_connected)
            .add_observer(log_client_disconnected);
    }
}

pub fn register_lightyear_protocol(app: &mut App) {
    app.register_required_components::<ClientOf, Transport>();

    let mut registry = LightyearProtocolRegistry { app };
    protocol::register_protocol(&mut registry);
}

struct LightyearProtocolRegistry<'a> {
    app: &'a mut App,
}

impl ProtocolRegistry for LightyearProtocolRegistry<'_> {
    fn add_channel<C: Send + Sync + 'static>(&mut self, channel: ProtocolChannel) {
        let mode = match channel {
            ProtocolChannel::Reliable => ChannelMode::OrderedReliable(ReliableSettings::default()),
            ProtocolChannel::Unreliable => ChannelMode::UnorderedUnreliable,
        };

        self.app
            .add_channel::<C>(ChannelSettings { mode, ..default() })
            .add_direction(NetworkDirection::Bidirectional);
    }

    fn add_message<M: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static>(
        &mut self,
        direction: ProtocolDirection,
        _channel: ProtocolChannel,
    ) {
        let direction = match direction {
            ProtocolDirection::ClientToServer => NetworkDirection::ClientToServer,
            ProtocolDirection::ServerToClient => NetworkDirection::ServerToClient,
        };

        self.app.register_message::<M>().add_direction(direction);
    }
}

fn open_websocket_server(mut commands: Commands) {
    let port = std::env::var("SERVER_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(5000);
    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
    let config = ServerConfig::builder()
        .with_bind_address(bind_addr)
        .with_no_encryption();

    let server = commands
        .spawn((
            Name::new("Lanes and Lies WebSocket Server"),
            LocalAddr(bind_addr),
            RawServer,
            WebSocketServerIo { config },
        ))
        .id();
    commands.trigger(Start { entity: server });
}

fn log_client_connected(trigger: On<Add, Connected>, clients: Query<&RemoteId>) {
    let client_id = clients.get(trigger.entity).map(|remote| remote.0);
    info!("Client connected: {:?}", client_id);
}

fn log_client_disconnected(trigger: On<Add, Disconnected>, clients: Query<&RemoteId>) {
    let client_id = clients.get(trigger.entity).map(|remote| remote.0);
    info!("Client disconnected: {:?}", client_id);
}

fn receive_c2s_messages(
    hello: Query<&mut MessageReceiver<C2SHello>>,
    activate_card: Query<&mut MessageReceiver<C2SActivateCard>>,
    signal_ready: Query<&mut MessageReceiver<C2SSignalReady>>,
    place_bid: Query<&mut MessageReceiver<C2SPlaceBid>>,
    submit_placement: Query<&mut MessageReceiver<C2SSubmitPlacement>>,
    acknowledge_result: Query<&mut MessageReceiver<C2SAcknowledgeResult>>,
    heartbeat: Query<(&RemoteId, &mut MessageReceiver<C2SHeartbeat>)>,
    connections: Option<Res<PlayerConnectionMap>>,
    mut heartbeat_writer: MessageWriter<PlayerHeartbeat>,
) {
    log_received("C2SHello", hello);
    log_received("C2SActivateCard", activate_card);
    log_received("C2SSignalReady", signal_ready);
    log_received("C2SPlaceBid", place_bid);
    log_received("C2SSubmitPlacement", submit_placement);
    log_received("C2SAcknowledgeResult", acknowledge_result);
    forward_heartbeats(heartbeat, connections.as_deref(), &mut heartbeat_writer);
}

fn log_received<M: std::fmt::Debug + Send + Sync + 'static>(
    label: &str,
    mut receivers: Query<&mut MessageReceiver<M>>,
) {
    for mut receiver in receivers.iter_mut() {
        for msg in receiver.receive() {
            debug!("Received {}: {:?}", label, msg);
        }
    }
}

fn forward_heartbeats(
    mut receivers: Query<(&RemoteId, &mut MessageReceiver<C2SHeartbeat>)>,
    connections: Option<&PlayerConnectionMap>,
    heartbeats: &mut MessageWriter<PlayerHeartbeat>,
) {
    for (remote, mut receiver) in receivers.iter_mut() {
        for msg in receiver.receive() {
            debug!("Received C2SHeartbeat: {:?}", msg);
            let Some(player) =
                connections.and_then(|connections| connections.0.get(&remote.0).copied())
            else {
                continue;
            };
            heartbeats.write(PlayerHeartbeat { player });
        }
    }
}

#[allow(dead_code)]
fn _unicast_compile_proof(mut sender: ServerMultiMessageSender, server: &Server, peer_id: PeerId) {
    let msg = S2CObjectiveIdentities { identities: vec![] };
    // ADR-001 unicast compile-proof - verified NetworkTarget::Single syntax.
    let _ = sender.send::<S2CObjectiveIdentities, ReliableChannel>(
        &msg,
        server,
        &NetworkTarget::Single(peer_id),
    );
}
