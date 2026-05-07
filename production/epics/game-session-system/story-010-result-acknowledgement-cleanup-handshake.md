# Story 010: Result Acknowledgement Cleanup Handshake

> **Epic**: Game Session System
> **Status**: Blocked - depends on S9-RS-001 and S9-RS-002
> **Layer**: Core / Networking + Presentation Integration
> **Type**: Integration
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 9 preparation only - Sprint 9 is not active

## Context

This is the S9-RS-003 follow-up story from the Sprint 9 draft. It owns the
cross-boundary cleanup handshake after the S9-RS-001 result acknowledgement
contract and S9-RS-002 Result Screen MVP are implemented.

This story exists so S9-RS-003 is not split ambiguously across the contract and
UI stories. It does not replace Game Session System Story 009 or Presentation
Layer Story 006. Story 009 owns the server-side result acknowledgement contract,
retained GAME_OVER result data, reconnect result resend, idempotent ack
semantics, and `ack_timeout_ms` cleanup. Story 006 owns the visible result
overlay, result copy, focus/reduced-motion behavior, and Return to Lobby
control. This story verifies that those pieces work together through the real
client/server boundary.

This story does not implement code during Sprint 9 preparation, activate Sprint
9, close Sprint 8, run smoke, run QA sign-off, run a gate, run `/dev-story`, or
run `/story-done`.

**Primary sources**:

- `production/sprints/sprint-9-draft.md`
- `production/epics/game-session-system/story-009-result-acknowledgement-and-result-data-contract.md`
- `production/epics/presentation-layer/story-006-result-screen-mvp.md`
- `design/ux/result-screen.md`
- `design/gdd/network-protocol.md`
- `design/gdd/game-session-system.md`
- `design/gdd/round-state-machine.md`
- `docs/architecture/adr-002-client-server-authority.md`
- `docs/architecture/adr-008-lightyear-channel-config.md`
- `docs/architecture/adr-011-reconnect-snapshot.md`
- `docs/architecture/adr-021-presentation-layer-architecture.md`

**GDD, UX, and TR trace**:

- `design/ux/result-screen.md` Exit table: Return to Lobby sends or has already
  sent `C2SAcknowledgeResult`, clears local ended-session UI state, and returns
  to the main-menu/lobby flow.
- `design/ux/result-screen.md` Interaction Map: Return to Lobby is activated by
  click or Enter; Escape focuses Return to Lobby but does not auto-exit.
- `design/gdd/network-protocol.md` Table A: `C2SAcknowledgeResult {}` is valid
  only during `GAME_OVER`.
- `design/gdd/network-protocol.md` NP-51: `C2SAcknowledgeResult` outside
  `GAME_OVER` is silently discarded and starts no teardown.
- `design/gdd/network-protocol.md` edge case: if `C2SAcknowledgeResult` is
  never received after `GAME_OVER`, the server waits up to `ack_timeout_ms`
  before cleaning up the session; the result is persisted regardless.
- `design/gdd/round-state-machine.md` Rule 11: GAME_OVER is terminal and emits
  `S2CGameOver` plus `S2CPhaseChanged(GAME_OVER)`.
- `TR-NP-001`: clients express intent through C2S messages only and do not
  commit authoritative game state.
- `TR-NP-005`: invalid-phase C2S messages are silently discarded.
- `TR-NP-006`: reconnect snapshots remain server-authoritative and
  secret-stripped.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md)
- [ADR-011: Reconnect Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

All referenced ADRs are Accepted.

**ADR Decision Summary**: Return to Lobby is a client UI exit action that may
send `C2SAcknowledgeResult`, but server cleanup remains authoritative. The
client must not treat route change as proof that server cleanup completed, must
not mutate server-owned ended-session state locally, and must not start rematch
or a new server session unless a separate server-authoritative flow exists.

**Engine**: Bevy 0.18 + Lightyear 0.26 + browser/WASM primary client |
**Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing Bevy `.rs` files and
`liv-bevy-lightyear` before editing Lightyear/networking `.rs` files. Keep one
production drain for `MessageReceiver<C2SAcknowledgeResult>`, keep
`S2CPhaseChanged` owned by the shared phase sink, and use Bevy 0.18 Required
Components API for any UI route changes.

**Control Manifest Rules (2026-05-05)**:

- Required: clients are read-only views of server-authoritative game state.
- Required: invalid-phase C2S handlers silently discard messages with no S2C
  response.
- Required: all non-heartbeat game-state messages use `ReliableChannel`.
- Required: `S2CPhaseChanged` is drained only by the shared phase sink.
- Required: reconnect sends snapshot before live messages and flushes deferred
  messages only after `snapshot_sent == true`.
- Forbidden: never let a client commit or mutate authoritative server state
  optimistically.
- Forbidden: never infer hidden opponent objective identities on the client.

---

## Scope

### In Scope

- Wire the Result Screen MVP Return to Lobby action to the S9-RS-001
  `C2SAcknowledgeResult` contract.
- Ensure Return to Lobby sends `C2SAcknowledgeResult` at most once per local
  result-screen exit attempt when connected and in `GAME_OVER`.
- Ensure Return to Lobby remains usable if the client is disconnected,
  acknowledgement send fails, or the server has already cleaned up the ended
  session.
- Ensure local ended-session UI state, cached result-screen state, and any
  result overlay focus state are cleared when routing back to the main-menu or
  lobby flow.
- Ensure Return to Lobby does not optimistically mutate server-owned session,
  acknowledgement, rematch, reconnect-token, or result persistence state.
- Verify duplicate acknowledgements remain idempotent from the end-to-end
  client/server route.
- Verify invalid-phase acknowledgement from the client route is impossible in
  normal UI flow and still silently discarded if forced by a test.
- Verify server cleanup behaves consistently after all participants acknowledge
  and after `ack_timeout_ms` expires.
- Verify reconnect after acknowledgement cleanup uses the accepted
  expired-session rejection or lobby/menu fallback path.
- Record evidence without closing S8-QA-001-W1 unless the separate S9-QA-001
  manual/browser route is actually captured.

### Out of Scope

- No new result data contract decisions. Story 009 owns the contract.
- No result overlay visual implementation, copy table, objective summary,
  focus polish, or reduced-motion behavior beyond the Return to Lobby handshake.
- No rematch protocol, rematch C2S/S2C messages, or rematch UI enablement.
- No manual/browser two-client GAME_OVER evidence closure. S9-QA-001 owns that.
- No Sprint 9 activation, Sprint 8 close-out, smoke, QA sign-off, gate-check,
  `/dev-story`, or `/story-done`.
- No public release readiness, broad accessibility completion, playtest
  validation, full playable-client manual QA, or full game completion claim.
- No updates to `production/sprint-status.yaml` or
  `production/session-state/**`.

---

## Acceptance Criteria

- [ ] **Return sends acknowledgement once**: GIVEN the result overlay is open,
      the client is connected, and the current phase is `GAME_OVER`, WHEN the
      player activates Return to Lobby by click or Enter, THEN the client sends
      one `C2SAcknowledgeResult` on `ReliableChannel` for that exit attempt.
- [ ] **Return clears local result UI**: GIVEN Return to Lobby is activated,
      WHEN the route changes, THEN result overlay entities/read models, cached
      result-screen focus state, and ended-session UI state are cleared from the
      client before the lobby/menu view is shown.
- [ ] **No optimistic server mutation**: GIVEN Return to Lobby routes locally,
      WHEN client state is inspected, THEN the client does not mark server
      cleanup complete, mutate server-owned session state, create a new server
      session, or start rematch without an authoritative S2C response.
- [ ] **Disconnected return remains usable**: GIVEN the result overlay is open
      and the transport is disconnected or closing, WHEN Return to Lobby is
      activated, THEN local route cleanup still succeeds and the evidence
      records whether the ack was skipped or failed to send.
- [ ] **Already-cleaned session fallback is safe**: GIVEN the server has already
      cleaned up ended-session result state because all players acknowledged or
      `ack_timeout_ms` elapsed, WHEN the client returns or reconnects with the
      old token, THEN no result reconstruction is attempted and the existing
      expired-session rejection or lobby/menu fallback is used.
- [ ] **Duplicate acknowledgement remains idempotent**: GIVEN the client sends
      `C2SAcknowledgeResult` twice because of repeated button activation,
      reconnect replay, or retry, WHEN the server handles the messages, THEN the
      second acknowledgement is a no-op, sends no S2C response, and does not
      panic.
- [ ] **Invalid-phase ack is discarded**: GIVEN a test forces
      `C2SAcknowledgeResult` before `GAME_OVER`, WHEN the server handler runs,
      THEN the message is silently discarded, starts no cleanup, sends no S2C
      response, and mutates no ended-session state.
- [ ] **All-ack cleanup path is observed**: GIVEN both participants activate
      Return to Lobby during the retained GAME_OVER window, WHEN both
      acknowledgements are processed, THEN ended-session result state,
      reconnect-token entries, and deferred reconnect queues for the session are
      removed by the server.
- [ ] **Timeout cleanup path is observed**: GIVEN one participant never sends
      `C2SAcknowledgeResult`, WHEN `ack_timeout_ms` elapses, THEN the same
      terminal cleanup path runs without changing the retained result payload
      before removal.
- [ ] **Evidence preserves Sprint 8 conditions**: GIVEN story evidence is
      written, WHEN it is reviewed, THEN it does not close S8-QA-001-W1 or claim
      manual/browser GAME_OVER unless S9-QA-001 captured the full route.
- [ ] **QA conditions remain carried**: GIVEN story evidence is written, WHEN
      QA-COND-0005 and QA-COND-0006 are referenced, THEN they remain carried as
      accepted-risk/deferred conditions unless separate actual closure evidence
      exists later.
- [ ] **Whitespace gates pass**: `git diff --check` passes and
      `git diff --cached --check` passes before commit.

---

## Implementation Notes

- Treat this as an end-to-end integration story, not a place to re-open the
  S9-RS-001 contract. If Story 009 changes the ack timing before implementation,
  update this story before assignment.
- The Return to Lobby button should be guarded against double activation in the
  UI, but server idempotence remains required and must be tested.
- Route cleanup should clear local result state even when the server rejects the
  old session token after terminal cleanup. The local route is not proof that
  the server accepted the acknowledgement.
- If implementation discovers that the main-menu/lobby route cannot be reached
  without creating a new server room, split that route repair into a separate
  playable-client story rather than weakening this story's authority boundary.
- Rematch must remain hidden or disabled. Do not add rematch protocol as part of
  cleanup handshake work.

## Performance Budget

No steady-state gameplay-loop impact is expected. The work happens only on
Return to Lobby activation, result acknowledgement handling, GAME_OVER cleanup,
and reconnect after cleanup. Client route cleanup must avoid per-frame polling
or repeated entity spawning after the overlay is closed.

---

## QA Test Cases

- **Return to Lobby sends acknowledgement**
  - Given: the result overlay is open in `GAME_OVER`.
  - When: Return to Lobby is activated.
  - Then: exactly one `C2SAcknowledgeResult` is sent for the activation and the
    local UI routes to lobby/menu.

- **Duplicate activation is safe**
  - Given: Return to Lobby has already been activated once.
  - When: the button is activated again or the ack is retried.
  - Then: the server acknowledgement set is unchanged after the first ack and
    no panic or duplicate cleanup occurs.

- **Disconnected local return**
  - Given: the transport is disconnected while the result screen is open.
  - When: Return to Lobby is activated.
  - Then: local result UI clears and the evidence records that no successful ack
    send is claimed.

- **All-ack cleanup**
  - Given: both participants return to lobby during the retained result window.
  - When: both acknowledgements are processed.
  - Then: ended-session result state and reconnect-token state are removed by
    the server.

- **Timeout cleanup**
  - Given: one participant never acknowledges.
  - When: `ack_timeout_ms` elapses.
  - Then: cleanup runs through the timeout path and the result remains the
    original server-authored result.

- **Post-cleanup reconnect fallback**
  - Given: terminal cleanup has completed.
  - When: a client reconnects with the old session token.
  - Then: no result data is reconstructed and the existing expired-session or
    lobby/menu fallback path is used.

---

## Test Evidence

**Story Type**: Integration

**Required automated test targets**:

- `tests/integration/session/result_acknowledgement_cleanup_handshake_test.rs`
  - Return to Lobby ack dispatch boundary
  - duplicate ack idempotence from client route
  - all-ack cleanup
  - timeout cleanup
  - post-cleanup reconnect fallback
- `tests/integration/presentation/result_screen_return_to_lobby_test.rs`
  - local result UI cleanup
  - no optimistic server-state mutation
  - disconnected Return to Lobby fallback

**Required regression commands**:

- `cargo test -p server --test result_acknowledgement_contract_test`
- `cargo test -p server --test result_acknowledgement_cleanup_handshake_test`
- `cargo test -p client --test result_screen_mvp_test`
- `cargo test -p client --test result_screen_return_to_lobby_test`
- `cargo check -p server`
- `cargo check -p client`
- `git diff --check`
- `git diff --cached --check` before commit

**Required evidence document**:

- `production/qa/evidence/result-acknowledgement-cleanup-handshake-evidence.md`

**Required evidence contents**:

- Commit, branch, build target, and command summary.
- Return to Lobby acknowledgement timing actually implemented.
- Local route cleanup behavior.
- Duplicate acknowledgement and invalid-phase acknowledgement results.
- All-ack cleanup and timeout cleanup results.
- Reconnect-after-cleanup fallback result.
- Explicit non-claims for Sprint 9 activation, Sprint 8 close-out,
  manual/browser GAME_OVER closure, public release readiness, broad
  accessibility completion, playtest validation, full playable-client manual
  QA, and full game completion.
- QA-COND-0005 and QA-COND-0006 carried-condition statement.

**Status**: [ ] Not yet implemented or captured.

---

## Dependencies

- Depends on: [Game Session System Story 009](story-009-result-acknowledgement-and-result-data-contract.md) complete for S9-RS-001 server acknowledgement handling and retained GAME_OVER result data.
- Depends on: [Presentation Layer Story 006](../presentation-layer/story-006-result-screen-mvp.md) complete for S9-RS-002 result overlay and Return to Lobby control.
- Depends on: HUD Story 007 complete for GAME_OVER FROZEN mode.
- Depends on: PLAYABLE-004 complete for controlled internal friend-game
  `GAME_OVER` endpoint evidence.
- Blocks: S9-QA-001 manual/browser two-client GAME_OVER evidence closure.

## Readiness Notes

**Implementation readiness verdict**: BLOCKED.

This story is ready as a follow-up package, but it cannot be assigned until
S9-RS-001 and S9-RS-002 are complete or explicitly replaced by producer-approved
fallbacks. It preserves S8-QA-001-W1, QA-COND-0005, and QA-COND-0006 as carried
conditions and does not activate Sprint 9.
