use server::core::economy::{PlayerEconomies, PlayerEconomy, S2CGoldBroadcast};
use server::core::rsm::AuctionSettled;
use server::feature::acquisition::PlayerHands;
use server::feature::auction::{
    settle_expired_auction, AuctionNetworkOutbox, AuctionPhase, AuctionState,
};
use shared::card::CardId;
use shared::protocol::{CardSource, S2CAuctionSettled};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
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

fn live_auction(
    card_id: CardId,
    current_price: u32,
    current_leader: Option<PlayerId>,
) -> AuctionState {
    AuctionState {
        phase: AuctionPhase::LiveBidding,
        card_id: Some(card_id),
        starting_price: 3,
        current_price,
        current_leader,
        timer_remaining_ms: 0,
    }
}

fn settle(
    auction: &mut AuctionState,
    economies: &mut PlayerEconomies,
    hands: Option<&mut PlayerHands>,
) -> (
    Option<AuctionSettled>,
    AuctionNetworkOutbox,
    Vec<S2CGoldBroadcast>,
) {
    let mut outbox = AuctionNetworkOutbox::default();
    let mut gold_broadcasts = Vec::new();
    let settled = settle_expired_auction(
        auction,
        economies,
        hands,
        None,
        &mut outbox,
        &mut gold_broadcasts,
    );
    (settled, outbox, gold_broadcasts)
}

#[test]
fn test_winner_with_hand_room_spends_gold_adds_card_and_emits_settlement() {
    let winner = player(1);
    let card_id = CardId(4);
    let mut auction = live_auction(card_id, 7, Some(winner));
    let mut economies = economies(&[(winner, 10, 7)]);
    let mut hands = hands(&[(winner, 3)]);

    let (settled, outbox, gold_broadcasts) = settle(&mut auction, &mut economies, Some(&mut hands));

    let economy = economies.0.get(&winner).expect("winner economy exists");
    assert_eq!(economy.gold, 3);
    assert_eq!(economy.reserved_gold, 0);
    assert_eq!(hands.hand_len(winner), 4);
    assert!(hands
        .hands
        .get(&winner)
        .expect("winner hand exists")
        .contains(&card_id));

    assert_eq!(outbox.card_acquired().len(), 1);
    assert_eq!(outbox.card_acquired()[0].player_id, winner);
    assert_eq!(outbox.card_acquired()[0].message.card_id, card_id);
    assert_eq!(
        outbox.card_acquired()[0].message.source,
        CardSource::AuctionWon
    );
    assert_eq!(
        outbox.settled()[0].message,
        S2CAuctionSettled {
            winner: Some(winner),
            amount: 7,
        }
    );
    let settled = settled.expect("winner settlement should emit internal message");
    assert_eq!(settled.winner, Some(winner));
    assert_eq!(settled.final_price, 7);
    assert_eq!(settled.card_id, card_id);
    assert_eq!(
        gold_broadcasts,
        vec![S2CGoldBroadcast {
            player_id: winner,
            gold: 3,
            reserved_gold: 0,
        }]
    );
    assert_eq!(auction.phase, AuctionPhase::Idle);
}

#[test]
fn test_winner_with_full_hand_spends_gold_discards_card_and_settles() {
    let winner = player(1);
    let card_id = CardId(4);
    let mut auction = live_auction(card_id, 7, Some(winner));
    let mut economies = economies(&[(winner, 10, 7)]);
    let mut hands = hands(&[(winner, 10)]);

    let (settled, outbox, _gold_broadcasts) =
        settle(&mut auction, &mut economies, Some(&mut hands));

    let economy = economies.0.get(&winner).expect("winner economy exists");
    assert_eq!(economy.gold, 3);
    assert_eq!(economy.reserved_gold, 0);
    assert_eq!(hands.hand_len(winner), 10);
    assert!(outbox.card_acquired().is_empty());
    assert_eq!(
        outbox.settled()[0].message,
        S2CAuctionSettled {
            winner: Some(winner),
            amount: 7,
        }
    );
    let settled = settled.expect("full-hand settlement should emit internal message");
    assert_eq!(settled.winner, Some(winner));
    assert_eq!(settled.final_price, 7);
    assert_eq!(settled.card_id, card_id);
    assert_eq!(auction.phase, AuctionPhase::Idle);
}

#[test]
fn test_no_bid_resolution_leaves_gold_unchanged_and_emits_none_settlement() {
    let first = player(1);
    let second = player(2);
    let card_id = CardId(2);
    let mut auction = live_auction(card_id, 3, None);
    let mut economies = economies(&[(first, 8, 0), (second, 12, 0)]);

    let (settled, outbox, gold_broadcasts) = settle(&mut auction, &mut economies, None);

    assert_eq!(economies.0.get(&first).expect("first exists").gold, 8);
    assert_eq!(economies.0.get(&second).expect("second exists").gold, 12);
    assert!(economies
        .0
        .values()
        .all(|economy| economy.reserved_gold == 0));
    assert!(outbox.card_acquired().is_empty());
    assert_eq!(
        outbox.settled()[0].message,
        S2CAuctionSettled {
            winner: None,
            amount: 0,
        }
    );
    let settled = settled.expect("no-bid settlement should emit internal message");
    assert_eq!(settled.winner, None);
    assert_eq!(settled.final_price, 0);
    assert_eq!(settled.card_id, card_id);
    assert!(gold_broadcasts.is_empty());
    assert_eq!(auction.phase, AuctionPhase::Idle);
}
