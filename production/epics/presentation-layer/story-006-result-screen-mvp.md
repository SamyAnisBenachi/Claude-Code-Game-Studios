# Story 006: Result Screen MVP

> **Epic**: Presentation Layer
> **Status**: Blocked - contract gaps
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 9 preparation only - Sprint 9 is not active

## Context

This is a docs-only preparation story for the post-match result screen. It does
not implement code, activate Sprint 9, close Sprint 8, run a gate, or claim
manual/browser GAME_OVER evidence.

**Primary sources**:

- `design/ux/result-screen.md`
- `design/gdd/network-protocol.md`
- `design/gdd/round-state-machine.md`
- `design/gdd/objective-system.md`
- `design/gdd/hud.md`
- `production/epics/playable-client/story-004-friend-game-result-endpoint-expansion.md`
- `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/playable-004-result-endpoint-trace.json`
- `production/qa/qa-signoff-sprint-8-2026-05-07.md`
- `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md`
- `production/qa/bugs/QA-COND-0006-playtest-fun-hypothesis-evidence.md`

**Current evidence boundary**:

PLAYABLE-004 proves a controlled internal friend-game endpoint where both real
primary clients observe `S2CGameOver` and `S2CPhaseChanged(GameOver)` through a
real Lightyear server/client route. The observed endpoint is a draw with
`loser = None` and `reason = Draw`.

That evidence does not prove a rendered result screen, manual/browser GAME_OVER,
public release readiness, broad Standard-tier accessibility completion,
playtest validation, full playable-client manual QA, or full game completion.

**GDD, UX, and TR trace**:

- `design/ux/result-screen.md` is the direct UX source for the result overlay,
  outcome copy, objective summary, Return to Lobby, rematch unavailable state,
  keyboard focus, reduced-motion behavior, and fallback states.
- `design/gdd/network-protocol.md` defines `S2CGameOver`,
  `S2CGameSnapshot`, `C2SAcknowledgeResult`, and the GAME_OVER valid C2S phase.
- `TR-NP-001` requires the server to remain authoritative and the client to
  express intent through C2S messages only.
- `TR-NP-005` requires invalid-phase C2S messages to be silently discarded.
- `TR-RSM-008` defines GAME_OVER detection and draw behavior after objective
  destruction.
- `TR-RSM-009` defines reliable `S2CPhaseChanged` broadcast ordering.
- `TR-HUD-009` defines FROZEN HUD mode on GAME_OVER.

No dedicated `TR-PRES-*` entry exists for the result screen. This story does
not invent one. If architecture review later registers a result-screen TR, the
story should be updated before implementation assignment.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md)
- [ADR-011: Reconnect Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

All referenced ADRs are Accepted.

**ADR Decision Summary**: The result screen is a read-only presentation surface
over server-authoritative result, phase, snapshot, HUD, and objective data. It
may send `C2SAcknowledgeResult` as a cleanup handshake, but it must not infer
hidden objective identities, locally decide the winner, start a rematch, or
mutate gameplay state.

**Engine**: Bevy 0.18 + Lightyear 0.26 + browser/WASM primary client |
**Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing any Bevy `.rs` file and
`liv-bevy-lightyear` before editing any Lightyear or networking `.rs` file.
Result-screen UI must use Bevy 0.18 Required Components API. Do not use
deprecated bundle APIs. Do not add duplicate Lightyear `MessageReceiver`
drains. If a shared result/snapshot resource exists when implementation starts,
the result screen must read that resource rather than draining the same message
type independently.

**Control Manifest Rules (2026-05-05)**:

- Required: client presentation is a read-only view of server-authoritative
  state.
- Required: `S2CPhaseChanged` is drained only by the shared phase sink.
- Required: presentation work runs in ADR-021 order:
  `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick`.
- Required: UI overlays use `bevy_ui`; board content remains world-space.
- Forbidden: client presentation must not assert or mutate authoritative game
  state.
- Forbidden: never infer hidden opponent objective identities on the client.
- Guardrail: steady-state presentation work stays below 1 ms per frame and
  phase-boundary presentation spikes stay below 3 ms.

---

## Scope

### In Scope

- Add a visible result overlay when the client reaches `GAME_OVER`.
- Render outcome copy for win, loss, draw, no-result, and missing-result
  fallback states.
- Display an objective summary for both players using only
  server-authoritative snapshot/result data.
- Display `Unknown` or a clear fallback when objective identity, final
  resources, final round, or result data is absent.
- Keep the HUD and board/background visible in frozen final-state mode behind
  the overlay.
- Provide Return to Lobby as the available exit action.
- Send `C2SAcknowledgeResult` from Return to Lobby only if the current
  acknowledgement contract is implemented or explicitly accepted for MVP.
- Disable or hide rematch unless rematch protocol support is explicitly scoped
  by another accepted story.
- Provide keyboard focus order, visible focus indicators, and reduced-motion
  behavior defined by `design/ux/result-screen.md`.
- Preserve the exact evidence boundary: controlled internal GAME_OVER is
  supported by PLAYABLE-004, while manual/browser GAME_OVER is not claimed.

### Out of Scope

- No server-side result contract, acknowledgement handler, rematch protocol, or
  reconnect payload implementation unless a separate owning story scopes it.
- No public, external, commercial, store, deployment, release-candidate, or
  launch readiness claim.
- No broad Standard-tier accessibility completion or QA-COND-0005 closure.
- No playtest evidence, fun-hypothesis validation, playtest report, or
  QA-COND-0006 closure.
- No full playable-client manual QA.
- No manual/browser GAME_OVER claim unless a future manual/browser evidence
  story actually captures it.
- No full game completion claim.
- No scoreboard/HUD real/fake reveal mutation.
- No client-inferred hidden objective identities.
- No rematch flow beyond disabled/hidden UI state.
- No broad UI redesign of HUD, board rendering, hand UI, shop/auction UI, or
  main menu.

---

## Acceptance Criteria

- [ ] **Result overlay appears**: GIVEN the client receives
      `S2CPhaseChanged(GameOver)` with a cached `S2CGameOver` payload, WHEN the
      presentation phase sink updates `CurrentClientPhase`, THEN a visible
      result overlay appears above the frozen board/HUD without requiring player
      input.
- [ ] **Missing result fallback appears**: GIVEN the client reaches
      `RoundPhase::GameOver` from `S2CGameSnapshot.phase` or phase change but
      has no cached or re-sent `S2CGameOver`, WHEN the overlay opens, THEN it
      shows a no-result or pending-result fallback, keeps Return to Lobby
      usable, and does not invent loser, reason, or objective-reveal data.
- [ ] **Victory copy is correct**: GIVEN `S2CGameOver.loser == Some(opponent)`
      and `reason == ObjectivesDestroyed`, WHEN the result overlay renders,
      THEN the headline reads `VICTORY` and the cause states that the opponent
      lost two real objectives.
- [ ] **Defeat copy is correct**: GIVEN `S2CGameOver.loser == Some(local_player)`
      and `reason == ObjectivesDestroyed`, WHEN the result overlay renders,
      THEN the headline reads `DEFEAT` and the cause states that two local real
      objectives were destroyed.
- [ ] **Draw copy is correct**: GIVEN `S2CGameOver.loser == None` and
      `reason == Draw`, WHEN the result overlay renders, THEN the headline
      reads `DRAW` and no player is presented as the winner.
- [ ] **No-result copy is correct**: GIVEN `reason == ResolutionTimeout`, WHEN
      the result overlay renders, THEN the headline reads `NO RESULT` and the
      cause states that resolution timed out without declaring a winner.
- [ ] **Disconnect result copy is correct**: GIVEN
      `reason == Disconnect`, WHEN the result overlay renders, THEN copy uses
      plain connection-loss language and maps win or loss only from the
      server-authoritative `loser` field.
- [ ] **Objective summary uses authoritative data only**: GIVEN the current
      snapshot contains own `objectives` and opponent `opponent_objectives`,
      WHEN the objective summary renders, THEN it displays five lanes per side
      with lane, HP, alive/destroyed state, and real/fake identity only where
      the server has provided that identity.
- [ ] **Unknown objective fallback is explicit**: GIVEN an opponent objective
      is alive and no post-game reveal payload provides its identity, WHEN the
      objective row renders, THEN the identity reads `Unknown` or equivalent
      fallback copy rather than a client-inferred real/fake value.
- [ ] **Destroyed opponent reveal is displayed**: GIVEN
      `OpponentObjectiveSnapshot.was_fake` is `Some(value)` for a destroyed
      opponent objective, WHEN the row renders, THEN the row displays `FAKE`
      for `true` and `REAL` for `false`.
- [ ] **Missing objective data fallback is explicit**: GIVEN a snapshot lacks
      objective data or has fewer than five lanes for either side, WHEN the
      overlay renders, THEN the summary remains stable and marks missing rows
      as `Unknown` or `Unavailable` without panicking or hiding Return to Lobby.
- [ ] **Final summary uses available authoritative fields**: GIVEN snapshot or
      result data contains final round, gold, current mana, mana cap, reserve
      mana, and real/fake objective counts, WHEN the summary renders, THEN those
      fields are displayed from authoritative data; absent fields are omitted or
      labelled `Unknown`.
- [ ] **HUD and background remain frozen**: GIVEN HUD entered FROZEN mode from
      `S2CPhaseChanged(GameOver)`, WHEN the result overlay renders and receives
      later incremental HUD/economy/objective messages, THEN HUD remains visible
      behind the overlay, final values are retained, and the result screen does
      not add real/fake markers to HUD dots.
- [ ] **Return to Lobby is available**: GIVEN the result overlay is open, WHEN
      the player activates Return to Lobby by click or Enter, THEN the client
      sends or has already sent `C2SAcknowledgeResult` when the acknowledgement
      contract is available, clears local ended-session result UI state, and
      routes to the main-menu/lobby flow.
- [ ] **Return does not require rematch support**: GIVEN rematch protocol is not
      scoped, WHEN the player activates Return to Lobby, THEN the route does not
      wait for rematch negotiation or a server-authoritative new session.
- [ ] **Rematch is disabled or hidden**: GIVEN no accepted rematch protocol
      exists for this story, WHEN the overlay renders, THEN Rematch is disabled
      or hidden and no rematch C2S message is sent.
- [ ] **Keyboard focus is deterministic**: GIVEN the overlay opens, WHEN the
      player uses keyboard navigation, THEN focus starts on the result heading
      or first available action according to implementation capability, Tab
      reaches Return to Lobby in logical order, Enter activates the focused
      action, and Escape focuses Return to Lobby without automatically leaving.
- [ ] **Focus indicators are visible**: GIVEN any button or optional objective
      row receives keyboard focus, WHEN focus is active, THEN a high-contrast
      focus indicator is visible and does not overlap text in a way that makes
      the label unreadable.
- [ ] **Reduced motion is honored**: GIVEN reduced-motion mode is enabled, WHEN
      the overlay opens or objective rows reveal, THEN iris wipe, bloom flash,
      scale pulse, row sequencing, and repeated flashes are removed or replaced
      by an instant state or simple fade while preserving all result
      information.
- [ ] **Photosensitivity guard holds**: GIVEN standard-motion mode is enabled,
      WHEN the result overlay opens, THEN no result-entry effect flashes more
      than three times per second and any GAME_OVER burst is single-shot only.
- [ ] **Viewport layout is stable**: GIVEN viewports `1366x768` and
      `1920x1080` plus 150 percent UI scale, WHEN the overlay renders all
      outcome states, THEN headline, cause, objective rows, summary, fallback
      copy, and buttons remain readable and do not overlap.
- [ ] **No release or QA overclaim**: GIVEN implementation evidence is written,
      WHEN the evidence and completion notes are reviewed, THEN they state that
      the story does not claim public release readiness, broad accessibility
      completion, playtest validation, fun-hypothesis validation, full
      playable-client manual QA, manual/browser GAME_OVER, or full game
      completion.
- [ ] **QA conditions remain carried**: GIVEN the story completes, WHEN
      condition files and evidence are reviewed, THEN QA-COND-0005 remains
      accepted risk for friend-game scope only unless separately verified under
      a future accessibility scope, and QA-COND-0006 remains
      accepted-risk/deferred unless actual playtest evidence is produced later.
- [ ] **Whitespace gate passes**: `git diff --check` passes.

---

## Implementation Notes

- Prefer an owning result-screen presentation module such as
  `client/src/presentation/result_screen.rs` if no existing module is more
  appropriate at implementation time.
- Cache the most recent `S2CGameOver` in a presentation read model if no shared
  result read model exists. Keep exactly one production drain for
  `MessageReceiver<S2CGameOver>`.
- Read current phase through `CurrentClientPhase`; do not drain
  `S2CPhaseChanged` outside the shared phase sink.
- Read final objective, resource, and round data from `S2CGameSnapshot` or
  existing authoritative presentation read models. Do not recompute final
  resources from deltas.
- Treat opponent alive objective identity as unavailable unless the server adds
  a post-game reveal payload or a GAME_OVER-specific snapshot projection.
- Use `C2SAcknowledgeResult` only as a cleanup handshake. It is not a result
  commit and must not gate whether the local player can read the result.
- Hide or disable rematch for MVP. A disabled control must not receive keyboard
  focus. If hidden, no rematch status copy is required.
- Preserve HUD ownership of FROZEN behavior. The result screen may dim or cover
  the scene, but it must not mutate HUD entities or scoreboard dot identities.
- If manual/browser evidence is later added, label it separately from
  PLAYABLE-004 controlled real-Lightyear evidence.

## Performance Budget

The overlay must preserve ADR-021 presentation guardrails: steady-state
presentation work below 1 ms per frame and phase-boundary spikes below 3 ms.
The result screen must not add per-frame entity spawning, duplicate Lightyear
message drains, polling for snapshots, or server-authority recomputation on the
client.

---

## QA Test Cases

- **Overlay from GAME_OVER**
  - Given: a client test app has `CurrentClientPhase.phase == GameOver`, a local
    player id, a cached `S2CGameOver`, and a final snapshot.
  - When: result-screen presentation systems run.
  - Then: the result overlay is visible above the frozen scene and outcome copy
    matches the server-authored loser/reason fields.

- **Missing payload fallback**
  - Given: a reconnect or snapshot-only path has `RoundPhase::GameOver` and no
    cached `S2CGameOver`.
  - When: the overlay renders.
  - Then: fallback copy is shown, Return to Lobby remains available, and winner,
    loser, reason, and hidden objective identities are not invented.

- **Objective summary fallback**
  - Given: own objectives are fully known, destroyed opponent objectives include
    `was_fake`, and alive opponent objectives have no reveal data.
  - When: objective rows render.
  - Then: own rows show authoritative identity, destroyed opponent rows show
    authoritative identity, and alive opponent rows show `Unknown`.

- **Frozen HUD preservation**
  - Given: HUD is frozen at GAME_OVER with final values.
  - When: the result overlay opens and later incremental HUD/economy/objective
    messages arrive.
  - Then: result overlay remains visible, HUD final values remain unchanged, and
    no HUD dot receives a new real/fake marker.

- **Return to Lobby**
  - Given: the overlay is open and Return to Lobby is focused.
  - When: Enter is pressed.
  - Then: `C2SAcknowledgeResult` is sent if the acknowledgement path is in
    scope, local result UI state is cleared, and the route transitions to the
    main-menu/lobby flow.

- **Accessibility and motion**
  - Given: standard motion and reduced-motion modes are tested at required
    viewports.
  - When: the overlay opens, focus moves, and Return to Lobby is activated.
  - Then: focus order is deterministic, focus indicators are visible, reduced
    motion removes sequencing and flash effects, and text does not overlap.

- **Scope guard review**
  - Given: result-screen evidence and completion notes exist.
  - When: QA reviews them.
  - Then: all non-claims are explicit, QA-COND-0005 and QA-COND-0006 are not
    closed, and manual/browser GAME_OVER is not claimed unless separate
    evidence exists.

---

## Test Evidence

**Story Type**: UI

**Required automated test target**:

- `tests/integration/presentation/result_screen_mvp_test.rs`
  - Registered as `result_screen_mvp_test`
  - Command: `cargo test -p client --test result_screen_mvp_test`

**Required regression commands**:

- `cargo test -p client --test hud_game_over_freeze_test`
- `cargo test -p server --test playable_client_friend_game_result_endpoint_test`
- `cargo check -p client`
- `git diff --check`

**Required evidence document**:

- `production/qa/evidence/result-screen-mvp-evidence.md`

**Required evidence contents**:

- Commit, branch, build target, and command summary.
- Outcome-copy table for victory, defeat, draw, no-result, disconnect, and
  missing-payload fallback.
- Objective-summary table showing own identities, destroyed opponent
  `was_fake`, alive opponent `Unknown`, and missing-data fallbacks.
- HUD frozen-state preservation summary.
- Return to Lobby behavior and acknowledgement-path disposition.
- Rematch hidden/disabled disposition.
- Keyboard focus and reduced-motion evidence.
- Viewport/layout evidence for `1366x768`, `1920x1080`, and 150 percent UI
  scale.
- Explicit non-claims for public release readiness, broad accessibility
  completion, playtest validation, fun-hypothesis validation, full
  playable-client manual QA, manual/browser GAME_OVER, and full game
  completion.
- QA-COND-0005 and QA-COND-0006 impact statement.

**Status**: [ ] Not yet implemented or captured.

---

## Dependencies

- Depends on: Presentation Layer Story 001 complete for the shared phase sink
  and `PresentationSet` order.
- Depends on: HUD Story 007 complete for GAME_OVER FROZEN mode.
- Depends on: HUD Story 008 complete for snapshot rebuild while FROZEN.
- Depends on: PLAYABLE-004 complete for controlled internal `GAME_OVER`
  endpoint evidence.
- Depends on: ADR-002, ADR-008, ADR-011, and ADR-021 Accepted.
- Blocks: Result-screen implementation assignment until the blockers below are
  resolved or explicitly accepted as MVP fallback scope.

## Blockers

- **RS-BLOCK-001 - Result acknowledgement handler**:
  `C2SAcknowledgeResult` exists in `shared/src/protocol.rs` and is registered on
  `ReliableChannel`, but current server source only logs it in
  `server/src/network/mod.rs`. No implemented server-side result
  acknowledgement handler, all-player acknowledgement tracking, or
  `ack_timeout_ms` cleanup path was found. Unblock by implementing the GSS or
  network acknowledgement contract, or by explicitly accepting a client-only
  Return to Lobby MVP where the ack send has no server cleanup effect.

- **RS-BLOCK-002 - GAME_OVER reconnect result payload**:
  `S2CGameSnapshot` includes `phase: RoundPhase` but does not include the final
  `S2CGameOver` payload fields `loser`, `round`, and `reason`. A reconnecting
  client at GAME_OVER can only show fallback result copy unless the server
  re-sends `S2CGameOver` or adds an authoritative result payload to the
  snapshot. Unblock by extending the protocol/reconnect contract, or by
  accepting the fallback behavior for MVP.

- **RS-BLOCK-003 - Full post-game objective reveal contract**:
  `OpponentObjectiveSnapshot.was_fake` only reveals destroyed opponent
  objectives. Alive opponent objective identities remain absent from the
  current snapshot contract. Unblock by adding a post-game reveal payload or
  GAME_OVER-specific snapshot projection, or by accepting `Unknown` for alive
  opponent lanes in the MVP.

- **RS-BLOCK-004 - Rematch protocol undefined**:
  No rematch C2S/S2C protocol, session negotiation, or same-session/fresh-lobby
  decision is currently scoped. This does not block the MVP if Rematch is hidden
  or disabled, but it blocks any enabled rematch button.

## Readiness Notes

**Implementation readiness verdict**: BLOCKED.

This story is specific enough for review, but it should not be assigned for
implementation until RS-BLOCK-001 through RS-BLOCK-003 are either resolved or
explicitly accepted as MVP fallback constraints. RS-BLOCK-004 is not blocking
only if Rematch remains hidden or disabled.

The Sprint 9 preparation state is docs-only. Do not update
`production/sprint-status.yaml`, `production/session-state/**`, Sprint 8
close-out files, or any Sprint 9 activation file from this story.
