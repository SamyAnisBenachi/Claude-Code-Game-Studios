use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use server::core::rsm::{GameOverEmitted, RoundPhase, RoundState, RsmPlugin};
use server::core::session::{
    ActiveSessions, ClassPreviews, ClassSelections, DeferredMessage, EndedSessionResultState,
    GameSessionPlugin, LobbyDeadline, LobbyHeartbeats, LobbyState, ReconnectTracker, RoomCode,
    RoomSession, RoomSessions, SessionConfig, SessionId, SessionSlot, SessionSlots,
};
use server::foundation::rng::ServerRng;
use shared::card::{CardId, ClassId};
use shared::protocol::CardSource;
use shared::protocol::{GameMode, GameOverReason};
use shared::session::PlayerId;
use uuid::Uuid;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_id(value: u128) -> SessionId {
    SessionId(Uuid::from_u128(value))
}

fn session_slots() -> SessionSlots {
    SessionSlots(vec![
        SessionSlot {
            index: 0,
            team: 0,
            player: Some(player(1)),
            class: Some(ClassId::Iop),
        },
        SessionSlot {
            index: 1,
            team: 1,
            player: Some(player(2)),
            class: Some(ClassId::Cra),
        },
    ])
}

fn class_selections() -> ClassSelections {
    ClassSelections(HashMap::from([
        (player(1), ClassId::Iop),
        (player(2), ClassId::Cra),
    ]))
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

fn room_sessions() -> RoomSessions {
    let mut rooms = RoomSessions::default();
    rooms.insert(RoomSession {
        session_id: session_id(1),
        room_code: RoomCode("ABCDEF".to_string()),
        owner: player(1),
        mode: GameMode::OneVOne,
        state: LobbyState::GameActive,
        slots: session_slots(),
        lobby_deadline: LobbyDeadline(90.0),
        heartbeats: LobbyHeartbeats(HashMap::from([(player(1), 0.0), (player(2), 0.0)])),
    });
    rooms
}

fn active_sessions() -> ActiveSessions {
    ActiveSessions(HashMap::from([
        (player(1), session_id(1)),
        (player(2), session_id(1)),
        (player(9), session_id(9)),
    ]))
}

fn game_active_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RsmPlugin);
    app.add_plugins(GameSessionPlugin);
    app.insert_resource(LobbyState::GameActive);
    app.insert_resource(session_config());
    app.insert_resource(ServerRng::new());
    app.insert_resource(session_slots());
    app.insert_resource(class_selections());
    app.insert_resource(ClassPreviews(HashMap::from([(player(1), ClassId::Iop)])));
    app.insert_resource(LobbyDeadline(90.0));
    app.insert_resource(LobbyHeartbeats(HashMap::from([
        (player(1), 0.0),
        (player(2), 0.0),
    ])));
    app.insert_resource(active_sessions());
    app.insert_resource(room_sessions());
    app
}

fn emit_game_over(app: &mut App) {
    app.world_mut().resource_mut::<RoundState>().phase = RoundPhase::GameOver;
    app.world_mut().write_message(GameOverEmitted {
        loser: Some(player(1)),
        round: 5,
        reason: GameOverReason::ObjectivesDestroyed,
    });
    app.update();
}

#[test]
fn game_over_teardown_removes_session_resources_and_broadcasts_result() {
    let mut app = game_active_app();

    emit_game_over(&mut app);

    assert!(!app.world().contains_resource::<SessionConfig>());
    assert!(!app.world().contains_resource::<ServerRng>());
    assert!(!app.world().contains_resource::<SessionSlots>());
    assert!(!app.world().contains_resource::<ClassSelections>());
    assert!(!app.world().contains_resource::<ClassPreviews>());
    assert!(!app.world().contains_resource::<LobbyDeadline>());
    assert!(!app.world().contains_resource::<LobbyHeartbeats>());
    assert_eq!(*app.world().resource::<LobbyState>(), LobbyState::GameOver);

    let outbox = app
        .world()
        .resource::<server::core::session::SessionNetworkOutbox>();
    assert_eq!(outbox.game_over().len(), 1);
    assert_eq!(outbox.game_over()[0].loser, Some(player(1)));
    assert_eq!(outbox.game_over()[0].round, 5);
    assert_eq!(
        outbox.game_over()[0].reason,
        GameOverReason::ObjectivesDestroyed
    );

    let ended = app.world().resource::<EndedSessionResultState>();
    assert_eq!(ended.result.round, 5);
    assert_eq!(ended.participants, HashSet::from([player(1), player(2)]));
    assert!(ended.acknowledged.is_empty());
    assert_eq!(ended.final_snapshots.len(), 2);
    assert!(ended.final_snapshots.values().all(|snapshot| {
        snapshot.phase == shared::protocol::RoundPhase::GameOver && snapshot.round_number == 5
    }));
}

#[test]
fn game_over_teardown_cleans_active_sessions_and_room_state() {
    let mut app = game_active_app();

    emit_game_over(&mut app);

    let active = app.world().resource::<ActiveSessions>();
    assert!(!active.0.contains_key(&player(1)));
    assert!(!active.0.contains_key(&player(2)));
    assert_eq!(active.0.get(&player(9)), Some(&session_id(9)));

    let rooms = app.world().resource::<RoomSessions>();
    let room = rooms
        .get(session_id(1))
        .expect("session room remains inspectable");
    assert_eq!(room.state, LobbyState::GameOver);
    assert!(room.heartbeats.0.is_empty());
}

#[test]
fn game_over_teardown_retains_reconnect_tracker_until_result_ack_cleanup() {
    let mut app = game_active_app();
    app.insert_resource(ReconnectTracker {
        snapshot_sent: HashMap::from([(player(1), true), (player(2), true), (player(9), true)]),
        deferred_queue: HashMap::from([
            (
                player(1),
                vec![DeferredMessage::CardAcquired {
                    card_id: CardId(10),
                    source: CardSource::ShopPurchase,
                }],
            ),
            (
                player(2),
                vec![DeferredMessage::CardAcquired {
                    card_id: CardId(20),
                    source: CardSource::ShopPurchase,
                }],
            ),
            (
                player(9),
                vec![DeferredMessage::CardAcquired {
                    card_id: CardId(90),
                    source: CardSource::ShopPurchase,
                }],
            ),
        ]),
        token_map: HashMap::from([
            ([1; 16], (session_id(1), player(1))),
            ([2; 16], (session_id(1), player(2))),
            ([9; 16], (session_id(9), player(9))),
        ]),
        sang_meprise_sent_to: HashSet::from([player(1), player(2), player(9)]),
        ..Default::default()
    });

    emit_game_over(&mut app);

    let tracker = app.world().resource::<ReconnectTracker>();
    assert_eq!(tracker.token_map.len(), 3);
    assert!(tracker.token_map.contains_key(&[1; 16]));
    assert!(tracker.token_map.contains_key(&[2; 16]));
    assert!(tracker.token_map.contains_key(&[9; 16]));
    assert!(tracker.deferred_queue.contains_key(&player(1)));
    assert!(tracker.deferred_queue.contains_key(&player(2)));
    assert!(tracker.deferred_queue.contains_key(&player(9)));
    assert_eq!(tracker.snapshot_sent.get(&player(1)), Some(&true));
    assert_eq!(tracker.snapshot_sent.get(&player(2)), Some(&true));
    assert_eq!(tracker.snapshot_sent.get(&player(9)), Some(&true));
    assert!(tracker.sang_meprise_sent_to.contains(&player(1)));
    assert!(tracker.sang_meprise_sent_to.contains(&player(2)));
    assert!(tracker.sang_meprise_sent_to.contains(&player(9)));
    assert!(app.world().contains_resource::<EndedSessionResultState>());
}

#[test]
fn game_over_teardown_is_idempotent_when_already_game_over() {
    let mut app = game_active_app();
    app.insert_resource(LobbyState::GameOver);

    emit_game_over(&mut app);

    assert_eq!(*app.world().resource::<LobbyState>(), LobbyState::GameOver);
    assert!(app
        .world()
        .resource::<server::core::session::SessionNetworkOutbox>()
        .game_over()
        .is_empty());
    assert!(app.world().contains_resource::<SessionConfig>());
    assert!(app.world().contains_resource::<ServerRng>());
}
