# Epic: Playable Client

> **Layer**: Polish / Cross-Cutting Client Integration
> **GDD**: Sprint 7 playable path traced to game-session-system, network-protocol, card-acquisition, hand-ui, shop-auction-ui, hud, board-rendering, and round-state-machine GDDs
> **Architecture Module**: `client/src/main.rs`, `client/src/network/`, `client/src/state/`, `client/src/presentation/`, `client/src/ui/`, plus server message bridges in `server/src/network/` and owning server systems
> **Status**: Ready
> **Stories**: 3 Sprint 7 must-have stories

## Overview

Sprint 7 makes the Polish build playable for an internal friend-game session through the real primary client path. This epic exists because the work cuts across client bootstrap, Lightyear connection flow, lobby/session entry, live draft and shop UI, hand/economy presentation, placement, resolution, and manual friend-game evidence. None of the existing single-system epics cleanly owns that whole vertical path.

The epic does not create public release readiness, broad accessibility completion, playtest validation, or full playable-client manual QA. It only owns the friend-game playable path needed for Sprint 7.

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

## Scope

### In Scope

- Primary client startup into a real Bevy window/browser path.
- Fresh lobby entry using real Lightyear C2S/S2C messages.
- Minimal friend-game lobby/session UI for create, join, class select, class confirm, and session entry.
- Live draft, shop, hand, economy, ready, placement, resolution, and next-loop bridge through real messages.
- Two-real-client friend-game evidence for the nearest complete playable endpoint.

### Out of Scope

- Public, external, commercial, release-candidate, store, deployment, or certification readiness.
- Broad Standard-tier accessibility completion.
- Editing the accepted-risk disposition for `QA-COND-0005` or `QA-COND-0006`.
- Claiming playtest validation, fun-hypothesis validation, or full playable-client manual QA.
- New modes, broad class/content expansion, or balance redesign.

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

## Definition of Done

- PLAYABLE-001, PLAYABLE-002, and PLAYABLE-003 are complete with their required evidence.
- Two real primary clients can use a real local server through the scoped friend-game path.
- Evidence documents the exact build, commit, commands, captures, reached endpoint, defects, and no-harness condition.
- Sprint 7 claims remain limited to internal friend-game playable quality.
