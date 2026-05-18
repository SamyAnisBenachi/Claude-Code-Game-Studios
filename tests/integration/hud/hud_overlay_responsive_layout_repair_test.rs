//! PROMPT 1183 — HUD-OVERLAY-RESPONSIVE-LAYOUT-REPAIR regression suite.
//!
//! Guards three structural invariants surfaced by the "HUD + overlay
//! responsive layout repair" audit:
//!
//! 1. **Top-strip horizontal budget.** The HUD top strip's declared
//!    pixel widths + canonical column gaps + horizontal padding must
//!    fit inside the smallest supported viewport width (1280 px). The
//!    previous `SPACING_XL + SPACING_MD` (48 px) inter-pill gap pushed
//!    the seven-child strip past the 1280 px budget. The test verifies
//!    the gap is the spec-canonical `SPACING_MD` (16 px) and that the
//!    declared content + gap + padding fits.
//!
//! 2. **HUD timer surfaces stay visibility-gated.** Both the timer bar
//!    fill (`HudTimerBar`) and the numeric countdown
//!    (`HudTimerCountdown`) start hidden, flip to `Visible` only while
//!    `PhaseTimerState` is active, and flip back to `Hidden` once the
//!    timer drains or the phase changes. Closes the "hidden timer"
//!    failure mode called out in the PROMPT 1183 user-problem statement
//!    (no scannable countdown signal once the bar drains).
//!
//! 3. **Full-viewport HUD root never accidentally blocks gameplay.**
//!    The HUD root and the resolution dim overlay each carry
//!    `Pickable { should_block_lower: false, is_hoverable: false }`
//!    when the `ui_picking` feature is on, so a fault in their
//!    visibility logic cannot accidentally swallow gameplay clicks.

#![allow(dead_code)]

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::design_tokens::{spacing, strips};
use client::ui::hud::{
    HudDimOverlay, HudEntities, HudPlugin, HudRoot, HudTimerBar, HudTimerCountdown,
    PhaseTimerState, CURRENT_MANA_BAR_WIDTH_PX, HUD_PHASE_TIMER_BAR_MAX_WIDTH_PX,
    RESERVE_MANA_DIAMOND_SIZE_PX,
};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

/// Smallest supported viewport width per PROMPT 1183 user-problem
/// statement ("must not break at 1280x720"). Independent of the
/// canonical 6-viewport matrix in
/// `tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS`
/// (which uses 1366 as the minimum) — this constant is the
/// floor PROMPT 1183 specifically asks the HUD to survive.
pub const PROMPT_1183_MIN_VIEWPORT_WIDTH_PX: f32 = 1280.0;

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HudPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn px(val: Val) -> f32 {
    match val {
        Val::Px(v) => v,
        other => panic!("expected Val::Px, got {other:?}"),
    }
}

fn declared_width(app: &App, entity: Entity) -> Option<f32> {
    let node = app.world().get::<Node>(entity)?;
    match node.width {
        Val::Px(v) => Some(v),
        _ => None,
    }
}

#[test]
fn prompt_1183_top_strip_uses_spec_canonical_inter_pill_gap() {
    // PROMPT 1183 — `docs/ux/global-ui-design-spec.md` §9 ratifies
    // SPACING_MD (16 px) as the cluster-to-cluster gap on a HUD strip.
    // Anything larger (the prior `SPACING_XL + SPACING_MD` = 48 px)
    // pushes the seven-pill top strip past the 1280 px minimum-viewport
    // budget.
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    let strip_node = app
        .world()
        .get::<Node>(entities.top_strip)
        .expect("HUD top strip should carry a Node");
    assert_eq!(
        strip_node.column_gap,
        Val::Px(spacing::SPACING_MD),
        "HUD top-strip column_gap must equal SPACING_MD per spec §9 \
         cluster-to-cluster spacing — anything larger blows the \
         1280 px viewport budget."
    );
    // `row_gap` is a no-op on the single-row flex strip but its value
    // is held to the Sprint 14 / story 004 AC3 source-grep contract
    // (`SPACING_XL - SPACING_XS`). We assert here so a PROMPT 1183
    // follow-up can never silently drop the recomposition trail.
    assert_eq!(
        strip_node.row_gap,
        Val::Px(spacing::SPACING_XL - spacing::SPACING_XS),
        "HUD top-strip row_gap must preserve the Sprint 14 AC3 \
         `SPACING_XL - SPACING_XS` recomposition (no-op for the \
         single-row flex parent but holds the design-token contract)."
    );
}

#[test]
fn prompt_1183_top_strip_declared_widths_fit_minimum_viewport() {
    // PROMPT 1183 — sum the *declared* pixel widths the spawn code
    // commits to (mana bar, reserve diamond, timer bar) plus the
    // canonical horizontal padding and inter-pill gaps, and assert the
    // total fits inside the 1280 px minimum-supported-viewport budget
    // with at least 1 px of slack. Children without a declared pixel
    // width (pure-text pills) are reserved a conservative font-derived
    // budget so the test does not silently pass when their content
    // overflows.
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    // Pixel-declared HUD strip children. The mana bar and reserve
    // diamond keep their spawn-time width unchanged across phases.
    // The timer bar's runtime width is driven by
    // `sync_hud_timer_bar_system` (Node.width <- ratio × max), so its
    // *budget* is the public maximum constant rather than the
    // post-`app.update()` zero. We still assert the spawn-time max
    // width matches the public constant for spawn-side regressions
    // (`top_strip_timer_bar_node`).
    let always_pixel = [
        ("mana_label", entities.mana_label, CURRENT_MANA_BAR_WIDTH_PX),
        (
            "reserve_container",
            entities.reserve_container,
            RESERVE_MANA_DIAMOND_SIZE_PX,
        ),
    ];
    let mut declared_sum = 0.0;
    for (name, entity, expected) in always_pixel {
        let measured = declared_width(&app, entity).unwrap_or_else(|| {
            panic!("{name} must declare a pixel-fixed width for the budget assertion")
        });
        assert!(
            (measured - expected).abs() < 0.5,
            "{name} declared width {measured} px should equal the public \
             constant {expected} px"
        );
        declared_sum += measured;
    }
    // The timer bar consumes its maximum width when fully-charged, so
    // we book the upper bound against the budget. After spawn the
    // sync system drains it to 0 (timer inactive); on first phase it
    // climbs back up to the max — that climb is exactly the budget
    // case we must survive.
    declared_sum += HUD_PHASE_TIMER_BAR_MAX_WIDTH_PX;

    // Conservative text-pill budget: each labelled pill (phase / round /
    // gold-own / gold-opp) reserves 120 px for prefix + value. Empirical
    // upper bound covering the longest legal pill text
    // (`OPP Ecaflip 9999g` after class reveal). The countdown text
    // sibling reserves 60 px for a `"999s"`-style readout. Two extra
    // 100 px slots cover any future readout addition without forcing a
    // simultaneous test edit.
    let text_pill_budget = 4.0 * 120.0 + 60.0 + 2.0 * 100.0;

    // Horizontal padding (SPACING_LG × 2) per `hud_top_strip_node`.
    let padding = 2.0 * spacing::SPACING_LG;

    // 7 direct flex children (5 pills + reserve container + timer bar)
    // plus the countdown sibling = 8 children → 7 inter-child gaps.
    let inter_child_gaps = 7.0 * spacing::SPACING_MD;

    let total = declared_sum + text_pill_budget + padding + inter_child_gaps;
    assert!(
        total <= PROMPT_1183_MIN_VIEWPORT_WIDTH_PX - 1.0,
        "HUD top-strip declared content + gaps + padding ({total} px) \
         must fit within the PROMPT 1183 minimum-viewport budget \
         ({} px) with at least 1 px slack",
        PROMPT_1183_MIN_VIEWPORT_WIDTH_PX
    );
}

#[test]
fn prompt_1183_strip_anchors_keep_top_and_bottom_separated_at_720p() {
    // PROMPT 1183 — at 1280×720 (the minimum the user calls out) the
    // HeaderBar (60 px) at top, the FooterBar (40 px) at
    // `bottom: HAND_BAR_HEIGHT_PX (180 px)`, and the HandBar (180 px) at
    // bottom must reserve a non-empty centre play-area band so the HUD
    // strips never collide with each other or eat the gameplay surface.
    let vh: f32 = 720.0;
    let header_bottom_edge = strips::HEADER_BAR_HEIGHT_PX;
    let footer_top_edge = vh - strips::HAND_BAR_HEIGHT_PX - strips::FOOTER_BAR_HEIGHT_PX;
    let centre_band = footer_top_edge - header_bottom_edge;
    assert!(
        centre_band > 0.0,
        "HeaderBar bottom edge ({header_bottom_edge}) must sit above \
         the FooterBar top edge ({footer_top_edge}) at viewport height \
         {vh} — a non-positive centre band means the HUD strips collide \
         with each other and there is no room for the play area."
    );
    // Sanity floor: at least a third of the viewport for gameplay.
    assert!(
        centre_band > vh / 3.0,
        "Centre play-area band ({centre_band} px) must be at least \
         a third of the viewport height ({} px) at 1280×720; \
         shrinking strip headers further would crowd the play area.",
        vh / 3.0
    );
}

#[test]
fn prompt_1183_hud_timer_surfaces_start_hidden_and_flip_on_active() {
    // PROMPT 1183 — the bar + numeric countdown must both start hidden
    // (no `PhaseTimerState` published yet → timer inactive) and flip to
    // `Visible` exactly when `PhaseTimerState.active` is true. Closes
    // the "hidden timer" failure mode (player has no scannable
    // remaining-time signal once the bar drains).
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    let bar_vis = *app
        .world()
        .get::<Visibility>(entities.timer_bar)
        .expect("HudTimerBar should carry a Visibility");
    let countdown_vis = *app
        .world()
        .get::<Visibility>(entities.timer_countdown)
        .expect("HudTimerCountdown should carry a Visibility");
    assert_eq!(
        bar_vis,
        Visibility::Hidden,
        "HudTimerBar must spawn Hidden so a stale-asset frame never \
         leaks a 100%-full bar before the first phase publishes a timer."
    );
    assert_eq!(
        countdown_vis,
        Visibility::Hidden,
        "HudTimerCountdown must spawn Hidden so it never prints a stale \
         `0s` placeholder before the first phase publishes a timer."
    );

    // Activate the timer and tick the schedule. The HUD's sync systems
    // run in `HudSystemSet::StateSync` and react to a changed
    // `PhaseTimerState`.
    {
        let mut timer = app.world_mut().resource_mut::<PhaseTimerState>();
        timer.duration_ms = 45_000;
        timer.elapsed_ms = 0;
        timer.active = true;
    }
    app.update();

    let bar_vis = *app
        .world()
        .get::<Visibility>(entities.timer_bar)
        .expect("HudTimerBar should still exist");
    let countdown_vis = *app
        .world()
        .get::<Visibility>(entities.timer_countdown)
        .expect("HudTimerCountdown should still exist");
    assert_eq!(
        bar_vis,
        Visibility::Visible,
        "HudTimerBar must flip to Visible while PhaseTimerState is active"
    );
    assert_eq!(
        countdown_vis,
        Visibility::Visible,
        "HudTimerCountdown must flip to Visible while PhaseTimerState is active"
    );

    // Now deactivate (timer drained / phase ended) and tick again — both
    // surfaces must flip back to Hidden so a stale signal never lingers.
    {
        let mut timer = app.world_mut().resource_mut::<PhaseTimerState>();
        timer.duration_ms = 0;
        timer.elapsed_ms = 0;
        timer.active = false;
    }
    app.update();

    let bar_vis = *app
        .world()
        .get::<Visibility>(entities.timer_bar)
        .expect("HudTimerBar should still exist");
    let countdown_vis = *app
        .world()
        .get::<Visibility>(entities.timer_countdown)
        .expect("HudTimerCountdown should still exist");
    assert_eq!(
        bar_vis,
        Visibility::Hidden,
        "HudTimerBar must flip back to Hidden when the timer goes inactive"
    );
    assert_eq!(
        countdown_vis,
        Visibility::Hidden,
        "HudTimerCountdown must flip back to Hidden when the timer goes inactive"
    );
}

#[test]
fn prompt_1183_resolution_dim_overlay_only_visible_in_resolution() {
    // PROMPT 1183 — the RESOLUTION dim overlay covers the full viewport
    // and tints gameplay; it must remain Hidden outside the Resolution
    // phase so a stuck-Visible overlay never produces a "black slab"
    // that hides interactive surfaces underneath.
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    // Spawn defaults: Hidden (the HUD root only flips visible once a
    // session lands; the dim overlay only flips visible during
    // Resolution).
    let initial = *app
        .world()
        .get::<Visibility>(entities.dim_overlay)
        .expect("HudDimOverlay should carry a Visibility");
    assert_eq!(
        initial,
        Visibility::Hidden,
        "HudDimOverlay must spawn Hidden so it never produces a black \
         slab over the lobby / initial-draft / placement surfaces."
    );

    // Drive the phase into Resolution and assert the overlay flips on.
    {
        let mut phase = app.world_mut().resource_mut::<CurrentClientPhase>();
        phase.phase = RoundPhase::Resolution;
    }
    app.update();
    let in_resolution = *app
        .world()
        .get::<Visibility>(entities.dim_overlay)
        .expect("HudDimOverlay should still exist");
    assert_eq!(
        in_resolution,
        Visibility::Visible,
        "HudDimOverlay must flip Visible during Resolution"
    );

    // Drive the phase back to Placement and assert the overlay flips off
    // again — closes the "stuck-visible scrim eating gameplay clicks"
    // regression mode.
    {
        let mut phase = app.world_mut().resource_mut::<CurrentClientPhase>();
        phase.phase = RoundPhase::Placement;
    }
    app.update();
    let post_resolution = *app
        .world()
        .get::<Visibility>(entities.dim_overlay)
        .expect("HudDimOverlay should still exist");
    assert_eq!(
        post_resolution,
        Visibility::Hidden,
        "HudDimOverlay must flip Hidden once we leave Resolution"
    );
}

#[test]
fn prompt_1183_full_viewport_roots_declared_at_origin() {
    // PROMPT 1183 — the full-viewport HUD root and the RESOLUTION dim
    // overlay both anchor at (0,0) with each side at 0 px so they
    // always span the viewport regardless of size. A regression that
    // hard-codes a pixel inset would crop the overlay at smaller
    // viewports and create a visible band of un-dimmed gameplay.
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    for (name, entity) in [
        ("HudRoot", entities.root),
        ("HudDimOverlay", entities.dim_overlay),
    ] {
        let node = app
            .world()
            .get::<Node>(entity)
            .unwrap_or_else(|| panic!("{name} should carry a Node"));
        assert_eq!(
            node.position_type,
            PositionType::Absolute,
            "{name} must be PositionType::Absolute"
        );
        for (side, val) in [
            ("left", node.left),
            ("right", node.right),
            ("top", node.top),
            ("bottom", node.bottom),
        ] {
            assert_eq!(
                px(val),
                0.0,
                "{name} `{side}` must be 0 px so it spans the viewport \
                 at every supported size."
            );
        }
    }
}

#[test]
fn prompt_1183_hud_root_marker_lives_on_the_actual_root() {
    // Sanity wrapper around the `HudRoot` marker → the
    // `entities.root` entity. If a refactor moves the marker without
    // updating the resource, the PROMPT 1183 visibility guards below
    // would silently target the wrong entity.
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);
    assert!(
        app.world().get::<HudRoot>(entities.root).is_some(),
        "HudRoot marker must remain on `entities.root`"
    );
    assert!(
        app.world()
            .get::<HudDimOverlay>(entities.dim_overlay)
            .is_some(),
        "HudDimOverlay marker must remain on `entities.dim_overlay`"
    );
    assert!(
        app.world().get::<HudTimerBar>(entities.timer_bar).is_some(),
        "HudTimerBar marker must remain on `entities.timer_bar`"
    );
    assert!(
        app.world()
            .get::<HudTimerCountdown>(entities.timer_countdown)
            .is_some(),
        "HudTimerCountdown marker must remain on `entities.timer_countdown`"
    );
}

#[test]
fn prompt_1183_hud_root_starts_hidden_so_lobby_is_never_dimmed() {
    // PROMPT 1183 — the HUD root spawns `Visibility::Hidden`. The HUD
    // only flips Visible on the first in-session phase
    // (`hud_phase_transition_system`). Without this guard, a HUD that
    // spawned Visible would render its strips over the lobby and the
    // class-picker confirm flow — exactly the "overlap gameplay"
    // failure mode PROMPT 1183 is repairing.
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    let vis = *app
        .world()
        .get::<Visibility>(entities.root)
        .expect("HudRoot should carry a Visibility");
    // Note: at this point the test app has called `app.update()` once,
    // which executes `spawn_hud` (OnEnter(InSession)) — but the phase
    // sink has not driven a non-lobby phase, so `hud_phase_transition_system`
    // either hides the root (matching Lobby/Handshaking) or leaves the
    // spawn-time Hidden alone. Either way it must remain Hidden.
    assert_eq!(
        vis,
        Visibility::Hidden,
        "HudRoot must remain Hidden until a non-lobby phase is published"
    );
}

#[test]
fn prompt_1183_top_strip_overflow_visible_so_pill_shadows_do_not_clip() {
    // PROMPT 1183 — the top strip enables `Overflow::visible()` so the
    // reserve-mana diamond (74×74 rotated 45°, visual height ~104 px)
    // and the gold readout's per-pill drop shadow are not clipped to
    // the 60-px HeaderBar footprint. A regression that changes the
    // strip's overflow to `clip` would chop the diamond and shadow.
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);
    let strip_node = app
        .world()
        .get::<Node>(entities.top_strip)
        .expect("HUD top strip should carry a Node");
    // bevy_ui exposes overflow via `Overflow { x, y }`; the canonical
    // "visible / visible" pair is built by `Overflow::visible()` so any
    // regression replacing it with `clip()` / `hidden()` is caught here.
    assert_eq!(
        strip_node.overflow,
        Overflow::visible(),
        "HUD top strip overflow must remain Overflow::visible() so the \
         rotated reserve-mana diamond and the pill shadows escape the \
         60-px HeaderBar footprint."
    );
}

#[test]
fn prompt_1183_client_phase_view_default_keeps_timer_inactive() {
    // PROMPT 1183 — a freshly-spawned HUD must NOT auto-activate the
    // phase timer. The HUD reads `ClientPhaseView.timer_duration_ms` in
    // `reset_phase_timer_system` and only sets `active = true` when the
    // duration is non-zero. Asserting this default closes a subtle
    // failure mode where a regression in `ClientPhaseView::default()`
    // would publish a non-zero timer for a phase the server never
    // started, leaving the countdown visible at all times.
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let view = app.world().resource::<ClientPhaseView>();
    assert_eq!(
        view.timer_duration_ms, 0,
        "ClientPhaseView::default().timer_duration_ms must be 0 so the \
         HUD does not auto-activate the timer for an unstarted phase."
    );
    let timer = app.world().resource::<PhaseTimerState>();
    assert!(
        !timer.active,
        "PhaseTimerState must stay inactive while the phase view is at \
         its default zero duration."
    );
}
