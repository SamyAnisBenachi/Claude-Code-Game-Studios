# PROMPT 1512 - Krosmaga Proxy Logical ID Map Stage 2

Status: COMPLETE

## Deliverables

- `design/assets/provenance/krosmaga-proxy-logical-id-map-stage2.md` --
  Stage 2 logical ID map extending Stage 1 beyond the 50 P1 `USE_DEV_PROXY`
  rows. Documentation/provenance only.
- `reports/PROMPT-1512-krosmaga-proxy-logical-id-map-stage2.md` -- this
  completion report.

## Coverage Totals

| Metric                                                    | Count |
|-----------------------------------------------------------|------:|
| Stage 2 newly mapped rows                                 |    13 |
| Of which: AMBIGUOUS-promoted (best candidate, manual review still required) | 12 |
| Of which: MISSING-promoted (single plausible candidate)   |     1 |
| Still ambiguous (deliberately unmapped)                   |    92 |
| Still missing (deliberately unmapped, CCGS-originals)     |    80 |
| No-art-needed-from-Krosmaga (Section C)                   |     9 |
| Manual-review-required Stage 2 rows                       |    13 |
| `lid_*` index rows cross-walked to existing Stage 1 paths |    16 |

Stage 1 totals are unchanged. Stage 2 only adds 13 new mapped rows; the
rest of the deltas are documentation (`lid_*` cross-walk, MISSING reasons).

## Scope Confirmation

- Documentation/provenance only.
- No Krosmaga assets copied into this repo.
- No edits to `assets/**`, `client/**`, `server/**`, `shared/**`,
  `Cargo.*`, sprint status, session-state, sprint plan, QA plan, stage
  paperwork, or Krosmaga source file.
- Every Stage 2 mapped row carries
  `source_class=licensed_krosmaga_dev_proxy / workflow_status=needed /
  release_class=dev_only` per the schema.
- Stage 2 row confidence scores are deliberately <=60 to signal that
  manual review remains a precondition for any dev-pack materialization.

## Why The Remaining Rows Stay Unmapped (Summary)

- Most still-ambiguous PROMPT-1258 rows require UX-locked panel shape /
  nine-slice geometry / button-state sets a single Krosmaga sprite does
  not deliver -- promoting them would mislead downstream consumers.
- Result-screen chrome (`ASSET-211..214`) is still UX-blocked per
  `logical-id-index.md`.
- Per-card illustrations beyond Stage 1's P1 wave require 1:1 manual
  review per card.
- CCGS-original board mechanics (lanes, hidden objectives, unit-base
  shadow strips, facedown traps, lane-wash overlays) have no 1:1
  Krosmaga analogue (see Stage 2 doc Section C).
- Per-event SFX disambiguation requires listening to WAVs; static index
  scan cannot promote them safely beyond shared-source candidates.

Full table-level breakdown lives in
`design/assets/provenance/krosmaga-proxy-logical-id-map-stage2.md`.

## Validation

- Path allowlist review: only `design/assets/provenance/` and
  `reports/` files written.
- `git diff --check` clean.
- Markdown rendered/parsed; no embedded `|` outside cells, no JSONL/CSV
  written so the parseable-rows check was a no-op.
- No Cargo run.

## Git

- Worktree: `D:/tmp/wt-1512`
- Branch: `worker/prompt-1512-krosmaga-proxy-id-stage2`

1512: KROSMAGA-PROXY-LOGICAL-ID-MAP-STAGE-2: COMPLETE
