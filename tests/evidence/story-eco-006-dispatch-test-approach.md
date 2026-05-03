# ECO-006 Dispatch Test Approach

## Scope

ECO-006 verifies Economy internal Bevy messages are converted into shared
protocol S2C payloads with the correct Lightyear target semantics:

- `S2CGoldUpdate` is private and targets only the owning player's `PeerId`.
- `S2CGoldBroadcast` is public and targets `NetworkTarget::All`.
- Both wire messages are registered as reliable server-to-client protocol
  messages.

## Approach

The executable coverage lives in `tests/integration/economy/network_dispatch_test.rs`
and runs as `cargo test -p server --test economy_network_dispatch_test`.

The tests use `EconomyNetworkOutbox`, matching the existing RSM/objective
dispatch evidence pattern. The systems still call Lightyear's verified
`ServerMultiMessageSender::send::<M, ReliableChannel>` path when a live
`Server` and sender are present, but the headless tests do not require a live
WebSocket. This keeps the test deterministic and focused on dispatch routing,
payload conversion, and channel registration.

## Coverage

- Draft start end-to-end: `DraftStarted` -> `on_draft_started` -> economy
  dispatch records one private update per owner and one public broadcast per
  player.
- Queued award path: pre-enqueued economy messages dispatch private update and
  public broadcast payloads with `reserved_gold`.
- Missing connection: private update is skipped without panic; public broadcast
  still dispatches.
- Protocol manifest: shared gold wire messages are registered as reliable S2C.
