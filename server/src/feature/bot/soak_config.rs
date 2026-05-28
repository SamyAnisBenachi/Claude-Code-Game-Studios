//! Bot-vs-bot soak session bounding (PROMPT 1640) and entrypoint gate (PROMPT 1743).
//!
//! Adds an opt-in `CCGS_BOT_MAX_ROUNDS` env-var that caps a soak run to a
//! fixed number of rounds and exits cleanly when the limit is reached.  The
//! bound is **never active in normal multiplayer sessions**; it only fires when
//! the env var is set to a non-zero integer.
//!
//! ## Activation contract
//!
//! | Env var                | Purpose                                | Default          |
//! |------------------------|----------------------------------------|------------------|
//! | `CCGS_BOT_SOAK_ENABLED`| Gate server-side bot room creation     | unset = blocked  |
//! | `CCGS_BOT_MAX_ROUNDS`  | Maximum rounds before forced GameOver  | unset = disabled |
//!
//! `CCGS_BOT_SOAK_ENABLED=1` must be set for the server to honour
//! `C2SCreateBotRoom` messages.  When absent (or any value other than `"1"`),
//! `handle_create_bot_room` silently drains all incoming messages without
//! creating a room.  This prevents accidental bot-soak entrypoint exposure in
//! release/operator environments where neither env var is set.
//!
//! When `CCGS_BOT_MAX_ROUNDS` is unset or `0` the round-bounding feature is a
//! true no-op: `BotSoakConfig::max_rounds` is `None` and the RSM transition
//! system skips the bound check entirely.

use bevy::prelude::*;

/// Env var that gates server-side bot room creation (PROMPT 1743).
///
/// Set to exactly `"1"` to allow `C2SCreateBotRoom` messages to be processed.
/// Any other value (including unset) causes the handler to silently drain and
/// discard all incoming requests.
pub const BOT_SOAK_ENABLED_ENV_VAR: &str = "CCGS_BOT_SOAK_ENABLED";

/// Returns `true` iff `CCGS_BOT_SOAK_ENABLED` is set to exactly `"1"` (after
/// trimming whitespace).  All other values — including unset — return `false`.
///
/// This is a **process-level read** performed once per call; callers that need
/// the result multiple times per frame should cache it locally.
pub fn is_bot_soak_enabled() -> bool {
    std::env::var(BOT_SOAK_ENABLED_ENV_VAR)
        .map(|v| v.trim() == "1")
        .unwrap_or(false)
}

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
