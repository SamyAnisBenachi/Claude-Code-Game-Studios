# Board Rendering Spec — Sprint 15 Board-Rendering Companion

> **Story**: `S11-UX-BOARD-RENDERING-SPEC`
> (`production/epics/board-rendering/story-013-board-rendering-spec.md`)
> **Sprint**: 15 (active; stage `Polish`; PROMPT 761 `Polish->Release` `FAIL`
> preserved)
> **Authoring prompt**: PROMPT 1004 (`/dev-story`)
> **Source-of-truth at authoring**: `origin/main@84e621e` (PROMPT 1002
> Sprint 15 QA plan) — story-013 file blob from PROMPT 995 integration.
> **Worktree**: `D:/_DEV/claude-code-game-studios-worktrees/s15-board-rendering-spec-1004`
> **Branch**: `work/s15-board-rendering-spec`
> **Roadmap rank**: 14 (Tier 3, Should, 0.75d) per
> `docs/ux/ui-clean-pass-roadmap.md`

This document is the **canonical board rendering spec** for the playable
client. It is the Tier 3 companion to `docs/ux/global-ui-design-spec.md`
(rank 6, Sprint 14, DONE). It governs world-space sprite layout under
`client/src/presentation/board_rendering.rs` and its sibling cell /
unit / objective / status-icon / ghost-preview render surfaces — the
visual contract that `production/epics/board-rendering/` BR-001 through
BR-012 implement.

Per `docs/ux/ui-clean-pass-roadmap.md` §3 Sequencing Rule 4, this rank-14
story is **doc-only** and depends on rank 6 as its parent design-spec
doc. Numeric values for layout primitives (z-layers, spacing tokens,
typography, overlay alpha, color palette, responsive matrix) are read
from rank 6; this spec adds the board-specific composition rules that
sit on top of those primitives.

---

## §1 Status / No-Claim Banner

This spec is **paperwork only**. PROMPT 1004 (this authoring run) is a
`/dev-story` documentation closure for story 013. It authors **only**
the files under `docs/ux/board-rendering-spec.md` and
`production/qa/evidence/sprint-15-board-rendering-spec/`. It does
**not** change any code, any test, any sprint plan, any sprint-status
row, any orchestrator state file, any QA-plan file, or any session-state
file. The Sprint 15 plan (active per PROMPT 997) and Sprint 15 QA plan
(authored per PROMPT 1002) are preserved verbatim.

### What this spec does NOT claim

This spec, and its adoption by the board-rendering epic stories (BR-001
through BR-012), does **not** claim, advance, or close any of:

- Public release readiness.
- Release-candidate (RC) readiness.
- Full game completion.
- Broad / Standard-tier accessibility completion (`QA-COND-0005`).
- Playtest / fun-hypothesis validation (`QA-COND-0006`).
- Full playable-client manual QA.
- Two-client GAME_OVER closure (`S8-QA-001-W1`).
- Final-art / asset-production completion (`PAW-TD-002-a` …
  `PAW-TD-006-a`).
- Sprint 14 row reopen (any of the 16 closed Sprint 14 rows).
- Sprint 15 row activation beyond what PROMPT 997 already activated.
- Polish → Release gate-check retry (PROMPT 761 `FAIL` preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO**
  retry is in scope).
- Stage advance from `Polish` to `Release`.
- Underlying drag-runtime bug fix (Sprint 12 story 019 remains
  `closed-with-conditions / cannot-reproduce`).
- Closure of `TQ-S12-C7` or any other `TQ-S12-C1..C7` Team-QA condition.
- Full Board Rendering epic closure (BR-008 reconnect/snapshot, BR-007
  protocol gap rows, and the final visual / evidence split-follow-ups
  remain open).

### Accept-risk dispositions preserved verbatim

The following dispositions are preserved **unchanged** by this spec:

- **`QA-COND-0005`** — Standard-tier accessibility remains
  **accepted-risk** (friend-game scope only). This spec is **friend-game
  visual polish only**; it does **not** pursue WCAG contrast ratios on
  board overlays, ≥44px hit-targets on cell nodes / status icons /
  ghost previews, full keyboard navigation, screen reader support,
  colorblind modes (beyond the Player A circle / Player B hexagon base-ring
  shape redundancy already shipped per BR-009), or text scaling on lane
  labels / HP-bar overflow numerics. A separate Standard-tier
  accessibility spec is the only path to advance `QA-COND-0005`; this
  spec is not it.
- **`QA-COND-0006`** — playtest / fun-hypothesis validation remains
  **accepted-risk / deferred**. A visibly documented board does not by
  itself produce playtest evidence.
- **`PAW-TD-002-a` … `PAW-TD-006-a`** — placeholder-art accept-risk
  across PAW-002..PAW-006. Board rendering spec authoring is layout /
  composition / hierarchy / overlay / status-icon-legend work and does
  **not** advance placeholder-art resolution. Final atlas / icon art
  replacement is out of spec scope. Color and shape tokens cited below
  are **placeholder palette / placeholder shape** for friend-game scope;
  final asset replacement is a separate sprint scope.
- **`S8-QA-001-W1`** — two-client GAME_OVER closure remains OPEN. This
  spec does not exercise live two-client GAME_OVER and does not advance
  this row.

If a future activation attempts to silently expand the claim (e.g. flips
`QA-COND-0005` to `closed`, claims Standard-tier conformance on board
overlays, or claims `PAW-TD-*-a` resolved by spec-only documentation),
the activation must be rejected and the row sent back for scope
correction.

---

## §2 Scope Boundaries — Friend-Game vs Standard-Tier

This spec governs **friend-game board visual polish only**. It is the
single source of truth for board cell layout, unit placement composition,
range overlay rendering, status icon legend, and ghost preview opacity
**for friend-game scope**.

### What this spec governs (in scope)

- World-space sprite composition for the 5-lane × 8-cell grid (cell
  rendering rules; §3).
- Unit placement composition including the F3 co-occupancy ±half-offset
  rule and the ChildOf hierarchy for HP bars and status icons (§4).
- Range overlay composition (spawn range highlights + draft-phase
  placement-ghost cursor mapping; §5).
- Status icon legend — canonical mapping of persistent keyword / state
  kinds to status icon atlas frames; Tier 1 vs Tier 2 priority; overflow
  badge rule (§6).
- Ghost preview opacity — canonical sprite-level alpha for the
  placement-preview ghost on the board (§7).
- Cross-references to `docs/ux/global-ui-design-spec.md` for z-layers,
  color tokens, overlay alpha scope-guard, and the responsive 6-viewport
  matrix (§8).
- Cross-references to ADR-021, ADR-020, ADR-017, ADR-011, ADR-008,
  ADR-002, and `design/gdd/board-rendering.md` (§9).
- Producer ratification gate (§10).

### What this spec does NOT govern (out of scope)

- **Standard-tier accessibility** — WCAG contrast checking on overlay
  tints / status icons / ghost preview alpha, ≥44px hit-target enforcement
  on board cells / ghost-click surfaces, focus order on world-space
  surfaces, keyboard navigation across the grid, screen-reader hints,
  colorblind modes beyond Player A/B base-ring shape redundancy, text
  scaling on lane number labels. These belong in a separate accessibility
  spec; pulling values out of this spec does NOT advance `QA-COND-0005`.
- **Final-art / asset-production** — `PAW-TD-*-a` placeholder PNGs
  (PAW-002..PAW-006) are preserved. Atlas frame art, cell tile art,
  status-icon glyph art, and ghost-preview art are **placeholder** for
  friend-game scope; final asset replacement is a separate sprint scope.
- **Gameplay rules** — combat resolution math, keyword resolution
  ordering, spawn validation, HP threshold calibration, OUTNUMBERED
  comparison semantics. These remain owned by `design/gdd/board-rendering.md`,
  `design/gdd/combat-resolution.md`, `design/gdd/keyword-system.md`,
  and `design/gdd/board-lane-system.md`. This spec is read-only against
  those GDDs.
- **Networking contracts** — Lightyear protocol shape, message
  ownership, single-drain discipline, replication channel mapping. These
  remain owned by ADR-008 (Lightyear channel config), ADR-011 (reconnect
  snapshot), and `design/gdd/network-protocol.md`. This spec is read-only
  against those.
- **HUD spec** — HUD per-element layout, top-strip child order, reserve
  readouts, opponent figurine. Owned by Sprint 14 stories
  `S11-UX-HUD-TOP-STRIP-LAYOUT` (DONE) /
  `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` (DONE) /
  `S11-UX-HUD-OPP-FIGURINE` (DONE).
- **Hand UI spec** — hand fan layout, drag-state visuals, ghost-source
  cursor. Owned by `design/gdd/hand-ui.md` and Sprint 15 Should Have
  `S12-UX-HAND-DRAG-STATE-VISUALS-001` (separate story). This spec only
  governs the **board-side** ghost preview (the sprite painted on the
  board cell), not the dragged card surface in the hand.
- **Animation / motion** — placement reveal tweens (BR-005 Complete),
  resolution playback (BR-006 Ready), reveal-tween easing curves,
  bevy_tweening lens contracts. Owned by ADR-017 and by their existing
  BR-* story files.
- **Interaction-state primitives** — hover / focus / pressed / disabled
  state primitives are owned by Sprint 15 Nice to Have
  `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` (separate story).
- **Localization** — string layout, RTL, text expansion on lane labels
  / HP overflow numerics. Separate scope.
- **Per-system GDD edits** — `design/gdd/board-rendering.md` is the
  GDD source of truth. This spec cross-references but does not modify it.
- **ADR edits** — ADR-021 / ADR-020 / ADR-017 / ADR-011 / ADR-008 /
  ADR-002 are read-only cross-links here.

### Cross-cutting concerns delegated upward

The following cross-cutting primitives delegate UP to
`docs/ux/global-ui-design-spec.md`. This spec consumes them and does
not re-author them:

| Concern | Delegated to | Note |
|---|---|---|
| Z-layer integer values for `World` / `Units` | `docs/ux/global-ui-design-spec.md` §3 | This spec adds the world-space-Z constants in §3 / §4 below; the global spec ratifies their integer alignment with bevy_ui layer values. ADR-021 R2 preserved. |
| Overlay alpha tokens (`OVERLAY_DIM_ALPHA`, `OVERLAY_SCRIM_ALPHA`) | `docs/ux/global-ui-design-spec.md` §6 | bevy_ui modal-scrim tokens. Board ghost preview alpha is **out of scope** for those tokens — see §7 below for the scope-guard cross-link. |
| Color palette (`PRIMARY`, `SECONDARY`, `ACCENT`, `SEMANTIC_SUCCESS` / `_WARNING` / `_ERROR`) | `docs/ux/global-ui-design-spec.md` §7 | Range overlay tints and status icon tints reference friend-game palette tokens. Final-art replacement remains separate. |
| Responsive 6-viewport matrix | `docs/ux/global-ui-design-spec.md` §8 | World board scales with viewport via camera zoom per §8 per-class scaling rules ("World-space sprites" class). |

### Cross-cutting concerns delegated outward

| Concern | Delegated to | Note |
|---|---|---|
| Presentation plugin order, PresentationSet, paint order | ADR-021 | World-space sprites paint below bevy_ui regardless of layer constants per ADR-021 R2. |
| Board state replication contract | ADR-020 | Authoritative source for `BoardPosition`, `UnitStats`, `ObjectiveHp`. |
| Animation queue and reveal tween | ADR-017 + `design/gdd/board-rendering.md` Rule 9 / Rule 7 | This spec does not re-specify resolution playback. |
| Snapshot / reconnect rebuild | ADR-011 + GDD Rule 11 | This spec does not re-specify reconnect. |
| Lightyear channel mapping | ADR-008 | This spec does not re-specify replication channels. |
| Client-as-view authority | ADR-002 | Board Rendering is read-only against server state. |

---

## §3 Cell Rendering Rules

The board is a **5-lane × 8-cell grid** of 40 world-space sprite cell
nodes. Cell rendering is the foundation for unit placement, range
overlay, and ghost preview; everything else in §4-§7 anchors to a cell.

### Canonical grid

| Property | Value | Source |
|---|---|---|
| Lane count | 5 (lanes 1..=5; 1 = top of screen, 5 = bottom) | `design/gdd/board-lane-system.md`; `GameConfig.lane_count = 5`. |
| Cells per lane | 8 (cells 1..=8) | `design/gdd/board-lane-system.md`; `GameConfig.cells_per_lane = 8`. |
| Total cell sprite entities | 40 (5 × 8) | Per `design/gdd/board-rendering.md` AC BR-1. |
| Coordinate authority | `BoardLayout::cell_to_world(lane, cell) -> Vec2` | ADR-021 R2 (world-space sprite rendering); TR-BR-002 (single coordinate authority); GDD Rule 3. |
| Pixel size — `cell_width` | 64.0 px (default; tunable 48–96) | `design/gdd/board-rendering.md` F1 + Tuning Knobs (`GameConfig.board_cell_width`). |
| Pixel size — `lane_height` | 80.0 px (default; tunable 64–112) | `design/gdd/board-rendering.md` F1 + Tuning Knobs (`GameConfig.board_lane_height`). |
| Board span at defaults | 448 px wide × 320 px tall | F1 example: 7 cell gaps × 64 = 448; 4 lane gaps × 80 = 320. |
| Per-cell sprite atlas | `env_cell_node_idle_32x32` (BLOCKING per GDD §Asset Requirements) | Board-elements atlas (shared with objectives, prisms, status icons per GDD Rule 5). |
| Pre-condition assertion | `assert!((1..=5).contains(&lane) && (1..=8).contains(&cell))` (release-mode assertion, not `debug_assert!`) | GDD F1 PRECONDITION + AC BR-2b. |

### `cell_to_world` formula (read-only excerpt from GDD F1)

```text
cell_to_world(lane, cell) = Vec2 {
    x: board_origin.x + (cell - 1) as f32 * cell_width,
    y: board_origin.y - (lane - 1) as f32 * lane_height,
}
```

This spec does not redefine the formula. It names it as the single
authority every spawn site, hover system, ghost preview, range overlay,
and Hand UI cursor mapping consumes. Inline cell-position literals in
spawn functions are forbidden per GDD Rule 3.

### Lane-color tinting

Lane numbering reads top-to-bottom (lane 1 = top of screen; lane 5 =
bottom). Player A (local player) and Player B (opponent) halves of the
board receive distinct tint families via `Sprite.color` (vertex-data
tint; **no** per-cell `Handle<ColorMaterial>` per GDD Rule 5):

| Half | Tint family | Source |
|---|---|---|
| Player A (local) | Cool family — friend-game placeholder | GDD §Asset Requirements "Player A / Player B half color tinting (cool vs. warm) is applied at runtime via `Sprite.color` — no separate per-player node textures required." |
| Player B (opponent) | Warm family — friend-game placeholder | Same source. |

Final-art tint values are deferred to a follow-on colorization pass story
(PAW-TD-*-a accept-risk).

### Cell composition

Each `BoardCellNode` entity is composed as a single `Sprite` from the
board-elements atlas, with optional state tint applied via `Sprite.color`
(per the spawn-highlight encoding documented in §5). Cell border / cell
fill are baked into the atlas frame (no compositing of two child sprites
per cell — this preserves the GDD Rule 5 draw-call budget).

### World-space z-layer reference

| World-space layer | Z value | Bevy module symbol | Global UI design spec layer reference |
|---|---|---|---|
| `Z_FIELD_WASH` | 0.0 | (local to board) | Below `World` (100) bevy_ui reference. |
| `Z_CELL_NODES` | 1.0 | (local to board) | Below `World` (100) bevy_ui reference. |
| `Z_TRAPS_STRUCTURES` | 2.0 | (local to board) | Below `World` (100) bevy_ui reference. |
| `Z_OBJECTIVES` | 2.5 | (local to board) | Below `World` (100) bevy_ui reference. |
| `Z_UNITS` | 3.0 | (local to board) | Maps conceptually to `Units` (200) per global UI spec §3. |
| `Z_HEALTH_BARS` | 3.1 (LOCAL 0.1 on child Transform; parent unit at `Z_UNITS = 3.0`) | (local to board) | Above `Units`; below bevy_ui `UiBase` (300). |
| `Z_GHOST_UNIT` | 3.5 | (local to board) | Above `Units`; below bevy_ui `UiBase` (300). |

**ADR-021 R2 paint order preserved**: world-space sprites paint below
bevy_ui regardless of the `World` (100) / `Units` (200) integer values
referenced in `docs/ux/global-ui-design-spec.md` §3. The global UI spec
constants exist for documentation and cross-layer audits; the per-sprite
`Transform.translation.z` values above are the authoritative paint
order within the world-space sprite batch.

### Relation between cell pixel size and the canonical 6-viewport matrix

Per `docs/ux/global-ui-design-spec.md` §8 "Per-class scaling rules", the
world board is in the **"World-space sprites (not bevy_ui)"** class:
"World board scales with viewport via camera zoom; sprite `Transform.z`
reads §3 World / Units layer constants. Tile sprites, unit sprites,
objective sprites."

The 6-viewport canonical matrix consumed:

| Viewport | Camera scaling behavior |
|---|---|
| `1366×768` (minimum supported) | Camera zoom adjusts so the full 5×8 board fits within the centred play area with the HeaderBar (60 px) + FooterBar (40 px) + HandBar (180 px) strip column (per global UI spec §9) cleared. Pixel-fixed strips push the world board into the remaining centre region. |
| `1920×1080` (baseline reference) | Design-source viewport; baseline captures land here. Cell pixel size at defaults reads ~64 px × 80 px at the design-source camera zoom. |
| `1920×1200` (16:10) | Vertical headroom relative to baseline; camera zoom unchanged; board centred in available centre region. |
| `1280×960` (4:3) | Camera zoom adjusts to fit the 5×8 grid into the narrower viewport; aspect-stretch chrome acceptable around the world bounds. |
| `3840×2160` (4K) | Camera zoom scales board up; pixel-fixed strips remain at design pixel values per global UI spec §9. |
| `2560×1080` (21:9) | Board remains centred; horizontal headroom on either side of the world bounds. |

The per-viewport zoom math is implemented in the camera setup at
`client/src/presentation/...` (see ADR-021 R2 for camera authority).
This spec ratifies the **principle** — board scales with viewport via
camera zoom; strip primitives stay pixel-fixed — and defers per-viewport
exact zoom factors to the camera setup module.

### Friend-game palette only

Cell tile art is placeholder per GDD §Asset Requirements (BLOCKING M2
priority for `env_cell_node_idle_32x32` and
`env_cell_node_spawn_active_32x32`; PLACEHOLDER for inactive and
invalid). This spec does **not** specify final-art cell tiles.
`PAW-TD-002-a` accept-risk preserved.

---

## §4 Unit Placement Rules

Each unit on the board is a single `Sprite` entity from the **unit
atlas** (per GDD Rule 5 single-atlas budget) anchored on a
`(team, lane, cell)` tuple via the F1 `cell_to_world` formula. HP bars
and status icons are child entities of the unit so co-occupancy offsets
and team transforms propagate through hierarchy.

### Canonical unit sprite anchor

| Property | Value | Source |
|---|---|---|
| Anchor primitive | `(team, lane, cell)` tuple | `team` ∈ {Player A, Player B}; `lane` ∈ 1..=5; `cell` ∈ 1..=8. Replicated via `BoardPosition { lane, cell }` and unit `owner: PlayerId` per ADR-020 / GDD Rule 1. |
| World position | `cell_to_world(lane, cell) + co_occupancy_offset (F3 if applicable)` | F1 + F3 from `design/gdd/board-rendering.md`. |
| Sprite source | Unit atlas (single `Handle<Image>` for all units per GDD Rule 5; per-card atlas frame index looked up by `card_id`) | GDD §Asset Requirements + AC BR-3a. |
| Sprite size | `UNIT_SPRITE_WIDTH = 48.0` px (fixed art constant) | GDD §Tuning Knobs Internal Constants. |
| Z layer | `Z_UNITS = 3.0` (§3) | World-space sprite paint order. |
| Per-team distinction | Player A base ring (circle, 48×16) + Player B base ring (hexagon/diamond, 48×16) shape-redundant for colorblind users | GDD §Asset Requirements "Unit Bases (colorblind redundancy — shapes are load-bearing)". |
| Card-miss fallback | `ui_unit_placeholder_48x64` atlas frame (solid color tile + "?" glyph) | GDD §Asset Requirements + EC-CARD-MISS + AC BR-EC-CARDMISS. |

### Co-occupancy ±half-offset rule (F3)

When two allied units share a `(team, lane, cell)` cell in 2v2 mode,
each is offset from the cell centre by `±co_occupancy_side_offset / 2`
along the X axis:

```text
x_offset(unit_index) = (unit_index as f32 - 0.5) * co_occupancy_side_offset

// PRECONDITION (per GDD F3): unit_index in {0, 1}.
// assert!(unit_index <= 1, "F3 co-occupancy: unit_index={} > 1 — invalid co-occupancy state", unit_index);
```

| Property | Value | Source |
|---|---|---|
| Side offset (default) | `co_occupancy_side_offset = 8.0` px | GDD F3 + Tuning Knobs (`GameConfig.board_co_occupancy_offset`). |
| Index-0 X displacement | `-4.0` px (left of cell centre at default) | F3 example. |
| Index-1 X displacement | `+4.0` px (right of cell centre at default) | F3 example. |
| Index assignment | Ascending entity ID among allied co-occupants | GDD F3 Variables row "Unit index". |
| Index ≥ 2 behavior | **Panic** via `assert!` (release-mode, not `debug_assert!`) — server-side bug; silent overflow would mask the bug | GDD F3 PRECONDITION + AC BR-22 / BR-22b. Story 009 acceptance "Co-occupancy index 2 triggers the GDD-mandated `assert!` with the offending index in the message". |
| Constraint | `co_occupancy_side_offset + UNIT_SPRITE_WIDTH/2 ≤ cell_width/2` (intake-clamped; warn on violation) | GDD Tuning Knobs "Constraint (intake-validated)" + AC BR-COOCC-CONSTRAINT. |
| 1v1 applicability | F3 not evaluated in 1v1 (at most one unit per player per lane) | GDD F3 "applies only in 2v2 mode". |

The **F3 index-2 `assert!`** is the load-bearing correctness mechanism
against silent server bugs that would otherwise render units outside
the cell with no diagnostic. It is enforced at the F3 call site and
verified by AC BR-22b. This spec ratifies the assertion as required
for any board rendering implementation pass.

### ChildOf hierarchy for HP bars and status icons

HP bars and status icons attach to their parent unit entity using the
Bevy 0.18 `ChildOf` component (NOT pre-0.16 `Parent` / `set_parent` —
both removed in 0.16; verified against GDD §Bevy 0.18 API Contract).

```text
unit (Sprite + Transform + ChildOf nothing)
├── hp_bar_background (Sprite + Transform + ChildOf(unit))
├── hp_bar_fill (Sprite + Transform + ChildOf(unit))
├── status_icon_0 (Sprite + Transform + ChildOf(unit))
├── status_icon_1 (Sprite + Transform + ChildOf(unit))
├── status_icon_2 (Sprite + Transform + ChildOf(unit))
└── status_overflow_badge (Sprite + Transform + ChildOf(unit))  // only when ≥4 effects
```

Co-occupancy `x_offset` is written onto the **parent unit**'s
`Transform.translation.x`. The HP bar and status icon children inherit
the offset automatically through Bevy's hierarchy + `GlobalTransform`
propagation. Children must NOT re-centre on the cell — story 009
acceptance "Status icons inherit co-occupancy X offset from the unit
parent through hierarchy; they do not re-center on the cell" + AC
BR-STATUS-COOCCUPANCY.

### Canonical HP-bar geometry

HP bar visual contract is preserved from GDD Rule 6 + Asset Requirements
+ AC BR-Z-LOCAL / BR-3c / BR-HP-INVARIANT. This spec names the
above-unit anchor, the atlas-shared white-pixel frame, and the color
threshold mapping; gameplay HP thresholds remain owned by the GDD.

| Property | Value | Source |
|---|---|---|
| Anchor | Above-unit child entity (top-anchored) | GDD Rule 6. |
| Sprite source | `hp_bar_white_pixel_1x2` reserved frame in the **unit atlas** (shared `Handle<Image>` with units; batches in a single draw call) | GDD §Asset Requirements R2 new + AC BR-3a / BR-3c. |
| Z (LOCAL) | `0.1` on child Transform (parent at `Z_UNITS = 3.0` → global Z = 3.1) | GDD §Bevy 0.18 API Contract "Health bar child Z is local, not global" + AC BR-Z-LOCAL. |
| Width-fill mechanism | `Transform.scale.x = fill ∈ [0.0, 1.0]` driven by F2 against replicated `UnitStats { hp_current, hp_max }` | GDD F2 + Rule 6 + AC BR-4. |
| Background tint | `Color::srgba(0.1, 0.1, 0.1, 0.7)` (dark grey) | GDD §Asset Requirements R2 new. |
| Fill color thresholds | Green if `fill ≥ green_threshold - HP_THRESHOLD_EPSILON` (default green threshold 0.6); Yellow if `fill ≥ red_threshold - HP_THRESHOLD_EPSILON` (default red threshold 0.3); Red otherwise | GDD F2 (thresholds are gameplay-tunable knobs in GameConfig — this spec does not re-specify the threshold defaults; they live in the GDD Tuning Knobs section). |
| Always-visible invariant | Every live unit (`hp_current > 0`) carries a visible HP bar regardless of `BoardRenderState` (except `Idle`) | GDD Rule 6 + AC BR-5. |
| Tween forbidden on fill scale | `UpdateHpBars` writes `scale.x` directly; no `Animator<Transform>` may be scheduled on the fill entity's scale axis | GDD Rule 6 HP bar update invariant + AC BR-HP-INVARIANT. |

This spec does not re-specify the gameplay HP threshold defaults; those
remain in `design/gdd/board-rendering.md` Tuning Knobs and are loaded
from `GameConfig`.

---

## §5 Range Overlay Rules

Range overlay covers two surfaces: persistent **spawn range highlights**
on cell nodes (BR-011 source contract) and the transient **draft-phase
placement-ghost cursor** (BR-004 ghost-preview bridge). Both render in
world-space via `Sprite.color` tint or per-cell child sprite — never via
bevy_ui — to stay inside the GDD Rule 5 draw-call budget.

### Spawn range highlight (persistent)

Per TR-BR-008 + BR-011 source contract (PROMPT story 011, Complete on
`origin/main`):

| Property | Value | Source |
|---|---|---|
| Snapshot seed source | `PlayerSnapshot.spawn_range_cells` from `S2CGameSnapshot` rebuild | TR-BR-008; BR-011 acceptance "Snapshot seed"; AC BR-SPAWN-HIGHLIGHTS. |
| Live update source | `ResolutionEvent::SpawnRangeChanged { player_id, new_spawn_range_cells }` inside the ordered reliable `S2CResolutionEvent` batch | TR-NP-014; BR-011 acceptance "Live update consumption"; AC BR-SPAWN-HIGHLIGHTS. |
| Forbidden source 1 | **Do NOT** derive spawn range from `ObjectiveDestroyed.was_fake` | BR-011 Control Manifest Rules "Forbidden: Do not derive spawn range from `ObjectiveDestroyed.was_fake`". |
| Forbidden source 2 | **Do NOT** consume a replicated `SpawnRange` ECS component (no such component exists in the protocol) | BR-011 Control Manifest Rules "Forbidden: Do not consume a replicated `SpawnRange` component". |
| Per-cell visual mechanism | `SpawnHighlightState { player_id, in_spawn_range }` component on each `BoardCellNode`; visualised by recolouring the cell sprite via `Sprite.color` (state encoded as tint per GDD Rule 4 R2 "Spawn highlights are encoded as `Sprite.color` tint on `Z_CELL_NODES` sprites — no separate sprite or Z layer") | BR-011 Implementation Notes; AC BR-SPAWN-HIGHLIGHTS. |
| Persistence rule | Spawn highlights persist across DRAFT and PLACEMENT frames; do not flicker per round transition | BR-011 acceptance "Persistence". |
| Update path ownership | Existing Board Rendering message-drain / resolution-event drain — no duplicate `MessageReceiver<S2CPhaseChanged>` or second Lightyear drain | BR-011 Implementation Notes; ADR-008 Lightyear single-drain discipline. |
| Tint palette | Friend-game palette tokens from `docs/ux/global-ui-design-spec.md` §7 (e.g. `SEMANTIC_SUCCESS` for valid-spawn highlight tint; tuned by per-cell-state mapping at implementation time) | Global UI spec §7 placeholder palette; PAW-TD-002-a accept-risk preserved. |
| Z layer | `Z_CELL_NODES = 1.0` (tint encoded on cell node sprite; no separate overlay sprite) | GDD Rule 4 R2 + §3 above. |

ADR-020 owns the board-lane state replication contract (the
`spawn_range_cells` field on `PlayerSnapshot` and the
`SpawnRangeChanged` resolution-log event variant). ADR-011 owns the
reconnect snapshot rebuild. This spec is read-only against both.

### Draft-phase placement-ghost cursor mapping (transient)

Per BR-004 (Ghost Preview Hand UI Bridge, Complete on `origin/main`):

| Property | Value | Source |
|---|---|---|
| Trigger | Hand UI writes `GhostPlacementChanged { target: Option<PlayTarget>, card_id: Option<CardId> }` per drag-state-change | GDD Rule 8 + BR-004. |
| Cursor-to-cell mapping | `Res<BoardLayout>` cell hit-test; Hand UI reads `BoardLayout` for cursor-to-cell snapping | GDD Rule 3 + TR-HU-002. |
| Variant-specific composition | `BoardCell` variant spawns a `GhostUnit` sprite at `cell_to_world(lane, cell)`; `TargetUnit` applies a `TargetUnitGhost` marker on the unit entity; `TargetObj` applies an `ObjectiveTargetGhost` marker on the objective entity; `LaneWide` spawns a `LaneGhostWash` over the full column; `Instant` is no-op on the board side (Hand UI fan slot is the entire visual) | GDD Rule 8 variant table. |
| Replacement discipline | Exactly one ghost preview per `card_id`; spawning a new ghost replaces any prior ghost for the same card | GDD Rule 8 + AC BR-8. |
| Reveal cleanup | On `S2CPlacementReveal`: despawn all ghost preview entities and clear all ghost marker components immediately | GDD Rule 8 + AC BR-9. |
| Invalid cell behavior | Ghost stays at last valid cell; invalid cell node shows brief red tint | EC-INVALID-GHOST. |
| Reverse events to Hand UI | `GhostClickedEvent { card_id }` (on click) and `GhostDragStartEvent { card_id }` (on mouse-down) written by Board Rendering, consumed by Hand UI | GDD Rule 8 "Reverse events to Hand UI" + AC BR-8e. |

### Range overlay z-layer reference

The range overlay surfaces compose as follows. For any **world-space**
overlay (cell tint, ghost sprite, lane-wide wash), the z-layer reference
is the local board Z-stack in §3. For any **bevy_ui** overlay (none in
current scope, but reserved for future status / scrim overlays painted
above the board), the layer reference is `UiOverlay` (400) per
`docs/ux/global-ui-design-spec.md` §3. Board ghost preview alpha is
**sprite-level** and does **NOT** consume the bevy_ui overlay alpha
tokens — see §7 below for the scope-guard cross-link.

### Friend-game palette

Range overlay tint values reference the friend-game palette tokens from
`docs/ux/global-ui-design-spec.md` §7 (`SEMANTIC_SUCCESS`,
`SEMANTIC_WARNING`, `SEMANTIC_ERROR`, `ACCENT`). Final-art tint values
are deferred to a follow-on colorization pass. PAW-TD-002-a accept-risk
preserved.

---

## §6 Status Icon Legend

This section is the canonical mapping of persistent keyword / state
kinds to status icon atlas frames, with Tier 1 / Tier 2 display priority,
overflow badge rule, and per-unit OUTNUMBERED distinction. It is the
implementation contract that BR-009 (Complete) shipped.

**This section folds the `S11-UX-BOARD-STATUS-ICON-LEGEND-001`
future-candidate cosmetic capture into the spec.** Per PROMPT 802
§9 producer-decision-5 reconciliation and the Sprint 15 plan §"Wider
Sprint 15 Backlog" note ("Two of these
(`S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` +
`S11-UX-BOARD-STATUS-ICON-LEGEND-001`) are folded as spec sections
into Sprint 15 Should Have `S11-UX-BOARD-RENDERING-SPEC` rather than
as separate captures"), `S11-UX-BOARD-STATUS-ICON-LEGEND-001` does
**not** remain a separate Sprint 15 story; it is closed by this
section landing.

### Status icon atlas mapping

Persistent keyword and state kinds in current implementation scope per
BR-009 acceptance:

| Keyword / state kind | Display tier | Reason | Source |
|---|---|---|---|
| `SHIELD` (a.k.a. SHIELDED) | **Tier 1** | Combat-deciding — attacking a SHIELDED unit wastes damage. Hiding this causes incorrect placement decisions. | GDD Rule 14 R4 "Display priority ordering" Tier 1 row + TR-BR-006. |
| `TAUNT` | **Tier 1** | Combat-deciding — TAUNT forces attacker targeting. | GDD Rule 14 R4 Tier 1 row. (Reserved for future addition; not in BR-009 current implementation scope; this spec names the tier assignment so future story landing keeps the Tier-1 sort order.) |
| `STEALTH` | **Tier 1** | Combat-deciding — STEALTH affects whether the unit is a valid target. | GDD Rule 14 R4 Tier 1 row. (Reserved; same note as TAUNT.) |
| `IMMUNE` | **Tier 1** | Combat-deciding — IMMUNE blocks damage. | GDD Rule 14 R4 Tier 1 row. (Reserved; same note as TAUNT.) |
| `STUN` | Tier 2 | Relevant but not directly placement-deciding in the same way. | GDD Rule 14 R4 Tier 2 row + TR-BR-006. |
| `SILENCE` | Tier 2 | Same. | GDD Rule 14 R4 Tier 2 row + TR-BR-006. |
| `INJURED` | Tier 2 | Same. | GDD Rule 14 R4 Tier 2 row + TR-BR-006. |
| `LEADER` | Tier 2 | Same. | GDD Rule 14 R4 Tier 2 row + TR-BR-006. |
| `HASTE` | Tier 2 | Same. | GDD Rule 14 R4 Tier 2 row. |
| `BODYGUARD` | Tier 2 | Same. | GDD Rule 14 R4 Tier 2 row + TR-BR-006 (per story 009 in-scope keyword list). |
| `OUTNUMBERED` | Tier 2 | Per-unit (not per-lane) per TR-BR-007 + `OQ-KS5` closed in `design/gdd/keyword-system.md`. | GDD Rule 14 R4 Tier 2 row + TR-BR-007 + TR-KW-010. |
| `INJURED`-granted keyword indicators (e.g. `KeywordPayload` icons exposed at runtime) | Tier 2 (per keyword `display_tier`) | Reads `display_tier` at runtime; must not hard-code keyword names. | BR-009 In Scope + GDD Rule 14 "Board Rendering must NOT hard-code keyword names — it reads `display_tier` from the keyword definition". |

The actual `display_tier: u8` field (1 or 2) per keyword is owned by
`design/gdd/keyword-system.md`. Board Rendering reads it at runtime;
this spec names the **mapping** in the table above for legibility but
does **not** override the GDD definition.

### Display priority and sort key

Per GDD Rule 14 R4 and AC BR-STATUS-TIER, the visible-slot fill order is:

1. **Sort by ascending `display_tier`** (Tier 1 fills the earliest slots).
2. Within a tier, **sort by descending remaining-duration** (longest-
   remaining first; timed states use their remaining round count; untimed
   states use `0` for deterministic ties per story 009 In Scope).
3. **Then by deterministic keyword/state key** for stable ties.

Insertion order in `StatusEffectsList` is **not** a tie-breaker — a
Tier-1 keyword inserted last must still occupy slot 0 if it is the
highest-priority effect (per AC BR-STATUS-TIER + story 009 acceptance
"Tier-1 effects always outrank Tier-2 effects regardless of insertion
order in `StatusEffectsList`").

### Overflow badge rule

Per GDD Rule 14 + AC BR-STATUS-CONTRACT + story 009 acceptance:

| Effect count | Visible | Notes |
|---|---|---|
| 1..=3 active effects | 1..=3 `StatusIcon` children; no badge | One icon per effect. |
| ≥4 active effects | Top 3 `StatusIcon` children + 1 `StatusOverflowBadge` child in the 4th slot showing `+N` | `N = total_active_effects - 3`. Badge occupies slot 3 (position 4 in the horizontal stack). |

### Per-unit OUTNUMBERED distinction (TR-BR-007 + TR-KW-010)

OUTNUMBERED is **per unit**, not per lane (per TR-BR-007 revision; OQ-KS5
closed). Each unit carrying the OUTNUMBERED keyword/state renders its
own OUTNUMBERED icon child; the global-board-count comparison that
determines OUTNUMBERED status uses a strict less-than comparison per
TR-KW-010. Board Rendering reads the per-unit OUTNUMBERED flag from the
status projection (visual state only; no gameplay logic in the client).

### Layout (top-right horizontal stack)

Per GDD Rule 14 + AC BR-STATUS-CONTRACT + AC BR-STATUS-COOCCUPANCY:

| Property | Value | Source |
|---|---|---|
| Position | Top-right of unit sprite (offset `Vec2 { x: +unit_w/2 - 8.0, y: +unit_h/2 - 8.0 }` from parent unit centre) | GDD Rule 14. |
| Icon size | 16 × 16 px per icon | GDD Rule 14. |
| Stack direction | Horizontal: icon[0] at top-right; icons[1..2] offset left by 16 px each; overflow badge at icon[3] slot | GDD Rule 14. |
| Atlas | Board-elements atlas (status icons share the second atlas — no third atlas) | GDD Rule 14 + AC BR-STATUS-CONTRACT (d) + AC BR-2-ATLAS. |
| Z (LOCAL) | `0.05` on child Transform (parent unit at `Z_UNITS = 3.0` → global Z = 3.05, just above unit, below HP bar at 3.1) | GDD Rule 14. |
| Update mechanism | `Changed<StatusEffectsList>` filter on parent unit; child icons spawned/despawned to match (instant on/off; no tween) | GDD Rule 14. |
| Co-occupancy inheritance | Status icons inherit parent unit's `Transform.translation.x` (including F3 co-occupancy offset) via `ChildOf` hierarchy; do NOT re-centre on the cell | §4 above + AC BR-STATUS-COOCCUPANCY + story 009 acceptance. |

### Tooltips

Per GDD Rule 14 Player Fantasy: "status indicators that require hovering
to be understood have failed." Every status icon must be readable from
its glyph alone; no tooltip is required for legibility in friend-game
scope. (A future polish pass may add hover tooltips for deeper info —
e.g. exact remaining duration — but the icon must communicate the
keyword without it.)

### Friend-game palette / placeholder art

Status icon glyph art is placeholder per GDD §Asset Requirements
BLOCKING M2 priority; final-art replacement is out of spec scope.
PAW-TD-003-a / PAW-TD-006-a accept-risk preserved.

---

## §7 Ghost Preview Opacity

This section names the canonical **sprite-level alpha rule** for the
hand drag-and-drop placement-preview ghost painted on the board.

**This section folds the `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001`
future-candidate cosmetic capture into the spec.** Per PROMPT 802
§9 producer-decision-5 reconciliation and the Sprint 15 plan §"Wider
Sprint 15 Backlog" note (same reference as §6 above),
`S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` does **not** remain a
separate Sprint 15 story; it is closed by this section landing.

### Canonical ghost preview opacity value

| Property | Value | Source |
|---|---|---|
| Token name | `GHOST_PREVIEW_ALPHA` | Defined by this spec. |
| Value | `0.5` (single named value; not an ad-hoc literal) | GDD AC BR-11 (a) "Sprite.color.alpha = 0.5"; GDD Rule 8 ghost variant table "Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.5), unit_size)"; story 009 / 011 do not override. |
| Mechanism | `Sprite.color.alpha` (vertex-data alpha) on the ghost sprite directly; the ghost reuses the real unit's atlas frame with the `Sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.5)` tint, OR `Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.5), unit_size)` for simpler ghost variants. **Sprite-level alpha, not modal scrim alpha.** | GDD Rule 8 ghost variant table + AC BR-11. |

### Rationale

`0.5` is the canonical alpha because:

1. **Already shipped + tested.** GDD AC BR-11 already locks `0.5` as the
   ghost sprite alpha; BR-004 (ghost preview bridge) and BR-009
   (co-occupancy + ChildOf) consume this value. Changing it would
   regress existing test assertions.
2. **Sufficient distinction from real units.** A `0.5` alpha reads as
   "preview, not committed" without making the ghost so faint that it
   becomes hard to position. Half-opaque is the default web/UI
   convention for placeholder previews.
3. **Vertex-data alpha batches with the unit atlas.** `Sprite.color.alpha`
   is a vertex-data tint; it does not break the unit-atlas single-draw-
   call batch (per GDD Rule 5 + AC BR-3a). A per-ghost `Handle<ColorMaterial>`
   would break the batch and is forbidden (GDD Rule 5).

### Scope guard — sprite-level alpha is NOT modal scrim alpha

The ghost preview alpha is **sprite-level alpha on a world-space
sprite**. It does **NOT** consume `OVERLAY_DIM_ALPHA` (`0.45`) or
`OVERLAY_SCRIM_ALPHA` (`0.55`) from `docs/ux/global-ui-design-spec.md`
§6 because those tokens govern **bevy_ui modal scrims** (HUD combat-
focus dim, settlement overlay, result-screen backdrop, connection-lost
overlay), not world-space sprite alpha.

This is an explicit cross-link to `docs/ux/global-ui-design-spec.md`
§6 "Scope guard" paragraph, which reads verbatim:

> **Scope guard**: this token covers *modal scrim / dim* surfaces only.
> Board ghost preview opacity (a future Tier 2 row
> `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001`) and any sprite-level alpha
> remain out of scope.

This spec §7 is the canonical home for the ghost preview alpha rule
per that scope guard. The two tokens (`OVERLAY_DIM_ALPHA` /
`OVERLAY_SCRIM_ALPHA`) and `GHOST_PREVIEW_ALPHA` are **deliberately
separate values** that govern different surface classes:

| Token | Surface class | Value | Owner spec |
|---|---|---|---|
| `OVERLAY_DIM_ALPHA` | bevy_ui modal-focus dim | `0.45` | `docs/ux/global-ui-design-spec.md` §6. |
| `OVERLAY_SCRIM_ALPHA` | bevy_ui modal scrim | `0.55` | `docs/ux/global-ui-design-spec.md` §6. |
| `OVERLAY_TOAST_ALPHA` | bevy_ui toast root | `0.80` | `docs/ux/global-ui-design-spec.md` §6. |
| `GHOST_PREVIEW_ALPHA` | World-space sprite (ghost preview) | `0.5` | **This spec §7.** |

### Ghost preview lifecycle

The ghost preview lifecycle is owned by GDD Rule 8 + BR-004; this spec
names the lifecycle for completeness but does **not** re-specify the
bridge protocol:

1. **Spawn on hover / drag-start.** Hand UI writes
   `GhostPlacementChanged { target: Some(<variant>), card_id: Some(card_id) }`.
   Board Rendering reads this and spawns the variant-appropriate ghost
   entity (or applies the appropriate marker component per the §5 variant
   table) at `cell_to_world(lane, cell)` with `Sprite.color.alpha = 0.5`.
2. **Move with cursor.** Subsequent `GhostPlacementChanged` messages
   from Hand UI move / re-spawn the ghost (one ghost per `card_id` at
   any time per GDD Rule 8 + AC BR-8).
3. **Despawn on drop or cancel.** Either:
   - Hand UI writes `GhostPlacementChanged { target: None, card_id: Some(card_id) }`
     (cancel / un-stage), and Board Rendering despawns the ghost.
   - The player commits the placement and `S2CPlacementReveal` is
     received; Board Rendering despawns all ghost preview entities and
     clears all ghost marker components immediately (GDD Rule 8 + AC
     BR-9).
4. **Reverse events to Hand UI.** While a ghost is alive, click and
   mouse-down gestures on the ghost write `GhostClickedEvent { card_id }`
   and `GhostDragStartEvent { card_id }` for Hand UI to consume (GDD
   Rule 8 + AC BR-8e). The actual removal is driven by Hand UI's
   subsequent `GhostPlacementChanged { target: None }` — Board Rendering
   does not auto-remove the ghost on its own click.

The bridge protocol (message ownership, single-drain discipline,
intra-client `Message<T>` vs. Lightyear S2C) is owned by GDD Rule 8 +
BR-004 + ADR-008. This spec is read-only against those.

---

## §8 References to docs/ux/global-ui-design-spec.md

This section enumerates the cross-references this spec depends on,
section by section, with a one-line rationale per reference. It is the
explicit cross-reference enumeration required by AC9 of story 013.

### References

| Global UI spec section | Referenced by | One-line rationale |
|---|---|---|
| `docs/ux/global-ui-design-spec.md` §3 (Z-Index Layer System) | This spec §3 (Cell Rendering Rules) + §4 (Unit Placement Rules) | Canonical `World` (100) / `Units` (200) layer integer values; ADR-021 R2 paint order preserved (world-space sprites paint below bevy_ui regardless of integer values). |
| `docs/ux/global-ui-design-spec.md` §6 (Overlay Alpha Tokens) | This spec §7 (Ghost Preview Opacity) — explicit scope-guard cross-link | `OVERLAY_DIM_ALPHA` / `OVERLAY_SCRIM_ALPHA` are bevy_ui modal-scrim tokens; board ghost preview alpha is **out of scope** for those tokens per the §6 "Scope guard" paragraph. Ghost preview opacity is named in §7 of this spec instead (`GHOST_PREVIEW_ALPHA = 0.5`). |
| `docs/ux/global-ui-design-spec.md` §7 (Color Tokens) | This spec §5 (Range Overlay Rules) + §6 (Status Icon Legend) | Friend-game palette tokens (`SEMANTIC_SUCCESS`, `SEMANTIC_WARNING`, `SEMANTIC_ERROR`, `ACCENT`, `PRIMARY`, `SECONDARY`) referenced for range overlay tints and status icon tints where applicable. Final-art tint values deferred to a follow-on colorization pass; PAW-TD-002-a accept-risk preserved. |
| `docs/ux/global-ui-design-spec.md` §8 (Responsive Layout Rules) | This spec §3 (Cell Rendering Rules) | Canonical 6-viewport matrix (`1366×768` / `1920×1080` / `1920×1200` / `1280×960` / `3840×2160` / `2560×1080`); world board scales with viewport via camera zoom per §8 "World-space sprites (not bevy_ui)" per-class scaling rule. |

### Cross-reference matrix summary

The four cross-references above satisfy AC9 of story 013 (literal
`docs/ux/global-ui-design-spec.md` matches ≥ 4 inside this spec).

---

## §9 ADR / GDD Cross-References

This section enumerates the ADR and GDD source documents this spec
cross-references. All references are **read-only** — this spec does NOT
modify ADRs, the board-rendering GDD, the keyword-system GDD, or any
other GDD.

### ADR cross-references

| ADR | Title | What this spec consumes |
|---|---|---|
| [ADR-021](../architecture/adr-021-presentation-layer-architecture.md) | Presentation Layer Architecture | R2 (world-space 2D sprite rendering; `bevy_ui` never used for board content); paint order (world-space sprites paint below bevy_ui); `Res<BoardLayout>` + `BoardLayout::cell_to_world(lane, cell)` shared coordinate authority; `PresentationSet` ordering (`PhaseTransition → MessageDrain → StateSync → AnimationTick`); `Res<CardAtlas>` single shared atlas resource. |
| [ADR-020](../architecture/adr-020-board-lane-state-architecture.md) | Board / Lane System State Architecture | Authoritative source for `BoardPosition { lane, cell }`, `UnitStats { hp_current, hp_max, owner }`, `ObjectiveHp { hp }` replication; `PlayerSnapshot.spawn_range_cells` snapshot-seed contract for §5; `ResolutionEvent::SpawnRangeChanged` live-update contract for §5. |
| [ADR-017](../architecture/adr-017-combat-resolution-execution-architecture.md) | Combat Resolution Execution Architecture | Resolution sub-step event ordering (`S2CResolutionEvent` `sub_step` 1..=6); animation queue / playback timing contract referenced by §5 (live update consumption ordering) and by the GDD F4 total-duration formula (cited in GDD; not re-specified here). |
| [ADR-011](../architecture/adr-011-reconnect-snapshot.md) | Reconnect and Snapshot | Snapshot rebuild rule (GDD Rule 11); `S2CGameSnapshot` consumption discipline; reconnect timing contract (`objective_identities_reconnect_timeout_ms`). Spawn highlight reconnect-seed source (§5) consumes ADR-011's snapshot envelope. |
| [ADR-008](../architecture/adr-008-lightyear-channel-config.md) | Lightyear Channel Configuration | Reliable channel ordering for `S2CResolutionEvent` (TR-NP-014 live update path); single-drain discipline ("no duplicate `MessageReceiver<S2CPhaseChanged>` or second Lightyear drain" per BR-011 Implementation Notes). |
| [ADR-002](../architecture/adr-002-client-server-authority.md) | Client-Server Authority | Board Rendering is a read-only presentation layer ("the client is a view"); no authoritative state, no game-logic C2S messages (sole exception: `C2SRequestSnapshot` for desync recovery per GDD Rule 1). All visual state in this spec is derived from server-replicated state or transient client-only presentation state. |

### GDD cross-references

| GDD | Path | What this spec cross-references |
|---|---|---|
| Board Rendering (GDD source of truth) | `design/gdd/board-rendering.md` | F1 (`cell_to_world`), F2 (HP bar fill fraction), F3 (co-occupancy offset), Rule 3 (BoardLayout authority), Rule 4 (Z-layer constants), Rule 5 (atlas / draw-call budget), Rule 6 (HP bars), Rule 8 (ghost unit lifecycle), Rule 11 (reconnect rebuild), Rule 12 (objective rendering — ADR-001 isolation), Rule 14 (status effect visual contract), Acceptance Criteria BR-1 / BR-2 / BR-2b / BR-3a / BR-4 / BR-5 / BR-7 / BR-8 / BR-9 / BR-11 / BR-22 / BR-22b / BR-Z-LOCAL / BR-SPAWN-HIGHLIGHTS / BR-STATUS-CONTRACT / BR-STATUS-TIER / BR-STATUS-COOCCUPANCY / BR-COOCC-CONSTRAINT / BR-2-ATLAS / BR-HP-INVARIANT / BR-EC-CARDMISS. |
| Board / Lane System | `design/gdd/board-lane-system.md` | `lane_count = 5`, `cells_per_lane = 8`, board-grid dimensions, spawn-range live source ownership (cross-references §5 source contract). |
| Keyword System | `design/gdd/keyword-system.md` | `display_tier: u8` definitions per keyword (Tier 1 vs Tier 2 sort order in §6); OUTNUMBERED keyword semantics (`OQ-KS5` closed; per-unit; global board count strict less-than). |
| Network Protocol | `design/gdd/network-protocol.md` | `S2CResolutionEvent` (`SpawnRangeChanged` variant ordering per TR-NP-014); `S2CPlacementReveal` / `S2CGameSnapshot` / `GhostPlacementChanged` message contracts. |
| Round State Machine | `design/gdd/round-state-machine.md` | `BoardRenderState` transitions are driven by `S2CPhaseChanged`; this spec consumes the RSM phase enumeration via the network protocol. |

This spec does **not** modify any of these GDDs. All cross-references
are read-only.

---

## §10 Producer Ratification Checklist

Per `production/epics/board-rendering/story-013-board-rendering-spec.md`
AC11 and the Sprint 15 plan, the spec values authored above require
ratification by **producer + UX-designer + art-director** at this
`/dev-story` authoring time. The ratification gates the spec's adoption
by the board-rendering epic stories.

### Sign-off rows

The three sign-off rows below are the AC11 ratification gate. Each is
recorded as ratified at PROMPT 1004 spec authoring with the rationale
captured per-role below.

| Role | Ratified at | Rationale |
|------|-------------|-----------|
| **Producer** | PROMPT 1004 (2026-05-17) | Spec values are either (a) verbatim cross-references to already-shipped Tier 0 modules and ADRs (§3 `World` / `Units` z-layers cross-link to global UI spec §3; §8 cross-references to global UI spec §3 / §6 / §7 / §8; §9 ADR cross-references all read-only) or (b) ratifications of values already shipped by the board-rendering epic (§3 `cell_to_world` cited verbatim from GDD F1; §4 F3 co-occupancy ±half-offset cited verbatim from GDD F3 with index-2 `assert!`; §4 HP bar geometry cited verbatim from GDD Rule 6 + AC BR-Z-LOCAL / BR-HP-INVARIANT; §5 spawn-highlight contract cited verbatim from BR-011 closure; §6 status icon legend cited verbatim from BR-009 closure + GDD Rule 14 R4) or (c) explicit new tokens (`GHOST_PREVIEW_ALPHA = 0.5` in §7, ratified to be the canonical token name for the already-shipped GDD AC BR-11 alpha value). The two folded future-candidate cosmetic captures (`S11-UX-BOARD-STATUS-ICON-LEGEND-001` → §6; `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` → §7) are explicitly closed by this spec landing per the Sprint 15 plan §"Wider Sprint 15 Backlog" note. Producer accepts the folding. Friend-game scope preserved verbatim per §1 + §2; `QA-COND-0005` / `QA-COND-0006` / `PAW-TD-*-a` / `S8-QA-001-W1` not advanced. |
| **UX-designer** | PROMPT 1004 (2026-05-17) | Layout primitives delegate UP cleanly to `docs/ux/global-ui-design-spec.md` (z-layers, color tokens, overlay alpha scope-guard, responsive matrix) and OUT cleanly to ADR-021 / ADR-020 / ADR-017 / ADR-011 / ADR-008 / ADR-002. The §6 status icon legend Tier 1 / Tier 2 mapping is the same priority ordering shipped by BR-009; the §7 ghost preview opacity `0.5` is the same alpha already locked by GDD AC BR-11. The §7 explicit scope-guard cross-link to global UI spec §6 confirms ghost preview alpha is sprite-level (NOT bevy_ui modal scrim), closing the open scope question. Co-occupancy ChildOf hierarchy rule (§4) preserves the BR-009 shipped invariant that status icons and HP bars inherit the parent unit's `Transform.translation.x` (including F3 offset) rather than re-centring on the cell — this is the load-bearing visual correctness behaviour and the spec ratifies it. |
| **Art-director** | PROMPT 1004 (2026-05-17) | `PAW-TD-002-a` … `PAW-TD-006-a` placeholder-art accept-risk preserved verbatim in §1 Status Banner. Cell tile art (§3), unit base ring art (§4), status icon glyph art (§6), and ghost preview art (§7) are friend-game placeholder; final-asset replacement remains a separate sprint scope. Player A circle / Player B hexagon base-ring shape redundancy (GDD §Asset Requirements colorblind-redundancy row) preserved in §4 — shapes are load-bearing, not just decorative, for friend-game-scope colorblind users. Friend-game palette tokens (§5 / §6) cross-reference `docs/ux/global-ui-design-spec.md` §7 without overriding palette values. Z-layer ordering (§3 / §4) preserves the existing PresentationPlugin composition per ADR-021 R2. |

### Ratification scope guard

The above ratification is **specifically scoped to friend-game board
visual polish** per §1 + §2. It does **not** ratify:

- Standard-tier accessibility values on board overlays / status icons /
  ghost previews / cell hit-targets (separate accessibility spec
  required to advance `QA-COND-0005`).
- Final-art atlas frames, palette, font assets (separate sprint scope;
  `PAW-TD-*-a` accept-risk preserved).
- Playtest validation (`QA-COND-0006` accept-risk preserved).
- Per-system GDD edits (read-only cross-link only per §9).
- ADR edits (read-only cross-link only per §9).
- Animation / motion / interaction-state primitives (owned by ADR-017
  and the Sprint 15 Nice to Have
  `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` story).
- HUD spec or hand-UI drag-state spec (owned by their respective Sprint
  14 / Sprint 15 stories per §2 Out of Scope).
- Two-client GAME_OVER closure (`S8-QA-001-W1` remains OPEN).
- Polish → Release gate-check retry (PROMPT 761 `FAIL` preserved; no
  retry in scope).

---

## Spec Adoption Matrix

This matrix maps the spec sections to the board-rendering epic stories
(BR-001 .. BR-012) that consume them. Stories are listed with their
status on `origin/main@84e621e` at this authoring time.

| Section | Consumer story | Status on `origin/main@84e621e` | Notes |
|---|---|---|---|
| §3 Cell Rendering Rules | BR-001 (plugin scaffold, board layout, card atlas) + BR-002 (board grid camera + z-layers) | Complete (BR-001, BR-002) | This spec ratifies the already-shipped 5×8 grid + `cell_to_world` + z-layer constants. No value change requested. |
| §4 Unit Placement Rules | BR-003 (snapshot spawn units, objectives, HP bars) + BR-009 (status icons + co-occupancy) | Complete (BR-003, BR-009) | F3 co-occupancy + ChildOf hierarchy + HP-bar Z (LOCAL 0.1) ratified verbatim from GDD + AC BR-Z-LOCAL / BR-22 / BR-22b / BR-COOCC-CONSTRAINT. |
| §5 Range Overlay Rules | BR-004 (ghost preview Hand UI bridge) + BR-011 (spawn range highlights) | Complete (BR-004, BR-011) | TR-BR-008 + TR-NP-014 spawn-range source contract ratified verbatim from BR-011 closure. |
| §6 Status Icon Legend | BR-009 (status icons + co-occupancy + spawn range) | Complete (BR-009) | Tier 1 / Tier 2 priority ordering ratified verbatim from GDD Rule 14 R4. **Folds `S11-UX-BOARD-STATUS-ICON-LEGEND-001` future candidate as a spec section, not as a separate story.** |
| §7 Ghost Preview Opacity | BR-004 (ghost preview Hand UI bridge) | Complete (BR-004) | `GHOST_PREVIEW_ALPHA = 0.5` ratified as the canonical token for the GDD AC BR-11 alpha value. **Folds `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` future candidate as a spec section, not as a separate story.** |
| §8 References to global UI spec | This spec — all sections cross-link | n/a (cross-reference enumeration only) | Four cross-references: global UI spec §3 / §6 / §7 / §8. Satisfies AC9 ≥ 4 matches. |
| §9 ADR / GDD Cross-References | This spec — all sections cross-link | n/a (read-only) | Six ADR cross-links (ADR-021, ADR-020, ADR-017, ADR-011, ADR-008, ADR-002) + five GDD cross-links. Satisfies AC10. |
| §10 Producer Ratification Checklist | This spec | n/a (ratification gate) | Producer + UX-designer + art-director sign-off rows recorded at PROMPT 1004. Satisfies AC11. |

### Stories NOT consumed by this spec (out-of-scope per §2)

| Out-of-scope surface | Owning story | Reason |
|---|---|---|
| HUD top strip + bottom strip + opponent figurine | Sprint 14 `S11-UX-HUD-TOP-STRIP-LAYOUT` / `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` / `S11-UX-HUD-OPP-FIGURINE` (DONE) | HUD is not board-rendering; owned by Sprint 14. |
| Hand UI drag-state visuals | Sprint 15 Should Have `S12-UX-HAND-DRAG-STATE-VISUALS-001` (separate story) | Hand-side drag visual is not board-side ghost preview. |
| Interaction-state primitives | Sprint 15 Nice to Have `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` (separate story) | Hover / focus / pressed / disabled is not in board-rendering scope. |
| Animation curves / reveal-tween easing | ADR-017 + BR-005 (placement reveal tween) / BR-006 (resolution playback) | Owned by their existing stories. |
| Reconnect / snapshot rebuild | ADR-011 + BR-007 (reconnect) / BR-008 (objective reveal + HUD fan-out) | Owned by ADR-011; protocol gaps on BR-007 / BR-008 remain open. |

---

## Cross-References

Related artifacts (read-only links):

- `production/epics/board-rendering/story-013-board-rendering-spec.md`
  — story file (AC1-AC16).
- `production/epics/board-rendering/EPIC.md` — epic-level Board
  Rendering charter.
- `production/sprints/sprint-15.md` — Sprint 15 plan (active per
  PROMPT 997).
- `production/qa/qa-plan-sprint-15.md` — Sprint 15 QA plan
  (authored per PROMPT 1002).
- `production/qa/evidence/sprint-15-board-rendering-spec/` — AC1-AC16
  doc-review checklist evidence (authored by this prompt).
- `docs/ux/global-ui-design-spec.md` — parent design-spec doc (Sprint
  14 `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`; DONE per PROMPT 911 / 912 / 922).
- `docs/ux/ui-clean-pass-roadmap.md` — rank 14 sequencing entry.
- `docs/architecture/adr-021-presentation-layer-architecture.md` — §3
  Cell Rendering + §4 Unit Placement + §9 cross-link.
- `docs/architecture/adr-020-board-lane-state-architecture.md` — §5
  Range Overlay source contract + §9 cross-link.
- `docs/architecture/adr-017-combat-resolution-execution-architecture.md`
  — §5 Range Overlay ordering + §9 cross-link.
- `docs/architecture/adr-011-reconnect-snapshot.md` — §5 Range
  Overlay snapshot-seed + §9 cross-link.
- `docs/architecture/adr-008-lightyear-channel-config.md` — §5 Range
  Overlay single-drain discipline + §9 cross-link.
- `docs/architecture/adr-002-client-server-authority.md` — §9
  cross-link (client-as-view).
- `design/gdd/board-rendering.md` — GDD source of truth (read-only).
- `design/gdd/board-lane-system.md` — board grid dimensions (read-only).
- `design/gdd/keyword-system.md` — `display_tier` definitions per
  keyword (read-only).
- `design/gdd/network-protocol.md` — message contracts referenced by
  §5 / §6 / §7 (read-only).
- BR-009 closure notes for status icon implementation surface.
- BR-011 story file for spawn range data-source contract.

---

## Authoring Trail

| Field | Value |
|-------|-------|
| **Authoring prompt** | PROMPT 1004 (`/dev-story` for story 013) |
| **Worker branch** | `work/s15-board-rendering-spec` |
| **Worktree** | `D:/_DEV/claude-code-game-studios-worktrees/s15-board-rendering-spec-1004` |
| **Source-of-truth at authoring** | `origin/main@84e621e` (PROMPT 1002 Sprint 15 QA plan authored on top of PROMPT 997 Sprint 15 activation) |
| **Files authored by PROMPT 1004** | `docs/ux/board-rendering-spec.md` (NEW) + `production/qa/evidence/sprint-15-board-rendering-spec/doc-review-checklist.md` (NEW) + `production/qa/evidence/sprint-15-board-rendering-spec/cross-ref-matrix.md` (NEW) + `production/qa/evidence/sprint-15-board-rendering-spec/ratification.md` (NEW) |
| **Files explicitly NOT changed by PROMPT 1004** | `client/**`, `server/**`, `shared/**`, `tests/**`, `Cargo.toml`, `Cargo.lock`, `production/sprint-status.yaml`, `production/sprints/sprint-15.md`, `production/stage.txt`, `production/session-state/**`, `production/qa/qa-plan-sprint-15.md`, story-013 file body (the story file is not edited by this prompt), other `production/epics/board-rendering/` story files (BR-001..BR-012 untouched), all ADRs (read-only cross-link only), all GDDs (read-only cross-link only) |
