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
cargo test -p server rsm_disconnect
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

cargo fmt -p server -- --check
passed

cargo check -p server
passed
```
