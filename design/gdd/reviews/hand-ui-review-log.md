# Review Log — Hand UI

## Review — 2026-04-30 (R3) — Verdict: APPROVED
Scope signal: L
Specialists: lean (no specialist agents)
Blocking items: 0 | Recommended: 4 (all resolved in-session)
Summary: R3 lean re-review found no new blocking items. Four minor recommended fixes applied: HU-10/HU-10b duplicate ACs merged into one; S2CGoldUpdate-as-resolver assumption made explicit in Rule 5c (guarded for future gold-neutral instant cards); state machine annotated to close the GRID→PASSIVE_LOCKED gap; BoardLayout initialization constraint added to Dependencies. Document is implementation-ready; OQ5–OQ8 are pre-implementation gates, not design gaps.
Prior verdict resolved: Yes — R2 NEEDS REVISION fully resolved.

---

## Review — 2026-04-30 (R2) — Verdict: NEEDS REVISION → revised in-session

Scope signal: L (multi-system integration; 3 formulas; bidirectional Board Rendering interface; NP protocol dependencies)
Specialists: game-designer, systems-designer, ux-designer, network-programmer, performance-analyst, qa-lead, creative-director
Blocking items resolved: 6 | Recommended: 5 | Advisory: 8
Prior verdict resolved: Yes — R1 (2026-04-30) APPROVED was premature; fresh-session re-review found new issues

### Summary (creative-director)

Fresh-session re-review caught what the in-session pass missed: two of the four player-fantasy pillars were being broken by current rules, and three protocol-level blockers made the spec literally unimplementable. Rule 9's auto-stage on timer expiry made the engine the commit agent (Pillar 1 broken); Rule 13's auto-decrement was specified but its trigger condition was unreachable under the default=0 invariant, making HU-26 vacuously true. Sold-out detection had contradictory outcomes from the same observable, and the activation lock could produce a guaranteed 3s freeze on silent server discard. All six resolved in-session: Rule 9 replaced with a 200ms grace window requiring explicit mouse-up; Rule 13 simplified to block-at-ceiling (no auto-decrement); sold-out ambiguity accepted (all non-arrival paths unified to timeout-revert); stale hand state documented as accepted failure; S2CActivationRejected added as a new OQ8 NP gate; HU-26 rewritten against the correct trigger. Recommended items addressed: Rule 5d (DRAFT_SHOP drag suppression), VA-10 (PASSIVE_LOCKED indicator), strip positioning spec, HU-24 drag sprite reset, HU-25/HU-04 AC precision fixes.

### Changes Applied (R2)

| Category | Item | Location |
|---|---|---|
| B1 | HU-26 rewritten (correct trigger: [ + ] path; "SHALL" language) | HU-26 AC |
| B2 | Rule 13 auto-decrement removed; [ + ] blocks at ceiling; VA-9/audio cleaned | Rule 13, VA-9, Edge Cases |
| B3 | Sold-out ambiguity accepted; Rule 14 unified; HU-10/10b/10c simplified; sold-out visual removed | Rule 14, VA-2, HU-10/10b/10c, Edge Cases |
| B4 | Stale hand state documented in Rule 10 | Rule 10 |
| B5 | Rule 5c + HU-28/28b updated with S2CActivationRejected; OQ8 added; Interactions updated | Rule 5c, HU-28/28b, Interactions, OQ8 |
| B6 | Rule 9 rewritten (200ms grace window); HU-15/15b updated; Edge Case updated | Rule 9, HU-15/15b, Edge Cases |
| Rec | Rule 5d: DRAFT_SHOP drag suppressed | Rule 5d |
| Rec | VA-10: PASSIVE_LOCKED read-only indicator (70% opacity + label) | VA-10 |
| Rec | Strip positioning spec (center-on-ghost-slot, overlap accepted) | Rule 13 |
| Rec | HU-24 extended (drag sprite Visibility::Hidden on reconnect) | HU-24 |
| Rec | HU-25 [ + ] disable corrected (3rd click, not 4th); HU-04 Animator<T> enumeration | HU-25, HU-04 |

### Remaining Open Questions

- OQ5: Card ID → visual asset mapping (ADR needed)
- OQ6: Atlas sharing with Board Rendering (draw-call implications)
- OQ7: Card art zoom resolution (120→240 upscale vs native 240 source)
- OQ8: S2CActivationRejected not in NP GDD — BLOCKING gate for HU-28/HU-28b



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
