# Story 005: BOT-DEBUG-OVERLAY-001 -- Debug-Only Bot Overlay (Data Push Path)

> **Epic**: Bot & Autoplay
> **Story ID**: BOT-DEBUG-OVERLAY-001
> **Status**: Draft -- Sprint 19 candidate; data contract defined by PROMPT 1604; implementation gated on PROMPT 1602 / 1603 main-land
> **Layer**: Integration -- debug-only client UI fed by server-streamed bot decisions
> **Type**: Integration -- debug overlay + data-push wire path
> **Sprint**: Sprint 19 candidate (NOT activated)
> **Authored**: 2026-05-21 by PROMPT 1608
> **Authoring source-of-truth**: `origin/main@576fbe8ce901a8b919a4c2db58847f2d497d3d15`

---

## Status / No-Claim Banner

This story is authored as a **Sprint 19 candidate**. The debug-overlay
data contract was defined by PROMPT 1604; implementation was sequenced
to follow PROMPT 1602 (Wave 3 placement heuristic, main-landed) and
PROMPT 1603 (bot-vs-bot soak entrypoint, integration PROMPT 1607 active).

PROMPT 1608 does NOT:

- Activate Sprint 19 or this row.
- Modify any code under `client/`, `server/`, `shared/`, `tests/`,
  `tools/`, `docs/`, or `.claude/`.
- Run `/dev-story`, `/story-readiness`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, or `/qa-plan`.

**Critical constraint preserved verbatim**: the overlay is **debug-only**
and gated behind `CCGS_DEBUG_UI=1` and F8 per the 2026-05-21
orchestrator note. It MUST NOT ship in a release build.

Non-claims preserved verbatim: NO public release readiness, NO RC
readiness, NO full game completion, NO accessibility advancement, NO
playtest validation, NO `S8-QA-001-W1` closure, NO `PAW-TD-*-a`
completion, NO stage advance.

---

## Problem Class / Prevention Target

**Defect class**: Bot decisions are only visible through the streamed
decision log (PROMPT 1597) and QA snapshots. During live development
and triage, operators cannot see "why did the bot bid X / place at lane
Y" without grepping logs.

**Prevention target**: An in-game debug overlay that:

- Subscribes to the same decision data the QA snapshot stream emits.
- Renders the current bot decision context (last bid, last placement
  rationale, current heuristic scores) on screen.
- Is gated behind a dev/debug env (`CCGS_DEBUG_UI=1`) and a function
  key (F8) so it never appears in release builds.
- Uses the same data-push path that bot QA tools consume so the
  overlay and the tools see identical state.

---

## Acceptance Criteria (ledger; not gated by PROMPT 1608)

- AC1: Overlay is mounted only when `CCGS_DEBUG_UI=1` is set at client
  launch.
- AC2: F8 toggles overlay visibility while it is mounted.
- AC3: Overlay renders bot decision context for at least: lobby ready
  state, last auction bid + rationale, last placement target + score.
- AC4: Overlay data comes from a single server-pushed message type
  (PROMPT 1604 contract), not from client-side speculation.
- AC5: Overlay is not built into release builds (`cfg(debug_assertions)`,
  feature flag, or equivalent compile-time exclusion).
- AC6: Overlay never blocks game input; it is purely additive.
- AC7: Documentation under `docs/autoplay/` or
  `docs/architecture/bot-debug-overlay.md` describes the data contract
  and toggle behaviour.

---

## Dependencies

- **PROMPT 1604** (data contract) -- the wire schema for the
  decision-push messages.
- **PROMPT 1602** (Wave 3 placement heuristic) -- main-landed; the
  placement decision feeds AC3.
- **PROMPT 1603 / 1607** (bot-vs-bot soak entrypoint) -- once landed,
  the overlay can be visually validated by attaching to a soak room.

---

## Implementation Lineage on `origin/main`

| PROMPT | Status | Slice |
|---|---|---|
| 1604 | Shipped | Data contract specification only |
| 1602 | Main-landed | Wave 3 placement heuristic (overlay data source) |
| 1603 | Worker | Soak entrypoint (overlay consumer for validation) |
| 1607 | Active integration | Soak integration refresh |

The overlay itself is **not yet on `origin/main`** at the PROMPT 1608
authoring tip.

---

## Recommended Sprint 19 Follow-Up Prompts

1. Verify PROMPT 1607 main-land status.
2. `/story-readiness BOT-DEBUG-OVERLAY-001` against the Sprint 19
   activation tip.
3. `/dev-story BOT-DEBUG-OVERLAY-001` to implement the overlay + data
   push.
4. `/story-done` once acceptance criteria pass.

---

## Test Evidence (target)

- Logic: data-push message serde unit tests under
  `tests/unit/bot/debug_overlay/`.
- Integration: overlay-mounted-with-flag integration test under
  `tests/integration/bot/`.
- Manual: screenshot evidence of the overlay during a bot-vs-bot soak
  run, captured under `production/qa/evidence/`.
- Release-exclusion: build script or feature-flag test that asserts
  the overlay module is not compiled into a release binary (deferred
  if not feasible at story scope).
