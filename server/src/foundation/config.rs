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
    #[error("duplicate CardId in cards.json: {0:?}")]
    DuplicateCardId(CardId),
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
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let raw: Vec<CardDataJson> =
            serde_json::from_slice(&bytes).map_err(CardCatalogLoadError::Json)?;
        let mut cards = HashMap::new();
        for json_card in raw {
            let card: CardData = json_card.into();
            let card_id = card.id;
            if cards.insert(card_id, card).is_some() {
                return Err(CardCatalogLoadError::DuplicateCardId(card_id));
            }
        }
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
    mut app_exit: MessageWriter<AppExit>,
) {
    let Some(cfg_asset) = config_assets.get(&game_assets.game_config) else {
        error!("validate_and_promote: GameConfigAsset handle not ready — this should not happen");
        app_exit.write(AppExit::error());
        return;
    };
    let Some(catalog_asset) = catalog_assets.get(&game_assets.card_catalog) else {
        error!("validate_and_promote: CardCatalog handle not ready — this should not happen");
        app_exit.write(AppExit::error());
        return;
    };

    if let Err(e) = validate_game_config(&cfg_asset.0) {
        error!("GameConfig validation failed: {e}");
        app_exit.write(AppExit::error());
        return;
    }

    if let Err(e) = validate_card_catalog(catalog_asset) {
        error!("CardCatalog validation failed: {e}");
        app_exit.write(AppExit::error());
        return;
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
    if c.shop_weight_cap <= 0.0 {
        return Err(format!(
            "shop_weight_cap must be > 0.0; got {}",
            c.shop_weight_cap
        ));
    }
    if c.shop_weight_cap >= 1.0 {
        return Err(format!(
            "shop_weight_cap must be < 1.0; got {}",
            c.shop_weight_cap
        ));
    }
    if !(c.shop_weight_per_card < c.shop_weight_cap) {
        return Err(format!(
            "shop_weight_per_card ({}) must be < shop_weight_cap ({})",
            c.shop_weight_per_card, c.shop_weight_cap
        ));
    }
    if c.common_pool_copies < 1 {
        return Err("common_pool_copies must be >= 1".into());
    }
    if c.uncommon_pool_copies < 1 {
        return Err("uncommon_pool_copies must be >= 1".into());
    }
    if c.rare_pool_copies < 1 {
        return Err("rare_pool_copies must be >= 1".into());
    }
    if c.fake_count < 1 {
        return Err(
            "fake_count must be >= 1 - the bluffing mechanic is a load-bearing design pillar"
                .into(),
        );
    }
    if c.fake_count > 3 {
        return Err(format!("fake_count must be <= 3; got {}", c.fake_count));
    }
    if c.objective_hp < 1 {
        return Err("objective_hp must be >= 1".into());
    }
    if c.placement_timer_seconds < 1 {
        return Err("placement_timer_seconds must be >= 1".into());
    }
    if c.auction_timer_seconds < 1 {
        return Err("auction_timer_seconds must be >= 1".into());
    }
    if c.auction_timer_reset_seconds >= c.auction_timer_seconds {
        return Err(format!(
            "auction_timer_reset_seconds ({}) must be < auction_timer_seconds ({})",
            c.auction_timer_reset_seconds, c.auction_timer_seconds
        ));
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
        return Err("CardCatalog is empty - no cards to draft".into());
    }
    for (key, card) in &c.cards {
        if key != &card.id {
            return Err(format!(
                "CardCatalog key {:?} does not match CardData.id {:?}",
                key, card.id
            ));
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
/// # use bevy::state::app::StatesPlugin;
/// # use server::foundation::config::ConfigPlugin;
/// App::new()
///     .add_plugins(MinimalPlugins)
///     .add_plugins(StatesPlugin)
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use bevy::prelude::{MinimalPlugins, Update};
    use bevy::state::app::StatesPlugin;
    use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};

    fn game_config_error(mut edit: impl FnMut(&mut shared::config::GameConfig)) -> String {
        let mut config = shared::config::GameConfig::default();
        edit(&mut config);
        validate_game_config(&config).expect_err("config should fail validation")
    }

    fn valid_card(id: u32) -> CardData {
        CardData {
            id: CardId(id),
            name_fr: format!("Carte {id}"),
            name_en: format!("Card {id}"),
            class: ClassId::Iop,
            family: Some("Test".to_string()),
            rarity: Rarity::Common,
            card_type: CardType::Minion,
            unit_type: UnitType::Blade,
            cost: 1,
            atk: 1,
            hp: 1,
            mp: 1,
            ar: 0,
            keywords: vec![],
            effect_text: String::new(),
            art_id: format!("test_{id}"),
            pool_copies_override: None,
        }
    }

    fn catalog_with(cards: Vec<CardData>) -> CardCatalog {
        CardCatalog {
            cards: cards.into_iter().map(|card| (card.id, card)).collect(),
        }
    }

    #[test]
    fn test_game_config_validation_default_passes() {
        assert_eq!(
            validate_game_config(&shared::config::GameConfig::default()),
            Ok(())
        );
    }

    #[test]
    fn test_game_config_validation_rejects_shop_weight_cap_above_one() {
        let err = game_config_error(|config| config.shop_weight_cap = 1.5);
        assert!(err.contains("shop_weight_cap"));
        assert!(err.contains("1.5"));
    }

    #[test]
    fn test_game_config_validation_rejects_shop_weight_cap_zero() {
        let err = game_config_error(|config| config.shop_weight_cap = 0.0);
        assert!(err.contains("shop_weight_cap"));
        assert!(err.contains("> 0.0"));
    }

    #[test]
    fn test_game_config_validation_rejects_shop_weight_per_card_at_cap() {
        let err = game_config_error(|config| {
            config.shop_weight_cap = 0.10;
            config.shop_weight_per_card = 0.10;
        });
        assert!(err.contains("shop_weight_per_card"));
        assert!(err.contains("shop_weight_cap"));
    }

    #[test]
    fn test_game_config_validation_rejects_zero_common_pool_copies() {
        let err = game_config_error(|config| config.common_pool_copies = 0);
        assert!(err.contains("common_pool_copies"));
    }

    #[test]
    fn test_game_config_validation_rejects_zero_uncommon_pool_copies() {
        let err = game_config_error(|config| config.uncommon_pool_copies = 0);
        assert!(err.contains("uncommon_pool_copies"));
    }

    #[test]
    fn test_game_config_validation_rejects_zero_rare_pool_copies() {
        let err = game_config_error(|config| config.rare_pool_copies = 0);
        assert!(err.contains("rare_pool_copies"));
    }

    #[test]
    fn test_game_config_validation_rejects_fake_count_zero_with_design_message() {
        let err = game_config_error(|config| config.fake_count = 0);
        assert!(err.contains("fake_count"));
        assert!(err.contains("load-bearing design pillar"));
    }

    #[test]
    fn test_game_config_validation_rejects_fake_count_four() {
        let err = game_config_error(|config| config.fake_count = 4);
        assert!(err.contains("fake_count"));
        assert!(err.contains("<= 3"));
    }

    #[test]
    fn test_game_config_validation_rejects_objective_hp_zero() {
        let err = game_config_error(|config| config.objective_hp = 0);
        assert!(err.contains("objective_hp"));
    }

    #[test]
    fn test_game_config_validation_rejects_placement_timer_zero() {
        let err = game_config_error(|config| config.placement_timer_seconds = 0);
        assert!(err.contains("placement_timer_seconds"));
    }

    #[test]
    fn test_game_config_validation_rejects_auction_timer_zero() {
        let err = game_config_error(|config| config.auction_timer_seconds = 0);
        assert!(err.contains("auction_timer_seconds"));
    }

    #[test]
    fn test_game_config_validation_rejects_auction_timer_reset_at_timer() {
        let err = game_config_error(|config| {
            config.auction_timer_seconds = 20;
            config.auction_timer_reset_seconds = 20;
        });
        assert!(err.contains("auction_timer_reset_seconds"));
        assert!(err.contains("20"));
    }

    #[test]
    fn test_game_config_validation_card_catalog_valid_catalog_passes() {
        let catalog = catalog_with(vec![valid_card(1)]);
        assert_eq!(validate_card_catalog(&catalog), Ok(()));
    }

    #[test]
    fn test_game_config_validation_card_catalog_empty_catalog_fails() {
        let catalog = CardCatalog {
            cards: HashMap::new(),
        };
        let err = validate_card_catalog(&catalog).expect_err("empty catalog should fail");
        assert!(err.contains("empty"));
    }

    #[test]
    fn test_game_config_validation_card_catalog_key_mismatch_fails() {
        let card = valid_card(1);
        let catalog = CardCatalog {
            cards: HashMap::from([(CardId(99), card)]),
        };
        let err = validate_card_catalog(&catalog).expect_err("key mismatch should fail");
        assert!(err.contains("CardCatalog key"));
        assert!(err.contains("CardData.id"));
    }

    #[test]
    fn test_game_config_validation_card_catalog_allows_non_positive_pool_override() {
        let mut card = valid_card(1);
        card.pool_copies_override = Some(-1);
        let catalog = catalog_with(vec![card]);
        assert_eq!(validate_card_catalog(&catalog), Ok(()));
    }

    #[test]
    fn test_game_config_validation_promote_success_inserts_resources_and_enters_lobby() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_message::<AppExit>();
        app.init_state::<AppState>();

        let mut config_assets = Assets::<GameConfigAsset>::default();
        let game_config = config_assets.add(GameConfigAsset(shared::config::GameConfig::default()));
        let mut catalog_assets = Assets::<CardCatalog>::default();
        let card_catalog = catalog_assets.add(catalog_with(vec![valid_card(1)]));

        app.insert_resource(GameAssets {
            game_config,
            card_catalog,
        });
        app.insert_resource(config_assets);
        app.insert_resource(catalog_assets);
        app.add_systems(Update, validate_and_promote);

        app.update();

        assert!(app.world().contains_resource::<GameConfig>());
        assert!(app.world().contains_resource::<CardCatalog>());
        assert!(matches!(
            app.world().resource::<NextState<AppState>>(),
            NextState::Pending(AppState::Lobby)
        ));
        assert!(app.world().resource::<Messages<AppExit>>().is_empty());
    }

    #[test]
    fn test_game_config_validation_promote_failure_writes_app_exit_and_does_not_promote() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_message::<AppExit>();
        app.init_state::<AppState>();

        let mut invalid_config = shared::config::GameConfig::default();
        invalid_config.fake_count = 0;
        let mut config_assets = Assets::<GameConfigAsset>::default();
        let game_config = config_assets.add(GameConfigAsset(invalid_config));
        let mut catalog_assets = Assets::<CardCatalog>::default();
        let card_catalog = catalog_assets.add(catalog_with(vec![valid_card(1)]));

        app.insert_resource(GameAssets {
            game_config,
            card_catalog,
        });
        app.insert_resource(config_assets);
        app.insert_resource(catalog_assets);
        app.add_systems(Update, validate_and_promote);

        app.update();

        assert!(!app.world().contains_resource::<GameConfig>());
        assert!(!app.world().contains_resource::<CardCatalog>());
        assert!(matches!(
            app.world().resource::<NextState<AppState>>(),
            NextState::Unchanged
        ));

        let exits = app.world().resource::<Messages<AppExit>>();
        let mut cursor = exits.get_cursor();
        let written: Vec<_> = cursor.read(exits).collect();
        assert_eq!(written, vec![&AppExit::error()]);
    }
}
