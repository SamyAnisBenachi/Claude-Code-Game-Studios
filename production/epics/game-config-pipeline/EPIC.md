# Epic: GameConfig & CardCatalog Loading Pipeline

> **Layer**: Foundation
> **GDD**: design/gdd/game-config.md · design/gdd/card-data-pool.md
> **Architecture Module**: `server/foundation/config.rs` + `server/foundation/assets/`
> **Status**: Ready
> **Stories**: 4 stories created — see table below

## Overview

Implements the server-side asset loading pipeline that reads `assets/config/game_config.ron` and `assets/data/cards.json` at startup, validates both against domain invariants, and promotes them to ECS resources (`Res<GameConfig>` and `Res<CardCatalog>`) before the server enters `AppState::Lobby`. Load failure is always fatal — the server must not start with missing or malformed config. A debug-only hot-reload watcher for `GameConfig` enables balance tuning without server restart. This epic owns no game logic and no shared types; those live in `shared/` (Epic 1). It wires the loading infrastructure only.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-004: Asset Loading Pipeline | Single `LoadingState` loads both `GameConfig` + `CardCatalog`; `validate_and_promote` system gates `AppState::Lobby` entry; hot-reload debug-only | MEDIUM |
| ADR-003: Cargo Workspace Structure | Server crate owns loading code; `GameConfig` struct definition lives in `shared/` | MEDIUM |

## GDD Requirements

> Note: `docs/architecture/tr-registry.yaml` has not yet been populated. TR-IDs below are informal references from the ADR "GDD Requirements Addressed" sections. Run `/architecture-review` to register stable IDs before stories are written.

| Informal TR-ID | Requirement | ADR Coverage |
|----------------|-------------|--------------|
| TR-GC-01 | All tuning knobs loaded from `assets/config/game_config.ron`; no hardcoded values in systems | ADR-004 ✅ |
| TR-GC-02 | Server-authoritative config; clients never read the file directly | ADR-004 ✅ |
| TR-GC-03 | Load failure aborts startup — no silent fallback to defaults | ADR-004 ✅ |
| TR-GC-04 | Dangerous-value validation (Rule 5): shop_weight_cap, fake_count, objective_hp, timers | ADR-004 ✅ |
| TR-GC-05 | Debug hot-reload supported; re-validates before applying; rejects invalid edits without crash | ADR-004 ✅ |
| TR-CDP-01 | Card data loaded from `assets/data/cards.json` via custom AssetLoader | ADR-004 ✅ |
| TR-CDP-02 | Duplicate CardIds are a fatal error at startup | ADR-004 ✅ |
| TR-CDP-07 | `CardCatalog` is immutable for server lifetime (no hot-reload) | ADR-004 ✅ |
| TR-CDP-09 | `pool_copies_override <= 0` is a soft error — warn and fall back to rarity default, do not abort | ADR-004 ✅ |

## Scope

### Deliverables

**Asset files**
- `assets/config/game_config.ron` — all fields from `design/gdd/game-config.md` Section G (Tuning Knobs) with their design-intent defaults
- `assets/data/cards.json` — starter fixture: minimum representative set covering Common/Uncommon/Rare/Epic/Legendary rarities and at least two ClassIds, sufficient to exercise all validation paths and pool initialization tests

**Server loading pipeline** (`server/src/foundation/` or `server/src/assets/`)
- `AppState` enum: `Loading`, `ConfigValidation`, `Lobby`, `InSession`
- `GameAssets` collection (`#[derive(AssetCollection, Resource)]`): handles for both `game_config` and `card_catalog`
- `RonAssetPlugin::<GameConfig>::new(&["ron"])` — idiomatic 0.18 RON asset loading
- `CardCatalogLoader` — custom `AssetLoader` impl; `#[derive(Default, TypePath)]` required by Bevy 0.18; reads `cards.json` via `serde_json`
- `validate_and_promote` system (runs `OnEnter(AppState::ConfigValidation)`):
  - Reads `GameConfig` asset and runs `validate_game_config()`
  - Reads `CardCatalog` asset and runs `validate_card_catalog()`
  - On any failure: `exit.write(AppExit::error())` — never `panic!`
  - On success: `commands.insert_resource(cfg.clone())`, `commands.insert_resource(cat.clone())`, transition to `AppState::Lobby`
- `validate_game_config(c: &GameConfig) -> Result<(), String>` — Rule 5 dangerous-value checks: `shop_weight_cap ∈ (0.0, 1.0)`, `shop_weight_per_card < shop_weight_cap`, `fake_count ∈ [1, 3]`, `objective_hp ≥ 1`, `placement_timer_seconds ≥ 1`
- `validate_card_catalog(c: &CardCatalog) -> Result<(), String>` — key-matches-id check; empty-catalog check
- `#[cfg(debug_assertions)]` hot-reload watcher (`hot_reload_game_config`): on `AssetEvent::<GameConfig>::Modified`, re-validate and replace resource or log warning and keep prior. `CardCatalog` is intentionally NOT hot-reloaded.

**Implementation note: GameConfig derive**
`GameConfig` lives in `shared/src/config.rs` (Epic 1 deliverable) with plain serde derives only. For asset loading, `Asset + TypePath` are needed. Two options: (a) add them to the shared struct by extending the bevy feature in `shared/Cargo.toml` — this is what ADR-004's code sample shows; (b) create a thin server-side wrapper. Resolve at implementation time; the CI gate for `shared/` dependency tree will reveal the correct path.

**Unit tests** (`server/tests/` or `tests/unit/foundation/`)
- `validate_game_config` — one passing case (valid config); one failing case per Rule 5 check (at minimum: invalid `shop_weight_cap`, invalid `fake_count`)
- `validate_card_catalog` — passing case (valid catalog); failing case: empty catalog; failing case: key-mismatch
- `pool_copies_override <= 0` — confirm soft error (warn emitted, server does not abort, rarity default used)

**Release-build verification** (explicit deliverable)
- `cargo build --release` + confirm `hot_reload_game_config` system is absent from symbols (e.g. via `nm` or `cargo bloat`). Document pass/fail in `tests/evidence/`.

## Definition of Done

- Server reaches `AppState::Lobby` with `Res<GameConfig>` and `Res<CardCatalog>` present and non-empty
- Deleting `game_config.ron` causes server to exit non-zero with a clear error log
- Deliberately invalid `shop_weight_cap = 1.5` causes fatal validation error with descriptive message
- Duplicate CardId in `cards.json` causes fatal validation error
- `pool_copies_override = -1` on one card produces a `warn!` but does NOT abort startup
- All `validate_game_config` and `validate_card_catalog` unit tests pass
- Debug hot-reload: edit + save `game_config.ron` produces `info!` "hot-reloaded successfully"; invalid edit produces `warn!` "rejected (kept previous)"
- Release build: `hot_reload_game_config` system absent from binary symbols (documented evidence in `tests/evidence/`)
- All Logic story evidence in `tests/unit/foundation/`

## Stories

| # | Story | Type | Status | ADR |
|---|-------|------|--------|-----|
| 001 | [Asset Data Files](story-001-asset-data-files.md) | Config/Data | Ready | ADR-004 |
| 002 | [Asset Loading Pipeline](story-002-asset-loading-pipeline.md) | Integration | Ready | ADR-004 |
| 003 | [Startup Validation Gate](story-003-startup-validation-gate.md) | Logic | Ready | ADR-004 |
| 004 | [Debug Hot-Reload & Release Verification](story-004-debug-hot-reload.md) | Integration | Ready | ADR-004 |

> Story sequence: 001 → 002 → 003 → 004 (linear chain).

## Next Step

Run `/story-readiness production/epics/game-config-pipeline/story-001-asset-data-files.md` before starting implementation.
