use std::collections::HashMap;

use bevy::prelude::*;
use server::core::rsm::{
    advance_phase, AuctionPhaseEntered, BroadcastPhaseChanged, DraftStarted, GameOverEmitted,
    LobbyComplete, PhaseAdvanceRequest, PlacementPhaseEntered, ResolutionPhaseEntered, RoundPhase,
    RoundState, ShopRefreshNeeded,
};
use server::core::session::{build_snapshot, PlayerSessionData, PlayerSessions, SessionConfig};
use server::foundation::config::GameConfig;
use server::lobby::handler::apply_class_choice;
use shared::card::ClassId;
use shared::protocol::{C2SClassChoice, DraftPhase, GameMode};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_config(players: &[PlayerId]) -> SessionConfig {
    let mut team_map = HashMap::new();
    let mut class_map = HashMap::new();

    for (index, player) in players.iter().copied().enumerate() {
        team_map.insert(player, index as u8);
        class_map.insert(player, ClassId::Iop);
    }

    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: players.len() as u8,
        team_map,
        class_map,
    }
}

fn sessions(entries: &[(PlayerId, ClassId, bool)]) -> PlayerSessions {
    let mut sessions = PlayerSessions::default();
    for (player_id, class, class_locked) in entries {
        sessions.players.insert(
            *player_id,
            PlayerSessionData {
                class: *class,
                class_locked: *class_locked,
            },
        );
    }
    sessions
}

fn app_with_sessions(sessions: PlayerSessions) -> App {
    let players = [player(1), player(2)];
    let mut app = App::new();
    app.add_message::<LobbyComplete>()
        .add_message::<DraftStarted>()
        .add_message::<ShopRefreshNeeded>()
        .add_message::<AuctionPhaseEntered>()
        .add_message::<PlacementPhaseEntered>()
        .add_message::<ResolutionPhaseEntered>()
        .add_message::<GameOverEmitted>()
        .add_message::<BroadcastPhaseChanged>()
        .insert_resource(RoundState {
            phase: RoundPhase::Lobby,
            round_number: 1,
            ..RoundState::new()
        })
        .insert_resource(PhaseAdvanceRequest::new(RoundPhase::Lobby))
        .insert_resource(session_config(&players))
        .insert_resource(sessions)
        .insert_resource(GameConfig(shared::config::GameConfig::default()))
        .add_systems(Update, advance_phase);
    app
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

#[test]
fn test_class_choice_unlocked_accepts_valid_class() {
    let player_a = player(1);
    let mut sessions = sessions(&[(player_a, ClassId::Neutral, false)]);

    apply_class_choice(
        &mut sessions,
        player_a,
        C2SClassChoice {
            class: ClassId::Xelor,
        },
    );

    let player = sessions.players.get(&player_a).expect("player exists");
    assert_eq!(player.class, ClassId::Xelor);
    assert!(!player.class_locked);
}

#[test]
fn test_class_choice_rejects_neutral_class() {
    let player_a = player(1);
    let mut sessions = sessions(&[(player_a, ClassId::Xelor, false)]);

    apply_class_choice(
        &mut sessions,
        player_a,
        C2SClassChoice {
            class: ClassId::Neutral,
        },
    );

    assert_eq!(sessions.class_of(player_a), ClassId::Xelor);
    assert!(!sessions.is_locked(player_a));
}

#[test]
fn test_class_choice_locked_rejects_change() {
    let player_a = player(1);
    let mut sessions = sessions(&[(player_a, ClassId::Xelor, true)]);

    apply_class_choice(
        &mut sessions,
        player_a,
        C2SClassChoice {
            class: ClassId::Iop,
        },
    );

    assert_eq!(sessions.class_of(player_a), ClassId::Xelor);
    assert!(sessions.is_locked(player_a));
}

#[test]
fn test_lobby_gate_rejects_transition_when_any_class_is_neutral() {
    let mut app = app_with_sessions(sessions(&[
        (player(1), ClassId::Xelor, false),
        (player(2), ClassId::Neutral, false),
    ]));

    app.update();

    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::Lobby
    );
    assert!(read_messages::<LobbyComplete>(&app).is_empty());
    assert!(read_messages::<DraftStarted>(&app).is_empty());
}

#[test]
fn test_lobby_gate_passes_and_locks_all_classes() {
    let mut app = app_with_sessions(sessions(&[
        (player(1), ClassId::Xelor, false),
        (player(2), ClassId::Sacrier, false),
    ]));

    app.update();

    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::DraftInitial
    );
    assert_eq!(read_messages::<LobbyComplete>(&app).len(), 1);
    let sessions = app.world().resource::<PlayerSessions>();
    assert!(sessions.is_locked(player(1)));
    assert!(sessions.is_locked(player(2)));
    assert!(sessions.all_classes_chosen());
}

#[test]
fn test_lobby_gate_lock_prevents_subsequent_class_change() {
    let mut sessions = sessions(&[
        (player(1), ClassId::Xelor, false),
        (player(2), ClassId::Sacrier, false),
    ]);
    assert!(sessions.all_classes_chosen());

    sessions.lock_all_classes();
    apply_class_choice(
        &mut sessions,
        player(1),
        C2SClassChoice {
            class: ClassId::Iop,
        },
    );

    assert_eq!(sessions.class_of(player(1)), ClassId::Xelor);
    assert!(sessions.is_locked(player(1)));
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "lock_all_classes: player has Neutral class")]
fn test_lock_all_classes_panics_when_gate_invariant_is_violated() {
    let mut sessions = sessions(&[(player(1), ClassId::Neutral, false)]);
    sessions.lock_all_classes();
}

#[test]
fn test_snapshot_contains_locked_class_id() {
    let player_a = player(1);
    let mut world = World::new();
    world.insert_resource(sessions(&[
        (player_a, ClassId::Sacrier, true),
        (player(2), ClassId::Xelor, true),
    ]));

    let snapshot = build_snapshot(player_a, &world).expect("snapshot builds");
    let player_snapshot = snapshot
        .players
        .iter()
        .find(|snapshot| snapshot.player_id == player_a)
        .expect("player snapshot exists");

    assert_eq!(player_snapshot.class_id, ClassId::Sacrier);
    assert_ne!(player_snapshot.class_id, ClassId::Neutral);
}

#[test]
fn test_lobby_gate_emits_draft_initial_payloads_after_locking() {
    let mut app = app_with_sessions(sessions(&[
        (player(1), ClassId::Xelor, false),
        (player(2), ClassId::Sacrier, false),
    ]));

    app.update();

    let drafts = read_messages::<DraftStarted>(&app);
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].phase, DraftPhase::Initial);
    assert_eq!(broadcasts.len(), 1);
    assert_eq!(broadcasts[0].phase, RoundPhase::DraftInitial);
}
