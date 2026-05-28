# PROMPT 1835 — Game-Completion Next-Wave Map (After PROMPT 1830)

**Date:** 2026-05-28  
**Source-of-truth tip:** `origin/main@71484998` (PROMPT 1830)  
**Stage:** Polish (locked)  
**Active sprint:** Sprint 18 (activated 2026-05-18; ~27% complete)  
**Known active lane:** PROMPT 1831 — fresh autoplay vs-bot live verify

---

## Executive Summary

The last 24 commits (PROMPTs 1806–1830) are entirely autoplay/screenshot-capture
infrastructure: Win32 PrintWindow, BitBlt fallback, frozen-frame detection, env
gating, stale-pyc guards, and report backfills. No gameplay source code has
changed since the PROMPT 1471 repair chain main-land. Sprint 18 is 27% complete
(3 of 12 rows closed). The Polish→Release gate-check (PROMPT 761) failed and has
not been retried. The next unlock is a live two-client evidence run after the
autoplay vs-bot lane (PROMPT 1831) resolves.

---

## Current Blockers by Category

| Blocker | Scope | Unblocked By |
|---------|-------|--------------|
| PROMPT 1472 POST-REPAIR-LIVE-TWO-CLIENT-QA-RETEST not yet completed | `client/src/ui/hand/`, `client/src/presentation/board_rendering.rs`, `client/src/ui/shop_auction/`, `client/src/presentation/qa_snapshot.rs` | PROMPT 1472 verify lane |
| Frozen pixel hash in vs-bot evidence runs (all 15 captures `0x26207c4c`) | autoplay, PROMPT 1831 | Post-1818 live run with bot room + human GUI active |
| Branch `integrate/auction-won-card-disposition-1141` outstanding | S18-AUCTION-WON-CARD-DISPOSITION-001 | Integration worker + story-done |
| Polish→Release gate-check FAIL (PROMPT 761, no retry) | release readiness | Sprint 18/19 gate-check retry |
| S11-HUD-TIMER-EYEBALL-VISUAL-001 human-operator blocked carry | HUD visual | Human visual sign-off |
| PROMPT 1054 UI snapshot visual retest BLOCKED-HUMAN-OPERATOR | QA evidence | Human GUI session |

---

## Parallel Lane Map

### LANE 1 — Autoplay/Bot Live Evidence (sequential with PROMPT 1831)

**Current:** PROMPT 1831 fresh autoplay vs-bot live verify (active).

All captures in the most recent evidence run (20260528-063609-Z) show frozen pixel
hash `0x26207c4c` — the BitBlt fallback fires but captures a static frame. A live
run with the bot room and game actually progressing is needed to validate post-1818
capture quality.

**Ready now (after 1831 completes):**

| Prompt Title | Scope | Dependencies |
|---|---|---|
| `PROMPT-1836-VSBOT-LIVE-EVIDENCE-POST-1831-VERIFY` | Run autoplay with bot room + human GUI; confirm distinct frames; post new evidence to `production/qa/evidence/autoplay-runs/` | PROMPT 1831 done |
| `PROMPT-1837-AUTOPLAY-SOAK-RECIPE-VALIDATION` | Run 10-game soak; measure win/loss/crash ratio; update `BOT-SOAK-ENTRYPOINT-001` story evidence | 1836 PASS |

**Conflict risk:** Low — evidence output only, no source edits.

---

### LANE 2 — Sprint 18 Must Have Completions (parallel-safe now)

Sprint 18 has 2 open Must Have rows that do not touch autoplay/screenshot code:

#### 2A. S18-AUCTION-WON-CARD-DISPOSITION-001 (0.75d)
Branch `integrate/auction-won-card-disposition-1141` is outstanding. This is the
auction-won card disposition flow: when a card is won at auction it must flow to
the player's hand/board.

**Files at risk:** `server/src/feature/auction/`, `client/src/ui/shop_auction/`,
`shared/src/protocol.rs`

| Prompt Title | Scope | Dependencies |
|---|---|---|
| `PROMPT-1838-AUCTION-WON-CARD-DISPOSITION-INTEGRATION` | Merge/rebase `integrate/auction-won-card-disposition-1141`; resolve conflicts; smoke-pass; story-done S18 row | PROMPT 1472 retest done (shop_auction conflict risk) |
| `PROMPT-1839-AUCTION-WON-CARD-DISPOSITION-VERIFY` | Verify-only: confirm card flows to hand; screenshot evidence | 1838 SHIP |

**Conflict risk:** HIGH with `client/src/ui/shop_auction/` — do not parallelize with
other shop_auction workers. Safe to parallelize with Lane 1 (no file overlap).

#### 2B. S18-UI-PLAY-AREA-CONTAINER-001 (0.75d)
PROMPT 1180 Lane A structural enabler — play area container unimplemented.
This is the structural Bevy UI container that hosts the board + HUD layout.

**Files at risk:** `client/src/ui/`, `client/src/presentation/`

| Prompt Title | Scope | Dependencies |
|---|---|---|
| `PROMPT-1840-UI-PLAY-AREA-CONTAINER-IMPL` | Implement play-area container node per Sprint 18 story spec; smoke; story-done | PROMPT 1472 retest done (board_rendering conflict risk) |

**Conflict risk:** HIGH with `client/src/presentation/board_rendering.rs` — hold until
PROMPT 1472 resolves.

---

### LANE 3 — Sprint 18 Should Have Completions (mostly paperwork-safe now)

These rows are implemented but need paperwork/story-done or minor verify work:

| Prompt Title | Row | Type | Safe Now? |
|---|---|---|---|
| `PROMPT-1841-S18-VIEWPORT-INVARIANT-STORY-DONE` | S18-UI-VIEWPORT-INVARIANT-LIVE-HARNESS-001 | Paperwork-only | YES |
| `PROMPT-1842-S18-SNAPSHOT-LAYOUT-FIELDS-STORY-DONE` | S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001 | Paperwork-only | YES |
| `PROMPT-1843-S18-SETTINGS-PANEL-RELAYOUT-VERIFY` | S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001 | Verify-only | YES |
| `PROMPT-1844-S18-CARD-ART-LABEL-STRIP-IMPL` | S18-UI-CARD-ART-AND-LABEL-STRIP-001 | Implement (PROMPT 1180 Lane C) | After PROMPT 1472 |

**Paperwork prompts (1841, 1842, 1843) are safe to run now in parallel.**
No source file edits; only sprint tracker + story state updates.

---

### LANE 4 — Gameplay Loop Gaps (gated on PROMPT 1472)

The PROMPT 1471 repair chain landed 20+ repairs, but the following gameplay
systems are either unimplemented or verified only at snapshot level, not live:

#### 4A. Round State Machine Completeness
`server/src/core/` hosts the round state machine. GDD `design/gdd/round-state-machine.md`
specifies phases: DRAFT → AUCTION → PLACEMENT → COMBAT → SHOP → ROUND_END.

| Prompt Title | Scope | Dependencies |
|---|---|---|
| `PROMPT-1845-ROUND-STATE-MACHINE-COMPLETENESS-AUDIT` | Read-only audit: compare round-state-machine.md acceptance criteria vs server/src/core/ implementation; list gaps | None (read-only) |
| `PROMPT-1846-ROUND-STATE-MACHINE-GAP-REPAIR` | Implement gaps found in 1845 audit | 1845 + PROMPT 1472 |

#### 4B. Combat Resolution
`server/src/feature/combat/` — GDD `design/gdd/combat-resolution.md`. Combat
formulas and keyword interactions (Speed, Bulwark, Ecaflip) need live verification.

| Prompt Title | Scope | Dependencies |
|---|---|---|
| `PROMPT-1847-COMBAT-RESOLUTION-LIVE-VERIFY` | Run two-client session through combat phase; capture combat log + result; verify vs GDD formula | PROMPT 1472 + human GUI |
| `PROMPT-1848-COMBAT-KEYWORD-INTEGRATION-TESTS` | Write integration tests for Speed/Bulwark/Ecaflip interactions per GDD acceptance criteria | None (test authoring only) |

#### 4C. Objective System
`server/src/core/` — GDD `design/gdd/objective-system.md`. ADR-024 (Sang Méprise
reveal) defines the reveal protocol. `ObjectiveIdentityCache` lifecycle is
specified but live verify evidence is absent from recent commits.

| Prompt Title | Scope | Dependencies |
|---|---|---|
| `PROMPT-1849-OBJECTIVE-SYSTEM-SANG-MEPRISE-LIVE-VERIFY` | Trigger Sang Méprise reveal in live two-client session; confirm unicast delivery; capture evidence | PROMPT 1472 |

**File conflict risk:** `server/src/core/` — do not overlap multiple workers.

---

### LANE 5 — Multiplayer / State / Snapshots

The QA snapshot system (fields added in PROMPT 1229) captures layout fields for
automated verification. The snapshot `CCGS_QA_SNAPSHOT=1` button is live since
`origin/main@8a3744e`.

| Prompt Title | Scope | Dependencies |
|---|---|---|
| `PROMPT-1850-QA-SNAPSHOT-COVERAGE-AUDIT` | Read-only: list all snapshot fields in `client/src/presentation/qa_snapshot.rs`; compare vs GDD acceptance criteria coverage; identify gaps | None (read-only) |
| `PROMPT-1851-QA-SNAPSHOT-GAP-REPAIR` | Add missing snapshot fields; update evidence baseline | 1850 + PROMPT 1472 (qa_snapshot.rs conflict risk) |
| `PROMPT-1852-RECONNECT-TRACKER-LIVE-VERIFY` | Verify `ReconnectTracker.sang_meprise_sent_to` persists across client reconnect; evidence capture | PROMPT 1472 |

---

### LANE 6 — Bot / Autoplay Epic (Sprint 19 Pipeline)

`production/epics/bot-and-autoplay/` has 5 Sprint 19 candidate stories
from PROMPT 1608. Status after PROMPT 1831:

| Story | Status | Next Action |
|---|---|---|
| `BOT-ROOM-PARTICIPANT-001` | Implemented; story-done deferred to Sprint 19 | story-done paperwork |
| `BOT-SOAK-ENTRYPOINT-001` | Worker PROMPT 1603 shipped; integration PROMPT 1607 status unconfirmed | Verify main-land |
| `AUTOPLAY-RECIPE-LIBRARY-001` | Bootstrap on origin/main; extension pending | Design + implement recipe library API |
| `AUTOPLAY-VS-BOT-QA-001` | Gated on 001+002+003 | Wait |
| `BOT-DEBUG-OVERLAY-001` | Data contract PROMPT 1604 done; impl gated on 1602/1603 | Ungate after 1607 verify |

| Prompt Title | Scope | Dependencies |
|---|---|---|
| `PROMPT-1853-BOT-SOAK-1607-MAIN-LAND-VERIFY` | Verify PROMPT 1607 integration is on main; confirm `BOT-SOAK-ENTRYPOINT-001` shippable | None (read-only verify) |
| `PROMPT-1854-AUTOPLAY-RECIPE-LIBRARY-DESIGN` | Design recipe library API: recipe format, execution engine, result schema | None (design doc) |
| `PROMPT-1855-BOT-DEBUG-OVERLAY-IMPL` | Implement bot debug overlay using PROMPT 1604 data contract | 1853 PASS |

---

### LANE 7 — Krosmaga UI / Art Wave

PROMPT 1280 (Krosmaga-style UI wave) is pending story-file authoring and main-land.
GDD `design/gdd/krosmaga-cards-reference.md` and `design/gdd/board-rendering.md`
are present. The art bible / visual direction has not landed in recent commits.

| Prompt Title | Scope | Dependencies |
|---|---|---|
| `PROMPT-1856-KROSMAGA-UI-ART-STORY-AUTHORING` | Author Sprint 19 story files for Krosmaga UI wave (card art polish, board visual pass, HUD chromework) | None (story authoring only) |
| `PROMPT-1857-CARD-SPRITE-ATLAS-AUDIT` | Audit `assets/` for placeholder vs shipped art; list gaps vs card pool | None (read-only) |
| `PROMPT-1858-HUD-TIMER-EYEBALL-VISUAL-IMPL` | Implement S11-HUD-TIMER-EYEBALL-VISUAL-001 (human-operator carry from Sprint 11→18); screenshot evidence | Human visual sign-off (S11 row) |

**File conflict risk for 1858:** `client/src/ui/` HUD systems — do not overlap with
Lane 2B (play-area container).

---

### LANE 8 — Tooling / Launcher

| Prompt Title | Scope | Dependencies |
|---|---|---|
| `PROMPT-1859-DEV-LAUNCHER-HEALTH-AUDIT` | Read-only audit of `tools/dev-launcher-app/`; list stale paths, missing env vars, race conditions | None |
| `PROMPT-1860-TWO-CLIENT-RUNTIME-SMOKE` | Run `tools/two-client-runtime` locally; capture output; verify no regressions from PROMPT 1471 repair chain | None (tool only) |
| `PROMPT-1861-CARGO-TOML-DEPENDENCY-AUDIT` | Read-only: verify all `Cargo.toml` versions (bevy 0.18, lightyear 0.26, bevy_tweening 0.18) are consistent; flag drift | None (read-only) |

---

### LANE 9 — Sprint / Paperwork / Release Blockers

| Prompt Title | Scope | Dependencies |
|---|---|---|
| `PROMPT-1862-SPRINT-18-COMPLETION-STATUS-REFRESH` | Read-only: tally all 12 rows, update completion count, surface Must Have gaps for sprint closure | None (read-only) |
| `PROMPT-1863-SPRINT-19-PLANNING-DRAFT` | Draft Sprint 19 row set: bot epic stories + Krosmaga UI wave + gameplay loop gaps + release prep | Sprint 18 completion status |
| `PROMPT-1864-POLISH-RELEASE-GATE-RETRY-PRECONDITIONS` | Audit PROMPT 761 gate-check failure findings; list what must be true before retry | None (read-only) |

---

## Parallelizability Matrix

```
LANE 1 (Bot/Autoplay evidence)     ─────────────┐  safe to parallelize with all lanes
LANE 3 paperwork (1841,1842,1843)  ─────────────┤  safe now; no source edits
LANE 5 read-only (1850)            ─────────────┤  safe now; read-only
LANE 6 read-only (1853,1854)       ─────────────┤  safe now; design/verify only
LANE 7 story authoring (1856,1857) ─────────────┤  safe now; docs/design only
LANE 8 tooling (1859,1860,1861)    ─────────────┤  safe now; tools/ not src/
LANE 9 read-only (1862,1864)       ─────────────┘  safe now; read-only

              ↓ PROMPT 1472 POST-REPAIR-LIVE-TWO-CLIENT-QA-RETEST ↓

LANE 2A auction-won-card (1838)    ───── gated (shop_auction file conflict)
LANE 2B play-area-container (1840) ───── gated (board_rendering conflict)
LANE 3 impl (1844 card-art-strip)  ───── gated (board_rendering conflict)
LANE 4 gameplay repairs (1846+)    ───── gated (server/src/core/ overlap)
LANE 5 repairs (1851,1852)         ───── gated (qa_snapshot.rs conflict)
```

**Immediate parallel batch (safe right now, no PROMPT 1472 dependency):**
1. PROMPT 1836 (vs-bot live evidence — after 1831)
2. PROMPT 1841 (viewport-invariant story-done paperwork)
3. PROMPT 1842 (snapshot-layout-fields story-done paperwork)
4. PROMPT 1843 (settings-panel verify)
5. PROMPT 1845 (round-state-machine audit — read-only)
6. PROMPT 1850 (QA snapshot coverage audit — read-only)
7. PROMPT 1853 (bot-soak 1607 main-land verify)
8. PROMPT 1857 (card sprite atlas audit)
9. PROMPT 1859 (dev-launcher health audit)
10. PROMPT 1861 (Cargo.toml dependency audit)
11. PROMPT 1862 (sprint-18 completion status refresh)
12. PROMPT 1864 (Polish→Release gate retry preconditions)

---

## File Ownership / Conflict Risk Map

| File / Directory | Risk Level | Active Workers Touching It |
|---|---|---|
| `client/src/ui/shop_auction/` | HIGH | PROMPT 1838 (gated) |
| `client/src/presentation/board_rendering.rs` | HIGH | PROMPT 1840, 1844 (gated) |
| `client/src/presentation/qa_snapshot.rs` | HIGH | PROMPT 1851 (gated) |
| `client/src/ui/hand/` | MEDIUM | PROMPT 1844 indirect |
| `server/src/feature/auction/` | MEDIUM | PROMPT 1838 (gated) |
| `server/src/core/` | MEDIUM | PROMPT 1846, 1849 (gated) |
| `shared/src/protocol.rs` | MEDIUM | PROMPT 1838 (gated) |
| `client/src/autoplay.rs` | LOW | PROMPT 1836/1837 (output only) |
| `tools/autoplay/` | LOW | PROMPT 1836/1837 |
| `production/sprints/` | LOW | PROMPT 1841–1843 (paperwork) |
| `assets/` | LOW | PROMPT 1857 (read-only) |
| `design/gdd/` | NONE | PROMPT 1856 (authoring) |
| `reports/` | NONE | Any worker |

---

## Release Gap Summary

For M2 (Playable Game) milestone gate: two players must connect, play a full 1v1
game through auction → placement → combat → shop → win condition. Evidence of this
is absent from recent commits (all work has been autoplay infrastructure). The
minimum release unblocking path is:

1. PROMPT 1472 POST-REPAIR-LIVE-TWO-CLIENT-QA-RETEST (overdue — must happen first)
2. S18-AUCTION-WON-CARD-DISPOSITION-001 integration (PROMPT 1838)
3. Combat resolution live verify (PROMPT 1847)
4. Win-condition verify (not yet mapped — needs a dedicated prompt)
5. Sprint 18 Must Have closure (all 4 rows)
6. Polish→Release gate-check retry (PROMPT 761 failure addressed)

---

1835: GAME-COMPLETION-NEXT-WAVE-MAP-AFTER-1830: SHIPPED
