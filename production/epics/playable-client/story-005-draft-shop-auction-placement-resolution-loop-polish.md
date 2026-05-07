# Story 005: DRAFT_SHOP / Auction / Placement / Resolution Loop Polish

> **Epic**: Playable Client
> **Status**: Ready
> **Layer**: Polish / Client and Server Integration
> **Type**: Integration
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 8 / LOOP-001

## Context

Sprint 7 proved the internal friend-game route through auction, post-auction
placement, resolution with `UnitPlaced`, and next-loop `DRAFT_SHOP`. Sprint 8
hardens that active loop so repeated DRAFT_SHOP, auction, placement, and
resolution passes do not leak stale UI state or client-side authority.

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
- `design/gdd/network-protocol.md` / `TR-NP-007`: `C2SSubmitPlacement` is silent
  and `S2CPlacementReveal` is the sole placement-close signal.
- `design/gdd/network-protocol.md` / `TR-NP-009`: `S2CResolutionEvent` must be
  received before `S2CPhaseChanged(DRAFT_SHOP)` on the reliable channel.
- `design/gdd/network-protocol.md` / `TR-NP-011`: `S2CPlacementReveal` contains
  both players' full placements atomically.
- `design/gdd/round-state-machine.md` / `TR-RSM-003`: auction rounds are detected
  by `round_number mod 3 == 0` after the RESOLUTION increment.
- `design/gdd/round-state-machine.md` / `TR-RSM-004`: draft entry emission order
  is `DraftStarted`, shop refresh, optional `AuctionPhaseEntered`, then
  phase broadcast.
- `design/gdd/round-state-machine.md` / `TR-RSM-007`: phase timers cover
  `DRAFT_SHOP`, `PLACEMENT`, `DRAFT_INITIAL`, and the RESOLUTION safety timeout.
- `design/gdd/round-state-machine.md` / `TR-RSM-009`: `S2CPhaseChanged` is
  broadcast on every phase transition and is always emitted last.
- `design/gdd/shop-auction-ui.md` / `TR-SAU-003`: `S2CAuctionSettled` drives
  settlement display through winner banner, settled price, and card movement or
  pool return.
- `design/gdd/shop-auction-ui.md` / `TR-SAU-006`: panel transitions and input
  gating follow authoritative phase and timing behavior.
- `design/gdd/hand-ui.md` / `TR-HU-008`: placement submit pre-validation reads
  `PlayerEconomyView`, while server validation remains authoritative.
- `design/gdd/board-rendering.md` / `TR-BR-005`: board rendering buffers
  phase-change and resolution-script messages received before snapshot
  completion.
- `design/gdd/board-rendering.md` / `TR-BR-008`: spawn range highlights persist
  across DRAFT and PLACEMENT frames and update from ordered resolution events.
- `design/gdd/hand-ui.md` / `TR-PRES-001`: Hand UI, HUD, and Shop/Auction UI read
  shared `PlayerEconomyView` instead of independently draining economy messages.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-007: Placement Buffer](../../../docs/architecture/adr-007-placement-buffer.md)
- [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md)
- [ADR-010: RSM Phase Event Bus](../../../docs/architecture/adr-010-rsm-event-bus.md)
- [ADR-011: Reconnect Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md)
- [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md)
- [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md)
- [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
- [ADR-023: Placement Timer Accessibility Authority](../../../docs/architecture/adr-023-placement-timer-accessibility-authority.md)

All referenced ADRs are Accepted.

**Engine**: Bevy 0.18 + Lightyear 0.26 + browser/WASM primary client |
**Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing any Bevy `.rs` file and
`liv-bevy-lightyear` before editing any Lightyear or networking `.rs` file.
Client UI and board changes must use Bevy 0.18 Required Components API and the
ADR-021 presentation order. Do not use deprecated bundle APIs, local phase
authority, duplicate S2C drains, or direct server imports from `client/`.

**Lightyear Notes**: The active loop must be observed through real C2S/S2C
traffic. Phase, hand, shop, auction, placement, resolution, and economy state
must update from reliable S2C messages or snapshots. New or repaired receive
paths must preserve one production drainer per Lightyear message type.

**Control Manifest Rules (2026-05-05)**:

- Required: client state is read-only and server remains authoritative.
- Required: `S2CPhaseChanged` is drained only by the shared phase sink.
- Required: Hand UI, HUD, and Shop/Auction UI read `PlayerEconomyView`.
- Required: `S2CResolutionEvent` stays before following
  `S2CPhaseChanged(DRAFT_SHOP)` on `ReliableChannel`.
- Required: Shop/Auction UI must not duplicate S2C phase or gold drains.
- Required: PLACEMENT timer duration comes from server phase or snapshot data,
  including the frozen session multiplier.
- Guardrail: presentation steady-state stays below 1 ms per frame and
  phase-boundary spikes stay below 3 ms.

## Sprint 8 Scope Guard

This story is internal friend-game active-loop polish only. It carries
QA-COND-0005 as friend-game-only accepted risk and does not verify
Standard-tier accessibility completion. It carries QA-COND-0006 as
accepted-risk/deferred and does not create playtest evidence, fun-hypothesis
validation, or a playtest report.

This story must not claim public, external, commercial, store, deployment,
release-candidate, or release readiness. It must not claim broad accessibility
completion, full playable-client manual QA, game-over coverage, full game
completion, or playtest validation.

## Scope

### In Scope

- Repeated active-loop behavior through `DRAFT_SHOP`, `DRAFT_AUCTION`,
  post-auction `DRAFT_SHOP`, non-empty `PLACEMENT`, `RESOLUTION`, and next-loop
  `DRAFT_SHOP`.
- Stale panel cleanup across shop, auction, placement, and resolution phase
  transitions.
- Stale timer cleanup across phase exit, auction settlement, settlement
  interrupt, and reconnect or snapshot recovery where it affects the active
  loop.
- Ready and retract-ready state cleanup across consecutive draft/shop passes.
- Auction feedback cleanup so accepted, rejected, settled, and late messages do
  not leak into later shop or placement phases.
- Preservation of `UnitPlaced` resolution evidence after non-empty placement.
- Verification that client UI does not create optimistic authority for phase,
  hand, shop, auction, placement, resolution, timer, or economy state.

### Out of Scope

- Result endpoint or game-over expansion. PLAYABLE-004 owns that scope.
- Public release readiness, broad accessibility completion, playtest validation,
  full playable-client manual QA, or full game completion.
- Closing QA-COND-0005 or QA-COND-0006.
- Broad UI redesign, art polish, card content expansion, balance tuning, new
  game modes, or reconnect polish beyond active-loop blockers.
- Replacing SAU-007 settlement behavior. This story may rely on or integrate
  SAU-007 but should not duplicate that story's settlement-to-shop ownership.

## Acceptance Criteria

- [ ] **Repeated active route is observable**: GIVEN one real local server and
      two real primary clients from the same commit, WHEN the active loop is
      exercised, THEN evidence records `DRAFT_SHOP -> PLACEMENT(non_empty) ->
      RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) ->
      RESOLUTION -> DRAFT_SHOP` or records the exact blocker before the route
      completes.
- [ ] **Second pass after Sprint 7 endpoint is covered**: GIVEN the Sprint 7
      endpoint is reached, WHEN scoped capacity allows continued execution, THEN
      at least two consecutive loop passes after that endpoint are evidenced; a
      shorter route records the concrete blocker and nearest endpoint.
- [ ] **Stale panels are cleared**: GIVEN phase changes move from shop or auction
      into placement or resolution, WHEN client UI settles after one update
      frame, THEN inactive shop, auction, ready, bid, and placement panels are
      hidden or reset according to the current server phase.
- [ ] **Stale timers are cleared**: GIVEN a draft/shop, auction, or placement
      timer exits its owning phase, WHEN the next phase is observed, THEN the old
      countdown no longer ticks, displays, or gates input.
- [ ] **Ready state does not duplicate**: GIVEN a player readies or retracts in
      one draft/shop pass, WHEN the next draft/shop pass starts, THEN ready,
      retract, waiting, and disabled states reflect only current authoritative
      phase state.
- [ ] **Auction feedback does not leak**: GIVEN accepted, rejected, in-flight,
      settled, and late auction messages occur near a phase boundary, WHEN shop
      or placement becomes active, THEN stale bid labels, toasts, price leaders,
      and disabled bid gates do not resurrect inactive auction UI.
- [ ] **Resolution replay keeps UnitPlaced**: GIVEN both players submit
      non-empty placements, WHEN resolution evidence is captured, THEN both
      clients observe `S2CResolutionEvent` content containing `UnitPlaced` before
      the following next-loop phase.
- [ ] **Client authority remains read-only**: GIVEN any local click, timer expiry,
      bid feedback, purchase feedback, ready action, placement submit, or
      resolution transition, WHEN visible client state changes, THEN the change
      is driven by inbound S2C or snapshot state and not by optimistic local
      mutation of server-owned state.
- [ ] **Required commands pass or blockers are explicit**:
      `cargo test -p server --test playable_client_active_loop_polish_test`,
      `cargo test -p client --test playable_client_active_loop_ui_state_test`,
      `cargo test -p client --test shop_auction_ui_auction_settlement_test`,
      `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`,
      `cargo test -p server --test playable_client_real_e2e_loop_test`,
      `cargo check --workspace`, and `git diff --check` pass, or exact failing
      command output is recorded as a blocker.
- [ ] **Evidence document exists**:
      `production/qa/evidence/sprint-8-friend-game-loop-evidence.md` records
      commit, commands, target, two-client setup, repeated-loop route, stale
      state checks, `UnitPlaced` evidence, defects, QA-COND-0005 and QA-COND-0006
      accepted-risk context, and all Sprint 8 non-claims.

## Likely Files Touched

- `tests/integration/playable_client/active_loop_polish_test.rs`
- `tests/integration/playable_client/active_loop_ui_state_test.rs`
- `tests/integration/playable_client/real_e2e_loop_test.rs`
- `client/src/ui/shop_auction/mod.rs`
- `client/src/ui/hand/mod.rs`
- `client/src/ui/hud/mod.rs`
- `client/src/presentation/mod.rs`
- `client/src/presentation/board_rendering.rs`
- `client/src/presentation/shared/economy_view.rs`
- `client/src/state/mod.rs`
- `server/src/network/mod.rs`
- `server/src/core/rsm/system.rs`
- `server/src/feature/board/placement.rs`
- `server/src/feature/combat/mod.rs`
- `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/`

Any source change must remain focused on stale loop state, ordering, evidence,
or authority drift found in the active friend-game loop.

## Implementation Notes

- Begin from the proven Sprint 7 route and add focused assertions around stale
  state and repeated-loop behavior before changing implementation.
- Keep the settlement-to-shop transition aligned with SAU-007. If SAU-007 is not
  merged, coordinate branch order rather than duplicating settlement ownership.
- Prefer state cleanup at phase-boundary sinks and owning UI resources over
  ad-hoc cleanup inside unrelated widgets.
- Do not add duplicate `MessageReceiver<S2CPhaseChanged>` or
  `MessageReceiver<S2CGoldUpdate>` drains. Read `CurrentClientPhase` and
  `PlayerEconomyView`.
- Preserve `S2CResolutionEvent` and `S2CPhaseChanged` reliable ordering. A
  detected ordering anomaly should become evidence and a focused defect, not a
  client-side guess.
- Keep manual evidence labels exact: internal friend-game loop evidence, not
  playtest validation and not full playable-client manual QA.

## Performance Budget

Loop polish must preserve presentation steady-state below 1 ms per frame and
phase-boundary spikes below 3 ms. Server steady-state remains at or below 5 ms
per tick and RESOLUTION remains at or below 15 ms. Cleanup should be O(1) over
current panel, timer, ready, auction feedback, and placement resources; do not
spawn or despawn steady-state UI every frame.

## QA Test Cases

- **Repeated active loop**
  - Given: both clients have reached the Sprint 7 endpoint.
  - When: the loop continues through another shop, auction, placement, and
    resolution path.
  - Then: evidence records the repeated route or the exact blocker before it
    completes.

- **Stale UI cleanup**
  - Given: auction, shop, ready, timer, and placement UI have all been active in
    previous phases.
  - When: the current authoritative phase changes.
  - Then: only the current phase's UI remains visible and interactive.

- **Late auction feedback**
  - Given: accepted, rejected, or settled messages arrive near a phase boundary.
  - When: shop or placement is active.
  - Then: stale bid feedback cannot update price, leader, buttons, or inactive
    panels.

- **Non-empty placement and resolution**
  - Given: both clients submit non-empty placements.
  - When: resolution runs.
  - Then: both clients observe `UnitPlaced` content before the next-loop phase.

## Test Evidence

**Story Type**: Integration

**Required automated test targets**:

- `tests/integration/playable_client/active_loop_polish_test.rs`
  - Registered as `playable_client_active_loop_polish_test`
  - Command: `cargo test -p server --test playable_client_active_loop_polish_test`
- `tests/integration/playable_client/active_loop_ui_state_test.rs`
  - Registered as `playable_client_active_loop_ui_state_test`
  - Command: `cargo test -p client --test playable_client_active_loop_ui_state_test`

**Required manual friend-game evidence document**:

- `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`

**Required capture artifact directory**:

- `production/qa/evidence/captures/sprint-8-friend-game-loop/`

**Required regression commands**:

- `cargo test -p client --test shop_auction_ui_auction_settlement_test`
- `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`
- `cargo test -p server --test playable_client_real_e2e_loop_test`
- `cargo check --workspace`
- `git diff --check`

**Final evidence expectations**:

- Exact commit and build target.
- Server and both primary-client commands.
- Confirmation that both clients used the same build.
- Repeated-loop route and exact endpoint reached.
- Stale panel, stale timer, ready-state, auction-feedback, placement, and
  resolution checks.
- `UnitPlaced` evidence after non-empty placement.
- Defects with severity, owner/system, workaround where one exists, and internal
  friend-game impact.
- No-harness statement.
- Explicit statement that this is internal friend-game evidence only and not
  public release readiness, broad accessibility completion, playtest validation,
  fun-hypothesis validation, full playable-client manual QA, game-over coverage,
  or full game completion.

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 003 Real End-to-End Loop Verification](story-003-real-end-to-end-loop-verification.md) - Complete.
- Depends on: [Auction Settlement and Shop Transition](../shop-auction-ui/story-007-auction-settlement-and-shop-transition.md) - Ready; must be underway or complete before LOOP-001 implementation changes duplicate settlement-to-shop behavior.
- Depends on: existing Card Acquisition, Auction System, RSM, Hand UI, HUD,
  Board Rendering, Economy, Presentation Layer, and Network Protocol contracts
  on `main`.
- Unlocks: S8-QA-001 repeated-loop manual smoke package and later Sprint 8
  evidence indexing.

## Blockers

- No design or story-doc blocker remains for readiness.
- `/dev-story` should wait until CI is green per Sprint 8 planning context.
- If SAU-007 is not underway or complete, run SAU-007 before LOOP-001 or
  explicitly coordinate branch order.
