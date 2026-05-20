//! Tests for the bot lobby auto-confirm loop (PROMPT 1514).
//!
//! Scope: prove that a room with a bot occupant can advance past
//! LobbyWaiting once the human owner confirms their class, by exercising
//! `bot_lobby_auto_confirm` directly on a hand-built `RoomSessions` state.
//! We do NOT run the full GameSessionPlugin here — the goal is to isolate the
//! foundation/flow change, not to re-test the lobby pipeline.

use bevy::prelude::*;
use server::core::session::{
    ClassSelections, LobbyDeadline, LobbyHeartbeats, LobbyState, RoomCode, RoomSession,
    RoomSessions, SessionId,
};
use server::feature::bot::{
    bot_lobby_auto_confirm, deterministic_class_for_bot, BotDecisionLog, BotPlayers,
};
use shared::card::ClassId;
use shared::session::PlayerId;
use std::collections::HashMap;
use uuid::Uuid;

const HUMAN: PlayerId = PlayerId(11);
const BOT: PlayerId = PlayerId((1 << 63) | 0xAAAA_AAAA);

fn make_session(state: LobbyState) -> RoomSession {
    let slots = server::core::session::SessionSlots(vec![
        server::core::session::SessionSlot {
            index: 0,
            team: 0,
            player: Some(HUMAN),
            class: None,
            is_bot: false,
        },
        server::core::session::SessionSlot {
            index: 1,
            team: 1,
            player: Some(BOT),
            class: None,
            is_bot: true,
        },
    ]);
    RoomSession {
        session_id: SessionId(Uuid::from_u128(0x1234_5678_90AB_CDEF_DEAD_BEEF_CAFE_F00D)),
        room_code: RoomCode("ABCD".to_string()),
        owner: HUMAN,
        mode: shared::protocol::GameMode::OneVOne,
        state,
        slots,
        lobby_deadline: LobbyDeadline(60.0),
        heartbeats: LobbyHeartbeats(HashMap::from([(HUMAN, 0.0)])),
    }
}

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<RoomSessions>();
    app.init_resource::<ClassSelections>();
    app.init_resource::<BotPlayers>();
    app.init_resource::<BotDecisionLog>();
    app.add_systems(Update, bot_lobby_auto_confirm);
    app
}

#[test]
fn test_deterministic_class_is_stable_across_calls() {
    let session = SessionId(Uuid::from_u128(0xA5A5_A5A5_5A5A_5A5A_1234_5678_9ABC_DEF0));
    let first = deterministic_class_for_bot(session, 1);
    let second = deterministic_class_for_bot(session, 1);
    assert_eq!(first, second, "same inputs must yield same class");
    assert_ne!(first, ClassId::Neutral, "bot must never pick Neutral");
}

#[test]
fn test_auto_confirm_sets_slot_class_and_selections_for_bot() {
    let mut app = make_app();
    let session = make_session(LobbyState::LobbyWaiting);
    let session_id = session.session_id;
    app.world_mut()
        .resource_mut::<RoomSessions>()
        .insert(session);

    app.update();

    let rooms = app.world().resource::<RoomSessions>();
    let stored = rooms.get(session_id).expect("session present");
    let bot_slot = &stored.slots.0[1];
    assert!(bot_slot.is_bot);
    let expected_class = deterministic_class_for_bot(session_id, 1);
    assert_eq!(bot_slot.class, Some(expected_class));

    let selections = app.world().resource::<ClassSelections>();
    assert_eq!(selections.0.get(&BOT).copied(), Some(expected_class));

    let bots = app.world().resource::<BotPlayers>();
    assert!(bots.contains(BOT), "bot state inserted on first confirm");
    let state = bots.get(BOT).expect("bot state");
    assert_eq!(state.class_choice, Some(expected_class));

    let log = app.world().resource::<BotDecisionLog>();
    assert_eq!(log.len(), 1);
    let entry = log.last().expect("entry");
    assert_eq!(entry.bot_player_id, BOT);
}

#[test]
fn test_auto_confirm_skips_human_slots() {
    let mut app = make_app();
    let session = make_session(LobbyState::LobbyWaiting);
    let session_id = session.session_id;
    app.world_mut()
        .resource_mut::<RoomSessions>()
        .insert(session);

    app.update();

    let rooms = app.world().resource::<RoomSessions>();
    let stored = rooms.get(session_id).expect("session present");
    let human_slot = &stored.slots.0[0];
    assert_eq!(human_slot.class, None, "human slot remains untouched");

    let selections = app.world().resource::<ClassSelections>();
    assert!(
        !selections.0.contains_key(&HUMAN),
        "human is not auto-confirmed by the bot loop"
    );
}

#[test]
fn test_auto_confirm_is_idempotent_across_ticks() {
    let mut app = make_app();
    let session = make_session(LobbyState::LobbyWaiting);
    app.world_mut()
        .resource_mut::<RoomSessions>()
        .insert(session);

    app.update();
    app.update();
    app.update();

    let log = app.world().resource::<BotDecisionLog>();
    assert_eq!(
        log.len(),
        1,
        "auto-confirm must record exactly one entry per bot per lobby"
    );
}

#[test]
fn test_auto_confirm_skips_when_lobby_not_waiting() {
    let mut app = make_app();
    let session = make_session(LobbyState::GameActive);
    let session_id = session.session_id;
    app.world_mut()
        .resource_mut::<RoomSessions>()
        .insert(session);

    app.update();

    let rooms = app.world().resource::<RoomSessions>();
    let stored = rooms.get(session_id).expect("session present");
    assert_eq!(
        stored.slots.0[1].class, None,
        "bot loop must not mutate slots outside LobbyWaiting"
    );
    assert!(app.world().resource::<ClassSelections>().0.is_empty());
}

#[test]
fn test_completing_human_confirm_after_bot_satisfies_all_classes_confirmed() {
    let mut app = make_app();
    let session = make_session(LobbyState::LobbyWaiting);
    let session_id = session.session_id;
    app.world_mut()
        .resource_mut::<RoomSessions>()
        .insert(session);

    app.update();

    // Simulate the human confirming a class via the existing path.
    {
        let mut rooms = app.world_mut().resource_mut::<RoomSessions>();
        let session = rooms.get_mut(session_id).expect("session");
        session.slots.0[0].class = Some(ClassId::Iop);
    }
    app.world_mut()
        .resource_mut::<ClassSelections>()
        .0
        .insert(HUMAN, ClassId::Iop);

    let rooms = app.world().resource::<RoomSessions>();
    let stored = rooms.get(session_id).expect("session");
    let selections = app.world().resource::<ClassSelections>();
    assert!(server::core::session::all_classes_confirmed(
        &stored.slots,
        selections,
    ));
    assert!(server::core::session::all_slots_filled(&stored.slots));
}
