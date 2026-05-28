//! PROMPT 2042 (CLIENT-PHASE-SYNC-CREATE-BOT-ROOM-P0-REPAIR).
//!
//! Validates the autoplay-gated lobby autopilot that emits the
//! `CreateBotRoom -> ConfirmClass` `LobbyCommand` sequence directly, so a
//! fresh autoplay run can leave Lobby and enter InSession without the
//! pixel-guessed UI clicks the older add-bot-lobby recipe used.
//!
//! * `lobby_autopilot_enabled_from` enforces the strict `"1"` env contract.
//! * `lobby_autopilot_step` is a no-op when `enabled == false`.
//! * Stage 1: emits exactly one `CreateBotRoom` once a handshake has landed
//!   (`local_player_id == Some(..)`) and no `session_id` exists yet; the
//!   `create_bot_room_sent` latch prevents re-emission.
//! * Stage 2: emits exactly one `ConfirmClass` once a `session_id` exists
//!   and the local class is not yet locked; the `confirm_class_sent` latch
//!   and the `class_confirm_in_flight` flag prevent re-emission.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use client::ui::lobby::{
    lobby_autopilot_enabled_from, lobby_autopilot_step, LobbyAutopilotState, LobbyCommand,
    LobbyInputState, LobbyViewState,
};
use shared::card::ClassId;
use shared::protocol::{BotKind, GameMode};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn fresh_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<LobbyViewState>();
    app.init_resource::<LobbyInputState>();
    app.init_resource::<LobbyAutopilotState>();
    app.add_message::<LobbyCommand>();
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
fn test_lobby_autopilot_enabled_only_for_exact_one_value() {
    test_helpers::init_test_tracing();

    assert!(lobby_autopilot_enabled_from(Some("1")));
    assert!(
        lobby_autopilot_enabled_from(Some(" 1 ")),
        "leading/trailing whitespace trimmed"
    );

    assert!(!lobby_autopilot_enabled_from(None));
    assert!(!lobby_autopilot_enabled_from(Some("")));
    assert!(!lobby_autopilot_enabled_from(Some("0")));
    assert!(!lobby_autopilot_enabled_from(Some("true")));
    assert!(!lobby_autopilot_enabled_from(Some("yes")));
    assert!(!lobby_autopilot_enabled_from(Some("11")));
}

#[test]
fn test_lobby_autopilot_disabled_is_noop() {
    test_helpers::init_test_tracing();
    let mut app = fresh_app();
    // Default `LobbyAutopilotState::enabled == false`; handshake already
    // landed so the gate alone is what blocks the emit.
    app.world_mut().resource_mut::<LobbyViewState>().local_player_id = Some(PlayerId(7));

    let _ = app.world_mut().run_system_once(
        |mut state: ResMut<LobbyAutopilotState>,
         mut lobby: ResMut<LobbyViewState>,
         mut input: ResMut<LobbyInputState>,
         mut writer: MessageWriter<LobbyCommand>| {
            lobby_autopilot_step(&mut state, &mut lobby, &mut input, &mut writer);
        },
    );

    assert!(
        drain_commands(app.world_mut()).is_empty(),
        "autopilot must be inert when LobbyAutopilotState::enabled == false"
    );
}

#[test]
fn test_lobby_autopilot_stage_one_emits_create_bot_room_after_handshake() {
    test_helpers::init_test_tracing();
    let mut app = fresh_app();
    app.world_mut().resource_mut::<LobbyAutopilotState>().enabled = true;
    app.world_mut().resource_mut::<LobbyViewState>().local_player_id = Some(PlayerId(3));
    // Pre-condition: no session yet (the whole point of CreateBotRoom).
    assert!(app.world().resource::<LobbyViewState>().session_id.is_none());

    let _ = app.world_mut().run_system_once(
        |mut state: ResMut<LobbyAutopilotState>,
         mut lobby: ResMut<LobbyViewState>,
         mut input: ResMut<LobbyInputState>,
         mut writer: MessageWriter<LobbyCommand>| {
            lobby_autopilot_step(&mut state, &mut lobby, &mut input, &mut writer);
        },
    );

    let commands = drain_commands(app.world_mut());
    assert_eq!(
        commands,
        vec![LobbyCommand::CreateBotRoom {
            mode: GameMode::OneVOne,
            bot_kind: BotKind::Default,
        }],
        "stage 1 must emit exactly one CreateBotRoom"
    );
    let state = *app.world().resource::<LobbyAutopilotState>();
    assert!(state.create_bot_room_sent, "stage 1 latch must trip");
    assert!(!state.confirm_class_sent, "stage 2 latch must NOT trip yet");
    let lobby = app.world().resource::<LobbyViewState>();
    assert!(
        lobby.status.contains("Autopilot"),
        "status must announce autopilot activity; got {}",
        lobby.status
    );
}

#[test]
fn test_lobby_autopilot_stage_one_does_not_emit_before_handshake() {
    test_helpers::init_test_tracing();
    let mut app = fresh_app();
    app.world_mut().resource_mut::<LobbyAutopilotState>().enabled = true;
    // local_player_id stays None — handshake has NOT landed.

    let _ = app.world_mut().run_system_once(
        |mut state: ResMut<LobbyAutopilotState>,
         mut lobby: ResMut<LobbyViewState>,
         mut input: ResMut<LobbyInputState>,
         mut writer: MessageWriter<LobbyCommand>| {
            lobby_autopilot_step(&mut state, &mut lobby, &mut input, &mut writer);
        },
    );

    assert!(
        drain_commands(app.world_mut()).is_empty(),
        "autopilot must wait for the handshake (local_player_id) before \
         emitting CreateBotRoom"
    );
    assert!(
        !app.world().resource::<LobbyAutopilotState>().create_bot_room_sent,
        "latch must stay armed until the message actually fires"
    );
}

#[test]
fn test_lobby_autopilot_create_bot_room_is_single_shot() {
    test_helpers::init_test_tracing();
    let mut app = fresh_app();
    app.world_mut().resource_mut::<LobbyAutopilotState>().enabled = true;
    app.world_mut().resource_mut::<LobbyViewState>().local_player_id = Some(PlayerId(5));

    // First tick — emits.
    let _ = app.world_mut().run_system_once(
        |mut state: ResMut<LobbyAutopilotState>,
         mut lobby: ResMut<LobbyViewState>,
         mut input: ResMut<LobbyInputState>,
         mut writer: MessageWriter<LobbyCommand>| {
            lobby_autopilot_step(&mut state, &mut lobby, &mut input, &mut writer);
        },
    );
    let first = drain_commands(app.world_mut());
    assert_eq!(first.len(), 1, "first call emits exactly once");

    // Second tick before the server's S2CRoomCreated lands — must NOT
    // re-emit, otherwise the server will reject with AlreadyInSession on
    // the second send.
    let _ = app.world_mut().run_system_once(
        |mut state: ResMut<LobbyAutopilotState>,
         mut lobby: ResMut<LobbyViewState>,
         mut input: ResMut<LobbyInputState>,
         mut writer: MessageWriter<LobbyCommand>| {
            lobby_autopilot_step(&mut state, &mut lobby, &mut input, &mut writer);
        },
    );
    assert!(
        drain_commands(app.world_mut()).is_empty(),
        "second tick before session_id lands must be a no-op (latch)"
    );
}

#[test]
fn test_lobby_autopilot_stage_two_emits_confirm_class_after_room_created() {
    test_helpers::init_test_tracing();
    let mut app = fresh_app();
    {
        let mut state = app.world_mut().resource_mut::<LobbyAutopilotState>();
        state.enabled = true;
        state.create_bot_room_sent = true;
    }
    {
        let mut lobby = app.world_mut().resource_mut::<LobbyViewState>();
        lobby.local_player_id = Some(PlayerId(2));
        lobby.session_id = Some("SESSION-XYZ".to_string());
        lobby.selected_class = ClassId::Iop;
    }
    {
        let mut input = app.world_mut().resource_mut::<LobbyInputState>();
        input.selected_class = ClassId::Iop;
    }

    let _ = app.world_mut().run_system_once(
        |mut state: ResMut<LobbyAutopilotState>,
         mut lobby: ResMut<LobbyViewState>,
         mut input: ResMut<LobbyInputState>,
         mut writer: MessageWriter<LobbyCommand>| {
            lobby_autopilot_step(&mut state, &mut lobby, &mut input, &mut writer);
        },
    );

    let commands = drain_commands(app.world_mut());
    assert_eq!(
        commands,
        vec![LobbyCommand::ConfirmClass {
            class_id: ClassId::Iop,
        }],
        "stage 2 must emit exactly one ConfirmClass with the selected class"
    );
    let state = *app.world().resource::<LobbyAutopilotState>();
    assert!(state.confirm_class_sent, "stage 2 latch must trip");
    let input = app.world().resource::<LobbyInputState>();
    assert!(
        input.class_confirm_in_flight,
        "stage 2 must set the existing class_confirm_in_flight flag, \
         mirroring request_confirm_class so the human-click path stays \
         coherent"
    );
}

#[test]
fn test_lobby_autopilot_stage_two_skipped_when_class_already_locked() {
    test_helpers::init_test_tracing();
    let mut app = fresh_app();
    {
        let mut state = app.world_mut().resource_mut::<LobbyAutopilotState>();
        state.enabled = true;
        state.create_bot_room_sent = true;
    }
    {
        let mut lobby = app.world_mut().resource_mut::<LobbyViewState>();
        lobby.local_player_id = Some(PlayerId(1));
        lobby.session_id = Some("SESSION-LOCKED".to_string());
        lobby.locked_class = Some(ClassId::Sacrier);
    }

    let _ = app.world_mut().run_system_once(
        |mut state: ResMut<LobbyAutopilotState>,
         mut lobby: ResMut<LobbyViewState>,
         mut input: ResMut<LobbyInputState>,
         mut writer: MessageWriter<LobbyCommand>| {
            lobby_autopilot_step(&mut state, &mut lobby, &mut input, &mut writer);
        },
    );

    assert!(
        drain_commands(app.world_mut()).is_empty(),
        "stage 2 must not re-confirm a class the server already locked"
    );
    assert!(
        !app.world().resource::<LobbyAutopilotState>().confirm_class_sent,
        "latch must remain unset — the work was already done by the server"
    );
}
