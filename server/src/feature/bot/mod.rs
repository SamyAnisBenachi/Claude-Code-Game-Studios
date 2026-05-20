//! Bot player feature module — foundation slice (PROMPT 1428).
//!
//! This module is intentionally minimal: only the state surface needed by
//! later phase workers is defined. No systems, no plugin, no protocol.
//! See `reports/PROMPT-1423-bot-player-ai-action-heuristic-audit.md` §8.1 for
//! the staged implementation plan that this scaffold unblocks.

#![allow(dead_code, unused_imports)]

pub mod action_loop;
pub mod lobby_loop;
pub mod state;

pub use action_loop::{bot_action_loop, BotActionLoopPlugin};
pub use lobby_loop::{bot_lobby_auto_confirm, deterministic_class_for_bot, BotLobbyPlugin};
pub use state::{
    BotDecisionEntry, BotDecisionKind, BotDecisionLog, BotDifficulty, BotPhaseTiming, BotPlayers,
    BotState, BotThinkDelayWindow, BOT_AUCTION_PASS_THRESHOLD_MS, BOT_SAFETY_MARGIN_MS,
    BOT_THINK_DELAY_MAX_MS, BOT_THINK_DELAY_MIN_MS,
};
