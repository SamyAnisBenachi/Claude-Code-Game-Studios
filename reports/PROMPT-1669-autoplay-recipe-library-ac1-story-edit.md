# PROMPT 1669 - AUTOPLAY-RECIPE-LIBRARY-AC1-STORY-EDIT

Status: SHIPPED

Source branch refreshed by orchestrator:

- Worker branch: `origin/worker/1669-recipe-library-ac1-story-edit`
- Worker commit: `5eb91192`
- Integration branch: `integrate/autoplay-recipe-ac1-story-edit-1669`
- Refreshed commit: `fdd8bfb8`
- Base at refresh: `origin/main@15992b27`

Scope:

- Updated `production/epics/bot-and-autoplay/story-003-autoplay-recipe-library-v1.md`.
- No source code, Cargo, sprint-status, session-state, or QA evidence files changed.

Summary:

- Aligned AUTOPLAY-RECIPE-LIBRARY-001 AC1 with the actual 11-recipe registry.
- Documented renamed and merged conceptual recipe coverage:
  `lobby_join` is superseded by `add-bot-lobby`, and draft/shop/auction concepts
  are covered by `draft-auction-probe`.
- Explicitly descoped `placement_reject_recovery` from v1 as a future v1.1 or
  separate story candidate, because no standalone rejection-recovery recipe exists.
- Preserved the story as readiness/paperwork alignment only; it does not claim
  story-done, sprint activation, or release readiness.

Validation:

- `git diff --check HEAD~1..HEAD` => PASS.
- Path scope checked: story file plus this report only after orchestrator report
  backfill.

1669: AUTOPLAY-RECIPE-LIBRARY-AC1-STORY-EDIT: SHIPPED
