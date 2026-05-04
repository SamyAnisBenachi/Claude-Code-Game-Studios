# Asset Specs - System: Keyword System

> **Source**: design/gdd/keyword-system.md
> **Art Bible**: design/art/art-bible.md
> **Generated**: 2026-05-04
> **Status**: 12 assets specced / 0 approved / 0 in production / 0 done
> **Asset IDs**: ASSET-163 through ASSET-174

---

## Scope Notes

This spec covers keyword-only indicators and movement VFX that are not already owned by Combat Resolution. Combat-owned indicators such as STUN, SHIELD, INJURED, LEADER, SILENCE base outline, and OUTNUMBERED are tracked in `combat-resolution-assets.md`.

BODYGUARD, IRREMOVABLE, UNTARGETABLE, REPEL, ATTRACT, TELEPORT, START OF TURN, END OF TURN, and SILENCE stripped-keyword dissolve need production tracking because they are visible rule explanations, not decoration.

---

## Assets

| Asset ID | Name | Category | Dimensions / Format | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-163 | BODYGUARD Shield-Arc Glyph | UI / Unit Indicator | 6x8 PNG-32 | atlas_ui_hud | Needed |
| ASSET-164 | BODYGUARD Bond Break Shards | VFX / Particle | 4x4 PNG-32, 2-frame split | atlas_vfx | Needed |
| ASSET-165 | IRREMOVABLE Chain-Link Glyph | UI / Unit Indicator | 6x6 PNG-32 | atlas_ui_hud | Needed |
| ASSET-166 | IRREMOVABLE Block Flash | VFX / Overlay | 64x96 flat Void flash material | N/A | Needed |
| ASSET-167 | UNTARGETABLE Diamond-Cross Glyph | UI / Unit Indicator | 6x6 PNG-32 | atlas_ui_hud | Needed |
| ASSET-168 | REPEL Push Flash | VFX / Overlay | 64x64 PNG-32 or material | atlas_vfx | Needed |
| ASSET-169 | ATTRACT Pull Flash | VFX / Overlay | 64x64 PNG-32 or material | atlas_vfx | Needed |
| ASSET-170 | TELEPORT Exit Bar | VFX | 64x4 PNG-32 | atlas_vfx | Needed |
| ASSET-171 | TELEPORT Entry Bar | VFX | 64x4 PNG-32 | atlas_vfx | Needed |
| ASSET-172 | START OF TURN Floating Label Style | Runtime Text / Material | Ivory, 0.8x base, 600ms fade | N/A | Needed |
| ASSET-173 | END OF TURN Trigger Pulse Hook | VFX / Reuse | Reuses ASSET-145 gold pulse at 50-100% opacity | N/A | Needed |
| ASSET-174 | SILENCE Stripped-Keyword Dissolve | VFX / Particle / Material | 200ms glyph dissolve material | atlas_vfx + material | Needed |

### Visual Direction

- **BODYGUARD**: Prism White arc on the protector, plus procedural dotted connector to the protected unit. Connector dots are not separate sprites; ASSET-163 owns the source glyph.
- **IRREMOVABLE**: Void chain-link at bottom-center of base ring. On blocked displacement, full-sprite Void flash at 15% opacity, no movement.
- **UNTARGETABLE**: Ivory diamond outline with cross-stroke at top-right of base ring. Must read as broken targeting reticle, not SHIELD.
- **REPEL / ATTRACT**: REPEL uses warm orange impact flash and accelerates away. ATTRACT uses Arcane Gold pull flash and decelerates toward caster. Trails reuse unit sprite copy opacity and do not require unique art.
- **TELEPORT**: Prism White bars at exit and entry. No translate trail and no APPEARANCE pulse.
- **SILENCE dissolve**: all stripped keyword glyphs highlight briefly, then dissolve simultaneously over 200ms. This is a fairness/readability asset.

### Technical Notes

- Glyphs live on the unit indicator layer and must not conflict with the combat glyph position map.
- Procedural connectors and runtime text labels are production-tracked here even when no PNG is required.
- All movement VFX must complete in 480ms or less.
