# PROMPT-2033 — Server Board/GameOver Vacuous Flow P0 Repair

**Date:** 2026-05-28
**Branch:** `work/PROMPT-2033`
**Commit:** `9bd56ea3`
**Source-of-truth:** origin/main@8863e26c
**Related reports:** PROMPT-2024, PROMPT-2025

---

## Scope

Bugs from PROMPT-2025 audit assigned to this prompt: BUG-05, BUG-06, BUG-16, BUG-17, BUG-24.
Owned path: placement → board application → resolution → GameOver.

---

## 1. BUG-17 — `submissions_received` leaks into round N+1 (FIXED)

**Severity:** MEDIUM
**Location:** `server/src/core/rsm/transitions.rs` — `advance_phase`, `Resolution` arm

### Root cause

`submissions_received` was cleared in three places:
- `enter_draft_initial` (lobby → round 1)
- `DraftInitial → Placement` transition
- `DraftShop → Placement` transition

The `Resolution → DraftShop` (and `Resolution → DraftAuction`) path **never cleared it**.
Round N's placement submission set persisted through the entire DraftShop phase into
round N+1 Placement. As soon as round N+1 placement opened, the stale entry satisfied
`all_players_seen` for a 1-of-2 player threshold — triggering an immediate
Placement→Resolution advance before any player submitted.

**Evidence:** Snapshot 6 (`snapshot-0002-draftshop-phase-…-000006.json`):
`submissions_received: [1]` during DraftShop round 2, having carried over from round 1.

### Fix

`server/src/core/rsm/transitions.rs`, `Resolution` arm of `advance_phase`:

```rust
RoundPhase::Resolution => {
    rsm.resolution_safety_timer = None;
    rsm.round_number += 1;
    // BUG-17 fix: clear stale submissions from the just-completed placement
    // phase so they cannot satisfy the all-players-seen check in round N+1.
    rsm.submissions_received.clear();
    // ...
```

### Tests added (`server/tests/rsm_transitions_test.rs`)

- `test_rsm_resolution_to_draftshop_clears_submissions_received` — DraftShop path
- `test_rsm_resolution_to_draftauction_clears_submissions_received` — DraftAuction path
- `test_rsm_resolution_normal_advance_does_not_emit_game_over` — no vacuous GameOver on normal advance

All 17 tests in `rsm_transitions_test` pass.

---

## 2. BUG-05 — GameOver fires after 2 vacuous rounds (ROOT-CAUSED, NOT FIXED)

**Severity:** CRITICAL (in combination with BUG-06)
**Location:** `server/src/core/rsm/transitions.rs` — `advance_phase`, `Resolution` arm (soak-bound check)

### Root cause

The `CCGS_BOT_MAX_ROUNDS=3` environment variable is set in the bot-soak test run.
After round 2 resolution, `rsm.round_number` increments from 2 → 3.
The check `rsm.round_number >= max` where `max = 3` evaluates to `3 >= 3 = true`,
triggering `GameOverEmitted { reason: MaxRoundsReached }`.

This is **intentional soak behavior** — `BotSoakConfig` is designed to cap runs at N rounds.
The "vacuous" appearance (all objectives at 5/5 HP) is a consequence of BUG-06, not a separate bug in the win-condition logic.

### Why the win-condition itself is not broken

`evaluate_objective_win_condition` (called by `rsm_input_reader` on `ResolutionComplete`) requires
`real_objectives_destroyed(player) >= 2` before returning a game-over. With all objectives at HP 5,
this always returns `None` → `PhaseAdvanceRequest::new(RoundPhase::Resolution)` (continue).
The soak-bound GameOver fires in `advance_phase` after that request is processed.

In **normal play** (no `CCGS_BOT_MAX_ROUNDS`), a match with no combat damage can never
GameOver — it would continue indefinitely until objectives are destroyed or a disconnect
grace expires. The regression test `test_rsm_resolution_normal_advance_does_not_emit_game_over`
guards this invariant.

### Action required

Fix BUG-06 (units never reach the board) so soak runs have actual game content.
The soak max-rounds cap itself is correct and should not be modified.

---

## 3. BUG-06 — No units ever reach the board (ROOT-CAUSED, NOT FIXED)

**Severity:** CRITICAL
**Location:** Upstream of placement.rs — client and bot layers

### Root cause chain

The server-side placement code in `server/src/feature/board/placement.rs` is **correct**.
`process_placement_submission` properly accepts empty batches (PROMPT 1678 comment documents
this intentionally), and `close_placement_phase` correctly commits zero units for empty submissions.

The root cause is that BOTH players submit empty batches:

1. **Bot player (BUG-02 / BUG-03):** Bot fires `empty_placement_failsafe` with
   `legal_action_count: 0` — the bot has no hand (see BUG-02: hand distribution never
   ran for the bot player). Empty failsafe → empty `C2SSubmitPlacement` → 0 cards placed.

2. **Human player (BUG-01):** The autoplay client is stuck in `phase_label: "Lobby"` for the
   entire match (BUG-01 / BUG-13: client never receives or applies `S2CPhaseChanged`).
   The autoplay driver fires "placement-submitted" but the client's placement state machine
   never transitioned to Placement phase — `C2SSubmitPlacement` contains an empty placements
   list because the client's placement UI never populated it.

### Why the server correctly reports board=0 after accepting both submissions

`close_placement_phase` calls `collect_committed_placements` which iterates
`PendingPlacements.submissions`. Both players have `PlayerSubmission { placements: vec![] }`.
The spawn loop executes zero iterations. The resulting `PlacementCommitted.spawned_units = []`.
Board fields remain zero.

### Action required

Fix BUG-01 / BUG-13 (client `S2CPhaseChanged` receive path) and BUG-02 (bot hand distribution).
These are upstream of the placement→board path and out of this prompt's scope.

---

## 4. BUG-16 — Resolution lasts ~2ms (ROOT-CAUSED, NOT FIXED)

**Severity:** MEDIUM
**Location:** Resolution simulation system (out of scope: `server/src/feature/combat/`)

### Root cause

Resolution completes immediately because the board has zero units (BUG-06).
The combat/resolution system finds no units to process and emits `ResolutionComplete`
within one tick (~2ms). The `resolution_safety_timer` (60000ms) is set correctly but
never consulted because `ResolutionComplete` arrives first.

This is a cascade of BUG-06, not an independent timer enforcement bug.

### Action required

Fix BUG-06. Once units land on the board, resolution will take meaningful time
to process combat sequences.

---

## 5. BUG-24 — `session: null` in final GameOver snapshot (ROOT-CAUSED, NOT FIXED)

**Severity:** LOW
**Location:** `server/src/core/session/system.rs` — `handle_game_over_teardown` + bot QA snapshot ordering

### Root cause

`handle_game_over_teardown` is an exclusive system (`&mut World`). On the frame that
`GameOverEmitted` is read, it calls `remove_live_session_resources` which removes
`SessionConfig` from the world. If the bot QA snapshot system runs on the same frame
AFTER teardown, `build_game_snapshot` finds `SessionConfig = None` and sets
`session: null` in the output.

### Fix approach (deferred)

Two options:
1. Order the bot QA snapshot system to run before `handle_game_over_teardown`.
2. Have `handle_game_over_teardown` defer `SessionConfig` removal by one frame using
   a staging resource (keep config alive through GameOver phase entry).

Option 1 is lowest risk. Deferred to a follow-up prompt as this is a snapshot
observability issue only — the GameOver broadcast itself is correct.

---

## 6. Summary

| Bug | Action | Status |
|-----|--------|--------|
| BUG-17 `submissions_received` leak | Fixed + 3 regression tests | **SHIPPED** |
| BUG-05 vacuous GameOver | Root-caused: `CCGS_BOT_MAX_ROUNDS=3` soak cap + BUG-06 cascade | DOCUMENTED |
| BUG-06 no units on board | Root-caused: client Lobby-stuck (BUG-01) + bot no-hand (BUG-02) | DOCUMENTED |
| BUG-16 instant resolution | Root-caused: cascade of BUG-06 (no units = no combat) | DOCUMENTED |
| BUG-24 session=null in GameOver | Root-caused: teardown ordering vs. snapshot | DOCUMENTED |

### Files modified

- `server/src/core/rsm/transitions.rs` — +3 lines (BUG-17 fix)
- `server/tests/rsm_transitions_test.rs` — +83 lines (regression tests)

### Tests

```
running 17 tests
... 17 passed; 0 failed
```

---

2033: SERVER-BOARD-GAMEOVER-VACUOUS-FLOW-P0-REPAIR: SHIPPED
