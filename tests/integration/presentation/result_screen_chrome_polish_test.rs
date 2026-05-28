//! PROMPT 1538 -- Result screen Krosmaga chrome polish.
//! PROMPT 1896 -- Result screen 720px overflow scroll guard fix.
//!
//! Focused tests for the chrome-only polish added on top of PROMPT 1481:
//!
//! - Step indicator pill mounts once and tracks the current step
//!   (`Step 1 of 2` on Hero, `Step 2 of 2` on Accounting).
//! - Title divider mounts under the headline and tints from the outcome
//!   accent palette (alpha-modulated) so it reads as part of the framed
//!   outcome hero.
//! - Section divider mounts inside the accounting panel exactly once.
//! - Actions row carries an overflow-safe `flex_shrink: 0.0` + non-zero
//!   `min_height` so the dismiss CTA stays reachable on 1280x720.
//! - Outer panel uses `Overflow::clip()` as a safety net against the
//!   objective grid spilling outside the framed modal on small viewports.
//! - Inner scroll pane wrapping the step indicator and hero/accounting panels
//!   enables `overflow_y: Scroll` so content taller than the 720px viewport
//!   can be scrolled without the Return-to-Lobby CTA scrolling out of reach.
//!
//! These are scoped to the visual chrome only; they assert markers and node
//! properties and never touch the data contract or focus order.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::ui::OverflowAxis;
use client::presentation::result_screen::{
    result_screen_outcome_accent, ResultScreenEntities, ResultScreenPlugin,
    ResultScreenSectionDivider, ResultScreenScrollPane, ResultScreenStep,
    ResultScreenStepActionRequest, ResultScreenStepIndicator, ResultScreenStepState,
    ResultScreenTitleDivider, ResultScreenViewState,
};
use client::presentation::PresentationGameSnapshotMessage;
use client::state::{ClientSessionIdentity, ClientState, CurrentClientPhase};
use shared::card::ClassId;
use shared::protocol::{
    BoardSnapshot, GameOverReason, PlacementTimerMultiplier, PlayerSnapshot, RoundPhase,
    S2CGameOver, S2CGameSnapshot,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn step_indicator_mounts_once_and_tracks_current_step() {
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );

    assert_eq!(
        query_count::<ResultScreenStepIndicator>(&mut app),
        1,
        "exactly one step indicator pill must be mounted on the result panel"
    );

    let entities = *app.world().resource::<ResultScreenEntities>();
    assert_eq!(text_of(&mut app, entities.step_indicator), "Step 1 of 2");

    advance_to_accounting(&mut app);
    app.update();
    assert_eq!(text_of(&mut app, entities.step_indicator), "Step 2 of 2");
}

#[test]
fn title_divider_mounts_once_and_tints_from_outcome_accent() {
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );

    assert_eq!(
        query_count::<ResultScreenTitleDivider>(&mut app),
        1,
        "exactly one title divider must be mounted under the hero headline"
    );

    let entities = *app.world().resource::<ResultScreenEntities>();
    let bg = background_of(&mut app, entities.title_divider);
    let victory_accent = result_screen_outcome_accent("VICTORY");
    assert_eq!(
        bg.to_srgba().red,
        victory_accent.with_alpha(0.55).to_srgba().red,
        "title divider must tint from the VICTORY outcome accent"
    );
    assert!(
        (bg.to_srgba().alpha - 0.55).abs() < f32::EPSILON,
        "title divider must use the outcome accent at 0.55 alpha so it frames \
         the headline without competing with it"
    );
}

#[test]
fn section_divider_mounts_once_on_accounting_panel() {
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );

    assert_eq!(
        query_count::<ResultScreenSectionDivider>(&mut app),
        1,
        "exactly one section divider must frame the accounting panel header"
    );
}

#[test]
fn actions_row_pins_a_minimum_height_so_cta_stays_reachable() {
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );

    let entities = *app.world().resource::<ResultScreenEntities>();
    // The action row is the direct parent of the Continue button; walk one
    // hop via the ChildOf relation. Bevy 0.18 stores parent links on
    // `ChildOf`.
    let action_row = app
        .world()
        .get::<ChildOf>(entities.continue_button)
        .map(|child_of| child_of.parent())
        .expect("Continue CTA must have an action-row parent");

    let node = app
        .world()
        .get::<Node>(action_row)
        .expect("action row must carry a Node component");
    assert!(
        matches!(node.flex_shrink, value if (value - 0.0).abs() < f32::EPSILON),
        "action row must not shrink under flex pressure: flex_shrink={:?}",
        node.flex_shrink
    );
    match node.min_height {
        Val::Px(px) => assert!(
            px >= 50.0,
            "action row min_height must reserve at least 50 px so the dismiss \
             CTA stays reachable on 1280x720; got {px} px"
        ),
        other => panic!("action row min_height must be a pixel value, got {other:?}"),
    }
}

#[test]
fn panel_clips_overflow_as_safety_net() {
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );

    let entities = *app.world().resource::<ResultScreenEntities>();
    let node = app
        .world()
        .get::<Node>(entities.panel)
        .expect("result panel must carry a Node component");
    assert_eq!(
        node.overflow.x,
        OverflowAxis::Clip,
        "panel must clip horizontal overflow as a Krosmaga-style framing safety net"
    );
    assert_eq!(
        node.overflow.y,
        OverflowAxis::Clip,
        "panel must clip vertical overflow as a Krosmaga-style framing safety net"
    );
}

#[test]
fn scroll_pane_enables_overflow_scroll_so_content_reachable_on_720p() {
    // PROMPT 1896: 1280x720 overflow guard.
    //
    // The scroll pane wraps the step indicator and hero/accounting panels
    // inside the result panel. On viewports as short as 720 px the accounting
    // content (resources, ledger, 10 objective rows) can exceed the available
    // height. The scroll pane must enable overflow-y scrolling so users can
    // reach all content; and it must grow to fill available space while the
    // actions row stays pinned below it.
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(
        &mut app,
        Some(result(Some(player(2)), GameOverReason::ObjectivesDestroyed)),
    );

    assert_eq!(
        query_count::<ResultScreenScrollPane>(&mut app),
        1,
        "exactly one scroll pane must wrap the step indicator and hero/accounting panels"
    );

    let scroll_pane_entity = app
        .world_mut()
        .query::<(Entity, &ResultScreenScrollPane)>()
        .iter(app.world())
        .next()
        .map(|(entity, _)| entity)
        .expect("ResultScreenScrollPane entity must exist");

    let node = app
        .world()
        .get::<Node>(scroll_pane_entity)
        .expect("scroll pane must carry a Node component");

    assert_eq!(
        node.overflow.y,
        OverflowAxis::Scroll,
        "scroll pane must enable overflow_y scroll so accounting content is \
         reachable on 1280x720 without hiding the Return-to-Lobby CTA"
    );
    assert!(
        (node.flex_grow - 1.0).abs() < f32::EPSILON,
        "scroll pane must flex_grow to fill available space above the pinned \
         actions row; got flex_grow={:?}",
        node.flex_grow
    );
    assert_eq!(
        node.min_height,
        Val::Px(0.0),
        "scroll pane min_height must be 0 px so it can shrink below its \
         intrinsic content height on short viewports, allowing the layout \
         engine to honour the actions row flex_shrink: 0 constraint"
    );
}

// --- test helpers -----------------------------------------------------------

fn query_count<C: Component>(app: &mut App) -> usize {
    app.world_mut().query::<&C>().iter(app.world()).count()
}

fn text_of(app: &mut App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .map(|text| text.0.clone())
        .unwrap_or_default()
}

fn background_of(app: &mut App, entity: Entity) -> Color {
    app.world()
        .get::<BackgroundColor>(entity)
        .map(|bg| bg.0)
        .expect("entity must have a BackgroundColor")
}

fn result_screen_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ResultScreenPlugin);
    app.init_resource::<ButtonInput<KeyCode>>();
    app.world_mut()
        .resource_mut::<ClientSessionIdentity>()
        .player_id = Some(player(1));
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn open_result_screen(app: &mut App, result: Option<S2CGameOver>) {
    app.world_mut()
        .write_message(PresentationGameSnapshotMessage(game_over_snapshot()));
    {
        let mut view_state = app.world_mut().resource_mut::<ResultScreenViewState>();
        view_state.cached_result = result;
    }
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = RoundPhase::GameOver;
    app.update();
    app.update();
}

fn advance_to_accounting(app: &mut App) {
    app.world_mut()
        .write_message(ResultScreenStepActionRequest::AdvanceToAccounting);
    app.update();
    assert_eq!(
        app.world().resource::<ResultScreenStepState>().current,
        ResultScreenStep::Accounting
    );
}

fn result(loser: Option<PlayerId>, reason: GameOverReason) -> S2CGameOver {
    S2CGameOver {
        loser,
        round: 9,
        reason,
    }
}

fn game_over_snapshot() -> S2CGameSnapshot {
    let own = player_snapshot(player(1), 8, 6, 2, 10);
    let opponent = player_snapshot(player(2), 11, 4, 1, 10);
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: own.player_id,
        round_number: 9,
        phase: RoundPhase::GameOver,
        timer_remaining_ms: None,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players: vec![own, opponent],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn player_snapshot(
    player_id: PlayerId,
    gold: u32,
    current_mana: u32,
    reserve_mana: u32,
    mana_cap: u8,
) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id: ClassId::Iop,
        gold,
        reserved_gold: 0,
        current_mana,
        reserve_mana,
        spawn_range_cells: 1,
        mana_cap,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: Vec::new(),
        opponent_objectives: Vec::new(),
    }
}
