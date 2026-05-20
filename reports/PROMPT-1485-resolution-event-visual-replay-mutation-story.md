# PROMPT-1485 Resolution Event Visual Replay Mutation Story Report

## Summary

Authored one future-ready implementation story for making
`S2CResolutionEvent` replay mutate visible presentation state in sub-step order.
The story is documentation-only and does not activate a future sprint.

## Story Path

- `production/epics/board-rendering/story-015-resolution-event-visual-replay-mutation.md`

## Epic Index Update

- Updated `production/epics/board-rendering/EPIC.md` to list Story 015 and
  adjust the story count from 14 to 15.

## Key Acceptance Criteria Covered

- Ordered replay application by `(sub_step, trigger_index)`.
- Unit movement and lane-change visible-state mutation.
- Placed/spawned unit display without duplicate board entities.
- Combat damage HP updates, including blocked/zero-damage cue handling.
- Unit death/removal marker or fade plus final removal before next phase.
- Objective HP/destruction feedback with hidden-identity boundaries preserved.
- Gold/reward feedback through deterministic presentation/HUD fanout.
- Spawn-range update ordering within replay.
- Buffered next-phase clarity after replay/objective reveal work drains.
- Deterministic World/App-based tests that do not require a full live session.

## Dependencies Made Explicit

- PROMPT 1472 full-flow PASS as the baseline flow dependency.
- PROMPT 1477 resolution/combat visual-state audit as the gap source.
- Board Rendering Story 006 for queue grouping, phase buffering, and invalid
  script recovery.
- Combat Resolution Story 011 for canonical `S2CResolutionEvent` /
  `ResolutionEvent` completeness and ordering.
- Board Rendering Story 003 for existing visible unit/objective/HP entities.
- Card Animations Stories 006 and 007 for objective reveal and damage numbers.
- Krosmaga visual reference reports PROMPT 1265 / 1266 / 1395 as visual
  hierarchy and feedback references only.

## Implementation-Ready Rationale

The story is scoped to a single missing presentation behavior: consume the
already-authoritative resolution log at replay tempo and mutate visible state as
each event is presented. It names concrete protocol variants, existing local
systems/resources, expected test fixtures, and authority boundaries. Acceptance
does not depend on a full live session because the required coverage can be
implemented with deterministic Bevy `App` / `World` tests using `Time<Virtual>`.

## Non-Claims

- No release readiness claimed.
- No final art or final VFX claimed.
- No broad combat redesign claimed.
- No sprint activation, sprint closeout, `/story-done`, sprint-status, QA-plan,
  or session-state update claimed.
- No Cargo validation performed, per prompt policy.

## Validation

- Static paperwork review only.
- `git diff --check` passed.
- No Cargo commands run.

## Source Availability Note

The exact PROMPT 1477 report path named in the prompt was not present in this
worktree snapshot. The story therefore references PROMPT 1477 as a required gap
source and uses the local code/story evidence available in this branch,
including existing Board Rendering replay queue and partial feedback surfaces.
Raw PROMPT 1265 / 1266 / 1395 report files were also not present as standalone
reports in this snapshot; existing future-candidate stories and architecture
notes cite PROMPT 1265 / 1266, and the new story requires all three references
to be read before implementation.

1485: RESOLUTION-EVENT-VISUAL-REPLAY-MUTATION-STORY: COMPLETE
