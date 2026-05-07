use bevy::prelude::Component;
use shared::card::{CardCatalog, CardData};

const CARD_DATA_JSON: &str = include_str!("../../assets/data/cards.json");

const DISPLAY_ART_PATHS: [(&str, &str); 8] = [
    (
        "cra_piercing_shot_003",
        "art/cards/display/card_cra_piercing_shot_003_art_display.png",
    ),
    (
        "ecaflip_decree_007",
        "art/cards/display/card_ecaflip_decree_007_art_display.png",
    ),
    (
        "gobball_sturdy_005",
        "art/cards/display/card_gobball_sturdy_005_art_display.png",
    ),
    (
        "iop_double_face_008",
        "art/cards/display/card_iop_double_face_008_art_display.png",
    ),
    (
        "iop_knight_001",
        "art/cards/display/card_iop_knight_001_art_display.png",
    ),
    (
        "sacrier_foot_002",
        "art/cards/display/card_sacrier_foot_002_art_display.png",
    ),
    (
        "sadida_rose_field_006",
        "art/cards/display/card_sadida_rose_field_006_art_display.png",
    ),
    (
        "xelor_time_trap_004",
        "art/cards/display/card_xelor_time_trap_004_art_display.png",
    ),
];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardDisplayArtAsset {
    pub path: &'static str,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardDisplayArtFallback {
    pub reason: CardDisplayArtFallbackReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardDisplayArtFallbackReason {
    NoArtId,
    MissingDisplayAsset,
}

pub fn default_client_card_catalog() -> CardCatalog {
    serde_json::from_str::<Vec<CardData>>(CARD_DATA_JSON)
        .expect("assets/data/cards.json should deserialize for client display catalog")
        .into_iter()
        .map(|card| (card.id, card))
        .collect()
}

pub fn resolve_card_display_art(
    card: Option<&CardData>,
) -> Result<&'static str, CardDisplayArtFallbackReason> {
    let Some(card) = card else {
        return Err(CardDisplayArtFallbackReason::MissingDisplayAsset);
    };
    let art_id = card.art_id.trim();
    if art_id.is_empty() {
        return Err(CardDisplayArtFallbackReason::NoArtId);
    }

    DISPLAY_ART_PATHS
        .iter()
        .find_map(|(known_art_id, path)| (*known_art_id == art_id).then_some(*path))
        .ok_or(CardDisplayArtFallbackReason::MissingDisplayAsset)
}
