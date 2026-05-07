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

use crate::state::{apply_handshake_message, ClientSessionIdentity, ClientState};

pub struct LobbyUiPlugin;

const LOBBY_PANEL_WIDTH: f32 = 420.0;
const ROOM_CODE_MAX: usize = 8;

impl Plugin for LobbyUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ClientState>()
            .init_resource::<ClientSessionIdentity>()
            .init_resource::<LobbyViewState>()
            .init_resource::<LobbyInputState>()
            .add_message::<LobbyCommand>()
            .add_systems(
                Startup,
                spawn_lobby_ui_system.run_if(in_state(ClientState::Lobby)),
            )
            .add_systems(OnEnter(ClientState::Lobby), spawn_lobby_ui_system)
            .add_systems(OnExit(ClientState::Lobby), despawn_lobby_ui_system)
            .add_systems(
                Update,
                (
                    drain_lobby_s2c_system,
                    lobby_keyboard_input_system,
                    send_lobby_commands_system,
                    refresh_lobby_ui_system,
                )
                    .chain()
                    .run_if(in_state(ClientState::Lobby)),
            );
    }
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
}

impl Default for LobbyInputState {
    fn default() -> Self {
        Self {
            join_room_code: std::env::var("JOIN_ROOM_CODE").unwrap_or_default(),
            requested_slot: 1,
            selected_class: ClassId::Iop,
        }
    }
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

pub fn drain_lobby_s2c_system(
    mut identity: ResMut<ClientSessionIdentity>,
    mut lobby: ResMut<LobbyViewState>,
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
        }
    }

    for mut receiver in &mut create_rejected {
        for message in receiver.receive() {
            lobby.status = format!("Create rejected: {:?}", message.reason);
        }
    }

    for mut receiver in &mut joined {
        for message in receiver.receive() {
            apply_join_ack(&mut lobby, &message);
        }
    }

    for mut receiver in &mut join_rejected {
        for message in receiver.receive() {
            lobby.status = format!("Join rejected: {:?}", message.reason);
        }
    }

    for mut receiver in &mut slot_updates {
        for message in receiver.receive() {
            apply_slot_update(&mut lobby, &message);
        }
    }

    for mut receiver in &mut class_locked {
        for message in receiver.receive() {
            apply_class_locked(&mut lobby, &message);
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
    mut input: ResMut<LobbyInputState>,
    mut lobby: ResMut<LobbyViewState>,
    mut commands: MessageWriter<LobbyCommand>,
) {
    append_room_code_keys(&keys, &mut input.join_room_code);

    if keys.just_pressed(KeyCode::Backspace) {
        input.join_room_code.pop();
    }

    if keys.just_pressed(KeyCode::Digit0) {
        input.requested_slot = 0;
    } else if keys.just_pressed(KeyCode::Digit1) {
        input.requested_slot = 1;
    } else if keys.just_pressed(KeyCode::Digit2) {
        input.requested_slot = 2;
    } else if keys.just_pressed(KeyCode::Digit3) {
        input.requested_slot = 3;
    }

    if let Some(class_id) = selected_class_from_keys(&keys) {
        input.selected_class = class_id;
        lobby.selected_class = class_id;
        commands.write(LobbyCommand::SelectClass { class_id });
    }

    if keys.just_pressed(KeyCode::KeyC) {
        lobby.status = "Creating room".to_string();
        commands.write(LobbyCommand::CreateRoom);
    }

    if keys.just_pressed(KeyCode::KeyJ) && !input.join_room_code.is_empty() {
        lobby.status = format!("Joining {}", input.join_room_code);
        commands.write(LobbyCommand::JoinRoom {
            room_code: input.join_room_code.clone(),
            requested_slot: input.requested_slot,
        });
    }

    if keys.just_pressed(KeyCode::Enter) {
        lobby.status = format!("Confirming {:?}", input.selected_class);
        commands.write(LobbyCommand::ConfirmClass {
            class_id: input.selected_class,
        });
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
                if let Some(mut sender) = create_room.iter_mut().next() {
                    sender.send::<ReliableChannel>(C2SCreateRoom {
                        mode: GameMode::OneVOne,
                    });
                }
            }
            LobbyCommand::JoinRoom {
                room_code,
                requested_slot,
            } => {
                if let Some(mut sender) = join_room.iter_mut().next() {
                    sender.send::<ReliableChannel>(C2SJoinRoom {
                        room_code: room_code.clone(),
                        requested_slot: *requested_slot,
                    });
                }
            }
            LobbyCommand::SelectClass { class_id } => {
                if let Some(mut sender) = select_class.iter_mut().next() {
                    sender.send::<ReliableChannel>(C2SSelectClass {
                        class_id: *class_id,
                    });
                }
            }
            LobbyCommand::ConfirmClass { class_id } => {
                if let Some(mut sender) = confirm_class.iter_mut().next() {
                    sender.send::<ReliableChannel>(C2SConfirmClass {
                        class_id: *class_id,
                    });
                }
            }
        }
    }
}

fn append_room_code_keys(keys: &ButtonInput<KeyCode>, room_code: &mut String) {
    if room_code.len() >= ROOM_CODE_MAX {
        return;
    }

    for (key, value) in room_code_key_map() {
        if keys.just_pressed(key) {
            room_code.push(value);
        }
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

fn spawn_lobby_ui_system(
    mut commands: Commands,
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
        });
}

fn refresh_lobby_ui_system(
    lobby: Res<LobbyViewState>,
    input: Res<LobbyInputState>,
    mut texts: Query<&mut Text, With<LobbyStatusText>>,
) {
    if !lobby.is_changed() && !input.is_changed() {
        return;
    }

    let Ok(mut text) = texts.single_mut() else {
        return;
    };

    text.0 = lobby_status_copy(&lobby, &input);
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
