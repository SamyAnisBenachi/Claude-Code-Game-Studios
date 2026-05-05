use std::collections::HashMap;

use bevy::prelude::Entity;
use lightyear::prelude::PeerId;
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::rsm::RoundPhase;
use server::core::session::{PlayerConnectionMap, SessionConfig};
use server::feature::acquisition::PlayerHands;
use server::feature::board::{
    deduct_committed_mana, process_placement_submission, AcceptedPlacement, BoardConfig,
    BoardOccupancy, PendingPlacements, PlacementSubmissionResult, SpawnRangeState,
};
use server::foundation::config::CardCatalog;
use server::network::resolve_submit_placement_sender;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{C2SSubmitPlacement, GameMode, PlacedCardSubmit, PlayTarget};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn card_id(id: u32) -> CardId {
    CardId(id)
}

fn session_config() -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(player(1), 0), (player(2), 1)]),
        class_map: HashMap::from([(player(1), ClassId::Iop), (player(2), ClassId::Cra)]),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn card(id: u32, card_type: CardType, cost: u32) -> CardData {
    CardData {
        id: card_id(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: None,
        rarity: Rarity::Common,
        card_type,
        unit_type: UnitType::Blade,
        cost,
        atk: 1,
        hp: 1,
        mp: 1,
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

fn hands(player_id: PlayerId, cards: Vec<CardId>) -> PlayerHands {
    PlayerHands {
        hands: HashMap::from([(player_id, cards)]),
    }
}

fn placement(
    id: CardId,
    target: PlayTarget,
    current_mana_spend: u32,
    reserve_mana_spend: u32,
) -> PlacedCardSubmit {
    PlacedCardSubmit {
        card_id: id,
        target,
        current_mana_spend,
        reserve_mana_spend,
    }
}

#[allow(clippy::too_many_arguments)]
fn submit(
    pending: &mut PendingPlacements,
    player_id: PlayerId,
    placements: Vec<PlacedCardSubmit>,
    phase: Option<RoundPhase>,
    catalog: &CardCatalog,
    economies: &PlayerEconomies,
    hands: &PlayerHands,
    occupancy: &BoardOccupancy,
) -> PlacementSubmissionResult {
    process_placement_submission(
        pending,
        player_id,
        placements,
        phase,
        Some(&session_config()),
        &BoardConfig::default(),
        &SpawnRangeState::default(),
        occupancy,
        Some(catalog),
        Some(economies),
        Some(hands),
    )
}

#[test]
fn test_unknown_sender_submit_is_discarded_before_internal_submission() {
    let peer = PeerId::Netcode(44);
    let mapped = PlayerConnectionMap(HashMap::from([(peer, player(1))]));
    let unknown = PlayerConnectionMap::default();
    let msg = C2SSubmitPlacement {
        placements: vec![placement(
            card_id(10),
            PlayTarget::BoardCell { lane: 1, cell: 1 },
            1,
            0,
        )],
    };

    assert!(resolve_submit_placement_sender(&unknown, peer, msg.clone()).is_none());

    let resolved = resolve_submit_placement_sender(&mapped, peer, msg)
        .expect("mapped peer should resolve to an internal placement submission");
    assert_eq!(resolved.player, player(1));
    assert_eq!(resolved.placements[0].card_id, card_id(10));
}

#[test]
fn test_wrong_phase_submit_is_silently_discarded_without_pending_write() {
    let catalog = catalog(vec![card(10, CardType::Minion, 1)]);
    let economies = PlayerEconomies(HashMap::from([(player(1), economy(1, 0))]));
    let hands = hands(player(1), vec![card_id(10)]);
    let mut pending = PendingPlacements::default();

    let result = submit(
        &mut pending,
        player(1),
        vec![placement(
            card_id(10),
            PlayTarget::BoardCell { lane: 1, cell: 1 },
            1,
            0,
        )],
        Some(RoundPhase::DraftInitial),
        &catalog,
        &economies,
        &hands,
        &BoardOccupancy::default(),
    );

    assert_eq!(result, PlacementSubmissionResult::DiscardedWrongPhase);
    assert!(!pending
        .submissions
        .get(&player(1))
        .is_some_and(|submission| submission.is_final));
}

#[test]
fn test_card_not_in_hand_rejects_full_batch_and_keeps_is_final_false() {
    let catalog = catalog(vec![card(10, CardType::Minion, 1)]);
    let economies = PlayerEconomies(HashMap::from([(player(1), economy(1, 0))]));
    let hands = hands(player(1), vec![card_id(99)]);
    let mut pending = PendingPlacements::default();

    let result = submit(
        &mut pending,
        player(1),
        vec![placement(
            card_id(10),
            PlayTarget::BoardCell { lane: 1, cell: 1 },
            1,
            0,
        )],
        Some(RoundPhase::Placement),
        &catalog,
        &economies,
        &hands,
        &BoardOccupancy::default(),
    );

    assert_eq!(result, PlacementSubmissionResult::CardNotInHand);
    assert!(!pending
        .submissions
        .get(&player(1))
        .is_some_and(|submission| submission.is_final));
}

#[test]
fn test_duplicate_card_id_rejects_full_batch_without_pending_write() {
    let catalog = catalog(vec![card(10, CardType::Minion, 1)]);
    let economies = PlayerEconomies(HashMap::from([(player(1), economy(2, 0))]));
    let hands = hands(player(1), vec![card_id(10)]);
    let mut pending = PendingPlacements::default();

    let result = submit(
        &mut pending,
        player(1),
        vec![
            placement(
                card_id(10),
                PlayTarget::BoardCell { lane: 1, cell: 1 },
                1,
                0,
            ),
            placement(
                card_id(10),
                PlayTarget::BoardCell { lane: 2, cell: 1 },
                1,
                0,
            ),
        ],
        Some(RoundPhase::Placement),
        &catalog,
        &economies,
        &hands,
        &BoardOccupancy::default(),
    );

    assert_eq!(result, PlacementSubmissionResult::DuplicateCardId);
    assert!(pending.submissions.is_empty());
}

#[test]
fn test_invalid_target_spawn_range_and_occupancy_each_reject_full_batch() {
    let catalog = catalog(vec![
        card(10, CardType::Minion, 1),
        card(20, CardType::Spell, 1),
    ]);
    let economies = PlayerEconomies(HashMap::from([(player(1), economy(3, 0))]));
    let hands = hands(player(1), vec![card_id(10), card_id(20)]);

    let mut pending = PendingPlacements::default();
    let invalid_target = submit(
        &mut pending,
        player(1),
        vec![placement(
            card_id(20),
            PlayTarget::TargetObj {
                player_id: player(99),
                lane: 1,
            },
            1,
            0,
        )],
        Some(RoundPhase::Placement),
        &catalog,
        &economies,
        &hands,
        &BoardOccupancy::default(),
    );
    assert_eq!(invalid_target, PlacementSubmissionResult::InvalidTarget);
    assert!(pending.submissions.is_empty());

    let spawn_range = submit(
        &mut pending,
        player(1),
        vec![placement(
            card_id(10),
            PlayTarget::BoardCell { lane: 1, cell: 2 },
            1,
            0,
        )],
        Some(RoundPhase::Placement),
        &catalog,
        &economies,
        &hands,
        &BoardOccupancy::default(),
    );
    assert_eq!(spawn_range, PlacementSubmissionResult::SpawnRangeRejected);
    assert!(pending.submissions.is_empty());

    let mut occupancy = BoardOccupancy::default();
    occupancy
        .minion_slots
        .insert((player(1), 1), Entity::PLACEHOLDER);
    let occupied = submit(
        &mut pending,
        player(1),
        vec![placement(
            card_id(10),
            PlayTarget::BoardCell { lane: 1, cell: 1 },
            1,
            0,
        )],
        Some(RoundPhase::Placement),
        &catalog,
        &economies,
        &hands,
        &occupancy,
    );
    assert_eq!(occupied, PlacementSubmissionResult::OccupancyRejected);
    assert!(pending.submissions.is_empty());
}

#[test]
fn test_explicit_current_and_reserve_overdraw_reject_full_batch_without_deduction() {
    let catalog = catalog(vec![
        card(10, CardType::Minion, 2),
        card(20, CardType::Minion, 2),
    ]);
    let economies = PlayerEconomies(HashMap::from([(player(1), economy(3, 0))]));
    let hands = hands(player(1), vec![card_id(10), card_id(20)]);
    let mut pending = PendingPlacements::default();

    let reserve_overdraw = submit(
        &mut pending,
        player(1),
        vec![placement(
            card_id(10),
            PlayTarget::BoardCell { lane: 1, cell: 1 },
            1,
            1,
        )],
        Some(RoundPhase::Placement),
        &catalog,
        &economies,
        &hands,
        &BoardOccupancy::default(),
    );
    assert_eq!(
        reserve_overdraw,
        PlacementSubmissionResult::InsufficientMana
    );
    assert!(pending.submissions.is_empty());
    assert_eq!(economies.0[&player(1)].current_mana, 3);
    assert_eq!(economies.0[&player(1)].reserve_mana, 0);

    let current_aggregate_overdraw = submit(
        &mut pending,
        player(1),
        vec![
            placement(
                card_id(10),
                PlayTarget::BoardCell { lane: 1, cell: 1 },
                2,
                0,
            ),
            placement(
                card_id(20),
                PlayTarget::BoardCell { lane: 2, cell: 1 },
                2,
                0,
            ),
        ],
        Some(RoundPhase::Placement),
        &catalog,
        &economies,
        &hands,
        &BoardOccupancy::default(),
    );
    assert_eq!(
        current_aggregate_overdraw,
        PlacementSubmissionResult::InsufficientMana
    );
    assert!(pending.submissions.is_empty());
    assert_eq!(economies.0[&player(1)].current_mana, 3);
}

#[test]
fn test_accepted_batch_preserves_explicit_split_without_deducting_until_close() {
    let catalog = catalog(vec![card(30, CardType::Minion, 3)]);
    let economies = PlayerEconomies(HashMap::from([(player(1), economy(1, 2))]));
    let hands = hands(player(1), vec![card_id(30)]);
    let mut pending = PendingPlacements::default();

    let result = submit(
        &mut pending,
        player(1),
        vec![placement(
            card_id(30),
            PlayTarget::BoardCell { lane: 1, cell: 1 },
            1,
            2,
        )],
        Some(RoundPhase::Placement),
        &catalog,
        &economies,
        &hands,
        &BoardOccupancy::default(),
    );

    assert_eq!(result, PlacementSubmissionResult::Accepted);
    let submission = pending
        .submissions
        .get(&player(1))
        .expect("valid placement should be recorded");
    assert_eq!(
        submission.placements,
        vec![AcceptedPlacement {
            owner_id: player(1),
            card_id: card_id(30),
            target: PlayTarget::BoardCell { lane: 1, cell: 1 },
            current_mana_spend: 1,
            reserve_mana_spend: 2,
        }]
    );
    assert!(submission.is_final);
    assert_eq!(economies.0[&player(1)].current_mana, 1);
    assert_eq!(economies.0[&player(1)].reserve_mana, 2);
}

#[test]
fn test_close_deduction_applies_accepted_explicit_split_exactly() {
    let catalog = catalog(vec![card(40, CardType::Minion, 3)]);
    let mut economies = PlayerEconomies(HashMap::from([(player(1), economy(5, 4))]));
    let committed = HashMap::from([(
        player(1),
        vec![AcceptedPlacement {
            owner_id: player(1),
            card_id: card_id(40),
            target: PlayTarget::BoardCell { lane: 1, cell: 1 },
            current_mana_spend: 1,
            reserve_mana_spend: 2,
        }],
    )]);

    assert!(deduct_committed_mana(&committed, &catalog, &mut economies));
    assert_eq!(economies.0[&player(1)].current_mana, 4);
    assert_eq!(economies.0[&player(1)].reserve_mana, 2);
}
