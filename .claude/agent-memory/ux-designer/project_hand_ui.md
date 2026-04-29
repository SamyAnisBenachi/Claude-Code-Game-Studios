---
name: Project context — Lanes and Lies Hand UI
description: Hand UI GDD Section C in active design as of 2026-04-29; locked decisions and open UX gaps catalogued
type: project
---

Hand UI GDD is at `design/gdd/hand-ui.md`. As of 2026-04-29, Overview and Player Fantasy sections are written. Section C (Detailed Design) is being authored.

Locked upstream decisions:
- PLACEMENT interaction: drag-from-hand to board (not click-to-play)
- DRAFT_INITIAL: 3x3 grid overlay, click-to-buy
- Staged cards removed from hand fan, shown at board destination as ghost/preview
- Batch submit via C2SSubmitPlacement (atomic, irrevocable)
- Empty submit = zero cards played
- Xelor class only: reserve split control appears per staged card after staging

Key UX gaps identified in 2026-04-29 session:
1. No-valid-target drag behavior (TargetUnit with empty board, occupied cell for Minion)
2. Instant card drag target is unspecified
3. Staged card fan representation (ghost slot vs. empty slot vs. nothing)
4. 10-second pressure micro-interactions (timer display, urgency signals)
5. DRAFT_INITIAL grid behavior on purchase (grid persistence, full-hand rejection)

**Why:** These gaps must be resolved before Section C can be written; each is load-bearing for ui-programmer implementation scope.
**How to apply:** When working on Hand UI, treat these 5 gaps as the primary unresolved design questions. Do not assume answers — surface them to the user.
