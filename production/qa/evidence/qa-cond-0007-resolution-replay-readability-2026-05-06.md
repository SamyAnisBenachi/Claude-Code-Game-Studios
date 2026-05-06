# QA-COND-0007 Resolution Replay Readability Evidence - 2026-05-06

| Field | Value |
|---|---|
| QA condition | QA-COND-0007: Deferred Manual Visual Evidence |
| Pass scope | Resolution replay readability only |
| Stage | Polish |
| Path | Evidence plus focused WASM capture harness |
| BR-006 implementation commit | `8caa1a0195fd817b1ce632877db2174a357e8162` |
| BR-006 story-done commit / capture source commit | `484bef101c16bc7931456b2c0f72676279d7a536` |
| Capture root | `production/qa/evidence/captures/qa-cond-0007-resolution-replay/` |
| Trace | `production/qa/evidence/captures/qa-cond-0007-resolution-replay/qa-cond-0007-resolution-replay-trace.json` |
| Browser/env | Chrome CDP capture, Trunk WASM harness, `http://127.0.0.1:8084/`, 1366x768 viewport, UI scale 100% |
| Input method | Deterministic ECS replay script using `BoardRenderingPlugin`, `CardAnimationsPlugin`, `AnimQueue`, `PendingPhaseChange`, and `SnapshotRecoveryRequested` |

## Scope Boundary

This pass covers only the missing QA-COND-0007 resolution replay readability
slice. It does not run or claim full playable-client manual QA. It does not
change gameplay rules, server combat resolution, networking, HUD behavior, hand
UI behavior, sprint status, or story-done records.

The previously integrated QA-COND-0007 Hand UI evidence covers placement timer
urgency/checkmark, reserve strip affordance, and submit validation inline
feedback. This file covers the remaining resolution replay readability item.

## Capture Command

```text
powershell -ExecutionPolicy Bypass -File production\qa\evidence\captures\qa-cond-0007-resolution-replay\qa-cond-0007-resolution-replay-capture.ps1 -ReadyTimeoutSeconds 600 -TrunkPort 8084 -DebugPort 9227
```

## Evidence Matrix

| Required item | Exact steps | Expected | Actual | Screenshot |
|---|---|---|---|---|
| Replay queue loaded | Seed a valid 3-group `S2CResolutionEvent`, enter `ResolutionReveal`, start `ResolutionRevealWait`, and buffer `DraftShop` in `PendingPhaseChange`. | Queue loads through BR-006 infrastructure and starts in `ResolutionExecuting`; current phase remains `Resolution`; `DraftShop` is pending. | Harness report shows `currentPhase=Resolution`, `boardRenderState=ResolutionExecuting`, `currentSubStep=1`, `pendingPhase=DraftShop`, `queueEmpty=false`. | `production/qa/evidence/captures/qa-cond-0007-resolution-replay/01-replay-start.png` |
| First result readability / no premature phase jump | Advance virtual time to 599ms, before the first 600ms group boundary. | First result is still readable; next phase has not applied while queue has active work. | Harness report shows `currentSubStep=1`, `currentPhase=Resolution`, `pendingPhase=DraftShop`, `phaseJumpBlocked=true`, `queueEmpty=false`. | `production/qa/evidence/captures/qa-cond-0007-resolution-replay/02-replay-mid-first-sub-step.png` |
| Result progression | Advance through the first group boundary and 150ms inter-step pause. | Replay advances to the second result group only after the pause, with the next phase still buffered. | Harness report shows `currentSubStep=2`, `currentPhase=Resolution`, `pendingPhase=DraftShop`, `phaseJumpBlocked=true`. | `production/qa/evidence/captures/qa-cond-0007-resolution-replay/03-replay-second-sub-step.png` |
| Final result still buffered | Advance to 2,099ms, one millisecond before final group drain. | Final result remains readable and `DraftShop` is still blocked until replay drain. | Harness report shows `currentSubStep=3`, `currentPhase=Resolution`, `pendingPhase=DraftShop`, `phaseJumpBlocked=true`. | `production/qa/evidence/captures/qa-cond-0007-resolution-replay/04-replay-final-sub-step-buffered.png` |
| After replay drain | Advance to 2,100ms, crossing the final group boundary. | Queue clears and the buffered `DraftShop` phase applies only after replay completion. | Harness report shows `currentPhase=DraftShop`, `phaseView=DraftShop`, `pendingPhase=none`, `queueEmpty=true`, `boardRenderState=DraftShop`. | `production/qa/evidence/captures/qa-cond-0007-resolution-replay/05-replay-drained-next-phase.png` |
| Recovery behavior | Seed an invalid replay script with sub-step 7. | Script is rejected, pending replay is cleared, queue is reset, and exactly one authoritative snapshot request is emitted. | Harness report shows one `ResolutionSubStepOutOfRange` request, no duplicate on the next update, `pendingScriptPresent=false`, and `queueEmpty=true`. | `production/qa/evidence/captures/qa-cond-0007-resolution-replay/06-recovery-snapshot-requested.png` |

## Verification Notes

The harness uses the BR-006 queue and phase-buffering resources directly:
`PendingResolutionScript`, `ResolutionRevealWait`, `AnimQueue`,
`PendingPhaseChange`, `CurrentClientPhase`, `ClientPhaseView`, and
`SnapshotRecoveryRequested`. The replay script is grouped through
`resolution_anim_groups_from_script`, preserving sub-step sorting and trigger
order.

Trace verdict fields are expected to be:

- `resultProgressionPass=true`
- `phaseBufferingPass=true`
- `recoveryPass=true`
- `readyForCapture=true`
- `playableClientManualQaClaimed=false`

## QA-COND-0007 Impact

QA-COND-0007 is closeable after this pass because all listed evidence items now
have committed evidence:

- placement timer urgency/checkmark: evidenced by
  `production/qa/evidence/deferred-visual-manual-sprint-6-2026-05-06.md`
- reserve strip affordance: evidenced by
  `production/qa/evidence/deferred-visual-manual-sprint-6-2026-05-06.md`
- submit validation inline feedback: evidenced by
  `production/qa/evidence/deferred-visual-manual-sprint-6-2026-05-06.md`
- resolution replay readability: evidenced by this file and its committed
  capture artifacts

This closure is visual/harness evidence only. It does not claim full
playable-client manual QA.
