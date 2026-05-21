# PROMPT 1593 — QA-PLAN-SNAPSHOT-FIELDS-MAINLAND-REFRESH-AFTER-1591

## Summary

Refresh of the QA-plan paperwork (PROMPT 1587) + QA-snapshot resolution-phase
observability fields (PROMPT 1586) integration stack onto the current
`origin/main` tip after PROMPT 1591 (tooling launcher + Krosmaga validator
coverage) advanced main from `945cbd71` to `c7cfc5a4`.

The PROMPT 1592 integration branch
`origin/integrate/qa-plan-snapshot-fields-1592 @ 9524da94` was strict-FF vs
the prior `origin/main @ 945cbd71` but became stale once PROMPT 1591 landed.
This refresh produces a new integration branch that is strict-FF vs the
current `origin/main @ c7cfc5a4`.

## Branch / Commit

- Refreshed branch (local + pushed): `integrate/qa-plan-snapshot-fields-1593`
- Tip commit (cherry-pick payload, pre-report): `7f78b553cafd65f0e3fb1b8a23cb63e14c8e62a8`
- Tip commit (this-report included, post-push): `7196bdad0e2bbb3efd0276a5ba51efd76352d726`
- Base: `origin/main @ c7cfc5a4a047686890f3a54f154f1e8929c1af70`
- Strict-FF check: `git merge-base --is-ancestor origin/main HEAD` → 0 (ancestor)
- Worktree: `D:/Tmp/wt-1593`

## Operation

Fresh worktree from `c7cfc5a4`, then cherry-pick of the four unique commits
from `origin/integrate/qa-plan-snapshot-fields-1592` (which itself layered
1586 + 1587 + 1590 + 1592 onto `945cbd71`). All four cherry-picks were clean
— there is zero file-level overlap between PROMPT 1591's payload (tooling +
launcher + reports) and the QA-plan/QA-snapshot payload carried here.

Cherry-pick order, in `git log --oneline c7cfc5a4..HEAD`:

```
7f78b553 PROMPT-1592 QA-plan + QA-snapshot fields mainland refresh after 1588 (1586+1587+1590 onto 945cbd71)
4f783dea PROMPT-1590 integration refresh report for QA-plan paperwork + QA-snapshot resolution-phase fields (1586+1587 onto origin/main 9be8827f)
d7142ef1 qa-plan-paperwork(s18): discharge sprint-18 §0.4 qa-plan-found stale-false correction (PROMPT 1587)
c7e517b0 PROMPT-1586 QA snapshot resolution-phase observability fields
```

The historical PROMPT-1590/1592 reports are carried verbatim as the prior
integration-refresh narrative for the same payload; this PROMPT-1593 report
documents the post-1591 re-rebase.

## Files Changed vs origin/main

```
client/Cargo.toml                                                                 |  10 +
client/src/presentation/qa_snapshot.rs                                            | 389 ++++++++++++++++++++-
production/sprint-status.yaml                                                     |  11 +-
reports/PROMPT-1586-qa-snapshot-resolution-phase-fields-followup.md               | 164 +++++++++
reports/PROMPT-1590-qa-plan-snapshot-fields-integration-refresh.md                |  83 +++++
reports/PROMPT-1592-qa-plan-snapshot-fields-mainland-refresh-after-1588.md        | 135 +++++++
tests/integration/qa_snapshot/resolution_phase_field_coverage_test.rs             | 313 +++++++++++++++++
7 files changed, 1099 insertions(+), 6 deletions(-)
```

This report itself will add one more entry once committed:
`reports/PROMPT-1593-qa-plan-snapshot-fields-mainland-refresh-after-1591.md`.

## Path Allowlist Review

All changed paths are within the owned scope of PROMPT 1593:

| Path | Owned-scope rule |
|---|---|
| `client/src/presentation/qa_snapshot.rs` | explicit owned scope (PROMPT 1586) |
| `tests/integration/qa_snapshot/resolution_phase_field_coverage_test.rs` | owned scope (qa snapshot tests changed by 1586) |
| `client/Cargo.toml` | related-Cargo (registers the new `[[test]]` binary for the PROMPT 1586 test — pure test wiring, no dependency change) |
| `production/sprint-status.yaml` | allowed: changed by 1587 for the QA-plan disposition row only (lines around 7990-8000) |
| `production/qa/**` | none touched directly here — PROMPT 1587 is paperwork-only against `sprint-status.yaml`; the QA plan body authored by PROMPT 1318/1320 is already on main |
| `reports/PROMPT-1586-*.md` | "include/carry reports from 1586/1587/1590/1592" |
| `reports/PROMPT-1590-*.md` | same |
| `reports/PROMPT-1592-*.md` | same |
| `reports/PROMPT-1593-*.md` | required output of this prompt |

No forbidden path touched: `client/src/ui/hand/mod.rs`,
`client/src/presentation/board_rendering.rs`,
`client/src/ui/shop_auction/mod.rs`, `server/**`, `shared/**`,
`production/session-state/**`, `production/sprints/**`,
`production/stage.txt`, unrelated Cargo/CI — all absent from the diff.

## Validation

- `git diff --check origin/main HEAD` → clean (no whitespace / conflict markers).
- `git merge-base --is-ancestor origin/main HEAD` → 0 (origin/main IS ancestor; strict-FF land path open).
- Cherry-picks applied with zero conflicts (no overlap with PROMPT 1591's tooling/launcher payload).
- Broad cargo verification deferred to the VERIFY lane per task rules; the
  registered focused test
  `qa_snapshot_resolution_phase_field_coverage_test` carries PROMPT 1586's
  field-coverage assertions and was validated on its original landing
  branch.

## Paperwork Non-Claims

This refresh does NOT:

- close Sprint 18,
- activate Sprint 19,
- advance the project stage,
- claim release readiness,
- retry PROMPT 761,
- close any carried QA condition,
- modify any file under `server/**`, `shared/**`, `production/sprints/**`,
  or `production/session-state/**`.

It carries the PROMPT 1587 paperwork-only correction to the Sprint 18
`qa_plan_found` row in `production/sprint-status.yaml` and the PROMPT 1586
QA-snapshot resolution-phase observability fields plus their integration
test — nothing else.

## Push Outcome

`git push -u origin integrate/qa-plan-snapshot-fields-1593` succeeded as a
new-branch creation; the branch is published as
`origin/integrate/qa-plan-snapshot-fields-1593 @ 7196bdad` (includes this
PROMPT-1593 report commit on top of the four cherry-picked commits).

## Status Line

1593: QA-PLAN-SNAPSHOT-FIELDS-MAINLAND-REFRESH-AFTER-1591: SHIPPED
