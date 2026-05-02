# RSM Story 006 Test Evidence

Command: `cargo test -p server --test rsm_network_dispatch_test`

Result: passed

```text
running 3 tests
test rsm_network_dispatch_sends_one_phase_change_per_broadcast ... ok
test rsm_network_dispatch_preserves_each_broadcast_payload_once ... ok
test rsm_resolution_safety_timeout_transitions_to_game_over ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Additional checks:

```text
cargo fmt -p server -- --check
passed

cargo test -p server --test rsm_disconnect_test
11 passed; 0 failed

cargo test -p server --test e2e_websocket_test e2e_websocket_heartbeat_roundtrip_and_reliable_channel
1 passed; 0 failed

cargo test -p server rsm_
passed; includes rsm_network_dispatch, rsm_disconnect, rsm_f2_ordering, rsm_timers, rsm_transitions, rsm_win_condition, and matching RSM-filtered tests

cargo check -p server
passed

Select-String MessageSender server/src/core/rsm
no matches

Select-String EventWriter|EventReader|Events<|add_event server/src/core/rsm
no matches
```
