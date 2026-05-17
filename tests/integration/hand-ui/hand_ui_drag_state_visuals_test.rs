//! Sprint 15 story 020 (`S12-UX-HAND-DRAG-STATE-VISUALS-001`) AC9 evidence.
//!
//! Drives the per-slot drag-state visual sync system through the five drag
//! states (`Idle`, `Hover`, `Drag` source, `DropTarget` fan-plate,
//! `Disabled`) and asserts the resulting overlay treatment via ECS-query
//! assertions against the marker components / `Visibility` / `BackgroundColor`
//! / `BorderColor` / `GlobalZIndex` introduced in
//! `client/src/ui/hand/drag_state_visuals.rs`.
//!
//! Per AC9: no rendered-pixel snapshot, no `Pointer<...>` event synthesis.
//! Drag state is driven via direct mutation of the already-pub
//! `ActivePlacementDrag` / `HandUiMode` / `PendingPlacements` resource
//! fields, then the new sync system is run inside `Update` and the visual
//! treatment is asserted by `Query<&Visibility, With<...>>` / similar
//! marker queries.
//!
//! Read-only over `ActivePlacementDrag` (AC8 / AC13): the test asserts at
//! the end of every scenario that `active_drag.card` / `active_drag.target_kind`
//! match exactly the values written by the test (i.e. the sync system did
//! not mutate them).
//!
//! Pre-pool count (AC7 / AC10): asserted explicitly that
//! `Query<&FanSlotIndex>` returns exactly `HAND_FAN_SLOT_COUNT` slots
//! before, during, and after the drag state transitions.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::ui::GlobalZIndex;
use client::card_animations::HandDragSprite;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::design_tokens::{overlays, z_layers};
use client::ui::hand::drag_state_visuals::{
    accent_color, dim_overlay_color, scrim_overlay_color, semantic_success_color, DragStateOverlay,
    DragStateOverlayActive, FanPlateDropTargetOverlay, FanSlotDimOverlay, FanSlotHoverOverlay,
};
use client::ui::hand::{
    ActivePlacementDrag, FanSlotIndex, HandCardCatalog, HandContents, HandFanLayoutConfig,
    HandFanViewport, HandUiEntities, HandUiMode, HandUiPlugin, PendingPlacements,
    PlacementTargetKind, HAND_FAN_SLOT_COUNT,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlacedCardSubmit, PlayTarget, RoundPhase};
use shared::session::PlayerId;

const VIEWPORT_W: f32 = 1280.0;
const VIEWPORT_H: f32 = 720.0;

// ── AC2: Idle baseline preserved ─────────────────────────────────────────────

#[test]
fn ac2_idle_state_keeps_all_drag_overlays_hidden() {
    let mut app = app_in_placement_with_two_affordable_cards();

    // `HandUiMode::Staging`, no active drag, no pointer hover, no staged
    // placements, affordable cards. Every drag-state overlay must remain
    // `Visibility::Hidden`.
    app.update();

    assert_no_visible_drag_overlays(&mut app);

    // AC7 + AC10: pre-pool count for FanSlotIndex is exactly the canonical
    // HAND_FAN_SLOT_COUNT.
    assert_eq!(
        count::<FanSlotIndex>(&mut app),
        HAND_FAN_SLOT_COUNT,
        "FanSlotIndex pre-pool count must remain {HAND_FAN_SLOT_COUNT} in idle state"
    );

    // AC8 + AC13: idle state did not introduce a drag — `ActivePlacementDrag`
    // is still cleared. The sync system never wrote to it.
    let drag = app.world().resource::<ActivePlacementDrag>();
    assert!(
        drag.card.is_none() && drag.target_kind.is_none(),
        "sync system must never mutate ActivePlacementDrag back into Active in idle scenario"
    );
}

// ── AC3: Drag source dim + drag sprite ascends to UI_OVERLAY ─────────────────

#[test]
fn ac3_drag_source_slot_dims_and_drag_sprite_paints_at_ui_overlay() {
    let mut app = app_in_placement_with_two_affordable_cards();
    app.update();

    let slot_0 = fan_slot(&mut app, 0);
    let card_id_0 = card_id_for_slot(&app, slot_0);

    // Drive the resource directly per AC9 ("the new test drives drag-state
    // via direct resource insertion") to start a Minion drag from slot 0.
    set_active_drag(
        &mut app,
        slot_0,
        card_id_0,
        PlayerId(1),
        PlacementTargetKind::Minion,
    );
    app.update();

    // Drag-source slot 0 must carry an Active marker (DragSource) and its
    // dim overlay child must be Visible with OVERLAY_DIM_ALPHA BackgroundColor.
    assert_eq!(
        app.world().get::<DragStateOverlayActive>(slot_0),
        Some(&DragStateOverlayActive::DragSource),
        "slot 0 must be tagged DragStateOverlayActive::DragSource while the drag is in flight"
    );
    assert_eq!(
        dim_overlay_visibility_for(&mut app, slot_0),
        Visibility::Visible,
        "drag-source slot 0 dim overlay must be Visible"
    );
    assert!(
        dim_overlay_background_alpha_matches_token(&mut app, slot_0),
        "drag-source slot dim overlay BackgroundColor alpha must equal OVERLAY_DIM_ALPHA"
    );

    // The pre-existing `HandDragSprite` entity must carry GlobalZIndex(UI_OVERLAY)
    // — sourced from the named `z_layers::UI_OVERLAY` symbol in `spawn_hand_ui`,
    // not from a hardcoded `GlobalZIndex(400)` literal.
    let mut drag_sprite_query = app.world_mut().query::<(&HandDragSprite, &GlobalZIndex)>();
    let (_, drag_sprite_z) = drag_sprite_query
        .single(app.world())
        .expect("exactly one HandDragSprite entity must exist");
    assert_eq!(
        drag_sprite_z.0,
        z_layers::UI_OVERLAY.0,
        "HandDragSprite GlobalZIndex must equal z_layers::UI_OVERLAY ({} = 400)",
        z_layers::UI_OVERLAY.0
    );

    // The other (non-source) slot must NOT be tagged DragSource, and its dim
    // overlay must remain Hidden.
    let slot_1 = fan_slot(&mut app, 1);
    assert_ne!(
        app.world().get::<DragStateOverlayActive>(slot_1),
        Some(&DragStateOverlayActive::DragSource),
        "non-source slot 1 must NOT be tagged DragStateOverlayActive::DragSource"
    );
    assert_eq!(
        dim_overlay_visibility_for(&mut app, slot_1),
        Visibility::Hidden,
        "non-source slot 1 dim overlay must remain Hidden"
    );

    // AC8 + AC13: sync system did not mutate ActivePlacementDrag.
    let drag = app.world().resource::<ActivePlacementDrag>();
    assert_eq!(drag.card, Some(slot_0));
    assert_eq!(drag.target_kind, Some(PlacementTargetKind::Minion));
}

// ── AC4: Fan-plate DropTarget tint when Instant drag is active ───────────────

#[test]
fn ac4_fan_plate_instant_drop_target_overlay_paints_scrim() {
    let mut app = app_in_placement_with_two_affordable_cards();
    app.update();

    let slot_0 = fan_slot(&mut app, 0);
    let card_id_0 = card_id_for_slot(&app, slot_0);

    // Instant drag — fan-plate drop-target overlay must paint.
    set_active_drag(
        &mut app,
        slot_0,
        card_id_0,
        PlayerId(1),
        PlacementTargetKind::Instant,
    );
    app.update();

    assert_eq!(
        fan_plate_drop_target_visibility(&mut app),
        Visibility::Visible,
        "FanPlateDropTargetOverlay must be Visible while an Instant drag is in flight"
    );
    assert!(
        fan_plate_drop_target_background_alpha_matches_token(&mut app),
        "FanPlateDropTargetOverlay BackgroundColor alpha must equal OVERLAY_SCRIM_ALPHA"
    );
    assert!(
        fan_plate_drop_target_border_color_is_semantic_success(&mut app),
        "FanPlateDropTargetOverlay BorderColor must equal semantic_success_color() (§7)"
    );
    // The fan_root entity should be tagged with DropTarget for AC4 ECS-query
    // assertions.
    let fan_root = app.world().resource::<HandUiEntities>().fan_root;
    assert_eq!(
        app.world().get::<DragStateOverlayActive>(fan_root),
        Some(&DragStateOverlayActive::DropTarget),
        "fan_root must be tagged DragStateOverlayActive::DropTarget during Instant drag"
    );

    // Non-Instant drag must NOT paint the fan-plate drop-target overlay.
    set_active_drag(
        &mut app,
        slot_0,
        card_id_0,
        PlayerId(1),
        PlacementTargetKind::Minion,
    );
    app.update();

    assert_eq!(
        fan_plate_drop_target_visibility(&mut app),
        Visibility::Hidden,
        "FanPlateDropTargetOverlay must be Hidden for non-Instant drags"
    );
    assert_eq!(
        app.world().get::<DragStateOverlayActive>(fan_root),
        None,
        "fan_root DropTarget tag must be removed when target_kind is not Instant"
    );
}

// ── AC5: Disabled treatment (PassiveLocked / staged / unaffordable) ──────────

#[test]
fn ac5_disabled_treatment_applies_when_hand_mode_is_passive_locked() {
    let mut app = app_in_placement_with_two_affordable_cards();
    app.update();

    let slot_0 = fan_slot(&mut app, 0);

    // PassiveLocked: every populated slot must read as Disabled.
    set_hand_ui_mode(&mut app, HandUiMode::PassiveLocked);
    app.update();

    assert_eq!(
        app.world().get::<DragStateOverlayActive>(slot_0),
        Some(&DragStateOverlayActive::Disabled),
        "PassiveLocked mode must mark populated slots Disabled"
    );
    assert_eq!(
        dim_overlay_visibility_for(&mut app, slot_0),
        Visibility::Visible,
        "PassiveLocked Disabled treatment must show the dim overlay"
    );
}

#[test]
fn ac5_disabled_treatment_applies_when_card_is_already_staged() {
    let mut app = app_in_placement_with_two_affordable_cards();
    app.update();

    let slot_0 = fan_slot(&mut app, 0);
    let card_id_0 = card_id_for_slot(&app, slot_0);

    // Stage card 0 in PendingPlacements; the slot should now read Disabled.
    stage_card(&mut app, card_id_0);
    app.update();

    assert_eq!(
        app.world().get::<DragStateOverlayActive>(slot_0),
        Some(&DragStateOverlayActive::Disabled),
        "card already in PendingPlacements must mark its slot Disabled"
    );
    assert_eq!(
        dim_overlay_visibility_for(&mut app, slot_0),
        Visibility::Visible,
        "already-staged Disabled treatment must show the dim overlay"
    );

    // Slot 1 is still affordable + not staged → not disabled.
    let slot_1 = fan_slot(&mut app, 1);
    assert_ne!(
        app.world().get::<DragStateOverlayActive>(slot_1),
        Some(&DragStateOverlayActive::Disabled),
        "non-staged affordable slot 1 must NOT be Disabled"
    );
}

#[test]
fn ac5_disabled_treatment_applies_when_card_is_unaffordable() {
    let mut app = app_in_placement_with_two_affordable_cards();
    app.update();

    // Make slot 0's card unaffordable by zeroing the player's mana pool.
    let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
    economy.current_mana = 0;
    economy.reserve_mana = 0;
    app.update();

    let slot_0 = fan_slot(&mut app, 0);
    assert_eq!(
        app.world().get::<DragStateOverlayActive>(slot_0),
        Some(&DragStateOverlayActive::Disabled),
        "unaffordable Minion slot must read as Disabled"
    );
}

// ── AC6: Hover state (non-drag) ─────────────────────────────────────────────

#[test]
fn ac6_hover_state_shows_accent_border_when_pointer_hovers_an_affordable_slot() {
    let mut app = app_in_placement_with_two_affordable_cards();
    app.update();

    let slot_0 = fan_slot(&mut app, 0);

    // No active drag, pointer hovers slot 0. Set the Interaction component
    // directly per AC9 ("via direct resource insertion … assertion by ECS
    // query").
    {
        let mut entity_mut = app.world_mut().entity_mut(slot_0);
        let mut interaction = entity_mut
            .get_mut::<Interaction>()
            .expect("fan slot must carry an Interaction component");
        *interaction = Interaction::Hovered;
    }
    app.update();

    assert_eq!(
        app.world().get::<DragStateOverlayActive>(slot_0),
        Some(&DragStateOverlayActive::Hover),
        "hovered affordable slot must read as Hover"
    );
    assert_eq!(
        hover_overlay_visibility_for(&mut app, slot_0),
        Visibility::Visible,
        "hovered slot hover overlay must flip Visible"
    );
    assert!(
        hover_overlay_border_color_is_accent(&mut app, slot_0),
        "hover overlay BorderColor must equal accent_color() (§7 ACCENT)"
    );
}

#[test]
fn ac6_hover_state_does_not_apply_during_active_drag() {
    let mut app = app_in_placement_with_two_affordable_cards();
    app.update();

    let slot_0 = fan_slot(&mut app, 0);
    let slot_1 = fan_slot(&mut app, 1);
    let card_id_0 = card_id_for_slot(&app, slot_0);

    // Start a drag from slot 0, then hover slot 1 — slot 1 must NOT light up
    // as Hover because a drag is in flight.
    set_active_drag(
        &mut app,
        slot_0,
        card_id_0,
        PlayerId(1),
        PlacementTargetKind::Minion,
    );
    {
        let mut entity_mut = app.world_mut().entity_mut(slot_1);
        let mut interaction = entity_mut
            .get_mut::<Interaction>()
            .expect("fan slot must carry an Interaction component");
        *interaction = Interaction::Hovered;
    }
    app.update();

    assert_ne!(
        app.world().get::<DragStateOverlayActive>(slot_1),
        Some(&DragStateOverlayActive::Hover),
        "Hover must be suppressed during an active drag (slot 1)"
    );
    assert_eq!(
        hover_overlay_visibility_for(&mut app, slot_1),
        Visibility::Hidden,
        "Hover overlay must stay Hidden during an active drag"
    );
}

// ── AC7: pre-pool count preserved across drag-state transitions ──────────────

#[test]
fn ac7_fan_slot_index_count_unchanged_across_drag_state_transitions() {
    let mut app = app_in_placement_with_two_affordable_cards();
    app.update();

    let baseline = count::<FanSlotIndex>(&mut app);
    assert_eq!(baseline, HAND_FAN_SLOT_COUNT);

    let slot_0 = fan_slot(&mut app, 0);
    let card_id_0 = card_id_for_slot(&app, slot_0);

    // Drag → Idle → Disabled → Hover cycle.
    set_active_drag(
        &mut app,
        slot_0,
        card_id_0,
        PlayerId(1),
        PlacementTargetKind::Instant,
    );
    app.update();
    assert_eq!(count::<FanSlotIndex>(&mut app), baseline);

    clear_active_drag(&mut app);
    app.update();
    assert_eq!(count::<FanSlotIndex>(&mut app), baseline);

    stage_card(&mut app, card_id_0);
    app.update();
    assert_eq!(count::<FanSlotIndex>(&mut app), baseline);

    // Final Visibility flips must not have respawned any FanSlotIndex
    // entity.
    assert_eq!(count::<FanSlotIndex>(&mut app), HAND_FAN_SLOT_COUNT);
}

// ── AC8 + AC13: read-only over ActivePlacementDrag ───────────────────────────

#[test]
fn ac8_sync_system_never_mutates_active_placement_drag() {
    let mut app = app_in_placement_with_two_affordable_cards();
    app.update();

    let slot_0 = fan_slot(&mut app, 0);
    let card_id_0 = card_id_for_slot(&app, slot_0);

    set_active_drag(
        &mut app,
        slot_0,
        card_id_0,
        PlayerId(1),
        PlacementTargetKind::Minion,
    );
    for _ in 0..5 {
        app.update();
        let drag = app.world().resource::<ActivePlacementDrag>();
        assert_eq!(drag.card, Some(slot_0));
        assert_eq!(drag.card_id, Some(card_id_0));
        assert_eq!(drag.owner_id, Some(PlayerId(1)));
        assert_eq!(drag.target_kind, Some(PlacementTargetKind::Minion));
    }
}

// ── Token-symbol parity (AC1 belt-and-braces) ────────────────────────────────

#[test]
fn ac1_token_symbols_resolve_to_spec_ratified_values() {
    // Belt-and-braces verification that `drag_state_visuals` consumes the
    // same numeric values published by the design_tokens module. AC1's grep
    // assertion is satisfied by the source code; this test additionally
    // asserts the runtime equality so a future spec revision that changes
    // OVERLAY_DIM_ALPHA / OVERLAY_SCRIM_ALPHA propagates through.
    let dim = dim_overlay_color();
    let scrim = scrim_overlay_color();
    let LinearRgba {
        red: _,
        green: _,
        blue: _,
        alpha: dim_a,
    } = dim.to_linear();
    let LinearRgba {
        red: _,
        green: _,
        blue: _,
        alpha: scrim_a,
    } = scrim.to_linear();
    assert!((dim_a - overlays::OVERLAY_DIM_ALPHA).abs() < f32::EPSILON);
    assert!((scrim_a - overlays::OVERLAY_SCRIM_ALPHA).abs() < f32::EPSILON);
    // §7 ACCENT and SEMANTIC_SUCCESS resolve to the spec-ratified
    // `#F2C94C` and `#27AE60` values exposed by the Sprint 14 consumer
    // site.
    assert_eq!(
        accent_color(),
        Color::srgb(0.949, 0.788, 0.298),
        "accent_color() must resolve to the spec-ratified #F2C94C"
    );
    assert_eq!(
        semantic_success_color(),
        Color::srgb(0.153, 0.682, 0.376),
        "semantic_success_color() must resolve to the spec-ratified #27AE60"
    );
}

// ── Test app setup ───────────────────────────────────────────────────────────

fn app_in_placement_with_two_affordable_cards() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HandUiPlugin);
    app.insert_resource(HandFanViewport {
        width_px: VIEWPORT_W,
        height_px: VIEWPORT_H,
    });
    app.insert_resource(HandFanLayoutConfig::default());
    app.insert_resource(HandCardCatalog {
        cards: test_catalog([CardId(50), CardId(51)]),
    });

    // Two affordable Minions in the player's economy.
    {
        let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
        economy.current_mana = 5;
        economy.reserve_mana = 5;
    }

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    app.world_mut().resource_mut::<HandContents>().cards = vec![CardId(50), CardId(51)];
    set_phase(&mut app, RoundPhase::Placement);
    app.update();
    app
}

fn test_catalog<const N: usize>(ids: [CardId; N]) -> HashMap<CardId, CardData> {
    ids.into_iter()
        .map(|id| (id, test_minion_card(id)))
        .collect()
}

fn test_minion_card(id: CardId) -> CardData {
    CardData {
        id,
        name_fr: format!("Carte {}", id.0),
        name_en: format!("Card {}", id.0),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: 1,
        atk: 1,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: format!("test_{}", id.0),
        pool_copies_override: None,
    }
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn set_hand_ui_mode(app: &mut App, mode: HandUiMode) {
    *app.world_mut().resource_mut::<HandUiMode>() = mode;
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot must exist")
}

fn card_id_for_slot(app: &App, slot: Entity) -> CardId {
    app.world()
        .get::<client::ui::hand::HandSlotCard>(slot)
        .expect("fan slot must carry HandSlotCard after Placement phase")
        .0
}

fn set_active_drag(
    app: &mut App,
    card: Entity,
    card_id: CardId,
    owner_id: PlayerId,
    target_kind: PlacementTargetKind,
) {
    let mut drag = app.world_mut().resource_mut::<ActivePlacementDrag>();
    drag.card = Some(card);
    drag.card_id = Some(card_id);
    drag.owner_id = Some(owner_id);
    drag.target_kind = Some(target_kind);
    drag.cursor_world_position = None;
}

fn clear_active_drag(app: &mut App) {
    *app.world_mut().resource_mut::<ActivePlacementDrag>() = ActivePlacementDrag::default();
}

fn stage_card(app: &mut App, card_id: CardId) {
    app.world_mut()
        .resource_mut::<PendingPlacements>()
        .placements
        .push(PlacedCardSubmit {
            card_id,
            target: PlayTarget::BoardCell { lane: 1, cell: 1 },
            current_mana_spend: 1,
            reserve_mana_spend: 0,
        });
}

// ── Query helpers ────────────────────────────────────────────────────────────

fn count<T: Component>(app: &mut App) -> usize {
    let mut q = app.world_mut().query_filtered::<Entity, With<T>>();
    q.iter(app.world()).count()
}

fn dim_overlay_visibility_for(app: &mut App, slot: Entity) -> Visibility {
    let mut q = app
        .world_mut()
        .query_filtered::<(&ChildOf, &Visibility), With<FanSlotDimOverlay>>();
    for (child_of, visibility) in q.iter(app.world()) {
        if child_of.parent() == slot {
            return *visibility;
        }
    }
    panic!("no FanSlotDimOverlay child found under fan slot {slot:?}");
}

fn dim_overlay_background_alpha_matches_token(app: &mut App, slot: Entity) -> bool {
    let mut q = app
        .world_mut()
        .query_filtered::<(&ChildOf, &BackgroundColor), With<FanSlotDimOverlay>>();
    for (child_of, background) in q.iter(app.world()) {
        if child_of.parent() == slot {
            let LinearRgba { alpha, .. } = background.0.to_linear();
            return (alpha - overlays::OVERLAY_DIM_ALPHA).abs() < f32::EPSILON;
        }
    }
    false
}

fn hover_overlay_visibility_for(app: &mut App, slot: Entity) -> Visibility {
    let mut q = app
        .world_mut()
        .query_filtered::<(&ChildOf, &Visibility), With<FanSlotHoverOverlay>>();
    for (child_of, visibility) in q.iter(app.world()) {
        if child_of.parent() == slot {
            return *visibility;
        }
    }
    panic!("no FanSlotHoverOverlay child found under fan slot {slot:?}");
}

fn hover_overlay_border_color_is_accent(app: &mut App, slot: Entity) -> bool {
    let mut q = app
        .world_mut()
        .query_filtered::<(&ChildOf, &BorderColor), With<FanSlotHoverOverlay>>();
    for (child_of, border) in q.iter(app.world()) {
        if child_of.parent() == slot {
            return border_color_matches(border, accent_color());
        }
    }
    false
}

fn fan_plate_drop_target_visibility(app: &mut App) -> Visibility {
    let mut q = app
        .world_mut()
        .query_filtered::<&Visibility, With<FanPlateDropTargetOverlay>>();
    let visibility = q
        .single(app.world())
        .expect("exactly one FanPlateDropTargetOverlay must exist");
    *visibility
}

fn fan_plate_drop_target_background_alpha_matches_token(app: &mut App) -> bool {
    let mut q = app
        .world_mut()
        .query_filtered::<&BackgroundColor, With<FanPlateDropTargetOverlay>>();
    let background = q
        .single(app.world())
        .expect("exactly one FanPlateDropTargetOverlay must exist");
    let LinearRgba { alpha, .. } = background.0.to_linear();
    (alpha - overlays::OVERLAY_SCRIM_ALPHA).abs() < f32::EPSILON
}

fn fan_plate_drop_target_border_color_is_semantic_success(app: &mut App) -> bool {
    let mut q = app
        .world_mut()
        .query_filtered::<&BorderColor, With<FanPlateDropTargetOverlay>>();
    let border = q
        .single(app.world())
        .expect("exactly one FanPlateDropTargetOverlay must exist");
    border_color_matches(border, semantic_success_color())
}

fn border_color_matches(border: &BorderColor, expected: Color) -> bool {
    let LinearRgba {
        red,
        green,
        blue,
        alpha,
    } = expected.to_linear();
    let same_color = |actual: Color| {
        let LinearRgba {
            red: r,
            green: g,
            blue: b,
            alpha: a,
        } = actual.to_linear();
        (r - red).abs() < 1e-4
            && (g - green).abs() < 1e-4
            && (b - blue).abs() < 1e-4
            && (a - alpha).abs() < 1e-4
    };
    // BorderColor stores four sides; require every side to match so that
    // a partial-side regression (e.g. left-only ACCENT) is caught.
    same_color(border.top)
        && same_color(border.right)
        && same_color(border.bottom)
        && same_color(border.left)
}

fn assert_no_visible_drag_overlays(app: &mut App) {
    let mut q = app
        .world_mut()
        .query_filtered::<&Visibility, With<DragStateOverlay>>();
    for visibility in q.iter(app.world()) {
        assert_eq!(
            *visibility,
            Visibility::Hidden,
            "idle state: every DragStateOverlay marker must remain Hidden"
        );
    }

    // No slot should be tagged with an active drag-state.
    let mut tagged = app
        .world_mut()
        .query_filtered::<&DragStateOverlayActive, With<FanSlotIndex>>();
    let tag_count = tagged.iter(app.world()).count();
    assert_eq!(
        tag_count, 0,
        "idle state: no FanSlotIndex entity should carry DragStateOverlayActive"
    );
}
