# Story 011: S16-TD-UI-HAND-MODSPLIT-001 -- Hand Module Split

> **Epic**: UI Clean-Pass
> **Story ID**: S16-TD-UI-HAND-MODSPLIT-001
> **Status**: Draft -- Sprint 16/17 candidate, NOT activated
> **Layer**: Presentation / UX foundational tech-debt (module boundary)
> **Type**: Tech Debt -- structural refactor (file split, no behaviour change)
> **Sprint**: Sprint 16 Phase A.2 candidate per PROMPT 1035 §"Suggested
> refactor sequence for Sprint 16 / 17" (parallel-safe with story 010
> `S16-TD-UI-SHOPAUCTION-MODSPLIT-001`). May slip to Sprint 17 at producer
> discretion if Sprint 16 capacity is reserved for the card-slot
> primitive headline row (story 009).
> **Authored**: 2026-05-17 by PROMPT 1044
> **Authoring source-of-truth**: `origin/main@a7a8b079` (PROMPT 1041
> `chore(state): record UI audit consumption (PROMPT 1041)`).
> **Estimated effort**: ~0.5d (file split + re-export wiring; smaller
> than shop_auction split because hand module is ~24 % fewer lines)

---

## Status / No-Claim Banner

This story is authored as a Sprint 16/17 candidate. **No sprint is
activated by this authoring run.** The story is paperwork only -- no
code change is attempted by PROMPT 1044.

PROMPT 1044 (this authoring run) does NOT:

- Activate Sprint 16 or Sprint 17.
- Modify `production/sprint-status.yaml`.
- Modify any `production/sprints/*` file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify `Cargo.toml`, `Cargo.lock`, `.cargo/`, or any build configuration.
- Modify any docs / spec file.
- Author `client/src/ui/hand/fan.rs`, `draft_grid.rs`, `reserve.rs`,
  `submit.rs`, or `state.rs` (those are the future `/dev-story`
  output).

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), Standard-tier hit-target conformance (≥44px),
playtest / fun-hypothesis validation (`QA-COND-0006`), full playable-client
manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`), final-art /
asset-production completion (`PAW-TD-*-a`), `Polish->Release` gate-check
retry, or stage advance from `Polish` to `Release`.

---

## Overview

PROMPT 1035 §"Module-size snapshot" identifies
`client/src/ui/hand/mod.rs` as the second-largest UI module:

- **4 149 lines** in one file.
- **4 distinct surfaces** hosted: Hand Fan, DraftInitial Grid (a
  duplicate of `shop_auction/draft_initial.rs::draft_initial_slot_node`
  with a different coordinate system), Reserve strip, Submit /
  action-panel controls.
- **11+ sync systems** registered against `HandUiPlugin` for fan
  layout, drag pickup/drop, draft-grid mount/unmount, reserve toggle,
  and submit/ready interactions.

The single-file shape produces the same three structural problems as
shop_auction (cross-surface collisions, ownership ambiguity, review
fatigue), plus one hand-specific debt: **`hand/drag_state_visuals.rs`
(Sprint 15 DONE) is forced to import its accent / success colours from
`shop_auction::auction_featured_card_*_color`** because no shared
palette module exists. The hand module split does not fix this
cross-module dependency (that is Phase B.1 colours story
`S16-TD-UI-COLORS-TOKEN-001`), but it cleanly separates hand-fan code
from draft-grid code so the Phase C.7 palette sweep can operate on
narrow file scopes per submodule.

The duplicate draft-initial grid implementation (`hand/mod.rs:3256`
`hand_draft_grid_slot_node` vs `shop_auction/mod.rs:4534`
`draft_initial_slot_node`) is **preserved verbatim by this split**.
Resolving the duplication is a separate Phase C.1 follow-on story
(`S16-UI-CARD-SLOT-MIGRATION-HAND-001` per PROMPT 1035 §"Phase C") that
runs AFTER story 009 card-slot primitive lands.

This story authors the future `/dev-story` worker's contract for a
**behaviour-preserving file split** of `client/src/ui/hand/mod.rs` into
a thin aggregator plus per-surface submodules. The split is
**re-exports only**: no production behaviour changes, no tuning value
changes, no public API renames. Per PROMPT 1035 §"Parallelization map",
this story owns the `s16-hand-modsplit` lane. The companion shop_auction
split is story 010 `S16-TD-UI-SHOPAUCTION-MODSPLIT-001`; the two splits
are **file-disjoint** and parallel-safe.

---

## Scope

### In Scope

#### Submodule layout

- Split `client/src/ui/hand/mod.rs` into the following submodules. The
  exact module names below are a strong recommendation; the
  `/dev-story` worker MAY choose alternate names so long as the
  cross-surface boundary set is preserved (one submodule per surface):

  | Submodule | Owns | Source-of-truth in current `mod.rs` (approximate lines) |
  |---|---|---|
  | `client/src/ui/hand/mod.rs` (aggregator) | `pub use` re-exports; `HandUiPlugin` definition + system registration; module declarations | top imports + plugin block |
  | `client/src/ui/hand/state.rs` | `HandUiRoot`, `HandFanRoot`, `HandDraftGridRoot`, `ReserveStripRoot`, `SubmitPanelRoot` markers; per-surface `Component` / `Resource` / `Event` / `Message` declarations; shared layout constants (`HAND_CARD_DISPLAY_*`, `HAND_DRAFT_GRID_CARD_*`, `HAND_BAR_HEIGHT_PX`, `FAN_SLOT_STAT_BADGE_PERCENT`, `FAN_SLOT_ICON_PERCENT`, fan layout config) | mod.rs:60-90 (constants), Component derives scattered through mod.rs |
  | `client/src/ui/hand/fan.rs` | Hand Fan surface: `HandFanLayoutConfig`, `spawn_fan_slot`, fan arc computation, fan slot chrome composition (7 chrome children per slot at `FAN_SLOT_*` percent anchors), fan slot card-stat badge wiring | mod.rs:~3100-3260 |
  | `client/src/ui/hand/draft_grid.rs` | DraftInitial grid surface in hand UI: `hand_draft_grid_slot_node`, draft-grid mount / unmount systems, 9-slot iteration | mod.rs:~3256-3289 |
  | `client/src/ui/hand/reserve.rs` | Reserve strip surface: `reserve_strip_node`, `reserve_strip_child_node`, reserve-toggle interaction | mod.rs:~3289-3310 |
  | `client/src/ui/hand/submit.rs` | Submit / action-panel surface: submit button node, ready toggle, submit-state visuals | mod.rs:~ (the `submit_*` and `ready_*` helpers; exact lines per current source) |

  The `/dev-story` worker MAY collapse `reserve.rs` and `submit.rs`
  into one `controls.rs` submodule if the reserve + submit + action-panel
  surfaces share state machines; the authored shape MUST keep `mod.rs`
  under **400 lines** (aggregator only, no surface logic).

#### Public-API contract preservation

- Every public item currently exported by `client/src/ui/hand/mod.rs`
  MUST remain importable via the same `client::ui::hand::*` path. The
  `mod.rs` aggregator re-exports each item. No consumer outside
  `client/src/ui/hand/` is required to update import paths.

- `client/src/ui/hand/drag_state_visuals.rs` (Sprint 15 DONE) is part
  of the `hand/` directory and **stays as-is**: its `use
  client::ui::shop_auction::auction_featured_card_*_color` imports
  are NOT modified by this split (resolving them is Phase B.1
  colours).

- The `HandUiPlugin` registration in `client/src/main.rs` MUST continue
  to work without change.

#### Existing test-bin import preservation

- Every test bin under `tests/integration/hand_ui/`,
  `tests/integration/presentation/`, and
  `tests/integration/ui_clean_pass/` that imports a symbol from
  `client::ui::hand::*` MUST continue to compile and pass without
  import changes. The split is **import-transparent**.

- Specifically the following test bins (PROMPT 1035 §"Test coverage
  gaps" enumerates them) MUST pass byte-for-byte the same assertions:
  - `tests/integration/hand_ui/hand_ui_chrome_composition_test.rs`
    (asserts 7 chrome children per fan slot at `20 / 15 / 50-7.5`
    percent anchors -- this test does NOT need to be updated; the
    chrome composition logic moves to `fan.rs` and its public symbols
    re-export through `mod.rs`).
  - `tests/integration/hand_ui/draft_initial_grid_test.rs` (asserts
    grid slot positions / cards).
  - Every other test bin that currently `use`s a `hand` symbol.

#### Anti-regression checks

- `cargo check -p client` passes against the split tree.
- `cargo test -p client --test 'hand_ui_*'` passes byte-for-byte the
  same assertions.
- `cargo test -p client --test 'ui_clean_pass_*'` passes byte-for-byte
  the same assertions.
- `cargo test -p client --test 'shop_auction_*'` passes (no shop /
  auction changes, but `auction_featured_card_lead_loss_test` imports
  hand visual constants indirectly via `drag_state_visuals.rs` consumers).
- A QA snapshot bundle captured at 1280 × 720 and 1920 × 1080 across
  DraftInitial, Placement (hand fan visible), and Auction (hand fan
  visible) phases matches the pre-split snapshot bundle pixel-for-pixel
  (or within QA-snapshot fuzz tolerance).

### Out of Scope

- **No public-API renames.** Every `pub` item keeps its current name
  and current path.
- **No new design tokens.** No `colors.rs`, no `card_slot.rs`, no
  `panel.rs` authored or migrated here.
- **No layout tweaks.** Every `Val::Px(...)` and `Val::Percent(...)`
  literal is preserved verbatim.
- **No duplicate-grid consolidation.** The `hand_draft_grid_slot_node`
  vs `shop_auction::draft_initial_slot_node` duplication is preserved.
  Resolution is Phase C.1 `S16-UI-CARD-SLOT-MIGRATION-HAND-001` AFTER
  story 009 card-slot primitive lands.
- **No drag-state-visuals re-author.** `drag_state_visuals.rs` is
  Sprint 15 DONE; its `shop_auction::auction_featured_card_*_color`
  imports are preserved.
- **No reserve-strip relocation.** PROMPT 1035 §"Hand -- Not migrated /
  debt" notes that the reserve strip is a HUD-adjacent readout that
  probably belongs in the HUD bottom-strip. Relocating it is a separate
  follow-on story; this split keeps it under `hand/reserve.rs`.
- **No fan-layout tuning.** `HandFanLayoutConfig::fan_base_margin_px =
  100.0`, `fan_half_spread_px = 280.0`, `arc_height_px = 10.0`,
  `max_rotation_deg = 10.0` are preserved verbatim.
- **No card-stat-format helper move.** `format_card_combat_stats` (if
  re-imported by hand fan code) stays in its current location;
  relocating to `design_tokens/card_stat_format.rs` is a follow-on per
  PROMPT 1035 §"Coverage of current active / recent repairs PROMPT 1029".
- **No `Polish->Release` gate-check retry.** PROMPT 761 FAIL preserved.

---

## Acceptance Criteria

All criteria are independently checkable BLOCKING criteria.

- [ ] **AC1 -- Submodule shape**: GIVEN the worker commit, WHEN the
  source tree under `client/src/ui/hand/` is inspected, THEN the
  directory contains AT LEAST: `mod.rs`, `state.rs`, `fan.rs`,
  `draft_grid.rs`, `reserve.rs`, `submit.rs` (or a single
  `controls.rs` if reserve + submit are co-located). `drag_state_visuals.rs`
  is UNCHANGED. The `mod.rs` aggregator is under **400 lines** (down
  from 4 149). No submodule exceeds **1 800 lines**. Verification: file
  presence + `wc -l`.

- [ ] **AC2 -- Re-export aggregator**: GIVEN the new `mod.rs`, WHEN
  inspected, THEN it consists only of (a) `mod state;` / `mod fan;` /
  etc. submodule declarations, (b) `pub use` re-exports for every
  previously-public item, (c) the `HandUiPlugin` definition + system
  registration (or a `mod plugin;` declaration), and (d) top-of-file
  imports needed for the plugin definition. No surface `fn` body
  lives in `mod.rs`. Verification: per-line grep + line count.

- [ ] **AC3 -- Import-transparent public API**: GIVEN the split tree,
  WHEN every existing consumer of `client::ui::hand::*` is compiled,
  THEN no consumer outside `client/src/ui/hand/` changes its `use`
  path. Verification: `git diff origin/main...HEAD -- 'client/src/'
  'tests/' 'server/' 'shared/' | grep '^[+-]use '` shows zero diff
  lines outside `client/src/ui/hand/**`.

- [ ] **AC4 -- `drag_state_visuals.rs` unchanged**: GIVEN the split
  tree, WHEN `client/src/ui/hand/drag_state_visuals.rs` is inspected,
  THEN it is byte-identical to its `origin/main` content at split
  time. Its `use shop_auction::auction_featured_card_*_color` imports
  are preserved verbatim. Resolving the wrong-direction dependency is
  Phase B.1 colours, NOT this row. Verification: `git diff
  origin/main...HEAD -- 'client/src/ui/hand/drag_state_visuals.rs'`
  is empty.

- [ ] **AC5 -- Behaviour preservation (cargo + tests)**: GIVEN the
  split tree, WHEN the test harness runs, THEN:
  - `cargo check -p client` succeeds with zero new warnings.
  - `cargo test -p client --test 'hand_ui_*'` passes byte-for-byte
    the same assertions as on `origin/main` pre-split (including
    `hand_ui_chrome_composition_test`'s 20 / 15 / 50-7.5 percent
    anchor assertions and `draft_initial_grid_test`'s grid-slot
    position assertions).
  - `cargo test -p client --test 'ui_clean_pass_*'` passes.
  - `cargo test -p client --test 'shop_auction_*'` passes (regression
    guard; shop_auction depends indirectly on hand types via
    drag_state_visuals).
  - `cargo test -p client --test 'presentation_*'` passes (regression
    guard; presentation layer may import `HandFanRoot` markers).
  Verification: test logs in evidence directory.

- [ ] **AC6 -- QA snapshot pixel parity**: GIVEN a QA snapshot bundle
  captured pre-split and post-split at 1280 × 720 and 1920 × 1080 across
  DraftInitial (hand draft grid visible), Placement R1 (hand fan
  visible), and Auction R3 (hand fan visible), THEN the PNG outputs
  are pixel-identical OR within QA-snapshot fuzz tolerance. The
  evidence directory at
  `production/qa/evidence/sprint-1X-hand-modsplit/` contains pre-split
  and post-split snapshot PNGs with a diff summary. Verification:
  snapshot directory contents + diff log.

- [ ] **AC7 -- No public-API rename, no behaviour change, no token
  introduction**: GIVEN the worker diff, WHEN inspected, THEN:
  - No `pub fn`, `pub struct`, `pub enum`, or `pub const` is renamed.
  - No `Val::Px(...)`, `Val::Percent(...)`, `Color::srgb*(...)` literal
    is changed.
  - No new file under `client/src/ui/design_tokens/` is authored.
  - No system registration order is changed.
  - `HandFanLayoutConfig` defaults are preserved verbatim.
  - No `Resource` / `Component` / `Event` / `Message` derive is added
    or removed.
  Verification: `git diff origin/main...HEAD -- 'client/src/ui/hand/'`
  is a pure file-move + re-export diff.

- [ ] **AC8 -- Non-claims**: GIVEN the worker commit, WHEN closure
  paperwork is inspected, THEN this row does NOT:
  - Modify any gameplay logic.
  - Modify any server / shared / protocol module.
  - Claim public release readiness, full game completion, manual QA,
    playtest validation, Standard-tier accessibility, final-art,
    two-client GAME_OVER closure, `Polish->Release` retry, or stage
    advance.
  - Reopen any Sprint 14 / Sprint 15 / Sprint 16 closed row.
  Verification: `git diff origin/main...HEAD --stat -- 'server/' 'shared/'`
  is empty.

---

## Implementation Notes

### Owned files (the future `/dev-story` worker authors these)

| Path | Expected change |
|------|-----------------|
| `client/src/ui/hand/mod.rs` | Reduce from 4 149 lines to ~400-line aggregator. |
| `client/src/ui/hand/state.rs` (NEW) | Move all `Component` / `Resource` / `Event` / `Message` declarations + shared layout constants. |
| `client/src/ui/hand/fan.rs` (NEW) | Move Hand Fan surface code (slot composition, chrome children, fan arc). |
| `client/src/ui/hand/draft_grid.rs` (NEW) | Move DraftInitial grid surface code. |
| `client/src/ui/hand/reserve.rs` (NEW) | Move reserve strip code. |
| `client/src/ui/hand/submit.rs` (NEW) | Move submit / action-panel code. |
| `production/qa/evidence/sprint-1X-hand-modsplit/` (NEW) | Evidence dir: pre/post QA snapshot bundles, cargo logs, doc-review checklist, `git diff --stat` proving import-transparent change set. |

### Forbidden files

- `client/src/ui/hand/drag_state_visuals.rs` -- Sprint 15 DONE;
  UNCHANGED here.
- `client/src/ui/shop_auction/**` -- shop_auction split is story 010.
- `client/src/ui/hud/**`, `client/src/ui/lobby.rs`,
  `client/src/ui/settings/**`, `client/src/ui/photosensitivity_warning.rs`,
  `client/src/ui/design_tokens/**` -- not part of this split.
- `client/src/presentation/**` -- not part of this split.
- `client/src/gameplay/**`, `server/src/**`, `shared/src/**` -- UNCHANGED.
- `tests/integration/**` -- existing tests UNCHANGED in source.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/qa-plan-*.md` -- shared-state writers.
- `docs/**`, `.claude/**`, `AGENTS.md`, `CLAUDE.md` -- not part.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `Trunk.toml` -- unchanged.

### Module integration touch points

- `client/src/main.rs` (or wherever `HandUiPlugin` is added) MUST NOT
  change.
- `client/src/ui/hand/drag_state_visuals.rs` MUST NOT change.
- `client/src/presentation/board_rendering.rs` (Sprint 13/14
  drag-runtime path) MAY import `HandFanRoot` or `HandFanSlot` markers
  through `client::ui::hand::*`; the `mod.rs` re-export preserves
  these paths.
- `tests/integration/hand_ui/**` imports must continue to resolve.

---

## Parallelization and Phase Breakdown

### Parallel-safety with sibling stories

| Sibling story | File scope | Parallel-safe with this row? |
|---|---|---|
| **Story 010 `S16-TD-UI-SHOPAUCTION-MODSPLIT-001`** | `client/src/ui/shop_auction/` | **YES** -- file-disjoint. |
| **Story 012 `S16-TD-UI-MODAL-PRIMITIVE-001`** | `design_tokens/` or `primitives/` + a single canonical migration site (default: a non-hand surface; lobby or result-screen) | Authoring: **YES**. Migration site: parallel-safe with this split if migration target is non-hand. |
| **Story 013 `S16-TD-UI-BUTTON-PRIMITIVE-001`** | Same as 012 | Authoring: **YES**. Per-surface migration of the hand submit button MUST land AFTER this split. |
| **Story 014 `S16-TD-UI-PANEL-PRIMITIVE-001`** | Same as 012 | Authoring: **YES**. |
| **Story 015 `S16-TD-UI-ARCHITECTURE-SEQUENCING-001`** | doc-only | **YES**. |
| **Story 009 `S12-TD-UI-CARD-SLOT-PRIMITIVE-001`** | shop slot phase-1 migration | **YES** -- phase-1 migration is in `shop_auction/`, not `hand/`. Phase C.1 `S16-UI-CARD-SLOT-MIGRATION-HAND-001` MUST land AFTER this split. |

### Files that MUST serialize

- `client/src/ui/hand/mod.rs` is owned exclusively by this row during
  the split. Any other Sprint 16+ row that touches a hand surface
  (fan, draft-grid, reserve, submit) MUST wait for this split to
  reach `origin/main` first.
- `production/sprint-status.yaml` (orchestrator-serialized).

### Dependencies and unblockers

- **No prerequisite stories.** This split is the foundation row of
  Phase A.2.
- **Unblocks** (per PROMPT 1035 §"Phase C"):
  - Phase C.1 `S16-UI-CARD-SLOT-MIGRATION-HAND-001` (hand fan +
    draft-grid migration to card-slot primitive).
  - Story 013 button primitive's hand-submit-button migration phase.
  - Phase C.7 palette sweep over hand fan badge colours (once Phase
    B.1 colours story lands).
  - Future story to relocate reserve strip to HUD bottom-strip (out of
    scope here).

---

## Dependency Map (for this story only)

| Direction | Dep | Reason |
|---|---|---|
| **Prerequisite** | `origin/main` HEAD at activation | Source tree must be clean. |
| **Prerequisite** | None within ui-clean-pass epic | This is the foundation row of Phase A.2. |
| **Unblocks** | Phase C.1 `S16-UI-CARD-SLOT-MIGRATION-HAND-001` | Hand fan + draft grid migration to card-slot primitive (after story 009 lands). |
| **Unblocks** | Story 013 button primitive (hand submit migration) | Hand submit button moves to `submit.rs` after split. |
| **Unblocks** | Future palette sweep over hand fan badges | Narrow file scope per submodule. |
| **Parallel-safe** | Story 010 shop_auction modsplit | File-disjoint. |
| **Parallel-safe** | Story 012 / 013 / 014 primitive authoring (not migration) | Different directory. |
| **Conflicts** | Any Sprint 16+ row touching hand fan / draft-grid / reserve / submit before this split lands | Same file. |

---

## Worker Contract (for `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout -b work/s16-hand-modsplit` from `origin/main` HEAD.
2. Read `client/src/ui/hand/mod.rs` in full and inventory:
   - Every `pub` item (count + names).
   - Every system registered against `HandUiPlugin`.
   - Every `drag_state_visuals.rs` import (these MUST resolve via the
     new `mod.rs` re-export).
3. Author the submodule split per AC1's table.
4. Verify AC2-AC8 locally:
   - `cargo check -p client`
   - `cargo test -p client --test 'hand_ui_*'`
   - `cargo test -p client --test 'ui_clean_pass_*'`
   - `cargo test -p client --test 'shop_auction_*'`
   - `cargo test -p client --test 'presentation_*'`
5. Capture QA snapshot bundles per AC6.
6. Push `work/s16-hand-modsplit`. Do NOT push `main`.
7. Hand off to the integration prompt for `origin/main` merge.

The worker MUST NOT:

- Rename any `pub` item.
- Change any layout literal.
- Touch `drag_state_visuals.rs` (Sprint 15 DONE).
- Touch any forbidden file.
- Run `/story-done` / `/smoke-check` / `/team-qa` / `/gate-check` /
  `/release-check` / `/qa-plan`.
- Modify `production/sprint-status.yaml`,
  `production/sprints/sprint-XX.md`, `production/stage.txt`, or
  `production/session-state/*`.

---

`011: S16-TD-UI-HAND-MODSPLIT-001: DRAFT`
