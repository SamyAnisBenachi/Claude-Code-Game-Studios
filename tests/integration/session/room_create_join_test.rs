use server::core::session::{
    add_bot_to_room, create_bot_room, create_room, join_room, normalise_room_code,
    remove_bot_from_room, room_code_from_bytes, ActiveSessions, BotSlotActionOutcome,
    CreateBotRoomOutcome, CreateRoomOutcome, JoinRoomOutcome, LobbyDeadline, LobbyState, RoomCode,
    RoomSessions, SessionId, ROOM_CODE_LEN,
};
use shared::protocol::{
    BotActionRejectedReason, CreateRoomRejectedReason, GameMode, JoinRejectedReason, S2CRoomCreated,
};
use shared::session::PlayerId;
use uuid::Uuid;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_id(value: u128) -> SessionId {
    SessionId(Uuid::from_u128(value))
}

fn create_fixed_room(
    rooms: &mut RoomSessions,
    active: &mut ActiveSessions,
    owner: PlayerId,
    id: SessionId,
    code: &str,
    now: f64,
) -> S2CRoomCreated {
    match create_room(
        rooms,
        active,
        owner,
        GameMode::OneVOne,
        now,
        90,
        id,
        RoomCode(code.to_string()),
    ) {
        CreateRoomOutcome::Created(message) => message,
        CreateRoomOutcome::Rejected(rejection) => {
            panic!("room creation should succeed, got {:?}", rejection.reason)
        }
    }
}

#[test]
fn test_create_then_join_room_happy_path_updates_slots_and_active_sessions() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let player_a = player(1);
    let player_b = player(2);
    let id = session_id(1);

    let created = create_fixed_room(&mut rooms, &mut active, player_a, id, "ABCDEF", 10.0);

    assert_eq!(created.session_id, id.0.to_string());
    assert_eq!(created.room_code, "ABCDEF");
    assert_eq!(created.mode, GameMode::OneVOne);
    assert_eq!(created.slots.len(), 2);
    assert_eq!(created.slots[0].slot, 0);
    assert_eq!(created.slots[0].team, 0);
    assert_eq!(created.slots[0].player_id, Some(player_a));
    assert_eq!(created.slots[1].slot, 1);
    assert_eq!(created.slots[1].team, 1);
    assert_eq!(created.slots[1].player_id, None);

    let session = rooms.get(id).expect("created room exists");
    assert_eq!(session.state, LobbyState::LobbyWaiting);
    assert_eq!(session.lobby_deadline, LobbyDeadline(100.0));
    assert_eq!(session.heartbeats.0.get(&player_a), Some(&10.0));
    assert_eq!(active.0.get(&player_a), Some(&id));

    let joined = join_room(&mut rooms, &mut active, player_b, "ABCDEF", 1, 12.0);

    let (ack, slot_update, slot_update_recipients) = match joined {
        JoinRoomOutcome::Joined {
            ack,
            slot_update,
            slot_update_recipients,
        } => (ack, slot_update, slot_update_recipients),
        JoinRoomOutcome::Rejected(rejection) => {
            panic!("join should succeed, got {:?}", rejection.reason)
        }
    };

    assert_eq!(ack.session_id, id.0.to_string());
    assert_eq!(ack.mode, GameMode::OneVOne);
    assert_eq!(ack.slots.len(), 2);
    assert_eq!(ack.slots[1].player_id, Some(player_b));
    assert_eq!(slot_update.slots, ack.slots);
    assert_eq!(slot_update_recipients, vec![player_a]);
    assert!(!slot_update_recipients.contains(&player_b));
    assert_eq!(active.0.get(&player_b), Some(&id));

    let session = rooms.get(id).expect("joined room exists");
    assert_eq!(session.state, LobbyState::LobbyWaiting);
    assert_eq!(session.slots.0[0].player, Some(player_a));
    assert_eq!(session.slots.0[1].player, Some(player_b));
    assert_eq!(session.heartbeats.0.get(&player_b), Some(&12.0));
}

#[test]
fn test_idempotent_create_returns_existing_lobby_waiting_room() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let owner = player(1);
    let first_id = session_id(1);

    let first = create_fixed_room(&mut rooms, &mut active, owner, first_id, "ABCDEF", 1.0);

    let second = create_room(
        &mut rooms,
        &mut active,
        owner,
        GameMode::OneVOne,
        2.0,
        90,
        session_id(2),
        RoomCode("G7TK2M".to_string()),
    );

    let second = match second {
        CreateRoomOutcome::Created(message) => message,
        CreateRoomOutcome::Rejected(rejection) => {
            panic!(
                "idempotent create should succeed, got {:?}",
                rejection.reason
            )
        }
    };

    assert_eq!(first.session_id, second.session_id);
    assert_eq!(second.room_code, "ABCDEF");
    assert_eq!(rooms.len(), 1);
    assert_eq!(active.0.get(&owner), Some(&first_id));
}

#[test]
fn test_create_bot_room_seats_synthetic_bot_in_first_opposing_slot() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let owner = player(1);
    let id = session_id(101);

    let created = create_bot_room(
        &mut rooms,
        &mut active,
        owner,
        GameMode::OneVOne,
        5.0,
        90,
        id,
        RoomCode("BOT001".to_string()),
    );

    let message = match created {
        CreateBotRoomOutcome::Created(message) => message,
        CreateBotRoomOutcome::Rejected(rejection) => {
            panic!("create_bot_room should succeed, got {:?}", rejection.reason)
        }
    };

    assert_eq!(message.session_id, id.0.to_string());
    assert_eq!(message.room_code, "BOT001");
    assert_eq!(message.slots.len(), 2);
    assert_eq!(message.slots[0].player_id, Some(owner));
    assert!(!message.slots[0].is_bot);
    assert!(message.slots[1].is_bot);
    let bot_player = message.slots[1]
        .player_id
        .expect("bot slot must carry a synthetic player id");
    assert!(
        bot_player.0 >= (1_u64 << 63),
        "bot ids reserve the high-bit range so fresh human ids do not collide"
    );

    let session = rooms.get(id).expect("created bot room exists");
    assert_eq!(session.slots.0[1].player, Some(bot_player));
    assert!(session.slots.0[1].is_bot);
    assert_eq!(session.heartbeats.0.get(&owner), Some(&5.0));
    assert!(
        !session.heartbeats.0.contains_key(&bot_player),
        "bot rooms do not fake a network heartbeat for the bot"
    );
    assert_eq!(active.0.get(&owner), Some(&id));
    assert_eq!(active.0.get(&bot_player), Some(&id));
}

#[test]
fn test_create_bot_room_rejects_active_player_with_bot_rejection() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let owner = player(1);

    create_fixed_room(&mut rooms, &mut active, owner, session_id(1), "ABCDEF", 1.0);

    let rejected = create_bot_room(
        &mut rooms,
        &mut active,
        owner,
        GameMode::OneVOne,
        2.0,
        90,
        session_id(2),
        RoomCode("BOT001".to_string()),
    );

    match rejected {
        CreateBotRoomOutcome::Rejected(rejection) => {
            assert_eq!(rejection.reason, BotActionRejectedReason::AlreadyInSession);
        }
        CreateBotRoomOutcome::Created(_) => {
            panic!("active player must not create a bot room")
        }
    }
}

#[test]
fn test_owner_adds_and_removes_bot_slot_then_human_can_join_it() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let owner = player(1);
    let teammate = player(2);
    let joiner = player(3);
    let id = session_id(202);

    match create_room(
        &mut rooms,
        &mut active,
        owner,
        GameMode::TwoVTwo,
        1.0,
        90,
        id,
        RoomCode("BOT2V2".to_string()),
    ) {
        CreateRoomOutcome::Created(_) => {}
        CreateRoomOutcome::Rejected(rejection) => {
            panic!("room creation should succeed, got {:?}", rejection.reason)
        }
    }
    assert!(matches!(
        join_room(&mut rooms, &mut active, teammate, "BOT2V2", 1, 2.0),
        JoinRoomOutcome::Joined { .. }
    ));

    match add_bot_to_room(&mut rooms, &mut active, teammate, 2, 2.5) {
        BotSlotActionOutcome::Rejected(rejection) => {
            assert_eq!(rejection.reason, BotActionRejectedReason::NotOwner);
        }
        BotSlotActionOutcome::Updated { .. } => {
            panic!("non-owner must not add a bot")
        }
    }

    let updated = add_bot_to_room(&mut rooms, &mut active, owner, 2, 3.0);
    let (slot_update, recipients) = match updated {
        BotSlotActionOutcome::Updated {
            slot_update,
            slot_update_recipients,
        } => (slot_update, slot_update_recipients),
        BotSlotActionOutcome::Rejected(rejection) => {
            panic!("owner add bot should succeed, got {:?}", rejection.reason)
        }
    };
    assert_eq!(recipients, vec![owner, teammate]);
    let bot_slot = slot_update
        .slots
        .iter()
        .find(|slot| slot.slot == 2)
        .expect("slot 2 must be present");
    assert!(bot_slot.is_bot);
    let bot_player = bot_slot.player_id.expect("bot slot must be occupied");
    assert_eq!(active.0.get(&bot_player), Some(&id));

    let occupied = join_room(&mut rooms, &mut active, joiner, "BOT2V2", 2, 3.5);
    assert_join_rejection(occupied, JoinRejectedReason::SlotOccupied);

    let removed = remove_bot_from_room(&mut rooms, &mut active, owner, 2);
    let (slot_update, recipients) = match removed {
        BotSlotActionOutcome::Updated {
            slot_update,
            slot_update_recipients,
        } => (slot_update, slot_update_recipients),
        BotSlotActionOutcome::Rejected(rejection) => {
            panic!(
                "owner remove bot should succeed, got {:?}",
                rejection.reason
            )
        }
    };
    assert_eq!(recipients, vec![owner, teammate]);
    let reopened = slot_update
        .slots
        .iter()
        .find(|slot| slot.slot == 2)
        .expect("slot 2 must be present");
    assert_eq!(reopened.player_id, None);
    assert!(!reopened.is_bot);
    assert_eq!(active.0.get(&bot_player), None);

    let joined = join_room(&mut rooms, &mut active, joiner, "BOT2V2", 2, 4.0);
    match joined {
        JoinRoomOutcome::Joined { ack, .. } => {
            let joined_slot = ack
                .slots
                .iter()
                .find(|slot| slot.slot == 2)
                .expect("slot 2 must be present");
            assert_eq!(joined_slot.player_id, Some(joiner));
            assert!(!joined_slot.is_bot);
        }
        JoinRoomOutcome::Rejected(rejection) => {
            panic!(
                "human should join reopened bot slot, got {:?}",
                rejection.reason
            )
        }
    }
}

#[test]
fn test_full_session_rejects_third_joiner() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let id = session_id(1);

    create_fixed_room(&mut rooms, &mut active, player(1), id, "ABCDEF", 1.0);
    assert!(matches!(
        join_room(&mut rooms, &mut active, player(2), "ABCDEF", 1, 2.0),
        JoinRoomOutcome::Joined { .. }
    ));

    let rejected = join_room(&mut rooms, &mut active, player(3), "ABCDEF", 0, 3.0);

    match rejected {
        JoinRoomOutcome::Rejected(rejection) => {
            assert_eq!(rejection.reason, JoinRejectedReason::SessionFull);
        }
        JoinRoomOutcome::Joined { .. } => panic!("third player must not join a full room"),
    }
}

#[test]
fn test_create_rejects_player_already_in_non_waiting_session() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let owner = player(1);
    let id = session_id(1);

    create_fixed_room(&mut rooms, &mut active, owner, id, "ABCDEF", 1.0);
    rooms.get_mut(id).expect("created room exists").state = LobbyState::GameActive;

    let rejected = create_room(
        &mut rooms,
        &mut active,
        owner,
        GameMode::OneVOne,
        2.0,
        90,
        session_id(2),
        RoomCode("G7TK2M".to_string()),
    );

    match rejected {
        CreateRoomOutcome::Rejected(rejection) => {
            assert_eq!(rejection.reason, CreateRoomRejectedReason::AlreadyInSession);
        }
        CreateRoomOutcome::Created(_) => panic!("active player must not create another room"),
    }
}

#[test]
fn test_join_rejection_paths_are_distinct() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let owner = player(1);
    let joiner = player(2);
    let other_owner = player(3);
    let id = session_id(1);
    let other_id = session_id(2);

    let missing = join_room(&mut rooms, &mut active, joiner, "XXXXXX", 0, 1.0);
    assert_join_rejection(missing, JoinRejectedReason::RoomNotFound);

    create_fixed_room(&mut rooms, &mut active, owner, id, "ABCDEF", 2.0);
    let invalid_slot = join_room(&mut rooms, &mut active, joiner, "ABCDEF", 9, 3.0);
    assert_join_rejection(invalid_slot, JoinRejectedReason::InvalidSlot);

    let occupied = join_room(&mut rooms, &mut active, joiner, "ABCDEF", 0, 4.0);
    assert_join_rejection(occupied, JoinRejectedReason::SlotOccupied);

    rooms.get_mut(id).expect("created room exists").state = LobbyState::LobbyReady;
    let not_joinable = join_room(&mut rooms, &mut active, joiner, "ABCDEF", 1, 5.0);
    assert_join_rejection(not_joinable, JoinRejectedReason::SessionNotJoinable);

    create_fixed_room(
        &mut rooms,
        &mut active,
        other_owner,
        other_id,
        "G7TK2M",
        6.0,
    );
    let already_in_session = join_room(&mut rooms, &mut active, other_owner, "ABCDEF", 1, 7.0);
    assert_join_rejection(already_in_session, JoinRejectedReason::AlreadyInSession);
}

#[test]
fn test_join_after_session_start_returns_session_in_progress() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let owner = player(1);
    let joiner = player(2);
    let id = session_id(1);

    create_fixed_room(&mut rooms, &mut active, owner, id, "ABCDEF", 1.0);
    rooms.get_mut(id).expect("created room exists").state = LobbyState::GameActive;

    let rejected = join_room(&mut rooms, &mut active, joiner, "ABCDEF", 1, 2.0);

    assert_join_rejection(rejected, JoinRejectedReason::SessionInProgress);
    assert_eq!(active.0.get(&joiner), None);
    let session = rooms.get(id).expect("created room still exists");
    assert_eq!(session.slots.0[1].player, None);
}

#[test]
fn test_room_code_generation_uses_six_unambiguous_uppercase_alphanumerics() {
    test_helpers::init_test_tracing();
    let bytes = [0, 1, 2, 30, 31, 255, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
    let code = room_code_from_bytes(&bytes);

    assert_eq!(code.0.len(), ROOM_CODE_LEN);
    assert!(code
        .0
        .chars()
        .all(|character| character.is_ascii_uppercase() || character.is_ascii_digit()));
    assert!(!code
        .0
        .chars()
        .any(|character| matches!(character, '0' | 'O' | '1' | 'I' | 'L')));
    assert_eq!(normalise_room_code("g7tk2m").0, "G7TK2M");
}

fn assert_join_rejection(outcome: JoinRoomOutcome, expected: JoinRejectedReason) {
    match outcome {
        JoinRoomOutcome::Rejected(rejection) => assert_eq!(rejection.reason, expected),
        JoinRoomOutcome::Joined { .. } => panic!("join should have been rejected"),
    }
}
