# PROMPT 2029 — QA Evidence Tools Truthfulness Audit

**Date:** 2026-05-28
**Branch:** work/PROMPT-2029
**Source tree:** origin/main@e7b51e84 (post-PROMPT-2019)
**Scope:** Read-only audit — no source edits.
**Task:** Identify how previous automation could mark runs PASS/SHIPPED while visible gameplay remains broken. Produce concrete QA/tooling fixes.

---

## Executive Summary

**No automated PASS has ever been established for any gameplay recipe.** The three bot-game runs on record (all from 2026-05-28, per PROMPT-1985 §2.1) all returned `PARTIAL`. Every SHIPPED/PASS label in the `reports/` directory is a report about the QA *tooling chain* itself — not evidence of a working, playable game.

The fundamental gap: the toolchain verifies RPC substrate reachability, screenshot file creation, and window non-minimization. It does not verify that correct game UI is rendered, that player actions produce observable state changes, or that game phases advance as expected. A completely black screen, a static loading spinner, or a crash dialog all pass the current automated validator.

---

## Section 1 — False Positive Mechanisms

### FP-1: Screenshots are captured but never analyzed for visual content

**Files:** `tools/autoplay/driver.py` (full flow), `tools/autoplay/analyze_evidence_run.py:272–358`, `tools/autoplay/validate_composite_run.py:128,461–533`

Screenshots are taken via `autoplay/screenshot` RPC and fallback `win32_printwindow` / `desktop_bitblt`. The analyzer's PASS verdict (`_compute_verdict`, `analyze_evidence_run.py:272`) requires:

- `driver_exit_code` is 0
- At least 1 screenshot captured
- No frozen-pixel detection pattern

The **only content check** is mean brightness > 15/255 (`NEAR_BLACK_BRIGHTNESS_THRESHOLD = 15`, `validate_composite_run.py:128`). This passes for any non-black image including a gray loading screen, a white dialog, a partially composited render, or a frozen DWM frame of a desktop icon.

**No check exists for:** correct UI visible, expected text/elements present, correct color regions, expected card layout, HUD elements, or phase-appropriate screen content.

---

### FP-2: Checkpoints are time-based, not state-based

**Files:** `tools/autoplay/recipes/_builder.py:148–164`, `tools/autoplay/recipes/resolution_observe.py:59–63`, `tools/autoplay/recipes/game_over_observe.py:76–82`, all recipe files

Every checkpoint is emitted by the driver after a fixed number of `wait(N)` ticks via the recipe builder. A checkpoint labeled `"placement-submitted"` only means **60 ticks elapsed** after the submit click was dispatched. It does not verify:

- The placement UI was visible when the click fired
- The click registered in the game (not fired into blank space)
- The server accepted the placement
- The game advanced to the next phase

The `autoplay/status` response includes `phase_label` and `client_state_label` fields (confirmed in timeline data), but **zero recipes use these for phase gating** — all phase transitions are assumed by time delay alone.

---

### FP-3: Click coordinates are static fractional guesses; misses are undetectable

**Files:** `tools/autoplay/recipes/_coords.py:34–47`, `tools/autoplay/recipes/placement_drag_probe.py:39–50`

All coordinate defaults are static fractions:

```python
"HAND_FIRST_CARD":   FracPoint(0.35, 0.92),   # y = 662px at 720p
"SUBMIT_BTN":        FracPoint(0.85, 0.92),   # near bottom-right corner
"LOBBY_ADD_BOT_BTN": FracPoint(0.5, 0.72),   # debug button, no guard
```

The `autoplay/status` API does not expose element geometry. If any UI element is not at its expected fractional position — due to layout changes, different resolution, window resize, or Bevy layout recalculation — the click fires into blank space. The recipe has no detection mechanism for a missed click. The run proceeds, all checkpoints pass on schedule, and the verdict is PASS or PARTIAL.

The PROMPT-1848 coverage report explicitly rates `placement-drag-probe` as **CRITICAL** fragility and names hand-strip and submit button as the highest-risk coordinates (§3 FRAG-01).

---

### FP-4: The `two-client-runtime` harness uses `MinimalPlugins` — no rendering, no UI

**Files:** `tools/two-client-runtime/src/main.rs` (comment block at lines 635–641), `tools/two-client-runtime/src/bot_route.rs`, `tools/two-client-runtime/src/bot_soak.rs`

The two-client-runtime test harness runs with `MinimalPlugins` — **no rendering, no windowing, no bevy_ui**. Client apps execute `App::update()` tick loops only, sending real C2S messages and receiving real S2C messages over production WebSocket transport.

A PASS from `two-client-runtime` validates the **server-side state machine and message protocol only**. The Bevy rendering pipeline, UI layout, sprite sheets, hand fan, HUD, resolution animations — none of these are exercised. A game that is completely visually broken still exits 0 with `endpoint_reached: "game_over"` in `final_state.json`.

---

### FP-5: Bot path submits empty placements — placement UI is never exercised

**Files:** `tools/two-client-runtime/src/route.rs:443–451`, `tools/two-client-runtime/src/bot_soak.rs`

```rust
// route.rs:443-451
fn scripted_placement(role: ClientRole, _state: &RouteState) -> Vec<PlacedCardSubmit> {
    // empty placement is valid — accepted by C2SSubmitPlacement
    let _ = role;
    Vec::new()
}
```

The friend-game harness always sends empty placements. The bot-soak trigger attempts a real placement only for the first card of Round 1 when a card was purchased at DraftInitial. All subsequent rounds use empty placements. **A PASS from either harness does not validate card placement, drag-and-drop, placement phase UI, or that the placement board is usable.**

---

### FP-6: `NEEDS_HUMAN_GUI` silently absorbs visual failures without blocking reports

**Files:** `tools/autoplay/validate_composite_run.py:200–209`, `tools/autoplay/analyze_evidence_run.py:272–311`

When window integrity checks fire (resize, too-small height, all-frozen captures), the verdict is downgraded to `NEEDS_HUMAN_GUI` rather than hard `FAIL`. This:

1. Does not count as FAIL in any automated gate (CI gate only blocks on exit code 2)
2. Is satisfied by noting `live_pass_status: "NOT-CLAIMED"` in the composite summary
3. Creates a narrative of "pending human review" — but the human review never happens

From PROMPT-1985 §2.2: all three available bot-soak runs produced PARTIAL/NEEDS_HUMAN_GUI verdicts. Yet downstream report authors cite these runs as supporting evidence for SHIPPED stories. The escape valve is structurally designed to never block progress.

---

### FP-7: Frozen-frame detection is a warning and fallback — not a hard abort

**Files:** `tools/autoplay/driver.py:515–529`, `tools/autoplay/analyze_evidence_run.py:303–310`, `tools/autoplay/win_capture.py:282–325`

The `_frozen_win32_check` in `driver.py:88–113` triggers the `desktop_bitblt` fallback when PrintWindow captures are byte-identical. It does **not abort the run**. A run where every PrintWindow capture is frozen is classified `NEEDS_HUMAN_GUI`, not `FAIL`.

The bitblt fallback itself only checks that pixels are non-zero. A static frozen desktop screenshot (showing whatever was behind the game window) passes this check. The PROMPT-1985 run `20260528-063609-Z` had all 15 hashes identical (frozen renderer) — this is the second-worst possible evidence of a working game, but it still produces a PARTIAL instead of FAIL.

---

### FP-8: The smoke recipe proves RPC substrate, not gameplay

**Files:** `tools/autoplay/recipes/smoke.py:14–25`

The smoke recipe sends one `KeyA` press and one `Left` mouse click to window center, waits one tick, takes a screenshot. Its explicit purpose: "Proves the RPC substrate." A smoke PASS means:

- The RPC endpoint responded
- The screenshot file was written (non-zero bytes)

A completely broken game with a visible crash dialog, black screen, or stuck loading state **passes smoke** as long as the RPC server is reachable.

---

### FP-9: Window foreground confirmation falls through silently

**Files:** `tools/autoplay/win_foreground.py:308–312`, `tools/autoplay/driver.py:474–505`

`ensure_foreground()` attempts to bring the Bevy window to the front before each screenshot. If all foreground attempts fail (e.g., another fullscreen window is blocking), the code falls through with a logged warning and captures the last DWM-composited frame. This is not an abort condition. The DWM frame may be stale — showing whatever the game displayed when it was last visible.

---

### FP-10: Mass report generation hides the signal-to-noise collapse

**Observation from `reports/` directory (PROMPT-1900 through PROMPT-2007):**

The `reports/` directory contains 50+ PROMPT reports in the PROMPT-1900–2007 range. Spot-checking reveals these are almost exclusively:
- Re-applications of the same report content onto successive `main` branches after NOT_FF rejections
- Reports *about* the tooling (window-resize verdict, operator contract, visible-target coverage map)
- Story readiness reports that document the absence of PASS evidence

**The volume of reports creates an appearance of progress** — each new PROMPT number suggests a new milestone. In practice, PROMPT-1979/1994 (window-resize verdict downgrade), PROMPT-1985 (story readiness), and PROMPT-1995 (visible-target coverage) are all documentation of known gaps, not evidence of fixes. An orchestrator reading the commit log sees 50+ `docs(reports)` commits since PROMPT-1900 and reasonably infers the game is in a stable evidence-collection phase, when the actual evidence base has not changed since the three PARTIAL runs on 2026-05-28.

---

## Section 2 — Evidence Quality Gaps

### EQG-1: No pixel-region assertions exist anywhere in the toolchain

Neither `validate_composite_run.py` nor `analyze_evidence_run.py` nor the driver contain any pixel-region or template-matching assertion. The only check is `NEAR_BLACK_BRIGHTNESS_THRESHOLD = 15` (mean brightness across the entire screenshot). This does not detect:

- UI elements missing from expected screen regions
- Wrong game screen being displayed (lobby when placement expected)
- Invisible cards/sprites (transparent, wrong Z-order, off-screen, wrong render layer)
- HUD elements absent
- Cards or text rendered in wrong positions

**There is no way for the current toolchain to detect that the game looks wrong.**

---

### EQG-2: `phase_label` is available in every tick but never used for gating

**Files:** `tools/autoplay/driver.py` (status polling loop), `tools/autoplay/recipes/_builder.py:149–164`

The status payload includes `phase_label` and `client_state_label` at every tick. The driver logs these in `driver-timeline.jsonl`. Zero recipes poll them before firing clicks. Every click fires based on a time counter regardless of whether the correct game phase is active.

If a phase transition is slow (network jitter, server load, mid-render), a click lands on the wrong overlay. If the game is stuck in a previous phase, the click fires into the wrong UI state. The driver has no way to know and no way to recover.

---

### EQG-3: Screenshot settle time is 300ms — insufficient for animation states

**Files:** `tools/autoplay/recipes/_builder.py:156–164`

```python
# _builder.py:156-164
self._emit("local.checkpoint", ...)
if screenshot:
    if settle_ticks > 0:
        self._next(settle_ticks)  # default 3 ticks = 300ms at 10Hz
    self._emit("autoplay/screenshot", ...)
```

At 10 Hz, 3 settle ticks = 300ms. Bevy tweening transitions and server message round-trips can easily exceed this. A screenshot labeled `"placement-loaded"` may capture the auction overlay if the server responded faster than expected, or still show the prior phase if the server is slow.

---

### EQG-4: The three existing runs are all permanently PARTIAL — no clean baseline exists

From PROMPT-1985 §2.1 (unchanged as of PROMPT-2019):

| Run | Window size | Verdict | Automated PASS? |
|-----|-------------|---------|-----------------|
| `20260528-051148-Z` | `[1280,720]` stable | PARTIAL — no capture labels, no pixel_hash | **NO** |
| `20260528-063609-Z` | `[1280,720]` stable | PARTIAL — all 15 hashes identical (frozen renderer) | **NO** |
| `20260528-090613-Z` | `[1280,720]→[1280,1076]` mid-run | PARTIAL — 11/15 PrintWindow frozen; click coords baked at 720-height | **NO** (human-review conditional only) |

**No clean automated PASS has ever been recorded.** All story readiness reports citing these runs do so with explicit caveats. Any statement in any SHIPPED story that implies an automated PASS is factually incorrect.

---

### EQG-5: No evidence that `autoplay/screenshot` captures game viewport vs. black surface

**Files:** `tools/autoplay/win_capture.py:374–408`, `tools/autoplay/driver.py:585–596`

The Bevy `autoplay/screenshot` RPC uses `Screenshot::primary_window()`. When the window is not composited (background, minimized, DWM throttled), this may return a near-black PNG. The brightness threshold (15/255) catches fully-black but not:

- Dimly-rendered or partially-composited surfaces
- DWM-frozen frames showing stale game state
- Correct window handle but wrong content (e.g., window dragged off-screen)

---

### EQG-6: The `idle` recipe trivially passes all validators

**Files:** `tools/autoplay/recipes/idle.py:8–10`, `tools/autoplay/validate_composite_run.py:77–78`

The idle recipe returns an empty action list. `RECIPE_REQUIRED_CHECKPOINTS["idle"] = []`. A run using recipe=idle that stays connected for the timeout duration exits 0, produces zero checkpoints, takes zero screenshots, and passes all validator checks. Any report citing an idle-recipe run as positive game-state evidence is meaningless.

---

## Section 3 — Required Validator Upgrades

Listed in priority order:

### VU-1 (CRITICAL): Phase-label gating in recipes

Add `poll_phase(expected_label, max_ticks)` primitive to `_builder.py`. The driver handles `local.poll_phase` by re-querying `autoplay/status` until `phase_label` matches or the cap is reached (emitting `local.block` on timeout). Without this, every click fires blind.

**Acceptance criteria:** A recipe that clicks "Submit" during the wrong phase (e.g., auction overlay visible) must emit `local.block` and set exit code 4, not proceed with a timed checkpoint.

---

### VU-2 (CRITICAL): Minimum pixel-region content assertion

Add `assert_region(x_frac, y_frac, w_frac, h_frac, min_variance, label)` to the recipe builder and driver. At minimum, verify:
- Hand strip region (bottom 15% of screen, `y > 0.85`) has pixel variance > threshold — indicates cards are rendered
- Board region (center `x: 0.1–0.9, y: 0.3–0.7`) has non-trivial variance — indicates board grid is visible
- HUD region (top `y < 0.15`) has non-trivial variance — indicates HUD is rendered

This is the single highest-value check given the reported symptom (game visually unplayable).

**Acceptance criteria:** A run where the hand strip region is solid-color (no cards rendered) must produce exit code 2 (FAIL), not PARTIAL.

---

### VU-3 (HIGH): Screenshot content diff between phases

Extend `validate_composite_run.py` to require a minimum perceptual hash distance between screenshots from different game-phase checkpoints (e.g., `lobby-joined` vs `placement-loaded`). Byte-identical detection already exists (`IDENTICAL-SCREENSHOTS`); extend it to phase-pair distance.

**Acceptance criteria:** A run where lobby and placement phase screenshots are identical (game stuck) must produce `WINDOW-PHASE-FROZEN` tag and verdict FAIL.

---

### VU-4 (HIGH): `phase_label` recorded per checkpoint row

The driver already has full status per tick. Add `phase_label` and `client_state_label` to each row in `checkpoints.jsonl`. The post-run validator checks that `phase_label` at checkpoint time matches the recipe's declared expected phase. Zero new RPC calls needed.

**Acceptance criteria:** Validator rejects a run where `phase_label` at the `"placement-submitted"` checkpoint was `"DraftAuction"` (wrong phase).

---

### VU-5 (MEDIUM): Hard FAIL on `NEEDS_HUMAN_GUI` in CI gate

`analyze_evidence_run.py` exit code 3 = NEEDS_HUMAN_GUI. Current CI gate only blocks on exit code 2 (FAIL). Treat exit code 3 as a blocking failure in all story gates. A story that cannot produce a clean automated run is not Done.

**Acceptance criteria:** Any story whose supporting evidence run produces NEEDS_HUMAN_GUI (exit 3) is automatically moved to BLOCKED in the sprint board.

---

### VU-6 (MEDIUM): Placement drag acceptance verification

Add a post-drag region check: after `placement_drag_probe`, assert that the board cell at `(fx=0.5, fy=0.55)` shows a non-background-color pixel cluster. A card-sized region of uniform background color after a drag indicates the drag missed.

---

### VU-7 (LOW): `phase_label` mismatch → emit `local.block` at runtime

The driver should emit `local.block` if the observed `phase_label` at action time does not match the recipe's declared expected phase (when declared). This provides runtime protection against clock-skew misalignment in addition to the post-run validator check (VU-4).

---

## Section 4 — Immediate Stop-The-Line Checks

These can be verified in under 5 minutes and will immediately reveal whether the game is actually rendering. **Before any story can be marked Done or any new PASS is accepted, ALL of these must pass:**

### STL-1: Take a live screenshot and inspect it manually

```powershell
cd tools/autoplay
python rpc.py screenshot
# Open the output PNG — does it show a recognizable game UI?
```

**Expected:** Visible game screen with cards, HUD, or lobby.
**Red flag:** Black screen, gray window, desktop background, loading spinner frozen in place.

---

### STL-2: Poll `phase_label` and verify game is in an expected state

```powershell
python rpc.py status
# Inspect phase_label field
```

**Expected:** `"Lobby"`, `"DraftAuction"`, `"Placement"`, or similar game phase.
**Red flag:** `null`, empty, or a startup state after more than 30 seconds of running.

---

### STL-3: Inspect `driver-timeline.jsonl` from the most recent run for phase_label progression

Open `production/qa/evidence/autoplay-runs/<most-recent>/driver-timeline.jsonl`. Check whether `phase_label` changes across ticks. If all ticks show the same `phase_label` despite the recipe advancing through lobby → class → placement, the game never transitioned — clicks did nothing.

**Expected:** `phase_label` values change as the recipe progresses through phases.
**Red flag:** All ticks show the same label (game stuck).

---

### STL-4: Run `analyze_evidence_run.py` on the most recent evidence directory

```powershell
python tools/autoplay/analyze_evidence_run.py <path-to-latest-run-dir>
# Check exit code and verdict field
```

**Expected:** Exit code 0 (PASS) with verdict `PASS`.
**Red flag:** Exit code 1 (PARTIAL), 2 (FAIL), or 3 (NEEDS_HUMAN_GUI).

If the result is anything other than clean PASS, **no story should be accepted as Done based on automated QA.**

---

### STL-5: Verify `two-client-runtime` PASS does NOT imply visual correctness

Any story whose acceptance criteria include a `two-client-runtime` PASS must have a separate visual evidence item (screenshot + human sign-off). The `two-client-runtime` binary uses `MinimalPlugins` — it has no rendering pipeline, no bevy_ui, no window. A PASS from this tool proves server-side protocol correctness only.

**Gate addition required:** Any story ticket citing `two-client-runtime` output as the primary evidence must be flagged as incomplete until a separate visual evidence item is attached.

---

### STL-6: Verify `CCGS_DEBUG_UI=1` and `CCGS_AUTOPLAY_BOT_ROOM_READY=1` are set before `vs-bot` runs

Without both env vars, the `vs-bot` recipe emits `local.block` immediately and exits 4 (blocked). The composite-summary records this as `blocked-*` and skips checkpoint validation. A run launched without these variables produces zero meaningful evidence but may look like a launched run in the session log.

---

## Section 5 — Root Cause Summary

The QA chain was built incrementally to solve **instrumentation problems** (capture quality, window foreground, frozen frames, window resize). Each PROMPT in the 1830–2007 range addressed a specific tooling weakness. The result is a sophisticated infrastructure for reliably capturing screenshots and detecting whether the capture pipeline itself is working.

**The missing layer is semantic validation**: does the captured screenshot show the correct game state?

| Layer | Current status |
|-------|---------------|
| RPC substrate reachable | ✅ Verified by smoke recipe |
| Screenshot file created | ✅ Verified by driver |
| Window non-minimized | ✅ Verified by win_foreground.py |
| Window non-frozen (DWM) | ✅ Verified by frozen-frame detection + bitblt fallback |
| Window-resize detected | ✅ Added in PROMPT-1979/1994 |
| **Screenshot shows correct game state** | ❌ Not implemented |
| **Clicks landed on correct UI elements** | ❌ Not detectable |
| **Game phase advanced after actions** | ❌ Not verified (only timed) |
| **Cards visually rendered in hand** | ❌ Not verified |
| **HUD showing correct values** | ❌ Not verified |
| **Placement drag accepted by game** | ❌ Not verified |

The 50+ SHIPPED/PASS reports in `reports/PROMPT-19xx` through `reports/PROMPT-20xx` are reports about the QA *tooling* readiness and chain re-applications onto successive main branches. They are not evidence that the game is playable. The game's visual playability state is entirely unknown from the automated toolchain.

---

## Appendix — File References

| File | Relevance |
|------|-----------|
| `tools/autoplay/analyze_evidence_run.py:272–358` | PASS verdict computation — no content check |
| `tools/autoplay/validate_composite_run.py:128` | Only brightness check (threshold=15) |
| `tools/autoplay/recipes/_builder.py:148–164` | Time-based checkpoint emission |
| `tools/autoplay/recipes/_coords.py:34–47` | Static fractional click targets |
| `tools/autoplay/recipes/smoke.py:14–25` | Smoke proves RPC only |
| `tools/two-client-runtime/src/main.rs:635–641` | MinimalPlugins — no rendering |
| `tools/two-client-runtime/src/route.rs:443–451` | Empty placement bot path |
| `tools/autoplay/win_foreground.py:308–312` | Silent foreground fallthrough |
| `reports/PROMPT-1985-bot-autoplay-story-readiness-report-refresh-after-1976.md:52–55` | All three runs are PARTIAL |
| `reports/PROMPT-1994-autoplay-composite-window-resize-verdict-refresh-after-1991.md` | Window-resize verdict scope (tooling only) |

---

---

## Section 6 — PASS/SHIPPED Claims vs. Raw Artifact Comparison (Phase 2 — Post-PROMPT-2024)

This section directly compares specific PASS/SHIPPED labels to their underlying artifact content.

---

### Claim 6-A: S8-QA-001 "PASS WITH WARNINGS" — Sprint 8 manual friend-game smoke

**Artifact:** `production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-manual-smoke-summary.json`
**Date stamped:** 2026-05-07
**Claimed verdict:** `"PASS WITH WARNINGS"`

**What the raw artifact actually says:**

```json
"attempted_full_native_or_browser_two_client_run": false,
"blocker": "The Codex shell session can run commands and tests, but it cannot drive
  two interactive Bevy native client windows..."
```

Every game-phase checklist item is labeled `"PASS via controlled real-Lightyear trace"`:

| Checklist item | Raw label | What "controlled real-Lightyear trace" means |
|---|---|---|
| `lobby_create_join` | PASS via trace | In-process Rust test, no window, no UI |
| `class_confirm` | PASS via trace | In-process Rust test, no window, no UI |
| `draft_initial` | PASS via trace | In-process Rust test, no window, no UI |
| `draft_shop` | PASS via trace | In-process Rust test, no window, no UI |
| `auction` | PASS via trace | In-process Rust test, no window, no UI |
| `non_empty_placement` | PASS via trace | In-process Rust test, no window, no UI |
| `resolution_unit_placed` | PASS via trace | In-process Rust test, no window, no UI |

`server_log`, `client_a_log`, `client_b_log` are all labeled `WARN` — no real client logs were captured.

**Misleading element:** The top-level verdict `"PASS WITH WARNINGS"` implies a smoke run was conducted and passed, with minor warnings. In reality, the GUI was never launched. The PASS comes entirely from headless in-process Rust tests that have no rendering pipeline. A broken UI, invisible cards, or a crash-on-first-render would all still produce this PASS verdict.

**Correct label:** `BLOCKED — no manual GUI execution; in-process protocol tests only`

---

### Claim 6-B: S9-QA-001 "No product defects found" — Sprint 9 manual game-over

**Artifact:** `production/qa/evidence/captures/sprint-9-manual-game-over/defects.md`
**Artifact:** `production/qa/evidence/captures/sprint-9-manual-game-over/route-summary.json`
**Date stamped:** 2026-05-08

**What the raw artifacts say:**

From `route-summary.json`:
- `"route_status": "blocked"`
- `"last_reached_step": "server_startup_clean"`
- `"client_a_launch": "not_reached"` — client A was never launched
- `"client_b_launch": "not_reached"` — client B was never launched
- ALL game phases: `"not_reached"` (room_create_join, class_confirm, draft_initial, draft_shop, draft_auction, placement, resolution, game_over, result_screen, return_to_lobby_ack)

From `defects.md`:
> "Product Defects Found: **None**. No product defects were encountered during this run because the client GUI was not reached."

**Misleading element:** "No product defects found" is literally true but structurally inverted. It is stated in a format that reads as a clean product verification. The actual meaning is: "we never ran any product code that could produce a defect." Any story gate that reads `defects.md` and finds "None" will record the story as green.

**Correct label:** `NOT RUN — AI agent cannot operate Bevy windowed clients; zero product paths exercised`

---

### Claim 6-C: All UI screenshot evidence is from Chrome harnesses, not the game

**Artifacts examined:**
- `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/capture-summary.json` — `captureTool: "PowerShell Chrome DevTools Protocol"`, URL: `http://127.0.0.1:8082/shop-auction-bid-target-focus-harness.html`
- `production/qa/evidence/captures/shop-auction-ui-draft-initial-clear-objective-overlay/capture-summary.json` — same harness origin
- `production/qa/evidence/captures/hud-011-mana-shapes/capture-summary.json` — same harness origin
- `production/qa/evidence/captures/hud-012-text-size-contrast/hud-012-text-size-contrast-capture-summary.json` — same harness origin
- `production/qa/evidence/captures/qa-cond-0007-hand-ui/qa-cond-0007-hand-ui-trace.json` — `captureTool: "PowerShell Chrome DevTools Protocol + Trunk WASM harness"`, URL: `http://127.0.0.1:8083/`
- `production/qa/evidence/captures/hand-ui-placement-staged-disclosure/hand-ui-placement-staged-disclosure-trace.json` — harness origin
- `production/qa/evidence/captures/qa-cond-0007-resolution-replay/qa-cond-0007-resolution-replay-trace.json` — harness origin

**Classification result:** Every set of UI screenshots in the entire `production/qa/evidence/captures/` directory was taken by Chrome DevTools Protocol against a local HTML harness page. These harnesses render UI components in isolation — they are not the running Bevy WASM client or native client connected to a live server.

**What harness captures do validate:** Individual widget states (button enabled/disabled, focus ring visible, layout bounds at specific viewport sizes).

**What harness captures do NOT validate:**
- That the game client actually loads these widgets
- That the widgets are visible during a real game session
- That the widgets render correctly in the Bevy render pipeline (not just in a Chrome HTML document)
- That game data (card counts, gold, phase label) is correctly bound to these widgets
- That two connected clients see the same state

**Misleading element:** Reports citing these captures as evidence of a working game UI imply that the described screens are visible to a human playing the game. In reality they prove only that an isolated HTML widget renders correctly in Chrome 147. The Bevy client could fail to render any of these elements and these captures would still pass.

---

### Claim 6-D: Identical PNG byte counts in `sau-011` focus scenarios

**Artifacts:** `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/`

File sizes from `Get-ChildItem` output:
```
sau-011-affordable-1366x768.png    19390 bytes
sau-011-focus-plus-1-1366x768.png  19390 bytes
sau-011-focus-plus-3-1366x768.png  19390 bytes
sau-011-focus-plus-5-1366x768.png  19390 bytes
```

The `capture-summary.json` confirms these are four distinct scenarios: affordable (no button focused), focus-plus-1 (button 1 focused), focus-plus-3 (button 3 focused), focus-plus-5 (button 5 focused). Each scenario should have a visually distinct state — one button with a 2px focus ring and enlarged border, others without.

Yet all four produce identical 19390-byte PNG files.

**Explanation:** The Chrome DevTools Protocol screenshot capture is likely being triggered before the harness JavaScript has fully rendered the focused state. The result is four captures of the same initial (unfocused) render. The `reported` data in the JSON is the harness's self-report of what it *intended* to render — it reads `focused: true` and `focus_ring_visible: true` for the relevant buttons — but the actual pixel content in the screenshots is identical across all four scenarios.

**Misleading element:** The `capture-summary.json` verdict shows `"focus_bounds_44px": true` for all scenarios (populated from the self-reported JSON, not from pixel analysis of the screenshots). Reports citing `sau-011` as proving focus ring behavior in the auction bid UI are citing the harness's self-report, not visual evidence. The screenshots themselves do not capture the focus state differences.

**This is a live instance of FP-1 (screenshots captured but not content-analyzed) applied to the harness infrastructure itself.**

---

### Claim 6-E: The only real two-client evidence is from commit `f08b2c8` (2026-05-12)

**Artifacts:**
- `production/qa/evidence/captures/manual-friend-game-evidence-2026-05-12/command-summary.md`
- `production/qa/evidence/captures/manual-friend-game-evidence-2026-05-12-backtrace/command-summary.md`
- `production/qa/evidence/captures/manual-friend-game-evidence-2026-05-12-auction-fix/command-summary.md`

These three documents are the ONLY evidence in the entire `production/qa/evidence/` directory of a real human-operated two-client native game session. They were all captured on 2026-05-12 and all three are on commit `f08b2c8` or `8e3d044` (two commits within the same sprint).

**What these runs actually showed:**
- Run 1 (commit `8e3d044`, 12:01): Real two-client game reached rounds 1–2. **Server crashed on round-2 placement entry with exit code 1** — confirmed regression.
- Run 2 (commit `8e3d044`, 13:24): 12 rounds clean but players did nothing (zero card purchases, zero activations). Clean because the crash was conditional on player interaction.
- Run 3 (commit `f08b2c8`, 17:15): 9 rounds clean with real bids, real purchases. Ended via disconnect (`GameOver{reason=Disconnect}`), not by reaching the game-over win/loss condition.

**Gap:** The current source tree is `origin/main@e7b51e84` (post-PROMPT-2019). The last real two-client native run was at `f08b2c8` — that is hundreds of commits and approximately 16 days ago (2026-05-12 vs. today 2026-05-28). **No real two-client session has been captured since `f08b2c8`.** Every story shipped since then has no visual human-operated evidence from the game itself.

**What this means for the "game is visually unplayable" report:** The audit cannot determine from these artifacts whether the game is currently unplayable — the raw evidence only goes to `f08b2c8`. However, given that:
1. 100+ story commits have landed since `f08b2c8`
2. No subsequent real-client session exists
3. All "PASS" evidence since then is from headless tests or HTML harnesses

...the reported visual unplayability is consistent with an undetected regression introduced somewhere in the post-`f08b2c8` commit range, which the toolchain would not have caught.

---

### Claim 6-F: Report-chain PROMPTs after 1900 create false progress signal

**Observation from `git log` (recent commits):**

Commits since PROMPT-1900 are almost exclusively `docs(reports)` commits. Titles include:
- "reapply autoplay visible-target coverage-map report chain over latest main after 1980"
- "POST-1912 autoplay viewport/window-guard verify report refresh after 1976"
- "reapply PROMPT 1979 autoplay window-resize verdict onto main after 1991"

Each commit adds a new report file with a new PROMPT number. The content of these reports is identical to earlier reports in the chain, updated only with the new base commit hash.

**The issue:** An orchestrator tracking PROMPT numbers as a proxy for progress sees 50+ merges since PROMPT-1900, suggesting active development. In reality:
- No new gameplay has been implemented
- No new QA runs have been executed
- No new visual evidence has been captured
- The merges are purely administrative: re-rooting stale branches that failed fast-forward checks

**This is the most operationally significant false positive:** It causes the orchestrator to believe the project is in active, healthy development with regular evidence refreshes, when the actual evidence base (real two-client runs) has been static since 2026-05-12.

---

## Section 7 — Updated Stop-The-Line Checklist (post-Phase-2 additions)

In addition to the 6 STL checks in Section 4, add:

### STL-7: Before accepting any story as Done — verify evidence source is not harness-only

Any story whose acceptance criteria cite visual evidence must confirm the evidence is from the real running Bevy client (native or WASM in browser), not from a Chrome DevTools Protocol capture of a localhost harness. Check `captureTool` field in the evidence JSON. If it says `"PowerShell Chrome DevTools Protocol"` and the URL is `http://127.0.0.1:8082/` or `http://127.0.0.1:8083/`, the evidence is harness-only.

**Harness-only evidence is valid for:** widget geometry, button label text, focus ring visibility in isolation.
**Harness-only evidence is invalid for:** proving the feature works in the integrated game client.

### STL-8: Check `attempted_full_native_or_browser_two_client_run` before accepting a smoke PASS

In any smoke or friend-game QA package, check for the explicit field:
```json
"attempted_full_native_or_browser_two_client_run": false
```
If this field is false (or absent), the smoke PASS is from headless in-process tests only — no visual game loop was exercised.

### STL-9: Verify the real-client evidence commit is within N commits of current HEAD

The last real two-client session was at `f08b2c8` (2026-05-12). Check:
```bash
git log --oneline f08b2c8..HEAD | wc -l
```
If this count is > 20, there is a substantial commit range with no visual coverage. Every story shipped in that range carries unverified visual risk.

---

## Amended Root Cause Statement

The original root cause (Section 5) stands. This phase-2 audit adds a structural layer:

**The evidence taxonomy is wrong.** Evidence is organized by artifact category (captures, traces, reports) rather than by what claim it substantiates. This allows:
1. Headless Rust tests to satisfy checklist items labeled as "game phase reached"
2. Chrome harness screenshots to satisfy checklist items labeled as "UI verified"
3. Non-executed routes to produce "no defects found" entries

A valid evidence taxonomy must require, for any story touching interactive game flow: at least one artifact from a real human-operated session (two native windows or two browser tabs) connected to a live server. Until such an artifact exists, the story's interactive acceptance criteria are unclosed regardless of how many headless tests pass.

---

2029: QA-EVIDENCE-TOOLING-TRUTHFULNESS-AUDIT: COMPLETE
