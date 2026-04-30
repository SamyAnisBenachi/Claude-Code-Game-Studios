# ADR-021: Presentation Layer Architecture

## Status

Accepted

## Date

2026-04-30

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 |
| **Domain** | Rendering + UI (hybrid) |
| **Knowledge Risk** | HIGH — 4 versions of breaking changes post-training-cutoff (~0.14) |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `design/gdd/board-rendering.md` (§ Bevy 0.18 API Contract), `design/gdd/card-animations.md`, `design/gdd/hud.md`, `design/gdd/hand-ui.md`, `docs/architecture/adr-002-client-server-authority.md`, `docs/architecture/adr-004-asset-loading-pipeline.md` |
| **Post-Cutoff APIs Used** | `ChildOf` component (0.16, replaces `Parent`); Required Components API — no `SpriteBundle`/`NodeBundle`/`Camera2dBundle` (0.15); `ImageNode` replaces `UiImage` (0.16); `MessageReceiver<T>` (Lightyear 0.26) for inbound S2C network messages; `MessageWriter<T>` / `MessageReader<T>` (Bevy 0.16) for Bevy-internal messages only; `Query::single()` returns `Result` (0.16); `ui_picking` feature flag (0.18, renamed from `bevy_ui_picking_backend`); `commands.entity(e).despawn()` recursive by default (0.16+); `get_entity()` returns `Result<EntityCommands, EntityDoesNotExistError>` (0.18) |
| **Verification Required** | (1) ✅ VERIFIED — `#[cfg(feature = "ui_picking")]` guard is correct. `breaking-changes.md` confirms the Cargo feature was renamed `bevy_ui_picking_backend` → `ui_picking` in 0.18; a component registered only under that feature must be gated accordingly. CI build matrix without `ui_picking` mandated in Validation Criteria. (2) ⚠️ IMPLEMENTATION GATE — bevy_tweening is a third-party crate not covered by engine-reference docs. `Lens<T>` method name `lerp()` is the historical name; verify with `cargo check` against `SpriteAlphaLens` stub before first Card Animations story. If renamed, only the method declaration changes — approach is unchanged. Captured in Risks table. (3) ⚠️ IMPLEMENTATION GATE — `Animator<T>::set_tweenable()` not verifiable from engine-reference (third-party crate). Run `cargo check` with a stub call before first Card Animations story. Fallback captured in Risks table. (4) ✅ VERIFIED — `breaking-changes.md` confirms `SpriteBundle` (which carried `Handle<TextureAtlas>`) was deprecated in 0.15. `Handle<TextureAtlas>` as an asset type no longer exists. Correct 0.15+ pattern: `Sprite { texture_atlas: Some(TextureAtlas { layout: Handle<TextureAtlasLayout>, index }), .. }`. `CardAtlas` struct in Key Interfaces uses this correctly. (5) ✅ VERIFIED — `Color::srgba` is the correct 0.15+ constructor (`Color::rgba` was renamed with the linear/sRGB split in 0.15). The `SpriteAlphaLens` implementation avoids constructors entirely, using `target.color.with_alpha(alpha)` — no constructor call needed in the lens. Where color literals are needed elsewhere, `Color::srgba` / `Color::srgb` are the correct names. (6) ✅ CONFIRMED — `current-best-practices.md` confirms board sprites use `Camera2d` + `Sprite` while UI uses `Node`; these render in separate passes with bevy_ui always above world-space. The drag-sprite edge case is resolved in Implementation Guideline 8: the hand drag preview is a bevy_ui `Node`, not a world-space `Sprite`, preserving z-order above board content. |

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-002 (client-server authority — client is a view, no game logic), ADR-003 (Cargo workspace — `client/` crate exists), ADR-004 (asset loading pipeline — `bevy_asset_loader` LoadingState pattern), ADR-009 (Round State Machine — `S2CPhaseChanged` broadcast contract) |
| **Enables** | Board Rendering epic, Hand UI epic, HUD epic, Shop/Auction UI epic, Card Animations epic |
| **Blocks** | All M2/M3 Presentation epic stories — no story for a Presentation sub-system may open until this ADR is Accepted |
| **Ordering Note** | ADR-004 established `bevy_asset_loader` LoadingState for the server. The client uses the same pattern for presentation assets (card TextureAtlas, board tileset). This ADR extends ADR-004 to the client; it does not contradict it. |

## Context

### Problem Statement

All five Presentation GDDs are designed and approved (Board Rendering, Hand UI, HUD, Shop/Auction UI, Card Animations), but no unifying ADR establishes how they compose, communicate, and order execution. Without it, five independent epic workstreams would make incompatible decisions about plugin structure, SystemSet ordering, TextureAtlas sharing, and animation lifecycle — inconsistencies expensive to reconcile mid-production. This ADR formalises five cross-cutting decisions: plugin composition, SystemSet ordering, rendering boundary, shared asset ownership, and tween lifecycle contract.

### Constraints

- **Bevy 0.18 Required Components API**: No `SpriteBundle`, `NodeBundle`, `Camera2dBundle`, or deprecated bundles. Components inserted individually via `commands.spawn((C1, C2, ...))`.
- **Client-only**: All code in this ADR lives in `client/`. No game logic, no authoritative state, no server-only types (ADR-002).
- **WASM ≤ 50 MB**: No additional rendering crates beyond the approved list (`technical-preferences.md`). `bevy_tweening` is already approved.
- **60 FPS target**: Presentation layer shares the 16.67 ms frame budget with all other client systems.
- **Pre-pooled entities**: Hand UI (Rule 1) and HUD (Rule 1) both mandate pre-pooling all ECS entities at session start — no per-round spawn/despawn in steady state.
- **No optimistic updates (ADR-002)**: All visual state changes flow from inbound S2C messages; no client-side game logic.
- **Lightyear MessageReceiver is single-drain**: Lightyear's `MessageReceiver<T>` can only be drained once per frame — the first system to read it consumes all messages (same constraint as server-side, per the `multiple_c2s_*_readers` forbidden patterns in `architecture.yaml`). On the client side this applies equally to inbound S2C messages.

### Requirements

- **R1**: Five presentation sub-systems must be composable as independently testable plugins sharing a defined SystemSet ordering.
- **R2**: Board content (units, objectives, prisms, HP bars, spawn range) always rendered as world-space 2D sprites; `bevy_ui` never used for board content.
- **R3**: Card `TextureAtlas` loaded once and shared via `Res<CardAtlas>` — no per-system reload or duplicate GPU texture upload.
- **R4**: `bevy_tweening` `Animator<T>` cancel-and-replace contract defined centrally and enforced consistently across all five sub-systems.
- **R5**: `S2CPhaseChanged` (Lightyear inbound) drained by exactly ONE shared phase-sink system. All sub-plugins read phase state from a `Res<CurrentClientPhase>` resource, not directly from the Lightyear receiver.
- **R6**: `BoardLayout` (lane → world coordinate map) available to all sub-systems as `Res<BoardLayout>` inserted by `BoardRenderingPlugin`.

## Decision

The client's presentation layer is structured as a `PresentationPlugin` that composes five ordered sub-plugins, each owning a distinct domain. A shared `PresentationSet` SystemSet defines execution order within `Update`. The board renders entirely in world-space 2D (world-space sprites are always below bevy_ui — this layering is deliberate and must not be reversed). All overlay UI uses `bevy_ui`. A single phase-sink system drains `MessageReceiver<S2CPhaseChanged>` (Lightyear) and writes to `Res<CurrentClientPhase>`; sub-plugin phase-transition systems read that resource. Assets are loaded via a client-side `bevy_asset_loader` `LoadingState` mirroring the server pattern (ADR-004), promoting shared handles into typed Resources before any sub-system initialises.

### Architecture Diagram

```
                    client/ crate — Presentation Layer

┌─────────────────────────────────────────────────────────────────┐
│ PresentationPlugin                                               │
│  (registration order = load dependency order — DO NOT reorder)  │
│                                                                  │
│  1. CardAnimationsPlugin  ── shared lens infra, AnimQueue Res   │
│  2. BoardRenderingPlugin  ── world-space sprites, BoardLayout    │
│  3. HandUiPlugin          ── bevy_ui fan, reads CardAtlas        │
│  4. HudPlugin             ── bevy_ui overlay                     │
│  5. ShopAuctionUiPlugin   ── bevy_ui panels                      │
└─────────────────────────────────────────────────────────────────┘

                    PresentationSet (inside Update)

  ┌─────────────────────────────────────────────────────────────┐
  │ PhaseTransition                                              │
  │   phase_sink_system drains MessageReceiver<S2CPhaseChanged>  │
  │   writes Res<CurrentClientPhase>                             │
  │   each sub-plugin reads Res<CurrentClientPhase>:            │
  │     switch mode, cancel tweens, toggle visibility            │
  └──────────────────────────┬──────────────────────────────────┘
                             ▼
  ┌─────────────────────────────────────────────────────────────┐
  │ MessageDrain                                                 │
  │   drain all other S2C Lightyear messages into local          │
  │   Resources (AnimQueue build, GoldDisplayState write,        │
  │   ObjectiveIdentityCache, etc.)                              │
  └──────────────────────────┬──────────────────────────────────┘
                             ▼
  ┌─────────────────────────────────────────────────────────────┐
  │ StateSync                                                    │
  │   apply Resource changes to ECS entities                     │
  │   update Text, Sprite, Transform                             │
  │   show/hide via Visibility — no S2C reads here               │
  └──────────────────────────┬──────────────────────────────────┘
                             ▼
  ┌─────────────────────────────────────────────────────────────┐
  │ AnimationTick                                                │
  │   advance AnimQueue, tick Animator<T>, emit                  │
  │   GroupDrainedSignal via MessageWriter<GroupDrainedSignal>   │
  └──────────────────────────┬──────────────────────────────────┘
                             │
              (Bevy PostUpdate → bevy_ui layout → render)

                    Shared Resources

  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
  │ CurrentClientPhase│  │ BoardLayout       │  │ CardAtlas        │
  │ (PresentationPlugin│  │ (BoardRendering   │  │ (BoardRendering  │
  │  phase-sink)      │  │  owns)            │  │  owns)           │
  └──────────────────┘  └──────────────────┘  └──────────────────┘

                         ┌──────────────────┐
                         │ AnimQueue         │
                         │ (CardAnimations   │
                         │  owns)            │
                         └──────────────────┘

                    Rendering Boundary (IMMUTABLE — do not invert)

  World-space 2D (Camera2d):         bevy_ui canvas (always above):
  ┌────────────────────────┐         ┌────────────────────────┐
  │ Board units  (Sprite)  │         │ HUD   (Node, Text)      │
  │ Objectives   (Sprite)  │         │ Hand fan  (Node)        │
  │ Prisms       (Sprite)  │         │ Shop panels  (Node)     │
  │ HP bars      (Sprite)  │         │ Auction bid box (Node)  │
  │ Spawn range  (Sprite)  │         └────────────────────────┘
  └────────────────────────┘
  Camera order: 0 (default)          Separate render pass — no camera needed;
                                     always draws above world-space content.
                                     World-space sprites CANNOT appear above
                                     bevy_ui without custom render layers.
```

### Key Interfaces

```rust
// ─── client/src/presentation/mod.rs ───────────────────────────────

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        // ORDER IS A CONTRACT — do not reorder.
        // CardAnimations first: provides SpriteAlphaLens + AnimQueue used by Board.
        // BoardRendering second: inserts BoardLayout + CardAtlas read by Hand/HUD.
        app.add_plugins((
            CardAnimationsPlugin,
            BoardRenderingPlugin,
            HandUiPlugin,
            HudPlugin,
            ShopAuctionUiPlugin,
        ));

        app.configure_sets(
            Update,
            (
                PresentationSet::PhaseTransition,
                PresentationSet::MessageDrain,
                PresentationSet::StateSync,
                PresentationSet::AnimationTick,
            ).chain(),
        );

        // Single phase-sink system: drains Lightyear MessageReceiver<S2CPhaseChanged>
        // and writes Res<CurrentClientPhase>. ALL sub-plugin phase-transition systems
        // read Res<CurrentClientPhase> — never MessageReceiver<S2CPhaseChanged> directly.
        app.init_resource::<CurrentClientPhase>();
        app.add_systems(
            Update,
            phase_sink_system.in_set(PresentationSet::PhaseTransition),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PresentationSet {
    /// Drain `MessageReceiver<S2CPhaseChanged>` (Lightyear) via the shared
    /// phase_sink_system only. Sub-plugin systems in this set read
    /// `Res<CurrentClientPhase>`, then switch visibility / cancel tweens.
    PhaseTransition,
    /// Drain all other inbound S2C Lightyear messages. Write into local
    /// Resources: AnimQueue, GoldDisplayState, ObjectiveIdentityCache, etc.
    MessageDrain,
    /// Apply Resource changes to ECS entities. Update Text, Sprite, Transform.
    /// Show/hide via Visibility. Must not read any S2C MessageReceiver<T>.
    StateSync,
    /// Advance AnimQueue. Tick Animator<T>. Emit GroupDrainedSignal.
    AnimationTick,
}

/// Canonical client-side phase state. Written by phase_sink_system only.
/// Read by all sub-plugin systems that need to know the current phase.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct CurrentClientPhase {
    pub phase: ClientPhase,
    pub round: u32,
}

fn phase_sink_system(
    mut receiver: MessageReceiver<S2CPhaseChanged>,   // Lightyear — single drain
    mut current: ResMut<CurrentClientPhase>,
) {
    // Last-write-wins if multiple phase messages arrived in one frame.
    for msg in receiver.read() {
        current.phase = msg.phase.into();
        current.round = msg.round;
    }
}


// ─── client/src/presentation/shared/card_atlas.rs ─────────────────

/// Shared card sprite atlas. Owned by BoardRenderingPlugin.
/// Inserted on OnEnter(ClientState::InSession). Read by HandUiPlugin.
///
/// Correct Bevy 0.18 usage — spawning a card sprite (no SpriteBundle):
///
///   commands.spawn((
///       Sprite {
///           image: card_atlas.image.clone(),
///           texture_atlas: Some(TextureAtlas {
///               layout: card_atlas.layout.clone(),
///               index: frame_index_for_card_id,
///           }),
///           ..default()
///       },
///       Transform::from_xyz(x, y, z),
///   ));
///
/// FORBIDDEN: Handle<TextureAtlas> — this asset type does not exist in 0.18.
#[derive(Resource)]
pub struct CardAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}


// ─── client/src/presentation/shared/board_layout.rs ──────────────

/// Lane/cell → world-space coordinate map. Owned by BoardRenderingPlugin.
/// Read by HandUiPlugin (drag coordinate conversion) and CardAnimationsPlugin.
/// See board-rendering.md Rule 3 for canonical formula.
#[derive(Resource)]
pub struct BoardLayout {
    pub board_origin: Vec2,
    pub cell_width: f32,   // world units per cell (default: 64.0)
    pub lane_height: f32,  // world units per lane (default: 80.0)
}

impl BoardLayout {
    pub fn cell_to_world(&self, lane: u8, cell: u8) -> Vec2 {
        Vec2 {
            x: self.board_origin.x + (cell - 1) as f32 * self.cell_width,
            y: self.board_origin.y - (lane - 1) as f32 * self.lane_height,
        }
    }
}


// ─── client/src/presentation/card_animations/lens.rs ─────────────

// NOTE: Verify bevy_tweening 0.18 Lens<T> trait method name before implementing.
// Historical name is `lerp()`; confirm against crate source or cargo check.

pub struct SpriteAlphaLens { pub start: f32, pub end: f32 }

impl Lens<Sprite> for SpriteAlphaLens {
    fn lerp(&mut self, target: &mut Sprite, ratio: f32) {
        let alpha = self.start + (self.end - self.start) * ratio;
        // Color has no mutating set_alpha() in Bevy 0.18. Use with_alpha().
        target.color = target.color.with_alpha(alpha);
    }
}


// ─── Tween cancel-and-replace contract ───────────────────────────
//
// NOTE: Confirm Animator<T>::set_tweenable() exists by this name in
// bevy_tweening 0.18 before committing. Run `cargo check` against a stub.
//
//  ✅ CORRECT — replace via set_tweenable():
//  if let Ok(mut anim) = animators.get_mut(entity) {
//      anim.set_tweenable(new_tween);
//  }
//
//  ✗ WRONG — despawn + respawn: discards BoardPosition, UnitOwner,
//            UnitKeywordState, and all other game-state components.
//
//  ✗ WRONG — direct Transform write while Animator<Transform> active:
//            the animator overwrites the value next frame silently.
```

### Implementation Guidelines

1. **Plugin registration order is a contract.** `CardAnimationsPlugin` must be first (provides `SpriteAlphaLens`). `BoardRenderingPlugin` must be second (inserts `BoardLayout` + `CardAtlas` that `HandUiPlugin` reads). Reordering causes runtime panics (Resource not yet inserted). A comment in `PresentationPlugin::build()` and a smoke test asserting Resources exist on session entry enforce this.

2. **Session-scoped Resources.** `BoardLayout` and `CardAtlas` are inserted on `OnEnter(ClientState::InSession)` and removed on `OnExit`. All systems reading these Resources must be scoped to `in_state(ClientState::InSession)`.

3. **Pre-pooled entities.** All HUD entities (18 total per hud.md Rule 1) and all Hand fan slots (10 fan + 9 grid + 1 drag sprite per hand-ui.md Rule 1) are spawned on session entry and despawned on session exit. Within a session, only `Visibility` is toggled — never spawn or despawn these entities mid-round.

4. **PickingBehavior guard.** Insert `PickingBehavior { should_block_lower: false, is_hoverable: false }` on HUD and other UI root `Node` entities ONLY inside `#[cfg(feature = "ui_picking")]`. Inserting without the feature compiled panics at runtime (unregistered component). CI must include a build without `ui_picking`.

5. **Local vs world Z for child entities.** A health bar entity targeting world-space Z `3.1`, spawned as `ChildOf(unit_entity)` where the unit has world Z `3.0`, must use `Transform::from_xyz(offset_x, offset_y, 0.1)`. The local `0.1` is added to the parent's `3.0` by `GlobalTransform` propagation. Never assign a child's `Transform.translation.z` to the intended world Z directly — the result will be `parent_z + intended_world_z`, not `intended_world_z`.

6. **Single S2CPhaseChanged drain.** `MessageReceiver<S2CPhaseChanged>` (Lightyear) is drained ONLY in `phase_sink_system` registered by `PresentationPlugin`. No sub-plugin system may register its own `MessageReceiver<S2CPhaseChanged>`. All sub-plugin phase-transition logic reads `Res<CurrentClientPhase>`. This parallels the `multiple_c2s_*_readers` forbidden patterns on the server.

7. **AnimationTick set contract.** The `AnimQueue` tick system runs first within `AnimationTick`. `GroupDrainedSignal` emitted by `MessageWriter<GroupDrainedSignal>` (Bevy internal) is available to consumers next frame — within the same frame only if a consumer system is ordered `.after()` the AnimQueue tick system in the same set.

8. **Rendering boundary is immutable.** Board content (units, objectives, prisms, HP bars, spawn range) is always world-space. bevy_ui panels (HUD, Hand, Shop, Auction) always render above world-space. There is no supported path to render a world-space sprite above a bevy_ui panel without a custom render layer setup. The hand drag-sprite preview is a bevy_ui `Node` element, not a world-space `Sprite`, to preserve correct z-ordering during drag.

## Alternatives Considered

### Alternative 1: Monolithic PresentationPlugin

- **Description**: All five systems' code in a single plugin. Ordering via explicit `.before()` / `.after()` chains throughout.
- **Pros**: Fewer files; no sub-plugin boundary.
- **Cons**: Single plugin grows to 500+ lines as the project matures. Cannot test one sub-system (e.g., HUD) in isolation without loading all five. No GDD-level ownership boundary per epic.
- **Rejection Reason**: Five independent GDD authors, five epic workstreams, five QA sign-offs — each needs a clean plugin boundary to own, test, and ship independently.

### Alternative 2: Event-driven only, no shared SystemSet

- **Description**: Sub-plugins communicate only via `MessageWriter/MessageReader<T>`. No `PresentationSet` ordering.
- **Pros**: Maximum decoupling.
- **Cons**: Bevy may execute phase-transition handlers from different sub-plugins in any order on the same `Update` frame. On the frame `S2CPhaseChanged { RESOLUTION }` arrives, one sub-system could be in RESOLUTION mode while another is still in PLACEMENT for one frame — visual artefact guaranteed on phase-boundary frames.
- **Rejection Reason**: Non-deterministic ordering at phase boundaries is a correctness risk. `PresentationSet::PhaseTransition` gives a guaranteed execution window across all sub-systems for the most critical frame class.

### Alternative 3: Shared PresentationAssets resource (atlas not owned by Board)

- **Description**: A top-level PresentationPlugin-level asset collection owns all shared handles.
- **Pros**: Cleaner if the shared asset set grows significantly.
- **Cons**: Adds indirection with no current benefit. For the current scope (one card atlas, one board tileset), `Res<CardAtlas>` directly inserted by `BoardRenderingPlugin` is sufficient.
- **Rejection Reason**: YAGNI. Migration to a `PresentationAssets` umbrella is non-breaking when the asset set grows.

### Alternative 4: Per-sub-plugin S2CPhaseChanged readers (rejected)

- **Description**: Each sub-plugin registers its own `MessageReceiver<S2CPhaseChanged>` (Lightyear) in its PhaseTransition system.
- **Pros**: Each sub-plugin fully self-contained.
- **Cons**: Lightyear's `MessageReceiver<T>` can only be drained once per frame — the first system to run consumes all messages. Sub-plugins ordered after the first would see zero phase change messages. This is the exact same constraint as `multiple_c2s_auction_bid_readers` on the server.
- **Rejection Reason**: Silent message loss with no compile-time error. Single shared `phase_sink_system` + `Res<CurrentClientPhase>` provides the same information to all sub-plugins without consuming the Lightyear buffer multiple times.

## Consequences

### Positive

- Five independently testable plugin boundaries map directly to five epic workstreams and five GDD owners.
- `PresentationSet` ordering guarantees deterministic phase-boundary behaviour across all sub-systems on every frame.
- Single `CardAtlas` resource eliminates duplicate texture upload and binding-slot inconsistency between board unit sprites and hand card sprites — both atlas consumers are guaranteed to use the same GPU texture binding.
- Tween cancel-and-replace protocol defined once; sub-systems cannot accidentally discard game-state components mid-animation.
- `BoardLayout` as a shared Resource guarantees drag coordinate conversion (Hand UI) and unit position lookup (Board Rendering) use identical math — no drift between the two systems' coordinate models.
- Single `phase_sink_system` for `S2CPhaseChanged` prevents the Lightyear MessageReceiver single-drain problem from propagating into presentation code.

### Negative

- `CardAnimationsPlugin` must be first in registration order. Compile-time enforcement is impossible — only a runtime panic surfaces the error (Resource not yet inserted). Mitigated by a smoke test asserting all shared Resources exist on `OnEnter(ClientState::InSession)`.
- `PresentationSet` ordering mandates all phase-change handlers run in `PhaseTransition`. A contributor adding a phase handler in `StateSync` silently breaks the ordering contract. Mitigated by the control manifest rule.
- Pre-pooling ~50 hidden entities at session start adds negligible memory overhead; no observable performance cost at the project scale.
- bevy_ui and world-space 2D sprites cannot exchange z-ordering without a custom render layer setup. The drag-sprite must be implemented as a bevy_ui `Node`, not a world-space `Sprite`, to maintain correct layering above board content during PLACEMENT.

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| `bevy_tweening` 0.18 `Lens<T>` trait method renamed (e.g. `lens()` vs `lerp()`) | Medium | High | Run `cargo check` against a `SpriteAlphaLens` stub before first Card Animations story. If renamed, update the lens impl — the approach is correct even if the method name changed. |
| `bevy_tweening` 0.18 `Animator<T>::set_tweenable()` renamed or removed | Medium | High | Verify against crates.io before first Card Animations story. Fallback: cancel-and-replace via despawn + `UnitGameState` marker component to re-attach game-state separately. |
| `PickingBehavior` crash in builds without `ui_picking` feature | Medium | High | `#[cfg(feature = "ui_picking")]` guard + CI build without that feature. |
| Two systems draining `MessageReceiver<S2CPhaseChanged>` on the same frame | Low | High | `phase_sink_system` is the sole registered reader. Control manifest rule forbids sub-plugin systems from registering `MessageReceiver<S2CPhaseChanged>`. |
| Health bar Z set to world Z instead of local Z | Medium | Medium | Code review checklist: every `ChildOf`-parented entity with a Z target must compute `local_z = target_world_z − parent_world_z`. Add ECS test asserting health bar's world Z via `GlobalTransform`. |
| Atlas frame index computed independently per sub-system, producing different results | Low | High | Both Board Rendering and Hand UI call a shared `CardAtlas::frame_index(card_id: CardId) -> usize` method. Index computation is never inlined per-system. |

## GDD Requirements Addressed

| GDD System | Requirement | How This ADR Addresses It |
|------------|-------------|--------------------------|
| board-rendering.md | Rule 2 — board rendered in 2D world-space using `Transform` + `Sprite`; no `bevy_ui` canvas for board content | Formalises the rendering boundary: board entities are always world-space sprites; HUD/Hand/Shop/Auction are bevy_ui. The boundary is immutable (Implementation Guideline 8). |
| board-rendering.md | Rule 3 — `BoardLayout` Resource accessible to any system mapping cell positions | `BoardLayout` is a shared Resource inserted by `BoardRenderingPlugin`, accessible project-wide via `Res<BoardLayout>`. |
| board-rendering.md | Bevy 0.18 API Contract — `SpriteAlphaLens`, cancel-and-replace, `TextureAtlas` as component field | `SpriteAlphaLens` owned by `CardAnimationsPlugin`. Cancel-and-replace contract defined in Key Interfaces. `CardAtlas` struct shows correct 0.18 atlas field usage. |
| hand-ui.md | Rule 1 — pre-pooled fan + drag sprite; atlas-sharing decision with Board Rendering open | Pre-pooling mandated in Implementation Guideline 3. Atlas sharing resolved by `Res<CardAtlas>`. Drag sprite is a bevy_ui `Node` (not world-space Sprite) per rendering boundary rule. |
| hud.md | Rule 1 — HUD entities pre-pooled at session start; `PickingBehavior` behind feature flag | Pre-pooling mandated in Implementation Guideline 3. Feature-flag guard in Implementation Guideline 4. |
| card-animations.md | Rule C-4 — same S2C event → simultaneous animation start across all sub-systems | `PresentationSet` guarantees all sub-systems process the same message batch (in `MessageDrain`) before any animation starts (in `AnimationTick`). No sub-system can start an animation for an event that another sub-system has not yet processed. |
| card-animations.md | Rule C-3 — tween cancel-and-replace must not despawn/respawn game-state entities | Cancel-and-replace contract (Key Interfaces) explicitly forbids despawn+respawn. `set_tweenable()` is the required path. |
| card-animations.md | Shared `SpriteAlphaLens` deliverable — required by board reveal tween, unit-death fade, ghost-fade | `SpriteAlphaLens` is defined and owned by `CardAnimationsPlugin`. All other sub-plugins import from `card_animations::lens`. |

## Performance Implications

- **CPU (steady-state)**: `PresentationSet` runs in `Update` once per frame. Steady state < 1 ms. Phase-boundary frame (hide/show ~50 entities, cancel tweens): < 3 ms spike — not player-perceptible at 60 FPS.
- **Memory**: `CurrentClientPhase` = 2 fields. `CardAtlas` = 2 Handles (~16 bytes). `BoardLayout` = Vec2 + 2 floats. `AnimQueue` = ≤ 6 `AnimGroup`s × ≤ 10 events. Total presentation layer Resources < 10 KB.
- **Load Time**: Card atlas loaded once in client `LoadingState`. Expected < 50 ms for a typical sprite sheet.
- **Network**: Presentation layer reads S2C messages via Lightyear `MessageReceiver<T>`; never writes C2S game-logic messages (ADR-002 Rule 1 — client is a read-only view).
- **WASM bundle**: No additional crates beyond the approved list. `bevy_tweening` already listed in `technical-preferences.md`.

## Migration Plan

No existing presentation code. Implementation order:

1. **Scaffold**: Create `client/src/presentation/mod.rs` with empty `PresentationPlugin`, `PresentationSet`, and `CurrentClientPhase`. Verify `configure_sets` chain compiles.
2. **Shared infra**: Implement `BoardLayout`, `CardAtlas`, `SpriteAlphaLens`, `phase_sink_system`. Unit tests for `cell_to_world()` and lens `lerp()`.
3. **BoardRenderingPlugin skeleton**: Insert `BoardLayout` + `CardAtlas` on `OnEnter(ClientState::InSession)`. Smoke test asserts both Resources accessible.
4. **HudPlugin skeleton**: Spawn pre-pooled entities on session enter. Verify `PickingBehavior` guard compiles both with and without `ui_picking`.
5. **Per-system epic stories**: Each Presentation epic begins with its sub-plugin skeleton; proceeds per `/create-stories` output.

## Validation Criteria

- [ ] `cargo check -p client` passes with no warnings after `PresentationPlugin` scaffold.
- [ ] `configure_sets(Update, (PhaseTransition, MessageDrain, StateSync, AnimationTick).chain())` accepted by Bevy scheduler — client reaches lobby with no scheduler conflicts.
- [ ] On `OnEnter(ClientState::InSession)`, `Res<BoardLayout>`, `Res<CardAtlas>`, and `Res<CurrentClientPhase>` are all accessible. Integration test injecting state directly.
- [ ] `BoardLayout::cell_to_world(1, 1)` == `board_origin`; `cell_to_world(1, 2)` == `board_origin + Vec2::new(cell_width, 0.0)`. Unit test.
- [ ] `SpriteAlphaLens::lerp()` at ratio `0.0` preserves original alpha; at `1.0` produces `end` alpha. Unit test on `Sprite::default()`.
- [ ] `cargo build -p client` without `ui_picking` feature compiles without panic. CI build matrix entry.
- [ ] `phase_sink_system` is the only system registering `MessageReceiver<S2CPhaseChanged>`. Verified by grep: `MessageReceiver<S2CPhaseChanged>` appears exactly once in `client/src/`.
- [ ] Tween cancel-and-replace does not despawn entities. ECS test advancing `Time<Virtual>` asserts entity survives `set_tweenable()` call with game-state components intact.

## Related Decisions

- [ADR-002 — Client-Server Authority Model](./adr-002-client-server-authority.md) — client is a view; this ADR operationalises it for the presentation layer.
- [ADR-003 — Cargo Workspace Structure](./adr-003-cargo-workspace-structure.md) — `client/` crate that this ADR structures.
- [ADR-004 — Asset Loading Pipeline](./adr-004-asset-loading-pipeline.md) — `bevy_asset_loader` LoadingState pattern extended to client-side presentation assets.
- [ADR-009 — Round State Machine Phase State](./adr-009-rsm-phase-state.md) — `S2CPhaseChanged` broadcast that `phase_sink_system` drains.
- [ADR-017 — Combat Resolution Execution Architecture](./adr-017-combat-resolution-execution-architecture.md) — `S2CResolutionEvent` consumed by Board Rendering and HUD in `MessageDrain`.
- `design/gdd/board-rendering.md` — Bevy 0.18 API Contract, `AnimQueue`, `BoardLayout`.
- `design/gdd/card-animations.md` — `Animator<T>` lifecycle, animation budget rules, `SpriteAlphaLens`.
- `design/gdd/hand-ui.md` — pre-pooled fan, atlas-sharing dependency.
- `design/gdd/hud.md` — pre-pooled HUD entities, `PickingBehavior` guard.
- `design/gdd/shop-auction-ui.md` — bevy_ui panel ownership.
