# PROMPT 1463 -- HUD Objective/Timer Readability Repair

Status: DONE

## Scope

Repaired and verified the HUD-owned readability lane only:

- `client/src/ui/hud/mod.rs`
- `tests/integration/hud/hud_phase_timer_countdown_test.rs`
- `tests/integration/hud/scoreboard_dot_message_test.rs`
- `tests/integration/ui_clean_pass/hud_top_strip_layout_test.rs`

I did not edit the explicitly excluded board rendering, hand, shop auction, server, shared, protocol, sprint-status, session-state, sprint plan, QA plan, stage, or QA snapshot files for this prompt.

## Repair

- Made the phase countdown render as a readable labelled chip (`TIME 30s`) instead of an ambiguous bare seconds token.
- Kept `PhaseTimerState::display_text()` unprefixed (`30s`) so PROMPT 1446/1452 snapshot countdown semantics remain compatible.
- Added a fixed minimum width, non-shrinking layout, larger font, dark backing, and high-contrast border to the countdown chip.
- Kept the timer bar as a stable sibling by preventing shrink collapse.
- Upgraded objective dot state to distinguish `Unknown`, `Alive`, and `Destroyed` explicitly.
- Replaced destroyed-dot transparent fill with visible high-contrast fill and border.
- Added marker names that expose owner, lane, and state, e.g. `HUD Opponent Objective Lane 3 Destroyed`.
- Preserved hidden-information behavior: opponent dots spawn unknown before first snapshot, then repaint to alive/destroyed when objective data arrives.
- Preserved phase label disambiguation already present in HUD: `KEEP-9`, `SHOP`, `AUCTION`, `PLACE`, `RESOLVE`.

## Verification

Cargo / Windows MSVC policy applied: yes.

First Cargo attempt in the isolated worktree used the required policy but failed on the shared target lock:

```text
failed to open: D:\_DEV\cargo-target\ccgs-msvc\debug\.cargo-lock
Access is denied. (os error 5)
```

Escalated rerun initially exposed stale shared-target metadata from another checkout. I ran `cargo clean -p client` with the same MSVC policy, then reran the targeted tests.

Final escalated rerun used the same policy and passed:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
cargo test -p client --test hud_phase_timer_countdown_test --test scoreboard_dot_message_test --test hud_scoreboard_dot_image_refresh_test --test hud_top_strip_layout_test
```

Results:

- `hud_phase_timer_countdown_test`: 7 passed.
- `hud_scoreboard_dot_image_refresh_test`: 3 passed.
- `hud_top_strip_layout_test`: 9 passed.
- `scoreboard_dot_message_test`: 5 passed.

Only existing deprecated-marker warnings were emitted.

`git diff --check` passed.

## Notes

- No full workspace tests were run, per prompt.
- I did not edit `client/src/presentation/qa_snapshot.rs`; no follow-up snapshot field request is required from this repair because the readable marker names and existing phase timer snapshot fields cover this prompt's targeted HUD assertions.
- Rebuilt on a fresh isolated worktree from latest `origin/main`.
- Branch: `work/hud-objective-timer-readability-repair-1463`.
- Changed file list was verified to be limited to the owned source/test files and this report.
- Commit: this branch commit.
- Push status: blocked. `git push origin work/hud-objective-timer-readability-repair-1463` failed in sandbox with `Could not connect to server`; escalated retry was rejected by tenant policy as external publication to GitHub.
- Relay status: pending final branch/commit/push result.

1463: HUD-OBJECTIVE-TIMER-READABILITY-REPAIR: DONE
