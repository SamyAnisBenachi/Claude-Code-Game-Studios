# Story 002: BOT-SOAK-ENTRYPOINT-001 -- Bot-vs-Bot Soak Entrypoint + QA Harness

> **Epic**: Bot & Autoplay
> **Story ID**: BOT-SOAK-ENTRYPOINT-001
> **Status**: Draft -- Sprint 19 candidate; PROMPT 1603 worker shipped; integration refresh PROMPT 1607 active (not yet confirmed main-landed)
> **Layer**: Integration -- server-side debug-only soak room + PowerShell harness
> **Type**: Integration -- new debug-only `Start-BotVsBotSoak.ps1` + `--bot-vs-bot-max-rounds` flag candidate
> **Sprint**: Sprint 19 candidate (NOT activated)
> **Authored**: 2026-05-21 by PROMPT 1608
> **Authoring source-of-truth**: `origin/main@576fbe8ce901a8b919a4c2db58847f2d497d3d15`

---

## Status / No-Claim Banner

This story is authored as a **Sprint 19 candidate** to track the
bot-vs-bot soak entrypoint that PROMPT 1603 began shipping. PROMPT 1607
is the active integration refresh; it is **NOT confirmed main-landed**
at the PROMPT 1608 authoring source-of-truth.

PROMPT 1608 does NOT:

- Activate Sprint 19.
- Modify any code, Cargo, Trunk, or CI artifact.
- Confirm landing of PROMPT 1603 or PROMPT 1607.
- Run `/dev-story`, `/story-readiness`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, or `/qa-plan`.

Non-claims preserved verbatim: NO public release readiness, NO RC
readiness, NO full game completion, NO accessibility / playtest
advancement, NO `S8-QA-001-W1` closure, NO `PAW-TD-*-a` completion,
NO stage advance.

---

## Problem Class / Prevention Target

**Defect class**: Bot decision regressions surface only after a full
round is played. Manual two-client play cannot exercise a long-running
soak (dozens or hundreds of rounds) at the rate required to catch
intermittent crashes, infinite-loop heuristics, or auction-pool drift.

**Prevention target**: A debug-only soak entrypoint that:

- Spawns two bots into a single room and runs the round loop without
  any human client attached.
- Supports an optional `--bot-vs-bot-max-rounds` flag to bound runtime
  and produce a deterministic completion event.
- Emits round-by-round QA snapshots and decision-log entries so a
  smoke pipeline can detect regressions without manual review.

---

## Acceptance Criteria (ledger; not gated by PROMPT 1608)

- AC1: A `Start-BotVsBotSoak.ps1` script (or equivalent Cargo
  invocation) launches a bot-vs-bot game without any human client.
- AC2: Both bots reach `SessionReady` and play through DRAFT_INITIAL →
  DRAFT_SHOP → AUCTION → PLACEMENT → RESOLUTION → next-loop without
  server panic.
- AC3: An `--bot-vs-bot-max-rounds N` flag bounds the run to N rounds
  (and exits cleanly on the bound).
- AC4: Per-round QA snapshot fields are emitted for both bot players
  and persisted to a soak-output file.
- AC5: Decision-log stream (PROMPT 1597) is captured for both bots
  across the soak window.
- AC6: The soak entrypoint is debug-only and excluded from release
  builds (feature flag or `cfg(debug_assertions)` or env gate).
- AC7: Documentation lives under `docs/autoplay/` or
  `tools/autoplay/README.md` describing how to run a soak.

---

## Implementation Lineage on `origin/main` (candidate; verify at readiness)

| PROMPT | Branch | Status |
|---|---|---|
| 1603 | `work/bot-flow-two-bot-soak-entrypoint-1603` | Worker shipped |
| 1607 | `integrate/bot-flow-two-bot-soak-entrypoint-1607` | Integration refresh active; main-land not yet confirmed at this authoring tip |

The Sprint 19 readiness step MUST verify which artifacts actually exist
on `origin/main`. If PROMPT 1607 has main-landed by then, the story may
be a paperwork-only `/story-done`; if not, the story is `/dev-story`
work over the remaining gap.

---

## Recommended Sprint 19 Follow-Up Prompts

1. Confirm PROMPT 1607 main-land status (or supersede with a fresh
   integration prompt against the Sprint 19 activation tip).
2. `/story-readiness BOT-SOAK-ENTRYPOINT-001`.
3. `/story-done` or `/dev-story` depending on the verdict.
4. (Optional) A CI smoke wiring follow-up that runs a bounded soak on
   each merge to `main`; this is a separate story, not in scope here.

---

## Test Evidence (target)

- Logic: bot decision determinism tests (existing under
  `tests/unit/bot/`); soak determinism harness deferred.
- Integration: bot-vs-bot soak integration test under
  `tests/integration/bot/` (deferred until PROMPT 1607 main-lands or a
  successor lands).
- Manual: orchestrator-driven bounded soak (e.g. `--max-rounds 5`)
  with captured QA snapshot under `production/qa/evidence/`.
