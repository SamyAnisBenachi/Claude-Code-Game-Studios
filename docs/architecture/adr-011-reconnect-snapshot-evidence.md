# ADR-011 Reconnect Snapshot Evidence

Date: 2026-05-03

## Lightyear 0.26 Checklist

Source evidence: `tests/evidence/lightyear-026-verification.md`.

| Item | Result | Evidence |
|---|---|---|
| 1. Reconnect uses a new transport identity | VERIFIED | Lightyear 0.26 uses a new `PeerId` per connection entity. |
| 2. Unicast target shape | VERIFIED | `NetworkTarget::Single(PeerId)` is the verified unicast form. |
| 3. Reliable enqueue order | VERIFIED | `ReliableChannel` is `OrderedReliable`; reconnect messages are enqueued in mandatory order. |
| 4. Connected signal shape | VERIFIED WITH API UPDATE | Use Bevy observer `On<Add, Connected>`, not legacy `OnConnected`. |
| 5. Old-connection messages do not deliver to the new identity | VERIFIED | New `PeerId` starts with an empty queue. |
| 6. Two-channel setup | VERIFIED | `ReliableChannel` and `UnreliableChannel` registered through Lightyear 0.26 `ChannelSettings`. |
| 7. S2C reconnect messages on reliable channel | VERIFIED | `S2CHandshake`, `S2CGameSnapshot`, `S2CObjectiveIdentities`, `S2CPhaseChanged`, and reconnect notifications are registered reliable. |
| 8. `C2SHello` first-message contract | IMPLEMENTED | `handle_reconnect` is the sole `C2SHello` drainer; generic network logging no longer drains it. |
| 9. Token identity bridge | IMPLEMENTED | `ReconnectTracker.token_map` maps `SessionToken` to `(SessionId, PlayerId)`. |
| 10. Snapshot-first reconnect sequence | IMPLEMENTED | `handle_reconnect` dispatch order is handshake, snapshot, objective identities, phase changed. |
| 11. Deferred live messages | IMPLEMENTED | `defer_unicast_for_reconnect` queues player-targeted live messages while `snapshot_sent == false`. |
| 12. Disconnect API naming | VERIFIED WITH API UPDATE | Lightyear 0.26 uses `PeerId`, `Connected`, and `Disconnected` marker components. |
| 13. Timeout closure | IMPLEMENTED | `hello_timeout_watchdog` closes silent pending peers without S2C. |
| 14. Reconnect restore edge cases | IMPLEMENTED | Sang Meprise reveal state is included in `S2CGameSnapshot.active_sang_meprise_reveals` and re-sent as `S2CSangMepriseReveal`. |

## Verification Commands

- `cargo test -p server --test reconnect_snapshot_test`
- `cargo test -p server --test snapshot_secret_strip_test`
- `cargo test -p server --test game_over_teardown_test`
- `cargo check -p server`
