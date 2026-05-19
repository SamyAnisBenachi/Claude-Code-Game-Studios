use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use shared::protocol::{
    self, BotKind, C2SAddBot, C2SCreateBotRoom, C2SHeartbeat, C2SHello, C2SRemoveBot, GameMode,
    ProtocolChannel, ProtocolDirection, ProtocolRegistry, ReliableChannel, S2CBotActionRejected,
    UnreliableChannel,
};

pub struct ClientNetworkPlugin;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

impl Plugin for ClientNetworkPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("ClientNetworkPlugin loaded");
        app.add_plugins(ClientPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / 60.0),
        });

        register_lightyear_protocol(app);

        app.init_resource::<ClientHelloState>()
            .init_resource::<ClientHeartbeatTimer>()
            .add_systems(Startup, connect_websocket_client)
            .add_systems(Update, (send_fresh_hello_once, send_heartbeat_system));
    }
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ClientHelloState {
    pub sent: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct ClientHeartbeatTimer {
    timer: Timer,
}

impl Default for ClientHeartbeatTimer {
    fn default() -> Self {
        Self {
            timer: Timer::new(HEARTBEAT_INTERVAL, TimerMode::Repeating),
        }
    }
}

pub fn register_lightyear_protocol(app: &mut App) {
    app.register_required_components::<Client, Transport>();

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

fn connect_websocket_client(mut commands: Commands) {
    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "ws://localhost:5000".to_string());

    let client = commands
        .spawn((
            Name::new("Lanes and Lies WebSocket Client"),
            Client::default(),
            RawClient,
            LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
            WebSocketClientIo::from_url(ClientConfig::default(), server_url),
        ))
        .id();
    commands.trigger(Connect { entity: client });
}

fn send_fresh_hello_once(
    mut state: ResMut<ClientHelloState>,
    mut senders: Query<&mut MessageSender<C2SHello>>,
) {
    if state.sent {
        return;
    }

    let Some(mut sender) = senders.iter_mut().next() else {
        return;
    };

    tracing::info!(
        msg_type = "C2SHello",
        protocol_version = shared::config::GameConfig::default().protocol_version,
        "c2s_send: enter"
    );
    sender.send::<ReliableChannel>(C2SHello {
        protocol_version: shared::config::GameConfig::default().protocol_version,
        session_token: None,
    });
    state.sent = true;
}

fn send_heartbeat_system(
    time: Res<Time>,
    mut heartbeat_timer: ResMut<ClientHeartbeatTimer>,
    mut senders: Query<&mut MessageSender<C2SHeartbeat>>,
) {
    if !heartbeat_due_after_tick(&mut heartbeat_timer, time.delta()) {
        return;
    }

    let Some(mut sender) = senders.iter_mut().next() else {
        return;
    };

    tracing::info!(msg_type = "C2SHeartbeat", "c2s_send: enter");
    sender.send::<UnreliableChannel>(C2SHeartbeat {});
}

pub fn heartbeat_due_after_tick(timer: &mut ClientHeartbeatTimer, delta: Duration) -> bool {
    timer.timer.tick(delta).just_finished()
}

#[allow(dead_code)]
fn _heartbeat_sender_compile_proof(mut sender: MessageSender<C2SHeartbeat>) {
    tracing::info!(msg_type = "C2SHeartbeat", "c2s_send: enter (compile_proof)");
    sender.send::<UnreliableChannel>(C2SHeartbeat {});
}

#[allow(dead_code)]
fn _bot_protocol_sender_compile_proof(
    mut create_bot_room: MessageSender<C2SCreateBotRoom>,
    mut add_bot: MessageSender<C2SAddBot>,
    mut remove_bot: MessageSender<C2SRemoveBot>,
) {
    tracing::info!(
        msg_type = "C2SCreateBotRoom",
        "c2s_send: enter (compile_proof)"
    );
    create_bot_room.send::<ReliableChannel>(C2SCreateBotRoom {
        mode: GameMode::OneVOne,
        bot_kind: BotKind::Default,
    });

    tracing::info!(msg_type = "C2SAddBot", "c2s_send: enter (compile_proof)");
    add_bot.send::<ReliableChannel>(C2SAddBot {
        slot: 1,
        bot_kind: BotKind::Default,
    });

    tracing::info!(msg_type = "C2SRemoveBot", "c2s_send: enter (compile_proof)");
    remove_bot.send::<ReliableChannel>(C2SRemoveBot { slot: 1 });
}

#[allow(dead_code)]
fn _bot_action_rejected_receiver_compile_proof(
    mut receiver: MessageReceiver<S2CBotActionRejected>,
) {
    for msg in receiver.receive() {
        tracing::info!(
            msg_type = "S2CBotActionRejected",
            reason = ?msg.reason,
            "s2c_recv: enter (compile_proof)"
        );
    }
}
