//! PROMPT 1627 — Bot Debug Overlay Integration Test.
//!
//! Closes GAP-03 from PROMPT 1622: no standalone integration test for the
//! debug bot overlay component landed in PROMPT 1617.
//!
//! Coverage:
//! 1. Enabled plugin spawns `DebugBotOverlayRoot` starting at
//!    `Visibility::Hidden`.
//! 2. `state.visible = true` mirrors to `Visibility::Visible` on the root.
//! 3. `state.visible = false` mirrors back to `Visibility::Hidden`.
//! 4. Disabled plugin spawns no `DebugBotOverlayRoot` entity.
//! 5. Disabled plugin inserts no `DebugBotOverlayEntities` resource.
//! 6. Disabled plugin still initialises the `DebugBotOverlayState` resource.
//! 7. Directly applying a payload to `state.latest` updates the body `Text`
//!    on the next update cycle (simulates what the drain system would write).
//! 8. `drain_debug_bot_state_receiver_system` is a no-op when no
//!    `MessageReceiver<S2CDebugBotStatePush>` entity is present — state stays
//!    at default values and receive_count stays at zero.
//! 9. `apply_overlay_toggle` toggles `state.visible` when called from a
//!    `World::run_system_once` closure, validating the production toggle
//!    helper in an ECS context.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use client::presentation::debug_bot_overlay::{
    apply_overlay_toggle, drain_debug_bot_state_receiver_system, DebugBotOverlayConfig,
    DebugBotOverlayEntities, DebugBotOverlayPlugin, DebugBotOverlayRoot, DebugBotOverlayState,
};
use shared::card::ClassId;
use shared::protocol::{DebugBotStateEntry, S2CDebugBotStatePush};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn enabled_overlay_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Pre-insert config so the plugin skips the env-var lookup in tests.
    app.insert_resource(DebugBotOverlayConfig { enabled: true });
    app.add_plugins(DebugBotOverlayPlugin);
    app.update(); // Startup: spawns UI entities + inserts DebugBotOverlayEntities
    app
}

fn disabled_overlay_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(DebugBotOverlayConfig { enabled: false });
    app.add_plugins(DebugBotOverlayPlugin);
    app.update();
    app
}

fn one_bot_payload() -> S2CDebugBotStatePush {
    S2CDebugBotStatePush {
        bots: vec![DebugBotStateEntry {
            player_id: PlayerId(99),
            class_id: Some(ClassId::Iop),
            gold: 7,
            current_mana: 2,
            mana_cap: 5,
            submitted: true,
            hand: vec![],
            decision_tail: vec![],
            last_bid_valuation: None,
        }],
        decision_log_total: 3,
        assembled_at_ms: 42000,
    }
}

#[test]
fn test_enabled_overlay_root_spawns_hidden() {
    test_helpers::init_test_tracing();
    let app = enabled_overlay_app();

    let entities = app.world().resource::<DebugBotOverlayEntities>();
    let visibility = app
        .world()
        .entity(entities.root)
        .get::<Visibility>()
        .expect("overlay root must carry a Visibility component");
    assert_eq!(
        *visibility,
        Visibility::Hidden,
        "overlay root must start at Visibility::Hidden when state.visible is false"
    );
}

#[test]
fn test_state_visible_true_syncs_to_visibility_visible() {
    test_helpers::init_test_tracing();
    let mut app = enabled_overlay_app();

    app.world_mut().resource_mut::<DebugBotOverlayState>().visible = true;
    app.update();

    let root = app.world().resource::<DebugBotOverlayEntities>().root;
    let visibility = *app
        .world()
        .entity(root)
        .get::<Visibility>()
        .expect("overlay root must carry a Visibility component");
    assert_eq!(
        visibility,
        Visibility::Visible,
        "state.visible=true must mirror to Visibility::Visible on the overlay root"
    );
}

#[test]
fn test_state_visible_false_syncs_to_visibility_hidden() {
    test_helpers::init_test_tracing();
    let mut app = enabled_overlay_app();

    // Raise then lower the overlay so the sync system sees both transitions.
    app.world_mut().resource_mut::<DebugBotOverlayState>().visible = true;
    app.update();
    app.world_mut().resource_mut::<DebugBotOverlayState>().visible = false;
    app.update();

    let root = app.world().resource::<DebugBotOverlayEntities>().root;
    let visibility = *app
        .world()
        .entity(root)
        .get::<Visibility>()
        .expect("overlay root must carry a Visibility component");
    assert_eq!(
        visibility,
        Visibility::Hidden,
        "state.visible=false must mirror back to Visibility::Hidden"
    );
}

#[test]
fn test_disabled_plugin_spawns_no_overlay_root() {
    test_helpers::init_test_tracing();
    let mut app = disabled_overlay_app();

    let mut q = app.world_mut().query::<&DebugBotOverlayRoot>();
    assert_eq!(
        q.iter(app.world()).count(),
        0,
        "disabled plugin must not spawn any DebugBotOverlayRoot entity"
    );
}

#[test]
fn test_disabled_plugin_inserts_no_entities_resource() {
    test_helpers::init_test_tracing();
    let app = disabled_overlay_app();

    assert!(
        !app.world().contains_resource::<DebugBotOverlayEntities>(),
        "disabled plugin must not insert DebugBotOverlayEntities resource"
    );
}

#[test]
fn test_disabled_plugin_still_initialises_state_resource() {
    test_helpers::init_test_tracing();
    let app = disabled_overlay_app();

    let state = app.world().resource::<DebugBotOverlayState>();
    assert!(
        !state.visible,
        "DebugBotOverlayState must be initialised with visible=false even when plugin is disabled"
    );
    assert!(
        state.latest.is_none(),
        "DebugBotOverlayState.latest must be None on init"
    );
    assert_eq!(
        state.receive_count, 0,
        "DebugBotOverlayState.receive_count must be 0 on init"
    );
}

#[test]
fn test_payload_in_state_latest_updates_body_text() {
    test_helpers::init_test_tracing();
    let mut app = enabled_overlay_app();

    // Confirm the body starts with the waiting placeholder.
    {
        let body_entity = app.world().resource::<DebugBotOverlayEntities>().body;
        let text = app
            .world()
            .entity(body_entity)
            .get::<Text>()
            .expect("body entity must carry a Text component");
        assert!(
            text.0.contains("Waiting"),
            "body text must start with the waiting placeholder; got {:?}",
            text.0
        );
    }

    // Simulate what drain_debug_bot_state_receiver_system would write.
    app.world_mut().resource_mut::<DebugBotOverlayState>().latest = Some(one_bot_payload());
    app.update();

    let body_entity = app.world().resource::<DebugBotOverlayEntities>().body;
    let text = app
        .world()
        .entity(body_entity)
        .get::<Text>()
        .expect("body entity must carry a Text component after payload");
    assert!(
        text.0.contains("Iop"),
        "body text must include the bot class after payload is applied; got {:?}",
        text.0
    );
    assert!(
        text.0.contains("gold=7"),
        "body text must include the bot gold value; got {:?}",
        text.0
    );
    assert!(
        text.0.contains("42000"),
        "body text must include assembled_at_ms from the payload; got {:?}",
        text.0
    );
}

#[test]
fn test_drain_system_noop_when_no_receiver_entity_present() {
    test_helpers::init_test_tracing();
    let mut world = World::new();
    world.insert_resource(DebugBotOverlayState::default());

    world
        .run_system_once(drain_debug_bot_state_receiver_system)
        .expect("drain system must not panic when no MessageReceiver entity is present");

    let state = world.resource::<DebugBotOverlayState>();
    assert!(
        state.latest.is_none(),
        "state.latest must remain None when no receiver entity is present"
    );
    assert_eq!(
        state.receive_count, 0,
        "receive_count must stay at 0 when the drain system has nothing to drain"
    );
}

#[test]
fn test_apply_overlay_toggle_via_run_system_once() {
    test_helpers::init_test_tracing();
    let mut world = World::new();
    world.insert_resource(DebugBotOverlayState::default());

    assert!(
        !world.resource::<DebugBotOverlayState>().visible,
        "initial state must be visible=false"
    );

    world
        .run_system_once(|mut state: ResMut<DebugBotOverlayState>| {
            apply_overlay_toggle(&mut state);
        })
        .expect("first toggle closure must run without error");

    assert!(
        world.resource::<DebugBotOverlayState>().visible,
        "apply_overlay_toggle must set visible=true from initial false"
    );

    world
        .run_system_once(|mut state: ResMut<DebugBotOverlayState>| {
            apply_overlay_toggle(&mut state);
        })
        .expect("second toggle closure must run without error");

    assert!(
        !world.resource::<DebugBotOverlayState>().visible,
        "second apply_overlay_toggle must restore visible=false"
    );
}
