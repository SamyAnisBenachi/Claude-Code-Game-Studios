# Story 019: S18-HUD-PHASE-CHIP-DISAMBIGUATION-001 -- Unique Phase-Chip Stem Per RoundPhase

> **Epic**: HUD
> **Story ID**: `S18-HUD-PHASE-CHIP-DISAMBIGUATION-001`
> **Status**: Draft -- Sprint 18 candidate / retro paperwork; NOT activated
> **Layer**: Presentation -- HUD phase chip (`client/src/ui/hud/mod.rs`)
> **Type**: Logic (per-phase stem string + reactive label)
> **Authored**: 2026-05-18 by PROMPT 1296 (landed-paperwork story-authoring wave)
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`
> **Implementing PROMPT**: 1250 -- `dev-story(s18-hud-phase-chip-disambiguation): unique phase-chip stem per RoundPhase`
> **Implementing commit**: `97d3d0b`
> **Source audit**: PROMPT 1287 §2 (already-landed inventory); PROMPT 1203 B-1203-DRI-02 / S-10 (phase chip ambiguous across DraftInitial / DraftShop / DraftAuction / Placement / Resolution)

---

## Status / No-Claim Banner

This story is a **retro paperwork stub** authored by PROMPT 1296 covering
work that **already landed on `origin/main` at `97d3d0b`**. PROMPT 1296
makes **no** code, test, Cargo, CI, sprint, QA, or session-state
mutations. Sprint 18 is **NOT activated** by this authoring run. All
standard non-claims preserved verbatim.

---

## Source Finding

**PROMPT 1203 B-1203-DRI-02 / S-10**: the HUD phase chip surfaced a
phase label drawn from a shared stem ("Draft" / "Combat" / "Round"
across several `RoundPhase` variants), making it ambiguous to the
player which sub-phase was active. Players could not tell DraftInitial
from DraftShop from DraftAuction by glancing at the chip; the only
disambiguator was the rest of the screen (the modal / panel actually
visible), which a player who lost focus could miss.

PROMPT 1250 (`dev-story`) replaced the shared stem with a unique
per-`RoundPhase` label so every phase variant produces a distinct
chip string.

---

## Landed Evidence (commit `97d3d0b`, PROMPT 1250)

Files touched by the implementing commit:

| Path | Role |
|------|------|
| `client/src/ui/hud/mod.rs` | Per-phase stem + label-resolution function. |
| `tests/integration/hud/reconnect_snapshot_rebuild_test.rs` | Assertions updated to expect new labels. |
| `tests/integration/hud/text_size_contrast_accessibility_test.rs` | Assertions updated. |
| `tests/unit/hud/phase_label_round_counter_test.rs` | Coverage extended: assertions for every `RoundPhase` variant. |
| `tests/unit/hud/phase_transitions_test.rs` | Assertions updated. |

---

## Acceptance Criteria (evidence-binding, closure-oriented)

- [ ] **AC1 -- Unique stem per RoundPhase**: `client/src/ui/hud/mod.rs`
  contains a label-resolution function returning a distinct string for
  every variant of `RoundPhase` (or equivalent phase enum). No two
  variants produce the same label.
- [ ] **AC2 -- Reactive on phase change**: the phase chip `Text` node
  updates within one frame of a `Res<CurrentClientPhase>` change. No
  one-frame stale-label window survives the transition.
- [ ] **AC3 -- ADR-009 preserved**: the HUD reads `Res<CurrentClientPhase>`;
  it does NOT drain `MessageReceiver<S2CPhaseChanged>` (single-drain
  discipline owned by the shared `phase_sink_system`).
- [ ] **AC4 -- Unit test coverage**:
  `tests/unit/hud/phase_label_round_counter_test.rs` asserts the label
  string for every `RoundPhase` variant.
- [ ] **AC5 -- Reconnect rebuild + accessibility tests PASS**: the
  updated integration tests
  (`tests/integration/hud/reconnect_snapshot_rebuild_test.rs`,
  `tests/integration/hud/text_size_contrast_accessibility_test.rs`)
  PASS at the Sprint 18 activation tip.
- [ ] **AC6 -- Plugin registration + entity counts preserved**: HUD
  plugin remains at its existing registration position;
  `HudEntities` is unchanged (no new pre-pooled top-level entity --
  the phase-chip text node already existed).
- [ ] **AC7 -- /dev-story disposition**: **NOT REQUIRED** if AC1..AC6
  remain satisfied at the Sprint 18 activation tip. If regression,
  `/story-readiness` MUST return NEEDS_WORK; follow-on implementation
  required before closure.

---

## Out of Scope

- Phase-chip colour theming / per-phase tinting. Polish candidate.
- Phase-chip icon glyphs. Out of host module (asset pipeline).
- DraftInitial modal countdown text (story 022 in shop-auction-ui,
  separate landed work).
- Sprint 18 activation, stage advance, gate-check retry.

---

## Authoring Trail

- 2026-05-18 -- PROMPT 1296 -- Retro paperwork stub authored against
  `origin/main@1345c6b`. Files touched: this file (NEW) and
  `production/epics/hud/EPIC.md` (table row added). Implementation
  landed via PROMPT 1250 at `97d3d0b` prior to this authoring; this
  stub does not re-author or alter that work.
