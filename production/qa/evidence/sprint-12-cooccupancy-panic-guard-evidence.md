# Sprint 12 — Story 014 Co-occupancy Panic-Guard — Evidence

> **Sprint**: Sprint 12 (active per PROMPT 798 / `b5eef0d`)
> **Story**: `S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001`
> **Story file**: `production/epics/playable-client/story-014-cooccupancy-panic-guard-decision.md`
> **QA-plan section**: `production/qa/qa-plan-sprint-12.md#story-014--cluster-b4-co-occupancy-panic-guard--decision-gate-evidence`
> **Triage source**: `production/qa/evidence/sprint-11-ignored-d5-triage.md` Cluster B4 row 86
> **Authored**: 2026-05-14 by PROMPT 800 (`/dev-story story-014-cooccupancy-panic-guard-decision.md`)
> **Worktree**: `D:/_DEV/claude-code-game-studios-worktrees/s11-cooccupancy-panic-guard-decision`
> **Worker branch**: `work/s11-cooccupancy-panic-guard-decision`
> **Source-of-truth at decision**: `origin/main@b5eef0d` (PROMPT 799 Sprint 12 QA plan commit)

---

## No-Claim Restatement (verbatim from story 014)

This evidence document records the implementation of story 014 under Sprint 12.
It does **not** claim: public release readiness, release-candidate readiness,
full game completion, broad / Standard-tier accessibility completion
(`QA-COND-0005`), playtest / fun-hypothesis validation (`QA-COND-0006`), full
playable-client manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`),
final-art / asset-production completion, Sprint 12 close-out, or stage advance
from Polish to Release.

Sprint 10 disposition (`closed-with-conditions` per PROMPT 763) and Sprint 11
disposition (`closed-with-conditions` per PROMPT 792) remain unchanged. PROMPT
761 Polish→Release gate-check FAIL evidence at
`production/gate-checks/gate-polish-release-2026-05-12.md` is preserved (no
retry attempted under PROMPT 800).

---

## Decision: Path B (Test rewritten to assert non-panic behaviour)

Path A explicitly **NOT** chosen. Path B chosen and rationale recorded in
story file Binary Design Decision section *before* the test-rewrite commit.

### Decision-recording ordering audit

`git log --oneline origin/main..HEAD -- <story-file> <test-file> <production-file>`
(run at worktree `s11-cooccupancy-panic-guard-decision`):

```
ae6635d fix(s12-b4): rewrite cooccupancy index-two test for Path B clamp behaviour (PROMPT 800 Wave 2)
d5053fe decide(s12-b4): record Path B rationale for co_occupancy_offset (PROMPT 800 Wave 1)
```

- **`d5053fe` (Wave 1, decision-recording)** lands before
- **`ae6635d` (Wave 2, code change / test rewrite)**

AC1 and AC7 satisfied: the rationale commit precedes the
`#[should_panic]`-removing commit.

### Path B rationale (transcribed from story 014)

**(a) Where the upstream bounds-check now lives.** The sole caller of
`co_occupancy_offset` is `snapshot_co_occupancy_offsets` at
`client/src/presentation/board_rendering.rs:1888-1927`. That caller asserts
`index <= u8::MAX as usize` at lines 1917-1921 — the parameter type's own
invariant (`u8`) bounds the input domain of `co_occupancy_offset`. The
function itself is intentionally **total over `u8`**: every `u8` value maps
to a defined `f32` offset.

**(b) What `co_occupancy_offset(2, ..)` now returns.** The function
(`client/src/presentation/board_rendering.rs:1929-1938`) reads:

```rust
pub fn co_occupancy_offset(unit_index: u8, side_offset: f32) -> f32 {
    if unit_index > 1 {
        warn!(
            "co_occupancy_offset: unit_index {} out of range, clamping to 1",
            unit_index
        );
    }
    let index = unit_index.min(1);
    (f32::from(index) - 0.5) * side_offset
}
```

For `unit_index = 2` (and any `unit_index >= 2`), the function emits a
`warn!` diagnostic and clamps to `1`, returning `(1.0 - 0.5) * side_offset =
0.5 * side_offset`. For `side_offset = 8.0` (test call site), return value
is exactly `4.0`.

**(c) Why silent overflow is no longer a risk.** Two defensive layers:

1. **Diagnostic visibility.** `warn!` is routed through Bevy's `bevy::log`
   subscriber (env-filter `client=info` and above in dev; targeted log
   surfaces in test harnesses). 3+ allied units on a cell are observable in
   CI logs and developer consoles — the same visibility profile that the
   previous panic provided in debug builds, minus the crash.
2. **Clamp semantics.** The clamp deliberately overlaps the third+ unit
   with the second-unit visual offset. For an inherently 2-slot layout
   (TR-BR-007 co-occupancy visual offsets, `design/gdd/board-rendering.md`
   does NOT specify >2-co-occupant behaviour), overlap is the correct
   degradation.

**(d) Presentation-layer alignment.** ADR-021 (Presentation Layer
Architecture) puts authoritative state on the server snapshot; the
presentation layer renders a derived view. A panic in the snapshot-
rendering path would crash the client on a *visual* anomaly that does not
affect game state. Warn + clamp is the ADR-021-aligned non-fatal
degradation.

**(e) Historical disposition.** The panic-guard was intentionally replaced
with warn+clamp in commit `ac9305b07764038611f4a62e79c018e072d41002`
(2026-05-08), `fix(board_rendering): observer refactor + Pointer<Click>/Press
to On<> + co_occupancy clamp + BoardRenderingConfig threading + ADR-021
PresentationSet::MessageDrain`. PROMPT 750 D-5 ignored the test pending the
written design write-up; PROMPT 800 records the write-up here and updates
the test to match the now-canonical production disposition.

---

## Code-change diff summary

**No production code under `client/`, `server/`, `shared/` is modified by
this story.** The chosen Path B leaves the production function
`co_occupancy_offset` (and all of `client/src/presentation/board_rendering.rs`)
unchanged.

### `tests/unit/board_rendering/status_icons_test.rs` (Wave 2 / `ae6635d`)

- Removed `#[ignore = "PROMPT 750 D-5: ..."]` owner comment.
- Removed `#[should_panic(expected = "unit_index=2")]` (NOT a silent
  deletion — Path B rationale in story file committed in Wave 1 first per
  AC7).
- Renamed `test_cooccupancy_index_two_panics_with_offending_index` →
  `test_cooccupancy_index_two_clamps_to_second_slot_offset`.
- New assertions lock the canonical 2-slot layout and the >=2 clamp:
  - `co_occupancy_offset(0, 8.0) ≈ -4.0`
  - `co_occupancy_offset(1, 8.0) ≈  4.0`
  - `co_occupancy_offset(2, 8.0) == co_occupancy_offset(1, 8.0)` (the
    clamp invariant)
- Replaced the `#[ignore]` owner comment with a one-line in-source
  rationale + commit pointer (`d5053fe`) so future readers find the design
  decision via `git log`.

### Story file (Wave 1 / `d5053fe`)

- "Binary Design Decision" section: Path B box ticked; Path A explicitly
  marked NOT chosen with reference rationale.
- Status banner updated `Draft` → `In Progress` with PROMPT 800 reference.
- Authoring trail appended with PROMPT 800 decision-recording disposition.

### Files NOT modified (verbatim from forbidden scope)

- `production/sprint-status.yaml` — preserved.
- `production/sprints/sprint-12.md` — preserved.
- `production/stage.txt` — preserved (`Polish`).
- `production/session-state/*` — preserved.
- `production/gate-checks/*` — preserved (PROMPT 761 FAIL retained).
- `production/qa/qa-plan-sprint-12.md` — preserved.
- `client/`, `server/`, `shared/` source — preserved (Path B leaves
  production behaviour unchanged).
- Other tests under `tests/` — preserved.

---

## Verification — pre/post test counts

### `cargo test -p client --test board_rendering_status_icons_test --no-fail-fast`

Targeted test crate (post-Wave 2 only — pre/post identical scope):

```
test test_cooccupancy_index_two_clamps_to_second_slot_offset ... ok
test test_status_icons_sort_by_tier_duration_and_overflow ... ok
test test_snapshot_cooccupancy_offsets_allied_units_by_unit_id ... ok
test test_status_icon_global_x_inherits_cooccupancy_parent_offset ... ok
test test_tier_two_equal_duration_sorts_deterministically_by_status_key ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Post-Wave-2: **5 passed / 0 failed / 0 ignored**. The PROMPT 750 D-5
`#[ignore]` is no longer present on this test crate.

### `cargo test -p client --no-fail-fast`

Aggregate across all `client` test binaries (post Wave 2):

```
client passed=391 failed=0 ignored=4
```

- **Pre baseline** (Sprint 11 close-out, `origin/main@b5eef0d`): 5 ignored
  carried (Cluster B `#[ignore]` rows from D-5 triage).
- **Post Wave 2**: 4 ignored. The B4 row dropped per AC5.

### `cargo test --workspace --tests --no-fail-fast`

```
workspace passed=1130 failed=0 ignored=4
```

- **Pre baseline** (Sprint 11 close-out smoke 2026-05-13): 1129 passed / 0
  failed / 5 ignored.
- **Post Wave 2**: 1130 passed / 0 failed / 4 ignored.
- Net: **+1 passing**, **−1 ignored** (the rewritten B4 test now passes
  un-ignored). AC5 satisfied.

### `cargo fmt -p client -- --check`

Exit code 0. No formatting drift introduced.

### `git diff --check origin/main...HEAD`

Exit code 0. No whitespace / merge-conflict markers.

---

## No-silent-deletion audit (AC7)

`grep -rn "should_panic" tests/`:

```
tests/unit/board_rendering/plugin_scaffold_test.rs:77:#[should_panic(expected = "invalid lane=0")]
tests/unit/board_rendering/plugin_scaffold_test.rs:83:#[should_panic(expected = "invalid lane=6")]
tests/unit/board_rendering/plugin_scaffold_test.rs:89:#[should_panic(expected = "invalid cell=0")]
tests/unit/board_rendering/plugin_scaffold_test.rs:95:#[should_panic(expected = "invalid cell=9")]
```

Only the four `plugin_scaffold_test.rs` `#[should_panic]` markers remain.
Each is unrelated to `co_occupancy_offset` (they assert `BoardLayout` lane
/ cell bounds invariants — separate production-side panic-guards that
remain in force). The B4 `#[should_panic(expected = "unit_index=2")]`
removal is the only `#[should_panic]` removal under this commit set, and
its removal is preceded by `d5053fe` (the rationale commit).

---

## Acceptance Criteria verdict

| AC | Verdict | Notes |
|----|---------|-------|
| AC1 — Binary decision recorded before code change | PASS | `d5053fe` precedes `ae6635d` in story trail. |
| AC2 — Production-design write-up under Path A | N/A | Path A NOT chosen. |
| AC3 — Production rationale under Path B | PASS | (a)–(e) above. Committed in `d5053fe` before test rewrite `ae6635d`. |
| AC4 — Test un-`#[ignore]`d and passes under chosen path | PASS | `test_cooccupancy_index_two_clamps_to_second_slot_offset` passes; no `#[ignore]`, no `#[should_panic]`. |
| AC5 — Workspace ignored count drops by 1 | PASS | 1129/0/5 → 1130/0/4. |
| AC6 — PROMPT 750 D-5 owner comment removed | PASS | Replaced with one-line Path B + commit pointer. |
| AC7 — `#[should_panic]` NOT silently deleted | PASS | Rationale commit `d5053fe` precedes removal commit `ae6635d`. No other `#[should_panic]` dropped. |
| AC8 — Sprint 12 disposition preserved | PASS | `sprint-status.yaml`, `sprints/sprint-12.md`, `stage.txt` unchanged. |
| AC9 — Evidence document slot populated | PASS | This file. |

---

## Cross-links

- Triage source: `production/qa/evidence/sprint-11-ignored-d5-triage.md`
  (Cluster B4, row 86).
- QA plan section: `production/qa/qa-plan-sprint-12.md` "S11-TD-COOCCUPANCY-
  PANIC-GUARD-DECISION-001 (story 014) — Cluster B4".
- Story file: `production/epics/playable-client/story-014-cooccupancy-
  panic-guard-decision.md`.
- Decision-recording commit: `d5053fe`.
- Code-change commit: `ae6635d`.
- Production code (unchanged under Path B):
  `client/src/presentation/board_rendering.rs:1929-1938`
  (`co_occupancy_offset`) and
  `client/src/presentation/board_rendering.rs:1888-1927`
  (`snapshot_co_occupancy_offsets`, the sole caller).
- Historical disposition commit (panic-guard → clamp):
  `ac9305b07764038611f4a62e79c018e072d41002` (2026-05-08).

---

## What PROMPT 800 did NOT do

- Did NOT run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`.
- Did NOT touch `production/sprint-status.yaml`,
  `production/sprints/sprint-12.md`, `production/stage.txt`,
  `production/session-state/*`, or `.claude/scheduled_tasks.lock`.
- Did NOT retry PROMPT 761 Polish→Release gate-check.
- Did NOT modify Sprint 11 / Sprint 10 closeout artefacts.
- Did NOT modify any code under `client/`, `server/`, `shared/`. (Path B
  is test-only; production code remained at its post-`ac9305b` state.)
- Did NOT introduce any new `#[ignore]` markers.
- Did NOT delete any `#[should_panic]` attribute beyond the single B4 row
  whose rationale was committed first.
- Did NOT push `main`; pushed only `work/s11-cooccupancy-panic-guard-
  decision` worker branch.
- Did NOT merge to `main`.
