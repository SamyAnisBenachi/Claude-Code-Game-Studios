use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::ui::{Display, FlexDirection, FlexWrap, JustifyContent, OverflowAxis, PositionType, Val};
use client::state::{ClientState, CurrentClientPhase};
use client::ui::settings::{
    AccessibilityPreferences, SettingsAccessibilityPlugin, SettingsActionRequest,
    SettingsBackCloseButton, SettingsCategoryNav, SettingsColorblindSelector, SettingsContentPane,
    SettingsControlAction, SettingsEntities, SettingsEntrySource, SettingsFocusOrder,
    SettingsFooterAction, SettingsHudUiScaleControl, SettingsMenuScaleApplied,
    SettingsMenuUiScaleControl, SettingsOpenRequested, SettingsPanel, SettingsReducedMotionToggle,
    SettingsStatusFooter, SettingsTimerSelector, SETTINGS_PANEL_BASE_WIDTH_PX,
    SETTINGS_PANEL_MIN_WIDTH_PX, UI_SCALE_MAX_PERCENT, UI_SCALE_MIN_PERCENT,
};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn test_settings_panel_root_uses_flex_column_with_max_height_and_overflow_scroll_y() {
    // Arrange: bring up the settings shell and open the panel.
    test_helpers::init_test_tracing();
    let app = app_with_open_settings();

    // Act: read the panel's Node.
    let panel_entity = app.world().resource::<SettingsEntities>().panel;
    let node = app
        .world()
        .get::<Node>(panel_entity)
        .expect("settings panel must carry a Node");

    // Assert: panel is a flex column with the bounded-overflow contract
    // PROMPT 1187 Lane F demands — Display::Flex, FlexDirection::Column,
    // max_height: Percent(92), and Overflow::scroll_y on the Y axis.
    assert_eq!(node.display, Display::Flex);
    assert_eq!(node.flex_direction, FlexDirection::Column);
    assert_eq!(node.max_height, Val::Percent(92.0));
    assert_eq!(node.max_width, Val::Percent(92.0));
    assert_eq!(node.overflow.y, OverflowAxis::Scroll);
    assert_ne!(node.position_type, PositionType::Absolute);
}

#[test]
fn test_settings_panel_children_use_flex_layout_not_absolute_offsets() {
    // Arrange: open settings so every leaf entity is spawned.
    test_helpers::init_test_tracing();
    let app = app_with_open_settings();

    // Act + Assert: every named leaf must be flex-laid; no relayed-out
    // child may carry PositionType::Absolute (the O-01 anti-pattern).
    let entities = *app.world().resource::<SettingsEntities>();
    let leaves = [
        ("back_close_button", entities.back_close_button),
        ("category_accessibility", entities.category_accessibility),
        ("content_pane", entities.content_pane),
        ("colorblind_selector", entities.colorblind_selector),
        ("reduced_motion_toggle", entities.reduced_motion_toggle),
        ("effective_timer_text", entities.effective_timer_text),
        ("menu_scale_control", entities.menu_scale_control),
        ("hud_scale_control", entities.hud_scale_control),
        ("status_footer", entities.status_footer),
        ("footer_close_button", entities.footer_close_button),
    ];

    for (name, entity) in leaves {
        let node = app
            .world()
            .get::<Node>(entity)
            .unwrap_or_else(|| panic!("settings child `{name}` must carry a Node"));
        assert_ne!(
            node.position_type,
            PositionType::Absolute,
            "settings child `{name}` must not use PositionType::Absolute"
        );
    }

    for (index, entity) in entities.timer_options.into_iter().enumerate() {
        let node = app
            .world()
            .get::<Node>(entity)
            .unwrap_or_else(|| panic!("timer option #{index} must carry a Node"));
        assert_ne!(
            node.position_type,
            PositionType::Absolute,
            "timer option #{index} must not use PositionType::Absolute"
        );
    }
}

#[test]
fn test_settings_timer_options_share_a_flex_row_wrap_parent() {
    // Arrange: open settings.
    test_helpers::init_test_tracing();
    let app = app_with_open_settings();
    let entities = *app.world().resource::<SettingsEntities>();

    // Act: walk every timer-option's ChildOf parent and assert they share
    // one parent, and that parent is the wrapped flex row.
    let parents: Vec<Entity> = entities
        .timer_options
        .into_iter()
        .map(|entity| {
            app.world()
                .get::<ChildOf>(entity)
                .expect("timer option must have a ChildOf parent")
                .parent()
        })
        .collect();
    let first_parent = parents[0];
    assert!(
        parents.iter().all(|p| *p == first_parent),
        "all timer options must share one parent flex-row, got {parents:?}"
    );

    // Assert: shared parent is a flex row with wrap enabled — keeps the
    // four options reachable on narrow viewports at any UI scale.
    let parent_node = app
        .world()
        .get::<Node>(first_parent)
        .expect("timer options row must carry a Node");
    assert_eq!(parent_node.display, Display::Flex);
    assert_eq!(parent_node.flex_direction, FlexDirection::Row);
    assert_eq!(parent_node.flex_wrap, FlexWrap::Wrap);
}

#[test]
fn test_settings_content_pane_overflow_scrolls_y_for_narrow_viewports() {
    // Arrange.
    test_helpers::init_test_tracing();
    let app = app_with_open_settings();
    let content_pane = app.world().resource::<SettingsEntities>().content_pane;

    // Act + Assert: content pane is a flex column with Overflow::scroll_y
    // so the 1280x720 / 1366x768 floor never hides controls behind a
    // dead zone (PROMPT 1180 audit C-5).
    let node = app
        .world()
        .get::<Node>(content_pane)
        .expect("content pane must carry a Node");
    assert_eq!(node.display, Display::Flex);
    assert_eq!(node.flex_direction, FlexDirection::Column);
    assert_eq!(node.overflow.y, OverflowAxis::Scroll);
}

#[test]
fn test_settings_footer_row_uses_space_between_for_status_and_close() {
    // Arrange.
    test_helpers::init_test_tracing();
    let app = app_with_open_settings();
    let entities = *app.world().resource::<SettingsEntities>();

    // Act: footer close + status footer share one parent — the footer
    // row — that is laid out as a flex row with SpaceBetween.
    let status_parent = app
        .world()
        .get::<ChildOf>(entities.status_footer)
        .expect("status footer must have a parent")
        .parent();
    let close_parent = app
        .world()
        .get::<ChildOf>(entities.footer_close_button)
        .expect("footer close must have a parent")
        .parent();
    assert_eq!(
        status_parent, close_parent,
        "status footer and close button must share the footer-row parent"
    );

    let footer_node = app
        .world()
        .get::<Node>(status_parent)
        .expect("footer row must carry a Node");
    assert_eq!(footer_node.display, Display::Flex);
    assert_eq!(footer_node.flex_direction, FlexDirection::Row);
    assert_eq!(footer_node.justify_content, JustifyContent::SpaceBetween);
}

#[test]
fn test_settings_panel_width_scales_with_ui_scale_at_75_and_150_percent() {
    // Arrange.
    test_helpers::init_test_tracing();
    let mut app = app_with_open_settings();
    let panel = app.world().resource::<SettingsEntities>().panel;

    // Act + Assert: at every supported UI-scale step, panel width tracks
    // SETTINGS_PANEL_BASE_WIDTH_PX × factor, with a min-width floor so 75 %
    // keeps the category column + content pane reachable.
    for percent in [UI_SCALE_MIN_PERCENT, 100, 125, UI_SCALE_MAX_PERCENT] {
        app.world_mut()
            .resource_mut::<AccessibilityPreferences>()
            .set_menu_ui_scale_percent(percent);
        app.update();

        let node = app
            .world()
            .get::<Node>(panel)
            .expect("settings panel must carry a Node");
        let expected_factor = f32::from(percent) / 100.0;
        let expected_width = SETTINGS_PANEL_BASE_WIDTH_PX * expected_factor;
        assert_eq!(
            node.width,
            Val::Px(expected_width),
            "panel width at {percent}% must equal base × factor"
        );
        assert_eq!(
            node.min_width,
            Val::Px(SETTINGS_PANEL_MIN_WIDTH_PX),
            "panel min_width must stay at the SETTINGS_PANEL_MIN_WIDTH_PX floor"
        );
        assert_eq!(node.max_width, Val::Percent(92.0));
        assert_eq!(node.max_height, Val::Percent(92.0));
        assert_eq!(node.height, Val::Auto);
        assert_eq!(node.min_height, Val::Auto);

        let applied = app
            .world()
            .get::<SettingsMenuScaleApplied>(panel)
            .expect("panel must expose the scale-application hook");
        assert_eq!(applied.percent, percent);
        assert!(
            (applied.factor - expected_factor).abs() < f32::EPSILON,
            "applied.factor for {percent}% expected {expected_factor}, got {}",
            applied.factor
        );
    }
}

#[test]
fn test_settings_marker_counts_unchanged_under_flex_relayout() {
    // Arrange.
    test_helpers::init_test_tracing();
    let mut app = app_with_open_settings();

    // Act + Assert: the flex relayout preserves the same set of marker
    // components the existing shell test guarantees (regression against
    // accidental removal of a control during refactor).
    assert_marker_count::<SettingsPanel>(&mut app, 1);
    assert_marker_count::<SettingsBackCloseButton>(&mut app, 1);
    assert_marker_count::<SettingsCategoryNav>(&mut app, 1);
    assert_marker_count::<SettingsContentPane>(&mut app, 1);
    assert_marker_count::<SettingsColorblindSelector>(&mut app, 1);
    assert_marker_count::<SettingsReducedMotionToggle>(&mut app, 1);
    assert_marker_count::<SettingsMenuUiScaleControl>(&mut app, 1);
    assert_marker_count::<SettingsHudUiScaleControl>(&mut app, 1);
    assert_marker_count::<SettingsStatusFooter>(&mut app, 1);
    assert_marker_count::<SettingsFooterAction>(&mut app, 1);
    assert_marker_count::<SettingsTimerSelector>(&mut app, 4);
}

#[test]
fn test_settings_focus_order_traverses_flex_hierarchy_in_documented_order() {
    // Arrange.
    test_helpers::init_test_tracing();
    let mut app = app_with_open_settings();

    // Act: open the panel and read the focus order.
    let entities = *app.world().resource::<SettingsEntities>();
    let order = app
        .world()
        .resource::<SettingsFocusOrder>()
        .entities
        .clone();

    // Assert: focus order start / continuity / end match the documented
    // sequence even though the new flex hierarchy reparents controls into
    // intermediate wrappers.
    assert_eq!(order.first().copied(), Some(entities.back_close_button));
    assert_eq!(order.get(1).copied(), Some(entities.category_accessibility));
    assert_eq!(order.get(2).copied(), Some(entities.colorblind_selector));
    assert_eq!(order.get(3).copied(), Some(entities.reduced_motion_toggle));
    for (offset, timer_entity) in entities.timer_options.into_iter().enumerate() {
        assert_eq!(
            order.get(4 + offset).copied(),
            Some(timer_entity),
            "timer option #{offset} must appear at focus index {}",
            4 + offset
        );
    }
    assert_eq!(order.get(8).copied(), Some(entities.menu_scale_control));
    assert_eq!(order.get(9).copied(), Some(entities.hud_scale_control));
    assert_eq!(order.last().copied(), Some(entities.footer_close_button));

    // Act: close from inside the panel.
    app.world_mut().write_message(SettingsActionRequest {
        action: SettingsControlAction::Close,
    });
    app.update();

    // Assert: closure clears the focus order regardless of new wrappers.
    assert!(app
        .world()
        .resource::<SettingsFocusOrder>()
        .entities
        .is_empty());
}

fn app_with_open_settings() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(SettingsAccessibilityPlugin);
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = RoundPhase::Lobby;
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Lobby);
    app.update();
    app.world_mut().write_message(SettingsOpenRequested {
        source: SettingsEntrySource::Lobby,
    });
    app.update();
    app
}

fn assert_marker_count<T: Component>(app: &mut App, expected: usize) {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    let actual = query.iter(app.world()).count();
    assert_eq!(
        actual,
        expected,
        "expected {expected} entities with {} marker, got {actual}",
        std::any::type_name::<T>()
    );
}
