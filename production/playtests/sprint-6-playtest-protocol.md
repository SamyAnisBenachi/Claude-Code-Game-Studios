# Sprint 6 Playtest Protocol

| Field | Value |
|---|---|
| Sprint | Sprint 6 |
| Purpose | Production-to-Polish playtest evidence package |
| Required sessions | New-player, mid-game, difficulty-curve |
| Evidence location | `production/playtests/` |
| Template location | `production/playtests/templates/` |
| Gate source | `production/qa/bugs/QA-COND-0006-playtest-fun-hypothesis-evidence.md` |

## Purpose

Sprint 6 must produce playtest evidence for the Production-to-Polish gate. The
package requires three documented sessions that cover:

- New-player experience.
- Mid-game systems.
- Difficulty curve.
- Fun-hypothesis validation or revision.

This protocol defines how to run and report those sessions. It does not create
filled session reports. Create filled reports only after actual sessions using:

- `production/playtests/sprint-6-new-player-[date]-[tester].md`
- `production/playtests/sprint-6-mid-game-[date]-[tester].md`
- `production/playtests/sprint-6-difficulty-curve-[date]-[tester].md`

## Required Sessions

| Session | Template | Primary Question |
|---|---|---|
| New-player | `templates/new-player-session.md` | Can a first-time player understand the goal, act without confusion loops, and feel the core read/fool tension? |
| Mid-game | `templates/mid-game-session.md` | Do auction, economy, class, prism, and deck-growth systems stay readable and engaging once the match is underway? |
| Difficulty curve | `templates/difficulty-curve-session.md` | Does pressure escalate without creating unfair spikes, dead time, or unrecoverable snowballing? |

## Common Report Headings

Every filled report must include these headings:

- Session Info
- Setup Checklist
- Test Focus
- First Impressions
- Gameplay Flow
- Bugs Encountered
- Feature-Specific Feedback
- Quantitative Data
- Success/Fail Criteria
- Fun-Hypothesis Questions
- Decision Rule
- Overall Assessment
- Top 3 Priorities From This Session

## Shared Facilitation Rules

- Record observations before interpreting them.
- Capture direct tester language when it explains confusion, delight, boredom,
  tension, or frustration.
- Avoid coaching unless the tester is blocked. If coaching happens, record what
  was said and why.
- Separate bugs from design findings. A crash, desync, invalid state, or broken
  input is a bug; unclear purpose, boredom, or bad pacing is a design finding.
- Mark any unplayed required block as missing evidence rather than inferring it.
- Record the build commit before the session begins.

## Shared Success Criteria

A session is successful evidence when all of the following are true:

- The session uses the correct template and filename pattern.
- Session Info is complete enough to reproduce the context.
- The session-specific required blocks are filled from actual play.
- At least one fun-hypothesis question is answered with observed evidence.
- Bugs, confusion points, and design concerns are listed even if none occurred.
- The report ends with exactly one decision: `VALIDATE`, `REVISE`, or
  `VALIDATE WITH CONDITIONS`.

## Shared Fail Criteria

A session is failed or incomplete evidence if any of the following are true:

- The session did not happen and the report is speculative.
- Build, tester, date, or session type is missing.
- A required session-specific block is blank.
- The tester could not reach the target flow because of a blocking bug and no
  partial evidence is recorded.
- The report has no explicit decision rule outcome.
- The decision outcome contradicts the recorded evidence.

## Fun Hypothesis

The Sprint 6 fun hypothesis is that Lanes and Lies works when players feel like
cunning tacticians in an information war:

- They can read opponent intent through auctions, gold, board state, and timing.
- They can fool the opponent through hidden objectives and lane commitment.
- Their deck or class plan can come online during the match.
- They have no idle spectator time; if they are not acting, they are reading
  meaningful live information.
- The auction feels like the signature tension point, not a side menu.

## Common Fun-Hypothesis Questions

Ask and answer these in every report where the session provides evidence:

1. Did the tester describe a moment of reading the opponent or predicting intent?
2. Did the tester describe a moment of fooling, being fooled, or caring about
   hidden objective uncertainty?
3. Did the tester feel a deck, class, or lane plan starting to come online?
4. Did any phase feel like dead time rather than active reading or decision
   making?
5. Did the auction feel tense, readable, and central to the match?
6. Would the tester voluntarily play another match, and why?

## Targeted Watch Items

These items must be explicitly checked when they occur:

- **YOU ARE LEADING idle window:** If a player leads an auction for 10 or more
  seconds with no opponent bid, record whether it felt tense, useful, boring, or
  dead.
- **Gold disparity:** If one player leads by 20 or more gold before round 9,
  record whether both players still had meaningful auction agency.
- **No-bid auctions:** Record whether no-bid or low-bid auctions feel strategic,
  confusing, or dull.
- **Objective HP clarity:** Record whether exact objective HP helps planning or
  weakens bluff tension.
- **Haste/Charge language:** Record any keyword-name confusion.
- **Resolution readability:** Record whether the tester understands why lanes
  resolved the way they did.

## Decision Rule

Each report must choose one outcome.

### VALIDATE

Use `VALIDATE` only when:

- The session-specific success criteria pass.
- The tester shows evidence of the intended fun hypothesis.
- No hard confusion loop blocks normal play.
- No observed issue requires design revision before the next gate.

### REVISE

Use `REVISE` when:

- A pillar-critical issue appears, especially idle spectating, unreadable auction
  tension, or hidden objectives failing to matter.
- A confusion loop prevents the tester from making meaningful decisions.
- Difficulty spikes, snowballing, or opaque systems make the session feel unfair.
- Blocking bugs prevent enough play to judge the target flow.

### VALIDATE WITH CONDITIONS

Use `VALIDATE WITH CONDITIONS` when:

- The core fun hypothesis is supported, but one or more issues need follow-up.
- The issue is bounded enough to track as a bug, balance task, UX task, or later
  design change.
- The report names the condition, evidence, owner/path if known, and recheck
  trigger.

## Aggregate Sprint 6 Outcome

After all three sessions exist, produce a separate aggregate playtest summary.
Do not infer the aggregate decision from this protocol alone.

Aggregate `VALIDATE` requires all three required sessions to be valid evidence
and no unresolved pillar-critical findings.

Aggregate `REVISE` is required if any session identifies a pillar-critical
failure that remains unresolved.

Aggregate `VALIDATE WITH CONDITIONS` is appropriate when the required evidence
exists, the fun hypothesis is mostly supported, and remaining issues are tracked
with explicit follow-up.
