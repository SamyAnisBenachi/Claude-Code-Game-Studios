# Asset Specs — System: HUD

> **Source**: design/gdd/hud.md
> **Art Bible**: design/art/art-bible.md
> **Generated**: 2026-05-01
> **Status**: 6 assets specced / 6 approved / 0 in production / 0 done

---

## 2026-05-04 Coverage Notes

- HUD remains the owner of HUD chips, reserve mana iconography, and HUD audio cues.
- Project font production tracking is split into shared ASSET-215 and ASSET-216 in `design/assets/specs/shared-fonts-materials-shaders-assets.md`. ASSET-090 remains the typography style anchor and selection gate.
- GAME_OVER result-screen assets are not designed here. Placeholder ownership rows are tracked in `design/assets/specs/game-session-system-assets.md` as ASSET-211 through ASSET-214.

---

## ASSET-088 — HUD Zone Chip Background

| Field | Value |
|-------|-------|
| Category | Sprite / 2D Art |
| Dimensions | 128×64 px *(see Conflict-1 note below)* |
| Format | PNG-32 (RGBA, straight alpha, no ICC profile, sRGB 8-bit) |
| Naming | `ui_hud_zone_chip_default_hud.png` |
| Atlas | `atlas_ui_hud` (1024×1024 max) · 2px gutter all edges → 132×68 px atlas footprint |
| 9-slice | Yes — horizontal only. Left cap: 20px fixed. Right cap: 20px fixed. Center body: 88px stretchable seamless tile strip. Slice margins: `left=20, right=20, top=0, bottom=0` |
| Memory | ~32 KB uncompressed (128×64×4 bytes) |

**Visual Description:**
A horizontal chip with straight-cut beveled ends that angle inward at roughly 45° — emphatically angular, not rounded, not circular. The interior fill is Ink Blue `#1A2D5A` at ~90% opacity, with a 1px inner bevel highlight along the top edge in lightened Ink Blue `#2E4A82` and a 1px shadow stroke along the bottom edge in Void `#0D0D14`. The outer perimeter carries a 2px Void `#0D0D14` outline baked into the sprite, with a single-pixel Prism White `#EEF4FF` catch-light at the top-left bevel corner only — enough to read as a solid game-piece without glow or photorealism. The center body strip is a clean seamless tile so the chip can stretch horizontally to accommodate any zone width.

**Art Bible Anchors:**
- §3.4 Shape Grammar: "flat angular chips — horizontal pill shapes with beveled ends. Distinct from world geometry." Beveled ends are non-negotiable — rounded corners would pull into Wakfu's organic register.
- §4.1 Palette: Ink Blue `#1A2D5A` is the designated UI background/depth color; Prism White `#EEF4FF` is the only permitted chrome highlight.
- §7.5 Iconography Style: 2px Void outline, flat interior fill — same rule as icons.
- §8.4 Outline Technique: baked at paint time, no procedural pass.

**Generation Prompt:**
```
Flat 2D game UI element, horizontal pill chip with angled/beveled ends (45-degree
cut, NOT rounded), dark navy blue fill (#1A2D5A), 2px near-black outline (#0D0D14),
single white catch-light pixel at top-left bevel corner, subtle 1px inner bevel
highlight on top edge (#2E4A82), cel-shaded style, Krosmaga/Ankama Wakfu game UI,
designed for 9-slice horizontal stretch, center body is a clean seamless tile strip,
isolated on transparent background

Style refs: Krosmaga HUD elements, Wakfu interface chrome
Negative: rounded pill, glowing edges, soft shadows, photorealistic metal,
brushstroke texture, Hearthstone golden frame, white background, drop shadow
```

**Engine Notes:**
Bevy 0.18 renders 9-slice via `ImageScaleMode::Sliced(TextureSlicer { ... })` on an `ImageNode`. Verify the exact `TextureSlicer` field names against Bevy 0.18 release notes — this API evolved post-cutoff (Bevy 0.14 training gap). `BorderRadius` in Bevy 0.18 is a field inside `Node`, not a standalone component.

> **Conflict-1 — Chip height:** Art Director estimated 128×48px; Technical Artist calculated 128×64px based on 2-line text content (6px top + 18px line 1 + ~14px line 2 at 0.65× + 6px bottom ≈ 44px minimum, making 48px tight). **Recommendation: use 128×64px.** Confirm with Art Director before commission.

**Status:** Needed

---

## ASSET-089 — Reserve Mana Diamond Icon

| Field | Value |
|-------|-------|
| Category | Sprite / 2D Art |
| Dimensions | 28×28 px (within art bible §7.5 range of 28–36px) |
| Format | PNG-32 (RGBA, straight alpha, no ICC profile, sRGB 8-bit) |
| Naming | `ui_hud_icon_reserve_diamond_default_hud.png` |
| Atlas | `atlas_ui_hud` · 2px gutter all sides → 32×32 px atlas cell |
| 9-slice | No |
| Memory | ~3 KB uncompressed; negligible in atlas |

**Visual Description:**
A standing diamond (square rotated 45°) with a two-stop vertical gradient interior: deep Ink Blue `#1A2D5A` at the lower vertex graduating to mid-blue `#2A5AAA` at the upper vertex, with a soft Prism White `#EEF4FF` inner glow concentrated in the upper third of the interior that falls off to zero at the lower vertex. The 2px Void `#0D0D14` outer outline is baked in, with a secondary 1px inner outline in lighter blue `#4A7ACC` sitting 2px inward from the Void boundary, creating a clean gem-like containment. On the lower face of the diamond, rendered at 65% opacity, a small clockwise circular arrow loop-glyph (~8px diameter at 28px render size) in Ivory `#F7F0DC` confirms the "carries forward" meaning visually. Heavy weight Ivory numeral centered on the diamond's visual centroid (slightly above geometric center to correct for the optical illusion of diamond shapes).

**Art Bible Anchors:**
- §3.4 Shape Grammar: "Reserve mana: Blue diamond shape (matching Krosmaga's AP RESERVE diamond) — a deliberate visual echo since the 298 cards assume this iconography." Shape is canonically specified and non-negotiable.
- §7.5 Iconography: "Reserve mana diamond | Blue gradient + soft Prism White inner glow | 28–36px."
- §7.6 Animation Feel: "Reserve mana diamond may pulse at ≤5% opacity delta maximum (only HUD element allowed idle animation)." Asset is designed so a ≤5% opacity shift reads as subtle breathing.
- §7.7 UX Constraint 2: "reserve mana must carry a loop/cycle glyph reinforcing 'carries forward.'" Loop glyph is a functional accessibility requirement, not decoration.
- §8.4 Outline Technique: 2px Void outline baked at paint time.

**Generation Prompt:**
```
Flat 2D game UI icon, standing diamond shape (square rotated 45 degrees),
dark-to-mid blue vertical gradient fill (bottom: #1A2D5A → top: #2A5AAA),
soft cool white inner glow in upper third (#EEF4FF, feathered),
secondary 1px inner outline 2px inset in lighter blue (#4A7ACC),
2px near-black outer outline (#0D0D14) baked in,
small clockwise circular arrow glyph on lower face at 65% opacity in warm white (#F7F0DC),
bold white numeral centered inside diamond,
Krosmaga AP crystal aesthetic, Ankama Wakfu mana gem style, cel-shaded flat illustration,
28px icon scale, isolated on transparent background

Style refs: Krosmaga AP diamond, Wakfu AP crystal, Dofus kama gem
Negative: circular shape, rounded rectangle, glowing aura, photorealistic gem,
specular sphere, 3D facets, Hearthstone mana crystal, Magic land frame
```

**Note:** If AD determines 32px or 36px reads better at target viewport, all even-integer sizes in the 28–36px range are WebGL2-compliant. Confirm before commission. Do not exceed 36px (art bible §7.5 maximum).

**Status:** Needed

---

## ASSET-090 — Project Display Font Style Anchor (split to ASSET-215/216)

| Field | Value |
|-------|-------|
| Category | Font Direction / Style Anchor |
| Dimensions | N/A (vector) |
| Format | TTF preferred (OTF acceptable). **Not** WOFF/WOFF2 — Bevy's asset server loads raw TTF/OTF, not browser font containers. |
| Naming | Production font files tracked as ASSET-215 and ASSET-216 |
| Delivery path | See `shared-fonts-materials-shaders-assets.md` |
| Memory | 80–200 KB per file on disk. Runtime glyph cache: <1 MB VRAM for HUD codepoints. |

**Typeface Character:**
Bold, slightly rounded display sans-serif in the Wakfu "clean cartoon stencil" register. Confident stroke weight, no serifs, corners softened 2–5% of cap-height — feels cut from painted wood, not typeset from digital geometry. **Tabular lining figures required** (gold and mana numerals animate via tween; proportional figures cause layout jitter). Condensed or semi-condensed width preferred (auction price at 3× base must not line-break within the panel). Open apertures and generous x-height for legibility at 11–12px (the floor of the HUD type scale at 1280×720). Two weights required: Bold (Heavy) for all resource numerals; Regular for phase labels, round counter, reserved suffix.

| Candidate | Verdict | Notes |
|-----------|---------|-------|
| Barlow Condensed Heavy | **First choice** | Condensed proportions, true two-weight family, tabular figures available. Corners land in the "softened but not rounded" zone. Ankama-adjacent at large display sizes. |
| Rajdhani Bold | **Second choice** | More geometric, harder corners. Proportional figures — verify tabular variant exists. Works at small sizes but less characterful at 3× auction scale. |
| Nunito Black | Reject | Too rounded — collapses into casual mobile register. |
| Bebas Neue | Reject | All-caps only, no Regular weight; phase labels require mixed case. |

**Selection gate:** Render `"11g (4r)"` at 1.0× Bold and `"PLACEMENT"` at 0.65× Regular in both candidates at 100%, 85%, and 65% opacity against Ink Blue `#1A2D5A`. Lock Barlow Condensed Heavy if it passes both; escalate to type review otherwise.

**Art Bible Anchors:**
- §7.4 Typography Direction: "Single display sans-serif, two weights (Heavy + Regular). No serif."
- HUD GDD Visual/Audio Requirements: "bold, slightly rounded sans-serif — the Wakfu 'clean cartoon stencil' register."

**Engine Flags:**
- `LineHeight` is a required component in Bevy 0.18 — auto-inserted; override explicitly via `LineHeight::RelativePx(...)` if line spacing needs adjustment.
- Variable font TTF (single file with `wght` axis): support in Bevy 0.18 **unverified post-cutoff** — test on WASM before committing; fall back to two static files if loading fails.
- Child `TextSpan` entities for the `(Yr)` parenthetical must reference the same or a sibling font handle and set `TextFont.font_size` independently.

**Status:** Needed

---

## ASSET-091 — Phase Transition Tick SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Format | OGG Vorbis — WASM delivery. WAV is source/master only, not shipped. |
| Naming | `snd_hud_phase_tick.ogg` |
| Delivery path | `assets/audio/sfx/snd_hud_phase_tick.ogg` |
| Duration | 80–150ms. Single hit, no reverb tail, no loop point. |
| Sample rate | 44 100 Hz |
| Channels | Mono |
| Source bit depth | 16-bit PCM |
| OGG quality | Q4–Q6 |
| Target file size | < 10 KB |

**Sonic Character:**
A single unpitched transient with <5ms attack — listener perceives it as starting at full amplitude. Timbral target: struck hardwood block. Fundamental centered 600–900 Hz, sharp click transient with a brief 2–4 kHz spike lasting under 20ms, followed by a short woody resonance that decays cleanly without ring. Think: placing a Go stone on a wooden board. Must be non-intrusive — audible at 60% master volume without being consciously noticed. No content below 200 Hz (those frequencies draw attention through ambient noise and must not compete with the auction panel or hand fan).

**Authoring notes:** Real foley preferred — 1/4-inch hardwood dowel struck against dense hardwood (maple on maple) in a dead room. If synthesized: 30ms gated white noise filtered 600–900 Hz bandpass, plus 2–4 kHz transient click layer at −6 dB relative. No reverb. No loop points.

**Fires on:** DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, PLACEMENT transitions.
**Silent on:** RESOLUTION and GAME_OVER (code-side gate — flag to programmer implementing `handle_phase_changed_system`).

> **Conflict-2 — Normalization:** Art Director specifies −12 dBFS peak delivery (so tick sits −15 dB below board combat SFX at 100% master). Technical Artist specifies −3 dBFS source headroom for mix. **Recommended resolution:** Deliver source master at −3 dBFS; audio director sets mix-level in game audio config to achieve the AD's −15 dB relative balance. Flag to Audio Director before final encode.

**Art Bible Anchors:**
- HUD GDD Audio Events: "Single dry medium-pitched wood-block tick — one hit, no reverb tail. Marks the moment without announcing it."
- §2 Mood — DRAFT_AUCTION: "Electric, escalating, exposed." The tick must not undercut the auction panel's tension.

**Status:** Needed

---

## ASSET-092 — Scoreboard Dot Darkening Thud SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Format | OGG Vorbis — WASM delivery. WAV is source/master only, not shipped. |
| Naming | `snd_hud_dot_thud.ogg` |
| Delivery path | `assets/audio/sfx/snd_hud_dot_thud.ogg` |
| Duration | 80–200ms. Short low stone-thud, no reverb tail, no loop point. |
| Sample rate | 44 100 Hz |
| Channels | Mono |
| Source bit depth | 16-bit PCM |
| OGG quality | Q4–Q6 |
| Target file size | < 10 KB |

**Sonic Character:**
A short, low-frequency impact with 10–20ms attack — producing a sense of mass and permanence, something heavy was placed, not struck. Fundamental 80–180 Hz with ~100–150ms exponential decay, no ring, no pitch wobble. Dense stone on stone: a heavy object placed firmly, not dropped. Must feel final and irreversible without being emotionally loaded. Not a crack, not a shatter — those registers belong to Board Rendering's objective destruction audio (ASSET-047/048). The dot thud arrives after the board animation; it is the ledger entry, not the event. Spectrally distinct from the phase tick (lower fundamental, slower attack) so the player's ear can distinguish them without visual confirmation.

**Authoring notes:** Real foley preferred — dense stone or concrete block gently but firmly placed on stone/concrete in a dead room ("firm placement" gesture, not a drop). If synthesized: 120 Hz sine wave burst (15ms attack, 130ms exponential decay) layered with 40ms low-passed noise burst (high-cut at 300 Hz) at −8 dB relative. No reverb.

**Fires on:** `HudObjectiveUpdate` receipt.
**Silent during:** RESOLUTION (code-side gate — flag to programmer implementing `handle_hud_objective_update` Observer).

**Same normalization conflict as ASSET-091 applies — see resolution above.**

**Art Bible Anchors:**
- HUD GDD Audio Events: "Short low stone-thud. Confirms permanent removal. Not mournful, not alarming. Silent during RESOLUTION."
- §2 Mood — RESOLUTION: "Board is the hero." Dot thud is explicitly subordinate to board audio during RESOLUTION.

**Status:** Needed

---

## ASSET-093 — GAME_OVER Resolved Chord SFX

| Field | Value |
|-------|-------|
| Category | Audio |
| Format | OGG Vorbis — WASM delivery. WAV is source/master only, not shipped. |
| Naming | `snd_hud_game_over_chord.ogg` |
| Delivery path | `assets/audio/sfx/snd_hud_game_over_chord.ogg` |
| Duration | 1 500–3 000ms. Settling chord with natural decay to silence. No loop point. |
| Sample rate | 44 100 Hz |
| Channels | Stereo (mono acceptable if audio budget is a constraint) |
| Source bit depth | 16-bit PCM (24-bit source acceptable, downsampled for OGG delivery) |
| OGG quality | Q5–Q7 |
| Target file size | < 40 KB |

**Sonic Character:**
A two-to-three note chord in low-to-mid register (root C3–E3, ~130–165 Hz), played simultaneously with ~80ms attack, ~200ms peak hold, then exponential decay to silence over 1.5–2 seconds. **Must resolve to tonic.** Recommended voicing: bare perfect fifth (C3 + G3) — resolves without committing to major or minor, reading as "settled" rather than "won" or "lost." If a third is added, use minor third (not major — major third reads as victory sting). Instrument: pizzicato cello/bass, plucked koto, or hammered dulcimer — organic, weighted, not synthetic pad or piano. No reverb tail beyond 0.5 seconds of natural room decay. Must end in silence before outcome screen audio begins.

**Authoring notes:** Pitch target — C3 + G3 (130 Hz + 196 Hz), optionally add Eb3 for minor flavor; do not add E natural (major third). Maximum peak −6 dBFS — louder than ASSET-091/092; the only HUD audio event that actively asserts the moment rather than confirming it. Stereo OGG delivery; coordinate crossfade timing with outcome-screen audio designer.

**Fires on:** `S2CPhaseChanged(GAME_OVER)` — once, no loop. One-shot playback (despawns on completion). Verify `PlaybackSettings::ONCE` or equivalent Bevy 0.18 API name against release notes (post-cutoff flag).

**Art Bible Anchors:**
- §2 Mood — GAME_OVER: "Resolute finality — Conclusive, Unambiguous, Open, Earned." Chord ambivalence (no major/minor commitment) serves this mood precisely.
- HUD GDD Audio Events: "Single resolved chord — two or three notes settling to tonic; no fanfare, no sting. Confirms finality without editorialising win or loss."

**Status:** Needed
