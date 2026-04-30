# Game Config

> **Status**: In Design
> **Author**: User + Agents
> **Last Updated**: 2026-04-28
> **Implements Pillar**: Simple surface — all tuning knobs externalized from code

## Overview

`GameConfig` is the single authoritative source for all balance-tunable values in Lanes and Lies. At server startup, `assets/config/game_config.ron` is deserialized into a `GameConfig` struct and inserted as a Bevy `Resource`. Every system that uses a tuning knob — pool copy counts, shop weights, economy constants, auction timers, combat parameters — reads its values from this resource rather than using hardcoded literals. No game logic resides in `GameConfig`; it is a read-only data container. Changing a balance value requires editing only the RON file, not touching compiled code.

## Player Fantasy

`GameConfig` is infrastructure with no direct player-facing behavior. The player experiences it indirectly: when the shop feels like it "knows what they're building," when gold income feels satisfying without being exploitable, or when auction stakes feel real — those feelings emerge from values defined in this file. Designers and developers are the direct users of this system during balance iteration.

## Detailed Rules

### Core Rules

**1. File location and format**
- Config file: `assets/config/game_config.ron` (RON — Rusty Object Notation)
- The file is bundled with the server binary. The client does not load it.
- All fields use `#[serde(default)]` — missing fields fall back to the struct's `Default` impl, which encodes the design-intent defaults from Section G (Tuning Knobs).

**2. Rust type**
```rust
#[derive(Asset, Resource, Deserialize, Reflect, Default, Clone)]
#[reflect(Resource)]
pub struct GameConfig {
    // Pool (Card Data & Pool)
    // Note: Epic and Legendary copy counts are hardcoded as Rust consts in the pool system
    // (EPIC_POOL_COPIES = 1, LEGENDARY_POOL_COPIES = 1) — scarcity is a load-bearing design
    // pillar, not a tuning knob. They are NOT fields in this struct.
    pub common_pool_copies: u32,
    pub uncommon_pool_copies: u32,
    pub rare_pool_copies: u32,
    pub shop_weight_per_card: f32,
    pub shop_weight_cap: f32,

    // Economy
    pub starting_gold: u32,
    pub gold_baseline_per_round: u32,
    pub interest_threshold_gold: u32,
    pub interest_max_bonus: u32,
    pub objective_gold_reward: u32,
    pub kill_gold_reward: u32,
    pub mana_cap: u32,
    pub refresh_base_cost: u32,
    pub refresh_cap: u32,

    // Objectives / Spawn
    pub objective_hp: u32,
    pub fake_count: u32,
    pub fake_objective_spawn_advance: u32,

    // Timers — RSM phase durations
    pub draft_initial_timer_seconds: u32,
    pub draft_shop_timer_seconds: u32,
    pub placement_timer_seconds: u32,
    pub resolution_max_duration_seconds: u32,
    pub disconnect_grace_seconds: u32,
    pub lobby_timeout_seconds: u32,
    pub lobby_heartbeat_timeout_seconds: u32,

    // Timers — Auction System
    pub auction_timer_seconds: u32,
    pub auction_timer_reset_seconds: u32,
    pub auction_max_duration_seconds: u32,
    // Starting bid floors — Auction System (added 2026-04-29 per auction-system.md)
    pub auction_floor_rare: u32,
    pub auction_floor_epic: u32,
    pub auction_floor_legendary: u32,
    pub legendary_pool_entry_round: u32,

    // Prism System
    pub prism_strike_damage: u32,         // default 1; safe 1–3
    pub prism_strike_mana_cost: u32,      // default 3; safe 1–5

    // Class mechanics
    pub xelor_sablier_steal: u32,

    // Network Protocol
    pub protocol_version: u32,
    pub hello_timeout_ms: u32,
    pub ack_timeout_ms: u32,
    pub heartbeat_interval_ms: u32,

    // Board Rendering — animation timings (updated R3 2026-04-30: fog fields removed; reveal-tween fields added)
    pub board_pre_anim_pause_ms: u32,              // default 400; safe 200–800
    pub board_sub_step_duration_ms: u32,           // default 600; safe 400–1000
    pub board_inter_step_pause_ms: u32,            // default 150; safe 100–300
    pub board_objective_reveal_hold_ms: u32,       // default 500; safe 300–800
    pub board_obj_reveal_anim_ms: u32,             // default 800; safe 500–1000 (R2 new — fake/real reveal VFX)
    pub board_unit_reveal_tween_ms: u32,           // default 250; safe 150–400 (R2 new — opponent placement reveal tween)
    pub board_reveal_timeout_ms: u32,              // default 2000; safe 1500–5000 (R2 new — ResolutionReveal stuck-state timeout)
    pub board_obj_id_reconnect_timeout_ms: u32,    // default 5000; safe 3000–10000 (R2 new — S2CObjectiveIdentities reconnect timeout)

    // Board Rendering — visual tuning (updated R3 2026-04-30: fog opacity removed)
    pub board_unit_reveal_start_scale: f32,        // default 0.4; safe 0.3–0.6 (R2 new — reveal tween start scale)
    pub board_cell_width: f32,                     // default 64.0; safe 48–96 (world units)
    pub board_lane_height: f32,                    // default 80.0; safe 64–112 (world units)
    pub board_hp_green_threshold: f32,             // default 0.6; safe 0.5–0.75
    pub board_hp_red_threshold: f32,               // default 0.3; safe 0.2–0.4
    pub board_co_occupancy_offset: f32,            // default 8.0; safe 4–16 (2v2 only)
    pub board_prism_spin_speed: f32,               // default 0.5; safe 0.2–1.0 (rad/s)
}
```

**3. Loading**
- Loaded via `bevy_asset_loader` during the server's `LoadingState`.
- After the loading state completes, the `GameConfig` asset is promoted to a `Res<GameConfig>`. All game systems start only after this promotion is confirmed.
- Load failure is fatal — if `game_config.ron` is absent or unparseable, the server aborts with a logged error.

**4. Access pattern**
- All systems read via `Res<GameConfig>`. No system holds mutable access after startup.
- Systems must not cache individual field values in local state — always read from `Res<GameConfig>` each frame, so hot-reload picks up changes immediately.

**5. Startup validation (dangerous values only)**
After loading (and again after each hot-reload), a validation system aborts the server if any of the following are violated:
- `shop_weight_cap > 0.0` — a cap of 0 causes division-by-zero in the shop weighting formula
- `shop_weight_cap < 1.0` — a cap of ≥ 1.0 makes the weight ceiling inert (no raw weight can exceed 1.0, so the cap becomes a no-op)
- `shop_weight_per_card < shop_weight_cap` — if per-card weight ≥ cap, the cap fires on the first copy acquired and per-acquisition scaling never operates; the archetype-weighting fantasy is nullified
- `common_pool_copies >= 1`, `uncommon_pool_copies >= 1`, `rare_pool_copies >= 1` — pools with 0 copies have no cards to distribute
- `fake_count >= 1` — `0` silently disables the bluffing mechanic, the "Lies" pillar of the game's identity
- `fake_count <= 3` — ensures `loss_threshold = 2` is reachable; at `fake_count = 4` only 1 real objective exists per player and the loss condition can never be triggered (`fake_count: 4` or `5` both produce unwinnable games)
- `objective_hp >= 1` — `0` causes u32 underflow on damage (debug: panic; release: wraps to ~4.29B, objectives become indestructible, win condition never triggers)
- `placement_timer_seconds >= 1` — `0` silently skips the PLACEMENT phase on the first tick
- `auction_timer_seconds >= 1` — `0` skips all bidding and produces undefined interaction with timer reset logic
- `auction_timer_reset_seconds < auction_timer_seconds` — a reset ≥ total timer allows a single bid to push duration above the initial value, inverting "shorter = more pressure"

All other values are trusted as authored.

**6. Debug hot-reload**
In debug builds (`#[cfg(debug_assertions)]`), a system watches for `AssetEvent::<GameConfig>::Modified`. On each change event, it: (1) deserializes the updated RON file, (2) re-runs the full validation check from Rule 5 — if validation fails, the reload is rejected and the existing resource is retained with a warning logged, (3) if validation passes, re-inserts the updated struct as `Res<GameConfig>`.

Production builds treat `game_config.ron` as immutable after startup — no watcher is registered.

**7. No game logic**
`GameConfig` is a plain data container. No methods beyond the serde/reflect derives. No computed properties. No mutable state.

---

### States and Transitions

| State | Description | Valid transitions |
|---|---|---|
| `Unloaded` | `LoadingState` in progress; `Res<GameConfig>` not yet inserted | → `Ready` (load + validation pass) / Server aborts (missing file, parse error, or validation failure) |
| `Ready` | `Res<GameConfig>` available to all systems; values read-only | → `Ready` (debug only: hot-reload re-inserts updated values after re-validation) / `Destroyed` (server shutdown) |

No partial states. `GameConfig` is either fully available or the server is not running.

---

### Interactions with Other Systems

| System | What is read from GameConfig |
|---|---|
| **Card Data & Pool** | `common/uncommon/rare_pool_copies`, `shop_weight_per_card`, `shop_weight_cap` |
| **Economy System** | `starting_gold`, `gold_baseline_per_round`, `interest_threshold_gold`, `interest_max_bonus`, `objective_gold_reward`, `kill_gold_reward`, `mana_cap`, `refresh_base_cost` |
| **Objective System** | `objective_hp`, `fake_count` |
| **Board / Lane System** | `fake_objective_spawn_advance` |
| **Auction System** | `auction_timer_seconds`, `auction_timer_reset_seconds`, `auction_max_duration_seconds`, `auction_floor_rare`, `auction_floor_epic`, `auction_floor_legendary`, `legendary_pool_entry_round` |
| **Board Rendering** | `board_pre_anim_pause_ms`, `board_sub_step_duration_ms`, `board_inter_step_pause_ms`, `board_fog_lift_ms`, `board_objective_reveal_hold_ms`, `board_fog_opacity`, `board_cell_width`, `board_lane_height`, `board_hp_green_threshold`, `board_hp_red_threshold`, `board_co_occupancy_offset`, `board_prism_spin_speed` |
| **Round State Machine** | `placement_timer_seconds`, `draft_initial_timer_seconds`, `draft_shop_timer_seconds`, `resolution_max_duration_seconds`, `auction_max_duration_seconds`, `disconnect_grace_seconds` |
| **Class System (Xelor)** | `xelor_sablier_steal` |
| **Server-side RNG** | *(none — RNG seeds are generated at runtime)* |

`GameConfig` has no upstream data dependencies. It reads from disk only.

## Formulas

`GameConfig` contains no runtime formulas. All mathematical operations are performed by the systems that read from it; the canonical formula definitions are in their respective GDDs.

**Structural invariants (validated at load time and on each hot-reload):**

| Invariant | Condition | Effect if violated |
|---|---|---|
| Shop weight cap is operational | `0.0 < shop_weight_cap < 1.0` | Server aborts (cap = 0 → division-by-zero; cap ≥ 1.0 → weight ceiling inert) |
| Shop weight scaling is operative | `shop_weight_per_card < shop_weight_cap` | Server aborts (cap fires on first copy; per-acquisition scaling never operates) |
| Common/Uncommon/Rare pool counts nonzero | `N_pool_copies >= 1` for Common, Uncommon, Rare | Server aborts (pools with 0 copies have no cards to distribute) |
| Fake count minimum | `fake_count >= 1` | Server aborts (`0` disables the bluffing mechanic — the "Lies" pillar) |
| Fake count maximum | `fake_count <= 3` | Server aborts (`fake_count = 4` leaves only 1 real objective; `loss_threshold = 2` is unreachable — game cannot end normally) |
| Objective HP nonzero | `objective_hp >= 1` | Server aborts (0 → u32 underflow on damage → debug panic or release wrap to ~4.29B) |
| PLACEMENT timer nonzero | `placement_timer_seconds >= 1` | Server aborts (0 silently skips the PLACEMENT phase) |
| AUCTION timer nonzero | `auction_timer_seconds >= 1` | Server aborts (0 skips bidding; undefined interaction with reset logic) |
| Auction reset below timer | `auction_timer_reset_seconds < auction_timer_seconds` | Server aborts (reset ≥ timer: single bid pushes duration above initial, inverting time-pressure intent) |

These invariants are preconditions that other systems rely on being true before they execute. They are not runtime formulas.

## Edge Cases

- **`game_config.ron` is missing or unparseable:** Fatal — server aborts with a logged error including the file path and, for parse errors, the location in the file. There is no fallback to defaults; a missing or broken config is a deployment error, not a runtime condition to tolerate.

- **A field is missing from the RON file (`#[serde(default)]` active):** The field silently falls back to the struct's `Default` impl value. This is intentional — it allows partial configs during development. The `Default` values must match the documented design-intent defaults from Section G exactly.

- **`shop_weight_cap = 0.0` or `>= 1.0`:** Validation failure at load time or hot-reload. Server aborts (load) or rejects the reload and retains the existing resource (hot-reload). A warning is logged identifying the offending value.

- **Any Common, Uncommon, or Rare `pool_copies = 0`:** Validation failure, abort or reject. (Epic and Legendary copy counts are hardcoded consts, not config fields — see struct note.)

- **`shop_weight_per_card >= shop_weight_cap`:** Validation failure — the cap fires on the first copy acquired, nullifying per-acquisition scaling. Abort or reject.

- **`fake_count = 5`:** Validation failure — no real objectives would exist. Abort (load) or reject (hot-reload) with logged reason.

- **`fake_count = 0`:** Validation failure — `0` disables the bluffing mechanic entirely. Abort or reject with error: *"`fake_count` must be ≥ 1 — the bluffing mechanic is a load-bearing design pillar."*

- **`objective_hp = 0`:** Validation failure — `0` causes u32 underflow on the first damage application (debug: panic; release: wraps to ~4.29B, making objectives permanently indestructible, win condition never triggers). Abort or reject.

- **`placement_timer_seconds = 0` or `auction_timer_seconds = 0`:** Validation failure — zero-duration timers skip the phase on the first server tick. Abort or reject.

- **`auction_timer_reset_seconds >= auction_timer_seconds`:** Validation failure — a reset ≥ total timer allows a single bid to push duration above the initial value. Abort or reject.

- **Out-of-range balance value not in the dangerous-only set** (e.g., `mana_cap: 100`, `starting_gold: 100`): Loaded and applied without error. The server trusts that designers know what they're setting. Extreme values may produce unintended gameplay but will not crash the server.

- **`fake_objective_spawn_advance = 0`:** Accepted. Destroying a fake objective grants no spawn expansion. This is a legal tuning choice (effectively disables the mechanic).

- **Hot-reload in a production build:** The watcher is never registered (`#[cfg(debug_assertions)]` guard). Editing `game_config.ron` on a running production server has no effect — a restart is required to apply changes.

- **Two systems read `Res<GameConfig>` in the same frame:** Safe by design — Bevy's `Res<T>` is an immutable shared reference; concurrent reads are always valid.

## Dependencies

| System | Relationship | Interface |
|---|---|---|
| **File system / Bevy asset pipeline** | Hard upstream | Reads `assets/config/game_config.ron` at startup. Fatal if absent. |
| **Card Data & Pool** | Downstream (hard) | Reads: `common/uncommon/rare_pool_copies`, `shop_weight_per_card`, `shop_weight_cap` |
| **Economy System** | Downstream (hard) | Reads: `starting_gold`, `gold_baseline_per_round`, `interest_threshold_gold`, `interest_max_bonus`, `objective_gold_reward`, `kill_gold_reward`, `mana_cap`, `refresh_base_cost` |
| **Objective System** | Downstream (hard) | Reads: `objective_hp`, `fake_count` |
| **Board / Lane System** | Downstream (hard) | Reads: `fake_objective_spawn_advance` |
| **Auction System** | Downstream (hard) | Reads: `auction_timer_seconds`, `auction_timer_reset_seconds`, `auction_max_duration_seconds`, `auction_floor_rare`, `auction_floor_epic`, `auction_floor_legendary` |
| **Round State Machine** | Downstream (hard) | Reads: `placement_timer_seconds`, `draft_initial_timer_seconds`, `draft_shop_timer_seconds`, `resolution_max_duration_seconds`, `auction_max_duration_seconds`, `disconnect_grace_seconds` |
| **Game Session System** | Downstream (hard) | Reads: `lobby_timeout_seconds` (for lobby deadline), `lobby_heartbeat_timeout_seconds` (for LOBBY heartbeat-gap detection). Note: `disconnect_grace_seconds` is RSM-owned and does NOT apply during LOBBY. |
| **Class System** | Downstream (soft) | Reads: `xelor_sablier_steal` (Xelor-specific; other classes have no dedicated config knobs at this time) |
| **Network Protocol** | Downstream (hard) | Reads: `protocol_version`, `hello_timeout_ms`, `ack_timeout_ms`, `heartbeat_interval_ms`, `disconnect_grace_seconds` |

**Bidirectionality:** `card-data-pool.md` ✓, `board-lane-system.md` ✓, `round-state-machine.md` ✓, `economy-system.md` ✓ — all list Game Config as an upstream dependency. `economy-system.md` must add `interest_threshold_gold` to its dependency table when the economy GDD is next revised. GDDs not yet authored (Objective System, Auction System, Class System) must list Game Config when written.

GameConfig has no peer dependencies. It reads from disk only and is the bottom of the dependency stack.

## Tuning Knobs

This is the authoritative list of all `GameConfig` fields and their design-intent defaults. Every value corresponds to a field in the Rust struct. Defaults are encoded in the struct's `Default` impl; missing RON fields fall back silently to these values.

| Field | Default | Safe Range | Gameplay Impact | Constraint |
|---|---|---|---|---|
| **Pool — Card Data & Pool** | | | | |
| `common_pool_copies` | 6 | 3–10 | Higher = more Common diversity and durability late-game; lower = earlier scarcity and empty slots | validated ≥ 1 |
| `uncommon_pool_copies` | 5 | 3–8 | Same as above for Uncommons | validated ≥ 1 |
| `rare_pool_copies` | 4 | 1–6 | At 1: Rares feel as scarce as Epics; at 6: Rares freely available all game | validated ≥ 1 |
| *(Epic copies)* | `EPIC_POOL_COPIES = 1` | *const* | Not tunable — Epic class-identity scarcity is load-bearing. Hardcoded in pool system. | *not in config* |
| *(Legendary copies)* | `LEGENDARY_POOL_COPIES = 1` | *const* | Not tunable — Legendary auction stakes fantasy depends on exactly 1 copy. Hardcoded in pool system. | *not in config* |
| `shop_weight_per_card` | 0.10 | 0.02–0.15 | At 15%: archetype feels scripted by round 4. At 0.02: weighting is imperceptible (effect negligible below ~0.05). | **validated: < shop_weight_cap** |
| `shop_weight_cap` | 0.65 | 0.50–0.80 | Must stay strictly between 0.0 and 1.0 (both exclusive); activates at ~7 acquired copies (at default per-card weight) | **validated: 0.0 < cap < 1.0** |
| **Economy System** | | | | |
| `starting_gold` | 5 | 3–8 | Higher = more initial draft choice; lower = more constraint and variance in opening | — |
| `gold_baseline_per_round` | 2 | 1–4 | Core economy pacing; affects interest threshold timing | — |
| `interest_threshold_gold` | 5 | 5–10 | The divisor in `floor(gold / interest_threshold_gold)`. Do not set below 5 — starting gold (5g) would immediately exceed the max-interest bracket, removing the miser/gambler tension. At 10: only meaningful at 10g+ | — |
| `interest_max_bonus` | 2 | 1–3 | Higher = stronger hoard incentive and snowball; lower = weaker reward for patience | — |
| `objective_gold_reward` | 3 | 2–5 | Higher = more snowball from first objective destruction | — |
| `kill_gold_reward` | 1 | 0–2 | 0 = remove combat gold loop entirely; 2 = stronger snowball from aggressive play | — |
| `mana_cap` | 10 | 6–14 | Higher = more cards playable per round; dramatically changes tempo ceiling | — |
| `refresh_base_cost` | 1 | 1–3 | Base gold cost of the first manual shop refresh per DRAFT phase; each additional refresh in the same phase costs +1g more | — |
| **Objective System** | | | | |
| `objective_hp` | 5 | 3–8 | Lower = faster games and fewer comebacks; higher = more durability and comeback potential | **validated: ≥ 1** |
| `fake_count` | 2 | 1–3 | More fakes = more bluff space; fewer = more direct information war | **validated: ≥ 1 and ≤ 3** |
| `fake_objective_spawn_advance` | 1 | 1–2 | Rows of spawn range unlocked per fake destroyed (used in Formula 3 of card-data-pool.md). At 2: Row 3 reachable after the first fake is destroyed. | — |
| **Timers — RSM phases** | | | | |
| `draft_initial_timer_seconds` | 45 | 30–90 | Round 1 DRAFT_INITIAL duration; early exit expected at ~25–30s when all players submit | — |
| `draft_shop_timer_seconds` | 30 | 20–60 | Per-round DRAFT_SHOP duration; early exit when all players signal ready | — |
| `placement_timer_seconds` | 10 | 5–20 | Shorter = more reflex/pressure; longer = more deliberation | **validated: ≥ 1** |
| `resolution_max_duration_seconds` | 60 | 30–120 | Safety timeout for RESOLUTION. Aborts to Draw if Combat Resolution doesn't complete. Must never fire in normal play. | — |
| `disconnect_grace_seconds` | 30 | 15–60 | Seconds before a disconnected player forfeits. 30s is intentional for WASM/browser — OS interrupts can cause 3–6s gaps. | — |
| `lobby_timeout_seconds` | 90 | 60–300 | How long the LOBBY waits for all slots to fill and all classes to be confirmed before cancelling the session. Countdown starts at room creation. Too low = false cancellations for friend groups; too high = long idle wait for abandoned lobbies. | — |
| `lobby_heartbeat_timeout_seconds` | 15 | 10–60 | Seconds without a client heartbeat before the Game Session System treats a LOBBY player as disconnected and cancels the session. Shorter than `disconnect_grace_seconds` — LOBBY has no game state to recover, so early timeout is safe. Must be ≪ `lobby_timeout_seconds`. | — |
| **Timers — Auction System** | | | | |
| `auction_timer_seconds` | 20 | 10–30 | Shorter = more time pressure and bluff risk; longer = more deliberation | **validated: ≥ 1** |
| `auction_timer_reset_seconds` | 5 | 3–10 | How much each accepted bid adds back to the timer | **validated: < auction_timer_seconds** |
| `auction_max_duration_seconds` | 120 | 60–300 | Safety timeout for DRAFT_AUCTION. Must be ≥ `auction_timer_seconds + (20 × auction_timer_reset_seconds)` to avoid cutting off a legitimate bidding war. | — |
| `auction_floor_rare` | 3 | 2–5 | Starting bid for Rare-rarity auction cards. Must stay above Uncommon shop cost (2g) to preserve rarity signal. | — |
| `auction_floor_epic` | 4 | 3–6 | Starting bid for neutral Epic-rarity auction cards. Requires original neutral Epic card designs (auction-system.md OQ1). | — |
| `auction_floor_legendary` | 5 | 4–8 | Starting bid for Legendary-rarity auction cards. Too low = Legendary feels too accessible early; too high = gates cashflow-poor players. | — |
| `legendary_pool_entry_round` | 6 | 3–9 | Earliest round at which Legendary cards become eligible for `draw_auction_card()`. Default 6 = second auction. Below 3: Legendaries appear when most players have 5–10g, making the 6g minimum bid uncontestable. Above 9: Legendaries may never appear in short games. | — |
| **Class System** | | | | |
| `xelor_sablier_steal` | 1 | 1–3 | Mana stolen from opponent's current pool per Sablier cast. Effective steal = `min(steal, opponent.current_mana)`. See Class System GDD for 0-mana behavior specification. | — |
| **Network Protocol** | | | | |
| `protocol_version` | 1 | N/A | Wire protocol version; must match client and server exactly. Any mismatch → `S2CHandshakeRejected`. Increment on any breaking wire change (new message type, removed message, field type change). | compatibility gate, not balance |
| `hello_timeout_ms` | 5000 | 2000–15000 | Milliseconds server waits for `C2SHello` after transport connect before closing. Too low: legitimate WASM cold-start clients kicked. Too high: slow DoS detection. | — |
| `ack_timeout_ms` | 10000 | 5000–30000 | Milliseconds server waits for `C2SAcknowledgeResult` after GAME_OVER before cleaning up the session. Result persisted regardless. | — |
| `heartbeat_interval_ms` | 5000 | 2000–15000 | Target interval at which the client sends `C2SHeartbeat`. Server uses heartbeat absence (plus `disconnect_grace_seconds`) to detect half-open WASM/WebSocket connections. Must be ≪ `disconnect_grace_seconds × 1000`. | — |
| **Board Rendering** (added 2026-04-30 per board-rendering.md /design-review revision) | | | | |
| `board_pre_anim_pause_ms` | 400 | 200–800 | Hold after fog lift before sub-step 1 animation begins. Too low: players can't read simultaneous reveal before action. Too high: dead time. | — |
| `board_sub_step_duration_ms` | 600 | 400–1000 | Active animation window per sub-step group. Was 800ms before 2026-04-30 revision; tightened to defend the ≤5s default match watch budget per Player Fantasy. Too low: sub-steps blur. Too high: watching becomes idle dead time. | — |
| `board_inter_step_pause_ms` | 150 | 100–300 | Silent pause between consecutive sub-step groups. Was 200ms before 2026-04-30. Too low: rushed; too high: stalls. | — |
| `board_fog_lift_ms` | 350 | 200–600 | Duration of the fog alpha fade-out tween at `S2CPlacementReveal`. Concurrent with `board_pre_anim_pause_ms`. Too low: reveal feels abrupt. Too high: sluggish. | — |
| `board_objective_reveal_hold_ms` | 500 | 300–800 | Hold time on objective entity before destruction VFX fires (suspense beat). Too low: instant; too high: padded. | — |
| `board_fog_opacity` | 0.6 | 0.4–0.8 | Opponent-half fog sprite alpha during PLACEMENT. Too low: opponent half partially readable; too high: harsh. Validated/clamped at intake (BR-FOG-OPACITY). | — |
| `board_cell_width` | 64.0 | 48–96 | Cell width in world units. At min: cramped read; at max: board may exceed viewport (camera spec OQ-BR-02 pending). | — |
| `board_lane_height` | 80.0 | 64–112 | Lane height in world units. Same constraints as `board_cell_width`. | — |
| `board_hp_green_threshold` | 0.6 | 0.5–0.75 | Fill fraction at or above which HP bar is green. Below: yellow (until red threshold). | — |
| `board_hp_red_threshold` | 0.3 | 0.2–0.4 | Fill fraction below which HP bar is red. Must be < `board_hp_green_threshold`. | — |
| `board_co_occupancy_offset` | 8.0 | 4–16 | World-unit X offset per unit from cell center in 2v2 co-occupancy. Below 4 with 48px sprites: visible overlap. Above 16: clips outside cell node. | — |
| `board_prism_spin_speed` | 0.5 | 0.2–1.0 | Prism rotation rate (rad/s). Too low: looks static; too high: distracting. | — |

## Visual/Audio Requirements

None. `GameConfig` is a server-side data resource with no visual or audio output.

## UI Requirements

None. `GameConfig` is not exposed in any player-facing UI. The values it holds affect what other systems display, but those displays are owned by the consuming systems, not GameConfig itself.

## Acceptance Criteria

### Config Loading

| # | Criterion | Type |
|---|---|---|
| GC1 | **GIVEN** a `game_config.ron` where every field is explicitly set to a non-default value (fixture: `common_pool_copies: 7`, `uncommon_pool_copies: 6`, `rare_pool_copies: 5`, `shop_weight_per_card: 0.08`, `shop_weight_cap: 0.70`, `starting_gold: 6`, `gold_baseline_per_round: 3`, `interest_threshold_gold: 6`, `interest_max_bonus: 3`, `objective_gold_reward: 4`, `kill_gold_reward: 2`, `mana_cap: 12`, `refresh_base_cost: 2`, `objective_hp: 6`, `fake_count: 3`, `fake_objective_spawn_advance: 2`, `draft_initial_timer_seconds: 60`, `draft_shop_timer_seconds: 25`, `placement_timer_seconds: 15`, `resolution_max_duration_seconds: 90`, `auction_max_duration_seconds: 180`, `disconnect_grace_seconds: 45`, `auction_timer_seconds: 25`, `auction_timer_reset_seconds: 8`, `xelor_sablier_steal: 2`), **WHEN** `load_game_config()` is called, **THEN** it returns `Ok(config)` where every field equals the fixture value (verified field-by-field). | BLOCKING |
| GC2a | **GIVEN** `load_game_config()` is called with a path that does not exist on the file system, **WHEN** the function returns, **THEN** it returns `Err(e)` where `e.to_string()` contains the literal string `"assets/config/game_config.ron"`. | BLOCKING |
| GC2b | **GIVEN** a Bevy `App` configured with the server startup plugin and no `game_config.ron` at `assets/config/game_config.ron`, **WHEN** the loading state runs, **THEN** the `App` does not advance to the `InGame` state and `Res<GameConfig>` is not present in the `World`. | BLOCKING (Integration) |
| GC3 | **GIVEN** a file at the expected config path containing deliberately malformed RON (e.g., `GameConfig( mana_cap: `), **WHEN** `load_game_config()` is called, **THEN** it returns `Err(e)` where `e.to_string()` contains `"assets/config/game_config.ron"` AND contains content beyond the path alone (a line/column position, or a non-empty parse error description). | BLOCKING |
| GC4 | **GIVEN** a `game_config.ron` that omits the `mana_cap` field (all other fields valid), **WHEN** `load_game_config()` is called, **THEN** it returns `Ok(config)` where `config.mana_cap == 10` (the design-intent default from the Tuning Knobs table). | BLOCKING |
| GCN-DEFAULTS | **GIVEN** `GameConfig::default()` is constructed, **THEN** every field equals the Tuning Knobs table value: `common_pool_copies == 6`, `uncommon_pool_copies == 5`, `rare_pool_copies == 4`, `shop_weight_per_card == 0.10`, `shop_weight_cap == 0.65`, `starting_gold == 5`, `gold_baseline_per_round == 2`, `interest_threshold_gold == 5`, `interest_max_bonus == 2`, `objective_gold_reward == 3`, `kill_gold_reward == 1`, `mana_cap == 10`, `refresh_base_cost == 1`, `objective_hp == 5`, `fake_count == 2`, `fake_objective_spawn_advance == 1`, `draft_initial_timer_seconds == 45`, `draft_shop_timer_seconds == 30`, `placement_timer_seconds == 10`, `resolution_max_duration_seconds == 60`, `auction_max_duration_seconds == 120`, `disconnect_grace_seconds == 30`, `auction_timer_seconds == 20`, `auction_timer_reset_seconds == 5`, `xelor_sablier_steal == 1`, `protocol_version == 1`, `hello_timeout_ms == 5000`, `ack_timeout_ms == 10000`, `heartbeat_interval_ms == 5000`, `lobby_heartbeat_timeout_seconds == 15`. | BLOCKING |

### Validation

| # | Criterion | Type |
|---|---|---|
| GC5 | **GIVEN** `shop_weight_cap = 0.0`, **WHEN** `validate_game_config()` is called, **THEN** it returns `Err`. | BLOCKING |
| GC6 | **GIVEN** `shop_weight_cap = 1.0`, **WHEN** `validate_game_config()` is called, **THEN** it returns `Err`. | BLOCKING |
| GC6b | **GIVEN** `shop_weight_cap = -0.1f32`, **WHEN** `validate_game_config()` is called, **THEN** it returns `Err`. | BLOCKING |
| GC6c | **GIVEN** `shop_weight_cap = 0.5` and `shop_weight_per_card = 0.10` (per-card weight below cap), **WHEN** `validate_game_config()` is called, **THEN** it returns `Ok(())`. | BLOCKING |
| GC6d | **GIVEN** `shop_weight_per_card = 0.15` and `shop_weight_cap = 0.10` (per-card weight ≥ cap), **WHEN** `validate_game_config()` is called, **THEN** it returns `Err`. | BLOCKING |
| GC7 | *(Three independent test cases, one per validated rarity)* **GIVEN** a `GameConfig` with exactly one of `common_pool_copies`, `uncommon_pool_copies`, or `rare_pool_copies` set to `0` (each tested separately, all other fields valid), **WHEN** `validate_game_config()` is called, **THEN** it returns `Err` in each of the three cases. | BLOCKING |
| GC8 | **GIVEN** `fake_count = 5`, **WHEN** `validate_game_config()` is called, **THEN** it returns `Err`. | BLOCKING |
| GC8c | **GIVEN** `fake_count = 4`, **WHEN** `validate_game_config()` is called, **THEN** it returns `Err` (only 1 real objective would exist; `loss_threshold = 2` is unreachable). | BLOCKING |
| GC8b | **GIVEN** `fake_count = 0`, **WHEN** `validate_game_config()` is called, **THEN** it returns `Err` and `e.to_string()` contains `"fake_count"`. | BLOCKING |
| GC9 | **GIVEN** a `GameConfig` with `mana_cap = 100` and all other fields valid, **WHEN** `validate_game_config()` is called, **THEN** it returns `Ok(())`. Additionally, **GIVEN** `load_game_config()` with a fixture containing `mana_cap: 100`, **THEN** it returns `Ok(config)` where `config.mana_cap == 100`. | BLOCKING |
| GC9b | **GIVEN** `objective_hp = 0`, **WHEN** `validate_game_config()` is called, **THEN** it returns `Err`. | BLOCKING |
| GC9c | **GIVEN** `placement_timer_seconds = 0`, **WHEN** `validate_game_config()` is called, **THEN** it returns `Err`. | BLOCKING |
| GC9d | **GIVEN** `auction_timer_seconds = 0`, **WHEN** `validate_game_config()` is called, **THEN** it returns `Err`. | BLOCKING |
| GC9e | **GIVEN** `auction_timer_reset_seconds = 20` and `auction_timer_seconds = 20` (reset equals total timer), **WHEN** `validate_game_config()` is called, **THEN** it returns `Err`. | BLOCKING |
| GC9f | **GIVEN** `auction_timer_reset_seconds = 5` and `auction_timer_seconds = 20` (reset less than total timer), **WHEN** `validate_game_config()` is called, **THEN** it returns `Ok(())`. | BLOCKING |

### Hot-Reload (Integration — require running Bevy `App`)

| # | Criterion | Type |
|---|---|---|
| GC10 | **GIVEN** a Bevy `App` running in debug mode with `GameConfig` loaded from a fixture containing `placement_timer_seconds: 10`, **WHEN** the fixture file is overwritten with `placement_timer_seconds: 15` and the `App` is updated until `AssetEvent::<GameConfig>::Modified` is processed, **THEN** querying `Res<GameConfig>` from the `World` returns a value where `placement_timer_seconds == 15` (before the update it was `10`). Test evidence: `tests/integration/game_config/hot_reload_valid_test.rs` | BLOCKING (Integration) |
| GC11 | **GIVEN** a Bevy `App` running in debug mode with `GameConfig` loaded from a valid fixture containing `placement_timer_seconds: 10`, **WHEN** the fixture file is overwritten with an invalid config (`shop_weight_cap: 0.0`) and the `App` is updated until the asset event is processed, **THEN** (a) `Res<GameConfig>.placement_timer_seconds == 10` (original value unchanged), AND (b) the test log capture buffer contains at least one WARN-level entry. Note: requires `tracing-test` crate or equivalent. Test evidence: `tests/integration/game_config/hot_reload_invalid_test.rs` | BLOCKING (Integration) |
| GC12 | **GIVEN** a Bevy `App` compiled without `debug_assertions` (release build) with `GameConfig` loaded from a fixture containing `placement_timer_seconds: 10`, **WHEN** the fixture file is overwritten with `placement_timer_seconds: 15` and the `App` is updated for 2 full ticks, **THEN** `Res<GameConfig>.placement_timer_seconds == 10` (no reload occurred). Test evidence: `tests/integration/game_config/no_hot_reload_release_test.rs` | BLOCKING (Integration) |

## Open Questions

| # | Question | Owner | Priority |
|---|---|---|---|
| OQ1 | Which version of `bevy_asset_loader` is compatible with Bevy 0.18? Verify on crates.io — assume ~0.22 until confirmed. | Engine Programmer | Before sprint start |
| OQ2 | Does `bevy_asset_loader` for Bevy 0.18 handle `TypePath` internally for custom asset types, or does `GameConfig`'s loader struct require an explicit `#[derive(TypePath)]`? | Engine Programmer | Before sprint start |
| OQ3 | `ron` is no longer re-exported from `bevy_asset` in Bevy 0.18 — add `ron = "0.8"` as a direct dependency in `Cargo.toml`. Confirm exact version against Bevy 0.18 compatibility before implementation. | Lead Programmer | Before sprint start |
