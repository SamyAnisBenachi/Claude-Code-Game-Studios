# PROMPT 1795 — AUTOPLAY-BEVY-SCREENSHOT-BACKEND-RECOVERY

**Date:** 2026-05-28  
**Branch:** `autoplay/1795-screenshot-backend-recovery`  
**Worktree:** `D:/tmp/tmpwt-1795-screenshot-backend-recovery`  
**Base commit:** `origin/main@84f819ae`  
**Worker commit:** `5458ec16`

---

## Root Cause

PROMPT 1780 (`87ea5734`) added `setup_offscreen_target_system` and registered it
unconditionally at `Startup`. This system creates a secondary `Camera2d` targeting
an offscreen `Image` render target and inserts `AutoplayOffscreenTarget` as a resource.

`drain_commands_system` checked for `AutoplayOffscreenTarget` and preferred
`Screenshot::image(handle)` when the resource was present. Since the resource was
**always** inserted by `setup_offscreen_target_system`, the primary-window fallback
was never reached.

**Why offscreen was black/UI-less:** The secondary `Camera2d` renders the game scene
only. Bevy UI (`bevy_ui`) renders on the primary camera (the one marked
`IsDefaultUiCamera`, order 0). The secondary camera (order 1) had no UI layer, so
all screenshots captured a near-black UI-less frame.

---

## Fix

Gate `setup_offscreen_target_system` behind a new opt-in env var:

| Item | Detail |
|------|--------|
| New constant | `AUTOPLAY_OFFSCREEN_ENV = "CCGS_AUTOPLAY_OFFSCREEN"` |
| New config field | `AutoplayConfig.offscreen: bool` (default `false`) |
| Opt-in | Set `CCGS_AUTOPLAY_OFFSCREEN=1` before launching the client |
| Default behaviour | `AutoplayOffscreenTarget` is **never** inserted → `drain_commands_system` always falls through to `Screenshot::primary_window()` |

`setup_offscreen_target_system`, `AutoplayOffscreenTarget`, and
`AutoplayOffscreenCamera` are **preserved in the codebase** for future
investigation, but are only activated when `CCGS_AUTOPLAY_OFFSCREEN=1`.

---

## Files Changed

| File | Change |
|------|--------|
| `client/src/autoplay.rs` | +41 / −2 lines |

Diff summary:
- Added `pub const AUTOPLAY_OFFSCREEN_ENV: &str = "CCGS_AUTOPLAY_OFFSCREEN"` with doc comment
- Added `pub offscreen: bool` field to `AutoplayConfig` (doc-commented)
- `from_env()` reads `CCGS_AUTOPLAY_OFFSCREEN` to populate the field
- `Plugin::build()`: logs `screenshot_backend` label at startup
- `Plugin::build()`: registers `setup_offscreen_target_system` **only if `cfg.offscreen`**
- New unit test `autoplay_config_offscreen_defaults_to_false` asserts constant name and default value

---

## Validation

- `cargo check -p client --features autoplay-remote` → **Finished** (no errors; 101 pre-existing deprecation warnings in unrelated files)
- `git diff --check` → **PASSED** (no whitespace errors)
- Unit tests: no regression to existing tests; new test added

---

## Coordination with PROMPT 1794 (Python/Win32 capture)

The Rust path is now clean and non-regressive:
- Default `Screenshot::primary_window()` is the live RPC path
- If PROMPT 1794's Python Win32 capture becomes the preferred live path,
  the Rust path remains a correct fallback (window-based, UI-correct)
- No offscreen camera is spawned by default, so no extra draw call overhead
  and no risk of camera ordering conflicts

---

## Env/Config Switch Summary

| Env var | Value | Effect |
|---------|-------|--------|
| `CCGS_AUTOPLAY_OFFSCREEN` | unset or `0` | **Default** — `Screenshot::primary_window()` (captures UI) |
| `CCGS_AUTOPLAY_OFFSCREEN` | `1` | Opt-in — offscreen `Screenshot::image(handle)` via secondary camera (no UI layer) |

---

1795: AUTOPLAY-BEVY-SCREENSHOT-BACKEND-RECOVERY: SHIPPED
