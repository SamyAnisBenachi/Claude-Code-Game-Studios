use server::core::session::{
    add_bot_to_room, build_room_list, create_room, join_room, ActiveSessions, BotSlotActionOutcome,
    CreateRoomOutcome, JoinRoomOutcome, LobbyState, RoomCode, RoomSessions, SessionId,
};
use shared::protocol::{GameMode, RoomListEntry};
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
    mode: GameMode,
    now: f64,
) {
    match create_room(
        rooms,
        active,
        owner,
        mode,
        now,
        90,
        id,
        RoomCode(code.to_string()),
    ) {
        CreateRoomOutcome::Created(_) => {}
        CreateRoomOutcome::Rejected(rejection) => {
            panic!("room creation should succeed, got {:?}", rejection.reason)
        }
    }
}

#[test]
fn test_build_room_list_excludes_non_waiting_states() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let owner = player(1);
    let id = session_id(1);

    create_fixed_room(
        &mut rooms,
        &mut active,
        owner,
        id,
        "AAAAAA",
        GameMode::OneVOne,
        1.0,
    );

    let waiting = build_room_list(&rooms, None);
    assert_eq!(waiting.rooms.len(), 1, "LobbyWaiting room must be listed");

    for state in [
        LobbyState::GameActive,
        LobbyState::LobbyReady,
        LobbyState::LobbyCancelled,
        LobbyState::GameOver,
    ] {
        rooms.get_mut(id).expect("room exists").state = state;
        let result = build_room_list(&rooms, None);
        assert!(
            result.rooms.is_empty(),
            "state {:?} must be filtered out of room list",
            state
        );
    }
}

#[test]
fn test_build_room_list_excludes_full_rooms() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let owner = player(1);
    let joiner = player(2);
    let id = session_id(1);

    create_fixed_room(
        &mut rooms,
        &mut active,
        owner,
        id,
        "AAAAAA",
        GameMode::OneVOne,
        1.0,
    );
    assert!(matches!(
        join_room(&mut rooms, &mut active, joiner, "AAAAAA", 1, 2.0),
        JoinRoomOutcome::Joined { .. }
    ));

    let result = build_room_list(&rooms, None);
    assert!(
        result.rooms.is_empty(),
        "fully occupied OneVOne must not be listed (no open slot)"
    );
}

#[test]
fn test_build_room_list_returns_entries_sorted_by_code() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();

    create_fixed_room(
        &mut rooms,
        &mut active,
        player(10),
        session_id(10),
        "BBBBBB",
        GameMode::OneVOne,
        1.0,
    );
    create_fixed_room(
        &mut rooms,
        &mut active,
        player(20),
        session_id(20),
        "AAAAAA",
        GameMode::OneVOne,
        1.0,
    );
    create_fixed_room(
        &mut rooms,
        &mut active,
        player(30),
        session_id(30),
        "CCCCCC",
        GameMode::TwoVTwo,
        1.0,
    );

    let result = build_room_list(&rooms, None);
    let codes: Vec<&str> = result.rooms.iter().map(|e| e.room_code.as_str()).collect();
    assert_eq!(
        codes,
        vec!["AAAAAA", "BBBBBB", "CCCCCC"],
        "room list must be sorted by room_code ascending"
    );
}

#[test]
fn test_build_room_list_reports_correct_filled_and_first_open_slot() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let owner = player(1);
    let joiner = player(2);
    let id = session_id(1);

    create_fixed_room(
        &mut rooms,
        &mut active,
        owner,
        id,
        "ABCDEF",
        GameMode::TwoVTwo,
        1.0,
    );
    assert!(matches!(
        join_room(&mut rooms, &mut active, joiner, "ABCDEF", 1, 2.0),
        JoinRoomOutcome::Joined { .. }
    ));

    let result = build_room_list(&rooms, None);
    assert_eq!(result.rooms.len(), 1, "2v2 with 2 of 4 filled is joinable");
    let entry: &RoomListEntry = &result.rooms[0];
    assert_eq!(entry.room_code, "ABCDEF");
    assert_eq!(entry.mode, GameMode::TwoVTwo);
    assert_eq!(entry.slots_filled, 2);
    assert_eq!(entry.slots_max, 4);
    assert_eq!(
        entry.first_open_slot,
        Some(2),
        "first open slot must be the next-index empty slot (after creator slot 0 and joiner slot 1)"
    );
}

#[test]
fn test_build_room_list_reports_bot_slots_as_filled_metadata() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let owner = player(1);
    let id = session_id(1);

    create_fixed_room(
        &mut rooms,
        &mut active,
        owner,
        id,
        "ABCDEF",
        GameMode::TwoVTwo,
        1.0,
    );
    match add_bot_to_room(&mut rooms, &mut active, owner, 2, 2.0) {
        BotSlotActionOutcome::Updated { .. } => {}
        BotSlotActionOutcome::Rejected(rejection) => {
            panic!("owner add bot should succeed, got {:?}", rejection.reason)
        }
    }

    let result = build_room_list(&rooms, None);
    assert_eq!(
        result.rooms.len(),
        1,
        "2v2 with one bot still has open slots"
    );
    let entry: &RoomListEntry = &result.rooms[0];
    assert_eq!(entry.room_code, "ABCDEF");
    assert_eq!(entry.slots_filled, 2, "bot slots count as occupied");
    assert_eq!(entry.bot_count, 1);
    assert_eq!(entry.first_open_slot, Some(1));
    assert!(
        entry.has_human_opponent,
        "open non-owner slots remain human opportunities in the browser"
    );
}

#[test]
fn test_build_room_list_excludes_requesters_own_session() {
    test_helpers::init_test_tracing();
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let owner = player(1);
    let other_owner = player(2);
    let own_id = session_id(1);
    let other_id = session_id(2);

    create_fixed_room(
        &mut rooms,
        &mut active,
        owner,
        own_id,
        "AAAAAA",
        GameMode::OneVOne,
        1.0,
    );
    create_fixed_room(
        &mut rooms,
        &mut active,
        other_owner,
        other_id,
        "BBBBBB",
        GameMode::OneVOne,
        1.0,
    );

    let everything = build_room_list(&rooms, None);
    assert_eq!(everything.rooms.len(), 2);

    let excluded = build_room_list(&rooms, Some(own_id));
    let codes: Vec<&str> = excluded
        .rooms
        .iter()
        .map(|e| e.room_code.as_str())
        .collect();
    assert_eq!(
        codes,
        vec!["BBBBBB"],
        "requester's own session must be filtered out when exclude_session is set"
    );
}

#[test]
fn test_build_room_list_on_empty_registry_returns_empty_vec() {
    test_helpers::init_test_tracing();
    let rooms = RoomSessions::default();

    let result = build_room_list(&rooms, None);
    assert!(
        result.rooms.is_empty(),
        "empty RoomSessions must produce an empty S2CRoomList"
    );

    let result_with_exclude = build_room_list(&rooms, Some(session_id(1)));
    assert!(
        result_with_exclude.rooms.is_empty(),
        "exclude_session against an empty registry must not panic and must produce empty list"
    );
}
