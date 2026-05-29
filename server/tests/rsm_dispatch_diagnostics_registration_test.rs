//! PROMPT 2060 — Regression coverage for the missing-registration caveat in
//! `ServerNetworkPlugin`.
//!
//! PROMPT 2043 added [`RsmDispatchDiagnostics`] as the queryable side-effect
//! of the `MissingSender` / `MissingServer` warn branches in
//! `server::network::rsm_dispatch`. The dispatcher systems take the resource
//! as `Option<ResMut<_>>`, so the warn fires regardless — but the counter
//! silently no-ops when the resource is not registered on the App.
//!
//! PROMPT 2051 (stale partial) surfaced the live caveat: production's
//! `ServerNetworkPlugin` had no `init_resource::<RsmDispatchDiagnostics>()`
//! call, leaving the counter invisible in production while the warn alone
//! carried the signal. PROMPT 2060 fixes that by routing the init through
//! [`register_rsm_dispatch_diagnostics`] from inside the plugin build.
//!
//! These tests pin the registration contract:
//!   * The standalone helper initialises the resource to its default value.
//!   * Calling the helper twice is idempotent (no panic, no overwrite of a
//!     non-default value — `init_resource` is a no-op when present).
//!
//! The integration check that `ServerNetworkPlugin::build` calls the helper
//! is enforced by the production `main.rs` wiring (single source of truth)
//! plus this file's coverage of the helper itself. We do not instantiate
//! `ServerNetworkPlugin` directly because it adds lightyear `ServerPlugins`
//! and a `Startup` system that binds TCP port 5000, the same reason the
//! shared `production_server_app_factory` omits the plugin.

use bevy::prelude::*;
use server::network::register_rsm_dispatch_diagnostics;
use server::network::rsm_dispatch::RsmDispatchDiagnostics;

#[test]
fn test_register_rsm_dispatch_diagnostics_inserts_resource_at_default() {
    // Arrange
    let mut app = App::new();
    assert!(
        app.world().get_resource::<RsmDispatchDiagnostics>().is_none(),
        "precondition: resource must be absent before registration"
    );

    // Act
    register_rsm_dispatch_diagnostics(&mut app);

    // Assert
    let diagnostics = app
        .world()
        .get_resource::<RsmDispatchDiagnostics>()
        .expect("register_rsm_dispatch_diagnostics must insert the resource");
    assert_eq!(
        *diagnostics,
        RsmDispatchDiagnostics::default(),
        "newly-registered counter must be at default (all zeros)"
    );
}

#[test]
fn test_register_rsm_dispatch_diagnostics_is_idempotent_and_preserves_existing_counter() {
    // Arrange: simulate a prior registration plus accumulated counts. This
    // models a re-build of the plugin in a test harness, or an unexpected
    // double-registration in a future refactor.
    let mut app = App::new();
    register_rsm_dispatch_diagnostics(&mut app);
    {
        let mut diagnostics = app
            .world_mut()
            .resource_mut::<RsmDispatchDiagnostics>();
        diagnostics.phase_changed_dropped_missing_sender = 7;
        diagnostics.opponent_disconnected_dropped_missing_server = 3;
    }

    // Act: registering again must not panic and must not clobber the
    // already-accumulated counters (Bevy's `init_resource` is a no-op when
    // the resource is already present).
    register_rsm_dispatch_diagnostics(&mut app);

    // Assert
    let post = app
        .world()
        .resource::<RsmDispatchDiagnostics>()
        .clone();
    assert_eq!(post.phase_changed_dropped_missing_sender, 7);
    assert_eq!(post.opponent_disconnected_dropped_missing_server, 3);
    assert_eq!(post.phase_changed_dropped_missing_server, 0);
    assert_eq!(post.opponent_disconnected_dropped_missing_sender, 0);
}
