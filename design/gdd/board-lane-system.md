# Board / Lane System

> **Status**: Approved — /design-review complete 2026-04-28; all blocking items resolved
> **Author**: User + Agents
> **Last Updated**: 2026-05-01
> **Implements Pillar**: Simple surface · Deep emergence · No idle spectating

## Overview

The Board/Lane System defines the spatial model of Lanes and Lies: a 5-lane arena where every lane is an 8-cell corridor connecting two opposing sides. It is both data infrastructure — the authoritative grid that Combat Resolution, Keyword execution, and Board Rendering all query for positions, occupancy, and pathfinding — and the game's primary decision surface, the place where players read the current state, choose where to commit units, and predict the opponent's intentions.

Each lane operates independently. A player occupies 4 cells on their side (Cells 1–4); the opponent owns Cells 5–8 (mirrored). Cell 1 is the spawn point; Cell 8 holds the lane's objective. Units placed during PLACEMENT enter at a cell within the player's current spawn range (determined by how many fake objectives they've destroyed), then advance toward Cell 8 each round by their MP value. The board tracks occupancy by card type: at most 1 Minion per player per lane, while Traps, Structures, and Fields have separate placement rules and do not compete for the Minion slot. The Board/Lane System owns all occupancy validation, spawn-range state, movement execution, and the authoritative list of what is where at every moment of the game.

## Player Fantasy

The board is the one place your opponent cannot lie. Their hand is hidden, their bid is hidden, two of their objectives are fakes — but the units they have placed are physical evidence of intent. Each PLACEMENT phase, the player scans all five lanes as a war-room commander reading dispositions: which lane has the opponent reinforced, which have they abandoned, where is their advancing line going to be next round. The fantasy is forensic — *I can see where they committed last round. Movement is deterministic: I know where that unit will be. My question is: what did they add on top of that?* Then both players place blind, and the board reveals whether either of them read it right.

## Detailed Rules

### Core Rules

**The Board/Lane System owns:**
- The authoritative 5-lane × 8-cell spatial grid (server-side only; clients receive replicated read-only view)
- Each unit's current lane and cell position
- Occupancy tracking for Minions, Traps, Structures, and Fields per lane/cell/player
- `SpawnRangeState` per player: the live authoritative spawn range projection used for placement validation, snapshot assembly, and `SpawnRangeChanged` resolution-log events
- Prism presence state per lane per player
- A pending-placement buffer (committed at RESOLUTION start)

The Board/Lane System does NOT own: combat damage, card stats, HP totals, objective HP, objective destruction facts/counters, or keyword effect logic. Objective System owns fake/real destruction records; Board/Lane consumes those facts to update its live spawn range projection.

---

**Rule 1 — Coordinate system:**
Cells are numbered 1–8 in absolute terms. The lane's left edge is cell 1; the right edge is cell 8.
- **Player A** (left side): home half = cells 1–4; spawn cell = 1; objective target = cell 8. Advances by **+1 per MP** per round.
- **Player B** (right side): home half = cells 5–8; spawn cell = 8; objective target = cell 1. Advances by **−1 per MP** per round.

"Your Cell N" is a relative display convention for players only. All server logic uses absolute cell numbers.

**Rule 2 — Board structure:**
5 independent lanes. Each lane has 8 cells. Lane and cell state are separate — a unit's position is (lane: 1–5, cell: 1–8). Lanes do not share unit positions; only the CHANGE LANE keyword moves units between lanes.

**Rule 3 — Minion slot limit:**
Each player has exactly 1 Minion slot per lane. The slot is occupied when a Minion is on the board in that lane for that player; it becomes free when the Minion is killed or returned to hand. In 2v2, each team has 2 Minion slots per lane (1 per player on the team).

A Minion placement is rejected server-side if the player's Minion slot for that lane is already occupied.

**Rule 4 — Spawn range:**
During PLACEMENT, a Minion may only be placed within the player's current spawn range (cells available for placement on their home side):

| `fake_objectives_destroyed` (by this player) | Cells available for Minion placement |
|---|---|
| 0 | Cell 1 only (absolute for Player A; cell 8 for Player B) |
| 1 | Cells 1–2 (absolute for Player A; cells 7–8 for Player B) |
| 2 | Cells 1–3 (absolute for Player A; cells 6–8 for Player B) |

Spawn range is **global** — it applies to all 5 lanes simultaneously.

`SpawnRangeState` is the single live source for this projection. Objective System owns `fake_objectives_destroyed(player)` as a destruction fact/counter, but it does not compute live spawn range cells and does not transport spawn range to clients. Board/Lane updates `SpawnRangeState`, uses it for validation, supplies `PlayerSnapshot.spawn_range_cells` during snapshot assembly, and contributes ordered `ResolutionEvent::SpawnRangeChanged { player_id, new_spawn_range_cells }` entries to the reliable `S2CResolutionEvent` batch.

**Exceptions to spawn range:**
- **Traps**: may be placed on any of the player's 20 home cells, regardless of spawn range
- **Structures**: may be placed on any of the player's 20 home cells, regardless of spawn range
- **Fields**: lane-wide effect, no cell position — placed on a lane, not a cell

**Rule 5 — Cell occupancy limits by card type:**

| Card type | Limit | Placement zone |
|---|---|---|
| Minion | 1 per player per **lane** | Within spawn range |
| Trap | 1 per player per **cell** | Any of player's 20 home cells |
| Structure | 1 per player per **cell** | Any of player's 20 home cells |
| Field | 1 per player per **lane** | Lane-wide; no cell |

Multiple units from different players (or 2v2 teammates) **may share a cell**. Allied units sharing a cell are visually rendered side-by-side (Board Rendering concern); in data they occupy the same absolute cell. Exception: ATTRACT displacement cannot place an opposing unit onto the caster's cell — see Rule 9a.

**Rule 6 — Pending placement buffer:**
During the PLACEMENT phase, submitted placements are validated and held in a per-player pending buffer — they are **not immediately committed to the board**. This preserves the simultaneous-reveal invariant: neither player can observe the other's placement until RESOLUTION begins. The buffer commits atomically at the start of sub-step 1 when RESOLUTION begins. Partial submissions (received before timer expiry) commit; unsubmitted slots are treated as no card played.

**Architecture note (server-side):** The buffer is a server-only data structure (`PlacementBuffer` resource), not Bevy entities. Unit entities are spawned only at sub-step 1 commit, at which point they are added to the Lightyear replication group. Buffer contents never exist in the ECS world as entities, preventing any accidental replication to clients before the reveal. Mana split validation occurs at submission time, but mana is deducted at PLACEMENT close before `S2CPlacementReveal` is enqueued. If GAME_OVER fires before the close sequence, the session ends with no refund work because no placement spend has been applied yet.

**Rule 7 — Movement execution (Resolution sub-steps):**
The Board/Lane System executes the following spatial operations during RESOLUTION, called by Combat Resolution (which sequences the sub-steps):

| Sub-step | Owner | Board/Lane action |
|---|---|---|
| 1. Placement effects (APPEARANCE) | Combat Resolution | Commits pending placements; APPEARANCE triggers fire |
| 2. CHARGE X bonus movement | Board/Lane System | Executes bonus advancement for all CHARGE X units across all lanes simultaneously |
| 3. FIRST STRIKE attacks | Combat Resolution | Board provides `get_units_at_cell(lane, cell)` queries |
| 4. Remove dead units | Board/Lane System | Removes units from grid; clears Minion slots; receives death list from Combat Resolution |
| 5. Standard movement | Board/Lane System | Advances all surviving units by their MP value across all lanes simultaneously |
| 6. Standard combat + objective damage | Combat Resolution | Board provides units-at-same-cell and units-at-objective-cell queries |

**Prism collection fires during sub-step 5**: when a unit ends standard movement at the prism cell for that player-side, `PrismCollected(player, lane)` is emitted and the prism token is removed from that lane-side.

**Rule 8 — Standard movement formula:**
```
new_cell = clamp(current_cell + advance_direction(player) × mp, 1, 8)
```
A unit at cell 8 (objective cell for Player A) with MP=3 stays at cell 8 — it does not overshoot. A unit at the objective cell attacks each round until killed.

**Rule 9 — Displacement keyword rules:**
- **REPEL X**: pushes target X cells toward their own spawn. Clamped: cannot push a unit past its own spawn cell.
- **ATTRACT X**: pulls target X cells toward the caster. Clamped to board bounds [1, 8]. Enemy targets halt 1 cell short of the caster's cell — see Rule 9a.
- **TELEPORT**: repositions a unit to a specified cell. Target cell range is defined per-card (card-level specification, not board-level).
- **CHANGE LANE**: moves unit to the adjacent lane (±1). Clamped to lanes 1–5. A unit in lane 1 cannot CHANGE LANE leftward; lane 5 cannot CHANGE LANE rightward — the attempt is a silent no-op.
- **IRREMOVABLE**: REPEL, ATTRACT, and CHANGE LANE have no effect. The displacement is silently discarded; the unit does not move.

**Rule 9a — ATTRACT: opposing unit co-occupation prohibition (1-cell-apart rule):**
When ATTRACT pulls an enemy unit toward the caster, the enemy unit cannot land on the caster's cell — it halts 1 cell short on the approach side. This is the authoritative board-level definition of the spatial constraint; the Keyword System GDD (Formula 2 in `keyword-system.md`) encodes the same rule as the canonical implementation formula:

```
effective_pull    = min(X, max(0, |caster_cell − target_cell| − 1))
attract_destination = target_cell + sign(caster_cell − target_cell) × effective_pull
```

**Scope — this rule applies only to ATTRACT displacement of enemy targets.** It does NOT restrict:
- **Standard movement (sub-step 5) and CHARGE X (sub-step 2):** opposing units may naturally share a cell after simultaneous independent movement — this is the condition that triggers standard combat in sub-step 6.
- **TELEPORT:** may place a unit on any valid cell regardless of enemy occupancy; TELEPORT explicitly bypasses this rule.
- **REPEL:** pushes away from the caster — the caster's cell is never the REPEL destination.
- **Friendly ATTRACT targets:** same-player units may stop at the caster's cell (Rule 5 co-occupation is permitted).

---

**Rule 10 — Trap trigger:**
A Trap triggers when an **enemy** unit enters the Trap's cell, regardless of how the unit entered (standard movement, CHARGE X movement, or forced displacement via REPEL/ATTRACT). Trigger fires immediately when the unit occupies the cell, before the current sub-step completes. A triggered Trap is removed from the board.

**Rule 11 — Prism positions:**
Each player has 1 prism per lane, located at their own spawn cell:
- Player A's prisms: absolute cell 1, one per lane (5 prisms total for Player A)
- Player B's prisms: absolute cell 8, one per lane (5 prisms total for Player B)
- Total across the board: 10 prisms (2 per lane)

A prism is collected — token removed, `PrismCollected(player, lane)` emitted — when a unit belonging to that player ends sub-step 5 (standard movement) at the prism's cell. The Prism System handles all reward delivery. Each player's 5 prisms respawn independently when that player has collected all 5 of their own prisms. The opponent's collection pace does not affect the other player's respawn cycle.

**Rule 12 — Board cleanup (OnResolutionEnd):**
After all 6 sub-steps complete and the RSM fires `OnResolutionEnd`:
1. Any units still at 0 HP that were not removed in sub-step 4 are swept (safety net)
2. Minion slots whose units no longer exist are cleared to empty
3. Prism state consistency check

---

### States and Transitions

The Board/Lane System has no internal state machine — it is persistent spatial data. It operates in two externally-driven contexts:

| Context | Driven by | Board role |
|---|---|---|
| **PLACEMENT validation** | RSM (during PLACEMENT phase) | Validates and queues submitted placements; rejects invalid placements |
| **RESOLUTION execution** | Combat Resolution (sub-step calls) | Executes movement commands; answers spatial queries; fires prism events |
| **Passive persistence** | Always (between phases) | Holds board state between rounds; no transforms during DRAFT |

---

### Interactions with Other Systems

| System | Direction | Interface |
|---|---|---|
| **RSM** | RSM → Board | RSM fires `OnResolutionEnd` (board cleanup); RSM PLACEMENT state gates placement submission |
| **Combat Resolution** | Combat ↔ Board | Combat sequences sub-steps; Board executes `move_unit`, `remove_unit`, `change_lane`, `teleport_unit`; Board answers `get_units_at_cell(lane, cell)` and `get_units_at_objective_cell(player)` |
| **Objective System** | Bidirectional facts | Board fires `UnitAtObjective(unit_id, lane)` at sub-step 6 end; Objective System applies damage and owns destruction facts/counters. Board/Lane reads or receives fake-destruction facts to update `SpawnRangeState`; Objective System never owns the live spawn range projection. |
| **Prism System** | Board → Prism | Board fires `PrismCollected(player, lane)` during sub-step 5 when prism condition met |
| **Economy System** | Objective/Board → Economy | Economy receives gold and mana-cap reward events from Objective System; it does not own or notify spawn range changes |
| **Keyword System** | Keyword ↔ Board | Keyword System calculates displacement deltas; Board executes movement via spatial API and enforces bounds/IRREMOVABLE |
| **Card Data & Pool** | Board reads | Board reads ATK, HP, MP, AR, keyword list from card definition at unit spawn time |
| **Network Protocol / Lightyear** | Board → Clients | Sub-step 1 commit sends a single `S2CPlacementReveal { placements: Vec<PlacedUnit> }` payload (all placed units for both players in one message) — not per-entity component replication — to guarantee atomic client-side reveal. Subsequent position updates use Lightyear component replication. Resolution events are delivered as an ordered `S2CResolutionEvent` replay log. Spawn range changes are delivered as `ResolutionEvent::SpawnRangeChanged` entries ordered after the corresponding `ObjectiveDestroyed`; recovery/reconnect uses `PlayerSnapshot.spawn_range_cells`. Do not add a replicated `SpawnRange` component unless future docs explicitly reverse the decision. |
| **Board Rendering** | Board → Rendering | Rendering reads unit positions from replicated components, seeds spawn highlights from `PlayerSnapshot.spawn_range_cells`, and updates live highlights from ordered `SpawnRangeChanged` entries; allied co-occupancy rendered side-by-side |

## Formulas

### F1 — Standard Unit Movement

The `unit_movement` formula is defined as:

```
new_cell = clamp(current_cell + direction(player) × mp, 1, 8)
```

where:
```
direction(Player A) = +1   (advances toward cell 8)
direction(Player B) = −1   (advances toward cell 1)
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Current cell | `current_cell` | u8 | 1–8 | Unit's absolute cell before movement |
| Movement points | `mp` | u8 | 0–6 | From card definition; 0 = WALL |
| Player direction | `direction` | i8 | {+1, −1} | +1 for Player A; −1 for Player B |

**Output range:** 1–8. A unit cannot leave the board. A unit already at the objective cell (8 for Player A, 1 for Player B) with any MP stays at that cell — it does not overshoot.

**Worked examples:**
- Player A unit at cell 2, MP=3: `clamp(2 + 3, 1, 8) = 5`
- Player A unit at cell 7, MP=3: `clamp(7 + 3, 1, 8) = 8` (clamped at objective)
- Player B unit at cell 5, MP=2: `clamp(5 − 2, 1, 8) = 3`
- WALL unit (MP=0): `clamp(cell + 0, 1, 8) = cell` (no movement)

**Rust implementation note:** Cast all operands to `i16` before arithmetic to prevent compile errors and debug-mode underflow panics on boundary values:
```rust
new_cell = clamp(current_cell as i16 + direction as i16 * mp as i16, 1, 8) as u8
```

**Direction argument by use case:**

| Use case | `direction` argument to F1 |
|---|---|
| Standard movement (sub-step 5) | `advance_direction(unit.owner)` |
| CHARGE X bonus movement (sub-step 2) | `advance_direction(unit.owner)` |
| REPEL X | `−advance_direction(target.owner)` (opposite of target's advance direction) |
| ATTRACT X | `sign(caster_cell − target_cell)` (toward caster; enemy target: halts 1 cell short of caster — see Rule 9a; if `caster_cell == target_cell`, sign = 0 → no movement) |

The same formula applies to all four use cases using the appropriate `direction` from the table above.

---

### F2 — Spawn Range Validation

A Minion placement cell is valid if:

```
// Named constants (loaded from GameConfig — not hardcoded in logic):
spawn_cell_A = 1   // Player A's absolute spawn cell
spawn_cell_B = 8   // Player B's absolute spawn cell

// Player A (home side = cells 1–4, spawn at cell 1):
valid_placement(Player A, target_cell) = (target_cell >= spawn_cell_A) AND (target_cell <= spawn_cell_A + fakes_A)

// Player B (home side = cells 5–8, spawn at cell 8):
valid_placement(Player B, target_cell) = (target_cell >= spawn_cell_B − fakes_B) AND (target_cell <= spawn_cell_B)
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Fakes destroyed | `fakes_A` / `fakes_B` | u8 | 0–2 | Opponent fake objectives this player has destroyed |
| Target cell | `target_cell` | u8 | 1–8 | Absolute cell the player wants to spawn into |

**Output:** boolean (placement accepted or rejected).

| `fakes` | Player A valid cells | Player B valid cells |
|---|---|---|
| 0 | {1} | {8} |
| 1 | {1, 2} | {7, 8} |
| 2 | {1, 2, 3} | {6, 7, 8} |

Structures and Traps bypass this formula — they can be placed on any of the player's 20 home cells.

---

### F3 — Objective Cell Detection

A unit is at the objective cell (attacks the objective in sub-step 6) when:

```
at_objective(unit) =
    ((unit.owner == Player A) AND (unit.cell == 8))
    OR
    ((unit.owner == Player B) AND (unit.cell == 1))
```

**Output:** boolean. Checked at the end of sub-step 6 for every alive unit. Units at the objective cell deal their base ATK as damage to the objective each round and remain there until killed.

**Worked examples:**
- Player A unit at cell 8 → `true`
- Player B unit at cell 1 → `true`
- Player A unit at cell 1 → `false` (Player A's objective is at cell 8; cell 1 is their prism cell, not objective)
- Player B unit at cell 8 → `false` (Player B's objective is at cell 1)

## Edge Cases

**If a unit advances multiple cells in one sub-step** (e.g., MP=4 or CHARGE X=3): the unit teleports directly to its final cell. Intermediate cells are never "visited" and do not trigger Traps. Only the final destination cell is occupied at end of movement.

**If a fake objective is destroyed during RESOLUTION** (sub-step 6 objective damage): the attacker's spawn range expansion takes effect at the start of the **next round's PLACEMENT phase** — not the current round. Placement validation (Formula F2) runs only during PLACEMENT; the current round's placements were already committed at sub-step 1.

The update still becomes visible to connected clients in the current RESOLUTION's reliable event batch: Board/Lane updates `SpawnRangeState` after receiving the fake-destruction fact and emits `SpawnRangeChanged` after the matching `ObjectiveDestroyed` entry. Clients use the new range to render the next actionable PLACEMENT state. Snapshot builders must read `SpawnRangeState`, not recompute directly from `ObjectiveCounters`, so reconnect state cannot drift from live validation state.

**If Strich is in lane 1 and an enemy unit enters lane 1**: Strich's CHANGE LANE auto-trigger fires but targets lane 0 (does not exist). The attempt is a silent no-op. Strich stays in lane 1. The same applies in lane 5 (cannot change rightward).

**If Strich attempts CHANGE LANE and both adjacent lanes already contain the player's own Minion**: Strich is blocked — it stays in its current lane. No error is raised. A full adjacent lane is treated identically to a boundary (no valid destination = no-op).

**If two of the same player's units CHANGE LANE into the same destination lane simultaneously**: the unit from the lower lane number succeeds and occupies the slot. The unit from the higher lane number is blocked — its CHANGE LANE is a silent no-op and it stays in its original lane. Lane-order tiebreak (lower lane = priority) applies.

**If REPEL or ATTRACT pushes a unit into a cell containing an enemy Trap**: the Trap triggers. "Enemy" in the Trap trigger rule is relative to the Trap's owner, not to the displacement source. A unit pushed by any effect into a Trap cell triggers it.

**If a unit is TELEPORTed to its own spawn cell (Player A to cell 1 / Player B to cell 8)**: no prism is collected. Prism collection is gated to sub-step 5 (standard movement) end-position only. Units arriving at the prism cell via TELEPORT, REPEL, ATTRACT, or any non-sub-step-5 mechanism do not collect the prism.

**If a unit with both CHARGE X and MP > 0 is on the board**: it advances X cells in sub-step 2 and MP cells in sub-step 5. These are independent movements. F1 is applied independently at each sub-step using the unit's current cell as input. A unit with CHARGE 2 and MP=3 advances from cell 1 to cell 3 in sub-step 2, then from cell 3 to cell 6 in sub-step 5.

**If a unit is killed in sub-step 1 (by an APPEARANCE trigger)**: it is removed from the board before sub-step 2 begins. The CHARGE X pass in sub-step 2 does not affect removed units.

**If a Player B unit reaches cell 1**: it attacks Player A's objective (F3). Cell 1 is Player A's prism cell, not Player B's. No prism collection fires for Player B — Player B's prism is at absolute cell 8, and collection requires the unit and prism to belong to the same player.

**If a WALL unit (MP=0) is parked at its owner's spawn cell**: it does not advance in sub-step 5, ending at the prism cell. The prism is collected. A WALL is the most reliable way to farm prisms — it holds the spawn cell every round without advancing.

**If a player attempts to place a second Trap on a cell they already have a Trap on**: the placement is rejected server-side. Gold is not deducted.

**If a player attempts to place a Minion when their lane's Minion slot is already occupied**: the placement is rejected. Gold is not deducted.

**If a unit with CHARGE X advances through a Trap cell in sub-step 2 without landing on it**: the Trap does not trigger. CHARGE X movement in sub-step 2 follows the same skip rule as standard movement in sub-step 5 — only the final destination cell of the sub-step is treated as occupied. A unit CHARGEing from cell 1 to cell 5 does not trigger Traps at cells 2, 3, or 4.

**If a unit with CHARGE X reaches the objective cell (cell 8 for Player A, cell 1 for Player B) during sub-step 2**: the unit remains at that cell through sub-step 5 (clamped by F1). `UnitAtObjective` does NOT fire at sub-step 2 — the event is gated to sub-step 6 end only (Rule 7). Objective damage applies normally at sub-step 6 regardless of which sub-step brought the unit to that cell.

**If two enemy units arrive at the same Trap cell in the same inter-sub-step pass** (e.g., two units from different original lanes both CHANGE LANE to the same destination cell in the same pass): the Trap triggers exactly once. Tiebreak: the unit originating from the lower lane number is the triggering unit — consistent with the CHANGE LANE tiebreak in Rule 9. The Trap is removed after the single trigger. The second unit enters the cell and is not affected by the (already-triggered) Trap.

## Dependencies

### Upstream Dependencies

| System | Type | Interface | Notes |
|---|---|---|---|
| **Card Data & Pool** (Approved) | Hard | Board reads `atk`, `hp`, `mp`, `ar`, `keywords` from card definition at unit spawn time | Card schema owns all base stats; Board tracks current HP separately for damage |
| **Game Config** (Designed) | Hard | Board reads `fake_objective_spawn_advance` (1 cell) to update `SpawnRangeState` on fake destruction | Spawn range expansion is data-driven, not hardcoded |
| **Round State Machine** (Designed) | Hard | RSM drives PLACEMENT and RESOLUTION phase boundaries; Board validates placements during PLACEMENT and executes sub-steps during RESOLUTION | RSM provisional interface finalized here: Board listens for `OnResolutionEnd`; provides placement validation API during PLACEMENT |

### Downstream Dependents

| System | Type | Interface | Notes |
|---|---|---|---|
| **Combat Resolution** *(Not Started)* | Hard | Combat sequences all 6 sub-steps; Board exposes `move_unit`, `remove_unit`, `change_lane`, `teleport_unit`, `get_units_at_cell`, `get_units_at_objective_cell` | Combat Resolution is the orchestrator; Board is the spatial executor |
| **Objective System** *(Not Started)* | Hard | Board fires `UnitAtObjective(unit_id, lane)` at sub-step 6 end; Objective System exposes fake/real destruction facts/counters | Objective System applies damage and checks loss condition; Board does not own objective HP or counters, and Objective does not own live spawn projection |
| **Prism System** *(Not Started — M3)* | Soft | Board fires `PrismCollected(player, lane)` during sub-step 5 when a player's unit ends movement at their prism cell | Prism System handles all reward delivery |
| **Economy System** (Designed) | Soft | Economy receives objective gold and mana-cap reward events; no spawn range interface | Spawn range is Board/Lane projection from Objective destruction facts, not an Economy notification |
| **Keyword System** *(Not Started — M3)* | Soft | Keyword System computes displacement deltas; Board executes and enforces IRREMOVABLE/bounds | Board provides spatial API; Keyword System owns keyword logic |
| **Board Rendering** *(Not Started)* | Soft | Rendering reads replicated `BoardPosition` components and protocol-delivered spawn range (`PlayerSnapshot.spawn_range_cells` + `SpawnRangeChanged`) | Board provides data; Rendering owns all visual presentation |
| **Network Protocol / Lightyear** *(Not Started)* | Hard | Unit positions replicated via Lightyear component replication; unit death, trap trigger, and `SpawnRangeChanged` events sent through reliable `S2CResolutionEvent` | Client holds read-only board mirror; no replicated `SpawnRange` component |

## Tuning Knobs

Most Board/Lane parameters are structural constants — they define the physical arena. Changing them requires redesigning Board Rendering, Objective System, and Prism System in tandem. They are listed here for completeness; they should not be adjusted independently.

| Knob | Default | Safe Range | Too Low | Too High |
|---|---|---|---|---|
| `lane_count` | 5 | 3–7 | Fewer strategic fronts; less tension and variety | Too many fronts to track simultaneously; cognitive overload |
| `cells_per_lane` | 8 | 6–10 | Units reach objectives too fast; no mid-board tension | Too much travel distance; games drag in early rounds |
| `spawn_range_default` | 1 cell | 1–2 | N/A — 1 is the minimum | Starting at 2 reduces the earned value of fake objective destruction |
| `spawn_range_max` | 3 cells | 2–4 | Less reward differentiation from destroying both fakes | Units can spawn deep enough to immediately threaten objectives |

**Cross-referenced constants (owned by Game Config — not tunable here):**

| Constant | Value | Source |
|---|---|---|
| `fake_objective_spawn_advance` | 1 cell per fake | game-config.md |
| `fake_count` | 2 of 5 objectives | game-config.md |

`lane_count` and `cells_per_lane` are structural — only change before dependent GDDs are authored.

## Visual/Audio Requirements

**Board Layout:**
- 5-lane isometric stone arena, 3/4 top-down angle. Each lane is a paved stone corridor between two objective pedestals; lanes separated by raised stone dividers
- Cell positions marked by diamond-shaped glowing cyan-blue nodes along each lane's centerline, gently pulsing at idle
- Player A's half (cells 1–4): cool-toned ambient lighting (blues, teals). Player B's half (cells 5–8): warm-toned (amber, orange). The cell 4/5 boundary is the most strongly lit seam — communicates contested territory at a glance
- Objective pedestals: stone with flame totem. Real = large golden flame. Fake = unlit stone egg with "?" marker (see reference `slide4_objectives_real_fake.png`). Larger than units to read as landmarks
- Lane numbers 1–5 displayed at both ends of the board in high-contrast text outside the play area

**Unit Placement and Presence:**
- Unit sprite centered on cell node; team-colored base ring — shape redundancy for colorblind: circle base = Player A; hexagon/diamond base = Player B
- Objective-attack state (unit at cell 8 for Player A / cell 1 for Player B): unit shifts forward, red-orange pulsing aura, impact ring above objective pedestal
- 2v2 co-occupancy: two allied units displayed side-by-side within the cell, shared base plate visually groups them; neither unit obscured

**Spawn Range Feedback:**
- Available cells: node shifts from cyan to warm gold-white glow
- Unavailable cells: node brightness reduced ~50%, stone texture desaturated
- Spawn range expansion event: radial gold pulse animation on newly unlocked cells

**Phase States:**
- DRAFT: board at low idle intensity — backdrop to shop/auction UI
- PLACEMENT: opponent's half covered by translucent dark-blue fog (~60% opacity); own placed units visible on own half only; spawn cells glow gold; placement timer ring on board frame
- RESOLUTION reveal: fog lifts simultaneously across both halves (fast sweep animation); all pending units appear in a single simultaneous beat; units then slide to their new positions

**Special Card Types on Board:**
- Trap: face-down card tile lying flat on cell, team-color glow ring, identity hidden until triggered
- Structure: upright face-up card/token on cell; castle-icon badge in corner distinguishes from unit
- Field: translucent colored wash over entire lane (color keyed to class/element); Field icon badge at lane edge; wash is bottom visual layer beneath units and nodes
- Layering order: Field wash → cell nodes → Traps/Structures → units → UI overlays

**Prism Visual:**
- Small faceted crystal orb on spawn cell node, slow rotation with inner-light sparkle cycle
- Collection: burst of light rays, orb dissolves upward in particle trails, collecting unit gets brief shimmer (~0.4 seconds total)

**Audio Cues:**
- Unit placed (own, PLACEMENT): soft stone-thud/card-snap — confirming, low-key, must not telegraph placement to opponent
- Simultaneous reveal (RESOLUTION start): sharp "veil lift" whoosh/chord sting — the signature audio moment of each round
- Unit advance: short footstep-shuffle per movement; slight per-lane audio offset prevents noise accumulation when multiple lanes resolve
- Prism collected: bright crystalline chime (high register, clean, distinct from combat audio)
- Trap triggered: percussive hit + class-themed accent, near-simultaneous with card flip reveal
- Objective attacked: heavy deep thud; if destroyed, explosion/shatter with musical hit

📌 **Asset Spec** — Visual/Audio requirements are defined. After the art bible is approved, run `/asset-spec system:board-lane-system` to produce per-asset visual descriptions, dimensions, and generation prompts from this section.

## UI Requirements

| UI element | Driven by | Notes |
|---|---|---|
| Board grid (5×8 cells) | `BoardPosition` components + board state | Board Rendering GDD owns visual implementation; Board/Lane provides authoritative data |
| Spawn range highlights | `SpawnRangeState` via snapshot + `SpawnRangeChanged` | During PLACEMENT: cells within range glow gold |
| Placement fog (opponent's half) | RSM PLACEMENT phase signal | Dark-blue fog over opponent's half; lifted on RESOLUTION reveal |
| Placement timer ring | RSM `placement_timer_seconds` | Board frame countdown; driven by RSM, displayed in board border UI |
| Placement rejection feedback | Board validation result | Red flash on attempted cell when placement rejected (full slot or invalid cell) |
| Prism collection notification | `PrismCollected(player, lane)` event | Crystal burst animation; Prism System drives reward text |
| Lane labels (1–5) | Static, per board setup | Persistent; outside play area; accessible text |

📌 **UX Flag — Board / Lane System**: This system has UI requirements. In Pre-Production, run `/ux-design` to create a UX spec for the board view and placement interaction before writing board-rendering epics. Stories referencing board UI should cite `design/ux/board-view.md`, not this GDD directly.

## Acceptance Criteria

| # | Criterion | Type |
|---|---|---|
| BL-1 | GIVEN Player A's unit is in lane 1 at cell 1 with MP=3, WHEN sub-step 5 fires, THEN unit position = cell 4. | BLOCKING |
| BL-2 | GIVEN Player A's unit is in lane 1 at cell 6 with MP=3, WHEN sub-step 5 fires, THEN unit position = cell 8 (clamped — does not overshoot). | BLOCKING |
| BL-3 | GIVEN Player B's unit is in lane 1 at cell 5 with MP=2, WHEN sub-step 5 fires, THEN unit position = cell 3. | BLOCKING |
| BL-4 | GIVEN Player A's WALL unit (MP=0) is in lane 1 at cell 1, WHEN sub-step 5 fires, THEN unit position = cell 1 (no movement). | BLOCKING |
| BL-5 | GIVEN Player A has 0 fakes destroyed, WHEN they attempt Minion placement at cell 2, THEN the placement is rejected. | BLOCKING |
| BL-5b | GIVEN Player B has 0 fakes destroyed, WHEN they attempt Minion placement at cell 7, THEN the placement is rejected. | BLOCKING |
| BL-6 | GIVEN Player A has 1 fake destroyed, WHEN they place a Minion at cell 2, THEN the placement is accepted. | BLOCKING |
| BL-6b | GIVEN Player B has 1 fake destroyed, WHEN they place a Minion at cell 7, THEN the placement is accepted. | BLOCKING |
| BL-7 | GIVEN Player A has 0 fakes destroyed, WHEN they place a Structure at cell 3, THEN the placement is accepted (Structures bypass spawn range). | BLOCKING |
| BL-8 | GIVEN Player A already has a Minion in lane 3, WHEN they submit another Minion to lane 3, THEN the second placement is rejected and mana is not deducted. | BLOCKING |
| BL-9 | GIVEN Player A has 10 gold AND has a Trap at (lane 2, cell 3), WHEN they attempt a second Trap at the same cell, THEN the placement is rejected AND Player A's gold remains 10. | BLOCKING |
| BL-10 | GIVEN Player A's unit is at cell 8 at end of sub-step 6, WHEN sub-step 6 completes, THEN the Board emits `UnitAtObjective(unit_id, lane)` exactly once for that unit. | BLOCKING |
| BL-11 | GIVEN Player A's unit at cell 8 survives round N, WHEN round N+1 sub-step 6 fires, THEN the unit is still at cell 8 and attacks the objective again. | BLOCKING |
| BL-12 | GIVEN Player A's WALL unit ends sub-step 5 at cell 1 (Player A's prism cell), WHEN the prism check runs, THEN `PrismCollected(Player A, lane)` fires and the prism token is removed. | BLOCKING |
| BL-13 | GIVEN Player B's unit reaches cell 1, WHEN the prism check runs, THEN no `PrismCollected` fires for Player B — Player B's prism is at cell 8, not cell 1. | BLOCKING |
| BL-14 | GIVEN Player A submitted unit X to (lane 1, cell 1) and Player B submitted unit Y to (lane 5, cell 8) in the pending buffer, WHEN sub-step 1 commits the buffer, THEN `get_units_at_cell(lane 1, cell 1)` returns unit X AND `get_units_at_cell(lane 5, cell 8)` returns unit Y — both visible after the same commit, with neither visible in the world state before sub-step 1 fires. | BLOCKING |
| BL-15 | GIVEN REPEL 3 targets Player A's unit at cell 2, WHEN REPEL fires, THEN unit moves to cell 1 (clamped at spawn — does not go to cell −1). | BLOCKING |
| BL-16 | GIVEN a unit enters a cell containing an enemy Trap via standard movement, WHEN it arrives, THEN the Trap triggers AND the Trap entity is removed from the board. | BLOCKING |
| BL-17 | GIVEN a unit is REPEL'd into a cell containing an enemy Trap, WHEN it arrives, THEN the Trap triggers AND the Trap entity is removed from the board (displacement counts as enemy entry). | BLOCKING |
| BL-18 | GIVEN a unit is TELEPORT'd to its own spawn cell (cell 1 for Player A), THEN no prism is collected — TELEPORT is not sub-step 5 standard movement. | BLOCKING |
| BL-19 | GIVEN Strich is in lane 5 and an enemy unit enters lane 5, WHEN Strich's auto-switch fires, THEN Strich stays in lane 5 — no valid lane to switch to. | BLOCKING |
| BL-20 | GIVEN Strich's adjacent lanes are both occupied by the player's own Minions, WHEN Strich's auto-switch fires, THEN Strich stays in its current lane. | BLOCKING |
| BL-21 | GIVEN a game initialises, WHEN the board is set up, THEN each player has exactly 5 Minion slots (one per lane), all initially empty. | BLOCKING |
| BL-22 | GIVEN a unit entity in lane 1 with `ChargeBonus(2)` and `MovementPoints(3)` components at cell 1 (set via direct ECS World state, not card pool lookup), WHEN resolution runs, THEN unit ends sub-step 2 at cell 3 AND ends sub-step 5 at cell 6. F1 is applied independently at each sub-step using the unit's current cell as input. | BLOCKING |
| BL-23 | GIVEN an IRREMOVABLE unit is targeted by REPEL, WHEN REPEL fires, THEN the unit does not move and no error is raised. | BLOCKING |
| BL-24 | GIVEN Player B's unit (the caster) is at cell 8, and ATTRACT 5 targets Player A's unit at cell 5, WHEN ATTRACT fires, THEN Player A's unit moves to cell 7 — 1 cell short of the caster's cell (Rule 9a, enemy target): `effective_pull = min(5, max(0, \|8−5\|−1)) = min(5, 2) = 2`; `attract_destination = 5 + sign(8−5) × 2 = 7`. | BLOCKING |
| BL-25 | GIVEN Player B's unit is at cell 1 at end of sub-step 6, WHEN sub-step 6 completes, THEN the Board emits `UnitAtObjective(unit_id, lane)` for Player B's unit. | BLOCKING |
| BL-26 | GIVEN a fake objective is destroyed during RESOLUTION sub-step 6, WHEN Board/Lane consumes the fake-destruction fact, THEN `SpawnRangeState` expands the attacker's range by 1 cell, the `S2CResolutionEvent` contains `SpawnRangeChanged` after the corresponding `ObjectiveDestroyed`, and the expanded range applies to the next PLACEMENT phase — not the current round's already-committed placements. | BLOCKING |
| BL-27 | GIVEN an enemy Trap is at cell 3 and Player A's unit is at cell 1 with MP=3, WHEN sub-step 5 fires, THEN the unit moves to cell 4 and the Trap at cell 3 does NOT trigger. | BLOCKING |
| BL-27b | GIVEN an enemy Trap is at (lane 1, cell 2) and Player A's unit is in lane 1 at cell 1 with CHARGE 3, WHEN sub-step 2 fires, THEN the unit moves to cell 4 and the Trap at cell 2 does NOT trigger (CHARGE X movement skips intermediate cells, same as standard movement). | BLOCKING |
| BL-28 | GIVEN any unit is in lane 1, WHEN CHANGE LANE leftward fires, THEN unit stays in lane 1 — no error raised. | BLOCKING |
| BL-29 | GIVEN Player A already has a Field active in lane 2, WHEN Player A submits a second Field to lane 2, THEN the placement is rejected and mana is not deducted. | BLOCKING |
| BL-30 | GIVEN Player A's unit has `ChargeBonus(2)` and `MovementPoints(2)` and is at cell 1 (the prism cell) at the start of sub-step 2 in lane 3, WHEN sub-steps 2 and 5 both fire, THEN no `PrismCollected` event is emitted — the unit ends at cell 5 (1+2+2), not at the prism cell. Prism collection requires ending sub-step 5 at the prism cell. | BLOCKING |
| BL-31 | GIVEN Player A has a Trap at (lane 2, cell 3), Player B's unit X is in lane 1 at cell 3 and Player B's unit Y is in lane 3 at cell 3, and both CHANGE LANE to lane 2 in the same inter-sub-step pass (both arrive at cell 3), THEN the Trap triggers exactly once — triggered by unit X (lower original lane = 1 wins the tiebreak). The Trap is removed. Unit Y enters lane 2 at cell 3 and is not affected. | BLOCKING |
| BL-32 | GIVEN Player A has a Field active in lane 2, WHEN Player B submits a Field to lane 2, THEN Player B's placement is accepted AND both Fields are present in lane 2 occupancy state simultaneously (each player may have one Field per lane independently). | BLOCKING |
| BL-33 | GIVEN a 2v2 game and Team A's Player 1 already has a Minion in lane 1, WHEN Team A's Player 2 submits a Minion to lane 1, THEN the placement is accepted (team has used 1 of 2 allowed slots). WHEN Team A's Player 2 also submits their own second Minion to lane 1 (personal slot already occupied), THEN the second placement is rejected and mana is not deducted. | BLOCKING |
| BL-34 | GIVEN Player A's unit (the caster) is at cell 3, and ATTRACT 6 targets an enemy (Player B) unit at cell 7, WHEN ATTRACT fires, THEN the enemy unit moves to cell 4 — 1 cell short of the caster's cell (Rule 9a, 1-cell-apart collision rule): `effective_pull = min(6, max(0, \|3−7\|−1)) = min(6, 3) = 3`; `attract_destination = 7 + sign(3−7) × 3 = 4`. | BLOCKING |
| BL-35 | GIVEN a player submits `C2SSubmitPlacement`, WHEN the server validates the batch, THEN sender identity, phase, hand ownership, duplicate card IDs, target legality, spawn range, occupancy rules, and explicit current/reserve mana budgets are all checked before any pending placement is written. Any failure silently discards the entire batch. | BLOCKING |
| BL-36 | GIVEN a placement batch passes all validation, WHEN it is accepted and later committed, THEN pending data preserves each card's explicit current/reserve split; economy deduction occurs at PLACEMENT close before `S2CPlacementReveal` and before any ECS unit entity spawn. | BLOCKING |

## Open Questions

1. **RESOLVED — Prism count and respawn model**: Count is confirmed at 10 total (2 per lane, 1 per player at own spawn cell). Respawn model is confirmed as per-player independent: each player's 5 prisms respawn when that player has collected all 5 of their own (Rule 11). Master GDD §3.4 was updated to match during previous session; master GDD AC P5 was updated to match during this design review. No action required before Prism System GDD.

2. **TELEPORT accessible range**: The TELEPORT keyword repositions a unit to "a specified cell within the caster's accessible range." This GDD recommends the range is defined per-card in card effect text rather than as a global board rule. Confirm before Keyword System GDD is authored.

3. **Multiple Structures per cell**: This GDD defines max 1 Structure per cell per player (consistent with Trap rule). This is not explicitly stated in the master GDD. Confirm before Structure card designs are authored.

4. **Partial vs all-or-nothing placement submission**: This GDD defines partial submissions as committing on timer expiry. Confirm whether placement is a single atomic "submit all" message or streaming per-card submissions — this choice belongs in the Network Protocol GDD.

5. **Per-player vs per-team spawn range in 2v2**: This GDD recommends per-player spawn range (each player on a team tracks their own `fakes_destroyed` counter independently). Confirm before Game Session System GDD is authored.
