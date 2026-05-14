//! Client `phase_changed=true` 60Hz idempotency test.
//!
//! Story: `S11-HU-PHASE-IDEMPOTENCY-001` (story-022). PROMPT 803 §3 DC-5.
//!
//! Asserts that the hand-UI phase-transition consumer at
//! `client/src/ui/hand/mod.rs:hand_ui_phase_transition_system` narrows its
//! `phase_changed=true` signal so that it fires only on actual
//! `RoundPhase` transitions (i.e., inequality against the previous frame's
//! observed phase value) — not on every Update tick.
//!
//! Observation strategy: the system body has an
//! `if phase_changed { placement_timer.submitted = false; ... }` block.
//! We set `placement_timer.submitted = true` between frames as a sentinel.
//! If `phase_changed` fires on an Update where the phase did not transition,
//! the sentinel is clobbered to `false`. The test asserts the sentinel
//! survives many ticks with the same phase value, and is correctly cleared
//! on the actual transition Update.
//!
//! No optimistic client-side phase authority is introduced or relied upon
//! by this test. `S2CPhaseChanged` drain remains the single source of phase
//! truth (ADR-021); the test drives `Res<CurrentClientPhase>` directly only
//! to simulate post-drain frames in isolation.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{HandUiPlugin, PlacementTimer};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const SAME_PHASE_FRAME_COUNT: u32 = 10;

#[test]
fn ac3_phase_changed_does_not_fire_on_frames_without_transition() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_phase(RoundPhase::Placement);

    // Set the sentinel that the system's `if phase_changed` block would clear.
    set_submitted_sentinel(&mut app);
    assert!(
        submitted(&app),
        "precondition: sentinel must be set before holding the phase"
    );

    for tick in 0..SAME_PHASE_FRAME_COUNT {
        app.update();
        assert!(
            submitted(&app),
            "phase_changed=true must not fire on tick {tick} when the observed \
             RoundPhase did not transition (sentinel was clobbered)"
        );
    }
}

#[test]
fn ac2_phase_changed_fires_on_actual_transition() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_phase(RoundPhase::Placement);

    set_submitted_sentinel(&mut app);
    // Hold the phase for several ticks; sentinel must survive.
    for _ in 0..3 {
        app.update();
    }
    assert!(submitted(&app), "sentinel must survive same-phase ticks");

    // Now transition to a different phase.
    set_phase(&mut app, RoundPhase::DraftShop);
    app.update();

    assert!(
        !submitted(&app),
        "phase_changed=true must fire on the actual transition tick \
         (sentinel was expected to be cleared)"
    );
}

#[test]
fn ac3_at_most_one_phase_changed_across_multi_frame_run_with_one_transition() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_phase(RoundPhase::Placement);

    // Count clears across a multi-tick run that contains exactly one
    // transition (Placement -> ShopAuction at frame 5).
    let mut clears = 0u32;
    for tick in 0..SAME_PHASE_FRAME_COUNT {
        set_submitted_sentinel(&mut app);
        if tick == 5 {
            set_phase(&mut app, RoundPhase::DraftShop);
        }
        app.update();
        if !submitted(&app) {
            clears += 1;
        }
    }

    assert_eq!(
        clears, 1,
        "phase_changed=true must fire exactly once across {SAME_PHASE_FRAME_COUNT} \
         ticks containing exactly one RoundPhase transition (observed {clears} clears)"
    );
}

#[test]
fn ac4_phase_changed_fires_on_first_observation() {
    test_helpers::init_test_tracing();
    // First-observation case: before any tick has run the system body,
    // `Local<Option<RoundPhase>>` is `None`, so the system must treat the
    // first observation as a transition. We assert this by setting the
    // sentinel BEFORE the first tick and verifying it gets cleared.
    let mut app = build_app_in_phase(RoundPhase::Placement);
    set_submitted_sentinel(&mut app);

    app.update();

    assert!(
        !submitted(&app),
        "first-observation tick must register as a transition \
         (sentinel was expected to be cleared on the first Update)"
    );
}

#[test]
fn ac2_repeated_same_phase_assignments_do_not_register_as_transitions() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_phase(RoundPhase::Placement);

    set_submitted_sentinel(&mut app);
    // Re-assign the SAME phase value several times across ticks; the
    // consumer must compare phase values, not mutable accesses, so no
    // transition should be registered.
    for _ in 0..5 {
        set_phase(&mut app, RoundPhase::Placement);
        app.update();
    }

    assert!(
        submitted(&app),
        "re-assigning the same RoundPhase value must not register as a \
         transition (sentinel was clobbered by a spurious phase_changed=true)"
    );
}

fn build_app_in_phase(phase: RoundPhase) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HandUiPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_phase(&mut app, phase);
    app
}

fn app_with_hand_ui_in_phase(phase: RoundPhase) -> App {
    let mut app = build_app_in_phase(phase);
    // Run one update so the system has observed the initial phase and the
    // `Local<Option<RoundPhase>>` is primed.
    app.update();
    app
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn set_submitted_sentinel(app: &mut App) {
    app.world_mut().resource_mut::<PlacementTimer>().submitted = true;
}

fn submitted(app: &App) -> bool {
    app.world().resource::<PlacementTimer>().submitted
}
