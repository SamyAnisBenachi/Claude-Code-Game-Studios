# PROMPT 2020 — Autoplay Poll Phase Pseudo-Action R02

**Branch**: `work/PROMPT-2020`
**Commit**: `be8f4ce7`
**Source root**: `origin/main@05014373`

---

## Summary

Implemented `poll_phase(label, max_ticks)` pseudo-action for the autoplay recipe
system, replacing brittle fixed `wait()` calls with phase-aware polling against
`autoplay/status["phase"]`.

---

## Changes

### `tools/autoplay/recipes/_builder.py`
- Added `RecipeBuilder.poll_phase(label, max_ticks=30)` method.
- Emits `{"method": "local.poll_phase", "params": {"label": ..., "max_ticks": ...}}`.
- Advances tick by 1 (one driver tick to schedule the poll action).
- Chainable; returns `self`.

### `tools/autoplay/driver.py`
- Added `"local.poll_phase"` to `LOCAL_METHODS` set.
- Added `_poll_for_phase(current_status, url, label, max_ticks, tick_secs, tick,
  started, log_fn, emit_checkpoint_fn, *, _rpc, _sleep)` module-level helper
  — injectable `_rpc` / `_sleep` parameters for unit-test isolation.
- Handler in action dispatch loop (`elif method == "local.poll_phase":`) passes
  the tick-start `status` snapshot as `current_status` so a recipe that already
  arrives in the target phase costs zero extra RPCs.

### `tests/tools/autoplay/test_poll_phase.py` (new)
39 tests across four classes:

| Class | Tests | Coverage |
|---|---|---|
| `TestPollPhaseBuilder` | 12 | Emitted method, params, tick advance, chaining, JSON round-trip |
| `TestPollPhaseDriverStructure` | 3 | `local.poll_phase` in `LOCAL_METHODS`, function existence + signature |
| `TestPollForPhaseBehaviour` | 19 | Immediate match, N-poll match, last-poll match, timeout, checkpoint row shape, RPC failure tolerance, None status, max_ticks=1 edge cases |
| `TestPollPhaseStaticCompat` | 3 | Allowlist union accepts it, JSON round-trip, pre-existing LOCAL_METHODS intact |

---

## Validation

### Path allowlist
```
git diff --check -- tools/autoplay/recipes/_builder.py tools/autoplay/driver.py tests/tools/autoplay/test_poll_phase.py
→ CLEAN on owned files
```

Pre-existing trailing whitespace in `.claude/settings.json` (unrelated, not
introduced by this PROMPT — excluded from check scope).

### Focused test run
```
pytest tests/tools/autoplay/test_poll_phase.py -v
→ 39 passed in 0.11s
```

### Backward compatibility
```
pytest tests/tools/autoplay/test_recipe_static.py \
       tests/tools/autoplay/test_driver_click_viewport_guard.py \
       tests/tools/autoplay/test_driver_screenshot_barrier.py
→ 163 passed in 0.29s
```

No existing recipe emits `local.poll_phase`, so `test_recipe_static.py`'s
`ALLOWED_METHODS` copy was not modified — it remains a self-contained mirror of
the pre-2020 allowlist. The new method is only exercised by new recipes that
explicitly call `b.poll_phase(...)`.

---

## Behavioural contract

- On match: emits checkpoint `{kind: "poll_phase", matched: true, polls: N, label, tick, elapsed_secs}`.
- On timeout: emits checkpoint with `matched: false, timed_out: true, polls: max_ticks`; driver continues (no abort — recipe controls next step).
- RPC failures during retry polls are swallowed; the slot counts as a failed poll and the loop continues.
- `status["phase"]` is the field polled; `None` or missing phase never matches.

---

## Files modified (owned scope only)

```
tools/autoplay/recipes/_builder.py   +19 lines
tools/autoplay/driver.py             +85 lines
tests/tools/autoplay/test_poll_phase.py  +389 lines (new)
```

No Bevy/Rust code, no bot runtime, no production status files, no broad recipe rewrites.

---

2020: AUTOPLAY-POLL-PHASE-PSEUDOACTION-R02: SHIPPED
