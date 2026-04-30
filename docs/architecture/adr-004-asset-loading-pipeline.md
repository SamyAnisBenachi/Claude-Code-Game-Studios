# ADR-004: Asset Loading Pipeline — GameConfig and CardCatalog

## Status

Accepted

## Date

2026-04-29

## Last Verified

2026-04-29

## Decision Makers

User + technical-director (architecture), gameplay-programmer (config consumers),
network-programmer (server lifecycle integration)

## Summary

Lanes and Lies needs deterministic, server-authoritative loading of two structured
data files (`GameConfig` tuning knobs and the `CardCatalog` card pool) before any
session can start. We adopt a single `bevy_asset_loader` `LoadingState` that loads
both assets at server startup, validates them, promotes them to ECS resources, and
gates the transition into `AppState::Lobby` — failing fatally on any structural or
range error.

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 |
| **Domain** | Core / Asset Pipeline |
| **Knowledge Risk** | MEDIUM — three 0.18-era API details must be respected (custom `AssetLoader` requires `#[derive(TypePath)]`; `ron` is no longer re-exported via `bevy_asset` and must be a direct crate dependency; `LoadContext::path()` returns `AssetPath`, not `&Path`). |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `docs/engine-reference/bevy/breaking-changes.md`, `docs/engine-reference/bevy/deprecated-apis.md`, `bevy_asset_loader` README (0.18-compatible release) |
| **Post-Cutoff APIs Used** | `bevy_asset_loader` 0.18 `LoadingState` API, `AssetLoader` trait shape (0.18), `TypePath` derive on loader structs, `AssetEvent::<T>::Modified` for hot-reload watchers |
| **Verification Required** | (1) Confirm `LoadingState` transitions correctly when both `GameConfig` and `CardCatalog` complete loading. (2) Confirm a missing or malformed RON/JSON file produces a fatal load error (not a silent default). (3) Confirm hot-reload re-validation in debug builds rejects an invalid edit without crashing the server. |

> **Note**: Knowledge Risk is MEDIUM. If the project upgrades past Bevy 0.18, this
> ADR must be re-validated — the `AssetLoader` trait and `bevy_asset_loader`
> integration both shifted across recent versions.

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-003 (workspace structure — server crate exists, `shared/` crate hosts the cross-cutting `GameConfig` struct) |
| **Enables** | ADR-006 (Card Pool schema — `CardCatalog` is the on-disk source of truth that the pool implementation will consume) |
| **Blocks** | All systems that read `Res<GameConfig>` (economy, auction, shop weighting, fake-objective placement, placement timer); Card Pool implementation (cannot proceed without `CardCatalog` loaded) |
| **Ordering Note** | Must be Accepted before any system that takes `Res<GameConfig>` or `Res<CardCatalog>` as a parameter is implemented. Test fixtures may instantiate these resources directly without the loader, but production server code path must go through `LoadingState`. |

## Context

### Problem Statement

The server is the single source of truth for all gameplay tuning (economy curves,
shop weighting, fake-objective counts, timers) and for the card pool itself. We
need a startup pipeline that:

1. Loads two structured files (`game_config.ron`, `cards.json`) from disk.
2. Validates them against domain invariants documented in the GDD.
3. Inserts them as ECS resources for systems to query.
4. Refuses to start a session if either file is missing, malformed, or violates
   invariants — silent fallback to defaults would mask balance regressions and
   produce non-deterministic playtests.

The decision must be made now because: (a) every gameplay system that reads tuning
values is blocked behind a stable `Res<GameConfig>` contract; (b) the Card Pool
implementation (ADR-006, pending) needs to know whether `CardCatalog` is mutable
or immutable, and how it relates to `PlayerPool` session state; (c) `bevy_asset_loader`
configuration is global to the app and must be set up once — retrofitting it
later forces every system to be rewritten to use the new resource types.

### Current State

No asset pipeline exists yet. The `GameConfig` struct and `CardCatalog` map are
referenced throughout the GDDs (`game-config.md`, `card-data-pool.md`) but have
no implementation. Without a loading contract, every consumer would invent its
own ad-hoc way to read config (file::read, hardcoded defaults, etc.), which would
break determinism and deployment.

### Constraints

- **Bevy 0.18 API surface**: custom `AssetLoader` impls require `#[derive(TypePath)]`;
  `ron` is no longer re-exported via `bevy_asset` (must be a direct dep);
  `LoadContext::path()` returns `AssetPath` not `&Path`.
- **Server-authoritative architecture**: all tuning must be the server's truth.
  Clients never read `game_config.ron` directly — any value a client needs is
  delivered via Lightyear protocol messages.
- **WASM compatibility**: `bevy_asset_loader` and `bevy_asset` work on WASM, but
  the server is native-only. The asset pipeline runs on the server binary.
- **Determinism**: reload must be impossible mid-session. Tuning changes only
  take effect at server restart (debug hot-reload is dev-only).
- **Failure must be loud**: a missing file, parse error, or invariant violation
  must abort startup. No "load with defaults and continue."

### Requirements

- **R1**: Both `GameConfig` and `CardCatalog` must be loaded and validated before
  the server transitions out of `AppState::Loading`.
- **R2**: After load, both must be available as `Res<T>` to all systems.
- **R3**: Invariant violations (per GDD `game-config.md` Rule 5 dangerous-value
  checks; per `card-data-pool.md` TR-CDP-02 duplicate-CardId check) must be
  fatal.
- **R4**: Soft errors (per-card `pool_copies_override <= 0`) must log a warning
  and fall back to the rarity default — they must not abort startup.
- **R5**: Debug-only hot-reload of `GameConfig` must be supported, must
  re-validate before applying, and must reject invalid reloads without crashing
  the server.
- **R6**: Load time impact must be negligible (< 100 ms total for both files at
  expected sizes — `game_config.ron` ~2 KB, `cards.json` ~50–200 KB depending
  on card count).

## Decision

We adopt a **single `LoadingState` at server startup** that loads two assets via
`bevy_asset_loader` typed collections, validates them, promotes them to ECS
resources, and gates the `AppState::Loading` → `AppState::Lobby` transition.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Server Process Startup                      │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
                    ┌────────────────────────┐
                    │  AppState::Loading     │
                    │  (initial state)       │
                    └────────────────────────┘
                                 │
              ┌──────────────────┴──────────────────┐
              ▼                                     ▼
   ┌──────────────────────┐              ┌──────────────────────┐
   │ assets/config/       │              │ assets/data/         │
   │   game_config.ron    │              │   cards.json         │
   │                      │              │                      │
   │ RonAssetPlugin       │              │ CardCatalogLoader    │
   │ <GameConfig>         │              │ (custom AssetLoader) │
   └──────────┬───────────┘              └──────────┬───────────┘
              │ asset handle                         │ asset handle
              ▼                                     ▼
   ┌──────────────────────────────────────────────────────────┐
   │ LoadingStateConfig (bevy_asset_loader)                    │
   │   .load_collection::<GameAssets>()                        │
   │   .continue_to_state(AppState::ConfigValidation)          │
   └──────────┬───────────────────────────────────────────────┘
              │ both assets fully loaded
              ▼
   ┌──────────────────────────────────────────────────────────┐
   │ AppState::ConfigValidation (transient — runs once)        │
   │                                                           │
   │   System: validate_and_promote                            │
   │     1. Read GameConfig asset; run dangerous-value checks. │
   │     2. Read CardCatalog asset; check duplicate CardIds.   │
   │     3. On fail → AppExit::error() (fatal).                │
   │     4. On pass → app.insert_resource(game_config.clone()) │
   │                  app.insert_resource(card_catalog.clone())│
   │     5. Transition to AppState::Lobby.                     │
   └──────────┬───────────────────────────────────────────────┘
              │
              ▼
   ┌──────────────────────────────────────────────────────────┐
   │ AppState::Lobby                                           │
   │   Res<GameConfig>   — read by every gameplay system       │
   │   Res<CardCatalog>  — read by Card Pool init              │
   │                                                           │
   │   PlayerPool is built from CardCatalog + GameConfig at    │
   │   session start (NOT loaded from file — see ADR-006).     │
   └──────────────────────────────────────────────────────────┘

   [Debug builds only]
   ┌──────────────────────────────────────────────────────────┐
   │ System: hot_reload_game_config                            │
   │   On AssetEvent::<GameConfig>::Modified:                  │
   │     1. Re-run dangerous-value validation.                 │
   │     2. On pass → re-insert Res<GameConfig> (replace).     │
   │     3. On fail → log warning, keep previous resource.     │
   └──────────────────────────────────────────────────────────┘
```

### Key Interfaces

The struct definitions below are the contracts every consumer must respect.
`GameConfig` lives in `shared/` (since both server and integration tests construct
it); `CardCatalog` lives in `server/` (server-only — never shipped to client).

```rust
// ─────────────────────────────────────────────────────────────────
// shared/src/config.rs
// ─────────────────────────────────────────────────────────────────
//
// IMPORTANT: GameConfig does NOT derive Resource here. The asset
// loader produces it as an Asset; the server promotes it to Resource
// at validation time by cloning into the world. This keeps shared/
// free of bevy_ecs dependencies that test code does not need.

use bevy::asset::Asset;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

#[derive(Asset, TypePath, Serialize, Deserialize, Debug, Clone)]
pub struct GameConfig {
    // Economy
    pub starting_gold: u32,
    pub interest_cap: u32,
    pub interest_rate: f32,
    pub round_income_base: u32,

    // Shop weighting (see game-config.md Rule 5)
    pub shop_weight_per_card: f32,    // must be > 0.0
    pub shop_weight_cap: f32,         // must be in (0.0, 1.0)

    // Objectives (see game-config.md Rule 5)
    pub fake_count: u8,               // must be in [1, 3]
    pub objective_hp: u32,            // must be >= 1

    // Timers
    pub placement_timer_seconds: u32, // must be >= 1
    pub auction_timer_seconds: u32,
    pub draft_timer_seconds: u32,

    // ... (additional knobs per game-config.md)
}

// ─────────────────────────────────────────────────────────────────
// server/src/assets/card_catalog.rs
// ─────────────────────────────────────────────────────────────────
//
// IMPORTANT: CardCatalog is IMMUTABLE for the server lifetime.
// PlayerPool (mutable, session-scoped) is built FROM this at session
// start — see ADR-006.

use bevy::asset::Asset;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type CardId = String; // stable string ID, see card-data-pool.md

#[derive(Asset, TypePath, Serialize, Deserialize, Debug, Clone)]
pub struct CardCatalog {
    pub cards: HashMap<CardId, CardData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardData {
    pub id: CardId,
    pub name: String,
    pub rarity: Rarity,
    pub cost: u32,
    pub stats: CardStats,
    /// Optional override for copies in the pool. <= 0 is a soft error
    /// (warn + use rarity default). Missing field = use rarity default.
    pub pool_copies_override: Option<i32>,
    // ... (additional fields per card-data-pool.md schema)
}

// ─────────────────────────────────────────────────────────────────
// server/src/assets/loader.rs
// ─────────────────────────────────────────────────────────────────
//
// Custom AssetLoader for cards.json. Bevy 0.18 requires TypePath on
// loader structs.

use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use thiserror::Error;

#[derive(Default, TypePath)]
pub struct CardCatalogLoader;

#[derive(Debug, Error)]
pub enum CardCatalogLoadError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json parse error: {0}")]
    Parse(#[from] serde_json::Error),
}

impl AssetLoader for CardCatalogLoader {
    type Asset = CardCatalog;
    type Settings = ();
    type Error = CardCatalogLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let catalog: CardCatalog = serde_json::from_slice(&bytes)?;
        Ok(catalog)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

// ─────────────────────────────────────────────────────────────────
// server/src/assets/mod.rs
// ─────────────────────────────────────────────────────────────────
//
// bevy_asset_loader collection — both handles loaded as one batch.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "config/game_config.ron")]
    pub game_config: Handle<GameConfig>,

    #[asset(path = "data/cards.json")]
    pub card_catalog: Handle<CardCatalog>,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Loading,
    ConfigValidation,
    Lobby,
    InSession,
}
```

### Implementation Guidelines

**1. Cargo dependencies (server crate)**

Bevy 0.18 dropped the `bevy_asset` re-export of `ron`. Add it as a direct dep:

```toml
# server/Cargo.toml
[dependencies]
bevy = "0.18"
bevy_asset_loader = { version = "<0.18-compatible>", features = ["standard_dynamic_assets"] }
bevy_common_assets = { version = "<0.18-compatible>", features = ["ron"] } # provides RonAssetPlugin
ron = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
```

`bevy_common_assets::ron::RonAssetPlugin::<GameConfig>::new(&["ron"])` is the
idiomatic 0.18 path for RON-backed assets. Do NOT hand-roll a RON loader — the
plugin handles `TypePath` and registration correctly.

**2. App wiring (server `main.rs`)**

```rust
fn main() {
    App::new()
        .add_plugins(MinimalPlugins)              // headless server — no DefaultPlugins
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
        .add_systems(
            OnEnter(AppState::ConfigValidation),
            validate_and_promote,
        )
        // Hot-reload watcher — DEBUG ONLY.
        .add_systems(
            Update,
            hot_reload_game_config
                .run_if(in_state(AppState::Lobby).or(in_state(AppState::InSession))),
        )
        .run();
}
```

The `hot_reload_game_config` system body must itself be `#[cfg(debug_assertions)]`,
or the system can be conditionally added only in debug builds:

```rust
#[cfg(debug_assertions)]
{ app.add_systems(Update, hot_reload_game_config); }
```

The latter is preferred because it removes the system entirely from release
builds (no scheduler cost).

**3. Validation system (`validate_and_promote`)**

This system runs ONCE on entry to `AppState::ConfigValidation`. It must:

- Read `GameConfig` from `Assets<GameConfig>` via the handle in `GameAssets`.
- Run all dangerous-value checks (see Rule 5 below). On any failure, log the
  full reason and call `app_exit_events.write(AppExit::error())`. Do NOT
  `panic!` — `AppExit::error()` lets Bevy shut cleanly and surface a non-zero
  exit code to the deployment platform.
- Read `CardCatalog`. Check that no two `CardData.id` values collide
  (`HashMap` deserialisation already enforces unique keys at the JSON level,
  but if the schema uses an array form anywhere, validate explicitly).
- Per-card `pool_copies_override <= 0`: log `warn!("card {id}: pool_copies_override
  {n} is invalid; falling back to rarity default")`, do NOT abort.
- On full success: `commands.insert_resource(game_config.clone())`,
  `commands.insert_resource(card_catalog.clone())`, then transition state via
  `next_state.set(AppState::Lobby)`.

```rust
fn validate_and_promote(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    configs: Res<Assets<GameConfig>>,
    catalogs: Res<Assets<CardCatalog>>,
    mut next_state: ResMut<NextState<AppState>>,
    // Bevy 0.17+ — AppExit is dispatched via MessageWriter<AppExit> (Message/Event split).
    // Verify exact system param name against liv-bevy-018 skill api_patterns.md before
    // implementing. Alternative: use std::process::exit(1) for fatal startup errors.
    mut app_exit: MessageWriter<AppExit>,
) {
    let Some(cfg) = configs.get(&game_assets.game_config) else {
        error!("GameConfig handle did not resolve to an asset");
        app_exit.write(AppExit::Error(NonZeroU8::MIN));
        return;
    };
    if let Err(e) = validate_game_config(cfg) {
        error!("GameConfig validation failed: {e}");
        app_exit.write(AppExit::Error(NonZeroU8::MIN));
        return;
    }

    let Some(cat) = catalogs.get(&game_assets.card_catalog) else {
        error!("CardCatalog handle did not resolve to an asset");
        app_exit.write(AppExit::Error(NonZeroU8::MIN));
        return;
    };
    if let Err(e) = validate_card_catalog(cat) {
        error!("CardCatalog validation failed: {e}");
        app_exit.write(AppExit::Error(NonZeroU8::MIN));
        return;
    }

    commands.insert_resource(cfg.clone());
    commands.insert_resource(cat.clone());
    info!("Assets loaded: GameConfig + CardCatalog ({} cards)", cat.cards.len());
    next_state.set(AppState::Lobby);
}
```

**4. Dangerous-value checks (Rule 5 from `game-config.md`)**

Validation is intentionally conservative — only values that would break gameplay
or produce divide-by-zero / infinite-loop behaviour are checked. Balance values
(e.g. is `interest_rate = 0.05` "right"?) are NOT validated; that is a designer
concern, not a load-time concern.

```rust
pub fn validate_game_config(c: &GameConfig) -> Result<(), String> {
    if !(0.0 < c.shop_weight_cap && c.shop_weight_cap < 1.0) {
        return Err(format!(
            "shop_weight_cap must be in (0.0, 1.0); got {}", c.shop_weight_cap
        ));
    }
    if !(c.shop_weight_per_card < c.shop_weight_cap) {
        return Err(format!(
            "shop_weight_per_card ({}) must be < shop_weight_cap ({})",
            c.shop_weight_per_card, c.shop_weight_cap
        ));
    }
    if !(1..=3).contains(&c.fake_count) {
        return Err(format!("fake_count must be in [1, 3]; got {}", c.fake_count));
    }
    if c.objective_hp < 1 {
        return Err("objective_hp must be >= 1".into());
    }
    if c.placement_timer_seconds < 1 {
        return Err("placement_timer_seconds must be >= 1".into());
    }
    Ok(())
}
```

**5. Card catalog validation (TR-CDP-02 duplicates)**

If `cards.json` is a `HashMap<CardId, CardData>`, duplicate keys are caught by
`serde_json` (last write wins, but with a strict deserialiser configuration it
errors). Belt-and-braces: also assert each `CardData.id == map_key`:

```rust
pub fn validate_card_catalog(c: &CardCatalog) -> Result<(), String> {
    for (key, card) in &c.cards {
        if key != &card.id {
            return Err(format!(
                "CardCatalog key '{key}' does not match CardData.id '{}'",
                card.id
            ));
        }
    }
    if c.cards.is_empty() {
        return Err("CardCatalog is empty — no cards to draft".into());
    }
    Ok(())
}
```

If the JSON schema is changed to an array form later, add an explicit duplicate
check at that point.

**6. Hot-reload watcher (debug-only)**

```rust
// Bevy 0.18: AssetEvent<T> uses the Observer pattern, not buffered EventReader.
// Register this function as: app.observe(hot_reload_game_config)
// The handler receives On<AssetEvent<T>> as its trigger parameter.
// Verify exact Observer trigger type against liv-bevy-018 skill api_patterns.md.
#[cfg(debug_assertions)]
fn hot_reload_game_config(
    trigger: On<AssetEvent<GameConfig>>,
    game_assets: Res<GameAssets>,
    configs: Res<Assets<GameConfig>>,
    mut commands: Commands,
) {
    let AssetEvent::Modified { id } = trigger.event() else { return; };
    if *id != game_assets.game_config.id() { return; }

    let Some(new_cfg) = configs.get(&game_assets.game_config) else { return; };
    match validate_game_config(new_cfg) {
        Ok(()) => {
            commands.insert_resource(new_cfg.clone());
            info!("GameConfig hot-reloaded successfully");
        }
        Err(e) => {
            warn!("GameConfig hot-reload rejected (kept previous): {e}");
        }
    }
}
```

`CardCatalog` is intentionally NOT hot-reloaded. It is the source of `PlayerPool`,
and reloading mid-session would corrupt the pool. Card data changes require a
server restart.

**7. Test fixtures**

Tests construct `GameConfig` directly via `GameConfig { ... }` and insert it
into the test `World` with `world.insert_resource(cfg)`. They do NOT go through
the loader. This is why `GameConfig` lives in `shared/` and does not derive
`Resource` (the server-side promotion path inserts the cloned asset as a
resource without needing a derive — `commands.insert_resource(T)` works on any
`T: Send + Sync + 'static`).

If a future Bevy version makes `insert_resource` require `T: Resource`, add
`#[derive(Resource)]` at that point and re-export — this is a cheap,
backwards-compatible change.

## Alternatives Considered

### Alternative 1: Synchronous `std::fs::read_to_string` at startup

- **Description**: Skip `bevy_asset_loader` entirely. Read both files synchronously
  in `main()` before building the `App`, parse them, and `app.insert_resource(...)`
  before any system runs.
- **Pros**: Simpler — no `LoadingState`, no `AssetLoader` impl, fewer crates.
  Failure path is just `process::exit(1)`.
- **Cons**: No hot-reload (couldn't add it later without rewriting the whole
  pipeline). No path mapping through `AssetServer`, so file paths are duplicated
  between code and asset config. Inconsistent with how the WASM client loads its
  own assets (the project uses `bevy_asset_loader` everywhere else, per
  `technical-preferences.md`).
- **Estimated Effort**: -30% vs chosen.
- **Rejection Reason**: We lose the unified asset-loading story across server
  and client, and we lose hot-reload (which is a meaningful dev productivity
  feature for tuning balance — designers want to edit `game_config.ron` and see
  the effect within seconds, not after a server rebuild).

### Alternative 2: Lazy / on-demand load at first-use

- **Description**: Don't load anything at startup. The first system that needs
  `GameConfig` triggers the load.
- **Pros**: Fastest possible startup if some sessions never need certain assets.
- **Cons**: Validation happens at unpredictable times — a malformed config could
  be detected mid-session, with no graceful failure path. Multiple systems would
  race to load, requiring locking. Determinism is destroyed.
- **Estimated Effort**: ~same.
- **Rejection Reason**: Catastrophic for a server-authoritative game. Failures
  must be detected at startup, not 10 minutes into a tournament.

### Alternative 3: Two separate loading states

- **Description**: One `LoadingState` for `GameConfig`, then transition to a
  second `LoadingState` for `CardCatalog`.
- **Pros**: If `CardCatalog` ever becomes optional (e.g. a tutorial mode that
  doesn't need cards), the staged design accommodates it.
- **Cons**: More state machine surface for no current benefit. Both files are
  required for any session.
- **Estimated Effort**: +10% vs chosen.
- **Rejection Reason**: YAGNI. Single `LoadingState` with a `load_collection`
  is the documented `bevy_asset_loader` pattern for "load N things in parallel
  before continuing."

### Alternative 4: Embed assets at compile time via `include_bytes!`

- **Description**: `include_bytes!("../assets/config/game_config.ron")` and
  parse at startup.
- **Pros**: No filesystem dependency in the deployed binary. No path resolution.
- **Cons**: Every balance change requires a rebuild and redeploy. No hot-reload.
  No designer-driven tuning workflow. Locks the asset format into the binary.
- **Estimated Effort**: -20% vs chosen.
- **Rejection Reason**: Defeats the purpose of having data-driven config —
  `coding-standards.md` requires "Gameplay values must be data-driven (external
  config), never hardcoded," and `include_bytes!` is hardcoding with extra
  steps.

## Consequences

### Positive

- Single, documented loading contract for both critical data files. New systems
  add `Res<GameConfig>` or `Res<CardCatalog>` to their signature and the
  guarantee is "this resource exists, has been validated, and is non-empty."
- Failures are loud and early: server fails to start rather than producing
  silently-wrong gameplay.
- `bevy_asset_loader` integration matches the WASM client's pipeline, keeping
  one mental model across the project.
- Debug hot-reload accelerates balance iteration — designers can edit
  `game_config.ron` and see the effect on the next round without a rebuild.
- Test fixtures bypass the loader cleanly, so unit tests don't need a filesystem.
- Validation is centralised in `validate_game_config` / `validate_card_catalog`
  — one place to audit invariants.

### Negative

- Three crates added to the server binary (`bevy_asset_loader`,
  `bevy_common_assets`, `ron` direct dep). Estimated +1–2 MB to the server
  binary (negligible on Railway).
- One transient app state (`ConfigValidation`) that is structurally noise but
  semantically valuable — without it, validation would have to interleave with
  `LoadingState` exit, which is awkward.
- `GameConfig` lives in `shared/` but the loading code lives in `server/`.
  Contributors editing the struct must remember the loader registration is
  separate.
- Custom `AssetLoader` for `CardCatalog` is ~30 lines of boilerplate the
  contributor must maintain when the JSON schema evolves. (Mitigation: the
  loader is dumb — it just deserialises. All schema work is in `CardData`.)

### Neutral

- Hot-reload is debug-only, so production behaviour is "load once, immutable"
  — same as Alternative 4, just with the file on disk.
- `CardCatalog` size grows linearly with card count; at projected ~150 cards,
  this is ~50 KB and ~5 ms to parse — well under any budget.

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| `bevy_asset_loader` 0.18-compatible release lags or has API drift | Low | High | Verify version on crates.io before committing Cargo.toml; if missing, fall back to manual `LoadingState` (same shape, slightly more boilerplate). |
| `ron` direct-dep version mismatches Bevy's internal expectations | Low | Medium | Pin `ron = "0.8"`. If conflict surfaces, follow Bevy 0.18 release notes for the supported version. |
| `TypePath` derive missing on `CardCatalogLoader` produces a confusing runtime error | Medium | Medium | Lint rule / code review checklist: every `impl AssetLoader` struct must `#[derive(Default, TypePath)]`. |
| Validation logic drifts from GDD `game-config.md` Rule 5 over time | Medium | High | Validation function lives next to `GameConfig` struct in `shared/`; PR template requires updating it when fields change. `/architecture-review` audit checks this. |
| Hot-reload watcher accidentally enabled in release build, causing nondeterminism in production | Low | High | Gate the `add_systems` call itself behind `#[cfg(debug_assertions)]` (not just the function body) — this guarantees the system is not even compiled into release builds. |
| Card pool author edits `cards.json` mid-development with malformed JSON, blocking other contributors | Medium | Low | Loud failure surfaces immediately; the bad commit is obvious. CI smoke test: `cargo run --bin server -- --validate-only` exits 0 if assets parse, non-zero otherwise. |

## Performance Implications

| Metric | Before | Expected After | Budget |
|--------|--------|----------------|--------|
| CPU (frame time) | n/a (system not yet implemented) | 0 ms steady-state (loader runs once at startup) | <2 ms game logic budget — unaffected |
| Memory | n/a | ~60 KB (GameConfig + CardCatalog cloned twice — once as Asset, once as Resource) | <256 MB WASM heap (server has no equivalent budget; trivial) |
| Load Time | n/a | <100 ms for both files at expected sizes | <5 s server cold start (Railway) |
| Network (if applicable) | n/a | 0 — all server-side, never sent to client | n/a |

The double-storage of the asset (held by `Assets<T>` and re-inserted as
`Res<T>`) is intentional. Removing the asset after promotion would save ~60 KB
but break hot-reload (which queries `Assets<GameConfig>` for the new value).
60 KB is well below any threshold worth worrying about.

## Migration Plan

This is a foundational ADR — there is no existing system to migrate.
Implementation order:

1. **Step 1 — Crate setup**: Add the three new dependencies to `server/Cargo.toml`
   and `shared/Cargo.toml` (where applicable). Verify `cargo build` succeeds.
2. **Step 2 — Skeleton structs**: Land `GameConfig` (in `shared/`) and
   `CardCatalog` + `CardData` (in `server/`) with minimal fields. Tests can
   construct these directly.
3. **Step 3 — Loader registration**: Wire `RonAssetPlugin::<GameConfig>` and
   `init_asset_loader::<CardCatalogLoader>` in `server/main.rs`. Add the
   `LoadingState` and `AppState`. Verify the server reaches `AppState::Lobby`
   with empty assets present.
4. **Step 4 — Validation**: Implement `validate_game_config` and
   `validate_card_catalog`. Add unit tests for each invariant (one passing
   case + one failing case per rule).
5. **Step 5 — Real assets**: Author `assets/config/game_config.ron` and
   `assets/data/cards.json` with the GDD's starting values. Verify load + validation
   succeeds. Verify a deliberately-broken file aborts startup with a clear log.
6. **Step 6 — Hot-reload (debug-only)**: Add the `hot_reload_game_config`
   system behind `#[cfg(debug_assertions)]`. Verify edit-save cycle works in a
   debug build; verify the system is absent from a release build (`cargo build
   --release`; inspect symbols).
7. **Step 7 — Documentation cascade**: Update `card-data-pool.md` and
   `game-config.md` Implementation Notes to reference this ADR. Add stable TR
   IDs to `tr-registry.yaml`.

**Rollback plan**: If `bevy_asset_loader` 0.18 integration proves unworkable,
fall back to Alternative 1 (synchronous `std::fs::read_to_string` at startup).
The validation functions, struct definitions, and resource contracts are
unchanged — only the loader plumbing differs. Estimated rollback: 1 day.

## Validation Criteria

- [ ] Server reaches `AppState::Lobby` with `Res<GameConfig>` and
      `Res<CardCatalog>` present and non-empty.
- [ ] Deleting `assets/config/game_config.ron` causes the server to exit with
      non-zero status and a clear error log line referencing the missing path.
- [ ] Deliberately setting `shop_weight_cap = 1.5` in the RON file causes the
      server to exit with `GameConfig validation failed: shop_weight_cap must
      be in (0.0, 1.0); got 1.5`.
- [ ] Deliberately introducing a duplicate-CardId in `cards.json` (via the
      array-form schema, if used) causes a fatal validation error.
- [ ] Setting one card's `pool_copies_override` to `-1` produces a `warn!`
      log line but does NOT abort startup.
- [ ] In a debug build, editing `game_config.ron` while the server is running
      produces an `info!` line "GameConfig hot-reloaded successfully" and
      systems pick up the new value on the next frame.
- [ ] In a debug build, editing `game_config.ron` to an invalid value produces
      a `warn!` line "GameConfig hot-reload rejected (kept previous)" and the
      server continues with the prior valid config.
- [ ] In a release build (`cargo build --release`), the
      `hot_reload_game_config` system is not present in the binary symbols.
- [ ] Total load time (server cold start to `AppState::Lobby`) is under 1
      second for the production-sized catalog.

## GDD Requirements Addressed

| GDD Document | System | Requirement | How This ADR Satisfies It |
|--------------|--------|-------------|---------------------------|
| `design/gdd/game-config.md` | GameConfig | TR-GC-01: All tuning knobs loaded from external config | `game_config.ron` is the single source; no hardcoded values in systems. |
| `design/gdd/game-config.md` | GameConfig | TR-GC-02: Server-authoritative config (clients never read the file) | `GameConfig` resource lives only on the server; client receives derived values via Lightyear messages. |
| `design/gdd/game-config.md` | GameConfig | TR-GC-03: Load failure aborts startup (no defaults) | `validate_and_promote` calls `AppExit::error()` on any failure path. |
| `design/gdd/game-config.md` | GameConfig | TR-GC-04: Dangerous-value validation (Rule 5) | `validate_game_config` enforces the documented ranges; failure is fatal. |
| `design/gdd/game-config.md` | GameConfig | TR-GC-05: Debug hot-reload supported | `hot_reload_game_config` runs in debug builds, re-validates before applying. |
| `design/gdd/card-data-pool.md` | Card Pool | TR-CDP-01: Card data loaded from external file | `cards.json` is the source; loaded via custom `AssetLoader` at startup. |
| `design/gdd/card-data-pool.md` | Card Pool | TR-CDP-02: Duplicate CardIds are a fatal error | `validate_card_catalog` enforces `key == card.id`; HashMap form rejects duplicates at deserialisation. |
| `design/gdd/card-data-pool.md` | Card Pool | TR-CDP-07: `CardCatalog` is immutable for server lifetime | No hot-reload; `Res<CardCatalog>` inserted once and never replaced. |
| `design/gdd/card-data-pool.md` | Card Pool | TR-CDP-09: `pool_copies_override <= 0` is a soft error | `validate_card_catalog` (or per-card init) logs a warning and falls back to rarity default. |

## Related

- `docs/architecture/adr-001-objective-identity-unicast.md` — independent decision; no overlap.
- ADR-003 (workspace structure) — prerequisite. `shared/` and `server/` crates must exist before this loader can be wired up.
- ADR-006 (Card Pool schema) — depends on this. `PlayerPool` is built from `CardCatalog` + `GameConfig` at session start.
- `design/gdd/game-config.md` — Rules 1–6, especially Rule 5 dangerous-value checks.
- `design/gdd/card-data-pool.md` — TR-CDP-01, TR-CDP-02, TR-CDP-07, TR-CDP-09.
- `docs/engine-reference/bevy/VERSION.md` — Bevy 0.18 pin and migration notes.
- `.claude/docs/technical-preferences.md` — `bevy_asset_loader` listed in allowed-libraries table; "no hardcoded balance values" forbidden pattern.
- Code (once implemented): `shared/src/config.rs`, `server/src/assets/{mod.rs,card_catalog.rs,loader.rs,validation.rs}`, `server/src/main.rs` app wiring.
