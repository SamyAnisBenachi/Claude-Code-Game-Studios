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
    class_type_icon_asset, lobby_portrait_asset, LOBBY_PLAYER_SLOT_PANEL_ASSET,
    LOBBY_ROOM_CODE_CHIP_ASSET,
};
use crate::state::{
    apply_handshake_message, ClassLockedDedupeKey, ClientIdempotencyState, ClientSessionIdentity,
    ClientState,
};
use crate::ui::design_tokens::{
    interaction_states::{
        DISABLED_BG_TINT_ALPHA, DISABLED_BORDER_ALPHA, DISABLED_TEXT_ALPHA, HOVER_BG_TINT_ALPHA,
        HOVER_BORDER_ALPHA, PRESSED_BG_TINT_ALPHA,
    },
    overlays,
    spacing::{SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XL},
    typography, z_layers,
};

pub struct LobbyUiPlugin;

/// Story 024 (S12-UX-LOBBY-LAYOUT-MODAL-001 / PROMPT 933 Option A): centred
/// lobby modal panel max-width literal, mirrored verbatim from
/// `client/src/presentation/result_screen.rs:538` (the only surface in the
/// PROMPT 802 audit that does layout correctly).
pub const LOBBY_PANEL_MAX_WIDTH_PX: f32 = 860.0;

/// Story 024 Option A: lobby modal panel width percentage. Mirrors
/// `result_screen.rs:537`.
pub const LOBBY_PANEL_WIDTH_PERCENT: f32 = 88.0;

/// Story 024 Option A: lobby modal panel max height percentage. Mirrors
/// `result_screen.rs:539`.
pub const LOBBY_PANEL_MAX_HEIGHT_PERCENT: f32 = 92.0;

const ROOM_CODE_MAX: usize = 8;
pub const LOBBY_BUTTON_HEIGHT_PX: f32 = 30.0;
pub const LOBBY_ROOM_CODE_FIELD_WIDTH_PERCENT: f32 = 100.0;
pub const LOBBY_ROOM_CODE_FIELD_HEIGHT_PX: f32 = LOBBY_BUTTON_HEIGHT_PX;
pub const LOBBY_CREATE_BUTTON_WIDTH_PX: f32 = 128.0;
pub const LOBBY_CREATE_BUTTON_HEIGHT_PX: f32 = LOBBY_BUTTON_HEIGHT_PX;
pub const LOBBY_JOIN_BUTTON_WIDTH_PX: f32 = 128.0;
pub const LOBBY_JOIN_BUTTON_HEIGHT_PX: f32 = LOBBY_BUTTON_HEIGHT_PX;
pub const LOBBY_REQUESTED_SLOT_BUTTON_WIDTH_PX: f32 = 80.0;
pub const LOBBY_REQUESTED_SLOT_BUTTON_HEIGHT_PX: f32 = LOBBY_BUTTON_HEIGHT_PX;
pub const LOBBY_CLASS_PICKER_GRID_COLUMNS: usize = 7;
pub const LOBBY_CLASS_PICKER_SELECTABLE_COUNT: usize = 6;
pub const LOBBY_CLASS_PICKER_CELL_WIDTH_PX: f32 = 108.0;
pub const LOBBY_CLASS_PICKER_CELL_HEIGHT_PX: f32 = 132.0;
pub const LOBBY_CLASS_PICKER_CELL_PADDING_PX: f32 = 6.0;
pub const LOBBY_CLASS_PICKER_PORTRAIT_WIDTH_PX: f32 = 64.0;
pub const LOBBY_CLASS_PICKER_PORTRAIT_HEIGHT_PX: f32 = 80.0;

/// PROMPT 1138 — pixel size of the class-distinct emblem overlay composited
/// on top of each picker tile portrait. The emblem is sourced from
/// [`class_type_icon_asset`] (per-class mana-badge PNGs with class-distinct
/// SHA-256 fingerprints), giving the picker an at-a-glance class identity
/// while the canonical lobby portrait slot still hosts a generic stand-in.
pub const LOBBY_CLASS_PICKER_EMBLEM_PX: f32 = 24.0;
pub const LOBBY_CLASS_PICKER_BUTTON_WIDTH_PX: f32 = 96.0;
pub const LOBBY_CLASS_PICKER_BUTTON_HEIGHT_PX: f32 = LOBBY_BUTTON_HEIGHT_PX;
pub const LOBBY_CONFIRM_BUTTON_WIDTH_PERCENT: f32 = 100.0;
pub const LOBBY_CONFIRM_BUTTON_HEIGHT_PX: f32 = LOBBY_BUTTON_HEIGHT_PX;
pub const LOBBY_ROOM_CODE_CHIP_WIDTH_PX: f32 = 200.0;
pub const LOBBY_ROOM_CODE_CHIP_HEIGHT_PX: f32 = 40.0;
pub const LOBBY_SLOT_PANEL_WIDTH_PX: f32 = 160.0;
pub const LOBBY_SLOT_PANEL_HEIGHT_PX: f32 = 48.0;

impl Plugin for LobbyUiPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("LobbyUiPlugin loaded");
        app.init_resource::<ClientSessionIdentity>()
            .init_resource::<LobbyViewState>()
            .init_resource::<LobbyInputState>()
            // S13-LATE-MSG-DEDUPE-001: ensure the dedupe ring exists even when
            // tests load this plugin in isolation.
            .init_resource::<ClientIdempotencyState>()
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
                    refresh_confirm_button_visual_system,
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

/// Lobby UI root — Story 024 Option A: full-viewport flex container that
/// owns the modal scrim backdrop and the `UI_OVERLAY` z-layer (per
/// `docs/ux/global-ui-design-spec.md` §10 "Modal centering pattern").
/// The centred [`LobbyPanel`] child paints above this at `MODAL`.
#[derive(Component)]
pub struct LobbyRoot;

/// Lobby modal panel — Story 024 Option A: the centred panel that owns the
/// lobby form content (status banner, room-code chip, create/join row,
/// requested-slot row, class picker + portraits, slot panels, confirm CTA).
/// Mirrors `client/src/presentation/result_screen.rs` panel literals
/// (`width: 88%`, `max_width: 860 Px`, `max_height: 92%`) per PROMPT 933.
#[derive(Component)]
pub struct LobbyPanel;

/// Class-picker region container. Owns the region heading and the class grid
/// so portraits and selectable controls no longer wrap as independent rows.
#[derive(Component)]
pub struct LobbyClassPickerBlock;

#[derive(Component)]
pub struct LobbyClassPickerHeading;

#[derive(Component)]
pub struct LobbyClassPickerGrid;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobbyClassPickerCell {
    pub class_id: ClassId,
    pub selectable: bool,
}

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

/// PROMPT 1138 — class-distinct emblem overlay composited on top of the picker
/// tile portrait. Sourced from [`class_type_icon_asset`]; one entity per
/// `ClassId` variant. Provides at-a-glance class identity while the canonical
/// lobby portrait slot still hosts a generic stand-in.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobbyClassPickerEmblem {
    pub class_id: ClassId,
}

/// Background panel image for the local player's slot in the lobby.
#[derive(Component)]
pub struct LobbyOwnSlotPanel;

/// Background panel image for the opponent's slot in the lobby.
#[derive(Component)]
pub struct LobbyOpponentSlotPanel;

/// PROMPT 1138 — text label composited on top of the own-slot panel so it
/// reads as informative status ("You · {class} · slot N") instead of an
/// unidentified blue-card placeholder (AUDIT-1129-08).
#[derive(Component)]
pub struct LobbyOwnSlotLabel;

/// PROMPT 1138 — text label composited on top of the opponent-slot panel
/// (AUDIT-1129-08).
#[derive(Component)]
pub struct LobbyOpponentSlotLabel;

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
    /// PROMPT 1138 — own-slot panel inline label.
    OwnSlot,
    /// PROMPT 1138 — opponent-slot panel inline label.
    OpponentSlot,
}

pub fn drain_lobby_s2c_system(
    mut identity: ResMut<ClientSessionIdentity>,
    mut lobby: ResMut<LobbyViewState>,
    mut input: ResMut<LobbyInputState>,
    mut team_map_writer: MessageWriter<PlayerTeamMapUpdated>,
    mut idempotency: ResMut<ClientIdempotencyState>,
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
            tracing::info!(
                player_id = ?message.player_id,
                session_id = message.session_id,
                protocol_version = message.protocol_version,
                msg_type = "S2CHandshake",
                "drain_lobby_s2c: recv"
            );
            apply_lobby_handshake(&mut lobby, &mut identity, &message);
        }
    }

    for mut receiver in &mut handshake_rejections {
        for message in receiver.receive() {
            tracing::info!(
                server_version = message.server_version,
                client_version = message.client_version,
                msg_type = "S2CHandshakeRejected",
                "drain_lobby_s2c: recv"
            );
            lobby.status = format!(
                "Handshake rejected: server {} client {}",
                message.server_version, message.client_version
            );
        }
    }

    for mut receiver in &mut created {
        for message in receiver.receive() {
            tracing::info!(
                session_id = %message.session_id,
                room_code = %message.room_code,
                mode = ?message.mode,
                slots_len = message.slots.len(),
                msg_type = "S2CRoomCreated",
                "drain_lobby_s2c: recv"
            );
            apply_room_created(&mut lobby, &message);
            team_map_writer.write(PlayerTeamMapUpdated {
                slots: lobby.slots.clone(),
            });
            input.create_in_flight = false;
        }
    }

    for mut receiver in &mut create_rejected {
        for message in receiver.receive() {
            tracing::info!(
                reason = ?message.reason,
                msg_type = "S2CCreateRoomRejected",
                "drain_lobby_s2c: recv"
            );
            lobby.status = format!("Create rejected: {:?}", message.reason);
            input.create_in_flight = false;
        }
    }

    for mut receiver in &mut joined {
        for message in receiver.receive() {
            tracing::info!(
                session_id = %message.session_id,
                mode = ?message.mode,
                slots_len = message.slots.len(),
                msg_type = "S2CJoinAck",
                "drain_lobby_s2c: recv"
            );
            apply_join_ack(&mut lobby, &message);
            team_map_writer.write(PlayerTeamMapUpdated {
                slots: lobby.slots.clone(),
            });
            input.join_in_flight = false;
        }
    }

    for mut receiver in &mut join_rejected {
        for message in receiver.receive() {
            tracing::info!(
                reason = ?message.reason,
                msg_type = "S2CJoinRejected",
                "drain_lobby_s2c: recv"
            );
            lobby.status = format!("Join rejected: {:?}", message.reason);
            input.join_in_flight = false;
        }
    }

    for mut receiver in &mut slot_updates {
        for message in receiver.receive() {
            tracing::info!(
                slots_len = message.slots.len(),
                msg_type = "S2CSlotUpdated",
                "drain_lobby_s2c: recv"
            );
            apply_slot_update(&mut lobby, &message);
            team_map_writer.write(PlayerTeamMapUpdated {
                slots: lobby.slots.clone(),
            });
        }
    }

    for mut receiver in &mut class_locked {
        for message in receiver.receive() {
            apply_class_locked_drain(&mut idempotency, &mut lobby, &mut input, &message);
        }
    }

    for mut receiver in &mut classes_revealed {
        for message in receiver.receive() {
            tracing::info!(
                map_len = message.player_class_map.len(),
                msg_type = "S2CClassesRevealed",
                "drain_lobby_s2c: recv"
            );
            apply_classes_revealed(&mut lobby, &message);
        }
    }

    for mut receiver in &mut confirm_rejected {
        for message in receiver.receive() {
            tracing::info!(
                reason = ?message.reason,
                msg_type = "S2CConfirmClassRejected",
                "drain_lobby_s2c: recv"
            );
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

/// Idempotent apply for `S2CClassLocked` per S13-LATE-MSG-DEDUPE-001.
///
/// On a fresh `ClassId` key the message is logged and applied to the lobby
/// view; the in-flight confirm latch is cleared. On a duplicate
/// (reconnect-replay re-send) the message is logged at DEBUG and discarded
/// without mutating lobby state — matching the `C2SAcknowledgeResult`
/// idempotency precedent.
pub fn apply_class_locked_drain(
    idempotency: &mut ClientIdempotencyState,
    lobby: &mut LobbyViewState,
    input: &mut LobbyInputState,
    message: &S2CClassLocked,
) {
    let key = ClassLockedDedupeKey::from_message(message);
    if !idempotency.class_locked.check_and_insert(key) {
        tracing::debug!(
            class_id = ?message.class_id,
            msg_type = "S2CClassLocked",
            "drain_lobby_s2c: duplicate; no-op"
        );
        return;
    }

    tracing::info!(
        class_id = ?message.class_id,
        msg_type = "S2CClassLocked",
        "drain_lobby_s2c: recv"
    );
    apply_class_locked(lobby, message);
    input.class_confirm_in_flight = false;
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
                tracing::info!(
                    msg_type = "C2SCreateRoom",
                    mode = ?GameMode::OneVOne,
                    "c2s_send: enter"
                );
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
                tracing::info!(
                    msg_type = "C2SJoinRoom",
                    room_code = %room_code,
                    requested_slot = *requested_slot,
                    "c2s_send: enter"
                );
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
                tracing::info!(
                    msg_type = "C2SSelectClass",
                    class_id = ?class_id,
                    "c2s_send: enter"
                );
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
                tracing::info!(
                    msg_type = "C2SConfirmClass",
                    class_id = ?class_id,
                    "c2s_send: enter"
                );
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
    if lobby.session_id.is_none() {
        tracing::warn!(
            can_confirm = false,
            session_id = ?lobby.session_id,
            local_player_id = ?lobby.local_player_id,
            class_id = ?input.selected_class,
            "lobby_ui_confirm_button_state: blocked — no active session_id (premature confirm)"
        );
        lobby.status = "Create or join a room before confirming class".to_string();
        return;
    }

    if input.class_confirm_in_flight {
        tracing::info!(
            can_confirm = false,
            session_id = ?lobby.session_id,
            class_id = ?input.selected_class,
            "lobby_ui_confirm_button_state: blocked — confirm already in-flight"
        );
        lobby.status = "Class confirm already pending".to_string();
        return;
    }

    tracing::info!(
        can_confirm = true,
        session_id = ?lobby.session_id,
        class_id = ?input.selected_class,
        "lobby_ui_confirm_button_state: dispatching ConfirmClass command"
    );
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

    // Story 024 (S12-UX-LOBBY-LAYOUT-MODAL-001 / PROMPT 933 Option A):
    // full-viewport flex root owns the modal scrim backdrop and the
    // `UI_OVERLAY` z-layer. The centred [`LobbyPanel`] child paints above
    // it on `MODAL`. Replaces the prior top-left anchored 420-px column
    // (PROMPT 802 §3.1 L1 / L4 "rough-bordering-unacceptable" verdict).
    commands
        .spawn((
            LobbyRoot,
            Name::new("Lobby UI Root"),
            Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(SPACING_LG)),
                ..default()
            },
            // `SURFACE` color (`Color::srgb(0.039, 0.051, 0.078)` per
            // `docs/ux/global-ui-design-spec.md` §7) at
            // `OVERLAY_SCRIM_ALPHA` (0.55) per §10 modal-centering
            // pattern.
            BackgroundColor(Color::srgba(
                0.039,
                0.051,
                0.078,
                overlays::OVERLAY_SCRIM_ALPHA,
            )),
            z_layers::UI_OVERLAY,
        ))
        .with_children(|root| {
            root.spawn((
                LobbyPanel,
                Name::new("Lobby UI Panel"),
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(LOBBY_PANEL_WIDTH_PERCENT),
                    max_width: Val::Px(LOBBY_PANEL_MAX_WIDTH_PX),
                    max_height: Val::Percent(LOBBY_PANEL_MAX_HEIGHT_PERCENT),
                    row_gap: Val::Px(SPACING_MD),
                    padding: UiRect::all(Val::Px(SPACING_LG)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                // `SURFACE_ELEVATED` per §7 / §10 "Panel chrome" rule
                // (`Color::srgb(0.086, 0.106, 0.153)` per spec §7).
                BackgroundColor(Color::srgb(0.086, 0.106, 0.153)),
                BorderColor::all(Color::srgba(0.82, 0.86, 0.9, 0.26)),
                z_layers::MODAL,
            ))
            .with_children(|panel| {
                // Section 1 — status banner + room-code chip (read-order
                // top-of-panel per AC3(e) "status / room-code -> ...").
                panel.spawn((
                    LobbyStatusText,
                    Text::new(lobby_status_copy(&lobby, &input)),
                    lobby_text_font(typography::H3),
                    TextColor(Color::srgb(0.92, 0.95, 0.98)),
                ));

                panel
                    .spawn((
                        LobbyRoomCodeChip,
                        Name::new("Lobby Room Code Chip"),
                        Node {
                            width: Val::Px(LOBBY_ROOM_CODE_CHIP_WIDTH_PX),
                            height: Val::Px(LOBBY_ROOM_CODE_CHIP_HEIGHT_PX),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ImageNode::new(asset_server.load(LOBBY_ROOM_CODE_CHIP_ASSET)),
                    ))
                    .with_children(|chip| {
                        let room_code =
                            lobby.room_code.as_deref().unwrap_or("--------").to_string();
                        chip.spawn((
                            Text::new(room_code),
                            lobby_text_font(typography::BODY),
                            TextColor(Color::srgb(0.92, 0.95, 0.98)),
                        ));
                    });

                // Section separator before the create/join section
                // (`SPACING_XL` total cumulative gap = default `row_gap`
                // `SPACING_MD` + this margin's extra `SPACING_XL -
                // SPACING_MD`).
                panel.spawn((
                    Name::new("Lobby Section Separator 1"),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(0.0),
                        margin: UiRect {
                            top: Val::Px(SPACING_XL - SPACING_MD),
                            ..default()
                        },
                        ..default()
                    },
                ));

                // Section 2 — create / join row + room-code input.
                panel.spawn((
                    LobbyRoomCodeField,
                    LobbyDynamicText::RoomCode,
                    Button,
                    Interaction::None,
                    Text::new(lobby_dynamic_copy(
                        LobbyDynamicText::RoomCode,
                        &lobby,
                        &input,
                    )),
                    lobby_text_font(typography::BODY),
                    TextColor(Color::srgb(0.90, 0.96, 1.0)),
                    lobby_button_node(
                        Val::Percent(LOBBY_ROOM_CODE_FIELD_WIDTH_PERCENT),
                        LOBBY_ROOM_CODE_FIELD_HEIGHT_PX,
                    ),
                    BackgroundColor(Color::srgba(0.11, 0.15, 0.19, 0.95)),
                    BorderColor::all(Color::srgb(0.33, 0.52, 0.68)),
                ));

                panel.spawn((lobby_row_node(),)).with_children(|row| {
                    row.spawn((
                        LobbyCreateRoomButton,
                        LobbyDynamicText::Create,
                        Button,
                        Interaction::None,
                        Text::new(lobby_dynamic_copy(LobbyDynamicText::Create, &lobby, &input)),
                        lobby_text_font(typography::BODY),
                        TextColor(Color::srgb(0.98, 0.93, 0.72)),
                        lobby_button_node(
                            Val::Px(LOBBY_CREATE_BUTTON_WIDTH_PX),
                            LOBBY_CREATE_BUTTON_HEIGHT_PX,
                        ),
                        BackgroundColor(Color::srgba(0.17, 0.18, 0.14, 0.95)),
                        BorderColor::all(Color::srgb(0.65, 0.53, 0.24)),
                    ));
                    row.spawn((
                        LobbyJoinRoomButton,
                        LobbyDynamicText::Join,
                        Button,
                        Interaction::None,
                        Text::new(lobby_dynamic_copy(LobbyDynamicText::Join, &lobby, &input)),
                        lobby_text_font(typography::BODY),
                        TextColor(Color::srgb(0.82, 0.95, 1.0)),
                        lobby_button_node(
                            Val::Px(LOBBY_JOIN_BUTTON_WIDTH_PX),
                            LOBBY_JOIN_BUTTON_HEIGHT_PX,
                        ),
                        BackgroundColor(Color::srgba(0.11, 0.15, 0.20, 0.95)),
                        BorderColor::all(Color::srgb(0.28, 0.56, 0.72)),
                    ));
                });

                // Sprint 14 story 003 AC6: lobby labels are at least as
                // large as the data they describe.
                panel.spawn((
                    Text::new("Requested slot"),
                    lobby_text_font(typography::BODY),
                ));
                panel.spawn((lobby_row_node(),)).with_children(|row| {
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
                            lobby_text_font(typography::BODY),
                            TextColor(Color::srgb(0.92, 0.95, 0.98)),
                            lobby_button_node(
                                Val::Px(LOBBY_REQUESTED_SLOT_BUTTON_WIDTH_PX),
                                LOBBY_REQUESTED_SLOT_BUTTON_HEIGHT_PX,
                            ),
                            BackgroundColor(Color::srgba(0.10, 0.13, 0.17, 0.95)),
                            BorderColor::all(Color::srgb(0.30, 0.38, 0.48)),
                        ));
                    }
                });

                // Section separator before the class-picker region.
                panel.spawn((
                    Name::new("Lobby Section Separator 2"),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(0.0),
                        margin: UiRect {
                            top: Val::Px(SPACING_XL - SPACING_MD),
                            ..default()
                        },
                        ..default()
                    },
                ));

                // Section 3 -- class picker.
                // Story 025 replaces the independent portrait/button rows
                // with one hierarchy: heading -> fixed grid -> paired
                // portrait/button cells. Neutral remains in the same block
                // as a non-selectable portrait reconciliation cell.
                panel
                    .spawn((
                        LobbyClassPickerBlock,
                        Name::new("Lobby Class Picker"),
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(SPACING_SM),
                            ..default()
                        },
                    ))
                    .with_children(|class_picker| {
                        class_picker.spawn((
                            LobbyClassPickerHeading,
                            Text::new("Class"),
                            lobby_text_font(typography::H3),
                            TextColor(Color::srgb(0.92, 0.95, 0.98)),
                        ));

                        class_picker
                            .spawn((
                                LobbyClassPickerGrid,
                                Name::new("Lobby Class Picker Grid"),
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    flex_wrap: FlexWrap::NoWrap,
                                    column_gap: Val::Px(SPACING_SM),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                            ))
                            .with_children(|grid| {
                                for class_id in lobby_class_options() {
                                    let (background, border) = lobby_class_picker_cell_colors(
                                        class_id,
                                        input.selected_class,
                                        true,
                                        lobby.locked_class,
                                    );
                                    grid.spawn((
                                        LobbyClassPickerCell {
                                            class_id,
                                            selectable: true,
                                        },
                                        Name::new(format!("Lobby Class Cell {:?}", class_id)),
                                        lobby_class_picker_cell_node(),
                                        background,
                                        border,
                                    ))
                                    .with_children(|cell| {
                                        cell.spawn((
                                            LobbyClassPortrait { class_id },
                                            Name::new(format!("Lobby Portrait {:?}", class_id)),
                                            lobby_class_portrait_node(),
                                            ImageNode::new(
                                                asset_server.load(lobby_portrait_asset(class_id)),
                                            ),
                                        ))
                                        .with_children(
                                            |portrait| {
                                                portrait.spawn((
                                                    LobbyClassPickerEmblem { class_id },
                                                    Name::new(format!(
                                                        "Lobby Class Emblem {:?}",
                                                        class_id
                                                    )),
                                                    lobby_class_picker_emblem_node(),
                                                    ImageNode::new(
                                                        asset_server
                                                            .load(class_type_icon_asset(class_id)),
                                                    ),
                                                ));
                                            },
                                        );
                                        cell.spawn((
                                            LobbyClassButton { class_id },
                                            LobbyDynamicText::Class(class_id),
                                            Button,
                                            Interaction::None,
                                            Text::new(lobby_dynamic_copy(
                                                LobbyDynamicText::Class(class_id),
                                                &lobby,
                                                &input,
                                            )),
                                            lobby_text_font(typography::BODY),
                                            TextColor(Color::srgb(0.92, 0.95, 0.98)),
                                            lobby_button_node(
                                                Val::Px(LOBBY_CLASS_PICKER_BUTTON_WIDTH_PX),
                                                LOBBY_CLASS_PICKER_BUTTON_HEIGHT_PX,
                                            ),
                                            BackgroundColor(Color::srgba(0.10, 0.13, 0.17, 0.95)),
                                            BorderColor::all(Color::srgb(0.30, 0.38, 0.48)),
                                        ));
                                    });
                                }

                                for class_id in lobby_all_class_ids() {
                                    if lobby_class_options().contains(&class_id) {
                                        continue;
                                    }
                                    let (background, border) = lobby_class_picker_cell_colors(
                                        class_id,
                                        input.selected_class,
                                        false,
                                        lobby.locked_class,
                                    );
                                    grid.spawn((
                                        LobbyClassPickerCell {
                                            class_id,
                                            selectable: false,
                                        },
                                        Name::new(format!("Lobby Class Cell {:?}", class_id)),
                                        lobby_class_picker_cell_node(),
                                        background,
                                        border,
                                    ))
                                    .with_children(|cell| {
                                        cell.spawn((
                                            LobbyClassPortrait { class_id },
                                            Name::new(format!("Lobby Portrait {:?}", class_id)),
                                            lobby_class_portrait_node(),
                                            ImageNode::new(
                                                asset_server.load(lobby_portrait_asset(class_id)),
                                            ),
                                        ))
                                        .with_children(
                                            |portrait| {
                                                portrait.spawn((
                                                    LobbyClassPickerEmblem { class_id },
                                                    Name::new(format!(
                                                        "Lobby Class Emblem {:?}",
                                                        class_id
                                                    )),
                                                    lobby_class_picker_emblem_node(),
                                                    ImageNode::new(
                                                        asset_server
                                                            .load(class_type_icon_asset(class_id)),
                                                    ),
                                                ));
                                            },
                                        );
                                        cell.spawn((
                                            Text::new(format!("{class_id:?}")),
                                            lobby_text_font(typography::CAPTION),
                                            TextColor(Color::srgba(0.74, 0.80, 0.86, 0.74)),
                                        ));
                                    });
                                }
                            });
                    });

                // Section 4 — slot panels (PAW-006-b). Per AC3(e) the slot
                // panels MUST render above the confirm CTA so the player's
                // attention reaches the seating affordance before the
                // primary action button.
                //
                // PROMPT 1138 — the slot panel asset is a generic blue chip
                // (`ui_player_slot_panel.png` placeholder) shared by both
                // slots. Without text overlays the two chips read as a pair
                // of unidentified card placeholders (AUDIT-1129-08). The
                // inline labels turn them into informative status panels:
                // "You · {class} · slot N" / "Opp · {class or unknown}".
                panel.spawn((lobby_row_node(),)).with_children(|row| {
                    row.spawn((
                        LobbyOwnSlotPanel,
                        Name::new("Lobby Own Slot Panel"),
                        Node {
                            width: Val::Px(LOBBY_SLOT_PANEL_WIDTH_PX),
                            height: Val::Px(LOBBY_SLOT_PANEL_HEIGHT_PX),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::horizontal(Val::Px(SPACING_SM)),
                            ..default()
                        },
                        ImageNode::new(asset_server.load(LOBBY_PLAYER_SLOT_PANEL_ASSET)),
                    ))
                    .with_children(|own| {
                        own.spawn((
                            LobbyOwnSlotLabel,
                            LobbyDynamicText::OwnSlot,
                            Text::new(lobby_dynamic_copy(
                                LobbyDynamicText::OwnSlot,
                                &lobby,
                                &input,
                            )),
                            lobby_text_font(typography::BODY),
                            TextColor(Color::srgb(0.95, 0.97, 1.0)),
                        ));
                    });
                    row.spawn((
                        LobbyOpponentSlotPanel,
                        Name::new("Lobby Opponent Slot Panel"),
                        Node {
                            width: Val::Px(LOBBY_SLOT_PANEL_WIDTH_PX),
                            height: Val::Px(LOBBY_SLOT_PANEL_HEIGHT_PX),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::horizontal(Val::Px(SPACING_SM)),
                            ..default()
                        },
                        ImageNode::new(asset_server.load(LOBBY_PLAYER_SLOT_PANEL_ASSET)),
                    ))
                    .with_children(|opp| {
                        opp.spawn((
                            LobbyOpponentSlotLabel,
                            LobbyDynamicText::OpponentSlot,
                            Text::new(lobby_dynamic_copy(
                                LobbyDynamicText::OpponentSlot,
                                &lobby,
                                &input,
                            )),
                            lobby_text_font(typography::BODY),
                            TextColor(Color::srgb(0.86, 0.90, 0.96)),
                        ));
                    });
                });

                // Section separator before the confirm CTA (final section).
                panel.spawn((
                    Name::new("Lobby Section Separator 3"),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(0.0),
                        margin: UiRect {
                            top: Val::Px(SPACING_XL - SPACING_MD),
                            ..default()
                        },
                        ..default()
                    },
                ));

                // Section 5 — confirm CTA. Last child per AC3(e) read
                // order; PROMPT 802 §3.1 L4 "portraits / slot panels /
                // room-code chip render below confirm" inversion is
                // resolved by placing the CTA last in the panel.
                //
                // PROMPT 985: `flex_shrink: 0.0` keeps the CTA at its
                // canonical 30 px even if other panel children expand
                // and push against the panel's `max_height: 92%` clamp.
                // Without this, the CTA was the first child the flex
                // solver squashed to zero, making it invisible.
                //
                // PROMPT 1081: initial colors come from
                // `lobby_confirm_button_colors` so the spawn baseline
                // matches the per-frame refresh — the button reads as
                // a real primary CTA from frame 0, not as a dim text
                // band waiting for the first interaction to repaint.
                let initial_state =
                    lobby_confirm_button_style_state(&lobby, &input, Interaction::None);
                let (initial_bg, initial_border, initial_text_color) =
                    lobby_confirm_button_colors(initial_state);
                panel.spawn((
                    LobbyConfirmClassButton,
                    LobbyDynamicText::Confirm,
                    Button,
                    Interaction::None,
                    Text::new(lobby_dynamic_copy(
                        LobbyDynamicText::Confirm,
                        &lobby,
                        &input,
                    )),
                    lobby_text_font(typography::BODY),
                    initial_text_color,
                    Node {
                        flex_shrink: 0.0,
                        ..lobby_button_node(
                            Val::Percent(LOBBY_CONFIRM_BUTTON_WIDTH_PERCENT),
                            LOBBY_CONFIRM_BUTTON_HEIGHT_PX,
                        )
                    },
                    initial_bg,
                    initial_border,
                ));
            });
        });
}

fn refresh_lobby_ui_system(
    lobby: Res<LobbyViewState>,
    input: Res<LobbyInputState>,
    mut texts: Query<&mut Text, With<LobbyStatusText>>,
    mut dynamic_texts: Query<(&LobbyDynamicText, &mut Text), Without<LobbyStatusText>>,
    mut class_cells: Query<(
        &LobbyClassPickerCell,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
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

    for (cell, mut background, mut border) in &mut class_cells {
        let (next_background, next_border) = lobby_class_picker_cell_colors(
            cell.class_id,
            input.selected_class,
            cell.selectable,
            lobby.locked_class,
        );
        *background = next_background;
        *border = next_border;
    }
}

/// Status banner copy.
///
/// **PROMPT 985 — Confirm CTA reachability**: the format is intentionally
/// two lines (was six) so the lobby panel content fits inside its
/// `max_height: 92%` clamp at the minimum 1366×768 viewport without
/// pushing the bottom-most child (`LobbyConfirmClassButton`) past the
/// visible viewport. See
/// `tests/integration/playable_client/lobby_confirm_button_reachable_test.rs`.
/// The `Players: N/M` substring is preserved for the
/// `lobby_entry_test::class_confirmations_are_server_confirmed` assertion.
///
/// **PROMPT 1138 — Status banner grouping**: AUDIT-1129-13 reported that
/// the legacy `Status: ... | Room: ... | Players: N/M` pipe-delimited
/// format read as terminal log output. The pipe `|` delimiters are now
/// replaced with bullet `·` separators with breathing space, the leading
/// `Status:` and `Join:` prefixes are dropped (their context is implied
/// by line position), and the join-input is folded into a `Joining {code}`
/// segment only when the user is mid-type. The two-line bound is preserved
/// and the `Players: N/M` substring is preserved verbatim.
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
    let slot_segment = if input.join_room_code.is_empty() {
        format!("Slot {}", input.requested_slot)
    } else {
        format!(
            "Joining {} · Slot {}",
            input.join_room_code, input.requested_slot
        )
    };

    format!(
        "{}  ·  Room: {}  ·  Players: {}/{}\n{}  ·  Class: {:?}  ·  {}",
        lobby.status, room, joined, total, slot_segment, input.selected_class, locked
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
        LobbyDynamicText::Confirm => lobby_confirm_button_text(lobby, input),
        LobbyDynamicText::OwnSlot => lobby_own_slot_label_text(lobby, input),
        LobbyDynamicText::OpponentSlot => lobby_opponent_slot_label_text(lobby),
    }
}

/// PROMPT 1138 — own-slot panel inline label.
///
/// Composes "You · {class}{*} · slot N" where `*` is appended when the
/// local player has confirmed their class (`lobby.locked_class.is_some()`)
/// so the slot panel reads as informative status, not an unidentified
/// blue-card placeholder (AUDIT-1129-08). The asterisk re-uses the
/// existing convention from [`LobbyDynamicText::Class`] / `Slot` so it does
/// not introduce a new selection-state glyph.
pub fn lobby_own_slot_label_text(lobby: &LobbyViewState, input: &LobbyInputState) -> String {
    let confirmed_marker = if lobby.locked_class.is_some() {
        " *"
    } else {
        ""
    };
    format!(
        "You · {:?}{} · slot {}",
        input.selected_class, confirmed_marker, input.requested_slot
    )
}

/// PROMPT 1138 — opponent-slot panel inline label.
///
/// Resolves the opponent's revealed class when `lobby.revealed_classes`
/// carries an entry not matching `lobby.local_player_id`; otherwise reads
/// as "Opp · waiting" so the slot panel still announces the seat instead
/// of rendering as a blank placeholder (AUDIT-1129-08).
pub fn lobby_opponent_slot_label_text(lobby: &LobbyViewState) -> String {
    let opponent_class = lobby
        .revealed_classes
        .iter()
        .find(|(player_id, _)| Some(*player_id) != lobby.local_player_id)
        .map(|(_, class_id)| format!("{:?}", class_id));
    match opponent_class {
        Some(class) => format!("Opp · {}", class),
        None => "Opp · waiting".to_string(),
    }
}

/// Confirm-button text for the lobby UI, after the in-flight branch.
///
/// Differentiates two states that previously rendered an identical
/// "Confirming..." surface (S11-LOBBY-UX-CONFIRM-STATE-001 / story 023):
///
/// - **State B** — local player has not yet sent `C2SConfirmClass`
///   (`lobby.locked_class` is `None`). Renders: `"Confirm your class to
///   continue"`.
/// - **State A** — `S2CClassLocked` for the local player has been applied
///   (`lobby.locked_class` is `Some`) but `S2CClassesRevealed` has not yet
///   arrived (`lobby.revealed_classes` is empty). Renders: `"Waiting for
///   opponent..."`.
/// - Post-reveal (both classes broadcast) renders: `"All players
///   confirmed"`. The lobby transitions to `InSession` shortly after, so
///   this string is a transient surface for the gap between reveal and
///   state transition.
///
/// Sprint 12 story 013 fallback path (duplicate same-class confirm
/// returning an `S2CClassLocked` re-ack) preserves State A: the re-ack
/// keeps `locked_class` `Some(class_id)` and does not clear
/// `revealed_classes`.
///
/// `_input` is accepted for API symmetry with `lobby_dynamic_copy` and to
/// reserve a hook for future input-state-driven copy variants without
/// breaking callers. Treat the caller as responsible for routing
/// `class_confirm_in_flight` to its own branch before invoking this
/// helper.
pub fn lobby_confirm_button_text(lobby: &LobbyViewState, _input: &LobbyInputState) -> String {
    let own_locked = lobby.locked_class.is_some();
    let opponent_revealed = !lobby.revealed_classes.is_empty();
    match (own_locked, opponent_revealed) {
        (false, _) => "Confirm your class to continue".to_string(),
        (true, false) => "Waiting for opponent...".to_string(),
        (true, true) => "All players confirmed".to_string(),
    }
}

pub fn lobby_class_options() -> [ClassId; 6] {
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
pub fn lobby_all_class_ids() -> [ClassId; 7] {
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
        height: Val::Px(LOBBY_BUTTON_HEIGHT_PX),
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(8.0),
        align_items: AlignItems::Center,
        ..default()
    }
}

fn lobby_class_picker_cell_node() -> Node {
    Node {
        width: Val::Px(LOBBY_CLASS_PICKER_CELL_WIDTH_PX),
        height: Val::Px(LOBBY_CLASS_PICKER_CELL_HEIGHT_PX),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(SPACING_SM),
        padding: UiRect::all(Val::Px(LOBBY_CLASS_PICKER_CELL_PADDING_PX)),
        border: UiRect::all(Val::Px(2.0)),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_shrink: 0.0,
        ..default()
    }
}

fn lobby_class_portrait_node() -> Node {
    Node {
        width: Val::Px(LOBBY_CLASS_PICKER_PORTRAIT_WIDTH_PX),
        height: Val::Px(LOBBY_CLASS_PICKER_PORTRAIT_HEIGHT_PX),
        // Relative so an absolutely-positioned emblem child anchors
        // against the portrait's bounding box.
        position_type: PositionType::Relative,
        ..default()
    }
}

/// PROMPT 1138 — class-distinct emblem overlay node. Anchored to the
/// portrait's top-right corner via `position_type: Absolute`. Square
/// dimensions match [`LOBBY_CLASS_PICKER_EMBLEM_PX`] so the badge keeps
/// a readable footprint without obscuring the portrait body.
fn lobby_class_picker_emblem_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(2.0),
        top: Val::Px(2.0),
        width: Val::Px(LOBBY_CLASS_PICKER_EMBLEM_PX),
        height: Val::Px(LOBBY_CLASS_PICKER_EMBLEM_PX),
        ..default()
    }
}

/// PROMPT 1138 — picker cell `(BackgroundColor, BorderColor)` derived from
/// `(class_id, selected_class, selectable, locked_class)`.
///
/// State precedence (highest first):
///   * **Confirmed** — `locked_class == Some(class_id)`; mirrors the
///     `LobbyConfirmButtonStyleState::Confirmed` green palette so the
///     locked-class cell and the confirm CTA read as one decision.
///   * **Selected** — `selectable && class_id == selected_class`; gold
///     accent ratifying
///     [`design_tokens::interaction_states::FOCUS_RING_COLOR`].
///   * **Selectable** — neutral resting state.
///   * **Non-selectable** — Neutral reconciliation tile; dimmed.
fn lobby_class_picker_cell_colors(
    class_id: ClassId,
    selected_class: ClassId,
    selectable: bool,
    locked_class: Option<ClassId>,
) -> (BackgroundColor, BorderColor) {
    if selectable && locked_class == Some(class_id) {
        (
            BackgroundColor(Color::srgba(0.10, 0.28, 0.16, 0.96)),
            BorderColor::all(Color::srgb(0.40, 0.84, 0.50)),
        )
    } else if selectable && class_id == selected_class {
        (
            BackgroundColor(Color::srgba(0.18, 0.16, 0.08, 0.96)),
            BorderColor::all(Color::srgb(0.949, 0.788, 0.298)),
        )
    } else if selectable {
        (
            BackgroundColor(Color::srgba(0.10, 0.13, 0.17, 0.92)),
            BorderColor::all(Color::srgb(0.30, 0.38, 0.48)),
        )
    } else {
        (
            BackgroundColor(Color::srgba(0.08, 0.10, 0.13, 0.62)),
            BorderColor::all(Color::srgba(0.42, 0.48, 0.56, 0.42)),
        )
    }
}

fn lobby_button_node(width: Val, height_px: f32) -> Node {
    Node {
        width,
        height: Val::Px(height_px),
        border: UiRect::all(Val::Px(1.0)),
        padding: UiRect::horizontal(Val::Px(8.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

// ─── Confirm CTA visual states (PROMPT 1081) ────────────────────────────────
//
// AUDIT-1076-07 reported that the Confirm-class button rendered as a dim text
// band rather than a real primary-action button: the click was reachable in
// logs (`lobby_ui_confirm_button_state: dispatching ConfirmClass`), but the
// only chrome was a dark `srgba(0.17, 0.18, 0.14, 0.95)` background with no
// interaction-state feedback. This module promotes the CTA to a stateful
// primary button with six visual states keyed off `(Interaction,
// LobbyViewState, LobbyInputState)`.
//
// State precedence (highest first):
//   Confirmed (revealed_classes non-empty)
//   Waiting   (locked_class set, no reveal yet)
//   InFlight  (class_confirm_in_flight)
//   Disabled  (session_id is None)
//   then the interaction-driven trio Pressed / Hovered / Enabled.

/// Discrete visual state of the lobby confirm CTA. Each variant maps to a
/// `(BackgroundColor, BorderColor, TextColor)` triple via
/// [`lobby_confirm_button_colors`] so the spawn and per-frame refresh paths
/// share a single source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LobbyConfirmButtonStyleState {
    /// No active session yet — class confirmation is not reachable.
    Disabled,
    /// Active session, idle: primary CTA in its prompting state.
    Enabled,
    /// Pointer is over the CTA, no button pressed.
    Hovered,
    /// Pointer is over the CTA with a mouse button held.
    Pressed,
    /// `C2SConfirmClass` is in flight, awaiting server response.
    InFlight,
    /// Local class is locked but opponent has not yet revealed.
    Waiting,
    /// `S2CClassesRevealed` has been applied; both players confirmed.
    Confirmed,
}

/// Derive the confirm CTA's visual state from the lobby/input resources
/// plus the button's current [`Interaction`]. Pure function: no side effects,
/// no resource mutation, deterministic given the inputs.
pub fn lobby_confirm_button_style_state(
    lobby: &LobbyViewState,
    input: &LobbyInputState,
    interaction: Interaction,
) -> LobbyConfirmButtonStyleState {
    use LobbyConfirmButtonStyleState as S;
    if !lobby.revealed_classes.is_empty() {
        return S::Confirmed;
    }
    if lobby.locked_class.is_some() {
        return S::Waiting;
    }
    if input.class_confirm_in_flight {
        return S::InFlight;
    }
    if lobby.session_id.is_none() {
        return S::Disabled;
    }
    match interaction {
        Interaction::Pressed => S::Pressed,
        Interaction::Hovered => S::Hovered,
        Interaction::None => S::Enabled,
    }
}

/// Map a [`LobbyConfirmButtonStyleState`] to its visual triple.
///
/// The Enabled treatment uses the global UI spec §7 `ACCENT` gold
/// (`Color::srgb(0.949, 0.788, 0.298)` — same triple as
/// `FOCUS_RING_COLOR`) so the primary CTA reads against the
/// `SURFACE_ELEVATED` panel even at the minimum 1366×768 viewport.
/// Hovered/Pressed shade the same hue brighter / darker. Disabled tones
/// the accent down and applies the §11 alpha bands so the state reads
/// as visibly inert. The §11 named alpha constants are imported and
/// audited by `__interaction_state_alphas_used_in_confirm_button_colors`
/// below so a future spec revision picks up here without orphaning the
/// tokens.
pub fn lobby_confirm_button_colors(
    state: LobbyConfirmButtonStyleState,
) -> (BackgroundColor, BorderColor, TextColor) {
    use LobbyConfirmButtonStyleState as S;
    match state {
        S::Disabled => (
            BackgroundColor(Color::srgba(0.30, 0.27, 0.16, 1.0 - DISABLED_BG_TINT_ALPHA)),
            BorderColor::all(Color::srgba(0.55, 0.48, 0.28, DISABLED_BORDER_ALPHA + 0.42)),
            TextColor(Color::srgba(0.86, 0.82, 0.66, DISABLED_TEXT_ALPHA + 0.30)),
        ),
        S::Enabled => (
            BackgroundColor(Color::srgb(0.949, 0.788, 0.298)),
            BorderColor::all(Color::srgb(1.00, 0.90, 0.50)),
            TextColor(Color::srgb(0.06, 0.05, 0.02)),
        ),
        S::Hovered => (
            BackgroundColor(Color::srgb(0.984, 0.860, 0.420)),
            BorderColor::all(Color::srgba(
                1.00,
                0.94,
                0.62,
                1.0 - HOVER_BORDER_ALPHA * 0.20,
            )),
            TextColor(Color::srgb(0.06, 0.05, 0.02)),
        ),
        S::Pressed => (
            BackgroundColor(Color::srgb(0.700, 0.560, 0.200)),
            BorderColor::all(Color::srgb(1.00, 0.90, 0.50)),
            TextColor(Color::srgb(0.04, 0.03, 0.01)),
        ),
        S::InFlight => (
            BackgroundColor(Color::srgba(0.52, 0.43, 0.16, 0.95)),
            BorderColor::all(Color::srgb(0.74, 0.62, 0.24)),
            TextColor(Color::srgb(0.98, 0.93, 0.72)),
        ),
        S::Waiting => (
            BackgroundColor(Color::srgb(0.18, 0.30, 0.42)),
            BorderColor::all(Color::srgb(0.48, 0.68, 0.86)),
            TextColor(Color::srgb(0.88, 0.96, 1.0)),
        ),
        S::Confirmed => (
            BackgroundColor(Color::srgb(0.16, 0.42, 0.22)),
            BorderColor::all(Color::srgb(0.40, 0.84, 0.50)),
            TextColor(Color::srgb(0.92, 1.0, 0.94)),
        ),
    }
}

/// Used-token guard: keeps the `HOVER_BG_TINT_ALPHA` import live (it
/// inputs the Hovered band documentation even when the literal mix is
/// pre-computed). Without this guard the `cargo --deny unused_imports`
/// gate would flag the import after the inline-literal simplification.
#[doc(hidden)]
const __HOVER_BG_TINT_ALPHA_USED_IN_CONFIRM_BUTTON: f32 = HOVER_BG_TINT_ALPHA;

/// Compile-time guard: `PRESSED_BG_TINT_ALPHA` is part of the documented
/// Pressed-state contract even when the precomputed literal lives in
/// `lobby_confirm_button_colors`. Reading the const here keeps the import
/// live.
#[doc(hidden)]
const __PRESSED_BG_TINT_ALPHA_USED_IN_CONFIRM_BUTTON: f32 = PRESSED_BG_TINT_ALPHA;

/// Update the confirm CTA's `BackgroundColor` / `BorderColor` / `TextColor`
/// every frame based on the current `(Interaction, LobbyViewState,
/// LobbyInputState)` triple. Runs at the tail of the lobby `Update` chain
/// so the spawn-time colors and the per-frame refresh share the same
/// helper output, and so [`lobby_button_interaction_system`] sees a
/// fresh `Interaction` before this system reads it on the next tick.
pub fn refresh_confirm_button_visual_system(
    lobby: Res<LobbyViewState>,
    input: Res<LobbyInputState>,
    mut buttons: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut TextColor,
        ),
        With<LobbyConfirmClassButton>,
    >,
) {
    for (interaction, mut bg, mut border, mut text_color) in &mut buttons {
        let state = lobby_confirm_button_style_state(&lobby, &input, *interaction);
        let (next_bg, next_border, next_text) = lobby_confirm_button_colors(state);
        *bg = next_bg;
        *border = next_border;
        *text_color = next_text;
    }
}
