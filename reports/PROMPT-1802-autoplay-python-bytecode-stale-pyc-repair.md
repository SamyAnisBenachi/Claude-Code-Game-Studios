# PROMPT 1802 — Autoplay Python Bytecode Stale-pyc Repair

**Date:** 2026-05-28
**Worktree:** `D:\tmp\wt-1802-stale-pyc-repair`
**Branch:** `worker/1802-stale-pyc-repair`
**Base commit:** `4eb69de4`

---

## Root Cause (from PROMPT 1801 live-verify)

PROMPT 1801 failed because Python executed
`tools/autoplay/__pycache__/driver.cpython-312.pyc`, which predated PROMPT 1794.
The stale bytecode lacked the `win_capture` integration; the driver log had zero
`win32_capture:` lines even though the source was correct on disk at the live-verify
commit (`4eb69de4`).

---

## Changes Made

### `tools/autoplay/Run-AutoplaySmoke.ps1`

Three-layer stale-pyc guard added immediately before the `Start-Process` driver
invocation:

1. **`__pycache__` cleanup** — removes `tools/autoplay/__pycache__` and
   `tools/autoplay/recipes/__pycache__` before every driver run.  If a stale `.pyc`
   is sitting from a previous session, it is gone before Python starts.

2. **`$env:PYTHONDONTWRITEBYTECODE = '1'`** — prevents Python from writing new
   `.pyc` files during this run, keeping the working tree clean.

3. **`'-B'` flag** — passed as the first positional arg to the Python invocation.
   `-B` is CPython's own "don't read or write bytecode" flag and is the most direct
   guard against stale `.pyc` execution regardless of `PYTHONPATH` or `sys.path`
   mutations.

Diff summary: 17 lines added, 0 lines removed.  No capture-algorithm files touched.

---

## Tests Added

**`tests/tools/autoplay/test_launcher_stale_pyc_guard.py`** — 5 static regression
tests that read `Run-AutoplaySmoke.ps1` as text:

| Test | Asserts |
|------|---------|
| `test_launcher_stale_pyc_guard_launcher_exists` | script exists on disk |
| `test_launcher_stale_pyc_guard_python_flag_b_present` | `'-B'` present in script |
| `test_launcher_stale_pyc_guard_env_var_set` | `PYTHONDONTWRITEBYTECODE` set |
| `test_launcher_stale_pyc_guard_cache_cleanup_present` | `__pycache__` + `Remove-Item` present |
| `test_launcher_stale_pyc_guard_cleanup_before_driver` | cleanup appears before `Start-Process $Python` |

All 5 tests pass:

```
5 passed in 0.02s
```

---

## Validation

- `git diff --check`: clean (no whitespace errors)
- Pytest: `5 passed in 0.02s`
- Scope: only `tools/autoplay/Run-AutoplaySmoke.ps1` and new test file modified.
  No Bevy/Rust, no sprint/status, no capture algorithms.

---

## How This Prevents Recurrence

Before this fix: `python driver.py` — Python checks `__pycache__/driver.cpython-312.pyc`;
if it exists and the `.pyc` timestamp passes, it runs the old code silently.

After this fix:
- Stale cache wiped before the process starts.
- `-B` instructs CPython to bypass bytecode entirely for this invocation.
- `PYTHONDONTWRITEBYTECODE=1` ensures no new stale `.pyc` is created as a side-effect.

Future source edits to any file under `tools/autoplay/` will always execute from
source during QA runs.

---

1802: AUTOPLAY-PYTHON-BYTECODE-STALE-PYC-REPAIR: SHIPPED
