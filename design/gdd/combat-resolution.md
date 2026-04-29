# Combat Resolution

> **Status**: In Design
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: Simple surface · Deep emergence · No idle spectating

## Overview

Combat Resolution is the phase where both players' secretly-placed cards are revealed and processed simultaneously. During the 10-second PLACEMENT window, each player selects which cards to play without knowing the opponent's choices; at RESOLUTION, all selections are exposed at once and a deterministic 6-step algorithm executes every effect across all five lanes in parallel. The six steps — applying placement effects, CHARGE bonus movement, FIRST STRIKE attacks, dead-unit removal, standard movement, and standard combat — are global passes: each step completes across all lanes before the next begins, ensuring outcomes depend entirely on unit positions and abilities, not server processing order. At the end of RESOLUTION, any unit occupying the opponent's far edge deals damage directly to the objective in that lane. For the player, RESOLUTION is the dramatic payoff of every round — the moment all economic and positional bets are settled simultaneously, where a correctly-read lane wins a combat and an unanticipated unit reaching the objective shifts the entire game.

## Player Fantasy

**Combat Resolution is the information payoff of every round.**

The player should feel like a tactician whose patience just paid off — not because they won the exchange, but because they *learned something*. RESOLUTION is when the lying stops. For ten seconds during PLACEMENT, both players committed to their bets in secret; RESOLUTION forces the board to tell the truth. Five lanes light up simultaneously and the player watches them all — not because they can change anything, but because they are reading.

**The specific moment:** The instant both placements appear simultaneously on screen. The player scans: *They sent nothing to lane 3. They doubled down on lane 4 with a FIRST STRIKE unit. Lane 5 just took my objective damage again.* That scan takes three seconds and yields enough information to restructure next round's plan entirely.

**What the player should feel:**
- **Active, not passive** — watching RESOLUTION is not downtime. The player is continuously updating their mental model of the opponent's deck, priorities, and which objectives might be real.
- **Narrative completion** — the 10-second PLACEMENT window was a question ("what are they doing?"); RESOLUTION answers it. Win or lose, the loop closes satisfyingly.
- **Strategic continuity** — by the time the gold ticks up at round end, the player has already started next round's placement in their head. The board does not pause. Neither does the player.

**What to avoid:** RESOLUTION must never feel like passive watching. The animation playback and the information reveal should arrive together — the player should be reading outcomes as they happen, not watching a cutscene. If a player has "nothing to do" during RESOLUTION, the UI has failed: they should be tracking unit positions, watching objective HP, noting which of the opponent's lanes stayed empty, and recalculating their economic position.

*Pillar alignment: "No idle spectating" — watching IS playing. "Deep emergence" — the strategic depth of hidden information becomes legible during this window.*

## Detailed Design

### Core Rules

[To be designed]

### States and Transitions

[To be designed]

### Interactions with Other Systems

[To be designed]

## Formulas

[To be designed]

## Edge Cases

[To be designed]

## Dependencies

[To be designed]

## Tuning Knobs

[To be designed]

## Visual/Audio Requirements

[To be designed]

## UI Requirements

[To be designed]

## Acceptance Criteria

[To be designed]

## Open Questions

[To be designed]
