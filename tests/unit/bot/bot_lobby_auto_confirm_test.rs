//! End-to-end progression tests for the bot lobby auto-confirm loop
//! (PROMPT 1583 — BOT-LOBBY-READY-AUTO-CONFIRM).
//!
//! Scope: prove that a single-human room paired with one bot can reach
//! `LobbyState::GameActive` without any client-side bot input, by running
//! `bot_lobby_auto_confirm` and `evaluate_room_session_ready` together in the
//! same `App` and simulating the human's class confirm via the canonical
//! `confirm_class` helper.
//!
//! The existing `bot_lobby_loop_test.rs` exercises the auto-confirm system in
//! isolation (class+slot bookkeeping, idempotency, lobby-state gating). This
//! file complements it by exercising the *progression* contract that the
//! orchestrator queued in PROMPT 1583: lobby → draft, no manual bot UI.
//!
//! Determinism: all classes are picked by `deterministic_class_for_bot`, the
//! human's class is fixed, and we override `LobbyDeadline` to a value well
//! into the future so the deadline gate never trips.

use bevy::prelude::*;
use server::core::session::{
    confirm_class, evaluate_room_session_ready, ActiveSessions, ClassSelections, LobbyDeadline,
    LobbyHeartbeats, LobbyState, PlayerSessions, RoomCode, RoomSession, RoomSessions, SessionId,
    SessionSlot, SessionSlots,
};
use server::feature::bot::{
    bot_lobby_auto_confirm, deterministic_class_for_bot, BotDecisionLog, BotPlayers,
};
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;
use std::collections::HashMap;
use uuid::Uuid;

const HUMAN: PlayerId = PlayerId(7);
const BOT: PlayerId = PlayerId((1 << 63) | 0xC0DE_C0DE);
const SESSION_UUID: u128 = 0x0BAD_F00D_BEEF_C0DE_1583_DEAD_BEEF_F00D;
const HUMAN_CLASS: ClassId = ClassId::Iop;

fn build_room() -> RoomSession {
    let slots = SessionSlots(vec![
        SessionSlot {
            index: 0,
            team: 0,
            player: Some(HUMAN),
            class: None,
            is_bot: false,
        },
        SessionSlot {
            index: 1,
            team: 1,
            player: Some(BOT),
            class: None,
            is_bot: true,
        },
    ]);
    RoomSession {
        session_id: SessionId(Uuid::from_u128(SESSION_UUID)),
        room_code: RoomCode("WXYZ".to_string()),
        owner: HUMAN,
        mode: GameMode::OneVOne,
        // LobbyDeadline is "must be <= now"; pick a horizon well beyond any
        // test tick to keep `f4_session_ready` from short-circuiting.
        state: LobbyState::LobbyWaiting,
        slots,
        lobby_deadline: LobbyDeadline(1_000_000.0),
        heartbeats: LobbyHeartbeats(HashMap::from([(HUMAN, 0.0)])),
    }
}

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<RoomSessions>()
        .init_resource::<ClassSelections>()
        .init_resource::<BotPlayers>()
        .init_resource::<BotDecisionLog>()
        .init_resource::<ActiveSessions>()
        .init_resource::<PlayerSessions>();
    app.add_systems(
        Update,
        (
            bot_lobby_auto_confirm,
            evaluate_room_session_ready.after(bot_lobby_auto_confirm),
        ),
    );
    app
}

fn install_room(app: &mut App) -> SessionId {
    let room = build_room();
    let session_id = room.session_id;
    app.world_mut()
        .resource_mut::<ActiveSessions>()
        .0
        .insert(HUMAN, session_id);
    app.world_mut()
        .resource_mut::<ActiveSessions>()
        .0
        .insert(BOT, session_id);
    app.world_mut().resource_mut::<RoomSessions>().insert(room);
    session_id
}

/// Drives the lobby exactly one tick to let the bot auto-confirm and the
/// evaluator observe the (still incomplete) state. The room must remain in
/// `LobbyWaiting` because the human has not confirmed yet.
#[test]
fn test_lobby_remains_waiting_until_human_confirms() {
    let mut app = make_app();
    let session_id = install_room(&mut app);

    app.update();

    let rooms = app.world().resource::<RoomSessions>();
    let stored = rooms.get(session_id).expect("session present");
    assert_eq!(
        stored.state,
        LobbyState::LobbyWaiting,
        "lobby must NOT advance while the human is still un-confirmed"
    );

    let expected_bot_class = deterministic_class_for_bot(session_id, 1);
    assert_eq!(stored.slots.0[1].class, Some(expected_bot_class));
    assert_eq!(stored.slots.0[0].class, None, "human class still pending");

    let selections = app.world().resource::<ClassSelections>();
    assert_eq!(selections.0.get(&BOT).copied(), Some(expected_bot_class));
    assert!(!selections.0.contains_key(&HUMAN));
}

/// End-to-end progression: bot auto-confirms, human confirms via the
/// canonical `confirm_class` helper, `evaluate_room_session_ready` lifts the
/// room to `GameActive`. This is the contract PROMPT 1583 was queued to
/// guarantee (single-human game reaches draft with no manual bot input).
#[test]
fn test_single_human_room_with_bot_progresses_to_game_active() {
    let mut app = make_app();
    let session_id = install_room(&mut app);

    // Tick 1: bot auto-confirm runs; evaluator sees an incomplete lobby and
    // leaves it in LobbyWaiting.
    app.update();
    assert_eq!(
        app.world()
            .resource::<RoomSessions>()
            .get(session_id)
            .expect("session")
            .state,
        LobbyState::LobbyWaiting,
    );

    // Simulate the human confirming a class via the canonical server path.
    // This mirrors what `handle_confirm_class` does for a real client message.
    {
        let world = app.world_mut();
        // SAFETY: we hold mut refs to disjoint resources via resource_scope.
        world.resource_scope(|world, mut rooms: Mut<RoomSessions>| {
            world.resource_scope(|world, mut selections: Mut<ClassSelections>| {
                let active_sessions = world.resource::<ActiveSessions>();
                let outcome = confirm_class(
                    &mut rooms,
                    active_sessions,
                    &mut selections,
                    HUMAN,
                    HUMAN_CLASS,
                );
                // The bot's class was set by the lobby loop, so the human's
                // confirm completes the lock set and must yield a `Locked`
                // outcome with a `revealed` payload.
                assert!(matches!(
                    outcome,
                    server::core::session::ConfirmClassOutcome::Locked { revealed: Some(_), .. }
                ));
            });
        });
    }

    // Tick 2: evaluator now sees all classes confirmed; room flips to
    // GameActive in the same Update.
    app.update();

    let rooms = app.world().resource::<RoomSessions>();
    let stored = rooms.get(session_id).expect("session present");
    assert_eq!(
        stored.state,
        LobbyState::GameActive,
        "human confirm + bot auto-confirm together must lift the lobby"
    );

    // The LobbyState resource is the global current-session signal that the
    // RSM reads to switch into DraftInitial. Confirm it was inserted.
    let global_state = app.world().resource::<LobbyState>();
    assert_eq!(
        *global_state,
        LobbyState::GameActive,
        "global LobbyState resource must mirror the room state"
    );

    // PlayerSessions must contain both the human (with the chosen class) and
    // the bot (with its deterministic class). This is the data the draft
    // phase reads to authorize ready signals.
    let player_sessions = app.world().resource::<PlayerSessions>();
    assert_eq!(player_sessions.class_of(HUMAN), HUMAN_CLASS);
    let expected_bot_class = deterministic_class_for_bot(session_id, 1);
    assert_eq!(player_sessions.class_of(BOT), expected_bot_class);
}

/// The bot auto-confirm path must be safe to re-run after the lobby has been
/// lifted: `evaluate_room_session_ready` switches the room to `GameActive`,
/// and `bot_lobby_auto_confirm` must NOT mutate `ClassSelections` for the
/// already-active session on subsequent ticks (idempotency across the state
/// flip).
#[test]
fn test_auto_confirm_is_inert_after_room_reaches_game_active() {
    let mut app = make_app();
    let session_id = install_room(&mut app);

    // Bring the lobby to GameActive via the human confirm path.
    app.update();
    {
        let world = app.world_mut();
        world.resource_scope(|world, mut rooms: Mut<RoomSessions>| {
            world.resource_scope(|world, mut selections: Mut<ClassSelections>| {
                let active_sessions = world.resource::<ActiveSessions>();
                let _ = confirm_class(
                    &mut rooms,
                    active_sessions,
                    &mut selections,
                    HUMAN,
                    HUMAN_CLASS,
                );
            });
        });
    }
    app.update();
    assert_eq!(
        app.world()
            .resource::<RoomSessions>()
            .get(session_id)
            .expect("session")
            .state,
        LobbyState::GameActive,
    );

    let log_len_after_lift = app.world().resource::<BotDecisionLog>().len();

    // Several more ticks must NOT append further bot decision entries.
    app.update();
    app.update();
    app.update();

    assert_eq!(
        app.world().resource::<BotDecisionLog>().len(),
        log_len_after_lift,
        "auto-confirm must remain idempotent once the room is no longer LobbyWaiting"
    );
}
