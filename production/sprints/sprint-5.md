# Sprint 5 -- 2026-05-04 to 2026-05-15

## Sprint Goal

Close integrated work, reconcile in-flight pull-forward, and complete the shortest server-critical path from placement through combat resolution to objective damage/game-over for a playable M1/M2 loop.

## Planning Notes

- Sprint 3 is treated as complete: every story in `production/sprint-status.yaml` is `done`.
- Sprint 4 is treated as complete: every story linked from `production/sprints/sprint-4.md` is marked `Complete`.
- Sprint 4's markdown dates are stale relative to repo state; Sprint 4 was planned for `2026-06-11` to `2026-06-24`, but repo evidence shows the listed work complete by `2026-05-03`.
- Sprint 5 planning starts from the current repo state on `2026-05-03`.
- PR-SPRINT skipped -- Lean mode. `production/review-mode.txt` is not present, so the sprint-plan workflow defaults to `lean`.
- Do not start new implementation before the closure queue and pull-forward revalidation are reconciled.

## Capacity

- Total days: 10
- Buffer (20%): 2 days reserved for unplanned work / integration surprises
- Available: **8 effective days**
- Planned Must Have scope: **8.0 estimated days**
- Should Have and Nice to Have scope is pull-forward only after Must Have work is stable

---

## Tasks

### Must Have (Critical Path)

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S5-01 | [Hand UI Story 9: PLACEMENT Timer](../epics/hand-ui/story-009-placement-timer.md) story-done closure | orchestrator | 0.25 | Integrated work ready for review | Story file is marked `Complete`; evidence, deviations, and status notes are reconciled without touching session-state files |
| S5-02 | [Combat Story 3: Sub-step 1 - Placement Commit + APPEARANCE](../epics/combat-resolution/story-003-substep1-placement-appearance.md) story-done closure | orchestrator | 0.25 | Integrated work ready for review | Story file is marked `Complete`; SS1 placement/APPEARANCE criteria are verified and documented |
| S5-03 | [Objective Story 5: Destruction Consequence Path](../epics/objective-system/story-005-destruction-consequence-path.md) story-done closure | orchestrator | 0.25 | Integrated work ready for review | Story file is marked `Complete`; consequence-path evidence is verified and documented |
| S5-04 | [Card Data Pool Story 6: Network Dispatch Wiring](../epics/card-data-pool/story-006-network-dispatch-wiring.md) pull-forward revalidation/integration | worker + orchestrator | 0.50 | CDP-004, CDP-005 Complete; Lightyear reliable unicast API evidence | `S2CShopSlots` and `S2CDraftOffering` dispatch correctly to target players on ReliableChannel; branch/worktree state reconciled |
| S5-05 | [Auction Story 7: Plugin Registration & System Scheduling](../epics/auction-system/story-007-auction-plugin-scheduling.md) pull-forward revalidation/integration | worker + orchestrator | 0.50 | AUC-003, AUC-006 Complete | `AuctionPlugin` registers the complete auction tick chain; scheduling tests pass; branch/worktree state reconciled |
| S5-06 | [Combat Story 4: Movement + Collision](../epics/combat-resolution/story-004-movement-collision.md) | worker | 1.00 | COMBAT-003 Complete | SS2/SS5 movement, STUN suppression, WALL halt, path-crossing halt, CHARGE X, and RANGE-vs-WALL behavior pass tests |
| S5-07 | [Combat Story 5: FIRST STRIKE Attacks](../epics/combat-resolution/story-005-substep3-first-strike.md) | worker | 1.00 | S5-06 | SS3 FIRST STRIKE damage, simultaneous HP snapshots, RANGE+FIRST STRIKE dual attack, FINAL BLOW timing, and lane-order credit pass tests |
| S5-08 | [Combat Story 6: Dead Removal + DEATH Chains + Kill Gold](../epics/combat-resolution/story-006-substep4-dead-removal.md) | worker | 1.00 | S5-07 | SS4 removes dead units, processes DEATH chains, emits kill gold, and preserves FINAL BLOW sub-step ordering |
| S5-09 | [Combat Story 7: Standard Combat + SHIELD + COUNTERATTACK](../epics/combat-resolution/story-007-substep6-combat-shield-counterattack.md) | worker | 1.00 | S5-08 | SS6 melee combat, SHIELD consumption/persistence, and COUNTERATTACK eligibility pass tests |
| S5-10 | [Combat Story 8: RANGE Targeting](../epics/combat-resolution/story-008-range-targeting.md) | worker | 1.00 | S5-09 | Forward-only RANGE targeting, RANGE+FIRST STRIKE second target acquisition, and RANGE-vs-WALL behavior pass tests |
| S5-11 | [Combat Story 9: Objective Damage + GAME_OVER](../epics/combat-resolution/story-009-objective-damage-gameover.md) | worker | 1.25 | S5-09, S5-10, OBJECTIVE-005 | Cell-8 objective damage, objective destruction rewards, loser detection, and mutual-destruction draw behavior pass tests |

### Should Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S5-12 | [Hand UI Story 11: Reserve Mana Split Strip](../epics/hand-ui/story-011-reserve-mana-strip.md) pull-forward revalidation/integration | worker + orchestrator | 0.50 | HAND-UI-005 | Reserve controls update staged `reserve_amount` values and branch/worktree state is reconciled |
| S5-13 | [Board Story 10: Displacement Keywords](../epics/board-lane-system/story-010-displacement-keywords.md) pull-forward revalidation/integration | worker + orchestrator | 0.50 | BOARD-002 | REPEL, ATTRACT, TELEPORT, CHANGE LANE, and spawn-range expansion pass tests without conflicting with COMBAT-004 |
| S5-14 | [Auction Story 8: Pool Integration](../epics/auction-system/story-008-pool-integration.md) | unassigned | 1.00 | S5-05, Card Data Pool draw/distribute APIs | Auction card draw consumes pool copies at draw time, empty-pool settlement fires immediately, Legendary stratification is enforced |
| S5-15 | [Objective Story 6: D4 Fake Reward Draw](../epics/objective-system/story-006-d4-fake-reward-draw.md) | unassigned | 1.00 | S5-03 | Fake objective rewards emit exactly one reward path, hand-full conversion and pool-empty no-op behavior pass tests |
| S5-16 | [Objective Story 7: Resolution-End Sync](../epics/objective-system/story-007-resolution-phase-subscription.md) | unassigned | 1.00 | S5-15 | ObjectiveDestroyed broadcasts are batched at RESOLUTION end in lane order and do not duplicate Sang Meprise visibility |
| S5-17 | [Hand UI Story 10: Submit Pre-Validation](../epics/hand-ui/story-010-submit-prevalidation.md) | unassigned | 0.75 | HAND-UI-005; S5-12 preferred | Reserve/mana overdraw blocks submit without sending `C2SSubmitPlacement`; correction clears the error and submits |
| S5-18 | [Server RNG Story 3: Determinism Proof & Session Reset](../epics/server-rng/story-003-determinism-session-reset.md) | unassigned | 0.75 | Server RNG Story 2 Complete | Deterministic scripted sequence and session reset evidence pass |

### Nice to Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S5-19 | [Combat Story 10: Persistent Keyword States](../epics/combat-resolution/story-010-persistent-keyword-states.md) | unassigned | 1.00 | S5-11 | INJURED, LEADER, and OUTNUMBERED persistent state behavior passes tests |
| S5-20 | [Combat Story 11: ResolutionEvent Log Completeness](../epics/combat-resolution/story-011-resolution-event-log.md) | unassigned | 1.00 | S5-19 and all prior Combat stories | `S2CPlacementReveal` and `S2CResolutionEvent` contain complete ordered resolution data |
| S5-21 | Create missing Board Rendering and Shop/Auction UI production epics/stories | unassigned | 1.00 | Approved GDDs and architecture references | Production epics exist for M2 visual playable path and can be planned into a later sprint |
| S5-22 | Clean up duplicate/stale Card Data Pool story files marked Ready | orchestrator | 0.50 | CDP-006 reconciliation | Duplicate older story files are classified or retired without losing current evidence |

---

## Carryover from Previous Sprint

| Task | Reason | New Estimate |
|------|--------|--------------|
| HAND-UI-009 | Already integrated but not yet story-done; must be closed before normal Sprint 5 capacity is counted | 0.25d |
| COMBAT-003 | Already integrated but not yet story-done; blocks COMBAT-004 and later combat path | 0.25d |
| OBJECTIVE-005 | Already integrated but not yet story-done; blocks Objective 006/007 and Combat 009 confidence | 0.25d |

---

## Out-of-Plan Pull-Forward Revalidation

The following items were launched before Sprint 5 was formally planned. They must be revalidated and reconciled before being counted as normal Sprint 5 scope:

- AUC-007 Plugin Registration & Scheduling
- HAND-UI-011 Reserve Mana Split Strip
- BOARD-010 Displacement Keywords
- CDP-006 Network Dispatch Wiring

Revalidation means confirming the active branch/worktree, current diff against `main`, story readiness, acceptance criteria coverage, test evidence, and integration order. If any item has no discoverable branch/worktree or has drifted from current `main`, treat it as blocked until the owner provides a handoff.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Combat path is serial and one slip blocks the playable loop | HIGH | HIGH | Keep COMBAT-004 through COMBAT-009 as the Must Have spine; defer all Should/Nice work first |
| Pull-forward branches may have launched before the current Sprint 5 baseline | HIGH | HIGH | Run explicit revalidation before counting AUC-007, HAND-UI-011, BOARD-010, or CDP-006 as normal scope |
| Objective and Combat ownership overlap at Cell 8 damage/game-over | MEDIUM | HIGH | Close OBJECTIVE-005 first; keep Combat 009 limited to calling the Objective API and asserting game-over outcomes |
| BOARD-010 displacement work may collide with COMBAT-004 movement/collision seams | MEDIUM | MEDIUM | Serialize integration review for shared movement helpers and require targeted regression tests |
| No Sprint 5 QA plan exists yet | HIGH | HIGH | Run `/qa-plan sprint-5` before new implementation begins |
| M2 visual playable path lacks Board Rendering and Shop/Auction UI production epics | MEDIUM | HIGH | Treat S5-21 as Nice to Have planning cleanup; do not block server-critical M1/M2 loop work on it |
| `production/risk-register/` is missing | MEDIUM | MEDIUM | Track Sprint 5 risks in this sprint plan until a risk register is created |

---

## Dependencies on External Factors

- Developer PowerShell for VS 2026 or CI remains the authoritative Cargo verification path for local builds that need MSVC `link.exe`.
- GitHub Actions must stay green after each story merge.
- Lightyear 0.26 reliable send/unicast API evidence remains the networking source of truth.
- In-flight worker handoffs must identify branch, commit, changed files, test evidence, and merge status before story-done.
- Sprint 5 cannot pass the Production to Polish gate without a Sprint 5 QA plan and QA sign-off.

---

## QA Plan

> WARNING: **No QA Plan**: Sprint 5 has no QA plan yet. Run `/qa-plan sprint-5`
> before new implementation begins. The Production to Polish gate requires a QA
> sign-off report, which requires a QA plan.

---

## Definition of Done for this Sprint

- [ ] All Must Have tasks completed
- [ ] All closure queue stories are story-done
- [ ] All pull-forward items are revalidated before being counted as Sprint 5 scope
- [ ] All tasks pass acceptance criteria
- [ ] QA plan exists (`production/qa/qa-plan-sprint-5.md` or dated equivalent)
- [ ] All Logic/Integration stories have passing unit/integration tests
- [ ] Smoke check passed (`/smoke-check sprint`)
- [ ] QA sign-off report: APPROVED or APPROVED WITH CONDITIONS (`/team-qa sprint`)
- [ ] No S1 or S2 bugs in delivered features
- [ ] Design documents updated for any deviations
- [ ] Code reviewed and merged

---

**Scope check:** Sprint 5 pulls selected M2/M3-adjacent work only after the Must Have playable-loop spine is stable. If Should Have or Nice to Have items begin before COMBAT-009 is complete, run `/scope-check` on the affected epics first.
