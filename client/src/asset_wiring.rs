use bevy::prelude::Component;
use shared::card::{CardCatalog, CardData};

const CARD_DATA_JSON: &str = include_str!("../../assets/data/cards.json");


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

    let path = format!("art/cards/display/card_{art_id}_art_display.png");
    Ok(Box::leak(path.into_boxed_str()))
}
