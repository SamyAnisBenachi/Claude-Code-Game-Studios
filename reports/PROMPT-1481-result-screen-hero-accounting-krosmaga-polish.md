# PROMPT 1481 -- RESULT-SCREEN-HERO-ACCOUNTING-KROSMAGA-POLISH

**Status:** PASS
**Branch:** `work/result-screen-hero-accounting-1481`
**Worktree:** `D:/_DEV/claude-code-game-studios-worktrees/prompt-1481-result-screen`
**Base:** `origin/main@56b5fc0c`

## Scope

Krosmaga-style polish on the result/resolution screen:

- Stronger outcome hero (large headline + outcome-tinted accent stripe + tinted panel border + round chip).
- Readable winner/round state (DISPLAY-sized headline, dedicated round chip).
- Visible accounting (explicit Gold/Mana/Reserve resources line + chunked "Objectives Lost" ledger).
- Clear transition affordance (primary accent-tinted Continue CTA + Enter/Space hint caption on the hero step; the verbose canonical summary line stays as the contract-bearing fallback at the bottom).

Strict ownership: writes contained to `client/src/presentation/result_screen.rs`. Test entry added under `client/Cargo.toml`; tests added under `tests/integration/presentation/`.

## Changes

### `client/src/presentation/result_screen.rs`

1. **Outcome accent palette** — new constants `OUTCOME_ACCENT_{VICTORY, DEFEAT, DRAW, NEUTRAL}` and `CTA_PRIMARY_BG / CTA_PRIMARY_BORDER / CTA_SECONDARY_BG`. Public helper `result_screen_outcome_accent(&headline)` maps the existing outcome headline to the accent — keeps colour selection in one place so the accent stripe, panel border, and any future outcome-tinted chrome stay in sync.
2. **Round chip** — new `ResultScreenRoundChip` marker + `result_screen_round_label(result, snapshot) -> Option<String>` helper. Mounted on the hero panel; hidden when no round is known.
3. **Explicit resources line** — new `ResultScreenResourcesLine` marker + `result_screen_resources_line(snapshot) -> Option<String>` helper. Mounted on the accounting panel; surfaces `Gold N | Mana C/M | Reserve R` in accent-gold body type.
4. **Compact ledger line** — new `ResultScreenLedgerLine` marker + `result_screen_ledger_line(snapshot) -> String` helper. Renders `Objectives Lost — You: X real / Y fake • Opponent: Z real / W fake` above the verbose canonical summary (which is preserved unchanged so existing tests keep their contract).
5. **Continue hint** — new `ResultScreenContinueHint` marker mounted under the hero panel ("Press Enter or Space to view round accounting."). Hidden when the user advances to the accounting step.
6. **Outcome accent stripe** — new `ResultScreenAccentStripe` marker, a 6 px tinted bar at the panel's left edge. The panel itself is now a `FlexDirection::Row` container with the stripe sibling to a new content column. Outer panel border also tints to a low-alpha version of the outcome accent at sync time.
7. **Hero-step typography boost** — headline promoted from `typography::H1` to `typography::DISPLAY` so the outcome word dominates the screen the way Krosmaga's victory/defeat hero does. The summary line drops to `typography::CAPTION` on the accounting panel to put weight on the new resources + ledger rows.
8. **CTA primary/secondary distinction** — `spawn_result_cta_button` extended with a `primary: bool` and explicit `border: Color` parameter. Continue is primary (200×50, H2 type, accent gold). Return-to-Lobby is secondary (176×46, H3 type, neutral border).
9. **Layout-safe layout** — panel still `max_width: 860 px` / `max_height: 92%` / 88% width and uses `FlexWrap::Wrap` on the objective grid. The 1280×720 safety viewport keeps both new lines and the action row in view; no absolute positioning was introduced.

`ResultScreenEntities` now carries `panel`, `accent_stripe`, `round_chip`, `continue_hint`, `resources_line`, `ledger_line` so the sync pass can drive the new chrome.

`sync_result_screen_ui_system` now:

- Tints the accent stripe and panel border from `result_screen_outcome_accent`.
- Drives the round chip text and visibility from `result_screen_round_label`.
- Drives the resources line text and visibility from `result_screen_resources_line`.
- Drives the ledger line text from `result_screen_ledger_line`.
- Toggles the continue hint with the existing Hero/Accounting step toggle.

### `tests/integration/presentation/result_screen_hero_accounting_polish_test.rs` (new, 7 tests)

| Test | Verifies |
|------|----------|
| `outcome_accent_is_distinct_per_outcome_class` | Victory ≠ Defeat ≠ Draw accents; Pending = NoResult (neutral); neutral ≠ any real outcome. |
| `round_label_prefers_result_round_over_snapshot_round` | "Round N" labelled from the authoritative result first, snapshot second, `None` when both are absent. |
| `resources_line_surfaces_local_player_gold_mana_reserve` | Gold/Mana c/m/Reserve readouts present; `None` without a snapshot. |
| `ledger_line_chunks_own_and_opponent_real_fake_losses` | Header, own/opponent labels, and real/fake counts surface. |
| `round_chip_mounts_visible_with_text_on_hero_step` | Round chip mounted (`Display::Flex`), single marker, text contains "Round 9". |
| `continue_hint_mounts_on_hero_then_hides_on_accounting` | Visible on Hero step; `Display::None` after `AdvanceToAccounting`. |
| `resources_and_ledger_lines_render_on_accounting_panel` | Resources line shows Gold/Mana readouts; ledger line shows "Objectives Lost"; both have exactly one marker. |

### `client/Cargo.toml`

One `[[test]]` entry registers the new focused polish test crate. No other Cargo edits.

## Validation

Focused tests (no broad workspace runs per PROMPT scope; full Cargo VERIFY is scheduled separately):

```
cargo test --test result_screen_mvp_test \
           --test result_screen_return_to_lobby_test \
           --test result_screen_hero_accounting_polish_test
```

Results:

- `result_screen_mvp_test`: 11 passed (existing two-step contract, headline copy, focus order, single S2CGameOver drain, reduced motion).
- `result_screen_return_to_lobby_test`: 2 passed (dedupe + cleanup contract).
- `result_screen_hero_accounting_polish_test`: 7 passed (new polish primitives).

Total: **20 / 20 passing**. No tests outside `tests/integration/presentation/` were modified.

## Gameplay semantics preserved

- `S2CGameOver` drain idempotency (`S13-LATE-MSG-DEDUPE-001`), single drainer of `MessageReceiver<S2CGameOver>`, no `MessageReceiver<S2CGameSnapshot>` introduced.
- Two-step state machine (`ResultScreenStep::{Hero, Accounting}`) unchanged: spawn always opens on Hero; Continue/Enter/Space advance to Accounting; Enter/Space on Accounting commits Return-to-Lobby.
- Focus order length: 2 on Hero ([Continue, Return]), 1 on Accounting ([Return]) — unchanged.
- `C2SAcknowledgeResult` send remains gated on the Return-to-Lobby commit, never on the Hero→Accounting advance.
- Canonical verbose summary line preserved verbatim (`Round R | Resources | Own real lost N | ...`); regression tests asserting on its contents continue to pass.

## Layout safety

- Outer panel: `width: 88%, max_width: 860 px, max_height: 92%`, padding 14/26/22/22, row-flex content column. At 1280×720 the safe area is 1126.4 × 662.4 — the panel fits with margin on all sides.
- Objective grid still uses `FlexWrap::Wrap` with `min_width: 270 px` columns, so it reflows below 540 px content width.
- No `PositionType::Absolute` introduced; all new entities flow inside the existing content column.

## Out of scope (deferred)

- No Krosmaga raster assets copied; visual identity stays code-driven for friend-game scope (PAW-TD-* boundary).
- Standard-tier accessibility, reduced-motion entry animations on the new entities, and a sound cue at result-time are deferred — `ResultScreenMotionState` and `AccessibilityPreferences` plumbing are preserved untouched so a follow-on can wire them.
- Per-class hero portrait is intentionally not added; class identity is still surfaced via the existing class persona text line on the hero panel.

## Git flow

- Worktree: `D:/_DEV/claude-code-game-studios-worktrees/prompt-1481-result-screen`
- Branch: `work/result-screen-hero-accounting-1481`
- Commit pushed to the worker branch only. `main` not touched.

1481: RESULT-SCREEN-HERO-ACCOUNTING-KROSMAGA-POLISH: PASS
