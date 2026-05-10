use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use lightyear::prelude::PeerId;
use server::core::session::{
    cleanup_ended_session_reconnect_state, EndedSessionResultState, PlayerConnectionMap,
    ReconnectDispatch, ReconnectTracker, SessionId,
};
use server::foundation::config::GameConfig;
use shared::protocol::{
    BoardSnapshot, C2SHello, GameOverReason, OpponentObjectiveSnapshot, PlacementTimerMultiplier,
    PlayerSnapshot, S2CGameOver, S2CGameSnapshot,
};
use shared::session::PlayerId;
use uuid::Uuid;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_id(value: u128) -> SessionId {
    SessionId(Uuid::from_u128(value))
}

fn game_over_snapshot(recipient: PlayerId, opponent: PlayerId) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: recipient,
        round_number: 7,
        phase: shared::protocol::RoundPhase::GameOver,
        timer_remaining_ms: None,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players: vec![
            PlayerSnapshot {
                player_id: recipient,
                class_id: shared::card::ClassId::Iop,
                gold: 12,
                reserved_gold: 0,
                current_mana: 0,
                reserve_mana: 3,
                spawn_range_cells: 2,
                mana_cap: 10,
                submitted: true,
                hand: Vec::new(),
                shop_slots: Vec::new(),
                pool_snapshot: Vec::new(),
                objectives: Vec::new(),
                opponent_objectives: vec![
                    OpponentObjectiveSnapshot {
                        lane: 1,
                        hp: 0,
                        is_destroyed: true,
                        was_fake: Some(false),
                    },
                    OpponentObjectiveSnapshot {
                        lane: 2,
                        hp: 5,
                        is_destroyed: false,
                        was_fake: None,
                    },
                ],
            },
            PlayerSnapshot {
                player_id: opponent,
                class_id: shared::card::ClassId::Cra,
                gold: 8,
                reserved_gold: 0,
                current_mana: 0,
                reserve_mana: 0,
                spawn_range_cells: 1,
                mana_cap: 10,
                submitted: true,
                hand: Vec::new(),
                shop_slots: Vec::new(),
                pool_snapshot: Vec::new(),
                objectives: Vec::new(),
                opponent_objectives: Vec::new(),
            },
        ],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn ended_state(
    player_a: PlayerId,
    player_b: PlayerId,
    session_id: SessionId,
) -> EndedSessionResultState {
    EndedSessionResultState {
        result: S2CGameOver {
            loser: Some(player_b),
            round: 7,
            reason: GameOverReason::ObjectivesDestroyed,
        },
        participants: HashSet::from([player_a, player_b]),
        acknowledged: HashSet::new(),
        final_snapshots: HashMap::from([
            (player_a, game_over_snapshot(player_a, player_b)),
            (player_b, game_over_snapshot(player_b, player_a)),
        ]),
        expires_at_ms: 10_000,
        session_ids: HashSet::from([session_id]),
    }
}

fn world_with_retained_result() -> (World, [u8; 16], PlayerId, PlayerId, SessionId) {
    let player_a = player(1);
    let player_b = player(2);
    let session_id = session_id(77);
    let token = [7; 16];
    let mut world = World::new();
    world.insert_resource(GameConfig(shared::config::GameConfig::default()));
    world.insert_resource(PlayerConnectionMap(HashMap::from([(
        PeerId::Netcode(12),
        player_b,
    )])));
    world.insert_resource(ReconnectTracker {
        snapshot_sent: HashMap::from([(player_a, true), (player_b, true)]),
        deferred_queue: HashMap::from([(player_a, Vec::new()), (player_b, Vec::new())]),
        token_map: HashMap::from([(token, (session_id, player_a))]),
        ..Default::default()
    });
    world.insert_resource(ended_state(player_a, player_b, session_id));
    (world, token, player_a, player_b, session_id)
}

#[test]
fn game_over_reconnect_resends_retained_snapshot_result_and_phase_before_deferred_flush() {
    test_helpers::init_test_tracing();
    let (mut world, token, player_a, _player_b, _session_id) = world_with_retained_result();
    let entity = world.spawn_empty().id();
    let new_peer = PeerId::Netcode(11);

    let result = server::core::session::process_reconnect_hello(
        &mut world,
        entity,
        new_peer,
        C2SHello {
            protocol_version: 1,
            session_token: Some(token),
        },
    );

    assert!(result.closes.is_empty());
    assert_eq!(result.dispatches.len(), 5);
    assert!(matches!(
        result.dispatches[0],
        ReconnectDispatch::Handshake { .. }
    ));
    assert!(matches!(
        result.dispatches[1],
        ReconnectDispatch::GameSnapshot { .. }
    ));
    assert!(matches!(
        result.dispatches[2],
        ReconnectDispatch::ObjectiveIdentities { .. }
    ));
    assert!(matches!(
        result.dispatches[3],
        ReconnectDispatch::GameOver { .. }
    ));
    assert!(matches!(
        result.dispatches[4],
        ReconnectDispatch::PhaseChanged { .. }
    ));

    let ReconnectDispatch::GameSnapshot { message, .. } = &result.dispatches[1] else {
        unreachable!("checked above");
    };
    assert_eq!(message.recipient_player_id, player_a);
    assert_eq!(message.phase, shared::protocol::RoundPhase::GameOver);
    assert_eq!(message.round_number, 7);
    assert_eq!(message.players[0].reserve_mana, 3);
    assert_eq!(
        message.players[0].opponent_objectives[0].was_fake,
        Some(false)
    );
    assert_eq!(message.players[0].opponent_objectives[1].was_fake, None);

    let snapshot_json = serde_json::to_value(message).expect("snapshot should serialize");
    assert!(snapshot_json.get("loser").is_none());
    assert!(snapshot_json.get("reason").is_none());

    let ReconnectDispatch::GameOver { message, .. } = &result.dispatches[3] else {
        unreachable!("checked above");
    };
    assert_eq!(message.loser, Some(player(2)));
    assert_eq!(message.round, 7);
    assert_eq!(message.reason, GameOverReason::ObjectivesDestroyed);

    let ReconnectDispatch::PhaseChanged { message, .. } = &result.dispatches[4] else {
        unreachable!("checked above");
    };
    assert_eq!(message.phase, shared::protocol::RoundPhase::GameOver);
    assert_eq!(message.round_number, 7);
    assert_eq!(message.timer_duration_ms, 0);

    assert_eq!(
        world
            .resource::<ReconnectTracker>()
            .snapshot_sent
            .get(&player_a),
        Some(&true)
    );
}

#[test]
fn reconnect_after_result_cleanup_uses_expired_session_rejection_path() {
    test_helpers::init_test_tracing();
    let (mut world, token, player_a, _player_b, _session_id) = world_with_retained_result();
    let ended = world.resource::<EndedSessionResultState>().clone();
    {
        let mut tracker = world.resource_mut::<ReconnectTracker>();
        cleanup_ended_session_reconnect_state(Some(&mut tracker), &ended);
    }
    world.remove_resource::<EndedSessionResultState>();

    let entity = world.spawn_empty().id();
    let result = server::core::session::process_reconnect_hello(
        &mut world,
        entity,
        PeerId::Netcode(99),
        C2SHello {
            protocol_version: 1,
            session_token: Some(token),
        },
    );

    assert_eq!(result.dispatches.len(), 1);
    assert!(matches!(
        result.dispatches[0],
        ReconnectDispatch::HandshakeRejected { .. }
    ));
    assert_eq!(result.closes.len(), 1);
    assert!(!world
        .resource::<ReconnectTracker>()
        .token_map
        .values()
        .any(|(_, player)| *player == player_a));
}
