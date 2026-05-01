// server/src/core/economy/api.rs -- Sole mutation API for PlayerEconomy.
//
// All economy field writes live here. Other systems call these functions rather
// than assigning currency fields directly.

// Scaffold API consumed by downstream stories.
#![allow(dead_code)]

use shared::config::GameConfig;

use crate::core::economy::state::{PlayerEconomy, SpendError};

/// Validate that `economy` can pay `cost`.
///
/// Reserve-only costs ignore `current_mana`; normal costs auto-split current
/// first, then reserve.
pub fn validate_spend(
    economy: &PlayerEconomy,
    cost: u32,
    from_reserve_only: bool,
) -> Result<(), SpendError> {
    if from_reserve_only {
        if economy.reserve_mana >= cost {
            Ok(())
        } else {
            Err(SpendError::InsufficientFunds)
        }
    } else if total_effective_mana(economy) >= cost {
        Ok(())
    } else {
        Err(SpendError::InsufficientFunds)
    }
}

/// Apply a previously validated mana spend.
///
/// Normal spends draw from `current_mana` first and overflow into reserve.
/// Reserve-only spends deduct from reserve and leave current mana untouched.
pub fn apply_spend(economy: &mut PlayerEconomy, cost: u32, from_reserve_only: bool) {
    if from_reserve_only {
        economy.reserve_mana = economy.reserve_mana.saturating_sub(cost);
        return;
    }

    let from_current = cost.min(economy.current_mana);
    let from_reserve = cost.saturating_sub(from_current);
    economy.current_mana = economy.current_mana.saturating_sub(from_current);
    economy.reserve_mana = economy.reserve_mana.saturating_sub(from_reserve);
}

/// Add persistent gold to the economy.
pub fn apply_gold_award(economy: &mut PlayerEconomy, amount: u32) {
    economy.gold = economy.gold.saturating_add(amount);
}

/// Apply the per-DRAFT mana ramp.
pub fn apply_mana_ramp(economy: &mut PlayerEconomy, round: u32) {
    economy.current_mana = round.min(economy.mana_cap);
}

/// Add persistent reserve mana. Reserve has no cap.
pub fn add_reserve(economy: &mut PlayerEconomy, amount: u32) {
    economy.reserve_mana = economy.reserve_mana.saturating_add(amount);
}

/// Reserve gold for an auction bid.
///
/// The reservation succeeds only when unreserved gold can cover `amount`.
pub fn reserve_gold(economy: &mut PlayerEconomy, amount: u32) -> Result<(), SpendError> {
    if !can_afford_bid(economy, amount) {
        return Err(SpendError::InsufficientFunds);
    }

    economy.reserved_gold = economy.reserved_gold.saturating_add(amount);
    debug_assert!(
        economy.reserved_gold <= economy.gold,
        "reserved_gold must never exceed gold"
    );
    Ok(())
}

/// Release an auction gold reservation, clamping at zero.
pub fn release_gold_reservation(economy: &mut PlayerEconomy, amount: u32) {
    economy.reserved_gold = economy.reserved_gold.saturating_sub(amount);
}

/// Validate an auction bid against hand capacity and unreserved gold.
pub fn validate_auction_bid(
    economy: &PlayerEconomy,
    bid_amount: u32,
    hand_size: u32,
) -> Result<(), SpendError> {
    if hand_size >= 10 {
        return Err(SpendError::HandFull);
    }

    if !can_afford_bid(economy, bid_amount) {
        return Err(SpendError::InsufficientFunds);
    }

    Ok(())
}

/// Discard all current-round mana.
pub fn discard_current_mana(economy: &mut PlayerEconomy) {
    economy.current_mana = 0;
}

/// Increase `mana_cap` by one, capped by `GameConfig`.
pub fn increment_mana_cap(economy: &mut PlayerEconomy, config: &GameConfig) {
    if economy.mana_cap < config.mana_cap_max {
        economy.mana_cap = economy.mana_cap.saturating_add(1).min(config.mana_cap_max);
    }
}

/// Return whether unreserved gold can cover an auction bid.
pub fn can_afford_bid(economy: &PlayerEconomy, amount: u32) -> bool {
    economy.gold.saturating_sub(economy.reserved_gold) >= amount
}

/// Return whether unreserved gold can cover a shop purchase.
pub fn can_afford_shop(economy: &PlayerEconomy, cost: u32) -> bool {
    economy.gold.saturating_sub(economy.reserved_gold) >= cost
}

/// Spend unreserved persistent gold for a shop purchase or manual refresh.
pub fn spend_gold(economy: &mut PlayerEconomy, cost: u32) -> Result<(), SpendError> {
    if !can_afford_shop(economy, cost) {
        return Err(SpendError::InsufficientFunds);
    }

    economy.gold = economy.gold.saturating_sub(cost);
    debug_assert!(
        economy.reserved_gold <= economy.gold,
        "reserved_gold must never exceed gold after shop spend"
    );
    Ok(())
}

/// Refund persistent gold after an atomic purchase rollback.
pub fn refund_gold(economy: &mut PlayerEconomy, amount: u32) {
    economy.gold = economy.gold.saturating_add(amount);
}

/// Total mana available for normal card costs.
pub fn total_effective_mana(economy: &PlayerEconomy) -> u32 {
    economy.current_mana.saturating_add(economy.reserve_mana)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn economy(
        gold: u32,
        current_mana: u32,
        reserve_mana: u32,
        mana_cap: u32,
        reserved_gold: u32,
    ) -> PlayerEconomy {
        PlayerEconomy {
            gold,
            current_mana,
            reserve_mana,
            mana_cap,
            reserved_gold,
        }
    }

    fn config_with_mana_cap_max(mana_cap_max: u32) -> GameConfig {
        GameConfig {
            mana_cap_max,
            ..GameConfig::default()
        }
    }

    #[test]
    fn test_auto_split_draws_current_first_then_reserve() {
        let mut econ = economy(0, 2, 3, 10, 0);

        assert_eq!(validate_spend(&econ, 4, false), Ok(()));
        apply_spend(&mut econ, 4, false);

        assert_eq!(econ.current_mana, 0);
        assert_eq!(econ.reserve_mana, 1);
    }

    #[test]
    fn test_auto_split_with_no_current_draws_from_reserve() {
        let mut econ = economy(0, 0, 5, 10, 0);

        assert_eq!(validate_spend(&econ, 3, false), Ok(()));
        apply_spend(&mut econ, 3, false);

        assert_eq!(econ.current_mana, 0);
        assert_eq!(econ.reserve_mana, 2);
    }

    #[test]
    fn test_auto_split_exact_current_cost_leaves_reserve_untouched() {
        let mut econ = economy(0, 4, 2, 10, 0);

        assert_eq!(validate_spend(&econ, 4, false), Ok(()));
        apply_spend(&mut econ, 4, false);

        assert_eq!(econ.current_mana, 0);
        assert_eq!(econ.reserve_mana, 2);
    }

    #[test]
    fn test_validate_spend_rejects_insufficient_total_without_mutation() {
        let econ = economy(0, 1, 1, 10, 0);

        assert_eq!(
            validate_spend(&econ, 3, false),
            Err(SpendError::InsufficientFunds)
        );
        assert_eq!(econ.current_mana, 1);
        assert_eq!(econ.reserve_mana, 1);
    }

    #[test]
    fn test_reserve_only_rejects_when_reserve_insufficient() {
        let econ = economy(0, 10, 3, 10, 0);

        assert_eq!(
            validate_spend(&econ, 4, true),
            Err(SpendError::InsufficientFunds)
        );
        assert_eq!(econ.current_mana, 10);
        assert_eq!(econ.reserve_mana, 3);
    }

    #[test]
    fn test_reserve_only_spend_deducts_reserve_and_leaves_current() {
        let mut econ = economy(0, 10, 5, 10, 0);

        assert_eq!(validate_spend(&econ, 4, true), Ok(()));
        apply_spend(&mut econ, 4, true);

        assert_eq!(econ.current_mana, 10);
        assert_eq!(econ.reserve_mana, 1);
    }

    #[test]
    fn test_add_reserve_then_discard_current_transfers_gelure_contract() {
        let mut econ = economy(0, 5, 2, 10, 0);

        let current = econ.current_mana;
        add_reserve(&mut econ, current);
        discard_current_mana(&mut econ);

        assert_eq!(econ.current_mana, 0);
        assert_eq!(econ.reserve_mana, 7);
    }

    #[test]
    fn test_zero_add_reserve_then_discard_current_is_legal_noop() {
        let mut econ = economy(0, 0, 2, 10, 0);

        add_reserve(&mut econ, 0);
        discard_current_mana(&mut econ);

        assert_eq!(econ.current_mana, 0);
        assert_eq!(econ.reserve_mana, 2);
    }

    #[test]
    fn test_increment_mana_cap_increases_by_one_below_ceiling() {
        let mut econ = economy(0, 0, 0, 10, 0);
        let config = config_with_mana_cap_max(12);

        increment_mana_cap(&mut econ, &config);

        assert_eq!(econ.mana_cap, 11);
    }

    #[test]
    fn test_increment_mana_cap_clamps_at_config_ceiling() {
        let mut econ = economy(0, 0, 0, 12, 0);
        let config = config_with_mana_cap_max(12);

        increment_mana_cap(&mut econ, &config);

        assert_eq!(econ.mana_cap, 12);
    }

    #[test]
    fn test_apply_gold_award_zero_is_noop() {
        let mut econ = economy(7, 0, 0, 10, 0);

        apply_gold_award(&mut econ, 0);

        assert_eq!(econ.gold, 7);
    }

    #[test]
    fn test_zero_cost_spend_is_noop() {
        let mut econ = economy(0, 3, 2, 10, 0);

        assert_eq!(validate_spend(&econ, 0, false), Ok(()));
        apply_spend(&mut econ, 0, false);

        assert_eq!(econ.current_mana, 3);
        assert_eq!(econ.reserve_mana, 2);
    }

    #[test]
    fn test_reserve_gold_rejects_when_unreserved_gold_insufficient() {
        let mut econ = economy(3, 0, 0, 10, 0);

        assert_eq!(
            reserve_gold(&mut econ, 5),
            Err(SpendError::InsufficientFunds)
        );

        assert_eq!(econ.gold, 3);
        assert_eq!(econ.reserved_gold, 0);
    }

    #[test]
    fn test_reserve_gold_respects_existing_reservation() {
        let mut econ = economy(8, 0, 0, 10, 5);

        assert!(!can_afford_bid(&econ, 4));
        assert!(!can_afford_shop(&econ, 4));
        assert_eq!(
            reserve_gold(&mut econ, 4),
            Err(SpendError::InsufficientFunds)
        );

        assert_eq!(econ.reserved_gold, 5);
    }

    #[test]
    fn test_reserve_gold_adds_to_reserved_amount_when_affordable() {
        let mut econ = economy(8, 0, 0, 10, 2);

        assert_eq!(reserve_gold(&mut econ, 4), Ok(()));

        assert_eq!(econ.reserved_gold, 6);
        assert_eq!(econ.gold, 8);
    }

    #[test]
    fn test_release_gold_reservation_clamps_to_zero() {
        let mut econ = economy(8, 0, 0, 10, 3);

        release_gold_reservation(&mut econ, 5);

        assert_eq!(econ.reserved_gold, 0);
    }

    #[test]
    fn test_spend_gold_deducts_unreserved_gold() {
        let mut econ = economy(8, 0, 0, 10, 2);

        assert_eq!(spend_gold(&mut econ, 4), Ok(()));

        assert_eq!(econ.gold, 4);
        assert_eq!(econ.reserved_gold, 2);
    }

    #[test]
    fn test_spend_gold_respects_reserved_gold() {
        let mut econ = economy(8, 0, 0, 10, 6);

        assert_eq!(spend_gold(&mut econ, 3), Err(SpendError::InsufficientFunds));

        assert_eq!(econ.gold, 8);
        assert_eq!(econ.reserved_gold, 6);
    }

    #[test]
    fn test_refund_gold_adds_back_to_persistent_gold() {
        let mut econ = economy(4, 0, 0, 10, 2);

        refund_gold(&mut econ, 3);

        assert_eq!(econ.gold, 7);
        assert_eq!(econ.reserved_gold, 2);
    }

    #[test]
    fn test_total_effective_mana_sums_current_and_reserve() {
        let econ = economy(0, 4, 9, 10, 0);

        assert_eq!(total_effective_mana(&econ), 13);
    }
}
