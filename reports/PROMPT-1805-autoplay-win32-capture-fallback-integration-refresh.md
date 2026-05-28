# PROMPT 1805 — Autoplay Win32 Capture Fallback Integration Refresh

**Date**: 2026-05-28  
**Branch**: `prompt-1805-win32-capture-fallback-integration`  
**Worktree**: `D:/_DEV/claude-code-game-studios-worktrees/1805-win32-capture-fallback-integration`  
**Base**: `origin/main@4eb69de4`

## Task Summary

Integration refresh: cherry-pick PROMPT 1803 Win32 capture fallback hardening
commits onto current `origin/main`, validate, and push integration branch.

## Commits Integrated

| SHA (original) | SHA (replayed) | Message |
|---|---|---|
| `42fb8721` | `6369d0bf` | feat(autoplay): PROMPT 1803 — Win32 capture fallback hardening |
| `c4e31552` | `6bb9b9c2` | docs(reports): PROMPT 1803 — Win32 capture fallback hardening report |

Cherry-picks applied cleanly with no conflicts.

## Files Changed

```
reports/PROMPT-1803-autoplay-window-foreground-win32-capture-fallback-repair.md  (+76)
tests/tools/autoplay/test_win32_capture.py                                        (+244)
tools/autoplay/driver.py                                                           (+4 -1)
tools/autoplay/win_capture.py                                                      (+15)
```

Total: 4 files, 338 insertions, 1 deletion.

## Path Allowlist Verification

All changed files are within permitted scope:
- `tools/autoplay/win_capture.py` ✓
- `tools/autoplay/driver.py` ✓
- `tests/tools/autoplay/test_win32_capture.py` ✓
- `reports/PROMPT-1803-*.md` ✓

Forbidden files not touched:
- `tools/autoplay/Run-AutoplaySmoke.ps1` — untouched ✓
- `tests/tools/autoplay/test_launcher_stale_pyc_guard.py` — untouched ✓
- No Bevy/Rust source, no production/sprint/status/session-state files ✓

## Validation

### git diff --check
```
diff --check PASS
```
No whitespace errors.

### Python tests: test_win32_capture.py
```
37 passed in 0.18s
```
All 37 tests passing:
- `TestIsAvailable` (3 tests)
- `TestWritePng` (6 tests)
- `TestCaptureGameWindow` (6 tests)
- `TestCaptureHwndToPng` (8 tests)
- `TestDriverWin32CaptureWiring` (5 tests)
- `TestCaptureHwndRestoreBeforeCapture` (4 tests)
- `TestCaptureHwndPixelHash` (2 tests)
- `TestDriverWin32CaptureOrchestration` (3 tests)

## Push Result

Branch pushed successfully:
```
origin/prompt-1805-win32-capture-fallback-integration
```
HEAD: `6bb9b9c2`

## Isolation

- Worked exclusively in dedicated worktree (not root checkout)
- No edits to root `D:\_DEV\Work\Claude-Code-Game-Studios` checkout
- Integration branch tracks `origin/main`; `main` not touched

1805: AUTOPLAY-WIN32-CAPTURE-FALLBACK-INTEGRATION-REFRESH: SHIPPED
