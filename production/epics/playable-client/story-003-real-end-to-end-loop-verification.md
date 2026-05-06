# Story 003: Real End-to-End Loop Verification

> **Epic**: Playable Client
> **Status**: Ready
> **Layer**: Polish / Client Integration
> **Type**: Integration
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 7 / PLAYABLE-003

## Context

PLAYABLE-003 verifies the Sprint 7 friend-game path with a real local server and two real primary clients. This is evidence and focused integration hardening for internal friend-game quality. It does not claim public release readiness, broad accessibility completion, playtest validation, fun-hypothesis validation, or full playable-client manual QA.

Current dependency state: PLAYABLE-001 and PLAYABLE-002 are Complete. PLAYABLE-003 may now verify the real end-to-end friend-game loop from the integrated primary client path.

**Primary sources**:

- `production/sprints/sprint-7.md`
- `production/sprint-status.yaml`
- `tests/smoke/critical-paths.md`
- `design/gdd/network-protocol.md`
- `design/gdd/game-session-system.md`
- `design/gdd/round-state-machine.md`
- `design/gdd/card-acquisition.md`
- `design/gdd/shop-auction-ui.md`
- `design/gdd/hand-ui.md`
- `design/gdd/board-rendering.md`
- `design/gdd/combat-resolution.md`
- `design/gdd/hud.md`

**GDD and TR trace**:

- `design/gdd/network-protocol.md` / `TR-NP-001`: clients express intent and server owns game logic.
- `design/gdd/network-protocol.md` / `TR-NP-006`: snapshots are unicast and strip opponent secrets before live messages.
- `design/gdd/network-protocol.md` / `TR-NP-007`: `C2SSubmitPlacement` is silent and `S2CPlacementReveal` is the placement-close signal.
- `design/gdd/network-protocol.md` / `TR-NP-009`: `S2CResolutionEvent` arrives before `S2CPhaseChanged(DRAFT_SHOP)` on the reliable channel.
- `design/gdd/network-protocol.md` / `TR-NP-011`: `S2CPlacementReveal` broadcasts both players' full placements atomically.
- `design/gdd/network-protocol.md` / `TR-NP-014`: live spawn range updates use ordered `ResolutionEvent::SpawnRangeChanged` entries.
- `design/gdd/game-session-system.md` / `TR-GSS-007`: `SessionReady` initializes session resources and starts the round loop.
- `design/gdd/round-state-machine.md` / `TR-RSM-007`: phase timers, all-submitted early exit, and resolution timeout govern the loop.
- `design/gdd/round-state-machine.md` / `TR-RSM-008`: game-over detection is evaluated after RESOLUTION.
- `design/gdd/round-state-machine.md` / `TR-RSM-009`: `S2CPhaseChanged` broadcasts every phase transition.
- `design/gdd/card-data-pool.md` / `TR-CDP-010`: draft/shop payloads reach the owner before phase/UI use.
- `design/gdd/hand-ui.md` / `TR-HU-008`: placement submit pre-validation reads projected economy while server validation remains authoritative.
- `design/gdd/shop-auction-ui.md` / `TR-SAU-006`: panel transitions and input gating follow authoritative phase data.
- `design/gdd/board-rendering.md` / `TR-BR-002`: `BoardLayout` is the client coordinate authority for board and hand cursor mapping.
- `design/gdd/board-rendering.md` / `TR-BR-008`: spawn range highlights rebuild from snapshots and live ordered resolution events.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-003: Cargo Workspace Structure](../../../docs/architecture/adr-003-cargo-workspace-structure.md)
- [ADR-007: Placement Buffer](../../../docs/architecture/adr-007-placement-buffer.md)
- [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md)
- [ADR-010: RSM Phase Event Bus](../../../docs/architecture/adr-010-rsm-event-bus.md)
- [ADR-011: Reconnect Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md)
- [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md)
- [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

**ADR Decision Summary**: The verification path must exercise real C2S/S2C traffic, real server authority, and real primary client presentation. Harnesses may support automated tests, but story completion evidence cannot depend on harness-injected game state.

**Engine**: Bevy 0.18 + Lightyear 0.26 + browser/WASM primary client | **Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing any Bevy `.rs` file and `liv-bevy-lightyear` before editing any Lightyear `.rs` file. Any verification harness must not ship as normal UI. If fixes are needed while verifying, they must keep Bevy 0.18 Required Components API, Bevy `Message` versus Observer semantics, and existing Presentation set ownership.

**Lightyear Notes**: Evidence must use a real local server and two real primary clients connected through WebSocket. The final evidence cannot be satisfied by direct `World` state injection, fake `S2CGameSnapshot` insertion, harness card state, or direct calls into server feature APIs. Reliable ordering of `S2CPlacementReveal`, `S2CResolutionEvent`, and `S2CPhaseChanged` must be observed or the nearest reachable endpoint must document the exact blocker.

**Control Manifest Rules (2026-05-05)**:

- Required: client is read-only and server remains authoritative.
- Required: `C2SSubmitPlacement` payload is submit-only and `S2CPlacementReveal` omits mana-spend fields.
- Required: `S2CResolutionEvent` stays before `S2CPhaseChanged(DRAFT_SHOP)` on `ReliableChannel`.
- Required: no duplicate S2C phase or economy drains.
- Required: reconnect/snapshot ordering remains unicast and secret-stripped.
- Guardrail: server steady-state tick at or below 5 ms; server RESOLUTION at or below 15 ms; client S2C processing plus view update at or below 2 ms per frame.

---

## Scope

### In Scope

- Run a real local server and two real primary clients on the same build.
- Verify create/join/class confirm from PLAYABLE-001.
- Verify live DRAFT_INITIAL, DRAFT_SHOP, purchase, refresh, ready, hand, and economy bridge from PLAYABLE-002.
- Verify auction if the current loop reaches DRAFT_AUCTION during the friend-game path.
- Verify PLACEMENT staging and submit through real `C2SSubmitPlacement`.
- Verify `S2CPlacementReveal`, board presentation, RESOLUTION event playback, and next DRAFT loop or the documented nearest reachable friend-game endpoint.
- Verify game-over if it is reachable within scoped Sprint 7 time; otherwise document the exact nearest reachable endpoint and defects.
- Record exact build, commit, commands, target, message observations, screenshots or captures, and defects.

### Out of Scope

- Public release readiness, release-candidate readiness, store readiness, external deployment, certification, or live service readiness.
- Broad Standard-tier accessibility completion.
- Editing `QA-COND-0005` or `QA-COND-0006` accepted-risk disposition.
- Playtest validation, fun-hypothesis validation, balance validation, or full playable-client manual QA.
- Formal QA sign-off, `/smoke-check`, `/team-qa`, `/gate-check`, or broad regression campaign.
- New game content, new game modes, broad UI polish, or cosmetic asset production.

---

## Acceptance Criteria

- [ ] **Evidence uses real clients and server**: GIVEN PLAYABLE-001 and PLAYABLE-002 are complete, WHEN final evidence is captured, THEN one real local server and two real primary clients run from the same commit with no direct `World` injection or harness-injected game state.
- [ ] **Lobby path is verified**: GIVEN both clients start fresh, WHEN the host creates a room and the second client joins, THEN evidence records `C2SHello`, `S2CHandshake`, create/join messages, slot updates, class confirm, class reveal, and server-confirmed session entry.
- [ ] **Draft and shop path is verified**: GIVEN the clients reach DRAFT_INITIAL and DRAFT_SHOP, WHEN purchase, refresh, and ready actions are performed, THEN evidence records real C2S/S2C messages and visible hand/economy/shop convergence from server state.
- [ ] **Auction path is verified or bounded**: GIVEN the loop reaches DRAFT_AUCTION during the run, WHEN auction messages and UI appear, THEN evidence records auction card, bid or no-bid state, settlement or transition behavior. If not reached, evidence names the exact phase and blocker that prevented auction coverage.
- [ ] **Placement path is verified**: GIVEN both clients reach PLACEMENT with playable cards, WHEN each submits placements, THEN evidence records real `C2SSubmitPlacement` sends and a server `S2CPlacementReveal` received by both clients.
- [ ] **Resolution and next-loop ordering are verified**: GIVEN placement reveal completes, WHEN RESOLUTION runs, THEN evidence records `S2CResolutionEvent` before `S2CPhaseChanged(DRAFT_SHOP)` or records the exact blocker that prevented observing that ordering.
- [ ] **Game-over or nearest endpoint is documented**: GIVEN the friend-game run continues until game-over or a scoped blocker appears, WHEN evidence is finalized, THEN it states either the observed `S2CGameOver` result or the nearest reachable friend-game endpoint with blocking defects.
- [ ] **Defects are classified for friend-game use**: GIVEN any issue blocks or degrades the path, WHEN it is recorded, THEN the evidence classifies severity for internal friend-game playability and names the likely owning story/system without expanding public release scope.
- [ ] **No forbidden claims are made**: GIVEN the evidence is reviewed, WHEN scope statements are inspected, THEN the document explicitly says this is friend-game evidence only and not public release readiness, broad accessibility completion, playtest validation, fun-hypothesis validation, QA sign-off, or full playable-client manual QA.
- [ ] **Regression commands pass or blockers are explicit**: `cargo test -p server --test playable_client_real_e2e_loop_test`, `cargo check --workspace`, and `git diff --check` pass, or the evidence records exact failing command output and the issue is classified as a Sprint 7 blocker.
- [ ] **Evidence document exists**: `production/qa/evidence/playable-client-real-e2e-loop.md` records commit, commands, build target, two-client setup, reached endpoint, captures, defects, no-harness statement, and friend-game-only scope statement.

---

## Likely Files Touched

- `tests/integration/playable_client/real_e2e_loop_test.rs`
- `production/qa/evidence/playable-client-real-e2e-loop.md`
- `production/qa/evidence/captures/playable-client-real-e2e-loop/`
- `server/Cargo.toml`
- `client/Cargo.toml`
- `tests/smoke/critical-paths.md`

Focused defect repairs discovered during verification may touch the owning implementation files from PLAYABLE-001 or PLAYABLE-002, most likely:

- `client/src/main.rs`
- `client/src/network/mod.rs`
- `client/src/state/mod.rs`
- `client/src/ui/lobby.rs`
- `client/src/ui/shop_auction/mod.rs`
- `client/src/ui/hand/mod.rs`
- `client/src/ui/hud/mod.rs`
- `client/src/presentation/mod.rs`
- `client/src/presentation/board_rendering.rs`
- `server/src/network/mod.rs`
- `server/src/core/session/reconnect.rs`
- `server/src/core/session/system.rs`
- `server/src/core/rsm/system.rs`
- `server/src/feature/acquisition/system.rs`
- `server/src/feature/board/placement.rs`

Any repair must remain scoped to friend-game playability and must not edit Sprint 6 accepted-risk disposition docs.

## Implementation Notes

- Treat this as verification plus focused blocker repair, not a broad QA sweep.
- PLAYABLE-002 must remain Complete via `/story-done` before verification starts; this dependency is currently satisfied.
- Use real commands and real targets. Automated helpers may launch processes or collect logs, but cannot seed authoritative game state directly.
- Evidence should include the first failing point if the full loop is not reachable. A documented nearest reachable endpoint is acceptable only when paired with blocker detail.
- Do not turn this story into a public QA plan, accessibility campaign, or playtest report.
- If the run exposes an issue outside the PLAYABLE path, record it as follow-up unless it blocks internal friend-game playability.

## Performance Budget

Verification should record whether obvious performance symptoms appear, but it is not a full profiling story. Any added automated E2E harness must avoid unbounded waits and must complete within 15 minutes on the local critical path. Runtime budgets remain the control-manifest defaults: server steady-state at or below 5 ms per tick, server RESOLUTION at or below 15 ms, client S2C processing plus view update at or below 2 ms per frame, and browser/WASM target around 60 FPS for friend-game use.

---

## QA Test Cases

- **Two real clients through lobby**
  - Given: Local server and two primary clients start from the same commit.
  - When: Host creates a room, joiner joins, and both confirm class.
  - Then: evidence records real network messages and server-confirmed entry into the session.

- **Draft/shop/hand bridge**
  - Given: Both clients reach DRAFT_INITIAL and DRAFT_SHOP.
  - When: purchase, refresh, and ready actions are performed.
  - Then: evidence records live S2C offering/slots/acquisition/economy changes and visible hand/economy convergence.

- **Placement and resolution**
  - Given: Both clients reach PLACEMENT with playable cards.
  - When: both submit placements.
  - Then: evidence records `S2CPlacementReveal`, RESOLUTION playback, and the next-loop phase transition or exact blocker.

- **Game-over or endpoint report**
  - Given: The run continues after one loop.
  - When: game-over is reached or a blocking defect appears.
  - Then: evidence states the final reached endpoint, observed defects, severity, and friend-game impact.

---

## Test Evidence

**Story Type**: Integration

**Required automated test target**:

- `tests/integration/playable_client/real_e2e_loop_test.rs`
  - Registered as `playable_client_real_e2e_loop_test`
  - Command: `cargo test -p server --test playable_client_real_e2e_loop_test`

**Required manual friend-game evidence document**:

- `production/qa/evidence/playable-client-real-e2e-loop.md`

**Required capture artifact directory**:

- `production/qa/evidence/captures/playable-client-real-e2e-loop/`

These required test and evidence paths match the Sprint 7 QA plan at `production/qa/qa-plan-sprint-7-2026-05-06.md`.

**Required regression commands**:

- `cargo test -p client --test playable_client_lobby_entry_test`
- `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`
- `cargo test -p server --test playable_client_draft_ready_bridge_test`
- `cargo test -p server --test e2e_websocket_test`
- `cargo check --workspace`
- `git diff --check`

**Final evidence expectations**:

- Exact commit and build target.
- Commands used to run the server and both primary clients.
- Environment details: OS, target type, browser name/version or native client build, viewport/window size, local server address/port, WebSocket/Lightyear transport, and confirmation that both clients fresh-started from no session token unless the run explicitly tests reconnect.
- Log artifacts: server stdout/stderr, host-client log, joiner-client log, and message trace excerpts or artifact paths for every reached phase.
- Capture artifacts: screenshots or recordings stored under `production/qa/evidence/captures/playable-client-real-e2e-loop/`, with filenames or a capture manifest that identify lobby, class reveal, draft/shop, placement, resolution, next-loop, and game-over or nearest endpoint where reached.
- Message trace for lobby, class, draft/shop, purchase, ready, placement, resolution, next-loop, and game-over where reached.
- Screenshots or captures for each reached phase.
- Defect table with severity, owner, workaround, and friend-game impact.
- No-harness statement confirming no direct state injection was used for completion.
- Explicit statement that evidence is internal friend-game evidence only, not public release readiness, not broad accessibility completion, not playtest validation, not fun-hypothesis validation, not QA sign-off, and not full playable-client manual QA.

**Status**: [ ] Not yet implemented or captured.

---

## Dependencies

- Depends on: [PLAYABLE-001 Primary Client Bootstrap + Fresh Lobby Entry](story-001-primary-client-bootstrap-fresh-lobby-entry.md) - Complete; required before this verification can start.
- Depends on: [PLAYABLE-002 Live Draft/Shop/Hand Bridge](story-002-live-draft-shop-hand-bridge.md) - Complete; required before final verification starts.
- Depends on: a local server and two primary clients that can run from the same commit.
- Unlocks: Sprint 7 friend-game evidence index cleanup and any focused follow-up repair stories found by verification.

## Blockers

None.
