# Story 015: S18-RESOLUTION-COMBAT-MINIMAL-OVERLAY-001 -- Emit Damage Numbers + Kill Markers from Resolution Script

> **Epic**: Board Rendering
> **Story ID**: `S18-RESOLUTION-COMBAT-MINIMAL-OVERLAY-001`
> **Status**: Draft -- Sprint 18 candidate / retro paperwork; NOT activated
> **Layer**: Presentation -- Board Rendering resolution playback (`client/src/presentation/board_rendering.rs`)
> **Type**: Integration (resolution-script consumer + visual overlay assertions)
> **Authored**: 2026-05-18 by PROMPT 1296 (landed-paperwork story-authoring wave)
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`
> **Implementing PROMPTs**: 1231 (`dev-story`) + 1234 (integrate)
> **Implementing commits**: `65307f2` + `92a1d46`
> **Source audit**: PROMPT 1287 §2 (already-landed inventory); PROMPT 1201 HUNT-1201-16 (no Resolution visual feedback)

---

## Status / No-Claim Banner

This story is a **retro paperwork stub** authored by PROMPT 1296 covering
work that **already landed on `origin/main`** at `65307f2` (dev-story)
and `92a1d46` (integrate). PROMPT 1296 makes **no** code, test, Cargo,
CI, sprint, QA, or session-state mutations. Sprint 18 is **NOT
activated** by this authoring run. All standard non-claims preserved
verbatim.

This story scopes the **minimal** visual overlay required to make
Resolution legible. Full polish (animation curves, hit-pause, screen-
shake, kill VFX) is **out of scope** and remains a future Polish
candidate.

---

## Source Finding

**PROMPT 1201 HUNT-1201-16**: at Resolution, the board rendered the
end-state (units re-positioned, damaged or removed) but with no
visible cue tying the change to the combat that caused it. Players
could not tell which unit attacked which, who took damage, or which
units died. The resolution script (`S2CResolutionEvent` log) was
already authoritative and complete; the rendering side was silent.

PROMPT 1231 (`dev-story`) + PROMPT 1234 (integrate) added a minimal
overlay layer driven directly from the existing
`ResolutionEvent::CombatDamage` / `ResolutionEvent::UnitRemoved` (or
equivalent) variants in the resolution script:

- Damage numbers spawn over the receiving unit at the resolution-event
  timestamp, fade out via the existing `AnimQueue` tween lifecycle.
- Kill markers (e.g. a brief "X" or skull glyph) appear at the cell
  of removed units for a short duration before despawn.

No new protocol surface; the overlay consumes the existing replicated
resolution script log per ADR-017.

---

## Landed Evidence (commits `65307f2` + `92a1d46`)

Files touched by the implementing commits:

| Path | Role |
|------|------|
| `client/Cargo.toml` | Cargo wiring for the new test target. |
| `client/src/presentation/board_rendering.rs` | Resolution-event consumer + overlay spawn/despawn systems. |
| `tests/integration/board_rendering/resolution_combat_feedback_test.rs` (NEW) | 411 LOC integration test asserting overlay entities and timing. |

---

## Acceptance Criteria (evidence-binding, closure-oriented)

- [ ] **AC1 -- Damage-number overlay**: `client/src/presentation/board_rendering.rs`
  spawns a transient overlay entity (text or sprite) at the receiving
  unit's cell position on every `ResolutionEvent::CombatDamage` (or
  equivalent damage variant) within the resolution script playback.
  Entity despawns when the `AnimQueue` tween completes.
- [ ] **AC2 -- Kill marker overlay**: a kill marker overlay spawns at
  the removed unit's last cell on every `ResolutionEvent::UnitRemoved`
  (or equivalent removal variant). Marker despawns after its
  configured visible window.
- [ ] **AC3 -- Resolution-script consumer only**: the overlay systems
  consume the existing `S2CResolutionEvent` log per ADR-017; they do
  NOT mutate authoritative state and do NOT introduce a new protocol
  type.
- [ ] **AC4 -- AnimQueue ordering preserved**: overlay spawns
  participate in the same `AnimQueue` ordering as the rest of the
  resolution playback; no out-of-order spawn (e.g. a damage number
  appearing before its triggering combat sub-step).
- [ ] **AC5 -- Integration test PASS**:
  `tests/integration/board_rendering/resolution_combat_feedback_test.rs`
  PASSES at the Sprint 18 activation tip, covering: damage-number
  spawn over the receiving unit, kill-marker spawn at the removed
  unit's last cell, despawn after the tween window, and absence of
  overlay entities outside `Phase::Resolution`.
- [ ] **AC6 -- ADR-021 boundaries preserved**: overlays are world-
  space sprites / text rendered inside `BoardRenderingPlugin`; they
  are NOT bevy_ui nodes. Plugin registration order unchanged.
- [ ] **AC7 -- /dev-story disposition**: **NOT REQUIRED** if AC1..AC6
  remain satisfied at the Sprint 18 activation tip. If regression,
  `/story-readiness` MUST return NEEDS_WORK; follow-on implementation
  required before closure.

---

## Out of Scope

- Final-art kill VFX / particle effects.
- Hit-pause, screen-shake, camera kicks. Polish candidates for a later
  sprint.
- Numeric tween curves, easing variations beyond the minimum needed
  for the overlay to be readable.
- Spell / Trap / Order resolution overlays. This story scopes combat-
  damage and unit-removal only.
- Sprint 18 activation, stage advance, gate-check retry.

---

## Authoring Trail

- 2026-05-18 -- PROMPT 1296 -- Retro paperwork stub authored against
  `origin/main@1345c6b`. Files touched: this file (NEW) and
  `production/epics/board-rendering/EPIC.md` (table row added).
  Implementation landed via PROMPT 1231 + 1234 at `65307f2` + `92a1d46`
  prior to this authoring; this stub does not re-author or alter that
  work.
