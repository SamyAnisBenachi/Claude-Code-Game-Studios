use std::collections::HashMap;

use bevy::ecs::entity::Entities;
use bevy::prelude::*;
use shared::card::{CardData, CardId, CardType, Keyword, SimpleKeyword};
use shared::keyword::{InjuredGrantedKeyword, KeywordPayload};
use shared::session::PlayerId;

use crate::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use crate::core::session::SessionConfig;
use crate::feature::keyword::components::UnitKeywordState;
use crate::feature::keyword::events::KeywordTriggered;
use crate::foundation::config::CardCatalog;

const DEFAULT_LEADER_ATK_BONUS: u8 = 1;

#[derive(Clone, Debug)]
struct UnitInfo {
    entity: Entity,
    owner: PlayerId,
    card_id: CardId,
    hp: u8,
    silenced_until_round: Option<u32>,
}

#[derive(Clone, Debug)]
struct LeaderCandidate {
    entity: Entity,
    owner: PlayerId,
    family: String,
}

/// Compatibility wrapper for future system scheduling.
pub fn leader_snapshot_system(world: &mut World) {
    let _ = snapshot_leader_bonuses(world, 0);
}

/// Compatibility wrapper for future system scheduling.
pub fn eval_outnumbered_system(world: &mut World) {
    let _ = eval_outnumbered_for_sub_step(world, 0);
}

/// Compatibility wrapper matching ADR-022's original function name.
pub fn eval_injured_bonuses(world: &mut World) -> Vec<Entity> {
    eval_injured_bonuses_at_boundary(world, 0, 0)
}

/// Clears BODYGUARD bonds whose target entity no longer exists.
pub fn bodyguard_cleanup_system(mut units: Query<&mut UnitKeywordState>, entities: &Entities) {
    for mut state in units.iter_mut() {
        if state
            .bodyguard_protects
            .is_some_and(|target| !entities.contains(target))
        {
            state.bodyguard_protects = None;
        }
    }
}

/// Snapshots LEADER bonuses after SS1 fully resolves and before SS2 starts.
///
/// Returns the LEADER entities whose snapshot was applied so the combat caller
/// can record local trace markers without coupling keyword evaluation to combat
/// logging internals.
pub fn snapshot_leader_bonuses(world: &mut World, current_round: u32) -> Vec<Entity> {
    let units = collect_unit_info(world);
    clear_leader_bonuses(world);

    let leaders = leader_candidates(world, &units, current_round);
    let mut applied_leaders = Vec::new();

    for leader in leaders {
        let mut applied_to_any = false;
        for unit in units.iter().filter(|unit| unit.hp > 0) {
            let Some(card) = card_for(unit.card_id, world) else {
                continue;
            };
            if unit.owner != leader.owner || card.family.as_deref() != Some(leader.family.as_str())
            {
                continue;
            }

            let Some(mut state) = world.get_mut::<UnitKeywordState>(unit.entity) else {
                continue;
            };
            state.leader_bonus_atk = state
                .leader_bonus_atk
                .saturating_add(DEFAULT_LEADER_ATK_BONUS);
            applied_to_any = true;
        }

        if applied_to_any {
            emit_keyword_triggered(
                world,
                Some(leader.entity),
                1,
                KeywordPayload::LeaderSnapshotTaken {
                    leader_unit_id: leader.entity.to_bits(),
                },
            );
            applied_leaders.push(leader.entity);
        }
    }

    applied_leaders
}

/// Recomputes OUTNUMBERED from the completed board state at a sub-step boundary.
///
/// The scan is bounded by live board units. It updates only units that carry the
/// OUTNUMBERED keyword and returns units whose cached boolean flipped.
pub fn eval_outnumbered_for_sub_step(world: &mut World, sub_step: u8) -> Vec<Entity> {
    let counts = count_live_board_units(world);
    let session_config = world.get_resource::<SessionConfig>().cloned();
    let outnumbered_by_player = counts
        .keys()
        .copied()
        .map(|player| {
            (
                player,
                player_count(player, &counts)
                    < opposing_count(player, &counts, session_config.as_ref()),
            )
        })
        .collect::<HashMap<_, _>>();

    let units = collect_unit_info(world);
    let mut flipped_units = Vec::new();
    let mut emitted_by_player = HashMap::<PlayerId, bool>::new();

    for unit in units {
        let Some(card) = card_for(unit.card_id, world) else {
            continue;
        };
        if !has_simple_keyword(card, SimpleKeyword::Outnumbered) {
            continue;
        }

        let now_active = outnumbered_by_player
            .get(&unit.owner)
            .copied()
            .unwrap_or(false);
        let Some(mut state) = world.get_mut::<UnitKeywordState>(unit.entity) else {
            continue;
        };
        let was_active = state.outnumbered_active;
        state.outnumbered_active = now_active;
        drop(state);

        if was_active != now_active {
            flipped_units.push(unit.entity);
            emitted_by_player.entry(unit.owner).or_insert(now_active);
        }
    }

    for (player_id, active) in emitted_by_player {
        emit_keyword_triggered(
            world,
            None,
            sub_step,
            KeywordPayload::OutnumberedFlipped { player_id, active },
        );
    }

    flipped_units
}

/// Evaluates INJURED-granted bonuses at a completed sub-step boundary.
///
/// INJURED itself is derived from `UnitStats.hp < CardData.hp`; only the
/// boundary-granted keyword state is cached on `UnitKeywordState`.
pub fn eval_injured_bonuses_at_boundary(
    world: &mut World,
    current_round: u32,
    sub_step: u8,
) -> Vec<Entity> {
    let units = collect_unit_info(world);
    let mut activated_units = Vec::new();

    for unit in units {
        let Some(card) = card_for(unit.card_id, world) else {
            continue;
        };
        let injured = unit.hp > 0 && unit.hp < card.hp;
        let silenced = is_silenced(unit.silenced_until_round, current_round);
        let Some(mut state) = world.get_mut::<UnitKeywordState>(unit.entity) else {
            continue;
        };

        let should_grant_first_strike = injured && !silenced && state.injured_grants_first_strike;
        let newly_active = should_grant_first_strike && !state.injured_first_strike_active;
        state.injured_first_strike_active = should_grant_first_strike;
        drop(state);

        if newly_active {
            emit_keyword_triggered(
                world,
                Some(unit.entity),
                sub_step,
                KeywordPayload::InjuredBonusActive {
                    granted_keyword: InjuredGrantedKeyword::FirstStrike,
                },
            );
            activated_units.push(unit.entity);
        }
    }

    activated_units
}

fn collect_unit_info(world: &mut World) -> Vec<UnitInfo> {
    let mut query = world.query::<(
        Entity,
        &UnitOwner,
        &UnitCardRef,
        &UnitStats,
        Option<&UnitKeywordState>,
    )>();
    query
        .iter(world)
        .map(|(entity, owner, card_ref, stats, state)| UnitInfo {
            entity,
            owner: owner.0,
            card_id: card_ref.0,
            hp: stats.hp,
            silenced_until_round: state.and_then(|state| state.silenced_until_round),
        })
        .collect()
}

fn clear_leader_bonuses(world: &mut World) {
    let mut query = world.query::<&mut UnitKeywordState>();
    for mut state in query.iter_mut(world) {
        state.leader_bonus_atk = 0;
        state.leader_bonus_hp = 0;
    }
}

fn leader_candidates(
    world: &World,
    units: &[UnitInfo],
    current_round: u32,
) -> Vec<LeaderCandidate> {
    let mut leaders_by_family = HashMap::<(PlayerId, String), LeaderCandidate>::new();

    for unit in units.iter().filter(|unit| unit.hp > 0) {
        let Some(card) = card_for(unit.card_id, world) else {
            continue;
        };
        if !has_simple_keyword(card, SimpleKeyword::Leader)
            || is_silenced(unit.silenced_until_round, current_round)
        {
            continue;
        }
        let Some(family) = card.family.clone() else {
            continue;
        };

        let key = (unit.owner, family.clone());
        let candidate = LeaderCandidate {
            entity: unit.entity,
            owner: unit.owner,
            family,
        };
        leaders_by_family
            .entry(key)
            .and_modify(|existing| {
                if candidate.entity.index() < existing.entity.index() {
                    *existing = candidate.clone();
                }
            })
            .or_insert(candidate);
    }

    let mut leaders = leaders_by_family.into_values().collect::<Vec<_>>();
    leaders.sort_by_key(|leader| (leader.owner.0, leader.family.clone(), leader.entity.index()));
    leaders
}

fn count_live_board_units(world: &mut World) -> HashMap<PlayerId, u32> {
    let raw_units = {
        let mut query = world.query::<(Entity, &UnitOwner, &UnitCardRef, &UnitStats)>();
        query
            .iter(world)
            .map(|(entity, owner, card_ref, stats)| (entity, owner.0, card_ref.0, stats.hp))
            .collect::<Vec<_>>()
    };

    let mut counts = HashMap::<PlayerId, u32>::new();
    for (_, owner, card_id, hp) in raw_units {
        if hp == 0 {
            continue;
        }
        let Some(card) = card_for(card_id, world) else {
            continue;
        };
        if !is_counted_for_outnumbered(card.card_type) {
            continue;
        }
        counts
            .entry(owner)
            .and_modify(|count| *count = count.saturating_add(1))
            .or_insert(1);
    }

    counts
}

fn is_counted_for_outnumbered(card_type: CardType) -> bool {
    matches!(card_type, CardType::Minion | CardType::Structure)
}

fn player_count(player: PlayerId, counts: &HashMap<PlayerId, u32>) -> u32 {
    counts.get(&player).copied().unwrap_or(0)
}

fn opposing_count(
    player: PlayerId,
    counts: &HashMap<PlayerId, u32>,
    session_config: Option<&SessionConfig>,
) -> u32 {
    let own_team = session_config.and_then(|session| session.team_map.get(&player).copied());

    counts
        .iter()
        .filter(|(other_player, _)| {
            if **other_player == player {
                return false;
            }
            let Some(session) = session_config else {
                return true;
            };
            match (own_team, session.team_map.get(other_player).copied()) {
                (Some(own), Some(other)) => own != other,
                _ => true,
            }
        })
        .map(|(_, count)| *count)
        .sum()
}

fn card_for(card_id: CardId, world: &World) -> Option<&CardData> {
    world.get_resource::<CardCatalog>()?.cards.get(&card_id)
}

fn has_simple_keyword(card: &CardData, keyword: SimpleKeyword) -> bool {
    card.keywords
        .iter()
        .any(|candidate| matches!(candidate, Keyword::Simple(simple) if *simple == keyword))
}

fn is_silenced(silenced_until_round: Option<u32>, current_round: u32) -> bool {
    silenced_until_round.is_some_and(|round| current_round <= round)
}

fn emit_keyword_triggered(
    world: &mut World,
    source: Option<Entity>,
    sub_step: u8,
    payload: KeywordPayload,
) {
    if let Some(mut messages) = world.get_resource_mut::<Messages<KeywordTriggered>>() {
        messages.write(KeywordTriggered {
            source_unit_id: source.map(Entity::to_bits),
            sub_step,
            payload,
        });
    }
}
