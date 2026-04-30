# Epic: Objective System

> **Layer**: Feature (M1)
> **GDD**: design/gdd/objective-system.md
> **Architecture Module**: `server/feature/objective/`
> **Status**: Ready
> **Stories**: 7 stories — see table below

## Overview

The Objective System owns the win condition infrastructure: five objectives per player
with visible HP and hidden real/fake identity. It assigns fake lanes via ServerRng at
DRAFT_INITIAL and delivers S2CObjectiveIdentities as reliable unicast per player
(ADR-001 pattern — identity never replicated as an ECS component, never broadcast).
It processes objective damage during RESOLUTION sub-steps, emits ObjectiveDestroyed
events (which Economy System and Board/Lane System subscribe to for gold rewards and
spawn-range expansion), and maintains ObjectiveCounters for the RSM win condition
check. Sang Méprise reveal is handled as a one-shot reliable unicast to the opponent
only, persisting client-side for RESOLUTION duration. On every reconnect,
S2CObjectiveIdentities is re-sent because reliable delivery is not guaranteed across
transport reconnects.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-001: Objective Identity Unicast | `ObjectiveIdentity { is_fake }` lives in server-only HiddenObjectives resource, never replicated; delivered via reliable unicast S2CObjectiveIdentities at DRAFT_INITIAL and on every reconnect | HIGH — Lightyear 0.26 unicast (`NetworkTarget::Single(PeerId)`) is post-cutoff; verify `ServerMultiMessageSender` API before implementing |
| ADR-010: RSM Event Bus | Objective System subscribes to ResolutionPhaseEntered via MessageReader; RSM reads Res<ObjectiveCounters> for GAME_OVER check without importing from feature/objective/ | HIGH |

## GDD Requirements

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-OBJ-001 | ObjectiveState { hp, ar } per lane/player is a replicated ECS component broadcast to both clients on every change | ADR-001 ✅ |
| TR-OBJ-002 | HiddenObjectives { is_fake: bool } is a server-only Resource, never replicated as an ECS component or sent in any broadcast | ADR-001 ✅ |
| TR-OBJ-003 | Fake lane assignment uses ServerRng at DRAFT_INITIAL (2 seeds/player, ascending player_id) per ADR-005 consumption order | ADR-001 ✅ |
| TR-OBJ-004 | S2CObjectiveIdentities sent as reliable unicast to each player immediately after fake assignment at DRAFT_INITIAL | ADR-001 ✅ |
| TR-OBJ-005 | S2CObjectiveIdentities re-sent on every reconnect; reliable delivery not guaranteed across transport reconnects | ADR-001 ✅ |
| TR-OBJ-006 | ObjectiveDestroyed event emitted with { target_player, lane, was_fake, attacker } when objective hp reaches 0 | ADR-010 ✅ |
| TR-OBJ-007 | Economy System receives ObjectiveDestroyed → apply_gold_award(attacker, objective_gold_reward) if attacker ≠ target | ADR-010 ✅ |
| TR-OBJ-008 | ObjectiveCounters { real_destroyed, fake_destroyed } Resource updated on each ObjectiveDestroyed; RSM reads it for GAME_OVER check | ADR-010 ✅ |
| TR-OBJ-009 | ResolutionPhaseEntered Message subscription triggers objective damage processing | ADR-010 ✅ |
| TR-OBJ-010 | Sang Méprise reveal: S2CSangMepriseReveal unicast to opponent only; persists in client local state for RESOLUTION duration; never broadcast | ADR-001 ✅ |

> **TR registry note:** TR-OBJ-001–010 referenced in `docs/architecture/architecture-traceability.md`
> but not yet machine-populated in `docs/architecture/tr-registry.yaml`.

## Definition of Done

This epic is complete when:
- All stories are implemented, reviewed, and closed via `/story-done`
- All acceptance criteria from `design/gdd/objective-system.md` are verified (41 ACs)
- All Logic and Integration stories have passing test files in `tests/`
- CI grep confirms: `grep -rn "ObjectiveIdentity\|is_fake" server/src/` never appears
  in a `Replicate` context or any S2C broadcast message
- `liv-bevy-018` and `liv-bevy-lightyear` skills activated on all `.rs` files in this module

## Stories

| # | Story | Type | Status | ADR |
|---|-------|------|--------|-----|
| 001 | [Objective State Model](story-001-objective-state-model.md) | Logic | Ready | ADR-001 |
| 002 | [Fake Assignment & Config Guards](story-002-fake-assignment-and-config-guards.md) | Logic | Ready | ADR-001 |
| 003 | [Identity Unicast Delivery](story-003-identity-unicast-delivery.md) | Integration | Ready | ADR-001 |
| 004 | [Damage Interface](story-004-damage-interface.md) | Logic | Ready | ADR-001 |
| 005 | [Destruction Consequence Path](story-005-destruction-consequence-path.md) | Logic | Ready | ADR-010, ADR-001 |
| 006 | [D4 Fake Reward Draw](story-006-d4-fake-reward-draw.md) | Logic | Ready | ADR-001 |
| 007 | [ResolutionPhaseEntered Subscription & RESOLUTION-end Sync](story-007-resolution-phase-subscription.md) | Integration | Ready | ADR-010, ADR-001 |

Work through stories in order — each story's `Depends on:` field tells you what must be DONE before you can start it.

## Next Step

Run `/story-readiness production/epics/objective-system/story-001-objective-state-model.md` then `/dev-story` to begin implementation.
