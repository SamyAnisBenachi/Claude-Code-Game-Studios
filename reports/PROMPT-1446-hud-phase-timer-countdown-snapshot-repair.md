# PROMPT 1446 -- HUD Phase Timer Countdown Snapshot Repair

Status: REPAIRED

## Root Cause

`phase_sink_system` passed `ClientPhaseView` through the phase-apply path even on frames with no `S2CPhaseChanged` messages. That mutable access could mark `ClientPhaseView` changed again, so `reset_phase_timer_system` treated the same phase view as a fresh phase transition and reset `PhaseTimerState.elapsed_ms` back to `0`. The HUD countdown and QA snapshot therefore kept reporting the phase duration (`30s`, `10s`, `12000`) instead of an elapsed-aware remaining value.

## Repair

- `client/src/presentation/mod.rs`
  - Returns early from `phase_sink_system` when no phase messages were drained, avoiding unnecessary mutable phase-view flow.
- `client/src/ui/hud/mod.rs`
  - `PhaseTimerState` now records the phase, round, local monotonic phase-start reference, duration, elapsed, and active state.
  - `reset_phase_timer_system` now resets only when phase, round, or duration actually changes.
  - HUD timer bar/countdown now read `PhaseTimerState::remaining_ms()` and `display_text()`.
- `client/src/presentation/qa_snapshot.rs`
  - `extras.timers.phase_timer` now exposes:
    - `phase_started_elapsed_ms`
    - `phase_duration_ms`
    - `computed_remaining_ms`
    - `display_text`
    - `timer_source`
  - Existing `duration_ms`, `elapsed_ms`, `remaining_ms`, and `active` fields remain for compatibility.
- Focused tests now cover same-phase changed-resource reset prevention and snapshot serialization that distinguishes duration from computed remaining time.

## Tests

Cargo policy applied: yes.

Command:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
cargo test -p client --test hud_phase_timer_countdown_test --test hud_phase_timer_bar_test --test qa_snapshot_overlay_test --test qa_snapshot_placement_auction_state_field_coverage_test
```

Result: passed.

- `hud_phase_timer_bar_test`: 4 passed.
- `hud_phase_timer_countdown_test`: 5 passed.
- `qa_snapshot_overlay_test`: 27 passed.
- `qa_snapshot_placement_auction_state_field_coverage_test`: 13 passed.

Note: the first Cargo attempt hit `Access is denied` on `D:\_DEV\cargo-target\ccgs-msvc\debug\.cargo-lock`; reran with approved escalation. A stale client metadata issue then required `cargo clean -p client` in the same target directory before the targeted tests passed.

## Live Retest

Live two-client QA snapshot retest remains required. Source and focused tests prove the HUD/snapshot countdown now uses elapsed-aware remaining time, but the original evidence was live snapshot/log correlation and should be re-captured to verify wall-clock decreasing values across snapshot IDs.

## Branch / Commit / Push

- Branch: `work/hud-phase-timer-countdown-snapshot-1446`
- Local commit: see final relay / `git rev-parse --short HEAD` after this report commit.
- Push status: blocked.
  - First push command failed to connect to `github.com:443`.
  - Escalated retry was rejected by policy as external data export to the configured GitHub remote.

Final line: `1446: HUD-PHASE-TIMER-COUNTDOWN-SNAPSHOT-REPAIR: REPAIRED`
