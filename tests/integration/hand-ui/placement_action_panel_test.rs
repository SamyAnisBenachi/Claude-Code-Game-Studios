//! PROMPT 1043 — Placement Action Panel + Submit Affordance P1 repair.
//!
//! Asserts that the placement phase exposes a structured action panel
//! (container with chrome, header, disclosure text, countdown, placed-count
//! readout, and a real Submit button) — replacing the previous floating
//! `Select a card / 8 / Submit / (0` left-column text fragments documented
//! by `reports/PROMPT-1034-full-ui-visual-quality-audit.md` §2.3 / §3 D3-D4
//! and `reports/PROMPT-1036-snapshot-state-log-correlation-audit.md` §4.5
//! (five consecutive Placement rounds closing with `committed_players=0`).
//!
//! Coverage:
//! - AC1: a `PlacementActionPanel` entity exists, carries `BackgroundColor`
//!   + `BorderColor` chrome, sits inside the hand-UI subtree, and goes
//!   `Visibility::Visible` during Placement / `Visibility::Hidden` outside.
//! - AC2: the action surface elements (disclosure guidance, timer,
//!   placed-count readout, submit button, submitted checkmark) are children
//!   of the panel — not floating siblings on `HandFanRoot`.
//! - AC3: the Submit button entity is a real button — `Button + Interaction`
//!   marker, intrinsic node ≥ 120 px wide (so "Submit (X cards)" cannot
//!   wrap into the truncated `(0` fragment the audit captured),
//!   `BackgroundColor` + `BorderColor` chrome, `border_radius > 0`, and a
//!   non-Auto pixel height.
//! - AC4: the placed-count readout updates from "0 placed" → "1 placed"
//!   when a card is staged via `HandUiPlacementDropResolved`, proving the
//!   readout reflects `PendingPlacements.staged_count()`.
//! - AC5: clicking the submit button (the canonical
//!   `HandSubmitButtonClicked` message path) emits the existing
//!   server-authoritative `C2SSubmitPlacement` — confirming the visual
//!   repair did NOT functionally rewire the submit message contract.

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::{
    hand::{
        FanSlotIndex, HandCardCatalog, HandContents, HandSubmitButton, HandSubmitButtonClicked,
        HandTimer, HandUiEntities, HandUiOutboundMessages, HandUiPlacementDropResolved,
        HandUiPlugin, PlacedCountReadout, PlacementActionPanelHeader, PlacementDisclosureGuidance,
        TimerSubmittedCheckmark,
    },
    shared::BoardLayout,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn ac1_action_panel_exists_with_chrome_and_visible_during_placement() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(10), CardType::Minion)]));
    set_hand(&mut app, [CardId(10)]);

    let panel = panel_entity(&mut app);

    // Panel chrome must exist — these are the visual difference between
    // "real bordered panel" and "floating text fragments".
    assert!(
        app.world().get::<BackgroundColor>(panel).is_some(),
        "AC1 placement action panel must carry a BackgroundColor so it \
         reads as a bordered surface, not as floating text",
    );
    assert!(
        app.world().get::<BorderColor>(panel).is_some(),
        "AC1 placement action panel must carry a BorderColor so the \
         affordance is visually delineated from the playfield",
    );

    let panel_node = app
        .world()
        .get::<Node>(panel)
        .expect("AC1 panel must have a Node component");
    assert!(
        !matches!(panel_node.width, Val::Auto),
        "AC1 panel width must be a non-Auto pixel value so the chrome \
         has a deterministic footprint; got {:?}",
        panel_node.width,
    );
    match panel_node.border {
        UiRect {
            left: Val::Px(left),
            right: Val::Px(right),
            top: Val::Px(top),
            bottom: Val::Px(bottom),
        } => {
            assert!(
                left > 0.0 && right > 0.0 && top > 0.0 && bottom > 0.0,
                "AC1 panel border must be > 0 on every edge; got \
                 left={left} right={right} top={top} bottom={bottom}",
            );
        }
        other => panic!(
            "AC1 panel border must be UiRect of Val::Px on every edge; \
             got {other:?}"
        ),
    }
    assert_eq!(
        panel_node.display,
        Display::Flex,
        "AC1 panel must use Display::Flex so children stack in a column",
    );
    assert_eq!(
        panel_node.flex_direction,
        FlexDirection::Column,
        "AC1 panel must lay children out as a column so the action stack \
         reads top-to-bottom",
    );

    // Visibility tracks staging mode — Visible during Placement, Hidden
    // outside. AC1 asserts the panel is visible during Placement (the
    // session this test builds starts in Placement).
    assert_eq!(
        app.world().get::<Visibility>(panel),
        Some(&Visibility::Visible),
        "AC1 panel must be Visible while the hand UI is in Staging mode",
    );

    // And Hidden after the phase leaves Placement.
    set_phase(&mut app, RoundPhase::DraftShop);
    run_update(&mut app);
    assert_eq!(
        app.world().get::<Visibility>(panel),
        Some(&Visibility::Hidden),
        "AC1 panel must hide its chrome when the hand UI exits Staging mode",
    );
}

#[test]
fn ac2_action_surface_elements_parented_inside_panel() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(20), CardType::Minion)]));
    set_hand(&mut app, [CardId(20)]);

    let panel = panel_entity(&mut app);
    let header = single_entity_with::<PlacementActionPanelHeader>(&mut app);
    let guidance = single_entity_with::<PlacementDisclosureGuidance>(&mut app);
    let timer = single_entity_with::<HandTimer>(&mut app);
    let checkmark = single_entity_with::<TimerSubmittedCheckmark>(&mut app);
    let placed_count = single_entity_with::<PlacedCountReadout>(&mut app);
    let submit = single_entity_with::<HandSubmitButton>(&mut app);

    assert_ancestor(&mut app, header, panel, "panel header");
    assert_ancestor(&mut app, guidance, panel, "placement disclosure guidance");
    assert_ancestor(&mut app, timer, panel, "placement countdown timer");
    assert_ancestor(&mut app, checkmark, panel, "timer submitted checkmark");
    assert_ancestor(&mut app, placed_count, panel, "placed-count readout");
    assert_ancestor(&mut app, submit, panel, "submit button");
}

#[test]
fn ac3_submit_button_is_a_real_chromed_button_wide_enough_for_label() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(30), CardType::Minion)]));
    set_hand(&mut app, [CardId(30)]);

    let submit = single_entity_with::<HandSubmitButton>(&mut app);

    assert!(
        app.world().get::<Button>(submit).is_some(),
        "AC3 submit button must carry the Button marker so input picking \
         routes Interaction::Pressed through it",
    );
    assert!(
        app.world().get::<Interaction>(submit).is_some(),
        "AC3 submit button must carry Interaction so the framework \
         updates its state in response to pointer events",
    );
    assert!(
        app.world().get::<BackgroundColor>(submit).is_some(),
        "AC3 submit button must carry BackgroundColor so it reads as a \
         button backplate, not bare text floating on the parent surface",
    );
    assert!(
        app.world().get::<BorderColor>(submit).is_some(),
        "AC3 submit button must carry BorderColor so the affordance has \
         a visible edge",
    );

    let node = app
        .world()
        .get::<Node>(submit)
        .expect("AC3 submit button must have a Node");
    // The previous regression — submit text "Submit (0 cards)" wrapped
    // into "(0" because the node was only 96 px wide. Guard a comfortable
    // floor that fits the longest label cycle ("Submitted" / "Submit (0
    // cards)") so we cannot regress to a truncated fragment.
    match node.width {
        Val::Px(width_px) => assert!(
            width_px >= 120.0,
            "AC3 submit button width must be ≥ 120 px so the label cannot \
             wrap into a truncated fragment; got {width_px} px"
        ),
        other => panic!(
            "AC3 submit button width must be a concrete Val::Px so the \
             label cannot wrap; got {other:?}"
        ),
    }
    match node.height {
        Val::Px(height_px) => assert!(
            height_px >= 32.0,
            "AC3 submit button height must be ≥ 32 px so the button \
             reads as a tap target; got {height_px} px"
        ),
        other => panic!(
            "AC3 submit button height must be a concrete Val::Px so the \
             button has deterministic visual height; got {other:?}"
        ),
    }

    let border_radius_present = match node.border_radius.top_left {
        Val::Px(px) => px > 0.0,
        Val::Percent(pct) => pct > 0.0,
        _ => false,
    };
    assert!(
        border_radius_present,
        "AC3 submit button must declare a non-zero top-left border_radius \
         so the affordance reads as a rounded button rather than a bare \
         rectangle",
    );
}

#[test]
fn ac4_placed_count_readout_tracks_pending_placements() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([
        (CardId(40), CardType::Minion),
        (CardId(41), CardType::Minion),
    ]));
    set_hand(&mut app, [CardId(40), CardId(41)]);

    // On entry, no cards staged → "0 placed".
    assert_eq!(placed_count_text(&mut app), "0 placed");

    // Stage one card.
    stage_card_in_slot(
        &mut app,
        0,
        PlayerId(1),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );
    assert_eq!(
        placed_count_text(&mut app),
        "1 placed",
        "AC4 placed-count readout must reflect PendingPlacements.staged_count() \
         after a card is staged",
    );

    // Stage a second card.
    stage_card_in_slot(
        &mut app,
        1,
        PlayerId(1),
        PlayTarget::BoardCell { lane: 1, cell: 2 },
    );
    assert_eq!(placed_count_text(&mut app), "2 placed");

    // Leave staging → readout blanks (no orphan number on other phases).
    set_phase(&mut app, RoundPhase::DraftShop);
    run_update(&mut app);
    assert_eq!(
        placed_count_text(&mut app),
        "",
        "AC4 placed-count readout must blank when the hand UI is not in \
         Staging mode so the chrome does not leak into DraftShop / Auction",
    );
}

#[test]
fn ac5_submit_click_still_emits_c2s_submit_placement() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(50), CardType::Minion)]));
    set_hand(&mut app, [CardId(50)]);
    stage_card_in_slot(
        &mut app,
        0,
        PlayerId(1),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );

    let button = single_entity_with::<HandSubmitButton>(&mut app);
    app.world_mut()
        .write_message(HandSubmitButtonClicked { button });
    run_update(&mut app);

    let submissions: Vec<_> = app
        .world()
        .resource::<HandUiOutboundMessages>()
        .submit_placements
        .clone();
    assert_eq!(
        submissions.len(),
        1,
        "AC5 visual repair must NOT functionally rewire the submit path — \
         a click on the new chromed Submit button must still emit exactly \
         one C2SSubmitPlacement message (the existing server-authoritative \
         flow)",
    );
    assert_eq!(
        submissions[0].placements.len(),
        1,
        "AC5 the submitted C2SSubmitPlacement must carry the one staged \
         placement; got {:?}",
        submissions[0].placements
    );
}

// ── helpers ──────────────────────────────────────────────────────────────

fn app_with_hand_ui_in_placement(catalog: HashMap<CardId, CardData>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HandUiPlugin);
    app.insert_resource(BoardLayout {
        board_origin: Vec2::ZERO,
        cell_width: 64.0,
        lane_height: 80.0,
    });
    app.insert_resource(HandCardCatalog { cards: catalog });
    {
        let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
        economy.current_mana = 10;
        economy.reserve_mana = 10;
    }
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_phase(&mut app, RoundPhase::Placement);
    run_update(&mut app);
    app
}

fn test_catalog<const N: usize>(entries: [(CardId, CardType); N]) -> HashMap<CardId, CardData> {
    entries
        .into_iter()
        .map(|(card_id, card_type)| (card_id, test_card(card_id, card_type)))
        .collect()
}

fn test_card(card_id: CardId, card_type: CardType) -> CardData {
    CardData {
        id: card_id,
        name_fr: format!("Carte {}", card_id.0),
        name_en: format!("Card {}", card_id.0),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type,
        unit_type: UnitType::Blade,
        cost: 1,
        atk: 1,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: format!("test_{}", card_id.0),
        pool_copies_override: None,
    }
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn set_hand<const N: usize>(app: &mut App, cards: [CardId; N]) {
    app.world_mut().resource_mut::<HandContents>().cards = cards.to_vec();
    run_update(app);
}

fn stage_card_in_slot(app: &mut App, slot_index: u8, owner_id: PlayerId, target: PlayTarget) {
    let slot = fan_slot(app, slot_index);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id,
        target: Some(target),
    });
    run_update(app);
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}

fn panel_entity(app: &mut App) -> Entity {
    app.world()
        .resource::<HandUiEntities>()
        .placement_action_panel
}

fn single_entity_with<M: Component>(app: &mut App) -> Entity {
    let mut query = app.world_mut().query_filtered::<Entity, With<M>>();
    query
        .single(app.world())
        .expect("expected exactly one entity with the requested marker")
}

fn placed_count_text(app: &mut App) -> String {
    let mut query = app
        .world_mut()
        .query_filtered::<&Text, With<PlacedCountReadout>>();
    query
        .single(app.world())
        .expect("placed-count readout should exist")
        .0
        .clone()
}

fn assert_ancestor(app: &mut App, child: Entity, expected_ancestor: Entity, label: &str) {
    let mut cursor = child;
    let mut hops = 0;
    while let Some(child_of) = app.world().get::<ChildOf>(cursor) {
        let parent = child_of.parent();
        if parent == expected_ancestor {
            return;
        }
        cursor = parent;
        hops += 1;
        if hops > 8 {
            break;
        }
    }
    panic!(
        "AC2 {label} entity {child:?} must be a descendant of the \
         placement action panel {expected_ancestor:?}",
    );
}
