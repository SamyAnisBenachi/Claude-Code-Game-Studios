# PROMPT 1838 — POST-1830-AUTOPLAY-TOOLING-FOCUSED-VERIFY

**Date**: 2026-05-28  
**Branch**: worker/1838-post-1830-autoplay-verify  
**Base commit**: origin/main@71484998  
**Scope**: tools/autoplay/**, tests/tools/autoplay/** — verify only, no source edits

> **Historical note (added by PROMPT 1862 backfill):** This report covers the
> post-1830 tooling baseline as of origin/main@71484998. It does NOT cover the
> analyze_evidence_run.py analyzer added in PROMPT 1833 or the viewport/click-target
> audit in PROMPT 1844; those changes have their own dedicated reports.

---

## 1. Python Compile Check

All autoplay tool modules and recipe modules compiled without errors.

**Modules checked** (`py_compile`):
- `tools/autoplay/driver.py` ✅
- `tools/autoplay/rpc.py` ✅
- `tools/autoplay/screenshot_poll.py` ✅
- `tools/autoplay/validate_composite_run.py` ✅
- `tools/autoplay/win_capture.py` ✅
- `tools/autoplay/win_foreground.py` ✅
- `tools/autoplay/recipes/__init__.py` ✅
- `tools/autoplay/recipes/_builder.py` ✅
- `tools/autoplay/recipes/_coords.py` ✅
- `tools/autoplay/recipes/add_bot_lobby.py` ✅
- `tools/autoplay/recipes/class_select.py` ✅
- `tools/autoplay/recipes/draft_auction_probe.py` ✅
- `tools/autoplay/recipes/full_game.py` ✅
- `tools/autoplay/recipes/game_over_observe.py` ✅
- `tools/autoplay/recipes/idle.py` ✅
- `tools/autoplay/recipes/lobby_create.py` ✅
- `tools/autoplay/recipes/placement_drag_probe.py` ✅
- `tools/autoplay/recipes/resolution_observe.py` ✅
- `tools/autoplay/recipes/round_loop.py` ✅
- `tools/autoplay/recipes/smoke.py` ✅
- `tools/autoplay/recipes/vs_bot.py` ✅

**Result: PASS (21/21 modules)**

---

## 2. Pytest — tests/tools/autoplay/

```
============================= test session starts =============================
collected 300 items

tests/tools/autoplay/test_driver_screenshot_barrier.py     ... (all PASSED)
tests/tools/autoplay/test_launcher_stale_pyc_guard.py      ... (all PASSED)
tests/tools/autoplay/test_recipe_static.py                 ... (all PASSED)
tests/tools/autoplay/test_screenshot_quality.py            ... (all PASSED)
tests/tools/autoplay/test_validate_composite_run.py        ... (all PASSED)
tests/tools/autoplay/test_win32_capture.py                 ... (all PASSED)
tests/tools/autoplay/test_win_foreground.py                ... (all PASSED)

============================= 300 passed in 2.19s =============================
```

**Result: PASS (300/300 tests)**

---

## 3. Recipe Registry List

12 recipes registered and importable:

```
add-bot-lobby
class-select
draft-auction-probe
full-game
game-over-observe
idle
lobby-create
placement-drag-probe
resolution-observe
round-loop
smoke
vs-bot
```

**Result: PASS — 12 recipes, registry loads without error**

---

## 4. Static String Checks

### 4a. `win32_printwindow`
Found in:
- `tools/autoplay/driver.py` — log labels (`win32_printwindow_failed`, `win32_printwindow=OK/FAILED/FROZEN`) at lines 90, 100, 204, 328, 335, 341
- `tools/autoplay/win_capture.py` — function docstring/log lines at lines 380, 386, 399, 404, 407, 420

**Result: PRESENT ✅ — in driver + win_capture as expected**

### 4b. `desktop_bitblt`
Found in:
- `tools/autoplay/driver.py` — import alias `_desktop_bitblt_capture` (line 57), fallback trigger logic (lines 329, 342, 346, 348)
- `tools/autoplay/win_capture.py` — function `capture_game_window_desktop_bitblt` definition (line 411) and ~20 log lines

**Result: PRESENT ✅ — fallback wired in driver, implementation in win_capture**

### 4c. `CCGS_AUTOPLAY_BOT_ROOM_READY`
Found in:
- `tools/autoplay/Run-AutoplaySmoke.ps1` — env gate check at line 62, BLOCKED message at line 64, hint at line 71
- `tools/autoplay/recipes/full_game.py` — `BOT_ROOM_ENV` constant (line 65), guard block
- `tools/autoplay/recipes/round_loop.py` — `_BOT_ROOM_ENV` constant (line 63), guard block
- `tools/autoplay/recipes/vs_bot.py` — `_BOT_ROOM_ENV` constant (line 50), guard block
- `tools/autoplay/README.md` — documentation for `round-loop` and `full-game` recipes

**Result: PRESENT ✅ — env gate consistent across ps1 script and all bot-requiring recipes**

### 4d. Stale-pyc guard
Found in:
- `tools/autoplay/Run-AutoplaySmoke.ps1` — lines 138–151: `__pycache__` removal loop + `PYTHONDONTWRITEBYTECODE=1` set
- `tools/dev-launcher/Start-AutoplayVsBot.ps1` — lines 312–321: `[stale-pyc-guard]` labeled clear loop + `PYTHONDONTWRITEBYTECODE=1` set

**Result: PRESENT ✅ — guard present in both smoke runner and dev-launcher**

---

## 5. git diff --check

```
EXIT=0
```

**Result: PASS — no trailing whitespace issues**

---

## Summary

| Check | Result | Detail |
|---|---|---|
| Python compile (21 modules) | ✅ PASS | All tools + all recipes compile clean |
| pytest 300 tests | ✅ PASS | 300/300 passed, 2.19s |
| Recipe registry list | ✅ PASS | 12 recipes, loads without error |
| Static: `win32_printwindow` | ✅ PRESENT | driver.py + win_capture.py |
| Static: `desktop_bitblt` | ✅ PRESENT | driver.py (fallback) + win_capture.py (impl) |
| Static: `CCGS_AUTOPLAY_BOT_ROOM_READY` | ✅ PRESENT | ps1 gate + 3 recipe guards + README |
| Static: stale-pyc guard | ✅ PRESENT | Run-AutoplaySmoke.ps1 + Start-AutoplayVsBot.ps1 |
| git diff --check | ✅ PASS | No whitespace issues |

**No failures detected. No repair prompts required.**

---

1838: POST-1830-AUTOPLAY-TOOLING-FOCUSED-VERIFY: PASS
