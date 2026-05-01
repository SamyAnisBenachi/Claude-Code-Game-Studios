// GameConfig — single source of truth for all tuning knobs
// ADR-003: NO #[derive(Resource)] here — server wraps it locally
// ADR-004: loaded via bevy_asset_loader from assets/config/game_config.ron

use serde::{Deserialize, Serialize};

/// All balance-tunable values. Loaded at server startup from game_config.ron.
/// ADR-004: all fields have #[serde(default)] — missing fields fall back to Default impl.
/// ADR-003: no Resource derive in shared/; server does app.insert_resource(config).
/// Epic 2 decision (ADR-004 path b): server creates GameConfigAsset + GameConfig wrapper types.
/// shared/ stays bevy-free. See server/src/foundation/config.rs for the wrapper definitions.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct GameConfig {
    // Pool — Card Data & Pool
    pub common_pool_copies: u32,
    pub uncommon_pool_copies: u32,
    pub rare_pool_copies: u32,
    pub shop_weight_per_card: f32,
    pub shop_weight_cap: f32,

    // Economy
    pub starting_gold: u32,
    pub gold_baseline_per_round: u32,
    pub interest_threshold_gold: u32,
    pub interest_max_bonus: u32,
    pub objective_gold_reward: u32,
    pub kill_gold_reward: u32,
    pub mana_cap: u32,
    pub mana_cap_max: u32,
    pub refresh_base_cost: u32,
    pub refresh_cap: u32,

    // Objectives / Spawn
    pub objective_hp: u32,
    pub fake_count: u32,
    pub fake_objective_spawn_advance: u32,

    // Timers — RSM phase durations
    pub draft_initial_timer_seconds: u32,
    pub draft_shop_timer_seconds: u32,
    pub placement_timer_seconds: u32,
    pub resolution_max_duration_seconds: u32,
    pub disconnect_grace_seconds: u32,
    pub lobby_timeout_seconds: u32,
    pub lobby_heartbeat_timeout_seconds: u32,

    // Timers — Auction System
    pub auction_timer_seconds: u32,
    pub auction_timer_reset_seconds: u32,
    pub auction_max_duration_seconds: u32,
    // Starting bid floors — Auction System (card-data-pool.md §Tuning Knobs)
    pub auction_floor_rare: u32,
    pub auction_floor_epic: u32,
    pub auction_floor_legendary: u32,

    // Class mechanics
    pub xelor_sablier_steal: u32,

    // Network Protocol
    pub protocol_version: u32,
    pub hello_timeout_ms: u32,
    pub ack_timeout_ms: u32,
    pub heartbeat_interval_ms: u32,
}

/// Design-intent defaults per game-config.md Tuning Knobs table.
/// ADR-004: GC4 verifies missing fields fall back to these values.
/// GCN-DEFAULTS: all fields must equal these values in GameConfig::default().
impl Default for GameConfig {
    fn default() -> Self {
        Self {
            common_pool_copies: 6,
            uncommon_pool_copies: 5,
            rare_pool_copies: 4,
            shop_weight_per_card: 0.10,
            shop_weight_cap: 0.65,
            starting_gold: 5,
            gold_baseline_per_round: 2,
            interest_threshold_gold: 5,
            interest_max_bonus: 2,
            objective_gold_reward: 3,
            kill_gold_reward: 1,
            mana_cap: 10,
            mana_cap_max: 12,
            refresh_base_cost: 1,
            refresh_cap: 1,
            objective_hp: 5,
            fake_count: 2,
            fake_objective_spawn_advance: 1,
            draft_initial_timer_seconds: 45,
            draft_shop_timer_seconds: 30,
            placement_timer_seconds: 10,
            resolution_max_duration_seconds: 60,
            disconnect_grace_seconds: 30,
            lobby_timeout_seconds: 90,
            lobby_heartbeat_timeout_seconds: 15,
            auction_timer_seconds: 20,
            auction_timer_reset_seconds: 5,
            auction_max_duration_seconds: 120,
            auction_floor_rare: 3,
            auction_floor_epic: 4,
            auction_floor_legendary: 5,
            xelor_sablier_steal: 1,
            protocol_version: 1,
            hello_timeout_ms: 5000,
            ack_timeout_ms: 10000,
            heartbeat_interval_ms: 5000,
        }
    }
}
