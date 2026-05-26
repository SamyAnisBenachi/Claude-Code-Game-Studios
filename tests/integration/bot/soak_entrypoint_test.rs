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

// ---------------------------------------------------------------------------
// PROMPT 1645 — PS1 launcher static-contract tests (script presence + content)
//
// These tests read the *source* script at tools/dev-launcher/Start-BotVsBotSoak.ps1
// (resolved relative to CARGO_MANIFEST_DIR / ..) and assert that the documented
// invocation contract is present in the file — without executing the script or
// spawning any process.  They are the durable automated gate that GAP-05
// required and PROMPT 1629 did not include.
//
// Path anchor: CARGO_MANIFEST_DIR = server/ → parent = repo root.
// ---------------------------------------------------------------------------

fn launcher_script_path() -> std::path::PathBuf {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("server/ must have a parent (repo root)")
        .join("tools/dev-launcher/Start-BotVsBotSoak.ps1")
}

fn launcher_script_content() -> String {
    let path = launcher_script_path();
    std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "Could not read Start-BotVsBotSoak.ps1 at {}: {}",
            path.display(),
            e
        )
    })
}

// ---- Script presence -------------------------------------------------------

#[test]
fn test_launcher_script_exists_at_canonical_path() {
    assert!(
        launcher_script_path().is_file(),
        "tools/dev-launcher/Start-BotVsBotSoak.ps1 must exist (GAP-05 contract)"
    );
}

// ---- PowerShell parameter contract ----------------------------------------

#[test]
fn test_launcher_declares_max_rounds_parameter() {
    let src = launcher_script_content();
    assert!(
        src.contains("[int]$MaxRounds"),
        "script must declare [int]$MaxRounds parameter (bounded-soak contract)"
    );
}

#[test]
fn test_launcher_declares_duration_seconds_parameter() {
    let src = launcher_script_content();
    assert!(
        src.contains("[int]$DurationSeconds"),
        "script must declare [int]$DurationSeconds parameter"
    );
}

#[test]
fn test_launcher_declares_dry_run_parameter() {
    let src = launcher_script_content();
    assert!(
        src.contains("[switch]$DryRun"),
        "script must declare [switch]$DryRun parameter (safe-invocation contract)"
    );
}

#[test]
fn test_launcher_declares_strict_port_parameter() {
    let src = launcher_script_content();
    assert!(
        src.contains("[switch]$StrictPort"),
        "script must declare [switch]$StrictPort parameter"
    );
}

// ---- Max-rounds / bounded-soak env-var contract ---------------------------

#[test]
fn test_launcher_sets_ccgs_bot_max_rounds_when_enabled() {
    let src = launcher_script_content();
    assert!(
        src.contains("CCGS_BOT_MAX_ROUNDS"),
        "script must reference CCGS_BOT_MAX_ROUNDS (bounded-soak server signal)"
    );
}

#[test]
fn test_launcher_max_rounds_default_is_zero_disabled() {
    let src = launcher_script_content();
    // Default must be 0 (disabled); non-zero opts in to bounded-soak mode.
    assert!(
        src.contains("[int]$MaxRounds = 0"),
        "MaxRounds default must be 0 (disabled); server soak is unbounded by default"
    );
}

#[test]
fn test_launcher_conditional_sets_max_rounds_env_var() {
    let src = launcher_script_content();
    // The script must only set the env var when MaxRounds > 0, so a zero value
    // does not accidentally bound a human-operated run.
    assert!(
        src.contains("$MaxRounds -gt 0"),
        "CCGS_BOT_MAX_ROUNDS must be set conditionally (only when MaxRounds > 0)"
    );
}

// ---- Evidence / log path contract -----------------------------------------

#[test]
fn test_launcher_sets_ccgs_bot_decision_log_path() {
    let src = launcher_script_content();
    assert!(
        src.contains("CCGS_BOT_DECISION_LOG_PATH"),
        "script must set CCGS_BOT_DECISION_LOG_PATH (decision-log evidence contract)"
    );
}

#[test]
fn test_launcher_sets_ccgs_qa_snapshot_dir() {
    let src = launcher_script_content();
    assert!(
        src.contains("CCGS_QA_SNAPSHOT_DIR"),
        "script must set CCGS_QA_SNAPSHOT_DIR (snapshot evidence contract)"
    );
}

#[test]
fn test_launcher_evidence_dir_under_production_qa() {
    let src = launcher_script_content();
    assert!(
        src.contains("production/qa/evidence"),
        "evidence directory must be under production/qa/evidence/ (canonical evidence path)"
    );
}

#[test]
fn test_launcher_produces_soak_summary_json() {
    let src = launcher_script_content();
    assert!(
        src.contains("soak-summary.json"),
        "script must write soak-summary.json (structured run artifact)"
    );
}

#[test]
fn test_launcher_decision_log_filename_is_jsonl() {
    let src = launcher_script_content();
    assert!(
        src.contains("bot-decision-log.jsonl"),
        "decision log artifact must be bot-decision-log.jsonl"
    );
}

#[test]
fn test_launcher_snapshots_subdirectory_is_server_snapshots() {
    let src = launcher_script_content();
    assert!(
        src.contains("server-snapshots"),
        "snapshot subdirectory must be named server-snapshots/ (canonical layout)"
    );
}

// ---- Invocation safety (non-destructive defaults) --------------------------

#[test]
fn test_launcher_has_help_flag() {
    let src = launcher_script_content();
    assert!(
        src.contains("[switch]$Help"),
        "script must have a -Help flag so operators can inspect contract without running"
    );
}

#[test]
fn test_launcher_sets_strict_mode() {
    let src = launcher_script_content();
    assert!(
        src.contains("Set-StrictMode"),
        "script must use Set-StrictMode for safe PowerShell execution"
    );
}

#[test]
fn test_launcher_default_port_is_5000() {
    let src = launcher_script_content();
    assert!(
        src.contains("[int]$Port = 5000"),
        "default server port must be 5000"
    );
}
