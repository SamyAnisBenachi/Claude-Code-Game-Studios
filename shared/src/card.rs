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
/// No-parameter keywords are wrapped in `Simple(SimpleKeyword)`.
/// Parameterized keywords carry their value in the adjacent `val` object.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kw", content = "val")]
pub enum Keyword {
    Simple(SimpleKeyword),
    RangeX { max_range: u8 },
    ChargeXMove { cells: u8 },
    ResistanceX { value: u8 },
    VulnerabilityX { value: u8 },
    RepelX { distance: u8 },
    AttractX { distance: u8 },
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SimpleKeyword {
    Appearance,
    Death,
    FinalBlow,
    Counterattack,
    StartOfTurn,
    EndOfTurn,
    FirstStrike,
    Haste,
    Wall,
    Bodyguard,
    Irremovable,
    Untargetable,
    Shield,
    Leader,
    Outnumbered,
    ArmorPiercing,
    Silence,
    Stun,
    Teleport,
    ChangeLane,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keyword_adjacent_tag_round_trips_all_variants() {
        let keywords = vec![
            Keyword::Simple(SimpleKeyword::Shield),
            Keyword::RangeX { max_range: 3 },
            Keyword::ChargeXMove { cells: 2 },
            Keyword::ResistanceX { value: 1 },
            Keyword::VulnerabilityX { value: 2 },
            Keyword::RepelX { distance: 4 },
            Keyword::AttractX { distance: 5 },
        ];

        let encoded = serde_json::to_value(&keywords).expect("keywords should serialize");
        assert_eq!(
            encoded,
            json!([
                { "kw": "Simple", "val": "Shield" },
                { "kw": "RangeX", "val": { "max_range": 3 } },
                { "kw": "ChargeXMove", "val": { "cells": 2 } },
                { "kw": "ResistanceX", "val": { "value": 1 } },
                { "kw": "VulnerabilityX", "val": { "value": 2 } },
                { "kw": "RepelX", "val": { "distance": 4 } },
                { "kw": "AttractX", "val": { "distance": 5 } }
            ])
        );

        let decoded: Vec<Keyword> =
            serde_json::from_value(encoded).expect("keywords should deserialize");
        assert_eq!(decoded, keywords);
    }

    #[test]
    fn all_simple_keywords_round_trip_through_simple_keyword_variant() {
        let simple_keywords = [
            SimpleKeyword::Appearance,
            SimpleKeyword::Death,
            SimpleKeyword::FinalBlow,
            SimpleKeyword::Counterattack,
            SimpleKeyword::StartOfTurn,
            SimpleKeyword::EndOfTurn,
            SimpleKeyword::FirstStrike,
            SimpleKeyword::Haste,
            SimpleKeyword::Wall,
            SimpleKeyword::Bodyguard,
            SimpleKeyword::Irremovable,
            SimpleKeyword::Untargetable,
            SimpleKeyword::Shield,
            SimpleKeyword::Leader,
            SimpleKeyword::Outnumbered,
            SimpleKeyword::ArmorPiercing,
            SimpleKeyword::Silence,
            SimpleKeyword::Stun,
            SimpleKeyword::Teleport,
            SimpleKeyword::ChangeLane,
        ];

        for simple_keyword in simple_keywords {
            let keyword = Keyword::Simple(simple_keyword);
            let encoded = serde_json::to_string(&keyword).expect("simple keyword should serialize");
            let decoded: Keyword =
                serde_json::from_str(&encoded).expect("simple keyword should deserialize");
            assert_eq!(decoded, keyword);
        }
    }

    #[test]
    fn cards_fixture_uses_current_keyword_schema() {
        let cards: Vec<CardData> =
            serde_json::from_str(include_str!("../../assets/data/cards.json"))
                .expect("cards.json should deserialize with adjacent-tag keywords");

        assert!(!cards.is_empty());
    }
}
