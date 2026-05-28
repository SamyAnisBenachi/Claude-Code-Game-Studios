# PROMPT 1982 — POST-1976-AUTOPLAY-VSBOT-CONTROLLED-GUI-SMOKE

**Date:** 2026-05-28
**Branch:** `work/PROMPT-1982`
**Base:** `origin/main` @ `32a59256d1de9a4fee362a2aa9006d1bb69b59db` (PROMPT 1976)
**Status:** NEEDS_HUMAN_GUI

---

## Summary

One controlled autoplay-vs-bot GUI smoke was executed against
`origin/main@32a59256` using the standard composite launcher
(`Start-AutoplayVsBot.ps1 -Recipe vs-bot`).

The game window opened correctly at exactly **1280×720 logical pixels** (AC-VPT-08
enforced via PROMPT 1912 Startup system + Run-AutoplaySmoke.ps1 default). The client
reached **Lobby phase**, the soak server bound on port 5000, and a screenshot was
captured. However, the run **aborted at tick 6** on the AC-VPT-03 viewport guard
(`cursor_logical=None`) before the first click could be dispatched.

The driver exit code is **5** (EXIT_VIEWPORT_GUARD), not 0. The composite outcome is
`smoke_failed_exit_5`. All required vs-bot checkpoints beyond `lobby-loaded` are
absent. Status: **NEEDS_HUMAN_GUI**.

---

## Run Parameters

| Field | Value |
|-------|-------|
| Launch command | `Start-AutoplayVsBot.ps1 -Recipe vs-bot -PlayRepoRoot … -Python …` |
| Composite evidence dir | `production/qa/evidence/composite-runs/2026-05-28-163529-autoplay-vs-bot` |
| Autoplay run dir | `production/qa/evidence/autoplay-runs/20260528-163531-Z` |
| Run started (UTC) | 2026-05-28T16:40:08Z |
| Run finished (UTC) | 2026-05-28T16:41:21Z |
| origin/main SHA | `32a59256d1de9a4fee362a2aa9006d1bb69b59db` |
| Soak server port | 5000 (free; soak bound within 30 s) |
| RPC port | 15873 (free) |
| Python | `D:/_APPS/Python312/python.exe` |
| CARGO_TARGET_DIR | `D:/_DEV/cargo-target/ccgs-msvc` (shared incremental cache) |
| CCGS_WINDOW_WIDTH | 1280 (set by Run-AutoplaySmoke.ps1 default) |
| CCGS_WINDOW_HEIGHT | 720 (set by Run-AutoplaySmoke.ps1 default) |
| CCGS_DEBUG_UI | 1 (set by composite launcher for vs-bot recipe) |
| CCGS_AUTOPLAY_BOT_ROOM_READY | 1 (set by composite launcher after soak bound) |
| Driver timeout | 420 s |
| Client startup cap | 120 s |

---

## Window / Viewport Evidence

### Window size at first action tick

```
status.json → window_logical_size: [1280, 720]
```

Window is exactly 1280×720 logical pixels — meets the ≥1280×720 minimum.

The `enforce_autoplay_window_size_system` Startup system (PROMPT 1912 / AC-VPT-08)
enforced this at client start.

### Screenshot captured before abort

```
driver.log: win32_capture: PNG written win32_tick_000005.png 1296x759 (46977 bytes)
            pixel_hash=0x192481b0
```

One win32_printwindow PNG captured (1296×759 — physical pixels including window
chrome; logical content is 1280×720). Non-black, non-frozen (single pixel_hash,
non-zero, non-blank). `screenshots/000000.png` also written (Bevy screenshot, 88506 bytes).

---

## Driver Abort: AC-VPT-03 Cursor Guard

### Exact driver.log lines

```
2026-05-28T16:41:18Z recipe=vs-bot actions=74 last_recipe_tick=260 build_win=(1280x720)
2026-05-28T16:41:18Z tick=1 checkpoint label=lobby-loaded
2026-05-28T16:41:19Z foreground: matched window title='Lanes and Lies' hwnd=0x023b124a
2026-05-28T16:41:19Z foreground_robust: SetForegroundWindow ret=1 hwnd=0x023b124a
2026-05-28T16:41:19Z win32_capture: pixel_hash=0x192481b0 width=1296 height=759
2026-05-28T16:41:19Z win32_capture: PNG written win32_tick_000005.png 1296x759 (46977 bytes)
2026-05-28T16:41:19Z tick=5 win32_printwindow=OK path=win32_tick_000005.png
2026-05-28T16:41:19Z tick=5 action method=autoplay/screenshot params_keys=['reason']
2026-05-28T16:41:19Z tick=5 screenshot file ready: 000000.png (88506 bytes)
2026-05-28T16:41:19Z tick=6 VIEWPORT-GUARD cursor_logical=None (cursor outside window); aborting before input dispatch
2026-05-28T16:41:19Z recipe aborted by viewport guard on tick 6
2026-05-28T16:41:19Z exit rc=5
```

### What tick 6 does

```
{'tick': 6, 'method': 'autoplay/input', 'params': {'cursor': {'screen': [640.0, 396.0]}}}
```

Tick 6 is the first `autoplay/input` in the recipe — a cursor-move to (640, 396)
inside the window. The viewport guard in driver.py fires before ANY `autoplay/input`
dispatch, checking that `cursor_logical` is non-None in the current status. Because
the physical mouse cursor was not hovering over the game window at that moment,
Bevy reported `cursor_logical: null` in the RPC status response. The guard aborted
the run.

### Checkpoint coverage

```
checkpoints.jsonl:
  {"tick":1,"kind":"checkpoint","label":"lobby-loaded",...}
  {"tick":6,"kind":"viewport_guard_cursor_none","reason":"cursor_logical is None — cursor left the window",...}
```

Only `lobby-loaded` was reached. Required checkpoints not reached:
`bot-added`, `lobby-confirmed`, `class-select-loaded`, `placement-loaded`,
`placement-submitted`.

---

## Analyzer + Validator Output

### analyze_evidence_run.py

```
=== analyze_evidence_run: 20260528-163531-Z ===

--- Launcher / Driver ---
  launcher_outcome : 'driver_failed'
  driver_exit_code : 5
  client_exit_code : None

--- Capture Labels ---
  families seen    : ['win32_capture', 'win32_printwindow']
  FROZEN lines     : 0

--- Screenshots ---
  root win32_tick  : 1 PNG(s)
  bevy screenshots : 1 PNG(s)
  total            : 2

--- pixel_hash ---
  total captures   : 1
  distinct count   : 1
  distinct values  : ['0x192481b0']
  frozen pattern   : False

VERDICT: FAIL
REASON : driver_exit_code=5
```

### validate_composite_run.py (--recipe vs-bot)

```
[validate_composite_run] FAIL: …/2026-05-28-163529-autoplay-vs-bot
  1 check(s) failed:
  FAIL: MISSING CHECKPOINTS for recipe 'vs-bot': 'bot-added', 'lobby-confirmed',
        'class-select-loaded', 'placement-loaded', 'placement-submitted'
  Observed labels: ['lobby-loaded']
```

---

## Acceptance Criteria Audit

| Acceptance Item | Status | Evidence |
|---|---|---|
| Driver exit 0 | ❌ FAIL | exit rc=5 (EXIT_VIEWPORT_GUARD) |
| Composite outcome ok/PASS | ❌ FAIL | outcome=smoke_failed_exit_5 |
| Game window ≥1280×720 | ✅ PASS | status.json: window_logical_size=[1280,720] |
| No offscreen click errors | ✅ N/A | Guard fired before any click dispatched |
| Distinct screenshot evidence | ⚠️ PARTIAL | 1 hash, non-black (single-frame run — not enough ticks for multi-shot comparison) |
| vs-bot checkpoint coverage | ❌ FAIL | Only lobby-loaded; 5 required checkpoints absent |

Three of six acceptance items fail. Status: **NEEDS_HUMAN_GUI**.

---

## Root Cause

**AC-VPT-03 cursor guard (driver.py):** Before any `autoplay/input` action is
dispatched, the driver reads `cursor_logical` from the Bevy RPC status. Bevy only
reports a non-null cursor position when the OS cursor is physically hovering over
the game window. In this automated session, the cursor was elsewhere on the
2560×1600 desktop at tick 6.

The guard was introduced by PROMPT 1857 / 1894 to prevent offscreen clicks (the
original bug described in the PROMPT 1982 context). It is working as designed.
The limitation is that the recipe cannot "bootstrap" cursor position from outside
the window: the first `autoplay/input` (which is a cursor-move) is blocked by the
same guard it would need to bypass.

**Interaction summary:**
- `cursor_logical=None` → first `autoplay/input` blocked → EXIT_VIEWPORT_GUARD (5)
- Game was healthy: Lobby loaded, window 1280×720, soak connected, screenshot non-black

---

## AC-VPT Status at origin/main@32a59256

| AC | Requirement | Status | Notes |
|---|---|---|---|
| AC-VPT-01 | Driver aborts if initial window < 1280×720 | **LANDED** | Not triggered (window was 1280×720) |
| AC-VPT-02 | Driver detects mid-run resize > ±10 px | **LANDED** | Not triggered (run aborted at tick 6) |
| AC-VPT-03 | Driver aborts if `cursor_logical=None` before `mouse_down` | **LANDED** | **Triggered this run** — exit 5 at tick 6 |
| AC-VPT-08 | Launcher sets CCGS_WINDOW_WIDTH/HEIGHT=1280/720 + Rust enforces at Startup | **LANDED** | Window=1280×720 confirmed |
| AC-VPT-05/06/07 | Composite verdict downgrade (PROMPT 1850) | Unconfirmed | Out of scope; not triggered this run |

---

## Repair Prompt Recommendation

The blocker is AC-VPT-03 in an automated context. Two repair paths:

### Option A — Pre-run cursor warmup (tooling-only fix, no recipe change)

Add a `Invoke-AutoplayCursorWarmup` step to `Start-AutoplayVsBot.ps1` (or
`Run-AutoplaySmoke.ps1`) that, after the client RPC port binds:
1. Calls `autoplay/status` to get the RPC-reported `window_logical_size`
2. Uses `GetWindowRect` via ctypes/win32 to get the window's screen-space position
3. Calls `SetCursorPos` to move the cursor to the window centre (e.g. window_left +
   window_width/2, window_top + window_height/2)
4. Waits ~500 ms for Bevy's cursor reporting to stabilize

This requires no recipe changes. The guard fires correctly; the warmup ensures
the cursor is inside the window when tick 6 arrives.

### Option B — Relax AC-VPT-03 for pure cursor-move actions

In `driver.py`, only enforce the `cursor_logical=None` guard when the action
dispatches a `mouse_down` or `mouse_up` event (not for bare `cursor` moves).
Reasoning: a pure cursor-move action cannot itself cause an offscreen click — it
just repositions the cursor. The guard was designed to prevent accidental mouse_down
at wrong coordinates; it should not prevent the recipe from first moving the cursor
into the window.

### Recommended

Option A is less invasive and keeps the guard logic intact. A PROMPT for Option A:
`PROMPT NNNN — autoplay-vsbot-cursor-warmup-before-recipe-start`

---

## Infrastructure Notes

- **Cargo build:** Incremental via shared target `D:/_DEV/cargo-target/ccgs-msvc`.
  Client compiled successfully (only deprecation warnings, no errors). Build time
  included in the 60–120 s startup window.
- **Soak server:** Started as background job on port 5000, bound within 30 s.
  `server.exe` and `bot-soak-trigger.exe` from today's build (2026-05-28 ~02:13 UTC)
  were reused without rebuild.
- **Stale-pyc guard:** Both `tools/autoplay/__pycache__` and `recipes/__pycache__`
  were absent (no stale pyc to clear). `PYTHONDONTWRITEBYTECODE=1` active.
- **Platform:** Console session, 2560×1600 display, `UserInteractive=True`.

---

## Evidence Paths

| Artifact | Path |
|---|---|
| Composite summary | `production/qa/evidence/composite-runs/2026-05-28-163529-autoplay-vs-bot/composite-summary.json` |
| Autoplay run dir | `production/qa/evidence/autoplay-runs/20260528-163531-Z/` |
| Driver log | `…/20260528-163531-Z/driver.log` |
| Launcher status | `…/20260528-163531-Z/launcher-status.json` |
| Checkpoints | `…/20260528-163531-Z/checkpoints.jsonl` |
| Win32 screenshot | `…/20260528-163531-Z/win32_tick_000005.png` (1296×759) |
| Bevy screenshot | `…/20260528-163531-Z/screenshots/000000.png` (88506 bytes) |
| Status snapshot | `…/20260528-163531-Z/status.json` |

---

1982: POST-1976-AUTOPLAY-VSBOT-CONTROLLED-GUI-SMOKE: NEEDS_HUMAN_GUI
