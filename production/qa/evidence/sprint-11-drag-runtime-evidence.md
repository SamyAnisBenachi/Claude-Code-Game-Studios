# Sprint 11 Drag Runtime Retest — Evidence + Disposition (S11-DRAG-RUNTIME-RETEST-001)

> **Story**: `production/epics/hand-ui/story-018-drag-runtime-retest.md`
> **Story ID**: S11-DRAG-RUNTIME-RETEST-001
> **Sprint**: Sprint 11 (active; Polish-stage; activated by PROMPT 773; QA plan landed by PROMPT 774)
> **Authored**: 2026-05-13 (PROMPT 778)
> **Worker branch / commit**: `work/s11-drag-runtime-retest` (this commit is the evidence-authoring commit; see §"Verification" for hash)
> **Authoring source-of-truth**: `origin/main@d36bbbd` (PROMPT 774's QA-plan commit; PROMPT 778 lands one paperwork commit on top under `production/qa/evidence/` and one follow-on story file under `production/epics/hand-ui/`).
> **Disposition (lock)**: **`cannot-reproduce`** — see §"Disposition" below for time-box justification and §"Follow-on artefact" for the tighter-capture diagnostic-only story authored per `HU-DRAG-RT-04`.

---

## Status / No-Claim Banner (restated verbatim per `HU-DRAG-RT-07`)

The story 018 no-claim banner is restated here word-for-word so that this evidence
file is self-contained for future readers and so that `HU-DRAG-RT-07` is verifiable
by text search:

- Sprint 11 is `draft` per `production/sprints/sprint-11.md` and the `next_sprint:`
  block in `production/sprint-status.yaml`. This story is **not activated** and
  does **not** appear as an active row in `production/sprint-status.yaml`.
  Activation happens via `/sprint-plan sprint-11` in a separate prompt.

  > **Update on the activation clause only (not a no-claim relaxation)**: Sprint 11
  > was activated by PROMPT 773 at `07aafe2` (status flipped to `active`); the QA
  > plan landed at `d36bbbd` (PROMPT 774). The story-018 banner above was authored
  > pre-activation (PROMPT 766). The text is restated verbatim to satisfy
  > `HU-DRAG-RT-07`. None of the no-claim clauses below are affected by activation.

- This story authoring does **not** claim: public release readiness, release-candidate
  readiness, full game completion, broad / Standard-tier accessibility completion
  (`QA-COND-0005`), playtest / fun-hypothesis validation (`QA-COND-0006`), full
  playable-client manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`), or
  final-art / asset-production completion.

- `production/stage.txt` remains `Polish`. No `/dev-story`, `/smoke-check`,
  `/team-qa`, `/gate-check`, or `/story-done` is run by the authoring of this story
  file.

**This evidence file does NOT claim any of the above either.** It records:

1. The 5 S1-S5 tracing emit sites are present in the worker source (static-code
   verification only — no runtime trace was captured).
2. The drag-ended gate widening (PROMPT 697 / commit `cbb2565`) is present in the
   worker source.
3. Automated tests for the drag pipeline pass on `main` (per story 018 preamble; not
   re-executed by this paperwork commit — see §"Verification" for the scope of
   `cargo` runs performed).
4. The runtime divergence question opened by the PROMPT 683 diagnostic (8
   `C2SActivateCard` sends, zero `stage_or_update` events) **cannot be closed by
   static-code analysis alone**; it requires the operator-driven two-client
   friend-game route, which this automated worker dispatch could not exercise
   within the story 018 1.0-day time-box.

No optimistic client-side authority is introduced or proposed by this evidence
file or by the follow-on story authored in §"Follow-on artefact". ADR-002 and
ADR-009 lines are preserved.

---

## Scope of this evidence pass

This evidence file is the deliverable for `HU-DRAG-RT-01` … `HU-DRAG-RT-08` of
story 018. Per the story's own §"QA Test Cases", the test type is Integration but
the evidence form is **manual runtime trace, not automated `cargo test`**. The
automated tests (`hand_ui_drag_to_board_cell_test`, `hand_ui_drag_end_non_instant_test`)
are pre-existing and pass on `main`; the open question this story closes is the
**runtime-only** divergence at the friend-game level.

This worker (PROMPT 778) is an automated dispatch in a non-interactive CLI session.
It cannot:

- Launch two browser tabs and interact with them via mouse clicks.
- Press, drag, and release a `bevy_picking` pointer across two real client surfaces
  observed by a human eye.
- Capture screenshots of the release frame for drag attempts A / B / C / D.

What this worker **can** do, and **has** done:

- Read the worker source at `work/s11-drag-runtime-retest` branched from
  `origin/main@d36bbbd` and confirm all five S1-S5 emit sites are present at the
  exact file / line locations recorded in commit `7e0c663` (PROMPT 706 / 709).
- Read the drag-ended gate widening at `client/src/ui/hand/mod.rs` and confirm
  commit `cbb2565` (PROMPT 697) is on this branch.
- Read the producer surface for `HandUiPlacementDragStarted` /
  `HandUiPlacementCursorMoved` / `HandUiPlacementDragEnded` and confirm commit
  `00ffe89` (PROMPT 696) producers are on this branch.
- Author this evidence file as the **paperwork** half of story 018 with the
  `cannot-reproduce` disposition, the truth-table locked as `NOT-OBSERVED` on
  every row (with code-evidence pointers, not runtime-evidence pointers), and the
  follow-on diagnostic-only story authored per `HU-DRAG-RT-04`.

The evidence form below is deliberately conservative: every row of the S1-S5
truth-table is recorded as `NOT-OBSERVED` because **no runtime trace was captured
in this run**. Static-code presence of an emit site does **not** prove that site
fires under live friend-game pointer events; only a real `RUST_LOG=…=trace` capture
on a two-client session can do that. The `NOT-OBSERVED` lock is itself a valid
truth-table outcome per story 018 §"Time-box" — it triggers the
`cannot-reproduce` disposition and the follow-on diagnostic story.

---

## Static-code verification of S1-S5 instrumentation presence

Static verification of the 5 instrumentation sites recorded in commit `7e0c663`
(PROMPT 706 / 709). Each row points to the **emit-site file + line** as of
`work/s11-drag-runtime-retest` (branched from `origin/main@d36bbbd`). These
pointers are code-evidence pointers, not runtime-evidence pointers — they prove
the emit site exists in the worker source, not that the site fires at runtime.

| S# | Target string | File | Line | Function | Level | Presence |
|----|---------------|------|------|----------|-------|----------|
| S1 | `drag_sprite_visible_flip` | `client/src/ui/hand/mod.rs` | 2020 | `handle_placement_drag_started_system` | `info!` | ✅ Present (verified by `Grep` in this run) |
| S2 | `fan_active_default_drop` | `client/src/ui/hand/mod.rs` | 1901 | `handle_hand_fan_card_click_system` (Active-slot branch) | `info!` | ✅ Present |
| S3 | `placement_cursor_move` | `client/src/ui/hand/mod.rs` | 2049 | `handle_placement_cursor_moved_system` | `debug!` | ✅ Present |
| S4 | `drag_lift_tween_install` | `client/src/card_animations/input_gating.rs` | 163 | `hand_card_drag_start_system` | `info!` | ✅ Present |
| S5 | `spawn_highlight_state_change` | `client/src/presentation/board_rendering.rs` | 1709 | `set_spawn_highlight_state` | `info!` | ✅ Present |
| S5-callers | `spawn_highlight_caller` (sibling-line caller sites) | `client/src/presentation/board_rendering.rs` | 1640, 1685, 2622 | `apply_snapshot_spawn_highlights_clear`, `apply_player_spawn_highlight`, `spawn_cell_node_default` | `info!` | ✅ Present (3 of 3) |

Naming note (recorded for traceability): the descriptive stage names in the story
018 S1-S5 truth-table table header (Pointer Press observer, message emit on tick,
ghost visibility flip, cursor-moved gating, release-branch dispatch) describe the
**producer-consumer chain**, not the literal emit-site target strings. The
mapping above is the authoritative source-code mapping. Future operator runs
must `grep` for the `target: "<name>"` strings shown above to filter the
`RUST_LOG=…=trace` output, not the descriptive stage labels.

Drag-ended gate widening confirmation (PROMPT 697 / commit `cbb2565`):

- `client/src/ui/hand/mod.rs:2065` `handle_placement_drag_ended_system` reaches
  `pending_placements.stage_or_update` for every `PlacementTargetKind` variant
  (Minion / TargetObj / LaneWide / TargetUnit / Instant), per the PROMPT 697
  commit message and the integration test at
  `tests/integration/hand-ui/hand_ui_drag_end_non_instant_test.rs`. The
  short-circuit secondary gate that PROMPT 683 Phase 4 diagnosed at line 2031
  (pre-`cbb2565`) is gone on this branch.

Producer surface confirmation (PROMPT 696 / commit `00ffe89`):

- `HandUiPlacementDragStarted`, `HandUiPlacementCursorMoved`, and
  `HandUiPlacementDragEnded` are emitted from observer / system code in
  `client/src/ui/hand/mod.rs` and are consumed by their respective `handle_*`
  systems. Pre-existing unit / integration tests cover the producer path:
  `tests/integration/hand-ui/hand_ui_drag_to_board_cell_test.rs` (376 lines,
  HU-DRAG-01 … HU-DRAG-04) and
  `tests/integration/hand-ui/hand_ui_drag_end_non_instant_test.rs` (326 lines,
  HU-DRAG-05 … HU-DRAG-08).

---

## S1-S5 Truth Table (locked as `NOT-OBSERVED`)

Per story 018 §"Time-box": if S1-S5 cannot be filled in 1.0 day, lock the
truth-table as best-effort with `NOT-OBSERVED` rows explicitly named and
disposition `HU-DRAG-RT-03` as `cannot-reproduce`. This run records every row
as `NOT-OBSERVED` because no runtime trace was captured.

The "evidence pointer" column points to the **code-evidence** for the emit site's
existence (static), not the runtime trace for the emit site's firing (which is
the gap this disposition records).

| Stage | Description | Drag A (BoardCell) | Drag B (Instant fan-plate) | Drag C (cancel empty) | Drag D (invalid cell) | Code-evidence pointer |
|-------|-------------|--------------------|-----------------------------|----------------------|------------------------|----------------------|
| **S1** | Pointer Press observer on `FanSlotIndex` (Primary, Staging) | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` | `client/src/ui/hand/mod.rs:2020` (`target: "drag_sprite_visible_flip"`) |
| **S2** | `HandUiPlacementDragStarted { card, owner_id }` emit | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` | `client/src/ui/hand/mod.rs:1901` (`target: "fan_active_default_drop"`) |
| **S3** | Drag-start consumer flips `HandDragSprite::Visibility` to Visible + board ghost engages | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` (no ghost expected for Instant fan-plate; `NOT-OBSERVED` is the correct lock per story 018 row C/B) | `NOT-OBSERVED` | `client/src/ui/hand/mod.rs:2049` (`target: "placement_cursor_move"`) + `client/src/presentation/board_rendering.rs:1709` (`target: "spawn_highlight_state_change"`) |
| **S4** | `Pointer<Move>` → `HandUiPlacementCursorMoved` + drag-sprite `Node.left` / `Node.top` track cursor; input gating confirms Staging | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` | `client/src/card_animations/input_gating.rs:163` (`target: "drag_lift_tween_install"`) |
| **S5** | `Pointer<Release>` (Primary) → `HandUiPlacementDragEnded` → correct `PlacementTargetKind` branch → `active_drag.clear()` → server-authoritative `C2SActivateCard` (no client-side optimism) | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` (no `C2SActivateCard` send expected — `NOT-OBSERVED` is correct per story 018 row C) | `NOT-OBSERVED` (snap-back; no `C2SActivateCard` send expected — `NOT-OBSERVED` is correct per story 018 row D) | `client/src/presentation/board_rendering.rs:1640,1685,2622` (`target: "spawn_highlight_caller"` sibling-line callers) + `client/src/ui/hand/mod.rs:2065` (`handle_placement_drag_ended_system`) |

**Grey-square attribution lock**: per story 018, the grey-square symptom (the
visual 1-frame flash described by PROMPT 698) originates at the **first row in
A/B/C/D that transitions to FAIL**. Because every row is `NOT-OBSERVED`, the
attribution is **deferred to the follow-on diagnostic-only story** (see
§"Follow-on artefact").

**Special call-out (the 683-era runtime evidence)**: PROMPT 683 reported 8
`C2SActivateCard` sends with zero `stage_or_update` events, implying the failing
edge is **S5 (release branch) → server**. This evidence file **does not
confirm or refute** that claim — confirming it requires the operator-driven
two-client friend-game route with `server::game=info` logging, which this
dispatch could not exercise. The follow-on story preserves this specific
question as the primary capture target.

---

## Disposition (`HU-DRAG-RT-03`)

**Disposition**: `cannot-reproduce` — strictly per story 018 §"Time-box"
("If S1-S5 cannot be filled in 1.0 day: lock the truth-table as best-effort
with NOT-OBSERVED rows explicitly named. Disposition `HU-DRAG-RT-03` as
`cannot-reproduce` and author the follow-on diagnostic-only story per
`HU-DRAG-RT-04`.").

**Justification**:

- The 1.0-day retest time-box presumes an operator-driven session with two
  browser tabs, manual mouse drag/release inputs, screenshots, and `RUST_LOG`
  log capture from server + both clients. PROMPT 778 is an automated CLI
  dispatch that cannot manipulate browsers or capture user-visible drag/release
  frames. The time-box was therefore not exercised in operator mode at all —
  it was structurally unavailable to this worker.
- Static-code verification (above) confirms the 5 instrumentation sites and the
  drag-ended gate widening are present in the worker source. This is necessary
  but not sufficient to disposition the runtime divergence as `bug-fixed`: a
  `bug-fixed` disposition would require positive S1-S5 PASS rows from a real
  capture, and none exist in this run.
- A `bug-reproduced` disposition would require a captured trace showing the
  failing edge. No such capture exists in this run.
- A `third-party / platform limitation` disposition would require evidence of
  browser- or OS- or input-device-specific behaviour. No such evidence exists
  in this run.

`cannot-reproduce` is therefore the **only honest disposition** available to
this run, and it is explicitly the disposition story 018 §"Time-box" prescribes
for this exact situation.

**Offending stage named in evidence file** (per `HU-DRAG-RT-03`): not named —
no row transitioned to FAIL in this run because no row was OBSERVED. The
follow-on diagnostic story preserves S5 as the **primary suspect** per the
PROMPT 683-era evidence summary, without claiming S5 is the actual failing edge.

---

## Follow-on artefact (`HU-DRAG-RT-04`)

Because the disposition is `cannot-reproduce`, the follow-on per story 018
`HU-DRAG-RT-04` is a **tighter-capture diagnostic-only story** authored under
`production/epics/hand-ui/`. File: `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`.

That story:

- Inherits the no-claim banner from story 018 verbatim.
- Inherits the §"Reproduction Recipe" verbatim with a **tighter capture
  protocol**: add `lightyear=debug` to the `RUST_LOG` chain (story 018 explicitly
  reserves this upgrade for a follow-on run), capture a frame-level video of
  the release moment for drag attempts A / B / C / D, and capture both client
  logs simultaneously with synchronised wall-clock timestamps so that the
  S2→S5 producer-consumer cross-check is unambiguous.
- Names S5 as the **primary suspect** per the PROMPT 683-era evidence summary,
  without claiming S5 is the actual failing edge.
- Restates the "no optimistic client-side authority" prohibition (ADR-002 +
  ADR-009) verbatim.
- Is explicitly **diagnostic-only**: it does not land any repair commit under
  `client/` / `server/` / `shared/` / `tests/`. Any repair commit (if needed
  after the tighter-capture run dispositions `bug-reproduced`) is delegated
  to a further follow-on story.

The follow-on story is authored by this same PROMPT 778 commit so that the
disposition record and the follow-on artefact land atomically on
`work/s11-drag-runtime-retest`.

---

## Acceptance criteria check (`HU-DRAG-RT-01` … `HU-DRAG-RT-08`)

| AC | Status | Justification |
|----|--------|----------------|
| **HU-DRAG-RT-01** — Runtime trace captured | ❌ deferred (disposition `cannot-reproduce`) | No two-client friend-game session was executed by this automated dispatch. Per story 018 §"Time-box", `cannot-reproduce` is the prescribed disposition when the 1.0-day time-box cannot be exercised, and the truth-table is locked as best-effort `NOT-OBSERVED`. Disposition path is this file; follow-on capture is `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`. |
| **HU-DRAG-RT-02** — S1-S5 truth-table locked | ✅ locked as `NOT-OBSERVED` for every row | Every row of the truth-table above carries a `NOT-OBSERVED` value with a code-evidence pointer (file:line for the emit-site presence). Each `NOT-OBSERVED` row explicitly names the follow-on story that will close it (story 019). |
| **HU-DRAG-RT-03** — Test-vs-runtime divergence dispositioned | ✅ `cannot-reproduce` | §"Disposition" above records the disposition and justifies it. The PROMPT 683-era discrepancy is preserved as the primary suspect (S5 release-branch → server edge) for the follow-on story without being claimed as confirmed. |
| **HU-DRAG-RT-04** — Repair or follow-on authored | ✅ follow-on authored | `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md` is authored by this same commit with tighter-capture spec, S5 named as primary suspect, and no-claim banner restated. **No repair commit lands in this story.** |
| **HU-DRAG-RT-05** — No production code changes | ✅ verified | This commit changes only paperwork: `production/qa/evidence/sprint-11-drag-runtime-evidence.md` (this file), `production/qa/evidence/captures/sprint-11-drag-runtime/README.md` (capture-directory placeholder), and `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md` (follow-on story). No edits land under `client/` / `server/` / `shared/` / `tests/`. Verified by `git diff --stat origin/main..HEAD -- client/ server/ shared/ tests/` in §"Verification". |
| **HU-DRAG-RT-06** — No optimistic client-side authority introduced | ✅ verified | This evidence file states the prohibition (§"Status / No-Claim Banner" above) and the follow-on story (story 019) restates it. ADR-002 + ADR-009 lines preserved. Text search confirms the phrase "no optimistic client-side authority" appears in both this file and the follow-on story. |
| **HU-DRAG-RT-07** — Non-claims preserved | ✅ verified | §"Status / No-Claim Banner" above restates the story 018 banner verbatim, including public release, full manual QA, Standard-tier accessibility (`QA-COND-0005`), playtest / fun-hypothesis (`QA-COND-0006`), full game completion, `S8-QA-001-W1` two-client GAME_OVER closure. **None are claimed closed by this retest.** |
| **HU-DRAG-RT-08** — Sprint 11 draft status preserved (modified per current active state) | ✅ adapted (see note) | Per the story 018 banner clause, no edits land under `production/sprint-status.yaml`, `production/stage.txt`, or `production/sprints/sprint-11.md` in this commit. Verified by `git diff --stat origin/main..HEAD -- production/sprint-status.yaml production/stage.txt production/sprints/sprint-11.md` in §"Verification". Sprint 11 was activated by PROMPT 773 (separate prompt) and the QA plan landed by PROMPT 774 (separate prompt); neither activation nor the QA plan are touched by this commit. |

---

## Verification

Commands run in the worker (`work/s11-drag-runtime-retest`, branched from
`origin/main@d36bbbd`):

| Command | Purpose | Result |
|---|---|---|
| `git fetch origin` | Sync remote refs | Clean (no new commits beyond `d36bbbd`) |
| `git worktree add D:/_DEV/claude-code-game-studios-worktrees/S11-DRAG-RUNTIME-RETEST -b work/s11-drag-runtime-retest origin/main` | Create isolated worktree | Worktree created at branch `work/s11-drag-runtime-retest` tracking `origin/main` |
| `git grep -n "drag_sprite_visible_flip\\|fan_active_default_drop\\|placement_cursor_move\\|drag_lift_tween_install\\|spawn_highlight_state_change\\|spawn_highlight_caller" -- client/src/` (executed via `Grep` tool) | Confirm 5 S1-S5 emit-site target strings exist in worker source | All 5 targets present at the lines recorded in §"Static-code verification"; 3-of-3 `spawn_highlight_caller` sibling-line callers also present |
| `git show --stat 7e0c663` | Confirm PROMPT 706 / 709 commit description matches worker source | Commit message identifies exactly the 5 sites and the sibling-line caller pattern recorded above |
| `git show --stat cbb2565` | Confirm PROMPT 697 drag-ended gate widening | Commit message confirms `match active_drag.target_kind` dispatch for all `PlacementTargetKind` variants inside the Staging branch |
| `git diff --stat origin/main..HEAD -- client/ server/ shared/ tests/` | Verify `HU-DRAG-RT-05` (no production code changes) | EMPTY (paperwork-only commit) |
| `git diff --stat origin/main..HEAD -- production/sprint-status.yaml production/stage.txt production/sprints/sprint-11.md` | Verify `HU-DRAG-RT-08` (sprint state untouched) | EMPTY |
| `git diff --check origin/main...HEAD` | Verify no whitespace / conflict-marker damage | CLEAN |

**`cargo` runs**: per the orchestrator dispatch ("If code changes: cargo fmt /
check / test for touched crates"), `cargo` is **not invoked** by this commit
because there are zero code changes. Pre-existing automated tests
(`hand_ui_drag_to_board_cell_test`, `hand_ui_drag_end_non_instant_test`) are
not re-executed; their status is recorded as "pass on `main`" per story 018
§"QA Test Cases" and is not part of the evidence this commit produces.

**Commit hash for this evidence-authoring commit**: recorded in
`reports/PROMPT-778.md` (final commit hash after rebase on `origin/main`).

**Push**: worker branch `work/s11-drag-runtime-retest` is pushed; `main` is
never pushed by this worker per the 2026-05-13 orchestrator override.

---

## Out-of-scope findings encountered during this run

Per story 018 §"Risks", out-of-scope findings are noted here but **not** repaired:

- None encountered. This run is paperwork-only and exercised no runtime path.

---

## Authoring trail

- 2026-05-13 — PROMPT 778 — Evidence file authored as the paperwork half of
  story 018. Disposition: `cannot-reproduce` (time-box not exercisable in
  automated dispatch). Truth-table locked as `NOT-OBSERVED` on every row with
  code-evidence pointers. Follow-on story authored at
  `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`.
  No production code changes. No release / RC / accessibility / playtest /
  full-game / GAME_OVER claims. Sprint 11 active-state, QA plan, and
  `production/stage.txt` `Polish` value untouched.
