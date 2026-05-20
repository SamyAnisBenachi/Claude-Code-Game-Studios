# Epic: Board Rendering

> **Layer**: Presentation
> **GDD**: design/gdd/board-rendering.md
> **Architecture Module**: `client/src/ui/board/` - `BoardRenderingPlugin` (sub-plugin #2 inside `PresentationPlugin`)
> **Status**: Ready - story set drafted for S5-21; Story 001 depends on Presentation Layer Story 001
> **Stories**: 15 stories created - 8 Complete, 3 Ready, 2 Blocked, 2 Draft; S6-03 browser/WASM evidence follow-up added as Story 012; canonical board rendering spec authoring story Story 013 Done via Sprint 15 PROMPT 1009 (PROMPT 1004 dev-story + PROMPT 1006 integration + PROMPT 1009 story-done closure); Story 014 is a future Sprint 19 Krosmaga-style PlayArea/targeting feedback candidate authored by PROMPT 1280; Story 015 is a future Sprint 19 resolution-event visual replay mutation candidate authored by PROMPT 1485

## Overview

Board Rendering implements the client-side world-space board view for the M2 visual playable path. It renders the 5 lane x 8 cell board, units, objectives, HP bars, spawn range highlights, ghost previews, placement reveal, and RESOLUTION playback from replicated server state and reliable S2C messages. The client remains a read-only view: Board Rendering does not validate gameplay, mutate authoritative state, or send C2S game-logic messages. The only planned C2S exception is the GDD-defined `C2SRequestSnapshot` desync-recovery request.

`BoardRenderingPlugin` is registered second in `PresentationPlugin`, after `CardAnimationsPlugin` and before `HandUiPlugin`, `HudPlugin`, and `ShopAuctionUiPlugin`. It owns `BoardLayout` and `CardAtlas` as session-scoped resources, provides world-space sprite rendering below bevy_ui, and coordinates with Card Animations for reveal and resolution tweens.

Shared ADR-021 infrastructure is owned by [Presentation Layer Story 001](../presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md). Board Rendering Story 001 must not implement `PresentationPlugin`, `PresentationSet`, or `phase_sink_system`; it depends on those surfaces before implementation.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|------------------|-------------|
| [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md) | Presentation plugin order, `PresentationSet`, single phase sink, world-space board sprites, shared `BoardLayout` and `CardAtlas`, Bevy 0.18 rendering API, tween cancel-and-replace contract | HIGH |
| [ADR-020: Board/Lane System State Architecture](../../../docs/architecture/adr-020-board-lane-state-architecture.md) | Server board state is replicated through unit ECS components; clients render `BoardPosition`, owner, card ref, stats, HP, and keyword state; spawn range is Board/Lane-owned `SpawnRangeState` delivered by snapshot + `SpawnRangeChanged` | HIGH |
| [ADR-017: Combat Resolution Execution Architecture](../../../docs/architecture/adr-017-combat-resolution-execution-architecture.md) | `S2CResolutionEvent` is the authoritative full resolution log; clients replay it visually without changing gameplay state | HIGH |
| [ADR-011: Reconnect and Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md) | Reconnect rebuild is driven by `S2CGameSnapshot` followed by identity and phase messages | HIGH |
| [ADR-001: Objective Identity Unicast](../../../docs/architecture/adr-001-objective-identity-unicast.md) | Objective identity stays server-only except owner unicast; Board Rendering strips fake/real before HUD fanout | LOW |
| [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md) | Reliable channel for phase, snapshot, placement reveal, resolution, and objective identity messages; single-drain discipline | HIGH |
| [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md) | Client is a view; no authoritative board logic in presentation code | LOW |

## GDD Requirements

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-BR-001 | RESOLUTION sub-step visual separation with `AnimQueue` and `Time<Virtual>` timers | ADR-021, ADR-017 |
| TR-BR-002 | `BoardLayout` client resource and canonical `cell_to_world` coordinate authority | ADR-021 |
| TR-BR-003 | Z-order constants in `rendering_constants.rs`; no inline Z literals; replicated board position drives sprites | ADR-021, ADR-020 |
| TR-BR-004 | WALL and collision visuals during SS5 movement playback | ADR-017, ADR-020, ADR-021 |
| TR-BR-005 | Snapshot/reconnect buffers: `PendingPhaseChange` and `PendingResolutionScript` | ADR-011, ADR-021 |
| TR-BR-006 | Persistent status indicators with Tier-1 priority ordering | ADR-021, ADR-018 |
| TR-BR-007 | OUTNUMBERED indicator and keyword-state display reconciliation | ADR-018, ADR-021 |
| TR-BR-008 | Persistent spawn range highlights from `PlayerSnapshot.spawn_range_cells` and live `SpawnRangeChanged` events | ADR-011, ADR-020, ADR-021, ADR-008 |

## Dependency Map

| Dependency | Existing Surface | Board Rendering Use |
|------------|------------------|---------------------|
| Board/Lane System | `server/src/feature/board/`, `BoardPosition`, owner/card/stat components, `SpawnRangeState` source | Spawn/update world-space unit sprites and HP bars from replicated entities; render spawn highlights from snapshot + live `SpawnRangeChanged` |
| Objective System | `ObjectiveHp`, `ObjectiveDestroyed`, objective identity unicast | Render standing objectives and reveal destroyed objective state; fan out HUD-safe updates. Objective does not provide live spawn range projection. |
| Combat Resolution | `S2CPlacementReveal`, `S2CResolutionEvent`, resolution log variants | Placement reveal collection and RESOLUTION animation queue playback |
| Network Protocol | `S2CPhaseChanged`, `S2CGameSnapshot`, `S2CObjectiveIdentities`, resolution and reveal messages | Phase state, reconnect rebuild, objective cache, desync recovery |
| Card Data and Pool | `CardId`, rarity/card metadata, atlas frame mapping | Choose unit/card placeholder frames and eventual atlas frames |
| Game Config | cell sizes, HP thresholds, timers, co-occupancy offsets | Pure formulas and validation for layout, HP bar color, reveal and stuck timers |
| Card Animations | `PlacementRevealAnimReady`, `BoardRebuildRequested`, tween lenses, `AnimQueue` | Board emits animation requests and consumes animation completion where needed |
| Hand UI | ghost preview messages, `BoardLayout` cursor mapping | Draw placement ghost previews and valid spawn highlights |
| HUD | `HudObjectiveUpdate` observer contract | Notify scoreboard without leaking fake/real identity |
| Presentation Layer | `PresentationPlugin`, `PresentationSet`, `CurrentClientPhase`, `phase_sink_system` | Provides shared scheduling and phase sink before Board Rendering registers as sub-plugin #2 |

## Current Implementation Gaps

- `client/src/ui/board/` and `BoardRenderingPlugin` do not exist yet.
- `BoardLayout` currently exists in `client/src/ui/shared.rs`, but its `cell_to_world` API returns `Option<Vec2>` while the Board Rendering GDD specifies a canonical non-optional coordinate function with assertions for invalid lane/cell input. Story 001 must reconcile this before downstream consumers rely on it.
- No client-side `CardAtlas` owner exists.
- No shared `PresentationPlugin`/`phase_sink_system` implementation is visible yet; Board Rendering Story 001 depends on Presentation Layer Story 001 for this cross-epic infrastructure.
- `C2SRequestSnapshot` is defined in the Network Protocol GDD but is not currently present in `shared/src/protocol.rs`; reconnect/desync recovery remains blocked until the protocol type and registration exist.
- Objective destruction transport is inconsistent across docs and code (`ObjectiveDestroyed`, `S2CObjectiveDestroyed`, and `ResolutionEvent::ObjectiveDestroyed` are not yet aligned). HUD fanout remains blocked until that contract lands.
- Spawn range source/transport prerequisites are implemented: NP-006 defines ordered `ResolutionEvent::SpawnRangeChanged`, BLS-012 makes snapshots read Board/Lane `SpawnRangeState`, and BR-011 is the remaining Board Rendering consumer story.

## Pre-Implementation Gates

| Gate | Blocks | Required Resolution |
|------|--------|---------------------|
| Protocol gap: `C2SRequestSnapshot` missing from `shared/src/protocol.rs` | Story 007 | Add shared protocol type and Lightyear registration on ReliableChannel |
| Objective destruction transport gap | Story 008 | Define the final S2C or resolution-log contract and crate location for `HudObjectiveUpdate` fanout |
| Resolution event variant completeness | Story 006 | Resolved 2026-05-05: Combat Resolution Story 011 is Complete and provides the stable `S2CResolutionEvent` / `ResolutionEvent` log contract for queue grouping and phase-buffer playback |
| Art bible / atlas frame count | Stories 003 and 010 | Confirm atlas budget and placeholder-to-final art path before performance evidence is signed off |
| Keyword/status display projection | Story 009 | Implement data-driven status icons and co-occupancy visual offsets |
| Spawn range prerequisite chain | NP-006 Complete -> BLS-012 Complete -> BR-011 Ready | NP defines schema/ordering (NP-33/TR-NP-014); BLS owns `SpawnRangeState` projection and snapshot source; BR consumes snapshot + live event for persistent highlights |
| Browser/WASM baseline evidence | Story 012 | Capture the narrowed BOARD-010 browser/WASM baseline screenshot and frame-time evidence for Sprint 6 S6-03 without claiming BOARD-009 final evidence, spawn highlights, traps, final VFX, or full epic closure |
| Final board evidence | Later split after BR-011 and final visual scope | Capture final status-icon atlas evidence, spawn-highlight evidence, trap/final-VFX evidence if still required, and any final-art icon evidence before full Board Rendering epic closure |
| Presentation Layer scaffold missing | Story 001 | Complete or readiness-approve Presentation Layer Story 001 before implementing Board Rendering Story 001 |

## Definition of Done

This epic is complete when:
- All stories are implemented, reviewed, and closed via `/story-done`.
- All blocking acceptance criteria from `design/gdd/board-rendering.md` are verified.
- Logic and integration stories have passing tests under `tests/unit/board_rendering/` or `tests/integration/board_rendering/`.
- Visual/UI evidence is captured under `production/qa/evidence/`.
- `BoardRenderingPlugin` is registered second in `PresentationPlugin`.
- `BoardLayout` and `CardAtlas` are session-scoped resources available to Hand UI, HUD, and Card Animations.
- `MessageReceiver<S2CPhaseChanged>` is drained only by the shared phase sink.
- The board renders as world-space sprites only; no board units/objectives/HP bars are implemented as bevy_ui nodes.
- `git diff --check`, relevant Cargo tests, and a WASM visual evidence pass are clean before final story closure.

## Stories

| # | Story | Type | Status | TR-IDs | ADR |
|---|-------|------|--------|--------|-----|
| 001 | [Plugin Scaffold, BoardLayout, and CardAtlas](story-001-plugin-scaffold-board-layout-card-atlas.md) | Logic | Complete | TR-BR-002 | ADR-021 |
| 002 | [Board Grid, Camera, and Z Layers](story-002-board-grid-camera-and-z-layers.md) | Logic | Complete | TR-BR-003 | ADR-021 |
| 003 | [Snapshot Spawn: Units, Objectives, and HP Bars](story-003-snapshot-spawn-units-objectives-and-hp-bars.md) | Integration | Complete | TR-BR-003 | ADR-020, ADR-021 |
| 004 | [Ghost Preview and Hand UI Bridge](story-004-ghost-preview-hand-ui-bridge.md) | Integration | Complete | TR-BR-002 | ADR-021 |
| 005 | [Placement Reveal Collect and Tween](story-005-placement-reveal-collect-and-tween.md) | Visual/Feel | Complete | TR-BR-001 | ADR-017, ADR-021 |
| 006 | [Resolution AnimQueue and Phase Buffering](story-006-resolution-anim-queue-and-phase-buffering.md) | Integration | Ready | TR-BR-001, TR-BR-004, TR-BR-005 | ADR-017, ADR-021 |
| 007 | [Reconnect Snapshot and Desync Recovery](story-007-reconnect-snapshot-and-desync-recovery.md) | Integration | Blocked | TR-BR-005 | ADR-011, ADR-021 |
| 008 | [Objective Reveal and HUD Fanout](story-008-objective-reveal-and-hud-fanout.md) | Integration | Blocked | TR-BR-005 | ADR-001, ADR-021 |
| 009 | [Status Icons and Co-Occupancy Visuals](story-009-status-icons-cooccupancy-and-spawn-range.md) | Visual/Feel | Complete | TR-BR-006, TR-BR-007 | ADR-018, ADR-021 |
| 010 | [Performance Evidence and CI Guards](story-010-performance-evidence-and-ci-guards.md) | Config/Data | Complete | TR-BR-003 | ADR-021 |
| 011 | [Spawn Range Highlights](story-011-spawn-range-highlights.md) | Visual/Feel | Ready | TR-BR-008, TR-NP-014 | ADR-011, ADR-020, ADR-021, ADR-008 |
| 012 | [Browser/WASM Board Performance Evidence](story-012-browser-wasm-board-performance-evidence.md) | Config/Data | Ready | TR-BR-003, TR-BR-005 | ADR-021 |
| 013 | [Canonical Board Rendering Spec](story-013-board-rendering-spec.md) | UX -- design-spec authoring (doc-only) | Complete (Sprint 15 Should Have; closed PROMPT 1009 on `origin/main` after PROMPT 1004 dev-story + PROMPT 1006 integration `08f389b`) | TR-BR-002, TR-BR-006, TR-BR-007, TR-BR-008 | ADR-021, ADR-020, ADR-017, ADR-011, ADR-008, ADR-002 |
| 014 | [PlayArea Hierarchy + Targeting Feedback](story-014-krosmaga-playarea-targeting-feedback.md) | UI + Visual/Feel + Integration | Draft -- future Sprint 19 candidate (`S19-BR-PLAYAREA-HIERARCHY-TARGETING-FEEDBACK-001`; PROMPT 1280 Krosmaga-style implementation wave), NOT activated | TR-BR-002, TR-BR-008 | ADR-021, ADR-020, ADR-017, ADR-011, ADR-008, ADR-002 |
| 015 | [Resolution Event Visual Replay Mutation](story-015-resolution-event-visual-replay-mutation.md) | Integration + Visual/Feel | Draft -- future Sprint 19 candidate (`S19-BR-RESOLUTION-EVENT-VISUAL-REPLAY-MUTATION-001`; PROMPT 1485), NOT activated | TR-BR-001, TR-BR-004, TR-BR-005 | ADR-017, ADR-021, ADR-008, ADR-011, ADR-002 |

**Story counts**: 2 Logic, 5 Integration, 3 Visual/Feel, 2 Config/Data, 1 UX design-spec authoring (Sprint 15 Should Have Done via PROMPT 1009), plus 2 Draft future Sprint 19 candidates and later final evidence split follow-up.

## Sprint 6 Candidate Order

Recommended Sprint 6 sequence:
1. Presentation Layer Story 001 - shared plugin, set, and phase sink foundation.
2. Story 001 - scaffold resources and plugin contract.
3. Story 002 - visible board grid, camera, and Z constants.
4. Story 003 - snapshot rebuild with placeholders.
5. Story 004 - ghost preview bridge for Hand UI.
6. Story 005 - placement reveal path.
7. Story 006 - resolution queue after Combat Story 011 event-log contract completion.
8. Story 008 - objective reveal/HUD fanout after transport contract lands.
9. Story 011 - spawn range highlights after NP-006 and BLS-012 land.
10. Story 010 - narrowed baseline performance and CI guard evidence once the visible path exists.
11. Story 012 - Sprint 6 S6-03 browser/WASM baseline screenshot and frame-time evidence after BOARD-010.

## Next Step

BR-011 is ready for `/dev-story` after NP-006 and BLS-012 completion. Story 012 remains ready for Sprint 6 S6-03 browser/WASM baseline evidence. Story 013 is Complete (Sprint 15 Should Have; closed via PROMPT 1009 on top of PROMPT 1006 integration `08f389b` -- the canonical board rendering spec at `docs/ux/board-rendering-spec.md` ships on `origin/main` with all 16 acceptance criteria met; folded the two Tier 2 cosmetic-capture future candidates `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` and `S11-UX-BOARD-STATUS-ICON-LEGEND-001` as sections rather than as separate Sprint 15 stories). Use `liv-bevy-018` for every Bevy `.rs` file and `liv-bevy-lightyear` for every Lightyear/networking `.rs` file.
