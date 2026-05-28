# PROMPT-2052 — Card-Asset / Hand-HUD Placeholder Repair Map

**Date**: 2026-05-29
**Source-of-truth at start**: `origin/main@450e3908`
**Branch**: `work/PROMPT-2052`
**Worktree**: `D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-2052`
**Related**: `reports/PROMPT-2045-card-asset-shop-placeholder-visual-repair.md`,
PROMPT 2038 (bid-button hover repair).

## Outcome

**STATUS**: SHIPPED (report-only, no source edits in this PROMPT).

This audit closes out the three remaining `NO ANALOGUE on disk` placeholder
routes that PROMPT 2045 flagged as still pointing at the universal `?`
placeholder PNG (`art/characters/ui_unit_placeholder_default_board.png`).
Each one is mapped to the smallest safe disk-honest substitute and scoped as
a constant-only re-point at the asset-wiring chokepoint
(`client/src/asset_wiring.rs`). All three repairs are mutually isolated and
do not overlap the shop/auction binding chain repaired by PROMPT 2045.

Source edits are deliberately deferred: each re-point will land as its own
follow-up PROMPT once the visual proxy is signed off by the hand / HUD
owners. This file is the actionable spec for those follow-ups.

## Scope reminder

In-scope (PROMPT 2052):
- `STAT_BADGE_AR_ASSET` (hand-card armor stat badge)
- `HUD_PHASE_TIMER_BAR_ASSET` (HUD top-strip phase timer bar fill)
- `HUD_OBJECTIVE_DOT_DESTROYED_ASSET` (HUD scoreboard destroyed dot)

Out of scope: any shop / draft-initial / auction surface (owned by PROMPT
2045 / 2038), any non-hand non-HUD constant, the asset-loader plumbing.

## Per-route repair map

### 1. `STAT_BADGE_AR_ASSET` — hand-fan armor badge

**Current binding** (`client/src/asset_wiring.rs:27`):
```rust
// NO ANALOGUE on disk — repointed to universal placeholder.
pub const STAT_BADGE_AR_ASSET: &str = "art/characters/ui_unit_placeholder_default_board.png";
```

**Loaded into** `PlaceholderAssets.stat_badge_ar` (lines 278, 363, 449).

**Rendered at** `client/src/ui/hand/mod.rs:4299-4308` — spawned as an
`ImageNode::new(placeholder.stat_badge_ar.clone())` in the top-right corner
of every fan slot (`fan_slot_stat_badge_node(StatBadgeCorner::TopRight)`).
The slot is always spawned, so every hand card paints the universal `?`
glyph in its top-right badge corner today.

**Disk reality**: no `ui_badge_ar*` / `ui_badge_armor*` / `ui_badge_def*`
file exists anywhere under `assets/art/ui/`. The other three stat badges
live in `assets/art/ui/hand/` and follow the
`ui_badge_<stat>_default_hud.png` convention:

| Stat | Asset on disk |
|---|---|
| ATK | `art/ui/hand/ui_badge_atk_default_hud.png` |
| HP  | `art/ui/hand/ui_badge_hp_default_hud.png`  |
| MP  | `art/ui/hand/ui_badge_mana_neutral_default_hud.png` |
| AR  | **missing** |

**Smallest safe substitute**: re-point to the **HP badge**
(`art/ui/hand/ui_badge_hp_default_hud.png`). HP and Armor are the two
defensive stats in the rules; reusing the HP badge keeps the corner
visually populated with a defensive-tier icon instead of a literal `?`
question mark. The numeric label child (`StatBadgeArLabel`,
`client/src/ui/hand/mod.rs:4309-4319`) already binds the correct AR value
on top of the chrome, so the duplicated HP background reads as "defensive
slot" without misrepresenting the value.

**Rejected alternatives**:
- Hiding the AR badge entirely → loses the AR value readout (the label rides
  on the badge as `ChildOf(ar_badge)`).
- Tinting the HP badge a different colour → requires a spawn-site edit
  (`ImageNode { color, .. }`) and breaks the constant-only chokepoint pattern
  the rest of the asset-wiring file follows.
- Authoring a new PNG → out of scope (PROMPT 2052 is a repair map, not an
  art-production prompt).

**Repair**: one-line edit in `client/src/asset_wiring.rs:27`:
```rust
pub const STAT_BADGE_AR_ASSET: &str = "art/ui/hand/ui_badge_hp_default_hud.png";
```
…and drop the `NO ANALOGUE on disk` comment line.

**Test impact**: `tests/integration/presentation/hand_ui_asset_wiring_test.rs`
already asserts the AR ImageNode is present
(`test_fan_slot_chrome_stat_badge_ar_image_node_present`). It does not
assert path uniqueness, so the re-point passes without test churn. Optional:
a new selector-style assertion mirroring PROMPT 2045's
`test_bid_button_selector_never_routes_to_placeholder` —
`test_stat_badge_ar_never_routes_to_placeholder`.

---

### 2. `HUD_PHASE_TIMER_BAR_ASSET` — HUD phase timer bar fill

**Current binding** (`client/src/asset_wiring.rs:90`):
```rust
// NO ANALOGUE on disk — repointed to universal placeholder.
pub const HUD_PHASE_TIMER_BAR_ASSET: &str = "art/characters/ui_unit_placeholder_default_board.png";
```

**Loaded into** `PlaceholderAssets.hud_phase_timer_bar` (lines 316, 397, 483).

**Rendered at** `client/src/ui/hud/mod.rs:915-939` — spawned as an
`ImageNode::new(server.load(HUD_PHASE_TIMER_BAR_ASSET))` whose `Node.width`
is animated 0 → `HUD_PHASE_TIMER_BAR_MAX_WIDTH_PX` (176 px,
`client/src/ui/hud/mod.rs:64`) by `sync_hud_timer_bar_system` based on the
`PhaseTimerState` remaining ratio. The image is intentionally treated as a
solid-fill bar — only the width changes.

**Disk reality**: no `ui_phase_timer*` / `ui_timer_bar*` / `ui_progress*`
PNG exists. However, a generic flat-fill texture is already present:
`assets/art/characters/hp_bar_white_pixel_1x2.png` — a 1×2 white pixel that
the board renderer already uses as an HP-bar fill (precedent:
`client/src/presentation/board_rendering.rs:71,113,126,3413` —
`HP_BAR_WHITE_PIXEL_ASSET` / `BoardAssets.hp_bar_white_pixel`).

**Smallest safe substitute**: re-point `HUD_PHASE_TIMER_BAR_ASSET` to
`art/characters/hp_bar_white_pixel_1x2.png`. A 1×2 white texture stretched
to a 176×N bar via `Node.width` gives a clean, untinted fill — semantically
correct for a bar that exists only to communicate "fraction remaining".
This is the smallest possible edit (constant-only, no spawn-site change).

If a tinted fill is later desired, the spawn site at
`client/src/ui/hud/mod.rs:917-921` can be extended with
`ImageNode { color: <design-token>, .. }` in a follow-up — but that change
is independent of this re-point.

**Rejected alternatives**:
- Reuse `ui_auction_panel_bg_default_hud.png` or `ui_shop_panel_chrome.png`
  → those are textured panels, not a clean fill; stretching them to a thin
  176×N bar will smear visible chrome patterns.
- Drop the `ImageNode` and use `BackgroundColor` instead → requires deleting
  the spawn-site `ImageNode::new(...)` plus updating the
  asset-wiring-foundation tests that assert the timer bar has a present
  image handle. Larger blast radius than the constant re-point.

**Repair**: one-line edit in `client/src/asset_wiring.rs:90`:
```rust
pub const HUD_PHASE_TIMER_BAR_ASSET: &str = "art/characters/hp_bar_white_pixel_1x2.png";
```
…and drop the `NO ANALOGUE on disk` comment line.

**Test impact**: `tests/integration/presentation/hud_asset_wiring_test.rs`
verifies the bar has a loaded handle (path-agnostic). No assertion locks
the universal placeholder path. Optional: add
`test_hud_phase_timer_bar_never_routes_to_placeholder`.

---

### 3. `HUD_OBJECTIVE_DOT_DESTROYED_ASSET` — HUD scoreboard destroyed dot

**Current binding** (`client/src/asset_wiring.rs:93-95`):
```rust
// NO ANALOGUE on disk — repointed to universal placeholder.
pub const HUD_OBJECTIVE_DOT_DESTROYED_ASSET: &str =
    "art/characters/ui_unit_placeholder_default_board.png";
```

**Loaded into** `PlaceholderAssets.hud_objective_dot_destroyed` (lines 318,
399, 485) and selected by
`hud_objective_dot_asset(ObjectiveDotState::Destroyed)` (line 230).

**Rendered at**:
- `client/src/ui/hud/mod.rs:1873` —
  `img.image = server.load(HUD_OBJECTIVE_DOT_DESTROYED_ASSET)` when a
  destroyed-update event arrives.
- `client/src/ui/hud/mod.rs:1921` — `sync_scoreboard_dot_image_for_state_system`
  routes any dot whose `ScoreboardDotState.destroyed` is `true` through this
  asset.

So every destroyed objective dot in both rows shows the universal `?`
placeholder today, while alive / unknown / fake dots all bind real PNGs:

| State     | Asset on disk |
|---|---|
| Alive     | `art/board/env_objective_real_reveal_board.png`  |
| Unknown   | `art/board/env_objective_unknown_board.png`      |
| Fake      | `art/board/env_objective_fake_crack_board.png`   |
| Destroyed | **missing** |

**Smallest safe substitute**: re-point to
`art/board/env_objective_fake_crack_board.png`. The fake-crack asset depicts
a cracked / broken objective — the exact visual semantic for "destroyed"
(the fake-vs-real distinction is a separate gameplay concept that surfaces
only on objective reveal; in the scoreboard context the cracked silhouette
reads as "no longer intact"). Reusing it is a strictly better visual proxy
than the universal `?` glyph and stays on the same board-objective art set
as the other three states.

**Rejected alternatives**:
- Tinting the Alive asset dark → requires spawn-site / sync-system edits
  (the dot already drives image swaps via `image.image =`; adding a `color`
  field would interfere with future per-state tints and tests). Bigger
  blast radius than a constant re-point.
- Reusing the Unknown asset → semantically wrong; "unknown" is fog-of-war
  pre-snapshot, not a post-destruction state. `sync_scoreboard_dot_image_for_state_system`
  already distinguishes the two via `state.known` vs `state.destroyed`
  (`client/src/ui/hud/mod.rs:1918-1924`), so collapsing them would lose the
  distinction in the alive/destroyed transition.
- Hiding the dot → breaks the 24-entity `HUD_ENTITY_COUNT` invariant
  documented at `client/src/ui/hud/mod.rs:50`.

**Repair**: two-line edit in `client/src/asset_wiring.rs:93-95`:
```rust
pub const HUD_OBJECTIVE_DOT_DESTROYED_ASSET: &str =
    "art/board/env_objective_fake_crack_board.png";
```
…and drop the `NO ANALOGUE on disk` comment line.

**Test impact**:
- `tests/integration/presentation/asset_wiring_foundation_test.rs:103`
  references the constant but tests path-presence, not path-uniqueness vs
  the fake asset.
- `tests/integration/hud/hud_scoreboard_dot_image_refresh_test.rs:216`
  asserts `expected_handle(&app, HUD_OBJECTIVE_DOT_DESTROYED_ASSET)` — this
  passes through the constant, so the re-point flows through without
  modification.
- `tests/integration/presentation/hud_asset_wiring_test.rs:211` is a
  comment-only reference.

No test churn required. Optional: add
`test_hud_objective_dot_selector_never_routes_to_placeholder`.

---

## Cross-route summary

| Constant | Current target | Repair target | Edit site | Blast radius |
|---|---|---|---|---|
| `STAT_BADGE_AR_ASSET` | `art/characters/ui_unit_placeholder_default_board.png` | `art/ui/hand/ui_badge_hp_default_hud.png` | `client/src/asset_wiring.rs:27` | 1 line + comment |
| `HUD_PHASE_TIMER_BAR_ASSET` | `art/characters/ui_unit_placeholder_default_board.png` | `art/characters/hp_bar_white_pixel_1x2.png` | `client/src/asset_wiring.rs:90` | 1 line + comment |
| `HUD_OBJECTIVE_DOT_DESTROYED_ASSET` | `art/characters/ui_unit_placeholder_default_board.png` | `art/board/env_objective_fake_crack_board.png` | `client/src/asset_wiring.rs:93-95` | 2 lines + comment |

All three repairs:
- Are isolated to the asset-wiring constant chokepoint — no spawn-site
  edits, no system edits, no asset-loader plumbing changes.
- Do not touch the shop / draft-initial / auction binding chain (already
  repaired by PROMPTs 2038 / 2045).
- Survive the existing test surface unmodified.
- Open the door for follow-up `…_never_routes_to_placeholder` selector
  guards mirroring PROMPT 2045's pattern, to prevent regressions.

## Recommended follow-up PROMPTs

Each constant should ship as its own atomic PROMPT to keep blame and revert
lanes clean (one constant, one PNG re-point, one optional selector test):

- **PROMPT-NEXT-A** — repoint `STAT_BADGE_AR_ASSET` → HP badge PNG.
- **PROMPT-NEXT-B** — repoint `HUD_PHASE_TIMER_BAR_ASSET` → white-pixel fill.
- **PROMPT-NEXT-C** — repoint `HUD_OBJECTIVE_DOT_DESTROYED_ASSET` → fake-crack PNG.

All three can run in parallel (no file conflicts beyond same-file adjacent
constants in `asset_wiring.rs` — trivially mergeable if staggered, or one
combined PROMPT if the user prefers a single landing commit).

## Validation performed

- `git fetch origin main` → checked out fresh `work/PROMPT-2052` from
  `origin/main@450e3908`.
- Grep / read pass against the three constants and their render sites.
- Disk survey of `assets/art/ui/{hand,hud,board,shop,auction,shop_auction}`
  and `assets/art/board/`, `assets/art/characters/` to confirm no
  authored substitute exists for AR / phase-timer / destroyed-dot.
- Precedent check: `hp_bar_white_pixel_1x2.png` already used as an
  HP-bar fill by `client/src/presentation/board_rendering.rs`.
- No source edits made by this PROMPT — `git status` clean except this
  report file. No `git diff --check` issues possible.
- Focused tests not rerun: report-only PROMPT, scope is the repair map, not
  the re-points themselves.

## Final line

2052: CARD-ASSET-HAND-HUD-PLACEHOLDER-REPAIR-MAP: SHIPPED
