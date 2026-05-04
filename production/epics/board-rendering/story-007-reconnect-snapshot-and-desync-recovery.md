# Story 007: Reconnect Snapshot and Desync Recovery

> **Epic**: Board Rendering
> **Status**: Blocked
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-005`
**ADR Governing Implementation**: [ADR-011: Reconnect and Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

Board Rendering must rebuild deterministically from `S2CGameSnapshot` after reconnect or desync. The GDD also allows Board Rendering to send `C2SRequestSnapshot` when the presentation state is stuck. This story is blocked until the shared protocol type and registration exist.

## Blocker

`C2SRequestSnapshot` is specified in `design/gdd/network-protocol.md` but is not currently present in `shared/src/protocol.rs`. This story should remain blocked until the protocol type is added and registered on ReliableChannel.

## Acceptance Criteria

- [ ] `S2CGameSnapshot` fully rebuilds board entities in a single frame.
- [ ] Snapshot rebuild clears `ObjectiveIdentityCache` until `S2CObjectiveIdentities` arrives.
- [ ] `PendingPhaseChange` and `PendingResolutionScript` are cleared or reconciled by snapshot receipt.
- [ ] Snapshot received while lobby/empty board state transitions board render state to the snapshot phase.
- [ ] Stuck reveal and stuck pending-resolution paths enqueue `C2SRequestSnapshot` after cooldown rules are available.
- [ ] If the server rate-limits the snapshot request, client logs the stuck state and relies on heartbeat disconnect as the backstop.

## Implementation Notes

- The snapshot path must be idempotent: duplicate snapshots should converge on the same rendered board.
- Do not broadcast or infer secrets from snapshot data; respect per-client stripping from the server.
- `S2CObjectiveIdentities` is explicitly resent on reconnect per ADR-011; Board Rendering should not assume reliable replay across transport reconnects.

## Out of Scope

- Adding `C2SRequestSnapshot` to shared protocol.
- Server reconnect snapshot assembly.
- Shop/Auction UI snapshot recovery (owned by `shop-auction-ui` Story 008).

## QA Test Cases

- **Lobby snapshot**
  - Given: no board entities exist
  - When: a snapshot for `DRAFT_INITIAL` arrives
  - Then: board entities are spawned, board state matches snapshot phase, and buffers are empty.

- **Reconnect cache clear**
  - Given: `ObjectiveIdentityCache` has entries
  - When: snapshot rebuild runs
  - Then: the cache is empty until fresh `S2CObjectiveIdentities` arrives.

- **Stuck recovery**
  - Given: `BoardRenderState == ResolutionReveal` without a resolution event past timeout
  - When: recovery system runs after protocol support exists
  - Then: exactly one `C2SRequestSnapshot` is enqueued for the cooldown window.

## Test Evidence

**Required evidence**:
- Integration: `tests/integration/board_rendering/reconnect_snapshot_test.rs`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 003](story-003-snapshot-spawn-units-objectives-and-hp-bars.md), `C2SRequestSnapshot` protocol implementation.
- Unlocks: reconnect QA for all board stories.
