# Story 008: Objective Reveal and HUD Fanout

> **Epic**: Board Rendering
> **Status**: Blocked
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-005`
**ADR Governing Implementation**: [ADR-001: Objective Identity Unicast](../../../docs/architecture/adr-001-objective-identity-unicast.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

Board Rendering owns the visual reveal of destroyed objectives and the privacy boundary before HUD receives scoreboard updates. Standing objectives never reveal real/fake identity. On destruction, Board Rendering may use the local identity cache for its own view, then emits a HUD-safe update that strips fake/real data.

## Blocker

The transport contract for objective destruction is not aligned yet. Existing docs and code mention `ObjectiveDestroyed`, `S2CObjectiveDestroyed`, and objective destruction inside the resolution log. This story should stay blocked until the final message/event type and crate location are defined.

## Acceptance Criteria

- [ ] Standing objectives render identically before destruction.
- [ ] Destroyed objective reveal uses identity data only when it is legally available to the local client.
- [ ] Board Rendering does not leak `was_fake` to HUD.
- [ ] Board Rendering triggers/emits `HudObjectiveUpdate` with only player/lane/destroyed state needed by the scoreboard.
- [ ] Missing objective entity on destruction logs a warning and does not spawn a replacement.
- [ ] Spawn range highlight updates after fake-objective destruction once the replicated spawn range source exists.

## Implementation Notes

- `ObjectiveIdentity` must never be replicated as a public component.
- If the final contract uses `ResolutionEvent::ObjectiveDestroyed`, Board Rendering should handle it during resolution playback and fan out the HUD-safe update in order.
- If the final contract uses a dedicated `S2CObjectiveDestroyed`, only one client-side system may drain that Lightyear message.
- HUD should remain a read-only observer of sanitized board-rendering output.

## Out of Scope

- Server objective damage/destruction rules.
- HUD dot rendering implementation.
- Audio cue finalization for fake reveals.

## QA Test Cases

- **Privacy boundary**
  - Given: an objective destruction includes `was_fake = true`
  - When: Board Rendering handles it
  - Then: the local board may reveal the correct visual, but HUD receives no fake/real field.

- **Missing objective**
  - Given: no objective entity exists for lane 3
  - When: destruction arrives for lane 3
  - Then: no new objective entity is spawned, a warning is logged, and HUD fanout still follows the agreed contract if safe.

- **Fanout once**
  - Given: one objective destruction signal
  - When: update systems run
  - Then: exactly one `HudObjectiveUpdate` is observed.

## Test Evidence

**Required evidence**:
- Integration: `tests/integration/board_rendering/objective_reveal_hud_fanout_test.rs`
- Visual evidence: `production/qa/evidence/board-rendering-objective-reveal-evidence.md`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 003](story-003-snapshot-spawn-units-objectives-and-hp-bars.md), final objective destruction transport contract, HUD observer contract.
- Unlocks: HUD objective scoreboard completion and board objective reveal QA.
