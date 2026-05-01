# Story 002: Fan Layout Formula — Card Position & Rotation

> **Epic**: Hand UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-001`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Hand fan uses `bevy_ui` `Node`-based absolute positioning. Fan card positions are computed by Formulas 1 and 2 from the GDD and applied to each pre-pooled fan slot entity's `Transform`. No flexbox layout — every card position is computed directly.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `bevy_ui` screen-space has `+Y downward`. Formula 1 uses subtraction (`card_y = fan_base_y − arc_height × t²`) to produce an upward arc — do NOT flip the sign. `Transform.rotation` applied as `Quat::from_rotation_z(angle_radians)`. Verify sign convention: positive angle = counterclockwise in bevy's coordinate system; adjust so right-hand cards lean right (+t → clockwise → negative radians in Bevy). `bevy_ui` AABB hit detection is acceptable at ≤15° tilt per GDD Tuning Knobs.

**Control Manifest Rules (Presentation Layer)**:
- Required: UI always `bevy_ui` (`Node`); no world-space sprites for fan cards.
- Required: `BoardLayout` and `CardAtlas` are session-scoped; all formula systems must be `in_state(ClientState::InSession)`.
- Forbidden: `NodeBundle` — use `Node { .. }` Required Components API.
- Forbidden: `Color::rgba()` — use `Color::srgba()`.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Formulas section, scoped to this story:*

- [ ] **HU-02**: GIVEN the hand has 5 cards, WHEN the fan layout renders, THEN:
  - Card at index 2 (center): `t = 0`, `card_x = fan_center_x`, `card_y = fan_base_y`, `card_rotation_deg = 0°`
  - Card at index 4 (rightmost): `t = +1.0`, `card_x = fan_center_x + fan_half_spread`, `card_y = fan_base_y − arc_height`, `card_rotation_deg = max_rotation_deg`

- [ ] **HU-02b**: GIVEN the hand has exactly 2 cards, WHEN the fan layout renders, THEN card at index 0 has `t = −1.0` AND card at index 1 has `t = +1.0`. (This surfaces the count=2 divide-by-zero fix: `half_span = (2−1)/2.0 = 0.5`; `t_0 = (0−0.5)/0.5 = −1.0`. The previous incorrect `max(half_span, 1.0)` clamp would compress these to ±0.5.)

- [ ] **HU-03**: GIVEN the hand has exactly 1 card, WHEN the fan renders, THEN: `t = 0` (via Formula 1 early-return for `count == 1`), `card_y = fan_base_y`, `card_rotation_deg = 0°`. Single card centered at `fan_center_x` with no arc lift and no tilt.

- [ ] **HU-03b**: GIVEN the hand has 0 cards, WHEN PLACEMENT begins, THEN Formula 1 is NOT evaluated (no division or arithmetic executed for count=0), no fan card slot entities are `Visibility::Visible`, AND the Submit button entity is still `Visibility::Visible` with label `"Submit (0 cards)"` and interaction state `Active`.

---

## Implementation Notes

*Derived from ADR-021 and GDD Formulas 1 & 2:*

**Formula 1 — Fan Card Screen Position** (from GDD):
```
if count == 0:
    // Skip entirely — no cards to render
if count == 1:
    t = 0.0      // single card: no arc, no tilt, centered
else:
    half_span = (count - 1) as f32 / 2.0
    t = (index as f32 - half_span) / half_span

card_x = fan_center_x + t * fan_half_spread
card_y = fan_base_y - arc_height * t * t    // SUBTRACTION: edges lift UP in screen-space (+Y down)
```

**Formula 2 — Fan Card Rotation** (from GDD):
```
card_rotation_deg = max_rotation_deg * t
```
Apply as `Quat::from_rotation_z(-card_rotation_deg.to_radians())` — negative because positive `max_rotation_deg` should make right-hand cards lean clockwise (in screen-space, clockwise = negative Z rotation in Bevy's right-handed coordinate system). Verify the sign against a visual test before shipping.

**Slot visibility**: Only fan slots where `index < hand_count` should be `Visibility::Visible`. Slots from `hand_count..10` remain `Visibility::Hidden`. This story computes positions; Story 003 drives which slots are visible per phase.

**Tuning constants** (from `Res<GameConfig>`):
- `fan_center_x`: screen_width / 2.0
- `fan_base_y`: screen_height − margin (from config)
- `fan_half_spread`: default 280 px (range 180–400 px)
- `arc_height`: default 10 px (range 0–20 px)
- `max_rotation_deg`: default 10° (range 5–15°)

All tuning values are read from `Res<GameConfig>` — never hardcoded in the system.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 001]: Entity spawning (pre-pooling). This story READS those entities and updates their positions.
- [Story 003]: Which slots are `Visibility::Visible` per phase.

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these — do not invent new test cases during implementation.*

- **HU-02**: count=5 position formula
  - Given: `count=5, fan_center_x=400.0, fan_half_spread=280.0, fan_base_y=500.0, arc_height=10.0, max_rotation_deg=10.0`
  - When: Formula computed for index=2 and index=4
  - Then (index=2): `t=0.0`, `card_x=400.0`, `card_y=500.0`, `card_rotation_deg=0.0°`
  - Then (index=4): `half_span=2.0`, `t=(4-2)/2=1.0`, `card_x=680.0`, `card_y=500.0−10.0×1.0=490.0`, `card_rotation_deg=10.0°`
  - Edge cases: index=0 (leftmost): `t=-1.0`, `card_x=120.0`, `card_y=490.0`, `card_rotation_deg=-10.0°`

- **HU-02b**: count=2 clamp fix
  - Given: `count=2`
  - When: Formula computed for index=0 and index=1
  - Then: `half_span=0.5`; index=0: `t=(0-0.5)/0.5=-1.0`; index=1: `t=(1-0.5)/0.5=+1.0`
  - Edge cases: Verify the old incorrect formula (`max(half_span, 1.0)` clamp) would produce `t=±0.5` — confirm the fix produces the correct `t=±1.0`

- **HU-03**: count=1 early-return
  - Given: `count=1`
  - When: Formula computed for index=0
  - Then: `t=0.0` (early-return branch); `card_y = fan_base_y` (no subtraction); `card_rotation_deg=0.0°`
  - Edge cases: Assert that the `if count == 1` branch is taken, not the general formula (avoids division by zero on `half_span=0`)

- **HU-03b**: count=0 skip + Submit button present
  - Given: `count=0`, `CurrentClientPhase = PLACEMENT`
  - When: Layout system runs
  - Then: No fan slot entity has `Visibility::Visible`; Submit button has `Visibility::Visible` with text `"Submit (0 cards)"` and `Active` interaction state
  - Edge cases: Confirm formula evaluation is not triggered (no arithmetic on count=0 path)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/hand-ui/fan_layout_formula_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (pre-pooled fan slot entities must exist before positions can be applied)
- Unlocks: Story 005 (PLACEMENT staging core reads fan positions)
