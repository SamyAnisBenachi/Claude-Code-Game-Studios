# PROMPT 2034 — User Live UI/UX Bug Ledger Backfill

Date: 2026-05-28
Branch basis: `origin/main@28482bd5` (strict-FF)
Scope: ledger/backfill only — no client/server/test code changes.

## Goal

The 2026-05-28 user-live play session surfaced UI/UX bugs that the prior
audit/repair wave (PROMPT 2024-2029, PROMPT 2033) did not durably capture in
the unplayable bug register. This prompt adds stable bug IDs and repair-worker
mapping for those bugs without renumbering any existing P0/P1/V1/T entries.

## Scope of edits

Only two paths touched, both in the explicit allow-list:

- `production/qa/bugs/current-unplayable-bug-register-2026-05-28.md`
- `reports/PROMPT-2034-user-live-ui-ux-bug-ledger-backfill.md` (this file)

No edits under `client/**`, `server/**`, `tests/**`, `production/sprint-status.yaml`,
`production/session-state/**`, `production/sprints/**`, `production/stage.txt`,
or any unrelated report.

## New bug IDs

### P0 (added; existing P0-001..P0-012 untouched)

- **P0-013** — Placement drag preview does not follow the cursor.
- **P0-014** — Placed cards never become visible units on the board (user-facing
  facet of the P0-007 board-empty cascade).

### V1 (added; existing V1-001..V1-008 untouched)

- **V1-009** — Hand fan spreads cards across the full hand rectangle instead of
  fanning.
- **V1-010** — Card and shop slot images render as `?`/`[]` placeholders
  (superset of V1-001 all-black class card).
- **V1-011** — Card stats and labels missing or unreadable.
- **V1-012** — First-round shop visuals are broken (missing art/prices,
  empty-but-clickable slots).
- **V1-013** — Resolution phase has no visible combat presentation.
- **V1-014** — Global bevy_ui anchoring/positioning is unreliable across
  window sizes.
- **V1-015** — Valid placement cells are not highlighted during drag.

### UX (new section; UX-001..UX-012)

User-facing acceptance handles for the live observations. Each UX-* row links
the technical P0/V1/P1/T row(s) that explain the cause. Repair workers should
treat the UX-* row as the user-visible acceptance criterion.

- **UX-001** — Drag preview tracks cursor.
- **UX-002** — No invalid-cell rejection at confirm without a pre-confirm
  visual cue.
- **UX-003** — Legal placement cells visibly highlighted during drag.
- **UX-004** — Drop produces a placement feedback cue.
- **UX-005** — Hand cards fan rather than spreading across the rectangle.
- **UX-006** — Card faces render real art, not `?`/`[]` placeholders.
- **UX-007** — Card stats and labels are present and legible.
- **UX-008** — First-round shop renders correctly on the first DraftShop.
- **UX-009** — Placed cards visibly appear as units on the board.
- **UX-010** — Resolution shows visible attacks/damage/objective feedback.
- **UX-011** — UI anchors hold consistently across window sizes.
- **UX-012** — Prior PASS/SHIPPED labels must match what a user sees in a live
  two-client run.

## Repair Wave Mapping additions

The "Repair workers not yet launched" list now also covers:

- Placement drag/preview repair (P0-013, UX-001, UX-003, UX-004, V1-015).
- Placement legality preview / pre-confirm rejection feedback
  (P0-013, P1-005, V1-015, UX-002, UX-003).
- Board unit visibility repair (P0-014, UX-009) downstream of the P0-007
  board cascade but owning the visible-unit criterion.
- Hand fan layout (V1-009, UX-005).
- Card art and stats rendering (V1-001, V1-010, V1-011, UX-006, UX-007).
- First-round shop initialization/visual repair (V1-012, UX-008).
- Resolution/combat visualization (V1-013, UX-010).
- Global bevy_ui anchoring/responsive layout (V1-014, V1-004, UX-011,
  cross-checked with T-005/T-006).
- Audit/test label truthfulness backstop (UX-012, reinforcing T-020..T-033).

## Preservation

- All existing P0-001..P0-012, P1-001..P1-017, V1-001..V1-008, T-001..T-033
  IDs, titles, evidence cells, and current-status cells are unchanged.
- PROMPT 2024-2029 and PROMPT 2033 sections (Executive State, Autoplay
  Click/Window Findings, QA Evidence Truthfulness, Server Board/GameOver,
  Flow Matrix, Rules For Future Updates) are unchanged in content; only
  additive rows and an additive paragraph in Repair Wave Mapping were
  inserted.
- The Primary sources list was extended by one entry pointing to this report.

## Validation

- Branch reset to `origin/main@28482bd5` before any edits; strict-FF basis.
- Only two paths modified, both in the allow-list.
- `git diff --check` clean (no whitespace/conflict errors) — verified at
  commit time.
- No code under `client/`, `server/`, `tests/`, no production state/sprint
  files touched.

## Final line

2034: USER-LIVE-UI-UX-BUG-LEDGER-BACKFILL: SHIPPED
