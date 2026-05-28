# PROMPT 2049 — Result Outcome Projection P1 Repair

Source of truth: `origin/main@798b5f8f` (rebased from claimed `158d0efe` ancestor — current `main` HEAD at session start).

Bug target: **P1-006** — Result screen does not reliably project win/loss/draw outcome (per `production/qa/bugs/current-unplayable-bug-register-2026-05-28.md` and `reports/PROMPT-2028-player-flow-unplayable-bug-classification.md` §P1-BUG-002, citing PROMPT 1937 GAP-1 / GAP-2).

Scope was kept tight: no broadening into server teardown or P1-013 snapshot ordering.

## Root Cause

`sync_result_screen_ui_system` in `client/src/presentation/result_screen.rs` calls `result_screen_outcome_copy(view_state.cached_result.as_ref(), local_player_id)`. That function returns the `RESULT PENDING` fallback whenever `cached_result` is `None`, even if `cached_snapshot` clearly indicates GameOver phase with deterministic destroyed-real-objective counts. In practice:

- The authoritative `S2CGameOver` message can lag the GameOver-phase snapshot, so the screen stays on `RESULT PENDING` while the snapshot already shows two real objectives destroyed.
- Per P1-013 (out-of-scope here), the final snapshot itself can lose `session`. Even when the message would still arrive, the user has no determinable visible result during the gap.

The snapshot already carries everything needed: `PlayerSnapshot.objectives` (own, with `is_real` + `is_destroyed`) and `PlayerSnapshot.opponent_objectives` (with `is_destroyed` + `was_fake: Option<bool>`). The mapping was simply not wired into the projection pipeline.

## Fix

Added `result_screen_outcome_copy_with_snapshot(result, snapshot, local_player_id)` in `client/src/presentation/result_screen.rs`:

- Delegates to the existing `result_screen_outcome_copy` when `S2CGameOver` is present (authoritative message wins).
- Otherwise, when `snapshot.phase == RoundPhase::GameOver`, derives the outcome from:
  - `own_real_destroyed = local.objectives.filter(is_real && is_destroyed).count()`
  - `opp_real_destroyed = local.opponent_objectives.filter(is_destroyed && was_fake == Some(false)).count()`
- Headlines:
  - `local_lost && opp_lost` → DRAW
  - `local_lost` → DEFEAT
  - `opp_lost` → VICTORY
  - neither (or no local player in snapshot, or non-GameOver phase) → existing `RESULT PENDING` fallback.

`sync_result_screen_ui_system` switched from `result_screen_outcome_copy(...)` to `result_screen_outcome_copy_with_snapshot(...)`, so the hero panel, accent stripe, panel border, and title divider all re-tint deterministically the moment the GameOver-phase snapshot lands — no longer waiting on `S2CGameOver`.

`OpponentObjectiveSnapshot.was_fake == None` (no reveal yet) is **not** counted as a real loss — this keeps the projection conservative when the opponent's destroyed objectives have not been disclosed.

## Owned Files Changed

- `client/src/presentation/result_screen.rs` — added `result_screen_outcome_copy_with_snapshot`; updated `sync_result_screen_ui_system` to call it.
- `client/Cargo.toml` — registered the new `[[test]]` entry (necessary for the new focused test to run).
- `tests/integration/presentation/result_screen_snapshot_projection_test.rs` — new focused test, 7 cases.
- `reports/PROMPT-2049-result-outcome-projection-p1-repair.md` — this report.

Not touched (per owned-scope rules): server, protocol, hand/drag UI, asset wiring, autoplay tooling, disconnect tracking, `production/session-state/**`.

## Tests

New focused suite (`result_screen_snapshot_projection_test`) — 7 cases, all pass:

1. `test_outcome_projection_two_real_own_objectives_destroyed_yields_defeat`
2. `test_outcome_projection_two_real_opponent_objectives_revealed_yields_victory`
3. `test_outcome_projection_both_sides_lost_two_real_yields_draw`
4. `test_outcome_projection_unknown_opponent_identity_does_not_count_as_real`
5. `test_outcome_projection_fake_own_objectives_destroyed_does_not_yield_defeat`
6. `test_outcome_projection_non_gameover_phase_does_not_project_outcome`
7. `test_outcome_projection_authoritative_result_overrides_snapshot_inference`

Existing pattern preserved (`result_screen_mvp_test` — 11 cases, all pass) confirming no regression in the authoritative-message path, the two-step reveal, or the Return-to-Lobby handshake.

## Validation

- Path allowlist: only `client/src/presentation/result_screen.rs`, `client/Cargo.toml` (test registration), `tests/integration/presentation/result_screen_snapshot_projection_test.rs`, and `reports/PROMPT-2049-*.md`.
- `git diff --check -- <owned files>` → clean (exit 0).
- `cargo test -p client --test result_screen_snapshot_projection_test` → `ok. 7 passed`.
- `cargo test -p client --test result_screen_mvp_test` → `ok. 11 passed`.
- Bevy 0.18 API discipline: `liv-bevy-018` skill consulted; no Message/Event API changes — the new function is pure data projection over existing protocol types. `sync_result_screen_ui_system` continues to use the same query/resource signatures.

## What This Does Not Close

- P1-013 (final GameOver snapshot loses `session`) is intentionally untouched per scope guard. The new projection fallback degrades gracefully when the snapshot can still be observed; it does not repair the teardown ordering itself.
- The fallback uses `is_destroyed` + `was_fake` reveals from the snapshot. If the opponent's real objective is destroyed but `was_fake` is still `None` at the moment of GameOver-phase observation, the projection conservatively reads as `RESULT PENDING` rather than guessing — this is intentional (see test #4).

2049: RESULT-OUTCOME-PROJECTION-P1-REPAIR: SHIPPED
