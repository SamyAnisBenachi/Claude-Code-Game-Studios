use server::core::economy::{
    can_afford_shop, release_gold_reservation, reserve_gold, spend_gold, validate_auction_bid,
    PlayerEconomy, SpendError,
};

fn economy(gold: u32, reserved_gold: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold,
        current_mana: 0,
        reserve_mana: 0,
        mana_cap: 10,
        reserved_gold,
    }
}

#[test]
fn bid_validation_rejects_insufficient_unreserved_gold_without_mutation() {
    let econ = economy(3, 0);

    assert_eq!(
        validate_auction_bid(&econ, 5, 0),
        Err(SpendError::InsufficientFunds)
    );
    assert_eq!(econ.gold, 3);
    assert_eq!(econ.reserved_gold, 0);
}

#[test]
fn bid_validation_rejects_full_hand_before_affordability() {
    let econ = economy(0, 0);

    assert_eq!(
        validate_auction_bid(&econ, 1, 10),
        Err(SpendError::HandFull)
    );
}

#[test]
fn shop_affordability_uses_unreserved_gold() {
    let econ = economy(8, 5);

    assert!(!can_afford_shop(&econ, 4));
    assert!(can_afford_shop(&econ, 3));
}

#[test]
fn reservation_lifecycle_releases_gold_for_shop_spend() {
    let mut econ = economy(10, 0);

    assert_eq!(reserve_gold(&mut econ, 7), Ok(()));
    assert_eq!(econ.reserved_gold, 7);
    assert!(!can_afford_shop(&econ, 4));

    release_gold_reservation(&mut econ, 7);

    assert_eq!(econ.reserved_gold, 0);
    assert!(can_afford_shop(&econ, 4));
}

#[test]
fn outbid_release_restores_full_gold_availability() {
    let mut econ = economy(8, 5);

    release_gold_reservation(&mut econ, 5);

    assert_eq!(econ.reserved_gold, 0);
    assert!(can_afford_shop(&econ, 8));
}

#[test]
fn auction_win_releases_reservation_then_spends_gold() {
    let mut econ = economy(8, 5);

    release_gold_reservation(&mut econ, 5);
    assert_eq!(spend_gold(&mut econ, 5), Ok(()));

    assert_eq!(econ.gold, 3);
    assert_eq!(econ.reserved_gold, 0);
}

#[test]
fn release_gold_reservation_saturates_when_amount_exceeds_reserved() {
    let mut econ = economy(8, 2);

    release_gold_reservation(&mut econ, 100);

    assert_eq!(econ.reserved_gold, 0);
}
