# Sprint 6 Deferred Visual/Manual Evidence - 2026-05-06

| Field | Value |
|---|---|
| QA condition | QA-COND-0007: Deferred Manual Visual Evidence |
| Pass scope | Hand UI visual evidence reduction only |
| Stories | HAND-UI-009, HAND-UI-010, HAND-UI-011, HAND-UI-014 |
| Path | Evidence-only for shipping Hand UI behavior; added a QA harness/capture tool and artifacts |
| Build commit | `7cad0fac41c9c7a031af7283a2ad87d366a4e7bf` |
| Worker branch | `work/qa-cond-0007-hand-ui-visual-evidence` |
| Capture root | `production/qa/evidence/captures/qa-cond-0007-hand-ui/` |
| Trace | `production/qa/evidence/captures/qa-cond-0007-hand-ui/qa-cond-0007-hand-ui-trace.json` |
| Browser/env | Chrome `147.0.7727.139`, PowerShell CDP capture, Trunk WASM harness, `http://127.0.0.1:8083/`, 1366x768 viewport, UI scale 100% |
| Input method | Deterministic ECS placement, reserve, timer, and submit sequence against the real `HandUiPlugin` |

## Scope Boundary

This pass covers the Hand UI evidence gap only:

- placement timer normal/urgent state and submitted checkmark
- reserve strip affordance, including plus/minus and disabled increment ceiling
- submit validation inline/correction state and corrected successful submit

Resolution replay readability is excluded from this evidence file. BR-006 owns
that path separately. Full playable-client manual QA is not claimed.

## Capture Command

```text
powershell -ExecutionPolicy Bypass -File production\qa\evidence\captures\qa-cond-0007-hand-ui\qa-cond-0007-hand-ui-capture.ps1 -ReadyTimeoutSeconds 600 -TrunkPort 8083 -DebugPort 9226
```

## Evidence Matrix

| Required item | Exact steps | Expected | Actual | Screenshot |
|---|---|---|---|---|
| Normal placement timer | Enter PLACEMENT with `timer_duration_ms = 10000`, one playable hand card, current/reserve mana 3/3. | Timer is visible before urgency threshold in `TimerState::Normal` with a whole-second label. | Timer text `10`; state `Normal`; `remaining_ms = 10000`; urgency fired `false`; urgency audio count `0`; Submit active. | `production/qa/evidence/captures/qa-cond-0007-hand-ui/01-normal-placement-timer.png` |
| Urgent timer at <=5s | Set `PlacementTimer.remaining_ms = 5001`, `urgency_fired = false`; advance virtual time by 2ms. | Crossing to <=5s sets `TimerState::Urgent` and emits exactly one `TimerUrgencyAudio`. | Timer text `5`; state `Urgent`; `remaining_ms = 4999`; urgency fired `true`; urgency audio count `1`; Submit active. | `production/qa/evidence/captures/qa-cond-0007-hand-ui/02-urgent-timer-leq-5s.png` |
| Submitted checkmark | Start PLACEMENT; set timer to 7000ms; click Submit; advance virtual time by 16ms. | Submit succeeds, timer remains visible/running, and submitted checkmark appears. | Submit text `Submitted`; Submit state `Inactive`; `remaining_ms = 6984`; checkmark visible `true`; submissions `1`. | `production/qa/evidence/captures/qa-cond-0007-hand-ui/03-submitted-checkmark.png` |
| Reserve strip with +/- | Stage a cost-3 Minion to lane 1 cell 1 with current/reserve mana 3/3. | Staged card shows reserve/current split text with decrement and increment controls visible. | Reserve strip visible `true`; reserve text `Reserve 0 Current 3`; minus visible `true`; plus visible `true`; plus disabled `false`; fan slot `Ghost`. | `production/qa/evidence/captures/qa-cond-0007-hand-ui/04-reserve-strip-plus-minus.png` |
| Disabled reserve ceiling state | From the staged cost-3 card, press reserve `+` three times. | At the reserve ceiling, the increment control disables and further increments are blocked. | Reserve text `Reserve 3 Current 0`; plus disabled `true`; minus disabled `false`; Submit remains active; pending placements `1`. | `production/qa/evidence/captures/qa-cond-0007-hand-ui/05-disabled-reserve-ceiling.png` |
| Invalid submit inline/correction state | New PLACEMENT fixture with current/reserve mana 0/3; stage cost-3 Minion; press `+` once; click Submit. | Current/reserve overdraw blocks submit, keeps Submit active, attaches inline correction state, and sends nothing. | Guidance `Adjust reserve/current mana`; disclosure `Correction(ManaOverdrawn)`; Submit active; error `ManaOverdrawn`; submissions `0`; checkmark hidden. | `production/qa/evidence/captures/qa-cond-0007-hand-ui/06-invalid-submit-inline-correction.png` |
| Corrected successful submit | From the invalid-submit fixture, press reserve `+` twice more so split is `Reserve 3 Current 0`; click Submit. | Corrected split clears the error, sends exactly once, disables Submit, and shows the checkmark. | Guidance `Placement submitted`; disclosure `Submitted`; Submit text `Submitted`; Submit inactive; error none; submissions `1`; checkmark visible. | `production/qa/evidence/captures/qa-cond-0007-hand-ui/07-corrected-successful-submit.png` |

## Verification Notes

The harness uses the real Bevy 0.18 `HandUiPlugin` resources, messages, and UI
entities. It records state from `PlacementTimer`, `TimerState`,
`TimerUrgencyAudio`, `TimerSubmittedCheckmark`, `ReserveStripButtonDisabled`,
`SubmitValidationError`, `PlacementDisclosureState`, `PendingPlacements`, and
`HandUiOutboundMessages`.

No Board Rendering or resolution replay code was edited. No
`production/sprint-status.yaml` or `production/session-state/**` files were
edited. QA-COND-0005 and QA-COND-0006 accepted-risk dispositions were not
changed.

## QA-COND-0007 Impact

Hand UI evidence for the listed QA-COND-0007 items is complete:

- placement timer urgency/checkmark: evidenced
- reserve strip affordance: evidenced
- submit validation inline feedback: evidenced

QA-COND-0007 is not closed by this pass because resolution replay readability
evidence is still excluded and belongs to BR-006. The condition remains Open /
Needs Evidence until the Board Rendering replay evidence also exists or the
remaining path is separately dispositioned.
