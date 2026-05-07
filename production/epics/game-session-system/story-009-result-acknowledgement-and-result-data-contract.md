# Story 009: Result Acknowledgement and Result Data Contract

> **Epic**: Game Session System
> **Status**: Ready
> **Layer**: Core / Networking
> **Type**: Integration
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 9 preparation only - Sprint 9 is not active

## Context

This is the S9-RS-001 prerequisite contract story for the Result Screen MVP.
It defines the server-owned result acknowledgement, final result data source,
and GAME_OVER reconnect behavior that Presentation Layer Story 006 must consume.

This story does not implement code, activate Sprint 9, close Sprint 8, run a
gate, run `/dev-story`, run `/story-done`, run smoke, or claim manual/browser
GAME_OVER evidence.

**Primary sources**:

- `design/ux/result-screen.md`
- `design/gdd/network-protocol.md`
- `design/gdd/game-session-system.md`
- `design/gdd/round-state-machine.md`
- `design/gdd/objective-system.md`
- `design/gdd/game-config.md`
- `docs/architecture/adr-002-client-server-authority.md`
- `docs/architecture/adr-008-lightyear-channel-config.md`
- `docs/architecture/adr-010-rsm-event-bus.md`
- `docs/architecture/adr-011-reconnect-snapshot.md`
- `production/epics/presentation-layer/story-006-result-screen-mvp.md`
- `production/sprints/sprint-9-draft.md`

**Current blocking gaps**:

- `C2SAcknowledgeResult` is registered in `shared/src/protocol.rs`, but
  `server/src/network/mod.rs` currently only logs the message.
- `S2CGameSnapshot` carries `phase: RoundPhase` and final player snapshot
  fields, but it does not carry `S2CGameOver` fields. A reconnecting client at
  GAME_OVER needs either a re-sent `S2CGameOver`, snapshot result fields, or an
  explicitly accepted no-result fallback.
- Current GAME_OVER teardown removes session resources immediately. The result
  screen needs retained authoritative result data during the acknowledgement
  window before terminal cleanup.
- Rematch protocol is undefined and remains out of scope.
- Full post-game reveal of alive opponent objective identities is not required
  for the MVP. Alive opponent lanes may remain `Unknown` unless a separate
  product scope changes that rule.

**GDD, UX, and TR trace**:

- `design/gdd/network-protocol.md` Rule 1: the server is authoritative and
  clients express intent with C2S messages only.
- `design/gdd/network-protocol.md` Rule 4 and `TR-NP-005`: invalid-phase C2S
  messages are silently discarded with no S2C response.
- `design/gdd/network-protocol.md` Table A: `C2SAcknowledgeResult {}` is valid
  only during `GAME_OVER` and confirms the rendered result as a UI handshake.
- `design/gdd/network-protocol.md` Section D.1 and `TR-NP-006`:
  `S2CGameSnapshot` is a per-player, secret-stripped reconnect snapshot.
- `design/gdd/network-protocol.md` edge case: if `C2SAcknowledgeResult` is
  never received after GAME_OVER, the server waits up to `ack_timeout_ms`
  before cleaning up the session. The game result is persisted regardless.
- `design/gdd/network-protocol.md` NP-51: `C2SAcknowledgeResult` outside
  GAME_OVER is silently discarded and starts no teardown.
- `design/gdd/game-config.md`: `ack_timeout_ms` default is `10000`.
- `design/ux/result-screen.md`: Return to Lobby sends or has already sent
  `C2SAcknowledgeResult`, clears local ended-session UI, and returns to the
  main-menu/lobby flow.
- `design/ux/result-screen.md`: reconnect at GAME_OVER either reconstructs the
  result screen from authoritative result payload or shows `RESULT PENDING`
  with Return to Lobby still usable.

**Accepted contract decisions for this story**:

1. **Acknowledgement timing**: the client sends `C2SAcknowledgeResult` from
   Return to Lobby. The result overlay may render without sending ack on first
   stable render.
2. **Server handling**: the server implements a real GAME_OVER acknowledgement
   handler in the session/network path. The current log-only drain is not an
   accepted fallback.
3. **Result data source**: GAME_OVER reconnect uses a retained per-player final
   snapshot plus a re-sent retained `S2CGameOver`. This story does not require
   adding final result fields to `S2CGameSnapshot`.
4. **Cleanup boundary**: terminal cleanup of ended-session result data waits
   until all result participants have acknowledged or `ack_timeout_ms` expires.
5. **Post-cleanup reconnect fallback**: after terminal cleanup, reconnect does
   not reconstruct result state. The server uses the existing expired-session
   rejection or close path, and the client returns to lobby/menu flow.
6. **Objective reveal scope**: destroyed opponent objective identity uses
   `OpponentObjectiveSnapshot.was_fake`. Alive opponent objective identity
   remains `Unknown` unless a separate story adds authoritative reveal payload.
7. **Rematch scope**: rematch remains hidden or disabled. No rematch protocol
   or new-session negotiation is added here.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md)
- [ADR-010: RSM Event Bus](../../../docs/architecture/adr-010-rsm-event-bus.md)
- [ADR-011: Reconnect Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md)

All referenced ADRs are Accepted.

**ADR Decision Summary**: The result acknowledgement is a server-side cleanup
handshake, not a game-result commit. The server remains authoritative for
result, phase, final snapshot, reconnect, and cleanup timing. The client may
clear local result UI on Return to Lobby, but it must not infer hidden data,
start rematch, or mutate server-owned session state optimistically.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing Bevy `.rs` files and
`liv-bevy-lightyear` before editing Lightyear/networking `.rs` files. Keep
exactly one production drain for `MessageReceiver<C2SAcknowledgeResult>`.
Use Bevy 0.18 `MessageReader` / `MessageWriter` APIs for buffered server
signals. Do not introduce duplicate Lightyear message drains or client-side
authority shortcuts.

**Control Manifest Rules (2026-05-05)**:

- Required: clients are read-only views of server-authoritative game state.
- Required: invalid-phase C2S handlers silently discard messages with no S2C
  response.
- Required: all non-heartbeat game-state messages use `ReliableChannel`.
- Required: `S2CGameSnapshot` is unicast per player and secret-stripped.
- Required: reconnect sends snapshot before live messages and flushes deferred
  messages only after `snapshot_sent == true`.
- Forbidden: never infer hidden opponent objective identities on the client.
- Forbidden: never let a client commit or mutate authoritative server state
  optimistically.

---

## Scope

### In Scope

- Define and implement the server handler for `C2SAcknowledgeResult`.
- Move acknowledgement ownership out of the current log-only generic drain and
  into the session/network path that can resolve sender, phase, and ended
  session state.
- Record result acknowledgements per stable `PlayerId`.
- Treat duplicate acknowledgements from the same player as no-ops.
- Treat acknowledgements from unmapped peers, stale tokens, or non-participant
  players as silent discards.
- Gate `C2SAcknowledgeResult` to `GAME_OVER` only.
- Preserve authoritative ended-session result data through the ack window:
  retained `S2CGameOver`, retained per-player final `S2CGameSnapshot`, ack set,
  participant set, and expiry timestamp based on `ack_timeout_ms`.
- On reconnect during the retained GAME_OVER window, re-send the retained
  per-player final snapshot and the retained `S2CGameOver` before allowing
  result-screen presentation to rebuild.
- After all result participants acknowledge or `ack_timeout_ms` expires, perform
  terminal cleanup of ended-session result state, reconnect token entries, and
  deferred reconnect queues.
- Preserve Return to Lobby as a local UI route that sends acknowledgement when
  connected and clears only local ended-session UI.
- Keep Rematch hidden or disabled.
- Preserve `Unknown` for alive opponent objective identities.
- Add automated integration tests for acknowledgement handling, duplicate ack
  safety, timeout cleanup, invalid-phase discard, and GAME_OVER reconnect.

### Out of Scope

- No Result Screen UI implementation. Presentation Layer Story 006 owns UI.
- No rematch protocol, rematch request message, rematch acceptance state, or
  same-session rematch negotiation.
- No full post-game alive-opponent objective reveal payload.
- No `S2CPostGameSummary` message.
- No required `S2CGameSnapshot` schema expansion for final result fields.
- No match history persistence, leaderboard updates, rewards, analytics, or
  public release readiness claim.
- No Sprint 9 activation, Sprint 8 close-out, smoke, QA sign-off, gate-check,
  `/dev-story`, or `/story-done`.
- No updates to `production/sprint-status.yaml` or
  `production/session-state/**`.

---

## Acceptance Criteria

- [ ] **Acknowledgement drain is owned by GSS**: GIVEN
      `C2SAcknowledgeResult` is registered on `ReliableChannel`, WHEN the
      server plugin is built, THEN exactly one production handler drains
      `MessageReceiver<C2SAcknowledgeResult>` and that handler can resolve the
      sender to a stable `PlayerId`.
- [ ] **Log-only fallback is removed or bypassed**: GIVEN
      `server/src/network/mod.rs` currently logs `C2SAcknowledgeResult`, WHEN
      this story is implemented, THEN acknowledgement behavior is no longer
      limited to logging and a real session-owned outcome is tested.
- [ ] **Invalid phase discard**: GIVEN the server is in any phase before
      GAME_OVER, WHEN a mapped player sends `C2SAcknowledgeResult`, THEN the
      server silently discards it, sends no S2C response, starts no cleanup, and
      mutates no ended-session state.
- [ ] **Unknown sender discard**: GIVEN an unmapped peer, stale token, or
      non-participant player sends `C2SAcknowledgeResult`, WHEN the handler
      runs, THEN the message is silently discarded and no session state changes.
- [ ] **Result state is retained before terminal cleanup**: GIVEN
      `GameOverEmitted { loser, round, reason }` is consumed, WHEN GSS
      broadcasts `S2CGameOver`, THEN it also stores retained ended-session
      state containing the `S2CGameOver`, participant ids, per-player final
      `S2CGameSnapshot` payloads, an empty acknowledgement set, and an expiry
      time of now plus `ack_timeout_ms`.
- [ ] **Result data source is explicit**: GIVEN a reconnecting client enters
      GAME_OVER during the retained ended-session window, WHEN the server sends
      recovery messages, THEN result headline, round, and cause come from the
      retained `S2CGameOver`; final resources and objective rows come from the
      retained per-player final `S2CGameSnapshot`.
- [ ] **No snapshot result-field dependency**: GIVEN the chosen contract uses
      retained `S2CGameOver` resend, WHEN `S2CGameSnapshot` is inspected, THEN
      this story does not require `loser`, `round`, or `reason` fields to be
      added to the snapshot schema.
- [ ] **GAME_OVER reconnect resend ordering**: GIVEN a valid session token
      reconnects before ack cleanup, WHEN the reconnect handler processes
      `C2SHello`, THEN the reconnecting player receives `S2CHandshake`,
      retained `S2CGameSnapshot`, any required own objective identity restore,
      retained `S2CGameOver`, and `S2CPhaseChanged(GameOver)` on
      `ReliableChannel` before deferred live messages are flushed.
- [ ] **Reconnect after cleanup is documented fallback**: GIVEN all players have
      acknowledged or `ack_timeout_ms` expired, WHEN a client attempts to
      reconnect with the old token, THEN the server does not reconstruct result
      state and uses the existing expired-session rejection or close path.
- [ ] **Acknowledgement marks only the sender**: GIVEN the ended-session state is
      retained and Player A sends `C2SAcknowledgeResult`, WHEN the handler runs,
      THEN Player A is marked acknowledged exactly once and Player B remains
      unacknowledged.
- [ ] **Duplicate acknowledgement is safe**: GIVEN Player A has already been
      marked acknowledged, WHEN Player A sends `C2SAcknowledgeResult` again
      from the same or a reconnected peer, THEN the second message is a no-op,
      sends no S2C response, and does not panic.
- [ ] **All-ack cleanup**: GIVEN all result participants are acknowledged,
      WHEN the acknowledgement handler completes, THEN terminal cleanup removes
      ended-session result state, reconnect token entries for the session, and
      deferred reconnect queue entries for the session.
- [ ] **Timeout cleanup**: GIVEN at least one participant never sends
      `C2SAcknowledgeResult`, WHEN `ack_timeout_ms` elapses after GAME_OVER,
      THEN the server performs the same terminal cleanup as the all-ack path.
- [ ] **Result persistence is not gated by ack**: GIVEN GAME_OVER has been
      emitted, WHEN acknowledgement is missing, duplicated, or delayed, THEN the
      server-authoritative result remains the emitted `S2CGameOver`; ack never
      changes loser, round, or reason.
- [ ] **Return to Lobby behavior is bounded**: GIVEN the result overlay is open,
      WHEN the player activates Return to Lobby, THEN the client sends
      `C2SAcknowledgeResult` if connected, clears local result UI state, and
      routes to the main-menu/lobby flow without claiming server cleanup has
      completed.
- [ ] **No optimistic server authority**: GIVEN Return to Lobby is activated,
      WHEN the local UI route changes, THEN the client does not locally mutate
      server session state, declare acknowledgement accepted, create a new
      server session, start rematch, or infer result data not received from S2C.
- [ ] **Rematch remains hidden or disabled**: GIVEN no rematch protocol is
      scoped by this story, WHEN result contract implementation is complete,
      THEN no rematch C2S message exists or is sent and Presentation Story 006
      must keep Rematch hidden or disabled.
- [ ] **Alive opponent objectives remain Unknown**: GIVEN a retained final
      snapshot contains alive opponent objectives with no reveal data, WHEN a
      reconnecting client rebuilds result state, THEN those identities remain
      unavailable to the client and are displayed by Presentation Story 006 as
      `Unknown`.
- [ ] **Destroyed opponent identity remains authoritative**: GIVEN a retained
      final snapshot contains `OpponentObjectiveSnapshot.was_fake = Some(value)`
      for a destroyed opponent objective, WHEN the result screen rebuilds, THEN
      that value is available to the client and no client inference is needed.
- [ ] **No release or evidence overclaim**: GIVEN implementation evidence is
      written, WHEN it is reviewed, THEN it does not claim Result Screen UI
      implementation, manual/browser GAME_OVER evidence, Sprint 9 activation,
      public release readiness, full game completion, broad accessibility
      completion, playtest validation, smoke, QA sign-off, or `/story-done`.
- [ ] **Whitespace gates pass**: `git diff --check` passes and
      `git diff --cached --check` passes before commit.

---

## Implementation Notes

- Prefer a small server-owned ended-session resource such as
  `EndedSessionResultState` containing:
  `result: S2CGameOver`, `participants: HashSet<PlayerId>`,
  `acknowledged: HashSet<PlayerId>`,
  `final_snapshots: HashMap<PlayerId, S2CGameSnapshot>`, and
  `expires_at_ms`.
- Capture retained final snapshots before any GAME_OVER cleanup removes data
  needed by `build_game_snapshot`.
- Keep `C2SAcknowledgeResult` as a cleanup handshake. It is not a result commit,
  not a rematch vote, and not proof that the player saw every animation.
- Use `shared::config::GameConfig::default().ack_timeout_ms` as the fallback if
  runtime config is unavailable.
- Preserve ADR-011 secret stripping in retained snapshots. Do not create one
  broadcast final snapshot.
- If the existing reconnect implementation cannot insert retained
  `S2CGameOver` between snapshot and phase without a scheduling conflict, add a
  test-backed dispatch point in the reconnect path rather than a second message
  drain.
- After cleanup, old tokens must not re-enter ended-session state. The reconnect
  fallback is lobby/menu recovery, not result reconstruction.
- Presentation Story 006 remains responsible for copy, focus, reduced motion,
  layout, and the Return to Lobby button. This story only defines the
  authoritative data and acknowledgement contract that presentation consumes.

## Performance Budget

No steady-state gameplay-loop impact is expected. The story adds work only on
GAME_OVER, Return to Lobby acknowledgement, reconnect during the retained
GAME_OVER window, and the ack timeout tick. Retained per-player final snapshots
are bounded to the player count and the `ack_timeout_ms` window.

---

## QA Test Cases

- **GAME_OVER retains result state**
  - Given: a two-player session emits
    `GameOverEmitted { loser: Some(PlayerA), round: 6, reason:
    ObjectivesDestroyed }`.
  - When: GSS handles game-over teardown.
  - Then: `S2CGameOver` is broadcast and retained ended-session state contains
    the same loser, round, reason, participants, per-player final snapshots, and
    empty acknowledgement set.

- **Ack valid only in GAME_OVER**
  - Given: the server is in PLACEMENT.
  - When: Player A sends `C2SAcknowledgeResult`.
  - Then: no ended-session state is created or mutated and no S2C response is
    sent.

- **Duplicate ack idempotence**
  - Given: ended-session state exists and Player A is already acknowledged.
  - When: Player A sends `C2SAcknowledgeResult` again.
  - Then: the acknowledgement set is unchanged, no duplicate cleanup runs, and
    no panic occurs.

- **All players acknowledged cleanup**
  - Given: ended-session state exists for Player A and Player B.
  - When: both players send `C2SAcknowledgeResult`.
  - Then: ended-session state, reconnect token entries, and deferred queues for
    the session are removed.

- **Ack timeout cleanup**
  - Given: ended-session state exists and Player B never acknowledges.
  - When: `ack_timeout_ms` elapses.
  - Then: cleanup runs without changing the retained result payload before
    removal.

- **GAME_OVER reconnect receives result**
  - Given: Player A reconnects with a valid token before ack cleanup.
  - When: the reconnect handler processes `C2SHello`.
  - Then: dispatch order contains handshake, retained final snapshot, objective
    identity restore if required, retained `S2CGameOver`, and
    `S2CPhaseChanged(GameOver)`.

- **Post-cleanup reconnect fallback**
  - Given: all players acknowledged and cleanup has removed ended-session state.
  - When: Player A reconnects with the old token.
  - Then: no result snapshot or `S2CGameOver` is sent and the existing expired
    session rejection or close path is used.

- **Objective reveal scope**
  - Given: the retained final snapshot includes destroyed opponent objective
    `was_fake` values and alive opponent objectives without reveal values.
  - When: the result data is consumed by the client.
  - Then: destroyed identity is available from server data and alive identity
    remains unavailable for `Unknown` presentation fallback.

---

## Test Evidence

**Story Type**: Integration

**Required automated test targets**:

- `tests/integration/session/result_acknowledgement_contract_test.rs`
  - Ack valid only in GAME_OVER
  - Unknown sender discard
  - Duplicate ack idempotence
  - All-ack cleanup
  - Timeout cleanup
- `tests/integration/session/game_over_reconnect_result_resend_test.rs`
  - Retained final snapshot plus retained `S2CGameOver` resend
  - Post-cleanup reconnect fallback
  - Alive opponent objective identity remains unavailable

**Required regression commands**:

- `cargo test -p server --test result_acknowledgement_contract_test`
- `cargo test -p server --test game_over_reconnect_result_resend_test`
- `cargo test -p server --test reconnect_snapshot_test`
- `cargo test -p server --test game_over_teardown_test`
- `cargo check -p server`
- `git diff --check`
- `git diff --cached --check` before commit

**Status**: [ ] Not yet implemented or captured.

---

## Dependencies

- Depends on: Game Session System Story 006 complete for GAME_OVER teardown and
  `S2CGameOver` broadcast behavior.
- Depends on: Game Session System Story 007 complete for reconnect token,
  snapshot, deferred queue, and recovery ordering behavior.
- Depends on: `TR-NP-001`, `TR-NP-005`, `TR-NP-006`, and NP-51 remaining
  active/current in `design/gdd/network-protocol.md`.
- Depends on: ADR-002, ADR-008, ADR-010, and ADR-011 Accepted.
- Blocks: Presentation Layer Story 006 Result Screen MVP implementation
  assignment.
- Blocks: [Story 010](story-010-result-acknowledgement-cleanup-handshake.md)
  for S9-RS-003 result acknowledgement cleanup and Return to Lobby handshake
  verification.

## Readiness Notes

**Implementation readiness verdict**: READY.

The story resolves the result acknowledgement timing, server handling contract,
result data source, GAME_OVER reconnect behavior, Return to Lobby boundary,
duplicate acknowledgement safety, rematch exclusion, and no-client-authority
constraints without activating Sprint 9 or implementing code.

The Sprint 9 preparation state is docs-only. Do not update
`production/sprint-status.yaml`, `production/session-state/**`, Sprint 8
close-out files, or any Sprint 9 activation file from this story.
