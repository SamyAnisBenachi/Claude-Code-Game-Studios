//! Unit tests for BotSoakConfig max-rounds bound (PROMPT 1640).
//!
//! These tests verify the resource inserts correctly and the env-var parsing
//! behaves as specified (disabled when absent/zero, active when positive).

use bevy::prelude::*;
use server::feature::bot::{BotSoakConfig, BOT_MAX_ROUNDS_ENV_VAR};

// ---------------------------------------------------------------------------
// Resource insertability
// ---------------------------------------------------------------------------

#[test]
fn test_bot_soak_config_default_is_disabled() {
    let config = BotSoakConfig::default();
    assert!(
        config.max_rounds.is_none(),
        "default BotSoakConfig must have max_rounds = None (disabled)"
    );
}

#[test]
fn test_bot_soak_config_inserts_into_fresh_world() {
    let mut app = App::new();
    app.insert_resource(BotSoakConfig {
        max_rounds: Some(5),
    });
    let config = app.world().resource::<BotSoakConfig>();
    assert_eq!(config.max_rounds, Some(5));
}

// ---------------------------------------------------------------------------
// from_env parsing
// ---------------------------------------------------------------------------

#[test]
fn test_from_env_absent_yields_none() {
    // Ensure the var is not set for this test.
    std::env::remove_var(BOT_MAX_ROUNDS_ENV_VAR);
    let config = BotSoakConfig::from_env();
    assert!(
        config.max_rounds.is_none(),
        "unset env var must yield max_rounds = None"
    );
}

#[test]
fn test_from_env_zero_yields_none() {
    std::env::set_var(BOT_MAX_ROUNDS_ENV_VAR, "0");
    let config = BotSoakConfig::from_env();
    std::env::remove_var(BOT_MAX_ROUNDS_ENV_VAR);
    assert!(
        config.max_rounds.is_none(),
        "CCGS_BOT_MAX_ROUNDS=0 must be treated as disabled (None)"
    );
}

#[test]
fn test_from_env_positive_integer_activates_bound() {
    std::env::set_var(BOT_MAX_ROUNDS_ENV_VAR, "10");
    let config = BotSoakConfig::from_env();
    std::env::remove_var(BOT_MAX_ROUNDS_ENV_VAR);
    assert_eq!(
        config.max_rounds,
        Some(10),
        "CCGS_BOT_MAX_ROUNDS=10 must yield max_rounds = Some(10)"
    );
}

#[test]
fn test_from_env_whitespace_trimmed() {
    std::env::set_var(BOT_MAX_ROUNDS_ENV_VAR, "  3  ");
    let config = BotSoakConfig::from_env();
    std::env::remove_var(BOT_MAX_ROUNDS_ENV_VAR);
    assert_eq!(config.max_rounds, Some(3));
}

#[test]
fn test_from_env_non_integer_yields_none() {
    std::env::set_var(BOT_MAX_ROUNDS_ENV_VAR, "notanumber");
    let config = BotSoakConfig::from_env();
    std::env::remove_var(BOT_MAX_ROUNDS_ENV_VAR);
    assert!(
        config.max_rounds.is_none(),
        "non-integer value must be treated as disabled"
    );
}
