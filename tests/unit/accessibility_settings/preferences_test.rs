use client::ui::settings::{
    clamp_ui_scale_percent, decode_preferences_payload, encode_preferences_payload,
    persist_preferences, AccessibilityPreferences, ColorblindMode, PreferenceStorageBackend,
    PreferenceValidationError, SettingsStatusFooterState, ACCESSIBILITY_PREFERENCES_STORAGE_KEY,
    UI_SCALE_MAX_PERCENT, UI_SCALE_MIN_PERCENT,
};
use shared::protocol::PlacementTimerMultiplier;

#[test]
fn test_preference_defaults_are_story_defaults() {
    let preferences = AccessibilityPreferences::default();

    assert_eq!(preferences.colorblind_mode, ColorblindMode::Off);
    assert!(!preferences.reduced_motion);
    assert_eq!(
        preferences.placement_timer_multiplier_request,
        PlacementTimerMultiplier::X1
    );
    assert_eq!(preferences.menu_ui_scale_percent, 100);
    assert_eq!(preferences.hud_ui_scale_percent, 100);
}

#[test]
fn test_preference_validation_clamps_scale_and_rejects_invalid_timer_ratio() {
    let mut preferences = AccessibilityPreferences::default();

    preferences.set_menu_ui_scale_percent(10);
    preferences.set_hud_ui_scale_percent(250);

    assert_eq!(preferences.menu_ui_scale_percent, UI_SCALE_MIN_PERCENT);
    assert_eq!(preferences.hud_ui_scale_percent, UI_SCALE_MAX_PERCENT);
    assert_eq!(clamp_ui_scale_percent(125), 125);

    assert_eq!(
        preferences.try_set_placement_timer_multiplier_from_ratio(1, 2),
        Err(PreferenceValidationError::InvalidPlacementTimerMultiplier)
    );
    assert_eq!(
        preferences.try_set_placement_timer_multiplier_from_ratio(4, 1),
        Err(PreferenceValidationError::InvalidPlacementTimerMultiplier)
    );

    preferences
        .try_set_placement_timer_multiplier_from_ratio(3, 2)
        .expect("1.5x should be a standard multiplayer timer ratio");
    assert_eq!(
        preferences.placement_timer_multiplier_request,
        PlacementTimerMultiplier::X1_5
    );
}

#[test]
fn test_preference_storage_round_trips_versioned_single_payload() {
    assert_eq!(
        ACCESSIBILITY_PREFERENCES_STORAGE_KEY,
        "lanes_and_lies.accessibility_preferences.v1"
    );

    let preferences = AccessibilityPreferences {
        colorblind_mode: ColorblindMode::Tritanopia,
        reduced_motion: true,
        placement_timer_multiplier_request: PlacementTimerMultiplier::X3,
        menu_ui_scale_percent: 125,
        hud_ui_scale_percent: 75,
    };
    let mut storage = PreferenceStorageBackend::InMemory { payload: None };

    storage
        .save(preferences)
        .expect("in-memory native storage should save");

    let PreferenceStorageBackend::InMemory {
        payload: Some(payload),
    } = &storage
    else {
        panic!("in-memory storage should hold one serialized payload");
    };
    assert!(payload.contains("\"version\":1"));
    assert_eq!(decode_preferences_payload(payload).unwrap(), preferences);
    assert_eq!(storage.load().unwrap(), Some(preferences));

    let encoded = encode_preferences_payload(preferences).unwrap();
    assert_eq!(decode_preferences_payload(&encoded).unwrap(), preferences);
}

#[test]
fn test_storage_write_failure_preserves_runtime_preferences_and_reports_warning() {
    let preferences = AccessibilityPreferences {
        colorblind_mode: ColorblindMode::Protanopia,
        reduced_motion: true,
        placement_timer_multiplier_request: PlacementTimerMultiplier::X2,
        menu_ui_scale_percent: 150,
        hud_ui_scale_percent: 125,
    };
    let mut storage = PreferenceStorageBackend::FailWrites { payload: None };
    let mut status = SettingsStatusFooterState::default();

    persist_preferences(&mut storage, preferences, &mut status);

    assert!(status.save_warning);
    assert!(status.message.contains("save failed"));
    assert_eq!(storage.load().unwrap(), None);
    assert_eq!(
        preferences.placement_timer_multiplier_request,
        PlacementTimerMultiplier::X2,
        "runtime preferences are already updated before persistence is attempted"
    );
}
