# Epic: Prism System

> **Layer**: Feature (M3)
> **GDD**: design/gdd/prism-system.md
> **Architecture Module**: `server/feature/prism/`
> **Status**: Ready
> **Stories**: Not yet created — run `/create-stories prism-system`

## Overview

The Prism System implements the per-lane reward tokens that make every lane a contested
economic resource throughout the game. It owns `PrismState` (a `Resource` tracking
`collected[lane][player]` booleans for 5 lanes × all players), consumes `PrismCollected`
messages emitted by the Board/Lane System during RESOLUTION sub-step 5, routes to
lane-specific rewards (Lane 1/5 → `prism_strike` spell card, Lane 2/4 → `prism_reserve`
spell card, Lane 3 → random draw from `PlayerPool`), and manages the full-set respawn
cycle. The single entry point is `resolve_prism_draws`, a system registered exclusively
in the RESOLUTION phase within the ADR-005 schedule slot
(`apply_placement_effects → resolve_ecaflip_triggers → resolve_prism_draws → award_fake_objective_rewards`).
Cards are added to hand via the `hand_push()` shared API (ADR-016), which Card Acquisition
also uses, avoiding `ResMut<PlayerHands>` access conflicts. Client-side token visibility
is replicated via `PrismPresence` components on the unreliable channel; `S2CCardAcquired`,
`S2CPrismRespawned`, and `S2CPrismRewardDropped` are sent as reliable unicasts per ADR-008.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-005: Server-side RNG | Names `resolve_prism_draws` in RESOLUTION schedule; Lane 3 consumes one `next_seed()`; hand-full short-circuit must skip seed consumption | LOW |
| ADR-008: Lightyear Channel Config | `S2CCardAcquired`, `S2CPrismRespawned`, `S2CPrismRewardDropped` → `ReliableChannel`; `PrismPresence` component replication → `UnreliableChannel` | HIGH |
| ADR-010: RSM Event Bus | `ResolutionPhaseEntered` gates `resolve_prism_draws`; ADR-010 Subscriber Contracts table must add Prism row before any story is marked Done | HIGH |
| ADR-016: Prism System Architecture | `PrismState` as single-writer `Resource`; `hand_push()` shared API; `PrismCollected` is `#[derive(Message)]` via `MessageReader`; `PrismPresence` replication pattern; Lightyear server→client unicast send pattern | HIGH |

## GDD Requirements

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-PRI-001 | Lane 1/5 grant `prism_strike` (1 damage, 3 mana cost); pool-untracked static card | ADR-016 ✅ |
| TR-PRI-002 | Lane 2/4 grant `prism_reserve` (0 cost, +1 reserve_mana); pool-untracked static card | ADR-016 ✅ |
| TR-PRI-003 | Lane 3 draws random Minion\|Spell via `draw_random(filter, seed)` in `ServerRng` `resolve_prism` slot | ADR-005 ✅ ADR-016 ✅ |
| TR-PRI-004 | Hand-full short-circuits Lane 3 before `next_seed()` call; Lanes 1/2/4/5 hand-full emits `S2CPrismRewardDropped` | ADR-016 ✅ |
| TR-PRI-005 | Full-set respawn: all 5 prisms reset after player collects all 5; `S2CPrismRespawned` reliable unicast to both players | ADR-008 ✅ ADR-016 ✅ |
| TR-PRI-006 | Respawned prism not collectible in same RESOLUTION — reward delivery (Rule 6) precedes respawn (Rule 9) | ADR-016 ✅ |
| TR-PRI-007 | No gold awarded for prism collection; Economy System not in call chain | ADR-016 ✅ |
| TR-PRI-008 | Processing order: ascending `player_id` then ascending `lane`; deterministic per ADR-005 inter-player ordering rule | ADR-005 ✅ ADR-016 ✅ |

## Pre-Implementation Gates

These must be resolved before any story in this epic can be marked **Ready**:

| Gate | Item | Owner | Status |
|------|------|-------|--------|
| OQ4 | `server-rng.md` Rule 3 caller table needs conditional note: "0 seeds consumed if hand is full at Lane 3 collection time" — prevents audit-replay misalignment | Server-side RNG GDD | Open |
| NP OQ1 | Lightyear 0.26 server-side unicast API (`send_message_to_target::<ReliableChannel, _>(msg, NetworkTarget::Single(client_id))`) must be verified against `docs.rs/lightyear/0.26`; use `liv-bevy-lightyear` skill | Network programmer | Open |
| NP GDD | Register `S2CPrismRespawned { player_id: PlayerId }` and `S2CPrismRewardDropped { player_id: PlayerId, lane: u8 }` in `network-protocol.md` | Network Protocol GDD | Open — documentation task |

## Definition of Done

This epic is complete when:

- All stories are implemented, reviewed, and closed via `/story-done`
- All acceptance criteria from `design/gdd/prism-system.md` (25 ACs: 23 BLOCKING, 1 ADVISORY, 1 relocated) are verified
- All Logic and Integration stories have passing test files in `tests/unit/prism/` or `tests/integration/prism/`
- ADR-010 Subscriber Contracts table updated to include `ResolutionPhaseEntered → resolve_prism_draws`
- `S2CPrismRespawned` and `S2CPrismRewardDropped` registered in `network-protocol.md`
- `server-rng.md` Rule 3 OQ4 conditional note added
- `DiscardLog` resource and `AuditLog` resource accessible for test inspection (PS-12, PS-17)
- `PrismPresence` component replication confirmed against Lightyear 0.26 API via `liv-bevy-lightyear`

## Next Step

Resolve the three pre-implementation gates above, then run `/create-stories prism-system`.
