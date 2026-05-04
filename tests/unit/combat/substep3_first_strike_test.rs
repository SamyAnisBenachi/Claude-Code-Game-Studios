use std::collections::HashMap;

use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use server::core::rsm::BeginResolution;
use server::core::session::SessionConfig;
use server::feature::board::{BoardCell, BoardConfig, BoardGrid, BoardOccupancy};
use server::feature::combat::{
    CombatKillLog, CombatPlugin, CombatResolutionTrace, CombatTraceEntry, KillRecord,
};
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

fn first_strike_keywords() -> Vec<Keyword> {
    vec![simple(SimpleKeyword::FirstStrike)]
}

fn range_first_strike_keywords(range: u8) -> Vec<Keyword> {
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
) -> Entity {
    let entity = app
        .world_mut()
        .spawn((
            UnitCardRef(card_id),
            UnitOwner(owner),
            stats,
            UnitKeywordState::default(),
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
        .expect("unit should have stats")
        .hp
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
    sub_step: u8,
) -> CombatTraceEntry {
    CombatTraceEntry::CombatDamage {
        attacker,
        defender,
        damage_amount,
        hp_after,
        was_blocked_by_shield: false,
        sub_step,
    }
}

fn kill_log(app: &App) -> &[KillRecord] {
    app.world().resource::<CombatKillLog>().records()
}

#[test]
fn cr_1_first_strike_damage_resolves_before_standard_movement() {
    let first_striker = card(10, first_strike_keywords());
    let defender_card = card(11, vec![]);
    let mut app = app_with_cards(vec![first_striker, defender_card]);
    let attacker = spawn_unit(
        &mut app,
        CardId(10),
        PLAYER_A,
        1,
        4,
        UnitStats::new(5, 3, 2, 0),
    );
    let defender = spawn_unit(
        &mut app,
        CardId(11),
        PLAYER_B,
        1,
        4,
        UnitStats::new(5, 1, 0, 0),
    );

    begin_resolution(&mut app);

    assert_eq!(hp(&app, defender), 2);
    let damage_index = trace_index(&app, damage(attacker, defender, 3, 2, 3));
    let movement_index = trace_index(&app, CombatTraceEntry::SubStepStarted(5));
    assert!(damage_index < movement_index);
}

#[test]
fn cr_2_mutual_first_strike_damage_uses_pre_damage_snapshots() {
    let first_striker_a = card(20, first_strike_keywords());
    let first_striker_b = card(21, first_strike_keywords());
    let mut app = app_with_cards(vec![first_striker_a, first_striker_b]);
    let unit_a = spawn_unit(
        &mut app,
        CardId(20),
        PLAYER_A,
        1,
        4,
        UnitStats::new(2, 3, 0, 0),
    );
    let unit_b = spawn_unit(
        &mut app,
        CardId(21),
        PLAYER_B,
        1,
        4,
        UnitStats::new(2, 3, 0, 0),
    );

    begin_resolution(&mut app);

    assert_eq!(hp(&app, unit_a), 0);
    assert_eq!(hp(&app, unit_b), 0);
    assert!(trace(&app).contains(&damage(unit_a, unit_b, 3, 0, 3)));
    assert!(trace(&app).contains(&damage(unit_b, unit_a, 3, 0, 3)));
    assert_eq!(kill_log(&app).len(), 2);
    assert!(app.world().get_entity(unit_a).is_ok());
    assert!(app.world().get_entity(unit_b).is_ok());
}

#[test]
fn cr_4_range_first_strike_logs_distinct_ss3_and_ss6_damage() {
    let ranged_first_striker = card(30, range_first_strike_keywords(2));
    let defender_card = card(31, vec![]);
    let mut app = app_with_cards(vec![ranged_first_striker, defender_card]);
    let attacker = spawn_unit(
        &mut app,
        CardId(30),
        PLAYER_A,
        1,
        3,
        UnitStats::new(5, 1, 0, 0),
    );
    let defender = spawn_unit(
        &mut app,
        CardId(31),
        PLAYER_B,
        2,
        5,
        UnitStats::new(5, 1, 0, 0),
    );

    begin_resolution(&mut app);

    assert_eq!(hp(&app, defender), 3);
    assert!(trace(&app).contains(&damage(attacker, defender, 1, 4, 3)));
    assert!(trace(&app).contains(&damage(attacker, defender, 1, 3, 6)));
}

#[test]
fn cr_22_final_blow_fires_in_ss3_before_dead_unit_removal() {
    let final_blow_first_striker = card(
        40,
        vec![
            simple(SimpleKeyword::FirstStrike),
            simple(SimpleKeyword::FinalBlow),
        ],
    );
    let defender_card = card(41, vec![]);
    let mut app = app_with_cards(vec![final_blow_first_striker, defender_card]);
    let attacker = spawn_unit(
        &mut app,
        CardId(40),
        PLAYER_A,
        1,
        4,
        UnitStats::new(5, 5, 0, 0),
    );
    let defender = spawn_unit(
        &mut app,
        CardId(41),
        PLAYER_B,
        1,
        4,
        UnitStats::new(3, 1, 0, 0),
    );

    begin_resolution(&mut app);

    let damage_index = trace_index(&app, damage(attacker, defender, 5, 0, 3));
    let final_blow_index = trace_index(
        &app,
        CombatTraceEntry::KeywordTriggered {
            unit: attacker,
            keyword: KeywordKind::FinalBlow,
            sub_step: 3,
        },
    );
    let substep4_index = trace_index(&app, CombatTraceEntry::SubStepStarted(4));

    assert!(damage_index < final_blow_index);
    assert!(final_blow_index < substep4_index);
    assert_eq!(hp(&app, defender), 0);
    assert!(app.world().get_entity(defender).is_ok());
    assert!(app
        .world()
        .entity(defender)
        .get::<BoardPosition>()
        .is_some());
    assert_eq!(kill_log(&app)[0].killer, attacker);
}

#[test]
fn cr_37_multisource_first_strike_damage_uses_attacker_lane_order_for_kill_credit() {
    let lane_2_attacker = card(50, range_first_strike_keywords(2));
    let lane_4_attacker = card(51, range_first_strike_keywords(2));
    let defender_card = card(52, vec![]);
    let mut app = app_with_cards(vec![lane_2_attacker, lane_4_attacker, defender_card]);
    let attacker_lane_2 = spawn_unit(
        &mut app,
        CardId(50),
        PLAYER_A,
        2,
        2,
        UnitStats::new(5, 3, 0, 0),
    );
    let attacker_lane_4 = spawn_unit(
        &mut app,
        CardId(51),
        PLAYER_A,
        4,
        2,
        UnitStats::new(5, 3, 0, 0),
    );
    let defender = spawn_unit(
        &mut app,
        CardId(52),
        PLAYER_B,
        3,
        3,
        UnitStats::new(4, 1, 0, 0),
    );

    begin_resolution(&mut app);

    let lane_2_damage = trace_index(&app, damage(attacker_lane_2, defender, 3, 1, 3));
    let lane_4_damage = trace_index(&app, damage(attacker_lane_4, defender, 3, 0, 3));
    assert!(lane_2_damage < lane_4_damage);
    assert_eq!(hp(&app, defender), 0);
    assert_eq!(
        kill_log(&app),
        &[KillRecord {
            killer: attacker_lane_4,
            victim: defender,
            killer_player_id: PLAYER_A,
            lethal_sub_step: 3,
        }]
    );
}

fn grid_indices(lane: u8, cell: u8) -> Option<(usize, usize)> {
    if !(1..=5).contains(&lane) || !(1..=8).contains(&cell) {
        return None;
    }
    Some((usize::from(lane - 1), usize::from(cell - 1)))
}
