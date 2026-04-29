# Board Rendering

> **Status**: In Design
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: No idle spectating · Simple surface · Deep emergence

## Overview

Board Rendering is the client-side system that consumes replicated server state and presents the game as a visual arena: a 5×8 grid of lanes where the player reads positions, threats, and information at a glance. It subscribes to the Lightyear-replicated board state — unit positions, HP values, objective HP, prism tokens, status effects, and spawn range — and renders them as sprites, health bars, and visual indicators in world space. When the Round State Machine shifts phases, the board's visual mode shifts with it: DRAFT/PLACEMENT shows the static board and highlights the player's valid spawn range; RESOLUTION begins with the simultaneous reveal of both players' placements and plays back the sub-step animation sequence from `S2CResolutionEvent`; the transition back to DRAFT returns the board to static state.

The board is the place the opponent cannot lie. Units they have already placed are physical evidence — their position, their type, and their facing are visible truth in a game of hidden hands and fake objectives. Board Rendering owns the legibility of that truth: unit sprites must be identifiable at full-board zoom, status effects must attach visibly to their owners, HP bars must update in real time during RESOLUTION so a player watching sub-step 6 sees damage landing as it happens. All interactive UI (hand cards, placement controls) is suppressed during RESOLUTION_EXECUTING — the board becomes a read-only tactical display and the player's job is to read it.

## Player Fantasy

**The board is the place the opponent cannot lie.**

Hands lie. Bids lie. Two of the opponent's five objectives are counterfeits. But the units they have committed to the field are sworn testimony — their positions are facts, their facing is intent, their HP is a record. Board Rendering exists to make that testimony legible.

**The emotional target:** The player feels like the director and audience of a five-act play that writes itself in real time. PLACEMENT is the rehearsal — quiet, deliberate, full of secret intent. RESOLUTION is the curtain rising on all five stages at once: lanes erupt simultaneously, units clash, objectives crack and reveal what they really were. The player's eyes sweep left to right, drinking in five lanes of consequence in seconds. The board doesn't argue; it just plays the tape. Every position is a fact the opponent committed in ink. Every objective shatter is a verdict on a bluff. The board is where the lies end — and where the better reader wins.

**What the player must feel:**
- **Active, not passive** — during RESOLUTION the board floods with information. A skilled player's eyes are scanning unit positions, objective HP deltas, status effect changes, lane commitments. Watching is reading is playing.
- **Legibility as earned power** — a veteran looks at the same mid-RESOLUTION board as a newcomer and extracts three times more information in the same glance. That gap must feel good to both: the veteran feels sharp; the newcomer can still follow what happened and learn the vocabulary.
- **The board makes me a better tactician** — not because the animations are beautiful, but because every sprite is exactly where it needs to be, every indicator is exactly the right size, and after twenty games the player reads the board faster than they think.

**What to avoid:** Treating the board as decorative substrate or invisible plumbing. The board is a protagonist in the experience — the surface on which the entire information war resolves. Animations that obscure tactical state have failed. Status indicators that require hovering to be understood have failed. If the player cannot take in all five lanes simultaneously during RESOLUTION, the board has failed.

*Pillar alignment: "No idle spectating" — watching IS playing when the board is designed to be read. "Simple surface" — the visual rule is that positions are facts: one rule, infinitely deep.*

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
