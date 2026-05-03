use std::collections::HashSet;

use bevy::prelude::*;
use lightyear::prelude::{
    MessageReceiver, NetworkTarget, PeerId, RemoteId, Server, ServerMultiMessageSender,
};
use shared::card::{CardId, Rarity};
use shared::protocol::{
    BidRejectedReason, C2SPlaceBid, ReliableChannel, S2CAuctionBidAccepted, S2CAuctionBidRejected,
};
use shared::session::PlayerId;

use crate::core::economy::{api, PlayerEconomies};
use crate::core::rsm::{AbortAuction, AuctionPhaseEntered};
use crate::core::session::{
    defer_unicast_for_reconnect, DeferredMessage, PlayerConnectionMap, ReconnectTracker,
};
use crate::feature::acquisition::{PlayerHands, MAX_HAND_SIZE};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuctionBid {
    pub bidder: PlayerId,
    pub peer_id: Option<PeerId>,
    pub amount: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuctionRejectionDispatch {
    pub player_id: PlayerId,
    pub peer_id: Option<PeerId>,
    pub message: S2CAuctionBidRejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuctionAcceptedDispatch {
    pub player_id: PlayerId,
    pub peer_id: Option<PeerId>,
    pub message: S2CAuctionBidAccepted,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct AuctionNetworkOutbox {
    rejected: Vec<AuctionRejectionDispatch>,
    accepted: Vec<AuctionAcceptedDispatch>,
}

impl AuctionNetworkOutbox {
    pub fn push_rejected(&mut self, dispatch: AuctionRejectionDispatch) {
        self.rejected.push(dispatch);
    }

    pub fn push_accepted(&mut self, dispatch: AuctionAcceptedDispatch) {
        self.accepted.push(dispatch);
    }

    pub fn rejected(&self) -> &[AuctionRejectionDispatch] {
        &self.rejected
    }

    pub fn accepted(&self) -> &[AuctionAcceptedDispatch] {
        &self.accepted
    }

    pub fn extend(&mut self, other: &AuctionNetworkOutbox) {
        self.rejected.extend(other.rejected.iter().cloned());
        self.accepted.extend(other.accepted.iter().cloned());
    }
}

pub fn auction_tick_system(
    mut auction: ResMut<AuctionState>,
    mut phase_entered: MessageReader<AuctionPhaseEntered>,
    mut abort: MessageReader<AbortAuction>,
    mut economies: ResMut<PlayerEconomies>,
    hands: Option<Res<PlayerHands>>,
    draw_fixture: Option<Res<AuctionCardDrawFixture>>,
    catalog: Res<CardCatalog>,
    config: Res<GameConfig>,
    connections: Option<Res<PlayerConnectionMap>>,
    mut reconnect_tracker: Option<ResMut<ReconnectTracker>>,
    mut bid_receivers: Query<(&RemoteId, &mut MessageReceiver<C2SPlaceBid>)>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
    mut network_outbox: Option<ResMut<AuctionNetworkOutbox>>,
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

    for _event in abort.read() {
        handle_abort_auction(&mut auction, &mut economies);
    }

    let bids = drain_bids(&mut bid_receivers, connections.as_deref());
    let mut frame_outbox = AuctionNetworkOutbox::default();
    process_bid_batch(
        &mut auction,
        &mut economies,
        hands.as_deref(),
        &config,
        bids,
        &mut frame_outbox,
    );

    if let Some(outbox) = network_outbox.as_deref_mut() {
        outbox.extend(&frame_outbox);
    }

    let pending_players = defer_auction_outbox_for_reconnect(
        &frame_outbox,
        connections.as_deref(),
        reconnect_tracker.as_deref_mut(),
    );

    if let (Some(server), Some(sender)) = (server.single().ok(), sender.as_mut()) {
        send_outbox_dispatches(
            sender,
            server,
            &frame_outbox,
            connections.as_deref(),
            &pending_players,
        );
    }
}

pub fn process_bid_batch(
    auction: &mut AuctionState,
    economies: &mut PlayerEconomies,
    hands: Option<&PlayerHands>,
    config: &GameConfig,
    bids: impl IntoIterator<Item = AuctionBid>,
    outbox: &mut AuctionNetworkOutbox,
) {
    for bid in bids {
        if auction.phase != AuctionPhase::LiveBidding {
            continue;
        }

        if auction.timer_remaining_ms == 0 {
            reject_bid(outbox, bid, BidRejectedReason::AuctionExpired);
            continue;
        }

        if bid.amount < auction.current_price.saturating_add(1) {
            reject_bid(outbox, bid, BidRejectedReason::AmountTooLow);
            continue;
        }

        if auction.current_leader == Some(bid.bidder) {
            reject_bid(outbox, bid, BidRejectedReason::AlreadyLeader);
            continue;
        }

        let Some(economy) = economies.0.get(&bid.bidder) else {
            tracing::debug!(
                bidder = bid.bidder.0,
                amount = bid.amount,
                "auction bid discarded because bidder economy is missing"
            );
            continue;
        };
        if !api::can_afford_bid(economy, bid.amount) {
            reject_bid(outbox, bid, BidRejectedReason::InsufficientGold);
            continue;
        }

        if hands
            .map(|hands| hands.hand_len(bid.bidder) >= MAX_HAND_SIZE)
            .unwrap_or(false)
        {
            reject_bid(outbox, bid, BidRejectedReason::HandFull);
            continue;
        }

        accept_bid(auction, economies, config, bid, outbox);
    }
}

fn handle_abort_auction(auction: &mut AuctionState, economies: &mut PlayerEconomies) {
    match auction.phase {
        AuctionPhase::Idle | AuctionPhase::Resolving => {}
        AuctionPhase::Selecting | AuctionPhase::LiveBidding => {
            if let Some(leader) = auction.current_leader {
                if let Some(economy) = economies.0.get_mut(&leader) {
                    let reserved_gold = economy.reserved_gold;
                    if reserved_gold > 0 {
                        api::release_gold_reservation(economy, reserved_gold);
                    }
                }
            }

            reset_to_idle(auction);
        }
    }
}

fn accept_bid(
    auction: &mut AuctionState,
    economies: &mut PlayerEconomies,
    config: &GameConfig,
    bid: AuctionBid,
    outbox: &mut AuctionNetworkOutbox,
) {
    if let Some(previous_leader) = auction.current_leader {
        if let Some(economy) = economies.0.get_mut(&previous_leader) {
            let reserved_gold = economy.reserved_gold;
            if reserved_gold > 0 {
                api::release_gold_reservation(economy, reserved_gold);
            }
        }
    }

    let Some(economy) = economies.0.get_mut(&bid.bidder) else {
        tracing::debug!(
            bidder = bid.bidder.0,
            amount = bid.amount,
            "auction bid acceptance skipped because bidder economy is missing"
        );
        return;
    };

    if api::reserve_gold(economy, bid.amount).is_err() {
        tracing::error!(
            bidder = bid.bidder.0,
            amount = bid.amount,
            "auction bid passed validation but reservation failed"
        );
        return;
    }

    auction.current_price = bid.amount;
    auction.current_leader = Some(bid.bidder);
    auction.timer_remaining_ms = auction
        .timer_remaining_ms
        .saturating_add(config.auction_timer_reset_seconds.saturating_mul(1000))
        .min(config.auction_timer_seconds.saturating_mul(1000));

    outbox.push_accepted(AuctionAcceptedDispatch {
        player_id: bid.bidder,
        peer_id: bid.peer_id,
        message: S2CAuctionBidAccepted {
            bidder: bid.bidder,
            amount: bid.amount,
            new_timer_ms: auction.timer_remaining_ms,
        },
    });
}

fn reject_bid(outbox: &mut AuctionNetworkOutbox, bid: AuctionBid, reason: BidRejectedReason) {
    outbox.push_rejected(AuctionRejectionDispatch {
        player_id: bid.bidder,
        peer_id: bid.peer_id,
        message: S2CAuctionBidRejected { reason },
    });
}

fn drain_bids(
    bid_receivers: &mut Query<(&RemoteId, &mut MessageReceiver<C2SPlaceBid>)>,
    connections: Option<&PlayerConnectionMap>,
) -> Vec<AuctionBid> {
    let mut bids = Vec::new();
    for (remote, mut receiver) in bid_receivers.iter_mut() {
        for bid in receiver.receive() {
            let Some(player_id) =
                connections.and_then(|connections| connections.0.get(&remote.0).copied())
            else {
                continue;
            };
            bids.push(AuctionBid {
                bidder: player_id,
                peer_id: Some(remote.0),
                amount: bid.amount,
            });
        }
    }
    bids
}

fn send_outbox_dispatches(
    sender: &mut ServerMultiMessageSender,
    server: &Server,
    outbox: &AuctionNetworkOutbox,
    connections: Option<&PlayerConnectionMap>,
    pending_players: &HashSet<PlayerId>,
) {
    for dispatch in outbox.rejected() {
        if pending_players.contains(&dispatch.player_id) {
            continue;
        }

        let Some(peer_id) = dispatch.peer_id else {
            continue;
        };
        let _ = sender.send::<S2CAuctionBidRejected, ReliableChannel>(
            &dispatch.message,
            server,
            &NetworkTarget::Single(peer_id),
        );
    }

    for dispatch in outbox.accepted() {
        let target = accepted_bid_target(connections, pending_players);
        let Some(target) = target else {
            continue;
        };
        let _ = sender.send::<S2CAuctionBidAccepted, ReliableChannel>(
            &dispatch.message,
            server,
            &target,
        );
    }
}

pub fn defer_auction_outbox_for_reconnect(
    outbox: &AuctionNetworkOutbox,
    connections: Option<&PlayerConnectionMap>,
    mut tracker: Option<&mut ReconnectTracker>,
) -> HashSet<PlayerId> {
    let pending_players = pending_reconnect_players(tracker.as_deref(), connections);

    for dispatch in outbox.rejected() {
        let _ = defer_unicast_for_reconnect(
            tracker.as_deref_mut(),
            dispatch.player_id,
            DeferredMessage::AuctionBidRejected(dispatch.message.clone()),
        );
    }

    for dispatch in outbox.accepted() {
        for player in &pending_players {
            let _ = defer_unicast_for_reconnect(
                tracker.as_deref_mut(),
                *player,
                DeferredMessage::AuctionBidAccepted(dispatch.message.clone()),
            );
        }
    }

    pending_players
}

fn pending_reconnect_players(
    tracker: Option<&ReconnectTracker>,
    connections: Option<&PlayerConnectionMap>,
) -> HashSet<PlayerId> {
    let Some(tracker) = tracker else {
        return HashSet::new();
    };

    tracker
        .snapshot_sent
        .iter()
        .filter_map(|(player, sent)| {
            (!*sent
                && connections
                    .map(|connections| connections.0.values().any(|mapped| mapped == player))
                    .unwrap_or(true))
            .then_some(*player)
        })
        .collect()
}

fn accepted_bid_target(
    connections: Option<&PlayerConnectionMap>,
    pending_players: &HashSet<PlayerId>,
) -> Option<NetworkTarget> {
    if pending_players.is_empty() {
        return Some(NetworkTarget::All);
    }

    let connections = connections?;
    let peers = connections
        .0
        .iter()
        .filter_map(|(peer_id, player)| (!pending_players.contains(player)).then_some(*peer_id))
        .collect::<Vec<_>>();
    (!peers.is_empty()).then_some(NetworkTarget::Only(peers))
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
