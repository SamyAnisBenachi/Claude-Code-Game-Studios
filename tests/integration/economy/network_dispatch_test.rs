use std::collections::HashMap;

use bevy::prelude::*;
use lightyear::prelude::PeerId;
use server::core::economy::{
    EconomyPlugin, S2CGoldBroadcast as EconomyGoldBroadcast, S2CGoldUpdate as EconomyGoldUpdate,
};
use server::core::rsm::{DraftStarted, RsmPlugin};
use server::core::session::{PlayerConnectionMap, SessionConfig, TeamId};
use server::foundation::config::GameConfig;
use server::network::economy_dispatch::{EconomyNetworkOutbox, EconomyNetworkPlugin};
use shared::card::ClassId;
use shared::protocol::{
    self, DraftPhase, GameMode, ProtocolChannel, ProtocolDirection, ProtocolRegistry,
    S2CGoldBroadcast as ProtocolGoldBroadcast, S2CGoldUpdate as ProtocolGoldUpdate,
};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_config(player_a: PlayerId, player_b: PlayerId) -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(player_a, 0 as TeamId), (player_b, 1 as TeamId)]),
        class_map: HashMap::from([(player_a, ClassId::Iop), (player_b, ClassId::Cra)]),
    }
}

fn app_with_economy_dispatch() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RsmPlugin);
    app.add_plugins(EconomyPlugin);
    app.add_plugins(EconomyNetworkPlugin);
    app.insert_resource(GameConfig(shared::config::GameConfig::default()));
    app
}

#[test]
fn draft_started_dispatches_private_updates_to_owner_peers_only() {
    let player_a = player(1);
    let player_b = player(2);
    let peer_a = PeerId::Netcode(11);
    let peer_b = PeerId::Netcode(12);
    let mut app = app_with_economy_dispatch();
    app.insert_resource(session_config(player_b, player_a));
    app.insert_resource(PlayerConnectionMap(HashMap::from([
        (peer_b, player_b),
        (peer_a, player_a),
    ])));

    app.world_mut().write_message(DraftStarted {
        round: 1,
        phase: DraftPhase::Initial,
    });
    app.update();

    let outbox = app.world().resource::<EconomyNetworkOutbox>();
    let updates = outbox.gold_updates();
    let broadcasts = outbox.gold_broadcasts();

    assert_eq!(updates.len(), 2);
    assert_eq!(updates[0].player_id, player_a);
    assert_eq!(updates[0].peer_id, peer_a);
    assert_eq!(updates[0].message.gold, 5);
    assert_eq!(updates[0].message.current_mana, 1);
    assert_eq!(updates[0].message.reserve_mana, 0);
    assert_eq!(updates[0].message.mana_cap, 10);
    assert_eq!(updates[1].player_id, player_b);
    assert_eq!(updates[1].peer_id, peer_b);
    assert_ne!(updates[0].peer_id, peer_b);
    assert_ne!(updates[1].peer_id, peer_a);

    assert_eq!(broadcasts.len(), 2);
    assert_eq!(broadcasts[0].player_id, player_a);
    assert_eq!(broadcasts[0].gold, 5);
    assert_eq!(broadcasts[0].reserved_gold, 0);
    assert_eq!(broadcasts[1].player_id, player_b);
}

#[test]
fn queued_award_messages_dispatch_reliable_update_and_public_broadcast() {
    let player_a = player(1);
    let peer_a = PeerId::Netcode(11);
    let mut app = app_with_economy_dispatch();
    app.insert_resource(PlayerConnectionMap(HashMap::from([(peer_a, player_a)])));

    app.world_mut().write_message(EconomyGoldUpdate {
        player: player_a,
        gold: 7,
        current_mana: 2,
        reserve_mana: 1,
        mana_cap: 10,
    });
    app.world_mut().write_message(EconomyGoldBroadcast {
        player_id: player_a,
        gold: 7,
        reserved_gold: 3,
    });
    app.update();

    let outbox = app.world().resource::<EconomyNetworkOutbox>();
    let update = outbox
        .gold_updates()
        .first()
        .expect("queued award update dispatch");
    assert_eq!(outbox.gold_updates().len(), 1);
    assert_eq!(update.player_id, player_a);
    assert_eq!(update.peer_id, peer_a);
    assert_eq!(update.message.gold, 7);
    assert_eq!(update.message.current_mana, 2);
    assert_eq!(update.message.reserve_mana, 1);

    let broadcast = outbox
        .gold_broadcasts()
        .first()
        .expect("queued award broadcast dispatch");
    assert_eq!(outbox.gold_broadcasts().len(), 1);
    assert_eq!(broadcast.player_id, player_a);
    assert_eq!(broadcast.gold, 7);
    assert_eq!(broadcast.reserved_gold, 3);
}

#[test]
fn missing_connection_skips_private_update_without_blocking_broadcasts() {
    let player_a = player(1);
    let player_b = player(2);
    let mut app = app_with_economy_dispatch();
    app.insert_resource(PlayerConnectionMap(HashMap::from([(
        PeerId::Netcode(12),
        player_b,
    )])));

    app.world_mut().write_message(EconomyGoldUpdate {
        player: player_a,
        gold: 8,
        current_mana: 3,
        reserve_mana: 0,
        mana_cap: 10,
    });
    app.world_mut().write_message(EconomyGoldBroadcast {
        player_id: player_a,
        gold: 8,
        reserved_gold: 0,
    });
    app.update();

    let outbox = app.world().resource::<EconomyNetworkOutbox>();
    assert!(outbox.gold_updates().is_empty());
    assert_eq!(outbox.gold_broadcasts().len(), 1);
    assert_eq!(outbox.gold_broadcasts()[0].player_id, player_a);
}

#[test]
fn protocol_manifest_registers_gold_messages_as_reliable_s2c() {
    #[derive(Default)]
    struct RecordingRegistry {
        messages: Vec<(String, ProtocolDirection, ProtocolChannel)>,
    }

    impl ProtocolRegistry for RecordingRegistry {
        fn add_channel<C: Send + Sync + 'static>(&mut self, _channel: ProtocolChannel) {}

        fn add_message<
            M: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
        >(
            &mut self,
            direction: ProtocolDirection,
            channel: ProtocolChannel,
        ) {
            self.messages
                .push((std::any::type_name::<M>().to_string(), direction, channel));
        }
    }

    let mut registry = RecordingRegistry::default();
    protocol::register_protocol(&mut registry);

    for message_name in [
        std::any::type_name::<ProtocolGoldUpdate>(),
        std::any::type_name::<ProtocolGoldBroadcast>(),
    ] {
        let matches = registry
            .messages
            .iter()
            .filter(|(name, _, _)| name == message_name)
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1);
        assert_eq!(
            matches[0],
            &(
                message_name.to_string(),
                ProtocolDirection::ServerToClient,
                ProtocolChannel::Reliable,
            )
        );
    }
}
