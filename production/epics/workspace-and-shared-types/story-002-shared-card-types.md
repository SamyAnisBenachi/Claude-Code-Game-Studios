# Story 002: Shared Card Types

> **Epic**: Workspace & Shared Types
> **Status**: Complete
> **Layer**: Foundation
> **Type**: Config/Data
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/card-data-pool.md`
**Requirement**: TR-??? (TR registry not yet populated — covers TR-CDP-01, TR-CDP-03)
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-006: Card Data Schema and Pool State Architecture (Part 1 — `shared/src/card.rs`)
**ADR Decision Summary**: Card definitions live in `shared/src/card.rs` as pure serde types — no Bevy plugin derives. `CardCatalog` is a type alias for `HashMap<CardId, CardData>`. `EPIC_POOL_COPIES = 1` and `LEGENDARY_POOL_COPIES = 1` are compile-time consts (not `GameConfig` fields) because their scarcity is a load-bearing design pillar.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: LOW
**Engine Notes**: `serde` and `HashMap` are pure-Rust crates with no Bevy API surface. No post-cutoff API risk. Derive macros are standard — no Bevy 0.18-specific concerns on this file.

**Control Manifest Rules (Foundation layer)**:
- Required: Epic and Legendary copy counts are compile-time constants (`EPIC_POOL_COPIES = 1`, `LEGENDARY_POOL_COPIES = 1`), never `GameConfig` fields.
- Required: `CardCatalog` is immutable after load — never mutate card definitions mid-session.
- Forbidden: Never derive `Resource`, add plugin code in `shared/`.
- Guardrail: O(1) `CardCatalog` lookup by `CardId` via `HashMap`.

---

## Acceptance Criteria

- [x] `shared/src/card.rs` implements `CardId(pub u32)` as a newtype with `#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]`
- [x] `Rarity` enum has exactly 5 variants: `Common`, `Uncommon`, `Rare`, `Epic`, `Legendary` with serde + copy derives
- [x] `ClassId` enum has exactly 7 variants: `Iop`, `Cra`, `Sacrier`, `Xelor`, `Ecaflip`, `Sadida`, `Neutral` with serde + copy derives
- [x] `CardType` enum has exactly 7 variants: `Minion`, `Spell`, `Trap`, `Structure`, `Field`, `Order`, `DoubleFace` with serde + copy derives
- [x] `UnitType` enum has exactly 4 variants: `Blade`, `Arcane`, `Shield`, `Neutral` (default) with serde + copy derives
- [x] `SimpleKeyword` enum exists with at minimum: `FirstStrike`, `Charge`, `AppearanceTrigger`, `DeathTrigger`, `FinalBlowTrigger`, `CounterattackTrigger`, `StartOfTurnTrigger`, `EndOfTurnTrigger`
- [x] `Keyword` enum uses `#[serde(untagged)]` with variants `Simple(SimpleKeyword)`, `RangeX { kw: String, max_range: u8 }`, `ChargeXMove { kw: String, cells: u8 }`, `ResistanceX { kw: String, value: u8 }`
- [x] `CardData` struct has all base fields: `id: CardId`, `name_fr: String`, `name_en: String`, `class: ClassId`, `family: Option<String>`, `rarity: Rarity`, `card_type: CardType`, `unit_type: UnitType`, `cost: u32`, `atk: u8`, `hp: u8`, `mp: u8`, `ar: u8`, `keywords: Vec<Keyword>`, `effect_text: String`, `art_id: String`, `pool_copies_override: Option<i32>`
- [x] `pub type CardCatalog = std::collections::HashMap<CardId, CardData>;` defined in this module
- [x] `pub const EPIC_POOL_COPIES: u32 = 1;` and `pub const LEGENDARY_POOL_COPIES: u32 = 1;` defined (not in `GameConfig`)
- [x] `shared/src/lib.rs` re-exports the card module (`pub mod card;`)
- [x] `cargo check -p shared` — covered by CI gate; ADVISORY for Config/Data story type

---

## Implementation Notes

*Derived from ADR-006 Part 1 (shared/src/card.rs):*

**Why `CardId` is a newtype (not `u32` directly):** The newtype wrapper prevents accidental integer arithmetic on IDs. A system cannot do `card_id + 1` or use a raw `u32` where a `CardId` is expected — the compiler enforces it. This matters when iterating lanes (which use lane indices as integers) alongside card IDs.

**`Keyword` serde shape:** The `#[serde(untagged)]` on `Keyword` means the JSON deserialiser tries each variant in order. `Simple` wraps a `SimpleKeyword` which uses the standard tagged enum format. Parameterised keywords (`RangeX` etc.) include a `"kw"` field that identifies the keyword name as a string. This allows the card JSON to be authored as either `"FirstStrike"` (simple) or `{ "kw": "RangeX", "max_range": 3 }` (parameterised). Do not change this serde shape without also updating `cards.json`.

**`CardData` stat fields on non-Minion cards:** Per GDD Rule 3, all stat fields (`atk`, `hp`, `mp`, `ar`) are present on all card types but carry zero values where semantically absent (e.g., `mp = 0` on Structures). Systems must check `card_type` before interpreting stats. Do not use `Option<u8>` for these fields — the GDD's explicit zero convention keeps the schema uniform and avoids serde `null` handling.

**`pool_copies_override: Option<i32>`:** Type is `i32` (not `u32`) to allow the JSON to contain negative values, which are a soft error (warn + use rarity default). If the type were `u32`, a `-1` in the JSON would fail to parse entirely rather than producing the intended soft-error path.

**`family: Option<String>`:** `None` for class cards; `Some("FamilyName")` for neutral cards. The server builds a `FamilyIndex: HashMap<String, Vec<CardId>>` at startup from this field, but that index lives in `server/` — not here.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 003: `GameConfig` struct in `shared/src/config.rs`
- Story 004: CI dependency-tree gates
- Epic 2 (game-config-pipeline): `CardCatalog` loading from `cards.json`
- Epic (Core — Card Data & Pool): `PlayerPool`, draw functions, pool mutations

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode. Basic checks below.*

- **AC: All types compile and serde round-trip**
  - Given: A `CardData` instance constructed in a unit test with all fields populated
  - When: Serialised to JSON via `serde_json::to_string()` then deserialised back
  - Then: The round-tripped value equals the original (`PartialEq`)
  - Edge cases: `keywords` containing both `Simple` and parameterised variants; `pool_copies_override: Some(-1)` serialises and deserialises correctly as `-1` (negative i32)

- **AC: EPIC/LEGENDARY consts are not in GameConfig**
  - Given: `shared/src/card.rs` as written
  - When: Grep for `EPIC_POOL_COPIES` across the codebase
  - Then: Found only in `shared/src/card.rs` as a `const`, never as a field name in any struct

- **AC: `cargo check -p shared` clean**
  - Given: Story 001 complete (workspace exists), Story 002 implemented
  - When: `cargo check -p shared` is run
  - Then: Zero errors, zero warnings

---

## Test Evidence

**Story Type**: Config/Data
**Required evidence**: Smoke check — `cargo check -p shared` output showing zero warnings — paste into `tests/evidence/story-002-shared-types-check.md`
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (workspace scaffolding) must be Done
- Unlocks: Story 004 (protocol skeleton + CI gates)
