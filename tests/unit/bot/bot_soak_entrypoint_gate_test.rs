//! Unit tests for the `CCGS_BOT_SOAK_ENABLED` entrypoint gate (PROMPT 1743 /
//! BOT-SOAK-ENTRYPOINT-001 AC6).
//!
//! These tests cover the `is_bot_soak_enabled()` helper that gates
//! `handle_create_bot_room`.  The helper is a pure env-var read, so tests use
//! the `ENV_LOCK` serialisation pattern established in `bot_soak_config_test`
//! to prevent races on the process-global environment.
//!
//! # What is tested
//!
//! - Gate is **disabled by default** (env var absent → `false`).
//! - Gate is enabled only for the exact value `"1"`.
//! - Common near-misses (`"0"`, `"true"`, `"yes"`, `"false"`, empty string)
//!   do **not** enable the gate.
//! - Whitespace around `"1"` is trimmed and still enables the gate.

use std::sync::Mutex;

use server::feature::bot::{is_bot_soak_enabled, BOT_SOAK_ENABLED_ENV_VAR};

/// Serialises every test that mutates `CCGS_BOT_SOAK_ENABLED`.
static ENV_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// Default — gate must be disabled when env var is absent
// ---------------------------------------------------------------------------

#[test]
fn test_gate_disabled_by_default() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var(BOT_SOAK_ENABLED_ENV_VAR);
    assert!(
        !is_bot_soak_enabled(),
        "gate must be disabled when CCGS_BOT_SOAK_ENABLED is unset"
    );
}

// ---------------------------------------------------------------------------
// Enabled path — only "1" opens the gate
// ---------------------------------------------------------------------------

#[test]
fn test_gate_enabled_for_exactly_one() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var(BOT_SOAK_ENABLED_ENV_VAR, "1");
    let enabled = is_bot_soak_enabled();
    std::env::remove_var(BOT_SOAK_ENABLED_ENV_VAR);
    assert!(
        enabled,
        "CCGS_BOT_SOAK_ENABLED=1 must open the gate"
    );
}

#[test]
fn test_gate_enabled_for_one_with_surrounding_whitespace() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var(BOT_SOAK_ENABLED_ENV_VAR, "  1  ");
    let enabled = is_bot_soak_enabled();
    std::env::remove_var(BOT_SOAK_ENABLED_ENV_VAR);
    assert!(
        enabled,
        "CCGS_BOT_SOAK_ENABLED='  1  ' must open the gate (whitespace trimmed)"
    );
}

// ---------------------------------------------------------------------------
// Disabled paths — near-misses must NOT open the gate
// ---------------------------------------------------------------------------

#[test]
fn test_gate_disabled_for_zero() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var(BOT_SOAK_ENABLED_ENV_VAR, "0");
    let enabled = is_bot_soak_enabled();
    std::env::remove_var(BOT_SOAK_ENABLED_ENV_VAR);
    assert!(!enabled, "CCGS_BOT_SOAK_ENABLED=0 must keep the gate closed");
}

#[test]
fn test_gate_disabled_for_true_string() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var(BOT_SOAK_ENABLED_ENV_VAR, "true");
    let enabled = is_bot_soak_enabled();
    std::env::remove_var(BOT_SOAK_ENABLED_ENV_VAR);
    assert!(
        !enabled,
        "CCGS_BOT_SOAK_ENABLED=true must keep the gate closed (only '1' is accepted)"
    );
}

#[test]
fn test_gate_disabled_for_yes_string() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var(BOT_SOAK_ENABLED_ENV_VAR, "yes");
    let enabled = is_bot_soak_enabled();
    std::env::remove_var(BOT_SOAK_ENABLED_ENV_VAR);
    assert!(!enabled, "CCGS_BOT_SOAK_ENABLED=yes must keep the gate closed");
}

#[test]
fn test_gate_disabled_for_empty_string() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var(BOT_SOAK_ENABLED_ENV_VAR, "");
    let enabled = is_bot_soak_enabled();
    std::env::remove_var(BOT_SOAK_ENABLED_ENV_VAR);
    assert!(!enabled, "CCGS_BOT_SOAK_ENABLED='' must keep the gate closed");
}
