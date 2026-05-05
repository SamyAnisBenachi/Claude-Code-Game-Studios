use std::collections::HashMap;

use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::rsm::BeginResolution;
use server::core::session::SessionConfig;
use server::feature::board::{BoardCell, BoardConfig, BoardGrid, BoardOccupancy};
use server::feature::combat::{
    CombatKillLog, CombatPlugin, CombatResolutionTrace, CombatTraceEntry, GoldAwardReason,
};
use server::feature::keyword::components::UnitKeywordState;
use server::feature::keyword::{ChainDeathBuffer, UnitDied};
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Keyword, Rarity, SimpleKeyword, UnitType};
use shared::keyword::KeywordKind;
use shared::protocol::GameMode;
use shared::session::PlayerId;

const ROUND: u32 = 5;
const PLAYER_A: PlayerId = PlayerId(1);
const PLAYER_B: PlayerId = PlayerId(2);

#[derive(Resource)]
struct DeathChainVictim(Entity);

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

fn final_blow_range_first_strike_keywords(range: u8) -> Vec<Keyword> {
    vec![
        Keyword::RangeX { max_range: range },
        simple(SimpleKeyword::FirstStrike),
        simple(SimpleKeyword::FinalBlow),
    ]
}

fn death_keywords() -> Vec<Keyword> {
    vec![simple(SimpleKeyword::Death)]
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

fn economy(gold: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold,
        current_mana: 0,
        reserve_mana: 0,
        mana_cap: 10,
        reserved_gold: 0,
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
    app.insert_resource(PlayerEconomies(HashMap::from([
        (PLAYER_A, economy(0)),
        (PLAYER_B, economy(0)),
    ])));
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

fn trace(app: &App) -> &[CombatTraceEntry] {
    app.world().resource::<CombatResolutionTrace>().entries()
}

fn trace_index(app: &App, expected: CombatTraceEntry) -> usize {
    trace(app)
        .iter()
        .position(|entry| *entry == expected)
        .expect("expected trace entry should exist")
}

fn unit_removed(unit: Entity, lane: u8, cell: u8) -> CombatTraceEntry {
    CombatTraceEntry::UnitRemoved { unit, lane, cell }
}

fn gold_awarded(player: PlayerId) -> CombatTraceEntry {
    CombatTraceEntry::GoldAwarded {
        player,
        amount: 1,
        reason: GoldAwardReason::Kill,
    }
}

fn keyword_triggered(unit: Entity, keyword: KeywordKind, sub_step: u8) -> CombatTraceEntry {
    CombatTraceEntry::KeywordTriggered {
        unit,
        keyword,
        sub_step,
    }
}

fn death_chain_observer(
    trigger: On<UnitDied>,
    victim: Res<DeathChainVictim>,
    mut units: Query<&mut UnitStats>,
    mut chain_death_buffer: ResMut<ChainDeathBuffer>,
) {
    if trigger.entity == victim.0 {
        return;
    }
    let Ok(mut stats) = units.get_mut(victim.0) else {
        return;
    };

    stats.hp = 0;
    chain_death_buffer
        .0
        .push_back((victim.0, Some(trigger.entity)));
}

#[test]
fn cr_16_ss4_removes_dead_unit_and_awards_kill_gold() {
    let first_striker = card(10, first_strike_keywords());
    let defender_card = card(11, vec![]);
    let mut app = app_with_cards(vec![first_striker, defender_card]);
    let _attacker = spawn_unit(
        &mut app,
        CardId(10),
        PLAYER_A,
        1,
        4,
        UnitStats::new(3, 5, 0, 0),
    );
    let defender = spawn_unit(
        &mut app,
        CardId(11),
        PLAYER_B,
        1,
        4,
        UnitStats::new(1, 0, 0, 0),
    );

    begin_resolution(&mut app);

    assert!(app.world().get_entity(defender).is_err());
    assert!(trace(&app).contains(&unit_removed(defender, 1, 4)));
    assert!(trace(&app).contains(&gold_awarded(PLAYER_A)));
    assert_eq!(
        app.world().resource::<PlayerEconomies>().0[&PLAYER_A].gold,
        1
    );
    assert!(app.world().resource::<CombatKillLog>().records().is_empty());
}

#[test]
fn cr_23_final_blow_remains_in_kill_sub_step_not_ss4() {
    let final_blow_attacker = card(20, final_blow_range_first_strike_keywords(2));
    let defender_card = card(21, vec![]);
    let mut app = app_with_cards(vec![final_blow_attacker, defender_card]);
    let attacker = spawn_unit(
        &mut app,
        CardId(20),
        PLAYER_A,
        1,
        3,
        UnitStats::new(3, 1, 0, 0),
    );
    let _defender = spawn_unit(
        &mut app,
        CardId(21),
        PLAYER_B,
        1,
        5,
        UnitStats::new(2, 0, 0, 0),
    );

    begin_resolution(&mut app);

    assert!(trace(&app).contains(&keyword_triggered(attacker, KeywordKind::FinalBlow, 6)));
    assert!(!trace(&app).contains(&keyword_triggered(attacker, KeywordKind::FinalBlow, 4)));
}

#[test]
fn cr_25_death_chain_drains_sequentially_through_chain_buffer() {
    let death_source_card = card(30, death_keywords());
    let death_victim_card = card(31, death_keywords());
    let mut app = app_with_cards(vec![death_source_card, death_victim_card]);
    app.add_observer(death_chain_observer);

    let source = spawn_unit(
        &mut app,
        CardId(30),
        PLAYER_A,
        1,
        3,
        UnitStats::new(0, 0, 0, 0),
    );
    let chained = spawn_unit(
        &mut app,
        CardId(31),
        PLAYER_B,
        2,
        4,
        UnitStats::new(1, 0, 0, 0),
    );
    app.insert_resource(DeathChainVictim(chained));

    begin_resolution(&mut app);

    let source_removed = trace_index(&app, unit_removed(source, 1, 3));
    let source_death = trace_index(&app, keyword_triggered(source, KeywordKind::Death, 4));
    let chained_removed = trace_index(&app, unit_removed(chained, 2, 4));
    let chained_death = trace_index(&app, keyword_triggered(chained, KeywordKind::Death, 4));

    assert!(source_removed < source_death);
    assert!(source_death < chained_removed);
    assert!(chained_removed < chained_death);
    assert!(app.world().get_entity(source).is_err());
    assert!(app.world().get_entity(chained).is_err());
    assert!(app.world().resource::<ChainDeathBuffer>().0.is_empty());
}

fn grid_indices(lane: u8, cell: u8) -> Option<(usize, usize)> {
    if !(1..=5).contains(&lane) || !(1..=8).contains(&cell) {
        return None;
    }
    Some((usize::from(lane - 1), usize::from(cell - 1)))
}
