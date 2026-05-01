use bevy::prelude::*;
use shared::card::{CardId, Rarity};

use crate::core::rsm::AuctionPhaseEntered;
use crate::foundation::config::{CardCatalog, GameConfig};

use super::state::{AuctionPhase, AuctionState};

/// Internal server queue item for the future network dispatch story.
///
/// This mirrors `shared::protocol::S2CAuctionCard` without adding Bevy
/// dependencies to `shared/`.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct S2CAuctionCard {
    pub card_id: CardId,
    pub starting_price: u32,
}

/// Temporary draw fixture for AUC-002 through AUC-006.
///
/// Story AUC-008 owns real shared auction-pool integration. Until then, tests
/// inject the card that would have been drawn.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct AuctionCardDrawFixture {
    pub card_id: Option<CardId>,
}

impl AuctionCardDrawFixture {
    pub fn with_card(card_id: CardId) -> Self {
        Self {
            card_id: Some(card_id),
        }
    }
}

pub fn auction_tick_system(
    mut auction: ResMut<AuctionState>,
    mut phase_entered: MessageReader<AuctionPhaseEntered>,
    draw_fixture: Option<Res<AuctionCardDrawFixture>>,
    catalog: Res<CardCatalog>,
    config: Res<GameConfig>,
    mut auction_cards: MessageWriter<S2CAuctionCard>,
) {
    for event in phase_entered.read() {
        if auction.phase != AuctionPhase::Idle {
            tracing::error!(
                round = event.round,
                phase = ?auction.phase,
                "AuctionPhaseEntered received while auction state is non-idle"
            );
            continue;
        }

        auction.phase = AuctionPhase::Selecting;

        let Some(card_id) = draw_fixture.as_ref().and_then(|fixture| fixture.card_id) else {
            tracing::error!(
                round = event.round,
                "AuctionPhaseEntered received before auction draw integration is available"
            );
            reset_to_idle(&mut auction);
            continue;
        };

        let starting_price = starting_price_for(card_id, &catalog, &config);
        auction.card_id = Some(card_id);
        auction.starting_price = starting_price;
        auction.current_price = starting_price;
        auction.current_leader = None;
        auction.timer_remaining_ms = config.auction_timer_seconds.saturating_mul(1000);
        auction.phase = AuctionPhase::LiveBidding;

        auction_cards.write(S2CAuctionCard {
            card_id,
            starting_price,
        });
    }
}

fn reset_to_idle(auction: &mut AuctionState) {
    auction.phase = AuctionPhase::Idle;
    auction.card_id = None;
    auction.starting_price = 0;
    auction.current_price = 0;
    auction.current_leader = None;
    auction.timer_remaining_ms = 0;
}

fn starting_price_for(card_id: CardId, catalog: &CardCatalog, config: &GameConfig) -> u32 {
    match catalog.cards.get(&card_id).map(|card| card.rarity) {
        Some(Rarity::Rare) => config.auction_floor_rare,
        Some(Rarity::Epic) => config.auction_floor_epic,
        Some(Rarity::Legendary) => config.auction_floor_legendary,
        Some(rarity) => {
            tracing::error!(
                card_id = ?card_id,
                rarity = ?rarity,
                "Auction draw fixture returned a card with non-auction rarity"
            );
            config.auction_floor_rare
        }
        None => {
            tracing::error!(
                card_id = ?card_id,
                "Auction draw fixture returned a card missing from CardCatalog"
            );
            config.auction_floor_rare
        }
    }
}
