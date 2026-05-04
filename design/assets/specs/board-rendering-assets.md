# Asset Specs — System: board-rendering

> **Source**: design/gdd/board-rendering.md (Visual/Audio Requirements §605)
> **Art Bible**: design/art/art-bible.md
> **Generated**: 2026-04-30
> **Review mode**: Full (art-director + technical-artist)
> **Status**: 30 assets specced / 30 approved / 0 in production / 0 done

---

## Flags Requiring Resolution Before Production

| # | Asset(s) | Flag | Recommendation |
|---|---|---|---|
| 1 | ASSET-026/ASSET-027/ASSET-028 | Tint-only vs. separate atlas frames for cell node state variants | **Separate frames** — AD wrote distinct art for each state; cleaner for implementation |
| 2 | ASSET-035/ASSET-036 | `ui_` prefix on world-space board sprites (GDD name canonical; naming inconsistency with `env_` convention) | Note only — GDD names are canonical; code must match GDD |
| 3 | ASSET-037 | 512×80 non-POT height disables WebGL2 mipmapping | **Author at 512×128 POT canvas** — feather art into lower 80px, upper 48px transparent |
| 4 | ASSET-039 | 48×64 GDD canvas incompatible with 64×96 unit atlas grid | **Author at 64×96 canvas** with 48×64 art centered, transparent padding |
| 5 | ASSET-044 | 1×2 odd-width violates §8.8 even-dimension rule | **Documented exception** — established Bevy HP bar pattern; 2px height satisfies even-height |
| 6 | ASSET-051 | Two-part SFX as single or two files | **Single composited file** — fallback to two files with Timer delay only if authoring requires it |

---

## 2026-05-04 Coverage Notes

- Board Rendering remains the owner of standing board sprites, objective base/reveal sprites, prism token sprite, and board/objective audio already listed here.
- Combat-specific hit flashes, damage-number materials, unit state indicators, and combat-specific audio are now tracked in `design/assets/specs/combat-resolution-assets.md`.
- Prism reward-card art and prism reward feedback are tracked in `design/assets/specs/prism-system-assets.md`; this file keeps ASSET-032/043/049 as the reusable board token, collection shimmer, and collection sound.
- Existing objective destruction audio rows ASSET-047 and ASSET-048 are reused by Combat Resolution and should not be duplicated under new IDs.

---

## ASSET-022 — env_board_background_default

| Field | Value |
|-------|-------|
| Category | Environment |
| Dimensions | 512×512 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | env_board_background_default.png |
| Atlas | Standalone (too large to pack efficiently into atlas_board 1024×1024) |
| M2 Priority | PLACEHOLDER |

**Visual Description:**
Top-down arena floor in near-black charcoal cobblestone (`#1A1A22`), five lane channels defined by continuous chains of cyan-blue neon diamond nodes rather than tile borders, stone surface showing lightly cracked hand-laid slabs at low contrast. Red directional arrows are engraved into each cell pointing toward the far end; Player A's near edge carries a Sky Blue `#3A8EDB` stone inlay trim strip; Player B's far edge carries Terracotta `#D45C22`. The floor is the stage — no individual stone competes for attention.

**Art Bible Anchors:**
- §6.1 Board Surface: dark cobblestone `#1A1A22`, blue neon diamond-chain lane lines as primary wayfinding, red arrows embedded in cells, player-side edge inlay stripes
- §3.3 Board Geometry: neutral surface with strong directional language; board is the stage
- §4.3 Player Side Colors: Sky Blue `#3A8EDB` / Terracotta `#D45C22` edge inlays

**Generation Prompt:**
Top-down game board, 5-lane arena floor, dark near-black cobblestone `#1A1A22`, hand-illustrated cel-shaded, Krosmaga/Ankama aesthetic, confident Void outlines `#0D0D14`, five vertical lane channels separated by 1–2px dark groove, each channel has a chain of glowing cyan-blue neon diamond nodes running full length, red directional arrow glyphs engraved into floor cells pointing upward, bottom edge sky blue `#3A8EDB` stone inlay stripe, top edge terracotta `#D45C22` stone inlay stripe, ambient cool blue-teal glow from lane nodes, fully saturated local color, game-ready sprite, PNG-32 straight alpha. Negative: photorealistic, grimdark, desaturated, Hearthstone warmth, flat vector minimalism, green lane lines.

**Status:** Needed

---

## ASSET-023 — env_lane_divider_64x80

| Field | Value |
|-------|-------|
| Category | Environment |
| Dimensions | 64×80 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | env_lane_divider_default_board.png |
| Atlas | atlas_board (1024×1024) |
| M2 Priority | PLACEHOLDER |

**Visual Description:**
A 64×80 vertical strip representing the groove between adjacent lanes — a 1–2px darkening channel cut into cobblestone, lighter than Void but darker than the stone base, with no glow, no cap geometry, no decoration. Reads as a subtle depth groove baked into the floor; cobblestone texture on either side consistent with ASSET-022 so seams are invisible when atlased.

**Art Bible Anchors:**
- §6.1 Board Surface: "Lane dividers: subtle 1–2px darkening groove — enough to show separation without visual noise"
- §3.3: Neutral surface language; no cage-grid aesthetic

**Generation Prompt:**
64×80 pixel game sprite, vertical lane divider strip, dark cobblestone texture Ankama cel-shaded style, 1–2px subtly darker groove centered vertically, groove color slightly deeper than `#1A1A22`, no glow, no neon, no decorative elements, seamlessly tileable vertically, Void `#0D0D14` micro-shadow on groove edges only, transparent background PNG-32 straight alpha. Negative: glowing edges, prominent border, decorative carvings, visible seams, bright color.

**Status:** Needed

---

## ASSET-024 — env_lane_number_label_01–05 (×5 sprites)

| Field | Value |
|-------|-------|
| Category | Environment / UI |
| Dimensions | 32×32 px each (5 sprites) |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | env_lane_number_label_01_board.png … env_lane_number_label_05_board.png |
| Atlas | atlas_board (1024×1024) — 5 frames, sequential indices |
| M2 Priority | BLOCKING |

**Visual Description:**
Five 32×32 sprites each displaying a single heavy-weight digit (1–5) in Ivory `#F7F0DC` with Void `#0D0D14` outline, set on a flat angular chip background (horizontal pill with beveled ends) in Ink Blue `#1A2D5A`. The numeral fills ~60% of chip height. No drop shadow, no decorative framing — pure information delivery readable from arm's length at both board ends.

**Art Bible Anchors:**
- §3.4 UI Shape Grammar: "HUD elements = flat angular chips with beveled ends"
- §7.4 Typography: Heavy weight, Ivory on Ink Blue for readouts
- §7.5 Iconography: 2px Void outline, flat interior fill

**Generation Prompt:**
32×32 pixel game UI sprite, lane number label, single large bold digit centered, Ankama cel-shaded, flat angular chip background with beveled horizontal ends in Ink Blue `#1A2D5A`, digit in Ivory `#F7F0DC` heavy display sans-serif, 2px Void `#0D0D14` outline, digit fills 60% chip height, high contrast, transparent PNG-32 straight alpha, produce 5 variants: "1" "2" "3" "4" "5". Negative: serif font, drop shadow, gradient fill, rounded soft edges.

**Status:** Generated Placeholder

---

## ASSET-025 — env_cell_node_idle_32x32

| Field | Value |
|-------|-------|
| Category | Environment |
| Dimensions | 32×32 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | env_cell_node_idle_board.png |
| Atlas | atlas_board (1024×1024) |
| M2 Priority | BLOCKING |
| Engine note | Idle pulse (±3% scale oscillation, 1.5s loop) is code-driven — not baked into sprite |

**Visual Description:**
A 32×32 diamond-shaped node in cyan-blue neon — the primary navigational landmark on the board grid. Confident 2px Void outline baked in, flat cyan-blue interior fill (`#3AB4CC`, distinct from stat-reserved HP Teal `#2AA8C4`), and a soft inner glow halo radiating from center to diamond tips at ~40% opacity. Single frame; scale pulse is code-driven.

**Art Bible Anchors:**
- §6.1 Board Surface: "blue neon diamond chain nodes — primary wayfinding"
- §8.4 Outline: 1–2px Void baked
- §4: Cyan-blue distinct from HP Teal (stat-reserved)

**Generation Prompt:**
32×32 pixel game sprite, diamond-shaped cell node, Ankama/Krosmaga cel-shaded, cyan-blue neon diamond `#3AB4CC`, 2px Void `#0D0D14` outline baked, soft inner radial glow from center to tips at 40% opacity, flat color fill interior, fully saturated, single frame idle state, transparent PNG-32 straight alpha. Negative: circle shape, green tint, animated frames in sprite, photorealistic glow, soft blurry edges.

**Status:** Generated Placeholder

---

## ASSET-026 — env_cell_node_spawn_active_32x32

| Field | Value |
|-------|-------|
| Category | Environment |
| Dimensions | 32×32 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | env_cell_node_active_board.png |
| Atlas | atlas_board (1024×1024) — separate frame from ASSET-025 |
| M2 Priority | BLOCKING |
| ⚑ Flag | Confirm separate sprite vs. runtime Sprite.color tint on ASSET-025 before painting. GDD says "spawn-highlight is a tint"; separate sprite is cleaner for M2 if art differs beyond tint. |

**Visual Description:**
Same diamond geometry as ASSET-025, interior shifted to warm gold-white — Arcane Gold `#F5C842`-tinted ivory bloom (`~#FFF0C0`). Void outline and diamond shape remain identical. The gold communicates "you may place here" using the established gold = significance vocabulary.

**Art Bible Anchors:**
- §4.1 Arcane Gold: "objectives, rewards, premium UI" — gold = valid significant action
- §3.3 / §6.1: Node diamond geometry constant across states
- §4.2 Semantic Color: gold = "game object of significance"

**Generation Prompt:**
32×32 pixel game sprite, diamond-shaped spawn cell node active state, Ankama cel-shaded, warm gold-white tint interior `#FFF0C0` with Arcane Gold `#F5C842` inner glow 50% opacity, 2px Void `#0D0D14` baked outline, same diamond geometry as idle node, single warm bloom from center, transparent PNG-32 straight alpha. Negative: cyan-blue fill, cool color, red tint, circle shape.

**Status:** Generated Placeholder

---

## ASSET-027 — env_cell_node_spawn_inactive_32x32

| Field | Value |
|-------|-------|
| Category | Environment |
| Dimensions | 32×32 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | env_cell_node_inactive_board.png |
| Atlas | atlas_board (1024×1024) |
| M2 Priority | PLACEHOLDER |
| ⚑ Flag | GDD says "M2: reuse idle node at 50% alpha." Confirm whether this is a runtime alpha tint on ASSET-025 (no separate file) or a distinct baked sprite. |

**Visual Description:**
Diamond node geometry at reduced luminance — interior desaturates toward mid-grey (`~#4A5060`), Void outline remains intact. Grid reads as complete; absence of glow or warm tint communicates unavailability without a new color.

**Art Bible Anchors:**
- §6.1: Cell node geometry consistent across states
- §1: Bold outlines hold at 32px even in dimmed state

**Generation Prompt:**
32×32 pixel game sprite, diamond-shaped cell node inactive/unavailable, Ankama cel-shaded, desaturated grey interior `#4A5060`, no glow, no color tint, 2px Void `#0D0D14` baked outline, flat fill, visually receded, transparent PNG-32 straight alpha. Negative: glowing, warm tint, cyan saturated fill, red or orange.

**Status:** Needed

---

## ASSET-028 — env_cell_node_invalid_32x32

| Field | Value |
|-------|-------|
| Category | Environment |
| Dimensions | 32×32 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | env_cell_node_invalid_board.png |
| Atlas | atlas_board (1024×1024) |
| M2 Priority | PLACEHOLDER |
| ⚑ Flag | GDD says "M2: red-tinted idle node." Confirm whether this is a runtime Sprite.color red tint on ASSET-025 (no separate file) or a distinct baked sprite. |

**Visual Description:**
Diamond node interior drops to near-board-surface tone (`~#252530`), nearly flush with cobblestone beneath. Void outline softened to `#1A1A2A`. Present but deliberately receding. No red, no X glyph — invalid communicates through absence of activation.

**Art Bible Anchors:**
- §4.2 Semantic Color: red = combat event — must not be used for invalid placement
- §9.2: Never ATK orange or HP teal for non-stat purpose

**Generation Prompt:**
32×32 pixel game sprite, diamond-shaped cell node invalid/blocked, Ankama cel-shaded, near-black interior `#252530` nearly flush with board background, softened outline `#1A1A2A`, no glow, no warning color, no X mark, visually receded into floor, transparent PNG-32 straight alpha. Negative: red tint, orange warning, glowing edges, X mark, cyan fill.

**Status:** Needed

---

## ASSET-029 — env_objective_unknown_64x96

| Field | Value |
|-------|-------|
| Category | Environment |
| Dimensions | 64×96 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | env_objective_unknown_board.png |
| Atlas | atlas_board (1024×1024) |
| M2 Priority | BLOCKING |
| Engine note | ALL standing objectives render as this sprite, per ADR-001. ObjectiveIdentityCache is never used for standing-objective rendering. |

**Visual Description:**
Squat three-tier cylindrical cobblestone pedestal topped by a stone-grey inert teardrop-flame silhouette with a floating `?` glyph (~8px) in Ivory `#F7F0DC` just above the tip. No glow, no flame, no color warmth — pixel-identical in silhouette and scale to ASSET-030 (the real reveal). Vertically taller than a standard unit, reading as a landmark.

**Art Bible Anchors:**
- §6.3 Objective Design: "Pixel-identical silhouette between real and fake. Stone-grey inert teardrop, same geometry, no glow. `?` glyph reads as unknown, not empty"
- §3.3: "Objectives vertically taller than units — landmarks"
- §8.1: 64×96 Board Tier

**Generation Prompt:**
64×96 pixel game sprite, unknown objective on cobblestone pedestal, Ankama cel-shaded, three-tier squat cylindrical stone pedestal cobblestone texture slightly lighter than `#1A1A22`, atop pedestal stone-grey inert teardrop-flame silhouette 2px Void `#0D0D14` outline, no flame, no glow, stone-grey fill `#888899`, small Ivory `#F7F0DC` `?` glyph floating 3px above tip, objective height 1.5× standard unit, transparent PNG-32 straight alpha. Negative: animated flame, gold glow, green tint, red tint, any warm color on teardrop.

**Status:** Generated Placeholder

---

## ASSET-030 — env_objective_real_reveal_64x96

| Field | Value |
|-------|-------|
| Category | Environment |
| Dimensions | 64×96 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | env_objective_real_reveal_board.png |
| Atlas | atlas_board (1024×1024) |
| M2 Priority | BLOCKING |
| Engine note | Swap TextureAtlas.index on existing objective entity — do NOT despawn/respawn (loses game-state components). |

**Visual Description:**
Same pedestal geometry as ASSET-029, teardrop now alive: Arcane Gold `#F5C842` flame fills the teardrop from base to tip, inner swirl glyph in deep amber `#C88020` within flame body, warm low-intensity glow halo ~4–6px beyond outline. No `?` glyph. This is the game's emotional peak — single brightest, most saturated element in the scene per Structured Luminance Hierarchy. This is the static post-reveal frame; the golden flash sequence is ASSET-040.

**Art Bible Anchors:**
- §6.3: "Real objective: animated flame in Arcane Gold `#F5C842`, inner swirl glyph, low warm glow halo"
- §1 Structured Luminance Hierarchy: "Revealed objective is the visual apex during RESOLUTION"
- §4.1 Arcane Gold: the gold = significance payoff

**Generation Prompt:**
64×96 pixel game sprite, revealed real objective on cobblestone pedestal, Ankama cel-shaded, three-tier squat cylindrical cobblestone pedestal identical to unknown variant, teardrop-flame filled with Arcane Gold `#F5C842` flame, inner swirl glyph in deep amber `#C88020`, 2px Void `#0D0D14` outer outline, warm golden glow halo 4–6px at 40% opacity, no `?` glyph, luminance apex, fully saturated gold, transparent PNG-32 straight alpha. Negative: stone grey fill, green tint, blue tint, cold flame, Hearthstone golden treatment, spinning particle loop.

**Status:** Generated Placeholder

---

## ASSET-031 — env_objective_fake_crack_64x96

| Field | Value |
|-------|-------|
| Category | Environment |
| Dimensions | 64×96 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | env_objective_fake_crack_board.png |
| Atlas | atlas_board (1024×1024) |
| M2 Priority | PLACEHOLDER |
| Engine note | Same TextureAtlas.index-swap pattern as ASSET-030 when is_fake = true confirmed. |

**Visual Description:**
Same pedestal and teardrop as ASSET-029, now post-reveal broken: 2–3 sharp diagonal crack lines in lighter stone tone (`#AAAACC`), "FAKE" text glyph in Ivory `#F7F0DC` above the tip, hairline crack at one pedestal tier joint. Color stays stone-grey throughout — no warm tones, no red. The underwhelming visual is intentional (the punchline lives in the audio, not the art).

**Art Bible Anchors:**
- §6.3: "Fake: crack overlay + FAKE text (800ms)"
- §9.2: No red for fake/losing state; stone-grey = absence of significance, not punishment

**Generation Prompt:**
64×96 pixel game sprite, fake objective revealed on cobblestone pedestal, Ankama cel-shaded, same three-tier cylindrical cobblestone pedestal, stone-grey teardrop `#888899` with 2px Void outline, 2–3 sharp diagonal crack lines in lighter grey `#AAAACC`, hairline crack at one pedestal tier joint, Ivory `#F7F0DC` "FAKE" text label above tip, no flame, no gold color, muted and subdued, transparent PNG-32 straight alpha. Negative: red tint, orange glow, gold flame, dramatic explosion rubble.

**Status:** Needed

---

## ASSET-032 — env_prism_idle_32x32

| Field | Value |
|-------|-------|
| Category | Environment |
| Dimensions | 32×32 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | env_prism_idle_board.png |
| Atlas | atlas_board (1024×1024) |
| M2 Priority | PLACEHOLDER |
| Engine note | Rotation (idle spin at prism_spin_speed rad/s) is code-driven via Transform.rotation — not baked into sprite. |

**Visual Description:**
Six-pointed star inset etched into cobblestone floor surface — a recessed inset, not a raised gem or floating icon. Star interior filled with soft Prism White `#EEF4FF` glow fading from center points outward. 1px Void outline traces the six points. Cobblestone surround consistent with board surface; star reads as floor geometry.

**Art Bible Anchors:**
- §6.4 Prism Cell: "six-pointed star inset etched into cobblestone. Available: soft Prism White `#EEF4FF` glow — floor geometry, not floating icon"
- §4.1 Prism White: "reserved for high-value magical events"

**Generation Prompt:**
32×32 pixel game sprite, prism cell six-pointed star inset floor marking, Ankama cel-shaded, dark cobblestone surround `#1A1A22`, six-pointed star inset etched into stone surface, star interior soft Prism White `#EEF4FF` glow radiating from center, 1px Void `#0D0D14` star outline, glow fades at 50% opacity at edges, reads as floor geometry not floating icon, transparent PNG-32 straight alpha. Negative: floating gem, raised icon, hexagonal shape, warm glow, gold color.

**Status:** Needed

---

## ASSET-033 — ui_unit_base_player_a_48x16

| Field | Value |
|-------|-------|
| Category | UI (bevy_ui layer) |
| Dimensions | 48×16 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | ui_unit_base_player_a_default_hud.png |
| Atlas | atlas_ui_hud (1024×1024) |
| M2 Priority | BLOCKING |
| Engine note | Rendered as bevy_ui ImageNode above world-space board sprites. Use ImageNode::new(handle) — not deprecated UiImage::new(). |

**Visual Description:**
Flat horizontal base ring in Sky Blue `#3A8EDB` with 1–2px Void outline, angular chip shape with beveled ends consistent with HUD grammar. Single subtle cel-shade highlight one luminance stop up at top center. A circle shape subtly inset on the base surface for colorblind backup (circle = Player A per §4.3).

**Art Bible Anchors:**
- §4.3 Player Side Colors: Sky Blue `#3A8EDB` / circle shape for Player A
- §3.4 UI Shape Grammar: flat angular chips with beveled ends
- §8.4 Outline: 1–2px Void baked

**Generation Prompt:**
48×16 pixel game sprite, unit base platform strip Player A, Ankama cel-shaded, flat horizontal beveled-end chip shape, solid Sky Blue `#3A8EDB` fill, subtle cel-shade highlight one stop lighter at top center, small inset circle glyph on surface center, 1–2px Void `#0D0D14` outline, transparent PNG-32 straight alpha. Negative: gradient, photorealism, diamond shape inset, terracotta color.

**Status:** Generated Placeholder

---

## ASSET-034 — ui_unit_base_player_b_48x16

| Field | Value |
|-------|-------|
| Category | UI (bevy_ui layer) |
| Dimensions | 48×16 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | ui_unit_base_player_b_default_hud.png |
| Atlas | atlas_ui_hud (1024×1024) |
| M2 Priority | BLOCKING |
| Engine note | Same bevy_ui ImageNode layer as ASSET-033. Must pair as a matched visual set — same scale, same outline weight, opposite color and inset shape. |

**Visual Description:**
Identical geometry to ASSET-033 in Terracotta `#D45C22` for Player B. Colorblind backup shape inset on the surface is a diamond (not a circle), distinguishing from Player A at shape-only read.

**Art Bible Anchors:**
- §4.3: Terracotta `#D45C22` / diamond shape for Player B
- §3.4: Matched pair with ASSET-033
- §9.3: Player identity carried by base ring, not board tiles

**Generation Prompt:**
48×16 pixel game sprite, unit base platform strip Player B, Ankama cel-shaded, flat horizontal beveled-end chip — identical geometry to Player A base, solid Terracotta `#D45C22` fill, subtle cel-shade highlight one stop lighter at top center, small inset diamond glyph on surface center, 1–2px Void `#0D0D14` outline, transparent PNG-32 straight alpha. Negative: sky blue fill, circle shape inset, gradient.

**Status:** Generated Placeholder

---

## ASSET-035 — ui_trap_tile_facedown_32x32

| Field | Value |
|-------|-------|
| Category | Environment (world-space board sprite) |
| Dimensions | 32×32 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | ui_trap_tile_facedown_board.png |
| Atlas | atlas_board (1024×1024) |
| M2 Priority | BLOCKING |
| ⚑ Flag | GDD uses `ui_` prefix but this is a world-space board sprite (env_ convention). GDD name is canonical — code must match GDD. |

**Visual Description:**
A deliberately boring, low-profile horizontal bar in neutral mid-grey (`~#666678`) with 1–2px Void outline — the one sprite designed to fail to attract attention. Near-flat rectangle, very slightly rounded corners, no icon, no glyph, no color. Silhouette: §3.1 Trap type — flat horizontal bar. "Reads as nothing until triggered."

**Art Bible Anchors:**
- §3.1: "Trap: near-flat horizontal bar — deliberately breaks silhouette rule to serve hidden-placement mechanic"
- §5.5: "Traps are the exception — the one place interesting pose = bad design"
- §9.1: No decorative noise

**Generation Prompt:**
32×32 pixel game sprite, face-down trap tile marker, Ankama cel-shaded, near-flat horizontal rectangle bar, neutral mid-grey `#666678`, 1–2px Void `#0D0D14` outline, no icon, no glyph, no color accent, deliberately boring and unassuming, reads as inert floor tile, very slight bevel on long edges only, transparent PNG-32 straight alpha. Negative: bright color, class icon, interesting silhouette, glowing edge, any detail that suggests a unit.

**Status:** Generated Placeholder

---

## ASSET-036 — ui_structure_token_32x32

| Field | Value |
|-------|-------|
| Category | Environment (world-space board sprite) |
| Dimensions | 32×32 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | ui_structure_token_default_board.png |
| Atlas | atlas_board (1024×1024) |
| M2 Priority | PLACEHOLDER |
| ⚑ Flag | Same ui_ / world-space naming inconsistency as ASSET-035. GDD name canonical. |

**Visual Description:**
Inverted trapezoid silhouette — broad at top, narrower at bottom — in stone-grey mid-tone with Void outline. Squat and anchored, flat top surface, communicating immovability. Thin faction-color accent strip at top edge as the only color differentiation. Unambiguously distinct from Minion (tall triangle) and Trap (flat bar) at 32px silhouette read.

**Art Bible Anchors:**
- §3.1: "Structure: inverted trapezoid — broad base, stable. Squat, wide, anchored to ground plane"
- §1 Silhouette First: identifiable by shape alone before color

**Generation Prompt:**
32×32 pixel game sprite, structure unit token, Ankama cel-shaded, inverted trapezoid silhouette — wider at top narrower at bottom, squat and anchored, stone-grey fill `#888899`, one cel-shade highlight at top flat surface, thin Arcane Gold `#F5C842` accent strip along top edge (faction color applied at runtime), 2px Void `#0D0D14` outline, transparent PNG-32 straight alpha. Negative: tall triangle, flat bar, rounded edges, floating, photorealistic detail.

**Status:** Needed

---

## ASSET-037 — ui_field_wash_lane_512x80

| Field | Value |
|-------|-------|
| Category | Environment (world-space overlay) |
| Dimensions | 512×128 px recommended (POT) — art in lower 80px, upper 48px transparent |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | ui_field_wash_lane_default.png |
| Atlas | Standalone |
| M2 Priority | PLACEHOLDER |
| ⚑ Flag | GDD specifies 512×80 (non-POT height disables WebGL2 mipmapping). TA recommends 512×128 POT canvas. If keeping 512×80, set mip_level_count=1 explicitly and document exception. TD sign-off required. |

**Visual Description:**
Horizontal tinted wash strip — a semi-transparent color band (~15–25% opacity) tinting a full-width lane during RESOLUTION lane activation. Flat color with soft vertical feather fading to transparent at 10px top/bottom margins. No outline, no shape geometry — a pure color compositing layer. Two color variants: warm Arcane Gold `#F5C842` (win activation) and cool Ink Blue `#3A5080` (loss/neutral activation).

**Art Bible Anchors:**
- §2 RESOLUTION: "Each lane activates in sequence with a warm burst then settles — sequential spotlight sweep"
- §3.5 Visual Hierarchy RESOLUTION: "Lane channels brighten"
- §4.2: Warm gold = win, cool blue-grey = loss/neutral

**Generation Prompt:**
512×128 pixel game sprite, lane activation color wash overlay, horizontal rectangle, flat color fill with soft vertical feather — solid at horizontal center fading to 0% opacity at top 10px and bottom 10px edges, transparent 48px margin above art area, produce two variants: warm Arcane Gold `#F5C842` at 20% opacity (win) and cool Ink Blue `#3A5080` at 20% opacity (loss/neutral), no outline, compositing overlay, transparent PNG-32 straight alpha. Negative: hard edges, opaque fill, decorative elements, text, icons.

**Status:** Needed

---

## ASSET-038 — ui_field_badge_icon_24x24

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 24×24 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | ui_field_badge_icon_default_hud.png |
| Atlas | atlas_ui_hud (1024×1024) |
| M2 Priority | PLACEHOLDER |

**Visual Description:**
Wide shallow oval icon (§3.1 Field silhouette: "wide shallow oval — environmental, not a unit") with 2px Void outline and flat stone-grey interior. Communicates "area effect / ground covering" through its wide horizontal spread. Reads clearly at 24px as a distinct type icon, separate from triangle (Minion), trapezoid (Structure), and bar (Trap).

**Art Bible Anchors:**
- §3.1: "Field: wide shallow oval — environmental, not a unit"
- §7.5 Iconography: 2px Void outline, flat fill, minimum 24px
- §9.2: Must not use Cra forest green, ATK orange, or HP teal

**Generation Prompt:**
24×24 pixel game sprite, field unit type badge icon, Ankama cel-shaded, wide shallow horizontal oval conveying area-cover spread, flat stone-grey fill `#888899`, minimal cel-shade highlight, 2px Void `#0D0D14` outline, simple flat icon, legible at 24px, no class color, transparent PNG-32 straight alpha. Negative: tall triangle, square, green fill, orange fill, teal fill, text label.

**Status:** Needed

---

## ASSET-039 — ui_unit_placeholder_48x64

| Field | Value |
|-------|-------|
| Category | UI / Error State |
| Dimensions | 64×96 px canvas with 48×64 px art centered, transparent padding |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | ui_unit_placeholder_default_board.png |
| Atlas | atlas_units (2048×2048) |
| M2 Priority | BLOCKING |
| ⚑ Flag | GDD specifies 48×64; TA flags: must be 64×96 canvas to match unit atlas grid layout. Author art at 48×64 centered on 64×96 canvas with transparent outer margin. |

**Visual Description:**
64×96 canvas with 48×64 art centered — Ink Blue `#1A2D5A` fill, 2px Void outer outline, 2px Arcane Gold `#F5C842` inner border (2px inset), large centered `?` glyph in Ivory `#F7F0DC` heavy weight. Communicates "unit present, art absent" — the thin gold border signals intentionality rather than empty cell.

**Art Bible Anchors:**
- §4.1: Ink Blue = depth background; Ivory = text readouts; Arcane Gold = significance
- §7.5: `?` glyph consistent with ASSET-029 unknown-state convention
- §1: Bold enough to hold at 64px even in error state

**Generation Prompt:**
64×96 pixel game sprite, missing unit art placeholder, Ankama game style, 48×64 art centered on 64×96 canvas with transparent outer margin, flat rectangle matching unit board footprint, Ink Blue `#1A2D5A` fill, 2px Void `#0D0D14` outer outline on the 48×64 art area, 2px Arcane Gold `#F5C842` inner border inset 2px, large centered `?` glyph in Ivory `#F7F0DC` heavy weight, legible, transparent PNG-32 straight alpha. Negative: bright red error color, X mark, gradient, decorative content.

**Status:** Generated Placeholder

---

## ASSET-040 — vfx_objective_real_flash (3-frame strip)

| Field | Value |
|-------|-------|
| Category | VFX |
| Dimensions | 64×96 px per frame (3 frames) |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | vfx_objective_real_flash_01.png / _02.png / _03.png |
| Atlas | atlas_vfx (1024×1024) |
| M2 Priority | BLOCKING |
| Engine note | Spawn overlay entity at Z_OBJECTIVES + epsilon (e.g. 2.6). Advance TextureAtlas.index via timed system — Bevy 0.18 animation system is GLTF-targeted; sprite frame stepping is done manually or via Animator<usize>-style wrapper. |

**Visual Description:**
Three discrete frames for the "500ms hold → golden flash overlay → slot cleared" sequence. Frame 01: flat Arcane Gold `#F5C842` flood at ~60% opacity. Frame 02: overexposed white-gold bloom (`~#FFF5CC`) at ~90% opacity — luminance apex. Frame 03: warm gold at ~20% opacity fading to transparent edges. Hard-edged color floods sized to overlay precisely on the 64×96 objective sprite; no outlines needed.

**Art Bible Anchors:**
- §1 Structured Luminance Hierarchy: "Single high-contrast burst — revealed objective is the visual apex"
- §2 GAME_OVER: "Objective destruction uses overexposed bloom then settles"
- §4.1 Arcane Gold: the gold = significance payoff

**Generation Prompt:**
64×96 pixel VFX frame strip, 3 frames, golden flash overlay for objective reveal, Ankama cel-shaded game VFX. Frame 01: flat Arcane Gold `#F5C842` full-canvas flood at 60% opacity. Frame 02: overexposed white-gold bloom `#FFF5CC` at 90% opacity, radial bloom from center. Frame 03: warm gold `#F5C842` at 20% opacity fading to transparent edges. Hard-edged frames, pure color overlay, transparent PNG-32 straight alpha. Negative: cool color, blue tint, complex shapes, character art.

**Status:** Generated Placeholder

---

## ASSET-041 — vfx_objective_attack_ring

| Field | Value |
|-------|-------|
| Category | VFX |
| Dimensions | 96×96 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | vfx_objective_attack_ring_loop.png |
| Atlas | atlas_vfx (1024×1024) |
| M2 Priority | PLACEHOLDER (M3) |
| Engine note | Child entity of objective entity at local Z offset ~0.1. Alpha pulse via SpriteAlphaLens (project-custom Lens<Sprite> — deliverable of reveal-tween story). |

**Visual Description:**
Circular ring outline on 96×96 transparent canvas — ring diameter ~80px, 4px stroke, deep red-orange `#CC3A10` (cooler and darker than ATK orange `#E07020`, not full `#FF0000`). No interior fill. 1px Void shadow offset on outer ring edge. Pulse driven by code; sprite provides static ring geometry. Red-orange confirms "combat event on objective" per §4.2 semantic color.

**Art Bible Anchors:**
- §4.2 Semantic Color: red/Crimson = "combat event just occurred"
- §9.2: Never reach full `#FF0000` outside Sacrier events and damage numbers
- GDD: "Objective attack aura: red-orange pulsing ring child sprite"

**Generation Prompt:**
96×96 pixel VFX sprite, objective attack aura ring, Ankama game style, circular ring outline 4px stroke, ring diameter 80px centered on 96px canvas, red-orange stroke `#CC3A10`, no interior fill, 1px Void `#0D0D14` shadow offset on outer edge, transparent PNG-32 straight alpha, single static frame. Negative: interior fill, full red `#FF0000`, ATK orange `#E07020`, square shape.

**Status:** Needed

---

## ASSET-042 — vfx_spawn_range_pulse

| Field | Value |
|-------|-------|
| Category | VFX |
| Dimensions | 64×64 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | vfx_spawn_range_pulse_loop.png |
| Atlas | atlas_vfx (1024×1024) |
| M2 Priority | PLACEHOLDER |
| Engine note | Scale/alpha driven by TransformScaleLens (bevy_tweening built-in). Entity at Z_CELL_NODES = 1.0. |

**Visual Description:**
Radial pulse ring in Arcane Gold `#F5C842` — thin circular ring (3px stroke) at mid-travel position (radius ~24px from center on 64×64 canvas), soft gold inner-edge glow at 30% opacity. No interior fill. VFX impression from code-driven scale animation, not sprite complexity.

**Art Bible Anchors:**
- §4.1 Arcane Gold: "objectives, rewards" — newly unlocked spawn range is a reward/expansion event
- §7.6: "Fast translate + single weighted settle. Animations confirm state change; they do not entertain"

**Generation Prompt:**
64×64 pixel VFX sprite, spawn range expansion pulse ring, Ankama game style, circular ring 3px stroke, ring at mid-travel radius ~24px from center, Arcane Gold `#F5C842` stroke, soft gold inner-edge glow at 30% opacity, no interior fill, transparent PNG-32 straight alpha, single keyframe. Negative: full opaque fill, blue tint, cool color.

**Status:** Needed

---

## ASSET-043 — vfx_prism_collect_shimmer

| Field | Value |
|-------|-------|
| Category | VFX |
| Dimensions | 64×96 px |
| Format | PNG-32 (RGBA), straight alpha, strip ICC, sRGB 8-bit |
| Naming | vfx_prism_collect_shimmer_01.png |
| Atlas | atlas_vfx (1024×1024) |
| M2 Priority | PLACEHOLDER |
| Engine note | Child entity via ChildOf(unit_entity) — NOT deprecated set_parent(). Local Z offset ~0.05. Alpha via SpriteAlphaLens. |

**Visual Description:**
Semi-transparent Prism White `#EEF4FF` diagonal highlight sweep from bottom-left to top-right on a fully transparent 64×96 base — a band of near-white at 50% opacity fading to 0% at edges. Applied as a child overlay over the unit sprite; the ~400ms duration is code-driven. Gives the unit a "charged with prism energy" read without obscuring unit identity.

**Art Bible Anchors:**
- §4.1 Prism White `#EEF4FF`: "reserved for high-value magical events — always draws the eye"
- §6.4: Prism = high-value magical event; shimmer confirms collection

**Generation Prompt:**
64×96 pixel VFX overlay sprite, prism collection shimmer, Ankama game style, fully transparent base, single diagonal highlight sweep band from bottom-left to top-right, Prism White `#EEF4FF` at 50% peak opacity fading to 0% at both ends, cool blue-white tint, no outline, no shape boundary, compositing overlay, transparent PNG-32 straight alpha. Negative: gold color, warm tint, hard edge, opaque fill.

**Status:** Needed

---

## ASSET-044 — hp_bar_white_pixel_1x2

| Field | Value |
|-------|-------|
| Category | Reserved Atlas Frame (technical) |
| Dimensions | 1×2 px |
| Format | PNG-32 (RGBA), pure white #FFFFFF, fully opaque (alpha = 255) |
| Naming | hp_bar_white_pixel_1x2.png |
| Atlas | atlas_units (2048×2048) — corner position, 2px transparent gutter on all four sides |
| M2 Priority | BLOCKING |
| ⚑ Flag | 1px width violates §8.8 even-dimension rule. Documented exception: established Bevy HP bar pattern; 1px enables Transform.scale.x-driven bar width. 2px height satisfies even-height rule. Exception noted in atlas spec sheet — not a general exemption. |

**Visual Description:**
Zero artistic content. Single-pixel white column, two pixels tall. Programmatically generated — no AI image prompt. HP bar fill and background sprites reference this frame to share `atlas_units` Handle&lt;Image&gt; and batch with unit sprites in a single draw call. Color injected entirely via `Sprite.color` tinting (Green/Yellow/Red per F2 HP thresholds); no per-unit Handle&lt;ColorMaterial&gt; (which would break batching).

**Art Bible Anchors:**
- GDD board-rendering.md Rule 5/6: HP bar batching via shared atlas frame

**Generation Prompt:**
None — programmatic asset. Deliver: 1px wide × 2px tall, pure white `#FFFFFF`, PNG-32 straight alpha, fully opaque. Place in atlas with 2px transparent gutter on all four sides.

**Status:** File Present Placeholder

---

## ASSET-045 — snd_reveal_sting

| Field | Value |
|-------|-------|
| Category | Audio |
| Naming | snd_reveal_sting.ogg |
| Audio format | OGG/Vorbis VBR q6–8 (WASM delivery); WAV 24-bit PCM (master) |
| Sample rate | 44100 Hz |
| Channels | Stereo |
| Duration | ≤ 600ms total |
| M2 Priority | BLOCKING |
| Engine note | AudioPlayer::new(handle). Preload in bevy_asset_loader AssetCollection at session start — not lazy-loaded. |

**Sonic Character:**
Two-layer hit: dry percussive transient ("ink-stamp thud") with fast attack (0–5ms) and short tail (80–120ms decay) centered low-mid (200–400Hz), immediately followed by a short staccato three-note minor or minor-seventh chord sting in plucked strings or pizzicato ensemble, immediate attack, 300–500ms natural decay, no reverb tail beyond 300ms. Emotional register: "something arrived you didn't see" — tense but not alarming. Mix role: medium-loud brief punctuation event.

**Status:** Needed

---

## ASSET-046 — snd_unit_advance

| Field | Value |
|-------|-------|
| Category | Audio |
| Naming | snd_unit_advance.ogg |
| Audio format | OGG/Vorbis VBR q5–6 (delivery); WAV master |
| Sample rate | 44100 Hz |
| Channels | Mono (per-lane pitch variation ±5–10 semitones applied by code; source file is single neutral pitch) |
| Duration | < 200ms total |
| M2 Priority | BLOCKING |
| Engine note | Must layer cleanly up to 5× simultaneously. Mix level well below ASSET-045 and objective sounds. |

**Sonic Character:**
Light slightly organic footstep cluster — two to three rapid soft impacts in 80–150ms on stone cobble. Mid-range (300–800Hz), dry, no sustain (full decay within 200ms). Matter-of-fact, mechanical, inevitable. Repeating per-lane event; must not stack into noise at full board.

**Status:** Needed

---

## ASSET-047 — snd_objective_destroy_real

| Field | Value |
|-------|-------|
| Category | Audio |
| Naming | snd_objective_destroy_real.ogg |
| Audio format | OGG/Vorbis VBR q7–8 (delivery); WAV master |
| Sample rate | 44100 Hz |
| Channels | Stereo |
| Duration | 1.0–1.5s |
| M2 Priority | BLOCKING |
| Engine note | Preload in AssetCollection at session start — highest emotional priority event, cannot be lazy-loaded. Author alongside ASSET-048; the contrast between them is the core design. |

**Sonic Character:**
Two-phase event. Phase 1 (0–200ms): sharp stone-shattering explosion transient — dense broadband crunch, stone fracture layers, mid-high (500Hz–4kHz) crack attack plus low-end punch (80–150Hz) within first 50ms for physical weight. Phase 2 (200ms–1.2s): full orchestral stab or choir-and-brass chord in major or major-seventh voicing, blooming immediately after the transient clears, decaying over 800ms–1s. The musical hit must feel like the score acknowledging the moment — not a sound effect. Loudest non-music event in the game.

**Status:** Needed

---

## ASSET-048 — snd_objective_destroy_fake

| Field | Value |
|-------|-------|
| Category | Audio |
| Naming | snd_objective_destroy_fake.ogg |
| Audio format | OGG/Vorbis VBR q5–6 (delivery); WAV master |
| Sample rate | 44100 Hz |
| Channels | Mono |
| Duration | < 350ms |
| M2 Priority | BLOCKING |
| Engine note | Preload alongside ASSET-047. The absence of a Phase 2 musical hit IS the punchline — must be authored in contrast with ASSET-047. |

**Sonic Character:**
Single dry hollow thud — immediate attack (0–5ms), short decay (100–200ms), low-mid pitch (150–300Hz), narrow muffled frequency content like striking stone with a padded mallet or tapping a wooden crate. No musical hit, no sustain, no reverb tail, no orchestral swell. Must feel like a punctured expectation — physically present but anticlimactic. The contrast with ASSET-047's orchestral bloom should make the player almost laugh.

**Status:** Needed

---

## ASSET-049 — snd_prism_collect

| Field | Value |
|-------|-------|
| Category | Audio |
| Naming | snd_prism_collect.ogg |
| Audio format | OGG/Vorbis VBR q5–6 (delivery); WAV master |
| Sample rate | 44100 Hz |
| Channels | Stereo |
| Duration | 400–600ms (including ring tail) |
| M2 Priority | ADVISORY |

**Sonic Character:**
Bright short crystalline chime — single pitched strike on glass harmonica, singing bowl, or metallophone at high register (C5–G5), clean attack (0–10ms), natural ring tail 400–600ms. Cool and bright — high overtone content, not woody or muted. Optional subtle secondary harmonic a fifth above at 30% for "magical crystal" read. Mix role: rewarding, brief, mid-level — must not obscure concurrent board sounds.

**Status:** Needed

---

## ASSET-050 — snd_objective_attack

| Field | Value |
|-------|-------|
| Category | Audio |
| Naming | snd_objective_attack.ogg |
| Audio format | OGG/Vorbis VBR q6–7 (delivery); WAV master |
| Sample rate | 44100 Hz |
| Channels | Mono |
| Duration | 350–650ms |
| M2 Priority | ADVISORY |

**Sonic Character:**
Heavy, deep, resonant thud centered low (60–120Hz) — like a large wooden siege door absorbing a hit. Fast attack (0–10ms), short body (150–250ms) with slow low-frequency tail (200–400ms) giving impression of mass. No musical content, pure physical weight. Mix role: medium-loud focal event, significant enough to signal "the objective is in danger" without ASSET-047's climactic register.

**Status:** Needed

---

## ASSET-051 — snd_trap_trigger

| Field | Value |
|-------|-------|
| Category | Audio |
| Naming | snd_trap_trigger.ogg |
| Audio format | OGG/Vorbis VBR q6–7 (delivery); WAV master |
| Sample rate | 44100 Hz |
| Channels | Stereo |
| Duration | 250–400ms |
| M2 Priority | ADVISORY |
| ⚑ Flag | Two-part SFX (percussive hit + card flip). Recommend single composited file for guaranteed timing. Fallback: snd_trap_trigger_hit.ogg + snd_trap_trigger_flip.ogg with Timer delay — confirm with audio direction before production. |

**Sonic Character:**
Two-layer reveal event. Layer 1 (attack): sharp dry percussive hit — snare-like crack or wooden block strike (400Hz–2kHz), short attack and decay (0–150ms), representing mechanical trigger. Layer 2 (reveal, overlapping at +50ms): papery card-flip rustle or flutter (80–200ms), mid-register, like a card snapped face-up on a table. Together: "a mechanical snap, then something revealed." Emotional register: dry "caught you" quality, not threatening. Mix role: mid-level punctuation, forward in mix because it announces a strategic reveal.

**Status:** Needed
