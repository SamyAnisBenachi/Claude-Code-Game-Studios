use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use crate::core::rsm::{advance_phase, rsm_input_reader, tick_disconnect_timers, DraftReadySignal};
use crate::core::session::PlayerConnectionMap;
use crate::feature::board::{BoardSystemSet, PlacementSubmissionReceived};
use crate::lobby::handler::handle_class_choice;
use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use shared::protocol::{
    self, C2SActivateCard, C2SSignalReady, C2SSubmitPlacement, ProtocolChannel, ProtocolDirection,
    ProtocolRegistry, ReliableChannel, S2CObjectiveIdentities,
};

pub mod economy_dispatch;
pub mod rsm_dispatch;

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / 60.0),
        });

        register_lightyear_protocol(app);

        app.add_plugins(economy_dispatch::EconomyNetworkPlugin);

        app.init_resource::<PlayerConnectionMap>()
            .add_systems(Startup, open_websocket_server)
            .add_systems(
                Update,
                (
                    receive_c2s_messages.before(tick_disconnect_timers),
                    drain_signal_ready_messages.before(rsm_input_reader),
                    drain_submit_placement_messages.before(BoardSystemSet::PlacementSubmission),
                    handle_class_choice,
                    rsm_dispatch::dispatch_phase_changed.after(advance_phase),
                ),
            )
            .add_observer(insert_replication_sender_on_link)
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

/// Inserts [`ReplicationSender`] on every `LinkOf` entity the moment it appears.
///
/// Lightyear 0.26 does not auto-insert `ReplicationSender` on `ClientOf` entities;
/// without it every `ServerMultiMessageSender::send` call silently fails with
/// "entity does not have ReplicationSender". This observer fires once per client
/// connection and is the idiomatic fix (verified in integration test os18b).
fn insert_replication_sender_on_link(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(ReplicationSender::new(
            std::time::Duration::ZERO,
            SendUpdatesMode::SinceLastAck,
            false,
        ));
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
    mut activate_card: Query<(&RemoteId, &mut MessageReceiver<C2SActivateCard>)>,
) {
    for (remote, mut receiver) in activate_card.iter_mut() {
        for msg in receiver.receive() {
            tracing::info!(
                target: "server::game",
                peer_id = ?remote.0,
                card_id = ?msg.card_id,
                "c2s_activate_card: recv"
            );
        }
    }
}

pub fn drain_signal_ready_messages(
    connections: Res<PlayerConnectionMap>,
    mut receivers: Query<(&RemoteId, &mut MessageReceiver<C2SSignalReady>)>,
    mut ready_signals: MessageWriter<DraftReadySignal>,
) {
    for (remote, mut receiver) in receivers.iter_mut() {
        for msg in receiver.receive() {
            tracing::info!(
                target: "server::game",
                peer_id = ?remote.0,
                retract = msg.retract,
                "c2s_signal_ready: recv"
            );
            let Some(signal) = resolve_signal_ready_sender(&connections, remote.0, msg) else {
                debug!(
                    "C2SSignalReady discarded because sender is not mapped to a player: {:?}",
                    remote.0
                );
                continue;
            };

            ready_signals.write(signal);
        }
    }
}

pub fn resolve_signal_ready_sender(
    connections: &PlayerConnectionMap,
    peer_id: PeerId,
    msg: C2SSignalReady,
) -> Option<DraftReadySignal> {
    connections
        .0
        .get(&peer_id)
        .copied()
        .map(|player| DraftReadySignal {
            player,
            ready: !msg.retract,
        })
}

pub fn drain_submit_placement_messages(
    connections: Res<PlayerConnectionMap>,
    mut receivers: Query<(&RemoteId, &mut MessageReceiver<C2SSubmitPlacement>)>,
    mut submissions: MessageWriter<PlacementSubmissionReceived>,
) {
    for (remote, mut receiver) in receivers.iter_mut() {
        for msg in receiver.receive() {
            tracing::info!(
                target: "server::game",
                peer_id = ?remote.0,
                placements_len = msg.placements.len(),
                "c2s_submit_placement: recv"
            );
            let Some(resolved) = resolve_submit_placement_sender(&connections, remote.0, msg)
            else {
                debug!(
                    "C2SSubmitPlacement discarded because sender is not mapped to a player: {:?}",
                    remote.0
                );
                continue;
            };

            submissions.write(resolved);
        }
    }
}

pub fn resolve_submit_placement_sender(
    connections: &PlayerConnectionMap,
    peer_id: PeerId,
    msg: C2SSubmitPlacement,
) -> Option<PlacementSubmissionReceived> {
    connections
        .0
        .get(&peer_id)
        .copied()
        .map(|player| PlacementSubmissionReceived {
            player,
            placements: msg.placements,
        })
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn _unicast_compile_proof(mut sender: ServerMultiMessageSender, server: &Server, peer_id: PeerId) {
    let msg = S2CObjectiveIdentities { identities: vec![] };
    // ADR-001 unicast compile-proof - verified NetworkTarget::Single syntax.
    if let Err(e) = sender.send::<S2CObjectiveIdentities, ReliableChannel>(
        &msg,
        server,
        &NetworkTarget::Single(peer_id),
    ) {
        tracing::error!(
            target: "server::game",
            peer_id = ?peer_id,
            err = ?e,
            "S2C send failed: type=S2CObjectiveIdentities, handler=_unicast_compile_proof"
        );
    }
}
