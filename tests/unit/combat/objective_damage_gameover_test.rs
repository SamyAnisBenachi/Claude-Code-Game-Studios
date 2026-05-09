use std::collections::HashMap;

use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::objective_contract::ObjectiveCounters;
use server::core::rsm::{
    BeginResolution, GameOverEmitted, ResolutionComplete, RoundPhase, RoundState, RsmPlugin,
};
use server::core::session::SessionConfig;
use server::feature::board::{
    BoardCell, BoardConfig, BoardGrid, BoardOccupancy, PlacementCommitted,
};
use server::feature::combat::{
    CombatPlugin, CombatResolutionTrace, CombatTraceEntry, GoldAwardReason,
};
use server::feature::keyword::components::UnitKeywordState;
use server::feature::objective::{
    HiddenObjectives, ObjectiveHp, ObjectiveSlot, PendingObjectiveEvents,
};
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Keyword, Rarity, SimpleKeyword, UnitType};
use shared::protocol::{GameMode, GameOverReason};
use shared::session::PlayerId;

const ROUND: u32 = 5;
const PLAYER_A: PlayerId = PlayerId(1);
const PLAYER_B: PlayerId = PlayerId(2);
const ATTACKER_CARD: CardId = CardId(10);
const FIRST_STRIKE_CARD: CardId = CardId(11);
const DEFENDER_CARD: CardId = CardId(12);

fn card(id: CardId, keywords: Vec<Keyword>) -> CardData {
    CardData {
        id,
        name_fr: format!("Carte {}", id.0),
        name_en: format!("Card {}", id.0),
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
        art_id: format!("test_{}", id.0),
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
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn app_with_combat() -> App {
    let mut app = App::new();
    app.add_plugins(CombatPlugin);
    app.add_message::<BeginResolution>();
    app.add_message::<PlacementCommitted>();
    insert_common_resources(&mut app);
    app
}

fn app_with_rsm_and_combat() -> App {
    let mut app = App::new();
    app.add_plugins((RsmPlugin, CombatPlugin));
    app.add_message::<PlacementCommitted>();
    insert_common_resources(&mut app);
    app.insert_resource(Time::<()>::default());
    *app.world_mut().resource_mut::<RoundState>() = RoundState {
        phase: RoundPhase::Resolution,
        round_number: ROUND,
        ..RoundState::new()
    };
    app
}

fn insert_common_resources(app: &mut App) {
    app.insert_resource(BoardConfig::default());
    app.insert_resource(BoardGrid::default());
    app.insert_resource(BoardOccupancy::default());
    app.insert_resource(session_config());
    app.insert_resource(CardCatalog {
        cards: [
            card(ATTACKER_CARD, vec![]),
            card(FIRST_STRIKE_CARD, vec![simple(SimpleKeyword::FirstStrike)]),
            card(DEFENDER_CARD, vec![]),
        ]
        .into_iter()
        .map(|card| (card.id, card))
        .collect(),
    });
    app.insert_resource(HiddenObjectives::default());
    app.insert_resource(ObjectiveCounters::default());
    app.insert_resource(PendingObjectiveEvents::default());
    app.insert_resource(PlayerEconomies(HashMap::from([
        (PLAYER_A, economy(0)),
        (PLAYER_B, economy(0)),
    ])));
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

fn spawn_objective(app: &mut App, owner: PlayerId, lane: u8, hp: u32, was_fake: bool) {
    app.world_mut().spawn((
        ObjectiveSlot {
            lane,
            player: owner,
            destroyed: false,
        },
        ObjectiveHp { hp },
    ));
    app.world_mut()
        .resource_mut::<HiddenObjectives>()
        .identities
        .insert((owner, lane), was_fake);
}

fn begin_resolution(app: &mut App) {
    app.world_mut()
        .write_message(BeginResolution { round: ROUND });
    app.update();
}

fn finish_resolution(app: &mut App) {
    app.update();
}

fn objective_hp(app: &mut App, owner: PlayerId, lane: u8) -> u32 {
    let world = app.world_mut();
    let mut query = world.query::<(&ObjectiveSlot, &ObjectiveHp)>();
    query
        .iter(world)
        .find_map(|(slot, hp)| (slot.player == owner && slot.lane == lane).then_some(hp.hp))
        .expect("objective should exist")
}

fn objective_destroyed(app: &mut App, owner: PlayerId, lane: u8) -> bool {
    let world = app.world_mut();
    let mut query = world.query::<&ObjectiveSlot>();
    query
        .iter(world)
        .find_map(|slot| (slot.player == owner && slot.lane == lane).then_some(slot.destroyed))
        .expect("objective should exist")
}

fn unit_cell(app: &App, unit: Entity) -> u8 {
    app.world()
        .entity(unit)
        .get::<BoardPosition>()
        .expect("unit should have a board position")
        .cell
}

fn trace(app: &App) -> &[CombatTraceEntry] {
    app.world().resource::<CombatResolutionTrace>().entries()
}

fn gold(app: &App, player: PlayerId) -> u32 {
    app.world()
        .resource::<PlayerEconomies>()
        .0
        .get(&player)
        .expect("economy should exist")
        .gold
}

fn real_destroyed(app: &App, player: PlayerId) -> u32 {
    app.world()
        .resource::<ObjectiveCounters>()
        .real_objectives_destroyed(player)
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

#[test]
fn test_cr_10_alive_unit_at_cell_8_damages_objective_and_remains() {
    let mut app = app_with_combat();
    spawn_objective(&mut app, PLAYER_B, 1, 5, false);
    let attacker = spawn_unit(
        &mut app,
        ATTACKER_CARD,
        PLAYER_A,
        1,
        8,
        UnitStats::new(5, 3, 0, 0),
    );

    begin_resolution(&mut app);

    assert_eq!(objective_hp(&mut app, PLAYER_B, 1), 2);
    assert_eq!(unit_cell(&app, attacker), 8);
    assert!(trace(&app).contains(&CombatTraceEntry::ObjectiveDamaged {
        target_player_id: PLAYER_B,
        lane: 1,
        hp_before: 5,
        hp_after: 2,
        attacker_id: Some(attacker),
    }));
}

#[test]
fn test_cr_11_unit_removed_after_ss3_does_not_deal_objective_damage() {
    let mut app = app_with_combat();
    spawn_objective(&mut app, PLAYER_B, 1, 5, false);
    let victim = spawn_unit(
        &mut app,
        FIRST_STRIKE_CARD,
        PLAYER_A,
        1,
        8,
        UnitStats::new(1, 1, 0, 0),
    );
    let first_strike_attacker = spawn_unit(
        &mut app,
        FIRST_STRIKE_CARD,
        PLAYER_B,
        1,
        8,
        UnitStats::new(5, 1, 0, 0),
    );

    begin_resolution(&mut app);

    assert_eq!(objective_hp(&mut app, PLAYER_B, 1), 5);
    assert!(!app.world().entities().contains(victim));
    assert!(trace(&app).contains(&CombatTraceEntry::CombatDamage {
        attacker: first_strike_attacker,
        defender: victim,
        damage_amount: 1,
        hp_after: 0,
        was_blocked_by_shield: false,
        sub_step: 3,
    }));
    assert!(!trace(&app).iter().any(|entry| matches!(
        entry,
        CombatTraceEntry::ObjectiveDamaged {
            attacker_id: Some(attacker),
            ..
        } if *attacker == victim
    )));
}

#[test]
fn test_cr_17_destroying_objective_awards_three_gold_and_no_kill_gold() {
    let mut app = app_with_combat();
    spawn_objective(&mut app, PLAYER_B, 2, 2, false);
    let attacker = spawn_unit(
        &mut app,
        ATTACKER_CARD,
        PLAYER_A,
        2,
        8,
        UnitStats::new(5, 3, 0, 0),
    );

    begin_resolution(&mut app);

    assert_eq!(objective_hp(&mut app, PLAYER_B, 2), 0);
    assert!(objective_destroyed(&mut app, PLAYER_B, 2));
    assert_eq!(gold(&app, PLAYER_A), 3);
    assert!(trace(&app).contains(&CombatTraceEntry::ObjectiveDestroyed {
        target_player_id: PLAYER_B,
        lane: 2,
        was_fake: false,
    }));
    assert!(trace(&app).contains(&CombatTraceEntry::GoldAwarded {
        player: PLAYER_A,
        amount: 3,
        reason: GoldAwardReason::ObjectiveReward,
    }));
    assert!(!trace(&app).iter().any(|entry| matches!(
        entry,
        CombatTraceEntry::GoldAwarded {
            player,
            amount: 1,
            reason: GoldAwardReason::Kill,
        } if *player == PLAYER_A
    )));
    assert!(trace(&app).contains(&CombatTraceEntry::ObjectiveDamaged {
        target_player_id: PLAYER_B,
        lane: 2,
        hp_before: 2,
        hp_after: 0,
        attacker_id: Some(attacker),
    }));
}

#[test]
fn test_cr_18_second_real_objective_destroyed_emits_game_over_after_completion() {
    let mut app = app_with_rsm_and_combat();
    app.world_mut()
        .resource_mut::<ObjectiveCounters>()
        .real_destroyed
        .insert(PLAYER_B, 1);
    spawn_objective(&mut app, PLAYER_B, 3, 2, false);
    spawn_unit(
        &mut app,
        ATTACKER_CARD,
        PLAYER_A,
        3,
        8,
        UnitStats::new(5, 3, 0, 0),
    );

    begin_resolution(&mut app);

    assert_eq!(real_destroyed(&app, PLAYER_B), 2);
    assert_eq!(read_messages::<ResolutionComplete>(&app).len(), 1);

    finish_resolution(&mut app);

    let game_over = read_messages::<GameOverEmitted>(&app);
    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::ObjectivesDestroyed);
    assert_eq!(game_over[0].loser, Some(PLAYER_B));
    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::GameOver
    );
}

#[test]
fn test_cr_19_mutual_second_real_objective_destruction_draws_after_all_destructions() {
    let mut app = app_with_rsm_and_combat();
    {
        let mut counters = app.world_mut().resource_mut::<ObjectiveCounters>();
        counters.real_destroyed.insert(PLAYER_A, 1);
        counters.real_destroyed.insert(PLAYER_B, 1);
    }
    spawn_objective(&mut app, PLAYER_B, 1, 2, false);
    spawn_objective(&mut app, PLAYER_A, 2, 2, false);
    spawn_unit(
        &mut app,
        ATTACKER_CARD,
        PLAYER_A,
        1,
        8,
        UnitStats::new(5, 3, 0, 0),
    );
    spawn_unit(
        &mut app,
        ATTACKER_CARD,
        PLAYER_B,
        2,
        1,
        UnitStats::new(5, 3, 0, 0),
    );

    begin_resolution(&mut app);

    assert_eq!(real_destroyed(&app, PLAYER_A), 2);
    assert_eq!(real_destroyed(&app, PLAYER_B), 2);
    assert!(objective_destroyed(&mut app, PLAYER_A, 2));
    assert!(objective_destroyed(&mut app, PLAYER_B, 1));

    finish_resolution(&mut app);

    let game_over = read_messages::<GameOverEmitted>(&app);
    assert_eq!(game_over.len(), 1);
    assert_eq!(game_over[0].reason, GameOverReason::Draw);
    assert_eq!(game_over[0].loser, None);
    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::GameOver
    );
}

#[test]
fn test_cr_27_objective_hp_saturates_to_zero_when_damage_exceeds_hp() {
    let mut app = app_with_combat();
    spawn_objective(&mut app, PLAYER_B, 4, 2, false);
    let attacker = spawn_unit(
        &mut app,
        ATTACKER_CARD,
        PLAYER_A,
        4,
        8,
        UnitStats::new(5, 3, 0, 0),
    );

    begin_resolution(&mut app);

    assert_eq!(objective_hp(&mut app, PLAYER_B, 4), 0);
    assert!(objective_destroyed(&mut app, PLAYER_B, 4));
    assert!(trace(&app).contains(&CombatTraceEntry::ObjectiveDamaged {
        target_player_id: PLAYER_B,
        lane: 4,
        hp_before: 2,
        hp_after: 0,
        attacker_id: Some(attacker),
    }));
}

fn grid_indices(lane: u8, cell: u8) -> Option<(usize, usize)> {
    if !(1..=5).contains(&lane) || !(1..=8).contains(&cell) {
        return None;
    }
    Some((usize::from(lane - 1), usize::from(cell - 1)))
}
