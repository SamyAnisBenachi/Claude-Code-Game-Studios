# Asset Specs - System: Card Animations

> **Source**: design/gdd/card-animations.md
> **Art Bible**: design/art/art-bible.md
> **Generated**: 2026-05-09
> **Status**: 24 assets specced / 0 approved / 0 in production / 0 done
> **Asset IDs**: ASSET-243 through ASSET-266

---

## Scope Notes

Card Animations owns *motion* assets — the lens implementations drive existing sprites and text using timing constants from `GameConfig`. This spec covers:

- Custom lens source files (shipped as Rust code, not image assets — documented here for pipeline completeness)
- The `PlacementPhaseAnimator` marker component (ECS type, no art asset)
- Animation timing data tables baked into `assets/config/game_config.ron`
- The jitter lookup table (F3) for deterministic damage-number offset
- Audio timing offset constants consumed by upstream systems

Card Animations does **not** own:
- Unit sprites (Board Rendering / Class System)
- Card frames or illustrations (Hand UI / card specs)
- Objective sprites (Board Rendering — ASSET-029/030/031)
- Impact flash VFX sprites (Combat Resolution — ASSET-132/133)
- Settlement overlay sprite (Shop/Auction UI)
- Timer bar material (shared-fonts-materials-shaders — ASSET-219)

Assets owned by other systems that Card Animations drives through tweens are referenced by their existing IDs; no new rows are minted for them.

---

## P0 Assets

### Custom Lens Types (Rust Source — No Image File)

These are Rust source files, not binary art assets. They are listed here because they must be authoured, reviewed, and versioned as deliverables in the animation pipeline. The "file" is the Rust source module; the "dimensions" column documents the target component type and API contract.

| Asset ID | Name | Category | Target Component / API | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-243 | `SpriteAlphaLens` Custom Lens | Rust / Animation Lens | `Sprite.color.with_alpha(f32)` — `EaseOutQuad` exits, `EaseInQuad` entrances | N/A | Needed |
| ASSET-244 | `BackgroundColorAlphaLens` Custom Lens | Rust / Animation Lens | `BackgroundColor.0.set_alpha(f32)` — `EaseOutCubic`, clamp [0.0, 1.0] | N/A | Needed |
| ASSET-245 | `SpriteColorLens` Custom Lens | Rust / Animation Lens | `Sprite.color` full RGBA — 300 ms default, palette-locked colors only | N/A | Needed |
| ASSET-246 | `TransformScaleXLens` Custom Lens | Rust / Animation Lens | `Transform.scale.x` only (Y/Z untouched) — `EaseOutQuad`, clamp ≥ 0.0 | N/A | Needed |
| ASSET-247 | `TextColorLens` Custom Lens | Rust / Animation Lens | `TextColor(Color::...)` newtype — `EaseOutCubic`, 500 ms for damage numbers | N/A | Needed |

### Animation Config Data Asset

| Asset ID | Name | Category | Format / Location | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-248 | Card Animations Timing Constants Block | Config Data | RON block in `assets/config/game_config.ron` — 7 fields (see Visual Direction) | N/A | Needed |

### Deterministic Jitter Table (F3)

| Asset ID | Name | Category | Format / Location | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-249 | Damage Number Jitter Table | Static Data / Rust Const | 8-entry `[(f32, f32)]` table in `src/card_animations/` — indices 0–7, pixel offsets from unit torso origin | N/A | Needed |

### Damage Number Text Entity Components

| Asset ID | Name | Category | Dimensions / Format | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-250 | Damage Number Text Style | Runtime Text / Material | `Text2d` + `TextFont` (Bold weight, 24 px min) + `TextColor` (Crimson Slate `#C13C38`) + `LineHeight` (required component, Bevy 0.18) — world-space, not UI-space | N/A | Needed |
| ASSET-251 | Damage Number `DespawnAfter` Timer Component | ECS Component / Data | `#[derive(Component)] struct DespawnAfter(pub Timer)` — initialized at spawn from F2 (`max(float_ms, fade_ms)`, default 500 ms) | N/A | Needed |
| ASSET-252 | `DamageNumber` Marker Component | ECS Component | `#[derive(Component)] struct DamageNumber;` — identifies damage-number entities for rebuild/reconnect cleanup sweep | N/A | Needed |

### PLACEMENT Phase Animator Marker

| Asset ID | Name | Category | Format / Location | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-253 | `PlacementPhaseAnimator` Marker Component | ECS Component | `#[derive(Component)] struct PlacementPhaseAnimator;` — attached alongside every `TweenAnim` spawned during PLACEMENT; queried by `PlacementCancelAllAnimsRequested` handler | N/A | Needed |

### StagedObjectiveRevealQueue Resource

| Asset ID | Name | Category | Format / Location | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-254 | `StagedObjectiveRevealQueue` Resource | ECS Resource | `VecDeque<(u8, Timer)>` — `u8` = `LaneId` (1-indexed); `Timer` initialized from `reveal_start_ms[i]` (F1); drained by `ResolutionObjectiveReveal` system after `AnimQueue` completes | N/A | Needed |

### `GroupDrainedSignal` Message Type

| Asset ID | Name | Category | Format / Location | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-255 | `GroupDrainedSignal` Message | Rust / ECS Message Type | `#[derive(Message)] struct GroupDrainedSignal;` — sole outbound emission from Card Animations; consumed by Board Rendering for GAME_OVER skip path (Rule C-8) | N/A | Needed |

---

## P1 Assets

### Domain Event Message Types (consumed by Card Animations)

These are the 15 inbound domain `#[derive(Message)]` types listed in Rule C-10. They are registered by upstream plugins; Card Animations reads them via `MessageReader<T>`. Tracked here for pipeline completeness — ownership is upstream.

| Asset ID | Name | Category | Owner System | Status |
|---|---|---|---|---|
| ASSET-256 | `PlacementRevealAnimReady` Message | Rust / Message Type | Board Rendering | Needed |
| ASSET-257 | `ObjectiveDestroyedAnimReady` Message | Rust / Message Type | Board Rendering | Needed |
| ASSET-258 | `BoardRebuildRequested` Message | Rust / Message Type | Board Rendering | Needed |
| ASSET-259 | `PlacementCancelAllAnimsRequested` Message | Rust / Message Type | Board Rendering | Needed |
| ASSET-260 | `DamageNumberSpawnRequested` Message | Rust / Message Type | Board Rendering | Needed |
| ASSET-261 | `CardAcquiredAnimReady` Message | Rust / Message Type | Hand UI | Needed |
| ASSET-262 | `SnapBackRequested` Message | Rust / Message Type | Hand UI | Needed |
| ASSET-263 | `HandHideRequested` / `HandShowRequested` Messages | Rust / Message Type | Hand UI | Needed |
| ASSET-264 | `AuctionPanelTransitionRequested` / `TimerBarEaseRequested` / `TimerColorZoneRequested` / `GoldTickRequested` / `SettlementOverlayRequested` / `NoBidsTransitionRequested` Messages | Rust / Message Type (×6) | Shop/Auction UI | Needed |
| ASSET-265 | `DisplacementAnimRequested` / `TrapFlipRequested` / `AuraPulseRequested` Messages | Rust / Message Type (×3) | Keyword System | Needed |

---

## Audio Assets

Card Animations does not own audio assets. All audio timing is specified as offset-based cues in `GameConfig`; upstream systems fire the audio at the correct offset. The key timing offset constants are tracked under ASSET-248.

| Config Key | Default | Used By |
|---|---|---|
| `impact_flash_audio_offset_ms` | 17 ms | Combat Resolution audio (unit advance impact) |
| Placement reveal flip offset | 27 ms (at 80 ms flip) | Board Rendering audio |
| `stagger_cadence_ms` | 100 ms | Objective reveal stagger timing |

---

### Visual Direction

**Timing constants block (ASSET-248)** — all fields in `game_config.ron`:

| Config Key | Default | Safe Range | Effect |
|---|---|---|---|
| `board_pre_anim_pause_ms` | 400 ms | 200–800 ms | Hold before RESOLUTION sub-step 1 (cognitive prep window) |
| `board_sub_step_duration_ms` | 600 ms | 451–1000 ms | Per-sub-step animation budget (must be ≥ 451 ms — see GDD Tuning Knobs sub-step floor) |
| `board_inter_step_pause_ms` | 150 ms | 100–300 ms | Gap between sub-steps |
| `card_draw_animation_ms` | 280 ms | 150–400 ms | Card-to-fan slide duration (`EaseOutQuint`) |
| `snap_back_duration_ms` | 220 ms | 100–250 ms | Drag snap-back duration (`EaseOutBack`, runtime-clamped ≤ 250 ms) |
| `stagger_cadence_ms` | 100 ms | 80–120 ms | Objective reveal stagger cadence (F1); minimum 80 ms |
| `impact_flash_audio_offset_ms` | 17 ms | 0–33 ms | Audio fire offset relative to unit advance domain event |

**Easing catalog summary (enforced at lens level):**

- Card fan slide: `EaseOutQuint`
- Snap-back: `EaseOutBack` (overshoot ~1.1)
- Panel transitions: `EaseOutCubic`
- Timer bar drain: `Linear` (continuous)
- Gold counter tick: `EaseOutQuad`
- Hover scale (60–80 ms, max 1.12×): `EaseOutQuad`
- Death fade (200–250 ms): `EaseOutQuad`
- Unit advance: `EaseOutQuad` (locked, 600 ms)
- Damage number float: `EaseOutCubic` (500 ms, +60 px)
- REPEL displacement: `EaseOutQuint`
- ATTRACT displacement: `EaseInOutQuad`
- Objective destruction overlay: step function (3-frame: 80%→60%→30% Prism White, 240 ms)

**Palette lock for color lenses** — `SpriteColorLens` and `TextColorLens` must not animate to any color outside:
- Prism White `#EEF4FF` (FIRST STRIKE / placement reveal)
- Warm Orange `#E07020` (standard combat impact)
- Crimson Slate `#C13C38` (damage numbers)
- Arcane Gold `#F5C842` (gold counter tick flash)
- Sky Blue `#3A8EDB` (Player A base)
- Terracotta `#D45C22` (Player B base)
- Ink Blue `#1A2D5A` (panels)
- Ivory (text default)

**Jitter table (ASSET-249 / F3)** — `jitter_table[event_id % 8]` (`Vec2` pixel offsets from unit torso):

| Index | Offset |
|---|---|
| 0 | `(0.0, 0.0)` |
| 1 | `(14.0, 6.0)` |
| 2 | `(-14.0, 6.0)` |
| 3 | `(8.0, 18.0)` |
| 4 | `(-8.0, 18.0)` |
| 5 | `(20.0, -2.0)` |
| 6 | `(-20.0, -2.0)` |
| 7 | `(0.0, 24.0)` |

### Technical Notes

- All custom lenses extend `bevy_tweening v0.15.0`'s `Lens<T>` trait (Bevy 0.18-compatible release).
- `Tracks<T>` is removed from the local crate — parallel same-component animation uses independent `TweenAnim` controller entities with `AnimTarget::component::<T>(target)`.
- Card Animations must load after all upstream plugins that register consumed message types (see Rule C-10 plugin load-order note).
- `CardAnimationsPlugin` registers one system set: `CardAnimationsSet::React.after(BoardRenderSet::ScheduleTweens)` — same-frame delivery of `PlacementCancelAllAnimsRequested` depends on this ordering.
- F2 startup assert: `max(float_tween_duration_ms, fade_tween_duration_ms) + 50 < board_sub_step_duration_ms` (strict `<`); panic with clear message on violation. Assert runs post-deserialization on the complete `GameConfig` struct.
- Bevy 0.18 `LineHeight` is a required component on `Text2d` entities — damage-number spawn must insert it alongside `TextFont`.
