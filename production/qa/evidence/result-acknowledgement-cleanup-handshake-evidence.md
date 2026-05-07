# Result Acknowledgement Cleanup Handshake Evidence

Date: 2026-05-07
Branch: `work/s9-rs-003-result-ack-cleanup-handshake`

## Scope Covered

- Result Screen Return to Lobby queues one `C2SAcknowledgeResult` attempt at most once per ended-session return.
- Duplicate Return to Lobby activations are idempotent.
- Local result-screen cache, visibility, and focus state are cleared before returning to `ClientState::Lobby`.
- Missing client `MessageSender<C2SAcknowledgeResult>` is treated as a disconnected/closing transport fallback: local cleanup and lobby navigation still complete.
- Return-to-lobby navigation does not overwrite the server-authored `CurrentClientPhase` view.
- Server all-ack cleanup and timeout cleanup continue to use the retained-result reconnect cleanup path compatible with S9-RS-001.

## Verification Run

- `cargo test -p client --test result_screen_return_to_lobby_test` - PASS, 2 passed.
- `cargo test -p server --test result_acknowledgement_cleanup_handshake_test` - PASS, 3 passed.
- `cargo test -p client --test result_screen_mvp_test` - PASS, 6 passed.
- `cargo test -p server --test result_acknowledgement_contract_test` - PASS, 5 passed.
- `cargo test -p server --test game_over_reconnect_result_resend_test` - PASS, 2 passed.
- `cargo check -p client` - PASS.
- `cargo check -p server` - PASS.
- `cargo fmt -p client -p server -- --check` - PASS.
- `git diff --check origin/main...HEAD` - PASS, no output.

## Not Claimed

- No `/story-done` run.
- No smoke test, QA sign-off, gate-check, sprint close-out, or full CI run.
- No manual browser route, manual native route, or end-to-end multiplayer route completion claimed.
