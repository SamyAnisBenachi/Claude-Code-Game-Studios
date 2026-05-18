# Story 019: S17-UI-BID-BUTTON-PHASE-RACE-001 -- Bid-Button Phase-Entry Race / Empty + Baked-`?` Cleanup

> **Epic**: Shop / Auction UI
> **Story ID**: S17-UI-BID-BUTTON-PHASE-RACE-001
> **Status**: Draft -- Sprint 17 Should Have candidate (SOURCE-1077-10); NOT activated by this authoring run
> **Layer**: Shop / Auction UI -- bid-button spawn / chrome / text race repair
> **Type**: Tech Debt -- UI race cleanup (single-surface, single-module)
> **Sprint**: Sprint 17 Should Have row per `production/sprints/sprint-17.md` §"Should Have". Activation is a separate explicit prompt (PROMPT 1093 pattern).
> **Authored**: 2026-05-18 by PROMPT 1095
> **Authoring source-of-truth**: `origin/main@7d36191fe94adf99d3448a58185d8079d828c29e`
> **Estimated effort**: ~0.25d (single-surface bid-button race repair)
> **Source audit**: PROMPT 1077 `reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md` §"Per-finding evidence" SOURCE-1077-10 (P2)

---

## Target Epic Justification

This story is filed under `production/epics/shop-auction-ui/`
rather than `production/epics/ui-clean-pass/`. Justification:

- The repair surface is bounded to
  `client/src/ui/shop_auction/auction_bid_buttons*.rs` (or the
  post-modsplit equivalent under `client/src/ui/shop_auction/`)
  plus optionally `assets/ui/ui_bid_button_disabled.png` if the
  baked `?` is replaced (the audit recommends keeping the asset
  and source-side mitigating via chrome-state override; the worker
  re-confirms at activation HEAD).
- The Shop / Auction UI epic owns every bid-button file. Existing
  sibling stories include `story-005-auction-bid-buttons-affordability-and-inflight.md`
  and `story-011-auction-bid-target-size-and-focus-evidence.md` —
  both single-surface bid-button rows. This row is the natural
  next bid-button story in the same epic.
- The UI Clean-Pass epic is reserved for cross-cutting / multi-
  surface UI structural refactors (modsplits, primitive
  ratifications, marker-split-across-modules). This row touches
  one module's bid-button entities only.
- Sprint 17 plan row source allows either epic ("Target epic:
  production/epics/shop-auction-ui/ or
  production/epics/ui-clean-pass/ based on existing ownership.
  Choose one and justify."). The existing shop-auction-ui story
  set demonstrates the file ownership pattern for bid-button work;
  choosing shop-auction-ui keeps single-surface bid-button rows
  co-located.

---

## Status / No-Claim Banner

This story is a Sprint 17 Should Have **candidate** authored by
PROMPT 1095. **No sprint is activated by this authoring run.**
PROMPT 1095 does NOT modify `production/sprint-status.yaml`,
`production/sprints/sprint-17.md`, `production/sprints/sprint-16.md`,
`production/stage.txt`, any `production/session-state/*` file, any
QA-plan / smoke / Team-QA / gate-check / release-check artifact
under `production/qa/`, any code under `client/`, `server/`,
`shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`,
`.github/`, or `Trunk.toml`. PROMPT 1095 does NOT run
`/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
`/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `cargo`,
`trunk`, or any CI command.

This story does **not** claim: public release readiness, release-
candidate readiness, full game completion, broad / Standard-tier
accessibility completion (`QA-COND-0005`), Standard-tier hit-target
conformance (>=44 px), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client
GAME_OVER closure (`S8-QA-001-W1`), final-art / asset-production
completion (`PAW-TD-*-a` — including the baked-`?` placeholder PNG
that the audit names; **the existing PNG is preserved or replaced
with a placeholder-class asset, not a real-art asset**),
`Polish->Release` gate-check retry, stage advance, closure of the
Sprint 12 story 019 underlying drag-runtime bug, closure of
`S11-HUD-TIMER-EYEBALL-VISUAL-001`, closure of any of the 24
PROMPT 1022 audit findings, closure of any SOURCE-1077-* finding
outside SOURCE-1077-10, or closure of any AUDIT-1076-* finding.

**No optimistic client-side authority is introduced or proposed.**
No protocol shape change. No new server-authoritative state. No
new C2S / S2C message. The auction system continues to send
`S2CAuctionCard` / `S2CAuctionBidAccepted` etc. unchanged; this row
is a client-side bid-button spawn-state cleanup.

Sprint 16 disposition `closed-with-conditions` preserved unchanged.
Sprint 15 / 14 / 13 / 12 / 11 / 10 dispositions preserved
unchanged. PROMPT 761 Polish->Release gate-check `FAIL` preserved.
`PAW-TD-*-a`, `QA-COND-0005`, `QA-COND-0006`, `TQ-S12-C1..C7`
preserved verbatim.

---

## Source Finding

### SOURCE-1077-10 (P2) — Bid-button text-spawn / chrome-asset race

- **Audit location**:
  `reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md`
  §"Per-finding evidence" SOURCE-1077-10 (P2).
- **Affected file lines at audit time**:
  - Spawn: `client/src/ui/shop_auction/mod.rs:4645-4678`.
    `Text::new("")` + `ImageNode::new(asset_server.load(bid_button_asset(
    BidButtonChromeState::Disabled)))`.
  - Chrome state mapping: `:5530-5535`
    (`fn auction_bid_chrome_state`). Only two states are emitted —
    `Normal` (when `Enabled`) and `Disabled` (all other states).
    The `BID_BUTTON_HOVER_ASSET` constant
    (`asset_wiring.rs:46`) exists but is repointed to
    `PLACEHOLDER_FALLBACK_ASSET` and is **never selected by
    `auction_bid_chrome_state`** — dead chrome state.
  - Text population: only inside
    `handle_auction_card_received_system` /
    `handle_auction_bid_accepted_system` (e.g. line 3697), which
    fire after `S2CAuctionCard` arrives.
- **Symptom**: bid-button text spawn-state is `Text::new("")`,
  populated only after `S2CAuctionCard` arrives. The
  `ui_bid_button_disabled.png` chrome asset has `"?"` baked in
  (per `PAW-TD-*-a` accept-risk). Phase-entry race produces
  visible empty / `"?"` buttons during the time between phase
  entry (`DraftAuction`) and `S2CAuctionCard` arrival, AND during
  the entire local-leading window (`HiddenLeading` state).
- **Audit recommended minimal repair surface (source-side)**:
  override the chrome image with a transparent / no-image asset
  when `AuctionBidButtonState::HiddenLeading` so the bid row
  hides entirely during local-leading. PROMPT 1042 already added
  a Pass affordance but the disabled-chrome `?` still surfaces
  during phase entry and other disabled states.

- **Note from PROMPT 1077 audit**: "Out of scope for source
  repair — this is `PAW-TD-*-a` placeholder-art accept-risk."
  The Sprint 17 plan however explicitly schedules a source-side
  mitigation row (`S17-UI-BID-BUTTON-PHASE-RACE-001`) and the
  Sprint 17 plan AC column reads: "bid-button text is `Loading…`
  (or equivalent) before `S2CAuctionCard` arrives, then numeric
  bid amounts after." This row implements the **source-side
  mitigation only**; the placeholder PNG is NOT replaced with
  real art.

---

## Problem Class / Prevention Target

**Defect class**: a bid button entity that spawns with an empty
text label and a baked-`?` chrome image, then is mutated when a
later S2C message arrives. The visible "?" reads as "we don't
know what to bid" to the player even though it is intended to
mean "disabled / not available".

**Prevention target**:

1. **Spawn-state text is meaningful, not empty.** At spawn, the
   bid-button text is `Loading…` (or an equivalent localizable
   pending-state label — concrete string TBD by implementing
   worker; consistent with the project's existing pending-state
   conventions).
2. **Chrome state override during `HiddenLeading` (per audit
   minimal repair)**: when `AuctionBidButtonState::HiddenLeading`,
   the bid row hides entirely (Visibility flipped to `Hidden` OR
   the chrome image set to a transparent / no-image asset).
3. **Either way: the visible `?` glyph stops surfacing during
   phase-entry race and local-leading windows.**

The placeholder PNG itself (`ui_bid_button_disabled.png` with
baked `?`) is preserved verbatim per `PAW-TD-*-a` accept-risk.
This row is a **chrome state / visibility cleanup**, not a real-
art row.

---

## Context

### Existing surface

- **`client/src/ui/shop_auction/auction_bid_buttons*.rs`** or
  **`client/src/ui/shop_auction/mod.rs:4645-4678`** (audit-time
  spawn site) — bid-button entities. The implementing worker
  re-verifies the post-modsplit layout at activation HEAD; if
  Sprint 17 Should Have row
  `S16-TD-UI-SHOPAUCTION-MODSPLIT-001` (story 010 of the UI
  Clean-Pass epic) has landed by then, the bid-button entities
  may live in a dedicated `auction_bid_buttons.rs` submodule.
- **`client/src/ui/shop_auction/mod.rs:5530-5535`** (audit-time
  `auction_bid_chrome_state`) — chrome state mapper. Possibly
  shifted by modsplit.
- **`client/src/asset_wiring.rs:46`** —
  `BID_BUTTON_HOVER_ASSET` constant; currently repointed to
  `PLACEHOLDER_FALLBACK_ASSET`. Implementing worker re-verifies
  whether the `HOVER` asset can be repurposed for the
  `HiddenLeading` override, OR whether a new transparent /
  no-image fallback asset is needed (e.g.
  `BID_BUTTON_HIDDEN_LEADING_ASSET` pointing at a 1x1 transparent
  PNG or an explicit `Handle<Image>::default()`).
- **`assets/ui/ui_bid_button_disabled.png`** — PNG chrome image
  with baked `?` glyph. **Preserved verbatim** per
  `PAW-TD-*-a` accept-risk. The implementing worker MUST NOT
  edit or replace this file.
- **`assets/ui/`** — directory where any new transparent PNG
  fallback asset would live IF the implementing worker chooses
  to author one. Authoring such an asset is in scope (it is a
  placeholder, not real art); the file count change is
  documented in the commit message.
- **Existing test bins**:
  `tests/integration/shop_auction_ui/auction_bid_buttons_test.rs`
  and `auction_activation_test.rs`. Per PROMPT 1077 §5 test gap
  matrix: "Tests construct a fully-populated auction state
  synchronously; the phase-entry race is bypassed." This row's
  new test asserts the race-state behaviour.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/shop-auction-ui.md` may already specify a
  bid-button "Loading…" state. Implementing worker re-verifies;
  if specified, the new text matches the spec; if unspecified,
  the worker MAY append a one-line note.
- **ADR-013** (Auction System State): no change. Server-side
  auction state machine unchanged.
- **ADR-015** (Card Acquisition Shop State): no change.
- **ADR-019** (Economy Resource Architecture): no change.
- **ADR-021** (Presentation Layer Architecture): no schedule
  change. The bid-button systems remain in their existing
  `PresentationSet` slots.
- **ADR-002** (Client-Server Authority): no change. The bid-
  button reads server-authoritative auction state only; no new
  C2S message is added.
- **ADR-008** (Lightyear Channel Configuration): no change. No
  new message; no new channel.
- **TR-SAU-002** (bid increment buttons render total commitment
  from current price plus preset offsets) preserved. This row
  only affects the spawn-state and `HiddenLeading`-state visual,
  not the numeric formula.
- **TR-SAU-005** (in-flight bid state) preserved unchanged.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` on every `.rs` edit. No
  Lightyear edits — `liv-bevy-lightyear` NOT required.

### Control Manifest Rules

- Required: bid-button spawn-state text is non-empty and
  meaningful (e.g. `Loading…`). Concrete string TBD by
  implementing worker.
- Required: when `AuctionBidButtonState::HiddenLeading`, either
  (a) the bid row Visibility is `Hidden`, OR (b) the chrome
  ImageNode is set to a transparent / no-image asset and the
  text is non-rendering (empty, or a single space, or
  `Visibility::Hidden` on the text child). Implementing worker
  chooses and justifies.
- Required: existing PROMPT 1042 Pass affordance preserved.
- Required: `Normal` and `Disabled` chrome state mappings
  preserved (this row does not retune the state mapper outside
  the `HiddenLeading` override and the spawn-state text).
- Required: `ui_bid_button_disabled.png` (baked `?` glyph)
  preserved verbatim. `PAW-TD-*-a` accept-risk preserved.
- Required: if a new transparent / no-image fallback asset is
  authored, it lives under `assets/ui/` with a documented name
  (e.g. `ui_bid_button_hidden_leading.png` — 1x1 transparent PNG
  or equivalent). The file is a placeholder, not real art.
- Required: `QA-COND-0005` preserved (no Standard-tier hit-target
  or contrast claim on the bid button by this row; Sprint 8
  story 011 `A11Y-ST-12` is the existing accessibility row for
  bid-button focus / target size and is NOT advanced by this
  row).
- Required: `QA-COND-0006` preserved (no playtest validation
  claim).
- Forbidden: replacing `ui_bid_button_disabled.png` with a real-
  art asset. Replacement with a non-`?` placeholder PNG is also
  out of scope unless the implementing worker is explicitly
  scoping the PAW-TD-* row (which is NOT in scope here).
- Forbidden: changing `shared/`, `server/`, or anything under
  `tests/integration/server/` / `tests/unit/server/`.
- Forbidden: changing `auction_bid_chrome_state` mapping for
  `Normal` or `Disabled` (only the `HiddenLeading` override is
  in scope).
- Forbidden: closing the existing PROMPT 1042 Pass affordance
  story or amending its acceptance criteria.
- Forbidden: closing `A11Y-ST-12` (Sprint 8 story 011 / Story
  005 `auction-bid-buttons-affordability-and-inflight` /
  Story 011 `auction-bid-target-size-and-focus-evidence`).
- Forbidden: closure of any SOURCE-1077-* finding outside
  SOURCE-1077-10.
- Forbidden: closure of any AUDIT-1076-* finding.

---

## Story Classification

**Story type**: **UI / Integration** (single-surface UI behaviour
fix + integration coverage for the race-state assertion).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story
Type" matrix:

- UI rows require manual walkthrough OR interaction test
  (ADVISORY gate).
- Integration rows require integration test OR documented
  playtest (BLOCKING gate).

This row delivers an **integration test** that asserts the spawn-
state text + the `HiddenLeading` chrome / visibility behaviour
without requiring a manual walkthrough. The integration test
asserts both the pre-`S2CAuctionCard` state and the
`HiddenLeading` state.

This is **NOT** a:

- Logic-only story (touches UI surface).
- Visual / feel story (no animation / shader / VFX).
- Final-art story (`PAW-TD-*-a` preserved).
- Accessibility story (`QA-COND-0005` preserved; `A11Y-ST-12`
  NOT advanced).
- Layout / chrome MVP story (separate from PROMPT 1042 Pass
  affordance and from story 014 panel-chrome-MVP).

---

## Dependencies and Parallelism

### Prerequisites

- None on `origin/main` beyond Sprint 16 / earlier baseline. This
  row stands alone on the bid-button entity.

### Parallelism summary

| Sibling story | Parallel-safe? | Notes |
|---|---|---|
| `S17-UI-CARD-DISPLAY-ART-HELPER-001` (Must) | **PARTIAL** | both edit `client/src/ui/shop_auction/`. Serialise — the helper bundle deletes helper bodies and adds `use`; this row edits the bid-button spawn and chrome state mapper. File overlap on `mod.rs`. |
| `S17-UI-CARD-SLOT-INSET-WIRING-001` (Should) | **YES** | disjoint (`design_tokens/card_slot.rs`). |
| `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` (Should) | **PARTIAL** | both edit `client/src/ui/shop_auction/` (marker split). Serialise. |
| `S17-UI-HUD-OPP-MANA-CLEANUP-001` (Should) | **YES** | disjoint (HUD). |
| `S17-UI-MODAL-BLACK-SLAB-001` (conditional Must) | **NO** | both edit `client/src/ui/shop_auction/mod.rs`. Serialise. |
| `S17-UI-SHOP-AUCTION-SURFACE-PAINT-001` (conditional Must) | **NO** | both edit `client/src/ui/shop_auction/`. Serialise. |
| `S17-UI-HAND-B0004-CLEANUP-001` (Nice) | **YES** | disjoint (hand UI). |
| `S17-OPS-VULKAN-VALIDATION-GATING-001` (Nice) | **YES** | disjoint. |
| `S17-SERVER-START-OF-TURN-DEBUG-001` (Nice) | **YES** | disjoint. |

The Sprint 17 producer SHOULD schedule this row **after** the
conditional Must Have shop / auction surface paint
(`S17-UI-SHOP-AUCTION-SURFACE-PAINT-001`) lands on `origin/main`,
so the bid-button spawn site is in its post-repair shape.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Bid-button spawn-state text is non-empty and
  meaningful**: GIVEN the post-implementation client running
  through `Lobby -> Handshaking -> InSession -> DraftAuction`,
  WHEN the bid-button entities are spawned at phase entry, WHEN
  inspected before `S2CAuctionCard` arrives, THEN their `Text`
  component contains a non-empty meaningful pending-state string
  (e.g. `"Loading…"`). The exact string chosen by the
  implementing worker is documented in the commit message AND
  used consistently across all three bid buttons (low / mid /
  high or equivalent offsets).

- [ ] **AC2 -- Bid-button text updates to numeric bid amounts on
  `S2CAuctionCard` arrival**: GIVEN the same flow, WHEN
  `S2CAuctionCard` arrives, THEN the bid-button text is updated
  to the numeric bid amounts per the existing TR-SAU-002 formula
  ("current price plus preset offsets"). Integration test asserts
  the sequence: spawn-state pending text -> `S2CAuctionCard`
  drain -> numeric text.

- [ ] **AC3 -- `HiddenLeading` chrome / visibility override
  (SOURCE-1077-10 audit minimal repair)**: GIVEN
  `AuctionBidButtonState::HiddenLeading` is the current bid-
  button state for the local player, WHEN inspected via ECS
  query, THEN either (a) the bid row Visibility is `Hidden`, OR
  (b) the chrome ImageNode is set to a transparent / no-image
  asset (e.g. `Handle<Image>::default()` or a new
  `BID_BUTTON_HIDDEN_LEADING_ASSET` placeholder) AND the text is
  non-rendering. Concrete strategy TBD by implementing worker;
  justified in the commit message.

- [ ] **AC4 -- Visible `?` glyph does not surface during
  phase-entry race**: GIVEN the integration test fixture that
  spawns the bid buttons in `DraftAuction` phase BEFORE draining
  `S2CAuctionCard`, WHEN any captured snapshot or ECS query is
  inspected, THEN the bid buttons either (a) carry the
  Loading… text and a non-`?` chrome image, OR (b) are
  `Visibility::Hidden`. The baked-`?` PNG
  (`ui_bid_button_disabled.png`) MAY still be loaded as the
  `Disabled` chrome handle, BUT the `Disabled` chrome MUST NOT
  be applied to the bid-button entity in the phase-entry race
  window (i.e. the chrome state should not be `Disabled` at
  spawn unless the bid is intentionally disabled and the auction
  card is known).

- [ ] **AC5 -- Visible `?` glyph does not surface during
  `HiddenLeading`**: GIVEN the integration test fixture that
  drives the bid-button state machine into `HiddenLeading`, WHEN
  inspected, THEN per AC3 the bid row is either hidden or the
  chrome is transparent. The `?` glyph is not visible.

- [ ] **AC6 -- Existing PROMPT 1042 Pass affordance preserved**:
  GIVEN the post-refactor build, WHEN the Pass affordance is
  exercised by the existing `auction_bid_buttons_test.rs`
  fixtures (PROMPT 1042 closure assertions), THEN every existing
  PASS continues to PASS. This row is additive over PROMPT 1042;
  it does not regress the Pass affordance.

- [ ] **AC7 -- `auction_bid_chrome_state` mapper preserved for
  `Normal` and `Disabled`**: GIVEN
  `client/src/ui/shop_auction/.../auction_bid_chrome_state`
  (post-refactor), WHEN inspected, THEN the `Normal` (Enabled)
  and `Disabled` (default) mappings are unchanged. The
  `HiddenLeading` branch is the only new code path.

- [ ] **AC8 -- `ui_bid_button_disabled.png` not modified**:
  GIVEN `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN
  `assets/ui/ui_bid_button_disabled.png` is unchanged.
  `PAW-TD-*-a` accept-risk preserved.

- [ ] **AC9 -- Integration test bin authored**: GIVEN a new test
  or extended assertions in
  `tests/integration/shop_auction_ui/auction_bid_buttons_test.rs`
  (or a NEW
  `tests/integration/shop_auction_ui/auction_bid_buttons_phase_race_test.rs`),
  WHEN run, THEN it asserts AC1, AC2, AC3, AC4, AC5, AC7 against
  a real Bevy 0.18 `App` per the existing
  `tests/integration/shop_auction_ui/` pattern.

- [ ] **AC10 -- No protocol or server change**: GIVEN
  `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN there
  are zero changes under `server/`, `shared/`, or
  `tests/integration/server/`. The implementation is client-side
  only.

- [ ] **AC11 -- ADR-021 schedule preserved**: GIVEN `cargo build
  -p client` under the Cargo resource policy, WHEN run, THEN no
  new system-set or schedule wiring is introduced. The bid-
  button systems remain in their existing `PresentationSet`
  slots.

- [ ] **AC12 -- No accept-risk closure claimed**: GIVEN the
  commit message and any evidence document, WHEN inspected, THEN
  they explicitly do NOT claim closure of `S8-QA-001-W1`,
  `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, or any other
  accept-risk disposition. `A11Y-ST-12` (Sprint 8 / Sprint 11
  bid-button focus + target size accessibility) is NOT advanced.
  Final-art replacement of the baked-`?` PNG is explicitly out
  of scope. Standard-tier hit-target conformance is NOT
  pursued. Playtest validation is NOT pursued.

- [ ] **AC13 -- Sprint 17 disposition preserved**: GIVEN the
  implementation commit(s), WHEN
  `production/sprint-status.yaml`, `production/sprints/sprint-17.md`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/*`, `production/gate-checks/*`, and
  `docs/architecture/adr-*.md` are diffed, THEN none are modified
  by this story's `/dev-story` worker.

- [ ] **AC14 -- Worker branch scope contained**: GIVEN the worker
  branch (slug recommendation:
  `work/s17-bid-button-phase-race`), WHEN inspected, THEN it
  pushes only the worker branch — never `main`. Files changed
  at worker time are scoped to the bid-button submodule under
  `client/src/ui/shop_auction/`, optionally `assets/ui/` (new
  placeholder fallback PNG IF chosen — placeholder only, not
  real art), optionally `client/src/asset_wiring.rs` (new
  fallback constant IF chosen), and the new / extended test
  bin under `tests/integration/shop_auction_ui/`.

- [ ] **AC15 -- Cargo resource policy applied for every Cargo
  command**: future implementation MUST set the Cargo resource
  policy env vars (`CARGO_TARGET_DIR=
  D:\_DEV\cargo-target\ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`,
  `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`,
  `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`) before
  every `cargo check` / `cargo test` invocation on Windows /
  MSVC. Story authoring (PROMPT 1095) does NOT invoke Cargo.

---

## Implementation Notes

### Owned files (likely change set)

| Path | Expected change |
|------|-----------------|
| `client/src/ui/shop_auction/.../auction_bid_buttons*.rs` OR `client/src/ui/shop_auction/mod.rs` (audit-time location ~`:4645-4678` spawn site) | Change spawn-state text from `Text::new("")` to `Text::new("Loading…")` (or equivalent). |
| `client/src/ui/shop_auction/.../auction_bid_chrome_state*` (audit-time ~`:5530-5535`) | Add the `HiddenLeading` override branch. |
| `client/src/asset_wiring.rs` (optional) | Add `BID_BUTTON_HIDDEN_LEADING_ASSET` (or equivalent) fallback constant. |
| `assets/ui/ui_bid_button_hidden_leading.png` (optional NEW; placeholder only) | 1x1 transparent PNG or equivalent. Worker MUST document in commit that this is a placeholder, not real art. |
| `tests/integration/shop_auction_ui/auction_bid_buttons_phase_race_test.rs` (NEW; or extended assertions in `auction_bid_buttons_test.rs`) | AC9 integration coverage. |
| `production/qa/evidence/sprint-17-bid-button-phase-race/evidence.md` (NEW, by `/dev-story` worker) | Evidence document; NOT authored by PROMPT 1095. |

### Forbidden files

- `assets/ui/ui_bid_button_disabled.png` — preserved verbatim
  (`PAW-TD-*-a`).
- Everything under `server/`, `shared/`.
- Everything under `tests/integration/server/`,
  `tests/unit/server/`, `tests/integration/lightyear*`,
  `tests/unit/lightyear*`.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/*`.
- All other `production/epics/*` story files (no cross-epic edit).
- `docs/architecture/adr-*.md` (no ADR amendment in scope).
- `.claude/`, `AGENTS.md`, `CLAUDE.md`, `CODEX.md`.

### Cargo resource policy

Per the binding Sprint 15+ QA plan precedent, every `cargo`
invocation on Windows / MSVC MUST set the five env vars under
AC15.

### Target citations

- Sprint 17 plan row source:
  `production/sprints/sprint-17.md` §"Should Have" row
  `S17-UI-BID-BUTTON-PHASE-RACE-001`.
- Source audit:
  `reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md`
  §"Per-finding evidence" SOURCE-1077-10.
- Existing test bins (potential extension target):
  `tests/integration/shop_auction_ui/auction_bid_buttons_test.rs`,
  `tests/integration/shop_auction_ui/auction_activation_test.rs`.
- Predecessor: PROMPT 1042 Pass affordance closure.

---

## Worker Contract (for future `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout` against Sprint 17 activation HEAD on a
   fresh worktree (suggested slug
   `work/s17-bid-button-phase-race`).
2. Read this story file end-to-end before any code change.
3. Re-verify the audit-time module shape by reading the current
   `client/src/ui/shop_auction/` directory. If
   `S16-TD-UI-SHOPAUCTION-MODSPLIT-001` (Sprint 16 / 17
   candidate) has landed by activation, the bid-button entities
   may live in a dedicated submodule.
4. Activate `liv-bevy-018` skill before any `.rs` edit. Do NOT
   activate `liv-bevy-lightyear`.
5. Pick the `HiddenLeading` override strategy (visibility flip vs
   transparent asset). Justify in the commit message.
6. If a new placeholder PNG is authored, document in the commit
   message that the file is a placeholder and that
   `PAW-TD-*-a` accept-risk is preserved.
7. Set the Cargo resource policy env vars per AC15 before every
   `cargo check` / `cargo test` invocation.
8. Run `cargo check -p client` and the targeted `cargo test -p
   client --test auction_bid_buttons_phase_race_test` (or
   extended `auction_bid_buttons_test`) under the Cargo
   resource policy.
9. Push the worker branch (never `main`).
10. Stop. Closure paperwork is later prompts' scope.

The worker MUST NOT:

- Modify `assets/ui/ui_bid_button_disabled.png`.
- Modify `server/`, `shared/`, or anything under
  `tests/integration/server/` / `tests/unit/server/`.
- Modify `shared/src/protocol.rs` auction message shapes.
- Modify Cargo / Trunk / CI files.
- Modify `production/sprint-status.yaml`, `production/sprints/`,
  `production/stage.txt`, `production/session-state/`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/`.
- Run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, or `/qa-plan` on this story.
- Run the full workspace `cargo test --workspace` invocation
  (targeted bins only).
- Run `trunk` or any CI command.
- Push to `main`.
- Claim closure of `A11Y-ST-12`, any AUDIT-1076-* finding, or
  any SOURCE-1077-* finding outside SOURCE-1077-10.
- Claim release-readiness, accessibility-completion, playtest-
  validation, two-client GAME_OVER closure, final-art
  completion, or stage advance.

### Build gate scope (parallel-agent isolation)

The build gate for this story MUST be scoped to files this
worker owns under `client/src/ui/shop_auction/` plus the new
test bin. The worker MUST NOT block on workspace-wide
compilation errors introduced by other in-flight Sprint 17
workers' branches. Per the Parallelism summary above, this row
serialises behind the conditional Must Have shop / auction
rows (modal + surface paint) and the helper bundle; the
orchestrator schedules accordingly.

### Relay / reporting expectation for future workers

Final status line:

```
N: S17-UI-BID-BUTTON-PHASE-RACE-001: STATUS
```

where `N` is the prompt number that ran `/dev-story`.

---

## Closure Trail

Closure trail is appended by future `/story-readiness`,
`/dev-story`, and `/story-done` prompts. No closure trail is
authored by PROMPT 1095.

### Conditions carried forward unchanged

- Sprint 16 disposition `closed-with-conditions` (UNCHANGED).
- Sprint 17 stage `Polish` (UNCHANGED).
- PROMPT 761 Polish->Release gate-check `FAIL` preserved.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` + `QA-COND-0006` accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved (specifically
  the baked-`?` PNG `ui_bid_button_disabled.png`).
- `TQ-S12-C1..C7` preserved verbatim.
- Sprint 15 / 14 / 13 / 12 / 11 / 10 dispositions preserved
  unchanged.
- HUD timer row `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-
  operator-blocked carry preserved; NOT closed by this row.
- 24 PROMPT 1022 audit findings preserved as report-only; NOT
  closed by this row.
- PROMPT 1042 Pass affordance preserved.

### Explicitly NOT claimed by this story or its `/dev-story` worker

- Closure of `A11Y-ST-12` (Sprint 8 / Sprint 11 bid-button
  focus + target size accessibility). The existing Shop / Auction
  UI epic stories 005 and 011 own that.
- Closure of any AUDIT-1076-* finding.
- Closure of any SOURCE-1077-* finding outside SOURCE-1077-10.
- Closure of any of the 24 PROMPT 1022 audit findings.
- Sprint 17 close-out.
- Real-art replacement of `ui_bid_button_disabled.png`.
- Public release readiness; release-candidate readiness; full
  game completion.
- Broad / Standard-tier accessibility completion; playtest /
  fun-hypothesis validation; full playable-client manual QA;
  two-client GAME_OVER closure; final-art completion;
  Polish->Release gate-check retry; stage advance.

`019: S17-UI-BID-BUTTON-PHASE-RACE-001: DRAFT`
