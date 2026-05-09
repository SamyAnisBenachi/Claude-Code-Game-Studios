use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use server::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::objective_contract::ObjectiveCounters;
use server::core::rsm::{
    advance_phase, AuctionSettled, BeginResolution, PlacementPhaseEntered, ResolutionPhaseEntered,
    RoundPhase, RoundState, RsmNetworkOutbox, RsmPlugin,
};
use server::core::session::SessionConfig;
use server::feature::board::{
    AcceptedPlacement, BoardCell, BoardGrid, BoardOccupancy, BoardPlugin, PendingPlacements,
    PlacementCommitted, PlayerSubmission, SpawnRangeState,
};
use server::feature::combat::{
    CombatNetworkMessage, CombatNetworkMessageKind, CombatNetworkOutbox, CombatPlugin,
    CombatResolutionTrace, CombatTraceEntry,
};
use server::feature::keyword::components::UnitKeywordState;
use server::feature::objective::{
    HiddenObjectives, ObjectiveHp, ObjectiveSlot, PendingObjectiveEvents,
};
use server::foundation::config::CardCatalog;
use server::network::rsm_dispatch::dispatch_phase_changed;
use shared::card::{CardData, CardId, CardType, ClassId, Keyword, Rarity, SimpleKeyword, UnitType};
use shared::keyword::KeywordKind;
use shared::protocol::{GameMode, GoldReason, PlayTarget, ResolutionEvent, S2CResolutionEvent};
use shared::session::PlayerId;

const ROUND: u32 = 7;
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

fn session_config() -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(PLAYER_A, 0), (PLAYER_B, 1)]),
        class_map: HashMap::from([(PLAYER_A, ClassId::Iop), (PLAYER_B, ClassId::Cra)]),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn app_with_combat(cards: Vec<CardData>) -> App {
    let mut app = App::new();
    app.add_plugins((BoardPlugin, CombatPlugin));
    app.add_message::<BeginResolution>();
    app.add_message::<PlacementPhaseEntered>();
    app.add_message::<ResolutionPhaseEntered>();
    app.insert_resource(session_config());
    app.insert_resource(CardCatalog {
        cards: cards.into_iter().map(|card| (card.id, card)).collect(),
    });
    app.insert_resource(HiddenObjectives::default());
    app.insert_resource(ObjectiveCounters::default());
    app.insert_resource(PendingObjectiveEvents::default());
    app.insert_resource(PlayerEconomies(HashMap::from([
        (PLAYER_A, economy()),
        (PLAYER_B, economy()),
    ])));
    app
}

fn app_with_rsm_combat_dispatch() -> App {
    let mut app = App::new();
    app.add_plugins((RsmPlugin, CombatPlugin));
    app.add_message::<AuctionSettled>();
    app.add_message::<PlacementCommitted>();
    app.insert_resource(Time::<()>::default());
    app.add_systems(Update, dispatch_phase_changed.after(advance_phase));
    *app.world_mut().resource_mut::<RoundState>() = RoundState {
        phase: RoundPhase::Resolution,
        round_number: 1,
        ..RoundState::new()
    };
    app
}

fn economy() -> PlayerEconomy {
    PlayerEconomy {
        gold: 0,
        current_mana: 0,
        reserve_mana: 0,
        mana_cap: 10,
        reserved_gold: 0,
    }
}

fn placed(card_id: CardId, owner_id: PlayerId, lane: u8, cell: u8) -> AcceptedPlacement {
    AcceptedPlacement {
        card_id,
        owner_id,
        target: PlayTarget::BoardCell { lane, cell },
        current_mana_spend: 0,
        reserve_mana_spend: 0,
    }
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

fn spawn_objective(app: &mut App, player: PlayerId, lane: u8, hp: u32, is_fake: bool) {
    app.world_mut().spawn((
        ObjectiveHp { hp },
        ObjectiveSlot {
            lane,
            player,
            destroyed: false,
        },
    ));
    app.world_mut()
        .resource_mut::<HiddenObjectives>()
        .identities
        .insert((player, lane), is_fake);
}

fn begin_resolution(app: &mut App) {
    app.world_mut()
        .write_message(BeginResolution { round: ROUND });
    app.update();
}

fn trace(app: &App) -> &[CombatTraceEntry] {
    app.world().resource::<CombatResolutionTrace>().entries()
}

fn resolution_batch(app: &App) -> &S2CResolutionEvent {
    let batches = app
        .world()
        .resource::<CombatNetworkOutbox>()
        .messages()
        .iter()
        .filter_map(|message| match message {
            CombatNetworkMessage::ResolutionEvent(batch) => Some(batch),
            CombatNetworkMessage::PlacementReveal(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(batches.len(), 1, "exactly one resolution batch is allowed");
    batches[0]
}

fn entity_id(entity: Entity) -> u64 {
    entity.to_bits()
}

#[test]
fn test_cr_30_placement_reveal_is_atomic_and_before_substep1_effects() {
    let mut app = app_with_combat(vec![
        card(10, vec![]),
        card(11, vec![]),
        card(12, vec![]),
        card(13, vec![]),
    ]);
    submit(
        &mut app,
        PLAYER_A,
        vec![
            placed(CardId(10), PLAYER_A, 1, 1),
            placed(CardId(11), PLAYER_A, 2, 1),
        ],
    );
    submit(
        &mut app,
        PLAYER_B,
        vec![
            placed(CardId(12), PLAYER_B, 1, 8),
            placed(CardId(13), PLAYER_B, 2, 8),
        ],
    );

    begin_resolution(&mut app);

    let outbox = app.world().resource::<CombatNetworkOutbox>();
    assert_eq!(
        outbox.message_kinds(),
        vec![
            CombatNetworkMessageKind::PlacementReveal,
            CombatNetworkMessageKind::ResolutionEvent,
        ]
    );
    let CombatNetworkMessage::PlacementReveal(reveal) = &outbox.messages()[0] else {
        panic!("first combat network message should be S2CPlacementReveal");
    };
    assert_eq!(reveal.placements.len(), 4);
    assert_eq!(
        reveal
            .placements
            .iter()
            .map(|placement| placement.owner_id)
            .collect::<Vec<_>>(),
        vec![PLAYER_A, PLAYER_A, PLAYER_B, PLAYER_B]
    );

    let reveal_index = trace(&app)
        .iter()
        .position(|entry| *entry == CombatTraceEntry::PlacementRevealEnqueued)
        .expect("placement reveal should be traced");
    let first_placement_index = trace(&app)
        .iter()
        .position(|entry| matches!(entry, CombatTraceEntry::UnitPlaced { .. }))
        .expect("unit placement should be traced");
    assert!(reveal_index < first_placement_index);

    let batch = resolution_batch(&app);
    assert_eq!(batch.round, ROUND);
    assert!(matches!(
        batch.events.first().map(|event| &event.event),
        Some(ResolutionEvent::SubStepBegin)
    ));
    assert_eq!(batch.events.first().map(|event| event.sub_step), Some(1));
    assert_eq!(
        batch
            .events
            .iter()
            .filter(|event| matches!(event.event, ResolutionEvent::UnitPlaced { .. }))
            .count(),
        4
    );
}

#[test]
fn test_cr_32_resolution_event_batch_serializes_complete_ordered_log() {
    let mut app = app_with_combat(vec![
        card(
            20,
            vec![
                Keyword::RangeX { max_range: 3 },
                simple(SimpleKeyword::FirstStrike),
            ],
        ),
        card(21, vec![simple(SimpleKeyword::Shield)]),
        card(22, vec![simple(SimpleKeyword::FinalBlow)]),
        card(23, vec![]),
        card(24, vec![]),
        card(25, vec![]),
        card(26, vec![]),
    ]);

    let first_striker = spawn_unit(
        &mut app,
        CardId(20),
        PLAYER_A,
        1,
        3,
        UnitStats::new(5, 2, 0, 0),
        UnitKeywordState::default(),
    );
    let shielded = spawn_unit(
        &mut app,
        CardId(21),
        PLAYER_B,
        1,
        4,
        UnitStats::new(5, 1, 0, 0),
        UnitKeywordState {
            shield_active: true,
            ..default()
        },
    );
    let final_blow_attacker = spawn_unit(
        &mut app,
        CardId(22),
        PLAYER_A,
        2,
        5,
        UnitStats::new(5, 3, 0, 0),
        UnitKeywordState::default(),
    );
    let killed = spawn_unit(
        &mut app,
        CardId(23),
        PLAYER_B,
        2,
        5,
        UnitStats::new(1, 0, 0, 0),
        UnitKeywordState::default(),
    );
    let nonlethal_attacker = spawn_unit(
        &mut app,
        CardId(24),
        PLAYER_A,
        3,
        5,
        UnitStats::new(5, 1, 0, 0),
        UnitKeywordState::default(),
    );
    let durable_defender = spawn_unit(
        &mut app,
        CardId(25),
        PLAYER_B,
        3,
        5,
        UnitStats::new(5, 0, 0, 0),
        UnitKeywordState::default(),
    );
    let objective_attacker = spawn_unit(
        &mut app,
        CardId(26),
        PLAYER_A,
        4,
        8,
        UnitStats::new(5, 3, 0, 0),
        UnitKeywordState::default(),
    );
    spawn_objective(&mut app, PLAYER_B, 4, 2, false);

    begin_resolution(&mut app);

    let batch = resolution_batch(&app);
    assert_eq!(batch.round, ROUND);
    let encoded = serde_json::to_string(batch).expect("resolution batch should serialize");
    let decoded: S2CResolutionEvent =
        serde_json::from_str(&encoded).expect("resolution batch should deserialize");
    assert_eq!(&decoded, batch);

    assert_eq!(
        batch
            .events
            .iter()
            .filter_map(|event| match event.event {
                ResolutionEvent::SubStepBegin => Some(event.sub_step),
                _ => None,
            })
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 6]
    );

    for (index, event) in batch.events.iter().enumerate() {
        assert_eq!(event.trigger_index, index as u32);
    }
    for window in batch.events.windows(2) {
        assert!(
            window[0].sub_step < window[1].sub_step
                || (window[0].sub_step == window[1].sub_step
                    && window[0].trigger_index < window[1].trigger_index)
        );
    }

    assert!(batch.events.iter().any(|tagged| matches!(
        tagged.event,
        ResolutionEvent::CombatDamage {
            attacker_id,
            defender_id,
            damage_amount: 0,
            was_blocked_by_shield: true,
            ..
        } if tagged.sub_step == 3
            && attacker_id == entity_id(first_striker)
            && defender_id == entity_id(shielded)
    )));
    assert!(batch.events.iter().any(|tagged| matches!(
        tagged.event,
        ResolutionEvent::CombatDamage {
            attacker_id,
            defender_id,
            damage_amount: 1,
            defender_hp_after: 4,
            was_blocked_by_shield: false,
        } if attacker_id == entity_id(nonlethal_attacker)
            && defender_id == entity_id(durable_defender)
            && tagged.sub_step == 6
    )));
    assert!(batch.events.iter().any(|tagged| matches!(
        tagged.event,
        ResolutionEvent::UnitRemoved { unit_id, lane: 2, cell: 5 }
            if unit_id == entity_id(killed)
    )));
    assert!(batch.events.iter().any(|tagged| matches!(
        tagged.event,
        ResolutionEvent::GoldAwarded {
            player: PLAYER_A,
            amount: 1,
            reason: GoldReason::Kill,
        }
    )));
    assert!(batch.events.iter().any(|tagged| matches!(
        tagged.event,
        ResolutionEvent::KeywordTriggered {
            unit_id,
            keyword: KeywordKind::FinalBlow,
        } if tagged.sub_step == 6 && unit_id == entity_id(final_blow_attacker)
    )));
    assert!(batch.events.iter().any(|tagged| matches!(
        tagged.event,
        ResolutionEvent::KeywordTriggered {
            unit_id,
            keyword: KeywordKind::Shield,
        } if tagged.sub_step == 3 && unit_id == entity_id(shielded)
    )));
    assert!(batch.events.iter().any(|tagged| matches!(
        tagged.event,
        ResolutionEvent::ObjectiveDamage {
            target_player_id: PLAYER_B,
            lane: 4,
            damage_amount: 2,
            objective_hp_after: 0,
            attacker_id: Some(attacker_id),
        } if tagged.sub_step == 6 && attacker_id == entity_id(objective_attacker)
    )));
    assert!(batch.events.iter().any(|tagged| matches!(
        tagged.event,
        ResolutionEvent::ObjectiveDestroyed {
            target_player_id: PLAYER_B,
            lane: 4,
            was_fake: false,
        }
    )));
    assert!(batch.events.iter().any(|tagged| matches!(
        tagged.event,
        ResolutionEvent::GoldAwarded {
            player: PLAYER_A,
            amount: 3,
            reason: GoldReason::ObjectiveDestroyed,
        }
    )));
}

#[test]
fn test_spawn_range_changed_follows_fake_objective_destroyed_in_resolution_batch() {
    let mut app = app_with_combat(vec![card(30, vec![])]);
    spawn_unit(
        &mut app,
        CardId(30),
        PLAYER_A,
        2,
        8,
        UnitStats::new(2, 1, 0, 0),
        UnitKeywordState::default(),
    );
    spawn_objective(&mut app, PLAYER_B, 2, 1, true);

    begin_resolution(&mut app);

    let batch = resolution_batch(&app);
    let objective_destroyed_index = batch
        .events
        .iter()
        .position(|tagged| {
            matches!(
                tagged.event,
                ResolutionEvent::ObjectiveDestroyed {
                    target_player_id: PLAYER_B,
                    lane: 2,
                    was_fake: true,
                }
            )
        })
        .expect("fake objective destruction should be in resolution batch");
    let spawn_range_changed_index = batch
        .events
        .iter()
        .position(|tagged| {
            matches!(
                tagged.event,
                ResolutionEvent::SpawnRangeChanged {
                    player_id: PLAYER_A,
                    new_spawn_range_cells: 2,
                }
            )
        })
        .expect("spawn range change should be in resolution batch");

    assert!(objective_destroyed_index < spawn_range_changed_index);
    assert_eq!(
        app.world().resource::<SpawnRangeState>().fakes_destroyed[0],
        1
    );
}

#[test]
fn test_cr_32_phase_change_is_not_observable_before_resolution_batch() {
    let mut app = app_with_rsm_combat_dispatch();
    app.world_mut().write_message(BeginResolution { round: 1 });

    app.update();

    assert_eq!(
        app.world()
            .resource::<CombatNetworkOutbox>()
            .message_kinds(),
        vec![CombatNetworkMessageKind::ResolutionEvent]
    );
    assert!(
        app.world()
            .resource::<RsmNetworkOutbox>()
            .phase_changed()
            .is_empty(),
        "RSM phase output must not be observable in the combat frame"
    );

    let resolution_event_index = trace(&app)
        .iter()
        .position(|entry| *entry == CombatTraceEntry::ResolutionEventEnqueued)
        .expect("resolution event should be traced");
    let completion_index = trace(&app)
        .iter()
        .position(|entry| *entry == CombatTraceEntry::ResolutionCompleteQueued)
        .expect("resolution completion should be traced");
    assert!(resolution_event_index < completion_index);

    app.update();

    assert_eq!(
        app.world()
            .resource::<RsmNetworkOutbox>()
            .phase_changed()
            .last()
            .map(|message| message.phase),
        Some(shared::protocol::RoundPhase::DraftShop)
    );
}

fn grid_indices(lane: u8, cell: u8) -> Option<(usize, usize)> {
    if !(1..=5).contains(&lane) || !(1..=8).contains(&cell) {
        return None;
    }
    Some((usize::from(lane - 1), usize::from(cell - 1)))
}
