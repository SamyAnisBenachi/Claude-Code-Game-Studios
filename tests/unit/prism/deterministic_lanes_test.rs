use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::ServerPlugins;
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::rsm::{RoundPhase, RoundState};
use server::feature::acquisition::PlayerHands;
use server::feature::prism::{
    AuditLog, DiscardLog, PrismCollected, PrismLaneKey, PrismPlugin, PrismPresence, PrismState,
};
use server::foundation::config::CardCatalog;
use server::foundation::rng::ServerRng;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::session::PlayerId;

const PRISM_STRIKE_ID: CardId = CardId(9001);
const PRISM_RESERVE_ID: CardId = CardId(9002);

fn app_with_prism() -> App {
    let mut app = App::new();
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / 60.0),
    })
    .add_plugins(PrismPlugin);
    app.world_mut().insert_resource(PrismState::default());
    app.world_mut().insert_resource(DiscardLog::default());
    app.world_mut().insert_resource(AuditLog::default());
    app.world_mut().insert_resource(PlayerHands::default());
    app.world_mut().insert_resource(prism_catalog());
    app.world_mut()
        .insert_resource(round_state(RoundPhase::Resolution));
    app.world_mut().insert_resource(ServerRng::new());
    app
}

fn round_state(phase: RoundPhase) -> RoundState {
    RoundState {
        phase,
        ..RoundState::default()
    }
}

fn prism_catalog() -> CardCatalog {
    CardCatalog {
        cards: [
            prism_card(PRISM_STRIKE_ID, "prism_strike", 3),
            prism_card(PRISM_RESERVE_ID, "prism_reserve", 0),
        ]
        .into_iter()
        .map(|card| (card.id, card))
        .collect(),
    }
}

fn prism_card(id: CardId, art_id: &str, cost: u32) -> CardData {
    CardData {
        id,
        name_fr: art_id.to_string(),
        name_en: art_id.to_string(),
        class: ClassId::Neutral,
        family: None,
        rarity: Rarity::Common,
        card_type: CardType::Spell,
        unit_type: UnitType::Neutral,
        cost,
        atk: 0,
        hp: 0,
        mp: 0,
        ar: 0,
        keywords: vec![],
        effect_text: String::new(),
        art_id: art_id.to_string(),
        pool_copies_override: Some(1),
    }
}

fn write_collected(app: &mut App, player_id: PlayerId, lane: u8) {
    app.world_mut()
        .resource_mut::<Messages<PrismCollected>>()
        .write(PrismCollected { player_id, lane });
}

fn hand(app: &App, player_id: PlayerId) -> Vec<CardId> {
    app.world()
        .resource::<PlayerHands>()
        .hands
        .get(&player_id)
        .cloned()
        .unwrap_or_default()
}

fn economy(gold: u32, current_mana: u32, reserve_mana: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold,
        current_mana,
        reserve_mana,
        mana_cap: 10,
        reserved_gold: 0,
    }
}

fn hand_with_len(len: u32) -> Vec<CardId> {
    (1..=len).map(CardId).collect()
}

#[test]
fn lane_1_and_5_grant_prism_strike_without_rng_or_gold_mutation() {
    let player = PlayerId(1);
    let mut app = app_with_prism();
    app.world_mut()
        .insert_resource(PlayerEconomies(HashMap::from([(player, economy(8, 4, 2))])));
    let seed_index_before = app.world().resource::<ServerRng>().current_seed_index();

    write_collected(&mut app, player, 5);
    write_collected(&mut app, player, 1);
    app.update();

    let state = app.world().resource::<PrismState>();
    assert!(state.collected[0][0]);
    assert!(state.collected[0][4]);
    assert_eq!(hand(&app, player), vec![PRISM_STRIKE_ID, PRISM_STRIKE_ID]);
    assert_eq!(
        app.world().resource::<ServerRng>().current_seed_index(),
        seed_index_before
    );
    let economy = app
        .world()
        .resource::<PlayerEconomies>()
        .0
        .get(&player)
        .expect("economy exists");
    assert_eq!(economy.gold, 8);
    assert_eq!(economy.current_mana, 4);
    assert_eq!(economy.reserve_mana, 2);
    assert!(app.world().resource::<DiscardLog>().entries.is_empty());
    assert!(app.world().resource::<AuditLog>().entries.is_empty());
}

#[test]
fn lane_2_and_4_grant_prism_reserve_and_presence_is_marked_collected() {
    let player = PlayerId(1);
    let mut app = app_with_prism();
    for lane in [2, 4] {
        app.world_mut().spawn((
            PrismLaneKey { player, lane },
            PrismPresence { collected: false },
        ));
        write_collected(&mut app, player, lane);
    }

    app.update();

    let state = app.world().resource::<PrismState>();
    assert!(state.collected[0][1]);
    assert!(state.collected[0][3]);
    assert_eq!(hand(&app, player), vec![PRISM_RESERVE_ID, PRISM_RESERVE_ID]);

    let mut query = app.world_mut().query::<(&PrismLaneKey, &PrismPresence)>();
    let collected_presence = query
        .iter(app.world())
        .filter(|(key, presence)| {
            key.player == player && [2, 4].contains(&key.lane) && presence.collected
        })
        .count();
    assert_eq!(collected_presence, 2);
}

#[test]
fn lane_4_at_nine_cards_adds_prism_reserve_as_tenth_card() {
    let player = PlayerId(1);
    let mut app = app_with_prism();
    app.world_mut()
        .resource_mut::<PlayerHands>()
        .hands
        .insert(player, hand_with_len(9));

    write_collected(&mut app, player, 4);
    app.update();

    let hand = hand(&app, player);
    assert_eq!(hand.len(), 10);
    assert_eq!(hand.last(), Some(&PRISM_RESERVE_ID));
    assert!(app.world().resource::<PrismState>().collected[0][3]);
}

#[test]
fn wall_originated_collection_message_is_processed_like_any_other_lane_event() {
    let player = PlayerId(1);
    let mut app = app_with_prism();

    write_collected(&mut app, player, 2);
    app.update();

    assert!(app.world().resource::<PrismState>().collected[0][1]);
    assert_eq!(hand(&app, player), vec![PRISM_RESERVE_ID]);
}

#[test]
fn stale_duplicate_is_discarded_without_reward_or_rng_consumption() {
    let player = PlayerId(1);
    let mut app = app_with_prism();
    app.world_mut().resource_mut::<PrismState>().collected[0][0] = true;
    let seed_index_before = app.world().resource::<ServerRng>().current_seed_index();

    write_collected(&mut app, player, 1);
    app.update();

    assert!(hand(&app, player).is_empty());
    assert_eq!(
        app.world().resource::<ServerRng>().current_seed_index(),
        seed_index_before
    );
    assert_eq!(
        app.world().resource::<DiscardLog>().entries,
        vec![(player, 1)]
    );
    assert!(app.world().resource::<AuditLog>().entries.is_empty());
}

#[test]
fn pending_messages_are_processed_by_lane_order_within_each_player() {
    let mut app = app_with_prism();
    let player_a = PlayerId(1);
    let player_b = PlayerId(2);

    write_collected(&mut app, player_b, 4);
    write_collected(&mut app, player_a, 2);
    write_collected(&mut app, player_a, 5);
    write_collected(&mut app, player_a, 1);
    write_collected(&mut app, player_b, 2);
    app.update();

    assert_eq!(
        hand(&app, player_a),
        vec![PRISM_STRIKE_ID, PRISM_RESERVE_ID, PRISM_STRIKE_ID]
    );
    assert_eq!(
        hand(&app, player_b),
        vec![PRISM_RESERVE_ID, PRISM_RESERVE_ID]
    );
}
