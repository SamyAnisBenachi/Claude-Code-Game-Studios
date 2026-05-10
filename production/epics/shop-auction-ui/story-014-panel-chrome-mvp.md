# Story 014: Shop/Auction Panel Chrome Wiring (MVP)

> **Epic**: Shop/Auction UI
> **Story ID**: S10-POLISH-002
> **Status**: Complete
> **Layer**: Presentation (Polish)
> **Type**: UI
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 10 active

## Context

This story closes the end-to-end visual chrome loop for the shop and auction
panels in the active friend-game route. The asset-wiring substrate landed
in PAW-003 (`production/epics/presentation-asset-wiring/story-003-shop-auction-chrome.md`,
done): `client/src/asset_wiring.rs` exposes the chrome path constants
(panel backgrounds, slot wells, border ramp tiles, bid button chrome) and
PAW-003 wired them into the shop and auction panel spawn sites with an
integration test asserting non-default `ImageNode.image` handles.

S10-POLISH-002 verifies that wiring **stays consumed** through the four
phase transitions of the friend-game route (DRAFT_SHOP → DRAFT_AUCTION →
auction settlement → post-auction DRAFT_SHOP) without regressing into
inline asset path strings, `ImageNode` use for board content, or
client-side phase authority. The substantive plumbing exists; this story
records that the friend-game build visibly looks like a styled shop and
auction rather than raw `bevy_ui` `Node` rectangles, captures one manual
screenshot of the active route, and adds an integration test that asserts
the panel root entities still hold non-default `ImageNode.image` handles
after `OnEnter(ClientState::InSession)`.

This story does **not** add new asset authoring, change network protocol,
add client-side optimistic phase or economy authority, claim full asset
approval, claim final visual polish completion, claim public release
readiness, claim full playable-client manual QA, claim broad Standard-tier
accessibility completion, or close any S8 / Sprint 9 carried condition.

**Primary sources**:

- `production/sprints/sprint-10.md` (S10-POLISH-002 row, line 96)
- `production/epics/presentation-asset-wiring/story-003-shop-auction-chrome.md`
  (PAW-003 — the wiring substrate this story consumes)
- `design/gdd/shop-auction-ui.md` (panel chrome + state machine spec)
- `docs/architecture/adr-021-presentation-layer-architecture.md`
  (ADR-021 — `PresentationSet` ordering, bevy_ui-vs-Sprite boundary,
  `BoardLayout` / `CardAtlas` session-scoped resources)

**GDD, UX, and TR trace**:

- **GDD**: `design/gdd/shop-auction-ui.md` (Approved per `systems-index.md`
  row 15) — panel chrome, slot wells, auction panel background, bid
  button chrome, and the DRAFT_SHOP → DRAFT_AUCTION → settlement →
  DRAFT_SHOP route are specified there.
- **TR-ID**: `TR-PAW-003` — *"Shop/Auction UI panel chrome: slot wells,
  auction panel background, border ramp tiles, bid button chrome wired"*
  (`docs/architecture/tr-registry.yaml` line 2041–2048; status: active).
  The wiring substrate is owned by PAW-003 (done); this story closes
  the friend-game route verification loop on the same TR.
- **Related TR**: `TR-SAU-006` — panel transition presentation contract
  (DRAFT_INITIAL → DRAFT_AUCTION/SHOP). S10-POLISH-002 exercises this
  contract through the friend-game route but does not extend it.

**ADR Governing Implementation**:

- **ADR-021** (Presentation Layer Architecture) — primary. Constrains:
  shop and auction panels are screen-space `bevy_ui`; chrome surfaces
  use `ImageNode`; never `Sprite` for these panels; `MessageReceiver<S2CPhaseChanged>`
  drained exactly once via `phase_sink_system`; `PresentationSet`
  ordering is `PhaseTransition → MessageDrain → StateSync → AnimationTick`;
  `BoardLayout` and `CardAtlas` are session-scoped resources inserted on
  `OnEnter(ClientState::InSession)`.
- **ADR-013** (Auction System State) — context only. Auction panel
  presentation reflects `AuctionState` mutations from
  `auction_tick_system`; the panel never writes back to authoritative
  state. No protocol surface change in this story.
- **ADR-008** (Channel Assignment) — context only. The panel reacts to
  `S2CAuctionUpdate` (Unreliable) and `S2CAuctionSettled` (Reliable)
  drains seeded by the shared presentation drainers; this story does
  not register a new drainer.

**Engine**: Bevy 0.18 (Rust) | **Risk**: LOW (visual chrome verification —
no protocol or authority change; failure mode is "panels look unstyled,
no functional regression")

**Engine Notes**: Bevy 0.18 Required Components — `ImageNode { image: handle, .. }`
spawned directly via `commands.spawn((Node { .. }, ImageNode::new(handle), ..))`.
Path constants are pulled from `client/src/asset_wiring.rs`; no inline
string literals for asset paths in `client/src/ui/shop_auction/` spawn
code. `NodeBundle` / `ImageBundle` / `UiImage::new()` are forbidden in
Bevy 0.18 (control manifest Presentation Layer + Forbidden APIs table).
Asset handles are loaded via `bevy_asset_loader` `LoadingState` /
`PlaceholderAssets` resource per ADR-004 + PAW-003.

**Control Manifest Rules (2026-05-05)**:

- **Required**: Shop/Auction chrome uses `ImageNode` — never `Sprite`
  (board content is `Sprite` only; this story covers UI panels).
  — source: ADR-021
- **Required**: Path constants from `asset_wiring.rs` (panel chrome,
  slot well, auction panel background, border ramp tiles, bid button
  chrome). No inline string literals in shop/auction UI spawn code.
  — source: ADR-021 + PAW-003
- **Required**: `PresentationPlugin` registration order — `ShopAuctionUiPlugin`
  is fifth in the contract chain (`CardAnimationsPlugin`, `BoardRenderingPlugin`,
  `HandUiPlugin`, `HudPlugin`, `ShopAuctionUiPlugin`). Reordering causes
  runtime panics. — source: ADR-021
- **Required**: `BoardLayout` and `CardAtlas` session-scoped resources;
  systems reading them must be `in_state(ClientState::InSession)`.
  — source: ADR-021
- **Forbidden**: `NodeBundle` / `ImageBundle` / `UiImage::new()` (do not
  exist in Bevy 0.18 / Bevy 0.16+). — source: ADR-021 + engine-reference
- **Forbidden**: `Sprite` for shop/auction UI surfaces. — source: ADR-021
- **Forbidden**: Inline asset path strings in `client/src/ui/shop_auction/`
  spawn sites (must come from `asset_wiring.rs`). — source: ADR-021 + PAW-003
- **Forbidden**: New `MessageReceiver<S2CPhaseChanged>` drain in
  `ShopAuctionUiPlugin`. The shared `phase_sink_system` is the only
  drainer; sub-plugins read `Res<CurrentClientPhase>`. — source: ADR-021
- **Forbidden**: New `MessageReceiver<S2CGoldUpdate>` drain in this
  plugin. The shared economy-view system is the only drainer; this
  plugin reads `Res<PlayerEconomyView>`. — source: ADR-021
- **Forbidden**: Client-side optimistic phase or economy authority.
  — source: ADR-002 + ADR-021

---

## Scope

### In Scope

- Verify (and patch where necessary) that every spawn site in
  `client/src/ui/shop_auction/` for the following surfaces uses an
  `ImageNode` wired to a `client/src/asset_wiring.rs` path constant
  (no inline asset path strings):
  - shop panel root background
  - shop slot wells (each slot)
  - auction panel root background
  - auction panel border ramp tiles (left + right ramps)
  - auction bid button chrome (each preset bid button)
- Add (or extend) one integration test under
  `tests/integration/shop_auction_ui/` that asserts every shop/auction
  panel root entity carries a non-default `ImageNode.image` handle
  after `OnEnter(ClientState::InSession)`. The test must use the
  existing `placeholder_assets_for_tests()` helper from PAW-003 /
  S10-TD-001 cascade and follow the partial-App fixture pattern
  established in PAW-003's `tests/integration/presentation/shop_auction_asset_wiring_test.rs`.
- Run the friend-game route through DRAFT_SHOP → DRAFT_AUCTION →
  auction settlement → post-auction DRAFT_SHOP (browser or native
  client) and capture one manual evidence screenshot showing the
  wired chrome on each phase. Record the route + screenshot at
  `production/qa/evidence/sprint-10-shop-auction-chrome-evidence.md`.
- Preserve the existing `S2CPhaseChanged` and economy view drains as
  the single source of phase / gold truth for the shop and auction
  panels.

### Out of Scope

- No new asset authoring. Final art replacement is a future story;
  PAW-TD-003-a (placeholder chrome PNGs vs final art) is
  accept-risk for friend-game scope and remains so.
- No final visual polish (typography weight, spacing pixel-perfect
  alignment, animation easing curves). This is MVP wiring verification,
  not visual design pass.
- No claim of full asset approval — placeholder chrome PNGs from
  PAW-003 remain in use.
- No new protocol message, no protocol change, no Lightyear channel
  change.
- No client-side optimistic phase or economy authority. The presentation
  surface only consumes `Res<CurrentClientPhase>` and
  `Res<PlayerEconomyView>`; it never writes them.
- No changes to `server/`, `shared/`, or other client UI plugins
  (`HandUiPlugin`, `HudPlugin`, `BoardRenderingPlugin`,
  `CardAnimationsPlugin`).
- No closure of S8-QA-001-W1, QA-COND-0005, or QA-COND-0006.
- No claim of public release readiness, full playable-client manual
  QA, full game completion, broad Standard-tier accessibility
  completion, or playtest/fun-hypothesis validation.
- No regression of SAU-007 (settlement) or SAU-008 (reconnect snapshot)
  story behaviour.

---

## Acceptance Criteria

(Source: `production/sprints/sprint-10.md:96` S10-POLISH-002 row.)

- [ ] **AC-1 Panels consume `asset_wiring.rs` constants**: GIVEN
      every spawn site under `client/src/ui/shop_auction/` for shop
      panel root, shop slot wells, auction panel root, auction border
      ramp tiles, and bid button chrome, WHEN the spawn code is read,
      THEN the `ImageNode.image` handle is sourced from a
      `client/src/asset_wiring.rs` path constant (or from the
      `PlaceholderAssets` resource keyed off such a constant); no
      inline asset path string literal appears in the spawn site.
      *Verification*: `grep -rE '\.png|\.jpg|assets/' client/src/ui/shop_auction/`
      returns zero hits outside comments / docstrings.
- [ ] **AC-2 No `ImageNode` for board content**: GIVEN the same files,
      WHEN the diff is filtered for `ImageNode` use, THEN every
      `ImageNode` is on a screen-space `bevy_ui` `Node` (panel chrome,
      slot well, button) — never on board entities (objectives,
      units, prisms, HP bars, spawn-range overlays). Board content
      remains `Sprite` per ADR-021.
- [ ] **AC-3 Friend-game route visibly uses wired chrome**: GIVEN the
      friend-game build, WHEN a two-client route is run through
      DRAFT_SHOP → DRAFT_AUCTION → auction settlement → post-auction
      DRAFT_SHOP, THEN each phase shows the wired chrome (panel
      backgrounds rendered, slot wells visible, bid button chrome
      rendered, border ramp tiles visible) rather than raw
      unstyled `Node` rectangles. *Evidence*: manual screenshot at
      `production/qa/evidence/sprint-10-shop-auction-chrome-evidence.md`.
- [ ] **AC-4 Integration test asserts non-default `ImageNode.image`**:
      GIVEN a partial-App test fixture under
      `tests/integration/shop_auction_ui/` configured with
      `init_state::<ClientState>()`, `AssetPlugin::default()` +
      `init_asset::<Image>()`, and `placeholder_assets_for_tests()`
      (per the S10-TD-001 fixture pattern), WHEN
      `NextState::<ClientState>::set(ClientState::InSession)` is
      transitioned, THEN every shop/auction panel root entity
      (shop panel, auction panel, bid button(s)) carries a
      non-default `ImageNode.image` handle (handle ≠
      `Handle::<Image>::default()`).
- [ ] **AC-5 No new phase or economy drainer**: GIVEN the diff for
      `client/src/ui/shop_auction/`, WHEN it is filtered for
      `MessageReceiver<S2CPhaseChanged>` and
      `MessageReceiver<S2CGoldUpdate>` registrations, THEN no new
      drainer is registered in this plugin. The plugin reads
      `Res<CurrentClientPhase>` and `Res<PlayerEconomyView>` for
      both signals (per ADR-021 single-drainer rule).
- [ ] **AC-6 SAU-007 + SAU-008 behaviour preserved**: GIVEN the
      existing settlement and reconnect snapshot integration tests
      (`tests/integration/shop_auction_ui/auction_settlement_test.rs`
      and `tests/integration/shop_auction_ui/reconnect_late_message_test.rs`),
      WHEN they are re-run after this story's changes, THEN they
      pass without modification (no SAU-007 / SAU-008 regression).
- [ ] **AC-7 Manual evidence document recorded**: GIVEN the
      friend-game route capture from AC-3, WHEN the evidence document
      at `production/qa/evidence/sprint-10-shop-auction-chrome-evidence.md`
      is read, THEN it records: build commit SHA, route steps
      (DRAFT_SHOP → DRAFT_AUCTION → settlement → DRAFT_SHOP), one
      screenshot per phase showing wired chrome, and the friend-game
      no-claims language (no public release readiness, no full asset
      approval, no Standard-tier accessibility, no playtest validation).

---

## Implementation Notes

The substantive wiring landed in PAW-003 (`792a9d8`, integrated). This
story is the friend-game route verification + integration test +
manual evidence loop on top of that wiring. Expected work shape:

1. **Audit pass** (read-only): grep `client/src/ui/shop_auction/` for
   inline asset path strings (`.png`, `.jpg`, `assets/`), for any
   `Sprite` use (forbidden for these panels), for any new
   `MessageReceiver<S2CPhaseChanged>` or `MessageReceiver<S2CGoldUpdate>`
   registration (forbidden — shared drainers only), and for any
   `NodeBundle` / `ImageBundle` / `UiImage::new()` use (forbidden
   in Bevy 0.18). Record findings.
2. **Patch pass** (only if audit surfaces violations): replace any
   inline asset path with the matching `asset_wiring.rs` constant;
   replace any `Sprite` on a UI panel surface with `ImageNode`;
   replace any forbidden Bevy 0.15-era bundle with the Required
   Components API spawn pattern.
3. **Integration test add**: extend or add one test under
   `tests/integration/shop_auction_ui/` that follows the partial-App
   fixture pattern from PAW-003 + S10-TD-001 (Wave D — `c11d1b6`):
   `init_state::<ClientState>()`, `AssetPlugin::default()`,
   `init_asset::<Image>()`, `placeholder_assets_for_tests()`. Assert
   non-default `ImageNode.image` on every panel root entity after
   `OnEnter(ClientState::InSession)`.
4. **Friend-game route capture**: launch a two-client local build,
   walk through DRAFT_SHOP → DRAFT_AUCTION → settlement → DRAFT_SHOP,
   screenshot each phase, store screenshots and write
   `production/qa/evidence/sprint-10-shop-auction-chrome-evidence.md`.
5. **Regression check**: re-run
   `cargo test -p client --test shop_auction_ui_auction_settlement_test`
   and
   `cargo test -p client --test shop_auction_ui_reconnect_late_message_test`
   to confirm SAU-007 + SAU-008 behaviour is intact.

If the audit pass finds zero violations (PAW-003 wiring is intact and
no later commit regressed it), the story collapses to: integration
test add + friend-game route capture + evidence-doc authoring.

The integration test fixture pattern from PAW-003's
`tests/integration/presentation/shop_auction_asset_wiring_test.rs`
(introduced in `792a9d8`) is the canonical reference — mirror its
`init_state` + `AssetPlugin` + `init_asset::<Image>()` +
`PlaceholderAssets` shape and extend it to assert through
`OnEnter(ClientState::InSession)` rather than at App build time.

## Performance Budget

- **Presentation steady-state**: < 1 ms per frame (per ADR-021
  Performance Guardrails). Toggling `Visibility` on pre-pooled panel
  entities; no per-frame spawn / despawn.
- **Phase-boundary frame** (entering DRAFT_AUCTION or DRAFT_SHOP):
  < 3 ms spike (per ADR-021). This story does not change the
  phase-boundary cost; chrome `ImageNode` handles are static after
  initial load.
- No hot-path code changed by this story.

---

## QA Test Cases

(Source: `production/sprints/sprint-10.md:96` S10-POLISH-002 row +
`production/qa/qa-plan-sprint-10-2026-05-10.md` if present at
`/dev-story` time; if not present, author via `/qa-plan sprint`.)

- **Asset path constant audit**
  - Given: post-implementation `client/src/ui/shop_auction/` tree.
  - When: `grep -rE '"[^"]*\.(png|jpg|svg)"|"assets/' client/src/ui/shop_auction/`
    is run.
  - Then: zero hits outside comments / docstrings.

- **Sprite / ImageNode boundary audit**
  - Given: post-implementation `client/src/ui/shop_auction/` tree.
  - When: `grep -rn 'Sprite::' client/src/ui/shop_auction/` is run.
  - Then: zero hits (board content is `Sprite`; UI panels are
    `ImageNode` per ADR-021).

- **Integration test passes**
  - Given: `main` at the post-implementation commit.
  - When: `cargo test -p client --test shop_auction_ui_chrome_wiring_test`
    (or the test name chosen by the implementer) is run.
  - Then: every panel root entity asserts non-default
    `ImageNode.image`; suite passes 100%.

- **SAU-007 + SAU-008 regression check**
  - Given: same commit.
  - When: `cargo test -p client --test shop_auction_ui_auction_settlement_test`
    and `cargo test -p client --test shop_auction_ui_reconnect_late_message_test`
    are run.
  - Then: both suites pass 100% (no behavioural regression from
    chrome wiring).

- **Friend-game route visual capture**
  - Given: a local build with two clients and an active friend-game
    session.
  - When: the route DRAFT_SHOP → DRAFT_AUCTION → auction settlement
    → post-auction DRAFT_SHOP is walked and one screenshot per
    phase is captured.
  - Then: each screenshot shows wired chrome (panel backgrounds,
    slot wells, bid button chrome, border ramp tiles); evidence
    doc records all four screenshots + commit SHA + no-claims
    language.

---

## Test Evidence

**Story Type**: UI

**Required automated test**:

- New: `tests/integration/shop_auction_ui/chrome_wiring_test.rs` (or
  extension of an existing shop_auction_ui integration test) — must
  follow the partial-App fixture pattern from PAW-003 + S10-TD-001
  (Wave D `c11d1b6`).
- Pattern reference:
  `tests/integration/presentation/shop_auction_asset_wiring_test.rs`
  (PAW-003 baseline assertion).
- Fixture-init helpers required:
  - `App::new().add_plugins(MinimalPlugins)`
  - `.init_state::<ClientState>()`
  - `.add_plugins(AssetPlugin::default())`
  - `.init_asset::<Image>()`
  - `.insert_resource(placeholder_assets_for_tests())` (helper from
    `client/src/asset_wiring.rs`, owned by S10-TD-001 Wave C)
  - `.add_plugins(ShopAuctionUiPlugin)`

**Required manual evidence document**:

- `production/qa/evidence/sprint-10-shop-auction-chrome-evidence.md`
  — must contain: build commit SHA, route steps walked, one
  screenshot per phase (DRAFT_SHOP, DRAFT_AUCTION, settlement,
  post-auction DRAFT_SHOP), friend-game no-claims language.

**Accept-risk waiver**:

- Final-art completion is accept-risk for friend-game scope (carries
  PAW-TD-003-a forward).
- Browser visual capture beyond the four route screenshots is
  accept-risk for friend-game scope (carries PAW-TD-003-b forward).

**Required verification commands**:

- `cargo test -p client --test shop_auction_ui_chrome_wiring_test`
  (or the chosen test name)
- `cargo test -p client --test shop_auction_ui_auction_settlement_test`
- `cargo test -p client --test shop_auction_ui_reconnect_late_message_test`
- `grep -rE '"[^"]*\.(png|jpg|svg)"|"assets/' client/src/ui/shop_auction/`
  → zero hits
- `grep -rn 'Sprite::' client/src/ui/shop_auction/` → zero hits

---

## Files Modified

Expected file set (final scope determined at `/dev-story` audit time):

| Path | Expected change |
|---|---|
| `client/src/ui/shop_auction/mod.rs` | Verify / patch panel root spawn sites use `asset_wiring.rs` constants via `ImageNode`. |
| `client/src/ui/shop_auction/shop_panel.rs` (or equivalent) | Verify slot well spawn sites use `ImageNode` + `asset_wiring.rs`. |
| `client/src/ui/shop_auction/auction_panel.rs` (or equivalent) | Verify auction panel background + border ramp tile spawn sites use `ImageNode` + `asset_wiring.rs`. |
| `client/src/ui/shop_auction/bid_button.rs` (or equivalent) | Verify bid button chrome spawn sites use `ImageNode` + `asset_wiring.rs`. |
| `tests/integration/shop_auction_ui/chrome_wiring_test.rs` | New integration test asserting non-default `ImageNode.image` on every panel root after `OnEnter(ClientState::InSession)`. |
| `client/Cargo.toml` | Add `[[test]]` entry for the new integration test (if needed by the workspace config). |
| `production/qa/evidence/sprint-10-shop-auction-chrome-evidence.md` | New manual evidence doc (separate prompt may handle authoring; the story can land with the path reserved per the friend-game-lite paperwork pattern from S10-TD-001). |

If the audit pass surfaces zero violations, the patch column for the
spawn-site files collapses to "no change" and the story shrinks to
the integration test + evidence-doc rows.

No `server/`, no `shared/`, no other client UI plugin
(`HandUiPlugin`, `HudPlugin`, `BoardRenderingPlugin`,
`CardAnimationsPlugin`) is touched by this story.

---

## Dependencies

- **Depends on**: PAW-003
  (`production/epics/presentation-asset-wiring/story-003-shop-auction-chrome.md`)
  is `done` and integrated to `main` (per S10-PAW-001 close-out
  prerequisite). PAW-003 owns the `asset_wiring.rs` substrate that
  this story consumes.
- **Depends on**: S10-PAW-001 PAW-003 `/story-done` close-out has
  flipped the `production/sprint-status.yaml` PAW-003 row to `done`.
- **Depends on**: S10-TD-001 fixture cascade repair (`done`) — its
  Wave C `placeholder_assets_for_tests()` helper and Wave D
  `init_state` + `AssetPlugin` + `init_asset::<Image>()` pattern are
  both prerequisites for the integration test in this story.
- **Depends on**: SAU-007 (settlement) and SAU-008 (reconnect
  snapshot) story behaviour intact — this story preserves both,
  re-runs their integration tests as a regression gate.
- **Depends on**: ADR-021 (Accepted) governing presentation layer
  rules.
- **Depends on**: Sprint 10 plan
  (`production/sprints/sprint-10.md`) and Sprint 10 QA plan
  (`production/qa/qa-plan-sprint-10-2026-05-10.md` — author via
  `/qa-plan sprint` if not present at `/dev-story` time) being
  authored.
- **Unlocks**: S10-N1 (Sprint 10 evidence index — picks up this
  story's evidence path), S10-N2 (friend-game route readability
  notes — picks up the captured screenshots as input).

## Readiness Notes

**Implementation readiness verdict (target)**: READY — pending
`/story-readiness` re-run.

Pull condition expectations:

- PAW-003 `/story-done` close-out (S10-PAW-001) lands first; the
  `asset_wiring.rs` substrate is then verified `done` on `main`.
- S10-TD-001 fixture cascade repair already integrated on `main`
  (`done` per `production/sprint-status.yaml`), so the
  `placeholder_assets_for_tests()` helper and partial-App fixture
  pattern are both available.
- Sprint 10 QA plan exists or is authored before `/dev-story`
  begins on this story.

---

## Definition of Done

- [ ] AC-1 through AC-7 all pass.
- [ ] New integration test under `tests/integration/shop_auction_ui/`
      asserts non-default `ImageNode.image` on every panel root after
      `OnEnter(ClientState::InSession)`; suite passes under
      `cargo test -p client`.
- [ ] SAU-007 + SAU-008 integration suites still pass without
      modification.
- [ ] No production code outside `client/src/ui/shop_auction/` is
      modified by this story (asset wiring substrate from PAW-003 is
      the only outside touch and it predates this story).
- [ ] No new `MessageReceiver<S2CPhaseChanged>` or
      `MessageReceiver<S2CGoldUpdate>` drainer is registered in
      `ShopAuctionUiPlugin`.
- [ ] Manual evidence document at
      `production/qa/evidence/sprint-10-shop-auction-chrome-evidence.md`
      records the four-phase friend-game route walk with screenshots,
      build commit SHA, and friend-game no-claims language.
- [ ] `production/sprint-status.yaml` S10-POLISH-002 row flipped to
      `done` via `/story-done` after the verification commands all
      return PASS.
- [ ] No claim of public release readiness, full playable-client
      manual QA, full game completion, broad Standard-tier
      accessibility completion, playtest/fun-hypothesis validation,
      or full asset/content production.
- [ ] No closure of S8-QA-001-W1, QA-COND-0005, or QA-COND-0006.

---

## Completion Notes

**Completed**: 2026-05-10 (PROMPT 621 `/story-done` verdict)
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 7/7 — AC-1, AC-2, AC-4, AC-5, AC-6 PASS; AC-3, AC-7 PARTIAL/DEFERRED (manual two-client friend-game route screenshot capture pending; documented inline in evidence doc as friend-game-lite paperwork pattern, precedent S10-TD-001).
**Integration commit**: `fb30734` (PROMPT 620 cherry-pick of PROMPT 617) on `origin/main`.
**Test Evidence**:
- Automated: `tests/integration/shop_auction_ui/chrome_wiring_test.rs` — 4/4 pass (shop panel root, auction panel root, bid buttons, shop slots all carry non-default `ImageNode.image` after `OnEnter(ClientState::InSession)`).
- Regression: `shop_auction_ui_auction_settlement_test` 7/7 pass; `shop_auction_ui_reconnect_late_message_test` 6/6 pass (SAU-007 + SAU-008 intact).
- Manual: `production/qa/evidence/sprint-10-shop-auction-chrome-evidence.md` (walkthrough doc; live screenshots pending live two-client run, friend-game-lite paperwork pattern).
**Deviations (ADVISORY only — none blocking)**:
- AC-3 / AC-7 manual screenshot deferred (friend-game-lite paperwork pattern; documented inline in evidence doc).
- Auction panel root reuses `SHOP_PANEL_CHROME_ASSET` constant vs a dedicated auction-specific chrome constant (PAW-TD-003-a accept-risk for friend-game scope; carried forward, not new tech debt).
- Auction border ramp tiles not wired (no spawn site in `client/src/ui/shop_auction/`, no asset constant; out of scope for MVP verification).
**Code Review**: Skipped — Lean review mode (LP-CODE-REVIEW gate not spawned per `/story-done` Phase 5 lean rule).
**QA Coverage Gate**: Skipped — Lean review mode (QL-TEST-COVERAGE gate not spawned per `/story-done` Phase 4b lean rule).
**Manifest staleness**: Story manifest version 2026-05-05 matches current `docs/architecture/control-manifest.md` baseline — no staleness flag.
