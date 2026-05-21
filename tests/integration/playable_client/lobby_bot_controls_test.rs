//! PROMPT 1596 (BOT-FLOW-LOBBY-ADD-REMOVE-BOT-UX).
//!
//! Validates the contextual Add Bot / Remove Bot controls in the in-room
//! lobby UI:
//!
//! * `lobby_bot_controls_for_slots` returns the right control variants for
//!   empty / bot-occupied / human-occupied / local-player slots.
//! * `request_add_bot` / `request_remove_bot` enqueue the right
//!   `LobbyCommand` variants and gate cleanly when the player is not yet in
//!   a session.
//! * `LobbyAddBotButton` / `LobbyRemoveBotButton` carry the target slot
//!   index so the interaction system dispatches the right command.
//! * The `LobbyBotControlsContainer` spawns once via `LobbyUiPlugin`'s
//!   `OnEnter(Lobby)` hook so the dynamic rebuild path has a parent entity
//!   from frame 0; subsequent slot updates repopulate it reactively.

use bevy::ecs::system::RunSystemOnce;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::ClientState;
use client::ui::lobby::{
    lobby_bot_controls_for_slots, request_add_bot, request_remove_bot, LobbyAddBotButton,
    LobbyBotControlsContainer, LobbyBotSlotControl, LobbyCommand, LobbyRemoveBotButton,
    LobbyUiPlugin, LobbyViewState,
};
use shared::protocol::{BotKind, SessionSlot};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn lobby_app() -> App {
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

fn slot(slot_index: u8, team: u8, player_id: Option<PlayerId>, is_bot: bool) -> SessionSlot {
    SessionSlot {
        slot: slot_index,
        team,
        player_id,
        class_id: None,
        class_confirmed: false,
        is_bot,
    }
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
fn test_bot_controls_empty_when_no_slots() {
    test_helpers::init_test_tracing();
    let controls = lobby_bot_controls_for_slots(&[], Some(PlayerId(1)));
    assert!(
        controls.is_empty(),
        "no slots = no bot controls; got {controls:?}"
    );
}

#[test]
fn test_bot_controls_skip_local_player_slot() {
    test_helpers::init_test_tracing();
    let local = PlayerId(1);
    let slots = vec![slot(0, 0, Some(local), false), slot(1, 1, None, false)];

    let controls = lobby_bot_controls_for_slots(&slots, Some(local));

    assert_eq!(
        controls,
        vec![LobbyBotSlotControl::AddBot { slot: 1 }],
        "local player's own seat must not produce a control; only the empty \
         opposing seat is eligible"
    );
}

#[test]
fn test_bot_controls_add_for_empty_slot() {
    test_helpers::init_test_tracing();
    let local = PlayerId(1);
    let slots = vec![slot(0, 0, Some(local), false), slot(1, 1, None, false)];

    let controls = lobby_bot_controls_for_slots(&slots, Some(local));

    assert_eq!(controls, vec![LobbyBotSlotControl::AddBot { slot: 1 }]);
}

#[test]
fn test_bot_controls_remove_for_bot_slot() {
    test_helpers::init_test_tracing();
    let local = PlayerId(1);
    let bot = PlayerId(99);
    let slots = vec![
        slot(0, 0, Some(local), false),
        slot(1, 1, Some(bot), true),
    ];

    let controls = lobby_bot_controls_for_slots(&slots, Some(local));

    assert_eq!(controls, vec![LobbyBotSlotControl::RemoveBot { slot: 1 }]);
}

#[test]
fn test_bot_controls_skip_human_opponent_slot() {
    test_helpers::init_test_tracing();
    let local = PlayerId(1);
    let opp = PlayerId(2);
    let slots = vec![
        slot(0, 0, Some(local), false),
        slot(1, 1, Some(opp), false),
    ];

    let controls = lobby_bot_controls_for_slots(&slots, Some(local));

    assert!(
        controls.is_empty(),
        "a human opponent must not surface an Add Bot / Remove Bot affordance; \
         got {controls:?}"
    );
}

#[test]
fn test_bot_controls_stable_order_by_slot_index() {
    test_helpers::init_test_tracing();
    let local = PlayerId(1);
    let bot = PlayerId(99);
    // 2v2: local in slot 0, bot in slot 3, empty slot 2, empty slot 1.
    // Output must be sorted ascending by slot index regardless of input
    // order.
    let slots = vec![
        slot(3, 1, Some(bot), true),
        slot(1, 0, None, false),
        slot(0, 0, Some(local), false),
        slot(2, 1, None, false),
    ];

    let controls = lobby_bot_controls_for_slots(&slots, Some(local));

    assert_eq!(
        controls,
        vec![
            LobbyBotSlotControl::AddBot { slot: 1 },
            LobbyBotSlotControl::AddBot { slot: 2 },
            LobbyBotSlotControl::RemoveBot { slot: 3 },
        ],
        "controls must be sorted by ascending slot index"
    );
}

#[test]
fn test_request_add_bot_blocked_before_session() {
    test_helpers::init_test_tracing();
    let mut app = lobby_app();

    let _ = app.world_mut().run_system_once(
        |mut lobby: ResMut<LobbyViewState>, mut writer: MessageWriter<LobbyCommand>| {
            assert!(lobby.session_id.is_none());
            request_add_bot(1, BotKind::Default, &mut lobby, &mut writer);
        },
    );

    let commands = collect_lobby_commands(app.world_mut());
    assert!(
        commands.is_empty(),
        "request_add_bot must NOT enqueue a command when session_id is None; \
         got {commands:?}"
    );
    let lobby = app.world().resource::<LobbyViewState>();
    assert!(
        lobby.status.contains("Create or join a room"),
        "status must hint the corrective action; got {}",
        lobby.status
    );
}

#[test]
fn test_request_add_bot_enqueues_command_when_session_active() {
    test_helpers::init_test_tracing();
    let mut app = lobby_app();
    app.world_mut().resource_mut::<LobbyViewState>().session_id =
        Some("SESSION-ABC".to_string());

    let _ = app.world_mut().run_system_once(
        |mut lobby: ResMut<LobbyViewState>, mut writer: MessageWriter<LobbyCommand>| {
            request_add_bot(1, BotKind::Default, &mut lobby, &mut writer);
        },
    );

    let commands = collect_lobby_commands(app.world_mut());
    assert_eq!(commands.len(), 1, "expected exactly one AddBot command");
    assert_eq!(
        commands[0],
        LobbyCommand::AddBot {
            slot: 1,
            bot_kind: BotKind::Default,
        }
    );
    let lobby = app.world().resource::<LobbyViewState>();
    assert!(
        lobby.status.contains("Adding bot to seat 1"),
        "status must announce the in-flight add; got {}",
        lobby.status
    );
}

#[test]
fn test_request_remove_bot_blocked_before_session_and_enqueues_otherwise() {
    test_helpers::init_test_tracing();
    let mut app = lobby_app();

    // Pre-session: no command, hint status.
    let _ = app.world_mut().run_system_once(
        |mut lobby: ResMut<LobbyViewState>, mut writer: MessageWriter<LobbyCommand>| {
            request_remove_bot(1, &mut lobby, &mut writer);
        },
    );
    let commands = collect_lobby_commands(app.world_mut());
    assert!(commands.is_empty());

    // Post-session: command enqueued.
    app.world_mut().resource_mut::<LobbyViewState>().session_id =
        Some("SESSION-ABC".to_string());
    let _ = app.world_mut().run_system_once(
        |mut lobby: ResMut<LobbyViewState>, mut writer: MessageWriter<LobbyCommand>| {
            request_remove_bot(1, &mut lobby, &mut writer);
        },
    );
    let commands = collect_lobby_commands(app.world_mut());
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0], LobbyCommand::RemoveBot { slot: 1 });
    let lobby = app.world().resource::<LobbyViewState>();
    assert!(
        lobby.status.contains("Removing bot from seat 1"),
        "status must announce the in-flight remove; got {}",
        lobby.status
    );
}

#[test]
fn test_lobby_ui_plugin_spawns_bot_controls_container() {
    test_helpers::init_test_tracing();
    let mut app = lobby_app();

    let world = app.world_mut();
    let mut q = world.query::<&LobbyBotControlsContainer>();
    let count = q.iter(world).count();
    assert_eq!(
        count, 1,
        "LobbyUiPlugin must spawn exactly one LobbyBotControlsContainer on \
         OnEnter(Lobby); got {count}"
    );
}

#[test]
fn test_bot_controls_container_repopulates_reactively_with_slot_changes() {
    test_helpers::init_test_tracing();
    let mut app = lobby_app();
    let local = PlayerId(1);

    // Frame 0: no slots, no buttons.
    {
        let world = app.world_mut();
        let mut adds = world.query::<&LobbyAddBotButton>();
        let mut removes = world.query::<&LobbyRemoveBotButton>();
        assert_eq!(adds.iter(world).count(), 0);
        assert_eq!(removes.iter(world).count(), 0);
    }

    // Room created with the local player in slot 0 and an empty slot 1.
    {
        let world = app.world_mut();
        let mut lobby = world.resource_mut::<LobbyViewState>();
        lobby.local_player_id = Some(local);
        lobby.session_id = Some("SESSION-ABC".to_string());
        lobby.slots = vec![slot(0, 0, Some(local), false), slot(1, 1, None, false)];
    }
    // refresh_lobby_ui_system runs in Update; one update reads
    // lobby.is_changed() and queues despawn+spawn commands; a second update
    // is needed so the new entities materialise in the query.
    app.update();
    app.update();

    {
        let world = app.world_mut();
        let mut q = world.query::<&LobbyAddBotButton>();
        let buttons: Vec<u8> = q.iter(world).map(|b| b.slot).collect();
        assert_eq!(
            buttons,
            vec![1],
            "post-slot-update, expected one Add Bot button targeting slot 1; got {buttons:?}"
        );
    }

    // Server confirms bot occupancy in slot 1.
    {
        let world = app.world_mut();
        let mut lobby = world.resource_mut::<LobbyViewState>();
        lobby.slots = vec![
            slot(0, 0, Some(local), false),
            slot(1, 1, Some(PlayerId(99)), true),
        ];
    }
    app.update();
    app.update();

    {
        let world = app.world_mut();
        let mut adds = world.query::<&LobbyAddBotButton>();
        assert_eq!(
            adds.iter(world).count(),
            0,
            "Add Bot button must despawn once slot is bot-occupied"
        );
        let mut removes = world.query::<&LobbyRemoveBotButton>();
        let buttons: Vec<u8> = removes.iter(world).map(|b| b.slot).collect();
        assert_eq!(
            buttons,
            vec![1],
            "post-bot-occupancy, expected one Remove Bot button targeting slot 1; got {buttons:?}"
        );
    }

    // A human peer takes the seat instead.
    {
        let world = app.world_mut();
        let mut lobby = world.resource_mut::<LobbyViewState>();
        lobby.slots = vec![
            slot(0, 0, Some(local), false),
            slot(1, 1, Some(PlayerId(2)), false),
        ];
    }
    app.update();
    app.update();

    {
        let world = app.world_mut();
        let mut adds = world.query::<&LobbyAddBotButton>();
        let mut removes = world.query::<&LobbyRemoveBotButton>();
        assert_eq!(adds.iter(world).count(), 0);
        assert_eq!(
            removes.iter(world).count(),
            0,
            "human-occupied opposing slot must produce no controls"
        );
    }
}
