use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::{ClientState, CurrentClientPhase, SessionSettingsView};
use client::ui::settings::{
    AccessibilityPreferences, ColorblindMode, SettingsAccessibilityPlugin, SettingsActionRequest,
    SettingsBackCloseButton, SettingsColorblindSelector, SettingsContentPane,
    SettingsControlAction, SettingsEffectiveTimerDisplay, SettingsEntities, SettingsEntrySource,
    SettingsFocusIndicator, SettingsFocusOrder, SettingsFooterAction, SettingsHudUiScaleControl,
    SettingsMenuScaleApplied, SettingsMenuUiScaleControl, SettingsOpenRequested, SettingsPanelMode,
    SettingsPanelState, SettingsPendingRequest, SettingsReducedMotionToggle, SettingsRoot,
    SettingsStatusFooter, SettingsTimerSelector,
};
use shared::protocol::{PlacementTimerMultiplier, RoundPhase};

#[test]
fn test_settings_shell_opens_and_closes_from_lobby_without_mutating_session_settings() {
    let mut app = app_with_settings();
    set_phase(&mut app, RoundPhase::Lobby);
    app.world_mut()
        .resource_mut::<SessionSettingsView>()
        .placement_timer_multiplier_effective = PlacementTimerMultiplier::X2;

    open_settings(&mut app, SettingsEntrySource::Lobby);

    assert_eq!(
        app.world().resource::<SettingsPanelState>().mode,
        SettingsPanelMode::Open
    );
    assert_eq!(
        app.world()
            .resource::<SessionSettingsView>()
            .placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X2
    );
    assert_visibility::<SettingsRoot>(&mut app, Visibility::Visible);
    assert_marker_count::<SettingsBackCloseButton>(&mut app, 1);
    assert_marker_count::<SettingsContentPane>(&mut app, 1);
    assert_marker_count::<SettingsStatusFooter>(&mut app, 1);
    assert_marker_count::<SettingsColorblindSelector>(&mut app, 1);
    assert_marker_count::<SettingsReducedMotionToggle>(&mut app, 1);
    assert_marker_count::<SettingsMenuUiScaleControl>(&mut app, 1);
    assert_marker_count::<SettingsHudUiScaleControl>(&mut app, 1);
    assert_marker_count::<SettingsTimerSelector>(&mut app, 4);

    let focus_order = app.world().resource::<SettingsFocusOrder>();
    assert_eq!(
        focus_order.focused_entity(),
        Some(app.world().resource::<SettingsEntities>().back_close_button)
    );
    assert_marker_count::<SettingsFocusIndicator>(&mut app, 1);

    app.world_mut().write_message(SettingsActionRequest {
        action: SettingsControlAction::Close,
    });
    app.update();

    assert_eq!(
        app.world().resource::<SettingsPanelState>().mode,
        SettingsPanelMode::Closed
    );
    assert_eq!(
        app.world()
            .resource::<SettingsPanelState>()
            .last_closed_source,
        Some(SettingsEntrySource::Lobby)
    );
    assert!(app
        .world()
        .resource::<SettingsFocusOrder>()
        .entities
        .is_empty());
}

#[test]
fn test_settings_unsafe_phase_queues_request_until_safe_boundary() {
    let mut app = app_with_settings();
    set_phase(&mut app, RoundPhase::Placement);

    open_settings(&mut app, SettingsEntrySource::SafeInGame);

    assert_eq!(
        app.world().resource::<SettingsPanelState>().mode,
        SettingsPanelMode::Closed
    );
    assert_eq!(
        app.world()
            .resource::<SettingsPendingRequest>()
            .requested_during_phase,
        Some(RoundPhase::Placement)
    );
    assert_visibility::<SettingsRoot>(&mut app, Visibility::Hidden);

    set_phase(&mut app, RoundPhase::DraftShop);
    app.update();

    assert_eq!(
        app.world().resource::<SettingsPanelState>().mode,
        SettingsPanelMode::Open
    );
    assert_eq!(
        app.world().resource::<SettingsPendingRequest>().source,
        None
    );
}

#[test]
fn test_settings_actions_update_preferences_independently_and_apply_menu_scale_hook() {
    let mut app = app_with_settings();
    set_phase(&mut app, RoundPhase::Lobby);
    open_settings(&mut app, SettingsEntrySource::Lobby);

    app.world_mut().write_message(SettingsActionRequest {
        action: SettingsControlAction::CycleColorblindMode,
    });
    app.world_mut().write_message(SettingsActionRequest {
        action: SettingsControlAction::ToggleReducedMotion,
    });
    app.world_mut().write_message(SettingsActionRequest {
        action: SettingsControlAction::CycleMenuUiScale,
    });
    app.world_mut().write_message(SettingsActionRequest {
        action: SettingsControlAction::CycleHudUiScale,
    });
    app.update();

    let preferences = app.world().resource::<AccessibilityPreferences>();
    assert_eq!(preferences.colorblind_mode, ColorblindMode::Protanopia);
    assert!(preferences.reduced_motion);
    assert_eq!(preferences.menu_ui_scale_percent, 125);
    assert_eq!(preferences.hud_ui_scale_percent, 125);

    let panel = app.world().resource::<SettingsEntities>().panel;
    let applied = app
        .world()
        .get::<SettingsMenuScaleApplied>(panel)
        .expect("settings panel should expose menu scale application hook");
    assert_eq!(applied.percent, 125);
    assert_eq!(applied.factor, 1.25);
}

#[test]
fn test_settings_focus_order_excludes_hidden_controls_when_closed() {
    let mut app = app_with_settings();

    assert!(app
        .world()
        .resource::<SettingsFocusOrder>()
        .entities
        .is_empty());

    set_phase(&mut app, RoundPhase::Lobby);
    open_settings(&mut app, SettingsEntrySource::Lobby);

    let entities = *app.world().resource::<SettingsEntities>();
    let order = &app.world().resource::<SettingsFocusOrder>().entities;
    assert_eq!(order.first().copied(), Some(entities.back_close_button));
    assert_eq!(order.get(1).copied(), Some(entities.category_accessibility));
    assert_eq!(order.last().copied(), Some(entities.footer_close_button));

    assert!(
        app.world()
            .get::<SettingsFooterAction>(entities.footer_close_button)
            .is_some(),
        "footer action should be represented by a stable marker"
    );
    assert!(
        app.world()
            .get::<SettingsEffectiveTimerDisplay>(entities.effective_timer_text)
            .is_some(),
        "effective timer display should be represented by a stable marker"
    );
}

fn app_with_settings() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(SettingsAccessibilityPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Lobby);
    app.update();
    app
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn open_settings(app: &mut App, source: SettingsEntrySource) {
    app.world_mut()
        .write_message(SettingsOpenRequested { source });
    app.update();
}

fn assert_visibility<T: Component>(app: &mut App, expected: Visibility) {
    let mut query = app.world_mut().query_filtered::<&Visibility, With<T>>();
    assert!(
        query
            .iter(app.world())
            .all(|visibility| *visibility == expected),
        "all matching entities should have {expected:?}"
    );
}

fn assert_marker_count<T: Component>(app: &mut App, expected: usize) {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    assert_eq!(query.iter(app.world()).count(), expected);
}
