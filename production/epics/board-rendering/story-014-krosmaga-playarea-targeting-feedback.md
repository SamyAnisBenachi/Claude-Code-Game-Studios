# Story 014: S19-BR-PLAYAREA-HIERARCHY-TARGETING-FEEDBACK-001 -- PlayArea Hierarchy + Targeting Feedback

> **Epic**: Board Rendering
> **Story ID**: `S19-BR-PLAYAREA-HIERARCHY-TARGETING-FEEDBACK-001`
> **Status**: Draft -- future Sprint 19 candidate; NOT activated
> **Layer**: Presentation / Board Rendering + PlayArea coordination
> **Type**: UI + Visual/Feel + Integration
> **Sprint**: Future Sprint 19 implementation wave; depends on Sprint 18 PlayArea/container foundations
> **Authored**: 2026-05-18 by PROMPT 1280
> **Authoring source-of-truth**: local `HEAD@051b59d`; workspace had unrelated dirty code/runtime files that were not touched
> **Source reports**: PROMPT 1265, 1266, 1267
> **Estimated effort**: ~0.75d

---

## Status / No-Claim Banner

This story is a future candidate only. It does not activate Sprint 19 and does
not claim release readiness, final-art completion, Standard-tier accessibility
completion, playtest validation, full game completion, or closure of
`PAW-TD-*-a`.

Krosmaga targeting visuals are translated into CCGS placement/resolution
semantics. This story does not copy Krosmaga movement/attack rules, AP/end-turn
semantics, or asset files.

---

## Source Findings

PROMPT 1266 defines the useful hierarchy:

1. Atmospheric background.
2. Physical board and board props.
3. Units, objectives, resource pickups.
4. HUD edge chrome and hand.
5. Hover/drag card and board targeting path.
6. Glossary/tooltips/event details.
7. Modal/result overlays.

It recommends a 16:9 target of top edge chrome `0%-6%`, board envelope
`7%-80%`, and hand/action region `80%-100%`. Targeting mode should dim
non-target lanes to roughly `45%-60%` perceived brightness while valid paths use
high-glow rings or connected path segments.

PROMPT 1265 shows that large card inspect, glossary panels, lane paths, damage
bursts, and endpoint symbols should work together at the point of action. PROMPT
1267 maps board cells, rails, objective states, deck trays, and placement/target
indicator references as dev-only proxy candidates after provenance gating.

---

## Scope

### In Scope

- Reconcile the board's PlayArea hierarchy with the Sprint 18 PlayArea container
  so the board remains the hero surface while hand, HUD, shop/auction panels,
  and CTAs stay in predictable edge regions.
- Add a board-targeting presentation layer for PLACEMENT drag/hover and any
  existing target-selection state:
  non-target dim, valid path/range segments, endpoint rings, source-card link,
  and invalid-target subdued state.
- Preserve current board authority:
  spawn range from `PlayerSnapshot.spawn_range_cells` and live
  `SpawnRangeChanged`, placement ghost from Hand UI, and resolution events from
  the authoritative log.
- Position targeting feedback in world space or UI overlay space according to
  ADR-021 and `docs/ux/board-rendering-spec.md`; document any split.
- Add QA snapshot/debug fields proving board envelope, active target cells,
  dim state, path/ring counts, and CTA/hand overlap status.
- Capture browser/WASM evidence for idle board, active placement targeting,
  invalid target, and resolution/damage feedback if already available.

### Out Of Scope

- New target-validation rules.
- New network messages or server-authoritative board state.
- Krosmaga movement/attack semantics.
- Deck/discard/fatigue props.
- Final board art or release approval.
- Closure of blocked Board Rendering reconnect/objective transport stories.

---

## Dependencies And Parallelism

| Dependency | Required posture |
|---|---|
| `S18-UI-PLAY-AREA-CONTAINER-001` | Should be Done or explicitly superseded before this story alters hierarchy. |
| BR-011 Spawn Range Highlights | Needed for persistent spawn-range source if not already landed. |
| `docs/ux/board-rendering-spec.md` | Governs cell, unit, range, status, and ghost-preview rules. |
| Hand UI drag/ghost stories | Source-card and drag ghost behavior must remain compatible. |
| Card Animations damage lifecycle | Damage bursts are reused if landed; this story does not re-own animation queues. |

This story owns board/PlayArea presentation and should not run concurrently with
workers editing the same board hierarchy files.

---

## Acceptance Criteria

- [ ] **AC1 -- Board envelope is dominant**: Across canonical viewports, the
  board occupies the central PlayArea and remains readable before panels, hand,
  or debug overlays.
- [ ] **AC2 -- Edge regions do not compete**: Top chrome, hand fan, event rail,
  and primary CTA occupy predictable edge regions without covering the cell
  matrix unless a hard modal is active.
- [ ] **AC3 -- Targeting dim state exists**: During active targeting/placement,
  non-target board regions are visibly dimmed while the source card and legal
  targets remain readable.
- [ ] **AC4 -- Valid path/rings are explicit**: Valid target cells render
  connected path/range segments or endpoint rings that are distinguishable from
  idle spawn-range highlights.
- [ ] **AC5 -- Invalid target state is distinct**: Invalid targets use a subdued
  or warning treatment that cannot be mistaken for a valid destination.
- [ ] **AC6 -- Source-card link preserved**: When targeting begins from a hand
  card or inspected card, the board feedback visually links to that source
  without blocking the primary CTA.
- [ ] **AC7 -- Authority boundary preserved**: Targeting visuals are derived
  from existing client mirrors/snapshots/messages and never decide legality.
- [ ] **AC8 -- Z-order follows specs**: Board sprites, targeting overlays,
  hover cards, glossary panels, and hard modals respect ADR-021 plus
  `docs/ux/board-rendering-spec.md`.
- [ ] **AC9 -- QA snapshot exposes state**: Snapshot/debug output includes board
  envelope, active targeting state, valid target count, invalid target count,
  path segment count, endpoint ring count, and overlap booleans.
- [ ] **AC10 -- Evidence captured**: Browser/WASM captures show idle board,
  active valid targeting, invalid targeting, and any existing damage feedback at
  minimum and target viewports.
- [ ] **AC11 -- No release/proxy claim**: Completion notes preserve dev-only
  proxy boundaries and final-art accepted-risk posture.

---

## Worker Contract

1. Worktree slug: `work/s19-br-playarea-hierarchy-targeting-feedback`.
2. Activate `liv-bevy-018` before touching Bevy/Rust files.
3. Read PROMPT 1265/1266/1267, ADR-021, and `docs/ux/board-rendering-spec.md`.
4. Keep validation server-authoritative and presentation read-only.
5. Run targeted board-rendering/hand-bridge tests plus browser/WASM evidence.
6. Do not copy Krosmaga assets and do not modify sprint/session state.

