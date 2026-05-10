use std::collections::{HashMap, HashSet};

use server::core::rsm::RoundPhase;
use server::core::session::{
    apply_result_acknowledgement, cleanup_ended_session_reconnect_state,
    resolve_result_acknowledgement, DeferredMessage, EndedSessionResultState, ReconnectTracker,
    ResultAcknowledgementOutcome, SessionId,
};
use shared::card::CardId;
use shared::protocol::{
    BoardSnapshot, CardSource, GameOverReason, PlacementTimerMultiplier, S2CGameOver,
    S2CGameSnapshot,
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

fn snapshot(recipient: PlayerId) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: recipient,
        round_number: 6,
        phase: shared::protocol::RoundPhase::GameOver,
        timer_remaining_ms: None,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players: Vec::new(),
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn ended_state() -> EndedSessionResultState {
    let player_a = player(1);
    let player_b = player(2);
    EndedSessionResultState {
        result: S2CGameOver {
            loser: Some(player_a),
            round: 6,
            reason: GameOverReason::ObjectivesDestroyed,
        },
        participants: HashSet::from([player_a, player_b]),
        acknowledged: HashSet::new(),
        final_snapshots: HashMap::from([
            (player_a, snapshot(player_a)),
            (player_b, snapshot(player_b)),
        ]),
        expires_at_ms: 10_000,
        session_ids: HashSet::from([session_id(10)]),
    }
}

#[test]
fn ack_drain_is_session_owned_not_network_log_only() {
    test_helpers::init_test_tracing();
    let network_source = include_str!("../../../server/src/network/mod.rs");
    let session_source = include_str!("../../../server/src/core/session/system.rs");
    let plugin_source = include_str!("../../../server/src/core/session/plugin.rs");

    assert!(
        !network_source.contains("C2SAcknowledgeResult"),
        "network module must not keep the logging-only C2SAcknowledgeResult drain"
    );
    assert!(
        session_source.contains("MessageReceiver<C2SAcknowledgeResult>"),
        "GSS session system must own the production C2SAcknowledgeResult receiver"
    );
    assert!(
        plugin_source.contains("handle_result_acknowledgements"),
        "GSS plugin must schedule the acknowledgement handler"
    );
}

#[test]
fn invalid_phase_unknown_sender_and_non_participant_ack_are_silent_discards() {
    test_helpers::init_test_tracing();
    let mut state = ended_state();

    assert_eq!(
        resolve_result_acknowledgement(
            Some(RoundPhase::Placement),
            Some(&mut state),
            Some(player(1))
        ),
        ResultAcknowledgementOutcome::Discarded
    );
    assert!(state.acknowledged.is_empty());

    assert_eq!(
        resolve_result_acknowledgement(Some(RoundPhase::GameOver), Some(&mut state), None),
        ResultAcknowledgementOutcome::Discarded
    );
    assert!(state.acknowledged.is_empty());

    assert_eq!(
        resolve_result_acknowledgement(
            Some(RoundPhase::GameOver),
            Some(&mut state),
            Some(player(99))
        ),
        ResultAcknowledgementOutcome::Discarded
    );
    assert!(state.acknowledged.is_empty());
}

#[test]
fn acknowledgement_marks_only_sender_and_duplicate_is_noop() {
    test_helpers::init_test_tracing();
    let mut state = ended_state();

    assert_eq!(
        apply_result_acknowledgement(Some(&mut state), player(1)),
        ResultAcknowledgementOutcome::Acknowledged
    );
    assert!(state.acknowledged.contains(&player(1)));
    assert!(!state.acknowledged.contains(&player(2)));

    assert_eq!(
        apply_result_acknowledgement(Some(&mut state), player(1)),
        ResultAcknowledgementOutcome::Duplicate
    );
    assert_eq!(state.acknowledged.len(), 1);
    assert_eq!(
        state.result.reason,
        GameOverReason::ObjectivesDestroyed,
        "ack must not mutate the authoritative result"
    );
}

#[test]
fn all_ack_cleanup_removes_result_session_tokens_and_deferred_queues() {
    test_helpers::init_test_tracing();
    let mut state = ended_state();
    assert_eq!(
        apply_result_acknowledgement(Some(&mut state), player(1)),
        ResultAcknowledgementOutcome::Acknowledged
    );
    assert_eq!(
        apply_result_acknowledgement(Some(&mut state), player(2)),
        ResultAcknowledgementOutcome::AllAcknowledged
    );

    let mut tracker = ReconnectTracker {
        snapshot_sent: HashMap::from([(player(1), true), (player(2), true), (player(9), true)]),
        deferred_queue: HashMap::from([
            (
                player(1),
                vec![DeferredMessage::CardAcquired {
                    card_id: CardId(10),
                    source: CardSource::ShopPurchase,
                }],
            ),
            (
                player(2),
                vec![DeferredMessage::CardAcquired {
                    card_id: CardId(20),
                    source: CardSource::ShopPurchase,
                }],
            ),
            (
                player(9),
                vec![DeferredMessage::CardAcquired {
                    card_id: CardId(90),
                    source: CardSource::ShopPurchase,
                }],
            ),
        ]),
        token_map: HashMap::from([
            ([1; 16], (session_id(10), player(1))),
            ([2; 16], (session_id(10), player(2))),
            ([9; 16], (session_id(90), player(9))),
        ]),
        sang_meprise_sent_to: HashSet::from([player(1), player(2), player(9)]),
        ..Default::default()
    };

    cleanup_ended_session_reconnect_state(Some(&mut tracker), &state);

    assert_eq!(tracker.token_map.len(), 1);
    assert!(tracker.token_map.contains_key(&[9; 16]));
    assert!(!tracker.deferred_queue.contains_key(&player(1)));
    assert!(!tracker.deferred_queue.contains_key(&player(2)));
    assert!(tracker.deferred_queue.contains_key(&player(9)));
    assert!(!tracker.snapshot_sent.contains_key(&player(1)));
    assert!(!tracker.snapshot_sent.contains_key(&player(2)));
    assert_eq!(tracker.snapshot_sent.get(&player(9)), Some(&true));
    assert!(!tracker.sang_meprise_sent_to.contains(&player(1)));
    assert!(!tracker.sang_meprise_sent_to.contains(&player(2)));
    assert!(tracker.sang_meprise_sent_to.contains(&player(9)));
}

#[test]
fn timeout_cleanup_uses_same_terminal_cleanup_path() {
    test_helpers::init_test_tracing();
    let state = ended_state();
    let mut tracker = ReconnectTracker {
        snapshot_sent: HashMap::from([(player(1), true), (player(2), true)]),
        deferred_queue: HashMap::from([
            (player(1), Vec::new()),
            (
                player(2),
                vec![DeferredMessage::CardAcquired {
                    card_id: CardId(20),
                    source: CardSource::ShopPurchase,
                }],
            ),
        ]),
        token_map: HashMap::from([
            ([1; 16], (session_id(10), player(1))),
            ([2; 16], (session_id(10), player(2))),
        ]),
        ..Default::default()
    };

    cleanup_ended_session_reconnect_state(Some(&mut tracker), &state);

    assert!(tracker.token_map.is_empty());
    assert!(tracker.deferred_queue.is_empty());
    assert!(tracker.snapshot_sent.is_empty());
}
