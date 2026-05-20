# PROMPT 1508 — REPAIR-HU-CHROME-02-HAND-FAN

**Branch**: `work/repair-hu-chrome-02-hand-fan-1508`  
**Base**: `origin/main@1172e165`

## Failure reproduced

```
cargo test -p client --test hand_ui_chrome_composition_test \
  fan_slot_chrome_children_have_absolute_layout_after_placement_entry
```

Panic at `tests/integration/hand-ui/hand_ui_chrome_composition_test.rs:93`:

```
chrome child 5v0 Node.width must not be Val::Auto
(HU-CHROME-02 regression canary — Node::default() reproduces the Verdict B 0×0 bug)
```

## Offending entity

Direct `ChildOf(slot)` entity carrying the `CardSlotArtImage` marker, spawned at
`client/src/ui/hand/mod.rs:3960` via the shared
`card_slot_art_image_node(CardSlotKind::HandFan)` builder
(`client/src/ui/design_tokens/card_slot.rs:940` → `card_slot_image_inset_node`,
line 692).

That builder ships a Node with `position_type: Absolute`, `left/right/top/bottom
= Val::Px(...)` from `CARD_SLOT_HAND_FAN_IMAGE_INSET` (4/4/4/28), and
`..default()` for `width` / `height` — i.e. `Val::Auto`. The HU-CHROME-02 canary
mandates `Val::Percent(>0)` for every direct slot child.

**Root cause history**: The hand-fan art-image child was introduced in
PROMPT 1348 (`26bc1204`) without updating its width/height to satisfy the
HU-CHROME-02 contract authored in PROMPT 682 (`f190cc76`). The canary was a
latent regression all along; PROMPT 1506 focused verify on top of the
1505 R2 main refresh surfaced it.

PROMPT 1490 did **not** add this child; it only nudged constants
(`HAND_CARD_DISPLAY_*`, `EXPECTED_STAT_BADGE_PERCENT`) and overlay polish. The
spawn site was unchanged but the test compilation/run was triggered fresh.

## Fix

`client/src/ui/hand/mod.rs:3959-3972` — after calling the shared builder,
override the art Node to percent-based 100%×100% sizing anchored top-left.
The hand-fan surface has no label-strip child, so the art image covers the
full slot box; chrome (frame, stat badges, icons, affordance overlays) is
absolute-positioned and paints on top.

```rust
let (mut art_node, art_z) = card_slot_art_image_node(CardSlotKind::HandFan);
// PROMPT 1508 — HU-CHROME-02 regression canary requires every direct
// ChildOf(slot) to declare an explicit positive Val::Percent width
// (Node::default() / Val::Auto reproduces the Verdict B 0×0 bug). [...]
art_node.left   = Val::Percent(0.0);
art_node.right  = Val::Auto;
art_node.top    = Val::Percent(0.0);
art_node.bottom = Val::Auto;
art_node.width  = Val::Percent(100.0);
art_node.height = Val::Percent(100.0);
```

This is the minimum-blast-radius repair: surgical at the hand spawn site,
no change to the shared `card_slot_art_image_node` builder (other surfaces
keep the px-inset sizing), no constant churn, no test edits.

## Diagnostic tracing

None added. The failure message already pinpointed entity `5v0` and the
panic line was the width assertion; reading the spawn-site source plus the
shared builder isolated the offender without instrumentation.

## Files changed

- `client/src/ui/hand/mod.rs` (+18 / -2 around L3959)

## Validation

| Command | Result |
|---|---|
| `git diff --check` | clean |
| `cargo test -p client --test hand_ui_chrome_composition_test -- fan_slot_chrome_children_have_absolute_layout_after_placement_entry --nocapture` | PASS |
| `cargo test -p client --test hand_ui_chrome_composition_test` | PASS (1/1) |
| `cargo test -p client --test hand_ui_fan_layout_formula_test` | PASS (6/6) |

## Deferred VERIFY

None required for the HU-CHROME-02 contract. Visual verification of the
hand-fan card art now stretching to 100%×100% of the slot box (vs. the
previous 4/4/4/28 px inset) is recommended at the next on-screen verify
pass — if the bottom 28 px reservation is reinstated for a future
hand-fan label strip, the art override here should switch to
percent-equivalents of those insets instead of 100% × 100%.

1508: REPAIR-HU-CHROME-02-HAND-FAN: DONE
