use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::ClientState;
use client::ui::lobby::{
    apply_room_list, format_room_list_row_label, lobby_initial_room_list_refresh_system,
    request_join_room_from_row, request_refresh_rooms, LobbyCommand, LobbyInputState,
    LobbyRefreshRoomsButton, LobbyRequestedSlotButton, LobbyRequestedSlotLabel,
    LobbyRoomListContainer, LobbyRoomListRow, LobbyUiPlugin, LobbyViewState,
};
use shared::protocol::{GameMode, RoomListEntry, S2CRoomList};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn entry(code: &str, slot: Option<u8>, filled: u8, max: u8, mode: GameMode) -> RoomListEntry {
    RoomListEntry {
        room_code: code.to_string(),
        mode,
        slots_filled: filled,
        slots_max: max,
        first_open_slot: slot,
    }
}

fn lobby_browser_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.add_plugins(LobbyUiPlugin);
    app.update();
    app
}

fn collect_lobby_commands(world: &mut World) -> Vec<LobbyCommand> {
    let mut out = Vec::new();
    world.resource_scope(
        |_world, mut messages: Mut<bevy::ecs::message::Messages<LobbyCommand>>| {
            for message in messages.drain() {
                out.push(message);
            }
        },
    );
    out
}

#[test]
fn test_drain_room_list_populates_lobby_view_state() {
    test_helpers::init_test_tracing();
    let mut lobby = LobbyViewState::default();
    let message = S2CRoomList {
        rooms: vec![
            entry("AAAAAA", Some(1), 1, 2, GameMode::OneVOne),
            entry("BBBBBB", Some(2), 2, 4, GameMode::TwoVTwo),
        ],
    };

    apply_room_list(&mut lobby, &message);

    assert_eq!(lobby.room_list.len(), 2);
    assert_eq!(lobby.room_list[0].room_code, "AAAAAA");
    assert_eq!(lobby.room_list[0].first_open_slot, Some(1));
    assert_eq!(lobby.room_list[1].room_code, "BBBBBB");
    assert_eq!(lobby.room_list[1].slots_max, 4);
}

#[test]
fn test_apply_empty_room_list_clears_view_state() {
    test_helpers::init_test_tracing();
    let mut lobby = LobbyViewState::default();
    lobby.room_list = vec![entry("AAAAAA", Some(1), 1, 2, GameMode::OneVOne)];

    apply_room_list(&mut lobby, &S2CRoomList { rooms: Vec::new() });

    assert!(
        lobby.room_list.is_empty(),
        "empty S2CRoomList must clear the view state list"
    );
}

#[test]
fn test_handshake_triggers_one_refresh_rooms_command() {
    test_helpers::init_test_tracing();
    let mut app = lobby_browser_app();

    // No handshake yet — lobby.local_player_id is None, so the refresh system
    // must NOT write a command on this update.
    app.update();
    let baseline = collect_lobby_commands(app.world_mut());
    assert!(
        !baseline
            .iter()
            .any(|cmd| matches!(cmd, LobbyCommand::RefreshRooms)),
        "no RefreshRooms before handshake; baseline={:?}",
        baseline
    );

    // Simulate handshake by setting local_player_id; mirrors the post-handshake
    // state set by `apply_lobby_handshake`.
    app.world_mut()
        .resource_mut::<LobbyViewState>()
        .local_player_id = Some(PlayerId(7));

    app.update();
    let first_tick = collect_lobby_commands(app.world_mut());
    let refresh_count_after_first = first_tick
        .iter()
        .filter(|cmd| matches!(cmd, LobbyCommand::RefreshRooms))
        .count();
    assert_eq!(
        refresh_count_after_first, 1,
        "exactly one RefreshRooms is enqueued on the first post-handshake tick; got {:?}",
        first_tick
    );

    // Subsequent ticks must not re-enqueue another RefreshRooms (Local<bool>
    // latch).
    app.update();
    app.update();
    let later = collect_lobby_commands(app.world_mut());
    let refresh_count_later = later
        .iter()
        .filter(|cmd| matches!(cmd, LobbyCommand::RefreshRooms))
        .count();
    assert_eq!(
        refresh_count_later, 0,
        "RefreshRooms is enqueued exactly once per handshake; got {:?}",
        later
    );
}

#[test]
fn test_initial_refresh_resets_when_local_player_id_clears() {
    test_helpers::init_test_tracing();
    let mut app = lobby_browser_app();

    app.world_mut()
        .resource_mut::<LobbyViewState>()
        .local_player_id = Some(PlayerId(3));
    app.update();
    let _ = collect_lobby_commands(app.world_mut());

    // Reset identity (handshake-reset path / disconnect).
    app.world_mut()
        .resource_mut::<LobbyViewState>()
        .local_player_id = None;
    app.update();
    let _ = collect_lobby_commands(app.world_mut());

    // New handshake — must re-fire exactly one RefreshRooms.
    app.world_mut()
        .resource_mut::<LobbyViewState>()
        .local_player_id = Some(PlayerId(9));
    app.update();
    let cmds = collect_lobby_commands(app.world_mut());
    let refresh_count = cmds
        .iter()
        .filter(|cmd| matches!(cmd, LobbyCommand::RefreshRooms))
        .count();
    assert_eq!(
        refresh_count, 1,
        "second handshake should re-enqueue exactly one RefreshRooms; got {:?}",
        cmds
    );
}

#[test]
fn test_clicking_room_row_writes_join_command_with_first_open_slot() {
    test_helpers::init_test_tracing();
    let mut app = lobby_browser_app();
    app.world_mut()
        .resource_mut::<LobbyViewState>()
        .local_player_id = Some(PlayerId(1));
    app.update();
    let _ = collect_lobby_commands(app.world_mut());

    // Drive a row click via a one-shot system that calls request_join_room_from_row.
    let click_system = move |mut input: ResMut<LobbyInputState>,
                             mut lobby: ResMut<LobbyViewState>,
                             mut commands: MessageWriter<LobbyCommand>| {
        let row = LobbyRoomListRow {
            room_code: "ABCDEF".to_string(),
            requested_slot: 2,
        };
        request_join_room_from_row(&row, &mut input, &mut lobby, &mut commands);
    };
    let id = app.world_mut().register_system(click_system);
    app.world_mut().run_system(id).unwrap();
    app.update();

    let cmds = collect_lobby_commands(app.world_mut());
    let join = cmds
        .iter()
        .find(|cmd| matches!(cmd, LobbyCommand::JoinRoom { .. }));
    let expected = LobbyCommand::JoinRoom {
        room_code: "ABCDEF".to_string(),
        requested_slot: 2,
    };
    assert_eq!(
        join.cloned(),
        Some(expected),
        "click on row writes JoinRoom with server-supplied first_open_slot; got {:?}",
        cmds
    );

    // join_in_flight latch is engaged so subsequent clicks are short-circuited.
    assert!(
        app.world().resource::<LobbyInputState>().join_in_flight,
        "join_in_flight latch must be set after the click"
    );

    // PROMPT 1178 — clicking a row also syncs the optimistic input-side
    // mirror so the displayed own-slot label can no longer read a stale
    // default while the server's `S2CJoinAck` is still in flight. The
    // authoritative slot still lands via `apply_join_ack` ->
    // `lobby.slots`; this just keeps the pre-ack readout coherent.
    assert_eq!(
        app.world().resource::<LobbyInputState>().requested_slot,
        2,
        "PROMPT 1178: clicking a row whose `first_open_slot = 2` must sync \
         `input.requested_slot` to `2` so the optimistic own-slot label \
         cannot read the stale `LobbyInputState::default()` slot 1"
    );
}

/// PROMPT 1178 — clicking a row whose `first_open_slot` differs from the
/// `LobbyInputState::default()` slot must surface as a join command that
/// targets the row's slot AND must keep `input.requested_slot` in sync.
/// This guards against a regression where the row click silently joined
/// a different slot than the one the UI was about to display.
#[test]
fn test_clicking_row_with_different_slot_syncs_requested_slot() {
    test_helpers::init_test_tracing();
    let mut app = lobby_browser_app();
    app.world_mut()
        .resource_mut::<LobbyViewState>()
        .local_player_id = Some(PlayerId(1));
    app.update();
    let _ = collect_lobby_commands(app.world_mut());

    // `LobbyInputState::default()` reads `requested_slot = 1`. Click a
    // row whose only open seat is slot `3` and confirm both sides line
    // up.
    let click_system = move |mut input: ResMut<LobbyInputState>,
                             mut lobby: ResMut<LobbyViewState>,
                             mut commands: MessageWriter<LobbyCommand>| {
        let row = LobbyRoomListRow {
            room_code: "ZZZZZZ".to_string(),
            requested_slot: 3,
        };
        request_join_room_from_row(&row, &mut input, &mut lobby, &mut commands);
    };
    let id = app.world_mut().register_system(click_system);
    app.world_mut().run_system(id).unwrap();
    app.update();

    assert_eq!(
        app.world().resource::<LobbyInputState>().requested_slot,
        3,
        "PROMPT 1178: row.requested_slot = 3 must be mirrored into \
         LobbyInputState.requested_slot so the own-slot label cannot read \
         stale slot 1 while the join is in flight"
    );
    let cmds = collect_lobby_commands(app.world_mut());
    assert!(
        cmds.iter().any(|cmd| matches!(
            cmd,
            LobbyCommand::JoinRoom {
                requested_slot: 3,
                ..
            }
        )),
        "PROMPT 1178: row click must still write JoinRoom with row's \
         first_open_slot (= 3); got {cmds:?}"
    );
}

/// PROMPT 1178 — existing-room browser block (refresh button + room-list
/// container + room rows) is rendered only while `lobby.session_id` is
/// `None`. Once the local player has joined or created a room, the
/// browser is irrelevant: it can only list OTHER joinable rooms (the
/// server filters out the local player's own room), and clicking
/// another row at that point would race the `S2CJoinAck`. Hiding it
/// post-join also reclaims panel content height — part of the budget
/// that keeps the Confirm CTA visible at the minimum 1280×720
/// viewport.
#[test]
fn test_existing_rooms_block_is_hidden_after_session_is_joined() {
    test_helpers::init_test_tracing();
    let mut app = lobby_browser_app();

    // Pre-session baseline: refresh button + container present.
    {
        let world = app.world_mut();
        let pre_refresh = world
            .query_filtered::<Entity, With<LobbyRefreshRoomsButton>>()
            .iter(world)
            .count();
        let pre_container = world
            .query_filtered::<Entity, With<LobbyRoomListContainer>>()
            .iter(world)
            .count();
        assert_eq!(
            pre_refresh, 1,
            "pre-session baseline: exactly one LobbyRefreshRoomsButton must exist"
        );
        assert_eq!(
            pre_container, 1,
            "pre-session baseline: exactly one LobbyRoomListContainer must exist"
        );
    }

    // Simulate the player joining a room (the lobby plugin enters with no
    // session; the room browser is what the user clicks to acquire one).
    // The lobby UI is respawned only on `OnEnter(ClientState::Lobby)`, so
    // we re-spawn the lobby UI by toggling state. Simpler: assert the
    // composition after re-spawning from scratch with a session injected
    // *before* the spawn system runs. We achieve that by despawning the
    // lobby UI and re-entering Lobby state with session_id pre-set.
    //
    // The lobby plugin's OnEnter system bails if a root already exists,
    // so the cleanest in-test approach is to drive the post-session
    // composition by spawning a fresh app with the session injected
    // before the initial `app.update()` runs.
    let mut joined_app = App::new();
    joined_app.add_plugins(MinimalPlugins);
    joined_app.add_plugins(bevy::asset::AssetPlugin::default());
    joined_app.init_asset::<bevy::image::Image>();
    joined_app.add_plugins(StatesPlugin);
    joined_app.init_state::<ClientState>();
    joined_app.insert_resource(ButtonInput::<KeyCode>::default());
    // Seed the lobby with a server-acknowledged session so the
    // OnEnter(Lobby) spawn system sees `session_id.is_some()`.
    joined_app.insert_resource(LobbyViewState {
        session_id: Some("session-uuid".to_string()),
        room_code: Some("ABCDEF".to_string()),
        local_player_id: Some(PlayerId(1)),
        ..Default::default()
    });
    joined_app.add_plugins(LobbyUiPlugin);
    joined_app.update();
    joined_app.update();

    let world = joined_app.world_mut();
    let refresh = world
        .query_filtered::<Entity, With<LobbyRefreshRoomsButton>>()
        .iter(world)
        .count();
    let container = world
        .query_filtered::<Entity, With<LobbyRoomListContainer>>()
        .iter(world)
        .count();
    let rows = world
        .query_filtered::<Entity, With<LobbyRoomListRow>>()
        .iter(world)
        .count();
    assert_eq!(
        refresh, 0,
        "PROMPT 1178: LobbyRefreshRoomsButton MUST NOT be present when \
         `lobby.session_id` is Some at OnEnter(Lobby); got {refresh}"
    );
    assert_eq!(
        container, 0,
        "PROMPT 1178: LobbyRoomListContainer MUST NOT be present when \
         `lobby.session_id` is Some at OnEnter(Lobby); got {container}"
    );
    assert_eq!(
        rows, 0,
        "PROMPT 1178: no LobbyRoomListRow may be spawned post-session; got {rows}"
    );
}

/// PROMPT 1178 — `Requested slot (manual join)` label + slot buttons are
/// rendered only while `lobby.session_id` is `None`. Once the player
/// has joined a room (via browser row click, manual typed-Join, or
/// `Create Room`), the requested-slot row is irrelevant — they've
/// already committed to a seat — and its continued presence read as
/// "next step is still slot selection", obscuring the actual Confirm
/// CTA. Hiding it post-session also reclaims panel content height,
/// part of the budget that keeps the Confirm CTA visible at the
/// minimum 1280×720 viewport.
#[test]
fn test_requested_slot_row_is_hidden_after_session_is_joined() {
    test_helpers::init_test_tracing();

    // Pre-session baseline: label + 4 slot buttons present.
    let mut pre_app = lobby_browser_app();
    {
        let world = pre_app.world_mut();
        let labels = world
            .query_filtered::<Entity, With<LobbyRequestedSlotLabel>>()
            .iter(world)
            .count();
        let buttons = world
            .query_filtered::<Entity, With<LobbyRequestedSlotButton>>()
            .iter(world)
            .count();
        assert_eq!(
            labels, 1,
            "pre-session baseline: exactly one LobbyRequestedSlotLabel must exist"
        );
        assert_eq!(
            buttons, 4,
            "pre-session baseline: exactly 4 LobbyRequestedSlotButton entities \
             (slots 0..=3) must exist"
        );
    }

    // Post-session: respawn with `session_id` seeded.
    let mut joined_app = App::new();
    joined_app.add_plugins(MinimalPlugins);
    joined_app.add_plugins(bevy::asset::AssetPlugin::default());
    joined_app.init_asset::<bevy::image::Image>();
    joined_app.add_plugins(StatesPlugin);
    joined_app.init_state::<ClientState>();
    joined_app.insert_resource(ButtonInput::<KeyCode>::default());
    joined_app.insert_resource(LobbyViewState {
        session_id: Some("session-uuid".to_string()),
        room_code: Some("ABCDEF".to_string()),
        local_player_id: Some(PlayerId(1)),
        ..Default::default()
    });
    joined_app.add_plugins(LobbyUiPlugin);
    joined_app.update();
    joined_app.update();

    let world = joined_app.world_mut();
    let labels = world
        .query_filtered::<Entity, With<LobbyRequestedSlotLabel>>()
        .iter(world)
        .count();
    let buttons = world
        .query_filtered::<Entity, With<LobbyRequestedSlotButton>>()
        .iter(world)
        .count();
    assert_eq!(
        labels, 0,
        "PROMPT 1178: LobbyRequestedSlotLabel MUST NOT be present post-session; \
         got {labels}"
    );
    assert_eq!(
        buttons, 0,
        "PROMPT 1178: LobbyRequestedSlotButton entities MUST NOT be present \
         post-session; got {buttons}"
    );
}

#[test]
fn test_full_room_renders_without_row_component() {
    test_helpers::init_test_tracing();
    let mut app = lobby_browser_app();

    // Inject a synthetic room list with one joinable and one full row.
    app.world_mut().resource_mut::<LobbyViewState>().room_list = vec![
        entry("AAAAAA", Some(1), 1, 2, GameMode::OneVOne),
        entry("FFFFFF", None, 2, 2, GameMode::OneVOne),
    ];

    // refresh_lobby_ui_system runs once the LobbyViewState changes.
    app.update();
    app.update();

    let world = app.world_mut();
    let mut row_query = world.query::<&LobbyRoomListRow>();
    let row_codes: Vec<String> = row_query
        .iter(world)
        .map(|row| row.room_code.clone())
        .collect();
    assert_eq!(
        row_codes.len(),
        1,
        "only joinable rows carry LobbyRoomListRow"
    );
    assert_eq!(
        row_codes[0], "AAAAAA",
        "the joinable row keeps its code; the full row is rendered as a non-button label"
    );
}

#[test]
fn test_refresh_button_request_writes_one_refresh_rooms_command() {
    test_helpers::init_test_tracing();
    let mut app = lobby_browser_app();
    // Skip the handshake-driven initial refresh by running an early update
    // before flipping local_player_id.
    app.update();
    let _ = collect_lobby_commands(app.world_mut());

    // Drive a refresh via a one-shot system that calls request_refresh_rooms.
    let click_system = move |mut lobby: ResMut<LobbyViewState>,
                             mut commands: MessageWriter<LobbyCommand>| {
        request_refresh_rooms(&mut lobby, &mut commands);
    };
    let id = app.world_mut().register_system(click_system);
    app.world_mut().run_system(id).unwrap();
    app.update();

    let cmds = collect_lobby_commands(app.world_mut());
    let refresh_count = cmds
        .iter()
        .filter(|cmd| matches!(cmd, LobbyCommand::RefreshRooms))
        .count();
    assert_eq!(
        refresh_count, 1,
        "request_refresh_rooms writes exactly one RefreshRooms; got {:?}",
        cmds
    );
    assert_eq!(
        app.world().resource::<LobbyViewState>().status,
        "Refreshing rooms",
        "status banner reflects the refresh action"
    );
}

#[test]
fn test_row_label_formats_room_code_mode_filled_max() {
    test_helpers::init_test_tracing();
    let entry = entry("ABCDEF", Some(2), 2, 4, GameMode::TwoVTwo);

    let label = format_room_list_row_label(&entry);

    assert_eq!(label, "ABCDEF · TwoVTwo · 2/4");
}

#[test]
fn test_lobby_initial_room_list_refresh_system_is_exposed() {
    // Compile-only guard: the system must remain part of the public API so the
    // plugin can register it. If the symbol disappears, the link breaks here
    // before drift accumulates into a silent regression.
    let _fn_ptr: fn(Res<LobbyViewState>, MessageWriter<LobbyCommand>, Local<bool>) =
        lobby_initial_room_list_refresh_system;
}
