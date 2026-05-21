//! Wave 2.5 auction bid funnel regression tests (PROMPT 1598).
//!
//! Scope: prove that a bot bid decision plumbed into the
//! `PendingBotBids` server-internal queue is drained and processed by the
//! authoritative `process_bid_batch` path, with no protocol changes and no
//! auction-rule shortcuts.
//!
//! These tests stay below the full `AuctionPlugin` schedule so we can drive
//! the funnel as a focused unit: the bot loop pushes into `PendingBotBids`,
//! then we invoke `process_bid_batch` directly with the drained batch and
//! assert the resulting `AuctionState` mutations.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::time::TimePlugin;
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::rsm::{DraftReadySignal, RoundPhase, RoundState};
use server::core::session::config::SessionConfig;
use server::feature::acquisition::PlayerHands;
use server::feature::auction::{
    process_bid_batch, AuctionNetworkOutbox, AuctionPhase, AuctionState, PendingBotBids,
};
use server::feature::board::PlacementSubmissionReceived;
use server::feature::bot::{bot_action_loop, BotDecisionKind, BotDecisionLog, BotPlayers, BotState};
use server::foundation::config::GameConfig;
use shared::card::{CardId, ClassId};
use shared::protocol::{GameMode, PlacementTimerMultiplier};
use shared::session::PlayerId;

const BOT: PlayerId = PlayerId(1 << 63);
const HUMAN: PlayerId = PlayerId(11);
const SEED: u64 = 0xFEED_FACE_DEAD_BEEF;
const AUCTION_CARD: CardId = CardId(202);

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

fn live_auction(starting_price: u32, current_price: u32) -> AuctionState {
    AuctionState {
        phase: AuctionPhase::LiveBidding,
        card_id: Some(AUCTION_CARD),
        starting_price,
        current_price,
        current_leader: None,
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

fn make_funnel_app(auction: AuctionState, gold: u32) -> App {
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
    // PROMPT 1598: the funnel destination. Tests that intentionally exercise
    // the legacy decision-only fallback simply omit this resource.
    app.init_resource::<PendingBotBids>();

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

#[test]
fn test_bot_auction_bid_funnel_enqueues_decision_into_pending_bot_bids() {
    // Arrange: live auction, bot has plenty of gold to clear the floor.
    let mut app = make_funnel_app(live_auction(3, 3), 20);

    // Act: bot loop ticks once and the bid decision is funnelled into the
    // server-internal queue.
    app.update();

    // Assert: the decision log holds an AuctionBid AND the queue is non-empty
    // with the same bidder + amount.
    let queue = app.world().resource::<PendingBotBids>();
    assert_eq!(
        queue.len(),
        1,
        "exactly one bot bid funnelled into PendingBotBids"
    );

    let (expected_amount, expected_card) = match last_decision(&app) {
        BotDecisionKind::AuctionBid {
            card_id, amount, ..
        } => (amount, card_id),
        other => panic!("expected AuctionBid decision, got {:?}", other),
    };

    let queued = app.world_mut().resource_mut::<PendingBotBids>().drain();
    assert_eq!(queued.len(), 1);
    let funnelled = queued[0];
    assert_eq!(
        funnelled.bidder, BOT,
        "queued bid carries the bot PlayerId"
    );
    assert_eq!(
        funnelled.amount, expected_amount,
        "queued amount matches the decision log entry"
    );
    assert!(
        funnelled.peer_id.is_none(),
        "bots have no PeerId; queued bid carries peer_id = None"
    );
    assert_eq!(
        expected_card, AUCTION_CARD,
        "decision references the live auction card"
    );
}

#[test]
fn test_bot_auction_pass_does_not_enqueue_into_pending_bot_bids() {
    // Arrange: live auction but bot is too poor to clear the floor.
    let mut app = make_funnel_app(live_auction(5, 5), 1);

    // Act
    app.update();

    // Assert: the decision log records a pass and the queue stays empty —
    // the funnel must not push spurious bids when the heuristic declines.
    match last_decision(&app) {
        BotDecisionKind::AuctionPass { reason } => {
            assert_eq!(reason, "insufficient_gold");
        }
        other => panic!("expected AuctionPass{{insufficient_gold}}, got {:?}", other),
    }
    let queue = app.world().resource::<PendingBotBids>();
    assert!(
        queue.is_empty(),
        "AuctionPass must not enqueue anything into PendingBotBids"
    );
}

#[test]
fn test_bot_auction_bid_clears_through_process_bid_batch_and_becomes_leader() {
    // Arrange: live auction, bot funnels its bid via the same bot loop that
    // production wires. Then drain the queue and run it through the
    // authoritative `process_bid_batch` — no shortcut, no special-case.
    let mut app = make_funnel_app(live_auction(3, 3), 50);
    app.update();

    let funnelled = app.world_mut().resource_mut::<PendingBotBids>().drain();
    assert_eq!(funnelled.len(), 1, "bot enqueued a bid");
    let bid = funnelled[0];

    // Act: feed the funnelled bid into the production bid-batch processor
    // (same path `auction_tick_system` uses after draining both sources).
    let mut auction = app.world().resource::<AuctionState>().clone();
    let mut economies = PlayerEconomies::default();
    economies.0.insert(BOT, economy_with_gold(50));
    let hands = PlayerHands::default();
    let config = GameConfig(shared::config::GameConfig::default());
    let mut outbox = AuctionNetworkOutbox::default();
    let mut gold_broadcasts = Vec::new();
    process_bid_batch(
        &mut auction,
        &mut economies,
        Some(&hands),
        &config,
        None,
        vec![bid],
        &mut outbox,
        &mut gold_broadcasts,
    );

    // Assert: the bot is now the current leader, current_price reflects the
    // bot's amount, no rejection was emitted, gold reservation moved off
    // the bot's purse, and an accepted dispatch was queued.
    assert_eq!(
        auction.current_leader,
        Some(BOT),
        "bot becomes auction leader through the normal validation path"
    );
    assert_eq!(
        auction.current_price, bid.amount,
        "auction current_price advances to the bot's bid amount"
    );
    assert!(
        outbox.rejected().is_empty(),
        "funnelled bot bid must not be rejected by process_bid_batch"
    );
    assert_eq!(
        outbox.accepted().len(),
        1,
        "process_bid_batch emits one acceptance for the bot bid"
    );
    let accepted = &outbox.accepted()[0];
    assert_eq!(accepted.player_id, BOT);
    assert_eq!(accepted.message.bidder, BOT);
    assert_eq!(accepted.message.amount, bid.amount);
    assert!(
        accepted.peer_id.is_none(),
        "bot acceptance dispatch carries no peer_id (bot has no client)"
    );

    let bot_economy = economies.0.get(&BOT).expect("bot economy present");
    assert_eq!(
        bot_economy.reserved_gold, bid.amount,
        "reservation equals the funnelled bid amount"
    );
    assert_eq!(
        bot_economy.gold, 50,
        "gross gold not yet spent — reservation only at LiveBidding"
    );
}

#[test]
fn test_bot_auction_bid_funnel_idempotent_at_same_current_price() {
    // Arrange: the heuristic already enforces "one decision per
    // (player, round, current_price)". Wave 2.5 must not double-enqueue
    // across re-ticks at the same price either.
    let mut app = make_funnel_app(live_auction(3, 3), 20);

    // Act: three ticks at the same price.
    app.update();
    app.update();
    app.update();

    // Assert: still exactly one queued bid.
    let queue = app.world().resource::<PendingBotBids>();
    assert_eq!(
        queue.len(),
        1,
        "no duplicate funnelled bid at unchanged current_price"
    );
}
