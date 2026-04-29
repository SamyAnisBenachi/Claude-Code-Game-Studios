# Keyword System

> **Status**: In Design
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: Deep emergence · Simple surface · No idle spectating

## Overview

The Keyword System is the authoritative design reference for all card abilities in Lanes and Lies. It specifies the exact behavior of approximately 28 keywords across three categories: **timing triggers** that fire in response to game events (APPEARANCE, DEATH, FINAL BLOW, COUNTERATTACK, INJURED, START OF TURN, END OF TURN); **combat keywords** that extend the base ATK/HP/AR resolution stack (FIRST STRIKE, CHARGE, RANGE 1-X, WALL, BODYGUARD, ARMOR-PIERCING, SHIELD, LEADER, RESISTANCE X, VULNERABILITY X, SILENCE, STUN, IRREMOVABLE, UNTARGETABLE, OUTNUMBERED); and **movement keywords** that alter unit positioning beyond standard MP-based advancement (CHARGE X, REPEL X, ATTRACT X, TELEPORT, CHANGE LANE). Keywords are declared as structured data in each card's `cards.json` definition; the Combat Resolution system evaluates them within its six-sub-step global pass framework, but what each keyword means — its trigger condition, its effect, its priority relative to other keywords, and its edge-case resolution — is owned exclusively by this document. For the player, keywords are the tactical vocabulary printed on every card: "FIRST STRIKE" tells a player everything they need to know about attack timing without reading a rules manual. The strategic depth of the game emerges primarily from keyword interactions — a WALL anchoring a lane against a FIRST STRIKE attacker, a RANGE unit attacking from outside COUNTERATTACK range, an INJURED unit unlocking bonus stats after absorbing a hit. The ~28 keyword definitions, combined with lane positioning and hidden placement, generate a combinatorial strategy space that rewards deep familiarity while remaining legible to newcomers on round one — the "Deep emergence" pillar operating within the "Simple surface" constraint.

## Player Fantasy

The Keyword System serves the fantasy of **authorship through anticipation** — the feeling that the board you built is a clockwork you wound up, and RESOLUTION is the moment you let it run.

During PLACEMENT, the player is not just choosing which cards to play. They are composing a sequence: *BODYGUARD enters first. FIRST STRIKE fires before anything can reach it. The CHARGE unit behind it advances two extra cells and lands at the objective.* Each keyword on each card is a gear the player slots into place — silently, with 10 seconds on the clock, knowing the opponent is building their own machine at the same time. The player commits, submits, and steps back.

Then RESOLUTION starts. And the player watches their clockwork fire.

**What the player should feel:**
- **"I built that."** — When FIRST STRIKE kills the blocker before the CHARGE unit passes, the player did not *react*. They *constructed*. That outcome existed in their head during PLACEMENT. The system confirmed it.
- **"I need to read theirs."** — The clockwork framing makes RESOLUTION active, not passive. The player watches their own machine run while simultaneously reading the opponent's: *they used IRREMOVABLE on lane 2, that's why the REPEL didn't work. They have a LEADER somewhere.* Watching is intelligence gathering.
- **"Each keyword is a gear that keeps its promise."** — At the card level, every keyword is a small contract: WALL stays put, SHIELD absorbs one hit, BODYGUARD stands in front. When a keyword fires exactly as named, the game feels legible and fair. When two keywords collide unexpectedly — SILENCE stripping INJURED's bonus mid-combat — the game rewards the player who read the interaction before placing.

**What to avoid:** Keywords must never feel like hidden gotchas. A player who loses a lane to COUNTERATTACK should leave thinking "I didn't account for that" — not "I couldn't have known." Every keyword name must describe its effect well enough that a first-time player reading the card gets the right mental model. *Simple surface* applies at the individual keyword level; *deep emergence* is what happens when the clockwork gears mesh in unexpected combinations.

*Pillar alignment: "No idle spectating" — RESOLUTION is the payoff phase of authorship, not downtime. "Deep emergence" — ~28 keywords combining with positions and card stats create the strategy space. "Simple surface" — each keyword fits its name, first read.*

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
