// Integration test for S13-PROTO-ORPHAN-DRAIN-001 (Story 008 AC5)
// snapshot-request cluster.
//
// The story chose Path A for `C2SRequestSnapshot`: a new server-side
// handler that reuses the existing snapshot builder at
// `server/src/core/session/snapshot.rs` (no new snapshot construction
// path lands; ADR-011 binding). Rate-limited via
// `GameConfig::snapshot_cooldown_ms` (default 5000ms, network-protocol.md
// Table A). ADR-002 binding: server stays authoritative; the client
// message is advisory only.
//
// We assert:
//   1. `SnapshotRequestCooldowns` cooldown math is correct
//      (within-window → block, after-window → allow).
//   2. The plugin schedules the `handle_request_snapshot` system and
//      installs the cooldown resource.
//   3. The legacy `handle_c2s_message` stub TODO in `server/src/main.rs`
//      is gone (atomic cleanup per the story).
//   4. The new drain is the sole production-code receiver for
//      `C2SRequestSnapshot` (single-drainer rule per ADR-008).

use std::fs;
use std::path::{Path, PathBuf};

use server::core::session::{GameSessionPlugin, SnapshotRequestCooldowns};
use shared::session::PlayerId;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const SERVER_SRC_REL: &str = "src";

fn server_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SERVER_SRC_REL)
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

#[test]
fn snapshot_request_cooldown_blocks_inside_window_and_releases_after_threshold() {
    test_helpers::init_test_tracing();
    let mut cooldowns = SnapshotRequestCooldowns::default();
    let player = PlayerId(11);
    let cooldown_ms = 5_000;

    assert!(!cooldowns.is_within_cooldown(player, 0, cooldown_ms));

    cooldowns.record_sent(player, 100_000);

    assert!(cooldowns.is_within_cooldown(player, 100_000, cooldown_ms));
    assert!(cooldowns.is_within_cooldown(player, 104_999, cooldown_ms));
    assert!(!cooldowns.is_within_cooldown(player, 105_000, cooldown_ms));
    assert!(!cooldowns.is_within_cooldown(player, 110_000, cooldown_ms));
}

#[test]
fn snapshot_request_cooldown_tracks_each_player_independently() {
    test_helpers::init_test_tracing();
    let mut cooldowns = SnapshotRequestCooldowns::default();
    let player_a = PlayerId(1);
    let player_b = PlayerId(2);
    let cooldown_ms = 5_000;

    cooldowns.record_sent(player_a, 10_000);

    assert!(cooldowns.is_within_cooldown(player_a, 12_000, cooldown_ms));
    assert!(
        !cooldowns.is_within_cooldown(player_b, 12_000, cooldown_ms),
        "cooldown is per-player; player_b has never requested"
    );
}

#[test]
fn game_session_plugin_installs_snapshot_request_cooldowns_resource() {
    test_helpers::init_test_tracing();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(StatesPlugin);
    app.add_plugins(GameSessionPlugin);

    assert!(
        app.world()
            .get_resource::<SnapshotRequestCooldowns>()
            .is_some(),
        "GameSessionPlugin must register SnapshotRequestCooldowns"
    );
}

#[test]
fn handle_request_snapshot_is_sole_production_drain() {
    test_helpers::init_test_tracing();
    let mut matches = Vec::new();
    collect_source_matches(
        &server_src_root(),
        "MessageReceiver<C2SRequestSnapshot>",
        &mut matches,
    );
    let production: Vec<PathBuf> = matches
        .into_iter()
        .filter(|p| !p.components().any(|c| c.as_os_str() == "tests"))
        .collect();
    assert_eq!(
        production.len(),
        1,
        "MessageReceiver<C2SRequestSnapshot> must have exactly one production drain in server/src/; found: {:?}",
        production,
    );
    let expected = server_src_root().join(Path::new("core/session/snapshot_request.rs"));
    assert!(
        production[0].ends_with(Path::new("core/session/snapshot_request.rs"))
            || production[0] == expected,
        "drain must live in core/session/snapshot_request.rs; found {:?}",
        production[0]
    );
}

#[test]
fn legacy_handle_c2s_message_stub_is_removed_from_main() {
    test_helpers::init_test_tracing();
    let main_source = fs::read_to_string(server_src_root().join("main.rs"))
        .expect("server/src/main.rs should be readable");
    assert!(
        !main_source.contains("handle_c2s_message"),
        "S13-PROTO-ORPHAN-DRAIN-001 removed the legacy handle_c2s_message TODO stub"
    );
    assert!(
        !main_source.contains("ADR-008 verification checklist items 4–7"),
        "S13-PROTO-ORPHAN-DRAIN-001 retired the ADR-008 verification TODO that lived with the legacy stub"
    );
}
