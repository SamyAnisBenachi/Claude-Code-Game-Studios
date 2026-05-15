# Sprint 14 - S11-UX-HUD-TOP-STRIP-LAYOUT Evidence

> Story: `production/epics/hud/story-015-hud-top-strip-layout.md`
> Prompt: 940 (`/dev-story` implementation worker)
> Worktree: `D:/_DEV/wt/ccgs-prompt-940-hud-top-strip`
> Branch: `work/s14-hud-top-strip-layout-940`
> Source-of-truth at start: `origin/main@e3ed056`

## No-Claim Restatement

This evidence records HUD top-strip layout composition work only. It does
not claim public release readiness, release-candidate readiness, full game
completion, full playable-client manual QA, final-art completion, Sprint 14
close-out, stage advance, or a Polish->Release retry.

Carried non-claims preserved verbatim:

- `QA-COND-0005`
- `QA-COND-0006`
- `PAW-TD-004-a`
- `S8-QA-001-W1`
- PROMPT 761 Polish->Release FAIL
- Sprint 12 story 019 underlying drag-runtime bug not claimed fixed

No Standard-tier accessibility claim is made. No final-art or asset
replacement is claimed. The HUD remains read-only over server-authoritative
state per ADR-002 and ADR-021.

## Cross-Links

| Source | Relevance |
|--------|-----------|
| `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md` section 3.2 H1 / H2 / H5 / H8 | Original HUD top-strip absolute-positioning, magic-offset, typography, and missing-flex-parent findings. |
| `docs/ux/ui-clean-pass-roadmap.md` rank 7 | Sprint 14 Tier 1 HUD top-strip row. |
| `docs/ux/global-ui-design-spec.md` sections 3, 4, 5, 9 | Z-layer, spacing scale, typography scale, and HeaderBar strip composition inputs. |
| `client/src/ui/design_tokens/z_layers.rs` | `UI_BASE = GlobalZIndex(300)` citation. |
| `client/src/ui/design_tokens/strips.rs` | `HeaderBar` primitive and 60 px deterministic header height. |

## Automated Evidence

| AC | Evidence | Status |
|----|----------|--------|
| AC1 - single flex parent | `hud_top_strip_layout_test::ac1_spawns_single_header_bar_top_strip_with_readouts_under_it` verifies `HudTopStrip` carries `strips::HeaderBar`, `Display::Flex`, `ChildOf(HudRoot)`, and owns the top-strip readouts. | PASS |
| AC2 - no top-strip absolute offsets | `hud_top_strip_layout_test::ac2_top_strip_children_do_not_use_absolute_offset_nodes` verifies direct top-strip child `Node.position_type != Absolute` and source-greps legacy top-strip offset helpers. | PASS |
| AC3 - `HudEntities` preservation | `hud_top_strip_layout_test::ac3_hud_entities_preserve_existing_fields_and_add_top_strip` verifies the new `top_strip` field plus all existing logical entity fields and `HUD_ENTITY_COUNT` preservation. | PASS |
| AC4 - ADR-021 schedule stability | `hud_top_strip_layout_test::ac4_hud_plugin_schedule_source_remains_unchanged` verifies the existing `HudSystemSet` chain and `OnEnter` / `OnExit` wiring remain unchanged. | PASS |
| AC9 - fixed HUD font sizing | `hud_top_strip_layout_test::ac9_top_strip_font_size_lines_do_not_use_viewport_scaled_values` verifies no `Val::Percent`, `Val::Vw`, or `Val::Vh` appears on `TextFont` / `font_size` lines in `client/src/ui/hud/mod.rs`. | PASS |
| AC10 - z-layer slot use | `hud_top_strip_layout_test::ac10_root_and_top_strip_consume_ui_base_z_layer` verifies both `HudRoot` and `HudTopStrip` consume `z_layers::UI_BASE`. | PASS |

## Longest-Content Table

Visual capture is deferred to `manual-capture-instructions.md`; no rendered
PNG evidence is claimed by this worker. The table below records the
longest-content cases the manual capturer must observe.

| Element | Longest expected content | Node / font intent | Capture status |
|---------|--------------------------|--------------------|----------------|
| Phase label | `DraftAuction` | `HUD_SECONDARY_FONT_SIZE_PX = typography::H2` | Manual capture pending |
| Round counter | `Round 6 / 6` story stress case; runtime currently renders compact `R6` | `HUD_SECONDARY_FONT_SIZE_PX = typography::H2` | Manual capture pending |
| Own gold | `99g (99r)` two-digit reserved-gold stress case | `HUD_GOLD_FONT_SIZE_PX = typography::DISPLAY`, reserved span `typography::H1` | Manual capture pending |
| Opponent gold | `99g (99r)` two-digit reserved-gold stress case | Same as own gold | Manual capture pending |
| Mana label | `99 / 99` double-digit mana stress case | 104 x 28 px bar, `typography::H2` | Manual capture pending |
| Reserve mana | `99` double-digit reserve stress case | 74 x 74 px diamond, child label 104 x 24 px, `typography::H2` | Manual capture pending |
| Timer bar | Full phase timer fill | 200 x 8 px fixed pixel bar | Manual capture pending |

## Dimension Table

The automated test asserts Node intent where Bevy exposes fixed pixel
dimensions without a renderer. Rendered text widths require the deferred
browser/WASM screenshots.

| Element | 1920x1080 intended dimensions | 1366x768 intended dimensions | Status |
|---------|-------------------------------|------------------------------|--------|
| `HudTopStrip` / `HeaderBar` | 1920 x 60 px (`width: 100%`, `height: 60`) | 1366 x 60 px (`width: 100%`, `height: 60`) | PASS by Node intent |
| Phase label | Text-measured width, fixed H2 font, token padding | Text-measured width, fixed H2 font, token padding | Manual capture pending |
| Round counter | Text-measured width, fixed H2 font, token padding | Text-measured width, fixed H2 font, token padding | Manual capture pending |
| Own / opponent gold | Text-measured width, fixed Display + H1 span fonts | Text-measured width, fixed Display + H1 span fonts | Manual capture pending |
| Mana label | 104 x 28 px | 104 x 28 px | PASS by Node intent |
| Reserve container | 74 x 74 px | 74 x 74 px | PASS by Node intent |
| Reserve label | 104 x 24 px child label | 104 x 24 px child label | PASS by Node intent |
| Timer bar | 200 x 8 px at full timer ratio | 200 x 8 px at full timer ratio | PASS by Node intent |

## Overlap Audit

| Check | Evidence | Status |
|-------|----------|--------|
| Top-strip children are flex-composed under one parent | Direct children are under `HudTopStrip`, and each direct child has non-absolute positioning. | PASS |
| HeaderBar does not rely on spawn order for paint slot | `HudTopStrip` carries `z_layers::UI_BASE`, same as `HudRoot`. | PASS |
| Text clipping / sibling overlap in renderer | Requires browser/WASM captures at 1920x1080 and 1366x768. | Manual capture pending |
| Timer bar overlap in renderer | Requires browser/WASM captures during a timed non-hidden phase. | Manual capture pending |
| Reserve mana overlap in renderer | Requires `DraftAuction` / economy auction capture with `reserve_mana > 0`. | Manual capture pending |

## Screenshot Capture Status

No screenshots are attached by this worker. This mirrors the PROMPT 928
headless-environment precedent: the implementation worker has no interactive
browser/WASM rendering capability, so the capture step is deferred and
documented in `manual-capture-instructions.md`.

Expected filenames after manual capture:

- `top-strip-1920x1080-draft-auction.png`
- `top-strip-1366x768-draft-auction.png`

The captures must not be treated as present until those PNGs are committed
or attached by a later capture worker.

## Z-Layer Citation

`client/src/ui/design_tokens/z_layers.rs` defines:

```text
UI_BASE = GlobalZIndex(300)
```

`HudRoot` and `HudTopStrip` both spawn with `z_layers::UI_BASE`. The focused
integration test asserts this directly by reading `GlobalZIndex` components
from the spawned ECS world.
