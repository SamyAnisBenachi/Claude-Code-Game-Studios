# PROMPT 982 QA Workflow Coverage Evidence

Date: 2026-05-16
Branch: work/prompt-982-qa-workflow-review
Worktree: D:\_DEV\claude-code-game-studios-worktrees\prompt-982-qa-workflow-review
Base inspected: origin/main at c385682d3da02eafa9f7cf5061b8b1eaa4788ab9

## Workflow Inspected

The review inspected the automated and evidence coverage for a two-client friend game:
lobby create/join, class confirm, draft shop, auction, placement submit, resolution,
reconnect, disconnect, game over, and result acknowledgement.

Covered by automated headless tests:

- Lobby/class confirm/session entry: `playable_client_real_e2e_loop_test`.
- Draft shop, auction bid/settlement, non-empty placement submit, resolution, and next loop:
  `playable_client_real_e2e_loop_test`.
- Planned objective route to game over and `C2SAcknowledgeResult` handshake:
  `playable_client_full_game_over_route_test`.
- Game-over reconnect result resend and cleanup rejection path:
  `game_over_reconnect_result_resend_test`.
- Mid-game reconnect snapshot ordering and deferred message flush:
  `reconnect_snapshot_test`.
- Disconnect grace, reconnect reset, mutual disconnect draw, and mid-resolution deferral:
  `rsm_disconnect_test`.
- UI-level result screen, return-to-lobby local intent, connection-lost overlay, and hand
  placement/drag core behavior: client test bins registered in `client/Cargo.toml`.

Covered by prior committed evidence:

- `production/qa/evidence/playable-client-real-e2e-loop.md` records a real in-process
  two-client Lightyear route through draft, shop, auction, placement, resolution, and
  return to the next loop. It does not claim game-over or result acknowledgement.
- `production/qa/evidence/sprint-13-two-client-runtime-evidence.md` records
  production WebSocket runtime harness runs reaching `endpoint_reached = "game_over"`
  for both clients, but it explicitly does not close `S8-QA-001-W1`.
- `production/qa/evidence/manual-friend-game-evidence-runbook.md` defines the manual
  native/browser friend-game route and keeps the required no-closure guardrail.

## Missing Coverage And False-Positive Risks

1. No single automated smoke currently proves the whole friend-game route in one run:
   two real clients, lobby/class confirm, auction, placement drag/submit, resolution,
   reconnect, disconnect, game over, and result acknowledgement.
2. The strongest automated route tests are headless Bevy/Lightyear app tests. They do not
   prove browser/WASM rendering, focus, pointer drag, two-window behavior, or screenshots.
3. The manual friend-game route is still evidence-only/open. No committed screenshot/video
   evidence proves a complete user-operated two-client game through Return to Lobby.
4. `tools/two-client-runtime/src/main.rs` currently treats both `"game_over"` and
   `"max_rounds"` as success. That can make a smoke run look green without proving the
   GameOver endpoint.
5. `tools/two-client-runtime/src/route.rs` uses empty scripted placements, so the runtime
   harness does not prove user-like planned-objective placement or drag-to-board behavior.
6. `docs/setup/two-client-runtime-harness.md` still says the harness does not reach
   `S2CGameOver` by default, while the committed Sprint 13 evidence shows seed-1 runs
   reaching `endpoint_reached = "game_over"`. The stale sentence can mislead release QA.
7. The full game-over route writes `target/test-evidence/e2e-game-over/run.log`; target
   evidence is useful locally but is not committed unless copied into `production/qa/evidence`.

## Test/Evidence Added

Added `workflow_route_coverage_tests_stay_registered_and_unignored` to
`tests/integration/session/result_acknowledgement_contract_test.rs`.

The test is intentionally static and low-risk. It fails if the critical workflow bins are
removed from the Cargo manifests, if the key playable-client/session/disconnect route tests
are marked `#[ignore]`, or if the manual QA runbook loses its result acknowledgement and
no-closure guardrails.

## Commands Run

All Cargo commands used:

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
