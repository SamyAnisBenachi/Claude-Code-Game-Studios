//! Sprint 18 story 023 (`S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001`).
//!
//! Drives the idle playable-affordance overlay sync system through the
//! AC1/AC2/AC3/AC7/AC8/AC9 states and asserts the per-slot marker /
//! `Visibility` via ECS-query assertions. Read-only over
//! `ActivePlacementDrag`, `PendingPlacements`, `HandUiMode`,
//! `CurrentClientPhase`, `PlayerEconomyView`, `HandCardCatalog` (each
//! preconditions is driven via direct resource mutation per the story's
//! "drives state via direct resource insertion" AC11 guidance — no
//! `Pointer<*>` event synthesis, independent of R1).
//!
//! The new overlays are deliberately distinct from
//! `drag_state_visuals::DragStateOverlay`, so the existing Story 020 query
//! semantics (`Query<&FanSlotIndex, Without<DragStateOverlay>>`) remain
//! intact. This is verified end-to-end by `hand_ui_drag_state_visuals_test`
//! continuing to PASS alongside this test.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::drag_state_visuals::DragStateOverlay;
use client::ui::hand::{
    ActivePlacementDrag, FanSlotIndex, FanSlotPlayableAffordanceActive,
    FanSlotPlayableAffordanceOverlay, FanSlotPlayableAffordanceUnaffordableOverlay,
    HandCardCatalog, HandContents, HandFanLayoutConfig, HandFanViewport, HandSlotCard, HandUiMode,
    HandUiPlugin, PendingPlacements, PlacementTargetKind, HAND_FAN_SLOT_COUNT,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlacedCardSubmit, PlayTarget, RoundPhase};
use shared::session::PlayerId;

const VIEWPORT_W: f32 = 1280.0;
const VIEWPORT_H: f32 = 720.0;

// ── AC1: Affordable idle slot lights up as Playable ──────────────────────────

#[test]
fn affordable_idle_slot_marks_playable_and_shows_overlay() {
    let mut app = app_in_placement_with_mixed_cost_cards();
    app.update();

    let slot_0 = fan_slot(&mut app, 0);

    assert_eq!(
        app.world().get::<FanSlotPlayableAffordanceActive>(slot_0),
        Some(&FanSlotPlayableAffordanceActive::Playable),
        "slot 0 (cost=1, pool=5) must read as Playable while idle"
    );
    assert_eq!(
        playable_overlay_visibility_for(&mut app, slot_0),
        Visibility::Visible,
        "Playable overlay must be Visible on the affordable idle slot"
    );
    assert_eq!(
        unaffordable_overlay_visibility_for(&mut app, slot_0),
        Visibility::Hidden,
        "Unaffordable overlay must remain Hidden on a Playable slot (AC3 mutual exclusion)"
    );

    // The new overlay child must NOT carry `DragStateOverlay` (Story 020 AC2
    // reconciliation: the affordance markers are disjoint from the drag-state
    // markers by construction).
    assert!(
        playable_overlay_under_slot_lacks_drag_state_marker(&mut app, slot_0),
        "Playable overlay must NOT carry DragStateOverlay"
    );
}

// ── AC2: Unaffordable idle slot shows the dim cover ──────────────────────────

#[test]
fn unaffordable_idle_slot_marks_unaffordable_and_shows_dim_overlay() {
    let mut app = app_in_placement_with_mixed_cost_cards();
    app.update();

    // Slot 1 has cost=4 with pool=5 — it is initially affordable. Crash the
    // economy pool down to cost-1 so it flips Unaffordable.
    {
        let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
        economy.current_mana = 1;
        economy.reserve_mana = 1;
    }
    app.update();

    let slot_1 = fan_slot(&mut app, 1);
    assert_eq!(
        app.world().get::<FanSlotPlayableAffordanceActive>(slot_1),
        Some(&FanSlotPlayableAffordanceActive::Unaffordable),
        "slot 1 (cost=4, pool=1+1=2 < 4) must read as Unaffordable"
    );
    assert_eq!(
        unaffordable_overlay_visibility_for(&mut app, slot_1),
        Visibility::Visible,
        "Unaffordable overlay must be Visible on the unaffordable idle slot"
    );
    assert_eq!(
        playable_overlay_visibility_for(&mut app, slot_1),
        Visibility::Hidden,
        "Playable overlay must remain Hidden on an Unaffordable slot (AC3 mutual exclusion)"
    );
}

// ── AC6: Reactive update on PlayerEconomyView change ─────────────────────────

#[test]
fn affordance_flips_when_economy_changes() {
    let mut app = app_in_placement_with_mixed_cost_cards();
    app.update();

    let slot_1 = fan_slot(&mut app, 1);

    // Starting state: slot_1 (cost=4) is affordable with pool 5+5=10.
    assert_eq!(
        app.world().get::<FanSlotPlayableAffordanceActive>(slot_1),
        Some(&FanSlotPlayableAffordanceActive::Playable),
    );

    // Crash the pool → slot_1 must flip to Unaffordable next tick.
    {
        let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
        economy.current_mana = 0;
        economy.reserve_mana = 0;
    }
    app.update();
    assert_eq!(
        app.world().get::<FanSlotPlayableAffordanceActive>(slot_1),
        Some(&FanSlotPlayableAffordanceActive::Unaffordable),
        "slot 1 must flip to Unaffordable when pool drops below cost"
    );
    assert_eq!(
        unaffordable_overlay_visibility_for(&mut app, slot_1),
        Visibility::Visible,
    );
    assert_eq!(
        playable_overlay_visibility_for(&mut app, slot_1),
        Visibility::Hidden,
    );

    // Restore the pool → slot_1 must flip back to Playable next tick.
    {
        let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
        economy.current_mana = 4;
        economy.reserve_mana = 0;
    }
    app.update();
    assert_eq!(
        app.world().get::<FanSlotPlayableAffordanceActive>(slot_1),
        Some(&FanSlotPlayableAffordanceActive::Playable),
        "slot 1 must flip back to Playable when pool restores",
    );
    assert_eq!(
        playable_overlay_visibility_for(&mut app, slot_1),
        Visibility::Visible,
    );
    assert_eq!(
        unaffordable_overlay_visibility_for(&mut app, slot_1),
        Visibility::Hidden,
    );
}

// ── AC7: Drag in flight suppresses all idle affordance ───────────────────────

#[test]
fn active_drag_suppresses_idle_affordance() {
    let mut app = app_in_placement_with_mixed_cost_cards();
    app.update();

    let slot_0 = fan_slot(&mut app, 0);
    let card_id_0 = card_id_for_slot(&app, slot_0);

    // Confirm slot_0 is Playable before drag.
    assert_eq!(
        app.world().get::<FanSlotPlayableAffordanceActive>(slot_0),
        Some(&FanSlotPlayableAffordanceActive::Playable),
    );

    set_active_drag(
        &mut app,
        slot_0,
        card_id_0,
        PlayerId(1),
        PlacementTargetKind::Minion,
    );
    app.update();

    // AC7: every affordance marker is removed and every overlay is Hidden.
    assert_eq!(
        app.world().get::<FanSlotPlayableAffordanceActive>(slot_0),
        None,
        "FanSlotPlayableAffordanceActive must be removed during an active drag",
    );
    assert_no_visible_affordance_overlays(&mut app);
}

// ── AC9: Staged-card suppression ─────────────────────────────────────────────

#[test]
fn staged_card_suppresses_idle_affordance_on_its_slot() {
    let mut app = app_in_placement_with_mixed_cost_cards();
    app.update();

    let slot_0 = fan_slot(&mut app, 0);
    let card_id_0 = card_id_for_slot(&app, slot_0);

    // Stage slot_0's card. The slot should now have neither affordance
    // marker nor a visible affordance overlay (the staged ghost dim
    // treatment from Story 005 / 008 owns the slot's visual).
    stage_card(&mut app, card_id_0);
    app.update();

    assert_eq!(
        app.world().get::<FanSlotPlayableAffordanceActive>(slot_0),
        None,
        "staged card's slot must lose its FanSlotPlayableAffordanceActive marker",
    );
    assert_eq!(
        playable_overlay_visibility_for(&mut app, slot_0),
        Visibility::Hidden,
    );
    assert_eq!(
        unaffordable_overlay_visibility_for(&mut app, slot_0),
        Visibility::Hidden,
    );

    // The non-staged sibling slot is unaffected (still Playable).
    let slot_1 = fan_slot(&mut app, 1);
    assert_eq!(
        app.world().get::<FanSlotPlayableAffordanceActive>(slot_1),
        Some(&FanSlotPlayableAffordanceActive::Playable),
        "non-staged sibling slot must remain Playable",
    );
}

// ── AC8: Phase / mode gating ────────────────────────────────────────────────

#[test]
fn passive_locked_mode_suppresses_idle_affordance() {
    let mut app = app_in_placement_with_mixed_cost_cards();
    app.update();

    // `HandUiMode::PassiveLocked` must suppress the idle affordance even in
    // PLACEMENT phase (the drag-state Disabled treatment owns this mode).
    set_hand_ui_mode(&mut app, HandUiMode::PassiveLocked);
    app.update();

    assert_no_visible_affordance_overlays(&mut app);
    assert_no_affordance_markers(&mut app);
}

#[test]
fn non_placement_phase_suppresses_idle_affordance() {
    let mut app = app_in_placement_with_mixed_cost_cards();
    app.update();

    // Force the phase off Placement; affordance must clear regardless of mode.
    set_phase(&mut app, RoundPhase::DraftShop);
    app.update();

    assert_no_visible_affordance_overlays(&mut app);
    assert_no_affordance_markers(&mut app);
}

// ── AC10: Empty fan slots stay Hidden ────────────────────────────────────────

#[test]
fn empty_fan_slots_show_no_affordance_overlay() {
    let mut app = app_in_placement_with_mixed_cost_cards();
    app.update();

    // Only 2 cards in HandContents, but 10 pre-pooled slots. Every empty
    // slot must keep its overlays Hidden and have no Active marker.
    let mut empty_slot_count = 0;
    for index in 2u8..HAND_FAN_SLOT_COUNT as u8 {
        let slot = fan_slot(&mut app, index);
        assert!(
            app.world().get::<HandSlotCard>(slot).is_none(),
            "slot {} must be empty in pre-pool",
            index
        );
        assert_eq!(
            app.world().get::<FanSlotPlayableAffordanceActive>(slot),
            None,
            "empty slot {} must not carry a FanSlotPlayableAffordanceActive marker",
            index
        );
        assert_eq!(
            playable_overlay_visibility_for(&mut app, slot),
            Visibility::Hidden,
        );
        assert_eq!(
            unaffordable_overlay_visibility_for(&mut app, slot),
            Visibility::Hidden,
        );
        empty_slot_count += 1;
    }
    assert_eq!(empty_slot_count, 8, "exactly 8 empty pre-pool slots expected");
}

// ── AC14: pre-pool count preserved across transitions ────────────────────────

#[test]
fn fan_slot_index_count_is_preserved_across_affordance_transitions() {
    let mut app = app_in_placement_with_mixed_cost_cards();
    app.update();

    let baseline = count::<FanSlotIndex>(&mut app);
    assert_eq!(baseline, HAND_FAN_SLOT_COUNT);

    let slot_0 = fan_slot(&mut app, 0);
    let card_id_0 = card_id_for_slot(&app, slot_0);

    // Playable → drag-suppressed → Unaffordable cycle.
    set_active_drag(
        &mut app,
        slot_0,
        card_id_0,
        PlayerId(1),
        PlacementTargetKind::Minion,
    );
    app.update();
    assert_eq!(count::<FanSlotIndex>(&mut app), baseline);

    clear_active_drag(&mut app);
    {
        let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
        economy.current_mana = 0;
        economy.reserve_mana = 0;
    }
    app.update();
    assert_eq!(count::<FanSlotIndex>(&mut app), baseline);

    // Final tally: no FanSlotIndex entity was spawned / despawned by the
    // affordance sync flips.
    assert_eq!(count::<FanSlotIndex>(&mut app), HAND_FAN_SLOT_COUNT);
}

// ── AC13: read-only over PlayerEconomyView / catalog / drag / placements ────

#[test]
fn sync_system_never_mutates_economy_or_drag_state() {
    let mut app = app_in_placement_with_mixed_cost_cards();

    let economy_before = app.world().resource::<PlayerEconomyView>().clone();
    let drag_before = *app.world().resource::<ActivePlacementDrag>();

    for _ in 0..5 {
        app.update();
        assert_eq!(
            *app.world().resource::<PlayerEconomyView>(),
            economy_before,
            "sync system must never mutate PlayerEconomyView",
        );
        assert_eq!(
            *app.world().resource::<ActivePlacementDrag>(),
            drag_before,
            "sync system must never mutate ActivePlacementDrag",
        );
    }
}

// ── Test app setup ───────────────────────────────────────────────────────────

fn app_in_placement_with_mixed_cost_cards() -> App {
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
        cards: test_catalog([(CardId(50), 1u32), (CardId(51), 4u32)]),
    });

    // Pool 5 + 5 = 10. Slot 0 (cost 1) and Slot 1 (cost 4) are both
    // affordable from the start.
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

fn test_catalog<const N: usize>(rows: [(CardId, u32); N]) -> HashMap<CardId, CardData> {
    rows.into_iter()
        .map(|(id, cost)| (id, test_minion_card(id, cost)))
        .collect()
}

fn test_minion_card(id: CardId, cost: u32) -> CardData {
    CardData {
        id,
        name_fr: format!("Carte {}", id.0),
        name_en: format!("Card {}", id.0),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost,
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
        .get::<HandSlotCard>(slot)
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

fn playable_overlay_visibility_for(app: &mut App, slot: Entity) -> Visibility {
    let mut q = app
        .world_mut()
        .query_filtered::<(&ChildOf, &Visibility), With<FanSlotPlayableAffordanceOverlay>>();
    for (child_of, visibility) in q.iter(app.world()) {
        if child_of.parent() == slot {
            return *visibility;
        }
    }
    panic!("no FanSlotPlayableAffordanceOverlay child found under fan slot {slot:?}");
}

fn unaffordable_overlay_visibility_for(app: &mut App, slot: Entity) -> Visibility {
    let mut q = app
        .world_mut()
        .query_filtered::<(&ChildOf, &Visibility), With<FanSlotPlayableAffordanceUnaffordableOverlay>>();
    for (child_of, visibility) in q.iter(app.world()) {
        if child_of.parent() == slot {
            return *visibility;
        }
    }
    panic!("no FanSlotPlayableAffordanceUnaffordableOverlay child found under fan slot {slot:?}");
}

fn playable_overlay_under_slot_lacks_drag_state_marker(app: &mut App, slot: Entity) -> bool {
    let mut q = app
        .world_mut()
        .query_filtered::<(Entity, &ChildOf), With<FanSlotPlayableAffordanceOverlay>>();
    let mut found = false;
    for (overlay_entity, child_of) in q.iter(app.world()) {
        if child_of.parent() == slot {
            found = true;
            if app.world().get::<DragStateOverlay>(overlay_entity).is_some() {
                return false;
            }
        }
    }
    found
}

fn assert_no_visible_affordance_overlays(app: &mut App) {
    {
        let mut q = app
            .world_mut()
            .query_filtered::<&Visibility, With<FanSlotPlayableAffordanceOverlay>>();
        for visibility in q.iter(app.world()) {
            assert_eq!(
                *visibility,
                Visibility::Hidden,
                "every Playable overlay must be Hidden in suppressed state",
            );
        }
    }
    let mut q = app
        .world_mut()
        .query_filtered::<&Visibility, With<FanSlotPlayableAffordanceUnaffordableOverlay>>();
    for visibility in q.iter(app.world()) {
        assert_eq!(
            *visibility,
            Visibility::Hidden,
            "every Unaffordable overlay must be Hidden in suppressed state",
        );
    }
}

fn assert_no_affordance_markers(app: &mut App) {
    let mut q = app
        .world_mut()
        .query::<&FanSlotPlayableAffordanceActive>();
    let count = q.iter(app.world()).count();
    assert_eq!(
        count, 0,
        "no slot should carry a FanSlotPlayableAffordanceActive marker in suppressed state",
    );
}
