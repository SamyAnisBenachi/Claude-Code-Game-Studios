# PROMPT 1818 — AUTOPLAY-FROZEN-PRINTWINDOW-TRIGGERS-BITBLT-FALLBACK

**Status:** SHIPPED
**Date:** 2026-05-28
**Branch:** `worktree-1818-frozen-printwindow-bitblt-fallback`
**Commit:** `d8b41463`

---

## 1. Problem Addressed

PROMPT 1817 identified that `win32_printwindow` can return `True` while
producing byte-identical frozen frames (DWM stale-buffer behavior). The
existing BitBlt fallback only triggered on `PrintWindow` API failure
(`ret=False`), so it was never exercised across 15 checkpoints in the 1817
live-verify run.

---

## 2. Implementation

### `tools/autoplay/driver.py`

**New module-level helper `_frozen_win32_check`:**

```python
def _frozen_win32_check(
    win32_shot: Path,
    win32_ok: bool,
    last_hash: str | None,
) -> tuple[bool, str, str | None]:
```

Returns `(need_bitblt, reason, updated_last_hash)`:

| Condition | `need_bitblt` | `reason` |
|---|---|---|
| `win32_ok=False` | `True` | `"win32_printwindow_failed"` |
| `win32_ok=True`, `last_hash=None` (first tick) | `False` | `""` |
| `win32_ok=True`, hash changed | `False` | `""` |
| `win32_ok=True`, hash == `last_hash` (frozen) | `True` | `"frozen_printwindow"` |

**State variable in `main()`:**
```python
last_win32_hash: str | None = None
```
Initialised to `None` so the first capture never false-triggers.

**Updated screenshot capture block:**
```python
_win32_ok = _win32_capture(_win32_shot, log)
log(f"tick={tick} win32_printwindow={'OK' if _win32_ok else 'FAILED'} ...")
_need_bitblt, _bitblt_reason, last_win32_hash = _frozen_win32_check(
    _win32_shot, _win32_ok, last_win32_hash
)
if _bitblt_reason == "frozen_printwindow":
    log(f"tick={tick} win32_printwindow=FROZEN hash={last_win32_hash} — triggering desktop_bitblt fallback")
if _need_bitblt:
    _bitblt_shot = artifact_dir / f"bitblt_tick_{tick:06d}.png"
    _bitblt_ok = _desktop_bitblt_capture(_bitblt_shot, log)
    log(f"tick={tick} desktop_bitblt={'OK' if _bitblt_ok else 'FAILED'} reason={_bitblt_reason} ...")
```

**Log output examples:**

- Frozen frame detected:
  ```
  tick=3 win32_printwindow=OK path=win32_tick_000003.png
  tick=3 win32_printwindow=FROZEN hash=2e045fb3ebdd... — triggering desktop_bitblt fallback
  tick=3 desktop_bitblt=OK reason=frozen_printwindow path=bitblt_tick_000003.png
  ```

- PrintWindow API failure (existing path):
  ```
  tick=3 win32_printwindow=FAILED path=win32_tick_000003.png
  tick=3 desktop_bitblt=OK reason=win32_printwindow_failed path=bitblt_tick_000003.png
  ```

---

## 3. Files Changed

| File | Change |
|---|---|
| `tools/autoplay/driver.py` | Add `import hashlib`; add `_frozen_win32_check()` helper; add `last_win32_hash` state var; use frozen check in screenshot branch |
| `tests/tools/autoplay/test_win32_capture.py` | 17 new tests in `TestFrozenWin32Check` (7) and `TestDriverFrozenPrintWindowStructural` (7); update `TestDriverDesktopBitbltFallback.test_driver_calls_desktop_bitblt_after_win32_failure` → `test_driver_calls_desktop_bitblt_via_frozen_check` |

---

## 4. Validation

### Path allowlist review

Only Python tooling files were modified:
- `tools/autoplay/driver.py` ✅
- `tests/tools/autoplay/test_win32_capture.py` ✅

No Rust/Bevy source, gameplay code, production sprint/status files touched.

### Whitespace check

```
git diff --check HEAD    → (no output — clean)
```

### Test results

```
pytest tests/tools/autoplay/test_win32_capture.py -v
76 passed in 0.27s
```

### New tests (covering all 5 required scenarios)

| Scenario | Test | Result |
|---|---|---|
| First capture does not trigger BitBlt (no prior hash) | `test_frozen_win32_check_first_capture_no_bitblt_without_prior_hash` | PASS |
| Repeated identical hashes trigger BitBlt | `test_frozen_win32_check_identical_hash_triggers_bitblt` | PASS |
| Changed hash does not trigger BitBlt | `test_frozen_win32_check_changed_hash_no_bitblt` | PASS |
| Log records frozen-frame reason | `test_frozen_win32_check_logs_reason_is_frozen_printwindow` | PASS |
| Existing PrintWindow failure still triggers BitBlt | `test_frozen_win32_check_printwindow_failure_triggers_bitblt` | PASS |

Additional regression tests: hash stored on first capture, hash unchanged on failure, 7 structural driver checks.

---

## 5. Integration Readiness

Ready to merge to `main`. This is Python-only tooling; no Rust build required.

The frozen-frame detection fires on the **second** consecutive identical
PrintWindow capture (i.e., 2 ticks with the same hash), which avoids
false-positives from static game screens that legitimately don't change
between two adjacent ticks. For the PROMPT 1817 scenario where all 15
checkpoints had the same hash, BitBlt would trigger on tick 2 and every
subsequent frozen tick.

---

1818: AUTOPLAY-FROZEN-PRINTWINDOW-TRIGGERS-BITBLT-FALLBACK: SHIPPED
