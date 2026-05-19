//! PROMPT 1336 — pure unit test for the mana-preview projection helper
//! (`client::ui::hud::project_mana_preview`). Closes AC10 of
//! `S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001` by covering the six cases
//! enumerated in the story spec:
//!
//! 1. zero-cost suppression
//! 2. current-only spend
//! 3. reserve spillover
//! 4. exact exhaustion
//! 5. overdrawn true (saturating clamp at zero)
//! 6. staged multi-card baseline (current + reserve sub-cases)
//!
//! The helper is independent of any ECS scheduling — these assertions run
//! against the projection arithmetic directly with no `App`, no `World`, and
//! no plugin wiring. They are the unit-test slice that backs the AC9
//! integration test in `tests/integration/hand-ui/mana_preview_during_drag_test.rs`.

use client::presentation::PlayerEconomyView;
use client::ui::hud::{project_mana_preview, ManaPreviewOutcome};
use shared::card::{CardData, CardId, CardType, ClassId, Keyword, Rarity, UnitType};

fn view(current_mana: u32, reserve_mana: u32, mana_cap: u8) -> PlayerEconomyView {
    PlayerEconomyView {
        gold: 0,
        current_mana,
        reserve_mana,
        mana_cap,
        initialized: true,
        last_update_source: None,
    }
}

fn minion(cost: u32) -> CardData {
    CardData {
        id: CardId(1),
        name_fr: "Aperçu".to_string(),
        name_en: "Preview".to_string(),
        class: ClassId::Iop,
        family: None,
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost,
        atk: 1,
        hp: 1,
        mp: 1,
        ar: 0,
        keywords: Vec::<Keyword>::new(),
        effect_text: String::new(),
        art_id: "unit_test".to_string(),
        pool_copies_override: None,
    }
}

fn spell(cost: u32) -> CardData {
    CardData {
        id: CardId(2),
        name_fr: "Sort".to_string(),
        name_en: "Spell".to_string(),
        class: ClassId::Cra,
        family: None,
        rarity: Rarity::Common,
        card_type: CardType::Spell,
        unit_type: UnitType::Arcane,
        cost,
        atk: 0,
        hp: 0,
        mp: 0,
        ar: 0,
        keywords: Vec::<Keyword>::new(),
        effect_text: String::new(),
        art_id: "unit_test".to_string(),
        pool_copies_override: None,
    }
}

/// AC10 (1) — Zero-cost Minion suppresses the projection. The HUD
/// callers fall through to the existing authoritative paint when
/// `suppressed: true`, so callers never accidentally subtract a
/// zero-cost card's cost from the readout.
#[test]
fn zero_cost_minion_suppresses_projection() {
    let outcome = project_mana_preview(&view(3, 0, 3), Some(&minion(0)), 0, 0);
    assert_eq!(
        outcome,
        ManaPreviewOutcome {
            current: 3,
            reserve: 0,
            overdrawn: false,
            suppressed: true,
        }
    );
}

/// AC10 (2) — Current-only spend leaves the reserve pool untouched.
#[test]
fn current_only_spend_leaves_reserve_unchanged() {
    let outcome = project_mana_preview(&view(5, 2, 5), Some(&minion(4)), 0, 0);
    assert_eq!(
        outcome,
        ManaPreviewOutcome {
            current: 1,
            reserve: 2,
            overdrawn: false,
            suppressed: false,
        }
    );
}

/// AC10 (3) — Reserve spillover when `cost > current_mana`.
#[test]
fn reserve_spillover_when_cost_exceeds_current() {
    let outcome = project_mana_preview(&view(1, 3, 3), Some(&minion(3)), 0, 0);
    assert_eq!(
        outcome,
        ManaPreviewOutcome {
            current: 0,
            reserve: 1,
            overdrawn: false,
            suppressed: false,
        }
    );
}

/// AC10 (4) — Exact exhaustion drains both pools to zero with
/// `overdrawn = false` because the cost equals the combined pool.
#[test]
fn exact_exhaustion_zeroes_both_pools_without_overdraw() {
    let outcome = project_mana_preview(&view(3, 4, 7), Some(&minion(7)), 0, 0);
    assert_eq!(
        outcome,
        ManaPreviewOutcome {
            current: 0,
            reserve: 0,
            overdrawn: false,
            suppressed: false,
        }
    );
}

/// AC10 (5) — `overdrawn = true` when `cost > baseline_current + baseline_reserve`.
/// Both pools clamp at zero via saturating arithmetic.
#[test]
fn overdrawn_flag_set_when_cost_exceeds_combined_pool() {
    let outcome = project_mana_preview(&view(1, 1, 2), Some(&minion(5)), 0, 0);
    assert_eq!(
        outcome,
        ManaPreviewOutcome {
            current: 0,
            reserve: 0,
            overdrawn: true,
            suppressed: false,
        }
    );
}

/// AC10 (6) — Staged multi-card baseline subtracts both `current_mana_spend`
/// and `reserve_mana_spend` from the authoritative pool BEFORE applying the
/// in-flight cost. Covers both sub-cases from the story:
/// * current-only staged baseline (current=6, staged_current=3, cost=2 → 1)
/// * reserve-staged baseline (current=0, reserve=5, staged_reserve=2, cost=2 → 0/1)
#[test]
fn staged_multi_card_baseline_subtracts_before_in_flight_cost() {
    let current_only = project_mana_preview(
        &view(6, 0, 6),
        Some(&minion(2)),
        /* staged_current */ 3,
        /* staged_reserve */ 0,
    );
    assert_eq!(
        current_only,
        ManaPreviewOutcome {
            current: 1,
            reserve: 0,
            overdrawn: false,
            suppressed: false,
        },
        "AC6 current-only branch: (6 - 3) - 2 = 1",
    );

    let reserve_staged = project_mana_preview(
        &view(0, 5, 5),
        Some(&minion(2)),
        /* staged_current */ 0,
        /* staged_reserve */ 2,
    );
    assert_eq!(
        reserve_staged,
        ManaPreviewOutcome {
            current: 0,
            reserve: 1,
            overdrawn: false,
            suppressed: false,
        },
        "AC6 reserve-staged branch: current = 0, reserve baseline (5 - 2) minus spillover (2) = 1",
    );
}

/// AC8 companion — non-Minion drag suppresses the projection regardless of
/// cost or pool sizes. The HUD continues to paint authoritative values.
#[test]
fn non_minion_card_suppresses_projection() {
    let outcome = project_mana_preview(&view(5, 2, 5), Some(&spell(3)), 0, 0);
    assert_eq!(
        outcome,
        ManaPreviewOutcome {
            current: 5,
            reserve: 2,
            overdrawn: false,
            suppressed: true,
        }
    );
}

/// Drag-end companion — when no card is being dragged, the helper reports the
/// post-staged baseline as `suppressed`, so the system falls through to the
/// authoritative paint path.
#[test]
fn no_drag_card_returns_suppressed_baseline() {
    let outcome = project_mana_preview(&view(4, 1, 5), None, 0, 0);
    assert_eq!(
        outcome,
        ManaPreviewOutcome {
            current: 4,
            reserve: 1,
            overdrawn: false,
            suppressed: true,
        }
    );
}

/// Saturating-baseline companion — staged sums that briefly exceed the
/// authoritative pool (e.g. between an `S2CGoldUpdate` arrival and the next
/// staging recompute) must NOT underflow the baseline.
#[test]
fn staged_baseline_saturating_subtraction_never_underflows() {
    let outcome = project_mana_preview(
        &view(2, 1, 5),
        Some(&minion(1)),
        /* staged_current */ 10,
        /* staged_reserve */ 10,
    );
    assert_eq!(
        outcome,
        ManaPreviewOutcome {
            current: 0,
            reserve: 0,
            overdrawn: true,
            suppressed: false,
        }
    );
}
