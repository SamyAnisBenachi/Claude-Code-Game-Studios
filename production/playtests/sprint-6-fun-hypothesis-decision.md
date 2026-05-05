# Sprint 6 Fun Hypothesis Decision

| Field | Value |
|---|---|
| Sprint | Sprint 6 |
| Current status | Producer Deferred / Accepted Risk |
| Gate condition | `production/qa/bugs/QA-COND-0006-playtest-fun-hypothesis-evidence.md` |
| Playtest protocol | `production/playtests/sprint-6-playtest-protocol.md` |
| Playtest package commit | `250de37` |
| Required reports | Deferred; no Sprint 6 reports created |
| Producer decision date | 2026-05-05 |

## Purpose

This file records the aggregate Sprint 6 fun-hypothesis decision after the
three required playtest reports are filled. It now also records the 2026-05-05
producer decision to defer S6-02 / QA-COND-0006 out of active Sprint 6
remediation scope.

This is a producer decision / reclassification, not evidence completion. The
core fun hypothesis remains unvalidated and unrevised by playtest evidence.

## Core Fun Hypothesis

Players should feel like cunning tacticians in an information war:

- Auction reads should let players infer opponent intent from bid/drop timing,
  gold pressure, board state, and commitment patterns.
- Hidden-objective deception should create real/fake suspense where players care
  about fooling the opponent and being fooled.
- Simultaneous placement should make commitment feel tense, readable after
  reveal, and strategically meaningful.
- No phase should create idle spectating; when players are not directly acting,
  they should be reading meaningful live information.

## Required Reports

| Session | Required report placeholder | Status |
|---|---|---|
| New-player | `production/playtests/sprint-6-new-player-[date]-[tester].md` | Deferred by producer decision; not run |
| Mid-game | `production/playtests/sprint-6-mid-game-[date]-[tester].md` | Deferred by producer decision; not run |
| Difficulty-curve | `production/playtests/sprint-6-difficulty-curve-[date]-[tester].md` | Deferred by producer decision; not run |

## Decision States

### VALIDATE

Use `VALIDATE` when at least 2 of the 3 required sessions support the core fun
hypothesis and no report contains an unresolved pillar-critical failure.
Auction tension is readable, hidden-objective suspense creates deception rather
than confusion, simultaneous placement supports meaningful commitment, and
non-input phases avoid idle spectating.

### REVISE

Use `REVISE` when the session evidence shows the core fun hypothesis is not
working. This includes repeated confusion loops that prevent meaningful play,
auction tension that is unreadable or irrelevant, real/fake objective suspense
collapsing into confusion, simultaneous placement failing to communicate
commitment/payoff, or no-input phases becoming idle spectating.

### VALIDATE WITH CONDITIONS

Use `VALIDATE WITH CONDITIONS` when at least 2 of the 3 required sessions
support the core fun hypothesis, but specific bounded issues must be tracked and
rechecked. Conditions must name the issue, evidence source, owner/path if known,
and the recheck trigger.

## Decision Criteria

- At least 2 of 3 filled session reports must support the core fun hypothesis
  for `VALIDATE` or `VALIDATE WITH CONDITIONS`.
- Repeated confusion loops in any required session trigger `REVISE` or
  `VALIDATE WITH CONDITIONS`, depending on severity and containment.
- Auction evidence must show readable tension, meaningful bid/drop decisions,
  and opponent-read value.
- Real/fake objective evidence must show suspense and deception rather than
  sustained confusion.
- Simultaneous placement evidence must show that commitment, reveal, and
  resolution are understandable enough to support tactical reads.
- No-idle evidence must show that non-input phases provide meaningful
  information to read rather than passive spectating.

## Producer Reclassification

Decision date: 2026-05-05

Producer decision:
- Do not run Sprint 6 playtests now.
- Do not implement a dev-only playable shell just to satisfy S6-02.
- Do not fabricate playtest reports.
- Reclassify S6-02 / QA-COND-0006 out of active Sprint 6 remediation scope.
- Continue Sprint 6 on the remaining planned remediation work.

Disposition:
S6-02 / QA-COND-0006 is accepted as deferred risk for Sprint 6. It should not
remain an active blocker for S6-06 if the producer accepts this deferral.

Production -> Polish gate handling:
The gate must carry the missing playtest and fun-hypothesis evidence as an
explicit condition/risk, not as passed playtest evidence.

## Current Decision

Decision: Producer Deferred / Accepted Risk

Rationale:
No Sprint 6 playtest reports exist. The producer has decided not to run Sprint 6
playtests now, not to implement a dev-only playable shell just to satisfy S6-02,
and not to fabricate playtest reports.

This decision reclassifies S6-02 / QA-COND-0006 out of active Sprint 6
remediation scope. It does not validate the fun hypothesis, revise the fun
hypothesis, or complete QA evidence. QA-COND-0006 must not be marked `Verified`
from this decision.

Conditions, if any:
- The missing new-player report remains a future evidence requirement if the
  playtest gate condition is later reopened for evidence-based closure.
- The missing mid-game report remains a future evidence requirement if the
  playtest gate condition is later reopened for evidence-based closure.
- The missing difficulty-curve report remains a future evidence requirement if
  the playtest gate condition is later reopened for evidence-based closure.
- Sprint 6 final sign-off and Production -> Polish gate-check must list
  QA-COND-0006 as an explicit accepted risk / deferred condition, not as passed
  playtest evidence.
