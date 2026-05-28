# PROMPT 1804 — Autoplay Python Bytecode Stale-PYC Integration Refresh

**Date:** 2026-05-28
**Branch:** `worker/1804-stale-pyc-integration`
**Worktree:** `D:\tmp\wt-1804-stale-pyc-integration`
**Base:** `origin/main@4eb69de4`
**Cherry-picked commit:** `94352f66` (PROMPT 1802)
**Integration commit:** `9a3c0cde`

## Summary

PROMPT 1802 (`fix(autoplay): PROMPT 1802 — stale-pyc guard in Run-AutoplaySmoke.ps1`)
was cleanly cherry-picked from `worker/1802-stale-pyc-repair` onto a fresh integration
branch based on `origin/main@4eb69de4`.

## Payload Scope (path allowlist)

| File | Status |
|---|---|
| `tools/autoplay/Run-AutoplaySmoke.ps1` | included — stale-pyc guard |
| `tests/tools/autoplay/test_launcher_stale_pyc_guard.py` | included — focused test |
| `reports/PROMPT-1802-autoplay-python-bytecode-stale-pyc-repair.md` | included — upstream report |

No forbidden files touched (`driver.py`, `win_capture.py`, Bevy/Rust source,
production sprint/session-state files, broad Cargo suites).

## Validation

### git diff --check
```
EXIT:0  (no whitespace errors)
```

### Focused pytest
```
tests/tools/autoplay/test_launcher_stale_pyc_guard.py::test_launcher_stale_pyc_guard_launcher_exists        PASSED
tests/tools/autoplay/test_launcher_stale_pyc_guard.py::test_launcher_stale_pyc_guard_python_flag_b_present  PASSED
tests/tools/autoplay/test_launcher_stale_pyc_guard.py::test_launcher_stale_pyc_guard_env_var_set            PASSED
tests/tools/autoplay/test_launcher_stale_pyc_guard.py::test_launcher_stale_pyc_guard_cache_cleanup_present  PASSED
tests/tools/autoplay/test_launcher_stale_pyc_guard.py::test_launcher_stale_pyc_guard_cleanup_before_driver  PASSED

5 passed in 0.07s
```

## Push

Branch `worker/1804-stale-pyc-integration` pushed to origin. Integration commit: `9a3c0cde`.

1804: AUTOPLAY-PYTHON-BYTECODE-STALE-PYC-INTEGRATION-REFRESH: SHIPPED
