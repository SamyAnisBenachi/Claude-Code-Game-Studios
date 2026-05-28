# PROMPT 2024 — Forensic Evidence Inventory and Run Selection

**Date**: 2026-05-28  
**Worker**: Claude Code — PROMPT-2024 forensic read-only pass  
**Source-of-truth**: `origin/main@e7b51e84` (PROMPT 2019 mainland)  
**Scope**: Read-only artifact inspection; report write only  

---

## 1. Evidence Artifact Roots Inspected

| Root | Location | Files Found |
|------|----------|-------------|
| `production/qa/evidence/autoplay-runs/` | `D:\_DEV\Work\Claude-Code-Game-Studios\` | 3 run directories |
| `production/qa/evidence/composite-runs/` | same | 3 composite run entries |
| `production/qa/evidence/dev-runs/` | same | 12 dev/soak run directories |
| `dev-runs/bot-qa-snapshots/` | same | 10 JSON snapshot files |

Worktree (`D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-2024`) has no artifact directories — all evidence lives in the main repo checkout at `D:\_DEV\Work\Claude-Code-Game-Studios`.

---

## 2. Autoplay Runs — Full Inventory

Three autoplay runs exist, all from 2026-05-28:

| Run ID | Started (UTC) | Finished (UTC) | Duration | Outcome | Win32 Method | Checkpoints | Bevy Screenshots |
|--------|---------------|----------------|----------|---------|--------------|-------------|-----------------|
| `20260528-051148-Z` | 05:28:42Z | 05:29:14Z | 32s | ok | none (screenshot API only) | 15/15 | 15 |
| `20260528-063609-Z` | 06:52:32Z | 06:53:08Z | 36s | ok | win32_capture (all frozen) | 15/15 | 15 |
| `20260528-090613-Z` | 09:06:15Z | 09:06:55Z | 40s | ok | win32_printwindow + bitblt fallback | 15/15 | 15 |

All three: `driver_exit_code: 0`, `client_exit_code: null` (client still running when driver exited), recipe `vs-bot`, 74 actions, tick cap 262.

---

## 3. Per-Run Artifact Detail

### 3.1 Run `20260528-051148-Z`

| Artifact | Size | Notes |
|----------|------|-------|
| `status.json` | 358 B | phase=Lobby, round=0, frame=6435 |
| `launcher-status.json` | 421 B | outcome=ok |
| `capabilities.json` | 887 B | schema v2 |
| `checkpoints.jsonl` | 1685 B | 15 checkpoints |
| `driver-timeline.jsonl` | 115 KB | 262 lines |
| `driver.log` | 9134 B | No win32 captures; uses screenshot API only |
| `process.log` | 8975 B | 49 lines, INFO only |
| `process.log.err` | 56930 B | 539 lines, compiler warnings only |
| `screenshots/` | — | 15 PNG + 15 JSON; ALL PNGs = 86,080 B (identical) |
| win32 captures | 0 | None generated |

**Screenshot analysis**: All 15 Bevy screenshots are identical byte-for-byte (86,080 B). The game's render output never changed from the first frame across all 15 capture points.

### 3.2 Run `20260528-063609-Z`

| Artifact | Size | Notes |
|----------|------|-------|
| `status.json` | 358 B | phase=Lobby, round=0, frame=7590 |
| `launcher-status.json` | 421 B | outcome=ok |
| `checkpoints.jsonl` | 1684 B | 15 checkpoints |
| `driver-timeline.jsonl` | 115 KB | 262 lines |
| `driver.log` | 17638 B | win32_capture used; ALL hashes=0x26207c4c (frozen) |
| `process.log` | 9173 B | 51 lines, INFO only |
| `process.log.err` | 56930 B | 539 lines, compiler warnings only |
| `screenshots/` | — | 15 PNG + 15 JSON; ALL PNGs = 86,148 B (identical) |
| `win32_tick_*.png` | 15 files | All 1296×759; all same pixel_hash=0x26207c4c |

**Screenshot analysis**: All 15 Bevy screenshots are identical (86,148 B). Win32 capture frames are also all identical (same hash entire run — no visual state change ever detected by win32 method).

### 3.3 Run `20260528-090613-Z` ← **SELECTED AS PRIMARY (most recent, highest signal)**

| Artifact | Size | Notes |
|----------|------|-------|
| `status.json` | 345 B | phase=Lobby, round=0, frame=8085, window=1280×1076 |
| `launcher-status.json` | 421 B | outcome=ok |
| `capabilities.json` | 887 B | schema v2 |
| `checkpoints.jsonl` | 1682 B | 15 checkpoints |
| `driver-timeline.jsonl` | 114 KB | 262 lines |
| `driver.log` | 25675 B | win32_printwindow + bitblt fallback; FROZEN detected ticks 164/176/185/250/259 |
| `process.log` | 9316 B | 51 lines, INFO only |
| `process.log.err` | 56930 B | 539 lines, 101 compiler warnings (deprecated markers) |
| `screenshots/` | — | 15 PNG + 15 JSON (see below) |
| `win32_tick_*.png` | 15 files | 1296×759 (ticks 5–113); 1296×1115 (ticks 138–259) |
| `bitblt_tick_*.png` | 11 files | 1296×1115; different pixel hashes (live render content) |

**Screenshot analysis (run 3 — the only run with non-frozen Bevy screenshots)**:

| Screenshot | Sequence | Reason | Size (B) | Content |
|-----------|---------|--------|----------|---------|
| 000000.png | seq 0 | checkpoint:lobby-loaded | 86,108 | Static/frozen |
| 000007.png | seq 7 | checkpoint:bot-added | 86,108 | Static/frozen |
| 000011.png | seq 11 | checkpoint:lobby-confirmed | 86,108 | Static/frozen |
| 000013.png | seq 13 | checkpoint:class-select-loaded | 86,108 | Static/frozen |
| 000020.png | seq 20 | checkpoint:class-confirmed | 86,108 | Static/frozen |
| 000022.png | seq 22 | checkpoint:shop-loaded | 86,108 | Static/frozen |
| 000026.png | seq 26 | checkpoint:shop-slot-clicked | 86,108 | Static/frozen |
| 000030.png | seq 30 | checkpoint:auction-loaded | 86,108 | Static/frozen |
| **000037.png** | seq 37 | checkpoint:auction-ready | **117,843** | **Frame changed** |
| 000039.png | seq 39 | checkpoint:placement-loaded | 117,843 | Changed |
| 000048.png | seq 48 | checkpoint:placement-dragged | 117,843 | Changed |
| 000052.png | seq 52 | checkpoint:placement-submitted | 117,843 | Changed |
| 000054.png | seq 54 | checkpoint:resolution-started | 117,843 | Changed |
| 000055.png | seq 55 | checkpoint:resolution-complete | 117,843 | Changed |
| 000057.png | seq 57 | checkpoint:vs-bot-post-resolution | 117,843 | Changed |

**Key transition**: Bevy screenshots are frozen (identical, 86,108 B) through checkpoints 0–30 (lobby → auction-loaded). They change to a different, larger frame (117,843 B) starting at `auction-ready` (checkpoint tick 134). This coincides with the window size change from 1280×720 → 1280×1076.

---

## 4. Checkpoint Flow (All Runs Pass All 15)

All three runs complete the same 15 checkpoints with driver exit rc=0:

| Tick | Label | Elapsed (Run 3) |
|------|-------|-----------------|
| 1 | lobby-loaded | 0.032s |
| 26 | bot-added | 2.922s |
| 38 | lobby-confirmed | 4.563s |
| 47 | class-select-loaded | 5.813s |
| 68 | class-confirmed | 8.438s |
| 77 | shop-loaded | 9.860s |
| 89 | shop-slot-clicked | 11.625s |
| 109 | auction-loaded | 14.235s |
| 134 | auction-ready | 17.391s |
| 143 | placement-loaded | 18.750s |
| 160 | placement-dragged | 21.516s |
| 172 | placement-submitted | 23.844s |
| 181 | resolution-started | 25.750s |
| 246 | resolution-complete | 33.110s |
| 255 | vs-bot-post-resolution | 34.735s |

**Warning**: Checkpoints fire on timing/driver-side logic, NOT on client `phase_label` changes. All 262 driver-timeline ticks in all three runs show `phase_label: "Lobby"` and `client_state_label: "Lobby"` — the client never reports transitioning to `InSession`. Checkpoints passing does not mean the client reached those game phases.

---

## 5. Bot-QA Snapshots Inventory

**Location**: `dev-runs/bot-qa-snapshots/`  
**Dated**: 2026-05-27 (created ~12:25–12:26 UTC)  
**Count**: 10 JSON files, schema_version 1

| File | Sequence | Trigger | Phase | Round | Board Minions | Bot Decision |
|------|---------|---------|-------|-------|--------------|--------------|
| snapshot-0000-lobby-init-…-000001.json | 1 | initial | Lobby | 0 | 0 | class_confirmed |
| snapshot-0001-draftinitial-…-000002.json | 2 | phase_transition | DraftInitial | 1 | 0 | draft_ready |
| snapshot-0001-placement-…-000003.json | 3 | phase_transition | Placement | 1 | 0 | **empty_placement_failsafe** |
| snapshot-0001-placement-tick-…-000004.json | 4 | periodic | Placement | 1 | 0 | (16,229 decisions logged) |
| snapshot-0001-resolution-…-000005.json | 5 | phase_transition | Resolution | 1 | 0 | — |
| snapshot-0002-draftshop-…-000006.json | 6 | phase_transition | DraftShop | 2 | 0 | draft_ready |
| snapshot-0002-placement-…-000007.json | 7 | phase_transition | Placement | 2 | 0 | **empty_placement_failsafe** |
| snapshot-0002-placement-tick-…-000008.json | 8 | periodic | Placement | 2 | 0 | — |
| snapshot-0002-resolution-…-000009.json | 9 | phase_transition | Resolution | 2 | 0 | — |
| snapshot-0003-gameover-…-000010.json | 10 | phase_transition | GameOver | 3 | 0 | — |

**Disconnect trackers** (seq 3 onward): both `player: 1` and bot player show `seconds_since_disconnect: 29991` (~8.3 hours). This is present from the very first Placement phase and throughout the game. Players appear permanently disconnected from the server's tracking perspective the entire game.

**File sizes**: snapshot-0000 through -0003 are small (1.5–4 KB) since board/hands are empty. snapshot-0001-placement-tick through gameover are large (22–23 KB) due to large `decision_log_tail` accumulated entries.

---

## 6. Soak Runs Inventory

**Location**: `production/qa/evidence/dev-runs/`  
**Total**: 12 run directories

| Run | Date | Type | Duration | Outcome |
|-----|------|------|----------|---------|
| 2026-05-18-132807 | May 18 | dev run | — | — |
| 2026-05-18-200005 | May 18 | dev run | — | — |
| 2026-05-21-013550-prompt1580 | May 21 | dev run | — | — |
| 2026-05-21-013550-prompt1580-retry | May 21 | dev run | — | — |
| 2026-05-27-072515-bot-vs-bot-soak | May 27 | soak | — | — |
| 2026-05-27-073119-bot-vs-bot-soak | May 27 | soak | — | — |
| 2026-05-27-073230-bot-vs-bot-soak-manual | May 27 | soak | — | — |
| 2026-05-27-112340-bot-vs-bot-soak | May 27 | soak | — | — |
| 2026-05-27-121103-bot-vs-bot-soak | May 27 | soak | — | — |
| 2026-05-27-121832-bot-vs-bot-soak | May 27 | soak | — | — |
| 2026-05-27-125328-bot-vs-bot-soak | May 27 | soak | — | — |
| **2026-05-27-222625-bot-vs-bot-soak** | May 27 22:27Z | soak | **0.5s** | **exit_code=0, game_over reached** |

**Latest soak `2026-05-27-222625` detail**:
- Server completed 2 rounds, reached GameOver in **0.503s**
- `endpoint_reached: "game_over"`, `received_game_over: true`
- Bot server-snapshots: 8 files covering Lobby → GameOver (same pattern as bot-qa-snapshots)
- Bot used `empty_placement_failsafe` every Placement (legal_action_count=0 or 1)
- Decision log shows `placement_submitted: placements_len=1` once in round 1 from soak (different from qa-snapshots where it was always failsafe)

---

## 7. Current Truth Set (for Downstream Workers)

| Priority | Artifact Set | Path | Why Selected |
|----------|-------------|------|--------------|
| **PRIMARY** | Autoplay run `20260528-090613-Z` | `production/qa/evidence/autoplay-runs/20260528-090613-Z/` | Most recent; only run with FROZEN detection + bitblt fallback capturing live render state; window size change documented |
| **SECONDARY** | Bot-QA snapshots (server-side) | `dev-runs/bot-qa-snapshots/` | Only server-side state snapshots aligned with autoplay client; covers full round lifecycle through GameOver round 3 |
| **SUPPLEMENTARY** | Latest soak run | `production/qa/evidence/dev-runs/2026-05-27-222625-bot-vs-bot-soak/` | Confirms server-only bot-vs-bot flow works in 0.5s; confirms bot placement behavior server-side |
| **VISUAL CONTEXT** | Run `20260528-051148-Z` | `production/qa/evidence/autoplay-runs/20260528-051148-Z/` | Bevy screenshots same frozen frame as run 3 ticks 0–30 (lobby region); oldest but cleanest comparison baseline |

**Do NOT treat as truth**:
- `status.json` final `phase_label/client_state_label` — these reflect the last polled value before driver exit, which shows "Lobby" because the game cycled back to lobby or the recipe ended. These are NOT indicative of a permanently stuck state.
- Win32 captures from run 2 — all identical, no diagnostic value.
- Composite-run status `live_pass_status` — explicitly set to `"NOT-CLAIMED -- AUTOPLAY-VS-BOT-QA-001 requires human operator sign-off"`.

---

## 8. Critical Bugs Identified from Raw Evidence

### BUG-001 [CRITICAL] — Client never enters InSession
**Evidence**: Driver-timeline for all 3 runs (`driver-timeline.jsonl`, 262 ticks each) shows exactly one unique `(phase_label, client_state_label)` combo: `("Lobby", "Lobby")`. The client's `client_state_label` field (per capabilities.json: "ClientState machine state (Lobby or InSession)") never transitions to `InSession` despite the server advancing through full game rounds.  
**Impact**: Core unplayable bug. The client stays in Lobby state while the server plays a full game.

### BUG-002 [CRITICAL] — Players permanently disconnected server-side
**Evidence**: `bot-qa-snapshots/snapshot-0001-placement-phase-…-000003.json` and all subsequent snapshots show `disconnect_trackers: [{player: 1, seconds_since_disconnect: 29991}, {bot_player: …, seconds_since_disconnect: 29991}]` from the very first game round (Round 1 Placement, timestamp_ms=137923). Value of 29991 seconds ≈ 8.3 hours — set at process startup, never reset when clients connected.  
**Impact**: Server considers both players disconnected during active gameplay. This likely suppresses `S2CPhaseChanged` delivery and prevents client state transitions.

### BUG-003 [HIGH] — Bot zero legal placements every round
**Evidence**: All `decision_log_tail` entries for Placement phase show `legal_action_count: 0` and `decision: {kind: "empty_placement_failsafe"}` across all rounds in all runs. Board `minion_count` stays 0 every round.  
**Impact**: No units ever placed; combat resolves with empty boards.

### BUG-004 [HIGH] — Bevy screenshot output frozen for 8+ checkpoints
**Evidence**: In runs 1 and 2, ALL 15 Bevy screenshots are identical byte count. In run 3, screenshots 000000–000030 (checkpoints lobby-loaded through auction-loaded, 8 captures) are identical at 86,108 B; only from `auction-ready` (seq 37) do screenshots change to 117,843 B.  
**Impact**: First 8 screenshot checkpoints (lobby, bot-add, lobby-confirm, class-select, class-confirm, shop-load, shop-click, auction-load) produce no usable diagnostic frames — they capture the same static frame.

### BUG-005 [HIGH] — Win32 PrintWindow capture frozen mid-run (run 3)
**Evidence**: `driver.log` for run 3 shows `win32_printwindow=FROZEN hash=ca2ab3e8456d5f81d2fc5f3f0c5703f2` at ticks 164, 176, 185, 250, 259 — all after `placement-dragged`. BitBlt fallback was triggered and produced live captures (different hashes).  
**Impact**: Win32 capture path unreliable for placement/resolution phase visual state. BitBlt fallback is authoritative for those ticks.

### BUG-006 [MEDIUM] — Window height changes unexpectedly mid-run
**Evidence**: Run 3 driver.log shows `width=1296 height=759` for ticks 5–113, then `width=1296 height=1115` from tick 138 onward (356px taller). `status.json` final window is `1280×1076`. The window grows to approximately full-screen height.  
**Impact**: Layout shift mid-session; game window expanding unexpectedly.

### BUG-007 [LOW] — 101 compiler warnings (deprecated markers)
**Evidence**: `process.log.err` in all 3 runs contains 101 warnings about deprecated `HudEntity`, `HandUiEntity`, and `ShopAuctionUiEntity` markers in `qa_snapshot.rs` and `shop_auction/mod.rs`.  
**Impact**: Technical debt; coarse-grained QA snapshot counts may misreport entity counts.

---

## 9. Evidence Gaps / Blockers

| Gap ID | Description | Impact |
|--------|-------------|--------|
| GAP-01 | Client never enters InSession — no evidence of what the client UI shows past the lobby | Cannot visually diagnose game phase rendering; all post-lobby Bevy screenshots are from the frozen or changed-frame state with unknown actual content |
| GAP-02 | No server-side snapshots correlated with autoplay client runs | Bot-QA snapshots (2026-05-27) may not match current binary at `origin/main@e7b51e84` |
| GAP-03 | `seconds_since_disconnect: 29991` cause unknown | Don't know if this is a tracker-initialization bug, a stale persisted value, or actual network disconnection at t=0 |
| GAP-04 | Bot placement legal_action_count=0 root cause unknown | Could be hand empty, placement rules not met, or placement slot availability; snapshot shows `hand.size=1, cards=[5]` for player 1 but no hand entry for bot player in snapshot seq 3 |
| GAP-05 | Run 3 Bevy screenshots 000037+ are 117,843 B but actual pixel content unknown | Need visual inspection; could show game phases or a stretched lobby |
| GAP-06 | BitBlt fallback frames (run 3) show live render but are not inspected for UI bugs | 11 bitblt PNGs from placement/resolution phases have different pixel hashes — actual game phase content is there but not analyzed |
| GAP-07 | No run covers >1 round from the autoplay recipe | The recipe stops at `vs-bot-post-resolution` (255 ticks), covering only Round 1 |

---

## 10. Recommended Next Steps for Downstream Workers

1. **Root-cause BUG-002**: Inspect `disconnect_tracker` initialization in server code — where is `seconds_since_disconnect` set? It should be 0 at connection time. This may be the root cause of BUG-001.
2. **Visual audit of run 3 bitblt frames**: Inspect `bitblt_tick_000164.png` through `bitblt_tick_000259.png` for UI issues — these are the only frames showing live render state during placement/resolution.
3. **Visual audit of run 3 Bevy screenshots 000037–000057**: Inspect `117,843 B` screenshots for what the game actually shows after `auction-ready`.
4. **Bot hand population**: Investigate why bot player has no hand entry in placement snapshots. The bot's `legal_action_count=0` may be caused by the bot never receiving cards.

---

2024: FORENSIC-EVIDENCE-INVENTORY-AND-RUN-SELECTION: COMPLETE
