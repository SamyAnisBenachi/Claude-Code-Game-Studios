use server::core::economy::{PlayerEconomies, PlayerEconomy, S2CGoldBroadcast};
use server::feature::auction::{
    decrement_live_bidding_timer, enforce_live_bidding_deadline, process_bid_batch, AuctionBid,
    AuctionNetworkOutbox, AuctionPhase, AuctionState,
};
use server::foundation::config::GameConfig;
use shared::card::CardId;
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn bid(bidder: PlayerId, amount: u32) -> AuctionBid {
    AuctionBid {
        bidder,
        peer_id: None,
        amount,
    }
}

fn live_auction(
    current_price: u32,
    current_leader: Option<PlayerId>,
    timer_ms: u32,
) -> AuctionState {
    AuctionState {
        phase: AuctionPhase::LiveBidding,
        card_id: Some(CardId(99)),
        starting_price: 3,
        current_price,
        current_leader,
        timer_remaining_ms: timer_ms,
        live_bidding_deadline_elapsed_ms: None,
    }
}

fn economy(gold: u32, reserved_gold: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold,
        current_mana: 0,
        reserve_mana: 0,
        mana_cap: 10,
        reserved_gold,
    }
}

fn economies(entries: &[(PlayerId, u32, u32)]) -> PlayerEconomies {
    PlayerEconomies(
        entries
            .iter()
            .map(|(player, gold, reserved)| (*player, economy(*gold, *reserved)))
            .collect(),
    )
}

fn auction_config(timer_seconds: u32, reset_seconds: u32) -> GameConfig {
    GameConfig(shared::config::GameConfig {
        auction_timer_seconds: timer_seconds,
        auction_timer_reset_seconds: reset_seconds,
        ..shared::config::GameConfig::default()
    })
}

fn process(
    auction: &mut AuctionState,
    economies: &mut PlayerEconomies,
    config: &GameConfig,
    bids: Vec<AuctionBid>,
) -> (AuctionNetworkOutbox, Vec<S2CGoldBroadcast>) {
    let mut network_outbox = AuctionNetworkOutbox::default();
    let mut gold_broadcasts = Vec::new();
    process_bid_batch(
        auction,
        economies,
        None,
        config,
        None,
        bids,
        &mut network_outbox,
        &mut gold_broadcasts,
    );
    (network_outbox, gold_broadcasts)
}

#[test]
fn accepted_outbid_releases_previous_reservation_before_reserving_new_bid() {
    test_helpers::init_test_tracing();
    let previous_leader = player(1);
    let new_leader = player(2);
    let config = auction_config(20, 5);
    let mut auction = live_auction(5, Some(previous_leader), 10_000);
    let mut economies = economies(&[(previous_leader, 10, 5), (new_leader, 10, 0)]);

    let pre_previous_reserved = economies.0[&previous_leader].reserved_gold;
    let pre_new_reserved = economies.0[&new_leader].reserved_gold;

    let (outbox, gold_broadcasts) = process(
        &mut auction,
        &mut economies,
        &config,
        vec![bid(new_leader, 6)],
    );

    assert_eq!(pre_previous_reserved, 5);
    assert_eq!(pre_new_reserved, 0);
    assert_eq!(economies.0[&previous_leader].reserved_gold, 0);
    assert_eq!(economies.0[&new_leader].reserved_gold, 6);
    assert_eq!(auction.current_price, 6);
    assert_eq!(auction.current_leader, Some(new_leader));
    assert_eq!(auction.timer_remaining_ms, 15_000);

    assert_eq!(outbox.accepted().len(), 1);
    assert_eq!(outbox.accepted()[0].player_id, new_leader);
    assert_eq!(outbox.accepted()[0].message.amount, 6);
    assert_eq!(outbox.accepted()[0].message.new_timer_ms, 15_000);
    assert!(outbox.rejected().is_empty());

    assert_eq!(
        gold_broadcasts,
        vec![
            S2CGoldBroadcast {
                player_id: previous_leader,
                gold: 10,
                reserved_gold: 0,
            },
            S2CGoldBroadcast {
                player_id: new_leader,
                gold: 10,
                reserved_gold: 6,
            },
        ]
    );
}

#[test]
fn accepted_first_bid_skips_release_and_broadcasts_only_new_reservation() {
    test_helpers::init_test_tracing();
    let bidder = player(1);
    let config = auction_config(20, 5);
    let mut auction = live_auction(3, None, 10_000);
    let mut economies = economies(&[(bidder, 10, 0)]);

    let (outbox, gold_broadcasts) =
        process(&mut auction, &mut economies, &config, vec![bid(bidder, 4)]);

    assert_eq!(auction.current_price, 4);
    assert_eq!(auction.current_leader, Some(bidder));
    assert_eq!(economies.0[&bidder].reserved_gold, 4);
    assert_eq!(outbox.accepted().len(), 1);
    assert_eq!(
        gold_broadcasts,
        vec![S2CGoldBroadcast {
            player_id: bidder,
            gold: 10,
            reserved_gold: 4,
        }]
    );
}

#[test]
fn accepted_bid_timer_reset_adds_reset_when_below_cap() {
    test_helpers::init_test_tracing();
    let bidder = player(1);
    let config = auction_config(20, 5);
    let mut auction = live_auction(3, None, 3_000);
    let mut economies = economies(&[(bidder, 10, 0)]);

    let (outbox, _gold_broadcasts) =
        process(&mut auction, &mut economies, &config, vec![bid(bidder, 4)]);

    assert_eq!(auction.timer_remaining_ms, 8_000);
    assert_eq!(outbox.accepted()[0].message.new_timer_ms, 8_000);
}

#[test]
fn accepted_bid_timer_reset_caps_at_auction_timer() {
    test_helpers::init_test_tracing();
    let bidder = player(1);
    let config = auction_config(20, 5);
    let mut auction = live_auction(3, None, 17_000);
    let mut economies = economies(&[(bidder, 10, 0)]);

    let (outbox, _gold_broadcasts) =
        process(&mut auction, &mut economies, &config, vec![bid(bidder, 4)]);

    assert_eq!(auction.timer_remaining_ms, 20_000);
    assert_eq!(outbox.accepted()[0].message.new_timer_ms, 20_000);
}

#[test]
fn enforce_live_bidding_deadline_collapses_timer_when_deadline_passed() {
    // PROMPT 1091 / AUDIT-1076-12 regression: in the 2026-05-17 run-7 manual
    // playtest, a 20s auction window took 149s of wall-clock time to settle.
    // The bid extension reset `timer_remaining_ms` to 20_000, but the per-tick
    // `decrement_live_bidding_timer` under-counted real time (~13% of
    // wall-clock), so settlement only fired ~7.5x later than configured.
    //
    // `enforce_live_bidding_deadline` is the safety net: it collapses
    // `timer_remaining_ms` to the absolute remainder against
    // `live_bidding_deadline_elapsed_ms`. Even when the decrement path is
    // off by 7x, the deadline anchor is wall-clock truthful — so the auction
    // still settles within (initial + extension) seconds of the last bid.
    test_helpers::init_test_tracing();

    // Scenario A: decrement under-counted (timer_remaining_ms still says
    // 19_500), but real wall-clock has already passed the absolute deadline.
    // Safety net must collapse to 0 so `settle_expired_auction` fires.
    let mut auction = live_auction(5, None, 19_500);
    auction.live_bidding_deadline_elapsed_ms = Some(10_000);
    let now_ms = 30_000; // 20s past the deadline
    enforce_live_bidding_deadline(&mut auction, now_ms);
    assert_eq!(
        auction.timer_remaining_ms, 0,
        "deadline past -> timer must collapse to 0 for settle_expired_auction"
    );

    // Scenario B: deadline still in the future, decrement value is the
    // smaller of the two — safety net must not move the timer backwards
    // (i.e., must not extend it).
    let mut auction = live_auction(5, None, 5_000);
    auction.live_bidding_deadline_elapsed_ms = Some(50_000);
    let now_ms = 35_000; // deadline_remaining = 15_000, decrement value = 5_000
    enforce_live_bidding_deadline(&mut auction, now_ms);
    assert_eq!(
        auction.timer_remaining_ms, 5_000,
        "decrement smaller than deadline_remaining -> use decrement value"
    );

    // Scenario C: deadline anchor absent (None) — no-op so unit tests and
    // legacy callers without a Time resource keep their existing semantics.
    let mut auction = live_auction(5, None, 12_000);
    auction.live_bidding_deadline_elapsed_ms = None;
    enforce_live_bidding_deadline(&mut auction, 99_999_999);
    assert_eq!(
        auction.timer_remaining_ms, 12_000,
        "no deadline anchor -> enforce_live_bidding_deadline must be a no-op"
    );

    // Scenario D: phase is not LiveBidding — no-op even with a deadline set.
    let mut auction = live_auction(5, None, 12_000);
    auction.phase = AuctionPhase::Resolving;
    auction.live_bidding_deadline_elapsed_ms = Some(0);
    enforce_live_bidding_deadline(&mut auction, 99_999_999);
    assert_eq!(
        auction.timer_remaining_ms, 12_000,
        "non-LiveBidding phase -> enforce_live_bidding_deadline must be a no-op"
    );
}

#[test]
fn timer_decrement_subtracts_raw_delta_and_saturates_at_zero() {
    // Regression: a prior `.min(1000)` clamp here caused auctions to never
    // settle when Update ticks were sparse (e.g., 17-min stuck auctions in
    // manual playtest 2026-05-12). The timer must track real wall-clock
    // delta so settlement fires within the configured window even when the
    // schedule fires infrequently between bid bursts.
    test_helpers::init_test_tracing();
    for (start, raw_delta, expected) in [
        (12_000, 5_000, 7_000),
        (12_000, 1_000, 11_000),
        (12_000, 999, 11_001),
        (12_000, 1_001, 10_999),
        (500, 5_000, 0),
        (20_000, 20_000, 0),
        (20_000, 60_000, 0),
    ] {
        let mut auction = live_auction(5, None, start);

        decrement_live_bidding_timer(&mut auction, raw_delta);

        assert_eq!(
            auction.timer_remaining_ms, expected,
            "decrement_live_bidding_timer(start={start}, raw_delta={raw_delta})"
        );
    }
}
