# ADR-023: Placement Timer Accessibility Authority

## Status

Accepted

## Date

2026-05-05

## Last Verified

2026-05-05

## Decision Makers

User + accessibility authority + lead-programmer

## Summary

The PLACEMENT timer accessibility multiplier is a server-authoritative
lobby/session setting, not a client-local runtime preference. Each player may
request a multiplayer-safe multiplier before `SessionReady`. The effective
multiplier is the highest requested multiplayer-safe value across all players,
capped at 3x, and frozen when `SessionReady` fires. The RSM applies the frozen
value to its PLACEMENT timer duration and broadcasts the resulting effective
duration to clients via `S2CPhaseChanged`.

The setting is displayed as a neutral room/session timer setting. The UI must
not identify which player requested the extension. The 0.5x option is not part
of multiplayer Standard-tier accessibility and must not be exposed in multiplayer
session negotiation; if documented at all, 0.5x is limited to solo, custom, or
debug pace controls outside standard multiplayer.

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Core / Networking / UI |
| **Knowledge Risk** | HIGH - Bevy 0.15-0.18 and Lightyear 0.26 are post-cutoff |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `design/gdd/game-session-system.md`, `design/gdd/round-state-machine.md`, `design/gdd/network-protocol.md`, `design/accessibility-requirements.md`, `design/ux/settings-accessibility.md`, `design/ux/hud.md`, `design/ux/hand-ui.md` |
| **Post-Cutoff APIs Used** | None directly in this docs-only ADR. Implementation will use existing Bevy resource, Observer, and Lightyear message patterns defined by ADR-009, ADR-012, and ADR-021. |
| **Verification Required** | Confirm the final implementation writes the frozen multiplier into `SessionConfig` before `SessionReady` triggers, and that `S2CPhaseChanged.timer_duration_ms` contains the RSM-computed effective PLACEMENT duration. |

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-002 (server authority), ADR-009 (RSM phase state), ADR-012 (SessionReady delivery), ADR-021 (presentation phase sink) |
| **Enables** | Multiplayer-safe PLACEMENT timer accessibility stories; Settings/Lobby timer negotiation; HUD/Hand UI timer display against server-provided duration |
| **Blocks** | Any implementation that exposes a local-only multiplayer PLACEMENT timer multiplier or includes 0.5x in Standard-tier multiplayer accessibility |
| **Ordering Note** | The GSS must finalize the effective multiplier before triggering `SessionReady`; the RSM reads only the frozen `SessionConfig` value after that point. |

## Context

### Problem Statement

The accessibility requirements and Settings UX currently describe a PLACEMENT
timer multiplier as if it were a local setting. That creates a multiplayer
authority problem: the RSM owns PLACEMENT duration, but a local client setting
cannot safely lengthen or shorten the server phase for only one player. It would
either desync client display from server truth, create a timing exploit, or leak
which player needs accessibility support.

The existing option list also includes 0.5x. In multiplayer Standard-tier
accessibility, a faster-than-default timer is not an accommodation and is not
multiplayer-safe. A player-requested 0.5x could shorten the opponent's decision
window or create conflicting requests that the server must arbitrate. The
multiplayer accessibility contract must therefore be extension-only.

### Constraints

- The server is the sole authority for phase duration and transitions.
- PLACEMENT remains simultaneous and secret; timer negotiation must not reveal
  hidden gameplay information or player-specific accessibility needs.
- Settings can be opened during safe in-game phases, but `SessionReady` freezes
  the active match configuration.
- Clients must display the timer duration the server sent; they must not multiply
  the timer locally.
- The effective setting must work for all multiplayer modes, not only 1v1.

### Requirements

- Support multiplayer-safe PLACEMENT timer values: 1x, 1.5x, 2x, and 3x.
- Default to 1x when no player requests an extension.
- Compute effective multiplier as the highest requested multiplayer-safe value
  across players.
- Cap the effective multiplier at 3x.
- Freeze the effective value at `SessionReady`.
- Apply the frozen value to the RSM-owned PLACEMENT duration, including any
  RSM-selected base PLACEMENT duration such as auction-followup placement.
- Display the value as a neutral room/session timer setting.
- Keep 0.5x out of multiplayer Standard-tier accessibility.

## Decision

The Game Session System owns PLACEMENT timer multiplier negotiation during
LOBBY. Each player's request is stored as lobby/session setting data until
`SessionReady`. The server accepts only multiplayer-safe values for standard
multiplayer negotiation:

- `1x`
- `1.5x`
- `2x`
- `3x`

The effective multiplier is:

```text
effective_placement_timer_multiplier =
    min(max(requested_multiplier[player] for all players), 3x)
```

Players who do not submit a request contribute `1x`. Requests below `1x`,
including `0.5x`, are not multiplayer-safe and are ignored or normalized to `1x`
by the multiplayer session policy. Values above `3x` do not raise the effective
value above `3x`.

Immediately before `SessionReady`, the GSS writes the frozen effective value into
`SessionConfig`:

```rust
pub struct SessionConfig {
    pub mode: GameMode,
    pub player_count: u8,
    pub team_map: HashMap<PlayerId, TeamId>,
    pub class_map: HashMap<PlayerId, ClassId>,
    pub placement_timer_multiplier_effective: PlacementTimerMultiplier,
}
```

After `SessionReady`, the value is immutable for the active match. Changing the
setting during DRAFT or GAME_OVER may update local preference storage for the
next lobby/session, but it must not alter the active match's frozen value.

On PLACEMENT entry, the RSM selects the base PLACEMENT duration from its normal
rules and applies the frozen multiplier:

```text
base_ms =
    placement_timer_seconds * 1000
    OR auction_followup_placement_timer_seconds * 1000

effective_ms = base_ms * placement_timer_multiplier_effective
```

`S2CPhaseChanged.timer_duration_ms` carries `effective_ms`. The HUD, Hand UI,
and any other client timer display use that server-provided duration directly.
They do not recompute it from local Settings.

### Architecture Diagram

```text
Client Settings / Lobby
  C2SSetPlacementTimerMultiplier { requested_multiplier }
          |
          v
Game Session System (LOBBY only)
  stores per-player requests
  computes highest multiplayer-safe value, capped at 3x
          |
          v
SessionReady boundary
  SessionConfig.placement_timer_multiplier_effective is written and frozen
          |
          v
Round State Machine
  PLACEMENT entry:
    effective_ms = selected_base_placement_ms * frozen_multiplier
    starts authoritative placement_timer
    broadcasts S2CPhaseChanged { timer_duration_ms: Some(effective_ms) }
          |
          v
Client Presentation
  displays neutral room/session timer duration from S2CPhaseChanged
```

### Key Interfaces

```rust
pub enum PlacementTimerMultiplier {
    X1,
    X1_5,
    X2,
    X3,
}

pub struct C2SSetPlacementTimerMultiplier {
    pub multiplier: PlacementTimerMultiplier,
}

pub struct S2CSessionSettingsUpdated {
    pub placement_timer_multiplier_effective: PlacementTimerMultiplier,
}

pub struct S2CPhaseChanged {
    pub phase: RoundPhase,
    pub round_number: u32,
    pub timer_duration_ms: Option<u32>,
}
```

`S2CSessionSettingsUpdated` is a neutral room/session status message. It must not
carry the identity of the player whose request raised the effective value.

## Alternatives Considered

### Alternative 1: Client-Local Timer Multiplier

- **Description**: Each client locally scales its own PLACEMENT countdown.
- **Pros**: Simple UI persistence; no protocol work.
- **Cons**: Does not change the authoritative server phase. A client showing
  30 seconds while the server closes at 10 seconds is misleading and unfair.
- **Rejection Reason**: Violates server authority and creates desync.

### Alternative 2: Host/Room Owner Chooses the Timer

- **Description**: The room creator selects one timer multiplier for everyone.
- **Pros**: Simple negotiation model.
- **Cons**: Lets one player deny another player's accessibility need. Also
  turns an accommodation into a social negotiation.
- **Rejection Reason**: Standard-tier accessibility must not depend on host
  permission.

### Alternative 3: Lowest or Unanimous Requested Value Wins

- **Description**: The server chooses the lowest requested multiplier, or applies
  an extension only if everyone asks for it.
- **Pros**: Minimizes match duration.
- **Cons**: Fails the player who needs more time. In practice this makes the
  accessibility option ineffective.
- **Rejection Reason**: Accessibility need must dominate pace preference in
  standard multiplayer.

### Alternative 4: Include 0.5x in Multiplayer

- **Description**: Expose 0.5x as another multiplayer timer option.
- **Pros**: Useful for solo/debug pacing and expert challenge modes.
- **Cons**: Shortens the opponent's decision window and is not an accessibility
  accommodation.
- **Rejection Reason**: Not multiplayer-safe for Standard-tier accessibility.
  0.5x belongs only in solo, custom, or debug pace controls if it exists at all.

## Consequences

### Positive

- Multiplayer accessibility is server-authoritative and fair to every player in
  the session.
- The highest-requested rule prevents one player from denying another player's
  timer accommodation.
- Neutral display avoids outing a player's accessibility need.
- Clients have one timer source of truth: `S2CPhaseChanged.timer_duration_ms`.

### Negative

- A player who prefers faster pacing cannot override another player's requested
  extension in standard multiplayer.
- Lobby/session settings need protocol and GSS state coverage before the UI can
  be implemented.
- Active-match setting changes cannot take effect immediately after
  `SessionReady`; UI must explain that they apply to the next session.

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Client still multiplies the timer locally | Medium | High | Control manifest rule: use server-provided `timer_duration_ms` only. |
| UI reveals which player requested the extension | Medium | Medium | `S2CSessionSettingsUpdated` carries effective value only; no player attribution. |
| 0.5x leaks into multiplayer UI | Medium | Medium | Settings/accessibility docs remove it from Standard multiplayer; protocol accepts only multiplayer-safe values. |
| RSM applies multiplier inconsistently to auction-followup PLACEMENT | Low | Medium | RSM rule states multiplier applies after selecting the base PLACEMENT duration. |

## GDD Requirements Addressed

| GDD System | Requirement | How This ADR Addresses It |
|------------|-------------|----------------------------|
| `design/accessibility-requirements.md` | Standard-tier PLACEMENT timer extension | Defines extension-only multiplayer-safe values and removes 0.5x from Standard multiplayer. |
| `design/ux/settings-accessibility.md` | Settings must not create multiplayer timing exploits | Moves active match authority to GSS/RSM and treats Settings as request/preference UI. |
| `design/ux/hud.md` | HUD timer display | Requires HUD to render server-provided effective timer duration neutrally. |
| `design/ux/hand-ui.md` | PLACEMENT staging under timer pressure | Requires Hand UI to use the server-provided timer duration and frozen session setting. |
| `design/gdd/game-session-system.md` | `SessionReady` handoff and `SessionConfig` | Adds frozen `placement_timer_multiplier_effective` to the session handoff. |
| `design/gdd/round-state-machine.md` | RSM owns PLACEMENT timer | Applies the frozen multiplier to the RSM-selected PLACEMENT base duration. |
| `design/gdd/network-protocol.md` | Multiplayer message contract | Adds lobby/session setting messages and snapshot/state fields for neutral effective timer display. |

## Performance Implications

- **CPU**: Negligible. One max operation over session players on setting change
  and/or `SessionReady`.
- **Memory**: Negligible. One enum per player request plus one frozen enum in
  `SessionConfig`.
- **Load Time**: None.
- **Network**: Tiny reliable lobby/session setting messages; no per-frame traffic.

## Migration Plan

1. Add this ADR and repair design docs to remove local-authority language.
2. Update Settings/HUD/Hand UI specs to show multiplayer-safe values only.
3. Update GSS/RSM/NP GDDs to define negotiation, freeze, and effective timer
   broadcast semantics.
4. Register control-manifest and TR entries so implementation stories inherit
   the server-authoritative contract.
5. Implement in a future code story. This ADR does not implement code.

## Validation Criteria

- [ ] Given multiple players request different multiplayer-safe multipliers
  before `SessionReady`, the GSS freezes the highest requested value, capped at
  3x, into `SessionConfig`.
- [ ] Given no player requests a multiplier, the frozen value is 1x.
- [ ] Given a multiplayer client attempts to request 0.5x, the effective
  multiplayer value remains at least 1x and the Standard UI does not expose 0.5x.
- [ ] Given PLACEMENT begins, `S2CPhaseChanged.timer_duration_ms` equals the
  selected base PLACEMENT duration multiplied by the frozen session multiplier.
- [ ] Given an active match is past `SessionReady`, changing Settings does not
  alter the active match's frozen multiplier.
- [ ] The lobby and active-session UI display the effective multiplier as a
  neutral room/session setting with no player attribution.

## Related Decisions

- [ADR-002: Client-Server Authority Model](./adr-002-client-server-authority.md)
- [ADR-009: Round State Machine Phase Representation as ECS Resource](./adr-009-rsm-phase-state.md)
- [ADR-012: SessionReady Delivery - Observer vs Buffered Events](./adr-012-session-ready-delivery.md)
- [ADR-021: Presentation Layer Architecture](./adr-021-presentation-layer-architecture.md)
