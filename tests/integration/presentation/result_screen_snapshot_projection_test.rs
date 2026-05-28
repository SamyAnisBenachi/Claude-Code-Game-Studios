//! PROMPT-2049 / P1-006: when the authoritative `S2CGameOver` message is
//! not yet (or never) received, the result screen must still project a
//! deterministic VICTORY / DEFEAT / DRAW headline from the cached GameOver
//! snapshot's destroyed-real-objective counts. Stalling on
//! "RESULT PENDING" forever is the failure mode this guards against.

use client::presentation::result_screen::{
    result_screen_outcome_copy_with_snapshot, ResultScreenOutcomeCopy,
};
use shared::card::ClassId;
use shared::protocol::{
    BoardSnapshot, ObjectiveSnapshot, OpponentObjectiveSnapshot, PlacementTimerMultiplier,
    PlayerSnapshot, RoundPhase, S2CGameSnapshot,
};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn objective(lane: u8, hp: u8, is_real: bool, is_destroyed: bool) -> ObjectiveSnapshot {
    ObjectiveSnapshot {
        lane,
        hp,
        is_real,
        is_destroyed,
    }
}

fn opp_objective(
    lane: u8,
    hp: u8,
    is_destroyed: bool,
    was_fake: Option<bool>,
) -> OpponentObjectiveSnapshot {
    OpponentObjectiveSnapshot {
        lane,
        hp,
        is_destroyed,
        was_fake,
    }
}

fn player_snapshot(
    player_id: PlayerId,
    objectives: Vec<ObjectiveSnapshot>,
    opponent_objectives: Vec<OpponentObjectiveSnapshot>,
) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id: ClassId::Iop,
        gold: 0,
        reserved_gold: 0,
        current_mana: 0,
        reserve_mana: 0,
        spawn_range_cells: 1,
        mana_cap: 10,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives,
        opponent_objectives,
    }
}

fn snapshot_with_phase(phase: RoundPhase, local: PlayerSnapshot) -> S2CGameSnapshot {
    let recipient = local.player_id;
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: recipient,
        round_number: 3,
        phase,
        timer_remaining_ms: None,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players: vec![local],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

#[test]
fn test_outcome_projection_two_real_own_objectives_destroyed_yields_defeat() {
    // Arrange
    let local = player(1);
    let local_snapshot = player_snapshot(
        local,
        vec![
            objective(1, 0, true, true),
            objective(2, 0, true, true),
            objective(3, 5, true, false),
            objective(4, 4, false, false),
            objective(5, 7, false, false),
        ],
        vec![opp_objective(1, 6, false, None)],
    );
    let snapshot = snapshot_with_phase(RoundPhase::GameOver, local_snapshot);

    // Act
    let copy: ResultScreenOutcomeCopy =
        result_screen_outcome_copy_with_snapshot(None, Some(&snapshot), Some(local));

    // Assert
    assert_eq!(copy.headline, "DEFEAT");
    assert!(copy.has_result);
    assert!(copy.cause.contains("Two of your real objectives"));
}

#[test]
fn test_outcome_projection_two_real_opponent_objectives_revealed_yields_victory() {
    // Arrange
    let local = player(1);
    let local_snapshot = player_snapshot(
        local,
        vec![
            objective(1, 5, true, false),
            objective(2, 5, true, false),
        ],
        vec![
            opp_objective(1, 0, true, Some(false)),
            opp_objective(2, 0, true, Some(false)),
            opp_objective(3, 0, true, Some(true)),
        ],
    );
    let snapshot = snapshot_with_phase(RoundPhase::GameOver, local_snapshot);

    // Act
    let copy = result_screen_outcome_copy_with_snapshot(None, Some(&snapshot), Some(local));

    // Assert
    assert_eq!(copy.headline, "VICTORY");
    assert!(copy.has_result);
    assert!(copy.cause.contains("Opponent lost two real objectives"));
}

#[test]
fn test_outcome_projection_both_sides_lost_two_real_yields_draw() {
    // Arrange
    let local = player(1);
    let local_snapshot = player_snapshot(
        local,
        vec![
            objective(1, 0, true, true),
            objective(2, 0, true, true),
        ],
        vec![
            opp_objective(1, 0, true, Some(false)),
            opp_objective(2, 0, true, Some(false)),
        ],
    );
    let snapshot = snapshot_with_phase(RoundPhase::GameOver, local_snapshot);

    // Act
    let copy = result_screen_outcome_copy_with_snapshot(None, Some(&snapshot), Some(local));

    // Assert
    assert_eq!(copy.headline, "DRAW");
    assert!(copy.has_result);
}

#[test]
fn test_outcome_projection_unknown_opponent_identity_does_not_count_as_real() {
    // Arrange — opponent objectives destroyed but `was_fake` is None
    // (no reveal yet) so we must not count them as real losses.
    let local = player(1);
    let local_snapshot = player_snapshot(
        local,
        vec![objective(1, 5, true, false)],
        vec![
            opp_objective(1, 0, true, None),
            opp_objective(2, 0, true, None),
        ],
    );
    let snapshot = snapshot_with_phase(RoundPhase::GameOver, local_snapshot);

    // Act
    let copy = result_screen_outcome_copy_with_snapshot(None, Some(&snapshot), Some(local));

    // Assert — no deterministic outcome derivable, fall back to pending.
    assert_eq!(copy.headline, "RESULT PENDING");
    assert!(!copy.has_result);
}

#[test]
fn test_outcome_projection_fake_own_objectives_destroyed_does_not_yield_defeat() {
    // Arrange — only fake objectives destroyed; not a loss condition.
    let local = player(1);
    let local_snapshot = player_snapshot(
        local,
        vec![
            objective(1, 0, false, true),
            objective(2, 0, false, true),
            objective(3, 5, true, false),
        ],
        Vec::new(),
    );
    let snapshot = snapshot_with_phase(RoundPhase::GameOver, local_snapshot);

    // Act
    let copy = result_screen_outcome_copy_with_snapshot(None, Some(&snapshot), Some(local));

    // Assert
    assert_eq!(copy.headline, "RESULT PENDING");
    assert!(!copy.has_result);
}

#[test]
fn test_outcome_projection_non_gameover_phase_does_not_project_outcome() {
    // Arrange — snapshot says Resolution, not GameOver. Even with real
    // destruction we must not project a final outcome from a mid-game tick.
    let local = player(1);
    let local_snapshot = player_snapshot(
        local,
        vec![
            objective(1, 0, true, true),
            objective(2, 0, true, true),
        ],
        Vec::new(),
    );
    let snapshot = snapshot_with_phase(RoundPhase::Resolution, local_snapshot);

    // Act
    let copy = result_screen_outcome_copy_with_snapshot(None, Some(&snapshot), Some(local));

    // Assert
    assert_eq!(copy.headline, "RESULT PENDING");
    assert!(!copy.has_result);
}

#[test]
fn test_outcome_projection_authoritative_result_overrides_snapshot_inference() {
    use shared::protocol::{GameOverReason, S2CGameOver};

    // Arrange — snapshot would derive VICTORY, but S2CGameOver declares DEFEAT.
    let local = player(1);
    let local_snapshot = player_snapshot(
        local,
        vec![objective(1, 5, true, false)],
        vec![
            opp_objective(1, 0, true, Some(false)),
            opp_objective(2, 0, true, Some(false)),
        ],
    );
    let snapshot = snapshot_with_phase(RoundPhase::GameOver, local_snapshot);
    let result = S2CGameOver {
        loser: Some(local),
        round: 3,
        reason: GameOverReason::ObjectivesDestroyed,
    };

    // Act
    let copy =
        result_screen_outcome_copy_with_snapshot(Some(&result), Some(&snapshot), Some(local));

    // Assert — the authoritative message wins.
    assert_eq!(copy.headline, "DEFEAT");
}
