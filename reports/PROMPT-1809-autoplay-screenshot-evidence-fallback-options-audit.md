# PROMPT 1809 — AUTOPLAY-SCREENSHOT-EVIDENCE-FALLBACK-OPTIONS-AUDIT

**Status:** SHIPPED (report only)
**Date:** 2026-05-28 UTC
**HEAD referenced:** `d6aabbdd` (origin/main at task time)

---

## Executive Summary

The core blocking condition is that `SetForegroundWindow` returns 0 on every
call during automated runs. This is a Windows foreground-lock enforcement:
only the process that currently holds the foreground (desktop shell,
task bar, or other foreground app) can transfer focus to another window.
The Python driver process has no HWND of its own, so the Win32 workaround
path (`AllowSetForegroundWindow` + `AttachThreadInput`) cannot be applied
from the driver side without additional support.

Both current capture paths — Bevy RPC `Screenshot::primary_window()` and
Win32 `PrintWindow(PW_RENDERFULLCONTENT)` — depend on DWM actively
compositing the game window (i.e. it must be the foreground window, or at
minimum a recently-foreground window whose backbuffer DWM has not staled).
Because neither condition is met, both produce a frozen identical frame
across all 15 capture ticks even though game logic (checkpoints) progresses.

This report evaluates five fallback strategies, states implementation scope,
risk, reliability, and the test that would prove each one.

---

## Current Architecture (Reference)

| Layer | File | Mechanism | DWM dependency? |
|---|---|---|---|
| Python foreground barrier | `tools/autoplay/win_foreground.py` | `ShowWindow` + `SetForegroundWindow` + `BringWindowToTop` | N/A (enables DWM compositing) |
| Win32 GDI capture | `tools/autoplay/win_capture.py` | `PrintWindow(PW_RENDERFULLCONTENT)` | **YES** — captures DWM-composited buffer |
| Bevy RPC capture | `client/src/autoplay.rs` (`drain_commands_system`) | `Screenshot::primary_window()` → `save_to_disk` | **YES** — reads GPU swapchain |
| Screenshot file poll | `tools/autoplay/screenshot_poll.py` | Waits up to 3 s for PNG to land | No |
| WinitSettings | `client/src/autoplay.rs` (`AutoplayPlugin::build`) | `WinitSettings::game()` — no tick throttle | No |
| Offscreen camera (opt-in) | `client/src/autoplay.rs` (`setup_offscreen_target_system`) | Secondary Camera2d → Image target | **NO** — but misses bevy_ui |
| QA Snapshot | `client/src/presentation/qa_snapshot.rs` | F9 / button → JSON + `Screenshot::primary_window()` | YES (for PNG) / **NO** (for JSON) |

**Observed failure mode (PROMPT 1807):**
- 15/15 checkpoints reached (game logic is fine)
- `SetForegroundWindow ret=0` on all 15 ticks
- `pixel_hash=0x26207c4c` constant across all Win32 captures
- All 15 Bevy RPC screenshots share MD5 `34556a68da323f8a3824e87f9ea3f00b`
- Composite validator exits 1: `IDENTICAL-SCREENSHOTS`

---

## Option 1 — Stronger Win32 Focus Activation (AttachThreadInput / ALT injection)

### What it does

`SetForegroundWindow` fails from a background process because Windows 2000+
foreground-lock rules only allow transfer from the current foreground process.
Two well-known workarounds exist:

**1a. AttachThreadInput**:
Call `GetWindowThreadProcessId(hwnd, ...)` to get the target thread ID, then
`AttachThreadInput(calling_thread_id, target_thread_id, True)`, then
`SetForegroundWindow(hwnd)`, then `AttachThreadInput(..., False)`.
This syncs the input queue of the Python thread with the Bevy/winit thread,
tricking Windows into accepting the foreground transfer.

**1b. ALT key injection before SetForegroundWindow**:
Send a `VK_MENU` keydown + keyup via `keybd_event` or `SendInput` before
calling `SetForegroundWindow`. Windows considers any recent user keystroke as
"foreground entitlement", and will permit the transfer.

Both can be combined for maximum coverage.

### Implementation Scope

- **Files**: `tools/autoplay/win_foreground.py` (`_foreground_window`),
  optionally also `tools/autoplay/win_capture.py` (call before PrintWindow)
- **Lines changed**: ~20–35 lines added to `_foreground_window`
- **Python API used**: `ctypes.windll.user32.GetWindowThreadProcessId`,
  `ctypes.windll.kernel32.GetCurrentThreadId`, `ctypes.windll.user32.AttachThreadInput`,
  `ctypes.windll.user32.keybd_event` — all stdlib ctypes, no external deps

### Risk

- **Medium.** `AttachThreadInput` can deadlock if the target thread is blocking
  on a Windows message (rare but possible during Bevy/winit system calls).
  Mitigate by always detaching in a `finally` block.
- ALT injection may trigger unintended game keyboard handlers if the game has
  ALT bindings. Bevy's default keyboard input does not bind Alt in the autoplay
  recipe, but the game may — inspect `_coords.py` and recipe ALT usage first.
- Technique is well-established in Windows test automation (Pywinauto, AutoIt).
  Will not work across UAC elevation boundaries or Terminal Services sessions.

### Likely Reliability

**High (≈85%)** in a standard single-user desktop session where the Python
driver and Bevy client run as the same user. Drops to near-zero across
elevation boundaries or locked/screensaver sessions.

### Proof Test

`SetForegroundWindow ret=1` appears in at least the first 3 driver.log ticks;
`pixel_hash` in `win32_tick_*.png` differs between lobby tick and resolution
tick; composite validator exits 0.

### Ownership

`tools/autoplay/win_foreground.py` — autoplay Python tooling owner.
PROMPT 1808 is already scoped to this; if PROMPT 1808 ships AttachThreadInput,
Option 1 is closed.

---

## Option 2 — Desktop-Region BitBlt Capture (No Foreground Dependency)

### What it does

Instead of `PrintWindow(PW_RENDERFULLCONTENT)` (which reads the DWM-composited
window buffer), use `BitBlt` from the **desktop device context** (`GetDC(NULL)`)
into a compatible bitmap over the window's screen rectangle (`GetWindowRect`).
`BitBlt` from the desktop DC captures whatever pixels are physically present on
screen at those coordinates at the moment of the call — independent of
foreground/compositing state, as long as the window is visible and not obscured.

### Implementation Scope

- **Files**: `tools/autoplay/win_capture.py` (`_capture_hwnd_to_png`)
- **Lines changed**: ~15–20 lines; replace the `hwnd_dc = user32.GetDC(hwnd)`
  path with `desktop_dc = user32.GetDC(None)` + `GetWindowRect` for bounds +
  `BitBlt(mem_dc, 0, 0, w, h, desktop_dc, rect.left, rect.top, SRCCOPY)`.
  `_SW_RESTORE` / `ShowWindow` before capture to ensure the window is not minimised.
- No foreground requirement; no `SetForegroundWindow` call needed.

### Risk

- **Low-Medium.** Captures whatever is at the window's screen coordinates —
  if another window is fully covering the game window (possible in automated
  CI where other processes open dialogs), you capture the covering window's
  pixels. Mitigation: add `SetWindowPos(hwnd, HWND_TOPMOST, ...)` before
  the `BitBlt` (then restore `HWND_NOTOPMOST` after) so the game window is
  always on top for the duration of the capture.
- Does not work if the window is minimised (pixels at those coordinates belong
  to another surface). The existing `ShowWindow(_SW_RESTORE)` call handles this.
- Does not work on a locked screen / screensaver / headless RDP session without
  a virtual framebuffer.

### Likely Reliability

**High (≈80%)** in a normal desktop session where the game window is visible.
Can be combined with `HWND_TOPMOST` toggle for near-100% in the non-locked case.

### Proof Test

`pixel_hash` differs between lobby tick and at least one post-lobby tick;
all 15 `win32_tick_*.png` files do not share the same MD5; composite validator
exits 0.

### Ownership

`tools/autoplay/win_capture.py` — autoplay Python tooling owner.
Can be delivered as a self-contained change, parallel to Option 1.

---

## Option 3 — Bevy-Side Offscreen Render Target (No DWM Dependency)

### What it does

`client/src/autoplay.rs` already contains an experimental offscreen path
(`CCGS_AUTOPLAY_OFFSCREEN=1`): a secondary `Camera2d` renders the scene into
an `Image` asset; `Screenshot::image(handle)` captures it without touching the
OS swapchain. The known limitation (documented in `autoplay.rs:53–56`) is that
`bevy_ui` only renders to `IsDefaultUiCamera` cameras, so the secondary camera
misses UI content.

The fix: in Bevy 0.18 the `UiCameraConfig` / `IsDefaultUiCamera` approach
has changed. To render bevy_ui to the offscreen camera, the offscreen camera
must become the `IsDefaultUiCamera`, but the primary display camera must also
render UI. One approach is to use `Camera::is_active = true` on both cameras
and set `IsDefaultUiCamera` on the offscreen one, with the understanding that
Bevy 0.18 may render UI to multiple cameras if both carry `IsDefaultUiCamera`.
An alternative is to write a `RenderGraph` pass that copies the final primary
swapchain output to a CPU-readable texture — but that is much deeper engine work.

### Implementation Scope

- **Files**: `client/src/autoplay.rs` (`setup_offscreen_target_system`,
  `drain_commands_system`)
- **Lines changed**: ~30–60 lines
- **Bevy API risk**: Very high. The offscreen UI rendering path requires
  understanding exactly how Bevy 0.18 routes bevy_ui to cameras. The
  `IsDefaultUiCamera` component and `CameraRenderGraph` API differ across
  0.15–0.18. Must consult `liv-bevy-018` skill and the 0.17→0.18 migration
  guide before any edit.

### Risk

- **High.** Touching camera ordering for an always-on offscreen target risks
  breaking normal game display (UI disappears, double-rendering artefacts).
  Also, the secondary camera renders game-scene entities but not world-space UI
  (bevy_ui is screen-space), so even with correct camera routing the offscreen
  frame may not match what the player sees.
- The existing offscreen path is explicitly documented as "experimental — misses
  bevy_ui layer" (`autoplay.rs:133`). This gap is the core risk.

### Likely Reliability

**Very high (≈95%) once correctly implemented** — completely independent of OS
foreground lock, DWM, screen state, or session type. This is the correct
long-term solution for headless/CI screenshot evidence. The risk is in the
implementation path, not in the underlying approach.

### Proof Test

Enable `CCGS_AUTOPLAY_OFFSCREEN=1`; run vs-bot recipe; Bevy RPC screenshots
at lobby, shop, placement, resolution phases all show distinct visible UI
matching the expected phase. No Win32 capture needed in this path.

### Ownership

`client/src/autoplay.rs` — gameplay-programmer (Rust) + liv-bevy-018 skill.
Non-trivial scope; should be its own PROMPT, not bundled with Python tooling.

---

## Option 4 — In-Game QA Snapshot JSON as Non-Image Fallback

### What it does

`client/src/presentation/qa_snapshot.rs` (`QASnapshotPlugin`) already produces
a rich `snapshot.json` bundle on F9 or button click, containing:
- `current_phase.phase` + `round` (phase label)
- `placement_state.available`, `staged_count`, `submitted`
- `auction_state.available`, `panel_state`, `card_id`
- `ui_counts.*_visible` — per-surface visible entity counts
- `extras.board.units`, `extras.hand.*`
- `extras.resolution_phase.active`, `event_count`, `per_lane_objective`
- `session_identity.session_id`, `client_state`

These fields reliably distinguish every game phase even if pixel content is
frozen, because they reflect live ECS state independent of GPU/DWM.

Two delivery paths:

**4a. Inject F9 via existing `autoplay/input` RPC** — zero Rust changes:
```json
{"method": "autoplay/input", "params": {"keys_down": ["F9"]}}
```
Followed immediately by a keys_up. QA snapshot fires, JSON is written to
`qa-snapshots/`. The driver can read the JSON as phase evidence.

**4b. New `autoplay/qa_snapshot` RPC** — minimal Rust change:
Add an `AutoplayCommand::QASnapshot` variant; in `drain_commands_system` write
a `QASnapshotRequested` message. This is cleaner than input injection and
avoids any risk of accidental F9 interaction with game UI.

### Implementation Scope

- **Option 4a**: Python-only change in `tools/autoplay/driver.py` and the
  relevant recipe — inject F9 before each screenshot tick. ~5 lines.
- **Option 4b**: `client/src/autoplay.rs` (~20 lines) + `tools/autoplay/driver.py`
  (~10 lines for a new RPC call and JSON artifact harvesting).
- **Files (4a)**: `tools/autoplay/driver.py`, recipes
- **Files (4b)**: `client/src/autoplay.rs`, `tools/autoplay/driver.py`

### Risk

- **Low.** QA snapshot is already tested, gate-checked, and in production.
  F9 injection is low-risk (the shortcut is specifically chosen to be free in
  Bevy 0.18 and unbound in the game). The JSON output is deterministic.
- Limitation: JSON evidence does NOT prove visual distinctness. It proves that
  game logic is at a different phase — a different kind of evidence. The
  composite validator currently checks image content hashes; a JSON-based
  validator would need to be added if JSON evidence is to be the primary gate.

### Likely Reliability

**Very high (≈98%)** for state-based phase evidence. Zero for pixel evidence.
Suitable as a fallback evidence layer while image capture issues are being resolved.

### Proof Test

After each checkpoint, `qa-snapshots/<id>/snapshot.json` has a different
`current_phase.phase` value (e.g. `Lobby`, `Shop`, `Placement`, `Resolution`).
A new composite validator check `DISTINCT-PHASE-LABELS` reads these and passes
if at least 3 distinct phase labels appear across the run.

### Ownership

- 4a: `tools/autoplay/driver.py` + recipe files — autoplay Python tooling owner.
- 4b: `client/src/autoplay.rs` + `tools/autoplay/driver.py`.
  Rust side needs `liv-bevy-018` skill.

---

## Option 5 — Launcher / Operator Constraints

### What it does

Add a pre-run gate in `tools/autoplay/Run-AutoplaySmoke.ps1` (and
`Start-AutoplayVsBot.ps1`) that:

1. After launching the client, polls `GetForegroundWindow` via a small Python
   helper until the game window holds focus, with a 10 s timeout.
2. Emits a loud warning (or blocks) if focus transfer fails before driver start.
3. Documents the requirement: "the game window must be the active foreground
   window at driver start; do not lock the screen, do not switch focus during
   the run."

A stronger variant: Use Windows Task Scheduler or `Start-Process -Verb RunAs`
with `AllowSetForegroundWindow(ASFW_ANY)` called from a trusted process before
launching the driver, so the driver process has foreground-set entitlement for
its lifetime.

### Implementation Scope

- **Files**: `tools/autoplay/Run-AutoplaySmoke.ps1`,
  possibly a new `tools/autoplay/win_focus_gate.py`
- **Lines changed**: ~20–40 PS1 lines + ~30 Python lines
- No Rust changes

### Risk

- **Low** for the guard check itself. It makes the existing failure visible
  immediately rather than silently producing 15 identical screenshots.
- Does NOT fix the root cause. Even with a focus-gate at launch, the driver
  process can lose foreground mid-run (Windows notifications, Codex UI, etc.).
- Useful as a diagnostic improvement regardless of which other option is chosen.

### Likely Reliability

**Low as a standalone fix** — merely a diagnostic gate. High value as a
complement to Options 1 or 2.

### Proof Test

Running `Run-AutoplaySmoke.ps1` without the game window in focus exits early
with an error message: `ERROR: game window is not foreground; cannot start driver`.

### Ownership

`tools/autoplay/Run-AutoplaySmoke.ps1` — autoplay Python/PS tooling owner.

---

## Ranked Recommendation (if PROMPT 1808 Fails)

| Rank | Option | Rationale |
|---|---|---|
| **1** | **Option 2 (desktop BitBlt)** | Eliminates the DWM dependency entirely for Win32 captures. Small, self-contained Python change in `win_capture.py`. No Rust required. Delivers distinct image evidence without needing foreground focus. Pair with `SetWindowPos(HWND_TOPMOST)` before capture for near-100% non-occluded reliability. |
| **2** | **Option 4a (F9 QA snapshot injection)** | Zero Rust changes. Immediate phase-label evidence that can distinguish all 15 checkpoints even with frozen images. Does not fix images but provides a fallback evidence layer that unblocks PROMPT validation while image fixes are developed. |
| **3** | **Option 1 (AttachThreadInput)** | If PROMPT 1808 didn't attempt AttachThreadInput (only BringWindowToTop), this is the next Win32 focus escalation. Highest probability of restoring native Bevy RPC screenshot quality. Pairs with Option 2 for belt-and-suspenders coverage. |
| **4** | **Option 3 (Bevy offscreen target)** | Long-term correct solution. Assign only after Options 1+2 are both confirmed failing, as this requires non-trivial Rust work with bevy_ui camera routing risk. |
| **5** | **Option 5 (launcher guard)** | Low-priority diagnostic improvement. Add regardless of which option is chosen, but it does not unblock screenshot distinctness on its own. |

---

## Suggested Next PROMPTs (if PROMPT 1808 Fails)

| PROMPT | Scope | Owner |
|---|---|---|
| PROMPT 1810 | Desktop BitBlt capture in `win_capture.py` — replace `PrintWindow` path with `GetDC(None)` + `BitBlt` + `SetWindowPos(HWND_TOPMOST)` toggle | `tools/autoplay/win_capture.py` |
| PROMPT 1811 | F9 QA snapshot injection in driver + JSON phase-label validator check | `tools/autoplay/driver.py` + `tools/autoplay/validate_composite_run.py` |
| PROMPT 1812 | Bevy offscreen target UI routing repair (offscreen camera captures bevy_ui) | `client/src/autoplay.rs` + `liv-bevy-018` skill |

---

1809: AUTOPLAY-SCREENSHOT-EVIDENCE-FALLBACK-OPTIONS-AUDIT: SHIPPED
