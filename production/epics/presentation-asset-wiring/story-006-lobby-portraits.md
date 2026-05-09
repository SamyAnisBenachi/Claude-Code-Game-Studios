# Story 006: Lobby Class Portraits, Slot Panels, Room Code Chip

> **Epic**: Presentation Asset Wiring
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-09

## Context

**GDD**: `design/gdd/game-session-system.md`
**Requirement**: `TR-PAW-006`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Lobby is screen-space `bevy_ui` — class portraits, player slot panel state surfaces, and the room code chip use `ImageNode`. `Sprite` is forbidden for lobby surfaces.

**Engine**: Bevy 0.18 | **Risk**: MEDIUM
**Engine Notes**: Bevy 0.18 Required Components — `ImageNode { image: handle, .. }` spawned directly. Path constants come from `client/src/asset_wiring.rs` and class-keyed selectors. Lobby spawn does not depend on game-state ECS resources beyond `PlaceholderAssets`.

**Control Manifest Rules (Presentation Layer)**:
- Required: Lobby chrome uses `ImageNode` — never `Sprite`
- Required: Path constants from `asset_wiring.rs` (lobby class portrait, slot panel states, room code chip)
- Forbidden: `Sprite` for lobby UI surfaces

---

## Acceptance Criteria

*From TR-PAW-006 (scoped to lobby class portraits, player slot panel states, room code chip):*

- [x] **PAW-006-a**: Each class-portrait button in the lobby spawns a chrome `ImageNode` child wired through `PlaceholderAssets` to the lobby class portrait path constant for that class. Evidence: integration commit `724470e` (`client/src/ui/lobby.rs`); merge `bb80b47`.
- [x] **PAW-006-b**: Each player slot panel spawns with an `ImageNode` chrome layer wired to the slot-panel state path constant. Evidence: integration commit `724470e`.
- [x] **PAW-006-c**: The room code chip spawns with an `ImageNode` chrome wired to the room-code chip path constant. Evidence: integration commit `724470e`.
- [x] **PAW-006-d**: Integration test asserts every lobby chrome surface has a non-empty `ImageNode.image` handle (not a default) and that no `Sprite` component appears on lobby UI entities. Evidence: `tests/integration/presentation/lobby_asset_wiring_test.rs` (introduced in `724470e`, 281-line test file).

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/presentation/lobby_asset_wiring_test.rs`
**Status**: [x] Created and passing in integration commit `724470e`; merged on main at `bb80b47`. Test file added under `client/Cargo.toml [[test]]` entry.

**Accept-risk waiver**: None — automated test exists.

---

## Tech Debt

*Quality items deferred per friend-game scope (logged as accept-risk):*

- **PAW-TD-006-a**: Lobby class portraits, slot panel states, and the room code chip currently load 1×1 px placeholder PNGs through `PlaceholderAssets` rather than final art. Accept-risk for friend-game scope; not a public-release-readiness or final-art completion claim.
- **PAW-TD-006-b**: No browser/native manual visual capture of lobby chrome; coverage is automated handle-resolution only. Accept-risk for friend-game scope; manual visual evidence is owned by future QA passes.

---

## Dependencies

- Depends on: Story 001 (`asset_wiring.rs` foundation, `PlaceholderAssets` resource) — Done
- Unlocks: None (leaf story in this epic)
