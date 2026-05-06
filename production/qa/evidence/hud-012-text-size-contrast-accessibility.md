# HUD-012 Text Size and Contrast Accessibility Evidence

| Field | Value |
|---|---|
| Story | HUD-012 Text Size and Contrast Accessibility Evidence |
| QA condition | QA-COND-0005 |
| QA-COND-0005 status | Open |
| Evidence date | 2026-05-06 |
| Browser | Chrome 147.0.7727.139 |
| Build target | Browser/WASM Bevy client harness via Trunk |
| Commit | Worker branch HEAD; exact hash reported in worker final status |
| Capture command | `trunk serve hud-text-size-contrast-harness.html --release --port 8082 --address 127.0.0.1`; `powershell.exe -ExecutionPolicy Bypass -File production/qa/evidence/captures/hud-012-text-size-contrast/capture.ps1` |
| Automated measurement command | `cargo test -p client --test hud_text_size_contrast_accessibility_test` |
| Artifact directory | `production/qa/evidence/captures/hud-012-text-size-contrast/` |

## Fixture

HUD-012 uses phase `DRAFT_AUCTION`, round `9`, own `gold=11`, own
`reserved_gold=4`, opponent `gold=8`, opponent `reserved_gold=3`,
`current_mana=6`, `mana_cap=10`, and `reserve_mana=2`.

The automated harness validates the same fixed HUD typography at both required
viewports: `1366x768` and `1920x1080`. The browser/WASM harness exports the
same fixture values and measured accessibility tokens through
`globalThis.__hud012TextSizeContrastEvidence` before each screenshot is taken.

## Screenshot Artifacts

Browser/WASM screenshots were captured after the WASM harness reported ready.
Both captures show the HUD-owned phase, round, gold, reserved-gold suffix,
current mana, and reserve mana labels readable at their screen-edge anchors with
no overlap between HUD zones.

| Viewport | Screenshot |
|---|---|
| 1366x768 | `production/qa/evidence/captures/hud-012-text-size-contrast/hud-012-text-size-contrast-1366x768.png` |
| 1920x1080 | `production/qa/evidence/captures/hud-012-text-size-contrast/hud-012-text-size-contrast-1920x1080.png` |

Additional artifacts:

- `production/qa/evidence/captures/hud-012-text-size-contrast/hud-012-text-size-contrast-capture-summary.json`
- `production/qa/evidence/captures/hud-012-text-size-contrast/capture.ps1`

## Text Size Measurements

Measured from Bevy `TextFont.font_size` values in the HUD entity tree and
exported by the browser/WASM harness summary.

| HUD label | 1366x768 | 1920x1080 | Required floor | Verdict |
|---|---:|---:|---:|---|
| Own gold primary `11g` | 40 px | 40 px | 40 px | Pass |
| Opponent gold primary `8g` | 40 px | 40 px | 40 px | Pass |
| Own reserved suffix ` (4r)` | 26 px | 26 px | 20 px | Pass |
| Opponent reserved suffix ` (3r)` | 26 px | 26 px | 20 px | Pass |
| Current mana `6 / 10` | 20 px | 20 px | 20 px | Pass |
| Reserve mana `+2 reserve` | 20 px | 20 px | 20 px | Pass |
| Phase label `AUCTION` | 20 px | 20 px | 20 px | Pass |
| Round counter `R9` | 20 px | 20 px | 20 px | Pass |
| Own gold placeholder `--g` | 40 px | 40 px | 40 px | Pass |
| Opponent gold placeholder `--g` | 40 px | 40 px | 40 px | Pass |
| Mana placeholder `-- / --` | 20 px | 20 px | 20 px | Pass |

## Contrast Measurements

Foreground values are composited against each immediate HUD text background
token after the HUD-011 mana-shape rebase. The primary HUD text background token
is `Color::srgba(0.04, 0.07, 0.12, 1.0)`.

| HUD pair | Contrast ratio | Required ratio | Verdict |
|---|---:|---:|---|
| Phase label, round counter, and placeholders on HUD text background | 17.87:1 | 4.5:1 | Pass |
| Own and opponent gold primary text on HUD text background | 12.95:1 | 4.5:1 | Pass |
| Reserved suffix text after 65% alpha compositing | 6.74:1 | 4.5:1 | Pass |
| Current mana text on current mana bar fill | 13.56:1 | 4.5:1 | Pass |
| Reserve mana text on reserve mana diamond fill | 14.80:1 | 4.5:1 | Pass |

The focused test covers DRAFT_AUCTION and RESOLUTION HUD states. No phase-specific
dimming is applied to the HUD text/background pairs, and RESOLUTION retains the
same passing contrast tokens.

## Verification Summary

Passed in the worker worktree after rebasing on latest `origin/main` and adding
Browser/WASM capture evidence:

- `cargo test -p client --test hud_text_size_contrast_accessibility_test`
- `cargo test -p client --test hud_gold_mana_display_test --test hud_phase_label_round_counter_test --test hud_economy_auction_inline_gold_test`
- `cargo fmt -p client -- --check`
- `cargo check -p client`

`git diff --check` also passed after rebase.

## A11Y-ST-01 Impact

HUD-012 verifies HUD-owned A11Y-ST-01 text-size floors for gold, mana, reserve,
phase, round, reserved-gold suffixes, and cold-start placeholders. Card text and
the actual auction price counter remain outside this story.

## A11Y-ST-03 Impact

HUD-012 verifies HUD-owned A11Y-ST-03 contrast tokens for HUD text/background
pairs. Card, Settings, Shop/Auction, Hand UI, board, and result-screen contrast
remain outside this story.

## QA-COND-0005 Impact

QA-COND-0005 remains Open. This evidence reduces only the HUD-owned A11Y-ST-01
and A11Y-ST-03 gaps. The condition must remain open until every other
Standard-tier blocker has implementation and evidence, reclassification,
dependency-blocking, or accepted-risk disposition.

## Blockers

None for HUD-012 Browser/WASM text-size and contrast evidence. QA-COND-0005
remains Open overall because non-HUD Standard-tier accessibility rows still need
their own implementation and evidence, reclassification, dependency-blocking, or
accepted-risk disposition.
