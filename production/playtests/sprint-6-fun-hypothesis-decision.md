# Sprint 6 Fun Hypothesis Decision

| Field | Value |
|---|---|
| Sprint | Sprint 6 |
| Current status | Pending Sessions |
| Gate condition | `production/qa/bugs/QA-COND-0006-playtest-fun-hypothesis-evidence.md` |
| Playtest protocol | `production/playtests/sprint-6-playtest-protocol.md` |
| Playtest package commit | `250de37` |
| Required reports | New-player, mid-game, difficulty-curve |

## Purpose

This file records the aggregate Sprint 6 fun-hypothesis decision after the
three required playtest reports are filled. It remains `Pending Sessions` until
the new-player, mid-game, and difficulty-curve reports exist and contain
evidence-based outcomes.

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
| New-player | `production/playtests/sprint-6-new-player-[date]-[tester].md` | Pending |
| Mid-game | `production/playtests/sprint-6-mid-game-[date]-[tester].md` | Pending |
| Difficulty-curve | `production/playtests/sprint-6-difficulty-curve-[date]-[tester].md` | Pending |

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

## Current Decision

Decision: Pending Sessions

Rationale:
The Sprint 6 decision cannot be made until all three required reports are filled
from actual playtest sessions. QA-COND-0006 remains open until those reports
exist and this decision is updated to `VALIDATE`, `REVISE`, or
`VALIDATE WITH CONDITIONS`.

Conditions, if any:
- Pending completion of the new-player report.
- Pending completion of the mid-game report.
- Pending completion of the difficulty-curve report.
