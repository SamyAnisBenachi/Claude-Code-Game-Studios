use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::ServerPlugins;
use server::core::pool::{PlayerPool, PlayerPools};
use server::core::rsm::{RoundPhase, RoundState};
use server::feature::acquisition::PlayerHands;
use server::feature::prism::{AuditLog, DiscardLog, PrismCollected, PrismPlugin, PrismState};
use server::foundation::config::CardCatalog;
use server::foundation::rng::ServerRng;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::config::GameConfig;
use shared::session::PlayerId;

const PRISM_STRIKE_ID: CardId = CardId(9001);
const DRAW_MINION_ID: CardId = CardId(9101);
const DRAW_SPELL_ID: CardId = CardId(9102);
const DRAW_TRAP_ID: CardId = CardId(9103);

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
    app.world_mut().insert_resource(PlayerPools {
        pools: HashMap::from([(PlayerId(1), player_pool()), (PlayerId(2), player_pool())]),
    });
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
            card(PRISM_STRIKE_ID, "prism_strike", CardType::Spell, Some(1)),
            card(DRAW_MINION_ID, "draw_minion", CardType::Minion, Some(1)),
            card(DRAW_SPELL_ID, "draw_spell", CardType::Spell, Some(1)),
            card(DRAW_TRAP_ID, "draw_trap", CardType::Trap, Some(1)),
        ]
        .into_iter()
        .map(|card| (card.id, card))
        .collect(),
    }
}

fn card(id: CardId, art_id: &str, card_type: CardType, copies: Option<i32>) -> CardData {
    CardData {
        id,
        name_fr: art_id.to_string(),
        name_en: art_id.to_string(),
        class: ClassId::Neutral,
        family: None,
        rarity: Rarity::Common,
        card_type,
        unit_type: UnitType::Neutral,
        cost: 1,
        atk: 0,
        hp: 0,
        mp: 0,
        ar: 0,
        keywords: vec![],
        effect_text: String::new(),
        art_id: art_id.to_string(),
        pool_copies_override: copies,
    }
}

fn player_pool() -> PlayerPool {
    let mut pool = PlayerPool::initialize(&prism_catalog().cards, &GameConfig::default());
    pool.distribute(PRISM_STRIKE_ID)
        .expect("static prism card is not part of Lane 3 draw pool");
    pool.distribute(DRAW_MINION_ID)
        .expect("minion starts available");
    pool.distribute(DRAW_TRAP_ID)
        .expect("trap should not be eligible for Lane 3");
    pool
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

fn hand_with_len(len: u32) -> Vec<CardId> {
    (1..=len).map(CardId).collect()
}

#[test]
fn lane3_success_draws_minion_or_spell_adds_to_hand_and_consumes_one_seed() {
    let player = PlayerId(1);
    let mut app = app_with_prism();
    let seed_index_before = app.world().resource::<ServerRng>().current_seed_index();

    write_collected(&mut app, player, 3);
    app.update();

    let state = app.world().resource::<PrismState>();
    assert!(state.collected[0][2]);
    assert_eq!(hand(&app, player), vec![DRAW_SPELL_ID]);
    assert_eq!(
        app.world().resource::<ServerRng>().current_seed_index(),
        seed_index_before + 1
    );
    assert_eq!(
        app.world().resource::<AuditLog>().entries,
        vec![server::feature::prism::PrismAuditEntry {
            player_id: player,
            lane: 3,
            seed_index: seed_index_before,
            result: Some(DRAW_SPELL_ID),
        }]
    );
    let pool = app
        .world()
        .resource::<PlayerPools>()
        .pools
        .get(&player)
        .expect("player pool exists");
    assert_eq!(pool.copies_remaining(DRAW_SPELL_ID), 0);
    assert_eq!(pool.copies_remaining(DRAW_TRAP_ID), 0);
}

#[test]
fn lane3_hand_full_marks_collected_without_seed_draw_or_audit_entry() {
    let player = PlayerId(1);
    let mut app = app_with_prism();
    app.world_mut()
        .resource_mut::<PlayerHands>()
        .hands
        .insert(player, hand_with_len(10));
    let seed_index_before = app.world().resource::<ServerRng>().current_seed_index();

    write_collected(&mut app, player, 3);
    app.update();

    assert!(app.world().resource::<PrismState>().collected[0][2]);
    assert_eq!(hand(&app, player).len(), 10);
    assert_eq!(
        app.world().resource::<ServerRng>().current_seed_index(),
        seed_index_before
    );
    assert!(app.world().resource::<AuditLog>().entries.is_empty());
    let pool = app
        .world()
        .resource::<PlayerPools>()
        .pools
        .get(&player)
        .expect("player pool exists");
    assert_eq!(pool.copies_remaining(DRAW_SPELL_ID), 1);
}

#[test]
fn lane3_pool_exhausted_consumes_seed_and_logs_none_without_hand_add() {
    let player = PlayerId(1);
    let mut app = app_with_prism();
    {
        let mut pools = app.world_mut().resource_mut::<PlayerPools>();
        let pool = pools.pools.get_mut(&player).expect("player pool exists");
        pool.distribute(DRAW_SPELL_ID)
            .expect("spell starts available");
    }
    let seed_index_before = app.world().resource::<ServerRng>().current_seed_index();

    write_collected(&mut app, player, 3);
    app.update();

    assert!(app.world().resource::<PrismState>().collected[0][2]);
    assert!(hand(&app, player).is_empty());
    assert_eq!(
        app.world().resource::<ServerRng>().current_seed_index(),
        seed_index_before + 1
    );
    assert_eq!(
        app.world().resource::<AuditLog>().entries,
        vec![server::feature::prism::PrismAuditEntry {
            player_id: player,
            lane: 3,
            seed_index: seed_index_before,
            result: None,
        }]
    );
}

#[test]
fn lane3_audit_entries_follow_player_then_lane_processing_order() {
    let mut app = app_with_prism();
    let player_a = PlayerId(1);
    let player_b = PlayerId(2);
    let seed_index_before = app.world().resource::<ServerRng>().current_seed_index();

    write_collected(&mut app, player_b, 3);
    write_collected(&mut app, player_a, 3);
    write_collected(&mut app, player_a, 1);
    app.update();

    let entries = &app.world().resource::<AuditLog>().entries;
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].player_id, player_a);
    assert_eq!(entries[0].lane, 3);
    assert_eq!(entries[0].seed_index, seed_index_before);
    assert_eq!(entries[1].player_id, player_b);
    assert_eq!(entries[1].lane, 3);
    assert_eq!(entries[1].seed_index, seed_index_before + 1);
    assert_eq!(hand(&app, player_a), vec![PRISM_STRIKE_ID, DRAW_SPELL_ID]);
    assert_eq!(hand(&app, player_b), vec![DRAW_SPELL_ID]);
}
