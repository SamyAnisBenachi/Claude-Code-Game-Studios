# Manual Browser/WASM Capture Instructions - S11-UX-HUD-TOP-STRIP-LAYOUT

> Companion to: `README.md` in this directory.
> Required because the PROMPT 940 implementation worker ran without
> interactive browser/WASM rendering capability.

## Required Environment

| Component | Required |
|-----------|----------|
| OS | Windows 11 Pro or the current Sprint 14 capture baseline. |
| Browser | Chromium-based browser with devtools device toolbar. |
| Viewports | 1920x1080 and 1366x768, DPR 1. |
| Client | `trunk serve --release` from this worker branch. |
| Server | `cargo run -p server --release` from the same source revision. |
| Branch | `work/s14-hud-top-strip-layout-940`. |

## Setup

```powershell
git fetch origin work/s14-hud-top-strip-layout-940
git checkout work/s14-hud-top-strip-layout-940

$env:CARGO_TARGET_DIR='D:/_DEV/cargo-target/ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'

trunk serve --release
```

In a second terminal:

```powershell
$env:CARGO_TARGET_DIR='D:/_DEV/cargo-target/ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'

cargo run -p server --release
```

Open two browser windows at `http://127.0.0.1:8080` and run a two-player
friend-game session.

## Reach Point

Capture during `DraftAuction` if possible, because it exercises the
reserved-gold inline span and reserve mana. If `DraftAuction` is not
reachable in the capture window, use `DraftShop` and record the missing
reserve-mana observation in the capturer log.

Target visible content:

- Phase label visible in the top strip.
- Round counter visible in the top strip.
- Own and opponent gold visible in the top strip.
- Current mana visible as the horizontal bar.
- Reserve mana visible as the diamond when `reserve_mana > 0`.
- Phase timer bar visible with a partial fill during a timed phase.

## Capture 1 - 1920x1080

1. Lock the browser viewport to 1920x1080, DPR 1.
2. Reach `DraftAuction`.
3. Wait until the timer bar is visibly mid-countdown.
4. Capture a full-size screenshot without browser chrome.
5. Save as:
   `production/qa/evidence/sprint-14-hud-top-strip-layout/top-strip-1920x1080-draft-auction.png`

## Capture 2 - 1366x768

Repeat the same steps at 1366x768, DPR 1.

Save as:
`production/qa/evidence/sprint-14-hud-top-strip-layout/top-strip-1366x768-draft-auction.png`

## Visual Checks To Record

| Check | Required observation |
|-------|----------------------|
| Text fit | No phase, round, gold, mana, reserve, or timer text is clipped. |
| Stable dimensions | Fixed-pixel children keep the same rendered pixel size at both viewports. |
| No overlap | Top-strip siblings do not overlap each other, the timer bar, HUD figurine, scoreboard dots, dim overlay, or bottom strip. |
| Z-layer | HUD top strip remains visible in the UI base layer and is not hidden by base-layer spawn order. |
| No non-claim drift | Do not claim Standard-tier accessibility, final-art replacement, `S8-QA-001-W1`, or PROMPT 761 retry closure. |

## Capturer Log

Fill this table when the screenshots land:

| Capture | Captured at | Captured by | Notes |
|---------|-------------|-------------|-------|
| `top-strip-1920x1080-draft-auction.png` | YYYY-MM-DD | TBD | TBD |
| `top-strip-1366x768-draft-auction.png` | YYYY-MM-DD | TBD | TBD |
