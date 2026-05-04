# Asset Specs - System: Combat Resolution

> **Source**: design/gdd/combat-resolution.md
> **Art Bible**: design/art/art-bible.md
> **Generated**: 2026-05-04
> **Status**: 35 assets specced / 0 approved / 0 in production / 0 done
> **Asset IDs**: ASSET-128 through ASSET-162

---

## Scope Notes

Combat Resolution owns transient combat readability assets: reveal flips, hit timing flashes, damage-number styles, state indicators that fire during RESOLUTION, and combat-specific audio cues. It does not duplicate Board Rendering objective destruction audio or prism-token board sprites:

- Reuse ASSET-047 for real objective destruction audio.
- Reuse ASSET-048 for fake objective destruction audio.
- Reuse ASSET-050 for heavy objective attack/damage impact where the objective itself is hit.
- Reuse ASSET-043 for the generic prism collection shimmer.

All visual assets follow the GDD rule: no blur/glow/bloom on unit sprites. Impact flashes are flat one-frame color fills. Objective destruction uses overlay frames rather than a GPU post-process pass.

---

## P0 Assets

| Asset ID | Name | Category | Dimensions / Format | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-128 | Placement Reveal Card-Back Silhouette | Sprite / Overlay | 64x96 PNG-32 | atlas_vfx | Needed |
| ASSET-129 | Placement Reveal Prism Edge Flash | VFX | 64x96 PNG-32, 3-frame strip | atlas_vfx | Needed |
| ASSET-130 | Player-Side Base Ring Reveal Flash | VFX | 48x16 PNG-32, Player A/B variants | atlas_ui_hud | Needed |
| ASSET-131 | CHARGE X Motion Trail Copy Material | Material / Runtime Tint | Runtime sprite material, 40% opacity | N/A | Needed |
| ASSET-132 | FIRST STRIKE Impact Flash | VFX | 64x64 PNG-32 | atlas_vfx | Needed |
| ASSET-133 | Standard Combat Impact Flash | VFX | 64x64 PNG-32 | atlas_vfx | Needed |
| ASSET-134 | Damage Number Text Style | Runtime Text / Material | Heavy font, 24px min, Crimson Slate | N/A | Needed |
| ASSET-135 | SHIELD Active Hex Glyph | UI / Unit Indicator | 8x8 PNG-32 | atlas_ui_hud | Needed |
| ASSET-136 | SHIELD Absorb Burst Particles | VFX / Particle | 4x4 PNG-32 particle, 3-particle burst | atlas_vfx | Needed |
| ASSET-137 | STUN Orbit Star Glyph | UI / Unit Indicator | 6x6 PNG-32, 3 orbiting instances | atlas_ui_hud | Needed |
| ASSET-138 | INJURED Outline Pulse Material | Shader / Material | Void to rust-brown outline pulse | N/A | Needed |
| ASSET-139 | LEADER Crown Glyph | UI / Unit Indicator | 8x8 PNG-32 | atlas_ui_hud | Needed |
| ASSET-140 | LEADER Family Buff Ring Tint | Material / Runtime Tint | Arcane Gold 20% opacity base-ring tint | N/A | Needed |
| ASSET-141 | SILENCE Desaturation Outline Material | Shader / Material | Outline desaturates Void to grey | N/A | Needed |
| ASSET-142 | OUTNUMBERED Arrow-Down Glyph | UI / Unit Indicator | 8x8 PNG-32 | atlas_ui_hud | Needed |
| ASSET-143 | Death Squash / Crimson Tint Material | Shader / Material | 50% vertical squash, Crimson tint | N/A | Needed |
| ASSET-144 | Death Crimson Particle Burst | VFX / Particle | 4x4 PNG-32 particle, 3-4 particles | atlas_vfx | Needed |
| ASSET-145 | Trigger Gold Pulse Ring | VFX | 64x64 PNG-32 | atlas_vfx | Needed |
| ASSET-146 | Ranged Bolt Projectile - Blade | VFX / Projectile | 32x8 PNG-32 | atlas_vfx | Needed |
| ASSET-147 | Ranged Bolt Projectile - Arcane | VFX / Projectile | 32x8 PNG-32 | atlas_vfx | Needed |
| ASSET-148 | Ranged Bolt Projectile - Neutral | VFX / Projectile | 32x8 PNG-32 | atlas_vfx | Needed |
| ASSET-149 | Objective HP Pip Damage Flash | Material / UI Overlay | Crimson Slate pip flash material | N/A | Needed |
| ASSET-150 | Objective Destruction Prism Overlay Frames | VFX / Screen Overlay | 3 frames, full-screen UI overlay | atlas_vfx or UI layer | Needed |
| ASSET-151 | Real Objective Lane Gold Flood | VFX / Lane Overlay | 512x128 PNG-32 lane overlay | Standalone or atlas_vfx | Needed |
| ASSET-152 | Fake Objective Question Dissolve | VFX / Glyph Animation | 64x96 PNG-32 overlay | atlas_vfx | Needed |
| ASSET-153 | Kill Gold +1 Float Text Style | Runtime Text / Material | Arcane Gold, 1.5x base, +20px float | N/A | Needed |
| ASSET-154 | Objective Gold +3 Float Text Style | Runtime Text / Material | Arcane Gold, 2x base, + HUD bloom hook | N/A | Needed |

### Visual Direction

- **Placement reveal**: back-of-card silhouette to Prism White edge squash to front sprite, total 80-100ms. All lanes reveal simultaneously.
- **Impact timing**: FIRST STRIKE is Prism White; standard hit is warm orange. These colors must not drift because timing read depends on them.
- **Damage numbers**: Crimson Slate, Heavy weight, world-space, never overlapping. SHIELD absorption intentionally shows no damage number.
- **Persistent indicators**: attach to unit entities and survive movement. They animate only on state change.
- **Objective destruction**: full-screen Prism White overlay frames simulate bloom without post-processing. Real objective adds lane gold flood; fake objective dissolves the question mark with no gold fill.

### Technical Notes

- World-space combat VFX belong in `atlas_vfx` unless they are pure runtime materials.
- Unit primary outlines remain baked into unit art; ASSET-138 and ASSET-141 are secondary outline effects only.
- Damage and gold floats are runtime text entities using the shared display font from `shared-fonts-materials-shaders-assets.md`.

---

## Audio Assets

| Asset ID | Name | Category | Format | Naming | Status |
|---|---|---|---|---|---|
| ASSET-155 | Placement Reveal Flip SFX | Audio | OGG Vorbis / WAV master | `sfx_combat_reveal_flip.ogg` | Needed |
| ASSET-156 | FIRST STRIKE Impact SFX | Audio | OGG Vorbis / WAV master | `sfx_combat_first_strike_hit.ogg` | Needed |
| ASSET-157 | Standard Combat Impact SFX | Audio | OGG Vorbis / WAV master | `sfx_combat_standard_hit.ogg` | Needed |
| ASSET-158 | Unit Death SFX | Audio | OGG Vorbis / WAV master | `sfx_combat_unit_death.ogg` | Needed |
| ASSET-159 | SHIELD Absorb SFX | Audio | OGG Vorbis / WAV master | `sfx_combat_shield_absorb.ogg` | Needed |
| ASSET-160 | SHIELD Break SFX | Audio | OGG Vorbis / WAV master | `sfx_combat_shield_break.ogg` | Needed |
| ASSET-161 | Kill Gold Reward SFX | Audio | OGG Vorbis / WAV master | `sfx_combat_kill_gold_reward.ogg` | Needed |
| ASSET-162 | COUNTERATTACK Response SFX | Audio | OGG Vorbis / WAV master | `sfx_combat_counterattack.ogg` | Needed |

### Sonic Direction

- FIRST STRIKE must be brighter and sharper than standard combat impact so timing advantage is audible.
- Standard impact is warm, physical, and mid-register.
- Unit death is short and final, not cinematic.
- SHIELD absorb and SHIELD break are distinct: absorb is blocked force, break is the consumed state.
- COUNTERATTACK is a reactive snap after incoming damage, not a second generic hit.
