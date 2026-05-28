# PROMPT 1861 — AUTOPLAY-VSBOT-WINDOW-SIZE-OPERATOR-CONTRACT-RECONCILE

**Type:** Operator Contract / Runbook — Corrected and Reconciled
**Date:** 2026-05-28
**Status:** RECONCILED
**Supersedes:** PROMPT 1847 (`tmpwt-1847-window-size-operator-contract/reports/PROMPT-1847-autoplay-vsbot-window-size-operator-contract.md`)
**Reconciles against:** PROMPT 1844 (`reports/PROMPT-1844-autoplay-vsbot-viewport-click-evidence-audit.md`), PROMPT 1846 (`reports/PROMPT-1846-autoplay-evidence-analyzer-latest-run-application.md`)
**Active repairs referenced:** PROMPT 1842 (default window size), PROMPT 1857 (blocking click/viewport guard), PROMPT 1850 (composite verdict downgrade)

---

## RECONCILIATION NOTICE

PROMPT 1847 was shipped but contained evidence claims that were subsequently
invalidated by deeper forensic audits (PROMPT 1844, PROMPT 1846). The following
key corrections apply to this document:

| PROMPT 1847 Claim | Correction |
|---|---|
| PROMPT 1831 run `090613-Z` cited as passing "live PASS run" evidence | **INCORRECT** — 090613-Z is NOT clean automated PASS evidence; see §2 below |
| All three 2026-05-28 runs treated as providing sufficient automated evidence | **INCORRECT** — all three runs are PARTIAL; automated PASS is not claimed on any |
| PROMPT 1843 cited as "active/repairing" click viewport guard | **STALE** — PROMPT 1843 was renamed to PROMPT 1857; 1843 worktree not found at audit |

This document retains all valid operator checklist details from PROMPT 1847 and
adds explicit requirements from AC-VPT-01..08 (PROMPT 1844 §8) and the AC-VPT-05
composite downgrade requirement.

---

## 1. Purpose

This document is the authoritative operator/QA contract for running the autoplay
vs-bot GUI smoke. It defines the minimum required window geometry, the visibility
preflight checklist, what constitutes valid evidence, and a failure taxonomy drawn
from observed regressions (PROMPTs 1817, 1820, 1829, 1831, 1844, 1846).

PROMPTs 1842, 1857 (formerly 1843), and 1850 are repairing the default window
size, click-target viewport guard, and composite verdict downgrade respectively.
This contract documents the *requirements* those fixes must satisfy and the
operator checks that apply once they land.

---

## 2. Evidence Status of All 2026-05-28 Runs — NOT Automated PASS

**None of the three available runs constitutes an automated PASS.** All three
have been assessed as PARTIAL by the `analyze_evidence_run.py` analyzer
(PROMPT 1833/1846) and by forensic audit (PROMPT 1844).

| Run | Analyzer Verdict | Reason | Automated PASS? | Human-Review Usable? |
|---|---|---|---|---|
| `20260528-051148-Z` | **PARTIAL** | No win32 capture labels; only Bevy RPC screenshots; no pixel_hash data | **NO** | Possibly — requires human visual inspection of all 15 Bevy PNGs |
| `20260528-063609-Z` | **PARTIAL** | All 15 pixel_hash captures identical (`0x26207c4c`); frozen renderer throughout | **NO** | **NO** — frozen-all captures do not prove live state transitions |
| `20260528-090613-Z` | **PARTIAL** | Win32 PrintWindow frozen 11/15 times; mid-run window resize triggered stale coordinates | **NO** | Conditionally — bitblt PNGs show 12 distinct hashes; human must verify UI not clipped and clicks visible |

### 2.1 Run 090613-Z Is Specifically Not a Clean PASS

Run `090613-Z` was previously cited (PROMPT 1847, §5) as evidence that "PROMPT 1831"
produced a clean PASS-style result. PROMPT 1844 forensic audit and PROMPT 1846 analyzer
output have **invalidated** that characterisation. Specific disqualifiers:

1. **Mid-run window resize** — the window shrank from 720 to 505 px at tick 115
   (triggered by `ShowWindow SW_RESTORE` DWM snap-restore), then grew to 1076 px
   by tick 127. All clicks after tick 114 were executed against a 720-baked recipe
   in a 1076-high window — coordinate fractions are structurally wrong.

2. **720-baked coordinates post-resize** — recipe built at tick 1 for `[1280, 720]`;
   no rebuild after resize. Post-resize clicks (`placement-dragged`, `placement-submitted`)
   land at 61.5% height instead of the intended 92% (`y=662` in a 1076-high window).

3. **PrintWindow frozen throughout** — all 15 `win32_printwindow` captures returned
   frozen hashes. The 11 `desktop_bitblt` fallback captures show 12 distinct pixel
   hashes (real live frames), but the dominant capture path was frozen.

4. **Null cursor before resize** — `cursor_logical` was `None` at ticks 113–114,
   indicating the cursor was outside the window bounds at the moment the resize event
   fired. A click was dispatched at tick 115 with the cursor outside the active viewport.

5. **Checkpoints are time-based, not state-verified** — all 15 checkpoints passed
   because they are tick-count events, not confirmation that a click landed on the
   correct element. Checkpoint completion does not prove click accuracy.

**Summary**: 090613-Z is evidence of the window resize bug, not evidence of correct
bot behaviour. Its `bitblt_tick_*.png` files have conditional human-review utility
(12 distinct hashes), provided the reviewer confirms the UI is not clipped and the
bot actions are visible in the correct positions.

---

## 3. Required Window / Client Geometry

### 3.1 Minimum Acceptable Size (AC-VPT-08)

| Dimension | Minimum | Recommended | Notes |
|-----------|---------|-------------|-------|
| Width  | **1280 px** | 1280 px | Below this the HUD and lane tiles begin to clip |
| Height | **720 px**  | 1080 px | 720 px is the floor; 1080 px gives the full card hand visible |
| Resolution | 1280×720 | 1280×1080 | Minimum logical size for all fractional click targets to be in-bounds |

> **Enforcement requirement (AC-VPT-01):** The autoplay driver MUST abort with exit
> code ≠ 0 if the initial `window_logical_size` from the first status poll is below
> `[1280, 720]`. A shrinkage to `[W, < 720]` before any click MUST also abort.

### 3.2 Mid-Run Resize Policy (AC-VPT-02)

The driver MUST detect when `window_logical_size` changes after recipe build.
**On any resize > ±10 px in either dimension after tick 1, the driver MUST:**

1. Log a `WARN: window_resized` entry in `driver.log`.
2. Emit a `local.note` checkpoint with the old and new sizes.
3. Abort with a non-zero exit code OR mark the run `NEEDS_HUMAN_GUI` and refuse
   to emit a clean `smoke_exit_code=0`.

A run that experienced a mid-run resize (even if all checkpoints passed) is
classified as `NEEDS_HUMAN_GUI` and does not qualify for automated PASS signoff.
The composite report MUST record a non-zero `window_resize_events` count and
downgrade the verdict accordingly.

**Root cause of 090613-Z resize:** `win_foreground.py`'s `SetWindowPos +
ShowWindow SW_RESTORE` sequence interacted with Windows 11 DWM snap-restore,
triggering rapid `WM_SIZE` events. PROMPT 1857 (click/viewport guard) and
PROMPT 1850 (composite verdict downgrade) are addressing this.

### 3.3 Window State Requirements

| Requirement | Detail |
|-------------|--------|
| Not minimized | DWM does not composite minimized windows; PrintWindow returns a frozen/stale buffer |
| Visible on primary display | The game window must be on the primary monitor and not behind a full-screen application |
| Not occluded | Another full-screen window on top will produce stale captures from PrintWindow |
| Foreground (focus) | `foreground_robust` helper (PROMPT 1808) sets foreground before each capture; the window must be allow-listed by Windows for foreground switching |

### 3.4 Environment Variables — Required for vs-bot Recipe

These must be set **before** invoking the launcher. `Run-AutoplaySmoke.ps1` will
exit 4 (BLOCKED) if `CCGS_AUTOPLAY_BOT_ROOM_READY` is absent.

| Variable | Value | Who sets it | Effect if absent |
|----------|-------|-------------|-----------------|
| `CCGS_DEBUG_UI` | `"1"` | Operator (or auto-set by launcher) | Add Bot button hidden; recipe cannot add a bot |
| `CCGS_AUTOPLAY_BOT_ROOM_READY` | `"1"` | Operator | Driver exits BLOCKED (code 4) after `vs-bot-precheck` |
| `CCGS_AUTOPLAY` | `"1"` | Set by launcher automatically | RPC server not started |
| `CCGS_AUTOPLAY_PORT` | `"15873"` | Set by launcher automatically | Driver cannot connect |

> `Run-AutoplaySmoke.ps1` auto-sets `CCGS_DEBUG_UI=1` if absent (PROMPT 1824).
> It does **not** auto-set `CCGS_AUTOPLAY_BOT_ROOM_READY` — this is intentional,
> because that variable requires a live bot soak room to be running first.

---

## 4. Visibility Preflight — What the Operator Checks Before Starting

Run this checklist in order **before** invoking `Run-AutoplaySmoke.ps1`.

### Step 0 — Bot Soak Room Running

```powershell
pwsh -File tools/dev-launcher/Start-BotVsBotSoak.ps1
# Wait for: "Server listening on port ..."
```

Verify:
```powershell
netstat -an | Select-String ":5000"
# Expected: one LISTENING line
```

### Step 1 — Port 15873 Is Free

```powershell
netstat -an | Select-String ":15873"
# Expected: no output (nothing bound yet)
```

If output is present, a stale autoplay client is still running. Kill it before proceeding.

### Step 2 — Python Resolves Correctly

```powershell
python --version
# Expected: Python 3.12.x
# Canonical path: D:/_APPS/Python312/python.exe
```

### Step 3 — Stale `__pycache__` Cleared

The launcher clears `tools/autoplay/__pycache__` and `tools/autoplay/recipes/__pycache__`
automatically (PROMPT 1802/1814). No manual action required — but if running the
driver directly (not via the launcher), clear manually:

```powershell
Remove-Item -Recurse -Force tools/autoplay/__pycache__ -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force tools/autoplay/recipes/__pycache__ -ErrorAction SilentlyContinue
```

### Step 4 — Monitor Is Not Locked / Screen Saver Inactive

DWM compositing is suspended on a locked workstation. Run the autoplay smoke only
from an unlocked interactive desktop session with the monitor on.

### Step 5 — No Full-Screen Overlay (games, video, Zoom share)

A full-screen application on top of the game window will cause `PrintWindow` to
capture the overlay's stale DWM buffer rather than the game. Minimize all other
full-screen applications.

### Step 6 — Game Window Will Be Foregrounded; Desktop Left Idle

After the client launches, do not click another window or application. The
`foreground_robust` helper (PROMPT 1808) attempts to foreground the game window
before each capture, but Windows can deny this if focus has moved since startup.

**Additionally:** do not have any Windows-snapped windows on the same virtual
desktop as the game. The DWM snap-restore animation triggered by `SW_RESTORE`
on a snapped window caused the 090613-Z mid-run resize. If the game window is
not snapped at launch, this interaction cannot occur.

---

## 5. Launch Command (Canonical)

From `D:\_DEV\Work\Claude-Code-Game-Studios` in a PowerShell terminal:

```powershell
# Set required env var (bot soak room must already be running):
$env:CCGS_AUTOPLAY_BOT_ROOM_READY = "1"

# Run the vs-bot smoke:
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe vs-bot
```

Launcher will:
1. Auto-set `CCGS_DEBUG_UI=1` if absent.
2. `cargo build -p client --bin client --features autoplay-remote` — may take 5–8 min on cold build.
3. Launch client with `CCGS_AUTOPLAY=1`, `CCGS_AUTOPLAY_PORT=15873`.
4. Wait up to `$ClientStartupSecs` (default 60s) for port 15873 to bind.
5. Clear `__pycache__`, set `PYTHONDONTWRITEBYTECODE=1`.
6. Run `python -B driver.py --recipe vs-bot`.
7. Write artifacts to `production/qa/evidence/autoplay-runs/<stamp>/`.

---

## 6. Evidence That Proves the Bot Clicked Visible Targets

A successful vs-bot run must produce ALL of the following before an automated
PASS can be claimed. Runs with mid-run resize or frozen-all captures cannot
satisfy these criteria and must receive `NEEDS_HUMAN_GUI` or `PARTIAL` verdicts.

### 6.1 Composite Outcome

```
production/qa/evidence/composite-runs/<stamp>/composite-summary.json
  → "outcome": "ok"
```

### 6.2 All Checkpoints Reached

`production/qa/evidence/autoplay-runs/<stamp>/checkpoints.jsonl` must contain
all 15 checkpoints in order:

```
lobby-loaded → bot-added → lobby-confirmed → class-select-loaded →
class-confirmed → shop-loaded → shop-slot-clicked → auction-loaded →
auction-ready → placement-loaded → placement-dragged → placement-submitted →
resolution-started → resolution-complete → vs-bot-post-resolution
```

> **Important:** checkpoint passage alone does NOT prove click accuracy.
> Checkpoints are time-based (tick count). A run can pass all 15 checkpoints
> even if the bot clicked wrong targets in every post-resize phase.
> See §2.1 item 5 above.

### 6.3 No Mid-Run Window Resize (AC-VPT-02)

`driver.log` and `driver-timeline.jsonl` must show a stable `window_logical_size`
throughout the run. Any change exceeding ±10 px in either dimension after tick 1
disqualifies the run from automated PASS status.

The `composite-summary.json` field `window_resize_events` MUST be 0 for a clean PASS.
A non-zero count MUST downgrade the verdict to `NEEDS_HUMAN_GUI` (AC-VPT-07 / PROMPT 1850).

### 6.4 Cursor Not None Before Clicks (AC-VPT-03)

`driver-timeline.jsonl` must show `cursor_logical` is not `None` at any tick
immediately preceding a `mouse_down` action. A `None` cursor means the cursor
was outside the window boundary at click time — the click may have been silently
ignored by Bevy.

### 6.5 All Click Coordinates In-Bounds (AC-VPT-01, AC-VPT-08)

All `click(x, y)` entries in `driver.log` must have:
- `x < window_width` at the tick of the click
- `y < window_height` at the tick of the click
- `window_width >= 1280` and `window_height >= 720`

The PROMPT 1857 guard (when landed) will abort the run before any out-of-bounds
click is dispatched. Until that guard lands, this must be verified manually.

### 6.6 Distinct Screenshot Hashes (≥ 3 unique) (AC-VPT-06)

`driver.log` must show **≥ 3 distinct pixel hashes** across `win32_tick_*.png`
or `bitblt_tick_*.png` (whichever backend was active). This proves the game
rendered visually distinct states across the run.

If ALL win32_printwindow captures returned frozen hashes (`win32_capture_quality:
frozen_all`), the run MUST be downgraded to `NEEDS_HUMAN_GUI` — the automated
evidence does not prove live state transitions (AC-VPT-05 / PROMPT 1850).

```
# Good — 12 unique hashes from desktop_bitblt fallback (best current example, run 090613):
desktop_bitblt=OK hash=0xb4db8636
desktop_bitblt=OK hash=0xbb3c81cc
...  (12 distinct values)
# However: run 090613 has mid-run resize → still PARTIAL, not PASS
```

```
# Bad — all frozen (run 063609):
win32_capture=OK pixel_hash=0x26207c4c
win32_capture=OK pixel_hash=0x26207c4c  ← same
... (1 hash across all 15 ticks) → INSUFFICIENT, not PASS
```

### 6.7 Post-1818 Log Labels

`driver.log` must use the post-1818 label format:

| Label | Meaning |
|-------|---------|
| `win32_printwindow=OK` | PrintWindow succeeded, frame is new |
| `win32_printwindow=FROZEN hash=<md5>` | PrintWindow froze; BitBlt fallback triggered |
| `desktop_bitblt=OK reason=frozen_printwindow` | BitBlt captured a live frame |

The pre-1818 label `win32_capture=OK` alone (without `win32_printwindow`) indicates
stale bytecode — the driver ran from `__pycache__` compiled before PROMPT 1813/1818.
Stop the run, clear `__pycache__`, and re-run.

> **Note on run 063609:** This run shows `win32_capture` labels but NOT
> `win32_printwindow` labels, and all 15 hashes are identical. This is the stale
> bytecode / pre-1813 capture pattern (F4 in §7 below). The run cannot be used
> as evidence of live gameplay.

### 6.8 Launcher Status JSON

```json
{
  "outcome": "ok",
  "driver_exit_code": 0,
  "client_exit_code": null
}
```

`client_exit_code: null` is expected — the launcher kills the client after the
driver exits cleanly.

### 6.9 Human Operator Sign-Off (AUTOPLAY-VS-BOT-QA-001)

Per the composite harness disclaimer, a live PASS for **AUTOPLAY-VS-BOT-QA-001**
requires human operator sign-off. The automated run produces evidence; a human
must review the artifacts and affirm:

- The UI was not clipped in any screenshot.
- Bot action clicks are visible on the correct UI elements.
- No screenshot shows blank space or an offscreen click target.
- The run had no mid-run window resize (or, if it did, the human explicitly
  verifies post-resize clicks landed on visible targets despite coordinate mismatch).

**Human review of 090613-Z:** The `bitblt_tick_*.png` files (11 files) show 12
distinct pixel hashes and represent real live frames. A reviewer can inspect these
alongside the 15 Bevy RPC screenshots to check for UI clipping and click visibility.
However, the reviewer must explicitly acknowledge the post-resize coordinate mismatch
(§2.1 items 1–2) and confirm whether post-resize actions (auction-ready through
placement-submitted) produced the expected game state transitions despite the misaligned
coordinates.

---

## 7. Failure Taxonomy

### F1 — Too-Small Window (AC-VPT-01 / AC-VPT-08)

**Symptom:** Bot reaches `shop-slot-clicked` but never reaches `auction-loaded` or later.
Click coordinates are computed from viewport dimensions at startup; if the window was
smaller than expected, the computed target lands outside the visible button area.

**PROMPT 1842 is repairing this** — default window size will be standardized to `[1280, 720]`.
**PROMPT 1857** adds a guard that aborts on `window_height < 720` at first status poll.

**Immediate mitigation:** Resize the game window to ≥ 1280×720 before launching the run.
If the window starts too small, kill the run, resize, and restart.

---

### F2 — Clipped UI / Offscreen Layout

**Symptom:** Correct window size but UI elements rendered outside the visible area.
The bot's click reaches the viewport but the target widget was not drawn in its expected
position (e.g., the card hand was clipped below the bottom edge).

**PROMPT 1857 is repairing this** — click-target viewport guard will validate coordinates.

---

### F3 — Offscreen Click (AC-VPT-03)

**Symptom:** The bot sends a click at coordinates outside the window's current client rect.
Win32 mouse events outside the client area are silently ignored by the game.

**Observed in:** 090613-Z tick 115 — click dispatched when cursor was `None` (outside window)
during mid-run resize. The `y=396` coordinate targeted for a 720-height window was
structurally wrong in the 505-high window at dispatch time.

**Fix (PROMPT 1857):** Guard that asserts `cursor_logical is not None` and
`click_y < window_height` before dispatch; abort run if violated.

---

### F4 — Stale Screenshot (Pre-1818 Bytecode)

**Symptom:** All screenshots are byte-for-byte identical. `driver.log` uses the label
`win32_capture=OK` without any `win32_printwindow=` lines.

**Root cause:** The Python driver ran from a stale `.pyc` file compiled before PROMPT 1813
modified the capture backend. The stale-pyc guard (PROMPT 1802/1814) clears `__pycache__`
before every launcher-run; this failure only occurs when the driver is invoked directly
without the launcher, or when the guard is bypassed.

**Observed in:** Run 063609-Z — `win32_capture` only, all 15 hashes identical.

---

### F5 — Frozen PrintWindow (DWM Stale Buffer)

**Symptom:** `PrintWindow` API returns `True` (success) on every tick but all frames are
identical. Post-1818 driver emits `win32_printwindow=FROZEN` and falls back to
`desktop_bitblt`.

**Root cause:** Windows DWM does not update the PrintWindow buffer when the source window
is not actively composited (minimized, occluded, locked workstation, or mid-resize).

**Observed in:** 090613-Z — 11/15 captures frozen after window resize.

**If ALL printwindow captures are frozen** (AC-VPT-05), the composite report MUST record
`win32_capture_quality: frozen_all` and downgrade to `NEEDS_HUMAN_GUI`. PROMPT 1850
implements this downgrade.

---

### F6 — Insufficient Distinctness (< 3 unique hashes)

**Symptom:** Run completes (exit 0, all checkpoints) but only 1–2 distinct screenshot
hashes appear. The game UI did not visually change — render stall, frozen-frame miss, or
limited state transitions.

**Threshold:** ≥ 3 distinct hashes required. Runs 051148-Z and 063609-Z do not meet
this threshold (0 hashes and 1 hash respectively). 090613-Z produces 12 distinct
`desktop_bitblt` hashes — passes this criterion — but fails on mid-run resize.

---

### F7 — Mid-Run Window Resize (AC-VPT-02) — NEW

**Symptom:** `driver-timeline.jsonl` shows `window_logical_size` changing by > ±10 px
after tick 1. All recipe coordinates become stale (baked at tick 1, not updated).
Post-resize click fractions are structurally wrong.

**Observed in:** 090613-Z — 720 → 505 → 1076 during ticks 115–127.

**Root cause:** `win_foreground.py` `SW_RESTORE + SetWindowPos` on a DWM-snapped window
triggers snap-restore animation with rapid `WM_SIZE` events. Winit propagates each event.

**Fix:** PROMPT 1857 aborts the run on detection. PROMPT 1842 ensures the window is not
snapped at launch. The Rust `AutoplayPlugin` `window.resolution.set()` at `Startup`
(PROMPT 1842) sets initial size but does not block OS-initiated `WM_SIZE` events —
a `WindowResized` guard in Python is still required.

---

### F8 — BLOCKED Exit (Code 4) — Env Var Not Set

**Symptom:** Launcher exits immediately after the client builds. `driver.log` shows
`vs-bot-precheck` emitting `local.block`.

**Root cause:** `CCGS_AUTOPLAY_BOT_ROOM_READY` was not set, or the bot soak room
was not running before the driver connected.

---

## 8. Acceptance Criteria for Repairs (from PROMPT 1844 §8)

These are the binding ACs that PROMPT 1842, 1857, and 1850 must collectively satisfy
before a clean automated PASS can be claimed on any future run:

| AC | Requirement | Blocking? | Responsible PROMPT |
|---|---|---|---|
| AC-VPT-01 | Driver aborts (exit ≠ 0) if initial `window_logical_size` < `[1280, 720]` | BLOCKING | 1857 |
| AC-VPT-02 | Driver detects mid-run resize > ±10 px; logs `WARN: window_resized`; aborts or marks `NEEDS_HUMAN_GUI` | BLOCKING | 1857 |
| AC-VPT-03 | Driver warns and skips click (or aborts) if `cursor_logical` is `None` before `mouse_down` | ADVISORY | 1857 |
| AC-VPT-04 | If run continues after resize, recipe must be rebuilt for new window size | ADVISORY | future |
| AC-VPT-05 | If all `win32_printwindow` captures are frozen, composite report records `frozen_all`; verdict downgraded to `NEEDS_HUMAN_GUI` | ADVISORY | 1850 |
| AC-VPT-06 | Claiming mechanical PASS requires ≥ 1 in-game screenshot per checkpoint and ≥ 3 distinct `desktop_bitblt` hashes | BLOCKING | 1850 |
| AC-VPT-07 | Composite report records `initial_window_size`, `final_window_size`, `window_resize_events`; non-zero count downgrades verdict | ADVISORY | 1850 |
| AC-VPT-08 | Client launcher sets `CCGS_WINDOW_WIDTH=1280` and `CCGS_WINDOW_HEIGHT=720`; Rust `AutoplayPlugin` enforces via `window.resolution.set()` at `Startup` | NORMATIVE | 1842 |

---

## 9. Quick-Reference Summary Card

```
BEFORE RUNNING:
  [ ] Bot soak room running  →  netstat :5000 → LISTENING
  [ ] Port 15873 free        →  netstat :15873 → no output
  [ ] Screen unlocked, monitor on, no full-screen overlay
  [ ] Game window NOT in a DWM-snapped state (prevents mid-run resize)
  [ ] Leave desktop idle after launch (do not click away)
  [ ] $env:CCGS_AUTOPLAY_BOT_ROOM_READY = "1"

AFTER RUNNING:
  [ ] composite-summary.json → "outcome": "ok"
  [ ] composite-summary.json → window_resize_events: 0
  [ ] All 15 checkpoints in checkpoints.jsonl
  [ ] driver.log: "win32_printwindow=" labels present (not "win32_capture=" only)
  [ ] driver.log: ≥ 3 distinct hashes across win32_tick_*.png or bitblt_tick_*.png
  [ ] launcher-status.json: "driver_exit_code": 0
  [ ] driver-timeline.jsonl: window_logical_size stable throughout (no resize > ±10 px)
  [ ] driver-timeline.jsonl: cursor_logical not None at any mouse_down tick

FAILURE FLAGS (updated):
  1 distinct hash only          → F4 stale pyc  OR  F5 frozen PrintWindow (pre-1818)
  FROZEN lines + ≥ 3 hashes OK → F5 handled by 1818 fallback — acceptable if resize=0
  FROZEN lines + resize > 10px  → F5 + F7 — mid-run resize; run is NEEDS_HUMAN_GUI
  Checkpoints stop early        → F1 too-small window  OR  F3 offscreen click
  driver exit 4                 → F8 env var not set
  win32_capture=OK only (no win32_printwindow) → F4 stale pyc — re-run with cleared __pycache__
  window_resize_events > 0      → F7 mid-run resize — automated PASS NOT claimable

CURRENT STATUS OF THREE 2026-05-28 RUNS:
  051148-Z  → PARTIAL — no pixel_hash data; human visual review required
  063609-Z  → PARTIAL — all hashes frozen/identical; NOT usable for any PASS
  090613-Z  → PARTIAL — mid-run resize + frozen printwindow; NOT automated PASS
             (conditional human-review utility: review bitblt_tick_*.png + Bevy PNGs)
```

---

## 10. Relationship to Active Repairs

| PROMPT | Repair | Status at 2026-05-28 | Impact on This Contract |
|--------|--------|---------------------|------------------------|
| 1842 | Default window size (`CCGS_WINDOW_WIDTH/HEIGHT` + Rust `window.resolution.set()`) | Active, not landed on main | Eliminates F1; AC-VPT-08 |
| 1857 | Click-target viewport guard (was PROMPT 1843) | Active, not landed on main | Eliminates F3, AC-VPT-01/02/03 |
| 1850 | Composite verdict downgrade on resize/frozen-all | Active, not landed on main | Implements AC-VPT-05/07; ensures PARTIAL not mis-reported as PASS |

Once all three repairs land on `main`, re-run the smoke and confirm:
- No F1/F3/F7 failures under default launch conditions.
- The Section 6 evidence checklist is fully satisfied.
- `window_resize_events: 0` in composite-summary.json.
- `distinct_hashes >= 3` and no frozen-all.
- This contract document remains valid; the failure taxonomy entries still apply
  to non-default or manually resized windows.

---

1861: AUTOPLAY-VSBOT-WINDOW-SIZE-OPERATOR-CONTRACT-RECONCILE: SHIPPED
