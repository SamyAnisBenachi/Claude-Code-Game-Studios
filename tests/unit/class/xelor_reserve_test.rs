use std::collections::HashMap;

use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use server::core::economy::{api as economy_api, PlayerEconomies, PlayerEconomy};
use server::feature::class::resolution::effects::{
    apply_gelure, apply_rollback, apply_xelorium, pay_xelorium_cost, RollbackMovementRules,
};
use server::feature::keyword::UnitKeywordState;
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::session::PlayerId;

const MINION_CARD_ID: CardId = CardId(10_001);
const STRUCTURE_CARD_ID: CardId = CardId(10_002);
const ROLLBACK_COST: u32 = 3;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn economy(current_mana: u32, reserve_mana: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold: 0,
        current_mana,
        reserve_mana,
        mana_cap: 10,
        reserved_gold: 0,
    }
}

fn economies(entries: &[(PlayerId, u32, u32)]) -> PlayerEconomies {
    PlayerEconomies(HashMap::from_iter(entries.iter().map(
        |(player, current, reserve)| (*player, economy(*current, *reserve)),
    )))
}

fn card(id: CardId, card_type: CardType) -> CardData {
    CardData {
        id,
        name_fr: format!("card {}", id.0),
        name_en: format!("card {}", id.0),
        class: ClassId::Neutral,
        family: None,
        rarity: Rarity::Common,
        card_type,
        unit_type: UnitType::Neutral,
        cost: 0,
        atk: 1,
        hp: 2,
        mp: 3,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: String::new(),
        pool_copies_override: None,
    }
}

fn insert_catalog(world: &mut World) {
    world.insert_resource(CardCatalog {
        cards: HashMap::from([
            (MINION_CARD_ID, card(MINION_CARD_ID, CardType::Minion)),
            (
                STRUCTURE_CARD_ID,
                card(STRUCTURE_CARD_ID, CardType::Structure),
            ),
        ]),
    });
}

fn spawn_unit(
    world: &mut World,
    owner: PlayerId,
    lane: u8,
    cell: u8,
    card_id: CardId,
    keyword_state: Option<UnitKeywordState>,
) -> Entity {
    let mut entity = world.spawn((
        UnitCardRef(card_id),
        UnitOwner(owner),
        UnitStats::new(2, 1, 3, 0),
        BoardPosition { lane, cell },
    ));
    if let Some(keyword_state) = keyword_state {
        entity.insert(keyword_state);
    }
    entity.id()
}

fn cell(world: &World, entity: Entity) -> u8 {
    world
        .get::<BoardPosition>(entity)
        .expect("unit should have BoardPosition")
        .cell
}

fn player_a_rules() -> RollbackMovementRules {
    RollbackMovementRules::new(1, 1, 8)
}

#[test]
fn cs_ac_04_gelure_transfers_current_mana_to_reserve() {
    let xelor = player(1);
    let mut economies = economies(&[(xelor, 5, 2)]);

    apply_gelure(&mut economies, xelor);

    let economy = economies.0.get(&xelor).expect("xelor economy");
    assert_eq!(economy.current_mana, 0);
    assert_eq!(economy.reserve_mana, 7);
}

#[test]
fn cs_ac_05_xelorium_steals_opponent_current_mana_after_cost() {
    let xelor = player(1);
    let opponent = player(2);
    let mut economies = economies(&[(xelor, 8, 3), (opponent, 6, 8)]);

    assert_eq!(pay_xelorium_cost(&mut economies, xelor), Ok(()));
    apply_xelorium(&mut economies, xelor, opponent);

    let xelor_economy = economies.0.get(&xelor).expect("xelor economy");
    let opponent_economy = economies.0.get(&opponent).expect("opponent economy");
    assert_eq!(xelor_economy.current_mana, 4);
    assert_eq!(xelor_economy.reserve_mana, 9);
    assert_eq!(opponent_economy.current_mana, 0);
    assert_eq!(opponent_economy.reserve_mana, 8);
}

#[test]
fn cs_ac_05b_xelorium_exact_current_mana_cost_is_valid() {
    let xelor = player(1);
    let opponent = player(2);
    let mut economies = economies(&[(xelor, 4, 0), (opponent, 6, 0)]);

    assert_eq!(pay_xelorium_cost(&mut economies, xelor), Ok(()));
    apply_xelorium(&mut economies, xelor, opponent);

    let xelor_economy = economies.0.get(&xelor).expect("xelor economy");
    let opponent_economy = economies.0.get(&opponent).expect("opponent economy");
    assert_eq!(xelor_economy.current_mana, 0);
    assert_eq!(xelor_economy.reserve_mana, 6);
    assert_eq!(opponent_economy.current_mana, 0);
}

#[test]
fn cs_ac_06_rollback_consumes_reserve_and_moves_friendly_minions() {
    let xelor = player(1);
    let opponent = player(2);
    let mut economies = economies(&[(xelor, 0, 4), (opponent, 0, 0)]);
    let mut world = World::new();
    insert_catalog(&mut world);
    let first = spawn_unit(&mut world, xelor, 1, 2, MINION_CARD_ID, None);
    let second = spawn_unit(&mut world, xelor, 2, 3, MINION_CARD_ID, None);
    let third = spawn_unit(&mut world, xelor, 3, 5, MINION_CARD_ID, None);
    let enemy = spawn_unit(&mut world, opponent, 1, 2, MINION_CARD_ID, None);
    let structure = spawn_unit(&mut world, xelor, 4, 2, STRUCTURE_CARD_ID, None);

    let outcome = apply_rollback(&mut economies, &mut world, xelor, player_a_rules());

    assert_eq!(outcome.reserve_spent, 4);
    assert_eq!(outcome.units_moved, 3);
    assert_eq!(economies.0.get(&xelor).expect("xelor").reserve_mana, 0);
    assert_eq!(cell(&world, first), 6);
    assert_eq!(cell(&world, second), 7);
    assert_eq!(cell(&world, third), 8);
    assert_eq!(cell(&world, enemy), 2);
    assert_eq!(cell(&world, structure), 2);
}

#[test]
fn cs_ac_07_rollback_zero_reserve_keeps_units_in_place_after_normal_cost() {
    let xelor = player(1);
    let mut economies = economies(&[(xelor, 5, 0)]);
    let mut world = World::new();
    insert_catalog(&mut world);
    let first = spawn_unit(&mut world, xelor, 1, 3, MINION_CARD_ID, None);
    let second = spawn_unit(&mut world, xelor, 2, 5, MINION_CARD_ID, None);

    {
        let economy = economies.0.get_mut(&xelor).expect("xelor economy");
        assert_eq!(
            economy_api::validate_spend(economy, ROLLBACK_COST, false),
            Ok(())
        );
        economy_api::apply_spend(economy, ROLLBACK_COST, false);
    }
    let outcome = apply_rollback(&mut economies, &mut world, xelor, player_a_rules());

    let economy = economies.0.get(&xelor).expect("xelor economy");
    assert_eq!(economy.current_mana, 2);
    assert_eq!(economy.reserve_mana, 0);
    assert_eq!(outcome.reserve_spent, 0);
    assert_eq!(outcome.units_moved, 0);
    assert_eq!(cell(&world, first), 3);
    assert_eq!(cell(&world, second), 5);
}

#[test]
fn cs_ac_08_rollback_skips_stunned_units() {
    let xelor = player(1);
    let mut economies = economies(&[(xelor, 0, 5)]);
    let mut world = World::new();
    insert_catalog(&mut world);
    let healthy = spawn_unit(&mut world, xelor, 1, 2, MINION_CARD_ID, None);
    let stunned = spawn_unit(
        &mut world,
        xelor,
        1,
        4,
        MINION_CARD_ID,
        Some(UnitKeywordState {
            stun_active: true,
            ..UnitKeywordState::default()
        }),
    );

    let outcome = apply_rollback(&mut economies, &mut world, xelor, player_a_rules());

    assert_eq!(outcome.reserve_spent, 5);
    assert_eq!(outcome.units_moved, 1);
    assert_eq!(outcome.stunned_units_skipped, 1);
    assert_eq!(economies.0.get(&xelor).expect("xelor").reserve_mana, 0);
    assert_eq!(cell(&world, healthy), 7);
    assert_eq!(cell(&world, stunned), 4);
}
