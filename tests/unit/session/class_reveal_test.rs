use server::core::session::{
    confirm_class, create_room, join_room, select_class, ActiveSessions, ClassPreviews,
    ClassSelections, ConfirmClassOutcome, CreateRoomOutcome, GameSessionPlugin, JoinRoomOutcome,
    LobbyState, RoomCode, RoomSessions, SelectClassOutcome, SessionId,
};
use shared::card::ClassId;
use shared::protocol::{ConfirmClassRejectedReason, GameMode};
use shared::session::PlayerId;
use uuid::Uuid;

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
) {
    match create_room(
        rooms,
        active,
        owner,
        GameMode::OneVOne,
        1.0,
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

fn two_player_room() -> (RoomSessions, ActiveSessions, SessionId, PlayerId, PlayerId) {
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let player_a = player(1);
    let player_b = player(2);
    let id = session_id(1);

    create_fixed_room(&mut rooms, &mut active, player_a, id, "ABCDEF");
    match join_room(&mut rooms, &mut active, player_b, "ABCDEF", 1, 2.0) {
        JoinRoomOutcome::Joined { .. } => {}
        JoinRoomOutcome::Rejected(rejection) => {
            panic!("join should succeed, got {:?}", rejection.reason)
        }
    }

    (rooms, active, id, player_a, player_b)
}

#[test]
fn select_class_updates_preview_only_and_never_locks() {
    let (rooms, active, id, player_a, _) = two_player_room();
    let mut previews = ClassPreviews::default();

    assert_eq!(
        select_class(&rooms, &active, &mut previews, player_a, ClassId::Iop),
        SelectClassOutcome::PreviewUpdated
    );
    assert_eq!(
        select_class(&rooms, &active, &mut previews, player_a, ClassId::Cra),
        SelectClassOutcome::PreviewUpdated
    );
    assert_eq!(
        select_class(&rooms, &active, &mut previews, player_a, ClassId::Xelor),
        SelectClassOutcome::PreviewUpdated
    );

    assert_eq!(previews.0.get(&player_a), Some(&ClassId::Xelor));
    let session = rooms.get(id).expect("room exists");
    assert_eq!(session.slots.0[0].class, None);
}

#[test]
fn single_confirm_unicasts_lock_without_reveal() {
    let (mut rooms, active, id, player_a, player_b) = two_player_room();
    let mut selections = ClassSelections::default();

    let outcome = confirm_class(&mut rooms, &active, &mut selections, player_a, ClassId::Iop);

    match outcome {
        ConfirmClassOutcome::Locked {
            locked,
            revealed,
            reveal_recipients,
        } => {
            assert_eq!(locked.class_id, ClassId::Iop);
            assert!(revealed.is_none());
            assert!(reveal_recipients.is_empty());
        }
        other => panic!("confirm should lock, got {other:?}"),
    }

    let session = rooms.get(id).expect("room exists");
    assert_eq!(session.slots.0[0].class, Some(ClassId::Iop));
    assert_eq!(session.slots.0[1].class, None);
    assert_eq!(selections.0.get(&player_a), Some(&ClassId::Iop));
    assert_eq!(selections.0.get(&player_b), None);
}

#[test]
fn final_confirm_broadcasts_one_reveal_to_all_players() {
    let (mut rooms, active, _, player_a, player_b) = two_player_room();
    let mut selections = ClassSelections::default();

    let _ = confirm_class(&mut rooms, &active, &mut selections, player_a, ClassId::Iop);
    let outcome = confirm_class(&mut rooms, &active, &mut selections, player_b, ClassId::Cra);

    match outcome {
        ConfirmClassOutcome::Locked {
            locked,
            revealed: Some(revealed),
            reveal_recipients,
        } => {
            assert_eq!(locked.class_id, ClassId::Cra);
            assert_eq!(reveal_recipients, vec![player_a, player_b]);
            assert_eq!(
                revealed.player_class_map,
                vec![(player_a, ClassId::Iop), (player_b, ClassId::Cra)]
            );
        }
        other => panic!("final confirm should reveal, got {other:?}"),
    }

    assert_eq!(selections.0.len(), 2);
}

#[test]
fn relock_with_different_class_is_rejected_without_mutating_state() {
    let (mut rooms, active, id, player_a, _) = two_player_room();
    let mut selections = ClassSelections::default();

    let _ = confirm_class(&mut rooms, &active, &mut selections, player_a, ClassId::Iop);
    let rejected = confirm_class(
        &mut rooms,
        &active,
        &mut selections,
        player_a,
        ClassId::Sacrier,
    );

    match rejected {
        ConfirmClassOutcome::Rejected(message) => {
            assert_eq!(
                message.reason,
                ConfirmClassRejectedReason::ClassAlreadyConfirmed
            );
        }
        other => panic!("re-lock should reject, got {other:?}"),
    }

    let session = rooms.get(id).expect("room exists");
    assert_eq!(session.slots.0[0].class, Some(ClassId::Iop));
    assert_eq!(selections.0.get(&player_a), Some(&ClassId::Iop));
}

#[test]
fn duplicate_confirm_with_same_class_is_silent_idempotent_noop() {
    let (mut rooms, active, id, player_a, _) = two_player_room();
    let mut selections = ClassSelections::default();

    let _ = confirm_class(&mut rooms, &active, &mut selections, player_a, ClassId::Iop);
    let duplicate = confirm_class(&mut rooms, &active, &mut selections, player_a, ClassId::Iop);

    assert!(matches!(duplicate, ConfirmClassOutcome::Ignored));
    let session = rooms.get(id).expect("room exists");
    assert_eq!(session.slots.0[0].class, Some(ClassId::Iop));
    assert_eq!(selections.0.len(), 1);
}

#[test]
fn confirm_does_not_reveal_before_all_slots_are_filled() {
    let mut rooms = RoomSessions::default();
    let mut active = ActiveSessions::default();
    let player_a = player(1);
    let id = session_id(1);
    let mut selections = ClassSelections::default();

    create_fixed_room(&mut rooms, &mut active, player_a, id, "ABCDEF");
    let outcome = confirm_class(&mut rooms, &active, &mut selections, player_a, ClassId::Iop);

    match outcome {
        ConfirmClassOutcome::Locked {
            revealed,
            reveal_recipients,
            ..
        } => {
            assert!(revealed.is_none());
            assert!(reveal_recipients.is_empty());
        }
        other => panic!("confirm should lock, got {other:?}"),
    }
}

#[test]
fn class_messages_outside_lobby_waiting_are_silently_ignored() {
    let (mut rooms, active, id, player_a, _) = two_player_room();
    let mut previews = ClassPreviews::default();
    let mut selections = ClassSelections::default();
    rooms.get_mut(id).expect("room exists").state = LobbyState::GameActive;

    let preview = select_class(&rooms, &active, &mut previews, player_a, ClassId::Iop);
    let confirm = confirm_class(&mut rooms, &active, &mut selections, player_a, ClassId::Iop);

    assert_eq!(preview, SelectClassOutcome::Ignored);
    assert!(matches!(confirm, ConfirmClassOutcome::Ignored));
    assert!(previews.0.is_empty());
    assert!(selections.0.is_empty());
}

#[test]
fn game_session_plugin_registers_class_resources() {
    let mut app = bevy::prelude::App::new();
    app.add_plugins(bevy::prelude::MinimalPlugins);
    app.add_plugins(GameSessionPlugin);
    app.add_message::<server::core::rsm::PlayerHeartbeat>();

    app.update();

    assert!(app.world().get_resource::<ClassPreviews>().is_some());
    assert!(app.world().get_resource::<ClassSelections>().is_some());
}
