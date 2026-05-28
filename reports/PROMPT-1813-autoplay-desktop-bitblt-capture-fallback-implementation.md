# PROMPT 1813 — Autoplay Desktop BitBlt Capture Fallback Implementation

**Date**: 2026-05-28
**Branch**: `feat/1813-desktop-bitblt-capture`
**Commit**: `969ca093`
**Worktree**: `tmpwt-1813-desktop-bitblt-capture`

---

## Summary

Implemented the `desktop_bitblt` capture backend for the autoplay screenshot
pipeline and wired it as an automatic fallback when `win32_printwindow` fails.
All 62 tests pass; `git diff --check` clean.

---

## Problem

PROMPT 1807/1809 live verifies showed that `PrintWindow` (the existing
`win32_printwindow` backend) returned non-blank but byte-identical/frozen frames
for the backgrounded game window.  PROMPT 1809 recommended desktop BitBlt via
`GetDC(NULL)` over the window screen rect as the next fallback, because it reads
DWM's composed screen pixels directly rather than a potentially stale window
backbuffer.

---

## Implementation

### `tools/autoplay/win_capture.py`

Added:

- **`_SRCCOPY = 0x00CC0020`** — raster-op constant for `BitBlt` direct copy.
- **`_capture_hwnd_bitblt_to_png(hwnd, output_path, log, *, user32, gdi32)`** —
  inner GDI pipeline using desktop DC:
  1. `GetWindowRect(hwnd)` → screen-space `(left, top, right, bottom)`
  2. `GetDC(None)` → desktop DC (all composed screen pixels, including DWM output)
  3. `CreateCompatibleDC` + `CreateCompatibleBitmap(desktop_dc, w, h)`
  4. `SelectObject` → select bitmap into mem DC
  5. `BitBlt(mem_dc, 0, 0, w, h, desktop_dc, rect.left, rect.top, SRCCOPY)`
  6. `GetDIBits` → extract BGRA pixels
  7. Blank-image guard (all-zero pixels → failure, logged)
  8. BGRA→RGB conversion + `_write_png`
  9. DC/object cleanup in `finally` blocks (no handle leaks)
- **`capture_game_window_desktop_bitblt(output_path, log)`** — public entry
  point; same HWND-discovery path as `capture_game_window`, delegates to
  `_capture_hwnd_bitblt_to_png`.  All log lines prefixed `desktop_bitblt:`.

Log labels used to distinguish backends:
- `win32_printwindow:` — existing PrintWindow backend (log prefix updated)
- `desktop_bitblt:` — new BitBlt backend

### `tools/autoplay/driver.py`

- Import `capture_game_window_desktop_bitblt as _desktop_bitblt_capture`.
- After `_win32_ok = _win32_capture(...)`, if `not _win32_ok`, attempt
  `_desktop_bitblt_capture(...)` and log `desktop_bitblt=OK/FAILED`.
- Log label for the first attempt renamed `win32_printwindow=OK/FAILED` (was
  `win32_capture=`), making the two backends distinguishable in run logs.

**Fallback wiring** (deterministic selection for the next live verify):

```
ensure_foreground(log)
sleep(0.12)                        # DWM settle
win32_printwindow attempt          # → win32_tick_NNNNNN.png
if win32_printwindow FAILED:
    desktop_bitblt attempt         # → bitblt_tick_NNNNNN.png
Bevy RPC autoplay/screenshot       # unchanged
```

The default path is unchanged: `win32_printwindow` runs first and `bevy_rpc`
always fires regardless.  `desktop_bitblt` is only active when `win32_printwindow`
returns `False` — no behaviour change for runs where PrintWindow succeeds.

---

## Validation

### `git diff --check`

```
(no output — clean)
```

### pytest

```
62 passed in 0.20s
```

New test classes (24 tests):
- `TestCaptureHwndBitblt` (14 tests) — `_capture_hwnd_bitblt_to_png` failure
  paths (GetWindowRect, zero-size, GetDC(NULL), CreateCompatibleDC,
  CreateCompatibleBitmap, BitBlt, GetDIBits, blank-image) + happy paths
  (PNG written, pixel_hash logged, PNG written logged, screen-origin assertion,
  DC release on failure).
- `TestCaptureGameWindowDesktopBitblt` (5 tests) — public API: non-Windows
  no-op, no window found, exception swallowed, delegates to inner function,
  logs `desktop_bitblt:` prefix.
- `TestDriverDesktopBitbltFallback` (7 tests) — structural: import present,
  alias present, fallback inside `if not _win32_ok:` guard, log label, tick
  filename, placement after win32 attempt, `win32_printwindow=` label.

Updated test (1):
- `TestDriverWin32CaptureOrchestration::test_driver_logs_win32_capture_result`
  updated to check for `win32_printwindow=` (matches renamed log label).

---

## Files Changed

| File | Change |
|------|--------|
| `tools/autoplay/win_capture.py` | +`_SRCCOPY`, +`_capture_hwnd_bitblt_to_png`, +`capture_game_window_desktop_bitblt`, updated log prefixes in `capture_game_window` |
| `tools/autoplay/driver.py` | +import `_desktop_bitblt_capture`, +fallback block after `_win32_ok`, log label renamed `win32_printwindow=` |
| `tests/tools/autoplay/test_win32_capture.py` | +24 new tests across 3 new classes, 1 existing test updated |

---

## Selecting the Backend for the Next Live Verify

The `desktop_bitblt` backend **activates automatically** when `win32_printwindow`
fails or returns `False`.  To exercise it deterministically in a live verify:

1. If `win32_printwindow` is expected to fail (frozen/blank), `desktop_bitblt`
   will fire on its own — look for `desktop_bitblt=OK/FAILED` in `driver.log`.
2. To force `desktop_bitblt` without a PrintWindow failure, temporarily patch
   `_win32_capture` to always return `False` in the driver, or run
   `capture_game_window_desktop_bitblt` directly from a one-shot script.
3. Evidence files: `win32_tick_NNNNNN.png` (PrintWindow) vs
   `bitblt_tick_NNNNNN.png` (BitBlt fallback) — compare pixel hashes in the log.

---

## Ready for Integration

Yes — implementation is self-contained, all existing tests pass, 24 new tests
added, no Rust/Cargo files touched, worktree on `feat/1813-desktop-bitblt-capture`
pushed to origin.

---

1813: AUTOPLAY-DESKTOP-BITBLT-CAPTURE-FALLBACK-IMPLEMENTATION: SHIPPED
