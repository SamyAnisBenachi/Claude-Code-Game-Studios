# Story 017: S17-UI-CARD-DISPLAY-ART-HELPER-001 -- Card Display Art Helper / Chrome Preservation + Dedup + Leak Fix + Existence Check (PROMPT 1077 P0 Bundle)

> **Epic**: UI Clean-Pass
> **Story ID**: S17-UI-CARD-DISPLAY-ART-HELPER-001
> **Status**: Done -- closed by PROMPT 1117 /story-done paperwork on 2026-05-18 (origin/main@30c9e0f PROMPT 1114 integration tip)
> **Layer**: Presentation / Asset wiring (cross-cut: hand + shop + auction + draft surfaces)
> **Type**: Tech Debt -- structural dedup + correctness bundle (one bundled story per Sprint 17 plan)
> **Sprint**: Sprint 17 Must Have row per `production/sprints/sprint-17.md` §"Must Have (Critical Path)". Activation is a separate explicit prompt (PROMPT 1093 pattern).
> **Authored**: 2026-05-18 by PROMPT 1095
> **Authoring source-of-truth**: `origin/main@7d36191fe94adf99d3448a58185d8079d828c29e`
> (`integrate(s17): merge Sprint 17 plan draft into main (PROMPT 1093 paperwork-only)`)
> **Estimated effort**: ~0.75d (bundled: dedup + chrome preservation + leak fix + existence check; the four findings must land together to avoid re-introducing the empty-slot bug)
> **Source audit**: PROMPT 1077 `reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md` §"Per-finding evidence" SOURCE-1077-01 (P0), SOURCE-1077-02 (P0), SOURCE-1077-03 (P1), SOURCE-1077-04 (P1)

---

## Status / No-Claim Banner

This story is a Sprint 17 Must Have **candidate** authored by PROMPT
1095. **No sprint is activated by this authoring run.** PROMPT 1095
does NOT:

- Activate Sprint 17.
- Modify `production/sprint-status.yaml` (top-level `sprint: 16 / status: closed-with-conditions / stage: Polish` preserved verbatim).
- Modify `production/sprints/sprint-17.md`,
  `production/sprints/sprint-16.md`, or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check
  artifact under `production/qa/`.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Run `cargo`, `trunk`, or any CI command.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, or
  `Trunk.toml`.

This story does **not** claim: public release readiness, release-
candidate readiness, full game completion, broad / Standard-tier
accessibility completion (`QA-COND-0005`), Standard-tier hit-target
conformance (>=44 px), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), final-art / asset-production completion
(`PAW-TD-*-a`), `Polish->Release` gate-check retry, stage advance
from Polish to Release, closure of the Sprint 12 story 019
underlying drag-runtime bug, closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`,
closure of any of the 24 PROMPT 1022 audit findings, closure of any
SOURCE-1077-* finding outside the four absorbed by this bundle
(SOURCE-1077-01 / 02 / 03 / 04), closure of any AUDIT-1076-* finding,
or any new server-authoritative state.

**No optimistic client-side authority is introduced or proposed.** No
protocol shape change. No new server-authoritative resource. This
story is a client-side asset-wiring + UI helper dedup; the server is
unchanged.

Sprint 16 disposition `closed-with-conditions` per PROMPT 1082 +
PROMPT 1088 preserved unchanged. Sprint 15 / 14 / 13 / 12 / 11 / 10
dispositions preserved unchanged. PROMPT 761 Polish->Release gate-
check `FAIL` evidence preserved. `PAW-TD-*-a`, `QA-COND-0005`,
`QA-COND-0006`, `TQ-S12-C1..C7` preserved verbatim.

---

## Source Findings

This story is a **bundle** of four PROMPT 1077 findings. They are
bundled because the dedup (SOURCE-1077-02) MUST land in the same
commit as the slot-well chrome preservation (SOURCE-1077-01) — if
dedup lands first, the chrome fix is replicated to only one helper
copy and the other copy continues to remove the slot's ImageNode on
fallback. The leak fix (SOURCE-1077-03) and existence check
(SOURCE-1077-04) share `client/src/asset_wiring.rs:505-518` with the
helper sites and are cheaper to land together than to schedule as
follow-on prompts.

### SOURCE-1077-01 (P0) — Slot-well chrome lost when card art is missing

- **Audit location**: `reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md`
  §"Per-finding evidence" SOURCE-1077-01 (P0).
- **Affected file lines at audit time**:
  - `client/src/ui/shop_auction/mod.rs:5777-5798` `fn apply_card_display_art`
  - `client/src/ui/hand/mod.rs:4477-4498` `fn apply_card_display_art` (verbatim copy)
  - Slot spawn site at `client/src/ui/shop_auction/mod.rs:4389` inserts
    `ImageNode::new(asset_server.load(SHOP_SLOT_WELL_IDLE_ASSET))`
    as the slot's chrome. The `Err` branch at `:5793-5795` (and hand
    twin at `:4493-4495`) executes
    `entity_commands.remove::<(CardDisplayArtAsset, ImageNode)>()` —
    removing the same `ImageNode` that the slot uses as its chrome.
- **User-visible symptom**: empty rectangle (no slot-well background,
  no card art, no border chrome) on any shop / draft / auction / hand
  surface the moment a card is bound to it whose `art_id` is empty,
  missing, or unresolvable.
- **Surfaces affected**: shop slot, shop footer slot, hand fan, draft
  initial keep-9, auction featured.

### SOURCE-1077-02 (P0) — Duplicate `apply_card_display_art` definitions

- **Audit location**: same report §"Per-finding evidence" SOURCE-1077-02 (P0).
- **Affected file lines at audit time**:
  - `client/src/ui/shop_auction/mod.rs:5777-5803`
    (`apply_card_display_art` + `clear_card_display_art`)
  - `client/src/ui/hand/mod.rs:4477-4503`
    (`apply_card_display_art` + `clear_card_display_art`)
  - Two implementations are verbatim copies (function body, signature,
    component set). Six callers in each module.
- **Likely root cause**: when stories 010 / 011 / PROMPT 1029 added
  the card-art helper, the author copied the implementation into
  `hand/mod.rs` rather than importing it.
- **User-visible symptom (latent)**: any future fix landing in only
  one copy silently drifts the hand and shop_auction surfaces apart.

### SOURCE-1077-03 (P1) — `Box::leak` per render in `resolve_card_display_art`

- **Audit location**: same report §"Per-finding evidence" SOURCE-1077-03 (P1).
- **Affected file lines at audit time**:
  `client/src/asset_wiring.rs:505-518`.
- **Audited body**:

  ```rust
  let path = format!("art/cards/display/card_{art_id}_art_display.png");
  Ok(Box::leak(path.into_boxed_str()))
  ```

- **User-visible symptom (latent)**: per-call `String` leak from
  `apply_card_display_art` / `sync_hand_fan_card_art_system` /
  shop-slot apply / auction-featured apply / etc. Accumulates over a
  long session (N rounds * 3 shop slots * per-phase rebroadcast).

### SOURCE-1077-04 (P1) — `resolve_card_display_art` returns path without existence check

- **Audit location**: same report §"Per-finding evidence" SOURCE-1077-04 (P1).
- **Affected file lines at audit time**:
  `client/src/asset_wiring.rs:505-518` (same function as -03).
- **User-visible symptom**: when `art_id` typo lands, when a card is
  added to the catalog before its art file, or when a test fixture
  synthesises card data with `art_id: format!("test_{id}")` (see
  SOURCE-1077-15 in the same report), the slot renders blank chrome
  silently. Bevy's `AssetServer::load` returns a `Handle<Image>`
  regardless; if the file is missing the loader logs a warning at load
  time but never propagates an error to the consumer.

### Why bundled (per Sprint 17 plan)

Per `production/sprints/sprint-17.md` §"Must Have (Critical Path)" row
`S17-UI-CARD-DISPLAY-ART-HELPER-001` Source column: "Bundled because
dedup MUST land in the same commit as the slot-well chrome fix or the
dedup re-introduces the empty-slot bug; the leak fix and existence
check share `client/src/asset_wiring.rs:505-518` and are cheaper to
land together."

---

## Problem Class / Prevention Target

**Defect class**: a single helper has two verbatim copies (-02), and
both copies remove the slot's chrome ImageNode when card art is
missing (-01), so a single typo in `art_id` or an unloaded asset
deletes the slot-well background. Underneath, the helper plumbing
itself (`resolve_card_display_art` at `asset_wiring.rs:505-518`) leaks
a fresh `&'static str` per render (-03) and does not validate the path
exists on disk (-04) so the failure mode that triggers -01 is silent.

**Prevention target**:

1. Lift the helper to a **single owner** so future fixes cannot drift
   between hand and shop_auction.
2. **Preserve the slot's chrome ImageNode** when card art is missing
   — chrome and card-art are distinct presentation concerns and must
   live on distinct entities (or be served by distinct components).
3. **Stop leaking** on every call by removing the `'static` lifetime
   constraint from the resolver return type.
4. **Probe the asset** at startup (or on session entry) so missing-art
   defects are surfaced as a startup warning rather than as a silent
   blank slot at runtime.

The four together close the loop: missing art is detected up front
(-04), the resolver no longer leaks (-03), and the slot stays
visually intact (-01) even when the underlying card-art handle is
unresolvable. Dedup (-02) ensures a future regression in any of these
three behaviours cannot land in only one helper copy.

---

## Context

### Existing surface

- **`client/src/asset_wiring.rs`** — the natural single-owner site for
  `resolve_card_display_art` + `apply_card_display_art` +
  `clear_card_display_art`. PROMPT 1077 audit recommends this owner
  (alongside the existing `resolve_card_display_art`). An alternative
  is a new `client/src/ui/design_tokens/card_art.rs` module — the
  implementation prompt MUST choose one and justify in the commit
  message.
- **`client/src/ui/shop_auction/mod.rs`** — current copy at
  `:5777-5803`; six callers: `apply_shop_slot` (`:5725`),
  `apply_shop_footer_slot` (`:5774`), auction featured (`:3484`),
  `handle_draft_offering_system` (`:2125`), plus the entity-spawn
  sites at `:4389` (slot well chrome) and the auction `auction_featured_card_node`.
- **`client/src/ui/hand/mod.rs`** — current copy at `:4477-4503`; six
  callers: `sync_hand_fan_card_art_system` (`:1416`), purchase
  confirmation (`:1794`), acquisition / placement paths (`:1895`,
  `:2251`).
- **`shared/src/card.rs`** — `art_id` field on `CardData` /
  `CardDefinition`. **No protocol change** in scope; `art_id`
  semantics unchanged.
- **`assets/art/cards/display/`** — 16 production art files at audit
  time (per PROMPT 1077 §SOURCE-1077-04 evidence). File names follow
  `card_{art_id}_art_display.png` convention.

### GDD / ADR / TR trace

- **GDD**: no GDD update in scope. The chrome / card-art distinction
  is a presentation-layer implementation detail not currently
  specified in `design/gdd/`. Implementation prompt MAY append a one-
  line note to `design/gdd/shop-auction-ui.md` and/or
  `design/gdd/hand-ui.md` documenting the chrome-vs-card-art separation
  if the producer requests it; otherwise GDD is untouched.
- **ADR-021** (Presentation Layer Architecture): no system-set / no
  schedule change. The helper lifts inside the presentation layer; its
  consumers (hand + shop_auction systems) keep their existing schedule
  slots.
- **ADR-002** (Client-Server Authority): no change. The helper reads
  card catalog state only; no S2C drain is added and no C2S message
  is sent.
- **ADR-001** (Objective Identity Unicast): no change. Card art does
  not carry objective identity; `was_fake` invariant unaffected.
- **ADR-004** (Asset Loading Pipeline): existence-check (-04) probes
  the asset registry; consistent with `bevy_asset_loader` typed
  collection patterns. The implementation prompt MAY use `AssetServer::
  load_acquire` or an equivalent check; concrete API choice TBD by the
  implementing worker against the live Bevy 0.18 API at activation
  HEAD.
- **TR registry**: no new TR. This is correctness + dedup for an
  existing rendering helper.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` on every `.rs` edit. No
  Lightyear edits in scope — `liv-bevy-lightyear` is **NOT** required
  unless the implementation prompt expands scope (which it should
  not).

### Control Manifest Rules

- Required: lift `apply_card_display_art` + `clear_card_display_art`
  to a **single owner** module. Both `hand/mod.rs` and
  `shop_auction/mod.rs` `use` the helper, and zero verbatim copies
  survive at sprint close.
- Required: slot-well chrome is preserved when card art is missing.
  Concrete strategy (separate child entity, or distinct
  `CardArtImageNode` component, or `PlaceholderAssets.shop_slot_well_idle`
  fallback in the Err branch) is **TBD by the implementing worker**;
  the worker MUST justify in the commit message which strategy was
  chosen.
- Required: `resolve_card_display_art` returns a non-`'static` type
  (`String`, `Arc<str>`, or `Cow<'static, str>` plus a registry lookup
  that returns `&'static str` for cached paths). The change MUST NOT
  leak memory per call. The call sites accept the new type via
  `AssetServer::load`'s `impl Into<AssetPath>` boundary.
- Required: existence check probes the asset registry (or filesystem
  in dev builds) before returning. Strategy choices include (a)
  startup walk of `CardCatalog` calling `asset_server.load_acquire`
  per art_id and logging warnings for missing files, (b) `OnEnter(
  ClientState::InSession)` precheck, or (c) on-demand probe at first
  bind. The implementation prompt picks one and documents it.
- Required: missing-art fallback path is documented (e.g.
  `PlaceholderAssets.card_art_missing` or equivalent). The fallback
  does NOT erase the slot well chrome.
- Required: no new server-authoritative state; no protocol shape
  change.
- Required: no new Lightyear channel; no new C2S / S2C message.
- Required: `PAW-TD-*-a` placeholder-art accept-risk preserved
  verbatim — no final-art replacement, no new asset production.
- Required: `QA-COND-0005` accept-risk preserved — no Standard-tier
  hit-target work on card surfaces by this row.
- Required: `QA-COND-0006` accept-risk preserved — no playtest
  validation claim.
- Forbidden: introducing a client-side card-data authority (catalog
  remains JSON-baked per existing pattern; no S2C `CardCatalog` is
  added).
- Forbidden: modifying `shared/`, `server/`, or any file under
  `tests/integration/server/`.
- Forbidden: real-art production in `assets/art/cards/display/` (any
  new art file pulls `PAW-TD-*-a` into scope and is OUT OF SCOPE for
  this row).
- Forbidden: changing per-call schedule placement of
  `sync_hand_fan_card_art_system`, `apply_shop_slot`, or
  `apply_shop_footer_slot`. Helpers are lifted; call sites and
  schedule slots remain.
- Forbidden: introducing animation / tween on the chrome / art
  swap. Slot chrome and art bindings are instantaneous, matching
  existing behaviour.
- Forbidden: closure of any AUDIT-1076-* or SOURCE-1077-* finding
  outside the four bundled here.

---

## Story Classification

**Story type**: **Logic + Integration** (mixed — single bundle).

- Helper dedup + leak fix + existence check are **Logic** (unit-
  testable via `World::new()` with the helper and a synthetic
  CardCatalog fixture).
- Slot-well chrome preservation is **Integration** (requires the
  shop-slot spawn site + the `apply_card_display_art` Err branch to
  interact correctly; requires a real `World` with the slot entity
  spawned).
- Per `.claude/docs/coding-standards.md` "Test Evidence by Story
  Type" matrix:
  - **Logic** rows require automated unit test (BLOCKING gate).
  - **Integration** rows require integration test OR documented
    playtest (BLOCKING gate).
- This row delivers BOTH — automated tests under `tests/unit/asset_wiring/`
  (or equivalent) AND `tests/integration/ui_clean_pass/` (or
  `tests/integration/shop_auction_ui/`).

This is **NOT** a:

- Visual / feel story (no shader, no VFX, no animation curve change).
- UI layout story (no menu / HUD / screen flow change).
- Config / data tuning story.
- Final-art / asset-production story (`PAW-TD-*-a` preserved).
- Accessibility story (`QA-COND-0005` preserved).

---

## Dependencies and Parallelism

### Prerequisites (must be on `origin/main` at Sprint 17 activation HEAD)

- None of the four PROMPT 1077 P0 findings has an in-flight repair
  track at draft time; the bundle is entirely net-new on Sprint 17.

### Conditional / overlap with other Sprint 17 rows

- **S17-UI-MODAL-BLACK-SLAB-001** (Sprint 17 conditional Must Have,
  PROMPT 1080 / 1083 in-flight): touches
  `client/src/ui/shop_auction/mod.rs` modal layout sites
  (`:5XXX` range per the in-flight worker). Helper lift may move the
  `apply_card_display_art` definitions out of `shop_auction/mod.rs`
  to `asset_wiring.rs` or a new module. **File-overlap-with-modal-
  repair** if both touch `shop_auction/mod.rs`. Sequencing: this
  story SHOULD land **after** the modal repair lands on `origin/main`
  so the implementing worker rebases on top of a stable shop_auction
  module shape. The Sprint 17 producer schedules accordingly.
- **S17-UI-SHOP-AUCTION-SURFACE-PAINT-001** (Sprint 17 conditional
  Must Have, PROMPT 1085 in-flight): same file-overlap concern
  (touches `client/src/ui/shop_auction/shop_slot*.rs` +
  `client/src/ui/shop_auction/auction_*.rs`). Sequencing: this story
  SHOULD land **after** the shop / auction surface paint lands so the
  shop-slot apply site is in its post-repair shape. The Sprint 17
  producer schedules accordingly.
- **S17-UI-CARD-SLOT-INSET-WIRING-001** (Sprint 17 Should Have,
  SOURCE-1077-06 sibling): touches
  `client/src/ui/design_tokens/card_slot.rs` only. **Parallel-safe**
  with this row — disjoint files. May land in either order.
- **S17-UI-PLACEMENT-PERSPECTIVE-001** (Sprint 17 conditional Must
  Have, PROMPT 1086 already on `origin/main` per the plan
  `next_sprint_17_draft` late-breaking update): pre-dropped at
  PROMPT 1090 commit. No file overlap.

### Per-surface card-slot migration siblings (out of scope)

The four `S17-UI-CARD-SLOT-MIGRATION-*` rows (HAND / DRAFT-GRID /
AUCTION-FEATURED / BOARD-GHOST) remain Sprint 17+ Backlog. This story
does NOT migrate any consumer surface — it only ratifies the helper
and the chrome separation; per-surface child-layout migration is a
separate row family.

### Parallelism summary

| Sibling story | Parallel-safe? | Notes |
|---|---|---|
| `S17-UI-MODAL-BLACK-SLAB-001` (conditional Must) | **NO** | both edit `client/src/ui/shop_auction/mod.rs`; serialise. |
| `S17-UI-SHOP-AUCTION-SURFACE-PAINT-001` (conditional Must) | **NO** | both edit `client/src/ui/shop_auction/`; serialise. |
| `S17-UI-CARD-SLOT-INSET-WIRING-001` (Should) | **YES** | disjoint (`design_tokens/card_slot.rs` only). |
| `S17-UI-HUD-OPP-MANA-CLEANUP-001` (Should) | **YES** | disjoint (`client/src/ui/hud/`). |
| `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` (Should) | **PARTIAL** | qa_snapshot lives in `client/src/presentation/`; marker definitions in `client/src/ui/hud/`, `client/src/ui/hand/`, `client/src/ui/shop_auction/`. File overlap on `hand/mod.rs` and `shop_auction/mod.rs` — serialise. |
| `S17-UI-BID-BUTTON-PHASE-RACE-001` (Should) | **NO** | both edit `client/src/ui/shop_auction/`; serialise. |
| `S17-UI-HAND-B0004-CLEANUP-001` (Nice) | **PARTIAL** | edits `client/src/ui/hand/fan_root*.rs`, `client/src/ui/hand/hand_bar*.rs`; potential overlap with the helper lift if hand fan call sites move into a re-imported helper. Serialise. |
| `S17-OPS-VULKAN-VALIDATION-GATING-001` (Nice) | **YES** | disjoint (`client/src/main.rs`). |
| `S17-SERVER-START-OF-TURN-DEBUG-001` (Nice) | **YES** | disjoint (`server/`). |

---

## Acceptance Criteria

All criteria are independently checkable.

- [x] **AC1 -- Helper exists at a single owner site (dedup, SOURCE-1077-02)**:
  GIVEN the post-implementation build, WHEN
  `grep -rn "fn apply_card_display_art" client/src/ shared/src/`
  is run, THEN it returns **exactly one** match. WHEN
  `grep -rn "fn clear_card_display_art" client/src/ shared/src/` is
  run, THEN it returns **exactly one** match. The single owner site
  is in `client/src/asset_wiring.rs` OR
  `client/src/ui/design_tokens/card_art.rs` (or equivalent new
  module). The two former copies in
  `client/src/ui/shop_auction/mod.rs` and `client/src/ui/hand/mod.rs`
  are deleted; both modules `use` the helper from the new owner.

- [x] **AC2 -- Slot-well chrome survives missing card art (SOURCE-1077-01)**:
  GIVEN a shop / draft / auction / hand slot is spawned with its
  spawn-time chrome `ImageNode` (currently
  `ImageNode::new(asset_server.load(SHOP_SLOT_WELL_IDLE_ASSET))` for
  the shop slot), WHEN a card whose `art_id` resolves to a missing /
  empty / unloadable path is bound to the slot via
  `apply_card_display_art`, THEN the slot's chrome `ImageNode`
  remains attached and visible. Automated assertion (see AC8). The
  chrome strategy chosen by the worker (separate `CardArtImageNode`
  child entity, OR distinct ImageNode component on the slot,
  OR `PlaceholderAssets`-fallback) is documented in the commit
  message and in the new module's doc comment.

- [x] **AC3 -- `resolve_card_display_art` does not leak (SOURCE-1077-03)**:
  GIVEN `client/src/asset_wiring.rs` post-refactor, WHEN
  `grep -rn "Box::leak" client/src/ shared/src/` is run, THEN there is
  no occurrence in the `resolve_card_display_art` body. The function
  signature returns a non-`'static` type (`String`, `Arc<str>`, or
  equivalent). Call sites at the helper consumer (hand fan, shop slot
  apply, auction featured apply, shop footer apply, draft offering)
  accept the new type via `AssetServer::load`'s
  `impl Into<AssetPath>` boundary or equivalent.

- [x] **AC4 -- Asset existence check at startup or session entry
  (SOURCE-1077-04)**: GIVEN the post-refactor client build runs against
  the production `CardCatalog`, WHEN the client enters `ClientState::
  InSession` (or at app startup — implementer's choice), THEN every
  card's `art_id` is probed via `AssetServer::load_acquire` (or
  equivalent existence check) and missing files emit a `warn!` log
  line with the offending `art_id` and the constructed path. Missing
  art does NOT panic. Missing art falls through to a documented
  placeholder asset (e.g. `PlaceholderAssets.card_art_missing` or
  equivalent — implementer's choice; recorded in the commit message).
  When the placeholder fires, the slot chrome (AC2) remains intact;
  the card-art ImageNode is set to the placeholder path.

- [x] **AC5 -- Existing card-painting behaviour preserved on a happy-path card**:
  GIVEN a card whose `art_id` resolves to a valid production art file
  on disk (e.g. one of the 16 art files in
  `assets/art/cards/display/` at audit time), WHEN
  `apply_card_display_art` runs for that card, THEN the slot's
  card-art ImageNode is set to that resolved path. AND the slot's
  chrome remains intact (AC2 strengthened — happy-path also preserves
  chrome). Integration test asserts both ImageNodes coexist on the
  slot subtree.

- [x] **AC6 -- `clear_card_display_art` removes only card-art binding,
  not chrome**: GIVEN a slot whose card-art ImageNode is currently
  bound, WHEN `clear_card_display_art` runs (e.g. on slot vacate /
  hand-card discard / shop-slot refresh), THEN only the
  `CardDisplayArtAsset` (card-art) component / child is removed; the
  slot's chrome ImageNode remains intact. The slot returns to an
  empty-well visual state, NOT a transparent rectangle.

- [x] **AC7 -- No silent failure on `art_id = "missing"`**: GIVEN a
  card fixture whose `art_id` is `"missing"` (canonical sentinel
  documented in the new module's doc-comment), WHEN
  `resolve_card_display_art` is called, THEN the function returns the
  documented placeholder path (per AC4) and the AssetServer load
  succeeds against an existing placeholder file. No `warn!` for the
  documented `"missing"` sentinel — only for unexpected missing
  paths.

- [x] **AC8 -- Unit tests cover the helper logic**: GIVEN
  `tests/unit/asset_wiring/card_display_art_helper_test.rs` (NEW) or
  an equivalent location, WHEN run, THEN it asserts:
  (a) `resolve_card_display_art` returns a non-leaking type
  (compiler-checked by the return type; runtime-checked by a 1000-
  call no-panic stress run).
  (b) `resolve_card_display_art` returns the placeholder path for
  `art_id = "missing"`.
  (c) `resolve_card_display_art` returns the production path for a
  known-good `art_id` from a fixture catalog.
  Tests use `World::new()` ECS test pattern per `liv-bevy-018`.

- [x] **AC9 -- Integration tests cover the chrome preservation
  invariant**: GIVEN `tests/integration/ui_clean_pass/card_slot_chrome_preservation_test.rs`
  (NEW) or `tests/integration/shop_auction_ui/card_slot_chrome_preservation_test.rs`
  (NEW; exact target dir chosen by the implementing worker to match
  the post-modular-split layout), WHEN run, THEN it asserts:
  (a) shop slot spawn site produces an entity carrying the chrome
  `ImageNode` (asserted by querying `(With<ShopSlotMarker>,
  With<ImageNode>)` or equivalent post-refactor marker).
  (b) After calling `apply_card_display_art` with a missing-art
  fixture, the chrome marker still carries `ImageNode` (the chrome
  ImageNode survives). The card-art child / component is set to the
  placeholder.
  (c) After calling `clear_card_display_art`, the chrome marker still
  carries `ImageNode`. The card-art binding is gone.
  Tests use `App::new()` with the relevant sub-plugins per the
  existing `tests/integration/shop_auction_ui/` fixtures (PROMPT 1067
  / 1073 style; see `card_slot_primitive_test.rs` if it exists).

- [x] **AC10 -- Test fixture art-id coverage**: at least one new
  test asserts the existence-check behaviour with **two** fixtures:
  one whose `art_id` is the production `"missing"` sentinel (path
  resolves to placeholder), and one whose `art_id` is
  `format!("absent_{n}")` for some `n` (path does NOT resolve;
  startup warn fires). The warn assertion uses a captured logger
  facility or an equivalent observer pattern; if the existing test
  harness has no logger capture hook, the AC is satisfied by an
  alternative observable side-effect (e.g. a `Resource` counting
  missing-art warnings).

- [x] **AC11 -- ADR-021 schedule preserved**: GIVEN `cargo build -p
  client`, WHEN run under the Cargo resource policy (§"Cargo resource
  policy" below), THEN no new system-set or schedule wiring is
  introduced. The helper consumers (hand fan, shop slot, auction
  featured) keep their existing `PresentationSet` slot. The new
  existence-check system slots into `OnEnter(ClientState::InSession)`
  OR `Startup`; either is acceptable.

- [x] **AC12 -- No protocol or server change**: GIVEN
  `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN there are
  zero changes under `server/`, `shared/`, `tests/integration/server/`,
  or `tests/unit/server/`. The implementation is client-side only.
  No new `C2S*` or `S2C*` message; no new `shared::card::*` type or
  field.

- [x] **AC13 -- No accept-risk closure claimed**: GIVEN the
  commit message and any evidence document, WHEN inspected, THEN
  they explicitly do NOT claim closure of `S8-QA-001-W1`,
  `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a` (specifically
  `PAW-TD-004-a` for card art placeholder accept-risk), or any
  other accept-risk disposition. Final-art replacement of any of
  the 16 production art files is explicitly out of scope.
  Standard-tier accessibility is not pursued. Playtest validation
  is not pursued.

- [x] **AC14 -- Sprint 17 disposition preserved**: GIVEN the
  implementation commit(s), WHEN
  `production/sprint-status.yaml`, `production/sprints/sprint-17.md`,
  `production/sprints/sprint-16.md` (and earlier), `production/stage.txt`,
  `production/session-state/*`, `production/qa/*`,
  `production/gate-checks/*`, and `docs/architecture/adr-*.md` are
  diffed, THEN none are modified by this story's `/dev-story` worker.
  Story-file status flips and sprint-status row updates are reserved
  for the future `/story-readiness` and `/story-done` paperwork
  prompts that follow `/dev-story`.

- [x] **AC15 -- Worker branch scope contained**: GIVEN the
  implementation worker branch (slug recommendation:
  `work/s17-card-display-art-helper-bundle`), WHEN inspected, THEN
  it pushes only the worker branch — never `main`. Files changed at
  worker time are scoped to `client/src/asset_wiring.rs`,
  `client/src/ui/design_tokens/card_art.rs` (if chosen),
  `client/src/ui/shop_auction/mod.rs` (delete-only for the two helper
  function bodies + `use` import addition), `client/src/ui/hand/mod.rs`
  (delete-only for the two helper function bodies + `use` import
  addition), and new test files under
  `tests/unit/asset_wiring/` and `tests/integration/ui_clean_pass/`
  or `tests/integration/shop_auction_ui/` per AC8 / AC9.

- [x] **AC16 -- Cargo resource policy applied for every Cargo
  command**: future implementation MUST set the binding Cargo
  resource policy env vars (`CARGO_TARGET_DIR=
  D:\_DEV\cargo-target\ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`,
  `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`,
  `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`) before every
  `cargo check` / `cargo test` invocation on Windows / MSVC. Disk
  preflight (~>= 50 GB free on D:) recorded in the evidence file.
  Story authoring (PROMPT 1095) does NOT invoke Cargo; no evidence
  file is authored here. `/dev-story` worker authors the evidence
  document under `production/qa/evidence/sprint-17-card-display-art-helper/`
  (NEW; reserved by this story file but NOT created by this
  authoring run).

---

## Implementation Notes

### Owned files (likely change set; final list deferred to `/dev-story`)

| Path | Expected change |
|------|-----------------|
| `client/src/asset_wiring.rs` (or new `client/src/ui/design_tokens/card_art.rs`) | **Add** the single-owner helper functions (`apply_card_display_art`, `clear_card_display_art`, refactored `resolve_card_display_art`). Add the existence-check system (or integrate it into an existing startup sequence). |
| `client/src/ui/shop_auction/mod.rs` | **Delete** the local copy of `apply_card_display_art` + `clear_card_display_art`; **add** `use crate::asset_wiring::{apply_card_display_art, clear_card_display_art};` (or equivalent path). Update spawn site at the slot well to use the chrome strategy chosen (e.g. spawn a separate `CardArtImageNode` child entity). |
| `client/src/ui/hand/mod.rs` | Same delete + `use` pattern; same chrome strategy at the hand-fan / draft-grid spawn sites if applicable. |
| `client/src/main.rs` or wherever `ClientPlugins::build` runs | **Possibly add** registration of the new existence-check system (one line). |
| `tests/unit/asset_wiring/card_display_art_helper_test.rs` (NEW; or equivalent path matching the existing `tests/unit/` directory layout) | **New** unit test bin for AC8 helper behaviour. |
| `tests/integration/ui_clean_pass/card_slot_chrome_preservation_test.rs` OR `tests/integration/shop_auction_ui/card_slot_chrome_preservation_test.rs` (NEW) | **New** integration test bin for AC9 chrome preservation. |
| `production/qa/evidence/sprint-17-card-display-art-helper/evidence.md` (NEW, by `/dev-story` worker) | Evidence document; NOT authored by PROMPT 1095. |

Exact line ranges and module paths MUST be re-verified at Sprint 17
activation HEAD; the line numbers above reflect the audit-time
`origin/main@e6a6e11` cited by PROMPT 1077.

### Forbidden files

- Everything under `server/`, `shared/`.
- Everything under `tests/integration/server/`, `tests/unit/server/`,
  `tests/integration/lightyear*`, `tests/unit/lightyear*`.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/*`.
- All other `production/epics/*` story files (no cross-epic edit).
- `assets/art/cards/display/*.png` (no real-art production;
  `PAW-TD-*-a` preserved).
- `docs/architecture/adr-*.md` (no ADR amendment in scope; if the
  worker concludes an ADR amendment is required, the worker pauses
  and escalates).
- `.claude/`, `AGENTS.md`, `CLAUDE.md`, `CODEX.md`.

### Cargo resource policy (binding for all Sprint 17 Cargo invocations)

Per `production/qa/qa-plan-sprint-15.md` §"Cargo Resource Policy on
Windows/MSVC" binding precedent (preserved across PROMPT 815 / 833 /
844 / 851 / 872 / 884 / 889 / 902 / 906 / 907 / 912 / 917 / 918 / 930
/ 938 / 941 / 951 / 955 / 959 / 961 / 970 / 973 / 975 / 982 / 983 and
preserved through Sprint 16 PROMPT 1066), every `cargo` invocation on
Windows / MSVC MUST set:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

The Sprint 17 `/qa-plan` prompt (a separate post-activation prompt)
is expected to restate this policy verbatim; this story file MUST
NOT amend the policy.

### Target citations

- Sprint 17 plan row source: `production/sprints/sprint-17.md`
  §"Must Have (Critical Path)" row `S17-UI-CARD-DISPLAY-ART-HELPER-001`.
- Sprint 17 plan rationale: same file §"Planning Notes" bullet 6
  ("The PROMPT 1077 P0 structural findings ... Single bundled row
  recommended (dedup + leak fix + existence check) because the two
  findings must land together to avoid re-introducing the bug after
  dedup.") and §"Sprint Goal" item 2 ("Land the PROMPT 1077 P0
  structural card-display-art bundle ...").
- Source audit: `reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md`
  §"Per-finding evidence" SOURCE-1077-01 / 02 / 03 / 04.

---

## Worker Contract (for future `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout` against Sprint 17 activation HEAD on a fresh
   worktree (suggested slug `work/s17-card-display-art-helper-bundle`).
2. Read this story file end-to-end before any code change.
3. Re-verify the audit-time line ranges by reading the current
   `client/src/asset_wiring.rs`,
   `client/src/ui/shop_auction/mod.rs`, and
   `client/src/ui/hand/mod.rs` (the post-Sprint 16 close-out and any
   in-flight Sprint 17 modal / surface-paint repairs that have landed
   by activation may have shifted line numbers).
4. Re-verify chrome / card-art entity structure on the current shop
   slot, hand fan slot, auction featured card, and draft initial
   grid spawn sites; choose the chrome-vs-art separation strategy
   (separate `CardArtImageNode` child entity vs distinct ImageNode
   vs `PlaceholderAssets`-fallback) and document the choice in the
   commit message.
5. Choose the single-owner module (`client/src/asset_wiring.rs` or
   a new `client/src/ui/design_tokens/card_art.rs` module) and
   justify in the commit message.
6. Activate `liv-bevy-018` skill before any `.rs` edit. Do NOT
   activate `liv-bevy-lightyear` — this row does not touch Lightyear.
7. Apply the four changes in a SINGLE commit (per Sprint 17 plan
   bundling rationale): dedup + chrome preservation + leak fix +
   existence check. Do NOT split into multiple commits.
8. Author the two test bins (AC8 unit; AC9 integration) and the
   evidence document under `production/qa/evidence/sprint-17-card-
   display-art-helper/`.
9. Set the Cargo resource policy env vars per AC16 before every
   `cargo check` / `cargo test` invocation.
10. Run `cargo check -p client` and the targeted `cargo test -p
    client --test <bin>` invocations for the new test bins; confirm
    zero new warnings on the touched files (existing warnings on
    other files are NOT addressed by this story).
11. Push the worker branch (never `main`).
12. Stop. Closure paperwork (`/story-done`, integration `/no-ff`
    merge) is a later prompt's scope.

The worker MUST NOT:

- Modify `server/`, `shared/`, or any file under
  `tests/integration/server/` or `tests/unit/server/`.
- Modify `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`,
  `Trunk.toml`.
- Modify any file under `production/sprint-status.yaml`,
  `production/sprints/`, `production/stage.txt`,
  `production/session-state/`, `production/qa/qa-plan-*.md`,
  `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`,
  `production/gate-checks/`.
- Run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, or `/qa-plan` on this story.
- Run the full workspace `cargo test --workspace --tests
  --no-fail-fast` invocation (forbidden per Sprint 15+ QA Policy
  §"Test Scope Per Prompt Type"; targeted `cargo check -p client` +
  the named per-bin `cargo test` is the policy-correct shape).
- Run `trunk` or any CI command.
- Push to `main`.
- Touch `assets/art/cards/display/*.png` or any other production
  art asset.
- Claim closure of any AUDIT-1076-* or SOURCE-1077-* finding
  outside the four bundled here (SOURCE-1077-01 / 02 / 03 / 04).
- Claim release-readiness, accessibility-completion, playtest-
  validation, two-client GAME_OVER closure, final-art completion,
  or stage advance.

### Build gate scope (parallel-agent isolation)

In a parallel-agent Sprint 17 cohort, the `/dev-story` worker for
this story MAY be running concurrently with other Sprint 17 workers.
The build gate for this story MUST be scoped to the files this
worker owns:

- `cargo check -p client` is acceptable.
- Targeted `cargo test -p client --test <named bin>` for the bins
  authored under AC8 / AC9 is acceptable.
- The worker MUST NOT block on workspace-wide compilation errors
  introduced by other in-flight workers' branches; if a sibling
  worker's broken types appear at `git fetch` time, the
  serialised file-overlap rows (S17-UI-MODAL-BLACK-SLAB-001,
  S17-UI-SHOP-AUCTION-SURFACE-PAINT-001, S17-UI-QA-SNAPSHOT-MARKER-
  SPLIT-001) MUST already have landed on `origin/main` per the
  scheduling notes above — that is the orchestrator's
  responsibility, not the worker's.

### Relay / reporting expectation for future workers

Per the Sprint 17 plan and orchestrator contract, every implementing
worker reports back through the GCS local app-server relay (one
single-line DONE summary). The final status line for this story
SHALL be:

```
N: S17-UI-CARD-DISPLAY-ART-HELPER-001: STATUS
```

where `N` is the prompt number that ran `/dev-story` and `STATUS`
is a real outcome word (DONE, BLOCKED, FAILED, PASS — not a colour
name). Per the 2026-05-13 override rule "No delimiter line, no
HTML/span/CSS/ANSI markup."

---

## Completion Notes

Closed by PROMPT 1117 /story-done paperwork on 2026-05-18 against
source-of-truth `origin/main@30c9e0f6d7b867d25d3f8ba5d273c2f1890b02a7`
(PROMPT 1114 integration tip
`integrate(s17): merge PROMPT 1113 card-display art-helper into main
(PROMPT 1114)` merging PROMPT 1113 worker
`4f577d68610e5231a94385634d828edd913a1f4e`
`dev-story(s17-card-display-art-helper): lift helper to single owner
+ remove leak + chrome preservation + existence-check probe (PROMPT
1113)` onto `origin/main` via no-ff merge).

### PROMPT 1113 worker + PROMPT 1114 integration outcome

- **SOURCE-1077-02 dedup** -- `apply_card_display_art` +
  `clear_card_display_art` lifted to `client/src/asset_wiring.rs` as
  `pub fn` (verified post-merge: `apply_card_display_art` at
  `asset_wiring.rs:594`, `clear_card_display_art` at
  `asset_wiring.rs:627`). Verbatim copies in
  `client/src/ui/shop_auction/mod.rs` and
  `client/src/ui/hand/mod.rs` deleted; both modules now
  `use crate::asset_wiring::{apply_card_display_art,
  clear_card_display_art, ...}`.
- **SOURCE-1077-01 chrome preservation** -- `apply` Err branch +
  `clear` no longer remove `ImageNode`; spawn-time chrome
  (e.g. shop slot's `SHOP_SLOT_WELL_IDLE_ASSET` well) survives
  missing card art. Worker chose the
  "do-not-touch-`ImageNode`-on-Err/Clear" strategy from the three
  documented options; rationale recorded in evidence.md §"Chrome-
  preservation strategy".
- **SOURCE-1077-03 leak fix** -- `resolve_card_display_art` returns
  `Result<String, ...>`; `Box::leak` removed. `CardDisplayArtAsset.
  path: String` (was `&'static str`). Verified post-merge: no
  `Box::leak` matches under `client/src/` outside the historical
  doc-comment at `asset_wiring.rs:553` describing the prior bug.
- **SOURCE-1077-04 existence check** -- new
  `probe_card_display_art_paths` system registered on
  `OnEnter(ClientState::InSession)`; emits `warn!` with `art_id` +
  `path` for missing files; `MissingCardArtWarnings` resource counts
  warnings for test observability. The documented `"missing"`
  sentinel routes through the placeholder without firing a warn
  (AC7).

### Test evidence

- `tests/unit/asset_wiring/card_display_art_helper_test.rs` (NEW;
  AC8) -- 6/6 pass.
- `tests/integration/presentation/card_display_art_chrome_preservation_test.rs`
  (NEW; AC9 + AC10) -- 8/8 pass.
- Adjusted for `&'static str` -> `String` signature change:
  `tests/integration/shop_auction_ui/shop_panel_test.rs` 10/10,
  `tests/integration/shop_auction_ui/auction_activation_test.rs`
  8/8, `tests/integration/hand-ui/draft_initial_grid_test.rs` 6/6,
  `tests/integration/playable_client/draft_shop_hand_bridge_test.rs`
  5/5.
- Adjacent regression sweep at integration tip:
  `asset_wiring_foundation_test` 9/9,
  `hand_ui_asset_wiring_test` 10/10,
  `shop_auction_asset_wiring_test` 5/5,
  `shop_auction_ui_card_cost_combat_stat_rendering_test` 8/8,
  `ui_clean_pass_card_slot_primitive_test` 27/27.
- Evidence file:
  `production/qa/evidence/sprint-17-card-display-art-helper/evidence.md`.

### Cargo resource policy advisory (AC16)

PROMPT 1114 integration recorded a process / policy advisory note:
the PowerShell-syntax env-var block (`$env:CARGO_TARGET_DIR=...` +
`CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` +
`CARGO_INCREMENTAL=0` + `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`)
did not propagate through the bash->powershell boundary on its
first invocation, so the first `cargo test` invocation built into
the integration worktree-local `target/` rather than
`D:\_DEV\cargo-target\ccgs-msvc`. Resource impact: a few GB local
to the worktree; D: free remained ~744 GB (well above the 50 GB
preflight threshold). All 11 test binaries + 2 cargo check
invocations passed against the merged tree (102 test cases total).
**Build correctness gate the integration prompt required is
unaffected.** Recorded explicitly as an advisory / process note,
not a product failure, in this Completion Notes section, in
`reports/PROMPT-1114-s17-card-display-art-helper-integration.md`,
in the `production/sprint-status.yaml` `sprint_17_story_done:`
PROMPT 1117 `batch_note` + row notes, in the
`production/session-state/active.md` PROMPT 1117 banner, and in
the `production/session-state/codex-orchestrator-state.md` PROMPT
1117 paragraph. **NOT hidden.**

### Per-AC outcome

- AC1 single owner -- PASS (`grep -rn "fn apply_card_display_art" client/src/ shared/src/`
  -> 1 match at `client/src/asset_wiring.rs:594`; same for
  `clear_card_display_art` at line 627).
- AC2 chrome survives missing card art -- PASS
  (`shop_slot_chrome_survives_missing_card_art_apply`).
- AC3 no `Box::leak` -- PASS (`grep -rn "Box::leak" client/src/`
  -> 0 functional matches; resolver returns `String`).
- AC4 existence check -- PASS (`probe_card_display_art_paths`
  registered on `OnEnter(ClientState::InSession)`; warns with
  `art_id` + `path` per missing file; counts in
  `MissingCardArtWarnings`).
- AC5 happy-path apply preserves card-art + chrome -- PASS.
- AC6 clear preserves chrome -- PASS.
- AC7 `missing` sentinel routes to placeholder without warn -- PASS.
- AC8 unit tests -- PASS (6/6).
- AC9 integration tests -- PASS (8/8).
- AC10 fixture coverage -- PASS (documented sentinel + absent_N
  observable via `MissingCardArtWarnings` resource counter).
- AC11 ADR-021 schedule preserved -- PASS (helper consumers'
  `PresentationSet` placement unchanged; probe slots into existing
  `OnEnter(InSession)`).
- AC12 no protocol / server change -- PASS (`git diff
  origin/main^...30c9e0f` touches no `server/`, `shared/`,
  `tests/integration/server/`, `tests/unit/server/`).
- AC13 no accept-risk closure -- PASS (no claim of
  `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  or other accept-risk disposition).
- AC14 Sprint 17 disposition preserved by worker + integration --
  PASS (PROMPT 1113 worker + PROMPT 1114 integration diffs
  touched zero files under `production/sprint-status.yaml`,
  `production/sprints/`, `production/stage.txt`,
  `production/session-state/`, `production/qa/qa-plan-sprint-17.md`,
  `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`,
  `production/gate-checks/*`, `docs/architecture/adr-*.md`;
  PROMPT 1117 is the first authorised modifier of
  `production/sprint-status.yaml` + `production/session-state/*`
  for this row).
- AC15 worker branch scope contained -- PASS (PROMPT 1113 worker
  pushed `work/s17-card-display-art-helper` only; never `main`;
  integration into `origin/main` performed separately by PROMPT
  1114 via `integrate/s17-card-display-art-helper-1114` ->
  `30c9e0f`).
- AC16 Cargo resource policy applied -- **PASS-WORKER +
  ADVISORY-INTEGRATION**. PROMPT 1113 worker applied all 5 env
  vars before every cargo invocation (evidence.md AC16 row).
  PROMPT 1114 integration encountered the bash->powershell env-var
  propagation gap noted above; build correctness gate
  unaffected; advisory recorded explicitly per the prompt's
  binding-record requirement. PROMPT 1117 itself does NOT invoke
  Cargo (paperwork-only closure).

### Closure trail (commits)

1. **PROMPT 1095** -- net-new Sprint 17 story authoring batch
   (story 017 drafted).
2. **PROMPT 1097** -- paperwork-only main integration of the
   Sprint 17 story authoring batch (`bc3db29`).
3. **PROMPT 1099** -- Sprint 17 activation (`cb62a9e`).
4. **PROMPT 1100** -- `/qa-plan sprint-17` authoring (`ff47075`).
5. **PROMPT 1113** -- `/dev-story` worker
   (`4f577d68610e5231a94385634d828edd913a1f4e`) on branch
   `work/s17-card-display-art-helper`.
6. **PROMPT 1114** -- integration of PROMPT 1113 onto
   `origin/main` via no-ff merge
   (`30c9e0f6d7b867d25d3f8ba5d273c2f1890b02a7`) on branch
   `integrate/s17-card-display-art-helper-1114`.
7. **PROMPT 1117** -- this `/story-done` paperwork (flips ACs to
   `[x]`; appends Completion Notes; records closure tip; updates
   `production/sprint-status.yaml`,
   `production/session-state/active.md`,
   `production/session-state/codex-orchestrator-state.md`).

### Conditions carried forward unchanged (preserved by every prompt above)

- Sprint 16 disposition `closed-with-conditions` per PROMPT 1082 +
  PROMPT 1088 (UNCHANGED).
- Sprint 17 stage `Polish` (UNCHANGED; `production/stage.txt` NOT
  modified by any prompt in the sequence).
- PROMPT 761 `Polish->Release` gate-check `FAIL` preserved; **NO
  retry** in Sprint 17.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` (friend-game scope) + `QA-COND-0006` (playtest
  deferred) accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved across PAW-002
  .. PAW-006.
- `TQ-S12-C1..C7` preserved verbatim.
- Sprint 15 / 14 / 13 / 12 / 11 / 10 dispositions preserved
  unchanged.
- HUD timer row `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-
  blocked carry preserved; NOT closed by this row.
- 24 PROMPT 1022 audit findings preserved as report-only; NOT closed
  by this row.

### Explicitly NOT claimed by this story or its `/dev-story` worker

- Sprint 17 close-out.
- Closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-
  blocked Must Have carry).
- Closure of any AUDIT-1076-* finding (this story addresses
  SOURCE-1077-01 / 02 / 03 / 04 only).
- Closure of any SOURCE-1077-* finding other than the four named
  above (SOURCE-1077-05 / 07 / 11 / 12 / 13 / 14 / 15 remain Sprint
  18+ Backlog; SOURCE-1077-06 / 08 / 09 / 10 / 16 are addressed by
  sibling Sprint 17 rows).
- Closure of any of the 24 PROMPT 1022 audit findings.
- Public release readiness; release-candidate readiness; full game
  completion.
- Broad / Standard-tier accessibility completion; playtest /
  fun-hypothesis validation; full playable-client manual QA;
  two-client GAME_OVER closure; final-art / asset-production
  completion; Polish->Release gate-check retry; stage advance.
- Per-surface card-slot primitive migration of any consumer surface
  (HAND / DRAFT-GRID / AUCTION-FEATURED / BOARD-GHOST); those
  remain Sprint 17+ Backlog candidates per
  `production/sprints/sprint-17.md` §"Wider Sprint 17 Backlog".

`017: S17-UI-CARD-DISPLAY-ART-HELPER-001: DONE`
