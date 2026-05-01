# Story 003: Phase Label, Round Counter, and Instantaneous Transitions

> **Epic**: HUD
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hud.md`
**Requirement**: `TR-HUD-003`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: `S2CPhaseChanged` (Lightyear) is drained ONLY by the shared `phase_sink_system` registered in `PresentationPlugin`. `HudPlugin` reads phase state from `Res<CurrentClientPhase>` — never directly from `MessageReceiver<S2CPhaseChanged>`. HUD phase-transition systems run in `PhaseTransition` set. Phase label and round counter updates are instantaneous (no tween, no `Animator`).

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `MessageReceiver<S2CPhaseChanged>` must NOT be registered in `HudPlugin` — `PresentationPlugin` owns the single drain. HUD reads `Res<CurrentClientPhase>`. Phase transition system in `PhaseTransition` set. `Text` writes are direct (not via `Animator<T>`). `query.single()` returns `Result`.

**Control Manifest Rules (Presentation Layer)**:
- Required: Sub-plugins read `Res<CurrentClientPhase>` — never `MessageReceiver<S2CPhaseChanged>` directly. Phase handlers in `PhaseTransition` set.
- Forbidden: Never register `MessageReceiver<S2CPhaseChanged>` in more than one system (first drain consumes all). Phase label must not have `Animator<T>` attached.
- Guardrail: Phase label and round counter must update within the same ECS tick as the phase change — no deferred write.

---

## Acceptance Criteria

*From GDD `design/gdd/hud.md`, scoped to this story:*

- [ ] **HUD-05** (BLOCKING): GIVEN HUD is visible, WHEN `S2CPhaseChanged` fires for each RSM phase, THEN phase label `Text` reads: `DRAFT_INITIAL → "DRAFT INITIAL"`, `DRAFT_SHOP → "DRAFT"`, `DRAFT_AUCTION → "AUCTION"`, `PLACEMENT → "PLACEMENT"`, `RESOLUTION → "RESOLUTION"`, `GAME_OVER → "GAME OVER"`. LOBBY: HUD hidden (no phase label visible).
- [ ] **HUD-22** (BLOCKING): GIVEN HUD is in any visible non-LOBBY phase, WHEN `S2CPhaseChanged{round_number=9}` arrives, THEN round counter `Text` reads exactly `"R9"` (not `"9"`, not `"Round 9"`, not `"R09"`).
- [ ] **HUD-12b — label portion** (BLOCKING): GIVEN HUD is in any visible mode, WHEN `S2CPhaseChanged` fires for any phase, THEN phase label `Text` and round counter `Text` reflect the new values within the same ECS tick. No `Animator<T>` component is attached to phase label or round counter entities.
- [ ] **HUD-11 — timer boundary** (BLOCKING): The `S2CPhaseChanged` handler uses the pattern `let S2CPhaseChanged { phase, round_number, .. } = msg;`, discarding `timer_duration_ms`. No HUD entity receives timer data at any point.

---

## Implementation Notes

*Derived from ADR-021 Implementation Guidelines:*

- HUD's `PhaseTransition` system reads `Res<CurrentClientPhase>` (written by `phase_sink_system` in `PresentationPlugin`). On phase change, write the phase label string and round counter string directly to their `Text` components.
- Phase label string mapping (exact, from GDD Rule 5):
  | Phase | Label string |
  |---|---|
  | `DRAFT_INITIAL` | `"DRAFT INITIAL"` |
  | `DRAFT_SHOP` | `"DRAFT"` |
  | `DRAFT_AUCTION` | `"AUCTION"` |
  | `PLACEMENT` | `"PLACEMENT"` |
  | `RESOLUTION` | `"RESOLUTION"` |
  | `GAME_OVER` | `"GAME OVER"` |
  | `LOBBY` | (HUD hidden — label unchanged from last value) |
- Round counter: `format!("R{}", current.round)` — zero-pad NOT applied.
- `timer_duration_ms` discard: destructure `let S2CPhaseChanged { phase, round_number, .. } = msg;` in the `phase_sink_system` handler. HUD never reads `timer_duration_ms`.
- Phase label and round counter entities must never have an `Animator<T>` component — text replace in place (direct `Text` write).
- Mode transitions (ECONOMY_BASIC / ECONOMY_AUCTION / FROZEN / HIDDEN) are gated in other stories (005, 006, 007). This story only covers text strings.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 005]: Phase visibility transitions (HIDDEN → ECONOMY_BASIC on DRAFT_INITIAL, etc.)
- [Story 006]: ECONOMY_AUCTION gold label format switch triggered by phase change
- [Story 007]: GAME_OVER freeze mode (FROZEN behavior beyond the label string)
- [Story 004]: HUD-12b dot portion (dot instantaneous flip on `HudObjectiveUpdate`)

---

## QA Test Cases

*Written by qa-lead at story creation.*

**HUD-05**: Phase label strings for all phases
  - Given: HUD initialized and visible (phase != LOBBY)
  - When: `CurrentClientPhase.phase` set to each of the 6 active phases in sequence; `PhaseTransition` runs each tick
  - Then: Phase label `Text` matches the exact string for each phase per the mapping table above
  - Edge cases: `GAME_OVER` → `"GAME OVER"` (two words, space between); `DRAFT_INITIAL` → `"DRAFT INITIAL"` (two words); unknown phase variant → log warning, label unchanged

**HUD-22**: Round counter format
  - Given: HUD visible, `CurrentClientPhase.round = 9`
  - When: `PhaseTransition` system runs
  - Then: Round counter entity `Text == "R9"` (exactly)
  - Edge cases: `round = 1` → `"R1"`; `round = 20` → `"R20"` (not `"R020"`); `round = 0` → `"R0"` (edge case — verify no panic)

**HUD-12b (label portion)**: Instantaneous update in same tick
  - Given: Phase label `Text = "DRAFT"`, round counter `Text = "R5"`
  - When: `CurrentClientPhase` updated to `{phase: PLACEMENT, round: 6}` in the same tick; `PhaseTransition` runs
  - Then: Phase label `Text == "PLACEMENT"` and round counter `Text == "R6"` within that tick; no `Animator<T>` component on either entity
  - Edge cases: Two phase changes in the same tick (reconnect artifact) → last-write-wins; no crash

**HUD-11 (timer boundary)**: timer_duration_ms discarded
  - Given: `S2CPhaseChanged{phase: PLACEMENT, round_number: 3, timer_duration_ms: 60000}` processed
  - When: Query all HUD `Text`/`TextSpan` components
  - Then: No entity text contains `"60000"`, `"60s"`, or any representation of the timer value
  - Edge cases: `timer_duration_ms = 0` → same: no HUD entity text changes

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/hud/phase_label_round_counter_test.rs` — must exist and pass

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: Story 001 (entity pool, `HudEntities` resource)
- Unlocks: Story 005 (phase transitions build on the phase strings), Story 007 (GAME_OVER builds on phase label)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 4/4 passing (HUD-05, HUD-22, HUD-12b label/round portion, HUD-11 timer boundary).
**Deviations**: Advisory only - worker commit `52a3605` exists on `work/hud-003-phase-label-round-counter` but is not a direct ancestor of current `main`; main integration commit `ce76a88` is included in `main` and carries the HUD-003 source/test implementation. Advisory only - `TR-HUD-003` currently maps to older registry AC metadata while this story verifies current story-scoped GDD criteria HUD-05, HUD-22, HUD-12b, and HUD-11; current behavior is covered by tests. Advisory only - the shared `PresentationPlugin`/`phase_sink_system` wiring is not present yet, so HUD-003 verifies the HUD sub-plugin contract through `Res<CurrentClientPhase>` and the phase-message application helper.
**Test Evidence**: Logic test file at `tests/unit/hud/phase_label_round_counter_test.rs`; `cargo test -p client --test hud_phase_label_round_counter_test` passed 6/6. Regression `cargo test -p client --test hud_plugin_scaffold_test --test hud_gold_mana_display_test --test hud_phase_label_round_counter_test` passed 16/16. `cargo check -p client` passed.
**Code Review**: Skipped - Lean mode.
