//! S10-POLISH-001 — HUD Visual Chrome MVP — RESOLUTION dim overlay coverage.
//!
//! Acceptance criteria covered:
//! - Wired phase timer bar visible (HudTimerBar present, ImageNode wired).
//! - RESOLUTION dim overlay visible only while `Phase::Resolution`.
//! - Dim overlay is pre-pooled (entity ID stable across phase transitions).
//! - Single source of phase truth preserved (HudPlugin contains no system
//!   that writes to `CurrentClientPhase`).
//! - FROZEN-mode tiebreak: GAME_OVER hides dim overlay; later snapshot with
//!   `phase == Resolution` rebuilds dim overlay visibility.
//! - No countdown numerals on the timer bar (no `Text`/`TextSpan` child).
//! - `HUD_ENTITY_COUNT == 22` invariant after this story.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::{
    presentation::PresentationGameSnapshotMessage,
    state::{ClientState, CurrentClientPhase},
    ui::hud::{
        HudDimOverlay, HudEntities, HudEntity, HudPlayerIds, HudPlugin, HudTimerBar,
        HUD_DOTS_PER_ROW, HUD_ENTITY_COUNT,
    },
};
use shared::{
    card::ClassId,
    protocol::{
        BoardSnapshot, ObjectiveSnapshot, OpponentObjectiveSnapshot, PlayerSnapshot, RoundPhase,
        S2CGameSnapshot,
    },
    session::PlayerId,
};

#[path = "../../test_helpers.rs"]
mod test_helpers;

// ── Sub-test 1: Timer bar visible in timed phases (wired ImageNode) ──────────

#[test]
fn test_timer_bar_present_with_image_node() {
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();

    // After session entry, the timer bar entity exists and carries an
    // `ImageNode` (per PAW-004 + S10-POLISH-001 wiring). The image handle
    // is the asset_wiring constant — under test contexts it is the default
    // handle (no AssetServer); we assert the entity layout itself.
    let entities = hud_entities(&app);
    let timer_bar = entities.timer_bar;

    assert!(
        app.world().get::<HudTimerBar>(timer_bar).is_some(),
        "HudTimerBar marker must be on the wired entity"
    );
    assert!(
        app.world().get::<ImageNode>(timer_bar).is_some(),
        "Timer bar must use ImageNode (Bevy 0.18 Required Components API), not Sprite or Text"
    );
}

// ── Sub-test 2: No countdown numerals on the timer bar (HUD-11) ──────────────

#[test]
fn test_timer_bar_no_countdown_numerals() {
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let timer_bar = hud_entities(&app).timer_bar;

    assert!(
        app.world().get::<Text>(timer_bar).is_none(),
        "Timer bar entity itself must not carry Text (HUD-11 forbids countdown numerals)"
    );
    assert!(
        app.world().get::<TextSpan>(timer_bar).is_none(),
        "Timer bar entity itself must not carry TextSpan (HUD-11 forbids countdown numerals)"
    );

    // No descendants of the timer bar may carry Text/TextSpan either.
    if let Some(children) = app.world().get::<Children>(timer_bar) {
        for child in children.iter() {
            assert!(
                app.world().get::<Text>(child).is_none(),
                "Timer bar descendant must not carry Text (HUD-11)"
            );
            assert!(
                app.world().get::<TextSpan>(child).is_none(),
                "Timer bar descendant must not carry TextSpan (HUD-11)"
            );
        }
    }
}

// ── Sub-test 3: Dim overlay visible only while Phase::Resolution ─────────────

#[test]
fn test_dim_overlay_visible_only_in_resolution() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let dim_overlay = hud_entities(&app).dim_overlay;

    // Verify the marker + initial Visibility::Hidden invariant (LOBBY default).
    assert!(
        app.world().get::<HudDimOverlay>(dim_overlay).is_some(),
        "HudDimOverlay marker must be present on the pre-pooled overlay"
    );

    let non_resolution_phases = [
        RoundPhase::Handshaking,
        RoundPhase::Lobby,
        RoundPhase::DraftInitial,
        RoundPhase::DraftShop,
        RoundPhase::DraftAuction,
        RoundPhase::Placement,
        RoundPhase::GameOver,
    ];

    for phase in non_resolution_phases {
        set_current_phase(&mut app, phase);
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(dim_overlay),
            Some(&Visibility::Hidden),
            "Dim overlay must be Hidden in phase {phase:?}"
        );
    }

    set_current_phase(&mut app, RoundPhase::Resolution);
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(dim_overlay),
        Some(&Visibility::Visible),
        "Dim overlay must be Visible in Phase::Resolution"
    );

    // Lifts on transition out of Resolution.
    set_current_phase(&mut app, RoundPhase::DraftShop);
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(dim_overlay),
        Some(&Visibility::Hidden),
        "Dim overlay must lift on transition Resolution → DraftShop"
    );
}

// ── Sub-test 4: Dim overlay is pre-pooled (entity ID stable) ─────────────────

#[test]
fn test_dim_overlay_pre_pooled_entity_id_stable_across_phase_transitions() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let initial_dim_overlay = hud_entities(&app).dim_overlay;

    let transitions = [
        RoundPhase::Placement,
        RoundPhase::Resolution,
        RoundPhase::DraftShop,
        RoundPhase::Resolution,
        RoundPhase::GameOver,
    ];

    for phase in transitions {
        set_current_phase(&mut app, phase);
        app.update();
        let observed = hud_entities(&app).dim_overlay;
        assert_eq!(
            observed, initial_dim_overlay,
            "Dim overlay entity ID must remain stable across phase transitions \
             (pre-pooled, not spawned/despawned per phase). Observed change at phase {phase:?}"
        );
    }

    // Exactly one HudDimOverlay-marker entity exists in the world after all
    // transitions — no duplicate spawn.
    let mut count_query = app.world_mut().query::<&HudDimOverlay>();
    let dim_count = count_query.iter(app.world()).count();
    assert_eq!(
        dim_count, 1,
        "Exactly one HudDimOverlay entity must exist (pre-pooled, single instance)"
    );
}

// ── Sub-test 5: Single source of phase truth (TR-HUD-006 + ADR-002) ──────────

#[test]
fn test_no_client_side_phase_authority_in_dim_overlay_system() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();

    // The dim overlay system reads `Res<CurrentClientPhase>` and never
    // mutates it. Drive a transition by writing the resource directly,
    // run the schedule, and verify the resource value remains exactly what
    // the test set — no system in HudPlugin overwrites it from a synthetic
    // source.
    set_current_phase(&mut app, RoundPhase::Resolution);
    let round_marker = 4242_u32;
    {
        let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
        current.round = round_marker;
    }
    app.update();
    assert_eq!(
        app.world().resource::<CurrentClientPhase>().phase,
        RoundPhase::Resolution,
        "HudPlugin must not overwrite CurrentClientPhase.phase"
    );
    assert_eq!(
        app.world().resource::<CurrentClientPhase>().round,
        round_marker,
        "HudPlugin must not overwrite CurrentClientPhase.round"
    );

    // And switching back: the dim overlay system reacts but the resource
    // retains the value the test set.
    set_current_phase(&mut app, RoundPhase::DraftShop);
    app.update();
    assert_eq!(
        app.world().resource::<CurrentClientPhase>().phase,
        RoundPhase::DraftShop,
        "HudPlugin must not synthesize a phase change beyond the test's set value"
    );
}

// ── Sub-test 6: FROZEN-mode tiebreak (TR-HUD-009 + ADR-011) ──────────────────

#[test]
fn test_frozen_mode_tiebreak_dim_overlay_hidden_on_game_over_then_restored_by_snapshot() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let dim_overlay = hud_entities(&app).dim_overlay;

    // Enter Resolution → dim overlay visible.
    set_current_phase(&mut app, RoundPhase::Resolution);
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(dim_overlay),
        Some(&Visibility::Visible),
        "Pre-condition: dim overlay must be Visible in Resolution"
    );

    // GAME_OVER arrives → dim overlay must hide (FROZEN ≠ RESOLUTION).
    set_current_phase(&mut app, RoundPhase::GameOver);
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(dim_overlay),
        Some(&Visibility::Hidden),
        "On GAME_OVER FROZEN, dim overlay must be Hidden (FROZEN is not RESOLUTION)"
    );

    // A late S2CGameSnapshot with phase == Resolution arrives → snapshot
    // wins (ADR-011), CurrentClientPhase rebuilds to Resolution, dim
    // overlay restores to Visible.
    write_snapshot(
        &mut app,
        snapshot(RoundPhase::Resolution, 9, ClassId::Iop, ClassId::Cra),
    );
    app.update();
    assert_eq!(
        app.world().resource::<CurrentClientPhase>().phase,
        RoundPhase::Resolution,
        "Snapshot rebuild must restore CurrentClientPhase to Resolution (snapshot-wins per ADR-011)"
    );
    assert_eq!(
        app.world().get::<Visibility>(dim_overlay),
        Some(&Visibility::Visible),
        "Snapshot rebuild must restore dim overlay to Visible when phase == Resolution"
    );
}

// ── Sub-test 7: HUD_ENTITY_COUNT invariant (post-S10-POLISH-001 = 22) ────────

#[test]
fn test_hud_entity_count_is_twenty_two_after_dim_overlay_added() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();

    let observed = count_with::<HudEntity>(&mut app);
    assert_eq!(
        observed, HUD_ENTITY_COUNT,
        "HUD_ENTITY_COUNT constant ({HUD_ENTITY_COUNT}) must match the actual count \
         of HudEntity-marked entities after S10-POLISH-001 (dim overlay = +1)"
    );
    assert_eq!(
        HUD_ENTITY_COUNT, 22,
        "Post-S10-POLISH-001 invariant: HUD_ENTITY_COUNT must be 22 \
         (PAW-004 baseline 21 + dim overlay 1)"
    );
}

// ── Sub-test 8: Dim overlay carries HudEntity marker ─────────────────────────

#[test]
fn test_dim_overlay_carries_hud_entity_marker() {
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let dim_overlay = hud_entities(&app).dim_overlay;

    assert!(
        app.world().get::<HudEntity>(dim_overlay).is_some(),
        "Dim overlay must carry the HudEntity marker (counts toward HUD_ENTITY_COUNT)"
    );
    assert!(
        app.world().get::<BackgroundColor>(dim_overlay).is_some(),
        "Dim overlay must carry a BackgroundColor (the dim alpha)"
    );
    assert!(
        app.world().get::<Node>(dim_overlay).is_some(),
        "Dim overlay must be a Bevy 0.18 Node (Required Components API)"
    );
}

// ── Test fixture (canonical pattern from PROMPT 595/603/606) ─────────────────

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HudPlugin);
    app.insert_resource(HudPlayerIds {
        local_id: player(1),
        opponent_id: player(2),
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

fn set_current_phase(app: &mut App, phase: RoundPhase) {
    let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
    current.phase = phase;
}

fn snapshot(
    phase: RoundPhase,
    round_number: u32,
    own_class: ClassId,
    opponent_class: ClassId,
) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: player(1),
        round_number,
        phase,
        timer_remaining_ms: Some(12_000),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
        players: vec![
            player_snapshot(player(1), own_class),
            player_snapshot(player(2), opponent_class),
        ],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn player_snapshot(player_id: PlayerId, class_id: ClassId) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id,
        gold: 10,
        reserved_gold: 0,
        current_mana: 3,
        reserve_mana: 0,
        spawn_range_cells: 1,
        mana_cap: 10,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: empty_objectives(),
        opponent_objectives: empty_opponent_objectives(),
    }
}

fn empty_objectives() -> Vec<ObjectiveSnapshot> {
    (0..HUD_DOTS_PER_ROW)
        .map(|i| ObjectiveSnapshot {
            lane: (i + 1) as u8,
            hp: 3,
            is_real: false,
            is_destroyed: false,
        })
        .collect()
}

fn empty_opponent_objectives() -> Vec<OpponentObjectiveSnapshot> {
    (0..HUD_DOTS_PER_ROW)
        .map(|i| OpponentObjectiveSnapshot {
            lane: (i + 1) as u8,
            hp: 3,
            is_destroyed: false,
            was_fake: None,
        })
        .collect()
}

fn write_snapshot(app: &mut App, snapshot: S2CGameSnapshot) {
    app.world_mut()
        .resource_mut::<Messages<PresentationGameSnapshotMessage>>()
        .write(PresentationGameSnapshotMessage(snapshot));
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
