//! Wave-2 auction bid decision tests for the bot action loop (PROMPT 1582).
//!
//! Scope: prove that the deterministic auction bid heuristic added in Wave 2
//! produces the expected `BotDecisionKind::AuctionBid` / `AuctionPass` log
//! entries given the observable `AuctionState`/`PlayerEconomies`/`PlayerHands`
//! resources. Wave 2 is decision-only — these tests do NOT assert any wire
//! message was emitted because PROMPT 1582 explicitly defers auction-side
//! ingestion to a later wave.
//!
//! The tests build a minimal `App` with `bot_action_loop` registered and
//! hand-craft the input resources for each scenario. We deliberately avoid
//! the full `BotActionLoopPlugin` schedule so we can drive single-tick
//! state transitions with `app.update()` without colliding with other
//! feature plugins.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::time::TimePlugin;
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::rsm::DraftReadySignal;
use server::core::rsm::{RoundPhase, RoundState};
use server::core::session::config::SessionConfig;
use server::feature::acquisition::PlayerHands;
use server::feature::auction::{AuctionPhase, AuctionState};
use server::feature::board::PlacementSubmissionReceived;
use server::feature::bot::{
    bot_action_loop, BotDecisionKind, BotDecisionLog, BotPlayers, BotState,
};
use shared::card::{CardId, ClassId};
use shared::protocol::{GameMode, PlacementTimerMultiplier};
use shared::session::PlayerId;

const BOT: PlayerId = PlayerId(1 << 63);
const HUMAN: PlayerId = PlayerId(11);
const SEED: u64 = 0xDEAD_BEEF_C0FFEE_42;
const AUCTION_CARD: CardId = CardId(101);

fn test_session() -> SessionConfig {
    let mut team_map = HashMap::new();
    team_map.insert(BOT, 1);
    team_map.insert(HUMAN, 0);
    let mut class_map = HashMap::new();
    class_map.insert(BOT, ClassId::Iop);
    class_map.insert(HUMAN, ClassId::Cra);
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map,
        class_map,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
    }
}

fn live_auction(starting_price: u32, current_price: u32, current_leader: Option<PlayerId>) -> AuctionState {
    AuctionState {
        phase: AuctionPhase::LiveBidding,
        card_id: Some(AUCTION_CARD),
        starting_price,
        current_price,
        current_leader,
        // Comfortably above BOT_AUCTION_PASS_THRESHOLD_MS so the timer-gate
        // does not fire unless a test explicitly overrides it.
        timer_remaining_ms: 10_000,
        live_bidding_deadline_elapsed_ms: None,
    }
}

fn economy_with_gold(gold: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold,
        current_mana: 0,
        reserve_mana: 0,
        mana_cap: 0,
        reserved_gold: 0,
    }
}

fn make_app(auction: AuctionState, gold: u32) -> App {
    let mut app = App::new();
    app.add_plugins(TimePlugin);
    app.add_message::<DraftReadySignal>();
    app.add_message::<PlacementSubmissionReceived>();
    app.add_systems(Update, bot_action_loop);

    let mut round_state = RoundState::new();
    round_state.phase = RoundPhase::DraftAuction;
    round_state.round_number = 1;
    app.insert_resource(round_state);
    app.insert_resource(test_session());

    let mut bots = BotPlayers::default();
    bots.insert(BotState::new(BOT, SEED));
    app.insert_resource(bots);
    app.init_resource::<BotDecisionLog>();

    app.insert_resource(auction);

    let mut economies = PlayerEconomies::default();
    economies.0.insert(BOT, economy_with_gold(gold));
    app.insert_resource(economies);

    app.insert_resource(PlayerHands::default());

    app
}

fn last_decision(app: &App) -> BotDecisionKind {
    app.world()
        .resource::<BotDecisionLog>()
        .last()
        .expect("expected at least one bot decision entry")
        .decision
        .clone()
}

fn count_decisions(app: &App) -> usize {
    app.world().resource::<BotDecisionLog>().entries.len()
}

#[test]
fn test_bot_auction_emits_bid_when_gold_and_valuation_clear_gate() {
    // Arrange: live auction, bot has plenty of gold, hand has room.
    let mut app = make_app(live_auction(3, 3, None), 20);

    // Act
    app.update();

    // Assert: exactly one decision, and it is an AuctionBid for the
    // auction card with an amount strictly above current_price (the bid
    // must clear the +1 floor enforced by process_bid_batch).
    assert_eq!(count_decisions(&app), 1, "one bid decision per (player, round, price)");
    match last_decision(&app) {
        BotDecisionKind::AuctionBid { card_id, amount, valuation } => {
            assert_eq!(card_id, AUCTION_CARD, "bid is for the live auction card");
            assert!(amount > 3, "bid amount must outbid the current price floor");
            assert!(valuation >= amount, "amount must not exceed reservation valuation");
        }
        other => panic!("expected AuctionBid, got {:?}", other),
    }
}

#[test]
fn test_bot_auction_passes_when_unreserved_gold_is_insufficient() {
    // Arrange: starting price is 5, bot only has 1 gold.
    let mut app = make_app(live_auction(5, 5, None), 1);

    // Act
    app.update();

    // Assert: bot logs an AuctionPass with the insufficient_gold reason.
    match last_decision(&app) {
        BotDecisionKind::AuctionPass { reason } => {
            assert_eq!(reason, "insufficient_gold");
        }
        other => panic!("expected AuctionPass{{insufficient_gold}}, got {:?}", other),
    }
}

#[test]
fn test_bot_auction_passes_when_hand_is_full() {
    // Arrange: live auction with affordable price, but hand already at cap.
    let mut app = make_app(live_auction(3, 3, None), 50);
    {
        let mut hands = app.world_mut().resource_mut::<PlayerHands>();
        // MAX_HAND_SIZE is 10 — push 10 dummy cards.
        for i in 0..10 {
            hands.push_card(BOT, CardId(900 + i));
        }
    }

    // Act
    app.update();

    // Assert
    match last_decision(&app) {
        BotDecisionKind::AuctionPass { reason } => {
            assert_eq!(reason, "hand_full");
        }
        other => panic!("expected AuctionPass{{hand_full}}, got {:?}", other),
    }
}

#[test]
fn test_bot_auction_passes_when_bot_is_already_leader() {
    // Arrange: auction reports bot as current_leader.
    let mut app = make_app(live_auction(3, 4, Some(BOT)), 50);

    // Act
    app.update();

    // Assert
    match last_decision(&app) {
        BotDecisionKind::AuctionPass { reason } => {
            assert_eq!(reason, "already_leader");
        }
        other => panic!("expected AuctionPass{{already_leader}}, got {:?}", other),
    }
}

#[test]
fn test_bot_auction_passes_when_timer_below_threshold() {
    // Arrange: bid would otherwise fire but timer is critically low.
    let mut auction = live_auction(3, 3, None);
    auction.timer_remaining_ms = 100; // well below BOT_AUCTION_PASS_THRESHOLD_MS
    let mut app = make_app(auction, 50);

    // Act
    app.update();

    // Assert
    match last_decision(&app) {
        BotDecisionKind::AuctionPass { reason } => {
            assert_eq!(reason, "timer_below_threshold");
        }
        other => panic!("expected AuctionPass{{timer_below_threshold}}, got {:?}", other),
    }
}

#[test]
fn test_bot_auction_decision_is_idempotent_at_same_current_price() {
    // Arrange: live auction; bid should fire on tick 1 but NOT re-fire on
    // tick 2 because current_price has not changed.
    let mut app = make_app(live_auction(3, 3, None), 20);

    // Act
    app.update();
    let entries_after_first_tick = count_decisions(&app);
    app.update();
    app.update();
    let entries_after_three_ticks = count_decisions(&app);

    // Assert: exactly one decision across multiple ticks at the same price.
    assert_eq!(entries_after_first_tick, 1);
    assert_eq!(
        entries_after_three_ticks, 1,
        "no duplicate AuctionBid/Pass when current_price has not advanced",
    );
}

#[test]
fn test_bot_auction_reevaluates_when_current_price_rises() {
    // Arrange: tick 1 — bot bids; tick 2 — simulate a human raising the
    // price, bot should re-evaluate and emit a fresh decision.
    let mut app = make_app(live_auction(3, 3, None), 50);

    // Act
    app.update();
    let entries_after_first = count_decisions(&app);

    // Simulate a human raising the price above the bot's current bid.
    {
        let mut auction = app.world_mut().resource_mut::<AuctionState>();
        auction.current_price = 6;
    }
    app.update();
    let entries_after_raise = count_decisions(&app);

    // Assert: a second decision is logged once the price moves.
    assert_eq!(entries_after_first, 1);
    assert_eq!(
        entries_after_raise, 2,
        "a price raise must produce a fresh bot decision entry",
    );
}

#[test]
fn test_bot_auction_decision_is_deterministic_for_same_seed() {
    // Arrange: two independent apps with the same seed must produce
    // identical valuations and identical decisions.
    let mut app_a = make_app(live_auction(3, 3, None), 20);
    let mut app_b = make_app(live_auction(3, 3, None), 20);

    // Act
    app_a.update();
    app_b.update();

    // Assert
    let a = last_decision(&app_a);
    let b = last_decision(&app_b);
    assert_eq!(a, b, "deterministic seed must produce identical decisions");
}

#[test]
fn test_bot_auction_fallback_when_auction_state_resource_is_absent() {
    // Arrange: no AuctionState inserted — exercises the Wave-1 fallback
    // path so the legacy "pass once per round" contract is preserved for
    // test scaffolds that bypass the auction plugin.
    let mut app = App::new();
    app.add_plugins(TimePlugin);
    app.add_message::<DraftReadySignal>();
    app.add_message::<PlacementSubmissionReceived>();
    app.add_systems(Update, bot_action_loop);

    let mut round_state = RoundState::new();
    round_state.phase = RoundPhase::DraftAuction;
    round_state.round_number = 1;
    app.insert_resource(round_state);
    app.insert_resource(test_session());
    let mut bots = BotPlayers::default();
    bots.insert(BotState::new(BOT, SEED));
    app.insert_resource(bots);
    app.init_resource::<BotDecisionLog>();

    // Act
    app.update();

    // Assert: one pass entry, with the new fallback reason literal.
    match last_decision(&app) {
        BotDecisionKind::AuctionPass { reason } => {
            assert_eq!(reason, "auction_state_unavailable");
        }
        other => panic!("expected AuctionPass{{auction_state_unavailable}}, got {:?}", other),
    }
}
