# Board Rendering

> **Status**: Designed — /design-review 2026-04-30 MAJOR REVISION resolved in-session
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-30
> **Implements Pillar**: No idle spectating · Simple surface · Deep emergence

## Overview

Board Rendering is the client-side system that consumes replicated server state and presents the game as a visual arena: a 5×8 grid of lanes where the player reads positions, threats, and information at a glance. It subscribes to the Lightyear-replicated board state — unit positions, HP values, objective HP, prism tokens, status effects, and spawn range — and renders them as sprites, health bars, and visual indicators in world space. When the Round State Machine shifts phases, the board's visual mode shifts with it: DRAFT/PLACEMENT shows the static board and highlights the player's valid spawn range; RESOLUTION begins with the simultaneous reveal of both players' placements and plays back the sub-step animation sequence from `S2CResolutionEvent`; the transition back to DRAFT returns the board to static state.

The board is the place the opponent cannot lie. Units they have already placed are physical evidence — their position, their type, and their facing are visible truth in a game of hidden hands and fake objectives. Board Rendering owns the legibility of that truth: unit sprites must be identifiable at full-board zoom, status effects must attach visibly to their owners, HP bars must update in real time during RESOLUTION so a player watching sub-step 6 sees damage landing as it happens. All interactive UI (hand cards, placement controls) is suppressed during RESOLUTION_EXECUTING — the board becomes a read-only tactical display and the player's job is to read it.

## Player Fantasy

**The board is the place the opponent cannot lie.**

Hands lie. Bids lie. Two of the opponent's five objectives are counterfeits. But the units they have committed to the field are sworn testimony — their positions are facts, their facing is intent, their HP is a record. Board Rendering exists to make that testimony legible.

**The emotional target:** The player feels like the director and audience of a five-act play that writes itself in real time. PLACEMENT is the rehearsal — quiet, deliberate, full of secret intent. RESOLUTION is the curtain rising on all five stages at once: lanes erupt simultaneously, units clash, objectives crack and reveal what they really were. The player's eyes sweep left to right, drinking in five lanes of consequence in seconds. The board doesn't argue; it just plays the tape. Every position is a fact the opponent committed in ink. Every objective shatter is a verdict on a bluff. The board is where the lies end — and where the better reader wins.

**What the player must feel:**
- **Watching IS reading** — RESOLUTION is the savor-the-payoff phase. The player's input is locked, deliberately, so they can absorb the consequences of decisions already committed. Watching is not a non-action; it is the act of converting the round's hidden information into knowledge that informs the next PLACEMENT. The player who skims sub-steps 2–4 will misread the board next round. This is dramaturgy, not interactivity — and the design owes the player a tight, legible cut, not a long movie.
- **Legibility as earned power** — a veteran looks at the same mid-RESOLUTION board as a newcomer and extracts three times more information in the same glance: unit type vocabulary (range vs melee silhouettes, class color), HP delta patterns across rounds, opponent placement tells, prism contest outcomes. The newcomer can still follow what happened and learn the vocabulary; the veteran reads three rounds ahead.
- **The board makes me a better tactician** — not because the animations are beautiful, but because every sprite is exactly where it needs to be, every indicator is exactly the right size, and after twenty games the player reads the board faster than they think.

**What to avoid:** Treating the board as decorative substrate or invisible plumbing. The board is a protagonist in the experience — the surface on which the entire information war resolves. Animations that obscure tactical state have failed. Status indicators that require hovering to be understood have failed. If the player cannot take in all five lanes simultaneously during RESOLUTION, the board has failed. Animation budgets that exceed ~5 seconds default per RESOLUTION have also failed — beyond that the savor-the-payoff phase becomes idle dead time.

*Pillar alignment: "No idle spectating" applies to PLACEMENT and DRAFT phases where decisions are live. RESOLUTION is the deliberate watch-the-tape phase — kept tight (≤5s default, ≤8.5s ceiling) so the watch never becomes idle. "Simple surface" — the visual rule is that positions are facts: one rule, infinitely deep.*

## Detailed Design

### Data Structures

**`AnimGroup`** — one resolution sub-step's worth of simultaneous animations:

```
struct AnimGroup {
    sub_step: u8,              // 1..=6 (validated on intake; OOR = fatal desync, see Rule 9)
    events: Vec<ResolutionEvent>,  // owned by combat-resolution.md (see OQ-BR-03)
    duration_ms: u32,          // = resolution_sub_step_duration_ms
}
```

**`AnimQueue`** — `Resource` (NOT a sentinel-component pattern). Holds the queue of `AnimGroup`s for the current resolution playback, the index of the currently-playing group, and the elapsed timer:

```
#[derive(Resource, Default)]
struct AnimQueue {
    groups: Vec<AnimGroup>,    // sorted ascending by sub_step
    current_index: usize,
    group_timer: Timer,        // Bevy Timer, TimerMode::Once, advances via Time<Virtual>
    inter_step_timer: Timer,   // pause between groups
    total_duration_ms: u32,    // computed via F4 at queue construction
}

impl AnimQueue {
    fn total_duration_ms(&self) -> u32 { self.total_duration_ms }
}
```

**`PendingPhaseChange`** — `Resource<Option<RoundPhase>>` holding a buffered `S2CPhaseChanged` value when one arrives during RESOLUTION (see Rule 10). Last-write-wins on duplicate buffer (server is authoritative).

**`PendingResolutionScript`** — `Resource<Option<S2CResolutionEvent>>` holding a `S2CResolutionEvent` that arrived before its corresponding `S2CPlacementReveal` (see Rule 9). Cleared on consumption.

**Timer mechanism.** Sub-step advancement uses Bevy `Timer` components on `AnimQueue`, ticked by `Time<Virtual>` (pausable, manipulable in tests via injected delta). Wall-clock time and `Time<Real>` are NOT used — `Time<Virtual>` enables `App::update()` test patterns where the test injects time deltas to verify timing invariants headlessly (see ACs BR-7, BR-14, BR-15, BR-20).

**`ObjectiveIdentityCache`** — `Resource<HashMap<(PlayerId, Lane), bool>>` holding `is_fake` per objective, populated from `S2CObjectiveIdentities` (unicast at DRAFT_INITIAL and re-sent on reconnect per ADR-001). Board Rendering does **not** read this cache for standing-objective rendering (Rule 12 — all standing objectives render identically). The cache is kept here only to suppress the "surprise" audio sting on fake reveal when Sang Méprise was active (see OQ-BR-01).

### Bevy 0.18 API Contract

This subsection enforces the post-cutoff Bevy 0.18 API patterns. All implementers MUST follow these — they are the most common implementation traps for code generated from training data ≤0.14.

| Pattern | Required | Forbidden (pre-0.16) |
|---|---|---|
| Despawn an entity (with or without children) | `commands.entity(e).despawn()` (recursive by default in 0.16+) | `despawn_recursive()`, `despawn_descendants()` |
| Despawn an entity that may not exist | `if let Some(mut ec) = commands.get_entity(e) { ec.despawn(); }` | `commands.get_entity(e).map(EntityCommands::despawn)` (does not compile in 0.18) |
| Parent a child entity | `commands.entity(child).insert(ChildOf(parent))` or `with_children` | `set_parent()`, `Parent` component query |
| Read network/intra-client messages | `MessageReader<T>` | `EventReader<T>` (removed in 0.17+) |
| Write network/intra-client messages | `MessageWriter<T>` + `.write(...)` | `EventWriter<T>` + `.send(...)` (removed in 0.17+) |
| Single-entity query | `let Ok(e) = q.single() else { return; }` | `let e = q.single();` (returns `Result` in 0.16+, panics if used as value) |
| Sprite construction (no texture, e.g. fog) | `Sprite { color: Color::srgba(..), ..default() }` | `SpriteBundle` (deprecated 0.15+) |
| Sprite color | `Color::srgba(r, g, b, a)` | `Color::rgba(...)` (renamed 0.15) |
| Hierarchy parenting | `ChildOf` component (0.16+) | `Parent` component (removed) |

**Health bar child Z is local, not global.** The constant `Z_HEALTH_BARS = 3.1` is the **target world-space Z**. Because health bar entities are spawned as children of unit entities (whose `Transform.translation.z = 3.0`), the health bar child's `Transform.translation.z` must be `0.1` (LOCAL — added to parent's Z), not `3.1`. Any spawn site that sets `Transform::from_xyz(_, _, Z_HEALTH_BARS)` on a health bar child is incorrect. See AC BR-Z-LOCAL.

**Custom `bevy_tweening` lens for fog alpha.** `bevy_tweening` ships with `TransformPositionLens`, `TransformRotationLens`, `TransformScaleLens` — but no `Sprite.color.alpha` lens. The fog lift (Rule 7) requires a custom `SpriteAlphaLens` implementing `Lens<Sprite>` that mutates `sprite.color.set_alpha(...)`. This lens is a deliverable of the fog lift implementation story.

**Tween cancel and replace.** To replace an active `Tween<Transform>` on an entity (Rule 9, edge case "co-occupant death"), call `animator.set_tweenable(new_tween)` on the existing `Animator<Transform>` component — do NOT despawn-and-respawn the entity (loses game-state components) and do NOT write `Transform.translation` directly while an active animator exists (BR-16 invariant).

### Core Rules

**Rule 1 — Client-side only.** Board Rendering contains no game logic, no validation, and no authoritative state. It reads Lightyear-replicated ECS components and consumes reliable S2C messages to drive visual output. It never sends C2S messages.

**Rule 2 — Rendering model.** The board is rendered in 2D world-space using Bevy's `Transform` + `Sprite` system (Required Components API — no deprecated `SpriteBundle`). The coordinate system maps lanes (1–5) and cells (1–8) to world positions via the `BoardLayout` resource. All board elements are world-space sprites; no bevy_ui canvas is used for board content.

**Rule 3 — BoardLayout resource.** A single `BoardLayout` resource is inserted at startup and must be accessible to any system that maps cell positions to screen positions:

```
BoardLayout {
    board_origin: Vec2,    // world-space position of (lane=1, cell=1)
    cell_width: f32,       // world units per cell (default: 64.0)
    lane_height: f32,      // world units per lane (default: 80.0)
}

cell_to_world(lane: u8, cell: u8) -> Vec2 {
    x = board_origin.x + (cell - 1) as f32 * cell_width
    y = board_origin.y - (lane - 1) as f32 * lane_height
}
```

Hand UI uses `Res<BoardLayout>` for drag-to-cell snapping. Other Presentation systems use it for tooltip anchoring and hover detection. Never hardcode cell positions in spawn functions.

**Rule 4 — Rendering layer order (Z-axis).** All layer Z values are defined as named constants in `rendering_constants.rs`; no inline literals are used in spawn functions:

| Layer | Z constant | Contents |
|---|---|---|
| `Z_FIELD_WASH` | 0.0 | Lane-wide translucent Field card overlays |
| `Z_CELL_NODES` | 1.0 | Diamond-shaped cell node sprites |
| `Z_TRAPS_STRUCTURES` | 2.0 | Trap face-down tiles; Structure tokens |
| `Z_UNITS` | 3.0 | Unit sprites |
| `Z_HEALTH_BARS` | 3.1 | Health bar child sprites (offset via Transform parent) |
| `Z_GHOST_UNIT` | 3.5 | Ghost unit preview during PLACEMENT |
| `Z_FOG` | 4.0 | PLACEMENT fog overlay sprites |
| `Z_SPAWN_HIGHLIGHTS` | 4.1 | Spawn range highlight overlays |

**Rule 5 — Draw call budget.** All unit sprites must come from a single `TextureAtlas` (one draw call for all units). Cell nodes, objectives, prisms, and tokens must share a second "board elements" atlas. Fog and ghost sprites are the only permitted per-frame translucent batches. Health bars must be child `Sprite` entities on the same unit atlas — custom materials per unit are forbidden (they break sprite batching, producing one draw call per unit). Target ceiling: ≤ 15 draw calls per frame for the entire board.

**Rule 6 — Health bars.** Each unit entity has two child sprite entities: a background bar and a fill bar. Fill width is driven by scaling `Transform.scale.x` proportional to `hp_current / hp_max`. Color thresholds: ≥ `health_bar_green_threshold` (0.6) → green; between `health_bar_red_threshold` (0.3) and green → yellow; < `health_bar_red_threshold` → red. Health bars are always visible on all units.

**Rule 7 — Fog overlay (PLACEMENT phase only).** Two large `Sprite` entities cover each board half during PLACEMENT: one for the local player's half (`Visibility::Hidden`), one for the opponent's half (`Visibility::Visible`, `Color::srgba(0.05, 0.05, 0.2, 0.6)` — dark blue, ~60% opacity). Fog sprites are never despawned; toggled via `Visibility`. The fog lift on `S2CPlacementReveal` is a `bevy_tweening` alpha fade-out on `Sprite.color.alpha` over `fog_lift_duration_ms` (default 350ms). Both halves lift simultaneously.

**Rule 8 — Ghost unit lifecycle.** The ghost unit is a client-local entity tagged with marker component `GhostUnit`; it has no `Replicated` component and is never known to the server. Hand UI communicates targeting via a `GhostPlacementChanged { cell: Option<(u8, u8)>, card_id: Option<CardId> }` message. Board Rendering reads this message each frame and spawns/moves/despawns the ghost entity accordingly. Only one `GhostUnit` entity may exist at any time — despawn any existing ghost before spawning a new one. Ghost visual: same art as the real unit, `Sprite { color: Color::srgba(1.0, 1.0, 1.0, 0.5), .. }`, no HP bar, no status indicators. On `S2CPlacementReveal`: despawn all ghost units immediately; real unit entities for all newly placed cards appear simultaneously from replication data.

**Rule 9 — Resolution animation queue.** On receipt of `S2CResolutionEvent`, Board Rendering partitions the flat event list into `AnimGroup`s by `sub_step`, sorted ascending by sub_step. Groups play sequentially: each group's events are scheduled as simultaneous `bevy_tweening` Tweens in the same frame, then `resolution_sub_step_duration_ms` elapses (measured via `Time<Virtual>` against `AnimQueue.group_timer`), then `inter_step_pause_ms` pause, then the next group begins. All Tweens for a resolution batch are scheduled in a single frame — never spread across frames. Final state data (unit positions, HP values) is always maintained in a non-tween resource/component that remains authoritative regardless of animation state. **Validation on intake:** any `sub_step` value outside `[1, 6]` is treated as a fatal protocol desync — discard the entire `AnimQueue`, log error, and request a fresh `S2CGameSnapshot` from the server (per network-protocol.md client contract). Out-of-range sub_step is a server-side serialization bug or version mismatch, never a normal occurrence; silent skip is forbidden because it corrupts subsequent state references.

**Rule 10 — Phase change buffering during RESOLUTION.** Phase transitions during the RESOLUTION sequence must not interrupt animation playback. The buffer protects the resolution sequence from being silently truncated regardless of which direction the ordering anomaly comes from:

- **If `S2CPhaseChanged(DRAFT_SHOP)` arrives in any of `Placement`, `ResolutionReveal`, `ResolutionExecuting`, or `ResolutionObjectiveReveal`:** store in `PendingPhaseChange` (last-write-wins on duplicate). Do not transition. After `ResolutionObjectiveReveal` completes, drain the buffer and apply the transition.
- **If `S2CPhaseChanged(GAME_OVER)` arrives during any RESOLUTION state:** complete the current `AnimGroup` (do not interrupt mid-tween), skip remaining groups in the queue, execute `ResolutionObjectiveReveal` for any buffered `ObjectiveDestroyed` events, then transition to `GameOver`. Never skip the objective reveal — it is the mandatory emotional beat.
- **If a second `S2CPhaseChanged` arrives while one is already buffered:** last-write-wins. Server is authoritative; the latest target phase is the truth.

**Rule 11 — Reconnect rebuild.** On `S2CGameSnapshot` receipt in any state, discard all in-progress animation state (clear `AnimQueue`, `PendingPhaseChange`, `PendingResolutionScript`; cancel all active `Animator<Transform>` and `Animator<Sprite>` components), despawn all board entities, and rebuild the full board from snapshot data in a single frame (one `App::update()` tick). Transition to the rendering state matching `snapshot.phase`.

**Animation is never replayed on reconnect.** When `snapshot.phase == RESOLUTION`, enter `DraftShop` immediately — the reconnecting client receives the authoritative final state directly via Lightyear component replication and the snapshot payload. The resolution animation playback is sacrificed in exchange for instant, deterministic recovery.

**ADR-001 reconnect requirement.** After processing the snapshot, the client must wait for a re-sent `S2CObjectiveIdentities` unicast message (per ADR-001) to repopulate the `ObjectiveIdentityCache` before entering any actionable phase (DRAFT_SHOP, DRAFT_AUCTION, PLACEMENT). Without this, the player cannot evaluate which of their own objectives to defend. If the cache is empty when an actionable phase begins, hold in a `Reconnecting` sub-state and log a warning.

**ResolutionReveal stuck-state recovery.** If `S2CPlacementReveal` was received but `S2CResolutionEvent` does not arrive within 2000ms (server crash mid-resolution, lost message), the client requests a fresh `S2CGameSnapshot` from the server (single C2S `RequestSnapshot` call) and resets `BoardRenderState` to whatever the snapshot delivers. This is the only fallback — without it, the player is permanently stuck on a fog-lifted board with no animation, no input, no recovery. See network-protocol.md for the C2S `RequestSnapshot` contract (currently undefined — flagged as new OQ).

**2v2 reconnect symmetry.** When one player in a 2v2 match reconnects mid-RESOLUTION, the non-reconnecting clients keep animating uninterrupted (their `S2CResolutionEvent` is unaffected). The reconnecting client snapshots-then-fast-forwards to `DraftShop` per the rule above; it does not try to catch up to the live animation.

**Rule 12 — Objective rendering (ADR-001 constraint).** Board Rendering does not know which objectives are real or fake. All standing objectives render identically: stone-egg sprite + "?" glyph + HP bar + slow idle pulse (2s scale oscillation ±2%). The fill on the HP bar reflects `ObjectiveHp.hp` replicated component. On `ObjectiveDestroyed.was_fake=false`: 500ms hold → real-reveal golden flash → destruction VFX → slot cleared. On `ObjectiveDestroyed.was_fake=true`: 500ms hold → crack animation + "FAKE" overlay (800ms) → slot cleared → spawn range highlight refreshes. Multiple destructions in one RESOLUTION: reveal in ascending lane order, sequentially.

---

### States and Transitions

Board Rendering maintains a `BoardRenderState` enum driven exclusively by network events. It has no internal timers beyond animation duration.

| State | Active when | Fog | Spawn highlights | Ghost unit | Anim queue | HP bars |
|---|---|---|---|---|---|---|
| `Idle` | Pre-handshake | — | — | — | — | — |
| `Lobby` | Phase = LOBBY | Off | Off | Off | Off | Off |
| `DraftInitial` | Phase = DRAFT_INITIAL | Off | On | Off | Off | On |
| `DraftShop` / `DraftAuction` | Phase = DRAFT_SHOP or DRAFT_AUCTION | Off | On | Off | Off | On |
| `Placement` | Phase = PLACEMENT | **Opponent half active** | On (own spawn cells) | On | Off | On |
| `ResolutionReveal` | `S2CPlacementReveal` received | Lifting (fade-out tween) | Off | Despawned | Pending | On |
| `ResolutionExecuting` | Animation queue draining | Off | Off | Off | Active | On (live-update) |
| `ResolutionObjectiveReveal` | Queue exhausted; objective VFX playing | Off | Off | Off | Off | Frozen |
| `GameOver` | Phase = GAME_OVER | Off | Off | Off | Off | Frozen |

**Valid transitions:**

| From | Trigger | To |
|---|---|---|
| `Idle` | `S2CGameSnapshot` received | Phase-matched state |
| `Lobby` | `S2CPhaseChanged(DRAFT_INITIAL)` | `DraftInitial` |
| `DraftInitial` | `S2CPhaseChanged(PLACEMENT)` | `Placement` |
| `DraftShop` | `S2CPhaseChanged(PLACEMENT)` | `Placement` |
| `DraftShop` | `S2CPhaseChanged(DRAFT_AUCTION)` | `DraftAuction` |
| `DraftAuction` | `S2CPhaseChanged(DRAFT_SHOP)` | `DraftShop` |
| `Placement` | `S2CPlacementReveal` received | `ResolutionReveal` |
| `ResolutionReveal` | `pre_animation_pause_ms` elapsed | `ResolutionExecuting` |
| `ResolutionExecuting` | Queue exhausted | `ResolutionObjectiveReveal` |
| `ResolutionObjectiveReveal` | Reveal animations complete | Target from `PendingPhaseChange` buffer (`DraftShop`, `GameOver`, or default `DraftShop` if buffer empty) |
| Any | `S2CGameSnapshot` received | Phase-matched state (full board rebuild). If `snapshot.phase == RESOLUTION`, target is `DraftShop` (no animation replay per Rule 11). Wait for `S2CObjectiveIdentities` re-send before any actionable phase. |
| `Placement`, `ResolutionReveal`, `ResolutionExecuting`, `ResolutionObjectiveReveal` | `S2CPhaseChanged(DRAFT_SHOP)` | (no transition) — buffered in `PendingPhaseChange`; applied after `ResolutionObjectiveReveal` completes |
| Any RESOLUTION state | `S2CPhaseChanged(GAME_OVER)` | Complete current `AnimGroup` → execute `ResolutionObjectiveReveal` → `GameOver` |
| Any non-RESOLUTION state (except `GameOver`) | `S2CPhaseChanged(GAME_OVER)` | `GameOver` (immediate) |
| `ResolutionReveal` | 2000ms elapsed without `S2CResolutionEvent` | Request fresh snapshot via C2S `RequestSnapshot`; transition to phase from snapshot |

---

### Interactions with Other Systems

| System | Direction | Interface |
|---|---|---|
| **Board / Lane System** | Board → Rendering | Lightyear replicates `BoardPosition { lane: u8, cell: u8 }` and `UnitStats { hp_current: u8, hp_max: u8, owner: PlayerId }` per unit; Board Rendering reads these components each frame to update sprite positions and HP bar fill |
| **Objective System** | Objective → Rendering | Lightyear replicates `ObjectiveHp { hp: u8 }` per objective; `ObjectiveDestroyed { target_player_id, lane, was_fake }` reliable message triggers destruction reveal sequence in `ResolutionObjectiveReveal` |
| **Network Protocol** | Protocol → Rendering | `S2CPlacementReveal` → `ResolutionReveal` state; `S2CResolutionEvent` → animation queue population; `S2CPhaseChanged` → all `BoardRenderState` transitions; `S2CGameSnapshot` → full board rebuild |
| **Round State Machine** (client mirror) | RSM → Rendering | Phase state is received via `S2CPhaseChanged` — Board Rendering has no direct RSM dependency, only network messages |
| **Card Data & Pool** | Pool → Rendering | Card art `TextureAtlas` slice indices are looked up at unit spawn time by `card_id`; Board Rendering reads the card definition to select the correct atlas frame |
| **Game Config** | Config → Rendering | `lane_count=5` and `cells_per_lane=8` confirm board grid dimensions at startup; animation timing constants (`resolution_sub_step_duration_ms`, `fog_lift_duration_ms`) loaded from `GameConfig` resource |
| **Hand UI** | Hand UI → Rendering | `GhostPlacementChanged { cell: Option<(u8, u8)>, card_id: Option<CardId> }` message written by Hand UI; Board Rendering reads it to spawn/move/despawn the ghost unit. Hand UI reads `Res<BoardLayout>` for cell-position mapping (coordinate lookup only — no reverse dependency) |
| **HUD** | Rendering → HUD | `BoardRenderState` transition events signal HUD to show/hide the placement timer ring; the timer ring is a HUD element (bevy_ui, fixed screen position), not a Board Rendering element |
| **Card Animations** (M3) | Animations → Rendering | Card Animations will replace placeholder `bevy_tweening` slide tweens with polished animation curves in M3; Board Rendering schedules tweens from `S2CResolutionEvent` data in M2; Card Animations overrides tween configuration at M3 without changing event consumption logic |
| **ADR-001** | Constraint | `ObjectiveIdentity` (real/fake) is NOT a replicated ECS component; Board Rendering never branches on identity for standing objectives; only `ObjectiveDestroyed.was_fake` reveals identity at destruction |

## Formulas

### F1 — Cell-to-World Coordinate

The `cell_to_world` formula is defined as:

```
// PRECONDITION: 1 <= lane <= 5  AND  1 <= cell <= 8
//   - Out-of-range inputs are u8 underflow traps: 0u8 - 1 wraps to 255 in
//     release mode, producing world positions ~16,000 units off-screen with
//     no panic and no log.
//   - The implementation MUST guard with: assert!((1..=5).contains(&lane) && (1..=8).contains(&cell), "cell_to_world out of range: lane={}, cell={}", lane, cell);
//   - assert! (not debug_assert!) — we want the failure loud in WASM release builds.

cell_to_world(lane, cell) = Vec2 {
    x: board_origin_x + (cell - 1) as f32 * cell_width,
    y: board_origin_y - (lane - 1) as f32 * lane_height,
}
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Lane number | `lane` | u8 | 1–5 (asserted) | Logical lane index (1 = top of screen, 5 = bottom). `lane=0` or `lane > 5` is a panic. |
| Cell number | `cell` | u8 | 1–8 (asserted) | Absolute cell position within the lane. `cell=0` or `cell > 8` is a panic. |
| Board origin X | `board_origin_x` | f32 | tunable | World-space X of (lane=1, cell=1) — defined in `BoardLayout` resource |
| Board origin Y | `board_origin_y` | f32 | tunable | World-space Y of (lane=1, cell=1) — defined in `BoardLayout` resource |
| Cell width | `cell_width` | f32 | 48.0–96.0 | World units per cell; default 64.0 |
| Lane height | `lane_height` | f32 | 64.0–112.0 | World units per lane; default 80.0 |

**Output Range:** 2D world-space position. For default values the board spans 448 units wide (7 cell gaps × 64) and 320 units tall (4 lane gaps × 80).

**Example:** `cell_to_world(lane=3, cell=5)` → `(board_origin_x + 256, board_origin_y − 160)`. A unit in the middle lane, on Player B's home half.

---

### F2 — Health Bar Fill Fraction

The `health_bar_fill` formula is defined as:

```
// PRECONDITION: hp_max >= 1
//   - hp_max == 0 produces 0.0/0.0 = NaN; clamp(NaN, 0.0, 1.0) = NaN;
//     scale.x = NaN renders an invisible/degenerate sprite with no Bevy error.
//   - The implementation MUST guard at intake (replication ingestion):
//       let hp_max_safe = hp_max.max(1);
//       if hp_max == 0 { warn!("UnitStats.hp_max=0 from server; clamped to 1"); }
//   - Friend-game policy: silent clamp + warning, do NOT panic. Log captures the
//     server-contract violation; client keeps rendering.

fill = clamp(hp_current as f32 / hp_max_safe as f32, 0.0, 1.0)

bar_color = if fill >= health_bar_green_threshold { Green }
            else if fill >= health_bar_red_threshold { Yellow }
            else { Red }
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Current HP | `hp_current` | u8 | 0–`hp_max` (overflow tolerated; clamp saturates) | Unit's replicated current HP |
| Max HP | `hp_max` | u8 | 1–20 (clamped to ≥1 at intake) | Unit's maximum HP from card definition. `hp_max=0` is a server-contract violation; client clamps to 1 + log warning. |
| Green threshold | `health_bar_green_threshold` | f32 | 0.5–0.75 | Fill fraction at or above which bar renders green; default 0.6 |
| Red threshold | `health_bar_red_threshold` | f32 | 0.2–0.4 | Fill fraction below which bar renders red; default 0.3 |

**Output Range:** fill ∈ [0.0, 1.0]; color ∈ {Green, Yellow, Red}.

**Zero-HP visual.** At `fill=0.0`, the HP bar's `Transform.scale.x = 0.0` renders the bar structurally invisible. This is intentional: a unit at 0 HP is dead and despawns synchronously in the same tick (sub-step 5 of RESOLUTION). The "HP bars always visible" invariant (Rule 6, BR-5) applies to all **live** units (`hp_current > 0`); a 0-HP unit in mid-despawn does not violate the invariant. See edge case "EC-HP-ZERO".

**Examples:**
- `hp_current=2, hp_max=5` → fill=0.40 → Yellow (below green, at or above red threshold)
- `hp_current=5, hp_max=5` → fill=1.00 → Green
- `hp_current=1, hp_max=5` → fill=0.20 → Red (at red threshold boundary)
- `hp_current=0, hp_max=5` → fill=0.00 → Red, scale.x=0.0 (bar invisible; unit despawning same tick)
- `hp_current=3, hp_max=0` (server bug) → hp_max clamped to 1 → fill=clamp(3.0, 0.0, 1.0)=1.0 → Green + warn!() logged

---

### F3 — Co-occupancy Render Offset (2v2 only)

The `co_occupancy_offset` formula is defined as:

```
// PRECONDITION: unit_index in {0, 1}
//   - unit_index >= 2 is a server-side bug (more than two allied co-occupants
//     in a single cell — not allowed by 2v2 rules). Silently producing
//     out-of-cell render coordinates would mask the bug; instead:
//   - The implementation MUST guard with: assert!(unit_index <= 1, "F3 co-occupancy: unit_index={} > 1 — invalid co-occupancy state", unit_index);

x_offset(unit_index) = (unit_index as f32 - 0.5) * co_occupancy_side_offset
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Unit index | `unit_index` | u8 | 0–1 (asserted) | Render slot within the cell; assigned by ascending entity ID among allied co-occupants. `unit_index >= 2` is a panic. |
| Side offset | `co_occupancy_side_offset` | f32 | 4.0–16.0 | World units of X displacement per unit from cell center; default 8.0 |

**Output Range:** x_offset ∈ [−`co_occupancy_side_offset`/2, +`co_occupancy_side_offset`/2]. For default: [−4.0, +4.0].

**Example:** Two allied units at (lane=2, cell=4), `co_occupancy_side_offset=16`:
- unit_index=0 → x_offset = (0.0 − 0.5) × 16 = −8.0 (left of cell center)
- unit_index=1 → x_offset = (1.0 − 0.5) × 16 = +8.0 (right of cell center)

This formula applies only in 2v2 mode. In 1v1 at most one unit per player per lane; offset is not evaluated. **M2 scope note:** F3 is retained on the M2 critical path per design decision 2026-04-30 — the client-side rendering support for 2v2 lands in M2 even if the 2v2 game mode itself is not committed for friend-game launch. Avoids rework if 2v2 is added later.

---

### F4 — Resolution Animation Total Duration

The `resolution_animation_duration` formula is defined as:

```
total_ms = pre_animation_pause_ms
         + N_groups * (resolution_sub_step_duration_ms + inter_step_pause_ms)
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Pre-animation pause | `pre_animation_pause_ms` | u32 | 200–800 | Hold after fog lift before sub-step 1 animation begins; default 400ms |
| Sub-step duration | `resolution_sub_step_duration_ms` | u32 | 400–1500 | Active animation window per sub-step group; default 800ms |
| Inter-step pause | `inter_step_pause_ms` | u32 | 100–400 | Silent pause between consecutive groups; default 200ms |
| Group count | `N_groups` | u8 | 0–6 | Count of distinct `sub_step` values present in `S2CResolutionEvent`; sub-steps with no events contribute 0ms |

**Output Range:**
- Minimum: `pre_animation_pause_ms` (no event groups)
- Typical (all 6 sub-steps active, default timings): 400 + 6×(800+200) = **6,400ms** (~6.4 s)
- Maximum (all sub-steps, tuning ceiling): 800 + 6×(1500+400) = **12,200ms**

**Example:** A round with events only in sub-steps 1, 5, 6 (N_groups=3, defaults): `total_ms = 400 + 3×(800+200) = 3,400ms`.

## Edge Cases

**If `S2CResolutionEvent` contains a `sub_step` value outside [1–6]:** skip that group and log a warning; do not halt the animation queue.

**If `S2CPhaseChanged(DRAFT_SHOP)` arrives while `ResolutionExecuting` is active:** buffer the message; apply the transition only after `ResolutionObjectiveReveal` completes. The player must always see the full resolution sequence.

**If `S2CPhaseChanged(GAME_OVER)` arrives mid-`ResolutionExecuting`:** complete the current `AnimGroup`, skip remaining groups, execute `ResolutionObjectiveReveal` for any buffered `ObjectiveDestroyed` events, then transition to `GameOver`. Never skip the objective reveal — it is the mandatory emotional beat.

**If `S2CGameSnapshot` arrives mid-RESOLUTION (reconnect):** discard all in-progress animation state, clear the ghost unit, rebuild the full board from snapshot in one frame. If `snapshot.phase == RESOLUTION` AND `S2CResolutionEvent` has already been buffered, enter `ResolutionExecuting`. If `S2CResolutionEvent` has not yet arrived, enter `DraftShop` — the animation is not replayed; the reconnecting client receives the final board state directly via replication.

**If `S2CResolutionEvent` has N_groups=0 (no events):** advance from the `ResolutionReveal` pause directly to `ResolutionObjectiveReveal` without spawning any Tweens.

**If `ObjectiveHp` replicates a value of 0 while `ResolutionExecuting` is active:** the HP bar clamps to 0 (F2 guarantees no negative fill). The destruction VFX fires separately when `ObjectiveDestroyed` arrives during `ResolutionObjectiveReveal`.

**If two objectives are destroyed in the same RESOLUTION:** reveal in ascending lane order, sequentially. Each reveal plays its full 500ms hold → reveal animation → slot clear before the next lane begins.

**If a co-occupying allied unit dies mid-RESOLUTION:** the surviving unit must return to cell center. Cancel any in-flight `bevy_tweening` tween on the surviving unit and substitute a 0ms snap-to-center tween — do not write `Transform` directly while a tween is active on the same entity.

**If the ghost unit is hovered to an invalid cell (outside spawn range, or Minion slot occupied):** the ghost stays at the last valid cell; the invalid cell node shows a brief red tint. The ghost does not move to the invalid cell.

**If `S2CPlacementReveal` arrives before `S2CResolutionEvent`:** enter `ResolutionReveal` and begin the fog-lift tween, but do not transition to `ResolutionExecuting` until `S2CResolutionEvent` also arrives. Hold in `ResolutionReveal` indefinitely. Log a warning if the hold exceeds 2000ms.

**If `S2CResolutionEvent` arrives before `S2CPlacementReveal`** (reliable channel ordering violation — should not occur): buffer the event; do not begin any animation. When `S2CPlacementReveal` arrives, lift fog and enter `ResolutionExecuting` immediately with no `pre_animation_pause_ms` hold. Assert and log a warning.

**If a unit's `card_id` has no matching entry in the local card asset pool at spawn time** (stale client assets): render a placeholder sprite (solid-color tile + "?" glyph) at the correct cell. HP bar still renders using replicated `UnitStats`. Log an asset-miss warning. Never panic or skip the entity spawn.

**If `ObjectiveDestroyed` arrives for a lane where no objective entity currently exists on the client** (replication removed it before the reliable message was processed): suppress the destruction VFX; update spawn range highlights immediately; log a warning. Do not spawn a temporary entity — this risks double-reveal if the replicated entity arrives late.

**If `GhostPlacementChanged { cell: None, card_id: None }` arrives and no ghost entity exists** (deselect event after ghost was already cleared by `S2CPlacementReveal`): no-op. Use `commands.get_entity(e).map(EntityCommands::despawn)` — calling `commands.despawn()` on a nonexistent entity panics in Bevy 0.18.

## Dependencies

### Upstream Dependencies

| System | Type | Interface |
|---|---|---|
| **Board / Lane System** (Approved) | Hard | Lightyear replicates `BoardPosition { lane, cell }` and `UnitStats { hp_current, hp_max, owner }` per unit entity to the client; Board Rendering queries these components each frame to drive sprite positions and HP bar fill |
| **Objective System** (Approved) | Hard | Lightyear replicates `ObjectiveHp { hp }` per objective; `ObjectiveDestroyed { target_player_id, lane, was_fake }` reliable message drives the destruction reveal sequence in `ResolutionObjectiveReveal` |
| **Combat Resolution** (Designed) | Hard | Resolution sub-step event data arrives via `S2CResolutionEvent` (owned by Network Protocol); Board Rendering has no direct interface with Combat Resolution |
| **Network Protocol** (Approved) | Hard | `S2CPlacementReveal` → fog lift + unit reveal; `S2CResolutionEvent` → animation queue; `S2CPhaseChanged` → all `BoardRenderState` transitions; `S2CGameSnapshot` → full board rebuild on connect/reconnect |
| **Card Data & Pool** (Approved) | Hard | `TextureAtlas` asset loaded at startup; slice index looked up by `card_id` at unit spawn time; fallback to placeholder sprite if `card_id` is missing (EC-12) |
| **Game Config** (Approved) | Soft | `lane_count=5` and `cells_per_lane=8` confirm board grid dimensions at startup; animation timing constants (`resolution_sub_step_duration_ms`, `fog_lift_duration_ms`, `pre_animation_pause_ms`, `inter_step_pause_ms`) loaded from `GameConfig` resource |

### Peer Presentation Systems (same layer — no hard dependency, shared resource)

| System | Direction | Interface |
|---|---|---|
| **Hand UI** (Not Started) | Hand UI → Rendering | Hand UI writes `GhostPlacementChanged { cell, card_id }` messages; Board Rendering reads them to manage the ghost unit. Hand UI reads `Res<BoardLayout>` for cell-to-world coordinate lookup (no reverse dependency). |
| **HUD** (Not Started) | Rendering → HUD | `BoardRenderState` transition events signal HUD to show/hide the placement timer ring; HUD reads `ObjectiveHp` replicated components directly for display — no data passes through Board Rendering |

### Downstream Dependents

| System | Type | Interface |
|---|---|---|
| **Card Animations** (Not Started — M3) | Soft | Card Animations replaces placeholder `bevy_tweening` slide tweens with polished curves in M3; Board Rendering schedules tweens from `S2CResolutionEvent` data in M2 without knowing the final curve implementation |

## Tuning Knobs

| Knob | GameConfig field | Default | Safe Range | Too Low | Too High |
|---|---|---|---|---|---|
| `cell_width` | `board_cell_width` | 64.0 px | 48–96 | Sprites overlap; board too cramped to read | Board wider than viewport |
| `lane_height` | `board_lane_height` | 80.0 px | 64–112 | Lanes too close; unit sprites overlap vertically | Board taller than viewport |
| `fog_opacity` | `board_fog_opacity` | 0.6 | 0.4–0.8 | Opponent half still partially readable | Opponent half completely black; harsh |
| `fog_lift_duration_ms` | `board_fog_lift_ms` | 350 ms | 200–600 | Reveal feels abrupt; dramatic moment lost | Reveal sluggish; players wait too long |
| `pre_animation_pause_ms` | `board_pre_anim_pause_ms` | 400 ms | 200–800 | Players can't read the simultaneous reveal before animation begins | Dead time before action |
| `resolution_sub_step_duration_ms` | `board_sub_step_duration_ms` | 800 ms | 400–1500 | Sub-steps blur together | Resolution drags; "No idle spectating" violated |
| `inter_step_pause_ms` | `board_inter_step_pause_ms` | 200 ms | 100–400 | No breathing room; feels rushed | Resolution stalls between steps |
| `health_bar_green_threshold` | `board_hp_green_threshold` | 0.6 | 0.5–0.75 | Danger not signalled early enough | Bar turns yellow at healthy HP |
| `health_bar_red_threshold` | `board_hp_red_threshold` | 0.3 | 0.2–0.4 | Late warning (bar red only when nearly dead) | Bar constantly red; misleading |
| `co_occupancy_side_offset` | `board_co_occupancy_offset` | 8.0 px | 4–16 | Units nearly overlap in 2v2 | Units clip outside cell node |
| `prism_spin_speed` | `board_prism_spin_speed` | 0.5 rad/s | 0.2–1.0 | Prism looks static; easy to miss | Prism visibly spinning; distracting |
| `objective_reveal_hold_ms` | `board_objective_reveal_hold_ms` | 500 ms | 300–800 | Suspense lost; feels instant | Momentum killed; reveal feels padded |

**Cross-referenced knobs (owned by upstream GDDs — do not duplicate here):**

| Constant | Value | Source |
|---|---|---|
| `lane_count` | 5 | board-lane-system.md |
| `cells_per_lane` | 8 | board-lane-system.md |
| `placement_timer_seconds` | 10s | game-config.md |

## Visual/Audio Requirements

*Visual targets specified by `board-lane-system.md` (approved). This section specifies the asset requirements, VFX event catalog, and M2 hackathon priorities for implementing those targets.*

---

### Asset Requirements

**Priority:** `BLOCKING` = must ship real art for M2. `PLACEHOLDER` = colored rect or tint acceptable for hackathon.

#### Board Environment

| Asset | Approx Dims | Atlas | Priority | Notes |
|---|---|---|---|---|
| `env_board_background_default` | 512×512 | standalone | PLACEHOLDER | Stone arena floor, 3/4 perspective; flat warm-grey rect acceptable for M2 |
| `env_lane_divider_64x80` | 64×80 | board-elements | PLACEHOLDER | Raised stone ridge; flat dark line acceptable |
| `env_lane_number_label_01–05` | 32×32 ×5 | board-elements | BLOCKING | Lane numbers 1–5; high-contrast; displayed at both board ends; text asset or font glyph |

#### Cell Nodes

| Asset | Approx Dims | Atlas | Priority | Notes |
|---|---|---|---|---|
| `env_cell_node_idle_32x32` | 32×32 | board-elements | BLOCKING | Diamond shape, cyan-blue; primary navigational landmark; must be readable at board zoom |
| `env_cell_node_spawn_active_32x32` | 32×32 | board-elements | BLOCKING | Warm gold-white variant; PLACEMENT spawn highlight; must contrast with idle |
| `env_cell_node_spawn_inactive_32x32` | 32×32 | board-elements | PLACEHOLDER | M2: reuse idle node at 50% alpha |
| `env_cell_node_invalid_32x32` | 32×32 | board-elements | PLACEHOLDER | M2: red-tinted idle node |

Player A / Player B half color tinting (cool vs. warm) is applied at runtime via `Sprite.color` — no separate per-player node textures required.

#### Objectives

| Asset | Approx Dims | Atlas | Priority | Notes |
|---|---|---|---|---|
| `env_objective_unknown_64x96` | 64×96 | board-elements | BLOCKING | Stone egg + "?" glyph; all standing objectives render as this (ADR-001) |
| `env_objective_real_reveal_64x96` | 64×96 | board-elements | BLOCKING | Golden flame totem; displayed only during the ~500ms reveal window; this is the game's emotional peak |
| `env_objective_fake_crack_64x96` | 64×96 | board-elements | PLACEHOLDER | M2: tinted unknown sprite + "FAKE" text overlay |

#### Unit Bases (colorblind redundancy — shapes are load-bearing)

| Asset | Approx Dims | Atlas | Priority | Notes |
|---|---|---|---|---|
| `ui_unit_base_player_a_48x16` | 48×16 | board-elements | BLOCKING | Circle base ring; Player A; shape distinguishes from Player B in colorblind modes |
| `ui_unit_base_player_b_48x16` | 48×16 | board-elements | BLOCKING | Hexagon/diamond base ring; Player B; must be visually distinct shape from circle |

#### Special Card Type Tokens

| Asset | Approx Dims | Atlas | Priority | Notes |
|---|---|---|---|---|
| `ui_trap_tile_facedown_32x32` | 32×32 | board-elements | BLOCKING | Face-down card tile; must signal "something hidden here"; team-color ring via `Sprite.color` tint |
| `ui_structure_token_32x32` | 32×32 | board-elements | PLACEHOLDER | M2: colored rect + "S" badge glyph |
| `ui_field_wash_lane_512x80` | 512×80 | standalone | PLACEHOLDER | Full-lane translucent wash; flat colored rect at 30% alpha fully acceptable |
| `ui_field_badge_icon_24x24` | 24×24 | board-elements | PLACEHOLDER | Lane-edge badge indicating Field is active |

#### Prism

| Asset | Approx Dims | Atlas | Priority | Notes |
|---|---|---|---|---|
| `env_prism_idle_32x32` | 32×32 | board-elements | PLACEHOLDER | M2: white diamond sprite with slow rotation; inner-light sparkle cycle is M3 |

#### Fog Overlay

No texture asset. Two fog sprites use `Sprite { color: Color::srgba(0.05, 0.05, 0.2, 0.6) }` with no texture. The solid color is the fog.

#### Unit Sprite Fallback

Board Rendering requires one fallback atlas frame for EC-12 (missing card_id):
- `ui_unit_placeholder_48x64` — solid color tile + "?" glyph. Prevents render-loop panics on asset miss.

---

### VFX Event List

Complexity: **Simple** = tint/alpha tween on existing sprite · **Medium** = multi-step bevy_tweening sequence or particle-lite · **Complex** = shader or full particle system.

| Event | Trigger | Visual | Complexity | M2 Status | Audio Cue |
|---|---|---|---|---|---|
| Cell node idle pulse | Always | Scale ±3% oscillation, 1.5s loop | Simple | BLOCKING | None |
| Spawn highlight activation | PLACEMENT start | Node tint: cyan → gold-white (held) | Simple | BLOCKING | None |
| Spawn range expansion | Fake destroyed | Radial gold pulse on newly unlocked nodes | Medium | PLACEHOLDER (instant tint swap) | None |
| Ghost unit appear/move | `GhostPlacementChanged` | Semi-transparent sprite snaps to cell | Simple | BLOCKING | None |
| **Fog lift — simultaneous reveal** | `S2CPlacementReveal` | Alpha fade-out on both fog sprites, 350ms, synchronized | Simple | **BLOCKING** | **Sharp "veil lift" whoosh + chord sting** |
| Unit placed (own, post-reveal) | `S2CPlacementReveal` | Real sprite replaces ghost; no flash | Simple | BLOCKING | Soft stone-thud/card-snap (low volume) |
| Unit advance | Sub-step 2/5 move event | Slide tween cell-to-cell over `resolution_sub_step_duration_ms` | Simple | **BLOCKING** | Short footstep-shuffle; per-lane audio offset |
| HP bar live-update | HP change during RESOLUTION | Fill bar `scale.x` lerp to new value | Simple | BLOCKING | None |
| Objective attack aura | Unit reaches objective cell | Unit shifts forward ~4px; red-orange pulsing ring child sprite | Medium | PLACEHOLDER (shift only, no ring) | Heavy deep thud on HP decrease |
| Objective idle pulse | Always (standing) | Scale ±2% oscillation, 2s loop | Simple | BLOCKING | None |
| **Objective: real reveal** | `ObjectiveDestroyed.was_fake=false` | 500ms hold → golden flash overlay (3-step alpha) → slot cleared | Medium | **BLOCKING** | **Explosion/shatter + musical hit** |
| **Objective: fake reveal** | `ObjectiveDestroyed.was_fake=true` | 500ms hold → crack overlay + "FAKE" text (800ms) → slot cleared | Medium | **BLOCKING** | Hollow dud thud (intentionally underwhelming — the bluff punchline) |
| Prism idle spin | Always (prism present) | Transform rotation at `prism_spin_speed` rad/s | Simple | PLACEHOLDER (white diamond) | None |
| Prism collection | Unit enters prism cell (sub-step 5 end) | Scale spike to 1.5× then fade; collecting unit shimmer overlay, ~400ms | Medium | PLACEHOLDER (instant despawn, skip shimmer) | Bright crystalline chime |
| Trap trigger | Trap fires during RESOLUTION | Card-flip Y-axis tween → face revealed; unit passes through | Medium | PLACEHOLDER (instant face reveal, no flip) | Percussive hit + card flip reveal |
| Unit death | HP reaches 0 → `UnitRemoved` event | Alpha fade to 0, 300ms; entity despawn | Simple | BLOCKING | None (audio owned by Combat Resolution) |
| Co-occupant death → survivor recenters | Allied unit dies mid-RESOLUTION | Cancel active tween; 0ms snap-to-center tween | Simple | BLOCKING | None |
| Invalid cell hover | Ghost dragged out of range | Cell node brief red tint, 200ms, then revert | Simple | BLOCKING | None |

---

### M2 Hackathon Priorities

**Test:** Can two people sit down and follow what is happening in every lane without reading a tooltip? If yes, M2 visual bar is met.

**Must ship real art for M2 (BLOCKING):** cell nodes (idle + spawn-active), objective unknown sprite, objective real-reveal sprite, unit base rings (both shapes), lane number labels, trap face-down tile, fog lift VFX (alpha tween + audio sting), unit slide tweens, unit death fade.

**Acceptable as placeholder for M2:** board background, lane dividers, prism art and collection VFX, objective attack aura ring, fake crack frame, trap flip animation, structure token, Field wash, spawn range expansion pulse.

**Audio minimum for M2 — three cues are load-bearing:**
1. **Fog lift chord sting** — without this, the simultaneous reveal loses its drama.
2. **Footstep shuffle on advance** — signals that something is moving in a lane.
3. **Objective destruction hit** — the game's biggest moment needs audio weight.

All other audio is advisory for M2.

📌 **Asset Spec** — Visual/Audio requirements are defined. After the art bible is approved, run `/asset-spec system:board-rendering` to produce per-asset visual descriptions, dimensions, and AI generation prompts from this section.

## UI Requirements

Board Rendering is a world-space 2D sprite system. It owns no `bevy_ui` canvas elements. All positioning is driven by `Transform` in world-space via `Res<BoardLayout>`.

| UI Element | Owner | Board Rendering role |
|---|---|---|
| Board grid (5×8 cells, 40 nodes) | **Board Rendering** | Spawns and updates 40 cell node sprite entities at world positions from `cell_to_world(lane, cell)` |
| Unit sprites + health bars | **Board Rendering** | Spawns/despawns unit entities with child HP bar sprites; positions driven by `BoardPosition` replicated component |
| Fog overlay (opponent half, PLACEMENT) | **Board Rendering** | Two `Sprite` entities at `Z_FOG`; visibility and alpha driven by `BoardRenderState` |
| Spawn range highlights | **Board Rendering** | Recolors cell node sprites in response to `SpawnRangeChanged` messages during PLACEMENT |
| Ghost unit preview | **Board Rendering** | Spawns/moves/despawns the `GhostUnit` entity in response to `GhostPlacementChanged` messages from Hand UI |
| Trap tile, Structure token, Field wash | **Board Rendering** | Spawns world-space sprite entities per card type at correct cell or lane position |
| Prism token | **Board Rendering** | Spawns rotating sprite entity at spawn cell; despawns on `PrismCollected` event |
| Objective sprites + HP bars | **Board Rendering** | Spawns objective entities with HP bar; drives destruction reveal sequence in `ResolutionObjectiveReveal` |
| Placement timer ring | **HUD** (not Board Rendering) | Board Rendering publishes `BoardRenderState` transitions; HUD shows/hides the ring in response |
| Hand cards, shop panel, auction overlay | **Hand UI / Shop-Auction UI** (not Board Rendering) | Board Rendering provides `Res<BoardLayout>` for coordinate queries only |

📌 **UX Flag — Board Rendering:** This system's world-space layout determines where Hand UI drag targets and hover feedback must appear. Run `/ux-design board-view` before writing Board Rendering epics to produce a UX spec for the board view and PLACEMENT interaction. Stories referencing board UI should cite `design/ux/board-view.md`.

## Acceptance Criteria

**Classification:** BLOCKING = automated `#[test]` against real ECS `World` (no renderer, no mocks) — must pass before story is Done. ADVISORY = screenshot, live playtest, or visual inspection — evidence in `production/qa/evidence/`.

| ID | Criterion | Classification |
|---|---|---|
| BR-1 | GIVEN a `BoardLayout` with default `cell_width=64.0` and `lane_height=80.0`, WHEN the board initializes, THEN exactly 40 `CellNode` entities exist, each carrying a `Transform` whose `translation.xy` matches `cell_to_world(lane, cell)` for every (lane, cell) in [1–5]×[1–8]. No two entities share the same world position. | BLOCKING |
| BR-2 | GIVEN `board_origin=(0.0, 0.0)`, `cell_width=64.0`, `lane_height=80.0`, WHEN `cell_to_world` is called for (1,1), (5,8), and (3,5), THEN the returned `Vec2` values are `(0.0, 0.0)`, `(448.0, −320.0)`, and `(256.0, −160.0)` respectively (tolerance ≤0.01). | BLOCKING |
| BR-3 | GIVEN a fully-populated board (5 lanes, 2v2 co-occupancy, all objectives present, fog active), WHEN the frame is rendered, THEN total draw call count is ≤15; all unit sprites originate from one `TextureAtlas` batch; no per-unit custom materials are present. | ADVISORY |
| BR-4 | GIVEN a unit with `UnitStats { hp_current, hp_max }`, WHEN fill = `clamp(hp_current/hp_max, 0.0, 1.0)` is evaluated, THEN: (a) 5/5 → fill=1.0, Green; (b) 2/5 → fill=0.4, Yellow; (c) 1/5 → fill=0.2, Red; (d) 0/5 → fill=0.0, Red; (e) 6/5 (overflow) → fill=1.0, Green. HP bar child entity `Transform.scale.x` matches fill ±0.01. | BLOCKING |
| BR-5 | GIVEN any unit entity present on the board, REGARDLESS of `BoardRenderState`, THEN the unit's child HP bar entity has `Visibility::Visible` (never Hidden). | BLOCKING |
| BR-6 | GIVEN `BoardRenderState` transitions to `Placement`, THEN: (a) exactly two fog `Sprite` entities exist tagged with a fog marker; (b) opponent-half fog has `Visibility::Visible` and `Sprite.color.alpha ≥ 0.55`; (c) local-player-half fog has `Visibility::Hidden`. The same two entities are reused on subsequent PLACEMENT entries (not respawned). | BLOCKING |
| BR-7 | GIVEN `S2CPlacementReveal` is received and `ResolutionReveal` is entered, WHEN the fog lift tween begins, THEN both fog sprites' alpha values decrease simultaneously within the same frame (no sequential ordering). The lift completes within `fog_lift_duration_ms` ±50ms; both entities reach `Sprite.color.alpha = 0.0` after completion. | ADVISORY |
| BR-8 | GIVEN a `GhostPlacementChanged` event is received while a ghost entity already exists, WHEN the system processes it, THEN the old ghost is despawned before the new one is spawned, and exactly one `GhostUnit` marker entity exists in the World after processing. | BLOCKING |
| BR-9 | GIVEN one `GhostUnit` entity exists, WHEN `S2CPlacementReveal` is received, THEN the `GhostUnit` entity is despawned within the same frame (zero `GhostUnit` entities after system runs). | BLOCKING |
| BR-10 | GIVEN no `GhostUnit` entity exists, WHEN `GhostPlacementChanged { cell: None, card_id: None }` is received, THEN no panic occurs, no entity is spawned, and World state is unchanged. The despawn path uses `commands.get_entity(e).map(EntityCommands::despawn)`, not `commands.despawn()` on an unresolved entity. | BLOCKING |
| BR-11 | GIVEN a ghost unit entity is spawned, WHEN inspected in the ECS World, THEN: (a) `Sprite.color.alpha = 0.5`; (b) no child HP bar entity exists; (c) no `Replicated` component is present; (d) `Transform.translation.xy` matches `cell_to_world(lane, cell)` for the target cell. | BLOCKING |
| BR-12 | GIVEN a `S2CResolutionEvent` with sub_step values [1, 1, 3, 3, 3, 5], WHEN the animation queue is built, THEN exactly 3 `AnimGroup` entries exist sorted [1, 3, 5] with event counts [2, 3, 1] respectively; total duration equals `pre_animation_pause_ms + 3*(resolution_sub_step_duration_ms + inter_step_pause_ms)` ±1ms. | BLOCKING |
| BR-13 | GIVEN default timings (`pre=400, sub_step=800, inter=200`) and `N_groups=3`, THEN `total_ms=3400`. GIVEN `N_groups=0` (no events), THEN `total_ms=pre_animation_pause_ms`. Both values produced without side effects. | BLOCKING |
| BR-14 | GIVEN the board is in `ResolutionExecuting` with 3 `AnimGroup`s queued, WHEN `S2CPhaseChanged(DRAFT_SHOP)` is received, THEN: (a) the message is buffered and not applied immediately; (b) the animation queue drains normally; (c) after `ResolutionObjectiveReveal` completes, the board transitions to `DraftShop`. | BLOCKING |
| BR-15 | GIVEN the board is in `ResolutionExecuting` executing group 2 of 4, WHEN `S2CPhaseChanged(GAME_OVER)` is received, THEN: (a) group 2 completes its duration; (b) groups 3 and 4 are discarded; (c) `ResolutionObjectiveReveal` runs for buffered `ObjectiveDestroyed` events; (d) board transitions to `GameOver`. Board never reaches `DraftShop`. | BLOCKING |
| BR-16 | GIVEN a unit entity has an in-flight `Tween<Transform>` active, WHEN any system repositions that unit, THEN the active tween is cancelled and replaced with a new tween (0ms snap is acceptable). No system writes directly to `Transform.translation` on an entity with an active `Tween<Transform>` lens registered. | BLOCKING |
| BR-17 | GIVEN the board is in any state with N entities, WHEN `S2CGameSnapshot` is received, THEN: (a) all prior board entities are despawned within the same frame; (b) rebuilt entity count and component values match snapshot data exactly; (c) no in-progress tweens from the prior state remain. Operation completes in a single `App::update()` tick. | BLOCKING |
| BR-18 | GIVEN `S2CGameSnapshot { phase: RESOLUTION }` AND `S2CResolutionEvent` already buffered, WHEN rebuild completes, THEN `BoardRenderState` is `ResolutionExecuting`. GIVEN `S2CGameSnapshot { phase: RESOLUTION }` AND no `S2CResolutionEvent` buffered, THEN `BoardRenderState` is `DraftShop`. | BLOCKING |
| BR-19 | GIVEN 5 objective entities in the World in any state prior to `ObjectiveDestroyed`, WHEN inspected, THEN every objective entity uses the `env_objective_unknown_64x96` atlas frame index. No entity carries component data that differentiates real from fake. | BLOCKING |
| BR-20 | GIVEN `ObjectiveDestroyed { was_fake: false, lane: 3 }` is received during `ResolutionObjectiveReveal`, WHEN the sequence plays, THEN: (a) objective holds for `objective_reveal_hold_ms` (500ms ±50ms) before changing; (b) golden flash overlay fires after the hold; (c) objective entity despawns and lane slot clears; (d) spawn range highlights refresh within the same frame. | ADVISORY |
| BR-21 | GIVEN `ObjectiveDestroyed` for lanes 4 and 2 in the same RESOLUTION, WHEN `ResolutionObjectiveReveal` processes them, THEN lane 2's reveal (500ms hold + reveal + clear) fully completes before lane 4's reveal begins. The two reveals do not overlap. | BLOCKING |
| BR-22 | GIVEN two allied units at the same (lane, cell) with `co_occupancy_side_offset=8.0`, WHEN render positions are computed, THEN unit_index=0 → `x_offset=−4.0`; unit_index=1 → `x_offset=+4.0`. The two units' `Transform.translation.x` values differ by exactly `co_occupancy_side_offset` (8.0 world units). | BLOCKING |
| BR-23 | GIVEN two allied units at the same cell (indices 0 and 1) with active `Tween<Transform>` on both, WHEN unit_index=0 dies mid-RESOLUTION, THEN the surviving unit (index=1): (a) has its active tween cancelled; (b) has a new 0ms snap tween substituted that moves it to `x_offset=0.0`; (c) `Transform.translation.x` is never written directly while a tween is active. | BLOCKING |
| BR-24 | GIVEN a `S2CResolutionEvent` containing `sub_step=7` (out of range), WHEN the animation queue is built, THEN: (a) that group is omitted; (b) remaining valid groups are processed normally; (c) a warning is logged containing "sub_step" and the value; (d) no panic occurs. | BLOCKING |
| BR-25 | GIVEN all Rust source files in the board rendering module, WHEN scanned for inline `f32` literals assigned to `Transform.translation.z`, THEN no such literals are found in spawn functions. All Z values reference named constants from `rendering_constants.rs`. | ADVISORY |

**Test file targets:**
- BLOCKING unit tests → `tests/unit/board_rendering/`
- BLOCKING state machine integration tests → `tests/integration/board_rendering/`
- ADVISORY evidence → `production/qa/evidence/board-rendering-[sprint].md`

**Implementation notes:** BR-2, BR-4, BR-13, BR-22 are pure function tests — no ECS `World` needed. BR-14/BR-15 require time-stepped ECS (use `App::update()` in a loop; inject messages between updates). BR-16 is a no-panic safety invariant test. BR-3 is the only AC requiring a live browser build with GPU frame capture.

## Open Questions

**OQ-BR-01 — Sang Méprise suppression signal (OPEN)**
The fake reveal sequence has a "confirmed reveal" variant (no surprise sting) when the Sang Méprise ability was active this round — the attacker already knew the identity. Board Rendering needs a signal indicating this. The delivery mechanism is undefined: replicated component, a field in `S2CResolutionEvent`, or a dedicated S2C message. Must be resolved before the Sang Méprise keyword is authored in the Keyword System GDD.
*Owner: Network Protocol GDD + Keyword System GDD. Blocking: Keyword System.*

**OQ-BR-02 — Camera specification (OPEN)**
The GDD specifies world-space 2D sprites but does not define the camera setup. Fixed orthographic (static position and zoom) or dynamic (zoom/pan on events like objective reveal)? A fixed camera is simpler and consistent with "board as constant reference." Panning violates the "legibility at full-board zoom" requirement if any part of the board is ever outside view.
*Owner: Board Rendering GDD. Recommended: fixed orthographic, no pan/zoom in M2. The Board Rendering epic should include a camera-setup story.*

**OQ-BR-03 — `ResolutionEvent` enum variants (OPEN)**
The animation queue (Core Rule 9) dispatches on `ResolutionEvent` variants, but the complete enum is not yet defined. Combat Resolution GDD flags this as OQ5. Board Rendering cannot be fully implemented until variants (`UnitMoved`, `UnitAttacked`, `UnitKilled`, `TrapTriggered`, `ObjectiveDamaged`, etc.) are specified and registered in the Network Protocol GDD.
*Owner: Combat Resolution GDD + Network Protocol GDD. Blocking: Board Rendering implementation.*

**OQ-BR-04 — Spawn range update signal (OPEN)**
Board Rendering needs a signal when a player's spawn range expands (to update cell node highlights). The `SpawnRangeChanged` message assumed in this GDD is not yet defined in the Network Protocol GDD. Simplest option: derive it from `ObjectiveDestroyed` (fake destruction is the only cause). Must be resolved before the Board Rendering spawn highlight story is implemented.
*Owner: Network Protocol GDD. Blocking: Board Rendering spawn highlight implementation.*

**OQ-BR-05 — Unit atlas frame count and dimensions (OPEN)**
The draw call budget (≤15 per frame) assumes all unit sprites fit in one `TextureAtlas`. The art bible is not yet authored. If the 30+ Krosmaga cards each need a unique facing sprite, the atlas must be sized accordingly. If the atlas exceeds WASM bundle constraints, the single-draw-call assumption breaks and the budget must be revised.
*Owner: Art Director + Art Bible. Blocking: asset pipeline setup.*
