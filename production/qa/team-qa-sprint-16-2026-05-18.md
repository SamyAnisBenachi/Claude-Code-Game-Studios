# Team-QA Report: Sprint 16 (Polish stage; deferred-backlog closeout sprint)

| Field | Value |
|---|---|
| **Date** | 2026-05-18 |
| **Sprint** | Sprint 16 -- `active` (Polish stage; activated by PROMPT 1064 on 2026-05-17) |
| **Stage** | `Polish` (unchanged) |
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Scope** | Friend-game / Polish slice -- deferred-backlog closeout sprint: last canonical UI clean-pass roadmap row (Tier 3 rank 13 card-slot primitive) + Windows AppCompat manifest ops hygiene + workspace dead-code warning cleanup + Sprint 13 -> 14 -> 15 -> 16 human-operator-blocked HUD timer eyeball carry. Explicitly NOT a release sprint. |
| **Skill** | `/team-qa sprint` (qa-lead + producer roles; serialized shared-status writer per 2026-05-13 orchestrator override; no spawned CCGS subagents -- paperwork-only single-context review of record). |
| **Prompt** | PROMPT 1078 -- Sprint 16 Team-QA |
| **Worktree** | `D:\_DEV\claude-code-game-studios-worktrees\sprint-16-team-qa-1078` |
| **Branch** | `qa/sprint-16-team-qa-1078` (NEW; tracks `origin/main`) |
| **Commit Under Review (HEAD)** | `f8eac30d98af1ad21ed3ca6dd06e219ce9f9df19` (`story-done(s16): close card-slot primitive row (PROMPT 1074)`) |
| **HEAD == origin/main** | yes (verified after `git fetch origin`) |
| **Smoke evidence tip** | `origin/qa/sprint-16-smoke-check-1075@56655fc8c20c1aad8485f2de41c656cbb7c96900` -- strict fast-forward descendant of `origin/main@f8eac30` (merge-base == `f8eac30`); smoke evidence file `production/qa/smoke-sprint-16-2026-05-18.md` lives on the smoke branch, **NOT yet integrated to `origin/main`** (documentation-completeness pending integration; see `Pending Integration` condition below). |
| **Review mode** | Lean (no `production/review-mode.txt` override) |
| **Cargo policy applied** | **N/A** -- no `cargo` command was invoked by PROMPT 1078 (paperwork-only, review-of-record on existing PROMPT 1067 / 1068 / 1069 worker outputs + PROMPT 1070 / 1071 / 1073 integrations + PROMPT 1072 / 1074 /story-done closures + PROMPT 1075 smoke evidence). PROMPT 1075 smoke applied the binding Windows / MSVC cargo resource policy with one documented `CARGO_TARGET_DIR` sibling-dir workaround for live-process contention (classified environmental / host-state-only; see `Smoke Warning Classification` below). |

---

## Verdict: APPROVED-WITH-CONDITIONS

Sprint 16 stands at **3 of 4 active rows closed** on `origin/main@f8eac30`:

- **Must Have: 0 / 1 done.** The single open row is the Sprint 13 -> 14 -> 15 -> 16 carry `S11-HUD-TIMER-EYEBALL-VISUAL-001` (story 014), which is **human-operator-blocked** per the original Sprint 13 close-out (PROMPT 894), the Sprint 14 / Sprint 15 carries, the Sprint 15 close-out (PROMPT 1056), and the 2026-05-17 orchestrator decision to defer human visual testing later. Closure requires a real two-client run with screenshot capture across `DraftInitial` 45s / `DraftShop` 30s / `Placement` 10-12s phases; **no LLM `/story-done` is authorised**. This is **NOT a regression** and **NOT a remediable blocker** for Sprint 16 close-out-with-conditions; the Sprint 16 activation banner (`production/sprints/sprint-16.md` lines 22-29) explicitly marks the row "MUST NOT block non-human Sprint 16 development lanes" and allows it to carry to Sprint 17 if no human-operator slot opens in the Sprint 16 timebox.
- **Should Have: 1 / 1 done.** `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` (story 009; Tier 3 rank 13; last canonical UI clean-pass roadmap row) closed by PROMPT 1074 paperwork-only `/story-done` on `origin/main@c9b5716` -> tip advanced to `f8eac30`. Closure basis: PROMPT 1067 worker (`3bdf6ac`, `feat(ui): add card-slot primitive and migrate shop slot phase 1`) + PROMPT 1073 integration (`d12adc4`, `integrate(s16): merge card-slot primitive worker into 1073`) + evidence dir `production/qa/evidence/sprint-16-ui-card-slot-primitive/`.
- **Nice to Have: 2 / 2 done.** `S15-OPS-APPCOMPAT-MANIFEST-001` (devops story 006) and `S15-TD-WORKSPACE-DEAD-CODE-WARNING-001` (ui-clean-pass story 016) both closed by PROMPT 1072 paperwork-only `/story-done` batch on `origin/main@bd374dd` -> tip advanced to `c9b5716`. Closure bases: PROMPT 1068 worker (`ed58e3d`) + PROMPT 1071 integration (`488a9cd`) for the AppCompat row; PROMPT 1069 worker (`2251a93`) + PROMPT 1070 integration (`bd374dd`) for the dead-code warning row.

The Sprint 16 smoke (PROMPT 1075) is **PASS-WITH-WARNINGS** on the source-of-truth HEAD this Team-QA reviews, with the single warning classified as **environment / host-state-only** (Windows live-process contention on the mandated Cargo target dir; documented sibling-dir workaround applied; no code regression -- see `Smoke Warning Classification` below). The full workspace cargo aggregate at smoke time:

- `cargo fmt --all -- --check`: PASS (exit 0, no output).
- `cargo check --workspace --all-targets`: PASS, **zero warnings, zero errors** (pre-existing Sprint 14 `count_with_image_node` dead-code warning was removed by PROMPT 1069 worker + PROMPT 1070 integration as part of `S15-TD-WORKSPACE-DEAD-CODE-WARNING-001` closure).
- `cargo test --workspace --tests --no-fail-fast`: PASS at **1464 passed / 0 failed / 0 ignored / 0 measured / 0 filtered across 223 binaries**. Net delta vs Sprint 14 PROMPT 983 rerun: **+109 passed tests, +9 effective binaries, -5 ignored (now 0), AppCompat OS-740 warning resolved by PROMPT 1072 closure of `S15-OPS-APPCOMPAT-MANIFEST-001`**.
- `cargo test -p shared --test spawn_range_live_refresh_contract -- --nocapture`: **5 / 5 PASS** (renamed Sprint 16 target per PROMPT 1072 Mechanism (d) Cargo `[[test]] name` rename `spawn_range_live_update_contract -> spawn_range_live_refresh_contract`; source file NOT renamed; spawned `.exe` filename no longer matches the Windows AppCompat installer-detection substring heuristic, so no UAC / OS-740 block on this Windows host).
- `git diff --check` / `git diff --cached --check` (smoke worktree): both empty (PASS, exit 0).
- D: free space: 802 GB at smoke entry / 788 GB at smoke exit. No disk-pressure cleanup invoked; well above the 40 GB threshold.

The conditions on this approval are exactly the carry conditions from the Sprint 16 plan + Sprint 16 QA plan (PROMPT 1066) + the Sprint 15 close-out (PROMPT 1056), preserved verbatim (none closed by this report):

1. **`S11-HUD-TIMER-EYEBALL-VISUAL-001`** -- carried OPEN as Must Have human-operator-blocked Sprint 13 -> 14 -> 15 -> 16 carry. **No LLM `/story-done` is authorised**; closure requires a real human-operator screenshot capture session. Allowed to carry into Sprint 17 if no human-operator slot opens within the Sprint 16 timebox; producer documents the cause in Sprint 16 close-out.
2. **`S8-QA-001-W1`** -- manual / browser two-client GAME_OVER gap remains **OPEN**. No Sprint 16 row touched this surface; Sprint 13 story 017 AC12 forbid-auto-closure carries through Sprint 14 / Sprint 15 / Sprint 16 unchanged.
3. **`QA-COND-0005`** -- Standard-tier accessibility remains **accepted-risk / friend-game scope only**. Sprint 16 card-slot primitive work is friend-game visual polish only and does NOT pursue >=44 px hit-targets, WCAG contrast ratios, full keyboard navigation, screen-reader support, colour-blind modes, or text scaling.
4. **`QA-COND-0006`** -- playtest / fun-hypothesis validation remains **accepted-risk / deferred**. Sprint 16 closeout sprint does NOT advance playtest validation.
5. **`PAW-TD-*-a`** -- placeholder-art accept-risk across PAW-002..PAW-006 preserved. Sprint 16 primitive / Cargo-manifest / test-hygiene work does NOT advance placeholder-art resolution.
6. **`TQ-S12-C1..C7`** -- preserved verbatim. **TQ-S12-C2 binding**: no third same-scope retest of Sprint 12 story 019 is authorised by Sprint 16. **TQ-S12-C7** AppCompat informational condition explicitly NOT closed by `S15-OPS-APPCOMPAT-MANIFEST-001` (the manifest row is an ops hygiene robustness improvement; the informational condition closure is a separate decision outside Sprint 16 scope).
7. **PROMPT 683-era runtime divergence question** -- preserved as folded into Sprint 12 story 019 `closed-with-conditions / cannot-reproduce`.
8. **PROMPT 761 `Polish->Release` gate-check `FAIL`** -- preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO retry** attempted by any Sprint 16 row, no retry attempted by PROMPT 1075 smoke, and no retry attempted by this Team-QA. Sprint 16 is **NOT** a `Polish->Release` sprint.
9. **Sprint 12 story 019 underlying drag-runtime bug** -- NOT claimed fixed; `closed-with-conditions / cannot-reproduce` preserved.
10. **PROMPT 1054 P1 UI snapshot visual retest `BLOCKED-HUMAN-OPERATOR`** -- preserved (state record at `origin/main@8bec9dc` per PROMPT 1055). No pixel-level acceptance claimed by Sprint 16.
11. **Sprint 10 / 11 / 12 / 13 / 14 / 15 closeouts** -- preserved unchanged (PROMPT 763 / 792 / 817 / 894 / 987 / 1056 respectively).
12. **24 PROMPT 1022 QA snapshot audit findings** -- remain report-only inputs to future story authoring; none are Sprint 16 active rows; none closed by Sprint 16 or by this report.
13. **PROMPT 1075 smoke evidence integration** -- the smoke evidence file `production/qa/smoke-sprint-16-2026-05-18.md` lives on `origin/qa/sprint-16-smoke-check-1075@56655fc` (strict fast-forward descendant of `origin/main@f8eac30`; merge-base == HEAD). Integration of the evidence file into `origin/main` is a separate paperwork-only integration prompt analogous to the PROMPT 982 / PROMPT 986 pattern for Sprint 14. The smoke result itself stands at PASS-WITH-WARNINGS regardless; integration of the evidence file is a documentation-completeness step, NOT a re-verification. This Team-QA accepts the smoke evidence as authoritative for sign-off purposes.

This Team-QA report makes **no claim** of (preserved non-claims, verbatim):

- no public release readiness
- no release-candidate readiness
- no full game completion
- no broad / Standard-tier accessibility completion (`QA-COND-0005` unchanged)
- no playtest / fun-hypothesis validation (`QA-COND-0006` unchanged)
- no full playable-client manual QA (`S8-QA-001-W1` unchanged)
- no two-client `GAME_OVER` closure
- no final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved)
- no `Polish->Release` retry -- PROMPT 761 Polish->Release gate-check `FAIL` preserved
- no Sprint 16 close-out (this Team-QA sign-off is a precondition to a separate close-out-with-conditions decision, not the close-out itself)
- no `S8-QA-001-W1` closure
- no `S11-HUD-TIMER-EYEBALL-VISUAL-001` closure (human-operator-blocked carry; no LLM `/story-done` authorised; allowed to carry to Sprint 17)
- no `TQ-S12-C7` closure
- no PROMPT 1054 P1 UI snapshot retest closure (`BLOCKED-HUMAN-OPERATOR` preserved)
- no closure of any of the 24 PROMPT 1022 QA snapshot audit findings
- no closure of the three per-surface card-slot migration siblings (Sprint 16+ `S16-UI-CARD-SLOT-MIGRATION-HAND-001` / `-AUCTION-001` / `-BOARD-GHOST-001`)
- no pixel-level QA snapshot capture for the Sprint 16 card-slot primitive shop-panel bundles at 1366x768 / 1920x1080 (story 009 AC6 PARTIAL preserved; QA snapshot bundles remain human-operator-deferred via the `S15-QA-SNAPSHOT-DEFAULT-DEV` flow per `production/qa/evidence/sprint-16-ui-card-slot-primitive/qa-snapshot-1366x768/README.md` + `qa-snapshot-1920x1080/README.md`)
- no stage advance from Polish
- no underlying drag-runtime bug fix (Sprint 12 story 019 `cannot-reproduce` preserved)
- no full UI clean-pass repair beyond the 4 Sprint 16 candidate rows (the three per-surface card-slot migration siblings remain Sprint 16+ Draft authoring candidates)
- no `gate-check` or `release-check` run by PROMPT 1078

PROMPT 1078 did **not** run `/dev-story`, `/smoke-check`, `/gate-check`, `/release-check`, `/story-done`, `/story-readiness`, or `/qa-plan`. PROMPT 1078 did **not** modify production code under `client/`, `server/`, `shared/`, or `tests/`. PROMPT 1078 did **not** modify `production/sprint-status.yaml`, `production/sprints/sprint-16.md`, `production/sprints/sprint-15.md` (or any earlier sprint plan), `production/stage.txt`, `production/gate-checks/`, `production/qa/qa-plan-sprint-16.md`, any Sprint 16 story file, any Sprint 16 evidence file, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`, `.claude/settings.json`, `.octogent/`, or `.claude/scheduled_tasks.lock`.

---

## Verification (Preflight)

| Step | Result |
|---|---|
| `git fetch origin` | OK |
| `git rev-parse HEAD` (this branch) | `f8eac30d98af1ad21ed3ca6dd06e219ce9f9df19` |
| `git rev-parse origin/main` | `f8eac30d98af1ad21ed3ca6dd06e219ce9f9df19` (matches HEAD) |
| `git rev-parse origin/qa/sprint-16-smoke-check-1075` | `56655fc8c20c1aad8485f2de41c656cbb7c96900` |
| `git merge-base origin/main origin/qa/sprint-16-smoke-check-1075` | `f8eac30d98af1ad21ed3ca6dd06e219ce9f9df19` (strict fast-forward; smoke evidence is a compatible descendant of origin/main) |
| `git status --short --branch` (this branch) | clean -- `## qa/sprint-16-team-qa-1078...origin/main`; no modifications, no untracked files |
| `production/stage.txt` | `Polish` (unchanged) |
| `production/sprint-status.yaml` top level | `sprint: 16`, `status: active`, `stage: Polish` |
| PROMPT 761 Polish->Release gate-check FAIL evidence | preserved at `production/gate-checks/gate-polish-release-2026-05-12.md` |
| Sprint 16 smoke evidence (PROMPT 1075) | exists at `production/qa/smoke-sprint-16-2026-05-18.md` on `origin/qa/sprint-16-smoke-check-1075@56655fc` -- **NOT yet integrated to `origin/main`** (see `Pending Integration` approval condition above) |
| Sprint 16 QA plan | exists at `production/qa/qa-plan-sprint-16.md` (PROMPT 1066) |
| Workspace ignored-test count (per smoke aggregate) | **0** (parity with Sprint 12 / 13 / 14 / 15 baseline retired in PROMPT 814; preserved through Sprint 16) |

---

## Sprint 16 Row Closure (3 / 4 done)

Verified by reading `production/sprint-status.yaml` `stories:` block at HEAD `f8eac30`. Each closed row below carries `status: done`, a `completed:` date, integration commits on `origin/main`, a `/story-done` prompt, and acceptance + test evidence references.

### Must Have (0 / 1 done)

| ID | Title | Status | Closed | Integration commit | `/story-done` |
|---|---|---|---|---|---|
| S11-HUD-TIMER-EYEBALL-VISUAL-001 (hud story 014) | HUD Timer Eyeball Visual Check (Sprint 13 -> 14 -> 15 -> 16 carry) | **ready** (human-operator-blocked; `no_llm_story_done: true`) | -- | -- | -- |

### Should Have (1 / 1 done)

| ID | Title | Status | Closed | Integration commit | `/story-done` |
|---|---|---|---|---|---|
| S12-TD-UI-CARD-SLOT-PRIMITIVE-001 (ui-clean-pass story 009) | UI Card-Slot Primitive (Tier 3 rank 13; last canonical UI clean-pass roadmap row) | **done** | 2026-05-17 | `d12adc4` (PROMPT 1073) -- worker `3bdf6ac` (PROMPT 1067) | PROMPT 1074 |

### Nice to Have (2 / 2 done)

| ID | Title | Status | Closed | Integration commit | `/story-done` |
|---|---|---|---|---|---|
| S15-OPS-APPCOMPAT-MANIFEST-001 (devops story 006) | Windows AppCompat Manifest for spawn_range_live_update_contract Test Binary (ops hygiene; Mechanism (d) Cargo `[[test]] name` rename) | **done** | 2026-05-17 | `488a9cd` (PROMPT 1071) -- worker `ed58e3d` (PROMPT 1068) | PROMPT 1072 |
| S15-TD-WORKSPACE-DEAD-CODE-WARNING-001 (ui-clean-pass story 016) | Workspace Dead-Code Warning Cleanup at `tests/integration/presentation/hand_ui_asset_wiring_test.rs` (Option A delete `count_with_image_node`) | **done** | 2026-05-17 | `bd374dd` (PROMPT 1070 final integration tip merging origin/main into PROMPT 1070 integration) -- worker `2251a93` (PROMPT 1069) | PROMPT 1072 |

**Closure**: 3 of 4 rows done. The single open row is the Sprint 13 -> 14 -> 15 -> 16 carry `S11-HUD-TIMER-EYEBALL-VISUAL-001`, carried as Must Have human-operator-blocked per the PROMPT 1064 activation banner and the PROMPT 1056 Sprint 15 close-out decision. Closure requires real two-client browser/native screenshot capture across the three timed phases (`DraftInitial` 45s, `DraftShop` 30s, `Placement` 10-12s); no LLM `/story-done` is authorised; evidence target path `production/qa/evidence/sprint-16-hud-timer-visual-check/` is NEW and unpopulated. This is the same human-operator-blocked carry that has been in place since Sprint 10 smoke retry-7 W2 -> Sprint 11 -> Sprint 12 -> Sprint 13 -> Sprint 14 -> Sprint 15 -> Sprint 16.

### Sprint 16 Acceptance Criteria Outcomes (per `production/sprint-status.yaml` notes)

- **S12-TD-UI-CARD-SLOT-PRIMITIVE-001 (story 009)**: AC1 authoritative primitive module + token usage PASS; AC2 no nested cards / stable aspect ratio PASS; AC3 hover/focus/pressed/disabled state mapping via existing interaction primitives PASS; AC4 image/text containment at 1366x768 + 1024x600 sentinel PASS; AC5 per-surface migration boundaries split into phases PASS (Phase 1 = `shop_slot_node` only; hand + auction featured + board ghost UNCHANGED); AC7 tests including viewport-invariant / layout-contract test PASS; AC8 non-claims (no gameplay / no server / no release / no final-art) PASS; **AC6 visual evidence / screenshot harness expectations PARTIAL** -- paperwork present (evidence.md doc-review checklist, cargo-test pass log, cargo-check pass log, git-diff-stat-disjoint-surfaces.txt, spec-heading-scan.txt, spec-adoption-matrix-diff.md) but QA snapshot bundles at 1366x768 and 1920x1080 deferred to human operator (PROMPT 1067 worker has no playable-client runtime; capture instructions live in `qa-snapshot-1366x768/README.md` and `qa-snapshot-1920x1080/README.md` within the evidence dir). **AC6 PARTIAL is accepted as advisory-only**; the BLOCKING AC7 integration test PASSES at the source-of-truth HEAD; QA snapshot capture remains a Sprint 16+ human-operator follow-up that does **not** reopen the row or block Sprint 16 close-out-with-conditions.
- **S15-OPS-APPCOMPAT-MANIFEST-001 (story 006)**: AC1-AC6 all PASS. AC3 BLOCKING evidence: 5 consecutive `cargo test -p shared --test spawn_range_live_refresh_contract` invocations PASS without rename workaround / no `os error 740` (Mechanism (d) Cargo `[[test]] name` rename; source file NOT renamed); reproduced by PROMPT 1075 smoke at the workspace aggregate level (no AppCompat block observed on this Windows host).
- **S15-TD-WORKSPACE-DEAD-CODE-WARNING-001 (story 016)**: AC1-AC7 all PASS. AC3 BLOCKING evidence: `cargo check --workspace --all-targets` returns zero `count_with_image_node` warning lines; reproduced by PROMPT 1075 smoke at the workspace aggregate level (zero warnings). AC4 BLOCKING test-preservation: 4 `test_fan_slot_chrome_*` tests PASS unchanged; sibling `count_child_of_with` helper preserved.

---

## Smoke Status (PROMPT 1075)

**Verdict**: `PASS-WITH-WARNINGS`. Source: `production/qa/smoke-sprint-16-2026-05-18.md` on `origin/qa/sprint-16-smoke-check-1075@56655fc8c20c1aad8485f2de41c656cbb7c96900` -- strict fast-forward descendant of the source-of-truth HEAD `f8eac30` (merge-base == HEAD; the smoke branch adds exactly one paperwork-only commit on top of `origin/main`).

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS (exit 0, no output) |
| `cargo check --workspace --all-targets` | PASS -- `Finished dev profile [optimized] target(s) in 1m 36s`. **Zero warnings, zero errors.** Pre-existing Sprint 14 `count_with_image_node` dead-code warning resolved by PROMPT 1069 / PROMPT 1070 (`S15-TD-WORKSPACE-DEAD-CODE-WARNING-001` closure). |
| `cargo test --workspace --tests --no-fail-fast` | PASS -- 223 binaries / **1464 passed / 0 failed / 0 ignored / 0 measured / 0 filtered**. No AppCompat OS-740 block on the renamed `spawn_range_live_refresh_contract-*.exe`. |
| `cargo test -p shared --test spawn_range_live_refresh_contract -- --nocapture` | PASS -- **5 / 5** (Mechanism (d) rename binary launches cleanly on this Windows host with no UAC / OS-740 block). |
| `git diff --check` / `git diff --cached --check` (smoke worktree) | PASS (exit 0, empty output) |

**Functional total**: **1464 passed / 0 failed / 0 ignored across 223 binaries** (no carve-outs; AppCompat workaround no longer needed).

### Smoke Warning Classification

The single warning recorded by PROMPT 1075 is **environment / host-state-only**. Specifically:

- At smoke entry, `tasklist` showed externally-running game binaries holding artifacts under the mandated `D:\_DEV\cargo-target\ccgs-msvc` tree: `server.exe` PID 26084, `client.exe` PID 31420, `client.exe` PID 32524 (an additional `client.exe` PID 37564 was briefly observed and exited on its own). These are user / orchestrator-owned processes that the smoke scope explicitly forbids killing.
- The first `cargo test --workspace --tests --no-fail-fast` attempt against the mandated dir failed at the link stage with `error: failed to remove file D:\_DEV\cargo-target\ccgs-msvc\debug\server.exe` / `client.exe` (`Caused by: Access is denied. (os error 5)`). This is a Windows-host file-locking artifact of the live game processes, NOT a code regression.
- The PROMPT 1075 prompt scope authorized stale-target cleanup only at D: free space < 40 GB (the host had 802 GB free at smoke entry). PROMPT 1075 applied an analogous-to-PROMPT-983 environmental workaround: set `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc-smoke-1075` (sibling of the mandated dir under the same `D:\_DEV\cargo-target\` parent) and preserved every other Cargo resource policy field verbatim (`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`). The cargo aggregate then ran cleanly in the sibling tree (1464 / 0 / 0 / 0 / 0 across 223 binaries; `spawn_range_live_refresh_contract` 5 / 5 PASS). Disk delta: ~14 GB (788 GB free at smoke exit).
- **Classification**: environment / host-state-only -- not a code regression, not a product-code regression, not authored by any Sprint 16 closure prompt (PROMPT 1067 / 1068 / 1069 / 1070 / 1071 / 1072 / 1073 / 1074). Owner-identifiable (live developer/orchestrator processes holding cargo target artifacts). Documented (smoke report `Cargo Target Dir Contention Workaround` section). Reproducible mitigation (sibling target dir under same parent; or pre-smoke gate that closes in-flight game binaries; or persistent sibling target dir; each option documented in the smoke report under "Forward-looking options").
- **Per `/smoke-check` verdict rules**: PASS if the suite runs cleanly; PASS-WITH-WARNINGS when results are clean but a known, documented, owner-identifiable environmental issue required a workaround. The PROMPT 1075 smoke matches the latter.

The pre-existing Sprint 14 `count_with_image_node` dead-code warning at `tests/integration/presentation/hand_ui_asset_wiring_test.rs` is **NOT** present in the PROMPT 1075 smoke (resolved by Sprint 16 PROMPT 1069 / 1070 closure). The Sprint 14 AppCompat OS-740 warning on `spawn_range_live_update_contract-*.exe` is **NOT** present in the PROMPT 1075 smoke (resolved by Sprint 16 PROMPT 1068 / 1071 / 1072 Mechanism (d) rename). Sprint 16 has therefore **netted out two prior smoke warnings while introducing one new environmental warning**; the new warning is strictly an external-process contention artifact and not a code regression.

The PROMPT 1075 environmental warning is **accepted by this Team-QA without remediation**: it does NOT indicate a regression and does NOT block Sprint 16 close-out-with-conditions. Forward-looking mitigation (sibling target dir persistence, pre-smoke gate, etc.) remains a backlog candidate; this Team-QA does not file it as a Sprint 16 active row or as a gate-blocking remediation.

---

## Skill / Engine Compliance (per CLAUDE.md routing)

| Surface | Skill activated by worker | Verification |
|---|---|---|
| `client/src/ui/design_tokens/card_slot.rs` (NEW), `client/src/ui/design_tokens/mod.rs` (aggregator), `client/src/ui/shop_auction/mod.rs::shop_slot_node` (Phase 1 migration), `tests/integration/ui_clean_pass/card_slot_primitive_test.rs` (NEW) | `liv-bevy-018` MANDATORY (Bevy 0.18 Required Components API; UI; bevy_ui imports) | Activated by PROMPT 1067 worker per Sprint 16 QA plan `liv-bevy-018` MANDATORY clause for story 009 (`production/qa/qa-plan-sprint-16.md` line ~184). Integration test bin registered in `client/Cargo.toml` as `ui_clean_pass_card_slot_primitive_test` and PASSES per evidence dir `production/qa/evidence/sprint-16-ui-card-slot-primitive/`. Verified by PROMPT 1075 smoke (full workspace test PASS at 1464 / 0). |
| `shared/Cargo.toml` `[[test]] name = "spawn_range_live_refresh_contract"` (renamed from `spawn_range_live_update_contract`; source file NOT renamed) | `liv-bevy-018` NOT required (Cargo manifest-only edit; no `.rs` change) | PROMPT 1068 worker per Sprint 16 QA plan `liv-bevy-018` NOT REQUIRED clause for story 006. Verified by PROMPT 1075 smoke (5 / 5 PASS on renamed binary; full workspace aggregate also PASS). |
| `tests/integration/presentation/hand_ui_asset_wiring_test.rs` (Option A delete `count_with_image_node` helper; sibling `count_child_of_with` preserved) | `liv-bevy-018` MANDATORY (test file imports `bevy`) | PROMPT 1069 worker per Sprint 16 QA plan `liv-bevy-018` MANDATORY clause for story 016. Verified by PROMPT 1075 smoke (zero warnings; 4 PAW-002-f chrome-presence tests PASS unchanged). |

**`liv-bevy-lightyear` activation**: NOT required for any Sprint 16 row. AC8 binding zero protocol diff for story 009 verified by PROMPT 1067 worker via `git diff origin/main...HEAD -- shared/src/protocol.rs` returning empty. Stories 006 + 016 do not touch `shared/src/protocol.rs` or any lightyear surface.

---

## Carried OPEN / Accepted-Risk (NOT closed by Sprint 16 or this report)

| Disposition | Item | Source | Notes |
|---|---|---|---|
| OPEN | `S8-QA-001-W1` | Sprint 8 carry through every subsequent sprint | Manual / browser two-client GAME_OVER gap. No Sprint 16 row touched this surface. |
| OPEN (human-operator-blocked) | `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Sprint 13 -> 14 -> 15 -> 16 Must Have carry) | PROMPT 822 / 823 / 894 / 897 / 987 / 997 / 1056 / 1064 | Manual 2-client run + screenshot capture across `DraftInitial 45s` / `DraftShop 30s` / `Placement 10-12s`. No LLM `/story-done` authorised. Allowed to carry into Sprint 17. |
| accepted-risk | `QA-COND-0005` | Friend-game scope | Standard-tier accessibility not pursued. |
| accepted-risk / deferred | `QA-COND-0006` | Friend-game scope | Playtest / fun-hypothesis validation not pursued. |
| accepted-risk | `PAW-TD-*-a` across PAW-002..PAW-006 | Friend-game scope | Placeholder-art preserved. Sprint 16 primitive / Cargo-manifest / test-hygiene work does not advance final-art. |
| preserved | `TQ-S12-C1..C7` | Sprint 12 Team-QA conditions | Verbatim. TQ-S12-C2 binding: no third same-scope retest of Sprint 12 story 019. **TQ-S12-C7** AppCompat informational condition explicitly NOT closed by `S15-OPS-APPCOMPAT-MANIFEST-001` (the manifest row is ops hygiene; the informational condition closure is a separate decision). |
| preserved (folded) | PROMPT 683-era runtime divergence question | Sprint 12 story 019 `cannot-reproduce` closure | Not re-litigated by Sprint 16. |
| FAIL (preserved; NO retry) | PROMPT 761 Polish->Release gate-check | `production/gate-checks/gate-polish-release-2026-05-12.md` | Stage remains Polish. Sprint 16 did not retry; PROMPT 1075 did not retry; this Team-QA does not retry. |
| `closed-with-conditions / cannot-reproduce` (preserved) | Sprint 12 story 019 underlying drag-runtime bug | Sprint 12 closeout | NOT claimed fixed by Sprint 16. |
| BLOCKED-HUMAN-OPERATOR (preserved) | PROMPT 1054 P1 UI snapshot visual retest | `reports/PROMPT-1054-s15-p1-ui-snapshot-visual-retest.md` | Pixel-level acceptance remains deferred to a human-operator QA snapshot capture session. |
| advisory-only PARTIAL (not blocking) | Story 009 AC6 QA snapshot bundles at 1366x768 + 1920x1080 | `production/qa/evidence/sprint-16-ui-card-slot-primitive/qa-snapshot-1366x768/README.md` + `qa-snapshot-1920x1080/README.md` | Paperwork present; pixel-level capture remains human-operator deferred via `S15-QA-SNAPSHOT-DEFAULT-DEV` flow. Does NOT reopen the row and does NOT block close-out. |
| Draft (not Sprint 16 active rows) | `S16-UI-CARD-SLOT-MIGRATION-HAND-001` / `-AUCTION-001` / `-BOARD-GHOST-001` | Sprint 16+ follow-on family | Per-surface card-slot migration siblings (hand fan + draft grid + auction featured + board staged ghost) deferred from Sprint 16 by story 009 AC5 phase boundary; remain Draft authoring candidates. |
| Report-only (not Sprint 16 active rows) | 24 PROMPT 1022 QA snapshot audit findings | `reports/PROMPT-1022-qa-snapshot-visual-state-audit.md` | Inputs to future story authoring; none closed by Sprint 16. |
| preserved | Sprint 10 / 11 / 12 / 13 / 14 / 15 closeouts | PROMPT 763 / 792 / 817 / 894 / 987 / 1056 respectively | Unchanged. |

---

## Approval Conditions

This APPROVED-WITH-CONDITIONS verdict is conditioned on the following understandings, all of which are already documented in the Sprint 16 plan + Sprint 16 QA plan + Sprint 15 close-out and require no remediation:

1. **`S11-HUD-TIMER-EYEBALL-VISUAL-001`** remains open as Sprint 16 Must Have human-operator-blocked. Its closure (or carry forward into Sprint 17) is a separate paperwork decision and is **NOT** a blocker for Sprint 16 close-out-with-conditions under the plan's "deferrable / human-operator-blocked / human-later" disposition. The PROMPT 822 author + PROMPT 823 + PROMPT 1000 READY status remain valid against the source-of-truth HEAD (story 014 unchanged on origin/main through four sprint carries).
2. **PROMPT 1075 smoke evidence not yet on `origin/main`.** The `production/qa/smoke-sprint-16-2026-05-18.md` file lives on `origin/qa/sprint-16-smoke-check-1075@56655fc` (strict fast-forward descendant; merge-base == HEAD). Integration of the smoke evidence into `origin/main` is a separate paperwork-only integration prompt analogous to the PROMPT 982 / PROMPT 986 pattern for Sprint 14. The smoke result itself stands at PASS-WITH-WARNINGS regardless; integration of the evidence file is a documentation-completeness step, NOT a re-verification. This Team-QA accepts the smoke evidence as authoritative for sign-off purposes.
3. **Smoke warnings remain environment / host-state-only and accepted unchanged.** The Cargo target dir contention from externally-running game binaries (`server.exe` + 2x `client.exe`) is out of Sprint 16 row scope and out of close-out scope. Forward-looking mitigation options are backlog candidates only.
4. **PROMPT 761 Polish->Release `FAIL`** preserved with no retry. Stage remains `Polish`. Sprint 16 is NOT a release sprint and does NOT advance the project to Release.
5. **`S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`, PROMPT 1054 P1 UI snapshot retest `BLOCKED-HUMAN-OPERATOR`, 24 PROMPT 1022 QA snapshot audit findings** all preserved unchanged.
6. **All cargo invocations during Sprint 16** (PROMPT 1068 / 1069 workers; PROMPT 1070 / 1071 / 1073 integrations; PROMPT 1075 smoke; not this Team-QA) applied the binding Windows / MSVC Cargo resource policy (`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc` mandated; PROMPT 1075 documented the sibling-dir environmental workaround under the same `D:\_DEV\cargo-target\` parent; all other policy fields preserved verbatim across all Sprint 16 Cargo invocations), per PROMPT 815 / 833 / 884 / 982 / 983 / 1066 binding precedent.
7. **Story 009 AC6 PARTIAL is advisory-only and not blocking**. The BLOCKING AC7 integration test PASSES; QA snapshot bundles at 1366x768 + 1920x1080 remain human-operator-deferred via the `S15-QA-SNAPSHOT-DEFAULT-DEV` flow per the placeholder READMEs in the evidence dir. Closure of AC6 to a pixel-level state is a Sprint 16+ human-operator follow-up that does not reopen the row.
8. **No product / test failure discovered** in the PROMPT 1075 smoke evidence. The single warning is strictly environmental / host-state-only and requires no NEEDS-REPAIR routing. This Team-QA does **not** specify any repair prompt and does **not** route a follow-on `/dev-story`.

---

## Sprint 16 Next Launchable Prompts (post-Team-QA)

Sprint 16 Team-QA approval enables, but does not perform, the following downstream paperwork (each as a separate prompt):

1. **PROMPT 1075 smoke evidence integration** (paperwork-only fast-forward merge of `origin/qa/sprint-16-smoke-check-1075@56655fc` -> `origin/main`; adds `production/qa/smoke-sprint-16-2026-05-18.md`; analogous to PROMPT 982 / PROMPT 986 Sprint 14 pattern). Optional pre-close-out completeness step; NOT a hard blocker because the smoke result is already authoritative for sign-off.
2. **Sprint 16 close-out-with-conditions** (paperwork-only; flips `production/sprint-status.yaml` top-level `status: active -> closed-with-conditions`; appends `sprint_16_closeout:` block at EOF following `sprint_15_closeout:` precedent; carries `S11-HUD-TIMER-EYEBALL-VISUAL-001` into Sprint 17 as Must Have human-operator-blocked unless a human-operator slot closes it on `origin/main` before the close-out prompt runs; preserves stage `Polish` verbatim; preserves PROMPT 761 Polish->Release FAIL verbatim; preserves all carried conditions verbatim). The Sprint 16 close-out is **NOT** performed by PROMPT 1078; it is a separate explicit prompt that mirrors the PROMPT 763 / 792 / 817 / 894 / 987 / 1056 close-out pattern.
3. **Sprint 17 plan draft + activation** (after Sprint 16 close-out lands on `origin/main`; mirrors the PROMPT 988 / 997 / 1024 / 1064 draft + activation pattern). Sprint 17 candidates include: the three per-surface card-slot migration siblings (`S16-UI-CARD-SLOT-MIGRATION-HAND-001` / `-AUCTION-001` / `-BOARD-GHOST-001`), the human-operator-blocked HUD timer eyeball carry (if not closed by then), pulled candidates from the 24 PROMPT 1022 audit findings.
4. **Human-operator QA snapshot capture session** (parallel-safe with all of the above; producer-scheduled). Captures Sprint 16 card-slot primitive shop-panel bundles at 1366x768 + 1920x1080 to discharge story 009 AC6 PARTIAL; replaces placeholder READMEs in `production/qa/evidence/sprint-16-ui-card-slot-primitive/qa-snapshot-{1366x768,1920x1080}/`. Does not reopen the closed row.
5. **Human-operator HUD timer screenshot capture session** (parallel-safe; producer-scheduled). Discharges `S11-HUD-TIMER-EYEBALL-VISUAL-001` Sprint 16 Must Have carry; after evidence lands, a separate paperwork `/story-done` prompt flips the row status.

**Explicitly NOT next**: `gate-check` (no Polish->Release retry in scope; PROMPT 761 FAIL preserved), `release-check`, `/dev-story` (no further Sprint 16 implementation rows), stage advance from Polish to Release.

---

## Files Changed by PROMPT 1078

| File | Status | Notes |
|---|---|---|
| `production/qa/team-qa-sprint-16-2026-05-18.md` | NEW (this file) | Sprint 16 Team-QA report; matches `team-qa-sprint-N-YYYY-MM-DD.md` naming convention (precedent: `team-qa-sprint-10-2026-05-11.md`, `team-qa-sprint-11-2026-05-13.md`, `team-qa-sprint-12-2026-05-14.md`, `team-qa-sprint-14-2026-05-16.md`). |
| `production/session-state/active.md` | MODIFIED | PROMPT 1078 banner prepended above PROMPT 1074 banner. |
| `production/session-state/codex-orchestrator-state.md` | MODIFIED | PROMPT 1078 section prepended above PROMPT 1074 section. |
| `reports/PROMPT-1078-Sprint-16-Team-QA.md` | NEW | Mandatory final report (gitignored; NOT staged or committed). |

Explicitly NOT touched (forbidden by PROMPT 1078 scope):

- `client/`, `server/`, `shared/`, `tests/` -- no production or test code edits.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml` -- no build / CI / cargo policy edits.
- `production/sprint-status.yaml` -- no row flips, no top-level edits, no `sprint_16_*` block edits.
- `production/stage.txt` -- no stage advance.
- `production/sprints/sprint-16.md` (and all earlier sprint plans) -- no sprint plan edits.
- `production/qa/qa-plan-sprint-16.md` -- no QA plan edits.
- `production/qa/smoke-sprint-16-2026-05-18.md` -- NOT created on this branch (lives on `origin/qa/sprint-16-smoke-check-1075`); integration is a separate prompt.
- Any Sprint 16 story file under `production/epics/` -- no story edits, no row flips, no `/story-done` invocation.
- Any Sprint 16 evidence file under `production/qa/evidence/` -- preserved verbatim on origin/main.
- `production/gate-checks/` -- PROMPT 761 Polish->Release FAIL preserved verbatim.
- `.octogent/`, `.claude/scheduled_tasks.lock`, `.claude/settings.json` -- not touched.
- No cargo / trunk / CI command invoked.

---

## Final Line

```
1078: SPRINT-16-TEAM-QA: APPROVED-WITH-CONDITIONS
```
