# Sprint 12 -- Fixture D Residuals (Cluster B1 + B5 Umbrella) -- Evidence

> **Story**: `S11-TD-FIXTURE-D-RESIDUALS-001`
> (`production/epics/playable-client/story-015-fixture-d-residuals.md`)
> **Implementation prompt**: PROMPT 812 -- /dev-story
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s11-fixture-d-residuals`
> **Branch**: `work/s11-fixture-d-residuals`
> **Base (origin/main)**: `a8ef42d` (PROMPT 807 Sprint 13 candidate mapping)
> **Authored**: 2026-05-14
> **Sprint**: Sprint 12 (active per PROMPT 798)
> **Stage**: Polish (unchanged)

---

## No-Claim Banner (verbatim from story 015)

This story is authored as a Sprint 12 draft Must Have umbrella. Sprint 12
activation is preserved. PROMPT 812 (this implementation run) does NOT
modify `production/sprint-status.yaml`, `production/sprints/sprint-12.md`,
`production/stage.txt`, or any session-state file. PROMPT 812 does NOT
retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

**No optimistic client-side authority is introduced or proposed by this
story or by either chosen sub-disposition (B1.a or B5.a).** ADR-002
binding preserved. Both sub-dispositions are test-only changes -- zero
production-code change under either.

---

## Producer Decision -- Umbrella (CHOSEN)

**Umbrella chosen.** Rationale (from story 015 "Producer Decision"
section, recorded BEFORE any code change):

- Both B1 and B5 are test-only or near-test-only changes (B1 is a
  fixture event-firing repair; B5 is a one-line formula multiplier
  update with zero production-code touch).
- Combined diff is small (~5 net loc across two test files plus this
  story file).
- Both share the "decision-before-code" discipline and the same
  Sprint 11 D-5 triage origin (Cluster B residuals).
- Batching reduces re-review cost and lets one evidence document
  (this file) cover both un-`#[ignore]` actions.
- Umbrella matches how Sprint 11 close-out paperwork and Sprint 12
  draft already folded the row.

**Split path NOT chosen.** Authoring two follow-on story files would
add paperwork without proportional review-surface reduction given the
test-only scope of both chosen sub-dispositions.

---

## Per-Sub-Disposition Decisions (CHOSEN)

### B1 -- Path B1.a (Expand fixture / event-firing repair) -- CHOSEN

**Rationale** (from story 015 "Per-Sub-Disposition Decisions" section,
recorded BEFORE code change):

Investigation at PROMPT 812 time corrected the PROMPT 750 D-5 owner
comment. The `GhostDragStartEvent` producer is NOT in `HandUiPlugin`
as the original ignore-comment supposed. It is an observer on
`BoardRenderingPlugin`:

- `add_observer(on_ghost_drag_start)` at
  `client/src/presentation/board_rendering.rs:893`.
- Matching observer `add_observer(on_ghost_clicked)` at line 892.
- Observer function bodies: `on_ghost_drag_start` at line 1113;
  `on_ghost_clicked` at line 1095.

The fixture failure is therefore not "missing plugin." It is that
the test was driving the observer with `world.write_message(Pointer<E>)`,
which only enqueues the message but does NOT fire the observer.

In real gameplay, `bevy_picking::DefaultPickingPlugins` calls
`commands.trigger_targets(...)` to fire those observers; under
`MinimalPlugins` the test must do the same itself.

**Fix**: drive `Pointer<Press>` and `Pointer<Click>` via
`world.trigger(event)` in the test body. The observer then fires,
writes `GhostDragStartEvent` and `GhostClickedEvent` into the message
buffers, and the existing assertions drain them as before.

- Zero production-code change.
- No `HandUiPlugin` registration added.
- The PROMPT 750 D-5 ignore-comment is removed (precondition: the
  test now passes without `#[ignore]`).
- ADR-002 preserved (test-only fix; no authority shift).

**Path B1.b NOT chosen** because the producer lives on
`BoardRenderingPlugin`, not Hand UI -- relocation would be
semantically wrong and would lose cross-plugin verification.

### B5 -- Path B5.a (Update formula 57 -> 66) -- CHOSEN

**Rationale** (from story 015 "Per-Sub-Disposition Decisions" section,
recorded BEFORE code change):

Investigation at PROMPT 812 time established the 9-entity delta is
intentional capacity, not over-production.
`spawn_draft_initial_grid` at
`client/src/ui/shop_auction/mod.rs:3654-3720` spawns **three**
`ShopAuctionUiEntity`-tagged entities per draft slot:

1. The slot container (line 3666-3677).
2. A dedicated text-child entity that holds the card name + cost
   text (line 3680-3690).
3. The "BOUGHT" bought-overlay (line 3697-3711).

The slot-text child is deliberate -- the inline comment at lines
3663-3664 reads:

> Spawn the slot container WITHOUT Text so it doesn't render as a
> white dot when no text is set. Text lives in a dedicated child
> entity instead.

Trimming the text-child back out (Path B5.b) would re-introduce the
white-dot rendering regression that the split text-child fix
prevents.

The formula is the stale artefact: the multiplier
`SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT * 2` was written when only
`slot + overlay` were tagged, and was not updated when the
`text_entity` was added.

**Fix**: change the multiplier from `* 2` to `* 3` in the assertion
at
`tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs`,
raising the expected total from 57 to 66.

- Zero production-code change. The formula lives in the test file,
  not in `client/src/`.
- The PROMPT 750 D-5 ignore-comment is removed (precondition: the
  test now passes without `#[ignore]`).
- ADR-002 preserved (test-only fix).

**Path B5.b NOT chosen** because the 9-entity "over-production" is
not over-production -- it is the deliberate text-child fix.

---

## Code-Change Diff Summary

Two test files modified; zero production-code under `client/src/`,
`server/src/`, or `shared/src/` modified.

```
tests/integration/board_rendering/ghost_preview_bridge_test.rs |  15 ++++++++++++---
tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs    |  12 ++++++++++--
2 files changed, 22 insertions(+), 5 deletions(-)
```

### `tests/integration/board_rendering/ghost_preview_bridge_test.rs`

- Removed `#[ignore = "PROMPT 750 D-5 follow-on: ..."]` from
  `br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui`.
- Replaced `app.world_mut().write_message(pointer_press(...))` and
  `app.world_mut().write_message(pointer_click(...))` with
  `app.world_mut().trigger(press)` and
  `app.world_mut().trigger(click)`.
- Added explanatory test-comment referencing PROMPT 812 B1.a, the
  observer registration site, and the
  `bevy_picking::DefaultPickingPlugins` parallel.

### `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs`

- Removed `#[ignore = "PROMPT 750 D-5: ShopAuctionUiEntity count drift -- ..."]`
  from `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes`.
- Changed the formula multiplier in the expected-count assertion
  from `SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT * 2` to
  `SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT * 3`.
- Added explanatory test-comment referencing PROMPT 812 B5.a, the
  `spawn_draft_initial_grid` site, the deliberate text-child
  pattern, and the white-dot rendering bug inline comment that the
  text-child fix prevents.

### Production-code scope check (AC7)

`git diff origin/main...HEAD -- 'client/src/**' 'server/src/**' 'shared/src/**'`
returns an empty diff. Zero production-code change confirmed.

---

## Verification Results

### Targeted B1 test

```
$ cargo test -p client --test board_rendering_ghost_preview_bridge_test --no-fail-fast
running 4 tests
test br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui ... ok
test br_8_board_cell_ghost_replaces_existing_card_preview ... ok
test br_10_clear_none_removes_matching_card_ghosts_without_spawn_range_edits ... ok
test br_8_variant_matrix_marks_or_spawns_expected_board_ghosts ... ok
test result: ok. 4 passed; 0 failed; 0 ignored
```

### Targeted B5 test

```
$ cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test --no-fail-fast
running 8 tests
test shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes ... ok
test result: ok. 8 passed; 0 failed; 0 ignored
```

### Full `cargo test -p client --no-fail-fast`

| Bucket | Pre-PROMPT 812 (PROMPT 810 baseline) | Post-PROMPT 812 |
|--------|--------------------------------------|-----------------|
| passed | 394 | **396** |
| failed | 0 | 0 |
| ignored | 2 (B1 + B5) | **0** |

Ignored count drops from 2 to 0 (umbrella closes both B1 and B5
under AC5). No new undocumented `#[ignore]` markers introduced.

### `cargo test --workspace --tests --no-fail-fast`

| Bucket | Pre-PROMPT 812 (post-PROMPT 810 baseline) | Post-PROMPT 812 |
|--------|-------------------------------------------|-----------------|
| passed | 1133 | **1135** |
| failed | 0 | 0 |
| ignored | 2 (B1 + B5) | **0** |
| suites (binaries) | 191 | 191 |

Workspace ignored count drops 2 -> 0. No new undocumented
`#[ignore]` introduced. (Sprint 11 close-out baseline was 5 ignored;
PROMPT 805 closed B3 + B4, PROMPT 806/809/810 closed B2, leaving the
two umbrella-scoped tests that PROMPT 812 closes here.)

### `cargo fmt -p client -- --check`

Clean (no output).

### `git diff --check origin/main...HEAD`

Clean (no whitespace or conflict markers).

---

## ADR / GDD / TR Trace

- **ADR-002 (Client-Server Authority)**: preserved. No client-side
  optimistic mutation introduced. Both sub-dispositions are
  test-only fixes that do not touch authoritative state.
- **ADR-021 (Presentation Layer Architecture)**: preserved. B1.a
  driver change matches the architecture (observer-based
  ghost-drag producer on `BoardRenderingPlugin`).
- **GDD trace**: B1 covered by `design/gdd/board-rendering.md`
  (TR-BR-002 board layout) and `design/gdd/hand-ui.md` (ghost
  preview producer). B5 covered by
  `design/gdd/shop-auction-ui.md` prepool count contract.
  Neither sub-disposition modifies the GDDs.
- **TR trace**: no new TR added; no existing TR modified.

---

## Cross-Links

- Story file: `production/epics/playable-client/story-015-fixture-d-residuals.md`
- Sprint 11 D-5 triage: `production/qa/evidence/sprint-11-ignored-d5-triage.md`
  - Cluster B1 row 83 (board-rendering test-infra owner)
  - Cluster B5 row 87 (shop-auction-ui scaffold owner)
- Sprint 12 plan row: `production/sprints/sprint-12.md` (Sprint 12
  Must Have row `S11-TD-FIXTURE-D-RESIDUALS-001`)
- Sprint 12 QA plan: `production/qa/qa-plan-sprint-12.md` (story
  015 entry)
- Precedent for decision-first discipline: Cluster B4 (Story 014 --
  Co-occupancy Panic Guard).
- Sibling fixture work: Cluster B2 (Story 012 -- HUD snapshot phase
  bridge) -- disjoint files.
- Bevy 0.18 driver pattern referenced: `world.trigger(event)` for
  Observer-based event delivery (vs `write_message` for buffered
  `MessageReader` consumers). See skill `liv-bevy-018`.

---

## What PROMPT 812 explicitly did NOT do

- Did NOT modify `production/sprint-status.yaml`.
- Did NOT modify `production/sprints/sprint-12.md`.
- Did NOT modify `production/stage.txt`.
- Did NOT modify any session-state file.
- Did NOT run `/story-readiness`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, or `/release-check`.
- Did NOT close Sprint 12.
- Did NOT retry the PROMPT 761 Polish->Release gate-check.
- Did NOT change `S8-QA-001-W1` disposition.
- Did NOT change `QA-COND-0005` or `QA-COND-0006` disposition.
- Did NOT modify any production code under `client/src/`,
  `server/src/`, or `shared/src/`.
- Did NOT introduce any client-side optimistic state mutation
  (ADR-002 binding preserved).

---

## Status

PASS -- umbrella closure of Cluster B1 + B5 fixture residuals
landed under the decision-first discipline. Both targeted tests
un-`#[ignore]`d and passing. Workspace ignored-count delta = -2.
Zero production-code change.
