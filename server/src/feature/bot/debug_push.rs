//! Debug-only god-mode bot state push (PROMPT 1614).
//!
//! Streams an [`S2CDebugBotStatePush`] message to every human peer at a
//! rate-limited cadence so the client-side F8 overlay
//! (`client::presentation::debug_bot_overlay`) can display:
//!
//! - Per-bot class / gold / mana / submitted state (already in
//!   `S2CGameSnapshot` but redacted/incomplete for the bot's perspective).
//! - The bot's **hand** (god-mode, never replicated in production).
//! - The tail of the bot's `BotDecisionLog` plus the most recent auction
//!   valuation.
//!
//! ## Activation contract
//!
//! Mirrors the dual gating pattern used by [`BotQaSnapshotPlugin`]:
//! `CCGS_BOT_DEBUG_UI=1` forces enabled, `0` forces disabled, unset defaults
//! to `cfg!(debug_assertions)`. Tests can pre-insert a deterministic
//! [`BotDebugPushConfig`] resource before adding the plugin.
//!
//! ## Non-product rule
//!
//! - The plugin is always registered. Every system early-returns when the
//!   config is disabled or no bot is in the session — production servers
//!   pay zero observable cost.
//! - **No semantic gameplay mutation**. The push reads observable resources
//!   via `Option<Res<…>>` and emits one debug-only S2C message; no other
//!   server state changes.
//! - Reliable channel: keeps order with `S2CGameSnapshot` so the overlay
//!   never claims a phase the snapshot has not delivered to the client yet.
//! - **No edits to existing bot files**. Reads
//!   [`BotPlayers`](crate::feature::bot::state::BotPlayers),
//!   [`BotDecisionLog`](crate::feature::bot::state::BotDecisionLog),
//!   [`PlayerHands`](crate::feature::acquisition::PlayerHands),
//!   [`PlayerEconomies`](crate::core::economy::PlayerEconomies), and
//!   [`SessionConfig`](crate::core::session::config::SessionConfig) only.
//!
//! Data-contract source: `reports/PROMPT-1604-bot-flow-debug-overlay-data-contract.md`.

#![allow(dead_code)]

use std::collections::HashSet;

use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, Server, ServerMultiMessageSender};
use shared::protocol::{
    DebugBotDecisionEntry, DebugBotStateEntry, ReliableChannel, S2CDebugBotStatePush,
};
use shared::session::PlayerId;

use crate::core::economy::PlayerEconomies;
use crate::core::session::config::SessionConfig;
use crate::core::session::state::PlayerConnectionMap;
use crate::feature::acquisition::PlayerHands;
use crate::feature::bot::state::{BotDecisionKind, BotDecisionLog, BotPlayers};

/// Env var: enables/disables the server-side debug bot-state push.
pub const BOT_DEBUG_PUSH_ENV_VAR: &str = "CCGS_BOT_DEBUG_UI";

/// Default cadence between two consecutive debug pushes, in milliseconds.
/// 500 ms keeps the overlay responsive to live phase changes without
/// dominating the reliable channel during steady-state play.
pub const DEFAULT_BOT_DEBUG_PUSH_INTERVAL_MS: u64 = 500;

/// Hard cap on the per-bot decision-tail attached to a push. Matches the
/// human-readable cap recommended by PROMPT 1604 §4.1 (16 entries).
pub const DEBUG_BOT_DECISION_TAIL_CAP: usize = 16;

// ---------------------------------------------------------------------------
// Config + state
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct BotDebugPushConfig {
    pub enabled: bool,
    pub interval_ms: u64,
    pub tail_cap: usize,
}

impl Default for BotDebugPushConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_ms: DEFAULT_BOT_DEBUG_PUSH_INTERVAL_MS,
            tail_cap: DEBUG_BOT_DECISION_TAIL_CAP,
        }
    }
}

impl BotDebugPushConfig {
    /// Build a config from environment variables. Mirrors the
    /// [`BotQaSnapshotConfig::from_env_values`](crate::feature::bot::qa_snapshot::BotQaSnapshotConfig)
    /// parsing rules byte-for-byte so operators only learn one convention.
    pub fn from_env() -> Self {
        Self::from_env_values(
            std::env::var(BOT_DEBUG_PUSH_ENV_VAR).ok().as_deref(),
            cfg!(debug_assertions),
        )
    }

    pub fn from_env_values(enable_var: Option<&str>, dev_default_enabled: bool) -> Self {
        let enabled = match enable_var.map(str::trim) {
            None | Some("") => dev_default_enabled,
            Some("1") => true,
            Some("0") => false,
            Some(other) => {
                tracing::warn!(
                    target: "server::bot::debug_push",
                    value = %other,
                    "{} has invalid value; treating as disabled (expected 1, 0, or unset)",
                    BOT_DEBUG_PUSH_ENV_VAR,
                );
                false
            }
        };
        Self {
            enabled,
            interval_ms: DEFAULT_BOT_DEBUG_PUSH_INTERVAL_MS,
            tail_cap: DEBUG_BOT_DECISION_TAIL_CAP,
        }
    }
}

/// Mutable runtime state for the debug push system. Single resource so the
/// system signature stays small.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BotDebugPushState {
    /// Server wallclock ms at which the next push is allowed to fire.
    /// `0` means "fire on the next tick".
    pub next_push_ms: u64,
    /// Monotonic per-process counter so the assembly helper can label
    /// pushes for debug logging.
    pub sequence: u64,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct BotDebugPushPlugin;

impl Plugin for BotDebugPushPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<BotDebugPushConfig>() {
            app.insert_resource(BotDebugPushConfig::from_env());
        }
        app.init_resource::<BotDebugPushState>();
        app.add_systems(Update, bot_debug_push_system);
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (assembly)
// ---------------------------------------------------------------------------

/// Label one [`BotDecisionKind`] for the wire-side `kind_label` field. Lowercase
/// snake_case so it is grep-stable across releases. Mirrors
/// `DecisionKindSnapshot` tags from the QA-snapshot module.
pub fn decision_kind_label(kind: &BotDecisionKind) -> &'static str {
    match kind {
        BotDecisionKind::ClassChosen { .. } => "class_chosen",
        BotDecisionKind::ClassConfirmed => "class_confirmed",
        BotDecisionKind::Purchased { .. } => "purchased",
        BotDecisionKind::Refreshed { .. } => "refreshed",
        BotDecisionKind::PurchaseSkipped { .. } => "purchase_skipped",
        BotDecisionKind::DraftReady => "draft_ready",
        BotDecisionKind::AuctionBid { .. } => "auction_bid",
        BotDecisionKind::AuctionPass { .. } => "auction_pass",
        BotDecisionKind::PlacementSubmitted { .. } => "placement_submitted",
        BotDecisionKind::PlacementSkipped { .. } => "placement_skipped",
        BotDecisionKind::EmptyPlacementFailsafe => "empty_placement_failsafe",
        BotDecisionKind::ResultAcknowledged => "result_acknowledged",
    }
}

/// Render the variant-specific payload of a [`BotDecisionKind`] as a short
/// human-readable summary. `None` for variants without per-variant fields.
pub fn decision_detail(kind: &BotDecisionKind) -> Option<String> {
    match kind {
        BotDecisionKind::ClassChosen { class_id } => Some(format!("class={:?}", class_id)),
        BotDecisionKind::ClassConfirmed
        | BotDecisionKind::DraftReady
        | BotDecisionKind::EmptyPlacementFailsafe
        | BotDecisionKind::ResultAcknowledged => None,
        BotDecisionKind::Purchased {
            card_id,
            source,
            gold_after,
        } => Some(format!(
            "card={} source={:?} gold_after={gold_after}",
            card_id.0, source
        )),
        BotDecisionKind::Refreshed { gold_after } => Some(format!("gold_after={gold_after}")),
        BotDecisionKind::PurchaseSkipped { reason } => Some(format!("reason={reason}")),
        BotDecisionKind::AuctionBid {
            card_id,
            amount,
            valuation,
        } => Some(format!(
            "card={} amt={amount} val={valuation}",
            card_id.0
        )),
        BotDecisionKind::AuctionPass { reason } => Some(format!("reason={reason}")),
        BotDecisionKind::PlacementSubmitted { placements_len } => {
            Some(format!("placements_len={placements_len}"))
        }
        BotDecisionKind::PlacementSkipped { reason } => Some(format!("reason={reason}")),
    }
}

/// Pure function: build the wire payload from observable references. Exposed
/// so tests can assert deterministic shape without spinning up a Bevy app.
pub fn assemble_debug_bot_state_push(
    bots: &BotPlayers,
    decision_log: &BotDecisionLog,
    hands: Option<&PlayerHands>,
    economies: Option<&PlayerEconomies>,
    session: Option<&SessionConfig>,
    submitted: &HashSet<PlayerId>,
    assembled_at_ms: u64,
    tail_cap: usize,
) -> S2CDebugBotStatePush {
    let mut ordered_bot_ids: Vec<PlayerId> = bots.bots.keys().copied().collect();
    ordered_bot_ids.sort_by_key(|p| p.0);

    let bots_payload = ordered_bot_ids
        .into_iter()
        .map(|player_id| {
            let bot = bots
                .get(player_id)
                .expect("bot id sourced from BotPlayers iteration");

            // Tail: filter the decision log to entries the bot produced.
            let mut bot_entries: Vec<DebugBotDecisionEntry> = decision_log
                .entries
                .iter()
                .filter(|entry| entry.bot_player_id == player_id)
                .map(|entry| DebugBotDecisionEntry {
                    round_number: entry.round_number,
                    phase: entry.phase,
                    timestamp_ms: entry.timestamp_ms,
                    kind_label: decision_kind_label(&entry.decision).to_string(),
                    detail: decision_detail(&entry.decision),
                })
                .collect();
            let bot_entries_len = bot_entries.len();
            let start = bot_entries_len.saturating_sub(tail_cap);
            let decision_tail: Vec<DebugBotDecisionEntry> = bot_entries.drain(start..).collect();

            // Find the most recent auction bid for this bot (irrespective of
            // tail-cap) so the overlay can always surface the latest
            // valuation even when the bot has scrolled past it.
            let last_bid_valuation = decision_log
                .entries
                .iter()
                .rev()
                .find_map(|entry| match (entry.bot_player_id, &entry.decision) {
                    (id, BotDecisionKind::AuctionBid { valuation, .. }) if id == player_id => {
                        Some(*valuation)
                    }
                    _ => None,
                });

            let hand = hands
                .and_then(|h| h.hands.get(&player_id))
                .cloned()
                .unwrap_or_default();

            let economy = economies.and_then(|e| e.0.get(&player_id));
            let gold = economy.map(|e| e.gold).unwrap_or(0);
            let current_mana = economy.map(|e| e.current_mana).unwrap_or(0);
            // mana_cap on the wire is u8; clamp to be defensive against any
            // future cap change in `PlayerEconomy` without breaking the wire
            // size budget.
            let mana_cap = economy
                .map(|e| u8::try_from(e.mana_cap).unwrap_or(u8::MAX))
                .unwrap_or(0);

            let class_id = bot
                .class_choice
                .or_else(|| session.and_then(|cfg| cfg.class_map.get(&player_id).copied()));

            DebugBotStateEntry {
                player_id,
                class_id,
                gold,
                current_mana,
                mana_cap,
                submitted: submitted.contains(&player_id),
                hand,
                decision_tail,
                last_bid_valuation,
            }
        })
        .collect();

    S2CDebugBotStatePush {
        bots: bots_payload,
        decision_log_total: u32::try_from(decision_log.entries.len()).unwrap_or(u32::MAX),
        assembled_at_ms,
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

fn now_ms(time: &Time) -> u64 {
    (time.elapsed().as_secs_f64() * 1_000.0) as u64
}

/// Server `Update` system: rate-limited assemble + broadcast of
/// [`S2CDebugBotStatePush`] to every human peer in the session.
///
/// Cost-gating order:
/// 1. `config.enabled` (env-gated).
/// 2. At least one bot in `BotPlayers`.
/// 3. `now >= state.next_push_ms` (interval-gated).
///
/// If any of the resources required to produce a meaningful payload are
/// missing (e.g., during loading) the system early-returns without bumping
/// the timer so the first valid frame still produces a push.
#[allow(clippy::too_many_arguments)]
pub fn bot_debug_push_system(
    time: Res<Time>,
    config: Res<BotDebugPushConfig>,
    mut state: ResMut<BotDebugPushState>,
    bots: Option<Res<BotPlayers>>,
    decision_log: Option<Res<BotDecisionLog>>,
    hands: Option<Res<PlayerHands>>,
    economies: Option<Res<PlayerEconomies>>,
    session: Option<Res<SessionConfig>>,
    connections: Option<Res<PlayerConnectionMap>>,
    round_state: Option<Res<crate::core::rsm::state::RoundState>>,
    mut sender: ServerMultiMessageSender,
    server_query: Query<&Server>,
) {
    if !config.enabled {
        return;
    }
    let Some(bots) = bots.as_deref() else {
        return;
    };
    if bots.is_empty() {
        return;
    }
    let now = now_ms(&time);
    if now < state.next_push_ms {
        return;
    }
    let Some(decision_log) = decision_log.as_deref() else {
        return;
    };

    // Recipients = every human-mapped peer (i.e. all players in
    // `PlayerConnectionMap` whose player_id is NOT in `BotPlayers`).
    let recipient_peers = match connections.as_deref() {
        Some(map) => map
            .0
            .iter()
            .filter_map(|(peer_id, player_id)| {
                (!bots.contains(*player_id)).then_some(*peer_id)
            })
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };

    // Assemble even when there are no recipients so that integration tests
    // that drive the assembly directly can inspect the payload. The wire
    // send is skipped below if the peer list is empty.
    let submitted: HashSet<PlayerId> = round_state
        .as_deref()
        .map(|rs| rs.submissions_received.iter().copied().collect())
        .unwrap_or_default();

    let payload = assemble_debug_bot_state_push(
        bots,
        decision_log,
        hands.as_deref(),
        economies.as_deref(),
        session.as_deref(),
        &submitted,
        now,
        config.tail_cap,
    );

    state.sequence = state.sequence.wrapping_add(1);
    state.next_push_ms = now.saturating_add(config.interval_ms);

    if recipient_peers.is_empty() {
        tracing::trace!(
            target: "server::bot::debug_push",
            bots = payload.bots.len(),
            decision_log_total = payload.decision_log_total,
            "debug bot-state push assembled (no recipients yet)"
        );
        return;
    }

    let Some(server) = server_query.iter().next() else {
        tracing::trace!(
            target: "server::bot::debug_push",
            "debug bot-state push DROPPED — Server entity not available yet"
        );
        return;
    };

    if let Err(e) = sender.send::<S2CDebugBotStatePush, ReliableChannel>(
        &payload,
        server,
        &NetworkTarget::Only(recipient_peers.clone()),
    ) {
        tracing::warn!(
            target: "server::bot::debug_push",
            err = ?e,
            "S2C send failed: type=S2CDebugBotStatePush, handler=bot_debug_push_system"
        );
        return;
    }

    tracing::debug!(
        target: "server::bot::debug_push",
        bots = payload.bots.len(),
        decision_log_total = payload.decision_log_total,
        recipients = recipient_peers.len(),
        sequence = state.sequence,
        "debug bot-state push sent"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::card::{CardId, ClassId};
    use shared::protocol::RoundPhase as WireRoundPhase;

    use crate::core::economy::state::PlayerEconomy;
    use crate::feature::bot::state::{BotDecisionEntry, BotDecisionKind, BotState};

    fn make_decision(
        player: PlayerId,
        kind: BotDecisionKind,
        ts: u64,
    ) -> BotDecisionEntry {
        BotDecisionEntry {
            round_number: 1,
            phase: WireRoundPhase::DraftAuction,
            bot_player_id: player,
            decision: kind,
            timestamp_ms: ts,
            legal_action_count: None,
            seed: 1,
            seed_word_counter: 1,
        }
    }

    #[test]
    fn from_env_values_respects_explicit_enable() {
        assert!(BotDebugPushConfig::from_env_values(Some("1"), false).enabled);
        assert!(!BotDebugPushConfig::from_env_values(Some("0"), true).enabled);
    }

    #[test]
    fn from_env_values_uses_dev_default_when_unset() {
        assert!(BotDebugPushConfig::from_env_values(None, true).enabled);
        assert!(BotDebugPushConfig::from_env_values(Some(""), true).enabled);
        assert!(BotDebugPushConfig::from_env_values(Some("   "), true).enabled);
        assert!(!BotDebugPushConfig::from_env_values(None, false).enabled);
    }

    #[test]
    fn from_env_values_invalid_is_disabled() {
        assert!(!BotDebugPushConfig::from_env_values(Some("yes"), true).enabled);
        assert!(!BotDebugPushConfig::from_env_values(Some("on"), true).enabled);
    }

    #[test]
    fn decision_kind_label_is_snake_case() {
        let cases: &[(BotDecisionKind, &str)] = &[
            (BotDecisionKind::ClassConfirmed, "class_confirmed"),
            (BotDecisionKind::DraftReady, "draft_ready"),
            (BotDecisionKind::EmptyPlacementFailsafe, "empty_placement_failsafe"),
            (BotDecisionKind::ResultAcknowledged, "result_acknowledged"),
        ];
        for (kind, expected) in cases {
            assert_eq!(decision_kind_label(kind), *expected);
        }
    }

    #[test]
    fn decision_detail_formats_auction_bid() {
        let detail = decision_detail(&BotDecisionKind::AuctionBid {
            card_id: CardId(42),
            amount: 4,
            valuation: 5,
        });
        assert_eq!(detail, Some("card=42 amt=4 val=5".to_string()));
    }

    #[test]
    fn assemble_orders_bots_by_player_id_and_caps_tail() {
        let bot_a = PlayerId(2);
        let bot_b = PlayerId(1);
        let mut bots = BotPlayers::default();
        bots.insert(BotState::new(bot_a, 0));
        bots.insert(BotState::new(bot_b, 1));

        let mut log = BotDecisionLog::default();
        // Push 20 placement decisions for bot_a so cap (default 16) trims.
        for i in 0..20u32 {
            log.push(make_decision(
                bot_a,
                BotDecisionKind::PlacementSubmitted { placements_len: i as u8 },
                u64::from(i),
            ));
        }
        // Push 2 for bot_b.
        log.push(make_decision(bot_b, BotDecisionKind::DraftReady, 100));
        log.push(make_decision(
            bot_b,
            BotDecisionKind::AuctionBid {
                card_id: CardId(7),
                amount: 3,
                valuation: 4,
            },
            101,
        ));

        let submitted: HashSet<PlayerId> = HashSet::from([bot_a]);
        let push = assemble_debug_bot_state_push(
            &bots,
            &log,
            None,
            None,
            None,
            &submitted,
            999,
            DEBUG_BOT_DECISION_TAIL_CAP,
        );

        assert_eq!(push.bots.len(), 2);
        assert_eq!(push.bots[0].player_id, bot_b);
        assert_eq!(push.bots[1].player_id, bot_a);
        assert!(push.bots[1].submitted);
        assert!(!push.bots[0].submitted);
        assert_eq!(push.bots[1].decision_tail.len(), DEBUG_BOT_DECISION_TAIL_CAP);
        assert_eq!(push.decision_log_total, 22);
        assert_eq!(push.assembled_at_ms, 999);

        // Last bid valuation reflects the most recent AuctionBid for bot_b.
        assert_eq!(push.bots[0].last_bid_valuation, Some(4));
        // bot_a never bid, so None.
        assert_eq!(push.bots[1].last_bid_valuation, None);
    }

    #[test]
    fn assemble_pulls_class_from_bot_then_falls_back_to_session_class_map() {
        let bot_id = PlayerId(9);
        let mut bots = BotPlayers::default();
        bots.insert(BotState::new(bot_id, 0));
        let log = BotDecisionLog::default();
        let submitted = HashSet::new();

        // Path 1: no class_choice yet, session class map carries it.
        let mut class_map = std::collections::HashMap::new();
        class_map.insert(bot_id, ClassId::Cra);
        let session = SessionConfig {
            mode: shared::protocol::GameMode::OneVOne,
            player_count: 1,
            team_map: std::collections::HashMap::new(),
            class_map,
            placement_timer_multiplier_effective:
                shared::protocol::PlacementTimerMultiplier::X1,
        };
        let push = assemble_debug_bot_state_push(
            &bots, &log, None, None, Some(&session), &submitted, 0, DEBUG_BOT_DECISION_TAIL_CAP,
        );
        assert_eq!(push.bots[0].class_id, Some(ClassId::Cra));

        // Path 2: bot's own class_choice wins over session.
        bots.get_mut(bot_id).unwrap().class_choice = Some(ClassId::Sacrier);
        let push = assemble_debug_bot_state_push(
            &bots, &log, None, None, Some(&session), &submitted, 0, DEBUG_BOT_DECISION_TAIL_CAP,
        );
        assert_eq!(push.bots[0].class_id, Some(ClassId::Sacrier));
    }

    #[test]
    fn assemble_includes_hand_and_economy_when_available() {
        let bot_id = PlayerId(3);
        let mut bots = BotPlayers::default();
        bots.insert(BotState::new(bot_id, 0));
        let log = BotDecisionLog::default();
        let submitted = HashSet::new();

        let mut hands = PlayerHands::default();
        hands
            .hands
            .insert(bot_id, vec![CardId(11), CardId(22), CardId(33)]);

        let mut economies = PlayerEconomies::default();
        economies.0.insert(
            bot_id,
            PlayerEconomy {
                gold: 12,
                current_mana: 4,
                reserve_mana: 1,
                mana_cap: 7,
                reserved_gold: 0,
            },
        );

        let push = assemble_debug_bot_state_push(
            &bots,
            &log,
            Some(&hands),
            Some(&economies),
            None,
            &submitted,
            0,
            DEBUG_BOT_DECISION_TAIL_CAP,
        );
        assert_eq!(push.bots[0].hand, vec![CardId(11), CardId(22), CardId(33)]);
        assert_eq!(push.bots[0].gold, 12);
        assert_eq!(push.bots[0].current_mana, 4);
        assert_eq!(push.bots[0].mana_cap, 7);
    }

    #[test]
    fn assemble_handles_missing_resources_gracefully() {
        let bot_id = PlayerId(5);
        let mut bots = BotPlayers::default();
        bots.insert(BotState::new(bot_id, 0));
        let log = BotDecisionLog::default();
        let submitted = HashSet::new();
        let push = assemble_debug_bot_state_push(
            &bots,
            &log,
            None,
            None,
            None,
            &submitted,
            42,
            DEBUG_BOT_DECISION_TAIL_CAP,
        );
        assert_eq!(push.bots.len(), 1);
        assert!(push.bots[0].hand.is_empty());
        assert_eq!(push.bots[0].gold, 0);
        assert_eq!(push.bots[0].mana_cap, 0);
        assert_eq!(push.bots[0].class_id, None);
        assert_eq!(push.assembled_at_ms, 42);
    }
}
