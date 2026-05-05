use std::collections::HashMap;

use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use server::core::rsm::BeginResolution;
use server::core::session::SessionConfig;
use server::feature::board::{BoardCell, BoardConfig, BoardGrid, BoardOccupancy};
use server::feature::combat::{CombatPlugin, CombatResolutionTrace, CombatTraceEntry};
use server::feature::keyword::components::UnitKeywordState;
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Keyword, Rarity, SimpleKeyword, UnitType};
use shared::keyword::KeywordKind;
use shared::protocol::GameMode;
use shared::session::PlayerId;

const ROUND: u32 = 5;
const PLAYER_A: PlayerId = PlayerId(1);
const PLAYER_B: PlayerId = PlayerId(2);

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
        cost: 0,
        atk: 1,
        hp: 3,
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

fn range_first_strike(range: u8) -> Vec<Keyword> {
    vec![
        Keyword::RangeX { max_range: range },
        simple(SimpleKeyword::FirstStrike),
    ]
}

fn session_config() -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(PLAYER_A, 0), (PLAYER_B, 1)]),
        class_map: HashMap::from([(PLAYER_A, ClassId::Iop), (PLAYER_B, ClassId::Cra)]),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn app_with_cards(cards: Vec<CardData>) -> App {
    let mut app = App::new();
    app.add_plugins(CombatPlugin);
    app.insert_resource(BoardConfig::default());
    app.insert_resource(BoardGrid::default());
    app.insert_resource(BoardOccupancy::default());
    app.insert_resource(session_config());
    app.insert_resource(CardCatalog {
        cards: cards.into_iter().map(|card| (card.id, card)).collect(),
    });
    app
}

fn spawn_unit(
    app: &mut App,
    card_id: CardId,
    owner: PlayerId,
    lane: u8,
    cell: u8,
    stats: UnitStats,
    keyword_state: UnitKeywordState,
) -> Entity {
    let entity = app
        .world_mut()
        .spawn((
            UnitCardRef(card_id),
            UnitOwner(owner),
            stats,
            keyword_state,
            BoardPosition { lane, cell },
        ))
        .id();

    if let Some((lane_index, cell_index)) = grid_indices(lane, cell) {
        app.world_mut().resource_mut::<BoardGrid>().lanes[lane_index][cell_index] =
            Some(BoardCell::new(entity));
    }
    app.world_mut()
        .resource_mut::<BoardOccupancy>()
        .minion_slots
        .insert((owner, lane), entity);

    entity
}

fn begin_resolution(app: &mut App) {
    app.world_mut()
        .write_message(BeginResolution { round: ROUND });
    app.update();
}

fn hp(app: &App, unit: Entity) -> u8 {
    app.world()
        .entity(unit)
        .get::<UnitStats>()
        .expect("unit should have UnitStats")
        .hp
}

fn shield_active(app: &App, unit: Entity) -> bool {
    app.world()
        .entity(unit)
        .get::<UnitKeywordState>()
        .expect("unit should have UnitKeywordState")
        .shield_active
}

fn trace(app: &App) -> &[CombatTraceEntry] {
    app.world().resource::<CombatResolutionTrace>().entries()
}

fn trace_index(app: &App, expected: CombatTraceEntry) -> usize {
    trace(app)
        .iter()
        .position(|entry| *entry == expected)
        .expect("expected trace entry should exist")
}

fn damage(
    attacker: Entity,
    defender: Entity,
    damage_amount: u8,
    hp_after: u8,
    blocked: bool,
    sub_step: u8,
) -> CombatTraceEntry {
    CombatTraceEntry::CombatDamage {
        attacker,
        defender,
        damage_amount,
        hp_after,
        was_blocked_by_shield: blocked,
        sub_step,
    }
}

fn keyword(unit: Entity, keyword: KeywordKind, sub_step: u8) -> CombatTraceEntry {
    CombatTraceEntry::KeywordTriggered {
        unit,
        keyword,
        sub_step,
    }
}

fn keyword_count(app: &App, expected: CombatTraceEntry) -> usize {
    trace(app)
        .iter()
        .filter(|entry| **entry == expected)
        .count()
}

#[test]
fn cr_6_and_cr_29_shield_consumed_in_ss3_does_not_block_ss6() {
    let attacker_card = card(10, range_first_strike(3));
    let shield_card = card(11, vec![simple(SimpleKeyword::Shield)]);
    let mut app = app_with_cards(vec![attacker_card, shield_card]);
    let attacker = spawn_unit(
        &mut app,
        CardId(10),
        PLAYER_A,
        1,
        3,
        UnitStats::new(5, 2, 0, 0),
        UnitKeywordState::default(),
    );
    let shielded = spawn_unit(
        &mut app,
        CardId(11),
        PLAYER_B,
        1,
        5,
        UnitStats::new(6, 1, 0, 0),
        UnitKeywordState {
            shield_active: true,
            ..default()
        },
    );

    begin_resolution(&mut app);

    assert_eq!(hp(&app, shielded), 4);
    assert!(!shield_active(&app, shielded));
    assert_eq!(
        keyword_count(&app, keyword(shielded, KeywordKind::Shield, 3)),
        1
    );
    assert!(trace(&app).contains(&damage(attacker, shielded, 0, 6, true, 3)));
    assert!(trace(&app).contains(&damage(attacker, shielded, 2, 4, false, 6)));
}

#[test]
fn cr_7_shield_persists_when_no_attack_lands() {
    let shield_card = card(20, vec![simple(SimpleKeyword::Shield)]);
    let mut app = app_with_cards(vec![shield_card]);
    let shielded = spawn_unit(
        &mut app,
        CardId(20),
        PLAYER_B,
        3,
        5,
        UnitStats::new(6, 1, 0, 0),
        UnitKeywordState {
            shield_active: true,
            ..default()
        },
    );

    begin_resolution(&mut app);

    assert_eq!(hp(&app, shielded), 6);
    assert!(shield_active(&app, shielded));
    assert_eq!(
        keyword_count(&app, keyword(shielded, KeywordKind::Shield, 3)),
        0
    );
    assert_eq!(
        keyword_count(&app, keyword(shielded, KeywordKind::Shield, 6)),
        0
    );
}

#[test]
fn cr_20_range_attack_does_not_trigger_counterattack_at_distance() {
    let ranged_card = card(30, range_first_strike(3));
    let counter_card = card(31, vec![simple(SimpleKeyword::Counterattack)]);
    let mut app = app_with_cards(vec![ranged_card, counter_card]);
    let ranged = spawn_unit(
        &mut app,
        CardId(30),
        PLAYER_A,
        1,
        3,
        UnitStats::new(6, 2, 0, 0),
        UnitKeywordState::default(),
    );
    let counter = spawn_unit(
        &mut app,
        CardId(31),
        PLAYER_B,
        1,
        5,
        UnitStats::new(6, 1, 0, 0),
        UnitKeywordState::default(),
    );

    begin_resolution(&mut app);

    assert!(trace(&app).contains(&damage(ranged, counter, 2, 4, false, 3)));
    assert!(trace(&app).contains(&damage(ranged, counter, 2, 2, false, 6)));
    assert_eq!(
        keyword_count(&app, keyword(counter, KeywordKind::Counterattack, 6)),
        0
    );
    assert_eq!(hp(&app, ranged), 6);
}

#[test]
fn cr_21_counterattack_fires_for_same_cell_melee_contact() {
    let attacker_card = card(40, vec![]);
    let counter_card = card(41, vec![simple(SimpleKeyword::Counterattack)]);
    let mut app = app_with_cards(vec![attacker_card, counter_card]);
    let attacker = spawn_unit(
        &mut app,
        CardId(40),
        PLAYER_A,
        1,
        5,
        UnitStats::new(7, 3, 0, 0),
        UnitKeywordState::default(),
    );
    let counter = spawn_unit(
        &mut app,
        CardId(41),
        PLAYER_B,
        1,
        5,
        UnitStats::new(7, 2, 0, 0),
        UnitKeywordState::default(),
    );

    begin_resolution(&mut app);

    let counterattack_index = trace_index(&app, keyword(counter, KeywordKind::Counterattack, 6));
    let retaliation_index = trace(&app)
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            matches!(
                entry,
                CombatTraceEntry::CombatDamage {
                    attacker: damage_attacker,
                    defender,
                    damage_amount: 2,
                    sub_step: 6,
                    ..
                } if *damage_attacker == counter && *defender == attacker
            )
        })
        .map(|(index, _)| index)
        .last()
        .expect("counterattack retaliation damage should be logged");

    assert!(counterattack_index < retaliation_index);
    assert_eq!(hp(&app, attacker), 3);
    assert_eq!(hp(&app, counter), 4);
}

#[test]
fn shield_absorption_does_not_suppress_counterattack() {
    let attacker_card = card(45, vec![]);
    let shield_counter_card = card(
        46,
        vec![
            simple(SimpleKeyword::Shield),
            simple(SimpleKeyword::Counterattack),
        ],
    );
    let mut app = app_with_cards(vec![attacker_card, shield_counter_card]);
    let attacker = spawn_unit(
        &mut app,
        CardId(45),
        PLAYER_A,
        1,
        5,
        UnitStats::new(6, 3, 0, 0),
        UnitKeywordState::default(),
    );
    let shield_counter = spawn_unit(
        &mut app,
        CardId(46),
        PLAYER_B,
        1,
        5,
        UnitStats::new(7, 2, 0, 0),
        UnitKeywordState {
            shield_active: true,
            ..default()
        },
    );

    begin_resolution(&mut app);

    let shield_index = trace_index(&app, keyword(shield_counter, KeywordKind::Shield, 6));
    let counterattack_index =
        trace_index(&app, keyword(shield_counter, KeywordKind::Counterattack, 6));
    let retaliation_index = trace(&app)
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            matches!(
                entry,
                CombatTraceEntry::CombatDamage {
                    attacker: damage_attacker,
                    defender,
                    damage_amount: 2,
                    sub_step: 6,
                    ..
                } if *damage_attacker == shield_counter && *defender == attacker
            )
        })
        .map(|(index, _)| index)
        .last()
        .expect("counterattack retaliation damage should be logged");

    assert!(trace(&app).contains(&damage(attacker, shield_counter, 0, 7, true, 6)));
    assert!(shield_index < counterattack_index);
    assert!(counterattack_index < retaliation_index);
    assert_eq!(hp(&app, shield_counter), 7);
    assert_eq!(hp(&app, attacker), 2);
}

#[test]
fn cr_35_counterattack_fires_for_collision_halt_adjacency() {
    let attacker_card = card(50, vec![]);
    let counter_card = card(51, vec![simple(SimpleKeyword::Counterattack)]);
    let mut app = app_with_cards(vec![attacker_card, counter_card]);
    let attacker = spawn_unit(
        &mut app,
        CardId(50),
        PLAYER_A,
        1,
        4,
        UnitStats::new(7, 3, 2, 0),
        UnitKeywordState::default(),
    );
    let counter = spawn_unit(
        &mut app,
        CardId(51),
        PLAYER_B,
        1,
        5,
        UnitStats::new(7, 2, 2, 0),
        UnitKeywordState::default(),
    );

    begin_resolution(&mut app);

    assert_eq!(
        app.world()
            .entity(attacker)
            .get::<BoardPosition>()
            .unwrap()
            .cell,
        4
    );
    assert_eq!(
        app.world()
            .entity(counter)
            .get::<BoardPosition>()
            .unwrap()
            .cell,
        5
    );
    assert!(trace(&app).contains(&keyword(counter, KeywordKind::Counterattack, 6)));
    assert_eq!(hp(&app, attacker), 3);
    assert_eq!(hp(&app, counter), 4);
}

#[test]
fn cr_36_shield_absorbs_two_first_strike_sources_once() {
    let lane_2_attacker_card = card(60, range_first_strike(3));
    let lane_4_attacker_card = card(61, range_first_strike(3));
    let shield_card = card(62, vec![simple(SimpleKeyword::Shield)]);
    let mut app = app_with_cards(vec![
        lane_2_attacker_card,
        lane_4_attacker_card,
        shield_card,
    ]);
    let lane_2_attacker = spawn_unit(
        &mut app,
        CardId(60),
        PLAYER_A,
        2,
        3,
        UnitStats::new(5, 2, 0, 0),
        UnitKeywordState::default(),
    );
    let lane_4_attacker = spawn_unit(
        &mut app,
        CardId(61),
        PLAYER_A,
        4,
        3,
        UnitStats::new(5, 2, 0, 0),
        UnitKeywordState::default(),
    );
    let shielded = spawn_unit(
        &mut app,
        CardId(62),
        PLAYER_B,
        3,
        5,
        UnitStats::new(10, 1, 0, 0),
        UnitKeywordState {
            shield_active: true,
            ..default()
        },
    );

    begin_resolution(&mut app);

    assert_eq!(
        keyword_count(&app, keyword(shielded, KeywordKind::Shield, 3)),
        1
    );
    assert!(trace(&app).contains(&damage(lane_2_attacker, shielded, 0, 10, true, 3)));
    assert!(trace(&app).contains(&damage(lane_4_attacker, shielded, 0, 10, true, 3)));
    assert!(!shield_active(&app, shielded));
}

fn grid_indices(lane: u8, cell: u8) -> Option<(usize, usize)> {
    if !(1..=5).contains(&lane) || !(1..=8).contains(&cell) {
        return None;
    }
    Some((usize::from(lane - 1), usize::from(cell - 1)))
}
