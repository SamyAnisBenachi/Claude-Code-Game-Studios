# Story 011: Current and Reserve Mana Shape Distinction

> **Epic**: HUD
> **Status**: Complete
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 6 S6-04 / QA-COND-0005

## Context

**Sprint Gate**: Sprint 6 S6-04 / QA-COND-0005 Standard-tier accessibility remediation.

**GDD**: `design/gdd/hud.md`

**GDD Requirements**:

- `design/gdd/hud.md` Rule 2: the HUD bottom mana zone displays the local player's mana and reserve mana, with vertical space reserved so the reserve readout appearing or disappearing does not shift layout.
- `design/gdd/hud.md` Rule 3: own mana displays exactly `<current_mana> / <mana_cap>` and reserve mana displays exactly `+<reserve_mana> reserve`, hidden when `reserve_mana == 0`.
- `design/gdd/hud.md` HUD-03: `S2CGoldUpdate { gold=8, current_mana=6, mana_cap=10, reserve_mana=2 }` must render mana text `6 / 10` and reserve text `+2 reserve`.
- `design/gdd/hud.md` HUD-21: changing `mana_cap` updates the denominator and hides reserve when `reserve_mana=0`.

**Accessibility Requirement**: `A11Y-ST-13` from `design/accessibility-requirements.md` and `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`. The current mana and reserve mana containers must be visually distinguishable without relying on color alone. The recommended implementation target is current mana as a bar-shaped container and reserve mana as a diamond-shaped container.

**TR IDs**: `TR-HUD-002` for mana display state and visibility. `A11Y-ST-13` is a Sprint 6 accessibility row, not a registered `TR-HUD-*` requirement.

**ADR Governing Implementation**:

- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)

**ADR Decision Summary**: HUD is a read-only `bevy_ui` overlay inside `HudPlugin`, registered under `PresentationPlugin`. It must not send C2S game-logic messages. HUD presentation state is derived from authoritative S2C economy state and the shared presentation economy view, and HUD entities must be pre-pooled at session entry rather than spawned during per-update rendering.

**Engine**: Bevy 0.18 + browser/WASM target | **Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` for any Bevy UI implementation. Bevy 0.18 uses Required Components API; do not use `NodeBundle`. HUD shape containers should remain `bevy_ui` nodes, not world-space sprites. If the reserve diamond uses rotation or an equivalent geometric approach, verify the exact Bevy 0.18 UI transform API with `cargo check` before committing. Preserve the `#[cfg(feature = "ui_picking")]` guard on HUD picking behavior.

**Control Manifest Rules (2026-05-05)**:

- Required: Presentation UI overlays use `bevy_ui`; board content remains world-space.
- Required: HUD and other presentation entities are pre-pooled at session entry; no per-update spawn/despawn for steady-state UI.
- Required: `S2CGoldUpdate` is drained only by the shared economy-view system; HUD reads shared economy state and must not add an independent S2C economy drain.
- Forbidden: Client presentation must not assert or mutate authoritative game state.
- Guardrail: Presentation steady-state work stays below 1 ms/frame; phase-boundary presentation spikes stay below 3 ms.

---

## Scope

### In Scope

- Add a non-color shape distinction between current mana and reserve mana in the HUD.
- Implement the smallest UI change that makes current mana read as a bar-shaped container and reserve mana read as a diamond-shaped container.
- Preserve existing mana text behavior: current mana remains `<current_mana> / <mana_cap>` and reserve mana remains `+<reserve_mana> reserve`, hidden at zero reserve.
- Preserve existing authoritative economy data flow and HUD update timing.
- Add automated evidence proving the shape distinction exists in component/layout state, not only in color values.
- Capture browser/WASM visual evidence that a colorblind or grayscale review can distinguish the two mana pools without hue.
- Update the HUD-011 evidence document at the exact path listed in `## Test Evidence`.

### Out of Scope

- No code implementation in this story-readiness pass.
- No Settings or Accessibility screen work.
- No colorblind palette selector or user preference persistence.
- No changes to mana economy rules, reserve mana spend rules, or server authority.
- No redesign of gold labels, phase label, round counter, scoreboard dots, timer UI, Hand UI reserve split controls, or Shop/Auction UI.
- No changes to `production/sprint-status.yaml`, `production/session-state/**`, `design/assets/**`, or `AGENTS.md`.
- Do not close QA-COND-0005 from this story alone unless all remaining Standard-tier rows have separate valid dispositions.

---

## Acceptance Criteria

- [x] **A11Y-ST-13 shape distinction**: GIVEN HUD is visible with `current_mana=6`, `mana_cap=10`, and `reserve_mana=2`, WHEN current mana and reserve mana render, THEN current mana is contained in a horizontally oriented bar or segmented bar shape, and reserve mana is contained in a diamond shape. The two resources remain distinguishable when hue and saturation are removed from the capture.
- [x] **Current mana remains a current/cap readout**: GIVEN `S2CGoldUpdate { current_mana=6, mana_cap=10, reserve_mana=2 }`, WHEN HUD state sync completes, THEN the current mana text still reads exactly `6 / 10`; the shape work does not collapse current and reserve mana into one total.
- [x] **Reserve mana text and visibility are preserved**: GIVEN `reserve_mana=2`, WHEN HUD state sync completes, THEN reserve text still reads exactly `+2 reserve` and is visible. GIVEN a later update with `reserve_mana=0`, THEN the reserve readout is hidden and no stale diamond, numeral, or label remains visible.
- [x] **Shape is not color-only**: GIVEN the HUD mana cluster is inspected in the automated UI test, WHEN color fields are ignored, THEN the current mana container has bar geometry and the reserve mana container has diamond geometry as explicit component/layout state. A test that only compares color values does not satisfy this criterion.
- [x] **No new steady-state spawning**: GIVEN the HUD has spawned at session entry, WHEN three `S2CGoldUpdate` messages change current/reserve mana over later frames, THEN no additional HUD shape or text entities are spawned during those updates. Any shape nodes required by the implementation are pre-pooled before the first visible HUD frame.
- [x] **Existing economy display regressions stay green**: `cargo test -p client --test hud_gold_mana_display_test` passes after the shape change.
- [x] **Focused shape test passes**: `cargo test -p client --test hud_mana_shape_distinction_test` passes. The target must be backed by `tests/unit/hud/mana_shape_distinction_test.rs` and registered in `client/Cargo.toml`.
- [x] **Browser/WASM evidence exists**: `production/qa/evidence/hud-011-mana-shapes-evidence.md` records the browser, viewport, fixture values, capture method, capture artifact directory, grayscale/color-stripped review result, and pass/fail verdict for A11Y-ST-13.
- [x] **Browser/WASM capture covers two viewports**: The evidence document includes captures at `1366x768` and `1920x1080` with current mana and reserve mana both visible, not clipped, and not overlapping Hand UI, Shop/Auction UI, board content, or other HUD zones.
- [x] **QA-COND-0005 impact is explicit**: The evidence document states that HUD-011 implements and verifies only A11Y-ST-13. QA-COND-0005 remains Open until the remaining Standard-tier rows are implemented/evidenced, reclassified, dependency-blocked, or accepted as risk.
- [x] **Whitespace gate passes**: `git diff --check` passes.

---

## Implementation Notes

- Keep the HUD in `client/src/ui/hud/`. Do not move mana display ownership into Hand UI or Shop/Auction UI.
- Prefer using the existing mana and reserve label nodes as the visual containers if that preserves readable text and stable layout. If wrapper/child nodes are required, spawn them during HUD session entry and include them in `HudEntities` or an equivalent pre-pooled entity registry.
- The current mana shape should be meaningfully bar-like: horizontal orientation, width greater than height, or discrete horizontal segments. A square, circle, or diamond for current mana does not satisfy A11Y-ST-13.
- The reserve mana shape should be meaningfully diamond-like: a rotated-square/diamond outline or fill. A rounded rectangle, pill, square, or text-only label does not satisfy A11Y-ST-13.
- Shape distinction must not depend on teal vs blue, opacity, luminance, gradient, glow, or text color. Those can remain supplementary but cannot be the only signal.
- Preserve text values and visibility from Story 002. If the design eventually wants only a numeral inside the diamond, that is a separate UX/GDD change; this story keeps the current GDD text contract.
- Do not add art assets or edit `design/assets/**`; use UI geometry/components for this remediation.
- If adding a browser harness or fixture mode, keep it dev/test-only and document the exact URL/seed in the evidence file.

---

## QA Test Cases

- **A11Y-ST-13 component geometry**
  - Given: HUD is initialized in a client test world with mana state `current_mana=6`, `mana_cap=10`, and `reserve_mana=2`
  - When: HUD state sync completes
  - Then: current mana exposes bar geometry and reserve mana exposes diamond geometry through component/layout state; the assertion does not read color values

- **Reserve zero hides shape**
  - Given: reserve mana was visible with `+2 reserve`
  - When: a later economy update sets `reserve_mana=0`
  - Then: reserve text and the reserve diamond container are hidden together; no stale reserve shape remains visible

- **Existing text behavior**
  - Given: the Story 002 HUD mana fixture
  - When: `cargo test -p client --test hud_gold_mana_display_test` runs
  - Then: all existing mana text, reserve visibility, cold-start placeholder, and `mana_cap=0` guard tests remain green

- **No steady-state spawn**
  - Given: HUD entity IDs are captured after session-entry spawn
  - When: multiple economy updates change mana and reserve values
  - Then: all mana shape/text entity IDs are unchanged and no additional HUD shape entities are spawned during update frames

- **Browser/WASM grayscale review**
  - Given: the browser/WASM HUD fixture runs with both current and reserve mana visible
  - When: QA reviews the captured HUD cluster at `1366x768` and `1920x1080` with hue/saturation removed
  - Then: current mana is still identifiable as the bar resource and reserve mana is still identifiable as the diamond resource

---

## Test Evidence

**Story Type**: UI

**Required automated test target**:

- `tests/unit/hud/mana_shape_distinction_test.rs`
- `cargo test -p client --test hud_mana_shape_distinction_test`

**Required regression target**:

- `cargo test -p client --test hud_gold_mana_display_test`

**Required browser/WASM evidence path**:

- `production/qa/evidence/hud-011-mana-shapes-evidence.md`

**Required capture artifact directory**:

- `production/qa/evidence/captures/hud-011-mana-shapes/`

**Required evidence contents**:

- Browser and viewport details for `1366x768` and `1920x1080`.
- Fixture values: `current_mana=6`, `mana_cap=10`, `reserve_mana=2`.
- Screenshot or capture artifact references under the required capture directory.
- Grayscale/color-stripped review result proving the distinction is not color-only.
- Test command output summary for the automated shape test and HUD gold/mana regression.
- QA-COND-0005 impact statement limited to A11Y-ST-13.

**Status**: [x] Implemented, captured, and verified.

---

## Dependencies

- Depends on: `production/epics/hud/story-001-hud-plugin-scaffold.md` (Story 001, Complete) for the HUD entity pool and root UI hierarchy.
- Depends on: `production/epics/hud/story-002-gold-mana-display.md` (Story 002, Complete) for gold/mana display behavior and reserve visibility.
- Depends on: `production/epics/hud/story-010-numeric-tween-animation.md` (Story 010, Complete) to preserve current/reserve mana animation behavior.
- Depends on: `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md` A11Y-ST-13 row identifying this as a Sprint 6 must-implement blocker.
- Depends on: ADR-021 and ADR-002 Accepted.
- Unlocks: A11Y-ST-13 can move from "must implement in Sprint 6" to "implemented + evidence attached" after this story is implemented and `production/qa/evidence/hud-011-mana-shapes-evidence.md` passes QA review.

## Performance Budget

No measurable runtime performance impact expected. This is a small pre-pooled UI geometry change in the HUD mana cluster. The implementation must not add per-frame entity creation, texture uploads, additional network message drains, or polling work outside existing HUD state sync.

## QA-COND-0005 Impact

This story targets only the A11Y-ST-13 mana-pool shape distinction sub-gap. Completing it reduces QA-COND-0005 by one must-implement row, but QA-COND-0005 remains Open until every other Standard-tier blocker has implementation/evidence, reclassification, dependency-blocking, or accepted-risk disposition.

## No Open Questions

No unresolved design question blocks this story. The implementation target is fixed for Sprint 6: current mana uses a bar-shaped container, reserve mana uses a diamond-shaped container, and existing HUD economy text behavior is preserved.

## Completion Notes

**Completed**: 2026-05-06
**Criteria**: 11/11 passing.
**Deviations**: None. Story manifest version `2026-05-05` matches the current control manifest.
**Test Evidence**: `production/qa/evidence/hud-011-mana-shapes-evidence.md` exists and records PASS evidence for A11Y-ST-13, including `1366x768` and `1920x1080` browser/WASM captures plus grayscale captures under `production/qa/evidence/captures/hud-011-mana-shapes/`.
**Verification**: `cargo test -p client --test hud_mana_shape_distinction_test` passed 3/3; `cargo test -p client --test hud_gold_mana_display_test` passed 6/6; `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
**QA-COND-0005 Impact**: A11Y-ST-13 is implemented and evidenced by HUD-011. QA-COND-0005 remains Open until the remaining Standard-tier rows are implemented/evidenced, reclassified, dependency-blocked, or accepted as risk.
**Code Review**: Skipped - Lean mode. `production/review-mode.txt` is absent, so QL-TEST-COVERAGE and LP-CODE-REVIEW gates were skipped by story-done policy.
