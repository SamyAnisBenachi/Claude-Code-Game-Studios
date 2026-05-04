# Epic: Presentation Layer

> **Layer**: Presentation
> **GDD**: ADR-021 cross-epic infrastructure
> **Architecture Module**: `client/src/presentation/` - `PresentationPlugin`
> **Status**: Ready-for-Readiness
> **Stories**: 1 proposed shared foundation story

## Overview

Presentation Layer owns the shared client-side infrastructure defined by ADR-021. It provides the top-level `PresentationPlugin`, global `PresentationSet` ordering, the single `S2CPhaseChanged` phase sink, and the canonical `CurrentClientPhase` access path used by presentation sub-plugins.

This epic exists because ADR-021 defines cross-epic infrastructure that should not be owned by Board Rendering 001 or Shop/Auction UI 001. Board Rendering, HUD, Hand UI, Card Animations, and Shop/Auction UI remain independently owned sub-plugins; this epic owns only the scheduling and phase-state bridge that lets those sub-plugins compose safely.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|------------------|-------------|
| [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md) | `PresentationPlugin` composition, `PresentationSet` ordering, single `phase_sink_system`, shared `CurrentClientPhase`, cross-plugin scheduling bridge | HIGH |
| [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md) | Client is a read-only view; presentation consumes server-authoritative S2C state only | LOW |
| [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md) | Reliable S2C delivery and Lightyear single-drain discipline | HIGH |
| [ADR-009: Round State Machine Phase State](../../../docs/architecture/adr-009-rsm-phase-state.md) | `S2CPhaseChanged` is the phase broadcast consumed by the client presentation layer | HIGH |

## Requirements

| Source | Requirement | ADR Coverage |
|--------|-------------|--------------|
| ADR-021 R1 | Presentation sub-systems are composable as independently testable plugins sharing defined SystemSet ordering | ADR-021 |
| ADR-021 R5 | `S2CPhaseChanged` is drained by exactly one shared phase sink; sub-plugins read `Res<CurrentClientPhase>` | ADR-021, ADR-008 |
| Control Manifest | `PresentationSet` order is `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick` | ADR-021 |
| Control Manifest | `PresentationPlugin` registration order is Card Animations, Board Rendering, Hand UI, HUD, Shop/Auction UI | ADR-021 |

## Traceability Gap

No dedicated `TR-PRES-*` entry exists in `docs/architecture/tr-registry.yaml` for this shared ADR-021 presentation infrastructure. This epic does not invent a TR-ID. Story 001 is traceable to ADR-021 R1 and R5 and is marked `Ready-for-Readiness` so the readiness gate can explicitly accept or reject ADR-level traceability before implementation.

## Dependency Map

| Dependency | Existing Surface | Presentation Layer Use |
|------------|------------------|------------------------|
| Network Protocol | `S2CPhaseChanged`, `RoundPhase` | Phase sink drains the Lightyear receiver into `CurrentClientPhase` |
| Client State | `ClientState`, current `CurrentClientPhase` scaffold | Provides the session gate and phase resource shape |
| Card Animations | `CardAnimationsPlugin`, `CardAnimationsSet` | Registered first and bridged into `PresentationSet::AnimationTick` / animation scheduling |
| Board Rendering | Future `BoardRenderingPlugin` | Registered second after Card Animations once its own story implements it |
| Hand UI | `HandUiPlugin`, `HandUiSystemSet` | Reads shared phase state; phase/message/state systems bridge into global ordering |
| HUD | `HudPlugin`, `HudSystemSet` | Reads shared phase state; phase/message/state systems bridge into global ordering |
| Shop/Auction UI | Future `ShopAuctionUiPlugin` | Registered fifth once its own story implements it |

## Current Implementation Gaps

- `client/src/presentation/` and `PresentationPlugin` do not exist yet.
- `PresentationSet` does not exist yet; HUD, Hand UI, and Card Animations currently use local system sets only.
- `CurrentClientPhase` exists in `client/src/state/mod.rs`, but there is no ADR-021 presentation-layer export/path contract.
- No shared `phase_sink_system` currently drains `MessageReceiver<S2CPhaseChanged>`.
- `client/src/main.rs` registers `ClientNetworkPlugin`, but not the top-level Presentation plugin.
- `BoardRenderingPlugin` and `ShopAuctionUiPlugin` do not exist yet and must remain owned by their own epics.

## Definition of Done

This epic is complete when:

- `PresentationPlugin` exists and is exported from the client crate.
- `PresentationSet` defines and configures the ADR-021 global presentation order.
- `phase_sink_system` is the only client-side drain of `MessageReceiver<S2CPhaseChanged>`.
- HUD, Hand UI, and Card Animations scheduling can compose through the shared presentation order without taking ownership of ADR-021 infrastructure.
- Board Rendering Story 001 and Shop/Auction UI Story 001 explicitly depend on this shared foundation story.
- The implementation story has passing integration evidence under `tests/integration/presentation/`.
- No BoardLayout, CardAtlas, BoardRenderingPlugin, ShopAuctionUiPlugin, panel tree, visual spawning, gameplay logic, or protocol change is implemented by this epic.

## Stories

| # | Story | Type | Status | Requirement | ADR |
|---|-------|------|--------|-------------|-----|
| 001 | [PresentationPlugin, PresentationSet, and Phase Sink](story-001-presentation-plugin-set-and-phase-sink.md) | Integration | Ready-for-Readiness | ADR-021 R1, R5 | ADR-021 |

## Next Step

Run `/story-readiness production/epics/presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md` before implementation. Use `liv-bevy-018` for every Bevy `.rs` file and `liv-bevy-lightyear` for every Lightyear/networking `.rs` file during implementation.
