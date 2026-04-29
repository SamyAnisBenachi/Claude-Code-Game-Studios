# Shop / Auction UI

> **Status**: In Design
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: No idle spectating · Auction as signature · Simple surface

## Overview

Shop / Auction UI is the client-side presentation system that surfaces every economic decision a player faces each round. It owns three distinct panels corresponding to three RSM phases: the **Draft Offering** (DRAFT_INITIAL — a fixed 9-card grid sent by the server at game start, purchasable within a 5-gold budget over 45 seconds), the **Shop Panel** (DRAFT_SHOP — three refreshable slots drawn from the player's personal pool, purchased with gold, refreshable at escalating cost), and the **Auction Panel** (DRAFT_AUCTION — the game's signature interaction: a live bid timer, current price counter, leader display, and bid input the player uses to contest the neutral card on the block).

The system consumes six server-to-client messages — `S2CDraftOffering`, `S2CShopSlots`, `S2CAuctionCard`, `S2CAuctionBidAccepted`, `S2CAuctionSettled`, `S2CAuctionBidRejected` — and `S2CGoldUpdate` for real-time economy display. All panel state transitions follow RSM phase changes. During DRAFT_AUCTION the Shop Panel remains visible but all purchase and refresh interactions are locked; the Auction Panel is the primary UI focus. During DRAFT_SHOP the Auction Panel is dismissed and the Shop Panel becomes fully interactive. The system produces `C2SPurchaseCard`, `C2SRefreshShop`, and `C2SPlaceBid` messages from player input.

The player fantasy this system serves is economic agency under imperfect information: the player sees their own gold and the opponent's visible free gold, sees the card on the block, and chooses how much conviction to reveal with each bid. The shop builds an archetype in silence; the auction forces a single, legible decision.

## Player Fantasy

The Shop / Auction UI is your dashboard, but it is also a window your opponent is staring through. Every number on it tells two stories at once: your gold is your plan, and it is also their read on you.

It is the smallest possible surface where conviction has to land. Three things, always: how much you have, what it would buy, and how long until the moment passes. The panel does not editorialize. It does not tell you which slot to take or when to bid. It shows you the cost. The decision is yours — and the UI's restraint is what makes the decision feel like one.

In DRAFT_INITIAL the panel asks *who are you?* Nine cards, a 5-gold budget, no refresh. You are reading the lineup while your opponent reads theirs, and every card you pick is a card they cannot have. In DRAFT_SHOP it asks *who are you becoming?* Three slots refresh toward your archetype, and you feel the tilt if you have committed: a second Gobball appearing beside the first, probability leaning your way without announcing itself. In DRAFT_AUCTION it asks *how badly?* Five gold left, eight seconds on the timer, the +1g button under your cursor. The auction panel does not help. It just holds the number up and waits.

The other moment this system owns: buying nothing. You close DRAFT_SHOP with your gold intact, cards unspent, opponent's gold total unchanged on your screen. You have revealed nothing. The panel made that restraint legible — not an absence, but a position.

**Pillar alignment:** "No idle spectating" — every state of this UI offers a live, meaningful signal or a decision. "Auction as signature" — the auction takeover is the panel's highest-stakes configuration; it is designed to be the most consequential 20 seconds of the round. "Simple surface" — three values, one question at a time.

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
