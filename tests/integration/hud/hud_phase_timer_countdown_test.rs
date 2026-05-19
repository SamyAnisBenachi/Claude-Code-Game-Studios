//! S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139, UI-1129-06)
//! — numeric countdown text label coverage.
//!
//! The HUD top strip now carries a `HudTimerCountdown` text entity that
//! reflects [`PhaseTimerState`] as a remaining-seconds readout. The
//! existing `HudTimerBar` retains the proportional fill; this test
//! locks in the new text contract so future workers cannot regress the
//! "no visible timer in Placement/DraftShop/DraftAuction" defect that
//! AUDIT-1129 / AUDIT-1131 raised.

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimePlugin;
use client::{
    state::{ClientPhaseView, ClientState},
    ui::hud::{HudEntities, HudPlayerIds, HudPlugin, HudTimerCountdown, PhaseTimerState},
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn countdown_shows_remaining_seconds_for_placement_phase() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase_view(&mut app, 60_000);
    app.update();

    assert_eq!(
        text(&app, entities.timer_countdown),
        "60s",
        "countdown must render the rounded remaining seconds at reset",
    );
    assert_eq!(
        app.world().get::<Visibility>(entities.timer_countdown),
        Some(&Visibility::Visible),
        "countdown must be Visible while duration > 0",
    );
}

#[test]
fn countdown_ticks_down_each_frame() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase_view(&mut app, 30_000);
    app.update();
    assert_eq!(text(&app, entities.timer_countdown), "30s");

    advance_time(&mut app, Duration::from_millis(5_000));
    app.update();
    assert_eq!(
        text(&app, entities.timer_countdown),
        "25s",
        "countdown must decrement as Time::delta accumulates",
    );

    advance_time(&mut app, Duration::from_millis(24_999));
    app.update();
    // Round-up keeps the readout from showing `0s` while time remains.
    assert_eq!(
        text(&app, entities.timer_countdown),
        "1s",
        "countdown must round up so 1ms remaining still reads as `1s`",
    );

    advance_time(&mut app, Duration::from_millis(1));
    app.update();
    assert_eq!(
        text(&app, entities.timer_countdown),
        "0s",
        "countdown must read `0s` once the budget is fully elapsed",
    );
}

#[test]
fn countdown_does_not_reset_when_same_phase_view_is_marked_changed() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase_view(&mut app, 30_000);
    app.update();

    advance_time(&mut app, Duration::from_millis(5_000));
    app.update();
    assert_eq!(text(&app, entities.timer_countdown), "25s");

    advance_time(&mut app, Duration::ZERO);
    set_phase_view(&mut app, 30_000);
    app.update();

    let timer = app.world().resource::<PhaseTimerState>();
    assert_eq!(
        timer.elapsed_ms, 5_000,
        "same phase/round/duration writes must not reset elapsed_ms"
    );
    assert_eq!(
        text(&app, entities.timer_countdown),
        "25s",
        "countdown must stay elapsed-aware when ClientPhaseView is re-marked changed"
    );
}

#[test]
fn countdown_is_hidden_for_resolution_or_lobby_with_zero_duration() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    set_phase_view(&mut app, 0);
    app.update();

    assert_eq!(
        text(&app, entities.timer_countdown),
        "",
        "countdown text must be empty when no timer is active",
    );
    assert_eq!(
        app.world().get::<Visibility>(entities.timer_countdown),
        Some(&Visibility::Hidden),
        "countdown must be Hidden when duration_ms == 0",
    );
}

#[test]
fn countdown_entity_is_uniquely_marked() {
    let mut app = app_with_hud_in_session();
    let mut q = app
        .world_mut()
        .query_filtered::<Entity, With<HudTimerCountdown>>();
    assert_eq!(
        q.iter(app.world()).count(),
        1,
        "exactly one HudTimerCountdown entity must be pre-pooled",
    );
}

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
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
    let _ = app.world().resource::<PhaseTimerState>();
}

fn advance_time(app: &mut App, delta: Duration) {
    app.world_mut().resource_mut::<Time>().advance_by(delta);
}

fn text(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .expect("HudTimerCountdown must have Text")
        .0
        .clone()
}
