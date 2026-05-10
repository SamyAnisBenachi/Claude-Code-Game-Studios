use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::card_animations::CardAnimationsPlugin;
use client::presentation::board_rendering::perf_harness::{
    baseline_fixture_snapshot, BoardRenderingPerfHarnessPlugin, BoardWasmPerfFixtureCounts,
    BoardWasmPerfHarnessReport, PerfBudgetStatus, BOARD_RENDERING_BASELINE_SEED,
};
use client::presentation::board_rendering::{
    BoardCellNode, BoardRenderingPlugin, BoardUnit, HpBarBackground, HpBarFill, StandingObjective,
    StatusIcon, StatusOverflowBadge,
};
use client::state::ClientState;
use client::ui::shared::{BOARD_CELL_COUNT, BOARD_LANE_COUNT};

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn test_browser_wasm_perf_harness_seeds_board_012_baseline_and_records_budgets() {
    test_helpers::init_test_tracing();
    let mut app = app_with_harness();

    for _ in 0..180 {
        app.update();
    }

    let report = app.world().resource::<BoardWasmPerfHarnessReport>();
    assert_eq!(report.seed, BOARD_RENDERING_BASELINE_SEED);
    assert_eq!(
        report.fixture_counts,
        BoardWasmPerfFixtureCounts::expected()
    );
    assert!(report.ready_for_capture);
    assert!(report.total_frame_avg_ms.is_some());
    assert!(report.total_frame_max_ms.is_some());
    assert_ne!(
        report.steady_state_presentation_budget,
        PerfBudgetStatus::NotSampled
    );
    assert_ne!(
        report.reconnect_snapshot_rebuild_budget,
        PerfBudgetStatus::NotSampled
    );
    assert_eq!(
        report.phase_boundary_presentation_spike_budget,
        PerfBudgetStatus::NotSampled
    );
    assert_eq!(
        report.screenshot_path,
        "production/qa/evidence/captures/board-rendering-baseline-1920x1080.png"
    );
    assert_eq!(
        report.trace_path,
        "production/qa/evidence/captures/board-rendering-baseline-timing.json"
    );

    let world = app.world_mut();
    let mut cells = world.query_filtered::<Entity, With<BoardCellNode>>();
    assert_eq!(
        cells.iter(world).count(),
        usize::from(BOARD_LANE_COUNT) * usize::from(BOARD_CELL_COUNT)
    );

    let mut units = world.query_filtered::<Entity, With<BoardUnit>>();
    assert_eq!(units.iter(world).count(), 20);

    let mut objectives = world.query_filtered::<Entity, With<StandingObjective>>();
    assert_eq!(objectives.iter(world).count(), 10);

    let mut hp_backgrounds = world.query_filtered::<Entity, With<HpBarBackground>>();
    assert_eq!(hp_backgrounds.iter(world).count(), 30);

    let mut hp_fills = world.query_filtered::<Entity, With<HpBarFill>>();
    assert_eq!(hp_fills.iter(world).count(), 30);

    let mut status_icons =
        world.query_filtered::<Entity, Or<(With<StatusIcon>, With<StatusOverflowBadge>)>>();
    assert_eq!(status_icons.iter(world).count(), 0);
}

#[test]
fn test_browser_wasm_perf_fixture_snapshot_uses_required_seed_counts() {
    test_helpers::init_test_tracing();
    let snapshot = baseline_fixture_snapshot();

    assert_eq!(snapshot.recipient_player_id.0, 1);
    assert_eq!(snapshot.phase, shared::protocol::RoundPhase::Resolution);
    assert_eq!(snapshot.board.units.len(), 20);
    assert_eq!(snapshot.players.len(), 2);
    assert_eq!(
        snapshot
            .players
            .iter()
            .flat_map(|player| player.objectives.iter())
            .count(),
        10
    );
    assert!(snapshot.board.units.iter().all(|unit| unit.stats.is_some()));
}

fn app_with_harness() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins((
        CardAnimationsPlugin,
        BoardRenderingPlugin,
        BoardRenderingPerfHarnessPlugin,
    ));
    app
}
