# Story 007: S19-PRES-RESULT-MULLIGAN-OVERLAY-CHROME-001 -- Result + Mulligan-Style Overlay Chrome

> **Epic**: Presentation Layer
> **Story ID**: `S19-PRES-RESULT-MULLIGAN-OVERLAY-CHROME-001`
> **Status**: Draft -- future Sprint 19 candidate; NOT activated
> **Layer**: Presentation / Cross-surface overlays
> **Type**: UI + Integration + Visual evidence
> **Sprint**: Future Sprint 19 implementation wave; depends on result-screen MVP and overlay primitives
> **Authored**: 2026-05-18 by PROMPT 1280
> **Authoring source-of-truth**: local `HEAD@051b59d`; workspace had unrelated dirty code/runtime files that were not touched
> **Source reports**: PROMPT 1265, 1266, 1267
> **Estimated effort**: ~0.6d

---

## Status / No-Claim Banner

This story is a future candidate only. It does not activate Sprint 19 and does
not claim release readiness, final-art completion, Standard-tier accessibility
completion, playtest validation, full game completion, or closure of
`PAW-TD-*-a`.

CCGS does not currently have a Krosmaga-style mulligan game state. This story
uses the mulligan footage only as overlay chrome/layout reference for CCGS
decision overlays such as DraftInitial card choice. It does not add a mulligan
mechanic, opening-hand replacement flow, AP reserve card, or ranked meta
progression.

---

## Source Findings

PROMPT 1266 describes Krosmaga decision/result overlays as:

- Full-screen dim and blur/defocus with the board still spatially visible.
- Large card rows inside a modal decision band.
- CTA below the decision content and centered.
- Result flow with emotional outcome first and detailed accounting second.
- Large glossy blue primary buttons; readouts remain distinct from buttons.

PROMPT 1265 warns that CCGS should prioritize its own objective reveal,
rematch/return needs, and match result semantics rather than Krosmaga ranked
progression. PROMPT 1267 maps result, mulligan, shared panel, button, rank-badge,
victory hero, and result audio candidates as dev-only proxy material after the
provenance boundary exists.

---

## Scope

### In Scope

- Polish the result-screen chrome above the frozen board/HUD:
  stronger scrim/blur posture, outcome banner, large primary CTA, secondary
  return/history/accounting area where already supported by data.
- Preserve the result-screen MVP data contract: no new server result contract,
  no rematch protocol, and alive opponent objectives remain `Unknown` unless a
  separate authority story exists.
- Add a decision-overlay chrome treatment for DraftInitial / opening card-choice
  surfaces that uses large full-detail cards and a centered CTA below the card
  row.
- Use a stronger modal scrim value only where documented and tested; do not
  silently rewrite global overlay tokens without a token/story decision.
- Distinguish primary CTAs from read-only chips and status labels.
- Add QA snapshot/debug fields for overlay type, scrim stage, CTA bounds, card
  row bounds, result/accounting step, and focus target.
- Capture browser/WASM evidence for result and DraftInitial decision overlays at
  minimum and target viewports.

### Out Of Scope

- Adding a new CCGS mulligan phase or opening-hand replacement mechanic.
- Krosmaga ranked XP/progression semantics.
- New server-authoritative result fields.
- Rematch implementation.
- Final art, result audio approval, or release approval.
- Manual/browser two-client GAME_OVER closure unless separately evidenced.

---

## Dependencies And Parallelism

| Dependency | Required posture |
|---|---|
| Presentation Story 006 Result Screen MVP | Must be Done before polishing result chrome. |
| UI overlay/panel primitives | Reuse modal/scrim/panel primitives where landed. |
| `S19-UI-CARD-RENDERING-FIDELITY-HOVER-GLOSSARY-001` | Required if decision cards use shared full-detail card anatomy. |
| `S18-PAW-KROSMAGA-DEV-PROXY-PACK-BOUNDARY-001` | Required before local dev-proxy result/panel/button use. |

This story is cross-surface. Assign it after result-screen MVP and overlay
primitive work are stable, and avoid concurrent edits to result-screen or
DraftInitial overlay files.

---

## Acceptance Criteria

- [ ] **AC1 -- Result chrome has clear hierarchy**: Result overlay presents
  outcome first, then detail/accounting if data exists, without obscuring the
  frozen board beyond the chosen scrim/blur treatment.
- [ ] **AC2 -- Result CTA is dominant**: Return/Acknowledge primary CTA is large,
  centered or bottom-centered, visually filled, and distinct from status chips.
- [ ] **AC3 -- Result data contract preserved**: No new server result protocol,
  rematch protocol, or alive-objective reveal is introduced by this story.
- [ ] **AC4 -- DraftInitial decision overlay uses large cards**: The card-choice
  overlay uses large readable full-detail cards and a centered CTA below the
  decision content.
- [ ] **AC5 -- No mulligan semantic drift**: Labels/copy and tests do not imply
  a Krosmaga mulligan mechanic unless a separate game-design story has added
  one.
- [ ] **AC6 -- Scrim/blur is token-disciplined**: Any stronger scrim value is
  named, documented, tested, and scoped to this overlay class or routed through
  the established overlay-token story path.
- [ ] **AC7 -- Focus/fallback behavior preserved**: Keyboard/focus fallback from
  result-screen MVP remains valid; missing result data still renders a safe
  fallback.
- [ ] **AC8 -- QA snapshot exposes overlay state**: Snapshot/debug output records
  overlay kind, scrim stage, CTA bounds, card row bounds, active result step, and
  focus target.
- [ ] **AC9 -- Evidence captured**: Browser/WASM captures show result overlay
  and DraftInitial decision overlay at minimum and target viewports, including
  reduced-motion/no-flash posture if animation is used.
- [ ] **AC10 -- No release/proxy claim**: Completion notes preserve dev-only
  proxy boundaries, no manual/browser GAME_OVER claim, and final-art accepted
  risk.

---

## Worker Contract

1. Worktree slug: `work/s19-pres-result-mulligan-overlay-chrome`.
2. Activate `liv-bevy-018` before touching Bevy/Rust files.
3. Read PROMPT 1265/1266/1267 and Presentation Story 006.
4. Preserve server-authoritative result semantics and MVP fallback behavior.
5. Run targeted presentation/result + shop/decision overlay tests and visual
   evidence.
6. Do not copy Krosmaga assets and do not modify sprint/session state.

