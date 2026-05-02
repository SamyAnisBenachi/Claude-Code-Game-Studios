use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::ServerPlugins;
use lightyear::prelude::PeerId;
use server::core::pool::{PlayerPool, PlayerPools};
use server::core::rsm::{RoundPhase, RoundState};
use server::core::session::{DeferredMessage, PlayerConnectionMap, ReconnectTracker};
use server::feature::acquisition::PlayerHands;
use server::feature::prism::{
    AuditLog, DiscardLog, PrismCollected, PrismNetworkOutbox, PrismPlugin, PrismState,
};
use server::foundation::config::CardCatalog;
use server::foundation::rng::ServerRng;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::config::GameConfig;
use shared::protocol::{CardSource, S2CPrismRewardDropped};
use shared::session::PlayerId;

const PRISM_STRIKE_ID: CardId = CardId(9001);
const PRISM_RESERVE_ID: CardId = CardId(9002);
const DRAW_SPELL_ID: CardId = CardId(9102);

fn app_with_prism(snapshot_sent: bool) -> App {
    let player_a = PlayerId(1);
    let player_b = PlayerId(2);
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
        pools: HashMap::from([(player_a, player_pool()), (player_b, player_pool())]),
    });
    app.world_mut()
        .insert_resource(PlayerConnectionMap(HashMap::from([
            (PeerId::Netcode(11), player_a),
            (PeerId::Netcode(12), player_b),
        ])));
    app.world_mut().insert_resource(ReconnectTracker {
        snapshot_sent: HashMap::from([(player_a, snapshot_sent), (player_b, snapshot_sent)]),
        deferred_queue: HashMap::new(),
        token_map: HashMap::new(),
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
            card(PRISM_STRIKE_ID, "prism_strike", CardType::Spell, Some(4)),
            card(PRISM_RESERVE_ID, "prism_reserve", CardType::Spell, Some(4)),
            card(DRAW_SPELL_ID, "draw_spell", CardType::Spell, Some(2)),
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
    PlayerPool::initialize(&prism_catalog().cards, &GameConfig::default())
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
fn hand_full_lanes_1_2_4_5_stage_reward_dropped_unicast_without_card() {
    let player = PlayerId(1);
    let mut app = app_with_prism(true);
    app.world_mut()
        .resource_mut::<PlayerHands>()
        .hands
        .insert(player, hand_with_len(10));

    write_collected(&mut app, player, 2);
    app.update();

    assert_eq!(hand(&app, player).len(), 10);
    assert!(app.world().resource::<PrismState>().collected[0][1]);

    let outbox = app.world().resource::<PrismNetworkOutbox>();
    assert!(outbox.card_acquired().is_empty());
    assert_eq!(outbox.reward_dropped().len(), 1);
    let dispatch = &outbox.reward_dropped()[0];
    assert_eq!(dispatch.player_id, player);
    assert_eq!(dispatch.peer_id, Some(PeerId::Netcode(11)));
    assert_eq!(
        dispatch.message,
        S2CPrismRewardDropped {
            player_id: player,
            lane: 2
        }
    );
}

#[test]
fn successful_collection_stages_card_acquired_unicast_to_owner_only() {
    let player = PlayerId(1);
    let mut app = app_with_prism(true);

    write_collected(&mut app, player, 4);
    app.update();

    assert_eq!(hand(&app, player), vec![PRISM_RESERVE_ID]);

    let outbox = app.world().resource::<PrismNetworkOutbox>();
    assert_eq!(outbox.card_acquired().len(), 1);
    assert!(outbox.reward_dropped().is_empty());
    let dispatch = &outbox.card_acquired()[0];
    assert_eq!(dispatch.player_id, player);
    assert_eq!(dispatch.peer_id, Some(PeerId::Netcode(11)));
    assert_ne!(dispatch.peer_id, Some(PeerId::Netcode(12)));
    assert_eq!(dispatch.message.card_id, PRISM_RESERVE_ID);
    assert_eq!(dispatch.message.source, CardSource::PrismLane4);
}

#[test]
fn lane3_success_stages_card_acquired_with_lane3_source() {
    let player = PlayerId(1);
    let mut app = app_with_prism(true);

    write_collected(&mut app, player, 3);
    app.update();

    let outbox = app.world().resource::<PrismNetworkOutbox>();
    assert_eq!(outbox.card_acquired().len(), 1);
    assert!(outbox.reward_dropped().is_empty());
    assert_eq!(
        outbox.card_acquired()[0].message.source,
        CardSource::PrismLane3
    );
    assert_eq!(
        hand(&app, player),
        vec![outbox.card_acquired()[0].message.card_id]
    );
    assert_eq!(
        app.world().resource::<AuditLog>().entries[0].result,
        Some(outbox.card_acquired()[0].message.card_id)
    );
}

#[test]
fn success_and_hand_full_in_same_resolution_stage_independent_messages() {
    let player = PlayerId(1);
    let mut app = app_with_prism(true);
    app.world_mut()
        .resource_mut::<PlayerHands>()
        .hands
        .insert(player, hand_with_len(9));

    write_collected(&mut app, player, 5);
    write_collected(&mut app, player, 1);
    app.update();

    let hand = hand(&app, player);
    assert_eq!(hand.len(), 10);
    assert_eq!(hand.last(), Some(&PRISM_STRIKE_ID));

    let outbox = app.world().resource::<PrismNetworkOutbox>();
    assert_eq!(outbox.card_acquired().len(), 1);
    assert_eq!(outbox.reward_dropped().len(), 1);
    assert_eq!(outbox.card_acquired()[0].message.card_id, PRISM_STRIKE_ID);
    assert_eq!(
        outbox.card_acquired()[0].message.source,
        CardSource::PrismLane1
    );
    assert_eq!(outbox.reward_dropped()[0].message.lane, 5);
}

#[test]
fn lane3_hand_full_is_silent_and_consumes_no_seed() {
    let player = PlayerId(1);
    let mut app = app_with_prism(true);
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
    let outbox = app.world().resource::<PrismNetworkOutbox>();
    assert!(outbox.card_acquired().is_empty());
    assert!(outbox.reward_dropped().is_empty());
}

#[test]
fn snapshot_gate_defers_prism_messages_until_owner_snapshot_is_sent() {
    let player = PlayerId(1);
    let mut app = app_with_prism(false);

    write_collected(&mut app, player, 4);
    app.update();

    let outbox = app.world().resource::<PrismNetworkOutbox>();
    assert!(outbox.card_acquired().is_empty());
    assert!(outbox.reward_dropped().is_empty());

    let tracker = app.world().resource::<ReconnectTracker>();
    let deferred = tracker
        .deferred_queue
        .get(&player)
        .expect("player should have deferred message");
    assert_eq!(deferred.len(), 1);
    match &deferred[0] {
        DeferredMessage::CardAcquired { card_id, source } => {
            assert_eq!(*card_id, PRISM_RESERVE_ID);
            assert_eq!(*source, CardSource::PrismLane4);
        }
        other => panic!("expected CardAcquired, got {other:?}"),
    }
}

#[test]
fn snapshot_gate_defers_reward_dropped_until_owner_snapshot_is_sent() {
    let player = PlayerId(1);
    let mut app = app_with_prism(false);
    app.world_mut()
        .resource_mut::<PlayerHands>()
        .hands
        .insert(player, hand_with_len(10));

    write_collected(&mut app, player, 2);
    app.update();

    let outbox = app.world().resource::<PrismNetworkOutbox>();
    assert!(outbox.card_acquired().is_empty());
    assert!(outbox.reward_dropped().is_empty());

    let tracker = app.world().resource::<ReconnectTracker>();
    let deferred = tracker
        .deferred_queue
        .get(&player)
        .expect("player should have deferred message");
    assert_eq!(deferred.len(), 1);
    match &deferred[0] {
        DeferredMessage::PrismRewardDropped { player_id, lane } => {
            assert_eq!(*player_id, player);
            assert_eq!(*lane, 2);
        }
        other => panic!("expected PrismRewardDropped, got {other:?}"),
    }
}
