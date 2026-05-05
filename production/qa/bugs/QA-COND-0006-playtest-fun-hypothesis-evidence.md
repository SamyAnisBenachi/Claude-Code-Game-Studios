# QA-COND-0006: Playtest/Fun-Hypothesis Evidence

| Field | Value |
|---|---|
| ID | QA-COND-0006 |
| Kind | Evidence Gap |
| Severity | S2 High |
| Priority | P4 Accepted risk candidate |
| Status | Accepted Risk |
| Action State | Deferred Accepted |
| Reported | 2026-05-05 |
| Source | Production-to-Polish gate check |
| Producer Decision | 2026-05-05: Reclassified out of active Sprint 6 remediation scope |

## Summary

The repository does not contain production playtest evidence for the
Production-to-Polish gate. The producer decision on 2026-05-05 defers this
requirement out of active Sprint 6 remediation scope as an accepted risk.

This is a producer decision / reclassification, not evidence completion. No
Sprint 6 playtests are being run now, no dev-only playable shell will be
implemented just to satisfy S6-02, and no playtest reports may be fabricated.

## Source Evidence

- `production/gate-checks/gate-production-polish-sprint-5-2026-05-05.md`
  records no `production/playtests/` evidence.
- The same gate check marks the playtest session count, required coverage, and
  fun-hypothesis validation or revision as failed.
- The gate check identifies missing playtest evidence as a hard
  Production-to-Polish blocker.
- `production/playtests/sprint-6-fun-hypothesis-decision.md` records the
  2026-05-05 producer deferral / accepted-risk decision.
- `production/sprint-status.yaml` records S6-02 as accepted risk and removes it
  from the active S6-06 blocker list.

## Future Closure Evidence

Future evidence-based closure still requires actual playtest evidence that
includes:

- At least three documented production playtest sessions.
- Coverage of new-player experience.
- Coverage of mid-game systems.
- Coverage of difficulty curve.
- A fun-hypothesis validation or revision decision based on the session results.

## Current Blocker Status

QA-COND-0006 is no longer an active S6-06 Sprint 6 remediation blocker because
the producer accepted deferral / reclassification on 2026-05-05.

The underlying playtest evidence gap remains real. Production -> Polish gate
review must carry it as an explicit condition/risk, not as passed playtest
evidence.

## Verification Guard

QA-COND-0006 must not be marked `Verified` from this decision. It may only move
to `Verified` after actual playtest evidence exists and QA verifies that the
evidence satisfies the gate requirement. This accepted-risk disposition does not
validate or revise the fun hypothesis.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not create playtest reports.
- Does not revise the fun hypothesis.
- Does not implement a dev-only playable shell.
- Does not close the Production -> Polish playtest evidence gap by evidence.
