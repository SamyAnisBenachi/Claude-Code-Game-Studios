# Review Log — Hand UI

## Review — 2026-04-30 — Verdict: APPROVED (revised in-session)

Scope signal: L (multi-system integration; 3 formulas; Hand UI ↔ Board Rendering interface)
Specialists: game-designer, systems-designer, ux-designer, network-programmer, performance-analyst, qa-lead, creative-director
Blocking items resolved: 10 (4 P0 + 6 P1) | Recommended: 11 (partially addressed in-session)
Prior verdict resolved: N/A — first review

### Summary (creative-director)

The GDD was structurally complete and pillar-aligned in spirit but had four cross-cutting P0 blockers preventing sprint pipeline entry. All four were resolved in-session. The two most significant fixes were: (1) the `GhostPlacementChanged` interface mismatch with Board Rendering — the payload was extended from `{ cell: Option<(u8,u8)>, card_id }` to `{ target: Option<PlayTarget>, card_id }` to support all five PlayTarget variants, with `GhostClickedEvent` and `GhostDragStartEvent` reverse events added for un-staging; (2) Reserve Mana Split, named in the Overview with zero specification, was fully spec'd as Rule 13 with a class-agnostic +/- control, VA-9 visual spec, edge cases, and three ACs. Formula 1 had two bugs (count=2 clamp, Y-direction inverted for bevy_ui screen-space) both corrected. The state machine gained the missing GRID→PASSIVE first-round transition. Rule 9 was updated to auto-stage if the cursor is over a valid target at timer expiry, honouring the anti-pillar "not a twitch game." AC quality was overhauled — 8 FAIL ACs (visual assertions in BLOCKING) were rewritten using a state-vs-visual split convention; 16 new ACs added.

### Changes Applied

| Category | Item | Location |
|---|---|---|
| P0 | GhostPlacementChanged payload extended to `target: Option<PlayTarget>` | Hand UI Rule 6/7/8, Interactions; Board Rendering Rule 8, Interactions, ACs |
| P0 | Reserve Mana Split spec'd (Rule 13, VA-9, 3 ACs, edge cases) | hand-ui.md |
| P0 | Un-stage matrix: GhostClickedEvent + GhostDragStartEvent reverse events; ACs HU-21/21b/21c | hand-ui.md, board-rendering.md |
| P0 | Rule 10 client-side pre-validation; ACs HU-17b/17c | hand-ui.md |
| P1 | Formula 1: count=2 clamp removed, Y direction negated, count=0 guard | Formula 1 |
| P1 | GRID→PASSIVE state transition added | States and Transitions |
| P1 | Rule 9: auto-stage on timer expiry if valid target under cursor; AC HU-15b | Rule 9, Edge Cases |
| P1 | AC overhaul: 8 FAILs rewritten (state vs visual); 16 ACs added total | Acceptance Criteria |
| P1 | Rule 5b (OQ2 → zoom=click), Rule 5c (activation lock), Rule 14 (purchase timeout) | Detailed Design |
| Advisory | "WASM GC spikes" wording fixed; fan_half_spread min raised to 180px; OQ6/OQ7 added | Rule 1, Tuning Knobs, OQs |

### Open Questions Closed This Review

| OQ | Resolution |
|---|---|
| OQ1 (Reserve mana split) | Spec'd as Rule 13 — class-agnostic +/- control, parameterizable by Class System |
| OQ2 (Zoom→click activation) | Promoted to Rule 5b: activate immediately on click while zoomed |
| OQ4 (GhostPlacementChanged interface) | Resolved — Board Rendering Designed; payload extended; reverse events defined |

### Remaining Open Questions

- OQ5: Card ID → visual asset mapping (ADR needed)
- OQ6: Atlas sharing with Board Rendering (draw-call implications)
- OQ7: Card art zoom resolution (120→240 upscale vs native 240 source)
