//! Server-side QA snapshot + bot-decision-log streamer (PROMPT 1597).
//!
//! Authoritative server-side QA evidence for bot-driven flows. Produces two
//! kinds of artefacts under a configurable output directory:
//!
//! 1. **Snapshot JSON files** (`snapshot-<round>-<phase>-<ts>-<seq>.json`) —
//!    serialised view of every observable resource useful for bot debugging
//!    (phase, RSM timers, both hands, board occupancy, economies, auction,
//!    objectives, bot RNG counters, decision-log tail). Written on:
//!    - phase transitions (any change in `RoundState.phase`),
//!    - periodic interval (default every 10 s, configurable in code),
//!    - best-effort graceful shutdown (one final dump when `AppExit` fires).
//! 2. **Bot decision-log JSONL** (`bot-decision-log.jsonl`) — one JSON object
//!    per [`BotDecisionEntry`] appended in order and `flush()`-ed after every
//!    append so the file is durable even if the process dies between events.
//!
//! ## Activation contract
//!
//! Mirrors the client-side QA snapshot env-var convention
//! (`CCGS_QA_SNAPSHOT*`). Server-side variables are namespaced under
//! `CCGS_BOT_*` so the two subsystems can be toggled independently:
//!
//! | Env var | Purpose | Default |
//! |---|---|---|
//! | `CCGS_BOT_QA_SNAPSHOT` | `1` forces enabled, `0` disabled, unset = `cfg!(debug_assertions)` | dev: enabled, release: disabled |
//! | `CCGS_BOT_QA_SNAPSHOT_DIR` | Output directory for snapshot JSONs | `dev-runs/bot-qa-snapshots` |
//! | `CCGS_BOT_DECISION_LOG_PATH` | Path of JSONL decision-log file | `dev-runs/bot-decision-log.jsonl` |
//!
//! The server does not consume CLI arguments today (only `SERVER_PORT` is
//! parsed in `network::mod`), so env vars are the project-conventional knob.
//!
//! ## Non-product rule
//!
//! - The plugin is always registered; every system early-returns when the
//!   config is disabled, so the production server pays zero cost.
//! - No semantic gameplay mutation. The snapshot reads observable resources
//!   via `Option<Res<…>>` so missing-resource test scaffolds compile and the
//!   plugin is a strict observer.
//! - No new protocol messages, no client surface, no replication.
//! - Evidence is **best-effort**: every I/O failure is logged at `warn` and
//!   the system continues. Snapshots are diagnostic, not authoritative.
//!
//! ## Scope discipline (PROMPT 1597 owned scope)
//!
//! Owned files: `server/src/feature/bot/qa_snapshot.rs` (this file),
//! `server/src/feature/bot/mod.rs` (re-export + plugin wire), `server/src/main.rs`
//! (plugin registration), `server/Cargo.toml` (test wiring), and the unit test
//! under `tests/unit/bot/`.

#![allow(dead_code)]

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use bevy::prelude::*;
use serde::Serialize;
use shared::card::{CardId, ClassId};
use shared::protocol::RoundPhase as WireRoundPhase;
use shared::session::PlayerId;

use crate::core::economy::PlayerEconomies;
use crate::core::rsm::state::{RoundPhase, RoundState};
use crate::core::session::config::SessionConfig;
use crate::feature::acquisition::PlayerHands;
use crate::feature::auction::{AuctionPhase, AuctionState};
use crate::feature::board::state::{BoardOccupancy, BOARD_CELLS_PER_LANE, BOARD_LANE_COUNT};
use crate::feature::bot::state::{
    BotDecisionEntry, BotDecisionKind, BotDecisionLog, BotPlayers,
};
use crate::feature::objective::state::{ObjectiveHp, ObjectiveSlot};

/// Env var: enables/disables the server-side bot QA snapshot subsystem.
pub const BOT_QA_SNAPSHOT_ENV_VAR: &str = "CCGS_BOT_QA_SNAPSHOT";
/// Env var: overrides the output directory for snapshot JSON files.
pub const BOT_QA_SNAPSHOT_DIR_ENV_VAR: &str = "CCGS_BOT_QA_SNAPSHOT_DIR";
/// Env var: overrides the JSONL path of the streamed bot decision log.
pub const BOT_DECISION_LOG_PATH_ENV_VAR: &str = "CCGS_BOT_DECISION_LOG_PATH";

/// Default output directory for snapshot JSON files. Sits under repo-root
/// `dev-runs/` per the inventory follow-up's "evidence under dev-runs" rule.
pub const DEFAULT_BOT_QA_SNAPSHOT_DIR: &str = "dev-runs/bot-qa-snapshots";
/// Default JSONL path of the streamed bot decision log.
pub const DEFAULT_BOT_DECISION_LOG_PATH: &str = "dev-runs/bot-decision-log.jsonl";

/// Default periodic snapshot interval (milliseconds). PROMPT 1594 follow-up
/// pinned the cadence at 10 s.
pub const DEFAULT_PERIODIC_SNAPSHOT_INTERVAL_MS: u64 = 10_000;

/// Snapshot schema version. Bump when the JSON shape changes in a way that
/// breaks downstream evidence parsers.
pub const BOT_QA_SNAPSHOT_SCHEMA_VERSION: u32 = 1;

/// Hard cap on the decision-log tail embedded in each snapshot. The full log
/// already streams to `bot-decision-log.jsonl`; the tail is a convenience for
/// readers inspecting a single snapshot in isolation.
pub const DECISION_LOG_TAIL_CAP: usize = 64;

// ---------------------------------------------------------------------------
// Config + state resources
// ---------------------------------------------------------------------------

/// Server-side bot QA snapshot configuration.
///
/// Populated from environment variables at plugin build time. Tests can insert
/// the resource directly before adding the plugin so the env vars never need
/// to be touched.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct BotQaSnapshotConfig {
    /// When `false`, every snapshot/streamer system is a no-op.
    pub enabled: bool,
    /// Directory snapshot JSON files are written into.
    pub snapshot_dir: PathBuf,
    /// JSONL file the decision-log streamer appends to.
    pub decision_log_path: PathBuf,
    /// Minimum interval between two periodic snapshots, in milliseconds.
    pub periodic_interval_ms: u64,
}

impl Default for BotQaSnapshotConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            snapshot_dir: PathBuf::from(DEFAULT_BOT_QA_SNAPSHOT_DIR),
            decision_log_path: PathBuf::from(DEFAULT_BOT_DECISION_LOG_PATH),
            periodic_interval_ms: DEFAULT_PERIODIC_SNAPSHOT_INTERVAL_MS,
        }
    }
}

impl BotQaSnapshotConfig {
    /// Build a config from environment variables.
    ///
    /// Activation rule for [`BOT_QA_SNAPSHOT_ENV_VAR`]:
    /// - `"1"` forces enabled.
    /// - `"0"` forces disabled.
    /// - Unset / empty / whitespace-only: defaults to `dev_default_enabled`
    ///   (production callers pass `cfg!(debug_assertions)`).
    /// - Any other value is logged and treated as disabled (never panics).
    ///
    /// Blank `*_DIR` / `*_PATH` values fall back to the default.
    pub fn from_env() -> Self {
        Self::from_env_values(
            std::env::var(BOT_QA_SNAPSHOT_ENV_VAR).ok().as_deref(),
            std::env::var(BOT_QA_SNAPSHOT_DIR_ENV_VAR).ok().as_deref(),
            std::env::var(BOT_DECISION_LOG_PATH_ENV_VAR).ok().as_deref(),
            cfg!(debug_assertions),
        )
    }

    /// Deterministic constructor used by [`from_env`](Self::from_env) and the
    /// unit tests so the env-parsing rules can be exercised without touching
    /// the process environment.
    pub fn from_env_values(
        enable_var: Option<&str>,
        dir_var: Option<&str>,
        decision_log_var: Option<&str>,
        dev_default_enabled: bool,
    ) -> Self {
        let enabled = match enable_var.map(str::trim) {
            None | Some("") => dev_default_enabled,
            Some("1") => true,
            Some("0") => false,
            Some(other) => {
                tracing::warn!(
                    target: "server::bot::qa_snapshot",
                    value = %other,
                    "{} has invalid value; treating as disabled (expected 1, 0, or unset)",
                    BOT_QA_SNAPSHOT_ENV_VAR,
                );
                false
            }
        };
        let snapshot_dir = dir_var
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BOT_QA_SNAPSHOT_DIR));
        let decision_log_path = decision_log_var
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BOT_DECISION_LOG_PATH));
        Self {
            enabled,
            snapshot_dir,
            decision_log_path,
            periodic_interval_ms: DEFAULT_PERIODIC_SNAPSHOT_INTERVAL_MS,
        }
    }
}

/// Mutable runtime state for the snapshot/streamer systems.
///
/// Kept in a single resource so the system signatures stay small. The
/// decision-log writer is wrapped in a `Mutex` so the resource is `Sync`
/// without requiring the streamer system to be exclusive — Bevy's scheduler
/// only ever runs one instance of a given system at a time, but `&mut Resource`
/// access from multiple systems in the same schedule would otherwise force
/// serialisation we'd rather express explicitly.
#[derive(Resource, Default)]
pub struct BotQaSnapshotState {
    /// Last RSM phase observed by the writer. `None` until the first frame
    /// with a `RoundState` resource is processed.
    pub last_phase: Option<WireRoundPhase>,
    /// Monotonic ms timestamp at which the next periodic snapshot may fire.
    /// `0` means "fire on the next tick" (used at startup so the operator
    /// gets an initial snapshot without waiting a full interval).
    pub next_periodic_ms: u64,
    /// Number of `BotDecisionLog` entries already streamed to JSONL. The
    /// streamer appends `entries[offset..]` then bumps `offset` to `entries.len()`.
    pub decision_log_offset: usize,
    /// Monotonic per-process counter so concurrent snapshots with identical
    /// `(round, phase, ts)` produce distinct filenames.
    pub sequence: u64,
    /// Buffered handle for `bot-decision-log.jsonl`. Opened lazily on first
    /// append; `None` means "not opened yet (or last open failed)".
    pub decision_log_writer: Mutex<Option<BufWriter<File>>>,
    /// Path the open `decision_log_writer` was opened against. Re-opens on
    /// path change (in practice: config edits / tests reusing the same
    /// process).
    pub decision_log_writer_path: Option<PathBuf>,
    /// Set to `true` once the graceful-shutdown dump has been written so the
    /// system does not write a second one if `AppExit` fires repeatedly.
    pub shutdown_dump_done: bool,
}

// ---------------------------------------------------------------------------
// Serialisable snapshot model
// ---------------------------------------------------------------------------

/// Top-level snapshot document. One JSON file per write.
#[derive(Serialize, Debug)]
pub struct BotQaSnapshot {
    /// Snapshot schema version. Mirrors [`BOT_QA_SNAPSHOT_SCHEMA_VERSION`].
    pub schema_version: u32,
    /// What caused this snapshot to be written.
    pub trigger: SnapshotTrigger,
    /// Server wallclock (ms since `Time::elapsed`).
    pub timestamp_ms: u64,
    /// Per-process monotonic sequence number assigned at write time.
    pub sequence: u64,
    /// RSM-derived fields. `None` when no `RoundState` resource exists.
    pub round: Option<RoundSnapshot>,
    /// Session config snapshot (player roster + teams + classes).
    pub session: Option<SessionSnapshot>,
    /// Auction snapshot. `None` when no auction is running.
    pub auction: Option<AuctionSnapshot>,
    /// Per-player economy snapshot.
    pub economies: Vec<EconomySnapshot>,
    /// Per-player hand snapshot.
    pub hands: Vec<HandSnapshot>,
    /// Board occupancy summary.
    pub board: Option<BoardSnapshot>,
    /// Per-player objective HP / destruction state.
    pub objectives: Vec<ObjectiveSnapshot>,
    /// Per-bot state (seed, RNG counter, phase timing, last decision).
    pub bots: Vec<BotStateSnapshot>,
    /// Tail of the `BotDecisionLog` capped at [`DECISION_LOG_TAIL_CAP`].
    pub decision_log_tail: Vec<DecisionEntrySnapshot>,
    /// Total decision-log entry count (full count, not the cap).
    pub decision_log_total: usize,
}

/// What triggered a snapshot write.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotTrigger {
    /// First snapshot after the subsystem activated.
    Initial,
    /// `RoundState.phase` changed since the last snapshot.
    PhaseTransition,
    /// Periodic 10-second-by-default tick.
    Periodic,
    /// Best-effort dump triggered by `AppExit`.
    GracefulShutdown,
}

#[derive(Serialize, Debug)]
pub struct RoundSnapshot {
    pub phase: WireRoundPhase,
    pub round_number: u32,
    pub draft_ready_players: Vec<PlayerId>,
    pub submissions_received: Vec<PlayerId>,
    pub disconnect_trackers: Vec<DisconnectTrackerEntry>,
    pub timers_ms: RoundTimersSnapshot,
}

#[derive(Serialize, Debug)]
pub struct DisconnectTrackerEntry {
    pub player: PlayerId,
    pub seconds_since_disconnect: u32,
}

/// Per-timer remaining time, in milliseconds. `None` means the timer is not
/// armed (or the underlying `Timer` resource is absent).
#[derive(Serialize, Debug, Default)]
pub struct RoundTimersSnapshot {
    pub placement: Option<u32>,
    pub placement_grace: Option<u32>,
    pub draft_initial: Option<u32>,
    pub draft_shop: Option<u32>,
    pub auction_safety: Option<u32>,
    pub resolution_safety: Option<u32>,
}

#[derive(Serialize, Debug)]
pub struct SessionSnapshot {
    pub mode: String,
    pub player_count: u8,
    pub players: Vec<SessionPlayerSnapshot>,
    pub placement_timer_multiplier: String,
}

#[derive(Serialize, Debug)]
pub struct SessionPlayerSnapshot {
    pub player: PlayerId,
    pub team: u8,
    pub class: Option<ClassId>,
    pub is_bot: bool,
}

#[derive(Serialize, Debug)]
pub struct AuctionSnapshot {
    pub phase: &'static str,
    pub card_id: Option<CardId>,
    pub starting_price: u32,
    pub current_price: u32,
    pub current_leader: Option<PlayerId>,
    pub timer_remaining_ms: u32,
    pub live_bidding_deadline_elapsed_ms: Option<u64>,
}

#[derive(Serialize, Debug)]
pub struct EconomySnapshot {
    pub player: PlayerId,
    pub gold: u32,
    pub current_mana: u32,
    pub reserve_mana: u32,
    pub mana_cap: u32,
    pub reserved_gold: u32,
}

#[derive(Serialize, Debug)]
pub struct HandSnapshot {
    pub player: PlayerId,
    pub size: usize,
    pub cards: Vec<CardId>,
}

#[derive(Serialize, Debug)]
pub struct BoardSnapshot {
    pub minion_count: usize,
    pub trap_count: usize,
    pub structure_count: usize,
    pub field_count: usize,
    /// Per-lane minion occupancy summary keyed by player id.
    pub per_player_minions: Vec<BoardPerPlayerSnapshot>,
}

#[derive(Serialize, Debug)]
pub struct BoardPerPlayerSnapshot {
    pub player: PlayerId,
    pub occupied_lanes: Vec<u8>,
}

#[derive(Serialize, Debug)]
pub struct ObjectiveSnapshot {
    pub player: PlayerId,
    pub lane: u8,
    pub hp: u32,
    pub destroyed: bool,
}

#[derive(Serialize, Debug)]
pub struct BotStateSnapshot {
    pub player: PlayerId,
    pub difficulty: &'static str,
    pub rng_seed: u64,
    pub rng_word_counter: u64,
    pub last_decision_at_ms: Option<u64>,
    pub class_choice: Option<ClassId>,
    pub next_decision_at_ms: Option<u64>,
    pub failsafe_deadline_ms: Option<u64>,
}

/// Wire shape for one [`BotDecisionEntry`]. Mirrors the in-memory struct but
/// flattens the [`BotDecisionKind`] enum into a tagged JSON object so the
/// snapshot file is human-readable without a manual `serde_json` parser.
#[derive(Serialize, Debug, Clone)]
pub struct DecisionEntrySnapshot {
    pub round_number: u32,
    pub phase: WireRoundPhase,
    pub bot_player_id: PlayerId,
    pub timestamp_ms: u64,
    pub seed: u64,
    pub seed_word_counter: u64,
    pub legal_action_count: Option<u32>,
    pub decision: DecisionKindSnapshot,
}

/// Tagged-object form of [`BotDecisionKind`] for JSON output. The `kind` tag
/// is the camelCase enum variant; remaining fields belong to the variant.
#[derive(Serialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DecisionKindSnapshot {
    ClassChosen { class_id: ClassId },
    ClassConfirmed,
    Purchased { card_id: CardId, source: String, gold_after: u32 },
    Refreshed { gold_after: u32 },
    PurchaseSkipped { reason: String },
    DraftReady,
    AuctionBid { card_id: CardId, amount: u32, valuation: u32 },
    AuctionPass { reason: String },
    PlacementSubmitted { placements_len: u8 },
    PlacementSkipped { reason: String },
    EmptyPlacementFailsafe,
    ResultAcknowledged,
}

impl From<&BotDecisionEntry> for DecisionEntrySnapshot {
    fn from(entry: &BotDecisionEntry) -> Self {
        Self {
            round_number: entry.round_number,
            phase: entry.phase,
            bot_player_id: entry.bot_player_id,
            timestamp_ms: entry.timestamp_ms,
            seed: entry.seed,
            seed_word_counter: entry.seed_word_counter,
            legal_action_count: entry.legal_action_count,
            decision: (&entry.decision).into(),
        }
    }
}

impl From<&BotDecisionKind> for DecisionKindSnapshot {
    fn from(kind: &BotDecisionKind) -> Self {
        match kind {
            BotDecisionKind::ClassChosen { class_id } => {
                DecisionKindSnapshot::ClassChosen { class_id: *class_id }
            }
            BotDecisionKind::ClassConfirmed => DecisionKindSnapshot::ClassConfirmed,
            BotDecisionKind::Purchased { card_id, source, gold_after } => {
                DecisionKindSnapshot::Purchased {
                    card_id: *card_id,
                    source: format!("{:?}", source),
                    gold_after: *gold_after,
                }
            }
            BotDecisionKind::Refreshed { gold_after } => {
                DecisionKindSnapshot::Refreshed { gold_after: *gold_after }
            }
            BotDecisionKind::PurchaseSkipped { reason } => {
                DecisionKindSnapshot::PurchaseSkipped { reason: (*reason).to_string() }
            }
            BotDecisionKind::DraftReady => DecisionKindSnapshot::DraftReady,
            BotDecisionKind::AuctionBid { card_id, amount, valuation } => {
                DecisionKindSnapshot::AuctionBid {
                    card_id: *card_id,
                    amount: *amount,
                    valuation: *valuation,
                }
            }
            BotDecisionKind::AuctionPass { reason } => {
                DecisionKindSnapshot::AuctionPass { reason: (*reason).to_string() }
            }
            BotDecisionKind::PlacementSubmitted { placements_len } => {
                DecisionKindSnapshot::PlacementSubmitted { placements_len: *placements_len }
            }
            BotDecisionKind::PlacementSkipped { reason } => {
                DecisionKindSnapshot::PlacementSkipped { reason: (*reason).to_string() }
            }
            BotDecisionKind::EmptyPlacementFailsafe => {
                DecisionKindSnapshot::EmptyPlacementFailsafe
            }
            BotDecisionKind::ResultAcknowledged => DecisionKindSnapshot::ResultAcknowledged,
        }
    }
}

// ---------------------------------------------------------------------------
// Snapshot assembly + write helpers
// ---------------------------------------------------------------------------

/// Read-only references aggregated for one snapshot write. Pulled into a
/// dedicated struct so the writer-system and shutdown-system can build it
/// uniformly without duplicating the `Option<Res<…>>` plumbing.
pub struct SnapshotInputs<'a> {
    pub timestamp_ms: u64,
    pub round_state: Option<&'a RoundState>,
    pub session: Option<&'a SessionConfig>,
    pub bots: Option<&'a BotPlayers>,
    pub decision_log: Option<&'a BotDecisionLog>,
    pub auction: Option<&'a AuctionState>,
    pub economies: Option<&'a PlayerEconomies>,
    pub hands: Option<&'a PlayerHands>,
    pub board: Option<&'a BoardOccupancy>,
    pub objectives: Vec<(ObjectiveSlot, ObjectiveHp)>,
}

/// Assemble a [`BotQaSnapshot`] from observed resources.
///
/// Pure function: builds a serializable struct with no I/O. The writer
/// systems thread the snapshot through `serde_json::to_writer_pretty`.
pub fn assemble_snapshot(
    trigger: SnapshotTrigger,
    sequence: u64,
    inputs: &SnapshotInputs<'_>,
) -> BotQaSnapshot {
    let round = inputs.round_state.map(|rs| RoundSnapshot {
        phase: protocol_phase(rs.phase),
        round_number: rs.round_number,
        draft_ready_players: sorted_players(rs.draft_ready_players.iter().copied()),
        submissions_received: sorted_players(rs.submissions_received.iter().copied()),
        disconnect_trackers: {
            let mut v: Vec<_> = rs
                .disconnect_trackers
                .iter()
                .map(|(p, s)| DisconnectTrackerEntry {
                    player: *p,
                    seconds_since_disconnect: *s,
                })
                .collect();
            v.sort_by_key(|e| e.player.0);
            v
        },
        timers_ms: RoundTimersSnapshot {
            placement: rs.placement_timer.as_ref().map(timer_remaining_ms),
            placement_grace: rs
                .placement_deadline_grace_timer
                .as_ref()
                .map(timer_remaining_ms),
            draft_initial: rs.draft_initial_timer.as_ref().map(timer_remaining_ms),
            draft_shop: rs.draft_shop_timer.as_ref().map(timer_remaining_ms),
            auction_safety: rs.auction_safety_timer.as_ref().map(timer_remaining_ms),
            resolution_safety: rs
                .resolution_safety_timer
                .as_ref()
                .map(timer_remaining_ms),
        },
    });

    let bot_player_set: HashSet<PlayerId> = inputs
        .bots
        .map(|b| b.bots.keys().copied().collect())
        .unwrap_or_default();

    let session = inputs.session.map(|cfg| SessionSnapshot {
        mode: format!("{:?}", cfg.mode),
        player_count: cfg.player_count,
        placement_timer_multiplier: format!("{:?}", cfg.placement_timer_multiplier_effective),
        players: {
            let mut v: Vec<_> = cfg
                .players()
                .map(|p| SessionPlayerSnapshot {
                    player: p,
                    team: cfg.team_map.get(&p).copied().unwrap_or(u8::MAX),
                    class: cfg.class_map.get(&p).copied(),
                    is_bot: bot_player_set.contains(&p),
                })
                .collect();
            v.sort_by_key(|p| p.player.0);
            v
        },
    });

    let auction = inputs.auction.map(|a| AuctionSnapshot {
        phase: auction_phase_label(a.phase),
        card_id: a.card_id,
        starting_price: a.starting_price,
        current_price: a.current_price,
        current_leader: a.current_leader,
        timer_remaining_ms: a.timer_remaining_ms,
        live_bidding_deadline_elapsed_ms: a.live_bidding_deadline_elapsed_ms,
    });

    let economies = inputs
        .economies
        .map(|e| {
            let mut v: Vec<_> = e
                .0
                .iter()
                .map(|(p, econ)| EconomySnapshot {
                    player: *p,
                    gold: econ.gold,
                    current_mana: econ.current_mana,
                    reserve_mana: econ.reserve_mana,
                    mana_cap: econ.mana_cap,
                    reserved_gold: econ.reserved_gold,
                })
                .collect();
            v.sort_by_key(|s| s.player.0);
            v
        })
        .unwrap_or_default();

    let hands = inputs
        .hands
        .map(|h| {
            let mut v: Vec<_> = h
                .hands
                .iter()
                .map(|(p, cards)| HandSnapshot {
                    player: *p,
                    size: cards.len(),
                    cards: cards.clone(),
                })
                .collect();
            v.sort_by_key(|h| h.player.0);
            v
        })
        .unwrap_or_default();

    let board = inputs.board.map(|occ| {
        let mut per_player: std::collections::HashMap<PlayerId, Vec<u8>> =
            std::collections::HashMap::new();
        for (player, lane) in occ.minion_slots.keys() {
            per_player.entry(*player).or_default().push(*lane);
        }
        let mut per_player_minions: Vec<_> = per_player
            .into_iter()
            .map(|(player, mut occupied_lanes)| {
                occupied_lanes.sort_unstable();
                occupied_lanes.dedup();
                BoardPerPlayerSnapshot { player, occupied_lanes }
            })
            .collect();
        per_player_minions.sort_by_key(|s| s.player.0);
        BoardSnapshot {
            minion_count: occ.minion_slots.len(),
            trap_count: occ.traps.len(),
            structure_count: occ.structures.len(),
            field_count: occ.fields.len(),
            per_player_minions,
        }
    });

    let mut objectives: Vec<_> = inputs
        .objectives
        .iter()
        .map(|(slot, hp)| ObjectiveSnapshot {
            player: slot.player,
            lane: slot.lane,
            hp: hp.hp,
            destroyed: slot.destroyed,
        })
        .collect();
    objectives.sort_by_key(|o| (o.player.0, o.lane));

    let bots = inputs
        .bots
        .map(|b| {
            let mut v: Vec<_> = b
                .bots
                .iter()
                .map(|(player, state)| BotStateSnapshot {
                    player: *player,
                    difficulty: bot_difficulty_label(state),
                    rng_seed: state.rng_seed,
                    rng_word_counter: state.rng_word_counter,
                    last_decision_at_ms: state.last_decision_at_ms,
                    class_choice: state.class_choice,
                    next_decision_at_ms: state.phase_timing.next_decision_at_ms,
                    failsafe_deadline_ms: state.phase_timing.failsafe_deadline_ms,
                })
                .collect();
            v.sort_by_key(|b| b.player.0);
            v
        })
        .unwrap_or_default();

    let (decision_log_total, decision_log_tail) = inputs
        .decision_log
        .map(|log| {
            let total = log.entries.len();
            let start = total.saturating_sub(DECISION_LOG_TAIL_CAP);
            let tail: Vec<_> = log.entries[start..]
                .iter()
                .map(DecisionEntrySnapshot::from)
                .collect();
            (total, tail)
        })
        .unwrap_or((0, Vec::new()));

    BotQaSnapshot {
        schema_version: BOT_QA_SNAPSHOT_SCHEMA_VERSION,
        trigger,
        timestamp_ms: inputs.timestamp_ms,
        sequence,
        round,
        session,
        auction,
        economies,
        hands,
        board,
        objectives,
        bots,
        decision_log_tail,
        decision_log_total,
    }
}

fn sorted_players<I: IntoIterator<Item = PlayerId>>(iter: I) -> Vec<PlayerId> {
    let mut v: Vec<_> = iter.into_iter().collect();
    v.sort_by_key(|p| p.0);
    v
}

fn bot_difficulty_label(state: &crate::feature::bot::state::BotState) -> &'static str {
    use crate::feature::bot::state::BotDifficulty;
    match state.difficulty {
        BotDifficulty::Mvp => "mvp",
    }
}

fn auction_phase_label(phase: AuctionPhase) -> &'static str {
    match phase {
        AuctionPhase::Idle => "idle",
        AuctionPhase::Selecting => "selecting",
        AuctionPhase::LiveBidding => "live_bidding",
        AuctionPhase::Resolving => "resolving",
    }
}

fn protocol_phase(phase: RoundPhase) -> WireRoundPhase {
    match phase {
        RoundPhase::Lobby => WireRoundPhase::Lobby,
        RoundPhase::DraftInitial => WireRoundPhase::DraftInitial,
        RoundPhase::DraftAuction => WireRoundPhase::DraftAuction,
        RoundPhase::DraftShop => WireRoundPhase::DraftShop,
        RoundPhase::Placement => WireRoundPhase::Placement,
        RoundPhase::Resolution => WireRoundPhase::Resolution,
        RoundPhase::GameOver => WireRoundPhase::GameOver,
    }
}

/// Derive the canonical filename for a snapshot. Pure helper exposed so tests
/// can assert the naming scheme without touching the filesystem.
pub fn snapshot_filename(snapshot: &BotQaSnapshot) -> String {
    let round = snapshot.round.as_ref().map(|r| r.round_number).unwrap_or(0);
    let phase = snapshot
        .round
        .as_ref()
        .map(|r| format!("{:?}", r.phase).to_lowercase())
        .unwrap_or_else(|| "no_round".to_string());
    let trigger = match snapshot.trigger {
        SnapshotTrigger::Initial => "init",
        SnapshotTrigger::PhaseTransition => "phase",
        SnapshotTrigger::Periodic => "tick",
        SnapshotTrigger::GracefulShutdown => "shutdown",
    };
    format!(
        "snapshot-{:04}-{phase}-{trigger}-{:013}-{:06}.json",
        round, snapshot.timestamp_ms, snapshot.sequence
    )
}

/// Write `snapshot` to `dir/<derived-filename>`. Returns the written path.
///
/// Creates `dir` (and any missing parents) on demand. All I/O errors are
/// surfaced; the caller logs them rather than propagating to bail the system.
pub fn write_snapshot_to_disk(
    snapshot: &BotQaSnapshot,
    dir: &Path,
) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let filename = snapshot_filename(snapshot);
    let path = dir.join(filename);
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, snapshot)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(path)
}

/// Append a single decision entry to `path` as a newline-terminated JSON
/// object. Opens or reuses `writer_slot`'s buffered handle; re-opens when the
/// configured path changes.
pub fn append_decision_entry(
    entry: &DecisionEntrySnapshot,
    path: &Path,
    writer_slot: &mut Option<BufWriter<File>>,
    writer_path: &mut Option<PathBuf>,
) -> std::io::Result<()> {
    let reopen = match writer_path {
        Some(existing) => existing.as_path() != path,
        None => true,
    };
    if reopen {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        *writer_slot = Some(BufWriter::new(file));
        *writer_path = Some(path.to_path_buf());
    }
    let writer = writer_slot.as_mut().expect("writer_slot just opened above");
    let json = serde_json::to_string(entry)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    writer.write_all(json.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn now_ms(time: &Time) -> u64 {
    (time.elapsed().as_secs_f64() * 1_000.0) as u64
}

fn timer_remaining_ms(timer: &Timer) -> u32 {
    u32::try_from(timer.remaining().as_millis()).unwrap_or(u32::MAX)
}

/// Periodic / phase-transition snapshot writer. Runs every `Update`; early
/// returns when the subsystem is disabled or no bot session is observable.
#[allow(clippy::too_many_arguments)]
pub fn bot_qa_snapshot_writer_system(
    time: Res<Time>,
    config: Res<BotQaSnapshotConfig>,
    mut state: ResMut<BotQaSnapshotState>,
    round_state: Option<Res<RoundState>>,
    session: Option<Res<SessionConfig>>,
    bots: Option<Res<BotPlayers>>,
    decision_log: Option<Res<BotDecisionLog>>,
    auction: Option<Res<AuctionState>>,
    economies: Option<Res<PlayerEconomies>>,
    hands: Option<Res<PlayerHands>>,
    board: Option<Res<BoardOccupancy>>,
    objective_query: Query<(&ObjectiveSlot, &ObjectiveHp)>,
) {
    if !config.enabled {
        return;
    }
    // Only snapshot when at least one bot is participating; without a bot
    // session this evidence channel has nothing to document and the file
    // churn would just confuse human QA.
    let bot_count = bots.as_deref().map(|b| b.bots.len()).unwrap_or(0);
    if bot_count == 0 {
        return;
    }

    let now = now_ms(&time);
    let current_phase = round_state.as_deref().map(|rs| protocol_phase(rs.phase));
    let phase_changed = match (state.last_phase, current_phase) {
        (Some(prev), Some(curr)) => prev != curr,
        (None, Some(_)) => false, // first sighting handled by `Initial` trigger
        _ => false,
    };
    let initial = state.last_phase.is_none() && current_phase.is_some();
    let periodic_due = now >= state.next_periodic_ms;

    if !(initial || phase_changed || periodic_due) {
        return;
    }

    let trigger = if initial {
        SnapshotTrigger::Initial
    } else if phase_changed {
        SnapshotTrigger::PhaseTransition
    } else {
        SnapshotTrigger::Periodic
    };

    let objectives = collect_objectives(&objective_query);
    state.sequence = state.sequence.wrapping_add(1);
    let inputs = SnapshotInputs {
        timestamp_ms: now,
        round_state: round_state.as_deref(),
        session: session.as_deref(),
        bots: bots.as_deref(),
        decision_log: decision_log.as_deref(),
        auction: auction.as_deref(),
        economies: economies.as_deref(),
        hands: hands.as_deref(),
        board: board.as_deref(),
        objectives,
    };
    let snapshot = assemble_snapshot(trigger, state.sequence, &inputs);

    match write_snapshot_to_disk(&snapshot, &config.snapshot_dir) {
        Ok(path) => {
            tracing::debug!(
                target: "server::bot::qa_snapshot",
                trigger = ?trigger,
                path = %path.display(),
                phase = ?current_phase,
                "bot QA snapshot written"
            );
        }
        Err(err) => {
            tracing::warn!(
                target: "server::bot::qa_snapshot",
                trigger = ?trigger,
                dir = %config.snapshot_dir.display(),
                error = %err,
                "bot QA snapshot write failed"
            );
        }
    }

    if current_phase.is_some() {
        state.last_phase = current_phase;
    }
    state.next_periodic_ms = now.saturating_add(config.periodic_interval_ms);
}

/// Decision-log streamer. Appends every new [`BotDecisionEntry`] since the
/// previous tick to the JSONL file and `flush()`-es each append for durability.
pub fn bot_decision_log_streamer_system(
    config: Res<BotQaSnapshotConfig>,
    mut state: ResMut<BotQaSnapshotState>,
    decision_log: Option<Res<BotDecisionLog>>,
) {
    if !config.enabled {
        return;
    }
    let Some(log) = decision_log else {
        return;
    };
    if log.entries.len() <= state.decision_log_offset {
        return;
    }

    let path = config.decision_log_path.clone();
    // Reborrow `state` once so the split-borrow on its disjoint fields
    // (`decision_log_writer` immutable + `decision_log_writer_path` mutable)
    // is visible to the borrow checker without ResMut's auto-deref hiding it.
    let state: &mut BotQaSnapshotState = &mut *state;

    let new_entries: Vec<DecisionEntrySnapshot> = log.entries[state.decision_log_offset..]
        .iter()
        .map(DecisionEntrySnapshot::from)
        .collect();

    let mut writer_guard = match state.decision_log_writer.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            tracing::warn!(
                target: "server::bot::qa_snapshot",
                "decision-log writer mutex poisoned; recovering"
            );
            poisoned.into_inner()
        }
    };

    let mut wrote = 0usize;
    for entry in &new_entries {
        match append_decision_entry(
            entry,
            &path,
            &mut *writer_guard,
            &mut state.decision_log_writer_path,
        ) {
            Ok(()) => wrote += 1,
            Err(err) => {
                tracing::warn!(
                    target: "server::bot::qa_snapshot",
                    path = %path.display(),
                    error = %err,
                    "bot decision-log append failed"
                );
                break;
            }
        }
    }
    drop(writer_guard);
    state.decision_log_offset = state.decision_log_offset.saturating_add(wrote);
}

/// Best-effort graceful-shutdown writer. Reads `AppExit` once per frame and,
/// the first time it sees one, emits a final snapshot with trigger
/// [`SnapshotTrigger::GracefulShutdown`].
#[allow(clippy::too_many_arguments)]
pub fn bot_qa_snapshot_shutdown_system(
    time: Res<Time>,
    config: Res<BotQaSnapshotConfig>,
    mut state: ResMut<BotQaSnapshotState>,
    mut exit_events: MessageReader<AppExit>,
    round_state: Option<Res<RoundState>>,
    session: Option<Res<SessionConfig>>,
    bots: Option<Res<BotPlayers>>,
    decision_log: Option<Res<BotDecisionLog>>,
    auction: Option<Res<AuctionState>>,
    economies: Option<Res<PlayerEconomies>>,
    hands: Option<Res<PlayerHands>>,
    board: Option<Res<BoardOccupancy>>,
    objective_query: Query<(&ObjectiveSlot, &ObjectiveHp)>,
) {
    if !config.enabled || state.shutdown_dump_done {
        // Drain so the reader cursor advances and we do not re-process on
        // future frames if shutdown was already handled.
        let _ = exit_events.read().count();
        return;
    }
    if exit_events.read().next().is_none() {
        return;
    }

    let now = now_ms(&time);
    let objectives = collect_objectives(&objective_query);
    state.sequence = state.sequence.wrapping_add(1);
    let inputs = SnapshotInputs {
        timestamp_ms: now,
        round_state: round_state.as_deref(),
        session: session.as_deref(),
        bots: bots.as_deref(),
        decision_log: decision_log.as_deref(),
        auction: auction.as_deref(),
        economies: economies.as_deref(),
        hands: hands.as_deref(),
        board: board.as_deref(),
        objectives,
    };
    let snapshot = assemble_snapshot(SnapshotTrigger::GracefulShutdown, state.sequence, &inputs);
    match write_snapshot_to_disk(&snapshot, &config.snapshot_dir) {
        Ok(path) => {
            tracing::info!(
                target: "server::bot::qa_snapshot",
                path = %path.display(),
                "bot QA snapshot (graceful shutdown) written"
            );
        }
        Err(err) => {
            tracing::warn!(
                target: "server::bot::qa_snapshot",
                dir = %config.snapshot_dir.display(),
                error = %err,
                "bot QA snapshot (graceful shutdown) write failed"
            );
        }
    }
    state.shutdown_dump_done = true;
}

fn collect_objectives(
    query: &Query<(&ObjectiveSlot, &ObjectiveHp)>,
) -> Vec<(ObjectiveSlot, ObjectiveHp)> {
    query.iter().map(|(slot, hp)| (*slot, hp.clone())).collect()
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin: registers config + state + the three writer systems.
///
/// Idempotent: if [`BotQaSnapshotConfig`] is already present (e.g. a test
/// inserted a deterministic config), `init_resource` is a no-op and the
/// pre-inserted config is honoured.
pub struct BotQaSnapshotPlugin;

impl Plugin for BotQaSnapshotPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<BotQaSnapshotConfig>() {
            app.insert_resource(BotQaSnapshotConfig::from_env());
        }
        app.init_resource::<BotQaSnapshotState>();
        app.add_systems(
            Update,
            (bot_qa_snapshot_writer_system, bot_decision_log_streamer_system),
        );
        app.add_systems(Last, bot_qa_snapshot_shutdown_system);
    }
}

// ---------------------------------------------------------------------------
// Unused-import suppression for fields only read by serde
// ---------------------------------------------------------------------------

const _: () = {
    let _ = BOARD_LANE_COUNT;
    let _ = BOARD_CELLS_PER_LANE;
};

#[cfg(test)]
mod tests {
    use super::*;
    use shared::protocol::CardSource;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Test-only temp directory holder. Manual cleanup avoids adding a
    /// `tempfile` dev-dependency just for these unit tests; the directory is
    /// nested under [`std::env::temp_dir`] and named with a monotonic counter
    /// + process id so concurrent test runs cannot collide.
    struct TempDir(PathBuf);

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    impl TempDir {
        fn new(label: &str) -> Self {
            let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let pid = std::process::id();
            let path = std::env::temp_dir().join(format!(
                "ccgs-bot-qa-snapshot-test-{label}-{pid}-{n}"
            ));
            std::fs::create_dir_all(&path).expect("create tempdir");
            Self(path)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn entry(kind: BotDecisionKind, ts: u64) -> BotDecisionEntry {
        BotDecisionEntry {
            round_number: 1,
            phase: WireRoundPhase::DraftInitial,
            bot_player_id: PlayerId(42),
            decision: kind,
            timestamp_ms: ts,
            legal_action_count: None,
            seed: 99,
            seed_word_counter: 3,
        }
    }

    #[test]
    fn from_env_values_respects_explicit_enable() {
        let cfg = BotQaSnapshotConfig::from_env_values(Some("1"), None, None, false);
        assert!(cfg.enabled);
        let cfg = BotQaSnapshotConfig::from_env_values(Some("0"), None, None, true);
        assert!(!cfg.enabled);
    }

    #[test]
    fn from_env_values_uses_dev_default_when_unset() {
        let cfg = BotQaSnapshotConfig::from_env_values(None, None, None, true);
        assert!(cfg.enabled);
        let cfg = BotQaSnapshotConfig::from_env_values(Some(""), None, None, true);
        assert!(cfg.enabled);
        let cfg = BotQaSnapshotConfig::from_env_values(Some("   "), None, None, true);
        assert!(cfg.enabled);
    }

    #[test]
    fn from_env_values_invalid_is_disabled() {
        let cfg = BotQaSnapshotConfig::from_env_values(Some("yes"), None, None, true);
        assert!(!cfg.enabled);
    }

    #[test]
    fn from_env_values_paths_default_when_blank() {
        let cfg = BotQaSnapshotConfig::from_env_values(Some("1"), Some("   "), Some(""), false);
        assert_eq!(cfg.snapshot_dir, PathBuf::from(DEFAULT_BOT_QA_SNAPSHOT_DIR));
        assert_eq!(
            cfg.decision_log_path,
            PathBuf::from(DEFAULT_BOT_DECISION_LOG_PATH)
        );
    }

    #[test]
    fn from_env_values_paths_override() {
        let cfg = BotQaSnapshotConfig::from_env_values(
            Some("1"),
            Some("/tmp/x"),
            Some("/tmp/y.jsonl"),
            false,
        );
        assert_eq!(cfg.snapshot_dir, PathBuf::from("/tmp/x"));
        assert_eq!(cfg.decision_log_path, PathBuf::from("/tmp/y.jsonl"));
    }

    #[test]
    fn snapshot_filename_includes_phase_round_trigger_ts_seq() {
        let snapshot = BotQaSnapshot {
            schema_version: BOT_QA_SNAPSHOT_SCHEMA_VERSION,
            trigger: SnapshotTrigger::PhaseTransition,
            timestamp_ms: 1234,
            sequence: 7,
            round: Some(RoundSnapshot {
                phase: WireRoundPhase::Placement,
                round_number: 3,
                draft_ready_players: vec![],
                submissions_received: vec![],
                disconnect_trackers: vec![],
                timers_ms: RoundTimersSnapshot::default(),
            }),
            session: None,
            auction: None,
            economies: vec![],
            hands: vec![],
            board: None,
            objectives: vec![],
            bots: vec![],
            decision_log_tail: vec![],
            decision_log_total: 0,
        };
        let name = snapshot_filename(&snapshot);
        assert!(name.starts_with("snapshot-0003-placement-phase-"));
        assert!(name.ends_with("-000007.json"));
        assert!(name.contains("0000000001234"));
    }

    #[test]
    fn decision_entry_snapshot_roundtrips_purchased_kind() {
        let e = entry(
            BotDecisionKind::Purchased {
                card_id: CardId(11),
                source: CardSource::ShopPurchase,
                gold_after: 8,
            },
            123,
        );
        let snap: DecisionEntrySnapshot = (&e).into();
        let json = serde_json::to_string(&snap).expect("serialize");
        assert!(json.contains("\"kind\":\"purchased\""));
        assert!(json.contains("\"card_id\":11"));
        assert!(json.contains("\"gold_after\":8"));
        assert!(json.contains("\"source\":\"ShopPurchase\""));
    }

    #[test]
    fn append_decision_entry_streams_each_call_with_flush_and_reuses_handle() {
        let dir = TempDir::new("stream-reuse");
        let path = dir.path().join("decision-log.jsonl");

        let mut writer_slot: Option<BufWriter<File>> = None;
        let mut writer_path: Option<PathBuf> = None;

        let snap1: DecisionEntrySnapshot =
            (&entry(BotDecisionKind::DraftReady, 100)).into();
        let snap2: DecisionEntrySnapshot =
            (&entry(BotDecisionKind::AuctionPass { reason: "x" }, 200)).into();

        append_decision_entry(&snap1, &path, &mut writer_slot, &mut writer_path)
            .expect("append 1");
        // After flush the bytes must be visible to other readers.
        let after_first = std::fs::read_to_string(&path).expect("read after first");
        assert!(after_first.contains("\"kind\":\"draft_ready\""));
        assert!(after_first.ends_with('\n'));

        // Handle reuse: writer_slot stays populated and writer_path stays the
        // same. The next append must NOT reopen (we cannot detect reopen
        // directly but we assert the resource invariants).
        assert!(writer_slot.is_some());
        assert_eq!(writer_path.as_deref(), Some(path.as_path()));

        append_decision_entry(&snap2, &path, &mut writer_slot, &mut writer_path)
            .expect("append 2");
        let after_second = std::fs::read_to_string(&path).expect("read after second");
        let mut lines = after_second.lines();
        let l1 = lines.next().expect("line 1");
        let l2 = lines.next().expect("line 2");
        assert!(lines.next().is_none());
        assert!(l1.contains("\"kind\":\"draft_ready\""));
        assert!(l2.contains("\"kind\":\"auction_pass\""));
        assert!(l2.contains("\"reason\":\"x\""));
    }

    #[test]
    fn append_decision_entry_reopens_when_path_changes() {
        let dir = TempDir::new("stream-reopen");
        let path_a = dir.path().join("a.jsonl");
        let path_b = dir.path().join("b.jsonl");

        let mut writer_slot: Option<BufWriter<File>> = None;
        let mut writer_path: Option<PathBuf> = None;

        let snap: DecisionEntrySnapshot =
            (&entry(BotDecisionKind::DraftReady, 1)).into();
        append_decision_entry(&snap, &path_a, &mut writer_slot, &mut writer_path)
            .expect("append a");
        append_decision_entry(&snap, &path_b, &mut writer_slot, &mut writer_path)
            .expect("append b");

        assert_eq!(writer_path.as_deref(), Some(path_b.as_path()));
        let a = std::fs::read_to_string(&path_a).unwrap();
        let b = std::fs::read_to_string(&path_b).unwrap();
        assert_eq!(a.lines().count(), 1);
        assert_eq!(b.lines().count(), 1);
    }

    #[test]
    fn write_snapshot_to_disk_creates_dir_and_pretty_json() {
        let dir = TempDir::new("write-snapshot");
        let out = dir.path().join("nested/sub");

        let snapshot = BotQaSnapshot {
            schema_version: BOT_QA_SNAPSHOT_SCHEMA_VERSION,
            trigger: SnapshotTrigger::Initial,
            timestamp_ms: 0,
            sequence: 0,
            round: None,
            session: None,
            auction: None,
            economies: vec![],
            hands: vec![],
            board: None,
            objectives: vec![],
            bots: vec![],
            decision_log_tail: vec![],
            decision_log_total: 0,
        };
        let path = write_snapshot_to_disk(&snapshot, &out).expect("write");
        assert!(path.exists());
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("\"schema_version\": 1"));
        assert!(body.contains("\"trigger\": \"initial\""));
    }

    #[test]
    fn assemble_snapshot_summarises_observed_resources() {
        let mut bots = BotPlayers::default();
        bots.insert(crate::feature::bot::state::BotState::new(PlayerId(42), 99));

        let mut log = BotDecisionLog::default();
        log.push(entry(BotDecisionKind::DraftReady, 1));
        log.push(entry(BotDecisionKind::EmptyPlacementFailsafe, 2));

        let mut econ_map = HashMap::new();
        econ_map.insert(
            PlayerId(42),
            crate::core::economy::PlayerEconomy {
                gold: 10,
                current_mana: 2,
                reserve_mana: 1,
                mana_cap: 5,
                reserved_gold: 0,
            },
        );
        let economies = PlayerEconomies(econ_map);

        let mut hands_map = HashMap::new();
        hands_map.insert(PlayerId(42), vec![CardId(1), CardId(2), CardId(3)]);
        let hands = PlayerHands { hands: hands_map };

        let inputs = SnapshotInputs {
            timestamp_ms: 555,
            round_state: None,
            session: None,
            bots: Some(&bots),
            decision_log: Some(&log),
            auction: None,
            economies: Some(&economies),
            hands: Some(&hands),
            board: None,
            objectives: vec![],
        };
        let snap = assemble_snapshot(SnapshotTrigger::Initial, 1, &inputs);
        assert_eq!(snap.bots.len(), 1);
        assert_eq!(snap.bots[0].player, PlayerId(42));
        assert_eq!(snap.economies.len(), 1);
        assert_eq!(snap.economies[0].gold, 10);
        assert_eq!(snap.hands.len(), 1);
        assert_eq!(snap.hands[0].size, 3);
        assert_eq!(snap.decision_log_total, 2);
        assert_eq!(snap.decision_log_tail.len(), 2);
    }

    #[test]
    fn assemble_snapshot_caps_decision_log_tail() {
        let mut log = BotDecisionLog::default();
        for i in 0..(DECISION_LOG_TAIL_CAP as u64 + 10) {
            log.push(entry(BotDecisionKind::DraftReady, i));
        }
        let inputs = SnapshotInputs {
            timestamp_ms: 1,
            round_state: None,
            session: None,
            bots: None,
            decision_log: Some(&log),
            auction: None,
            economies: None,
            hands: None,
            board: None,
            objectives: vec![],
        };
        let snap = assemble_snapshot(SnapshotTrigger::Periodic, 1, &inputs);
        assert_eq!(snap.decision_log_total, DECISION_LOG_TAIL_CAP + 10);
        assert_eq!(snap.decision_log_tail.len(), DECISION_LOG_TAIL_CAP);
        // Tail must contain the *last* entries (timestamps 10..74).
        assert_eq!(snap.decision_log_tail.first().unwrap().timestamp_ms, 10);
        assert_eq!(
            snap.decision_log_tail.last().unwrap().timestamp_ms,
            DECISION_LOG_TAIL_CAP as u64 + 9
        );
    }
}
