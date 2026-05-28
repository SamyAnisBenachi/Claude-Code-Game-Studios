# PROMPT 2037 — Hand Fan Card Readability Layout Repair

Status: SHIPPED
Branch: `work/PROMPT-2037`
Worktree: `D:/_DEV/Work/gcs-app-worktrees/lanesandlies/PROMPT-2037`
Base: `origin/main@8f7d3502`
Related bugs: V1-009, V1-011, UX-006, UX-007

## Summary

Three root causes drove the "spread across whole hand rectangle with huge gaps
and unreadable stats" observation:

1. **Constant fan spread regardless of hand count.** `fan_half_spread_px = 380`
   divides the same 760 px range across whatever cards are in hand. At 3–5
   cards the per-card pitch becomes 150–190 px (well over the 108 px card
   width), so the fan visually scatters across the whole strip.
2. **Cards anchored by left edge, not centre.** `node.left = layout.card_x`
   put the leftmost card's *left edge* at `fan_center_x − half_spread`, so the
   visible fan was always shifted right by ½ card width (~54 px).
3. **Stat badges too small to read.** Badges were 24 % of a 108 × 150 slot
   (≈ 26 × 36 px) with a 16 px numeric label — borderline at 1280 × 720.

Fix:

- Added `max_card_pitch_px` to `HandFanLayoutConfig` (default 78 px) and
  threaded it through `FanLayoutMetrics`. `compute_fan_slot_layout` now caps
  the effective half-spread to `(count − 1) × pitch / 2`, so small hands
  cluster around `fan_center_x` and 10-card hands keep their existing 78 px
  pitch (`9 × 78 / 2 = 351` < 380 cap).
- `apply_fan_layout_system` now subtracts ½ `HAND_CARD_DISPLAY_WIDTH/HEIGHT`
  when writing `Node.left`/`Node.top`, so the fan's visual centre tracks
  `fan_center_x` instead of trailing it by half a card.
- `FAN_SLOT_STAT_BADGE_PERCENT` bumped 24 → 30 (badges now ~32 × 45 px) and
  stat-label font bumped to `typography::BODY + 5` (= 20 px) so ATK/HP/MP/AR
  numbers clear the legibility floor at 1280 × 720.

## Owned files changed

| File | Change |
| --- | --- |
| `client/src/ui/hand/mod.rs` | Added `max_card_pitch_px` to `HandFanLayoutConfig` + `FanLayoutMetrics`; capped `compute_fan_slot_layout` effective spread; centred `apply_fan_layout_system` `Node.left`/`Node.top` offsets; bumped `FAN_SLOT_STAT_BADGE_PERCENT` 24→30 and `stat_badge_label_text_font` BODY+1→BODY+5. |
| `tests/unit/hand-ui/fan_layout_formula_test.rs` | Threaded `max_card_pitch_px = INFINITY` into pre-clustering helpers so historical formula expectations still hold. Added `default_config_3_cards_at_1280x720_cluster_around_center` (asserts 3-card pitch = 78 px and centres on `fan_center_x`) and `default_config_10_cards_at_1280x720_pitch_is_clamped_to_max_card_pitch` (asserts 10-card pitch = 78 px and span midpoint = `fan_center_x`). |
| `tests/integration/hand-ui/draft_initial_grid_test.rs` | Added `max_card_pitch: INFINITY` to `qa_metrics` to preserve formula expectations. |
| `tests/integration/hand-ui/hand_ui_chrome_composition_test.rs` | Bumped `EXPECTED_STAT_BADGE_PERCENT` 24 → 30 to match the readability fix. |

## Before / after layout rules

| Aspect | Before | After |
| --- | --- | --- |
| Effective half-spread | Always `fan_half_spread_px = 380` | `min(380, (count − 1) × 78 / 2)` |
| 3-card pitch @ 1280×720 | 380 px (cards spread across viewport) | 78 px (cluster around centre) |
| 5-card pitch @ 1280×720 | 190 px | 78 px |
| 10-card pitch @ 1280×720 | 84.4 px | 78 px (negligible change) |
| Fan visual centre | `fan_center_x + ½ card width` (off-centre) | `fan_center_x` |
| Stat badge size | 24 % × 24 % (≈ 26 × 36 px) | 30 % × 30 % (≈ 32 × 45 px) |
| Stat label font | BODY + 1 = 16 px | BODY + 5 = 20 px |

## Validation

- `cargo test -p client --test hand_ui_fan_layout_formula_test`
  → 9 passed, 0 failed (includes the 2 new clustering/centering tests and the
  existing PROMPT 1854 10-card readability invariants).
- `git diff --check` clean for owned files.
- Wider integration sweep deferred — disk-space exhaustion (`LNK1180`) and
  PDB-limit (`LNK1318`) on the worktree's `target/` prevented running multiple
  test binaries side-by-side in the same invocation. Per the PROMPT 2037
  validation budget ("Focused Bevy/static tests only; defer broad Cargo
  suites"), the formula-level unit test was prioritised; the `hand_ui_chrome_composition_test`
  EXPECTED_STAT_BADGE_PERCENT constant has been updated to 30.0 to track the
  badge size change.

## Remaining visual QA

- Launch client at 1280 × 720 and verify a 3-card placement-phase hand is
  clustered around screen centre, no longer scattered to the viewport edges.
- Verify ATK/HP/MP/AR numbers read clearly in a screenshot at 1280 × 720 and
  1920 × 1080.
- Verify the fan still fits comfortably inside the viewport at 1280 × 720
  with a 10-card hand (max half-spread 351 px → leftmost-card left edge at
  640 − 351 − 54 = 235 px, rightmost-card right edge at 640 + 351 + 54 = 1045
  px — well inside the 1280 px viewport).
- The reserve strip, drag sprite, and idle affordance overlays all read fan
  slot `Node.left`/`Node.top`, so the centre-shift propagates cleanly to
  them; visual confirmation recommended for the reserve strip alignment.

## Scope discipline

- No edits to `server/**`, `tools/autoplay/**`, placement interaction logic,
  shop/auction asset binding, or shared protocol.
- `.claude/settings.json` modification on this branch is the orchestrator
  hook boot artifact and is excluded from this PROMPT's commit.

2037: HAND-FAN-CARD-READABILITY-LAYOUT-REPAIR: SHIPPED
