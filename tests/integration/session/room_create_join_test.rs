use server::core::session::{
    create_room, join_room, normalise_room_code, room_code_from_bytes, ActiveSessions,
    CreateRoomOutcome, JoinRoomOutcome, LobbyDeadline, LobbyState, RoomCode, RoomSessions,
    SessionId, ROOM_CODE_LEN,
};
use shared::protocol::{CreateRoomRejectedReason, GameMode, JoinRejectedReason, S2CRoomCreated};
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
