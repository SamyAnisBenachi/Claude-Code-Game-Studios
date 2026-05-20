# PROMPT 1511 — REPAIR-HU-CHROME-02-HAND-FAN-ALL-DIRECT-CHILDREN

Status: **NO-OP — canary already green on origin/main `5d46b9a9` (PROMPT 1508).**

## TL;DR

The failing-state premise from PROMPT 1510 does not reproduce on the declared
source-of-truth. On a clean worktree based at
`origin/main = 5d46b9a92bb1cdf894b62b32a53175cd0861224b` (PROMPT-1508 repair
HU-CHROME-02 hand-fan art-image width), the canary
`hand_ui_chrome_composition_test::fan_slot_chrome_children_have_absolute_layout_after_placement_entry`
passes, and so does the full `hand_ui_chrome_composition_test` integration suite
and the `hand_ui_fan_layout_formula_test` unit suite. No code edit was needed.

PROMPT 1510 likely ran against a worktree that had not yet absorbed PROMPT
1508 (`5d46b9a9`), or had reverted the `art_node` width override at
`client/src/ui/hand/mod.rs:3970-3975`.

## Worktree / branch

- Worktree: `D:/tmp/wt-1511`
- Branch: `repair/hu-chrome-02-hand-fan-1511`
- Base commit: `5d46b9a92bb1cdf894b62b32a53175cd0861224b` (origin/main, PROMPT 1508)
- `git status --short` → clean
- `git diff --check` → clean (no edits required)

## Reproduction attempt (the test is GREEN)

```
CARGO_TARGET_DIR=D:/tmp/cargo-target-1511 CARGO_PROFILE_TEST_DEBUG=0 \
CARGO_INCREMENTAL=0 \
cargo test --test hand_ui_chrome_composition_test \
  fan_slot_chrome_children_have_absolute_layout_after_placement_entry \
  -- --nocapture
```

Result:

```
running 1 test
test fan_slot_chrome_children_have_absolute_layout_after_placement_entry ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Full file:

```
cargo test --test hand_ui_chrome_composition_test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Companion suite (regression budget):

```
cargo test --test hand_ui_fan_layout_formula_test
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Direct-child inventory under each fan slot (HU-CHROME-02/03 contract)

Twelve direct `ChildOf(slot)` entities are spawned in `spawn_hand_ui` at
`client/src/ui/hand/mod.rs`. Every one declares `PositionType::Absolute` and a
positive `Val::Percent` width — none uses `Val::Auto` or `Val::Px(0.0)`. The
intrinsic slot box is `HAND_CARD_DISPLAY_WIDTH_PX × HAND_CARD_DISPLAY_HEIGHT_PX`
once the slot is promoted to `FanSlotState::Active` by
`apply_fan_layout_system`, so the percents resolve against a real containing
block.

| # | Marker / Name | Spawn site | `position_type` | `width` | `height` | `left/right/top/bottom` | Source builder |
|---|---|---|---|---|---|---|---|
| 1 | `CardSlotArtImage` (`Fan Slot N Card Art`) | `mod.rs:3977-3987` | `Absolute` | `Val::Percent(100.0)` | `Val::Percent(100.0)` | `Percent(0)` / `Auto` / `Percent(0)` / `Auto` | `card_slot_art_image_node(CardSlotKind::HandFan)` + PROMPT 1508 override `mod.rs:3970-3975` |
| 2 | `HandCardFrame` | `mod.rs:3996-4003` | `Absolute` | `Val::Percent(100.0)` | `Val::Percent(100.0)` | `0% / Auto / 0% / Auto` | `fan_slot_card_frame_node` `mod.rs:4663-4672` |
| 3 | `StatBadgeAtk` (BottomLeft) | `mod.rs:4004-4013` | `Absolute` | `Val::Percent(24.0)` | `Val::Percent(24.0)` | `0% / Auto / Auto / 0%` | `fan_slot_stat_badge_node(BottomLeft)` `mod.rs:4674-4693` |
| 4 | `StatBadgeHp` (BottomRight) | `mod.rs:4025-4034` | `Absolute` | `Val::Percent(24.0)` | `Val::Percent(24.0)` | `Auto / 0% / Auto / 0%` | `fan_slot_stat_badge_node(BottomRight)` |
| 5 | `StatBadgeMp` (TopLeft) | `mod.rs:4046-4055` | `Absolute` | `Val::Percent(24.0)` | `Val::Percent(24.0)` | `0% / Auto / 0% / Auto` | `fan_slot_stat_badge_node(TopLeft)` |
| 6 | `StatBadgeAr` (TopRight) | `mod.rs:4067-4076` | `Absolute` | `Val::Percent(24.0)` | `Val::Percent(24.0)` | `Auto / 0% / 0% / Auto` | `fan_slot_stat_badge_node(TopRight)` |
| 7 | `HandRarityIcon` (TopCenter) | `mod.rs:4088-4095` | `Absolute` | `Val::Percent(15.0)` | `Val::Percent(15.0)` | `42.5% / — / 0% / Auto` | `fan_slot_icon_node(TopCenter)` `mod.rs:4726-4740` |
| 8 | `HandTypeIcon` (BottomCenter) | `mod.rs:4096-4103` | `Absolute` | `Val::Percent(15.0)` | `Val::Percent(15.0)` | `42.5% / — / Auto / 0%` | `fan_slot_icon_node(BottomCenter)` |
| 9 | `FanSlotDimOverlay` (`DragStateOverlay`) | `drag_state_visuals.rs:166-175` (`spawn_fan_slot_drag_state_overlays`) | `Absolute` | `Val::Percent(100.0)` | `Val::Percent(100.0)` | `0% / — / 0% / —` | `dim_overlay_node` `drag_state_visuals.rs:121-130` |
| 10 | `FanSlotHoverOverlay` (`DragStateOverlay`) | `drag_state_visuals.rs:177-186` | `Absolute` | `Val::Percent(100.0)` | `Val::Percent(100.0)` | `0% / — / 0% / —` + `border 2px` | `hover_overlay_node` `drag_state_visuals.rs:134-144` |
| 11 | `FanSlotPlayableAffordanceOverlay` | `mod.rs:4472-4484` | `Absolute` | `Val::Percent(100.0)` | `Val::Percent(100.0)` | `0% / — / 0% / —` + `border 3px` | `playable_affordance_overlay_node` `mod.rs:4432-4442` |
| 12 | `FanSlotPlayableAffordanceUnaffordableOverlay` | `mod.rs:4486-4498` | `Absolute` | `Val::Percent(100.0)` | `Val::Percent(100.0)` | `0% / — / 0% / —` + `border 1px` | `unaffordable_affordance_overlay_node` `mod.rs:4444-4454` |

No other code path spawns a `ChildOf(slot)` direct child of a fan slot. The
stat-badge text labels (`StatBadgeAtkLabel` / `StatBadgeHpLabel` /
`StatBadgeMpLabel` / `StatBadgeArLabel`) are grandchildren (children of the
badges, not the slot), so they do not participate in this canary. The
`children_of(slot)` helper used by the test confirms this: it only counts
entities whose `ChildOf.parent()` equals the slot.

## Why PROMPT 1510 still failed

PROMPT 1510 ran on a worktree where the PROMPT 1508 override at
`client/src/ui/hand/mod.rs:3970-3975` was either absent or had been clobbered
back to the shared `card_slot_art_image_node(CardSlotKind::HandFan)` defaults,
which set `width = Val::Auto` (sized by `left`/`right` Px insets — fine for
slots with a label-strip child, but a `Val::Auto` violation per HU-CHROME-02).
On origin/main `5d46b9a9` the override is present, the art child's width is
`Val::Percent(100.0)`, and the assertion does not fire.

## Offending child identity — before vs after PROMPT 1508

| | Before PROMPT 1508 | After PROMPT 1508 (origin/main `5d46b9a9`) |
|---|---|---|
| Marker | `CardSlotArtImage` (`Fan Slot N Card Art`) | `CardSlotArtImage` (`Fan Slot N Card Art`) |
| `position_type` | `Absolute` | `Absolute` |
| `width` | `Val::Auto` (sized by `left`/`right` Px insets from `card_slot_art_image_node`) — **violates HU-CHROME-02** | `Val::Percent(100.0)` — satisfies HU-CHROME-02 |
| `height` | `Val::Auto` | `Val::Percent(100.0)` |
| Site | `client/src/ui/hand/mod.rs:3977-3987` (without override) | `client/src/ui/hand/mod.rs:3970-3987` (override + spawn) |

No other direct fan-slot child was or is in violation. The canary is now
satisfied by construction across all twelve direct children listed above.

## Validation commands

- `git diff --check` → clean, no edits in this worker.
- `cargo test --test hand_ui_chrome_composition_test fan_slot_chrome_children_have_absolute_layout_after_placement_entry -- --nocapture` → `1 passed`.
- `cargo test --test hand_ui_chrome_composition_test` → `1 passed`.
- `cargo test --test hand_ui_fan_layout_formula_test` → `6 passed`.

MSVC Cargo policy honoured: `CARGO_TARGET_DIR=D:/tmp/cargo-target-1511`,
`CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`; no broad workspace tests.

## Recommendation

No source change required against current `origin/main`. The worker branch
`repair/hu-chrome-02-hand-fan-1511` carries no commits beyond `5d46b9a9` and
need not be pushed. Re-run PROMPT 1510's failing scenario against a worktree
freshly rebased onto `origin/main = 5d46b9a9` to confirm the canary is green
in their environment too; if it still fails there, the regression is in their
local stack (likely the art-node override at `mod.rs:3970-3975` got dropped
during a merge or rebase).

1511: REPAIR-HU-CHROME-02-HAND-FAN-ALL-DIRECT-CHILDREN: NOOP-ALREADY-GREEN
