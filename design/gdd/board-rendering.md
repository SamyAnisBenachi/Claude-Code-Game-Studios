# Board Rendering

> **Status**: In Design
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: No idle spectating · Simple surface · Deep emergence

## Overview

Board Rendering is the client-side system that consumes replicated server state and presents the game as a visual arena: a 5×8 grid of lanes where the player reads positions, threats, and information at a glance. It subscribes to the Lightyear-replicated board state — unit positions, HP values, objective HP, prism tokens, status effects, and spawn range — and renders them as sprites, health bars, and visual indicators in world space. When the Round State Machine shifts phases, the board's visual mode shifts with it: DRAFT/PLACEMENT shows the static board and highlights the player's valid spawn range; RESOLUTION begins with the simultaneous reveal of both players' placements and plays back the sub-step animation sequence from `S2CResolutionEvent`; the transition back to DRAFT returns the board to static state.

The board is the place the opponent cannot lie. Units they have already placed are physical evidence — their position, their type, and their facing are visible truth in a game of hidden hands and fake objectives. Board Rendering owns the legibility of that truth: unit sprites must be identifiable at full-board zoom, status effects must attach visibly to their owners, HP bars must update in real time during RESOLUTION so a player watching sub-step 6 sees damage landing as it happens. All interactive UI (hand cards, placement controls) is suppressed during RESOLUTION_EXECUTING — the board becomes a read-only tactical display and the player's job is to read it.

## Player Fantasy

**The board is the place the opponent cannot lie.**

Hands lie. Bids lie. Two of the opponent's five objectives are counterfeits. But the units they have committed to the field are sworn testimony — their positions are facts, their facing is intent, their HP is a record. Board Rendering exists to make that testimony legible.

**The emotional target:** The player feels like the director and audience of a five-act play that writes itself in real time. PLACEMENT is the rehearsal — quiet, deliberate, full of secret intent. RESOLUTION is the curtain rising on all five stages at once: lanes erupt simultaneously, units clash, objectives crack and reveal what they really were. The player's eyes sweep left to right, drinking in five lanes of consequence in seconds. The board doesn't argue; it just plays the tape. Every position is a fact the opponent committed in ink. Every objective shatter is a verdict on a bluff. The board is where the lies end — and where the better reader wins.

**What the player must feel:**
- **Active, not passive** — during RESOLUTION the board floods with information. A skilled player's eyes are scanning unit positions, objective HP deltas, status effect changes, lane commitments. Watching is reading is playing.
- **Legibility as earned power** — a veteran looks at the same mid-RESOLUTION board as a newcomer and extracts three times more information in the same glance. That gap must feel good to both: the veteran feels sharp; the newcomer can still follow what happened and learn the vocabulary.
- **The board makes me a better tactician** — not because the animations are beautiful, but because every sprite is exactly where it needs to be, every indicator is exactly the right size, and after twenty games the player reads the board faster than they think.

**What to avoid:** Treating the board as decorative substrate or invisible plumbing. The board is a protagonist in the experience — the surface on which the entire information war resolves. Animations that obscure tactical state have failed. Status indicators that require hovering to be understood have failed. If the player cannot take in all five lanes simultaneously during RESOLUTION, the board has failed.

*Pillar alignment: "No idle spectating" — watching IS playing when the board is designed to be read. "Simple surface" — the visual rule is that positions are facts: one rule, infinitely deep.*

## Detailed Design

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

**Rule 9 — Resolution animation queue.** On receipt of `S2CResolutionEvent`, Board Rendering partitions the flat event list into `AnimGroup`s by `sub_step`, sorted ascending by sub_step. Groups play sequentially: each group's events are scheduled as simultaneous `bevy_tweening` Tweens in the same frame, then `resolution_sub_step_duration_ms` elapses, then `inter_step_pause_ms` pause, then the next group begins. All Tweens for a resolution batch are scheduled in a single frame — never spread across frames. Final state data (unit positions, HP values) is always maintained in a non-tween resource/component that remains authoritative regardless of animation state.

**Rule 10 — Tween interrupt (phase skip).** If `S2CPhaseChanged(DRAFT_SHOP)` arrives while RESOLUTION animation is playing: buffer the message, do not apply it. After `ResolutionObjectiveReveal` completes, apply the buffered transition. Exception: on `S2CPhaseChanged(GAME_OVER)`, complete the current `AnimGroup`, skip remaining groups, execute objective reveals, then transition to `GameOver`.

**Rule 11 — Reconnect rebuild.** On `S2CGameSnapshot` receipt in any state, discard all in-progress animation state, despawn all board entities, and rebuild the full board from snapshot data in a single frame. Transition to the rendering state matching `snapshot.phase`. If `snapshot.phase == RESOLUTION`, enter `ResolutionExecuting` only if `S2CResolutionEvent` has also been received; otherwise enter `DraftShop` (animation is not replayed for reconnecting clients — they receive the authoritative final state directly via Lightyear component replication).

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
| `ResolutionObjectiveReveal` | Reveal animations complete | `DraftShop` (or `GameOver` if buffered) |
| Any | `S2CGameSnapshot` received | Phase-matched state (full board rebuild) |
| Any (except `GameOver`) | `S2CPhaseChanged(GAME_OVER)` | `GameOver` |

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
cell_to_world(lane, cell) = Vec2 {
    x: board_origin_x + (cell - 1) as f32 * cell_width,
    y: board_origin_y - (lane - 1) as f32 * lane_height,
}
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Lane number | `lane` | u8 | 1–5 | Logical lane index (1 = top of screen, 5 = bottom) |
| Cell number | `cell` | u8 | 1–8 | Absolute cell position within the lane |
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
fill = clamp(hp_current / hp_max, 0.0, 1.0)

bar_color = if fill >= health_bar_green_threshold { Green }
            else if fill >= health_bar_red_threshold { Yellow }
            else { Red }
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Current HP | `hp_current` | u8 | 0–`hp_max` | Unit's replicated current HP |
| Max HP | `hp_max` | u8 | 1–20 | Unit's maximum HP from card definition |
| Green threshold | `health_bar_green_threshold` | f32 | 0.5–0.75 | Fill fraction at or above which bar renders green; default 0.6 |
| Red threshold | `health_bar_red_threshold` | f32 | 0.2–0.4 | Fill fraction below which bar renders red; default 0.3 |

**Output Range:** fill ∈ [0.0, 1.0]; color ∈ {Green, Yellow, Red}.

**Examples:**
- `hp_current=2, hp_max=5` → fill=0.40 → Yellow (below green, at or above red threshold)
- `hp_current=5, hp_max=5` → fill=1.00 → Green
- `hp_current=1, hp_max=5` → fill=0.20 → Red (at red threshold boundary)

---

### F3 — Co-occupancy Render Offset (2v2 only)

The `co_occupancy_offset` formula is defined as:

```
x_offset(unit_index) = (unit_index as f32 - 0.5) * co_occupancy_side_offset
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Unit index | `unit_index` | u8 | 0–1 | Render slot within the cell; assigned by ascending entity ID among allied co-occupants |
| Side offset | `co_occupancy_side_offset` | f32 | 4.0–16.0 | World units of X displacement per unit from cell center; default 8.0 |

**Output Range:** x_offset ∈ [−`co_occupancy_side_offset`/2, +`co_occupancy_side_offset`/2]. For default: [−4.0, +4.0].

**Example:** Two allied units at (lane=2, cell=4), `co_occupancy_side_offset=16`:
- unit_index=0 → x_offset = (0.0 − 0.5) × 16 = −8.0 (left of cell center)
- unit_index=1 → x_offset = (1.0 − 0.5) × 16 = +8.0 (right of cell center)

This formula applies only in 2v2 mode. In 1v1 at most one unit per player per lane; offset is not evaluated.

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

[To be designed]

## Tuning Knobs

[To be designed]

## Visual/Audio Requirements

[To be designed]

## UI Requirements

[To be designed]

## Acceptance Criteria

[To be designed]

## Open Questions

[To be designed]
