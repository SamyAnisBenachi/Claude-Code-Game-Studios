# Asset Specs — System: class-system

> **Source**: design/gdd/class-system.md
> **Art Bible**: design/art/art-bible.md
> **Generated**: 2026-05-01
> **Status**: 34 assets specced / 0 approved / 0 in production / 0 done
> **Asset IDs**: ASSET-094 through ASSET-127

---

## Pre-Production Flags

Resolve these before beginning art production.

| Flag | Affected Assets | Resolution Path |
|------|-----------------|-----------------|
| **F-CS-1** Art Bible §4.4 is missing Ecaflip and Sadida class colors; Eniripsa is listed but not a playable class in this game. AD proposed Ecaflip `#D4A017`, Sadida `#5C7A3E`; TA canonical Wakfu references: Ecaflip `#E8C020`, Sadida `#5C9E3A`. AD must confirm hex and patch §4.4 before any Ecaflip/Sadida art begins. | ASSET-098, 099, 101, 102, 103, 104, 105, 110, 111 | AD patches art bible §4.4 |
| **F-CS-2** Figurines at 192×288 × 5 frames do not fit in any existing atlas. A new `atlas_figurines` (1024×1024, ~4 MB heap) is recommended. | ASSET-094 to 099 | AD + TA budget approval |
| **F-CS-3** Class Picker Panel Background dimensions depend on the UX spec. Do not begin art production until `/ux-design class-picker` is run and canvas dimensions are locked. | ASSET-112 | Run `/ux-design class-picker` |
| **F-CS-4** Rollback Zero-Reserve Warning may be a pure `bevy_ui` node (zero asset budget). Confirm with UI programmer before producing ASSET-119. | ASSET-119 | UI programmer decision |
| **F-CS-5** Bevy 0.18 audio API uses `AudioPlayer` + `PlaybackSettings` — verify against 0.18 release notes before audio production. Same risk as hand-ui-assets.md FLAG-2. | ASSET-123 to 127 | Engine programmer verification |
| **Silhouette test** Sadida token trio (ASSET-102, 103, 104) and all six class icons (ASSET-106 to 111) must pass the 64px / 32px silhouette-differentiation test before full art production begins. | ASSET-102, 103, 104, 106–111 | Art director sign-off |

---

## ASSET-094 — Figurine: Iop

| Field | Value |
|-------|-------|
| Category | Sprite / 2D Art |
| Dimensions | 192×288 px per frame, 5-frame horizontal strip (960×288 px) |
| Format | PNG-32 (RGBA), straight alpha, sRGB 8-bit, no ICC |
| Naming | `char_iop_figurine_idle_192x288.png` |
| Atlas | `atlas_figurines` (new 1024×1024 — requires F-CS-2 approval) |
| Frames | 5 — frame 0 = rest pose, frames 1–4 = idle micro-loop (blade quiver, ≤4px amplitude, 5–8s) |
| Engine Notes | `TextureAtlas` tile_size `UVec2::new(192, 288)`, columns=5, rows=1. Bevy 0.18 Required Components API. HP numeral on pedestal base is runtime `bevy_ui` text — not baked into sprite. |
| Flags | F-CS-2 |

**Visual Description:**
A 1:5–1:6 heroic-adult figure on a circular stone pedestal engraved with a broadsword glyph, two-handed sword raised overhead at 45°, chest thrust forward, feet planted wide with challenging eyes. Dominant warm orange-red `#E05A00` armor plates with flame-yellow `#F5C842` edge highlights and Void `#0D0D14` 2px outlines throughout; fully saturated flat local color, no gradients.

**Art Bible Anchors:**
- §5.4: heroic adult 1:5–1:6 ratio, circular pedestal with class symbol, idle ≤4px amplitude
- §4.4: Iop — Warm Orange-Red `#E05A00`, accent Flame Yellow
- §5.2: tall spiky hair, two-handed sword taller than body, wide-stance chest-out

**Generation Prompt:**
Ankama Wakfu Krosmaga style 2D game figurine sprite, Iop warrior chibi-heroic character, 1:5 head-to-body ratio, circular stone pedestal engraved with broadsword glyph, two-handed sword raised overhead, chest puffed out, feet planted wide, tall spiky orange hair, wide daring eyes, flat cel-shaded, dominant warm orange-red #E05A00 armor, flame yellow #F5C842 highlights, Void black #0D0D14 2px outlines, no gradients, fully saturated local color, white background, game sprite asset — negative: photorealistic, grimdark, flat vector, muted colors, gradients, 3D render

**Status:** Needed

---

## ASSET-095 — Figurine: Cra

| Field | Value |
|-------|-------|
| Category | Sprite / 2D Art |
| Dimensions | 192×288 px per frame, 5-frame horizontal strip |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `char_cra_figurine_idle_192x288.png` |
| Atlas | `atlas_figurines` (new) |
| Frames | 5 — bowstring tension oscillation idle, ≤4px amplitude |
| Flags | F-CS-2 |

**Visual Description:**
A slender heroic-adult figure on a circular pedestal bearing a bow-and-arrow glyph, posed side-on at 45° drawing a large composite bow skyward. An oversized quiver protrudes dramatically above the right shoulder, the horizontal bow arm creating a strong lateral cross-bar read at any scale. Forest Green `#2A6B3C` leather armor with quiver-brown `#8B6020` strap accents; Void `#0D0D14` 2px outlines.

**Art Bible Anchors:**
- §5.4: heroic adult, pedestal with class symbol
- §4.4: Cra — Forest Green `#2A6B3C`, accent quiver brown
- §5.2: bow held laterally (strong horizontal cross-bar), oversized quiver above shoulder
- §3.1: archer lateral-draw triangle silhouette principle

**Generation Prompt:**
Ankama Wakfu Krosmaga style 2D game figurine sprite, Cra archer chibi-heroic character, 1:5 head-to-body ratio, circular stone pedestal engraved with bow-and-arrow glyph, body side-on 45°, large composite bow drawn skyward, strong horizontal bow arm, oversized quiver above right shoulder, flat cel-shaded, forest green #2A6B3C leather armor, quiver brown #8B6020 straps, Void black #0D0D14 2px outlines, no gradients, white background — negative: front-facing symmetrical, photorealistic, flat vector, 3D render

**Status:** Needed

---

## ASSET-096 — Figurine: Sacrier

| Field | Value |
|-------|-------|
| Category | Sprite / 2D Art |
| Dimensions | 192×288 px per frame, 5-frame horizontal strip |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `char_sacrier_figurine_idle_192x288.png` |
| Atlas | `atlas_figurines` (new) |
| Frames | 5 — slow chest-breath rise-fall idle, 2–3px amplitude |
| Flags | F-CS-2 |

**Visual Description:**
A broad-shouldered heroic figure with exaggerated wide upper torso on a pedestal bearing an anchor-chain glyph, both arms spread wide and forward with chin raised — an open invitation to take a hit. Bone-white `#F0ECD5` tattoo scarring traced across arms and chest over blood-red `#8B1A2F` armor; a heavy iron chain trails from one wrist.

**Art Bible Anchors:**
- §5.4: heroic adult, pedestal; Sacrier — arms open, chin raised
- §4.4: Sacrier — Blood Red `#8B1A2F`, accent bone white (legitimate Crimson Slate context)
- §5.2: broad upper body, tattoo scarring, anchor chain, aggressive weight-forward

**Generation Prompt:**
Ankama Wakfu Krosmaga style 2D game figurine sprite, Sacrier warrior chibi-heroic character, 1:5 head-to-body ratio, circular stone pedestal engraved with anchor-chain glyph, arms spread wide chin raised defiantly, exaggerated broad shoulders, bone-white tattoo scarring on arms and chest, iron anchor chain trailing from wrist, flat cel-shaded, blood red #8B1A2F, bone white #F0ECD5 tattoo accents, Void black #0D0D14 2px outlines, white background — negative: timid pose, photorealistic, grimdark gore, flat vector, 3D render

**Status:** Needed

---

## ASSET-097 — Figurine: Xelor

| Field | Value |
|-------|-------|
| Category | Sprite / 2D Art |
| Dimensions | 192×288 px per frame, 5-frame horizontal strip |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `char_xelor_figurine_idle_192x288.png` |
| Atlas | `atlas_figurines` (new) |
| Frames | 5 — clock-hand tick micro-translate on wrist idle, 2px per tick |
| Flags | F-CS-2 |

**Visual Description:**
A slightly hunched conspiratorial figure on a pedestal bearing a clock-face glyph, with a large ornate clock-dial halo hovering behind the head as the dominant silhouette element. One finger points dramatically at an oversized pocket watch; head tilted down, eyes glancing up knowingly. Dark purple-blue `#2E1B6E` robes with clockface silver `#C0C8D4` filigree on watch, halo, and robe trim.

**Art Bible Anchors:**
- §5.4: heroic adult, pedestal; Xelor — finger pointing at watch face
- §4.4: Xelor — Dark Purple-Blue `#2E1B6E`, clockface silver
- §5.2: clock-dial halo behind head (primary silhouette), oversized pocket watch, hunched conspiratorial

**Generation Prompt:**
Ankama Wakfu Krosmaga style 2D game figurine sprite, Xelor time-mage chibi-heroic character, 1:5 head-to-body ratio, circular stone pedestal engraved with clock-face glyph, large ornate clock-dial halo behind head, one finger pointing at oversized pocket watch in other hand, slightly hunched forward, eyes glancing upward knowingly, flat cel-shaded, dark purple-blue #2E1B6E robes, clockface silver #C0C8D4 filigree, Void black #0D0D14 2px outlines, white background — negative: cheerful upright pose, photorealistic, flat vector, 3D render

**Status:** Needed

---

## ASSET-098 — Figurine: Ecaflip

| Field | Value |
|-------|-------|
| Category | Sprite / 2D Art |
| Dimensions | 192×288 px per frame, 5-frame horizontal strip |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `char_ecaflip_figurine_idle_192x288.png` |
| Atlas | `atlas_figurines` (new) |
| Frames | 5 — coin-flip or ear-twitch idle, 3–4px amplitude |
| Flags | F-CS-1 (Ecaflip color unconfirmed — AD `#D4A017` vs TA `#E8C020`), F-CS-2 |

**Visual Description:**
A lithe cat-eared humanoid at heroic proportions on a pedestal bearing a coin glyph, tossing a large gold coin upward with one hand while the other rests on the hip with a smirk. A long striped tail curls upward and to the left as a curved silhouette anchor. Warm yellow-gold `#D4A017` fur-trim and coin accents over warm cream base. ⚠️ AD sign-off required on Ecaflip color before production.

**Art Bible Anchors:**
- §5.4: heroic adult, pedestal; Ecaflip — tossing a coin upward, mischievous grin
- §4.4: ⚠️ Ecaflip color not in art bible — AD sign-off required (F-CS-1)
- §5.2: cat-eared, coin/dice motif, tail curling upward

**Generation Prompt:**
Ankama Wakfu Krosmaga style 2D game figurine sprite, Ecaflip cat-person chibi-heroic character, 1:5 head-to-body ratio, circular stone pedestal engraved with coin glyph, cat ears, tossing oversized gold coin upward, other hand on hip, mischievous grin, striped tail curling upward-left, flat cel-shaded, warm yellow-gold #D4A017 fur and coin, warm cream body, Void black #0D0D14 2px outlines, white background — negative: serious expression, photorealistic, flat vector, 3D render

**Status:** Needed

---

## ASSET-099 — Figurine: Sadida

| Field | Value |
|-------|-------|
| Category | Sprite / 2D Art |
| Dimensions | 192×288 px per frame, 5-frame horizontal strip |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `char_sadida_figurine_idle_192x288.png` |
| Atlas | `atlas_figurines` (new) |
| Frames | 5 — leaf/vine sway idle, ≤4px amplitude |
| Flags | F-CS-1 (Sadida color unconfirmed — AD `#5C7A3E` vs TA `#5C9E3A`), F-CS-2 |

**Visual Description:**
A grounded broad-footed heroic figure on a pedestal bearing a sprouting-seed glyph, one hand extended palm-up with a seedling sprouting from the palm — vines trail from wrist to pedestal. A small marionette-puppet dangles at the figure's side as secondary class signal. Earthy green-brown `#5C7A3E` layered clothing with vine-cord `#8B6914` belt and trim. ⚠️ AD sign-off required on Sadida color before production.

**Art Bible Anchors:**
- §5.4: heroic adult, pedestal; Sadida — surrounded by sprouting seeds
- §4.4: ⚠️ Sadida color not in art bible — AD sign-off required (F-CS-1)
- §5.2: nature/plant motif, vine or puppet, grounded earthy pose

**Generation Prompt:**
Ankama Wakfu Krosmaga style 2D game figurine sprite, Sadida nature-witch chibi-heroic character, 1:5 head-to-body ratio, circular stone pedestal engraved with sprouting seed glyph, one hand extended palm-up with seedling sprouting, vines trailing from wrist to pedestal, small marionette puppet dangling at side, wide-planted feet, calm nurturing expression, flat cel-shaded, earthy green-brown #5C7A3E clothing, vine-cord brown #8B6914 trim, Void black #0D0D14 2px outlines, white background — negative: aggressive pose, photorealistic, flat vector, 3D render

**Status:** Needed

---

## ASSET-100 — Token Sprite: Mummy / Momie (Xelor)

| Field | Value |
|-------|-------|
| Category | Sprite / 2D Art |
| Dimensions | 64×96 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `char_mummy_idle_64x96.png` |
| Atlas | `atlas_units` (2048×2048) |
| Frames | 1 |
| Engine Notes | `TextureAtlas` index lookup by token type. Stats HP=2/ATK=2/MP=3 from `cards.json` at runtime. |

**Visual Description:**
A chibi 1:1.5 Egyptian mummy unit — humanoid wrapped in linen bandages with Xelor purple-blue `#2E1B6E` dye-staining the wrapping strips for class identity. Two glowing violet eye-slits are the only facial feature; arms outstretched slightly forward in a classic mummy shuffle. Bandage outline is chunky and irregular — frayed ends give organic texture without losing the clean line principle at 64px.

**Art Bible Anchors:**
- §3.1/§5.1: chibi 1:1.5 ratio, no neck, glowing eye-slits as "large eyes"
- §4.4: Xelor `#2E1B6E` as dye stain on bandages for class affiliation read
- §1: silhouette reads as "outstretched undead" by shape alone at 64px board scale

**Generation Prompt:**
Ankama Wakfu Krosmaga style 2D game token sprite, chibi Egyptian mummy unit, 1:1.5 head-to-body ratio, no neck, glowing violet eye-slits as only facial feature, arms outstretched forward, chunky irregular linen bandage wrapping with frayed ends, Xelor purple-blue #2E1B6E dye staining bandage strips, flat cel-shaded, Void black #0D0D14 2px outlines, 64×96 board scale, white background — negative: photorealistic, adult proportions, 3D render, gradients

**Status:** Needed

---

## ASSET-101 — Token Sprite: Chacha Noir (Ecaflip)

| Field | Value |
|-------|-------|
| Category | Sprite / 2D Art |
| Dimensions | 64×96 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `char_chacha_noir_idle_64x96.png` |
| Atlas | `atlas_units` |
| Frames | 1 |
| Flags | F-CS-1 (Ecaflip color `#D4A017` unconfirmed — applied to eyes/claws only) |

**Visual Description:**
A shadow-form transformed Bow Meow — small feline unit with entirely near-black silhouette body and wisps of dark smoke trailing from outline edges. Ecaflip warm yellow-gold `#D4A017` glowing eyes (wide, slightly manic) and matching gold claw-tips are the only color relief. Silhouette reads immediately as a cat — pointed ears, arched back, tail raised — with shadow-smoke distortion signaling supernatural nature.

**Art Bible Anchors:**
- §3.1/§5.1: chibi proportions; cat-shape must read at 64px
- §4.4: ⚠️ Ecaflip `#D4A017` flagged — applied to glowing eyes/claws only
- §4: Void `#0D0D14` — body is near-void; slightly lighter smoke edge for silhouette separation

**Generation Prompt:**
Ankama Wakfu Krosmaga style 2D game token sprite, chibi shadow-cat Chacha Noir, entirely near-black shadow body with dark smoke wisps at silhouette edges, pointed cat ears, arched back, tail raised, slight crouch, glowing warm yellow-gold #D4A017 wide manic eyes and gold claw-tips, flat cel-shaded, slightly lighter dark outline for silhouette separation, 64×96 board scale, white background — negative: friendly cute pet, photorealistic, 3D render

**Status:** Needed

---

## ASSET-102 — Token Sprite: Madoll / La Folle (Sadida)

| Field | Value |
|-------|-------|
| Category | Sprite / 2D Art |
| Dimensions | 64×96 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `char_madoll_idle_64x96.png` |
| Atlas | `atlas_units` |
| Frames | 1 |
| Flags | F-CS-1 (Sadida color `#5C7A3E` unconfirmed). Must pass silhouette differentiation test vs. ASSET-103 and ASSET-104 at 64px. |

**Visual Description:**
A small stuffed ragdoll marionette — oversized round head with messy yarn-hair tuft, button eyes, stitched mouth-smile, visible thread joints at shoulders and knees. Pose is passive and slightly asymmetrical (one arm lower, head tilted) communicating support-unit. Sadida earthy green-brown `#5C7A3E` fabric body with vine-cord `#8B6914` stitching; handmade and endearing, not threatening.

**Art Bible Anchors:**
- §3.1/§5.1: chibi 1:1.5; wide-head/dangling-arm silhouette reads at 64px
- §4.4: ⚠️ Sadida `#5C7A3E` — AD sign-off required
- §9: ally/support token — smaller and simpler than hero figurines

**Generation Prompt:**
Ankama Wakfu Krosmaga style 2D game token sprite, chibi stuffed ragdoll marionette Sadida Madoll, 1:1.5 ratio, oversized round head with yarn hair tuft, button eyes, stitched smile, thread-joint shoulders and knees, one arm hanging lower, head tilted softly, passive stance, flat cel-shaded, earthy green-brown #5C7A3E fabric, vine-cord brown #8B6914 stitching, Void black #0D0D14 2px outlines, 64×96 board scale, white background — negative: aggressive pose, photorealistic, adult proportions, 3D render

**Status:** Needed

---

## ASSET-103 — Token Sprite: La Gonflable (Sadida)

| Field | Value |
|-------|-------|
| Category | Sprite / 2D Art |
| Dimensions | 64×96 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `char_la_gonflable_idle_64x96.png` |
| Atlas | `atlas_units` |
| Frames | 1 |
| Flags | F-CS-1 (Sadida color unconfirmed). Must be distinguishable from ASSET-102 and ASSET-104 at 64px by spherical silhouette. |

**Visual Description:**
A round, heavily inflated balloon-creature — near-perfect sphere body with stubby inflated limbs barely protruding and a round balloon-head with a content closed-eye smile on top. Sadida earthy greens with a lighter sky-suffused `#7AB055` highlight on the inflated belly dome; small vine-leaf patterns on surface. Floats slightly above the ground baseline.

**Art Bible Anchors:**
- §3.1: round sphere = defensive/supportive archetype — visually distinct from warrior triangle forms
- §4.4: ⚠️ Sadida `#5C7A3E` base, lighter `#7AB055` inflate highlight — flagged
- §1: perfect sphere silhouette reads instantly at 64px; must differentiate from ASSET-102 and ASSET-104

**Generation Prompt:**
Ankama Wakfu Krosmaga style 2D game token sprite, chibi inflatable balloon creature Sadida La Gonflable, near-perfect sphere body with stubby inflated arm and leg nubs, round balloon head, content closed-eyes smile, floating slightly above ground, small vine-leaf patterns on surface, flat cel-shaded, earthy green #5C7A3E with lighter inflate highlight #7AB055 on belly dome, Void black #0D0D14 2px outlines, 64×96 board scale, white background — negative: aggressive sharp silhouette, photorealistic, 3D render

**Status:** Needed

---

## ASSET-104 — Token Sprite: La Sacrifiée (Sadida)

| Field | Value |
|-------|-------|
| Category | Sprite / 2D Art |
| Dimensions | 64×96 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `char_la_sacrifiee_idle_64x96.png` |
| Atlas | `atlas_units` |
| Frames | 1 |
| Flags | F-CS-1 (Sadida color unconfirmed). Must be distinguishable from ASSET-102 at 64px via X-eye and reaching-gesture silhouette. |

**Visual Description:**
Closely derived from the Madoll ragdoll design (ASSET-102) but with damage-state visual language: a cracked X-stitch eye, bandage wrapping on one arm, tear-drop stitching under the eyes, thread-bare patches with exposed cotton batting at joints, and one hand slightly outstretched in a reaching "last act" gesture. Same Sadida palette as Madoll for token family cohesion; reads as fragile and tragic, not grotesque.

**Art Bible Anchors:**
- §3.1/§5.1: chibi 1:1.5; X-eye and reaching hand must differentiate this from ASSET-102 at 64px
- §4.4: ⚠️ Sadida `#5C7A3E` — same palette as Madoll for visual family cohesion, flagged
- §9: ally token — tragic not monstrous

**Generation Prompt:**
Ankama Wakfu Krosmaga style 2D game token sprite, chibi damaged stuffed ragdoll Sadida La Sacrifiée, 1:1.5 ratio, cracked X-stitch eye replacing one button eye, bandage wrapping on one arm, tear-drop stitching under eyes, thread-bare worn patches, exposed cotton batting at joints, one arm outstretched in reaching gesture, slight forward lean, flat cel-shaded, earthy green-brown #5C7A3E worn fabric, Void black #0D0D14 2px outlines, 64×96 board scale, white background — negative: healthy intact appearance, photorealistic, 3D render

**Status:** Needed

---

## ASSET-105 — Graine / Seed Cell Floor Marker (Sadida)

| Field | Value |
|-------|-------|
| Category | Environment |
| Dimensions | 32×32 px per frame |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `env_sadida_seed_idle_32x32.png` (5-frame idle strip, 160×32 px) + `env_sadida_seed_active_32x32.png` (single active frame) |
| Atlas | `atlas_board` (1024×1024) — two separate atlas entries |
| Frames | 6 total: 5-frame idle pulse strip + 1 active frame |
| Engine Notes | Idle strip: `TextureAtlas` tile_size `UVec2::new(32, 32)`, columns=5. Active frame: single sprite swap on walk-over trigger. Max 1 seed per cell enforced at runtime (CS-7). |
| Flags | F-CS-1 (Sadida glow color `#5C7A3E` unconfirmed) |

**Visual Description:**
A teardrop-seed shape intaglio-etched into the cobblestone board surface — carved into stone like a stamp, filled with Sadida earthy green `#5C7A3E` glow from within. Mirrors the Prism Cell (§6.4) etched-into-stone visual language but uses an organic teardrop rather than a geometric star. Idle: 2s soft pulse, 30–70% opacity. Active frame: glow intensifies to `#8AC86A` with a tiny seedling-sprout surging upward, then returns to idle.

**Art Bible Anchors:**
- §6.4: Prism Cell etched-into-stone language — same treatment, organic shape variant
- §4.4: ⚠️ Sadida `#5C7A3E` glow — flagged
- §3.1: teardrop organic shape deliberately avoids geometric star (Prism) and diamond (reserve) reads
- §9.2: green here is Sadida class mechanic — context distinguishes from Cra class color

**Generation Prompt:**
Ankama Wakfu Krosmaga style 2D game board cell floor marker, 32×32 px, teardrop seed shape intaglio-etched into dark cobblestone tile surface, soft green glow from within etched cavity, earthy green #5C7A3E base glow, organic hand-carved look, flat cel-shaded, Void dark stone surrounding tile, small seedling sprout icon at center, top-down board perspective, distinct from geometric Prism Cell star marker — negative: photorealistic, 3D render, blue or red glow, geometric crystal shape, bright neon

**Status:** Needed

---

## ASSET-106 — Class Icon: Iop

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 32×32 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_class_icon_iop_default_32x32.png` |
| Atlas | `atlas_ui_hud` (1024×1024) |
| Frames | 1 (runtime `Sprite::color` tint for hover/selected — no second frame) |

**Visual Description:**
Two-handed broadsword pointing straight upward — blade occupying ~70% of icon height, crossguard at midpoint with a simple beveled edge highlight. Three-color flat: silver blade `#C8CDD6`, warm orange crossguard `#E05A00`, dark brown hilt wrap `#3D2010`; Void `#0D0D14` 2px outline. At 24px the orange crossguard reads as the class-distinctive element.

**Art Bible Anchors:**
- §7.5: 32–48px class icon, 3-color flat + Void 2px outline, legible at 24px
- §4.4: Iop `#E05A00` on crossguard — class color anchor
- §3.1: upward-thrust sword = Iop warrior silhouette

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D UI icon, flat class glyph, two-handed broadsword pointing straight upward, crossguard at center, beveled blade highlight, 3-color flat: silver blade #C8CDD6, warm orange crossguard #E05A00, dark brown hilt #3D2010, Void black #0D0D14 2px outline, 32×32, white background, legible at 24px — negative: gradients, glow, 3D render, drop shadows, complex filigree

**Status:** Needed

---

## ASSET-107 — Class Icon: Cra

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 32×32 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_class_icon_cra_default_32x32.png` |
| Atlas | `atlas_ui_hud` |
| Frames | 1 |

**Visual Description:**
Recurve bow rotated horizontal (laying sideways) with arrow nocked, string pulled back mid-draw — the strong horizontal cross-bar is the dominant silhouette, directly echoing §5.2 "bow held laterally." Three-color flat: forest green bow limbs `#2A6B3C`, quiver-brown arrow shaft `#8B6020`, bone-white bowstring `#F0ECD5`; Void `#0D0D14` 2px outline.

**Art Bible Anchors:**
- §7.5: class icon spec
- §4.4: Cra `#2A6B3C` + quiver brown
- §5.2: bow held laterally — icon mirrors this silhouette cue

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D UI icon, flat class glyph, recurve bow held horizontally with arrow nocked string drawn back, strong horizontal silhouette, 3-color flat: forest green bow #2A6B3C, quiver-brown arrow #8B6020, bone-white bowstring #F0ECD5, Void black #0D0D14 2px outline, 32×32, white background, legible at 24px — negative: vertical composition, gradients, glow, 3D render

**Status:** Needed

---

## ASSET-108 — Class Icon: Sacrier

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 32×32 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_class_icon_sacrier_default_32x32.png` |
| Atlas | `atlas_ui_hud` |
| Frames | 1 |

**Visual Description:**
Heavy maritime iron anchor with 2–3 simple oval chain links draped across its crown — legible at 24px. Three-color flat: anchor iron-grey `#5A6070`, chain links bone-white `#F0ECD5`, rust-blood accent on anchor shank `#8B1A2F`; Void `#0D0D14` 2px outline.

**Art Bible Anchors:**
- §7.5: class icon spec
- §4.4: Sacrier `#8B1A2F` as accent — legitimate Crimson Slate context
- §5.2: anchor chain as defining class motif

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D UI icon, flat class glyph, heavy maritime anchor with thick chain links draped across crown, 2–3 oval chain links, 3-color flat: iron-grey anchor #5A6070, bone-white chain #F0ECD5, blood-rust accent on shank #8B1A2F, Void black #0D0D14 2px outline, 32×32, white background, legible at 24px — negative: gradients, glow, delicate thin lines, 3D render

**Status:** Needed

---

## ASSET-109 — Class Icon: Xelor

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 32×32 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_class_icon_xelor_default_32x32.png` |
| Atlas | `atlas_ui_hud` |
| Frames | 1 |

**Visual Description:**
Pocket watch face viewed straight-on — tick marks at cardinal positions, two clock hands set to ~11:58. Three-color flat: silver watch case `#C0C8D4`, dark purple-blue watch face `#2E1B6E`, bone-white clock hands `#F0ECD5`; Void `#0D0D14` 2px outline. Circular case + two hands compress to a clean clock-face read at 24px.

**Art Bible Anchors:**
- §7.5: class icon spec
- §4.4: Xelor `#2E1B6E` + clockface silver
- §5.2: pocket watch as defining class prop

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D UI icon, flat class glyph, pocket watch face straight-on, clock hands at 11:58, tick marks at cardinal positions, circular case, 3-color flat: silver case #C0C8D4, dark purple-blue face #2E1B6E, bone-white hands #F0ECD5, Void black #0D0D14 2px outline, 32×32, white background, legible at 24px — negative: gradients, glow, digital numerals, 3D render

**Status:** Needed

---

## ASSET-110 — Class Icon: Ecaflip

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 32×32 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_class_icon_ecaflip_default_32x32.png` |
| Atlas | `atlas_ui_hud` |
| Frames | 1 |
| Flags | F-CS-1 (Ecaflip color `#D4A017` unconfirmed) |

**Visual Description:**
Gold coin at a slight 3/4-view tilt (ellipse silhouette to differentiate from a flat circle) with a cat paw-print embossed on the visible face. Three-color flat: warm gold coin `#D4A017`, darker amber paw emboss `#A07010`, cream rim highlight `#F5E4AA`; Void `#0D0D14` 2px outline. The paw emboss disambiguates from any generic coin icon. ⚠️ Color requires AD sign-off.

**Art Bible Anchors:**
- §7.5: class icon spec
- §4.4: ⚠️ Ecaflip color not in art bible — AD sign-off required
- §5.2: coin/dice motif as defining class object

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D UI icon, flat class glyph, gold coin slight 3/4-tilt ellipse, cat paw-print embossed on visible face, 3-color flat: warm gold #D4A017, amber paw emboss #A07010, cream rim #F5E4AA, Void black #0D0D14 2px outline, 32×32, white background, legible at 24px — negative: flat circle no tilt, gradients, glow, 3D render

**Status:** Needed

---

## ASSET-111 — Class Icon: Sadida

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 32×32 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_class_icon_sadida_default_32x32.png` |
| Atlas | `atlas_ui_hud` |
| Frames | 1 |
| Flags | F-CS-1 (Sadida color `#5C7A3E` unconfirmed) |

**Visual Description:**
Seedling sprout — single stem with two symmetrical leaf-pairs spreading outward and a teardrop seed shape at the base (echoing ASSET-105 Graine marker for visual family cohesion). Three-color flat: earthy green `#5C7A3E` stem/leaves, darker brown-green `#3D5528` seed base, lighter `#7AB055` leaf highlight; Void `#0D0D14` 2px outline. The Y-shape silhouette is unique among the six class icons. ⚠️ Color requires AD sign-off.

**Art Bible Anchors:**
- §7.5: class icon spec
- §4.4: ⚠️ Sadida color not in art bible — AD sign-off required
- §6.4/ASSET-105: teardrop seed base creates visual family cohesion across Sadida board elements

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D UI icon, flat class glyph, seedling sprout with single stem, two symmetrical leaf pairs, teardrop seed base, 3-color flat: earthy green #5C7A3E stem and leaves, darker brown-green #3D5528 seed base, lighter green highlight #7AB055, Void black #0D0D14 2px outline, 32×32, white background, Y-branch silhouette, legible at 24px — negative: full tree, complex plant, blue or purple colors, gradients, glow, 3D render

**Status:** Needed

---

## ASSET-112 — Class Picker Panel Background

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | TBD — pending `/ux-design class-picker` (F-CS-3). Placeholder: 512×512 px (POT, even). |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_class_picker_panel_default.png` (size suffix omitted — dimensions unresolved) |
| Atlas | Standalone file (LOBBY-only; load + unload with screen). If Bevy 0.18 `ImageScaleMode::Sliced` is confirmed, can be 128×128 9-slice tile and move to `atlas_ui_hud`. |
| Frames | 1 |
| Flags | F-CS-3 — do not begin art production until UX spec locks canvas dimensions. Verify `ImageScaleMode::Sliced` in Bevy 0.18 (post-cutoff) before assuming 9-slice works. |

**Visual Description:**
Deep Ink Blue `#1A2D5A` panel with subtle vignette darkening at extreme edges and a warm amber `#E87C1E` ambient gradient at ~15% opacity from above-center (late-afternoon tavern chandelier, §2 LOBBY mood). A faint repeating diamond-grid watermark in slightly lighter `#253D6E` gives surface texture implying a card game table without distraction. No character art, no class-specific colors.

**Art Bible Anchors:**
- §2: LOBBY mood — neutral-warm ambient, composed, watchful, late-afternoon tavern light
- §4: Ink Blue `#1A2D5A` as UI background base
- §4: Auction Amber `#E87C1E` at very low opacity as warm ambient accent only
- §1: composed, not celebratory

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D game UI lobby background panel, deep navy ink blue #1A2D5A base, subtle vignette darkening at edges, warm amber #E87C1E 15% opacity ambient gradient from above-center simulating tavern chandelier, faint repeating diamond-grid watermark in lighter ink blue #253D6E, flat cel-shaded, no characters no class art, composed deliberate strategic atmosphere, 512×512 game UI panel asset — negative: gradients dominant, busy illustration, character art, bright celebratory, neon, 3D render

**Status:** Needed

---

## ASSET-113 — Class Option Tile (reusable frame)

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 120×180 px per state (Card Display tier — §8.1) |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_class_tile_default_120x180.png` / `ui_class_tile_hover_120x180.png` / `ui_class_tile_active_120x180.png` |
| Atlas | `atlas_ui_hud` (3 states × 120×180 = 64,800 px²) |
| Frames | 3 static frames — one per state, swapped at runtime |
| Engine Notes | Runtime class-color tint via `Sprite::color` multiplier — frame is neutral chrome, class color fills interior via overlay or tint. Coordinate composition with UI programmer. |

**Visual Description:**
Rectangular card-like frame (120×180 px, §8.1 Card Display proportions) with 4px slightly-rounded corners and a prominent 3px Void `#0D0D14` outer border. Interior transparent at runtime. A thin inner bevel white at 50% opacity sits just inside the border. Top 20% is a header band. Three states: Default (Void border), Hover (border brightens to class accent, 30% inner glow), Active/Selected (4px Arcane Gold `#F5C842` border + gold inner glow).

**Art Bible Anchors:**
- §3.2: Krosmaga card frame anatomy — rectangular portrait, slightly-rounded corners
- §4: Arcane Gold `#F5C842` premium UI chrome for selected/committed state
- §3.4: flat angular chip UI language

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D game UI element, rectangular card-frame tile, 120×180px, 4px rounded corners, 3px Void black #0D0D14 outer border, thin inner bevel white highlight line, top 20% header band, transparent interior fill area, 3-state variants: default/hover(class accent glow 30%)/selected(4px Arcane Gold #F5C842 border gold inner glow), Krosmaga card frame aesthetic, white background — negative: gradients dominant, rounded pill, beveled 3D emboss, drop shadows, complex ornate border

**Status:** Needed

---

## ASSET-114 — Class Locked Indicator Badge

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 32×32 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_class_locked_badge_default_32x32.png` |
| Atlas | `atlas_ui_hud` |
| Frames | 1 |

**Visual Description:**
Circular badge (Arcane Gold `#F5C842` filled disc, Void `#0D0D14` 2px circular border) with a simplified padlock icon at center: bone-white `#F0ECD5` shackle arch, deeper gold `#B8901A` lock body, Void keyhole slot. Overlaid top-right on the class option tile (ASSET-113). Gold = "committed/premium" semantic. Must be distinguishable from ASSET-117 (Garde-Temps Exhausted Badge) — different glyph and color.

**Art Bible Anchors:**
- §4: Arcane Gold — premium/confirmed state
- §7.5: flat outlined badge, 2px Void outline
- §1: circular shape reads as overlay vs. rectangular tile background

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D game UI badge, 32×32 circular badge, Arcane Gold #F5C842 filled disc, Void black #0D0D14 2px circular border, centered simplified padlock: white shackle arch #F0ECD5, deeper gold lock body #B8901A, Void black keyhole, 3-color flat, crisp at 24px, tile overlay badge, white background — negative: gradients, glow, complex ornate lock, rectangular shape, 3D render

**Status:** Needed

---

## ASSET-115 — "Waiting for Opponent" Placeholder Tile

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 120×180 px (same canvas as ASSET-113 for slot consistency) |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_class_waiting_tile_default_120x180.png` |
| Atlas | `atlas_ui_hud` |
| Frames | 1 (pulse animation is runtime `bevy_tweening` opacity oscillation — no extra frames) |
| Engine Notes | Replaced at runtime when `S2CClassReveal` arrives. |

**Visual Description:**
Same frame dimensions and border chrome as ASSET-113 but with Ink Blue `#1A2D5A` fill — neutral, non-class-colored. Center: large question mark glyph at 60% opacity Ivory `#F7F0DC`. Header band shows "Waiting…" label in Ivory. Treatment must feel deliberately neutral so the revealed class tile feels like an emergence.

**Art Bible Anchors:**
- §4: Ink Blue `#1A2D5A` — pending/unknown state
- §4: Ivory `#F7F0DC` — neutral text
- §1: question mark is the visual apex — "unknown" not "empty"
- §9: no class-specific colors

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D game UI placeholder tile, 120×180px, same proportions as class option tile, Ink Blue #1A2D5A fill, Void black #0D0D14 2px border, large centered question mark glyph 60% opacity ivory #F7F0DC, small "Waiting…" label in header band in ivory, neutral non-class-colored, flat, white background — negative: class colors, gradients, glow, complex illustration, 3D render

**Status:** Needed

---

## ASSET-116 — Sinistro Objective Indicator Icon

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 24×24 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_sinistro_indicator_default_24x24.png` |
| Atlas | `atlas_ui_hud` |
| Frames | 1 (persistent static — removed on `SinistroDestroyed` event per NP-6) |
| Engine Notes | Positioned at objective tile corner via world-space Sprite at fixed offset. Must not occlude HP displays. |

**Visual Description:**
A 24px flat icon — a clock face silhouette with a dark crescent cut from its lower half ("shadowed clock" motif). Xelor purple-blue `#2E1B6E` fill on the clock face, clockface silver `#C0C8D4` hands, Void `#0D0D14` 2px outline. Signals "this objective is under Xelor temporal influence." Must be visually distinct from objective HP, fake/real indicators, and other objective decorations at objective tile scale.

**Art Bible Anchors:**
- §4.4: Xelor `#2E1B6E` + clockface silver
- §7.5: flat outlined icon, 2px Void, identifiable at 16px
- §1: corner badge must not compete with primary objective HP display

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D game UI micro-icon, 24×24px, clock face circular silhouette with dark crescent shadow cut from lower half, Xelor purple-blue #2E1B6E clock face fill, silver clock hands #C0C8D4, Void black #0D0D14 2px outline, 3-color flat, reads clearly at 16px, objective-tile corner overlay badge, white background — negative: gradients, glow, complex filigree, bright colors, 3D render

**Status:** Needed

---

## ASSET-117 — Garde-Temps Exhausted Badge

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 32×32 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_garde_temps_exhausted_badge_default_32x32.png` |
| Atlas | `atlas_ui_hud` |
| Frames | 1 (permanent state — no animation) |
| Engine Notes | Tooltip ("Already used this game.") is runtime `bevy_ui` text. State driven by `garde_temps_used_this_game >= garde_temps_per_game_cap`. Coordinate with ASSET-118 and hand-ui mana-insufficient state — three visually distinct treatments required. |

**Visual Description:**
A pill-rounded rectangular badge with fully desaturated mid-grey `#6A6A6A` fill — communicating "permanently unavailable." Centered: a small Xelor clock-face icon with a bold "×" overlay in Void `#0D0D14` (crossed-out clock). Small "USED" text in Ivory capitals beside the clock. Desaturated grey must be visually distinct from the temporary reserve-insufficient state (ASSET-118, blue diamond language).

**Art Bible Anchors:**
- §4.4: Xelor clock motif maintained even in exhausted state
- §4: desaturated grey = permanently unavailable — distinct from temporary insufficiency
- §3.4: pill-rounded chip matches flat angular chip UI language

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D game UI card badge, 32×32px, pill-rounded badge, desaturated mid-grey #6A6A6A fill, Void black #0D0D14 2px border, centered small clock-face icon with bold X cross-out overlay, "USED" text in small ivory capitals beside clock, permanently-consumed visual language, flat 3-color, white background — negative: gradients, glow, bright colors, class primary colors, 3D render

**Status:** Needed

---

## ASSET-118 — Reserve Insufficient Indicator Glyph

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 24×24 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_reserve_insufficient_glyph_default_24x24.png` |
| Atlas | `atlas_ui_hud` |
| Frames | 1 |
| Engine Notes | Paired with runtime `bevy_ui` text "R" prefix label. Coordinate icon + text layout with UI programmer. Must be distinguishable from mana-insufficient (ASSET-056, orange amber family). |

**Visual Description:**
A diamond shape (echoing the reserve blue diamond §3.4) with a bold vertical slash through the center — "no reserve" symbol. Reserve blue `#2AA8C4` diamond outline with 40% opacity fill; Void `#0D0D14` slash and 2px outline. The blue diamond anchors it to the reserve mechanic; the slash transforms meaning from "reserve value" to "reserve unavailable."

**Art Bible Anchors:**
- §3.4: reserve blue diamond UI motif — this glyph echoes it
- §4: HP Teal `#2AA8C4` adapted as reserve indicator per diamond lineage (slash differentiates from HP stat)
- §4: Color as Information — blue = reserve family; slash = unavailable
- §1: must be distinguishable from mana-insufficient (orange/amber) at a glance

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D game UI micro-glyph, 24×24px, diamond shape with bold vertical slash through center, reserve-insufficient indicator, blue diamond outline #2AA8C4 with 40% opacity blue fill, Void black #0D0D14 slash and 2px outline, flat 3-color, echoes reserve blue diamond UI language, distinct from orange mana indicators, white background — negative: circular shape, orange or red colors, gradients, 3D render

**Status:** Needed

---

## ASSET-119 — Rollback Zero-Reserve Inline Warning

| Field | Value |
|-------|-------|
| Category | UI |
| Dimensions | 160×24 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `ui_rollback_warning_panel_default_160x24.png` |
| Atlas | `atlas_ui_hud` |
| Frames | 1 (static backing panel; warning text is runtime `bevy_ui` Text node) |
| Engine Notes | GDD: visible without hover. Spawn as persistent `Sprite` child of staged-card entity when `reserve == 0` AND Rollback is staged. *(F-CS-4: confirm with UI programmer if pure `bevy_ui` Node + BackgroundColor suffices — if so, this asset is zero-budget.)* |

**Visual Description:**
A full-width horizontal strip (160×24 px) with Auction Amber `#E87C1E` background and Void `#0D0D14` thin 1px top/bottom border. Left: a small triangle warning glyph (⚠, ~14px) in Void. Right: warning text in Ivory `#F7F0DC`. Always-visible — never hover-triggered.

**Art Bible Anchors:**
- §4: Auction Amber `#E87C1E` — warning/escalation color
- §4: Ivory `#F7F0DC` text on saturated backgrounds
- §1: inline visible warning is the correct hierarchy for time-pressure UI

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D game UI inline warning strip, 160×24px, Auction Amber #E87C1E background, Void black #0D0D14 thin 1px top/bottom border, left-aligned small triangle warning icon in Void black, text "No reserve — Rollback will have no effect." in ivory #F7F0DC small legible font, flat, always-visible non-hover strip, white background — negative: modal popup, rounded corners, red background, 3D render

**Status:** Needed

---

## ASSET-120 — Sang Méprise Reveal Marker — Real variant

| Field | Value |
|-------|-------|
| Category | VFX / Particles |
| Dimensions | 64×96 px (matches objective tile canvas — clean overlay with no scaling) |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `vfx_sang_meprise_real_64x96.png` |
| Atlas | `atlas_vfx` (1024×1024) |
| Frames | 1 (fade-in/fade-out via `bevy_tweening` alpha ramp; 4s halo pulse is runtime tween — no extra frames) |
| Engine Notes | Spawned as `Sprite` entity over objective tile z-stack; cleared on RESOLUTION end event. `Sprite::color.a` driven by tween. |

**Visual Description:**
A soft six-pointed starburst halo in Arcane Gold `#F5C842` overlaying a full objective tile (64×96 px), with a small centered Void `#0D0D14` checkmark inside a small gold disc. The halo radiates at ~30% opacity at the periphery, denser at center. **CRITICAL §9.2: NOT green.** Arcane Gold = "real/confirmed" — derives from "Arcane Gold = objectives of significance."

**Art Bible Anchors:**
- §9.2 CRITICAL: never green for real objective (Cra owns green)
- §4: Arcane Gold `#F5C842` — objective significance
- ASSET-114 design lineage: gold disc + Void symbol = consistent confirmed/premium language
- §1: must read on top of objective tile without occluding HP counter

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D game VFX overlay, Sang Méprise reveal marker REAL variant, 64×96px, soft six-pointed starburst halo in Arcane Gold #F5C842 over objective tile area, small checkmark in Void black #0D0D14 centered on small gold disc, gentle radial glow at 30% periphery, flat cel-shaded glow effect, white background — negative: green any shade, red any shade, class-specific colors, gradients dominant, 3D render

**Status:** Needed

---

## ASSET-121 — Sang Méprise Reveal Marker — Fake variant

| Field | Value |
|-------|-------|
| Category | VFX / Particles |
| Dimensions | 64×96 px |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `vfx_sang_meprise_fake_64x96.png` |
| Atlas | `atlas_vfx` |
| Frames | 1 |
| Engine Notes | Same spawn/clear behavior as ASSET-120. Two separate atlas entries for real and fake variants. |

**Visual Description:**
A thin broken/wispy elliptical ring in Prism White `#EEF4FF` overlaying the objective tile — incomplete (three-quarters rendered with wispy gaps), communicating "void/illusion." A "?" glyph in pale blue-grey `#8899BB` at the center. Deliberately muted and cool, contrasting with ASSET-120's warm solid starburst in shape (broken ring vs. starburst), color (cool vs. warm), and fullness (broken vs. solid). **CRITICAL §9.2: NOT red.**

**Art Bible Anchors:**
- §9.2 CRITICAL: never red for fake objective (Sacrier/combat owns red)
- §4: Prism White `#EEF4FF` — spectral/illusory effects (fake = an illusion)
- §1: shape contrast with ASSET-120 ensures real/fake distinguishable without color alone (colorblind safety)

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D game VFX overlay, Sang Méprise reveal marker FAKE variant, 64×96px, thin broken wispy elliptical ring in Prism White #EEF4FF, incomplete three-quarter ring with wisp gaps, question mark glyph in pale blue-grey #8899BB centered in ring, cool spectral mist aesthetic, deliberately muted, flat cel-shaded, visually distinct from REAL variant (broken ring vs. solid starburst, cool vs. warm), white background — negative: red any shade, green any shade, warm gold, solid filled circle, 3D render

**Status:** Needed

---

## ASSET-122 — Xelorium Drain Flash

| Field | Value |
|-------|-------|
| Category | VFX / Particles |
| Dimensions | 48×48 px per frame, 3-frame strip (144×48 px total) |
| Format | PNG-32 straight alpha, sRGB 8-bit |
| Naming | `vfx_xelorium_drain_flash_loop.png` |
| Atlas | `atlas_vfx` (confirm with UI programmer — may need `atlas_ui_hud` if HUD uses a separate render layer) |
| Frames | 3 horizontal strip — frame 0: glow start; frame 1: peak suction; frame 2: dissipate |
| Engine Notes | `TextureAtlas` tile_size `UVec2::new(48, 48)`, columns=3, rows=1. One-shot playback then despawn (~250ms total). Positioned over opponent mana counter HUD element. |

**Visual Description:**
3-frame drain animation overlay. Frame 0: Xelor purple-blue `#2E1B6E` burst radiating outward from mana pip cluster. Frame 1 (peak): clockface silver `#C0C8D4` tendrils spiraling inward — the suction keyframe. Frame 2: brief Prism White `#EEF4FF` flash at maximum compression, then fades. Inward-collapse metaphor — mana is stolen, not destroyed. Crimson Slate explicitly excluded: this is a steal, not combat damage.

**Art Bible Anchors:**
- §4.4: Xelor purple-blue `#2E1B6E` + clockface silver — Xelor owns this VFX
- §4: Prism White for magical peak compression
- §4: Crimson Slate `#8B1A2F` explicitly NOT used — "things that hurt" must not be invoked for a mana steal

**Generation Prompt:**
Ankama Wakfu Krosmaga 2D game VFX animation frames, Xelorium mana drain effect, 48×48px per frame 3-frame strip, frame 0: Xelor purple-blue #2E1B6E burst radiating outward; frame 1: silver #C0C8D4 tendrils spiraling inward compression suction; frame 2: brief Prism White #EEF4FF flash at peak then dissipate, inward collapse visual metaphor, cool purple-silver-white palette only, flat cel-shaded, white background — negative: Crimson Slate red, warm orange, outward explosion, damage number, 3D render

**Status:** Needed

---

## ASSET-123 — Class Select Hover SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Format | OGG Vorbis, 96 kbps mono |
| Duration | 80–120ms |
| Naming | `audio_class_select_hover.ogg` |
| Atlas | N/A — `assets/audio/ui/` |
| Loop | No (one-shot) |
| Flags | F-CS-5. Implement with 50ms debounce — six tiles can trigger this in rapid succession. |

**Sonic Character:**
Short UI hover — single light percussion transient (warm hollow woodblock or light mallet) with brief resonant tail. Mid-range, dry, slight small-room reverb. Exploratory, not committing. Volume: -12 to -9 dBFS.

**Status:** Needed

---

## ASSET-124 — Class Confirm / Ready SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Format | OGG Vorbis, 96 kbps mono |
| Duration | 200–280ms |
| Naming | `audio_class_confirm_ready.ogg` |
| Atlas | N/A — `assets/audio/ui/` |
| Loop | No (one-shot) |
| Flags | F-CS-5. Must be distinguishable from ASSET-019 (DRAFT_INITIAL Ready Signal) — coordinate timbre. |

**Sonic Character:**
Two-transient design: initial resonant wooden/bone "clack" (commitment moment) immediately followed by a brief ascending two-note pentatonic chime (+4 semitones, e.g., C → E) with warm metallic bell timbre. 400ms hall reverb tail. Decisive and satisfying — a chess piece placed firmly. Volume: -8 to -6 dBFS (audibly louder than ASSET-123).

**Status:** Needed

---

## ASSET-125 — Opponent Class Reveal SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Format | OGG Vorbis, 96 kbps mono |
| Duration | 350–500ms |
| Naming | `audio_class_opponent_reveal.ogg` |
| Atlas | N/A — `assets/audio/ui/` |
| Loop | No (one-shot) |
| Flags | F-CS-5. Brief enough not to overlap with LOBBY → DRAFT_INITIAL transition audio. |

**Sonic Character:**
Three-layer: (1) low resonant drum thump at onset for weight; (2) mid-range rising harp-like arpeggio ascending three notes over 200ms — "curtain being pulled back"; (3) optional subtle 150ms ambient tail. Neutral dramatic tone — not celebratory. Volume: -6 dBFS, loudest UI sound in the lobby sequence.

**Status:** Needed

---

## ASSET-126 — Reserve Gain SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Format | OGG Vorbis, 96 kbps mono |
| Duration | 180–240ms |
| Naming | `audio_reserve_gain.ogg` |
| Atlas | N/A — `assets/audio/ui/` |
| Loop | No (one-shot) |
| Flags | F-CS-5. NP-5 (open): trigger point provisional. Implement with 30–50ms stagger for concurrent triggers. Short decay (<200ms ring-out) required. |

**Sonic Character:**
Two-layer: (1) cool crystalline chime in blue-register (~880–1200Hz high bell) representing the reserve diamond visual language; (2) brief low sub-bass pulse (40–60Hz, 80ms) for physical weight — "something valuable was deposited." Generic to all reserve-gain events (Xelorium, Miss Nuit, Gelure, Sablier, Mummy passive, prism reward). Volume: -10 dBFS.

**Status:** Needed

---

## ASSET-127 — Ready Retract SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Format | OGG Vorbis, 96 kbps mono |
| Duration | 120–180ms |
| Naming | `audio_class_ready_retract.ogg` |
| Atlas | N/A — `assets/audio/ui/` |
| Loop | No (one-shot) |
| Flags | F-CS-5. Coordinate with ASSET-020 (shop-phase ready retract) — same timbre family, different pitch/duration. |

**Sonic Character:**
Descending two-note counterpart to ASSET-124 — same pentatonic interval in reverse order, slightly muffled timbre with a soft paper-shuffle texture underneath communicating retraction motion. Not an error tone — this is a valid action. Volume: -12 to -10 dBFS (softer than confirm).

**Status:** Needed
