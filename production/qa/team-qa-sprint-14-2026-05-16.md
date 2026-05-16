# Team-QA Report: Sprint 14 (Polish / friend-game scope)

| Field | Value |
|---|---|
| **Date** | 2026-05-16 |
| **Sprint** | Sprint 14 -- `active` (Polish stage; activated by PROMPT 897) |
| **Stage** | `Polish` (unchanged) |
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Scope** | Friend-game / Polish slice -- UI clean-pass Tier 0 foundation + Tier 1 layout composition + Sprint 13 HUD timer eyeball carry. Explicitly NOT public release readiness. |
| **Skill** | `/team-qa sprint` (qa-lead + producer roles; serialized shared-status writer per 2026-05-13 orchestrator override; no spawned agents -- paperwork-only single-context review of record). |
| **Prompt** | PROMPT 984 -- Sprint 14 Team QA |
| **Worktree** | `D:\_DEV\Work\Claude-Code-Game-Studios` root checkout, branch `work/s14-team-qa` (NEW; tracks `origin/main`). Root-checkout dirt that existed at prompt entry (`M .claude/settings.json`, `M AGENTS.md`, `M CODEX.md`, `M production/epics/hud/story-016-hud-bottom-strip-layout.md`, `M production/epics/shop-auction-ui/EPIC.md`, `M production/epics/shop-auction-ui/story-018-auction-lead-loss-state.md`, `M production/qa/qa-plan-sprint-14.md`, `M production/session-state/active.md`, `M production/session-state/codex-orchestrator-state.md`, `M production/sprint-status.yaml`, `M production/sprints/sprint-14.md`, plus 5 untracked) was stashed (`PROMPT-984-pre-qa-stash`) before checkout and is NOT touched, staged, deleted, modified, or relied on by this report. |
| **Branch** | `work/s14-team-qa` (NEW; tracks `origin/main`) |
| **Commit Under Review (HEAD)** | `f94f4893cae3690372c5a12f81145de42bb4d94e` (`integrate(s14): merge work/s14-smoke-repair-ui-drift (PROMPT 982)`) |
| **HEAD == origin/main** | yes (verified after `git fetch origin`) |
| **Review mode** | Lean (no `production/review-mode.txt` override) |
| **Cargo policy applied** | **N/A** -- no `cargo` command was invoked by PROMPT 984 (paperwork-only, review-of-record on existing PROMPT 982 integration + PROMPT 983 smoke evidence; PROMPT 982 and PROMPT 983 both applied the binding Windows/MSVC cargo resource policy at their respective Cargo invocations). |

---

## Verdict: APPROVED-WITH-CONDITIONS

Sprint 14 stands at **16 of 17 rows closed** on `origin/main@f94f489`:

- **Must Have: 9 / 9 done** (all Tier 0 ranks 1-6 + Tier 1 ranks 7, 10, 12 closed).
- **Should Have: 3 / 4 done.** The single open row is the Sprint 13 carry `S11-HUD-TIMER-EYEBALL-VISUAL-001` (story 014), which is **human-operator-blocked** per the original Sprint 13 closeout (PROMPT 894) and the Sprint 14 plan -- closure requires a real two-client run with screenshot capture across `DraftInitial` 45s / `DraftShop` 30s / `Placement` 10-12s phases; no LLM `/story-done` is authorised. This is NOT a regression and NOT a remediable blocker for Sprint 14 close-out.
- **Nice to Have: 4 / 4 done.**

The Sprint 14 smoke rerun (PROMPT 983) is **PASS-WITH-WARNINGS** on the same HEAD this report reviews, with the single warning classified as **environment/tool-only** (Windows AppCompat false positive; no code regression -- see `Smoke Warning Classification` below). The PROMPT 978/979/982 UI drift repair tests are at parity:

- `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test`: **8 / 8 PASS**.
- `cargo test -p client --test ui_clean_pass_z_layers_test`: **6 / 6 PASS**.
- `cargo test --workspace --tests --no-fail-fast` aggregate: **213 binaries / 1350 passed / 0 failed / 0 ignored / 0 measured / 0 filtered out**, plus the AppCompat-blocked binary verified passing (5 / 5) via the documented rename workaround. Functional total: **1355 passed / 0 failed / 0 ignored across 214 effective binaries**.

The conditions on this approval are exactly the carry conditions from the Sprint 14 plan + QA plan, preserved verbatim (none closed by this report):

1. **`S11-HUD-TIMER-EYEBALL-VISUAL-001`** -- carried OPEN; human-operator-blocked manual visual check.
2. **`S8-QA-001-W1`** -- manual / browser two-client GAME_OVER gap remains **OPEN**. No Sprint 14 row touched this surface.
3. **`QA-COND-0005`** -- Standard-tier accessibility remains **accepted-risk** (friend-game scope). Sprint 14 UI clean-pass was friend-game visual polish only. The L5 `LOBBY_BUTTON_HEIGHT = 30.0` defect remains accepted-risk; story 026 was layout-stability work, not ≥44 px hit-target conformance.
4. **`QA-COND-0006`** -- playtest / fun-hypothesis validation remains **accepted-risk / deferred**. No playtest sessions were required or run by any Sprint 14 row.
5. **`PAW-TD-*-a`** -- placeholder-art accept-risk across PAW-002..PAW-006 preserved. Story 016 auction featured card, story 017 HUD opponent figurine, and story 018 auction lead-loss differentiation were achieved by layout / composition / scale / typography / token color, NOT by final-art replacement.
6. **`TQ-S12-C1..C7`** -- preserved verbatim. TQ-S12-C2 binding: no third same-scope retest of Sprint 12 story 019 is authorised by Sprint 14.
7. **PROMPT 683-era runtime divergence question** -- preserved as folded into Sprint 12 story 019 `closed-with-conditions / cannot-reproduce`.
8. **PROMPT 761 `Polish->Release` gate-check `FAIL`** -- preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO retry** attempted by any Sprint 14 row and no retry attempted by this Team-QA.
9. **Sprint 12 story 019 underlying drag-runtime bug** -- NOT claimed fixed.
10. **Sprint 10 / 11 / 12 / 13 closeouts** -- preserved unchanged.

This Team-QA report makes **no claim** of (preserved non-claims):

- no public release readiness
- no release-candidate readiness
- no full game completion
- no broad / Standard-tier accessibility completion (`QA-COND-0005` unchanged)
- no playtest / fun-hypothesis validation (`QA-COND-0006` unchanged)
- no full playable-client manual QA (`S8-QA-001-W1` unchanged)
- no two-client GAME_OVER closure
- no final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved)
- no `Polish->Release` retry -- PROMPT 761 Polish->Release gate-check `FAIL` preserved
- no Sprint 14 close-out (this Team-QA sign-off is a precondition to a separate close-out decision, not the close-out itself)
- no `S8-QA-001-W1` closure
- no `S11-HUD-TIMER-EYEBALL-VISUAL-001` closure
- no stage advance from Polish
- no underlying drag-runtime bug fix
- no full UI clean-pass repair beyond the 17 Sprint 14 candidate rows (Tier 2 / Tier 3 ranks remain out of scope)

PROMPT 984 did **not** run `/dev-story`, `/smoke-check`, `/gate-check`, `/release-check`, `/story-done`, `/story-readiness`, or `/qa-plan`. PROMPT 984 did **not** modify production code under `client/`, `server/`, `shared/`, or `tests/`. PROMPT 984 did **not** modify `production/sprint-status.yaml`, `production/sprints/sprint-14.md`, `production/sprints/sprint-13.md`, `production/stage.txt`, `production/gate-checks/`, `production/qa/qa-plan-sprint-14.md`, any Sprint 14 story file, any Sprint 14 evidence file, `.claude/settings.json`, `.octogent/`, `.claude/scheduled_tasks.lock`, or `reports/` (other than the mandatory `reports/PROMPT-984-*.md` final-report file, which is gitignored and NOT staged or committed). The root-checkout dirt that existed at prompt entry was stashed under `PROMPT-984-pre-qa-stash` before branch checkout and was NOT touched.

---

## Verification (Preflight)

| Step | Result |
|---|---|
| `git fetch origin` | OK |
| `git rev-parse HEAD` (this branch) | `f94f4893cae3690372c5a12f81145de42bb4d94e` |
| `git rev-parse origin/main` | `f94f4893cae3690372c5a12f81145de42bb4d94e` (matches HEAD) |
| `git status --short` (this branch) | clean -- no modifications, no untracked files (pre-existing root-checkout dirt was stashed before branch checkout) |
| `production/stage.txt` | `Polish` (unchanged) |
| `production/sprint-status.yaml` top level | `sprint: 14`, `status: active`, `stage: Polish` |
| PROMPT 761 Polish->Release gate-check FAIL evidence | preserved at `production/gate-checks/gate-polish-release-2026-05-12.md` (`Verdict: **FAIL**`, line 14) |
| Sprint 14 smoke evidence (PROMPT 983 rerun) | exists at `production/qa/smoke-sprint-14-2026-05-16-rerun.md` on `origin/qa/sprint-14-smoke-rerun-983@0053f2b6` -- **NOT yet integrated to `origin/main`** (see `Pending Integration` condition below) |
| Sprint 14 QA plan | exists at `production/qa/qa-plan-sprint-14.md` (PROMPT 898) |
| Workspace ignored-test count | **0** (per PROMPT 983 smoke aggregate: `0 ignored`; Sprint 12 retirement under PROMPT 814 preserved through Sprint 14) |

---

## Sprint 14 Row Closure (16 / 17 done)

Verified by reading `production/sprint-status.yaml` `stories:` block at HEAD `f94f489`. Each row below carries `status: done`, a `completed:` date, integration commits on `origin/main`, a `/story-done` prompt, and acceptance + test evidence references.

### Must Have (9 / 9 done)

| ID | Title | Status | Closed | Integration commit | `/story-done` |
|---|---|---|---|---|---|
| S11-TD-UI-ZINDEX-LAYERS (story 002) | Centralised UI Z-Index Layer Constants | **done** | 2026-05-15 | `36c0b4b` (PROMPT 902) | PROMPT 903 |
| S11-TD-UI-FONT-CONSTANTS (story 003) | Typography Scale Tokens | **done** | 2026-05-15 | `eb1c128` (PROMPT 906) | PROMPT 908 |
| S11-TD-UI-FLEX-STRIPS (story 004) | Flex Strip Composition Primitives + SPACING tokens | **done** | 2026-05-15 | `6ab4a27` (PROMPT 918) | PROMPT 919 |
| S11-TD-UI-VIEWPORT-INVARIANT-TESTS (story 005) | Viewport-Invariant Test Bin (6-viewport matrix) | **done** | 2026-05-15 | `42eae31` (PROMPT 907) | PROMPT 909 |
| S12-TD-UI-OVERLAY-ALPHA-TOKEN-001 (story 006) | Single-Source Overlay Alpha Token | **done** | 2026-05-15 | `c4e1936` (PROMPT 917) | PROMPT 921 |
| S12-UX-GLOBAL-UI-DESIGN-SPEC-001 (story 007) | Canonical Global UI Design Spec | **done** | 2026-05-15 | `3d99a04` (PROMPT 912) | PROMPT 922 |
| S11-UX-HUD-TOP-STRIP-LAYOUT (hud story 015) | HUD Top Strip Layout (HeaderBar flex) | **done** | 2026-05-15 | `4b9a23b` (PROMPT 941) | PROMPT 942 |
| S11-UX-AUCTION-FEATURED-CARD (shop-auction-ui story 016) | Auction Featured Card Visual Hierarchy | **done** | 2026-05-15 | `b828587` (PROMPT 930) | PROMPT 931 |
| S12-UX-LOBBY-LAYOUT-MODAL-001 (playable-client story 024) | Lobby Layout Modal (Option A centred panel) | **done** | 2026-05-15 | `c25aba7` (PROMPT 938) | PROMPT 939 |

### Should Have (3 / 4 done)

| ID | Title | Status | Closed | Integration commit | `/story-done` |
|---|---|---|---|---|---|
| S11-HUD-TIMER-EYEBALL-VISUAL-001 (hud story 014) | HUD Timer Eyeball Visual Check (Sprint 13 carry) | **ready** (human-operator-blocked) | -- | -- | -- |
| S11-UX-HUD-BOTTOM-STRIP-LAYOUT (hud story 016) | HUD Bottom Strip Layout (FooterBar flex) | **done** | 2026-05-15 | `45c2d03` (PROMPT 955) | PROMPT 956 |
| S11-UX-DRAFT-GRID-CENTERED-MODAL (shop-auction-ui story 015) | Draft Initial Grid Centered Modal Layout | **done** | 2026-05-15 | `a9721bc` (PROMPT 951) | PROMPT 953 |
| S11-UX-LOBBY-CLASS-PICKER (playable-client story 025) | Lobby Class-Picker Layout & Hierarchy | **done** | 2026-05-16 | `fed5fb9` (PROMPT 961) | PROMPT 962 |

### Nice to Have (4 / 4 done)

| ID | Title | Status | Closed | Integration commit | `/story-done` |
|---|---|---|---|---|---|
| S11-UX-HUD-OPP-FIGURINE (hud story 017) | HUD Opponent Figurine Composition | **done** | 2026-05-16 | `a3bc885` (PROMPT 975) | PROMPT 976 |
| S11-UX-AUCTION-FREE-GOLD-COUNTERS (shop-auction-ui story 017) | Auction Free-Gold Counters Layout & Readability | **done** | 2026-05-16 | `5f5e72f` (PROMPT 959) | PROMPT 960 |
| S11-UX-LOBBY-BUTTON-HITTARGETS (playable-client story 026) | Lobby Button Dimensions & Hit-Target Stability (friend-game scope) | **done** | 2026-05-16 | `2e0715f` (PROMPT 970) | PROMPT 972 |
| S12-UX-AUCTION-LEAD-LOSS-STATE-001 (shop-auction-ui story 018) | Auction Featured Card Leading / Losing State (PROMPT 967 producer-decision-4 Option A) | **done** | 2026-05-16 | `e3ca5d6` (PROMPT 973) | PROMPT 974 |

**Closure**: 16 of 17 rows done. The single open row is Sprint 13 carry `S11-HUD-TIMER-EYEBALL-VISUAL-001`, carried as Should Have human-operator-blocked per PROMPT 894 closeout. Closure requires real two-client browser/native screenshot capture across the three timed phases; no LLM `/story-done` is authorised; evidence target path `production/qa/evidence/sprint-14-hud-timer-visual-check/` is NEW and unpopulated. This is the same human-operator-blocked carry that has been in place since Sprint 10 retry-7 W2 -> Sprint 11 -> Sprint 12 -> Sprint 13 -> Sprint 14.

---

## Smoke Status (PROMPT 983 Rerun)

**Verdict**: `PASS-WITH-WARNINGS`. Source: `production/qa/smoke-sprint-14-2026-05-16-rerun.md` on `origin/qa/sprint-14-smoke-rerun-983@0053f2b6` (NOT yet integrated to `origin/main`; same HEAD `f94f4893` as this Team-QA reviews).

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS (exit 0, no output) |
| `cargo check --workspace --all-targets` | PASS -- `Finished dev profile [optimized] target(s) in 20.06s`. 1 pre-existing dead-code warning (`count_with_image_node` at `tests/integration/presentation/hand_ui_asset_wiring_test.rs:43`); NOT introduced by Sprint 14. |
| `cargo test --workspace --tests --no-fail-fast` | PASS-WITH-WARNINGS -- 213 binaries / **1350 passed / 0 failed / 0 ignored / 0 measured / 0 filtered**; 1 binary `spawn_range_live_update_contract-*.exe` failed to spawn under cargo on Windows (OS error 740 -- AppCompat elevation heuristic on `update` substring). |
| `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test -- --nocapture` | PASS -- **8 / 8** (parity with PROMPT 982 integration result). |
| `cargo test -p client --test ui_clean_pass_z_layers_test -- --nocapture` | PASS -- **6 / 6** (parity with PROMPT 982 integration result). |
| AppCompat renamed-binary rerun (`srluc_appcompat_renamed.exe`, 5 consecutive runs) | **5 / 5 runs PASS -- each 5 / 5 tests** (identical AppCompat classification to PROMPT 815 / 790 / 979 / 982). |
| `git diff --check` / `git diff --cached --check` (smoke worktree) | PASS (exit 0, empty output) |

**Functional total** (cargo aggregate + 5 from direct-run of the renamed binary): **1355 passed / 0 failed / 0 ignored across 214 effective binaries**.

### Smoke Warning Classification

The single warning is **environment / tool-only**. Specifically:

- The Windows Application Compatibility installer-detection heuristic intercepts the spawn of any executable whose filename contains the substring `update` and demands UAC elevation unless an embedded application manifest declares `level="asInvoker"`. Cargo-emitted rustc test binaries do not embed such a manifest, so the spawn fails with OS error 740 ("requires elevation").
- Manual mitigation in PROMPT 983: rename the binary to `srluc_appcompat_renamed.exe` (drop the `update` substring) and invoke directly. Five consecutive invocations returned `ok. 5 passed; 0 failed; 0 ignored`. The product code under test passes; the cargo runner simply cannot launch the binary on this Windows host under its current name.
- Identical classification to PROMPT 815 (Sprint 12 smoke), PROMPT 790 (earlier observation), PROMPT 979 (UI drift repair worker), and PROMPT 982 (UI drift repair integration). No code regression. Owner-identifiable. Documented. Reproducible mitigation.

The pre-existing `count_with_image_node` dead-code warning at `tests/integration/presentation/hand_ui_asset_wiring_test.rs:43` was carried forward unchanged from Sprint 13 baseline; it was NOT introduced by any Sprint 14 row and is out of Sprint 14 smoke scope. The Sprint 14 plan does not require its removal; if the producer wishes to silence it, that is a separate (Sprint 15-candidate) Nice-to-Have row.

Both warnings are accepted by this Team-QA without remediation: neither indicates a regression and neither blocks Sprint 14 close-out.

---

## Carried OPEN / Accepted-Risk (NOT closed by Sprint 14 or this report)

| Disposition | Item | Source | Notes |
|---|---|---|---|
| OPEN | `S8-QA-001-W1` | Sprint 8 carry through every subsequent sprint | Manual / browser two-client GAME_OVER gap. No Sprint 14 row touched this surface. Story 017 (Sprint 13) AC12 forbid-auto-closure preserved. |
| OPEN (human-operator-blocked) | `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Sprint 13 carry into Sprint 14 Should Have) | PROMPT 822 / 823 / 894 / 897 | Manual 2-client run + screenshot capture across `DraftInitial 45s` / `DraftShop 30s` / `Placement 10-12s`. No LLM `/story-done` authorised. |
| accepted-risk | `QA-COND-0005` | Friend-game scope | Standard-tier accessibility not pursued. The L5 `LOBBY_BUTTON_HEIGHT = 30.0` defect remains accepted-risk; story 026 was friend-game stability work, not ≥44 px hit-target conformance. |
| accepted-risk / deferred | `QA-COND-0006` | Friend-game scope | Playtest / fun-hypothesis validation not pursued. |
| accepted-risk | `PAW-TD-*-a` across PAW-002..PAW-006 | Friend-game scope | Placeholder-art preserved. Story 016 featured-card / story 017 figurine / story 018 lead-loss differentiation achieved by layout / token color, not final-art replacement. |
| preserved | `TQ-S12-C1..C7` | Sprint 12 Team-QA conditions | Verbatim. TQ-S12-C2 binding: no third same-scope retest of Sprint 12 story 019. |
| preserved (folded) | PROMPT 683-era runtime divergence question | Sprint 12 story 019 `cannot-reproduce` closure | Not re-litigated by Sprint 14. |
| FAIL (preserved; NO retry) | PROMPT 761 Polish->Release gate-check | `production/gate-checks/gate-polish-release-2026-05-12.md` | Stage remains Polish. Sprint 14 did not retry; this Team-QA does not retry. |
| `closed-with-conditions / cannot-reproduce` (preserved) | Sprint 12 story 019 underlying drag-runtime bug | Sprint 12 closeout | NOT claimed fixed by Sprint 14. |
| preserved | Sprint 10 / 11 / 12 / 13 closeouts | PROMPT 763 / 792 / 817 / 894 respectively | Unchanged. |

---

## Approval Conditions

This APPROVED-WITH-CONDITIONS verdict is conditioned on the following understandings, all of which are already documented in the Sprint 14 plan + QA plan and require no remediation:

1. **`S11-HUD-TIMER-EYEBALL-VISUAL-001`** remains open as Sprint 14 Should Have human-operator-blocked. Its closure (or carry forward into Sprint 15) is a separate paperwork decision and is NOT a blocker for Sprint 14 close-out under the plan's "deferrable / human-operator-blocked" disposition. The PROMPT 822 author + PROMPT 823 READY status remain valid.
2. **PROMPT 983 smoke evidence not yet on `origin/main`.** The `production/qa/smoke-sprint-14-2026-05-16-rerun.md` file lives on `origin/qa/sprint-14-smoke-rerun-983@0053f2b6` (worker branch). Integration of the smoke evidence into `origin/main` is a separate paperwork-only integration prompt (mirroring the PROMPT 982 pattern for the worker repair branch, but evidence-only). The smoke result itself stands at PASS-WITH-WARNINGS regardless; integration of the evidence file is a documentation completeness step, NOT a re-verification. This Team-QA accepts the smoke evidence as authoritative for sign-off purposes.
3. **Smoke warnings remain environment / tool-only and accepted unchanged.** The AppCompat false positive on `spawn_range_live_update_contract` and the pre-existing `count_with_image_node` dead-code warning are both out of Sprint 14 scope and out of close-out scope.
4. **PROMPT 761 Polish->Release `FAIL`** preserved with no retry. Stage remains `Polish`. Sprint 14 is NOT a release sprint and does NOT advance the project to Release.
5. **`S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`** all preserved unchanged.
6. **All cargo invocations during Sprint 14** (worker + integration prompts; not this Team-QA) applied the binding Windows / MSVC cargo resource policy (`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, `RUSTFLAGS="-C debuginfo=0 -C link-arg=/DEBUG:NONE"`), per PROMPT 815 / 833 / 884 / 982 / 983 binding precedent.

---

## Files Changed by PROMPT 984

| File | Status | Notes |
|---|---|---|
| `production/qa/team-qa-sprint-14-2026-05-16.md` | NEW (this file) | Sprint 14 Team-QA report; matches `team-qa-sprint-N-YYYY-MM-DD.md` naming convention (precedent: `team-qa-sprint-10-2026-05-11.md`, `team-qa-sprint-11-2026-05-13.md`, `team-qa-sprint-12-2026-05-14.md`). |

Explicitly NOT touched (forbidden by PROMPT 984 scope):

- `client/`, `server/`, `shared/`, `tests/` -- no production or test code edits.
- `production/sprint-status.yaml` -- no row flips, no top-level edits, no `sprint_14_*` block edits.
- `production/stage.txt` -- no stage advance.
- `production/sprints/sprint-14.md` -- no sprint plan edits.
- `production/qa/qa-plan-sprint-14.md` -- no QA plan edits.
- Any Sprint 14 story file -- no story edits, no row flips, no `/story-done` invocation.
- `production/gate-checks/gate-polish-release-2026-05-12.md` -- no gate retry.
- `production/session-state/active.md`, `production/session-state/codex-orchestrator-state.md` -- PROMPT 984 allowed-edits permit session-state banner only if QA checkpoint convention requires it; precedent (Sprint 11 + Sprint 12 team-qa reports under PROMPT 793 + PROMPT 816) did not modify session-state when their Team-QA reports landed, so this Sprint 14 Team-QA also declines to touch session-state.
- `.claude/settings.json`, `.octogent/`, `.claude/scheduled_tasks.lock`.

Root-checkout dirt at `D:\_DEV\Work\Claude-Code-Game-Studios` was stashed under `PROMPT-984-pre-qa-stash` before this branch was created and was NOT staged, unstaged, deleted, modified, or relied on by PROMPT 984.

---

## Next Recommended Step

**Sprint 14 close-out disposition may launch next** (separate prompt). The close-out prompt should:

1. Mirror the PROMPT 894 (Sprint 13 closeout) + PROMPT 817 (Sprint 12 closeout) pattern: paperwork-only, dedicated worktree off `origin/main@f94f489` (or later if smoke evidence integration lands first).
2. Reach `closed-with-conditions` disposition (NOT `closed`), with `S11-HUD-TIMER-EYEBALL-VISUAL-001` explicitly carried into Sprint 15 planning as a human-operator-blocked Sprint 13 -> 14 -> 15 carry.
3. Optionally integrate the PROMPT 983 smoke evidence file (`production/qa/smoke-sprint-14-2026-05-16-rerun.md`) to `origin/main` first via a small paperwork-only integration prompt, so the close-out commits reference smoke evidence that is fully on `main`. This is a hygiene step, not a Sprint 14 close-out blocker per se -- the Team-QA accepts the smoke evidence on the worker branch as authoritative.
4. Preserve all carry conditions verbatim: `S8-QA-001-W1` OPEN, `QA-COND-0005` accepted-risk, `QA-COND-0006` accepted-risk, `PAW-TD-*-a` accept-risk, `TQ-S12-C1..C7` verbatim, PROMPT 683-era runtime divergence question, PROMPT 761 Polish->Release FAIL with NO retry, Sprint 12 story 019 `closed-with-conditions / cannot-reproduce`, all prior sprint closeouts.
5. Not advance stage from Polish; not claim public release / RC readiness, full game completion, broad accessibility, playtest validation, full playable-client manual QA, two-client GAME_OVER closure, or final-art completion.

---

## Final Status Line

984: SPRINT-14-TEAM-QA: APPROVED-WITH-CONDITIONS
