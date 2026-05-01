use bevy::prelude::*;
use shared::card::{CardData, Keyword, SimpleKeyword};
use shared::keyword::KeywordPayload;
use shared::protocol::EntityId;

use crate::core::board::{UnitCardRef, UnitStats};
use crate::feature::keyword::components::{EnteredPlayRound, UnitKeywordState};
use crate::feature::keyword::events::KeywordTriggered;
use crate::foundation::config::CardCatalog;

pub const STUN_DURATION_ROUNDS: u8 = 1;

/// RESOLUTION sub-steps where summoning sickness and STUN gate unit actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionSubStep {
    ChargeX,
    FirstStrike,
    StandardMovement,
    StandardAttack,
}

impl ActionSubStep {
    pub const fn number(self) -> u8 {
        match self {
            Self::ChargeX => 2,
            Self::FirstStrike => 3,
            Self::StandardMovement => 5,
            Self::StandardAttack => 6,
        }
    }
}

/// Damage application summary for one FIRST STRIKE pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DamageResult {
    pub attacker: Entity,
    pub target: Entity,
    pub attacker_hp_before: Option<u8>,
    pub target_hp_before: Option<u8>,
    pub damage_to_attacker: u8,
    pub damage_to_target: u8,
    pub attacker_hp_after: Option<u8>,
    pub target_hp_after: Option<u8>,
    pub target_retaliated_in_ss3: bool,
    pub applied: bool,
}

impl DamageResult {
    fn not_applied(attacker: Entity, target: Entity) -> Self {
        Self {
            attacker,
            target,
            attacker_hp_before: None,
            target_hp_before: None,
            damage_to_attacker: 0,
            damage_to_target: 0,
            attacker_hp_after: None,
            target_hp_after: None,
            target_retaliated_in_ss3: false,
            applied: false,
        }
    }

    pub fn attacker_defeated(&self) -> bool {
        self.attacker_hp_after == Some(0)
    }

    pub fn target_defeated(&self) -> bool {
        self.target_hp_after == Some(0)
    }
}

/// Applies FIRST STRIKE damage for one attacker-target pair.
///
/// If the target also has FIRST STRIKE and is not STUNned, both damage values
/// are computed from pre-hit stat snapshots before either HP value is mutated.
pub fn apply_first_strike(attacker: Entity, target: Entity, world: &mut World) -> DamageResult {
    if attacker == target {
        return DamageResult::not_applied(attacker, target);
    }

    let Some(attacker_stats) = world.get::<UnitStats>(attacker).copied() else {
        return DamageResult::not_applied(attacker, target);
    };
    let Some(target_stats) = world.get::<UnitStats>(target).copied() else {
        return DamageResult::not_applied(attacker, target);
    };

    let target_retaliated_in_ss3 =
        unit_has_card_simple_keyword(target, SimpleKeyword::FirstStrike, world)
            && !is_stunned(target, world);
    let damage_to_target = combat_damage(attacker, target, &attacker_stats, &target_stats, world);
    let damage_to_attacker = if target_retaliated_in_ss3 {
        combat_damage(target, attacker, &target_stats, &attacker_stats, world)
    } else {
        0
    };

    let attacker_hp_after = attacker_stats.hp.saturating_sub(damage_to_attacker);
    let target_hp_after = target_stats.hp.saturating_sub(damage_to_target);

    if let Some(mut stats) = world.get_mut::<UnitStats>(attacker) {
        stats.hp = attacker_hp_after;
    }
    if let Some(mut stats) = world.get_mut::<UnitStats>(target) {
        stats.hp = target_hp_after;
    }

    DamageResult {
        attacker,
        target,
        attacker_hp_before: Some(attacker_stats.hp),
        target_hp_before: Some(target_stats.hp),
        damage_to_attacker,
        damage_to_target,
        attacker_hp_after: Some(attacker_hp_after),
        target_hp_after: Some(target_hp_after),
        target_retaliated_in_ss3,
        applied: true,
    }
}

/// Applies one-RESOLUTION STUN state immediately and emits the protocol payload.
pub fn apply_stun(
    unit: Entity,
    source_unit_id: Option<EntityId>,
    sub_step: u8,
    world: &mut World,
) -> bool {
    let Some(mut kw_state) = world.get_mut::<UnitKeywordState>(unit) else {
        return false;
    };

    kw_state.stun_active = true;
    drop(kw_state);

    if let Some(mut messages) = world.get_resource_mut::<Messages<KeywordTriggered>>() {
        messages.write(KeywordTriggered {
            source_unit_id,
            sub_step,
            payload: KeywordPayload::StunApplied {
                duration_rounds: STUN_DURATION_ROUNDS,
            },
        });
    }

    true
}

/// Clears STUN at RESOLUTION end. This is called structurally, not time-based.
pub fn clear_stun_state_at_resolution_end(world: &mut World) -> usize {
    let mut query = world.query::<&mut UnitKeywordState>();
    let mut cleared = 0;

    for mut kw_state in query.iter_mut(world) {
        if kw_state.stun_active {
            kw_state.stun_active = false;
            cleared += 1;
        }
    }

    cleared
}

/// True when a unit may act in a gated RESOLUTION sub-step.
pub fn can_participate_in_sub_step(
    unit: Entity,
    sub_step: ActionSubStep,
    current_round: u32,
    world: &World,
) -> bool {
    match sub_step {
        ActionSubStep::ChargeX
        | ActionSubStep::FirstStrike
        | ActionSubStep::StandardMovement
        | ActionSubStep::StandardAttack => {}
    }

    if is_stunned(unit, world) {
        return false;
    }

    let Some(entered_round) = world.get::<EnteredPlayRound>(unit).map(|round| round.0) else {
        return true;
    };

    if entered_round > current_round {
        return false;
    }
    if entered_round < current_round {
        return true;
    }

    unit_has_active_simple_keyword(unit, SimpleKeyword::Haste, current_round, world)
}

/// True when a unit can execute a FIRST STRIKE attack in SS3.
pub fn can_execute_first_strike(unit: Entity, current_round: u32, world: &World) -> bool {
    can_participate_in_sub_step(unit, ActionSubStep::FirstStrike, current_round, world)
        && unit_has_active_simple_keyword(unit, SimpleKeyword::FirstStrike, current_round, world)
}

/// Returns the CHARGE X bonus movement for SS2 when the unit is eligible.
pub fn charge_x_cells_for_sub_step(unit: Entity, current_round: u32, world: &World) -> Option<u8> {
    if !can_participate_in_sub_step(unit, ActionSubStep::ChargeX, current_round, world)
        || is_silenced(unit, current_round, world)
    {
        return None;
    }

    card_data(unit, world)?.keywords.iter().find_map(|keyword| {
        if let Keyword::ChargeXMove { cells } = keyword {
            Some(*cells)
        } else {
            None
        }
    })
}

/// True when a unit can perform standard SS5 movement.
pub fn can_execute_standard_movement(unit: Entity, current_round: u32, world: &World) -> bool {
    can_participate_in_sub_step(unit, ActionSubStep::StandardMovement, current_round, world)
}

/// True when a unit can perform standard SS6 attacks.
pub fn can_execute_standard_attack(unit: Entity, current_round: u32, world: &World) -> bool {
    can_participate_in_sub_step(unit, ActionSubStep::StandardAttack, current_round, world)
}

/// Reads the immutable card definition to check for a no-parameter keyword.
pub fn unit_has_card_simple_keyword(unit: Entity, keyword: SimpleKeyword, world: &World) -> bool {
    card_data(unit, world).is_some_and(|card| {
        card.keywords
            .iter()
            .any(|candidate| matches!(candidate, Keyword::Simple(simple) if *simple == keyword))
    })
}

/// Checks a card keyword after SILENCE suppression for the current round.
pub fn unit_has_active_simple_keyword(
    unit: Entity,
    keyword: SimpleKeyword,
    current_round: u32,
    world: &World,
) -> bool {
    !is_silenced(unit, current_round, world) && unit_has_card_simple_keyword(unit, keyword, world)
}

pub fn check_shield_absorb(_unit: Entity, _sub_step: u8, _world: &World) -> bool {
    todo!()
}

pub fn apply_bodyguard_bond(_bodyguard: Entity, _protected: Entity, _world: &mut World) {
    todo!()
}

pub fn apply_repel(_target: Entity, _distance: u8, _world: &mut World) -> u8 {
    todo!()
}

pub fn apply_attract(_caster: Entity, _target: Entity, _distance: u8, _world: &mut World) -> u8 {
    todo!()
}

pub fn apply_teleport(_target: Entity, _lane: u8, _cell: u8, _world: &mut World) {
    todo!()
}

pub fn apply_change_lane(_target: Entity, _lane_delta: i8, _world: &mut World) {
    todo!()
}

pub fn check_irremovable(_target: Entity, _world: &World) -> bool {
    todo!()
}

pub fn check_counterattack_proximity(_world: &World, _defender: Entity, _attacker: Entity) -> bool {
    todo!()
}

pub fn apply_counterattack(
    _world: &mut World,
    _defender: Entity,
    _attacker: Entity,
    _sub_step: u8,
) {
    todo!()
}

fn card_data<'a>(unit: Entity, world: &'a World) -> Option<&'a CardData> {
    let card_id = world.get::<UnitCardRef>(unit)?.0;
    world.get_resource::<CardCatalog>()?.cards.get(&card_id)
}

fn combat_damage(
    attacker: Entity,
    _target: Entity,
    attacker_stats: &UnitStats,
    target_stats: &UnitStats,
    world: &World,
) -> u8 {
    let leader_bonus = world
        .get::<UnitKeywordState>(attacker)
        .map_or(0, |state| state.leader_bonus_atk);
    let attack = attacker_stats.atk.saturating_add(leader_bonus);

    attack.saturating_sub(target_stats.ar)
}

fn is_stunned(unit: Entity, world: &World) -> bool {
    world
        .get::<UnitKeywordState>(unit)
        .is_some_and(|state| state.stun_active)
}

fn is_silenced(unit: Entity, current_round: u32, world: &World) -> bool {
    world
        .get::<UnitKeywordState>(unit)
        .and_then(|state| state.silenced_until_round)
        .is_some_and(|until_round| current_round <= until_round)
}
