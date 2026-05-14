# Team-QA Report: Sprint 12 (Polish / friend-game scope)

| Field | Value |
|---|---|
| **Date** | 2026-05-14 |
| **Sprint** | Sprint 12 — `active` (Polish stage; activated by PROMPT 798) |
| **Stage** | `Polish` (unchanged) |
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Scope** | Friend-game / Polish slice — explicitly NOT public release readiness |
| **Skill** | `/team-qa sprint` (qa-lead + qa-tester roles, fresh worktree) |
| **Prompt** | PROMPT 816 — Sprint 12 Team QA / QA Sign-Off |
| **Worktree** | `D:\_DEV\claude-code-game-studios-worktrees\sprint-12-team-qa` (NEW) |
| **Branch** | `qa/sprint-12-team-qa` (NEW; tracks `origin/main`) |
| **Commit Under Review (HEAD)** | `bce480239922117dc704ce2503320f12dbdd4c33` (`qa(s12): /smoke-check Sprint 12 PASS-WITH-WARNINGS (PROMPT 815)`) |
| **HEAD == origin/main** | yes (verified after `git fetch origin`) |
| **Root-checkout dirt** | preserved untouched per operating contract (` M .claude/settings.json`, staged `production/session-state/autonomous-monitor-task.md`, untracked `Dtmpworkspace-test-output.txt` — none staged, unstaged, deleted, modified, or relied on by PROMPT 816) |
| **Review mode** | Lean (no `production/review-mode.txt` override) |
| **Cargo policy applied** | **N/A** — no `cargo` command was invoked by PROMPT 816 (paperwork-only, review-of-record on existing PROMPT 815 smoke evidence). |

---

## Verdict: APPROVED WITH CONDITIONS

Sprint 12 Must Have scope is complete (5 / 5 `done` on `origin/main@bce4802`). The Sprint 12 smoke check (PROMPT 815) is `PASS-WITH-WARNINGS` with the single warning classified as an environmental Windows AppCompat false positive (no code regression — see §"Smoke Warning Classification" below). The workspace ignored-test count has dropped from the Sprint 11 close-out baseline of **5** to **0** — all five retained Cluster B D-5 `#[ignore]` tests are retired by Sprint 12 Must Have stories 012 / 013 / 014 / 015 under documented decision-first dispositions. The story 019 tighter-capture diagnostic is closed `closed-with-conditions / cannot-reproduce after second time-box exhaustion`; the underlying drag-runtime bug is **NOT claimed fixed** and is escalated to PROMPT 804 Sprint 13 candidate runtime-hardening stories per the PROMPT 807 evidence-file mapping. Carried conditions (`S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, placeholder-art `PAW-TD-*-a` accept-risk, HUD-timer eyeball check W2, PROMPT 761 Polish→Release `FAIL`) are preserved unchanged.

This Team-QA report makes **no claim** of (preserved non-claims):

- no public release readiness
- no release-candidate readiness
- no full game completion
- no broad / Standard-tier accessibility completion (`QA-COND-0005` unchanged)
- no playtest / fun-hypothesis validation (`QA-COND-0006` unchanged)
- no full playable-client manual QA (`S8-QA-001-W1` unchanged)
- no two-client GAME_OVER closure
- no final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved)
- no Polish→Release retry — PROMPT 761 Polish→Release gate-check `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`
- no Sprint 12 close-out (this Team-QA sign-off is a precondition to a separate close-out decision, not the close-out itself)
- no `S8-QA-001-W1` closure
- no stage advance from Polish
- no underlying drag-runtime bug fix (story 019 closed `cannot-reproduce`, not `bug-fixed`)

PROMPT 816 did **not** run `/dev-story`, `/smoke-check`, `/gate-check`, `/release-check`, `/story-done`, `/story-readiness`, or `/qa-plan`. PROMPT 816 did **not** modify production code under `client/`, `server/`, `shared/`, or `tests/`. PROMPT 816 did **not** modify `production/sprint-status.yaml`, `production/sprints/sprint-12.md`, `production/sprints/sprint-11.md`, `production/stage.txt`, `production/gate-checks/`, `production/qa/qa-plan-sprint-12.md`, `production/qa/qa-plan-sprint-11.md`, `production/qa/smoke-sprint-12-2026-05-14.md`, `production/qa/smoke-sprint-11-2026-05-13.md`, `production/qa/team-qa-sprint-11-2026-05-13.md`, any Sprint 12 story file, any Sprint 12 evidence file, `.claude/settings.json`, `.octogent/`, `.claude/scheduled_tasks.lock`, or `reports/` (other than the mandatory `reports/PROMPT-816-*.md` final-report file, which is gitignored and NOT staged or committed).

---

## Verification (Preflight)

| Step | Result |
|---|---|
| `git fetch origin` | OK |
| `git rev-parse HEAD` (Team-QA worktree) | `bce480239922117dc704ce2503320f12dbdd4c33` |
| `git rev-parse origin/main` | `bce480239922117dc704ce2503320f12dbdd4c33` (matches HEAD) |
| `git status --short` (Team-QA worktree) | clean — no modifications, no untracked files |
| `production/stage.txt` | `Polish` (unchanged) |
| `production/sprint-status.yaml` sprint row | `sprint: 12`, `status: active`, `stage: Polish` |
| PROMPT 761 Polish→Release gate-check FAIL evidence | preserved at `production/gate-checks/gate-polish-release-2026-05-12.md` |
| Sprint 12 smoke evidence | exists at `production/qa/smoke-sprint-12-2026-05-14.md` — verdict `PASS-WITH-WARNINGS` |
| Sprint 12 QA plan | exists at `production/qa/qa-plan-sprint-12.md` (PROMPT 799) |
| Workspace `#[ignore]` count (rg over `*.rs`) | **0** (verified — see §"Workspace Ignored Count Verification") |
| Root-checkout dirt at `D:\_DEV\Work\Claude-Code-Game-Studios` | preserved untouched (no stage, unstage, delete, modify, or read by PROMPT 816) |

---

## Sprint 12 Must Have Completion (5 / 5 `done`)

Verified by reading `production/sprint-status.yaml` `stories:` block and each story file. Each Must Have row carries `status: done`, `completed: "2026-05-14"` (with disposition note), and a `/story-done` verdict landed via PROMPT 814.

| ID | Cluster | Title | Status | Closed | Integration commits on `origin/main` | `/story-done` prompt | Evidence |
|---|---|---|---|---|---|---|---|
| S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001 (story 012) | B2 | HudPlugin snapshot.phase → CurrentClientPhase bridge fixture gap | **done** | 2026-05-14 | `c1eef10` (PROMPT 806 worker; PROMPT 809 integration verify) | PROMPT 814 | `production/qa/evidence/sprint-12-fixture-hud-snapshot-phase-bridge-evidence.md` |
| S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001 (story 013) | B3 | Lobby ConfirmClass intent not emitted alongside SelectClass | **done** | 2026-05-14 | `d8d0196` (PROMPT 801 worker `7c07329` cherry-picked by PROMPT 805) | PROMPT 814 | `production/qa/evidence/sprint-12-lobby-confirm-class-intent-chain-evidence.md` |
| S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001 (story 014) | B4 | co_occupancy_offset no longer panics on index 2 — binary design decision | **done** | 2026-05-14 | lineage `d5053fe` decision → `ae6635d` test rewrite → `1c3f760` evidence → fast-forward to `d8d0196` (PROMPT 800 worker; PROMPT 805 integration) | PROMPT 814 | `production/qa/evidence/sprint-12-cooccupancy-panic-guard-evidence.md` |
| S11-TD-FIXTURE-D-RESIDUALS-001 (story 015) | B1 + B5 umbrella | Board ghost producer fixture (B1) + ShopAuctionUiEntity count drift (B5) | **done** | 2026-05-14 | `0bfdd76` (decision-record) + `a3c624e` (un-`#[ignore]` of both tests) (PROMPT 812 worker; PROMPT 813 integration) | PROMPT 814 | `production/qa/evidence/sprint-12-fixture-d-residuals-evidence.md` |
| S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001 (story 019) | — | Drag-and-drop runtime divergence — tighter-capture diagnostic-only retest | **done** (closed-with-conditions / cannot-reproduce — second time-box exhaustion) | 2026-05-14 | `c2a08a6` (evidence) + `a8ef42d` (Sprint 13 escalation mapping) (PROMPT 807 worker; PROMPT 810 integration) | PROMPT 814 | `production/qa/evidence/sprint-11-drag-runtime-evidence-tighter.md` |

**Must Have closure**: **5 / 5 done.**

Should Have rows (`S11-HUD-TIMER-EYEBALL-VISUAL-001`, `S11-HU-PHASE-IDEMPOTENCY-001`, `S11-SERVER-POOL-INIT-LOG-GUARD-001`, `S11-LOBBY-UX-CONFIRM-STATE-001`) and Nice to Have rows (`S11-TD-CARGO-DISK-USAGE-001`, `S11-TD-CARGO-PDB-LIMIT-001`, `S11-OPS-ORCHESTRATOR-LOCK-001`, `S11-OPS-GH-CLI-001`, `S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001`) remain `blocked` — no story files authored, `/story-readiness` pending. This is consistent with the Sprint 12 plan's stated friend-game / Polish scope-cap and is not a regression.

---

## Per-Story Evidence Review

### Story 012 (B2) — HudPlugin snapshot.phase bridge

Evidence file `production/qa/evidence/sprint-12-fixture-hud-snapshot-phase-bridge-evidence.md`:

- **Decision recorded BEFORE code change**: Path B (relocate assertion) chosen. Four-point rationale captured in story file's "Design Decision" section; coverage-gap verification confirmed Path B does NOT create a coverage gap (existing HUD-side test `reconnect_snapshot_rebuild_test.rs::full_snapshot_rebuild_populates_all_hud_zones_without_respawning_entities` already exercises the `snapshot.phase → CurrentClientPhase` bridge).
- **Test diff**: removed `#[ignore]` + misplaced `CurrentClientPhase` assertion from `tests/integration/board_rendering/snapshot_spawn_test.rs:39`; authored new dedicated `tests/integration/hud/snapshot_phase_bridge_test.rs::test_hud_plugin_bridges_snapshot_phase_and_round_into_current_client_phase` mirroring the `app_with_hud_in_session()` fixture pattern.
- **Production diff**: empty (`git diff origin/main...HEAD -- 'server/src/**' 'client/src/**' 'shared/src/**'` = empty per evidence AC5 audit).
- **ADR conformance**: ADR-002 (Client-Server Authority), ADR-009 (RSM Phase State), ADR-021 (Presentation Layer Architecture / single shared phase sink) all preserved. No optimistic client-side phase authority introduced.
- **AC1–AC8**: all satisfied.

**Verdict**: ✅ acceptable for sign-off.

### Story 013 (B3) — Lobby ConfirmClass intent chain

Evidence file `production/qa/evidence/sprint-12-lobby-confirm-class-intent-chain-evidence.md`:

- **Investigation note recorded**: traces the two-button chain through `lobby_button_interaction_system` (client/src/ui/lobby.rs:465) and `send_lobby_commands_system` (line 510). The PROMPT 750 D-5 owner comment was a misdiagnosis — the input chain does NOT stop at `SelectClass`; the test failed because the test fixture did not simulate the `S2CJoinAck` round-trip and therefore the production `session_id.is_none()` gate at line 792 correctly suppressed `ConfirmClass`.
- **Disposition chosen**: **fallback path** (test redesign + production gate preserved). The production `session_id.is_none()` gate at `client/src/ui/lobby.rs:792` is **binding per ADR-002** and is preserved unchanged. The test fixture is updated to mirror the production S2CJoinAck round-trip.
- **Production diff**: empty (zero `client/src/` change). Test fixture diff scoped to `tests/integration/playable_client/native_operator_controls_test.rs` with an inline comment referencing ADR-002 and the production gate.
- **Protocol verification**: `C2SConfirmClass` shape verified present at `shared/src/protocol.rs:431` and registered on the reliable channel at `shared/src/protocol.rs:62`.
- **Test asserts both intents end-to-end**: `cargo test -p client --test playable_client_native_operator_controls_test` reports `5 passed; 0 failed; 0 ignored`. The previously-ignored test now passes.
- **No-optimism audit**: grep confirms `lobby.locked_class` is written only inside `apply_class_locked` (S2C consumer at `client/src/ui/lobby.rs:392`). No new optimistic write site introduced.
- **ADR conformance**: ADR-002 (Client-Server Authority), ADR-008 (Lightyear channel config), ADR-012 (Session-ready delivery) all preserved.
- **AC1–AC9**: all satisfied.

**Verdict**: ✅ acceptable for sign-off.

### Story 014 (B4) — Co-occupancy panic-guard decision

Evidence file `production/qa/evidence/sprint-12-cooccupancy-panic-guard-evidence.md`:

- **Decision recorded BEFORE code change**: Path B (test rewritten to assert non-panic clamp behaviour) chosen. Path A explicitly marked NOT chosen.
- **Decision-recording commit precedes code-change commit (AC1, AC7)**: `git log` evidence shows `d5053fe` (Wave 1, decision) **before** `ae6635d` (Wave 2, code change / test rewrite). The `#[should_panic]` invariant was **NOT silently deleted** — Path B rationale was committed first.
- **Five-point Path B rationale captured**: (a) upstream caller invariant via `u8` parameter type at `snapshot_co_occupancy_offsets:1888-1927`; (b) post-clamp return `co_occupancy_offset(2, 8.0) == 4.0`; (c) `warn!` diagnostic + ADR-021 non-fatal degradation; (d) ADR-021 alignment; (e) historical disposition referencing commit `ac9305b` (2026-05-08 observer refactor + clamp).
- **Test diff**: removed `#[ignore]` + `#[should_panic]` from `tests/unit/board_rendering/status_icons_test.rs:167`; renamed `test_cooccupancy_index_two_panics_with_offending_index` → `test_cooccupancy_index_two_clamps_to_second_slot_offset`; new assertions lock the canonical 2-slot layout and the >=2 clamp invariant.
- **Production diff**: empty — Path B leaves `co_occupancy_offset` (and all of `client/src/presentation/board_rendering.rs`) unchanged.
- **Test pass count**: targeted `cargo test -p client --test board_rendering_status_icons_test --no-fail-fast` reports `5 passed; 0 failed; 0 ignored`. Workspace-level: `1130 passed / 0 failed / 4 ignored` post-Wave-2 (delta: +1 passing, −1 ignored — B4 row dropped).
- **ADR conformance**: ADR-002 (Client-Server Authority) and ADR-021 (Presentation Layer Architecture) preserved.
- **AC1–AC9**: all satisfied.

**Verdict**: ✅ acceptable for sign-off.

### Story 015 (B1 + B5 umbrella) — Fixture D residuals

Evidence file `production/qa/evidence/sprint-12-fixture-d-residuals-evidence.md`:

- **Producer decision recorded BEFORE code change**: **umbrella retained**. Split path NOT chosen. Five-point rationale captured (both sub-dispositions test-only or near-test-only; combined diff small; shared decision-before-code discipline; same Sprint 11 D-5 triage origin; reduces re-review cost).
- **B1 sub-disposition**: **Path B1.a chosen** (expand fixture / event-firing repair). Original PROMPT 750 D-5 owner comment corrected — the `GhostDragStartEvent` producer is NOT in `HandUiPlugin`; it is an observer on `BoardRenderingPlugin` (`add_observer(on_ghost_drag_start)` at `client/src/presentation/board_rendering.rs:893`). Fix: drive `Pointer<Press>` / `Pointer<Click>` via `world.trigger(event)` in the test body — matching `bevy_picking::DefaultPickingPlugins` real gameplay behaviour.
- **B5 sub-disposition**: **Path B5.a chosen** (update formula 57 → 66). Investigation established the 9-entity delta is intentional capacity, not over-production — `spawn_draft_initial_grid` at `client/src/ui/shop_auction/mod.rs:3654-3720` spawns three `ShopAuctionUiEntity`-tagged entities per draft slot (slot container, dedicated text-child to prevent white-dot rendering, BOUGHT overlay). Trimming back (Path B5.b) would re-introduce the white-dot regression. The formula multiplier `* 2` was the stale artefact.
- **Test diffs**: removed `#[ignore]` from both target tests; replaced `write_message` with `world.trigger(event)` in `ghost_preview_bridge_test.rs`; updated multiplier `* 2 → * 3` in `plugin_scaffold_formulas_test.rs`. Both test files retain explanatory comments referencing PROMPT 812 + the production site.
- **Production diff**: empty (`git diff origin/main...HEAD -- 'client/src/**' 'server/src/**' 'shared/src/**'` returns empty). Zero production-code change confirmed.
- **Test pass counts**: targeted B1 (`board_rendering_ghost_preview_bridge_test`) 4 / 0 / 0; targeted B5 (`shop_auction_ui_plugin_scaffold_formulas_test`) 8 / 0 / 0; client crate full `passed: 396 / failed: 0 / ignored: 0` (delta from PROMPT 810 baseline: +2 passing, -2 ignored); workspace `1135 passed / 0 failed / 0 ignored` (Sprint 11 close-out baseline was 5 ignored; PROMPT 805 closed B3 + B4, PROMPT 806/809/810 closed B2, this PROMPT 812 closes B1 + B5).
- **ADR conformance**: ADR-002 (Client-Server Authority) and ADR-021 (Presentation Layer Architecture) preserved.
- **AC1–AC10**: all satisfied.

**Verdict**: ✅ acceptable for sign-off.

### Story 019 — Drag-runtime tighter-capture diagnostic

Evidence file `production/qa/evidence/sprint-11-drag-runtime-evidence-tighter.md`:

- **Disposition**: `cannot-reproduce` (**second time** since PROMPT 778 / story 018). Locked per story 019 §"Time-box" rule — when the 1.5-day operator-driven time-box cannot be exercised inside a non-interactive CLI dispatch, `cannot-reproduce` (second time) is the prescribed disposition.
- **Underlying drag-runtime bug NOT claimed fixed**. Truth-table locked as `NOT-OBSERVED` on every cell of the S1–S5 × {A, B, C, D} grid, with code-evidence pointers (static-code presence of each emit site) but **NOT** runtime-evidence pointers (no runtime trace captured).
- **Static-code re-verification** on the tighter-capture base (`origin/main@d8d0196`): all 5 S1–S5 instrumentation sites + 3 sibling `spawn_highlight_caller` callers present at the same file:line locations recorded in the parent (`d36bbbd`-era) evidence. No drift in the drag-runtime code area between `d36bbbd` → `d8d0196`.
- **Escalation preserved**: `HU-DRAG-RT-19-04` rule for second-time `cannot-reproduce` requires **no third same-scope retest** without expanded tracing. The evidence file recommends authoring a Sprint 13 expanded-tracing follow-on story (or threading through the three already-authored PROMPT 804 Sprint 13 candidates: `story-017-two-client-runtime-harness.md`, `story-018-obs-tracing-targets.md`, `story-019-obs-wallclock-timestamps.md` — playable-client epic). The mapping was appended by PROMPT 807 commit `a8ef42d`.
- **Recommended expanded-tracing scope (advisory)**: per-channel selective `lightyear` debug logging; persistent in-process millisecond-UTC tracing init; operator workflow runbook; no protocol-shape modification; no optimistic client-side authority; no `S8-QA-001-W1` closure claims.
- **`cargo test -p client` verification** during PROMPT 807 run: exit 0, 59 passed / 0 failed / 1 ignored (the 1 ignored is `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes` carried from `origin/main@d8d0196`; that B5 ignored test is subsequently retired by PROMPT 812 — story 015).
- **HU-DRAG-RT-19-01 → HU-DRAG-RT-19-08**: AC1, AC2 deferred → `cannot-reproduce` (second time); AC3, AC4, AC5, AC6, AC7, AC8 all satisfied. No production code change; no optimistic client-side authority introduced; all carried non-claims preserved.

**Verdict**: ✅ acceptable for sign-off as `closed-with-conditions / cannot-reproduce` — the disposition is honest and the escalation path is documented. Story 019 is **NOT** a `bug-fixed` disposition; the drag-runtime question remains carried for Sprint 13 expanded-tracing follow-on.

---

## Smoke Check Summary (PROMPT 815, `production/qa/smoke-sprint-12-2026-05-14.md`)

| Command | Result |
|---|---|
| `cargo fmt --check` | PASS (exit 0, no output) |
| `cargo check --workspace` | PASS (`Finished dev profile [optimized + debuginfo] target(s) in 1m 22s`) |
| `cargo test --workspace --tests --no-fail-fast` | cargo-aggregate `189 binaries / 1130 passed / 0 failed / 0 ignored`; **1 binary failed to spawn** (`spawn_range_live_update_contract-*.exe`, Windows AppCompat false positive). Direct-run of renamed binary: 5 passed / 0 failed / 0 ignored. **Functional total**: **1135 passed / 0 failed / 0 ignored** — exact parity with PROMPT 813 baseline at `a3c624e`. |
| `git diff --check` | empty (PASS) |
| `git diff --cached --check` | empty (PASS) |

Smoke verdict: **PASS-WITH-WARNINGS**. The single warning is classified below.

---

## Smoke Warning Classification

The PROMPT 815 smoke warning is the failure of `cargo test --workspace --tests --no-fail-fast` to spawn one test binary:

- **Binary**: `target\debug\deps\spawn_range_live_update_contract-<hash>.exe`
- **Source**: `tests/unit/protocol/spawn_range_live_update_contract_test.rs`
- **Cargo error**: `could not execute process ... The requested operation requires elevation. (os error 740)`

**Root cause** (per PROMPT 815 smoke evidence §"Windows AppCompat Workaround"):

- Windows Application Compatibility shim's installer-detection heuristic intercepts spawn of any executable whose filename contains the substrings `setup`, `install`, `update`, or `patch` and demands UAC elevation unless an embedded application manifest declares `level="asInvoker"`.
- Cargo-emitted rustc test binaries do not embed such a manifest.
- The test source file name (`spawn_range_live_update_contract_test.rs`) is named after the live-update protocol contract it validates; that string ends up in the cargo bin name verbatim.

**Classification**: **Windows / AppCompat environment warning** — **NOT a code regression**.

- The binary itself compiles, passes formatting checks, and the 5 tests inside pass when launched directly under a filename that does not contain the substring `update` (verified per smoke evidence: `5 passed; 0 failed; 0 ignored` when run as `D:\tmp\spawn_range_live_upd_contract.exe`).
- Functional total `1135 / 0 / 0` is at parity with the PROMPT 813 baseline; no behavioural drift in the test contents.
- The warning is host-environment-dependent: PROMPT 813's `cargo test` run on the same code did not trigger AppCompat (due to AppCompat shim's cache; the cache miss on a fresh build hash on the current run caused the intercept). Future smoke runs on a different host or after a different rebuild may or may not exhibit this; it is non-deterministic at the host level.

**Recommendation (advisory only, no immediate dev-tooling block)**: this could be filed as a Sprint 13 candidate Nice-to-Have item against the existing `S11-TD-CARGO-DISK-USAGE-001` / `S11-TD-CARGO-PDB-LIMIT-001` cluster (devops-engineer owned) — specifically a small follow-on `S12-DEVOPS-WINDOWS-APPCOMPAT-MANIFEST-001` (or equivalent slug) that documents the AppCompat heuristic in `docs/setup/dev-environment.md` and recommends either: (a) renaming `spawn_range_live_update_contract_test.rs` to a substring-clean equivalent (e.g., `spawn_range_live_upd_contract_test.rs` — would require care to update any cross-references), or (b) embedding a Windows manifest via `embed-resource` / `winres` in the test crate's `Cargo.toml`. **Neither is implemented or required by this Team-QA sign-off.**

This warning does **not** block Sprint 12 Team-QA approval. It does not block subsequent Sprint 12 close-out either. It surfaces a documented dev-tooling environmental quirk and a clear workaround.

---

## Workspace Ignored Count Verification

Cross-references:

- Sprint 11 close-out baseline (PROMPT 790 / `production/qa/smoke-sprint-11-2026-05-13.md`): **5 ignored** across 189 binaries — the 5 retained Cluster B `#[ignore]` tests (B1–B5).
- PROMPT 813 integration baseline at `origin/main@a3c624e`: **0 ignored** workspace-wide.
- PROMPT 815 smoke at `origin/main@7e55952` (then `bce4802` after the smoke evidence commit): cargo aggregate `0 ignored`; functional total `1135 / 0 / 0`.

Direct grep over `*.rs` files in this Team-QA worktree (`bce4802`):

```text
Grep pattern: #\[ignore
Path: D:/_DEV/claude-code-game-studios-worktrees/sprint-12-team-qa
Glob: *.rs
Result: No matches found.
```

**Confirmed**: **0** `#[ignore]` markers in any `.rs` file on `origin/main@bce4802`. The Sprint 11 close-out baseline of 5 retained Cluster B D-5 tests has been fully retired by the Sprint 12 Must Have stories. Workspace ignored count delta: **5 → 0** (−5).

---

## Cluster B D-5 Retirement Summary

| # | Test (binary :: name) | Sprint 12 story | Disposition | Outcome |
|---|---|---|---|---|
| B1 | `tests/integration/board_rendering/ghost_preview_bridge_test.rs :: br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui` | `S11-TD-FIXTURE-D-RESIDUALS-001` (story 015, umbrella) | Path B1.a — drive `Pointer<Press>` / `Pointer<Click>` via `world.trigger(event)` in test body (test-only; zero production change) | **retired** |
| B2 | `tests/integration/board_rendering/snapshot_spawn_test.rs :: test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives` | `S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001` (story 012) | Path B — relocate `snapshot.phase → CurrentClientPhase` assertion into dedicated HUD-side test (test-only; zero production change) | **retired** |
| B3 | `tests/integration/playable_client/native_operator_controls_test.rs :: test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands` | `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001` (story 013) | Fallback path — test fixture rewritten to simulate `S2CJoinAck` + `session_id` round-trip; production ADR-002 session-id gate preserved unchanged | **retired** |
| B4 | `tests/unit/board_rendering/status_icons_test.rs :: test_cooccupancy_index_two_panics_with_offending_index` | `S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001` (story 014) | Path B — test rewritten to assert clamp invariant; `#[should_panic]` removed **deliberately** (not silently — rationale commit `d5053fe` precedes code-change commit `ae6635d`); production `co_occupancy_offset` unchanged | **retired** |
| B5 | `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs :: shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes` | `S11-TD-FIXTURE-D-RESIDUALS-001` (story 015, umbrella) | Path B5.a — formula multiplier updated `* 2 → * 3` (57 → 66) to match the deliberate text-child entity in `spawn_draft_initial_grid`; rationale captured | **retired** |

**Total retired**: **5 / 5**. No new undocumented `#[ignore]` introduced. All five dispositions follow the decision-first discipline (rationale recorded BEFORE code change) per the Sprint 12 QA plan's hard constraint.

---

## Carried Conditions Table (preserved unchanged)

| Condition | Status before Sprint 12 | Status after Sprint 12 Team-QA | Notes |
|---|---|---|---|
| `S8-QA-001-W1` — manual / browser two-client GAME_OVER gap | OPEN | **OPEN (unchanged)** | No manual / browser two-client GAME_OVER route executed in Sprint 12. Sign-off does NOT close this. |
| `QA-COND-0005` — Standard-tier accessibility | accepted-risk / friend-game scope | **accepted-risk / friend-game scope (unchanged)** | Sprint 12 explicitly does not pursue Standard-tier accessibility completion. Sign-off does NOT close this. |
| `QA-COND-0006` — playtest / fun-hypothesis validation | accepted-risk / deferred | **accepted-risk / deferred (unchanged)** | Sprint 12 evidence is friend-game / fixture / paperwork / diagnostic; no playtest evidence. Sign-off does NOT close this. |
| 5 retained Cluster B D-5 `#[ignore]` tests (Sprint 11 close-out) | retained with owner-named follow-up slugs | **all 5 retired by Sprint 12 Must Have stories under documented decision-first dispositions** | Workspace ignored count: 5 → 0. None silently dropped. |
| HUD timer eyeball visual check (W2) | deferred | **deferred (unchanged)** | `S11-HUD-TIMER-EYEBALL-VISUAL-001` Should-Have row remains `blocked` (no story file authored). |
| Placeholder / friend-game art scope (`PAW-TD-*-a`) | accept-risk | **accept-risk (unchanged)** | Placeholder PNGs across PAW-002..PAW-006. No final-art / asset-production-completion claim. |
| PROMPT 683-era runtime divergence question | preserved unchanged (folded into story 019 tighter-capture) | **preserved unchanged** | Story 019 closed `cannot-reproduce` (second time); the PROMPT 683 question is **NOT** separately claimed closed. Escalated to Sprint 13 expanded-tracing follow-on. |
| PROMPT 761 Polish→Release gate-check | FAIL (0/13 required artefacts) | **FAIL preserved (unchanged)** | No retry attempted by PROMPT 816. Evidence preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`. |
| Sprint 10 disposition | `closed-with-conditions` (PROMPT 763) | **`closed-with-conditions` (unchanged)** | Recorded under `sprint_10_closeout:` in `production/sprint-status.yaml`. |
| Sprint 11 disposition | `closed-with-conditions` (PROMPT 792) | **`closed-with-conditions` (unchanged)** | Recorded under `sprint_11_closeout:` in `production/sprint-status.yaml`. |
| Underlying drag-runtime bug | OPEN diagnostic question (PROMPT 683 lineage) | **NOT claimed fixed; escalated to PROMPT 804 Sprint 13 candidate runtime-hardening stories** | Story 019 closed `cannot-reproduce`, not `bug-fixed`. Sprint 13 follow-on path documented. |

---

## QA Recommendation

**Ready for Sprint 12 close-out with conditions.**

Sprint 12 Must Have scope is complete on `origin/main@bce4802`, the workspace test suite passes cleanly at functional total `1135 / 0 / 0`, the workspace ignored count has dropped from the Sprint 11 close-out baseline of 5 to 0 (all 5 retained Cluster B D-5 tests retired under decision-first dispositions with rationale recorded BEFORE each code change), and the single PROMPT 815 smoke warning is classified as a Windows AppCompat environmental false positive (no code regression). The story 019 disposition is honest (`cannot-reproduce`, second time-box exhaustion) with the underlying drag-runtime bug explicitly NOT claimed fixed and the escalation path to Sprint 13 expanded-tracing candidates documented.

The friend-game / Polish acceptance criteria specified in `production/sprints/sprint-12.md` "Definition of Done for this Sprint" are met for the closure-relevant subset:

- [x] All Must Have tasks completed and integrated.
- [x] All Must Have tasks pass acceptance criteria (per `/story-done` verdicts at PROMPT 814 commit `7e55952`).
- [x] Sprint 12 QA plan exists at `production/qa/qa-plan-sprint-12.md` (PROMPT 799).
- [x] All Logic/Integration stories have passing unit/integration tests (workspace `1135 passed / 0 failed`).
- [x] `cargo test -p server` and `cargo test -p client` pass without regression vs. Sprint 11 close-out baseline (1129 → 1135 = +6 tests un-`#[ignore]`d; 0 failures).
- [x] Workspace ignored count drops by ≥1 without introducing new undocumented `#[ignore]` markers (5 → 0 = −5; AC exceeded).
- [x] `/smoke-check sprint` recorded as `PASS-WITH-WARNINGS` with documented warnings (`production/qa/smoke-sprint-12-2026-05-14.md`); the warning is documented Windows AppCompat false positive.
- [x] `/team-qa sprint` produced this `APPROVED WITH CONDITIONS` sign-off report.
- [x] No S1 or S2 bugs in delivered Must Have features (verified against per-story evidence; story 019 `cannot-reproduce` is a documented diagnostic outcome, not a regression).
- [x] `production/sprint-status.yaml` reflects every Must Have story as `done` (or, for story 019, `done` with `closed-with-conditions` disposition note).
- [x] `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006` retain their pre-Sprint-12 disposition.
- [x] `production/stage.txt` remains `Polish`.
- [x] No public release readiness, release-candidate readiness, full playable-client manual QA, full game completion, broad Standard-tier accessibility completion, playtest / fun-hypothesis validation, or full asset / content production is claimed.

**Conditions attached to this approval** (mirroring Sprint 10 / Sprint 11 sign-off pattern):

- **TQ-S12-C1** — Sprint 12 close-out is a separate orchestrator decision. This Team-QA report does NOT close Sprint 12. Closing requires either (a) pulling each Should Have / Nice to Have row into active scope and closing it, or (b) explicit deferral with a written reason captured in `sprint_12_closeout:` paperwork analogous to `sprint_11_closeout:`.
- **TQ-S12-C2** — Story 019 underlying drag-runtime bug remains an OPEN diagnostic question. The Sprint 13 expanded-tracing escalation path (via `story-017-two-client-runtime-harness.md`, `story-018-obs-tracing-targets.md`, `story-019-obs-wallclock-timestamps.md` — PROMPT 804 candidates — OR via a new `production/epics/hand-ui/story-NNN-drag-runtime-expanded-tracing.md`) MUST be authored before any further drag-runtime claim. **No third same-scope retest** is authorised under this sign-off.
- **TQ-S12-C3** — `S8-QA-001-W1` remains OPEN. Sign-off does NOT include manual / browser two-client GAME_OVER evidence. Any future release-scope claim must address it separately.
- **TQ-S12-C4** — `QA-COND-0005` (Standard-tier accessibility) and `QA-COND-0006` (playtest / fun-hypothesis validation) remain accepted-risk / deferred. Sign-off does NOT include accessibility or playtest evidence.
- **TQ-S12-C5** — PROMPT 761 `Polish→Release` gate-check `FAIL` remains preserved. Do NOT retry the Polish→Release gate-check until release-scope artefacts (final art, manual-QA sign-off, accessibility completion, playtest evidence) actually exist on `main`. Sign-off does NOT authorise a retry.
- **TQ-S12-C6** — Placeholder / friend-game art scope (`PAW-TD-*-a`) remains accept-risk. Sign-off does NOT include final-art or asset-production-completion evidence.
- **TQ-S12-C7** — Windows AppCompat smoke warning is informational. If the warning persists on subsequent smoke runs across different hosts, recommend filing a small Sprint 13 candidate (devops-engineer owned) under `docs/setup/dev-environment.md` documenting the AppCompat heuristic and a manifest/rename workaround. This is **NOT a blocker** for Sprint 12 close-out or any release claim.

**Not ready for**: Polish→Release advancement, public release readiness, release-candidate readiness, full game completion, broad accessibility completion, playtest / fun-hypothesis validation, full playable-client manual QA, final-art / asset-production completion, `S8-QA-001-W1` closure, or underlying drag-runtime bug-fix claim.

---

## Checks Run (read-only by PROMPT 816)

- `git fetch origin` — OK.
- `git rev-parse HEAD` vs `git rev-parse origin/main` — both `bce4802...`; matches.
- `git status --short` (Team-QA worktree) — clean.
- `git log --oneline -10` — recent commits include `bce4802 qa(s12): /smoke-check Sprint 12 PASS-WITH-WARNINGS (PROMPT 815)`, `7e55952 qa(s12): /story-done batch for 5 Sprint 12 Must Have rows (PROMPT 814)`, `a8ef42d qa(s11/s12): map drag-runtime escalation to PROMPT 804 Sprint 13 candidates (PROMPT 807)`, `c2a08a6 qa(s11/s12): author tighter-capture drag-runtime retest evidence (PROMPT 807)`, `c1eef10 dev(s12-b2): Path B relocate snapshot.phase HUD bridge assertion (PROMPT 806)`.
- Read `production/sprint-status.yaml` — Sprint 12 row `status: active`, `stage: Polish`; 5/5 Must Have rows `status: done` with completion notes and integration commit references.
- Read `production/sprints/sprint-12.md` — Definition-of-Done items mapped above.
- Read `production/qa/qa-plan-sprint-12.md` — Sprint 12 QA plan exists; story-file gate table consistent with current state (all 5 Must Have rows EXISTS / `/story-readiness` PASSED).
- Read `production/qa/smoke-sprint-12-2026-05-14.md` — verdict `PASS-WITH-WARNINGS`; aggregates `189 binaries / 1130 passed / 0 failed / 0 ignored`; functional total `1135 / 0 / 0`.
- Read `production/qa/evidence/sprint-12-fixture-hud-snapshot-phase-bridge-evidence.md` — Path B chosen; AC1–AC8 satisfied; zero production diff.
- Read `production/qa/evidence/sprint-12-lobby-confirm-class-intent-chain-evidence.md` — fallback path; AC1–AC9 satisfied; ADR-002 / ADR-008 / ADR-012 preserved.
- Read `production/qa/evidence/sprint-12-cooccupancy-panic-guard-evidence.md` — Path B; rationale commit `d5053fe` precedes code change `ae6635d`; `#[should_panic]` removed deliberately, not silently; AC1–AC9 satisfied.
- Read `production/qa/evidence/sprint-12-fixture-d-residuals-evidence.md` — umbrella retained; B1.a + B5.a sub-dispositions; AC1–AC10 satisfied; zero production diff.
- Read `production/qa/evidence/sprint-11-drag-runtime-evidence-tighter.md` — `cannot-reproduce` (second time); static-code re-verification of 5 S1–S5 sites; Sprint 13 expanded-tracing escalation documented; AC1–AC8 with AC1/AC2 deferred per time-box rule.
- Read `production/gate-checks/gate-polish-release-2026-05-12.md` — `FAIL` verdict preserved.
- Read `production/stage.txt` — `Polish` (unchanged).
- Read `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md` — Status header confirms `Done -- closed by PROMPT 814 ... closed-with-conditions / evidence-captured-cannot-reproduce ... underlying drag-runtime bug is NOT claimed fixed; the regression is escalated to the PROMPT 804 Sprint 13 candidate runtime-hardening stories`.
- Grep over `*.rs` for `#[ignore` markers — 0 matches workspace-wide.
- No `cargo` commands executed by PROMPT 816. Smoke results cited authoritatively from PROMPT 815 evidence file.

---

## Files Changed by PROMPT 816

- `production/qa/team-qa-sprint-12-2026-05-14.md` (this file — NEW)
- `production/session-state/active.md` (banner prepended — paperwork only, no state mutation beyond the banner)
- `production/session-state/codex-orchestrator-state.md` (PROMPT 816 disposition section prepended)
- `reports/PROMPT-816-Sprint-12-Team-QA.md` (mandatory final report file; NOT staged or committed — `reports/` is gitignored)

Explicitly **not** touched by PROMPT 816:

- `.claude/settings.json` (root-checkout dirt preserved as-is, NOT staged, NOT committed)
- `client/`, `server/`, `shared/`, `tests/`
- `production/sprint-status.yaml`
- `production/stage.txt`
- `production/sprints/sprint-12.md`
- `production/sprints/sprint-11.md`
- `production/qa/qa-plan-sprint-12.md`
- `production/qa/qa-plan-sprint-11.md`
- `production/qa/smoke-sprint-12-2026-05-14.md`
- `production/qa/smoke-sprint-11-2026-05-13.md`
- `production/qa/team-qa-sprint-11-2026-05-13.md`
- any Sprint 12 story file (`story-012` / `013` / `014` / `015` / `019`)
- any Sprint 12 evidence file
- `production/gate-checks/gate-polish-release-2026-05-12.md`
- `.octogent/`, `.claude/scheduled_tasks.lock`
- any `reports/` file other than `reports/PROMPT-816-Sprint-12-Team-QA.md`

Root-checkout dirt at `D:\_DEV\Work\Claude-Code-Game-Studios` (` M .claude/settings.json`, staged `production/session-state/autonomous-monitor-task.md`, untracked `Dtmpworkspace-test-output.txt`) was **not** touched, staged, unstaged, deleted, or relied on by PROMPT 816. The Team-QA worktree was a freshly created, clean checkout of `origin/main`.

---

## Cross-references

- Sprint 12 plan: `production/sprints/sprint-12.md`
- Sprint 12 QA plan: `production/qa/qa-plan-sprint-12.md`
- Sprint 12 smoke: `production/qa/smoke-sprint-12-2026-05-14.md` (PASS-WITH-WARNINGS)
- Sprint 12 evidence files:
  - `production/qa/evidence/sprint-12-fixture-hud-snapshot-phase-bridge-evidence.md` (story 012 / B2)
  - `production/qa/evidence/sprint-12-lobby-confirm-class-intent-chain-evidence.md` (story 013 / B3)
  - `production/qa/evidence/sprint-12-cooccupancy-panic-guard-evidence.md` (story 014 / B4)
  - `production/qa/evidence/sprint-12-fixture-d-residuals-evidence.md` (story 015 / B1 + B5 umbrella)
  - `production/qa/evidence/sprint-11-drag-runtime-evidence-tighter.md` (story 019)
- Sprint 12 story files (5 Must Have, all `Status: Done`):
  - `production/epics/playable-client/story-012-fixture-hud-snapshot-phase-bridge.md`
  - `production/epics/playable-client/story-013-lobby-confirm-class-intent-chain.md`
  - `production/epics/playable-client/story-014-cooccupancy-panic-guard-decision.md`
  - `production/epics/playable-client/story-015-fixture-d-residuals.md`
  - `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`
- Sprint 11 D-5 triage (baseline for Cluster B retirement): `production/qa/evidence/sprint-11-ignored-d5-triage.md`
- Sprint 11 Team-QA pattern reference: `production/qa/team-qa-sprint-11-2026-05-13.md`
- Sprint 10 Team-QA pattern reference: `production/qa/team-qa-sprint-10-2026-05-11.md`
- Polish→Release gate FAIL (preserved): `production/gate-checks/gate-polish-release-2026-05-12.md`
- Sprint status authoritative: `production/sprint-status.yaml`
- Stage authoritative: `production/stage.txt` (`Polish`)
- PROMPT 814 `/story-done` batch report: `reports/PROMPT-814-S11-Sprint-12-Must-Have-Story-Done-Batch.md`
- PROMPT 815 smoke report: `reports/PROMPT-815-Sprint-12-Smoke-Check.md`

---

## Next Recommended Step

**PROMPT 817 — Sprint 12 close-out paperwork** (separate orchestrator prompt). Run `/sprint-status` / close-out skill on a new worktree from latest `origin/main` after this PROMPT 816 Team-QA evidence lands. PROMPT 817 should:

- consume this `APPROVED WITH CONDITIONS` Team-QA verdict
- write a `sprint_12_closeout:` block into `production/sprint-status.yaml` mirroring the `sprint_11_closeout:` pattern (5 / 5 Must Have done; 0 / 4 Should Have done; 0 / 5 Nice to Have done — explicit deferral with written rationale into Sprint 13 planning)
- flip `production/sprint-status.yaml` top-level `status: active → closed-with-conditions` for Sprint 12
- **NOT** advance stage from Polish, **NOT** retry Polish→Release, **NOT** claim release-readiness, broad accessibility, playtest validation, or two-client GAME_OVER closure
- preserve all carried conditions verbatim (TQ-S12-C1 through TQ-S12-C7)

After PROMPT 817 Sprint 12 close-out lands, the next eligible Sprint 13 activation prompt may follow (with the PROMPT 804 candidate runtime-hardening stories as the natural focus).

---

**End of report — PROMPT 816 / Sprint 12 Team-QA verdict: APPROVED WITH CONDITIONS (ready for Sprint 12 close-out with conditions; NOT a release sign-off).**

Status line: `816: SPRINT-12-TEAM-QA: APPROVED-WITH-CONDITIONS`
