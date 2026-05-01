# Story 005: Phase Transitions and RESOLUTION Persistence

> **Epic**: HUD
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hud.md`
**Requirement**: `TR-HUD-006`, `TR-HUD-003`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: HUD phase-transition system runs in `PhaseTransition` set, reads `Res<CurrentClientPhase>`. Mode enum (`HudMode`) drives visibility and format: `HIDDEN` (LOBBY), `ECONOMY_BASIC` (most phases), `ECONOMY_AUCTION` (Story 006), `FROZEN` (Story 007). All presentation sub-plugins run their `PhaseTransition` systems in the same set, guaranteeing atomic phase-boundary behaviour — HUD and sister UIs change state on the same frame.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `Visibility::Visible` / `Visibility::Hidden` — insert via `commands.entity(e).insert(Visibility::Visible)` or direct component mutation. `InheritedVisibility` is auto-computed — do not manually set it. Resource `HudMode` must be a proper `Resource` type with `#[derive(Resource)]`.

**Control Manifest Rules (Presentation Layer)**:
- Required: Phase handlers in `PhaseTransition` set. `HudMode` resource is the single source of truth for current mode. Visibility changes via `Visibility` component — never modify `InheritedVisibility` directly.
- Forbidden: Never read `MessageReceiver<S2CPhaseChanged>` in HudPlugin — read `Res<CurrentClientPhase>`.
- Guardrail: All `PhaseTransition` systems across all sub-plugins run before any `MessageDrain` systems — deterministic cross-sub-plugin phase sync.

---

## Acceptance Criteria

*From GDD `design/gdd/hud.md`, scoped to this story:*

- [ ] **HUD-15** (BLOCKING): GIVEN HUD in HIDDEN mode (`Visibility::Hidden`), WHEN `S2CPhaseChanged(DRAFT_INITIAL)` fires, THEN HUD root becomes `Visibility::Visible`, `HudMode = ECONOMY_BASIC`, phase label reads `"DRAFT INITIAL"`, round counter shows the current round number, and all 10 dots are in ALIVE state.
- [ ] **HUD-16** (BLOCKING): GIVEN HUD in ECONOMY_BASIC mode after a non-auction RESOLUTION, WHEN `S2CPhaseChanged(DRAFT_SHOP)` fires, THEN phase label reads `"DRAFT"`, both gold labels remain in ECONOMY_BASIC format (`"Xg"`, no parenthetical), `HudMode = ECONOMY_BASIC`.
- [ ] **HUD-18** (BLOCKING): GIVEN HUD in ECONOMY_BASIC mode with phase label `"PLACEMENT"`, WHEN `S2CPhaseChanged(RESOLUTION)` fires, THEN phase label reads `"RESOLUTION"`, HUD root remains `Visibility::Visible`, no HUD zone is hidden, and gold label values update when subsequent `S2CGoldUpdate` messages arrive.
- [ ] **HUD-09** (BLOCKING): GIVEN HUD in any visible mode, WHEN `S2CPhaseChanged(RESOLUTION)` fires, THEN HUD root `Node` has `Visibility::Visible`; `HudMode = ECONOMY_BASIC`; `GoldDisplayState.gold` on own gold entity updates correctly when `S2CGoldUpdate` is processed. (Sister-UI hiding requires a cross-plugin integration test — not verifiable in `HudPlugin` isolation.)

---

## Implementation Notes

*Derived from ADR-021 Implementation Guidelines:*

- `HudMode` resource with variants: `Hidden`, `EconomyBasic`, `EconomyAuction`, `Frozen`.
- Phase-transition system in `PhaseTransition` set:
  ```rust
  fn hud_phase_transition_system(
      current: Res<CurrentClientPhase>,
      mut hud_mode: ResMut<HudMode>,
      hud_entities: Res<HudEntities>,
      mut visibility: Query<&mut Visibility>,
  ) {
      match current.phase {
          ClientPhase::Lobby => {
              *hud_mode = HudMode::Hidden;
              // hide root
          }
          ClientPhase::DraftInitial | ClientPhase::DraftShop | ClientPhase::Placement | ClientPhase::Resolution => {
              // ECONOMY_AUCTION exit: if transitioning FROM DRAFT_AUCTION, revert gold labels to BASIC_FORMAT
              *hud_mode = HudMode::EconomyBasic;
              // show root
          }
          // DRAFT_AUCTION and GAME_OVER handled in Stories 006 and 007
          _ => {}
      }
  }
  ```
- DRAFT_INITIAL is the only transition from HIDDEN → visible. All 10 dots are initialised to ALIVE at session start (Story 001). No dot reset needed on DRAFT_INITIAL unless reconnect (Story 008).
- RESOLUTION persistence: HUD root stays `Visibility::Visible` during RESOLUTION — do NOT hide any zone. This is enforced by the mode staying `ECONOMY_BASIC`.
- ECONOMY_AUCTION exit (DRAFT_SHOP transition): clear the `TextSpan` child text to `""` for both gold labels. See Story 006 for the full AUCTION entry/exit contract — this story only needs the exit direction.
- Gold label format for ECONOMY_BASIC: `"{gold}g"` — no parenthetical. On entering ECONOMY_BASIC from any mode, ensure both gold label `TextSpan` children read `""`.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 006]: DRAFT_AUCTION entry (`ECONOMY_AUCTION` mode — parenthetical format switch)
- [Story 007]: GAME_OVER transition (`FROZEN` mode)
- [Story 004]: Dot ALIVE/DESTROYED state transitions (Observer)

---

## QA Test Cases

*Written by qa-lead at story creation.*

**HUD-15**: LOBBY → DRAFT_INITIAL visibility transition
  - Given: `HudMode = Hidden`; HUD root `Visibility::Hidden`
  - When: `CurrentClientPhase{phase: DraftInitial, round: 1}` set; `PhaseTransition` runs
  - Then: Root `Visibility::Visible`; `HudMode = EconomyBasic`; phase label `Text == "DRAFT INITIAL"`; round counter `Text == "R1"`; all 10 dot state flags == ALIVE
  - Edge cases: Round 2 DRAFT_INITIAL (reconnect) → same transition, round counter shows correct round

**HUD-16**: RESOLUTION → DRAFT_SHOP (non-auction round)
  - Given: `HudMode = EconomyBasic`; gold labels in BASIC format
  - When: `CurrentClientPhase{phase: DraftShop, round: 4}` set (after non-auction RESOLUTION); `PhaseTransition` runs
  - Then: Phase label `Text == "DRAFT"`; own gold label parent `Text` has no parenthetical; opponent gold label parent `Text` has no parenthetical; `HudMode = EconomyBasic`
  - Edge cases: Transition from `EconomyAuction` → `DraftShop` (auction round exit) → TextSpan children must be `""`

**HUD-18**: PLACEMENT → RESOLUTION persistence
  - Given: `HudMode = EconomyBasic`; phase label `"PLACEMENT"`; HUD root visible
  - When: `CurrentClientPhase{phase: Resolution}` set; `PhaseTransition` runs
  - Then: Phase label `Text == "RESOLUTION"`; HUD root `Visibility::Visible`; `HudMode = EconomyBasic`; no zone hidden
  - Edge cases: Confirm `reserve_label` visibility state unchanged (not forcibly hidden during RESOLUTION)

**HUD-09**: RESOLUTION — gold updates accepted
  - Given: HUD in ECONOMY_BASIC, phase = RESOLUTION
  - When: `S2CGoldUpdate{gold=15, current_mana=5, mana_cap=10, reserve_mana=0}` processed during RESOLUTION
  - Then: `GoldDisplayState.gold == 15.0`; mana label `Text == "5 / 10"`
  - Edge cases: Multiple S2CGoldUpdate during RESOLUTION → all processed (no freeze during RESOLUTION — only GAME_OVER freezes)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/hud/phase_transitions_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (entity pool), Story 002 (GoldDisplayState for HUD-09 verification), Story 003 (phase label text for HUD-15)
- Unlocks: Story 006 (ECONOMY_AUCTION builds on the mode state machine), Story 007 (GAME_OVER adds FROZEN mode)
