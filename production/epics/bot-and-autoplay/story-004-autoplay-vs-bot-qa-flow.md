# Story 004: AUTOPLAY-VS-BOT-QA-001 -- Autoplay-vs-Bot QA Flow

> **Epic**: Bot & Autoplay
> **Story ID**: AUTOPLAY-VS-BOT-QA-001
> **Status**: Draft -- Sprint 19 candidate; gated on stories 001 + 002 + 003
> **Layer**: Integration -- QA harness consuming both bot and autoplay surfaces
> **Type**: Integration -- harness composition + QA evidence pipeline
> **Sprint**: Sprint 19 candidate (NOT activated)
> **Authored**: 2026-05-21 by PROMPT 1608
> **Authoring source-of-truth**: `origin/main@576fbe8ce901a8b919a4c2db58847f2d497d3d15`

---

## Status / No-Claim Banner

This story is authored as a **Sprint 19 candidate** that composes the
bot participant (story 001), bot-vs-bot soak entrypoint (story 002), and
autoplay recipe library v1 (story 003) into a single repeatable QA flow.
It is **gated** on 001 / 002 / 003 reaching at least readiness verdicts
on the Sprint 19 activation tip.

PROMPT 1608 does NOT:

- Activate Sprint 19 or this row.
- Modify any code under `client/`, `server/`, `shared/`, `tests/`,
  `tools/`, `docs/`, or `.claude/`.
- Run `/dev-story`, `/story-readiness`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, or `/qa-plan`.

Non-claims preserved verbatim: NO public release readiness, NO RC
readiness, NO full game completion, NO accessibility advancement, NO
playtest validation, NO `S8-QA-001-W1` closure, NO `PAW-TD-*-a`
completion, NO stage advance.

---

## Problem Class / Prevention Target

**Defect class**: Manual two-client QA is the bottleneck on Sprint
close-outs (S8-QA-001-W1 has been carried across many sprints partly
for this reason). Without a repeatable autonomous flow that exercises
the full friend-game loop, regressions in HOT-A / HOT-B / HOT-C
gameplay paths surface late.

**Prevention target**: A single QA flow that:

- Spawns a server.
- Connects one real client driven by autoplay recipes (story 003)
  through real UI input.
- Joins a bot opponent (story 001) on the other side.
- Plays the friend-game loop to at least one full RESOLUTION and
  ideally to GAME_OVER.
- Captures QA snapshots, decision log, and recipe pass/fail at each
  step.
- Produces a single artifact under `production/qa/evidence/` that the
  Sprint close-out paperwork can cite.

This unblocks the gameplay HOT lanes (currently blocked on real
gameplay evidence per the 2026-05-21 orchestrator note).

---

## Acceptance Criteria (ledger; not gated by PROMPT 1608)

- AC1: A single invocation (PowerShell script or Cargo bin) spawns a
  server, a real autoplay-driven client, and a bot opponent in one
  game room.
- AC2: The flow exercises class confirm, draft/shop/auction, drag/drop
  placement, accepted ACK, rejection recovery, and resolution.
- AC3: Per-step QA snapshot fields are captured and persisted.
- AC4: Decision-log entries (PROMPT 1597) are captured for the bot
  side and recipe-log entries are captured for the autoplay side.
- AC5: The flow reaches at least one full RESOLUTION; reaching
  GAME_OVER is a stretch acceptance target, not a blocker.
- AC6: Pass/fail verdict is emitted as a structured report (e.g.
  `reports/AUTOPLAY-VS-BOT-<date>.md`).
- AC7: Documentation under `docs/autoplay/` describes how to run the
  flow and what the artifacts mean.

---

## Dependencies

- **Story 001 (BOT-ROOM-PARTICIPANT-001)**: bot must reach
  `SessionReady`, place units, and not panic.
- **Story 002 (BOT-SOAK-ENTRYPOINT-001)**: the harness shares the
  `--max-rounds` bounding flag and the soak-output schema.
- **Story 003 (AUTOPLAY-RECIPE-LIBRARY-001)**: the recipe library
  supplies the real UI inputs.

This story does NOT begin readiness until all three dependencies have
either landed or have committed Sprint 19 candidate plans.

---

## Recommended Sprint 19 Follow-Up Prompts

1. Verify dependencies on the Sprint 19 activation tip.
2. `/story-readiness AUTOPLAY-VS-BOT-QA-001`.
3. `/dev-story AUTOPLAY-VS-BOT-QA-001`.
4. `/story-done` once acceptance criteria pass.
5. (Optional) A CI-wiring follow-up story to run this flow on every
   merge to `main`; out of scope here.

---

## Test Evidence (target)

- Integration: end-to-end harness run under
  `tests/integration/autoplay_vs_bot/`.
- Manual: orchestrator-driven friend-game pass with captured artifacts
  under `production/qa/evidence/autoplay-vs-bot-<date>.md`.
- Logic: no new pure-function logic; this story is composition.
