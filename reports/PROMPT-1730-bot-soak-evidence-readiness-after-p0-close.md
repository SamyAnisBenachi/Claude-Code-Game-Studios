# PROMPT 1730 — BOT-SOAK-EVIDENCE-READINESS-AFTER-P0-CLOSE

**Date**: 2026-05-28  
**Operator**: Claude Sonnet 4.6  
**Source-of-truth audited**: `origin/main` @ `cbf4479d` (fix(bot): PROMPT 1721 — populate last_decision_at_ms on every action-loop decision)  
**Scope**: Read-only. Report written from `reports/1730-bot-soak-readiness` worktree based on `origin/main@cbf4479d`.

---

## 0. Purpose

Reassess `BOT-SOAK-ENTRYPOINT-001` story readiness after the P0 evidence gap fix wave
(POMPTs 1719–1722) and PROMPT 1728 verification. Determine: is a fresh bounded soak
now the correct next action, or are more implementation gaps blocking it?

---

## 1. P0/P1 Gap Classification

Evidence sources: PROMPT 1684 audit, reports 1719–1728, `git log origin/main --oneline`.

### P0 Gaps

| Gap | Description | Fix PROMPT | Status | SHA on origin/main |
|-----|-------------|------------|--------|-------------------|
| P0-A | Placement coordinates absent from bot decision log | 1719 | **CLOSED** | `a0719f5c` — `feat(bot): PROMPT 1719 — add placement coord logging to BotDecisionKind` |
| P0-B | No winner / game_over_reason in `final_state.json` | 1720 | **NEEDS_VERIFY** | NOT on `origin/main` — only on `origin/feat/1720-bot-final-state-winner` @ `9362275d` |
| P0-C | `BotState.last_decision_at_ms` frozen at Lobby | 1721 | **CLOSED** | `cbf4479d` — `fix(bot): PROMPT 1721 — populate last_decision_at_ms on every action-loop decision` |

**P0-A detail**: PROMPT 1726 successfully rebased the 1719 cherry-pick and merged FF into `origin/main@3a4f7721`. The `BotDecisionKind::PlacementSubmitted` variant now carries `coords: Vec<PlacementCoord>` (lane, cell, card_id, mana). Verified passing in PROMPT 1728: `bot_placement_wave_3_test` passes.

**P0-B detail**: PROMPT 1720 shipped `game_over_outcome`, `game_over_reason`, `game_over_round`, `our_player_id`, `game_over_loser_id` to `FinalState` in `tools/two-client-runtime/src/bot_soak.rs`. Commit `9362275d` is on `origin/feat/1720-bot-final-state-winner`. The integration refresh (equivalent of PROMPT 1724-1726 pattern) has NOT been run for this branch. **This fix is NOT on `origin/main`.**

**P0-C detail**: PROMPT 1727 cherry-picked the 1721 fix onto `origin/main@ac55edfc` and pushed `integrate/bot-last-decision-at-ms-1727`. It was confirmed FF-mergeable. PROMPT 1728 verified 110/110 bot tests pass including the three targeted `last_decision_at_ms` tests.

### P1 Gaps

| Gap | Description | Fix PROMPT | Status | Notes |
|-----|-------------|------------|--------|-------|
| P1-A | `legal_action_count` null for all draft/shop decisions | 1722 | **NEEDS_VERIFY** | Only on `origin/feat/1722-bot-draft-legal-action-count` @ `302b4a79` — NOT on `origin/main` |
| P1-B | No per-unit detail in board snapshots (card_id, hp, atk) | — | **STILL_OPEN** | No fix report found |
| P1-C | Disconnect trackers saturated (always 29996) | — | **STILL_OPEN** | No fix report found |
| P1-D | Trigger submits 0 placements — not flagged by harness | — | **STILL_OPEN** | Note: partially resolved operationally by PROMPT 1692 non-empty trigger fix; structured warning still absent |
| P1-E | Shop offering card IDs not recorded | — | **STILL_OPEN** | No fix report found |
| P1-F | `bot-soak-trigger/server.log` empty (copy-before-flush) | — | **STILL_OPEN** | No fix report found |

**P1-A detail**: PROMPT 1722 instrumented `bot_draft_auto_pick` with full `BotDecisionEntry` emission (Purchased, PurchaseSkipped variants with reason strings and `legal_action_count = Some(displayed.len())`). Validated: `cargo check -p server` clean, 12/12 action_loop unit tests pass. Commit `302b4a79` is on feature branch only.

### P2 Gaps

All P2 gaps from the 1684 audit (P2-A through P2-G) remain **STILL_OPEN** — no fix reports exist. Per the 1684 audit these are "nice-to-have" debuggability improvements and are not blockers for story-done.

---

## 2. Integration Status Summary

| Fix | Branch | On origin/main? |
|-----|--------|-----------------|
| P0-A placement coords (1719) | merged via 1726 | ✅ YES (`a0719f5c`) |
| P0-B winner/reason (1720) | `feat/1720-bot-final-state-winner` | ❌ NO — needs integration refresh |
| P0-C last_decision_at_ms (1721) | merged via 1727 | ✅ YES (`cbf4479d`) |
| P1-A legal_action_count (1722) | `feat/1722-bot-draft-legal-action-count` | ❌ NO — needs integration refresh |

---

## 3. Sprint Gate Status

Per PROMPT 1711 (Wave Map), gate C-4 for `BOT-SOAK-ENTRYPOINT-001` story-done requires:

1. **C-2**: Sprint 19 activated (requires C-1: Sprint 18 close-out)
2. **All P0 evidence gaps closed** on `origin/main`
3. **One fresh bounded soak** with enhanced evidence

Current gate status:

| Gate | Status | Blocker |
|------|--------|---------|
| Sprint 18 paperwork complete | PARTIAL — stories 1712–1715 Done; 1716–1718 unknown | Verify sprint-status.yaml |
| Sprint 18 close-out (C-1) | UNKNOWN — depends on paperwork + human-blocked rows | Not confirmed |
| Sprint 19 activated (C-2) | ❌ NOT ACTIVATED | Requires C-1 |
| P0-A on main | ✅ DONE | — |
| P0-B on main | ❌ MISSING | Integration refresh needed |
| P0-C on main | ✅ DONE | — |
| Fresh bounded soak with enhanced evidence | ❌ NOT RUN | Requires all P0 on main first |

The last verified soak (PROMPT 1710) ran on `origin/main@7f9c605e` — **before** any of the P0 fixes landed. That soak is insufficient as story-done evidence even though it passed, because the P0 deficiencies identified by PROMPT 1684 were still present at that SHA.

---

## 4. Is a Fresh Bounded Soak the Next Required Action?

**No.** Two concrete blockers remain before a soak run is meaningful:

### Blocker 1 — P0-B not on origin/main

`final_state.json` still lacks `game_over_outcome`, `game_over_reason`, `game_over_round`,
`our_player_id`, and `game_over_loser_id` on `origin/main`. A soak run today would produce
evidence that still cannot answer "who won and why" from structured files, failing the very
criterion the 1684 audit identified as reconstruction-blocking.

**Required action**: Integration refresh for PROMPT 1720 — rebasing
`feat/1720-bot-final-state-winner` onto current `origin/main@cbf4479d` and FF-merging.

### Blocker 2 — P1-A not on origin/main (strong advisory)

Draft-phase `legal_action_count` null is a P1 gap. While not as critical as P0-B, running
a soak while this remains unmerged means the resulting evidence still has the diagnostic
dark spot. Since the fix is already shipped on a feature branch, merging it before the soak
costs one integration refresh prompt — far cheaper than re-running soak evidence after.

**Recommended action**: Integration refresh for PROMPT 1722 — rebasing
`feat/1722-bot-draft-legal-action-count` onto `origin/main` and FF-merging.

---

## 5. Next-Action Plan

### Step 1 — Land P0-B (MANDATORY, run immediately)

🔻🔻🔻 PROMPT 1731 : BOT-P0-B-FINAL-STATE-WINNER-INTEGRATION-REFRESH

```
Context:
- PROMPT 1720 shipped winner/reason fields to final_state.json.
- Commit 9362275d is on origin/feat/1720-bot-final-state-winner.
- Current origin/main: cbf4479d.
- Prior integration pattern: see reports/PROMPT-1726-bot-p0-placement-coords-log-mainland-refresh2.md
  and reports/PROMPT-1727-bot-p0-last-decision-at-ms-integration-refresh.md.

Task: Integration refresh for PROMPT 1720.
- git fetch origin
- Create new branch from origin/main@cbf4479d: integrate/bot-final-state-winner-1731
- Cherry-pick 9362275d (PROMPT 1720 commit from feat/1720-bot-final-state-winner)
- Resolve any conflicts (expected: none — touches only tools/two-client-runtime/src/)
- Validate: git diff --check PASS; path allowlist (only tools/two-client-runtime/src/ files)
- Validate: cargo check -p two-client-runtime → clean
- Validate: cargo test -p two-client-runtime → all pass
- Confirm merge-base: origin/main is ancestor of branch tip
- Push branch to origin
- Write report to reports/PROMPT-1731-bot-p0-final-state-winner-integration-refresh.md
- Commit report + push

Report must state: READY_FOR_MAINLAND_ENQUEUE or blocker reason.
Final line: 1731: BOT-P0-B-FINAL-STATE-WINNER-INTEGRATION-REFRESH: <STATUS>
```

### Step 2 — Land P1-A (recommended, run in parallel with Step 1)

🔻🔻🔻 PROMPT 1732 : BOT-P1-A-LEGAL-ACTION-COUNT-INTEGRATION-REFRESH

```
Context:
- PROMPT 1722 shipped legal_action_count for draft/shop purchase evidence.
- Commit 302b4a79 is on origin/feat/1722-bot-draft-legal-action-count.
- Current origin/main: cbf4479d.
- Files changed by 1722: server/src/feature/bot/action_loop.rs only.

Task: Integration refresh for PROMPT 1722.
- git fetch origin
- Create new branch from origin/main@cbf4479d: integrate/bot-legal-action-count-1732
- Cherry-pick 302b4a79
- Validate: git diff --check PASS; path allowlist (only server/src/feature/bot/action_loop.rs)
- Validate: cargo check -p server → clean
- Validate: cargo test -p server --lib "feature::bot::action_loop::tests" → 12+/12 pass
- Confirm merge-base: origin/main is ancestor of branch tip
- Push branch to origin
- Write report to reports/PROMPT-1732-bot-p1-legal-action-count-integration-refresh.md
- Commit report + push

Report must state: READY_FOR_MAINLAND_ENQUEUE or blocker reason.
Final line: 1732: BOT-P1-A-LEGAL-ACTION-COUNT-INTEGRATION-REFRESH: <STATUS>
```

**Parallel safety**: 1731 and 1732 touch different crates (`tools/two-client-runtime/` vs `server/src/feature/bot/`) — fully safe to run in parallel.

### Step 3 — Mainland enqueue (after Steps 1 + 2 both READY)

After both integration refresh reports confirm `READY_FOR_MAINLAND_ENQUEUE`, merge both branches to `origin/main` FF-only in sequence (one at a time; each advances main tip).

### Step 4 — Fresh bounded soak (after Step 3 lands + Sprint 19 active)

🔻🔻🔻 PROMPT 1733 : BOT-SOAK-ENTRYPOINT-BOUNDED-SOAK-EVIDENCE

```
Context:
- By this point: all P0 gaps (P0-A, P0-B, P0-C) and P1-A are on origin/main.
- Sprint 19 must be active (gate C-2 from PROMPT 1711 Wave Map).
- Last verified soak: PROMPT 1710 on origin/main@7f9c605e (pre-P0 fixes).
- Evidence dir pattern: production/qa/evidence/dev-runs/YYYY-MM-DD-HHMMSS-bot-vs-bot-soak/

Task: Run a fresh bounded soak (--max-rounds 5 or 10) from latest origin/main
      after all P0 fixes have landed.
- git pull --ff-only origin main
- Rebuild server + bot-soak-trigger from the P0-fixed main
- Run: Start-BotVsBotSoak.ps1 -MaxRounds 10 (or equivalent Cargo invocation)
- Collect evidence: final_state.json (verify game_over_outcome + game_over_reason populated),
  bot-decision-log.jsonl (verify placement coords present, legal_action_count non-null for
  draft phases), server-snapshots/ (verify last_decision_at_ms advances past Lobby)
- Gate criteria (all must pass):
  * final_state.json::endpoint_reached == "game_over"
  * final_state.json::game_over_outcome != null
  * final_state.json::game_over_reason != null
  * bot-decision-log.jsonl placement entries contain coords (lane, card_id)
  * bot-decision-log.jsonl draft entries have legal_action_count != null
  * Snapshot bots[].last_decision_at_ms advances across rounds (not frozen)
  * exit_code == 0; server.err empty; WARN count == 0
- Write evidence to production/qa/evidence/dev-runs/
- Write report to reports/PROMPT-1733-bot-soak-entrypoint-bounded-soak-evidence.md

Gate note: this prompt is BLOCKED until Sprint 19 is activated (C-2).
Final line: 1733: BOT-SOAK-ENTRYPOINT-BOUNDED-SOAK-EVIDENCE: <STATUS>
```

### Step 5 — Story-done (after Step 4 passes + Sprint 19 active)

🔻🔻🔻 PROMPT 1734 : BOT-SOAK-ENTRYPOINT-001-STORY-DONE (after C-2 + Step 4 PASS)

```
/story-done BOT-SOAK-ENTRYPOINT-001
Evidence: PROMPT 1728 (110/110 bot tests), PROMPT 1710 (live soak realism verify),
          PROMPT 1733 (fresh bounded soak with enhanced evidence — P0 fixed),
          PROMPT 1731/1732 (P0-B and P1-A on origin/main).
```

---

## 6. Remaining Open Gaps (Non-blocking for story-done)

The following 1684 audit gaps are NOT required for `BOT-SOAK-ENTRYPOINT-001` story-done
but should be tracked for future soak hardening stories:

| Gap | Description | Sprint |
|-----|-------------|--------|
| P1-B | Per-unit board snapshot detail | Sprint 20+ |
| P1-C | Disconnect tracker Option<u32> | Sprint 20+ |
| P1-D | Per-player placement count warning in harness | Sprint 20+ |
| P1-E | Shop offering card IDs in snapshots | Sprint 20+ |
| P1-F | server.log copy-before-flush fix | Sprint 20+ |
| P2-A through P2-G | Debuggability improvements | Deferred |

---

## 7. Confidence Assessment

| Question | Answer | Confidence |
|----------|--------|------------|
| Are P0-A and P0-C on origin/main? | YES — `a0719f5c`, `cbf4479d` | HIGH (git log confirmed) |
| Is P0-B on origin/main? | NO — on feature branch only | HIGH (git branch --contains confirmed) |
| Is P1-A on origin/main? | NO — on feature branch only | HIGH (git branch --contains confirmed) |
| Does a fresh soak need to be run? | YES, but not yet | HIGH |
| Is the story-done gate for BOT-SOAK-ENTRYPOINT-001 clear? | NO — 2 integration refreshes + soak run needed | HIGH |

---

## 8. Summary

**Two implementation PROMPTs (1731 + 1732) must land before a soak is meaningful.**
P0-B (winner/reason) and P1-A (legal_action_count draft) are fully implemented on feature
branches but have not been integration-refreshed onto `origin/main`. The soak gate also
requires Sprint 19 to be activated (C-2 chain from PROMPT 1711). Once those three
prerequisites are met (1731 + 1732 merged, Sprint 19 active), PROMPT 1733 runs the
definitive evidence soak, and PROMPT 1734 completes the story-done.

---

1730: BOT-SOAK-EVIDENCE-READINESS-AFTER-P0-CLOSE: SHIPPED
