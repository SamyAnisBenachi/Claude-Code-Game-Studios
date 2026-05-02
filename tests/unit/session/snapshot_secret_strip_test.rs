use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::pool::{PlayerPool, PlayerPools};
use server::core::rsm::{RoundPhase, RoundState};
use server::core::session::{
    build_game_snapshot, PlayerSessionData, PlayerSessions, SessionConfig,
};
use server::feature::acquisition::{PlayerHands, PlayerShopState, ShopPhase, ShopStates};
use server::feature::objective::{
    HiddenObjectives, ObjectiveHp, ObjectiveSlot, OBJECTIVE_LANE_COUNT,
};
use server::foundation::config::GameConfig;
use shared::card::{CardId, ClassId};
use shared::protocol::RoundPhase as ProtocolRoundPhase;
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn insert_session(world: &mut World, players: &[PlayerId]) {
    let mut team_map = HashMap::new();
    let mut class_map = HashMap::new();
    let mut sessions = PlayerSessions::default();

    for (index, player_id) in players.iter().copied().enumerate() {
        let class = if index == 0 {
            ClassId::Iop
        } else {
            ClassId::Cra
        };
        team_map.insert(player_id, index as u8);
        class_map.insert(player_id, class);
        sessions.players.insert(
            player_id,
            PlayerSessionData {
                class,
                class_locked: true,
            },
        );
    }

    world.insert_resource(SessionConfig {
        mode: shared::protocol::GameMode::OneVOne,
        player_count: players.len() as u8,
        team_map,
        class_map,
    });
    world.insert_resource(sessions);
}

fn insert_round_state(world: &mut World, submitted_player: PlayerId) {
    let mut placement_timer = Timer::from_seconds(10.0, TimerMode::Once);
    placement_timer.tick(Duration::from_secs(4));

    let mut state = RoundState {
        phase: RoundPhase::Placement,
        round_number: 7,
        placement_timer: Some(placement_timer),
        ..RoundState::new()
    };
    state.submissions_received.insert(submitted_player);
    world.insert_resource(state);
}

fn insert_economy(world: &mut World, player_a: PlayerId, player_b: PlayerId) {
    world.insert_resource(PlayerEconomies(HashMap::from([
        (
            player_a,
            PlayerEconomy {
                gold: 15,
                current_mana: 3,
                reserve_mana: 1,
                mana_cap: 10,
                reserved_gold: 2,
            },
        ),
        (
            player_b,
            PlayerEconomy {
                gold: 20,
                current_mana: 6,
                reserve_mana: 0,
                mana_cap: 11,
                reserved_gold: 0,
            },
        ),
    ])));
}

fn insert_private_card_state(world: &mut World, player_a: PlayerId, player_b: PlayerId) {
    world.insert_resource(PlayerHands {
        hands: HashMap::from([
            (player_a, vec![CardId(101)]),
            (player_b, vec![CardId(201), CardId(202)]),
        ]),
    });

    world.insert_resource(ShopStates {
        players: HashMap::from([
            (
                player_a,
                PlayerShopState {
                    phase: ShopPhase::ShopActive,
                    current_slots: [Some(CardId(301)), None, Some(CardId(302))],
                    ..PlayerShopState::default()
                },
            ),
            (
                player_b,
                PlayerShopState {
                    phase: ShopPhase::ShopActive,
                    current_slots: [Some(CardId(401)), Some(CardId(402)), None],
                    ..PlayerShopState::default()
                },
            ),
        ]),
    });

    world.insert_resource(PlayerPools {
        pools: HashMap::from([
            (
                player_a,
                PlayerPool {
                    copies_remaining: HashMap::from([(CardId(501), 2)]),
                    initial_count: HashMap::new(),
                    shop_slots: Vec::new(),
                },
            ),
            (
                player_b,
                PlayerPool {
                    copies_remaining: HashMap::from([(CardId(601), 4), (CardId(602), 1)]),
                    initial_count: HashMap::new(),
                    shop_slots: Vec::new(),
                },
            ),
        ]),
    });
}

fn insert_objectives(world: &mut World, player_a: PlayerId, player_b: PlayerId) {
    let mut hidden = HiddenObjectives::default();

    for owner in [player_a, player_b] {
        for lane in 1..=OBJECTIVE_LANE_COUNT {
            let is_fake = lane == 2 || (owner == player_b && lane == 3);
            hidden.identities.insert((owner, lane), is_fake);

            let destroyed = owner == player_a && lane == 2;
            world.spawn((
                ObjectiveSlot {
                    lane,
                    player: owner,
                    destroyed,
                },
                ObjectiveHp {
                    hp: if destroyed { 0 } else { 5 },
                },
            ));
        }
    }

    world.insert_resource(hidden);
}

#[test]
fn snapshot_populates_hud_fields_for_recipient() {
    let player_a = player(1);
    let player_b = player(2);
    let mut world = World::new();
    world.insert_resource(GameConfig(shared::config::GameConfig::default()));
    insert_session(&mut world, &[player_a, player_b]);
    insert_round_state(&mut world, player_b);
    insert_economy(&mut world, player_a, player_b);
    insert_private_card_state(&mut world, player_a, player_b);
    insert_objectives(&mut world, player_a, player_b);

    let snapshot = build_game_snapshot(player_b, &mut world).expect("snapshot builds");

    assert_eq!(snapshot.protocol_version, 1);
    assert_eq!(snapshot.recipient_player_id, player_b);
    assert_eq!(snapshot.round_number, 7);
    assert_eq!(snapshot.phase, ProtocolRoundPhase::Placement);
    assert_eq!(snapshot.timer_remaining_ms, Some(6000));

    let own = snapshot
        .players
        .iter()
        .find(|player| player.player_id == player_b)
        .expect("recipient snapshot exists");
    assert_eq!(own.gold, 20);
    assert_eq!(own.reserved_gold, 0);
    assert_eq!(own.current_mana, 6);
    assert_eq!(own.reserve_mana, 0);
    assert_eq!(own.mana_cap, 11);
    assert!(own.submitted);
    assert_eq!(own.hand, vec![CardId(201), CardId(202)]);
    assert_eq!(
        own.shop_slots,
        vec![Some(CardId(401)), Some(CardId(402)), None]
    );
    assert_eq!(own.pool_snapshot, vec![(CardId(601), 4), (CardId(602), 1)]);
    assert_eq!(own.objectives.len(), usize::from(OBJECTIVE_LANE_COUNT));
    assert!(own.objectives.iter().any(|objective| objective.is_real));
}

#[test]
fn snapshot_strips_opponent_private_state_and_identity() {
    let player_a = player(1);
    let player_b = player(2);
    let mut world = World::new();
    world.insert_resource(GameConfig(shared::config::GameConfig::default()));
    insert_session(&mut world, &[player_a, player_b]);
    insert_round_state(&mut world, player_b);
    insert_economy(&mut world, player_a, player_b);
    insert_private_card_state(&mut world, player_a, player_b);
    insert_objectives(&mut world, player_a, player_b);

    let snapshot = build_game_snapshot(player_b, &mut world).expect("snapshot builds");
    let opponent = snapshot
        .players
        .iter()
        .find(|player| player.player_id == player_a)
        .expect("opponent snapshot exists");

    assert_eq!(opponent.gold, 15);
    assert_eq!(opponent.reserved_gold, 2);
    assert_eq!(opponent.hand, Vec::<CardId>::new());
    assert_eq!(opponent.shop_slots, Vec::<Option<CardId>>::new());
    assert_eq!(opponent.pool_snapshot, Vec::<(CardId, u8)>::new());
    assert!(opponent
        .objectives
        .iter()
        .all(|objective| !objective.is_real));

    let own = snapshot
        .players
        .iter()
        .find(|player| player.player_id == player_b)
        .expect("recipient snapshot exists");
    let destroyed_opponent_objective = own
        .opponent_objectives
        .iter()
        .find(|objective| objective.lane == 2)
        .expect("destroyed opponent objective present");
    assert!(destroyed_opponent_objective.is_destroyed);
    assert_eq!(destroyed_opponent_objective.was_fake, Some(true));
}
