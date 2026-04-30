# Asset Specs — System: Shop / Auction UI

> **Source**: design/gdd/shop-auction-ui.md
> **Art Bible**: design/art/art-bible.md
> **Generated**: 2026-04-30
> **Status**: 21 assets specced / 21 approved / 0 in production / 0 done

---

## Cross-Cutting Technical Flags

Resolve these before any art production begins. Several asset dimensions and atlas assignments depend on answers from the engine programmer.

| # | Flag | Assets Affected | Resolution Path |
|---|---|---|---|
| 1 | `ImageNode` stretch/tile mode in Bevy 0.18 — verify `ImageScaleMode` API exists | 006, 007 | Check 0.15–0.18 migration guides |
| 2 | 9-slice support in `bevy_ui` 0.18 — if confirmed, border tiles drop to ~32px (near-zero budget) | 007 | Check 0.18 release notes |
| 3 | Particle system API — native Bevy 0.18 vs. third-party crate; confirm sprite-sheet UV indexing method | 008 | Engine review |
| 4 | Additive blend mode for `Sprite` — built-in `Sprite::color` field vs. custom `AlphaMode::Add` material | 009 | Check Bevy 0.18 `Sprite` component docs |
| 5 | `bevy_audio` OGG Vorbis support + `LOOPSTART`/`LOOPEND` loop metadata parsing | 012–021 | Check `bevy_audio` 0.18 feature set |
| 6 | UX spec panel pixel dimensions (GDD OQ4 unresolved) — governs canvas sizes for panel bg and border tiles | 006, 007, 010 | Run `/ux-design shop-auction-ui` |

---

## ASSET-001 — Gold Coin Icon

| Field | Value |
|-------|-------|
| Category | UI Icon / Sprite |
| Dimensions | 48×48px master → downsampled to 24×24px HUD variant. Both even integers. Never upscale. |
| Format | PNG-32 RGBA, straight alpha, strip ICC profile |
| Naming | `ui_gold_coin_default_48.png` / `ui_gold_coin_default_24.png` |
| Texture Res | 48px master (largest tier per §8.1 → downsample) |
| Atlas | `atlas_ui_hud` |
| Padding | 2px transparent gutter on all sides within atlas |
| LOD | None — 2D UI icon; two discrete size variants |
| Status | Needed |

**Visual Description:**
A 24–48px flat circular coin viewed at a slight isometric tilt (~15°) so it reads as a solid object rather than a flat disc. The face carries a bold inset star or crown glyph in Void `#0D0D14`, the coin body filled Arcane Gold `#F5C842` with a single cel-shade highlight arc across the upper-left quadrant in brighter warm yellow (`#FFE07A`) and a hard lower shadow strip in deep amber (`#B8941A`). At 24px every interior detail reduces to pure silhouette — the oval tilt and single highlight are the only surviving reads.

**Art Bible Anchors:**
- §1 Visual Identity Statement: bold Void outline, saturated local color, holds visual weight at 24px
- §3.4 UI Shape Grammar: lives inside angular Ink Blue chip alongside Heavy Arcane Gold numerals — must not compete with the numeral for brightness
- §4.1 Primary Palette: Arcane Gold `#F5C842`; Void `#0D0D14` for outline
- §7.5 Iconography Style: 2px Void outline, flat interior fill — standard outlined icon (not gem shape treatment)

**Generation Prompt:**
`Single gold coin icon, isometric tilt 15 degrees, Ankama/Krosmaga cel-shade style, flat circular coin body filled Arcane Gold #F5C842, single cel-shade highlight arc upper-left in bright warm yellow, hard lower-edge shadow strip deep amber, bold star glyph inset on face in near-black void color, 2px void outline bounding entire form, isolated transparent background, game UI icon style, bold graphic, no photorealism, no smooth gradients, no glow, no ambient occlusion -- negative prompt: photorealistic, 3D render, soft shadows, texture noise, ambient occlusion, glossy, decorative flourishes, Hearthstone style, flat vector no-outline, minimalist`

**Engine Constraints:**
- Both sizes packed into `atlas_ui_hud` (not standalone textures) to preserve sprite batching
- Displayed via `ImageNode` with `TextureAtlas` UV rect — verify `ImageNode` API in Bevy 0.18
- Color must match Arcane Gold `#F5C842` exactly; must not use ATK Orange `#E07020` or HP Teal `#2AA8C4` (globally reserved per §9.2)

**⚠️ Flag:** 24px falls below the §7.5 iconography scale lower bound of 32px. If the §1 silhouette test fails at 24px, constrain the HUD chip container and render the icon at 32px minimum.

---

## ASSET-002 — Rarity Gem — Rare

| Field | Value |
|-------|-------|
| Category | UI Badge / Sprite |
| Dimensions | 32×32px master → downsampled to 24×24px Card Display variant. Even integers. |
| Format | PNG-32 RGBA, straight alpha, strip ICC profile |
| Naming | `ui_gem_rare_default_32.png` / `ui_gem_rare_default_24.png` |
| Texture Res | 32px (Card Zoom use); 24px (Card Display use) |
| Atlas | `atlas_cards` |
| Padding | 2px transparent gutter on all sides within atlas |
| LOD | None — 2D UI badge |
| Status | Needed |

**Visual Description:**
A perfect circle gem badge with a two-stop radial gradient fill: Ink Blue `#1A2D5A` at the outer ring thinning to lighter sky-blue `#4A7AB5` at center, producing a glassy inward glow without any highlight dot. No Void outline — gem shape treatment per §7.5. A subtle inner-edge concentric ring in `#2A4D8A` separates the outer color zone from the center brightening. Reads as a smooth polished stone, not a faceted crystal. The circular silhouette is its colorblind-safety differentiator from Epic (diamond) and Legendary (star).

**Art Bible Anchors:**
- §3.4 UI Shape Grammar: circle distinguishes Rare from Epic and Legendary by geometry alone
- §4.6 Colorblind Safety: shape-only differentiator mandatory — color is secondary confirmation
- §7.5 Iconography Style: gem shape treatment — 2-color gradient fill, NO Void outline exception applies here

**Generation Prompt:**
`Rarity gem badge, circular shape, game UI badge, Ankama cel-shade style, two-stop radial gradient Ink Blue #1A2D5A outer to lighter sky blue #4A7AB5 center, subtle concentric inner ring #2A4D8A, no outline, polished smooth stone not faceted crystal, isolated transparent background, small circular jewel badge for card game, clean rendering -- negative prompt: outline, border, photorealistic, ambient occlusion, 3D render, faceted gem, sparkle, glow, warm tones`

**Engine Constraints:**
- Must share `atlas_cards` with card frames — same draw call avoids texture swap per card render
- 24px variant: baked 2px Void outline per §8.4 Card Display line weight

**⚠️ Flag:** Confirm the Rare gem blue is distinct from HP Teal `#2AA8C4` and Ink Blue `#1A2D5A` — both are globally reserved semantic colors per §9.2.

---

## ASSET-003 — Rarity Gem — Epic

| Field | Value |
|-------|-------|
| Category | UI Badge / Sprite |
| Dimensions | 32×32px master → 24×24px Card Display variant. Even integers. |
| Format | PNG-32 RGBA, straight alpha, strip ICC profile |
| Naming | `ui_gem_epic_default_32.png` / `ui_gem_epic_default_24.png` |
| Texture Res | 32px / 24px |
| Atlas | `atlas_cards` |
| Padding | 2px transparent gutter on all sides within atlas |
| LOD | None — 2D UI badge |
| Status | Needed |

**Visual Description:**
A rotated-square (diamond orientation, point up and down) gem badge with radial gradient from deep purple `#6B35A0` at outer edges to vivid lavender-violet `#A060D8` at center. No Void outline. The diamond silhouette is the colorblind differentiator from Rare (circle) and Legendary (star/octagon). A single faint cel-shade highlight facet line across the upper-left face — a 10–15% brightness shift in the same purple hue, not a new color — reinforces the cut-stone read.

**Art Bible Anchors:**
- §3.4 UI Shape Grammar: diamond shape distinguishes Epic from Rare and Legendary by geometry alone
- §4.6 Colorblind Safety: shape-only differentiator mandatory
- §9.2 Color Prohibitions: Epic purple must not bleed into Xelor class `#2E1B6E` — Epic center `#A060D8` is well above Xelor's dark anchor in brightness

**Generation Prompt:**
`Rarity gem badge, rotated diamond shape pointing up and down, game UI badge, Ankama cel-shade style, two-stop radial gradient deep purple #6B35A0 outer to vivid lavender-violet #A060D8 center, single faint facet highlight line upper-left as brightness shift in same hue not new color, no outline, polished cut-gem, isolated transparent background -- negative prompt: outline, border, photorealistic, ambient occlusion, 3D render, sparkle, glow halo, warm tones, blue tones, Hearthstone style`

**Engine Constraints:**
- Same `atlas_cards` co-residency requirement as ASSET-002
- Baked 2px Void outline at Card Display 24px per §8.4

**⚠️ Flag:** Epic purple must be reviewed against Xelor class color `#2E1B6E` to confirm they are distinguishable at 24px before production pass.

---

## ASSET-004 — Rarity Gem — Legendary

| Field | Value |
|-------|-------|
| Category | UI Badge / Sprite |
| Dimensions | 32×32px master → 24×24px Card Display variant. Even integers. |
| Format | PNG-32 RGBA, straight alpha, strip ICC profile |
| Naming | `ui_gem_legendary_default_32.png` / `ui_gem_legendary_default_24.png` |
| Texture Res | 32px / 24px |
| Atlas | `atlas_cards` |
| Padding | 2px transparent gutter on all sides within atlas |
| LOD | None — 2D UI badge |
| Status | Needed |

**Visual Description:**
An eight-pointed star (or symmetrical octagonal gem) badge with radial gradient from Arcane Gold `#F5C842` at outer star points to near-Prism White `#EEF4FF` at center. No Void outline. Two subtle cel-shade facet lines run diagonally — lighter than center in Prism White, creating a multi-faceted luminous read. At 24px the eight-point read reduces to a bright starburst flare — acceptable; structural complexity is the premium tier signal.

**Art Bible Anchors:**
- §1 Silhouette First: star/octagon = most complex silhouette, reserved for highest rarity
- §4.1 Primary Palette: Arcane Gold `#F5C842` + Prism White `#EEF4FF` — both semantically correct for premium/magical significance
- §4.6 Colorblind Safety: star/octagon distinguishable from circle (Rare) and diamond (Epic) at 24px by silhouette alone

**Generation Prompt:**
`Rarity gem badge, eight-pointed star or symmetrical octagonal gem shape, game UI badge, Ankama cel-shade style, radial gradient Arcane Gold #F5C842 outer to near-white Prism White #EEF4FF center, two diagonal cel-shade facet lines as brightness variants within same palette, no outline, luminous premium jewel, isolated transparent background -- negative prompt: outline, border, photorealistic, 3D render, animated shimmer, Hearthstone foil, muted tones, round shape, diamond shape`

**Engine Constraints:**
- Same `atlas_cards` co-residency requirement as ASSET-002, ASSET-003

**⚠️ Flag:** Arcane Gold gem color may collide visually with the Arcane Gold mana cost badge on Legendary card frames. The two elements must be distinguishable by shape alone at 24px per §1 silhouette rule.

---

## ASSET-005 — DRAFT_SHOP Slot Well Highlight Strip

| Field | Value |
|-------|-------|
| Category | UI Background Texture |
| Dimensions | 120×8px. Both even integers. Non-POT frame is valid inside a POT atlas. |
| Format | PNG-32 RGBA, straight alpha, strip ICC profile |
| Naming | `ui_shop_slot_highlight_default_hud.png` |
| Texture Res | 120×8px (matches 120px Card Display slot width per §8.1) |
| Atlas | `atlas_ui_hud` |
| Padding | 2px transparent gutter on all sides within atlas |
| LOD | None — static 2D texture strip |
| Status | Needed |

**Visual Description:**
A narrow horizontal texture strip ~8px tall and 120px wide representing a soft, warm spotlight spill across the top edge of a card slot well. Graduates from warm Ivory-tinted highlight (`#F7F0DC` at ~70% opacity) at the topmost pixel, through warm amber-cream mid-tone (`#D4B87A` at ~40% opacity) across ~4–5px, then falling to full transparency at the lower edge — two distinct opacity bands baked as flat cel-shade steps, not a smooth airbrush. This is baked art, explicitly designated as such in the GDD (VA.1).

**Art Bible Anchors:**
- §2 DRAFT_SHOP Mood: "individual cards lit from above like a shopkeeper's display lamp" — this strip is the primary mood carrier at the slot level
- §4.1 Primary Palette: Ivory `#F7F0DC` warm highlight; warm amber-cream derived from parchment border `#D4AF72` used in DRAFT_SHOP panel spec
- §8.4 Outline Technique: baked into sprite — no runtime lighting pass
- §9.1 Style Prohibitions: no airbrush gradients; cel-shade step banding only

**Generation Prompt:**
`Horizontal highlight strip texture, top-edge spotlight spill, warm ivory cream light #F7F0DC at top, baked cel-shade warm lamp highlight, three discrete stepped opacity bands top to transparent bottom, warm shopkeeper display lamp from above, 120x8px canvas, no smooth gradient, no dynamic light, flat stepped opacity, isolated transparent background -- negative prompt: smooth airbrush gradient, photorealistic glow, cool light, neon, hard edge, outline, animated`

**Engine Constraints:**
- Pre-baked texture loaded via `ImageNode`; no shader dependency — Bevy 0.18 `BackgroundColor` renders flat color only, gradient requires baked texture
- 120×8px inside a POT atlas is valid — individual frame dimensions need only be even integers; the atlas sheet itself must be POT

---

## ASSET-006 — Auction Panel Background

| Field | Value |
|-------|-------|
| Category | UI Panel Background Texture |
| Dimensions | 512×512px provisional (POT). Confirm against UX spec panel container before production — see Flag. |
| Format | PNG-32 RGBA, straight alpha, strip ICC profile |
| Naming | `ui_auction_panel_bg_default_hud.png` |
| Texture Res | 512×512px |
| Atlas | `atlas_ui_hud` |
| Padding | 2px transparent gutter on all sides within atlas |
| LOD | None — 2D UI panel background |
| Status | Needed |

**Visual Description:**
A rectangular panel background filled with Ink Blue `#1A2D5A` as base. A cel-shade luminance gradient runs from a slightly lighter center-top zone (`#2A4470`, ~15–20% luminance above base) radiating downward and outward to the base, executed as 2–3 discrete stepped bands — not a smooth gradient. Overall read is cool, pressurized, receding. No gold, no warm tones, no decorative texture. The border color is a separate runtime element (ASSET-007); this texture is only the fill.

**Art Bible Anchors:**
- §2 DRAFT_AUCTION Mood: "cool-to-neutral; the money is the only warm thing left in the frame" — background must recede so gold elements dominate
- §1 Structured Luminance Hierarchy: background explicitly yields to the auction card and price counter as the luminance apex
- §9.1 Style Prohibitions: no Hearthstone warm-glow, no decorative noise, no photorealistic ambient occlusion

**Generation Prompt:**
`Panel background texture, dark Ink Blue #1A2D5A base, cel-shade stepped luminance gradient lighter at top-center #2A4470 radiating to dark base, 2-3 discrete bands not smooth gradient, cool pressurized atmosphere, Ankama cel-shade style game UI panel, no decorative texture, no warm tones, no gold, 512x512px, isolated no border -- negative prompt: warm tones, gold, smooth gradient, decorative pattern, glowing, Hearthstone style, textured noise, vignette`

**Engine Constraints:**
- Bevy 0.18 `BackgroundColor` renders flat color only — gradient requires this pre-baked texture via `ImageNode`
- ⚠️ Post-cutoff flag: Verify `ImageNode` `ImageScaleMode` or equivalent UV-stretch API in Bevy 0.18 before production. If stretch mode is unavailable, author at exact panel pixel dimensions (even integers, atlas-fitting).

**⚠️ Flag:** Panel pixel dimensions are unresolved (GDD OQ4). Author at 512×512px provisional. Revise to exact dimensions after `/ux-design shop-auction-ui` is run.

---

## ASSET-007 — Auction Panel Border Ramp Tiles (4 tiles)

| Field | Value |
|-------|-------|
| Category | UI Panel Border Overlay — 4 color variants |
| Dimensions | 256×256px per tile (transparent interior, 6px filled border ring). Even integers. 4 tiles × 0.25 MB = 1 MB total. |
| Format | PNG-32 RGBA, straight alpha, strip ICC profile |
| Naming | `ui_auction_border_tier1_hud.png` (Pale Ink Blue, 0–3g) · `ui_auction_border_tier2_hud.png` (Auction Amber, 4–6g) · `ui_auction_border_tier3_hud.png` (Deep Amber, 7–9g) · `ui_auction_border_tier4_hud.png` (Crimson-Amber, 10+g) |
| Texture Res | 256×256px per tile |
| Atlas | `atlas_ui_hud` |
| Padding | 2px transparent gutter on all sides within atlas |
| LOD | None — 2D UI border overlay |
| Status | Needed |

**Visual Description (geometry shared; only fill color changes across tiers):**
Each tile: 6px solid border ring around a fully transparent interior. A 1px inner cel-shade highlight (1px at ~40% white overlay on the innermost edge) suggests slight raised volume. The outer edge carries a 1px micro-shadow at deeper value. The four tier colors:
- **Tier 1 (0–3g):** Flat Pale Ink Blue `#2A4D8A` fill — flat, no glow, cool and understated
- **Tier 2 (4–6g):** Auction Amber `#E87C1E` fill + soft inner warm glow band (`#F5A050` at 30% opacity, inner 2px)
- **Tier 3 (7–9g):** Deep Amber `#C45A00` fill + stronger inner glow (`#E07020` at 40% opacity)
- **Tier 4 (10+g):** Crimson-Amber `#9C2000` fill + strongest inner glow (`#C03000` at 50% opacity) — never reaches full `#FF0000`

**Art Bible Anchors:**
- §4.5 Auction Escalation Color Track: four tiers with exact hex values; border escalation is the primary at-a-glance intensity signal
- §4.2 Semantic Color Vocabulary: Amber = "auction active and escalating"; stop short of full red
- §9.2 Color Prohibitions: Tier 4 `#9C2000` is within spec; full `#FF0000` is prohibited (reserved for Sacrier combat events)

**Generation Prompt:**
`9-slice compatible border frame tile, 256x256px, transparent interior, 6px filled border ring, cel-shade game UI border, [TIER COLOR] flat fill, 1px inner edge highlight 40% white overlay for subtle raised volume, 1px outer edge micro-shadow deeper value, Ankama-style panel chrome, clean bold, isolated transparent background -- negative prompt: photorealistic, 3D bevel, outer glow, decorative motif, complex ornament, smooth gradient, filled interior`
*(Substitute `[TIER COLOR]` for each tile: `#2A4D8A` / `#E87C1E` with inner `#F5A050` / `#C45A00` with inner `#E07020` / `#9C2000` with inner `#C03000`)*

**Engine Constraints:**
- ⚠️ Post-cutoff flag: Verify 9-slice support in `bevy_ui` 0.18 (`ImageScaleMode::Sliced` or equivalent). If available: author as 9-slice strips — corner + edge pieces only, dramatically lower atlas budget. If unavailable: use these full-frame 256×256 overlays stacked per tier; runtime cross-fade via `bevy_tweening` on `ImageNode` alpha between two overlapping border entities.
- 300ms cross-fade between tiers (GDD Rule 4a): requires two stacked `ImageNode` border entities (outgoing fades out, incoming fades in simultaneously)

**⚠️ RESOLVED BUDGET CONFLICT:** Original brief would have produced four 512×512 tiles = 4 MB, exhausting the entire `atlas_ui_hud` budget. Reduced to 256×256px (1 MB total for all four). If 9-slice is confirmed available, replace with ~32×32 strips (near-zero atlas budget).

---

## ASSET-008 — Gold Particle Glow Sprite

| Field | Value |
|-------|-------|
| Category | VFX / Particle |
| Dimensions | 16×16px single frame. Even integers. |
| Format | PNG-32 RGBA, straight alpha, strip ICC profile |
| Naming | `vfx_gold_particle_loop_16.png` |
| Texture Res | 16×16px |
| Atlas | `atlas_vfx` |
| Padding | 2px transparent gutter on all sides within atlas |
| LOD | None — VFX particle sprite |
| Status | Needed |

**Visual Description:**
A single spark/particle sprite — an elongated diamond-lozenge shape, ~12×6px of opaque coverage within the 16px canvas, with sharp tips at top and bottom and softer sides. Core is Arcane Gold `#F5C842`, fading radially to near-transparent at tips and sides in 2–3 cel-shade opacity steps (not smooth gradient). A 1px Prism White `#EEF4FF` center highlight dot at geometric center establishes the luminance apex. No Void outline — light-emitting particle.

**Art Bible Anchors:**
- §2 DRAFT_AUCTION Mood Carrier: "gold particle frame radiates from card border outward; density and color follow §4.5 escalation ramp"
- §4.1 Palette: Arcane Gold core; Prism White highlight — both semantically correct (gold = significance, Prism White = magical/luminous)
- §7.5 Iconography: no Void outline on light-emitting particles; glow form is the identity

**Generation Prompt:**
`Single spark particle sprite, elongated diamond lozenge vertical orientation, Arcane Gold #F5C842 core, 2-3 opacity falloff steps toward transparent tips and sides, Prism White #EEF4FF center highlight dot, no outline, light-emitting particle appearance, Ankama game VFX cel-shade 2D, transparent background, 16x16 canvas, warm gold spark -- negative prompt: outline border, smooth radial gradient, photorealistic, lens flare, 3D, complex shape, multiple particles`

**Engine Constraints:**
- Rendered as world-space `Sprite` instanced by particle system (not a `bevy_ui` `ImageNode`)
- ⚠️ Post-cutoff flag: Verify whether Bevy 0.18 ships a native 2D particle system or requires a third-party crate (e.g., `bevy_hanabi`). Confirm sprite-sheet UV indexing method before atlas packing.

---

## ASSET-009 — Gold Bloom Glow Sprite

| Field | Value |
|-------|-------|
| Category | VFX / Overlay |
| Dimensions | 64×64px single frame. Even integers. |
| Format | PNG-32 RGBA, straight alpha, strip ICC profile |
| Naming | `vfx_gold_bloom_loop_64.png` |
| Texture Res | 64×64px |
| Atlas | `atlas_vfx` |
| Padding | 2px transparent gutter on all sides within atlas |
| LOD | None — 2D VFX overlay, scaled at runtime via `Transform::scale` |
| Status | Needed |

**Visual Description:**
A soft circular aureole ~64px square — not a spark, not a point. Radially symmetric disc stepping from near-solid Arcane Gold at its innermost 8px ring (~60% opacity) through two intermediate bands (40%, 20%) to fully transparent at the outer edge — all discrete cel-shade steps. No hard edge. No Prism White peak at center (unlike ASSET-008). This is pure warm diffuse light: at 24px (HUD coin context) reads as a subtle warm halo; scaled to the price counter it becomes "the only warm thing left in the frame."

**Art Bible Anchors:**
- §2 DRAFT_AUCTION Mood Carrier: "warm-gold bloom on each bid increment, contrasting against the cooling background — the money is the only warm thing left in the frame"
- §7.6 Animation Feel: "gold bloom pulse" behind price counter on 60ms bid tick
- §1 Structured Luminance Hierarchy: brightens foreground action elements to reinforce luminance apex

**Generation Prompt:**
`Soft circular aureole bloom glow sprite, warm diffuse light disc, Arcane Gold #F5C842 warm center fading outward, 3-4 discrete stepped opacity bands from 60% inner to transparent outer, no smooth gradient, no hard edge, no center spike, pure warm diffuse aureole, cel-shade 2D game VFX, transparent background, 64x64 canvas -- negative prompt: outline, sharp edge, cool tones, photorealistic lens bloom, smooth radial gradient, star spike, complex shape`

**Engine Constraints:**
- ⚠️ Post-cutoff flag: Verify Bevy 0.18 `Sprite` supports additive blend mode via built-in field or requires a custom 2D material with `AlphaMode::Add`
- Additive blend will break sprite batching — flag for performance review if the bloom is a per-frame constant during DRAFT_AUCTION
- Same atlas sprite reused at two scales via `Transform::scale` — no second texture needed

---

## ASSET-010 — Prism White Flash Sprite

| Field | Value |
|-------|-------|
| Category | VFX / UI Overlay |
| Dimensions | 512×16px (POT width, even-integer height). Even integers. |
| Format | PNG-32 RGBA, straight alpha, strip ICC profile |
| Naming | `vfx_prism_flash_default_512.png` |
| Texture Res | 512×16px |
| Atlas | `atlas_ui_hud` *(reassigned from atlas_vfx — timer bar is a bevy_ui Node; overlay must live in UI atlas for correct Z layer)* |
| Padding | 2px transparent gutter on all sides within atlas |
| LOD | None — single-frame flash overlay |
| Status | Needed |

**Visual Description:**
A rectangular soft-edged texture: uniform Prism White `#EEF4FF` fill across the core 60% of width, feathering at both horizontal ends through a single soft-edge opacity step (~30% opacity band, ~6px wide on each end) to full transparency. Reads as a clean rectangular overexposed light event. No gold, no outline, no cel-shade steps at the core. The color is cool-white with a slight blue tint — not warm ivory.

**Art Bible Anchors:**
- §4.2 Semantic Color Vocabulary: Prism White `#EEF4FF` = "magical resolution, phase transition" — signals server bid registration
- §4.1 Palette: Prism White is the sole correct color for magical resolution flashes
- §9.1 Style Prohibitions: no warm glow; Prism White is cool-white, not warm ivory

**Generation Prompt:**
`Rectangular flash overlay sprite, cool white Prism White #EEF4FF fill, soft-feathered horizontal edges single opacity step to transparent at each end, uniform bright core center 60% width, overexposed light flash appearance, 512x16 dimensions, cool blue-white tint not warm ivory, transparent background -- negative prompt: warm yellow, gold tones, outline border, hard edge, colored tint, glow halo outside edges, photorealistic, animated`

**Engine Constraints:**
- Used as child `ImageNode` within the timer bar `Node` hierarchy — NOT a world-space sprite
- Flash animation (appear → fade) driven by `bevy_tweening` on `ImageNode` color alpha, or equivalent
- ⚠️ Resolved conflict: reassigned from `atlas_vfx` to `atlas_ui_hud`. Revert only if timer bar is confirmed as a world-space sprite.

---

## ASSET-011 — Bid Pulse Ring Frames

| Field | Value |
|-------|-------|
| Category | VFX / Animation Strip |
| Dimensions | 5-frame strip: 64×64px per frame, sheet 320×64px. Even integers. Authored in white for runtime `Sprite::color` tinting — one strip covers all 4 escalation tier colors. |
| Format | PNG-32 RGBA, straight alpha, strip ICC profile |
| Naming | `vfx_bid_pulse_ring_loop_64.png` |
| Texture Res | 64×64px per frame |
| Atlas | `atlas_vfx` — 5 individual 64×64px atlas rects |
| Padding | 2px transparent gutter between each frame and on outer edge of strip |
| LOD | None — 2D VFX animation |
| Status | Needed |

**Visual Description:**
5-frame animation: a gold annular ring expanding outward from the price counter. Frame 1: tightly-contracted ring ~16px outer diameter, 3px ring width, outer edge in Arcane Gold `#F5C842` (expressed as white for runtime tinting), 1px Void outer contour, inner edge fading to transparent. Frames 2–5: ring expands radially (24→36→48→56px outer diameter), ring width narrows 3→1px, opacity decreases 100%→15%, Void contour fades alongside opacity. Clean geometric form, no texture noise — one-shot event per bid increment.

**Art Bible Anchors:**
- §7.6 Animation Feel: "price counter ticks up, gold bloom pulse" — this ring is the per-bid discrete escalation event; one-shot, not looping idle
- §4.1 Palette: Arcane Gold `#F5C842` (runtime tint target); Void `#0D0D14` outer contour on early frames
- §2 DRAFT_AUCTION Mood Carrier: "pulse frequency increases as bids climb" — this ring is the unit pulse

**Generation Prompt:**
`5-frame animation strip for expanding pulse ring, annular ring shape, authored in white for runtime color tinting, 1px dark outline on outer edge first 3 frames fading out, inner edge transparent, ring expands from 16px diameter frame 1 to 56px frame 5, ring width narrows from 3px to 1px, opacity fades 100% to 15% across sequence, cel-shade 2D game VFX, transparent background, 64x64 per frame, clean geometric ring, no texture noise -- negative prompt: soft glow blob, blurred edges, photorealistic, complex ornament, color gradient fill, filled interior`

**Engine Constraints:**
- Author in white; runtime `Sprite::color` tinting applies escalation tier color (one strip covers all 4 tiers — reduces atlas usage from 4×5 to 1×5 frames)
- ⚠️ Post-cutoff flag: Verify Bevy 0.18 one-shot atlas frame sequence API in `bevy_sprite`. If not available natively, implement via manual system advancing `TextureAtlas::index` for N frames then stopping.

---

## ASSET-012 — DRAFT_INITIAL Entry Sting

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | 2–3 seconds |
| Format | OGG Vorbis |
| Bitrate | 128–192 kbps stereo |
| Naming | `audio_draft_initial_entry.ogg` |
| Loop | No |
| Atlas | N/A — standalone audio file |
| Status | Needed |

**Sonic Description:**
A 2–3 second ascending major phrase scored for light harp glissando and warm muted trumpet — the harp opens with a rising arpeggio in a bright major key, the trumpet answers with a short confident fanfare landing on a sustained fourth or fifth. Warm, unhurried, full-spectrum with gentle high-end shimmer. Emotional register: open possibility and genuine excitement — the auditory equivalent of turning a fresh hand face-up in daylight. No percussion, no minor intervals, no urgency.

**Engine Constraints:**
- ⚠️ Post-cutoff flag: Verify `bevy_audio` OGG Vorbis support in Bevy 0.18 and confirm `AudioPlugin` is included in the app. Use `PlaybackSettings::ONCE` (or 0.18 equivalent) — not looping.
- Plays once on `S2CPhaseChanged(DRAFT_INITIAL)`.

---

## ASSET-013 — DRAFT_INITIAL Purchase Chime

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | 300–400ms |
| Format | OGG Vorbis |
| Bitrate | 96 kbps mono |
| Naming | `audio_draft_initial_purchase.ogg` |
| Loop | No |
| Atlas | N/A — standalone audio file |
| Status | Needed |

**Sonic Description:**
A single bright metallic chime strike (~C5–E5 range), warm overtone profile, no reverb tail longer than natural decay. Clean transient attack, medium-fast decay. Emotional register: small immediate satisfaction — positive confirmation without drama. Noticeably higher in pitch than ASSET-016 (DRAFT_SHOP chime) while sharing the same "confirmation chime" timbre family.

**Engine Constraints:**
- ⚠️ Post-cutoff flag: `bevy_audio` OGG support + non-spatial audio spawn pattern in Bevy 0.18 — verify against migration guide.
- ⚠️ Note: Confirm pitch is distinct from ASSET-019 (Ready Signal) — both are short ascending tones; must not be confused by the player.

---

## ASSET-014 — DRAFT_INITIAL Budget Depleted Bell

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | ~600–800ms (one bell beat with natural decay) |
| Format | OGG Vorbis |
| Bitrate | 96 kbps mono |
| Naming | `audio_draft_initial_budget_depleted.ogg` |
| Loop | No |
| Atlas | N/A — standalone audio file |
| Status | Needed |

**Sonic Description:**
A single struck low-mid bell tone (~G3–B3), neutral and slightly hollow timbre — temple bowl or mid-sized church bell, not a harsh alarm. Emotional register: quiet finality. Reports the closed budget state without emotional penalty. No pitch drop that reads as "sad" or "wrong." The tone simply stops the player's decision loop cleanly.

**Engine Constraints:**
- Triggered client-side when `S2CGoldUpdate` reduces `local_gold` to 0 during DRAFT_INITIAL — no dedicated server message exists for this event.
- ⚠️ Post-cutoff flag: same `bevy_audio` OGG caveat as ASSET-012.

---

## ASSET-015 — DRAFT_SHOP Entry Phrase

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | 1–2 seconds |
| Format | OGG Vorbis |
| Bitrate | 128–192 kbps stereo |
| Naming | `audio_draft_shop_entry.ogg` |
| Loop | No |
| Atlas | N/A — standalone audio file |
| Status | Needed |

**Sonic Description:**
A 1–2s descending phrase for fingerpicked acoustic guitar or plucked lute — two to three notes stepping gently downward in a warm pentatonic figure, unhurried, with natural string resonance and dry close-mic character. Clearly contrasts the DRAFT_INITIAL sting (open/airy, ASSET-012) and any auction countdown urgency. Emotional register: deliberate quietness after auction tension.

**Engine Constraints:**
- Per VA.6 GDD: fires at transition-animation-complete (~350ms after `S2CAuctionSettled`) with a 50ms holdoff to guarantee the auction urgency tone is fully silent before this sting begins.
- ⚠️ Post-cutoff flag: same `bevy_audio` OGG caveat as ASSET-012.

---

## ASSET-016 — DRAFT_SHOP Purchase Chime

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | 300–400ms |
| Format | OGG Vorbis |
| Bitrate | 96 kbps mono |
| Naming | `audio_draft_shop_purchase.ogg` |
| Loop | No |
| Atlas | N/A — standalone audio file |
| Status | Needed |

**Sonic Description:**
Single warm chime strike, a minor third to fifth lower in pitch than ASSET-013 (DRAFT_INITIAL chime), same instrument family — a slightly larger or rounder bell tone. Emotional register: quiet satisfaction and deliberate commitment, matching DRAFT_SHOP "Calculation" mood. Lower pitch marks this as a heavier, more considered transaction than the bright DRAFT_INITIAL chime while shared timbre keeps both in the same game audio vocabulary.

**Engine Constraints:**
- Same implementation path as ASSET-013 — triggered on `S2CCardAcquired` during DRAFT_SHOP.
- ⚠️ Post-cutoff flag: same `bevy_audio` OGG caveat as ASSET-012.

---

## ASSET-017 — Shop Refresh Swoosh

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | 200–300ms |
| Format | OGG Vorbis |
| Bitrate | 96 kbps stereo |
| Naming | `audio_shop_refresh_swoosh.ogg` |
| Loop | No |
| Atlas | N/A — standalone audio file |
| Status | Needed |

**Sonic Description:**
Paper shuffle and card-slide source material — multiple cards fanned and released in quick succession, producing a layered swish texture with natural high-end paper flutter at onset and a soft settle at the tail. Dry acoustic, no reverb processing. Emotional register: brisk efficiency — the shopkeeper clearing the counter. Neutral in valence; confirms the action without celebration.

**Engine Constraints:**
- Triggered on `C2SRefreshShop` send (client-side, same frame as button disable — not awaiting `S2CShopSlots`).
- ⚠️ Post-cutoff flag: same `bevy_audio` OGG caveat as ASSET-012.

---

## ASSET-018 — Shop Refresh Failed

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | 200ms |
| Format | OGG Vorbis |
| Bitrate | 96 kbps mono |
| Naming | `audio_shop_refresh_failed.ogg` |
| Loop | No |
| Atlas | N/A — standalone audio file |
| Status | Needed |

**Sonic Description:**
ASSET-017 waveform time-reversed and de-brightened (low-shelf cut below 400Hz, gentle high-shelf roll-off above 6kHz) — a muffled, incomplete swoosh that starts with the tail texture and ends with a dampened onset. Emotional register: muted impediment, not alarm. Clearly distinct from ASSET-017 by directionality; clearly less harsh than any error buzzer. A retry prompt, not a failure state.

**Engine Constraints:**
- Triggered on the 5-second `C2SRefreshShop` timeout (DRAFT_SHOP Rule 5 edge case).
- ⚠️ Post-cutoff flag: same `bevy_audio` OGG caveat as ASSET-012.

---

## ASSET-019 — Ready Signal

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | 200–300ms |
| Format | OGG Vorbis |
| Bitrate | 96 kbps mono |
| Naming | `audio_ready_signal.ogg` |
| Loop | No |
| Atlas | N/A — standalone audio file |
| Status | Needed |

**Sonic Description:**
Short ascending two-note phrase (major second or minor third interval) on a bright metallic idiophone — glockenspiel or vibraphone. Clean attack, short sustain, no reverb tail past natural decay. The upward step reads as "done and stepping up" — decisive small flourish rather than celebration. Emotional register: committed finality with a hint of confident anticipation.

**Engine Constraints:**
- Triggered on `C2SSignalReady { retract: false }` send (both DRAFT_INITIAL Rule 7 and DRAFT_SHOP equivalent) — client-side, not on server acknowledgment.
- ⚠️ Post-cutoff flag: same `bevy_audio` OGG caveat as ASSET-012.

---

## ASSET-020 — Ready Retracted

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | 200–300ms |
| Format | OGG Vorbis |
| Bitrate | 96 kbps mono |
| Naming | `audio_ready_retracted.ogg` |
| Loop | No |
| Atlas | N/A — standalone audio file |
| Status | Needed |

**Sonic Description:**
Descending counterpart to ASSET-019 — same two pitches in reverse order, same instrument family, same duration, softer attack velocity. Reverses the "stepping up" gesture using tonal material the player's ear already knows from ASSET-019, making the undo semantically clear without text feedback. Emotional register: mild reversal — neither failure nor relief.

**Engine Constraints:**
- Triggered on `C2SSignalReady { retract: true }` send.
- ⚠️ Post-cutoff flag: same `bevy_audio` OGG caveat as ASSET-012.

---

## ASSET-021 — Countdown Tick Loop

| Field | Value |
|-------|-------|
| Category | Audio |
| Duration | 1 second per tick cycle (looping) |
| Format | OGG Vorbis |
| Bitrate | 96 kbps mono |
| Naming | `audio_countdown_tick_loop.ogg` |
| Loop | Yes — OGG `LOOPSTART`/`LOOPEND` metadata required; timer-based fallback recommended (see Constraints) |
| Atlas | N/A — standalone audio file |
| Status | Needed |

**Sonic Description:**
Looping rhythmic tick at exactly 1Hz — dry mechanical source (light wood block, clock escapement, or similar percussive transient), sub-50ms per tick, no pitch identity. Purely mechanical urgency: marks time without emotional commentary, registering peripherally rather than consciously. Distinct from the pitch-based anxiety language reserved for the DRAFT_AUCTION escalation system. Shared audio asset used by DRAFT_INITIAL and DRAFT_SHOP red-zone handlers (separate trigger handlers per GDD VA.6 to prevent double-fire).

**Engine Constraints:**
- ⚠️ Post-cutoff flag: Verify OGG `LOOPSTART`/`LOOPEND` comment field support in Bevy 0.18 `bevy_audio` (`rodio` backend may not parse loop metadata). **Recommended fallback:** spawn the tick SFX once per second via a Bevy `Timer` with `TimerMode::Repeating` — frame-accurate control, enables easy frequency escalation in final 5s without a second audio file. Timer-based approach is robust regardless of OGG loop metadata support.
- A single `AudioState` flag gating "tick already playing for this phase" prevents duplicate playback if the handler fires multiple frames (per GDD VA.6 specification).
