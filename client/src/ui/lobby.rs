use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::window::{CursorIcon, SystemCursorIcon};
use lightyear::prelude::{MessageReceiver, MessageSender};
use shared::card::ClassId;
use shared::protocol::{
    BotKind, C2SAddBot, C2SConfirmClass, C2SCreateBotRoom, C2SCreateRoom, C2SJoinRoom,
    C2SListRooms, C2SRemoveBot, C2SSelectClass, GameMode, ReliableChannel, RoomListEntry,
    S2CBotActionRejected, S2CClassLocked, S2CClassesRevealed, S2CConfirmClassRejected,
    S2CCreateRoomRejected, S2CHandshake, S2CHandshakeRejected, S2CJoinAck, S2CJoinRejected,
    S2CRoomCreated, S2CRoomList, S2CSlotUpdated, SessionSlot,
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
    status_chip::StatusChip,
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

// Wave-2 interaction-state migration (S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001):
// Named base-color constants for Create / Join so the spawn sites reference
// tokens and the change-detection overlay system can compute tints from them.
const LOBBY_CREATE_BUTTON_BG: Color = Color::srgba(0.17, 0.18, 0.14, 0.95);
const LOBBY_CREATE_BUTTON_BORDER: Color = Color::srgb(0.65, 0.53, 0.24);
const LOBBY_JOIN_BUTTON_BG: Color = Color::srgba(0.11, 0.15, 0.20, 0.95);
const LOBBY_JOIN_BUTTON_BORDER: Color = Color::srgb(0.28, 0.56, 0.72);
pub const LOBBY_REQUESTED_SLOT_BUTTON_WIDTH_PX: f32 = 80.0;
pub const LOBBY_REQUESTED_SLOT_BUTTON_HEIGHT_PX: f32 = LOBBY_BUTTON_HEIGHT_PX;
pub const LOBBY_CLASS_PICKER_GRID_COLUMNS: usize = 7;
pub const LOBBY_CLASS_PICKER_SELECTABLE_COUNT: usize = 6;
pub const LOBBY_CLASS_PICKER_CELL_WIDTH_PX: f32 = 108.0;
pub const LOBBY_CLASS_PICKER_CELL_HEIGHT_PX: f32 = 132.0;
pub const LOBBY_CLASS_PICKER_CELL_PADDING_PX: f32 = 6.0;
pub const LOBBY_CLASS_PICKER_PORTRAIT_WIDTH_PX: f32 = 64.0;
pub const LOBBY_CLASS_PICKER_PORTRAIT_HEIGHT_PX: f32 = 80.0;
pub const LOBBY_SELECTED_CLASS_PANEL_HEIGHT_PX: f32 = 76.0;
pub const LOBBY_SELECTED_CLASS_PORTRAIT_WIDTH_PX: f32 = 56.0;
pub const LOBBY_SELECTED_CLASS_PORTRAIT_HEIGHT_PX: f32 = 64.0;

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
                    drain_bot_action_rejected_system,
                    lobby_initial_room_list_refresh_system,
                    lobby_keyboard_input_system,
                    lobby_button_interaction_system,
                    // Wave-2 overlay tints on Create / Join buttons (AC2).
                    // Runs after the action-handler so action dispatch and
                    // visual update land in the same tick.
                    lobby_create_join_interaction_overlay_system,
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
    /// PROMPT 1160 — joinable-room list received from the server via
    /// `S2CRoomList`. Empty by default; populated on handshake and on every
    /// Refresh interaction. Server filters out the local player's own room and
    /// any non-`LobbyWaiting`/fully-occupied rooms (see
    /// `server::core::session::system::build_room_list`).
    pub room_list: Vec<RoomListEntry>,
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
            room_list: Vec::new(),
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
    /// PROMPT 1160 — request the server's joinable-room list via `C2SListRooms`.
    RefreshRooms,
    SelectClass {
        class_id: ClassId,
    },
    ConfirmClass {
        class_id: ClassId,
    },
    /// PROMPT 1596 — Mode 1 QA helper: ask the server to occupy an empty
    /// non-local slot with a bot of the given flavour. Sent via
    /// `C2SAddBot`; success surfaces back through `S2CSlotUpdated`,
    /// rejection through `S2CBotActionRejected`.
    AddBot {
        slot: u8,
        bot_kind: BotKind,
    },
    /// PROMPT 1596 — Mode 1 QA helper: ask the server to evict the bot
    /// currently occupying `slot`. Sent via `C2SRemoveBot`.
    RemoveBot {
        slot: u8,
    },
    /// PROMPT 1603 — debug-only bot-vs-bot soak QA helper: ask the server to
    /// create a fresh room pre-seeded with a bot in the opposing-team slot.
    /// Sent via `C2SCreateBotRoom`. Surfaced only behind `CCGS_DEBUG_UI=1`.
    /// A follow-up `C2SAddBot` is still required from the room owner to fill
    /// the second seat for the actual bot-vs-bot soak (see
    /// `tools/dev-launcher/Start-BotVsBotSoak.ps1`).
    CreateBotRoom {
        mode: GameMode,
        bot_kind: BotKind,
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

/// PROMPT 1398 (S18-LOBBY-CONFIRM-CTA-VIEWPORT-REACHABILITY-001) — body
/// region that owns every panel child ABOVE the Confirm CTA (status banner,
/// room-code chip, create/join row, optional existing-room browser, optional
/// requested-slot row, class picker, slot status chips). The body region
/// carries `flex_grow: 1.0`, `flex_shrink: 1.0`, `min_height: 0.0`, and
/// `overflow: clip_y`, so when the cumulative body content exceeds the
/// panel's `max_height: 92%` content area the body is the slot that
/// absorbs the pressure — the Confirm CTA (the next sibling, with
/// `flex_shrink: 0.0`) stays anchored to the panel's bottom edge and on
/// every supported viewport (1280×720, 1366×768, 1920×1080).
///
/// AUDIT-1392-P04 / HUNT-1201-01: users could pick a class but never reach
/// the Confirm CTA at 1280×720 because the body content silently overflowed
/// past the panel clamp, pushing the CTA below the visible viewport. This
/// wrapper structurally guarantees the CTA stays reachable regardless of
/// how dense the body content becomes (room-list rows, slot panels, future
/// additions).
#[derive(Component)]
pub struct LobbyPanelBody;

/// Class-picker region container. Owns the region heading and the class grid
/// so portraits and selectable controls no longer wrap as independent rows.
#[derive(Component)]
pub struct LobbyClassPickerBlock;

#[derive(Component)]
pub struct LobbyClassPickerHeading;

#[derive(Component)]
pub struct LobbyClassPickerGrid;

#[derive(Component)]
pub struct LobbySelectedClassIdentityPanel;

#[derive(Component)]
pub struct LobbySelectedClassIdentityPortrait;

#[derive(Component)]
pub struct LobbySelectedClassIdentityText;

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

/// PROMPT 1178 — text label that introduces the manual-typed-join
/// requested-slot row. Marked as its own component so tests can assert
/// that the label is present only while `lobby.session_id` is `None`
/// (manual typed-join surface), and absent after the player joined a
/// room from the existing-room browser or created their own room.
#[derive(Component)]
pub struct LobbyRequestedSlotLabel;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobbyClassButton {
    pub class_id: ClassId,
}

#[derive(Component)]
pub struct LobbyConfirmClassButton;

/// PROMPT 1160 — refresh button in the existing-room browser panel. Pressing it
/// writes a `LobbyCommand::RefreshRooms` which sends `C2SListRooms` on the
/// reliable channel.
#[derive(Component)]
pub struct LobbyRefreshRoomsButton;

/// PROMPT 1160 — container for the joinable-room rows in the lobby. Rebuilt
/// from `LobbyViewState.room_list` whenever it changes (see
/// `refresh_lobby_ui_system`).
#[derive(Component)]
pub struct LobbyRoomListContainer;

/// PROMPT 1160 — one row in the existing-room browser. Clicking the row writes
/// `LobbyCommand::JoinRoom { room_code, requested_slot }` using the
/// server-supplied `first_open_slot` so the click does not require the player
/// to think about slot indices. Rows whose `first_open_slot` is `None` (full)
/// are rendered as a non-interactive label rather than a button.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct LobbyRoomListRow {
    pub room_code: String,
    pub requested_slot: u8,
}

/// PROMPT 1596 (BOT-FLOW-LOBBY-ADD-REMOVE-BOT-UX) — dynamic container for
/// contextual Add Bot / Remove Bot controls. Spawned empty inside the lobby
/// panel body during `spawn_lobby_ui_system`; populated reactively by
/// `refresh_lobby_ui_system` whenever `LobbyViewState` changes so the row of
/// controls always reflects the authoritative `lobby.slots` snapshot.
///
/// The container is sized by its children: empty before the local player
/// joins/creates a room (no slot data yet) and after both seats are taken by
/// humans; non-zero only when there is at least one non-local slot that is
/// either empty (Add Bot eligible) or bot-occupied (Remove Bot eligible).
#[derive(Component)]
pub struct LobbyBotControlsContainer;

/// PROMPT 1596 — Add Bot button marker. Carries the target slot index so the
/// interaction system can dispatch the right `C2SAddBot` payload. Rendered
/// only when the slot is empty (no `player_id`, `is_bot == false`).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobbyAddBotButton {
    pub slot: u8,
}

/// PROMPT 1596 — Remove Bot button marker. Rendered only when the slot is
/// currently held by a bot (`is_bot == true`).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobbyRemoveBotButton {
    pub slot: u8,
}

/// PROMPT 1603 — debug-only `Create 2-Bot Soak Room` button marker. Spawned
/// only when [`is_debug_ui_enabled`] returns `true` AND
/// `LobbyViewState.session_id` is `None` (the player has not yet joined or
/// created a room). Provides the entry point for the headless bot-vs-bot
/// soak QA flow tracked under `reports/PROMPT-1594-bot-flow-inventory-followup.md`
/// item 7. The control is invisible in production builds where the
/// `CCGS_DEBUG_UI` env var is unset.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobbyCreateBotRoomButton;

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
    /// PROMPT 1160 — Refresh button label in the existing-room browser.
    Refresh,
    /// PROMPT 1160 — empty-state label that renders when the room list is
    /// empty, so the panel never appears as a blank slab.
    RoomListEmpty,
    /// PROMPT 1487 — selected-class identity panel copy.
    SelectedClassIdentity,
    /// PROMPT 1596 — Add Bot button label, parameterised by target slot.
    AddBot(u8),
    /// PROMPT 1596 — Remove Bot button label, parameterised by target slot.
    RemoveBot(u8),
    /// PROMPT 1603 — debug-only `Create 2-Bot Soak Room` button label.
    CreateBotRoom,
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
    mut room_lists: Query<&mut MessageReceiver<S2CRoomList>>,
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

    for mut receiver in &mut room_lists {
        for message in receiver.receive() {
            tracing::info!(
                room_count = message.rooms.len(),
                msg_type = "S2CRoomList",
                "drain_lobby_s2c: recv"
            );
            apply_room_list(&mut lobby, &message);
        }
    }
}

/// PROMPT 1160 — apply `S2CRoomList` into `LobbyViewState.room_list`. Pure
/// function: no I/O, no command writes; exposed for tests.
pub fn apply_room_list(lobby: &mut LobbyViewState, message: &S2CRoomList) {
    lobby.room_list = message.rooms.clone();
}

/// PROMPT 1596 (BOT-FLOW-LOBBY-ADD-REMOVE-BOT-UX) — drain
/// `S2CBotActionRejected` and surface the reason on the lobby status banner.
///
/// Split out of `drain_lobby_s2c_system` to keep that system's `SystemParam`
/// tuple under the Bevy 0.18 16-element ceiling. Pure status-line mutation —
/// no slot mutation (server emits an authoritative `S2CSlotUpdated` on
/// success, so no client-side optimistic state needs unwinding on rejection).
pub fn drain_bot_action_rejected_system(
    mut lobby: ResMut<LobbyViewState>,
    mut receivers: Query<&mut MessageReceiver<S2CBotActionRejected>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::info!(
                reason = ?message.reason,
                msg_type = "S2CBotActionRejected",
                "drain_lobby_s2c: recv"
            );
            lobby.status = format!("Bot action rejected: {:?}", message.reason);
        }
    }
}

/// PROMPT 1596 — describe the contextual bot control eligible for a slot.
/// Pure helper exposed for tests; consumed by `rebuild_bot_controls_rows`.
///
/// The local player's own slot never receives a control (you cannot evict
/// yourself, and you do not need to invite yourself). Slots held by remote
/// humans (a real peer joined) also receive no control — Mode 1 is
/// 1-human-plus-1-bot, and an Add/Remove Bot affordance next to a human
/// seat would be misleading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LobbyBotSlotControl {
    /// Slot is empty (`player_id.is_none() && !is_bot`) — eligible for
    /// `C2SAddBot`.
    AddBot { slot: u8 },
    /// Slot is held by a bot (`is_bot == true`) — eligible for
    /// `C2SRemoveBot`.
    RemoveBot { slot: u8 },
}

/// PROMPT 1596 — derive the ordered list of contextual bot controls for the
/// current lobby snapshot. Stable order = ascending slot index, so the row
/// reads left-to-right in the same order as the slot panels above.
pub fn lobby_bot_controls_for_slots(
    slots: &[SessionSlot],
    local_player_id: Option<PlayerId>,
) -> Vec<LobbyBotSlotControl> {
    let mut controls: Vec<LobbyBotSlotControl> = slots
        .iter()
        .filter(|slot| match local_player_id {
            Some(local) => slot.player_id != Some(local),
            None => true,
        })
        .filter_map(|slot| {
            if slot.is_bot {
                Some(LobbyBotSlotControl::RemoveBot { slot: slot.slot })
            } else if slot.player_id.is_none() {
                Some(LobbyBotSlotControl::AddBot { slot: slot.slot })
            } else {
                None
            }
        })
        .collect();
    controls.sort_by_key(|control| match control {
        LobbyBotSlotControl::AddBot { slot } | LobbyBotSlotControl::RemoveBot { slot } => *slot,
    });
    controls
}

/// PROMPT 1596 — write a `LobbyCommand::AddBot` after gating on
/// `session_id.is_some()`. Surfaces a status banner update so the player
/// gets immediate feedback while the round-trip is in flight.
pub fn request_add_bot(
    slot: u8,
    bot_kind: BotKind,
    lobby: &mut LobbyViewState,
    commands: &mut MessageWriter<LobbyCommand>,
) {
    if lobby.session_id.is_none() {
        lobby.status = "Create or join a room before adding a bot".to_string();
        return;
    }
    lobby.status = format!("Adding bot to seat {slot}");
    commands.write(LobbyCommand::AddBot { slot, bot_kind });
}

/// PROMPT 1596 — write a `LobbyCommand::RemoveBot` after gating on
/// `session_id.is_some()`.
pub fn request_remove_bot(
    slot: u8,
    lobby: &mut LobbyViewState,
    commands: &mut MessageWriter<LobbyCommand>,
) {
    if lobby.session_id.is_none() {
        lobby.status = "Create or join a room before removing a bot".to_string();
        return;
    }
    lobby.status = format!("Removing bot from seat {slot}");
    commands.write(LobbyCommand::RemoveBot { slot });
}

/// PROMPT 1603 — pure helper for the `CCGS_DEBUG_UI` env-var contract. Exposed
/// for tests so the gating logic can be exercised without touching the
/// process-wide env. Returns `true` only when the raw value is exactly `"1"`
/// after trimming whitespace; every other value (absent, empty, `"0"`,
/// `"true"`, etc.) returns `false`. The strict `"1"` contract mirrors the
/// `CCGS_QA_SNAPSHOT=1` precedent (see
/// `client/src/presentation/qa_snapshot.rs`) so a single mental model covers
/// both debug-only surfaces.
pub fn debug_ui_enabled_from(value: Option<&str>) -> bool {
    matches!(value, Some(v) if v.trim() == "1")
}

/// PROMPT 1603 — process-wide read of the `CCGS_DEBUG_UI` env var. Read once
/// at lobby spawn time (`spawn_lobby_ui_system`) so a debug-only entry point
/// can be added/removed by relaunching the client rather than re-spawning
/// the UI per frame.
pub fn is_debug_ui_enabled() -> bool {
    debug_ui_enabled_from(std::env::var("CCGS_DEBUG_UI").ok().as_deref())
}

/// PROMPT 1603 — write a `LobbyCommand::CreateBotRoom`. Unlike
/// [`request_add_bot`] / [`request_remove_bot`] this surface does NOT require
/// a pre-existing session: its whole point is to bootstrap one. Pre-flights
/// against `lobby.session_id` to refuse silently when the caller already has
/// a room (mirrors the server contract — `C2SCreateBotRoom` is rejected with
/// `S2CBotActionRejected::AlreadyInSession` in that case, see
/// `server/src/core/session/system.rs`).
pub fn request_create_bot_room(
    mode: GameMode,
    bot_kind: BotKind,
    lobby: &mut LobbyViewState,
    commands: &mut MessageWriter<LobbyCommand>,
) {
    if lobby.session_id.is_some() {
        lobby.status = "Already in a room; leave it before creating a soak room".to_string();
        return;
    }
    lobby.status = "Creating 2-bot soak room".to_string();
    commands.write(LobbyCommand::CreateBotRoom { mode, bot_kind });
}

/// PROMPT 1160 — request the room list exactly once after the first handshake
/// completes, so the browser panel is populated before the user thinks to
/// interact. Split out from `drain_lobby_s2c_system` to keep that system's
/// `SystemParam` tuple within the 16-element limit. Re-runs only if the
/// player's identity drops to `None` and re-arrives (handshake reset path).
pub fn lobby_initial_room_list_refresh_system(
    lobby: Res<LobbyViewState>,
    mut commands: MessageWriter<LobbyCommand>,
    mut already_requested: Local<bool>,
) {
    if lobby.local_player_id.is_none() {
        *already_requested = false;
        return;
    }

    if *already_requested {
        return;
    }

    *already_requested = true;
    tracing::info!("lobby_initial_room_list_refresh: enqueuing RefreshRooms");
    commands.write(LobbyCommand::RefreshRooms);
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
            Option<&LobbyRefreshRoomsButton>,
            Option<&LobbyRoomListRow>,
            Option<&LobbyAddBotButton>,
            Option<&LobbyRemoveBotButton>,
            Option<&LobbyCreateBotRoomButton>,
        ),
        Changed<Interaction>,
    >,
    mut input: ResMut<LobbyInputState>,
    mut lobby: ResMut<LobbyViewState>,
    mut commands: MessageWriter<LobbyCommand>,
) {
    for (
        interaction,
        room_code,
        create,
        join,
        slot,
        class,
        confirm,
        refresh,
        row,
        add_bot,
        remove_bot,
        create_bot_room,
    ) in &mut interactions
    {
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
        } else if refresh.is_some() {
            request_refresh_rooms(&mut lobby, &mut commands);
        } else if let Some(row) = row {
            request_join_room_from_row(row, &mut input, &mut lobby, &mut commands);
        } else if let Some(add_bot) = add_bot {
            request_add_bot(add_bot.slot, BotKind::Default, &mut lobby, &mut commands);
        } else if let Some(remove_bot) = remove_bot {
            request_remove_bot(remove_bot.slot, &mut lobby, &mut commands);
        } else if create_bot_room.is_some() {
            request_create_bot_room(
                GameMode::OneVOne,
                BotKind::Default,
                &mut lobby,
                &mut commands,
            );
        }
    }
}

fn send_lobby_commands_system(
    mut commands: MessageReader<LobbyCommand>,
    mut create_room: Query<&mut MessageSender<C2SCreateRoom>>,
    mut join_room: Query<&mut MessageSender<C2SJoinRoom>>,
    mut select_class: Query<&mut MessageSender<C2SSelectClass>>,
    mut confirm_class: Query<&mut MessageSender<C2SConfirmClass>>,
    mut list_rooms: Query<&mut MessageSender<C2SListRooms>>,
    mut add_bot: Query<&mut MessageSender<C2SAddBot>>,
    mut remove_bot: Query<&mut MessageSender<C2SRemoveBot>>,
    mut create_bot_room: Query<&mut MessageSender<C2SCreateBotRoom>>,
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
            LobbyCommand::RefreshRooms => {
                let Some(mut sender) = list_rooms.iter_mut().next() else {
                    warn!(
                        "C2S send DROPPED: type=C2SListRooms, handler=send_lobby_commands_system, reason=no_sender_entity"
                    );
                    continue;
                };
                tracing::info!(msg_type = "C2SListRooms", "c2s_send: enter");
                sender.send::<ReliableChannel>(C2SListRooms::default());
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
            LobbyCommand::AddBot { slot, bot_kind } => {
                let Some(mut sender) = add_bot.iter_mut().next() else {
                    warn!(
                        slot = *slot,
                        bot_kind = ?bot_kind,
                        "C2S send DROPPED: type=C2SAddBot, handler=send_lobby_commands_system, reason=no_sender_entity"
                    );
                    continue;
                };
                tracing::info!(
                    msg_type = "C2SAddBot",
                    slot = *slot,
                    bot_kind = ?bot_kind,
                    "c2s_send: enter"
                );
                sender.send::<ReliableChannel>(C2SAddBot {
                    slot: *slot,
                    bot_kind: *bot_kind,
                });
            }
            LobbyCommand::RemoveBot { slot } => {
                let Some(mut sender) = remove_bot.iter_mut().next() else {
                    warn!(
                        slot = *slot,
                        "C2S send DROPPED: type=C2SRemoveBot, handler=send_lobby_commands_system, reason=no_sender_entity"
                    );
                    continue;
                };
                tracing::info!(
                    msg_type = "C2SRemoveBot",
                    slot = *slot,
                    "c2s_send: enter"
                );
                sender.send::<ReliableChannel>(C2SRemoveBot { slot: *slot });
            }
            LobbyCommand::CreateBotRoom { mode, bot_kind } => {
                let Some(mut sender) = create_bot_room.iter_mut().next() else {
                    warn!(
                        mode = ?mode,
                        bot_kind = ?bot_kind,
                        "C2S send DROPPED: type=C2SCreateBotRoom, handler=send_lobby_commands_system, reason=no_sender_entity"
                    );
                    continue;
                };
                tracing::info!(
                    msg_type = "C2SCreateBotRoom",
                    mode = ?mode,
                    bot_kind = ?bot_kind,
                    "c2s_send: enter"
                );
                sender.send::<ReliableChannel>(C2SCreateBotRoom {
                    mode: *mode,
                    bot_kind: *bot_kind,
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

/// PROMPT 1160 — write a `LobbyCommand::RefreshRooms` and surface a lightweight
/// banner update. Always writes exactly one command per click; the
/// `send_lobby_commands_system` reader collapses repeats into individual
/// `C2SListRooms` sends with no need for a latch (the request is cheap and
/// idempotent server-side).
pub fn request_refresh_rooms(
    lobby: &mut LobbyViewState,
    commands: &mut MessageWriter<LobbyCommand>,
) {
    lobby.status = "Refreshing rooms".to_string();
    commands.write(LobbyCommand::RefreshRooms);
}

/// PROMPT 1160 — turn a clicked room-list row into a `JoinRoom` command using
/// the server-supplied `requested_slot`. Mirrors the `Join` button path so the
/// existing `input.join_in_flight` latch and rejection surface (`apply_join_ack`
/// / `S2CJoinRejected` drainer) continue to apply.
///
/// PROMPT 1178 — `input.requested_slot` is now synced to the clicked row's
/// `first_open_slot` so the displayed own-slot label in the lobby cannot read
/// a stale value (e.g. the `LobbyInputState::default()` `requested_slot = 1`
/// after the user clicks a row whose only open seat is slot `2`). The
/// authoritative server-confirmed slot still lands through `S2CJoinAck` /
/// `S2CSlotUpdated` -> `lobby.slots`; this just keeps the optimistic
/// input-side mirror coherent until the ack lands.
pub fn request_join_room_from_row(
    row: &LobbyRoomListRow,
    input: &mut LobbyInputState,
    lobby: &mut LobbyViewState,
    commands: &mut MessageWriter<LobbyCommand>,
) {
    if input.join_in_flight {
        lobby.status = "Join already pending".to_string();
        return;
    }

    input.requested_slot = row.requested_slot;
    input.join_in_flight = true;
    lobby.status = format!("Joining {}", row.room_code);
    commands.write(LobbyCommand::JoinRoom {
        room_code: row.room_code.clone(),
        requested_slot: row.requested_slot,
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
                // PROMPT 1398 — body region. Owns every panel child ABOVE
                // the Confirm CTA so that when content density grows past
                // the panel's `max_height: 92%` content area, the body
                // (flex_grow:1, flex_shrink:1, min_height:0, overflow:
                // clip_y) absorbs the pressure. The Confirm CTA is the
                // next sibling at the panel level with `flex_shrink:0.0`,
                // so it stays anchored to the panel's bottom edge on
                // every supported viewport (1280×720, 1366×768, 1920×1080).
                panel
                    .spawn((
                        LobbyPanelBody,
                        Name::new("Lobby Panel Body"),
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            min_height: Val::Px(0.0),
                            row_gap: Val::Px(SPACING_MD),
                            overflow: Overflow::clip_y(),
                            ..default()
                        },
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
                                // PROMPT 1398 — the room-code chip is a read-only
                                // status label (not a Button). Mark it explicitly
                                // with the canonical `StatusChip` token so QA
                                // queries can distinguish chips from primary
                                // buttons at runtime.
                                StatusChip,
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

                        // PROMPT 1178 — section separator 1 (the SPACING_XL gap
                        // between the status/room-code group and the create-join
                        // row) is removed. The default `row_gap: SPACING_MD`
                        // panel rule still gives an air-gap between sections, and
                        // the saved 16 px of vertical real-estate buys part of
                        // the budget needed to keep the Confirm CTA inside the
                        // viewport at 1280×720 after the PROMPT 1160 existing-
                        // room browser landed.

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
                                CursorIcon::System(SystemCursorIcon::Pointer),
                                Text::new(lobby_dynamic_copy(
                                    LobbyDynamicText::Create,
                                    &lobby,
                                    &input,
                                )),
                                lobby_text_font(typography::BODY),
                                TextColor(Color::srgb(0.98, 0.93, 0.72)),
                                lobby_button_node(
                                    Val::Px(LOBBY_CREATE_BUTTON_WIDTH_PX),
                                    LOBBY_CREATE_BUTTON_HEIGHT_PX,
                                ),
                                BackgroundColor(LOBBY_CREATE_BUTTON_BG),
                                BorderColor::all(LOBBY_CREATE_BUTTON_BORDER),
                            ));
                            row.spawn((
                                LobbyJoinRoomButton,
                                LobbyDynamicText::Join,
                                Button,
                                Interaction::None,
                                CursorIcon::System(SystemCursorIcon::Pointer),
                                Text::new(lobby_dynamic_copy(
                                    LobbyDynamicText::Join,
                                    &lobby,
                                    &input,
                                )),
                                lobby_text_font(typography::BODY),
                                TextColor(Color::srgb(0.82, 0.95, 1.0)),
                                lobby_button_node(
                                    Val::Px(LOBBY_JOIN_BUTTON_WIDTH_PX),
                                    LOBBY_JOIN_BUTTON_HEIGHT_PX,
                                ),
                                BackgroundColor(LOBBY_JOIN_BUTTON_BG),
                                BorderColor::all(LOBBY_JOIN_BUTTON_BORDER),
                            ));
                        });

                        // PROMPT 1603 — debug-only `Create 2-Bot Soak Room`
                        // button. Spawned only when `CCGS_DEBUG_UI=1` AND
                        // the player is not yet in a session, so the
                        // affordance lives on the same surface as
                        // Create Room / Join Room and disappears once a
                        // room is owned. Production builds (env unset)
                        // never spawn the entity, so the control is
                        // invisible by default. Wired to
                        // `request_create_bot_room`, which writes
                        // `LobbyCommand::CreateBotRoom` -> `C2SCreateBotRoom`.
                        if lobby.session_id.is_none() && is_debug_ui_enabled() {
                            panel.spawn((lobby_row_node(),)).with_children(|row| {
                                row.spawn((
                                    LobbyCreateBotRoomButton,
                                    LobbyDynamicText::CreateBotRoom,
                                    Button,
                                    Interaction::None,
                                    Text::new(lobby_dynamic_copy(
                                        LobbyDynamicText::CreateBotRoom,
                                        &lobby,
                                        &input,
                                    )),
                                    lobby_text_font(typography::CAPTION),
                                    TextColor(Color::srgba(0.95, 0.78, 0.62, 0.95)),
                                    // Match Create Room width so the debug
                                    // affordance reads as a peer of the
                                    // primary CTAs without dominating the
                                    // row.
                                    lobby_button_node(
                                        Val::Px(LOBBY_CREATE_BUTTON_WIDTH_PX + 96.0),
                                        LOBBY_CREATE_BUTTON_HEIGHT_PX,
                                    ),
                                    BackgroundColor(Color::srgba(0.18, 0.12, 0.08, 0.95)),
                                    BorderColor::all(Color::srgb(0.58, 0.36, 0.22)),
                                    Name::new("Lobby Create Bot Room Button (debug)"),
                                ));
                            });
                        }

                        // PROMPT 1160 / PROMPT 1178 — existing-room browser section.
                        // Visible only before the local client has an active
                        // `session_id`. Once the player has joined or created a
                        // room, the browser becomes irrelevant: it can only list
                        // OTHER joinable rooms (the server filters out the local
                        // player's own room), and clicking another row at that
                        // point would race the `S2CJoinAck`. Hiding it post-join
                        // also reclaims ~84 px of panel content height (heading
                        // row 30 + intra-block row_gap SPACING_SM + room-list row
                        // 30 + the panel-level row_gap SPACING_MD before/after),
                        // which is part of the budget that keeps the Confirm CTA
                        // visible at the minimum 1280×720 viewport.
                        if lobby.session_id.is_none() {
                            panel
                                .spawn((
                                    Name::new("Lobby Existing Rooms Block"),
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(SPACING_SM),
                                        ..default()
                                    },
                                ))
                                .with_children(|block| {
                                    block.spawn((lobby_row_node(),)).with_children(|row| {
                                        row.spawn((
                                            Text::new("Existing rooms"),
                                            lobby_text_font(typography::H3),
                                            TextColor(Color::srgb(0.92, 0.95, 0.98)),
                                        ));
                                        row.spawn((
                                            LobbyRefreshRoomsButton,
                                            LobbyDynamicText::Refresh,
                                            Button,
                                            Interaction::None,
                                            Text::new(lobby_dynamic_copy(
                                                LobbyDynamicText::Refresh,
                                                &lobby,
                                                &input,
                                            )),
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
                                    block.spawn((
                                        LobbyRoomListContainer,
                                        Name::new("Lobby Room List Container"),
                                        Node {
                                            width: Val::Percent(100.0),
                                            flex_direction: FlexDirection::Column,
                                            row_gap: Val::Px(SPACING_SM),
                                            ..default()
                                        },
                                    ));
                                });
                        }

                        // PROMPT 1178 — `Requested slot` label + slot buttons are a
                        // manual-typed-Join seat preference, NOT a primary
                        // progression path after the player joined a room (either
                        // via the browser row click or via `Create Room`). They
                        // are now visible only when `session_id` is `None`, so
                        // they appear next to the typed-Join controls instead of
                        // squatting between the browser and the class picker
                        // after join — which read as "still need to pick a slot"
                        // and obscured the actual next step (Confirm class). When
                        // present, the row stays keyboard-reachable via the
                        // existing digit-key handlers and is consumed by
                        // `C2SJoinRoom { requested_slot }` for manual typed
                        // joins. Hiding this section post-join also reclaims
                        // ~81 px of panel content height, which restores
                        // Confirm-CTA visibility at the minimum 1280×720
                        // viewport.
                        if lobby.session_id.is_none() {
                            // Sprint 14 story 003 AC6: lobby labels are at least as
                            // large as the data they describe.
                            //
                            // PROMPT 1398 — make the leading word render as
                            // helper copy ("Pick a slot for manual join") so it
                            // reads as instructional text, NOT as a heading
                            // that competes with the gold Confirm CTA for the
                            // user's attention. AUDIT-1392-P04 noted that the
                            // legacy "Requested slot" label phrased like a
                            // button label even though the row's actual
                            // affordance is the four slot buttons immediately
                            // below it.
                            panel.spawn((
                                LobbyRequestedSlotLabel,
                                Text::new("Manual join seat preference:"),
                                lobby_text_font(typography::CAPTION),
                                TextColor(Color::srgba(0.78, 0.84, 0.92, 0.86)),
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
                        }

                        // PROMPT 1178 — section separator 2 (between the (now
                        // session-gated) Requested-slot row and the class picker)
                        // is removed. With both the browser and the slot picker
                        // hidden post-join, an extra SPACING_XL air-gap before
                        // the class picker would only widen the space the
                        // Confirm CTA still needs to fit inside.

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
                                    Text::new("Choose your class"),
                                    lobby_text_font(typography::H3),
                                    TextColor(Color::srgb(0.92, 0.95, 0.98)),
                                ));

                                class_picker
                                    .spawn((
                                        LobbySelectedClassIdentityPanel,
                                        StatusChip,
                                        Name::new("Lobby Selected Class Identity"),
                                        lobby_selected_class_identity_panel_node(),
                                        BackgroundColor(Color::srgba(0.12, 0.15, 0.18, 0.96)),
                                        BorderColor::all(Color::srgb(0.72, 0.60, 0.28)),
                                    ))
                                    .with_children(|identity| {
                                        identity.spawn((
                                            LobbySelectedClassIdentityPortrait,
                                            Name::new("Lobby Selected Class Portrait"),
                                            lobby_selected_class_portrait_node(),
                                            ImageNode::new(
                                                asset_server.load(lobby_portrait_asset(
                                                    input.selected_class,
                                                )),
                                            ),
                                        ));
                                        identity.spawn((
                                            LobbySelectedClassIdentityText,
                                            LobbyDynamicText::SelectedClassIdentity,
                                            Text::new(lobby_dynamic_copy(
                                                LobbyDynamicText::SelectedClassIdentity,
                                                &lobby,
                                                &input,
                                            )),
                                            lobby_text_font(typography::BODY),
                                            TextColor(Color::srgb(0.98, 0.94, 0.78)),
                                        ));
                                    });

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
                                            let (background, border) =
                                                lobby_class_picker_cell_colors(
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
                                                Name::new(format!(
                                                    "Lobby Class Cell {:?}",
                                                    class_id
                                                )),
                                                lobby_class_picker_cell_node(),
                                                background,
                                                border,
                                            ))
                                            .with_children(|cell| {
                                                cell.spawn((
                                                    LobbyClassPortrait { class_id },
                                                    Name::new(format!(
                                                        "Lobby Portrait {:?}",
                                                        class_id
                                                    )),
                                                    lobby_class_portrait_node(),
                                                    ImageNode::new(
                                                        asset_server
                                                            .load(lobby_portrait_asset(class_id)),
                                                    ),
                                                ))
                                                .with_children(|portrait| {
                                                    portrait.spawn((
                                                        LobbyClassPickerEmblem { class_id },
                                                        Name::new(format!(
                                                            "Lobby Class Emblem {:?}",
                                                            class_id
                                                        )),
                                                        lobby_class_picker_emblem_node(),
                                                        ImageNode::new(
                                                            asset_server.load(
                                                                class_type_icon_asset(class_id),
                                                            ),
                                                        ),
                                                    ));
                                                });
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
                                                    BackgroundColor(Color::srgba(
                                                        0.10, 0.13, 0.17, 0.95,
                                                    )),
                                                    BorderColor::all(Color::srgb(0.30, 0.38, 0.48)),
                                                ));
                                            });
                                        }

                                        for class_id in lobby_all_class_ids() {
                                            if lobby_class_options().contains(&class_id) {
                                                continue;
                                            }
                                            let (background, border) =
                                                lobby_class_picker_cell_colors(
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
                                                Name::new(format!(
                                                    "Lobby Class Cell {:?}",
                                                    class_id
                                                )),
                                                lobby_class_picker_cell_node(),
                                                background,
                                                border,
                                            ))
                                            .with_children(|cell| {
                                                cell.spawn((
                                                    LobbyClassPortrait { class_id },
                                                    Name::new(format!(
                                                        "Lobby Portrait {:?}",
                                                        class_id
                                                    )),
                                                    lobby_class_portrait_node(),
                                                    ImageNode::new(
                                                        asset_server
                                                            .load(lobby_portrait_asset(class_id)),
                                                    ),
                                                ))
                                                .with_children(|portrait| {
                                                    portrait.spawn((
                                                        LobbyClassPickerEmblem { class_id },
                                                        Name::new(format!(
                                                            "Lobby Class Emblem {:?}",
                                                            class_id
                                                        )),
                                                        lobby_class_picker_emblem_node(),
                                                        ImageNode::new(
                                                            asset_server.load(
                                                                class_type_icon_asset(class_id),
                                                            ),
                                                        ),
                                                    ));
                                                });
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
                        //
                        // PROMPT 1178 — the `ImageNode.color` field tints the panel
                        // asset down to a low-saturation greyish-blue so the chip
                        // reads as informational status, not as a primary click
                        // target. The two panels carry NO `Button` / `Interaction`
                        // markers (they have not in any prior revision either) so
                        // they remain non-interactive at the ECS level; the tint
                        // closes the visual gap. The PROMPT 1138 chrome-wiring
                        // contract (non-default `ImageNode.image` handle sourced
                        // from `LOBBY_PLAYER_SLOT_PANEL_ASSET`) is preserved.
                        panel.spawn((lobby_row_node(),)).with_children(|row| {
                            row.spawn((
                                LobbyOwnSlotPanel,
                                // PROMPT 1398 — the slot panel is a read-only
                                // status chip that announces "you · class · slot
                                // N". The visible glyph reads like the gold CTA
                                // when the user scans the panel from the top
                                // (AUDIT-1392-P04 "you are slot 1 looks like a
                                // button"); the `StatusChip` marker codifies the
                                // read-only role at the ECS level so QA queries
                                // and accessibility tooling can distinguish it
                                // from primary actions.
                                StatusChip,
                                Name::new("Lobby Own Slot Panel"),
                                Node {
                                    width: Val::Px(LOBBY_SLOT_PANEL_WIDTH_PX),
                                    height: Val::Px(LOBBY_SLOT_PANEL_HEIGHT_PX),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::horizontal(Val::Px(SPACING_SM)),
                                    ..default()
                                },
                                lobby_slot_chip_image_node(
                                    asset_server.load(LOBBY_PLAYER_SLOT_PANEL_ASSET),
                                ),
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
                                    // PROMPT 1398 — drop slot-panel text to
                                    // `CAPTION` so it reads as a secondary label
                                    // (paired with the muted tint), not a
                                    // button-sized headline. Friend-game scope:
                                    // the slot chip is informational only.
                                    lobby_text_font(typography::CAPTION),
                                    TextColor(Color::srgb(0.95, 0.97, 1.0)),
                                ));
                            });
                            row.spawn((
                                LobbyOpponentSlotPanel,
                                // PROMPT 1398 — opponent-slot status chip; see
                                // own-slot panel for rationale.
                                StatusChip,
                                Name::new("Lobby Opponent Slot Panel"),
                                Node {
                                    width: Val::Px(LOBBY_SLOT_PANEL_WIDTH_PX),
                                    height: Val::Px(LOBBY_SLOT_PANEL_HEIGHT_PX),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::horizontal(Val::Px(SPACING_SM)),
                                    ..default()
                                },
                                lobby_slot_chip_image_node(
                                    asset_server.load(LOBBY_PLAYER_SLOT_PANEL_ASSET),
                                ),
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
                                    lobby_text_font(typography::CAPTION),
                                    TextColor(Color::srgb(0.86, 0.90, 0.96)),
                                ));
                            });
                        });

                        // Section 4-bis — PROMPT 1596 contextual bot controls
                        // (BOT-FLOW-LOBBY-ADD-REMOVE-BOT-UX). Empty container
                        // spawned once here; `refresh_lobby_ui_system`
                        // repopulates the row whenever `LobbyViewState`
                        // changes so the Add Bot / Remove Bot affordances
                        // always match the authoritative `lobby.slots`
                        // snapshot. The row collapses to zero height while
                        // no controls are eligible (pre-room and
                        // human-vs-human seatings), so it adds no
                        // viewport-budget pressure on the Confirm CTA.
                        panel.spawn((
                            LobbyBotControlsContainer,
                            Name::new("Lobby Bot Controls"),
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(SPACING_SM),
                                align_items: AlignItems::Center,
                                flex_shrink: 0.0,
                                ..default()
                            },
                        ));
                    });

                // Section 5 — confirm CTA. Last DIRECT child of the
                // panel per AC3(e) read order; PROMPT 802 §3.1 L4
                // "portraits / slot panels / room-code chip render below
                // confirm" inversion is resolved by placing the CTA last
                // at the panel level (immediately after the
                // [`LobbyPanelBody`] body region above).
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
                //
                // PROMPT 1398: `margin.top = SPACING_XL - SPACING_MD`
                // preserves the legacy "section separator" air-gap
                // (`SPACING_XL` total cumulative gap counting the
                // panel's `row_gap: SPACING_MD`) between the body
                // region's last child and the CTA, without needing a
                // zero-height separator entity. The pair (body region
                // with `flex_grow: 1.0`, CTA with `flex_shrink: 0.0`)
                // is what structurally anchors the CTA to the panel
                // bottom — the margin is just the chrome.
                let initial_state =
                    lobby_confirm_button_style_state(&lobby, &input, Interaction::None);
                let (BackgroundColor(initial_bg), initial_border, initial_text_color) =
                    lobby_confirm_button_colors(initial_state);
                panel.spawn((
                    LobbyConfirmClassButton,
                    LobbyDynamicText::Confirm,
                    Button,
                    Interaction::None,
                    CursorIcon::System(SystemCursorIcon::Pointer),
                    Text::new(lobby_dynamic_copy(
                        LobbyDynamicText::Confirm,
                        &lobby,
                        &input,
                    )),
                    lobby_text_font(typography::BODY),
                    initial_text_color,
                    Node {
                        flex_shrink: 0.0,
                        margin: UiRect {
                            top: Val::Px(SPACING_XL - SPACING_MD),
                            ..default()
                        },
                        ..lobby_button_node(
                            Val::Percent(LOBBY_CONFIRM_BUTTON_WIDTH_PERCENT),
                            LOBBY_CONFIRM_BUTTON_HEIGHT_PX,
                        )
                    },
                    BackgroundColor(initial_bg),
                    initial_border,
                ));
            });
        });
}

fn refresh_lobby_ui_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    lobby: Res<LobbyViewState>,
    input: Res<LobbyInputState>,
    mut texts: Query<&mut Text, With<LobbyStatusText>>,
    mut dynamic_texts: Query<(&LobbyDynamicText, &mut Text), Without<LobbyStatusText>>,
    mut class_cells: Query<(
        &LobbyClassPickerCell,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    mut selected_portraits: Query<&mut ImageNode, With<LobbySelectedClassIdentityPortrait>>,
    room_list_container: Query<Entity, With<LobbyRoomListContainer>>,
    bot_controls_container: Query<Entity, With<LobbyBotControlsContainer>>,
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

    for mut portrait in &mut selected_portraits {
        portrait.image = asset_server.load(lobby_portrait_asset(input.selected_class));
    }

    if lobby.is_changed() {
        for container in &room_list_container {
            rebuild_room_list_rows(&mut commands, container, &lobby);
        }
        for container in &bot_controls_container {
            rebuild_bot_controls_rows(&mut commands, container, &lobby);
        }
    }
}

/// PROMPT 1596 — re-populate the contextual bot-controls container with one
/// Add/Remove button per eligible non-local slot. Mirrors
/// `rebuild_room_list_rows`: always despawns descendants first so successive
/// `S2CSlotUpdated` payloads do not accumulate stale buttons. When no slot
/// is eligible (pre-room, fully human-occupied), the container is left empty
/// and collapses to zero height (`flex` with no children + `Auto` height).
fn rebuild_bot_controls_rows(
    commands: &mut Commands,
    container_entity: Entity,
    lobby: &LobbyViewState,
) {
    commands
        .entity(container_entity)
        .despawn_related::<Children>();

    let controls = lobby_bot_controls_for_slots(&lobby.slots, lobby.local_player_id);
    if controls.is_empty() {
        return;
    }

    commands.entity(container_entity).with_children(|parent| {
        for control in controls {
            match control {
                LobbyBotSlotControl::AddBot { slot } => {
                    parent.spawn((
                        LobbyAddBotButton { slot },
                        LobbyDynamicText::AddBot(slot),
                        Button,
                        Interaction::None,
                        Text::new(format!("Add Bot (seat {slot})")),
                        lobby_text_font(typography::BODY),
                        TextColor(Color::srgb(0.98, 0.93, 0.72)),
                        lobby_button_node(
                            Val::Px(LOBBY_JOIN_BUTTON_WIDTH_PX),
                            LOBBY_BUTTON_HEIGHT_PX,
                        ),
                        BackgroundColor(Color::srgba(0.17, 0.18, 0.14, 0.95)),
                        BorderColor::all(Color::srgb(0.65, 0.53, 0.24)),
                    ));
                }
                LobbyBotSlotControl::RemoveBot { slot } => {
                    parent.spawn((
                        LobbyRemoveBotButton { slot },
                        LobbyDynamicText::RemoveBot(slot),
                        Button,
                        Interaction::None,
                        Text::new(format!("Remove Bot (seat {slot})")),
                        lobby_text_font(typography::BODY),
                        TextColor(Color::srgb(0.95, 0.82, 0.78)),
                        lobby_button_node(
                            Val::Px(LOBBY_JOIN_BUTTON_WIDTH_PX),
                            LOBBY_BUTTON_HEIGHT_PX,
                        ),
                        BackgroundColor(Color::srgba(0.22, 0.13, 0.13, 0.95)),
                        BorderColor::all(Color::srgb(0.66, 0.34, 0.30)),
                    ));
                }
            }
        }
    });
}

/// PROMPT 1160 — re-populate the room-list container with one entity per row.
/// Always despawns descendants first so successive `S2CRoomList` payloads do
/// not accumulate stale rows. When the list is empty the helper renders a
/// single `RoomListEmpty` label so the panel never appears as a blank slab.
fn rebuild_room_list_rows(
    commands: &mut Commands,
    container_entity: Entity,
    lobby: &LobbyViewState,
) {
    commands
        .entity(container_entity)
        .despawn_related::<Children>();

    if lobby.room_list.is_empty() {
        commands.entity(container_entity).with_children(|parent| {
            parent.spawn((
                LobbyDynamicText::RoomListEmpty,
                Text::new("No joinable rooms. Create a room to host."),
                lobby_text_font(typography::BODY),
                TextColor(Color::srgba(0.78, 0.84, 0.92, 0.86)),
            ));
        });
        return;
    }

    commands.entity(container_entity).with_children(|parent| {
        for entry in &lobby.room_list {
            let label = format_room_list_row_label(entry);
            match entry.first_open_slot {
                Some(slot) => {
                    parent.spawn((
                        LobbyRoomListRow {
                            room_code: entry.room_code.clone(),
                            requested_slot: slot,
                        },
                        Button,
                        Interaction::None,
                        Text::new(label),
                        lobby_text_font(typography::BODY),
                        TextColor(Color::srgb(0.92, 0.95, 0.98)),
                        lobby_button_node(Val::Percent(100.0), LOBBY_BUTTON_HEIGHT_PX),
                        BackgroundColor(Color::srgba(0.11, 0.15, 0.20, 0.95)),
                        BorderColor::all(Color::srgb(0.28, 0.56, 0.72)),
                    ));
                }
                None => {
                    // Defensive: server filters out full rooms, but if one
                    // slips through we render it as a non-interactive label
                    // (contract addendum: never produce a JoinRoom with no
                    // open slot).
                    parent.spawn((
                        Text::new(label),
                        lobby_text_font(typography::BODY),
                        TextColor(Color::srgba(0.74, 0.80, 0.86, 0.74)),
                    ));
                }
            }
        }
    });
}

/// PROMPT 1160 — row text builder. Format chosen to read at a glance:
/// "ABCDEF · OneVOne · 1/2". The leading code matches the typed-Join surface so
/// the player can correlate browser rows with the room-code chip.
pub fn format_room_list_row_label(entry: &RoomListEntry) -> String {
    let seat = entry
        .first_open_slot
        .map(|slot| format!("seat {slot} open"))
        .unwrap_or_else(|| "full".to_string());
    format!(
        "Join {} - {:?} - {}/{} players - {}",
        entry.room_code, entry.mode, entry.slots_filled, entry.slots_max, seat
    )
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
            format!("Type room code: {rendered_code} - {focus}")
        }
        LobbyDynamicText::Slot(slot) => {
            if input.requested_slot == slot {
                format!("Seat {slot} *")
            } else {
                format!("Seat {slot}")
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
        LobbyDynamicText::Refresh => "Refresh Rooms".to_string(),
        LobbyDynamicText::RoomListEmpty => {
            if lobby.room_list.is_empty() {
                "No joinable rooms. Create a room to host.".to_string()
            } else {
                String::new()
            }
        }
        LobbyDynamicText::SelectedClassIdentity => lobby_selected_class_identity_text(lobby, input),
        LobbyDynamicText::AddBot(slot) => format!("Add Bot (seat {slot})"),
        LobbyDynamicText::RemoveBot(slot) => format!("Remove Bot (seat {slot})"),
        LobbyDynamicText::CreateBotRoom => "Create 2-Bot Soak Room".to_string(),
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
///
/// PROMPT 1178 — the displayed slot index now prefers the
/// server-authoritative `lobby.slots[local_player_id].slot` when both are
/// present, falling back to `input.requested_slot` only before the join
/// acknowledgement has landed. This closes the gap where a row-click
/// join into a different slot (e.g. row says `first_open_slot = 2`)
/// could otherwise read "You · X · slot 1" because `LobbyInputState`
/// defaulted to `requested_slot = 1`. The fallback path is kept so the
/// label still reads sensibly during the pre-handshake / pre-join
/// window where `slots` is empty.
pub fn lobby_own_slot_label_text(lobby: &LobbyViewState, input: &LobbyInputState) -> String {
    let confirmed_marker = if lobby.locked_class.is_some() {
        " *"
    } else {
        ""
    };
    let slot_index = lobby
        .local_player_id
        .and_then(|local| {
            lobby
                .slots
                .iter()
                .find(|s| s.player_id == Some(local))
                .map(|s| s.slot)
        })
        .unwrap_or(input.requested_slot);
    let confirm_state = if lobby.locked_class.is_some() {
        "confirmed"
    } else {
        "not confirmed"
    };
    format!(
        "You - {:?}{} - slot {} - {}",
        input.selected_class, confirmed_marker, slot_index, confirm_state
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
        Some(class) => format!("Opponent - {class}"),
        None => "Opponent - waiting for player".to_string(),
    }
}

pub fn lobby_selected_class_identity_text(
    lobby: &LobbyViewState,
    input: &LobbyInputState,
) -> String {
    if let Some(locked_class) = lobby.locked_class {
        return format!("Confirmed: {locked_class:?}\nWaiting for opponent");
    }

    if lobby.session_id.is_none() {
        format!(
            "Selected: {:?}\nCreate or join a room, then confirm",
            input.selected_class
        )
    } else {
        format!(
            "Selected: {:?}\nConfirm this class to continue",
            input.selected_class
        )
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
        // UI-1280: allow cells to compress proportionally at narrow viewports
        // (e.g. 1280×720) where the 7×108px grid fits with only a 4px margin.
        // flex_shrink=1 (CSS default) lets Taffy absorb pixel-rounding deficits
        // without hard-overflowing the 7th cell past the panel edge.
        // Cells keep their preferred width at all well-sized viewports.
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

fn lobby_selected_class_identity_panel_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Px(LOBBY_SELECTED_CLASS_PANEL_HEIGHT_PX),
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(SPACING_MD),
        padding: UiRect::all(Val::Px(SPACING_SM)),
        border: UiRect::all(Val::Px(2.0)),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexStart,
        flex_shrink: 0.0,
        ..default()
    }
}

fn lobby_selected_class_portrait_node() -> Node {
    Node {
        width: Val::Px(LOBBY_SELECTED_CLASS_PORTRAIT_WIDTH_PX),
        height: Val::Px(LOBBY_SELECTED_CLASS_PORTRAIT_HEIGHT_PX),
        flex_shrink: 0.0,
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

/// PROMPT 1178 — build a desaturated `ImageNode` for the lobby slot-panel
/// chips so they read as informational status, not as primary buttons.
///
/// The panel's PNG asset (`ui_player_slot_panel.png`) is a saturated
/// blue card placeholder; rendered at full color it competes with the
/// gold-accent primary Confirm CTA for the player's attention and
/// reads as "click me". Tinting `ImageNode.color` desaturates the
/// chrome to a muted slate-grey, which is the visual cue the §3.1 L4
/// hierarchy uses for "informational chip / readout / status panel".
/// The `image` handle is preserved verbatim so the existing PROMPT
/// 1138 chrome-wiring contract
/// (`tests/integration/session/lobby_chrome_wiring_test.rs`) — which
/// asserts a non-default `ImageNode.image` handle for both slot
/// panels — continues to pass.
pub fn lobby_slot_chip_image_node(image: Handle<Image>) -> ImageNode {
    ImageNode::new(image).with_color(Color::srgba(0.62, 0.68, 0.78, 0.70))
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
        S::Hovered => {
            // White overlay at HOVER_BG_TINT_ALPHA on the Enabled base
            // (0.949, 0.788, 0.298). Border alpha = HOVER_BORDER_ALPHA.
            let wh = |b: f32| b * (1.0 - HOVER_BG_TINT_ALPHA) + HOVER_BG_TINT_ALPHA;
            (
                BackgroundColor(Color::srgb(wh(0.949), wh(0.788), wh(0.298))),
                BorderColor::all(Color::srgba(1.00, 0.94, 0.62, HOVER_BORDER_ALPHA)),
                TextColor(Color::srgb(0.06, 0.05, 0.02)),
            )
        }
        S::Pressed => {
            // Black overlay at PRESSED_BG_TINT_ALPHA on the Enabled base.
            let dk = |b: f32| b * (1.0 - PRESSED_BG_TINT_ALPHA);
            (
                BackgroundColor(Color::srgb(dk(0.949), dk(0.788), dk(0.298))),
                BorderColor::all(Color::srgb(1.00, 0.90, 0.50)),
                TextColor(Color::srgb(0.04, 0.03, 0.01)),
            )
        }
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

// ---------------------------------------------------------------------------
// Wave-2 interaction-state migration helpers (S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001)
// ---------------------------------------------------------------------------

/// Compute overlay-tinted `BackgroundColor` / `BorderColor` from a base
/// color pair and the current `Interaction` state, using the canonical
/// `interaction_states::*` alpha tokens as the tint magnitudes.
///
/// - `None` → base colors unchanged (Default band).
/// - `Hovered` → white overlay at `HOVER_BG_TINT_ALPHA`; border alpha set
///   to `HOVER_BORDER_ALPHA`.
/// - `Pressed` → black overlay at `PRESSED_BG_TINT_ALPHA`; border unchanged.
fn apply_interaction_tint(
    base_bg: Color,
    base_border: Color,
    interaction: Interaction,
) -> (BackgroundColor, BorderColor) {
    let bg = base_bg.to_srgba();
    let border = base_border.to_srgba();
    match interaction {
        Interaction::None => (BackgroundColor(base_bg), BorderColor::all(base_border)),
        Interaction::Hovered => (
            BackgroundColor(Color::srgba(
                bg.red * (1.0 - HOVER_BG_TINT_ALPHA) + HOVER_BG_TINT_ALPHA,
                bg.green * (1.0 - HOVER_BG_TINT_ALPHA) + HOVER_BG_TINT_ALPHA,
                bg.blue * (1.0 - HOVER_BG_TINT_ALPHA) + HOVER_BG_TINT_ALPHA,
                bg.alpha,
            )),
            BorderColor::all(Color::srgba(
                border.red,
                border.green,
                border.blue,
                HOVER_BORDER_ALPHA,
            )),
        ),
        Interaction::Pressed => (
            BackgroundColor(Color::srgba(
                bg.red * (1.0 - PRESSED_BG_TINT_ALPHA),
                bg.green * (1.0 - PRESSED_BG_TINT_ALPHA),
                bg.blue * (1.0 - PRESSED_BG_TINT_ALPHA),
                bg.alpha,
            )),
            BorderColor::all(base_border),
        ),
    }
}

/// Apply canonical 4-state overlay tints to `LobbyCreateRoomButton` and
/// `LobbyJoinRoomButton` on `Interaction` change. Fires only when the
/// pointer interaction changes; no per-frame cost at steady state.
pub fn lobby_create_join_interaction_overlay_system(
    mut query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            Option<&LobbyCreateRoomButton>,
        ),
        (
            Changed<Interaction>,
            Or<(With<LobbyCreateRoomButton>, With<LobbyJoinRoomButton>)>,
        ),
    >,
) {
    for (interaction, mut bg, mut border, create) in &mut query {
        let (base_bg, base_border) = if create.is_some() {
            (LOBBY_CREATE_BUTTON_BG, LOBBY_CREATE_BUTTON_BORDER)
        } else {
            (LOBBY_JOIN_BUTTON_BG, LOBBY_JOIN_BUTTON_BORDER)
        };
        let (new_bg, new_border) = apply_interaction_tint(base_bg, base_border, *interaction);
        *bg = new_bg;
        *border = new_border;
    }
}

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
