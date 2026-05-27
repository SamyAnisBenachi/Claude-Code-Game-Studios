# PROMPT 1693 — Bot & Autoplay Story-Done Readiness After PROMPT 1678 Soak Pass

**Date:** 2026-05-27
**Source-of-truth at audit:** `origin/main@f9324431` (PROMPT 1678 soak pass record)
**Supersedes:** Crashed/killed PROMPT 1683
**Prior readiness audit:** PROMPT 1661 at `origin/main@9fa54ea7`
**Scope:** Read-only readiness map. No sprint-status, session-state, or source code edits.
**Sprint 19:** NOT activated. No story-done calls issued by this report.

---

## 1. Audit Scope and Sources

| Source | Path |
|---|---|
| Bot/autoplay epic | `production/epics/bot-and-autoplay/EPIC.md` |
| Story files | `production/epics/bot-and-autoplay/story-001..006.md` |
| Soak evidence | `production/qa/evidence/dev-runs/2026-05-27-125328-bot-vs-bot-soak` |
| PROMPT 1661 readiness audit | `reports/PROMPT-1661-bot-autoplay-story-done-readiness-after-1656.md` |
| PROMPT 1664 soak live verify | `reports/PROMPT-1664-bot-vs-bot-bounded-soak-live-verify.md` |
| PROMPT 1666 overlay status reconcile | `reports/PROMPT-1666-bot-debug-overlay-status-reconcile.md` |
| PROMPT 1667 AC1 inventory reconcile | `reports/PROMPT-1667-autoplay-recipe-library-ac1-inventory-reconcile.md` |
| PROMPT 1669 story-003 edit | `reports/PROMPT-1669-autoplay-recipe-library-ac1-story-edit.md` |
| PROMPT 1670 overlay story update | `reports/PROMPT-1670-bot-debug-overlay-story-status-ac5-ruling-prep.md` |
| PROMPT 1678 soak fix | `reports/PROMPT-1678-bot-placement-legal-unit-acquisition-repair.md` |
| PROMPT 1684 evidence audit | `reports/PROMPT-1684-bot-soak-evidence-completeness-audit-post-1678.md` |
| PROMPT 1691 sprint-18/19 readiness | `reports/PROMPT-1691-sprint-18-19-game-completion-readiness-refresh.md` |

---

## 2. Key Events Since PROMPT 1661 (Chronological)

| PROMPT | Verdict | Effect on Stories |
|---|---|---|
| 1666 | SHIPPED | Corrected story-005 status: impl IS on main. Story upgraded from NEEDS REPAIR → NEEDS VERIFY |
| 1667 | COMPLETE | AUTOPLAY-RECIPE-LIBRARY-001 AC1 reconcile: 12-name → 11-recipe mapping documented; `placement_reject_recovery` descoped from v1 |
| 1668 | (AC7 operator pack) | Bot-room AC7 live smoke operator runbook prepared; no live run executed yet |
| 1669 | SHIPPED | Story-003 file updated: AC1 table aligned with actual registry |
| 1670 | SHIPPED | Story-005 file updated: stale "not yet on main" language removed; AC5 ruling options documented |
| 1671 | SHIPPED | `Test-PortFree` false-negative fixed in `Start-BotVsBotSoak.ps1` (PROMPT 1664 Blocker B1 resolved) |
| 1672 | SHIPPED | Headless bot-room trigger path landed (`bot-soak-trigger` binary) — Blocker B2 resolved |
| 1673 | SHIPPED | `bot_soak_config_test` serial isolation fixed (`--test-threads=1` enforced) |
| 1674 | SHIPPED | Exit-code reconciliation in soak launcher trigger |
| 1675 | SHIPPED | Bot lobby auto-confirm repair: missing class selections no longer stall bot lobby join |
| 1677 | SHIPPED | Bot placement failsafe spam-debounce: prevents repeated failsafe submissions |
| **1678** | **SHIPPED** | **Two root-cause fixes: (1) empty-batch placement accepted when bot has no hand entry; (2) `bot_draft_auto_pick` system — bot now purchases cheapest card each draft phase. Soak result: PASS** |
| 1679 | SHIPPED | Stale-binary rebuild guard added to `Start-BotVsBotSoak.ps1` |
| 1684 | SHIPPED | Evidence completeness audit: 3 P0 + 6 P1 + 7 P2 gaps catalogued; none block current story-done on PASS run |

---

## 3. PROMPT 1678 Soak Result — Authoritative Evidence

**Run:** `production/qa/evidence/dev-runs/2026-05-27-125328-bot-vs-bot-soak`
**Exit code:** 0
**Outcome:** `endpoint_reached=game_over`, `received_game_over=true`
**Rounds completed:** 2 (MaxRoundsReached at round 3 trigger)
**Bot behavior observed:**
- Lobby: `bot_lobby_auto_confirm` Iop class confirmed (legal=6)
- DraftInitial R1: `bot_draft_auto_pick` purchased card 101; DraftReadySignal emitted
- Placement R1: `build_bot_placements` found 1 card; submitted non-empty placement (len=1)
- DraftShop R2: `bot_draft_auto_pick` purchased card 101 again; economy: +2g bot
- Placement R2: submitted non-empty placement (len=1)
- Resolution R1+R2: combat resolved, broadcasts successful
- GameOver: `MaxRoundsReached` broadcast, server clean exit

**All 133 tests pass** (including 6 new placement validation tests + 4 new bot auto-pick tests).

This is the first successful end-to-end headless bot-vs-bot soak with non-empty placements on `origin/main`.

---

## 4. Per-Story Readiness Map

### Story 001 — BOT-ROOM-PARTICIPANT-001
**Verdict: NEEDS VERIFY** *(unchanged from PROMPT 1661)*

| AC | Evidence | Status |
|----|----------|--------|
| AC1 Bot joins real room via C2S handshake | PROMPT 1430/1439 on main | PASS (static) |
| AC2 Bot picks class and confirms | PROMPT 1583 on main + 1675 repair | PASS (static) |
| AC3 Bot reaches SessionReady via ADR-012 Observer | PROMPT 1583 on main | PASS (static) |
| AC4 Bot bids in auctions (Wave 2/2.5) | PROMPT 1582/1598 on main | PASS (static) |
| AC5 Bot places units (Wave 3 heuristic) | PROMPT 1602 + 1678 auto-pick + empty-batch fix | PASS (static + headless) |
| AC6 Decision telemetry in QA snapshot + decision log | PROMPT 1597 on main | PASS (static) |
| **AC7** Human completes friend-game round against bot without panic | Not run | **HUMAN/LIVE GUI — OPEN** |
| **AC8** Bot lifecycle does not corrupt state | Deferred per PROMPT 1665 ruling | **OUT OF SCOPE for story-001** |

**Remaining blocker:** AC7 — human operator must run a friend-game vs bot, observe no server panic. No automated substitute.

**Path to done:** Sprint 19 activation → `/story-readiness` (cite PROMPT 1665 §5 for AC8 out-of-scope) → human operator live smoke → `/story-done`.

---

### Story 002 — BOT-SOAK-ENTRYPOINT-001
**Verdict: NEEDS PAPERWORK** *(upgraded from NEEDS LIVE VERIFY)*

This is the story most directly advanced by PROMPT 1678. Both PROMPT 1664 blockers are now resolved:
- **B1 (port detection):** Fixed by PROMPT 1671
- **B2 (no headless trigger):** Fixed by PROMPT 1672 (`bot-soak-trigger` binary)

| AC | Evidence | Status |
|----|----------|--------|
| AC1 `Start-BotVsBotSoak.ps1` launches bot-vs-bot game | Script + 1671/1674/1679 repairs; launcher drives soak | PASS |
| AC2 Both bots reach SessionReady + play full loop without panic | PROMPT 1678 soak: 2 rounds, game_over, exit 0 | **PASS (headless soak)** |
| AC3 `CCGS_BOT_MAX_ROUNDS` bounds run and exits cleanly | env var activates per soak log; MaxRoundsReached at round 3 | PASS |
| AC4 Per-round QA snapshot fields emitted | 8 snapshots in `server-snapshots/` (lobby → gameover) | PASS |
| AC5 Decision-log captured for both bots | `bot-decision-log.jsonl` 5 entries; lobby + draft + placement × 2 rounds | PASS |
| AC6 Soak entrypoint is debug-only / env-gated | Server only enters soak mode when `CCGS_BOT_MAX_ROUNDS` set | PASS |
| AC7 Documentation under `docs/autoplay/` | `docs/autoplay.md` covers soak invocation | PASS |

**P0 evidence gaps (PROMPT 1684) — not story-done blockers on current PASS run:**

| Gap | Severity | Story-Done Impact |
|----|----------|-------------------|
| Placement coordinates (lane/card/cell) absent from decision log | P0 | Advisory — does not invalidate PASS |
| `winner` + `game_over_reason` absent from `final_state.json` | P0 | Advisory — CI workaround: check `received_game_over + exit_code=0` |
| `BotState.last_decision_at_ms` frozen at Lobby in snapshots | P0 | Advisory — decisions confirmed in `bot-decision-log.jsonl` instead |

Per PROMPT 1691 assessment: *"These gaps do not block BOT-SOAK-ENTRYPOINT-001 story-done on the current PASS run."*

**Path to done:** Sprint 19 activation → `/story-readiness` (attach soak evidence path: `2026-05-27-125328-bot-vs-bot-soak`) → `/story-done`. No additional live run needed.

**Follow-up (separate PROMPTs, not gating story-done):** Fix P0-A, P0-B, P0-C per PROMPT 1684 recommendations.

---

### Story 003 — AUTOPLAY-RECIPE-LIBRARY-001
**Verdict: NEEDS LIVE VERIFY** *(AC1 gap closed; live GUI gate remains)*

| AC | Evidence | Status |
|----|----------|--------|
| **AC1** Recipe inventory (conceptual 12 → actual 11) | PROMPT 1667 reconcile + PROMPT 1669 story edit: mapping table in story file | **PASS (reconciled)** |
| **AC2** `full-game` recipe reaches full RESOLUTION in live GUI | Not yet run | **HUMAN/LIVE GUI GATE — OPEN** |
| AC3 Real UI input, no direct state mutation | Architecture enforced | PASS (static) |
| **AC4** Deterministic given fixed seed | Not verified | NEEDS VERIFY |
| AC5 CLI invocation produces structured report | `tools/autoplay/driver.py` + artifact schema | PASS (static) |
| **AC6** Failures surface failing step + QA snapshot | Validator 29 tests PASS (PROMPT 1651); live failure surfacing unverified | NEEDS LIVE RUN |
| AC7 Documentation lists every recipe | `docs/autoplay.md` refreshed PROMPT 1646 + README updated PROMPT 1655 | PASS |

**What changed since PROMPT 1661:** AC1 gap is now closed. The story file reflects the actual 11-recipe registry with the 12→11 mapping rationale. `placement_reject_recovery` is formally descoped to v1.1.

**Remaining open items:**
- AC2: Live `full-game` recipe run in browser with human operator
- AC4: Deterministic seed verify (can be done headlessly — no GUI gate, but no test exists yet)
- AC6: Live run confirming failure step surfacing

**Path to done:** Sprint 19 activation → human operator runs `Run-AutoplaySmoke.ps1 -Recipe full-game` in live GUI → confirm AC4/AC6 → `/story-done`.

---

### Story 004 — AUTOPLAY-VS-BOT-QA-001
**Verdict: NEEDS LIVE VERIFY + UPSTREAM GATES** *(no change from PROMPT 1661)*

All static/headless surfaces are green:
- 64 pytest autoplay recipe static tests PASS (PROMPT 1647)
- 29 pytest composite evidence validator PASS (PROMPT 1651/1657)
- 36 Rust bot soak integration tests PASS (PROMPT 1645)
- `Start-AutoplayVsBot.ps1` hardened (PROMPT 1648)
- Dev-launcher Autoplay vs Bot button wired (PROMPT 1652, 87 tests)
- Composite validator documented (PROMPT 1656)

| AC | Evidence | Status |
|----|----------|--------|
| AC1 Single invocation spawns server + autoplay client + bot | `Start-AutoplayVsBot.ps1` scaffold | PASS (static) |
| **AC2** Flow exercises full game loop phases | `-Recipe` passthrough fixed PROMPT 1655 | **NEEDS LIVE RUN** |
| AC3 Per-step QA snapshot fields captured | Infrastructure wired | NEEDS LIVE RUN |
| AC4 Decision-log (bot) + recipe-log (autoplay) captured | Infrastructure wired | NEEDS LIVE RUN |
| **AC5** Reaches at least one full RESOLUTION | — | **HUMAN/LIVE GUI GATE — OPEN** |
| AC6 Pass/fail verdict as structured report | `composite-summary.json` schema + validator PASS | PASS (static) |
| AC7 Documentation describes flow + artifacts | `evidence-operator-guide.md` §10 + `autoplay-vs-bot-flow.md` | PASS |

**Hard upstream gates still open:** Stories 001, 002, 003 must reach `/story-done` before story-004 can advance.

**Path to done:** Stories 001+002+003 done → Sprint 19 activation → human operator runs `Start-AutoplayVsBot.ps1` → composite harness reaches RESOLUTION → `/story-done`.

---

### Story 005 — BOT-DEBUG-OVERLAY-001
**Verdict: NEEDS VERIFY** *(corrected from NEEDS REPAIR per PROMPT 1666)*

PROMPT 1661 incorrectly reported "no implementation has landed." PROMPT 1666 confirmed full implementation on `origin/main`. PROMPT 1670 updated the story file.

| AC | Evidence | Status |
|----|----------|--------|
| AC1 Overlay only when `CCGS_DEBUG_UI=1` | `debug_bot_overlay.rs:43`; integration test confirms no-spawn | PASS |
| AC2 F8 toggles overlay visibility | `KeyCode::F8` toggle in code; 7 client tests PASS | PASS (code); live visual ADVISORY |
| AC3 Overlay renders lobby/auction/placement bot state | PROMPT 1623 §4 audit: all fields confirmed | PASS |
| AC4 Data from server-pushed `S2CDebugBotStatePush` | Protocol wired server→client | PASS |
| **AC5** Overlay not built into release builds | Runtime env-gating (`cfg(debug_assertions)` + `CCGS_DEBUG_UI`) — NOT compile-time exclusion | **HUMAN DECISION NEEDED** |
| AC6 Overlay never blocks game input | `should_block_lower=false` confirmed PROMPT 1623 §3 | PASS |
| AC7 Documentation | `docs/architecture/bot-debug-overlay.md` 342 lines | PASS |

**AC5 ruling options (from PROMPT 1670):**
- Option A: Accept runtime env-gating as satisfying AC5 (no code change needed)
- Option B: Update AC5 wording to match shipped runtime-gated design (story edit only)
- Option C: Require true compile-time `#[cfg(not(feature="release"))]` exclusion (code change required)

**Implementation on main:**
- `client/src/presentation/debug_bot_overlay.rs` (499 lines)
- `server/src/feature/bot/debug_push.rs` (653 lines)
- `docs/architecture/bot-debug-overlay.md` (342 lines)
- `tests/unit/bot/bot_debug_push_test.rs` (25 tests PASS)
- `tests/integration/playable_client/bot_debug_overlay_test.rs` (280 lines)

**Path to done:** Human decision on AC5 at `/story-readiness` (choose Option A/B/C) → Sprint 19 activation → `/story-done`. Optional: live visual verify of F8 toggle in browser (ADVISORY, not blocking).

---

### Story 006 — BOT-DISCONNECT-REJOIN-006
**Verdict: BLOCKED — Sprint 20+** *(no change)*

No implementation. Story file authored by PROMPT 1650. Hard gate on story-001 done. Sprint 20+ candidate.

---

## 5. Consolidated Readiness Table

| Story | ID | Verdict | Since 1661 | Primary Blocker | Path to Done |
|---|---|---|---|---|---|
| Bot Room Participant | BOT-ROOM-PARTICIPANT-001 | **NEEDS VERIFY** | No change | Sprint 19 + AC7 human live smoke | Sprint 19 → `/story-readiness` → human smoke → `/story-done` |
| Bot-vs-Bot Soak Entrypoint | BOT-SOAK-ENTRYPOINT-001 | **NEEDS PAPERWORK** | **Upgraded** — PASS soak obtained | Sprint 19 activation only | Sprint 19 → `/story-readiness` → `/story-done` (no new run needed) |
| Autoplay Recipe Library v1 | AUTOPLAY-RECIPE-LIBRARY-001 | **NEEDS LIVE VERIFY** | AC1 gap closed | Sprint 19 + live `full-game` recipe run | Sprint 19 → live full-game run → `/story-done` |
| Autoplay-vs-Bot QA Flow | AUTOPLAY-VS-BOT-QA-001 | **NEEDS LIVE VERIFY** | No change | Upstream 001+002+003 + live composite run | Upstream done → Sprint 19 → live composite → `/story-done` |
| Bot Debug Overlay | BOT-DEBUG-OVERLAY-001 | **NEEDS VERIFY** | **Corrected from NEEDS REPAIR** | Sprint 19 + AC5 human ruling | AC5 ruling at `/story-readiness` → Sprint 19 → `/story-done` |
| Bot Disconnect/Rejoin | BOT-DISCONNECT-REJOIN-006 | **BLOCKED** | No change | story-001 not done, no impl | story-001 done → Sprint 20 → `/dev-story` |

---

## 6. Live GUI Gates — Human Required

| Story | Gate | What Is Needed |
|---|---|---|
| BOT-ROOM-PARTICIPANT-001 (AC7) | OPEN | Human runs friend-game vs bot in live browser; no server panic observed |
| AUTOPLAY-RECIPE-LIBRARY-001 (AC2) | OPEN | Human runs `Run-AutoplaySmoke.ps1 -Recipe full-game` against live server+client |
| AUTOPLAY-VS-BOT-QA-001 (AC5) | OPEN (upstream gated) | Human runs `Start-AutoplayVsBot.ps1`; composite harness reaches RESOLUTION |
| BOT-DEBUG-OVERLAY-001 (AC5) | DECISION PENDING | Human chooses Option A/B/C at `/story-readiness` for runtime vs compile-time exclusion |

No automated substitute exists for any of these gates.

---

## 7. Implementation Follow-Up Work (Not Blocking Story-Done)

The following items emerge from PROMPT 1684's evidence audit. They do not gate any current story-done but represent instrumentation debt that will make the next soak failure diagnosable:

| Priority | Finding | Fix Needed |
|---|---|---|
| P0-A | Placement coordinates absent from `placement_submitted` decision log | Add `placed_lane`, `placed_card_id` to decision log entry |
| P0-B | `winner` + `game_over_reason` absent from `final_state.json` and GameOver snapshot | Add structured fields to both |
| P0-C | `BotState.last_decision_at_ms` frozen at Lobby value in all post-lobby snapshots | Fix serialization: update field on every decision write |
| P1-A | `legal_action_count: null` for draft phases in decision log | Populate with shop offering size at decision time |
| P1-D | Player 1 (trigger client) placed 0 units across both rounds — not flagged | Add per-player placement count check to soak pass criteria |

---

## 8. What Is NOT Needed Before Story-Done (Clarifications)

- **No new soak run is needed for story-002.** The `2026-05-27-125328-bot-vs-bot-soak` evidence (exit 0, 2 rounds, game_over) is sufficient. The P0 evidence gaps from PROMPT 1684 are instrumentation improvements, not soak invalidators.
- **Story-005 does not need a new `/dev-story`.** The implementation is complete and on main. Only AC5 human ruling + Sprint 19 activation are needed.
- **Story-003 AC1 is closed.** The 12→11 recipe mapping is reconciled. No new recipes need to be written before story-done (the optional `placement-reject-probe` recipe is deferred to v1.1).

---

## 9. Confidence and Caveats

- This audit reads git log, story files, and prior PROMPT reports. No Cargo run, no GUI session.
- Source of truth is `origin/main@f9324431`. PROMPTs 1681, 1682, 1685 were listed as in-flight in PROMPT 1691; their completion reports were not found in the reports directory at audit time. If they shipped, their outputs may update the readiness map for stories 001 or 002 further.
- Sprint-status.yaml was not read in full (1.3 MB); story-done verdicts are derived from git log + story file cross-reference.
- PROMPT 1691 is a parallel audit by a different agent; where its findings align with this report, the agreement increases confidence. Where they differ, treat this report as the bot/autoplay-specific authority.

---

1693: BOT-AUTOPLAY-STORY-DONE-READINESS-AFTER-1678-RECOVERY: SHIPPED
