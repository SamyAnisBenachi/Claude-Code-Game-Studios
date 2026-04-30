# Board Rendering

> **Status**: Designed — /design-review 2026-04-29 R2 MAJOR REVISION resolved in-session
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: No idle spectating · Simple surface · Deep emergence
>
> **R2 changes (2026-04-29):** Bevy 0.18 API Contract corrections (`get_entity` returns `Result`; `Sprite::from_color` for solid-color sprites; `TextureAtlas` is a component, not a `Handle<TextureAtlas>` asset). **Fog overlay system removed** — opponent commitment hiding is now server-side replication filtering of newly-placed-this-phase entities; the dramatic reveal beat is a 250ms scale-and-fade-in tween on each appearing opponent entity. F2 boundary epsilon + threshold-inversion validator added. F4 ceiling raised to 11.5s (objective reveals counted in-budget). New Rule 13 (SystemSet ordering) and Rule 14 (status-effect visual contract). Reconnect `S2CObjectiveIdentities` timeout = 5s. Player Fantasy revised to honest acknowledgement of veteran watch-time. `C2SRequestSnapshot` escalated as cross-doc dependency on `network-protocol.md`. ~16 BLOCKING items resolved.

## Overview

Board Rendering is the client-side system that consumes replicated server state and presents the game as a visual arena: a 5×8 grid of lanes where the player reads positions, threats, and information at a glance. It subscribes to the Lightyear-replicated board state — unit positions, HP values, objective HP, prism tokens, status effects, and spawn range — and renders them as sprites, health bars, and visual indicators in world space. When the Round State Machine shifts phases, the board's visual mode shifts with it: DRAFT/PLACEMENT shows the static board and highlights the player's valid spawn range; RESOLUTION begins with the simultaneous reveal of both players' placements and plays back the sub-step animation sequence from `S2CResolutionEvent`; the transition back to DRAFT returns the board to static state.

The board is the place the opponent cannot lie. Units they have already placed are physical evidence — their position, their type, and their facing are visible truth in a game of hidden hands and fake objectives. Board Rendering owns the legibility of that truth: unit sprites must be identifiable at full-board zoom, status effects must attach visibly to their owners, HP bars must update in real time during RESOLUTION so a player watching sub-step 6 sees damage landing as it happens. All interactive UI (hand cards, placement controls) is suppressed during RESOLUTION_EXECUTING — the board becomes a read-only tactical display and the player's job is to read it.

## Player Fantasy

**The board is the place the opponent cannot lie.**

Hands lie. Bids lie. Two of the opponent's five objectives are counterfeits. But the units they have committed to the field are sworn testimony — their positions are facts, their facing is intent, their HP is a record. Board Rendering exists to make that testimony legible.

**The emotional target:** The player feels like the director and audience of a five-act play that writes itself in real time. PLACEMENT is the rehearsal — quiet, deliberate, full of secret intent. RESOLUTION is the curtain rising on all five stages at once: lanes erupt simultaneously, units clash, objectives crack and reveal what they really were. The player's eyes sweep left to right, drinking in five lanes of consequence in seconds. The board doesn't argue; it just plays the tape. Every position is a fact the opponent committed in ink. Every objective shatter is a verdict on a bluff. The board is where the lies end — and where the better reader wins.

**What the player must feel:**
- **Watching IS reading (with honest caveat)** — RESOLUTION is the savor-the-payoff phase. The player's input is locked, deliberately, so they can absorb the consequences of decisions already committed. Watching converts the round's hidden information into knowledge that informs the next PLACEMENT. **Honest trade-off:** for veterans by round 10+, the watch will become meditative rather than informational — they will read the round in the first 1–2 seconds and wait through the rest. We accept this for the friend-game scope: the savor-the-payoff frame is the game's emotional anchor, and a tap-to-skip mechanism would dilute it. If post-friend-game playtests show the watch becomes painful, a "long press to fast-forward" knob is the planned escape hatch (out of scope for M2).
- **Legibility as earned power** — a veteran looks at the same mid-RESOLUTION board as a newcomer and extracts three times more information in the same glance: unit type vocabulary (range vs melee silhouettes, class color), HP delta patterns across rounds, opponent placement tells, prism contest outcomes. The newcomer can follow the spatial events (units moved, things died); strategic vocabulary is learned by playing, not from board legibility alone — the GDD makes no claim that the visual board teaches its own meta.
- **The board makes me a better tactician** — not because the animations are beautiful, but because every sprite is exactly where it needs to be, every indicator is exactly the right size, and after twenty games the player reads the board faster than they think.

**What to avoid:** Treating the board as decorative substrate or invisible plumbing. The board is a protagonist in the experience — the surface on which the entire information war resolves. Animations that obscure tactical state have failed. Status indicators that require hovering to be understood have failed. Animation budgets that exceed the stated ceiling per RESOLUTION have also failed — beyond that the savor-the-payoff phase becomes idle dead time even for first-timers.

*Pillar alignment: "No idle spectating" applies to PLACEMENT and DRAFT phases where decisions are live. RESOLUTION is the deliberate watch-the-tape phase — kept tight (≤5s default, **≤11.5s absolute ceiling** including objective-reveal sequence — see F4) so the watch never becomes structural dead time. The veteran-fatigue trade-off is acknowledged above. "Simple surface" — the visual rule is that positions are facts: one rule, infinitely deep.*

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

| Pattern | Required | Forbidden / wrong |
|---|---|---|
| Despawn an entity (with or without children) | `commands.entity(e).despawn()` (recursive by default in 0.16+) | `despawn_recursive()`, `despawn_descendants()` |
| Despawn an entity that may not exist | `if let Ok(mut ec) = commands.get_entity(e) { ec.despawn(); }` (`get_entity` returns **`Result<EntityCommands, EntityDoesNotExistError>`** in 0.18, NOT `Option`) | `if let Some(mut ec) = commands.get_entity(e) { ... }` (won't compile in 0.18); `commands.get_entity(e).map(EntityCommands::despawn)` (also fails) |
| Parent a child entity | `commands.entity(child).insert(ChildOf(parent))` or `with_children` | `set_parent()`, `Parent` component query |
| Read network/intra-client messages | `MessageReader<T>` | `EventReader<T>` (removed in 0.17+) |
| Write network/intra-client messages | `MessageWriter<T>` + `.write(...)` | `EventWriter<T>` + `.send(...)` (removed in 0.17+) |
| Single-entity query | `let Ok(e) = q.single() else { return; }` | `let e = q.single();` (returns `Result` in 0.16+, panics if used as value) |
| Solid-color (untextured) sprite | `Sprite::from_color(Color::srgba(r, g, b, a), Vec2::new(w, h))` — uses Bevy's built-in 1×1 white-pixel asset internally | `Sprite { color: Color::srgba(..), ..default() }` (renders **invisible** in 0.18 — `image` field default is a null `Handle<Image>`) |
| Sprite color tint | `Color::srgba(r, g, b, a)` (vertex-data tint, batches with siblings) | `Color::rgba(...)` (renamed 0.15); per-unit `Handle<ColorMaterial>` (breaks batching) |
| Atlas frame on a sprite | `Sprite { image: atlas_image, texture_atlas: Some(TextureAtlas { layout: Handle<TextureAtlasLayout>, index }), .. }` — `TextureAtlas` is a runtime struct (component field), NOT an asset; the underlying texture is a `Handle<Image>`; the layout is a `Handle<TextureAtlasLayout>` | `Handle<TextureAtlas>` — this type was split in 0.15 and **does not exist** as an asset handle in 0.18 |
| Hierarchy parenting | `ChildOf` component (0.16+) | `Parent` component (removed) |

**Health bar child Z is local, not global.** The constant `Z_HEALTH_BARS = 3.1` is the **target world-space Z**. Because health bar entities are spawned as children of unit entities (whose `Transform.translation.z = 3.0`), the health bar child's `Transform.translation.z` must be `0.1` (LOCAL — added to parent's Z), not `3.1`. Any spawn site that sets `Transform::from_xyz(_, _, Z_HEALTH_BARS)` on a health bar child is incorrect. See AC BR-Z-LOCAL.

**Custom `bevy_tweening` lens for sprite alpha.** `bevy_tweening` ships with `TransformPositionLens`, `TransformRotationLens`, `TransformScaleLens` — but no `Sprite.color.alpha` lens. The reveal tween (Rule 7), unit-death fade, ghost-fade-on-deselect, and any other alpha-driven sprite tween require a custom `SpriteAlphaLens` implementing `Lens<Sprite>` that mutates `sprite.color.set_alpha(...)`. This lens is a deliverable of the reveal-tween implementation story; subsequent alpha tweens reuse it.

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
| `Z_CELL_NODES` | 1.0 | Diamond-shaped cell node sprites (spawn-highlight state is encoded as a `Sprite.color` tint on the cell node — no separate Z layer) |
| `Z_TRAPS_STRUCTURES` | 2.0 | Trap face-down tiles; Structure tokens |
| `Z_OBJECTIVES` | 2.5 | Standing objective sprites (always rendered; not hidden by anything) |
| `Z_UNITS` | 3.0 | Unit sprites |
| `Z_HEALTH_BARS` | 3.1 | Health bar child sprites (LOCAL Z = 0.1 on child Transform; parent unit at Z_UNITS = 3.0; see BR-Z-LOCAL) |
| `Z_GHOST_UNIT` | 3.5 | Ghost unit preview during PLACEMENT (local-player half only) |

**Removed in R2:** `Z_FOG`, `Z_SPAWN_HIGHLIGHTS`. The fog overlay system is gone (see Rule 7). Spawn highlights are encoded as `Sprite.color` tint on `Z_CELL_NODES` sprites — no separate sprite or Z layer.

**Rule 5 — Draw call budget.** All unit sprites must come from a single unit `TextureAtlasLayout` (one draw call for all units, sharing one `Handle<Image>` for the underlying atlas image). Cell nodes, objectives, prisms, and tokens must share a second "board elements" atlas. The ghost unit, alpha-mid-reveal units (during reveal tween), and Field washes are the only permitted per-frame translucent batches. Health bars are child sprite entities — they MUST use the unit atlas's reserved 1×2 white-pixel frame (`hp_bar_white_pixel`, see Asset Requirements) so they batch with units; per-unit `Handle<ColorMaterial>` is forbidden (breaks batching). **Target ceiling: ≤ 15 draw calls per frame for the entire board.** Atlas-split fallback policy (per OQ-BR-05): each additional atlas raises the ceiling by 1 with technical-director approval.

**Rule 6 — Health bars.** Each unit entity has two child sprite entities: a background bar and a fill bar. Fill width is driven by scaling `Transform.scale.x` proportional to `hp_current / hp_max`. Color thresholds: ≥ `health_bar_green_threshold` (0.6) → green; between `health_bar_red_threshold` (0.3) and green → yellow; < `health_bar_red_threshold` → red. Health bars are always visible on all units.

**Rule 7 — Commitment hiding via server-side replication filtering + reveal tween (REPLACES fog overlay system, R2 2026-04-29).**

There is **no fog overlay**. The board (cells, lane labels, lane dividers, Field washes, standing objectives, prior-round persistent units/traps/structures) is fully visible to both players at all times — including during PLACEMENT. The grid is the place the opponent cannot lie; it must be readable.

What IS hidden during PLACEMENT is the opponent's **newly-placed-this-phase commitments** (units, traps, structures placed via `C2SSubmitPlacement` during the current PLACEMENT). This hiding is enforced **server-side**, not client-side:

- The server does NOT replicate newly-placed entities to the non-owner client until `S2CPlacementReveal` is sent (canonical end of PLACEMENT). The owner sees their own placements immediately on submission (drag-and-stage flow per `hand-ui.md`).
- On `S2CPlacementReveal` receipt, all of the opponent's newly-placed entities arrive via Lightyear component replication in the same frame as the message.
- Board Rendering applies a **reveal tween** to each newly-spawned opponent entity: scale from `unit_reveal_tween_start_scale` (0.4) → 1.0 + alpha fade 0 → 1 over `unit_reveal_tween_duration_ms` (default 250ms). All reveal tweens start in the same frame, producing the simultaneous "curtain rising on five lanes" beat.
- The local player's own newly-placed units do **not** play the reveal tween — they were already visible during PLACEMENT (drag-and-stage; see `hand-ui.md`). The reveal tween fires ONLY on entities that were newly replicated in the `S2CPlacementReveal` frame (detected via Bevy's `Added<Replicated>` filter or equivalent on the spawn frame).
- `pre_animation_pause_ms` (F4) begins **after** the reveal tween completes (i.e., reveal tween 250ms → pre-animation pause 400ms → sub-step 1). Sequential, not concurrent — the player gets a clean beat to absorb the reveal before sub-step 1 fires.

**Reveal-tween invariants:**

- The tween targets `Transform.scale` (uniform) via `bevy_tweening`'s built-in `TransformScaleLens` and `Sprite.color.alpha` via the project-local `SpriteAlphaLens` (custom `Lens<Sprite>` — bevy_tweening does not ship a sprite color lens; the lens is a deliverable of the reveal-tween implementation story).
- Authoritative state (HP, position) is set from replicated components on spawn — the tween is purely visual flourish; if the tween is cancelled (e.g. by reconnect snapshot per Rule 11), the entity snaps to scale=1.0, alpha=1.0.
- The reveal tween does NOT block input or state-machine transitions — it runs concurrently with the board entering `ResolutionReveal` state.

**Why this replaces fog:** the fog overlay was a redundant client-side visual mask on top of server-side filtering. With server filtering doing the actual hiding, the fog only obscured information the player should be able to read (opponent's grid, prior-round persistent state, objective HP). The reveal tween preserves the dramatic moment without the legibility cost — and removes ~60 lines of fog-management code (sprite lifecycle, visibility toggling, alpha-tween setup).

**M2 implementation note:** the reveal tween is BLOCKING for M2 because the simultaneous-reveal beat is the game's emotional anchor. Without it, opponent units pop into existence with no fanfare. A 250ms scale-up + fade-in is a minimal but sufficient flourish; richer choreography (lane-wave stagger, per-unit "pose" frames) is M3 polish.

**Rule 8 — Ghost unit lifecycle.** The ghost unit is a client-local entity tagged with marker component `GhostUnit`; it has no `Replicated` component and is never known to the server. Hand UI communicates targeting via a `GhostPlacementChanged { target: Option<PlayTarget>, card_id: Option<CardId> }` message (see network-protocol.md for `PlayTarget` definition). Board Rendering reads this message each frame and spawns/moves/despawns ghost-preview entities accordingly per variant:

| `target` variant | Board Rendering response |
|---|---|
| `Some(BoardCell { lane, cell })` | Spawn / move a `GhostUnit` entity at `cell_to_world(lane, cell)`. Ghost visual: same atlas frame as the real unit, with `Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.5), unit_size)` overlaid via tint OR (preferred) atlas-frame `Sprite` whose `Sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.5)` (vertex-data alpha). No HP bar, no status indicators. |
| `Some(TargetUnit { lane, unit_id })` | Apply `TargetUnitGhost` marker to the unit entity matching `unit_id` (Prism White outline pulse, 2 Hz). No new entity spawned. |
| `Some(TargetObj { player_id, lane })` | Apply `ObjectiveTargetGhost` marker to the matching objective entity (gold inner glow, static). No new entity spawned. |
| `Some(LaneWide { lane })` | Spawn / move a `LaneGhostWash` entity covering the entire column of `lane`. Translucent overlay matching the player's colour family. |
| `Some(Instant)` | No-op for Board Rendering — Instant cards have no board ghost. Hand UI's fan slot ghost is the entire visual. |
| `None` | Clear all ghost entities/markers for the corresponding `card_id`. |

Only one ghost preview may exist per `card_id` at any time — replace any existing ghost-of-the-same-card before spawning a new one. On `S2CPlacementReveal`: despawn all ghost preview entities and clear all ghost marker components immediately; real unit entities for all newly placed cards (own + opponent) appear from replication data in the same frame, and Rule 7's reveal tween fires on each newly-replicated **opponent** entity (the local player's own units do not reveal-tween — they were already on screen).

**Reverse events to Hand UI (un-staging surface).** Board Rendering owns the ghost entities, so the click/drag gestures that un-stage a card originate here. Two events are written by Board Rendering and consumed by Hand UI:

| Event | When written | Payload |
|---|---|---|
| `GhostClickedEvent` | Player clicks a ghost preview (any variant — `GhostUnit`, `TargetUnitGhost`, `ObjectiveTargetGhost`, `LaneGhostWash`) | `{ card_id: CardId }` |
| `GhostDragStartEvent` | Player mouse-downs on a ghost preview (drag-back-to-fan gesture per Hand UI Rule 8) | `{ card_id: CardId }` |

Both events are intra-client `Message<T>` types (Bevy 0.18 `MessageWriter` / `MessageReader` API, NOT pre-0.17 `EventWriter`/`EventReader`). After writing, Board Rendering does NOT remove the ghost — Hand UI's response (writing `GhostPlacementChanged { target: None, card_id }`) drives the actual removal. This keeps Hand UI as the single owner of the staging state machine.

**Rule 9 — Resolution animation queue.** On receipt of `S2CResolutionEvent`, Board Rendering partitions the flat event list into `AnimGroup`s by `sub_step`, sorted ascending by sub_step. Groups play sequentially: each group's events are scheduled as simultaneous `bevy_tweening` Tweens in the same frame, then `resolution_sub_step_duration_ms` elapses (measured via `Time<Virtual>` against `AnimQueue.group_timer`), then `inter_step_pause_ms` pause, then the next group begins. All Tweens for a resolution batch are scheduled in a single frame — never spread across frames. Final state data (unit positions, HP values) is always maintained in a non-tween resource/component that remains authoritative regardless of animation state. **Validation on intake:** any `sub_step` value outside `[1, 6]` is treated as a fatal protocol desync — discard the entire `AnimQueue`, log error, and request a fresh `S2CGameSnapshot` from the server (per network-protocol.md client contract). Out-of-range sub_step is a server-side serialization bug or version mismatch, never a normal occurrence; silent skip is forbidden because it corrupts subsequent state references.

**Rule 10 — Phase change buffering during RESOLUTION.** Phase transitions during the RESOLUTION sequence must not interrupt animation playback. The buffer protects the resolution sequence from being silently truncated regardless of which direction the ordering anomaly comes from:

- **If `S2CPhaseChanged(DRAFT_SHOP)` arrives in any of `Placement`, `ResolutionReveal`, `ResolutionExecuting`, or `ResolutionObjectiveReveal`:** store in `PendingPhaseChange` (last-write-wins on duplicate). Do not transition. After `ResolutionObjectiveReveal` completes, drain the buffer and apply the transition.
- **If `S2CPhaseChanged(GAME_OVER)` arrives during any RESOLUTION state:** complete the current `AnimGroup` (do not interrupt mid-tween), skip remaining groups in the queue, execute `ResolutionObjectiveReveal` for any buffered `ObjectiveDestroyed` events, then transition to `GameOver`. Never skip the objective reveal — it is the mandatory emotional beat.
- **If a second `S2CPhaseChanged` arrives while one is already buffered:** last-write-wins. Server is authoritative; the latest target phase is the truth.

**Rule 11 — Reconnect rebuild.** On `S2CGameSnapshot` receipt in any state, discard all in-progress animation state (clear `AnimQueue`, `PendingPhaseChange`, `PendingResolutionScript`; cancel all active `Animator<Transform>` and `Animator<Sprite>` components), despawn all board entities, and rebuild the full board from snapshot data in a single frame (one `App::update()` tick). Transition to the rendering state matching `snapshot.phase`.

**Animation is never replayed on reconnect.** When `snapshot.phase == RESOLUTION`, enter `DraftShop` immediately — the reconnecting client receives the authoritative final state directly via Lightyear component replication and the snapshot payload. The resolution animation playback is sacrificed in exchange for instant, deterministic recovery.

**ADR-001 reconnect requirement.** Rule 11 entry point clears the `ObjectiveIdentityCache` resource explicitly (it is a Resource, not animation state — listed separately from `AnimQueue` etc. for clarity). After processing the snapshot, the client must wait for a re-sent `S2CObjectiveIdentities` unicast message (per ADR-001) to repopulate the cache before entering any actionable phase (DRAFT_SHOP, DRAFT_AUCTION, PLACEMENT). Without this, the player cannot evaluate which of their own objectives to defend. The client holds in a `Reconnecting` sub-state until the cache is populated.

**`S2CObjectiveIdentities` reconnect timeout (R2 2026-04-29).** If `S2CObjectiveIdentities` does not arrive within `objective_identities_reconnect_timeout_ms` (default **5000ms**) of the snapshot rebuild completing, the client logs an error and re-issues `C2SRequestSnapshot`. Repeats indefinitely (exponential backoff: 5s, 10s, 20s, 30s ceiling) until either identities arrive or the Lightyear heartbeat-disconnect (30s grace per `network-protocol.md` Rule 8) terminates the session. This prevents permanent block on a buggy server reconnect handler.

**Snapshot phase-content invariant (R2 2026-04-29 — cross-doc dependency).** When `snapshot.phase == RESOLUTION` is sent, the server MUST include the post-resolution final state (final HP values, despawned units removed from `BoardSnapshot.units`, awarded gold in `PlayerSnapshot.gold`, updated `ObjectiveHp` values for any destroyed objectives this round). If the snapshot were sent with pre-resolution state, the reconnecting client lands on `DraftShop` with a desynced board. **This invariant must be added to `network-protocol.md` Rule 7 / `round-state-machine.md` Rule for snapshot construction.**

**ResolutionReveal stuck-state recovery.** If `S2CPlacementReveal` was received but `S2CResolutionEvent` does not arrive within `resolution_reveal_timeout_ms` (default 2000ms — server crash mid-resolution, lost message), the client requests a fresh `S2CGameSnapshot` from the server (single `C2SRequestSnapshot` call) and resets `BoardRenderState` to whatever the snapshot delivers. This is the only message-loss fallback — for a true server crash, Lightyear's 30s heartbeat-disconnect is the actual last resort. The `C2SRequestSnapshot` contract is **currently undefined in `network-protocol.md`** — see OQ-BR-06 (BLOCKING for implementation).

**2v2 reconnect symmetry.** When one player in a 2v2 match reconnects mid-RESOLUTION, the non-reconnecting clients keep animating uninterrupted (their `S2CResolutionEvent` is unaffected). The reconnecting client snapshots-then-fast-forwards to `DraftShop` per the rule above; it does not try to catch up to the live animation.

**Rule 12 — Objective rendering (ADR-001 constraint).** Board Rendering does not know which standing objectives are real or fake. All standing objectives render identically: stone-egg sprite + "?" glyph + HP bar + slow idle pulse (2s scale oscillation ±2%). The fill on the HP bar reflects `ObjectiveHp.hp` replicated component. On `ObjectiveDestroyed.was_fake=false`: 500ms hold → real-reveal golden flash → destruction VFX → slot cleared. On `ObjectiveDestroyed.was_fake=true`: 500ms hold → crack animation + "FAKE" overlay (800ms) → slot cleared → spawn range highlight refreshes. Multiple destructions in one RESOLUTION: reveal in ascending lane order, sequentially (sort `ObjectiveDestroyed` messages by `lane` before queuing — arrival order is not guaranteed).

**ADR-001 isolation invariant:** the rendering system MUST NOT query any component whose name suggests identity (`RealObjective`, `FakeObjective`, `ObjectiveIdentity`, `IsKnown`, etc.). The `ObjectiveIdentityCache` resource is read ONLY for the Sang Méprise audio-suppression branch (OQ-BR-01) and is never read for standing-objective rendering. See AC BR-19 for the runtime invariant test (ComponentId set equality across all standing objective entities).

**Trap face-down rendering** uses the same hidden-identity pattern but is currently unspecified pending NP-OQ-2 resolution (per-client component visibility in Lightyear 0.26). See OQ-BR-07.

**Rule 13 — SystemSet ordering (NEW R2 2026-04-29).** Board Rendering systems run within an explicit `BoardRenderSet` enum so that Rule 9's "all tweens scheduled in a single frame" invariant is enforceable:

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BoardRenderSet {
    ReadMessages,        // consume S2CPlacementReveal, S2CResolutionEvent, S2CPhaseChanged, S2CGameSnapshot
    ResolveStateMachine, // BoardRenderState transitions; populate AnimQueue / PendingPhaseChange / PendingResolutionScript
    SpawnEntities,       // spawn newly-replicated units, ghost previews; commands queued
    ScheduleTweens,      // build & insert Animator<Transform> / Animator<Sprite> for current AnimGroup or reveal tween
    UpdateHpBars,        // apply F2 fill + color from replicated UnitStats (poll-based per OQ-BR-10 Approach A)
    TickAnimations,      // bevy_tweening's component_animator_system runs HERE (configured via plugin ordering)
}
```

Configured ordering: `ReadMessages → ResolveStateMachine → SpawnEntities → ScheduleTweens → UpdateHpBars → TickAnimations`. All in `Update`. The `bevy_tweening` `TweeningPlugin` is configured to place its tick systems in `BoardRenderSet::TickAnimations` so reveal tweens / Tweens scheduled in the same frame begin their first tick on the next frame's `TickAnimations` (predictable 1-frame latency). All systems writing `Animator<*>` MUST be in `BoardRenderSet::ScheduleTweens` (enforced by lint or code review). All HP bar updates MUST be in `UpdateHpBars` to avoid races with `TickAnimations`.

**Rule 14 — Status effect visual contract (NEW R2 2026-04-29).** Status effects (HASTE, STUN, POISON, etc., owned by `keyword-system.md`) attach visibly to their owning unit:

| Property | Value |
|---|---|
| Position | Top-right of unit sprite (offset `Vec2 { x: +unit_w/2 - 8.0, y: +unit_h/2 - 8.0 }`) |
| Size | 16×16 px per icon |
| Z layer | LOCAL Z = 0.05 on child Transform (parent unit at `Z_UNITS` = 3.0; child renders at global Z = 3.05, just above unit, below HP bar at 3.1) |
| Atlas | board-elements atlas (status icons share the second atlas — no third atlas) |
| Max simultaneous visible | 3 icons; 4+ active effects render an overflow `+N` badge in the 4th slot |
| Layout | Horizontal stack: icon[0] at top-right corner; icons[1..2] offset left by 16px each; overflow badge at icon[3] slot |
| Update mechanism | Bevy `Changed<StatusEffectsList>` filter on parent unit; child icons spawned/despawned to match. No tweening — instant on/off. |
| Z stacking with co-occupancy | Status icons inherit parent unit's `Transform.translation.x` (including F3 co-occupancy offset); icons stay attached to their owning unit. |

**Tooltips:** the Player Fantasy says "status indicators that require hovering to be understood have failed." Every status icon must be readable from its glyph alone; no tooltip is required for legibility. (A future polish pass may add hover tooltips for *deeper* info — e.g. exact remaining duration — but the icon must communicate the keyword without it.)

**Cross-doc dependency:** the actual icon-to-keyword mapping is owned by `keyword-system.md`. Board Rendering provides the layout slots; the keyword GDD provides the icons.

---

### States and Transitions

Board Rendering maintains a `BoardRenderState` enum driven exclusively by network events. It has no internal timers beyond animation duration.

| State | Active when | Spawn highlights | Ghost unit | Anim queue | HP bars | Reveal tween |
|---|---|---|---|---|---|---|
| `Idle` | Pre-handshake | — | — | — | — | — |
| `Lobby` | Phase = LOBBY | Off | Off | Off | Off | — |
| `DraftInitial` | Phase = DRAFT_INITIAL | On | Off | Off | On | — |
| `DraftShop` / `DraftAuction` | Phase = DRAFT_SHOP or DRAFT_AUCTION | On | Off | Off | On | — |
| `Placement` | Phase = PLACEMENT | On (own spawn cells) | On | Off | On | — |
| `ResolutionReveal` | `S2CPlacementReveal` received | Off | Despawned | Pending | On | **Active (250ms scale + alpha on newly-replicated opponent entities)** |
| `Reconnecting` | Awaiting `S2CObjectiveIdentities` after snapshot | Off | Off | Off | On (no live-update) | — |
| `ResolutionExecuting` | Animation queue draining | Off | Off | Active | On (live-update) | — |
| `ResolutionObjectiveReveal` | Queue exhausted; objective VFX playing | Off | Off | Off | Frozen | — |
| `GameOver` | Phase = GAME_OVER | Off | Off | Off | Frozen | — |

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
| **Game Config** | Config → Rendering | `lane_count=5` and `cells_per_lane=8` confirm board grid dimensions at startup; animation timing constants (`resolution_sub_step_duration_ms`, `pre_animation_pause_ms`, `inter_step_pause_ms`, `unit_reveal_tween_duration_ms`, `objective_reveal_hold_ms`, `objective_reveal_anim_ms`, `resolution_reveal_timeout_ms`, `objective_identities_reconnect_timeout_ms`) loaded from `GameConfig` resource |
| **Hand UI** | Hand UI → Rendering | `GhostPlacementChanged { target: Option<PlayTarget>, card_id: Option<CardId> }` message written by Hand UI; Board Rendering reads it to spawn/move/despawn the variant-appropriate ghost preview (see Rule 8 table). Hand UI reads `Res<BoardLayout>` for cell-position mapping. |
| **Hand UI** | Rendering → Hand UI | `GhostClickedEvent { card_id }` and `GhostDragStartEvent { card_id }` intra-client messages emitted on player interaction with ghost preview entities (see Rule 8 reverse events); Hand UI consumes both to drive un-staging. |
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
// PRECONDITION 1: hp_max >= 1
//   - hp_max == 0 produces 0.0/0.0 = NaN; clamp(NaN, 0.0, 1.0) = NaN;
//     scale.x = NaN renders an invisible/degenerate sprite with no Bevy error.
//   - The implementation MUST guard at intake (replication ingestion):
//       let hp_max_safe = hp_max.max(1);
//       if hp_max == 0 { warn!("UnitStats.hp_max=0 from server; clamped to 1"); }
//   - Friend-game policy: silent clamp + warning, do NOT panic. Log captures the
//     server-contract violation; client keeps rendering.
//
// PRECONDITION 2: health_bar_green_threshold > health_bar_red_threshold
//   - If config injects green=0.5, red=0.6, the if-else falls through and
//     everything below 0.5 fill is Red — Yellow band disappears.
//   - The implementation MUST validate at config-load time:
//       assert!(red_threshold < green_threshold,
//           "HP threshold config invalid: red_threshold={} >= green_threshold={}",
//           red_threshold, green_threshold);
//   - This is an `assert!`, not `debug_assert!` — fires in release.
//
// FLOATING-POINT BOUNDARY: integer ratios at thresholds (e.g. 3/10=0.29999... f32)
// drift below the conceptual boundary. Use a small epsilon to make boundaries
// inclusive on the "good" side:
//
//   const HP_THRESHOLD_EPSILON: f32 = 1e-4;

fill = clamp(hp_current as f32 / hp_max_safe as f32, 0.0, 1.0)

bar_color = if fill >= health_bar_green_threshold - HP_THRESHOLD_EPSILON { Green }
            else if fill >= health_bar_red_threshold - HP_THRESHOLD_EPSILON { Yellow }
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

**Examples (all with `HP_THRESHOLD_EPSILON = 1e-4`):**
- `hp_current=2, hp_max=5` → fill=0.40 → Yellow (≥ red 0.3 − ε; < green 0.6 − ε)
- `hp_current=5, hp_max=5` → fill=1.00 → Green
- `hp_current=1, hp_max=5` → fill=0.2000... ≥ (0.3 − 1e-4) → **NO**, → Red (just below red threshold; epsilon does not bridge a 0.1 gap)
- `hp_current=3, hp_max=10` → fill=0.29999...f32 → without epsilon: Red. **With epsilon (`fill ≥ 0.3 − 1e-4 = 0.2999`):** 0.29999 ≥ 0.2999 → Yellow. **This is the bug fix.**
- `hp_current=0, hp_max=5` → fill=0.00 → Red, scale.x=0.0 (bar invisible; unit despawning same tick)
- `hp_current=3, hp_max=0` (server bug) → hp_max clamped to 1 → fill=clamp(3.0, 0.0, 1.0)=1.0 → Green + warn!() logged
- `green=0.5, red=0.6` (inverted config) → intake `assert!` fires at config load with diagnostic message → game refuses to start with bad config (R2 invariant)

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

### F4 — Resolution Animation Total Duration (revised R2 2026-04-29)

The `resolution_animation_duration` formula now includes the reveal tween (Rule 7) and the objective-reveal sequence (Rule 12) so the **wall-clock time from `S2CPlacementReveal` to next-phase entry** is fully accounted for:

```
total_ms = unit_reveal_tween_duration_ms                                // Rule 7 reveal tween
         + pre_animation_pause_ms                                       // hold before sub-step 1
         + N_groups * (resolution_sub_step_duration_ms + inter_step_pause_ms)
         + N_destroyed * (objective_reveal_hold_ms + objective_reveal_anim_ms)
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Unit reveal tween | `unit_reveal_tween_duration_ms` | u32 | 150–400 | Scale + alpha tween on opponent's newly-replicated entities at `ResolutionReveal` entry; default **250ms** (R2 new). Sequential before pre-animation pause. |
| Pre-animation pause | `pre_animation_pause_ms` | u32 | 200–800 | Hold after reveal tween completes, before sub-step 1 begins; default 400ms. **Sequential** with reveal tween (R2 — was concurrent with fog lift). |
| Sub-step duration | `resolution_sub_step_duration_ms` | u32 | 400–1000 | Active animation window per sub-step group; default 600ms |
| Inter-step pause | `inter_step_pause_ms` | u32 | 100–300 | Silent pause between consecutive groups; default 150ms |
| Group count | `N_groups` | u8 | 0–6 | Count of distinct `sub_step` values present in `S2CResolutionEvent` |
| Objective reveal hold | `objective_reveal_hold_ms` | u32 | 300–800 | Silent suspense beat per destroyed objective; default 500ms |
| Objective reveal anim | `objective_reveal_anim_ms` | u32 | 500–1000 | Reveal animation per destroyed objective (max of real/fake variants); default **800ms** |
| N destroyed | `N_destroyed` | u8 | 0–5 | Count of objectives destroyed this RESOLUTION |

**Output Range (defaults):**
- Minimum (N_groups=0, N_destroyed=0): 250 + 400 + 0 + 0 = **650ms**
- Typical (N_groups=6, N_destroyed=0): 250 + 400 + 6×(600+150) + 0 = **5,150ms** (~5.15 s)
- Typical with 1 destroy: 5,150 + 1×(500+800) = **6,450ms**
- **Maximum (all sub-steps + 2 destroys, tuning ceiling): 400 + 800 + 6×(1000+300) + 2×(800+1000) = 11,400ms ≈ 11.5s**
- Theoretical with 5 destroys (extremely rare — all objectives lost in one round): up to ~14.6s — flagged as out-of-scope; cap `N_destroyed` rendering to 2-per-RESOLUTION at the worst case (consolidate remaining as "+N more destroyed" summary, deferred polish).

**Across a 20-round friend-game match** at default timings (assume 1 destroy per ~5 rounds): ≈ 4 destroys × 1.3s = 5.2s extra → 20 × 5.15 + 5.2 ≈ **108s** total locked watch time, vs. the pre-R2 estimate of 98s. The additional 10s is the cost of the reveal tween (5s) + per-match objective reveals (~5s).

**Player Fantasy ceiling: ≤11.5s absolute (single RESOLUTION worst case).** Going beyond requires a Player Fantasy revisit. The "≤5s default" framing is preserved for typical rounds (no destroys). Veterans-fatigue trade-off acknowledged in Player Fantasy.

**N_groups=0 behavior.** When `S2CResolutionEvent` arrives with an empty event list (no events fired in any sub-step — rare but possible if both players placed only structures or skipped placement): the reveal tween runs (if opponent had any new entities), then the pre-animation pause runs (the player gets the dramatic post-reveal beat), no sub-step Tweens are spawned, then transition directly from `ResolutionExecuting` → `ResolutionObjectiveReveal`. `total_ms = unit_reveal_tween_duration_ms + pre_animation_pause_ms` (plus objective reveals if any).

**Example:** A round with events in sub-steps 1, 5, 6 (N_groups=3) and 0 destroys, defaults: `total_ms = 250 + 400 + 3×(600+150) = 2,900ms`.

## Edge Cases

**EC-SUBSTEP-OOR — If `S2CResolutionEvent` contains a `sub_step` value outside [1–6]:** treat as a fatal protocol desync. Discard the entire `AnimQueue` (clear all groups), clear `PendingResolutionScript` if present, log error containing `sub_step` value, and request a fresh `S2CGameSnapshot` from the server via C2S `RequestSnapshot`. Do NOT silently skip the group — silent skip corrupts subsequent state references because surrounding sub-steps assume the dropped group's outputs (HP changes, position changes, gold awards) were applied. Snapshot recovery is clean. This aligns with network-protocol.md client contract for protocol violations.

**EC-PHASE-BUFFER — If `S2CPhaseChanged(DRAFT_SHOP)` arrives while in `Placement`, `ResolutionReveal`, `ResolutionExecuting`, or `ResolutionObjectiveReveal`:** store in `PendingPhaseChange` (last-write-wins on duplicate); apply the transition only after `ResolutionObjectiveReveal` completes. The player must always see the full resolution sequence regardless of which RESOLUTION sub-state was active when the message arrived.

**EC-PHASE-GAMEOVER — If `S2CPhaseChanged(GAME_OVER)` arrives mid-`ResolutionExecuting`:** complete the current `AnimGroup`, skip remaining groups, execute `ResolutionObjectiveReveal` for any buffered `ObjectiveDestroyed` events, then transition to `GameOver`. Never skip the objective reveal — it is the mandatory emotional beat.

**EC-RECONNECT-RESOLUTION — If `S2CGameSnapshot` arrives mid-RESOLUTION (reconnect):** discard all in-progress animation state (clear `AnimQueue`, `PendingPhaseChange`, `PendingResolutionScript`; cancel all active `Animator<*>` components), despawn all board entities, rebuild the full board from snapshot in one frame. **If `snapshot.phase == RESOLUTION`, target state is `DraftShop` regardless of whether any `S2CResolutionEvent` has been received** (animation is never replayed for reconnecting clients — the snapshot delivers the final state directly). After rebuild, hold in a `Reconnecting` sub-state until `S2CObjectiveIdentities` is re-received (per ADR-001), then enter the actionable phase.

**EC-RESOLUTION-REVEAL-STUCK — If `S2CPlacementReveal` was received but `S2CResolutionEvent` does not arrive within `resolution_reveal_timeout_ms` (default 2000ms):** request a fresh `S2CGameSnapshot` via `C2SRequestSnapshot` and reset `BoardRenderState` to whatever the snapshot delivers. This is the only message-loss fallback; without it the client is permanently stuck on a revealed-but-static board with no input. For true server crashes, Lightyear's 30s heartbeat-disconnect grace is the actual last resort. The `C2SRequestSnapshot` contract is currently undefined in `network-protocol.md` — see OQ-BR-06 (BLOCKING).

**EC-NGROUPS-ZERO — If `S2CResolutionEvent` has N_groups=0 (no events):** the reveal tween still runs (Rule 7) if opponent had any newly-replicated entities; `pre_animation_pause_ms` then runs (player gets the dramatic post-reveal beat); no sub-step Tweens are spawned; transition directly from `ResolutionExecuting` → `ResolutionObjectiveReveal`. `AnimQueue.total_duration_ms = unit_reveal_tween_duration_ms + pre_animation_pause_ms` (plus objective reveals if any).

**EC-OBJ-HP-ZERO — If `ObjectiveHp` replicates a value of 0 while `ResolutionExecuting` is active:** the HP bar clamps to 0 (F2 saturating clamp; scale.x=0 → bar invisible — acceptable since the objective despawns when `ObjectiveDestroyed` fires in `ResolutionObjectiveReveal`).

**EC-MULTI-OBJ — If two objectives are destroyed in the same RESOLUTION:** reveal in ascending lane order, sequentially. Each reveal plays its full 500ms hold → reveal animation → slot clear before the next lane begins. Implementation MUST sort incoming `ObjectiveDestroyed` messages by `lane` before queuing — message arrival order is not guaranteed.

**EC-COOCCUPANT-DEATH — If a co-occupying allied unit dies mid-RESOLUTION:** the surviving unit must return to cell center. Call `animator.set_tweenable(snap_to_center_tween)` on the surviving unit's existing `Animator<Transform>` to replace the in-flight tween with a 0ms snap-to-center. Do NOT despawn-and-respawn the entity (loses game-state components). Do NOT write `Transform.translation` directly while a tween is active on the same entity (BR-16 invariant). 2v2 only — does not occur in 1v1.

**EC-INVALID-GHOST — If the ghost unit is hovered to an invalid cell (outside spawn range, or Minion slot occupied):** the ghost stays at the last valid cell; the invalid cell node shows a brief red tint. The ghost does not move to the invalid cell.

**EC-REVEAL-WAIT — If `S2CPlacementReveal` arrives before `S2CResolutionEvent`:** enter `ResolutionReveal`, run the reveal tween (Rule 7) on newly-replicated opponent entities, then start `pre_animation_pause_ms`. Wait for `S2CResolutionEvent` for up to `resolution_reveal_timeout_ms` (default 2000ms — see EC-RESOLUTION-REVEAL-STUCK for fallback). When the event arrives, transition to `ResolutionExecuting`. Log a warning if the wait exceeds the timeout.

**EC-EVENT-EARLY — If `S2CResolutionEvent` arrives before `S2CPlacementReveal`** (reliable channel ordering anomaly — see NP-OQ-3): store in `PendingResolutionScript`; do not begin any animation. When `S2CPlacementReveal` arrives, run the reveal tween AND consume the buffered script — enter `ResolutionExecuting` after the reveal tween completes (still get the reveal beat — the buffered script doesn't change Rule 7's reveal sequence). The `pre_animation_pause_ms` IS still applied because the reveal tween itself isn't the dramatic pause; it's the entity appearance (R2 design decision: do not skip pre_animation_pause on EC-EVENT-EARLY — preserves Player Fantasy beat). `PendingResolutionScript` is cleared on consumption. Log a warning.

**EC-PHASE-EARLY — If `S2CPhaseChanged(DRAFT_SHOP)` arrives before `S2CResolutionEvent`** (channel ordering anomaly that would silently drop the entire animation if not buffered): store in `PendingPhaseChange` (per Rule 10) regardless of current state. Continue waiting for `S2CResolutionEvent`. If the 2000ms ResolutionReveal timeout fires first, EC-RESOLUTION-REVEAL-STUCK fallback runs and snapshot recovery overrides the buffer. Without this rule the animation would be silently dropped on channel-ordering bugs.

**EC-CARD-MISS — If a unit's `card_id` has no matching entry in the local card asset pool at spawn time** (stale client assets): render the `ui_unit_placeholder_48x64` sprite (solid-color tile + "?" glyph) at the correct cell. HP bar still renders using replicated `UnitStats`. Log an asset-miss warning. Never panic or skip the entity spawn.

**EC-OBJ-MISS — If `ObjectiveDestroyed` arrives for a lane where no objective entity currently exists on the client** (replication removed it before the reliable message was processed): suppress the destruction VFX; update spawn range highlights immediately; log a warning. Do not spawn a temporary entity — this risks double-reveal if the replicated entity arrives late.

**EC-GHOST-DESELECT — If `GhostPlacementChanged { target: None, card_id: Some(_) }` arrives and no ghost entity/marker exists for that `card_id`** (deselect event after ghost was already cleared by `S2CPlacementReveal`, or duplicate clear): no-op. Use the Bevy 0.18 safe pattern: `if let Some(mut ec) = commands.get_entity(e) { ec.despawn(); }`. The pre-0.16 pattern `commands.get_entity(e).map(EntityCommands::despawn)` does not compile in 0.18 (borrowck on `EntityCommands`). Marker-component variants (TargetUnit, TargetObj) clear the marker if present and no-op if absent.

**EC-HP-ZERO — If a live unit reaches `hp_current=0`:** F2 produces `fill=0.0` and `scale.x=0.0` (bar invisible). This is intentional — the unit despawns synchronously in the same tick (sub-step 5). The "HP bars always visible" invariant (Rule 6, BR-5) applies to live units only. Tests must construct fixtures with `hp_current > 0` to assert visibility.

**EC-HP-MAX-ZERO — If `UnitStats.hp_max=0` arrives via replication** (server-contract violation; should not occur): clamp to 1 at intake (`hp_max_safe = hp_max.max(1)`), log warning. Do NOT panic — friend-game resilience prefers graceful degradation. Bar fills based on the clamped value.

**EC-RECONNECT-2V2 — If one player in a 2v2 match reconnects mid-RESOLUTION:** non-reconnecting clients keep animating uninterrupted (their `S2CResolutionEvent` is unaffected by the peer's reconnect). The reconnecting client snapshots-then-fast-forwards to `DraftShop` per EC-RECONNECT-RESOLUTION; it does not try to catch up to the live animation.

## Dependencies

### Upstream Dependencies

| System | Type | Interface |
|---|---|---|
| **Board / Lane System** (Approved) | Hard | Lightyear replicates `BoardPosition { lane, cell }` and `UnitStats { hp_current, hp_max, owner }` per unit entity to the client; Board Rendering queries these components each frame to drive sprite positions and HP bar fill |
| **Objective System** (Approved) | Hard | Lightyear replicates `ObjectiveHp { hp }` per objective; `ObjectiveDestroyed { target_player_id, lane, was_fake }` reliable message drives the destruction reveal sequence in `ResolutionObjectiveReveal` |
| **Combat Resolution** (Designed) | Hard | Resolution sub-step event data arrives via `S2CResolutionEvent` (owned by Network Protocol); Board Rendering has no direct interface with Combat Resolution |
| **Network Protocol** (Approved) | Hard | `S2CPlacementReveal` → opponent entities arrive via Lightyear replication + reveal tween; `S2CResolutionEvent` → animation queue; `S2CPhaseChanged` → all `BoardRenderState` transitions; `S2CGameSnapshot` → full board rebuild on connect/reconnect; **`C2SRequestSnapshot` (currently undefined per OQ-BR-06)** → only client recovery path |
| **Card Data & Pool** (Approved) | Hard | `TextureAtlas` asset loaded at startup; slice index looked up by `card_id` at unit spawn time; fallback to placeholder sprite if `card_id` is missing (EC-12) |
| **Game Config** (Approved) | Hard | `lane_count=5` and `cells_per_lane=8` confirm board grid dimensions at startup; animation timing constants (`board_sub_step_duration_ms`, `board_fog_lift_ms`, `board_pre_anim_pause_ms`, `board_inter_step_pause_ms`, `board_objective_reveal_hold_ms`) and visual tuning (`board_fog_opacity`, `board_cell_width`, `board_lane_height`, `board_hp_*_threshold`, `board_co_occupancy_offset`, `board_prism_spin_speed`) loaded from `GameConfig` resource (added to game-config.md 2026-04-30) |

### Peer Presentation Systems (same layer — no hard dependency, shared resource)

| System | Direction | Interface |
|---|---|---|
| **Hand UI** | Hand UI ↔ Rendering | Hand UI writes `GhostPlacementChanged { target: Option<PlayTarget>, card_id: Option<CardId> }` messages — Board Rendering reads them to manage variant-specific ghost previews per Rule 8. Board Rendering writes `GhostClickedEvent { card_id }` and `GhostDragStartEvent { card_id }` — Hand UI reads them to drive un-staging. Hand UI reads `Res<BoardLayout>` for cell-to-world coordinate lookup. |
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
| `resolution_sub_step_duration_ms` | `board_sub_step_duration_ms` | 600 ms | 400–1000 | Sub-steps blur together | Resolution drags; watching becomes idle dead time. Default revised down 2026-04-30 (was 800); ceiling tightened to 1000 (was 1500) to defend Player Fantasy ≤5s default. |
| `inter_step_pause_ms` | `board_inter_step_pause_ms` | 150 ms | 100–300 | No breathing room; feels rushed | Resolution stalls between steps. Default revised down 2026-04-30 (was 200); ceiling tightened to 300 (was 400). |
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

📌 **Visual Identity Anchor compliance (OQ-BR-08)** — The fog easing curve, unit oscillation profile, objective reveal flash, and prism rotation rate must all be cross-referenced against the project's Visual Identity Anchor (defined in `design/gdd/game-concept.md` and the art bible) before the M2 VFX polish stories begin. Specifications in this section are timing-and-magnitude only; the aesthetic character (sharp/soft, paper/metal/light) is owned by the anchor.

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

**Upstream dependencies (some ACs gated on external resolution):**
- **OQ-BR-03** — `ResolutionEvent` enum variants are owned by combat-resolution.md. ACs that construct `S2CResolutionEvent` payloads (BR-12, BR-14, BR-15) cannot be implemented until variants are finalized; until then these ACs are testable against the queue-grouping math using `sub_step` values only (the events themselves can be opaque placeholders for the duration test).
- **NP-OQ-3** — Lightyear 0.26 reliable channel ordering across message types. EC-EVENT-EARLY and EC-PHASE-EARLY assume ordering may be violated; if NP-OQ-3 confirms strict FIFO across types, those edge cases become unreachable but the buffer logic remains as a defense-in-depth invariant.

| ID | Criterion | Classification |
|---|---|---|
| BR-1 | GIVEN a `BoardLayout` with default `cell_width=64.0` and `lane_height=80.0`, WHEN the board initializes, THEN exactly 40 `CellNode` entities exist, each carrying a `Transform` whose `translation.xy` matches `cell_to_world(lane, cell)` for every (lane, cell) in [1–5]×[1–8]. No two entities share the same world position. | BLOCKING |
| BR-2 | GIVEN `board_origin=(0.0, 0.0)`, `cell_width=64.0`, `lane_height=80.0`, WHEN `cell_to_world` is called for (1,1), (5,8), and (3,5), THEN the returned `Vec2` values are `(0.0, 0.0)`, `(448.0, −320.0)`, and `(256.0, −160.0)` respectively (tolerance ≤0.01). | BLOCKING |
| BR-2b | GIVEN F1's precondition `1 <= lane <= 5 && 1 <= cell <= 8`, WHEN `cell_to_world` is called with `lane=0`, `cell=0`, `lane=6`, or `cell=9`, THEN the call panics with a message containing the offending input value. The panic uses `assert!`, not `debug_assert!`, so it fires in release builds. | BLOCKING |
| BR-3 | GIVEN a fully-populated board (5 lanes, 2v2 co-occupancy, all objectives present, fog active), WHEN the frame is rendered in a live WASM browser build, THEN total GPU draw call count is ≤15. Evidence: browser devtools performance panel screenshot in `production/qa/evidence/`. | ADVISORY |
| BR-3a | GIVEN a fully-populated board after spawn, WHEN all entities with a unit `Sprite` component are queried, THEN every such entity carries the same `Handle<TextureAtlas>` `AssetId` (single unit atlas). When all board-rendering sprite entities are queried, exactly 2 distinct `AssetId<TextureAtlas>` values are present (unit atlas + board-elements atlas; standalone-atlas sprites like background, Field washes count separately and are tracked in BR-3). | BLOCKING |
| BR-3b | GIVEN a fully-populated board after spawn, WHEN all unit entities are queried, THEN no entity carries a `Handle<ColorMaterial>`, `Handle<StandardMaterial>`, or any custom per-unit material handle. All unit tinting uses `Sprite.color` (vertex-data tint) only. This is the sentinel against the most common batch-breaking implementation error. | BLOCKING |
| BR-4 | GIVEN a unit with `UnitStats { hp_current, hp_max }` (with `hp_max ≥ 1` precondition enforced at intake), WHEN fill is computed, THEN: (a) 5/5 → fill=1.0, Green; (b) 2/5 → fill=0.4, Yellow; (c) 1/5 → fill=0.2, Red; (d) 0/5 → fill=0.0, Red, scale.x=0.0 (bar invisible — see EC-HP-ZERO); (e) 6/5 (overflow) → fill=1.0, Green; (f) `hp_max=0` server input → clamped to `hp_max=1` at intake + warn!() logged → fill computed against clamped value. HP bar child entity `Transform.scale.x` matches fill ±0.01 for cases (a)–(c) and (e). | BLOCKING |
| BR-5 | GIVEN any unit entity present on the board with `hp_current > 0` (live unit), REGARDLESS of `BoardRenderState` (except `Idle`), THEN the unit's child HP bar entity has `Visibility::Inherited` or `Visibility::Visible` (never `Visibility::Hidden`). The "Frozen" state in `ResolutionObjectiveReveal` and `GameOver` means fill values do not update — bars remain visible. Units at `hp_current=0` are exempt (despawning same tick — EC-HP-ZERO). | BLOCKING |
| BR-6 | GIVEN `BoardRenderState` transitions to `Placement`, THEN: (a) exactly two fog `Sprite` entities exist tagged with a fog marker; (b) opponent-half fog has `Visibility::Visible` and `Sprite.color.alpha ≥ 0.55`; (c) local-player-half fog has `Visibility::Hidden`. The same two entity IDs are reused on subsequent PLACEMENT entries (not respawned) — verifiable across two PLACEMENT cycles. | BLOCKING |
| BR-7 | GIVEN `BoardRenderState` is `Placement` with both fog entities at `alpha=0.6`, WHEN `S2CPlacementReveal` is received and the system runs, THEN: (a) within the same `App::update()` tick both fog entities have an `Animator<Sprite>` component in `AnimatorState::Playing`; (b) after injecting `fog_lift_duration_ms` of `Time<Virtual>` delta and calling `App::update()` to convergence, both fog entities have `Sprite.color.alpha == 0.0` ±0.01. | BLOCKING (reclassified from ADVISORY 2026-04-30 — testable headlessly with `Time<Virtual>` injection) |
| BR-8 | GIVEN a `GhostPlacementChanged { target: Some(BoardCell{...}), card_id }` event is received while a `GhostUnit` entity for the same `card_id` already exists, WHEN the system processes it, THEN exactly one `GhostUnit` marker entity exists in the World after the next `apply_deferred` flush (the prior ghost is replaced, not duplicated). | BLOCKING |
| BR-8b | GIVEN `GhostPlacementChanged { target: Some(TargetUnit{lane, unit_id}), card_id }` is received, WHEN the system processes it, THEN the unit entity matching `unit_id` has a `TargetUnitGhost` marker component. No new entity is spawned. | BLOCKING |
| BR-8c | GIVEN `GhostPlacementChanged { target: Some(LaneWide{lane}), card_id }` is received, WHEN the system processes it, THEN exactly one `LaneGhostWash` entity exists covering the column of `lane`. | BLOCKING |
| BR-8d | GIVEN `GhostPlacementChanged { target: Some(Instant), card_id }` is received, WHEN the system processes it, THEN no ghost entity is spawned and no marker components are added (Instant ghosts are owned by Hand UI's fan slot, not the board). | BLOCKING |
| BR-8e | GIVEN any ghost preview entity (or marker) exists for `card_id` and the player clicks it, WHEN the click is processed, THEN exactly one `GhostClickedEvent { card_id }` message is written. The ghost is NOT removed by Board Rendering — Hand UI's subsequent `GhostPlacementChanged { target: None, card_id }` drives removal. | BLOCKING |
| BR-9 | GIVEN one `GhostUnit` entity exists, WHEN `S2CPlacementReveal` is received, THEN the `GhostUnit` entity is despawned within the same frame (zero `GhostUnit` entities after system runs). | BLOCKING |
| BR-10 | GIVEN no ghost preview entity or marker exists for `card_id`, WHEN `GhostPlacementChanged { target: None, card_id: Some(card_id) }` is received, THEN no panic occurs, no entity is spawned, and World state is unchanged. The despawn path uses `if let Some(mut ec) = commands.get_entity(e) { ec.despawn(); }` (Bevy 0.18 safe pattern); the prior `commands.get_entity(e).map(EntityCommands::despawn)` form does NOT compile in 0.18 and must not be used. Marker-component variants are removed if present, no-op if absent. | BLOCKING |
| BR-11 | GIVEN a ghost unit entity is spawned, WHEN inspected in the ECS World, THEN: (a) `Sprite.color.alpha = 0.5`; (b) no child entity has a HP bar marker; (c) no `Replicated` component is present; (d) `Transform.translation.xy` matches `cell_to_world(lane, cell)` for the target cell. | BLOCKING |
| BR-12 | GIVEN a `S2CResolutionEvent` with sub_step values [1, 1, 3, 3, 3, 5] (events themselves are opaque placeholders pending OQ-BR-03), WHEN the animation queue is built, THEN `AnimQueue.groups.len() == 3` sorted ascending [1, 3, 5] with event counts [2, 3, 1] respectively; `AnimQueue.total_duration_ms()` equals `pre_animation_pause_ms + 3*(resolution_sub_step_duration_ms + inter_step_pause_ms)` ±1ms. The `AnimQueue` Resource exposes `total_duration_ms()` as a callable method without requiring a running `App`. | BLOCKING |
| BR-13 | GIVEN `N_groups=0` (empty `S2CResolutionEvent`), WHEN the queue is built, THEN `AnimQueue.groups.is_empty()`, `AnimQueue.total_duration_ms() == pre_animation_pause_ms`, and the state machine still runs `pre_animation_pause_ms` of pause before transitioning `ResolutionExecuting` → `ResolutionObjectiveReveal`. No Tweens are spawned during this transition. | BLOCKING |
| BR-14 | GIVEN the board is in `ResolutionExecuting` with 3 `AnimGroup`s queued, WHEN `S2CPhaseChanged(DRAFT_SHOP)` is inserted as a Message and `App::update()` is called, THEN: (a) `BoardRenderState` remains `ResolutionExecuting` (not `DraftShop`); (b) `PendingPhaseChange` resource holds `Some(DRAFT_SHOP)`; (c) after advancing simulated time past all 3 group durations via repeated `App::update()` with injected `Time<Virtual>` deltas and after `ResolutionObjectiveReveal` completes, `BoardRenderState == DraftShop` and `PendingPhaseChange == None`. | BLOCKING |
| BR-15 | GIVEN the board is in `ResolutionExecuting` executing group 2 of 4, WHEN `S2CPhaseChanged(GAME_OVER)` is inserted, THEN after time-stepping past group 2's duration: (a) groups 3 and 4 are discarded (`AnimQueue.current_index >= AnimQueue.groups.len()`); (b) `ResolutionObjectiveReveal` runs for buffered `ObjectiveDestroyed` events; (c) `BoardRenderState == GameOver` after objective reveal completes. Board never transitions through `DraftShop`. | BLOCKING |
| BR-16 | GIVEN a unit entity has an `Animator<Transform>` in `AnimatorState::Playing`, WHEN the reposition system is invoked with a new target cell, THEN: (a) the entity's `Animator<Transform>` is replaced via `set_tweenable()` (the new tween is now playing, the old one is no longer active); (b) the entity's `Transform.translation` is never set to the raw `cell_to_world(target)` value directly between the cancellation and the new tween's first interpolated frame. | BLOCKING |
| BR-17 | GIVEN the board is in any state with N entities, WHEN `S2CGameSnapshot` is received, THEN: (a) all prior board entities are despawned within the same `App::update()` tick; (b) rebuilt entity count and component values match snapshot data exactly; (c) zero `Animator<Transform>` and zero `Animator<Sprite>` components from the prior state remain in the World after rebuild (verify via World query over all `Animator<*>`); (d) `AnimQueue`, `PendingPhaseChange`, `PendingResolutionScript` resources are all cleared. Operation completes in a single tick. | BLOCKING |
| BR-18 | GIVEN `S2CGameSnapshot { phase: RESOLUTION }` is received, WHEN rebuild completes, THEN `BoardRenderState == DraftShop` (animation is never replayed for reconnecting clients per Rule 11). The system holds in a `Reconnecting` sub-state until `S2CObjectiveIdentities` is re-received per ADR-001, then enters `DraftShop`. | BLOCKING |
| BR-19 | GIVEN 5 objective entities in the World in any state prior to `ObjectiveDestroyed`, WHEN inspected, THEN: (a) every objective entity uses the `env_objective_unknown_64x96` atlas frame index; (b) the set of `ComponentId`s on each of the 5 entities is structurally identical (`World::inspect_entity()` returns the same component types for all 5); (c) a World query for any component type whose name contains "Real", "Fake", "Identity", "Known", or "IsTrue" returns zero results. This is the runtime ADR-001 isolation invariant. | BLOCKING |
| BR-20 | GIVEN `ObjectiveDestroyed { was_fake: false, lane: 3 }` is received during `ResolutionObjectiveReveal`, THEN: (a) after `App::update()` with 0ms time advance, the lane-3 objective entity is still present and shows the unknown frame; (b) after injecting `objective_reveal_hold_ms` (500ms) of `Time<Virtual>` delta, a real-reveal marker is present on the objective entity (atlas frame swapped to `env_objective_real_reveal_64x96` OR a `RevealFlash` marker component is attached); (c) after injecting the flash animation duration (~300ms) plus buffer, the objective entity is despawned; (d) spawn range highlight components on affected `CellNode` entities reflect the updated range within the same tick as the despawn. | BLOCKING (reclassified from ADVISORY 2026-04-30 — testable headlessly with `Time<Virtual>` injection) |
| BR-21 | GIVEN `ObjectiveDestroyed` for lanes 4 and 2 in the same RESOLUTION (delivered to the system in arbitrary message order), WHEN `ResolutionObjectiveReveal` processes them, THEN lane 2's reveal (500ms hold + reveal + clear) fully completes before lane 4's reveal begins. Verify by asserting: at the tick when lane 2's objective entity is despawned, lane 4's objective entity still carries the unknown atlas frame and has no reveal marker component. Implementation MUST sort by lane ascending before processing — message arrival order is not assumed. | BLOCKING |
| BR-22 | GIVEN two allied units at the same (lane, cell) with `co_occupancy_side_offset=8.0`, WHEN render positions are computed, THEN unit_index=0 → `x_offset=−4.0`; unit_index=1 → `x_offset=+4.0`. The two units' `Transform.translation.x` values differ by exactly `co_occupancy_side_offset` (8.0 world units). | BLOCKING |
| BR-22b | GIVEN F3's precondition `unit_index <= 1`, WHEN F3 is called with `unit_index=2` (server bug — three co-occupants), THEN the call panics with a message containing the offending value. Uses `assert!` (not `debug_assert!`) to fire in release. | BLOCKING |
| BR-23 | GIVEN two allied units at the same cell (indices 0 and 1) with active `Animator<Transform>` on both, WHEN unit_index=0 dies mid-RESOLUTION, THEN the surviving unit (index=1): (a) has its `Animator<Transform>` replaced via `animator.set_tweenable(snap_to_center_tween)`; (b) the new tween's target translation has `x_offset=0.0` (cell center); (c) `Transform.translation.x` is never written directly while an `Animator<Transform>` is active on the entity. | BLOCKING |
| BR-24 | GIVEN a `S2CResolutionEvent` containing `sub_step=7` (out of range), WHEN the animation queue is built, THEN: (a) the entire `AnimQueue` is discarded (treated as fatal protocol desync); (b) a `RequestSnapshot` C2S message is enqueued for transmission; (c) an error is logged containing "sub_step" and the offending value; (d) no panic occurs. This aligns with network-protocol.md client contract — silent skip is forbidden. | BLOCKING (semantics changed 2026-04-30 from "skip + log" to "fatal desync + snapshot recovery") |
| BR-Z-LOCAL | GIVEN a unit entity at `Transform.translation.z = Z_UNITS (3.0)` with an HP bar child entity, WHEN the HP bar child's `Transform` is inspected, THEN its `translation.z` is `0.1` (LOCAL — added to parent's Z to produce global Z=3.1=Z_HEALTH_BARS). The HP bar child's local Z must NOT equal `Z_HEALTH_BARS` directly (would render at global Z=6.1, above fog). | BLOCKING |
| BR-EC-EARLY | GIVEN `BoardRenderState` is `Placement` and no `S2CPlacementReveal` has arrived, WHEN `S2CResolutionEvent` arrives, THEN: (a) `BoardRenderState` remains `Placement`; (b) `PendingResolutionScript` holds `Some(_)`; (c) when `S2CPlacementReveal` subsequently arrives, fog lift begins immediately AND `ResolutionExecuting` is entered with NO `pre_animation_pause_ms` hold; (d) a warning is logged. | BLOCKING |
| BR-EC-CARDMISS | GIVEN a unit spawn event carries `card_id=CardId(9999)` (no matching entry in card asset pool), WHEN the spawn system processes it, THEN: (a) exactly one entity is spawned at `cell_to_world(lane, cell)`; (b) `Sprite` uses the `ui_unit_placeholder` atlas frame; (c) the entity's HP bar child exists with visibility per BR-5; (d) no panic; (e) a warning containing "card_id" or "asset-miss" is logged. | BLOCKING |
| BR-EC-OBJMISS | GIVEN no objective entity exists for lane 3 (replicated removal arrived before reliable message), WHEN `ObjectiveDestroyed { lane: 3, was_fake: false }` arrives, THEN: (a) no new entity is spawned; (b) spawn range highlight components on `CellNode` entities reflect the updated range; (c) no panic; (d) a warning is logged. World entity count is unchanged. | BLOCKING |
| BR-EC-STUCK | GIVEN `BoardRenderState` is `ResolutionReveal` and 2000ms has elapsed without `S2CResolutionEvent`, WHEN the system runs, THEN a `RequestSnapshot` C2S message is enqueued and a warning containing "ResolutionReveal stuck" is logged. After the resulting `S2CGameSnapshot` arrives, the board rebuilds per Rule 11. | BLOCKING |
| BR-2-ATLAS | GIVEN a fully-populated board with units, objectives, cell nodes, prism token, and fog, WHEN all `Handle<TextureAtlas>` components on board-rendering entities are collected (excluding standalone-atlas sprites: background, Field wash), THEN exactly 2 distinct `AssetId<TextureAtlas>` values are present. A third atlas indicates a regression. | BLOCKING |
| BR-FOG-OPACITY | GIVEN a `GameConfig` with `board_fog_opacity` outside the safe range [0.4, 0.8] (test values: 0.0, 0.3, 0.85, 1.0), WHEN the fog entity is initialized, THEN `Sprite.color.alpha` is clamped to [0.4, 0.8] at intake and a warning is logged when the input was out of range. | BLOCKING |
| BR-FRAME-TIME | GIVEN a worst-case board state (5 lanes active, maximum unit count, all objectives standing, prism present), WHEN `ResolutionExecuting` is active and one full `AnimGroup` batch is scheduled, THEN frame time on the WASM target build remains ≤16.67ms (60fps). Evidence: browser devtools performance panel capture in `production/qa/evidence/`. | ADVISORY |
| BR-RECONNECT-TIME | GIVEN `S2CGameSnapshot` triggers a full board rebuild (Rule 11), WHEN the rebuild frame is measured on the WASM target, THEN that single frame completes within 16.67ms. Evidence: browser devtools capture. | ADVISORY |
| BR-FIRST-LOOK | GIVEN a fresh-eye observer (someone who has never played the game) is shown the board on round 1 PLACEMENT, WHEN they are asked to identify (a) their side of the board, (b) the objective row, (c) one of their own unit cells, THEN they correctly identify all three within 5 seconds without instruction. Evidence: informal friend playtest, 3 observers minimum, in `production/qa/evidence/`. | ADVISORY |

**Test file targets:**
- BLOCKING unit tests → `tests/unit/board_rendering/`
- BLOCKING state machine integration tests → `tests/integration/board_rendering/`
- ADVISORY evidence → `production/qa/evidence/board-rendering-[sprint].md`

**CI lint rules (enforced separately from AC table):**
- **Z-literal lint** (replaces former BR-25): `tools/ci/z-layer-lint.sh` greps `src/board_rendering/` for inline `f32` literals assigned to `Transform.translation.z`. CI fails the build if any match is not a named constant from `rendering_constants.rs`. Status: **BLOCKING at CI level**, not in AC table.

**Implementation notes:**
- BR-2, BR-2b, BR-4, BR-13, BR-22, BR-22b are pure function tests — no ECS `World` needed.
- BR-7, BR-14, BR-15, BR-20, BR-21, BR-EC-STUCK require time-stepped ECS via `Time<Virtual>` injection: insert `Time<Virtual>` resource, set delta manually between `App::update()` calls, assert state at each tick.
- BR-3 and BR-FRAME-TIME and BR-RECONNECT-TIME require live WASM browser builds with GPU profiling.
- BR-FIRST-LOOK requires a friend playtest (informal — friend-game scope, no formal test plan).
- BR-12, BR-14, BR-15 use opaque placeholder events for `S2CResolutionEvent` payloads pending OQ-BR-03; the queue-grouping math is fully testable with just `sub_step` integers.

## Open Questions

**OQ-BR-01 — Sang Méprise suppression signal (OPEN)**
The fake reveal sequence has a "confirmed reveal" variant (no surprise sting) when the Sang Méprise ability was active this round — the attacker already knew the identity. Board Rendering needs a signal indicating this. The delivery mechanism is undefined: replicated component, a field in `S2CResolutionEvent`, or a dedicated S2C message. The `ObjectiveIdentityCache` resource (defined in Data Structures) holds the per-objective `is_fake` data after Sang Méprise reveal; the question is how Board Rendering knows whether to read it for a given destruction reveal.
*Owner: Network Protocol GDD + Keyword System GDD. Blocking: Keyword System.*

**OQ-BR-02 — Camera specification (OPEN — recommended resolution: fixed orthographic, no pan/zoom in M2)**
The GDD specifies world-space 2D sprites but does not define the camera setup. **Decision direction**: fixed orthographic at viewport-fitting zoom, no pan, no zoom. The Board Rendering epic includes a camera-setup story. Tuning knob safe ranges (`cell_width 48–96`, `lane_height 64–112`) currently have no camera constraint check; once camera is set the ranges should be re-validated against viewport dimensions.
*Owner: Board Rendering GDD. To close: write the camera-setup story spec and add an AC asserting fixed orthographic projection.*

**OQ-BR-03 — `ResolutionEvent` enum variants (OPEN — BLOCKING for Board Rendering implementation)**
The animation queue (Core Rule 9) dispatches on `ResolutionEvent` variants, but the complete enum is not yet defined. Combat Resolution GDD flags this as OQ5. Board Rendering cannot be fully implemented until variants (`UnitMoved`, `UnitAttacked`, `UnitKilled`, `TrapTriggered`, `ObjectiveDamaged`, `PrismCollected`, etc.) are specified and registered in the Network Protocol GDD. Until then, BR-12/BR-14/BR-15 are testable using opaque placeholder events (just the sub_step values matter for queue-grouping math).
*Owner: Combat Resolution GDD + Network Protocol GDD. Blocking: Board Rendering animation dispatch implementation.*

**OQ-BR-04 — Spawn range update signal (OPEN)**
Board Rendering needs a signal when a player's spawn range expands (to update cell node highlights). The `SpawnRangeChanged` message assumed in this GDD is not yet defined in the Network Protocol GDD. **Recommended resolution:** derive it from `ObjectiveDestroyed.was_fake == true` events in the `ResolutionEvent` stream — fake destruction is the only cause of spawn range change. No new message needed. To close: confirm with NP GDD that no other cause exists.
*Owner: Network Protocol GDD. Blocking: Board Rendering spawn highlight implementation.*

**OQ-BR-05 — Unit atlas frame count and dimensions (OPEN — ESCALATED 2026-04-30 to dependency-blocker on Board Rendering implementation epic)**
The draw call budget (≤15 per frame, AC BR-3) assumes all unit sprites fit in one `TextureAtlas`. Worst-case sizing analysis: 30+ cards × 2 facings × ~48×64 px × N animation frames. At N=2 (idle frame + flipped facing), atlas fits in 2048×2048 (~3.7 MB RGBA) — viable on most WASM/WebGL2 targets. If the art bible specifies more frames per card (death animation, attack frame, status overlays), the atlas may need to split, breaking the single-draw-call claim. **Fallback policy if atlas splits**: each additional unit atlas adds 1 to the draw call ceiling (≤15 + N_additional_atlases), requiring technical-director approval. Must be resolved before the Board Rendering implementation epic is pointed.
*Owner: Art Director + Art Bible. Blocking: asset pipeline setup, M2 unit rendering story estimation.*

**OQ-BR-06 — `RequestSnapshot` C2S contract (OPEN — NEW 2026-04-30)**
Rule 11's stuck-state recovery (EC-RESOLUTION-REVEAL-STUCK) and Rule 9's fatal sub_step desync handler (EC-SUBSTEP-OOR) both require a C2S `RequestSnapshot` message to ask the server to re-send `S2CGameSnapshot`. This message is not currently defined in network-protocol.md. Without it, the client has no recovery from server crash mid-resolution or protocol-version-mismatch desyncs.
*Owner: Network Protocol GDD. Blocking: Board Rendering implementation of EC-RESOLUTION-REVEAL-STUCK and EC-SUBSTEP-OOR.*

**OQ-BR-07 — Trap face-down rendering depends on NP-OQ-2 (OPEN — NEW 2026-04-30)**
ADR-001 covers `ObjectiveIdentity` per-client visibility. Trap face-down rendering uses the same hidden-identity pattern but for `card_id` on opponent traps. Network Protocol GDD's NP-OQ-2 asks whether Lightyear 0.26 supports per-client component visibility, or whether a `TrapPresence` (visible to both) + `TrapIdentity` (visible only to owner) split is required. Until NP-OQ-2 closes, the trap rendering branch in Board Rendering cannot be specified — does it read `card_id` directly, or does it always render face-down for opponent and use a separate identity component for self?
*Owner: Network Protocol GDD (NP-OQ-2). Blocking: Board Rendering trap rendering story.*

**OQ-BR-08 — Visual Identity Anchor compliance (OPEN — NEW 2026-04-30)**
The Visual/Audio Requirements section specifies render layers, fog tween timings, idle oscillation amplitudes, but does not reference the project's Visual Identity Anchor. If the anchor (defined in `design/gdd/game-concept.md` or art bible) says e.g. "everything reads as physical paper", then fog tween easing, unit oscillation curves, and objective fog reveal all have aesthetic constraints not captured here. Must be cross-referenced once the art bible is authored.
*Owner: Art Director + Art Bible. Blocking: M2 VFX polish stories (M3 if pushed).*

**OQ-BR-09 — Fake reveal audio cue revisit (OPEN — NEW 2026-04-30)**
Current spec for fake reveal is "hollow dud thud (intentionally underwhelming)". Game-designer review flag: the dud sound communicates failure for the attacker (correct) but also for the defender who executed the bluff (incorrect — defender should feel triumphant). A spatial-audio per-PlayerID solution is not in current architecture scope. Simpler alternative: replace dud with a "trickster chord" that frames the moment as cleverness rather than malfunction — communicates "this was a bluff" without taking sides emotionally.
*Owner: Audio Director (audio-director gate, not implementation gate). Non-blocking for M2 — defer to first audio pass.*

**OQ-BR-10 — HP bar live-update implementation: poll vs tween (OPEN — NEW 2026-04-30)**
Rule 6 says HP bars "live-update" during `ResolutionExecuting` and the VFX table says "fill bar `scale.x` lerp to new value". Two viable approaches: (A) frame-poll via `Changed<UnitStats>` filter — system reads replicated component each frame and updates child Transform.scale.x directly; (B) per-event tween — on each HP-change `ResolutionEvent`, schedule a `bevy_tweening` Tween on the HP bar's scale.x. Approach A is simpler and free when HP isn't changing; approach B adds ~50 more `Animator` components per resolution batch. **Recommended: approach A** — server-side smoothing of HP delivery removes the need for client-side tweening, and the visual "lerp" can be achieved by the natural per-tick replication cadence. To close: pick one and write the implementation pattern into Rule 6.
*Owner: Board Rendering GDD + lead-programmer. Blocking: HP bar implementation story.*
