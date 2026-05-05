# Story 012: HUD Text Size and Contrast Accessibility Evidence

> **Epic**: HUD
> **Status**: Ready
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 6 S6-04 / QA-COND-0005

## Context

**Sprint Gate**: Sprint 6 S6-04 / QA-COND-0005 Standard-tier accessibility remediation.

**GDD**: `design/gdd/hud.md`

**UX Spec**: `design/ux/hud.md`

**Accessibility Source**: `design/accessibility-requirements.md`

**GDD Requirements**:

- `design/gdd/hud.md` Rule 2: the HUD owns fixed screen-edge zones for phase/round, scoreboard, gold, mana, and reserve mana, with no layout reflow during a match.
- `design/gdd/hud.md` Rule 3: own gold displays `<gold>g` or `<gold>g (<reserved_gold>r)`, mana displays `<current_mana> / <mana_cap>`, reserve mana displays `+<reserve_mana> reserve`, phase label uses the Rule 5 phase strings, and round counter uses the `R<round_number>` format.
- `design/gdd/hud.md` HUD-03: the economy fixture `gold=8`, `current_mana=6`, `mana_cap=10`, `reserve_mana=2`, opponent `gold=6`, `reserved_gold=0` must render own gold `8g`, opponent gold `6g`, mana `6 / 10`, and reserve `+2 reserve`.
- `design/gdd/hud.md` HUD-05: phase labels render exactly `DRAFT INITIAL`, `DRAFT`, `AUCTION`, `PLACEMENT`, `RESOLUTION`, and `GAME OVER`.
- `design/gdd/hud.md` HUD-22: the round counter renders exactly `R<round_number>`, such as `R9`.
- `design/gdd/hud.md` Visual/Audio Requirements: the relative typography hierarchy uses own gold and own mana numerals as the base scale, phase/round and reserve labels at `0.65x`, and the parenthetical reserved suffix at `0.65x` and `65%` opacity.

**Accessibility Requirements**:

- `design/accessibility-requirements.md` A11Y-ST-01: HUD resource counters require a 20px minimum text-size floor at the browser/WASM target, with the auction price counter as a 40px exception owned outside HUD.
- `design/accessibility-requirements.md` A11Y-ST-03: UI text on backgrounds requires at least 4.5:1 contrast, with the auction price counter as a 7:1 exception owned outside HUD.
- `design/ux/hud.md` Accessibility: resource counters have a minimum 20px floor, gold counters have a 40px numeral-height target, and HUD gold counter contrast must be verified against its background.

**QA Condition**: `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` remains Open. The Sprint 6 accessibility disposition register lists A11Y-ST-01 and A11Y-ST-03 as evidence-only required rows that block closure until browser/WASM measurement evidence exists.

**TR IDs**: `TR-HUD-001` for gold label format, `TR-HUD-002` for mana/reserve label behavior, and `TR-HUD-003` for phase and round labels. A11Y-ST-01 and A11Y-ST-03 are Sprint 6 accessibility rows, not registered `TR-HUD-*` requirements.

**ADR Governing Implementation**:

- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)

**ADR Decision Summary**: HUD is a read-only Bevy UI presentation surface inside `HudPlugin`. It reads authoritative presentation state, keeps its entities pre-pooled, and must preserve display strings, update timing, and Lightyear drain ownership while any typography or contrast remediation is applied.

**Engine**: Bevy 0.18 + browser/WASM target | **Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing any Bevy `.rs` file. HUD text must remain Bevy 0.18 `Text`, `TextFont`, `TextColor`, `TextSpan`, and `Node` UI. Do not use `NodeBundle`, `TextBundle`, or world-space sprites for HUD text. Keep `PickingBehavior` behind `#[cfg(feature = "ui_picking")]`.

**Control Manifest Rules (2026-05-05)**:

- Required: UI overlays such as HUD use `bevy_ui`.
- Required: HUD entities are pre-pooled at session entry and toggled or updated in place.
- Required: `S2CGoldUpdate` is drained only by the shared economy-view system; HUD must not add an independent economy drain.
- Required: `S2CPhaseChanged` is drained only by the shared phase sink; HUD reads the shared phase state.
- Forbidden: Client presentation must not assert or mutate authoritative game state.
- Guardrail: Presentation steady-state work stays below 1 ms/frame; phase-boundary presentation spikes stay below 3 ms.

---

## Scope

### In Scope

- Verify or remediate HUD-owned text-size floors for:
  - own gold primary text,
  - opponent gold primary text,
  - DRAFT_AUCTION parenthetical reserved-gold suffixes,
  - current mana text,
  - reserve mana text,
  - phase label text,
  - round counter text,
  - cold-start placeholders for gold and mana.
- Verify DRAFT_AUCTION HUD gold labels with reserved-gold suffixes visible, because these are the auction-linked counters owned by HUD.
- Capture browser/WASM screenshot evidence at `1366x768` and `1920x1080`.
- Record measured text floors in CSS pixels or browser-equivalent rendered pixels for every listed HUD label at both viewports.
- Verify or remediate HUD-owned A11Y-ST-03 contrast for every HUD text/background pair listed in this story.
- Preserve existing HUD string formats, visibility behavior, update timing, entity pre-pooling, and authoritative data flow.
- Add automated or evidence-harness checks that expose text size and contrast measurements without relying only on manual visual judgment.
- Update the HUD-012 evidence document at the exact path listed in `## Test Evidence`.

### Out of Scope

- No code implementation in this story-readiness pass.
- No changes to card text floors, Settings text, Shop/Auction panel body text, shop card text, Hand UI text, board labels, or result screens.
- No implementation or verification of the actual auction price counter, which is owned by Shop/Auction UI. HUD-012 verifies only HUD gold labels while DRAFT_AUCTION reserved-gold suffixes are visible.
- No UI scaling preference implementation.
- No colorblind palette implementation.
- No Settings or Accessibility screen work.
- No changes to gameplay economy values, round state, phase timing, networking authority, or C2S messages.
- No changes to sprint status, session-state files, project asset files, or `AGENTS.md`.
- Do not close QA-COND-0005 from this story alone.

---

## Acceptance Criteria

- [ ] **HUD text-size fixture is captured**: GIVEN browser/WASM HUD evidence runs at `1366x768` and `1920x1080`, WHEN HUD is visible with `gold=11`, `reserved_gold=4`, opponent `gold=8`, opponent `reserved_gold=3`, `current_mana=6`, `mana_cap=10`, `reserve_mana=2`, phase `DRAFT_AUCTION`, and round `9`, THEN the evidence records measured text size for own gold, opponent gold, both reserved-gold suffixes, current mana, reserve mana, phase label, and round counter at both viewports.
- [ ] **Gold counter floor passes**: GIVEN the fixture above, WHEN text-size measurements are reviewed, THEN own and opponent gold primary text each measure at least `40 CSS px` or browser-equivalent rendered pixels at both `1366x768` and `1920x1080`.
- [ ] **Auction-linked HUD suffix floor passes**: GIVEN DRAFT_AUCTION HUD labels display `11g (4r)` and `8g (3r)`, WHEN the parenthetical suffix spans are measured, THEN each visible `(Yr)` suffix measures at least `20 CSS px` or browser-equivalent rendered pixels at both viewports.
- [ ] **Mana and reserve floors pass**: GIVEN the fixture above, WHEN current mana and reserve mana labels are measured, THEN `6 / 10` and `+2 reserve` each measure at least `20 CSS px` or browser-equivalent rendered pixels at both viewports.
- [ ] **Phase and round floors pass**: GIVEN the fixture above, WHEN phase and round labels are measured, THEN `AUCTION` and `R9` each measure at least `20 CSS px` or browser-equivalent rendered pixels at both viewports.
- [ ] **Cold-start placeholders remain readable**: GIVEN HUD has entered DRAFT_INITIAL before receiving economy state, WHEN the own gold placeholder `--g`, opponent gold placeholder `--g`, and mana placeholder `-- / --` are measured, THEN each placeholder meets the same floor as the runtime label it represents.
- [ ] **HUD contrast evidence covers every pair**: GIVEN the HUD text/background pairs are sampled after browser/WASM rendering, WHEN contrast ratios are computed from the composited foreground and immediate background colors, THEN own gold primary text, opponent gold primary text, reserved-gold suffixes, current mana, reserve mana, phase label, round counter, and placeholders each meet at least `4.5:1`.
- [ ] **Contrast survives phase-specific dimming**: GIVEN HUD may dim or change opacity during DRAFT_AUCTION or RESOLUTION, WHEN the text/background pairs are measured in DRAFT_AUCTION and RESOLUTION fixtures, THEN every HUD-owned text pair listed in this story still meets at least `4.5:1` after opacity compositing.
- [ ] **Existing HUD strings are preserved**: GIVEN the existing HUD behavior fixtures run, WHEN text size or contrast remediation is applied, THEN own gold, opponent gold, reserved suffix, mana, reserve, phase, and round strings remain exactly as required by HUD-03, HUD-05, HUD-17, HUD-22, and HUD-29.
- [ ] **Existing visibility behavior is preserved**: GIVEN reserve mana changes from `2` to `0`, WHEN HUD state sync completes, THEN reserve mana visibility still follows `reserve_mana > 0`; no contrast or font-size work leaves stale reserve text visible.
- [ ] **No new steady-state spawning**: GIVEN the HUD entity tree has spawned at session entry, WHEN three later economy and phase updates change displayed HUD values, THEN no additional HUD text entities are spawned during those updates.
- [ ] **Focused accessibility test passes**: `cargo test -p client --test hud_text_size_contrast_accessibility_test` passes. The target must be backed by `tests/integration/hud/text_size_contrast_accessibility_test.rs` and registered in `client/Cargo.toml`.
- [ ] **Existing HUD regression tests stay green**: `cargo test -p client --test hud_gold_mana_display_test --test hud_phase_label_round_counter_test --test hud_economy_auction_inline_gold_test` passes after the change.
- [ ] **Browser/WASM evidence exists**: `production/qa/evidence/hud-012-text-size-contrast-accessibility.md` records the browser, build target, fixture values, capture method, artifact directory, text-size table, contrast table, screenshots, pass/fail verdict, and QA-COND-0005 impact statement.
- [ ] **Browser/WASM capture covers two viewports**: The evidence document links screenshot artifacts for `1366x768` and `1920x1080`, and each capture shows HUD text readable, not clipped, and not overlapping Hand UI, Shop/Auction UI, board content, browser chrome, or other HUD zones.
- [ ] **A11Y-ST-01 impact is explicit**: The evidence document states that HUD-012 verifies HUD-owned A11Y-ST-01 only. Card text and the actual auction price counter remain outside this story.
- [ ] **A11Y-ST-03 impact is explicit**: The evidence document states that HUD-012 verifies HUD-owned A11Y-ST-03 only. Card, Settings, Shop/Auction, Hand UI, board, and result-screen contrast remain outside this story.
- [ ] **QA-COND-0005 remains open**: The evidence document states that QA-COND-0005 remains Open until all remaining Standard-tier rows are implemented and evidenced, reclassified, dependency-blocked, or accepted as risk.
- [ ] **Whitespace gate passes**: `git diff --check` passes.

---

## Implementation Notes

- Keep HUD text ownership in `client/src/ui/hud/`.
- Prefer adjusting `HudConfig`, local typography constants, theme tokens, or HUD-specific `TextFont` and `TextColor` values over changing message handlers or display formatting.
- If the evidence harness reads measured bounds from Bevy UI nodes, keep the marker components stable and test-only where possible.
- If contrast is computed from theme tokens, also validate against at least one browser/WASM screenshot sample so opacity, dimming, and background compositing are represented.
- The `65%` reserved suffix styling from the GDD remains acceptable only if the composited suffix contrast stays at or above `4.5:1`. If opacity makes contrast fail, use another subordinate treatment such as weight, spacing, or color token adjustment while preserving the inline `TextSpan` structure and exact string.
- Do not collapse phase and round into a larger combined label. They remain independently queryable and independently measured.
- Do not move the actual auction price counter into HUD to satisfy this story. Shop/Auction UI owns that counter and its 40px and 7:1 requirements.
- Do not add or edit art asset files; use existing UI typography and color resources.
- If a browser fixture route, debug overlay, or screenshot harness is added, keep it dev/test-only and document the exact URL, phase, fixture values, and capture command in the evidence file.

---

## QA Test Cases

- **HUD text-size measurement**
  - Given: Browser/WASM HUD runs the DRAFT_AUCTION fixture at `1366x768` and `1920x1080`
  - When: text measurements are exported
  - Then: gold primary text is at least `40 CSS px`, and suffix, mana, reserve, phase, and round labels are each at least `20 CSS px`

- **HUD contrast measurement**
  - Given: The same browser/WASM fixture is rendered
  - When: foreground/background pairs are sampled after compositing
  - Then: every HUD-owned text pair listed in this story is at least `4.5:1`

- **Dimming contrast guard**
  - Given: HUD is displayed in DRAFT_AUCTION and RESOLUTION visual states
  - When: contrast is measured for gold, suffix, mana, reserve, phase, and round labels
  - Then: any opacity or dimming still leaves text/background contrast at or above `4.5:1`

- **Existing string regression**
  - Given: Existing HUD regression tests run after the accessibility work
  - When: gold, mana, reserve, phase, round, and DRAFT_AUCTION labels are inspected
  - Then: exact text values remain unchanged from the HUD GDD acceptance criteria

- **No steady-state spawn regression**
  - Given: HUD text entity IDs are captured after session-entry spawn
  - When: economy and phase updates change the displayed values
  - Then: all tracked HUD text entity IDs remain unchanged and no new HUD text entity appears

---

## Test Evidence

**Story Type**: UI

**Required automated test target**:

- `tests/integration/hud/text_size_contrast_accessibility_test.rs`
  - Registered as `hud_text_size_contrast_accessibility_test`
  - Command: `cargo test -p client --test hud_text_size_contrast_accessibility_test`

**Required regression target**:

- `cargo test -p client --test hud_gold_mana_display_test --test hud_phase_label_round_counter_test --test hud_economy_auction_inline_gold_test`

**Required browser/WASM evidence path**:

- `production/qa/evidence/hud-012-text-size-contrast-accessibility.md`

**Required capture artifact directory**:

- `production/qa/evidence/captures/hud-012-text-size-contrast/`

**Required browser/WASM evidence contents**:

- Browser, build target, commit, and capture command.
- Viewports: `1366x768` and `1920x1080`.
- Fixture values: phase `DRAFT_AUCTION`, round `9`, own `gold=11`, own `reserved_gold=4`, opponent `gold=8`, opponent `reserved_gold=3`, `current_mana=6`, `mana_cap=10`, and `reserve_mana=2`.
- Screenshot artifact links under the required capture directory for both viewports.
- Text-size measurement table for own gold, opponent gold, reserved suffixes, current mana, reserve mana, phase label, round counter, and cold-start placeholders.
- Contrast measurement table for each HUD-owned text/background pair, including composited DRAFT_AUCTION and RESOLUTION dimming states.
- Test command output summary for the focused accessibility test and HUD regression targets.
- A11Y-ST-01 impact statement limited to HUD-owned text-size evidence.
- A11Y-ST-03 impact statement limited to HUD-owned contrast evidence.
- QA-COND-0005 impact statement confirming the condition remains Open.

**Status**: [ ] Not yet implemented or captured.

---

## Dependencies

- Depends on: `production/epics/hud/story-001-hud-plugin-scaffold.md` (Story 001, Complete) for the HUD entity pool and root UI hierarchy.
- Depends on: `production/epics/hud/story-002-gold-mana-display.md` (Story 002, Complete) for gold, mana, reserve, and placeholder behavior.
- Depends on: `production/epics/hud/story-003-phase-label-round-counter.md` (Story 003, Complete) for phase and round label behavior.
- Depends on: `production/epics/hud/story-006-economy-auction-inline-gold.md` (Story 006, Complete) for DRAFT_AUCTION inline gold and reserved suffix behavior.
- Depends on: `production/epics/hud/story-010-numeric-tween-animation.md` (Story 010, Complete) to preserve current numeric tween behavior while typography changes.
- Depends on: `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md` A11Y-ST-01 and A11Y-ST-03 rows identifying HUD text size and contrast as QA-COND-0005 closure blockers.
- Depends on: ADR-021 and ADR-002 Accepted.
- Unlocks: HUD-owned A11Y-ST-01 and A11Y-ST-03 can move from evidence-only required to implemented and evidenced after this story is implemented and `production/qa/evidence/hud-012-text-size-contrast-accessibility.md` passes QA review.

## Performance Budget

No measurable gameplay-loop performance impact expected. Text size and contrast work should be limited to fixed HUD typography, theme tokens, and evidence instrumentation. The implementation must not add per-frame entity creation, additional network message drains, texture uploads, or polling work outside existing HUD state sync.

## QA-COND-0005 Impact

This story targets only the HUD-owned portions of A11Y-ST-01 and A11Y-ST-03. Completing it reduces QA-COND-0005 by attaching HUD text-size and HUD contrast evidence, but QA-COND-0005 remains Open until every other Standard-tier blocker has implementation and evidence, reclassification, dependency-blocking, or accepted-risk disposition.

## No Open Questions

No unresolved design question blocks this story. The implementation target is fixed for Sprint 6: HUD gold, mana, reserve, phase, and round text must meet the stated text-size floors and `4.5:1` contrast in browser/WASM evidence while preserving current HUD behavior.
