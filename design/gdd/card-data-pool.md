# Card Data & Pool

> **Status**: Approved (post-revision)
> **Author**: User + Agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: Krosmaga Foundation — adapts existing cards, doesn't reinvent

## Overview

The Card Data & Pool system is the single source of truth for all card definitions and the authoritative manager of which card copies are currently available for distribution. It loads the complete card catalog (~298 cards, Krosmaga Extension=1) from `assets/data/cards.json` at game start, constructs the runtime pool with the correct number of copies per card, and exposes read-only query interfaces that every other system uses to ask questions like "give me 3 random cards weighted by this player's archetype" or "is there a Legendary copy still available for this auction round?" No game logic lives here — the Card Data & Pool system is pure data management.

## Player Fantasy

The shop is not a slot machine — it is a world that watches. As a player commits to an archetype, the pool leans toward them, rewarding conviction with coherence. *The game seems to know what they're building.* And every card carries weight: the pool is finite and personal — each player's copy counts are theirs alone, unaffected by what others buy. Within your pool, a Legendary card exists only once. When one surfaces in the auction, every player understands the weight of it — if you lose this bid, that copy will never appear in your auction again. The player does not think about probability tables or copy counts. They feel a world that responds to their choices, and stakes that cannot be undone.

## Detailed Rules

### Core Rules

**1. Card definition loading**
- At server startup, `assets/data/cards.json` is loaded and parsed into a `CardCatalog` — an immutable array of all card definitions.
- The client also loads `cards.json` independently at startup for display lookup. Client and server both hold the full catalog; only the server performs pool state mutations.
- Load failure is fatal — the server aborts startup. There is no graceful degradation.
- The catalog contains two categories of cards:
  - **Extension=1 sourced**: ~315 Minion and Spell cards from Krosmaga Extension=1 (after excluding Extension=518 cards)
  - **Original designs**: Trap, Structure, Field, Order, and Double-Face card types are original cards designed for this game; they do not exist in the Krosmaga Extension=1 set
- **Hard validation at load time** — any of the following aborts startup with a logged error (server does not start):
  - Duplicate `id` values in `cards.json` (all conflicting IDs logged)
  - Missing or unrecognized `rarity` field on any card (card ID logged)
  - `SHOP_WEIGHT_CAP ≤ 0` in loaded `GameConfig` (invalid value logged)
  - Note: `pool_copies_override ≤ 0` is a **soft** error — affected card receives its rarity default and the server continues (see Edge Cases)

**2. Per-player pool construction**
- When a game session starts, each player receives their own `PlayerPool` initialized from the `CardCatalog`.
- The pool is **per-player and independent** — other players drawing cards from their pool does not affect yours. This game does NOT use a shared global pool (unlike TFT).
- Pool copy counts per rarity: **Common=6, Uncommon=5, Rare=4, Epic=1, Legendary=1**.
- A card definition may include an optional `pool_copies_override` field that takes precedence over the rarity default for that specific card.

**3. Card definition schema**

Each card in `cards.json` has a base set of fields plus type-specific fields:

**Base fields (all card types):**

| Field | Type | Values | Notes |
|---|---|---|---|
| `id` | string | unique slug | Primary key; stable across patches |
| `name_fr` / `name_en` | string | display text | Canonical names |
| `card_type` | enum | `Minion`, `Spell`, `Trap`, `Structure`, `Field`, `Order`, `DoubleFace` | Field = formerly "Passive/Aura"; DoubleFace requires second-face fields (schema TBD) |
| `class` | enum | `Iop`, `Cra`, `Sacrier`, `Xelor`, `Ecaflip`, `Sadida`, `Neutral` | |
| `rarity` | enum | `Common`, `Uncommon`, `Rare`, `Epic`, `Legendary` | |
| `unit_type` | enum | `Blade`, `Arcane`, `Shield`, `Neutral` | Default `Neutral` for all cards |
| `family` | string or null | e.g., `"Gobball"` | Neutral families only; null for class cards |
| `keywords` | array | see keyword schema | Empty array if none |
| `effect_text` | string | human-readable description | For trigger cards and effects |
| `art_id` | string | sprite atlas key | |
| `pool_copies_override` | int or null | overrides rarity default | Optional |

**Minion-only fields:**

| Field | Type | Range | Notes |
|---|---|---|---|
| `cost` | int | 0–20 | Mana cost |
| `atk` | int | 0–20 | Base ATK |
| `hp` | int | 1–20 | Base HP |
| `mp` | int | 0–6 | Movement points; 0 = stationary (WALL) |
| `ar` | int | 0–10 | Base armor |

**Spell, Trap, Field, Order fields:**

| Field | Type | Range | Notes |
|---|---|---|---|
| `cost` | int | 0–20 | Mana cost to play |

**Structure fields:** Same as Minion but `atk` = 0 and `mp` = 0 always.

**Field card:** No additional fields beyond base. `cost` field present. Effect is defined in `effect_text`. Max 1 Field card per lane per player at any time.

**Keyword schema:**

No-parameter keywords are plain strings; parameterized keywords are objects:

```json
"keywords": [
  "FirstStrike",
  "Charge",
  { "kw": "RangeX",      "max_range": 2 },
  { "kw": "ChargeXMove", "cells": 1 },
  { "kw": "ResistanceX", "value": 1 },
  "AppearanceTrigger",
  "DeathTrigger"
]
```

Trigger keywords (`AppearanceTrigger`, `DeathTrigger`, `FinalBlowTrigger`, `CounterattackTrigger`, `StartOfTurnTrigger`, `EndOfTurnTrigger`) are flags — the card's `effect_text` describes what happens; the effect logic is code-side, keyed to the card's `id`.

**4. Pool queries**

**`SlotType` enum** (used by shop draw functions — determines which weighted algorithm runs):

```rust
enum SlotType { Class, Neutral }
```

Phase 1 of shop slot generation (the 50/50 split roll that determines SlotType) is the calling system's responsibility — Card Acquisition calls `next_seed()` from ServerRng and uses `gen_range(0..2)` to produce a SlotType. The pool never performs this roll.

| Operation | Input | Output | Notes |
|---|---|---|---|
| `is_available(card_id)` | CardId | bool | O(1) fast check |
| `copies_remaining(card_id)` | CardId | u32 | For UI display |
| `draw_initial_draft(class, count, seed)` | Class, u8, u64 | Vec\<CardId\> | 9 distinct cards; fully random from eligible class + neutral catalog cards (any rarity); no rarity floor guaranteed — see Draft Algorithm note below |
| `draw_class_card(class, seed)` | Class, u64 | Option\<CardId\> | **Phase 2 for class slots.** Weighted pick over the player's eligible class card_ids (Formula 2). Pool computes `total_acquired` internally from `initial_count − copies_remaining`. Returns None only if all class cards are exhausted. |
| `draw_neutral_family(seed)` | u64 | Option\<FamilyId\> | **Phase 2 for neutral slots.** Weighted pick over eligible neutral families (Formula 2). Returns None only if all neutral families are fully exhausted. |
| `draw_family_card(family, seed)` | FamilyId, u64 | Option\<CardId\> | **Phase 3 for neutral slots.** Uniform pick among available cards in the selected family (`copies_remaining > 0`). Returns None if the family is fully exhausted (should not occur when draw_neutral_family returned Some). |
| `draw_auction_card(seed)` | u64 | Option\<CardId\> | Draws from the **shared neutral auction pool** — a game-level pool of Neutral Rare and Legendary cards shared across all players. The auction offers one card per auction round; all players bid on the same card simultaneously. Epic cards are excluded (class-specific, no Neutral Epics). This pool is **separate from every player's personal shop pool** — there is no copy collision between shop and auction draws. Full shared pool management to be specified in Auction System GDD. Returns None if the shared auction pool is exhausted. |
| `draw_random(filter, seed)` | PoolFilter, u64 | Option\<CardId\> | Uniform pick over distinct eligible cards matching the filter (not copy-weighted). Used for draw effects and prism Lane 3. Caller is responsible for calling `distribute()` after receiving a non-None result if the effect consumes the card. |
| `distribute(card_id)` | CardId | `Result<(), DistributeError>` | Only mutation — decrements `copies_remaining` by 1. Returns `Err(DistributeError::Exhausted)` if `copies_remaining == 0`. Callers should check `is_available()` before calling. |

The pool never holds a random source — callers supply explicit seeds from the server-side RNG system. All randomness is server-seeded.

**Shop draw flow for Card Acquisition (per slot, aligns with server-rng.md seed table):**
1. Card Acquisition calls `next_seed()` → `gen_range(0..2)` → SlotType (Phase 1 split roll)
2. **Class slot:** `draw_class_card(class, next_seed())` → `Option<CardId>` (Phase 2)
3. **Neutral slot:** `draw_neutral_family(next_seed())` → `Option<FamilyId>` (Phase 2); if `Some(family)`, `draw_family_card(family, next_seed())` → `Option<CardId>` (Phase 3)
4. **Fallback (class exhausted):** if `draw_class_card` returns None, Card Acquisition retries as neutral with new seeds. This fallback is Card Acquisition's responsibility — the pool does not perform it internally.
5. **Both exhausted:** slot returns None (empty slot in UI).

**Shop refresh policy:** A shop refresh (auto at DRAFT phase start, or manual for escalating cost per Economy System) is N fresh calls to the draw functions above with new seeds from ServerRng. The pool does not track which cards were displayed this round — it only tracks copies distributed (purchased). Un-purchased displayed cards have unchanged `copies_remaining` and may reappear on refresh. Any within-round deduplication (preventing re-display of the same card in the same DRAFT phase) is Card Acquisition's responsibility, not the pool's.

**Draft Algorithm:** `draw_initial_draft` draws 9 distinct card IDs uniformly at random from the union of the player's class cards and all Neutral cards. Any rarity is eligible — including Epic and Legendary. Cards are drawn without replacement (no duplicates within the 9). The player keeps any subset they can afford within the 5g starting budget; unselected cards are discarded without decrementing the pool. Only purchased cards trigger `distribute()`.

**PoolFilter type:**
```
PoolFilter {
    card_type:  Option<CardType>,   // restrict to one card type (e.g., Minion only)
    class:      Option<Class>,       // restrict to a class or Neutral
    rarity:     Option<Vec<Rarity>>, // restrict to specific rarities
    max_cost:   Option<u32>,         // max mana cost (inclusive)
}
```
All fields are `None` by default (no restriction). A filter with all `None` draws from the full pool.

**5. Depletion handling**

- Shop/draw queries filter to `copies_remaining > 0` before rolling; never roll and reject.
- If an entire eligible rarity/class subset is exhausted, the shop slot returns `None` (UI shows empty slot).
- If no Neutral Rare/Legendary copies remain for an auction round, the auction is skipped silently.
- If a draw effect's entire eligible subset is exhausted, the draw produces nothing (silent failure, no compensation).

**6. Distribution visibility**

Players can see their own pool's remaining copy counts in the shop and hand UI (`copies_remaining`). Pool state is per-player and never shared — players do not see each other's pool state.

---

### States and Transitions

| State | Description | Valid transitions |
|---|---|---|
| `Unloaded` | `cards.json` not yet parsed; all queries invalid | → `Ready` (on successful load) |
| `Ready` | Pool initialized; all queries valid | → `Destroyed` (on game session end) |

No degraded or partial state. The pool either exists fully or not at all.

---

### Interactions with Other Systems

| System | Direction | What flows |
|---|---|---|
| **Server-side RNG** | Pool ← RNG | Seeds for all random draws |
| **Card Acquisition (Shop)** | Shop → Pool | Calls `draw_class_card()`, `draw_neutral_family()`, `draw_family_card()` for each slot each round; owns Phase 1 split roll and fallback logic; calls `distribute()` on purchase; owns within-round dedup policy |
| **Auction System** | Auction → Pool | Calls `draw_auction_card()`; `distribute()` on auction win |
| **Round State Machine** | RSM → Pool | Calls `draw_initial_draft()` at game start. Also triggers each DRAFT phase start event, which causes Card Acquisition to call shop draw functions each round. |
| **Shop UI / Board Rendering** | Read-only | Reads `copies_remaining()` for scarcity indicator display in shop UI (note: shop *cost* is rarity-based and static — `copies_remaining` is for the UI copy counter, not price calculation) |
| **Combat Resolution** | Combat → Pool | Reads card definitions by `id` for stats and keywords |
| **Board Rendering / UI** | Rendering → Pool | Reads card definitions by `id` for sprites and display |
| **Lightyear (network)** | Pool → Network | `copies_remaining()` delta-updated each round via `S2CPoolUpdate` (only changed counts). On initial connect or reconnect, server sends full pool state via `S2CPoolSnapshot` before resuming deltas. Also requires: `S2CDraftOffering` (9 initial draft card IDs), `S2CShopSlots` (3 shop slot IDs per round), `S2CAuctionCard` (auction card draw result broadcast). These message types to be fully specified in Network Protocol GDD. |

## Formulas

### Formula 1: Pool Copy Count

```
pool_copy_count = pool_copies_override           if pool_copies_override is not null AND > 0
               = rarity_base_copies[rarity]       if pool_copies_override is null
               = VALIDATION_ERROR (treat as null) if pool_copies_override <= 0
```

`override ≤ 0` (including 0 and all negative values) is rejected at load time with a logged error; the card is given its rarity default. Server does not abort.

**Upper bound:** No hard clamp is enforced in the formula — the valid data range is 1–99. Values above 99 are accepted but represent a data-entry error; enforcement is the responsibility of content-authoring tools, not the runtime formula.

| Rarity | Base Copies per Player Pool |
|---|---|
| Common | 6 |
| Uncommon | 5 |
| Rare | 4 |
| Epic | 1 |
| Legendary | 1 |

**Variables:**
| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Card rarity | `rarity` | enum | {Common…Legendary} | Card's rarity tier in cards.json |
| Override | `pool_copies_override` | int or null | 1–99 or null | Per-card override; null = use rarity default; 0 = validation error |
| Output | `pool_copy_count` | int | 1–99 | Copies in this player's pool at game start |

**Output Range:** 1 to 99. A card in the catalog always has at least 1 copy.
**Example:** Rare, no override → 4 copies. Same card with `pool_copies_override: 2` → 2 copies.

---

### Formula 2: Shop Slot Weighted Selection

This formula is implemented by `draw_class_card` (Phase 2, class slots) and `draw_neutral_family` (Phase 2, neutral slots). Phase 1 (split roll) and fallback logic are the calling system's responsibility. Phase 3 (`draw_family_card`) uses uniform selection, not this formula.

**Phase mapping:**

| Phase | Responsible | Pool function | Seeds |
|---|---|---|---|
| Phase 1: 50/50 split roll | Card Acquisition | (none — caller uses gen_range(0..2)) | 1 seed |
| Phase 2: Weighted pick | Pool | `draw_class_card` or `draw_neutral_family` | 1 seed |
| Phase 3: Uniform card within family | Pool | `draw_family_card` (neutral only) | 1 seed |
| Fallback (class exhausted → neutral) | Card Acquisition | (retry with new seeds) | 1 seed (new Phase 1) |

```
// Phase 2 algorithm (used by draw_class_card and draw_neutral_family):
// Precondition: |eligible_types| > 0 (guaranteed by fallback in calling system before this is called)
// CLASS slot: type = individual card_id (one weight entry per distinct card in the player's class)
// NEUTRAL slot: type = FAMILY (e.g., Gobball family; not individual cards)
//   total_acquired(t) is computed internally by the pool:
//   For class: total_acquired(card_id) = initial_count(card_id) − copies_remaining(card_id)
//   For neutral: total_acquired(family) = Σ total_acquired(card_id) for all card_ids in family
//   initial_count is stored at pool initialization and is never modified by distribute() calls.

raw_weight(t)        = (1 / |eligible_types|) + SHOP_WEIGHT_PER_CARD_OWNED × total_acquired(t)
raw_weight(t)        = clamp(raw_weight(t), 0.0, SHOP_WEIGHT_CAP)
normalized_weight(t) = raw_weight(t) / Σ raw_weight(t')   (for all t' in eligible_types)

// Phase 3 (draw_family_card): once a family is selected by Phase 2, draw uniformly from
// that family's available cards (all members with copies_remaining > 0, equal probability).
```

**Variables:**
| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Eligible types | `eligible_types` | set | ≥0 members | Class slots: {card_id \| player_class(card_id) == player_class AND copies_remaining(card_id) > 0}. Neutral slots: {family \| ∃ card_id in family with copies_remaining > 0}. |
| Count acquired | `total_acquired(t)` | int | 0–∞ | Cumulative copies of this type purchased via distribute() — never reset within a game session. For class slots: t = card_id. For neutral slots: t = family (sum across all card_ids in family). Computable as `initial_count(t) − copies_remaining(t)`; pool must retain initial per-card copy counts. |
| Weight bonus per card | `SHOP_WEIGHT_PER_CARD_OWNED` | float | tuning knob | **Default: 0.10** |
| Weight ceiling | `SHOP_WEIGHT_CAP` | float | >0.0 to 1.0 | **Default: 0.65** — activates at ~7 acquired copies. **Must be > 0** (cap = 0 causes division-by-zero in normalization). If cap < 1/\|eligible\_types\|, all types clamp to the same value and weighting becomes uniform (ownership signal lost). |
| Normalized weight | `normalized_weight(t)` | float | 0.0–1.0 | Final draw probability; all eligible types sum to 1.0 |

**Output Range:** Probability in [0.0, 1.0] per type, summing to 1.0. Returns `None` only if both subsets are fully exhausted.

**Example (total_acquired = 2 for each of 2 distinct Iop card IDs, class slot selected, 25 eligible Iop card IDs):**
```
Type A (×2): raw = 1/25 + 0.10×2 = 0.04 + 0.20 = 0.24
Type B (×2): raw = 0.24 (same)
Other 23: raw = 0.04 each
Sum = 0.24 + 0.24 + 23×0.04 = 1.40
P(Type A or B) = (0.24 + 0.24) / 1.40 = 34.3%   vs unweighted 8.0%   → 4.3x multiplier
```
This crosses the "feels intentional" threshold by mid-game (round 5-6).

---

### Formula 3: Spawn Range

```
fakes_clamped  = max(0, min(fakes_destroyed, 2))   // server clamps to valid range before evaluating
spawn_range    = min(1 + fakes_clamped × fake_objective_spawn_advance, 3)
spawn_rows     = { r ∈ {1, 2, 3} : r ≤ spawn_range }
```

`fake_objective_spawn_advance` is read from `GameConfig` (default 1). The lookup table below assumes the default value of 1:

| Fakes destroyed | Spawn range (at default advance=1) |
|---|---|
| 0 | Row 1 only |
| 1 | Rows 1–2 |
| 2+ | Rows 1–3 (maximum) |

At `fake_objective_spawn_advance = 2`: destroying 1 fake immediately unlocks Rows 1–3 (min(1 + 1×2, 3) = 3).

**`fakes_destroyed` state ownership:**
- **1v1**: per-player counter. Only tracks fakes YOU destroyed (not fakes your opponent destroyed).
- **2v2 / team modes**: per-team counter. Any team member destroying a fake increments the shared counter and expands ALL team members' spawn range simultaneously.
- **Lifetime**: monotonically non-decreasing within a game session. Never resets. Destroyed with the game session when it ends.
- **Data location**: `Player.fakes_destroyed` (1v1) or `Team.fakes_destroyed` (team modes). Maximum value stored: 2 (no mechanical benefit beyond 2 fakes).

**Variables:**
| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Fakes destroyed | `fakes_destroyed` | int | 0–2 | Capped at 2; only fakes you destroyed count (per-player) |
| Spawn advance | `fake_objective_spawn_advance` | u32 | 1–2 | From `GameConfig`; rows unlocked per fake destroyed (default 1) |
| Spawn rows | `spawn_rows` | set | {1}–{1,2,3} | Legal rows for Minion placement |

**Output Range:** Always a contiguous prefix of {1,2,3}. Monotonically non-decreasing.
**Example:** Team destroys 1 fake → both teammates can spawn at Row 1 or Row 2. Second fake → both can spawn at Row 1, 2, or 3.
**Structure exception:** Structures bypass this formula entirely — valid placement is any of your home cells (Cells 1–4 of all 5 lanes).

## Edge Cases

- **If `pool_copies_override ≤ 0` (zero or any negative integer):** Validation error at load time. Card ID is logged; card receives its rarity default copy count. Server does not abort. (Formula condition is `> 0` — both 0 and negatives are rejected.)

- **If class slot rolls but all class cards are exhausted:** Card Acquisition detects that `draw_class_card` returned None, then retries as a neutral slot (calling `draw_neutral_family` with a new seed). If both subsets are fully exhausted, the slot returns `None` (empty slot in UI). The pool does not perform this fallback internally — the calling system owns the retry logic.

- **Late-game class pool exhaustion:** When a player has purchased all available copies of all their class cards, `draw_class_card` returns None every time. The shop will show only neutral cards (via fallback) for the rest of the game. This is correct and intentional — the player has exhausted their class archetype pool. Add AC: GIVEN all class cards have `copies_remaining = 0`, WHEN `draw_class_card(class, seed)` is called, THEN it returns None.

- **Single eligible type (`eligible_types = 1`):** Formula 2 normalization produces `normalized_weight = 1.0` for the sole remaining eligible type. That card appears in every draw until purchased. This is correct behavior — not a bug — and signals that the archetype slot is nearly exhausted.

- **If the entire pool is exhausted:** `draw_random()` and draw-effect calls return `None`. Draw effects silently fail with no compensation.

- **If a Structure placement specifies Cell 5–8 (opponent's territory):** Server rejects with an error. Valid Structure placement: `target_cell ∈ {1, 2, 3, 4}` on the placing player's side only.

- **If `fakes_destroyed > 2`:** Clamped to 2. No additional spawn benefit beyond destroying both fakes.

- **Both players at Row 3 simultaneously:** Legal. Units at Row 3 with MP 3+ reach opponent territory on the same round they're placed. Accepted design outcome — "board compression state." Not a bug.

- **Minion at Row 3 with CHARGE X:** CHARGE X fires in Resolution sub-step 2 after placement. A Row 3 Minion + CHARGE 3 reaches Cell 6 (opponent's territory) in the same round it enters play. Legal and intentional.

- **Single-card neutral family:** A family with only 1 card remaining (`copies_remaining = 1`) is eligible with full weighting. It is removed from eligible types only when `copies_remaining = 0`.

- **Initial draft duplicates:** `draw_initial_draft()` returns 9 distinct card IDs — no duplicates within the 9-card batch, even if a type has multiple copies in the pool. Copies purchased are decremented; copies not purchased remain available.

- **Archetype lock-in with no pivot:** Once a player commits to an archetype, the shop weighting reinforces it for the rest of the game. There is no sell or discard mechanic — `total_acquired` never decreases, so weights are permanent commitments. This is **intentional commitment-under-uncertainty design** — early choices carry real stakes. Whether this creates a "trapped losing strategy" problem will be validated in playtesting. A pivot mechanic (e.g., sell for partial gold refund) is deferred to post-prototype iteration.

- **Client reconnection mid-game:** On reconnect, the server sends a full `S2CPoolSnapshot` message containing current `copies_remaining` for all cards in this player's pool, then resumes delta `S2CPoolUpdate` messages normally. This is the only recovery path — there is no partial-state sync. The client must discard its local pool state and rebuild from the snapshot.

## Dependencies

| System | Relationship | Interface |
|---|---|---|
| Server-side RNG | Hard upstream | Pool receives seeds for all random draws; never owns RNG source |
| Game Config | Hard upstream | Reads `SHOP_WEIGHT_PER_CARD_OWNED`, `SHOP_WEIGHT_CAP`, copy count defaults |
| Card Acquisition (Shop) | Downstream | Calls `draw_class_card()`, `draw_neutral_family()`, `draw_family_card()`, `draw_initial_draft()`, `distribute()` on purchase. Owns Phase 1 split roll and fallback logic. |
| Auction System | Downstream | Calls `draw_auction_card()`, `distribute()` on auction win |
| Round State Machine | Downstream | Calls `draw_initial_draft()` once at session start |
| Combat Resolution | Downstream (read-only) | Reads card definitions by `id` for stats and keywords |
| Board Rendering / Hand UI | Downstream (read-only) | Reads card definitions by `id` for display; reads `copies_remaining()` for UI |
| Lightyear Network | Downstream | Sends `copies_remaining()` to clients via `S2CPoolUpdate` (delta only); also requires `S2CDraftOffering`, `S2CShopSlots`, `S2CAuctionCard` (to be defined in Network Protocol GDD) |
| Game Session System | Hard upstream | Must perform `cards.json` catalog hash handshake at connection time to detect client/server version skew before a game starts |

**Shared Auction Pool note:** `draw_auction_card()` draws from a shared game-level pool (not per-player). The architecture of that pool — initialization, depletion tracking, multi-player behavior — is the Auction System GDD's responsibility. This GDD commits only to the function interface and the Neutral-only, no-Epic constraint.

## Tuning Knobs

| Knob | Default | Safe Range | Impact |
|---|---|---|---|
| `common_pool_copies` | 6 | 3–10 | Higher = more Common diversity; lower = scarcity and empty slots more likely |
| `uncommon_pool_copies` | 5 | 3–8 | Same as above for Uncommons |
| `rare_pool_copies` | 4 | 1–6 | At 1: Rare feels as scarce as Epic; at 6: Rares freely available all game |
| `epic_pool_copies` | 1 | 1 only | Epic is once-per-player — same scarcity weight as Legendary but available in the shop, not auction |
| `legendary_pool_copies` | 1 | 1 only | Not tunable — Legendary scarcity is load-bearing for the player fantasy |

Knobs that affect this system but are owned by the master GDD (Section 7):
- `shop_weight_per_card` (default 10%) — how strongly the shop leans toward your archetype
- `shop_weight_cap` (default 65%) — prevents scripted late-game shops at deep stacks
- `fake_objective_spawn_advance` (default 1 cell per fake) — governs spawn range expansion

## Visual/Audio Requirements

[To be designed]

## UI Requirements

[To be designed]

## Acceptance Criteria

### Pool Loading

| # | Criterion | Type |
|---|---|---|
| CP1a | **GIVEN** a `cards.json` fixture with N card definitions, **WHEN** pool initialization completes, **THEN** `card_catalog.len() == N` and `player_pool.len() == N`. | BLOCKING |
| CP1b | **GIVEN** a `cards.json` fixture with at least one card of each rarity, **WHEN** pool initialization completes, **THEN** `copies_remaining(card_id) >= 1` for every card in the catalog. | BLOCKING |
| CP2 | **GIVEN** a card with `pool_copies_override: 0` or any negative integer, **WHEN** the pool initializes, **THEN**: (1) pool initialization does not abort; (2) `copies_remaining(card_id)` equals the rarity default; (3) an error-level log entry containing the card's ID is emitted. | BLOCKING |
| CP3a | **GIVEN** `cards.json` does not exist at the expected path, **WHEN** `load_card_catalog()` is called, **THEN** it returns `Err` containing the attempted file path. | BLOCKING |
| CP3b | **GIVEN** `cards.json` exists but contains invalid JSON, **WHEN** `load_card_catalog()` is called, **THEN** it returns `Err` containing the file path and parse error location. | BLOCKING |
| CP3c | **GIVEN** `cards.json` is valid JSON but contains two card definitions with identical `id` values (e.g., `"iop_001"` appearing twice), **WHEN** `load_card_catalog()` is called, **THEN** it returns `Err` whose string representation contains all duplicate IDs and the server does not start. | BLOCKING |
| CP3d | **GIVEN** `cards.json` contains a card definition with a missing or unrecognized `rarity` field (e.g., `rarity: "UltraRare"`), **WHEN** `load_card_catalog()` is called, **THEN** it returns `Err` containing the offending card's ID and the server does not start. | BLOCKING |

### Pool Queries & Distribution

| # | Criterion | Type |
|---|---|---|
| CP4 | **GIVEN** a Rare card with no `pool_copies_override`, **WHEN** `player_pool.initialize()` completes (before any draws), **THEN** `copies_remaining(card_id) == 4`. | BLOCKING |
| CP5 | **GIVEN** `copies_remaining(card_id) == N` (N > 0), **WHEN** `distribute(card_id)` is called, **THEN** `copies_remaining(card_id) == N - 1` and `distribute` returns `Ok(())`. | BLOCKING |
| CP5c | **GIVEN** `copies_remaining(card_id) == 0`, **WHEN** `distribute(card_id)` is called, **THEN** it returns `Err(DistributeError::Exhausted)` and `copies_remaining` remains 0. | BLOCKING |
| CP5b | **GIVEN** a valid positive `pool_copies_override: 2` on a Rare card, **WHEN** `player_pool.initialize()` completes, **THEN** `copies_remaining(card_id) == 2` (override applied, not the rarity default of 4). | BLOCKING |
| CP6a | **GIVEN** `copies_remaining(card_id) == 0`, **WHEN** `is_available(card_id)` is called, **THEN** returns `false`. | BLOCKING |
| CP6b | **GIVEN** `copies_remaining(card_id) >= 1`, **WHEN** `is_available(card_id)` is called, **THEN** returns `true`. | BLOCKING |
| CP7 | **GIVEN** a pool where all class cards have `copies_remaining = 0` and at least one neutral card has `copies_remaining >= 1`, **WHEN** `draw_class_card(class, seed)` is called 100 times with distinct seeds, **THEN** every call returns `None` (class exhausted; fallback is the calling system's responsibility — the pool does not route to neutral internally). | BLOCKING |
| CP7b | **GIVEN** a pool where all class cards have `copies_remaining = 0` and at least one neutral card has `copies_remaining >= 1`, **WHEN** Card Acquisition executes the full fallback flow (detect `draw_class_card` → None, retry via `draw_neutral_family` + `draw_family_card` with new seeds), **THEN** the returned `CardId` belongs to the neutral subset (not `None`). *(Integration test — verifies Card Acquisition's fallback logic, not pool internals.)* | BLOCKING |
| CP8a | **GIVEN** both class and neutral subsets have `copies_remaining = 0` for all cards, **WHEN** `draw_shop_slot()` is called, **THEN** returns `None`. | BLOCKING |
| CP8b | **GIVEN** `draw_shop_slot()` returns `None` for a shop slot, **WHEN** the shop UI renders that slot, **THEN** the slot displays an empty-slot visual state. | ADVISORY |
| CP-IC | **GIVEN** a pool initialized with `initial_count(card_id) = N` (from rarity default or override), **WHEN** `distribute(card_id)` is called K times (K < N), **THEN** `initial_count(card_id) == N` (unchanged), `copies_remaining(card_id) == N - K`, and `total_acquired(card_id) == K`. *(Verifies initial_count immutability, Formula 2's total_acquired computation, and pool retention of initial counts.)* | BLOCKING |
| CP-SHC | **GIVEN** a pool with eligible class cards, **WHEN** `draw_class_card(class, seed)` is called, **THEN** the returned CardId has `class == player_class` and `copies_remaining(card_id) >= 1` at call time. | BLOCKING |
| CP-SHN | **GIVEN** a pool with eligible neutral families (at least one card with `copies_remaining >= 1`), **WHEN** `draw_neutral_family(seed)` followed by `draw_family_card(family, seed2)` is called, **THEN** the returned CardId belongs to the returned FamilyId, `class == Neutral`, and `copies_remaining(card_id) >= 1`. | BLOCKING |
| CP-NW | **GIVEN** any non-empty set of eligible_types with mixed owned/unowned types, **WHEN** `normalized_weight(t)` is computed for all t in eligible_types, **THEN** `|Σ normalized_weight(t) − 1.0| < 1e-6`. | BLOCKING |
| CP-A | **GIVEN** a pool where all Neutral Rare and Legendary cards have `copies_remaining = 0`, **WHEN** `draw_auction_card(seed)` is called, **THEN** it returns `None`. | BLOCKING |
| CP-B | **GIVEN** a pool where all cards matching a given `PoolFilter` have `copies_remaining = 0`, **WHEN** `draw_random(filter, seed)` is called, **THEN** it returns `None` and no `distribute()` is called. | BLOCKING |
| CP-C | **GIVEN** a pool with sufficient eligible cards for a player's class, **WHEN** `draw_initial_draft(class, 9, seed)` is called, **THEN** the returned `Vec<CardId>` has length 9 and contains no duplicate IDs. | BLOCKING |
| CP-C2 | **GIVEN** `draw_initial_draft(class=Iop, 9, seed)` returns a `Vec<CardId>` of length 9, **WHEN** each CardId is queried for its class field, **THEN** every CardId has `class == Iop OR class == Neutral`, and no CardId has any other class value. | BLOCKING |
| CP-C3 | **GIVEN** `draw_initial_draft(class, 9, seed)` returns `Vec<CardId>`, **WHEN** any card in the returned Vec is checked against `distribute()` calls, **THEN** `copies_remaining(card_id)` is unchanged for all unselected (not purchased) cards — `distribute()` is NOT called for cards the player did not choose. | BLOCKING |

### Shop Weighting

| # | Criterion | Type |
|---|---|---|
| CP9 | **GIVEN** a player owns 3 copies of card type T (class slot, 25 eligible types, no other ownership), SHOP_WEIGHT_PER_CARD=0.10, SHOP_WEIGHT_CAP=0.65, **WHEN** `raw_weight(T)` and `normalized_weight(T)` are computed, **THEN** `raw_weight(T) == 0.34` (±1e-6), `normalized_weight(T) == 0.2615` (±1e-4), and `normalized_weight(T) > normalized_weight(U)` for all unweighted types U. | BLOCKING |
| CP10 | **GIVEN** `GameConfig.shop_weight_cap = 0.65`, `GameConfig.shop_weight_per_card = 0.10`, 25 eligible types, and a player owns exactly 7 copies of card type T, **WHEN** the pre-clamp raw weight for T is computed (`1/25 + 0.10×7 = 0.74`), **THEN** the clamped output equals `GameConfig.shop_weight_cap` (0.65). | BLOCKING |

### Spawn Range

*(Terminology: "Row N" = Cell N on the player's own side of the board, counting from their spawn edge. Row 1 = Cell 1, Row 2 = Cell 2, Row 3 = Cell 3.)*

| # | Criterion | Type |
|---|---|---|
| CP11 | **GIVEN** a 1v1 game where `player.fakes_destroyed = 0`, **WHEN** the player submits a Minion placement at Row 2, **THEN** the server rejects it. | BLOCKING |
| CP12 | **GIVEN** a 1v1 game where `player.fakes_destroyed = 1`, **WHEN** the player submits a Minion placement at Row 2, **THEN** it is accepted and the unit is created at Row 2 in that lane. | BLOCKING |
| CP13 | **GIVEN** a 1v1 game where `player.fakes_destroyed = 2`, **WHEN** the player submits a Minion placement at Row 3, **THEN** it is accepted. | BLOCKING |
| CP14 | **GIVEN** `fakes_destroyed ∈ {0, 1, 2}` (tested for each), **WHEN** a Structure is placed at Row 4 (player's deepest home cell), **THEN** placement is accepted in all three cases. | BLOCKING |
| CP15 | **GIVEN** any `fakes_destroyed` value, **WHEN** a Structure placement targets Row 5 or beyond (opponent's territory), **THEN** the server rejects it. | BLOCKING |
| CP16 | **GIVEN** a 2v2 game where Player A destroys 1 of the opponent's fake objectives, **WHEN** Player B's spawn range is evaluated, **THEN** `spawn_rows = {1, 2}` for Player B (team-shared counter). | BLOCKING |
| CP16b | **GIVEN** a 1v1 game where Player A has `fakes_destroyed = 1` and Player B has `fakes_destroyed = 0`, **WHEN** Player B's spawn range is evaluated, **THEN** `spawn_rows = {1}` (B's counter is independent of A's). | BLOCKING |

## Open Questions

| # | Question | Owner | Notes |
|---|---|---|---|
| OQ1 | How many original Trap, Structure, and Field cards should the game ship with? What are their names and effects? | Game Designer | These card types don't exist in Krosmaga Extension=1; original designs required |
| OQ2 | Legendary 3-level evolution mechanic: keep as-is from Krosmaga, simplify to single level, or design new evolution rules? | Game Designer | Deferred for hackathon; Legendary cards ship as Level 1 only for now |
| OQ3 | Should the UI show a subtle indicator when shop weighting fires above a threshold (e.g., 2× multiplier), so hardcore players can "see" the system? | UX Designer | Economy designer recommended this; not blocking for launch |
| OQ4 | Exact `cards.json` schema for Structure, Trap, and Field card effects: how are continuous/triggered effects encoded when they're too complex for simple text? | Gameplay Programmer | Effect text is adequate for hackathon; structured effect encoding is post-launch |
| OQ5 | 2v2v2 mode: `fakes_destroyed` is per-team — but which enemy team's fakes count? Both? Only the team you last attacked? | Game Designer | Deferred until 2v2v2 is prioritized |
| OQ6 | `DoubleFace` card schema: what fields does the second face require? Does it have its own cost, stats, and effect_text, or is it a transformed version of the same card? | Game Designer / Gameplay Programmer | Required before any DoubleFace cards can be added to cards.json |
| OQ7 | **RESOLVED (2026-04-29):** The auction is a shared/common event — one card is drawn per auction round and all players bid on the same card simultaneously. The auction draws from a **shared neutral auction pool** that is separate from every player's personal shop pool. There is no shop/auction copy collision — the per-player shop pool and the shared auction pool are independent. Epic cards are excluded from auction (class-specific; no Neutral Epics exist). Full shared auction pool management (size, cycle-out rules, multi-player behavior) to be specified in Auction System GDD. | Auction System GDD | Shared pool architecture confirmed; implementation details deferred to Auction GDD |
