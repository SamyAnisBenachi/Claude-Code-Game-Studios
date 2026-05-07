# Sprint 9 Manual Evidence Harness Prep

> **Status**: Prepared plan only. No evidence claimed.
> **Prepared**: 2026-05-07
> **Source of truth**: `origin/main@e542e1319f99b91b3f6ad832b38ce8d103c5bad8`
> **Scope**: Sprint 9 manual/native/browser friend-game evidence planning.

This document prepares the manual evidence harness plan for Sprint 9 after the
native operator controls and Result Screen MVP land. It does not implement a
helper, does not claim manual evidence, and does not close `S8-QA-001-W1`.

The prompt-level integration points referenced by this prep are treated as:

- `397`: native operator controls integration, mapped to `S9-NATIVE-001`.
- `416`: Result Screen MVP integration, mapped to `S9-RS-002`.

No local source document names those prompt numbers directly, so the concrete
source-of-truth gates below use the Sprint 9 story IDs and files.

## Audit Summary

Audited source documents:

- `production/qa/evidence/manual-friend-game-evidence-runbook.md`
- `production/sprints/sprint-9.md`
- `production/sprint-status.yaml`
- `production/epics/playable-client/story-006-native-friend-game-operator-controls.md`
- `production/epics/presentation-layer/story-006-result-screen-mvp.md`
- `production/epics/game-session-system/story-010-result-acknowledgement-cleanup-handshake.md`
- `production/epics/playable-client/story-007-manual-browser-game-over-evidence-closure.md`
- `production/epics/playable-client/story-008-sprint-9-result-evidence-index-cleanup.md`
- `production/qa/evidence/sprint-8-friend-game-evidence-index.md`
- `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`

Findings:

- The manual evidence runbook exists and already defines the route, artifact
  package, screenshots/logs, defects table, result-screen checks, and non-claim
  boundaries needed for a future run.
- The runbook is a route execution runbook, not a Sprint 9 harness readiness
  disposition. It is also pinned to an older source baseline than the current
  `origin/main` used for this audit.
- Sprint 8 controlled internal real-Lightyear evidence reached `GAME_OVER`, but
  Sprint 8 did not capture a full manually driven browser/native two-client GUI
  route through `GAME_OVER`.
- Sprint 9 has no QA plan yet. Sprint 9 docs explicitly keep QA sign-off and
  gate claims unavailable until QA planning and evidence exist.
- `S9-NATIVE-001` is ready but not complete on the audited baseline.
- `S9-RS-002` is ready but not complete on the audited baseline.
- `S9-RS-003` remains blocked until the Result Screen MVP is complete. Full
  S9-QA-001 closure needs the Return to Lobby acknowledgement behavior, not only
  a visible result overlay.
- `S9-QA-001` remains blocked until operator/browser controls, result screen,
  and acknowledgement cleanup are usable.
- `S9-QA-002` remains blocked until S9-QA-001 produces evidence or a blocker
  record.

## Post-Integration Preconditions

Before attempting Sprint 9 manual evidence capture, record these checks in the
future evidence package:

| Gate | Required check | Blocking disposition if absent |
|---|---|---|
| Source baseline | `git rev-parse HEAD` and `git status --short --branch` recorded from the run commit | Stop. Do not capture evidence against an unknown or dirty source unless the dirt is explicitly documented and approved. |
| S9-NATIVE-001 or browser route | Native operator controls complete, or browser route is chosen and verified usable for manual operation | Keep S9-QA-001 blocked. Record the missing operator path as the blocker. |
| S9-RS-002 | Result Screen MVP complete and integrated, with `production/qa/evidence/result-screen-mvp-evidence.md` or equivalent story evidence available | Route may only capture up to `GAME_OVER`; do not claim rendered result-screen evidence. |
| S9-RS-003 | Return to Lobby acknowledgement cleanup handshake complete or explicitly replaced by an approved fallback | Do not close S8-QA-001-W1. Record result screen reached but acknowledgement incomplete. |
| S8 carried warnings | `S8-QA-001-W1`, `QA-COND-0005`, and `QA-COND-0006` are still represented honestly | Stop any closure wording that expands the claim beyond actual evidence. |
| Output folder | Dated capture folder created under `production/qa/evidence/captures/sprint-9-manual-game-over/` or a dated subfolder | Do not scatter artifacts across unrelated evidence folders. |

## Required Evidence After 397 And 416 Integrate

After native controls and Result Screen MVP are integrated, capture or explicitly
mark blocked for every item below.

| Evidence area | Required capture | Notes |
|---|---|---|
| Command summary | Branch, commit, OS/target, server/client commands, ports, env vars, tool versions, start/stop times, dirty state if any | Use the same commit for one local server and both clients. |
| Server log | Startup, room create/join, class/session-ready path, phase changes, relevant C2S/S2C observations, `S2CGameOver`, acknowledgement handling, shutdown | Raw logs may be summarized if they are too noisy, but summaries must include exact commands and timestamps. |
| Client A log | Host launch, create room, class confirm, purchases, ready/retract, auction, placement, result, Return to Lobby | Redact room code and local machine details before committing. |
| Client B log | Joiner launch, join room, class confirm, purchases, ready/retract, auction, placement, result, Return to Lobby | Capture both sides even when one client blocks. |
| Screenshots or video | Lobby, class reveal, draft/shop, auction, placement, resolution, `GAME_OVER` result, Return to Lobby after acknowledgement if available | Browser route should include viewport; native route should include window size. |
| Route transcript | Manual action log from create/join through reached endpoint | Include the exact last successful step if blocked. |
| Traffic observations | `S2CRoomCreated`, `S2CJoinAck`, `S2CClassesRevealed`, draft/shop payloads, auction messages, placement reveal, resolution event before terminal phase, `S2CGameOver`, `S2CPhaseChanged(GameOver)` | Evidence must come from real traffic/logs or visible client state, not direct world injection or fake snapshots. |
| Result screen | Outcome headline/cause, final round if available, objective summary, alive opponent `Unknown` fallback, destroyed objective identity from authoritative data, HUD frozen behind overlay, rematch hidden/disabled | Capture at least the reached outcome. Other copy states can come from S9-RS-002 evidence, not from S9-QA-001 unless actually exercised. |
| Return to Lobby | Acknowledgement send disposition, local result UI cleanup, route back to lobby/menu, duplicate activation handling if observed | Required for full S9-QA-001 closure unless an approved fallback replaces S9-RS-003. |
| Defects | ID, severity, owner/system, status, reproduction, friend-game impact, workaround, evidence link | Blocking product defects keep S8-QA-001-W1 open. |
| Non-claims | Public release readiness, broad accessibility completion, playtest validation, fun-hypothesis validation, full playable-client manual QA, full regression campaign, full game completion | These must remain explicit in the evidence and later index. |

## Lightweight Capture Helper Disposition

A lightweight log/capture helper is recommended but not required before the
first S9-QA-001 attempt.

Recommendation rationale:

- The required package spans one server, two clients, screenshots/video, a route
  transcript, redaction, and a defects table. Sprint 8's manual/browser gap was
  partly an evidence-collection gap, so repeatable artifact setup is useful.
- A helper would reduce missing command/log metadata and make retries easier.
- A helper is not a substitute for the manual route. It must not drive gameplay,
  inject Bevy `World` state, fake snapshots, send harness-only C2S messages, or
  claim closure.

Permitted helper scope if a later implementation prompt approves it:

- Create the dated capture directory.
- Write a `command-summary.md` template populated with commit, branch, status,
  selected target, ports, env vars, and tool versions.
- Start server and client processes with stdout/stderr redirected to named log
  files, or provide exact commands for a human operator to run.
- Emit timestamped manual checkpoint prompts for the operator.
- Provide a `route-summary.json` template for the reached endpoint and blockers.
- Record process exit codes and stop times.
- Provide a redaction checklist for room codes, usernames, machine identifiers,
  transient secrets, and local paths that should not be committed.

Forbidden helper scope:

- No gameplay automation that replaces the human route.
- No direct server feature calls, Bevy `World` injection, fake snapshots, or
  client-side authority shortcuts as evidence.
- No smoke, QA sign-off, gate-check, story-done, sprint close-out, or condition
  closure.
- No automatic claim that `S8-QA-001-W1` is closed.

If no helper is built, the existing manual runbook is sufficient for a one-off
capture as long as the operator manually collects every required artifact and
records any missing artifact as a blocker or warning.

## Evidence Closure Boundary

Full S9-QA-001 closure requires actual artifacts showing the route from lobby
create/join through class confirmation, draft/shop, auction, placement,
resolution, `GAME_OVER`, Result Screen MVP display, and Return to Lobby
acknowledgement or approved fallback.

Partial outcomes are acceptable only as blocker records. They must preserve:

- `S8-QA-001-W1` unless the full manual/browser/native two-client route is
  actually captured.
- `QA-COND-0005` as accepted risk for friend-game scope only unless separate
  accessibility closure evidence exists.
- `QA-COND-0006` as accepted-risk/deferred unless separate playtest evidence
  exists.
- No public release readiness, release-candidate readiness, broad accessibility
  completion, playtest validation, full playable-client manual QA, full
  regression campaign, or full game completion claim.

## Recommended Next Docs Step

After the native controls, Result Screen MVP, and acknowledgement cleanup land,
use this prep plus `manual-friend-game-evidence-runbook.md` to write the actual
S9-QA-001 evidence document:

- `production/qa/evidence/sprint-9-manual-game-over-evidence.md`

Only after that document or a blocker record exists should S9-QA-002 create or
update:

- `production/qa/evidence/sprint-9-result-evidence-index.md`
