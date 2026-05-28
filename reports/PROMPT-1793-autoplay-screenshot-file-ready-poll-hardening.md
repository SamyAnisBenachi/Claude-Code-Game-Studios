# PROMPT 1793 — Autoplay Screenshot File-Ready Poll Hardening

**Branch:** `work/autoplay-screenshot-file-ready-poll-1793`
**Base commit:** `bf9f2bf11b169ae70a00a3abeac356a2a0dc76b0`

## Problem

The `autoplay/screenshot` RPC returns immediately after queuing the screenshot
command. The Bevy `save_to_disk` observer runs asynchronously after the GPU
flush; by the time the driver continued to the next tick the PNG might not
exist on disk yet. This was identified as a secondary correctness gap (GAP-SCR-02)
by the PROMPT 1791 diagnostic.

## Solution

Implemented a bounded file-ready poll that bridges the gap between RPC return
and PNG write completion.

### New module: `tools/autoplay/screenshot_poll.py`

Extracted as a standalone module (no platform-specific imports) so it can be
imported and tested independently of `driver.py`'s `win_foreground` dependency.

- `wait_for_screenshot_file(path, tick, log_fn, poll_interval=0.1, timeout=3.0)`
  — polls `path.exists() and path.stat().st_size > 0` at 100 ms intervals.
- Returns `True` and logs `"tick=N screenshot file ready: <name> (<size> bytes)"`
  on success.
- Returns `False` and logs `"tick=N WARNING screenshot file-ready poll timed out
  after 3s: path=<path>"` on timeout. Does not raise; driver continues.
- `OSError` during stat is silently retried (handles transient FS races).

### Modified: `tools/autoplay/driver.py`

- Imports `wait_for_screenshot_file` from `screenshot_poll`.
- After a successful `autoplay/screenshot` RPC, extracts `relative_path` from
  the result dict and calls `wait_for_screenshot_file(artifact_dir / rel, tick, log)`.
- Guard is conditional on `method == "autoplay/screenshot"` and
  `isinstance(result, dict)` and `rel` being truthy — no change to other
  RPC methods.
- The existing frame-advance barrier (GAP-SCR-01, PROMPT 1766) and foreground
  barrier (PROMPT 1776) are fully preserved.

## Files Changed

| File | Change |
|------|--------|
| `tools/autoplay/screenshot_poll.py` | **New** — standalone poll module |
| `tools/autoplay/driver.py` | **Modified** — import + call after screenshot RPC |
| `tests/tools/autoplay/test_driver_screenshot_barrier.py` | **Modified** — 9 new tests (5 behavioural, 4 structural) |
| `reports/PROMPT-1793-autoplay-screenshot-file-ready-poll-hardening.md` | **New** — this report |

## Test Results

```
pytest tests/tools/autoplay/test_driver_screenshot_barrier.py -v
============================= test session starts =============================
platform win32 -- Python 3.12.10, pytest-9.0.3
collected 20 items

TestCheckpointSettleTicksGap::test_checkpoint_settle_default_is_3_ticks         PASSED
TestCheckpointSettleTicksGap::test_checkpoint_settle_ticks_zero_reverts_to_immediate PASSED
TestCheckpointSettleTicksGap::test_checkpoint_settle_ticks_custom_value         PASSED
TestCheckpointSettleTicksGap::test_checkpoint_no_screenshot_emits_no_screenshot_action PASSED
TestCheckpointSettleTicksGap::test_multiple_checkpoints_each_get_settle_gap     PASSED
TestCheckpointSettleTicksGap::test_checkpoint_screenshot_reason_matches_label   PASSED
TestCheckpointSettleTicksGap::test_settle_ticks_does_not_affect_checkpoint_without_screenshot PASSED
TestDriverFrameAdvanceBarrierPresent::test_driver_tracks_last_screenshot_frame  PASSED
TestDriverFrameAdvanceBarrierPresent::test_driver_checks_frame_before_screenshot PASSED
TestDriverFrameAdvanceBarrierPresent::test_driver_logs_stale_frame_warning      PASSED
TestDriverFrameAdvanceBarrierPresent::test_driver_frame_guard_initialised_to_minus_one PASSED
TestWaitForScreenshotFile::test_screenshot_file_ready_poll_returns_true_when_file_exists PASSED
TestWaitForScreenshotFile::test_screenshot_file_ready_poll_returns_false_on_timeout PASSED
TestWaitForScreenshotFile::test_screenshot_file_ready_poll_ignores_zero_byte_file PASSED
TestWaitForScreenshotFile::test_screenshot_file_ready_poll_logs_filename_and_size_on_success PASSED
TestWaitForScreenshotFile::test_screenshot_file_ready_poll_includes_tick_in_all_messages PASSED
TestDriverFileReadyPollPresent::test_driver_imports_screenshot_poll             PASSED
TestDriverFileReadyPollPresent::test_driver_calls_wait_for_screenshot_file      PASSED
TestDriverFileReadyPollPresent::test_driver_resolves_relative_path_from_result  PASSED
TestDriverFileReadyPollPresent::test_driver_file_poll_only_runs_for_screenshot_method PASSED

============================= 20 passed in 0.14s ==============================
```

**All 20 tests pass** (11 pre-existing + 9 new).

## Scope Boundary

This hardens completion detection on the driver side only. It does not by itself
prove screenshot distinctness — that is covered by PROMPT 1792 (screenshot
distinctness verification). Together, GAP-SCR-01 (frame-advance barrier),
GAP-SCR-02 (file-ready poll, this PROMPT), and the offscreen capture payload
(PROMPT 1790) form a three-layer defence against stale or missing screenshots.

## git diff --check

No whitespace errors.

1793: AUTOPLAY-SCREENSHOT-FILE-READY-POLL-HARDENING: SHIPPED
