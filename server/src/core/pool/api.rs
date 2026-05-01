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

// Scaffold API consumed by downstream stories.
#![allow(dead_code)]

use std::collections::HashMap;

use shared::card::{CardCatalog, CardId, ClassId, Rarity, EPIC_POOL_COPIES, LEGENDARY_POOL_COPIES};
use shared::config::GameConfig;

use crate::core::pool::state::{DistributeError, PlayerPool, PoolFilter};
use crate::foundation::rng::ServerRng;

/// Formula 2 weight output for one eligible draw type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ComputedWeight {
    pub card_id: CardId,
    pub raw_weight: f32,
    pub normalized_weight: f32,
}

/// Atomic shop refresh: draws up to `slot_count` cards and distributes each
/// successful draw before returning.
///
/// The returned Vec is compact. Its length may be below `slot_count` when the
/// pool is partially exhausted. This pure helper does not perform the future
/// Card Acquisition class/neutral split roll; the subscriber story owns that
/// system-level policy.
pub fn refresh_shop(
    pool: &mut PlayerPool,
    catalog: &CardCatalog,
    _family_index: &HashMap<String, Vec<CardId>>,
    rng: &mut ServerRng,
    config: &GameConfig,
    slot_count: usize,
) -> Vec<CardId> {
    let mut drawn: Vec<CardId> = Vec::with_capacity(slot_count);

    for slot_index in 0..slot_count {
        let mut eligible: Vec<CardId> = pool
            .copies_remaining
            .iter()
            .filter_map(|(id, remaining)| {
                if *remaining > 0 && !drawn.contains(id) && catalog.contains_key(id) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();
        eligible.sort_by_key(|id| id.0);

        if eligible.is_empty() {
            break;
        }

        let weights = compute_weights(&eligible, pool, config);
        let seed = rng.draw_shop_slot(0, slot_index as u8);
        let normalized_weights: Vec<f32> = weights
            .iter()
            .map(|weight| weight.normalized_weight)
            .collect();
        let Some(index) = weighted_cdf_select(&normalized_weights, seed) else {
            break;
        };
        let Some(card_id) = eligible.get(index).copied() else {
            break;
        };

        if pool.distribute(card_id).is_err() {
            break;
        }
        drawn.push(card_id);
    }

    drawn
}

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
            Rarity::Common => config.common_pool_copies,
            Rarity::Uncommon => config.uncommon_pool_copies,
            Rarity::Rare => config.rare_pool_copies,
            Rarity::Epic => EPIC_POOL_COPIES,
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
        let initial = self.initial_count.get(&card_id).copied().unwrap_or(0);
        let remaining = self.copies_remaining.get(&card_id).copied().unwrap_or(0);
        initial.saturating_sub(remaining)
    }

    // ─── Draw functions ─────────────────────────────────────────────────────

    /// Draw distinct card IDs for the initial draft offering.
    ///
    /// Eligible set is catalog-based: cards from `class` plus all Neutral cards.
    /// This function never calls `distribute()`, so copy counts are unchanged.
    pub fn draw_initial_draft(
        &self,
        catalog: &CardCatalog,
        class: ClassId,
        count: u8,
        seed: u64,
    ) -> Vec<CardId> {
        let mut eligible: Vec<CardId> = catalog
            .iter()
            .filter(|(_, card)| card.class == class || card.class == ClassId::Neutral)
            .map(|(id, _)| *id)
            .collect();
        eligible.sort_by_key(|id| id.0);

        use rand::seq::SliceRandom;
        use rand::SeedableRng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        eligible.shuffle(&mut rng);
        eligible.into_iter().take(count as usize).collect()
    }

    /// Weighted draw for a class shop slot.
    ///
    /// Eligible cards must match `player_class` and have at least one copy
    /// remaining. Returns `None` when the class subset is exhausted.
    pub fn draw_class_card(
        &self,
        catalog: &CardCatalog,
        player_class: ClassId,
        seed: u64,
        config: &GameConfig,
    ) -> Option<CardId> {
        let mut eligible: Vec<CardId> = catalog
            .iter()
            .filter(|(id, card)| card.class == player_class && self.copies_remaining(**id) > 0)
            .map(|(id, _)| *id)
            .collect();
        eligible.sort_by_key(|id| id.0);

        let weights = compute_weights(&eligible, self, config);
        let index = weighted_cdf_select(
            &weights
                .iter()
                .map(|w| w.normalized_weight)
                .collect::<Vec<_>>(),
            seed,
        )?;
        eligible.get(index).copied()
    }

    /// Weighted draw over eligible Neutral families.
    ///
    /// The weight unit is the family, with `total_acquired` aggregated across
    /// all cards in that family. Returns `None` when every neutral family is
    /// exhausted.
    pub fn draw_neutral_family(
        &self,
        catalog: &CardCatalog,
        family_index: &HashMap<String, Vec<CardId>>,
        seed: u64,
        config: &GameConfig,
    ) -> Option<String> {
        let mut eligible: Vec<(String, u32)> = family_index
            .iter()
            .filter_map(|(family, ids)| {
                let has_available_neutral = ids.iter().any(|id| {
                    catalog.get(id).is_some_and(|card| {
                        card.class == ClassId::Neutral && self.copies_remaining(*id) > 0
                    })
                });
                if !has_available_neutral {
                    return None;
                }

                let acquired = ids.iter().map(|id| self.total_acquired(*id)).sum();
                Some((family.clone(), acquired))
            })
            .collect();
        eligible.sort_by(|a, b| a.0.cmp(&b.0));

        let raw_weights = compute_raw_weights(
            eligible.len(),
            eligible.iter().map(|(_, acquired)| *acquired),
            config,
        );
        let normalized = normalize_weights(&raw_weights);
        let index = weighted_cdf_select(&normalized, seed)?;
        eligible.get(index).map(|(family, _)| family.clone())
    }

    /// Uniformly draw one available card from a previously selected family.
    pub fn draw_family_card(
        &self,
        family: &str,
        catalog: &CardCatalog,
        family_index: &HashMap<String, Vec<CardId>>,
        seed: u64,
    ) -> Option<CardId> {
        let mut eligible: Vec<CardId> = family_index
            .get(family)?
            .iter()
            .copied()
            .filter(|id| {
                catalog.get(id).is_some_and(|card| {
                    card.class == ClassId::Neutral && self.copies_remaining(*id) > 0
                })
            })
            .collect();
        eligible.sort_by_key(|id| id.0);
        uniform_select(&eligible, seed)
    }

    /// Uniformly draw from the shared neutral auction pool.
    ///
    /// Eligible cards are Neutral Rare and Neutral Legendary only. The auction
    /// system owns when to call `distribute()` on the shared pool.
    pub fn draw_auction_card(
        auction_pool: &PlayerPool,
        catalog: &CardCatalog,
        seed: u64,
    ) -> Option<CardId> {
        let mut eligible: Vec<CardId> = catalog
            .iter()
            .filter(|(id, card)| {
                card.class == ClassId::Neutral
                    && matches!(card.rarity, Rarity::Rare | Rarity::Legendary)
                    && auction_pool.copies_remaining(**id) > 0
            })
            .map(|(id, _)| *id)
            .collect();
        eligible.sort_by_key(|id| id.0);
        uniform_select(&eligible, seed)
    }

    /// Uniformly draw an available card matching `filter`.
    ///
    /// This function is read-only and never calls `distribute()`.
    pub fn draw_random(
        &self,
        catalog: &CardCatalog,
        filter: &PoolFilter,
        seed: u64,
    ) -> Option<CardId> {
        let mut eligible: Vec<CardId> = self
            .copies_remaining
            .iter()
            .filter(|(_, remaining)| **remaining > 0)
            .filter(|(id, _)| matches_filter(id, catalog, filter))
            .map(|(id, _)| *id)
            .collect();
        eligible.sort_by_key(|id| id.0);
        uniform_select(&eligible, seed)
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
            None => Err(DistributeError::UnknownCard),
            Some(n) if *n == 0 => Err(DistributeError::Exhausted),
            Some(n) => {
                *n -= 1;
                Ok(())
            }
        }
    }
}

/// Compute Formula 2 raw and normalized weights for eligible card IDs.
pub(crate) fn compute_weights(
    eligible: &[CardId],
    pool: &PlayerPool,
    config: &GameConfig,
) -> Vec<ComputedWeight> {
    let raw_weights = compute_raw_weights(
        eligible.len(),
        eligible.iter().map(|id| pool.total_acquired(*id)),
        config,
    );
    let normalized = normalize_weights(&raw_weights);

    eligible
        .iter()
        .copied()
        .zip(raw_weights.into_iter().zip(normalized))
        .map(
            |(card_id, (raw_weight, normalized_weight))| ComputedWeight {
                card_id,
                raw_weight,
                normalized_weight,
            },
        )
        .collect()
}

fn compute_raw_weights(
    eligible_count: usize,
    acquired_counts: impl IntoIterator<Item = u32>,
    config: &GameConfig,
) -> Vec<f32> {
    if eligible_count == 0 {
        return Vec::new();
    }

    let base_weight = 1.0_f32 / eligible_count as f32;
    acquired_counts
        .into_iter()
        .map(|acquired| {
            (base_weight + config.shop_weight_per_card * acquired as f32)
                .clamp(0.0, config.shop_weight_cap)
        })
        .collect()
}

fn normalize_weights(raw_weights: &[f32]) -> Vec<f32> {
    let total: f32 = raw_weights.iter().sum();
    if total <= 0.0 {
        return vec![0.0; raw_weights.len()];
    }
    raw_weights.iter().map(|weight| weight / total).collect()
}

fn weighted_cdf_select(normalized_weights: &[f32], seed: u64) -> Option<usize> {
    if normalized_weights.is_empty() {
        return None;
    }

    use rand::Rng;
    use rand::SeedableRng;
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    let draw = rng.gen_range(0.0_f32..1.0_f32);
    let mut cumulative = 0.0_f32;

    for (index, weight) in normalized_weights.iter().enumerate() {
        cumulative += *weight;
        if draw <= cumulative {
            return Some(index);
        }
    }

    Some(normalized_weights.len() - 1)
}

fn uniform_select(eligible: &[CardId], seed: u64) -> Option<CardId> {
    if eligible.is_empty() {
        return None;
    }

    use rand::Rng;
    use rand::SeedableRng;
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    let index = rng.gen_range(0..eligible.len());
    eligible.get(index).copied()
}

fn matches_filter(id: &CardId, catalog: &CardCatalog, filter: &PoolFilter) -> bool {
    let Some(card) = catalog.get(id) else {
        return false;
    };

    if let Some(card_type) = filter.card_type {
        if card.card_type != card_type {
            return false;
        }
    }
    if let Some(class) = filter.class {
        if card.class != class {
            return false;
        }
    }
    if let Some(ref rarities) = filter.rarity {
        if !rarities.contains(&card.rarity) {
            return false;
        }
    }
    if let Some(max_cost) = filter.max_cost {
        if card.cost > max_cost {
            return false;
        }
    }
    true
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{compute_weights, refresh_shop};
    use crate::core::pool::state::{DistributeError, ManualRefreshCount, PlayerPool, PoolFilter};
    use crate::foundation::rng::ServerRng;
    use shared::card::{CardCatalog, CardData, CardId, CardType, ClassId, Rarity, UnitType};
    use shared::config::GameConfig;
    use std::collections::HashMap;

    // ─── Helpers ──────────────────────────────────────────────────────────────

    fn make_card(id: u32, rarity: Rarity, override_copies: Option<i32>) -> CardData {
        make_card_with(
            id,
            ClassId::Iop,
            None,
            rarity,
            CardType::Minion,
            override_copies,
        )
    }

    fn make_card_with(
        id: u32,
        class: ClassId,
        family: Option<&str>,
        rarity: Rarity,
        card_type: CardType,
        override_copies: Option<i32>,
    ) -> CardData {
        CardData {
            id: CardId(id),
            name_fr: format!("Carte {id}"),
            name_en: format!("Card {id}"),
            class,
            family: family.map(String::from),
            rarity,
            card_type,
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

    fn family_index(entries: Vec<(&str, Vec<CardId>)>) -> HashMap<String, Vec<CardId>> {
        entries
            .into_iter()
            .map(|(family, ids)| (family.to_string(), ids))
            .collect()
    }

    // ─── AC-1: initialize_catalog_length ──────────────────────────────────────

    #[test]
    fn test_pool_initialize_catalog_length() {
        let catalog = make_catalog(vec![
            make_card(1, Rarity::Common, None),
            make_card(2, Rarity::Uncommon, None),
            make_card(3, Rarity::Rare, None),
            make_card(4, Rarity::Epic, None),
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
            make_card(1, Rarity::Common, None),
            make_card(2, Rarity::Uncommon, None),
            make_card(3, Rarity::Rare, None),
            make_card(4, Rarity::Epic, None),
            make_card(5, Rarity::Legendary, None),
        ]);
        let pool = PlayerPool::initialize(&catalog, &default_config());
        assert_eq!(pool.copies_remaining(CardId(1)), 6); // Common
        assert_eq!(pool.copies_remaining(CardId(2)), 5); // Uncommon
        assert_eq!(pool.copies_remaining(CardId(3)), 4); // Rare
        assert_eq!(pool.copies_remaining(CardId(4)), 1); // Epic (const)
        assert_eq!(pool.copies_remaining(CardId(5)), 1); // Legendary (const)
        for id in [1u32, 2, 3, 4, 5] {
            assert!(
                pool.copies_remaining(CardId(id)) >= 1,
                "card {id} must have >= 1 copy"
            );
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
        assert_eq!(pool.copies_remaining(CardId(1)), 1); // N - K
        assert_eq!(pool.total_acquired(CardId(1)), 3); // K
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

    #[test]
    fn test_draw_class_card_all_exhausted_returns_none() {
        let catalog = make_catalog(vec![
            make_card_with(
                1,
                ClassId::Iop,
                None,
                Rarity::Common,
                CardType::Minion,
                Some(1),
            ),
            make_card_with(
                2,
                ClassId::Iop,
                None,
                Rarity::Common,
                CardType::Minion,
                Some(1),
            ),
            make_card_with(
                3,
                ClassId::Neutral,
                Some("Gobball"),
                Rarity::Common,
                CardType::Minion,
                Some(1),
            ),
        ]);
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        pool.distribute(CardId(1)).unwrap();
        pool.distribute(CardId(2)).unwrap();

        for seed in 0..100 {
            assert_eq!(
                pool.draw_class_card(&catalog, ClassId::Iop, seed, &default_config()),
                None
            );
        }
    }

    #[test]
    fn test_draw_class_card_returns_correct_class() {
        let catalog = make_catalog(
            (1..=5)
                .map(|id| {
                    make_card_with(
                        id,
                        ClassId::Iop,
                        None,
                        Rarity::Common,
                        CardType::Minion,
                        None,
                    )
                })
                .chain(std::iter::once(make_card_with(
                    100,
                    ClassId::Neutral,
                    Some("Gobball"),
                    Rarity::Common,
                    CardType::Minion,
                    None,
                )))
                .collect(),
        );
        let pool = PlayerPool::initialize(&catalog, &default_config());

        for seed in 0..20 {
            let id = pool
                .draw_class_card(&catalog, ClassId::Iop, seed, &default_config())
                .expect("eligible Iop card should be drawn");
            let card = catalog.get(&id).expect("drawn id must exist in catalog");
            assert_eq!(card.class, ClassId::Iop);
            assert!(pool.copies_remaining(id) >= 1);
        }
    }

    #[test]
    fn test_draw_neutral_family_then_draw_family_card() {
        let catalog = make_catalog(vec![
            make_card_with(
                1,
                ClassId::Neutral,
                Some("Gobball"),
                Rarity::Common,
                CardType::Minion,
                Some(2),
            ),
            make_card_with(
                2,
                ClassId::Neutral,
                Some("Gobball"),
                Rarity::Common,
                CardType::Minion,
                Some(2),
            ),
            make_card_with(
                3,
                ClassId::Neutral,
                Some("Gobball"),
                Rarity::Common,
                CardType::Minion,
                Some(2),
            ),
        ]);
        let pool = PlayerPool::initialize(&catalog, &default_config());
        let families = family_index(vec![("Gobball", vec![CardId(1), CardId(2), CardId(3)])]);

        let family = pool
            .draw_neutral_family(&catalog, &families, 1, &default_config())
            .expect("Gobball family should be eligible");
        assert_eq!(family, "Gobball");
        let card_id = pool
            .draw_family_card(&family, &catalog, &families, 2)
            .expect("Gobball card should be drawable");
        let card = catalog
            .get(&card_id)
            .expect("drawn id must exist in catalog");
        assert_eq!(card.class, ClassId::Neutral);
        assert_eq!(card.family.as_deref(), Some("Gobball"));
        assert!(pool.copies_remaining(card_id) >= 1);
    }

    #[test]
    fn test_normalized_weights_sum_to_one() {
        let catalog = make_catalog(
            (1..=25)
                .map(|id| {
                    make_card_with(
                        id,
                        ClassId::Iop,
                        None,
                        Rarity::Common,
                        CardType::Minion,
                        Some(4),
                    )
                })
                .collect(),
        );
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        for id in [CardId(1), CardId(2), CardId(3)] {
            pool.distribute(id).unwrap();
            pool.distribute(id).unwrap();
        }

        let eligible: Vec<CardId> = (1..=25).map(CardId).collect();
        let weights = compute_weights(&eligible, &pool, &default_config());
        let total: f32 = weights.iter().map(|weight| weight.normalized_weight).sum();
        assert!((total - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_draw_auction_card_exhausted_returns_none() {
        let catalog = make_catalog(vec![
            make_card_with(
                1,
                ClassId::Neutral,
                Some("Gobball"),
                Rarity::Rare,
                CardType::Minion,
                Some(1),
            ),
            make_card_with(
                2,
                ClassId::Neutral,
                Some("Tofu"),
                Rarity::Legendary,
                CardType::Minion,
                Some(1),
            ),
            make_card_with(
                3,
                ClassId::Neutral,
                Some("Piwi"),
                Rarity::Common,
                CardType::Minion,
                Some(1),
            ),
            make_card_with(
                4,
                ClassId::Neutral,
                Some("Wabbit"),
                Rarity::Epic,
                CardType::Minion,
                Some(1),
            ),
            make_card_with(
                5,
                ClassId::Iop,
                None,
                Rarity::Rare,
                CardType::Minion,
                Some(1),
            ),
        ]);
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        pool.distribute(CardId(1)).unwrap();
        pool.distribute(CardId(2)).unwrap();

        assert_eq!(PlayerPool::draw_auction_card(&pool, &catalog, 42), None);
    }

    #[test]
    fn test_draw_random_exhausted_filter_returns_none() {
        let catalog = make_catalog(vec![
            make_card_with(
                1,
                ClassId::Neutral,
                Some("Tofu"),
                Rarity::Legendary,
                CardType::Minion,
                Some(1),
            ),
            make_card_with(
                2,
                ClassId::Neutral,
                Some("Gobball"),
                Rarity::Rare,
                CardType::Minion,
                Some(1),
            ),
        ]);
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        pool.distribute(CardId(1)).unwrap();
        let before = pool.copies_remaining.clone();
        let filter = PoolFilter {
            class: Some(ClassId::Neutral),
            rarity: Some(vec![Rarity::Legendary]),
            ..Default::default()
        };

        assert_eq!(pool.draw_random(&catalog, &filter, 42), None);
        assert_eq!(pool.copies_remaining, before);
    }

    #[test]
    fn test_draw_initial_draft_9_distinct_ids() {
        let catalog = make_catalog(
            (1..=15)
                .map(|id| {
                    make_card_with(
                        id,
                        ClassId::Iop,
                        None,
                        Rarity::Common,
                        CardType::Minion,
                        None,
                    )
                })
                .chain((100..105).map(|id| {
                    make_card_with(
                        id,
                        ClassId::Neutral,
                        Some("Gobball"),
                        Rarity::Common,
                        CardType::Minion,
                        None,
                    )
                }))
                .collect(),
        );
        let pool = PlayerPool::initialize(&catalog, &default_config());
        let draft = pool.draw_initial_draft(&catalog, ClassId::Iop, 9, 42);
        let distinct: std::collections::HashSet<CardId> = draft.iter().copied().collect();

        assert_eq!(draft.len(), 9);
        assert_eq!(distinct.len(), 9);
    }

    #[test]
    fn test_draw_initial_draft_class_and_neutral_only() {
        let catalog = make_catalog(
            (1..=9)
                .map(|id| {
                    make_card_with(
                        id,
                        ClassId::Iop,
                        None,
                        Rarity::Common,
                        CardType::Minion,
                        None,
                    )
                })
                .chain((100..109).map(|id| {
                    make_card_with(
                        id,
                        ClassId::Neutral,
                        Some("Gobball"),
                        Rarity::Common,
                        CardType::Minion,
                        None,
                    )
                }))
                .chain((200..209).map(|id| {
                    make_card_with(
                        id,
                        ClassId::Cra,
                        None,
                        Rarity::Common,
                        CardType::Minion,
                        None,
                    )
                }))
                .collect(),
        );
        let pool = PlayerPool::initialize(&catalog, &default_config());
        let draft = pool.draw_initial_draft(&catalog, ClassId::Iop, 9, 42);

        assert_eq!(draft.len(), 9);
        for id in draft {
            let card = catalog.get(&id).expect("draft id must exist");
            assert!(matches!(card.class, ClassId::Iop | ClassId::Neutral));
        }
    }

    #[test]
    fn test_draw_initial_draft_does_not_call_distribute() {
        let catalog = make_catalog(
            (1..=10)
                .map(|id| {
                    make_card_with(
                        id,
                        ClassId::Iop,
                        None,
                        Rarity::Common,
                        CardType::Minion,
                        None,
                    )
                })
                .collect(),
        );
        let pool = PlayerPool::initialize(&catalog, &default_config());
        let before = pool.copies_remaining.clone();
        let draft = pool.draw_initial_draft(&catalog, ClassId::Iop, 9, 42);

        assert_eq!(draft.len(), 9);
        assert_eq!(pool.copies_remaining, before);
    }

    #[test]
    fn test_formula2_raw_weight_at_3_owned() {
        let catalog = make_catalog(
            (1..=25)
                .map(|id| {
                    make_card_with(
                        id,
                        ClassId::Iop,
                        None,
                        Rarity::Common,
                        CardType::Minion,
                        Some(10),
                    )
                })
                .collect(),
        );
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        for _ in 0..3 {
            pool.distribute(CardId(1)).unwrap();
        }

        let eligible: Vec<CardId> = (1..=25).map(CardId).collect();
        let weights = compute_weights(&eligible, &pool, &default_config());
        let target = weights
            .iter()
            .find(|weight| weight.card_id == CardId(1))
            .expect("target weight exists");

        assert!((target.raw_weight - 0.34).abs() < 1e-6);
        assert!((target.normalized_weight - 0.2615).abs() < 1e-4);
        for other in weights.iter().filter(|weight| weight.card_id != CardId(1)) {
            assert!(target.normalized_weight > other.normalized_weight);
        }
    }

    #[test]
    fn test_formula2_weight_clamped_at_cap() {
        let catalog = make_catalog(
            (1..=25)
                .map(|id| {
                    make_card_with(
                        id,
                        ClassId::Iop,
                        None,
                        Rarity::Common,
                        CardType::Minion,
                        Some(10),
                    )
                })
                .collect(),
        );
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        for _ in 0..7 {
            pool.distribute(CardId(1)).unwrap();
        }

        let eligible: Vec<CardId> = (1..=25).map(CardId).collect();
        let weights = compute_weights(&eligible, &pool, &default_config());
        let target = weights
            .iter()
            .find(|weight| weight.card_id == CardId(1))
            .expect("target weight exists");

        assert!((target.raw_weight - 0.65).abs() < 1e-6);
    }

    #[test]
    fn test_refresh_shop_3_slots_full() {
        let catalog = make_catalog(
            (1..=10)
                .map(|id| {
                    make_card_with(
                        id,
                        ClassId::Iop,
                        None,
                        Rarity::Common,
                        CardType::Minion,
                        Some(2),
                    )
                })
                .collect(),
        );
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        let before = pool.copies_remaining.clone();
        let mut rng = ServerRng::from_seed(3);
        let families = HashMap::new();

        let slots = refresh_shop(
            &mut pool,
            &catalog,
            &families,
            &mut rng,
            &default_config(),
            3,
        );
        let distinct: std::collections::HashSet<CardId> = slots.iter().copied().collect();

        assert_eq!(slots.len(), 3);
        assert_eq!(distinct.len(), 3);
        for id in slots {
            assert!(before.get(&id).copied().unwrap_or(0) >= 1);
            assert_eq!(pool.copies_remaining(id), before[&id] - 1);
        }
    }

    #[test]
    fn test_refresh_shop_9_slots_initial_draft() {
        let catalog = make_catalog(
            (1..=15)
                .map(|id| {
                    make_card_with(
                        id,
                        ClassId::Iop,
                        None,
                        Rarity::Common,
                        CardType::Minion,
                        Some(2),
                    )
                })
                .collect(),
        );
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        let before = pool.copies_remaining.clone();
        let mut rng = ServerRng::from_seed(9);
        let families = HashMap::new();

        let slots = refresh_shop(
            &mut pool,
            &catalog,
            &families,
            &mut rng,
            &default_config(),
            9,
        );
        let distinct: std::collections::HashSet<CardId> = slots.iter().copied().collect();

        assert_eq!(slots.len(), 9);
        assert_eq!(distinct.len(), 9);
        for id in slots {
            assert_eq!(pool.copies_remaining(id), before[&id] - 1);
        }
    }

    #[test]
    fn test_refresh_shop_partial_fill() {
        let catalog = make_catalog(
            (1..=5)
                .map(|id| {
                    make_card_with(
                        id,
                        ClassId::Iop,
                        None,
                        Rarity::Common,
                        CardType::Minion,
                        Some(1),
                    )
                })
                .collect(),
        );
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        for id in [CardId(3), CardId(4), CardId(5)] {
            pool.distribute(id).unwrap();
        }
        let mut rng = ServerRng::from_seed(2);
        let families = HashMap::new();

        let slots = refresh_shop(
            &mut pool,
            &catalog,
            &families,
            &mut rng,
            &default_config(),
            3,
        );

        assert_eq!(slots.len(), 2);
        for id in slots {
            assert_eq!(pool.copies_remaining(id), 0);
        }
    }

    #[test]
    fn test_refresh_shop_fully_exhausted_returns_empty() {
        let catalog = make_catalog(
            (1..=3)
                .map(|id| {
                    make_card_with(
                        id,
                        ClassId::Iop,
                        None,
                        Rarity::Common,
                        CardType::Minion,
                        Some(1),
                    )
                })
                .collect(),
        );
        let mut pool = PlayerPool::initialize(&catalog, &default_config());
        for id in [CardId(1), CardId(2), CardId(3)] {
            pool.distribute(id).unwrap();
        }
        let before = pool.copies_remaining.clone();
        let mut rng = ServerRng::from_seed(0);
        let families = HashMap::new();

        let slots = refresh_shop(
            &mut pool,
            &catalog,
            &families,
            &mut rng,
            &default_config(),
            3,
        );

        assert!(slots.is_empty());
        assert_eq!(pool.copies_remaining, before);
    }

    #[test]
    fn test_manual_refresh_count_reset_on_draft_entry() {
        let mut counts = ManualRefreshCount(HashMap::from([(shared::session::PlayerId(1), 3)]));

        counts.reset_for_player(shared::session::PlayerId(1));

        assert_eq!(counts.0[&shared::session::PlayerId(1)], 0);
    }

    #[test]
    fn test_manual_refresh_count_reset_inserts_missing_player() {
        let mut counts = ManualRefreshCount::default();

        counts.reset_for_player(shared::session::PlayerId(7));

        assert_eq!(counts.0[&shared::session::PlayerId(7)], 0);
    }

    #[test]
    fn test_manual_refresh_count_reset_only_targets_one_player() {
        let mut counts = ManualRefreshCount(HashMap::from([
            (shared::session::PlayerId(1), 3),
            (shared::session::PlayerId(2), 2),
        ]));

        counts.reset_for_player(shared::session::PlayerId(1));

        assert_eq!(counts.0[&shared::session::PlayerId(1)], 0);
        assert_eq!(counts.0[&shared::session::PlayerId(2)], 2);
    }
}
