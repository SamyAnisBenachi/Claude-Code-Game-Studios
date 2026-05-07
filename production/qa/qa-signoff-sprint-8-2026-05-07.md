# QA Sign-Off Report: Sprint 8

**Date**: 2026-05-07
**QA cycle**: QA sign-off report only; no `/team-qa` run is claimed
**Stage**: Polish
**Scope source**: `production/sprint-status.yaml` - Sprint 8
**Sprint plan**: `production/sprints/sprint-8.md`
**QA plan**: `production/qa/qa-plan-sprint-8-2026-05-07.md`
**Smoke report**: `production/qa/smoke-sprint-8-2026-05-07.md`
**Smoke verdict**: PASS WITH WARNINGS
**Current source of truth**: `origin/main@c31645c93cc91701f56e5960abb4a46dca80ba5f`
**Latest green CI evidence**: `tests.yml` run `25506133839`, `success`, at `efbc9e4ed58128f385fb8bceea18302a4c830c17`
**QA Lead sign-off**: Approved with conditions

## Verdict

**APPROVED WITH CONDITIONS**

Sprint 8 QA sign-off is approved with conditions for the scoped internal
1v1 friend-game loop robustness goal only. The sign-off is supported by:

- S8-QA-001 smoke result: **PASS WITH WARNINGS**.
- Latest referenced green `tests.yml` CI run `25506133839`: **success**.
- PLAYABLE-004: Complete.
- LOOP-001: Complete.
- SAU-007: Complete.
- CONTENT-001A: reconciled as Sprint 8 supporting content under CONTENT-001.
- Sprint 8 evidence index integrated at
  `production/qa/evidence/sprint-8-friend-game-evidence-index.md`.

The approval is conditional because the full browser/native two-client manual
GUI route was not captured. Manual/browser `GAME_OVER` evidence is not claimed.
The controlled real-Lightyear evidence and CI are sufficient for this scoped
internal friend-game QA sign-off, but not for full playable-client manual QA or
public/release readiness.

This report does not run or replace `/dev-story`, `/story-done`,
`/smoke-check`, `/team-qa`, `/gate-check`, Sprint 8 close-out, public release
readiness, release-candidate readiness, broad accessibility completion,
playtest validation, full playable-client manual QA, or full game completion.

## Tested Scope

- Sprint 8 QA plan, smoke report, evidence documents, evidence index, sprint
  status, and session-state context.
- PLAYABLE-004 result endpoint evidence for internal friend-game `GAME_OVER`
  coverage through controlled real-Lightyear server/client routing.
- LOOP-001 repeated active-loop polish evidence for DRAFT_SHOP, auction,
  placement, resolution, stale-state cleanup, and authority boundaries.
- SAU-007 auction settlement and shop-transition evidence.
- CONTENT-001A runtime card variety floor reconciliation as supporting Sprint 8
  content only.
- QA-COND-0005 and QA-COND-0006 condition files as accepted-risk context only.
- Green CI run `25506133839`, including successful dep-purity jobs, cargo
  tests, and WASM bundle size check.

Out of scope for this sign-off:

- No public, external, commercial, store, deployment, release-candidate, or
  public launch readiness is claimed.
- No broad Standard-tier accessibility completion is claimed.
- No playtest evidence, fun-hypothesis validation, or playtest report is
  claimed.
- No full playable-client manual QA is claimed.
- No manual/browser `GAME_OVER` route is claimed.
- No full game completion is claimed.

## Evidence Summary

| Item | Evidence | Result |
|---|---|---|
| Sprint 8 QA plan | `production/qa/qa-plan-sprint-8-2026-05-07.md` | PASS for planning and scope guard coverage |
| S8-QA-001 smoke package | `production/qa/smoke-sprint-8-2026-05-07.md` | PASS WITH WARNINGS |
| Sprint 8 friend-game evidence | `production/qa/evidence/sprint-8-friend-game-loop-evidence.md` | PASS WITH CONDITIONS for scoped controlled friend-game evidence |
| Sprint 8 evidence index | `production/qa/evidence/sprint-8-friend-game-evidence-index.md` | PASS |
| PLAYABLE-004 result endpoint | `production/qa/evidence/captures/sprint-8-friend-game-loop/playable-004-result-endpoint-trace.json` | PASS for controlled real-Lightyear internal friend-game `GAME_OVER` endpoint evidence |
| LOOP-001 active-loop polish | `production/qa/evidence/captures/sprint-8-friend-game-loop/loop-001-active-loop-polish-trace.json` | PASS for repeated active-loop stability evidence |
| SAU-007 settlement transition | `production/qa/evidence/shop-auction-ui-settlement-transition-evidence.md` | PASS |
| CONTENT-001A support slice | `production/sprint-status.yaml` and `production/qa/evidence/sprint-8-friend-game-loop-evidence.md` | Reconciled as supporting content only |
| CI | `tests.yml` run `25506133839` | PASS |

PLAYABLE-004 reached this controlled internal friend-game route:

`DRAFT_INITIAL -> PLACEMENT(empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(endpoint) -> RESOLUTION -> GAME_OVER`.

LOOP-001 active-loop evidence covers:

`DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP`.

## CI Summary

`tests.yml` run `25506133839` completed successfully on
`efbc9e4ed58128f385fb8bceea18302a4c830c17`.

| Job | Status |
|---|---|
| `shared/ dep purity` | success |
| `client/ dep purity` | success |
| `server/ dep purity` | success |
| `Run Cargo Tests` | success |
| `WASM bundle size check` | success |

No failed CI jobs or failed CI steps were found in run `25506133839`. The
`Run Cargo Tests` job completed successfully and is the green CI source used for
this sign-off.

## Smoke Evidence Summary

S8-QA-001 supports conditional sign-off with the following status:

- Automated and controlled real-Lightyear evidence: PASS.
- Required command gate in the smoke report: PASS.
- `cargo check --workspace`: PASS.
- `git diff --check`: PASS in the smoke package.
- Full two-window native or browser manual GUI route: not captured.
- Manual/browser `GAME_OVER`: not claimed.

Carried S8-QA-001 warning:

The full browser/native two-client manual GUI route was not captured. The smoke
package uses committed controlled real-Lightyear traces and command evidence as
the scoped Sprint 8 QA basis. That is enough for internal friend-game QA
sign-off with conditions, but it is not full manual/browser playable-client QA.

## QA Condition Disposition

| ID | Required Disposition | Sign-Off Status |
|---|---|---|
| QA-COND-0005 | Accepted risk for friend-game scope only; not verified Standard-tier accessibility completion. | Carried as friend-game-only accepted risk. This sign-off does not mark Standard-tier accessibility complete and does not apply to public, external, commercial, release-candidate, or broader accessibility scope. |
| QA-COND-0006 | Accepted-risk/deferred; not playtest evidence or fun-hypothesis validation. | Carried as accepted-risk/deferred. This sign-off does not claim playtest sessions, a playtest report, fun-hypothesis validation, or fun-hypothesis revision. |

No QA-COND-0005 or QA-COND-0006 disposition files were edited by this sign-off.

## Conditions And Residual Risks

- S8-QA-001-W1 remains a bounded manual/browser smoke evidence gap. Full
  browser/native two-client manual route capture is deferred and not claimed.
- Manual/browser `GAME_OVER` is not claimed.
- QA-COND-0005 remains accepted risk for friend-game scope only. Standard-tier
  accessibility is not verified complete.
- QA-COND-0006 remains accepted-risk/deferred. Sprint 8 friend-game evidence is
  not playtest evidence and does not validate or revise the fun hypothesis.
- CONTENT-001A is supporting content only. It is not full card production, full
  balance completion, or content-production sign-off.
- This sign-off is not public release readiness.
- This sign-off is not release-candidate readiness.
- This sign-off is not full playable-client manual QA.
- This sign-off is not full game completion.

## Bugs Found

No new QA bug files were created by this sign-off.

No new S1/S2 blocker is identified in the scoped Sprint 8 friend-game evidence
reviewed for this sign-off. QA-COND-0005 and QA-COND-0006 remain accepted-risk
conditions, not verified completion evidence.

## Blockers

None for scoped Sprint 8 QA sign-off.

Sprint 8 can proceed to any later status close-out workflow from a QA sign-off
perspective only if the carried conditions remain explicit. Any later close-out
must not imply public release readiness, release-candidate readiness, broad
accessibility completion, playtest validation, full playable-client manual QA,
manual/browser `GAME_OVER`, or full game completion.

## Changed Files

- `production/qa/qa-signoff-sprint-8-2026-05-07.md`

No source files, test files, `production/sprint-status.yaml`,
`production/session-state/**`, condition files, smoke reports, gate-check
records, story files, or Sprint 8 close-out files were edited for this
sign-off.
