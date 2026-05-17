diff --git a/docs/ux/global-ui-design-spec.md b/docs/ux/global-ui-design-spec.md
index 42634a7..1a47585 100644
--- a/docs/ux/global-ui-design-spec.md
+++ b/docs/ux/global-ui-design-spec.md
@@ -470,10 +470,13 @@ section as guidance rather than a strict contract.

 ### Card slot composition

-Owned by Tier 3 story 13 (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`) — refactor
-of hand + shop + auction card slot primitive. This spec does not bind a
-card slot composition; story 13 authors the primitive after Tier 1
-surfaces stabilise.
+Forward reference: see §12 "Card Slot Primitive" below. Sprint 16 story
+009 (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`) authors the canonical primitive
+module at `client/src/ui/design_tokens/card_slot.rs` and Phase 1 migrates
+the shop slot well call site. Hand / draft / auction-featured / board
+staged-ghost migrations are owned by the Sprint 16+
+`S16-UI-CARD-SLOT-MIGRATION-*` follow-on family. §12 is the source-of-
+truth for the canonical numeric values; this section is a pointer.

 ### Modal centering pattern

@@ -578,6 +581,126 @@ states:

 ---

+## §12 Card Slot Primitive
+
+Canonical layout primitive for every card-painting surface in the
+playable client. Sprint 16 story 009
+(`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`, Tier 3 rank 13) authors the token
+module and amends this section; per-surface migration of the four
+existing card surfaces is split between this story (Phase 1 shop slot
+only) and the Sprint 16+ follow-on family
+(`S16-UI-CARD-SLOT-MIGRATION-*`).
+
+**Source-of-truth module**:
+`client/src/ui/design_tokens/card_slot.rs` (NEW; landed by Sprint 16
+story 009). This section ratifies the canonical numeric values for each
+of the five `CardSlotKind` variants; the §10 "Card slot composition"
+subsection above forward-references this section as the binding source.
+
+### CardSlotKind variants
+
+| Variant | Outer (px) | Aspect band | Border (px) | Z-layer | Canonical consumer |
+|---------|------------|-------------|-------------|---------|--------------------|
+| `HandFan`          |  96 × 136 portrait  | `0.69..=0.72` | `1.0` | `UI_BASE`    | `client/src/ui/hand/mod.rs` hand fan card (`HAND_CARD_DISPLAY_*`). |
+| `DraftGrid`        | 120 ×  56 landscape | `2.10..=2.18` | `1.0` | `UI_BASE`    | `client/src/ui/hand/mod.rs` draft initial grid (`HAND_DRAFT_GRID_CARD_*`). |
+| `ShopSlot`         | 136 ×  78 landscape | `1.70..=1.78` | `1.0` | `UI_BASE`    | `client/src/ui/shop_auction/mod.rs::shop_slot_node` (Phase 1 migration target). |
+| `AuctionFeatured`  | 380 × 280 landscape | `1.32..=1.40` | `3.0` | `UI_BASE`    | `client/src/ui/shop_auction/mod.rs::auction_featured_card_node` (`AUCTION_FEATURED_CARD_*`). |
+| `BoardStagedGhost` |  64 ×  80 portrait  | `0.78..=0.82` | `0.0` | `UI_OVERLAY` | World-space ghost preview sized to one board cell per `docs/ux/board-rendering-spec.md` BR-001 (`cell_width = 64.0`, `lane_height = 80.0`). |
+
+### Image / text / hit-target insets
+
+Insets are expressed as a `(left, right, top, bottom)` `UiRect` in
+pixels. The image rectangle and text rectangle MUST be disjoint within
+the outer rectangle — the integration test asserts containment per kind.
+
+| Variant | Image inset (L / R / T / B) | Text inset (L / R / T / B) | Hit-target inset |
+|---------|-----------------------------|----------------------------|------------------|
+| `HandFan`          |   4 /  4 /   4 / 28 |   4 /  4 / 112 /  4 | `UiRect::ZERO` (hit target == visual outer rectangle). |
+| `DraftGrid`        |   4 / 64 /   4 /  4 |  60 /  4 /   4 /  4 | `UiRect::ZERO`. |
+| `ShopSlot`         |   4 / 80 /   4 /  4 |  60 /  4 /   4 /  4 | `UiRect::ZERO`. |
+| `AuctionFeatured`  |  16 / 16 /  16 / 96 |  16 / 16 / 200 / 16 | `UiRect::ZERO`. |
+| `BoardStagedGhost` |   2 /  2 /   2 / 14 |   2 /  2 /  70 /  2 | `UiRect::ZERO`. |
+
+Per the AC7 contract the hit-target rectangle is a **superset of or
+equal to** the visual outer rectangle. The default `UiRect::ZERO` means
+the hit target equals the visual outer rectangle; a future per-surface
+migration sibling MAY outset further (e.g. focus-ring outset).
+
+### Composition rules
+
+1. **No nested cards.** A card slot is **leaf-only** — it has image and
+   text regions, NOT a child card slot. The `card_slot_node` builder for
+   kind `K` MUST NOT instantiate `card_slot_node(K')` for any other
+   kind. Composition that paints multiple cards (draft initial grid;
+   shop slot row) does so by placing N siblings under a flex parent.
+2. **Stable aspect ratio across viewports.** Slot dimensions are
+   pixel-fixed per §4 spacing scale — no viewport-driven scaling. The
