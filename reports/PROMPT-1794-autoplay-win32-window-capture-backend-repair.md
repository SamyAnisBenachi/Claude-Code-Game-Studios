# PROMPT 1794 — AUTOPLAY-WIN32-WINDOW-CAPTURE-BACKEND-REPAIR

**Date:** 2026-05-28  
**Branch:** `prompt-1794-win32-capture`  
**Commit:** `757698b4a1282d57ba5b4014b2a9f375f3194079`  
**Worktree:** `D:\_DEV\Work\Claude-Code-Game-Studios\tmpwt-1794-win32-capture`  
**Base:** `origin/main@84f819ae`

---

## Summary

Implemented a Windows-native driver-side screenshot capture backend for autoplay evidence.

Root cause addressed: PROMPT 1792 confirmed that Bevy's `Screenshot::primary_window` and
`Screenshot::image` capture the GPU backbuffer, which is near-black and byte-identical
when the window is offscreen or not actively composited by DWM. The existing RPC path
cannot observe what DWM has composited.

Solution: `PrintWindow(hwnd, mem_dc, PW_RENDERFULLCONTENT)` + `GetDIBits` captures the
DWM-composited surface regardless of window visibility or GPU backbuffer state. The result
is converted BGRA→RGB and written as a valid PNG using stdlib `zlib`/`struct` only.

---

## Files Changed

| File | Change |
|------|--------|
| `tools/autoplay/win_capture.py` | **NEW** — Win32 GDI capture backend (203 lines) |
| `tools/autoplay/driver.py` | **MODIFIED** — imports `_win32_capture`, calls it after `ensure_foreground()` |
| `tests/tools/autoplay/test_win32_capture.py` | **NEW** — 28 focused unit tests |

---

## Implementation Details

### `tools/autoplay/win_capture.py`

- **`is_available() -> bool`** — True if `sys.platform == "win32"`
- **`capture_game_window(output_path, log) -> bool`** — public entry point; discovers HWND
  via `win_foreground._list_visible_windows` / `_find_candidate` (no title-hint duplication),
  delegates to `_capture_hwnd_to_png`
- **`_capture_hwnd_to_png(hwnd, output_path, log, *, user32, gdi32) -> bool`** — GDI pipeline:
  `GetWindowRect` → `GetDC` → `CreateCompatibleDC/Bitmap` → `PrintWindow(PW_RENDERFULLCONTENT)`
  (fallback to `flags=0`) → `GetDIBits` → BGRA→RGB conversion → `_write_png`
- **`_write_png(path, width, height, rgb_rows)`** — minimal PNG encoder: IHDR + filter-0 IDAT
  (zlib level 6) + IEND; no external deps
- All injected `user32`/`gdi32` params for unit-test isolation
- Graceful `except Exception` swallows all errors and logs; non-Windows is a clean no-op

### `tools/autoplay/driver.py` change

```python
from win_capture import capture_game_window as _win32_capture  # import added

# in the autoplay/screenshot branch, after ensure_foreground(log):
_win32_shot = artifact_dir / f"win32_tick_{tick:06d}.png"
_win32_capture(_win32_shot, log)
```

- Win32 capture fires **before** the Bevy RPC to capture the clean DWM-composited state
- RPC screenshot path unchanged — still fires, file-ready poll still runs
- Default backend unchanged: RPC screenshot is the canonical Bevy artifact;
  Win32 capture is a supplemental evidence file (`win32_tick_NNNNNN.png`)
- `capture_game_window` is a no-op on non-Windows, so CI / Linux / WASM paths are unaffected

---

## Tests Run

```
pytest tests/tools/autoplay/ -v
```

**Result: 204 passed in 1.21s** (28 new + 176 existing — zero regressions)

Coverage areas:
- `TestIsAvailable` (3 tests) — platform guard
- `TestWritePng` (6 tests) — PNG signature, IHDR dimensions, color type, IDAT decompressibility
- `TestCaptureGameWindow` (6 tests) — non-Windows no-op, no-window, exception swallow, happy-path delegation
- `TestCaptureHwndToPng` (8 tests) — GetWindowRect/GetDC/PrintWindow/GetDIBits failure modes, happy path, PW retry log
- `TestDriverWin32CaptureWiring` (5 tests) — structural checks: import present, call after foreground barrier, tick in path

---

## Default Backend Changed?

**No.** The RPC screenshot remains the primary Bevy-side evidence artifact.  
Win32 capture is **supplemental**: it writes `win32_tick_NNNNNN.png` alongside the RPC
file. Both are visible in `driver.log`. The Win32 capture is the better evidence for
diagnosing near-black frames because it captures the DWM compositor output.

On non-Windows the driver behaviour is byte-for-byte identical to pre-1794.

---

## Validation

- `git diff --check` — CLEAN (no trailing whitespace)
- `pytest tests/tools/autoplay/ -v` — 204/204 PASSED
- No Cargo suites run (Python-only tooling change as scoped)
- Branch pushed: `origin/prompt-1794-win32-capture`

---

1794: AUTOPLAY-WIN32-WINDOW-CAPTURE-BACKEND-REPAIR: SHIPPED
