use std::collections::HashMap;

use server::core::session::{
    apply_placement_timer_multiplier_request_batch, build_session_config_with_settings, create_room,
    effective_session_settings_update, join_room, ActiveSessions, JoinRoomOutcome, LobbyState,
    PlacementTimerMultiplierRequests, RoomCode, RoomSessions, SessionConfig, SessionId, SessionSlot,
    SessionSlots,
};
use shared::card::ClassId;
use shared::protocol::{GameMode, PlacementTimerMultiplier};
use shared::session::PlayerId;
use uuid::Uuid;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn ready_slots() -> SessionSlots {
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

fn ready_selections() -> server::core::session::ClassSelections {
    server::core::session::ClassSelections(HashMap::from([
        (player(1), ClassId::Iop),
        (player(2), ClassId::Cra),
    ]))
}

fn frozen_session_config(multiplier: PlacementTimerMultiplier) -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(player(1), 0), (player(2), 1)]),
        class_map: HashMap::from([(player(1), ClassId::Iop), (player(2), ClassId::Cra)]),
        placement_timer_multiplier_effective: multiplier,
    }
}

#[test]
fn session_multiplier_defaults_to_x1_when_players_make_no_requests() {
    let slots = ready_slots();
    let config = build_session_config_with_settings(&slots, &ready_selections(), None);

    assert_eq!(
        config.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X1
    );
}

#[test]
fn session_multiplier_highest_request_wins_and_update_has_no_identity() {
    let slots = ready_slots();
    let mut requests = PlacementTimerMultiplierRequests::default();

    let update = apply_placement_timer_multiplier_request_batch(
        Some(&LobbyState::LobbyWaiting),
        None,
        Some(&slots),
        &mut requests,
        [
            (player(1), Some(PlacementTimerMultiplier::X1_5)),
            (player(2), Some(PlacementTimerMultiplier::X3)),
        ],
    )
    .expect("highest request should change the effective multiplier");

    assert_eq!(
        update.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X3
    );
    assert_eq!(
        requests.0.get(&player(1)),
        Some(&PlacementTimerMultiplier::X1_5)
    );
    assert_eq!(
        requests.0.get(&player(2)),
        Some(&PlacementTimerMultiplier::X3)
    );

    let value = serde_json::to_value(update).expect("settings update should serialize");
    assert!(value.get("placement_timer_multiplier_effective").is_some());
    assert!(value.get("requester").is_none());
    assert!(value.get("requester_id").is_none());
    assert!(value.get("player_id").is_none());
    assert!(value.get("connection_id").is_none());
}

#[test]
fn session_multiplier_invalid_or_unsupported_request_does_not_shorten_default() {
    let slots = ready_slots();
    let mut requests = PlacementTimerMultiplierRequests::default();
    let invalid = PlacementTimerMultiplier::from_standard_ratio(1, 2);

    let update = apply_placement_timer_multiplier_request_batch(
        Some(&LobbyState::LobbyWaiting),
        None,
        Some(&slots),
        &mut requests,
        [(player(1), invalid)],
    );

    assert!(update.is_none());
    assert!(requests.0.is_empty());
    let config = build_session_config_with_settings(&slots, &ready_selections(), Some(&requests));
    assert_eq!(
        config.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X1
    );
}

#[test]
fn session_multiplier_freezes_into_session_config_before_session_ready() {
    let slots = ready_slots();
    let mut requests = PlacementTimerMultiplierRequests::default();
    requests.0.insert(player(1), PlacementTimerMultiplier::X1_5);
    requests.0.insert(player(2), PlacementTimerMultiplier::X2);

    let config = build_session_config_with_settings(&slots, &ready_selections(), Some(&requests));

    assert_eq!(
        config.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X2
    );
}

#[test]
fn session_multiplier_ignores_post_session_ready_changes() {
    let slots = ready_slots();
    let frozen_config = frozen_session_config(PlacementTimerMultiplier::X2);
    let mut requests = PlacementTimerMultiplierRequests::default();

    let update = apply_placement_timer_multiplier_request_batch(
        Some(&LobbyState::GameActive),
        Some(&frozen_config),
        Some(&slots),
        &mut requests,
        [(player(1), Some(PlacementTimerMultiplier::X3))],
    );

    assert!(update.is_none());
    assert!(requests.0.is_empty());
    assert_eq!(
        frozen_config.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X2
    );
}

// PROMPT 1212 F-03 -- regression test: a joining client must receive the
// room's current effective placement timer multiplier on successful join, not
// the default X1. Before the fix, S2CSessionSettingsUpdated was only broadcast
// when the multiplier actually changed, so a player joining after the owner
// had already set a non-default value never received any settings message and
// their SessionSettingsView remained at X1 until the next change.

fn empty_slots_for_owner(owner: PlayerId) -> SessionSlots {
    SessionSlots(vec![
        SessionSlot {
            index: 0,
            team: 0,
            player: Some(owner),
            class: None,
        },
        SessionSlot {
            index: 1,
            team: 1,
            player: None,
            class: None,
        },
    ])
}

#[test]
fn effective_session_settings_update_returns_room_effective_multiplier_for_joiner() {
    let p1 = player(1);
    let p2 = player(2);

    // Room mid-lobby: P1 sits in slot 0, P2 has just landed in slot 1.
    let slots = SessionSlots(vec![
        SessionSlot {
            index: 0,
            team: 0,
            player: Some(p1),
            class: None,
        },
        SessionSlot {
            index: 1,
            team: 1,
            player: Some(p2),
            class: None,
        },
    ]);

    // P1 set X3 before P2 joined.
    let requests = PlacementTimerMultiplierRequests(HashMap::from([(
        p1,
        PlacementTimerMultiplier::X3,
    )]));

    let update = effective_session_settings_update(&slots, Some(&requests));
    assert_eq!(
        update.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X3,
        "joiner must see the room's effective multiplier, not the default X1"
    );
}

#[test]
fn effective_session_settings_update_defaults_to_x1_for_room_with_no_requests() {
    let p1 = player(1);
    let p2 = player(2);
    let slots = SessionSlots(vec![
        SessionSlot {
            index: 0,
            team: 0,
            player: Some(p1),
            class: None,
        },
        SessionSlot {
            index: 1,
            team: 1,
            player: Some(p2),
            class: None,
        },
    ]);

    let requests = PlacementTimerMultiplierRequests::default();

    let update = effective_session_settings_update(&slots, Some(&requests));
    assert_eq!(
        update.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X1,
        "joiner should see default X1 when no requests are recorded"
    );
}

#[test]
fn join_then_settings_snapshot_reflects_owner_non_default_multiplier() {
    // End-to-end at the function level:
    // 1. P1 creates a room (becomes owner in slot 0).
    // 2. P1 sets a non-default placement timer multiplier (X3).
    // 3. P2 joins slot 1 via `join_room`.
    // 4. The settings snapshot computed for the joiner from the room's slots
    //    matches the effective multiplier P1 had set.
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let p1 = player(1);
    let p2 = player(2);
    let room_code = "ABCDEF";
    let session_uuid = SessionId(Uuid::from_u128(0x1212));

    let created = create_room(
        &mut rooms,
        &mut active,
        p1,
        GameMode::OneVOne,
        0.0,
        120,
        session_uuid,
        RoomCode(room_code.to_string()),
    );
    let server::core::session::CreateRoomOutcome::Created(_) = created else {
        panic!("create_room should succeed for owner with no active session");
    };

    let mut requests = PlacementTimerMultiplierRequests::default();
    let pre_join_update = apply_placement_timer_multiplier_request_batch(
        Some(&LobbyState::LobbyWaiting),
        None,
        Some(&empty_slots_for_owner(p1)),
        &mut requests,
        [(p1, Some(PlacementTimerMultiplier::X3))],
    )
    .expect("owner's non-default request must produce an update");
    assert_eq!(
        pre_join_update.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X3
    );

    let outcome = join_room(&mut rooms, &mut active, p2, room_code, 1, 1.0);
    let JoinRoomOutcome::Joined { .. } = outcome else {
        panic!("P2 should join the owner's room successfully");
    };

    let session = rooms
        .get_by_code(&RoomCode(room_code.to_string()))
        .expect("joined room still exists");
    assert_eq!(session.slots.0[0].player, Some(p1));
    assert_eq!(session.slots.0[1].player, Some(p2));

    let update_for_joiner = effective_session_settings_update(&session.slots, Some(&requests));
    assert_eq!(
        update_for_joiner.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X3,
        "joining client must receive the owner's already-set X3 multiplier on join"
    );
}

#[test]
fn session_multiplier_x1_request_clears_existing_player_request() {
    let slots = ready_slots();
    let mut requests = PlacementTimerMultiplierRequests(HashMap::from([(
        player(1),
        PlacementTimerMultiplier::X2,
    )]));

    let update = apply_placement_timer_multiplier_request_batch(
        Some(&LobbyState::LobbyWaiting),
        None,
        Some(&slots),
        &mut requests,
        [(player(1), Some(PlacementTimerMultiplier::X1))],
    )
    .expect("clearing the only request should publish the default");

    assert_eq!(
        update.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X1
    );
    assert!(requests.0.is_empty());
}
