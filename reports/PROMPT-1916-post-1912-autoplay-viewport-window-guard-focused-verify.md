# PROMPT 1916 — POST-1912 Autoplay Viewport/Window-Guard Focused Verify

**Date:** 2026-05-28
**Worktree:** `D:\tmp\wt-1916-viewport-verify`
**Branch:** `worker/1916-viewport-guard-verify`
**Base commit:** `1c945fd2` (origin/main after PROMPT 1912 whitespace cleanup)

---

## 1. Scope

Read-only verification that `origin/main@1c945fd2` contains both the PROMPT 1894
click-target viewport guard and the PROMPT 1912 AC-VPT-01 window-size default
repair, that their tests are coherent, and that no immediate blockers remain
for live GUI vs-bot QA (mid-run resize / foreground shrink / offscreen click).

---

## 2. Landing Confirmation

### PROMPT 1880 / 1894 — Click-target viewport guard

| Commit | Description |
|--------|-------------|
| `e8a40f81` | feat: PROMPT 1880 — viewport guard refresh after 1872 (impl + tests) |
| `71484fc4` | docs: PROMPT 1894 — report refresh after 1856/1876 (docs only) |

**Files modified by 1880:**
- `tools/autoplay/driver.py` — +236 lines of guard logic
- `tests/tools/autoplay/test_driver_click_viewport_guard.py` — 400-line focused test suite (66 tests)

### PROMPT 1912 — AC-VPT-01 window-size default repair

| Commit | Description |
|--------|-------------|
| `e02d132f` | feat: PROMPT 1912 — reapply AC-VPT-01 window-size default repair onto post-1894 main |
| `fe2a9e88` | docs: PROMPT 1912 — report refresh |
| `1c945fd2` | docs: PROMPT 1912 whitespace cleanup |

**Files modified by 1912 (e02d132f):**
- `client/src/autoplay.rs` — `enforce_autoplay_window_size_system` Startup system + constants + unit test
- `tools/autoplay/Run-AutoplaySmoke.ps1` — env-var default block at lines 85–87
- Two report files

Both PROMPTs confirmed present in `origin/main@1c945fd2`.

---

## 3. Static Inspection

### `driver.py` viewport guard (PROMPT 1880 / 1894)

All four guard layers verified present:

| Layer | Trigger | Exit code | Implementation |
|-------|---------|-----------|----------------|
| **Pre-build minimum** | `window_logical_size` missing or < 1280×720 | `EXIT_VIEWPORT_GUARD = 5` | `_check_window_minimum()` called before recipe build |
| **Mid-run drift** (AC-VPT-02) | Window drifts > ±10 px from recipe-build size | `EXIT_VIEWPORT_GUARD = 5` | `_check_window_drift()` called every tick after recipe init |
| **Post-foreground shrink** (AC-VPT-08) | DWM SW_RESTORE shrinks window below 1280×720 after `ensure_foreground()` | `EXIT_VIEWPORT_GUARD = 5` | Re-polls status; emits `viewport_shrink_abort` checkpoint |
| **Cursor/OOB guard** | `cursor_logical=None` or click target outside window bounds | `EXIT_VIEWPORT_GUARD = 5` | Gated on `autoplay/input`; emits `viewport_guard_cursor_none` / `viewport_guard_oob` |

Constants: `_MIN_WIN_W = 1280.0`, `_MIN_WIN_H = 720.0`, `_WIN_DRIFT_PX = 10.0`

Exit code table in docstring updated to include:
`5 -- viewport guard triggered: invalid window size, mid-run resize beyond tolerance, None cursor_logical, or OOB click target`

### `Run-AutoplaySmoke.ps1` (PROMPT 1912 — AC-VPT-01)

Lines 85–87 (PROMPT 1842 block):
```powershell
if (-not $env:CCGS_WINDOW_WIDTH)  { $env:CCGS_WINDOW_WIDTH  = '1280' }
if (-not $env:CCGS_WINDOW_HEIGHT) { $env:CCGS_WINDOW_HEIGHT = '720'  }
Write-Host "[autoplay-smoke] viewport target: $($env:CCGS_WINDOW_WIDTH)x$($env:CCGS_WINDOW_HEIGHT) ..."
```
Comment states: "Unset here means the Rust fallback applies (same 1280x720 floor)."
Both env vars default to `1280` / `720` before client launch. Coherent with
`enforce_autoplay_window_size_system` in `client/src/autoplay.rs`.

### `client/src/autoplay.rs` (PROMPT 1912 — AC-VPT-01)

`enforce_autoplay_window_size_system` registered as `Startup` system inside
`AutoplayPlugin`. Reads `CCGS_WINDOW_WIDTH` / `CCGS_WINDOW_HEIGHT`; applies
`max(current, target)` so an already-larger window is never shrunk. Unit test
`autoplay_window_size_constants_match_dev_floor` confirms constants match
`SAFETY_VIEWPORT_DEV_FLOOR` (1280×720). System is Rust/Cargo gated; not
exercised by Python-only static verify, but constant test validates the floor.

---

## 4. Focused Test Run

```
pytest tests/tools/autoplay/test_driver_click_viewport_guard.py -v
platform win32 -- Python 3.12.10, pytest-9.0.3

collected 66 items
... (66 tests) ...
66 passed in 0.18s
```

All 66 tests pass against `origin/main@1c945fd2` driver.py. No regressions.

**Test classes:**

| Class | Tests | Covers |
|-------|-------|--------|
| `TestValidateCursorCoords` | 13 | OOB detection, axis clips, fractional coords, log content |
| `TestParseWindowSize` | 11 | Valid/invalid `window_logical_size` parsing |
| `TestCheckWindowMinimum` | 8 | Minimum-size gate, None handling |
| `TestCheckWindowDrift` | 10 | Mid-run resize within/beyond ±10 px tolerance |
| `TestExitViewportGuard` | 4 | EXIT_VIEWPORT_GUARD = 5, distinct from other codes |
| `TestDriverViewportGuardStructure` | 20 | Structural: all helpers defined, called in correct order, checkpoints emitted |

---

## 5. Coherence Assessment

| Check | Result |
|-------|--------|
| 1880 guard implementation matches 1894 docs | PASS — docs describe exactly what is in driver.py |
| 1912 RS constant matches PS1 default | PASS — both use 1280×720; env-var names align |
| PS1 default fires before client launch | PASS — block at lines 85–87, before `Start-Process cargo run` |
| Driver pre-build check matches minimum | PASS — `_MIN_WIN_W/H = 1280/720.0` aligns with RS constants |
| Post-foreground shrink check present | PASS — re-polls status after `ensure_foreground()`, emits `viewport_shrink_abort` |
| `cursor_logical=None` aborts before RPC | PASS — guard fires before `rpc(url, method, params)` call |
| OOB check gated on `autoplay/input` only | PASS — structural test `test_driver_guard_gated_on_autoplay_input` confirms ordering |

---

## 6. Remaining Blockers for Live GUI vs-Bot QA

### Eliminated by 1880/1894 + 1912

- ✅ Offscreen click from undersized window → pre-build minimum guard aborts with exit 5
- ✅ Mid-run user resize → drift guard aborts with exit 5
- ✅ Post-foreground shrink (DWM SW_RESTORE) → AC-VPT-08 check aborts with exit 5
- ✅ cursor outside window at input time → cursor_logical=None guard aborts with exit 5
- ✅ OOB recipe coordinate → `_validate_cursor_coords` aborts with exit 5
- ✅ Window opened below 1280×720 by OS → `enforce_autoplay_window_size_system` Startup corrects it
- ✅ Env vars not set → PS1 defaults to `1280`/`720` before build

### Not covered by static verify (require live GUI run)

- ⚠️ **090613 evidence caveat (C0)** — preserved. The 090613 autoplay run was conditional;
  the driver-side guards are now in place, but no post-1912 live GUI vs-bot run has been
  performed in this worker. The C0 caveat stands until a clean live run is confirmed.
- ⚠️ **`enforce_autoplay_window_size_system` Rust behaviour** — verified via constants and
  unit test only; not exercised via `cargo test` in this worker (no `--no-cargo` bypass
  exists; skipped per task scope).
- ⚠️ **DWM compositing latency** — the 120 ms sleep after `ensure_foreground` is empirical;
  may not be sufficient on heavily loaded machines. No change since 1776; not a new risk.

---

## 7. Summary

Both PROMPT 1894 (click-target viewport guard) and PROMPT 1912 (AC-VPT-01 window-size
default repair) are confirmed landed in `origin/main@1c945fd2`. The guard layers are
coherent across `driver.py`, `Run-AutoplaySmoke.ps1`, and `client/src/autoplay.rs`.
All 66 focused Python tests pass. No immediate static or logic blockers remain for
live GUI vs-bot QA. The 090613 C0 caveat is preserved: promotion requires a clean
post-1912 live run, which is out of scope for this static-verify worker.

---

1916: POST-1912-AUTOPLAY-VIEWPORT-WINDOW-GUARD-FOCUSED-VERIFY: SHIPPED
