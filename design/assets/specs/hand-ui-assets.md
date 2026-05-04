# Asset Specs — System: hand-ui

> **Source**: design/gdd/hand-ui.md
> **Art Bible**: design/art/art-bible.md
> **Generated**: 2026-04-30
> **Status**: 36 assets specced / 36 approved / 0 in production / 0 done
> **Asset IDs**: ASSET-052 through ASSET-087 (continuing from ASSET-051 in manifest)

---

## Summary

| Category | Count | Art Files | Notes |
|---|---|---|---|
| Runtime Card Composition | 3 | Per-card art now tracked in `design/assets/specs/cards/` | Display, zoom, drag templates |
| UI | 18 | 9 PNG · 4 code-only · 3 shader · 2 procedural | See per-asset Engine Notes |
| VFX / Particles | 5 | 1 PNG · 4 WGSL shaders | ASSET-073 is sprited; 074–077 GPU material |
| Audio | 10 | 10 OGG files | All `ui_hand` channel, mono, 44100 Hz |
| **Total** | **36** | **12 unique PNGs + 6 shaders + 10 OGG** | |

### Atlas Assignments

| Atlas | Assets |
|---|---|
| `atlas_cards` (2048×2048) | Per-card display illustrations (ASSET-227-234), ASSET-055 (N class variants) |
| `atlas_ui_hud` (1024×1024) | ASSET-056, 057, 058, 059, 061, 062, 063, 066, 067 |
| `atlas_vfx` (1024×1024) | ASSET-073 |
| Standalone (not atlased) | ASSET-053 (Card Zoom — on-demand load/evict) |
| None (shader / code / audio) | ASSET-060, 064–065, 068–072, 074–087 |

### Engine API Risk Flags (Bevy 0.18 — post-cutoff)

| Flag | Risk | Affects |
|---|---|---|
| FLAG-1 | HIGH | `UiMaterial`/`MaterialNode` API — verify before shader work on ASSET-071–076 |
| FLAG-2 | HIGH | Bevy 0.18 Audio API (`AudioPlayer`/`PlaybackSettings`) — verify before ASSET-078–087 |
| FLAG-3 | HIGH | `Material2d` for world-space outline — verify before ASSET-077 |

| FLAG-4 | ARCH | OQ6 — atlas sharing across UI/world boundary has no batching benefit; flag to ADR |

---

## 2026-05-04 Revision - Runtime Card Composition

ASSET-052 through ASSET-054 are runtime presentation layers, not unique card illustrations. Unique card illustrations are now tracked per current `cards.json` `art_id` in `design/assets/specs/cards/` as ASSET-227 through ASSET-234.

Cards are composed at runtime from:

1. Per-card illustration-only art.
2. Card frame chrome.
3. Runtime mana, ATK, HP, type, rarity, text, and keyword layers.
4. Runtime hover, ghost, drag, lock, and state overlays.

No per-card illustration spec may bake card frames, badges, text, type/rarity, hover, ghost, drag, or state overlays into the art file.

---

## ASSET-052 — Card Display Composition Template

| Field | Value |
|-------|-------|
| Category | Runtime Card Composition / Template |
| Dimensions | 120×180 px |
| Atlas | `atlas_cards` |
| Format | Runtime composition; per-card illustration derivative is PNG-32 straight alpha |
| Naming | `card_[art_id]_art_display.png` for illustration layer only |
| Status | Needed |

**Visual Description:**
A 120x180 runtime-composed portrait card. The per-card illustration fills the center art field only. Frame chrome, card name, keyword text, mana badge, ATK badge, HP badge, type icon, rarity badge/text, and all state overlays are separate runtime layers.

**Art Bible Anchors:**
- §1 Visual Identity: bold outlines, fully saturated local color, readable at 64px minimum
- §3.2 Card Frame Anatomy: portrait orientation, straight corners, stat badges as embedded gem shapes
- §4.1 Primary Palette: ATK orange `#E07020` and HP teal `#2AA8C4` globally reserved; Ivory for card name
- §7.5 Iconography: stat gems use 2-color gradient fill; 18–24px floor
- §8.1 Sprite Resolution Tiers: Card Display 120×180 px; authored at Card Zoom (240×360), derived down
- §8.4 Outline Technique: 2px Void baked; no procedural outline for primary identity

**Generation Prompt:**
None for this template. Generate per-card illustration-only art from `design/assets/specs/cards/<art_id>.md`; runtime layers compose the display card.

**Engine Notes:**
`bevy_asset_loader` loads `atlas_cards` at session start. `CardDataPlugin` maps `CardId → TextureAtlas frame index` (OQ5 dependency). 10 pre-pooled `ImageNode` fan slot entities. Frame index and visibility toggled per S2CCardAcquired. Ghost state (ASSET-071) swaps to custom `UiMaterial`.

**State Variants:** `default` (active fan slot). Ghost → ASSET-071 shader.

---

## ASSET-053 — Card Zoom Composition Template

| Field | Value |
|-------|-------|
| Category | Runtime Card Composition / Template |
| Dimensions | 240×360 px |
| Atlas | Standalone (NOT atlased) |
| Format | Runtime composition; per-card illustration master is PNG-32 straight alpha |
| Naming | `card_[art_id]_art_zoom.png` for illustration layer only |
| Status | Needed |

**Visual Description:**
A 240x360 runtime-composed zoom card. The per-card illustration master is loaded on demand and remains illustration-only. Badge overlays, frame chrome, text, rarity/type, hover outline, ghost, and drag state are runtime layers and must not be baked into the zoom illustration.

**Art Bible Anchors:**
- §1 Visual Identity: same character as ASSET-052 — identifiable by shape at both tiers
- §3.2 Card Frame Anatomy: 20px safe zone for badge overlays
- §5.3 Card Art vs. Board Sprite: 3/4 front view, slightly downward-looking; one-way fidelity rule
- §8.1 Sprite Resolution Tiers: Card Zoom 240×360 px; loaded on-demand; NOT atlased
- §8.4 Outline Technique: 3–4px outer; 1px inner details at this tier

**Generation Prompt:**
None for this template. Generate per-card illustration-only zoom masters from `design/assets/specs/cards/<art_id>.md`.

**Engine Notes:**
Loaded on-demand via strong `Handle<Image>` at hover-start; dropped on hover-end (Bevy evicts from GPU). Stored at `assets/art/cards/zoom/`. Hover scale transition (80ms ease-out) via `bevy_tweening` `Lens<Node>`. ASSET-074 overlay handles hover gold outline.

**State Variants:** Single `default` variant.

---

## ASSET-054 — Drag Card Composition

| Field | Value |
|-------|-------|
| Category | Runtime Card Composition |
| Dimensions | 120×180 px logical (132×198 px at 1.10× drag scale — even integers) |
| Atlas | `atlas_cards` (reuses ASSET-052 art frame — no separate file) |
| Format | PNG-32 straight alpha (shared with ASSET-052) |
| Naming | Reuses `card_[art_id]_art_display.png` illustration layer plus runtime badge children |
| Status | Needed (reuse of ASSET-052) |

**Visual Description:**
A world-space clone of the runtime-composed card at 120x180 logical, rendered without frame chrome. The illustration layer remains per-card art; mana, ATK, HP, and other required playability badges are runtime child layers. Drag state does not require a separate complete-card texture.

**Art Bible Anchors:**
- §1 Visual Identity: art reads without frame containment; Void outline baked into art
- §3.2 Card Frame Anatomy: stat badges persist; border/frame layer stripped per VA-3
- §7.6 Animation Feel: no shadow or glow; scale alone communicates lift

**Generation Prompt:**
*(Reuses the ASSET-052 composition pipeline and per-card illustration derivative. No separate complete-card generation.)*

**Engine Notes:**
Pre-pooled world-space `Sprite` entity (not bevy_ui). On drag-start: `Visible`, atlas index set, `Transform.scale = Vec3::splat(1.10)`. Cursor follow each frame via `Res<BoardLayout>`. Stat badge children are world-space sprites via `ChildOf` (Bevy 0.18). Frame chrome child stays on fan slot with `Visibility::Hidden`.

**State Variants:** Single drag state. ⚠️ Badge child world-space strategy must be confirmed with ui-programmer before implementation.

---

## ASSET-055 — Card Frame Chrome (Hand)

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 120×180 px |
| Atlas | `atlas_cards` |
| Format | PNG-32 straight alpha |
| Naming | `card_frame_[class]_default_display.png` (one per class) |
| Status | Needed |

**Visual Description:**
A 120×180 px portrait overlay frame — transparent PNG composited over card art. Straight-sided portrait rectangle with very slightly rounded corners in the class color, 2px Void `#0D0D14` outer edge baked in. Interior fully transparent. Top-left diamond notch for mana badge, top-right twin-gem notch for ATK+HP, bottom 18px parchment `#F7F0DC` band. Border color = class identity: Xelor `#2E1B6E`, Sacrier `#8B1A2F`, Iop `#E05A00`, Eniripsa `#C45FA0`, Cra `#2A6B3C`.

**Art Bible Anchors:**
- §3.2 Card Frame Anatomy: straight-sided portrait; badge notch positions defined
- §4.4 Class Color Identity: frame color IS the class identifier — canonical colors must be exact
- §8.4 Outline Technique: 2px Void baked onto outer frame edge

**Generation Prompt:**
Ankama/Krosmaga card frame chrome, transparent PNG overlay, 120×180 portrait. Border thickness ~6–8px, class-colored fill (variant: dark purple-blue #2E1B6E for Xelor default). Baked 2px void black outer rim (#0D0D14). Interior fully transparent. Bottom 18px: warm parchment (#F7F0DC) opaque strip. Top-left: diamond-shaped notch area (mana badge). Top-right: two stacked gem-shaped notch areas (ATK/HP). Negative: no interior fill, no rounded pill, no ornate border flourishes.

**Engine Notes:**
Separate `ImageNode` child overlaid on card art, higher Z-order within fan slot UI hierarchy. One atlas frame per class. Excluded from drag sprite — `Visibility::Hidden` on drag entity's frame child.

**State Variants:** One frame per class. Confirm class count before atlas layout finalization.

---

## ASSET-056 — Mana Cost Diamond Badge

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 32×32 px |
| Atlas | `atlas_ui_hud` |
| Format | PNG-32 straight alpha |
| Naming | `ui_badge_mana_[class]_default_hud.png` |
| Status | Needed |

**Visual Description:**
A diamond/gem shape badge with a class-color background fill and 2px Void outline. Interior: two-tone cel-shade — lighter class-color highlight at top, base class color at body. Bold Ivory `#F7F0DC` Heavy numeral centered. Top-left of card frame; first cost-signal the eye reaches.

**Art Bible Anchors:**
- §3.2 Card Frame Anatomy: mana cost top-left, diamond/gem badge, class-color background
- §4.1 Primary Palette: class color per §4.4; Ivory for numeral
- §7.5 Iconography: stat gems use 2-color gradient fill; 18–24px floor

**Generation Prompt:**
Ankama cel-shaded stat gem, diamond/rhombus shape, 24px longest axis. Class-color background (deep blue-purple #2E1B6E for Xelor variant). Two-color cel fill: lighter tint at top, base class color at body. Bold Ivory numeral (#F7F0DC), Heavy, 12px. Small inner specular glint (flat cel triangle). Negative: no round shape, no pill, no gradient glow.

**Engine Notes:**
Numeral rendered as `Text` child (not baked). Recommend shader-driven color tint on a single neutral diamond frame to save atlas space.

**State Variants:** One per class OR shader-driven single frame.

---

## ASSET-057 — ATK Stat Badge

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 24×24 px |
| Atlas | `atlas_ui_hud` |
| Format | PNG-32 straight alpha |
| Naming | `ui_badge_atk_default_hud.png` |
| Status | Needed |

**Visual Description:**
A diamond-shaped gem badge filled with ATK Orange `#E07020`. Interior cel-shade: warm lighter orange `#F09040` triangle at top, base ATK orange on body. No interior outline. Bold Ivory Heavy numeral centered. ATK orange is globally reserved — this color appears nowhere else in the UI system.

**Art Bible Anchors:**
- §3.2 Card Frame Anatomy: ATK at top-right, orange diamond — always
- §4.1 Primary Palette: ATK Orange `#E07020` — stat only, globally reserved
- §9.2 Color Prohibitions: never reuse ATK orange for any non-stat purpose

**Generation Prompt:**
Ankama cel-shaded ATK stat gem, diamond/rhombus, 24px. Fill: ATK orange (#E07020) base, lighter (#F09040) cel highlight at top. No interior stroke. Ivory Heavy numeral centered. Negative: no red tint, no gold drift, no glow, no rounded shape.

**Engine Notes:**
Single universal frame. Numeral via `Text` child. Minimum 16×16 px render floor at 10-card overlap enforced by bevy_ui Node min-size.

**State Variants:** Single `default` variant.

---

## ASSET-058 — HP Stat Badge

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 24×24 px |
| Atlas | `atlas_ui_hud` |
| Format | PNG-32 straight alpha |
| Naming | `ui_badge_hp_default_hud.png` |
| Status | Needed |

**Visual Description:**
A gem-shaped badge (slightly rounder than strict diamond — elongated hexagon or rounded rhombus), 18–24px, filled with HP Teal `#2AA8C4`. Interior cel-shade: cool lighter teal `#5ACFE0` highlight facet at top, base teal on body. Bold Ivory Heavy numeral centered. HP teal globally reserved; must not appear elsewhere in the UI system.

**Art Bible Anchors:**
- §3.2 Card Frame Anatomy: HP below ATK, top-right, teal/blue gem — teal always
- §4.1 Primary Palette: HP Teal `#2AA8C4` — stat only, globally reserved
- §9.2 Color Prohibitions: never reuse HP teal for any other UI meaning

**Generation Prompt:**
Ankama cel-shaded HP stat gem, rounded rhombus or elongated hexagon, 24px. Fill: HP teal (#2AA8C4) base, lighter cool teal (#5ACFE0) highlight at top facet. No interior stroke. Ivory Heavy numeral centered. Negative: no green tint (Cra class forbidden), no blue-grey desaturation, no glow halo.

**Engine Notes:**
Identical implementation to ASSET-087. Numeral via `Text` child.

**State Variants:** Single `default` variant.

---

## ASSET-059 — Type/Rarity Icon

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 24×24 px (minimum 18×18 px render floor at 10-card overlap) |
| Atlas | `atlas_ui_hud` |
| Format | PNG-32 straight alpha |
| Naming | `ui_icon_[type]_[rarity]_default_hud.png` |
| Status | Needed |

**Visual Description:**
An icon at the bottom corner of the card face. Flat Ankama-style: 2px Void `#0D0D14` outer stroke, flat filled interior. Six card type silhouettes: sword (Minion), cog (Trap), tower (Structure), starburst (Spell), spreading wave (Field), lightning bolt (Instant). Rarity via interior fill: common = Ivory `#F7F0DC`; uncommon = Arcane Gold `#F5C842`; rare = Prism White `#EEF4FF`.

**Art Bible Anchors:**
- §1 Visual Identity: identifiable by shape alone at 18px — toughest scale test in the set
- §7.5 Iconography: 2px Void outline, flat interior fill; "if unidentifiable at 24px, becomes text label"
- §4.1 Primary Palette: Arcane Gold for uncommon; Prism White for rare; Ivory for common

**Generation Prompt:**
Ankama-style flat card-game type icon, 18×18px centered on 24×24 canvas, 2px void black stroke (#0D0D14), flat interior fill. Six type variants: Minion = crossed-swords; Trap = cog/gear; Structure = tower; Spell = starburst; Field = spreading wave; Instant = lightning bolt. Common: Ivory (#F7F0DC). Uncommon: Arcane Gold (#F5C842). Rare: Prism White (#EEF4FF). Negative: no detailed illustration, no glow, no border-radius pill.

**Engine Notes:**
One atlas frame per type/rarity combination in the current implemented pool. Current `assets/data/cards.json` production variants:

| Variant | Cards |
|---|---|
| Minion/Common | Iop Knight |
| Minion/Uncommon | Sacrier Footsoldier |
| Spell/Rare | Piercing Shot |
| Trap/Epic | Time Trap |
| Structure/Common | Sturdy Gobball |
| Field/Legendary | Sadida Rose Field |
| Order/Rare | Ecaflip's Decree |
| DoubleFace/Uncommon | Double-Face Blade |

Full catalog expansion is deferred until roster/card IDs/art IDs are reconciled.

**State Variants:** Eight current type/rarity frames, plus future frames when the full catalog is reconciled.

---

## ASSET-060 — DRAFT_INITIAL Grid Panel

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | ~420×560 px (3×120 + 2×20px gutters + 30px padding each side) |
| Atlas | None (procedural — decision 2026-04-30) |
| Format | No art file |
| Naming | N/A — Bevy 0.18 Node styling |
| Status | Code task |

**Visual Description:**
A centered overlay panel with dark `#0D1830` fill, 2px Arcane Gold `#F5C842` border, 12px corner radius. Behind: 70% opacity Ink Blue `#1A2D5A` full-screen backdrop. No inner glow, no emboss, no drop shadow.

**Engine Notes:**
Bevy 0.18 Node: `background_color`, `border`, `border_color: #F5C842`, `border_radius: 12px`. Full-screen backdrop: separate Node at 70% opacity. No PNG.

---

## ASSET-061 — Grid Slot Cell

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 120×180 px per state frame |
| Atlas | `atlas_ui_hud` |
| Format | PNG-32 straight alpha |
| Naming | `ui_slot_grid_available_hud.png` · `ui_slot_grid_pending_hud.png` · `ui_slot_grid_empty_hud.png` |
| Status | Needed |

**Visual Description:**
A 120×180 px cell container in the 3×3 DRAFT_INITIAL grid. Available state: full opacity neutral border frame. Pending state: card dimmed to ~60% opacity, 1px Arcane Gold `#F5C842` inner cell edge. Empty/purchased state: flat `#0D1830` recessed rectangle with 1px `#1A2D5A` inset border. No sold-out visual — empty and purchased are identical by design.

**Art Bible Anchors:**
- §3.4 UI Shape Grammar: geometric/angular cell shapes; flat and structured
- §4.1 Primary Palette: Arcane Gold as pending-state indicator; Ink Blue for empty slot
- §9.3 Structural Prohibitions: no red/invalid state on cells

**Generation Prompt:**
Ankama-style card slot cell, 120×180px. Three states: (1) Available — subtle neutral border frame; (2) Pending — 1px Arcane Gold (#F5C842) inner border; (3) Empty — flat dark recessed rectangle (#0D1830, 1px #1A2D5A inset). No sold-out marker, no red indicator. Negative: no green signal, no outer glow.

**Engine Notes:**
9 pre-pooled grid slot entities. `GridSlotState` component drives frame swap. `HandFullLocked` → ASSET-072 overlay composited over `available` frame.

**State Variants:** `available`, `pending`, `empty` — 3 separate atlas frames.

---

## ASSET-062 — Grid Slot Empty Checkmark

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 32×32 px |
| Atlas | `atlas_ui_hud` |
| Format | PNG-32 straight alpha |
| Naming | `ui_icon_checkmark_faint_hud.png` |
| Status | Needed |

**Visual Description:**
A faint Ivory `#F7F0DC` checkmark glyph at approximately 40–50% opacity (baked into PNG alpha). Appears centered in a purchased grid slot for 500ms post-S2CCardAcquired, then hides. Angular two-segment checkmark, 2px stroke, no fill.

**Art Bible Anchors:**
- §7.5 Iconography: flat outlined style, 2px stroke; identifiable at small scale
- §4.1 Primary Palette: Ivory for confirmation glyphs

**Generation Prompt:**
Ankama-style checkmark glyph, 20×20px centered on 32×32 canvas. Ivory (#F7F0DC) at ~40% baked alpha. 2px Ivory stroke. No fill. Angular checkmark: two straight line segments, sharp angle at junction. Negative: no green color, no rounded tick, no background badge, no shadow.

**Engine Notes:**
`ImageNode` child of grid slot entity. `Visible` on S2CCardAcquired; hidden after 500ms via `Timer`. Hard visibility toggle — no tweening.

**State Variants:** Single `default`. Alpha baked.

---

## ASSET-063 — Submit Button

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 128×40 px |
| Atlas | `atlas_ui_hud` |
| Format | PNG-32 straight alpha |
| Naming | `ui_button_submit_active_hud.png` · `ui_button_submit_inactive_hud.png` |
| Status | Needed |

**Visual Description:**
An angular horizontal chip — flat beveled-end shape per §3.4 HUD chip language. Active state: Ink Blue `#1A2D5A` face, Ivory `#F7F0DC` Heavy label. Inactive/submitted state: darkened to near-black, label "Submitted" at ~50% chroma. No rounded pill — angular chip with beveled (angled-cut) ends is the strict §3.4 requirement.

**Art Bible Anchors:**
- §3.4 UI Shape Grammar: HUD elements are flat angular chips — beveled ends, NOT rounded
- §4.1 Primary Palette: Ink Blue as UI background; Ivory for label
- §7.7 UX Constraints: minimum 44×44 CSS px pointer target

**Generation Prompt:**
Ankama HUD chip button, 128×40px, angular horizontal pill with beveled (angled-cut) ends — NOT rounded. Active state: Ink Blue (#1A2D5A) fill, 2px Void (#0D0D14) outline, Ivory (#F7F0DC) Heavy label "Submit (2 cards)" centered. Inactive state: darkened fill, desaturated text, label "Submitted". Negative: no rounded pill, no inner glow.

**Engine Notes:**
Bevy 0.18 `Button` Node with `ImageNode` background. Label text is `Text` child. Two atlas frames: `active`, `inactive`.

**State Variants:** `active`, `inactive`. ADVISORY: `hover` variant for polish.

---

## ASSET-064 — Submit Pre-Validation Error Label

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | Text Node (no sprite) |
| Atlas | None |
| Format | No art file |
| Naming | N/A |
| Status | Code task |

**Visual Description:**
Crimson `#9C2000` Heavy-weight text beneath the Submit button on validation failure. Content: `"Reserve overdrawn"` / `"Mana overdrawn"` / `"Out-of-range placement"`. Minimal 40% Ink Blue backing. Single line, non-modal.

**Engine Notes:**
Bevy 0.18 `Text` Node with `TextColor` Crimson `#9C2000`. `SubmitValidationError` marker drives content string. Backing: procedural Node at 40% opacity.

---

## ASSET-065 — Placement Timer Panel

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 64×40 px (minimum 60×36 px rounded to even integers) |
| Atlas | None (procedural Node) |
| Format | No art file |
| Naming | N/A |
| Status | Code task |

**Visual Description:**
Semi-opaque Ink Blue `#1A2D5A` at 40% opacity backing, flat angular chip. Interior: single large Heavy numeral (whole seconds). Urgency steps: Ivory → Amber `#E87C1E` → Crimson `#9C2000` per second from 5s remaining.

**Art Bible Anchors:**
- §7.4 Typography: Heavy weight — timer is an action input
- §7.7 UX Constraints: must have semi-opaque background; never over animated board content

**Engine Notes:**
Bevy 0.18 Node with `background_color: Color::srgba(0.10, 0.18, 0.35, 0.40)`. `Text` child; `TextColor` updated each second. Per-second urgency pulse via `bevy_tweening` `Lens<Transform>`.

---

## ASSET-066 — Timer Checkmark Glyph

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 20×20 px |
| Atlas | `atlas_ui_hud` |
| Format | PNG-32 straight alpha |
| Naming | `ui_icon_checkmark_submit_hud.png` |
| Status | Needed |

**Visual Description:**
Ivory `#F7F0DC` checkmark, 20×20 px, full opacity. Angular two-segment glyph (not rounded tick), 2px stroke, no fill. Appears left of the timer numeral after Submit press. Distinct from ASSET-062 (faint grid checkmark) — full opacity, different context.

**Art Bible Anchors:**
- §4.1 Primary Palette: Ivory for confirmation glyphs
- §7.5 Iconography: flat outlined, 2px stroke, no fill; identifiable at 20px

**Generation Prompt:**
Ankama-style confirmation checkmark glyph, 20×20px, Ivory (#F7F0DC), fully opaque. 2px Ivory stroke, no fill. Angular checkmark: two straight line segments. Positioned left of timer numeral inside HUD panel. Negative: no rounded tick, no green, no circle background, no shadow.

**Engine Notes:**
`ImageNode` child of timer panel Node. `Visibility::Hidden` by default; `Visible` on C2SSubmitPlacement sent. Persists until RESOLUTION entry.

**State Variants:** Single `default`. Full Ivory opacity.

---

## ASSET-067 — Reserve Mana Split Strip

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | Strip: 96×24 px. Buttons (×2): 24×24 px each. Center display: 48×24 px. |
| Atlas | `atlas_ui_hud` (buttons only; strip background is procedural) |
| Format | PNG-32 straight alpha |
| Naming | `ui_button_reserve_minus_[state]_hud.png` · `ui_button_reserve_plus_[state]_hud.png` |
| Status | Needed |

**Visual Description:**
A 96×24 px horizontal strip anchored 8px above each staged fan ghost slot. Strip background: Ink Blue `#1A2D5A` 70% opacity, 4px corner radius. Left: 24×24 px `[−]` button — Ivory glyph active; Prism White `#EEF4FF` on hover; 30% chroma disabled. Center: 48×24 px Ivory Heavy numeric display `[N / cost]`. Right: 24×24 px `[+]` button with same states. Hidden for `cost == 0` cards.

**Art Bible Anchors:**
- §3.4 UI Shape Grammar: angular chip geometry; 4px radius
- §4.1 Primary Palette: Ink Blue backing; Ivory glyphs; Prism White hover
- §7.4 Typography: Heavy weight for numeric display

**Generation Prompt:**
Ankama HUD minus/plus button, 24×24px. Ink Blue (#1A2D5A) fill, 1px Void outline. Active: Ivory (#F7F0DC) minus/plus glyph (14px Heavy). Hover: glyph brightens to Prism White (#EEF4FF). Disabled: glyph at 30% chroma. Three states per button. Negative: no pill shape, no gradient, no drop shadow.

**Engine Notes:**
Strip background: procedural Node (no PNG). `[ − ]` and `[ + ]`: bevy_ui `Button` Nodes with `ImageNode` backgrounds (3 frames each). Numeric display: `Text` child. ⚠️ FLAG: verify `Interaction::Hovered` in WASM WebGL2 with Bevy 0.18 Input API changes.

**State Variants:** `minus_active`, `minus_hover`, `minus_disabled` · `plus_active`, `plus_hover`, `plus_disabled` (6 frames).

---

## ASSET-068 — "Auction in Progress" Label

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | Text Node (~200×28 px, content-driven) |
| Atlas | None |
| Format | No art file |
| Naming | N/A |
| Status | Code task |

**Visual Description:**
`"Auction in progress"` in Ivory `#F7F0DC`, 12px Regular weight, on a 40% opacity Ink Blue backing. Floats above the hand fan during PASSIVE_LOCKED. Subdued and informational — not urgent.

**Engine Notes:**
Bevy 0.18 `Text` Node, 12px, Ivory. Backing: procedural Node at 40% opacity. `Visibility::Hidden` outside PASSIVE_LOCKED. Hard show/hide.

---

## ASSET-069 — Hand Full Notification

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | Text Node (~180×32 px, content-driven) |
| Atlas | None |
| Format | No art file |
| Naming | N/A |
| Status | Code task |

**Visual Description:**
`"Hand full"` in Ivory `#F7F0DC` Heavy weight, appearing near the fan for 2 seconds. Neutral informational register — Ink Blue/Ivory (not Crimson, not Amber). Angular chip styling per §3.4.

**Engine Notes:**
Pre-pooled entity with `HandFullNotification` marker. Set `Visible` on hand-full event; hidden after 2000ms via `Timer`. ⚠️ HU-30 says "spawned/despawned" — recommend pre-pooling for consistency with Rule 1 anti-churn principle. Flag to gameplay-programmer.

---

## ASSET-070 — "No Valid Targets" Overlay

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | Covers full board area (bounds from `Res<BoardLayout>`) |
| Atlas | None |
| Format | No art file |
| Naming | N/A |
| Status | Code task |

**Visual Description:**
Full-board semi-opaque dark overlay (~80% opacity Ink Blue `#1A2D5A`) with centered `"No valid targets"` text chip. Board remains ghost-visible — not a blackout. No red tint per §9.2.

**Art Bible Anchors:**
- §4.1 Primary Palette: Ink Blue as suppression layer
- §9.2 Color Prohibitions: no red for invalid states
- §9.3 Structural Prohibitions: must not occlude the gold counter

**Engine Notes:**
`NoValidTargetsOverlay` marker entity. bevy_ui Node with absolute positioning matching board bounds from `Res<BoardLayout>`. `background_color: Color::srgba(0.10, 0.18, 0.35, 0.80)`. `Text` child, Heavy Ivory, centered.

---

## ASSET-071 — Fan Ghost Slot (Staged Card)

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 120×180 px (same canvas as ASSET-082 — no separate sprite) |
| Atlas | None |
| Format | Shader/WGSL |
| Naming | `assets/shaders/card_ghost.wgsl` |
| Status | Shader task |

**Visual Description (shader reference):**
ASSET-082's atlas frame rendered with a custom `UiMaterial` applying desaturation to 40% chroma (`desaturate: 0.40`) and 50% opacity (`opacity: 0.50`). No tint — desaturation alone is the "committed" signal per VA-2. Card identity remains readable by tonal silhouette. Transition is a hard swap (no tweening).

**Art Bible Anchors:**
- §1 Visual Identity: desaturation alone; tonal silhouette must survive at 40% chroma
- §7.6 Animation Feel: hard swap confirms state change; no idle pulse on ghost

**Engine Notes:**
`FanSlotState::Ghost` triggers swap from `ImageNode` to `MaterialNode<CardGhostMaterial>`. ⚠️ **FLAG-1 (HIGH)**: `UiMaterial`/`MaterialNode` API must be verified against 0.17→0.18 migration guide.

---

## ASSET-072 — Hand-Full Grid Lock Overlay

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 120×180 px per grid slot (overlay per locked slot) |
| Atlas | None |
| Format | Shader/WGSL (shared with ASSET-071 or dedicated `GridLockMaterial`) |
| Naming | Shared `assets/shaders/card_ghost.wgsl` with distinct uniforms |
| Status | Shader task |

**Visual Description (shader reference):**
`GridSlotState::HandFullLocked` applies: 30% chroma desaturation (`desaturate: 0.30`) plus Ink Blue `#1A2D5A` tint at 40% opacity (`tint_color: #1A2D5A, tint_strength: 0.40`). Combined: cold, muted, clearly non-interactive.

**Art Bible Anchors:**
- §4.1 Primary Palette: Ink Blue as lock/suppression tint
- §1 Visual Identity: 30% chroma floor — tonal silhouette must remain readable

**Engine Notes:**
Recommend shared parameterized material from ASSET-071 with additional `tint_color` and `tint_strength` uniforms. ⚠️ **FLAG-1**. ⚠️ Up to 8 slots locking simultaneously → structural ECS archetype-move overhead — profile at implementation.

---

## ASSET-073 — Card Purchase Bloom Flash

| Field | Value |
|-------|-------|
| Category | VFX |
| Dimensions | 128×192 px |
| Atlas | `atlas_vfx` |
| Format | PNG-32 straight alpha |
| Naming | `vfx_flash_purchase_default_vfx.png` |
| Status | Needed |

**Visual Description:**
A 60ms Arcane Gold `#F5C842` bloom flash filling the grid slot cell area on C2SPurchaseCard send. Fast, full-cell warm gold wash — high brightness, zero texture — appearing at frame 0 and fading by frame 3 (60ms at 60 FPS). No particle emission, no radiating ring — flat warm-gold luminance burst.

**Art Bible Anchors:**
- §4.1 Primary Palette: Arcane Gold for rewards/premium events — purchase confirmation
- §7.6 Animation Feel: 60ms fast confirm; no sustain, no elastic settle

**Generation Prompt:**
Ankama-style purchase confirmation bloom, 128×192px. Single frame of warm Arcane Gold (#F5C842) luminance burst — uniform flat gold wash, high brightness, zero texture. Shown at peak brightness (alpha 1.0; fades to 0.0 over 60ms). No radiating ring, no particle emission. Reference: camera flash in gold, flat. Negative: no starburst, no lens flare, no ring expansion, no secondary color.

**Engine Notes:**
Pre-pooled VFX overlay entity. 60ms fade via `bevy_tweening` `Lens<BackgroundColor>`. Single-frame preferred — no sprite sheet needed.

**State Variants:** Single one-shot flash. 60ms bevy_tweening fade.

---

## ASSET-074 — Card Hover Gold Outline Pulse

| Field | Value |
|-------|-------|
| Category | VFX |
| Dimensions | Shader (applied to hovered card's material) |
| Atlas | None |
| Format | WGSL shader — embedded in card hover `UiMaterial` |
| Naming | N/A |
| Status | Shader task |

**Visual Description (shader reference):**
2px Arcane Gold `#F5C842` outline tracing the hovered card perimeter. Pulses at 1Hz — brightening to full `#F5C842` and dimming to ~40% opacity on alternate half-cycles. Secondary GPU outline pass above baked Void outline. 1Hz = "selected/interactive," not urgent.

**Art Bible Anchors:**
- §8.4 Outline Technique: secondary GPU outline pass permitted for selection-state highlight
- §4.1 Primary Palette: Arcane Gold for premium UI chrome — hover selection

**Engine Notes:**
Custom `UiMaterial` with time uniform for 1Hz sine-wave opacity animation and exterior SDF outline. Not bevy_tweening (continuous periodic). ⚠️ **FLAG-1**.

---

## ASSET-075 — Fan Plate Prism White Border Glow

| Field | Value |
|-------|-------|
| Category | VFX |
| Dimensions | Shader (covers full fan plate Node) |
| Atlas | None |
| Format | WGSL shader uniform on fan plate `UiMaterial` |
| Naming | `assets/shaders/fan_plate.wgsl` |
| Status | Shader task |

**Visual Description (shader reference):**
3px Prism White `#EEF4FF` border glow at 60% opacity, pulsing at 0.5Hz on the fan plate during Instant card drag. Plate background brightens to `#1E2A3A`. 0.5Hz = slow, inviting rhythm — "inviting not urgent" per VA-7. Active during `FanPlateHighlighted` state.

**Art Bible Anchors:**
- §4.1 Primary Palette: Prism White for magical effects, high-value events
- §7.6 Animation Feel: 0.5Hz = "slow pulse signals inviting not urgent" per VA-7

**Engine Notes:**
`FanPlateHighlighted` marker triggers swap to custom `UiMaterial` with border glow + 0.5Hz time uniform. ⚠️ **FLAG-1**.

---

## ASSET-076 — Fan Plate Staged Gold Flash

| Field | Value |
|-------|-------|
| Category | VFX |
| Dimensions | Shader (same fan plate Node as ASSET-075) |
| Atlas | None |
| Format | WGSL uniform on ASSET-075 material |
| Naming | Shared `assets/shaders/fan_plate.wgsl` |
| Status | Shader task |

**Visual Description (shader reference):**
On Instant card staged (HU-19): border color shifts from Prism White `#EEF4FF` → Arcane Gold `#F5C842` (instant) → back to Prism White over 80ms. One-shot confirmation beat before the 0.5Hz glow resumes.

**Art Bible Anchors:**
- §4.1 Primary Palette: Arcane Gold — reward/significance confirmation
- §7.6 Animation Feel: 80ms flash — fast confirm; within 250ms PLACEMENT hard cap

**Engine Notes:**
One-shot `bevy_tweening` tween on border_color uniform: `#EEF4FF` → `#F5C842` (0ms) → `#EEF4FF` (80ms). Appropriate use of bevy_tweening (finite one-shot).

---

## ASSET-077 — TargetUnit Hover Outline

| Field | Value |
|-------|-------|
| Category | VFX |
| Dimensions | Shader (applied to 64×96 px world-space unit sprites) |
| Atlas | None |
| Format | `Material2d` WGSL |
| Naming | `assets/shaders/unit_target_outline.wgsl` |
| Status | Shader task |

**Visual Description (shader reference):**
2px Prism White `#EEF4FF` outline on valid-target world-space unit sprites during TargetUnit card drag hover. Pulses at 2Hz. 2Hz = faster than fan plate glow (0.5Hz), categorically distinct from urgency. Applied per `TargetUnitHover` marker component.

**Art Bible Anchors:**
- §4.1 Primary Palette: Prism White for magical/targeting register
- §8.4 Outline Technique: secondary GPU outline pass for selection-state highlight
- §7.6 Animation Feel: 2Hz = designated TargetUnit hover frequency per VA-4

**Engine Notes:**
World-space `Material2d` (not `UiMaterial`) with SDF exterior outline, 2Hz time uniform. Material swap on `TargetUnitHover` → default on drag-end. ⚠️ **FLAG-3 (HIGH)**: `Material2d` API changes 0.14→0.18 are post-cutoff — verify against engine reference.

---

## ASSET-078 — Card Lift SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | 60–80 ms |
| Format | OGG Vorbis (WASM delivery) / WAV (authoring master) |
| Naming | `sfx_card_lift_default.ogg` |
| Channel | `ui_hand` · Mono · 44100 Hz |
| Status | Needed |

**Sonic Character:**
Crisp, papery pick-up transient, 60–80ms, no reverb tail. High-frequency consonant like a single playing card lifted from a wooden table, brief air displacement component. 0ms attack, decay below audible within 80ms. Physical/tactile texture — card stock, not magical. No musical pitch center.

---

## ASSET-079 — Valid Targets Appear SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | ~100 ms |
| Format | OGG Vorbis / WAV |
| Naming | `sfx_targets_appear_default.ogg` |
| Channel | `ui_hand` · Mono · 44100 Hz |
| Status | Needed |

**Sonic Character:**
Subtle crystalline shimmer, ~100ms, nearly subliminal. Upper frequency range — glass-like. Triggered once per drag gesture (not per cell). Cool register, not warm.

**Engine Notes:**
⚠️ Only play when valid target set is non-empty at drag-start. Do not trigger on TargetUnit drags with no valid units.

---

## ASSET-080 — Successful Stage SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | 80–120 ms |
| Format | OGG Vorbis / WAV |
| Naming | `sfx_card_stage_default.ogg` |
| Channel | `ui_hand` · Mono · 44100 Hz |
| Status | Needed |

**Sonic Character:**
Weighted "thunk" or placement click, warm and physical, 80–120ms. Dense mid-frequency center — wooden chess piece placed on stone. Brief low-frequency body. Heavier than card-lift — asymmetry communicates irreversible commitment. No reverb. Warm tone is the reward signal.

---

## ASSET-081 — Snap-Back / Invalid Drop SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | ~80 ms |
| Format | OGG Vorbis / WAV |
| Naming | `sfx_card_snapback_default.ogg` |
| Channel | `ui_hand` · Mono · 44100 Hz |
| Status | Needed |

**Sonic Character:**
Soft quick whoosh-back, ~80ms — shorter and quieter than ASSET-080. Low-mid frequency, no hard consonant at attack. Air-movement transient communicating physical reversal, not failure or penalty. No error/buzzer tone.

---

## ASSET-082 — Instant Card Staged SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | ~100 ms |
| Format | OGG Vorbis / WAV |
| Naming | `sfx_card_instant_stage_default.ogg` |
| Channel | `ui_hand` · Mono · 44100 Hz |
| Status | Needed |

**Sonic Character:**
ASSET-080 base register (weighted thunk, 80–120ms) plus a brief crystalline overtone (~20ms cool-register resonance after the thunk). The overtone is audible (not subliminal). Combined: "successful stage + magical confirmation." Total ~100ms.

---

## ASSET-083 — Submit SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | ~400 ms (leading click transient + decaying ring) |
| Format | OGG Vorbis / WAV |
| Naming | `sfx_submit_default.ogg` |
| Channel | `ui_hand` · Mono · 44100 Hz |
| Status | Needed |

**Sonic Character:**
Sharp leading click + resonant ring decaying over ~400ms. Ring implies permanence and finality. Natural decay, no added reverb. **Must be audible at low browser volume** (VA-8 mandatory gate). Do NOT play on pre-validation failure.

---

## ASSET-084 — Timer Urgency SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | ~300–500 ms |
| Format | OGG Vorbis / WAV |
| Naming | `sfx_timer_urgency_default.ogg` |
| Channel | `ui_hand` · Mono · 44100 Hz |
| Status | Needed |

**Sonic Character:**
Single heartbeat-register tone fired exactly once at the 5-second mark. Low-mid frequency pulse, soft attack, ~300ms decay. **Hard constraint: no loop, no tick sequence.** One cue, one play. Must use non-looping playback (`PlaybackSettings::ONCE` or Bevy 0.18 equivalent).

---

## ASSET-085 — Card Acquired SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | ~150 ms |
| Format | OGG Vorbis / WAV |
| Naming | `sfx_card_acquired_default.ogg` |
| Channel | `ui_hand` · Mono · 44100 Hz |
| Status | Needed |

**Sonic Character:**
Light ascending two-note chime, ~150ms. Two notes a major second or minor third apart — small upward step, positive confirming without fanfare. Bright metallic timbre, fast attack, short natural decay. Must remain pleasant on up to 10 repetitions in DRAFT_INITIAL.

---

## ASSET-086 — Hand Full SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | ~200 ms |
| Format | OGG Vorbis / WAV |
| Naming | `sfx_hand_full_default.ogg` |
| Channel | `ui_hand` · Mono · 44100 Hz |
| Status | Needed |

**Sonic Character:**
Soft neutral bell tone, single strike, ~200ms. Mid-frequency, round, non-directional. "Informs rather than scolds." Notably softer and lower-fundamental than ASSET-085. No harmonic ascent. Single neutral pitch fading naturally.

---

## ASSET-087 — Reserve Adjust Click SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | ~50 ms |
| Format | OGG Vorbis / WAV |
| Naming | `sfx_reserve_adjust_default.ogg` |
| Channel | `ui_hand` · Mono · 44100 Hz |
| Status | Needed |

**Sonic Character:**
Soft mid-register click, ~50ms, no reverb. Plastic or soft-resin button depression at low dynamic volume. **Must not fatigue on rapid repetition** — player may click +/− multiple times in quick succession. Do NOT play when button is in Disabled state.
