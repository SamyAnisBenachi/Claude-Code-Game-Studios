use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use shared::protocol::{
    self, C2SAcknowledgeResult, C2SActivateCard, C2SConfirmClass, C2SCreateRoom, C2SHeartbeat,
    C2SHello, C2SJoinRoom, C2SPlaceBid, C2SPurchaseCard, C2SRefreshShop, C2SSelectClass,
    C2SSignalReady, C2SSubmitPlacement, ProtocolChannel, ProtocolDirection, ProtocolRegistry,
    UnreliableChannel,
};

pub struct ClientNetworkPlugin;

impl Plugin for ClientNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / 60.0),
        });

        register_lightyear_protocol(app);

        app.add_systems(Startup, connect_websocket_client)
            .add_systems(Update, c2s_sender_stubs);
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

fn c2s_sender_stubs(
    hello: Query<&MessageSender<C2SHello>>,
    create_room: Query<&MessageSender<C2SCreateRoom>>,
    join_room: Query<&MessageSender<C2SJoinRoom>>,
    select_class: Query<&MessageSender<C2SSelectClass>>,
    confirm_class: Query<&MessageSender<C2SConfirmClass>>,
    purchase_card: Query<&MessageSender<C2SPurchaseCard>>,
    refresh_shop: Query<&MessageSender<C2SRefreshShop>>,
    activate_card: Query<&MessageSender<C2SActivateCard>>,
    signal_ready: Query<&MessageSender<C2SSignalReady>>,
    place_bid: Query<&MessageSender<C2SPlaceBid>>,
    submit_placement: Query<&MessageSender<C2SSubmitPlacement>>,
    acknowledge_result: Query<&MessageSender<C2SAcknowledgeResult>>,
    heartbeat: Query<&MessageSender<C2SHeartbeat>>,
) {
    count_senders(hello);
    count_senders(create_room);
    count_senders(join_room);
    count_senders(select_class);
    count_senders(confirm_class);
    count_senders(purchase_card);
    count_senders(refresh_shop);
    count_senders(activate_card);
    count_senders(signal_ready);
    count_senders(place_bid);
    count_senders(submit_placement);
    count_senders(acknowledge_result);
    count_senders(heartbeat);
}

fn count_senders<M: Send + Sync + 'static>(senders: Query<&MessageSender<M>>) -> usize {
    senders.iter().count()
}

#[allow(dead_code)]
fn _heartbeat_sender_compile_proof(mut sender: MessageSender<C2SHeartbeat>) {
    sender.send::<UnreliableChannel>(C2SHeartbeat {});
}
