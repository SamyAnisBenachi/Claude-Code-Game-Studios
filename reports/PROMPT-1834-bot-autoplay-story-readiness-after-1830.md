# PROMPT 1834 — BOT-AUTOPLAY-STORY-READINESS-AFTER-1830

**Status:** DONE — read-only readiness assessment  
**Date:** 2026-05-28  
**Worktree:** `tmpwt-1834-bot-autoplay-readiness`  
**Branch:** `wt/1834-bot-autoplay-readiness`  
**HEAD:** `71484998` (origin/main)  
**Sources read:** epic EPIC.md + stories 001-006, reports/PROMPT-1758, PROMPT-1827 through PROMPT-1831, docs/autoplay/**

---

## Executive Summary

Six stories exist in the `bot-and-autoplay` epic. After the landings through
PROMPT 1830 / 1831:

| Story | ID | Implementation | Paperwork gate | Can story-done now? |
|---|---|---|---|---|
| 001 — Bot Room Participant | BOT-ROOM-PARTICIPANT-001 | **Complete on origin/main** | Sprint 19 not activated | **No — needs Sprint 19 activation** |
| 002 — Bot-vs-Bot Soak | BOT-SOAK-ENTRYPOINT-001 | **Complete on origin/main** | Sprint 19 not activated | **No — needs Sprint 19 activation** |
| 003 — Autoplay Recipe Library v1 | AUTOPLAY-RECIPE-LIBRARY-001 | **Substantially complete** | Sprint 19 not activated + `/story-readiness` at tip | **No — needs Sprint 19 activation** |
| 004 — Autoplay-vs-Bot QA Flow | AUTOPLAY-VS-BOT-QA-001 | Harness built; 2 PASS runs | **Human GUI sign-off outstanding** | **No — blocked on human operator sign-off** |
| 005 — Bot Debug Overlay | BOT-DEBUG-OVERLAY-001 | **Complete on origin/main** | **AC5 ruling outstanding** | **No — blocked on AC5 human ruling** |
| 006 — Bot Disconnect/Re-join | BOT-DISCONNECT-REJOIN-006 | Not started | Sprint 20+; gated on 001 story-done | **No — future sprint** |

**No story can be marked story-done today.** The primary blockers are:
(A) Sprint 19 is not activated, (B) human GUI sign-off on story 004, and
(C) human AC5 ruling on story 005.

---

## Story-by-Story Assessment

### Story 001 — BOT-ROOM-PARTICIPANT-001 (Bot Room Participant)

**Epic status:** Draft — ledger placeholder for landed work  
**Sprint placement:** Sprint 18 carry-tracking; story-done deferred to Sprint 19

#### Implementation (on origin/main as of 71484998)

All implementation waves confirmed landed:

| PROMPT | Slice |
|--------|-------|
| 1430 | Protocol room foundations |
| 1439 | Bot foundation scaffold |
| 1531 / 1582 | Action loop Waves 1 + 2 |
| 1583 | Lobby ready auto-confirm |
| 1598 | Bid funnel Wave 2.5 |
| 1602 | Wave 3 placement heuristic |

#### AC status (current read)

- **AC1–AC6**: Covered by the landed implementation waves. No targeted
  evidence report exists; readiness tooling would verify each at
  Sprint 19 activation.
- **AC7** (human completes a round against bot without server panic):
  Two live vs-bot runs (PROMPT 1758 @ `20260528-014126-Z` and PROMPT 1831
  @ `20260528-090613-Z`) both reached `vs-bot-post-resolution` with
  exit 0. This provides strong informal evidence for AC7 but is
  **not a formal `/story-done` AC sign-off.**
- **AC8** (bot lifecycle — disconnect/re-join): Explicitly deferred to
  Story 006 (Sprint 20+). Not blocking Story 001 story-done.

#### Readiness verdict

**CANNOT story-done.** Sprint 19 has not been activated. Once activated:

1. `/story-readiness BOT-ROOM-PARTICIPANT-001` to produce formal AC verdicts.
2. The two live PROMPT 1758 / 1831 PASS runs may serve as informal evidence
   for AC7 (manual smoke); the producing agent should confirm whether they
   satisfy the integration evidence gate.
3. `/story-done BOT-ROOM-PARTICIPANT-001`.

No implementation repair needed. Paperwork only.

---

### Story 002 — BOT-SOAK-ENTRYPOINT-001 (Bot-vs-Bot Soak Entrypoint)

**Epic status:** Draft — Sprint 19 candidate  
**Sprint placement:** Sprint 19 candidate (NOT activated)

#### Implementation

Per PROMPT 1762 (reconcile audit at `origin/main@7ca41fc4`):

- PROMPT 1607 integration confirmed main-landed at `origin/main@7ca41fc4`.
- Full implementation lineage 1603–1743 confirmed on `origin/main`.
- All AC1–AC6 deliverables present.
- Integration tests present (PROMPT 1762 AC5 evidence at commit `c84f03be`).

#### AC status (per PROMPT 1762 — most recent authoritative scan)

| AC | Verdict |
|----|---------|
| AC1 — `Start-BotVsBotSoak.ps1` exists | PASS (PROMPT 1762) |
| AC2 — Both bots reach SessionReady, full loop no panic | PASS (PROMPT 1762) |
| AC3 — `--bot-vs-bot-max-rounds` flag | PASS (PROMPT 1762) |
| AC4 — Per-round QA snapshots emitted for both bots | PASS (PROMPT 1762) |
| AC5 — Decision-log stream captured (integration tests) | PASS (PROMPT 1762, commit c84f03be) |
| AC6 — Debug-only / env-gated | PASS (PROMPT 1762) |
| AC7 — Docs under docs/autoplay/ | Verify at readiness |

#### Readiness verdict

**CANNOT story-done.** Sprint 19 not activated. When activated:

1. `/story-readiness BOT-SOAK-ENTRYPOINT-001` — expect green on AC1–AC6.
2. Confirm AC7 docs presence (PROMPT 1758 evidence references
   `docs/autoplay/bot-vs-bot-soak.md`; likely already present).
3. `/story-done BOT-SOAK-ENTRYPOINT-001` — paperwork only.

Recommended evidence artifact: run `Start-BotVsBotSoak.ps1 --max-rounds 3`
and capture the output file under `production/qa/evidence/` before `/story-done`.
PROMPT 1678 and PROMPT 1758 are the existing live soak citations.

---

### Story 003 — AUTOPLAY-RECIPE-LIBRARY-001 (Autoplay Recipe Library v1)

**Epic status:** Draft — Sprint 19 candidate  
**Sprint placement:** Sprint 19 candidate (NOT activated)

#### Implementation

Recipe registry as of PROMPT 1667 AC1 reconciliation:

| Registry name | Landed via |
|---|---|
| `smoke` | PROMPT 1609 |
| `idle` | PROMPT 1609 |
| `add-bot-lobby` | PROMPT 1634 |
| `lobby-create` | PROMPT 1636 |
| `class-select` | PROMPT 1636 |
| `draft-auction-probe` | PROMPT 1639 |
| `placement-drag-probe` | PROMPT 1639 |
| `resolution-observe` | PROMPT 1636 |
| `game-over-observe` | PROMPT 1636 |
| `round-loop` | PROMPT 1655 |
| `full-game` | PROMPT 1655 |

**Descoped from v1:** `placement_reject_recovery` — no standalone recipe.
The rejection-recovery UX is partially exercised inside `full-game` but not
addressable by name. AC1 non-blocker per story 003 text; candidate for a
future v1.1 or separate story ticket.

#### AC status

- **AC1** (named recipes covering full flow): PASS — 11 recipes in registry;
  reconciled by PROMPT 1667; all AC1 origins mapped (with `lobby_join`
  superseded by `add-bot-lobby` and `draft_initial`/`shop_buy`/`auction_bid`
  merged into `draft-auction-probe`).
- **AC2** (`full_friend_game` reaches at least one RESOLUTION): Supported by
  PROMPT 1758 and PROMPT 1831 — both runs reached `vs-bot-post-resolution`
  using `full-game` recipe. Informal evidence; formal gate is at readiness.
- **AC3** (real UI input, no direct state mutation): Design constraint; verify
  at readiness.
- **AC4** (determinism given fixed seed): Not formally verified; needs
  `/story-readiness`.
- **AC5** (CLI invocable, pass/fail report): Demonstrated in every live run.
- **AC6** (failures surface exact step): Demonstrated by checkpoint/driver.log
  structure in evidence runs.
- **AC7** (docs listing every recipe): `docs/autoplay/` updated; verify at
  readiness.

#### Placement rejection recipe/analyzer — current state

No dedicated `placement-reject-probe` recipe or analyzer exists in the
registry. The rejection-recovery path is partially exercised inside
`full-game` per story 003 text. A standalone recipe or analyzer tooling
(if needed) would require either:
- A separate PROMPT implementing `placement-reject-probe` recipe (v1.1 work).
- Or a post-placement analyzer script in `tools/autoplay/` that parses
  `checkpoints.jsonl` for rejection events and reports them.

Neither is currently on the queue; both are deferred.

#### Readiness verdict

**CANNOT story-done.** Sprint 19 not activated. When activated:

1. `/story-readiness AUTOPLAY-RECIPE-LIBRARY-001` at activation tip.
2. Expect green on AC1/AC2/AC5/AC6 immediately; AC4 (determinism) and
   AC3 (no state mutation) may need a dedicated verify step.
3. `/story-done` after readiness clears.

---

### Story 004 — AUTOPLAY-VS-BOT-QA-001 (Autoplay-vs-Bot QA Flow)

**Epic status:** Draft — Sprint 19 candidate (gated on 001 + 002 + 003)  
**Sprint placement:** Sprint 19 candidate (NOT activated)

#### Infrastructure state

| Component | Status |
|---|---|
| `Start-AutoplayVsBot.ps1` composite launcher | Landed (PROMPT 1644) |
| `Run-AutoplaySmoke.ps1` | Landed (PROMPT 1757 refresh) |
| `vs-bot` recipe | Landed (PROMPT 1655 lineage) |
| `validate_composite_run.py` | Landed |
| Composite evidence schema | Landed |

#### Live run history

| Run | Timestamp | Result | Notes |
|---|---|---|---|
| PROMPT 1758 | `20260528-014126-Z` | PASS (exit 0, all 15 checkpoints) | Pre-1818 driver (frozen PrintWindow, no fallback) |
| PROMPT 1831 | `20260528-090613-Z` | PASS (exit 0, all 15 checkpoints, 10 distinct hashes) | Post-1818 driver with frozen-fallback chain working |

The PROMPT 1831 run fully satisfies the scripted PASS criteria from the
`autoplay-vs-bot-flow.md` live PASS gate (steps 1–4):
- ✅ `Start-AutoplayVsBot.ps1` run in interactive session with visible desktop
- ✅ `full-game`-family recipe (`vs-bot`) exits 0
- ✅ `composite-summary.json` → `outcome: ok`
- ✅ All 15 checkpoints reached including `vs-bot-post-resolution`

**The remaining gate is step 5: human operator review and sign-off.**

Per the launcher and flow docs: "An operator must review artifacts and sign
off." The `live_pass_status` in every `composite-summary.json` is
`"NOT-CLAIMED — AUTOPLAY-VS-BOT-QA-001 requires human operator sign-off for
live PASS evidence."` No agent-authored report can substitute for this.

#### GAP-01 / GAP-02 status

The `autoplay-vs-bot-flow.md` Live PASS Gate section cites GAP-01 and GAP-02
as open until steps 1–5 are completed by a human operator. PROMPT 1831 closes
steps 1–4 via the agent-executed post-1818 run. GAP-01 and GAP-02 remain
**formally open** pending human sign-off on the artifacts.

#### Readiness verdict

**BLOCKED on human GUI sign-off.** The evidence infrastructure is working
correctly. The required human action is:

1. Operator inspects `production/qa/evidence/autoplay-runs/20260528-090613-Z/`
   (the PROMPT 1831 post-1818 run) or runs a fresh `Start-AutoplayVsBot.ps1`.
2. Operator confirms: outcome=ok, exit 0, all checkpoints, ≥3 distinct
   `pixel_hash` values.
3. Operator signs off and adds attestation to the story or a QA evidence doc.
4. Then: Sprint 19 activation → `/story-readiness` → `/story-done`.

Note: This story is also gated on 001/002/003 readiness, but human sign-off
is the near-term critical path.

---

### Story 005 — BOT-DEBUG-OVERLAY-001 (Debug-Only Bot Overlay)

**Epic status:** Draft — Sprint 19 candidate  
**Sprint placement:** Sprint 19 candidate (NOT activated)

#### Implementation (on origin/main)

All commits confirmed on origin/main per PROMPT 1666 reconcile audit:

| PROMPT | Commit | Slice |
|--------|--------|-------|
| 1614 | `37306162` | Core overlay: `debug_bot_overlay.rs`, `debug_push.rs`, `S2CDebugBotStatePush`, 16 tests |
| 1617 | — | Integration refresh (main-land) |
| 1618 | — | Compile verify: PASS (shared/server/client) |
| 1628 | — | 25 standalone `bot_debug_push` unit tests, all PASS |
| 1630 | — | `docs/architecture/bot-debug-overlay.md` (342 lines) |
| 1632 | `b0249375` | UX polish + arch doc + tests; tail-cap align; DEBUG docstring |

#### AC status (per PROMPT 1670 assessment)

| AC | Verdict | Blocker level |
|----|---------|---------------|
| AC1 — CCGS_DEBUG_UI=1 gate | **PASS** | — |
| AC2 — F8 toggle (code) | **PASS (code)** | Live visual = ADVISORY |
| AC3 — Overlay renders 3 decision fields | **PASS** | — |
| AC4 — Server-pushed data contract | **PASS** | — |
| AC5 — Not in release builds | **NEEDS HUMAN RULING** | **Story-done blocker** |
| AC6 — Does not block game input | **PASS** | — |
| AC7 — Architecture doc | **PASS** | — |

#### AC5 ruling options (human must choose one)

Three mutually exclusive options are documented in story 005:

**Option A** — Accept runtime env-gating as satisfying AC5 (no code change;
reinterpret "compile-time exclusion" broadly). Mark AC5 PASS.

**Option B** — Update story AC5 wording to match implementation, then pass
(story edit required; no code change). Mark AC5 PASS after edit.

**Option C** — Require true compile-time exclusion (Cargo feature flag;
code change required before story-done; AC5 NOT PASS until implemented).

Until the human chooses and the ruling is recorded, `/story-readiness` cannot
close AC5, blocking `/story-done`.

#### Live visual verify (ADVISORY)

The F8 overlay in browser at 1280×720 with click passthrough has not been
confirmed by a human operator (PROMPT 1621 was blocked). This is ADVISORY
unless Option C is adopted for AC5 (which would also require a new PROMPT).

#### Readiness verdict

**BLOCKED on AC5 human ruling.** When ruling is made:

1. If Option A or B: edit story if needed, run `/story-readiness` → `/story-done`.
2. If Option C: spawn implementation PROMPT for Cargo feature flag, then
   `/story-readiness` → `/story-done`.

Sprint 19 activation is also required before `/story-done`.

---

### Story 006 — BOT-DISCONNECT-REJOIN-006 (Bot Disconnect/Re-join Hardening)

**Epic status:** Draft — future-sprint candidate (Sprint 20+)  
**Sprint placement:** Unscheduled — NOT activated

No implementation exists. Story 006 is a specification-and-design placeholder.

**Hard gate:** BOT-ROOM-PARTICIPANT-001 must be story-done before Sprint 20
activation for this story.

Not actionable until Story 001 closes and Sprint 20 is planned.

---

## Placement Rejection Recipe / Analyzer — Status

The task description asks about "placement rejection recipe/analyzer."

Current state:
- **No standalone recipe** (`placement-reject-probe`) exists in the registry.
- The rejection-recovery UX path (unit bounces back to hand after server
  rejection) is **partially exercised inside `full-game`** but not
  addressable by name and not separately verified.
- Story 003 explicitly descopes this from v1 (not an AC1 blocker).

**Needs implementation if required:** A `placement-reject-probe` recipe or
a post-run analyzer script (e.g., `tools/autoplay/analyze_rejections.py`
parsing `checkpoints.jsonl` for rejection events) would need a new PROMPT.
This is a Sprint 19+ candidate, not currently queued.

---

## Stories That CAN Be Story-Done (after prereqs)

No story can be story-done today. However, the following can move to
story-done quickly **once Sprint 19 is activated**:

| Story | Next action | Estimated effort |
|---|---|---|
| **002** (Soak Entrypoint) | Sprint 19 activation → `/story-readiness` → `/story-done` | Paperwork only; implementation complete |
| **001** (Bot Room Participant) | Sprint 19 activation → `/story-readiness` → `/story-done` | Paperwork only; may cite PROMPT 1758/1831 for AC7 |
| **003** (Recipe Library v1) | Sprint 19 activation → `/story-readiness` → `/story-done` | Readiness check needed for AC4/AC7; rest likely green |

**Story 005** can follow quickly once the human AC5 ruling is recorded.

**Story 004** requires human operator sign-off on the PROMPT 1831 artifacts
(or a new operator-witnessed run) before it can close.

---

## Stories Blocked on 1831 Live Evidence

**Story 004 (AUTOPLAY-VS-BOT-QA-001)** is the only story requiring live
evidence from PROMPT 1831 for its critical path. PROMPT 1831's post-1818
run at `20260528-090613-Z` provides the scripted PASS artifact; the
remaining step is human operator sign-off (non-agent, non-automatable).

---

## Stories Requiring Human GUI Sign-off

| Story | Sign-off needed | Current evidence state |
|---|---|---|
| **004** (AUTOPLAY-VS-BOT-QA-001) | Human operator review of `20260528-090613-Z` artifacts and attestation | Two PASS runs exist (1758 pre-1818, 1831 post-1818); sign-off only outstanding step before Sprint 19 |
| **005** (BOT-DEBUG-OVERLAY-001) | Human ruling on AC5 (Option A/B/C) + optional live F8 visual verify (ADVISORY) | AC2 code PASS; live F8 ADVISORY-UNCONFIRMED |

---

## Recommended Next Actions

In priority order:

1. **[Human] AC5 ruling for Story 005** — pick Option A, B, or C from story
   file § "AC5 Ruling Options." Unblocks Story 005 story-done.

2. **[Human] PROMPT 1831 artifact sign-off for Story 004** — inspect
   `production/qa/evidence/autoplay-runs/20260528-090613-Z/`, confirm
   all BLOCKING checklist items from PROMPT 1827 §5 are green, and record
   operator attestation. Unblocks Story 004's live PASS gate.

3. **[Orchestrator] Sprint 18 close-out → Sprint 19 activation** — gating
   blocker for Stories 001, 002, 003, 004, 005. None of these can be
   story-done until Sprint 19 is active.

4. **[Post Sprint-19-activation] Run `/story-readiness` on 001, 002, 003
   in sequence** — 002 expected to clear immediately; 001 and 003 may
   need minor evidence artifacts (bounded soak output, AC4 determinism).

5. **[Future] `placement-reject-probe` recipe** — if required, spawn a new
   PROMPT to implement the standalone recipe or analyzer tooling under
   `tools/autoplay/`. Not currently blocking any story-done path.

---

1834: BOT-AUTOPLAY-STORY-READINESS-AFTER-1830: DONE
