# Smoke Check Report: Sprint 14 (Polish / friend-game scope)

**Date**: 2026-05-16
**Sprint**: Sprint 14 (Polish stage; active)
**Engine**: Bevy 0.18 + Lightyear 0.26
**Prompt**: PROMPT 978 -- Sprint 14 Smoke Check
**QA Plan**: `production/qa/qa-plan-sprint-14.md`
**Sprint Plan**: `production/sprints/sprint-14.md`
**Audit Source**: `reports/PROMPT-977-sprint-14-remaining-work-audit.md` (local ignored report source)
**Source Commit**: `c385682d3da02eafa9f7cf5061b8b1eaa4788ab9` (`origin/main`, `story-done S14 HUD opponent figurine`)
**Smoke Worktree**: `D:\Tmp\ccgs-prompt-978-smoke`
**Branch**: `qa/sprint-14-smoke-check-978`

---

## Verdict: FAIL

Sprint 14 smoke did not pass. The full workspace test command reached test execution after clearing a stale Windows file lock, but two client test bins failed with real assertion failures:

1. `client --test shop_auction_ui_plugin_scaffold_formulas_test`
   - Failing test: `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes`
   - Assertion: `left == right` failed at `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs:43`
   - Observed: `left: 78`, `right: 71`

2. `client --test ui_clean_pass_z_layers_test`
   - Failing test: `ac7_production_migration_sites_reference_design_tokens`
   - Assertion: `ui/lobby.rs` must reference `z_layers::UI_BASE`
   - Observed: search returned no match

The shared test binary `spawn_range_live_update_contract` also hit the known Windows AppCompat elevation heuristic because the generated executable name contains `update`. This was verified as environmental: copying the same binary to `D:\Tmp\spawn_range_live_upd_contract_prompt978.exe` and running it directly passed all 5 tests.

---

## Preflight

| Step | Result |
|---|---|
| `git fetch origin` | PASS after elevated retry; sandbox could not open `.git/FETCH_HEAD` |
| Root checkout suitability | Not used; root was detached at `e3ed056` and dirty while `origin/main` was `c385682` |
| Fresh worktree | PASS: `D:\Tmp\ccgs-prompt-978-smoke` from `origin/main` |
| `git rev-parse HEAD` | `c385682d3da02eafa9f7cf5061b8b1eaa4788ab9` |
| `git rev-parse origin/main` | `c385682d3da02eafa9f7cf5061b8b1eaa4788ab9` |
| `git status --short` before Cargo | Clean |
| D: free space before Cargo | `900.47 GB` free of `1305.01 GB` |
| D: free space after checks | `844.78 GB` free of `1305.01 GB` |
| `CARGO_TARGET_DIR` | `D:\_DEV\cargo-target\ccgs-msvc` |
| No-PDB env | `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE` |
| Disk threshold | PASS: above 40 GB; no target cleanup invoked |

---

## Commands And Results

### `cargo fmt --all -- --check`

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
cargo fmt --all -- --check
```

Result: PASS, exit 0.

### `cargo check --workspace --all-targets`

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
cargo check --workspace --all-targets
```

Result: PASS, exit 0.

Warning observed:

- `client\..\tests\integration\presentation\hand_ui_asset_wiring_test.rs:43:4`: function `count_with_image_node` is never used.

### `cargo test --workspace --tests --no-fail-fast`

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
cargo test --workspace --tests --no-fail-fast
```

First attempt result: ENVIRONMENTAL BLOCK before test execution. Cargo failed to remove `D:\_DEV\cargo-target\ccgs-msvc\debug\server.exe` with `Access is denied. (os error 5)`. A stale `server.exe` process (`PID 17944`, start time `2026-05-15 02:16:42`) was found and stopped with approval.

Second attempt result: FAIL, exit 1.

Failed targets:

- `-p client --test shop_auction_ui_plugin_scaffold_formulas_test`
- `-p client --test ui_clean_pass_z_layers_test`
- `-p shared --test spawn_range_live_update_contract` (Windows AppCompat spawn block; direct renamed run passed, see below)

### Targeted Failure Reruns

```powershell
cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test -- --nocapture
```

Result: FAIL. `7 passed; 1 failed; 0 ignored`.

Failing test:

- `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes`
- `assertion left == right failed`
- `left: 78`
- `right: 71`

```powershell
cargo test -p client --test ui_clean_pass_z_layers_test -- --nocapture
```

Result: FAIL. `5 passed; 1 failed; 0 ignored`.

Failing test:

- `ac7_production_migration_sites_reference_design_tokens`
- `AC7 production migration site ui/lobby.rs must reference design-token constant z_layers::UI_BASE`

### Windows AppCompat Workaround

```powershell
$bin = Get-ChildItem -LiteralPath 'D:\_DEV\cargo-target\ccgs-msvc\debug\deps' -Filter 'spawn_range_live_update_contract-*.exe' |
  Sort-Object LastWriteTime -Descending |
  Select-Object -First 1
$dest = 'D:\Tmp\spawn_range_live_upd_contract_prompt978.exe'
Copy-Item -LiteralPath $bin.FullName -Destination $dest -Force
& $dest
```

Result: PASS. `5 passed; 0 failed; 0 ignored`.

This preserves the prior Sprint 12 / Sprint 13 AppCompat diagnosis: the generated binary name, not the test body, triggers the Windows elevation heuristic.

### Sprint 14 QA-Plan Targeted UI Tests

No additional Sprint 14 QA-plan Cargo test bin was outside the coverage of `cargo test --workspace --tests --no-fail-fast`. The full workspace command is explicitly the Sprint 14 end-of-sprint integration smoke command in the QA plan. Targeted reruns were limited to the two failed client bins listed above.

Manual / human-only Sprint 14 evidence was not run:

- `S11-HUD-TIMER-EYEBALL-VISUAL-001` remains human-operator-blocked and requires real two-client screenshots for `DraftInitial`, `DraftShop`, and `Placement`.
- Cross-story visual consistency review is a separate follow-up before Sprint 14 close-out, not part of this smoke prompt.

### `git diff --check`

```powershell
git diff --check
```

Result before report authoring: PASS, exit 0.

Result after report authoring: PASS, exit 0.

---

## Ignored Tests

No actual `#[ignore]` attributes were found in `client/`, `server/`, `shared/`, or `tests/`:

```powershell
rg -n "^\s*#\s*\[\s*ignore" client server shared tests
```

Result: no matches.

The AppCompat-blocked `spawn_range_live_update_contract` binary is not an ignored test; it is a Windows execution heuristic and the renamed binary passed all 5 tests.

---

## Warnings And Environmental Notes

- `cargo check --workspace --all-targets` and `cargo test --workspace --tests --no-fail-fast` both emitted the existing dead-code warning for `count_with_image_node` in `hand_ui_asset_wiring_test.rs`.
- The first full test attempt was blocked by a stale `server.exe` process locking `D:\_DEV\cargo-target\ccgs-msvc\debug\server.exe`; after stopping it, the command reached test execution.
- The `spawn_range_live_update_contract` generated test executable hit Windows AppCompat elevation (`os error 740`). The renamed direct run passed.
- No disk cleanup was performed. D: remained well above the 40 GB threshold.

---

## Preserved Non-Claims

PROMPT 978 did not run `/story-done`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `/dev-story`, or Sprint 14 close-out.

No claim is made for:

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion (`QA-COND-0005` remains accepted-risk)
- playtest / fun-hypothesis validation (`QA-COND-0006` remains accepted-risk / deferred)
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1` remains open)
- final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved)
- Polish-to-Release gate retry (PROMPT 761 `FAIL` preserved)
- stage advance from `Polish`
- Sprint 14 close-out
- closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`
- underlying drag-runtime bug fix from Sprint 12 story 019

---

## Files Changed By PROMPT 978

- `production/qa/smoke-sprint-14-2026-05-16.md` (this file; new smoke evidence)

Explicitly not changed:

- `client/`, `server/`, `shared/`, `tests/`
- `production/sprint-status.yaml`
- `production/session-state/active.md`
- `production/session-state/codex-orchestrator-state.md`
- `production/sprints/sprint-14.md`
- `production/qa/qa-plan-sprint-14.md`
- `production/stage.txt`
- story files
- gate-check, release, team-qa, or cross-story consistency artifacts

---

## Next Required Action

Route a fix prompt before Sprint 14 Team-QA:

- Update `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes` expectations or the underlying pre-pooled entity count, depending on intended post-Sprint-14 composition.
- Restore or intentionally update the `ui/lobby.rs` z-layer migration assertion that expects a `z_layers::UI_BASE` reference.

Do not proceed to Sprint 14 Team-QA or close-out from this smoke result.

---

**End of report -- PROMPT 978 / Sprint 14 smoke verdict: FAIL**
