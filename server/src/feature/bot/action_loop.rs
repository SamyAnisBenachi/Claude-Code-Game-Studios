//! Bot participant action loop, Wave 1 (PROMPT 1531) + Wave 2 auction bid
//! (PROMPT 1582) + Wave 2.5 auction bid funnel (PROMPT 1598).
//!
//! After [`bot_lobby_auto_confirm`](super::lobby_loop::bot_lobby_auto_confirm)
//! lifts a room into `GameActive`, this module advances the bot through the
//! per-round phases that would otherwise stall without a human counterpart:
//!
//! - **Draft (initial & shop):** emit a deterministic `DraftReadySignal` so the
//!   RSM's `DRAFT_* -> Placement|Auction` gate fires without waiting on a
//!   non-existent human ready click.
//! - **Auction (Wave 2 + Wave 2.5):** deterministic bid decision behaviour.
//!   The bot reads `AuctionState` / `PlayerEconomies` / `PlayerHands` (all
//!   read-only) and records one
//!   `BotDecisionKind::AuctionBid { card_id, amount, valuation }` per
//!   `(player, round, current_price)` tuple when the heuristic decides to bid,
//!   or `BotDecisionKind::AuctionPass { reason }` with a precise reason
//!   literal otherwise. **Wave 2.5 (PROMPT 1598)** funnels the chosen bid
//!   amount into the server-internal `PendingBotBids` queue so
//!   `auction_tick_system` drains it alongside network `C2SPlaceBid` and the
//!   authoritative `process_bid_batch` validation applies uniformly — same
//!   price floor, leader gate, gold reservation, hand-full gate, expiry gate.
//!   No protocol message is added; bots have no `PeerId` and never produce a
//!   `C2SPlaceBid`. When `AuctionState` / `PendingBotBids` is absent (test
//!   scaffolding that bypasses the auction plugin) the Wave-1 "pass once per
//!   round" fallback still applies so legacy tests continue to compile.
//! - **Placement:** submit an empty (no-op) placement as a fail-safe so the
//!   `Placement -> Resolution` gate fires without waiting on the bot to place
//!   units. Heuristic placement (legal-cell pick) is explicitly deferred to a
//!   later wave.
//!
//! Scope discipline (PROMPT 1531 owned-scope, preserved by PROMPT 1582):
//! - **No new protocol messages.** We write the *internal* server-side
//!   `DraftReadySignal` and `PlacementSubmissionReceived` messages that the
//!   network layer already produces for human clients. The contract is reused,
//!   not extended. PROMPT 1582 does **not** emit any auction wire message.
//! - **No shop purchases.** Wave 1 just needs to advance flow; shop buys
//!   require gold/hand bookkeeping the bot does not yet own. Bots fall through
//!   `DraftShop` via the ready signal alone.
//! - **No client UI.** Server-only behavior.
//!
//! Determinism: every decision uses only inputs (round number, current phase,
//! bot membership, observable auction state) plus the bot's private
//! `ChaCha8Rng` seeded at session start. Two replays of the same session with
//! the same observable inputs produce the same `BotDecisionLog`. PROMPT 1582
//! draws RNG once per `(player, round)` to derive a stable per-round
//! valuation; the word counter is bumped in lockstep and snapshotted into the
//! decision log per ADR-005 audit convention.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use lightyear::prelude::PeerId;
use rand::RngCore;

use crate::core::economy::PlayerEconomies;
use crate::core::rsm::{
    state::{RoundPhase, RoundState},
    DraftReadySignal,
};
use crate::core::session::config::SessionConfig;
use crate::feature::acquisition::{PlayerHands, MAX_HAND_SIZE};
use crate::feature::auction::{AuctionBid, AuctionPhase, AuctionState, PendingBotBids};
use crate::feature::board::PlacementSubmissionReceived;
use crate::feature::bot::state::{
    BotDecisionEntry, BotDecisionKind, BotDecisionLog, BotPlayers,
    BOT_AUCTION_PASS_THRESHOLD_MS, BOT_AUCTION_VALUATION_NOISE_DENOMINATOR,
};
use shared::card::CardId;
use shared::session::PlayerId;

/// Convert the server-internal `RoundPhase` into the wire `shared::protocol`
/// counterpart. Local copy of the `core::rsm::transitions::protocol_round_phase`
/// private helper — kept here to avoid widening that module's surface for a
/// single audit-log writer.
fn protocol_phase(phase: RoundPhase) -> shared::protocol::RoundPhase {
    match phase {
        RoundPhase::Lobby => shared::protocol::RoundPhase::Lobby,
        RoundPhase::DraftInitial => shared::protocol::RoundPhase::DraftInitial,
        RoundPhase::DraftAuction => shared::protocol::RoundPhase::DraftAuction,
        RoundPhase::DraftShop => shared::protocol::RoundPhase::DraftShop,
        RoundPhase::Placement => shared::protocol::RoundPhase::Placement,
        RoundPhase::Resolution => shared::protocol::RoundPhase::Resolution,
        RoundPhase::GameOver => shared::protocol::RoundPhase::GameOver,
    }
}

/// Returns the set of bot `PlayerId`s currently participating in the active
/// session. Filters out lobby-only bots that have not yet been folded into a
/// `SessionConfig`, and humans who happen to share a synthetic id range.
fn session_bot_players(bots: &BotPlayers, session: &SessionConfig) -> Vec<PlayerId> {
    session
        .players()
        .filter(|player| bots.contains(*player))
        .collect()
}

/// Snapshot the bot's RNG counters for the decision log. Wave-1 branches never
/// consume RNG, so the counter stays at whatever `BotState::new` initialised it
/// to. Wave 2's auction-bid branch bumps the counter once per `(player, round)`
/// when the bot first observes the auction card; subsequent snapshots in the
/// same round therefore report the post-draw counter.
fn seed_snapshot(bots: &BotPlayers, player_id: PlayerId) -> (u64, u64) {
    bots.get(player_id)
        .map(|state| (state.rng_seed, state.rng_word_counter))
        .unwrap_or((0, 0))
}

/// Outcome of the Wave-2 auction bid heuristic for a single bot tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuctionDecision {
    /// Bot would submit a bid at `amount`. Wave 2 records the decision in the
    /// audit log only; the message is not pushed onto the auction wire (the
    /// integration wave that owns auction-side ingestion is deferred per
    /// PROMPT 1582 owned-scope).
    Bid { amount: u32, valuation: u32 },
    /// Bot declines to bid. The static reason literal feeds
    /// `BotDecisionKind::AuctionPass { reason }` directly.
    Pass { reason: &'static str },
}

/// Cached per-round auction context for a single bot. Populated lazily the
/// first tick the bot observes a `LiveBidding` auction in a given round, then
/// reused for every subsequent tick of the same round so the valuation does
/// not drift and the RNG is only advanced once per round.
///
/// `pub` so the type can appear in the public `bot_action_loop` system
/// signature via `Local<HashMap<_, AuctionRoundContext>>` without tripping
/// the `private_interfaces` lint. The type is otherwise an implementation
/// detail of the auction branch and is not re-exported from the module.
#[derive(Debug, Clone, Copy)]
pub struct AuctionRoundContext {
    /// The card the bot is reasoning about. Stored so a mid-round card swap
    /// (which the current auction protocol does not allow but defensive code
    /// must still handle) invalidates the cache instead of mispricing.
    card_id: CardId,
    /// Bot's reservation price for `card_id`. Maximum the bot will ever bid
    /// this round, independent of `current_price`.
    valuation: u32,
    /// The highest `current_price` snapshot the bot has already produced a
    /// decision against. Lets the bot stay quiet across ticks at the same
    /// price (idempotency) yet re-evaluate when a human raises the price.
    last_decision_price: u32,
}

/// Compute the bot's per-round valuation for `starting_price`.
///
/// Deterministic given the bot's RNG and word counter. Draws one `u64` from
/// the bot's private `ChaCha8Rng` and clamps the noise to half the starting
/// price (see [`BOT_AUCTION_VALUATION_NOISE_DENOMINATOR`]). The valuation
/// floor is `starting_price + 1` so the bot's reservation is always above
/// the rarity-derived minimum legal bid; the noise adds 0..=noise_max
/// headroom on top. Bumps the bot's `rng_word_counter` so the audit log can
/// track entropy consumption per ADR-005.
fn draw_valuation(bot: &mut crate::feature::bot::state::BotState, starting_price: u32) -> u32 {
    let noise_modulus = starting_price
        .saturating_div(BOT_AUCTION_VALUATION_NOISE_DENOMINATOR)
        .saturating_add(1);
    let word = bot.rng.next_u64();
    bot.rng_word_counter = bot.rng_word_counter.saturating_add(1);
    let noise = u32::try_from(word % u64::from(noise_modulus)).unwrap_or(u32::MAX);
    starting_price.saturating_add(1).saturating_add(noise)
}

/// Decide the bot's auction action against the observable `LiveBidding` state.
///
/// Pure decision logic with no side effects beyond the optional RNG draw the
/// caller already performed for `context.valuation`. The function only reads
/// the auction snapshot, the bot's economy, and hand size — it never mutates
/// any of these. All `Pass` reasons are static literals so the decision log
/// can store them without allocation.
fn decide_auction_action(
    bot_id: PlayerId,
    auction: &AuctionState,
    context: &AuctionRoundContext,
    economy_gold_minus_reserved: u32,
    hand_full: bool,
) -> AuctionDecision {
    if auction.phase != AuctionPhase::LiveBidding {
        return AuctionDecision::Pass { reason: "phase_not_live_bidding" };
    }
    if auction.card_id != Some(context.card_id) {
        return AuctionDecision::Pass { reason: "card_changed_mid_round" };
    }
    if auction.current_leader == Some(bot_id) {
        return AuctionDecision::Pass { reason: "already_leader" };
    }
    if hand_full {
        return AuctionDecision::Pass { reason: "hand_full" };
    }
    if auction.timer_remaining_ms < BOT_AUCTION_PASS_THRESHOLD_MS {
        return AuctionDecision::Pass { reason: "timer_below_threshold" };
    }
    let min_legal_bid = auction.current_price.saturating_add(1);
    if context.valuation < min_legal_bid {
        return AuctionDecision::Pass { reason: "valuation_below_min_bid" };
    }
    if economy_gold_minus_reserved < min_legal_bid {
        return AuctionDecision::Pass { reason: "insufficient_gold" };
    }
    let amount = context.valuation.min(economy_gold_minus_reserved).max(min_legal_bid);
    AuctionDecision::Bid { amount, valuation: context.valuation }
}

/// Returns the bot's unreserved gold (gold minus reserved) for the auction
/// pre-flight check, or `None` when the bot is missing from the economies
/// map (which is a server-side bug we surface to the caller as a pass).
fn unreserved_gold(economies: Option<&PlayerEconomies>, bot_id: PlayerId) -> Option<u32> {
    let economies = economies?;
    let economy = economies.0.get(&bot_id)?;
    Some(economy.gold.saturating_sub(economy.reserved_gold))
}

/// Returns true when the bot's hand has reached `MAX_HAND_SIZE`. Falls back to
/// `false` when `PlayerHands` is absent so the heuristic does not over-pass on
/// resource-missing test scaffolds.
fn hand_is_full(hands: Option<&PlayerHands>, bot_id: PlayerId) -> bool {
    hands.map(|h| h.hand_len(bot_id) >= MAX_HAND_SIZE).unwrap_or(false)
}

/// Wave-2 auction branch driver. Runs once per `bot_action_loop` invocation
/// when `RoundPhase::DraftAuction` is active. Routes to either the Wave-1
/// fallback (when `AuctionState` is not in the world) or the Wave-2 heuristic.
#[allow(clippy::too_many_arguments)]
fn run_auction_branch(
    round: u32,
    phase: RoundPhase,
    ts: u64,
    bot_players: &[PlayerId],
    bots: &mut BotPlayers,
    auction: Option<&AuctionState>,
    economies: Option<&PlayerEconomies>,
    hands: Option<&PlayerHands>,
    decision_log: &mut BotDecisionLog,
    auction_pass_logged: &mut HashSet<(PlayerId, u32)>,
    auction_round_ctx: &mut HashMap<(PlayerId, u32), AuctionRoundContext>,
    // PROMPT 1598: when `Some`, every `AuctionDecision::Bid` is funnelled
    // into the auction module's server-internal queue so the bot bid is
    // validated and applied by `process_bid_batch` on the same tick (one
    // frame of MessageWriter latency notwithstanding). When `None` (legacy
    // tests that bypass the auction plugin) Wave-2 decision-only behaviour
    // is preserved.
    mut pending_bot_bids: Option<&mut PendingBotBids>,
) {
    // Wave-1 fallback when AuctionState is absent (legacy tests, pre-auction
    // scaffolds). Preserves the "pass once per round" contract.
    let Some(auction) = auction else {
        for player_id in bot_players {
            let key = (*player_id, round);
            if auction_pass_logged.contains(&key) {
                continue;
            }
            auction_pass_logged.insert(key);

            let (seed, counter) = seed_snapshot(bots, *player_id);
            decision_log.push(BotDecisionEntry {
                round_number: round,
                phase: protocol_phase(phase),
                bot_player_id: *player_id,
                decision: BotDecisionKind::AuctionPass {
                    reason: "auction_state_unavailable",
                },
                timestamp_ms: ts,
                legal_action_count: None,
                seed,
                seed_word_counter: counter,
            });
            tracing::info!(
                target: "server::bot",
                bot_player_id = ?player_id,
                round,
                "bot_action_loop: auction pass (AuctionState resource absent; wave-1 fallback)"
            );
        }
        return;
    };

    // Wave-2 path: bot needs an active LiveBidding auction with a card. If
    // the auction is still selecting or already settled, log one pass per
    // round and bail.
    let Some(card_id) = auction.card_id else {
        for player_id in bot_players {
            let key = (*player_id, round);
            if auction_pass_logged.contains(&key) {
                continue;
            }
            auction_pass_logged.insert(key);

            let (seed, counter) = seed_snapshot(bots, *player_id);
            decision_log.push(BotDecisionEntry {
                round_number: round,
                phase: protocol_phase(phase),
                bot_player_id: *player_id,
                decision: BotDecisionKind::AuctionPass {
                    reason: "auction_card_not_selected",
                },
                timestamp_ms: ts,
                legal_action_count: None,
                seed,
                seed_word_counter: counter,
            });
        }
        return;
    };

    for player_id in bot_players {
        // Either reuse this round's cached context or compute a fresh one.
        // Resetting on card change keeps the heuristic well-defined if the
        // protocol ever swaps cards mid-round (today it does not).
        let context = match auction_round_ctx.get(&(*player_id, round)) {
            Some(ctx) if ctx.card_id == card_id => *ctx,
            _ => {
                let Some(bot_state) = bots.get_mut(*player_id) else {
                    continue;
                };
                let valuation = draw_valuation(bot_state, auction.starting_price);
                let ctx = AuctionRoundContext {
                    card_id,
                    valuation,
                    // Sentinel so the first real decision (against the live
                    // `current_price`) always fires regardless of whether
                    // `current_price` is at the floor or already raised.
                    last_decision_price: u32::MAX,
                };
                auction_round_ctx.insert((*player_id, round), ctx);
                ctx
            }
        };

        // Idempotency: skip if we've already produced a decision at this
        // exact `current_price`. Future ticks at a higher price (a human
        // raised) will re-evaluate; the bot can then re-bid against the new
        // price up to its reservation.
        if context.last_decision_price == auction.current_price {
            continue;
        }

        let gold = unreserved_gold(economies, *player_id).unwrap_or(0);
        let hand_full = hand_is_full(hands, *player_id);
        let decision = decide_auction_action(*player_id, auction, &context, gold, hand_full);

        // Update the per-round context's `last_decision_price` so we don't
        // re-log against the same price next tick.
        if let Some(ctx) = auction_round_ctx.get_mut(&(*player_id, round)) {
            ctx.last_decision_price = auction.current_price;
        }

        let (seed, counter) = seed_snapshot(bots, *player_id);
        match decision {
            AuctionDecision::Bid { amount, valuation } => {
                decision_log.push(BotDecisionEntry {
                    round_number: round,
                    phase: protocol_phase(phase),
                    bot_player_id: *player_id,
                    decision: BotDecisionKind::AuctionBid {
                        card_id,
                        amount,
                        valuation,
                    },
                    timestamp_ms: ts,
                    // The heuristic considered a single bid amount (no
                    // alternatives), hence 1. Future heuristics that
                    // evaluate multiple counter-bids should report the
                    // actual evaluated count here.
                    legal_action_count: Some(1),
                    seed,
                    seed_word_counter: counter,
                });
                // PROMPT 1598 (Wave 2.5): funnel the chosen bid into the
                // server-internal queue drained by `auction_tick_system`.
                // `peer_id: None` — bots have no client peer; the auction
                // outbox layer broadcasts acceptance to connected peers and
                // skips the unicast `S2CCardAcquired` (no peer to send to)
                // without affecting authoritative state (hand_push + gold
                // settlement still run in `settle_expired_auction`).
                let queued = pending_bot_bids.is_some();
                if let Some(queue) = pending_bot_bids.as_mut() {
                    queue.push(AuctionBid {
                        bidder: *player_id,
                        peer_id: None,
                        amount,
                    });
                }
                tracing::info!(
                    target: "server::bot",
                    bot_player_id = ?player_id,
                    round,
                    card_id = ?card_id,
                    amount,
                    valuation,
                    current_price = auction.current_price,
                    queued,
                    "bot_action_loop: auction bid funnelled into PendingBotBids (Wave 2.5)"
                );
            }
            AuctionDecision::Pass { reason } => {
                decision_log.push(BotDecisionEntry {
                    round_number: round,
                    phase: protocol_phase(phase),
                    bot_player_id: *player_id,
                    decision: BotDecisionKind::AuctionPass { reason },
                    timestamp_ms: ts,
                    legal_action_count: Some(0),
                    seed,
                    seed_word_counter: counter,
                });
                tracing::info!(
                    target: "server::bot",
                    bot_player_id = ?player_id,
                    round,
                    card_id = ?card_id,
                    reason,
                    current_price = auction.current_price,
                    "bot_action_loop: auction pass (heuristic gate)"
                );
            }
        }
    }

}

/// Wallclock helper: convert `Time::elapsed()` to integer milliseconds for the
/// decision log. Matches the convention used by `bot_lobby_auto_confirm`.
fn now_ms(time: &Time) -> u64 {
    (time.elapsed().as_secs_f64() * 1_000.0) as u64
}

/// Bot action-loop system.
///
/// Runs every frame after the RSM input reader and before phase transitions
/// (see `BotActionLoopPlugin`). Per phase, for each bot in the current
/// session:
///
/// | Phase                  | Action                                              |
/// |------------------------|-----------------------------------------------------|
/// | `Lobby`                | no-op (handled by [`bot_lobby_auto_confirm`])       |
/// | `DraftInitial`         | write `DraftReadySignal { ready: true }` once       |
/// | `DraftShop`            | write `DraftReadySignal { ready: true }` once       |
/// | `DraftAuction`         | log `AuctionPass` once per round (intentional pass) |
/// | `Placement`            | write empty `PlacementSubmissionReceived` once      |
/// | `Resolution`/`GameOver`| no-op                                               |
///
/// Idempotency uses RSM-owned state where possible (`draft_ready_players`,
/// `submissions_received`) so the bot stops re-emitting the moment the RSM
/// records the signal. The auction pass uses a system-local set keyed by
/// `(player, round_number)` because auction has no analogous server-side set.
#[allow(clippy::too_many_arguments)]
pub fn bot_action_loop(
    time: Res<Time>,
    round_state: Option<Res<RoundState>>,
    session: Option<Res<SessionConfig>>,
    mut bots: ResMut<BotPlayers>,
    auction: Option<Res<AuctionState>>,
    economies: Option<Res<PlayerEconomies>>,
    hands: Option<Res<PlayerHands>>,
    mut decision_log: ResMut<BotDecisionLog>,
    mut ready_signals: MessageWriter<DraftReadySignal>,
    mut placement_submissions: MessageWriter<PlacementSubmissionReceived>,
    // PROMPT 1598: bot bid funnel destination. `Option<ResMut<_>>` so
    // legacy tests that bypass `AuctionPlugin` still compile (the bot
    // logs an auction pass decision; nothing is funnelled).
    mut pending_bot_bids: Option<ResMut<PendingBotBids>>,
    mut auction_pass_logged: Local<HashSet<(PlayerId, u32)>>,
    mut auction_round_ctx: Local<HashMap<(PlayerId, u32), AuctionRoundContext>>,
) {
    let (Some(round_state), Some(session)) = (round_state, session) else {
        return;
    };
    if bots.is_empty() {
        return;
    }

    let bot_players = session_bot_players(&bots, &session);
    if bot_players.is_empty() {
        return;
    }

    let phase = round_state.phase;
    let round = round_state.round_number;
    let ts = now_ms(&time);

    match phase {
        RoundPhase::DraftInitial | RoundPhase::DraftShop => {
            for player_id in &bot_players {
                if round_state.draft_ready_players.contains(player_id) {
                    continue;
                }
                ready_signals.write(DraftReadySignal {
                    player: *player_id,
                    ready: true,
                });

                let (seed, counter) = seed_snapshot(&bots, *player_id);
                decision_log.push(BotDecisionEntry {
                    round_number: round,
                    phase: protocol_phase(phase),
                    bot_player_id: *player_id,
                    decision: BotDecisionKind::DraftReady,
                    timestamp_ms: ts,
                    legal_action_count: None,
                    seed,
                    seed_word_counter: counter,
                });

                tracing::info!(
                    target: "server::bot",
                    bot_player_id = ?player_id,
                    phase = ?phase,
                    round,
                    "bot_action_loop: DraftReadySignal emitted (deterministic pass)"
                );
            }
        }
        RoundPhase::DraftAuction => {
            run_auction_branch(
                round,
                phase,
                ts,
                &bot_players,
                &mut bots,
                auction.as_deref(),
                economies.as_deref(),
                hands.as_deref(),
                &mut decision_log,
                &mut auction_pass_logged,
                &mut auction_round_ctx,
                pending_bot_bids.as_deref_mut(),
            );
        }
        RoundPhase::Placement => {
            for player_id in &bot_players {
                if round_state.submissions_received.contains(player_id) {
                    continue;
                }
                // Empty placement = legal no-op fail-safe. The board placement
                // pipeline treats an empty vector as a deliberate "submit
                // nothing"; the bot does not consume hand or gold.
                let submission = PlacementSubmissionReceived {
                    player: *player_id,
                    peer_id: None::<PeerId>,
                    placements: Vec::new(),
                };
                placement_submissions.write(submission);

                let (seed, counter) = seed_snapshot(&bots, *player_id);
                decision_log.push(BotDecisionEntry {
                    round_number: round,
                    phase: protocol_phase(phase),
                    bot_player_id: *player_id,
                    decision: BotDecisionKind::EmptyPlacementFailsafe,
                    timestamp_ms: ts,
                    legal_action_count: Some(0),
                    seed,
                    seed_word_counter: counter,
                });

                tracing::info!(
                    target: "server::bot",
                    bot_player_id = ?player_id,
                    round,
                    "bot_action_loop: empty placement submitted (wave1 no-op fail-safe)"
                );
            }
        }
        RoundPhase::Lobby | RoundPhase::Resolution | RoundPhase::GameOver => {
            // No bot participation in these phases for wave1. Lobby is owned by
            // `bot_lobby_auto_confirm`; Resolution/GameOver have no bot inputs.
        }
    }
}

/// Plugin: register the bot action loop on the standard `Update` schedule.
///
/// Ordering: this system writes `DraftReadySignal` and
/// `PlacementSubmissionReceived`, which are read by
/// `core::rsm::transitions::rsm_input_reader` and
/// `feature::board::placement::handle_placement_submission` respectively.
/// Bevy's MessageWriter/MessageReader contract is event-bus-style (one frame
/// of latency is fine), so we do not declare hard ordering against those
/// systems — the signal lands on the next tick which is sufficient for flow
/// progress.
pub struct BotActionLoopPlugin;

impl Plugin for BotActionLoopPlugin {
    fn build(&self, app: &mut App) {
        // BotPlayers and BotDecisionLog are already initialised by
        // `BotLobbyPlugin`; `init_resource` is a no-op if present, so this
        // plugin is safe to add in either order.
        app.init_resource::<BotPlayers>()
            .init_resource::<BotDecisionLog>()
            .add_systems(Update, bot_action_loop);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::rsm::DraftReadySignal;
    use crate::core::session::config::SessionConfig;
    use crate::feature::board::PlacementSubmissionReceived;
    use crate::feature::bot::state::BotState;
    use bevy::app::App;
    use bevy::time::TimePlugin;
    use shared::card::ClassId;
    use shared::protocol::{GameMode, PlacementTimerMultiplier};
    use std::collections::HashMap;

    const BOT_ID: PlayerId = PlayerId(1 << 63);
    const HUMAN_ID: PlayerId = PlayerId(7);

    fn test_session_with_bot() -> SessionConfig {
        let mut team_map = HashMap::new();
        team_map.insert(BOT_ID, 1);
        team_map.insert(HUMAN_ID, 0);
        let mut class_map = HashMap::new();
        class_map.insert(BOT_ID, ClassId::Iop);
        class_map.insert(HUMAN_ID, ClassId::Cra);
        SessionConfig {
            mode: GameMode::OneVOne,
            player_count: 2,
            team_map,
            class_map,
            placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        }
    }

    fn make_app(phase: RoundPhase, round: u32) -> App {
        let mut app = App::new();
        app.add_plugins(TimePlugin);
        app.add_plugins(BotActionLoopPlugin);
        // Register the messages the loop writes so MessageWriter has a queue.
        app.add_message::<DraftReadySignal>();
        app.add_message::<PlacementSubmissionReceived>();

        let mut round_state = RoundState::new();
        round_state.phase = phase;
        round_state.round_number = round;
        app.insert_resource(round_state);
        app.insert_resource(test_session_with_bot());

        let mut bots = BotPlayers::default();
        bots.insert(BotState::new(BOT_ID, BOT_ID.0));
        app.insert_resource(bots);

        app
    }

    fn drain_ready(app: &mut App) -> Vec<DraftReadySignal> {
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<DraftReadySignal>>()
            .drain()
            .collect()
    }

    fn drain_placement(app: &mut App) -> Vec<PlacementSubmissionReceived> {
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<PlacementSubmissionReceived>>()
            .drain()
            .collect()
    }

    #[test]
    fn draft_initial_emits_ready_for_bot_only() {
        let mut app = make_app(RoundPhase::DraftInitial, 1);
        app.update();

        let ready: Vec<_> = drain_ready(&mut app);
        assert_eq!(ready.len(), 1, "exactly one ready signal expected");
        assert_eq!(ready[0].player, BOT_ID);
        assert!(ready[0].ready);

        let log = app.world().resource::<BotDecisionLog>();
        assert_eq!(log.len(), 1);
        assert!(matches!(
            log.entries[0].decision,
            BotDecisionKind::DraftReady
        ));
    }

    #[test]
    fn draft_ready_idempotent_after_rsm_records_signal() {
        let mut app = make_app(RoundPhase::DraftInitial, 1);
        app.update();
        // Simulate the RSM having recorded the ready signal.
        app.world_mut()
            .resource_mut::<RoundState>()
            .draft_ready_players
            .insert(BOT_ID);
        let _ = drain_ready(&mut app);

        app.update();
        let ready_after = drain_ready(&mut app);
        assert!(
            ready_after.is_empty(),
            "no duplicate ready signal once RSM tracks the bot"
        );
    }

    #[test]
    fn draft_shop_also_emits_ready() {
        let mut app = make_app(RoundPhase::DraftShop, 2);
        app.update();
        let ready = drain_ready(&mut app);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].player, BOT_ID);
    }

    #[test]
    fn placement_emits_empty_failsafe_once() {
        let mut app = make_app(RoundPhase::Placement, 3);
        app.update();

        let placements = drain_placement(&mut app);
        assert_eq!(placements.len(), 1);
        assert_eq!(placements[0].player, BOT_ID);
        assert!(placements[0].placements.is_empty());
        assert!(placements[0].peer_id.is_none());

        // Mark as received; second tick should not re-emit.
        app.world_mut()
            .resource_mut::<RoundState>()
            .submissions_received
            .insert(BOT_ID);
        app.update();
        let again = drain_placement(&mut app);
        assert!(
            again.is_empty(),
            "placement fail-safe must not re-emit once RSM records the submission"
        );

        let log = app.world().resource::<BotDecisionLog>();
        assert!(log
            .entries
            .iter()
            .any(|e| matches!(e.decision, BotDecisionKind::EmptyPlacementFailsafe)));
    }

    #[test]
    fn auction_logs_pass_once_per_round() {
        let mut app = make_app(RoundPhase::DraftAuction, 4);
        app.update();
        app.update();
        app.update();

        let log = app.world().resource::<BotDecisionLog>();
        let pass_count = log
            .entries
            .iter()
            .filter(|e| matches!(e.decision, BotDecisionKind::AuctionPass { .. }))
            .count();
        assert_eq!(pass_count, 1, "auction pass logged exactly once per round");

        // No ready or placement signals during auction.
        assert!(drain_ready(&mut app).is_empty());
        assert!(drain_placement(&mut app).is_empty());

        // A new round produces a new pass log entry.
        app.world_mut().resource_mut::<RoundState>().round_number = 5;
        app.update();
        let log = app.world().resource::<BotDecisionLog>();
        let pass_count = log
            .entries
            .iter()
            .filter(|e| matches!(e.decision, BotDecisionKind::AuctionPass { .. }))
            .count();
        assert_eq!(pass_count, 2, "next auction round logs a fresh pass");
    }

    #[test]
    fn idle_phases_emit_nothing() {
        for phase in [RoundPhase::Lobby, RoundPhase::Resolution, RoundPhase::GameOver] {
            let mut app = make_app(phase, 0);
            app.update();
            assert!(drain_ready(&mut app).is_empty(), "{:?}: no ready", phase);
            assert!(drain_placement(&mut app).is_empty(), "{:?}: no placement", phase);
            let log = app.world().resource::<BotDecisionLog>();
            assert_eq!(log.len(), 0, "{:?}: no decision log entries", phase);
        }
    }

    #[test]
    fn humans_only_session_is_a_noop() {
        let mut app = App::new();
        app.add_plugins(TimePlugin);
        app.add_plugins(BotActionLoopPlugin);
        app.add_message::<DraftReadySignal>();
        app.add_message::<PlacementSubmissionReceived>();

        let mut round_state = RoundState::new();
        round_state.phase = RoundPhase::DraftInitial;
        app.insert_resource(round_state);

        // Session with only humans.
        let mut team_map = HashMap::new();
        team_map.insert(HUMAN_ID, 0);
        team_map.insert(PlayerId(8), 1);
        let mut class_map = HashMap::new();
        class_map.insert(HUMAN_ID, ClassId::Cra);
        class_map.insert(PlayerId(8), ClassId::Iop);
        app.insert_resource(SessionConfig {
            mode: GameMode::OneVOne,
            player_count: 2,
            team_map,
            class_map,
            placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        });

        // BotPlayers is empty.
        app.insert_resource(BotPlayers::default());

        app.update();
        assert!(drain_ready(&mut app).is_empty());
        assert!(drain_placement(&mut app).is_empty());
        let log = app.world().resource::<BotDecisionLog>();
        assert_eq!(log.len(), 0);
    }
}
