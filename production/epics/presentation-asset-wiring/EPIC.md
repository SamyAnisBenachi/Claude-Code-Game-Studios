# Epic: Presentation Asset Wiring

> **Layer**: Presentation
> **GDD**: Cross-surface — `hand-ui.md`, `shop-auction-ui.md`, `hud.md`, `board-rendering.md`, `game-session-system.md`
> **Architecture Module**: `client/src/asset_wiring.rs` + per-surface UI modules
> **Status**: Ready
> **Stories**: 7 asset-wiring stories (Story 001 blocks stories 002-006; Story 007 gates any Krosmaga dev-proxy usage)

## Overview

Presentation Asset Wiring owns the systematic wiring of every placeholder PNG path across all five Presentation sub-systems plus the Lobby. The invariant: every UI surface uses a named path constant and a fallback chain so that final art delivery is a single path swap in `asset_wiring.rs` — no code changes required when production art arrives.

This epic does NOT define visual design or implement UI behaviour. It wires `ImageNode::new(asset_server.load(path))` (all bevy_ui surfaces) and `Sprite { image: asset_server.load(path), .. }` (world-space board content) at the exact locations where production art will land. Fallback chain for every surface: named path → `art/ui/shared/ui_placeholder_1x1_white.png`.

The asset path convention is derived from the existing `asset_wiring.rs` patterns:

| Surface | Path prefix | Pattern |
|---------|-------------|---------|
| Card display art | `art/cards/display/` | `card_{art_id}_art_display.png` |
| Card UI chrome | `art/ui/card/` | `ui_card_frame_{rarity}_hand.png` etc. |
| Shop / Auction UI | `art/ui/shop/`, `art/ui/auction/` | `ui_shop_panel_chrome.png` etc. |
| HUD | `art/ui/hud/` | `ui_class_figurine_{class_id}.png` etc. |
| Board characters | `art/characters/` | `ui_class_{class_id}_unit_board.png` |
| Board environment | `art/board/` | `env_board_chrome_default.png` |
| Lobby | `art/ui/lobby/` | `ui_class_portrait_{class_id}.png` etc. |
| Shared fallback | `art/ui/shared/` | `ui_placeholder_1x1_white.png` |

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|------------------|-------------|
| [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md) | `PresentationPlugin` composition, `CardAtlas` shared resource, `ImageNode` for bevy_ui surfaces, `Sprite` for world-space board content, rendering boundary immutable | HIGH |

## Requirements

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-PAW-001 | `asset_wiring.rs` exports per-surface path constants and `PlaceholderAssets` resource loaded before any sub-system; fallback chain defined for all 6 surfaces | ADR-021 |
| TR-PAW-002 | Hand UI card frame, stat badges, rarity icons, and class/type icons wired with `ImageNode::new()`; fallback on `CardDisplayArtFallback` | ADR-021 |
| TR-PAW-003 | Shop panel chrome, 5 slot well backgrounds, and 3 bid button chrome states wired with `ImageNode::new()` | ADR-021 |
| TR-PAW-004 | HUD class figurines (7 variants), phase timer bar, and 4 objective dot states wired with `ImageNode::new()` | ADR-021 |
| TR-PAW-005 | Board unit `Sprite.image` wired per `ClassId` as world-space sprite; fallback to `UNIT_PLACEHOLDER_ASSET`; board chrome path constants | ADR-021 |
| TR-PAW-006 | Lobby class portraits (7 variants), player slot panel backgrounds, and room code chip wired with `ImageNode::new()` | ADR-021 |
| TR-PAW-007 | Dev-only Krosmaga proxy pack/provenance boundary: logical asset IDs, three-axis provenance taxonomy, release scan failure on dev proxies, no Krosmaga files under `assets/**` | ADR-021 |

## Traceability Notes

Story 001 is cross-surface infrastructure; TR-PAW-001 covers the foundation resource and all path constants. Stories 002–006 are surface-specific and trace to the surface's primary GDD. All stories reference ADR-021 as the sole governing ADR — ADR-021 is the authority on `ImageNode` vs `Sprite` boundary and path constant organisation.

## Dependency Map

| Dependency | Surface |
|------------|---------|
| **Story 001 must be Done before Stories 002–006** | Provides `PlaceholderAssets` resource and all path constants |
| Hand UI (`client/src/ui/hand/`) | Story 002 |
| Shop/Auction UI (`client/src/ui/shop_auction/`) | Story 003 |
| HUD (`client/src/ui/hud/`) | Story 004 |
| Board Rendering (`client/src/presentation/board_rendering.rs`) | Story 005 |
| Lobby (`client/src/ui/lobby.rs`) | Story 006 |
| Krosmaga dev-proxy provenance boundary | Story 007 |

Stories 002–006 are independent of each other once Story 001 is Done.

## Current Implementation Gaps

- `client/src/asset_wiring.rs` currently contains only card display art resolution. No per-surface path constants or `PlaceholderAssets` resource exist.
- Hand UI card frame backgrounds, stat badge images, rarity icons, and class icons are unstyled (colour-only or absent).
- Shop/Auction UI panel chrome, slot well backgrounds, and bid button chrome are unstyled.
- HUD class figurines, phase timer bar, and objective dot state images are unstyled.
- Board unit sprites per class are absent; all units use the single `UNIT_PLACEHOLDER_ASSET` frame.
- Board chrome decorative elements have no path constant.
- Lobby class portraits, player slot panels, and room code chip are unstyled.

## Definition of Done

- All six stories Done.
- `grep -rn "UiImage" client/src/` returns zero results (forbidden pattern per control manifest).
- `cargo check -p client` passes with no warnings.
- `asset_wiring.rs` contains path constants for all 6 surfaces.
- Every surface has a fallback chain: named path → `art/ui/shared/ui_placeholder_1x1_white.png`.
- Integration tests pass for all Logic/Integration stories; UI story evidence files exist.

## Stories

| # | Story | Type | Status | TR-ID | ADR |
|---|-------|------|--------|-------|-----|
| 001 | [asset_wiring.rs Foundation and Placeholder Fallback PNGs](story-001-asset-wiring-foundation.md) | Integration | Ready | TR-PAW-001 | ADR-021 |
| 002 | [Hand UI Card Frames, Stat Badges, and Rarity/Type Icons](story-002-hand-ui-card-frames.md) | UI | Ready | TR-PAW-002 | ADR-021 |
| 003 | [Shop/Auction Panel Chrome, Slot Wells, and Bid Button Chrome](story-003-shop-auction-chrome.md) | UI | Ready | TR-PAW-003 | ADR-021 |
| 004 | [HUD Class Figurines, Phase Timer Bar, and Objective Dot States](story-004-hud-figurines-timer-dots.md) | UI | Ready | TR-PAW-004 | ADR-021 |
| 005 | [Board Unit Sprites Per Class and Board Chrome](story-005-board-unit-sprites.md) | Integration | Ready | TR-PAW-005 | ADR-021 |
| 006 | [Lobby Class Portraits, Player Slot Panels, and Room Code Chip](story-006-lobby-portraits.md) | UI | Ready | TR-PAW-006 | ADR-021 |
| 007 | [Dev-Only Krosmaga Proxy Pack + Provenance Boundary](story-007-krosmaga-dev-proxy-pack-boundary.md) | Docs + Tooling | Draft -- future Sprint 18 candidate (`S18-PAW-KROSMAGA-DEV-PROXY-PACK-BOUNDARY-001`; PROMPT 1280), NOT activated | TR-PAW-007 | ADR-021 |

## Next Step

Run `/story-readiness production/epics/presentation-asset-wiring/story-001-asset-wiring-foundation.md` before assigning Story 001 to a Codex worker. Stories 002–006 become assignable after Story 001 is Done; they can run in parallel.
