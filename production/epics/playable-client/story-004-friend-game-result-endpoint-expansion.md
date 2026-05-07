# Story 004: Friend-Game Result Endpoint Expansion

> **Epic**: Playable Client
> **Status**: Ready
> **Layer**: Polish / Client and Server Integration
> **Type**: Integration
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 8 / PLAYABLE-004

## Context

Sprint 7 proved the internal friend-game route through next-loop `DRAFT_SHOP`
after post-auction placement and resolution. Sprint 8 extends that evidence
toward result or game-over coverage while keeping the endpoint claim exact.

**Primary sources**:

- `production/sprints/sprint-8.md`
- `production/sprint-status.yaml`
- `production/qa/qa-plan-sprint-8-2026-05-07.md`
- `production/epics/playable-client/EPIC.md`
- `production/qa/evidence/sprint-7-friend-game-evidence-index.md`
- `production/qa/evidence/playable-client-real-e2e-loop.md`

**GDD and TR trace**:

- `design/gdd/network-protocol.md` / `TR-NP-001`: clients express intent through
  C2S messages only and the server owns all game logic.
- `design/gdd/network-protocol.md` / `TR-NP-009`: `S2CResolutionEvent` must be
  received before `S2CPhaseChanged(DRAFT_SHOP)` on the reliable channel.
- `design/gdd/network-protocol.md` / `TR-NP-011`: `S2CPlacementReveal` contains
  both players' full placements atomically, with no partial reveal.
- `design/gdd/round-state-machine.md` / `TR-RSM-008`: `GAME_OVER` is detected
  after RESOLUTION when a player has at least two real objectives destroyed;
  mutual destruction produces a draw.
- `design/gdd/round-state-machine.md` / `TR-RSM-009`: `S2CPhaseChanged` is
  broadcast on every transition and is always emitted last in the phase-entry
  sequence.
- `design/gdd/game-session-system.md` / `TR-GSS-007`: `SessionReady` creates
  `SessionConfig` and `ServerRng`, and GSS tears those resources down after
  game-over.
- `design/gdd/hud.md` / `TR-HUD-009`: on `GAME_OVER`, HUD enters FROZEN mode,
  retains the final state, and stops incremental gold or mana updates.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md)
- [ADR-010: RSM Phase Event Bus](../../../docs/architecture/adr-010-rsm-event-bus.md)
- [ADR-011: Reconnect Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md)
- [ADR-012: SessionReady Delivery](../../../docs/architecture/adr-012-session-ready-delivery.md)
- [ADR-017: Combat Resolution Execution Architecture](../../../docs/architecture/adr-017-combat-resolution-execution-architecture.md)
- [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

All referenced ADRs are Accepted.

**Engine**: Bevy 0.18 + Lightyear 0.26 + browser/WASM primary client |
**Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing any Bevy `.rs` file and
`liv-bevy-lightyear` before editing any Lightyear or networking `.rs` file.
Primary-client, HUD, board, and shop/auction changes must use Bevy 0.18
Required Components API and the existing presentation sets. Do not use
deprecated bundle APIs, duplicate S2C drains, local phase authority, or direct
server feature calls from client code.

**Lightyear Notes**: Endpoint evidence must use one real local server and two
real primary clients from the same commit. Completion evidence cannot use direct
`World` state injection, fake snapshot insertion, harness card state, or direct
server feature API calls as the endpoint proof. All result, phase, placement,
resolution, and game-over observations must come from real C2S/S2C traffic or
documented server/client logs from that traffic.

**Control Manifest Rules (2026-05-05)**:

- Required: server is the sole authority over all game state.
- Required: `S2CResolutionEvent` stays before `S2CPhaseChanged(DRAFT_SHOP)` on
  `ReliableChannel`.
- Required: `S2CPhaseChanged` is drained only by the shared phase sink.
- Required: `PlayerEconomyView` is the shared economy view for Hand UI, HUD,
  and Shop/Auction UI.
- Required: reconnect snapshots are unicast, secret-stripped, and ordered before
  live messages.
- Guardrail: server steady-state tick stays at or below 5 ms; server
  RESOLUTION stays at or below 15 ms; client S2C processing plus view update
  stays at or below 2 ms per frame.

## Sprint 8 Scope Guard

This story is internal friend-game endpoint expansion only. It carries
QA-COND-0005 as friend-game-only accepted risk and does not verify
Standard-tier accessibility completion. It carries QA-COND-0006 as
accepted-risk/deferred and does not create playtest evidence, fun-hypothesis
validation, or a playtest report.

This story must not claim public, external, commercial, store, deployment,
release-candidate, or release readiness. It must not claim broad accessibility
completion, full playable-client manual QA, full game completion, or game-over
coverage unless the evidence actually reaches and records `S2CGameOver`, HUD
FROZEN/result state, and teardown behavior.

## Scope

### In Scope

- Reproduce the Sprint 7 route through next-loop `DRAFT_SHOP` after
  post-auction placement and resolution.
- Extend the route beyond that endpoint toward result or game-over coverage.
- Capture the exact endpoint reached with two real primary clients and one real
  local server from the same commit.
- When `GAME_OVER` is reached, record `S2CGameOver`,
  `S2CPhaseChanged(GAME_OVER)`, HUD FROZEN/result state, and GSS teardown
  behavior.
- When `GAME_OVER` is not reached within the scoped implementation capacity,
  record the accepted nearest-endpoint improvement and classify the blocker.
- Preserve server authority for phase, objective, placement, resolution,
  economy, result, and teardown state.
- Update Sprint 8 friend-game evidence with commands, commit, route, endpoint,
  defects, and non-claims.

### Out of Scope

- Public release readiness, store readiness, deployment readiness, certification,
  or external QA.
- Broad Standard-tier accessibility completion or closure of QA-COND-0005.
- Playtest validation, fun-hypothesis validation, or closure of QA-COND-0006.
- Full playable-client manual QA or a full regression campaign.
- Full game completion, even when result or game-over evidence is reached.
- New game modes, broad card/content expansion, balance changes, or broad UI
  polish.
- Local client authority, direct state injection, fake snapshots, or test-only
  endpoint shortcuts used as completion evidence.

## Acceptance Criteria

- [ ] **Sprint 7 endpoint is reproduced**: GIVEN one real local server and two
      real primary clients from the same commit, WHEN the friend-game route is
      exercised, THEN evidence reaches or explicitly records the blocker before
      the Sprint 7 endpoint:
      `DRAFT_INITIAL -> PLACEMENT(empty) -> RESOLUTION -> DRAFT_SHOP ->
      PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP ->
      PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP`.
- [ ] **Route extends beyond next-loop DRAFT_SHOP**: GIVEN the Sprint 7 endpoint
      is reached, WHEN implementation exercises the next scoped actions, THEN
      evidence records at least one additional authoritative phase, result path,
      or blocker beyond that endpoint.
- [ ] **Game-over evidence is exact when reached**: GIVEN the route reaches
      `GAME_OVER`, WHEN evidence is finalized, THEN it records `S2CGameOver`,
      `S2CPhaseChanged(GAME_OVER)`, final round/reason/loser or draw payload,
      HUD FROZEN/result state, and GSS teardown behavior.
- [ ] **Nearest endpoint is explicit when game-over is not reached**: GIVEN
      `GAME_OVER` is not reached within scoped capacity, WHEN evidence is
      finalized, THEN it records the accepted nearest-endpoint improvement,
      classifies the blocker by severity and owner, and states that game-over
      coverage is not claimed.
- [ ] **No harness endpoint proof is used**: GIVEN endpoint evidence is reviewed,
      WHEN the evidence path is inspected, THEN it confirms no direct `World`
      injection, fake snapshot insertion, harness card state, or direct server
      feature API call was used for completion proof.
- [ ] **Reliable ordering remains observed**: GIVEN placement and resolution are
      part of the route, WHEN messages are captured, THEN `S2CPlacementReveal`,
      `S2CResolutionEvent`, and following `S2CPhaseChanged` observations remain
      ordered according to the GDD and any ordering defect is classified.
- [ ] **Defects are scoped to friend-game impact**: GIVEN any issue blocks or
      degrades endpoint expansion, WHEN it is recorded, THEN evidence lists
      severity, likely owner/system, workaround where one exists, and internal
      friend-game impact without expanding release scope.
- [ ] **Required commands pass or blockers are explicit**: `cargo test -p server
      --test playable_client_friend_game_result_endpoint_test`,
      `cargo test -p server --test playable_client_real_e2e_loop_test`,
      `cargo test -p client --test playable_client_lobby_entry_test`,
      `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`,
      `cargo test -p server --test playable_client_draft_ready_bridge_test`,
      `cargo test -p server --test e2e_websocket_test`,
      `cargo check --workspace`, and `git diff --check` pass, or exact failing
      command output is recorded as a blocker.
- [ ] **Evidence document exists**:
      `production/qa/evidence/sprint-8-friend-game-loop-evidence.md` records
      commit, commands, target, two-client setup, exact route, exact endpoint,
      captures, defects, no-harness statement, QA-COND-0005 and QA-COND-0006
      accepted-risk context, and all Sprint 8 non-claims.

## Likely Files Touched

- `tests/integration/playable_client/friend_game_result_endpoint_test.rs`
- `tests/integration/playable_client/real_e2e_loop_test.rs`
- `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/`
- `server/src/core/rsm/system.rs`
- `server/src/core/session/system.rs`
- `server/src/network/mod.rs`
- `client/src/presentation/mod.rs`
- `client/src/ui/hud/mod.rs`
- `client/src/presentation/board_rendering.rs`

Any source change must remain a focused endpoint repair discovered while
exercising the real friend-game route.

## Implementation Notes

- Start from the existing `playable_client_real_e2e_loop_test` route and extend
  it only as far as needed to prove the next endpoint or the concrete blocker.
- Treat result coverage as evidence-driven. Do not invent a local client result
  screen or manually force objective counters to satisfy the story.
- Keep any game-over repair on the owning system boundary: RSM owns phase and
  game-over emission, GSS owns teardown, HUD owns frozen final readout, and the
  client remains a read-only projection.
- When the route cannot reach `GAME_OVER`, record the nearest endpoint as an
  accepted Sprint 8 improvement only. Do not mark game-over or full game
  completion as proven.
- Preserve the Sprint 8 evidence wording that manual friend-game smoke is not a
  playtest and is not full playable-client manual QA.

## Performance Budget

The extended endpoint path must preserve the existing budgets: server
steady-state at or below 5 ms per tick, server RESOLUTION at or below 15 ms, and
client S2C processing plus view update at or below 2 ms per frame. Any
automated endpoint harness must avoid unbounded waits and complete within the
local critical path budget used by the existing real E2E loop test.

## QA Test Cases

- **Endpoint extension**
  - Given: the Sprint 7 route reaches next-loop `DRAFT_SHOP`.
  - When: the implementation continues the route from that endpoint.
  - Then: evidence records the next authoritative phase, result event, or
    blocker beyond the Sprint 7 endpoint.

- **Game-over reached**
  - Given: objective destruction or another legitimate server condition reaches
    `GAME_OVER`.
  - When: clients receive final result messages.
  - Then: evidence records `S2CGameOver`, `S2CPhaseChanged(GAME_OVER)`, HUD
    frozen state, and teardown observations.

- **Game-over not reached**
  - Given: scoped execution cannot reach `GAME_OVER`.
  - When: evidence is finalized.
  - Then: evidence names the exact nearest endpoint and defect classification
    while explicitly saying game-over is not claimed.

- **No harness proof**
  - Given: endpoint evidence is audited.
  - When: commands and traces are reviewed.
  - Then: completion proof comes from real server/client traffic, not injected
    state.

## Test Evidence

**Story Type**: Integration

**Required automated test target**:

- `tests/integration/playable_client/friend_game_result_endpoint_test.rs`
  - Registered as `playable_client_friend_game_result_endpoint_test`
  - Command: `cargo test -p server --test playable_client_friend_game_result_endpoint_test`

**Required manual friend-game evidence document**:

- `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`

**Required capture artifact directory**:

- `production/qa/evidence/captures/sprint-8-friend-game-loop/`

**Required regression commands**:

- `cargo test -p server --test playable_client_real_e2e_loop_test`
- `cargo test -p client --test playable_client_lobby_entry_test`
- `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`
- `cargo test -p server --test playable_client_draft_ready_bridge_test`
- `cargo test -p server --test e2e_websocket_test`
- `cargo check --workspace`
- `git diff --check`

**Final evidence expectations**:

- Exact commit and build target.
- Server and both primary-client commands.
- Confirmation that both clients used the same build.
- Message trace or log excerpts for every reached phase.
- Exact endpoint reached.
- `S2CGameOver`, HUD FROZEN/result state, and teardown observations when
  reached.
- Accepted nearest-endpoint improvement and defect classification when
  game-over is not reached.
- No-harness statement.
- Explicit statement that this is internal friend-game evidence only and not
  public release readiness, broad accessibility completion, playtest validation,
  fun-hypothesis validation, full playable-client manual QA, game-over coverage
  unless reached, or full game completion.

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 003 Real End-to-End Loop Verification](story-003-real-end-to-end-loop-verification.md) - Complete.
- Depends on: the completed game-over pipeline stories integrated on `main`,
  including RSM game-over emission, objective damage, GSS teardown, and HUD
  frozen final-state behavior. No additional story file is required for
  readiness.
- Unlocks: Sprint 8 endpoint decision evidence and S8-QA-001 friend-game manual
  smoke package.

## Blockers

- No design or story-doc blocker remains for readiness.
- `/dev-story` should wait until CI is green per Sprint 8 planning context.
