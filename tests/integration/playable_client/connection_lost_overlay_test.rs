//! Integration tests for S13-CONN-LOST-UX-001 (Story 021).
//!
//! Asserts that the proactive Reconnecting / Connection Lost overlay's
//! visibility transitions match AC3 (transport drop -> visible), AC4
//! (reconnect -> hidden), AC5 (GameOver -> hidden), and AC7 (overlay sits
//! below the result screen). AC1 / AC2 are covered by source-level grep
//! assertions over the production overlay module + presentation composition.
//!
//! No optimistic client-side authority is introduced or relied upon by these
//! tests; the overlay state is a read-only projection over the lightyear
//! transport events and the authoritative `CurrentClientPhase` resource.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use client::presentation::connection_lost_overlay::{
    dismiss_overlay_on_game_over_system, handle_transport_connected_event,
    handle_transport_disconnected_event, overlay_dismissed_by_phase,
    should_show_overlay_for_client_state, ConnectionLostOverlayEntities,
    ConnectionLostOverlayPlugin, ConnectionLostOverlayRoot, ConnectionLostOverlayState,
    CONNECTION_LOST_OVERLAY_Z_INDEX,
};
use client::state::{ClientState, CurrentClientPhase};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn client_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn read_to_string(rel: &[&str]) -> String {
    let mut path = client_src_root();
    for segment in rel {
        path.push(segment);
    }
    fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("could not read {} for AC source grep", path.display()))
}

fn make_overlay_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ConnectionLostOverlayPlugin);
    // Startup runs on first update: spawns overlay nodes + inserts the
    // ConnectionLostOverlayEntities resource.
    app.update();
    app
}

#[test]
fn ac2_predicate_should_show_overlay_only_in_session() {
    assert!(
        !should_show_overlay_for_client_state(ClientState::Lobby),
        "Lobby state must not raise the in-session overlay"
    );
    assert!(
        should_show_overlay_for_client_state(ClientState::InSession),
        "InSession must raise the overlay on transport drop"
    );
}

#[test]
fn ac5_predicate_overlay_dismissed_by_phase_only_at_game_over() {
    assert!(overlay_dismissed_by_phase(RoundPhase::GameOver));
    for phase in [
        RoundPhase::Lobby,
        RoundPhase::Handshaking,
        RoundPhase::DraftInitial,
        RoundPhase::DraftShop,
        RoundPhase::DraftAuction,
        RoundPhase::Placement,
        RoundPhase::Resolution,
    ] {
        assert!(
            !overlay_dismissed_by_phase(phase),
            "phase {phase:?} must not dismiss the overlay; only GameOver does"
        );
    }
}

#[test]
fn ac3_disconnect_handler_marks_overlay_visible_in_session() {
    let mut state = ConnectionLostOverlayState::default();
    assert!(!state.visible);
    handle_transport_disconnected_event(ClientState::InSession, &mut state);
    assert!(
        state.visible,
        "transport disconnect during InSession must mark the overlay visible (AC3)"
    );
}

#[test]
fn ac3_disconnect_handler_does_not_mark_overlay_visible_in_lobby() {
    let mut state = ConnectionLostOverlayState::default();
    handle_transport_disconnected_event(ClientState::Lobby, &mut state);
    assert!(
        !state.visible,
        "Lobby disconnect must not raise the in-session overlay (AC3 scoping)"
    );
}

#[test]
fn ac4_connected_handler_marks_overlay_hidden() {
    let mut state = ConnectionLostOverlayState { visible: true };
    handle_transport_connected_event(&mut state);
    assert!(
        !state.visible,
        "transport reconnect must dismiss the overlay within one frame (AC4)"
    );
}

#[test]
fn ac4_connected_handler_is_idempotent_when_hidden() {
    let mut state = ConnectionLostOverlayState::default();
    handle_transport_connected_event(&mut state);
    assert!(!state.visible);
}

#[test]
fn ac5_dismiss_system_hides_overlay_when_phase_is_game_over() {
    let mut world = World::new();
    world.insert_resource(ConnectionLostOverlayState { visible: true });
    world.insert_resource(CurrentClientPhase {
        phase: RoundPhase::GameOver,
        round: 4,
    });

    world
        .run_system_once(dismiss_overlay_on_game_over_system)
        .unwrap();

    assert!(
        !world.resource::<ConnectionLostOverlayState>().visible,
        "RoundPhase::GameOver must dismiss the overlay within one frame (AC5)"
    );
}

#[test]
fn ac5_dismiss_system_is_noop_during_active_gameplay() {
    let mut world = World::new();
    world.insert_resource(ConnectionLostOverlayState { visible: true });
    world.insert_resource(CurrentClientPhase {
        phase: RoundPhase::Placement,
        round: 3,
    });

    world
        .run_system_once(dismiss_overlay_on_game_over_system)
        .unwrap();

    assert!(
        world.resource::<ConnectionLostOverlayState>().visible,
        "non-GameOver phases must not dismiss the overlay (AC5 negative case)"
    );
}

#[test]
fn ac3_ac4_sync_system_mirrors_state_to_root_visibility() {
    test_helpers::init_test_tracing();
    let mut app = make_overlay_app();

    let root = app.world().resource::<ConnectionLostOverlayEntities>().root;

    let visibility_before = *app
        .world()
        .entity(root)
        .get::<Visibility>()
        .expect("overlay root must carry a Visibility component");
    assert_eq!(visibility_before, Visibility::Hidden);

    app.world_mut()
        .resource_mut::<ConnectionLostOverlayState>()
        .visible = true;
    app.update();

    let visibility_after = *app
        .world()
        .entity(root)
        .get::<Visibility>()
        .expect("overlay root must carry a Visibility component");
    assert_eq!(
        visibility_after,
        Visibility::Visible,
        "state.visible=true must mirror to Visibility::Visible (AC3)"
    );

    app.world_mut()
        .resource_mut::<ConnectionLostOverlayState>()
        .visible = false;
    app.update();

    let visibility_final = *app
        .world()
        .entity(root)
        .get::<Visibility>()
        .expect("overlay root must carry a Visibility component");
    assert_eq!(
        visibility_final,
        Visibility::Hidden,
        "state.visible=false must mirror to Visibility::Hidden (AC4)"
    );
}

#[test]
fn ac1_overlay_root_carries_marker_component_for_query_targeting() {
    let app = make_overlay_app();

    let entities = app.world().resource::<ConnectionLostOverlayEntities>();
    assert!(
        app.world()
            .entity(entities.root)
            .get::<ConnectionLostOverlayRoot>()
            .is_some(),
        "overlay root must carry the ConnectionLostOverlayRoot marker (AC1)"
    );
}

#[test]
fn ac7_overlay_z_index_is_below_result_screen() {
    // ResultScreenPlugin uses GlobalZIndex(100); the overlay must sit below
    // so that on GameOver the result screen visually overlays it.
    assert!(
        CONNECTION_LOST_OVERLAY_Z_INDEX < 100,
        "Connection-lost overlay z-index must stay below the result screen (AC7)"
    );
    assert!(
        CONNECTION_LOST_OVERLAY_Z_INDEX > 0,
        "Connection-lost overlay z-index must sit above gameplay UI (AC7)"
    );
}

#[test]
fn ac7_overlay_backdrop_alpha_lets_gameplay_show_through() {
    // Source-level audit: the spawn function uses a backdrop alpha strictly
    // less than the result screen's 0.46 so gameplay UI is visible through
    // the overlay (AC7 - non-blocking visual modal).
    let text = read_to_string(&["presentation", "connection_lost_overlay.rs"]);
    assert!(
        text.contains("Color::srgba(0.02, 0.025, 0.035, 0.32)"),
        "overlay backdrop must use alpha 0.32 (< result screen's 0.46) per AC7"
    );
}

#[test]
fn ac1_plugin_registered_in_presentation_composition_order() {
    let text = read_to_string(&["presentation", "mod.rs"]);
    assert!(
        text.contains("ConnectionLostOverlayPlugin"),
        "PresentationPlugin must register ConnectionLostOverlayPlugin per ADR-021 (AC1)"
    );
    assert!(
        text.contains("ResultScreenPlugin"),
        "ResultScreenPlugin must remain part of the PresentationPlugin order"
    );
    // ADR-021 contract: the plugin sits after ResultScreenPlugin in the
    // composition order. Order matters as a stable contract for downstream
    // agents reading this composition.
    let result_idx = text
        .find("app.add_plugins(ResultScreenPlugin)")
        .expect("ResultScreenPlugin must be added via app.add_plugins");
    let overlay_idx = text
        .find("app.add_plugins(ConnectionLostOverlayPlugin)")
        .expect("ConnectionLostOverlayPlugin must be added via app.add_plugins");
    assert!(
        overlay_idx > result_idx,
        "ConnectionLostOverlayPlugin must be registered AFTER ResultScreenPlugin per ADR-021"
    );
}

#[test]
fn ac2_overlay_subscribes_to_lightyear_transport_event_sources() {
    let text = read_to_string(&["presentation", "connection_lost_overlay.rs"]);
    assert!(
        text.contains("On<Add, Disconnected>"),
        "overlay must subscribe to lightyear Disconnected via On<Add, Disconnected> (AC2)"
    );
    assert!(
        text.contains("On<Add, Connected>"),
        "overlay must subscribe to lightyear Connected via On<Add, Connected> (AC2 / AC4)"
    );
    assert!(
        text.contains("add_observer(on_transport_disconnected)"),
        "ConnectionLostOverlayPlugin must register the disconnected observer (AC2)"
    );
    assert!(
        text.contains("add_observer(on_transport_connected)"),
        "ConnectionLostOverlayPlugin must register the connected observer (AC2)"
    );
}

#[test]
fn ac9_overlay_module_does_not_mutate_authoritative_state() {
    let text = read_to_string(&["presentation", "connection_lost_overlay.rs"]);
    // The overlay module must never write to authoritative state surfaces.
    // `MessageSender` / `MessageWriter` / `NextState` are the canonical write
    // surfaces; a read-only projection must not import or construct them.
    assert!(
        !text.contains("MessageSender"),
        "overlay must not emit network messages (AC9 / ADR-002)"
    );
    assert!(
        !text.contains("MessageWriter"),
        "overlay must not synthesise S2C messages (AC9 / ADR-002)"
    );
    assert!(
        !text.contains("NextState"),
        "overlay must not transition the client lifecycle (AC9 / ADR-002)"
    );
    assert!(
        text.contains("No optimistic client-side authority")
            || text.contains("no optimistic client-side authority"),
        "overlay module header must restate the no-optimistic-authority guarantee (AC9 evidence)"
    );
}

#[test]
fn ac10_no_protocol_or_server_changes_in_story_scope() {
    // Sanity audit: the overlay module must not reference any server-side
    // crate path or import items from `server::`. The `shared` import is
    // limited to `shared::protocol::RoundPhase`, which is read-only enum data.
    let text = read_to_string(&["presentation", "connection_lost_overlay.rs"]);
    assert!(
        !text.contains("use server::"),
        "overlay must not depend on the server crate (AC10)"
    );
    assert!(
        !text.contains("crate::network"),
        "overlay must not bypass into network internals (AC10 spirit)"
    );
}
