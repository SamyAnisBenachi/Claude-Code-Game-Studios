# Story 006: ECONOMY_AUCTION Inline Gold Format and TextSpan Rendering

> **Epic**: HUD
> **Status**: Ready
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hud.md`
**Requirement**: `TR-HUD-001`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Gold labels use Bevy 0.18 multi-span pattern: parent entity carries `Text + TextFont + TextColor` for the `Xg` span (full opacity); child entity carries `TextSpan + TextFont + TextColor` for the ` (Yr)` span (65% opacity, 0.65× scale). In ECONOMY_BASIC mode the child `TextSpan` entity's text is set to `""` (empty string — NOT despawned). On `S2CPhaseChanged(DRAFT_AUCTION)` entry, both gold labels switch to AUCTION_FORMAT.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: Multi-span: parent entity has `Text::new("Xg")` + child entity has `TextSpan::new(" (Yr)")` via `ChildOf`. Setting child span to `""` in ECONOMY_BASIC — NOT despawning the child. `TextFont` size ratio 0.65× on child span. `TextColor` with `.with_alpha(0.65)` on child span. `query.get(entity)` for targeted entity lookup.

**Control Manifest Rules (Presentation Layer)**:
- Required: Child `TextSpan` entity set to `""` in ECONOMY_BASIC (never despawned). Gold format switch is a direct `Text`/`TextSpan` write — no tween, no `Animator`.
- Forbidden: Never despawn + respawn TextSpan children for format changes. Never use `UiImage::new()`.
- Guardrail: Format switch must occur in the same tick as `S2CPhaseChanged(DRAFT_AUCTION)`.

---

## Acceptance Criteria

*From GDD `design/gdd/hud.md`, scoped to this story:*

- [ ] **HUD-17** (BLOCKING): GIVEN HUD in ECONOMY_BASIC with `GoldDisplayState.gold=11` (own) and `GoldDisplayState.gold=8` (opponent), WHEN `S2CPhaseChanged(DRAFT_AUCTION)` fires, THEN `HudMode = ECONOMY_AUCTION`; phase label reads `"AUCTION"`; own gold label parent `Text == "11g"` and its TextSpan child `Text == " (0r)"`; opponent gold label parent `Text == "8g"` and its TextSpan child `Text == " (0r)"`. (Server invariant: `reserved_gold == 0` at auction entry — `0r` is correct.)
- [ ] **HUD-08** (BLOCKING): GIVEN HUD in ECONOMY_BASIC, WHEN `S2CGoldBroadcast{player_id=opponent_id, gold=7, reserved_gold=0}` arrives, THEN opponent gold label reads `"7g"` with no parenthetical suffix. GIVEN HUD in ECONOMY_AUCTION, WHEN `S2CGoldBroadcast{player_id=opponent_id, gold=7, reserved_gold=3}` arrives, THEN opponent gold label parent `Text == "7g"` and child `TextSpan == " (3r)"`.
- [ ] **HUD-29** (BLOCKING): GIVEN HUD in ECONOMY_AUCTION with own `"11g (4r)"` and opponent `"8g (2r)"`, WHEN `S2CPhaseChanged(DRAFT_SHOP)` fires, THEN `HudMode = ECONOMY_BASIC`; own gold label parent `Text == "11g"` and child TextSpan `Text == ""`; opponent gold label parent `Text == "8g"` and child TextSpan `Text == ""`.
- [ ] **HUD-28** (ADVISORY): GIVEN HUD in ECONOMY_AUCTION and `S2CGoldBroadcast{player_id=opponent_id, gold=7, reserved_gold=3}` processed, THEN querying opponent gold parent entity returns `Text == "7g"`; its single child has `TextSpan == " (3r)"`. No top-level HUD entity outside this tree represents opponent reserved gold.

---

## Implementation Notes

*Derived from ADR-021 Implementation Guidelines and GDD Rule 3:*

- `HudMode::EconomyAuction` is entered on `S2CPhaseChanged(DRAFT_AUCTION)` (phase-transition system, `PhaseTransition` set).
- On AUCTION_FORMAT entry: set both gold label parent `Text` to `"{gold}g"` and both child `TextSpan` to `" ({reserved_gold}r)"`. Both will read `" (0r)"` at entry (server invariant).
- On BASIC_FORMAT re-entry: set both gold label child `TextSpan` to `""`. Parent `Text` reads `"{gold}g"`.
- `handle_gold_broadcast_system` (already from Story 002): unconditionally writes `GoldDisplayState.reserved_gold`. The change-detection system in `StateSync` derives the correct format based on `HudMode`:
  - `EconomyBasic`: parent `Text = "{gold}g"`, child `TextSpan = ""`
  - `EconomyAuction`: parent `Text = "{gold}g"`, child `TextSpan = " ({reserved_gold}r)"` (even when `reserved_gold == 0`)
- Server invariant violation guard: if `reserved_gold > gold`, clamp `reserved_gold` display to `gold` and log `warn!`. Never display negative free gold.
- Opponent reserved_gold in ECONOMY_BASIC mode: `GoldDisplayState.reserved_gold` is updated by `S2CGoldBroadcast` unconditionally (mode-independence contract, GDD Interactions section). Only the rendering in `StateSync` is mode-gated.
- TextSpan opacity: child `TextColor` set to `Color::srgba(r, g, b, 0.65)` at spawn (Story 001); do not re-set opacity on every format change.
- TextSpan scale: child `TextFont { font_size: base_size * 0.65, .. }` at spawn; do not re-set on format change.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 002]: `GoldDisplayState` backing component and ECONOMY_BASIC string formatting
- [Story 005]: Phase visibility toggle (HUD hidden/visible logic)
- [Story 009]: Same-tick tie-break verification

---

## QA Test Cases

*Written by qa-lead at story creation.*

**Manual check: HUD-17** — DRAFT_AUCTION entry format switch
  - Setup: HUD visible in ECONOMY_BASIC; own gold = 11g, opponent gold = 8g
  - When: Trigger `S2CPhaseChanged(DRAFT_AUCTION)`
  - Verify: Phase label shows `"AUCTION"`; own gold parent `Text == "11g"`, child `TextSpan == " (0r)"`; opponent gold parent `Text == "8g"`, child `TextSpan == " (0r)"`
  - Pass condition: Both gold labels show `"Xg (0r)"` format immediately in the same frame as phase change

**Manual check: HUD-08** — Opponent gold adaptive by phase
  - Setup (ECONOMY_BASIC): Trigger `S2CGoldBroadcast{player_id=opponent_id, gold=7, reserved_gold=0}`
  - Verify: Opponent gold label reads `"7g"` with no parenthetical
  - Pass condition: Child `TextSpan` text is `""` (empty)
  - Setup (ECONOMY_AUCTION): Enter auction mode; trigger `S2CGoldBroadcast{player_id=opponent_id, gold=7, reserved_gold=3}`
  - Verify: Parent `Text == "7g"`; child `TextSpan == " (3r)"`
  - Pass condition: No additional top-level HUD entity created for reserved gold display

**Manual check: HUD-29** — AUCTION → BASIC revert
  - Setup: HUD in ECONOMY_AUCTION; own `"11g (4r)"`, opponent `"8g (2r)"`
  - When: Trigger `S2CPhaseChanged(DRAFT_SHOP)`
  - Verify: Own gold parent `Text == "11g"`, child `TextSpan == ""`; opponent gold parent `Text == "8g"`, child `TextSpan == ""`; `HudMode == EconomyBasic`
  - Pass condition: Both TextSpan children are empty strings (not despawned entities)

**Manual check: HUD-28** — Entity tree structure (advisory)
  - Setup: ECONOMY_AUCTION active; `S2CGoldBroadcast{player_id=opponent_id, gold=7, reserved_gold=3}` processed
  - Verify: Query opponent gold parent entity — returns exactly one child entity; child has `TextSpan == " (3r)"`
  - Pass condition: No second top-level HUD entity represents opponent reserved gold

---

## Test Evidence

**Story Type**: UI
**Required evidence**: `production/qa/evidence/economy-auction-inline-gold-evidence.md` + walkthrough doc

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 002 (`GoldDisplayState` backing fields), Story 003 (phase label for `"AUCTION"` string), Story 005 (phase mode state machine)
- Unlocks: Story 009 (tie-break test uses ECONOMY_AUCTION format context)
