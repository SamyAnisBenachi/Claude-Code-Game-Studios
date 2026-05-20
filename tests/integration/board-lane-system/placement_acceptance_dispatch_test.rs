//! PROMPT 1546 — PLACEMENT-ACCEPTED-ACK-PROTOCOL-IMPLEMENTATION.
//!
//! Server side of the accepted-placement ACK contract. The optimistic-Submitted
//! client UI previously had no positive server signal between
//! `C2SSubmitPlacement` and the broadcast `S2CPlacementReveal` at phase close;
//! effect-only placements (Spell/Order/Instant) have neither reveal entries
//! nor entity spawns and were silently accepted with zero S2C surface. This
//! test pins the symmetric ACK lane:
//!
//! - `handle_placement_submission` writes one `PlacementAcceptanceDispatch`
//!   per accepted submission, carrying the submitter's `PlayerId`, the
//!   originating `peer_id`, the server's `placements_len`, and the
//!   `is_final` mirror of `PlayerSubmission`.
//! - The dispatcher schedule still emits exactly ONE `PlacementSubmitted`
//!   internal message per accept (regression guard for the existing
//!   resolution-close pipeline).
//! - Rejected submissions produce zero `PlacementAcceptanceDispatch`.

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::ServerPlugins;
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::rsm::{
    PlacementPhaseEntered, PlacementSubmitted, ResolutionPhaseEntered, RoundPhase, RoundState,
};
use server::core::session::SessionConfig;
use server::feature::acquisition::PlayerHands;
use server::feature::board::{
    BoardPlugin, PlacementAcceptanceDispatch, PlacementRejectionDispatch,
    PlacementSubmissionReceived,
};
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{GameMode, PlacedCardSubmit, PlayTarget};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const TICK_HZ: f64 = 60.0;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn card_id(id: u32) -> CardId {
    CardId(id)
}

fn submitted(card_id: CardId, target: PlayTarget, current_mana_spend: u32) -> PlacedCardSubmit {
    PlacedCardSubmit {
        card_id,
        target,
        current_mana_spend,
        reserve_mana_spend: 0,
    }
}

fn minion_card(id: u32, cost: u32) -> CardData {
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
        atk: 2,
        hp: 2,
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

fn session_config() -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(player(1), 0), (player(2), 1)]),
        class_map: HashMap::from([(player(1), ClassId::Iop), (player(2), ClassId::Ecaflip)]),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn player_hands(catalog_ids: Vec<(PlayerId, Vec<CardId>)>) -> PlayerHands {
    PlayerHands {
        hands: catalog_ids.into_iter().collect(),
    }
}

fn app_with_placement_systems(
    catalog: CardCatalog,
    economies: PlayerEconomies,
    hands: PlayerHands,
) -> App {
    let mut app = App::new();
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    })
    .add_plugins(BoardPlugin)
    .add_message::<PlacementPhaseEntered>()
    .add_message::<ResolutionPhaseEntered>()
    .insert_resource(RoundState {
        phase: RoundPhase::Placement,
        round_number: 2,
        ..RoundState::new()
    })
    .insert_resource(session_config())
    .insert_resource(catalog)
    .insert_resource(economies)
    .insert_resource(hands);
    app
}

fn write_message<T: bevy::prelude::Message>(app: &mut App, message: T) {
    app.world_mut().resource_mut::<Messages<T>>().write(message);
}

fn read_messages<T: bevy::prelude::Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

// =============================================================================
// Acceptance dispatch path: every accepted submission writes exactly one
// `PlacementAcceptanceDispatch` carrying the submitter's PlayerId, peer_id,
// placements_len, and is_final mirror. Also queues exactly one
// `PlacementSubmitted` (regression guard) and zero `PlacementRejectionDispatch`.
// =============================================================================

#[test]
fn accepted_submission_writes_one_acceptance_dispatch_for_submitter() {
    test_helpers::init_test_tracing();

    let mut app = app_with_placement_systems(
        catalog(vec![minion_card(103, 2)]),
        PlayerEconomies(HashMap::from([(player(2), economy(5, 0))])),
        player_hands(vec![(player(2), vec![card_id(103)])]),
    );

    // Player 2 (team 1) — spawn range allows lane 1 cell 8 (back row).
    write_message(
        &mut app,
        PlacementSubmissionReceived {
            player: player(2),
            peer_id: None,
            placements: vec![submitted(
                card_id(103),
                PlayTarget::BoardCell { lane: 1, cell: 8 },
                2,
            )],
        },
    );
    app.update();

    let dispatches = read_messages::<PlacementAcceptanceDispatch>(&app);
    assert_eq!(
        dispatches.len(),
        1,
        "accepted submission MUST queue exactly one PlacementAcceptanceDispatch \
         for the submitting player; got {} dispatches",
        dispatches.len(),
    );
    assert_eq!(dispatches[0].player, player(2));
    assert_eq!(dispatches[0].peer_id, None);
    assert_eq!(dispatches[0].placements_len, 1);
    assert!(
        dispatches[0].is_final,
        "process_placement_submission always commits with is_final=true today",
    );
    assert_eq!(
        read_messages::<PlacementSubmitted>(&app).len(),
        1,
        "accepted submission MUST still queue exactly one PlacementSubmitted",
    );
    assert!(
        read_messages::<PlacementRejectionDispatch>(&app).is_empty(),
        "accepted submission must NOT queue any PlacementRejectionDispatch",
    );
}

#[test]
fn rejected_submission_writes_no_acceptance_dispatch() {
    test_helpers::init_test_tracing();

    // Out-of-spawn-range target (cell=1 is on opponent's side for team 1) →
    // SpawnRangeRejected, so the acceptance lane MUST stay empty.
    let mut app = app_with_placement_systems(
        catalog(vec![minion_card(103, 2)]),
        PlayerEconomies(HashMap::from([(player(2), economy(5, 0))])),
        player_hands(vec![(player(2), vec![card_id(103)])]),
    );

    write_message(
        &mut app,
        PlacementSubmissionReceived {
            player: player(2),
            peer_id: None,
            placements: vec![submitted(
                card_id(103),
                PlayTarget::BoardCell { lane: 1, cell: 1 },
                2,
            )],
        },
    );
    app.update();

    assert!(
        read_messages::<PlacementAcceptanceDispatch>(&app).is_empty(),
        "rejected submission must NOT queue any PlacementAcceptanceDispatch",
    );
    assert_eq!(
        read_messages::<PlacementRejectionDispatch>(&app).len(),
        1,
        "rejected submission still queues exactly one PlacementRejectionDispatch",
    );
}

#[test]
fn acceptance_dispatch_carries_caller_peer_id() {
    test_helpers::init_test_tracing();

    // PeerId comes from PlacementSubmissionReceived; the dispatcher must
    // carry it through unchanged so the unicast S2C lands on the right peer.
    let peer = lightyear::prelude::PeerId::Netcode(7);
    let mut app = app_with_placement_systems(
        catalog(vec![minion_card(103, 2)]),
        PlayerEconomies(HashMap::from([(player(2), economy(5, 0))])),
        player_hands(vec![(player(2), vec![card_id(103)])]),
    );

    write_message(
        &mut app,
        PlacementSubmissionReceived {
            player: player(2),
            peer_id: Some(peer),
            placements: vec![submitted(
                card_id(103),
                PlayTarget::BoardCell { lane: 1, cell: 8 },
                2,
            )],
        },
    );
    app.update();

    let dispatches = read_messages::<PlacementAcceptanceDispatch>(&app);
    assert_eq!(dispatches.len(), 1);
    assert_eq!(dispatches[0].peer_id, Some(peer));
}

#[test]
fn acceptance_dispatch_reports_full_placements_len() {
    test_helpers::init_test_tracing();

    // Two-card batch on the same lane (different cells) — both must accept.
    let mut app = app_with_placement_systems(
        catalog(vec![minion_card(201, 1), minion_card(202, 1)]),
        PlayerEconomies(HashMap::from([(player(2), economy(5, 0))])),
        player_hands(vec![(player(2), vec![card_id(201), card_id(202)])]),
    );

    write_message(
        &mut app,
        PlacementSubmissionReceived {
            player: player(2),
            peer_id: None,
            placements: vec![
                submitted(card_id(201), PlayTarget::BoardCell { lane: 1, cell: 8 }, 1),
                submitted(card_id(202), PlayTarget::BoardCell { lane: 2, cell: 8 }, 1),
            ],
        },
    );
    app.update();

    let dispatches = read_messages::<PlacementAcceptanceDispatch>(&app);
    assert_eq!(dispatches.len(), 1, "one dispatch per batch, not per card");
    assert_eq!(
        dispatches[0].placements_len, 2,
        "placements_len must mirror the accepted batch size",
    );
}
