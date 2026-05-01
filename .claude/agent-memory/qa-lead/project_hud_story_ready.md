---
name: hud QL-STORY-READY review
description: Story-readiness verdicts for HUD epic (10 stories); 7 ADEQUATE, 2 GAPS, 1 BLOCKED
type: project
---

QL-STORY-READY review for HUD epic completed 2026-05-01.

**Results:**

| Story | Verdict | Key issue |
|---|---|---|
| 001 — Plugin Scaffold | GAPS | HUD-11 "no countdown" needs type enumeration; compile-check criterion must move to smoke-check (not Logic test) |
| 002 — Gold and Mana | ADEQUATE | — |
| 003 — Phase Label / Round Counter | ADEQUATE | — |
| 004 — Scoreboard Dot Observer | BLOCKED | OQ-HUD-05: HudObjectiveUpdate crate location unresolved; criteria are ADEQUATE once unblocked |
| 005 — Phase Transitions | ADEQUATE | — |
| 006 — ECONOMY_AUCTION TextSpan | ADEQUATE | — |
| 007 — GAME_OVER Freeze | ADEQUATE | — |
| 008 — Reconnect Snapshot | ADEQUATE | — |
| 009 — Same-Tick Tie-Break | ADEQUATE | Minor: reserved_gold post-tick assertion missing (advisory) |
| 010 — Numeric Tween | GAPS | GAME_OVER snap + multi-update collapse are correctness rules misclassified as Visual/Feel; should be BLOCKING Logic criteria |

**Why:** Story 004 blocked on OQ-HUD-05 (HudObjectiveUpdate trigger type cross-plugin crate location). Story 010 gaps are a type misclassification risk: a FROZEN HUD showing mid-tween values instead of authoritative final values is a Logic bug, not a feel issue.

**How to apply:** Do not mark Story 001, 004, or 010 as sprint-ready until gaps are resolved. Stories 002, 003, 005, 006, 007, 008, 009 are clear to enter sprint. For Story 010 correctness criteria (GAME_OVER snap, multi-update collapse), require BLOCKING unit tests regardless of Visual/Feel type label.
