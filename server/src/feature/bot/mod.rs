//! Bot player feature module — foundation slice (PROMPT 1428).
//!
//! This module is intentionally minimal: only the state surface needed by
//! later phase workers is defined. No systems, no plugin, no protocol.
//! See `reports/PROMPT-1423-bot-player-ai-action-heuristic-audit.md` §8.1 for
//! the staged implementation plan that this scaffold unblocks.

#![allow(dead_code, unused_imports)]

pub mod action_loop;
pub mod debug_push;
pub mod lobby_loop;
pub mod qa_snapshot;
pub mod soak_config;
pub mod state;

pub use action_loop::{bot_action_loop, BotActionLoopPlugin};
pub use debug_push::{
    assemble_debug_bot_state_push, decision_detail, decision_kind_label, BotDebugPushConfig,
    BotDebugPushPlugin, BotDebugPushState, BOT_DEBUG_PUSH_ENV_VAR, DEBUG_BOT_DECISION_TAIL_CAP,
    DEFAULT_BOT_DEBUG_PUSH_INTERVAL_MS,
};
pub use lobby_loop::{bot_lobby_auto_confirm, deterministic_class_for_bot, BotLobbyPlugin};
pub use qa_snapshot::{
    BotQaSnapshotConfig, BotQaSnapshotPlugin, BotQaSnapshotState,
    BOT_DECISION_LOG_PATH_ENV_VAR, BOT_QA_SNAPSHOT_DIR_ENV_VAR, BOT_QA_SNAPSHOT_ENV_VAR,
    DEFAULT_BOT_DECISION_LOG_PATH, DEFAULT_BOT_QA_SNAPSHOT_DIR,
    DEFAULT_PERIODIC_SNAPSHOT_INTERVAL_MS,
};
pub use soak_config::{BotSoakConfig, BotSoakPlugin, BOT_MAX_ROUNDS_ENV_VAR};
pub use state::{
    BotDecisionEntry, BotDecisionKind, BotDecisionLog, BotDifficulty, BotPhaseTiming, BotPlayers,
    BotState, BotThinkDelayWindow, BOT_AUCTION_PASS_THRESHOLD_MS, BOT_SAFETY_MARGIN_MS,
    BOT_THINK_DELAY_MAX_MS, BOT_THINK_DELAY_MIN_MS,
};
