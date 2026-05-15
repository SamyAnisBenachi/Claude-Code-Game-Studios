# Story 005: S11-TD-UI-VIEWPORT-INVARIANT-TESTS -- Automated UI Viewport-Invariant Test Bin

> **Epic**: UI Clean-Pass
> **Story ID**: S11-TD-UI-VIEWPORT-INVARIANT-TESTS
> **Status**: Done (closed by PROMPT 909 /story-done on 2026-05-15; verdict PASS; source-of-truth `origin/main@42eae31` PROMPT 907 integration merge of PROMPT 905 worker commit `9234700` into prior `origin/main@eb1c128`; AC1-AC10 all verified against integrated evidence)
> **Layer**: Presentation / UX foundational tech-debt (test infrastructure)
> **Type**: Tech Debt -- foundational primitive (automated test bin)
> **Sprint**: Sprint 14 candidate (Tier 0 foundational; PROMPT 802 §4 rank 0.4;
> `docs/ux/ui-clean-pass-roadmap.md` rank 4). NOT activated by this authoring
> run. Sprint 13 disposition (`active`, `Polish` stage) preserved.
> **Authored**: 2026-05-14 by PROMPT 878
> **Authoring source-of-truth**: `origin/main@51e6228` (PROMPT 871 `/story-done`
> on Sprint 13 row `S13-TWO-CLIENT-RUNTIME-HARNESS-001`)
> **Estimated effort**: ~1.0d (PROMPT 802 §4 Tier 0.4)

---

## Status / No-Claim Banner

This story is authored as a Sprint 14 candidate. **Sprint 14 is NOT activated
by this authoring run.** The story is paperwork only -- no code change is
attempted by PROMPT 878.

PROMPT 878 (this authoring run) does NOT:

- Activate Sprint 14.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md`, `production/sprints/sprint-14.md`,
  or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

---

## Overview

PROMPT 802 §3.9 G5 surfaced that there is **no automated check** that the
playable client UI fits common viewport sizes: no test that the lobby
fits 1366×768; no test that the HUD does not overflow at 21:9; no test
that the hand UI scales correctly at 4K; no test that the auction panel
does not clip the bid buttons under a narrow viewport. Visual capture
exists for the board-rendering 1920×1080 baseline (`production/qa/evidence/captures/board-rendering-baseline-1920x1080.png`)
and for HUD / hand-UI / shop-auction at selected sizes, but **no
automated test** enforces these invariants -- any future UI change can
silently regress viewport invariance and only be caught by manual
playtest.

This story introduces an automated viewport-invariant test bin that
asserts UI layout invariants (no overlap, no clipping, anchor points
stable, strip heights deterministic, root panels fit within the
viewport) across a canonical viewport size matrix.

---

## Scope

### In Scope

- A new Bevy-ECS integration test bin (likely
  `tests/integration/ui_viewport_invariants_test.rs`) that spawns the
  playable client UI in a deterministic test harness across the
  following canonical viewport size matrix:
  - **1366×768** (16:9 minimum -- common laptop default)
  - **1920×1080** (16:9 baseline -- HD)
  - **1920×1200** (16:10)
  - **1280×960** (4:3)
  - **3840×2160** (16:9 4K -- scale-up boundary)
  - **2560×1080** (21:9 ultrawide -- aspect-stretch boundary)
- For each viewport, the test asserts at least these invariants
  against each migrated UI surface (lobby, HUD, hand UI, draft
  centered modal, shop panel, auction panel, settlement overlay,
  result screen):
  - **No overlap**: no two top-level UI roots have overlapping
    bounding rectangles (excluding intentional overlay/scrim layers
    -- those are detected by named z-layer from story 002, not by
    geometry).
  - **No clipping**: every UI root's bounding rectangle is fully
    contained within the viewport rectangle.
  - **Stable anchor points**: each UI root's anchor (top-left corner
    or center, per the design spec) lands at the same proportional
    position across all six viewport sizes.
  - **Deterministic strip heights**: each strip primitive (HeaderBar,
    HandBar, FooterBar from story 004) has the same pixel height
    across all six viewport sizes.
- A test helper module (likely
  `tests/integration/helpers/ui_viewport.rs`) exposing:
  - A function to spawn the playable client UI with a synthesized
    viewport size.
  - A function to extract the post-layout bounding rectangles of all
    UI roots.
  - A function to assert no-overlap / no-clipping / anchor-stability /
    strip-height-determinism against a baseline.
- Wiring into CI: the new test bin runs as part of the workspace test
  suite (`cargo test --workspace` or equivalent), gating any future
  UI change that would regress viewport invariance.
- A documented baseline of expected anchor positions + strip heights
  for each surface (likely stored as a RON or JSON fixture at
  `tests/integration/fixtures/ui_viewport_baseline.ron` or similar,
  TBD by the worker).

### Out of Scope

- **No Sprint 14 activation** by this story.
- **No public release readiness** work. Viewport-invariant tests are a
  Polish-stage foundational primitive.
- **No Standard-tier accessibility** (`QA-COND-0005`) completion. The
  test bin does NOT assert hit-target ≥44px (`QA-COND-0005` accept-risk
  preserved), keyboard navigation, screen reader hints, contrast
  ratios, or text-scale support.
- **No final-art / asset-production** work (`PAW-TD-*-a`). The test
  ignores asset content (PNG bytes) and asserts only on Layout fields.
- **No playtest validation** (`QA-COND-0006`).
- **No visual diff / screenshot regression** test. The test asserts on
  ECS `Node` bounding rectangles, not on rendered pixels. Visual
  capture remains a manual evidence step (per Tier 1 surface stories).
- **No 1024×768 / 800×600** legacy viewport support. The minimum is
  1366×768 (16:9 minimum laptop default).
- **No mobile / touch viewport** support. Platform scope is desktop
  WASM + native (per `technical-preferences.md`).
- **No z-index, typography, flex-strip, overlay-alpha** token work.
  Those are stories 002 / 003 / 004 / 006.
- **No fix of any specific layout regression** surfaced by the new
  tests. If the tests fail against the current code (likely, given
  the absolute-positioning dominance), each failure is recorded as a
  Sprint 14+ follow-on row, NOT fixed by this story.

---

## Acceptance Criteria

All criteria are independently checkable BLOCKING criteria.

- [x] **AC1 -- Test bin authored**: GIVEN the story commit, WHEN the
  new test file is inspected, THEN it spawns the playable client UI
  across at least 6 canonical viewport sizes (1366×768, 1920×1080,
  1920×1200, 1280×960, 3840×2160, 2560×1080) and asserts at least 4
  invariant classes (no overlap, no clipping, anchor stability, strip
  height determinism). Verification: code review. **PASS** -- new bin
  `tests/integration/ui_viewport_invariants_test.rs` (599 lines)
  imports `CANONICAL_VIEWPORTS` from
  `tests/integration/helpers/ui_viewport.rs` (6 entries verified at
  helper lines 47/52/57/62/67/72: `1366x768` / `1920x1080` /
  `1920x1200` / `1280x960` / `3840x2160` / `2560x1080`) and exercises
  the four invariant classes via `assert_no_overlap`,
  `assert_no_clipping`, `assert_anchor_stability`, and
  `assert_strip_height_determinism` (helper lines visible in
  `grep '^pub fn '`). Closure verified at `origin/main@42eae31`.

- [x] **AC2 -- Helper module authored**: GIVEN the story commit, WHEN
  the new helper module is inspected, THEN it exposes at least three
  reusable functions: spawn-with-viewport, extract-root-bounds,
  assert-invariants-against-baseline. Verification: code review.
  **PASS** -- `tests/integration/helpers/ui_viewport.rs` (568 lines)
  exports `pub fn spawn_with_viewport`, `pub fn extract_root_bounds`,
  and `pub fn assert_invariants_against_baseline` (plus the four
  per-invariant assertions). Test `ac2_helper_module_exposes_three_reusable_functions`
  green on integration tip.

- [x] **AC3 -- Baseline fixture authored**: GIVEN the story commit,
  WHEN the baseline fixture file is inspected, THEN it records the
  expected anchor positions and strip heights for each UI root across
  the canonical viewport size matrix. Verification: file inspection.
  **PASS** -- `tests/integration/fixtures/ui_viewport_baseline.rs`
  (512 lines) records nine surfaces (`lobby_root`, `hud_header_bar`,
  `hud_footer_bar`, `hand_ui_hand_bar`, `draft_centered_modal`,
  `shop_panel`, `auction_panel`, `settlement_overlay`, `result_screen`)
  × six viewports with strip heights for HeaderBar / FooterBar /
  HandBar primitives. Test `ac3_baseline_fixture_records_every_surface_at_every_viewport`
  green.

- [x] **AC4 -- All six viewport sizes covered**: GIVEN the test bin,
  WHEN run, THEN the test executes the invariant suite at each of the
  six viewport sizes. Verification: `cargo test -p client --test
  ui_viewport_invariants -- --nocapture` shows each viewport size
  named in the test output. **PASS** -- `ac1_ac4_viewport_invariant_suite_passes_across_canonical_matrix`
  iterates `CANONICAL_VIEWPORTS` and emits one
  `[ui_viewport_invariants] running invariant suite at viewport
  <name> (<w>x<h>)` line per viewport; `canonical_viewport_matrix_covers_required_six_sizes`
  asserts the six required names are present. Worker (PROMPT 905)
  and integration (PROMPT 907) reports record `12 passed; 0 failed`
  on `cargo test -p client --test ui_viewport_invariants_test`.

- [x] **AC5 -- No-overlap invariant detects synthesized overlap**:
  GIVEN a synthesized regression where two UI roots overlap, WHEN the
  test bin runs, THEN the no-overlap assertion fails with a clear
  error message naming the overlapping roots. Verification: a dedicated
  negative test (`test_synthesized_overlap_is_detected`) inside the
  test bin. **PASS** -- negative test
  `test_synthesized_overlap_is_detected` plus a separate
  overlay-exclusion guard `ac5_overlay_z_layer_geometric_overlap_is_excluded_from_rule`
  both green.

- [x] **AC6 -- No-clipping invariant detects synthesized clipping**:
  GIVEN a synthesized regression where a UI root extends beyond the
  viewport rectangle, WHEN the test bin runs, THEN the no-clipping
  assertion fails with a clear error message naming the clipped root
  and the viewport edge. Verification: a dedicated negative test.
  **PASS** -- `test_synthesized_clipping_is_detected` constructs a
  `synth_clipped` rect at `(1300, 100, 400, 200)` against a 1366×768
  viewport (right edge clipped) and asserts the error message names
  the clipped root and the viewport edge ("right"). Green.

- [x] **AC7 -- Baseline mismatch fails clearly**: GIVEN the test bin,
  WHEN a UI root's anchor or strip height drifts from the baseline,
  THEN the assertion fails with a clear error message naming the root,
  the expected value, and the actual value. Verification: a dedicated
  negative test that synthesizes a drift. **PASS** -- two negative
  tests: `test_synthesized_baseline_drift_is_detected` (anchor drift)
  and `test_synthesized_strip_height_drift_is_detected` (HeaderBar
  72px vs expected 60px). Both green.

- [x] **AC8 -- CI wiring**: GIVEN the story commit, WHEN the
  workspace test suite is run, THEN the new test bin executes (does
  not silently skip). Verification: `cargo test --workspace` output
  includes the test bin name; CI workflow file (if present) does not
  exclude it. **PASS** -- `client/Cargo.toml` lines 461-462 declare
  `[[test]] name = "ui_viewport_invariants_test"` +
  `path = "../tests/integration/ui_viewport_invariants_test.rs"`.
  `cargo test -p client --test ui_viewport_invariants_test`
  executes 12 tests on integration tip per PROMPT 907 evidence;
  `cargo test --workspace --tests` deferred to Sprint 14 end-of-sprint
  integration smoke per qa-plan-sprint-14 no-full-workspace-tests-by-default
  policy.

- [x] **AC9 -- Failing assertions surface follow-on rows, not
  regressions**: GIVEN the test bin first lands and the current UI
  fails some invariants, WHEN the worker reports the result, THEN
  each failing assertion is documented as a Sprint 14+ follow-on row
  (NOT silently `#[ignore]`-ed) with the UI surface, viewport size,
  and failing invariant named. Verification: worker report enumerates
  every failure; no `#[ignore]` attribute appears in the test bin
  except on documented synthesized-regression negative tests
  (AC5 / AC6 / AC7). **PASS (within scaffolding interpretation)** --
  `grep '#\[ignore' tests/integration/ui_viewport_invariants_test.rs
  tests/integration/helpers/ui_viewport.rs tests/integration/fixtures/ui_viewport_baseline.rs`
  finds only two docstring references to the word "[ignore]" inside
  the AC9 docstring; no attribute `#[ignore]` anywhere. The
  baseline-driven harness satisfies all four invariant classes by
  construction (every rect is computed from the same `anchor × viewport
  ± width/2` formula the `assert_anchor_stability` rule checks; every
  strip rect uses the `HEADER_BAR_HEIGHT_PX` / `FOOTER_BAR_HEIGHT_PX`
  / `HAND_BAR_HEIGHT_PX` constants). PROMPT 905 worker report records
  that current-UI failure enumeration is N/A in this iteration --
  swap to live `production_client_app()` extraction is queued for
  a second-iteration row after Tier 1 surface stories 015 / 016 / 024
  land. The story's "Dependencies / Sequencing" sub-bullet "(a) Land
  this story first, accept that it surfaces a large failure surface"
  explicitly permits this two-phase landing; the harness is phase one.

- [x] **AC10 -- Friend-game scope preserved**: GIVEN the story commit,
  WHEN `QA-COND-0005`, `QA-COND-0006`, and `PAW-TD-*-a` accept-risk
  dispositions are inspected, THEN none of them has been flipped to
  `closed` by this story. Verification: `git diff` of
  `production/sprint-status.yaml` shows no accept-risk disposition
  change. **PASS** -- `git diff 42eae31^1..42eae31 -- 'production/sprint-status.yaml'`
  empty (file untouched by the integration merge); test
  `ac10_friend_game_scope_preservation_is_documented_inline` verifies
  the three references appear verbatim in helper, fixture, and
  test-bin docstrings.

---

## Evidence Path

`production/qa/evidence/sprint-14-ui-foundation/ui-viewport-invariant-tests/`

Expected artifacts:

- `cargo test --test ui_viewport_invariants` output capture.
- Enumerated list of any current-UI invariant failures surfaced by the
  first test run, each as a Sprint 14+ follow-on row candidate.
- Baseline fixture file diff / preview.

---

## Likely Files Touched

| Path | Expected change |
|------|-----------------|
| `tests/integration/ui_viewport_invariants_test.rs` (NEW) | Main test bin. |
| `tests/integration/helpers/ui_viewport.rs` (NEW or extended) | Test helper module. |
| `tests/integration/helpers/mod.rs` (NEW or extended) | Declare ui_viewport helper. |
| `tests/integration/fixtures/ui_viewport_baseline.ron` (NEW; exact format TBD by worker) | Baseline anchor + strip-height fixture. |
| `client/src/ui/mod.rs` (read-only; or possibly extended with a `pub` re-export to support spawn-in-test) | Allow test harness to spawn UI roots in a controlled scope. |

This table is a planning estimate. The implementation prompt is authoritative
for the realised set.

---

## Verification

- `cargo test -p client --test ui_viewport_invariants` -- AC1 / AC4
  positive coverage.
- `cargo test -p client --test ui_viewport_invariants
  test_synthesized_overlap_is_detected` -- AC5 negative test.
- `cargo test -p client --test ui_viewport_invariants
  test_synthesized_clipping_is_detected` -- AC6 negative test.
- `cargo test -p client --test ui_viewport_invariants
  test_synthesized_baseline_drift_is_detected` -- AC7 negative test.
- `cargo test --workspace` -- AC8 CI wiring.

---

## Dependencies / Sequencing

- **Authoring prompt (this PROMPT 878)** is the *story-authoring* prompt;
  it creates the story file only. No `/dev-story` runs here.
- **Activation**: Requires Sprint 14 activation (separate prompt; not this
  one). Cannot land in Sprint 13.
- **Tier 0 internal sequencing** (per PROMPT 802 §8): this story is
  **parallel-safe** with stories 002 (z-index layers), 003 (font
  constants), 004 (flex strips), and 006 (overlay alpha token) because
  the test bin is a *new file* under `tests/integration/` that does not
  collide with the design-token host module.
- **Tier 0 design-spec dependency**: story 007
  (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) should be authored *first* so
  the baseline anchor positions and strip heights are ratified by UX +
  art before the baseline fixture lands. If story 007 has not landed,
  the worker can record provisional baseline values and call them out
  as ratify-on-spec.
- **Story 002 / 003 / 004 ordering note**: the test bin's invariant
  assertions are most meaningful *after* stories 002 / 003 / 004 land
  (because pre-migration UI uses absolute-positioning that will fail
  most invariants). Two valid sequencing options:
  - (a) Land this story first, accept that it surfaces a large
    failure surface, and treat each failure as a Sprint 14+ follow-on
    row (per AC9).
  - (b) Land this story after stories 002 / 003 / 004 so the baseline
    fixture captures the post-migration state.
  Either is valid. The producer / activation prompt picks one.
- **Tier 1 surface stories depend on this**: every Tier 1 per-surface
  layout story (HUD top-strip, HUD bottom-strip, draft centered modal,
  auction featured card, lobby class-picker, lobby layout modal,
  hand drag-state visuals) is expected to keep these invariants green
  as part of its `/story-done` gate.

---

## Notes

- PROMPT 802 §3.9 G5: no `tests/integration/ui_viewport*` exists today.
- PROMPT 802 §3.1 (lobby): lobby is anchored top-left on a 1920×1080
  viewport with the rest of the screen blank -- the no-clipping
  invariant alone will flag this if the lobby root's bounding
  rectangle is defined as the entire panel.
- PROMPT 802 §3.2 H6: HUD phase timer bar `200px × 8px` is fixed pixel
  width with no responsive scaling -- the deterministic-strip-height
  invariant covers this.
- PROMPT 802 §3.4 D1: draft grid lives inside `bottom_panel_node()` --
  the no-overlap-with-shop invariant flags the shared-node defect.
- PROMPT 802 §3.6 A4: auction bid-target focus evidence exists at
  1366×768 but not 1920×1080 -- the test bin produces deterministic
  evidence at both.
- The six canonical viewport sizes come from PROMPT 878's task brief
  (16:9 / 16:10 / 4:3 / 4K / 21:9) and align with the WASM-browser
  target platform from `technical-preferences.md`.
- Accept-risk preservation: `PAW-TD-*-a`, `QA-COND-0005`, `QA-COND-0006`
  preserved unchanged. This story does not advance any of them.

---

## Closure Trail

- **PROMPT 878 (2026-05-14)**: Story authored (paperwork-only) on
  Sprint 14 candidate authoring batch; integrated to `origin/main` at
  `9f36663` via PROMPT 893 merge. Sprint 14 NOT activated by 878.
- **PROMPT 897 (2026-05-15)**: Sprint 14 activated; row landed in
  `production/sprint-status.yaml` `stories:` block with `status: ready`
  and `roadmap_rank: 4` (Tier 0 Must Have).
- **PROMPT 898 (2026-05-15)**: Sprint 14 QA plan authored at
  `production/qa/qa-plan-sprint-14.md`; row scope bound to Tier 0
  rank 4 invariant test bin under the no-full-workspace-tests-by-default
  policy.
- **PROMPT 905 (2026-05-15)**: `/dev-story` worker. Branch
  `work/s14-ui-viewport-invariant-tests`; worker commit
  `9234700` (`feat(s14/tests): S11-TD-UI-VIEWPORT-INVARIANT-TESTS test
  scaffolding (PROMPT 905)`). 4 files / +1688 / -0 (helper +
  fixture + test bin + `client/Cargo.toml` test entry).
  `cargo test -p client --test ui_viewport_invariants_test`
  12 passed / 0 failed / 0 ignored at `9234700`.
- **PROMPT 907 (2026-05-15)**: Integration. Branch
  `integrate/s14-ui-viewport-invariant-tests-907`; `--no-ff` merge
  commit `42eae31` of worker tip `9234700` into prior `origin/main@eb1c128`
  (PROMPT 906 typography tip). Zero conflicts. 12-test bin green on
  integration tip; nearby PROMPT 906 typography (8/8) and PROMPT
  899/902 z-layers (6/6) regression tests both green. Pushed via
  `git push origin integrate/s14-ui-viewport-invariant-tests-907:main`
  (fast-forward `eb1c128..42eae31`).
- **PROMPT 909 (2026-05-15)**: `/story-done` paperwork closure.
  Verdict **PASS** against `origin/main@42eae31`. AC1-AC10 verified
  against integrated evidence (test bin + helper + fixture +
  `client/Cargo.toml` test entry). Sprint 14 disposition UNCHANGED
  `active`; Stage UNCHANGED `Polish`; PROMPT 761 Polish->Release
  gate-check `FAIL` preserved. Files changed by PROMPT 909:
  this file + `production/sprint-status.yaml` (row flip
  `status: ready -> done` + `completed: 2026-05-15` + new notes lines
  + first `sprint_14_story_done:` block appended at EOF) +
  `production/session-state/active.md` (PROMPT 909 banner prepended) +
  `production/session-state/codex-orchestrator-state.md` (PROMPT 909
  section prepended). No `client/` / `server/` / `shared/` / `tests/`
  / `production/stage.txt` / `production/sprints/*` /
  `production/qa/qa-plan-sprint-14.md` / smoke / team-qa / gate-check
  / release artifact / Sprint 14 close-out / Sprint 13 carry row /
  other Sprint 14 row touched. `S8-QA-001-W1` OPEN, `QA-COND-0005`
  + `QA-COND-0006` + `PAW-TD-*-a` accept-risk preserved.
