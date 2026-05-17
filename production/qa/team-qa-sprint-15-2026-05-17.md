# Team-QA Report: Sprint 15 (Polish / friend-game scope)

| Field | Value |
|---|---|
| **Date** | 2026-05-17 |
| **Sprint** | Sprint 15 -- `active` (Polish stage; activated by PROMPT 997 on 2026-05-17) |
| **Stage** | `Polish` (unchanged; `production/stage.txt` = `Polish`) |
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Scope** | UI clean-pass closeout (Tier 1 Should adjacent + Tier 3 doc-only spec + Tier 0 interaction primitives) + Sprint 13 -> 14 -> 15 human-operator-blocked HUD timer eyeball carry + Sprint 13 evidence-vs-status paperwork row-flip. Friend-game / Polish slice only. Explicitly NOT public release readiness. |
| **Skill** | `/team-qa sprint` (qa-lead + producer roles; serialized shared-status writer per 2026-05-13 orchestrator override; no spawned agents -- paperwork-only single-context review of record). |
| **Prompt** | PROMPT 1015 -- Sprint 15 Team QA |
| **Worktree** | `D:/Tmp/ccgs-prompt-1015-team-qa` (fresh detached worktree off `origin/main`). Root checkout NOT used. |
| **Branch** | `qa/sprint-15-team-qa-1015` (NEW; tracks `origin/main`) |
| **Commit Under Review (origin/main HEAD)** | `f3e635d657589ce41b7b1e9667207e0830bfedb0` (`story-done(s15): close S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP (PROMPT 1010)`) |
| **HEAD == origin/main** | yes (verified after `git fetch origin`) |
| **Review mode** | Lean (no `production/review-mode.txt` override) |
| **Cargo policy applied** | **N/A** -- no `cargo` command was invoked by PROMPT 1015 (paperwork-only review-of-record on PROMPT 1009 / 1010 integrated story-done evidence + PROMPT 1012 smoke evidence + PROMPT 1011 evidence-slot reservation + PROMPT 998 / 1014 audio-repair forensic). PROMPT 1012 smoke applied the binding Windows/MSVC cargo resource policy at its Cargo invocations. |

---

## Verdict: APPROVED-WITH-CONDITIONS

Sprint 15 stands at **4 of 5 rows closed** on `origin/main@f3e635d`:

- **Must Have: 1 / 2 done.** `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP` closed paperwork-only by PROMPT 1010 (Sprint 13 evidence-vs-status gap discharged via PROMPT 891 evidence cite). The single open Must Have row is `S11-HUD-TIMER-EYEBALL-VISUAL-001` (story 014), the Sprint 13 -> 14 -> 15 human-operator-blocked cosmetic visual check carry; closure remains gated on a real two-client run with screenshot capture across `DraftInitial` 45s / `DraftShop` 30s / `Placement` 10-12s phases. No LLM `/story-done` is authorised. PROMPT 1011 authored a Sprint 15 evidence-slot reservation + runbook on a worker branch (`prompt-1011-hud-timer-human-capture-prep@b4c1b79`) -- **NOT yet merged to `origin/main`**.
- **Should Have: 2 / 2 done** (`S12-UX-HAND-DRAG-STATE-VISUALS-001` story 020 + `S11-UX-BOARD-RENDERING-SPEC` story 013), both closed by PROMPT 1009 with full per-row AC PASS.
- **Nice to Have: 1 / 1 done** (`S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` story 008), closed by PROMPT 1009 with full per-row AC PASS.

The Sprint 15 smoke check (PROMPT 1012, on worker branch `qa/sprint-15-smoke-check-1012@bd7691c`) is **`PASS-WITH-WARNINGS`** against the same `origin/main@f3e635d` this Team-QA reviews. Cargo aggregate: **216 binaries / 1384 passed / 0 failed / 0 ignored / 0 measured / 0 filtered out**; functional total **1389 passed / 0 failed / 0 ignored across 217 effective binaries** (cargo aggregate + AppCompat workaround direct-run). Smoke warning is environment/tool-only (Windows AppCompat false positive) plus a carry-warning that the PROMPT 998 placement-timer audio crash repair is NOT reachable from `origin/main`. The smoke evidence file `production/qa/smoke-sprint-15-2026-05-17.md` is on the smoke worker branch only -- **NOT yet integrated to `origin/main`** (same precedent as Sprint 14 PROMPT 983 smoke rerun, which was later integrated by PROMPT 986).

A new external-pending condition surfaced after Sprint 15 activation: **PROMPT 1014 (2026-05-17) confirmed that the placement-timer audio `UnrecognizedFormat` crash is a live regression on `origin/main@f3e635d`** -- `client/src/audio/mod.rs` still spawns `AudioPlayer` for an Ogg Opus placeholder asset that Bevy 0.18 cannot decode with the current `client/Cargo.toml` feature set (`bevy_audio` without `vorbis` / `symphonia-*`). The PROMPT 996 disarm fix + PROMPT 998 integration (`c508d9d`) exist on `origin/integrate/placement-timer-audio-crash-repair-998` but were never merged to `main`. This is a **known pending-integration regression** (NOT a Sprint 15 row, NOT in Sprint 15 scope), surfaced here as a carry-warning so it is not silently absorbed.

The conditions on this approval are exactly the carry conditions from the Sprint 15 plan + QA plan + the post-activation pending-integration items, preserved verbatim (none closed by this report):

1. **`S11-HUD-TIMER-EYEBALL-VISUAL-001`** -- carried OPEN; human-operator-blocked manual visual check; Sprint 13 -> 14 -> 15 carry; promoted Should -> Must in Sprint 15 plan. Closure remains gated on real two-client screenshot capture. PROMPT 1011 evidence runbook authored off-main (`b4c1b79`); producer schedules.
2. **PROMPT 998 placement-timer audio crash repair pending integration** -- code-only disarm fix at `c508d9d` on `origin/integrate/placement-timer-audio-crash-repair-998` (3 files / +102 / -13); `cargo test -p client --test timer_urgency_audio_crash_guard_test` 1 / 1 PASS at integration tip; push to `main` blocked at user-policy layer per PROMPT 998 redo report. NOT on `origin/main`. PROMPT 1014 confirms the live regression remains on `main`. Recommend a separate hot-fix re-land prompt (PROMPT 1014 Section 8 recommendation) before any release-tier consideration.
3. **PROMPT 1011 HUD timer evidence-slot reservation pending integration** -- runbook + command summary at `production/qa/evidence/sprint-15-hud-timer-visual-check/README.md` + `command-summary.md` (2 files / +454 / 0) authored on worker branch only. Integration is a separate orchestrator prompt; does NOT block this Team-QA, but is a prerequisite for the human-operator capture session.
4. **PROMPT 1012 smoke evidence pending integration** -- `production/qa/smoke-sprint-15-2026-05-17.md` (438 lines) on worker branch `qa/sprint-15-smoke-check-1012@bd7691c` only. Same precedent as Sprint 14 PROMPT 983 smoke rerun later integrated by PROMPT 986. Recommend integration before Sprint 15 close-out.
5. **`S8-QA-001-W1`** -- manual / browser two-client GAME_OVER gap remains **OPEN**. No Sprint 15 row touched this surface; Sprint 13 story 017 AC12 forbid-auto-closure carries through Sprint 14 (PROMPT 987) and Sprint 15 (PROMPT 997) unchanged.
6. **`QA-COND-0005`** -- Standard-tier accessibility remains **accepted-risk** (friend-game scope only). Sprint 15 hand-drag-state visuals, board rendering spec, and interaction-state primitives are friend-game visual polish only. The L5 `LOBBY_BUTTON_HEIGHT = 30.0` defect remains accepted-risk under `QA-COND-0005`.
7. **`QA-COND-0006`** -- playtest / fun-hypothesis validation remains **accepted-risk / deferred**. Sprint 15 UI clean-pass closeout does NOT advance playtest validation.
8. **`PAW-TD-*-a`** -- placeholder-art accept-risk preserved across PAW-002..PAW-006. Sprint 15 layout / composition / primitive work does NOT advance placeholder-art resolution.
9. **`TQ-S12-C1..C7`** -- preserved verbatim. **TQ-S12-C2 binding**: no third same-scope retest of Sprint 12 story 019 is authorised by Sprint 15. **TQ-S12-C7** explicitly NOT closed.
10. **PROMPT 683-era runtime divergence question** -- preserved as folded into Sprint 12 story 019 `closed-with-conditions / cannot-reproduce`.
11. **PROMPT 761 `Polish->Release` gate-check `FAIL`** -- preserved at `production/gate-checks/gate-polish-release-2026-05-12.md` (verdict line 14 verified `**FAIL**`); **NO retry** attempted by any Sprint 15 row and **NO retry** attempted by this Team-QA.
12. **Sprint 12 story 019 underlying drag-runtime bug** -- NOT claimed fixed. Sprint 15 hand-drag-state visuals work (`S12-UX-HAND-DRAG-STATE-VISUALS-001`) is layout / visual state work over already-extant client-side drag ephemeral state per ADR-012 binding; no new server-authoritative state, no protocol-shape change.
13. **Sprint 10 / 11 / 12 / 13 / 14 closeouts** -- preserved unchanged. Sprint 14 disposition `closed-with-conditions` per PROMPT 987; all 16 closed Sprint 14 `/story-done` closures preserved on `origin/main`.

This Team-QA report makes **no claim** of (preserved non-claims):

- no public release readiness
- no release-candidate (RC) readiness
- no full game completion
- no broad / Standard-tier accessibility completion (`QA-COND-0005` unchanged)
- no playtest / fun-hypothesis validation (`QA-COND-0006` unchanged)
- no full playable-client manual QA (`S8-QA-001-W1` unchanged)
- no two-client GAME_OVER closure
- no final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved)
- no `Polish->Release` retry -- PROMPT 761 FAIL preserved
- no Sprint 15 close-out (this Team-QA sign-off is a precondition to a separate close-out decision, not the close-out itself)
- no `S8-QA-001-W1` closure
- no `S11-HUD-TIMER-EYEBALL-VISUAL-001` closure
- no closure of PROMPT 996 / PROMPT 998 audio crash repair (still off-main)
- no `TQ-S12-C7` closure
- no stage advance from `Polish`
- no underlying drag-runtime bug fix (Sprint 12 story 019 `cannot-reproduce` preserved)
- no full UI clean-pass repair beyond the 5 Sprint 15 candidate rows (Tier 3 rank 13 multi-surface refactor + Tier 2 cosmetic captures bundle deferred to Sprint 16+)

PROMPT 1015 did **not** run `/dev-story`, `/smoke-check`, `/gate-check`, `/release-check`, `/story-done`, `/story-readiness`, or `/qa-plan`. PROMPT 1015 did **not** modify production code under `client/`, `server/`, `shared/`, or `tests/`. PROMPT 1015 did **not** modify `production/sprint-status.yaml` (no row/status flips), `production/sprints/sprint-15.md`, `production/qa/qa-plan-sprint-15.md`, `production/stage.txt`, `production/gate-checks/*`, any Sprint 15 story file, any prior-sprint evidence file, `.claude/settings.json`, `.octogent/`, `.claude/scheduled_tasks.lock`, or `reports/` (other than the mandatory `reports/PROMPT-1015-*.md` final-report file, which is gitignored and NOT staged or committed). Session-state files (`production/session-state/active.md` + `production/session-state/codex-orchestrator-state.md`) were NOT touched; the local Team-QA precedent (PROMPT 984 Sprint 14) does not require a banner.

---

## Verification (Preflight)

| Step | Result |
|---|---|
| `git fetch origin` | OK |
| `git worktree add D:/Tmp/ccgs-prompt-1015-team-qa -b qa/sprint-15-team-qa-1015 origin/main` | OK -- detached worktree on `origin/main@f3e635d`; branch tracks `origin/main` |
| `git rev-parse HEAD` (this branch) | `f3e635d657589ce41b7b1e9667207e0830bfedb0` |
| `git rev-parse origin/main` | `f3e635d657589ce41b7b1e9667207e0830bfedb0` (matches HEAD) |
| `git status --short` (this branch) | clean -- no modifications, no untracked files (fresh worktree) |
| `production/stage.txt` | `Polish` (unchanged) |
| `production/sprint-status.yaml` top level | `sprint: 15`, `status: active`, `stage: Polish` (verified lines 4 / 21 / 23) |
| PROMPT 761 `Polish->Release` gate-check FAIL evidence | preserved at `production/gate-checks/gate-polish-release-2026-05-12.md` (`Verdict: **FAIL**`, line 14) |
| Sprint 15 plan | exists at `production/sprints/sprint-15.md` (PROMPT 988 DRAFT body + PROMPT 997 ACTIVATED banner) |
| Sprint 15 QA plan | exists at `production/qa/qa-plan-sprint-15.md` (PROMPT 1002) |
| Sprint 15 smoke evidence (PROMPT 1012) | exists at `production/qa/smoke-sprint-15-2026-05-17.md` on `origin/qa/sprint-15-smoke-check-1012@bd7691c` -- **NOT yet integrated to `origin/main`** (`git merge-base --is-ancestor bd7691c origin/main` exits 1) |
| PROMPT 1011 HUD timer evidence runbook | exists at `production/qa/evidence/sprint-15-hud-timer-visual-check/README.md` (+ `command-summary.md`) on `origin/prompt-1011-hud-timer-human-capture-prep@b4c1b79` -- **NOT yet integrated to `origin/main`** (`git merge-base --is-ancestor b4c1b79 origin/main` exits 1) |
| PROMPT 998 audio crash repair integration | exists at `c508d9d` on `origin/integrate/placement-timer-audio-crash-repair-998` (parent `299696d` PROMPT 996 worker) -- **NOT yet integrated to `origin/main`** (`git merge-base --is-ancestor c508d9d origin/main` exits 1) |
| Workspace ignored-test count (PROMPT 1012 smoke) | **0** (Sprint 12 retirement under PROMPT 814 preserved through Sprint 15) |

---

## Sprint 15 Row Closure (4 / 5 done)

Verified by reading `production/sprint-status.yaml` `stories:` block at HEAD `f3e635d`. Each row below carries its current `status:`, the `/story-done` chain, integration commits on `origin/main`, and acceptance evidence references.

### Must Have (1 / 2 done)

| ID | Title | Status | Closed | Integration commit | `/story-done` |
|---|---|---|---|---|---|
| S11-HUD-TIMER-EYEBALL-VISUAL-001 (hud story 014) | HUD Timer Eyeball Visual Check (Sprint 13 -> 14 -> 15 human-operator-blocked carry; promoted Should -> Must in Sprint 15) | **ready** (human-operator-blocked; **OPEN**) | -- | -- | -- |
| S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP | Paperwork-only row-status flip discharging the Sprint 13 evidence-vs-status gap (PROMPT 891 / `S13-CONN-LOST-UX-001` cited as closure evidence) | **done** | 2026-05-17 | (none -- paperwork-only; evidence chain `febc56a` / `cb01c49` / `fcdad9a` cited) | PROMPT 1010 (`f3e635d`) |

### Should Have (2 / 2 done)

| ID | Title | Status | Closed | Integration commit | `/story-done` |
|---|---|---|---|---|---|
| S12-UX-HAND-DRAG-STATE-VISUALS-001 (hand-ui story 020) | Hand Drag-State Visuals (Tier 1 Should adjacent; hand-ui surface; AC1-AC19 PASS) | **done** | 2026-05-17 | `88a6db1` (PROMPT 1008) | PROMPT 1009 (`3b6acec`) |
| S11-UX-BOARD-RENDERING-SPEC (board-rendering story 013) | Board Rendering Spec (Tier 3 rank 14; doc-only canonical spec at `docs/ux/board-rendering-spec.md` NEW 865 lines; AC1-AC16 PASS) | **done** | 2026-05-17 | `08f389b` (PROMPT 1006) | PROMPT 1009 (`3b6acec`) |

### Nice to Have (1 / 1 done)

| ID | Title | Status | Closed | Integration commit | `/story-done` |
|---|---|---|---|---|---|
| S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001 (ui-clean-pass story 008) | UI Interaction State Primitives (Tier 0 Should adjacent; new module `client/src/ui/design_tokens/interaction_states.rs` 445 lines; AC1-AC12 PASS; per-surface migration explicitly OUT OF SCOPE deferred to Sprint 16+) | **done** | 2026-05-17 | `5d36c4b` (PROMPT 1007) | PROMPT 1009 (`3b6acec`) |

**Closure**: 4 of 5 rows done. The single open row is `S11-HUD-TIMER-EYEBALL-VISUAL-001`, carried as Must Have human-operator-blocked per PROMPT 894 (Sprint 13 closeout) + PROMPT 987 (Sprint 14 closeout) + PROMPT 988 plan + PROMPT 997 activation. Closure requires real two-client browser/native screenshot capture across the three timed phases; no LLM `/story-done` is authorised. PROMPT 1011 evidence-slot reservation runbook + command summary authored off-main on `prompt-1011-hud-timer-human-capture-prep@b4c1b79` (2 files / +454). This is the same human-operator-blocked carry that has been in place since Sprint 10 retry-7 W2 -> Sprint 11 -> Sprint 12 -> Sprint 13 -> Sprint 14 -> Sprint 15.

---

## Smoke Status (PROMPT 1012)

**Verdict**: `PASS-WITH-WARNINGS`. Source: `production/qa/smoke-sprint-15-2026-05-17.md` on `origin/qa/sprint-15-smoke-check-1012@bd7691c` (NOT yet integrated to `origin/main`; same `origin/main@f3e635d` HEAD as this Team-QA reviews). Cargo policy: full binding Windows/MSVC env-var block applied (`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`). Disk: 826 GB free throughout (>> 40 GB threshold; no cleanup needed).

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS (exit 0, no drift) |
| `cargo check --workspace --all-targets` | PASS; one pre-existing dead-code warning preserved from PROMPT 983 baseline (`count_with_image_node` in `tests/integration/presentation/hand_ui_asset_wiring_test.rs`); NOT introduced by Sprint 15. |
| `cargo test --workspace --tests --no-fail-fast` | PASS-WITH-WARNINGS -- **216 binaries / 1384 passed / 0 failed / 0 ignored / 0 measured / 0 filtered out**; 1 binary spawn-blocked by Windows AppCompat (`spawn_range_live_update_contract` -- OS error 740 on `update` substring; identical classification to PROMPT 815 / 979 / 982 / 983). |
| AppCompat workaround (rename binary `srluc_appcompat_renamed_1012.exe`, 5 consecutive runs) | **5 / 5 runs PASS; aggregate 25 / 25** (identical to PROMPT 815 / 979 / 982 / 983 classification). |
| Story 020 targeted (in workspace run) | `hand_ui_drag_state_visuals_test` -- **11 / 11 PASS** (parity with PROMPT 1003 worker / PROMPT 1008 integration). |
| Story 008 targeted (in workspace run) | `ui_clean_pass_interaction_state_primitives_test` -- **8 / 8 PASS** (parity with PROMPT 1005 worker / PROMPT 1007 integration). |
| Sprint 14 baseline 1 (in workspace run) | `shop_auction_ui_plugin_scaffold_formulas_test` -- **8 / 8 PASS** (parity with PROMPT 982 / 983). |
| Sprint 14 baseline 2 (in workspace run) | `ui_clean_pass_z_layers_test` -- **6 / 6 PASS** (parity with PROMPT 982 / 983). |
| `git diff --check` / `git diff --cached --check` (smoke worktree) | PASS (exit 0, empty output) |

**Functional total** (cargo aggregate + AppCompat workaround direct-run): **1389 passed / 0 failed / 0 ignored across 217 effective binaries**. Net improvement vs Sprint 14 PROMPT 983 (1355 passed / 214 effective binaries): +34 passes, +3 effective binaries (story 020 + story 008 new test bins + auxiliary). No regression.

### Smoke Warning Classification

| Warning | Classification | Action by this Team-QA |
|---|---|---|
| `spawn_range_live_update_contract-*.exe` cargo-spawn blocked on Windows by AppCompat heuristic on the `update` substring in the path | **Environment / tool-only** (identical to PROMPT 815 / 979 / 982 / 983). Verified PASS via documented rename workaround (5 / 5 PASS). NOT a code regression. NOT a Sprint 15 row defect. | **Carried unchanged** -- same warning carried through Sprint 11 / 12 / 13 / 14 Team-QA approvals. No remediation work scheduled in Sprint 15. |
| PROMPT 998 placement-timer audio crash repair `c508d9d` is NOT reachable from `origin/main` at smoke time | **External pending integration** (worker branch only). Smoke does NOT claim the audio crash repair is landed. PROMPT 1014 (post-smoke) confirms the live regression persists on `origin/main`. | **Carried as Condition 2** above. Recommend separate hot-fix re-land prompt before close-out per PROMPT 1014 §8. |

---

## PROMPT 1014 Audio-Crash Diagnostic Integration

PROMPT 1014 (2026-05-17) is a paperwork-only diagnostic that materially affects the Sprint 15 risk picture and is integrated into this Team-QA as Condition 2 above. Key findings (re-verified by this Team-QA via `git merge-base --is-ancestor`):

- `c508d9d` (PROMPT 998 integrate) is NOT on `origin/main` -- VERIFIED at this Team-QA's smoke worktree base `f3e635d`.
- `299696d` (PROMPT 996 disarm fix worker) is NOT on `origin/main` -- VERIFIED.
- `origin/main:client/src/audio/mod.rs` still spawns `AudioPlayer` for an Ogg Opus asset that Bevy 0.18 cannot decode with current features.
- `origin/main:client/Cargo.toml` declares `bevy = { ..., default-features = false, features = ["2d", "webgl2", "bevy_audio"] }` -- no `vorbis` / `symphonia-*` decoder feature.
- Placeholder asset `assets/audio/ui/hand/sfx_timer_urgency_default.ogg` is 134-byte Ogg Opus (NOT Ogg Vorbis).
- PROMPT 1014 fix-comparison verdict: Option E (re-land PROMPT 996 disarm as hot-fix immediately; defer asset replacement + Vorbis feature flip to a separate design-led story). PROMPT 996 disarm guard is the only robust user-code defence on Bevy 0.18 for this exact `play_queued_audio_system<AudioSource>` panic mode.

This Team-QA does **not** apply the audio fix. It does **not** open a formal bug ticket. It records the regression as Condition 2 above and recommends the hot-fix re-land prompt (PROMPT 1014 §8) be sequenced before any Sprint 15 close-out paperwork.

---

## /story-done Closures (PROMPT 1009 + PROMPT 1010)

PROMPT 1009 (`3b6acec`) integrated `/story-done` batch closed three Sprint 15 implementation rows in one commit:

- **S11-UX-BOARD-RENDERING-SPEC** (story 013): AC1-AC16 PASS. Doc-only canonical board rendering spec at `docs/ux/board-rendering-spec.md` (NEW 865 lines) + 3 evidence files under `production/qa/evidence/sprint-15-board-rendering-spec/`. Worker PROMPT 1004 `477806a` / integration PROMPT 1006 `08f389b`. Folds `S11-UX-BOARD-STATUS-ICON-LEGEND-001` + `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` Tier 2 cosmetic-capture future candidates as spec sections.
- **S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001** (story 008): AC1-AC12 PASS. Primitive module `client/src/ui/design_tokens/interaction_states.rs` (NEW 445 lines exporting `HOVER_*` / `FOCUS_*` / `PRESSED_*` / `DISABLED_*` token-set families) + `docs/ux/global-ui-design-spec.md` amendments + 8-test integration bin `tests/integration/ui_clean_pass/interaction_state_primitives_test.rs`. Worker PROMPT 1005 `ea26e34` / integration PROMPT 1007 `5d36c4b`. Per-surface migration explicitly OUT OF SCOPE for Sprint 15 per AC10 -- deferred to Sprint 16+ family `S16-UI-INTERACTION-STATE-MIGRATION-*`.
- **S12-UX-HAND-DRAG-STATE-VISUALS-001** (story 020): AC1-AC19 PASS. New submodule `client/src/ui/hand/drag_state_visuals.rs` (NEW 368 lines) + `client/src/ui/hand/mod.rs` (+42 / -5) + `tests/integration/hand-ui/hand_ui_drag_state_visuals_test.rs` (NEW 726 lines / 11 ECS-query AC9 BLOCKING assertions) + `client/Cargo.toml` (+4 for test bin registration). Worker PROMPT 1003 `cce9a90` / integration PROMPT 1008 `88a6db1`. ADR-012 binding preserved (read-only over already-extant client-side ephemeral drag state); Sprint 12 story 019 underlying drag-runtime bug NOT claimed fixed (cannot-reproduce preserved per TQ-S12-C2).

PROMPT 1010 (`f3e635d`) `/story-done` closed the fourth row:

- **S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP**: paperwork-only Sprint 13 evidence-vs-status gap discharge. Evidence chain PROMPT 889 worker `febc56a` + PROMPT 890 integration `cb01c49` + PROMPT 891 `/story-done` `fcdad9a` (which cited the backlog row as closure-with-evidence rationale but explicitly deferred the row-status flip per AC8 design). Sprint 13 closeout (PROMPT 894) `conditions_carried_forward_unchanged` and Sprint 14 closeout (PROMPT 987) `explicitly_not_claimed` both recorded the deferral. PROMPT 1010 is the separate paperwork prompt that AC8 designed for. No code change, no test change, no production code touched.

Both `/story-done` prompts preserved `S11-HUD-TIMER-EYEBALL-VISUAL-001` `status: ready` verbatim (verified via diff inspection in both reports). Both prompts preserved all 16 closed Sprint 14 `/story-done` closures + Sprint 10 / 11 / 12 / 13 / 14 dispositions + all accept-risk rows verbatim.

---

## Pending Integrations (NOT closed by this report)

Three pieces of off-main work are pending integration to `origin/main`. None are blockers for Team-QA approval (the closed-rows ratification stands on the integrated `origin/main` evidence); each is a documented next-step for the orchestrator's close-out sequencing.

| Branch | Tip commit | Authored by | Scope | Recommended action |
|---|---|---|---|---|
| `origin/qa/sprint-15-smoke-check-1012` | `bd7691c` | PROMPT 1012 | Sprint 15 smoke evidence (`production/qa/smoke-sprint-15-2026-05-17.md` 438 lines) | Integration prompt mirroring Sprint 14 PROMPT 986 precedent. Recommended before close-out. |
| `origin/prompt-1011-hud-timer-human-capture-prep` | `b4c1b79` | PROMPT 1011 | HUD timer eyeball human-operator evidence-slot reservation + runbook (`production/qa/evidence/sprint-15-hud-timer-visual-check/README.md` + `command-summary.md`, 454 lines) | Integration prompt before the human-operator capture session begins. |
| `origin/integrate/placement-timer-audio-crash-repair-998` | `c508d9d` | PROMPT 998 (worker PROMPT 996) | Placement timer audio crash disarm (3 files / +102 / -13; `client/src/audio/mod.rs` + `client/Cargo.toml` + `tests/integration/playable_client/timer_urgency_audio_crash_guard_test.rs`); `cargo test -p client --test timer_urgency_audio_crash_guard_test` 1 / 1 PASS at integration tip | Separate hot-fix re-land prompt per PROMPT 1014 §8 Option E recommendation. Recommended before close-out so the live audio crash regression is no longer carried on `origin/main`. |

None of these are required for Team-QA APPROVED-WITH-CONDITIONS; all three are recommended close-out prerequisites and are recorded as Conditions 2-4 above.

---

## Forbidden-Path Sweep (PROMPT 1015 actions)

| Path | Touched by PROMPT 1015? |
|---|---|
| `client/` | NO |
| `server/` | NO |
| `shared/` | NO |
| `tests/` | NO |
| `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/` | NO |
| `production/sprint-status.yaml` | NO (no row/status flips) |
| `production/sprints/sprint-15.md` | NO |
| `production/qa/qa-plan-sprint-15.md` | NO |
| `production/stage.txt` | NO (remains `Polish`) |
| `production/gate-checks/*` | NO (PROMPT 761 FAIL preserved) |
| `production/session-state/active.md` | NO (PROMPT 984 Sprint 14 Team-QA precedent does not require a banner) |
| `production/session-state/codex-orchestrator-state.md` | NO (same precedent) |
| Any Sprint 15 story file (014 / 020 / 013 / 008) | NO |
| Any Sprint 13 / 14 evidence | NO |
| Any smoke / gate-check / release artifact | NO (only this Team-QA report authored) |
| `.claude/settings.json`, `.octogent/`, `.claude/scheduled_tasks.lock` | NO |
| PROMPT 998 / 996 audio repair branches | NO (no merge attempted, recorded as carry warning only) |
| PROMPT 1011 HUD timer prep branch | NO (no merge attempted, recorded as carry warning only) |
| PROMPT 1012 smoke branch | NO (no merge attempted, recorded as carry warning only) |

Files modified by PROMPT 1015:

- `production/qa/team-qa-sprint-15-2026-05-17.md` (NEW; this report)

---

## Recommended Next Prompt

Per PROMPT 1015 instructions: "closeout with conditions if Team-QA approves with only accepted carries; repair prompt if needs-work/blocker".

This Team-QA verdict is **APPROVED-WITH-CONDITIONS** with **no blocker for the four closed rows**, but with a live production-code regression on `origin/main` (PROMPT 1014 confirmed audio `UnrecognizedFormat` panic on the placement timer path) and three pending integrations (PROMPT 1012 smoke + PROMPT 1011 HUD timer prep + PROMPT 998 audio repair).

Recommended sequencing for the orchestrator:

1. **Hot-fix re-land prompt for PROMPT 996 audio disarm** -- new dedicated prompt per PROMPT 1014 §8: cherry-pick `299696d` to a `fix/audio-crash-reland-main-1015b` branch off `origin/main@f3e635d`, run `cargo test -p client --test timer_urgency_audio_crash_guard_test` (must PASS), run cargo aggregate (Sprint 15 baseline: 1389 passed / 0 failed), open PR, integrate prompt verifies `git merge-base --is-ancestor` BEFORE declaring success. Removes the only un-carried live-regression-on-main from the Sprint 15 risk picture.
2. **PROMPT 1012 smoke evidence integration** -- merge `qa/sprint-15-smoke-check-1012@bd7691c` to `origin/main`, mirroring Sprint 14 PROMPT 986 precedent.
3. **PROMPT 1011 HUD timer evidence-slot reservation integration** -- merge `prompt-1011-hud-timer-human-capture-prep@b4c1b79` to `origin/main`. Unblocks the human-operator capture session.
4. **Human-operator HUD timer eyeball capture session** -- producer schedules a real human-operator slot; operator follows the PROMPT 1011 runbook end-to-end; captures 3 mid-countdown screenshots; fills the `<<RUN RESULTS START>>...<<RUN RESULTS END>>` block; commits per Section K recipe. Parallel-safe with steps 1-3 by file scope (after step 3 lands).
5. **Paperwork `/story-done` on `S11-HUD-TIMER-EYEBALL-VISUAL-001`** -- only after step 4 evidence lands on `origin/main`. Flips row `status: ready -> done`. No LLM `/story-done` until real evidence exists.
6. **Sprint 15 close-out paperwork** -- `closed-with-conditions` per PROMPT 894 / PROMPT 987 paperwork-only precedent. Carries the same conditions enumerated above into Sprint 16 planning.

Steps 1-3 are parallel-safe by file scope (audio fix vs smoke evidence vs HUD prep are file-disjoint). Step 4 depends on step 3. Step 5 depends on step 4. Step 6 depends on steps 1-5 plus the producer's explicit close-out decision.

If the producer prefers to close Sprint 15 without the audio hot-fix re-land (treating it as a Sprint 15 -> Sprint 16 carry condition), this Team-QA's APPROVED-WITH-CONDITIONS verdict still holds -- the audio regression is enumerated as Condition 2 above and would carry forward verbatim to Sprint 16 planning. This is acceptable but the producer must document the cause in Sprint 15 close-out and surface the carry to the audio team for prioritisation.

---

## Final git status (PROMPT 1015 worktree)

```
On branch qa/sprint-15-team-qa-1015
Your branch is up to date with 'origin/main'.

(this file staged + committed)
```

Local `HEAD` after commit will fast-forward `f3e635d -> <new-commit>` with the single Team-QA file added. Push attempt to `origin/main` documented in the final PROMPT 1015 report; if push is blocked at policy layer, the worker branch remains ready for separate integration per the orchestrator's standard pattern.

---

`1015: SPRINT-15-TEAM-QA: APPROVED-WITH-CONDITIONS`
