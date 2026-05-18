# Smoke Check Report: Sprint 16 (Polish stage)

**Date**: 2026-05-18
**Sprint**: Sprint 16 (Polish stage -- `active`; deferred-backlog closeout sprint)
**Engine**: Bevy 0.18 + Lightyear 0.26
**QA Plan**: `production/qa/qa-plan-sprint-16.md`
**Prompt**: PROMPT 1075 -- Sprint 16 Smoke Check
**Source of truth (origin/main HEAD)**: `f8eac30d98af1ad21ed3ca6dd06e219ce9f9df19` (`story-done(s16): close card-slot primitive row (PROMPT 1074)`)
**Smoke worktree**: `D:/_DEV/claude-code-game-studios-worktrees/sprint-16-smoke-check-1075`
**Worker branch**: `qa/sprint-16-smoke-check-1075` (created off `f8eac30`)
**Smoke worktree HEAD at entry**: `f8eac30d98af1ad21ed3ca6dd06e219ce9f9df19` (matches origin/main)
**Smoke environment**: Windows 11 / D: drive. 802 GB free at smoke entry; 788 GB free at smoke exit. No disk-pressure cleanup invoked.

---

## Verdict: PASS-WITH-WARNINGS

The full workspace builds cleanly, formatting is clean, and the cargo workspace test aggregate is **1464 passed / 0 failed / 0 ignored / 0 measured / 0 filtered across 223 binaries**.

- `cargo fmt --all -- --check` -- PASS (exit 0, no output)
- `cargo check --workspace --all-targets` -- PASS (`Finished dev profile [optimized] target(s) in 1m 36s`, zero warnings, zero errors)
- `cargo test --workspace --tests --no-fail-fast` -- PASS (223 binaries / 1464 / 0 / 0 / 0 / 0)
- `cargo test -p shared --test spawn_range_live_refresh_contract -- --nocapture` -- 5/5 PASS (renamed Sprint 16 target per PROMPT 1072 closure; no AppCompat block on this run)
- `git diff --check` -- empty (no whitespace defects)
- `git diff --cached --check` -- empty (no staged changes at smoke entry)

The single warning is **environmental, not a code regression**:

- The mandated Cargo target dir (`D:\_DEV\cargo-target\ccgs-msvc`) was contended at smoke entry by live game processes that this smoke check is not authorized to terminate (`server.exe` PID 26084, `client.exe` PIDs 31420 + 32524, all running off the shared cargo target tree). The first `cargo test --workspace` attempt against the mandated dir failed with `error: failed to remove file D:\_DEV\cargo-target\ccgs-msvc\debug\server.exe` / `client.exe` (`Caused by: Access is denied. (os error 5)`) when cargo tried to relink the bin targets.
- PROMPT 1075 applied a Windows-host environmental workaround analogous in spirit to the PROMPT 983 AppCompat rename workaround: a sibling Cargo target dir `D:\_DEV\cargo-target\ccgs-msvc-smoke-1075` was used, preserving every other Cargo resource policy field verbatim (`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`). See [Cargo Target Dir Contention Workaround](#cargo-target-dir-contention-workaround) for the full rationale and command tape.
- This is **not** a code regression, **not** a Sprint 16 deliverable defect, **not** a test failure, and **not** authored by any PROMPT 1067 / 1068 / 1069 / 1070 / 1071 / 1072 / 1073 / 1074 code change. It is contention with externally-running game binaries on the developer host.

Per `/smoke-check` verdict rules: **PASS** if the suite ran cleanly; **PASS-WITH-WARNINGS** when results are clean but a known, documented, owner-identifiable environmental issue required a workaround. This smoke matches the latter.

This report makes **no claim** of (preserved non-claims, verbatim):

- no public release readiness
- no release-candidate readiness
- no full game completion
- no broad / Standard-tier accessibility completion (`QA-COND-0005` unchanged)
- no playtest / fun-hypothesis validation (`QA-COND-0006` unchanged)
- no full playable-client manual QA (`S8-QA-001-W1` OPEN, unchanged)
- no two-client `GAME_OVER` closure
- no final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved)
- no `Polish->Release` gate-check retry (PROMPT 761 `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`)
- no stage advance from `Polish`
- no Sprint 16 close-out
- no closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have, human-operator-blocked; carried-condition row; remains `status: ready` and `human_operator_blocked: true` per the 2026-05-17 orchestrator decision)

PROMPT 1075 did **not** run `/gate-check`, `/team-qa`, `/release-check`, `/story-done`, `/dev-story`, `/qa-plan`, or `/story-readiness`. PROMPT 1075 did **not** issue any QA sign-off, did **not** close Sprint 16, did **not** advance stage from `Polish`, did **not** flip any `sprint-status.yaml` row, did **not** edit any story file, did **not** edit `production/stage.txt`, did **not** edit any Sprint 16 sprint plan or QA plan file, did **not** edit `production/session-state/active.md` or `production/session-state/codex-orchestrator-state.md` (the immediate Sprint 14 smoke precedent PROMPT 983 declined to touch session-state, and this Sprint 16 smoke follows that precedent).

---

## Preflight

| Step | Result |
|---|---|
| `git fetch origin` | OK (no output) |
| Worktree HEAD (`f8eac30`) == `origin/main` | yes (verified twice -- before and after `git fetch origin`) |
| `git status --short --branch` (smoke worktree) | clean -- on `qa/sprint-16-smoke-check-1075`, no modifications, no untracked files |
| D: free space at smoke entry | **802 GB free** (well above 40 GB stop-threshold; no cleanup needed) |
| D: free space at smoke exit | **788 GB free** (~14 GB used by the sibling smoke target dir) |
| `production/stage.txt` (verified) | `Polish` (unchanged) |
| Sprint 16 disposition (verified) | `active`; 3 of 4 active rows `done`; only `S11-HUD-TIMER-EYEBALL-VISUAL-001` remains `status: ready` (Must Have, human-operator-blocked) |
| `CARGO_TARGET_DIR` (effective) | `D:\_DEV\cargo-target\ccgs-msvc-smoke-1075` (sibling of the mandated `ccgs-msvc` dir; see [Cargo Target Dir Contention Workaround](#cargo-target-dir-contention-workaround)) |
| `CARGO_PROFILE_DEV_DEBUG` | `0` (per policy) |
| `CARGO_PROFILE_TEST_DEBUG` | `0` (per policy) |
| `CARGO_INCREMENTAL` | `0` (per policy) |
| `RUSTFLAGS` | `-C debuginfo=0 -C link-arg=/DEBUG:NONE` (per policy) |
| Subagents spawned | none (serialized worker per prompt) |

---

## Cargo Resource Policy (applied: PARTIAL)

| Field | Mandated value | Applied value | Status |
|---|---|---|---|
| `CARGO_TARGET_DIR` | `D:\_DEV\cargo-target\ccgs-msvc` | `D:\_DEV\cargo-target\ccgs-msvc-smoke-1075` | **DEVIATED** -- environmental workaround for process-locked artifacts; see [Cargo Target Dir Contention Workaround](#cargo-target-dir-contention-workaround) |
| `CARGO_PROFILE_DEV_DEBUG` | `0` | `0` | PASS |
| `CARGO_PROFILE_TEST_DEBUG` | `0` | `0` | PASS |
| `CARGO_INCREMENTAL` | `0` | `0` | PASS |
| `RUSTFLAGS` | `-C debuginfo=0 -C link-arg=/DEBUG:NONE` | `-C debuginfo=0 -C link-arg=/DEBUG:NONE` | PASS |

No PDB/debuginfo emitted; all linker outputs link with `/DEBUG:NONE`. No incremental compilation. Sibling target dir is still under the `D:\_DEV\cargo-target\` shared parent and respects the disk-pressure intent of the policy. Disk usage delta: ~14 GB (802 GB free at entry, 788 GB at exit). No `Remove-Item -LiteralPath` cleanup invoked; not authorized at this free-space level.

---

## Commands and Results

### `git fetch origin`

PASS (no output). `origin/main` confirmed at `f8eac30d98af1ad21ed3ca6dd06e219ce9f9df19` (PROMPT 1074 tip).

### `git status --short --branch` (smoke worktree)

```
## qa/sprint-16-smoke-check-1075
```

Clean worktree on the dedicated smoke branch. No staged or unstaged modifications; no untracked files.

### `cargo fmt --all -- --check`

**Result**: PASS (exit 0, no output). Zero formatting drift on the workspace.

### `cargo check --workspace --all-targets`

**Result**: PASS. `Finished dev profile [optimized] target(s) in 1m 36s`. Zero compilation errors, **zero warnings** (the pre-existing `count_with_image_node` dead-code warning from PROMPT 983 / Sprint 14 was removed by PROMPT 1069 worker + PROMPT 1070 integration as part of `S15-TD-WORKSPACE-DEAD-CODE-WARNING-001`; cargo check is now warning-clean on origin/main HEAD `f8eac30`).

### `cargo test --workspace --tests --no-fail-fast`

**Result**: PASS. All 223 test binaries built and ran cleanly. Aggregate:

| Metric | Count |
|---|---|
| passed | **1464** |
| failed | **0** |
| ignored | **0** |
| measured | 0 |
| filtered out | 0 |
| binaries with emitted `test result: ok.` line | 223 |

Notable parity vs prior smokes:

- Sprint 11 (PROMPT 790): 1129 passed / 0 failed / 5 ignored across 189 binaries.
- Sprint 12 (PROMPT 815): 1135 passed / 0 failed / 0 ignored.
- Sprint 14 rerun (PROMPT 983): 1350 passed (cargo aggregate) + 5 (AppCompat-renamed direct run) = 1355 effective passed across 214 effective binaries; AppCompat OS error 740 environmental warning on `spawn_range_live_update_contract-*.exe`.
- **Sprint 16 (PROMPT 1075): 1464 passed / 0 failed / 0 ignored across 223 binaries; no AppCompat block (PROMPT 1072 rename closed `S15-OPS-APPCOMPAT-MANIFEST-001`; binary now ships as `spawn_range_live_refresh_contract-*.exe` and clears the installer-detection substring heuristic).**

Net delta vs Sprint 14 rerun: +9 effective binaries, +109 passed tests, -5 ignored → 0 ignored, AppCompat warning resolved. All-green improvement across the Sprint 14 → 15 → 16 closure path.

### `cargo test -p shared --test spawn_range_live_refresh_contract -- --nocapture`

**Result**: PASS -- 5/5. Confirmation that PROMPT 1072 Mechanism (d) Cargo `[[test]]` name rename works on this Windows host -- the renamed binary launches without UAC / AppCompat interference:

```
Running ..\tests\unit\protocol\spawn_range_live_update_contract_test.rs (D:\_DEV\cargo-target\ccgs-msvc-smoke-1075\debug\deps\spawn_range_live_refresh_contract-dcf81efb462e9aa5.exe)

running 5 tests
test test_spawn_range_is_not_registered_as_standalone_protocol_message ... ok
test test_s2c_resolution_event_remains_registered_on_reliable_channel ... ok
test test_spawn_range_changed_is_ordered_after_fake_objective_destroyed_in_same_batch ... ok
test test_spawn_range_changed_schema_round_trips_through_resolution_batch ... ok
test test_player_snapshot_spawn_range_cells_remains_public_recovery_field ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Note: the test source file path is still `tests/unit/protocol/spawn_range_live_update_contract_test.rs` (source not renamed per PROMPT 1072 design); only the Cargo `[[test]]` name -- and therefore the emitted binary name -- was renamed to drop the `update` substring. The `update` substring still appears in the source path string `cargo` prints, but the spawned `.exe` filename is `spawn_range_live_refresh_contract-*.exe`, which is the file Windows AppCompat scans.

### `git diff --check`

PASS (empty, exit 0). No whitespace defects in working tree.

### `git diff --cached --check`

PASS (empty, exit 0). No staged changes at smoke entry.

---

## Cargo Target Dir Contention Workaround

At smoke entry, `tasklist` showed the following processes holding artifacts under the mandated `D:\_DEV\cargo-target\ccgs-msvc` tree:

```
server.exe                   26084 Console                    1     31,532 K
client.exe                   31420 Console                    1    487,296 K
client.exe                   32524 Console                    1    486,404 K
```

(One additional `client.exe` PID 37564 was observed briefly during preflight and exited on its own before the workaround was applied.)

The first `cargo test --workspace --tests --no-fail-fast` attempt against the mandated `D:\_DEV\cargo-target\ccgs-msvc` failed at the link stage:

```
error: failed to remove file `D:\_DEV\cargo-target\ccgs-msvc\debug\server.exe`

Caused by:
  Access is denied. (os error 5)
warning: build failed, waiting for other jobs to finish...
error: failed to remove file `D:\_DEV\cargo-target\ccgs-msvc\debug\client.exe`

Caused by:
  Access is denied. (os error 5)
```

PROMPT 1075 scope explicitly forbids killing user / orchestrator-owned processes. The `D:\_DEV\cargo-target\ccgs-msvc` cleanup authorization in the prompt is gated on D: free space < 40 GB (the host has 802 GB free) and is for stale artifacts -- not for live-locked binaries owned by other developer activity.

Workaround applied: PROMPT 1075 set `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc-smoke-1075` (sibling of the mandated dir under the same `D:\_DEV\cargo-target\` parent) for the cargo invocations. Every other Cargo resource policy field was preserved verbatim. The cargo aggregate ran cleanly in the sibling tree (1464 / 0 / 0 / 0 / 0 across 223 binaries; spawn_range_live_refresh_contract 5/5 PASS). At smoke exit only `server.exe` PID 26084 remained running; the workaround consumed ~14 GB of D: (788 GB free at exit, well clear of the 40 GB threshold).

Classification: **environment / host-state-only warning** -- not a code regression, not a product-code regression, not authored by Sprint 16 closure prompts (PROMPT 1067 / 1068 / 1069 / 1070 / 1071 / 1072 / 1073 / 1074). The contention is from externally-running game binaries; it would clear immediately if the orchestrator or developer closed `server.exe` PID 26084 (and any subsequent client/server spawns).

Forward-looking options (**not** invoked or recommended for closure by PROMPT 1075 scope -- left to producer/orchestrator judgment):

1. **Per-run workaround (used here)**: sibling target dir under `D:\_DEV\cargo-target\` parent. No process termination; no policy-spirit violation (same RUSTFLAGS / no-PDB / no-incremental / disk-pressure-friendly parent). Disk cost: ~14 GB per smoke run.
2. **Pre-smoke gate**: the orchestrator (or a one-shot housekeeping prompt) closes any in-flight `client.exe` / `server.exe` instances before kicking off a smoke. Restores adherence to the literal `CARGO_TARGET_DIR` policy value.
3. **Persistent sibling target dir**: add `D:\_DEV\cargo-target\ccgs-msvc-smoke` (no per-prompt suffix) so successive smokes reuse the same tree and amortize disk cost. Would still preserve all resource fields. Backlog candidate only.

PROMPT 1075 does **not** file any of the above as Sprint 16 active rows or as gate-blocking; the smoke completed PASS-WITH-WARNINGS via option 1.

---

## Failures and Blockers

- **0 test failures** in the cargo aggregate (1464 / 0 / 0 / 0 / 0 across 223 binaries).
- **0 build errors** (`cargo check --workspace --all-targets` PASS; `cargo test --workspace --tests --no-fail-fast` PASS in the sibling target dir).
- **0 formatting drift** (`cargo fmt --all -- --check` PASS).
- **0 whitespace defects** (`git diff --check` + `git diff --cached --check` both clean).
- **0 cargo warnings** (the prior Sprint 14 `count_with_image_node` dead-code warning was closed by PROMPT 1069 / PROMPT 1070).
- **0 AppCompat OS-740 blocks** (PROMPT 1072 Cargo `[[test]]` rename eliminated the `update` substring on the spawned binary; the renamed `spawn_range_live_refresh_contract-*.exe` launches cleanly on this Windows host with no UAC prompt).
- **0 ignored tests** (`#[ignore]`d count workspace-wide: **0**; consistent with the Sprint 12 baseline and the PROMPT 814 retirement work).
- **0 disk-space blockers** at any point in the smoke run (802 GB free at entry, 788 GB at exit; well above the 40 GB threshold).
- **1 environmental warning** documented in [Cargo Target Dir Contention Workaround](#cargo-target-dir-contention-workaround). **Not a code regression**; not blocking. Mitigated by sibling target dir.

---

## Ignored tests (documented)

**Total `#[ignore]`d in workspace**: **0**. Parity with Sprint 12 / Sprint 13 / Sprint 14 / Sprint 15 baseline retired in PROMPT 814.

---

## Sprint 16 disposition (preserved by PROMPT 1075)

- **Sprint 16**: `active` (Polish-stage; activated PROMPT 1064 on 2026-05-17 against `origin/main@6f9308c`). 3 of 4 active rows `done` after PROMPT 1074 closure (0/1 Must Have + 1/1 Should Have + 2/2 Nice to Have done; 1 Must Have row `S11-HUD-TIMER-EYEBALL-VISUAL-001` remains `status: ready` and `human_operator_blocked: true`). PROMPT 1075 did **not** edit `production/sprints/sprint-16.md`, did **not** edit `production/sprint-status.yaml`, did **not** edit `production/stage.txt`, did **not** modify `.claude/settings.json`, did **not** touch any story file, did **not** touch `production/qa/qa-plan-sprint-16.md`, did **not** touch session-state files.
- **`S11-HUD-TIMER-EYEBALL-VISUAL-001`**: remains `status: ready` + `human_operator_blocked: true`. **No LLM `/story-done` authorised**; closure remains gated on a human-operator screenshot capture session across DraftInitial 45s / DraftShop 30s / Placement 10-12s phases per the 2026-05-17 orchestrator decision. Row is allowed to carry forward as a Sprint 16 → Sprint 17 carry if not closed by a human-operator slot during Sprint 16. PROMPT 1075 explicitly does NOT close this row, does NOT advance it, does NOT block on it (per the row's own `no_llm_story_done` field declaring that this row must not block non-human Sprint 16 development lanes).
- **Stage**: `Polish`. PROMPT 761 Polish->Release gate-check `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`. No retry attempted by PROMPT 1075.
- **Sprint 15 close-out**: `closed-with-conditions` per PROMPT 1056 -- preserved unchanged.
- **Sprint 14 close-out**: `closed-with-conditions` -- preserved unchanged.
- **Sprint 13 close-out**: `closed-with-conditions` -- preserved unchanged.
- **Sprint 12 close-out**: `closed-with-conditions` -- preserved unchanged.
- **Sprint 11 close-out**: `closed-with-conditions` -- preserved unchanged.
- **Sprint 10 close-out**: `closed-with-conditions` -- preserved unchanged.
- **Sprint 16 close-out**: **NOT** performed by this prompt. Smoke is one of several gates required before close-out; Team-QA, gate-check, and Sprint 16 close-out remain pending in separate prompts after S11 is either human-closed or formally carried to Sprint 17.

---

## Files changed by PROMPT 1075

- `production/qa/smoke-sprint-16-2026-05-18.md` (this file -- NEW)
- `reports/PROMPT-1075-Sprint-16-Smoke-Check.md` (mandatory final report; tracked separately under `reports/`)

Explicitly **not** touched (forbidden by PROMPT 1075 scope):

- `client/`, `server/`, `shared/`, `tests/`
- `Cargo.toml`, `Cargo.lock`, `.cargo/`
- `production/sprint-status.yaml`
- `production/stage.txt`
- `production/sprints/sprint-16.md`
- `production/qa/qa-plan-sprint-16.md`
- any prior sprint plan, QA plan, smoke evidence, team-QA, or close-out artifact
- `production/gate-checks/gate-polish-release-2026-05-12.md`
- any Sprint 16 story file (including `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`)
- `production/session-state/active.md` and `production/session-state/codex-orchestrator-state.md` (smoke checkpoint follows the PROMPT 983 precedent and does not write a session-state banner; existing Sprint 16 banners are preserved verbatim)
- `.claude/settings.json`, `.octogent/`, `.claude/scheduled_tasks.lock`
- WASM bundle build outputs, deployment configs
- the contended `D:\_DEV\cargo-target\ccgs-msvc` target dir (not modified, not cleaned)

Root-checkout dirt at `D:\_DEV\Work\Claude-Code-Game-Studios` was **not** touched, staged, unstaged, deleted, or relied on by this smoke check. The smoke worktree was a freshly created clean checkout of `origin/main@f8eac30` on its own branch `qa/sprint-16-smoke-check-1075`.

---

## Verification commands (for re-run)

```bash
git -C D:/_DEV/Work/Claude-Code-Game-Studios fetch origin
git -C D:/_DEV/Work/Claude-Code-Game-Studios rev-parse origin/main   # expect f8eac30d98af1ad21ed3ca6dd06e219ce9f9df19

git worktree add -b qa/sprint-16-smoke-check-1075 \
  D:/_DEV/claude-code-game-studios-worktrees/sprint-16-smoke-check-1075 \
  f8eac30d98af1ad21ed3ca6dd06e219ce9f9df19

cd D:/_DEV/claude-code-game-studios-worktrees/sprint-16-smoke-check-1075

# Cargo policy (sibling target dir workaround for live process contention)
export CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc-smoke-1075'
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_INCREMENTAL=0
export RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'

df -h /d                                                              # expect >40 GB free

cargo fmt --all -- --check                                            # expect exit 0, no output
cargo check --workspace --all-targets                                 # expect Finished dev profile, 0 warnings
cargo test --workspace --tests --no-fail-fast                         # expect 223 binaries / 1464 / 0 / 0 / 0 / 0
cargo test -p shared --test spawn_range_live_refresh_contract -- --nocapture   # expect 5/5
git diff --check                                                      # expect empty
git diff --cached --check                                             # expect empty
```

If the mandated `D:\_DEV\cargo-target\ccgs-msvc` is **not** under live-process contention (no `server.exe` / `client.exe` PIDs holding `debug/server.exe` or `debug/client.exe`), the standard literal policy path may be used instead:

```bash
export CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
```

---

## Cross-references

- Sprint 16 plan: `production/sprints/sprint-16.md`
- Sprint 16 QA plan: `production/qa/qa-plan-sprint-16.md`
- Sprint 16 activation: PROMPT 1064 / 2026-05-17
- Sprint 16 row closures:
  - PROMPT 1072 -- `S15-OPS-APPCOMPAT-MANIFEST-001` + `S15-TD-WORKSPACE-DEAD-CODE-WARNING-001` /story-done (origin/main@c9b5716)
  - PROMPT 1074 -- `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` /story-done (origin/main@f8eac30)
- Prior smokes:
  - Sprint 14 rerun (PROMPT 983, PASS-WITH-WARNINGS / AppCompat env warning): `production/qa/smoke-sprint-14-2026-05-16-rerun.md`
  - Sprint 12 (PROMPT 815, PASS-WITH-WARNINGS): `production/qa/smoke-sprint-12-2026-05-14.md`
  - Sprint 11 (PROMPT 790, PASS-WITH-WARNINGS): `production/qa/smoke-sprint-11-2026-05-13.md`
- Polish->Release gate FAIL (preserved): `production/gate-checks/gate-polish-release-2026-05-12.md`
- Sprint status: `production/sprint-status.yaml`
- Stage: `production/stage.txt` (`Polish`)

---

## Next recommended step

**PROMPT 1076 (or successor)** -- after this smoke evidence lands on `origin/main`, the eligible next actions are (producer judgment; PROMPT 1075 does NOT prescribe a single path):

1. **Team-QA sign-off for Sprint 16** -- the smoke gate is now green; a `/team-qa` Sprint 16 prompt may proceed against this smoke HEAD once Sprint 16 non-human rows are confirmed done (they are, after PROMPT 1072 + PROMPT 1074).
2. **Sprint 16 close-out planning** -- Sprint 16 can be closed-with-conditions when (a) Team-QA signs off, (b) the human-operator-blocked `S11-HUD-TIMER-EYEBALL-VISUAL-001` is either human-closed or formally carried to Sprint 17, and (c) carry-conditions (S8-QA-001-W1, QA-COND-0005, QA-COND-0006, PAW-TD-*-a, PROMPT 761 Polish->Release FAIL) are restated verbatim.
3. **Cargo target dir contention root-cause prompt** (Nice-to-Have only) -- a short housekeeping prompt could close the open `server.exe` / `client.exe` instances on the developer host before subsequent smokes, restoring literal `CARGO_TARGET_DIR` policy adherence.

PROMPT 1075 explicitly defers all of `/team-qa`, `/gate-check`, `/release-check`, `/story-done`, `/dev-story`, `/qa-plan`, and Sprint-16 close-out to later prompts. This smoke does **not** advance stage from `Polish`, does **not** retry Polish->Release, does **not** close Sprint 16, does **not** close `S11-HUD-TIMER-EYEBALL-VISUAL-001`, and does **not** claim release-readiness, broad accessibility, playtest validation, two-client GAME_OVER closure, or final-art completion.

---

**End of report -- PROMPT 1075 / Sprint 16 smoke verdict: PASS-WITH-WARNINGS (environment/host-state-only Cargo target dir contention warning; 1464 passed / 0 failed / 0 ignored across 223 binaries; no code regression).**
