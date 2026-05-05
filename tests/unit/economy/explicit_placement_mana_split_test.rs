use server::core::economy::{
    apply_explicit_mana_split, validate_explicit_mana_split, PlayerEconomy, SpendError,
};

fn economy(current_mana: u32, reserve_mana: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold: 0,
        current_mana,
        reserve_mana,
        mana_cap: 10,
        reserved_gold: 0,
    }
}

#[test]
fn test_explicit_split_validation_succeeds_for_exact_current_and_reserve_allocation() {
    let econ = economy(3, 2);

    assert_eq!(validate_explicit_mana_split(&econ, 5, 3, 2), Ok(()));
    assert_eq!(econ.current_mana, 3);
    assert_eq!(econ.reserve_mana, 2);
}

#[test]
fn test_explicit_split_rejects_current_overdraw_without_mutation() {
    let econ = economy(2, 5);

    assert_eq!(
        validate_explicit_mana_split(&econ, 5, 3, 2),
        Err(SpendError::InsufficientCurrentMana)
    );
    assert_eq!(econ.current_mana, 2);
    assert_eq!(econ.reserve_mana, 5);
}

#[test]
fn test_explicit_split_rejects_reserve_overdraw_without_mutation() {
    let econ = economy(5, 1);

    assert_eq!(
        validate_explicit_mana_split(&econ, 5, 3, 2),
        Err(SpendError::InsufficientReserveMana)
    );
    assert_eq!(econ.current_mana, 5);
    assert_eq!(econ.reserve_mana, 1);
}

#[test]
fn test_explicit_split_rejects_invalid_sum_before_affordability() {
    let econ = economy(0, 0);

    assert_eq!(
        validate_explicit_mana_split(&econ, 5, 4, 0),
        Err(SpendError::InvalidManaSplit)
    );
    assert_eq!(econ.current_mana, 0);
    assert_eq!(econ.reserve_mana, 0);
}

#[test]
fn test_explicit_split_application_deducts_exact_requested_amounts() {
    let mut econ = economy(3, 2);

    assert_eq!(validate_explicit_mana_split(&econ, 5, 3, 2), Ok(()));
    apply_explicit_mana_split(&mut econ, 3, 2);

    assert_eq!(econ.current_mana, 0);
    assert_eq!(econ.reserve_mana, 0);
}

#[test]
fn test_explicit_zero_cost_requires_zero_split() {
    let econ = economy(3, 2);

    assert_eq!(validate_explicit_mana_split(&econ, 0, 0, 0), Ok(()));
    assert_eq!(
        validate_explicit_mana_split(&econ, 0, 1, 0),
        Err(SpendError::InvalidManaSplit)
    );
}
