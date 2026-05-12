//! S11-HUD-TIMER-BAR-VISIBILITY-001 — wire the `HudTimerBar` entity to a
//! ticking `PhaseTimerState` driven by `ClientPhaseView.timer_duration_ms`.
//!
//! Coverage:
//! - Reset on phase change writes the new duration into `PhaseTimerState`.
//! - Tick advances `elapsed_ms` by `Time::delta()` while the timer is active.
//! - `sync_hud_timer_bar_system` scales `Node.width` per remaining ratio.
//! - The bar is hidden when `timer_duration_ms == 0` (phases without a
//!   countdown — Resolution, Lobby, etc.).

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimePlugin;
use client::{
    state::{ClientPhaseView, ClientState},
    ui::hud::{
        HudEntities, HudPlayerIds, HudPlugin, PhaseTimerState, HUD_PHASE_TIMER_BAR_MAX_WIDTH_PX,
    },
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

// ── Sub-test 1: Reset on phase change with non-zero duration ─────────────────

#[test]
fn test_reset_on_phase_change_with_duration() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();

    // Pre-condition: timer starts inactive (PhaseTimerState::default()).
    {
        let timer = app.world().resource::<PhaseTimerState>();
        assert_eq!(timer.duration_ms, 0);
        assert_eq!(timer.elapsed_ms, 0);
        assert!(!timer.active);
    }

    // Drive a phase change with a 30s timer (e.g. DraftShop).
    set_phase_view(&mut app, 30_000);
    app.update();

    let timer = app.world().resource::<PhaseTimerState>();
    assert_eq!(
        timer.duration_ms, 30_000,
        "Reset must copy ClientPhaseView.timer_duration_ms into PhaseTimerState"
    );
    assert_eq!(
        timer.elapsed_ms, 0,
        "Reset must zero elapsed_ms on phase change"
    );
    assert!(
        timer.active,
        "Reset must set active=true when duration_ms > 0"
    );
}

// ── Sub-test 2: Tick increments elapsed while active ─────────────────────────

#[test]
fn test_tick_increments_elapsed() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();

    set_phase_view(&mut app, 30_000);
    app.update();

    // After reset, elapsed_ms == 0. Advance Time by 1000ms and tick.
    advance_time(&mut app, Duration::from_millis(1000));
    app.update();

    let timer = app.world().resource::<PhaseTimerState>();
    assert_eq!(
        timer.elapsed_ms, 1000,
        "tick_phase_timer_system must add Time::delta to elapsed_ms"
    );
    assert_eq!(timer.duration_ms, 30_000, "duration_ms unchanged");
    assert!(timer.active, "active stays true while duration > 0");

    // Advance again — elapsed accumulates.
    advance_time(&mut app, Duration::from_millis(500));
    app.update();
    let timer = app.world().resource::<PhaseTimerState>();
    assert_eq!(timer.elapsed_ms, 1500, "Subsequent ticks must accumulate");
}

// ── Sub-test 3: Node.width tracks remaining ratio ────────────────────────────

#[test]
fn test_node_width_updates_per_remaining_pct() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let timer_bar = hud_entities(&app).timer_bar;

    set_phase_view(&mut app, 30_000);
    app.update();

    // At elapsed=0/30000, remaining=100% → full width.
    let width = read_node_width(&app, timer_bar);
    assert!(
        (width - HUD_PHASE_TIMER_BAR_MAX_WIDTH_PX).abs() < 0.01,
        "At reset, bar must render at full width (got {width}, expected {HUD_PHASE_TIMER_BAR_MAX_WIDTH_PX})"
    );
    assert_eq!(
        app.world().get::<Visibility>(timer_bar),
        Some(&Visibility::Visible),
        "Timer bar must be Visible while duration > 0"
    );

    // Advance to 50% elapsed → 50% remaining → half width.
    advance_time(&mut app, Duration::from_millis(15_000));
    app.update();
    let width = read_node_width(&app, timer_bar);
    let expected = HUD_PHASE_TIMER_BAR_MAX_WIDTH_PX * 0.5;
    assert!(
        (width - expected).abs() < 0.01,
        "At 50% elapsed, bar must render at half width (got {width}, expected {expected})"
    );

    // Advance past full duration → clamped to 0 width.
    advance_time(&mut app, Duration::from_millis(20_000));
    app.update();
    let width = read_node_width(&app, timer_bar);
    assert!(
        width.abs() < 0.01,
        "When elapsed_ms saturates at duration_ms, bar must render at 0 width (got {width})"
    );
}

// ── Sub-test 4: Bar hidden when duration == 0 ────────────────────────────────

#[test]
fn test_bar_hidden_when_duration_zero() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let timer_bar = hud_entities(&app).timer_bar;

    // Phases with no timer (e.g. Resolution, Lobby) carry timer_duration_ms == 0.
    set_phase_view(&mut app, 0);
    app.update();

    let timer = app.world().resource::<PhaseTimerState>();
    assert!(!timer.active, "active must be false when duration_ms == 0");
    assert_eq!(
        timer.duration_ms, 0,
        "duration_ms must be 0 (no active countdown)"
    );

    assert_eq!(
        app.world().get::<Visibility>(timer_bar),
        Some(&Visibility::Hidden),
        "Timer bar must be Hidden when duration_ms == 0"
    );
    let width = read_node_width(&app, timer_bar);
    assert!(
        width.abs() < 0.01,
        "Width must collapse to 0 when bar is hidden (got {width})"
    );

    // And a tick during an inactive timer must not mutate state.
    advance_time(&mut app, Duration::from_millis(1000));
    app.update();
    let timer = app.world().resource::<PhaseTimerState>();
    assert_eq!(
        timer.elapsed_ms, 0,
        "tick must not advance elapsed_ms while inactive"
    );
}

// ── Test fixture — same pattern as hud_resolution_dim_test.rs ────────────────

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    // Disable TimePlugin so `Time::advance_by` is authoritative in tests
    // (otherwise TimePlugin's `time_system` overwrites the manually advanced
    // delta with real-clock values each frame). Pattern matches
    // `tests/integration/rsm/rsm_network_dispatch_test.rs`.
    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());
    app.insert_resource(Time::<()>::default());
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HudPlugin);
    app.insert_resource(HudPlayerIds {
        local_id: PlayerId(1),
        opponent_id: PlayerId(2),
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn set_phase_view(app: &mut App, timer_duration_ms: u32) {
    let mut phase_view = app.world_mut().resource_mut::<ClientPhaseView>();
    phase_view.timer_duration_ms = timer_duration_ms;
}

fn advance_time(app: &mut App, delta: Duration) {
    app.world_mut().resource_mut::<Time>().advance_by(delta);
}

fn read_node_width(app: &App, entity: Entity) -> f32 {
    let node = app
        .world()
        .get::<Node>(entity)
        .expect("HudTimerBar must carry a Node component");
    match node.width {
        Val::Px(px) => px,
        other => panic!("HudTimerBar Node.width must be Val::Px, got {other:?}"),
    }
}
