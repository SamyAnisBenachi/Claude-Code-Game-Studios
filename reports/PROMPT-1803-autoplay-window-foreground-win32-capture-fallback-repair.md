# PROMPT 1803 — Autoplay Window Foreground / Win32 Capture Fallback Repair

## Worktree

- **Path**: `D:\_DEV\Work\Claude-Code-Game-Studios\tmpwt-1803-win32-capture-fallback`
- **Branch**: `prompt-1803-win32-capture-fallback-repair`
- **Base commit**: `4eb69de4` (docs(reports): PROMPT 1799 — autoplay screenshot validator mainland whitespace fix report)
- **Implementation commit**: `42fb8721` (feat(autoplay): PROMPT 1803 — Win32 capture fallback hardening)

## Summary of Changes

### `tools/autoplay/win_capture.py`

1. Added module-level constants `_SW_RESTORE = 9` and `_SW_SHOWNOACTIVATE = 4`
   alongside the existing GDI constants.
2. In `_capture_hwnd_to_png`, before `GetWindowRect`:
   - Calls `user32.ShowWindow(hwnd, _SW_RESTORE)` to un-minimize the window.
   - Calls `user32.SetForegroundWindow(hwnd)` to attempt foreground promotion.
   - Logs both return values:
     `win32_capture: ShowWindow ret={ret} hwnd={hwnd:#010x}`
     `win32_capture: SetForegroundWindow ret={ret} hwnd={hwnd:#010x}`
   - Capture proceeds regardless of `SetForegroundWindow` return value (0 is
     acceptable in non-interactive sessions).
3. After `GetDIBits` succeeds, computes a pixel distinguishability hash:
   `pixel_hash = zlib.adler32(raw[:min(4096, len(raw))]) & 0xFFFFFFFF`
   and logs: `win32_capture: pixel_hash={hash:#010x} width={width} height={height}`
   This allows byte-identical capture detection in `driver.log`.

### `tools/autoplay/driver.py`

1. After `ensure_foreground(log)`, added `time.sleep(0.12)` so DWM has time to
   composite the foregrounded window before the capture fires.
2. Changed `_win32_capture(_win32_shot, log)` to
   `_win32_ok = _win32_capture(_win32_shot, log)` to capture the return value.
3. Added a log line immediately after:
   `log(f"tick={tick} win32_capture={'OK' if _win32_ok else 'FAILED'} path={_win32_shot.name}")`

### `tests/tools/autoplay/test_win32_capture.py`

Added three new test classes at the end of the file:

**`TestCaptureHwndRestoreBeforeCapture`** (4 tests):
- `test_win32_capture_hwnd_calls_show_window_before_printwindow` — call-order
  tracking confirms ShowWindow precedes PrintWindow.
- `test_win32_capture_hwnd_logs_show_window_result` — log contains "ShowWindow".
- `test_win32_capture_hwnd_logs_setforegroundwindow_result` — log contains
  "SetForegroundWindow".
- `test_win32_capture_hwnd_proceeds_even_when_setforeground_returns_zero` —
  capture returns True even when SetForegroundWindow returns 0.

**`TestCaptureHwndPixelHash`** (2 tests):
- `test_win32_capture_hwnd_logs_pixel_hash_on_success` — log contains "pixel_hash=".
- `test_win32_capture_hwnd_pixel_hash_differs_between_captures` — captures with
  different fill values produce different hashes.

**`TestDriverWin32CaptureOrchestration`** (3 tests):
- `test_driver_captures_win32_return_value` — `_win32_ok = _win32_capture(` present.
- `test_driver_logs_win32_capture_result` — `win32_capture=` present in source.
- `test_driver_has_dwm_settle_sleep_after_ensure_foreground` — `time.sleep(0.12)`
  appears between `ensure_foreground` and `_win32_capture` in source order.

## Test Results

```
57 passed in 0.32s
```

All 37 pre-existing tests continue to pass. All 9 new tests pass on first run.

## git diff --check

No whitespace issues — command produced no output (clean).

## Final Status

1803: AUTOPLAY-WINDOW-FOREGROUND-WIN32-CAPTURE-FALLBACK-REPAIR: SHIPPED
