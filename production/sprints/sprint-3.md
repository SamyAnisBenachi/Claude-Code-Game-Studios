# Sprint 3 -- 2026-05-28 to 2026-06-10

## Sprint Goal

Complete the remaining Core Sprint B backbone while intentionally pulling forward selected M2/M3 scaffolds: lobby/class flow, RSM timers/input + win-condition path, the last networking proof, auction entry infrastructure, class lifecycle state, and keyword module scaffolding.

## Planning Notes

- Target dates remain 2026-05-28 to 2026-06-10. These are planning targets, not blockers for early implementation.
- Scope was intentionally expanded beyond the original Sprint 3 plan on 2026-05-01.
- Machine-readable story status lives in `production/sprint-status.yaml`.
- Status snapshot below reflects `production/sprint-status.yaml` as of 2026-05-01.

## Capacity

- Total days: 10
- Buffer (20%): 2 days reserved for unplanned work / verification surprises
- Available: **8 effective days**
- Current tracked scope: **14 stories / 16.0 estimated days**
- Current Must Have scope: **10 stories / 11.5 estimated days**

The expanded scope intentionally exceeds the original active sprint capacity. Treat Should Have and Nice to Have items as pull-forward or parallel-lane work unless the active owners confirm capacity.

---

## Producer Feasibility Gate

PR-SPRINT skipped -- Lean mode.

Original Must Have scope totaled 7.5 effective days. The current Sprint 3 tracker intentionally adds M2/M3 scaffold work (`CS-001`, `AUC-001`, `AUC-002`, `KW-001`), raising Must Have scope to 11.5 effective days. This is accepted as an intentional scope expansion, not an accidental overcommit.

---

## Sprint Status Snapshot

Progress: **4/14 stories complete (29%)**

| Status | Count | Stories |
|--------|-------|---------|
| Done | 4 | S3-01, CS-001, AUC-001, AUC-002 |
| In Progress | 5 | S3-02, S3-04, S3-06, S3-08, KW-001 |
| Ready for Dev | 2 | S3-03, S3-05 |
| Backlog | 3 | S3-07, S3-09, S3-10 |

---

## Tasks

### Must Have (Critical Path + Intentional Expansion)

| ID | Status | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|--------|------|-------------|-----------|--------------|---------------------|
| S3-01 | Done 2026-05-01 | [GSS Story 1: Lobby Scaffold](../epics/game-session-system/story-001-lobby-scaffold.md) | claude-s3-01-gss-lobby | 1.0 | Foundation shared/protocol types | `server/src/core/session/` scaffold exists; `SessionReady`, `SessionConfig`, lobby resources compile; session scaffold tests pass |
| S3-02 | In Progress | [GSS Story 2: Room Create and Join](../epics/game-session-system/story-002-room-create-join.md) | codex-s3-02-room-create-join | 1.5 | S3-01, S2-09 | Create/join handlers enforce one active session per player; idempotent create works; slot state broadcasts full slot vector |
| S3-03 | Ready for Dev | [GSS Story 3: Class Selection and Reveal](../epics/game-session-system/story-003-class-selection-reveal.md) | unassigned | 1.0 | S3-02 | Preview never broadcasts; confirm locks atomically; `S2CClassesRevealed` broadcasts once after all occupied slots lock |
| S3-04 | In Progress | [RSM Story 3: Timers and Input Reader](../epics/round-state-machine/story-003-timers-and-input-reader.md) | claude-s3-04-rsm-timers | 1.5 | S2-07 | RSM timers use `GameConfig`; stale inbound messages are discarded; ready/submission exits and timer expiries transition correctly |
| S3-05 | Ready for Dev | [RSM Story 4: Win Condition and Game Over](../epics/round-state-machine/story-004-win-condition-and-game-over.md) | unassigned | 1.5 | S3-04 | Objective-loss, no-loss, and draw paths emit correct `GameOverEmitted`; `BroadcastPhaseChanged` remains last; no `server::feature` imports |
| S3-06 | In Progress | [Carryover: E2E WebSocket Round-Trip](../epics/lightyear-protocol-verification/story-004-e2e-websocket-roundtrip.md) | codex-s3-06-websocket | 1.0 | S2-09 | Heartbeat round-trip test passes; reliable-channel proof exists; WASM bundle size evidence documented |
| CS-001 | Done 2026-05-01 | [Class Lifecycle / PlayerSessions Scaffold](../epics/class-system/story-001-class-lifecycle.md) | claude-cs-001-class-lifecycle | 1.0 | Workspace/shared types, RSM scaffold | Player class lifecycle is server-authoritative; classes lock correctly; snapshots include `class_id` |
| AUC-001 | Done 2026-05-01 | [AuctionState Types & Snapshot Scaffold](../epics/auction-system/story-001-auction-state-scaffold.md) | claude-auc-001-auction-state | 1.0 | Shared protocol types | Auction state and snapshot scaffold compile; AU10-a through AU10-e pass |
| AUC-002 | Done 2026-05-01 | [Auction Phase Entry](../epics/auction-system/story-002-auction-phase-entry.md) | codex-auc-002-phase-entry | 1.0 | AUC-001, RSM event bus, card pool | `AuctionPhaseEntered` transitions IDLE to LIVE_BIDDING; auction card queue behavior passes AU1-a/AU1-b/AU23 |
| KW-001 | In Progress | [Keyword System Module Scaffold](../epics/keyword-system/story-001-module-scaffold.md) | codex-kw-001-keyword-scaffold | 1.0 | ADR-018, ADR-022, ADR-006 amendment | Keyword module tree, component/resource/event scaffolds, protocol keyword types, and plugin smoke tests exist |

### Should Have

| ID | Status | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|--------|------|-------------|-----------|--------------|---------------------|
| S3-07 | Backlog | [Card Pool Story 4: ShopRefreshNeeded Subscriber + SessionReady Init](../epics/card-data-pool/story-004-shop-refresh-subscriber-session-ready.md) | unassigned | 1.5 | S2-04, S3-01, S2-07 | `SessionReady` initializes per-player pools; `ShopRefreshNeeded` fills independent shops/offering; `ManualRefreshCount` resets |
| S3-08 | In Progress | [Economy Story 3: Interest Snapshot & Resolution End](../epics/economy-system/story-003-interest-snapshot-resolution.md) | claude-s3-08-economy-interest | 1.0 | S2-08, S2-07 | Resolution snapshot captures gold; current mana discarded; stale snapshot overwritten; interest cap path tested |

### Nice to Have

| ID | Status | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|--------|------|-------------|-----------|--------------|---------------------|
| S3-09 | Backlog / Blocked | [GSS Story 4: F4 Predicate and SessionReady Trigger](../epics/game-session-system/story-004-f4-session-ready.md) | unassigned | 1.5 | S3-01, S3-02, S3-03, ADR-012 evidence refresh | Story readiness revalidated against ADR-012; `SessionReady` fires once; lobby-to-draft same-tick integration passes |
| S3-10 | Backlog | [Card Pool Story 5: Manual Refresh + Cost Escalation](../epics/card-data-pool/story-005-manual-refresh-cost-escalation.md) | unassigned | 1.0 | S3-07, Economy API | Manual refresh costs escalate 1g/2g/3g; insufficient gold and wrong phase do not mutate state |

---

## Carryover and Scope Expansion

| Task | Reason | New Estimate |
|------|--------|--------------|
| S2-10 E2E WebSocket Round-Trip -> S3-06 | Still `in-progress` in `production/sprint-status.yaml` at Sprint 3 planning time | 1.0d |
| CS-001 Class Lifecycle / PlayerSessions Scaffold | Intentional M3 scaffold pull-forward to support lobby/class flow and session snapshots | 1.0d |
| AUC-001 AuctionState Types & Snapshot Scaffold | Intentional M2 auction infrastructure pull-forward | 1.0d |
| AUC-002 Auction Phase Entry | Intentional M2 auction infrastructure pull-forward after AUC-001 completion | 1.0d |
| KW-001 Keyword System Module Scaffold | Intentional M3 scaffold pull-forward after ADR-018/ADR-022 acceptance | 1.0d |

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Expanded Sprint 3 scope exceeds original capacity | HIGH | HIGH | Treat the expansion as intentional; use `production/sprint-status.yaml` as the source of truth and avoid pulling additional backlog unless owners explicitly free capacity |
| Five stories are currently in progress at once | HIGH | MEDIUM | Keep ownership explicit; route merge/CI issues back to the owning story agent before starting new stories |
| GSS Story 004 file still says Blocked even though ADR-012 evidence later resolved the core observer-ordering concern | MEDIUM | HIGH | Keep S3-09 Nice to Have until `/story-readiness` refreshes the story file against `tests/evidence/lightyear-026-verification.md` |
| Lightyear 0.26 send/receive APIs differ from older assumptions | MEDIUM | HIGH | Use S1/S2 verification evidence and `liv-bevy-lightyear` on all networking work |
| M2/M3 scaffold stories can outpace design/architecture freshness | MEDIUM | MEDIUM | Re-read current ADRs, control manifest, and story files at story start; document advisory drift in `/story-done` |
| GSS room/session resources overlap with existing `server/src/core/session/state.rs` and class lifecycle additions | MEDIUM | MEDIUM | Extend current session resources carefully; do not replace active user/prior-agent work |

---

## Dependencies on External Factors

- Developer PowerShell for VS 2026 or CI remains the authoritative Cargo verification path for local builds that need MSVC `link.exe`.
- GitHub Actions must stay green after each story merge.
- Lightyear 0.26 verified API evidence in `tests/evidence/lightyear-026-verification.md` must be treated as the networking source of truth.
- QA plan coverage must stay aligned with the intentionally expanded scope.

---

## QA Plan

QA Plan: [`production/qa/qa-plan-sprint-3-2026-05-01.md`](../qa/qa-plan-sprint-3-2026-05-01.md)

---

## Definition of Done for this Sprint

- [ ] All Must Have tasks completed
- [ ] All tasks pass acceptance criteria
- [x] QA plan exists (`production/qa/qa-plan-sprint-3-2026-05-01.md`)
- [ ] All Logic/Integration stories have passing unit/integration tests
- [ ] Smoke check passed (`/smoke-check sprint`)
- [ ] QA sign-off report: APPROVED or APPROVED WITH CONDITIONS (`/team-qa sprint`)
- [ ] No S1, S2, or S3 bugs in delivered features
- [ ] Design documents updated for any deviations
- [ ] Code reviewed and merged

---

**Scope check:** Sprint 3 includes stories added beyond the original plan. Run `/scope-check` on the affected epics if the expanded scope starts displacing Core Sprint B completion work.
