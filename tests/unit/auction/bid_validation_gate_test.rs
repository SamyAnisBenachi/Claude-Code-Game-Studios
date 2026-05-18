use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::feature::acquisition::PlayerHands;
use server::feature::auction::{
    process_bid_batch, AuctionBid, AuctionNetworkOutbox, AuctionPhase, AuctionState,
};
use server::foundation::config::GameConfig;
use shared::card::CardId;
use shared::protocol::BidRejectedReason;
use shared::session::PlayerId;

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

fn hands(entries: &[(PlayerId, usize)]) -> PlayerHands {
    let mut hands = PlayerHands::default();
    for (player, count) in entries {
        hands.hands.insert(
            *player,
            (0..*count)
                .map(|index| CardId(u32::try_from(index).unwrap_or(u32::MAX)))
                .collect(),
        );
    }
    hands
}

fn process(
    auction: &mut AuctionState,
    economies: &mut PlayerEconomies,
    hands: Option<&PlayerHands>,
    bids: Vec<AuctionBid>,
) -> AuctionNetworkOutbox {
    let config = GameConfig(shared::config::GameConfig::default());
    let mut outbox = AuctionNetworkOutbox::default();
    let mut gold_broadcasts = Vec::new();
    process_bid_batch(
        auction,
        economies,
        hands,
        &config,
        None,
        bids,
        &mut outbox,
        &mut gold_broadcasts,
    );
    outbox
}

#[test]
fn self_bid_rejected_with_already_leader_without_state_change() {
    let leader = player(1);
    let mut auction = live_auction(5, Some(leader), 8_000);
    let before = auction.clone();
    let mut economies = economies(&[(leader, 20, 5)]);

    let outbox = process(&mut auction, &mut economies, None, vec![bid(leader, 6)]);

    assert_eq!(auction.phase, before.phase);
    assert_eq!(auction.current_price, before.current_price);
    assert_eq!(auction.current_leader, before.current_leader);
    assert_eq!(auction.timer_remaining_ms, before.timer_remaining_ms);
    assert_eq!(economies.0[&leader].reserved_gold, 5);
    assert_eq!(outbox.rejected().len(), 1);
    assert_eq!(
        outbox.rejected()[0].message.reason,
        BidRejectedReason::AlreadyLeader
    );
    assert!(outbox.accepted().is_empty());
}

#[test]
fn full_hand_bid_rejected_without_state_change() {
    let bidder = player(2);
    let mut auction = live_auction(3, None, 10_000);
    let mut economies = economies(&[(bidder, 20, 0)]);
    let hands = hands(&[(bidder, 10)]);

    let outbox = process(
        &mut auction,
        &mut economies,
        Some(&hands),
        vec![bid(bidder, 4)],
    );

    assert_eq!(auction.current_price, 3);
    assert_eq!(auction.current_leader, None);
    assert_eq!(economies.0[&bidder].reserved_gold, 0);
    assert_eq!(
        outbox.rejected()[0].message.reason,
        BidRejectedReason::HandFull
    );
}

#[test]
fn insufficient_unreserved_gold_rejected_without_state_change() {
    let bidder = player(3);
    let mut auction = live_auction(5, None, 8_000);
    let mut economies = economies(&[(bidder, 8, 5)]);

    let outbox = process(&mut auction, &mut economies, None, vec![bid(bidder, 6)]);

    assert_eq!(auction.current_price, 5);
    assert_eq!(auction.current_leader, None);
    assert_eq!(economies.0[&bidder].gold, 8);
    assert_eq!(economies.0[&bidder].reserved_gold, 5);
    assert_eq!(
        outbox.rejected()[0].message.reason,
        BidRejectedReason::InsufficientGold
    );
}

#[test]
fn exact_free_gold_is_not_rejected_for_insufficient_gold() {
    let bidder = player(1);
    let mut auction = live_auction(5, None, 8_000);
    let mut economies = economies(&[(bidder, 6, 0)]);

    let outbox = process(&mut auction, &mut economies, None, vec![bid(bidder, 6)]);

    assert!(outbox.rejected().is_empty());
    assert_eq!(outbox.accepted().len(), 1);
    assert_eq!(auction.current_leader, Some(bidder));
    assert_eq!(economies.0[&bidder].reserved_gold, 6);
}

#[test]
fn at_price_and_below_price_rejected_as_amount_too_low() {
    for amount in [7, 6] {
        let bidder = player(1);
        let mut auction = live_auction(7, None, 8_000);
        let mut economies = economies(&[(bidder, 20, 0)]);

        let outbox = process(
            &mut auction,
            &mut economies,
            None,
            vec![bid(bidder, amount)],
        );

        assert_eq!(auction.current_price, 7);
        assert_eq!(auction.current_leader, None);
        assert_eq!(
            outbox.rejected()[0].message.reason,
            BidRejectedReason::AmountTooLow
        );
    }
}

#[test]
fn live_bidding_with_zero_timer_rejects_as_auction_expired() {
    let bidder = player(1);
    let mut auction = live_auction(5, None, 0);
    let mut economies = economies(&[(bidder, 20, 0)]);

    let outbox = process(&mut auction, &mut economies, None, vec![bid(bidder, 6)]);

    assert_eq!(auction.phase, AuctionPhase::LiveBidding);
    assert_eq!(auction.current_price, 5);
    assert_eq!(auction.current_leader, None);
    assert_eq!(
        outbox.rejected()[0].message.reason,
        BidRejectedReason::AuctionExpired
    );
}

#[test]
fn idle_bid_is_silently_discarded() {
    let bidder = player(1);
    let mut auction = AuctionState::default();
    let mut economies = economies(&[(bidder, 20, 0)]);

    let outbox = process(&mut auction, &mut economies, None, vec![bid(bidder, 99)]);

    assert_eq!(auction.phase, AuctionPhase::Idle);
    assert_eq!(auction.current_price, 0);
    assert_eq!(auction.current_leader, None);
    assert!(outbox.rejected().is_empty());
    assert!(outbox.accepted().is_empty());
}

#[test]
fn same_tick_duplicate_amount_accepts_first_then_rejects_second() {
    let first = player(1);
    let second = player(2);
    let mut auction = live_auction(5, None, 10_000);
    let mut economies = economies(&[(first, 20, 0), (second, 20, 0)]);

    let outbox = process(
        &mut auction,
        &mut economies,
        None,
        vec![bid(first, 6), bid(second, 6)],
    );

    assert_eq!(auction.current_price, 6);
    assert_eq!(auction.current_leader, Some(first));
    assert_eq!(economies.0[&first].reserved_gold, 6);
    assert_eq!(economies.0[&second].reserved_gold, 0);
    assert_eq!(outbox.accepted().len(), 1);
    assert_eq!(outbox.accepted()[0].player_id, first);
    assert_eq!(outbox.rejected().len(), 1);
    assert_eq!(outbox.rejected()[0].player_id, second);
    assert_eq!(
        outbox.rejected()[0].message.reason,
        BidRejectedReason::AmountTooLow
    );
}

#[test]
fn same_tick_duplicate_amount_respects_arrival_order() {
    let first = player(1);
    let second = player(2);
    let mut auction = live_auction(5, None, 10_000);
    let mut economies = economies(&[(first, 20, 0), (second, 20, 0)]);

    let outbox = process(
        &mut auction,
        &mut economies,
        None,
        vec![bid(second, 6), bid(first, 6)],
    );

    assert_eq!(auction.current_leader, Some(second));
    assert_eq!(economies.0[&second].reserved_gold, 6);
    assert_eq!(economies.0[&first].reserved_gold, 0);
    assert_eq!(outbox.accepted()[0].player_id, second);
    assert_eq!(outbox.rejected()[0].player_id, first);
}
