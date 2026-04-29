// server/src/core/pool/api.rs — Sole-mutation discipline for PlayerPool (ADR-006)
//
// This file contains ALL methods that write to `copies_remaining`.
// CI grep gate enforced at /story-done:
//   `grep -rE "copies_remaining\.(insert|remove|entry)" server/src/ | grep -v "core/pool/api.rs"`
//   must return zero matches.
//
// Story 001 scope: initialize(), distribute(), is_available(),
//   copies_remaining(), total_acquired().
// Draw functions (draw_class_card, draw_neutral_family, etc.) — Story 002.

use std::collections::HashMap;

use shared::card::{CardCatalog, CardId, Rarity, EPIC_POOL_COPIES, LEGENDARY_POOL_COPIES};
use shared::config::GameConfig;

use crate::core::pool::state::{DistributeError, PlayerPool};

impl PlayerPool {
    /// Build a new pool from the catalog.
    ///
    /// Copy count precedence (ADR-006, card-data-pool.md Rule 1):
    /// 1. `pool_copies_override: Some(n)` where `n >= 1` → use `n`
    /// 2. `pool_copies_override: Some(n)` where `n <= 0` → soft error: log + use rarity default
    /// 3. `pool_copies_override: None` → use rarity default from `GameConfig`
    ///
    /// Soft errors are logged via `tracing::error!` and do not abort startup.
    pub fn initialize(catalog: &CardCatalog, config: &GameConfig) -> Self {
        let mut copies_remaining = HashMap::new();
        let mut initial_count = HashMap::new();

        for (id, card) in catalog {
            let base = Self::rarity_copies(card.rarity, config);
            let copies = match card.pool_copies_override {
                Some(n) if n >= 1 => n as u32,
                Some(n) => {
                    tracing::error!(
                        card_id = ?id,
                        override_value = n,
                        rarity_default = base,
                        "pool_copies_override <= 0 — using rarity default"
                    );
                    base
                }
                None => base,
            };
            copies_remaining.insert(*id, copies);
            initial_count.insert(*id, copies);
        }

        Self {
            copies_remaining,
            initial_count,
            shop_slots: vec![None; 3],
        }
    }

    /// Rarity-default copy count from `GameConfig` or compile-time constant.
    ///
    /// Epic and Legendary use constants, not `GameConfig` fields — their scarcity
    /// is a load-bearing design pillar (card-data-pool.md Player Fantasy).
    fn rarity_copies(rarity: Rarity, config: &GameConfig) -> u32 {
        match rarity {
            Rarity::Common    => config.common_pool_copies,
            Rarity::Uncommon  => config.uncommon_pool_copies,
            Rarity::Rare      => config.rare_pool_copies,
            Rarity::Epic      => EPIC_POOL_COPIES,
            Rarity::Legendary => LEGENDARY_POOL_COPIES,
        }
    }

    // ─── Read-only queries ────────────────────────────────────────────────────

    /// O(1) availability check.
    ///
    /// Returns `false` for unknown `card_id` — callers may probe safely.
    pub fn is_available(&self, card_id: CardId) -> bool {
        self.copies_remaining.get(&card_id).copied().unwrap_or(0) > 0
    }

    /// Current available copy count (for UI display or eligibility checks).
    ///
    /// Returns 0 for unknown `card_id`.
    pub fn copies_remaining(&self, card_id: CardId) -> u32 {
        self.copies_remaining.get(&card_id).copied().unwrap_or(0)
    }

    /// How many copies have been distributed (purchased) since session start.
    ///
    /// Derived: `initial_count[id] - copies_remaining[id]`. No separate field.
    /// Returns 0 for unknown `card_id`.
    pub fn total_acquired(&self, card_id: CardId) -> u32 {
        let initial   = self.initial_count.get(&card_id).copied().unwrap_or(0);
        let remaining = self.copies_remaining.get(&card_id).copied().unwrap_or(0);
        initial.saturating_sub(remaining)
    }

    // ─── Mutation ─────────────────────────────────────────────────────────────

    /// Decrement `copies_remaining` by 1 — the SOLE pool mutation.
    ///
    /// Returns `Ok(())` on success.
    /// Returns `Err(Exhausted)` if `copies_remaining == 0`; pool unchanged.
    /// Returns `Err(UnknownCard)` if `card_id` is not in this pool.
    /// Never decrements below 0.
    pub fn distribute(&mut self, card_id: CardId) -> Result<(), DistributeError> {
        match self.copies_remaining.get_mut(&card_id) {
            None              => Err(DistributeError::UnknownCard),
            Some(n) if *n == 0 => Err(DistributeError::Exhausted),
            Some(n)           => { *n -= 1; Ok(()) }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use shared::card::{CardCatalog, CardData, CardId, CardType, ClassId, Rarity, UnitType};
    use shared::config::GameConfig;
    use crate::core::pool::state::{DistributeError, PlayerPool};

    // ─── Helpers ──────────────────────────────────────────────────────────────

    fn make_card(id: u32, rarity: Rarity, override_copies: Option<i32>) -> CardData {
        CardData {
            id: CardId(id),
            name_fr: format!("Carte {id}"),
            name_en: format!("Card {id}"),
            class: ClassId::Iop,
            family: None,
            rarity,
            card_type: CardType::Minion,
            unit_type: UnitType::Blade,
            cost: 1,
            atk: 1,
            hp: 1,
            mp: 1,
            ar: 0,
            keywords: vec![],
            effect_text: String::new(),
            art_id: String::new(),
            pool_copies_override: override_copies,
        }
    }

    fn make_catalog(cards: Vec<CardData>) -> CardCatalog {
        cards.into_iter().map(|c| (c.id, c)).collect()
    }

    fn default_config() -> GameConfig {
        GameConfig::default()
        // default: common=6, uncommon=5, rare=4, epic=1(const), legendary=1(const)
    }

    // ─── AC-1: initialize_catalog_length ──────────────────────────────────────

    #[test]
    fn test_pool_initialize_catalog_length() {
        let catalog = make_catalog(vec![
            make_card(1, Rarity::Common,    None),
            make_card(2, Rarity::Uncommon,  None),
            make_card(3, Rarity::Rare,      None),
            make_card(4, Rarity::Epic,      None),
            make_card(5, Rarity::Legendary, None),
        ]);
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert_eq!(pool.copies_remaining.len(), 5);
        assert_eq!(pool.initial_count.len(), 5);
    }

    #[test]
    fn test_pool_initialize_empty_catalog_no_panic() {
        let catalog: CardCatalog = HashMap::new();
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert_eq!(pool.copies_remaining.len(), 0);
        assert_eq!(pool.initial_count.len(), 0);
    }

    // ─── AC-2: all cards have copies >= 1 ────────────────────────────────────

    #[test]
    fn test_pool_initialize_all_cards_have_copies() {
        let catalog = make_catalog(vec![
            make_card(1, Rarity::Common,    None),
            make_card(2, Rarity::Uncommon,  None),
            make_card(3, Rarity::Rare,      None),
            make_card(4, Rarity::Epic,      None),
            make_card(5, Rarity::Legendary, None),
        ]);
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert_eq!(pool.copies_remaining(CardId(1)), 6); // Common
        assert_eq!(pool.copies_remaining(CardId(2)), 5); // Uncommon
        assert_eq!(pool.copies_remaining(CardId(3)), 4); // Rare
        assert_eq!(pool.copies_remaining(CardId(4)), 1); // Epic (const)
        assert_eq!(pool.copies_remaining(CardId(5)), 1); // Legendary (const)
        for id in [1u32, 2, 3, 4, 5] {
            assert!(pool.copies_remaining(CardId(id)) >= 1, "card {id} must have >= 1 copy");
        }
    }

    // ─── AC-3: soft error on override <= 0 ────────────────────────────────────

    #[test]
    fn test_pool_soft_error_override_zero_no_panic_uses_rarity_default() {
        // Must not panic; must fall back to rarity default (Rare = 4)
        let catalog = make_catalog(vec![make_card(1, Rarity::Rare, Some(0))]);
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert_eq!(pool.copies_remaining(CardId(1)), 4);
    }

    #[test]
    fn test_pool_soft_error_override_negative_no_panic_uses_rarity_default() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Rare, Some(-3))]);
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert_eq!(pool.copies_remaining(CardId(1)), 4);
    }

    #[test]
    fn test_pool_soft_error_override_i32_min_no_panic() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Common, Some(i32::MIN))]);
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert_eq!(pool.copies_remaining(CardId(1)), 6); // Common rarity default
    }

    // ─── AC-4: rare, no override → rarity default ─────────────────────────────

    #[test]
    fn test_pool_rare_no_override_gets_rarity_default() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Rare, None)]);
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert_eq!(pool.copies_remaining(CardId(1)), 4);
    }

    #[test]
    fn test_pool_rare_no_override_respects_config_value() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Rare, None)]);
        let mut config = default_config();
        config.rare_pool_copies = 1;
        let pool = PlayerPool::initialize(&catalog, &config);
        assert_eq!(pool.copies_remaining(CardId(1)), 1);
    }

    // ─── AC-5: distribute decrements correctly ────────────────────────────────

    #[test]
    fn test_pool_distribute_decrements_correctly() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Rare, None)]); // 4 copies
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        assert_eq!(pool.distribute(CardId(1)), Ok(()));
        assert_eq!(pool.copies_remaining(CardId(1)), 3);
    }

    #[test]
    fn test_pool_distribute_four_times_sequence() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Rare, None)]); // 4 copies
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        for expected_after in (0u32..4).rev() {
            assert_eq!(pool.distribute(CardId(1)), Ok(()));
            assert_eq!(pool.copies_remaining(CardId(1)), expected_after);
        }
    }

    // ─── AC-6: positive override applied ──────────────────────────────────────

    #[test]
    fn test_pool_positive_override_applied() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Rare, Some(2))]);
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert_eq!(pool.copies_remaining(CardId(1)), 2); // override, not rarity default
    }

    #[test]
    fn test_pool_positive_override_one_minimum() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Common, Some(1))]);
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert_eq!(pool.copies_remaining(CardId(1)), 1);
    }

    // ─── AC-7: distribute on exhausted card ──────────────────────────────────

    #[test]
    fn test_pool_distribute_exhausted_error() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Rare, Some(1))]);
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        pool.distribute(CardId(1)).unwrap(); // consume the only copy
        assert_eq!(pool.copies_remaining(CardId(1)), 0);
        let result = pool.distribute(CardId(1));
        assert_eq!(result, Err(DistributeError::Exhausted));
        assert_eq!(pool.copies_remaining(CardId(1)), 0); // no underflow
    }

    #[test]
    fn test_pool_distribute_exhausted_repeatedly_stays_exhausted() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Common, Some(1))]);
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        pool.distribute(CardId(1)).unwrap();
        assert_eq!(pool.distribute(CardId(1)), Err(DistributeError::Exhausted));
        assert_eq!(pool.distribute(CardId(1)), Err(DistributeError::Exhausted));
        assert_eq!(pool.copies_remaining(CardId(1)), 0);
    }

    // ─── AC-8: is_available false at zero ────────────────────────────────────

    #[test]
    fn test_pool_is_available_false_at_zero() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Rare, Some(1))]);
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        pool.distribute(CardId(1)).unwrap();
        assert!(!pool.is_available(CardId(1)));
    }

    #[test]
    fn test_pool_is_available_false_for_unknown_card() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Common, None)]);
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert!(!pool.is_available(CardId(999))); // not in pool — no panic
    }

    // ─── AC-9: is_available true above zero ──────────────────────────────────

    #[test]
    fn test_pool_is_available_true_above_zero() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Common, None)]); // 6 copies
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert!(pool.is_available(CardId(1)));
    }

    #[test]
    fn test_pool_is_available_true_at_one_copy_remaining() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Rare, Some(1))]);
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert!(pool.is_available(CardId(1)));
    }

    // ─── AC-10: initial_count immutable, total_acquired derived ───────────────

    #[test]
    fn test_pool_initial_count_immutable_total_acquired_correct() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Rare, None)]); // initial = 4
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        let initial = *pool.initial_count.get(&CardId(1)).unwrap();
        assert_eq!(initial, 4);

        pool.distribute(CardId(1)).unwrap();
        pool.distribute(CardId(1)).unwrap();
        pool.distribute(CardId(1)).unwrap(); // K = 3

        assert_eq!(*pool.initial_count.get(&CardId(1)).unwrap(), 4); // unchanged
        assert_eq!(pool.copies_remaining(CardId(1)), 1);              // N - K
        assert_eq!(pool.total_acquired(CardId(1)), 3);                // K
    }

    #[test]
    fn test_pool_total_acquired_zero_before_any_distribute() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Rare, None)]);
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert_eq!(pool.total_acquired(CardId(1)), 0);
    }

    #[test]
    fn test_pool_total_acquired_equals_n_when_all_distributed() {
        let catalog = make_catalog(vec![make_card(1, Rarity::Rare, Some(2))]);
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        pool.distribute(CardId(1)).unwrap();
        pool.distribute(CardId(1)).unwrap();
        assert_eq!(pool.total_acquired(CardId(1)), 2);
        assert_eq!(pool.copies_remaining(CardId(1)), 0);
    }
}
