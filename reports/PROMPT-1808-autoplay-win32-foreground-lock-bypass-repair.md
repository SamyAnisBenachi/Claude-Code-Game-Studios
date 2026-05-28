# PROMPT 1808 — AUTOPLAY-WIN32-FOREGROUND-LOCK-BYPASS-REPAIR

**Status:** SHIPPED
**Date:** 2026-05-28 UTC
**Worktree:** `tmpwt-1808-win32-foreground-lock-bypass`
**Branch:** `feat/1808-win32-foreground-lock-bypass`
**Commit:** `f5ef1aa9`
**Base:** `origin/main@d6aabbdd`

---

## Problem Summary (from PROMPT 1807)

`SetForegroundWindow` returned **0 on every tick** across a full 15-checkpoint
vs-bot run. Because the Python driver process never owned the Windows foreground
lock, DWM served a frozen DWM-composited frame to both:

- `PrintWindow(PW_RENDERFULLCONTENT)` — Win32 driver-side captures (all 47377 bytes,
  pixel_hash `0x26207c4c` identical across all 15 ticks)
- Bevy RPC `autoplay/screenshot` — GPU backbuffer captures (all 86148 bytes, MD5
  `34556a68da323f8a3824e87f9ea3f00b` identical across all 15 ticks)

The game logic progressed correctly (all 15 checkpoints reached), but the screenshot
evidence was frozen at the first frame captured.

---

## Root Cause

Windows foreground lock (`SPI_GETFOREGROUNDLOCKTIMEOUT`): `SetForegroundWindow` is
blocked when the calling process is not the current foreground process. A console-mode
Python process launched from PowerShell has no HWND and cannot use the conventional
`AllowSetForegroundWindow` pattern (which requires the target process to consent).

The prior fallback (`BringWindowToTop`) does not transfer focus — it only adjusts
Z-order — so DWM continued to serve the stale composition buffer.

---

## API Sequence Implemented

New function `_foreground_window_robust(user32, kernel32, hwnd, log)` in
`tools/autoplay/win_foreground.py`:

### Step 1 — Thread identification
```python
current_hwnd_fg = user32.GetForegroundWindow()
current_thread  = kernel32.GetCurrentThreadId()
fg_thread       = user32.GetWindowThreadProcessId(current_hwnd_fg, None)
target_thread   = user32.GetWindowThreadProcessId(hwnd, None)
```
Logged: `foreground_robust: hwnd=0x... current_fg=0x... fg_thread=N target_thread=M current_thread=K`

### Step 2 — AttachThreadInput bypass
```python
user32.AttachThreadInput(current_thread, fg_thread, True)   # our thread → fg thread
user32.AttachThreadInput(target_thread,  fg_thread, True)   # target thread → fg thread
```
After attaching, Windows considers both threads foreground-capable, enabling
`SetForegroundWindow` to succeed from a non-foreground process.

Guards: skipped when `fg_thread == 0` (no foreground window) or
`fg_thread == current_thread` (already same thread). Always detached in `finally`.

### Step 3 — Z-order manipulation
```python
user32.ShowWindow(hwnd, SW_RESTORE)
user32.SetWindowPos(hwnd, HWND_TOPMOST,    0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE)
user32.SetWindowPos(hwnd, HWND_NOTOPMOST,  0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE)
```
Brings window to Z-top without permanently making it always-on-top.

### Step 4 — Primary SetForegroundWindow + activation
```python
sfg_ret = user32.SetForegroundWindow(hwnd)
if sfg_ret:
    user32.SetFocus(hwnd)
    user32.SetActiveWindow(hwnd)
    return True
```
`SetFocus` and `SetActiveWindow` together trigger DWM re-composition of fresh GPU frames.

### Step 5 — Synthetic ALT key fallback
```python
user32.keybd_event(0x12, 0, 0, 0)          # VK_MENU down
user32.keybd_event(0x12, 0, 0x0002, 0)     # VK_MENU up
sfg_ret2 = user32.SetForegroundWindow(hwnd)
```
A key event briefly satisfies the foreground guard independently of thread attachment.
Used only when Step 4 fails. Isolated to a single down+up pair; no visible side-effect.

### Step 6 — Detach (always in finally)
```python
user32.AttachThreadInput(current_thread, fg_thread, False)
user32.AttachThreadInput(target_thread,  fg_thread, False)
```

### Failure path
When all attempts fail, logs `foreground_robust: all foreground attempts failed` and
returns `False` — capture continues with the last DWM-composited frame (same behavior
as before the fix, with better logging).

---

## Logging Fields Added

Every call and return value is logged, enabling exact failure path diagnosis:

| Log line | Meaning |
|---|---|
| `foreground_robust: hwnd=... current_fg=... fg_thread=N target_thread=M current_thread=K` | Thread topology diagnostic |
| `foreground_robust: already foreground — ShowWindow SW_RESTORE OK` | Fast path taken |
| `foreground_robust: AttachThreadInput(current=K->fg=N) ret=1` | Attach succeeded |
| `foreground_robust: AttachThreadInput(current=K->fg=N) ret=0` | Attach failed (may still proceed) |
| `foreground_robust: SetWindowPos TOPMOST->NOTOPMOST hwnd=...` | Z-order manipulation |
| `foreground_robust: SetForegroundWindow ret=1 hwnd=...` | Primary attempt result |
| `foreground_robust: SetFocus+SetActiveWindow OK hwnd=...` | Focus transfer complete |
| `foreground_robust: ... trying synthetic ALT key fallback` | Primary failed, trying ALT |
| `foreground_robust: SetForegroundWindow after ALT key ret=1 hwnd=...` | ALT fallback result |
| `foreground_robust: all foreground attempts failed hwnd=...` | All paths exhausted |
| `foreground_robust: AttachThreadInput detach(current=K->fg=N)` | Cleanup confirmation |

---

## Files Changed

| File | Change |
|---|---|
| `tools/autoplay/win_foreground.py` | Added `_foreground_window_robust`; updated `ensure_foreground` to call it with `kernel32`; `_foreground_window` kept unchanged |
| `tests/tools/autoplay/test_win_foreground.py` | Added `TestForegroundWindowRobust` (20 tests); updated `TestEnsureForeground` (3 tests) to patch `_foreground_window_robust`; added `_make_kernel32` helper |

**`win_capture.py`**: Not modified — `_capture_hwnd_to_png` retains its own defensive
`ShowWindow`/`SetForegroundWindow` calls for defense-in-depth. The fix is in
`ensure_foreground` which is called before `_win32_capture` in `driver.py`.

**`driver.py`**: Not modified — existing `ensure_foreground(log)` + `time.sleep(0.12)` +
`_win32_capture()` sequence is preserved; the robust bypass is transparent.

---

## Test Results

```
tests/tools/autoplay/test_win_foreground.py     59 passed
Full suite (tests/tools/autoplay/)             261 passed
```

`git diff --check`: no whitespace errors (LF→CRLF normalization warnings only, Windows).

---

## New Tests (TestForegroundWindowRobust — 20 tests)

| Test | Verifies |
|---|---|
| `test_win_foreground_robust_already_foreground_returns_true` | Fast path when hwnd == current_fg |
| `test_win_foreground_robust_already_foreground_calls_show_window` | SW_RESTORE on fast path |
| `test_win_foreground_robust_attaches_to_fg_thread` | AttachThreadInput(current→fg) called |
| `test_win_foreground_robust_detaches_fg_thread_in_finally` | AttachThreadInput(current→fg, False) in finally |
| `test_win_foreground_robust_detaches_even_when_setforeground_fails` | Detach happens even on failure |
| `test_win_foreground_robust_attaches_target_thread_to_fg` | AttachThreadInput(target→fg) called |
| `test_win_foreground_robust_calls_setwindowpos_topmost_then_notopmost` | TOPMOST→NOTOPMOST sequence |
| `test_win_foreground_robust_returns_true_when_setforeground_succeeds` | Happy path returns True |
| `test_win_foreground_robust_calls_setfocus_on_success` | SetFocus called on success |
| `test_win_foreground_robust_calls_setactivewindow_on_success` | SetActiveWindow called on success |
| `test_win_foreground_robust_logs_setforeground_return_value` | Return value always logged |
| `test_win_foreground_robust_tries_alt_key_when_setforeground_fails` | ALT key sent as fallback |
| `test_win_foreground_robust_alt_key_sends_vk_menu` | ALT key uses VK_MENU=0x12 |
| `test_win_foreground_robust_logs_alt_key_fallback_attempt` | ALT fallback logged |
| `test_win_foreground_robust_returns_false_when_all_attempts_fail` | Returns False when all fail |
| `test_win_foreground_robust_capture_proceeds_when_all_fail` | No exception on total failure |
| `test_win_foreground_robust_returns_false_on_os_error` | OSError caught, returns False |
| `test_win_foreground_robust_logs_thread_ids_for_diagnosis` | Thread IDs in first log line |
| `test_win_foreground_robust_skips_attach_when_fg_thread_equals_current` | No self-attach |
| `test_win_foreground_robust_skips_attach_when_fg_hwnd_is_zero` | No attach with zero fg |

---

## Worktree / Branch / Commit

| Field | Value |
|---|---|
| Worktree path | `D:\_DEV\Work\Claude-Code-Game-Studios\tmpwt-1808-win32-foreground-lock-bypass` |
| Branch | `feat/1808-win32-foreground-lock-bypass` |
| Commit | `f5ef1aa9` |
| Base | `origin/main@d6aabbdd` |
| Push | `origin/feat/1808-win32-foreground-lock-bypass` ✓ |

---

1808: AUTOPLAY-WIN32-FOREGROUND-LOCK-BYPASS-REPAIR: SHIPPED
