# Story 015: Fixture D Residuals -- Cluster B1 + B5 Umbrella (Umbrella vs Split Decision)

> **Epic**: Playable Client
> **Story ID**: S11-TD-FIXTURE-D-RESIDUALS-001
> **Status**: Draft -- Sprint 12 draft Must Have (Cluster B1 + B5 umbrella);
> NOT activated
> **Layer**: Tech Debt / Test Fixtures + Scaffold Decision (umbrella OR split)
> **Type**: Decision-first (umbrella-vs-split producer decision) + fixture
> cleanup + scaffold decision
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 12 (draft per PROMPT 793 at `origin/main@8a8451e`; NOT yet activated)
> **Authored**: 2026-05-14 by PROMPT 795 (producer + qa-lead, worktree `work/sprint-12-must-story-authoring`)
> **Authoring source-of-truth**: `origin/main@f72cc60` (PROMPT 793 Sprint 12 draft plan + PROMPT 794 story-019 slug correction).

---

## Status / No-Claim Banner

This story is authored as a Sprint 12 draft Must Have umbrella. Sprint 12
is **NOT activated**; activation happens via `/sprint-plan sprint-12` in a
separate prompt. PROMPT 795 (this authoring run) does NOT:

- Activate Sprint 12.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-12.md`.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any session-state file.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan`.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify the ignored tests
  (`tests/integration/board_rendering/ghost_preview_bridge_test.rs:147`
  and
  `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs:25`).
- Make the umbrella-vs-split producer decision (that decision is
  recorded by the implementation prompt or by a follow-on producer
  prompt; see "Producer Decision" below).
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 disposition (`closed-with-conditions`) and Sprint 11 disposition
(`closed-with-conditions` per PROMPT 792) remain unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence at
`production/gate-checks/gate-polish-release-2026-05-12.md` is preserved.

**No optimistic client-side authority is introduced or proposed by this
story or by any disposition pathway recorded in "Acceptance Criteria"**.
ADR-002 binding for any production code change that may land under
the "trim spawn" B5 sub-disposition.

---

## Context

Sprint 11 D-5 triage evidence
(`production/qa/evidence/sprint-11-ignored-d5-triage.md`) retained two
distinct-root-cause `#[ignore]` tests under separate Cluster B rows that
the Sprint 12 draft folds into a single umbrella row (with a producer
option to split). The two tests are:

### B1 (`tests/integration/board_rendering/ghost_preview_bridge_test.rs:147`)

> `#[ignore = "PROMPT 750 D-5 follow-on: GhostDragStartEvent producer
> system not present in BoardRenderingPlugin-only fixture -- needs
> HandUiPlugin pointer-to-drag bridge or fixture expansion (revealed after
> D-3 picking events were registered)"]`

Test: `br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui`.
Root cause: the `BoardRenderingPlugin`-only fixture
(`app_with_board_rendering()`) does not register the `HandUiPlugin`
producer system that emits `GhostDragStartEvent`. The assertion expects
the ghost-drag event to be produced and consumed, but the producer side
is missing in the fixture.

Disposition shape: **fixture expansion OR assertion-scope reduction**.
- **Path B1.a -- Expand fixture**: register `HandUiPlugin`'s
  pointer-to-drag bridge in `app_with_board_rendering()` so the
  ghost-drag producer fires end-to-end.
- **Path B1.b -- Scope assertion to a `HandUiPlugin` fixture**: relocate
  the producer-side assertion to a Hand UI integration test where the
  producer is already wired; retain only the consumer-side
  (board-rendering) assertion in `ghost_preview_bridge_test.rs`.

### B5 (`tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs:25`)

> `#[ignore = "PROMPT 750 D-5: ShopAuctionUiEntity count drift -- actual=66,
> formula expects=57 (9 entity delta); needs scaffold owner to either
> update formula or trim spawn"]`

Test: `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes`.
Root cause: the `ShopAuctionUiEntities` count formula
(`shop_auction_ui_prepooled_entity_count(...)` or equivalent canonical
formula name -- verify at implementation time) returns 57; the actual
prepooled spawn produces 66. Either the formula is stale (caller's
expectation), or the spawn is over-producing (production-side).

Disposition shape: **update formula OR trim spawn**.
- **Path B5.a -- Update formula**: update the formula constants to
  reflect the actual 66-entity prepool; assertion is re-armed at 66.
  Rationale captured before code change.
- **Path B5.b -- Trim spawn**: identify the 9-entity over-production in
  the scaffold spawn site and trim to 57; assertion stays at 57.
  Rationale captured before code change.

**Both clusters share a "decision-before-code-change" discipline**
identical to Cluster B4 (Story 014 -- Co-occupancy Panic Guard). Each
sub-decision (B1.a/B1.b, B5.a/B5.b) is binary and gated on a written
rationale.

**Primary sources**:

- `production/qa/evidence/sprint-11-ignored-d5-triage.md` (Cluster B1
  row 83 and Cluster B5 row 87)
- `production/sprints/sprint-12.md` (Sprint 12 draft Must Have row
  `S11-TD-FIXTURE-D-RESIDUALS-001`, line 127, with the explicit
  umbrella-vs-split optionality language)
- `tests/integration/board_rendering/ghost_preview_bridge_test.rs:147`
  (B1 test, with PROMPT 750 D-5 owner comment preserved on
  `origin/main@f72cc60`)
- `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs:25`
  (B5 test, with PROMPT 750 D-5 owner comment preserved on
  `origin/main@f72cc60`)

**GDD, UX, and TR trace**:

- B1: `design/gdd/board-rendering.md` (TR-BR-002 board layout) and
  `design/gdd/hand-ui.md` (ghost preview producer) cover the
  end-to-end ghost-drag flow.
- B5: `design/gdd/shop-auction-ui.md` covers the prepool count
  contract. Neither sub-disposition modifies the GDD.
- No new TR is added by this story.

**ADR Governing Implementation**:

- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
  -- bridge architecture for ghost preview between Hand UI and Board
  Rendering plugins. Binding for B1 sub-dispositions.
- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
  -- no client-side optimism (binding for any B5 trim-spawn path that
  touches authoritative scaffold state).
- No protocol or networking ADR is touched.

**Engine**: Bevy 0.18 (Rust) | **Risk**: LOW-MEDIUM (B1 is test-fixture;
B5 may touch production scaffold spawn under Path B5.b)

**Engine Notes**: Bevy 0.18 plugin composition is additive -- Path B1.a
adds plugins to the fixture; Path B1.b moves assertion entries between
tests. Path B5.a updates a formula constant; Path B5.b trims a spawn
loop in the scaffold (production-side). Both B5 paths require a written
rationale before code change.

**Mandatory skills**:
- `liv-bevy-018` -- any read/review/edit of Bevy `.rs` code touched.

**Control Manifest Rules (2026-05-05)**:
- Required: Producer decision (umbrella vs split) recorded in this
  story file before any code change is staged.
- Required: Per-sub-disposition rationale (B1.a vs B1.b, B5.a vs B5.b)
  recorded in this story file before code change.
- Required: If split is chosen, two follow-on story files are authored
  (`S11-TD-FIXTURE-BOARD-GHOST-DRAG-PRODUCER-001` for B1 and
  `S11-TD-SHOP-AUCTION-UI-COUNT-DRIFT-001` for B5); this umbrella
  story closes as the producer-decision-record artefact.
- Forbidden: Silently dropping either B1 or B5 from disposition.
  Both must land closure under the chosen path.

---

## Story Classification

**Story type**: Composite -- producer decision (umbrella vs split) +
fixture cleanup (B1) + scaffold decision (B5). The story is
**decision-first**: the umbrella-vs-split decision and the per-sub-
disposition decisions are recorded before any code change lands.

This is **NOT** a:

- Pure evidence-only story (executable code changes land after
  decisions are recorded).
- Pure repair story (the disposition space includes test-only paths
  for both clusters).
- Pure fixture-cleanup story (B5 may land production scaffold change
  under Path B5.b).

---

## Producer Decision (umbrella vs split)

The implementation prompt (or a separate producer prompt) MUST record
exactly one of the following before any code change is staged.

- [x] **Umbrella (this story)**: keep B1 + B5 dispositions inside this
      story; close both clusters under
      `S11-TD-FIXTURE-D-RESIDUALS-001`. Rationale:
      Both B1 and B5 are test-only or near-test-only changes (B1 is a
      fixture event-firing repair; B5 is a one-line formula multiplier
      update with zero production-code touch). Combined diff is small
      (~5 lines across two test files plus this story file). Both
      share the "decision-before-code" discipline and the same Sprint
      11 D-5 triage origin (Cluster B residuals). Batching reduces
      re-review cost and lets one evidence document
      (`production/qa/evidence/sprint-12-fixture-d-residuals-evidence.md`)
      cover both un-`#[ignore]` actions. The umbrella also matches how
      the Sprint 11 close-out paperwork and Sprint 12 draft already
      folded the row.

- [ ] **Split into two stories**: NOT CHOSEN. Authoring two follow-on
      story files (`S11-TD-FIXTURE-BOARD-GHOST-DRAG-PRODUCER-001` for
      B1 and `S11-TD-SHOP-AUCTION-UI-COUNT-DRIFT-001` for B5) is
      unnecessary because the combined diff is small, both changes are
      test-only or near-test-only (zero production-code change under
      the chosen B1.a and B5.a sub-dispositions), and a single
      evidence document covers both. Split would add paperwork without
      proportional review-surface reduction.

The default producer recommendation (advisory only; not binding) is
**umbrella** because (a) both clusters were folded under
`S11-TD-FIXTURE-D-RESIDUALS-001` in the Sprint 11 close-out paperwork
and the Sprint 12 draft; (b) batched review surface is small; (c) the
decision-first discipline applies identically to both. The split path
is available if the producer prefers per-test rows.

---

## Per-Sub-Disposition Decisions

If the umbrella path is chosen, both sub-dispositions must be recorded
in this story file. If the split path is chosen, each follow-on story
records its own sub-disposition.

### B1 (Ghost-Drag Producer Fixture Gap)

- [x] **Path B1.a -- Expand fixture (test-only event-firing repair)**.
      Rationale: Investigation at PROMPT 812 time shows that the
      `GhostDragStartEvent` producer is NOT actually in `HandUiPlugin`
      as the PROMPT 750 D-5 owner comment supposed. The producer is an
      observer on `BoardRenderingPlugin` itself:
      `add_observer(on_ghost_drag_start)` at
      `client/src/presentation/board_rendering.rs:893`, with the
      observer function at line 1113 (`fn on_ghost_drag_start(trigger:
      On<Pointer<Press>>, ...)`). The matching click observer
      `on_ghost_clicked` is registered at line 892 and defined at line
      1095. The fixture failure is therefore not a missing plugin --
      it is that the test uses `world.write_message(Pointer::<Press>::
      new(...))`, which only queues the message but does NOT fire the
      observer. In real gameplay,
      `bevy_picking::DefaultPickingPlugins` calls
      `commands.trigger_targets(...)`; under `MinimalPlugins` the test
      must do the same itself. The fix is therefore to drive the
      `Pointer<Press>` / `Pointer<Click>` events via
      `world.trigger_targets(event, ghost)` in the test, which fires
      the observer and lets it `writer.write(GhostDragStartEvent {...})`
      and `writer.write(GhostClickedEvent {...})` into the message
      buffer that the assertions then drain. No `HandUiPlugin`
      registration is required. The test stays in
      `tests/integration/board_rendering/` because the assertion still
      exercises the board-rendering observer end-to-end. Zero
      production-code change. ADR-002 (no client-side optimism)
      preserved because this is a test fix only.

- [ ] **Path B1.b -- Scope assertion to a `HandUiPlugin` fixture**.
      NOT CHOSEN. Relocating would lose the cross-plugin verification
      value, and the producer is in `BoardRenderingPlugin` (not Hand
      UI) so relocation is semantically wrong. The PROMPT 750 D-5
      owner comment that suggested the producer lived in `HandUiPlugin`
      was based on an older code shape; the current `origin/main`
      shape has the observer on `BoardRenderingPlugin`.

### B5 (`ShopAuctionUiEntity` Count Drift)

- [x] **Path B5.a -- Update formula (57 -> 66)**. Rationale:
      Investigation at PROMPT 812 time shows the 9-entity delta is
      intentional capacity, not over-production. `spawn_draft_initial_grid`
      at `client/src/ui/shop_auction/mod.rs:3654-3720` spawns three
      `ShopAuctionUiEntity`-tagged entities per draft slot: (1) the
      slot container (line 3666-3677), (2) a dedicated text-child
      entity that holds the card name + cost text (line 3680-3690),
      and (3) the "BOUGHT" bought-overlay (line 3697-3711). The
      slot-text child was added deliberately -- the inline comment at
      lines 3663-3664 says: "Spawn the slot container WITHOUT Text so
      it doesn't render as a white dot when no text is set. Text lives
      in a dedicated child entity instead." Trimming this child back
      out (Path B5.b) would re-introduce the white-dot rendering
      regression that the split text-child fix prevents. The formula
      is therefore the stale artefact: the multiplier
      `SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT * 2` was written when
      only `slot + overlay` were tagged, and was not updated when the
      `text_entity` was added. The fix is to change the multiplier
      from `* 2` to `* 3` in the assertion, which raises the expected
      total from 57 to 66. Zero production-code change (formula lives
      in the test file at
      `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs`).
      ADR-002 preserved.

- [ ] **Path B5.b -- Trim spawn (66 -> 57)**. NOT CHOSEN. The 9-entity
      over-production is not over-production -- it is the 9 slot
      text-child entities deliberately introduced to fix the
      white-dot rendering bug. Trimming would regress that fix.

**Both B5 paths require a written rationale before code change.** The
silent-deletion guard from Cluster B4 (Story 014) applies in spirit
to B5: the assertion may not be changed without a recorded
production-side reason.

---

## Scope

### In Scope

- **Producer decision recorded** in this story file (umbrella vs
  split).
- **Per-sub-disposition decisions recorded** for B1 (B1.a vs B1.b) and
  B5 (B5.a vs B5.b) -- in this story file if umbrella, in the
  respective split stories if split.
- **Code changes** under the chosen sub-dispositions:
  - B1.a: expand `app_with_board_rendering()` fixture in
    `tests/integration/board_rendering/ghost_preview_bridge_test.rs`
    (or its `mod.rs` / shared helper) to register `HandUiPlugin`
    pointer-to-drag bridge. Un-`#[ignore]` the test.
  - B1.b: relocate producer-side assertion to a Hand UI integration
    test (existing or new); un-`#[ignore]` the residual board-
    rendering test.
  - B5.a: update the formula in `shop_auction_ui` scaffold to
    return 66 (rationale captured). Un-`#[ignore]` the test.
  - B5.b: trim the scaffold spawn from 66 to 57 (rationale captured;
    narrowly scoped production change). Un-`#[ignore]` the test.
- **Original PROMPT 750 D-5 owner comments removed** only after each
  affected test passes locally under the chosen sub-disposition.
- **Evidence document slot reserved** at
  `production/qa/evidence/sprint-12-fixture-d-residuals-evidence.md`
  (NEW; populated by the implementation prompt). If split, each
  follow-on story reserves its own evidence-doc slot.

### Out of Scope

- **No optimistic client-side authority introduced**. ADR-002 binding.
- No expansion to other Cluster B residuals (B2/B3/B4) -- each is
  scoped to its own Sprint 12 Must Have story.
- No broader rework of the ghost-preview bridge architecture beyond
  the chosen B1 sub-disposition.
- No broader rework of the shop-auction-ui scaffold beyond the chosen
  B5 sub-disposition.
- No Sprint 12 activation. No `production/stage.txt` modification.
  No `production/sprint-status.yaml` modification. No
  `production/sprints/sprint-12.md` modification under this story.
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, QA sign-off, or close-out under this story authoring
  prompt.
- No closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, or any
  carried Sprint 11 / Sprint 10 condition.
- No claim of public release readiness, release-candidate readiness,
  full playable-client manual QA, full game completion, broad
  Standard-tier accessibility completion, playtest / fun-hypothesis
  validation, or final-art / asset-production completion.

---

## Acceptance Criteria

(Source: `production/sprints/sprint-12.md:127` Sprint 12 draft Must Have
row. ACs below are draft and become binding at Sprint 12 activation.)

- [ ] **AC1 -- Umbrella-vs-split producer decision recorded in this
      story file**: GIVEN this story file, WHEN the "Producer
      Decision" section is read at the implementation commit, THEN
      exactly one of {umbrella, split} is checked, the unchecked path
      is explicitly marked NOT chosen, and a written rationale is
      present under the chosen path. The decision-recording commit
      precedes any code change.

- [ ] **AC2 -- Per-sub-disposition decisions recorded**: GIVEN the
      umbrella-vs-split decision, WHEN the relevant story file(s)
      are read, THEN both B1 and B5 sub-dispositions are recorded
      with a written rationale (under "Per-Sub-Disposition
      Decisions" if umbrella; in each follow-on story if split).
      Each sub-disposition recording commit precedes its
      corresponding code change.

- [ ] **AC3 -- B1 test un-`#[ignore]`d and passes under chosen
      sub-disposition**: GIVEN the chosen B1 sub-disposition, WHEN
      `cargo test -p client --test ghost_preview_bridge` (or the
      equivalent `cargo test` invocation) is run at the
      implementation commit, THEN
      `br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui`
      (B1.a) **or** its relocated counterpart in the Hand UI test
      plus the residual board-rendering test (B1.b) pass without
      `#[ignore]` tagging.

- [ ] **AC4 -- B5 test un-`#[ignore]`d and passes under chosen
      sub-disposition**: GIVEN the chosen B5 sub-disposition, WHEN
      `cargo test -p client --test plugin_scaffold_formulas` (or
      the equivalent `cargo test` invocation) is run at the
      implementation commit, THEN
      `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes`
      passes without `#[ignore]` tagging. The assertion's expected
      count (57 under B5.b; 66 under B5.a) matches the production
      reality post-fix.

- [ ] **AC5 -- Workspace ignored count drops by 2 (umbrella) or 1
      per split story**: GIVEN Sprint 11 close-out baseline of 5
      retained Cluster B `#[ignore]` tests on `origin/main`, WHEN
      `cargo test --workspace --tests --no-fail-fast` is run at the
      implementation commit, THEN the workspace ignored count drops
      by 2 (umbrella path closes both B1 and B5) or by 1 per split
      story commit (each split story drops 1). No new undocumented
      `#[ignore]` marker is introduced.

- [ ] **AC6 -- Original PROMPT 750 D-5 owner comments removed only
      after each test passes**: GIVEN the implementation commit,
      WHEN
      `tests/integration/board_rendering/ghost_preview_bridge_test.rs`
      and
      `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs`
      are read, THEN no PROMPT 750 D-5 owner comment for either test
      remains.

- [ ] **AC7 -- Production-code change scope-capped**: GIVEN the diff
      of the implementation commit set, WHEN paths under
      `client/src/` (excluding `#[cfg(test)]`-gated test helpers)
      and `shared/src/` are filtered, THEN:
      - Under B1.a or B1.b: zero production-code changes
        (fixture / test-relocation only).
      - Under B5.a: zero production-code changes (formula constant
        update may live in `client/src/ui/shop_auction_ui/` -- if
        the formula is in production source, the change is one
        constant; if the formula is in a test helper, no production
        change).
      - Under B5.b: production change is scope-capped to the
        scaffold spawn site and a single rationale doc reference.
      *Evidence*: `git show` of every commit in this story's trail
      filtered to non-test paths.

- [ ] **AC8 -- No optimistic client-side authority introduced**:
      GIVEN the implementation commit, WHEN the diff is reviewed for
      any client-side mutation of authoritative state outside the
      shared phase sink, snapshot drainers, and S2C consumers, THEN
      no such mutation is present. ADR-002 binding. *Evidence*: text
      search for "no optimistic" in the evidence document.

- [ ] **AC9 -- If split chosen, two follow-on story files authored**:
      GIVEN the split decision, WHEN
      `production/epics/playable-client/` is listed, THEN
      `story-NNN-fixture-board-ghost-drag-producer.md`
      (`S11-TD-FIXTURE-BOARD-GHOST-DRAG-PRODUCER-001`) and
      `story-NNN-shop-auction-ui-count-drift.md`
      (`S11-TD-SHOP-AUCTION-UI-COUNT-DRIFT-001`) exist with the
      no-claim banner, evidence-path conventions, and decision-first
      discipline inherited from this story. This umbrella story's
      Sprint 12 row in `production/sprint-status.yaml` (once
      activated) is closed as the producer-decision-record artefact.

- [ ] **AC10 -- Sprint 12 disposition preserved**: GIVEN the
      implementation commit, WHEN `production/sprint-status.yaml`,
      `production/sprints/sprint-12.md`, and `production/stage.txt`
      are diffed, THEN none of them are modified under this story.
      Sprint 12 activation disposition is preserved. Stage remains
      `Polish`. Sprint 11 disposition (`closed-with-conditions`) is
      unchanged.

- [ ] **AC11 -- Evidence document slot(s) reserved**: GIVEN this
      story file, WHEN the evidence-doc path is checked, THEN a slot
      is reserved at
      `production/qa/evidence/sprint-12-fixture-d-residuals-evidence.md`
      (umbrella) or one slot per split story (if split chosen).
      Authoring of the evidence file(s) is deferred to the
      implementation prompt(s).

---

## Implementation Notes

This story is **draft** at authoring time. Activation requires (in
order):

1. `/sprint-plan sprint-12` activates Sprint 12 (separate prompt).
2. This story passes `/story-readiness` (separate prompt).
3. Sprint 12 `/qa-plan sprint` is authored (separate prompt).
4. `/dev-story story-015-fixture-d-residuals.md` is dispatched
   (separate prompt). If the split decision is made, the dispatch
   may delegate to two separate `/dev-story` runs.

Expected implementation flow:

1. **Wave 1 -- Umbrella-vs-split decision**: the implementation
   prompt (or a separate producer prompt) records the umbrella-vs-
   split decision in this story file's "Producer Decision" section.
   **This commit precedes any code change.**
2. **Wave 2 -- Per-sub-disposition decisions**: B1.a vs B1.b and
   B5.a vs B5.b decisions recorded with rationale.
3. **Wave 3 -- Code changes**: applied per chosen sub-dispositions.
   B1: fixture expansion or assertion relocation. B5: formula update
   or scaffold spawn trim.
4. **Wave 4 -- Validation**: run `cargo test -p client --no-fail-fast`
   and `cargo test --workspace --tests --no-fail-fast`; capture
   pre/post pass + ignored counts in the evidence document.
5. **Wave 5 -- Evidence doc**: populate
   `production/qa/evidence/sprint-12-fixture-d-residuals-evidence.md`
   (umbrella) or each split story's evidence doc with the decisions
   transcribed, diff summary, pre/post counts, no-claim restatement,
   and cross-link to Cluster B1 row 83 + Cluster B5 row 87 in the
   triage doc.

---

## Performance Budget

N/A -- fixture / scaffold / formula changes only; no hot-path code
changed.

---

## QA Test Cases

(Draft -- becomes binding at Sprint 12 activation. Sprint 12 QA plan
authored via `/qa-plan sprint` will pull from this set.)

- **Decision-first ordering audit**
  - Given: the story's commit trail on `main`.
  - When: `git log --oneline` is reviewed.
  - Then: umbrella-vs-split decision commit precedes any code-change
    commit; per-sub-disposition decision commit(s) precede their
    corresponding code-change commit(s).

- **B1 test passes under chosen sub-disposition**
  - Given: implementation commit set on `main` for this story.
  - When: `cargo test -p client --test ghost_preview_bridge` (or
    equivalent) is run.
  - Then: the chosen B1 sub-disposition's test variant passes
    without `#[ignore]` tagging.

- **B5 test passes under chosen sub-disposition**
  - Given: implementation commit set on `main` for this story.
  - When: `cargo test -p client --test plugin_scaffold_formulas`
    (or equivalent) is run.
  - Then: `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes`
    passes without `#[ignore]` tagging; expected count matches the
    chosen sub-disposition's reality.

- **Workspace ignored-count regression check**
  - Given: Sprint 11 close-out baseline (1129 passing / 5 ignored on
    `origin/main@8a8451e`).
  - When: `cargo test --workspace --tests --no-fail-fast` is run at
    the implementation commit.
  - Then: workspace ignored count drops by 2 (umbrella) or 1 per
    split commit; no new undocumented `#[ignore]` marker is
    introduced.

- **Production diff scope audit**
  - Given: the union diff of every commit in this story's trail.
  - When: paths under `client/src/` (excluding `#[cfg(test)]` gates)
    and `shared/src/` are filtered.
  - Then: scope matches AC7 (zero production change under B1.x or
    B5.a; narrowly scoped scaffold spawn change under B5.b).

- **No-optimism audit**
  - Given: the diff of the implementation commit set.
  - When: the diff is reviewed for any client-side optimistic state
    mutation.
  - Then: no such mutation is present. ADR-002 binding.

- **Sprint 12 disposition preservation audit**
  - Given: the implementation commit.
  - When: `production/sprint-status.yaml`,
    `production/sprints/sprint-12.md`, and `production/stage.txt`
    are diffed.
  - Then: none of them are modified under this story id.

---

## Test Evidence

**Story Type**: Decision-first composite (umbrella vs split + B1 + B5
fixture cleanup / scaffold decision).

**Evidence path**: `production/qa/evidence/sprint-12-fixture-d-residuals-evidence.md`
(NEW; populated by the implementation prompt). If split chosen, each
follow-on story reserves its own evidence-doc slot.

**Required evidence content** (deferred to implementation prompt):

- Umbrella-vs-split producer decision + rationale (transcribed from
  this story file's "Producer Decision" section).
- Per-sub-disposition decisions + rationale (B1.a vs B1.b, B5.a vs
  B5.b).
- Code-change diff summary per sub-disposition.
- Pre/post `cargo test -p client` pass + ignored counts.
- Pre/post `cargo test --workspace --tests --no-fail-fast` pass +
  ignored counts.
- No-claim restatement (verbatim from this story file's "Status /
  No-Claim Banner" section), including the explicit "no optimistic
  client-side authority" line.
- Cross-link back to this story file and to Cluster B1 row 83 +
  Cluster B5 row 87 in
  `production/qa/evidence/sprint-11-ignored-d5-triage.md`.

**Required verification commands** (for the implementation prompt):

- `cargo test -p client --no-fail-fast`
- `cargo test --workspace --tests --no-fail-fast`
- `git log --oneline -- production/epics/playable-client/story-015-fixture-d-residuals.md tests/integration/board_rendering/ghost_preview_bridge_test.rs tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs client/src/`
- `git diff <pre-impl-sha>..<impl-sha> -- 'client/src/**' 'shared/src/**'`
- `git diff --check` and `git diff --cached --check` before commit

**Status**: [ ] Captured and locked

---

## Owner / Classification

- **Owner**: test infra + scaffold owner + qa-lead -- per Cluster B1
  row 83 (board-rendering test-infra owner) and Cluster B5 row 87
  (shop-auction-ui scaffold owner) in
  `production/qa/evidence/sprint-11-ignored-d5-triage.md`.
- **Estimated days**: 1.25 (per Sprint 12 draft row).
- **Story classification**: composite decision-first (umbrella vs
  split) + fixture cleanup (B1) + formula/scaffold decision (B5).

## Dependencies

- **Depends on**: Sprint 12 activation via `/sprint-plan sprint-12`
  (separate prompt). This story remains `Draft` until then.
- **Depends on**: Sprint 11 D-5 triage doc
  (`production/qa/evidence/sprint-11-ignored-d5-triage.md`, Cluster B1
  row 83 and Cluster B5 row 87) for owner / disposition / decision-
  gate language.
- **Depends on**: Sprint 12 QA plan authored via `/qa-plan sprint`
  (separate prompt) before `/dev-story` runs.
- **Coordinated with**: `S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001`
  (Story 012 -- Cluster B2 HUD bridge fixture; sibling fixture work,
  but disjoint files).
- **Coordinated with**: `S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001`
  (Story 014 -- Cluster B4 binary decision; precedent for decision-
  first discipline).
- **Not coordinated with**: `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001`
  (Story 013 -- different module; no shared file scope).
- **Not coordinated with**: `S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001`
  (story 019 in hand-ui epic -- different module; no shared file
  scope).

## Readiness Notes

**Implementation readiness verdict**: Draft -- substantive work has not
started. The live downstream gates are Sprint 12 activation, Sprint 12
QA plan authorship, and `/story-readiness` PASS on this file.

Pre-conditions for `/story-readiness` PASS:

- Sprint 12 is activated (`sprint:` field in
  `production/sprint-status.yaml` bumped + active row written) --
  **Pending separate `/sprint-plan sprint-12` prompt.**
- Sprint 12 QA plan exists at `production/qa/qa-plan-sprint-12.md` --
  **Pending separate `/qa-plan sprint` prompt.**
- This story file is referenced from Sprint 12's active row in
  `production/sprint-status.yaml` after activation -- **Pending
  separate `/sprint-plan sprint-12` prompt.**

Open questions to resolve at `/story-readiness` time:

- Is the producer prepared to record the umbrella-vs-split decision
  inside this story file at implementation time, or should a
  separate producer prompt land the decision before `/dev-story`
  dispatch?
- For B5: is the formula in production source
  (`client/src/ui/shop_auction_ui/`) or in a test helper? (Determines
  whether Path B5.a is a production change or test-helper change.)
- For B1: is the producer-side `GhostDragStartEvent` emit-site coverage
  already present in `tests/integration/hand-ui/`? (Determines whether
  Path B1.b creates a coverage gap.)

---

## Files Anticipated To Be Modified (planning estimate, NOT binding)

| Path | Anticipated change |
|------|--------------------|
| `tests/integration/board_rendering/ghost_preview_bridge_test.rs` | B1.a: expand fixture; un-`#[ignore]`. B1.b: split assertion; un-`#[ignore]` residual. |
| `tests/integration/hand-ui/ghost_preview_producer_test.rs` (NEW; B1.b only) | new test asserting producer-side `GhostDragStartEvent` emission with `HandUiPlugin` wired. (May instead append to an existing Hand UI test file.) |
| `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs` | B5.a: assertion expected count updated to 66. B5.b: assertion stays at 57 after production spawn trim. Both: un-`#[ignore]` after test passes. |
| `client/src/ui/shop_auction_ui/<scaffold>.rs` (B5.b only) | B5.b: trim spawn from 66 to 57; commit comment references the rationale. |
| `client/src/ui/shop_auction_ui/<formula>.rs` (B5.a only, if formula is in production source) | B5.a: update formula constant from 57 to 66; commit comment references the rationale. |
| This story file (decision-recording commit) | "Producer Decision" + "Per-Sub-Disposition Decisions" sections updated. **Commit precedes code change.** |
| `production/qa/evidence/sprint-12-fixture-d-residuals-evidence.md` (NEW; umbrella) OR `production/qa/evidence/sprint-12-fixture-board-ghost-drag-producer-evidence.md` + `production/qa/evidence/sprint-12-shop-auction-ui-count-drift-evidence.md` (split) | evidence document(s) per AC11 |
| `production/epics/playable-client/story-NNN-fixture-board-ghost-drag-producer.md` (NEW; split only) | new story file inheriting from this story; covers B1 only. |
| `production/epics/playable-client/story-NNN-shop-auction-ui-count-drift.md` (NEW; split only) | new story file inheriting from this story; covers B5 only. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Implementation prompt lands the code change before recording the umbrella-vs-split decision | Medium | High | AC1 hard constraint; verified by `git log --oneline`. |
| Per-sub-disposition rationale is hand-waved | Medium | Medium | AC2 + AC7 + AC8 force written rationale + scope-cap; evidence doc cross-link forces explicit recording. |
| Path B5.b production scaffold trim introduces a regression elsewhere | Low | Medium | Scope-cap to the scaffold spawn site; `cargo test --workspace --tests --no-fail-fast` regression check; if a regression surfaces, the scope expands to fix the regression OR Path B5.a is reconsidered. |
| Path B1.b relocation creates a coverage gap (no Hand UI test covers the producer side) | Medium | Medium | `/story-readiness` open question forces verification; if no coverage exists, Path B1.a becomes binding. |
| Split path is chosen but one of the follow-on stories is never authored | Medium | Medium | AC9 hard constraint: both follow-on stories must exist before this umbrella closes. |
| Production change accidentally introduces client-side optimism | Low | High | AC8 hard constraint + ADR-002 reviewer check. |
| Sprint 12 activation does not happen before implementation dispatch | Low | High | Activation is a separate prompt gate; this story stays `Draft` until activation. |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator emitting the implementation
prompt, not for the worker:

- `production/sprint-status.yaml` `sprint:` field reads `sprint 12` and
  this story is referenced from the active row, OR the row is held with
  a written blocker.
- `production/stage.txt` reads `Polish` and is unchanged.
- `production/sprints/sprint-12.md` status block reads `active` (after
  separate `/sprint-plan sprint-12` activation prompt).
- The PROMPT 761 Polish->Release gate-check FAIL evidence at
  `production/gate-checks/gate-polish-release-2026-05-12.md` is
  preserved.
- `git diff --check` and `git diff --cached --check` pass before any
  commit.

---

## Authoring Trail

- 2026-05-14 -- PROMPT 795 -- Story file authored as a Sprint 12 draft
  Must Have umbrella for Cluster B1 + B5. Sprint 12 is `draft`
  (PROMPT 793) and not yet activated -- this story is **not yet
  activated** into the Sprint 12 active scope. Activation is a
  separate prompt (`/sprint-plan sprint-12`). No code changes, no
  smoke / gate / QA / `/dev-story` / `/story-done` / `/story-readiness` /
  `/qa-plan` run. Source-of-truth at authoring: `origin/main@f72cc60`.
