use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use server::feature::keyword::components::UnitKeywordState;
use server::feature::keyword::effects::{
    apply_first_strike, apply_simultaneous_attacks_to_target, check_shield_absorb,
    consume_shield_for_sub_step,
};
use server::feature::keyword::KeywordTriggered;
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Keyword, Rarity, SimpleKeyword, UnitType};
use shared::keyword::KeywordPayload;
use shared::session::PlayerId;

const SS3: u8 = 3;
const SS6: u8 = 6;

fn card(id: u32, keywords: Vec<Keyword>) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Card {id}"),
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

fn app_with_keyword_messages() -> App {
    let mut app = App::new();
    app.add_message::<KeywordTriggered>();
    app.finish();
    app.cleanup();
    app
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
    keyword_state: UnitKeywordState,
) -> Entity {
    world
        .spawn((
            UnitCardRef(card_id),
            UnitStats::new(stats.hp, stats.atk, stats.mp, stats.ar),
            BoardPosition { lane: 1, cell: 5 },
            UnitOwner(PlayerId(0)),
            keyword_state,
        ))
        .id()
}

fn hp(world: &World, entity: Entity) -> u8 {
    world
        .get::<UnitStats>(entity)
        .expect("unit should have UnitStats")
        .hp
}

fn shield_active(world: &World, entity: Entity) -> bool {
    world
        .get::<UnitKeywordState>(entity)
        .expect("unit should have UnitKeywordState")
        .shield_active
}

fn keyword_messages(world: &World) -> Vec<KeywordTriggered> {
    let messages = world.resource::<Messages<KeywordTriggered>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

#[test]
fn check_shield_absorb_consumes_once_only_in_damage_sub_steps() {
    let mut state = UnitKeywordState {
        shield_active: true,
        ..default()
    };

    assert!(!check_shield_absorb(&mut state, 1));
    assert!(state.shield_active);
    assert!(check_shield_absorb(&mut state, SS3));
    assert!(!state.shield_active);
    assert!(!check_shield_absorb(&mut state, SS6));
}

#[test]
fn consume_shield_for_sub_step_emits_shield_consumed_payload() {
    let mut app = app_with_keyword_messages();
    let world = app.world_mut();
    let unit = spawn_unit(
        world,
        CardId(1),
        UnitStats::new(6, 1, 0, 0),
        UnitKeywordState {
            shield_active: true,
            ..default()
        },
    );

    assert!(consume_shield_for_sub_step(unit, SS3, world));
    assert!(!shield_active(world, unit));

    let messages = keyword_messages(world);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].source_unit_id, Some(unit.to_bits()));
    assert_eq!(messages[0].sub_step, SS3);
    assert!(matches!(
        messages[0].payload,
        KeywordPayload::ShieldConsumed
    ));
}

#[test]
fn kw_024_shield_consumed_in_ss3_does_not_protect_ss6_melee_attackers() {
    let mut app = app_with_keyword_messages();
    let world = app.world_mut();
    insert_catalog(
        world,
        vec![
            card(
                1,
                vec![
                    simple(SimpleKeyword::FirstStrike),
                    Keyword::RangeX { max_range: 3 },
                ],
            ),
            card(2, vec![simple(SimpleKeyword::Shield)]),
            card(3, vec![]),
            card(4, vec![]),
        ],
    );
    let range_first_strike = spawn_unit(
        world,
        CardId(1),
        UnitStats::new(4, 2, 0, 0),
        UnitKeywordState::default(),
    );
    let shielded = spawn_unit(
        world,
        CardId(2),
        UnitStats::new(6, 1, 0, 0),
        UnitKeywordState {
            shield_active: true,
            ..default()
        },
    );
    let melee_a = spawn_unit(
        world,
        CardId(3),
        UnitStats::new(4, 3, 0, 0),
        UnitKeywordState::default(),
    );
    let melee_b = spawn_unit(
        world,
        CardId(4),
        UnitStats::new(4, 2, 0, 0),
        UnitKeywordState::default(),
    );

    let ss3 = apply_first_strike(range_first_strike, shielded, world);

    assert!(ss3.applied);
    assert!(ss3.target_shield_absorbed);
    assert_eq!(ss3.damage_to_target, 0);
    assert_eq!(hp(world, shielded), 6);
    assert!(!shield_active(world, shielded));

    let ss6 = apply_simultaneous_attacks_to_target(shielded, &[melee_a, melee_b], SS6, world);

    assert!(ss6.applied);
    assert!(!ss6.shield_absorbed);
    assert_eq!(ss6.per_attacker_damage, vec![(melee_a, 3), (melee_b, 2)]);
    assert_eq!(ss6.damage_to_target, 5);
    assert_eq!(hp(world, shielded), 1);

    let messages = keyword_messages(world);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].sub_step, SS3);
    assert!(matches!(
        messages[0].payload,
        KeywordPayload::ShieldConsumed
    ));
}

#[test]
fn kw_037_range_first_strike_second_attack_hits_after_ss3_shield_consumption() {
    let mut app = app_with_keyword_messages();
    let world = app.world_mut();
    insert_catalog(
        world,
        vec![
            card(
                1,
                vec![
                    simple(SimpleKeyword::FirstStrike),
                    Keyword::RangeX { max_range: 3 },
                ],
            ),
            card(2, vec![simple(SimpleKeyword::Shield)]),
        ],
    );
    let range_first_strike = spawn_unit(
        world,
        CardId(1),
        UnitStats::new(4, 2, 0, 0),
        UnitKeywordState::default(),
    );
    let shielded = spawn_unit(
        world,
        CardId(2),
        UnitStats::new(6, 1, 0, 0),
        UnitKeywordState {
            shield_active: true,
            ..default()
        },
    );

    let ss3 = apply_first_strike(range_first_strike, shielded, world);
    let ss6 = apply_simultaneous_attacks_to_target(shielded, &[range_first_strike], SS6, world);

    assert!(ss3.target_shield_absorbed);
    assert_eq!(ss3.damage_to_target, 0);
    assert!(!ss6.shield_absorbed);
    assert_eq!(ss6.damage_to_target, 2);
    assert_eq!(hp(world, shielded), 4);
    assert_eq!(keyword_messages(world).len(), 1);
}

#[test]
fn simultaneous_attackers_are_absorbed_by_one_shield_consumption() {
    let mut app = app_with_keyword_messages();
    let world = app.world_mut();
    let shielded = spawn_unit(
        world,
        CardId(1),
        UnitStats::new(6, 1, 0, 0),
        UnitKeywordState {
            shield_active: true,
            ..default()
        },
    );
    let attacker_a = spawn_unit(
        world,
        CardId(2),
        UnitStats::new(4, 3, 0, 0),
        UnitKeywordState::default(),
    );
    let attacker_b = spawn_unit(
        world,
        CardId(3),
        UnitStats::new(4, 2, 0, 0),
        UnitKeywordState::default(),
    );

    let result =
        apply_simultaneous_attacks_to_target(shielded, &[attacker_a, attacker_b], SS6, world);

    assert!(result.applied);
    assert!(result.shield_absorbed);
    assert_eq!(
        result.per_attacker_damage,
        vec![(attacker_a, 0), (attacker_b, 0)]
    );
    assert_eq!(result.damage_to_target, 0);
    assert_eq!(hp(world, shielded), 6);
    assert!(!shield_active(world, shielded));
    assert_eq!(keyword_messages(world).len(), 1);
}

#[test]
fn shield_persists_across_rounds_until_an_incoming_attack_triggers_it() {
    let mut app = app_with_keyword_messages();
    let world = app.world_mut();
    let shielded = spawn_unit(
        world,
        CardId(1),
        UnitStats::new(6, 1, 0, 0),
        UnitKeywordState {
            shield_active: true,
            ..default()
        },
    );

    let no_attack = apply_simultaneous_attacks_to_target(shielded, &[], SS6, world);

    assert!(!no_attack.applied);
    assert!(shield_active(world, shielded));
    assert!(keyword_messages(world).is_empty());
    assert!(consume_shield_for_sub_step(shielded, SS3, world));
}

#[test]
fn shield_consumed_in_ss6_does_not_protect_next_round_ss3() {
    let mut app = app_with_keyword_messages();
    let world = app.world_mut();
    insert_catalog(
        world,
        vec![
            card(1, vec![simple(SimpleKeyword::FirstStrike)]),
            card(2, vec![]),
        ],
    );
    let ss6_attacker = spawn_unit(
        world,
        CardId(2),
        UnitStats::new(4, 2, 0, 0),
        UnitKeywordState::default(),
    );
    let next_round_first_strike = spawn_unit(
        world,
        CardId(1),
        UnitStats::new(4, 2, 0, 0),
        UnitKeywordState::default(),
    );
    let shielded = spawn_unit(
        world,
        CardId(2),
        UnitStats::new(6, 1, 0, 0),
        UnitKeywordState {
            shield_active: true,
            ..default()
        },
    );

    let ss6 = apply_simultaneous_attacks_to_target(shielded, &[ss6_attacker], SS6, world);
    let next_ss3 = apply_first_strike(next_round_first_strike, shielded, world);

    assert!(ss6.shield_absorbed);
    assert_eq!(ss6.damage_to_target, 0);
    assert!(!next_ss3.target_shield_absorbed);
    assert_eq!(next_ss3.damage_to_target, 2);
    assert_eq!(hp(world, shielded), 4);
    assert_eq!(keyword_messages(world).len(), 1);
}
