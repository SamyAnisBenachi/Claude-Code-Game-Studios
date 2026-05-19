# PROMPT 1435 - BOT-PLAYER-FOUNDATION-SCAFFOLD-INTEGRATION

## Summary

Integrated the server-only bot foundation scaffold from source branch
`origin/work/bot-player-foundation-1428` onto current `origin/main` in a fresh
integration worktree/branch.

Status: `READY_FOR_MAIN_LAND`.

## Source and base

- Base before final rebase: `origin/main@896a21acb83d2ab37680c7c7891021d4d6870aa4`.
- Final base after `origin/main` advanced during this task:
  `origin/main@30a270d071a85135c66ecfc5c11b5cc990075dc2`.
- Source worker branch: `origin/work/bot-player-foundation-1428`.
- Expanded source worker commit:
  `388ff4bd2a22698f4d3b1e644446a43175daa359`.
- Source worker report was available from the source worktree at
  `D:\_DEV\claude-code-game-studios-worktrees\bot-player-foundation-1428\reports\PROMPT-1428-bot-player-foundation-scaffold.md`;
  it was not present as a tracked path in source commit `388ff4bd`.

## Integration branch

- Worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\bot-player-foundation-scaffold-integration-1435`
- Branch: `integrate/bot-player-foundation-scaffold-1435`.
- Integration commit after rebase:
  `9807c1d9c8a2bab2f7c25d34509999d54a9ad1f3`.
- Report commit: pending at report creation; final branch head is recorded in
  the relay/final response after commit.

## Changed files

- `server/Cargo.toml`
- `server/src/feature/bot/mod.rs`
- `server/src/feature/bot/state.rs`
- `server/src/feature/mod.rs`
- `tests/unit/bot/bot_foundation_state_test.rs`
- `reports/PROMPT-1435-bot-player-foundation-scaffold-integration.md`

## Conflicts

No cherry-pick conflicts.

`origin/main` advanced once after the worktree was created. I rebased
`integrate/bot-player-foundation-scaffold-1435` onto
`origin/main@30a270d071a85135c66ecfc5c11b5cc990075dc2`; the rebase completed
cleanly.

## Scope compliance

The integration stayed within the prompt's allowed write set. No forbidden
paths were modified:

- `client/**`
- `shared/**`
- `server/src/feature/auction/**`
- `server/src/feature/acquisition/**`
- `server/src/feature/board/**`
- `server/src/core/rsm/**`
- `server/src/lobby/**`
- `server/src/network/**`
- `production/**`
- `Cargo.lock`, `.cargo/**`, `.github/**`, `.claude/**`

No protocol/networking code was touched, so `liv-bevy-lightyear` was not used.
`liv-bevy-018` was used for Bevy/Rust review.

## Verification

Windows/MSVC Cargo policy was applied for both Cargo runs:

- `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`
- `CARGO_PROFILE_DEV_DEBUG=0`
- `CARGO_PROFILE_TEST_DEBUG=0`
- `CARGO_INCREMENTAL=0`
- `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`

Results:

- `cargo check -p server`: PASS after final rebase.
- `cargo test -p server --test bot_foundation_state_test`: PASS, 8 passed.
- `git diff --check HEAD~1..HEAD`: PASS.

Notes:

- The first sandboxed `cargo check -p server` attempt failed because the
  sandbox could not open
  `D:\_DEV\cargo-target\ccgs-msvc\debug\.cargo-lock` (`Access is denied`).
  The approved rerun using the same Cargo policy passed.
- Broad workspace tests were not run, per prompt.

## Push and main status

Branch push is pending at report creation. Main was not pushed from this worker.
This branch is intended for orchestrator main-land after review:
`READY_FOR_MAIN_LAND`.

## Next unblocked bot work

- Phase 2 heuristic pure functions can consume `server::feature::bot::*`.
- Later phase workers can populate `BotPhaseTiming`, advance
  `rng_word_counter`, and append `BotDecisionEntry` rows without changing this
  foundation shape.
- Lobby/class-choice bot logic can use `BotState.class_choice` and
  `BotDecisionKind::ClassChosen` / `ClassConfirmed`.

1435: BOT-PLAYER-FOUNDATION-SCAFFOLD-INTEGRATION: READY_FOR_MAIN_LAND
