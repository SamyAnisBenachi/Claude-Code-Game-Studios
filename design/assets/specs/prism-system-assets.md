# Asset Specs - System: Prism System

> **Source**: design/gdd/prism-system.md
> **Art Bible**: design/art/art-bible.md
> **Generated**: 2026-05-04
> **Status**: 10 assets specced / 0 approved / 0 in production / 0 done
> **Asset IDs**: ASSET-185 through ASSET-194

---

## Scope Notes

Board Rendering already owns the board prism token and generic collection shimmer:

- ASSET-032 `env_prism_idle_32x32`
- ASSET-043 `vfx_prism_collect_shimmer`
- ASSET-049 `snd_prism_collect`

This file tracks Prism System rewards and player-facing feedback after collection. `prism_strike` and `prism_reserve` are static reward cards from the registry even if they are not yet present in the current `assets/data/cards.json`; their card art is illustration-only and uses the same runtime card composition pipeline as all other cards.

---

## Assets

| Asset ID | Name | Category | Dimensions / Format | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-185 | Card Illustration: prism_strike | Card Illustration | 240x360 PNG-32 zoom master, 120x180 display derivative | atlas_cards + on-demand zoom | Needed |
| ASSET-186 | Card Illustration: prism_reserve | Card Illustration | 240x360 PNG-32 zoom master, 120x180 display derivative | atlas_cards + on-demand zoom | Needed |
| ASSET-187 | Prism Reward Card-Acquired Shimmer | VFX / Reuse | Reuses ASSET-043 over hand-card arrival | atlas_vfx | Needed / Reuse |
| ASSET-188 | Prism Reward Dropped Indicator | UI / Toast | Runtime text + small prism icon | atlas_ui_hud | Needed |
| ASSET-189 | Prism Set Respawn Pulse | VFX | 64x64 PNG-32 or material, 5-token wave | atlas_vfx | Placeholder |
| ASSET-190 | Prism Strike Projectile | VFX / Projectile | 48x12 PNG-32, lane-to-objective | atlas_vfx | Needed |
| ASSET-191 | Prism Reserve Bar Ping | VFX / UI Overlay | Prism White reserve-diamond ping material | atlas_ui_hud | Needed |
| ASSET-192 | Prism Strike Reward Icon | UI Icon | 24x24 PNG-32 | atlas_ui_hud | Needed |
| ASSET-193 | Prism Reserve Reward Icon | UI Icon | 24x24 PNG-32 | atlas_ui_hud | Needed |
| ASSET-194 | Prism Random Draw Reward Icon | UI Icon | 24x24 PNG-32 | atlas_ui_hud | Needed |

### Illustration Direction

- **prism_strike**: Original spell-card illustration only. Show a cool Prism White projectile striking a distant objective pedestal. Do not bake mana cost, type, rarity, text, frame, target icon, hover state, or overlays.
- **prism_reserve**: Original spell-card illustration only. Show a reserve diamond receiving Prism White energy, with the "carries forward" loop-glyph concept reflected compositionally. Do not bake runtime card UI.

### UI / VFX Direction

- Reward dropped must be visible but not punitive. Use Ink Blue/Ivory text and a small Prism White icon, not red.
- Respawn pulse is placeholder until HUD/board timing is final. It should never imply an immediate second reward.
- Lane 1/5, Lane 2/4, and Lane 3 reward icons must be shape-distinct because lane rewards are strategic information.
