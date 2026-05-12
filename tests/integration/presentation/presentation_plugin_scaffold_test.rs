use std::fs;
use std::path::Path;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::{
    apply_phase_changed_messages, CurrentClientPhase, PresentationPlugin, PresentationSet,
};
use client::state::{apply_session_settings_updated_message, ClientState, SessionSettingsView};
use shared::protocol::{
    PlacementTimerMultiplier, RoundPhase, S2CPhaseChanged, S2CSessionSettingsUpdated,
};

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[derive(Resource, Default, Debug, PartialEq, Eq)]
struct PresentationOrder(Vec<&'static str>);

#[test]
fn presentation_plugin_registers_phase_state_and_runs_sets_in_order() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.add_plugins(PresentationPlugin);
    app.init_resource::<PresentationOrder>();
    app.add_systems(
        Update,
        (
            record_phase_transition.in_set(PresentationSet::PhaseTransition),
            record_message_drain.in_set(PresentationSet::MessageDrain),
            record_state_sync.in_set(PresentationSet::StateSync),
            record_animation_tick.in_set(PresentationSet::AnimationTick),
        ),
    );

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    assert!(app.world().get_resource::<CurrentClientPhase>().is_some());
    assert_eq!(
        app.world().resource::<PresentationOrder>().0,
        ["phase", "drain", "sync", "animation"]
    );
}

#[test]
fn phase_sink_application_is_last_write_wins_and_ignores_timer_data() {
    test_helpers::init_test_tracing();
    let mut current = CurrentClientPhase {
        phase: RoundPhase::DraftInitial,
        round: 1,
    };

    apply_phase_changed_messages(
        [
            S2CPhaseChanged {
                phase: RoundPhase::Placement,
                round_number: 2,
                timer_duration_ms: 10_000,
            },
            S2CPhaseChanged {
                phase: RoundPhase::Resolution,
                round_number: 3,
                timer_duration_ms: 1,
            },
        ],
        &mut current,
    );

    assert_eq!(current.phase, RoundPhase::Resolution);
    assert_eq!(current.round, 3);
}

#[test]
fn phase_receiver_source_guard_has_one_production_drain() {
    test_helpers::init_test_tracing();
    let client_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut matches = Vec::new();
    collect_source_matches(
        &client_src,
        "MessageReceiver<S2CPhaseChanged>",
        &mut matches,
    );

    assert_eq!(
        matches,
        vec![client_src.join("presentation").join("mod.rs")]
    );
}

#[test]
fn session_settings_update_application_is_neutral_last_write_wins() {
    test_helpers::init_test_tracing();
    let mut settings = SessionSettingsView::default();

    for message in [
        S2CSessionSettingsUpdated {
            placement_timer_multiplier_effective: PlacementTimerMultiplier::X1_5,
        },
        S2CSessionSettingsUpdated {
            placement_timer_multiplier_effective: PlacementTimerMultiplier::X3,
        },
    ] {
        apply_session_settings_updated_message(&message, &mut settings);
    }

    assert_eq!(
        settings.placement_timer_multiplier_effective,
        PlacementTimerMultiplier::X3
    );
}

#[test]
fn session_settings_receiver_source_guard_has_one_production_drain() {
    test_helpers::init_test_tracing();
    let client_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut matches = Vec::new();
    collect_source_matches(
        &client_src,
        "MessageReceiver<S2CSessionSettingsUpdated>",
        &mut matches,
    );

    assert_eq!(
        matches,
        vec![client_src.join("presentation").join("mod.rs")]
    );
}

fn record_phase_transition(mut order: ResMut<PresentationOrder>) {
    order.0.push("phase");
}

fn record_message_drain(mut order: ResMut<PresentationOrder>) {
    order.0.push("drain");
}

fn record_state_sync(mut order: ResMut<PresentationOrder>) {
    order.0.push("sync");
}

fn record_animation_tick(mut order: ResMut<PresentationOrder>) {
    order.0.push("animation");
}

fn collect_source_matches(path: &Path, needle: &str, matches: &mut Vec<std::path::PathBuf>) {
    let entries = fs::read_dir(path).expect("client source directory should be readable");
    for entry in entries {
        let path = entry.expect("source entry should be readable").path();
        if path.is_dir() {
            collect_source_matches(&path, needle, matches);
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let contents = fs::read_to_string(&path).expect("Rust source file should be readable");
        for _ in contents.match_indices(needle) {
            matches.push(path.clone());
        }
    }
}
