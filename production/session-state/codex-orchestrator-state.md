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

- HUD-002 worker: assumed launched from the last prompt; awaiting worker output.
- CARD-ANIM-002 story-done: completed at `43a2678`; window can be cleared.

## Tracker In-Progress But No Live Window Confirmed

These are marked `in-progress` in `production/sprint-status.yaml`, but the user
confirmed no corresponding agent window is currently running. Treat them as
stale/incomplete until explicitly relaunched or closed:
None currently tracked here.

## Recently Implemented, Needs Formal Story-Done

- CARD-ANIM-004: AnimQueue Resolution Drain implemented on branch
  `work/card-anim-004-animqueue-resolution-drain` at `2ecd58f`; merged into
  `main` at `b7204e5`. Local anim queue + plugin scaffold tests,
  `cargo check -p client`, and `cargo fmt -p client -- --check` passed after
  integration.
- CARD-ANIM-009: CI Boundary Enforcement implemented on branch
  `work/card-anim-009-ci-boundary-enforcement` at `55b5331`; cherry-picked into
  `main` at `75e11ea` because the branch was based before the asset-sorting
  commits. Local grep boundary check and `git diff --check` passed.
- CARD-ANIM-008: Input Gating implemented on branch
  `work/card-anim-008-input-gating` at `0d75fb0`; cherry-picked into `main` at
  `9308bf3` because the branch was based before recent tracking and feature
  commits. Local input gating test, plugin scaffold test, full `cargo test -p
  client`, `cargo check -p client`, `cargo fmt -p client -- --check`, and
  `git diff --check` passed. Manual CA-13b and CA-22 evidence remains pending
  until bid-button UI and DRAFT_INITIAL animation sequencing UI exist.
- KW-004: STUN State implemented on branch `work/kw-004-stun-state` at
  `7543293`; cherry-picked into `main` at `b8b1287` because the branch was
  based before recent tracking and BOARD-002 commits. Local `stun_test`,
  `cargo check -p server`, `cargo check -p shared`, `cargo fmt --all -- --check`,
  and `git diff --check` passed.
- BOARD-001: Board Grid Initialization implemented on branch
  `work/board-001-grid-initialization` at `7d38a34`; merged into `main` at
  `6e5d80b`. Local `board_grid_initialization_test`, `cargo check -p server`,
  and `cargo fmt -p server --check` passed.
- BOARD-002: Standard Unit Movement implemented on branch
  `work/board-002-standard-unit-movement` at `4a76028`; cherry-picked into
  `main` at `0d8e41c`. Local `standard_movement_test`, `cargo check -p server`,
  `cargo fmt -p server -- --check`, and `git diff --check` passed.
- CA-003: Card Acquisition Draw Pipeline implemented on branch
  `work/ca-003-draw-pipeline` at `c6200f0`; merged into `main` at `98cb52a`.
  Local draw pipeline suite, full server tests, `cargo check -p server`, and
  `cargo fmt --all` passed. `cargo check --workspace` was blocked on stale HUD
  BorderColor issue in the branch base; main has fixed it at `cbce522`.
- CA-006: Card Acquisition External Bypass implemented on branch
  `work/ca-006-external-bypass` at `6af1137`; merged into `main`. Local
  `card_acquisition_external_bypass_test` and `cargo check -p server` passed
  after integration.

## Recently Closed

- HUD-001: implemented at `b04748b`; Bevy 0.18 BorderColor fix at `cbce522`;
  test harness fix at `95b58ae`; story-done closed after
  `hud_plugin_scaffold_test` and `cargo check -p client` passed locally.
- S3-08: Economy Interest Snapshot & Resolution End implemented on branch
  `work/s3-08-economy-interest-snapshot` at `db61102`; merged into `main` at
  `4961356`; story-done committed at `4f838b6`.
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
- S3-06: E2E WebSocket Roundtrip implemented at `a32a3df`; HUD Bevy 0.18 WASM
  blocker fixed at `cbce522`; story-done committed and pushed at `57159e9`.
  Note: sprint-status marks S3-06 done but still has owner
  `codex-s3-06-websocket`; clean this in a later tracker hygiene pass if needed.
- S3-04: RSM Timers + Input Reader implemented at `eff5cf9`; blocker fixed at
  `ec6f433`/`61e45ad`; story-done committed at `1045dbc`.
- S3-05: RSM Win Condition and Game Over implemented at `5bf6bde`; story-done
  committed at `4d745a8`.
- CA-002: Card Acquisition Draft Initial implemented at `2c6c65b`; story-done
  committed at `79d5024`. `production/sprint-status.yaml` has no CA-002 entry,
  so the closeout updated only the story file and session state.
- KW-003: First Strike and Haste implemented at `874d86b`; story-done was
  absorbed into asset commit `bee8b47`, with acceptance checkbox/test-note
  cleanup finalized in a follow-up closure commit. `production/sprint-status.yaml`
  has no KW-003 entry, so the closeout updated only the story file and session
  state.
- CARD-ANIM-002: Tween Cancel/Replace Lifecycle implemented at `1354d5a` and
  merged into `main` at `e9103d9`; story-done closed after lifecycle tests,
  paired scaffold+lifecycle tests, and `cargo check -p client` passed locally.

## Story-Done Queue

1. BOARD-001
2. CA-003
3. CA-006
4. CARD-ANIM-004
5. CARD-ANIM-009
6. BOARD-002
7. KW-004
8. CARD-ANIM-008

Run only one story-done at a time.

## Launch Blocks / Wait Conditions

- CA-004 / CA-005: depend on CA-003 implementation, now available but should
  wait for CA-003 story-done unless explicitly pulled in worktree mode.
- CA-006: implemented and merged; pending story-done.
- KW-004: unblocked by KW-003 story-done; run readiness first because its story
  text may still contain stale ADR-018 Proposed/BLOCKED wording. Implemented and
  integrated; pending story-done.
- CARD-ANIM-009: implemented and integrated; pending story-done.
- CARD-ANIM-008: implemented and integrated; pending story-done. Manual CA-13b
  and CA-22 evidence remains deferred until the dependent UI exists.
- HAND-UI-001: unblocked by HUD-001 story-done; use worktree mode for any new
  implementation.
- AUC-004: do not start yet; story depends on economy-system story-005 and has
  OQ9 pre-implementation gate.
- Prism stories: blocked until NP OQ1 Lightyear 0.26 unicast API and hand-write
  API alignment with Card Acquisition are confirmed.

## Next Parallel Launch Candidates

- HAND-UI-001: plugin scaffold; unblocked by HUD-001 story-done. Use worktree
  mode.
- HUD-002: Gold/Mana Display; unblocked by HUD-001 story-done. Run readiness
  before launch.
- HUD-003: Phase Label/Round Counter; unblocked by HUD-001 story-done. Run
  readiness before launch.
- CARD-ANIM-006: Ready after CARD-ANIM-001; run readiness before launch because
  CARD-ANIM-002/004/008/009 are already implemented or pending closure.

## Resolved Design Gates

- OQ-KS9: resolved in `design/gdd/combat-resolution.md` via `f8ceafd`.
- OQ-HUD-05: resolved in HUD story 004 via `64b0cfd`; HUD story 004 still has
  other blockers and should not be implemented yet.
- KW-SC-1: `On<UnitDied>` observer param compile probe passed with
  `cargo check -p server`; no permanent files were needed.

## Current Dirty-Tree Notes

As of the asset sorting pass, generated art assets were moved into `assets/art/`
and committed. `.codex-tmp/` is ignored as a local scratch workspace. Use
worktree mode for all new code workers.
