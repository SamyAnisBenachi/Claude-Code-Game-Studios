use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::ServerPlugins;
use lightyear::prelude::PeerId;
use server::core::rsm::DraftStarted;
use server::core::session::{
    DeferredMessage, PlayerConnectionMap, ReconnectTracker, SessionConfig, TeamId,
};
use server::feature::objective::{
    HiddenObjectives, ObjectiveIdentitiesReady, ObjectiveNetworkOutbox, ObjectivePlugin,
    OBJECTIVE_LANE_COUNT,
};
use server::foundation::config::GameConfig;
use server::foundation::rng::ServerRng;
use shared::card::ClassId;
use shared::protocol::{
    self, DraftPhase, GameMode, ProtocolChannel, ProtocolDirection, ProtocolRegistry,
    S2CObjectiveIdentities,
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

fn base_app() -> App {
    let mut app = App::new();
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / 60.0),
    })
    .add_message::<DraftStarted>()
    .add_plugins(ObjectivePlugin);
    app
}

fn hidden_objectives(player_a: PlayerId, player_b: PlayerId) -> HiddenObjectives {
    let mut hidden = HiddenObjectives::default();
    for lane in 1..=OBJECTIVE_LANE_COUNT {
        hidden.identities.insert((player_a, lane), false);
        hidden
            .identities
            .insert((player_b, lane), lane == 2 || lane == 4);
    }
    hidden
}

#[test]
fn draft_initial_sends_one_owner_unicast_per_player_in_order() {
    let player_a = player(1);
    let player_b = player(2);
    let peer_a = PeerId::Netcode(10);
    let peer_b = PeerId::Netcode(20);
    let mut app = base_app();
    app.insert_resource(session_config(player_b, player_a));
    app.insert_resource(GameConfig(shared::config::GameConfig::default()));
    app.insert_resource(ServerRng::from_seed(7));
    app.insert_resource(PlayerConnectionMap(HashMap::from([
        (peer_b, player_b),
        (peer_a, player_a),
    ])));
    app.insert_resource(ReconnectTracker {
        snapshot_sent: HashMap::from([(player_a, true), (player_b, true)]),
        ..Default::default()
    });

    app.world_mut().write_message(DraftStarted {
        round: 1,
        phase: DraftPhase::Initial,
    });
    app.update();

    let outbox = app.world().resource::<ObjectiveNetworkOutbox>();
    let dispatches = outbox.identity_dispatches();
    assert_eq!(dispatches.len(), 2);
    assert_eq!(dispatches[0].player_id, player_a);
    assert_eq!(dispatches[0].peer_id, Some(peer_a));
    assert_eq!(dispatches[1].player_id, player_b);
    assert_eq!(dispatches[1].peer_id, Some(peer_b));
    assert!(dispatches
        .iter()
        .all(|dispatch| dispatch.message.identities.len() == usize::from(OBJECTIVE_LANE_COUNT)));
}

#[test]
fn identity_payload_contains_only_recipient_lanes() {
    let player_a = player(1);
    let player_b = player(2);
    let mut app = base_app();
    app.insert_resource(hidden_objectives(player_a, player_b));
    app.insert_resource(PlayerConnectionMap(HashMap::from([(
        PeerId::Netcode(10),
        player_a,
    )])));
    app.insert_resource(ReconnectTracker {
        snapshot_sent: HashMap::from([(player_a, true)]),
        ..Default::default()
    });

    app.world_mut().write_message(ObjectiveIdentitiesReady {
        players: vec![player_a],
    });
    app.update();

    let outbox = app.world().resource::<ObjectiveNetworkOutbox>();
    let dispatch = outbox
        .identity_dispatches()
        .first()
        .expect("identity dispatch should be recorded");
    assert_eq!(dispatch.player_id, player_a);
    assert_eq!(
        dispatch.message.identities,
        vec![(1, false), (2, false), (3, false), (4, false), (5, false)]
    );
}

#[test]
fn snapshot_pending_defers_identity_unicast() {
    let player_a = player(1);
    let player_b = player(2);
    let mut app = base_app();
    app.insert_resource(hidden_objectives(player_a, player_b));
    app.insert_resource(PlayerConnectionMap(HashMap::from([(
        PeerId::Netcode(10),
        player_a,
    )])));
    app.insert_resource(ReconnectTracker {
        snapshot_sent: HashMap::from([(player_a, false)]),
        ..Default::default()
    });

    app.world_mut().write_message(ObjectiveIdentitiesReady {
        players: vec![player_a],
    });
    app.update();

    let tracker = app.world().resource::<ReconnectTracker>();
    let queued = tracker
        .deferred_queue
        .get(&player_a)
        .expect("pending snapshot player should have a deferred queue");
    let Some(DeferredMessage::ObjectiveIdentities(message)) = queued.first() else {
        panic!("expected deferred S2CObjectiveIdentities");
    };
    assert_eq!(
        message.identities,
        vec![(1, false), (2, false), (3, false), (4, false), (5, false)]
    );
}

#[test]
fn protocol_manifest_registers_objective_identities_as_reliable_s2c() {
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

    let objective_identity_messages = registry
        .messages
        .iter()
        .filter(|(name, _, _)| name == std::any::type_name::<S2CObjectiveIdentities>())
        .collect::<Vec<_>>();
    assert_eq!(objective_identity_messages.len(), 1);
    assert_eq!(
        objective_identity_messages[0],
        &(
            std::any::type_name::<S2CObjectiveIdentities>().to_string(),
            ProtocolDirection::ServerToClient,
            ProtocolChannel::Reliable
        )
    );
    assert!(!registry
        .messages
        .iter()
        .any(|(name, _, _)| name.ends_with("::ObjectiveIdentity")));
}
