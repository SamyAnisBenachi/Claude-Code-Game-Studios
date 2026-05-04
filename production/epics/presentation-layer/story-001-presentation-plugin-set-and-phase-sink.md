# Story 001: PresentationPlugin, PresentationSet, and Phase Sink

> **Epic**: Presentation Layer
> **Status**: Ready-for-Readiness
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: None. This is ADR-021 cross-epic infrastructure.
**Requirement**: ADR-021 R1 and R5; Control Manifest Presentation Layer rules.
**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md), [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md), [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)

ADR-021 defines the shared presentation foundation as cross-epic infrastructure. It must not be implemented inside Board Rendering 001 or Shop/Auction UI 001. This story creates the future implementation target for the shared bridge: `PresentationPlugin`, `PresentationSet`, `phase_sink_system`, and the canonical presentation access path for `CurrentClientPhase`.

`CurrentClientPhase` already exists in `client/src/state/mod.rs`, and HUD / Hand UI already read it. This story should preserve one canonical shared resource while exposing it through the presentation-layer contract expected by ADR-021.

**Readiness note**: Status is `Ready-for-Readiness`, not `Ready`, because this story is ADR-only infrastructure without a direct GDD file or registered TR-ID. `/story-readiness` should verify that ADR-021 R1/R5 and the Control Manifest are sufficient traceability, or require a TR registry entry before implementation.

## Traceability Gap

No dedicated `TR-PRES-*` entry exists in `docs/architecture/tr-registry.yaml` for this shared ADR-021 presentation infrastructure. This story does not invent one. It is traceable to ADR-021 R1 and R5, plus the Presentation Layer rules in `docs/architecture/control-manifest.md`.

## Acceptance Criteria

- [ ] `client/src/presentation/mod.rs` exists and defines `PresentationPlugin`.
- [ ] `client/src/presentation/mod.rs` defines `PresentationSet` with these ordered variants: `PhaseTransition`, `MessageDrain`, `StateSync`, `AnimationTick`.
- [ ] `PresentationPlugin` configures `PresentationSet` in `Update` as `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick`, chained and gated to `ClientState::InSession` where appropriate.
- [ ] `PresentationPlugin` owns the ADR-021 phase sink: exactly one client system drains `MessageReceiver<S2CPhaseChanged>`.
- [ ] `phase_sink_system` updates `CurrentClientPhase` using last-write-wins if more than one phase message is received in the same frame.
- [ ] `phase_sink_system` ignores timer display data; timer-bearing UI remains owned by downstream presentation systems.
- [ ] `CurrentClientPhase` has one canonical shared presentation access path, either by moving it into `client/src/presentation/` with compatibility re-exports or by re-exporting the existing `client/src/state/mod.rs` type from `client/src/presentation/mod.rs`.
- [ ] `client/src/lib.rs` exports the presentation module.
- [ ] `client/src/main.rs` registers `PresentationPlugin` after networking/protocol setup in the client app.
- [ ] Existing HUD scheduling bridges into `PresentationSet::PhaseTransition`, `PresentationSet::MessageDrain`, and `PresentationSet::StateSync` without changing HUD feature behavior.
- [ ] Existing Hand UI scheduling bridges into `PresentationSet::PhaseTransition`, `PresentationSet::MessageDrain`, and `PresentationSet::StateSync` without changing Hand UI feature behavior.
- [ ] Existing Card Animations scheduling bridges into the shared presentation ordering without changing animation feature behavior.
- [ ] Board Rendering and Shop/Auction UI registration slots are documented in `PresentationPlugin` in the ADR-021 order, but their actual plugin implementations are not created by this story.
- [ ] `rg "MessageReceiver<S2CPhaseChanged>" client/src` reports exactly one production-code drain, owned by the shared phase sink.
- [ ] Integration evidence exists at `tests/integration/presentation/presentation_plugin_scaffold_test.rs`.

## Implementation Notes

- Follow ADR-021 registration order: `CardAnimationsPlugin`, `BoardRenderingPlugin`, `HandUiPlugin`, `HudPlugin`, `ShopAuctionUiPlugin`.
- Because `BoardRenderingPlugin` and `ShopAuctionUiPlugin` do not exist yet, do not create their implementations here. Use narrowly scoped module gates, comments, or follow-up TODOs that do not break compilation.
- Do not register `MessageReceiver<S2CPhaseChanged>` in HUD, Hand UI, Card Animations, Board Rendering, or Shop/Auction UI.
- Use the existing `apply_phase_changed_message` behavior unless readiness or implementation review chooses to move that helper with `CurrentClientPhase`.
- Preserve client authority boundaries: the phase sink updates presentation state only and never sends C2S messages or mutates gameplay state.
- Use Bevy 0.18 Required Components and Message APIs when implementation begins. No `EventReader`, `EventWriter`, `Events<T>`, or deprecated bundle APIs.
- Use Lightyear 0.26 `MessageReceiver<S2CPhaseChanged>` only in the shared sink. Verify the exact receiver query pattern with `liv-bevy-lightyear` before implementation.

## Out of Scope

- `BoardLayout`
- `CardAtlas`
- `BoardRenderingPlugin`
- `ShopAuctionUiPlugin`
- Shop/Auction panel trees
- Board, HUD, hand, shop, or auction visual spawning
- Gameplay logic
- Protocol changes
- Server changes
- Asset loading or atlas ownership
- Card animation feature behavior beyond scheduling bridge integration

## QA Test Cases

- **Presentation set order**
  - Given: a minimal `App` with `PresentationPlugin`
  - When: the app schedule is built
  - Then: `PresentationSet::PhaseTransition`, `MessageDrain`, `StateSync`, and `AnimationTick` are configured in ADR-021 order.

- **Single phase sink**
  - Given: client source after implementation
  - When: `rg "MessageReceiver<S2CPhaseChanged>" client/src` is run
  - Then: exactly one production-code occurrence is the shared `phase_sink_system`.

- **Last-write-wins phase state**
  - Given: two `S2CPhaseChanged` messages are available to the phase sink in one frame
  - When: `phase_sink_system` runs
  - Then: `CurrentClientPhase.phase` and `CurrentClientPhase.round` match the last received phase message.

- **Sub-plugin phase consumption**
  - Given: HUD and Hand UI are registered through `PresentationPlugin`
  - When: `CurrentClientPhase` changes through the shared sink
  - Then: HUD and Hand UI phase-transition systems observe the shared resource and do not drain `S2CPhaseChanged` directly.

## Test Evidence

**Required evidence**:
- Integration: `tests/integration/presentation/presentation_plugin_scaffold_test.rs`
- CI or local grep guard: `MessageReceiver<S2CPhaseChanged>` appears only in the shared phase sink under production client source.

**Status**: [ ] Not yet created

## Dependencies

- Depends on: ADR-021 Accepted; existing client `ClientState`; existing `CurrentClientPhase` scaffold or approved move/re-export path.
- Unlocks: `production/epics/board-rendering/story-001-plugin-scaffold-board-layout-card-atlas.md`; `production/epics/shop-auction-ui/story-001-plugin-scaffold-panel-tree-and-formulas.md`; final ADR-021 registration bridge for HUD, Hand UI, and Card Animations.
