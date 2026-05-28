# Story 002: BOT-SOAK-ENTRYPOINT-001 -- Bot-vs-Bot Soak Entrypoint + QA Harness

> **Epic**: Bot & Autoplay
> **Story ID**: BOT-SOAK-ENTRYPOINT-001
> **Status**: Draft -- Sprint 19 candidate; PROMPT 1603 worker shipped; PROMPT 1607 confirmed main-landed (PROMPT 1762, `origin/main@7ca41fc4`); implementation lineage 1603–1743 present on `origin/main`
> **Layer**: Integration -- server-side debug-only soak room + PowerShell harness
> **Type**: Integration -- new debug-only `Start-BotVsBotSoak.ps1` + `--bot-vs-bot-max-rounds` flag candidate
> **Sprint**: Sprint 19 candidate (NOT activated)
> **Authored**: 2026-05-21 by PROMPT 1608
> **Authoring source-of-truth**: `origin/main@576fbe8ce901a8b919a4c2db58847f2d497d3d15`

---

## Status / No-Claim Banner

This story is authored as a **Sprint 19 candidate** to track the
bot-vs-bot soak entrypoint that PROMPT 1603 began shipping. PROMPT 1607
is the integration refresh; it is **confirmed main-landed** as of PROMPT
1762 (`origin/main@7ca41fc4`, 2026-05-28). Full implementation lineage
1603–1743 is present on `origin/main`. The earlier "not yet confirmed
main-landed" text was accurate only at the PROMPT 1608 authoring tip
(`576fbe8c`); this banner was repaired by PROMPT 1769.

PROMPT 1608 does NOT (preserved verbatim — authoring-time non-claims):

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
| 1607 | `integrate/bot-flow-two-bot-soak-entrypoint-1607` | Confirmed main-landed — PROMPT 1762 (`origin/main@7ca41fc4`, 2026-05-28) |
| 1603–1743 | (full lineage) | All implementation commits confirmed on `origin/main` by PROMPT 1762 |

PROMPT 1762 confirmed all AC1–AC6 deliverables are present on `origin/main`.
The story is implementation-complete; remaining gates are Sprint 19 activation
and `/story-done` paperwork (see PROMPT 1768 readiness report for full detail).

---

## Recommended Sprint 19 Follow-Up Prompts

1. ~~Confirm PROMPT 1607 main-land status~~ — **DONE** (PROMPT 1762 confirmed;
   PROMPT 1769 repaired this banner).
2. Sprint 18 close-out → Sprint 19 planning + activation (PROMPT 1768 Blocker 2).
3. `/story-readiness BOT-SOAK-ENTRYPOINT-001` once Sprint 19 row exists.
4. `/story-done` — paperwork-only (all implementation on `origin/main`); reference
   PROMPT 1678 and PROMPT 1758 as live soak evidence.
5. (Optional) A CI smoke wiring follow-up that runs a bounded soak on
   each merge to `main`; this is a separate story, not in scope here.

---

## Test Evidence (target)

- Logic: bot decision determinism tests (existing under
  `tests/unit/bot/`); soak determinism harness deferred.
- Integration: bot-vs-bot soak integration test under
  `tests/integration/bot/` (PROMPT 1607 confirmed main-landed; integration
  tests present per PROMPT 1762 AC5 evidence at commit `c84f03be`).
- Manual: orchestrator-driven bounded soak (e.g. `--max-rounds 5`)
  with captured QA snapshot under `production/qa/evidence/`.
