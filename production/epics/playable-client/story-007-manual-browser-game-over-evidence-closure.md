# Story 007: Manual Browser GAME_OVER Evidence Closure

> **Epic**: Playable Client
> **Status**: Blocked - depends on result screen and acknowledgement handshake
> **Layer**: Polish / QA Evidence
> **Type**: Integration
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 9 preparation only - Sprint 9 is not active

## Context

This is the S9-QA-001 story from the Sprint 9 draft. It owns the carried manual
browser or native two-client GAME_OVER evidence gap from Sprint 8.

Sprint 8 closed with controlled real-Lightyear friend-game evidence reaching
`GAME_OVER`, but the full manually driven browser or native two-client GUI route
was not captured. This story turns that warning into a bounded Sprint 9 evidence
closure path after the Result Screen MVP and result acknowledgement cleanup
handshake are available.

This story is evidence work, not a broad implementation story. If the route
finds a blocking product defect, the evidence must record the blocker and keep
the Sprint 8 warning open instead of weakening the route or claiming closure.

This story does not implement code during Sprint 9 preparation, activate Sprint
9, close Sprint 8, run smoke, run QA sign-off, run a gate, run `/dev-story`, or
run `/story-done`.

**Primary sources**:

- `production/sprints/sprint-9-draft.md`
- `production/sprints/sprint-8.md`
- `production/qa/smoke-sprint-8-2026-05-07.md`
- `production/qa/qa-signoff-sprint-8-2026-05-07.md`
- `production/qa/evidence/sprint-8-friend-game-evidence-index.md`
- `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`
- `production/epics/playable-client/story-004-friend-game-result-endpoint-expansion.md`
- `production/epics/game-session-system/story-010-result-acknowledgement-cleanup-handshake.md`
- `production/epics/presentation-layer/story-006-result-screen-mvp.md`
- `design/ux/result-screen.md`
- `design/gdd/network-protocol.md`
- `design/gdd/round-state-machine.md`
- `design/gdd/hud.md`
- `docs/architecture/adr-002-client-server-authority.md`
- `docs/architecture/adr-008-lightyear-channel-config.md`
- `docs/architecture/adr-011-reconnect-snapshot.md`
- `docs/architecture/adr-021-presentation-layer-architecture.md`

**GDD, UX, and TR trace**:

- `design/gdd/network-protocol.md` and `TR-NP-001`: clients express intent
  through C2S messages only and the server remains authoritative.
- `design/gdd/network-protocol.md` and `TR-NP-009`: resolution messages precede
  the following phase change on the reliable channel.
- `design/gdd/network-protocol.md` and `TR-NP-011`: placement reveal contains
  both players' placements atomically.
- `design/gdd/round-state-machine.md` Rule 11 and `TR-RSM-008`: GAME_OVER is
  detected after objective loss, with mutual destruction producing a draw.
- `design/gdd/round-state-machine.md` and `TR-RSM-009`: `S2CPhaseChanged` is
  broadcast on every transition and after phase-entry work.
- `design/gdd/hud.md` Rule 10 and `TR-HUD-009`: HUD enters FROZEN mode on
  `GAME_OVER` and remains a final-state record beneath the result overlay.
- `design/ux/result-screen.md`: result overlay displays outcome, cause,
  objective summary, Return to Lobby, rematch unavailable state, focus behavior,
  reduced-motion behavior, and fallback copy.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md)
- [ADR-011: Reconnect Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

All referenced ADRs are Accepted.

**ADR Decision Summary**: Evidence must come from the real primary client path
using real client/server messages. It must not use direct `World` state
injection, fake snapshots, direct server feature calls, client-side authority,
or debug-only shortcuts as proof of the manual route.

**Engine**: Bevy 0.18 + Lightyear 0.26 + browser/WASM or native primary client |
**Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` for any Bevy follow-up discovered during
evidence collection and `liv-bevy-lightyear` for any networking follow-up. This
story's preparation scope is docs-only, but later evidence must use the real
primary client path and one real local server with two real clients.

**Control Manifest Rules (2026-05-05)**:

- Required: client presentation is a read-only view of server-authoritative
  state.
- Required: all non-heartbeat game-state messages use `ReliableChannel`.
- Required: `S2CPhaseChanged` is drained only by the shared phase sink.
- Required: reconnect snapshots are server-authoritative and secret-stripped.
- Forbidden: never use direct server feature calls, fake snapshots, or local
  client authority as endpoint proof.

---

## Scope

### In Scope

- Capture a full manually driven browser or native two-client route through:
  lobby create/join, class confirmation, draft/shop, auction, placement,
  resolution, `GAME_OVER`, Result Screen MVP display, and Return to Lobby
  acknowledgement.
- Use one real local server and two real primary clients from the same commit.
- Record exact commands, branch, commit, build target, client identities,
  browser/native environment, viewport if browser, and route steps.
- Capture evidence that both clients observe `S2CGameOver` and
  `S2CPhaseChanged(GameOver)` through real traffic.
- Capture evidence that the result screen displays the final outcome and that
  Return to Lobby performs the acknowledgement cleanup route from S9-RS-003.
- Close S8-QA-001-W1 only if the full manual/browser or native two-client route
  is actually captured.
- If the route remains blocked, record the exact blocker, affected owner, last
  reached route step, and keep S8-QA-001-W1 open.
- Preserve QA-COND-0005 as accepted risk for friend-game scope only unless a
  separate accessibility closure story provides actual evidence.
- Preserve QA-COND-0006 as accepted-risk/deferred unless a separate playtest
  workflow provides actual playtest evidence.

### Out of Scope

- No implementation work unless a separate story is assigned later.
- No replacement for S9-RS-001, S9-RS-002, or S9-RS-003.
- No public, external, commercial, store, deployment, release-candidate, or
  launch readiness claim.
- No broad Standard-tier accessibility completion or QA-COND-0005 closure.
- No playtest evidence, fun-hypothesis validation, playtest report, or
  QA-COND-0006 closure.
- No full playable-client manual QA beyond the scoped two-client friend-game
  route.
- No full regression campaign.
- No full game completion claim.
- No Sprint 9 activation, Sprint 8 close-out, smoke, QA sign-off, gate-check,
  `/dev-story`, or `/story-done`.
- No updates to `production/sprint-status.yaml` or
  `production/session-state/**`.

---

## Acceptance Criteria

- [ ] **Preconditions are checked**: GIVEN Sprint 9 is being activated or this
      evidence is being assigned, WHEN the work starts, THEN the status of
      S9-RS-002, S9-RS-003, and the Prompt 367 native blank lobby follow-up is
      recorded before any closure claim.
- [ ] **Two real clients are used**: GIVEN evidence collection starts, WHEN the
      route is captured, THEN it uses one real local server and two real primary
      clients from the same commit, not direct server calls, injected world
      state, fake snapshots, or harness-only client state.
- [ ] **Full route is captured or blocker is recorded**: GIVEN the route is
      exercised, WHEN evidence is finalized, THEN it either captures lobby
      create/join through result acknowledgement or records the exact last
      reached step and blocker without closing S8-QA-001-W1.
- [ ] **GAME_OVER traffic is observed**: GIVEN the route reaches GAME_OVER, WHEN
      evidence is reviewed, THEN both clients have evidence of
      `S2CGameOver` and `S2CPhaseChanged(GameOver)` from real traffic or logs.
- [ ] **Result screen is observed**: GIVEN both clients reach GAME_OVER, WHEN
      the result overlay appears, THEN evidence records outcome headline, cause,
      final round if available, objective summary fallback behavior, rematch
      disabled or hidden state, and HUD frozen underneath.
- [ ] **Return to Lobby is observed**: GIVEN the result overlay is open, WHEN
      Return to Lobby is activated on at least one client, THEN evidence records
      acknowledgement send disposition, local result UI cleanup, and route back
      to the lobby or menu flow.
- [ ] **S8-QA-001-W1 closure is exact**: GIVEN the full route is captured, WHEN
      the evidence index is updated, THEN it states exactly that the manual
      browser/native two-client friend-game GAME_OVER route was captured and
      does not expand the claim to full manual QA.
- [ ] **Blocked route keeps warning open**: GIVEN a blocker prevents the full
      route, WHEN evidence is written, THEN S8-QA-001-W1 remains open and the
      evidence records the blocker, owner, severity, workaround if any, and
      next recommended story.
- [ ] **Sprint 8 conditions are preserved**: GIVEN evidence is written, WHEN
      QA-COND-0005 and QA-COND-0006 are reviewed, THEN they remain carried as
      accepted-risk/deferred conditions unless separate actual closure evidence
      exists later.
- [ ] **No overclaim language appears**: GIVEN evidence and index updates are
      reviewed, WHEN non-claims are checked, THEN they do not claim public
      release readiness, broad accessibility completion, playtest validation,
      full playable-client manual QA, full regression, or full game completion.
- [ ] **Whitespace gates pass**: `git diff --check` passes and
      `git diff --cached --check` passes before commit.

---

## Implementation Notes

- Prefer browser/WASM evidence if the browser route is stable enough to capture
  the full flow. Native evidence is acceptable if the native blank lobby
  follow-up is resolved and the route is fully manually driven.
- Evidence may include screenshots, logs, command summaries, trace JSON, or a
  concise manual step transcript. The capture method must be repeatable enough
  for another contributor to audit the result.
- If the route needs a small repair, stop and create or assign a separate
  implementation story. This evidence story must not quietly absorb product
  work.
- If only controlled internal Lightyear evidence is available again, record it
  as supporting context and keep the manual/browser warning open.

## Performance Budget

No new runtime performance budget is introduced by this evidence story. Any
captured route should record obvious stalls, blank screens, hangs, or route
timeouts as defects rather than treating them as acceptable evidence noise.

---

## QA Test Cases

- **Full manual route captured**
  - Given: two real clients and one real local server are launched from the same
    commit.
  - When: the friend-game route is manually driven through Return to Lobby.
  - Then: evidence records every route step from lobby create/join through
    result acknowledgement.

- **GAME_OVER route blocked**
  - Given: the manual route cannot reach `GAME_OVER`.
  - When: evidence is finalized.
  - Then: the last reached step, blocker, owner, and S8-QA-001-W1 carried state
    are recorded without claiming closure.

- **Result screen observed**
  - Given: both clients receive GAME_OVER.
  - When: the Result Screen MVP opens.
  - Then: evidence records outcome copy, objective fallback behavior, frozen
    HUD, and rematch unavailable state.

- **Return to Lobby observed**
  - Given: the result screen is open.
  - When: Return to Lobby is activated.
  - Then: evidence records acknowledgement disposition and local route cleanup.

---

## Test Evidence

**Story Type**: Integration

**Required manual evidence document**:

- `production/qa/evidence/sprint-9-manual-game-over-evidence.md`

**Required capture directory**:

- `production/qa/evidence/captures/sprint-9-manual-game-over/`

**Required evidence index update after capture**:

- `production/qa/evidence/sprint-9-result-evidence-index.md`

**Required evidence contents**:

- Branch, commit, build target, and exact commands.
- Browser or native environment, including viewport for browser evidence.
- Two real clients and one real server from the same commit.
- Route step transcript from lobby create/join through Return to Lobby.
- `S2CGameOver` and `S2CPhaseChanged(GameOver)` observation for both clients
  when reached.
- Result screen observation, Return to Lobby acknowledgement disposition, and
  route cleanup observation.
- Any blocker, defect, workaround, owner, and carried warning state.
- Explicit non-claims for public release readiness, broad accessibility
  completion, playtest validation, full playable-client manual QA, full
  regression campaign, and full game completion.
- QA-COND-0005 and QA-COND-0006 carried-condition statement.

**Required verification commands for later evidence work**:

- `cargo check --workspace`
- relevant focused route or regression commands selected by S9-RS-002 and
  S9-RS-003 completion notes
- `git diff --check`
- `git diff --cached --check` before commit

**Status**: [ ] Not yet captured.

---

## Dependencies

- Depends on: [Presentation Layer Story 006](../presentation-layer/story-006-result-screen-mvp.md) complete for S9-RS-002 Result Screen MVP.
- Depends on: [Game Session System Story 010](../game-session-system/story-010-result-acknowledgement-cleanup-handshake.md) complete for S9-RS-003 Return to Lobby acknowledgement cleanup.
- Depends on: Prompt 367 native blank lobby status checked at Sprint 9
  activation time. If unresolved, native route evidence remains blocked and the
  browser route must be used or the blocker must be recorded.
- Depends on: Sprint 8 evidence files remaining explicit that manual/browser
  GAME_OVER is not yet claimed until this story captures it.
- Blocks: [Story 008](story-008-sprint-9-result-evidence-index-cleanup.md)
  when S9-QA-002 is pulled after evidence exists.

## Readiness Notes

**Implementation readiness verdict**: BLOCKED.

This story is a ready evidence package, but it cannot be assigned until
S9-RS-002 and S9-RS-003 are complete or explicitly replaced by
producer-approved fallbacks. Story preparation alone does not close S8-QA-001-W1
and does not activate Sprint 9.
