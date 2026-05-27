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

This story was authored as a **Sprint 19 candidate** at `origin/main@576fbe8c`
(2026-05-21). The debug-overlay data contract was defined by PROMPT 1604.

**Implementation landed update (PROMPT 1670 / 2026-05-27):**
The overlay is **fully implemented and on `origin/main`** as of PROMPT 1614 →
PROMPT 1617 main-land → verified PROMPT 1618. Commits `37306162` (PROMPT 1614
implementation) and `b0249375` (PROMPT 1632 polish + arch doc + tests) are both
confirmed ancestors of `origin/main` per PROMPT 1666 reconcile audit at
`origin/main@e4249f07`.

The stale language "not yet on `origin/main`" from PROMPT 1608 is superseded.
Story-done, sprint activation, and `/story-readiness` AC5 ruling are **not**
performed by PROMPT 1670 — see AC Status section.

PROMPT 1670 does NOT:

- Activate Sprint 19 or this row.
- Modify any code under `client/`, `server/`, `shared/`, `tests/`,
  `tools/`, `docs/`, or `.claude/`.
- Run `/dev-story`, `/story-readiness`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, or `/qa-plan`.
- Make the AC5 ruling.

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

## AC Status (PROMPT 1670 assessment — not a readiness ruling)

Based on PROMPT 1666 reconcile audit and the implementation report chain.
AC5 ruling is deferred to human at `/story-readiness`.

| AC | Verdict | Evidence |
|----|---------|----------|
| AC1 | **PASS** | `debug_bot_overlay.rs:43`; integration test confirms no-spawn when unset (commit b0249375). PROMPT 1623 §4 full audit. |
| AC2 | **PASS (code) — live visual ADVISORY** | `KeyCode::F8`, `just_pressed` toggle; 7 inline client tests pass. PROMPT 1621 live visual blocked (human/GUI required). Advisory only, not blocking. |
| AC3 | **PASS** | All three fields (lobby ready state, last auction bid + rationale, last placement target + score) confirmed present by PROMPT 1623 §4 audit. |
| AC4 | **PASS** | `S2CDebugBotStatePush` protocol; server push via `bot_debug_push_system`. PROMPT 1614 implementation, PROMPT 1618 compile verify. |
| AC5 | **NEEDS HUMAN RULING** | Implementation uses runtime env-gating (`CCGS_DEBUG_UI=1` + `cfg(debug_assertions)` default). Overlay IS compiled into the release binary; it is not present at runtime without the env var. Story text says "compile-time exclusion." PROMPT 1623 auditor called this PASS on dual-gate safety. See AC5 Ruling Options below. |
| AC6 | **PASS** | `should_block_lower=false` Pickable config confirmed (PROMPT 1623 §3). |
| AC7 | **PASS** | `docs/architecture/bot-debug-overlay.md` exists (342 lines). Covers data contract, operator workflow, release safety, test coverage. Landed in commit b0249375. |

**Summary**: AC1/AC3/AC4/AC6/AC7 PASS. AC2 code PASS with live visual verify
ADVISORY. **AC5 needs human ruling before `/story-readiness` can close the story.**

---

## AC5 Ruling Options (paste one into `/story-readiness`)

Three mutually exclusive options. The ruling is yours; PROMPT 1670 does not choose.

---

**Option A — Accept runtime env-gating as satisfying AC5 (reinterpret)**

> AC5 is satisfied. The story phrase "compile-time exclusion" is interpreted
> broadly to include strong runtime env-gating. The dual guard
> (`CCGS_DEBUG_UI=1` env var checked at spawn + `cfg(debug_assertions)`
> default-off) provides equivalent release safety without a compile-time
> feature flag. No story edit or code change required. Mark AC5 **PASS**.

---

**Option B — Update AC5 wording to match implementation, then pass**

> AC5 wording is updated to reflect the shipped design. Replace
> "`cfg(debug_assertions)`, feature flag, or equivalent compile-time
> exclusion" with "`CCGS_DEBUG_UI=1` env-gating or equivalent runtime
> exclusion that prevents display in operator-facing builds." A story
> edit is required (may be done inline at `/story-readiness`). Mark
> AC5 **PASS** after the edit is committed.

---

**Option C — Require true compile-time exclusion before story-done**

> AC5 remains blocked. The story requires that the overlay is not
> _compiled_ into release builds. A follow-up PROMPT must add a Cargo
> feature flag (e.g. `debug_overlay`) and gate `debug_bot_overlay.rs`
> behind `#[cfg(feature = "debug_overlay")]`. Story-done is deferred
> until that PROMPT lands on `origin/main`. Mark AC5 **NOT PASS**.

---

## Dependencies

- **PROMPT 1604** (data contract) -- the wire schema for the
  decision-push messages. **Shipped.**
- **PROMPT 1602** (Wave 3 placement heuristic) -- main-landed; the
  placement decision feeds AC3. **Shipped.**
- **PROMPT 1603 / 1607** (bot-vs-bot soak entrypoint) -- used for
  visual validation via soak room.

---

## Implementation Lineage on `origin/main`

Updated by PROMPT 1670 at `origin/main@de2aafcb`. All commits below confirmed
ancestors of `origin/main` per PROMPT 1666 reconcile audit (`origin/main@e4249f07`).

| PROMPT | Commit | Status | Slice |
|--------|--------|--------|-------|
| 1604 | — | Shipped | Data contract specification only |
| 1602 | — | Main-landed | Wave 3 placement heuristic (overlay data source) |
| 1614 | `37306162` | **Main-landed** | Core overlay: `debug_bot_overlay.rs`, `debug_push.rs`, `S2CDebugBotStatePush` protocol, 16 tests |
| 1617 | — | **Main-landed** | Integration refresh — cherry-pick clean, `READY_FOR_MAINLAND_ENQUEUE` |
| 1618 | — | **Main-landed** | Focused verify — Cargo check PASS (shared/server/client), 9/9 server + 7/7 client tests PASS |
| 1623 | — | Shipped (read-only) | UX contract audit — all critical AC PASS |
| 1628 | — | **Main-landed** | 25 standalone bot_debug_push unit tests, all PASS |
| 1630 | — | Shipped | `docs/architecture/bot-debug-overlay.md` authored (342 lines) |
| 1632 | `b0249375` | **Main-landed** | UX polish + arch doc + tests: tail-cap align, DEBUG docstring, full arch doc |
| 1635 | — | `NOOP_ALREADY_LANDED` | Recovery integration — confirmed b0249375 on main, no re-apply needed |

---

## Recommended Follow-Up Prompts

1. Human ruling on AC5 (see AC5 Ruling Options above).
2. `/story-readiness BOT-DEBUG-OVERLAY-001` after AC5 ruling.
3. Optional: live visual verify of F8 overlay in browser, 1280×720 (ADVISORY).
4. `/story-done` once readiness clears.

---

## Test Evidence (current)

- **Unit**: `tests/unit/bot/bot_debug_push_test.rs` — 377 lines, 25 tests, all PASS (PROMPT 1628).
- **Integration**: `tests/integration/playable_client/bot_debug_overlay_test.rs` — 280 lines; confirms no-mount when `CCGS_DEBUG_UI` unset (PROMPT 1632).
- **Architecture doc**: `docs/architecture/bot-debug-overlay.md` — data contract, operator workflow, release safety, test coverage (PROMPT 1630/1632).
- **Live visual verify**: ADVISORY-UNCONFIRMED — F8 overlay in browser, 1280×720, click passthrough (PROMPT 1621 blocked; advisory, not blocking unless Option C AC5 ruling adopted).
