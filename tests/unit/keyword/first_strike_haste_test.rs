use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use server::feature::keyword::components::{EnteredPlayRound, UnitKeywordState};
use server::feature::keyword::effects::{
    apply_first_strike, can_execute_first_strike, can_execute_standard_attack,
    can_execute_standard_movement, can_participate_in_sub_step, charge_x_cells_for_sub_step,
    ActionSubStep,
};
use server::feature::keyword::movement::{advance_destination, PlayerSide};
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Keyword, Rarity, SimpleKeyword, UnitType};
use shared::session::PlayerId;

const ROUND: u32 = 7;

fn card(id: u32, keywords: Vec<Keyword>) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Neutral,
        family: None,
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Neutral,
        cost: 1,
        atk: 0,
        hp: 0,
        mp: 0,
        ar: 0,
        keywords,
        effect_text: String::new(),
        art_id: format!("test_{id}"),
        pool_copies_override: None,
    }
}

fn simple(keyword: SimpleKeyword) -> Keyword {
    Keyword::Simple(keyword)
}

fn insert_catalog(world: &mut World, cards: Vec<CardData>) {
    world.insert_resource(CardCatalog {
        cards: cards.into_iter().map(|card| (card.id, card)).collect(),
    });
}

fn spawn_unit(
    world: &mut World,
    card_id: CardId,
    stats: UnitStats,
    entered_round: Option<u32>,
    keyword_state: UnitKeywordState,
) -> Entity {
    let mut entity = world.spawn((
        UnitCardRef(card_id),
        UnitStats::new(stats.hp, stats.atk, stats.mp, stats.ar),
        BoardPosition { lane: 1, cell: 5 },
        UnitOwner(PlayerId(0)),
        keyword_state,
    ));

    if let Some(round) = entered_round {
        entity.insert(EnteredPlayRound(round));
    }

    entity.id()
}

fn hp(world: &World, entity: Entity) -> u8 {
    world
        .get::<UnitStats>(entity)
        .expect("unit should have UnitStats")
        .hp
}

#[test]
fn kw_011_first_strike_hits_standard_enemy_without_ss3_retaliation() {
    let mut world = World::new();
    insert_catalog(
        &mut world,
        vec![
            card(1, vec![simple(SimpleKeyword::FirstStrike)]),
            card(2, vec![]),
        ],
    );
    let attacker = spawn_unit(
        &mut world,
        CardId(1),
        UnitStats::new(4, 3, 0, 0),
        None,
        UnitKeywordState::default(),
    );
    let target = spawn_unit(
        &mut world,
        CardId(2),
        UnitStats::new(4, 3, 0, 0),
        None,
        UnitKeywordState::default(),
    );

    let result = apply_first_strike(attacker, target, &mut world);

    assert!(result.applied);
    assert_eq!(result.damage_to_target, 3);
    assert_eq!(result.damage_to_attacker, 0);
    assert_eq!(hp(&world, attacker), 4);
    assert_eq!(hp(&world, target), 1);
    assert!(!result.target_retaliated_in_ss3);
    assert!(can_execute_standard_attack(target, ROUND, &world));
}

#[test]
fn kw_011_first_strike_can_kill_before_standard_enemy_retaliates() {
    let mut world = World::new();
    insert_catalog(
        &mut world,
        vec![
            card(1, vec![simple(SimpleKeyword::FirstStrike)]),
            card(2, vec![]),
        ],
    );
    let attacker = spawn_unit(
        &mut world,
        CardId(1),
        UnitStats::new(4, 3, 0, 0),
        None,
        UnitKeywordState::default(),
    );
    let target = spawn_unit(
        &mut world,
        CardId(2),
        UnitStats::new(2, 3, 0, 0),
        None,
        UnitKeywordState::default(),
    );

    let result = apply_first_strike(attacker, target, &mut world);

    assert!(result.target_defeated());
    assert_eq!(hp(&world, target), 0);
    assert_eq!(hp(&world, attacker), 4);
    assert_eq!(result.damage_to_attacker, 0);
}

#[test]
fn kw_012_first_strike_units_damage_each_other_from_pre_hit_snapshot() {
    let mut world = World::new();
    insert_catalog(
        &mut world,
        vec![
            card(1, vec![simple(SimpleKeyword::FirstStrike)]),
            card(2, vec![simple(SimpleKeyword::FirstStrike)]),
        ],
    );
    let attacker = spawn_unit(
        &mut world,
        CardId(1),
        UnitStats::new(4, 3, 0, 0),
        None,
        UnitKeywordState::default(),
    );
    let target = spawn_unit(
        &mut world,
        CardId(2),
        UnitStats::new(3, 2, 0, 0),
        None,
        UnitKeywordState::default(),
    );

    let result = apply_first_strike(attacker, target, &mut world);

    assert!(result.target_retaliated_in_ss3);
    assert_eq!(result.attacker_hp_before, Some(4));
    assert_eq!(result.target_hp_before, Some(3));
    assert_eq!(result.damage_to_target, 3);
    assert_eq!(result.damage_to_attacker, 2);
    assert_eq!(hp(&world, attacker), 2);
    assert_eq!(hp(&world, target), 0);
}

#[test]
fn kw_013_haste_unit_placed_this_round_can_move_and_attack() {
    let mut world = World::new();
    insert_catalog(
        &mut world,
        vec![card(1, vec![simple(SimpleKeyword::Haste)]), card(2, vec![])],
    );
    let haste = spawn_unit(
        &mut world,
        CardId(1),
        UnitStats::new(3, 2, 2, 0),
        Some(ROUND),
        UnitKeywordState::default(),
    );
    let no_haste = spawn_unit(
        &mut world,
        CardId(2),
        UnitStats::new(3, 2, 2, 0),
        Some(ROUND),
        UnitKeywordState::default(),
    );

    assert!(can_execute_standard_movement(haste, ROUND, &world));
    assert!(can_execute_standard_attack(haste, ROUND, &world));
    assert!(!can_execute_standard_movement(no_haste, ROUND, &world));
    assert!(!can_execute_standard_attack(no_haste, ROUND, &world));
}

#[test]
fn kw_014_stun_overrides_haste_for_all_action_sub_steps() {
    let mut world = World::new();
    insert_catalog(
        &mut world,
        vec![card(
            1,
            vec![
                simple(SimpleKeyword::Haste),
                simple(SimpleKeyword::FirstStrike),
            ],
        )],
    );
    let stunned_haste = spawn_unit(
        &mut world,
        CardId(1),
        UnitStats::new(3, 2, 2, 0),
        Some(ROUND),
        UnitKeywordState {
            stun_active: true,
            ..default()
        },
    );

    for sub_step in [
        ActionSubStep::ChargeX,
        ActionSubStep::FirstStrike,
        ActionSubStep::StandardMovement,
        ActionSubStep::StandardAttack,
    ] {
        assert!(
            !can_participate_in_sub_step(stunned_haste, sub_step, ROUND, &world),
            "stunned HASTE unit should skip SS{}",
            sub_step.number()
        );
    }
    assert!(charge_x_cells_for_sub_step(stunned_haste, ROUND, &world).is_none());
    assert!(!can_execute_first_strike(stunned_haste, ROUND, &world));
}

#[test]
fn kw_042_haste_first_strike_unit_attacks_in_placement_round_ss3() {
    let mut world = World::new();
    insert_catalog(
        &mut world,
        vec![
            card(
                1,
                vec![
                    simple(SimpleKeyword::Haste),
                    simple(SimpleKeyword::FirstStrike),
                ],
            ),
            card(2, vec![]),
            card(3, vec![simple(SimpleKeyword::FirstStrike)]),
        ],
    );
    let haste_first_strike = spawn_unit(
        &mut world,
        CardId(1),
        UnitStats::new(4, 3, 0, 0),
        Some(ROUND),
        UnitKeywordState::default(),
    );
    let target = spawn_unit(
        &mut world,
        CardId(2),
        UnitStats::new(4, 2, 0, 0),
        None,
        UnitKeywordState::default(),
    );
    let first_strike_without_haste = spawn_unit(
        &mut world,
        CardId(3),
        UnitStats::new(4, 3, 0, 0),
        Some(ROUND),
        UnitKeywordState::default(),
    );

    assert!(can_execute_first_strike(haste_first_strike, ROUND, &world));
    assert!(!can_execute_first_strike(
        first_strike_without_haste,
        ROUND,
        &world
    ));

    let result = apply_first_strike(haste_first_strike, target, &mut world);

    assert_eq!(result.damage_to_target, 3);
    assert_eq!(hp(&world, target), 1);
    assert!(can_execute_standard_attack(
        haste_first_strike,
        ROUND,
        &world
    ));
}

#[test]
fn kw_043_haste_charge_x_unit_gets_ss2_bonus_and_later_actions() {
    let mut world = World::new();
    insert_catalog(
        &mut world,
        vec![card(
            1,
            vec![
                simple(SimpleKeyword::Haste),
                Keyword::ChargeXMove { cells: 2 },
            ],
        )],
    );
    let haste_charge = spawn_unit(
        &mut world,
        CardId(1),
        UnitStats::new(3, 2, 1, 0),
        Some(ROUND),
        UnitKeywordState::default(),
    );

    let charge_cells =
        charge_x_cells_for_sub_step(haste_charge, ROUND, &world).expect("CHARGE X should fire");

    assert_eq!(charge_cells, 2);
    assert_eq!(advance_destination(1, PlayerSide::PlayerA, charge_cells), 3);
    assert!(can_execute_standard_movement(haste_charge, ROUND, &world));
    assert!(can_execute_standard_attack(haste_charge, ROUND, &world));
}
