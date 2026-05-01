# Codex Orchestrator State

Updated: 2026-05-01
Owner: Codex orchestration window

Purpose: durable coordination notes for parallel implementation. This file tracks
agent windows, pending story-done work, unlocks, and known blockers. It is not the
authoritative story status tracker; `production/sprint-status.yaml` remains the
source of truth for story status.

## Current Policy

- Do not block new implementation work on GitHub Actions unless CI reports a red
  failure that needs repair.
- New workers use one Git worktree and one branch per story:
  `D:\_DEV\claude-code-game-studios-worktrees\<story-id>` on
  `work/<story-id>-<short-slug>`.
- Workers run local Developer PowerShell checks, commit explicit owned paths,
  push their story branch, and report branch name, commit hash plus CI run if
  available.
- The root checkout stays reserved for orchestrator integration merges,
  story-done, CI triage, and state tracking.
- Story-done windows are serialized because they edit shared production files.
- Keep commits scoped. If the pre-commit hook blocks due to mixed files, unstage
  and re-add explicit owned paths.
- Existing shared-tree workers already launched before the worktree switch may
  finish normally; do not migrate them mid-story.

## Live Windows Confirmed By User

- S3-06 story-done window.

## Tracker In-Progress But No Live Window Confirmed

These are marked `in-progress` in `production/sprint-status.yaml`, but the user
confirmed no corresponding agent window is currently running. Treat them as
stale/incomplete until explicitly relaunched or closed:
None currently tracked here.

## Recently Implemented, Needs Formal Story-Done

- CA-002: Card Acquisition Draft Initial implemented at `2c6c65b`; local draft
  initial tests, state scaffold tests, and `cargo check -p server` passed.
- KW-003: First Strike and Haste implemented at `874d86b`; local
  `first_strike_haste_test` and `cargo check -p server` passed. Story text still
  has stale ADR-018 Proposed/BLOCKED wording; completion review must document
  that ADR-018 is Accepted in the 2026-05-01 manifest.
- S3-04: RSM Timers + Input Reader implemented at `eff5cf9`; local RSM/economy
  suite, full server tests, cargo check, and single-writer grep passed.
- S3-05: RSM Win Condition and Game Over implemented at `5bf6bde`; local RSM
  win-condition and F2 ordering tests passed, RSM feature-import grep passed,
  and `git diff --check` passed for S3-05 files.
- S3-06: E2E WebSocket Roundtrip implemented at `a32a3df`; local websocket
  test and WASM client release build passed.
- HUD-001: HUD Plugin Scaffold implemented at `b04748b`; `git diff --check` and
  `cargo fmt --check` passed. Local client test was blocked by native
  `aws-lc-sys` dependency compilation; document this in story-done.
- CARD-ANIM-002: Tween Cancel/Replace Lifecycle implemented on branch
  `work/card-anim-002-tween-cancel-replace` at `1354d5a`; merged into `main` at
  `e9103d9`. Local plugin scaffold + tween lifecycle tests and
  `cargo check -p client` passed. Merge conflict in `client/Cargo.toml` resolved
  by keeping both HUD-001 and CARD-ANIM-002 test targets.
- S3-08: Economy Interest Snapshot & Resolution End implemented on branch
  `work/s3-08-economy-interest-snapshot` at `db61102`; merged into `main` at
  `4961356`. Local `economy_interest_snapshot_test`, `cargo check -p server`,
  and `cargo fmt --check` passed.

## Recently Closed

- CA-001: implemented at `05dc190`; story-done committed and pushed at
  `c4c3fa9`.
- AUC-003: implemented at `44afdb5`; story-done committed and pushed at
  `579db68`.
- CS-002: implemented at `20b24fa`; story-done committed and pushed at
  `bd3487a`.
- KW-002: implemented at `7fe9b5d`; tracking claim pushed at `699c227`;
  story-done committed and pushed at `765ecfc`.
- CARD-ANIM-001: implemented at `23fad70`; story-done committed and pushed at
  `ab7d56f`.

## Story-Done Queue

1. S3-06
2. S3-04
3. S3-05
4. CA-002
5. KW-003
6. HUD-001
7. S3-08
8. CARD-ANIM-002

Run only one story-done at a time.

## Launch Blocks / Wait Conditions

- CA-003 / CA-006: CA-002 is implemented but not story-done. CA-003 depends only
  on CA-001 and can be launched in worktree mode if it avoids unmerged root
  dirty files; story-done remains serialized.
- KW-004: waits for KW-003 story-done if the story depends on first-strike
  behavior; otherwise can be launched in worktree mode after checking story
  dependencies.
- CARD-ANIM-002 / CARD-ANIM-004 / CARD-ANIM-009: unblocked by
  CARD-ANIM-001 story-done; launch in worktree mode.
- HAND-UI-001: can launch after HUD-001 handoff or in worktree mode if it
  clearly avoids HUD files.
- AUC-004: do not start yet; story depends on economy-system story-005 and has
  OQ9 pre-implementation gate.
- Prism stories: blocked until NP OQ1 Lightyear 0.26 unicast API and hand-write
  API alignment with Card Acquisition are confirmed.

## Next Parallel Launch Candidates

- BOARD-001: Board Grid Initialization; no dependencies.
- CARD-ANIM-004: AnimQueue resolution drain; depends on CARD-ANIM-001 done.
- CARD-ANIM-009: CI boundary enforcement; depends on CARD-ANIM-001 done.
- CARD-ANIM-008: Input Gating; depends on CARD-ANIM-002 implementation, now
  available but should wait for CARD-ANIM-002 story-done unless explicitly
  pulled in worktree mode.
- HUD-002: Gold/Mana Display; depends on HUD-001 implementation, now available
  but should wait for HUD-001 story-done unless explicitly pulled in worktree
  mode with caveat.
- HUD-003: Phase Label/Round Counter; depends on HUD-001 implementation, now
  available but should wait for HUD-001 story-done unless explicitly pulled.

## Resolved Design Gates

- OQ-KS9: resolved in `design/gdd/combat-resolution.md` via `f8ceafd`.
- OQ-HUD-05: resolved in HUD story 004 via `64b0cfd`; HUD story 004 still has
  other blockers and should not be implemented yet.
- KW-SC-1: `On<UnitDied>` observer param compile probe passed with
  `cargo check -p server`; no permanent files were needed.

## Current Dirty-Tree Notes

As of the last check after HUD-001 handoff, root dirty files are limited to
untracked `.codex-tmp/` and manually generated `assets/art/`. Use worktree mode
for all new code workers.
