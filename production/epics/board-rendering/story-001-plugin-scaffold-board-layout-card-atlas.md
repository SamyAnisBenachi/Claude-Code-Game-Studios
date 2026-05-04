# Story 001: Plugin Scaffold, BoardLayout, and CardAtlas

> **Epic**: Board Rendering
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-002`
**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

`BoardRenderingPlugin` owns the session-scoped `BoardLayout` and `CardAtlas` resources consumed by Hand UI, HUD, and Card Animations. Current client code has `BoardLayout` in `client/src/ui/shared.rs`, but its `cell_to_world` returns `Option<Vec2>`; the GDD requires a canonical coordinate authority with assertion-backed invalid input handling. This story establishes the plugin/resource contract before any visual entity spawning.

Shared ADR-021 infrastructure (`PresentationPlugin`, `PresentationSet`, `phase_sink_system`, and the canonical `CurrentClientPhase` path) is owned by [Presentation Layer Story 001](../presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md). Do not implement those shared surfaces here.

## Acceptance Criteria

- [ ] `BoardRenderingPlugin` can be registered in a minimal client `App` without panic.
- [ ] `BoardLayout` is inserted on `OnEnter(ClientState::InSession)` and removed on `OnExit`.
- [ ] `CardAtlas` is inserted on session entry as a shared resource with `Handle<Image>` plus `Handle<TextureAtlasLayout>`.
- [ ] `BoardLayout::cell_to_world(1, 1)` returns `board_origin`.
- [ ] `BoardLayout::cell_to_world(1, 2)` returns `board_origin + Vec2::new(cell_width, 0.0)`.
- [ ] Invalid lane/cell values assert in release builds through the agreed GDD path: valid lanes are `1..=5`, valid cells are `1..=8`, and invalid values such as `lane=0`, `lane=6`, `cell=0`, or `cell=9` are not silently ignored.
- [ ] No `MessageReceiver<S2CPhaseChanged>` is registered by this plugin; phase is read through `Res<CurrentClientPhase>` only.
- [ ] This story relies on `PresentationPlugin`, `PresentationSet`, and `phase_sink_system` from Presentation Layer Story 001 rather than defining them locally.

## Implementation Notes

- Follow ADR-021 plugin order: `CardAnimationsPlugin`, `BoardRenderingPlugin`, `HandUiPlugin`, `HudPlugin`, `ShopAuctionUiPlugin`.
- Use Bevy 0.18 Required Components API. Do not introduce `*Bundle` types.
- `CardAtlas` must use the Bevy 0.18 pattern: `Handle<Image>` plus `Handle<TextureAtlasLayout>` inside `Sprite.texture_atlas`.
- Keep `BoardLayout` as the single coordinate authority. Do not hardcode lane/cell positions in later stories.
- ADR-021 performance budget: presentation steady-state target is `< 1 ms` per frame and phase-boundary spikes target `< 3 ms`. This scaffold is expected to have no per-frame rendering impact beyond session-entry resource insertion/removal.
- If retaining the existing `client/src/ui/shared.rs` type, update its contract in this story so all current consumers compile against the canonical API.

## Out of Scope

- `PresentationPlugin`, `PresentationSet`, `phase_sink_system`, and shared `CurrentClientPhase` path ownership (Presentation Layer Story 001).
- Board grid/camera spawning (Story 002).
- Snapshot-driven unit and objective spawning (Story 003).
- Ghost previews and placement reveal animation (Stories 004 and 005).

## QA Test Cases

- **BR-2 layout formula**
  - Given: a `BoardLayout` with `board_origin = Vec2::new(-128.0, 160.0)`, `cell_width = 64.0`, `lane_height = 80.0`
  - When: `cell_to_world(1, 1)`, `cell_to_world(1, 2)`, and `cell_to_world(2, 1)` are called
  - Then: positions match the GDD formula exactly.

- **Resource lifecycle**
  - Given: `App::new()` with `BoardRenderingPlugin`
  - When: `ClientState::InSession` is entered and then exited
  - Then: `BoardLayout` and `CardAtlas` exist during the session and are absent after exit.

- **Phase drain guard**
  - Given: client source after implementation
  - When: `rg "MessageReceiver<S2CPhaseChanged>" client/src`
  - Then: only the shared phase sink owns the Lightyear receiver.

## Test Evidence

**Required evidence**:
- Logic: `tests/unit/board_rendering/plugin_scaffold_test.rs`
- CI grep: `MessageReceiver<S2CPhaseChanged>` appears only in the shared phase sink.

**Status**: [x] Verified on 2026-05-04

## Dependencies

- Depends on: `production/epics/presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md` - shared `PresentationPlugin`, `PresentationSet`, `phase_sink_system`, and `CurrentClientPhase` path must be complete before BoardRenderingPlugin is implemented or registered.
- Unlocks: Stories 002, 003, 004, 005, 006, 007, 008, 009, 010.

## Completion Notes

**Completed**: 2026-05-04
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 8/8 passing.
**Verification**: Current `main` includes integrated commit `b5abcd5`; `BoardRenderingPlugin` registers in a minimal client app, inserts `BoardLayout` and `CardAtlas` on `OnEnter(ClientState::InSession)`, removes both on `OnExit`, and is registered through the completed `PresentationPlugin` order slot. `BoardLayout::cell_to_world` returns `board_origin` for `(1, 1)`, `board_origin + Vec2::new(cell_width, 0.0)` for `(1, 2)`, applies lane Y offsets, and uses `assert!` for invalid lanes/cells so failures are not debug-only. `CardAtlas` is a shared Resource containing `Handle<Image>` and `Handle<TextureAtlasLayout>`.
**Test Evidence**: `tests/unit/board_rendering/plugin_scaffold_test.rs`; `cargo test -p client --test board_rendering_plugin_scaffold_test` passed 9/9. Regression checks also passed: `cargo test -p client --test hand_ui_placement_drag_highlights_test --test card_animations_placement_reveal_test` passed 14/14; `cargo check -p client` passed; `git diff --check` passed.
**Deviations**: Advisory only - `TR-BR-002` in `docs/architecture/tr-registry.yaml` says `cell_to_world(lane, cell) -> Vec3`, while the current Board Rendering GDD F1 formula and ADR-021 specify `Vec2`. Implementation follows the current GDD/ADR and existing Presentation consumers.
**Scope**: No scope creep found into snapshots, unit spawning, new tweens, resolution animation queue, shop/auction UI, or visual asset generation. Hand UI and Card Animations edits were limited to adapting existing consumers/tests to the canonical `BoardLayout::cell_to_world` return contract.
**Code Review**: Lean mode; QL-TEST-COVERAGE and LP-CODE-REVIEW gates skipped per `/story-done` review-mode rules.
