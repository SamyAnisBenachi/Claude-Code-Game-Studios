//! Bot-vs-bot soak entrypoint integration test scaffold (PROMPT 1629).
//!
//! Verifies the soak launcher contract — env-var activation, two-bot resource
//! setup, and `BotQaSnapshotPlugin` initialization — without launching any
//! long-running GUI or server process.
//!
//! ## Soak launcher contract (story-002 AC2 / AC6 subset)
//!
//! The launcher (`tools/dev-launcher/Start-BotVsBotSoak.ps1`) sets three env
//! vars before starting `server.exe`:
//!
//! | Env var | Purpose |
//! |---|---|
//! | `CCGS_BOT_QA_SNAPSHOT` | `1` = force-enable server-side snapshots |
//! | `CCGS_BOT_QA_SNAPSHOT_DIR` | Override snapshot output directory |
//! | `CCGS_BOT_DECISION_LOG_PATH` | Override decision-log JSONL path |
//!
//! The tests below prove that `BotQaSnapshotConfig::from_env_values` honours
//! every branch of the activation rule, that `BotPlayers` can hold two bots
//! (the precondition for a headless two-bot room), and that the plugin
//! initialises without panicking inside a `MinimalPlugins` App.

use bevy::prelude::*;
use server::feature::bot::{
    BotPlayers, BotQaSnapshotConfig, BotQaSnapshotPlugin, BotQaSnapshotState,
    BotState, BOT_DECISION_LOG_PATH_ENV_VAR, BOT_QA_SNAPSHOT_DIR_ENV_VAR,
    BOT_QA_SNAPSHOT_ENV_VAR, DEFAULT_BOT_DECISION_LOG_PATH, DEFAULT_BOT_QA_SNAPSHOT_DIR,
    DEFAULT_PERIODIC_SNAPSHOT_INTERVAL_MS,
};
use shared::session::PlayerId;
use std::path::PathBuf;

#[path = "../../test_helpers.rs"]
mod test_helpers;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a `BotQaSnapshotConfig` via the pure `from_env_values` constructor.
fn cfg(enable: Option<&str>, dir: Option<&str>, log: Option<&str>) -> BotQaSnapshotConfig {
    BotQaSnapshotConfig::from_env_values(enable, dir, log, false)
}

fn cfg_dev(enable: Option<&str>, dev_default: bool) -> BotQaSnapshotConfig {
    BotQaSnapshotConfig::from_env_values(enable, None, None, dev_default)
}

// ---------------------------------------------------------------------------
// BotQaSnapshotConfig::from_env_values — activation contract
// ---------------------------------------------------------------------------

#[test]
fn test_enable_var_1_forces_enabled() {
    assert!(cfg(Some("1"), None, None).enabled);
}

#[test]
fn test_enable_var_0_forces_disabled() {
    assert!(!cfg(Some("0"), None, None).enabled);
}

#[test]
fn test_enable_var_none_follows_dev_default_false() {
    assert!(!cfg_dev(None, false).enabled);
}

#[test]
fn test_enable_var_none_follows_dev_default_true() {
    assert!(cfg_dev(None, true).enabled);
}

#[test]
fn test_enable_var_empty_string_follows_dev_default_true() {
    assert!(cfg_dev(Some(""), true).enabled);
}

#[test]
fn test_enable_var_empty_string_follows_dev_default_false() {
    assert!(!cfg_dev(Some(""), false).enabled);
}

#[test]
fn test_enable_var_whitespace_only_follows_dev_default() {
    assert!(cfg_dev(Some("   "), true).enabled);
    assert!(!cfg_dev(Some("   "), false).enabled);
}

#[test]
fn test_enable_var_invalid_value_disables() {
    // Any unrecognised value (e.g. "yes", "true", "on") is treated as
    // disabled; the system warns but never panics.
    assert!(!cfg(Some("yes"), None, None).enabled);
    assert!(!cfg(Some("true"), None, None).enabled);
    assert!(!cfg(Some("on"), None, None).enabled);
}

// ---------------------------------------------------------------------------
// BotQaSnapshotConfig — path defaults and overrides
// ---------------------------------------------------------------------------

#[test]
fn test_snapshot_dir_defaults_to_constant() {
    assert_eq!(
        cfg(None, None, None).snapshot_dir,
        PathBuf::from(DEFAULT_BOT_QA_SNAPSHOT_DIR)
    );
}

#[test]
fn test_decision_log_path_defaults_to_constant() {
    assert_eq!(
        cfg(None, None, None).decision_log_path,
        PathBuf::from(DEFAULT_BOT_DECISION_LOG_PATH)
    );
}

#[test]
fn test_snapshot_dir_overridden_by_non_empty_var() {
    assert_eq!(
        cfg(None, Some("/tmp/soak-snapshots"), None).snapshot_dir,
        PathBuf::from("/tmp/soak-snapshots")
    );
}

#[test]
fn test_decision_log_overridden_by_non_empty_var() {
    assert_eq!(
        cfg(None, None, Some("/tmp/soak.jsonl")).decision_log_path,
        PathBuf::from("/tmp/soak.jsonl")
    );
}

#[test]
fn test_blank_dir_var_falls_back_to_default() {
    assert_eq!(
        cfg(None, Some(""), None).snapshot_dir,
        PathBuf::from(DEFAULT_BOT_QA_SNAPSHOT_DIR)
    );
}

#[test]
fn test_periodic_interval_is_constant() {
    assert_eq!(
        cfg(None, None, None).periodic_interval_ms,
        DEFAULT_PERIODIC_SNAPSHOT_INTERVAL_MS
    );
}

// ---------------------------------------------------------------------------
// BotPlayers — two-bot soak precondition
// ---------------------------------------------------------------------------

#[test]
fn test_two_bots_can_be_inserted_into_bot_players() {
    let mut bots = BotPlayers::default();
    bots.insert(BotState::new(PlayerId(0x8000_0000_0000_0001), 1));
    bots.insert(BotState::new(PlayerId(0x8000_0000_0000_0002), 2));
    assert_eq!(bots.len(), 2, "two-bot soak needs exactly two bot entries");
    assert!(!bots.is_empty());
}

#[test]
fn test_bot_players_contains_correctly_identifies_bot_ids() {
    let id_a = PlayerId(0x8000_0000_0000_0001);
    let id_b = PlayerId(0x8000_0000_0000_0002);
    let id_human = PlayerId(42);

    let mut bots = BotPlayers::default();
    bots.insert(BotState::new(id_a, id_a.0));
    bots.insert(BotState::new(id_b, id_b.0));

    assert!(bots.contains(id_a));
    assert!(bots.contains(id_b));
    assert!(!bots.contains(id_human), "human id must not appear in BotPlayers");
}

#[test]
fn test_bot_state_seed_is_preserved() {
    let id = PlayerId(0x8000_0000_DEAD_BEEF);
    let state = BotState::new(id, 0xCAFE_F00D);
    assert_eq!(state.rng_seed, 0xCAFE_F00D);
    assert_eq!(state.rng_word_counter, 0, "fresh bot starts at word counter 0");
    assert!(state.class_choice.is_none(), "no class assigned at construction");
}

// ---------------------------------------------------------------------------
// BotQaSnapshotPlugin — App-level initialisation
// ---------------------------------------------------------------------------

#[test]
fn test_bot_qa_snapshot_plugin_initialises_without_panic() {
    test_helpers::init_test_tracing();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Insert a disabled config so no disk I/O occurs during the test.
    app.insert_resource(BotQaSnapshotConfig {
        enabled: false,
        snapshot_dir: PathBuf::from(DEFAULT_BOT_QA_SNAPSHOT_DIR),
        decision_log_path: PathBuf::from(DEFAULT_BOT_DECISION_LOG_PATH),
        periodic_interval_ms: DEFAULT_PERIODIC_SNAPSHOT_INTERVAL_MS,
    });

    app.add_plugins(BotQaSnapshotPlugin);

    // Tick once; all systems must early-return cleanly when disabled.
    app.update();

    assert!(
        app.world().get_resource::<BotQaSnapshotConfig>().is_some(),
        "plugin must register BotQaSnapshotConfig"
    );
    assert!(
        app.world().get_resource::<BotQaSnapshotState>().is_some(),
        "plugin must register BotQaSnapshotState"
    );
}

// ---------------------------------------------------------------------------
// Soak env-var constant names (regression guard against launcher drift)
// ---------------------------------------------------------------------------

#[test]
fn test_env_var_constant_names_match_launcher_contract() {
    // Start-BotVsBotSoak.ps1 sets these exact env var names. If the Rust
    // constants drift, the launcher and server would operate on different vars.
    assert_eq!(BOT_QA_SNAPSHOT_ENV_VAR, "CCGS_BOT_QA_SNAPSHOT");
    assert_eq!(BOT_QA_SNAPSHOT_DIR_ENV_VAR, "CCGS_BOT_QA_SNAPSHOT_DIR");
    assert_eq!(BOT_DECISION_LOG_PATH_ENV_VAR, "CCGS_BOT_DECISION_LOG_PATH");
}
