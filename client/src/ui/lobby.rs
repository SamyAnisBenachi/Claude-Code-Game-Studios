use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use lightyear::prelude::{MessageReceiver, MessageSender};
use shared::card::ClassId;
use shared::protocol::{
    C2SConfirmClass, C2SCreateRoom, C2SJoinRoom, C2SSelectClass, GameMode, ReliableChannel,
    S2CClassLocked, S2CClassesRevealed, S2CConfirmClassRejected, S2CCreateRoomRejected,
    S2CHandshake, S2CHandshakeRejected, S2CJoinAck, S2CJoinRejected, S2CRoomCreated,
    S2CSlotUpdated, SessionSlot,
};
use shared::session::PlayerId;

use crate::asset_wiring::{
    lobby_portrait_asset, LOBBY_PLAYER_SLOT_PANEL_ASSET, LOBBY_ROOM_CODE_CHIP_ASSET,
};
use crate::state::{apply_handshake_message, ClientSessionIdentity, ClientState};

pub struct LobbyUiPlugin;

const LOBBY_PANEL_WIDTH: f32 = 420.0;
const ROOM_CODE_MAX: usize = 8;
const LOBBY_BUTTON_HEIGHT: f32 = 30.0;

impl Plugin for LobbyUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClientSessionIdentity>()
            .init_resource::<LobbyViewState>()
            .init_resource::<LobbyInputState>()
            .add_message::<KeyboardInput>()
            .add_message::<LobbyCommand>()
            .add_message::<PlayerTeamMapUpdated>()
            .add_systems(OnEnter(ClientState::Lobby), spawn_lobby_ui_system)
            .add_systems(OnExit(ClientState::Lobby), despawn_lobby_ui_system)
            .add_systems(
                OnEnter(ClientState::InSession),
                broadcast_player_team_map_on_session_enter_system,
            )
            .add_systems(
                Update,
                (
                    drain_lobby_s2c_system,
                    lobby_keyboard_input_system,
                    lobby_button_interaction_system,
                    send_lobby_commands_system,
                    refresh_lobby_ui_system,
                )
                    .chain()
                    .run_if(in_state(ClientState::Lobby)),
            );
    }
}

/// On entering an in-session client state, re-emit the current lobby slot map
/// so consumers like board rendering pick up the team assignments even if they
/// only register their `MessageReader` while the InSession schedule is active.
pub fn broadcast_player_team_map_on_session_enter_system(
    lobby: Res<LobbyViewState>,
    mut writer: MessageWriter<PlayerTeamMapUpdated>,
) {
    writer.write(PlayerTeamMapUpdated {
        slots: lobby.slots.clone(),
    });
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct LobbyViewState {
    pub local_player_id: Option<PlayerId>,
    pub session_id: Option<String>,
    pub room_code: Option<String>,
    pub mode: GameMode,
    pub slots: Vec<SessionSlot>,
    pub selected_class: ClassId,
    pub locked_class: Option<ClassId>,
    pub revealed_classes: Vec<(PlayerId, ClassId)>,
    pub status: String,
}

impl Default for LobbyViewState {
    fn default() -> Self {
        Self {
            local_player_id: None,
            session_id: None,
            room_code: None,
            mode: GameMode::OneVOne,
            slots: Vec::new(),
            selected_class: ClassId::Iop,
            locked_class: None,
            revealed_classes: Vec::new(),
            status: "Connecting".to_string(),
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct LobbyInputState {
    pub join_room_code: String,
    pub requested_slot: u8,
    pub selected_class: ClassId,
    pub room_code_focused: bool,
    pub room_code_selected: bool,
    pub create_in_flight: bool,
    pub join_in_flight: bool,
    pub class_confirm_in_flight: bool,
}

impl Default for LobbyInputState {
    fn default() -> Self {
        Self {
            join_room_code: normalize_room_code_text(
                &std::env::var("JOIN_ROOM_CODE").unwrap_or_default(),
            ),
            requested_slot: 1,
            selected_class: ClassId::Iop,
            room_code_focused: false,
            room_code_selected: false,
            create_in_flight: false,
            join_in_flight: false,
            class_confirm_in_flight: false,
        }
    }
}

/// Broadcast whenever the lobby slot map changes (room created, joined, or
/// slot updated). Decouples downstream consumers (e.g. board rendering's
/// `PlayerTeamMap`) from a direct `LobbyViewState` resource read across module
/// boundaries.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct PlayerTeamMapUpdated {
    pub slots: Vec<SessionSlot>,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum LobbyCommand {
    CreateRoom,
    JoinRoom {
        room_code: String,
        requested_slot: u8,
    },
    SelectClass {
        class_id: ClassId,
    },
    ConfirmClass {
        class_id: ClassId,
    },
}

#[derive(Component)]
struct LobbyRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobbyCamera;

#[derive(Component)]
struct LobbyStatusText;

#[derive(Component)]
pub struct LobbyRoomCodeField;

#[derive(Component)]
pub struct LobbyCreateRoomButton;

#[derive(Component)]
pub struct LobbyJoinRoomButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobbyRequestedSlotButton {
    pub slot: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobbyClassButton {
    pub class_id: ClassId,
}

#[derive(Component)]
pub struct LobbyConfirmClassButton;

/// Background portrait image for a class selection card in the lobby class picker.
/// One entity per `ClassId` variant (7 total). The `ImageNode` is the portrait image;
/// selection state is conveyed by a separate overlay, not by swapping this image.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobbyClassPortrait {
    pub class_id: ClassId,
}

/// Background panel image for the local player's slot in the lobby.
#[derive(Component)]
pub struct LobbyOwnSlotPanel;

/// Background panel image for the opponent's slot in the lobby.
#[derive(Component)]
pub struct LobbyOpponentSlotPanel;

/// Background image chip that frames the room code display in the lobby.
#[derive(Component)]
pub struct LobbyRoomCodeChip;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum LobbyDynamicText {
    RoomCode,
    Slot(u8),
    Class(ClassId),
    Create,
    Join,
    Confirm,
}

pub fn drain_lobby_s2c_system(
    mut identity: ResMut<ClientSessionIdentity>,
    mut lobby: ResMut<LobbyViewState>,
    mut input: ResMut<LobbyInputState>,
    mut team_map_writer: MessageWriter<PlayerTeamMapUpdated>,
    mut handshakes: Query<&mut MessageReceiver<S2CHandshake>>,
    mut handshake_rejections: Query<&mut MessageReceiver<S2CHandshakeRejected>>,
    mut created: Query<&mut MessageReceiver<S2CRoomCreated>>,
    mut create_rejected: Query<&mut MessageReceiver<S2CCreateRoomRejected>>,
    mut joined: Query<&mut MessageReceiver<S2CJoinAck>>,
    mut join_rejected: Query<&mut MessageReceiver<S2CJoinRejected>>,
    mut slot_updates: Query<&mut MessageReceiver<S2CSlotUpdated>>,
    mut class_locked: Query<&mut MessageReceiver<S2CClassLocked>>,
    mut classes_revealed: Query<&mut MessageReceiver<S2CClassesRevealed>>,
    mut confirm_rejected: Query<&mut MessageReceiver<S2CConfirmClassRejected>>,
) {
    for mut receiver in &mut handshakes {
        for message in receiver.receive() {
            apply_lobby_handshake(&mut lobby, &mut identity, &message);
        }
    }

    for mut receiver in &mut handshake_rejections {
        for message in receiver.receive() {
            lobby.status = format!(
                "Handshake rejected: server {} client {}",
                message.server_version, message.client_version
            );
        }
    }

    for mut receiver in &mut created {
        for message in receiver.receive() {
            apply_room_created(&mut lobby, &message);
            team_map_writer.write(PlayerTeamMapUpdated {
                slots: lobby.slots.clone(),
            });
            input.create_in_flight = false;
        }
    }

    for mut receiver in &mut create_rejected {
        for message in receiver.receive() {
            lobby.status = format!("Create rejected: {:?}", message.reason);
            input.create_in_flight = false;
        }
    }

    for mut receiver in &mut joined {
        for message in receiver.receive() {
            apply_join_ack(&mut lobby, &message);
            team_map_writer.write(PlayerTeamMapUpdated {
                slots: lobby.slots.clone(),
            });
            input.join_in_flight = false;
        }
    }

    for mut receiver in &mut join_rejected {
        for message in receiver.receive() {
            lobby.status = format!("Join rejected: {:?}", message.reason);
            input.join_in_flight = false;
        }
    }

    for mut receiver in &mut slot_updates {
        for message in receiver.receive() {
            apply_slot_update(&mut lobby, &message);
            team_map_writer.write(PlayerTeamMapUpdated {
                slots: lobby.slots.clone(),
            });
        }
    }

    for mut receiver in &mut class_locked {
        for message in receiver.receive() {
            apply_class_locked(&mut lobby, &message);
            input.class_confirm_in_flight = false;
        }
    }

    for mut receiver in &mut classes_revealed {
        for message in receiver.receive() {
            apply_classes_revealed(&mut lobby, &message);
        }
    }

    for mut receiver in &mut confirm_rejected {
        for message in receiver.receive() {
            lobby.status = format!("Class confirm rejected: {:?}", message.reason);
            input.class_confirm_in_flight = false;
        }
    }
}

pub fn apply_lobby_handshake(
    lobby: &mut LobbyViewState,
    identity: &mut ClientSessionIdentity,
    message: &S2CHandshake,
) {
    apply_handshake_message(message, identity);
    lobby.local_player_id = Some(message.player_id);
    lobby.status = format!("Connected as player {}", message.player_id.0);
}

pub fn apply_room_created(lobby: &mut LobbyViewState, message: &S2CRoomCreated) {
    lobby.session_id = Some(message.session_id.clone());
    lobby.room_code = Some(message.room_code.clone());
    lobby.mode = message.mode;
    lobby.slots = message.slots.clone();
    lobby.status = format!("Room {} created", message.room_code);
}

pub fn apply_join_ack(lobby: &mut LobbyViewState, message: &S2CJoinAck) {
    lobby.session_id = Some(message.session_id.clone());
    lobby.mode = message.mode;
    lobby.slots = message.slots.clone();
    lobby.status = "Joined room".to_string();
}

pub fn apply_slot_update(lobby: &mut LobbyViewState, message: &S2CSlotUpdated) {
    lobby.slots = message.slots.clone();
    lobby.status = "Lobby slots updated".to_string();
}

pub fn apply_class_locked(lobby: &mut LobbyViewState, message: &S2CClassLocked) {
    lobby.locked_class = Some(message.class_id);
    lobby.status = format!("Class locked: {:?}", message.class_id);
}

pub fn apply_classes_revealed(lobby: &mut LobbyViewState, message: &S2CClassesRevealed) {
    lobby.revealed_classes = message.player_class_map.clone();
    lobby.status = "All classes confirmed".to_string();
}

fn lobby_keyboard_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut keyboard_input: MessageReader<KeyboardInput>,
    mut input: ResMut<LobbyInputState>,
    mut lobby: ResMut<LobbyViewState>,
    mut commands: MessageWriter<LobbyCommand>,
) {
    let typed_any = if input.room_code_focused {
        append_room_code_text_events(&mut keyboard_input, &mut input)
    } else {
        for _ in keyboard_input.read() {}
        false
    };

    if input.room_code_focused {
        if !typed_any {
            append_room_code_keys(&keys, &mut input);
        }

        if keys.just_pressed(KeyCode::Backspace) {
            if input.room_code_selected {
                input.join_room_code.clear();
                input.room_code_selected = false;
            } else {
                input.join_room_code.pop();
            }
        }

        if keys.just_pressed(KeyCode::Escape) {
            input.room_code_focused = false;
            input.room_code_selected = false;
        }

        if keys.just_pressed(KeyCode::Enter) {
            input.room_code_focused = false;
            input.room_code_selected = false;
            request_join_room(&mut input, &mut lobby, &mut commands);
        }

        return;
    }

    if let Some(slot) = requested_slot_from_keys(&keys) {
        input.requested_slot = slot;
    }

    if let Some(class_id) = selected_class_from_keys(&keys) {
        request_select_class(class_id, &mut input, &mut lobby, &mut commands);
    }

    if keys.just_pressed(KeyCode::KeyC) {
        request_create_room(&mut input, &mut lobby, &mut commands);
    }

    if keys.just_pressed(KeyCode::KeyJ) {
        request_join_room(&mut input, &mut lobby, &mut commands);
    }

    if keys.just_pressed(KeyCode::Enter) {
        request_confirm_class(&mut input, &mut lobby, &mut commands);
    }
}

fn lobby_button_interaction_system(
    mut interactions: Query<
        (
            &Interaction,
            Option<&LobbyRoomCodeField>,
            Option<&LobbyCreateRoomButton>,
            Option<&LobbyJoinRoomButton>,
            Option<&LobbyRequestedSlotButton>,
            Option<&LobbyClassButton>,
            Option<&LobbyConfirmClassButton>,
        ),
        Changed<Interaction>,
    >,
    mut input: ResMut<LobbyInputState>,
    mut lobby: ResMut<LobbyViewState>,
    mut commands: MessageWriter<LobbyCommand>,
) {
    for (interaction, room_code, create, join, slot, class, confirm) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if room_code.is_some() {
            input.room_code_focused = true;
            input.room_code_selected = !input.join_room_code.is_empty();
            continue;
        }

        input.room_code_focused = false;
        input.room_code_selected = false;

        if create.is_some() {
            request_create_room(&mut input, &mut lobby, &mut commands);
        } else if join.is_some() {
            request_join_room(&mut input, &mut lobby, &mut commands);
        } else if let Some(slot) = slot {
            input.requested_slot = slot.slot;
        } else if let Some(class) = class {
            request_select_class(class.class_id, &mut input, &mut lobby, &mut commands);
        } else if confirm.is_some() {
            request_confirm_class(&mut input, &mut lobby, &mut commands);
        }
    }
}

fn send_lobby_commands_system(
    mut commands: MessageReader<LobbyCommand>,
    mut create_room: Query<&mut MessageSender<C2SCreateRoom>>,
    mut join_room: Query<&mut MessageSender<C2SJoinRoom>>,
    mut select_class: Query<&mut MessageSender<C2SSelectClass>>,
    mut confirm_class: Query<&mut MessageSender<C2SConfirmClass>>,
) {
    for command in commands.read() {
        match command {
            LobbyCommand::CreateRoom => {
                let Some(mut sender) = create_room.iter_mut().next() else {
                    warn!(
                        "C2S send DROPPED: type=C2SCreateRoom, handler=send_lobby_commands_system, reason=no_sender_entity"
                    );
                    continue;
                };
                sender.send::<ReliableChannel>(C2SCreateRoom {
                    mode: GameMode::OneVOne,
                });
            }
            LobbyCommand::JoinRoom {
                room_code,
                requested_slot,
            } => {
                let Some(mut sender) = join_room.iter_mut().next() else {
                    warn!(
                        room_code = %room_code,
                        "C2S send DROPPED: type=C2SJoinRoom, handler=send_lobby_commands_system, reason=no_sender_entity"
                    );
                    continue;
                };
                sender.send::<ReliableChannel>(C2SJoinRoom {
                    room_code: room_code.clone(),
                    requested_slot: *requested_slot,
                });
            }
            LobbyCommand::SelectClass { class_id } => {
                let Some(mut sender) = select_class.iter_mut().next() else {
                    warn!(
                        class_id = ?class_id,
                        "C2S send DROPPED: type=C2SSelectClass, handler=send_lobby_commands_system, reason=no_sender_entity"
                    );
                    continue;
                };
                sender.send::<ReliableChannel>(C2SSelectClass {
                    class_id: *class_id,
                });
            }
            LobbyCommand::ConfirmClass { class_id } => {
                let Some(mut sender) = confirm_class.iter_mut().next() else {
                    warn!(
                        class_id = ?class_id,
                        "C2S send DROPPED: type=C2SConfirmClass, handler=send_lobby_commands_system, reason=no_sender_entity"
                    );
                    continue;
                };
                sender.send::<ReliableChannel>(C2SConfirmClass {
                    class_id: *class_id,
                });
            }
        }
    }
}

fn append_room_code_text_events(
    keyboard_input: &mut MessageReader<KeyboardInput>,
    input: &mut LobbyInputState,
) -> bool {
    let mut appended = false;

    for event in keyboard_input.read() {
        if !event.state.is_pressed() {
            continue;
        }

        let Some(text) = event.text.as_ref() else {
            continue;
        };

        for value in text.chars() {
            if let Some(value) = normalize_room_code_char(value) {
                appended |= push_room_code_char(input, value);
            }
        }
    }

    appended
}

fn append_room_code_keys(keys: &ButtonInput<KeyCode>, input: &mut LobbyInputState) -> bool {
    let mut appended = false;
    for (key, value) in room_code_key_map() {
        if input.join_room_code.len() >= ROOM_CODE_MAX && !input.room_code_selected {
            break;
        }

        if keys.just_pressed(key) {
            appended |= push_room_code_char(input, value);
        }
    }

    appended
}

fn push_room_code_char(input: &mut LobbyInputState, value: char) -> bool {
    if input.room_code_selected {
        input.join_room_code.clear();
        input.room_code_selected = false;
    }

    if input.join_room_code.len() >= ROOM_CODE_MAX {
        return false;
    }

    input.join_room_code.push(value);
    true
}

pub fn normalize_room_code_text(raw: &str) -> String {
    raw.chars()
        .filter_map(normalize_room_code_char)
        .take(ROOM_CODE_MAX)
        .collect()
}

fn normalize_room_code_char(value: char) -> Option<char> {
    value
        .is_ascii_alphanumeric()
        .then(|| value.to_ascii_uppercase())
}

fn requested_slot_from_keys(keys: &ButtonInput<KeyCode>) -> Option<u8> {
    if keys.just_pressed(KeyCode::Digit0) {
        Some(0)
    } else if keys.just_pressed(KeyCode::Digit1) {
        Some(1)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(2)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(3)
    } else {
        None
    }
}

fn room_code_key_map() -> [(KeyCode, char); 36] {
    [
        (KeyCode::Digit0, '0'),
        (KeyCode::Digit1, '1'),
        (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'),
        (KeyCode::Digit4, '4'),
        (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'),
        (KeyCode::Digit7, '7'),
        (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
        (KeyCode::KeyA, 'A'),
        (KeyCode::KeyB, 'B'),
        (KeyCode::KeyC, 'C'),
        (KeyCode::KeyD, 'D'),
        (KeyCode::KeyE, 'E'),
        (KeyCode::KeyF, 'F'),
        (KeyCode::KeyG, 'G'),
        (KeyCode::KeyH, 'H'),
        (KeyCode::KeyI, 'I'),
        (KeyCode::KeyJ, 'J'),
        (KeyCode::KeyK, 'K'),
        (KeyCode::KeyL, 'L'),
        (KeyCode::KeyM, 'M'),
        (KeyCode::KeyN, 'N'),
        (KeyCode::KeyO, 'O'),
        (KeyCode::KeyP, 'P'),
        (KeyCode::KeyQ, 'Q'),
        (KeyCode::KeyR, 'R'),
        (KeyCode::KeyS, 'S'),
        (KeyCode::KeyT, 'T'),
        (KeyCode::KeyU, 'U'),
        (KeyCode::KeyV, 'V'),
        (KeyCode::KeyW, 'W'),
        (KeyCode::KeyX, 'X'),
        (KeyCode::KeyY, 'Y'),
        (KeyCode::KeyZ, 'Z'),
    ]
}

fn selected_class_from_keys(keys: &ButtonInput<KeyCode>) -> Option<ClassId> {
    if keys.just_pressed(KeyCode::KeyI) {
        Some(ClassId::Iop)
    } else if keys.just_pressed(KeyCode::KeyR) {
        Some(ClassId::Cra)
    } else if keys.just_pressed(KeyCode::KeyS) {
        Some(ClassId::Sacrier)
    } else if keys.just_pressed(KeyCode::KeyX) {
        Some(ClassId::Xelor)
    } else if keys.just_pressed(KeyCode::KeyE) {
        Some(ClassId::Ecaflip)
    } else if keys.just_pressed(KeyCode::KeyA) {
        Some(ClassId::Sadida)
    } else {
        None
    }
}

fn request_create_room(
    input: &mut LobbyInputState,
    lobby: &mut LobbyViewState,
    commands: &mut MessageWriter<LobbyCommand>,
) {
    if input.create_in_flight {
        lobby.status = "Create already pending".to_string();
        return;
    }

    input.create_in_flight = true;
    lobby.status = "Creating room".to_string();
    commands.write(LobbyCommand::CreateRoom);
}

fn request_join_room(
    input: &mut LobbyInputState,
    lobby: &mut LobbyViewState,
    commands: &mut MessageWriter<LobbyCommand>,
) {
    if input.join_in_flight {
        lobby.status = "Join already pending".to_string();
        return;
    }

    let room_code = normalize_room_code_text(&input.join_room_code);
    input.join_room_code = room_code.clone();
    if room_code.is_empty() {
        lobby.status = "Enter a room code before joining".to_string();
        return;
    }

    input.join_in_flight = true;
    lobby.status = format!("Joining {}", room_code);
    commands.write(LobbyCommand::JoinRoom {
        room_code,
        requested_slot: input.requested_slot,
    });
}

fn request_select_class(
    class_id: ClassId,
    input: &mut LobbyInputState,
    lobby: &mut LobbyViewState,
    commands: &mut MessageWriter<LobbyCommand>,
) {
    input.selected_class = class_id;
    lobby.selected_class = class_id;
    lobby.status = format!("Previewing {:?}", class_id);
    commands.write(LobbyCommand::SelectClass { class_id });
}

fn request_confirm_class(
    input: &mut LobbyInputState,
    lobby: &mut LobbyViewState,
    commands: &mut MessageWriter<LobbyCommand>,
) {
    if input.class_confirm_in_flight {
        lobby.status = "Class confirm already pending".to_string();
        return;
    }

    input.class_confirm_in_flight = true;
    lobby.status = format!("Confirming {:?}", input.selected_class);
    commands.write(LobbyCommand::ConfirmClass {
        class_id: input.selected_class,
    });
}

fn spawn_lobby_ui_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    lobby: Res<LobbyViewState>,
    input: Res<LobbyInputState>,
    roots: Query<Entity, With<LobbyRoot>>,
    cameras: Query<Entity, With<LobbyCamera>>,
) {
    if cameras.is_empty() {
        commands.spawn((LobbyCamera, Name::new("Lobby Camera"), Camera2d));
    }

    if !roots.is_empty() {
        return;
    }

    commands
        .spawn((
            LobbyRoot,
            Name::new("Lobby UI Root"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(24.0),
                top: Val::Px(24.0),
                width: Val::Px(LOBBY_PANEL_WIDTH),
                max_width: Val::Percent(92.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.07, 0.09, 0.12, 0.92)),
        ))
        .with_children(|parent| {
            parent.spawn((
                LobbyStatusText,
                Text::new(lobby_status_copy(&lobby, &input)),
                lobby_text_font(18.0),
                TextColor(Color::srgb(0.92, 0.95, 0.98)),
            ));

            parent.spawn((
                LobbyRoomCodeField,
                LobbyDynamicText::RoomCode,
                Button,
                Interaction::None,
                Text::new(lobby_dynamic_copy(
                    LobbyDynamicText::RoomCode,
                    &lobby,
                    &input,
                )),
                lobby_text_font(15.0),
                TextColor(Color::srgb(0.90, 0.96, 1.0)),
                lobby_button_node(Val::Percent(100.0)),
                BackgroundColor(Color::srgba(0.11, 0.15, 0.19, 0.95)),
                BorderColor::all(Color::srgb(0.33, 0.52, 0.68)),
            ));

            parent.spawn((lobby_row_node(),)).with_children(|row| {
                row.spawn((
                    LobbyCreateRoomButton,
                    LobbyDynamicText::Create,
                    Button,
                    Interaction::None,
                    Text::new(lobby_dynamic_copy(LobbyDynamicText::Create, &lobby, &input)),
                    lobby_text_font(14.0),
                    TextColor(Color::srgb(0.98, 0.93, 0.72)),
                    lobby_button_node(Val::Px(128.0)),
                    BackgroundColor(Color::srgba(0.17, 0.18, 0.14, 0.95)),
                    BorderColor::all(Color::srgb(0.65, 0.53, 0.24)),
                ));
                row.spawn((
                    LobbyJoinRoomButton,
                    LobbyDynamicText::Join,
                    Button,
                    Interaction::None,
                    Text::new(lobby_dynamic_copy(LobbyDynamicText::Join, &lobby, &input)),
                    lobby_text_font(14.0),
                    TextColor(Color::srgb(0.82, 0.95, 1.0)),
                    lobby_button_node(Val::Px(128.0)),
                    BackgroundColor(Color::srgba(0.11, 0.15, 0.20, 0.95)),
                    BorderColor::all(Color::srgb(0.28, 0.56, 0.72)),
                ));
            });

            parent.spawn((Text::new("Requested slot"), lobby_text_font(13.0)));
            parent.spawn((lobby_row_node(),)).with_children(|row| {
                for slot in 0..=3 {
                    row.spawn((
                        LobbyRequestedSlotButton { slot },
                        LobbyDynamicText::Slot(slot),
                        Button,
                        Interaction::None,
                        Text::new(lobby_dynamic_copy(
                            LobbyDynamicText::Slot(slot),
                            &lobby,
                            &input,
                        )),
                        lobby_text_font(13.0),
                        TextColor(Color::srgb(0.92, 0.95, 0.98)),
                        lobby_button_node(Val::Px(72.0)),
                        BackgroundColor(Color::srgba(0.10, 0.13, 0.17, 0.95)),
                        BorderColor::all(Color::srgb(0.30, 0.38, 0.48)),
                    ));
                }
            });

            parent.spawn((Text::new("Class"), lobby_text_font(13.0)));
            parent.spawn((lobby_wrap_row_node(),)).with_children(|row| {
                for class_id in lobby_class_options() {
                    row.spawn((
                        LobbyClassButton { class_id },
                        LobbyDynamicText::Class(class_id),
                        Button,
                        Interaction::None,
                        Text::new(lobby_dynamic_copy(
                            LobbyDynamicText::Class(class_id),
                            &lobby,
                            &input,
                        )),
                        lobby_text_font(13.0),
                        TextColor(Color::srgb(0.92, 0.95, 0.98)),
                        lobby_button_node(Val::Px(92.0)),
                        BackgroundColor(Color::srgba(0.10, 0.13, 0.17, 0.95)),
                        BorderColor::all(Color::srgb(0.30, 0.38, 0.48)),
                    ));
                }
            });

            parent.spawn((
                LobbyConfirmClassButton,
                LobbyDynamicText::Confirm,
                Button,
                Interaction::None,
                Text::new(lobby_dynamic_copy(
                    LobbyDynamicText::Confirm,
                    &lobby,
                    &input,
                )),
                lobby_text_font(14.0),
                TextColor(Color::srgb(0.98, 0.93, 0.72)),
                lobby_button_node(Val::Percent(100.0)),
                BackgroundColor(Color::srgba(0.17, 0.18, 0.14, 0.95)),
                BorderColor::all(Color::srgb(0.65, 0.53, 0.24)),
            ));

            // ── Class portraits (PAW-006-a) ───────────────────────────────────
            // One portrait ImageNode per ClassId variant (7 total, including Neutral).
            // The portrait image is always shown; selection state uses a separate overlay.
            parent.spawn((lobby_wrap_row_node(),)).with_children(|row| {
                for class_id in lobby_all_class_ids() {
                    row.spawn((
                        LobbyClassPortrait { class_id },
                        Name::new(format!("Lobby Portrait {:?}", class_id)),
                        Node {
                            width: Val::Px(64.0),
                            height: Val::Px(80.0),
                            ..default()
                        },
                        ImageNode::new(asset_server.load(lobby_portrait_asset(class_id))),
                    ));
                }
            });

            // ── Player slot panels (PAW-006-b) ────────────────────────────────
            parent.spawn((lobby_row_node(),)).with_children(|row| {
                row.spawn((
                    LobbyOwnSlotPanel,
                    Name::new("Lobby Own Slot Panel"),
                    Node {
                        width: Val::Px(160.0),
                        height: Val::Px(48.0),
                        ..default()
                    },
                    ImageNode::new(asset_server.load(LOBBY_PLAYER_SLOT_PANEL_ASSET)),
                ));
                row.spawn((
                    LobbyOpponentSlotPanel,
                    Name::new("Lobby Opponent Slot Panel"),
                    Node {
                        width: Val::Px(160.0),
                        height: Val::Px(48.0),
                        ..default()
                    },
                    ImageNode::new(asset_server.load(LOBBY_PLAYER_SLOT_PANEL_ASSET)),
                ));
            });

            // ── Room code chip (PAW-006-c) ─────────────────────────────────────
            // The chip is the background image; the room code text is a separate
            // Text child layered above it.
            parent
                .spawn((
                    LobbyRoomCodeChip,
                    Name::new("Lobby Room Code Chip"),
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(40.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ImageNode::new(asset_server.load(LOBBY_ROOM_CODE_CHIP_ASSET)),
                ))
                .with_children(|chip| {
                    let room_code = lobby.room_code.as_deref().unwrap_or("--------").to_string();
                    chip.spawn((
                        Text::new(room_code),
                        lobby_text_font(14.0),
                        TextColor(Color::srgb(0.92, 0.95, 0.98)),
                    ));
                });
        });
}

fn refresh_lobby_ui_system(
    lobby: Res<LobbyViewState>,
    input: Res<LobbyInputState>,
    mut texts: Query<&mut Text, With<LobbyStatusText>>,
    mut dynamic_texts: Query<(&LobbyDynamicText, &mut Text), Without<LobbyStatusText>>,
) {
    if !lobby.is_changed() && !input.is_changed() {
        return;
    }

    let Ok(mut text) = texts.single_mut() else {
        return;
    };

    text.0 = lobby_status_copy(&lobby, &input);

    for (role, mut text) in &mut dynamic_texts {
        text.0 = lobby_dynamic_copy(*role, &lobby, &input);
    }
}

pub fn lobby_status_copy(lobby: &LobbyViewState, input: &LobbyInputState) -> String {
    let room = lobby.room_code.as_deref().unwrap_or("----");
    let joined = lobby
        .slots
        .iter()
        .filter(|slot| slot.player_id.is_some())
        .count();
    let total = lobby.slots.len().max(1);
    let locked = lobby
        .locked_class
        .map(|class_id| format!("{:?}", class_id))
        .unwrap_or_else(|| "not confirmed".to_string());
    let code_input = if input.join_room_code.is_empty() {
        "empty".to_string()
    } else {
        input.join_room_code.clone()
    };

    format!(
        "Status: {}\nRoom: {}\nPlayers: {}/{}\nJoin: {} slot {}\nClass: {:?}\nConfirmed: {}",
        lobby.status,
        room,
        joined,
        total,
        code_input,
        input.requested_slot,
        input.selected_class,
        locked
    )
}

fn lobby_dynamic_copy(
    role: LobbyDynamicText,
    lobby: &LobbyViewState,
    input: &LobbyInputState,
) -> String {
    match role {
        LobbyDynamicText::RoomCode => {
            let code = if input.join_room_code.is_empty() {
                "--------".to_string()
            } else {
                input.join_room_code.clone()
            };
            let focus = if input.room_code_focused {
                if input.room_code_selected {
                    "selected"
                } else {
                    "typing"
                }
            } else {
                "idle"
            };
            let rendered_code = if input.room_code_selected {
                format!("[{code}]")
            } else if input.room_code_focused {
                format!("{code}|")
            } else {
                code
            };
            format!("Room code: {rendered_code} ({focus})")
        }
        LobbyDynamicText::Slot(slot) => {
            if input.requested_slot == slot {
                format!("Slot {slot} *")
            } else {
                format!("Slot {slot}")
            }
        }
        LobbyDynamicText::Class(class_id) => {
            if input.selected_class == class_id {
                format!("{class_id:?} *")
            } else {
                format!("{class_id:?}")
            }
        }
        LobbyDynamicText::Create if input.create_in_flight => "Creating...".to_string(),
        LobbyDynamicText::Create => "Create Room".to_string(),
        LobbyDynamicText::Join if input.join_in_flight => "Joining...".to_string(),
        LobbyDynamicText::Join if input.join_room_code.is_empty() => "Join Room".to_string(),
        LobbyDynamicText::Join => format!("Join {}", input.join_room_code),
        LobbyDynamicText::Confirm if input.class_confirm_in_flight => "Confirming...".to_string(),
        LobbyDynamicText::Confirm => {
            let locked = lobby
                .locked_class
                .map_or(false, |locked| locked == input.selected_class);
            if locked {
                format!("Confirmed {:?}", input.selected_class)
            } else {
                format!("Confirm {:?}", input.selected_class)
            }
        }
    }
}

fn lobby_class_options() -> [ClassId; 6] {
    [
        ClassId::Iop,
        ClassId::Cra,
        ClassId::Sacrier,
        ClassId::Xelor,
        ClassId::Ecaflip,
        ClassId::Sadida,
    ]
}

/// All 7 ClassId variants used for portrait display (includes Neutral).
fn lobby_all_class_ids() -> [ClassId; 7] {
    [
        ClassId::Iop,
        ClassId::Cra,
        ClassId::Sacrier,
        ClassId::Xelor,
        ClassId::Ecaflip,
        ClassId::Sadida,
        ClassId::Neutral,
    ]
}

fn despawn_lobby_ui_system(
    mut commands: Commands,
    roots: Query<Entity, With<LobbyRoot>>,
    cameras: Query<Entity, With<LobbyCamera>>,
) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }

    for entity in &cameras {
        commands.entity(entity).despawn();
    }
}

fn lobby_text_font(font_size: f32) -> TextFont {
    TextFont {
        font_size,
        ..default()
    }
}

fn lobby_row_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Px(LOBBY_BUTTON_HEIGHT),
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(8.0),
        align_items: AlignItems::Center,
        ..default()
    }
}

fn lobby_wrap_row_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        row_gap: Val::Px(6.0),
        column_gap: Val::Px(6.0),
        align_items: AlignItems::Center,
        ..default()
    }
}

fn lobby_button_node(width: Val) -> Node {
    Node {
        width,
        height: Val::Px(LOBBY_BUTTON_HEIGHT),
        border: UiRect::all(Val::Px(1.0)),
        padding: UiRect::horizontal(Val::Px(8.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}
