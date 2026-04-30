# Epic: Board / Lane System

> **Layer**: Feature (M1)
> **GDD**: design/gdd/board-lane-system.md
> **Architecture Module**: `server/feature/board/`
> **Status**: Ready
> **Stories**: Not yet created — run `/create-stories board-lane-system`

## Overview

The Board/Lane System owns the spatial model of the game: a 5-lane, 8-cell-deep grid
per player, the PendingPlacements buffer that holds submitted cards until reveal, spawn
range enforcement (only the first N cells from spawn are valid placement targets), and
the PrismState tracker that emits PrismCollected events on spawn-cell contact. It
receives PlacementPhaseEntered to open the submission window, validates and buffers
C2SSubmitPlacement messages (all-or-nothing per player), and on ResolutionPhaseEntered
it atomically commits the buffer: enqueuing S2CPlacementReveal first, then spawning
unit ECS entities. It also subscribes to fake_objectives_destroyed from the Objective
System to expand the player's spawn range when a fake objective is destroyed.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-007: Placement Buffer | Placements buffered as plain Rust data (not ECS entities) until S2CPlacementReveal is enqueued; reveal-before-spawn ordering is a load-bearing invariant that prevents leaking opponent placements via Lightyear replication | HIGH |
| ADR-010: RSM Event Bus | Board/Lane subscribes to PlacementPhaseEntered and ResolutionPhaseEntered via MessageReader; RSM has zero direct imports from feature/ | HIGH |

## GDD Requirements

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-BLS-001 | BoardGrid `[[Option<BoardCell>; 8]; 5]` per player is the sole authoritative spatial state | ADR-007 ✅ |
| TR-BLS-002 | Spawn range enforcement: only cells within SpawnRange from the player's spawn row are valid placement targets; Minions only — Structures and Traps bypass range | ADR-007 ✅ |
| TR-BLS-003 | Submitted cards are buffered in PendingPlacements resource (plain Rust data, NOT ECS entities) during PLACEMENT | ADR-007 ✅ |
| TR-BLS-004 | Placement validation is all-or-nothing per player: any invalid card in the batch silently discards the entire submission | ADR-007 ✅ |
| TR-BLS-005 | S2CPlacementReveal is enqueued on ReliableChannel before any unit ECS entity is spawned — ordering is a load-bearing invariant | ADR-007 ✅ |
| TR-BLS-006 | PendingPlacements is fully cleared on entry to each new PLACEMENT phase | ADR-007 ✅ |
| TR-BLS-007 | Mana deduction happens at PLACEMENT close, not at C2SSubmitPlacement receipt | ADR-007 ✅ |
| TR-BLS-008 | PlacementPhaseEntered Message subscription opens the placement window | ADR-010 ✅ |
| TR-BLS-009 | ResolutionPhaseEntered Message subscription triggers buffer commit (reveal + spawn) | ADR-010 ✅ |
| TR-BLS-010 | PrismState tracks per-player per-lane prism collection; PrismCollected event emitted when a unit ends standard movement at own spawn cell | ADR-010 ✅ |

> **TR registry note:** TR-BLS-001–010 are referenced in `docs/architecture/architecture-traceability.md`
> but not yet machine-populated in `docs/architecture/tr-registry.yaml`.
> Run `/architecture-review` for M2 systems to populate the YAML before `/create-stories`.

## Definition of Done

This epic is complete when:
- All stories are implemented, reviewed, and closed via `/story-done`
- All acceptance criteria from `design/gdd/board-lane-system.md` are verified (33 ACs including BR proxy ACs)
- All Logic and Integration stories have passing test files in `tests/`
- The load-bearing invariant (reveal-before-spawn per ADR-007) is enforced by CI grep:
  `grep -rn "commands.spawn" server/src/feature/board/` must never appear before
  `sender.send::<ReliableChannel>(S2CPlacementReveal)` in the same system
- `liv-bevy-018` and `liv-bevy-lightyear` skills activated on all `.rs` files in this module

## Next Step

Run `/create-stories board-lane-system` to break this epic into implementable stories.
