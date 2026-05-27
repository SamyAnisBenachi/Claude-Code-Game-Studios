//! Unit tests for BotSoakConfig max-rounds bound (PROMPT 1640).
//!
//! These tests verify the resource inserts correctly and the env-var parsing
//! behaves as specified (disabled when absent/zero, active when positive).
//!
//! # Parallelism note (PROMPT 1673)
//!
//! Several tests mutate `CCGS_BOT_MAX_ROUNDS` via `std::env::set_var`, which is
//! a process-global operation.  Cargo runs tests in the same binary concurrently
//! by default, so without serialisation the env writes race and produce flaky
//! failures.  `ENV_LOCK` — a process-local `Mutex<()>` — ensures only one
//! env-touching test runs at a time.  Tests that do not touch the env var are
//! safe to run concurrently and do not acquire the lock.

use std::sync::Mutex;

use bevy::prelude::*;
use server::feature::bot::{BotSoakConfig, BOT_MAX_ROUNDS_ENV_VAR};

/// Serialises every test that calls `std::env::set_var` / `remove_var` on
/// `CCGS_BOT_MAX_ROUNDS`.  Acquiring this lock is mandatory for the `from_env_*`
/// group; the two pure-struct tests skip it.
static ENV_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// Resource insertability — no env access, safe to run concurrently
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
// from_env parsing — each test acquires ENV_LOCK before touching the env var
// ---------------------------------------------------------------------------

#[test]
fn test_from_env_absent_yields_none() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var(BOT_MAX_ROUNDS_ENV_VAR);
    let config = BotSoakConfig::from_env();
    assert!(
        config.max_rounds.is_none(),
        "unset env var must yield max_rounds = None"
    );
}

#[test]
fn test_from_env_zero_yields_none() {
    let _guard = ENV_LOCK.lock().unwrap();
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
    let _guard = ENV_LOCK.lock().unwrap();
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
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var(BOT_MAX_ROUNDS_ENV_VAR, "  3  ");
    let config = BotSoakConfig::from_env();
    std::env::remove_var(BOT_MAX_ROUNDS_ENV_VAR);
    assert_eq!(config.max_rounds, Some(3));
}

#[test]
fn test_from_env_non_integer_yields_none() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var(BOT_MAX_ROUNDS_ENV_VAR, "notanumber");
    let config = BotSoakConfig::from_env();
    std::env::remove_var(BOT_MAX_ROUNDS_ENV_VAR);
    assert!(
        config.max_rounds.is_none(),
        "non-integer value must be treated as disabled"
    );
}
