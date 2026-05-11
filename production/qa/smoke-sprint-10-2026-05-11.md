# Smoke Check Report: Sprint 10 Close-Out

**Date**: 2026-05-11
**Sprint**: Sprint 10 (Polish — close-out kickoff)
**Engine**: Bevy 0.18 + Lightyear 0.26
**QA Plan**: NOT FOUND for Sprint 10 (`production/qa/qa-plan-sprint-10-*.md` absent; most recent QA plan is sprint-8 dated 2026-05-07).
**Argument**: `sprint`
**Commit Under Smoke**: `217428a01f581dfe8745b1707988b470434cb26b` (Finding D fix); main has since advanced to `7382a82` (PROMPT 675 /team-qa report — APPROVED WITH CONDITIONS) and `172c2d7` (PROMPT 676 /gate-check — FAIL). This report is scoped to the Sprint 10 delivered-feature tip at `217428a` per PROMPT 674.
**Smoke Environment**: CI authoritative per project memory `project_tech_stack.md` (local builds gated on Developer PowerShell for VS 2026 — not active in this session). CI workflow: `.github/workflows/tests.yml`.

---

## Verdict: FAIL

Sprint 10 smoke check **FAILS** Phase 2 (Automated Tests) on `origin/main` at `217428a`.

`Run Cargo Tests` CI job on `217428a` completed with conclusion `failure`. The failing test is a Sprint 10 source-invariant guard that triggers from a doc comment introduced during Sprint 10 close-out. Subsequent test stages (server tests, shared tests, RSM single-writer invariant, shared purity, E2E WebSocket) were **SKIPPED** by `set -euo pipefail` after the guard failed, so no positive CI signal exists for the Sprint 10 critical-path automated tests.

Per `/smoke-check` skill verdict rules: **FAIL if automated test suite ran and reported one or more test failures**. The verdict is therefore FAIL regardless of qualitative manual smoke signal.

This report makes **no claim** of: public release readiness, full playable-client manual QA, full game completion, broad accessibility completion, playtest fun-hypothesis validation, or Sprint 10 close-out.

Per PROMPT 674: this report STOPS at FAIL — `/team-qa` already ran independently (PROMPT 675, `7382a82`) and does not re-fire from here.

---

## Environment

- Root branch: `main`, working tree clean (only `.claude/scheduled_tasks.lock` untracked).
- Test directory: found at `tests/`.
- CI configured: yes (`.github/workflows/tests.yml`).
- Smoke test source: `tests/smoke/critical-paths.md` (no Sprint 10 QA plan).
- Engine: Bevy 0.18 + Lightyear 0.26 per `.claude/docs/technical-preferences.md` and `docs/engine-reference/bevy/VERSION.md`.
- Local build prerequisite (Developer PowerShell for VS 2026) NOT active in this session — local cargo execution not attempted; CI is authoritative.
- HEAD at smoke entry: `217428a` (`fix(session): close c2s_confirm_class silent-discard + lobby premature-confirm guard (PROMPT 670)`).
- Recent CI history on `main`: **last green = `1d683bb` at 2026-05-07 22:54 UTC**. Every CI run since the S10-POLISH-001 integration commit `b780f0e` (2026-05-10 19:01 UTC) has reported `failure`. Aggregate over last 200 runs on main: 162 failure / 37 success / 1 in-progress.

---

## Phase 2 — Automated Tests

**Status**: **FAIL** (CI `Run Cargo Tests` job on `217428a`, run id `25665438619`).

### Failing test

- **`board_rendering_does_not_register_phase_receiver`** in [tests/unit/board_rendering/plugin_scaffold_test.rs:150-172](tests/unit/board_rendering/plugin_scaffold_test.rs#L150-L172) — panicked at line 160.
  - Test asserts: the only `client/src/**/*.rs` file containing the substring `MessageReceiver<S2CPhaseChanged>` must be `client/src/presentation/mod.rs`.
  - Actual matches found:
    1. [client/src/presentation/mod.rs:150](client/src/presentation/mod.rs#L150) — real ECS query (expected).
    2. [client/src/ui/hud/mod.rs:1187](client/src/ui/hud/mod.rs#L1187) — **false positive in a `///` doc comment** that documents that the HUD does *not* drain that channel. The comment text contains the literal string `MessageReceiver<S2CPhaseChanged>`.
  - Root cause of false positive: [tests/unit/board_rendering/plugin_scaffold_test.rs:199-204](tests/unit/board_rendering/plugin_scaffold_test.rs#L199-L204) — `normalize_source()` strips whitespace only; it does **not** strip Rust comments before substring matching.
  - Failure introduced at: `b780f0e` (S10-POLISH-001 integration, 2026-05-10) — the doc comment was added as part of HUD chrome documentation work, not as a code change to HUD behavior.
  - Functional impact: **zero on game behavior**. HUD does not actually drain `MessageReceiver<S2CPhaseChanged>`; the comment correctly documents that invariant. This is a test-tooling false positive blocking CI.

### Stages SKIPPED by `set -euo pipefail`

The following CI stages did not run because the source-guard step bailed first:

- Check shared crate
- E2E WebSocket round-trip
- Run server tests
- Run shared tests
- Check RSM single-writer invariant
- Check `shared/` purity

Net consequence: **no positive CI signal exists for the Sprint 10 delivered feature tests** (HUD dim overlay, SAU chrome wiring, lobby chrome wiring, reward loop, lobby class-confirm Finding D fix, hand-fan Finding B v2 V3 fix). Their test files exist on disk and were independently exercised at story-time per `/story-done` evidence, but CI has not re-validated them on the integrated tip.

### Other CI jobs (informational)

- `shared/ dep purity` — success
- `client/ dep purity` — success
- `server/ dep purity` — success
- `WASM bundle size check` — skipped (cascade)

---

## Phase 3 — Test Coverage (Sprint 10 critical paths per PROMPT 674)

Story list drawn from `production/sprint-status.yaml` Sprint 10 + PROMPT 674 critical-path scope.

| Story / Critical Path | Story Type | Test File (on disk) | Coverage Status |
|---|---|---|---|
| S10-PAW-001 PAW-002..006 close-out batch | Integration | `tests/integration/presentation/{hand_ui,shop_auction,hud,board,lobby}_asset_wiring_test.rs` | COVERED (CI-unverified post-217428a) |
| S10-TD-001 test-fixture cascade-fail repair | Integration | `tests/integration/hand-ui/draft_initial_grid_test.rs`, `tests/integration/shop_auction_ui/{shop_panel_test.rs,auction_activation_test.rs}` | COVERED (CI-unverified post-217428a) |
| S10-TD-002 plugin-registration audit | Config/Data | n/a | EXPECTED |
| S10-CARRY-001 Sprint 9 carry-over consolidation | Config/Data | n/a | EXPECTED |
| S10-POLISH-001 HUD visual chrome MVP | Integration | `tests/integration/hud/hud_resolution_dim_test.rs` | COVERED (CI-unverified post-217428a; story evidence 8/8 PASS pre-merge) |
| S10-POLISH-002 SAU panel chrome wiring | Integration | `tests/integration/shop_auction_ui/chrome_wiring_test.rs` | COVERED (CI-unverified post-217428a; story evidence 4/4 PASS pre-merge) |
| S10-POLISH-003 Lobby visual chrome MVP | Integration | `tests/integration/session/lobby_chrome_wiring_test.rs` | COVERED (CI-unverified post-217428a; story evidence 5/5 PASS pre-merge) |
| ECO-004 Kill & Objective awards reward loop | Logic / Integration | `tests/integration/economy/reward_loop_awards_test.rs` | COVERED (CI-unverified post-217428a) |
| Lobby room creation + class confirm (Finding D fix, PROMPT 670) | Integration | `tests/integration/session/room_create_join_test.rs`, `tests/integration/playable_client/lobby_entry_*test.rs` | COVERED (CI-unverified post-217428a) |
| DRAFT_INITIAL card purchase (Finding B v2 V3 fix, PROMPT 671) | Integration | `tests/integration/card_acquisition/draft_initial_test.rs`, `tests/integration/hand-ui/draft_initial_grid_test.rs`, `tests/integration/hand-ui/hand_ui_slot_onscreen_test.rs`, `tests/integration/hand-ui/hand_ui_viewport_sync_test.rs` | COVERED (CI-unverified post-217428a) |

**Summary**: 8 COVERED (CI-unverified), 2 EXPECTED, 0 MISSING, 0 MANUAL.

All Sprint 10 Logic/Integration stories have test files on disk. The advisory gap is that CI never reached them on the integrated tip because of the Phase 2 source-guard failure.

### Pre-existing carried findings (NOT Sprint 10 regressions; surfaced for transparency)

Per `production/session-state/active.md` (2026-05-10 PROMPT 630 closure):

1. `hud_asset_wiring_test` — 0/6 (HUD epic asset-wiring sub-area; S10-TD-001 closure tail).
2. `hud_plugin_scaffold_test` — 3/4 (PAW-004 timer-bar `Name` break).
3. Broken `*_harness.rs` bins (harness/test-infra cross-epic; Bevy 0.18 Input feature reorg).

These were known and accepted at Sprint 10 mid-stream; not new regressions from PROMPT 670 / 671 fixes or from `b780f0e` HUD chrome integration.

---

## Phase 4 — Manual Smoke Checks

Manual smoke batches (Batch 1 core stability, Batch 2 sprint mechanic + regression, Batch 3 data integrity + perf) were **not run** because the Phase 2 FAIL signal is determinate under the skill's first-matching-rule verdict logic. The user can run manual smoke independently if desired; per PROMPT 674's stop-on-FAIL clause, no `/team-qa` invocation follows this report.

---

## Missing Test Evidence

All Sprint 10 Logic and Integration stories have test files. No MISSING entries.

---

## Verdict: FAIL

### Required remediation before re-running `/smoke-check`

One of the two options below must land on `main` (both are zero-risk to game behavior):

**Option A (smallest surface — recommended)** — Reword the offending doc comment so the substring `MessageReceiver<S2CPhaseChanged>` no longer appears in [client/src/ui/hud/mod.rs:1187](client/src/ui/hud/mod.rs#L1187). Replace with semantic-equivalent prose like *"never reads the phase-changed message channel"*.

**Option B (test-tool fix — slightly larger)** — Tighten [tests/unit/board_rendering/plugin_scaffold_test.rs:199-204](tests/unit/board_rendering/plugin_scaffold_test.rs#L199-L204) `normalize_source()` to strip line- and block-comments before substring matching. Requires a follow-up test of the helper.

Either fix is a candidate Sprint 10 close-out blocker repair (estimate ≤ 0.25 d) or, alternatively, a Sprint 11 CI-debt-001 story.

### Carry state preserved by this report

- Sprint 9 closed-with-conditions; **Sprint 10 close-out NOT claimed**.
- S8-QA-001-W1 manual/browser two-client GAME_OVER gap remains open.
- QA-COND-0005 (Standard-tier accessibility) accepted-risk friend-game scope.
- QA-COND-0006 (playtest fun-hypothesis validation) accepted-risk / deferred.
- No public release readiness, release-candidate readiness, broad accessibility completion, full game completion, playtest validation, or full playable-client manual QA claimed.

### Skill-compliant gate handoff

> The smoke check failed. Do not hand off to QA until these failures are resolved:
> - `board_rendering_does_not_register_phase_receiver` — CI source-invariant guard false-positive on HUD doc comment (root cause and remediation above).
>
> Fix per Option A or Option B and re-run `/smoke-check sprint` before any subsequent `/team-qa` re-run or `/gate-check` re-attempt. Note: `/team-qa` already ran independently as PROMPT 675 (`7382a82`) with APPROVED WITH CONDITIONS; PROMPT 676 `/gate-check` returned FAIL specifically on the missing smoke artifact this file now provides — gate-check must be re-run after smoke FAIL is resolved.

---

## Cross-references

- CI run id (current HEAD `217428a`): `25665438619` — `failure`.
- CI run id (parent `d9ee107`): `25665302009` — `failure` (same root cause).
- Last green CI on `main`: `1d683bb` at 2026-05-07 22:54 UTC.
- Parallel reports landed during this run:
  - `172c2d7` qa(gate-check): Polish→Release Sprint 10 close-out gate — FAIL (PROMPT 676)
  - `7382a82` docs(qa): team-qa Sprint 10 close-out report (PROMPT 675) — APPROVED WITH CONDITIONS
- Smoke skill invoked: `.claude/skills/smoke-check/SKILL.md`.

---

## Retry post-686 (PROMPT 687)

**Date**: 2026-05-11 (UTC)
**Invoked at**: HEAD `b378512` (local) / `0501d88` (origin/main); CI source-invariant guard fix landed at `3a283c9` (PROMPT 686).
**Argument**: `sprint`
**Trigger**: PROMPT 687 — re-run `/smoke-check sprint` to verify the source-guard false-positive is cleared.

### Verdict: FAIL (different root cause; source-guard cascade is cleared)

The PROMPT 686 doc-comment reword **successfully** unblocks the CI source-invariant guard. All three CI runs spanning the fix now pass `Check Board Rendering source guards` and proceed through the stages that were SKIPPED in the PROMPT 674 cascade. **A new, contained failure** then surfaces in the server test stage that was masked by the previous cascade.

### CI runs verified

| Run ID | Commit | Conclusion | Source-guard step | New failure |
|---|---|---|---|---|
| `25669402727` | `3a283c9` (doc reword) | failure | PASS ✅ | `duplicate_confirm_with_same_class_is_silent_idempotent_noop` |
| `25669486325` | `b378512` (HEAD local) | failure | PASS ✅ | same |
| `25669539042` | `0501d88` (origin/main) | failure | PASS ✅ | same |

All three runs report exactly one failing test out of ≥24 server test groups (97 individual server tests pass; 1 fails). The previously-blocking source-guard check is green on every run.

### New failure (now exposed by source-guard unblock)

- **`duplicate_confirm_with_same_class_is_silent_idempotent_noop`** in [tests/unit/session/class_reveal_test.rs:175](tests/unit/session/class_reveal_test.rs#L175) — panicked.
  - Test asserts: after `confirm_class(...)` is called twice with the same player + same class, the second call returns `ConfirmClassOutcome::Ignored`.
  - Actual outcome on current `main`: `ConfirmClassOutcome::Locked { ... }` (via `class_lock_ack(...)`).
  - Root cause: the Finding D fix at `217428a` (PROMPT 670, `fix(session): close c2s_confirm_class silent-discard + lobby premature-confirm guard`) intentionally changed `confirm_class` in [server/src/core/session/system.rs:1152-1158](server/src/core/session/system.rs#L1152-L1158) so that a duplicate same-class confirm now returns an explicit lock-ack instead of silently discarding. The corresponding test, authored at `c93ff35` (S3-03, Sprint 3), was not updated to the new Finding D semantics.
  - Why this only surfaces now: at PROMPT 674 the source-guard step bailed before server tests ran; the stale assertion was never exercised by CI. The source-guard reword at `3a283c9` removed the cascade, so server tests run and this assertion fails deterministically.
  - Functional impact: **zero on game runtime behavior** — the Finding D code path is correct and intended. The failure is an out-of-date test expectation, not a game regression.

### CI stages that now run (previously SKIPPED)

All previously-skipped stages on `b378512` (HEAD) ran:

- ✅ Check shared crate
- ✅ E2E WebSocket round-trip
- ❌ Run server tests (1 stale-assertion failure — root cause above)
- — Run shared tests (SKIPPED by `set -euo pipefail` after server-test FAIL)
- — Check RSM single-writer invariant (SKIPPED)
- — Check shared/ purity (SKIPPED)
- — WASM bundle size check (cascade SKIPPED)

### Phase 3 — Test Coverage delta vs PROMPT 674

No change. All Sprint 10 Logic / Integration stories from the original PROMPT 674 table still have test files on disk and remain COVERED (CI-verified for non-session stages on this retry; the failing test is in the lobby/class-reveal area, not in the Sprint 10 chrome stories).

### Phase 4 — Manual Smoke Checks

Not run. Phase 2 returned a deterministic CI failure under the skill's first-matching-rule verdict logic. Per PROMPT 687: "If FAIL → STOP and surface."

### Required remediation before re-running `/smoke-check`

One of two zero-game-impact options must land on `main`:

**Option A (smallest surface — recommended)** — Update [tests/unit/session/class_reveal_test.rs:175](tests/unit/session/class_reveal_test.rs#L175) to match the Finding D semantics. Replace the assertion `assert!(matches!(duplicate, ConfirmClassOutcome::Ignored))` with the appropriate `ConfirmClassOutcome::Locked { .. }` shape (and rename the test to reflect the new "duplicate confirm acks instead of silently discarding" semantics). Estimate ≤ 0.25 d.

**Option B (revert subset of Finding D)** — Re-introduce a silent-discard branch for the duplicate-same-class case inside `confirm_class`. NOT recommended — it would re-introduce the silent-discard behavior that PROMPT 670 explicitly closed.

Option A is the correct remediation: the production semantics are intentional, the test is stale.

### Carry state (unchanged)

- Sprint 9 closed-with-conditions; **Sprint 10 close-out NOT claimed** (now blocked on a different root cause than at PROMPT 674).
- S8-QA-001-W1 manual/browser two-client GAME_OVER gap remains open.
- QA-COND-0005 (Standard-tier accessibility) accepted-risk friend-game scope.
- QA-COND-0006 (playtest fun-hypothesis validation) accepted-risk / deferred.
- No public release readiness, release-candidate readiness, broad accessibility completion, full game completion, playtest validation, or full playable-client manual QA claimed.

### Skill-compliant gate handoff

> The smoke check failed (different root cause vs PROMPT 674). Do not hand off to QA until this failure is resolved:
> - `duplicate_confirm_with_same_class_is_silent_idempotent_noop` — stale assertion against Finding D (217428a) intended `confirm_class` duplicate-handling semantics; the production code path is correct.
>
> Fix per Option A and re-run `/smoke-check sprint` before any subsequent `/team-qa` re-run or `/gate-check` re-attempt.

### Cross-references

- Previous attempt: PROMPT 674 FAIL section above — source-guard false positive at HUD doc comment (root cause), fixed at `3a283c9` (PROMPT 686).
- CI guard fix commit: `3a283c9` `fix(hud): reword doc comment to satisfy CI source-invariant guard (PROMPT 686)`.
- Finding D fix commit: `217428a` `fix(session): close c2s_confirm_class silent-discard + lobby premature-confirm guard (PROMPT 670 / Finding D real root cause)`.
- CI runs verified: `25669402727`, `25669486325`, `25669539042` — all completed `failure` 2026-05-11 UTC.
