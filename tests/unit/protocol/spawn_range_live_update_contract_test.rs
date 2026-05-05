use shared::card::{CardId, ClassId};
use shared::protocol::{
    BoardSnapshot, ObjectiveSnapshot, OpponentObjectiveSnapshot, PlayerSnapshot, ProtocolChannel,
    ProtocolDirection, ProtocolRegistry, ResolutionEvent, RoundPhase, S2CGameSnapshot,
    S2CResolutionEvent, TaggedEvent,
};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

#[derive(Default)]
struct RecordingRegistry {
    messages: Vec<(String, ProtocolDirection, ProtocolChannel)>,
}

impl ProtocolRegistry for RecordingRegistry {
    fn add_channel<C: Send + Sync + 'static>(&mut self, _channel: ProtocolChannel) {}

    fn add_message<M: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static>(
        &mut self,
        direction: ProtocolDirection,
        channel: ProtocolChannel,
    ) {
        self.messages
            .push((std::any::type_name::<M>().to_string(), direction, channel));
    }
}

#[test]
fn test_spawn_range_changed_schema_round_trips_through_resolution_batch() {
    let batch = S2CResolutionEvent {
        round: 4,
        events: vec![TaggedEvent {
            sub_step: 6,
            trigger_index: 12,
            event: ResolutionEvent::SpawnRangeChanged {
                player_id: player(1),
                new_spawn_range_cells: 2,
            },
        }],
    };

    let encoded = serde_json::to_string(&batch).expect("resolution batch should serialize");
    assert!(encoded.contains("SpawnRangeChanged"));
    assert!(encoded.contains("new_spawn_range_cells"));

    let decoded: S2CResolutionEvent =
        serde_json::from_str(&encoded).expect("resolution batch should deserialize");
    assert_eq!(decoded, batch);
    assert!(matches!(
        decoded.events[0].event,
        ResolutionEvent::SpawnRangeChanged {
            player_id,
            new_spawn_range_cells: 2,
        } if player_id == player(1)
    ));
}

#[test]
fn test_spawn_range_changed_is_ordered_after_fake_objective_destroyed_in_same_batch() {
    let attacker = player(1);
    let defender = player(2);
    let batch = S2CResolutionEvent {
        round: 5,
        events: vec![
            TaggedEvent {
                sub_step: 6,
                trigger_index: 20,
                event: ResolutionEvent::ObjectiveDestroyed {
                    target_player_id: defender,
                    lane: 3,
                    was_fake: true,
                },
            },
            TaggedEvent {
                sub_step: 6,
                trigger_index: 21,
                event: ResolutionEvent::SpawnRangeChanged {
                    player_id: attacker,
                    new_spawn_range_cells: 2,
                },
            },
        ],
    };

    let objective_destroyed_index = batch
        .events
        .iter()
        .position(|event| {
            matches!(
                event.event,
                ResolutionEvent::ObjectiveDestroyed {
                    target_player_id,
                    lane: 3,
                    was_fake: true,
                } if target_player_id == defender
            )
        })
        .expect("fake objective destruction should be present");
    let spawn_range_changed_index = batch
        .events
        .iter()
        .position(|event| {
            matches!(
                event.event,
                ResolutionEvent::SpawnRangeChanged {
                    player_id,
                    new_spawn_range_cells: 2,
                } if player_id == attacker
            )
        })
        .expect("spawn range update should be present");

    assert!(objective_destroyed_index < spawn_range_changed_index);
    assert!(batch.events[0].trigger_index < batch.events[1].trigger_index);
}

#[test]
fn test_s2c_resolution_event_remains_registered_on_reliable_channel() {
    let mut registry = RecordingRegistry::default();
    shared::protocol::register_protocol(&mut registry);

    let message_name = std::any::type_name::<S2CResolutionEvent>();
    let registrations = registry
        .messages
        .iter()
        .filter(|(registered, _, _)| registered == message_name)
        .collect::<Vec<_>>();

    assert_eq!(registrations.len(), 1);
    assert_eq!(
        registrations[0],
        &(
            message_name.to_string(),
            ProtocolDirection::ServerToClient,
            ProtocolChannel::Reliable,
        )
    );
}

#[test]
fn test_player_snapshot_spawn_range_cells_remains_public_recovery_field() {
    let snapshot = PlayerSnapshot {
        player_id: player(1),
        class_id: ClassId::Iop,
        gold: 7,
        reserved_gold: 0,
        current_mana: 4,
        reserve_mana: 1,
        spawn_range_cells: 3,
        mana_cap: 10,
        submitted: false,
        hand: vec![CardId(101)],
        shop_slots: vec![Some(CardId(201)), None, Some(CardId(203))],
        pool_snapshot: vec![(CardId(301), 2)],
        objectives: vec![ObjectiveSnapshot {
            lane: 1,
            hp: 3,
            is_real: true,
            is_destroyed: false,
        }],
        opponent_objectives: vec![OpponentObjectiveSnapshot {
            lane: 1,
            hp: 3,
            is_destroyed: false,
            was_fake: None,
        }],
    };
    let game_snapshot = S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: snapshot.player_id,
        round_number: 2,
        phase: RoundPhase::Placement,
        timer_remaining_ms: Some(30_000),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
        players: vec![snapshot],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    };

    let encoded = serde_json::to_value(&game_snapshot).expect("snapshot should serialize");
    assert_eq!(encoded["players"][0]["spawn_range_cells"], 3);

    let decoded: S2CGameSnapshot =
        serde_json::from_value(encoded).expect("snapshot should deserialize");
    assert_eq!(decoded.players[0].spawn_range_cells, 3);
}

#[test]
fn test_spawn_range_is_not_registered_as_standalone_protocol_message() {
    let mut registry = RecordingRegistry::default();
    shared::protocol::register_protocol(&mut registry);

    assert!(registry.messages.iter().all(|(message_name, _, _)| {
        !message_name.ends_with("SpawnRange") && !message_name.ends_with("SpawnRangeChanged")
    }));
}
