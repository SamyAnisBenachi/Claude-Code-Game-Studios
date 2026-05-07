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

#[test]
fn duplicate_ack_before_all_ack_is_idempotent_and_then_terminal_cleanup_removes_retention() {
    let mut ended = ended_state();
    let mut tracker = reconnect_tracker();

    assert_eq!(
        apply_result_acknowledgement(Some(&mut ended), player(1)),
        ResultAcknowledgementOutcome::Acknowledged
    );
    assert_eq!(
        apply_result_acknowledgement(Some(&mut ended), player(1)),
        ResultAcknowledgementOutcome::Duplicate
    );
    assert_eq!(ended.acknowledged, HashSet::from([player(1)]));

    assert_eq!(
        apply_result_acknowledgement(Some(&mut ended), player(2)),
        ResultAcknowledgementOutcome::AllAcknowledged
    );

    cleanup_ended_session_reconnect_state(Some(&mut tracker), &ended);

    assert!(tracker.token_map.is_empty());
    assert!(tracker.deferred_queue.is_empty());
    assert!(tracker.snapshot_sent.is_empty());
    assert!(tracker.sang_meprise_sent_to.is_empty());
    assert_eq!(
        ended.result.reason,
        GameOverReason::ObjectivesDestroyed,
        "ack cleanup must not mutate the server-authored result"
    );
}

#[test]
fn timeout_cleanup_uses_the_same_retention_cleanup_without_requiring_acknowledgements() {
    let ended = ended_state();
    let mut tracker = reconnect_tracker();

    cleanup_ended_session_reconnect_state(Some(&mut tracker), &ended);

    assert!(ended.acknowledged.is_empty());
    assert!(tracker.token_map.is_empty());
    assert!(tracker.deferred_queue.is_empty());
    assert!(tracker.snapshot_sent.is_empty());
    assert!(tracker.sang_meprise_sent_to.is_empty());
}

#[test]
fn stale_acknowledgements_are_silent_discards_until_game_over_retention_exists() {
    let mut ended = ended_state();

    assert_eq!(
        resolve_result_acknowledgement(Some(RoundPhase::GameOver), None, Some(player(1))),
        ResultAcknowledgementOutcome::Discarded
    );
    assert_eq!(
        resolve_result_acknowledgement(
            Some(RoundPhase::Placement),
            Some(&mut ended),
            Some(player(1))
        ),
        ResultAcknowledgementOutcome::Discarded
    );
    assert_eq!(
        resolve_result_acknowledgement(
            Some(RoundPhase::GameOver),
            Some(&mut ended),
            Some(player(99))
        ),
        ResultAcknowledgementOutcome::Discarded
    );
    assert!(ended.acknowledged.is_empty());
}

fn ended_state() -> EndedSessionResultState {
    let player_a = player(1);
    let player_b = player(2);
    EndedSessionResultState {
        result: S2CGameOver {
            loser: Some(player_b),
            round: 9,
            reason: GameOverReason::ObjectivesDestroyed,
        },
        participants: HashSet::from([player_a, player_b]),
        acknowledged: HashSet::new(),
        final_snapshots: HashMap::from([
            (player_a, snapshot(player_a)),
            (player_b, snapshot(player_b)),
        ]),
        expires_at_ms: 10_000,
        session_ids: HashSet::from([session_id(9)]),
    }
}

fn reconnect_tracker() -> ReconnectTracker {
    ReconnectTracker {
        snapshot_sent: HashMap::from([(player(1), true), (player(2), true)]),
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
        ]),
        token_map: HashMap::from([
            ([1; 16], (session_id(9), player(1))),
            ([2; 16], (session_id(9), player(2))),
        ]),
        sang_meprise_sent_to: HashSet::from([player(1), player(2)]),
        ..Default::default()
    }
}

fn snapshot(recipient: PlayerId) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: recipient,
        round_number: 9,
        phase: shared::protocol::RoundPhase::GameOver,
        timer_remaining_ms: None,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players: Vec::new(),
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn session_id(value: u128) -> SessionId {
    SessionId(Uuid::from_u128(value))
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
