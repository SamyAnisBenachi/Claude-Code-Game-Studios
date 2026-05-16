# PROMPT 982 QA Workflow Review

Date: 2026-05-16
Worker branch: `work/prompt-982-qa-workflow-review`
Worktree: `D:\_DEV\claude-code-game-studios-worktrees\prompt-982-qa-workflow-review`
Base: `origin/main` at `c385682d3da02eafa9f7cf5061b8b1eaa4788ab9`

## Workflow Simulated Or Inspected

Reviewed the automated, harness, and evidence coverage for the real friend-game route:
two clients, lobby/class confirm, auction, placement drag/submit, resolution, reconnect,
disconnect, game over, and result acknowledgement.

Automated coverage inspected and re-run:

- `playable_client_real_e2e_loop_test`: real in-process Lightyear two-client route through
  lobby/class confirm, draft/shop, auction, non-empty placement submit, resolution, and next loop.
- `playable_client_full_game_over_route_test`: planned-objective route through GameOver and
  `C2SAcknowledgeResult`.
- `game_over_reconnect_result_resend_test`: retained GameOver snapshot/result resend and cleanup
  rejection path after result cleanup.
- `rsm_disconnect_test`: disconnect grace, reconnect reset, mutual disconnect draw, auction abort,
  and mid-resolution deferral.
- Existing client UI bins in `client/Cargo.toml`: result screen, Return to Lobby local intent,
  reconnect rebuild, connection-lost overlay, and hand placement/drag core behavior.

Prior evidence inspected:

- `production/qa/evidence/playable-client-real-e2e-loop.md`
- `production/qa/evidence/sprint-13-two-client-runtime-evidence.md`
- `production/qa/evidence/sprint-9-manual-game-over-evidence.md`
- `production/qa/evidence/sprint-9-result-evidence-index.md`
- `production/qa/evidence/manual-friend-game-evidence-runbook.md`
- `docs/setup/two-client-runtime-harness.md`
- `tools/two-client-runtime/src/main.rs`
- `tools/two-client-runtime/src/route.rs`

## Missing Coverage Found

1. The route is covered in slices, not one complete automated smoke. There is no single test/harness
   that exercises two real clients through lobby/class confirm, auction, placement drag/submit,
   resolution, reconnect, disconnect, GameOver, and result acknowledgement.
2. The strongest automated coverage is headless/in-process. It proves protocol and ECS behavior, but
   not browser/WASM rendering, two-window focus, real pointer drag, screenshots, or visual result ack.
3. Manual friend-game closure evidence remains open. The Sprint 9 evidence explicitly says the full
   GUI route was not executed, and the runbook still defers closure to a future captured run.
4. The runtime harness can produce a false green for endpoint coverage because
   `tools/two-client-runtime/src/main.rs` treats both `"game_over"` and `"max_rounds"` as success.
5. The runtime harness is not user-like for placement. `tools/two-client-runtime/src/route.rs`
   scripts empty placements, so it does not prove planned-objective non-empty placement or drag intent.
6. The runtime harness docs are stale. `docs/setup/two-client-runtime-harness.md` says the harness
   does not reach `S2CGameOver` by default, while Sprint 13 runtime evidence records seed-1 runs
   reaching `endpoint_reached = "game_over"`.
7. Some useful evidence is ephemeral. The full game-over route writes under `target/test-evidence`,
   which disappears from committed evidence unless copied into `production/qa/evidence`.

No production gameplay/server defect was proven in this worker. I did not patch production
client/server/shared code.

## Tests And Evidence Added

Added `workflow_route_coverage_tests_stay_registered_and_unignored` in
`tests/integration/session/result_acknowledgement_contract_test.rs`.

This test hardens coverage against regressions that previously could make QA look green while
critical workflow routes were not actually exercised:

- Ensures the key server workflow bins stay registered in `server/Cargo.toml`.
- Ensures the key client UI workflow bins stay registered in `client/Cargo.toml`.
- Ensures the playable-client/session/disconnect route tests are not changed to `#[ignore]`.
- Ensures the manual friend-game runbook retains the result acknowledgement requirement and the
  explicit no-closure guardrail for `S8-QA-001-W1`.
- Ensures the two-client runtime harness doc retains the no-closure guardrail.

Added evidence summary:

- `production/qa/evidence/prompt-982-qa-workflow-coverage.md`

## Follow-Up Prompts

These are QA/harness follow-ups, not production-code patches.

### FOLLOW-UP 1 - Runtime Harness Endpoint Hardening

PROMPT TBD -- Harden Two-Client Runtime GameOver Smoke

Repo: `D:\_DEV\Work\Claude-Code-Game-Studios`
Mode: isolated worktree, QA/harness only.
Owned files: `tools/two-client-runtime/**`, `docs/setup/two-client-runtime-harness.md`,
`production/qa/evidence/**`, and test-only helpers under `tests/integration/playable_client/**`.

Task:

- Split the runtime harness success modes so the canonical GameOver smoke fails if
  `endpoint_reached = "max_rounds"` unless an explicit exploratory flag permits max-round cutoff.
- Port planned-objective non-empty placement scripting from
  `playable_client_full_game_over_route_test` into the runtime harness route, or add an equivalent
  test-only route that proves non-empty placement before GameOver.
- Persist a committed evidence artifact with the final state JSON and command output whenever the
  canonical smoke is used as QA evidence.
- Update `docs/setup/two-client-runtime-harness.md` so the default GameOver behavior matches the
  current seed-1 evidence, and keep the `S8-QA-001-W1` no-closure caveat.

Verification:

- Run the hardened GameOver smoke and prove that `endpoint_reached = "game_over"` is required.
- Run one exploratory max-round command and prove it is opt-in, not the canonical pass condition.
- Run `git diff --check`.

### FOLLOW-UP 2 - Manual Friend-Game Evidence Capture

PROMPT TBD -- Capture Real Two-Client Friend-Game QA Evidence

Repo: `D:\_DEV\Work\Claude-Code-Game-Studios`
Mode: QA evidence only, no production code edits.
Owned files: `production/qa/evidence/**` and a report under `reports/**`.

Task:

- Execute `production/qa/evidence/manual-friend-game-evidence-runbook.md` with two real clients
  against a local server.
- Capture screenshots or video for lobby/class confirm, auction, placement drag/submit, resolution,
  reconnect or refresh recovery, disconnect/lost-connection behavior, GameOver, result screen, and
  Return to Lobby/result acknowledgement.
- Record server/client logs, exact commands, seed/config, build hash, defects, and whether the route
  is sufficient to close or continue `S8-QA-001-W1`.

Verification:

- Attach/copy all evidence under `production/qa/evidence/**`.
- Run `git diff --check`.

## Commands Run And Results

```text
git fetch origin
```

Result: PASS after approval for network/git metadata access.

```text
git worktree add -b work/prompt-982-qa-workflow-review D:\_DEV\claude-code-game-studios-worktrees\prompt-982-qa-workflow-review origin/main
```

Result: PASS. Worktree created at `c385682d3da02eafa9f7cf5061b8b1eaa4788ab9`.

```text
C:\Users\Sam\.cargo\bin\cargo.exe fmt --all
```

Result: PASS.

```text
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'; $env:CARGO_PROFILE_DEV_DEBUG='0'; $env:CARGO_PROFILE_TEST_DEBUG='0'; $env:CARGO_INCREMENTAL='0'; $env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'; C:\Users\Sam\.cargo\bin\cargo.exe test -p server --test result_acknowledgement_contract_test workflow_route_coverage_tests_stay_registered_and_unignored
```

Result: PASS, 1 passed, 0 failed.

```text
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'; $env:CARGO_PROFILE_DEV_DEBUG='0'; $env:CARGO_PROFILE_TEST_DEBUG='0'; $env:CARGO_INCREMENTAL='0'; $env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'; C:\Users\Sam\.cargo\bin\cargo.exe test -p server --test result_acknowledgement_contract_test
```

Result: PASS, 6 passed, 0 failed.

```text
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'; $env:CARGO_PROFILE_DEV_DEBUG='0'; $env:CARGO_PROFILE_TEST_DEBUG='0'; $env:CARGO_INCREMENTAL='0'; $env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'; C:\Users\Sam\.cargo\bin\cargo.exe test -p server --test game_over_reconnect_result_resend_test
```

Result: PASS, 2 passed, 0 failed.

```text
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'; $env:CARGO_PROFILE_DEV_DEBUG='0'; $env:CARGO_PROFILE_TEST_DEBUG='0'; $env:CARGO_INCREMENTAL='0'; $env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'; C:\Users\Sam\.cargo\bin\cargo.exe test -p server --test rsm_disconnect_test
```

Result: PASS, 11 passed, 0 failed.

```text
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'; $env:CARGO_PROFILE_DEV_DEBUG='0'; $env:CARGO_PROFILE_TEST_DEBUG='0'; $env:CARGO_INCREMENTAL='0'; $env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'; C:\Users\Sam\.cargo\bin\cargo.exe test -p server --test playable_client_full_game_over_route_test full_game_over_route_including_acknowledgement_handshake
```

Result: PASS, 1 passed, 0 failed.

```text
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'; $env:CARGO_PROFILE_DEV_DEBUG='0'; $env:CARGO_PROFILE_TEST_DEBUG='0'; $env:CARGO_INCREMENTAL='0'; $env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'; C:\Users\Sam\.cargo\bin\cargo.exe test -p server --test playable_client_real_e2e_loop_test real_lightyear_two_client_draft_shop_auction_placement_resolution_reaches_next_loop
```

Result: PASS, 1 passed, 0 failed.

```text
git diff --check
```

Result: PASS.

```text
git diff --check --cached
```

Result: PASS.

## Branch, Commit, Push

Branch: `work/prompt-982-qa-workflow-review`
Commit hash: pending until this report is committed.
Push status: pending until verification, commit, and push complete.
