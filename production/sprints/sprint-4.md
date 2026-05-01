# Sprint 4 -- 2026-06-11 to 2026-06-24

## Sprint Goal

Finish the Core handoff into a networked M1 loop by landing SessionReady, pool/shop initialization, game-over/disconnect/phase dispatch, and auction-ready economy validation.

## Planning Notes

- Target dates are planning targets and may move if Sprint 3 closes early or late.
- This plan intentionally does not replace `production/sprint-status.yaml`; Sprint 3 remains the active machine-readable tracker until it is formally closed.
- Sprint 4 assumes Sprint 3 must-haves close before kickoff. If Sprint 3 work remains open on 2026-06-11, carry it into Sprint 4 and defer Should Have or Nice to Have work first.
- PR-SPRINT skipped -- Lean mode.

## Capacity

- Total days: 10
- Buffer (20%): 2 days reserved for unplanned work / verification surprises
- Available: **8 effective days**
- Planned Must Have scope: **8.0 estimated days**
- Pull-forward scope: **7.0 estimated days**

---

## Tasks

### Must Have (Critical Path)

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S4-01 | [GSS Story 4: F4 Predicate and SessionReady Trigger](../epics/game-session-system/story-004-f4-session-ready.md) | unassigned | 1.5 | S3-03, ADR-012 evidence refresh | F4 predicate creates `SessionConfig` and `ServerRng`; `SessionReady` fires once; RSM enters `DraftInitial` same tick; ADR-012 verification documented |
| S4-02 | [Card Pool Story 4: ShopRefreshNeeded Subscriber + SessionReady Init](../epics/card-data-pool/story-004-shop-refresh-subscriber-session-ready.md) | unassigned | 1.5 | S4-01, S2-04, RSM event bus | `SessionReady` initializes per-player pools; `ShopRefreshNeeded` fills initial offering/shop slots; per-player pools remain isolated |
| S4-03 | [Card Pool Story 5: Manual Refresh + Cost Escalation](../epics/card-data-pool/story-005-manual-refresh-cost-escalation.md) | unassigned | 1.0 | S4-02, Economy API | Manual refresh costs escalate correctly; wrong phase and insufficient gold do not mutate state; counter resets on draft entry |
| S4-04 | [GSS Story 6: Game-Over Teardown](../epics/game-session-system/story-006-game-over-teardown.md) | unassigned | 1.0 | S4-01, S3-05 | `GameOverEmitted` broadcasts `S2CGameOver`; `SessionConfig` and `ServerRng` are removed; teardown is idempotent |
| S4-05 | [RSM Story 5: Disconnect Handling](../epics/round-state-machine/story-005-disconnect-handling.md) | unassigned | 1.0 | S3-05, Lightyear disconnect API evidence | Single disconnect, reconnect-within-grace, mutual disconnect draw, and mid-resolution deferral paths pass tests |
| S4-06 | [RSM Story 6: Network Dispatch Wiring](../epics/round-state-machine/story-006-network-dispatch-wiring.md) | unassigned | 1.0 | S4-05, Lightyear send API evidence | `BroadcastPhaseChanged` dispatches exactly one reliable `S2CPhaseChanged`; RSM keeps zero Lightyear sender imports |
| S4-07 | [Economy Story 5: Auction Reservation and Bid Validation](../epics/economy-system/story-005-auction-reservation-bid-validation.md) | unassigned | 1.0 | Economy Story 1 | Auction bid validation rejects hand-full/insufficient funds/too-low bids; reservation lifecycle and `spend_gold` tests pass |

### Should Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S4-08 | [Board Story 1: Board Grid Initialization](../epics/board-lane-system/story-001-board-grid-initialization.md) | unassigned | 1.0 | None | `BoardGrid`, `BoardOccupancy`, `SpawnRangeState`, `PrismState`, and `BoardConfig` resources initialize correctly |
| S4-09 | [Objective Story 1: Objective State Model](../epics/objective-system/story-001-objective-state-model.md) | unassigned | 1.0 | None | Each player has 5 objective slots; `ObjectiveHp`, `HiddenObjectives`, and `ObjectiveCounters` exist and initialize correctly |
| S4-10 | [Card Acquisition Story 1: State Scaffold](../epics/card-acquisition/story-001-state-scaffold.md) | unassigned | 1.0 | None | `ShopStates`, `PlayerHands`, `ShopRefreshTriggered`, and phase-gate scaffolding exist; AuctionLock silently discards C2S shop messages |
| S4-11 | [GSS Story 5: Lobby Disconnect Dual-Signal Cancel](../epics/game-session-system/story-005-lobby-disconnect-dual-signal.md) | unassigned | 1.0 | S3-02, Lightyear disconnect API evidence | Lobby disconnect, heartbeat gap, lobby timeout, and first-signal-wins cleanup paths pass tests |

### Nice to Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S4-12 | [Card Acquisition Story 2: Draft Initial - 9-Card Offering](../epics/card-acquisition/story-002-draft-initial.md) | unassigned | 1.0 | S4-10, S4-02 | DRAFT_INITIAL offering contains up to 9 distinct cards; refresh is discarded; offering is unicast only to target player |
| S4-13 | [Auction Story 3: AbortAuction Handler](../epics/auction-system/story-003-auction-abort-handler.md) | unassigned | 1.0 | AUC-002, S4-07, RSM `AbortAuction` | Live/Selecting abort returns auction to IDLE without `AuctionSettled`; Resolving remains uninterruptible |
| S4-14 | [Economy Story 6: Network Dispatch Wiring](../epics/economy-system/story-006-network-dispatch-wiring.md) | unassigned | 1.0 | Economy Story 2, Lightyear send API evidence | `S2CGoldUpdate` unicasts private economy state; `S2CGoldBroadcast` broadcasts public gold; both use ReliableChannel |

---

## Carryover from Previous Sprint

| Task | Reason | New Estimate |
|------|--------|--------------|
| Sprint 3 incomplete must-haves, if any remain at kickoff | `production/sprint-status.yaml` currently tracks Sprint 3 as active; Sprint 4 should not assume unfinished Sprint 3 work is magically complete | Defer Should/Nice scope first |
| GSS Story 4 readiness refresh | Story file still marks the story Blocked pending ADR-012 verification; current project notes say the evidence may be available but the story needs refresh | Included in S4-01 |
| Card Pool Story 4 and 5 | Planned in Sprint 3 as backlog/pull-forward but not active at this planning time | Included in S4-02 and S4-03 |

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Sprint 3 remains open at Sprint 4 kickoff | HIGH | HIGH | Keep Sprint 3 tracker authoritative until closeout; displace Sprint 4 Should/Nice items before adding capacity pressure |
| S4-01 starts from a story file still marked Blocked | MEDIUM | HIGH | Run `/story-readiness` on GSS Story 4 first and update the story with ADR-012 evidence before implementation |
| Lightyear dispatch and connection-event APIs drift from story assumptions | MEDIUM | HIGH | Use `liv-bevy-lightyear`; verify against existing `tests/evidence/lightyear-026-verification.md` before code work |
| Card Pool manual refresh and Card Acquisition refresh overlap semantically | MEDIUM | MEDIUM | Keep S4-03 scoped to Core pool/manual refresh behavior; Card Acquisition owns M2 display and purchase workflow later |
| No Sprint 4 QA plan exists | HIGH | HIGH | Run `/qa-plan sprint-4` before the first Sprint 4 implementation session |
| Must Have scope exactly fills available capacity | MEDIUM | MEDIUM | Treat S4-08 through S4-14 as pull-forward only; do not pull them if any Must Have slips |

---

## Dependencies on External Factors

- Developer PowerShell for VS 2026 or CI remains the authoritative Cargo verification path for local builds that need MSVC `link.exe`.
- GitHub Actions must stay green after each story merge.
- Lightyear 0.26 API evidence in `tests/evidence/lightyear-026-verification.md` must be treated as the networking source of truth.
- Sprint 3 closeout must reconcile `production/sprint-status.yaml` before Sprint 4 becomes the active tracker.

---

## QA Plan

> WARNING: **No QA Plan**: This sprint was planned without a Sprint 4 QA plan. Run `/qa-plan sprint-4`
> before the first story is implemented. The Production to Polish gate requires a QA
> sign-off report, which requires a QA plan.

---

## Definition of Done for this Sprint

- [ ] All Must Have tasks completed
- [ ] All tasks pass acceptance criteria
- [ ] QA plan exists (`production/qa/qa-plan-sprint-4.md` or dated equivalent)
- [ ] All Logic/Integration stories have passing unit/integration tests
- [ ] Smoke check passed (`/smoke-check sprint`)
- [ ] QA sign-off report: APPROVED or APPROVED WITH CONDITIONS (`/team-qa sprint`)
- [ ] No S1, S2, S3, or S4 bugs in delivered features
- [ ] Design documents updated for any deviations
- [ ] Code reviewed and merged

---

**Scope check:** Sprint 4 pulls selected M1/M2 scaffold work after Core closure. If Should Have or Nice to Have items begin before all Core Must Have work is complete, run `/scope-check` on the affected epics first.
