//! `PlayArea` — flex container for the middle band of the in-session
//! viewport (Sprint 18 story 020 / `S18-UI-PLAY-AREA-CONTAINER-001`).
//!
//! `PlayArea` is the canonical parent for every panel that paints between
//! the HUD top strip (`HeaderBar`, 60 px) and the bottom strip column
//! (`FooterBar` + `HandBar`, 40 + 180 = 220 px). It enforces the
//! "strip-budget contract" introduced by PROMPT 1180 §6 Lane A: each
//! in-session panel parents into `PlayArea`, the strip primitives stay
//! viewport-edge-anchored as siblings, and no consumer hand-computes
//! `(top, bottom)` offsets against the viewport edges any more.
//!
//! ## Geometry
//!
//! `play_area_node()` produces a `PositionType::Absolute` rectangle
//! anchored relative to the viewport edges:
//!
//! - `top = HEADER_BAR_HEIGHT_PX`
//! - `bottom = HAND_BAR_HEIGHT_PX + FOOTER_BAR_HEIGHT_PX`
//! - `left = 0`, `right = 0`
//! - `display = Flex`, `flex_direction = Column`
//!
//! The strip-height constants are re-used verbatim from
//! [`crate::ui::design_tokens::strips`]; no new pixel literals are
//! introduced here.
//!
//! ## Relationship to strip primitives
//!
//! `PlayArea` is a **sibling** of `HeaderBar` / `FooterBar` / `HandBar`,
//! not a child or parent. Strip primitives keep their existing
//! viewport-edge anchors and their existing consumers (`spawn_hand_ui`
//! spawns `HandBar`; HUD spawns `HeaderBar` / `FooterBar`); `PlayArea`
//! fills the residual middle band that the four canonical strips leave
//! uncovered. This composition is the precondition for Lane J (story 026)
//! overlay-panel overflow hardening which consumes `PlayArea` as a
//! viewport-safe content-budget reference.
//!
//! ## Lifecycle
//!
//! - Spawned on `OnEnter(ClientState::InSession)` by `spawn_play_area`.
//! - Despawned on `OnExit(ClientState::InSession)` by `despawn_play_area`.
//! - The spawned entity is recorded in the [`PlayAreaRoot`] resource so
//!   consumer plugins (`HandUiPlugin`, `ShopAuctionUiPlugin`) can parent
//!   their panels into it without re-deriving the entity from a query.
//!
//! ## System ordering
//!
//! `spawn_play_area` runs in the [`PlayAreaSpawnSet`] `SystemSet` on
//! `OnEnter(ClientState::InSession)`. Consumer spawn systems register
//! `.after(PlayAreaSpawnSet)` so the `PlayAreaRoot` resource is available
//! when they query for it. Consumer spawn systems also fall back to
//! their previous parent (`fan_root` / `ShopAuctionUiRoot`) when
//! `PlayAreaRoot` is absent — this preserves the existing harness apps
//! under `client/src/*_harness.rs` that build a minimal `App` with only
//! `HandUiPlugin` / `ShopAuctionUiPlugin` and no `PlayAreaPlugin`.
//!
//! ## Scope discipline
//!
//! Layout-only. Does not advance Standard-tier accessibility
//! (`QA-COND-0005`), playtest validation (`QA-COND-0006`), or
//! final-art / asset-production (`PAW-TD-*-a`). No system-set / authority
//! change (ADR-021 / ADR-002).

use bevy::prelude::*;

use crate::state::ClientState;
use crate::ui::design_tokens::strips::{
    FOOTER_BAR_HEIGHT_PX, HAND_BAR_HEIGHT_PX, HEADER_BAR_HEIGHT_PX,
};

/// Marker component for the canonical `PlayArea` flex container.
/// Spawned at most once per in-session UI tree.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayArea;

/// Resource holding the live `PlayArea` entity, populated by
/// [`spawn_play_area`] on `OnEnter(ClientState::InSession)` and removed
/// by [`despawn_play_area`] on `OnExit`. Consumer spawn systems read
/// this resource to parent their panels into the `PlayArea` middle band
/// rather than the full-viewport `ShopAuctionUiRoot` / `fan_root`.
#[derive(Resource, Debug, Clone, Copy)]
pub struct PlayAreaRoot(pub Entity);

/// `SystemSet` for the `PlayArea` spawn step on
/// `OnEnter(ClientState::InSession)`. Consumer spawn systems register
/// `.after(PlayAreaSpawnSet)` so the [`PlayAreaRoot`] resource is
/// available when they query for it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayAreaSpawnSet;

/// Combined vertical pixel budget reserved by the bottom-of-viewport
/// strip column (`HandBar` + `FooterBar`). Re-exported for the
/// integration test bin so the budget contract has a single source of
/// truth.
pub const PLAY_AREA_BOTTOM_RESERVE_PX: f32 = HAND_BAR_HEIGHT_PX + FOOTER_BAR_HEIGHT_PX;

/// Build the canonical `PlayArea` node — a `PositionType::Absolute`
/// flex column container anchored between the `HeaderBar` strip and the
/// `HandBar` + `FooterBar` strip column.
///
/// Geometry is fully derived from the strip-height constants; no
/// per-viewport literal is introduced here. The `Display::Flex` +
/// `flex_direction: Column` axes are intentional so consumer panels can
/// either parent into `PlayArea` as relative-flow flex children or as
/// `position_type: Absolute` children anchored to `PlayArea`'s edges
/// (the latter pattern is used by the shop / auction / footer / toast
/// surfaces because they are mutually-exclusively-visible per game
/// phase and want to fill the middle band rather than stack).
pub fn play_area_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: Val::Px(HEADER_BAR_HEIGHT_PX),
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        bottom: Val::Px(PLAY_AREA_BOTTOM_RESERVE_PX),
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

/// Spawn the single `PlayArea` entity on `OnEnter(ClientState::InSession)`.
/// Idempotent: if [`PlayAreaRoot`] is already present (e.g. a duplicate
/// transition fired by a test harness) the system is a no-op.
pub fn spawn_play_area(mut commands: Commands, existing: Option<Res<PlayAreaRoot>>) {
    if existing.is_some() {
        return;
    }
    let entity = commands
        .spawn((Name::new("PlayArea"), PlayArea, play_area_node()))
        .id();
    commands.insert_resource(PlayAreaRoot(entity));
}

/// Despawn the `PlayArea` entity and clear the [`PlayAreaRoot`] resource
/// on `OnExit(ClientState::InSession)`.
pub fn despawn_play_area(mut commands: Commands, existing: Option<Res<PlayAreaRoot>>) {
    let Some(root) = existing else {
        return;
    };
    commands.entity(root.0).despawn();
    commands.remove_resource::<PlayAreaRoot>();
}

/// Plugin that registers the `PlayArea` spawn / despawn systems on the
/// session lifecycle. Register **before** `HandUiPlugin` and
/// `ShopAuctionUiPlugin` so [`PlayAreaSpawnSet`] is the canonical
/// ordering anchor; consumer plugins chain their spawn systems with
/// `.after(PlayAreaSpawnSet)`.
pub struct PlayAreaPlugin;

impl Plugin for PlayAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ClientState::InSession),
            spawn_play_area.in_set(PlayAreaSpawnSet),
        )
        .add_systems(OnExit(ClientState::InSession), despawn_play_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ac1_play_area_node_is_absolute_with_documented_offsets() {
        let node = play_area_node();
        assert_eq!(node.position_type, PositionType::Absolute);
        assert_eq!(node.top, Val::Px(HEADER_BAR_HEIGHT_PX));
        assert_eq!(node.left, Val::Px(0.0));
        assert_eq!(node.right, Val::Px(0.0));
        assert_eq!(node.bottom, Val::Px(PLAY_AREA_BOTTOM_RESERVE_PX));
    }

    #[test]
    fn ac1_play_area_node_is_flex_column() {
        let node = play_area_node();
        assert_eq!(node.display, Display::Flex);
        assert_eq!(node.flex_direction, FlexDirection::Column);
    }

    #[test]
    fn ac1_bottom_reserve_matches_strip_constants() {
        assert_eq!(
            PLAY_AREA_BOTTOM_RESERVE_PX,
            HAND_BAR_HEIGHT_PX + FOOTER_BAR_HEIGHT_PX
        );
    }
}
