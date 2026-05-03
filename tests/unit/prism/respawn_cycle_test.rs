use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::ServerPlugins;
use lightyear::prelude::PeerId;
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::pool::{PlayerPool, PlayerPools};
use server::core::rsm::{RoundPhase, RoundState};
use server::core::session::{PlayerConnectionMap, SessionConfig};
use server::feature::acquisition::PlayerHands;
use server::feature::prism::{
    AuditLog, DiscardLog, PrismCollected, PrismLaneKey, PrismNetworkDispatch, PrismNetworkOutbox,
    PrismPlugin, PrismPresence, PrismState, PRISM_LANE_COUNT,
};
use server::foundation::config::CardCatalog;
use server::foundation::rng::ServerRng;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::config::GameConfig;
use shared::protocol::{CardSource, GameMode, S2CPrismRespawned};
use shared::session::PlayerId;

const PRISM_STRIKE_ID: CardId = CardId(9001);
const PRISM_RESERVE_ID: CardId = CardId(9002);
const DRAW_SPELL_ID: CardId = CardId(9102);
const DRAW_TRAP_ID: CardId = CardId(9103);

fn app_with_prism(session: Option<SessionConfig>) -> App {
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
        pools: (1..=4)
            .map(|player| (PlayerId(player), player_pool()))
            .collect(),
    });
    app.world_mut()
        .insert_resource(PlayerConnectionMap(HashMap::from([
            (PeerId::Netcode(11), PlayerId(1)),
            (PeerId::Netcode(12), PlayerId(2)),
            (PeerId::Netcode(13), PlayerId(3)),
            (PeerId::Netcode(14), PlayerId(4)),
        ])));

    if let Some(session) = session {
        app.world_mut().insert_resource(session);
    }

    app
}

fn round_state(phase: RoundPhase) -> RoundState {
    RoundState {
        phase,
        ..RoundState::default()
    }
}

fn two_v_two_same_team_session() -> SessionConfig {
    SessionConfig {
        mode: GameMode::TwoVTwo,
        player_count: 4,
        team_map: HashMap::from([
            (PlayerId(1), 0),
            (PlayerId(2), 0),
            (PlayerId(3), 1),
            (PlayerId(4), 1),
        ]),
        class_map: HashMap::from([
            (PlayerId(1), ClassId::Iop),
            (PlayerId(2), ClassId::Cra),
            (PlayerId(3), ClassId::Sacrier),
            (PlayerId(4), ClassId::Xelor),
        ]),
    }
}

fn prism_catalog() -> CardCatalog {
    CardCatalog {
        cards: [
            card(PRISM_STRIKE_ID, "prism_strike", CardType::Spell, Some(1)),
            card(PRISM_RESERVE_ID, "prism_reserve", CardType::Spell, Some(1)),
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
        .expect("static prism strike is removed from Lane 3 pool");
    pool.distribute(PRISM_RESERVE_ID)
        .expect("static prism reserve is removed from Lane 3 pool");
    pool.distribute(DRAW_TRAP_ID)
        .expect("trap should not be eligible for Lane 3");
    pool
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

fn spawn_presence(app: &mut App, player: PlayerId, collected: [bool; PRISM_LANE_COUNT]) {
    for (lane_index, collected) in collected.into_iter().enumerate() {
        app.world_mut().spawn((
            PrismLaneKey {
                player,
                lane: u8::try_from(lane_index + 1).expect("prism lane fits in u8"),
            },
            PrismPresence { collected },
        ));
    }
}

fn presence_for(app: &mut App, player: PlayerId) -> Vec<(u8, bool)> {
    let mut query = app.world_mut().query::<(&PrismLaneKey, &PrismPresence)>();
    let mut entries = query
        .iter(app.world())
        .filter_map(|(key, presence)| {
            (key.player == player).then_some((key.lane, presence.collected))
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|(lane, _)| *lane);
    entries
}

#[test]
fn full_set_respawn_resets_only_completed_player_after_reward_delivery() {
    let player_a = PlayerId(1);
    let player_b = PlayerId(2);
    let mut app = app_with_prism(None);
    app.world_mut()
        .insert_resource(PlayerEconomies(HashMap::from([
            (player_a, economy(8, 4, 2)),
            (player_b, economy(3, 1, 5)),
        ])));
    app.world_mut().resource_mut::<PrismState>().collected[0] = [true, true, true, true, false];
    app.world_mut().resource_mut::<PrismState>().collected[1] = [true, true, false, false, false];
    spawn_presence(&mut app, player_a, [true, true, true, true, false]);
    spawn_presence(&mut app, player_b, [true, true, false, false, false]);

    write_collected(&mut app, player_a, 5);
    app.update();

    let state = app.world().resource::<PrismState>();
    assert_eq!(state.collected[0], [false; PRISM_LANE_COUNT]);
    assert_eq!(state.collected[1], [true, true, false, false, false]);
    assert_eq!(
        state.pending_respawn,
        [false; server::feature::prism::MAX_PLAYERS]
    );
    assert_eq!(hand(&app, player_a), vec![PRISM_STRIKE_ID]);
    assert!(hand(&app, player_b).is_empty());

    let player_a_economy = app
        .world()
        .resource::<PlayerEconomies>()
        .0
        .get(&player_a)
        .expect("player economy exists");
    assert_eq!(player_a_economy.gold, 8);
    assert_eq!(player_a_economy.current_mana, 4);
    assert_eq!(player_a_economy.reserve_mana, 2);

    assert_eq!(
        presence_for(&mut app, player_a),
        vec![(1, false), (2, false), (3, false), (4, false), (5, false)]
    );
    assert_eq!(
        presence_for(&mut app, player_b),
        vec![(1, true), (2, true), (3, false), (4, false), (5, false)]
    );

    let outbox = app.world().resource::<PrismNetworkOutbox>();
    assert_eq!(outbox.card_acquired().len(), 1);
    assert!(outbox.reward_dropped().is_empty());
    assert_eq!(outbox.respawned().len(), 1);
    assert_eq!(
        outbox.respawned()[0].message,
        S2CPrismRespawned {
            player_id: player_a
        }
    );
    assert!(matches!(
        &outbox.sequence()[0],
        PrismNetworkDispatch::CardAcquired(dispatch)
            if dispatch.player_id == player_a
                && dispatch.message.source == CardSource::PrismLane5
    ));
    assert!(matches!(
        &outbox.sequence()[1],
        PrismNetworkDispatch::Respawned(dispatch) if dispatch.player_id == player_a
    ));
}

#[test]
fn two_players_can_respawn_independently_in_the_same_resolution() {
    let player_a = PlayerId(1);
    let player_b = PlayerId(2);
    let mut app = app_with_prism(None);
    app.world_mut().resource_mut::<PrismState>().collected[0] = [true, true, true, true, false];
    app.world_mut().resource_mut::<PrismState>().collected[1] = [true, true, true, true, false];

    write_collected(&mut app, player_b, 5);
    write_collected(&mut app, player_a, 5);
    app.update();

    let state = app.world().resource::<PrismState>();
    assert_eq!(state.collected[0], [false; PRISM_LANE_COUNT]);
    assert_eq!(state.collected[1], [false; PRISM_LANE_COUNT]);

    let respawned = app
        .world()
        .resource::<PrismNetworkOutbox>()
        .respawned()
        .iter()
        .map(|dispatch| dispatch.player_id)
        .collect::<Vec<_>>();
    assert_eq!(respawned, vec![player_a, player_b]);
}

#[test]
fn partial_opponent_progress_survives_other_player_respawn() {
    let player_a = PlayerId(1);
    let player_b = PlayerId(2);
    let mut app = app_with_prism(None);
    app.world_mut().resource_mut::<PrismState>().collected[0] = [true, true, true, true, false];
    app.world_mut().resource_mut::<PrismState>().collected[1] = [true, true, true, false, false];

    write_collected(&mut app, player_b, 4);
    write_collected(&mut app, player_a, 5);
    app.update();

    let state = app.world().resource::<PrismState>();
    assert_eq!(state.collected[0], [false; PRISM_LANE_COUNT]);
    assert_eq!(state.collected[1], [true, true, true, true, false]);

    let outbox = app.world().resource::<PrismNetworkOutbox>();
    assert_eq!(outbox.card_acquired().len(), 2);
    assert_eq!(outbox.respawned().len(), 1);
    assert_eq!(outbox.respawned()[0].player_id, player_a);
    assert!(matches!(
        outbox.sequence(),
        [
            PrismNetworkDispatch::CardAcquired(_),
            PrismNetworkDispatch::CardAcquired(_),
            PrismNetworkDispatch::Respawned(_)
        ]
    ));
}

#[test]
fn same_team_players_collect_same_lane_independently_in_two_v_two_session() {
    let player_a = PlayerId(1);
    let player_b = PlayerId(2);
    let mut app = app_with_prism(Some(two_v_two_same_team_session()));

    write_collected(&mut app, player_b, 3);
    write_collected(&mut app, player_a, 3);
    app.update();

    let state = app.world().resource::<PrismState>();
    assert!(state.collected[0][2]);
    assert!(state.collected[1][2]);
    assert_eq!(hand(&app, player_a), vec![DRAW_SPELL_ID]);
    assert_eq!(hand(&app, player_b), vec![DRAW_SPELL_ID]);

    let audit = &app.world().resource::<AuditLog>().entries;
    assert_eq!(audit.len(), 2);
    assert_eq!(audit[0].player_id, player_a);
    assert_eq!(audit[1].player_id, player_b);

    let outbox = app.world().resource::<PrismNetworkOutbox>();
    assert_eq!(outbox.card_acquired().len(), 2);
    assert!(outbox.respawned().is_empty());
}

#[test]
fn same_team_prism_state_without_events_does_not_bleed_between_players() {
    let player_a = PlayerId(1);
    let player_b = PlayerId(2);
    let mut app = app_with_prism(Some(two_v_two_same_team_session()));
    app.world_mut().resource_mut::<PrismState>().collected[0][2] = true;
    app.world_mut().resource_mut::<PrismState>().collected[1][2] = false;

    app.update();

    let state = app.world().resource::<PrismState>();
    assert!(state.collected[0][2]);
    assert!(!state.collected[1][2]);
    assert!(hand(&app, player_a).is_empty());
    assert!(hand(&app, player_b).is_empty());
    assert!(app
        .world()
        .resource::<PrismNetworkOutbox>()
        .sequence()
        .is_empty());
}
