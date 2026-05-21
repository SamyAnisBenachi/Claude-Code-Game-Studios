//! PROMPT 1603 (BOT-FLOW-TWO-BOT-SOAK-ENTRYPOINT).
//!
//! Validates the debug-only `Create 2-Bot Soak Room` lobby entrypoint:
//!
//! * `debug_ui_enabled_from` enforces the strict `"1"` env contract (no other
//!   value enables the surface).
//! * `request_create_bot_room` enqueues a `LobbyCommand::CreateBotRoom` only
//!   when no session is active yet, mirroring the server-side
//!   `S2CBotActionRejected::AlreadyInSession` guard so the client UX cannot
//!   double-create.
//! * `LobbyUiPlugin` does NOT spawn the debug button when `CCGS_DEBUG_UI` is
//!   unset, so the production lobby is unchanged.

use bevy::ecs::system::RunSystemOnce;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::ClientState;
use client::ui::lobby::{
    debug_ui_enabled_from, request_create_bot_room, LobbyCommand, LobbyCreateBotRoomButton,
    LobbyUiPlugin, LobbyViewState,
};
use shared::protocol::{BotKind, GameMode};

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
fn test_debug_ui_enabled_only_for_exact_one_value() {
    test_helpers::init_test_tracing();

    // Arrange / Act / Assert — strict `"1"` contract.
    assert!(debug_ui_enabled_from(Some("1")));
    assert!(debug_ui_enabled_from(Some(" 1 ")), "leading/trailing whitespace trimmed");

    // Every other shape is disabled, mirroring the CCGS_QA_SNAPSHOT precedent.
    assert!(!debug_ui_enabled_from(None));
    assert!(!debug_ui_enabled_from(Some("")));
    assert!(!debug_ui_enabled_from(Some("0")));
    assert!(!debug_ui_enabled_from(Some("true")));
    assert!(!debug_ui_enabled_from(Some("yes")));
    assert!(!debug_ui_enabled_from(Some("11")));
    assert!(!debug_ui_enabled_from(Some("1\n0")));
}

#[test]
fn test_request_create_bot_room_enqueues_command_before_session() {
    test_helpers::init_test_tracing();
    let mut app = lobby_app();

    let _ = app.world_mut().run_system_once(
        |mut lobby: ResMut<LobbyViewState>, mut writer: MessageWriter<LobbyCommand>| {
            assert!(lobby.session_id.is_none());
            request_create_bot_room(
                GameMode::OneVOne,
                BotKind::Default,
                &mut lobby,
                &mut writer,
            );
        },
    );

    let commands = collect_lobby_commands(app.world_mut());
    assert_eq!(commands.len(), 1, "expected exactly one CreateBotRoom command");
    assert_eq!(
        commands[0],
        LobbyCommand::CreateBotRoom {
            mode: GameMode::OneVOne,
            bot_kind: BotKind::Default,
        }
    );
    let lobby = app.world().resource::<LobbyViewState>();
    assert!(
        lobby.status.contains("Creating 2-bot soak room"),
        "status must announce the in-flight create; got {}",
        lobby.status
    );
}

#[test]
fn test_request_create_bot_room_blocked_when_already_in_session() {
    test_helpers::init_test_tracing();
    let mut app = lobby_app();
    app.world_mut().resource_mut::<LobbyViewState>().session_id =
        Some("SESSION-ABC".to_string());

    let _ = app.world_mut().run_system_once(
        |mut lobby: ResMut<LobbyViewState>, mut writer: MessageWriter<LobbyCommand>| {
            request_create_bot_room(
                GameMode::OneVOne,
                BotKind::Default,
                &mut lobby,
                &mut writer,
            );
        },
    );

    let commands = collect_lobby_commands(app.world_mut());
    assert!(
        commands.is_empty(),
        "request_create_bot_room must refuse silently when session_id is Some; got {commands:?}"
    );
    let lobby = app.world().resource::<LobbyViewState>();
    assert!(
        lobby.status.contains("Already in a room"),
        "status must hint the conflict; got {}",
        lobby.status
    );
}

#[test]
fn test_lobby_ui_does_not_spawn_debug_button_without_env() {
    test_helpers::init_test_tracing();
    // Defensive: a sibling test or parent harness might have leaked the env.
    // SAFETY: this test is the sole reader/writer of the var in this file
    // and cargo-test isolates each crate's test process; matches the
    // CCGS_QA_SNAPSHOT integration test pattern.
    // SAFETY: std::env::remove_var is unsafe on Rust 2024; the test process
    // owns its environment.
    unsafe {
        std::env::remove_var("CCGS_DEBUG_UI");
    }

    let mut app = lobby_app();
    let world = app.world_mut();
    let mut q = world.query::<&LobbyCreateBotRoomButton>();
    assert_eq!(
        q.iter(world).count(),
        0,
        "LobbyUiPlugin must NOT spawn the debug Create-Bot-Room button when \
         CCGS_DEBUG_UI is unset; the surface is debug-only and absent from \
         production lobbies"
    );
}
