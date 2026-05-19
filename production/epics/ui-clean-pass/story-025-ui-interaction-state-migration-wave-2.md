# Story 025: S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001 -- Bind `interaction_states::*` Tokens to P1 Surfaces

> **Epic**: UI Clean-Pass
> **Story ID**: S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001
> **Status**: Draft -- Sprint 18 candidate row (sprint **active**, stage **Polish**); NOT implemented; NOT marked `READY` while active source workers PROMPT 1347 / 1348 / 1349 remain unlanded (see §"Dependencies and Parallelism" blocking note).
> **Layer**: Presentation -- per-surface interaction-state token binding (lobby + hand + shop_auction)
> **Type**: Tech Debt -- per-surface migration (root-cause RC-4)
> **Sprint**: Sprint 18 candidate per PROMPT 1180 §6 Lane I.
> **Authored**: 2026-05-18 by PROMPT 1189
> **Refreshed**: 2026-05-19 by PROMPT 1355 (folds PROMPT 1350 readiness findings)
> **Authoring source-of-truth**: `origin/main@efb698e`
> **Refresh source-of-truth**: `origin/main@6e0453f8` (PROMPT 1346 settings-panel reconcile tip, fetched 2026-05-19)
> **Estimated effort**: ~0.8d
> **Source audit**: PROMPT 1180 §2 RC-4, §6 Lane I (PROMPT 1198 candidate); cross-refs L-03, S-05, UI-1129-09. Refresh audit: `reports/PROMPT-1350-s18-ui-interaction-state-migration-wave-2-readiness.md`.

---

## Status / No-Claim Banner

Sprint 18 candidate row. Sprint 18 is **active**; stage remains **Polish**. **No implementation landed.** No claim on release readiness, `QA-COND-0005` Standard-tier completion (affordance is one input, not closure), `QA-COND-0006`, `PAW-TD-*-a`, gate-check retry, stage advance, final-art replacement, or closure of any audit finding outside Lane I / RC-4. This refresh (PROMPT 1355) is paperwork-only: it patches the story body to be coherent with current `origin/main` so a later `/dev-story` worker can launch cleanly once the source-collision lanes clear. PROMPT 1355 does NOT mark the story `READY`, does NOT run `/dev-story`, does NOT touch source / tests / sprint / QA / gate-check / session-state files, and does NOT advance any accept-risk disposition.

## Problem Class / Prevention Target

**Defect class** (RC-4 -- refreshed 2026-05-19): `client/src/ui/design_tokens/interaction_states.rs` (story 008 / PROMPT 1009 Done) **publishes** the canonical `HOVER_BG_TINT_ALPHA` / `PRESSED_BG_TINT_ALPHA` / `DISABLED_BG_TINT_ALPHA` overlay-tint contract, but **token imports are not consistently applied as that contract at P1 spawn sites**. The original PROMPT 1180 audit framed this as "unconsumed"; that framing is **partially stale** on current `origin/main`:

- `client/src/ui/lobby.rs:22-25` already imports the canonical tokens and consumes `HOVER_BG_TINT_ALPHA` / `HOVER_BORDER_ALPHA` / `PRESSED_BG_TINT_ALPHA` / `DISABLED_BG_TINT_ALPHA` / `DISABLED_BORDER_ALPHA` / `DISABLED_TEXT_ALPHA` in `lobby_confirm_button_colors` -- but feeds them into a per-state palette that *also* hand-picks inline `Color::srgb(...)` triples for the Enabled / Hovered / Pressed / InFlight / Waiting / Confirmed bands. The overlay-tint contract is not the live driver of the visible palette swap; only the `Disabled` band is fully token-derived.
- `client/src/ui/design_tokens/card_slot.rs` references `interaction_states` in doc comments (no runtime consumption).
- `client/src/ui/shop_auction/mod.rs` and `client/src/ui/hand/mod.rs` do **not** import the overlay-tint tokens at all; their primary-action button spawn sites already carry visible baseline chrome via `primary_action_button_background_color()` / `primary_action_button_border_color()` (PROMPT 1182, lines `113` / `117`) but no hover / pressed / disabled state-swap path.

**Updated defect class**: tokens are imported in one surface but **not applied as the documented overlay/state contract**; the other P1 surfaces (shop ready / refresh, auction bid / pass, placement Submit) have no overlay-tint path at all. Mirror failures from PROMPT 1180: lobby confirm CTA (L-03), shop ready / refresh as text-only buttons (UI-1129-09 -- partially addressed by PROMPT 1182's baseline chrome), bid increments as baked-`?` PNGs (S-05). HUD pills still read as clickable buttons despite being status chips.

**Prevention target**: every P1 button (lobby create / join, shop ready + refresh, auction bid increments + pass, placement Submit, **plus** the lobby Confirm CTA's hover / pressed / disabled bands -- see AC1) consumes the canonical `interaction_states::*` overlay-tint contract via `Interaction` change-detection, with token-consistent semantics for hover / pressed / disabled across surfaces. Status chips (HUD pills, lobby status banner, effective-timer readout) continue to NOT carry `Interaction` (§5 C-4).

## 1180 Lane Coverage

Owns Lane I:

> | **I — Interaction-state migration (P1 surfaces)** | `client/src/ui/lobby.rs` (E owner), `client/src/ui/shop_auction/mod.rs` (H owner), `client/src/ui/hand/mod.rs` (new owner) | `tests/integration/ui_clean_pass/interaction_state_consumer_coverage_test.rs` (NEW) | **P2** | After E + H + G complete |

Wave 3 — runs after Lanes E (PROMPT 1178), G (PROMPT 1183), H (PROMPT 1182) land on `origin/main`. **All three landed** on current main as of refresh date (PROMPT 1350 §1 verified): 1178 → `80a6699`, 1182 → `54f4185`, 1183 → `577ad95`. Wave 1 prerequisite gate is therefore **CLEARED**. However, an additional **active-worker collision gate** now blocks `/dev-story` -- see §"Dependencies and Parallelism" blocking note below.

## Context

Refreshed line / symbol citations against `origin/main@6e0453f8`. **Line numbers are advisory** in 5 k+ line modules with active in-flight editors; **grep-by-symbol is binding** when an exact line drifts.

- `client/src/ui/design_tokens/interaction_states.rs` — story 008 module; PUBLISHED with full overlay-tint contract; PARTIALLY consumed today (lobby imports the tokens but does not drive the visible palette swap through them).
- `client/src/ui/lobby.rs:22-25` — token imports already wired (`HOVER_BG_TINT_ALPHA`, `HOVER_BORDER_ALPHA`, `PRESSED_BG_TINT_ALPHA`, `DISABLED_BG_TINT_ALPHA`, `DISABLED_BORDER_ALPHA`, `DISABLED_TEXT_ALPHA`).
- `client/src/ui/lobby.rs::lobby_confirm_button_colors` — existing per-state palette (Enabled / Hovered / Pressed / InFlight / Waiting / Confirmed / Disabled). See AC1 for the grandfather decision.
- `client/src/ui/lobby.rs::LobbyConfirmButton` (declaration) — see AC1.
- `client/src/ui/lobby.rs::LobbyCreateRoomButton` (decl + spawn, advisory ~line 1256) and `LobbyJoinRoomButton` (decl + spawn, advisory ~line 1271) — currently spawned with inline `Color::srgba(...)` triples; AC2 migrates these to token-derived overlay tints.
- `client/src/ui/shop_auction/mod.rs::primary_action_button_background_color` (decl `113`) and `primary_action_button_border_color` (decl `117`) — PROMPT 1182 baseline-chrome helpers; AC3 / AC4 build **on top of** these, not replace them.
- `client/src/ui/shop_auction/mod.rs::ShopReadyButton`, `ShopRefreshButton`, `AuctionBidButton`, `AuctionPassButton` — P1 button components; spawn sites consume `primary_action_button_*` helpers today (visible baseline shipped by PROMPT 1182). Wave 2 layers overlay-tint deltas on top.
- `client/src/ui/hand/mod.rs:673` — `pub struct HandSubmitButton;` declaration (NOTE: original story authored `PlacementSubmitButton`; that name does not exist on current main; corrected by PROMPT 1355 refresh).
- `client/src/ui/hand/mod.rs` spawn site at advisory ~line 3972 — `HandSubmitButton` spawned with `Button + Interaction::None + Text + TextColor(PLACEMENT_ACTION_PANEL_BUTTON_TEXT_COLOR) + BackgroundColor(PLACEMENT_ACTION_PANEL_BUTTON_BACKGROUND) + BorderColor::all(PLACEMENT_ACTION_PANEL_BUTTON_BORDER) + HandSubmitInteractionState::Inactive` (line numbers advisory; bind by symbol `HandSubmitButton`).
- `client/src/ui/hand/mod.rs::HandSubmitInteractionState` (decl advisory ~line 752) — `{ Active, Inactive }` enum tracked by the existing per-frame refresh path; AC5 must integrate the overlay-tint contract without regressing this two-state semantic.

**GDD / ADR**: no change.

**Engine / skills**: Bevy 0.18; `liv-bevy-018`; `Interaction` change-detection canonical.

### Control Manifest Rules

- Required: every P1 button carries (i) `Interaction`; (ii) `BackgroundColor` from `Default` token (token-derived; for shop / auction / draft primary actions the `Default` band is the PROMPT 1182 `primary_action_button_background_color()` helper, layered with no overlay tint at `Default`); (iii) change-detection system swapping between four states on `Interaction.changed()`.
- Required: status chips (HUD pills, lobby status banner, effective-timer readout) do NOT carry `Interaction` (§5 C-4).
- Required: cursor changes on hover; status chips unchanged.
- Required: shop / auction / draft primary-action button spawn sites continue to consume `primary_action_button_background_color()` / `primary_action_button_border_color()` (PROMPT 1182 baseline chrome). Wave 2 LAYERS overlay-tint deltas on top of those helpers via the change-detection system; Wave 2 does NOT replace the helpers and does NOT delete the baseline visible chrome.
- Forbidden: new RGB literals at spawn sites (see AC10 for the AC1-grandfather carve-out).
- Forbidden: editing `interaction_states.rs` (consume-only).
- Forbidden: regressing PROMPT 1182's visible baseline chrome on shop / auction / draft primary-action buttons.
- Forbidden: regressing the lobby Confirm CTA's existing seven-state per-frame palette (Disabled / Enabled / Hovered / Pressed / InFlight / Waiting / Confirmed) shipped by PROMPT 1081 / PROMPT 1138 -- see AC1.

## Story Classification

**Integration**.

## Dependencies and Parallelism

### Prerequisites (BLOCKING)

- Lane E (PROMPT 1178), Lane G (PROMPT 1183), Lane H (PROMPT 1182) landed on `origin/main`. **CLEARED** as of refresh (verified by PROMPT 1350 §1).

### Active-Worker Collision Block (BLOCKING — refresh 2026-05-19)

`/dev-story` for this row is **BLOCKED** until the following three active source workers either land on `origin/main` OR are formally abandoned. They overlap Wave 2's two largest owned files (`client/src/ui/shop_auction/mod.rs` and `client/src/ui/hand/mod.rs`):

| Active worker | Scope overlap with Wave 2 owned files |
|---|---|
| **PROMPT 1347** (`S18-AUCTION-WON-CARD-DISPOSITION-DEV-STORY`, story 020) | Edits `client/src/ui/shop_auction/mod.rs` (auction-won card disposition + winner discoverability) AND `client/src/ui/hand/mod.rs` (winner-side hand affordance / newly-acquired pulse). **HARD COLLISION** with both Wave 2 files. |
| **PROMPT 1348** (`S18-UI-CARD-ART-AND-LABEL-STRIP-DEV-STORY`, story 022 / Lane C) | Owned files explicitly include `client/src/ui/hand/mod.rs::sync_hand_fan_card_art_system`, `client/src/ui/shop_auction/mod.rs::handle_draft_offering_system`, `client/src/ui/shop_auction/mod.rs::auction_featured_card_node`, `client/src/ui/design_tokens/card_slot.rs`. **HARD COLLISION** with both Wave 2 files. |
| **PROMPT 1349** (`S18-UI-OVERLAY-PANEL-OVERFLOW-HARDENING-DEV-STORY`, story 026 / Lane J) | Owns `client/src/ui/shop_auction/mod.rs::{draft_initial_modal_panel_node, draft_initial_slot_node, draft_initial_grid_node}` (PROMPT 1182 surface). **HARD COLLISION** on `shop_auction/mod.rs`. |

A `/dev-story` worker launched while any of 1347 / 1348 / 1349 is still in-flight will rebase-trample on the 6.4 k-line `shop_auction/mod.rs` and the 5.5 k-line `hand/mod.rs`. The §"Parallelism" clause of each of those three spawn prompts explicitly says: *"If you detect a file conflict with another active worker, stop and report BLOCKED."* Wave 2 is the most parallel-conflict-prone Lane I row (spans lobby + hand + shop_auction) and must serialize after them.

**Unblocking sequence** (informational; out of this refresh's scope):

1. Land PROMPT 1347 → `origin/main` (or abandon).
2. Land PROMPT 1348 → `origin/main` (or abandon).
3. Land PROMPT 1349 → `origin/main` (or abandon).
4. (Optional) Re-run `/story-readiness` on this story to rebase line citations against the new main tip.
5. Launch a single Wave 2 worker (`work/s18-ui-interaction-state-migration-wave-2`).

PROMPT 1355 (this refresh) does NOT perform steps 1-5 and does NOT mark this story `READY`.

### Sibling parallelism (story-disjoint paperwork)

| Sibling | Parallel-safe? | Notes |
|---|---|---|
| Stories 020 / 021 / 022 / 023 / 024 / 026 / 027 (paperwork-only refreshes) | YES | Disjoint files. |
| Active PROMPTs 1178 / 1182 / 1183 | LANDED | Wave 2 prerequisites (cleared). |
| Active PROMPTs 1187 / 1188 | YES | Different surfaces / docs. |
| Active PROMPTs 1347 / 1348 / 1349 | **NO -- HARD COLLISION** | See "Active-Worker Collision Block" above. /dev-story must wait. |

Most parallel-conflict-prone Lane I row (spans lobby + hand + shop_auction).

## Acceptance Criteria

- [ ] **AC1 -- Lobby Confirm CTA token binding (L-03 resolved, least-regressive interpretation)**.
  The existing seven-state per-frame palette on `LobbyConfirmButton` (Disabled / Enabled / Hovered / Pressed / InFlight / Waiting / Confirmed -- shipped by PROMPT 1081 / PROMPT 1138, driven by `lobby_confirm_button_colors`) is **grandfathered**: Wave 2 does NOT migrate the full 7-state palette onto the canonical 4-state overlay-tint contract, because doing so would visibly regress shipped CTA UX. Wave 2 **does** require token-consistent semantics on the three states that DO map onto the canonical contract: `Hovered` MUST consume `HOVER_BG_TINT_ALPHA` + `HOVER_BORDER_ALPHA`; `Pressed` MUST consume `PRESSED_BG_TINT_ALPHA`; the `Disabled` band MUST remain fully token-derived (`DISABLED_BG_TINT_ALPHA` + `DISABLED_BORDER_ALPHA` + `DISABLED_TEXT_ALPHA`). The `InFlight` / `Waiting` / `Confirmed` bands are grandfathered as inline literals because they do not have canonical-token equivalents in `interaction_states.rs`. Any future broader migration is OUT OF SCOPE for Wave 2 and would require a fresh story.
- [ ] **AC2 -- Lobby Create / Join CTA token binding**.
  `LobbyCreateRoomButton` and `LobbyJoinRoomButton` (currently spawned with inline `Color::srgba(0.17, 0.18, 0.14, 0.95)` etc.) migrate to canonical 4-state overlay-tint binding (`Default` / `Hover` / `Pressed` / `Disabled`). No 7-state grandfather carve-out: these two are first-class candidates for the canonical contract.
- [ ] **AC3 -- Shop ready + refresh token binding (layered on PROMPT 1182)**.
  `ShopReadyButton` and `ShopRefreshButton` spawn sites CONTINUE to consume `primary_action_button_background_color()` / `primary_action_button_border_color()` (the PROMPT 1182 visible-chrome baseline that closed UI-1129-09); Wave 2 LAYERS `Hover` / `Pressed` / `Disabled` overlay tints via the change-detection system **on top of** those helpers. AC3 does NOT replace the helpers and does NOT remove the baseline chrome. Same applies to `DraftInitialReadyButton`, `DraftInitialObjectiveDismissButton`, `DraftInitialObjectiveRetrievalButton` if Wave 2 also touches them (verify scope at /dev-story time against PROMPT 1349 landed state).
- [ ] **AC4 -- Auction bid (3 variants) + pass token binding (S-05 baked-`?`-PNG cleanup, layered on PROMPT 1182)**.
  `AuctionBidButton` (3 increment variants) and `AuctionPassButton` spawn sites CONTINUE to consume `primary_action_button_background_color()` / `primary_action_button_border_color()`; Wave 2 LAYERS overlay tints on top via change-detection. S-05 framing (baked-`?` PNGs at bid spawn) was partially addressed by PROMPT 1182's visible-chrome additions; Wave 2 re-verifies S-05 against current main and closes any residual baked-PNG defect class observed at /dev-story time.
- [ ] **AC5 -- HandSubmitButton token binding (replaces stale PlacementSubmitButton citation)**.
  Bind by **symbol** `HandSubmitButton` (declaration in `client/src/ui/hand/mod.rs`, advisory line `~673`) and its spawn site (advisory line `~3972`). Line numbers are advisory; grep-by-symbol (`grep -n "pub struct HandSubmitButton" client/src/ui/hand/mod.rs`) is binding when an exact line drifts. The original story authored the name `PlacementSubmitButton` -- no such component exists on current main; that citation is REPLACED by `HandSubmitButton` (PROMPT 1355 refresh). The button also carries `HandSubmitInteractionState { Active, Inactive }` (advisory line `~752`) driven by an existing per-frame refresh path; Wave 2 must integrate the overlay-tint contract WITHOUT regressing this two-state semantic. Note: after PROMPT 1226 added auto-submit on the Placement→Resolution edge, the manual submit affordance still exists; its salience has changed but it remains a P1 surface deserving the 4-state token treatment.
- [ ] **AC6 -- Status chips do NOT carry `Interaction`**: `grep -rn "Interaction" client/src/ui/hud/` shows no pill spawn carries it. Lobby status banner + effective-timer readout same. (Currently SATISFIED on `origin/main@6e0453f8` per PROMPT 1350 §3; AC6 is a regression guard, not new work.)
- [ ] **AC7 -- Cursor changes on hover for P1 buttons; status chips unchanged.** Wires `CursorIcon::Pointer` (or equivalent) onto lobby Create / Join / Confirm, shop ready / refresh, auction bid (3) / pass, placement Submit.
- [ ] **AC8 -- Token module unchanged**: zero diff on `client/src/ui/design_tokens/interaction_states.rs`. Verifiable via `git diff` gate.
- [ ] **AC9 -- `tests/integration/ui_clean_pass/interaction_state_consumer_coverage_test.rs` (NEW)** asserts (i) `Interaction` is attached on each P1 surface; (ii) `BackgroundColor` at spawn reads from the documented `Default` token / helper (per-surface: PROMPT 1182 helper for shop / auction / draft; `interaction_states::*` Default for lobby Create / Join; per-band grandfather rule for lobby Confirm CTA per AC1); (iii) driving `Interaction` through `Hovered` and `Pressed` swaps `BackgroundColor` for each P1 surface, with overlay-tint values matching `HOVER_BG_TINT_ALPHA` and `PRESSED_BG_TINT_ALPHA` respectively (modulo the AC1 lobby Confirm CTA grandfather carve-out).
- [ ] **AC10 -- No new RGB literals at consumer spawn sites**, EXCEPT the AC1-grandfathered lobby Confirm CTA `InFlight` / `Waiting` / `Confirmed` bands which preserve their existing inline `Color::srgb(...)` literals (no new literals introduced; the bands themselves are pre-existing per PROMPT 1081 / PROMPT 1138 and not new work).
- [ ] **AC11 -- `liv-bevy-018` activated** for every `.rs` edit / read / review.
- [ ] **AC12 -- Cargo resource policy applied** per project standard env vars (worker contract).
- [ ] **AC13 -- No accept-risk closure**: `QA-COND-0005` Standard-tier accessibility not advanced; `QA-COND-0006` playtest validation not advanced; `PAW-TD-*-a` placeholder-art accept-risk preserved.
- [ ] **AC14 -- Sprint disposition preserved**: Sprint 18 `active`, stage `Polish` -- /dev-story does NOT flip sprint, stage, or close any other Sprint 18 row.
- [ ] **AC15 -- Worker branch scope contained**; slug `work/s18-ui-interaction-state-migration-wave-2`.

## Implementation Notes

### Owned files (Wave 2 /dev-story scope -- not this refresh)

| Path | Expected change |
|------|-----------------|
| `client/src/ui/lobby.rs` | AC1 (Confirm CTA Hover / Pressed / Disabled bands -- grandfather carve-out for the 4 non-canonical bands) + AC2 (Create / Join 4-state binding) + change-detection system. |
| `client/src/ui/shop_auction/mod.rs` | AC3 (ready / refresh: layered overlay tint on top of PROMPT 1182 helpers) + AC4 (bid (3) / pass: layered overlay tint). |
| `client/src/ui/hand/mod.rs` | AC5 (`HandSubmitButton` 4-state binding; preserve `HandSubmitInteractionState { Active, Inactive }` semantic). |
| `tests/integration/ui_clean_pass/interaction_state_consumer_coverage_test.rs` (NEW) | AC9. |
| `client/src/ui/mod.rs` | Register change-detection system in `Update` set if needed. **Verify at /dev-story time** whether existing per-frame refresh systems (e.g. `lobby_confirm_button_refresh`, `shop_ready_button_state_refresh`, `auction_bid_button_state_refresh`) already cover Wave 2's state-swap responsibility; if yes, this row is empty and the `mod.rs` entry can be dropped from the worker diff. |

### Forbidden files (Wave 2 /dev-story scope)

- `client/src/ui/design_tokens/interaction_states.rs` (consume-only; AC8 zero-diff guard).
- `client/src/ui/hud/**`, `client/src/ui/settings/**` (out of scope; status chips + settings panel are separate rows).
- Server, shared, ADRs, sprint / state / QA / gate-check / session-state files.

### Files forbidden by THIS refresh (PROMPT 1355) scope

- `client/**`, `server/**`, `shared/**`, `tests/**`
- `Cargo.toml`, `Cargo.lock`, `.cargo/**`, `.github/**`, `Trunk.toml`
- `production/sprint-status.yaml`, `production/session-state/**`, `production/sprints/**`, `production/stage.txt`, `production/qa/**`, `production/gate-checks/**`

This refresh writes ONLY `production/epics/ui-clean-pass/story-025-ui-interaction-state-migration-wave-2.md` and `reports/PROMPT-1355-s18-interaction-state-migration-story-refresh.md`.

## Worker Contract (for the future /dev-story worker, NOT this refresh)

1. **Verify prerequisites** (Wave 1: 1178 / 1182 / 1183 landed). All cleared as of PROMPT 1350 §1. BLOCK + relay if any regress.
2. **Verify active-worker collision gate** (Wave 2 entry block): PROMPT 1347 / 1348 / 1349 either landed on `origin/main` OR formally abandoned. If any is still in-flight on the worker branch state, **BLOCK + relay**; do NOT touch `client/src/ui/shop_auction/mod.rs` or `client/src/ui/hand/mod.rs` until they clear.
3. Worktree slug `work/s18-ui-interaction-state-migration-wave-2`.
4. Read story + PROMPT 1180 §2 RC-4 + §5 C-4 + §6 Lane I + PROMPT 1350 readiness audit + PROMPT 1355 refresh report.
5. Activate `liv-bevy-018`.
6. Cargo resource policy env vars.
7. Targeted tests only.
8. Push worker branch only; do NOT push `main`.
9. Honour the AC1 grandfather carve-out: do NOT migrate the lobby Confirm CTA's 7-state palette wholesale; bind only the Hover / Pressed / Disabled bands to the canonical contract and preserve the InFlight / Waiting / Confirmed bands as-is.
10. Honour the AC3 / AC4 layering rule: build on top of PROMPT 1182's `primary_action_button_*` helpers; do NOT replace them; do NOT regress the visible baseline chrome.
11. Honour the AC5 component-name correction: bind `HandSubmitButton` (not `PlacementSubmitButton`); preserve the `HandSubmitInteractionState { Active, Inactive }` two-state semantic.

## Refresh Trail

| Prompt | Date | Refresh scope |
|---|---|---|
| PROMPT 1189 | 2026-05-18 | Initial authoring on `origin/main@efb698e`. |
| PROMPT 1350 | 2026-05-19 | Read-only readiness audit on `origin/main@1e9548f`; verdict `BLOCKED`; report at `reports/PROMPT-1350-s18-ui-interaction-state-migration-wave-2-readiness.md`. |
| PROMPT 1355 | 2026-05-19 | Story body refresh on `origin/main@6e0453f8`; folds PROMPT 1350 findings (Problem Class, AC1 grandfather, AC3 / AC4 layering, AC5 `HandSubmitButton`, active-worker collision block). Story is NOT marked `READY`; /dev-story remains blocked until PROMPT 1347 / 1348 / 1349 land or are abandoned. Report at `reports/PROMPT-1355-s18-interaction-state-migration-story-refresh.md`. |
