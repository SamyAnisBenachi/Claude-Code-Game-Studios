//! Bot player foundation state (PROMPT 1423 §3.1, §6.1; PROMPT 1428 Phase 1).
//!
//! Additive scaffold only. No runtime systems, no scheduler wiring, no
//! gameplay decisions. Provides the stable resource/types shape later phase
//! workers will populate without rewriting the foundation.
//!
//! Design references:
//! - PROMPT-1423 audit §1.2 (`peer_id: Option<PeerId>` is bot-friendly).
//! - PROMPT-1423 audit §3.1 (deterministic seed, separate from `ServerRng`).
//! - PROMPT-1423 audit §3.3 (think-delay / safety-margin defaults).
//! - PROMPT-1423 audit §6.1 (`BotDecisionLog` / `BotDecisionEntry` shape).
//!
//! ADR-005: bot RNG MUST remain isolated from `ServerRng` so the authoritative
//! audit log only enumerates ADR-005 `RngEvent` variants. The bot owns its own
//! `ChaCha8Rng` seed; that seed and its counter are recorded in the local
//! `BotDecisionLog`, never in `ServerRng.audit_log`.

#![allow(dead_code)]

use std::collections::HashMap;

use bevy::prelude::Resource;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use shared::card::{CardId, ClassId};
use shared::protocol::{CardSource, RoundPhase};
use shared::session::PlayerId;

/// Lower jitter bound for any bot per-decision think delay.
pub const BOT_THINK_DELAY_MIN_MS: u32 = 300;
/// Upper jitter bound for any bot per-decision think delay.
pub const BOT_THINK_DELAY_MAX_MS: u32 = 1_200;
/// Margin subtracted from a phase timer before the bot's fail-safe arms.
pub const BOT_SAFETY_MARGIN_MS: u32 = 800;
/// Auction-pass cutoff: with less time than this remaining the bot will not bid.
pub const BOT_AUCTION_PASS_THRESHOLD_MS: u32 = 500;

/// MVP difficulty enum. Single variant for the foundation slice; later phases
/// may add `Easy` / `Hard` without breaking the `BotState` shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BotDifficulty {
    #[default]
    Mvp,
}

/// Bounded random-jitter window used between consecutive bot decisions inside
/// a single phase. Both bounds are inclusive and measured in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BotThinkDelayWindow {
    pub min_ms: u32,
    pub max_ms: u32,
}

impl BotThinkDelayWindow {
    pub const fn new(min_ms: u32, max_ms: u32) -> Self {
        Self { min_ms, max_ms }
    }
}

impl Default for BotThinkDelayWindow {
    fn default() -> Self {
        Self::new(BOT_THINK_DELAY_MIN_MS, BOT_THINK_DELAY_MAX_MS)
    }
}

/// Per-phase cooldown / fail-safe state. Each phase the bot participates in
/// records the wallclock cutoff at which its fail-safe is armed; later phase
/// systems compare the cutoff against `Time` and force a deterministic no-op
/// submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BotPhaseTiming {
    /// Earliest wallclock ms at which the next bot decision may run.
    /// `None` means "no decision pending" or "bot has not entered this phase".
    pub next_decision_at_ms: Option<u64>,
    /// Wallclock ms at which the bot MUST submit a fail-safe action,
    /// regardless of heuristic state. `None` means the fail-safe is not armed.
    pub failsafe_deadline_ms: Option<u64>,
}

/// Authoritative per-bot state. Stored under [`BotPlayers`] keyed by the
/// bot's `PlayerId`. All fields are owned by the bot domain; no other system
/// should mutate this struct.
#[derive(Debug)]
pub struct BotState {
    /// Stable session-scoped identity. Mirrors the map key for ergonomic use.
    pub player_id: PlayerId,
    /// Difficulty / heuristic family. Default `Mvp` for the foundation slice.
    pub difficulty: BotDifficulty,
    /// Raw seed used to construct `rng`. Persisted so a future replay tool
    /// can reproduce bot decisions without re-deriving the hash.
    pub rng_seed: u64,
    /// Bot-private RNG, isolated from `ServerRng` per ADR-005. Advanced
    /// only at bot decision points; counter snapshotted into the decision log.
    pub rng: ChaCha8Rng,
    /// Number of `u64` words the bot has drawn from `rng`. Mirrors the
    /// audit-log seed-index discipline used by `ServerRng`, scoped to the bot.
    pub rng_word_counter: u64,
    /// Wallclock ms at which the last bot decision (any phase) was emitted.
    /// `None` means the bot has not acted yet this session.
    pub last_decision_at_ms: Option<u64>,
    /// Per-decision random-jitter window. Defaults to the audit's
    /// `[BOT_THINK_DELAY_MIN_MS, BOT_THINK_DELAY_MAX_MS]` window.
    pub think_delay: BotThinkDelayWindow,
    /// Per-phase cooldown + fail-safe deadline state.
    pub phase_timing: BotPhaseTiming,
    /// Optional class choice cache. Foundation slice leaves this `None`; the
    /// lobby system in a later phase records the picked class here once.
    pub class_choice: Option<ClassId>,
}

impl BotState {
    /// Build a fresh bot state for `player_id` from a deterministic `seed`.
    ///
    /// Per PROMPT-1423 audit §3.1 the seed is derived externally (typically
    /// from `(session_id, player_id)`) so this foundation is agnostic to the
    /// hash source. The bot's RNG and word counter are initialised in lockstep
    /// so audit-log entries can quote both.
    pub fn new(player_id: PlayerId, seed: u64) -> Self {
        Self {
            player_id,
            difficulty: BotDifficulty::default(),
            rng_seed: seed,
            rng: ChaCha8Rng::seed_from_u64(seed),
            rng_word_counter: 0,
            last_decision_at_ms: None,
            think_delay: BotThinkDelayWindow::default(),
            phase_timing: BotPhaseTiming::default(),
            class_choice: None,
        }
    }
}

/// Bevy resource holding every bot present in the current session. Keyed by
/// the bot's `PlayerId`. Empty in production sessions with two human peers.
///
/// `BotPlayers` is the single source of truth for "is this `PlayerId` a bot?"
/// — later systems gate decision ticks on `bots.contains(player)`.
#[derive(Resource, Default, Debug)]
pub struct BotPlayers {
    pub bots: HashMap<PlayerId, BotState>,
}

impl BotPlayers {
    pub fn insert(&mut self, state: BotState) {
        self.bots.insert(state.player_id, state);
    }

    pub fn contains(&self, player_id: PlayerId) -> bool {
        self.bots.contains_key(&player_id)
    }

    pub fn get(&self, player_id: PlayerId) -> Option<&BotState> {
        self.bots.get(&player_id)
    }

    pub fn get_mut(&mut self, player_id: PlayerId) -> Option<&mut BotState> {
        self.bots.get_mut(&player_id)
    }

    pub fn len(&self) -> usize {
        self.bots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bots.is_empty()
    }
}

/// Classifies a single bot decision for the server-only audit log.
///
/// Variants mirror PROMPT-1423 audit §6.1. Reason strings are `&'static str`
/// so the log avoids per-entry allocation; later phase systems are expected
/// to use a small closed set of literals per decision branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BotDecisionKind {
    ClassChosen { class_id: ClassId },
    ClassConfirmed,
    Purchased {
        card_id: CardId,
        source: CardSource,
        gold_after: u32,
    },
    Refreshed { gold_after: u32 },
    PurchaseSkipped { reason: &'static str },
    DraftReady,
    AuctionBid {
        card_id: CardId,
        amount: u32,
        valuation: u32,
    },
    AuctionPass { reason: &'static str },
    PlacementSubmitted { placements_len: u8 },
    PlacementSkipped { reason: &'static str },
    EmptyPlacementFailsafe,
    ResultAcknowledged,
}

/// One append-only audit entry. The bot domain writes one of these per
/// decision branch so a later VERIFY lane can replay/inspect every action
/// the bot took.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotDecisionEntry {
    /// Server round counter the decision belongs to. `0` before the first
    /// round (lobby / class choice).
    pub round_number: u32,
    /// RSM phase at the moment the bot recorded the decision.
    pub phase: RoundPhase,
    /// Which bot acted.
    pub bot_player_id: PlayerId,
    /// What it did (or why it deliberately did nothing).
    pub decision: BotDecisionKind,
    /// Server wallclock ms when the decision was recorded.
    pub timestamp_ms: u64,
    /// How many legal options the heuristic considered. `None` when the
    /// decision was administrative (e.g. ack) and a count is meaningless.
    pub legal_action_count: Option<u32>,
    /// Bot RNG seed snapshot. Mirrors `BotState.rng_seed`; recorded per entry
    /// so a log fragment is self-contained for replay.
    pub seed: u64,
    /// Word counter snapshot taken AFTER the bot consumed any RNG for this
    /// decision. Equal to the previous entry's counter when the decision
    /// consumed no entropy.
    pub seed_word_counter: u64,
}

/// Server-only audit log of bot decisions. Never replicated to clients.
///
/// Bounded growth: callers append once per decision; the log is cleared on
/// session teardown. A later VERIFY lane (PROMPT-1423 audit §6.2) may dump
/// the log to disk under `production/qa/evidence/` at game-over.
#[derive(Resource, Default, Debug)]
pub struct BotDecisionLog {
    pub entries: Vec<BotDecisionEntry>,
}

impl BotDecisionLog {
    pub fn push(&mut self, entry: BotDecisionEntry) {
        self.entries.push(entry);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn last(&self) -> Option<&BotDecisionEntry> {
        self.entries.last()
    }
}
