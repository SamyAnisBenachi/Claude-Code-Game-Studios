//! PROMPT 1404 / `S19-UI-PHASE-CHANGE-BANNER-001` regression bin.
//!
//! Closes V-P1-04 / RC-8 from PROMPT 1396: phase transitions previously
//! only repainted the HUD phase pill, leaving the player without a
//! Krosmaga-style centered banner cue. The plugin under test
//! ([`client::ui::PhaseBannerPlugin`]) spawns a transient centered
//! overlay on every major `RoundPhase` transition and auto-despawns
//! after [`PHASE_BANNER_LIFETIME`].
//!
//! Coverage:
//! - AC1: banner spawns on a `RoundPhase` change with the expected
//!   label text and marker.
//! - AC2: banner auto-despawns once its lifetime elapses.
//! - AC3: banner panel declares a bounded `max_width` so the label
//!   never reaches the viewport edges.
//! - AC4: phases that intentionally have no banner (Lobby /
//!   Handshaking / GameOver) clear any in-flight banner without
//!   spawning a new one.
//! - AC5: re-entering a banner-bearing phase from a different phase
//!   raises a fresh banner with the new label.

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimePlugin;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::{
    phase_banner_label_for, PhaseBannerLabel, PhaseBannerPanel, PhaseBannerPlugin, PhaseBannerRoot,
    PHASE_BANNER_BACKGROUND_COLOR, PHASE_BANNER_BORDER_COLOR, PHASE_BANNER_LIFETIME,
    PHASE_BANNER_MAX_WIDTH_PERCENT, PHASE_BANNER_MAX_WIDTH_PX, PHASE_BANNER_MIN_HEIGHT_PX,
    PHASE_BANNER_TEXT_COLOR,
};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn ac1_banner_spawns_on_phase_change_with_expected_label() {
    test_helpers::init_test_tracing();
    let mut app = app_with_phase_banner_in_session();

    // Pre-condition: starting phase is `Lobby` (no banner label) so the
    // tree should be empty after the first `update`.
    app.update();
    assert_eq!(
        count_banners(&mut app),
        0,
        "AC1: no banner must spawn while phase is Lobby (no banner label)"
    );

    // Transition Lobby -> Placement; one banner with label `PLACEMENT`
    // must exist.
    set_phase(&mut app, RoundPhase::Placement);
    app.update();

    assert_eq!(
        count_banners(&mut app),
        1,
        "AC1: phase transition into Placement must spawn exactly one banner"
    );
    assert_eq!(
        single_banner_phase(&mut app),
        RoundPhase::Placement,
        "AC1: banner must carry the phase it announces"
    );
    assert_eq!(
        banner_label_text(&mut app),
        "PLACEMENT",
        "AC1: banner label must match phase_banner_label_for(Placement)"
    );
    assert_eq!(
        phase_banner_label_for(RoundPhase::Placement),
        Some("PLACEMENT"),
        "AC1: the public label resolver must agree with the spawn site"
    );
}

#[test]
fn ac2_banner_auto_despawns_after_its_lifetime() {
    test_helpers::init_test_tracing();
    let mut app = app_with_phase_banner_in_session();

    app.update();
    set_phase(&mut app, RoundPhase::DraftAuction);
    app.update();
    assert_eq!(
        count_banners(&mut app),
        1,
        "AC2: precondition — banner spawns on DraftAuction transition"
    );

    // Advance time past the banner's lifetime; the tick system must
    // despawn it within a single `update`.
    advance_time(&mut app, PHASE_BANNER_LIFETIME + Duration::from_millis(50));
    app.update();

    assert_eq!(
        count_banners(&mut app),
        0,
        "AC2: banner must auto-despawn after PHASE_BANNER_LIFETIME elapses"
    );
}

#[test]
fn ac3_banner_panel_declares_bounded_max_width() {
    test_helpers::init_test_tracing();
    let mut app = app_with_phase_banner_in_session();

    app.update();
    set_phase(&mut app, RoundPhase::Resolution);
    app.update();

    let panel = single_panel_node(&mut app);
    match panel.max_width {
        Val::Percent(pct) => assert!(
            (pct - PHASE_BANNER_MAX_WIDTH_PERCENT).abs() < f32::EPSILON,
            "AC3: banner panel max_width must equal PHASE_BANNER_MAX_WIDTH_PERCENT \
             (got {pct}, expected {PHASE_BANNER_MAX_WIDTH_PERCENT})"
        ),
        other => panic!(
            "AC3: banner panel max_width must be Val::Percent so the panel \
             scales bounded against the viewport (got {other:?})"
        ),
    }

    match panel.width {
        Val::Px(px) => assert!(
            (px - PHASE_BANNER_MAX_WIDTH_PX).abs() < f32::EPSILON,
            "AC3: banner panel width must equal PHASE_BANNER_MAX_WIDTH_PX \
             (got {px}, expected {PHASE_BANNER_MAX_WIDTH_PX})"
        ),
        other => panic!(
            "AC3: banner panel width must be Val::Px so the rendered width is \
             deterministic (got {other:?})"
        ),
    }

    assert!(
        PHASE_BANNER_MAX_WIDTH_PERCENT < 100.0,
        "AC3: panel max_width % must leave horizontal breathing room at every viewport"
    );
    assert_eq!(
        panel.min_height,
        Val::Px(PHASE_BANNER_MIN_HEIGHT_PX),
        "AC3: banner panel should reserve enough vertical chrome for the phase callout"
    );
}

#[test]
fn ac3_banner_panel_uses_readable_layered_chrome() {
    test_helpers::init_test_tracing();
    let mut app = app_with_phase_banner_in_session();

    app.update();
    set_phase(&mut app, RoundPhase::DraftShop);
    app.update();

    let panel = single_panel(&mut app);
    assert_eq!(
        app.world()
            .get::<BackgroundColor>(panel)
            .map(|color| color.0),
        Some(PHASE_BANNER_BACKGROUND_COLOR),
        "phase banner panel should use the shared readable background"
    );
    assert_eq!(
        app.world().get::<BorderColor>(panel).map(|color| color.top),
        Some(PHASE_BANNER_BORDER_COLOR),
        "phase banner panel should use the shared bright border"
    );
    let label = single_banner_label(&mut app);
    assert_eq!(
        app.world().get::<TextColor>(label).map(|color| color.0),
        Some(PHASE_BANNER_TEXT_COLOR),
        "phase banner label should use the shared readable text color"
    );
}

#[test]
fn ac4_phases_without_a_label_do_not_spawn_a_banner() {
    test_helpers::init_test_tracing();
    let mut app = app_with_phase_banner_in_session();

    app.update();
    // Raise a banner first so we can verify the no-label transition
    // also clears any in-flight banner.
    set_phase(&mut app, RoundPhase::DraftShop);
    app.update();
    assert_eq!(
        count_banners(&mut app),
        1,
        "AC4: precondition — DraftShop transition raises a banner"
    );

    // Transition into a phase intentionally without a banner — GameOver
    // owns its own result-screen modal and would clash with a transient
    // centered banner.
    set_phase(&mut app, RoundPhase::GameOver);
    app.update();

    assert_eq!(
        count_banners(&mut app),
        0,
        "AC4: transitioning into a no-banner phase must clear any in-flight banner"
    );
    assert!(
        phase_banner_label_for(RoundPhase::GameOver).is_none(),
        "AC4: GameOver must have no banner label"
    );
    assert!(
        phase_banner_label_for(RoundPhase::Lobby).is_none(),
        "AC4: Lobby must have no banner label"
    );
    assert!(
        phase_banner_label_for(RoundPhase::Handshaking).is_none(),
        "AC4: Handshaking must have no banner label"
    );
}

#[test]
fn ac5_consecutive_phase_changes_swap_the_banner_label() {
    test_helpers::init_test_tracing();
    let mut app = app_with_phase_banner_in_session();

    app.update();
    set_phase(&mut app, RoundPhase::DraftInitial);
    app.update();
    assert_eq!(
        banner_label_text(&mut app),
        "DRAFT",
        "AC5: first transition must spawn the DRAFT banner"
    );

    // A second phase transition (before the timer expires) must despawn
    // the stale banner and spawn a fresh one with the new label.
    set_phase(&mut app, RoundPhase::Placement);
    app.update();

    assert_eq!(
        count_banners(&mut app),
        1,
        "AC5: a consecutive phase change must keep exactly one banner alive"
    );
    assert_eq!(
        banner_label_text(&mut app),
        "PLACEMENT",
        "AC5: a consecutive phase change must replace the label with the new phase's"
    );
    assert_eq!(
        single_banner_phase(&mut app),
        RoundPhase::Placement,
        "AC5: the new banner must carry the new phase"
    );
}

// ─── Fixture ────────────────────────────────────────────────────────────────

fn app_with_phase_banner_in_session() -> App {
    let mut app = App::new();
    // Disable TimePlugin so `Time::advance_by` is authoritative in tests
    // (same pattern as `tests/integration/hud/hud_phase_timer_bar_test.rs`).
    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());
    app.insert_resource(Time::<()>::default());
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(PhaseBannerPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
    current.phase = phase;
}

fn advance_time(app: &mut App, delta: Duration) {
    app.world_mut().resource_mut::<Time>().advance_by(delta);
}

fn banner_entities(app: &mut App) -> Vec<Entity> {
    app.world_mut()
        .query_filtered::<Entity, With<PhaseBannerRoot>>()
        .iter(app.world())
        .collect()
}

fn count_banners(app: &mut App) -> usize {
    banner_entities(app).len()
}

fn single_banner_phase(app: &mut App) -> RoundPhase {
    let entities = banner_entities(app);
    assert_eq!(
        entities.len(),
        1,
        "expected exactly one banner; found {}",
        entities.len()
    );
    app.world()
        .get::<PhaseBannerRoot>(entities[0])
        .expect("banner entity must carry PhaseBannerRoot")
        .phase
}

fn single_panel_node(app: &mut App) -> Node {
    let panel = single_panel(app);
    app.world()
        .get::<Node>(panel)
        .expect("banner panel must carry a Node component")
        .clone()
}

fn single_panel(app: &mut App) -> Entity {
    let entities: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<PhaseBannerPanel>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        entities.len(),
        1,
        "expected exactly one banner panel; found {}",
        entities.len()
    );
    entities[0]
}

fn single_banner_label(app: &mut App) -> Entity {
    let entities: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<PhaseBannerLabel>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        entities.len(),
        1,
        "expected exactly one banner label; found {}",
        entities.len()
    );
    entities[0]
}

fn banner_label_text(app: &mut App) -> String {
    let label = single_banner_label(app);
    app.world()
        .get::<Text>(label)
        .expect("banner label must carry a Text component")
        .0
        .clone()
}
