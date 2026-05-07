# Sprint 7 -- 2026-05-07 to 2026-05-20

## Sprint Goal

Make the Polish build playable for an internal friend-game session through the
real primary client path, without claiming public release readiness, broad
accessibility completion, or full playtest validation.

## Planning Notes

- Current stage is `Polish`.
- Sprint 6 is complete and remains closed.
- Sprint 7 is the first Polish sprint plan.
- Scope target is friend-game playable quality, not public release readiness.
- PR-SPRINT skipped -- Lean mode. `production/review-mode.txt` is not present,
  so the sprint-plan workflow defaults to `lean`.
- Critical playable-path audit result: the primary client/WASM path is not
  actually playable end-to-end. No dedicated Sprint 7 scope audit or playable
  critical-path audit artifact was found in this checkout; local planning review
  found the current client starts network and presentation plugins but has no
  real lobby/menu flow that sends player intent and transitions into
  `ClientState::InSession`.
- `QA-COND-0001`, `QA-COND-0002`, `QA-COND-0003`, `QA-COND-0004`, and
  `QA-COND-0007` are Closed.
- `QA-COND-0005` remains accepted risk for friend-game scope only. This is not
  verified Standard-tier accessibility completion and must not dominate Sprint 7
  unless a gap directly blocks core gameplay readability or stability.
- `QA-COND-0006` remains accepted-risk/deferred. This is not playtest evidence,
  fun-hypothesis validation, or a public QA claim.
- Sprint 7 status close-out completed on 2026-05-07. Sprint 7 is closed with
  conditions after smoke `PASS WITH WARNINGS` and QA sign-off `APPROVED WITH
  CONDITIONS`.
- Close-out did not run `/gate-check`, Sprint 8 planning, new implementation,
  smoke, `/team-qa`, or `/story-done`.
- Full playable-client manual QA is not claimed by Sprint 6 smoke, QA sign-off,
  gate reports, or this Sprint 7 plan.
- Sprint 7 must use real client/server messages and real client state. Harness
  fixtures may support tests, but Must Have completion cannot depend on
  harness-injected game state.
- `/sprint-plan` does not create placeholder story files in this project. The
  required PLAYABLE story docs are listed below and must be created/readiness
  reviewed in a separate docs-only prompt before `/dev-story` begins.

## Capacity

- Total workdays: 10
- Buffer (20%): 2 days reserved for integration surprises, local environment
  friction, and evidence capture
- Available: **8 effective planned days**
- Planned Must Have scope: **6.0 estimated days**
- Should Have scope is conditional and must not displace the real playable
  client path.

---

## Tasks

### Must Have (Real Playable Path)

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| PLAYABLE-001 | Primary Client Bootstrap + Fresh Lobby Entry | client/network gameplay programmer | 2.00 | `client/src/main.rs`; `client/src/network/`; Game Session System; Lightyear protocol; Presentation plugin; `ClientState` | Real client plugins/window are sufficient for the primary client to run; fresh `C2SHello` maps a player; create/join/class confirm flow is available through real C2S/S2C messages; a minimal lobby/session entry UI exists for friend-game use; transition to `ClientState::InSession` happens only from server-confirmed session/phase/snapshot state |
| PLAYABLE-002 | Live Draft/Shop/Hand Bridge | client/network gameplay programmer + UI programmer | 2.00 | PLAYABLE-001; Card Data Pool network dispatch; Card Acquisition purchase flow; Economy view; Shop/Auction UI; Hand UI | `C2SSignalReady` reaches the server-side `DraftReadySignal` path; regular purchase confirmation reaches the client hand and economy views; DRAFT_INITIAL and DRAFT_SHOP purchase/ready UI runs through real WebSocket messages; no client-side optimistic authority is introduced |
| PLAYABLE-003 | Real End-to-End Loop Verification | QA tester + orchestrator + client/server programmer | 2.00 | PLAYABLE-001 and PLAYABLE-002 complete; real local server; two real clients | Two real clients run through create/join/class confirm, draft/shop, auction where available, placement, resolution, next loop, and game-over or documented nearest reachable friend-game endpoint; evidence records exact build/commit, commands, screenshots or captures, and defects; no harness-injected state is used; result is friend-game evidence only, not public QA or playtest validation |

### Should Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| SAU-007 | [Auction Settlement and Shop Transition](../epics/shop-auction-ui/story-007-auction-settlement-and-shop-transition.md) | UI/client programmer | 1.00 | PLAYABLE-001/002 stable enough to exercise live settlement; SAU-004, SAU-005, SAU-006 Complete | Settlement state, winner/no-bid presentation, late-message suppression, and auction-to-shop transition are implemented and evidenced without blocking the Must Have playable path |
| ECO-004 | [Kill and Objective Awards](../epics/economy-system/story-004-kill-and-objective-awards.md) readiness refresh / reward-loop polish | gameplay programmer | 0.75 | PLAYABLE-003 exposes reward-loop gap or capacity remains after Must Have | Story readiness is refreshed against current Combat/Objective event names and Bevy 0.18 `Message` usage; implementation only proceeds if it directly improves the friend-game reward loop and does not expand broad economy scope |

### Nice To Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S7-N1 | Friend-game evidence index cleanup | orchestrator | 0.25 | PLAYABLE-003 evidence captured | Sprint 7 evidence paths are easy to audit from the sprint plan, future QA plan, and manual friend-game report |

---

## Required PLAYABLE Story Docs

`/sprint-plan` does not scaffold incomplete placeholder story docs. Create these
in a separate docs-only prompt before implementation:

| Planned ID | Required story file |
|------------|---------------------|
| PLAYABLE-001 | `production/epics/playable-client/story-001-primary-client-bootstrap-fresh-lobby-entry.md` |
| PLAYABLE-002 | `production/epics/playable-client/story-002-live-draft-shop-hand-bridge.md` |
| PLAYABLE-003 | `production/epics/playable-client/story-003-real-end-to-end-loop-verification.md` |

Until those files exist and pass story readiness, the PLAYABLE rows in
`production/sprint-status.yaml` are tracked as blocked by missing story docs.

## Carryover from Previous Sprint

| Task | Reason | New Estimate |
|------|--------|--------------|
| SAU-007 | Sprint 6 Should Have remained backlog and is still relevant if the live auction path reaches settlement | 1.00d |
| ECO-004 | Existing Ready story may need a current readiness refresh before it can polish the live reward loop | 0.75d |

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Primary client has no real friend-game entry path | HIGH | HIGH | Put PLAYABLE-001 first; do not spend capacity on polish before a real lobby/session transition works |
| Client networking currently exposes sender stubs without user-driven flow | HIGH | HIGH | PLAYABLE-001 must replace stubs-only reachability with real intent sends and server-confirmed state transitions |
| Draft Ready UI may not reach server RSM readiness | HIGH | HIGH | PLAYABLE-002 explicitly wires `C2SSignalReady` to `DraftReadySignal` and verifies all-ready phase progression |
| Harness evidence could mask broken primary client behavior | HIGH | HIGH | PLAYABLE-003 requires two real clients and forbids harness-injected state for completion evidence |
| No Sprint 7 QA plan exists yet | HIGH | MEDIUM | Run `/qa-plan sprint-7` after PLAYABLE story docs exist and before implementation starts |
| Accessibility debt can expand beyond friend-game scope | MEDIUM | MEDIUM | Treat QA-COND-0005 as accepted risk; only fix accessibility issues that directly affect core gameplay readability/stability |
| QA-COND-0006 could be misreported as playtest validation | MEDIUM | HIGH | Keep PLAYABLE-003 evidence labeled friend-game manual evidence, not playtest/fun-hypothesis closure |

## Dependencies on External Factors

- A local server and two real primary clients can be run against the same build.
- Browser/WASM or native client target can launch with real window/rendering
  plugins as needed for friend-game use.
- Existing Game Session System, RSM, Card Data Pool, Card Acquisition, Economy,
  Shop/Auction UI, Hand UI, HUD, Board Rendering, and Presentation Layer
  surfaces remain stable enough to integrate through real messages.
- A Sprint 7 QA plan must be created before implementation starts.

## QA Plan

Sprint 7 QA planning exists at
[`production/qa/qa-plan-sprint-7-2026-05-06.md`](../qa/qa-plan-sprint-7-2026-05-06.md).
No `/smoke-check`, `/team-qa`, `/gate-check`, or Sprint 8 planning is claimed
by the S7-N1 evidence-index cleanup.

## Friend-Game Evidence Index

S7-N1 is complete. The concise evidence index is
[`production/qa/evidence/sprint-7-friend-game-evidence-index.md`](../qa/evidence/sprint-7-friend-game-evidence-index.md).

The verified endpoint is next-loop DRAFT_SHOP after post-auction
placement/resolution. Game-over, public release readiness, broad accessibility
completion, playtest/fun-hypothesis validation, full playable-client manual QA,
and full game completion are not claimed.

## Close-Out Status

Sprint 7 is **closed with conditions** as of 2026-05-07.

| Item | Final status | Evidence |
|------|--------------|----------|
| Must Have scope | Complete | PLAYABLE-001, PLAYABLE-002, and PLAYABLE-003 are Complete in `production/sprint-status.yaml` |
| Nice To Have scope | Complete | S7-N1 evidence index cleanup is Complete |
| Conditional Should Have scope | Not pulled; remains backlog | SAU-007 and ECO-004 stay backlog and are not required for Sprint 7 closure |
| Smoke | PASS WITH WARNINGS | `production/qa/smoke-sprint-7-2026-05-07.md` |
| QA sign-off | APPROVED WITH CONDITIONS | `production/qa/qa-signoff-sprint-7-2026-05-07.md` |
| Verified endpoint | next-loop DRAFT_SHOP after post-auction placement/resolution | `production/qa/evidence/sprint-7-friend-game-evidence-index.md` |

Carried conditions:

- QA-COND-0005 remains friend-game-only accepted risk. This is not verified
  Standard-tier accessibility completion and does not apply to any public,
  external, commercial, or broader release candidate.
- QA-COND-0006 remains accepted-risk/deferred. This is not playtest evidence,
  fun-hypothesis validation, a playtest report, or a public QA claim.
- No public release readiness is claimed.
- No game-over coverage is claimed.
- No full playable-client manual QA, full game completion, broad accessibility
  completion, or playtest/fun-hypothesis validation is claimed.
- The honest endpoint remains next-loop DRAFT_SHOP after post-auction
  placement/resolution.

Sprint 8 planning is safe as the next docs-planning step only if explicitly
requested and only with the carried conditions above preserved. No Sprint 8 plan
was created by this close-out.

## Out of Scope

- Public, external, commercial, or release-candidate readiness.
- Broad Standard-tier accessibility completion.
- Claiming QA-COND-0005 as verified accessibility completion.
- Claiming QA-COND-0006 as playtest evidence or fun-hypothesis validation.
- Full playable-client manual QA beyond the scoped internal friend-game
  evidence required by PLAYABLE-003.
- New game modes, 2v2/3v3 scope, broad class/keyword/prism polish, store or
  deployment readiness.
- Implementing code as part of this sprint-planning prompt.

## Definition of Done for this Sprint

- [x] PLAYABLE-001, PLAYABLE-002, and PLAYABLE-003 story docs exist and pass
      readiness before implementation starts.
- [x] All Must Have tasks completed.
- [x] Real primary client path supports an internal friend-game session through
      lobby/session entry, draft/shop, placement, resolution, and next-loop or
      documented nearest reachable endpoint.
- [x] Friend-game manual evidence exists for two real clients with no
      harness-injected state.
- [x] Sprint 7 QA plan exists before implementation begins.
- [x] All Logic/Integration stories have passing unit/integration tests.
- [x] Smoke check passed before sprint closure.
- [x] QA sign-off report is complete before any future gate or release claim.
- [x] No S1 or S2 bugs remain in the scoped friend-game path.
- [x] QA-COND-0005 remains labeled accepted risk unless separately verified.
- [x] QA-COND-0006 remains labeled accepted-risk/deferred unless actual
      playtest evidence is produced later.

## Next Recommended Step

When the user explicitly requests it, run `/sprint-plan new` for Sprint 8
planning. Keep QA-COND-0005, QA-COND-0006, no public release readiness, no
game-over coverage, and the next-loop DRAFT_SHOP endpoint as carried Sprint 8
planning constraints.

## Historical First Recommended Prompt After Plan

This historical prompt was completed earlier in Sprint 7 and is no longer the
current next step after close-out.

`PROMPT 269 -- Create Sprint 7 PLAYABLE Story Docs and Readiness`

Create `PLAYABLE-001`, `PLAYABLE-002`, and `PLAYABLE-003` story files from this
Sprint 7 plan as docs-only work. Do not implement code. Include acceptance
criteria, dependencies, Bevy 0.18 and Lightyear 0.26 guardrails, and explicit
no-claim language for public release readiness, Standard-tier accessibility
completion, playtest validation, and full playable-client manual QA. After the
story docs are ready, run `/qa-plan sprint-7` before `/dev-story`.

---

**Scope check:** Sprint 7 is a Polish friend-game sprint. If any work beyond the
listed Must Have and conditional Should Have rows is proposed, run `/scope-check`
before implementation begins and confirm it does not displace the real primary
client path.
