# PROMPT-1433 - S18 Lobby Confirm CTA Reachability Main-Land

Status: refresh prepared for main-land.

## Base

- Fetched origin before refresh.
- Current `origin/main`: `896a21acb83d2ab37680c7c7891021d4d6870aa4`.
- Required PROMPT 1432 commit `896a21acb83d2ab37680c7c7891021d4d6870aa4` is an ancestor of `origin/main`.

## Source

- Source commit used: `3662f549eded58ad0483b449c9c87a72e447db1f`.
- Source commit subject: `dev-story(s18-lobby-confirm-cta-viewport-reachability): wrap upper content in LobbyPanelBody so Confirm CTA stays viewport-reachable (PROMPT 1398)`.
- Source replay method: `git cherry-pick -n 3662f549eded58ad0483b449c9c87a72e447db1f`.

## Final Branch

- Branch: `mainland/s18-lobby-confirm-cta-reachability-1433-refresh`.
- Worktree: `D:\Tmp\ccgs-prompt-1433-lobby-confirm-cta-mainland-refresh`.
- Final commit: this report-bearing refresh commit on the branch. Git computes the exact object id after this file is committed; the completion relay records the exact final hash.

## Changed Files

- `client/Cargo.toml`
- `client/src/ui/lobby.rs`
- `tests/integration/playable_client/lobby_class_picker_layout_test.rs`
- `tests/integration/playable_client/lobby_confirm_cta_viewport_reachability_test.rs`
- `reports/PROMPT-1433-s18-lobby-confirm-cta-reachability-main-land.md`

## Conflicts

- None. The replay applied cleanly.

## Verification

- `git merge-base --is-ancestor 896a21acb83d2ab37680c7c7891021d4d6870aa4 origin/main`: PASS.
- Changed-file allowlist check: PASS.
- `git diff --check`: PASS before report creation; final report file is plain Markdown and rechecked before commit.

## Cargo Policy

- Broad Cargo was not run, per prompt instruction.
- Targeted Cargo was not run because the replay had no conflicts and no semantic uncertainty.
- Windows/MSVC Cargo resource policy was not invoked because no Cargo command was needed.

## Push/Main Status

- Main push is attempted after the refresh commit is created.
- If protected-branch or permission policy blocks the main push, this branch is kept/pushed as the refresh branch and the final status is `READY_FOR_MAIN_LAND`.

## Non-Claims

- No broad workspace Cargo verification is claimed.
- No browser/manual viewport run is claimed.
- No files outside the prompt allowlist are claimed as changed by this refresh.
