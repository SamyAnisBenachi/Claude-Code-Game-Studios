# Asset Specs - Shared Fonts / Materials / Shaders

> **Source**: design/art/art-bible.md; design/accessibility-requirements.md; design/ux/interaction-patterns.md
> **Generated**: 2026-05-04
> **Status**: 12 assets specced / 0 approved / 0 in production / 0 done
> **Asset IDs**: ASSET-215 through ASSET-226

---

## Scope Notes

These are cross-system production assets used by HUD, Hand UI, Shop/Auction UI, Lobby, Board/Combat overlays, accessibility settings, and future result screens. They are intentionally not folded into a single owning screen spec.

ASSET-090 in `hud-assets.md` is retained as the original typography style anchor, but production font tracking is split here into Regular and Bold files so the font assets are owned once across the project.

---

## Assets

| Asset ID | Name | Category | Format / Dimensions | Delivery Path | Status |
|---|---|---|---|---|---|
| ASSET-215 | Project Display Font - Regular | Font | TTF preferred, OTF acceptable | `assets/fonts/font_display_regular.ttf` | Needed |
| ASSET-216 | Project Display Font - Bold | Font | TTF preferred, OTF acceptable | `assets/fonts/font_display_bold.ttf` | Needed |
| ASSET-217 | Keyboard Focus Ring Material | UI Material | 2px Prism White outline | Runtime material | Needed |
| ASSET-218 | Button Chrome Material Set | UI Material | primary / secondary / ghost / disabled | Runtime material + optional 9-slice | Needed |
| ASSET-219 | Shared Timer Bar Material Set | UI Material | calm / attention / urgent variants | Runtime material | Needed |
| ASSET-220 | Card Ghost / Lock Desaturation Shader | WGSL Shader | desaturation + opacity uniforms | `assets/shaders/card_ghost.wgsl` | Needed |
| ASSET-221 | Gold Selection Outline Shader | WGSL Shader | 1Hz gold outline pulse | `assets/shaders/gold_selection_outline.wgsl` | Needed |
| ASSET-222 | Unit Target Outline Material2D | WGSL Shader | Prism White outline, 2Hz pulse | `assets/shaders/unit_target_outline.wgsl` | Needed |
| ASSET-223 | Colorblind Palette Override Materials | Accessibility Material Set | protanopia / deuteranopia / tritanopia | Runtime palette table | Placeholder |
| ASSET-224 | Reduced-Motion Animation Policy Map | Accessibility Data Asset | JSON/RON policy map | `assets/config/reduced_motion.ron` | Placeholder |
| ASSET-225 | Audio Bus Settings UI Controls | UI / Settings | sliders/icons for music/SFX/UI buses | atlas_ui_hud | Placeholder |
| ASSET-226 | Brightness / Gamma Overlay Material | Accessibility Material | -50% to +50% brightness/gamma adjustment | Runtime material | Placeholder |

### Typography Direction

- Single sans-serif family, two weights: Regular and Bold.
- Tabular lining figures are required for gold, mana, timer, and auction price counters.
- Font must pass small-size tests for card keyword text at 14px and resource counters at 20px.
- Auction price counter requires at least 7:1 contrast on its background.

### Material / Shader Direction

- Focus rings are mandatory for every interactive UI element and must be visible on keyboard navigation.
- Button disabled state must be opacity/structure based, with text or context explaining the reason.
- Shared timer bar materials include the general amber/crimson ramp. The auction timer green/yellow/red exception is tracked separately in ASSET-182.
- Desaturation shaders must preserve silhouette readability and never erase card identity.

### Accessibility Notes

- Colorblind overrides are not a substitute for shape backups; they are an additional settings layer.
- Reduced motion must remove or cut non-essential UI motion while preserving board movement needed for readability.
- Audio bus settings require at least three independent volume sliders: music, SFX, UI audio.
