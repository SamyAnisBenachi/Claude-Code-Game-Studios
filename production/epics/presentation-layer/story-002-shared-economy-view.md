# Story 002: Shared Economy View

> **Epic**: Presentation Layer
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-PRES-001`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` - read fresh at review time)*

**ADR Governing Implementation**: ADR-021: Presentation Layer Architecture; ADR-002: Client-Server Authority; ADR-008: Lightyear Channel Config; ADR-019: Economy Resource Architecture
**ADR Decision Summary**: Presentation consumes server-authoritative S2C data and exposes shared client-side view resources to sub-plugins. `S2CGoldUpdate` and reconnect snapshots are the authoritative source for the local player's current/reserve mana; Hand UI must read a shared resource rather than maintaining its own partial mirror.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: This story drains Lightyear S2C economy/snapshot messages in `PresentationSet::MessageDrain` and writes Bevy resources. Use `liv-bevy-018` and `liv-bevy-lightyear` before implementation.

**Control Manifest Rules (Presentation layer)**:
- Required: `S2CGoldUpdate` is drained by one shared presentation system.
- Required: `PlayerEconomyView` is the client-side read model for own `gold`, `current_mana`, `reserve_mana`, and `mana_cap`.
- Required: Hand UI, HUD, and Shop/Auction UI read `Res<PlayerEconomyView>` rather than draining `S2CGoldUpdate` independently.
- Forbidden: Do not mutate economy view from local input; only S2C/snapshot data can update it.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Rule 10 and Rule 13, scoped to this shared presentation prerequisite:*

- [x] **PRES-002 / TR-PRES-001 - resource shape**: `client/src/presentation/shared/economy_view.rs` (or equivalent shared presentation module) defines `PlayerEconomyView` with at least:
  - `gold: u32`
  - `current_mana: u32`
  - `reserve_mana: u32`
  - `mana_cap: u8`
  - `last_update_source` or equivalent test-observable marker for S2C vs snapshot updates

- [x] **PRES-002 / TR-PRES-001 - S2CGoldUpdate drain**: Exactly one production client system drains `MessageReceiver<S2CGoldUpdate>` and updates `PlayerEconomyView` in `PresentationSet::MessageDrain`.

- [x] **PRES-002 / TR-PRES-001 - reconnect snapshot seed**: When `S2CGameSnapshot` is processed for the local player, `PlayerEconomyView` is seeded from the local `PlayerSnapshot` before Hand UI submit validation can run.

- [x] **PRES-002 / TR-PRES-001 - no local optimism**: Local purchase, activation, reserve-strip, or submit input must not mutate `PlayerEconomyView`; only authoritative S2C/snapshot data may change it.

- [x] **PRES-002 / TR-PRES-001 - Hand UI consumption path**: Hand UI submit pre-validation and reserve strip controls read `Res<PlayerEconomyView>` for current/reserve mana limits. They must not drain `S2CGoldUpdate` directly.

- [x] Grep guard: production client source contains exactly one `MessageReceiver<S2CGoldUpdate>` drain after this story.

---

## Implementation Notes

*Derived from ADR-021 PresentationSet ordering and ADR-002 client authority:*

Recommended resource:

```rust
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct PlayerEconomyView {
    pub gold: u32,
    pub current_mana: u32,
    pub reserve_mana: u32,
    pub mana_cap: u8,
    pub initialized: bool,
}
```

Register the drain in `PresentationSet::MessageDrain`, after the shared phase sink has already updated `CurrentClientPhase` for the frame. `StateSync` systems then update text/buttons from `PlayerEconomyView`.

`S2CGoldBroadcast` is public gold/reservation data and may be represented by a separate shared resource if needed by HUD or Shop/Auction UI. Do not overload `PlayerEconomyView` with opponent-private or public-auction concerns unless the implementation story explicitly expands scope.

---

## Out of Scope

- Protocol payload split (`NP-005`)
- Server authority validation (`BLS-011`)
- Economy explicit split API (`ECO-007`)
- HAND-UI-010 validation button/error behavior
- HUD/Shop visual polish beyond reading the shared resource

---

## QA Test Cases

- **S2CGoldUpdate updates shared view**
  - Given: a client app with `PlayerEconomyView`
  - When: `S2CGoldUpdate { gold: 4, current_mana: 3, reserve_mana: 2, mana_cap: 10 }` is delivered
  - Then: the resource matches those values after one `App::update()`.

- **Snapshot initializes before validation**
  - Given: reconnect snapshot contains the local player's economy values
  - When: snapshot handling runs
  - Then: `PlayerEconomyView.initialized == true` and values match the local snapshot entry.

- **No local mutation**
  - Given: reserve strip input changes local staged placement split
  - When: the input system runs without any S2C economy message
  - Then: `PlayerEconomyView` values remain unchanged.

- **Single drain grep**
  - Given: production client source
  - When: `rg "MessageReceiver<S2CGoldUpdate>" client/src` runs
  - Then: exactly one production occurrence drains the message.

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/integration/presentation/shared_economy_view_test.rs`
- Grep evidence in `production/qa/evidence/shared-economy-view-evidence.md`

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: Presentation Layer Story 001 complete; `S2CGoldUpdate` and `S2CGameSnapshot` protocol types available.
- Unlocks: `production/epics/hand-ui/story-010-submit-prevalidation.md` client-economy-view prerequisite.

## Completion Notes

**Completed**: 2026-05-05
**Verdict**: COMPLETE
**Criteria**: 6/6 passing; resource shape, single `S2CGoldUpdate` drain, reconnect snapshot seed, no local optimism, Hand UI consumption path, and grep guard verified.
**Test Evidence**: `tests/integration/presentation/shared_economy_view_test.rs` exists and `cargo test -p client --test shared_economy_view_test` passed 3/3. Required grep evidence exists at `production/qa/evidence/shared-economy-view-evidence.md`.
**Regression Evidence**: Affected HUD/Hand/PRES bundle passed: `hud_gold_mana_display_test` 6/6, `same_tick_tie_break_test` 3/3, `reconnect_snapshot_rebuild_test` 3/3, `hand_ui_reserve_mana_strip_test` 3/3, `hand_ui_draft_initial_grid_test` 5/5, and `hud_numeric_tween_animation_test` 4/4. `cargo check -p client`, `cargo fmt -p client -- --check`, `rg -n "MessageReceiver<S2CGoldUpdate>" client/src`, and `git diff --check` passed; the grep returned exactly one production occurrence in `client/src/presentation/shared/economy_view.rs`.
**Verification**: Current `main` includes worker commit `58afb3b2ba321bbea4bd5331a13bc228f952e6ed`, main integration commit `8587fa9`, and integration fix commit `e14feb6`. `PlayerEconomyView` mirrors own gold/current mana/reserve mana/mana cap from authoritative S2C gold updates and local reconnect snapshots; Hand UI and HUD read the shared resource instead of draining `S2CGoldUpdate` independently.
**Deviations**: None. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
**Tech Debt**: None logged.
**Sprint Status**: Unchanged; no matching PRES-002 row exists in `production/sprint-status.yaml`.
**Next Recommended**: PRES-002 no longer blocks HAND-UI-010. HAND-UI-010 still depends on BLS-011 (`production/epics/board-lane-system/story-011-placement-submit-authority-validation.md`) before readiness can be rechecked.
