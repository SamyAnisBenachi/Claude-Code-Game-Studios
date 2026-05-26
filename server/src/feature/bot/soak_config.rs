//! Bot-vs-bot soak session bounding (PROMPT 1640).
//!
//! Adds an opt-in `CCGS_BOT_MAX_ROUNDS` env-var that caps a soak run to a
//! fixed number of rounds and exits cleanly when the limit is reached.  The
//! bound is **never active in normal multiplayer sessions**; it only fires when
//! the env var is set to a non-zero integer.
//!
//! ## Activation contract
//!
//! | Env var              | Purpose                                | Default          |
//! |----------------------|----------------------------------------|------------------|
//! | `CCGS_BOT_MAX_ROUNDS`| Maximum rounds before forced GameOver  | unset = disabled |
//!
//! When `CCGS_BOT_MAX_ROUNDS` is unset or `0` the feature is a true no-op:
//! `BotSoakConfig::max_rounds` is `None` and the RSM transition system skips
//! the bound check entirely.

use bevy::prelude::*;

/// Env var name for the max-rounds soak bound.
pub const BOT_MAX_ROUNDS_ENV_VAR: &str = "CCGS_BOT_MAX_ROUNDS";

/// Server resource injected at startup when `CCGS_BOT_MAX_ROUNDS` is set.
///
/// `max_rounds == None` means the feature is disabled (normal play).
/// `max_rounds == Some(n)` means the RSM will trigger GameOver after `n`
/// completed rounds regardless of objective HP.
#[derive(Resource, Debug, Clone, Default)]
pub struct BotSoakConfig {
    pub max_rounds: Option<u32>,
}

impl BotSoakConfig {
    /// Read the env var and build the config.  Returns `None` if the var is
    /// absent or parses to zero (treat zero as "disabled").
    pub fn from_env() -> Self {
        let max_rounds = std::env::var(BOT_MAX_ROUNDS_ENV_VAR)
            .ok()
            .and_then(|v| v.trim().parse::<u32>().ok())
            .filter(|&n| n > 0);
        if let Some(n) = max_rounds {
            tracing::info!(
                max_rounds = n,
                env_var = BOT_MAX_ROUNDS_ENV_VAR,
                "BotSoakConfig: max-rounds bound active (soak mode)"
            );
        }
        Self { max_rounds }
    }
}

fn startup_insert_soak_config(mut commands: Commands) {
    commands.insert_resource(BotSoakConfig::from_env());
}

/// Plugin — always registered; pays zero runtime cost when env var is unset.
pub struct BotSoakPlugin;

impl Plugin for BotSoakPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup_insert_soak_config);
    }
}
