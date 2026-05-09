# Story 005: Board Unit Sprites Per Class and Board Chrome

> **Epic**: Presentation Asset Wiring
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-09

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-PAW-005`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Board content (units, objectives, prisms, HP bars, spawn range) is ALWAYS world-space 2D — `Sprite` + `Transform` with `Camera2d`. **`ImageNode` must NOT be used for board content.** The rendering boundary is immutable: world-space sprites cannot appear above bevy_ui without custom render layers.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: Bevy 0.18 Required Components API — no `SpriteBundle`. Spawn board unit sprites individually: `commands.spawn((Sprite { image: handle, .. }, Transform::from_xyz(x, y, z)))`. `Handle<TextureAtlas>` as an asset type does not exist in 0.18 — use `Sprite.texture_atlas: Option<TextureAtlas>` for atlas-based sprites, or a plain `Handle<Image>` on `Sprite.image` for standalone images.

**Control Manifest Rules (Presentation Layer)**:
- Required: Board content uses world-space `Sprite` + `Camera2d` — never `ImageNode`
- Required: Path constants from `asset_wiring.rs` — no inline strings in board system code
- Forbidden: `SpriteBundle` (does not exist in Bevy 0.18)
- Forbidden: `Handle<TextureAtlas>` as an asset type (deprecated in 0.15)
- Guardrail: Board steady-state < 1 ms/frame; sprite batching via same atlas per class

---

## Acceptance Criteria

*From TR-PAW-005 (scoped to board unit sprites and chrome):*

- [x] **PAW-005-a**: When a `BoardUnit` entity is spawned for a unit whose `ClassId` has a dedicated sprite, `Sprite.image` is set to `asset_server.load(board_unit_asset(class_id))`. The class is derived from `BoardUnitSourceClass(class_id)` on the entity. Evidence: integration commit `7782c6f` (`client/src/presentation/board_rendering.rs`); merge `ece5f48`.
- [x] **PAW-005-b**: When `CardAtlas::unit_frame(card_id)` returns `None` (no atlas frame for this card) AND no per-class sprite is available, `Sprite.image` falls back to the existing `UNIT_PLACEHOLDER_ASSET` path (unchanged from current board rendering). Class-based sprites take priority over the atlas fallback. Evidence: integration commit `7782c6f`.
- [x] **PAW-005-c**: A board chrome entity exists (spawned on `OnEnter(ClientState::InSession)`) with `Sprite.image` loaded from `BOARD_CHROME_ASSET`. Chrome renders at a Z value between the board background (Z `0.0`) and unit entities (Z `3.0`–`5.0`). Evidence: integration commit `7782c6f` (`client/src/presentation/board_rendering/rendering_constants.rs` 1-line diff adds Z constant).
- [x] **PAW-005-d**: An integration test spawns a `BoardUnit` entity with `BoardUnitSourceClass(ClassId::Iop)` and asserts `Sprite.image` resolves to the Iop-class asset handle (not the placeholder). Evidence: `tests/integration/presentation/board_asset_wiring_test.rs` (introduced in `7782c6f`).
- [x] **PAW-005-e**: An integration test spawns a `BoardUnit` entity with no matching class (or `ClassId::Neutral` if neutral has no dedicated sprite) and asserts `Sprite.image` resolves to `UNIT_PLACEHOLDER_ASSET` (the fallback path, not an empty handle). Evidence: `tests/integration/presentation/board_asset_wiring_test.rs`.
- [x] **PAW-005-f**: `cargo check -p client` passes; no `ImageNode` used for board unit or board chrome entities. Evidence: integration commit `7782c6f` (no `ImageNode` introduced for board surfaces).

---

## Implementation Notes

*Derived from ADR-021 Implementation Guidelines:*

**World-space Sprite pattern (Bevy 0.18).** Do not use `SpriteBundle` — it does not exist. Use Required Components:

```rust
// Class-keyed per-unit sprite
let image_handle = asset_server.load(board_unit_asset(source_class));
commands.spawn((
    Sprite {
        image: image_handle,
        // Atlas-based frame overrides this field — leave as default for standalone images
        ..default()
    },
    Transform::from_xyz(world_x, world_y, 3.0),
    BoardUnit { unit_id },
    BoardUnitSourceClass(source_class),
    // ... other game-state components
));
```

**Priority logic.** In the unit spawn/update system, apply this priority:
1. If `CardAtlas.unit_frames` has an entry for `card_id` → use `Sprite.texture_atlas` (existing atlas frame path, unchanged).
2. Else if `board_unit_asset(source_class)` is not `PLACEHOLDER_FALLBACK_ASSET` → use `Sprite.image = asset_server.load(board_unit_asset(source_class))`.
3. Else → use `Sprite.image = asset_server.load(UNIT_PLACEHOLDER_ASSET)`.

This preserves the existing atlas-frame path for cards that have atlas entries; class-based sprites are only for cards without atlas entries. Final art for a class = add atlas frame OR update the class constant — either path works.

**Board chrome entity.** Spawn as a sibling to the board background. Z=`1.5` places it above `BOARD_BACKGROUND_ASSET` (Z `0.0`) but below board cells (Z `2.0`) if that ordering is established in `rendering_constants.rs`. Confirm against existing Z constants before hardcoding:

```rust
commands.spawn((
    Sprite {
        image: asset_server.load(BOARD_CHROME_ASSET),
        ..default()
    },
    Transform::from_xyz(0.0, 0.0, BOARD_CHROME_Z),  // define const in rendering_constants.rs
    BoardRenderingEntity,
));
```

**Selector function.** `board_unit_asset(ClassId) -> &'static str` is defined in Story 001. All 7 variants return a dedicated path; `ClassId::Neutral` may return `UNIT_PLACEHOLDER_ASSET` if no neutral-specific art is planned.

---

## In Scope

- Create 1×1 px placeholder PNGs at the 7 class-unit paths (`art/characters/ui_class_{iop,cra,sacrier,xelor,ecaflip,sadida,neutral}_unit_board.png`) and `art/board/env_board_chrome_default.png`.

## Out of Scope

- Atlas frame index wiring — handled by `CardAtlas.unit_frames` (existing board rendering).
- HP bar images — already wired via `HP_BAR_WHITE_PIXEL_ASSET` (existing).
- Objective, prism, spawn range sprites — existing paths in `board_rendering.rs` constants.
- Board cell idle/active state images — existing paths.

---

## QA Test Cases

*Story type: Integration — automated test specs.*

- **AC PAW-005-d**: Class-keyed sprite on unit spawn
  - Given: A test `App` with `ClientState::InSession`, `BoardRuntimeAssets` and `PlaceholderAssets` inserted
  - When: A `BoardUnit` entity is spawned with `BoardUnitSourceClass(ClassId::Iop)` and no atlas frame entry in `CardAtlas.unit_frames` for the card ID
  - Then: The entity's `Sprite.image` handle resolves to the path `art/characters/ui_class_iop_unit_board.png`
  - Edge cases: Test all 7 class variants to confirm no class falls through to placeholder unexpectedly

- **AC PAW-005-e**: Fallback to UNIT_PLACEHOLDER_ASSET
  - Given: Same setup as above
  - When: A `BoardUnit` entity is spawned with a class that has no dedicated sprite (or with an atlas frame entry that overrides the class path)
  - Then: `Sprite.image` resolves to `art/characters/ui_unit_placeholder_default_board.png` (the existing placeholder constant, not an empty handle)
  - Edge cases: Entity must have `Sprite.image` set (non-default) in all branches; no silent empty handle

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/presentation/board_asset_wiring_test.rs`
**Status**: [x] Created and passing in integration commit `7782c6f`; merged on main at `ece5f48`. Test file added under `client/Cargo.toml [[test]]` entry (148-line test file).

**Accept-risk waiver**: None — automated test exists.

---

## Tech Debt

*Quality items deferred per friend-game scope (logged as accept-risk):*

- **PAW-TD-005-a**: Board unit sprites and board chrome currently load 1×1 px placeholder PNGs at the 7 class-unit paths and `BOARD_CHROME_ASSET` rather than final art. Accept-risk for friend-game scope; not a public-release-readiness or final-art completion claim.
- **PAW-TD-005-b**: No browser/native manual visual capture of board unit sprite Z-ordering and class-keyed art; coverage is automated handle-resolution only. Accept-risk for friend-game scope; manual visual evidence is owned by future QA passes.

---

## Dependencies

- Depends on: Story 001 (`asset_wiring.rs` Foundation) — Done
- Unlocks: None (leaf story in this epic)
