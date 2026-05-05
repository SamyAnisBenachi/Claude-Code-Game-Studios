# HUD-011 Mana Shapes Evidence

Status: PASS for HUD-011 / A11Y-ST-13 implementation evidence.

Story: `production/epics/hud/story-011-current-reserve-mana-shapes.md`

QA condition: `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md`

## Scope

Included:

- A11Y-ST-13 current/reserve mana shape distinction only.
- Current mana as a horizontal bar container.
- Reserve mana as a diamond container.
- Automated ECS proof that shape distinction is component/layout state, not color.
- Browser/WASM captures at `1366x768` and `1920x1080`.
- Grayscale captures for color-stripped review.

Excluded:

- Broader Settings or Accessibility work.
- Colorblind modes, UI scaling, reduced motion, input remapping, tutorial
  persistence, and audio controls.
- Closure of QA-COND-0005 as a whole.

## Fixture

Harness:

```text
client/hud-mana-shapes-harness.html
client/src/hud_mana_shapes_harness.rs
```

Capture URL:

```text
http://127.0.0.1:8081/hud-mana-shapes-harness.html?fixture=hud_011_mana_shapes
```

Fixture values:

```text
current_mana=6
mana_cap=10
reserve_mana=2
```

## Browser Environment

Capture date: 2026-05-05

Tooling:

- Trunk: `0.21.14`
- Chrome: `147.0.7727.139`
- Capture tool: PowerShell Chrome DevTools Protocol

Build and serve commands:

```text
cd client
trunk build hud-mana-shapes-harness.html --release
trunk serve hud-mana-shapes-harness.html --release --port 8081 --address 127.0.0.1 --no-autoreload true --no-error-reporting true
```

Capture command from repository root:

```text
powershell.exe -ExecutionPolicy Bypass -File production\qa\evidence\captures\hud-011-mana-shapes\capture.ps1
```

## Artifacts

Summary:

```text
production/qa/evidence/captures/hud-011-mana-shapes/capture-summary.json
```

Color captures:

```text
production/qa/evidence/captures/hud-011-mana-shapes/hud-011-mana-shapes-1366x768.png
production/qa/evidence/captures/hud-011-mana-shapes/hud-011-mana-shapes-1920x1080.png
```

Grayscale captures:

```text
production/qa/evidence/captures/hud-011-mana-shapes/hud-011-mana-shapes-1366x768-grayscale.png
production/qa/evidence/captures/hud-011-mana-shapes/hud-011-mana-shapes-1920x1080-grayscale.png
```

Capture summary verdict:

```text
artifactProduction=PASS
colorCaptureCount=2
grayscaleCaptureCount=2
fixtureValues=current_mana=6, mana_cap=10, reserve_mana=2
```

## Grayscale Review

Result: PASS.

At both `1366x768` and `1920x1080`, hue and saturation removal preserves the
non-color distinction:

- Current mana remains identifiable as the horizontal bar at the bottom-left.
- Reserve mana remains identifiable as the diamond above the current mana bar.
- The mana cluster is visible, not clipped, and not overlapping Hand UI,
  Shop/Auction UI, board content, or other HUD zones in the scoped HUD fixture.

## Automated Verification

Focused test:

```text
cargo test -p client --test hud_mana_shape_distinction_test
```

Result: PASS, 3/3.

Coverage:

- Current mana exposes `CurrentManaShape` plus `ManaShapeGeometry { kind: Bar }`.
- Reserve mana exposes `ReserveManaShape` plus
  `ManaShapeGeometry { kind: Diamond }`.
- Assertions inspect layout/geometry values and ignore color.
- Reserve zero hides the diamond container and reserve label together.
- Repeated economy updates reuse pre-pooled mana shape/text entities.

Regression test:

```text
cargo test -p client --test hud_gold_mana_display_test
```

Result: PASS, 6/6.

Regression coverage retained:

- `6 / 10` current/cap mana text.
- `+2 reserve` reserve text when reserve is positive.
- Reserve visibility hidden at zero.
- Existing gold/mana display behavior remains green.

## QA-COND-0005 Impact

HUD-011 implements and verifies only A11Y-ST-13: current and reserve mana pools
are visually distinguishable without relying on color alone.

QA-COND-0005 remains Open until the remaining Standard-tier rows are
implemented/evidenced, reclassified, dependency-blocked, or accepted as risk.
