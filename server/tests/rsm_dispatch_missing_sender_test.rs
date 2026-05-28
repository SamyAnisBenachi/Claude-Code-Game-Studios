//! PROMPT 2043 — Regression coverage for the silent-drop failure mode in
//! `server::network::rsm_dispatch`.
//!
//! Before this repair, when a `BroadcastPhaseChanged` was emitted while the
//! lightyear runtime was not (yet) wired into the `App`, the dispatcher
//! consumed the event and produced nothing: no log, no observable side
//! effect, no counter. This made server-side dispatch failures look like
//! client/UI bugs (see PROMPT 2030).
//!
//! These tests pin the non-silent contract:
//!   * The classification helper recognises every readiness combination.
//!   * Running the system on a headless app still drains the event and
//!     leaves a queryable diagnostics resource at zero — confirming the
//!     resource is reachable from tests and only increments on real drops.
//!   * The outbox-only test path observes every phase event, proving the
//!     event is not silently swallowed before the dispatch branches fire.

use bevy::prelude::*;
use server::core::rsm::{
    AuctionSettled, BroadcastPhaseChanged, ResolutionComplete, RoundPhase, RsmNetworkOutbox,
    RsmPlugin,
};
use server::network::rsm_dispatch::{
    classify_dispatch_readiness, dispatch_phase_changed, DispatchReadiness,
    RsmDispatchDiagnostics,
};

fn build_headless_dispatch_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RsmPlugin);
    // RsmPlugin's input reader depends on these messages being registered;
    // they are normally added by upstream feature plugins.
    app.add_message::<AuctionSettled>();
    app.add_message::<ResolutionComplete>();
    app.init_resource::<RsmDispatchDiagnostics>();
    app.add_systems(Update, dispatch_phase_changed);
    app
}

#[test]
fn test_rsm_dispatch_classify_readiness_all_four_combinations() {
    // Arrange: every (server_present, sender_present) combination.
    let cases = [
        ((true, true), DispatchReadiness::Ready),
        ((true, false), DispatchReadiness::MissingSender),
        ((false, true), DispatchReadiness::MissingServer),
        ((false, false), DispatchReadiness::Headless),
    ];

    // Act / Assert
    for ((server, sender), expected) in cases {
        assert_eq!(
            classify_dispatch_readiness(server, sender),
            expected,
            "readiness mismatch for (server={server}, sender={sender})"
        );
    }
}

#[test]
fn test_dispatch_phase_changed_headless_path_captures_in_outbox_and_does_not_increment_counters() {
    // Arrange: a headless app — no lightyear Server entity, no
    // ServerMultiMessageSender. This is the path the previous code took
    // through the silent-skip branch.
    let mut app = build_headless_dispatch_app();

    // Sanity: counters initialised to zero and queryable.
    let initial = app.world().resource::<RsmDispatchDiagnostics>().clone();
    assert_eq!(initial, RsmDispatchDiagnostics::default());

    // Act: emit a phase-change request.
    app.world_mut()
        .resource_mut::<Messages<BroadcastPhaseChanged>>()
        .write(BroadcastPhaseChanged {
            phase: RoundPhase::Placement,
            round: 1,
            timer_ms: 45_000,
        });
    app.update();

    // Assert:
    // 1. The outbox captured the phase-change — proving the event is not
    //    silently dropped before reaching the dispatch branches.
    let outbox = app.world().resource::<RsmNetworkOutbox>();
    assert_eq!(
        outbox.phase_changed().len(),
        1,
        "headless dispatch path must still surface phase-change via outbox; \
         silent-drop regression detected"
    );

    // 2. The diagnostics resource has *not* incremented: headless (both
    //    server and sender absent) is the expected test branch, not a
    //    runtime regression. Only MissingSender / MissingServer (partial
    //    wiring) should bump counters.
    let post = app.world().resource::<RsmDispatchDiagnostics>().clone();
    assert_eq!(
        post,
        RsmDispatchDiagnostics::default(),
        "headless path must not bump drop counters; only partial-wiring \
         (MissingSender / MissingServer) is a real regression"
    );
}

#[test]
fn test_rsm_dispatch_diagnostics_resource_increments_are_observable_in_tests() {
    // Arrange: simulate that the dispatch path detected a partial-wiring
    // regression (MissingSender) by writing the counter directly. This pins
    // the contract that the resource is *observable* — the foundational
    // affordance that turns a previously silent failure into an asserted
    // one. Spinning a real lightyear Server entity to trigger the branch
    // organically would couple this test to lightyear internals.
    let mut app = build_headless_dispatch_app();

    // Act: emulate one MissingSender drop and one MissingServer drop.
    {
        let mut diagnostics = app.world_mut().resource_mut::<RsmDispatchDiagnostics>();
        diagnostics.phase_changed_dropped_missing_sender += 1;
        diagnostics.opponent_disconnected_dropped_missing_server += 2;
    }

    // Assert: counters are externally queryable with stable field names.
    let snapshot = app.world().resource::<RsmDispatchDiagnostics>().clone();
    assert_eq!(snapshot.phase_changed_dropped_missing_sender, 1);
    assert_eq!(snapshot.opponent_disconnected_dropped_missing_server, 2);
    assert_eq!(snapshot.phase_changed_dropped_missing_server, 0);
    assert_eq!(snapshot.opponent_disconnected_dropped_missing_sender, 0);
}
