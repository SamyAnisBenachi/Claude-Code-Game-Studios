# Story 004: HUD Figurines, Phase Timer Bar, Objective Dots

> **Epic**: Presentation Asset Wiring
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-09

## Context

**GDD**: `design/gdd/hud.md`
**Requirement**: `TR-PAW-004`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: HUD is screen-space `bevy_ui` — class figurines (own/opponent), the phase timer bar, and objective dot state sprites use `ImageNode`. `Sprite` is forbidden for HUD surfaces.

**Engine**: Bevy 0.18 | **Risk**: MEDIUM
**Engine Notes**: Bevy 0.18 Required Components — `ImageNode { image: handle, .. }` spawned directly. Sync systems update handles in response to `PresentationGameSnapshotMessage` (figurine class) and `HudObjectiveUpdate` (dot DESTROYED state). Path constants come from `client/src/asset_wiring.rs`.

**Control Manifest Rules (Presentation Layer)**:
- Required: HUD chrome uses `ImageNode` — never `Sprite`
- Required: Path constants from `asset_wiring.rs` (HUD figurine fallback, phase timer bar, objective dot states)
- Required: Figurine handle synced from authoritative `PresentationGameSnapshotMessage` class
- Required: Objective dot handle synced on `HudObjectiveUpdate` to DESTROYED state
- Forbidden: `Sprite` for HUD surfaces

---

## Acceptance Criteria

*From TR-PAW-004 (scoped to HUD class figurines, phase timer bar, objective dot state sprites):*

- [x] **PAW-004-a**: Two `HudFigurine` marker entities (own + opponent) spawn at HUD pool initialization with `ImageNode.image` wired to `PlaceholderAssets.fallback`. Evidence: integration commit `a7e397a` (`client/src/ui/hud/mod.rs`); merge `2132129`.
- [x] **PAW-004-b**: One `HudTimerBar` marker entity spawns at HUD pool initialization with `ImageNode.image` wired to `HUD_PHASE_TIMER_BAR_ASSET`. Evidence: integration commit `a7e397a`; `client/src/asset_wiring.rs` 5-line diff.
- [x] **PAW-004-c**: 10 objective dot `ImageNode` entities spawn at HUD pool initialization — 5 own row start as Alive state, 5 opponent row start as Unknown state. Evidence: integration commit `a7e397a`.
- [x] **PAW-004-d**: `sync_figurine_image_system` updates own and opponent figurine `ImageNode.image` to the class-specific path on `PresentationGameSnapshotMessage`. Evidence: integration commit `a7e397a`; covered in `tests/integration/presentation/hud_asset_wiring_test.rs`.
- [x] **PAW-004-e**: `sync_dot_image_on_objective_destroyed_system` updates the corresponding objective dot `ImageNode.image` to the DESTROYED state on `HudObjectiveUpdate`. Evidence: integration commit `a7e397a`; covered in `tests/integration/presentation/hud_asset_wiring_test.rs`.
- [x] **PAW-004-f**: `HUD_ENTITY_COUNT` increased from 19 → 21 to account for `HudFigurine` and `HudTimerBar`; foundation test updated. Evidence: integration commit `a7e397a` (`tests/integration/presentation/asset_wiring_foundation_test.rs` 25-line diff).
- [x] **PAW-004-g**: Integration test (6 sub-tests) covers PAW-004-d, PAW-004-e, marker presence, and edge cases. Evidence: `tests/integration/presentation/hud_asset_wiring_test.rs` (introduced in `a7e397a`).

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/presentation/hud_asset_wiring_test.rs` (and updated `asset_wiring_foundation_test.rs`)
**Status**: [x] Created and passing in integration commit `a7e397a`; merged on main at `2132129`. Test file added under `client/Cargo.toml [[test]]` entry.

**Accept-risk waiver**: None — automated test exists.

---

## Tech Debt

*Quality items deferred per friend-game scope (logged as accept-risk):*

- **PAW-TD-004-a**: HUD figurines, phase timer bar, and objective dot state sprites currently load 1×1 px placeholder PNGs through `PlaceholderAssets` and HUD path constants rather than final art. Accept-risk for friend-game scope; not a public-release-readiness or final-art completion claim.
- **PAW-TD-004-b**: Objective dot opponent row starts as Unknown and only flips to DESTROYED on authoritative `HudObjectiveUpdate`; no Alive/Unknown disambiguation art exists per design. Accept-risk for friend-game scope.
- **PAW-TD-004-c**: No browser/native manual visual capture of HUD chrome; coverage is automated handle-resolution only. Accept-risk for friend-game scope; manual visual evidence is owned by future QA passes.

---

## Dependencies

- Depends on: Story 001 (`asset_wiring.rs` foundation, `PlaceholderAssets` resource) — Done
- Unlocks: None (leaf story in this epic)
