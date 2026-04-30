# Sprint 3 -- 2026-05-28 to 2026-06-10

## Sprint Goal

Complete the remaining Core Sprint B backbone: lobby/class flow, RSM timers/input + win-condition path, and the last networking proof so Core subscribers can integrate safely.

## Capacity

- Total days: 10
- Buffer (20%): 2 days reserved for unplanned work / verification surprises
- Available: **8 effective days**

---

## Producer Feasibility Gate

PR-SPRINT skipped -- Lean mode.

Must Have scope totals 7.5 effective days, leaving 0.5 days of active working slack inside the 8-day capacity after buffer. Should Have and Nice to Have tasks are explicit pull-forward items only.

---

## Tasks

### Must Have (Critical Path)

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S3-01 | [GSS Story 1: Lobby Scaffold](../epics/game-session-system/story-001-lobby-scaffold.md) | gameplay-programmer | 1.0 | Foundation shared/protocol types | `server/src/core/session/` scaffold exists; `SessionReady`, `SessionConfig`, lobby resources compile; session scaffold tests pass |
| S3-02 | [GSS Story 2: Room Create and Join](../epics/game-session-system/story-002-room-create-join.md) | network/gameplay-programmer | 1.5 | S3-01, S2-09 | Create/join handlers enforce one active session per player; idempotent create works; slot state broadcasts full slot vector |
| S3-03 | [GSS Story 3: Class Selection and Reveal](../epics/game-session-system/story-003-class-selection-reveal.md) | gameplay-programmer | 1.0 | S3-02 | Preview never broadcasts; confirm locks atomically; `S2CClassesRevealed` broadcasts once after all occupied slots lock |
| S3-04 | [RSM Story 3: Timers and Input Reader](../epics/round-state-machine/story-003-timers-and-input-reader.md) | gameplay-programmer | 1.5 | S2-07 | RSM timers use `GameConfig`; stale inbound messages are discarded; ready/submission exits and timer expiries transition correctly |
| S3-05 | [RSM Story 4: Win Condition and Game Over](../epics/round-state-machine/story-004-win-condition-and-game-over.md) | gameplay-programmer | 1.5 | S3-04 | Objective-loss, no-loss, and draw paths emit correct `GameOverEmitted`; `BroadcastPhaseChanged` remains last; no `server::feature` imports |
| S3-06 | [Carryover: E2E WebSocket Round-Trip](../epics/lightyear-protocol-verification/story-004-e2e-websocket-roundtrip.md) | network-programmer | 1.0 | S2-09 | Heartbeat round-trip test passes; reliable-channel proof exists; WASM bundle size evidence documented |

### Should Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S3-07 | [Card Pool Story 4: ShopRefreshNeeded Subscriber + SessionReady Init](../epics/card-data-pool/story-004-shop-refresh-subscriber-session-ready.md) | gameplay-programmer | 1.5 | S2-04, S3-01, S2-07 | `SessionReady` initializes per-player pools; `ShopRefreshNeeded` fills independent shops/offering; `ManualRefreshCount` resets |
| S3-08 | [Economy Story 3: Interest Snapshot & Resolution End](../epics/economy-system/story-003-interest-snapshot-resolution.md) | gameplay-programmer | 1.0 | S2-08, S2-07 | Resolution snapshot captures gold; current mana discarded; stale snapshot overwritten; interest cap path tested |

### Nice to Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S3-09 | [GSS Story 4: F4 Predicate and SessionReady Trigger](../epics/game-session-system/story-004-f4-session-ready.md) | gameplay-programmer | 1.5 | S3-01, S3-02, S3-03, ADR-012 evidence refresh | Story readiness revalidated against ADR-012; `SessionReady` fires once; lobby-to-draft same-tick integration passes |
| S3-10 | [Card Pool Story 5: Manual Refresh + Cost Escalation](../epics/card-data-pool/story-005-manual-refresh-cost-escalation.md) | gameplay-programmer | 1.0 | S3-07, Economy API | Manual refresh costs escalate 1g/2g/3g; insufficient gold and wrong phase do not mutate state |

---

## Carryover from Previous Sprint

| Task | Reason | New Estimate |
|------|--------|--------------|
| S2-10 E2E WebSocket Round-Trip -> S3-06 | Still `in-progress` in `production/sprint-status.yaml` at Sprint 3 planning time | 1.0d |

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Sprint 3 has no QA plan yet | HIGH | HIGH | Run `/qa-plan sprint` before implementation starts; do not treat stories as QA-ready until test expectations are defined |
| GSS Story 004 file still says Blocked even though ADR-012 evidence later resolved the core observer-ordering concern | MEDIUM | HIGH | Keep S3-09 Nice to Have until `/story-readiness` refreshes the story file against `tests/evidence/lightyear-026-verification.md` |
| Lightyear 0.26 send/receive APIs differ from older assumptions | MEDIUM | HIGH | Use S1/S2 verification evidence and `liv-bevy-lightyear` on all networking work |
| S2-10 consumes Sprint 3 active slack | MEDIUM | MEDIUM | Keep S3-07 and S3-08 as pull-forward; only pull them once Must Have work is green |
| GSS room/session resources overlap with existing minimal `server/src/core/session/state.rs` | MEDIUM | MEDIUM | Read current implementation before editing; extend existing types rather than replacing user or prior-agent work |

---

## Dependencies on External Factors

- Developer PowerShell for VS 2026 or CI remains the authoritative Cargo verification path for local builds that need MSVC `link.exe`.
- GitHub Actions must stay green after each story merge.
- Lightyear 0.26 verified API evidence in `tests/evidence/lightyear-026-verification.md` must be treated as the networking source of truth.

---

## QA Plan

No Sprint 3 QA plan exists at planning time.

> **No QA Plan**: This sprint was started without a QA plan. Run `/qa-plan sprint`
> before the last story is implemented. The Production -> Polish gate requires a QA
> sign-off report, which requires a QA plan.

---

## Definition of Done for this Sprint

- [ ] All Must Have tasks completed
- [ ] All tasks pass acceptance criteria
- [ ] QA plan exists (`production/qa/qa-plan-sprint-3.md`)
- [ ] All Logic/Integration stories have passing unit/integration tests
- [ ] Smoke check passed (`/smoke-check sprint`)
- [ ] QA sign-off report: APPROVED or APPROVED WITH CONDITIONS (`/team-qa sprint`)
- [ ] No S1, S2, or S3 bugs in delivered features
- [ ] Design documents updated for any deviations
- [ ] Code reviewed and merged

---

**Scope check:** If this sprint includes stories added beyond the original epic scope, run `/scope-check [epic]` to detect scope creep before implementation begins.
