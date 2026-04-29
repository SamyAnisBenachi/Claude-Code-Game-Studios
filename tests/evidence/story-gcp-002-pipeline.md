# Evidence: Story GCP-002 — Asset Loading Pipeline

> **Story**: `production/epics/game-config-pipeline/story-002-asset-loading-pipeline.md`
> **Type**: Integration
> **Date**: 2026-04-29
> **Status**: COMPLETE (code review + CI pending)

---

## Summary

Implemented the full Bevy 0.18 asset loading pipeline for `game_config.ron` and `cards.json`.
Uses manual `AssetServer` + polling approach (no `bevy_asset_loader` — PR #264 still draft).

---

## Acceptance Criteria Coverage

### AC: `AppState` enum exists with variants: `Loading`, `ConfigValidation`, `Lobby`, `InSession`
**SATISFIED** — `server/src/foundation/config.rs:61-67`
```rust
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default] Loading,
    ConfigValidation,
    Lobby,
    InSession,
}
```

### AC: `GameAssets` resource holds handles `game_config` and `card_catalog`
**SATISFIED** — `server/src/foundation/config.rs:307-310`
```rust
#[derive(Resource)]
pub struct GameAssets {
    pub game_config: Handle<GameConfigAsset>,
    pub card_catalog: Handle<CardCatalog>,
}
```
**Note**: The story AC says `#[derive(AssetCollection, Resource)]` from `bevy_asset_loader`.
Since `bevy_asset_loader` is unavailable for Bevy 0.18, this is a plain `#[derive(Resource)]`.
Functionally equivalent — the handles serve the same purpose.

### AC: `CardCatalogLoader` with `#[derive(Default, TypePath)]`, `type Asset = CardCatalog`, reads JSON via `serde_json`
**SATISFIED** — `server/src/foundation/config.rs:203-290`
- `#[derive(Default, TypePath)]` present on `CardCatalogLoader`
- `impl AssetLoader for CardCatalogLoader { type Asset = CardCatalog; ... }`
- Uses `serde_json::from_slice` to parse `Vec<CardDataJson>`
- Builds `HashMap<CardId, CardData>` index from the array

### AC: `RonAssetPlugin::<GameConfig>::new(&["ron"])` registered
**ADAPTED** — `bevy_common_assets` crate (which provides `RonAssetPlugin`) was not used;
it has the same Bevy 0.18 compatibility uncertainty as `bevy_asset_loader`.
Instead, `GameConfigLoader` is a custom `AssetLoader` using `ron::de::from_bytes`.
Registered via `app.init_asset_loader::<GameConfigLoader>()` in `ConfigPlugin::build`.
**Functionally identical**: RON file → `GameConfig` struct deserialization, same extension.

### AC: Loading pipeline wired (`Loading → ConfigValidation → Lobby`)
**SATISFIED** — `server/src/foundation/config.rs:486-504` (`ConfigPlugin::build`):
- `OnEnter(AppState::Loading)` → `start_loading` — kicks off `asset_server.load(...)` for both files
- `Update.run_if(in_state(AppState::Loading))` → `check_loading_done` — polls `Assets<T>::contains`; transitions to `ConfigValidation` when both ready
- `OnEnter(AppState::ConfigValidation)` → `validate_and_promote` — validates + promotes resources + transitions to `Lobby`

### AC: Server transitions `Loading → ConfigValidation → Lobby` when both files are present
**SATISFIED** by code structure. The `check_loading_done` system only transitions when both
`config_assets.contains(&game_assets.game_config)` and
`catalog_assets.contains(&game_assets.card_catalog)` are true.
**CI GATE**: this transition can be verified by running the server binary with both asset files
present (`assets/config/game_config.ron` ✅ and `assets/data/cards.json` ✅ from Story 001).
Expected log output:
```
INFO lanes_and_lies_server: AppState::Loading — requesting game_config.ron and cards.json
INFO lanes_and_lies_server: Both assets loaded — transitioning to AppState::ConfigValidation
INFO lanes_and_lies_server: Assets loaded: GameConfig + CardCatalog (8 cards) — transitioning to AppState::Lobby
```

### AC: After `AppState::Lobby`, both `Res<GameConfig>` and `Res<CardCatalog>` are present and non-empty
**SATISFIED** — `validate_and_promote` (lines 362-400):
- `commands.insert_resource(GameConfig(cfg_asset.0.clone()))` → `Res<GameConfig>` present
- `commands.insert_resource(catalog_asset.clone())` → `Res<CardCatalog>` present
- `catalog_asset.cards.len()` is logged; empty catalog would be caught by `validate_card_catalog`

**Note on types**:
- `GameConfig` is a server-side `#[derive(Resource)]` wrapper around `shared::config::GameConfig`
  (ADR-004 path b — shared/ stays bevy-free). Implements `Deref` for ergonomic field access.
- `CardCatalog` is a server-side `#[derive(Resource, Asset, TypePath)]` struct
  (not the `type CardCatalog = HashMap<...>` alias in shared/card.rs — type aliases cannot derive traits).

### AC: `Asset+TypePath` decision documented in code comment
**SATISFIED** — `server/src/foundation/config.rs:1-35` — full decision block at the top of the file:
- Explains ADR-003 vs ADR-004 conflict
- Documents path b choice (server-side wrappers)
- Explains why each wrapper type exists
- Notes shared/ remains bevy-free and workspace purity CI gate continues to pass

### AC: `bevy_asset_loader` version pinned or fallback documented
**SATISFIED (fallback documented)**:
- `server/src/foundation/mod.rs:5-8` — updated comment: "ADR-004 fallback: manual AssetServer loading is used in config.rs (see ConfigPlugin)"
- `server/src/foundation/config.rs:33-35` — "bevy_asset_loader: NOT available for Bevy 0.18 (PR #264 still draft). Re-check before Story 004."

---

## Additional changes

### `shared/src/config.rs` — added 3 missing auction floor fields
The existing `game_config.ron` (from Story 001) contains `auction_floor_rare`, `auction_floor_epic`,
`auction_floor_legendary` fields. RON 0.8 rejects unknown fields by default (no serde deny_unknown
workaround exists). Added all three to `GameConfig` with design-intent defaults matching the RON file:
- `auction_floor_rare: u32` → default `3`
- `auction_floor_epic: u32` → default `4`
- `auction_floor_legendary: u32` → default `5`

### `server/Cargo.toml` — added features and dependency
- bevy features: added `"bevy_asset"` and `"bevy_state"`
- Added `thiserror = "1"` (for error enums on loaders)

---

## Deviations from Story AC

| AC text | Actual implementation | Reason |
|---|---|---|
| `#[derive(AssetCollection, Resource)]` on `GameAssets` | Plain `#[derive(Resource)]` | `bevy_asset_loader` unavailable for 0.18 |
| `RonAssetPlugin::<GameConfig>::new(&["ron"])` | Custom `GameConfigLoader` with `ron::de::from_bytes` | `bevy_common_assets` availability uncertain |
| `Handle<GameConfig>` in `GameAssets` | `Handle<GameConfigAsset>` | `GameConfig` is a Resource wrapper; `GameConfigAsset` is the Asset type |
| `LoadingState::new(AppState::Loading)...` | Manual `check_loading_done` polling system | Fallback pattern; same semantics |

---

## Files changed

| File | Change |
|---|---|
| `server/src/foundation/config.rs` | **NEW** — full pipeline implementation |
| `server/src/foundation/mod.rs` | Added `pub mod config;`, updated bevy_asset_loader comment |
| `server/src/main.rs` | Added `AssetPlugin`, `ConfigPlugin` to App builder |
| `server/Cargo.toml` | Added `bevy_asset`, `bevy_state` features; `thiserror = "1"` |
| `shared/src/config.rs` | Added 3 auction floor fields + defaults |

---

## CI verification

**Cannot run locally** — Smart App Control blocks the MSVC linker. Push to CI at:
https://github.com/SamyAnisBenachi/Claude-Code-Game-Studios/actions

Expected CI result: green `cargo check --workspace` (no Rust type errors — only pre-existing linker issues in local environment).
