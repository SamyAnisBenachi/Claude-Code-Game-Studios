use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use server::core::rsm::BeginResolution;
use server::core::session::SessionConfig;
use server::feature::board::{
    AcceptedPlacement, BoardCell, BoardConfig, BoardGrid, BoardOccupancy, PendingPlacements,
    PlayerSubmission,
};
use server::feature::combat::{CombatPlugin, CombatResolutionTrace, CombatTraceEntry};
use server::feature::keyword::components::UnitKeywordState;
use server::feature::keyword::{ChainDeathBuffer, UnitDied};
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Keyword, Rarity, SimpleKeyword, UnitType};
use shared::keyword::KeywordKind;
use shared::protocol::{GameMode, PlayTarget};
use shared::session::PlayerId;

const PLAYER_A: PlayerId = PlayerId(1);
const PLAYER_B: PlayerId = PlayerId(2);

#[derive(Resource)]
struct ChainVictim(Entity);

#[derive(Clone, Debug)]
struct CardFixture {
    id: u32,
    family: Option<&'static str>,
    keywords: Vec<Keyword>,
    hp: u8,
    atk: u8,
    card_type: CardType,
}

impl CardFixture {
    fn minion(id: u32, hp: u8, atk: u8, keywords: Vec<Keyword>) -> Self {
        Self {
            id,
            family: None,
            keywords,
            hp,
            atk,
            card_type: CardType::Minion,
        }
    }

    fn family_minion(
        id: u32,
        family: &'static str,
        hp: u8,
        atk: u8,
        keywords: Vec<Keyword>,
    ) -> Self {
        Self {
            id,
            family: Some(family),
            keywords,
            hp,
            atk,
            card_type: CardType::Minion,
        }
    }
}

fn card(fixture: CardFixture) -> CardData {
    CardData {
        id: CardId(fixture.id),
        name_fr: format!("Carte {}", fixture.id),
        name_en: format!("Card {}", fixture.id),
        class: ClassId::Neutral,
        family: fixture.family.map(str::to_string),
        rarity: Rarity::Common,
        card_type: fixture.card_type,
        unit_type: UnitType::Neutral,
        cost: 0,
        atk: fixture.atk,
        hp: fixture.hp,
        mp: 0,
        ar: 0,
        keywords: fixture.keywords,
        effect_text: String::new(),
        art_id: format!("persistent_{}", fixture.id),
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

fn app_with_cards(cards: Vec<CardData>) -> App {
    let mut app = App::new();
    app.add_plugins(CombatPlugin);
    app.insert_resource(BoardConfig::default());
    app.insert_resource(BoardGrid::default());
    app.insert_resource(BoardOccupancy::default());
    app.insert_resource(PendingPlacements::default());
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

    insert_board_occupant(app, owner, lane, cell, entity);
    entity
}

fn submit(app: &mut App, player: PlayerId, placements: Vec<AcceptedPlacement>) {
    app.world_mut()
        .resource_mut::<PendingPlacements>()
        .submissions
        .insert(
            player,
            PlayerSubmission {
                placements,
                submitted_at: Duration::ZERO,
                is_final: true,
            },
        );
}

fn placed(card_id: CardId, owner_id: PlayerId, lane: u8, cell: u8) -> AcceptedPlacement {
    AcceptedPlacement {
        owner_id,
        card_id,
        target: PlayTarget::BoardCell { lane, cell },
        current_mana_spend: 0,
        reserve_mana_spend: 0,
    }
}

fn begin_resolution(app: &mut App, round: u32) {
    app.world_mut().write_message(BeginResolution { round });
    app.update();
}

fn hp(app: &App, unit: Entity) -> u8 {
    app.world()
        .entity(unit)
        .get::<UnitStats>()
        .expect("unit should have stats")
        .hp
}

fn keyword_state(app: &App, unit: Entity) -> &UnitKeywordState {
    app.world()
        .entity(unit)
        .get::<UnitKeywordState>()
        .expect("unit should have keyword state")
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

fn damage_count(app: &App, attacker: Entity, defender: Entity, sub_step: u8) -> usize {
    trace(app)
        .iter()
        .filter(|entry| {
            matches!(
                entry,
                CombatTraceEntry::CombatDamage {
                    attacker: logged_attacker,
                    defender: logged_defender,
                    sub_step: logged_sub_step,
                    ..
                } if *logged_attacker == attacker
                    && *logged_defender == defender
                    && *logged_sub_step == sub_step
            )
        })
        .count()
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

fn keyword(unit: Entity, keyword: KeywordKind, sub_step: u8) -> CombatTraceEntry {
    CombatTraceEntry::KeywordTriggered {
        unit,
        keyword,
        sub_step,
    }
}

fn unit_removed(unit: Entity, lane: u8, cell: u8) -> CombatTraceEntry {
    CombatTraceEntry::UnitRemoved { unit, lane, cell }
}

fn entity_for_card(app: &mut App, card_id: CardId) -> Entity {
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &UnitCardRef)>();
    query
        .iter(world)
        .find_map(|(entity, card)| (card.0 == card_id).then_some(entity))
        .expect("card entity should exist")
}

fn insert_board_occupant(app: &mut App, owner: PlayerId, lane: u8, cell: u8, entity: Entity) {
    if let Some((lane_index, cell_index)) = grid_indices(lane, cell) {
        app.world_mut().resource_mut::<BoardGrid>().lanes[lane_index][cell_index] =
            Some(BoardCell::new(entity));
    }
    app.world_mut()
        .resource_mut::<BoardOccupancy>()
        .minion_slots
        .insert((owner, lane), entity);
}

fn death_chain_observer(
    trigger: On<UnitDied>,
    victim: Res<ChainVictim>,
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
fn cr_26_injured_first_strike_activates_after_ss3_boundary() {
    let first_striker = card(CardFixture::minion(
        10,
        30,
        2,
        vec![simple(SimpleKeyword::FirstStrike)],
    ));
    let injured_bonus = card(CardFixture::minion(11, 5, 1, vec![]));
    let mut app = app_with_cards(vec![first_striker, injured_bonus]);
    let attacker = spawn_unit(
        &mut app,
        CardId(10),
        PLAYER_A,
        1,
        4,
        UnitStats::new(30, 2, 0, 0),
        UnitKeywordState::default(),
    );
    let injured = spawn_unit(
        &mut app,
        CardId(11),
        PLAYER_B,
        1,
        4,
        UnitStats::new(5, 1, 0, 0),
        UnitKeywordState {
            injured_grants_first_strike: true,
            ..default()
        },
    );

    begin_resolution(&mut app, 10);

    assert!(hp(&app, injured) < 5);
    assert_eq!(damage_count(&app, injured, attacker, 3), 0);
    assert!(keyword_state(&app, injured).injured_first_strike_active);
    assert!(trace(&app).contains(&keyword(injured, KeywordKind::Injured, 4)));
}

#[test]
fn cr_34_injured_first_strike_persists_into_next_round() {
    let first_striker = card(CardFixture::minion(
        20,
        30,
        1,
        vec![simple(SimpleKeyword::FirstStrike)],
    ));
    let injured_bonus = card(CardFixture::minion(21, 5, 1, vec![]));
    let mut app = app_with_cards(vec![first_striker, injured_bonus]);
    let attacker = spawn_unit(
        &mut app,
        CardId(20),
        PLAYER_A,
        1,
        4,
        UnitStats::new(30, 1, 0, 0),
        UnitKeywordState::default(),
    );
    let injured = spawn_unit(
        &mut app,
        CardId(21),
        PLAYER_B,
        1,
        4,
        UnitStats::new(5, 1, 0, 0),
        UnitKeywordState {
            injured_grants_first_strike: true,
            ..default()
        },
    );

    begin_resolution(&mut app, 20);
    assert_eq!(damage_count(&app, injured, attacker, 3), 0);

    begin_resolution(&mut app, 21);

    assert_eq!(damage_count(&app, injured, attacker, 3), 1);
    assert!(keyword_state(&app, injured).injured_first_strike_active);
}

#[test]
fn cr_33_leader_snapshot_persists_after_ss4_death_and_recomputes_next_round() {
    let leader = card(CardFixture::family_minion(
        30,
        "Tofu",
        1,
        0,
        vec![simple(SimpleKeyword::Leader)],
    ));
    let family_attacker = card(CardFixture::family_minion(31, "Tofu", 20, 2, vec![]));
    let leader_killer = card(CardFixture::minion(
        32,
        20,
        1,
        vec![simple(SimpleKeyword::FirstStrike)],
    ));
    let defender_card = card(CardFixture::minion(33, 20, 1, vec![]));
    let mut app = app_with_cards(vec![leader, family_attacker, leader_killer, defender_card]);
    let leader = spawn_unit(
        &mut app,
        CardId(30),
        PLAYER_A,
        1,
        4,
        UnitStats::new(1, 0, 0, 0),
        UnitKeywordState::default(),
    );
    let family = spawn_unit(
        &mut app,
        CardId(31),
        PLAYER_A,
        2,
        4,
        UnitStats::new(20, 2, 0, 0),
        UnitKeywordState::default(),
    );
    let _killer = spawn_unit(
        &mut app,
        CardId(32),
        PLAYER_B,
        1,
        4,
        UnitStats::new(20, 1, 0, 0),
        UnitKeywordState::default(),
    );
    let defender = spawn_unit(
        &mut app,
        CardId(33),
        PLAYER_B,
        2,
        4,
        UnitStats::new(20, 1, 0, 0),
        UnitKeywordState::default(),
    );

    begin_resolution(&mut app, 30);

    assert!(app.world().get_entity(leader).is_err());
    assert_eq!(keyword_state(&app, family).leader_bonus_atk, 1);
    assert!(trace(&app).contains(&damage(family, defender, 3, 17, 6)));

    let leader_removed = trace_index(&app, unit_removed(leader, 1, 4));
    let family_damage = trace_index(&app, damage(family, defender, 3, 17, 6));
    assert!(leader_removed < family_damage);

    begin_resolution(&mut app, 31);

    assert_eq!(keyword_state(&app, family).leader_bonus_atk, 0);
    assert!(trace(&app).contains(&damage(family, defender, 2, 15, 6)));
}

#[test]
fn cr_33_leader_entering_during_ss1_is_included_in_post_ss1_snapshot() {
    let leader = card(CardFixture::family_minion(
        40,
        "Tofu",
        5,
        0,
        vec![simple(SimpleKeyword::Leader)],
    ));
    let family_attacker = card(CardFixture::family_minion(41, "Tofu", 20, 2, vec![]));
    let defender_card = card(CardFixture::minion(42, 20, 1, vec![]));
    let mut app = app_with_cards(vec![leader, family_attacker, defender_card]);
    let family = spawn_unit(
        &mut app,
        CardId(41),
        PLAYER_A,
        2,
        4,
        UnitStats::new(20, 2, 0, 0),
        UnitKeywordState::default(),
    );
    let defender = spawn_unit(
        &mut app,
        CardId(42),
        PLAYER_B,
        2,
        4,
        UnitStats::new(20, 1, 0, 0),
        UnitKeywordState::default(),
    );
    submit(&mut app, PLAYER_A, vec![placed(CardId(40), PLAYER_A, 1, 1)]);

    begin_resolution(&mut app, 40);

    let leader = entity_for_card(&mut app, CardId(40));
    assert_eq!(keyword_state(&app, family).leader_bonus_atk, 1);
    assert_eq!(keyword_state(&app, leader).leader_bonus_atk, 1);
    assert!(trace(&app).contains(&keyword(leader, KeywordKind::Leader, 1)));
    assert!(trace(&app).contains(&damage(family, defender, 3, 17, 6)));
}

#[test]
fn kw_040_outnumbered_flips_only_after_ss4_chain_drain() {
    let outnumbered_first_striker = card(CardFixture::minion(
        50,
        20,
        1,
        vec![
            simple(SimpleKeyword::Outnumbered),
            simple(SimpleKeyword::FirstStrike),
        ],
    ));
    let death_source = card(CardFixture::minion(
        51,
        1,
        0,
        vec![simple(SimpleKeyword::Death)],
    ));
    let chain_victim = card(CardFixture::minion(52, 5, 0, vec![]));
    let survivor = card(CardFixture::minion(53, 5, 0, vec![]));
    let mut app = app_with_cards(vec![
        outnumbered_first_striker,
        death_source,
        chain_victim,
        survivor,
    ]);
    app.add_observer(death_chain_observer);

    let outnumbered = spawn_unit(
        &mut app,
        CardId(50),
        PLAYER_A,
        1,
        4,
        UnitStats::new(20, 1, 0, 0),
        UnitKeywordState {
            outnumbered_active: true,
            ..default()
        },
    );
    let source = spawn_unit(
        &mut app,
        CardId(51),
        PLAYER_B,
        1,
        4,
        UnitStats::new(1, 0, 0, 0),
        UnitKeywordState::default(),
    );
    let victim = spawn_unit(
        &mut app,
        CardId(52),
        PLAYER_B,
        2,
        4,
        UnitStats::new(5, 0, 0, 0),
        UnitKeywordState::default(),
    );
    let _survivor = spawn_unit(
        &mut app,
        CardId(53),
        PLAYER_B,
        3,
        4,
        UnitStats::new(5, 0, 0, 0),
        UnitKeywordState::default(),
    );
    app.insert_resource(ChainVictim(victim));

    begin_resolution(&mut app, 50);

    assert!(!keyword_state(&app, outnumbered).outnumbered_active);
    assert!(app.world().resource::<ChainDeathBuffer>().0.is_empty());

    let source_removed = trace_index(&app, unit_removed(source, 1, 4));
    let victim_removed = trace_index(&app, unit_removed(victim, 2, 4));
    let outnumbered_flip = trace_index(&app, keyword(outnumbered, KeywordKind::Outnumbered, 5));

    assert!(source_removed < victim_removed);
    assert!(victim_removed < outnumbered_flip);
}

fn grid_indices(lane: u8, cell: u8) -> Option<(usize, usize)> {
    if !(1..=5).contains(&lane) || !(1..=8).contains(&cell) {
        return None;
    }
    Some((usize::from(lane - 1), usize::from(cell - 1)))
}
