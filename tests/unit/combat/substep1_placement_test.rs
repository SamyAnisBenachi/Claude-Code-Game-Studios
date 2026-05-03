use std::time::Duration;

use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use server::core::rsm::BeginResolution;
use server::feature::board::{
    BoardCell, BoardGrid, BoardOccupancy, PendingPlacements, PlayerSubmission,
};
use server::feature::combat::{
    AppearanceEffect, AppearanceEffectRegistry, AppearanceTarget, CombatNetworkMessageKind,
    CombatNetworkOutbox, CombatPlugin, CombatResolutionTrace, CombatTraceEntry,
};
use server::feature::keyword::components::UnitKeywordState;
use server::feature::keyword::effects::{
    can_execute_standard_movement, charge_x_cells_for_sub_step,
};
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Keyword, Rarity, SimpleKeyword, UnitType};
use shared::keyword::KeywordKind;
use shared::protocol::{PlacedCard, PlayTarget};
use shared::session::PlayerId;

const ROUND: u32 = 3;
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
        hp: 2,
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

fn placed(card_id: CardId, owner_id: PlayerId, lane: u8, cell: u8) -> PlacedCard {
    PlacedCard {
        card_id,
        owner_id,
        target: PlayTarget::BoardCell { lane, cell },
        reserve_amount: 0,
    }
}

fn app_with_cards(cards: Vec<CardData>) -> App {
    let mut app = App::new();
    app.add_plugins(CombatPlugin);
    app.insert_resource(BoardGrid::default());
    app.insert_resource(BoardOccupancy::default());
    app.insert_resource(PendingPlacements::default());
    app.insert_resource(CardCatalog {
        cards: cards.into_iter().map(|card| (card.id, card)).collect(),
    });
    app
}

fn submit(app: &mut App, player: PlayerId, placements: Vec<PlacedCard>) {
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

fn entity_for_card(app: &mut App, card_id: CardId) -> Entity {
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &UnitCardRef)>();
    query
        .iter(world)
        .find_map(|(entity, card)| (card.0 == card_id).then_some(entity))
        .expect("card entity should exist")
}

fn spawn_existing_unit(
    app: &mut App,
    card_id: CardId,
    owner: PlayerId,
    lane: u8,
    cell: u8,
    hp: u8,
    mp: u8,
) -> Entity {
    let entity = app
        .world_mut()
        .spawn((
            UnitCardRef(card_id),
            UnitOwner(owner),
            UnitStats::new(hp, 1, mp, 0),
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

#[test]
fn cr_24_appearance_fires_before_substep2() {
    let appearance_card = card(10, vec![simple(SimpleKeyword::Appearance)]);
    let mut app = app_with_cards(vec![appearance_card]);
    submit(&mut app, PLAYER_A, vec![placed(CardId(10), PLAYER_A, 1, 1)]);

    begin_resolution(&mut app);

    let unit = entity_for_card(&mut app, CardId(10));
    let reveal_index = trace_index(&app, CombatTraceEntry::PlacementRevealEnqueued);
    let placed_index = trace_index(
        &app,
        CombatTraceEntry::UnitPlaced {
            entity: unit,
            lane: 1,
            cell: 1,
        },
    );
    let appearance_index = trace_index(
        &app,
        CombatTraceEntry::KeywordTriggered {
            unit,
            keyword: KeywordKind::Appearance,
            sub_step: 1,
        },
    );
    let substep2_index = trace_index(&app, CombatTraceEntry::SubStepStarted(2));

    assert!(reveal_index < placed_index);
    assert!(placed_index < appearance_index);
    assert!(appearance_index < substep2_index);
    assert_eq!(
        app.world()
            .resource::<CombatNetworkOutbox>()
            .message_kinds()
            .first(),
        Some(&CombatNetworkMessageKind::PlacementReveal)
    );
}

#[test]
fn cr_38_death_from_appearance_is_deferred_until_all_appearances_finish() {
    let killer = card(20, vec![simple(SimpleKeyword::Appearance)]);
    let observer = card(21, vec![simple(SimpleKeyword::Appearance)]);
    let death_target = card(22, vec![simple(SimpleKeyword::Death)]);
    let mut app = app_with_cards(vec![killer, observer, death_target]);
    app.world_mut()
        .resource_mut::<AppearanceEffectRegistry>()
        .insert(
            CardId(20),
            vec![AppearanceEffect::Damage {
                target: AppearanceTarget::FirstEnemyInLane,
                amount: 1,
            }],
        );
    let target = spawn_existing_unit(&mut app, CardId(22), PLAYER_B, 1, 2, 1, 0);
    submit(
        &mut app,
        PLAYER_A,
        vec![
            placed(CardId(20), PLAYER_A, 1, 1),
            placed(CardId(21), PLAYER_A, 2, 1),
        ],
    );

    begin_resolution(&mut app);

    let killer = entity_for_card(&mut app, CardId(20));
    let observer = entity_for_card(&mut app, CardId(21));
    let killer_appearance = trace_index(
        &app,
        CombatTraceEntry::KeywordTriggered {
            unit: killer,
            keyword: KeywordKind::Appearance,
            sub_step: 1,
        },
    );
    let observer_appearance = trace_index(
        &app,
        CombatTraceEntry::KeywordTriggered {
            unit: observer,
            keyword: KeywordKind::Appearance,
            sub_step: 1,
        },
    );
    let death = trace_index(
        &app,
        CombatTraceEntry::KeywordTriggered {
            unit: target,
            keyword: KeywordKind::Death,
            sub_step: 1,
        },
    );

    assert!(killer_appearance < observer_appearance);
    assert!(observer_appearance < death);
    assert_eq!(
        app.world()
            .entity(target)
            .get::<UnitStats>()
            .expect("target should have stats")
            .hp,
        0
    );
}

#[test]
fn cr_39_change_lane_from_appearance_executes_before_substep2() {
    let mover_card = card(30, vec![simple(SimpleKeyword::Appearance)]);
    let mut app = app_with_cards(vec![mover_card]);
    app.world_mut()
        .resource_mut::<AppearanceEffectRegistry>()
        .insert(CardId(30), vec![AppearanceEffect::ChangeLane { delta: 1 }]);
    submit(&mut app, PLAYER_A, vec![placed(CardId(30), PLAYER_A, 2, 1)]);

    begin_resolution(&mut app);

    let mover = entity_for_card(&mut app, CardId(30));
    let lane_change_index = trace_index(
        &app,
        CombatTraceEntry::UnitChangedLane {
            unit: mover,
            from_lane: 2,
            to_lane: 3,
            sub_step: 1,
        },
    );
    let substep2_index = trace_index(&app, CombatTraceEntry::SubStepStarted(2));

    assert!(lane_change_index < substep2_index);
    assert_eq!(
        app.world()
            .entity(mover)
            .get::<BoardPosition>()
            .expect("mover should have board position")
            .lane,
        3
    );
}

#[test]
fn cr_40_stun_from_appearance_suppresses_charge_x_and_standard_movement() {
    let stunner = card(40, vec![simple(SimpleKeyword::Appearance)]);
    let charge_target = card(41, vec![Keyword::ChargeXMove { cells: 2 }]);
    let mut app = app_with_cards(vec![stunner, charge_target]);
    app.world_mut()
        .resource_mut::<AppearanceEffectRegistry>()
        .insert(
            CardId(40),
            vec![AppearanceEffect::Stun {
                target: AppearanceTarget::FirstEnemyInLane,
            }],
        );
    let target = spawn_existing_unit(&mut app, CardId(41), PLAYER_B, 1, 4, 2, 1);
    submit(&mut app, PLAYER_A, vec![placed(CardId(40), PLAYER_A, 1, 1)]);

    begin_resolution(&mut app);

    let target_position = app
        .world()
        .entity(target)
        .get::<BoardPosition>()
        .expect("target should have board position");
    assert_eq!((target_position.lane, target_position.cell), (1, 4));
    assert!(
        app.world()
            .entity(target)
            .get::<UnitKeywordState>()
            .expect("target should have keyword state")
            .stun_active
    );
    assert!(charge_x_cells_for_sub_step(target, ROUND, app.world()).is_none());
    assert!(!can_execute_standard_movement(target, ROUND, app.world()));
}

fn grid_indices(lane: u8, cell: u8) -> Option<(usize, usize)> {
    if !(1..=5).contains(&lane) || !(1..=8).contains(&cell) {
        return None;
    }
    Some((usize::from(lane - 1), usize::from(cell - 1)))
}
