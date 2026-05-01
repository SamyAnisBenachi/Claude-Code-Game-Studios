use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::ServerPlugins;
use server::core::board::{BoardPosition, UnitCardRef, UnitOwner};
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::rsm::{
    PlacementPhaseEntered, PlacementSubmitted, ResolutionPhaseEntered, RoundPhase, RoundState,
};
use server::core::session::SessionConfig;
use server::feature::board::{
    get_units_at_cell, BoardGrid, BoardPlugin, PendingPlacements, PlacementCommitTrace,
    PlacementCommitTraceEntry, PlacementCommitted, PlacementRevealOutbox,
    PlacementSubmissionReceived, PlayerSubmission,
};
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{GameMode, PlacedCard, PlayTarget};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn card_id(id: u32) -> CardId {
    CardId(id)
}

fn placed_minion(owner_id: PlayerId, card_id: CardId, lane: u8, cell: u8) -> PlacedCard {
    PlacedCard {
        card_id,
        owner_id,
        target: PlayTarget::BoardCell { lane, cell },
    }
}

fn minion_card(id: u32, cost: u32, hp: u8, atk: u8, mp: u8) -> CardData {
    CardData {
        id: card_id(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: None,
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost,
        atk,
        hp,
        mp,
        ar: 0,
        keywords: vec![],
        effect_text: String::new(),
        art_id: format!("test_{id}"),
        pool_copies_override: Some(1),
    }
}

fn catalog(cards: Vec<CardData>) -> CardCatalog {
    CardCatalog {
        cards: cards.into_iter().map(|card| (card.id, card)).collect(),
    }
}

fn economy(current_mana: u32, reserve_mana: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold: 0,
        current_mana,
        reserve_mana,
        mana_cap: 10,
        reserved_gold: 0,
    }
}

fn session_config() -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(player(1), 0), (player(2), 1)]),
        class_map: HashMap::from([(player(1), ClassId::Iop), (player(2), ClassId::Cra)]),
    }
}

fn app_with_board() -> App {
    let mut app = App::new();
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / 60.0),
    })
    .add_plugins(BoardPlugin)
    .insert_resource(RoundState {
        phase: RoundPhase::Placement,
        round_number: 1,
        ..RoundState::new()
    })
    .insert_resource(session_config())
    .insert_resource(catalog(vec![
        minion_card(10, 2, 3, 4, 1),
        minion_card(20, 3, 5, 2, 2),
        minion_card(30, 1, 1, 1, 1),
    ]))
    .insert_resource(PlayerEconomies(HashMap::from([
        (player(1), economy(10, 0)),
        (player(2), economy(10, 0)),
    ])));
    app
}

fn write_message<T: Message>(app: &mut App, message: T) {
    app.world_mut().resource_mut::<Messages<T>>().write(message);
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

#[test]
fn test_placement_phase_entered_clears_pending_buffer() {
    let mut app = app_with_board();
    app.world_mut()
        .resource_mut::<PendingPlacements>()
        .submissions
        .insert(
            player(1),
            PlayerSubmission {
                placements: vec![placed_minion(player(1), card_id(10), 1, 1)],
                submitted_at: std::time::Duration::ZERO,
                is_final: true,
            },
        );

    write_message(&mut app, PlacementPhaseEntered { round: 2 });
    app.update();

    assert!(app
        .world()
        .resource::<PendingPlacements>()
        .submissions
        .is_empty());
}

#[test]
fn test_duplicate_submission_keeps_first_final_batch() {
    let mut app = app_with_board();
    let first = placed_minion(player(1), card_id(10), 1, 1);
    let second = placed_minion(player(1), card_id(30), 2, 1);

    write_message(
        &mut app,
        PlacementSubmissionReceived {
            player: player(1),
            placements: vec![first.clone()],
        },
    );
    app.update();

    write_message(
        &mut app,
        PlacementSubmissionReceived {
            player: player(1),
            placements: vec![second],
        },
    );
    app.update();

    let pending = app.world().resource::<PendingPlacements>();
    let submission = pending
        .submissions
        .get(&player(1))
        .expect("first submission should be retained");
    assert_eq!(submission.placements, vec![first]);
    assert!(submission.is_final);

    let submitted = read_messages::<PlacementSubmitted>(&app);
    assert_eq!(submitted.len(), 1);
    assert!(app
        .world()
        .resource::<PlacementRevealOutbox>()
        .messages()
        .is_empty());
}

#[test]
fn test_close_placement_phase_reveals_before_spawning_units_atomically() {
    let mut app = app_with_board();
    app.world_mut()
        .resource_mut::<PendingPlacements>()
        .submissions
        .extend([
            (
                player(1),
                PlayerSubmission {
                    placements: vec![placed_minion(player(1), card_id(10), 1, 1)],
                    submitted_at: std::time::Duration::ZERO,
                    is_final: true,
                },
            ),
            (
                player(2),
                PlayerSubmission {
                    placements: vec![placed_minion(player(2), card_id(20), 5, 8)],
                    submitted_at: std::time::Duration::ZERO,
                    is_final: true,
                },
            ),
        ]);

    assert!(get_units_at_cell(app.world().resource::<BoardGrid>(), 1, 1).is_empty());
    assert!(get_units_at_cell(app.world().resource::<BoardGrid>(), 5, 8).is_empty());

    write_message(&mut app, ResolutionPhaseEntered { round: 3 });
    app.update();

    let outbox = app.world().resource::<PlacementRevealOutbox>();
    assert_eq!(outbox.messages().len(), 1);
    assert_eq!(
        outbox.messages()[0].placements,
        vec![
            placed_minion(player(1), card_id(10), 1, 1),
            placed_minion(player(2), card_id(20), 5, 8),
        ]
    );

    let player_a_units = get_units_at_cell(app.world().resource::<BoardGrid>(), 1, 1);
    let player_b_units = get_units_at_cell(app.world().resource::<BoardGrid>(), 5, 8);
    assert_eq!(player_a_units.len(), 1);
    assert_eq!(player_b_units.len(), 1);

    let a_entity = app.world().entity(player_a_units[0]);
    assert_eq!(a_entity.get::<UnitOwner>(), Some(&UnitOwner(player(1))));
    assert_eq!(
        a_entity.get::<UnitCardRef>(),
        Some(&UnitCardRef(card_id(10)))
    );
    assert_eq!(
        a_entity.get::<BoardPosition>(),
        Some(&BoardPosition { lane: 1, cell: 1 })
    );

    let trace = app.world().resource::<PlacementCommitTrace>().entries();
    let reveal_index = trace
        .iter()
        .position(|entry| *entry == PlacementCommitTraceEntry::PlacementRevealEnqueued)
        .expect("reveal enqueue should be traced");
    let first_spawn_index = trace
        .iter()
        .position(|entry| matches!(entry, PlacementCommitTraceEntry::UnitSpawned { .. }))
        .expect("unit spawn should be traced");
    assert!(reveal_index < first_spawn_index);

    let committed = read_messages::<PlacementCommitted>(&app);
    assert_eq!(committed.len(), 1);
    assert_eq!(committed[0].round_number, 3);
    assert!(app
        .world()
        .resource::<PendingPlacements>()
        .submissions
        .is_empty());

    let economies = app.world().resource::<PlayerEconomies>();
    assert_eq!(economies.0[&player(1)].current_mana, 8);
    assert_eq!(economies.0[&player(2)].current_mana, 7);
}
