# PROMPT-1776 — Autoplay Screenshot Window Foreground Repair

**Date**: 2026-05-28  
**Branch**: wt/1776-autoplay-screenshot-foreground  
**Scope**: Python tooling only — no Rust, no Cargo, no gameplay code changed.

---

## Problem

PROMPT 1775 live-verify confirmed `status.frame` advances correctly (~237 fps)
and the composite run exits 0, but all screenshots in the run were byte-identical.
Root cause: the Bevy primary window is not the active composited surface when
`Screenshot::primary_window()` fires. The GPU backbuffer for a background window
is stale (or identical to the previous frame) on Windows DWM, even though the
Bevy ECS is ticking at full speed.

PROMPT 1774 already inserted `WinitSettings::game()` to force continuous rendering
when the window loses focus, but that fixes the ECS tick rate, not the GPU
composition issue when the window is genuinely background.

---

## Solution

Add a Windows-only foreground helper that calls `SetForegroundWindow` (with
`ShowWindow` + `BringWindowToTop` fallback) before each `autoplay/screenshot`
RPC, so the Bevy window is the active composited surface when the capture fires.

---

## Changed Files

### New: `tools/autoplay/win_foreground.py`

Public API: `ensure_foreground(log: Callable[[str], None]) -> None`

- No-op on non-Windows platforms.
- Uses stdlib `ctypes` only — no third-party dependencies.
- Enumerates visible top-level windows via `user32.EnumWindows`.
- Matches the first window whose title contains any of:
  `"ccgs"`, `"claude code game"`, `"bevy app"`, `"bevy"` (case-insensitive).
- Calls `ShowWindow(hwnd, SW_RESTORE)` then `SetForegroundWindow(hwnd)`.
- Falls back to `BringWindowToTop(hwnd)` when `SetForegroundWindow` returns 0
  (Windows focus-theft rules can block it silently).
- Swallows all exceptions with a log line; never crashes the driver.
- Every branch emits at least one log line to `driver.log`.

**Layered design for testability:**
- `_find_candidate(windows)` — pure Python, no ctypes; directly unit-testable.
- `_list_visible_windows(user32)` — thin Win32 wrapper; `user32` injectable.
- `_foreground_window(user32, hwnd, log)` — Win32 calls; `user32` injectable.
- `ensure_foreground(log)` — orchestrates; wires up real `ctypes.windll.user32`.

### Modified: `tools/autoplay/driver.py`

Added import:
```python
from win_foreground import ensure_foreground
```

Added call inside the `autoplay/screenshot` branch, after the existing
frame-advance barrier (PROMPT 1766) and before the RPC:
```python
ensure_foreground(log)
```

The call is scoped strictly to `method == "autoplay/screenshot"` — no other
RPC methods are affected.

### New: `tests/tools/autoplay/test_win_foreground.py`

25 tests across 3 classes:

| Class | Tests | What they cover |
|---|---|---|
| `TestFindCandidate` | 10 | Pure-Python window matching: empty list, no match, CCGS title, Bevy App title, case-insensitivity, first-match, substring, constant validity |
| `TestForegroundWindow` | 8 | Mocked user32: ShowWindow called, SetForegroundWindow called, return True on success, log OK message, fallback to BringWindowToTop on zero, log fallback, return False on OSError, log error |
| `TestEnsureForeground` | 7 | Monkeypatched `_IS_WINDOWS`: no-op on non-Windows, log emitted, no-window-found log, foreground called on match, title logged, exception swallowed, window-count in log |

All 25 tests pass. All 147 tests in the full `tests/tools/autoplay/` suite pass.

---

## Limits and Known Risks

| Risk | Mitigation |
|---|---|
| `SetForegroundWindow` blocked by Windows focus rules | `BringWindowToTop` fallback; both logged so driver.log captures outcome |
| Bevy window title not matching hints | Hints cover "ccgs", "bevy app", "bevy"; no-match logs count of scanned windows for diagnosis |
| Window foreground on its own may not be enough if DWM is throttling | This is the minimal fix; a follow-up VERIFY run will confirm distinctness |
| Multi-window sessions | `_find_candidate` returns the first match; acceptable for single-client autoplay |

---

## Live Validation

Live GUI smoke not run by this worker (no running client available headlessly).
A follow-up VERIFY worker should rerun the vs-bot composite to confirm screenshots
are now distinct. All automated Python tests pass.

---

## Test Command

```
D:/_APPS/Python312/python.exe -m pytest tests/tools/autoplay/test_win_foreground.py -v
D:/_APPS/Python312/python.exe -m pytest tests/tools/autoplay/ -v
```

Result: **147 passed, 0 failed, 0 errors** in 0.75s.

---

1776: AUTOPLAY-SCREENSHOT-WINDOW-FOREGROUND-REPAIR: SHIPPED
