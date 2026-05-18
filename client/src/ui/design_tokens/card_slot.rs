//! Named card-slot primitive — Sprint 16 Tier 3 rank 13 row (story 009 /
//! `S12-TD-UI-CARD-SLOT-PRIMITIVE-001`).
//!
//! Every surface that paints a card in the playable client (hand fan,
//! draft initial grid, shop slot well, auction featured card, board
//! staged-ghost preview) requests its outer-rectangle layout from the
//! [`card_slot_geometry`] catalog and its [`bevy::ui::Node`] from the
//! [`card_slot_node`] builder instead of authoring per-site
//! width / height / aspect-ratio / image-bounds / text-bounds / hit-target
//! / z-layer literals. PROMPT 802 §3.3 HA1 + §3.3 HA5 surfaced that the
//! client had **no canonical card-slot primitive**: hand, draft, shop, and
//! auction each authored their own slot Node, drifting layout across
//! surfaces. This module is the single source of truth that the Sprint 16+
//! per-surface migration siblings (`S16-UI-CARD-SLOT-MIGRATION-*`) consume.
//!
//! ## Catalog
//!
//! Five canonical [`CardSlotKind`] variants — one per in-game card surface.
//! Each carries a [`CardSlotGeometry`] entry with named numeric constants
//! for the outer rectangle, aspect-ratio band, image / text / hit-target
//! insets, and z-layer reference. No inline `Val::Px(N)` numeric literal
//! is authored inside the module body — every value flows from a named
//! `const` so the canonical defaults can be edited in one place.
//!
//! | Kind | Outer (px) | Z-layer | Canonical consumer |
//! |------|-----------|---------|--------------------|
//! | [`CardSlotKind::HandFan`]          |  96 × 136 portrait  | [`z_layers::UI_BASE`]    | `client/src/ui/hand/mod.rs` hand fan (`HAND_CARD_DISPLAY_*`). |
//! | [`CardSlotKind::DraftGrid`]        | 120 ×  56 landscape | [`z_layers::UI_BASE`]    | `client/src/ui/hand/mod.rs` draft initial grid (`HAND_DRAFT_GRID_CARD_*`). |
//! | [`CardSlotKind::ShopSlot`]         | 136 ×  78 landscape | [`z_layers::UI_BASE`]    | `client/src/ui/shop_auction/mod.rs::shop_slot_node` (Phase 1 migration). |
//! | [`CardSlotKind::AuctionFeatured`]  | 380 × 280 landscape | [`z_layers::UI_BASE`]    | `client/src/ui/shop_auction/mod.rs::auction_featured_card_node` (`AUCTION_FEATURED_CARD_*`). |
//! | [`CardSlotKind::BoardStagedGhost`] |  64 ×  80 portrait  | [`z_layers::UI_OVERLAY`] | World-space ghost preview (sized to one board cell per `docs/ux/board-rendering-spec.md` BR-001 `cell_to_world`). |
//!
//! Per-kind geometry values are read verbatim from the per-surface
//! literals shipped on `origin/main` at story-authoring time so Phase 1
//! migration introduces **no visual regression**. The Sprint 16 producer
//! MAY re-tune the values in a separate decision; this primitive defaults
//! to "preserve current per-surface values".
//!
//! ## Interaction-state composition
//!
//! Card-slot kinds compose with the named hover / focus / pressed /
//! disabled tokens published by
//! [`crate::ui::design_tokens::interaction_states`] (Sprint 15 story 008
//! DONE). The doc comment on each [`CardSlotKind`] variant names the four
//! primitive families it consumes; per-surface migration of the actual
//! interaction-state visuals is owned by the Sprint 16+
//! `S16-UI-INTERACTION-STATE-MIGRATION-*` family, NOT this primitive.
//!
//! ## Spec cross-reference
//!
//! Canonical numeric values are ratified by
//! `docs/ux/global-ui-design-spec.md` §12 "Card Slot Primitive" (Sprint 16
//! amendment, this story). The spec is the source-of-truth for the
//! defaults; this module is the source-of-truth for the implementation.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: card-slot geometry is a
//!   read-only presentation primitive. No
//!   [`lightyear::prelude::MessageReceiver`] drain is introduced; no
//!   schedule ordering shifts.
//! - **ADR-002 Client-Server Authority**: card-slot geometry never carries
//!   game state. Layout values are friend-game presentation primitives.
//!
//! ## Scope (Sprint 16 story 009)
//!
//! - **Friend-game scope boundary preserved.** `QA-COND-0005` Standard-tier
//!   accessibility (≥44px hit-targets; WCAG contrast on slot chrome; full
//!   keyboard-navigation focus order; screen-reader hints; colorblind
//!   modes; text scaling), `QA-COND-0006` playtest validation, and
//!   `PAW-TD-*-a` placeholder-art accept-risk are **not** advanced by this
//!   module. The hit-target API returns the *current* hit rectangle per
//!   kind; it does not enforce a Standard-tier floor.
//! - **Layout primitive only.** This module composes the *layout*; it does
//!   **not** replace placeholder art (`PAW-TD-002-a` / `PAW-TD-003-a`),
//!   alter game-state machines, or introduce final-art chrome.
//! - **No nested cards.** A card slot is leaf-only — it has image and
//!   text regions, NOT a child card slot. The Node builder for kind `K`
//!   never instantiates [`card_slot_node`] for any other kind `K'`.
//! - **Phase 1 migration only by this row.** The Sprint 16 default scope
//!   is primitive + spec + shop slot Phase 1; the remaining three surfaces
//!   (`HandFan` + `DraftGrid` hand surfaces, `AuctionFeatured` auction
//!   surface, `BoardStagedGhost` board surface) are migrated by the
//!   Sprint 16+ `S16-UI-CARD-SLOT-MIGRATION-*` follow-on family.

use bevy::prelude::default;
use bevy::ui::{Display, FlexDirection, GlobalZIndex, Node, PositionType, UiRect, Val};

use crate::ui::design_tokens::z_layers;

// ---------------------------------------------------------------------------
// CardSlotKind — canonical card surfaces in the playable client
// ---------------------------------------------------------------------------

/// Canonical card surface in the playable client.
///
/// Each variant maps to one in-game site where a card is painted. The
/// per-variant [`CardSlotGeometry`] catalog ([`card_slot_geometry`]) is the
/// single source of truth for the slot's outer rectangle, aspect-ratio
/// band, image / text / hit-target insets, and z-layer.
///
/// See [`card_slot_node`] for the [`bevy::ui::Node`] builder consumed by
/// surfaces that compose a card slot inside a [`bevy::ui::UiRoot`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CardSlotKind {
    /// Hand fan card display — `96 × 136 px` portrait. Canonical consumer
    /// is `client/src/ui/hand/mod.rs` (`HAND_CARD_DISPLAY_WIDTH_PX` /
    /// `HAND_CARD_DISPLAY_HEIGHT_PX`).
    ///
    /// Interaction-state composition: pointer hover layers
    /// [`crate::ui::design_tokens::interaction_states::HOVER_BG_TINT_ALPHA`]
    /// / [`crate::ui::design_tokens::interaction_states::HOVER_BORDER_ALPHA`];
    /// keyboard focus draws
    /// [`crate::ui::design_tokens::interaction_states::FOCUS_RING_COLOR`]
    /// at
    /// [`crate::ui::design_tokens::interaction_states::FOCUS_RING_WIDTH_PX`]
    /// /
    /// [`crate::ui::design_tokens::interaction_states::FOCUS_RING_OFFSET_PX`];
    /// pressed (drag pickup) applies
    /// [`crate::ui::design_tokens::interaction_states::PRESSED_BG_TINT_ALPHA`];
    /// disabled (no legal target / hand-full) flattens via
    /// [`crate::ui::design_tokens::interaction_states::DISABLED_BG_TINT_ALPHA`]
    /// /
    /// [`crate::ui::design_tokens::interaction_states::DISABLED_TEXT_ALPHA`]
    /// /
    /// [`crate::ui::design_tokens::interaction_states::DISABLED_BORDER_ALPHA`].
    /// Per-surface migration of the actual interaction visuals is owned by
    /// the Sprint 16+ `S16-UI-INTERACTION-STATE-MIGRATION-*` family.
    HandFan,

    /// Draft initial grid card — `120 × 56 px` landscape. Canonical
    /// consumer is the draft initial grid pane in
    /// `client/src/ui/hand/mod.rs`
    /// (`HAND_DRAFT_GRID_CARD_WIDTH_PX` / `HAND_DRAFT_GRID_CARD_HEIGHT_PX`).
    ///
    /// Interaction-state composition: hover / focus / pressed / disabled
    /// references identical to [`CardSlotKind::HandFan`] above; see that
    /// variant's doc comment for the canonical token family names.
    DraftGrid,

    /// Shop slot well — `136 × 78 px` landscape. Canonical consumer is
    /// `client/src/ui/shop_auction/mod.rs::shop_slot_node`. The Phase 1
    /// migration in this story routes the Node through
    /// [`card_slot_node`] for this kind.
    ///
    /// Interaction-state composition: hover layers
    /// [`crate::ui::design_tokens::interaction_states::HOVER_BG_TINT_ALPHA`]
    /// /
    /// [`crate::ui::design_tokens::interaction_states::HOVER_BORDER_ALPHA`];
    /// keyboard focus draws
    /// [`crate::ui::design_tokens::interaction_states::FOCUS_RING_COLOR`]
    /// per
    /// [`crate::ui::design_tokens::interaction_states::FOCUS_RING_WIDTH_PX`]
    /// /
    /// [`crate::ui::design_tokens::interaction_states::FOCUS_RING_OFFSET_PX`];
    /// pressed (purchase click)
    /// uses
    /// [`crate::ui::design_tokens::interaction_states::PRESSED_BG_TINT_ALPHA`];
    /// the `cannot afford` disabled state uses
    /// [`crate::ui::design_tokens::interaction_states::DISABLED_BG_TINT_ALPHA`]
    /// /
    /// [`crate::ui::design_tokens::interaction_states::DISABLED_TEXT_ALPHA`]
    /// /
    /// [`crate::ui::design_tokens::interaction_states::DISABLED_BORDER_ALPHA`].
    ShopSlot,

    /// Auction featured card — `380 × 280 px` landscape. Canonical consumer
    /// is `client/src/ui/shop_auction/mod.rs::auction_featured_card_node`
    /// (`AUCTION_FEATURED_CARD_WIDTH_PX` /
    /// `AUCTION_FEATURED_CARD_HEIGHT_PX`). Migration of this call site is
    /// OWNED by Sprint 16+ `S16-UI-CARD-SLOT-MIGRATION-AUCTION-001` —
    /// **not by this story**; only the variant + geometry are declared
    /// here.
    ///
    /// Interaction-state composition: hover / focus / pressed / disabled
    /// references identical to [`CardSlotKind::ShopSlot`] above; auction
    /// `bid` button gating uses the same disabled family as the shop slot
    /// `cannot afford` state.
    AuctionFeatured,

    /// Board staged-ghost preview — `64 × 80 px` (one board cell at GDD
    /// default `cell_width = 64.0` / `lane_height = 80.0`; see
    /// `docs/ux/board-rendering-spec.md` §3 / BR-001). World-space sprite,
    /// **not** a [`bevy::ui::Node`] consumer; this variant exists in the
    /// catalog so the primitive is the single source of truth for
    /// card-slot geometry across both `bevy_ui` and world-space surfaces.
    /// Phase 4 (Sprint 16+
    /// `S16-UI-CARD-SLOT-MIGRATION-BOARD-GHOST-001`) is responsible for
    /// consuming the geometry; this story only declares it.
    ///
    /// Interaction-state composition: not applicable — the staged ghost
    /// is a preview-only world-space sprite without pointer / focus
    /// affordances. The four interaction-state token families remain
    /// importable for symmetry with the other variants.
    BoardStagedGhost,
}

/// Strictly-ordered catalog of every [`CardSlotKind`] variant. Exposed for
/// integration tests and any future audit tooling that needs to iterate
/// every kind in canonical order.
pub const ALL_CARD_SLOT_KINDS: [CardSlotKind; 5] = [
    CardSlotKind::HandFan,
    CardSlotKind::DraftGrid,
    CardSlotKind::ShopSlot,
    CardSlotKind::AuctionFeatured,
    CardSlotKind::BoardStagedGhost,
];

// ---------------------------------------------------------------------------
// CardSlotGeometry — per-kind layout contract
// ---------------------------------------------------------------------------

/// Per-kind layout contract for a [`CardSlotKind`].
///
/// Carries the outer visual rectangle, aspect-ratio band, image / text /
/// hit-target insets, border thickness, and named z-layer for the kind.
/// Returned by [`card_slot_geometry`].
///
/// Every numeric field is read from a named `const` declared at the head
/// of this module — no inline magic literals at the public-API boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardSlotGeometry {
    /// Outer visual rectangle width, in pixels.
    pub outer_width_px: f32,

    /// Outer visual rectangle height, in pixels.
    pub outer_height_px: f32,

    /// Allowed aspect-ratio band `(min, max)`. The kind's
    /// `outer_width_px / outer_height_px` MUST fall inside this band; the
    /// integration test asserts the relationship.
    pub aspect_ratio_band: (f32, f32),

    /// Border thickness applied to the outer rectangle. Friend-game
    /// chrome only; not an accessibility contract.
    pub border_thickness_px: f32,

    /// Inset between the outer rectangle and the card's art region. Read
    /// as a left / right / top / bottom [`UiRect`] in pixel units. The
    /// integration test asserts the image rectangle fits inside the
    /// outer rectangle.
    pub image_inset_px: UiRect,

    /// Inset between the outer rectangle and the card's text region
    /// (name + cost). Read as a left / right / top / bottom [`UiRect`]
    /// in pixel units. The integration test asserts the text rectangle
    /// fits inside the outer rectangle and does not overlap the image
    /// rectangle.
    pub text_inset_px: UiRect,

    /// Inset between the outer rectangle and the canonical hit-target
    /// rectangle. Default is [`UiRect::ZERO`] (hit target equals visual
    /// outer rectangle); a surface that needs a larger or offset hit
    /// target (e.g. focus ring outset) reads this directly.
    ///
    /// Per the AC7 contract the hit-target rectangle is a superset of (or
    /// equal to) the visual outer rectangle — hit target is never smaller
    /// than the visual rectangle.
    pub hit_target_inset_px: UiRect,

    /// Named z-layer reference from
    /// [`crate::ui::design_tokens::z_layers`]. `UI_BASE` for the four
    /// `bevy_ui` consumers; `UI_OVERLAY` for the world-space board
    /// staged-ghost preview while it is dragged from the hand.
    pub z_layer: GlobalZIndex,
}

// ---------------------------------------------------------------------------
// HandFan — 96 × 136 px portrait
// ---------------------------------------------------------------------------

/// Hand fan card outer width — `96.0 px`. Ratified verbatim from
/// `client/src/ui/hand/mod.rs::HAND_CARD_DISPLAY_WIDTH_PX`.
pub const CARD_SLOT_HAND_FAN_WIDTH_PX: f32 = 96.0;

/// Hand fan card outer height — `136.0 px`. Ratified verbatim from
/// `client/src/ui/hand/mod.rs::HAND_CARD_DISPLAY_HEIGHT_PX`.
pub const CARD_SLOT_HAND_FAN_HEIGHT_PX: f32 = 136.0;

/// Hand fan card aspect-ratio band lower bound — `0.69`. Tight band
/// around the `96 / 136 ≈ 0.706` ratio so a future revision cannot
/// silently drift the slot into landscape.
pub const CARD_SLOT_HAND_FAN_ASPECT_MIN: f32 = 0.69;

/// Hand fan card aspect-ratio band upper bound — `0.72`. Tight band
/// around the `96 / 136 ≈ 0.706` ratio.
pub const CARD_SLOT_HAND_FAN_ASPECT_MAX: f32 = 0.72;

/// Hand fan card border thickness — `1.0 px`. Friend-game chrome only.
pub const CARD_SLOT_HAND_FAN_BORDER_PX: f32 = 1.0;

/// Hand fan card image inset — left / right / top / bottom (`4 / 4 / 4 /
/// 28 px`). Image region occupies the upper portion of the portrait card;
/// text overlays the bottom band via the text inset.
pub const CARD_SLOT_HAND_FAN_IMAGE_INSET: UiRect = UiRect {
    left: Val::Px(4.0),
    right: Val::Px(4.0),
    top: Val::Px(4.0),
    bottom: Val::Px(28.0),
};

/// Hand fan card text inset — left / right / top / bottom (`4 / 4 / 112 /
/// 4 px`). Text region sits in the bottom band of the portrait card,
/// disjoint from the image region above.
pub const CARD_SLOT_HAND_FAN_TEXT_INSET: UiRect = UiRect {
    left: Val::Px(4.0),
    right: Val::Px(4.0),
    top: Val::Px(112.0),
    bottom: Val::Px(4.0),
};

// ---------------------------------------------------------------------------
// DraftGrid — 120 × 56 px landscape
// ---------------------------------------------------------------------------

/// Draft grid card outer width — `120.0 px`. Ratified verbatim from
/// `client/src/ui/hand/mod.rs::HAND_DRAFT_GRID_CARD_WIDTH_PX`.
pub const CARD_SLOT_DRAFT_GRID_WIDTH_PX: f32 = 120.0;

/// Draft grid card outer height — `56.0 px`. Ratified verbatim from
/// `client/src/ui/hand/mod.rs::HAND_DRAFT_GRID_CARD_HEIGHT_PX`.
pub const CARD_SLOT_DRAFT_GRID_HEIGHT_PX: f32 = 56.0;

/// Draft grid card aspect-ratio band lower bound — `2.10`. Tight band
/// around the `120 / 56 ≈ 2.143` ratio.
pub const CARD_SLOT_DRAFT_GRID_ASPECT_MIN: f32 = 2.10;

/// Draft grid card aspect-ratio band upper bound — `2.18`. Tight band
/// around the `120 / 56 ≈ 2.143` ratio.
pub const CARD_SLOT_DRAFT_GRID_ASPECT_MAX: f32 = 2.18;

/// Draft grid card border thickness — `1.0 px`. Friend-game chrome only.
pub const CARD_SLOT_DRAFT_GRID_BORDER_PX: f32 = 1.0;

/// Draft grid card image inset — left / right / top / bottom (`4 / 64 / 4
/// / 4 px`). Image region occupies the left half of the landscape card,
/// disjoint from the text region on the right.
pub const CARD_SLOT_DRAFT_GRID_IMAGE_INSET: UiRect = UiRect {
    left: Val::Px(4.0),
    right: Val::Px(64.0),
    top: Val::Px(4.0),
    bottom: Val::Px(4.0),
};

/// Draft grid card text inset — left / right / top / bottom (`60 / 4 / 4
/// / 4 px`). Text region occupies the right half of the landscape card,
/// disjoint from the image region on the left.
pub const CARD_SLOT_DRAFT_GRID_TEXT_INSET: UiRect = UiRect {
    left: Val::Px(60.0),
    right: Val::Px(4.0),
    top: Val::Px(4.0),
    bottom: Val::Px(4.0),
};

// ---------------------------------------------------------------------------
// ShopSlot — 136 × 78 px landscape  (Phase 1 migration target)
// ---------------------------------------------------------------------------

/// Shop slot outer width — `136.0 px`. Ratified verbatim from the
/// pre-migration `client/src/ui/shop_auction/mod.rs::shop_slot_node`
/// literal.
pub const CARD_SLOT_SHOP_SLOT_WIDTH_PX: f32 = 136.0;

/// Shop slot outer height — `78.0 px`. Ratified verbatim from the
/// pre-migration `client/src/ui/shop_auction/mod.rs::shop_slot_node`
/// literal.
pub const CARD_SLOT_SHOP_SLOT_HEIGHT_PX: f32 = 78.0;

/// Shop slot aspect-ratio band lower bound — `1.70`. Tight band around
/// the `136 / 78 ≈ 1.744` ratio.
pub const CARD_SLOT_SHOP_SLOT_ASPECT_MIN: f32 = 1.70;

/// Shop slot aspect-ratio band upper bound — `1.78`. Tight band around
/// the `136 / 78 ≈ 1.744` ratio.
pub const CARD_SLOT_SHOP_SLOT_ASPECT_MAX: f32 = 1.78;

/// Shop slot border thickness — `1.0 px`. Friend-game chrome only.
/// Ratified verbatim from the pre-migration literal.
pub const CARD_SLOT_SHOP_SLOT_BORDER_PX: f32 = 1.0;

/// Shop slot image inset — left / right / top / bottom (`4 / 80 / 4 / 4
/// px`). Image region occupies the left half of the landscape slot,
/// disjoint from the text region on the right.
pub const CARD_SLOT_SHOP_SLOT_IMAGE_INSET: UiRect = UiRect {
    left: Val::Px(4.0),
    right: Val::Px(80.0),
    top: Val::Px(4.0),
    bottom: Val::Px(4.0),
};

/// Shop slot text inset — left / right / top / bottom (`60 / 4 / 4 / 4
/// px`). Text region occupies the right half of the landscape slot,
/// disjoint from the image region on the left.
pub const CARD_SLOT_SHOP_SLOT_TEXT_INSET: UiRect = UiRect {
    left: Val::Px(60.0),
    right: Val::Px(4.0),
    top: Val::Px(4.0),
    bottom: Val::Px(4.0),
};

// ---------------------------------------------------------------------------
// AuctionFeatured — 380 × 280 px landscape
// ---------------------------------------------------------------------------

/// Auction featured card outer width — `380.0 px`. Ratified verbatim from
/// `client/src/ui/shop_auction/mod.rs::AUCTION_FEATURED_CARD_WIDTH_PX`.
pub const CARD_SLOT_AUCTION_FEATURED_WIDTH_PX: f32 = 380.0;

/// Auction featured card outer height — `280.0 px`. Ratified verbatim
/// from `client/src/ui/shop_auction/mod.rs::AUCTION_FEATURED_CARD_HEIGHT_PX`.
pub const CARD_SLOT_AUCTION_FEATURED_HEIGHT_PX: f32 = 280.0;

/// Auction featured card aspect-ratio band lower bound — `1.32`. Tight
/// band around the `380 / 280 ≈ 1.357` ratio.
pub const CARD_SLOT_AUCTION_FEATURED_ASPECT_MIN: f32 = 1.32;

/// Auction featured card aspect-ratio band upper bound — `1.40`. Tight
/// band around the `380 / 280 ≈ 1.357` ratio.
pub const CARD_SLOT_AUCTION_FEATURED_ASPECT_MAX: f32 = 1.40;

/// Auction featured card border thickness — `3.0 px`. Ratified verbatim
/// from `client/src/ui/shop_auction/mod.rs::AUCTION_FEATURED_CARD_FRAME_THICKNESS_PX`.
/// Heavier than the shop slot to satisfy the featured-card
/// differentiation contract from Sprint 14 PROMPT 931.
pub const CARD_SLOT_AUCTION_FEATURED_BORDER_PX: f32 = 3.0;

/// Auction featured card image inset — left / right / top / bottom (`16
/// / 16 / 16 / 96 px`). Image region occupies the upper portion of the
/// landscape card; text overlays the bottom band via the text inset.
pub const CARD_SLOT_AUCTION_FEATURED_IMAGE_INSET: UiRect = UiRect {
    left: Val::Px(16.0),
    right: Val::Px(16.0),
    top: Val::Px(16.0),
    bottom: Val::Px(96.0),
};

/// Auction featured card text inset — left / right / top / bottom (`16 /
/// 16 / 200 / 16 px`). Text region sits in the bottom band of the
/// landscape card, disjoint from the image region above.
pub const CARD_SLOT_AUCTION_FEATURED_TEXT_INSET: UiRect = UiRect {
    left: Val::Px(16.0),
    right: Val::Px(16.0),
    top: Val::Px(200.0),
    bottom: Val::Px(16.0),
};

// ---------------------------------------------------------------------------
// BoardStagedGhost — 64 × 80 px portrait (world-space; one board cell)
// ---------------------------------------------------------------------------

/// Board staged-ghost outer width — `64.0 px`. Ratified verbatim from
/// `design/gdd/board-rendering.md` Rule 3 `cell_width = 64.0` (world
/// units, one board cell).
pub const CARD_SLOT_BOARD_GHOST_WIDTH_PX: f32 = 64.0;

/// Board staged-ghost outer height — `80.0 px`. Ratified verbatim from
/// `design/gdd/board-rendering.md` Rule 3 `lane_height = 80.0` (world
/// units, one board lane row).
pub const CARD_SLOT_BOARD_GHOST_HEIGHT_PX: f32 = 80.0;

/// Board staged-ghost aspect-ratio band lower bound — `0.78`. Tight band
/// around the `64 / 80 = 0.8` ratio.
pub const CARD_SLOT_BOARD_GHOST_ASPECT_MIN: f32 = 0.78;

/// Board staged-ghost aspect-ratio band upper bound — `0.82`. Tight band
/// around the `64 / 80 = 0.8` ratio.
pub const CARD_SLOT_BOARD_GHOST_ASPECT_MAX: f32 = 0.82;

/// Board staged-ghost border thickness — `0.0 px`. World-space sprite has
/// no `bevy_ui` border; the field exists for catalog symmetry.
pub const CARD_SLOT_BOARD_GHOST_BORDER_PX: f32 = 0.0;

/// Board staged-ghost image inset — left / right / top / bottom (`2 / 2
/// / 2 / 14 px`). Image region occupies the upper portion of the ghost
/// preview; the text region (rare-art card name) sits in the bottom band.
pub const CARD_SLOT_BOARD_GHOST_IMAGE_INSET: UiRect = UiRect {
    left: Val::Px(2.0),
    right: Val::Px(2.0),
    top: Val::Px(2.0),
    bottom: Val::Px(14.0),
};

/// Board staged-ghost text inset — left / right / top / bottom (`2 / 2 /
/// 70 / 2 px`). Text region sits in the bottom band of the ghost preview,
/// disjoint from the image region above.
pub const CARD_SLOT_BOARD_GHOST_TEXT_INSET: UiRect = UiRect {
    left: Val::Px(2.0),
    right: Val::Px(2.0),
    top: Val::Px(70.0),
    bottom: Val::Px(2.0),
};

// ---------------------------------------------------------------------------
// Geometry accessor
// ---------------------------------------------------------------------------

/// Returns the canonical [`CardSlotGeometry`] for a given [`CardSlotKind`].
///
/// Every per-kind value is read from a named `const` declared above. The
/// hit-target inset defaults to [`UiRect::ZERO`] for every kind — surfaces
/// that need a larger or offset hit target read the field directly.
pub const fn card_slot_geometry(kind: CardSlotKind) -> CardSlotGeometry {
    match kind {
        CardSlotKind::HandFan => CardSlotGeometry {
            outer_width_px: CARD_SLOT_HAND_FAN_WIDTH_PX,
            outer_height_px: CARD_SLOT_HAND_FAN_HEIGHT_PX,
            aspect_ratio_band: (CARD_SLOT_HAND_FAN_ASPECT_MIN, CARD_SLOT_HAND_FAN_ASPECT_MAX),
            border_thickness_px: CARD_SLOT_HAND_FAN_BORDER_PX,
            image_inset_px: CARD_SLOT_HAND_FAN_IMAGE_INSET,
            text_inset_px: CARD_SLOT_HAND_FAN_TEXT_INSET,
            hit_target_inset_px: UiRect::ZERO,
            z_layer: z_layers::UI_BASE,
        },
        CardSlotKind::DraftGrid => CardSlotGeometry {
            outer_width_px: CARD_SLOT_DRAFT_GRID_WIDTH_PX,
            outer_height_px: CARD_SLOT_DRAFT_GRID_HEIGHT_PX,
            aspect_ratio_band: (
                CARD_SLOT_DRAFT_GRID_ASPECT_MIN,
                CARD_SLOT_DRAFT_GRID_ASPECT_MAX,
            ),
            border_thickness_px: CARD_SLOT_DRAFT_GRID_BORDER_PX,
            image_inset_px: CARD_SLOT_DRAFT_GRID_IMAGE_INSET,
            text_inset_px: CARD_SLOT_DRAFT_GRID_TEXT_INSET,
            hit_target_inset_px: UiRect::ZERO,
            z_layer: z_layers::UI_BASE,
        },
        CardSlotKind::ShopSlot => CardSlotGeometry {
            outer_width_px: CARD_SLOT_SHOP_SLOT_WIDTH_PX,
            outer_height_px: CARD_SLOT_SHOP_SLOT_HEIGHT_PX,
            aspect_ratio_band: (
                CARD_SLOT_SHOP_SLOT_ASPECT_MIN,
                CARD_SLOT_SHOP_SLOT_ASPECT_MAX,
            ),
            border_thickness_px: CARD_SLOT_SHOP_SLOT_BORDER_PX,
            image_inset_px: CARD_SLOT_SHOP_SLOT_IMAGE_INSET,
            text_inset_px: CARD_SLOT_SHOP_SLOT_TEXT_INSET,
            hit_target_inset_px: UiRect::ZERO,
            z_layer: z_layers::UI_BASE,
        },
        CardSlotKind::AuctionFeatured => CardSlotGeometry {
            outer_width_px: CARD_SLOT_AUCTION_FEATURED_WIDTH_PX,
            outer_height_px: CARD_SLOT_AUCTION_FEATURED_HEIGHT_PX,
            aspect_ratio_band: (
                CARD_SLOT_AUCTION_FEATURED_ASPECT_MIN,
                CARD_SLOT_AUCTION_FEATURED_ASPECT_MAX,
            ),
            border_thickness_px: CARD_SLOT_AUCTION_FEATURED_BORDER_PX,
            image_inset_px: CARD_SLOT_AUCTION_FEATURED_IMAGE_INSET,
            text_inset_px: CARD_SLOT_AUCTION_FEATURED_TEXT_INSET,
            hit_target_inset_px: UiRect::ZERO,
            z_layer: z_layers::UI_BASE,
        },
        CardSlotKind::BoardStagedGhost => CardSlotGeometry {
            outer_width_px: CARD_SLOT_BOARD_GHOST_WIDTH_PX,
            outer_height_px: CARD_SLOT_BOARD_GHOST_HEIGHT_PX,
            aspect_ratio_band: (
                CARD_SLOT_BOARD_GHOST_ASPECT_MIN,
                CARD_SLOT_BOARD_GHOST_ASPECT_MAX,
            ),
            border_thickness_px: CARD_SLOT_BOARD_GHOST_BORDER_PX,
            image_inset_px: CARD_SLOT_BOARD_GHOST_IMAGE_INSET,
            text_inset_px: CARD_SLOT_BOARD_GHOST_TEXT_INSET,
            hit_target_inset_px: UiRect::ZERO,
            z_layer: z_layers::UI_OVERLAY,
        },
    }
}

/// Returns the canonical card-art [`UiRect`] inset for the given kind.
/// Convenience accessor over [`card_slot_geometry`].
pub const fn card_slot_image_inset(kind: CardSlotKind) -> UiRect {
    card_slot_geometry(kind).image_inset_px
}

/// Returns the canonical text-block [`UiRect`] inset for the given kind.
/// Convenience accessor over [`card_slot_geometry`].
pub const fn card_slot_text_inset(kind: CardSlotKind) -> UiRect {
    card_slot_geometry(kind).text_inset_px
}

/// Returns the canonical hit-target [`UiRect`] inset for the given kind.
/// Default is [`UiRect::ZERO`] — hit target equals the visual outer
/// rectangle. A surface that needs a larger or offset hit target reads
/// the field directly; the integration test asserts the hit target is a
/// superset of (or equal to) the visual outer rectangle for every kind.
pub const fn card_slot_hit_target(kind: CardSlotKind) -> UiRect {
    card_slot_geometry(kind).hit_target_inset_px
}

// ---------------------------------------------------------------------------
// Node builder
// ---------------------------------------------------------------------------

/// Builds a fully-composed [`bevy::ui::Node`] for the given
/// [`CardSlotKind`].
///
/// The Node carries `position_type`, `width`, `height`, `border`, and
/// `padding` set from the geometry. `display` and `flex_direction` are
/// deterministic ([`Display::Flex`] + [`FlexDirection::Column`]) so
/// image-then-text vertical stacking is the cheap default for the
/// landscape kinds and image-with-overlay-text composition is the cheap
/// default for the portrait `HandFan` kind via an [`PositionType::Absolute`]
/// text child.
///
/// The builder NEVER instantiates [`card_slot_node`] for any other kind
/// (the leaf-only / no-nested-cards rule per AC2). Surfaces that need to
/// paint multiple cards do so by placing N siblings under a flex parent,
/// not by nesting a card slot inside another card slot.
///
/// `position_type` defaults to [`PositionType::Absolute`] to match the
/// existing per-surface call-site convention (every current card surface
/// positions its slot via explicit `left` / `top` offsets); a consumer
/// that wants relative positioning can override the field on the returned
/// Node.
///
/// ## Inset / GlobalZIndex companions
///
/// Sprint 17 row `S17-UI-CARD-SLOT-INSET-WIRING-001` (SOURCE-1077-06)
/// adds two net-additive sibling builders that consume the geometry
/// catalog's `image_inset_px`, `text_inset_px`, and `z_layer` fields:
/// [`card_slot_image_inset_node`] and [`card_slot_text_inset_node`].
/// Each returns a `(Node, GlobalZIndex)` bundle suitable for direct
/// `Commands::spawn` use so per-surface migration siblings (the
/// Sprint 17+ Backlog family `S17-UI-CARD-SLOT-MIGRATION-*`) reduce
/// to a thin re-author of three component-set inserts instead of
/// re-authoring child-positioning arithmetic per consumer site.
///
/// ## Padding catalog status
///
/// `card_slot_geometry(kind)` does NOT currently expose a padding
/// rectangle (only `image_inset_px`, `text_inset_px`, and
/// `hit_target_inset_px`). The sibling inset builders below therefore
/// emit no `Node.padding` field — child layout is driven by the
/// inset rectangles themselves via [`PositionType::Absolute`]. A
/// future revision that promotes padding into [`CardSlotGeometry`]
/// would land in a separate row; this row does not retune the
/// geometry catalog (AC8).
pub fn card_slot_node(kind: CardSlotKind) -> Node {
    let geometry = card_slot_geometry(kind);
    Node {
        position_type: PositionType::Absolute,
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        width: Val::Px(geometry.outer_width_px),
        height: Val::Px(geometry.outer_height_px),
        border: UiRect::all(Val::Px(geometry.border_thickness_px)),
        ..default()
    }
}

/// Builds the per-kind **image-inset child** for a card slot.
///
/// Returns a `(Node, GlobalZIndex)` bundle. The Node is
/// [`PositionType::Absolute`] and its `left` / `right` / `top` /
/// `bottom` fields are read verbatim from
/// `card_slot_geometry(kind).image_inset_px`; the bundle's
/// [`GlobalZIndex`] is read verbatim from
/// `card_slot_geometry(kind).z_layer`. No numeric literal is authored
/// inside this builder — every value flows through the geometry
/// catalog so the canonical inset / z-layer constants edit in one
/// place (AC1 / AC3 / AC8).
///
/// This builder is **net-additive** relative to the Sprint 16 story
/// 009 primitive: it does NOT touch [`card_slot_node`] (the outer
/// rectangle builder), it does NOT migrate any consumer surface, and
/// it does NOT retune any [`card_slot_geometry`] constant. Per-surface
/// migration of `HandFan` / `DraftGrid` / `AuctionFeatured` /
/// `BoardStagedGhost` remains owned by the Sprint 17+ Backlog family
/// `S17-UI-CARD-SLOT-MIGRATION-*`; this builder is the canonical
/// child-positioning primitive those rows will consume.
///
/// ## Output shape
///
/// ```rust,ignore
/// // Spawn an image child sized to the canonical inset for the kind.
/// commands.entity(card_root).with_children(|parent| {
///     parent.spawn(card_slot_image_inset_node(CardSlotKind::ShopSlot));
/// });
/// ```
///
/// The returned Node carries no `width` / `height` fields — width and
/// height are *derived* from the four absolute-position edges (the
/// rectangle bounded by `left` / `right` / `top` / `bottom` of the
/// parent's interior). This matches how the geometry catalog stores
/// the inset (a [`UiRect`] of four side values, not a width / height
/// pair). See AC6 for the per-side equality assertions.
pub fn card_slot_image_inset_node(kind: CardSlotKind) -> (Node, GlobalZIndex) {
    let geometry = card_slot_geometry(kind);
    let inset = geometry.image_inset_px;
    (
        Node {
            position_type: PositionType::Absolute,
            left: inset.left,
            right: inset.right,
            top: inset.top,
            bottom: inset.bottom,
            ..default()
        },
        geometry.z_layer,
    )
}

/// Builds the per-kind **text-inset child** for a card slot.
///
/// Returns a `(Node, GlobalZIndex)` bundle. The Node is
/// [`PositionType::Absolute`] and its `left` / `right` / `top` /
/// `bottom` fields are read verbatim from
/// `card_slot_geometry(kind).text_inset_px`; the bundle's
/// [`GlobalZIndex`] is read verbatim from
/// `card_slot_geometry(kind).z_layer`. No numeric literal is authored
/// inside this builder — every value flows through the geometry
/// catalog (AC2 / AC3 / AC8).
///
/// This builder is the text-region counterpart to
/// [`card_slot_image_inset_node`]; the two share the same
/// [`GlobalZIndex`] for a given kind so the image and text children
/// composite into the same z-layer as their parent card slot.
/// Per-surface migration of consumer sites remains owned by the
/// Sprint 17+ Backlog family `S17-UI-CARD-SLOT-MIGRATION-*`.
///
/// ## Output shape
///
/// ```rust,ignore
/// commands.entity(card_root).with_children(|parent| {
///     parent.spawn(card_slot_text_inset_node(CardSlotKind::ShopSlot));
/// });
/// ```
pub fn card_slot_text_inset_node(kind: CardSlotKind) -> (Node, GlobalZIndex) {
    let geometry = card_slot_geometry(kind);
    let inset = geometry.text_inset_px;
    (
        Node {
            position_type: PositionType::Absolute,
            left: inset.left,
            right: inset.right,
            top: inset.top,
            bottom: inset.bottom,
            ..default()
        },
        geometry.z_layer,
    )
}

// ---------------------------------------------------------------------------
// Inline unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn aspect_ratio(geometry: &CardSlotGeometry) -> f32 {
        geometry.outer_width_px / geometry.outer_height_px
    }

    #[test]
    fn all_card_slot_kinds_enumerates_every_variant() {
        // ALL_CARD_SLOT_KINDS is the canonical iteration source. Adding a
        // variant to the enum without updating the array would break
        // downstream tests; this test catches the divergence locally.
        assert_eq!(ALL_CARD_SLOT_KINDS.len(), 5);
        for kind in ALL_CARD_SLOT_KINDS {
            // Resolves to a valid geometry — no panic.
            let _ = card_slot_geometry(kind);
        }
    }

    #[test]
    fn ac2_each_kinds_aspect_ratio_falls_in_declared_band() {
        for kind in ALL_CARD_SLOT_KINDS {
            let geometry = card_slot_geometry(kind);
            let (min, max) = geometry.aspect_ratio_band;
            let ratio = aspect_ratio(&geometry);
            assert!(
                ratio >= min && ratio <= max,
                "AC2 aspect ratio out of band for {kind:?}: ratio={ratio:.4} band=({min:.4}, {max:.4})",
            );
        }
    }

    #[test]
    fn ac2_each_kinds_outer_dimensions_are_strictly_positive_and_finite() {
        for kind in ALL_CARD_SLOT_KINDS {
            let geometry = card_slot_geometry(kind);
            assert!(geometry.outer_width_px > 0.0 && geometry.outer_width_px.is_finite());
            assert!(geometry.outer_height_px > 0.0 && geometry.outer_height_px.is_finite());
        }
    }

    #[test]
    fn ac4_image_and_text_insets_fit_inside_outer_rectangle_per_kind() {
        for kind in ALL_CARD_SLOT_KINDS {
            let geometry = card_slot_geometry(kind);
            let (image_l, image_r, image_t, image_b) = inset_pixels(&geometry.image_inset_px);
            let (text_l, text_r, text_t, text_b) = inset_pixels(&geometry.text_inset_px);
            assert!(
                image_l + image_r < geometry.outer_width_px,
                "AC4 image inset width overflow for {kind:?}",
            );
            assert!(
                image_t + image_b < geometry.outer_height_px,
                "AC4 image inset height overflow for {kind:?}",
            );
            assert!(
                text_l + text_r < geometry.outer_width_px,
                "AC4 text inset width overflow for {kind:?}",
            );
            assert!(
                text_t + text_b < geometry.outer_height_px,
                "AC4 text inset height overflow for {kind:?}",
            );
        }
    }

    fn inset_pixels(rect: &UiRect) -> (f32, f32, f32, f32) {
        // `Val::ZERO` resolves to `Val::Px(0.0)`; the `Val::Px(px)` arm
        // covers both the explicit zero rect and per-side pixel values.
        let to_px = |v: Val| match v {
            Val::Px(px) => px,
            other => panic!("AC4 expected Val::Px inset, found {other:?}"),
        };
        (
            to_px(rect.left),
            to_px(rect.right),
            to_px(rect.top),
            to_px(rect.bottom),
        )
    }

    #[test]
    fn ac1_all_accessors_return_geometry_consistent_values() {
        for kind in ALL_CARD_SLOT_KINDS {
            let geometry = card_slot_geometry(kind);
            assert_eq!(card_slot_image_inset(kind), geometry.image_inset_px);
            assert_eq!(card_slot_text_inset(kind), geometry.text_inset_px);
            assert_eq!(card_slot_hit_target(kind), geometry.hit_target_inset_px);
        }
    }

    #[test]
    fn ac7_card_slot_node_width_height_match_geometry_for_shop_slot() {
        let geometry = card_slot_geometry(CardSlotKind::ShopSlot);
        let node = card_slot_node(CardSlotKind::ShopSlot);
        assert_eq!(node.width, Val::Px(geometry.outer_width_px));
        assert_eq!(node.height, Val::Px(geometry.outer_height_px));
    }

    // -----------------------------------------------------------------
    // Sprint 17 S17-UI-CARD-SLOT-INSET-WIRING-001 (SOURCE-1077-06) —
    // sibling inset builders + GlobalZIndex wiring.
    // -----------------------------------------------------------------

    #[test]
    fn s17_image_inset_node_matches_geometry_per_kind() {
        // AC1 / AC6(a): the image-inset builder's Node is
        // PositionType::Absolute and its four edge fields equal the
        // geometry catalog's image_inset_px for every kind.
        for kind in ALL_CARD_SLOT_KINDS {
            let geometry = card_slot_geometry(kind);
            let (node, _z) = card_slot_image_inset_node(kind);
            assert_eq!(
                node.position_type,
                PositionType::Absolute,
                "image inset node must be Absolute for {kind:?}",
            );
            assert_eq!(
                node.left, geometry.image_inset_px.left,
                "image inset left drift for {kind:?}",
            );
            assert_eq!(
                node.right, geometry.image_inset_px.right,
                "image inset right drift for {kind:?}",
            );
            assert_eq!(
                node.top, geometry.image_inset_px.top,
                "image inset top drift for {kind:?}",
            );
            assert_eq!(
                node.bottom, geometry.image_inset_px.bottom,
                "image inset bottom drift for {kind:?}",
            );
        }
    }

    #[test]
    fn s17_text_inset_node_matches_geometry_per_kind() {
        // AC2 / AC6(b): the text-inset builder's Node is
        // PositionType::Absolute and its four edge fields equal the
        // geometry catalog's text_inset_px for every kind.
        for kind in ALL_CARD_SLOT_KINDS {
            let geometry = card_slot_geometry(kind);
            let (node, _z) = card_slot_text_inset_node(kind);
            assert_eq!(
                node.position_type,
                PositionType::Absolute,
                "text inset node must be Absolute for {kind:?}",
            );
            assert_eq!(
                node.left, geometry.text_inset_px.left,
                "text inset left drift for {kind:?}",
            );
            assert_eq!(
                node.right, geometry.text_inset_px.right,
                "text inset right drift for {kind:?}",
            );
            assert_eq!(
                node.top, geometry.text_inset_px.top,
                "text inset top drift for {kind:?}",
            );
            assert_eq!(
                node.bottom, geometry.text_inset_px.bottom,
                "text inset bottom drift for {kind:?}",
            );
        }
    }

    #[test]
    fn s17_inset_builders_thread_global_z_index_from_geometry_per_kind() {
        // AC3 / AC6(c): both inset builders emit a GlobalZIndex equal
        // to card_slot_geometry(kind).z_layer.
        for kind in ALL_CARD_SLOT_KINDS {
            let geometry = card_slot_geometry(kind);
            let (_image_node, image_z) = card_slot_image_inset_node(kind);
            let (_text_node, text_z) = card_slot_text_inset_node(kind);
            assert_eq!(
                image_z.0, geometry.z_layer.0,
                "image inset GlobalZIndex drift for {kind:?}",
            );
            assert_eq!(
                text_z.0, geometry.z_layer.0,
                "text inset GlobalZIndex drift for {kind:?}",
            );
        }
    }
}
