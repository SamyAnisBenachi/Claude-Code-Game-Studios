# Story 010: S16-TD-UI-SHOPAUCTION-MODSPLIT-001 -- Shop/Auction Module Split

> **Epic**: UI Clean-Pass
> **Story ID**: S16-TD-UI-SHOPAUCTION-MODSPLIT-001
> **Status**: Draft -- Sprint 16/17 candidate, NOT activated
> **Layer**: Presentation / UX foundational tech-debt (module boundary)
> **Type**: Tech Debt -- structural refactor (file split, no behaviour change)
> **Sprint**: Sprint 16 Phase A.1 candidate per PROMPT 1035 §"Suggested
> refactor sequence for Sprint 16 / 17" (parallel-safe with story 011
> `S16-TD-UI-HAND-MODSPLIT-001`). May slip to Sprint 17 at producer
> discretion if Sprint 16 capacity is reserved for the card-slot
> primitive headline row (story 009).
> **Authored**: 2026-05-17 by PROMPT 1044
> **Authoring source-of-truth**: `origin/main@a7a8b079` (PROMPT 1041
> `chore(state): record UI audit consumption (PROMPT 1041)`).
> **Estimated effort**: ~1.0d (file split + re-export wiring + existing
> test-bin import survey; no behaviour change)

---

## Status / No-Claim Banner

This story is authored as a Sprint 16/17 candidate. **No sprint is
activated by this authoring run.** The story is paperwork only -- no
code change is attempted by PROMPT 1044.

PROMPT 1044 (this authoring run) does NOT:

- Activate Sprint 16 or Sprint 17.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-14.md`, `production/sprints/sprint-15.md`,
  `production/sprints/sprint-16.md` (draft), or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify `Cargo.toml`, `Cargo.lock`, `.cargo/`, or any build configuration.
- Modify `docs/ux/global-ui-design-spec.md` or any other docs file.
- Author `client/src/ui/shop_auction/draft_initial.rs`,
  `client/src/ui/shop_auction/shop.rs`,
  `client/src/ui/shop_auction/auction.rs`, or any sibling submodule
  (those are the future `/dev-story` output).

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
`client/src/ui/shop_auction/mod.rs` as the single largest architectural
debt in the playable client:

- **5 435 lines** in one file.
- **6 distinct surfaces** hosted in the module: DraftInitial centred
  modal, Shop panel, Auction panel, Settlement overlay, Toasts, Footer
  slots.
- **41 inline `*_node()` builders**, 55 inline `Color::srgba(...)`
  literals, 113 raw `Val::Px(...)` integer literals.
- **13+ sync systems** registered against `ShopAuctionPlugin` cover the
  three surfaces' visibility flips, bid handling, settlement transitions,
  toast drain, and footer hand-card refresh.

The single-file shape produces three structural problems that block
parallel surface work:

1. **Cross-surface collisions.** Any two workers touching the shop, the
   auction, the draft modal, the settlement overlay, the toast stack, or
   the footer hand-card row must serialize against `mod.rs`. Sprint 16
   Phase C migrations (card-slot per-surface migration, auction flex
   primitives, shop control row, modal panel consolidation) need
   parallel-safe file boundaries.
2. **Ownership ambiguity.** Helpers like `format_card_combat_stats`,
   `auction_featured_card_accent_color`, the bid-button colour matchers,
   and the toast text builders sit in a single 5 435-line file where
   peer surface modules (e.g. `hand/drag_state_visuals.rs`) reach back
   into them by `pub fn`. This is the wrong direction of dependency:
   surface helpers should not be the source for cross-surface tokens.
3. **Review fatigue.** A `git diff` that touches `mod.rs` is hard to
   review because the file mixes resource declarations, message
   handlers, spawn helpers, layout helpers, colour helpers, and three
   independent state machines. A reviewer cannot tell which surface a
   given change targets without a wide context window.

This story authors the future `/dev-story` worker's contract for a
**behaviour-preserving file split** of `client/src/ui/shop_auction/mod.rs`
into a thin aggregator plus per-surface submodules. The split is
**re-exports only**: no production behaviour changes, no tuning value
changes, no public API renames at the module boundary (existing
consumers under `tests/integration/shop_auction_ui/**`,
`tests/integration/ui_clean_pass/**`, `client/src/ui/hand/drag_state_visuals.rs`,
`client/src/main.rs` plugin registration, and any `pub use shop_auction::*`
re-exports continue to work without import changes).

Per PROMPT 1035 §"Parallelization map", this story owns the
`s16-shop-auction-modsplit` lane. Phase B.2 (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`,
story 009) and every Phase C migration that touches shop, auction,
draft, settlement, toasts, or footer slots is **blocked** on this split
landing first. The companion module split for `client/src/ui/hand/mod.rs`
is story 011 `S16-TD-UI-HAND-MODSPLIT-001`; the two splits are
**file-disjoint** and parallel-safe.

---

## Scope

### In Scope

#### Submodule layout

- Split `client/src/ui/shop_auction/mod.rs` into the following submodules.
  The exact module names below are a strong recommendation; the
  `/dev-story` worker MAY choose alternate names so long as the
  cross-surface boundary set is preserved (one submodule per surface,
  no two surfaces sharing a submodule):

  | Submodule | Owns | Source-of-truth in current `mod.rs` (approximate lines) |
  |---|---|---|
  | `client/src/ui/shop_auction/mod.rs` (aggregator) | `pub use` re-exports for all public types/fns/consts; `ShopAuctionPlugin` definition + system registration; module declarations (`mod state;`, `mod draft_initial;`, …) | top-of-file imports, the `ShopAuctionPlugin` impl block, `pub use` lines |
  | `client/src/ui/shop_auction/state.rs` | `ShopAuctionUiRoot`, `ShopAuctionPanelRoot`, `AuctionPanelRoot`, `DraftInitialOverlayRoot`, `SettlementOverlayRoot`, `ToastQueue`, `ToastRoot`, any per-surface root markers; all `Component`/`Resource`/`Event`/`Message` declarations consumed across multiple surfaces; per-surface state-machine enums (e.g. `DraftInitialPhase`, `AuctionBidState`); shared layout constants (`DRAFT_INITIAL_GRID_*`, `AUCTION_FEATURED_CARD_*`, `SHOP_SLOT_*`) | mod.rs:30-100 (constants), `Component` derives scattered through mod.rs |
  | `client/src/ui/shop_auction/spawn.rs` | `spawn_shop_auction_ui` (the top-level entry) + the shared bottom-panel root spawn helper (`bottom_panel_node`) | mod.rs:~4480-4500 (bottom_panel_node + spawn entry) |
  | `client/src/ui/shop_auction/draft_initial.rs` | `DraftInitial` centred modal: `draft_initial_centering_root_node`, `draft_initial_modal_panel_node`, `draft_initial_slot_node`, `draft_initial_status_node`; all systems that mutate the DraftInitial overlay's visibility / lifecycle | mod.rs:~4493-4655 |
  | `client/src/ui/shop_auction/shop.rs` | `Shop` panel: `shop_slot_node`, `shop_refresh_button_node`, `shop_ready_button_node`, `shop_ready_status_node`, `shop_hand_full_banner_node`, `shop_footer_slot_node`; all systems that mutate the Shop panel's offer rows, refresh/ready interactions, hand-full banner toggle | mod.rs:~4640-4960 |
  | `client/src/ui/shop_auction/auction.rs` | `Auction` panel: `auction_panel_node`, `auction_status_text_node`, `auction_timer_bar_node`, `auction_bid_status_text_node`, `auction_free_gold_counter_group_node`, `auction_bid_button_node`, `auction_featured_card_*` (including `auction_featured_card_accent_color` / `_leading_color` / `_losing_color` until Phase B.1 lifts them to `colors.rs`); all bid-handling systems | mod.rs:~4700-5170 |
  | `client/src/ui/shop_auction/settlement.rs` | Settlement overlay: `settlement_overlay_text_node`, the `overlay_node` scrim root, the transition tween system that drives settlement alpha 0.72 → 1.00, the post-settlement dismissal handler | mod.rs:~3565-3620, ~4975-5010 |
  | `client/src/ui/shop_auction/toasts.rs` | `toast_node`, `ToastQueue` drain system, toast spawn/despawn helpers, toast text colour helpers | mod.rs:~3605-3625, ~4950-4975 |
  | `client/src/ui/shop_auction/footer.rs` (OPTIONAL) | `footer_node`, `shop_footer_slot_node` if not co-located with `shop.rs` | mod.rs:~4930-4950 |

  The `/dev-story` worker MAY collapse the optional `footer.rs` into
  `shop.rs` if the footer slots are owned by the shop surface in
  practice (current footer slots render shop-purchase echoes); the
  authored shape MUST keep `mod.rs` under ~400 lines (aggregator only,
  no surface logic).

#### Public-API contract preservation

- Every public item currently exported by `client/src/ui/shop_auction/mod.rs`
  (every `pub fn`, `pub struct`, `pub enum`, `pub const`) MUST remain
  importable via the same `client::ui::shop_auction::*` path. The
  `mod.rs` aggregator re-exports each item from its new submodule
  (`pub use draft_initial::draft_initial_slot_node;`, etc.). No
  consumer outside `client/src/ui/shop_auction/` is required to update
  its import paths.

- The current `pub fn shop_auction::auction_featured_card_accent_color()`,
  `_leading_color()`, `_losing_color()`, and `_lead_loss_color()` helpers
  (consumed by `client/src/ui/hand/drag_state_visuals.rs` and by the
  `shop_auction_ui::auction_featured_card_lead_loss_test` test bin)
  MUST remain importable via the same path. Lifting them into
  `design_tokens/colors.rs` is a **separate Phase B.1 story**
  (`S16-TD-UI-COLORS-TOKEN-001`, NOT authored by this story); the
  current surface-owned form is preserved by this split.

- The `ShopAuctionPlugin` registration in `client/src/main.rs` (or
  wherever the plugin is added to the App) MUST continue to work
  without change. The plugin definition moves into the new aggregator
  `mod.rs` (or stays there if already there); system registration
  order is preserved byte-for-byte against `origin/main` at split
  time.

#### Existing test-bin import preservation

- Every test bin under `tests/integration/shop_auction_ui/` and
  `tests/integration/ui_clean_pass/` that imports a symbol from
  `client::ui::shop_auction::*` MUST continue to compile and pass
  without import changes. The split is **import-transparent**.

- Specifically, the following test bins (PROMPT 1035 §"Test coverage
  gaps" enumerates them) MUST pass byte-for-byte the same assertions
  after the split:
  - `tests/integration/shop_auction_ui/auction_featured_card_layout_test.rs`
  - `tests/integration/shop_auction_ui/auction_featured_card_lead_loss_test.rs`
  - `tests/integration/shop_auction_ui/draft_initial_centered_modal_layout_test.rs`
  - `tests/integration/ui_clean_pass/overlay_alpha_test.rs` (grep guards
    against old 0.45 / 0.46 / 0.58 literals continue to pass; new file
    locations included in grep paths)
  - Every other test bin that currently `use`s a `shop_auction` symbol.

#### `client/Cargo.toml` test-bin registration

- If `client/Cargo.toml` registers any `[[test]]` bin for the
  `shop_auction` integration tests, those registrations MUST NOT change.
  No new `[[test]]` bin is authored by this story (this is a source-tree
  split, not a test-bin add).

#### Anti-regression checks

- `cargo check -p client` passes against the split tree.
- `cargo test -p client --test shop_auction_*` (every bin in the
  `shop_auction_ui` integration directory) passes byte-for-byte the
  same assertions.
- `cargo test -p client --test ui_clean_pass_*` passes byte-for-byte
  the same assertions (including the `overlay_alpha_test.rs` grep
  guard).
- A QA snapshot bundle captured at 1280 × 720 and 1920 × 1080 across
  DraftInitial, Shop, Auction, and Settlement phases matches the
  pre-split snapshot bundle pixel-for-pixel (or within the existing
  QA-snapshot fuzz tolerance; exact tolerance is the existing
  `S15-QA-SNAPSHOT-DEFAULT-DEV` harness's default).

### Out of Scope

- **No public-API renames.** Every `pub` item keeps its exact current
  name and current path (via `mod.rs` re-export). API cleanup is a
  separate follow-on story.
- **No new design tokens.** No `colors.rs`, no `panel.rs`, no
  `pill.rs`, no `card_slot.rs` are authored or migrated by this split.
  Those are Phase B stories.
- **No layout tweaks.** No `Val::Px(...)` literal is changed. No
  anchor moves. No flex re-author. The auction `Val::Percent(34.0 + …)`
  pattern is preserved verbatim (its replacement is Phase C.4 story
  `S16-UI-AUCTION-FLEX-PRIMITIVES-001` per PROMPT 1035 §"Phase C").
- **No bid-button interaction-state migration.** Bid-button colour
  matchers stay inline; migration to `interaction_states::*` is
  separate Phase C.8 family `S16-UI-INTERACTION-STATE-MIGRATION-*`.
- **No settlement-scrim alpha audit.** The `OVERLAY_SCRIM_ALPHA` follow-on
  test that PROMPT 1035 §"Not migrated / debt" names is a separate
  Phase D test-discipline story.
- **No card-slot migration.** Story 009 `S12-TD-UI-CARD-SLOT-PRIMITIVE-001`
  (Phase B.2) is the canonical card-slot consumer; shop slot phase-1
  migration happens under that story or its Phase C.2 follow-on, NOT
  here.
- **No modal/panel/button primitive authoring.** Stories 012
  (`S16-TD-UI-MODAL-PRIMITIVE-001`), 013
  (`S16-TD-UI-BUTTON-PRIMITIVE-001`), and 014
  (`S16-TD-UI-PANEL-PRIMITIVE-001`) author those primitives separately.
- **No hand module split.** Story 011 `S16-TD-UI-HAND-MODSPLIT-001` is
  the parallel-safe companion split; it is **not** in this story's
  scope.
- **No `Polish->Release` gate-check retry.** PROMPT 761 FAIL preserved.
- **No stage advance.** `production/stage.txt` remains `Polish`.
- **No Sprint 14 / Sprint 15 / Sprint 16 row reopen.** All closed rows
  remain Done unchanged.

---

## Acceptance Criteria

All criteria are independently checkable BLOCKING criteria. The future
`/dev-story` worker MUST satisfy each before `/story-done` runs.

- [ ] **AC1 -- Submodule shape**: GIVEN the worker commit, WHEN the
  source tree under `client/src/ui/shop_auction/` is inspected, THEN
  the directory contains AT LEAST the following files: `mod.rs`,
  `state.rs`, `spawn.rs`, `draft_initial.rs`, `shop.rs`, `auction.rs`,
  `settlement.rs`, `toasts.rs`. Optional: `footer.rs` if footer slots
  are extracted from `shop.rs`. The `mod.rs` aggregator is under **400
  lines** (down from 5 435). No submodule exceeds **2 000 lines**.
  Verification: file presence + `wc -l`.

- [ ] **AC2 -- Re-export aggregator**: GIVEN the new `mod.rs`, WHEN
  inspected, THEN it consists only of (a) `mod state;` / `mod
  draft_initial;` / etc. submodule declarations, (b) `pub use`
  re-exports for every previously-public item, (c) the
  `ShopAuctionPlugin` definition + system registration (or a `mod
  plugin;` declaration if the plugin is split into its own submodule),
  and (d) top-of-file `use` imports needed for the plugin definition.
  No surface-specific `fn` body lives in `mod.rs`. Verification:
  per-line grep + line count + re-export presence check against the
  pre-split public API surface.

- [ ] **AC3 -- Import-transparent public API**: GIVEN the split tree,
  WHEN every existing consumer of `client::ui::shop_auction::*` is
  compiled, THEN no consumer outside `client/src/ui/shop_auction/`
  changes its `use` path. Verification: `git diff origin/main...HEAD
  -- 'client/src/' 'tests/' 'server/' 'shared/' | grep '^[+-]use '`
  shows zero diff lines outside `client/src/ui/shop_auction/**`.

- [ ] **AC4 -- Cross-module dependency preservation**: GIVEN the split
  tree, WHEN `client/src/ui/hand/drag_state_visuals.rs` is inspected,
  THEN its imports of
  `shop_auction::auction_featured_card_accent_color` (or whichever
  helpers it currently imports) continue to resolve via the
  `mod.rs` re-export. **The wrong-direction-of-dependency smell that
  PROMPT 1035 §"Hand (`client/src/ui/hand/mod.rs`) -- Not migrated /
  debt" calls out is PRESERVED, NOT FIXED, by this split.** Lifting
  those colour helpers to `design_tokens/colors.rs` is the separate
  Phase B.1 story. Verification: import path read + test compile.

- [ ] **AC5 -- Behaviour preservation (cargo + tests)**: GIVEN the
  split tree, WHEN the test harness runs, THEN:
  - `cargo check -p client` succeeds with zero new warnings (existing
    pre-split warnings are preserved verbatim; no new dead-code,
    unused-import, or unused-mut warning is introduced).
  - `cargo test -p client --test 'shop_auction_*'` passes byte-for-byte
    the same assertions as on `origin/main` pre-split.
  - `cargo test -p client --test 'ui_clean_pass_*'` passes byte-for-byte
    the same assertions, including the `overlay_alpha_test.rs` grep
    guard. The grep guard's path-filter list MUST be updated if the
    file move changes line numbers but MUST NOT introduce new
    forbidden-literal paths.
  - `cargo test -p client --test 'hand_ui_*'` passes (no hand changes,
    but `drag_state_visuals.rs` re-imports `shop_auction` symbols).
  - `cargo test -p client --test 'hud_*'` passes (HUD does not import
    `shop_auction` symbols today; this is a regression guard).
  Verification: test logs in evidence directory.

- [ ] **AC6 -- QA snapshot pixel parity**: GIVEN a QA snapshot bundle
  captured pre-split and post-split at 1280 × 720 and 1920 × 1080 across
  the DraftInitial / Shop / Auction / Settlement phases (per the
  `S15-QA-SNAPSHOT-DEFAULT-DEV` flow per PROMPT 1021 / 1023), THEN the
  PNG outputs are pixel-identical OR within the QA-snapshot harness's
  existing fuzz tolerance. The evidence directory at
  `production/qa/evidence/sprint-1X-shopauction-modsplit/` contains
  pre-split and post-split snapshot PNGs side-by-side with a diff
  summary. Verification: snapshot directory contents + diff log.

- [ ] **AC7 -- No public-API rename, no behaviour change, no token
  introduction**: GIVEN the worker diff, WHEN inspected, THEN:
  - No `pub fn`, `pub struct`, `pub enum`, or `pub const` is renamed.
  - No `Val::Px(...)`, `Val::Percent(...)`, `Color::srgb*(...)` literal
    is changed.
  - No new file under `client/src/ui/design_tokens/` is authored.
  - No `interaction_states::*` import is added or removed.
  - No system registration order is changed (`add_systems(...)` calls
    appear in the same order against `origin/main`).
  - No `Resource`, `Component`, `Event`, or `Message` derive is added
    or removed.
  Verification: `git diff origin/main...HEAD -- 'client/src/ui/shop_auction/'`
  is a pure file-move + re-export diff (no semantic change visible per
  `git diff -M --diff-filter=R`).

- [ ] **AC8 -- Non-claims (no gameplay / no server / no release)**:
  GIVEN the worker commit, WHEN the closure paperwork is inspected,
  THEN this row does NOT:
  - Modify any gameplay logic.
  - Modify any server / shared / protocol module.
  - Modify `Cargo.toml` test-bin registrations (except the trivial
    case if the worker adds `[[test]]` entries that already mirror
    existing ones).
  - Claim public release readiness, release-candidate readiness, full
    game completion, full playable-client manual QA, playtest /
    fun-hypothesis validation (`QA-COND-0006`), Standard-tier
    accessibility (`QA-COND-0005`), final-art / asset-production
    completion (`PAW-TD-*-a`), two-client GAME_OVER closure
    (`S8-QA-001-W1`), the `Polish->Release` gate-check retry, or
    stage advance from `Polish` to `Release`.
  - Reopen any Sprint 14 / Sprint 15 / Sprint 16 closed row.
  Verification: `git diff origin/main...HEAD --stat -- 'server/' 'shared/'`
  is empty; paperwork review of the `/story-done` close-out section
  confirms the non-claims.

---

## Implementation Notes

### Owned files (the future `/dev-story` worker authors these)

| Path | Expected change |
|------|-----------------|
| `client/src/ui/shop_auction/mod.rs` | Reduce from 5 435 lines to ~400-line aggregator (module declarations + `pub use` re-exports + `ShopAuctionPlugin` definition). |
| `client/src/ui/shop_auction/state.rs` (NEW) | Move all `Component` / `Resource` / `Event` / `Message` declarations + shared layout constants. |
| `client/src/ui/shop_auction/spawn.rs` (NEW) | Move `spawn_shop_auction_ui` + `bottom_panel_node` + shared scaffolding. |
| `client/src/ui/shop_auction/draft_initial.rs` (NEW) | Move DraftInitial centred-modal surface code. |
| `client/src/ui/shop_auction/shop.rs` (NEW) | Move Shop panel surface code. |
| `client/src/ui/shop_auction/auction.rs` (NEW) | Move Auction panel surface code (incl. `auction_featured_card_*_color` helpers). |
| `client/src/ui/shop_auction/settlement.rs` (NEW) | Move Settlement overlay surface code. |
| `client/src/ui/shop_auction/toasts.rs` (NEW) | Move Toast queue + spawn helpers. |
| `client/src/ui/shop_auction/footer.rs` (NEW, OPTIONAL) | Move footer slots if extracted separately from `shop.rs`. |
| `production/qa/evidence/sprint-1X-shopauction-modsplit/` (NEW) | Evidence dir: pre/post QA snapshot bundles, cargo logs, doc-review checklist, `git diff --stat` proving import-transparent change set. |

### Forbidden files (the future `/dev-story` worker MUST NOT touch these)

- `client/src/ui/hand/**` -- hand module split is story 011, not this row.
- `client/src/ui/hud/**`, `client/src/ui/lobby.rs`,
  `client/src/ui/settings/**`, `client/src/ui/photosensitivity_warning.rs`,
  `client/src/ui/design_tokens/**` -- not part of this split.
- `client/src/presentation/**` -- not part of this split.
- `client/src/gameplay/**`, `server/src/**`, `shared/src/**` -- UNCHANGED.
- `tests/integration/**` -- existing tests are UNCHANGED in source; only
  their pass/fail status is verified.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md` --
  shared-state writers, never touched by `/dev-story`.
- `docs/**`, `.claude/**`, `AGENTS.md`, `CLAUDE.md` -- not part of this
  split.
- `Cargo.toml` (workspace and `client/`), `Cargo.lock`, `.cargo/`,
  `Trunk.toml` -- only the `client/Cargo.toml` `[[test]]` registration
  table may be touched if a new test bin is added (none expected; AC1
  + AC7 forbid net-new bins).

### Module integration touch points

- `client/src/main.rs` (or wherever `ShopAuctionPlugin` is added) MUST
  NOT change. The plugin's `add_systems(...)` body lives in `mod.rs`
  (or in a `plugin.rs` submodule if the worker prefers); either way
  the `App::add_plugins(ShopAuctionPlugin)` call site is unchanged.
- `client/src/ui/hand/drag_state_visuals.rs` (Sprint 15 story
  `S12-UX-HAND-DRAG-STATE-VISUALS-001` DONE) imports
  `shop_auction::auction_featured_card_*_color` helpers. The
  `mod.rs` re-export preserves the path; no change to
  `drag_state_visuals.rs` is required.
- `tests/integration/shop_auction_ui/**` imports
  `shop_auction::*` symbols heavily (per `auction_featured_card_layout_test.rs`,
  `auction_featured_card_lead_loss_test.rs`,
  `draft_initial_centered_modal_layout_test.rs`). Re-export
  preservation is the AC3 requirement.

---

## Parallelization and Phase Breakdown

### Parallel-safety with sibling stories

| Sibling story | File scope | Parallel-safe with this row? |
|---|---|---|
| **Story 011 `S16-TD-UI-HAND-MODSPLIT-001`** | `client/src/ui/hand/mod.rs` only | **YES** -- file-disjoint. Two workers can land both splits in parallel; integration prompt rebases them serially against `origin/main`. |
| **Story 012 `S16-TD-UI-MODAL-PRIMITIVE-001`** | `client/src/ui/design_tokens/modal.rs` NEW (or `client/src/ui/primitives/modal.rs` NEW) + a single canonical migration site | Primitive authoring: **YES**, file-disjoint (different directory). Migration site choice: **conflict if the primitive picks the DraftInitial modal as its phase-1 demo before this split lands**; producer schedules the primitive AFTER this split, OR picks a non-shop_auction modal (e.g. result-screen or connection-lost) as the phase-1 demo. |
| **Story 013 `S16-TD-UI-BUTTON-PRIMITIVE-001`** | `client/src/ui/design_tokens/button.rs` NEW (or `client/src/ui/primitives/button.rs` NEW) + canonical migration | Same as story 012 -- primitive authoring file-disjoint; per-surface migration of any shop / auction / draft button MUST land AFTER this split. |
| **Story 014 `S16-TD-UI-PANEL-PRIMITIVE-001`** | `client/src/ui/design_tokens/panel.rs` NEW + canonical migration | Same as story 012 / 013. |
| **Story 015 `S16-TD-UI-ARCHITECTURE-SEQUENCING-001`** | `production/epics/ui-clean-pass/` doc-only sequencing map | **YES** -- doc-only, no source touches. |
| **Story 009 `S12-TD-UI-CARD-SLOT-PRIMITIVE-001`** (already authored; Sprint 16 candidate, NOT activated) | Primitive module + spec amendment + shop slot phase-1 migration | **Conflict if Phase 1 migration lands BEFORE this split** -- the migration touches `shop_auction/mod.rs::shop_slot_node` at the old line offset. Producer SHOULD schedule this split BEFORE story 009 phase-1 migration. Alternatively, story 009's phase-1 demo MAY be deferred to a follow-on row `S16-UI-CARD-SLOT-MIGRATION-SHOP-001` that runs AFTER this split. |

### Files that MUST serialize

- `client/src/ui/shop_auction/mod.rs` is owned exclusively by this row
  during the split. Any other Sprint 16+ row that touches a shop /
  auction / draft / settlement / toast / footer surface MUST wait for
  this split to reach `origin/main` first.
- `production/sprint-status.yaml` (orchestrator-serialized as always).

### Dependencies and unblockers

- **No prerequisite stories.** This split is the foundation row of
  Phase A; it depends only on `origin/main` HEAD at activation time.
- **Unblocks** (per PROMPT 1035 §"Phase C"):
  - Story 009 phase-1 shop slot migration (or a follow-on Phase C.2
    family row that subsumes it).
  - Story 012 modal primitive's DraftInitial migration phase.
  - Story 013 button primitive's bid-button / refresh-button /
    ready-button migration phases.
  - Story 014 panel primitive's draft-initial / settlement-panel
    migrations.
  - Phase C.4 `S16-UI-AUCTION-FLEX-PRIMITIVES-001` (auction
    `Val::Percent(34.0)` anchor replacement).
  - Phase C.5 `S16-UI-SHOP-CONTROL-ROW-001` (shop control row flex
    primitive).
  - Phase C.6 `S16-UI-MODAL-PANEL-CONSOLIDATION-001` (result /
    connection-lost / photosensitivity / lobby / draft / settlement
    panel consolidation).
  - Phase C.7 palette sweep (`S16-UI-COLORS-MIGRATION-001`).

---

## Dependency Map (for this story only)

| Direction | Dep | Reason |
|---|---|---|
| **Prerequisite** | `origin/main` HEAD at activation | Source tree must be clean; integration rebases are the orchestrator's job. |
| **Prerequisite** | None within ui-clean-pass epic | This is the foundation row of Phase A. |
| **Unblocks** | Story 009 phase-1 shop slot migration | shop slot call site moves to `shop.rs` after split. |
| **Unblocks** | Story 012 modal primitive (DraftInitial migration) | DraftInitial moves to `draft_initial.rs` after split. |
| **Unblocks** | Story 013 button primitive (shop / auction button migration) | Bid buttons + shop refresh/ready buttons move to `auction.rs` / `shop.rs` after split. |
| **Unblocks** | Story 014 panel primitive (draft / settlement panel migration) | Both panels move to dedicated submodules after split. |
| **Parallel-safe** | Story 011 hand modsplit | File-disjoint. |
| **Parallel-safe** | Story 012 / 013 / 014 PRIMITIVE AUTHORING (not migration) | Different directory (`design_tokens/` or `primitives/`). |
| **Conflicts** | Any Phase C migration touching shop / auction / draft / settlement / toasts / footer before this split lands | Same file. Producer serializes. |

---

## Worker Contract (for `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout -b work/s16-shop-auction-modsplit` from `origin/main` HEAD.
2. Read `client/src/ui/shop_auction/mod.rs` in full and inventory:
   - Every `pub` item (count + names).
   - Every system registered against `ShopAuctionPlugin`.
   - Every cross-surface helper (colour functions, formatters,
     constant tables).
3. Author the submodule split per AC1's table.
4. Verify AC2-AC8 locally:
   - `cargo check -p client`
   - `cargo test -p client --test 'shop_auction_*'`
   - `cargo test -p client --test 'ui_clean_pass_*'`
   - `cargo test -p client --test 'hand_ui_*'`
   - `cargo test -p client --test 'hud_*'`
5. Capture QA snapshot bundles per AC6.
6. Push `work/s16-shop-auction-modsplit`. Do NOT push `main`.
7. Hand off to the integration prompt for `origin/main` merge +
   `/story-done`.

The worker MUST NOT:

- Rename any `pub` item.
- Change any layout literal.
- Touch any forbidden file per the Forbidden files table.
- Run `/story-done` / `/smoke-check` / `/team-qa` / `/gate-check` /
  `/release-check` / `/qa-plan`.
- Modify `production/sprint-status.yaml`,
  `production/sprints/sprint-XX.md`, `production/stage.txt`, or
  `production/session-state/*`.

---

`010: S16-TD-UI-SHOPAUCTION-MODSPLIT-001: DRAFT`
