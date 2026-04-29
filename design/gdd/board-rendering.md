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

[To be designed]

## Edge Cases

[To be designed]

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
