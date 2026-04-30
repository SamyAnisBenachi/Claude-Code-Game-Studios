// shared/src/card.rs — Card data types, shared between server and client.
// ADR-006 Part 1: pure serde types, no Bevy derives. Immutable after load.

use serde::{Deserialize, Serialize};

/// Stable numeric identifier matching Krosmaga Extension=1 card IDs.
/// Newtype wrapper prevents accidental integer arithmetic on IDs.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CardId(pub u32);

/// Rarity tier. Determines base pool copy count and auction eligibility.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Card category. Determines which stat fields are present and valid.
///
/// - `Minion`: has `cost`, `atk`, `hp`, `mp`, `ar`
/// - `Spell`: has `cost` only (beyond base fields)
/// - `Trap`: has `cost` only
/// - `Structure`: has `cost`, `atk`=0, `mp`=0, `hp`, `ar`
/// - `Field`: has `cost` only; effect in `effect_text`
/// - `Order`: has `cost` only
/// - `DoubleFace`: second-face schema TBD (GDD OQ6)
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CardType {
    Minion,
    Spell,
    Trap,
    Structure,
    Field,
    Order,
    DoubleFace,
}

/// Class affiliation. `Neutral` cards belong to no class and appear in
/// every player's shop neutral pool.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClassId {
    Iop,
    Cra,
    Sacrier,
    Xelor,
    Ecaflip,
    Sadida,
    Neutral,
}

/// Unit type used for archetype/interaction tagging.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitType {
    Blade,
    Arcane,
    Shield,
    Neutral,
}

/// Keyword variants.
///
/// No-parameter keywords are plain enum variants.
/// Parameterized keywords carry their value inline.
/// `#[serde(untagged)]` allows JSON: `"FirstStrike"` or `{"kw":"RangeX","max_range":3}`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Keyword {
    Simple(SimpleKeyword),
    RangeX {
        #[serde(rename = "kw")]
        kw: String,
        max_range: u8,
    },
    ChargeXMove {
        #[serde(rename = "kw")]
        kw: String,
        cells: u8,
    },
    ResistanceX {
        #[serde(rename = "kw")]
        kw: String,
        value: u8,
    },
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SimpleKeyword {
    FirstStrike,
    Charge,
    AppearanceTrigger,
    DeathTrigger,
    FinalBlowTrigger,
    CounterattackTrigger,
    StartOfTurnTrigger,
    EndOfTurnTrigger,
}

/// Immutable definition of one card. Loaded from `assets/data/cards.json`.
///
/// Stat fields (`atk`, `hp`, `mp`, `ar`, `cost`) are present on all cards but
/// carry zero values where semantically absent. Systems must check `card_type`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CardData {
    pub id: CardId,
    pub name_fr: String,
    pub name_en: String,
    pub class: ClassId,
    pub family: Option<String>,
    pub rarity: Rarity,
    pub card_type: CardType,
    pub unit_type: UnitType,
    pub cost: u32,
    pub atk: u8,
    pub hp: u8,
    pub mp: u8,
    pub ar: u8,
    pub keywords: Vec<Keyword>,
    pub effect_text: String,
    pub art_id: String,
    /// `None` → rarity default. `Some(n >= 1)` → use n. `Some(n <= 0)` → soft error, use rarity default.
    pub pool_copies_override: Option<i32>,
}

/// Immutable map of all card definitions. Built once at server startup.
/// Server inserts as `Res<CardCatalog>`. Client builds for display lookup.
pub type CardCatalog = std::collections::HashMap<CardId, CardData>;

/// Epic and Legendary copy counts are compile-time constants — never `GameConfig` fields.
/// Their scarcity is a load-bearing design pillar (card-data-pool.md Player Fantasy).
pub const EPIC_POOL_COPIES: u32 = 1;
pub const LEGENDARY_POOL_COPIES: u32 = 1;
