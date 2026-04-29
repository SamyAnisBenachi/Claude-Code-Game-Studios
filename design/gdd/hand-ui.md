# Hand UI

> **Status**: In Design
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: No idle spectating · Simple surface

## Overview

Hand UI is the client-side card fan display and interaction layer through which players access and play their hand across all game phases. It subscribes to the server-authoritative hand state (a `Vec<CardId>` of up to 10 cards, delivered via unicast network messages) and renders those cards as an interactive fan at the bottom of the screen. Its behavior is phase-driven: during DRAFT_INITIAL it presents the 9-card offering for click-to-buy selection within the 45-second, 5-gold budget window; during DRAFT_SHOP and DRAFT_AUCTION it displays the current hand alongside the shop, enabling instant-effect card activation via `C2SActivateCard`; during PLACEMENT it enters the game's highest-tension state — 10 seconds for the player to select cards, assign each a PlayTarget (board cell, target unit, target objective, lane-wide, or instant), optionally split mana from the reserve pool, and submit the full batch via `C2SSubmitPlacement` as a single atomic commit with no retraction. During RESOLUTION, Hand UI fully suppresses all interaction and becomes invisible — the board is the sole display. Hand UI owns the hand's visual state and the card-play interaction chain; it does not own shop slot display, auction bidding, or board and unit rendering — those belong to Shop/Auction UI and Board Rendering respectively.

## Player Fantasy

[To be designed]

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
