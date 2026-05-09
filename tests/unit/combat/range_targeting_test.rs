use std::collections::HashMap;

use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use server::core::rsm::BeginResolution;
use server::core::session::SessionConfig;
use server::feature::board::{
    BoardCell, BoardConfig, BoardGrid, BoardOccupancy, PlacementCommitted,
};
use server::feature::combat::{CombatPlugin, CombatResolutionTrace, CombatTraceEntry};
use server::feature::keyword::components::UnitKeywordState;
use server::foundation::config::CardCatalog;
use server::foundation::rng::{RngEvent, ServerRng};
use shared::card::{CardData, CardId, CardType, ClassId, Keyword, Rarity, SimpleKeyword, UnitType};
use shared::protocol::GameMode;
use shared::session::PlayerId;

const ROUND: u32 = 5;
const PLAYER_A: PlayerId = PlayerId(1);
const PLAYER_B: PlayerId = PlayerId(2);
const RANGE_SEED: u64 = 0x5EED_0008;

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

fn range_keyword(range: u8) -> Vec<Keyword> {
    vec![Keyword::RangeX { max_range: range }]
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

fn app_with_cards(cards: Vec<CardData>, seed: u64) -> App {
    let mut app = App::new();
    app.add_plugins(CombatPlugin);
    app.add_message::<BeginResolution>();
    app.add_message::<PlacementCommitted>();
    app.insert_resource(BoardConfig::default());
    app.insert_resource(BoardGrid::default());
    app.insert_resource(BoardOccupancy::default());
    app.insert_resource(session_config());
    app.insert_resource(ServerRng::from_seed(seed));
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
        .expect("unit should have UnitStats")
        .hp
}

fn cell(app: &App, unit: Entity) -> u8 {
    app.world()
        .entity(unit)
        .get::<BoardPosition>()
        .expect("unit should have BoardPosition")
        .cell
}

fn trace(app: &App) -> &[CombatTraceEntry] {
    app.world().resource::<CombatResolutionTrace>().entries()
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

fn range_select_events(app: &App) -> Vec<RngEvent> {
    app.world()
        .resource::<ServerRng>()
        .audit_log()
        .iter()
        .filter_map(|entry| match entry.event_type {
            RngEvent::RangeEquidistantSelect { .. } => Some(entry.event_type.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn test_cr_3_single_nearest_range_target_attacks_without_rng_consumption() {
    let range_card = card(10, range_keyword(3));
    let near_card = card(11, vec![]);
    let far_card = card(12, vec![]);
    let mut app = app_with_cards(vec![range_card, near_card, far_card], RANGE_SEED);
    let attacker = spawn_unit(
        &mut app,
        CardId(10),
        PLAYER_A,
        1,
        2,
        UnitStats::new(5, 2, 0, 0),
    );
    let nearest = spawn_unit(
        &mut app,
        CardId(11),
        PLAYER_B,
        1,
        4,
        UnitStats::new(5, 1, 0, 0),
    );
    let farther = spawn_unit(
        &mut app,
        CardId(12),
        PLAYER_B,
        1,
        5,
        UnitStats::new(5, 1, 0, 0),
    );

    begin_resolution(&mut app);

    assert_eq!(cell(&app, attacker), 2);
    assert!(trace(&app).contains(&damage(attacker, nearest, 2, 3, 6)));
    assert_eq!(hp(&app, farther), 5);
    assert!(range_select_events(&app).is_empty());
    assert_eq!(app.world().resource::<ServerRng>().current_seed_index(), 1);
}

#[test]
fn test_cr_3_equidistant_range_target_consumes_one_seed_and_is_deterministic() {
    let range_card = card(20, range_keyword(3));
    let target_a_card = card(41, vec![]);
    let target_b_card = card(42, vec![]);
    let mut expected_rng = ServerRng::from_seed(RANGE_SEED);
    let expected_index = expected_rng.range_equidistant_select(PLAYER_A.0 as u32, 1) as usize % 2;

    let mut first_run = app_with_cards(
        vec![
            range_card.clone(),
            target_a_card.clone(),
            target_b_card.clone(),
        ],
        RANGE_SEED,
    );
    let attacker = spawn_unit(
        &mut first_run,
        CardId(20),
        PLAYER_A,
        1,
        2,
        UnitStats::new(5, 2, 0, 0),
    );
    let mut ordered_targets = [
        spawn_unit(
            &mut first_run,
            CardId(41),
            PLAYER_B,
            1,
            4,
            UnitStats::new(5, 1, 0, 0),
        ),
        spawn_unit(
            &mut first_run,
            CardId(42),
            PLAYER_B,
            1,
            4,
            UnitStats::new(5, 1, 0, 0),
        ),
    ];
    ordered_targets.sort_by_key(|target| target.to_bits());
    let expected_target = ordered_targets[expected_index];

    begin_resolution(&mut first_run);

    assert!(trace(&first_run).contains(&damage(attacker, expected_target, 2, 3, 6)));
    assert_eq!(
        range_select_events(&first_run),
        vec![RngEvent::RangeEquidistantSelect {
            player_id: PLAYER_A.0 as u32,
            lane: 1
        }]
    );
    assert_eq!(
        first_run
            .world()
            .resource::<ServerRng>()
            .current_seed_index(),
        2
    );

    let mut second_run = app_with_cards(vec![range_card, target_a_card, target_b_card], RANGE_SEED);
    let second_attacker = spawn_unit(
        &mut second_run,
        CardId(20),
        PLAYER_A,
        1,
        2,
        UnitStats::new(5, 2, 0, 0),
    );
    let mut second_targets = [
        spawn_unit(
            &mut second_run,
            CardId(41),
            PLAYER_B,
            1,
            4,
            UnitStats::new(5, 1, 0, 0),
        ),
        spawn_unit(
            &mut second_run,
            CardId(42),
            PLAYER_B,
            1,
            4,
            UnitStats::new(5, 1, 0, 0),
        ),
    ];
    second_targets.sort_by_key(|target| target.to_bits());

    begin_resolution(&mut second_run);

    assert!(trace(&second_run).contains(&damage(
        second_attacker,
        second_targets[expected_index],
        2,
        3,
        6
    )));
    assert_eq!(
        range_select_events(&second_run),
        range_select_events(&first_run)
    );
}

#[test]
fn test_cr_4_range_first_strike_logs_ss3_and_ss6_damage() {
    let range_first_striker = card(50, range_first_strike(2));
    let defender_card = card(51, vec![]);
    let mut app = app_with_cards(vec![range_first_striker, defender_card], RANGE_SEED);
    let attacker = spawn_unit(
        &mut app,
        CardId(50),
        PLAYER_A,
        1,
        3,
        UnitStats::new(5, 1, 0, 0),
    );
    let defender = spawn_unit(
        &mut app,
        CardId(51),
        PLAYER_B,
        1,
        5,
        UnitStats::new(5, 1, 0, 0),
    );

    begin_resolution(&mut app);

    assert!(trace(&app).contains(&damage(attacker, defender, 1, 4, 3)));
    assert!(trace(&app).contains(&damage(attacker, defender, 1, 3, 6)));
}

#[test]
fn test_cr_28_range_targeting_ignores_enemy_behind_attacker() {
    let range_card = card(60, range_keyword(3));
    let behind_card = card(61, vec![]);
    let forward_card = card(62, vec![]);
    let mut app = app_with_cards(vec![range_card, behind_card, forward_card], RANGE_SEED);
    let attacker = spawn_unit(
        &mut app,
        CardId(60),
        PLAYER_A,
        1,
        4,
        UnitStats::new(5, 2, 0, 0),
    );
    let behind = spawn_unit(
        &mut app,
        CardId(61),
        PLAYER_B,
        1,
        2,
        UnitStats::new(5, 1, 0, 0),
    );
    let forward = spawn_unit(
        &mut app,
        CardId(62),
        PLAYER_B,
        1,
        6,
        UnitStats::new(5, 1, 0, 0),
    );

    begin_resolution(&mut app);

    assert_eq!(hp(&app, behind), 5);
    assert!(trace(&app).contains(&damage(attacker, forward, 2, 3, 6)));
}

#[test]
fn test_cr_44_range_attacks_wall_from_current_cell_after_ss5_exemption() {
    let range_card = card(70, range_keyword(3));
    let wall_card = card(71, vec![simple(SimpleKeyword::Wall)]);
    let mut app = app_with_cards(vec![range_card, wall_card], RANGE_SEED);
    let ranged = spawn_unit(
        &mut app,
        CardId(70),
        PLAYER_A,
        1,
        2,
        UnitStats::new(5, 2, 3, 0),
    );
    let wall = spawn_unit(
        &mut app,
        CardId(71),
        PLAYER_B,
        1,
        4,
        UnitStats::new(5, 0, 0, 0),
    );

    begin_resolution(&mut app);

    assert_eq!(cell(&app, ranged), 2);
    assert!(trace(&app).contains(&damage(ranged, wall, 2, 3, 6)));
    assert!(!trace(&app)
        .iter()
        .any(|entry| matches!(entry, CombatTraceEntry::UnitMoved { unit, .. } if *unit == ranged)));
}

#[test]
fn test_cr_45_ss6_reacquires_after_ss3_target_is_removed() {
    let range_first_striker = card(80, range_first_strike(3));
    let first_target_card = card(81, vec![]);
    let second_target_card = card(82, vec![]);
    let mut app = app_with_cards(
        vec![range_first_striker, first_target_card, second_target_card],
        RANGE_SEED,
    );
    let attacker = spawn_unit(
        &mut app,
        CardId(80),
        PLAYER_A,
        1,
        3,
        UnitStats::new(5, 3, 0, 0),
    );
    let ss3_target = spawn_unit(
        &mut app,
        CardId(81),
        PLAYER_B,
        1,
        4,
        UnitStats::new(1, 1, 0, 0),
    );
    let ss6_target = spawn_unit(
        &mut app,
        CardId(82),
        PLAYER_B,
        1,
        6,
        UnitStats::new(5, 1, 0, 0),
    );

    begin_resolution(&mut app);

    assert!(trace(&app).contains(&damage(attacker, ss3_target, 3, 0, 3)));
    assert!(trace(&app).contains(&CombatTraceEntry::UnitRemoved {
        unit: ss3_target,
        lane: 1,
        cell: 4,
    }));
    assert!(trace(&app).contains(&damage(attacker, ss6_target, 3, 2, 6)));
}

fn grid_indices(lane: u8, cell: u8) -> Option<(usize, usize)> {
    if !(1..=5).contains(&lane) || !(1..=8).contains(&cell) {
        return None;
    }
    Some((usize::from(lane - 1), usize::from(cell - 1)))
}
