# Sprint 13 -- Late Message Dedupe Evidence (Story 020)

> **Story**: `S13-LATE-MSG-DEDUPE-001`
> **Epic**: Playable Client (`production/epics/playable-client/story-020-late-msg-dedupe.md`)
> **Worker prompt**: PROMPT 872
> **Worker branch**: `work/s13-late-msg-dedupe`
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s13-late-msg-dedupe`
> **Source-of-truth at implementation**: `origin/main@3cf5e41`

## No-claim restatement

This story does not claim public release readiness, release-candidate readiness,
full game completion, broad / Standard-tier accessibility completion
(`QA-COND-0005`), playtest / fun-hypothesis validation (`QA-COND-0006`), full
playable-client manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`), or
final-art / asset-production completion.

**No optimistic client-side authority is introduced.** Dedupe state is
purely a defensive read-side filter on the client; the server remains the
sole authority over game state. The dedupe state is part of the read-only
client projection (ADR-002 / ADR-008 / ADR-011 binding).

## Cross-link to source finding

- PROMPT 803 §3 DC-6 (Reconnect / snapshot late-message idempotency, HIGH).
- PROMPT 803 §4 Lane C (DC-6 late-message idempotency).
- PROMPT 803 §5 Should row 1 (S13-LATE-MSG-DEDUPE-001).
- Precedent test: `tests/integration/session/result_acknowledgement_contract_test.rs:91-96`
  (`acknowledgement_marks_only_sender_and_duplicate_is_noop`).

## Dedupe key construction rationale

The reliable Lightyear channel guarantees byte-identity of replayed messages
(`server/src/core/session/reconnect.rs:198-233` reconnect flush replays the
same authoritative messages). Each per-drain key is therefore reduced from
existing message fields without modifying the shared protocol:

| Drain | Key shape | Source |
|-------|-----------|--------|
| `S2CGameOver` | `(round: u32, reason_index: u8, loser: Option<PlayerId>)` | message itself; one GameOver per round |
| `S2CClassLocked` | `ClassId` | message itself; one class lock per session |
| `S2CPlacementReveal` | `(round: u32, digest: u64)` | round = `CurrentClientPhase.round`; digest = `DefaultHasher` over placement vector |

`DefaultHasher` (`std::collections::hash_map::DefaultHasher`, `SipHasher13`)
is fixed-seed at construction so the digest is process-stable; cross-process
stability is not required because the dedupe ring is per-client and reset
on `OnExit(ClientState::InSession)`.

`S13-PROTO-MESSAGE-ID-001` (Sprint 14 candidate) would replace these
canonical-content keys with an explicit `sequence_num` field on the
protocol; this story intentionally avoids that protocol change (AC7).

## Per-message dedupe diff summary

| File | Change |
|------|--------|
| `client/src/state/idempotency.rs` (NEW) | `ClientIdempotencyState` resource, three `DedupeRing<K>` rings, three typed key builders, `ClientIdempotencyPlugin` (installs the resource and registers `OnExit(InSession)` clear), 7 unit tests. |
| `client/src/state/mod.rs` | `pub mod idempotency;` + re-exports. |
| `client/src/presentation/mod.rs` | Install `ClientIdempotencyPlugin` ahead of all drain plugins. |
| `client/src/presentation/result_screen.rs` | `drain_result_screen_game_over_receiver_system` now consults `ClientIdempotencyState::game_over` via `apply_game_over_drain`; duplicates log DEBUG and return without mutating `ResultScreenViewState`. Plugin also `init_resource::<ClientIdempotencyState>()` so isolated test apps still build. |
| `client/src/ui/lobby.rs` | `drain_lobby_s2c_system` for `S2CClassLocked` now calls `apply_class_locked_drain`, which consults `ClientIdempotencyState::class_locked`; duplicates log DEBUG and return without mutating `LobbyViewState` or clearing `class_confirm_in_flight`. Plugin also `init_resource::<ClientIdempotencyState>()`. |
| `client/src/presentation/board_rendering.rs` | `drain_placement_reveal_system` now applies `filter_placement_reveal_for_dedupe`, which consults `ClientIdempotencyState::placement_reveal` keyed by `(CurrentClientPhase.round, content-digest)`; duplicates log DEBUG and are dropped before the reveal pipeline runs. Plugin also `init_resource::<ClientIdempotencyState>()`. |
| `client/Cargo.toml` | Register the new `late_msg_dedupe_test` integration test entry. |
| `tests/integration/session/late_msg_dedupe_test.rs` (NEW) | 17 integration tests: 3x `apply_*_drain` first-then-duplicate, 2x distinct-key-not-deduped, 3x source-grep guards confirming each drain consults the dedupe ring, 1x reconnect-replay scenario, 2x session-exit clear lifecycle, 2x bound semantics, 1x AC7 protocol-shape source check, 1x AC9 read-only projection. |
| `production/qa/evidence/sprint-13-late-msg-dedupe-evidence.md` (NEW) | This file. |

## Acceptance criteria verification

| AC | Verdict | Evidence |
|----|---------|----------|
| AC1 | PASS | `apply_game_over_drain` in `client/src/presentation/result_screen.rs`; tests `s2c_game_over_drain_first_apply_caches_then_duplicate_is_noop`, `s2c_game_over_drain_consults_dedupe_ring_in_production_source`. |
| AC2 | PASS | `apply_class_locked_drain` in `client/src/ui/lobby.rs`; tests `s2c_class_locked_drain_first_apply_locks_then_duplicate_is_noop`, `s2c_class_locked_drain_consults_dedupe_ring_in_production_source`. |
| AC3 | PASS | `filter_placement_reveal_for_dedupe` in `client/src/presentation/board_rendering.rs`; tests `s2c_placement_reveal_drain_first_apply_returns_message_then_duplicate_is_noop`, `s2c_placement_reveal_drain_consults_dedupe_ring_in_production_source`. The `S2CPlaceUnit` reference in the story file is a planning-time naming for what the protocol calls `S2CPlacementReveal`; both refer to the placement-class S2C drain. |
| AC4 | PASS | `ac4_game_over_reconnect_replay_runs_result_screen_sequence_exactly_once` -- pre-reconnect drain caches, sentinel detects any duplicate side effect, post-reconnect drain leaves the sentinel untouched. |
| AC5 | PASS | `ClientIdempotencyPlugin::build` registers `reset_client_idempotency_on_session_exit_system` on `OnExit(ClientState::InSession)`; tests `ac5_clear_for_session_exit_resets_all_drain_rings`, `ac5_session_exit_system_is_wired_to_on_exit_in_session`. The reconnect path does not exit `InSession`, so dedupe state is preserved across reconnect (per ADR-011). |
| AC6 | PASS | `pub const DEDUPE_BOUND: usize = 32` documented inline in `client/src/state/idempotency.rs`; `DedupeRing<K>::check_and_insert` evicts the oldest key on overflow. Tests `ac6_dedupe_ring_evicts_oldest_when_bound_exceeded`, `ac6_dedupe_bound_documented_inline`, plus unit test `dedupe_ring_evicts_oldest_when_bound_exceeded`. |
| AC7 | PASS | `git diff origin/main...HEAD -- shared/src/protocol.rs` is empty (verified by AC7 test `ac7_no_new_message_id_field_in_protocol`). Dedupe keys are constructed from existing fields. |
| AC8 | PASS | `git diff origin/main...HEAD -- server/` is empty -- the change set is confined to client + tests + production paperwork. |
| AC9 | PASS | The dedupe state is a read-only projection on the client. Test `ac9_dedupe_state_is_a_read_only_projection_no_optimistic_authority`. Banner-level "no optimistic" restatement preserved verbatim above. |
| AC10 | PARTIAL (story-prescribed targeted check only per PROMPT 872) | Per worker prompt, full workspace tests are out of scope ("Run only story-prescribed targeted checks"). Targeted scope verified: `cargo check -p client` clean; `cargo fmt -p client -- --check` clean; `cargo test -p client --test late_msg_dedupe_test` 17/17 pass; nearby regression set (`result_screen_mvp_test`, `result_screen_return_to_lobby_test`, `presentation_protocol_orphan_drain_test`, `board_rendering_placement_reveal_test`, `playable_client_phase_changed_idempotency_test`) green; client lib unit tests for `state::idempotency` 7/7 pass. No new `#[ignore]` markers introduced. |
| AC11 | PASS | `git diff origin/main...HEAD -- production/sprint-status.yaml production/sprints/sprint-12.md production/stage.txt production/qa/qa-plan-sprint-12.md` is empty. |
| AC12 | PASS | This file. |

## Test commands and results

```text
$ cargo fmt -p client -- --check
(clean)

$ cargo check -p client
   Finished `dev` profile [optimized] target(s) in 5.70s

$ cargo test -p client --test late_msg_dedupe_test
running 17 tests
test ac7_no_new_message_id_field_in_protocol ... ok
test ac9_dedupe_state_is_a_read_only_projection_no_optimistic_authority ... ok
test ac5_clear_for_session_exit_resets_all_drain_rings ... ok
test ac6_dedupe_ring_evicts_oldest_when_bound_exceeded ... ok
test s2c_game_over_drain_first_apply_caches_then_duplicate_is_noop ... ok
test s2c_class_locked_drain_first_apply_locks_then_duplicate_is_noop ... ok
test s2c_class_locked_drain_distinct_class_is_not_deduped ... ok
test s2c_placement_reveal_drain_distinct_payload_is_not_deduped ... ok
test ac4_game_over_reconnect_replay_runs_result_screen_sequence_exactly_once ... ok
test s2c_game_over_drain_distinct_round_is_not_deduped ... ok
test s2c_placement_reveal_drain_distinct_round_is_not_deduped ... ok
test s2c_placement_reveal_drain_first_apply_returns_message_then_duplicate_is_noop ... ok
test s2c_placement_reveal_drain_consults_dedupe_ring_in_production_source ... ok
test ac6_dedupe_bound_documented_inline ... ok
test s2c_game_over_drain_consults_dedupe_ring_in_production_source ... ok
test s2c_class_locked_drain_consults_dedupe_ring_in_production_source ... ok
test ac5_session_exit_system_is_wired_to_on_exit_in_session ... ok
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test -p client --lib state::idempotency
running 7 tests
test state::idempotency::tests::dedupe_ring_clear_empties_state ... ok
test state::idempotency::tests::dedupe_ring_duplicate_returns_false_and_does_not_grow ... ok
test state::idempotency::tests::dedupe_ring_evicts_oldest_when_bound_exceeded ... ok
test state::idempotency::tests::dedupe_ring_inserts_unique_returns_true ... ok
test state::idempotency::tests::game_over_key_distinguishes_round_reason_loser ... ok
test state::idempotency::tests::placement_reveal_key_changes_with_round_or_payload ... ok
test state::idempotency::tests::placement_reveal_key_is_stable_for_identical_payload ... ok
test result: ok. 7 passed; 0 failed
```

## Cargo policy footprint

Worker honoured the Cargo policy in PROMPT 872:

```text
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

No stale target directory cleanup was required during this story.
