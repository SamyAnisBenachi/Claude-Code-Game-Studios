// ADR-012 Open Condition: Commands::trigger() flush ordering in Bevy 0.18
//
// Verifies that Res<SessionConfig> inserted via Commands::insert_resource()
// BEFORE Commands::trigger(SessionReady) is visible inside the Observer handler
// in the same Update tick — with no apply_deferred between them.
//
// PASS → "ADR-012 open condition: RESOLVED — flush ordering confirmed, no apply_deferred needed"
// FAIL → add apply_deferred to RsmPlugin::build() chain (document fix in evidence file)
//
// Run: cargo test -p server session_ready_observer

use bevy::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Event)]
struct SessionReady;

#[derive(Resource)]
struct SessionConfig {
    value: u32,
}

// One-shot guard: prevent the trigger system from re-firing on subsequent updates.
#[derive(Resource)]
struct TriggerFired;

/// Verifies that Commands::trigger() dispatches the Observer in the same
/// Update tick as the trigger call — not deferred to the next frame.
#[test]
fn test_session_ready_observer_fires_in_same_frame() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let observer_fired = Arc::new(AtomicBool::new(false));
    let flag = observer_fired.clone();

    app.observe(move |_trigger: Trigger<SessionReady>| {
        flag.store(true, Ordering::SeqCst);
    });

    app.add_systems(Update, |mut commands: Commands, guard: Option<Res<TriggerFired>>| {
        if guard.is_some() {
            return;
        }
        commands.insert_resource(TriggerFired);
        commands.trigger(SessionReady);
    });

    // Act: one update tick — system queues trigger, commands flush, observer runs
    app.update();

    // Assert: Observer must have fired within the same update() call
    assert!(
        observer_fired.load(Ordering::SeqCst),
        "ADR-012 FAIL: SessionReady Observer did not fire in the same update() frame. \
         Commands::trigger() defers to next frame — switch to World::trigger() in an \
         exclusive system to guarantee same-tick dispatch.",
    );
}

/// Core ADR-012 open condition: Res<SessionConfig> inserted via
/// Commands::insert_resource() immediately before Commands::trigger(SessionReady)
/// must be visible inside the Observer handler — no apply_deferred between them.
///
/// This is the critical invariant for the GSS → RSM handoff:
///   commands.insert_resource(SessionConfig { ... });   // step 3
///   commands.insert_resource(ServerRng::new(seed));    // step 4
///   commands.trigger(SessionReady);                    // step 5 — observer fires
///
/// If this test PASSES: implement GSS per ADR-012 Decision (Commands path).
/// If this test FAILS: use the exclusive-system fallback (World::insert_resource
/// + World::trigger) documented in ADR-012 §Alternative 2.
#[test]
fn test_session_ready_observer_resource_visible_after_commands_insert() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let config_visible = Arc::new(AtomicBool::new(false));
    let flag = config_visible.clone();

    // Observer accesses Res<SessionConfig> — will panic if resource not present
    // when triggered (which would also constitute a test failure).
    app.observe(move |_trigger: Trigger<SessionReady>, config: Res<SessionConfig>| {
        flag.store(config.value == 42, Ordering::SeqCst);
    });

    app.add_systems(Update, |mut commands: Commands, guard: Option<Res<TriggerFired>>| {
        if guard.is_some() {
            return;
        }
        commands.insert_resource(TriggerFired);
        // Order matters: insert_resource before trigger — this is what ADR-012 relies on.
        commands.insert_resource(SessionConfig { value: 42 });
        commands.trigger(SessionReady);
    });

    // Act: one update — system queues (insert_resource, trigger) in order,
    // commands flush in order: insert first, then observer fires.
    app.update();

    // Assert
    assert!(
        config_visible.load(Ordering::SeqCst),
        "ADR-012 FAIL: Res<SessionConfig> was NOT visible in SessionReady Observer. \
         Commands::insert_resource() before Commands::trigger() does not guarantee \
         the resource is applied before the Observer fires. \
         Fix: switch GSS to exclusive system using World::insert_resource() + \
         World::trigger(SessionReady) — see ADR-012 §Alternative 2.",
    );
}
