# PROMPT-2032 — Bot Placement Failsafe Spinloop P0 Repair

**Date:** 2026-05-28  
**Source-of-truth at task start:** origin/main@8863e26c (post PROMPT-2025 audit)  
**Worktree:** `D:/_DEV/Work/gcs-app-worktrees/lanesandlies/PROMPT-2032` on `work/PROMPT-2032`  
**Files modified:**
- `server/src/feature/bot/action_loop.rs`
- `server/src/feature/bot/state.rs`

---

## 1. Bug Status at Origin/Main Baseline

The PROMPT-2025 audit (snapshot evidence from `dev-runs/bot-qa-snapshots/`) identified
four bugs in the bot placement scheduler:

| Bug | Symptom | Status at origin/main@8863e26c |
|-----|---------|-------------------------------|
| BUG-03 | `empty_placement_failsafe` spam ~1/ms, 16 229 decisions in 2 rounds | **ALREADY FIXED** by PROMPT 1677 (`c20eb100`, 2026-05-27) |
| BUG-19 | `next_decision_at_ms` / `failsafe_deadline_ms` always null | **NOT YET FIXED** — fields exist in `BotPhaseTiming` but never populated |
| BUG-21 | `last_decision_at_ms` stuck at lobby class decision | **ALREADY FIXED** by PROMPT 1721 (`cbf4479d`, 2026-05-28 00:03) |
| BUG-23 | Duplicate timestamps in decision log | **FIXED as a side-effect** of BUG-03 fix — debounce ensures one entry per (player, round) |

### Why the audit snapshots showed spam despite PROMPT 1677

The bot-QA snapshots in `dev-runs/bot-qa-snapshots/` and the autoplay runs in
`production/qa/evidence/autoplay-runs/20260528-*-Z/` were generated from a server
binary built **before** PROMPT 1677 was committed. The fixes were in the codebase
but the binary was stale. This is confirmed by:
- PROMPT 1677 committed: 2026-05-27 13:00
- PROMPT 1721 committed: 2026-05-28 00:03
- Autoplay runs timestamped: 2026-05-28 05:11–09:06 (same day, binary not rebuilt)
- `dev-runs/` is gitignored (untracked) — captured separately from the code commits

---

## 2. Fix Applied by PROMPT 2032 (BUG-19)

**Root cause:** `BotState.phase_timing.next_decision_at_ms` and
`BotState.phase_timing.failsafe_deadline_ms` were designed in the original
`BotPhaseTiming` scaffold (state.rs) to track scheduling state, but no code
in `action_loop.rs` ever populated them. QA snapshots therefore always showed
`null` for both fields across the entire game, making the bot's scheduling
state unobservable.

**Fix in `state.rs`:**  
Added `BOT_PLACEMENT_FAILSAFE_WINDOW_MS: u64 = 10_000` constant alongside the
existing `BOT_THINK_DELAY_*` and `BOT_SAFETY_MARGIN_MS` constants.

**Fix in `action_loop.rs`, Placement arm:**

1. **Phase entry arming** — on the first tick of Placement for each `(player_id, round)`
   (detected as `!placement_submitted.contains(&debounce_key)` with
   `failsafe_deadline_ms.is_none()` guard), sets:
   - `phase_timing.next_decision_at_ms = Some(ts)` — bot scheduled to decide NOW
   - `phase_timing.failsafe_deadline_ms = Some(ts + BOT_PLACEMENT_FAILSAFE_WINDOW_MS)` — deadline 10 s from entry

2. **Post-emission cleanup** — immediately after emitting the placement submission,
   clears `phase_timing.next_decision_at_ms = None` ("no pending decision";
   decision was just executed). `failsafe_deadline_ms` stays set until phase exits
   so snapshots taken after submission still show the deadline was armed.

3. **Phase-exit cleanup** — in the `Lobby | Resolution | GameOver` arm, clears both
   `next_decision_at_ms` and `failsafe_deadline_ms` for all bots so the inter-round
   idle state shows fully null scheduling fields.

4. **RSM confirmation cleanup** — if `submissions_received.contains(player_id)` is true
   (RSM accepted the submission), clears both fields immediately.

---

## 3. Tests Added

Two new regression tests in `server/src/feature/bot/action_loop.rs::tests`:

| Test | Validates |
|------|-----------|
| `placement_arms_failsafe_deadline_on_entry` | `failsafe_deadline_ms` is `Some` after first tick; `next_decision_at_ms` is `None` (cleared post-emission) |
| `placement_phase_timing_cleared_on_resolution` | Both fields are `None` after transitioning to `RoundPhase::Resolution` |

---

## 4. Existing Tests (All Pass)

Previously added tests that remain green:

| Test | What it proves |
|------|---------------|
| `placement_emits_empty_failsafe_once` | Debounce emits exactly one placement per round |
| `placement_debounce_survives_many_ticks_without_rsm_update` | 20 ticks → exactly 1 decision log entry (no spam) |
| `placement_decision_updates_last_decision_at_ms` | `last_decision_at_ms` is Some after placement |
| `draft_decision_updates_last_decision_at_ms` | BUG-21 coverage for draft path |
| `auction_pass_updates_last_decision_at_ms` | BUG-21 coverage for auction path |

Full run: **38 bot tests pass, 0 failures.**

---

## 5. Path Allowlist Review

Modified files:
```
server/src/feature/bot/action_loop.rs  ← preferred scope
server/src/feature/bot/state.rs        ← preferred scope
```

No forbidden files touched (`client/src/**`, `acquisition/**`, `board/**`, `combat/**`,
`session-state/**`, `sprint-status.yaml`, unrelated Cargo/CI files).

`git diff --check` passes (no trailing whitespace or merge conflict markers).

---

## 6. Summary of All Bugs from PROMPT-2025

| Bug | Resolution |
|-----|-----------|
| BUG-03 (spam 16 229 decisions) | Fixed by PROMPT 1677 — local `placement_submitted` debounce per `(player_id, round)` |
| BUG-19 (null timestamps) | Fixed by PROMPT 2032 — `phase_timing` armed on Placement entry, cleared on phase exit |
| BUG-21 (last_decision_at_ms stuck) | Fixed by PROMPT 1721 — all six decision-push sites update the field |
| BUG-23 (duplicate timestamps) | Fixed as side-effect of BUG-03 — debounce prevents same-round re-entry |

---

2032: BOT-PLACEMENT-FAILSAFE-SPINLOOP-P0-REPAIR: SHIPPED
