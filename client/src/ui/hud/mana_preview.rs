//! PROMPT 1336 — pure projection helper for the HUD mana preview during a
//! `Placement` drag (story `S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001`).
//!
//! Closes the AC5 / AC6 / AC8 / AC10 spec gaps left by the landed
//! PROMPT 1228 implementation (`compute_placement_drag_mana_preview` in
//! `client/src/ui/hud/mod.rs`):
//!
//! * AC6 — already-staged `PendingPlacements` spend is subtracted from the
//!   authoritative pool *before* the in-flight card's cost is applied, so the
//!   multi-card staging flow shows the correct projected remainder.
//! * AC8 — only `CardType::Minion` cards drive a numeric projection. Non-Minion
//!   types (Spell, Trap, Structure, Field, Order, DoubleFace) keep the HUD on
//!   its authoritative readout because the server-side mana split only fires
//!   for Minions (per GDD Rule 6).
//! * AC5 — when `cost > baseline_current + baseline_reserve` the outcome is
//!   flagged `overdrawn` so the HUD can surface a negative affordance signal
//!   on top of the clamped-to-zero pools.
//! * AC10 — the projection is exercised by a standalone unit test bin at
//!   `tests/unit/hand-ui/mana_preview_projection_test.rs`.
//!
//! Display-only. The function never mutates server state and is invoked from
//! a HUD sync system that runs after the authoritative state has been mirrored
//! into `ManaDisplayState` (ADR-002 binding preserved).

use shared::card::{CardData, CardType};

use crate::presentation::PlayerEconomyView;

/// Outcome of the mana preview projection.
///
/// `current` / `reserve` are always populated with the values the HUD should
/// paint for that frame:
///
/// * When `suppressed` is true (no drag, non-Minion card, or zero-cost card),
///   they hold the post-staged authoritative baseline. Callers should fall
///   through to the existing authoritative-paint branch instead of using these
///   values directly so the existing tween-target / format path keeps owning
///   the rendered text.
/// * When `overdrawn` is true, both fields are zero (the saturating spend
///   clamps both pools at the floor). Callers should surface the negative
///   affordance treatment in addition to painting the clamped numbers.
/// * Otherwise the fields hold the projected post-drop pools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManaPreviewOutcome {
    pub current: u32,
    pub reserve: u32,
    pub overdrawn: bool,
    pub suppressed: bool,
}

impl ManaPreviewOutcome {
    /// Construct a suppressed outcome with the supplied baseline values. The
    /// `current` / `reserve` fields are filled for callers that want to paint
    /// the post-staged baseline themselves; the HUD `sync_mana_text_system`
    /// callers in `client/src/ui/hud/mod.rs` instead fall through to the
    /// existing authoritative paint when `suppressed` is true.
    pub fn suppressed(current: u32, reserve: u32) -> Self {
        Self {
            current,
            reserve,
            overdrawn: false,
            suppressed: true,
        }
    }
}

/// Pure projection of the HUD mana preview during a placement drag.
///
/// `staged_current` / `staged_reserve` are the aggregated `current_mana_spend`
/// and `reserve_mana_spend` over `PendingPlacements::placements`. They are
/// subtracted from the authoritative pool with saturating arithmetic so that
/// the baseline never goes negative if the staged sums temporarily exceed the
/// authoritative pool (e.g. between an `S2CGoldUpdate` arrival and the next
/// staging recompute).
///
/// Spend split mirrors the canonical `apply_explicit_mana_split` fallback
/// (current first, then reserve), matching the default `reserve_amount = 0`
/// produced by `spawn_reserve_strip` for a freshly-staged card.
pub fn project_mana_preview(
    view: &PlayerEconomyView,
    drag_card_def: Option<&CardData>,
    staged_current: u32,
    staged_reserve: u32,
) -> ManaPreviewOutcome {
    let baseline_current = view.current_mana.saturating_sub(staged_current);
    let baseline_reserve = view.reserve_mana.saturating_sub(staged_reserve);

    let Some(card) = drag_card_def else {
        return ManaPreviewOutcome::suppressed(baseline_current, baseline_reserve);
    };

    if card.card_type != CardType::Minion {
        return ManaPreviewOutcome::suppressed(baseline_current, baseline_reserve);
    }

    let cost = card.cost;
    if cost == 0 {
        return ManaPreviewOutcome::suppressed(baseline_current, baseline_reserve);
    }

    let from_current = cost.min(baseline_current);
    let remaining_cost = cost - from_current;
    let from_reserve = remaining_cost.min(baseline_reserve);
    let overdrawn = cost > baseline_current + baseline_reserve;

    ManaPreviewOutcome {
        current: baseline_current - from_current,
        reserve: baseline_reserve - from_reserve,
        overdrawn,
        suppressed: false,
    }
}
