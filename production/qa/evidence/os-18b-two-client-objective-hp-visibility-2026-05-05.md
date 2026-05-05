# OS-18b Two-Client Objective HP Visibility Evidence

> **Date**: 2026-05-05  
> **Story**: S6-05 / Objective System Story 008  
> **Branch under test**: `work/objective-system-008-os18b-two-client-objective-hp-visibility`  
> **Harness**: `tests/integration/network/os18b_two_client_objective_hp_visibility_test.rs`  
> **Raw log**: Not captured; command output was reviewed in-session.

## Command

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Sam\.codex\memories\ccgs-os008-target'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
cargo test -p server --test os18b_two_client_objective_hp_visibility_test -- --nocapture
```

Result: PASS - 1 passed, 0 failed.

The temporary `CARGO_TARGET_DIR` and `RUSTFLAGS` were used only to avoid the local
MSVC debug PDB limit and worker-drive space pressure during verification.

## Harness Scenario

- One live Lightyear server app and two live WebSocket client apps are started.
- Both client connection entities install replication endpoints.
- The production Objective System DRAFT_INITIAL initialization spawns replicated
  `ObjectiveHp` entities with initial HP 3.
- After both clients observe initial HP state, the harness arms client-side HP
  observation.
- In one server tick/sub-step, the harness calls:
  - `take_damage(world, lane 3, PlayerId(1), 2)`
  - `take_damage(world, lane 3, PlayerId(1), 2)`
- The first call creates the client-invisible intermediate HP 1.
- The second call reaches final HP 0 and triggers the consequence path.
- `ResolutionComplete` is written to emit RESOLUTION-end objective destruction output.

## Server Assertions

- Final authoritative server `ObjectiveHp`: `0`.
- Queued consequence path events: exactly one
  `ObjectiveDestroyed { target_player_id: PlayerId(2), lane: 3, was_fake: false }`.
- Real objective destroyed counter for `PlayerId(2)`: `1`.
- RESOLUTION-end emitted `ObjectiveDestroyed` messages: exactly one matching event.

## Client Observations

| Client | Observed post-damage `ObjectiveHp` sequence |
|---|---|
| Client A | `[0]` |
| Client B | `[0]` |

- Intermediate HP `1`: not observed by either client.
- Duplicate final HP `[0, 0]`: not observed by either client.
- Missing final HP `[]`: not observed; both clients received final HP.

## Bug Exposed And Fix

The first harness run timed out waiting for initial `ObjectiveHp` replication to
both clients. The bug was in Objective initialization: it spawned objectives with
`Replicate::default()`, which targets a single replication sender and is not valid
once two client replication senders exist.

Fix applied:

- Objective initialization now uses `Replicate::to_clients(NetworkTarget::All)` for
  public `ObjectiveHp` entities, matching the existing Prism and Board public-state
  replication pattern.

## Verdict

PASS. This evidence satisfies OS-18b live two-client Objective HP visibility:
final server HP is correct, consequence/destruction are single-fire, and both
clients observe only the final public `ObjectiveHp` value.

`QA-COND-0003` was not edited by this worker. It can be closed later by the
serialized review/story closure flow using this evidence.
