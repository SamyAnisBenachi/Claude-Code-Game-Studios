# Story 002: Hand UI Card Frames, Stat Badges, Rarity/Type Icons

> **Epic**: Presentation Asset Wiring
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-09

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-PAW-002`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Hand UI is screen-space `bevy_ui` — `ImageNode` is the correct surface for card frames, stat badges, and rarity/type icons. `Sprite` is forbidden for hand UI.

**Engine**: Bevy 0.18 | **Risk**: MEDIUM
**Engine Notes**: Bevy 0.18 Required Components — no `NodeBundle` / `ImageBundle`. Spawn `ImageNode { image: handle, .. }` directly with `Node` and other layout components. Path constants come from `client/src/asset_wiring.rs`; no inline string literals in hand UI spawn code.

**Control Manifest Rules (Presentation Layer)**:
- Required: Hand UI chrome uses `ImageNode` — never `Sprite`
- Required: Path constants from `asset_wiring.rs` and the `rarity_icon_asset()` selector — no inline strings
- Required: Spawn ordering — `spawn_hand_ui.after(insert_placeholder_assets)` so `PlaceholderAssets` is available
- Forbidden: `NodeBundle` / `ImageBundle` (do not exist in Bevy 0.18)
- Forbidden: `Sprite` for hand UI surfaces

---

## Acceptance Criteria

*From TR-PAW-002 (scoped to hand UI card frames, stat badges, rarity/type icons):*

- [x] **PAW-002-a**: A `HandCardFrame` chrome child is spawned for every hand slot with `ImageNode.image` resolved through `PlaceholderAssets`. Evidence: integration commit `40a9f72` (`client/src/ui/hand/mod.rs`); merge `69a03cc`.
- [x] **PAW-002-b**: Stat badge children (`StatBadgeAtk`, `StatBadgeHp`, `StatBadgeMana`) spawn with their respective `ImageNode` handles wired to placeholder assets and update on card arrival. Evidence: integration commit `40a9f72`.
- [x] **PAW-002-c**: A `HandRarityIcon` child uses the `rarity_icon_asset(rarity)` selector added to `client/src/asset_wiring.rs`; a `HandTypeIcon` child uses the type-icon path constant. Evidence: integration commit `40a9f72` (10-line `asset_wiring.rs` diff adds `rarity_icon_asset`).
- [x] **PAW-002-d**: `sync_fan_slot_chrome_system` updates chrome `ImageNode` handles when a hand card arrives or falls back. Evidence: integration commit `40a9f72`.
- [x] **PAW-002-e**: System ordering `spawn_hand_ui.after(insert_placeholder_assets)` is registered so `PlaceholderAssets` is always available before spawn. Evidence: integration commit `40a9f72`; covered by `tests/integration/presentation/hand_ui_asset_wiring_test.rs`.
- [x] **PAW-002-f**: Integration test asserts every spawned hand slot has the 7 chrome marker components and that `ImageNode.image` is a non-empty handle (not a default). Evidence: `tests/integration/presentation/hand_ui_asset_wiring_test.rs` (introduced in `40a9f72`).

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/presentation/hand_ui_asset_wiring_test.rs`
**Status**: [x] Created and passing in integration commit `40a9f72`; merged on main at `69a03cc`. Test file added under `client/Cargo.toml [[test]]` entry.

**Accept-risk waiver**: None — automated test exists.

---

## Tech Debt

*Quality items deferred per friend-game scope (logged as accept-risk):*

- **PAW-TD-002-a**: Hand UI card frames, stat badges, and rarity/type icons currently load 1×1 px placeholder PNGs through `PlaceholderAssets` rather than final art. Accept-risk for friend-game scope; not a public-release-readiness or final-art completion claim.
- **PAW-TD-002-b**: No browser/native manual visual capture of hand UI chrome; coverage is automated handle-resolution only. Accept-risk for friend-game scope; manual visual evidence is owned by future QA passes.

---

## Dependencies

- Depends on: Story 001 (`asset_wiring.rs` foundation, `PlaceholderAssets` resource) — Done
- Unlocks: None (leaf story in this epic)
