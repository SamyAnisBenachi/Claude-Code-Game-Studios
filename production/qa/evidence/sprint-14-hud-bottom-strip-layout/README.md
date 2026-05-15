# Sprint 14 HUD Bottom Strip Layout Evidence

Story: `production/epics/hud/story-016-hud-bottom-strip-layout.md`

Worker branch: `work/s14-hud-bottom-strip`

Source base: `origin/main@72438318f5065c70003ab5ae6fa586092a46524b`

Implementation commit: `<pending worker commit>`

## Acceptance Evidence

- `HudBottomStrip` is a structural child of `HudRoot`.
- `HudBottomStrip` consumes `strips::FooterBar`, `strips::footer_bar_node()`,
  and `z_layers::UI_BASE`.
- Own-player `HudFigurine` is reparented under `HudBottomStrip` and composes as
  a flex child rather than using root-level absolute offsets.
- `HudTopStrip` remains the owner of current mana and reserve mana.
- `HUD_ENTITY_COUNT` is unchanged because the bottom strip is structural and is
  not marked with `HudEntity`.
- HUD schedule source remains unchanged.
- Scoreboard dots remain outside `HudBottomStrip`; no objective identity is
  claimed by the bottom-strip story.

## Automated Checks

- `hud_bottom_strip_layout_test` covers AC1, AC2, AC3, AC4, AC5, AC10, AC11,
  and AC12 with world-based Bevy ECS assertions plus source grep guards.
- Targeted adjacent coverage is expected from `hud_top_strip_layout_test`.

## Visual Capture Slot

Runtime screenshot capture is deferred in this worker environment. Expected
capture filenames for a later visual pass:

- `hud-bottom-strip-draft.png`
- `hud-bottom-strip-auction.png`

The implementation keeps layout assertions in code so the visual pass can focus
on overlap, clipping, and final presentation checks.
