# Story 002: Fan Layout Formula — Card Position & Rotation

> **Epic**: Hand UI
> **Status**: Complete
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

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: Story 001 (pre-pooled fan slot entities must exist before positions can be applied)
- Unlocks: Story 005 (PLACEMENT staging core reads fan positions)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 4/4 passing
**Deviations**:
- Advisory: Story metadata references `TR-HU-001`, but the current registry maps `TR-HU-001` to HU-01 pre-pooling while this story verifies HU-02, HU-02b, HU-03, and HU-03b from `design/gdd/hand-ui.md`.
- Advisory: The integrated client reads fan tuning from `Res<HandFanLayoutConfig>` because shared/client `GameConfig` does not currently expose fan layout fields. The layout system still reads a resource and tests override it, but it is not wired to `Res<GameConfig>`.
**Test Evidence**: Logic test `tests/unit/hand-ui/fan_layout_formula_test.rs`; `cargo test -p client --test hand_ui_fan_layout_formula_test` passed 5/5; `cargo test -p client --test hand_ui_plugin_scaffold_test` passed 3/3; `cargo check -p client` passed.
**Code Review**: Skipped - lean mode.
**Scope**: Implementation already integrated on `main` via `047aff9`; worker commit `da0fe3a` is present on `work/hand-ui-002-fan-layout-formula`. `production/sprint-status.yaml` unchanged because no HAND-UI-002 row exists.

---

## Verdict 3 Reconciliation — Viewport Sync Repair (PROMPT 642, 2026-05-10)

**Trigger**: Finding B v2 Verdict 3 traced the still-mispositioned hand fan to `HandFanViewport` having no writer system. The resource was registered with `init_resource` only and stayed at the `Default` value (800×600) for the lifetime of the app. Every consumer of `HandFanLayoutConfig::metrics_for_viewport(*viewport)` — including `apply_fan_layout_system` and `apply_reserve_strip_layout_system` — read this stale default, so the fan was anchored to a 600px-tall screen even when the actual primary window was 1080p+. AC-HU-02 (`fan_center_x = screen_width / 2`) and AC-HU-03b (`fan_base_y = screen_height − margin`) implicitly require a runtime-screen-anchored viewport; the missing writer silently violated both ACs whenever the real window size differed from 800×600.

**Status flip**: Done → Done (retained). This is a reconciliation of a previously-implicit AC requirement, not a regression — the original 4/4 unit-test ACs remain satisfied. The new viewport-sync invariant is now explicit and test-enforced at runtime.

**Repair**: Added `sync_hand_fan_viewport_from_window_system` in `client/src/ui/hand/mod.rs` (reads `Single<&Window, With<PrimaryWindow>>` and `set_if_neq`s the `HandFanViewport` resource each frame). Registered in `HandUiPlugin` with `.before(HandUiSystemSet::StateSync).run_if(in_state(ClientState::InSession))` so the writer always runs before `apply_fan_layout_system` and `apply_reserve_strip_layout_system` consume the resource.

**New runtime test**: `tests/integration/hand-ui/hand_ui_viewport_sync_test.rs` (registered as `[[test]] hand_ui_viewport_sync_test` in `client/Cargo.toml`):
- `viewport_sync_anchors_fan_layout_to_primary_window_at_1920_1080` — spawns `(Window 1920×1080, PrimaryWindow)`, drives 3 acquisitions, transitions to PLACEMENT, asserts `HandFanViewport == (1920.0, 1080.0)` and that fan slots 0/1/2 land at the formula positions for a 1920-wide × 1080-tall viewport (centered around `x=960`, base near `y=980`).
- `default_viewport_persists_when_no_primary_window_is_present` — sanity guard so existing tests that inject `HandFanViewport` directly (and never spawn a Window) keep working: when no `(Window, PrimaryWindow)` entity exists, the system is a no-op and the resource keeps its 800×600 default.

**Regression suite**: `cargo test -p client --test placement_entry_post_acquisition_test` → 1/1 PASS; `cargo test -p client --test hand_ui_draft_initial_grid_test` → 6/6 PASS; `cargo test -p client --test hand_ui_fan_layout_formula_test` → 5/5 PASS (default 800×600 path unchanged).

---

## Verdict A Reconciliation — Fan-Slot Coord-Space Alignment (PROMPT 671, 2026-05-11)

**Trigger**: After the PROMPT 642 viewport-sync repair the user's PLACEMENT-phase retest still showed an empty fan strip at 1920×1080. PROMPT 669 source diagnostic established Verdict A: `HandFanLayoutConfig::metrics_for_viewport` was returning `fan_base_y = viewport.height_px − fan_base_margin_px` (a viewport-space coord) while every fan slot is `ChildOf(fan_root)`, and `fan_root` is `position_type: Absolute, left:0, right:0, bottom:0, height:260`. With the strip's top-left at `(0, viewport.height − 260)`, a child whose `Node.top` is interpreted as ~`viewport.height − 100` resolves to a screen-Y of roughly `2 × viewport.height − 360` — off-screen at every viewport taller than ~860 px. The PROMPT 642 viewport sync was correct but landed atop a stale coord-space contract, so the visible symptom persisted.

**Repair**: `metrics_for_viewport` now returns `fan_base_y = HAND_FAN_STRIP_HEIGHT_PX − fan_base_margin_px` (LOCAL-to-fan_root). `fan_center_x = viewport.width_px / 2.0` is unchanged — the strip is `left:0 right:0`, so its local X axis matches the viewport X axis. A new module-level constant `HAND_FAN_STRIP_HEIGHT_PX = 260.0` replaces the hardcoded literal at the `fan_root` spawn site so the strip height and the metrics derivation are kept in sync from a single source of truth.

**AC HU-02 re-affirmation**: HU-02 / HU-02b / HU-03 / HU-03b remain satisfied by the formula math; only the coord-space *contract* of `fan_base_y` changes (LOCAL, not viewport). The pure-formula unit tests (`hu_02_count_five_positions_center_and_edges`, `hu_02b_count_two_uses_full_normalized_span`, `hu_03_single_card_early_return_centers_without_arc_or_tilt`) feed metrics directly to `compute_fan_slot_layout` and still exercise the unchanged `card_y = fan_base_y − arc_height × t²` invariant. The two system-level tests that compare against `transform.translation.y` (`layout_system_applies_formula_to_visible_pooled_slots` and `draft_initial_grid_test::qa_metrics`) were updated to expect the new LOCAL base (`260 − 100 = 160` at default 800×600).

**New regression test (closes audit gap from PROMPT 646)**: `tests/integration/hand-ui/hand_ui_slot_onscreen_test.rs` runs at 800×600, 1280×720, and 1920×1080. For each occupied fan slot, it derives the absolute on-screen position from `Node.left`/`Node.top` plus the strip's known anchor (`bottom:0, height:HAND_FAN_STRIP_HEIGHT_PX`) and asserts the slot is inside `[0, viewport.width] × [0, viewport.height]`. It also asserts `Node.top ∈ [0, HAND_FAN_STRIP_HEIGHT_PX]` — a viewport-coord regression of the kind PROMPT 669 found would push `Node.top` to ~980 at 1080p and fail this bound. Registered as `[[test]] hand_ui_slot_onscreen_test` in `client/Cargo.toml`. Headless `bevy::ui::UiPlugin` is intentionally avoided (no other hand-ui test brings in AssetPlugin / WindowPlugin / TransformPlugin / ImagePlugin / Camera2d); computing the on-screen position from Node + known parent layout tests the same invariant with no fragile dependence on a partial render pipeline.

**Stale-expectation correction in viewport_sync test**: `hand_ui_viewport_sync_test.rs::viewport_sync_anchors_fan_layout_to_primary_window_at_1920_1080` previously asserted `fan_base_y ≈ 980` (1080 − 100 viewport-coord). That expectation rode on the same Verdict-A bug the rest of the source carried — the slot positions it asserted were the off-screen positions. Expectations are now `fan_base_y = HAND_FAN_STRIP_HEIGHT_PX − fan_base_margin_px = 160`. The viewport-sync invariant itself (`HandFanViewport == (1920.0, 1080.0)`) is unchanged and still gated.

**Worker scope**: PROMPT 671 was Worker A (PRIMARY repair — coord-space alignment). Worker B (chrome children sizing audit at `client/src/ui/hand/mod.rs` ~L2566-2618) is a separate prompt and out of scope here.
