# PROMPT-2025 â€” Snapshot / Log / Gamestate Correlation Audit

**Date:** 2026-05-28
**Source-of-truth:** origin/main@e7b51e84 (post PROMPT-2019 mainland)
**Evidence scope:**
- 10 bot-QA snapshots: `dev-runs/bot-qa-snapshots/snapshot-000[0-3]-*.json`
- 3 autoplay runs: `production/qa/evidence/autoplay-runs/20260528-{051148,063609,090613}-Z/`

---

## 1. Phase-by-Phase Timeline

### 1.1 Bot-QA Snapshots (server-side game state, tick-stamped)

| Seq | Snapshot file | Server ms | Phase | Round | Notes |
|-----|---------------|-----------|-------|-------|-------|
| 1 | snapshot-0000-lobby-init-â€¦-000001.json | 137890 | Lobby | 0 | Bot present, no session. `decision_log_total: 1` (class_confirmed only). |
| 2 | snapshot-0001-draftinitial-phase-â€¦-000002.json | 137905 | DraftInitial | 1 | +15 ms. Session created. Both players present. Both disconnect timers: 30000 s. Bot fires `draft_ready` immediately; `legal_action_count: null`. |
| 3 | snapshot-0001-placement-phase-â€¦-000003.json | 137923 | Placement | 1 | +18 ms. Player 1 hand: `[5]` (1 card). Bot hand: **missing**. Player 1 gold: 5â†’3 (unexplained). Bot fires `empty_placement_failsafe` with `legal_action_count: 0` immediately. |
| 4 | snapshot-0001-placement-tick-â€¦-000004.json | 147924 | Placement | 1 | +10001 ms (timer expired). `decision_log_total: 7647`. Player 1 submitted. Grace timer 250 ms. Board: 0 units. |
| 5 | snapshot-0001-resolution-phase-â€¦-000005.json | 148175 | Resolution | 1 | +251 ms. Board: 0 units. All objectives HP 5. `decision_log_total: 7849`. |
| 6 | snapshot-0002-draftshop-phase-â€¦-000006.json | 148177 | DraftShop | 2 | +2 ms. Bot fires `draft_ready`; `legal_action_count: null`. Economies incremented. Hand still `[5]`. `submissions_received: [1]` stale. |
| 7 | snapshot-0002-placement-phase-â€¦-000007.json | 148181 | Placement | 2 | +4 ms. Bot immediately fires `empty_placement_failsafe`. Hand unchanged. |
| 8 | snapshot-0002-placement-tick-â€¦-000008.json | 158182 | Placement | 2 | +10001 ms. Player 1 submitted. Board: 0 units. |
| 9 | snapshot-0002-resolution-phase-â€¦-000009.json | 158435 | Resolution | 2 | +253 ms. `decision_log_total: 16229`. Board: 0 units. All objectives HP 5. |
| 10 | snapshot-0003-gameover-phase-â€¦-000010.json | 158437 | GameOver | 3 | +2 ms. `session: null`. All 10 objectives HP 5, none destroyed. No combat occurred. |

### 1.2 Autoplay Runs (client-side driver, tick-stamped)

All 3 runs execute the same 16-checkpoint `vs-bot` recipe. Summary across runs:

| Driver Tick | Checkpoint label | Client `phase_label` | Client `round` |
|-------------|-----------------|----------------------|----------------|
| 1 | lobby-loaded | **Lobby** | **0** |
| 26 | bot-added | **Lobby** | **0** |
| 38 | lobby-confirmed | **Lobby** | **0** |
| 47 | class-select-loaded | **Lobby** | **0** |
| 68 | class-confirmed | **Lobby** | **0** |
| 77 | shop-loaded | **Lobby** | **0** |
| 89 | shop-slot-clicked | **Lobby** | **0** |
| 109 | auction-loaded | **Lobby** | **0** |
| 134 | auction-ready | **Lobby** | **0** |
| 143 | placement-loaded | **Lobby** | **0** |
| 160 | placement-dragged | **Lobby** | **0** |
| 172 | placement-submitted | **Lobby** | **0** |
| 181 | resolution-started | **Lobby** | **0** |
| 246 | resolution-complete | **Lobby** | **0** |
| 255+ | all remaining | **Lobby** | **0** |

**Every tick across all 3 runs reports `phase_label: "Lobby"` and `round: 0`.** The server snapshot sequence shows a complete 2-round match; the client reflects none of it.

---

## 2. Bug Table

| ID | Severity | Symptom | Evidence path + location | Subsystem | Reproducible |
|----|----------|---------|--------------------------|-----------|--------------|
| BUG-01 | **CRITICAL** | Client `phase_label` and `round` stuck at `"Lobby"` / `0` for entire game â€” all 260+ driver ticks across 3 runs | `driver-timeline.jsonl` ticks 1â€“262 (all 3 runs); `status.json` all 3 runs: `"phase_label":"Lobby","round":0` | ClientState / `S2CPhaseChanged` receive path | Yes |
| BUG-02 | **CRITICAL** | Bot player never receives a hand. No hand entry for bot in any snapshot from seq 3 onward | `snapshot-0001-placement-phase-â€¦-000003.json` l.76-84: only player 1 in `hands[]`; same pattern snaps 4â€“9 | Server hand distribution / bot player hand allocation | Yes |
| BUG-03 | **CRITICAL** | Bot fires `empty_placement_failsafe` with `legal_action_count: 0` in a tight ~1 ms loop â€” ~7849 decisions in round 1, ~8380 in round 2, **16229 total** across a 2-round game | `snapshot-0001-placement-tick-â€¦-000004.json` l.168-936: 56+ identical entries; `decision_log_total` 7647â†’7849â†’16229 across snaps 4,5,9 | Bot decision engine â€” missing loop throttle / `next_decision_at_ms` guard | Yes |
| BUG-04 | **CRITICAL** | DraftInitial completes in 15 ms despite `draft_initial: 44999` timer â€” auto-advance bypasses configured timer | `snapshot-0001-draftinitial-â€¦-000002.json` l.24: `draft_initial: 44999`; snap 3 at 137923 = only 18 ms after snap 2 | Phase timer enforcement for draft phases | Yes |
| BUG-05 | **CRITICAL** | GameOver reached after 2 rounds with all 10 objectives at full HP (5/5, none destroyed) â€” win condition fires vacuously | `snapshot-0003-gameover-â€¦-000010.json` l.76-137: all objectives `hp: 5, destroyed: false`; round=3 | Win condition check / round-advance trigger | Yes |
| BUG-06 | **CRITICAL** | No units ever placed on board across all rounds â€” board fields (`minion_count`, `trap_count`, `structure_count`, `field_count`) remain 0, `per_player_minions: []` | Snaps 3â€“9: all board fields = 0 throughout | Placement submission / server unit placement resolution | Yes |
| BUG-07 | **HIGH** | Player 1 starts with 1 card (`[5]`) and retains it unchanged through DraftShop into round 2 â€” draft awarding is non-functional | Snap 3 l.76-84: `size:1, cards:[5]`; snap 6 (DraftShop complete): same; snap 7 round-2 Placement: same | DraftInitial / DraftShop card awarding system | Yes |
| BUG-08 | **HIGH** | Both players show `seconds_since_disconnect: 30000` (~8.3 hours) in the very first snapshot â€” disconnect tracker initialized to wrong value | `snapshot-0001-draftinitial-â€¦-000002.json` l.12-19: both players `seconds_since_disconnect: 30000` | Disconnect tracker initialization | Yes |
| BUG-09 | **HIGH** | `draft_ready` bot decision fires with `legal_action_count: null` in both DraftInitial and DraftShop â€” not 0 or positive | Snap 2 l.178: `legal_action_count: null`; snap 6 l.932: same | Bot action enumeration / decision logging in draft phases | Yes |
| BUG-10 | **HIGH** | Player 1 gold drops 5â†’3 at DraftInitialâ†’Placement transition with no card purchase, auction, or charge recorded | Snap 2 l.59: player 1 gold=5; snap 3 l.59: player 1 gold=3 | Economy deduction in DraftInitial phase | Yes |
| BUG-11 | **HIGH** | `SetForegroundWindow` returns 0 on every capture call in runs 1 and 2 â€” win32 capture infrastructure unreliable | `driver.log` run 1 lines 5,15,21,27â€¦: `SetForegroundWindow returned 0 â€” trying BringWindowToTop` | Win32 foreground/capture driver | Yes |
| BUG-12 | **HIGH** | Run 3: `win32_printwindow=FROZEN` on 10+ consecutive checkpoints (ticks 51,72,81,93,113,147,164,176,185,250,259), requiring `desktop_bitblt` fallback every time | `driver.log` run 3 line 66: `FROZEN hash=0874d30fâ€¦` and subsequent lines | Win32 PrintWindow â€” GPU-accelerated window not painting into DIB surface | Yes |
| BUG-13 | **HIGH** | `client_state_label` stuck at `"Lobby"` for all 260+ ticks â€” `ClientState` never transitions to `InSession` | `driver-timeline.jsonl` every entry all 3 runs: `"client_state_label":"Lobby"` | ClientState machine / session join flow | Yes |
| BUG-14 | **HIGH** | Bot `rng_word_counter` stays 0 across all snapshots despite 16229 decisions â€” RNG path is never consumed | Snap 1 l.47: `rng_word_counter: 0`; snap 10 l.143: still `0` | Bot RNG consumption / seeded decision path | Yes |
| BUG-15 | **MEDIUM** | DraftShop completes in 4 ms with `draft_shop: 30000` timer configured â€” same instant-skip pattern as BUG-04 | Snap 6 l.24: `draft_shop: 30000`; snap 7 at 148181 = 4 ms later | Phase timer enforcement for DraftShop | Yes |
| BUG-16 | **MEDIUM** | Resolution phase lasts ~2 ms in both rounds despite `resolution_safety: 60000` â€” no simulation or animation occurs | Snap 5 l.29: `resolution_safety: 60000`; snap 6 arrives 2 ms after resolution start | Resolution controller / simulation scheduler | Yes |
| BUG-17 | **MEDIUM** | `submissions_received: [1]` persists from round 1 into round 2 DraftShop â€” not cleared on round advance | Snap 6 l.10: `submissions_received: [1]` during DraftShop round 2 | Round state reset / submissions list clear | Yes |
| BUG-18 | **MEDIUM** | `draft_ready_players` always `[]` in every snapshot including during/after DraftInitial and DraftShop, despite bot firing `draft_ready` | Snaps 2,3,6,7: `draft_ready_players: []` | Draft ready-player tracking | Yes |
| BUG-19 | **MEDIUM** | Bot `next_decision_at_ms` and `failsafe_deadline_ms` always null across all snapshots, even during the active placement failsafe spin-loop | Snaps 1â€“10 bot entry: `next_decision_at_ms: null, failsafe_deadline_ms: null` | Bot deadline / scheduler tracking | Yes |
| BUG-20 | **MEDIUM** | 101 compiler warnings from deprecated `HudEntity`, `HandUiEntity`, `ShopAuctionUiEntity` markers in production code paths | `process.log.err` (run 1) lines 1â€“537: deprecated struct references to `qa_snapshot.rs:923/925/927`, `hud/mod.rs`, `hand/mod.rs:982+`, `shop_auction/mod.rs:984+` | QA snapshot telemetry / UI marker architecture | Yes |
| BUG-21 | **MEDIUM** | Bot `last_decision_at_ms` stuck at 137889 (lobby class decision) through entire game including snap 10 â€” never updated for any of the 16228 subsequent decisions | Snaps 1â€“10 bot entry: `last_decision_at_ms: 137889` | Bot state tracking / `last_decision_at_ms` update path | Yes |
| BUG-22 | **MEDIUM** | Window height changes 759â†’1115 px mid-run between ticks 134 and 138 in run 3 (after `auction-ready` checkpoint) â€” affects all subsequent capture coordinates | `driver.log` run 3 line 167: `height=1115` vs earlier `height=759`; `status.json` run 3: `window_logical_size:[1280,1076]` | Window resize handling / Bevy `Window` re-scale event | Yes |
| BUG-23 | **LOW** | Duplicate timestamps in bot decision log: entries at ms 147863, 148109, 148138, 158350, 158370, 158417 appear twice consecutively with identical data | Snap 4 l.470/476 (both ms=147863); snap 5 l.293/299 (both ms=148109); snap 9 l.348/354 (both ms=158350) | Bot decision deduplication / tick-to-ms collision | Yes |
| BUG-24 | **LOW** | `session: null` in final GameOver snapshot â€” session torn down before GameOver phase can read it; objectives/scores inaccessible at end screen | Snap 10 l.21: `"session": null`; all prior snaps have valid session object | Session lifecycle / teardown ordering relative to GameOver phase | Yes |
| BUG-25 | **LOW** | `client_exit_code: null` in all 3 `launcher-status.json` files â€” client never observed to exit cleanly; driver exits first | `launcher-status.json` all 3 runs: `"client_exit_code": null, "outcome": "ok"` | Autoplay launcher / client exit observation | Yes |

---

## 3. Bugs That Make the Game Unplayable

The following bugs individually prevent a functional match. Together they mean no real game can complete:

### BUG-01 + BUG-13 (CRITICAL) â€” Client never leaves Lobby
Every driver tick across all 3 runs reports `phase_label: "Lobby"` and `client_state_label: "Lobby"` with `round: 0`. The server progresses through DraftInitial â†’ Placement â†’ Resolution â†’ DraftShop â†’ Placement â†’ Resolution â†’ GameOver, but the client reflects none of it. The client is not receiving or applying `S2CPhaseChanged` messages. Every game UI screen (hand, board, HUD, phase banners) operates on perpetually stale Lobby state.

### BUG-03 (CRITICAL) â€” Bot placement spin-loop
The bot fires `empty_placement_failsafe` with `legal_action_count: 0` approximately once per millisecond during every placement phase â€” 16,229 total decisions in a 2-round game. This is a server-side runaway CPU loop with no throttle, deadline, or backoff. At scale it will cause server-side CPU exhaustion.

### BUG-05 (CRITICAL) â€” Premature GameOver with no combat
The win condition triggers after only 2 rounds while all 10 objectives remain at full HP (5/5, none destroyed). A real match can never complete correctly.

### BUG-06 (CRITICAL) â€” No units placed on board
No units from either player are ever applied to the board across all rounds. The core gameplay loop (place units â†’ resolve combat â†’ deal damage) never executes.

### BUG-07 (HIGH) â€” Draft card awarding broken
Player 1 retains a single card (`[5]`) from start to finish. The bot has no hand at all. Neither DraftInitial nor DraftShop awards cards, so deck-building is non-functional.

---

## 4. Raw Key Observations Per Evidence File

### `dev-runs/bot-qa-snapshots/snapshot-0000-lobby-init-â€¦-000001.json`
Clean lobby. `decision_log_total: 1` (class_confirmed). No session, no objectives, no economies. Normal baseline.

### `snapshot-0001-draftinitial-phase-â€¦-000002.json`
Session created 15 ms after lobby. Both players present with `seconds_since_disconnect: 30000` (BUG-08). Economies initialized (5 gold, 1 mana each). 10 objectives at HP 5. Bot fires `draft_ready` with `legal_action_count: null` (BUG-09) at snapshot time. Hands empty â€” expected at this phase start.

### `snapshot-0001-placement-phase-â€¦-000003.json`
18 ms later. Player 1 hand: 1 card. Bot: no hand entry (BUG-02). Player 1 gold: 3 (unexplained drop from 5, BUG-10). Bot immediately fires `empty_placement_failsafe` (BUG-03 onset).

### `snapshot-0001-placement-tick-â€¦-000004.json`
10001 ms later (timer expired). `decision_log_total: 7647` â€” ~7644 failsafe decisions in ~10 seconds (BUG-03). Player 1 submitted. Grace timer 250 ms active. Board: 0 units. Disconnect counters counting backward from 30000 (BUG-08 still visible). 56 identical failsafe log entries in tail (tail bounded).

### `snapshot-0001-resolution-phase-â€¦-000005.json`
251 ms after placement end. Board: 0 units. All objectives HP 5. `decision_log_total: 7849`. `resolution_safety: 60000` set but resolution lasted 251 ms (BUG-16).

### `snapshot-0002-draftshop-phase-â€¦-000006.json`
2 ms after resolution start â€” effectively instant (BUG-16 confirmed). DraftShop round 2. Bot fires `draft_ready` immediately with `legal_action_count: null` (BUG-09). Hand still `[5]` â€” DraftShop awarded nothing (BUG-07). `submissions_received: [1]` stale from round 1 (BUG-17). Economies incremented normally.

### `snapshot-0002-placement-phase-â€¦-000007.json`
4 ms after DraftShop â€” instant skip (BUG-15). Bot fires `empty_placement_failsafe` immediately again. Same hand, no bot hand.

### `snapshot-0002-placement-tick-â€¦-000008.json`
Pattern identical to round 1. `decision_log_total` approaching 16000+. Player 1 submitted.

### `snapshot-0002-resolution-phase-â€¦-000009.json`
`decision_log_total: 16229`. Board: 0 units. All objectives HP 5. Duplicate timestamp entries at ms 158350 (BUG-23).

### `snapshot-0003-gameover-phase-â€¦-000010.json`
`session: null` (BUG-24). All 10 objectives HP 5, none destroyed. GameOver with no combat (BUG-05). `decision_log_total: 16229` unchanged â€” 0 bot decisions during 2 ms GameOver transition.

### `production/qa/evidence/autoplay-runs/20260528-051148-Z/`
First run. Baseline. `SetForegroundWindow returned 0` throughout (BUG-11). No win32 screenshots. `phase_label: "Lobby"` and `round: 0` for all 262 ticks (BUG-01, BUG-13). 101 compiler warnings from deprecated markers (BUG-20). `client_exit_code: null` (BUG-25). Outcome: `"ok"` (misleading â€” no real game completed).

### `production/qa/evidence/autoplay-runs/20260528-063609-Z/`
Second run. Win32 capture added. Win32 pixel hash `0x26207c4c` constant across all checkpoints â€” screen not changing (corroborates BUG-01: client stuck in lobby). `SetForegroundWindow returned 0` throughout (BUG-11). Same phase stall. `client_exit_code: null`.

### `production/qa/evidence/autoplay-runs/20260528-090613-Z/`
Third run. `foreground_robust` logic added â€” `SetForegroundWindow` succeeds on many calls. `win32_printwindow=FROZEN` on 10 checkpoints (BUG-12) â€” GPU window not flushing to GDI; fallback to `desktop_bitblt` each time. Window height changes 759â†’1115 px at tick 138 mid-run (BUG-22). Phase_label permanently `"Lobby"` (BUG-01). `client_exit_code: null`.

---

## 5. Summary

The game is **completely unplayable**. Root causes traceable directly to artifacts:

1. **Client stuck in Lobby** (BUG-01 / BUG-13): server completes a full 2-round match; client never receives or applies phase transitions. All gameplay UI operates on stale Lobby state. Reproduced in all 3 autoplay runs.

2. **Bot spin-loop** (BUG-03): 16,229 `empty_placement_failsafe` decisions in 2 rounds at ~1 decision/ms. No throttle, deadline, or backoff. Server-side runaway loop.

3. **No board units** (BUG-06) + **no combat damage** (BUG-05): placement submission is accepted but no units land on the board. GameOver fires after 2 vacuous rounds with all objectives intact.

4. **Draft awarding broken** (BUG-07): neither DraftInitial nor DraftShop delivers cards to either player. Bot has no hand; player 1 keeps 1 card throughout.

5. **Secondary signal**: all phase timers (DraftInitial 44999 ms, DraftShop 30000 ms, Resolution 60000 ms) are bypassed within milliseconds, suggesting a single broken timer-enforcement path upstream of all draft/resolution phases.

---

2025: SNAPSHOT-LOG-GAMESTATE-CORRELATION-AUDIT: COMPLETE
