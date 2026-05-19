# PROMPT 1483 - Krosmaga Proxy Logical ID Map Stage 1

Status: COMPLETE

## Deliverables

- `design/assets/provenance/krosmaga-proxy-logical-id-map-stage1.md` - Stage 1 logical ID map for the 50 high-value `USE_DEV_PROXY` rows from PROMPT-1258.
- `reports/PROMPT-1483-krosmaga-proxy-logical-id-map-stage1.md` - this completion report.

## Coverage Totals

| Metric | Count |
|---|---:|
| Stage 1 mapped rows | 50 |
| Exact matches | 6 |
| Good matches | 44 |
| Rows requiring conversion/materialization | 50 |
| Source ambiguous rows intentionally unmapped | 104 |
| Source missing rows intentionally unmapped | 81 |

## Scope Confirmation

- Documentation/provenance only.
- No Krosmaga assets copied into this repo.
- No `assets/**`, `client/**`, `server/**`, `shared/**`, `Cargo.*`, sprint status, session-state, sprint plan, QA plan, or Krosmaga source file edits.
- Every mapped row is explicitly dev-only: `source_class=licensed_krosmaga_dev_proxy`, `workflow_status=needed`, `release_class=dev_only`.

## Top Blocked / Ambiguous Assets

The Stage 1 map leaves all ambiguous and missing rows unmapped. Highest-signal blocked groups from PROMPT-1258 are rarity gems/panel chrome, many custom Bevy UI materials, board cell/chrome variants without direct art matches, most audio rows beyond the four timing proxies, and result/modal chrome that is still UX-dependent.

Concrete examples are included in `design/assets/provenance/krosmaga-proxy-logical-id-map-stage1.md` under `Ambiguous / Missing Rows Explicitly Not Mapped`.

## Validation

- Static file/path review only, per prompt.
- Used existing reports/manifests first; no recursive scan of the Krosmaga bank.
- No Cargo was run.

## Git

- Worktree: `D:\Tmp\ccgs-prompt-1483`
- Branch: `work/krosmaga-proxy-logical-id-map-stage1-1483`

1483: KROSMAGA-PROXY-LOGICAL-ID-MAP-STAGE1: COMPLETE
