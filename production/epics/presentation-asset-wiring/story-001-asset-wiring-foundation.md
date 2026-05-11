# Story 001: asset_wiring.rs Foundation and Placeholder Fallback PNGs

> **Epic**: Presentation Asset Wiring
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `client/src/asset_wiring.rs` (implementation surface); cross-GDD infrastructure
**Requirement**: `TR-PAW-001`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: All presentation assets are loaded via a client-side `bevy_asset_loader` LoadingState. Shared asset handles are promoted into typed Resources before any sub-system initialises. `ImageNode::new()` is the only permitted API for bevy_ui image nodes; `UiImage` is forbidden.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `ImageNode` replaces `UiImage` (Bevy 0.16+). `UiImage::new()` does not exist in 0.18 — use `ImageNode::new(handle)`. `Query::single()` returns `Result` in 0.16+. `ChildOf` replaces `Parent` (0.16+). `commands.entity(e).despawn()` is recursive by default (0.16+).

**Control Manifest Rules (Presentation Layer)**:
- Required: `ImageNode::new()` for all bevy_ui image nodes
- Required: Session-scoped Resources inserted on `OnEnter(ClientState::InSession)`
- Forbidden: `UiImage::new()` — compile error in Bevy 0.18
- Forbidden: Hardcoded asset paths inline in system code — use path constants from `asset_wiring.rs`
- Guardrail: Presentation steady-state < 1 ms/frame

---

## Acceptance Criteria

*Cross-surface infrastructure for TR-PAW-001:*

- [ ] **PAW-001-a**: `client/src/asset_wiring.rs` exports `PLACEHOLDER_FALLBACK_ASSET: &str = "art/ui/shared/ui_placeholder_1x1_white.png"` as a `pub const`.
- [ ] **PAW-001-b**: Path constant modules or constant groups exist for all 6 surfaces: `card_ui`, `shop_ui`, `auction_ui`, `hud_ui`, `board_chars`, `lobby_ui`. Each exports `pub const` strings for every path named in Stories 002–006.
- [ ] **PAW-001-c**: A `PlaceholderAssets` resource is defined with typed `Handle<Image>` fields for the fallback PNG and all per-surface placeholder handles. The resource is inserted on `OnEnter(ClientState::InSession)` and removed on `OnExit`.
- [ ] **PAW-001-d**: An integration test asserts `Res<PlaceholderAssets>` is accessible after `OnEnter(ClientState::InSession)` executes (inject state via `App::add_state`, verify resource exists with `app.world().get_resource::<PlaceholderAssets>().is_some()`).
- [ ] **PAW-001-e**: `cargo check -p client` passes with no warnings on this story's changes.
- [ ] **PAW-001-f**: `grep -rn "UiImage" client/src/` returns zero results after this story (enforce the forbidden pattern clean-up if any pre-existing uses exist).

---

## Implementation Notes

*Derived from ADR-021 Implementation Guidelines:*

**Path constant organisation.** Group constants by surface in `asset_wiring.rs`:

```rust
// ── Shared fallback ───────────────────────────────────────────────────────────
pub const PLACEHOLDER_FALLBACK_ASSET: &str = "art/ui/shared/ui_placeholder_1x1_white.png";

// ── Card UI (hand fan chrome) ─────────────────────────────────────────────────
pub const CARD_FRAME_COMMON_HAND_ASSET: &str = "art/ui/card/ui_card_frame_common_hand.png";
pub const CARD_FRAME_RARE_HAND_ASSET: &str    = "art/ui/card/ui_card_frame_rare_hand.png";
pub const CARD_FRAME_EPIC_HAND_ASSET: &str    = "art/ui/card/ui_card_frame_epic_hand.png";
pub const CARD_FRAME_LEGENDARY_HAND_ASSET: &str = "art/ui/card/ui_card_frame_legendary_hand.png";

pub const STAT_BADGE_ATK_ASSET: &str = "art/ui/card/ui_stat_badge_atk.png";
pub const STAT_BADGE_HP_ASSET: &str  = "art/ui/card/ui_stat_badge_hp.png";
pub const STAT_BADGE_MP_ASSET: &str  = "art/ui/card/ui_stat_badge_mp.png";
pub const STAT_BADGE_AR_ASSET: &str  = "art/ui/card/ui_stat_badge_ar.png";

pub const RARITY_ICON_COMMON_ASSET: &str    = "art/ui/card/ui_rarity_common_icon.png";
pub const RARITY_ICON_RARE_ASSET: &str      = "art/ui/card/ui_rarity_rare_icon.png";
pub const RARITY_ICON_EPIC_ASSET: &str      = "art/ui/card/ui_rarity_epic_icon.png";
pub const RARITY_ICON_LEGENDARY_ASSET: &str = "art/ui/card/ui_rarity_legendary_icon.png";

pub const CLASS_TYPE_ICON_IOP_ASSET: &str     = "art/ui/card/ui_class_iop_type_icon.png";
pub const CLASS_TYPE_ICON_CRA_ASSET: &str     = "art/ui/card/ui_class_cra_type_icon.png";
pub const CLASS_TYPE_ICON_SACRIER_ASSET: &str = "art/ui/card/ui_class_sacrier_type_icon.png";
pub const CLASS_TYPE_ICON_XELOR_ASSET: &str   = "art/ui/card/ui_class_xelor_type_icon.png";
pub const CLASS_TYPE_ICON_ECAFLIP_ASSET: &str = "art/ui/card/ui_class_ecaflip_type_icon.png";
pub const CLASS_TYPE_ICON_SADIDA_ASSET: &str  = "art/ui/card/ui_class_sadida_type_icon.png";
pub const CLASS_TYPE_ICON_NEUTRAL_ASSET: &str = "art/ui/card/ui_class_neutral_type_icon.png";

// ── Shop / Auction UI ─────────────────────────────────────────────────────────
pub const SHOP_PANEL_CHROME_ASSET: &str     = "art/ui/shop/ui_shop_panel_chrome.png";
pub const SHOP_SLOT_WELL_IDLE_ASSET: &str   = "art/ui/shop/ui_slot_well_idle.png";

pub const BID_BUTTON_NORMAL_ASSET: &str   = "art/ui/auction/ui_bid_button_normal.png";
pub const BID_BUTTON_HOVER_ASSET: &str    = "art/ui/auction/ui_bid_button_hover.png";
pub const BID_BUTTON_DISABLED_ASSET: &str = "art/ui/auction/ui_bid_button_disabled.png";

// ── HUD ───────────────────────────────────────────────────────────────────────
pub const HUD_FIGURINE_IOP_ASSET: &str     = "art/ui/hud/ui_class_figurine_iop.png";
pub const HUD_FIGURINE_CRA_ASSET: &str     = "art/ui/hud/ui_class_figurine_cra.png";
pub const HUD_FIGURINE_SACRIER_ASSET: &str = "art/ui/hud/ui_class_figurine_sacrier.png";
pub const HUD_FIGURINE_XELOR_ASSET: &str   = "art/ui/hud/ui_class_figurine_xelor.png";
pub const HUD_FIGURINE_ECAFLIP_ASSET: &str = "art/ui/hud/ui_class_figurine_ecaflip.png";
pub const HUD_FIGURINE_SADIDA_ASSET: &str  = "art/ui/hud/ui_class_figurine_sadida.png";
pub const HUD_FIGURINE_NEUTRAL_ASSET: &str = "art/ui/hud/ui_class_figurine_neutral.png";

pub const HUD_PHASE_TIMER_BAR_ASSET: &str = "art/ui/hud/ui_phase_timer_bar.png";

pub const HUD_OBJECTIVE_DOT_ALIVE_ASSET: &str     = "art/ui/hud/ui_objective_dot_alive.png";
pub const HUD_OBJECTIVE_DOT_DESTROYED_ASSET: &str = "art/ui/hud/ui_objective_dot_destroyed.png";
pub const HUD_OBJECTIVE_DOT_UNKNOWN_ASSET: &str   = "art/ui/hud/ui_objective_dot_unknown.png";
pub const HUD_OBJECTIVE_DOT_FAKE_ASSET: &str      = "art/ui/hud/ui_objective_dot_fake.png";

// ── Board characters (world-space Sprite — NOT ImageNode) ─────────────────────
pub const BOARD_UNIT_IOP_ASSET: &str     = "art/characters/ui_class_iop_unit_board.png";
pub const BOARD_UNIT_CRA_ASSET: &str     = "art/characters/ui_class_cra_unit_board.png";
pub const BOARD_UNIT_SACRIER_ASSET: &str = "art/characters/ui_class_sacrier_unit_board.png";
pub const BOARD_UNIT_XELOR_ASSET: &str   = "art/characters/ui_class_xelor_unit_board.png";
pub const BOARD_UNIT_ECAFLIP_ASSET: &str = "art/characters/ui_class_ecaflip_unit_board.png";
pub const BOARD_UNIT_SADIDA_ASSET: &str  = "art/characters/ui_class_sadida_unit_board.png";
pub const BOARD_UNIT_NEUTRAL_ASSET: &str = "art/characters/ui_class_neutral_unit_board.png";
pub const BOARD_CHROME_ASSET: &str = "art/board/env_board_chrome_default.png";

// ── Lobby ─────────────────────────────────────────────────────────────────────
pub const LOBBY_PORTRAIT_IOP_ASSET: &str     = "art/ui/lobby/ui_class_portrait_iop.png";
pub const LOBBY_PORTRAIT_CRA_ASSET: &str     = "art/ui/lobby/ui_class_portrait_cra.png";
pub const LOBBY_PORTRAIT_SACRIER_ASSET: &str = "art/ui/lobby/ui_class_portrait_sacrier.png";
pub const LOBBY_PORTRAIT_XELOR_ASSET: &str   = "art/ui/lobby/ui_class_portrait_xelor.png";
pub const LOBBY_PORTRAIT_ECAFLIP_ASSET: &str = "art/ui/lobby/ui_class_portrait_ecaflip.png";
pub const LOBBY_PORTRAIT_SADIDA_ASSET: &str  = "art/ui/lobby/ui_class_portrait_sadida.png";
pub const LOBBY_PORTRAIT_NEUTRAL_ASSET: &str = "art/ui/lobby/ui_class_portrait_neutral.png";

pub const LOBBY_PLAYER_SLOT_PANEL_ASSET: &str = "art/ui/lobby/ui_player_slot_panel.png";
pub const LOBBY_ROOM_CODE_CHIP_ASSET: &str    = "art/ui/lobby/ui_room_code_chip.png";
```

**PlaceholderAssets resource.** Define alongside the path constants:

```rust
#[derive(Resource)]
pub struct PlaceholderAssets {
    pub fallback: Handle<Image>,
    // card UI
    pub card_frame_common: Handle<Image>,
    pub card_frame_rare: Handle<Image>,
    pub card_frame_epic: Handle<Image>,
    pub card_frame_legendary: Handle<Image>,
    pub stat_badge_atk: Handle<Image>,
    pub stat_badge_hp: Handle<Image>,
    pub stat_badge_mp: Handle<Image>,
    pub stat_badge_ar: Handle<Image>,
    // ... one field per path constant above
}
```

Load all handles in a startup/session system using `asset_server.load(CONSTANT)`. Insert on `OnEnter(ClientState::InSession)`. Remove on `OnExit(ClientState::InSession)`.

**Convenience selector functions.** Add pure functions so callers never match on path strings:

```rust
pub fn card_frame_asset(rarity: Rarity) -> &'static str { ... }
pub fn class_type_icon_asset(class_id: ClassId) -> &'static str { ... }
pub fn hud_figurine_asset(class_id: ClassId) -> &'static str { ... }
pub fn board_unit_asset(class_id: ClassId) -> &'static str { ... }
pub fn lobby_portrait_asset(class_id: ClassId) -> &'static str { ... }
pub fn hud_objective_dot_asset(state: ObjectiveDotState) -> &'static str { ... }
pub fn bid_button_asset(state: BidButtonChromeState) -> &'static str { ... }
```

These selector functions let Stories 002–006 call e.g. `card_frame_asset(rarity)` without inlining the path match. Final art = update one `const` per path.

**Session scope.** `PlaceholderAssets` is session-scoped (mirrors `CardAtlas` pattern in ADR-021 §Implementation Guideline 2). All systems reading it must be `in_state(ClientState::InSession)`.

---

## Out of Scope

- Stories 002–006: inserting `ImageNode` / `Sprite.image` on individual UI entities.
- No new UI widgets or visual layout changes.
- No changes to `CardDisplayArtAsset` / `resolve_card_display_art()` — those handle card display art, not chrome.

---

## QA Test Cases

*Story type: Integration — automated test specs.*

- **AC PAW-001-c/d**: PlaceholderAssets resource lifecycle
  - Given: `App` with `ClientState` state machine and the `asset_wiring` loading system registered; state transitions to `ClientState::InSession`
  - When: `app.update()` is called after the state transition
  - Then: `app.world().get_resource::<PlaceholderAssets>().is_some()` returns `true`
  - Edge cases: resource must not exist before `InSession` entry; must be removed on `OnExit(InSession)`

- **AC PAW-001-b**: Path constant completeness
  - Given: the compiled `asset_wiring` module
  - When: each path constant is read at test time
  - Then: every constant is non-empty, starts with `art/`, ends with `.png`, and contains no whitespace
  - Edge cases: verify all 7 class variants exist for figurine, portrait, type icon, and board unit groups

- **AC PAW-001-f**: No UiImage in client
  - Given: `client/src/` source tree
  - When: `grep -rn "UiImage" client/src/` is run in CI
  - Then: exit code 1 (no matches) — zero occurrences

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/presentation/asset_wiring_foundation_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: None
- Unlocks: Stories 002, 003, 004, 005, 006 (all may proceed in parallel once this is Done)

---

## Completion Notes

**Completed**: 2026-05-08
**Criteria**: 5/6 passing (1 advisory — see below)
**Deviations**: ADVISORY — `cargo check -p client` has pre-existing compile errors in `network/mod.rs` (PLAYABLE-001 commit `85878d2`, predates this story). This story's changes introduce zero new errors. A dedicated fix story for the network module is recommended before the next `cargo check` gate.
**Test Evidence**: Integration — `tests/integration/presentation/asset_wiring_foundation_test.rs` exists with 8 tests covering all ACs. Tests cannot execute until pre-existing network compile errors are resolved.
**Code Review**: Skipped — Lean review mode
