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
    ui::hud::{
        HudEntities, HudPlayerIds, HudPlugin, HudTimerCountdown, PhaseTimerState,
        HUD_STRIP_BACKGROUND_COLOR, HUD_STRIP_BORDER_COLOR, HUD_TIMER_COUNTDOWN_FONT_SIZE_PX,
        HUD_TIMER_COUNTDOWN_MIN_WIDTH_PX, HUD_TIMER_TEXT_COLOR,
    },
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
        "TIME 60s",
        "countdown must render a labelled rounded remaining-seconds chip at reset",
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
    assert_eq!(text(&app, entities.timer_countdown), "TIME 30s");

    advance_time(&mut app, Duration::from_millis(5_000));
    app.update();
    assert_eq!(
        text(&app, entities.timer_countdown),
        "TIME 25s",
        "countdown must decrement as Time::delta accumulates",
    );

    advance_time(&mut app, Duration::from_millis(24_999));
    app.update();
    // Round-up keeps the readout from showing `0s` while time remains.
    assert_eq!(
        text(&app, entities.timer_countdown),
        "TIME 1s",
        "countdown must round up so 1ms remaining still reads as `1s`",
    );

    advance_time(&mut app, Duration::from_millis(1));
    app.update();
    assert_eq!(
        text(&app, entities.timer_countdown),
        "TIME 0s",
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
    assert_eq!(text(&app, entities.timer_countdown), "TIME 25s");

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
        "TIME 25s",
        "countdown must stay elapsed-aware when ClientPhaseView is re-marked changed"
    );
}

#[test]
fn countdown_state_display_text_remains_snapshot_compatible() {
    let timer = PhaseTimerState {
        active: true,
        duration_ms: 30_000,
        elapsed_ms: 5_000,
        ..default()
    };

    assert_eq!(
        timer.display_text(),
        "25s",
        "PhaseTimerState display_text remains the unprefixed snapshot field"
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

#[test]
fn countdown_chip_uses_readable_fixed_layout() {
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    let node = app
        .world()
        .get::<Node>(entities.timer_countdown)
        .expect("countdown should carry a Node");
    assert_eq!(node.min_width, Val::Px(HUD_TIMER_COUNTDOWN_MIN_WIDTH_PX));
    assert_eq!(node.flex_shrink, 0.0);
    assert_ne!(node.border.left, Val::Px(0.0));

    let font = app
        .world()
        .get::<TextFont>(entities.timer_countdown)
        .expect("countdown should carry TextFont");
    assert_eq!(font.font_size, HUD_TIMER_COUNTDOWN_FONT_SIZE_PX);

    assert!(
        app.world()
            .get::<BorderColor>(entities.timer_countdown)
            .is_some(),
        "countdown should carry a high-contrast border"
    );
    assert_eq!(
        app.world()
            .get::<TextColor>(entities.timer_countdown)
            .map(|color| color.0),
        Some(HUD_TIMER_TEXT_COLOR),
        "countdown should be the brightest timer-priority readout in the top strip"
    );
}

#[test]
fn hud_edge_strips_carry_layered_chrome() {
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    for (name, entity) in [
        ("top strip", entities.top_strip),
        ("bottom strip", entities.bottom_strip),
    ] {
        assert_eq!(
            app.world()
                .get::<BackgroundColor>(entity)
                .map(|color| color.0),
            Some(HUD_STRIP_BACKGROUND_COLOR),
            "{name} should carry the shared edge-strip background"
        );
        assert_eq!(
            app.world()
                .get::<BorderColor>(entity)
                .map(|color| color.top),
            Some(HUD_STRIP_BORDER_COLOR),
            "{name} should carry the shared edge-strip border"
        );
    }
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
