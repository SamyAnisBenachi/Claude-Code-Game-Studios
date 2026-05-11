use bevy::prelude::*;
use lightyear::prelude::MessageSender;
use serde::{Deserialize, Serialize};
use shared::protocol::{
    C2SSetPlacementTimerMultiplier, PlacementTimerMultiplier, ReliableChannel, RoundPhase,
};

use crate::state::{CurrentClientPhase, SessionSettingsView};

pub const ACCESSIBILITY_PREFERENCES_STORAGE_KEY: &str =
    "lanes_and_lies.accessibility_preferences.v1";
pub const ACCESSIBILITY_PREFERENCES_PAYLOAD_VERSION: u32 = 1;
pub const UI_SCALE_MIN_PERCENT: u8 = 75;
pub const UI_SCALE_MAX_PERCENT: u8 = 150;
pub const UI_SCALE_STEP_PERCENT: u8 = 25;
pub const SETTINGS_PANEL_BASE_WIDTH_PX: f32 = 760.0;
pub const SETTINGS_PANEL_BASE_HEIGHT_PX: f32 = 520.0;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorblindMode {
    #[default]
    Off,
    Protanopia,
    Deuteranopia,
    Tritanopia,
}

impl ColorblindMode {
    pub const VALUES: [Self; 4] = [
        Self::Off,
        Self::Protanopia,
        Self::Deuteranopia,
        Self::Tritanopia,
    ];

    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Protanopia,
            Self::Protanopia => Self::Deuteranopia,
            Self::Deuteranopia => Self::Tritanopia,
            Self::Tritanopia => Self::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Protanopia => "Protanopia",
            Self::Deuteranopia => "Deuteranopia",
            Self::Tritanopia => "Tritanopia",
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibilityPreferences {
    pub colorblind_mode: ColorblindMode,
    pub reduced_motion: bool,
    pub placement_timer_multiplier_request: PlacementTimerMultiplier,
    pub menu_ui_scale_percent: u8,
    pub hud_ui_scale_percent: u8,
}

impl Default for AccessibilityPreferences {
    fn default() -> Self {
        Self {
            colorblind_mode: ColorblindMode::Off,
            reduced_motion: false,
            placement_timer_multiplier_request: PlacementTimerMultiplier::X1,
            menu_ui_scale_percent: 100,
            hud_ui_scale_percent: 100,
        }
    }
}

impl AccessibilityPreferences {
    pub fn set_menu_ui_scale_percent(&mut self, percent: u8) {
        self.menu_ui_scale_percent = clamp_ui_scale_percent(percent);
    }

    pub fn set_hud_ui_scale_percent(&mut self, percent: u8) {
        self.hud_ui_scale_percent = clamp_ui_scale_percent(percent);
    }

    pub fn try_set_placement_timer_multiplier_from_ratio(
        &mut self,
        numerator: u32,
        denominator: u32,
    ) -> Result<(), PreferenceValidationError> {
        let Some(multiplier) =
            PlacementTimerMultiplier::from_standard_ratio(numerator, denominator)
        else {
            return Err(PreferenceValidationError::InvalidPlacementTimerMultiplier);
        };

        self.placement_timer_multiplier_request = multiplier;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferenceValidationError {
    InvalidPlacementTimerMultiplier,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibilityPreferencesPayload {
    pub version: u32,
    pub preferences: AccessibilityPreferences,
}

impl From<AccessibilityPreferences> for AccessibilityPreferencesPayload {
    fn from(preferences: AccessibilityPreferences) -> Self {
        Self {
            version: ACCESSIBILITY_PREFERENCES_PAYLOAD_VERSION,
            preferences,
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub enum PreferenceStorageBackend {
    BrowserLocalStorage,
    InMemory { payload: Option<String> },
    Unavailable,
    FailWrites { payload: Option<String> },
}

impl Default for PreferenceStorageBackend {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::BrowserLocalStorage
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::InMemory { payload: None }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreferenceStorageError {
    Unavailable,
    WriteFailed,
    EncodeFailed,
    DecodeFailed,
}

impl PreferenceStorageBackend {
    pub fn load(&self) -> Result<Option<AccessibilityPreferences>, PreferenceStorageError> {
        let Some(payload) = self.load_payload()? else {
            return Ok(None);
        };

        decode_preferences_payload(&payload).map(Some)
    }

    pub fn save(
        &mut self,
        preferences: AccessibilityPreferences,
    ) -> Result<(), PreferenceStorageError> {
        let payload = encode_preferences_payload(preferences)?;
        self.save_payload(payload)
    }

    fn load_payload(&self) -> Result<Option<String>, PreferenceStorageError> {
        match self {
            Self::BrowserLocalStorage => load_browser_storage_payload(),
            Self::InMemory { payload } | Self::FailWrites { payload } => Ok(payload.clone()),
            Self::Unavailable => Ok(None),
        }
    }

    fn save_payload(&mut self, payload: String) -> Result<(), PreferenceStorageError> {
        match self {
            Self::BrowserLocalStorage => save_browser_storage_payload(&payload),
            Self::InMemory {
                payload: stored_payload,
            } => {
                *stored_payload = Some(payload);
                Ok(())
            }
            Self::FailWrites { .. } | Self::Unavailable => Err(PreferenceStorageError::WriteFailed),
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn load_browser_storage_payload() -> Result<Option<String>, PreferenceStorageError> {
    let storage = web_sys::window()
        .ok_or(PreferenceStorageError::Unavailable)?
        .local_storage()
        .map_err(|_| PreferenceStorageError::Unavailable)?
        .ok_or(PreferenceStorageError::Unavailable)?;

    storage
        .get_item(ACCESSIBILITY_PREFERENCES_STORAGE_KEY)
        .map_err(|_| PreferenceStorageError::Unavailable)
}

#[cfg(not(target_arch = "wasm32"))]
fn load_browser_storage_payload() -> Result<Option<String>, PreferenceStorageError> {
    Err(PreferenceStorageError::Unavailable)
}

#[cfg(target_arch = "wasm32")]
fn save_browser_storage_payload(payload: &str) -> Result<(), PreferenceStorageError> {
    let storage = web_sys::window()
        .ok_or(PreferenceStorageError::Unavailable)?
        .local_storage()
        .map_err(|_| PreferenceStorageError::Unavailable)?
        .ok_or(PreferenceStorageError::Unavailable)?;

    storage
        .set_item(ACCESSIBILITY_PREFERENCES_STORAGE_KEY, payload)
        .map_err(|_| PreferenceStorageError::WriteFailed)
}

#[cfg(not(target_arch = "wasm32"))]
fn save_browser_storage_payload(_payload: &str) -> Result<(), PreferenceStorageError> {
    Err(PreferenceStorageError::Unavailable)
}

pub fn encode_preferences_payload(
    preferences: AccessibilityPreferences,
) -> Result<String, PreferenceStorageError> {
    serde_json::to_string(&AccessibilityPreferencesPayload::from(preferences))
        .map_err(|_| PreferenceStorageError::EncodeFailed)
}

pub fn decode_preferences_payload(
    payload: &str,
) -> Result<AccessibilityPreferences, PreferenceStorageError> {
    let decoded: AccessibilityPreferencesPayload =
        serde_json::from_str(payload).map_err(|_| PreferenceStorageError::DecodeFailed)?;

    if decoded.version != ACCESSIBILITY_PREFERENCES_PAYLOAD_VERSION {
        return Err(PreferenceStorageError::DecodeFailed);
    }

    Ok(decoded.preferences)
}

pub fn clamp_ui_scale_percent(percent: u8) -> u8 {
    percent.clamp(UI_SCALE_MIN_PERCENT, UI_SCALE_MAX_PERCENT)
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsSystemSet {
    Entry,
    Input,
    StateSync,
}

pub struct SettingsAccessibilityPlugin;

impl Plugin for SettingsAccessibilityPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("SettingsAccessibilityPlugin loaded");
        app.init_resource::<CurrentClientPhase>()
            .init_resource::<SessionSettingsView>()
            .init_resource::<AccessibilityPreferences>()
            .init_resource::<PreferenceStorageBackend>()
            .init_resource::<SettingsPanelState>()
            .init_resource::<SettingsStatusFooterState>()
            .init_resource::<SettingsFocusOrder>()
            .init_resource::<SettingsOutboundMessages>()
            .init_resource::<SettingsSessionLifecycle>()
            .init_resource::<SettingsPendingRequest>()
            .add_message::<SettingsOpenRequested>()
            .add_message::<SettingsCloseRequested>()
            .add_message::<SettingsActionRequest>()
            .configure_sets(
                Update,
                (
                    SettingsSystemSet::Entry,
                    SettingsSystemSet::Input,
                    SettingsSystemSet::StateSync,
                )
                    .chain(),
            )
            .add_systems(
                Startup,
                (load_accessibility_preferences_system, spawn_settings_shell).chain(),
            )
            .add_systems(
                Update,
                (
                    track_settings_session_ready_system,
                    handle_settings_open_requests_system,
                    handle_settings_close_requests_system,
                    settings_control_interaction_system,
                    settings_keyboard_input_system,
                    handle_settings_actions_system,
                    open_pending_settings_request_at_safe_boundary_system,
                    sync_settings_shell_visibility_system,
                    sync_settings_text_system,
                    sync_settings_focus_indicator_system,
                )
                    .chain(),
            );
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SettingsEntrySource {
    #[default]
    Title,
    Lobby,
    SafeInGame,
    GameOver,
    Help,
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SettingsPanelState {
    pub mode: SettingsPanelMode,
    pub source: Option<SettingsEntrySource>,
    pub last_closed_source: Option<SettingsEntrySource>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPanelMode {
    #[default]
    Closed,
    Open,
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SettingsPendingRequest {
    pub source: Option<SettingsEntrySource>,
    pub requested_during_phase: Option<RoundPhase>,
}

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct SettingsStatusFooterState {
    pub save_warning: bool,
    pub message: String,
}

impl SettingsStatusFooterState {
    fn set_saved(&mut self) {
        self.save_warning = false;
        self.message.clear();
        self.message.push_str("Preferences saved");
    }

    fn set_save_warning(&mut self) {
        self.save_warning = true;
        self.message.clear();
        self.message
            .push_str("Preferences active for this session; save failed");
    }
}

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct SettingsFocusOrder {
    pub entities: Vec<Entity>,
    pub focused_index: usize,
}

impl SettingsFocusOrder {
    pub fn focused_entity(&self) -> Option<Entity> {
        self.entities.get(self.focused_index).copied()
    }

    fn set_entities(&mut self, entities: Vec<Entity>) {
        self.entities = entities;
        if self.entities.is_empty() {
            self.focused_index = 0;
        } else {
            self.focused_index = self.focused_index.min(self.entities.len() - 1);
        }
    }

    fn focus_next(&mut self) {
        if self.entities.is_empty() {
            self.focused_index = 0;
            return;
        }

        self.focused_index = (self.focused_index + 1) % self.entities.len();
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct SettingsOutboundMessages {
    pub placement_timer_requests: Vec<C2SSetPlacementTimerMultiplier>,
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SettingsSessionLifecycle {
    pub session_ready: bool,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct SettingsEntities {
    pub root: Entity,
    pub panel: Entity,
    pub back_close_button: Entity,
    pub category_accessibility: Entity,
    pub content_pane: Entity,
    pub colorblind_selector: Entity,
    pub reduced_motion_toggle: Entity,
    pub timer_options: [Entity; 4],
    pub effective_timer_text: Entity,
    pub menu_scale_control: Entity,
    pub hud_scale_control: Entity,
    pub status_footer: Entity,
    pub footer_close_button: Entity,
}

impl SettingsEntities {
    fn visible_focus_order(self) -> Vec<Entity> {
        let mut order = Vec::with_capacity(11);
        order.push(self.back_close_button);
        order.push(self.category_accessibility);
        order.push(self.colorblind_selector);
        order.push(self.reduced_motion_toggle);
        order.extend(self.timer_options);
        order.push(self.menu_scale_control);
        order.push(self.hud_scale_control);
        order.push(self.footer_close_button);
        order
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsRoot;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsPanel;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsBackCloseButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsCategoryNav;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsContentPane;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsStatusFooter;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsFooterAction;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsColorblindSelector;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsReducedMotionToggle;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsTimerSelector;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsMenuUiScaleControl;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsHudUiScaleControl;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsFocusIndicator;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct SettingsMenuScaleApplied {
    pub percent: u8,
    pub factor: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsTimerOption {
    pub multiplier: PlacementTimerMultiplier,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsEffectiveTimerDisplay {
    pub multiplier: PlacementTimerMultiplier,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    Accessibility,
}

#[derive(Component, Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsOpenRequested {
    pub source: SettingsEntrySource,
}

#[derive(Component, Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsCloseRequested;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsActionRequest {
    pub action: SettingsControlAction,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsControlActionMarker(pub SettingsControlAction);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsControlAction {
    Close,
    SelectCategory(SettingsCategory),
    CycleColorblindMode,
    ToggleReducedMotion,
    SelectPlacementTimer(PlacementTimerMultiplier),
    CycleMenuUiScale,
    CycleHudUiScale,
}

pub fn load_accessibility_preferences_system(
    storage: Res<PreferenceStorageBackend>,
    mut preferences: ResMut<AccessibilityPreferences>,
    mut status: ResMut<SettingsStatusFooterState>,
) {
    match storage.load() {
        Ok(Some(loaded_preferences)) => {
            *preferences = loaded_preferences;
            status.message.clear();
        }
        Ok(None) => {
            if matches!(*storage, PreferenceStorageBackend::Unavailable) {
                status.set_save_warning();
            }
        }
        Err(_) => status.set_save_warning(),
    }
}

pub fn spawn_settings_shell(
    mut commands: Commands,
    existing: Option<Res<SettingsEntities>>,
    preferences: Res<AccessibilityPreferences>,
    settings_view: Res<SessionSettingsView>,
) {
    if existing.is_some() {
        return;
    }

    let root = commands
        .spawn((
            Name::new("Settings Accessibility Root"),
            SettingsRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.025, 0.03, 0.70)),
            Visibility::Hidden,
        ))
        .id();

    let panel = commands
        .spawn((
            Name::new("Settings Accessibility Panel"),
            SettingsPanel,
            SettingsMenuScaleApplied {
                percent: preferences.menu_ui_scale_percent,
                factor: menu_ui_scale_factor(preferences.menu_ui_scale_percent),
            },
            settings_panel_node(preferences.menu_ui_scale_percent),
            BackgroundColor(Color::srgb(0.08, 0.09, 0.11)),
            BorderColor::all(Color::srgb(0.72, 0.78, 0.86)),
            Visibility::Hidden,
            ChildOf(root),
        ))
        .id();

    let back_close_button = spawn_text_control(
        &mut commands,
        panel,
        "Settings Back Close",
        "Back / Close",
        SettingsBackCloseButton,
        SettingsControlAction::Close,
        back_close_node(),
    );
    let category_accessibility = spawn_text_control(
        &mut commands,
        panel,
        "Settings Accessibility Category",
        "Accessibility",
        SettingsCategoryNav,
        SettingsControlAction::SelectCategory(SettingsCategory::Accessibility),
        category_node(),
    );
    let content_pane = commands
        .spawn((
            Name::new("Settings Accessibility Content Pane"),
            SettingsContentPane,
            SettingsCategory::Accessibility,
            content_pane_node(),
            BackgroundColor(Color::srgb(0.105, 0.115, 0.135)),
            BorderColor::all(Color::srgb(0.28, 0.32, 0.38)),
            Visibility::Hidden,
            ChildOf(panel),
        ))
        .id();

    let colorblind_selector = spawn_text_control(
        &mut commands,
        content_pane,
        "Settings Colorblind Mode Selector",
        "",
        SettingsColorblindSelector,
        SettingsControlAction::CycleColorblindMode,
        control_node(0),
    );
    let reduced_motion_toggle = spawn_text_control(
        &mut commands,
        content_pane,
        "Settings Reduced Motion Toggle",
        "",
        SettingsReducedMotionToggle,
        SettingsControlAction::ToggleReducedMotion,
        control_node(1),
    );
    let timer_options = std::array::from_fn(|index| {
        let multiplier = PlacementTimerMultiplier::MULTIPLAYER_STANDARD_VALUES[index];
        let entity = spawn_text_control(
            &mut commands,
            content_pane,
            "Settings Placement Timer Option",
            placement_timer_label(multiplier),
            SettingsTimerSelector,
            SettingsControlAction::SelectPlacementTimer(multiplier),
            timer_option_node(index),
        );
        commands
            .entity(entity)
            .insert(SettingsTimerOption { multiplier });
        entity
    });
    let effective_timer_text = commands
        .spawn((
            Name::new("Settings Effective Timer Display"),
            SettingsEffectiveTimerDisplay {
                multiplier: settings_view.placement_timer_multiplier_effective,
            },
            Text::new(""),
            settings_text_font(15.0),
            TextColor(Color::srgb(0.82, 0.88, 0.94)),
            effective_timer_node(),
            Visibility::Hidden,
            ChildOf(content_pane),
        ))
        .id();
    let menu_scale_control = spawn_text_control(
        &mut commands,
        content_pane,
        "Settings Menu UI Scale Control",
        "",
        SettingsMenuUiScaleControl,
        SettingsControlAction::CycleMenuUiScale,
        control_node(2),
    );
    let hud_scale_control = spawn_text_control(
        &mut commands,
        content_pane,
        "Settings HUD UI Scale Control",
        "",
        SettingsHudUiScaleControl,
        SettingsControlAction::CycleHudUiScale,
        control_node(3),
    );
    let status_footer = commands
        .spawn((
            Name::new("Settings Status Footer"),
            SettingsStatusFooter,
            Text::new(""),
            settings_text_font(14.0),
            TextColor(Color::srgb(0.80, 0.86, 0.92)),
            status_footer_node(),
            Visibility::Hidden,
            ChildOf(panel),
        ))
        .id();
    let footer_close_button = spawn_text_control(
        &mut commands,
        panel,
        "Settings Footer Close",
        "Close",
        SettingsFooterAction,
        SettingsControlAction::Close,
        footer_close_node(),
    );

    commands.insert_resource(SettingsEntities {
        root,
        panel,
        back_close_button,
        category_accessibility,
        content_pane,
        colorblind_selector,
        reduced_motion_toggle,
        timer_options,
        effective_timer_text,
        menu_scale_control,
        hud_scale_control,
        status_footer,
        footer_close_button,
    });
}

pub fn handle_settings_open_requests_system(
    mut requests: MessageReader<SettingsOpenRequested>,
    current_phase: Res<CurrentClientPhase>,
    mut panel_state: ResMut<SettingsPanelState>,
    mut pending_request: ResMut<SettingsPendingRequest>,
) {
    for request in requests.read() {
        request_settings_open(
            request.source,
            current_phase.phase,
            &mut panel_state,
            &mut pending_request,
        );
    }
}

pub fn handle_settings_close_requests_system(
    mut requests: MessageReader<SettingsCloseRequested>,
    mut panel_state: ResMut<SettingsPanelState>,
    mut focus_order: ResMut<SettingsFocusOrder>,
) {
    for _request in requests.read() {
        close_settings_panel(&mut panel_state, &mut focus_order);
    }
}

pub fn settings_control_interaction_system(
    interactions: Query<(&Interaction, &SettingsControlActionMarker), Changed<Interaction>>,
    mut actions: MessageWriter<SettingsActionRequest>,
) {
    for (interaction, action) in &interactions {
        if *interaction == Interaction::Pressed {
            actions.write(SettingsActionRequest { action: action.0 });
        }
    }
}

pub fn settings_keyboard_input_system(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut focus_order: ResMut<SettingsFocusOrder>,
    actions_query: Query<&SettingsControlActionMarker>,
    mut close_writer: MessageWriter<SettingsCloseRequested>,
    mut action_writer: MessageWriter<SettingsActionRequest>,
) {
    let Some(keys) = keys else {
        return;
    };

    if keys.just_pressed(KeyCode::Escape) {
        close_writer.write(SettingsCloseRequested);
        return;
    }

    if keys.just_pressed(KeyCode::Tab) {
        focus_order.focus_next();
    }

    if !(keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space)) {
        return;
    }

    let Some(focused) = focus_order.focused_entity() else {
        return;
    };
    let Ok(action) = actions_query.get(focused) else {
        return;
    };

    action_writer.write(SettingsActionRequest { action: action.0 });
}

pub fn handle_settings_actions_system(
    mut actions: MessageReader<SettingsActionRequest>,
    mut preferences: ResMut<AccessibilityPreferences>,
    mut storage: ResMut<PreferenceStorageBackend>,
    mut status: ResMut<SettingsStatusFooterState>,
    mut panel_state: ResMut<SettingsPanelState>,
    mut focus_order: ResMut<SettingsFocusOrder>,
    current_phase: Res<CurrentClientPhase>,
    session_lifecycle: Res<SettingsSessionLifecycle>,
    mut outbound: ResMut<SettingsOutboundMessages>,
    mut timer_senders: Query<&mut MessageSender<C2SSetPlacementTimerMultiplier>>,
) {
    for request in actions.read() {
        match request.action {
            SettingsControlAction::Close => {
                close_settings_panel(&mut panel_state, &mut focus_order);
            }
            SettingsControlAction::SelectCategory(SettingsCategory::Accessibility) => {}
            SettingsControlAction::CycleColorblindMode => {
                preferences.colorblind_mode = preferences.colorblind_mode.next();
                persist_preferences(&mut storage, *preferences, &mut status);
            }
            SettingsControlAction::ToggleReducedMotion => {
                preferences.reduced_motion = !preferences.reduced_motion;
                persist_preferences(&mut storage, *preferences, &mut status);
            }
            SettingsControlAction::SelectPlacementTimer(multiplier) => {
                apply_placement_timer_selection(
                    multiplier,
                    &mut preferences,
                    &mut storage,
                    &mut status,
                    current_phase.phase,
                    session_lifecycle.session_ready,
                    &mut outbound,
                    &mut timer_senders,
                );
            }
            SettingsControlAction::CycleMenuUiScale => {
                let next_scale = next_ui_scale_percent(preferences.menu_ui_scale_percent);
                preferences.set_menu_ui_scale_percent(next_scale);
                persist_preferences(&mut storage, *preferences, &mut status);
            }
            SettingsControlAction::CycleHudUiScale => {
                let next_scale = next_ui_scale_percent(preferences.hud_ui_scale_percent);
                preferences.set_hud_ui_scale_percent(next_scale);
                persist_preferences(&mut storage, *preferences, &mut status);
            }
        }
    }
}

pub fn open_pending_settings_request_at_safe_boundary_system(
    current_phase: Res<CurrentClientPhase>,
    mut panel_state: ResMut<SettingsPanelState>,
    mut pending_request: ResMut<SettingsPendingRequest>,
) {
    let Some(source) = pending_request.source else {
        return;
    };

    if unsafe_phase_for_full_settings(current_phase.phase) {
        return;
    }

    panel_state.mode = SettingsPanelMode::Open;
    panel_state.source = Some(source);
    pending_request.source = None;
    pending_request.requested_during_phase = None;
}

pub fn track_settings_session_ready_system(
    current_phase: Res<CurrentClientPhase>,
    mut session_lifecycle: ResMut<SettingsSessionLifecycle>,
) {
    if matches!(
        current_phase.phase,
        RoundPhase::DraftInitial | RoundPhase::DraftShop
    ) {
        session_lifecycle.session_ready = true;
    }

    if current_phase.phase == RoundPhase::Lobby {
        session_lifecycle.session_ready = false;
    }
}

pub fn sync_settings_shell_visibility_system(
    entities: Option<Res<SettingsEntities>>,
    panel_state: Res<SettingsPanelState>,
    preferences: Res<AccessibilityPreferences>,
    mut focus_order: ResMut<SettingsFocusOrder>,
    mut visibility: Query<&mut Visibility>,
    mut panels: Query<(&mut Node, &mut SettingsMenuScaleApplied), With<SettingsPanel>>,
) {
    let Some(entities) = entities else {
        return;
    };

    let visible = panel_state.mode == SettingsPanelMode::Open;
    let target_visibility = visibility_for(visible);
    for entity in [
        entities.root,
        entities.panel,
        entities.back_close_button,
        entities.category_accessibility,
        entities.content_pane,
        entities.colorblind_selector,
        entities.reduced_motion_toggle,
        entities.effective_timer_text,
        entities.menu_scale_control,
        entities.hud_scale_control,
        entities.status_footer,
        entities.footer_close_button,
    ] {
        set_visibility(entity, target_visibility, &mut visibility);
    }

    for entity in entities.timer_options {
        set_visibility(entity, target_visibility, &mut visibility);
    }

    if visible {
        focus_order.set_entities(entities.visible_focus_order());
    } else {
        focus_order.set_entities(Vec::new());
    }

    if let Ok((mut node, mut applied)) = panels.get_mut(entities.panel) {
        let factor = menu_ui_scale_factor(preferences.menu_ui_scale_percent);
        node.width = Val::Px(SETTINGS_PANEL_BASE_WIDTH_PX * factor);
        node.height = Val::Px(SETTINGS_PANEL_BASE_HEIGHT_PX * factor);
        applied.percent = preferences.menu_ui_scale_percent;
        applied.factor = factor;
    }
}

pub fn sync_settings_text_system(
    entities: Option<Res<SettingsEntities>>,
    preferences: Res<AccessibilityPreferences>,
    settings_view: Res<SessionSettingsView>,
    status: Res<SettingsStatusFooterState>,
    mut texts: Query<&mut Text>,
    mut effective_displays: Query<&mut SettingsEffectiveTimerDisplay>,
) {
    let Some(entities) = entities else {
        return;
    };

    set_text(
        entities.colorblind_selector,
        format!("Colorblind mode: {}", preferences.colorblind_mode.label()),
        &mut texts,
    );
    set_text(
        entities.reduced_motion_toggle,
        format!(
            "Reduced motion: {}",
            if preferences.reduced_motion {
                "On"
            } else {
                "Off"
            }
        ),
        &mut texts,
    );
    set_text(
        entities.effective_timer_text,
        format!(
            "Effective room timer: {}",
            placement_timer_label(settings_view.placement_timer_multiplier_effective)
        ),
        &mut texts,
    );
    set_text(
        entities.menu_scale_control,
        format!("Menu UI scale: {}%", preferences.menu_ui_scale_percent),
        &mut texts,
    );
    set_text(
        entities.hud_scale_control,
        format!("HUD UI scale: {}%", preferences.hud_ui_scale_percent),
        &mut texts,
    );
    set_text(entities.status_footer, status.message.clone(), &mut texts);

    if let Ok(mut display) = effective_displays.get_mut(entities.effective_timer_text) {
        display.multiplier = settings_view.placement_timer_multiplier_effective;
    }
}

pub fn sync_settings_focus_indicator_system(
    mut commands: Commands,
    focus_order: Res<SettingsFocusOrder>,
    focused: Query<Entity, With<SettingsFocusIndicator>>,
) {
    let next_focused = focus_order.focused_entity();

    for entity in &focused {
        if Some(entity) != next_focused {
            commands.entity(entity).remove::<SettingsFocusIndicator>();
        }
    }

    if let Some(entity) = next_focused {
        commands.entity(entity).insert(SettingsFocusIndicator);
    }
}

pub fn request_settings_open(
    source: SettingsEntrySource,
    phase: RoundPhase,
    panel_state: &mut SettingsPanelState,
    pending_request: &mut SettingsPendingRequest,
) {
    if unsafe_phase_for_full_settings(phase) {
        pending_request.source = Some(source);
        pending_request.requested_during_phase = Some(phase);
        return;
    }

    panel_state.mode = SettingsPanelMode::Open;
    panel_state.source = Some(source);
    pending_request.source = None;
    pending_request.requested_during_phase = None;
}

pub fn close_settings_panel(
    panel_state: &mut SettingsPanelState,
    focus_order: &mut SettingsFocusOrder,
) {
    panel_state.mode = SettingsPanelMode::Closed;
    panel_state.last_closed_source = panel_state.source;
    panel_state.source = None;
    focus_order.set_entities(Vec::new());
}

pub fn apply_placement_timer_selection(
    multiplier: PlacementTimerMultiplier,
    preferences: &mut AccessibilityPreferences,
    storage: &mut PreferenceStorageBackend,
    status: &mut SettingsStatusFooterState,
    phase: RoundPhase,
    session_ready: bool,
    outbound: &mut SettingsOutboundMessages,
    timer_senders: &mut Query<&mut MessageSender<C2SSetPlacementTimerMultiplier>>,
) {
    preferences.placement_timer_multiplier_request = multiplier;
    persist_preferences(storage, *preferences, status);

    if phase != RoundPhase::Lobby || session_ready {
        return;
    }

    let message = C2SSetPlacementTimerMultiplier { multiplier };
    match timer_senders.single_mut() {
        Ok(mut sender) => {
            tracing::info!(
                msg_type = "C2SSetPlacementTimerMultiplier",
                multiplier = ?message.multiplier,
                handler = "apply_placement_timer_selection",
                "c2s_send: enter"
            );
            sender.send::<ReliableChannel>(message.clone());
        }
        Err(e) => {
            error!(
                "C2S send failed: type=C2SSetPlacementTimerMultiplier, handler=apply_placement_timer_selection, query_err={:?}",
                e
            );
        }
    }
    outbound.placement_timer_requests.push(message);
}

pub fn persist_preferences(
    storage: &mut PreferenceStorageBackend,
    preferences: AccessibilityPreferences,
    status: &mut SettingsStatusFooterState,
) {
    match storage.save(preferences) {
        Ok(()) => status.set_saved(),
        Err(_) => status.set_save_warning(),
    }
}

pub fn unsafe_phase_for_full_settings(phase: RoundPhase) -> bool {
    matches!(
        phase,
        RoundPhase::Placement | RoundPhase::DraftAuction | RoundPhase::Resolution
    )
}

pub fn placement_timer_label(multiplier: PlacementTimerMultiplier) -> &'static str {
    match multiplier {
        PlacementTimerMultiplier::X1 => "1x",
        PlacementTimerMultiplier::X1_5 => "1.5x",
        PlacementTimerMultiplier::X2 => "2x",
        PlacementTimerMultiplier::X3 => "3x",
    }
}

pub fn menu_ui_scale_factor(percent: u8) -> f32 {
    f32::from(clamp_ui_scale_percent(percent)) / 100.0
}

fn spawn_text_control<M: Component>(
    commands: &mut Commands,
    parent: Entity,
    name: &'static str,
    text: &'static str,
    marker: M,
    action: SettingsControlAction,
    node: Node,
) -> Entity {
    commands
        .spawn((
            Name::new(name),
            marker,
            Button,
            Interaction::None,
            SettingsControlActionMarker(action),
            Text::new(text),
            settings_text_font(16.0),
            TextColor(Color::srgb(0.94, 0.96, 0.98)),
            node,
            BackgroundColor(Color::srgb(0.13, 0.15, 0.18)),
            BorderColor::all(Color::srgb(0.36, 0.42, 0.50)),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .id()
}

fn set_visibility(entity: Entity, visibility: Visibility, query: &mut Query<&mut Visibility>) {
    if let Ok(mut current) = query.get_mut(entity) {
        *current = visibility;
    }
}

fn set_text(entity: Entity, text: String, texts: &mut Query<&mut Text>) {
    if let Ok(mut current) = texts.get_mut(entity) {
        current.0 = text;
    }
}

fn visibility_for(visible: bool) -> Visibility {
    if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    }
}

fn next_ui_scale_percent(current: u8) -> u8 {
    if current >= UI_SCALE_MAX_PERCENT {
        UI_SCALE_MIN_PERCENT
    } else {
        clamp_ui_scale_percent(current.saturating_add(UI_SCALE_STEP_PERCENT))
    }
}

fn settings_text_font(font_size: f32) -> TextFont {
    TextFont {
        font_size,
        ..default()
    }
}

fn settings_panel_node(menu_scale_percent: u8) -> Node {
    let factor = menu_ui_scale_factor(menu_scale_percent);
    Node {
        width: Val::Px(SETTINGS_PANEL_BASE_WIDTH_PX * factor),
        height: Val::Px(SETTINGS_PANEL_BASE_HEIGHT_PX * factor),
        min_width: Val::Px(SETTINGS_PANEL_BASE_WIDTH_PX * factor),
        min_height: Val::Px(SETTINGS_PANEL_BASE_HEIGHT_PX * factor),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    }
}

fn back_close_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(24.0),
        top: Val::Px(18.0),
        width: Val::Px(136.0),
        height: Val::Px(36.0),
        border: UiRect::all(Val::Px(1.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

fn category_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(24.0),
        top: Val::Px(86.0),
        width: Val::Px(170.0),
        height: Val::Px(40.0),
        border: UiRect::all(Val::Px(1.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

fn content_pane_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(220.0),
        right: Val::Px(24.0),
        top: Val::Px(76.0),
        bottom: Val::Px(74.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

fn control_node(index: usize) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(24.0),
        right: Val::Px(24.0),
        top: Val::Px(24.0 + index as f32 * 58.0),
        height: Val::Px(38.0),
        border: UiRect::all(Val::Px(1.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

fn timer_option_node(index: usize) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(24.0 + index as f32 * 86.0),
        top: Val::Px(138.0),
        width: Val::Px(72.0),
        height: Val::Px(34.0),
        border: UiRect::all(Val::Px(1.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

fn effective_timer_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(380.0),
        right: Val::Px(24.0),
        top: Val::Px(142.0),
        height: Val::Px(28.0),
        ..default()
    }
}

fn status_footer_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(24.0),
        right: Val::Px(180.0),
        bottom: Val::Px(20.0),
        height: Val::Px(30.0),
        ..default()
    }
}

fn footer_close_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(24.0),
        bottom: Val::Px(18.0),
        width: Val::Px(124.0),
        height: Val::Px(36.0),
        border: UiRect::all(Val::Px(1.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}
