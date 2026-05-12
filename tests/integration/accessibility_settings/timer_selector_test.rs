use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::{ClientState, CurrentClientPhase, SessionSettingsView};
use client::ui::settings::{
    AccessibilityPreferences, SettingsAccessibilityPlugin, SettingsActionRequest,
    SettingsControlAction, SettingsEffectiveTimerDisplay, SettingsEntities, SettingsEntrySource,
    SettingsOpenRequested, SettingsOutboundMessages, SettingsSessionLifecycle, SettingsTimerOption,
};
use shared::protocol::{PlacementTimerMultiplier, RoundPhase};

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn test_timer_selector_exposes_only_multiplayer_standard_values() {
    test_helpers::init_test_tracing();
    let mut app = app_with_open_settings();

    let labels = timer_option_texts(&app);
    assert_eq!(labels, ["1x", "1.5x", "2x", "3x"]);
    assert!(!labels.iter().any(|label| label.contains("0.5")));
    assert!(timer_related_texts(&mut app)
        .iter()
        .all(|text| !text.contains("player")
            && !text.contains("requester")
            && !text.contains("ID")));

    let entities = *app.world().resource::<SettingsEntities>();
    for (entity, multiplier) in entities.timer_options.into_iter().zip([
        PlacementTimerMultiplier::X1,
        PlacementTimerMultiplier::X1_5,
        PlacementTimerMultiplier::X2,
        PlacementTimerMultiplier::X3,
    ]) {
        assert_eq!(
            app.world().get::<SettingsTimerOption>(entity),
            Some(&SettingsTimerOption { multiplier })
        );
    }
}

#[test]
fn test_lobby_before_session_ready_timer_change_sends_one_c2s_intent() {
    test_helpers::init_test_tracing();
    let mut app = app_with_open_settings();

    app.world_mut().write_message(SettingsActionRequest {
        action: SettingsControlAction::SelectPlacementTimer(PlacementTimerMultiplier::X3),
    });
    app.update();

    let outbound = app.world().resource::<SettingsOutboundMessages>();
    assert_eq!(outbound.placement_timer_requests.len(), 1);
    assert_eq!(
        outbound.placement_timer_requests[0].multiplier,
        PlacementTimerMultiplier::X3
    );
    assert_eq!(
        app.world()
            .resource::<AccessibilityPreferences>()
            .placement_timer_multiplier_request,
        PlacementTimerMultiplier::X3
    );
}

#[test]
fn test_after_session_ready_timer_change_is_next_session_preference_only() {
    test_helpers::init_test_tracing();
    let mut app = app_with_open_settings();
    app.world_mut()
        .resource_mut::<SettingsSessionLifecycle>()
        .session_ready = true;
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = RoundPhase::DraftShop;
    app.world_mut()
        .resource_mut::<SessionSettingsView>()
        .placement_timer_multiplier_effective = PlacementTimerMultiplier::X1_5;

    app.world_mut().write_message(SettingsActionRequest {
        action: SettingsControlAction::SelectPlacementTimer(PlacementTimerMultiplier::X2),
    });
    app.update();

    assert!(
        app.world()
            .resource::<SettingsOutboundMessages>()
            .placement_timer_requests
            .is_empty(),
        "active-session changes should not emit timer C2S requests"
    );
    assert_eq!(
        app.world()
            .resource::<AccessibilityPreferences>()
            .placement_timer_multiplier_request,
        PlacementTimerMultiplier::X2
    );
    assert_eq!(
        app.world()
            .resource::<SessionSettingsView>()
            .placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X1_5,
        "active neutral session settings view must remain server-owned"
    );
}

#[test]
fn test_effective_timer_display_reads_neutral_session_settings_view() {
    test_helpers::init_test_tracing();
    let mut app = app_with_open_settings();
    app.world_mut()
        .resource_mut::<AccessibilityPreferences>()
        .placement_timer_multiplier_request = PlacementTimerMultiplier::X3;
    app.world_mut()
        .resource_mut::<SessionSettingsView>()
        .placement_timer_multiplier_effective = PlacementTimerMultiplier::X1_5;
    app.update();

    let effective_text = text_for(
        &app,
        app.world()
            .resource::<SettingsEntities>()
            .effective_timer_text,
    );
    assert_eq!(effective_text, "Effective room timer: 1.5x");
    assert_eq!(
        app.world()
            .get::<SettingsEffectiveTimerDisplay>(
                app.world()
                    .resource::<SettingsEntities>()
                    .effective_timer_text
            )
            .unwrap()
            .multiplier,
        PlacementTimerMultiplier::X1_5
    );
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

fn timer_option_texts(app: &App) -> [String; 4] {
    app.world()
        .resource::<SettingsEntities>()
        .timer_options
        .map(|entity| text_for(app, entity))
}

fn timer_related_texts(app: &mut App) -> Vec<String> {
    let entities = *app.world().resource::<SettingsEntities>();
    let mut texts = entities
        .timer_options
        .into_iter()
        .map(|entity| text_for(app, entity))
        .collect::<Vec<_>>();
    texts.push(text_for(app, entities.effective_timer_text));
    texts
}

fn text_for(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .expect("settings text entity should have Text")
        .0
        .clone()
}
