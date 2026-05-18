use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::network::{heartbeat_due_after_tick, ClientHeartbeatTimer};
use client::state::{
    apply_handshake_message, should_enter_session_from_phase, should_enter_session_from_snapshot,
    ClientSessionIdentity, ClientState,
};
use client::ui::lobby::{
    apply_class_locked, apply_classes_revealed, apply_join_ack, apply_lobby_handshake,
    apply_room_created, apply_slot_update, lobby_status_copy, LobbyCamera, LobbyInputState,
    LobbyUiPlugin, LobbyViewState,
};
use shared::card::ClassId;
use shared::protocol::{
    BoardSnapshot, GameMode, PlacementTimerMultiplier, RoundPhase, S2CClassLocked,
    S2CClassesRevealed, S2CGameSnapshot, S2CHandshake, S2CJoinAck, S2CRoomCreated, S2CSlotUpdated,
    SessionSlot,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const TOKEN: [u8; 16] = [7; 16];

#[test]
fn fresh_handshake_records_identity_without_entering_session() {
    test_helpers::init_test_tracing();
    let handshake = S2CHandshake {
        protocol_version: 1,
        session_id: 42,
        session_token: TOKEN,
        player_id: PlayerId(11),
    };
    let mut identity = ClientSessionIdentity::default();
    let mut lobby = LobbyViewState::default();

    apply_lobby_handshake(&mut lobby, &mut identity, &handshake);

    assert_eq!(identity.player_id, Some(PlayerId(11)));
    assert_eq!(identity.session_id, Some(42));
    assert_eq!(identity.session_token, Some(TOKEN));
    assert_eq!(lobby.local_player_id, Some(PlayerId(11)));
    assert!(!should_enter_session_from_phase(
        &identity,
        RoundPhase::Lobby
    ));
}

#[test]
fn lobby_startup_spawns_visible_ui_camera_until_session_entry() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(LobbyUiPlugin);

    app.update();
    app.update();

    {
        let world = app.world_mut();
        let mut cameras = world.query_filtered::<Entity, (With<Camera2d>, With<LobbyCamera>)>();
        assert_eq!(cameras.iter(world).count(), 1);

        let mut texts = world.query::<&Text>();
        let rendered_copy = texts
            .iter(world)
            .map(|text| text.0.as_str())
            .collect::<Vec<_>>();
        // PROMPT 1138 — AUDIT-1129-13 dropped the redundant `Status: ` prefix
        // from `lobby_status_copy`; the substring assertion now checks the
        // raw status state surface ("Connecting"), which is invariant under
        // the regrouping.
        assert!(
            rendered_copy.iter().any(|copy| copy.contains("Connecting")),
            "lobby text should be populated on the first lobby frame"
        );
    }

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    let world = app.world_mut();
    let mut cameras = world.query_filtered::<Entity, With<LobbyCamera>>();
    assert_eq!(cameras.iter(world).count(), 0);
}

#[test]
fn lobby_state_is_updated_only_from_authoritative_s2c_messages() {
    test_helpers::init_test_tracing();
    let mut lobby = LobbyViewState::default();
    let input = LobbyInputState {
        join_room_code: "abc123".to_string(),
        room_code_focused: false,
        room_code_selected: false,
        requested_slot: 1,
        selected_class: ClassId::Xelor,
        create_in_flight: false,
        join_in_flight: false,
        class_confirm_in_flight: false,
    };

    assert_eq!(lobby.room_code, None);
    assert!(lobby.slots.is_empty());

    let created = S2CRoomCreated {
        session_id: "session-a".to_string(),
        room_code: "ABC123".to_string(),
        mode: GameMode::OneVOne,
        slots: vec![
            slot(0, 0, Some(PlayerId(1)), None, false),
            slot(1, 1, None, None, false),
        ],
    };
    apply_room_created(&mut lobby, &created);

    assert_eq!(lobby.room_code.as_deref(), Some("ABC123"));
    assert_eq!(lobby.slots.len(), 2);
    assert_eq!(lobby.slots[0].player_id, Some(PlayerId(1)));

    let joined = S2CJoinAck {
        session_id: "session-a".to_string(),
        mode: GameMode::OneVOne,
        slots: vec![
            slot(0, 0, Some(PlayerId(1)), None, false),
            slot(1, 1, Some(PlayerId(2)), None, false),
        ],
    };
    apply_join_ack(&mut lobby, &joined);

    assert_eq!(lobby.slots[1].player_id, Some(PlayerId(2)));
    assert!(lobby_status_copy(&lobby, &input).contains("Players: 2/2"));
}

#[test]
fn class_confirmations_are_server_confirmed() {
    test_helpers::init_test_tracing();
    let mut lobby = LobbyViewState::default();

    apply_slot_update(
        &mut lobby,
        &S2CSlotUpdated {
            slots: vec![
                slot(0, 0, Some(PlayerId(1)), Some(ClassId::Iop), true),
                slot(1, 1, Some(PlayerId(2)), Some(ClassId::Cra), true),
            ],
        },
    );
    apply_class_locked(
        &mut lobby,
        &S2CClassLocked {
            class_id: ClassId::Iop,
        },
    );
    apply_classes_revealed(
        &mut lobby,
        &S2CClassesRevealed {
            player_class_map: vec![(PlayerId(1), ClassId::Iop), (PlayerId(2), ClassId::Cra)],
        },
    );

    assert_eq!(lobby.locked_class, Some(ClassId::Iop));
    assert_eq!(lobby.revealed_classes.len(), 2);
    assert!(lobby.slots.iter().all(|slot| slot.class_confirmed));
}

#[test]
fn in_session_transition_requires_server_phase_or_snapshot_and_identity() {
    test_helpers::init_test_tracing();
    let mut identity = ClientSessionIdentity::default();
    assert!(!should_enter_session_from_phase(
        &identity,
        RoundPhase::DraftInitial
    ));

    apply_handshake_message(
        &S2CHandshake {
            protocol_version: 1,
            session_id: 3,
            session_token: TOKEN,
            player_id: PlayerId(9),
        },
        &mut identity,
    );

    assert!(!should_enter_session_from_phase(
        &identity,
        RoundPhase::Lobby
    ));
    assert!(should_enter_session_from_phase(
        &identity,
        RoundPhase::DraftInitial
    ));

    let matching_snapshot = snapshot(PlayerId(9), RoundPhase::DraftInitial);
    let other_snapshot = snapshot(PlayerId(10), RoundPhase::DraftInitial);
    assert!(should_enter_session_from_snapshot(
        &identity,
        &matching_snapshot
    ));
    assert!(!should_enter_session_from_snapshot(
        &identity,
        &other_snapshot
    ));
}

#[test]
fn heartbeat_timer_uses_practical_unreliable_path_interval() {
    test_helpers::init_test_tracing();
    let mut timer = ClientHeartbeatTimer::default();

    assert!(!heartbeat_due_after_tick(
        &mut timer,
        Duration::from_secs(4)
    ));
    assert!(heartbeat_due_after_tick(&mut timer, Duration::from_secs(1)));
}

fn slot(
    slot: u8,
    team: u8,
    player_id: Option<PlayerId>,
    class_id: Option<ClassId>,
    class_confirmed: bool,
) -> SessionSlot {
    SessionSlot {
        slot,
        team,
        player_id,
        class_id,
        class_confirmed,
    }
}

fn snapshot(recipient_player_id: PlayerId, phase: RoundPhase) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id,
        round_number: 1,
        phase,
        timer_remaining_ms: Some(30_000),
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players: Vec::new(),
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}
