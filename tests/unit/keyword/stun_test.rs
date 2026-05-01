use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use server::feature::keyword::components::{EnteredPlayRound, UnitKeywordState};
use server::feature::keyword::effects::{
    apply_first_strike, apply_stun, can_execute_first_strike, can_execute_standard_attack,
    can_execute_standard_movement, can_participate_in_sub_step,
    charge_x_cells_for_sub_step, clear_stun_state_at_resolution_end, ActionSubStep,
    STUN_DURATION_ROUNDS,
};
use server::feature::keyword::KeywordTriggered;
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Keyword, Rarity, SimpleKeyword, UnitType};
use shared::keyword::KeywordPayload;
use shared::session::PlayerId;

const ROUND: u32 = 9;

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

fn stun_active(world: &World, entity: Entity) -> bool {
    world
        .get::<UnitKeywordState>(entity)
        .expect("unit should have keyword state")
        .stun_active
}

fn keyword_messages(world: &World) -> Vec<KeywordTriggered> {
    let messages = world.resource::<Messages<KeywordTriggered>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

#[test]
fn kw_015a_stunned_unit_takes_damage_but_cannot_act() {
    let mut world = World::new();
    insert_catalog(
        &mut world,
        vec![
            card(1, vec![simple(SimpleKeyword::FirstStrike)]),
            card(
                2,
                vec![
                    simple(SimpleKeyword::Haste),
                    simple(SimpleKeyword::FirstStrike),
                    Keyword::ChargeXMove { cells: 2 },
                ],
            ),
        ],
    );
    let attacker = spawn_unit(
        &mut world,
        CardId(1),
        UnitStats::new(4, 2, 0, 0),
        None,
        UnitKeywordState::default(),
    );
    let stunned_target = spawn_unit(
        &mut world,
        CardId(2),
        UnitStats::new(5, 3, 2, 0),
        Some(ROUND),
        UnitKeywordState {
            stun_active: true,
            ..default()
        },
    );

    let result = apply_first_strike(attacker, stunned_target, &mut world);

    assert!(result.applied);
    assert_eq!(result.damage_to_target, 2);
    assert_eq!(hp(&world, stunned_target), 3);
    assert_eq!(result.damage_to_attacker, 0);
    assert!(!result.target_retaliated_in_ss3);
    assert!(charge_x_cells_for_sub_step(stunned_target, ROUND, &world).is_none());
    assert!(!can_execute_first_strike(stunned_target, ROUND, &world));
    assert!(!can_execute_standard_movement(
        stunned_target,
        ROUND,
        &world
    ));
    assert!(!can_execute_standard_attack(stunned_target, ROUND, &world));
}

#[test]
fn kw_015b_stun_clears_at_resolution_end_before_next_round_actions() {
    let mut world = World::new();
    insert_catalog(
        &mut world,
        vec![card(
            1,
            vec![
                simple(SimpleKeyword::FirstStrike),
                Keyword::ChargeXMove { cells: 2 },
            ],
        )],
    );
    let unit = spawn_unit(
        &mut world,
        CardId(1),
        UnitStats::new(5, 3, 2, 0),
        Some(ROUND),
        UnitKeywordState {
            stun_active: true,
            ..default()
        },
    );

    assert!(!can_execute_standard_attack(unit, ROUND + 1, &world));

    let cleared = clear_stun_state_at_resolution_end(&mut world);

    assert_eq!(cleared, 1);
    assert!(!stun_active(&world, unit));
    assert_eq!(clear_stun_state_at_resolution_end(&mut world), 0);
    assert_eq!(charge_x_cells_for_sub_step(unit, ROUND + 1, &world), Some(2));
    assert!(can_execute_first_strike(unit, ROUND + 1, &world));
    assert!(can_execute_standard_movement(unit, ROUND + 1, &world));
    assert!(can_execute_standard_attack(unit, ROUND + 1, &world));
}

#[test]
fn kw_034_stun_applied_in_ss1_overrides_haste_and_emits_one_round_payload() {
    let mut app = App::new();
    app.add_message::<KeywordTriggered>();
    app.finish();
    app.cleanup();

    let world = app.world_mut();
    insert_catalog(
        world,
        vec![card(
            1,
            vec![
                simple(SimpleKeyword::Haste),
                simple(SimpleKeyword::FirstStrike),
                Keyword::ChargeXMove { cells: 3 },
            ],
        )],
    );
    let haste_unit = spawn_unit(
        world,
        CardId(1),
        UnitStats::new(5, 3, 2, 0),
        Some(ROUND),
        UnitKeywordState::default(),
    );

    assert!(can_execute_standard_attack(haste_unit, ROUND, world));

    assert!(apply_stun(haste_unit, Some(55), 1, world));

    assert!(stun_active(world, haste_unit));
    assert!(charge_x_cells_for_sub_step(haste_unit, ROUND, world).is_none());
    assert!(!can_execute_first_strike(haste_unit, ROUND, world));
    for sub_step in [
        ActionSubStep::ChargeX,
        ActionSubStep::FirstStrike,
        ActionSubStep::StandardMovement,
        ActionSubStep::StandardAttack,
    ] {
        assert!(
            !can_participate_in_sub_step(haste_unit, sub_step, ROUND, world),
            "stunned HASTE unit should skip SS{}",
            sub_step.number()
        );
    }

    let messages = keyword_messages(world);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].source_unit_id, Some(55));
    assert_eq!(messages[0].sub_step, 1);
    assert!(matches!(
        messages[0].payload,
        KeywordPayload::StunApplied { duration_rounds: STUN_DURATION_ROUNDS }
    ));
}

#[test]
fn apply_stun_is_a_noop_without_keyword_state() {
    let mut app = App::new();
    app.add_message::<KeywordTriggered>();
    app.finish();
    app.cleanup();

    let world = app.world_mut();
    let unit = world.spawn_empty().id();

    assert!(!apply_stun(unit, None, 1, world));
    assert!(keyword_messages(world).is_empty());
}
