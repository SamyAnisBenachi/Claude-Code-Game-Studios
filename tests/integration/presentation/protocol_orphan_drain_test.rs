// Integration tests for S13-PROTO-ORPHAN-DRAIN-001 (Story 008 AC5).
//
// One test per Path A cluster (lifecycle / prism / snapshot-request) asserts
// the newly added drain is invoked when its corresponding S2C/C2S message
// is sent. Follows the
// `tests/integration/session/result_acknowledgement_contract_test.rs`
// precedent: apply-functions exercised directly + source-grep guard
// proving exactly one production drain exists for each message
// (single-drainer rule per ADR-008).

use std::fs;
use std::path::{Path, PathBuf};

use client::presentation::PresentationPlugin;
use client::state::{
    apply_opponent_disconnected_message, apply_opponent_reconnected_message,
    apply_prism_respawned_message, apply_prism_reward_dropped_message,
    apply_session_cancelled_message, OpponentConnectionView, OpponentDisconnectIndicator,
    PrismLifecycleView, PrismRespawnEvent, PrismRewardDroppedEvent, SessionLifecycleView,
};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use shared::protocol::{
    S2COpponentDisconnected, S2COpponentReconnected, S2CPrismRespawned, S2CPrismRewardDropped,
    S2CSessionCancelled, SessionCancelledReason,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const CLIENT_SRC_REL: &str = "src";

fn client_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(CLIENT_SRC_REL)
}

fn collect_source_matches(path: &Path, needle: &str, matches: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_source_matches(&path, needle, matches);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "rs") {
            if let Ok(text) = fs::read_to_string(&path) {
                if text.contains(needle) {
                    matches.push(path);
                }
            }
        }
    }
}

fn assert_single_production_drain(needle: &str) {
    let mut matches = Vec::new();
    collect_source_matches(&client_src_root(), needle, &mut matches);
    let production: Vec<PathBuf> = matches
        .into_iter()
        .filter(|p| !p.components().any(|c| c.as_os_str() == "tests"))
        .collect();
    assert_eq!(
        production.len(),
        1,
        "{} must have exactly one production drain in client/src/; found: {:?}",
        needle,
        production,
    );
}

#[test]
fn s2c_opponent_disconnect_and_reconnect_pair_apply_to_connection_view() {
    test_helpers::init_test_tracing();
    let mut view = OpponentConnectionView::default();
    let opponent = PlayerId(42);

    apply_opponent_disconnected_message(
        &S2COpponentDisconnected {
            player_id: opponent,
            grace_remaining_ms: 25_000,
        },
        &mut view,
    );
    assert_eq!(
        view.disconnected,
        Some(OpponentDisconnectIndicator {
            player_id: opponent,
            grace_remaining_ms: 25_000,
        })
    );

    apply_opponent_disconnected_message(
        &S2COpponentDisconnected {
            player_id: opponent,
            grace_remaining_ms: 10_000,
        },
        &mut view,
    );
    assert_eq!(
        view.disconnected.map(|d| d.grace_remaining_ms),
        Some(10_000),
        "last-write-wins on disconnect updates"
    );

    apply_opponent_reconnected_message(
        &S2COpponentReconnected {
            player_id: opponent,
        },
        &mut view,
    );
    assert_eq!(
        view.disconnected, None,
        "reconnect clears the disconnect indicator"
    );
}

#[test]
fn s2c_session_cancelled_applies_to_session_lifecycle_view() {
    test_helpers::init_test_tracing();
    let mut view = SessionLifecycleView::default();
    assert_eq!(view.cancellation, None);

    apply_session_cancelled_message(
        &S2CSessionCancelled {
            reason: SessionCancelledReason::LobbyTimeout,
        },
        &mut view,
    );
    assert_eq!(
        view.cancellation,
        Some(SessionCancelledReason::LobbyTimeout)
    );

    apply_session_cancelled_message(
        &S2CSessionCancelled {
            reason: SessionCancelledReason::PlayerDisconnected,
        },
        &mut view,
    );
    assert_eq!(
        view.cancellation,
        Some(SessionCancelledReason::PlayerDisconnected),
        "last-write-wins on subsequent cancel reasons"
    );
}

#[test]
fn lifecycle_cluster_drains_are_registered_exactly_once_in_production() {
    test_helpers::init_test_tracing();
    assert_single_production_drain("MessageReceiver<S2COpponentDisconnected>");
    assert_single_production_drain("MessageReceiver<S2COpponentReconnected>");
    assert_single_production_drain("MessageReceiver<S2CSessionCancelled>");
}

#[test]
fn s2c_prism_respawned_and_reward_dropped_apply_to_lifecycle_view() {
    test_helpers::init_test_tracing();
    let mut view = PrismLifecycleView::default();
    let owner = PlayerId(7);

    apply_prism_respawned_message(&S2CPrismRespawned { player_id: owner }, &mut view);
    assert_eq!(
        view.last_respawn,
        Some(PrismRespawnEvent { player_id: owner })
    );

    apply_prism_reward_dropped_message(
        &S2CPrismRewardDropped {
            player_id: owner,
            lane: 2,
        },
        &mut view,
    );
    apply_prism_reward_dropped_message(
        &S2CPrismRewardDropped {
            player_id: owner,
            lane: 5,
        },
        &mut view,
    );

    assert_eq!(
        view.pending_rewards_lost,
        vec![
            PrismRewardDroppedEvent {
                player_id: owner,
                lane: 2,
            },
            PrismRewardDroppedEvent {
                player_id: owner,
                lane: 5,
            },
        ],
        "every reward-dropped event is appended; client never silently coalesces drops"
    );
}

#[test]
fn prism_cluster_drains_are_registered_exactly_once_in_production() {
    test_helpers::init_test_tracing();
    assert_single_production_drain("MessageReceiver<S2CPrismRespawned>");
    assert_single_production_drain("MessageReceiver<S2CPrismRewardDropped>");
}

#[test]
fn presentation_plugin_initialises_orphan_drain_views() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.add_plugins(PresentationPlugin);
    app.update();

    assert!(app
        .world()
        .get_resource::<OpponentConnectionView>()
        .is_some());
    assert!(app.world().get_resource::<PrismLifecycleView>().is_some());
    assert!(app.world().get_resource::<SessionLifecycleView>().is_some());
}
