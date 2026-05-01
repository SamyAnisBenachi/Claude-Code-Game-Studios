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
- Workers run local Developer PowerShell checks, commit explicit owned paths,
  push, and report commit hash plus CI run if available.
- Story-done windows are serialized because they edit shared production files.
- Keep commits scoped. If the pre-commit hook blocks due to mixed files, unstage
  and re-add explicit owned paths.

## Live Windows Confirmed By User

- S3-04 resume worker: RSM Timers + Input Reader.
- S3-06 resume worker: E2E WebSocket Roundtrip.

## Tracker In-Progress But No Live Window Confirmed

These are marked `in-progress` in `production/sprint-status.yaml`, but the user
confirmed no corresponding agent window is currently running. Treat them as
stale/incomplete until explicitly relaunched or closed:

- S3-08: Economy Interest Snapshot (`claude-s3-08-economy-interest`)

## Recently Implemented, Needs Formal Story-Done

- CARD-ANIM-001: implemented at `23fad70`.

## Recently Closed

- CA-001: implemented at `05dc190`; story-done committed and pushed at
  `c4c3fa9`.
- AUC-003: implemented at `44afdb5`; story-done committed and pushed at
  `579db68`.
- CS-002: implemented at `20b24fa`; story-done committed and pushed at
  `bd3487a`.
- KW-002: implemented at `7fe9b5d`; tracking claim pushed at `699c227`;
  story-done committed and pushed at `765ecfc`.

## Story-Done Queue

1. CARD-ANIM-001

Run only one story-done at a time.

## Launch Blocks / Wait Conditions

- S3-05: wait for S3-04 to complete; same RSM area.
- CA-002 / CA-003 / CA-006: wait for CA-001 story-done and local acquisition
  changes to settle.
- KW-003: unblocked by KW-002 story-done; safe to launch after current RSM
  dirty tree settles or in a new clean window that avoids RSM files.
- CARD-ANIM-002 / CARD-ANIM-004: wait for CARD-ANIM-001 story-done.
- HUD-001 / HAND-UI-001: hold until presentation scaffold churn from
  CARD-ANIM-001 is closed.
- AUC-004: do not start yet; story depends on economy-system story-005 and has
  OQ9 pre-implementation gate.
- Prism stories: blocked until NP OQ1 Lightyear 0.26 unicast API and hand-write
  API alignment with Card Acquisition are confirmed.

## Resolved Design Gates

- OQ-KS9: resolved in `design/gdd/combat-resolution.md` via `f8ceafd`.
- OQ-HUD-05: resolved in HUD story 004 via `64b0cfd`; HUD story 004 still has
  other blockers and should not be implemented yet.
- KW-SC-1: `On<UnitDied>` observer param compile probe passed with
  `cargo check -p server`; no permanent files were needed.

## Current Dirty-Tree Notes

As of the last check, the working tree contains active worker changes in
`server/src/core/economy/plugin.rs`, `server/src/core/rsm/events.rs`,
`server/src/core/rsm/mod.rs`, `server/src/core/rsm/plugin.rs`,
`server/src/core/rsm/state.rs`, `server/src/core/rsm/transitions.rs`, and
`server/src/core/rsm/system.rs`. Treat these as active S3-04/S3-06 worker output
until the user reports the worker handoff. Do not launch another RSM story
against the same files.
