use bevy::prelude::*;
use lightyear::prelude::PeerId;
use server::core::session::{
    confirm_class, create_room, join_room, process_reconnect_hello, select_class, ActiveSessions,
    ClassPreviews, ClassSelections, ConfirmClassOutcome, CreateRoomOutcome, JoinRoomOutcome,
    NextFreshPlayerId, PlayerConnectionMap, ReconnectDispatch, ReconnectTracker, RoomCode,
    RoomSessions, SelectClassOutcome, SessionId,
};
use shared::card::ClassId;
use shared::protocol::{C2SHello, GameMode};
use shared::session::PlayerId;
use uuid::Uuid;

#[test]
fn fresh_hello_maps_peer_to_stable_player_and_returns_handshake() {
    let mut world = World::new();
    world.insert_resource(PlayerConnectionMap::default());
    world.insert_resource(NextFreshPlayerId::default());
    world.insert_resource(ReconnectTracker::default());

    let entity = world.spawn_empty().id();
    let peer = PeerId::Netcode(101);
    let result = process_reconnect_hello(
        &mut world,
        entity,
        peer,
        C2SHello {
            protocol_version: 1,
            session_token: None,
        },
    );

    assert!(result.closes.is_empty());
    assert_eq!(
        world.resource::<PlayerConnectionMap>().0.get(&peer),
        Some(&PlayerId(1))
    );

    let ReconnectDispatch::Handshake { peer_id, message } = &result.dispatches[0] else {
        panic!("fresh hello should return S2CHandshake");
    };
    assert_eq!(*peer_id, peer);
    assert_eq!(message.player_id, PlayerId(1));
    assert_eq!(message.session_token.len(), 16);
    assert!(world
        .resource::<ReconnectTracker>()
        .token_map
        .contains_key(&message.session_token));
}

#[test]
fn repeated_fresh_hello_reuses_existing_peer_mapping() {
    let mut world = World::new();
    world.insert_resource(PlayerConnectionMap::default());
    world.insert_resource(NextFreshPlayerId::default());
    world.insert_resource(ReconnectTracker::default());

    let entity = world.spawn_empty().id();
    let peer = PeerId::Netcode(202);
    for _ in 0..2 {
        let _ = process_reconnect_hello(
            &mut world,
            entity,
            peer,
            C2SHello {
                protocol_version: 1,
                session_token: None,
            },
        );
    }

    assert_eq!(
        world.resource::<PlayerConnectionMap>().0.get(&peer),
        Some(&PlayerId(1))
    );
    assert_eq!(world.resource::<PlayerConnectionMap>().0.len(), 1);
}

#[test]
fn mapped_fresh_players_can_create_join_select_and_confirm_lobby() {
    let mut world = World::new();
    world.insert_resource(PlayerConnectionMap::default());
    world.insert_resource(NextFreshPlayerId::default());
    world.insert_resource(ReconnectTracker::default());

    let owner_peer = PeerId::Netcode(301);
    let join_peer = PeerId::Netcode(302);
    let owner = fresh_player(&mut world, owner_peer);
    let joiner = fresh_player(&mut world, join_peer);

    let mut rooms = RoomSessions::default();
    let mut active_sessions = ActiveSessions::default();
    let mut previews = ClassPreviews::default();
    let mut selections = ClassSelections::default();
    let session_id = SessionId(Uuid::from_u128(0xA11CE));
    let room_code = RoomCode("AB12CD".to_string());

    let CreateRoomOutcome::Created(created) = create_room(
        &mut rooms,
        &mut active_sessions,
        owner,
        GameMode::OneVOne,
        0.0,
        120,
        session_id,
        room_code.clone(),
    ) else {
        panic!("mapped owner should create room");
    };
    assert_eq!(created.slots[0].player_id, Some(owner));

    let JoinRoomOutcome::Joined {
        ack, slot_update, ..
    } = join_room(
        &mut rooms,
        &mut active_sessions,
        joiner,
        &room_code.0,
        1,
        1.0,
    )
    else {
        panic!("mapped joiner should join room");
    };
    assert_eq!(ack.slots[1].player_id, Some(joiner));
    assert_eq!(slot_update.slots[1].player_id, Some(joiner));

    assert_eq!(
        select_class(&rooms, &active_sessions, &mut previews, owner, ClassId::Iop),
        SelectClassOutcome::PreviewUpdated
    );
    assert!(matches!(
        confirm_class(
            &mut rooms,
            &active_sessions,
            &mut selections,
            owner,
            ClassId::Iop
        ),
        ConfirmClassOutcome::Locked { .. }
    ));
    assert!(matches!(
        confirm_class(
            &mut rooms,
            &active_sessions,
            &mut selections,
            joiner,
            ClassId::Cra
        ),
        ConfirmClassOutcome::Locked { .. }
    ));
}

fn fresh_player(world: &mut World, peer: PeerId) -> PlayerId {
    let entity = world.spawn_empty().id();
    let result = process_reconnect_hello(
        world,
        entity,
        peer,
        C2SHello {
            protocol_version: 1,
            session_token: None,
        },
    );

    let ReconnectDispatch::Handshake { message, .. } = &result.dispatches[0] else {
        panic!("fresh hello should return handshake");
    };
    message.player_id
}
