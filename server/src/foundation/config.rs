// server/src/foundation/config.rs — Asset loading pipeline (ADR-004)
//
// ARCHITECTURE DECISION — Asset+TypePath for GameConfig (ADR-003 vs ADR-004):
//
//   ADR-004 shows `#[derive(Asset, TypePath)]` on GameConfig in shared/.
//   ADR-003 strictly forbids Bevy deps in shared/ — the crate compiles with only
//   serde; pulling bevy_asset into shared/ would drag bevy_ecs into the shared crate
//   and break the workspace purity CI gate.
//
//   Decision (ADR-004 §Implementation Guidelines, path b): create server-side wrappers.
//
//   - `GameConfigAsset`: wraps shared::config::GameConfig, derives Asset+TypePath.
//     Used only during the loading phase; discarded after validation.
//   - `GameConfig`: server-only Resource that wraps shared::config::GameConfig.
//     Inserted into ECS world after validation. Implements Deref for ergonomic
//     field access (`config.starting_gold` instead of `config.0.starting_gold`).
//   - `CardCatalog`: server-only struct (NOT the type alias in shared/card.rs).
//     The shared CardCatalog is `type CardCatalog = HashMap<CardId, CardData>` —
//     type aliases cannot derive Asset/Resource. This server struct holds the same
//     data in a newtype-wrapping struct and derives Asset+TypePath+Resource.
//
//   shared/ remains entirely bevy-free. The workspace purity CI gate continues to pass.
//
// PIPELINE:
//   AppState::Loading
//     ↓  start_loading (OnEnter)  — kicks off AssetServer::load for both assets
//     ↓  check_loading_done (Update while in Loading) — polls Assets<T>::contains
//   AppState::ConfigValidation
//     ↓  validate_and_promote (OnEnter) — validates, inserts resources, transitions
//   AppState::Lobby
//     — Res<GameConfig> and Res<CardCatalog> are available to all systems
//
// bevy_asset_loader: NOT available for Bevy 0.18 (PR #264 still draft as of 2026-04-29).
// Manual loading approach is used instead (same semantics, more boilerplate).
// Re-check crates.io before Story 004 (hot-reload watcher).

use std::collections::HashMap;
use std::ops::Deref;

use bevy::asset::{
    io::Reader, Asset, AssetApp, AssetLoader, AssetServer, Assets, Handle, LoadContext,
};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;
use thiserror::Error;

use shared::card::{CardData, CardId};

// =============================================================================
// AppState — server lifecycle state machine
// =============================================================================

/// Server lifecycle states.
///
/// Transition flow: `Loading` → `ConfigValidation` → `Lobby` → `InSession`.
///
/// - `Loading`: asset files are being read from disk by the AssetServer.
/// - `ConfigValidation`: both assets have loaded; `validate_and_promote` runs.
/// - `Lobby`: game is ready; `Res<GameConfig>` and `Res<CardCatalog>` are live.
/// - `InSession`: a match is in progress (wired in later epics).
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Loading,
    ConfigValidation,
    Lobby,
    InSession,
}

// =============================================================================
// GameConfigAsset — the loaded-but-not-yet-validated form of GameConfig
// =============================================================================

/// Asset representation of `game_config.ron`. Loaded via `GameConfigLoader`.
///
/// This is a transient type: it exists only during the `Loading` and
/// `ConfigValidation` states. After `validate_and_promote` runs, the inner
/// `shared::config::GameConfig` is wrapped in `GameConfig` (the ECS resource)
/// and this handle is no longer accessed.
///
/// ADR-004 path b: wraps `shared::config::GameConfig` so that `shared/` can stay
/// bevy-free while still participating in the asset loading pipeline.
#[derive(Asset, TypePath, Clone, Debug)]
pub struct GameConfigAsset(pub shared::config::GameConfig);

/// Loader for `assets/config/game_config.ron`.
///
/// Reads the file as bytes, deserialises via `ron::de::from_bytes`.
/// Extension: `.ron`.
#[derive(Default, TypePath)]
pub struct GameConfigLoader;

/// Errors that can occur while loading `game_config.ron`.
#[derive(Debug, Error)]
pub enum GameConfigLoadError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("ron: {0}")]
    Ron(#[from] ron::error::SpannedError),
}

impl AssetLoader for GameConfigLoader {
    type Asset = GameConfigAsset;
    type Settings = ();
    type Error = GameConfigLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        use bevy::asset::AsyncReadExt as _;
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let config: shared::config::GameConfig =
            ron::de::from_bytes(&bytes).map_err(GameConfigLoadError::Ron)?;
        Ok(GameConfigAsset(config))
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

// =============================================================================
// GameConfig — the live ECS resource
// =============================================================================

/// Authoritative `GameConfig` resource.
///
/// Inserted into the ECS world by `validate_and_promote` after the asset loads
/// and passes validation. All systems read balance values through this resource.
///
/// Implements `Deref<Target = shared::config::GameConfig>` so callers can write
/// `config.starting_gold` instead of `config.0.starting_gold`.
///
/// ADR-003: this type lives in `server/` only; never in `shared/`.
///
/// # Example
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use server::foundation::config::GameConfig;
/// fn my_system(config: Res<GameConfig>) {
///     info!("Starting gold: {}", config.starting_gold);
/// }
/// ```
#[derive(Resource, Clone, Debug)]
pub struct GameConfig(pub shared::config::GameConfig);

impl Deref for GameConfig {
    type Target = shared::config::GameConfig;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// =============================================================================
// CardCatalog — server-side struct (not the shared type alias)
// =============================================================================

/// Immutable catalog of all card definitions loaded from `assets/data/cards.json`.
///
/// This is a **server-side struct**, distinct from `shared::card::CardCatalog`
/// (which is a `type` alias and cannot derive `Asset` or `Resource`).
/// The cards are identical — this is purely a newtype for ECS/Asset integration.
///
/// Inserted into the ECS world by `validate_and_promote`. Immutable after that point.
/// ADR-006 Part 1: card definitions are never mutated mid-session.
///
/// # Example
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use server::foundation::config::CardCatalog;
/// fn my_system(catalog: Res<CardCatalog>) {
///     info!("{} cards loaded", catalog.cards.len());
/// }
/// ```
#[derive(Asset, TypePath, Resource, Clone, Debug)]
pub struct CardCatalog {
    pub cards: HashMap<CardId, CardData>,
}

impl CardCatalog {
    /// Number of card definitions in the catalog.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// True if the catalog contains no cards (should never happen in production).
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

// =============================================================================
// CardCatalogLoader — JSON asset loader
// =============================================================================

/// Loader for `assets/data/cards.json`.
///
/// Reads the file as bytes, deserialises a `Vec<CardData>` via `serde_json`,
/// then builds the `HashMap<CardId, CardData>` index.
/// Extension: `.json`.
///
/// AC: struct exists, `#[derive(Default, TypePath)]`, implements `AssetLoader`
/// with `type Asset = CardCatalog`, reads JSON via `serde_json`.
#[derive(Default, TypePath)]
pub struct CardCatalogLoader;

/// Errors that can occur while loading `cards.json`.
#[derive(Debug, Error)]
pub enum CardCatalogLoadError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Wire deserialization — cards.json is a flat array of card objects.
#[derive(Deserialize)]
struct CardDataJson {
    pub id: u32,
    pub name_fr: String,
    pub name_en: String,
    pub class: shared::card::ClassId,
    pub family: Option<String>,
    pub rarity: shared::card::Rarity,
    pub card_type: shared::card::CardType,
    pub unit_type: shared::card::UnitType,
    pub cost: u32,
    pub atk: u8,
    pub hp: u8,
    pub mp: u8,
    pub ar: u8,
    pub keywords: Vec<shared::card::Keyword>,
    pub effect_text: String,
    pub art_id: String,
    pub pool_copies_override: Option<i32>,
}

impl From<CardDataJson> for CardData {
    fn from(j: CardDataJson) -> Self {
        CardData {
            id: CardId(j.id),
            name_fr: j.name_fr,
            name_en: j.name_en,
            class: j.class,
            family: j.family,
            rarity: j.rarity,
            card_type: j.card_type,
            unit_type: j.unit_type,
            cost: j.cost,
            atk: j.atk,
            hp: j.hp,
            mp: j.mp,
            ar: j.ar,
            keywords: j.keywords,
            effect_text: j.effect_text,
            art_id: j.art_id,
            pool_copies_override: j.pool_copies_override,
        }
    }
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
        use bevy::asset::AsyncReadExt as _;
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let raw: Vec<CardDataJson> =
            serde_json::from_slice(&bytes).map_err(CardCatalogLoadError::Json)?;
        let cards: HashMap<CardId, CardData> = raw
            .into_iter()
            .map(|j| {
                let card: CardData = j.into();
                (card.id, card)
            })
            .collect();
        Ok(CardCatalog { cards })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

// =============================================================================
// GameAssets — resource that holds asset handles during Loading
// =============================================================================

/// Holds `Handle<T>` references to the two assets being loaded.
///
/// Inserted by `start_loading` on `OnEnter(AppState::Loading)`.
/// Read by `check_loading_done` (polls for completion) and `validate_and_promote`
/// (accesses the loaded data). Remains in the world after `Lobby` is entered —
/// this is intentional: it acts as a retain-pin so the AssetServer does not
/// unload the assets.
///
/// ADR-004: bevy_asset_loader is not available for Bevy 0.18 (PR #264 draft),
/// so this resource replicates what an AssetCollection would do manually.
#[derive(Resource)]
pub struct GameAssets {
    pub game_config: Handle<GameConfigAsset>,
    pub card_catalog: Handle<CardCatalog>,
}

// =============================================================================
// Systems
// =============================================================================

/// Kicks off async loading of both asset files.
///
/// Runs `OnEnter(AppState::Loading)`. Inserts `GameAssets` with the two handles.
/// The AssetServer loads the files in the background; `check_loading_done` polls
/// until both are ready.
pub fn start_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("AppState::Loading — requesting game_config.ron and cards.json");
    let game_config = asset_server.load("config/game_config.ron");
    let card_catalog = asset_server.load("data/cards.json");
    commands.insert_resource(GameAssets {
        game_config,
        card_catalog,
    });
}

/// Polls asset readiness; transitions to `ConfigValidation` when both are loaded.
///
/// Runs every frame while `in_state(AppState::Loading)`.
/// Uses `Assets<T>::contains` (Bevy 0.18 API) to check handle presence.
pub fn check_loading_done(
    game_assets: Res<GameAssets>,
    config_assets: Res<Assets<GameConfigAsset>>,
    catalog_assets: Res<Assets<CardCatalog>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if config_assets.contains(&game_assets.game_config)
        && catalog_assets.contains(&game_assets.card_catalog)
    {
        info!("Both assets loaded — transitioning to AppState::ConfigValidation");
        next_state.set(AppState::ConfigValidation);
    }
}

/// Validates both loaded assets and promotes them to ECS resources.
///
/// Runs `OnEnter(AppState::ConfigValidation)`.
///
/// **Story 002 stub behaviour**: validation functions are present and called,
/// but on failure this stub logs an error and transitions to Lobby anyway.
/// Story 003 replaces this with `AppExit` on validation failure.
///
/// On success:
/// - Inserts `Res<GameConfig>` (server wrapper around the loaded config)
/// - Inserts `Res<CardCatalog>` (server struct with the card HashMap)
/// - Logs asset summary
/// - Transitions to `AppState::Lobby`
pub fn validate_and_promote(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    config_assets: Res<Assets<GameConfigAsset>>,
    catalog_assets: Res<Assets<CardCatalog>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(cfg_asset) = config_assets.get(&game_assets.game_config) else {
        error!("validate_and_promote: GameConfigAsset handle not ready — this should not happen");
        next_state.set(AppState::Lobby);
        return;
    };
    let Some(catalog_asset) = catalog_assets.get(&game_assets.card_catalog) else {
        error!("validate_and_promote: CardCatalog handle not ready — this should not happen");
        next_state.set(AppState::Lobby);
        return;
    };

    // Validate GameConfig
    // TODO(Story 003): if Err, call AppExit::Error instead of continuing.
    if let Err(e) = validate_game_config(&cfg_asset.0) {
        error!("GameConfig validation failed: {e}");
    }

    // Validate CardCatalog
    // TODO(Story 003): if Err, call AppExit::Error instead of continuing.
    if let Err(e) = validate_card_catalog(catalog_asset) {
        error!("CardCatalog validation failed: {e}");
    }

    let card_count = catalog_asset.cards.len();
    commands.insert_resource(GameConfig(cfg_asset.0.clone()));
    commands.insert_resource(catalog_asset.clone());

    info!(
        "Assets loaded: GameConfig + CardCatalog ({card_count} cards) — transitioning to AppState::Lobby"
    );
    next_state.set(AppState::Lobby);
}

// =============================================================================
// Validation functions — full logic (Story 003 will wire fatal exit paths)
// =============================================================================

/// Validates `GameConfig` fields against design invariants.
///
/// Returns `Ok(())` when all constraints pass, or `Err(description)` for the
/// first failing constraint. Story 003 makes validation failures fatal.
///
/// Constraints (from game-config.md §Acceptance Criteria):
/// - `shop_weight_cap` ∈ (0.0, 1.0)
/// - `shop_weight_per_card` < `shop_weight_cap`
/// - `fake_count` ∈ [1, 3]
/// - `objective_hp` >= 1
/// - `placement_timer_seconds` >= 1
pub fn validate_game_config(c: &shared::config::GameConfig) -> Result<(), String> {
    if !(0.0 < c.shop_weight_cap && c.shop_weight_cap < 1.0) {
        return Err(format!(
            "shop_weight_cap must be in (0.0, 1.0); got {}",
            c.shop_weight_cap
        ));
    }
    if !(c.shop_weight_per_card < c.shop_weight_cap) {
        return Err(format!(
            "shop_weight_per_card ({}) must be < shop_weight_cap ({})",
            c.shop_weight_per_card, c.shop_weight_cap
        ));
    }
    if !(1..=3).contains(&c.fake_count) {
        return Err(format!(
            "fake_count must be in [1, 3]; got {}",
            c.fake_count
        ));
    }
    if c.objective_hp < 1 {
        return Err("objective_hp must be >= 1".into());
    }
    if c.placement_timer_seconds < 1 {
        return Err("placement_timer_seconds must be >= 1".into());
    }
    Ok(())
}

/// Validates `CardCatalog` structural invariants.
///
/// Returns `Ok(())` when all constraints pass, or `Err(description)` for the
/// first failing constraint. Story 003 makes validation failures fatal.
///
/// Constraints:
/// - Catalog must be non-empty.
/// - Every entry's HashMap key must match the card's own `id` field.
pub fn validate_card_catalog(c: &CardCatalog) -> Result<(), String> {
    if c.cards.is_empty() {
        return Err("CardCatalog is empty".into());
    }
    for (key, card) in &c.cards {
        if key != &card.id {
            return Err(format!("key {:?} != card.id {:?}", key, card.id));
        }
    }
    Ok(())
}

// =============================================================================
// ConfigPlugin — registers all types, loaders, and systems
// =============================================================================

/// Registers all asset types, loaders, state machine, and loading systems.
///
/// Add to `App` before `app.run()`. Assumes `AssetPlugin` is already present.
///
/// # Example
/// ```rust,no_run
/// # use bevy::asset::AssetPlugin;
/// # use bevy::prelude::*;
/// # use server::foundation::config::ConfigPlugin;
/// App::new()
///     .add_plugins(MinimalPlugins)
///     .add_plugins(AssetPlugin::default())
///     .add_plugins(ConfigPlugin)
///     .run();
/// ```
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        // Register asset types
        app.init_asset::<GameConfigAsset>();
        app.init_asset_loader::<GameConfigLoader>();
        app.init_asset::<CardCatalog>();
        app.init_asset_loader::<CardCatalogLoader>();

        // State machine
        app.init_state::<AppState>();

        // Loading pipeline systems
        app.add_systems(OnEnter(AppState::Loading), start_loading);
        app.add_systems(
            Update,
            check_loading_done.run_if(in_state(AppState::Loading)),
        );
        app.add_systems(OnEnter(AppState::ConfigValidation), validate_and_promote);
    }
}
