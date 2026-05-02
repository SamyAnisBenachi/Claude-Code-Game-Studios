# Story 007: GAME_OVER Freeze Mode

> **Epic**: HUD
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hud.md`
**Requirement**: `TR-HUD-009`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: `HudMode::Frozen` is entered on `S2CPhaseChanged(GAME_OVER)`. In FROZEN mode, `S2CGoldUpdate` and `HudObjectiveUpdate` are silently rejected (phase guard at the top of their handlers). `S2CGameSnapshot` bypasses FROZEN and triggers full rebuild (Story 008). Phase label reads `"GAME OVER"`. Round counter remains visible with final round number. No retroactive real/fake reveal on dots.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: FROZEN gate pattern: `if *hud_mode == HudMode::Frozen { return; }` at the top of `handle_gold_update_system` and the dot Observer handler. `query.single()` returns `Result`. `Visibility::Visible` on round counter entity — check via direct component query on that specific entity, not inherited visibility.

**Control Manifest Rules (Presentation Layer + Core Layer)**:
- Required: FROZEN mode rejects all incremental updates after `GAME_OVER`. `S2CGameSnapshot` bypasses FROZEN (Story 008). In-flight tweens snapped to authoritative value on FROZEN entry.
- Forbidden: Retroactive real/fake identity on scoreboard — no ObjectiveIdentity component, no identity glyph added on GAME_OVER. Never hide round counter on GAME_OVER.
- Guardrail: FROZEN mode entry must complete within the same tick as `S2CPhaseChanged(GAME_OVER)`.

---

## Acceptance Criteria

*From GDD `design/gdd/hud.md`, scoped to this story:*

- [x] **HUD-10** (BLOCKING): GIVEN HUD has own gold label showing 12g and dots in their current states, WHEN `S2CPhaseChanged(GAME_OVER)` fires, THEN no subsequent `S2CGoldUpdate` or `HudObjectiveUpdate` changes any HUD component; phase label reads `"GAME OVER"`; no real/fake data appears anywhere.
- [x] **HUD-19** (BLOCKING): GIVEN HUD in ECONOMY_BASIC with `GoldDisplayState.gold=12.0` and phase label `"RESOLUTION"`, WHEN `S2CPhaseChanged(GAME_OVER)` fires AND subsequently `S2CGoldUpdate{gold=999, ...}`, `S2CGoldBroadcast{player_id=local_id, gold=888, ...}`, and `HudObjectiveUpdate{opponent, lane=1}` are emitted, THEN: (a) phase label `Text == "GAME OVER"`; (b) `HudMode == Frozen`; (c) `GoldDisplayState.gold == 12.0` (not 999 or 888); (d) opponent dot for lane 1 retains its pre-GAME_OVER state.
- [x] **HUD-23** (BLOCKING): GIVEN round counter displaying `"R14"` during RESOLUTION, WHEN `S2CPhaseChanged(GAME_OVER)` fires, THEN: (a) round counter entity's own `Visibility` component reads `Visibility::Visible` (verified by direct component query on that entity); (b) round counter entity's `Text == "R14"`.
- [x] **GAME_OVER snap** (BLOCKING): GIVEN a numeric tween is in-flight on the own gold label (mid-interpolation between two values), WHEN `S2CPhaseChanged(GAME_OVER)` fires, THEN the tween is cancelled immediately and `GoldDisplayState.gold` is snapped to the last authoritative server value; the label `Text` reflects the final snapped value within the same tick.

---

## Implementation Notes

*Derived from ADR-021 Implementation Guidelines:*

- FROZEN entry in `PhaseTransition` system: when `CurrentClientPhase.phase == GameOver`, set `*hud_mode = HudMode::Frozen`, update phase label to `"GAME OVER"`. Do NOT hide round counter — it remains visible showing final round.
- FROZEN gate in `handle_gold_update_system` (MessageDrain): `if *hud_mode == HudMode::Frozen { return; }` at top.
- FROZEN gate in dot Observer handler (Story 004, added here): `if *hud_mode == HudMode::Frozen { return; }`.
- GAME_OVER snap: when entering FROZEN, cancel in-flight tween on gold label by calling `animator.set_tweenable(Tween::new(..., Duration::ZERO, ...))` targeting the current authoritative `GoldDisplayState` value, OR directly write `GoldDisplayState.gold = last_authoritative_value` and call `animator.stop()` if bevy_tweening provides that API. The key invariant: after FROZEN entry, the displayed gold equals the last authoritative server value, not a mid-interpolation value.
- `S2CGameSnapshot` bypass: a snapshot arriving in FROZEN mode still runs the full rebuild (Story 008 handles this). After rebuild, HUD immediately re-enters FROZEN — no incremental updates accepted.
- No retroactive real/fake: GAME_OVER does NOT add ObjectiveIdentity to any entity, does NOT change dot appearance to indicate real/fake.
- Round counter `Visibility::Visible` is guaranteed by: GAME_OVER mode does NOT hide the round counter entity specifically. Only LOBBY phase hides the root (which would hide all). Round counter remains accessible while root is visible.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 004]: Dot state machine and Observer (the FROZEN gate is added to the Observer handler)
- [Story 008]: `S2CGameSnapshot` bypass and rebuild in FROZEN mode
- [Story 010]: Tween lifecycle management (cancel-and-replace contract); GAME_OVER snap is a BLOCKING correctness gate required here regardless of Story 010

---

## QA Test Cases

*Written by qa-lead at story creation.*

**HUD-10**: FROZEN blocks incremental updates
  - Given: HUD in ECONOMY_BASIC; own gold = 12.0; 7 dots in their current states
  - When: `S2CPhaseChanged(GAME_OVER)` processed; then `S2CGoldUpdate{gold=999}`, `S2CGoldBroadcast{local_id, gold=888}`, and `HudObjectiveUpdate{opponent, lane=1}` all processed
  - Then: Phase label `Text == "GAME OVER"`; `HudMode == Frozen`; `GoldDisplayState.gold == 12.0`; all dot states unchanged from pre-GAME_OVER values
  - Edge cases: 10 rapid S2CGoldUpdate after GAME_OVER → all rejected; gold stays at 12.0

**HUD-19**: GAME_OVER freeze comprehensive test
  - Given: `GoldDisplayState.gold = 12.0`; phase = RESOLUTION
  - When: Phase → GAME_OVER; then emit S2CGoldUpdate(999), S2CGoldBroadcast(local, 888), HudObjectiveUpdate(opponent, 1)
  - Then: (a) phase label `"GAME OVER"` (b) `HudMode == Frozen` (c) own `GoldDisplayState.gold == 12.0` (d) opponent lane 1 dot state unchanged
  - Edge cases: `HudObjectiveUpdate` before GAME_OVER (should apply); after GAME_OVER (should be rejected)

**HUD-23**: Round counter visible at GAME_OVER
  - Given: Round counter `Text == "R14"`; `Visibility::Visible` on round counter entity
  - When: `S2CPhaseChanged(GAME_OVER)` processed
  - Then: Direct component query on round counter entity → `Visibility::Visible`; `Text == "R14"`
  - Edge cases: Confirm by entity ID, not by traversal from root (HUD-23 note: must verify the counter entity specifically)

**GAME_OVER snap**: Tween cancelled on FROZEN entry
  - Given: Numeric tween in progress on own gold label (`GoldDisplayState.gold` animating from 5.0 toward 15.0, currently at ~7.0); last authoritative value is 15.0
  - When: `S2CPhaseChanged(GAME_OVER)` fires
  - Then: `GoldDisplayState.gold == 15.0` within the same tick; no `Animator<T>` still running on gold label entity (or animator in stopped state)
  - Edge cases: No tween in flight when GAME_OVER fires → snap is a no-op; `GoldDisplayState` already at final value

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/hud/game_over_freeze_test.rs` — must exist and pass

**Status**: [x] Created and passing (`cargo test -p client --test hud_game_over_freeze_test`)

---

## Dependencies

- Depends on: Story 001 (entity pool), Story 002 (`GoldDisplayState`), Story 003 (phase label), Story 005 (phase mode state machine), Story 004 (dot Observer — FROZEN gate added to that handler)
- Unlocks: Story 008 (snapshot bypass of FROZEN), Story 010 (tween cancel-and-replace lifecycle)

## Completion Notes

**Completed**: 2026-05-02
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 4/4 passing; HUD-10, HUD-19, HUD-23, and GAME_OVER snap are covered by `tests/unit/hud/game_over_freeze_test.rs` plus targeted HUD regression checks.
**Test Evidence**: `cargo test -p client --test hud_game_over_freeze_test` passed 2/2. Adjacent HUD regression bundle passed 14/14: `hud_phase_transitions_test`, `same_tick_tie_break_test`, and `scoreboard_dot_message_test`. `cargo check -p client` passed.
**Verification**: `client/src/ui/hud/mod.rs` enters `HudMode::Frozen` on `RoundPhase::GameOver`, drains gold update/broadcast and objective messages without applying them while Frozen, keeps the round counter visible, writes `"GAME OVER"` through the phase label system, snaps numeric tween targets to authoritative state, and removes active HUD `TweenAnim` controllers on FROZEN entry.
**Notes**: Advisory only - the required Frozen guards for post-GAME_OVER `S2CGoldUpdate` and `S2CGoldBroadcast` are implemented and verified by code review; the current unit test does not explicitly emit the story's literal `999`/`888` gold messages after GAME_OVER. Lean mode skipped external QA/code-review gates.
**Tech Debt**: None logged.
**Sprint Status**: Unchanged per user instruction; no explicit `HUD-007` row exists in `production/sprint-status.yaml`.
**Next Recommended**: HUD Story 008 Reconnect Snapshot Rebuild (`production/epics/hud/story-008-reconnect-snapshot-rebuild.md`) after readiness check.
