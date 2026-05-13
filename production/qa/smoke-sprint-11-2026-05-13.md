# Smoke Check Report: Sprint 11 (Polish / friend-game scope)

**Date**: 2026-05-13
**Sprint**: Sprint 11 (Polish stage — active per PROMPT 773)
**Engine**: Bevy 0.18 + Lightyear 0.26
**QA Plan**: `production/qa/qa-plan-sprint-11.md` (landed by PROMPT 774, 2026-05-13)
**Prompt**: PROMPT 790 — Sprint 11 Smoke Check
**HEAD at smoke entry**: `18758b25df209fa03cf9c0ba5237c7577ef33f8e` (`story-done(s11): close ignored D-5 triage`, integration commit landed by PROMPT 789, 2026-05-13).
**HEAD == origin/main**: yes (`origin/main` was at the same SHA after `git fetch origin`).
**Branch / working tree**: `main`; working copy carries one unstaged modification at `.claude/settings.json` (preserved untouched as required by the operating contract). No staged changes. No new untracked files.
**Smoke environment**: local root checkout (no worktree) on Windows 11 / D: drive, ~222 GB free at smoke entry — sufficient for full workspace test execution.
**Scope**: Sprint 11 Polish / friend-game smoke check only.

---

## Verdict: PASS-WITH-WARNINGS

All Sprint 11 Must Have rows are `done` on `origin/main` at `18758b2`. The full workspace test suite passes with zero failures. The 5 `#[ignore]`-tagged D-5 tests documented in `production/qa/evidence/sprint-11-ignored-d5-triage.md` (Cluster B — retained) remain ignored, as expected; no test that is *not* in that documented set is failing or skipped.

Per `/smoke-check` skill verdict rules: **PASS** if the automated test suite ran cleanly; **PASS-WITH-WARNINGS** when known, owner-named ignored tests with a landed triage disposition remain unresolved. This run matches the latter condition.

This report makes **no claim** of (preserved non-claims):

- no public release readiness
- no release-candidate readiness
- no full game completion
- no broad / Standard-tier accessibility completion (`QA-COND-0005` unchanged)
- no playtest / fun-hypothesis validation (`QA-COND-0006` unchanged)
- no full playable-client manual QA (`S8-QA-001-W1` unchanged)
- no final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved)
- no Sprint 11 close-out

PROMPT 790 did **not** run `/gate-check`, did **not** run `/team-qa`, did **not** run `/release-check`, did **not** issue any QA sign-off, did **not** close Sprint 11.

---

## Preflight

| Step | Result |
|---|---|
| `git fetch origin` | OK |
| `git rev-parse HEAD` | `18758b25df209fa03cf9c0ba5237c7577ef33f8e` |
| `git rev-parse origin/main` | `18758b25df209fa03cf9c0ba5237c7577ef33f8e` (matches HEAD) |
| `git status --short` | ` M .claude/settings.json` (preserved untouched; pre-existing modification) |
| D: free space | ~222 GB free / 1.3 TB total (`df -h /d`) — sufficient for Cargo workspace |

---

## Commands and Results

### `cargo fmt --check`

```
cargo fmt --check
```

**Result**: PASS (exit 0, no output — no formatting drift on the workspace).

### `cargo check --workspace`

```
cargo check --workspace
```

**Result**: PASS — `Finished \`dev\` profile [optimized + debuginfo] target(s) in 1m 15s`. Zero compilation errors. No new warnings beyond those already present on the integrated tip.

### `cargo test --workspace --tests --no-fail-fast`

```
cargo test --workspace --tests --no-fail-fast
```

**Result**: PASS-WITH-WARNINGS.

Aggregated totals (summed across all binaries reporting a `test result:` line — 189 binaries):

| Metric | Count |
|---|---|
| passed | **1129** |
| failed | **0** |
| ignored | **5** |
| measured | 0 |
| filtered out | 0 |

The 1129 / 0 / 5 totals match the post-integration counts cited by PROMPT 779 worker-side workspace verification and reaffirm the workspace pass-count delta `+6` (1123 → 1129) that landed via `S11-TD-FIXTURE-HAND-UI-ONENTER-001` at integration commit `d7f4103` (PROMPT 784, 2026-05-13).

### `git diff --check`

Output (warning only, not a whitespace error):

```
warning: in the working copy of '.claude/settings.json', LF will be replaced by CRLF the next time Git touches it
```

The Git CRLF advisory is informational and is **not** a `git diff --check` whitespace error. `.claude/settings.json` remains untouched by PROMPT 790 (pre-existing in-tree modification preserved as required by the operating contract). No actual whitespace defects reported on any other path.

### `git diff --cached --check`

Output: (empty — no staged changes at smoke entry, clean.)

---

## Ignored tests (documented Cluster B — 5 retained D-5)

Cross-reference: `production/qa/evidence/sprint-11-ignored-d5-triage.md` (PROMPT 787 author / PROMPT 788 integration at `1d96281` / PROMPT 789 `/story-done` paperwork at `18758b2`).

The triage evidence enumerates **6 resolved (Cluster A) + 5 retained (Cluster B) = 11** original D-5 ignored tests surfaced by Sprint 10 smoke retry-7 W1. Cluster A was un-`#[ignore]`d by `S11-TD-FIXTURE-HAND-UI-ONENTER-001` (PROMPT 779 worker / PROMPT 784 integration / PROMPT 785 `/story-done`). Cluster B remains `#[ignore]` on `origin/main@18758b2` with owner-named dispositions:

| # | Test (binary :: name) | Owner-named disposition | Triage row |
|---|---|---|---|
| B1 | `tests/integration/board_rendering/ghost_preview_bridge_test.rs :: br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui` | board `GhostDragStartEvent` producer system not present in `BoardRenderingPlugin`-only fixture; needs HandUiPlugin pointer-to-drag bridge or fixture expansion | Cluster B1 (`S11-TD-FIXTURE-D-RESIDUALS-001` umbrella OR split `S11-TD-FIXTURE-BOARD-GHOST-DRAG-PRODUCER-001`) |
| B2 | `tests/integration/board_rendering/snapshot_spawn_test.rs :: test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives` | assertion expects HudPlugin to bridge `snapshot.phase -> CurrentClientPhase`, but HudPlugin is not in this fixture; either expand fixture or relocate assertion to a HUD test (needs owner decision) | Cluster B2 (`S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001`) |
| B3 | `tests/integration/playable_client/native_operator_controls_test.rs :: test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands` | `ConfirmClass` intent not emitted alongside `SelectClass` — input chain stops at `SelectClass`; needs lobby input system investigation (revealed after D-1 fix) | Cluster B3 (`S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001`) |
| B4 | `tests/unit/board_rendering/status_icons_test.rs :: test_cooccupancy_index_two_panics_with_offending_index` | production `co_occupancy_offset` no longer panics on index 2 — needs design decision: restore panic guard or update test to assert non-panic behavior | Cluster B4 (`S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001`) |
| B5 | `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs :: shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes` | `ShopAuctionUiEntity` count drift — actual=66, formula expects=57 (9 entity delta); needs scaffold owner to either update formula or trim spawn | Cluster B5 (`S11-TD-FIXTURE-D-RESIDUALS-001` umbrella OR split `S11-TD-SHOP-AUCTION-UI-COUNT-DRIFT-001`) |

Each row carries `still ignored` state on `origin/main@18758b2` with the original PROMPT 750 D-5 owner-named comment unchanged. The triage record explicitly does **not** authorise immediate implementation for any B-row; each follow-up slug requires its own story file + `/story-readiness` in a separate prompt before `/dev-story` can begin.

**Total `#[ignore]`d in workspace**: **5** (Cluster B only) — matches the cargo aggregate above.

---

## Failures and Blockers

None. No failing test. No build error. No `git diff --check` whitespace defect. No disk-space blocker (D: ~222 GB free). No tooling block. No `cargo fmt --check` drift.

---

## Sprint 11 disposition (preserved)

- **Sprint 11**: `active` (Polish-stage; activated by PROMPT 773). PROMPT 790 did **not** edit `production/sprints/sprint-11.md`, did **not** edit `production/stage.txt`, did **not** modify `.claude/settings.json`, did **not** modify `production/sprint-status.yaml`.
- **Stage**: `Polish`. PROMPT 761 Polish→Release gate-check `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`. No retry attempted by PROMPT 790.
- **Sprint 10**: `closed-with-conditions` per PROMPT 763, recorded under `sprint_10_closeout:` in `production/sprint-status.yaml`. Unchanged.
- **All Sprint 11 Must Have rows**: `done` (6/6) at `origin/main@18758b2` per PROMPTs 780 / 781 / 783 / 785 / 786 / 789. Sprint 11 **not** closed by this prompt.

---

## Files changed by PROMPT 790

- `production/qa/smoke-sprint-11-2026-05-13.md` (this file — NEW)
- `production/session-state/active.md` (banner prepended)
- `production/session-state/codex-orchestrator-state.md` (operating-rules banner refreshed; PROMPT 790 disposition section prepended)
- `reports/PROMPT-790.md` (mandatory final report; NOT staged or committed)

Explicitly **not** touched:

- `.claude/settings.json` (the dirty in-tree modification is preserved as-is)
- `client/`, `server/`, `shared/`, `tests/`
- `production/sprint-status.yaml`
- `production/stage.txt`
- `production/sprints/sprint-11.md`
- `production/qa/evidence/sprint-11-ignored-d5-triage.md`
- any `reports/` file other than `reports/PROMPT-790.md`

---

## Verification commands (for re-run)

```bash
git fetch origin
git rev-parse HEAD                                       # expect 18758b25df209fa03cf9c0ba5237c7577ef33f8e
git rev-parse origin/main                                # expect 18758b25df209fa03cf9c0ba5237c7577ef33f8e
df -h /d                                                 # expect >5 GB free
cargo fmt --check                                        # expect exit 0, no output
cargo check --workspace                                  # expect Finished dev profile
cargo test --workspace --tests --no-fail-fast            # expect 1129 passed / 0 failed / 5 ignored
git diff --check                                         # expect CRLF advisory on .claude/settings.json only
git diff --cached --check                                # expect empty
```

---

## Cross-references

- Triage evidence (Cluster B owner-named dispositions): `production/qa/evidence/sprint-11-ignored-d5-triage.md`
- Hand UI OnEnter fixture repair (Cluster A resolution): `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`
- Sprint 11 plan: `production/sprints/sprint-11.md`
- Sprint 11 QA plan: `production/qa/qa-plan-sprint-11.md`
- Polish→Release gate FAIL (preserved): `production/gate-checks/gate-polish-release-2026-05-12.md`
- Sprint status: `production/sprint-status.yaml`
- Stage: `production/stage.txt` (`Polish`)

---

**End of report — PROMPT 790 / Sprint 11 smoke verdict: PASS-WITH-WARNINGS**
