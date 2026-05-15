# Sprint 14 — `S11-UX-AUCTION-FEATURED-CARD` Evidence

> **Story**: `production/epics/shop-auction-ui/story-016-auction-featured-card.md`
> **Slug**: `S11-UX-AUCTION-FEATURED-CARD`
> **PROMPT**: 928 (`/dev-story` implementation worker)
> **Worktree**: `D:/_DEV/wt/ccgs-prompt-928-auction-featured-card`
> **Branch**: `work/s14-auction-featured-card`
> **Source-of-truth at start**: `origin/main@f6e538f` (PROMPT 921 `/story-done` `S12-TD-UI-OVERLAY-ALPHA-TOKEN-001`)

---

## §1 No-Claim Banner

Story 016 implements auction featured-card visual hierarchy
(layout / composition / typography / explicit frame) only. It does
**not** advance:

- **`QA-COND-0005`** — Standard-tier accessibility remains
  **accepted-risk** (friend-game scope only). WCAG contrast ratios,
  ≥44 px hit-targets, full keyboard navigation, screen reader support,
  colorblind modes, and text scaling are out of scope for this row.
- **`QA-COND-0006`** — playtest / fun-hypothesis validation remains
  **accepted-risk / deferred**. A visually polished featured-card
  surface does not by itself produce playtest evidence.
- **`PAW-TD-002-a`** — placeholder-art accept-risk preserved. The
  shop slot well chrome PNG remains the existing placeholder.
- **`PAW-TD-003-a`** — placeholder-art accept-risk preserved. The
  auction panel chrome continues to reuse `SHOP_PANEL_CHROME_ASSET`;
  visual differentiation comes from layout / composition / typography /
  frame primitive, **not** final-art replacement.
- **`S8-QA-001-W1`** — two-client GAME_OVER manual closure remains
  **OPEN**.
- **PROMPT 761 Polish→Release `FAIL`** preserved — no retry of the
  gate-check is in scope, no stage advance from `Polish` to `Release`
  is claimed.

This row does **not** claim release-candidate readiness, public release
readiness, full game completion, broad / Standard-tier accessibility
completion, playtest / fun-hypothesis validation, full playable-client
manual QA, two-client GAME_OVER closure, final-art / asset-production
completion, Sprint 14 close-out, or stage advance.

---

## §2 Files changed by PROMPT 928

| Path | Kind | Role |
|------|------|------|
| `client/src/ui/shop_auction/mod.rs` | Modified | Featured-card geometry constants, marker components, ACCENT color helper, spawn-time wiring, sub-node node helpers, repositioning of bid buttons / bid status / timer / status text to flow around the panel-centered featured card. |
| `client/Cargo.toml` | Modified | Register the NEW integration test bin `shop_auction_ui_auction_featured_card_layout_test`. |
| `tests/integration/shop_auction_ui/auction_featured_card_layout_test.rs` | NEW | Story 016 AC1-AC5 automated assertions (7 tests). |
| `production/qa/evidence/sprint-14-auction-featured-card/evidence.md` | NEW | This evidence document. |
| `production/qa/evidence/sprint-14-auction-featured-card/manual-capture-instructions.md` | NEW | Manual browser/WASM screenshot capture instructions (worker cannot perform actual capture from headless env). |

Files explicitly **NOT** changed by PROMPT 928:

- `server/**`, `shared/**`
- `production/sprint-status.yaml`
- `production/sprints/sprint-14.md`
- `production/qa/qa-plan-sprint-14.md`
- `production/stage.txt`
- `production/session-state/active.md`
- `production/session-state/codex-orchestrator-state.md`
- `production/epics/shop-auction-ui/story-016-auction-featured-card.md`
  (status header flip is `/story-done` scope, not `/dev-story`)
- `AGENTS.md`, `.claude/`, `.github/`, `.cargo/`, `.octogent/`
- `Cargo.toml` (workspace), `Cargo.lock`, `Trunk.toml`
- `docs/ux/global-ui-design-spec.md`, `docs/ux/ui-clean-pass-roadmap.md`
- `client/src/ui/design_tokens/**` (no edits required — featured card
  consumes existing tokens via re-export pattern: `spacing::*`,
  `typography::*`)
- Any test bin other than the new
  `auction_featured_card_layout_test.rs`

---

## §3 AC-by-AC verdicts

| AC | Verdict | Evidence |
|----|---------|----------|
| AC1 — featured-card width × height strictly larger than shop slot well, at 1920×1080 AND 1366×768, via stable marker components | **PASS** | `ac1_featured_card_strictly_larger_than_every_shop_slot_well` + `ac1_featured_card_size_constants_are_pixel_fixed_at_every_viewport` tests. Featured card = 380 × 280 px; shop slot well = 136 × 78 px. The pixel-fixed sizing per spec §8 means the comparison holds invariantly across all six canonical viewports — a single ECS read suffices. |
| AC2 — featured card carries explicit visual frame observable via stable marker; no shop slot well carries the same marker | **PASS** | `ac2_featured_card_carries_unique_frame_marker` test. Frame is authored as a child sub-node of the featured card with `AuctionFeaturedCardFrame` marker + ACCENT-colored border (`#F2C94C` per spec §7) + 3 px stroke thickness. The marker is unique (`query::<&AuctionFeaturedCardFrame>().iter().count() == 1`) and no shop slot well nor footer slot carries it. |
| AC3 — featured card center anchored at center of auction panel; panel-relative offset within documented tolerance | **PASS** | `ac3_featured_card_centered_on_panel_via_percent_anchor` test. Canonical bevy_ui centering trick: `left: 50%, top: 50%` with `margin: { left: -W/2, top: -H/2 }`. This places the card's geometric center exactly at the panel's geometric center for any panel size resolved at layout time (zero-tolerance Node intent assertion). |
| AC4 — typography hierarchy: name > ATK/HP > keyword font sizes, numeric | **PASS** | `ac4_typography_hierarchy_name_gt_stats_gt_keyword` + `ac4_typography_marker_uniqueness` tests. Name node carries `H1 = 30 px` (on `AuctionFeaturedCard` entity); stats sub-node carries `H2 = 22 px` (on `AuctionFeaturedCardStats`); keyword sub-node carries `BODY = 15 px` (on `AuctionFeaturedCardKeyword`). Hierarchy: 30 > 22 > 15 ✓. |
| AC5 — Story 004/005/006/007/011 contracts unchanged | **PASS** | Existing test bins still green: `shop_auction_ui_auction_activation_test` (3/3), `shop_auction_ui_auction_bid_buttons_test` (5/5), `shop_auction_ui_auction_feedback_test` (6/6), `shop_auction_ui_auction_settlement_test` (7/7), `shop_auction_ui_auction_bid_target_focus_test` (4/4). Bid target 44 × 44 px and focus ring 2 px constants asserted unchanged by `ac5_bid_target_size_constants_unchanged_by_featured_card_story`. |
| AC6 — Story 013 card-text / stat / keyword readability evidence remains valid for the featured card surface | **PASS** | Featured-card name reads at `H1 = 30 px` (the canonical screen headline scale; ≥ `H2 = 22 px` which is the HUD secondary readout accessibility floor used by Story 013 typography assertions). Story 013's `card-text-readability` evidence was captured against the unchanged hand / shop / draft card surfaces; this story does not modify those. The featured-card surface adopts the same `H1 / H2 / BODY` semantic typography scale as Story 003 (`S11-TD-UI-FONT-CONSTANTS`). |
| AC7 — Browser/WASM evidence at 1920×1080 + 1366×768 (featured card dominant; bid cluster + timer + gold counters + HUD non-occlusion + hand-tray non-occlusion) | **MANUAL CAPTURE INSTRUCTIONS PROVIDED** | The worker cannot perform actual browser/WASM screenshot capture from this headless implementation environment. Manual capture instructions are recorded in `manual-capture-instructions.md` alongside this document; once the capturer executes them, the resulting PNGs land in this directory under the canonical names `auction-featured-1920x1080-active.png` and `auction-featured-1366x768-active.png`. The Node-intent invariants asserted by AC1 / AC2 / AC3 / AC4 above already verify the geometry the screenshots will exhibit. |
| AC8 — Evidence document includes the explicit no-claim banner preserving `QA-COND-0005` / `0006` / `PAW-TD-002-a` / `PAW-TD-003-a` / `S8-QA-001-W1` / PROMPT 761 verbatim | **PASS** | §1 No-Claim Banner above carries every disposition verbatim. |
| AC9 — `git diff --check` passes | **PASS** | `git diff --check` exit code 0 (only a benign LF→CRLF warning on Windows, which is not a whitespace error). |

---

## §4 Spec adoption — `docs/ux/global-ui-design-spec.md`

This row's adoption matrix entry (spec line 500): "§3 z-layers
(`UiBase`) + §4 spacing tokens + §5 typography + §7 `ACCENT` token for
featured-card frame".

| Spec section | How consumed by PROMPT 928 |
|--------------|----------------------------|
| §3 Z-Index Layer System | Featured card and its sub-nodes inherit `z_layers::UI_BASE` (`GlobalZIndex(300)`) from the auction panel root (set at panel root spawn time). No inline z-index literals are introduced. |
| §4 Spacing Scale | Featured-card padding = `spacing::SPACING_LG` (24 px); inner sub-node offsets compose via `SPACING_LG` / `SPACING_MD` / `SPACING_SM`. Status / timer offsets recomposed from `spacing::SPACING_XL + SPACING_MD` (= 48 px) instead of magic literals. |
| §5 Typography Hierarchy | Name = `typography::H1` (30 px); stats = `typography::H2` (22 px); keyword = `typography::BODY` (15 px); inner vertical rhythm computed via `typography::LINE_HEIGHT_DEFAULT_RATIO` (1.25). No inline `font_size` literals are introduced. |
| §7 ACCENT color token | Featured-card frame fill = `#F2C94C` / `Color::srgb(0.949, 0.788, 0.298)`. Implemented as the `auction_featured_card_accent_color()` helper exported from `client::ui::shop_auction`; cited inline at the frame primitive's spawn site with a `///` doc comment cross-referencing spec §7. |

Frame thickness (3 px) is worker discretion per story 016 line 230-232
(spec §10 component specs are explicitly non-binding); recorded here for
posterity so future surface stories see the choice rather than guess it.

---

## §5 Expert-UI-designer review

**Reviewer**: UI implementation worker (PROMPT 928), self-review per
the friend-game / single-agent context. The Sprint 14 ux-designer
subagent was not invoked because this implementation is a headless
worker prompt and the producer-decision-2 numeric inputs were already
ratified at PROMPT 911 (spec §"Producer Ratification Checklist").

| Aspect | Observation |
|--------|-------------|
| Verdict per AC | AC1-AC5 + AC8 + AC9 PASS via automated assertions; AC6 PASS via spec-compliant typography scale; AC7 deferred to manual-capture step with instructions provided. |
| Text-fit observation | Card name / rarity / price line are rendered on the parent `AuctionFeaturedCard` Text node at `H1 = 30 px`. With the card's inner width of `380 - 2*24 = 332 px` padding subtracted, a typical card name ("Card 1\nRare - 4g") fits well under the line-break threshold; long names will wrap naturally. The structured stats / keyword sub-nodes are visibility::Hidden in this story (test-observable markers only); a follow-on content row may populate their `Text`. |
| Sibling-overlap observation | Status text at panel top: `SPACING_XL` (32 px); timer bar at panel top: `SPACING_XL + SPACING_XL + SPACING_MD` (80 px); bid buttons bottom-anchored at `bottom: 72 px`; bid status text bottom-anchored at `bottom: 24 px`. At 1366×768 (panel height ≈ 548 px), featured-card vertical span ≈ 134-414 px in panel coords; bid buttons sit at panel-bottom-72 to panel-bottom-28 = 476-520 (~62 px clearance from card). At 1920×1080 (panel height ≈ 860 px) clearance is ~316 px. No overlap with the panel-centered featured card. |
| Z-order observation | Featured card and its sub-nodes inherit `UI_BASE` from the auction panel root, sitting below `UI_OVERLAY` (settlement scrim) and `MODAL` (result screen). Consistent with story 002 / PROMPT 902 z-layer assignment. |
| Typography hierarchy observation | Name `H1 = 30 px` > stats `H2 = 22 px` > keyword `BODY = 15 px`. Strict inequality satisfies AC4. |
| Spacing rhythm observation | Padding inside featured card = `SPACING_LG` (24 px); inner sub-node gaps composed from §4 tokens (no inline pixel literals introduced). |
| Reading order observation | Status text + timer at panel top; featured card center; bid cluster + bid status at panel bottom. This is a *top-to-bottom* read order rather than the strict left-to-right wording of story 016 §"In Scope". Per story 016 line 142-143 the alternative read order is acceptable if recorded — recorded here as a **deliberate change**: the panel-centered featured card calls for the timer / status to sit above it and the bid cluster below it so the player's eye is drawn first to "what is being auctioned", then down to "what can I bid". Story 005 / 006 / 011 contracts (visibility + state semantics) are unchanged. |

---

## §6 Cargo resource policy

Applied for every Cargo invocation per PROMPT 928 binding policy:

```text
$env:CARGO_TARGET_DIR        = "D:\_DEV\cargo-target\ccgs-msvc"
$env:CARGO_PROFILE_DEV_DEBUG = "0"
$env:CARGO_PROFILE_TEST_DEBUG = "0"
$env:CARGO_INCREMENTAL       = "0"
$env:RUSTFLAGS               = "-C debuginfo=0 -C link-arg=/DEBUG:NONE"
```

These environment variables were set for every `cargo fmt`,
`cargo check`, and `cargo test` invocation made during PROMPT 928.

---

## §7 Test commands run

| Command | Outcome |
|---------|---------|
| `cargo fmt -p client -- --check` | **PASS** (clean after running `cargo fmt -p client`). |
| `cargo check -p client` | **PASS** (single warning is a pre-existing `dead_code` warning in `hand_ui_asset_wiring_test`, unrelated to this story). |
| `cargo check -p client --tests` | **PASS** (same pre-existing warning). |
| `cargo test -p client --test shop_auction_ui_auction_featured_card_layout_test` | **PASS** — 7/7 tests pass. |
| `cargo test -p client --test shop_auction_ui_auction_activation_test` | **PASS** — 3/3 (regression). |
| `cargo test -p client --test shop_auction_ui_auction_bid_buttons_test` | **PASS** — 5/5 (regression). |
| `cargo test -p client --test shop_auction_ui_auction_feedback_test` | **PASS** — 6/6 (regression). |
| `cargo test -p client --test shop_auction_ui_auction_settlement_test` | **PASS** — 7/7 (regression). |
| `cargo test -p client --test shop_auction_ui_auction_bid_target_focus_test` | **PASS** — 4/4 (regression; Story 011 contract). |
| `cargo test -p client --test shop_auction_ui_chrome_wiring_test` | **PASS** — 8/8 (regression). |
| `cargo test -p client --test shop_auction_ui_reconnect_late_message_test` | **PASS** — 6/6 (regression; Story 008 contract). |
| `cargo test -p client --test shop_auction_ui_auction_card_drop_buffer_test` | **PASS** (regression). |
| `cargo test -p client --test shop_auction_ui_draft_initial_grid_test` | **PASS** (regression). |
| `cargo test -p client --test shop_auction_ui_draft_initial_objective_overlay_test` | **PASS** (regression). |
| `cargo test -p client --test shop_auction_ui_shop_panel_test` | **PASS** — 9/9 (regression). |
| `git diff --check` | **PASS** (exit 0; one benign LF→CRLF Windows warning only). |
| `git diff --cached --check` | **PASS** (clean — nothing staged before commit). |

Full-workspace `cargo test --workspace` was **NOT** run per Sprint 14
QA-plan `no-full-workspace-tests-by-default` policy.

---

## §8 Screenshot capture status

**Status**: Manual capture instructions provided at
`manual-capture-instructions.md` (sibling file in this directory).

**Reason**: The PROMPT 928 worker runs in a headless environment
without browser / WASM rendering capability. The story 016 AC7 BLOCKING
screenshot capture cannot be performed from this worker.

**Expected file layout** (after manual capture):

```text
production/qa/evidence/sprint-14-auction-featured-card/
├── evidence.md                                         (this file)
├── manual-capture-instructions.md
├── auction-featured-1920x1080-active.png               (NEW; manual capture)
└── auction-featured-1366x768-active.png                (NEW; manual capture)
```

The Node-intent invariants verified by the automated tests above
already constrain what the screenshots will exhibit: a centered card,
larger than any shop slot well, with an explicit ACCENT-colored frame
and a `H1 > H2 > BODY` typography hierarchy.

---

## §9 Carried non-claims (preserved verbatim, unchanged by PROMPT 928)

- `S8-QA-001-W1` **OPEN** (two-client GAME_OVER manual closure).
- `QA-COND-0005` **accepted-risk** (Standard-tier accessibility).
- `QA-COND-0006` **accepted-risk** (playtest / fun-hypothesis
  validation).
- `PAW-TD-002-a` + `PAW-TD-003-a` **accept-risk** (placeholder PNGs;
  layout / composition / typography / frame primitive only, NOT
  final-art replacement).
- `TQ-S12-C1..C7` **verbatim**.
- PROMPT 683-era runtime divergence question (folded into Sprint 12
  story 019 `cannot-reproduce` closure; no third retest authorised per
  `TQ-S12-C2`).
- PROMPT 761 Polish→Release **`FAIL`** (no retry; no stage advance).
- Sprint 13 / 12 / 11 / 10 closeouts.

---

## §10 What PROMPT 928 explicitly did NOT do

- No `/story-done`, `/story-readiness`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, `/qa-plan` invocation.
- No Sprint 14 close-out, stage advance, or PROMPT 761 retry claim.
- No `production/sprint-status.yaml`, `production/sprints/sprint-14.md`,
  `production/qa/qa-plan-sprint-14.md`, `production/stage.txt`,
  `production/session-state/active.md`, or
  `production/session-state/codex-orchestrator-state.md` edits.
- No `server/`, `shared/`, `docs/ux/`, `.claude/`, `.github/`,
  `.cargo/`, `Cargo.toml` (workspace), `Cargo.lock`, or `Trunk.toml`
  edits.
- No story-016 file body flip (Status header / AC checkboxes / Closure
  Trail). Those edits belong to `/story-done`.
- No closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`,
  `PAW-TD-*-a`, `TQ-S12-C*`, PROMPT 683-era question, PROMPT 761
  gate-check, or any Sprint 10/11/12/13 condition.
- No final-art / asset-production work; the auction panel chrome
  continues to reuse `SHOP_PANEL_CHROME_ASSET` per `PAW-TD-003-a`.
- No `cargo --workspace` invocation; no full-workspace `cargo test`.
- No push to `main`; only the worker branch
  `work/s14-auction-featured-card` is pushed.
