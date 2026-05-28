# PROMPT 2028 — Player Flow Unplayable Bug Classification

**Date:** 2026-05-28 (updated after PROMPT 2024 landed)
**Branch:** `work/PROMPT-2028`
**Source-of-truth:** `origin/main@5016bc07` (PROMPT 2024)
**Previous revision:** against `origin/main@e7b51e84` (PROMPT 2019)
**Scope:** Read-only audit — no source edits, no production/sprint writes.

### Evidence Sources

| Source | Key contribution |
|---|---|
| `reports/PROMPT-2024-forensic-evidence-inventory-and-run-selection.md` | Raw artifact inventory; all 3 autoplay runs; all 10 bot-qa-snapshots; confirms client never InSession |
| `reports/PROMPT-2023-post-2019-2022-game-completion-readiness-refresh.md` | Sprint 18 state 9/12; FRAG-03 closed (poll_phase); R-04 closed (Add Bot docs) |
| `reports/PROMPT-2018-bot-autoplay-current-state-closure-audit-after-2016.md` | Stop-doing list; operator gate; viewport_shrink_guard loose end |
| `reports/PROMPT-1985-bot-autoplay-story-readiness-report-refresh-after-1976.md` | 3 PARTIAL runs; AC-VPT status |
| `reports/PROMPT-1937-qa-snapshot-observability-gap-report-refresh-after-1931.md` | 14 snapshot gaps (GAP-1..14) |
| `reports/PROMPT-1883-live-two-client-full-flow-retest-after-1872.md` | Binary staleness panic; server log; Lightyear mismatch |
| `reports/PROMPT-1848-autoplay-recipe-visible-target-coverage-map.md` | FRAG-01..06 register |
| `production/qa/evidence/captures/playable-client-real-e2e-loop/phase-captures.md` | Attempt 5 confirms: no room/session/game flow observed in live logs |
| `production/qa/bugs/QA-COND-0001..0007` | Closed (0001, 0007); accepted risk (0006) |

---

## 1. Executive Summary

Two independent evidence streams, examined after PROMPT-2024's forensic pass,
confirm the game is unplayable end-to-end in both the human two-client scenario
and the automated (autoplay driver) scenario:

**Human scenario (PROMPT 1883):** Both clients panic at Lightyear handshake due to a
binary protocol mismatch (client.exe 7 days stale). No human player reaches the lobby UI.

**Autoplay scenario (PROMPT 2024):** The client NEVER transitions out of `ClientState::Lobby`
across all 262 driver ticks in all 3 runs. All 15 checkpoint labels fire on driver
timing logic alone — the client never receives a `S2CPhaseChanged` that would move it
to `InSession`. The server runs a headless game with bot players; the autoplay Bevy
client and the bot-soak server are not joined to the same session.

Additionally, the bot always uses `empty_placement_failsafe` (zero legal actions),
meaning no units are ever placed and combat resolves with an empty board every round.

**Net result:** The entire flow from lobby through game-over is BLOCKED or DEGRADED
at every stage for both human and automation. After PROMPT 2024, the root causes
are no longer hypothetical — they are confirmed from raw artifact evidence.

---

## 2. Flow Matrix (Updated Against PROMPT-2024 Truth Set)

| Flow Stage | Human — Current State | Human — Fresh Binaries | Automation (Autoplay Driver) | Primary Evidence |
|---|---|---|---|---|
| **Server startup** | PLAYABLE | PLAYABLE | PLAYABLE | PROMPT 1883 server log; soak runs |
| **Client launch / connect** | **BLOCKED** — Lightyear protocol panic | PLAYABLE (Attempt 5) | PLAYABLE (driver connects) | PROMPT 1883; phase-captures attempt-5 |
| **Lobby / room create** | **BLOCKED** | **DEGRADED** — server path verified (prompt-290); UI not human-tested | **BLOCKED** — client never exits Lobby state (all 262 ticks `client_state_label: "Lobby"`) | PROMPT 2024 §4; driver-timeline.jsonl |
| **Class select / confirm** | **BLOCKED** | **DEGRADED** — server path verified | **BLOCKED** — client never InSession | PROMPT 2024 §4 |
| **Draft Initial (free pick)** | **BLOCKED** | **DEGRADED** — server verified; card rarity absent from snapshot | **BLOCKED** — client never InSession | PROMPT 2024 §4; GAP-4 |
| **Shop (DraftShop)** | **BLOCKED** | **DEGRADED** — server verified; 3-slot UI not human-tested | **BLOCKED** — client never InSession | PROMPT 2024 §4 |
| **Auction** | **BLOCKED** | **DEGRADED** — server verified; leader label perspective unverified | **BLOCKED** — client never InSession | PROMPT 2024 §4 |
| **Placement (drag + submit)** | **BLOCKED** | **DEGRADED** — server verified; placement ACK heuristic (BUG-P1-01) | **BLOCKED** — client never InSession; even if reached, FRAG-01 still open | PROMPT 2024 §4; GAP-7 |
| **Resolution / combat replay** | **BLOCKED** | **DEGRADED** — QA harness evidence (COND-0007 closed); bot always empty board | **BLOCKED** — client never InSession | PROMPT 2024 §5 BUG-003 |
| **Game Over screen** | **BLOCKED** | **DEGRADED** — screen exists; outcome (win/loss/draw) not displayed (BUG-P1-02) | **BLOCKED** — client never InSession | GAP-1/2; PROMPT 2015 |
| **Return to Lobby** | **BLOCKED** | **UNKNOWN** — no test of any kind | **UNKNOWN** | No evidence anywhere |

---

## 3. Prioritized Bug Backlog

### Key: What Changed After PROMPT-2024

- `[NEW-2024]` = not present in prior revision; confirmed by PROMPT-2024 forensic pass
- `[UPDATED]` = prior finding revised by new evidence
- `[CLOSED]` = was open, now fixed (PROMPT 2020/2021)
- `[UNCHANGED]` = no new evidence; original finding stands

---

### P0 — Game-Ending / Completely Unplayable

---

#### P0-BUG-001 · Client Never Transitions to InSession `[NEW-2024 / CONFIRMED]`
- **Flow:** ALL game phases (automation scenario)
- **Subsystem:** Client networking / session state (`client/src/state/mod.rs`, `ClientState`)
- **Evidence:** PROMPT 2024 §4 — driver-timeline.jsonl for all three runs (262 ticks each, 3 runs = 786 total tick records) shows exactly one `(phase_label, client_state_label)` pair across the entire evidence base: `("Lobby", "Lobby")`. The `client_state_label` field is documented in `capabilities.json` as "ClientState machine state (Lobby or InSession)." It never reads "InSession."
- **Exact artifact path:** `production/qa/evidence/autoplay-runs/20260528-090613-Z/driver-timeline.jsonl` (selected run), all 262 ticks
- **Impact:** All checkpoints fire on driver timing, not on client phase transitions. Every screenshot labeled `class-select-loaded`, `auction-loaded`, `placement-loaded` etc. is taken while the client is still in Lobby state. The driver completes a "game" that the client never participates in.
- **Hypothesis for root cause (unconfirmed):** The `add-bot-lobby` recipe's Add Bot button click (FRAG-02) likely misses or the bot-soak server is not joined to the same room the client creates. Either the room creation fails silently or the server's bot invitation is not reaching the client session.
- **Minimal repro:** Run `vs-bot` recipe; inspect `driver-timeline.jsonl` — all entries will show `client_state_label: "Lobby"` while checkpoints advance.

---

#### P0-BUG-002 · Binary Protocol Mismatch (Lightyear Hash Panic) `[UNCHANGED / CONFIRMED]`
- **Flow:** ALL (human two-client scenario — blocks before any UI)
- **Subsystem:** Build / Lightyear networking
- **Evidence:** PROMPT 1883 — `client.exe` built 2026-05-21 (`3a4603af`, branch `play-main`); `server.exe` built 2026-05-28. 20+ Rust commits apart, including protocol-affecting PROMPT 1729 (UI interaction state wave 2). Both clients produce identical panic:
  ```
  lightyear::protocol::ProtocolCheckPlugin::receive_verify_protocol:
    the message protocol doesn't match
  ```
- **Exact artifact path:** `reports/PROMPT-1883-live-two-client-full-flow-retest-after-1872.md` §Blocker; `production/qa/evidence/captures/playable-client-real-e2e-loop/attempt-5-client-a.stderr.log` (Attempt 5 resolves this with fresh rebuild — no MissingComponent)
- **Impact:** Human player cannot reach any UI. Server starts cleanly; both clients crash within 1 second of connect.
- **Minimal repro:** Run `start-two-clients.bat` without rebuilding from current HEAD.
- **Fix:** `cargo build -p server && cargo build -p client --bin client` from same HEAD. See PROMPT 1706 Step 1.

---

#### P0-BUG-003 · No Successful Human GUI End-to-End Flow on Record `[UNCHANGED / CONFIRMED]`
- **Flow:** Lobby → Class Select → Draft → Auction → Placement → Resolution → Game Over
- **Subsystem:** QA coverage gap
- **Evidence:** `production/qa/evidence/captures/playable-client-real-e2e-loop/phase-captures.md` Attempt 5 states explicitly: "Production live logs still do not prove room creation, join, class confirmation, session entry, draft/shop, placement, resolution, next loop, or game-over." Controlled in-process tests (prompt-290/296/298 traces) prove server-side flow but not human-observable client UI.
- **Exact artifact path:** `phase-captures.md` §Attempt 5; `prompt-290-room-session-trace.json`, `prompt-298-auction-placement-resolution-trace.json`
- **Impact:** No flow has ever been manually verified. Any UI bug introduced since the controlled tests went undetected.
- **Minimal repro:** Attempt to play a complete game with two human players. This has never been done.
- **Fix:** Human operator runs PROMPT 1706 checklist L-01→R-04 with fresh binaries.

---

### P1 — Flow-Blocking or Major Player Experience Failure

---

#### P1-BUG-001 · Placement ACK Is Heuristic-Only (S2CPlacementAck Not Shipped) `[UNCHANGED / CONFIRMED]`
- **Flow:** Placement
- **Subsystem:** Server protocol / `server/src/feature/acquisition/` → client `client/src/ui/hand/mod.rs`
- **Evidence:** PROMPT 1937 GAP-7 — "Placement ACK is heuristic-only: S2CPlacementAck still not shipped." Open since PROMPT 1500 range. Confirmed not added in PROMPT 2019–2024 interval (no source changes to placement messages in intervening commits).
- **Exact artifact path:** `reports/PROMPT-1937-qa-snapshot-observability-gap-report-refresh-after-1931.md` §GAP-7
- **Impact:** Client infers placement acceptance from secondary signals. Player submitting a placement cannot reliably confirm server accepted it. Rejection recovery UX (PROMPT 1468) fires on heuristic, not explicit ACK.
- **Fix:** Add `S2CPlacementAck` message to shared protocol; emit from `process_placement_submission`; consume in client hand_ui.

---

#### P1-BUG-002 · Result Screen Does Not Display Win/Loss/Draw Outcome `[UNCHANGED / CONFIRMED]`
- **Flow:** Game Over
- **Subsystem:** Client UI — `client/src/presentation/result_screen.rs`
- **Evidence:** PROMPT 1937 GAP-1 + GAP-2 — "`ResultScreenViewState` still not read by snapshot system"; "local win/loss/draw not projected." Source inspection of `result_screen.rs` confirms `OUTCOME_ACCENT_VICTORY`/`OUTCOME_ACCENT_DEFEAT`/`OUTCOME_ACCENT_DRAW` color tokens are defined but the field projection from `S2CGameOver.reason` → visual outcome display is unverified in any snapshot or human test.
- **Exact artifact path:** `reports/PROMPT-1937-...` §GAP-1/2; `client/src/presentation/result_screen.rs` lines 22–30 (outcome accent palette)
- **Impact:** Human player reaches game-over screen but cannot tell who won. PROMPT 2015 fixed the 720px overflow scroll (BUG is visible at all sizes now) but the outcome data is still absent.
- **Fix:** Wire `S2CGameOver.reason` into `ResultScreenViewState`; add to snapshot read path; verify in result-screen harness.

---

#### P1-BUG-003 · disconnect_trackers Not Zeroed at Connection Time `[NEW-2024 / CONFIRMED from artifact]`
- **Flow:** ALL (server session management)
- **Subsystem:** Server — `server/src/core/rsm/state.rs` (`RoundState.disconnect_trackers`)
- **Evidence:** PROMPT 2024 §5 — bot-qa-snapshots `snapshot-0001-placement-phase-000003.json` shows `disconnect_trackers: [{player: 1, seconds_since_disconnect: 29991}, ...]` from the very first Placement phase (timestamp_ms=137923, i.e., ~138ms into the game). Value 29991 seconds ≈ 8.3 hours = time since server process start, never reset on connection. Present in all 10 bot-qa-snapshots throughout all rounds.
- **Exact artifact path:** `dev-runs/bot-qa-snapshots/snapshot-0001-placement-phase-...-000003.json` (main checkout)
- **Impact:** Server considers both players permanently disconnected from start. May suppress `S2CPhaseChanged` delivery to clients or trigger disconnect-grace timers prematurely. Likely contributes to P0-BUG-001 (client never InSession) if disconnect state prevents session-entry messages from being sent.
- **Fix:** Zero `disconnect_trackers[player_id].seconds_since_disconnect` when a player's `S2CHandshake` is sent / `snapshot_sent` registered. Inspect `server/src/core/session/reconnect.rs` for the initialization path.

---

#### P1-BUG-004 · Bot Always Uses empty_placement_failsafe — Zero Legal Placements `[NEW-2024 / CONFIRMED from artifact]`
- **Flow:** Placement → Resolution (all rounds)
- **Subsystem:** Server bot AI — `server/src/feature/bot/` (decision engine)
- **Evidence:** PROMPT 2024 §5 — all 10 bot-qa-snapshots covering Placement phases in rounds 1–2 show `decision: {kind: "empty_placement_failsafe"}` with `legal_action_count: 0`. Board `minion_count` stays 0 in every resolution snapshot. One soak run (`2026-05-27-222625`) shows `placement_submitted: placements_len=1` once, but bot-qa-snapshots (the canonical truth set) show failsafe throughout.
- **Exact artifact path:** `dev-runs/bot-qa-snapshots/snapshot-0001-placement-phase-...-000003.json`; `snapshot-0002-placement-phase-...-000007.json`
- **Impact:** Bot never places any units. Combat resolves empty-board every round. Player never sees meaningful combat. Game advances only through timeout/forfeit, not through actual gameplay.
- **Hypothesis for root cause:** PROMPT 2024 §9 GAP-04 notes: "snapshot shows `hand.size=1, cards=[5]` for player 1 but no hand entry for bot player in snapshot seq 3." The bot may not be receiving cards (no hand state) and therefore has no legal placements.
- **Fix:** Verify bot player receives `S2CDraftOffering` and `S2CCardAcquired` during DraftInitial. Inspect why bot player has no hand snapshot entry at Placement phase.

---

#### P1-BUG-005 · First 8 Bevy Screenshot Checkpoints Are a Frozen Static Frame `[NEW-2024 / CONFIRMED from artifact]`
- **Flow:** Lobby → Auction-loaded (automation)
- **Subsystem:** Autoplay screenshot infrastructure / Bevy render
- **Evidence:** PROMPT 2024 §3.1 — runs 1 and 2: ALL 15 Bevy screenshots identical byte-for-byte (86,080 B / 86,148 B). Run 3: screenshots 000000–000030 (8 captures, `lobby-loaded` through `auction-loaded`) are identical at 86,108 B; only from `auction-ready` (seq 37) do screenshots change to 117,843 B.
- **Exact artifact path:** `production/qa/evidence/autoplay-runs/20260528-090613-Z/screenshots/000000.png` through `000030.png` (identical), `000037.png` (changed)
- **Impact:** Diagnostic screenshots from the first 8 checkpoints (lobby, bot-add, lobby-confirm, class-select, class-confirm, shop-load, shop-click, auction-load) show no game state. These are the most critical diagnostic frames and they produce no usable information.
- **Note:** In the context of P0-BUG-001, this is expected: if the client never leaves Lobby, the Bevy render output is always the same Lobby frame.

---

#### P1-BUG-006 · Win32 PrintWindow Capture Freezes Mid-Run (Placement/Resolution) `[NEW-2024 / CONFIRMED from artifact]`
- **Flow:** Placement → Resolution (automation)
- **Subsystem:** Autoplay win32 capture backend — `tools/autoplay/win_capture.py`
- **Evidence:** PROMPT 2024 §3.3 — run 3 `driver.log` shows `win32_printwindow=FROZEN hash=ca2ab3e8456d5f81d2fc5f3f0c5703f2` at ticks 164, 176, 185, 250, 259 (all post-`placement-dragged`). BitBlt fallback was triggered and produced 11 distinct hashes — these ARE live captures.
- **Exact artifact path:** `production/qa/evidence/autoplay-runs/20260528-090613-Z/driver.log`; `bitblt_tick_000164.png` through `bitblt_tick_000259.png`
- **Impact:** Primary capture path (PrintWindow) fails during the phases most interesting for visual diagnosis. BitBlt fallback is authoritative for placement/resolution frames. The 11 bitblt PNGs have different pixel hashes and are the only live render content in the entire evidence base.
- **Fix:** Investigate why PrintWindow freezes after window height change (P1-BUG-007). BitBlt fallback is working; ensuring it fires sooner (or is the primary path) would recover diagnostic value.

---

#### P1-BUG-007 · Window Height Expands Unexpectedly Mid-Run (720 → 1076px) `[NEW-2024 / CONFIRMED from artifact]`
- **Flow:** All interactive phases (automation environment)
- **Subsystem:** OS window management / Bevy `Window` resource
- **Evidence:** PROMPT 2024 §3.3 — run 3 `driver.log` shows `width=1296 height=759` (OS chrome + 1280×720 logical) for ticks 5–113, then `width=1296 height=1115` from tick 138 onward. `status.json` final window = `1280×1076`. Bevy screenshot content changes byte count at the same point (86,108 B → 117,843 B). AC-VPT-01 (`enforce_autoplay_window_size_system`) enforces startup size but not mid-run resize.
- **Exact artifact path:** `production/qa/evidence/autoplay-runs/20260528-090613-Z/status.json`; `driver.log` ticks 113→138
- **Impact:** DWM snapping the window to a larger size mid-run invalidates all baked click coordinates (baked at 720px height). Clicks at fy=0.85/0.92 land at wrong positions in the expanded window. AC-VPT-02/08 drift guards (PROMPT 1880/1894) should abort the run when drift exceeds threshold — this may be why the run survives (bitblt shows live content) but clicks miss.

---

#### P1-BUG-008 · All 3 Autoplay Runs Return PARTIAL — No Clean PASS (Gate Blocked) `[UPDATED — now explained by P0-BUG-001]`
- **Flow:** All (AUTOPLAY-VS-BOT-QA-001 gate)
- **Subsystem:** Autoplay driver / client session establishment
- **Evidence:** PROMPT 1985 §2.1 — run `051148-Z` PARTIAL (no pixel_hash), run `063609-Z` PARTIAL (all hashes identical/frozen), run `090613-Z` PARTIAL (resize/frozen PrintWindow). PROMPT 2024 §4 confirms root cause: client never InSession → screenshots are all from the same Lobby frame → renderer never changes → hashes identical.
- **Exact artifact path:** `reports/PROMPT-1985-...` §2.1; `production/qa/evidence/autoplay-runs/20260528-090613-Z/launcher-status.json` (`live_pass_status: "NOT-CLAIMED"`)
- **Update vs prior revision:** Previously labeled as "no clean run" without root cause. PROMPT 2024 confirms the PARTIAL verdict is a downstream consequence of P0-BUG-001 (client never InSession). Fixing P0-BUG-001 is required before PASS can be achieved.

---

#### P1-BUG-009 · Placement Drag Coordinates at fy=0.92 — 58px From Bottom Edge (FRAG-01) `[UNCHANGED]`
- **Flow:** Placement (automation)
- **Subsystem:** Autoplay recipes — `tools/autoplay/recipes/_coords.py`
- **Evidence:** PROMPT 1848 FRAG-01 — `HAND_FIRST_CARD (0.35, 0.92)` and `SUBMIT_BTN (0.85, 0.92)` at y=662px in 720px window. PROMPT 2023 §Autoplay Fragility Register — FRAG-02/R-01 still open (fy fix not implemented).
- **Exact artifact path:** `tools/autoplay/recipes/_coords.py` (HAND_FIRST_CARD, SUBMIT_BTN defaults); `reports/PROMPT-2023-...` §Fragility Register
- **Fix:** Lower `HAND_FIRST_CARD` fy from 0.92 → 0.88; lower `SUBMIT_BTN` fy from 0.92 → 0.88 in `_coords.py`.

---

### P2 — Observability / QA Tooling / Lower Severity

All 14 QA snapshot gaps from PROMPT 1937 remain open except where noted.

#### P2-BUG-001 · Result Screen Outcome Not in QA Snapshot (GAP-1 + GAP-2) `[UNCHANGED]`
- **Subsystem:** `client/src/presentation/qa_snapshot.rs`, `result_screen.rs`
- **Evidence path:** `reports/PROMPT-1937-...` §GAP-1/2

#### P2-BUG-002 · Bot Debug Overlay Absent From Snapshot (GAP-3) `[UNCHANGED]`
- **Subsystem:** `client/src/presentation/debug_bot_overlay.rs`, `qa_snapshot.rs`
- **Evidence path:** `reports/PROMPT-1937-...` §GAP-3

#### P2-BUG-003 · Card Class/Rarity Not in Shop/Draft Slot Snapshot (GAP-4) `[UNCHANGED]`
- **Subsystem:** `client/src/ui/shop_auction/mod.rs`, `qa_snapshot.rs`
- **Evidence path:** `reports/PROMPT-1937-...` §GAP-4

#### P2-BUG-004 · Auctioned Card Class/Rarity Not in Auction State Snapshot (GAP-5) `[UNCHANGED]`
- **Subsystem:** `client/src/ui/shop_auction/mod.rs`, `qa_snapshot.rs`
- **Evidence path:** `reports/PROMPT-1937-...` §GAP-5

#### P2-BUG-005 · Board Unit Class ID Absent From Snapshot (GAP-6) `[UNCHANGED]`
- **Subsystem:** `client/src/presentation/board_rendering.rs`, `qa_snapshot.rs`
- **Evidence path:** `reports/PROMPT-1937-...` §GAP-6

#### P2-BUG-006 · Frozen Renderer Not Detectable From Snapshot JSON Alone (GAP-13) `[UNCHANGED]`
- **Subsystem:** `client/src/presentation/qa_snapshot.rs` (`ScreenshotInfo`)
- **Evidence path:** `reports/PROMPT-1937-...` §GAP-13

#### P2-BUG-007 · Window Resize Events Not Tracked in Snapshot (GAP-12) `[UNCHANGED — partially mitigated]`
- **Subsystem:** `client/src/autoplay.rs`, `qa_snapshot.rs` (`WindowInfo`)
- **Mitigation on main:** `enforce_autoplay_window_size_system` (PROMPT 1912) enforces 1280×720 startup. Mid-run resize still untracked.
- **Evidence path:** `reports/PROMPT-1937-...` §GAP-12

#### P2-BUG-008 · No Playtest Evidence for Fun Hypothesis (QA-COND-0006) `[UNCHANGED — accepted risk]`
- **Subsystem:** Production / QA
- **Evidence path:** `production/qa/bugs/QA-COND-0006-playtest-fun-hypothesis-evidence.md`
- **Status:** Accepted Risk (producer decision 2026-05-05). No playtest sessions ever conducted.

#### P2-BUG-009 · `viewport_shrink_guard.py` Not Imported by driver.py `[UNCHANGED]`
- **Subsystem:** `tools/autoplay/driver.py`, `tools/autoplay/viewport_shrink_guard.py`
- **Evidence path:** `reports/PROMPT-2018-...` §2.2
- **Note:** Driver has inlined equivalent guards (AC-VPT-02/08). Module is tested (31 tests PASS) but not integrated.

#### P2-BUG-010 · 101 Compiler Warnings (Deprecated Marker Components) `[NEW-2024]`
- **Subsystem:** `client/src/presentation/qa_snapshot.rs`, `client/src/ui/shop_auction/mod.rs`
- **Evidence path:** PROMPT 2024 §3 — `process.log.err` all 3 runs: 101 warnings about deprecated `HudEntity`, `HandUiEntity`, `ShopAuctionUiEntity` markers
- **Impact:** Technical debt; coarse-grained snapshot entity counts may misreport.

---

## 4. Closed / Superseded Items (Since Prior Revision)

| Prior item | Status | Resolution |
|---|---|---|
| BUG-009 (No phase-label gating / FRAG-03) | **CLOSED** | PROMPT 2020 — `poll_phase()` pseudo-action shipped; 39 new tests PASS; recipes not yet updated to use it but framework exists |
| R-04 (Add Bot coord documentation gap) | **CLOSED** | PROMPT 2021 — measurement protocol in `docs/autoplay.md` and `evidence-operator-guide.md` |
| FRAG-03 listed as OPEN | **CLOSED** | Same as BUG-009 above |

---

## 5. Human vs. Automation Separation (Deduplicated)

| Bug ID | Human Player | Automation | Both | P-Level |
|---|---|---|---|---|
| P0-BUG-001 Client never InSession | — | ✓ | — | P0 |
| P0-BUG-002 Binary protocol panic | ✓ | — | — | P0 |
| P0-BUG-003 No live GUI test | ✓ (risk) | — | ✓ | P0 |
| P1-BUG-001 PlacementAck heuristic | ✓ | — | — | P1 |
| P1-BUG-002 Result screen no outcome | ✓ | — | — | P1 |
| P1-BUG-003 disconnect_trackers stale | — | — | ✓ | P1 |
| P1-BUG-004 Bot zero placements | — | — | ✓ | P1 |
| P1-BUG-005 Screenshots frozen (lobby) | — | ✓ | — | P1 |
| P1-BUG-006 PrintWindow freezes | — | ✓ | — | P1 |
| P1-BUG-007 Window expands mid-run | — | ✓ | — | P1 |
| P1-BUG-008 All 3 runs PARTIAL | — | ✓ | — | P1 |
| P1-BUG-009 fy=0.92 coords fragile | — | ✓ | — | P1 |
| P2-BUG-001..010 | — | ✓ | — | P2 |

**Human-player blockers (unblocking order):**
1. P0-BUG-002 → rebuild binaries
2. P0-BUG-003 → run manual two-client test
3. P1-BUG-001 → ship S2CPlacementAck
4. P1-BUG-002 → wire result screen outcome
5. P1-BUG-003 → fix disconnect_tracker init

**Automation blockers (unblocking order):**
1. P0-BUG-001 → diagnose why client never reaches InSession (likely FRAG-02 Add Bot miss + bot-soak server not joining client room)
2. P1-BUG-003 → fix disconnect_tracker init (may be same root cause as P0-BUG-001)
3. P1-BUG-004 → fix bot hand/card receipt so bot makes legal placements
4. P1-BUG-007 → prevent DWM mid-run window expand
5. P1-BUG-009 → lower fy=0.92 coords to 0.88

---

## 6. Recommended Next Workers (Priority Order)

### Immediate — Root-Cause P0-BUG-001

**Dispatch:** One source-reading worker to inspect:
- `server/src/core/session/reconnect.rs` — where is `seconds_since_disconnect` initialized? Should reset to 0 on `snapshot_sent`/`S2CHandshake` path.
- `client/src/state/mod.rs` — what `S2C` message triggers `ClientState::Lobby → InSession`? Does the `vs-bot` recipe flow actually send the required server message?
- `client/src/ui/lobby.rs` — does the Add Bot button render at fy=0.72 when `CCGS_DEBUG_UI=1`? Cross-reference with PROMPT 2021 measurement protocol.

**Purpose:** Determine whether P0-BUG-001 and P1-BUG-003 share a root cause (disconnect_tracker preventing phase messages) or are independent (bot-soak server not joining the client's room).

### Second — Rebuild and Manual Test

**Human operator action (not dispatchable):**
```powershell
$env:CARGO_TARGET_DIR = "D:\_DEV\cargo-target\ccgs-msvc"
cargo build -p server
cargo build -p client --bin client
# then run PROMPT 1706 checklist L-01→R-04
```

### Third — Fix Bot Hand Receipt (P1-BUG-004)

**Dispatch:** Worker to inspect why bot player has no hand entry in `snapshot-0001-placement-phase-...-000003.json`. Verify bot receives `S2CDraftOffering` and `S2CCardAcquired`.

### Fourth — P1-BUG-001 (PlacementAck) and P1-BUG-002 (Result Screen Outcome)

These are independent of P0 fixes and can be addressed in parallel once a source-edit worker is available.

---

## 7. Validation

### Path allowlist check

Only `reports/PROMPT-2028-player-flow-unplayable-bug-classification.md` written or modified.
Zero source files, zero `production/session-state/**`, zero `production/sprints/**`,
zero `production/qa/**` evidence files, zero `Cargo.*` touched.

### git diff --check

Report file only. Text, no trailing whitespace.

---

2028: PLAYER-FLOW-UNPLAYABLE-BUG-CLASSIFICATION: COMPLETE
