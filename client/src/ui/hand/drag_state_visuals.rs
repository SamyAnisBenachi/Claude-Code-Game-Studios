//! Hand-card drag-state visual differentiation — Sprint 15 story 020
//! (`S12-UX-HAND-DRAG-STATE-VISUALS-001`).
//!
//! Adds per-state overlay child nodes under every pre-pooled hand fan-slot
//! entity (and the `HandFanRoot` fan-plate region) so the five drag states
//! (`Idle`, `Hover`, `Drag` source, `DropTarget` fan-plate, `Disabled`) render
//! distinct visual treatments composed from the Sprint 14 Tier 0 token set:
//!
//! - z-layer: [`z_layers::UI_OVERLAY`] (`HandDragSprite` ascends here while
//!   the drag is in flight; already set in `spawn_hand_ui`).
//! - dim overlay alpha: [`overlays::OVERLAY_DIM_ALPHA`] (`0.45`) for the
//!   drag-source slot and for `Disabled` slots.
//! - scrim overlay alpha: [`overlays::OVERLAY_SCRIM_ALPHA`] (`0.55`) for the
//!   fan-plate Instant drop-target region.
//! - §7 `ACCENT` colour for the `Hover` border outline — imported from the
//!   Sprint 14 spec ratified consumer site in `shop_auction` per story 020
//!   "Likely Files" guidance ("imports them from wherever the global UI
//!   design spec consumers landed them in Sprint 14"). No new colour tokens
//!   are authored by this story.
//! - §7 `SEMANTIC_SUCCESS` colour reserved for the fan-plate drop-target
//!   border affordance (worker discretion under AC4; the dim scrim overlay
//!   is the primary cue).
//!
//! All overlays are **child nodes** of existing pre-pooled fan-slot entities
//! (or of the existing `HandFanRoot`). They are not new top-level pre-pool
//! entries: ADR-021 Impl Guideline 5 is preserved
//! ([`HAND_UI_ENTITY_COUNT`] is unchanged). The sync system reads from
//! [`ActivePlacementDrag`], [`HandUiMode`], [`PendingPlacements`], and
//! [`PlayerEconomyView`] **read-only** — no mutation, no new server-
//! authoritative state, no new Lightyear message. ADR-002 + ADR-012
//! binding preserved.
//!
//! [`HAND_UI_ENTITY_COUNT`]: super::HAND_UI_ENTITY_COUNT

use bevy::prelude::*;

use shared::card::CardType;

use crate::presentation::PlayerEconomyView;
use crate::ui::design_tokens::{overlays, z_layers};
use crate::ui::shop_auction::{
    auction_featured_card_accent_color, auction_featured_card_leading_color,
};

use super::{
    ActivePlacementDrag, FanSlotIndex, HandCardCatalog, HandSlotCard, HandUiEntities, HandUiMode,
    PendingPlacements, PlacementTargetKind,
};

/// §7 `ACCENT` colour symbol — imported from the existing Sprint 14
/// consumer site rather than authored as a new design token (story 020
/// "Likely Files" guidance forbids new colour tokens).
///
/// Matches the spec-ratified `#F2C94C` accent value documented at
/// `docs/ux/global-ui-design-spec.md` §7 and exposed via
/// [`auction_featured_card_accent_color`] in Sprint 14.
pub fn accent_color() -> Color {
    auction_featured_card_accent_color()
}

/// §7 `SEMANTIC_SUCCESS` colour symbol — imported from the existing
/// Sprint 14 consumer site (`auction_featured_card_leading_color()`
/// resolves to `#27AE60` per the global UI design spec §7).
pub fn semantic_success_color() -> Color {
    auction_featured_card_leading_color()
}

/// Translucent black tint whose alpha channel is sourced from
/// [`overlays::OVERLAY_DIM_ALPHA`] (`0.45`). Used for the drag-source
/// fan-slot dim treatment AND for the `Disabled` fan-slot treatment.
pub fn dim_overlay_color() -> Color {
    Color::srgba(0.0, 0.0, 0.0, overlays::OVERLAY_DIM_ALPHA)
}

/// Translucent neutral tint whose alpha channel is sourced from
/// [`overlays::OVERLAY_SCRIM_ALPHA`] (`0.55`). Used for the fan-plate
/// Instant drop-target affordance.
pub fn scrim_overlay_color() -> Color {
    Color::srgba(0.0, 0.0, 0.0, overlays::OVERLAY_SCRIM_ALPHA)
}

/// Marker on every drag-state overlay child node — used to query the
/// overlay set in tests and to assert AC2 baseline ("an idle slot has no
/// **visible** overlay" — overlays are present but `Visibility::Hidden`).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragStateOverlay;

/// Marker on the per-slot full-cover dim overlay (drag-source AND
/// disabled treatments use the same overlay; the sync system toggles
/// visibility based on which trigger is active and tags the overlay with
/// [`DragStateOverlayActive`] for AC3 / AC5 assertions).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanSlotDimOverlay;

/// Marker on the per-slot full-cover hover border overlay. The overlay
/// uses [`BorderColor`] sourced from [`accent_color`] (§7 `ACCENT`).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanSlotHoverOverlay;

/// Marker on the fan-plate Instant drop-target scrim overlay. The
/// overlay uses [`BackgroundColor`] sourced from [`scrim_overlay_color`]
/// (§6 [`OVERLAY_SCRIM_ALPHA`]).
///
/// [`OVERLAY_SCRIM_ALPHA`]: overlays::OVERLAY_SCRIM_ALPHA
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanPlateDropTargetOverlay;

/// Tagged onto whichever drag-state overlay is currently the active
/// treatment so AC9 ECS-query assertions can read a single component.
/// Removed when no drag state is active for the slot.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragStateOverlayActive {
    Hover,
    DragSource,
    Disabled,
    DropTarget,
}

/// Layout for the per-slot dim overlay (full-cover, sits on top of all
/// chrome children).
fn dim_overlay_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(0.0),
        top: Val::Percent(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

/// Layout for the per-slot hover border overlay (full-cover with a 2 px
/// border so the [`BorderColor`] paints around the slot when active).
fn hover_overlay_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(0.0),
        top: Val::Percent(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    }
}

/// Layout for the fan-plate Instant drop-target overlay (full-cover of
/// the fan-plate region).
fn drop_target_overlay_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(0.0),
        top: Val::Percent(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

/// Spawn the drag-state overlay child nodes for a single pre-pooled fan
/// slot. Called once per slot from `spawn_hand_ui` after the chrome
/// children are spawned so the overlays render above the chrome.
///
/// Both overlays start `Visibility::Hidden`; the sync system flips them
/// `Visible` when the appropriate drag state applies.
pub fn spawn_fan_slot_drag_state_overlays(commands: &mut Commands, slot: Entity, slot_index: u8) {
    commands.spawn((
        Name::new(format!("Fan Slot {slot_index} Drag-State Dim Overlay")),
        super::HandUiEntity,
        DragStateOverlay,
        FanSlotDimOverlay,
        dim_overlay_node(),
        BackgroundColor(dim_overlay_color()),
        Visibility::Hidden,
        ChildOf(slot),
    ));

    commands.spawn((
        Name::new(format!("Fan Slot {slot_index} Drag-State Hover Overlay")),
        super::HandUiEntity,
        DragStateOverlay,
        FanSlotHoverOverlay,
        hover_overlay_node(),
        BorderColor::all(accent_color()),
        Visibility::Hidden,
        ChildOf(slot),
    ));
}

/// Spawn the fan-plate Instant drop-target overlay as a child of the
/// `HandFanRoot` entity. Called once from `spawn_hand_ui`.
pub fn spawn_fan_plate_drop_target_overlay(commands: &mut Commands, fan_root: Entity) {
    commands.spawn((
        Name::new("Hand UI Fan Plate Drag-State DropTarget Overlay"),
        super::HandUiEntity,
        DragStateOverlay,
        FanPlateDropTargetOverlay,
        drop_target_overlay_node(),
        BackgroundColor(scrim_overlay_color()),
        BorderColor::all(semantic_success_color()),
        Visibility::Hidden,
        ChildOf(fan_root),
        z_layers::UI_OVERLAY,
    ));
}

/// Sync per-slot drag-state visual overlays from the already-extant
/// ephemeral drag state. Read-only over [`ActivePlacementDrag`],
/// [`HandUiMode`], [`PendingPlacements`], and [`PlayerEconomyView`].
/// ADR-002 + ADR-012 binding preserved (no mutation of the drag
/// resource; no new server-authoritative state).
pub fn sync_hand_drag_state_visuals_system(
    mode: Res<HandUiMode>,
    active_drag: Res<ActivePlacementDrag>,
    pending_placements: Res<PendingPlacements>,
    economy: Res<PlayerEconomyView>,
    catalog: Res<HandCardCatalog>,
    entities: Option<Res<HandUiEntities>>,
    slots: Query<(Entity, &FanSlotIndex, Option<&HandSlotCard>, &Interaction)>,
    mut dim_overlays: Query<(&ChildOf, &mut Visibility), With<FanSlotDimOverlay>>,
    mut hover_overlays: Query<
        (&ChildOf, &mut Visibility),
        (With<FanSlotHoverOverlay>, Without<FanSlotDimOverlay>),
    >,
    mut drop_target_overlay: Query<
        &mut Visibility,
        (
            With<FanPlateDropTargetOverlay>,
            Without<FanSlotDimOverlay>,
            Without<FanSlotHoverOverlay>,
        ),
    >,
    mut commands: Commands,
) {
    let drag_active = active_drag.is_active();
    let drag_source_entity = if drag_active { active_drag.card } else { None };
    let drop_target_active =
        drag_active && active_drag.target_kind == Some(PlacementTargetKind::Instant);

    // ── Fan-plate Instant drop-target overlay ───────────────────────────────
    for mut visibility in &mut drop_target_overlay {
        *visibility = if drop_target_active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // ── Per-slot dim / disabled / hover overlays ────────────────────────────
    let staged_ids: Vec<_> = pending_placements
        .placements
        .iter()
        .map(|p| p.card_id)
        .collect();

    // Resolve the active state per slot once, then patch each overlay's
    // visibility from the same resolved state — keeps drag-source and
    // disabled treatments mutually-exclusive on a given frame.
    let mut slot_states: std::collections::HashMap<Entity, DragStateOverlayActive> =
        std::collections::HashMap::new();

    let global_disabled_mode = matches!(*mode, HandUiMode::PassiveLocked);

    for (slot_entity, _slot_index, slot_card, interaction) in slots.iter() {
        // 1) Drag source dim — only when this slot is the drag source.
        if Some(slot_entity) == drag_source_entity {
            slot_states.insert(slot_entity, DragStateOverlayActive::DragSource);
            continue;
        }

        // 2) Disabled treatment.
        let disabled = global_disabled_mode
            || slot_card.is_some_and(|card| {
                staged_ids.contains(&card.0) || !slot_is_affordable(card.0, &catalog, &economy)
            });
        if disabled {
            slot_states.insert(slot_entity, DragStateOverlayActive::Disabled);
            continue;
        }

        // 3) Hover treatment — only when no drag is in flight and the
        //    pointer is over a non-disabled slot.
        if !drag_active
            && *interaction == Interaction::Hovered
            && matches!(*mode, HandUiMode::Passive | HandUiMode::Staging)
            && slot_card.is_some()
        {
            slot_states.insert(slot_entity, DragStateOverlayActive::Hover);
        }
    }

    // ── Patch dim overlays ───────────────────────────────────────────────────
    for (child_of, mut visibility) in &mut dim_overlays {
        let parent = child_of.parent();
        let active = matches!(
            slot_states.get(&parent),
            Some(DragStateOverlayActive::DragSource) | Some(DragStateOverlayActive::Disabled),
        );
        *visibility = if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // ── Patch hover overlays ────────────────────────────────────────────────
    for (child_of, mut visibility) in &mut hover_overlays {
        let parent = child_of.parent();
        let active = matches!(
            slot_states.get(&parent),
            Some(DragStateOverlayActive::Hover)
        );
        *visibility = if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // ── Tag the per-slot active state for AC9 ECS-query assertions ──────────
    for (slot_entity, _, _, _) in slots.iter() {
        match slot_states.get(&slot_entity) {
            Some(state) => {
                commands.entity(slot_entity).insert(*state);
            }
            None => {
                commands
                    .entity(slot_entity)
                    .remove::<DragStateOverlayActive>();
            }
        }
    }

    // Mirror the fan-plate state for AC4 ECS-query assertions.
    if let Some(entities) = entities {
        if drop_target_active {
            commands
                .entity(entities.fan_root)
                .insert(DragStateOverlayActive::DropTarget);
        } else {
            commands
                .entity(entities.fan_root)
                .remove::<DragStateOverlayActive>();
        }
    }
}

/// Returns true iff the player's `current + reserve` mana suffices for the
/// card's mana cost. Cards missing from the catalog default to affordable
/// (mirrors the conservative pre-validation pattern used by the existing
/// reserve-strip + submit pre-validation systems — see `apply_submit_validation`
/// and `sync_reserve_strip_state_system`).
fn slot_is_affordable(
    card_id: shared::card::CardId,
    catalog: &HandCardCatalog,
    economy: &PlayerEconomyView,
) -> bool {
    let Some(card) = catalog.cards.get(&card_id) else {
        return true;
    };
    // Instants / passives bypass mana cost in this affordability check; the
    // disabled overlay is reserved for minions / placeables whose mana cost
    // exceeds the player's combined pool.
    if card.card_type != CardType::Minion {
        return true;
    }
    let available = economy.current_mana.saturating_add(economy.reserve_mana);
    available >= card.cost
}
