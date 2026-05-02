use bevy::prelude::{Entity, World};
use shared::card::CardType;
use shared::session::PlayerId;
use tracing::warn;

use crate::core::board::{BoardPosition, TokenUnit, UnitCardRef, UnitOwner, UnitStats};
use crate::core::economy::{api, PlayerEconomies, SpendError};
use crate::feature::keyword::UnitKeywordState;
use crate::foundation::config::CardCatalog;

/// Xelorium's mana cost from the current card data contract.
pub const XELORIUM_MANA_COST: u32 = 4;

/// Movement rules supplied by the RESOLUTION caller for Rollback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RollbackMovementRules {
    pub direction: i16,
    pub cell_min: u8,
    pub cell_max: u8,
}

impl RollbackMovementRules {
    pub const fn new(direction: i16, cell_min: u8, cell_max: u8) -> Self {
        Self {
            direction,
            cell_min,
            cell_max,
        }
    }
}

/// Summary returned by `apply_rollback` for tests and future resolution logs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RollbackOutcome {
    pub reserve_spent: u32,
    pub units_moved: usize,
    pub stunned_units_skipped: usize,
}

/// CS-1: transfer all current mana into reserve, then zero current mana.
pub fn apply_gelure(economies: &mut PlayerEconomies, player_id: PlayerId) {
    let Some(economy) = economies.0.get_mut(&player_id) else {
        warn!(?player_id, "apply_gelure: player not in economies");
        return;
    };

    let amount = economy.current_mana;
    api::add_reserve(economy, amount);
    api::discard_current_mana(economy);
}

/// Deduct Xelorium's normal mana cost before the steal formula runs.
pub fn pay_xelorium_cost(
    economies: &mut PlayerEconomies,
    caster_id: PlayerId,
) -> Result<(), SpendError> {
    let Some(economy) = economies.0.get_mut(&caster_id) else {
        warn!(?caster_id, "pay_xelorium_cost: caster not in economies");
        return Err(SpendError::PlayerNotFound);
    };

    api::validate_spend(economy, XELORIUM_MANA_COST, false)?;
    api::apply_spend(economy, XELORIUM_MANA_COST, false);
    Ok(())
}

/// CS-2: move the opponent's current mana into the caster's reserve.
///
/// Call after Xelorium's own mana cost has been deducted.
pub fn apply_xelorium(economies: &mut PlayerEconomies, caster_id: PlayerId, opponent_id: PlayerId) {
    if caster_id == opponent_id {
        warn!(
            ?caster_id,
            "apply_xelorium: caster and opponent are identical"
        );
        return;
    }

    let stolen = if let Some(opponent) = economies.0.get_mut(&opponent_id) {
        let current_mana = opponent.current_mana;
        api::discard_current_mana(opponent);
        current_mana
    } else {
        warn!(?opponent_id, "apply_xelorium: opponent not in economies");
        0
    };

    let Some(caster) = economies.0.get_mut(&caster_id) else {
        warn!(?caster_id, "apply_xelorium: caster not in economies");
        return;
    };
    api::add_reserve(caster, stolen);
}

/// CS-3: consume all reserve and advance friendly, non-STUNned Minion units.
pub fn apply_rollback(
    economies: &mut PlayerEconomies,
    world: &mut World,
    player_id: PlayerId,
    rules: RollbackMovementRules,
) -> RollbackOutcome {
    let reserve_spent = {
        let Some(economy) = economies.0.get_mut(&player_id) else {
            warn!(?player_id, "apply_rollback: player not in economies");
            return RollbackOutcome::default();
        };

        let reserve = economy.reserve_mana;
        api::apply_spend(economy, reserve, true);
        reserve
    };

    let (planned_moves, stunned_units_skipped) =
        collect_rollback_moves(world, player_id, reserve_spent, rules);

    let mut units_moved = 0;
    for (entity, new_cell) in planned_moves {
        let Some(mut position) = world.get_mut::<BoardPosition>(entity) else {
            continue;
        };
        if position.cell != new_cell {
            units_moved += 1;
        }
        position.cell = new_cell;
    }

    RollbackOutcome {
        reserve_spent,
        units_moved,
        stunned_units_skipped,
    }
}

fn collect_rollback_moves(
    world: &mut World,
    player_id: PlayerId,
    reserve_spent: u32,
    rules: RollbackMovementRules,
) -> (Vec<(Entity, u8)>, usize) {
    let mut query = world.query::<(
        Entity,
        &BoardPosition,
        &UnitOwner,
        &UnitStats,
        Option<&UnitKeywordState>,
        Option<&UnitCardRef>,
        Option<&TokenUnit>,
    )>();
    let catalog = world.get_resource::<CardCatalog>();
    let mut moves = Vec::new();
    let mut stunned_units_skipped = 0;

    for (entity, position, owner, _stats, keyword_state, card_ref, token) in query.iter(world) {
        if owner.0 != player_id || !is_rollback_minion(card_ref, token, catalog) {
            continue;
        }

        if keyword_state.is_some_and(|state| state.stun_active) {
            stunned_units_skipped += 1;
            continue;
        }

        moves.push((
            entity,
            rollback_destination(position.cell, reserve_spent, rules),
        ));
    }

    (moves, stunned_units_skipped)
}

fn is_rollback_minion(
    card_ref: Option<&UnitCardRef>,
    token: Option<&TokenUnit>,
    catalog: Option<&CardCatalog>,
) -> bool {
    if token.is_some() {
        return true;
    }

    let Some(card_ref) = card_ref else {
        return true;
    };
    let Some(catalog) = catalog else {
        return false;
    };

    catalog
        .cards
        .get(&card_ref.0)
        .is_some_and(|card| card.card_type == CardType::Minion)
}

fn rollback_destination(current_cell: u8, reserve_spent: u32, rules: RollbackMovementRules) -> u8 {
    let reserve = i32::try_from(reserve_spent).unwrap_or(i32::MAX);
    let movement = i32::from(rules.direction).saturating_mul(reserve);
    let destination = i32::from(current_cell).saturating_add(movement);
    destination.clamp(i32::from(rules.cell_min), i32::from(rules.cell_max)) as u8
}
