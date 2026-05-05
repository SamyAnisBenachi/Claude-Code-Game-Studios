# Story 009: Status Icons and Co-Occupancy Visuals

> **Epic**: Board Rendering
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Visual/Feel
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-006`, `TR-BR-007`, with supporting trace to `TR-KW-010` and `TR-NP-006`
**ADR Governing Implementation**: [ADR-018: Keyword System](../../../docs/architecture/adr-018-keyword-system.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

This story is the independently launchable visual scope split out of the original combined BOARD-009. It covers persistent status icon rendering, status overflow badges, status ordering, OUTNUMBERED per-unit visual state, and co-occupancy offsets. Spawn range source and highlight updates are split out because the current docs and code do not agree on the live source: the Board Rendering GDD names a replicated `SpawnRange` component, the Network Protocol GDD names a `SpawnRangeChanged` resolution-log event plus `PlayerSnapshot.spawn_range_cells`, and current code only exposes snapshot `spawn_range_cells` while server-side live state is split between `SpawnRangeState` and `ObjectiveCounters`.

## Requirement Trace

- `TR-BR-006`: persistent state indicator glyphs with Tier-1 priority ordering.
- `TR-BR-007`: OUTNUMBERED indicator is per-unit, not per-lane. `OQ-KS5` is closed in `design/gdd/keyword-system.md`.
- `TR-KW-010`: OUTNUMBERED uses global board count and strict less-than comparison.
- `TR-NP-006`: `S2CGameSnapshot` is the reconnect source used to seed visible client state.
- Direct GDD ACs: `BR-STATUS-CONTRACT`, `BR-STATUS-TIER`, `BR-STATUS-COOCCUPANCY`, status-icon participation in `BR-2-ATLAS`, co-occupancy `BR-22` and `BR-22b`.

Trace registrations still needed outside this story: Network Protocol should register explicit TR coverage for keyword-state fields in `UnitBoardState` and for the eventual spawn-range live update contract. Those trace gaps do not block this narrowed visual story because the story owns the presentation projection it needs and excludes spawn range.

## In Scope

- Add the presentation-side status display model used by Board Rendering, including `StatusEffectsList`, `StatusEffectVisual`, `StatusIcon`, `StatusOverflowBadge`, and a data-driven keyword display definition resource keyed by keyword/state kind.
- Add or wire the snapshot/status projection needed for currently defined persistent keyword states: SHIELD, STUN, SILENCE, INJURED, LEADER, HASTE, BODYGUARD, OUTNUMBERED, and INJURED-granted keyword indicators exposed by `KeywordPayload`.
- Add a session-scoped `PlayerTeamMap` or equivalent presentation resource if the current client session state does not already expose player-to-team mapping to Board Rendering. Seed it from the existing lobby slot messages that carry `SessionSlot.team`.
- Store display priority as data, not branch logic. Tier 1 is used for combat-deciding protection states present in the current implementation scope, starting with SHIELD. Tier 2 is used for the other persistent or resolution-scoped states listed above.
- Sort visible icons by ascending `display_tier`, then descending `remaining_duration_sort_key`, then deterministic keyword/state key. Timed states use their remaining round count. Untimed state indicators use `0` so ties remain deterministic without inventing duration.
- Compute co-occupancy render offsets for units sharing a visible `(team, lane, cell)` group, ordered by ascending `unit_id`, with the GDD F3 assertion for more than two allied co-occupants.
- Ensure status icon children inherit the parent unit transform using Bevy `ChildOf`, so the co-occupancy X offset automatically carries to icons.
- Keep status icons on the existing board-elements atlas through `CardAtlas.board_elements_image` and `CardAtlas.board_elements_layout`.

## Acceptance Criteria

- [ ] A unit with 1..=3 active status effects renders exactly one `StatusIcon` child per effect.
- [ ] A unit with more than 3 active effects renders the top 3 status icons plus one `StatusOverflowBadge` child in the fourth slot showing the hidden count.
- [ ] Tier-1 effects always outrank Tier-2 effects regardless of insertion order in `StatusEffectsList`.
- [ ] Tier-2 effects sort by descending `remaining_duration_sort_key`; equal keys sort deterministically by keyword/state key.
- [ ] `StatusIcon` and `StatusOverflowBadge` children are positioned in the Rule 14 top-right horizontal stack and use local child transforms.
- [ ] Status icons inherit co-occupancy X offset from the unit parent through hierarchy; they do not re-center on the cell.
- [ ] Status icons and overflow badges use the board-elements atlas, not a third atlas or per-icon standalone image.
- [ ] OUTNUMBERED appears per unit carrying the OUTNUMBERED keyword/state and uses the global board-count boolean from the status projection.
- [ ] Co-occupancy positions two allied units at the same visible `(team, lane, cell)` using F3: index 0 gets negative half-offset and index 1 gets positive half-offset.
- [ ] Co-occupancy index 2 triggers the GDD-mandated `assert!` with the offending index in the message.

## Implementation Notes

- Use Bevy 0.18 Required Components API. Do not use `SpriteBundle`, `NodeBundle`, `Camera2dBundle`, `TransformBundle`, `SpatialBundle`, or `Handle<TextureAtlas>`.
- Spawn icon children as component tuples with `Sprite { texture_atlas: Some(TextureAtlas { layout, index }), .. }`, `Transform`, `Visibility`, and `ChildOf(unit_entity)`.
- Use `ChildOf`, not `Parent` or `set_parent`.
- Do not add a new `MessageReceiver<S2CPhaseChanged>` or duplicate any existing Lightyear receiver. If resolution keyword events are consumed, extend the existing Board Rendering resolution-event drain or consume a local resource written by the single drain.
- Use Bevy internal messages with `#[derive(Message)]`, `MessageWriter`, `MessageReader`, and `app.add_message::<T>()` where a local presentation signal is needed.
- Co-occupancy offset must affect children through hierarchy, not by re-centering icons on the cell.
- Status effects are visual state only; client must not run gameplay keyword logic.
- Keep the update path inside ADR-021 budgets: steady-state Presentation work under 1 ms per frame and phase-boundary or rebuild spike under 3 ms.
- Expected status-icon draw-call impact remains the GDD Rule 5 caveat: per-keyword color tinting may add 3 to 5 batches. Do not create a third atlas to reduce batching.

## Control Manifest Rules

- Manifest version reviewed: `2026-05-05`.
- Board content stays world-space `Sprite` plus `Transform`; no bevy_ui board content.
- `PresentationSet` ordering remains `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick`.
- `CardAtlas` remains the shared atlas resource for unit and board-elements sprites.
- Lightyear message ownership remains single-drain per S2C type.
- Tests for Bevy systems use real ECS `World` or `App` state, not pure mocks.

## Out of Scope

- Keyword gameplay implementation.
- Full keyword card data authoring.
- Spawn range source, replication, and persistent spawn range highlight updates.
- Final full-board atlas and browser performance evidence for status icons.
- Trap identity protocol work.
- Final art production for icons.

## Split Follow-Ups

- Spawn range source and replication story: reconcile `SpawnRange` component versus `SpawnRangeChanged` event versus `PlayerSnapshot.spawn_range_cells`; choose one live source, register trace coverage, and implement Board Rendering highlight updates from that source.
- Final visual and evidence story: capture status-icon atlas evidence, browser frame-time evidence, and any final-art icon evidence after status icons and spawn range are both implemented.

## QA Test Cases

- **Tier priority**
  - Given: one Tier-1 effect and three Tier-2 effects inserted in different orders
  - When: status icon update runs
  - Then: Tier-1 occupies slot 0 in every ordering.

- **Overflow badge**
  - Given: four active effects
  - When: icon update runs
  - Then: three icons plus one `+1` overflow badge exist.

- **Co-occupancy**
  - Given: a unit with co-occupancy X offset
  - When: status icon position is computed
  - Then: icon world X includes the unit offset.

- **OUTNUMBERED per-unit**
  - Given: a unit carrying OUTNUMBERED and a projected active OUTNUMBERED boolean
  - When: status icon update runs
  - Then: that unit receives an OUTNUMBERED status icon and unrelated units do not.

## Test Evidence

**Required evidence**:
- Visual/Feel: `production/qa/evidence/board-rendering-status-icons-evidence.md`
- Unit/integration support: `tests/unit/board_rendering/status_icons_test.rs`
- Integration support if snapshot projection is touched: `tests/integration/board_rendering/status_snapshot_projection_test.rs`

**Status**: [x] Unit support created; final visual/evidence capture deferred and not closed

## Dependencies

- Depends on: [Story 003](story-003-snapshot-spawn-units-objectives-and-hp-bars.md) Complete for snapshot-spawned board units, child HP-bar hierarchy, `LaneCell`, and `CardAtlas`.
- Not required: [Story 004](story-004-ghost-preview-hand-ui-bridge.md), [Story 005](story-005-placement-reveal-collect-and-tween.md), spawn range replication/source, and trap identity protocol work.
- Unlocks: status-icon portion of Board legibility polish and the final Board Rendering atlas/evidence follow-up.

## Completion Notes

**Completed**: 2026-05-05
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 10/10 passing for the narrowed status-icon and co-occupancy visual scope. Status icons render one child per visible effect, cap at three visible icons plus an overflow badge, sort by display tier then duration then deterministic key, use local top-right stack transforms, inherit co-occupancy parent X offsets through `ChildOf`, use the board-elements atlas, support OUTNUMBERED as a per-unit status key, and apply F3 allied co-occupancy offsets with the required index-2 `assert!`.
**Deviations**: None blocking for the narrowed scope. Advisory/deferred: final visual/browser evidence remains open, and spawn range source/replication/highlight closure remains out of scope and not closed by this story-done pass.
**Test Evidence**: `tests/unit/board_rendering/status_icons_test.rs`; `cargo test -p client --test board_rendering_status_icons_test` passed 5/5. Requested adjacent regressions also passed: `board_rendering_snapshot_spawn_test` 5/5, `board_rendering_grid_camera_test` 6/6, `board_rendering_plugin_scaffold_test` 9/9, and `board_rendering_placement_reveal_test` 3/3. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
**Code Review**: Complete locally. Lean mode applied because `production/review-mode.txt` is absent; QL-TEST-COVERAGE and LP-CODE-REVIEW external gates were skipped.
**Verification Notes**: Worker commit `9693bab086e946b6908fde7c2ee537dfa19eba91` was integrated onto current `main` as implementation commit `ea8783d38c1a9f1aa5133b3d11607bffaa3f6ad7`. `production/sprint-status.yaml` was not updated because no matching BOARD-009 row exists. `AGENTS.md`, `production/session-state/codex-orchestrator-state.md`, and `design/assets/**` were not touched for this closure.
