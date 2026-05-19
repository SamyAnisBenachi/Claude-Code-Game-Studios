# Epic: Presentation Layer

> **Layer**: Presentation
> **GDD**: ADR-021 cross-epic infrastructure and Sprint 6 accessibility gate control
> **Architecture Module**: `client/src/presentation/` - `PresentationPlugin`
> **Status**: Ready
> **Stories**: 6 shared prerequisite/control/accessibility/result/overlay stories

## Overview

Presentation Layer owns the shared client-side infrastructure defined by ADR-021. It provides the top-level `PresentationPlugin`, global `PresentationSet` ordering, the single `S2CPhaseChanged` phase sink, and the canonical `CurrentClientPhase` access path used by presentation sub-plugins.

This epic exists because ADR-021 defines cross-epic infrastructure that should not be owned by Board Rendering 001 or Shop/Auction UI 001. Board Rendering, HUD, Hand UI, Card Animations, and Shop/Auction UI remain independently owned sub-plugins; this epic owns only the scheduling and phase-state bridge that lets those sub-plugins compose safely.

Sprint 6 also uses this epic for S6-04 accessibility control stories. Story 003 is docs-only and prevents broad Settings / Accessibility implementation scope from opening until QA-COND-0005 has a complete row-by-row evidence, reclassification, accepted-risk, or dependency disposition register. Story 005 is now the final cross-surface Browser/WASM evidence story for A11Y-ST-02 after Hand UI Story 015 and Shop/Auction UI Story 013 land their owning surface work.

Sprint 9 also uses this epic for the Result Screen MVP story because
the screen is a cross-surface presentation overlay above frozen HUD and board
state. Story 006 is ready after S9-RS-001 completed the result
acknowledgement and retained GAME_OVER result data contract. Alive opponent
objective identities remain `Unknown` for MVP unless a separate
server-authoritative reveal payload is scoped, and rematch remains hidden or
disabled. Story 006 also owns the initial S9-RS-004 focus, reduced-motion, and
viewport evidence scope; no standalone S9-RS-004 story exists unless
implementation evidence later shows a route-blocking polish split is needed.

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
| TR-PRES-001 | Shared `PlayerEconomyView` mirrors own current/reserve mana from `S2CGoldUpdate` and `S2CGameSnapshot`; presentation sub-plugins read the resource instead of draining economy messages independently | ADR-021, ADR-002, ADR-008 |
| S6-04 / QA-COND-0005 | Standard-tier accessibility gaps must be implemented/evidenced, accepted risk with producer signoff, reclassified out of the Production -> Polish gate, or explicitly dependency-blocked before QA-COND-0005 closure can be considered | ADR-023, ADR-021 |
| A11Y-ST-02 / QA-COND-0005 | Card cost, ATK, HP, and keyword text floors must be verified with final browser/WASM evidence across the Hand UI and Shop/Auction UI card surfaces implemented by their split A11Y-ST-02 stories | ADR-021, ADR-002 |
| Result Screen UX | GAME_OVER result overlay displays server-authoritative win/loss/draw/no-result copy, objective summary with Unknown fallbacks, frozen HUD/background behavior, Return to Lobby, rematch disabled/hidden unless scoped, keyboard focus, reduced-motion behavior, viewport stability, and no public/full-QA/full-game claims | ADR-021, ADR-002, ADR-008, ADR-011 |
| PROMPT 1265 / 1266 / 1267 overlay findings | Result and DraftInitial decision overlays use Krosmaga-style chrome as composition reference only: stronger modal hierarchy, large cards, CTA below content, no mulligan/progression semantic drift, no Krosmaga asset release claim | ADR-021, ADR-002 |

## Traceability Notes

Story 001 remains ADR-only infrastructure and does not need a `TR-PRES-*` entry. Story 002 is GDD-derived shared client state required by HAND-UI-010, so it is registered as `TR-PRES-001` in `docs/architecture/tr-registry.yaml`. Story 003 is a Sprint 6 QA control/evidence story and does not need a `TR-PRES-*` entry because it is traceable to S6-04, QA-COND-0005, the Sprint 6 QA plan, and ADR-023. Story 005 is a Sprint 6 QA accessibility evidence story and does not need a new `TR-PRES-*` entry because it is traceable to A11Y-ST-02, QA-COND-0005, the Sprint 6 accessibility evidence register, Hand UI Story 015, Shop/Auction UI Story 013, and the owning GDD requirements for card surfaces. Story 006 is traced to `design/ux/result-screen.md` plus `TR-NP-001`, `TR-NP-005`, `TR-RSM-008`, `TR-RSM-009`, `TR-HUD-009`, PLAYABLE-004 controlled GAME_OVER evidence, and the carried QA-COND-0005/0006 risk files; it also owns the initial S9-RS-004 focus/reduced-motion/viewport evidence scope. No dedicated `TR-PRES-*` entry exists yet.

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
| Result Screen | Future result-screen presentation module | Opens from shared GAME_OVER phase/result state above frozen HUD and board without mutating either surface |

## Current Implementation Gaps

- `client/src/presentation/` and `PresentationPlugin` do not exist yet.
- `PresentationSet` does not exist yet; HUD, Hand UI, and Card Animations currently use local system sets only.
- `CurrentClientPhase` exists in `client/src/state/mod.rs`, but there is no ADR-021 presentation-layer export/path contract.
- No shared `phase_sink_system` currently drains `MessageReceiver<S2CPhaseChanged>`.
- `client/src/main.rs` registers `ClientNetworkPlugin`, but not the top-level Presentation plugin.
- `BoardRenderingPlugin` and `ShopAuctionUiPlugin` do not exist yet and must remain owned by their own epics.
- A11Y-ST-02 remains open because Hand UI Story 015, Shop/Auction UI Story 013, and the final Presentation Story 005 evidence pass have not yet been implemented and captured.
- Result Screen MVP is not implemented.
- S9-RS-001 is complete: `C2SAcknowledgeResult` has a server-side acknowledgement handler, retained ended-session result state, all-ack cleanup, timeout cleanup, and retained GAME_OVER reconnect resend.
- `S2CGameSnapshot` still does not include final loser, round, or reason fields; this is acceptable for Result Screen MVP because S9-RS-001 uses retained final snapshot plus retained `S2CGameOver` resend instead.
- Full post-game reveal data for alive opponent objective identities is not available and remains out of scope; destroyed opponent identity is available through `OpponentObjectiveSnapshot.was_fake`, and alive opponent identities render as `Unknown`.
- Rematch protocol is not defined and must stay disabled or hidden unless separately scoped.

## Definition of Done

This epic is complete when:

- `PresentationPlugin` exists and is exported from the client crate.
- `PresentationSet` defines and configures the ADR-021 global presentation order.
- `phase_sink_system` is the only client-side drain of `MessageReceiver<S2CPhaseChanged>`.
- HUD, Hand UI, and Card Animations scheduling can compose through the shared presentation order without taking ownership of ADR-021 infrastructure.
- Board Rendering Story 001 and Shop/Auction UI Story 001 explicitly depend on this shared foundation story.
- The implementation story has passing integration evidence under `tests/integration/presentation/`.
- S6-04 accessibility disposition work has a complete register at `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md` before broader accessibility implementation scope proceeds.
- A11Y-ST-02 card text accessibility evidence is captured at `production/qa/evidence/presentation-card-text-accessibility.md` with browser/WASM captures under `production/qa/evidence/captures/presentation-card-text-accessibility/` after Hand UI Story 015 and Shop/Auction UI Story 013 land, before the row is treated as implemented and evidenced.
- Result Screen MVP consumes the S9-RS-001 retained `S2CGameOver` plus retained final `S2CGameSnapshot` contract and keeps alive opponent objectives as `Unknown` unless separate server authority exists.
- Game Session System Story 009 / S9-RS-001 remains complete before Presentation Story 006 implementation assignment.
- No BoardLayout, CardAtlas, BoardRenderingPlugin, ShopAuctionUiPlugin, panel tree, unrelated visual spawning, gameplay logic, rematch protocol, or server contract change is implemented by this epic.

## Stories

| # | Story | Type | Status | Requirement | ADR |
|---|-------|------|--------|-------------|-----|
| 001 | [PresentationPlugin, PresentationSet, and Phase Sink](story-001-presentation-plugin-set-and-phase-sink.md) | Integration | Ready-for-Readiness | ADR-021 R1, R5 | ADR-021 |
| 002 | [Shared Economy View](story-002-shared-economy-view.md) | Integration | Ready | TR-PRES-001 | ADR-021, ADR-002, ADR-008 |
| 003 | [S6 Accessibility Disposition and Evidence Register](story-003-s6-accessibility-disposition-and-evidence-register.md) | Config/Data | Ready | S6-04 / QA-COND-0005 | ADR-023, ADR-021, ADR-002 |
| 005 | [A11Y-ST-02 Cross-Surface Browser/WASM Evidence](story-005-card-text-stat-keyword-accessibility.md) | UI | Ready | A11Y-ST-02 / QA-COND-0005 | ADR-021, ADR-002, ADR-013, ADR-015, ADR-019 |
| 006 | [Result Screen MVP](story-006-result-screen-mvp.md) | UI | Ready | Result Screen UX / TR-NP-001 / TR-RSM-008 / TR-HUD-009 | ADR-021, ADR-002, ADR-008, ADR-011 |
| 007 | [Result + Mulligan-Style Overlay Chrome](story-007-krosmaga-result-mulligan-overlay-chrome.md) | UI + Integration + Visual Evidence | Draft -- future Sprint 19 candidate (`S19-PRES-RESULT-MULLIGAN-OVERLAY-CHROME-001`; PROMPT 1280 Krosmaga-style implementation wave), NOT activated | PROMPT 1265 / 1266 / 1267 overlay findings | ADR-021, ADR-002, ADR-008, ADR-011 |

## Next Step

For the accessibility path, run Hand UI Story 015 first, then Shop/Auction UI Story 013 after any needed settlement surface work, then run Presentation Story 005 as the final cross-surface Browser/WASM evidence pass. For the result-screen path, assign Story 006 within its MVP boundaries: no enabled rematch, alive opponent objectives remain `Unknown`, and no manual/browser GAME_OVER or QA closure claim is made by implementation readiness. Use `liv-bevy-018` for every Bevy `.rs` file and `liv-bevy-lightyear` for every Lightyear/networking `.rs` file during implementation. QA-COND-0005 remains Open until every Standard-tier accessibility row has valid implementation/evidence, dependency-blocking, reclassification, or accepted-risk disposition.
