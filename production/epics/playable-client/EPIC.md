# Epic: Playable Client

> **Layer**: Polish / Cross-Cutting Client Integration
> **GDD**: Sprint 7 playable path traced to game-session-system, network-protocol, card-acquisition, hand-ui, shop-auction-ui, hud, board-rendering, and round-state-machine GDDs
> **Architecture Module**: `client/src/main.rs`, `client/src/network/`, `client/src/state/`, `client/src/presentation/`, `client/src/ui/`, plus server message bridges in `server/src/network/` and owning server systems
> **Status**: Ready
> **Stories**: 8 Sprint 7/8 must-have stories, backlog prep stories, and Sprint 9 evidence-prep stories

## Overview

Sprint 7 makes the Polish build playable for an internal friend-game session through the real primary client path. Sprint 8 extends and hardens that same path beyond the proven next-loop DRAFT_SHOP endpoint without broadening the claim. A Sprint 9 story now owns the native operator-control gap found after S8-QA-001: controlled real-Lightyear tests cover the route, but native two-client manual evidence still needs clean player controls for lobby, draft, shop, auction, placement, and result dependency handling. Sprint 9 adds S9-QA-001 and S9-QA-002 as evidence-only stories for the carried manual/browser GAME_OVER gap and follow-up result evidence index. This epic exists because the work cuts across client bootstrap, Lightyear connection flow, lobby/session entry, live draft and shop UI, hand/economy presentation, placement, resolution, endpoint/result evidence, and manual friend-game evidence. None of the existing single-system epics cleanly owns that whole vertical path.

The epic does not create public release readiness, broad accessibility completion, playtest validation, full playable-client manual QA, or full game completion. It only owns the internal friend-game playable path needed for Sprint 7 and Sprint 8 friend-game robustness work. QA-COND-0005 remains accepted risk for friend-game scope only. QA-COND-0006 remains accepted-risk/deferred.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|------------------|-------------|
| [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md) | Server owns all game state; client emits C2S intents and mutates visible state only from S2C messages | HIGH |
| [ADR-003: Cargo Workspace Structure](../../../docs/architecture/adr-003-cargo-workspace-structure.md) | `shared/`, `server/`, and `client/` boundaries prevent client from importing server logic | MEDIUM |
| [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md) | Reliable channel owns game-state/control messages; only heartbeat remains unreliable | HIGH |
| [ADR-011: Reconnect Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md) | Snapshot and identity mapping are server-authoritative and ordered before live messages | HIGH |
| [ADR-012: SessionReady Delivery](../../../docs/architecture/adr-012-session-ready-delivery.md) | Lobby completion enters the round loop through the same-frame `SessionReady` Observer | HIGH |
| [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md) | Draft/shop hand state is server-authoritative and projected through S2C messages | HIGH |
| [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md) | Economy mutations stay server-side and presentation reads projected economy views | MEDIUM |
| [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md) | Primary client presentation uses the shared phase sink, economy view, and Bevy UI composition order | HIGH |

## Requirements

| Source | Requirement |
|--------|-------------|
| `production/sprints/sprint-7.md` | Friend-game playable quality through real primary client path, without public release, broad accessibility, full playtest, or full manual QA claims |
| `design/gdd/network-protocol.md` / `TR-NP-001` | Client expresses intent through C2S messages only; server is authoritative |
| `design/gdd/network-protocol.md` / `TR-NP-003` | `C2SHello` is the first connection message and produces `S2CHandshake` or rejection |
| `design/gdd/game-session-system.md` / `TR-GSS-001`, `TR-GSS-004`, `TR-GSS-007` | Create/join/class lock/session ready flow leads into DRAFT_INITIAL through the real session system |
| `design/gdd/card-data-pool.md` / `TR-CDP-010` | Draft/shop payloads are reliable unicast after authoritative server state exists and before client phase/UI use |
| `design/gdd/shop-auction-ui.md` / `TR-SAU-006` | Shop/Auction UI transitions and input gating follow server phase and authoritative messages |
| `design/gdd/hand-ui.md` / `TR-HU-005`, `TR-HU-008`, `TR-PRES-001` | Draft/hand/economy views use server-projected card and economy data |
| `design/gdd/network-protocol.md` / `TR-NP-007`, `TR-NP-009`, `TR-NP-011` | Placement close, resolution ordering, and next-loop phase updates remain same-channel authoritative messages |
| `production/sprints/sprint-8.md` | Friend-game result endpoint expansion and active loop polish beyond Sprint 7's next-loop DRAFT_SHOP endpoint, without public release, broad accessibility, playtest, full manual QA, game-over unless reached, or full completion claims |
| `design/gdd/round-state-machine.md` / `TR-RSM-008`, `TR-RSM-009` | GAME_OVER detection and reliable phase broadcasts remain server-authoritative and exactly evidenced when reached |
| `design/gdd/hud.md` / `TR-HUD-009` | HUD FROZEN mode records final state on GAME_OVER without incremental updates |
| `production/qa/evidence/sprint-8-friend-game-loop-evidence.md` / `S8-QA-001-W1` | Native/browser manual two-client route was not captured; future native operator controls must preserve no-claim language while closing or narrowing that evidence gap |
| `production/sprints/sprint-9.md` / `S9-QA-001` | Manual/browser or native two-client friend-game route must capture lobby create/join through result screen and acknowledgement before S8-QA-001-W1 can be closed |
| `production/sprints/sprint-9.md` / `S9-QA-002` | Sprint 9 result evidence index must record endpoint, manual/browser status, result-screen status, acknowledgement status, and carried non-claims after evidence exists |

## Scope

### In Scope

- Primary client startup into a real Bevy window/browser path.
- Fresh lobby entry using real Lightyear C2S/S2C messages.
- Minimal friend-game lobby/session UI for create, join, class select, class confirm, and session entry.
- Live draft, shop, hand, economy, ready, placement, resolution, and next-loop bridge through real messages.
- Two-real-client friend-game evidence for the nearest complete playable endpoint.
- Sprint 8 result endpoint expansion toward GAME_OVER or an explicit accepted nearest-endpoint improvement.
- Sprint 8 active DRAFT_SHOP, auction, placement, and resolution loop polish for stale panels, stale timers, ready-state cleanup, auction feedback cleanup, `UnitPlaced` visibility, and client authority drift.
- Backlog native operator controls that make the existing friend-game route manually driveable through real UI controls without debug-only command paths or client-side optimistic authority.
- Sprint 9 manual/browser or native two-client GAME_OVER evidence closure after S9-RS-002 and S9-RS-003 are complete.
- Sprint 9 result evidence index cleanup after S9-QA-001 evidence or blocker records exist.

### Out of Scope

- Public, external, commercial, release-candidate, store, deployment, or certification readiness.
- Broad Standard-tier accessibility completion.
- Editing the accepted-risk disposition for `QA-COND-0005` or `QA-COND-0006`.
- Claiming playtest validation, fun-hypothesis validation, or full playable-client manual QA.
- New modes, broad class/content expansion, or balance redesign.
- Full game completion or game-over coverage unless the exact Sprint 8 evidence reaches and records it.
- Sprint 9 activation, Sprint 8 close-out, or S8-QA-001 closure from docs-prep story authoring alone.

## Control Manifest Rules

- Client presentation is a read-only view of server-authoritative state.
- `ClientState` and visible client state change only from inbound S2C/snapshot state, not from local optimism.
- `shared/` remains protocol/data only; no server dependencies may be added to `client/`.
- `S2CPhaseChanged` is drained by the shared phase sink only.
- `PlayerEconomyView` is the shared economy view used by Hand UI, HUD, and Shop/Auction UI.
- Bevy 0.18 Required Components API is mandatory for UI and rendering work.
- Lightyear `MessageReceiver<T>` has one production drainer per message type.

## Dependency Map

| Dependency | Use |
|------------|-----|
| Game Session System | Room creation, join, class confirm, `SessionReady`, session token, snapshot |
| Network Protocol | C2S/S2C message definitions and reliable channel routing |
| Presentation Layer | Primary client phase sink, snapshot sink, economy view, plugin composition |
| Shop/Auction UI | Draft/shop purchase, refresh, auction, ready controls |
| Hand UI | DRAFT_INITIAL grid, hand fan, placement staging, placement submit |
| HUD | Gold, mana, phase, round, objective readouts |
| Board Rendering | Board layout, placement reveal, resolution visualization, spawn range highlights |
| RSM / Card Acquisition / Economy / Auction / Combat | Server-side authoritative loop behind the client bridge |

## Stories

| # | Story | Type | Status | Requirement |
|---|-------|------|--------|-------------|
| 001 | [Primary Client Bootstrap + Fresh Lobby Entry](story-001-primary-client-bootstrap-fresh-lobby-entry.md) | Integration | Ready | PLAYABLE-001 |
| 002 | [Live Draft/Shop/Hand Bridge](story-002-live-draft-shop-hand-bridge.md) | Integration | Ready | PLAYABLE-002 |
| 003 | [Real End-to-End Loop Verification](story-003-real-end-to-end-loop-verification.md) | Integration | Ready | PLAYABLE-003 |
| 004 | [Friend-Game Result Endpoint Expansion](story-004-friend-game-result-endpoint-expansion.md) | Integration | Ready | PLAYABLE-004 |
| 005 | [DRAFT_SHOP / Auction / Placement / Resolution Loop Polish](story-005-draft-shop-auction-placement-resolution-loop-polish.md) | Integration | Ready | LOOP-001 |
| 006 | [Native Friend-Game Operator Controls](story-006-native-friend-game-operator-controls.md) | Integration | Ready - backlog prep only | Native operator controls / S8-QA-001-W1 |
| 007 | [Manual Browser GAME_OVER Evidence Closure](story-007-manual-browser-game-over-evidence-closure.md) | Integration | Blocked - depends on result screen and acknowledgement handshake | S9-QA-001 / S8-QA-001-W1 |
| 008 | [Sprint 9 Result Evidence Index Cleanup](story-008-sprint-9-result-evidence-index-cleanup.md) | Config/Data | Blocked - depends on S9-QA-001 evidence | S9-QA-002 |

## Definition of Done

- PLAYABLE-001, PLAYABLE-002, and PLAYABLE-003 are complete with their required evidence.
- PLAYABLE-004 records either actual GAME_OVER/result evidence or an accepted nearest-endpoint improvement with no game-over claim.
- LOOP-001 records repeated active-loop behavior without stale panels, stale timers, duplicate ready state, stale auction feedback, missing `UnitPlaced`, or client-side optimistic authority.
- Two real primary clients can use a real local server through the scoped friend-game path.
- Evidence documents the exact build, commit, commands, captures, reached endpoint, defects, and no-harness condition.
- Sprint 7 and Sprint 8 claims remain limited to internal friend-game playable quality and robustness. Public release readiness, broad accessibility completion, playtest validation, full playable-client manual QA, game-over coverage unless actually reached, and full game completion remain out of scope.
- Native friend-game operator controls are documented and later evidenced before claiming a manually driven native two-client route. Story implementation alone does not close S8-QA-001-W1 or expand Sprint 8 claims.
- Sprint 9 manual/browser GAME_OVER evidence closes S8-QA-001-W1 only if Story 007 captures the full route through result acknowledgement. If blocked, Story 007 records the blocker and Story 008 indexes the carried warning without expanding the claim.
