# Story 025: S18-PLACEMENT-DRAG-CURSOR-COORD-SPACE-001 -- Project Pointer<Move> Into World-Space Before Cell Math (R1.b Close)

> **Epic**: Hand UI
> **Story ID**: `S18-PLACEMENT-DRAG-CURSOR-COORD-SPACE-001`
> **Status**: Draft -- Sprint 18 candidate / retro paperwork; NOT activated
> **Layer**: Presentation -- Hand UI drag pipeline (`client/src/ui/hand/mod.rs`)
> **Type**: Logic + Integration (cursor coordinate-space projection + drag-to-cell tests)
> **Authored**: 2026-05-18 by PROMPT 1296 (landed-paperwork story-authoring wave)
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db` (PROMPT 1285 Sprint 18 plan draft)
> **Implementing PROMPT**: 1210 -- `dev-story(s18-placement-drag-cursor-coord-space): project Pointer<Move> into world-space before cell math`
> **Implementing commit**: `c61bab3`
> **Source audit**: PROMPT 1287 §2 (already-landed inventory); PROMPT 1127 §R1.b (cursor coordinate-space mismatch); PROMPT 1201 HUNT-1201-11; PROMPT 1203 B-1203-PLA-01

---

## Status / No-Claim Banner

This story is a **retro paperwork stub** authored by PROMPT 1296 covering
work that **already landed on `origin/main` at `c61bab3`**. It exists
so that `/story-done` paperwork has a concrete target after Sprint 18
activation. PROMPT 1296 makes **no** code, test, Cargo, CI, sprint,
QA, or session-state mutations.

Sprint 18 is **NOT activated** by this authoring run. All standard
non-claims preserved verbatim. R1 (drag pipeline dead in shipped
build, `ui_picking` feature gate) was a **separate** repair and is
not claimed closed by this story; only the R1.b cursor coord-space
sub-finding is in scope here.

---

## Source Finding

**PROMPT 1127 §R1.b**: cursor coordinate-space mismatch. The drag
system consumed `Pointer<Move>` events in their native (logical /
viewport / scale-factor-adjusted) coordinate space and fed them
directly into `BoardLayout::world_to_cell`, which expects
world-space. The result: cell-mapping was correct at default DPI on
`1920x1080` and wrong on every other viewport / scale-factor combo,
producing placement ghosts that lagged or skipped cells.

PROMPT 1210 (`dev-story`) inserted an explicit projection from the
`Pointer<Move>::pointer_location.position` through the active
`Camera`/`Window` transform stack into world-space coordinates
**before** the cell-math step. This isolated the projection in one
place so future viewport / DPI changes do not regress the drag path.

---

## Landed Evidence (commit `c61bab3`, PROMPT 1210)

Files touched by the implementing commit:

| Path | Role |
|------|------|
| `client/Cargo.toml` | Cargo wiring for the new test target. |
| `client/src/ui/hand/mod.rs` | Cursor-to-world projection inserted before cell math. |
| `tests/integration/hand-ui/hand_ui_drag_cursor_world_projection_test.rs` (NEW) | 346 LOC integration test exercising multi-viewport projection invariants. |
| `tests/integration/hand-ui/hand_ui_drag_end_non_instant_test.rs` | Updated to use the projected coordinate path. |
| `tests/integration/hand-ui/placement_staged_disclosure_harness.rs` | Harness updated. |
| `tests/integration/hand-ui/hand_ui_drag_to_board_cell_test.rs` | Updated assertions. |
| `tests/integration/hand-ui/placement_staged_disclosure_accessibility_test.rs` | Harness consumer update. |
| `tests/integration/hand-ui/placement_unstaging_test.rs` | Harness consumer update. |
| `tests/unit/hand-ui/placement_drag_highlights_test.rs` | Coverage extended. |
| `tests/unit/hand-ui/placement_instant_staging_test.rs` | Coverage extended. |

---

## Acceptance Criteria (evidence-binding, closure-oriented)

- [ ] **AC1 -- Projection in one place**:
  `client/src/ui/hand/mod.rs` contains exactly one cursor-to-world
  projection helper consumed by the drag pipeline; no `Pointer<Move>`
  consumer fed directly into `BoardLayout::world_to_cell` without
  passing through the helper.
- [ ] **AC2 -- Multi-viewport integration PASS**:
  `tests/integration/hand-ui/hand_ui_drag_cursor_world_projection_test.rs`
  PASSES at the Sprint 18 activation tip, asserting that the same
  logical pointer position maps to the same world cell at `1280x720`,
  `1366x768`, and `1920x1080` viewports (and across the DPI / scale
  factors exercised by the test).
- [ ] **AC3 -- Drag-to-cell integration PASS**:
  `tests/integration/hand-ui/hand_ui_drag_to_board_cell_test.rs` and
  the accessibility / unstaging harness tests remain green.
- [ ] **AC4 -- ADR-021 plugin-order preserved**: `HandUiPlugin`
  remains sub-plugin #3 inside `PresentationPlugin`; `HAND_UI_ENTITY_COUNT`
  unchanged (this story only changes the math path, not the entity
  set).
- [ ] **AC5 -- ADR-002 preserved**: client-side projection only; no
  new C2S/S2C protocol surface; `shared/src/protocol.rs` diff empty.
- [ ] **AC6 -- HUNT-1201-11 closed**: PROMPT 1201 §HUNT-1201-11 cited
  the cursor coord-space bug; this story's evidence is the closure.
  R1 (drag pipeline dead in shipped build) remains a **separate**
  finding; this story does NOT claim R1 closure.
- [ ] **AC7 -- /dev-story disposition**: **NOT REQUIRED** if AC1..AC6
  remain satisfied on `origin/main` at the Sprint 18 activation tip.
  If regression, `/story-readiness` MUST return NEEDS_WORK; follow-on
  implementation required before closure.

---

## Out of Scope

- R1 `bevy_picking` feature-gate / shipped-build drag-pipeline-dead
  repair. Separate prompt.
- Mana preview during drag (story 022, separate landed work).
- Idle-hand playable affordance (story 023, separate landed work).
- Placement submission rejection feedback (story 027, separate landed
  work).
- AUDIT-1076-02 / AUDIT-1076-03 server-side placement loss. Server-
  side; out of host module.
- Sprint 18 activation, stage advance, gate-check retry.

---

## Authoring Trail

- 2026-05-18 -- PROMPT 1296 -- Retro paperwork stub authored against
  `origin/main@1345c6b`. Files touched: this file (NEW) and
  `production/epics/hand-ui/EPIC.md` (table row added). Implementation
  landed via PROMPT 1210 at `c61bab3` prior to this authoring; this
  stub does not re-author or alter that work.
