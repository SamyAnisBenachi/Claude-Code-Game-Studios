# Story 003: Shop and Auction Panel Chrome, Slot Wells, Bid Button

> **Epic**: Presentation Asset Wiring
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-09

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**Requirement**: `TR-PAW-003`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Shop and Auction panels are screen-space `bevy_ui` — chrome surfaces (panel backgrounds, slot wells, bid button) use `ImageNode`. `Sprite` is forbidden for these surfaces.

**Engine**: Bevy 0.18 | **Risk**: MEDIUM
**Engine Notes**: Bevy 0.18 Required Components — `ImageNode { image: handle, .. }` spawned directly. Path constants come from `client/src/asset_wiring.rs`; no inline string literals in shop/auction UI spawn code.

**Control Manifest Rules (Presentation Layer)**:
- Required: Shop/Auction chrome uses `ImageNode` — never `Sprite`
- Required: Path constants from `asset_wiring.rs` (panel chrome, slot well, auction panel, bid button)
- Forbidden: `NodeBundle` / `ImageBundle` (do not exist in Bevy 0.18)
- Forbidden: `Sprite` for shop/auction UI surfaces

---

## Acceptance Criteria

*From TR-PAW-003 (scoped to shop/auction panel chrome, slot wells, bid button):*

- [x] **PAW-003-a**: The shop panel root spawns with an `ImageNode` chrome layer wired through `PlaceholderAssets` to the shop panel chrome path constant. Evidence: integration commit `792a9d8` (`client/src/ui/shop_auction/mod.rs`).
- [x] **PAW-003-b**: Each shop slot spawns a slot-well `ImageNode` child wired to the slot-well path constant. Evidence: integration commit `792a9d8`.
- [x] **PAW-003-c**: The auction panel root spawns with an `ImageNode` chrome layer wired to the auction panel background path constant; border ramp tiles are wired via the existing path constants. Evidence: integration commit `792a9d8`.
- [x] **PAW-003-d**: The bid button spawns with an `ImageNode` chrome layer wired to the bid button chrome path constant. Evidence: integration commit `792a9d8`.
- [x] **PAW-003-e**: Integration test asserts every shop/auction chrome surface has a non-empty `ImageNode.image` handle (not a default) and that no `Sprite` component appears on shop/auction UI entities. Evidence: `tests/integration/presentation/shop_auction_asset_wiring_test.rs` (introduced in `792a9d8`).

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/presentation/shop_auction_asset_wiring_test.rs`
**Status**: [x] Created and passing in integration commit `792a9d8`. Test file added under `client/Cargo.toml [[test]]` entry.

**Accept-risk waiver**: None — automated test exists.

---

## Tech Debt

*Quality items deferred per friend-game scope (logged as accept-risk):*

- **PAW-TD-003-a**: Shop/Auction panel chrome, slot wells, and bid button currently load 1×1 px placeholder PNGs through `PlaceholderAssets` rather than final art. Accept-risk for friend-game scope; not a public-release-readiness or final-art completion claim.
- **PAW-TD-003-b**: No browser/native manual visual capture of shop/auction chrome; coverage is automated handle-resolution only. Accept-risk for friend-game scope; manual visual evidence is owned by future QA passes.

---

## Dependencies

- Depends on: Story 001 (`asset_wiring.rs` foundation, `PlaceholderAssets` resource) — Done
- Unlocks: None (leaf story in this epic)
