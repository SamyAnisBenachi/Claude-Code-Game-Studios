# Story 002: Asset Loading Pipeline

> **Epic**: GameConfig & CardCatalog Loading Pipeline
> **Status**: Ready
> **Layer**: Foundation
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/game-config.md` · `design/gdd/card-data-pool.md`
**Requirement**: TR-??? (covers TR-GC-01: loaded via bevy_asset_loader; TR-CDP-01: CardCatalog loaded via custom AssetLoader; TR-GC-02: server-authoritative)

**ADR Governing Implementation**: ADR-004: Asset Loading Pipeline
**ADR Decision Summary**: A single `bevy_asset_loader` `LoadingState` loads both `GameConfig` and `CardCatalog` in parallel. On completion, a transient `ConfigValidation` state runs `validate_and_promote`, which promotes both to ECS resources and transitions to `AppState::Lobby`. Failure in either path is always fatal.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**: Three Bevy 0.18-specific requirements: (1) custom `AssetLoader` impls must `#[derive(Default, TypePath)]`; (2) `ron` is no longer re-exported via `bevy_asset` — add as direct dep; (3) `LoadContext::path()` returns `AssetPath`, not `&Path`. Verify `bevy_asset_loader` 0.18-compatible version on crates.io — if unavailable, fall back to manual `LoadingState` (same shape, more boilerplate). Resolve the `Asset+TypePath` tension for `GameConfig` in `shared/` — document chosen path in a code comment (see Implementation Notes).

**Control Manifest Rules (Foundation layer)**:
- Required: `AssetLoader` impls must `#[derive(Default, TypePath)]` — required as of Bevy 0.18.
- Required: Add `ron = "0.8"` as a direct dep in `server/Cargo.toml`.
- Required: `CardCatalog` is immutable after load. Never mutate card definitions mid-session.
- Guardrail: `GameConfig` + `CardCatalog` load time < 100ms total at expected card count (~298 cards).

---

## Acceptance Criteria

- [ ] `AppState` enum exists with variants: `Loading`, `ConfigValidation`, `Lobby`, `InSession`
- [ ] `GameAssets` resource (`#[derive(AssetCollection, Resource)]`) holds handles: `game_config: Handle<GameConfig>` and `card_catalog: Handle<CardCatalog>`
- [ ] `RonAssetPlugin::<GameConfig>::new(&["ron"])` is registered in `server/main.rs`
- [ ] `CardCatalogLoader` struct exists with `#[derive(Default, TypePath)]` and implements `AssetLoader` with `type Asset = CardCatalog`; reads JSON via `serde_json`
- [ ] `LoadingState::new(AppState::Loading).continue_to_state(AppState::ConfigValidation).load_collection::<GameAssets>()` is wired in `server/main.rs`
- [ ] Server successfully transitions through `Loading` → `ConfigValidation` → `Lobby` when both asset files are present and valid
- [ ] After transition to `AppState::Lobby`, both `Res<GameConfig>` and `Res<CardCatalog>` are present in the ECS world and non-empty
- [ ] The `Asset+TypePath` decision for `GameConfig` in `shared/` is resolved and documented in a code comment in `server/src/foundation/config.rs` or the loader file
- [ ] `bevy_asset_loader` version is pinned in `workspace.dependencies` (or fallback to manual `LoadingState` is documented)

---

## Implementation Notes

*Derived from ADR-004 §Architecture diagram and Implementation Guidelines §1–3:*

**App wiring sketch (server/main.rs):**
```rust
fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(RonAssetPlugin::<GameConfig>::new(&["ron"]))
        .init_asset::<CardCatalog>()
        .init_asset_loader::<CardCatalogLoader>()
        .init_state::<AppState>()
        .add_loading_state(
            LoadingState::new(AppState::Loading)
                .continue_to_state(AppState::ConfigValidation)
                .load_collection::<GameAssets>(),
        )
        .add_systems(OnEnter(AppState::ConfigValidation), validate_and_promote)
        // hot-reload watcher added in Story 004
        .run();
}
```

**`CardCatalogLoader` skeleton:**
```rust
#[derive(Default, TypePath)]
pub struct CardCatalogLoader;

impl AssetLoader for CardCatalogLoader {
    type Asset = CardCatalog;
    type Settings = ();
    type Error = CardCatalogLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        // cards.json is a Vec<CardData>; convert to HashMap<CardId, CardData>
        let cards: Vec<CardData> = serde_json::from_slice(&bytes)?;
        let catalog: CardCatalog = cards.into_iter().map(|c| (c.id, c)).collect();
        Ok(catalog)
    }

    fn extensions(&self) -> &[&str] { &["json"] }
}
```

**`Asset+TypePath` decision for `GameConfig`:**
ADR-003 says `shared/` must use `bevy = { default-features = false, features = ["serialize"] }` only. ADR-004 shows `#[derive(Asset, TypePath)]` on `GameConfig`. Two valid paths:

**(a) Add `bevy_asset` feature to `shared/Cargo.toml`** (what ADR-004's code sample shows):
Add `"bevy_asset"` to the bevy features list in `shared/Cargo.toml`. Run `cargo tree -p shared` — if this pulls `bevy_ecs` or `bevy_render`, revert and use path (b).

**(b) Server-side wrapper**: Keep `GameConfig` in `shared/` as pure serde. Create `GameConfigAsset` in `server/src/foundation/config.rs` that wraps it with `#[derive(Asset, TypePath)]`. After loading, extract the inner `GameConfig` and insert it as `Res<GameConfig>`.

Document which path was taken in a comment in `server/src/foundation/config.rs`. The CI gate from workspace-and-shared-types Story 004 will fail if path (a) pulls disallowed crates into `shared/`.

**`validate_and_promote` stub (Story 003 implements the real validation):**
In this story, wire a minimal stub that reads both handles and transitions to `AppState::Lobby` unconditionally. Story 003 replaces this stub with real validation.

---

## Out of Scope

- Story 001: The data files themselves (`game_config.ron`, `cards.json`)
- Story 003: `validate_game_config()`, `validate_card_catalog()`, fatal exit paths — this story only wires the loading; validation is a stub
- Story 004: Debug hot-reload watcher

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **AC: Server reaches Lobby with both resources**
  - Given: Valid `game_config.ron` and `cards.json` present (Story 001 done)
  - When: Server binary starts and runs until `AppState::Lobby` is entered
  - Then: `world.contains_resource::<GameConfig>()` returns `true`; `world.contains_resource::<CardCatalog>()` returns `true`; catalog is non-empty (`catalog.len() > 0`)

- **AC: Missing file causes non-zero exit**
  - Given: `game_config.ron` is deleted/absent
  - When: Server binary starts
  - Then: Server exits with non-zero status code; log contains a message referencing the missing path
  - Note: This AC is partially covered here (bevy_asset_loader handles missing file) and fully verified in Story 003 (validate_and_promote)

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: Integration test or manual startup log showing state transitions and both resources present → `tests/evidence/story-gcp-002-pipeline.md`
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (asset files) + `workspace-and-shared-types` Story 004 (workspace fully scaffolded with CI gates passing)
- Unlocks: Story 003 (validation gate)
