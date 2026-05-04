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
        mp: 1,
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

fn cell(app: &App, unit: Entity) -> u8 {
    app.world()
        .entity(unit)
        .get::<BoardPosition>()
        .expect("unit should have a board position")
        .cell
}

fn trace(app: &App) -> &[CombatTraceEntry] {
    app.world().resource::<CombatResolutionTrace>().entries()
}

fn moved(unit: Entity, from_cell: u8, to_cell: u8, sub_step: u8) -> CombatTraceEntry {
    CombatTraceEntry::UnitMoved {
        unit,
        from_cell,
        to_cell,
        sub_step,
    }
}

#[test]
fn cr_5_stun_suppresses_charge_x_and_standard_movement() {
    let stunned_card = card(10, vec![Keyword::ChargeXMove { cells: 2 }]);
    let mut app = app_with_cards(vec![stunned_card]);
    let stunned = spawn_unit(
        &mut app,
        CardId(10),
        PLAYER_A,
        1,
        3,
        UnitStats::new(3, 1, 2, 0),
        UnitKeywordState {
            stun_active: true,
            ..default()
        },
    );

    begin_resolution(&mut app);

    assert_eq!(cell(&app, stunned), 3);
    assert!(!trace(&app).iter().any(|entry| {
        matches!(entry, CombatTraceEntry::UnitMoved { unit, .. } if *unit == stunned)
    }));
}

#[test]
fn cr_8_advancing_unit_halts_on_enemy_wall_cell() {
    let attacker_card = card(20, vec![]);
    let wall_card = card(21, vec![simple(SimpleKeyword::Wall)]);
    let mut app = app_with_cards(vec![attacker_card, wall_card]);
    let attacker = spawn_unit(
        &mut app,
        CardId(20),
        PLAYER_A,
        1,
        3,
        UnitStats::new(3, 2, 3, 0),
        UnitKeywordState::default(),
    );
    let wall = spawn_unit(
        &mut app,
        CardId(21),
        PLAYER_B,
        1,
        5,
        UnitStats::new(4, 0, 0, 0),
        UnitKeywordState::default(),
    );

    begin_resolution(&mut app);

    assert_eq!(cell(&app, attacker), 5);
    assert_eq!(cell(&app, wall), 5);
    assert!(trace(&app).contains(&moved(attacker, 3, 5, 5)));
}

#[test]
fn cr_9_path_crossing_halts_both_units_at_previous_cells() {
    let player_a_card = card(30, vec![]);
    let player_b_card = card(31, vec![]);
    let mut app = app_with_cards(vec![player_a_card, player_b_card]);
    let unit_a = spawn_unit(
        &mut app,
        CardId(30),
        PLAYER_A,
        1,
        4,
        UnitStats::new(3, 2, 2, 0),
        UnitKeywordState::default(),
    );
    let unit_b = spawn_unit(
        &mut app,
        CardId(31),
        PLAYER_B,
        1,
        5,
        UnitStats::new(3, 2, 2, 0),
        UnitKeywordState::default(),
    );

    begin_resolution(&mut app);

    assert_eq!(cell(&app, unit_a), 4);
    assert_eq!(cell(&app, unit_b), 5);
    assert!(!trace(&app).contains(&moved(unit_a, 4, 5, 5)));
    assert!(!trace(&app).contains(&moved(unit_b, 5, 4, 5)));
}

#[test]
fn cr_31_charge_x_and_standard_movement_are_separate_passes() {
    let charge_card = card(40, vec![Keyword::ChargeXMove { cells: 2 }]);
    let mut app = app_with_cards(vec![charge_card]);
    let charged = spawn_unit(
        &mut app,
        CardId(40),
        PLAYER_A,
        1,
        2,
        UnitStats::new(3, 2, 1, 0),
        UnitKeywordState::default(),
    );

    begin_resolution(&mut app);

    assert_eq!(cell(&app, charged), 5);
    assert!(trace(&app).contains(&moved(charged, 2, 4, 2)));
    assert!(trace(&app).contains(&moved(charged, 4, 5, 5)));
}

#[test]
fn cr_44_range_unit_with_wall_in_range_does_not_advance_toward_wall() {
    let range_card = card(50, vec![Keyword::RangeX { max_range: 3 }]);
    let wall_card = card(51, vec![simple(SimpleKeyword::Wall)]);
    let mut app = app_with_cards(vec![range_card, wall_card]);
    let ranged = spawn_unit(
        &mut app,
        CardId(50),
        PLAYER_A,
        1,
        2,
        UnitStats::new(3, 2, 3, 0),
        UnitKeywordState::default(),
    );
    let wall = spawn_unit(
        &mut app,
        CardId(51),
        PLAYER_B,
        1,
        4,
        UnitStats::new(4, 0, 0, 0),
        UnitKeywordState::default(),
    );

    begin_resolution(&mut app);

    assert_eq!(cell(&app, ranged), 2);
    assert_eq!(cell(&app, wall), 4);
    assert!(!trace(&app)
        .iter()
        .any(|entry| matches!(entry, CombatTraceEntry::UnitMoved { unit, .. } if *unit == ranged)));
}

fn grid_indices(lane: u8, cell: u8) -> Option<(usize, usize)> {
    if !(1..=5).contains(&lane) || !(1..=8).contains(&cell) {
        return None;
    }
    Some((usize::from(lane - 1), usize::from(cell - 1)))
}
