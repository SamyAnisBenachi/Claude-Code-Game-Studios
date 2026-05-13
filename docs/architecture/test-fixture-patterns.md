# Test Fixture Patterns

> Canonical patterns for building `MinimalPlugins`-based fixtures that exercise
> `client::ui::hand::HandUiPlugin` (and adjacent presentation plugins) without
> needing the full `DefaultPlugins`/render stack.
>
> **Stewardship**: Append new patterns here when you ship a fixture-helper that
> closes a recurring gap. Authored under Sprint 11 story
> `S11-TD-FIXTURE-HAND-UI-ONENTER-001`
> (`production/epics/playable-client/story-011-hand-ui-onenter-fixture-repair.md`).

## Why this doc exists

`MinimalPlugins`-based partial-App fixtures are the cheapest way to write
ECS-level tests against `client/` UI systems — no window, no renderer, no asset
server. But these fixtures must explicitly satisfy every prerequisite that
production-side plugin composition would normally hand to the system under
test. When a prerequisite is missing, the system typically *silently skips*
rather than panicking, and the failure surfaces downstream as a cryptic
"entity not found" / "resource not found" panic in the assertion code.

This doc lists each recurring gap and the canonical helper that closes it, so
future fixture authors don't have to rediscover the cascade.

---

## Pattern: drive `OnEnter(ClientState::InSession)` end-to-end

### When you need it

Any `MinimalPlugins` fixture that:

- adds `HandUiPlugin` (or another plugin that registers a `spawn_*` system on
  `OnEnter(ClientState::InSession)`), AND
- needs the spawn to *actually* fire (e.g., the test asserts on
  `HandUiEntities` / `FanSlotIndex` / `HandCardFrame` / chrome children).

### What goes wrong without the helper

`spawn_hand_ui` (`client/src/ui/hand/mod.rs`) early-returns on
`Option<Res<PlaceholderAssets>>::None`. In production, `AssetWiringPlugin`'s
`insert_placeholder_assets` system runs on `OnEnter(InSession)` and inserts the
resource via `AssetServer`. `HandUiPlugin` schedules
`spawn_hand_ui.after(insert_placeholder_assets)` so the ordering is correct.

A `MinimalPlugins` fixture that omits `AssetPlugin` / `AssetServer` (and does
not insert `PlaceholderAssets` directly) sees `insert_placeholder_assets`
silently skip on missing `Res<AssetServer>` and `spawn_hand_ui` early-return.
The state transition completes "cleanly" and the test fails downstream when
the spawn-dependent queries find nothing.

### Helper

```rust
use client::asset_wiring::enter_in_session_via_fixture;
```

Signature:

```rust
pub fn enter_in_session_via_fixture(app: &mut bevy::prelude::App);
```

Behavior:

1. Inserts `placeholder_assets_for_tests()` into the world if absent.
2. Sets `NextState::<ClientState>::Pending(ClientState::InSession)`.
3. Runs `app.update()` twice — the first cycle applies the state transition
   and runs `OnEnter(InSession)` systems (which queue spawn commands); the
   second cycle flushes those deferred commands so downstream queries
   resolve in the same tick as the assertions.

### Pre-conditions the caller must satisfy

| Plugin / Resource | Why |
|---|---|
| `MinimalPlugins` | Time, Hierarchy, schedule infrastructure |
| `StatesPlugin` + `init_state::<ClientState>()` | `OnEnter(InSession)` schedule registration |
| `HandUiPlugin` (or the plugin that registers `spawn_hand_ui`) | The actual `OnEnter(InSession)` system under test |

The helper does *not* add these plugins itself — fixtures vary in which
adjacent plugins they bundle (e.g., `ShopAuctionUiPlugin`, `TweeningPlugin`)
and we don't want to force a one-size-fits-all bundle.

### Minimal example

```rust
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::asset_wiring::enter_in_session_via_fixture;
use client::state::ClientState;
use client::ui::hand::{HandUiEntities, HandUiPlugin};

fn fixture() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(HandUiPlugin);
    enter_in_session_via_fixture(&mut app);
    app
}

#[test]
fn test_hand_ui_entities_exist_after_in_session_transition() {
    let app = fixture();
    let _entities = app.world().resource::<HandUiEntities>();
}
```

### Side effects and limits

- The helper flips `ClientState::Lobby -> ClientState::InSession`. It does not
  set a `RoundPhase`; callers that need the hand UI in
  `RoundPhase::Placement` (or any other phase) must set
  `CurrentClientPhase`/`ClientPhaseView` themselves and call `app.update()`
  once more so phase-transition systems observe the new phase.
- The helper inserts `placeholder_assets_for_tests()` — every `Handle<Image>`
  is `Handle::default()`. Tests that assert on a *non-default* image handle
  (PAW-002-e style assertions) need the
  `AssetPlugin + insert_placeholder_assets` pattern instead (see
  `tests/integration/presentation/hand_ui_asset_wiring_test.rs`).

### Related precedent

Mirrors the `placeholder_assets_for_tests()` precedent that closed Layer 3 of
the S10-TD-001 cascade (see
`production/epics/playable-client/story-009-test-fixture-cascade-fail-repair.md`).
This helper closes the residual gap that smoke retry-7 surfaced after Layers
1-3 landed.

---

## Pattern history

| Date | Story | Pattern |
|------|-------|---------|
| 2026-05-13 | `S11-TD-FIXTURE-HAND-UI-ONENTER-001` | `enter_in_session_via_fixture` |
