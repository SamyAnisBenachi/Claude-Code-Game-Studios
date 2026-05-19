# Story 024: S19-UI-HAND-FAN-KROSMAGA-LAYOUT-001 -- Krosmaga-Style Hand Fan Layout

> **Epic**: Hand UI
> **Story ID**: `S19-UI-HAND-FAN-KROSMAGA-LAYOUT-001`
> **Status**: Draft -- future Sprint 19 candidate; NOT activated
> **Layer**: Presentation / Hand UI
> **Type**: UI + Integration
> **Sprint**: Future Sprint 19 implementation wave; depends on Sprint 18 card/play-area foundations
> **Authored**: 2026-05-18 by PROMPT 1280
> **Authoring source-of-truth**: local `HEAD@051b59d`; workspace had unrelated dirty code/runtime files that were not touched
> **Source reports**: PROMPT 1265, 1266, 1267
> **Estimated effort**: ~0.5d

---

## Status / No-Claim Banner

This story is a future candidate only. It does not activate Sprint 19 and does
not claim release readiness, final-art completion, Standard-tier accessibility
completion, playtest validation, full game completion, or closure of
`PAW-TD-*-a`.

Krosmaga is used as a layout/reference source. This story does not import or
approve Krosmaga assets.

---

## Source Findings

PROMPT 1266 identifies the local hand fan as a bottom-edge gameplay surface, not
a flat debug row:

- Resting hand cards occupy roughly `6%-9%` viewport width and `15%-18%`
  viewport height in the source footage.
- The local hand region occupies roughly `6%-62%, 84%-100%` in the 16:10
  reference and should translate to a stable bottom band in CCGS.
- Cards overlap by about `25%-35%` of card width.
- Hovered cards rise, glow, and remain connected to card inspect/glossary.
- Locked/passive states dim the hand but preserve planning readability.

PROMPT 1265 confirms CCGS already has the logical hand surface; the gap is
presentation richness and continuity. PROMPT 1267 maps card backs/frames and
card movement audio as dev-only proxy candidates after the provenance boundary
exists.

---

## Scope

### In Scope

- Re-layout the local hand fan as a bottom-edge overlapping fan with stable
  height across canonical viewports.
- Preserve card portrait aspect ratio and the shared card anatomy from
  `S19-UI-CARD-RENDERING-FIDELITY-HOVER-GLOSSARY-001`.
- Apply deterministic slot position, overlap, lift, scale, and z-order formulas
  for hand sizes 0 through 10.
- Add hover/focus lift and glow that cooperates with the shared card inspect
  overlay.
- Preserve PLACEMENT drag, staged-card, passive, passive-locked, submitted, and
  reconnect rebuild behavior.
- Ensure the hand fan does not overlap the bottom-right primary action cluster
  or the board PlayArea in supported viewports.
- Add snapshot/debug fields for fan bounds, card count, focused slot, overlap
  factor, and CTA overlap status.

### Out Of Scope

- Any server hand-state change.
- Card acquisition rules or hand-size rules.
- New mulligan/deck/discard semantics.
- Final art or release approval.
- Replacing the existing drag/drop authority model.

---

## Dependencies And Parallelism

| Dependency | Required posture |
|---|---|
| `S19-UI-CARD-RENDERING-FIDELITY-HOVER-GLOSSARY-001` | Should land first if shared card anatomy changes are non-trivial. |
| `S18-UI-PLAY-AREA-CONTAINER-001` | Should be Done or consciously superseded so hand/board strip budgets are stable. |
| Hand UI Stories 020, 022, 023 | Must preserve drag-state visuals, mana-preview affordance, and idle-playable affordance semantics. |
| ADR-021 | Hand UI stays bevy_ui; drag sprite remains bevy_ui above board content. |

This story touches Hand UI layout and should not run concurrently with another
worker editing `client/src/ui/hand/**`.

---

## Acceptance Criteria

- [ ] **AC1 -- Bottom fan bounds stable**: At 1366x768, 1920x1080, 1920x1200,
  1280x960, 2560x1080, and 3840x2160, the local hand stays in the bottom
  action band and does not resize the PlayArea.
- [ ] **AC2 -- Overlap formula documented and tested**: Hand sizes 1 through 10
  use a deterministic overlap factor targeting `25%-35%` card width overlap,
  with a fallback clamp for narrow viewports.
- [ ] **AC3 -- Portrait cards stay aspect-fit**: Cards preserve portrait ratio
  and consume the shared card primitive zones without text/art stretching.
- [ ] **AC4 -- Hover/focus lift is visible**: Hovered/focused cards rise above
  neighbors, glow, and become the source for the card inspect overlay without
  disturbing neighboring slot layout.
- [ ] **AC5 -- CTA overlap blocked**: The fan never covers the primary
  Submit/Ready/Bid/action cluster; if viewport pressure exists, fan overlap
  increases before card text or CTA size is reduced.
- [ ] **AC6 -- Passive states remain legible**: Passive/locked/submitted hand
  states dim or desaturate cards without losing title/cost/stat readability.
- [ ] **AC7 -- Drag semantics preserved**: PLACEMENT drag source, ghost preview,
  staged-card hand marker, reserve-mana split strip, and submit validation still
  work after the layout change.
- [ ] **AC8 -- Reconnect rebuild stable**: Snapshot/reconnect hand rebuild
  produces the same final fan positions as initial live rendering for the same
  hand contents.
- [ ] **AC9 -- QA snapshot fields added**: Snapshot/debug output includes fan
  bounds, hand count, focused slot, overlap factor, and a boolean proving no CTA
  overlap.
- [ ] **AC10 -- Visual evidence captured**: Browser/WASM captures show 0, 1, 5,
  and 10-card hands with hover/focus at minimum and target viewports.
- [ ] **AC11 -- No release/proxy claim**: Completion notes preserve the dev-only
  Krosmaga boundary and final-art accept-risk posture.

---

## Worker Contract

1. Worktree slug: `work/s19-ui-hand-fan-krosmaga-layout`.
2. Activate `liv-bevy-018` before touching Bevy/Rust files.
3. Read PROMPT 1265/1266/1267 and Hand UI Stories 020/022/023.
4. Keep state client-presentation-only and server-authority-neutral.
5. Run targeted hand-ui integration tests plus browser/WASM viewport evidence.
6. Do not copy Krosmaga assets and do not modify sprint/session state.
