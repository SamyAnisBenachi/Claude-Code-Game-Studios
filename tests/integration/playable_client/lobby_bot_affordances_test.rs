//! PROMPT 1537 — lobby bot affordance wiring.
//!
//! Verifies the minimal client-side contract for the `Play vs Bot`
//! (pre-room) and `Add Bot` (in-room) lobby buttons. Tests follow the
//! existing `lobby_room_browser_test.rs` pattern: build a minimal Bevy
//! app with `LobbyUiPlugin`, drive the helpers directly, then drain
//! `LobbyCommand` messages to assert exactly what got written.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::ClientState;
use client::ui::lobby::{
    lobby_add_bot_target_slot, request_add_bot, request_play_vs_bot, LobbyCommand,
    LobbyInputState, LobbyUiPlugin, LobbyViewState,
};
use shared::card::ClassId;
use shared::protocol::{BotKind, GameMode, SessionSlot};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn empty_slot(slot: u8, team: u8) -> SessionSlot {
    SessionSlot {
        slot,
        team,
        player_id: None,
        class_id: None,
        class_confirmed: false,
        is_bot: false,
    }
}

fn occupied_slot(slot: u8, team: u8, player: PlayerId) -> SessionSlot {
    SessionSlot {
        slot,
        team,
        player_id: Some(player),
        class_id: Some(ClassId::Iop),
        class_confirmed: false,
        is_bot: false,
    }
}

fn bot_slot(slot: u8, team: u8) -> SessionSlot {
    SessionSlot {
        slot,
        team,
        player_id: None,
        class_id: Some(ClassId::Iop),
        class_confirmed: false,
        is_bot: true,
    }
}

fn lobby_bot_app() -> App {
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

fn drain_commands(world: &mut World) -> Vec<LobbyCommand> {
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
fn add_bot_target_slot_picks_first_empty_non_bot_seat() {
    test_helpers::init_test_tracing();
    let mut lobby = LobbyViewState::default();
    lobby.slots = vec![
        occupied_slot(0, 1, PlayerId(1)),
        bot_slot(1, 2),
        empty_slot(2, 2),
        empty_slot(3, 2),
    ];

    assert_eq!(lobby_add_bot_target_slot(&lobby), Some(2));
}

#[test]
fn add_bot_target_slot_none_when_no_empty_human_seat() {
    test_helpers::init_test_tracing();
    let mut lobby = LobbyViewState::default();
    lobby.slots = vec![
        occupied_slot(0, 1, PlayerId(1)),
        occupied_slot(1, 2, PlayerId(2)),
    ];

    assert_eq!(lobby_add_bot_target_slot(&lobby), None);
}

#[test]
fn add_bot_target_slot_skips_already_bot_seats() {
    test_helpers::init_test_tracing();
    let mut lobby = LobbyViewState::default();
    lobby.slots = vec![occupied_slot(0, 1, PlayerId(1)), bot_slot(1, 2)];

    assert_eq!(lobby_add_bot_target_slot(&lobby), None);
}

#[test]
fn request_play_vs_bot_writes_command_and_latches_in_flight() {
    test_helpers::init_test_tracing();
    let mut app = lobby_bot_app();
    let world = app.world_mut();

    world.resource_scope(|world, mut lobby: Mut<LobbyViewState>| {
        world.resource_scope(|world, mut input: Mut<LobbyInputState>| {
            let mut commands_state =
                bevy::ecs::system::SystemState::<MessageWriter<LobbyCommand>>::new(world);
            let mut commands = commands_state.get_mut(world);
            request_play_vs_bot(&mut input, &mut lobby, &mut commands);
            commands_state.apply(world);

            assert!(
                input.bot_action_in_flight,
                "request_play_vs_bot must set the in-flight latch"
            );
            assert!(
                lobby.status.contains("bot"),
                "status banner must mention bot: {:?}",
                lobby.status
            );
        });
    });

    let cmds = drain_commands(app.world_mut());
    assert_eq!(cmds.len(), 1, "exactly one command must be written");
    match &cmds[0] {
        LobbyCommand::PlayVsBot { bot_kind } => {
            assert_eq!(*bot_kind, BotKind::Default);
        }
        other => panic!("expected LobbyCommand::PlayVsBot, got {other:?}"),
    }
}

#[test]
fn request_play_vs_bot_rejected_when_already_in_room() {
    test_helpers::init_test_tracing();
    let mut app = lobby_bot_app();
    let world = app.world_mut();

    world.resource_scope(|world, mut lobby: Mut<LobbyViewState>| {
        lobby.session_id = Some("session-1".into());
        world.resource_scope(|world, mut input: Mut<LobbyInputState>| {
            let mut commands_state =
                bevy::ecs::system::SystemState::<MessageWriter<LobbyCommand>>::new(world);
            let mut commands = commands_state.get_mut(world);
            request_play_vs_bot(&mut input, &mut lobby, &mut commands);
            commands_state.apply(world);

            assert!(!input.bot_action_in_flight);
        });
    });

    let cmds = drain_commands(app.world_mut());
    assert!(
        cmds.is_empty(),
        "Play vs Bot must be a no-op when session_id is Some, got {cmds:?}"
    );
}

#[test]
fn request_play_vs_bot_drops_repeat_while_in_flight() {
    test_helpers::init_test_tracing();
    let mut app = lobby_bot_app();
    let world = app.world_mut();

    world.resource_scope(|world, mut lobby: Mut<LobbyViewState>| {
        world.resource_scope(|world, mut input: Mut<LobbyInputState>| {
            input.bot_action_in_flight = true;
            let mut commands_state =
                bevy::ecs::system::SystemState::<MessageWriter<LobbyCommand>>::new(world);
            let mut commands = commands_state.get_mut(world);
            request_play_vs_bot(&mut input, &mut lobby, &mut commands);
            commands_state.apply(world);
        });
    });

    let cmds = drain_commands(app.world_mut());
    assert!(
        cmds.is_empty(),
        "second click while in-flight must not stack a command, got {cmds:?}"
    );
}

#[test]
fn request_add_bot_writes_command_with_first_open_slot() {
    test_helpers::init_test_tracing();
    let mut app = lobby_bot_app();
    let world = app.world_mut();

    world.resource_scope(|world, mut lobby: Mut<LobbyViewState>| {
        lobby.session_id = Some("session-1".into());
        lobby.mode = GameMode::OneVOne;
        lobby.slots = vec![occupied_slot(0, 1, PlayerId(1)), empty_slot(1, 2)];

        world.resource_scope(|world, mut input: Mut<LobbyInputState>| {
            let mut commands_state =
                bevy::ecs::system::SystemState::<MessageWriter<LobbyCommand>>::new(world);
            let mut commands = commands_state.get_mut(world);
            request_add_bot(&mut input, &mut lobby, &mut commands);
            commands_state.apply(world);

            assert!(input.bot_action_in_flight);
        });
    });

    let cmds = drain_commands(app.world_mut());
    assert_eq!(cmds.len(), 1);
    match &cmds[0] {
        LobbyCommand::AddBot { slot, bot_kind } => {
            assert_eq!(*slot, 1, "must target the first empty non-bot slot");
            assert_eq!(*bot_kind, BotKind::Default);
        }
        other => panic!("expected LobbyCommand::AddBot, got {other:?}"),
    }
}

#[test]
fn request_add_bot_no_op_when_no_open_seat() {
    test_helpers::init_test_tracing();
    let mut app = lobby_bot_app();
    let world = app.world_mut();

    world.resource_scope(|world, mut lobby: Mut<LobbyViewState>| {
        lobby.session_id = Some("session-1".into());
        lobby.slots = vec![
            occupied_slot(0, 1, PlayerId(1)),
            occupied_slot(1, 2, PlayerId(2)),
        ];

        world.resource_scope(|world, mut input: Mut<LobbyInputState>| {
            let mut commands_state =
                bevy::ecs::system::SystemState::<MessageWriter<LobbyCommand>>::new(world);
            let mut commands = commands_state.get_mut(world);
            request_add_bot(&mut input, &mut lobby, &mut commands);
            commands_state.apply(world);

            assert!(!input.bot_action_in_flight);
        });
    });

    let cmds = drain_commands(app.world_mut());
    assert!(cmds.is_empty(), "no command when no eligible slot");
}

#[test]
fn request_add_bot_no_op_when_session_id_missing() {
    test_helpers::init_test_tracing();
    let mut app = lobby_bot_app();
    let world = app.world_mut();

    world.resource_scope(|world, mut lobby: Mut<LobbyViewState>| {
        lobby.session_id = None;
        lobby.slots = vec![empty_slot(0, 1), empty_slot(1, 2)];

        world.resource_scope(|world, mut input: Mut<LobbyInputState>| {
            let mut commands_state =
                bevy::ecs::system::SystemState::<MessageWriter<LobbyCommand>>::new(world);
            let mut commands = commands_state.get_mut(world);
            request_add_bot(&mut input, &mut lobby, &mut commands);
            commands_state.apply(world);

            assert!(!input.bot_action_in_flight);
        });
    });

    let cmds = drain_commands(app.world_mut());
    assert!(cmds.is_empty());
}
