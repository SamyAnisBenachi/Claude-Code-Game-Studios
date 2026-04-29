// GameConfig Defaults Test — game-config.md Acceptance Criteria GCN-DEFAULTS
// ADR-004: all fields must equal Tuning Knobs table values in GameConfig::default()
// Run: cargo test -p server

use shared::config::GameConfig;

#[test]
fn test_game_config_default_economy_values() {
    let config = GameConfig::default();
    assert_eq!(config.starting_gold, 5);
    assert_eq!(config.gold_baseline_per_round, 2);
    assert_eq!(config.interest_threshold_gold, 5);
    assert_eq!(config.interest_max_bonus, 2);
    assert_eq!(config.objective_gold_reward, 3);
    assert_eq!(config.kill_gold_reward, 1);
    assert_eq!(config.mana_cap, 10);
    assert_eq!(config.refresh_base_cost, 1);
}

#[test]
fn test_game_config_default_pool_values() {
    let config = GameConfig::default();
    assert_eq!(config.common_pool_copies, 6);
    assert_eq!(config.uncommon_pool_copies, 5);
    assert_eq!(config.rare_pool_copies, 4);
    assert!((config.shop_weight_per_card - 0.10).abs() < f32::EPSILON);
    assert!((config.shop_weight_cap - 0.65).abs() < f32::EPSILON);
}

#[test]
fn test_game_config_default_objective_values() {
    let config = GameConfig::default();
    assert_eq!(config.objective_hp, 5);
    assert_eq!(config.fake_count, 2);
    assert_eq!(config.fake_objective_spawn_advance, 1);
}

#[test]
fn test_game_config_default_rsm_timers() {
    let config = GameConfig::default();
    assert_eq!(config.draft_initial_timer_seconds, 45);
    assert_eq!(config.draft_shop_timer_seconds, 30);
    assert_eq!(config.placement_timer_seconds, 10);
    assert_eq!(config.resolution_max_duration_seconds, 60);
    assert_eq!(config.disconnect_grace_seconds, 30);
    assert_eq!(config.lobby_timeout_seconds, 90);
    assert_eq!(config.lobby_heartbeat_timeout_seconds, 15);
}

#[test]
fn test_game_config_default_auction_timers() {
    let config = GameConfig::default();
    assert_eq!(config.auction_timer_seconds, 20);
    assert_eq!(config.auction_timer_reset_seconds, 5);
    assert_eq!(config.auction_max_duration_seconds, 120);
    // ADR-004: auction_max_duration >= auction_timer + (20 * reset) = 20 + 100 = 120
    assert!(config.auction_max_duration_seconds >= config.auction_timer_seconds + 20 * config.auction_timer_reset_seconds);
}

#[test]
fn test_game_config_default_network_values() {
    let config = GameConfig::default();
    assert_eq!(config.protocol_version, 1);
    assert_eq!(config.hello_timeout_ms, 5000);
    assert_eq!(config.ack_timeout_ms, 10000);
    assert_eq!(config.heartbeat_interval_ms, 5000);
}

#[test]
fn test_game_config_dangerous_value_constraints() {
    // ADR-004 validation rules — these must hold for any valid config
    let config = GameConfig::default();
    assert!(config.shop_weight_cap > 0.0);
    assert!(config.shop_weight_cap < 1.0);
    assert!(config.shop_weight_per_card < config.shop_weight_cap);
    assert!(config.fake_count >= 1);
    assert!(config.fake_count <= 3);
    assert!(config.objective_hp >= 1);
    assert!(config.placement_timer_seconds >= 1);
    assert!(config.auction_timer_seconds >= 1);
    assert!(config.auction_timer_reset_seconds < config.auction_timer_seconds);
}
