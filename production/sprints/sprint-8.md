# Sprint 8 -- 2026-05-08 to 2026-05-21

## Sprint Goal

Make the internal 1v1 friend-game loop feel complete and robust beyond the
proven next-loop DRAFT_SHOP endpoint by polishing auction settlement, extending
the result-endpoint evidence, and hardening the active draft/shop, auction,
placement, and resolution loop without claiming public release readiness.

## Planning Notes

- Current stage is `Polish`.
- Sprint 7 is closed with conditions.
- Sprint 7 achieved friend-game playable evidence through:
  lobby -> draft/shop -> auction -> non-empty placement ->
  resolution(UnitPlaced) -> next-loop DRAFT_SHOP.
- The honest Sprint 7 endpoint remains next-loop DRAFT_SHOP after post-auction
  placement/resolution.
- Game-over is not claimed.
- Full game completion is not claimed.
- QA-COND-0005 remains friend-game-only accepted risk. This is not verified
  Standard-tier accessibility completion and must not be represented as public,
  external, commercial, or broader release readiness.
- QA-COND-0006 remains accepted-risk/deferred. This is not playtest evidence,
  fun-hypothesis validation, or a playtest report.
- PR-SPRINT skipped -- Lean mode. `production/review-mode.txt` is not present,
  so the sprint-plan workflow defaults to `lean`.
- The core GAME_OVER pipeline has completed implementation stories, but Sprint
  7 did not reach game-over through friend-game evidence. Sprint 8 must either
  extend the friend-game endpoint toward a result screen/game-over path or
  record an explicit accepted nearest-endpoint improvement with no game-over
  claim.
- Sprint 8 planning is docs/status only. No `/dev-story`, `/story-done`,
  `/smoke-check`, `/team-qa`, or `/gate-check` was run by this planning prompt.

## Capacity

- Total workdays: 10
- Buffer (20%): 2 days reserved for integration surprises, evidence capture
  friction, and endpoint-risk follow-up
- Available: **8 effective planned days**
- Planned Must Have scope: **6.0 estimated days**
- Should Have scope is conditional and must not displace the active loop
  robustness goal.

---

## Tasks

### Must Have (Critical Path)

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S8-DOCS-001 | Sprint 8 story docs and readiness package | orchestrator + producer | 0.50 | This sprint plan | New Sprint 8 planned story docs exist or existing story docs are confirmed current before implementation starts; every new story preserves the Sprint 7 no-claim language; missing-story blockers in `production/sprint-status.yaml` are cleared only by docs/readiness work, not by implementation. |
| SAU-007 | [Auction Settlement and Shop Transition](../epics/shop-auction-ui/story-007-auction-settlement-and-shop-transition.md) | UI/client programmer | 1.00 | SAU-004, SAU-005, SAU-006 complete; Sprint 7 friend-game path reaches auction settlement | Local winner, opponent winner, and no-bid settlement states work; in-flight bid state and late accepted/rejected effects are cleared; auction panel dismisses and shop panel expands with the UX-specified transition; DRAFT_SHOP timer starts after shop expansion; PLACEMENT phase interrupts settlement immediately. |
| PLAYABLE-004 | Friend-Game Result Endpoint Expansion | client/server gameplay programmer + QA tester | 2.00 | PLAYABLE-003 complete; RSM GAME_OVER, Combat Objective Damage, GSS Game-Over Teardown, HUD Game-Over Freeze complete | Two real clients extend the Sprint 7 route beyond the proven endpoint toward game-over/result coverage; if GAME_OVER is reached, evidence records `S2CGameOver`, HUD frozen/result state, and teardown behavior; if GAME_OVER is not reachable within scoped capacity, the story records an explicit accepted nearest-endpoint improvement and defect classification without claiming game-over or full game completion. |
| LOOP-001 | DRAFT_SHOP / Auction / Placement / Resolution Loop Polish | client/server gameplay programmer + UI programmer | 1.50 | SAU-007 underway or complete; PLAYABLE-003 evidence route | The active friend-game loop can repeat through DRAFT_SHOP, auction, post-auction DRAFT_SHOP, non-empty placement, resolution replay, and next-loop DRAFT_SHOP without stale panels, stale timers, duplicate ready state, stale auction feedback, missing `UnitPlaced`, or client-side optimistic authority. |
| S8-QA-001 | Friend-Game Manual Smoke Expansion Package | QA tester + orchestrator | 1.00 | SAU-007 and LOOP-001 stable enough to exercise; PLAYABLE-004 endpoint decision | Manual friend-game evidence covers at least two consecutive loop passes after Sprint 7's endpoint, settlement-to-shop behavior, placement/resolution replay, endpoint decision, and known defects; evidence is labeled internal friend-game only and does not claim public release readiness, broad accessibility completion, playtest validation, full playable-client manual QA, game-over coverage unless actually reached, or full game completion. |

### Should Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| CONTENT-001 | Runtime Card Variety Floor | content designer + gameplay programmer | 1.00 | Must Have loop path stable; no unresolved schema blocker | `assets/data/cards.json` reaches a small internal friend-game variety floor with enough runtime-valid cards to reduce repeated-shop/auction sameness; includes sufficient neutral Rare/Legendary auction-eligible cards for repeated auction evidence; avoids unimplemented mechanics unless guarded by existing keyword/effect behavior; catalog validation and relevant pool/shop tests pass. |
| ECO-004 | [Kill and Objective Awards](../epics/economy-system/story-004-kill-and-objective-awards.md) reward-loop polish | gameplay programmer | 1.00 | PLAYABLE-004 or S8-QA-001 shows a concrete reward-loop gameplay issue | Only pulled if friend-game evidence shows kill/objective/fake reward visibility or timing affects the loop; preserves current event contracts; no duplicate gold awards; all rewards land before interest snapshot; no broad economy rebalance. |
| SAU-008 | [Reconnect Snapshot and Late Message Recovery](../epics/shop-auction-ui/story-008-reconnect-snapshot-and-late-message-recovery.md) | UI/client programmer | 1.25 | SAU-007 complete; reconnect or late-message instability affects the active loop | Only pulled if the active loop shows reconnect, snapshot, or late-message instability; snapshot rebuild restores the correct panel; late accepted/rejected and stale purchase/refresh confirmations do not resurrect inactive panels; no duplicate Lightyear receiver drains are introduced. |

### Nice To Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S8-N1 | Active Loop Surface Polish Notes | UI programmer + orchestrator | 0.50 | Must Have evidence identifies rough presentation edges | Small HUD, hand, shop, auction, or board readability notes are captured and prioritized for later stories; fixes only proceed if they directly improve the active friend-game loop and do not expand into broad Standard-tier accessibility completion. |
| S8-N2 | Sprint 8 Evidence Index Cleanup | orchestrator | 0.25 | S8-QA-001 evidence captured | Sprint 8 evidence paths are indexed from one concise document; the index preserves the exact endpoint reached and all non-claims. |

---

## Required Sprint 8 Story Docs

`/sprint-plan` does not scaffold new story files. Create or refresh these in a
separate docs-only prompt before `/dev-story` begins:

| Planned ID | Required story file |
|------------|---------------------|
| S8-DOCS-001 | `production/epics/playable-client/story-004-friend-game-result-endpoint-expansion.md` and `production/epics/playable-client/story-005-draft-shop-auction-placement-resolution-loop-polish.md` |
| PLAYABLE-004 | `production/epics/playable-client/story-004-friend-game-result-endpoint-expansion.md` |
| LOOP-001 | `production/epics/playable-client/story-005-draft-shop-auction-placement-resolution-loop-polish.md` |
| S8-QA-001 | `production/qa/evidence/sprint-8-friend-game-loop-evidence.md` |

Existing story docs that are already present and Ready:

- `production/epics/shop-auction-ui/story-007-auction-settlement-and-shop-transition.md`
- `production/epics/shop-auction-ui/story-008-reconnect-snapshot-and-late-message-recovery.md`
- `production/epics/economy-system/story-004-kill-and-objective-awards.md`

Until the new PLAYABLE story files exist and pass story readiness, the matching
rows in `production/sprint-status.yaml` remain blocked by missing story docs.

## Carryover from Previous Sprint

| Task | Reason | New Estimate |
|------|--------|--------------|
| SAU-007 | Sprint 7 did not pull the conditional Should Have. It is now the most direct polish story for the proven auction path. | 1.00d |
| ECO-004 | Sprint 7 did not pull this conditional reward-loop polish. It remains useful only if new friend-game evidence shows a gameplay issue. | 1.00d |

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| GAME_OVER is implemented but not friend-game reachable yet | HIGH | HIGH | Put PLAYABLE-004 before broad polish; accept a documented nearest-endpoint improvement only with explicit no-game-over wording. |
| Auction settlement presentation can mask authoritative state ordering | MEDIUM | HIGH | Prioritize SAU-007; keep settlement terminal, late-message-safe, and interruptible by PLACEMENT. |
| Loop polish can sprawl across many UI surfaces | MEDIUM | MEDIUM | LOOP-001 must target only stale state, timers, replay, and active-loop coherence observed in friend-game evidence. |
| Card catalog has only a fixture-scale variety floor | MEDIUM | MEDIUM | Pull CONTENT-001 only after Must Have stability; keep it to runtime-valid internal friend-game variety, not full card production. |
| Reward-loop polish could become broad economy tuning | MEDIUM | MEDIUM | Pull ECO-004 only from concrete evidence of a loop issue; no broad balancing or economy redesign. |
| Reconnect scope can expand beyond active loop needs | MEDIUM | MEDIUM | Pull SAU-008 only if reconnect/late-message instability affects the active loop. |
| QA-COND-0005 could be misreported as verified accessibility completion | MEDIUM | HIGH | Preserve accepted-risk language in every Sprint 8 plan, story, evidence, smoke, and sign-off artifact. |
| QA-COND-0006 could be misreported as playtest validation | MEDIUM | HIGH | Keep friend-game evidence separate from playtest/fun-hypothesis evidence; do not close QA-COND-0006. |
| Sprint 8 has no QA plan yet | HIGH | MEDIUM | Run `/qa-plan sprint-8` before implementation starts; keep the warning below until a plan exists. |

## Dependencies on External Factors

- A local server and two real primary clients can run against the same build.
- The existing Sprint 7 controlled real-Lightyear evidence path remains
  repeatable enough to extend.
- Existing RSM, Combat Resolution, Objective System, Game Session System, HUD,
  Shop/Auction UI, Hand UI, Board Rendering, Economy, and Presentation Layer
  behavior remains stable enough to exercise the active loop.
- Sprint 8 QA plan must be created before implementation starts.

## QA Plan

No Sprint 8 QA plan was found at planning time.

> Warning: This sprint has no QA plan yet. Run `/qa-plan sprint-8` before
> implementation begins. Until that exists, test requirements for Sprint 8 are
> undefined and the sprint cannot close cleanly with QA sign-off.

## Out of Scope

- Public, external, commercial, store, deployment, or release-candidate
  readiness.
- Broad Standard-tier accessibility completion.
- Claiming QA-COND-0005 as verified accessibility completion.
- Claiming QA-COND-0006 as playtest evidence, fun-hypothesis validation, or a
  playtest report.
- Full playable-client manual QA beyond the scoped internal friend-game
  evidence required by S8-QA-001.
- Full game completion, even if GAME_OVER/result endpoint coverage is actually
  reached and evidenced by PLAYABLE-004.
- New game modes, 2v2/3v3 scope, broad class/keyword/prism polish, store
  metadata, deployment readiness, or public QA.
- Implementing code as part of this sprint-planning prompt.

## Definition of Done for this Sprint

- [ ] Sprint 8 QA plan exists before implementation begins.
- [ ] New Sprint 8 story docs exist and pass readiness before implementation
      starts.
- [ ] All Must Have tasks completed.
- [ ] SAU-007 settlement-to-shop transition is implemented and evidenced.
- [ ] PLAYABLE-004 records either actual GAME_OVER/result evidence or an
      explicitly accepted nearest-endpoint improvement without claiming
      game-over.
- [ ] LOOP-001 verifies the active DRAFT_SHOP / auction / placement /
      resolution loop can repeat without stale state or authority drift.
- [ ] S8-QA-001 friend-game manual evidence exists and records the exact endpoint
      reached.
- [ ] All Logic/Integration stories have passing unit/integration tests.
- [ ] Friend-game smoke evidence exists before sprint closure.
- [ ] QA sign-off report is complete before any future gate or release claim.
- [ ] No S1 or S2 bugs remain in the scoped friend-game path unless explicitly
      accepted as Sprint 8 conditions.
- [ ] QA-COND-0005 remains labeled accepted risk unless separately verified
      under a future accessibility scope.
- [ ] QA-COND-0006 remains labeled accepted-risk/deferred unless actual
      playtest evidence is produced later.
- [ ] Public release readiness, broad accessibility completion, playtest
      validation, full playable-client manual QA, and full game completion are
      not claimed by Sprint 8 artifacts.

## Next Recommended Step

Run `/qa-plan sprint-8` before implementation begins. Then create/refresh the
new Sprint 8 PLAYABLE story docs and run `/story-readiness` on each Must Have
story before any `/dev-story`.

---

**Scope check:** Sprint 8 is a Polish friend-game robustness sprint. If any work
beyond the listed Must Have and conditional Should Have rows is proposed, run
`/scope-check` before implementation begins and confirm it does not displace the
internal 1v1 active-loop goal.
