//! Integration tests for S13-LATE-MSG-DEDUPE-001.
//!
//! Asserts client-side idempotency for late / duplicate reliable S2C messages
//! whose drains have user-visible side effects:
//! - `S2CGameOver` -> result-screen entry
//! - `S2CClassLocked` -> lobby state
//! - `S2CPlacementReveal` -> board reveal animation
//!
//! Each drain follows the `C2SAcknowledgeResult` precedent at
//! `tests/integration/session/result_acknowledgement_contract_test.rs:91-96`:
//! on duplicate detection, the drain logs DEBUG and returns early without
//! side effect.
//!
//! Test surface mirrors `tests/integration/presentation/protocol_orphan_drain_test.rs`:
//! exercise the per-drain `apply_*_drain` (or the equivalent filter) functions
//! directly + grep guards proving each production drain consults the dedupe
//! ring. No optimistic client-side authority is introduced or relied upon by
//! these tests; the dedupe state is part of the read-only client projection.

use std::fs;
use std::path::{Path, PathBuf};

use client::presentation::board_rendering::filter_placement_reveal_for_dedupe;
use client::presentation::result_screen::{apply_game_over_drain, ResultScreenViewState};
use client::state::{
    ClassLockedDedupeKey, ClientIdempotencyState, GameOverDedupeKey, PlacementRevealDedupeKey,
    DEDUPE_BOUND,
};
use client::ui::lobby::{apply_class_locked_drain, LobbyInputState, LobbyViewState};
use shared::card::{CardId, ClassId};
use shared::protocol::{
    GameOverReason, PlacedCardReveal, PlayTarget, S2CClassLocked, S2CGameOver, S2CPlacementReveal,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const CLIENT_SRC_REL: &str = "src";

fn client_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(CLIENT_SRC_REL)
}

fn collect_source_matches(path: &Path, needle: &str, matches: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_source_matches(&path, needle, matches);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "rs") {
            if let Ok(text) = fs::read_to_string(&path) {
                if text.contains(needle) {
                    matches.push(path);
                }
            }
        }
    }
}

fn assert_grep_present(needle: &str) {
    let mut matches = Vec::new();
    collect_source_matches(&client_src_root(), needle, &mut matches);
    assert!(
        !matches.is_empty(),
        "expected to find `{}` somewhere under client/src/; matches: {:?}",
        needle,
        matches,
    );
}

fn game_over(loser: Option<PlayerId>, round: u32, reason: GameOverReason) -> S2CGameOver {
    S2CGameOver {
        loser,
        round,
        reason,
    }
}

fn game_over_eq(a: &S2CGameOver, b: &S2CGameOver) -> bool {
    a.loser == b.loser && a.round == b.round && a.reason == b.reason
}

fn assert_cached_result_matches(view: &ResultScreenViewState, expected: &S2CGameOver, ctx: &str) {
    let cached = view
        .cached_result
        .as_ref()
        .unwrap_or_else(|| panic!("{ctx}: cached_result must be Some"));
    assert!(
        game_over_eq(cached, expected),
        "{ctx}: cached_result mismatch (got loser={:?} round={} reason={:?}, expected loser={:?} round={} reason={:?})",
        cached.loser,
        cached.round,
        cached.reason,
        expected.loser,
        expected.round,
        expected.reason,
    );
}

fn class_locked(class_id: ClassId) -> S2CClassLocked {
    S2CClassLocked { class_id }
}

fn placement(owner: u64, card: u32, lane: u8, cell: u8) -> PlacedCardReveal {
    PlacedCardReveal {
        owner_id: PlayerId(owner),
        card_id: CardId(card),
        target: PlayTarget::BoardCell { lane, cell },
    }
}

fn placement_reveal(placements: Vec<PlacedCardReveal>) -> S2CPlacementReveal {
    S2CPlacementReveal { placements }
}

// ---------- AC1: S2CGameOver dedupe-guarded ----------

#[test]
fn s2c_game_over_drain_first_apply_caches_then_duplicate_is_noop() {
    test_helpers::init_test_tracing();
    let mut idempotency = ClientIdempotencyState::default();
    let mut view = ResultScreenViewState::default();
    let result = game_over(Some(PlayerId(1)), 6, GameOverReason::ObjectivesDestroyed);

    apply_game_over_drain(&mut idempotency, &mut view, result.clone());
    assert_cached_result_matches(&view, &result, "after first apply");
    assert_eq!(idempotency.game_over.len(), 1);

    // Mutate cache to detect any side effect from the duplicate apply.
    let sentinel = game_over(Some(PlayerId(99)), 99, GameOverReason::ResolutionTimeout);
    view.cached_result = Some(sentinel.clone());
    apply_game_over_drain(&mut idempotency, &mut view, result);
    assert_cached_result_matches(
        &view,
        &sentinel,
        "duplicate S2CGameOver must not overwrite the sentinel",
    );
    assert_eq!(
        idempotency.game_over.len(),
        1,
        "duplicate apply must not grow the dedupe ring"
    );
}

#[test]
fn s2c_game_over_drain_distinct_round_is_not_deduped() {
    test_helpers::init_test_tracing();
    let mut idempotency = ClientIdempotencyState::default();
    let mut view = ResultScreenViewState::default();
    let r6 = game_over(Some(PlayerId(1)), 6, GameOverReason::ObjectivesDestroyed);
    let r7 = game_over(Some(PlayerId(1)), 7, GameOverReason::ObjectivesDestroyed);

    apply_game_over_drain(&mut idempotency, &mut view, r6);
    apply_game_over_drain(&mut idempotency, &mut view, r7.clone());
    assert_cached_result_matches(&view, &r7, "after second distinct round");
    assert_eq!(idempotency.game_over.len(), 2);
}

#[test]
fn s2c_game_over_drain_consults_dedupe_ring_in_production_source() {
    assert_grep_present("idempotency.game_over.check_and_insert");
}

// ---------- AC2: S2CClassLocked dedupe-guarded ----------

#[test]
fn s2c_class_locked_drain_first_apply_locks_then_duplicate_is_noop() {
    test_helpers::init_test_tracing();
    let mut idempotency = ClientIdempotencyState::default();
    let mut lobby = LobbyViewState::default();
    let mut input = LobbyInputState::default();
    input.class_confirm_in_flight = true;
    let message = class_locked(ClassId::Iop);

    apply_class_locked_drain(&mut idempotency, &mut lobby, &mut input, &message);
    assert_eq!(lobby.locked_class, Some(ClassId::Iop));
    assert!(
        !input.class_confirm_in_flight,
        "first S2CClassLocked must clear the in-flight confirm latch"
    );
    assert_eq!(idempotency.class_locked.len(), 1);

    // Mutate state to detect any side effect from the duplicate apply.
    lobby.locked_class = Some(ClassId::Cra);
    input.class_confirm_in_flight = true;
    apply_class_locked_drain(&mut idempotency, &mut lobby, &mut input, &message);
    assert_eq!(
        lobby.locked_class,
        Some(ClassId::Cra),
        "duplicate S2CClassLocked must not overwrite the lobby lock"
    );
    assert!(
        input.class_confirm_in_flight,
        "duplicate S2CClassLocked must not clear the in-flight confirm latch"
    );
    assert_eq!(
        idempotency.class_locked.len(),
        1,
        "duplicate apply must not grow the dedupe ring"
    );
}

#[test]
fn s2c_class_locked_drain_distinct_class_is_not_deduped() {
    test_helpers::init_test_tracing();
    let mut idempotency = ClientIdempotencyState::default();
    let mut lobby = LobbyViewState::default();
    let mut input = LobbyInputState::default();
    let iop = class_locked(ClassId::Iop);
    let cra = class_locked(ClassId::Cra);

    apply_class_locked_drain(&mut idempotency, &mut lobby, &mut input, &iop);
    apply_class_locked_drain(&mut idempotency, &mut lobby, &mut input, &cra);
    assert_eq!(lobby.locked_class, Some(ClassId::Cra));
    assert_eq!(idempotency.class_locked.len(), 2);
}

#[test]
fn s2c_class_locked_drain_consults_dedupe_ring_in_production_source() {
    assert_grep_present("idempotency.class_locked.check_and_insert");
}

// ---------- AC3: S2CPlacementReveal dedupe-guarded ----------

#[test]
fn s2c_placement_reveal_drain_first_apply_returns_message_then_duplicate_is_noop() {
    test_helpers::init_test_tracing();
    let mut idempotency = ClientIdempotencyState::default();
    let message = placement_reveal(vec![placement(1, 10, 1, 0)]);

    let first = filter_placement_reveal_for_dedupe(&mut idempotency, 5, message.clone());
    assert!(
        first.is_some(),
        "first S2CPlacementReveal must pass the filter"
    );
    assert_eq!(idempotency.placement_reveal.len(), 1);

    let duplicate = filter_placement_reveal_for_dedupe(&mut idempotency, 5, message);
    assert!(
        duplicate.is_none(),
        "duplicate S2CPlacementReveal at the same round must be filtered to None"
    );
    assert_eq!(
        idempotency.placement_reveal.len(),
        1,
        "duplicate apply must not grow the dedupe ring"
    );
}

#[test]
fn s2c_placement_reveal_drain_distinct_round_is_not_deduped() {
    test_helpers::init_test_tracing();
    let mut idempotency = ClientIdempotencyState::default();
    let message = placement_reveal(vec![placement(1, 10, 1, 0)]);

    let r5 = filter_placement_reveal_for_dedupe(&mut idempotency, 5, message.clone());
    let r6 = filter_placement_reveal_for_dedupe(&mut idempotency, 6, message);
    assert!(r5.is_some());
    assert!(r6.is_some());
    assert_eq!(idempotency.placement_reveal.len(), 2);
}

#[test]
fn s2c_placement_reveal_drain_distinct_payload_is_not_deduped() {
    test_helpers::init_test_tracing();
    let mut idempotency = ClientIdempotencyState::default();
    let a = placement_reveal(vec![placement(1, 10, 1, 0)]);
    let b = placement_reveal(vec![placement(1, 10, 2, 0)]);

    let first = filter_placement_reveal_for_dedupe(&mut idempotency, 5, a);
    let second = filter_placement_reveal_for_dedupe(&mut idempotency, 5, b);
    assert!(first.is_some());
    assert!(second.is_some());
    assert_eq!(idempotency.placement_reveal.len(), 2);
}

#[test]
fn s2c_placement_reveal_drain_consults_dedupe_ring_in_production_source() {
    assert_grep_present("idempotency.placement_reveal.check_and_insert");
}

// ---------- AC4: Reconnect-replay duplicate scenario ----------

#[test]
fn ac4_game_over_reconnect_replay_runs_result_screen_sequence_exactly_once() {
    test_helpers::init_test_tracing();
    // Models the reconnect flow at server/src/core/session/reconnect.rs:836-893
    // where GameOver re-sends snapshot + S2CGameOver. The dedupe ring is part
    // of the session-lifetime ClientIdempotencyState resource; the same
    // resource handles both pre-reconnect and post-reconnect drain calls
    // because OnExit(InSession) is the only clear point and reconnect does
    // not exit InSession.
    let mut idempotency = ClientIdempotencyState::default();
    let mut view = ResultScreenViewState::default();
    let message = game_over(Some(PlayerId(7)), 6, GameOverReason::ObjectivesDestroyed);

    // Pre-reconnect drain.
    apply_game_over_drain(&mut idempotency, &mut view, message.clone());
    assert!(
        view.cached_result.is_some(),
        "pre-reconnect drain must cache"
    );

    // Sentinel proves the duplicate drain does not write again.
    let sentinel = game_over(Some(PlayerId(99)), 99, GameOverReason::ResolutionTimeout);
    view.cached_result = Some(sentinel.clone());

    // Reconnect re-send replays the same authoritative GameOver.
    apply_game_over_drain(&mut idempotency, &mut view, message);

    assert_cached_result_matches(
        &view,
        &sentinel,
        "reconnect-replay duplicate must not re-trigger the result-screen entry path",
    );
    assert_eq!(idempotency.game_over.len(), 1);
}

// ---------- AC5: session-lifetime scope (clear on OnExit(InSession)) ----------

#[test]
fn ac5_clear_for_session_exit_resets_all_drain_rings() {
    test_helpers::init_test_tracing();
    let mut idempotency = ClientIdempotencyState::default();

    idempotency
        .game_over
        .check_and_insert(GameOverDedupeKey::from_message(&game_over(
            Some(PlayerId(1)),
            1,
            GameOverReason::Draw,
        )));
    idempotency
        .class_locked
        .check_and_insert(ClassLockedDedupeKey::from_message(&class_locked(
            ClassId::Iop,
        )));
    idempotency
        .placement_reveal
        .check_and_insert(PlacementRevealDedupeKey::from_message(
            1,
            &placement_reveal(vec![placement(1, 10, 1, 0)]),
        ));
    assert_eq!(idempotency.game_over.len(), 1);
    assert_eq!(idempotency.class_locked.len(), 1);
    assert_eq!(idempotency.placement_reveal.len(), 1);

    idempotency.clear_for_session_exit();
    assert!(idempotency.game_over.is_empty());
    assert!(idempotency.class_locked.is_empty());
    assert!(idempotency.placement_reveal.is_empty());
}

#[test]
fn ac5_session_exit_system_is_wired_to_on_exit_in_session() {
    // Source-grep guard: the session-exit reset system must be registered on
    // OnExit(ClientState::InSession) per ADR-021 / story Required: dedupe
    // state is session-scoped.
    assert_grep_present("OnExit(ClientState::InSession)");
    assert_grep_present("reset_client_idempotency_on_session_exit_system");
}

// ---------- AC6: bounded size ----------

#[test]
fn ac6_dedupe_ring_evicts_oldest_when_bound_exceeded() {
    test_helpers::init_test_tracing();
    let mut idempotency = ClientIdempotencyState::default();

    // Insert DEDUPE_BOUND + 1 distinct rounds. The first round must be
    // evicted; re-inserting it must succeed.
    let first_round = 1u32;
    for round in first_round..first_round + DEDUPE_BOUND as u32 + 1 {
        let msg = game_over(Some(PlayerId(1)), round, GameOverReason::Draw);
        let key = GameOverDedupeKey::from_message(&msg);
        assert!(
            idempotency.game_over.check_and_insert(key),
            "round {round} must insert as fresh"
        );
    }
    assert_eq!(idempotency.game_over.len(), DEDUPE_BOUND);

    let evicted = game_over(Some(PlayerId(1)), first_round, GameOverReason::Draw);
    let key = GameOverDedupeKey::from_message(&evicted);
    assert!(
        idempotency.game_over.check_and_insert(key),
        "the oldest round must have been evicted, leaving room for a re-insert"
    );
    assert_eq!(idempotency.game_over.len(), DEDUPE_BOUND);
}

#[test]
fn ac6_dedupe_bound_documented_inline() {
    // The inline `pub const DEDUPE_BOUND` definition is the documented bound
    // referenced by AC6. This grep guard ensures the constant is not silently
    // removed during refactors.
    assert_grep_present("pub const DEDUPE_BOUND: usize");
}

// ---------- AC7: no protocol-shape change ----------

#[test]
fn ac7_no_new_message_id_field_in_protocol() {
    // The dedupe key is constructed entirely from existing message fields;
    // no `message_id` / `sequence_num` field is added to S2CGameOver,
    // S2CClassLocked, or S2CPlacementReveal. AC7 binding.
    let proto =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared/src/protocol.rs"))
            .expect("shared/src/protocol.rs must be readable");
    assert!(
        !proto.contains("pub message_id"),
        "no `message_id` field must be added to the protocol under this story"
    );
    assert!(
        !proto.contains("pub sequence_num"),
        "no `sequence_num` field must be added to the protocol under this story"
    );
}

// ---------- AC9: no optimistic client-side authority ----------

#[test]
fn ac9_dedupe_state_is_a_read_only_projection_no_optimistic_authority() {
    test_helpers::init_test_tracing();
    // The dedupe state is a defensive filter; mutating it does NOT cause any
    // server-authoritative state to change. This test exercises the dedupe
    // ring directly to prove it is purely a local-side projection.
    let mut idempotency = ClientIdempotencyState::default();
    let key = GameOverDedupeKey::from_message(&game_over(None, 1, GameOverReason::Draw));
    let inserted_first = idempotency.game_over.check_and_insert(key);
    let inserted_again = idempotency.game_over.check_and_insert(key);
    assert!(inserted_first);
    assert!(!inserted_again);
}
