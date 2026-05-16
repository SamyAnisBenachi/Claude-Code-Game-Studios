use std::collections::{HashMap, HashSet};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use lightyear::prelude::{
    MessageReceiver, NetworkTarget, PeerId, RemoteId, Server, ServerMultiMessageSender,
};
use shared::card::{CardId, ClassId, Rarity};
use shared::protocol::{
    BidRejectedReason, C2SPlaceBid, CardSource, DraftPhase, ReliableChannel, S2CAuctionBidAccepted,
    S2CAuctionBidRejected, S2CAuctionCard as ProtocolS2CAuctionCard, S2CAuctionSettled,
    S2CCardAcquired,
};
use shared::session::PlayerId;

use crate::core::economy::system::{gold_broadcast, S2CGoldBroadcast};
use crate::core::economy::{api, PlayerEconomies};
use crate::core::pool::PlayerPool;
use crate::core::rsm::{
    AbortAuction, AuctionPhaseEntered, AuctionSettled, DraftStarted, GameOverEmitted,
};
use crate::core::session::{
    defer_unicast_for_reconnect, DeferredMessage, PlayerConnectionMap, ReconnectTracker,
};
use crate::feature::acquisition::{hand_push, PlayerHands, MAX_HAND_SIZE};
use crate::foundation::config::{CardCatalog, GameConfig};
use crate::foundation::rng::ServerRng;

use super::state::{AuctionPhase, AuctionState};

/// Internal server queue item for auction draw observers.
///
/// The production Lightyear dispatch path sends the matching shared protocol
/// payload directly from `auction_tick_system`.
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

/// Shared neutral auction pool, separate from every player's personal shop pool.
#[derive(Resource)]
pub struct AuctionPool {
    pub pool: PlayerPool,
}

impl AuctionPool {
    pub fn from_catalog(catalog: &CardCatalog, config: &GameConfig) -> Self {
        Self {
            pool: PlayerPool::initialize(&catalog.cards, &config.0),
        }
    }

    pub fn copies_remaining(&self, card_id: CardId) -> u32 {
        self.pool.copies_remaining(card_id)
    }
}

impl Default for AuctionPool {
    fn default() -> Self {
        Self {
            pool: PlayerPool {
                copies_remaining: HashMap::new(),
                initial_count: HashMap::new(),
                shop_slots: Vec::new(),
            },
        }
    }
}

#[derive(SystemParam)]
pub struct AuctionMessageWriters<'w> {
    auction_cards: MessageWriter<'w, S2CAuctionCard>,
    gold_broadcasts: MessageWriter<'w, S2CGoldBroadcast>,
    settled: MessageWriter<'w, AuctionSettled>,
}

#[derive(SystemParam)]
pub struct AuctionStaticData<'w> {
    catalog: Res<'w, CardCatalog>,
    config: Res<'w, GameConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuctionDrawOutcome {
    Drawn(CardId),
    EmptyPool,
    MissingIntegration,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuctionCardAcquiredDispatch {
    pub player_id: PlayerId,
    pub peer_id: Option<PeerId>,
    pub message: S2CCardAcquired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuctionSettledDispatch {
    pub message: S2CAuctionSettled,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct AuctionNetworkOutbox {
    rejected: Vec<AuctionRejectionDispatch>,
    accepted: Vec<AuctionAcceptedDispatch>,
    card_acquired: Vec<AuctionCardAcquiredDispatch>,
    settled: Vec<AuctionSettledDispatch>,
}

impl AuctionNetworkOutbox {
    pub fn push_rejected(&mut self, dispatch: AuctionRejectionDispatch) {
        self.rejected.push(dispatch);
    }

    pub fn push_accepted(&mut self, dispatch: AuctionAcceptedDispatch) {
        self.accepted.push(dispatch);
    }

    pub fn push_card_acquired(&mut self, dispatch: AuctionCardAcquiredDispatch) {
        self.card_acquired.push(dispatch);
    }

    pub fn push_settled(&mut self, dispatch: AuctionSettledDispatch) {
        self.settled.push(dispatch);
    }

    pub fn rejected(&self) -> &[AuctionRejectionDispatch] {
        &self.rejected
    }

    pub fn accepted(&self) -> &[AuctionAcceptedDispatch] {
        &self.accepted
    }

    pub fn card_acquired(&self) -> &[AuctionCardAcquiredDispatch] {
        &self.card_acquired
    }

    pub fn settled(&self) -> &[AuctionSettledDispatch] {
        &self.settled
    }

    pub fn extend(&mut self, other: &AuctionNetworkOutbox) {
        self.rejected.extend(other.rejected.iter().cloned());
        self.accepted.extend(other.accepted.iter().cloned());
        self.card_acquired
            .extend(other.card_acquired.iter().cloned());
        self.settled.extend(other.settled.iter().cloned());
    }
}

pub fn auction_tick_system(
    mut auction: ResMut<AuctionState>,
    mut phase_entered: MessageReader<AuctionPhaseEntered>,
    mut abort: MessageReader<AbortAuction>,
    mut economies: ResMut<PlayerEconomies>,
    mut hands: Option<ResMut<PlayerHands>>,
    mut auction_pool: Option<ResMut<AuctionPool>>,
    draw_fixture: Option<Res<AuctionCardDrawFixture>>,
    data: AuctionStaticData,
    mut rng: Option<ResMut<ServerRng>>,
    time: Option<Res<Time>>,
    connections: Option<Res<PlayerConnectionMap>>,
    mut reconnect_tracker: Option<ResMut<ReconnectTracker>>,
    mut bid_receivers: Query<(&RemoteId, &mut MessageReceiver<C2SPlaceBid>)>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
    mut writers: AuctionMessageWriters,
) {
    for event in phase_entered.read() {
        tracing::info!(
            target: "server::game",
            round = event.round,
            phase = ?auction.phase,
            "auction_tick_system: AuctionPhaseEntered consumer enter"
        );
        if auction.phase != AuctionPhase::Idle {
            tracing::error!(
                target: "server::game",
                round = event.round,
                phase = ?auction.phase,
                "AuctionPhaseEntered received while auction state is non-idle"
            );
            continue;
        }

        tracing::info!(
            target: "server::game",
            round = event.round,
            from = ?AuctionPhase::Idle,
            to = ?AuctionPhase::Selecting,
            "auction_tick_system: state transition"
        );
        auction.phase = AuctionPhase::Selecting;

        let card_id = match select_auction_card(
            auction_pool.as_deref_mut(),
            draw_fixture.as_deref(),
            &data.catalog,
            &data.config,
            rng.as_deref_mut(),
            event.round,
        ) {
            AuctionDrawOutcome::Drawn(card_id) => card_id,
            AuctionDrawOutcome::EmptyPool => {
                writers.settled.write(AuctionSettled {
                    winner: None,
                    final_price: 0,
                    card_id: CardId(0),
                });
                reset_to_idle(&mut auction);
                continue;
            }
            AuctionDrawOutcome::MissingIntegration => {
                tracing::error!(
                    target: "server::game",
                    round = event.round,
                    "AuctionPhaseEntered received before auction draw integration is available"
                );
                reset_to_idle(&mut auction);
                continue;
            }
        };

        let starting_price = starting_price_for(card_id, &data.catalog, &data.config);
        auction.card_id = Some(card_id);
        auction.starting_price = starting_price;
        auction.current_price = starting_price;
        auction.current_leader = None;
        auction.timer_remaining_ms = data.config.auction_timer_seconds.saturating_mul(1000);
        tracing::info!(
            target: "server::game",
            round = event.round,
            card_id = ?card_id,
            starting_price,
            timer_ms = auction.timer_remaining_ms,
            from = ?AuctionPhase::Selecting,
            to = ?AuctionPhase::LiveBidding,
            "auction_tick_system: state transition"
        );
        auction.phase = AuctionPhase::LiveBidding;

        writers.auction_cards.write(S2CAuctionCard {
            card_id,
            starting_price,
        });

        if let (Ok(server), Some(sender)) = (server.single(), sender.as_mut()) {
            let message = ProtocolS2CAuctionCard {
                card_id,
                starting_price,
                timer_duration_ms: data.config.auction_timer_seconds.saturating_mul(1000),
            };
            tracing::info!(
                target: "server::game",
                round = event.round,
                card_id = ?card_id,
                starting_price,
                "auction_tick_system: broadcasting S2CAuctionCard enter"
            );
            if let Err(e) = sender.send::<ProtocolS2CAuctionCard, ReliableChannel>(
                &message,
                server,
                &NetworkTarget::All,
            ) {
                tracing::error!(
                    target: "server::game",
                    round = event.round,
                    card_id = ?card_id,
                    err = ?e,
                    "S2C send failed: type=S2CAuctionCard, handler=auction_tick_system"
                );
            }
        }
    }

    for _event in abort.read() {
        handle_abort_auction(&mut auction, &mut economies);
    }

    let bids = drain_bids(&mut bid_receivers, connections.as_deref());
    let mut frame_outbox = AuctionNetworkOutbox::default();
    let mut frame_gold_broadcasts = Vec::new();
    process_bid_batch(
        &mut auction,
        &mut economies,
        hands.as_deref(),
        &data.config,
        bids,
        &mut frame_outbox,
        &mut frame_gold_broadcasts,
    );

    if let Some(time) = time.as_deref() {
        let raw_delta_ms = u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX);
        decrement_live_bidding_timer(&mut auction, raw_delta_ms);
    }

    if let Some(settled) = settle_expired_auction(
        &mut auction,
        &mut economies,
        hands.as_deref_mut(),
        connections.as_deref(),
        &mut frame_outbox,
        &mut frame_gold_broadcasts,
    ) {
        writers.settled.write(settled);
    }

    for broadcast in frame_gold_broadcasts {
        writers.gold_broadcasts.write(broadcast);
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

pub fn initialize_auction_pool_on_draft_started(
    mut draft_started: MessageReader<DraftStarted>,
    catalog: Option<Res<CardCatalog>>,
    config: Option<Res<GameConfig>>,
    mut auction_pool: ResMut<AuctionPool>,
) {
    for message in draft_started.read() {
        if message.phase != DraftPhase::Initial {
            continue;
        }

        let (Some(catalog), Some(config)) = (catalog.as_deref(), config.as_deref()) else {
            tracing::warn!(
                target: "server::game",
                "DraftStarted::Initial received before CardCatalog or GameConfig; AuctionPool not initialized"
            );
            continue;
        };

        *auction_pool = AuctionPool::from_catalog(catalog, config);
    }
}

pub fn clear_auction_pool_on_game_over(
    mut game_over: MessageReader<GameOverEmitted>,
    mut auction_pool: ResMut<AuctionPool>,
) {
    if game_over.read().next().is_some() {
        *auction_pool = AuctionPool::default();
    }
}

fn select_auction_card(
    auction_pool: Option<&mut AuctionPool>,
    draw_fixture: Option<&AuctionCardDrawFixture>,
    catalog: &CardCatalog,
    config: &GameConfig,
    rng: Option<&mut ServerRng>,
    round: u32,
) -> AuctionDrawOutcome {
    if let Some(auction_pool) = auction_pool {
        return draw_from_auction_pool(auction_pool, catalog, config, rng, round);
    }

    draw_fixture
        .and_then(|fixture| fixture.card_id)
        .map(AuctionDrawOutcome::Drawn)
        .unwrap_or(AuctionDrawOutcome::MissingIntegration)
}

fn draw_from_auction_pool(
    auction_pool: &mut AuctionPool,
    catalog: &CardCatalog,
    config: &GameConfig,
    rng: Option<&mut ServerRng>,
    round: u32,
) -> AuctionDrawOutcome {
    let eligible_pool = build_eligible_auction_pool(&auction_pool.pool, catalog, config, round);
    let Some(seed) = auction_draw_seed(rng, round) else {
        return AuctionDrawOutcome::MissingIntegration;
    };
    let Some(card_id) = PlayerPool::draw_auction_card(&eligible_pool, &catalog.cards, seed) else {
        return AuctionDrawOutcome::EmptyPool;
    };

    if let Err(error) = auction_pool.pool.distribute(card_id) {
        tracing::error!(
            target: "server::game",
            card_id = card_id.0,
            ?error,
            "auction draw selected a card that could not be distributed"
        );
        return AuctionDrawOutcome::EmptyPool;
    }

    AuctionDrawOutcome::Drawn(card_id)
}

fn auction_draw_seed(rng: Option<&mut ServerRng>, round: u32) -> Option<u64> {
    if let Some(rng) = rng {
        return Some(rng.draw_auction_card(round));
    }

    tracing::error!(
        target: "server::game",
        round,
        "AuctionPhaseEntered processed without ServerRng; auction draw skipped"
    );
    None
}

fn build_eligible_auction_pool(
    auction_pool: &PlayerPool,
    catalog: &CardCatalog,
    config: &GameConfig,
    round: u32,
) -> PlayerPool {
    let copies_remaining = auction_pool
        .copies_remaining
        .iter()
        .filter_map(|(card_id, remaining)| {
            auction_card_is_round_eligible(*card_id, catalog, config, round)
                .then_some((*card_id, *remaining))
        })
        .collect();
    let initial_count = auction_pool
        .initial_count
        .iter()
        .filter_map(|(card_id, initial)| {
            auction_card_is_round_eligible(*card_id, catalog, config, round)
                .then_some((*card_id, *initial))
        })
        .collect();

    PlayerPool {
        copies_remaining,
        initial_count,
        shop_slots: Vec::new(),
    }
}

fn auction_card_is_round_eligible(
    card_id: CardId,
    catalog: &CardCatalog,
    config: &GameConfig,
    round: u32,
) -> bool {
    let Some(card) = catalog.cards.get(&card_id) else {
        return false;
    };

    if card.class != ClassId::Neutral {
        return false;
    }

    match card.rarity {
        Rarity::Rare => true,
        Rarity::Legendary => round >= config.legendary_pool_entry_round,
        _ => false,
    }
}

pub fn process_bid_batch(
    auction: &mut AuctionState,
    economies: &mut PlayerEconomies,
    hands: Option<&PlayerHands>,
    config: &GameConfig,
    bids: impl IntoIterator<Item = AuctionBid>,
    outbox: &mut AuctionNetworkOutbox,
    gold_broadcasts: &mut Vec<S2CGoldBroadcast>,
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
                target: "server::game",
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

        accept_bid(auction, economies, config, bid, outbox, gold_broadcasts);
    }
}

pub fn decrement_live_bidding_timer(auction: &mut AuctionState, raw_delta_ms: u32) {
    if auction.phase != AuctionPhase::LiveBidding {
        return;
    }

    // No per-tick clamp: tracks raw wall-clock delta so the auction expires in
    // bounded real time even when the Update schedule fires sparsely (e.g.,
    // headless server with network-driven ticks). saturating_sub handles
    // oversized deltas by zeroing the timer, which is the correct behavior —
    // if real time exceeds the configured auction window, settlement must
    // fire. A prior `.min(1000)` clamp here caused 17-minute stuck auctions
    // when Update ticks were tens of seconds apart between bursts.
    auction.timer_remaining_ms = auction.timer_remaining_ms.saturating_sub(raw_delta_ms);
}

pub fn settle_expired_auction(
    auction: &mut AuctionState,
    economies: &mut PlayerEconomies,
    hands: Option<&mut PlayerHands>,
    connections: Option<&PlayerConnectionMap>,
    outbox: &mut AuctionNetworkOutbox,
    gold_broadcasts: &mut Vec<S2CGoldBroadcast>,
) -> Option<AuctionSettled> {
    if auction.timer_remaining_ms != 0 {
        return None;
    }

    match auction.phase {
        AuctionPhase::LiveBidding => {
            tracing::info!(
                target: "server::game",
                final_price = auction.current_price,
                leader = ?auction.current_leader,
                from = ?AuctionPhase::LiveBidding,
                to = ?AuctionPhase::Resolving,
                "settle_expired_auction: state transition"
            );
            auction.phase = AuctionPhase::Resolving;
        }
        AuctionPhase::Resolving => {}
        AuctionPhase::Idle | AuctionPhase::Selecting => {
            return None;
        }
    }

    let card_id = auction.card_id.unwrap_or_else(|| {
        tracing::error!(target: "server::game", "auction settlement reached without an auction card");
        CardId(0)
    });

    let settled = match auction.current_leader {
        Some(winner) => {
            let bid_amount = auction.current_price;
            if let Some(economy) = economies.0.get_mut(&winner) {
                settle_winner_economy(winner, economy, bid_amount);
                gold_broadcasts.push(gold_broadcast(winner, economy));
            } else {
                tracing::error!(
                    target: "server::game",
                    winner = winner.0,
                    bid_amount,
                    "auction settlement winner economy missing"
                );
            }

            award_auction_card(winner, card_id, hands, connections, outbox);
            outbox.push_settled(AuctionSettledDispatch {
                message: S2CAuctionSettled {
                    winner: Some(winner),
                    amount: bid_amount,
                },
            });

            AuctionSettled {
                winner: Some(winner),
                final_price: bid_amount,
                card_id,
            }
        }
        None => {
            outbox.push_settled(AuctionSettledDispatch {
                message: S2CAuctionSettled {
                    winner: None,
                    amount: 0,
                },
            });

            AuctionSettled {
                winner: None,
                final_price: 0,
                card_id,
            }
        }
    };

    reset_to_idle(auction);
    Some(settled)
}

fn settle_winner_economy(
    winner: PlayerId,
    economy: &mut crate::core::economy::PlayerEconomy,
    bid_amount: u32,
) {
    if economy.gold < economy.reserved_gold {
        tracing::error!(
            target: "server::game",
            winner = winner.0,
            gold = economy.gold,
            reserved_gold = economy.reserved_gold,
            "CRITICAL: gold < reserved_gold at auction resolution; session corrupt"
        );
    }
    debug_assert!(
        economy.gold >= economy.reserved_gold,
        "gold invariant violated at auction resolution"
    );

    if economy.reserved_gold != bid_amount {
        tracing::error!(
            target: "server::game",
            winner = winner.0,
            bid_amount,
            reserved_gold = economy.reserved_gold,
            "CRITICAL: auction resolution reservation does not match winning bid"
        );
    }

    let reserved_gold = economy.reserved_gold;
    api::release_gold_reservation(economy, reserved_gold);
    if api::spend_gold(economy, bid_amount).is_err() {
        tracing::error!(
            target: "server::game",
            winner = winner.0,
            bid_amount,
            "CRITICAL: auction settlement spend failed after reservation release"
        );
    }
}

fn award_auction_card(
    winner: PlayerId,
    card_id: CardId,
    hands: Option<&mut PlayerHands>,
    connections: Option<&PlayerConnectionMap>,
    outbox: &mut AuctionNetworkOutbox,
) {
    let Some(hands) = hands else {
        tracing::error!(
            target: "server::game",
            winner = winner.0,
            card_id = card_id.0,
            "auction settlement could not award card because PlayerHands is missing"
        );
        return;
    };

    if hand_push(hands, winner, card_id).is_err() {
        tracing::error!(
            target: "server::game",
            winner = winner.0,
            card_id = card_id.0,
            "auction settlement winner hand full; card discarded"
        );
        return;
    }

    outbox.push_card_acquired(AuctionCardAcquiredDispatch {
        player_id: winner,
        peer_id: peer_for_player(connections, winner),
        message: S2CCardAcquired {
            card_id,
            source: CardSource::AuctionWon,
        },
    });
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
    gold_broadcasts: &mut Vec<S2CGoldBroadcast>,
) {
    let previous_bid_amount = auction.current_price;
    if let Some(previous_leader) = auction.current_leader {
        if let Some(economy) = economies.0.get_mut(&previous_leader) {
            api::release_gold_reservation(economy, previous_bid_amount);
            gold_broadcasts.push(gold_broadcast(previous_leader, economy));
        }
    }

    let Some(economy) = economies.0.get_mut(&bid.bidder) else {
        tracing::debug!(
            target: "server::game",
            bidder = bid.bidder.0,
            amount = bid.amount,
            "auction bid acceptance skipped because bidder economy is missing"
        );
        return;
    };

    if api::reserve_gold(economy, bid.amount).is_err() {
        tracing::error!(
            target: "server::game",
            bidder = bid.bidder.0,
            amount = bid.amount,
            "auction bid passed validation but reservation failed"
        );
        return;
    }
    gold_broadcasts.push(gold_broadcast(bid.bidder, economy));

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
            tracing::info!(
                target: "server::game",
                peer_id = ?remote.0,
                amount = bid.amount,
                "c2s_place_bid: recv"
            );
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

        tracing::info!(
            target: "server::game",
            player_id = dispatch.player_id.0,
            peer_id = ?dispatch.peer_id,
            reason = ?dispatch.message.reason,
            "send_outbox_dispatches: dispatching S2CAuctionBidRejected enter"
        );

        let Some(peer_id) = dispatch.peer_id else {
            tracing::warn!(
                target: "server::game",
                player_id = dispatch.player_id.0,
                "send_outbox_dispatches: S2CAuctionBidRejected DROPPED — peer_id unresolved; player not in PlayerConnectionMap or stale entry"
            );
            continue;
        };
        if let Err(e) = sender.send::<S2CAuctionBidRejected, ReliableChannel>(
            &dispatch.message,
            server,
            &NetworkTarget::Single(peer_id),
        ) {
            tracing::error!(
                target: "server::game",
                player_id = dispatch.player_id.0,
                peer_id = ?peer_id,
                err = ?e,
                "S2C send failed: type=S2CAuctionBidRejected, handler=send_outbox_dispatches"
            );
        }
    }

    for dispatch in outbox.accepted() {
        tracing::info!(
            target: "server::game",
            bidder = dispatch.message.bidder.0,
            amount = dispatch.message.amount,
            "send_outbox_dispatches: dispatching S2CAuctionBidAccepted enter"
        );
        let target = accepted_bid_target(connections, pending_players);
        let Some(target) = target else {
            tracing::warn!(
                target: "server::game",
                bidder = dispatch.message.bidder.0,
                "send_outbox_dispatches: S2CAuctionBidAccepted DROPPED — no broadcast target (all players pending reconnect)"
            );
            continue;
        };
        if let Err(e) = sender.send::<S2CAuctionBidAccepted, ReliableChannel>(
            &dispatch.message,
            server,
            &target,
        ) {
            tracing::error!(
                target: "server::game",
                bidder = dispatch.message.bidder.0,
                amount = dispatch.message.amount,
                err = ?e,
                "S2C send failed: type=S2CAuctionBidAccepted, handler=send_outbox_dispatches"
            );
        }
    }

    for dispatch in outbox.card_acquired() {
        if pending_players.contains(&dispatch.player_id) {
            continue;
        }

        tracing::info!(
            target: "server::game",
            player_id = dispatch.player_id.0,
            peer_id = ?dispatch.peer_id,
            card_id = ?dispatch.message.card_id,
            source = ?dispatch.message.source,
            "send_outbox_dispatches: dispatching S2CCardAcquired enter"
        );

        let Some(peer_id) = dispatch.peer_id else {
            tracing::warn!(
                target: "server::game",
                player_id = dispatch.player_id.0,
                card_id = ?dispatch.message.card_id,
                "send_outbox_dispatches: S2CCardAcquired DROPPED — peer_id unresolved; player not in PlayerConnectionMap or stale entry"
            );
            continue;
        };
        if let Err(e) = sender.send::<S2CCardAcquired, ReliableChannel>(
            &dispatch.message,
            server,
            &NetworkTarget::Single(peer_id),
        ) {
            tracing::error!(
                target: "server::game",
                player_id = dispatch.player_id.0,
                peer_id = ?peer_id,
                card_id = ?dispatch.message.card_id,
                err = ?e,
                "S2C send failed: type=S2CCardAcquired, handler=send_outbox_dispatches"
            );
        }
    }

    for dispatch in outbox.settled() {
        tracing::info!(
            target: "server::game",
            winner = ?dispatch.message.winner,
            amount = dispatch.message.amount,
            "send_outbox_dispatches: dispatching S2CAuctionSettled enter"
        );
        let target = accepted_bid_target(connections, pending_players);
        let Some(target) = target else {
            tracing::warn!(
                target: "server::game",
                winner = ?dispatch.message.winner,
                "send_outbox_dispatches: S2CAuctionSettled DROPPED — no broadcast target (all players pending reconnect)"
            );
            continue;
        };
        if let Err(e) =
            sender.send::<S2CAuctionSettled, ReliableChannel>(&dispatch.message, server, &target)
        {
            tracing::error!(
                target: "server::game",
                winner = ?dispatch.message.winner,
                amount = dispatch.message.amount,
                err = ?e,
                "S2C send failed: type=S2CAuctionSettled, handler=send_outbox_dispatches"
            );
        }
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

    for dispatch in outbox.card_acquired() {
        let _ = defer_unicast_for_reconnect(
            tracker.as_deref_mut(),
            dispatch.player_id,
            DeferredMessage::CardAcquired {
                card_id: dispatch.message.card_id,
                source: dispatch.message.source,
            },
        );
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

fn peer_for_player(
    connections: Option<&PlayerConnectionMap>,
    player_id: PlayerId,
) -> Option<PeerId> {
    connections?
        .0
        .iter()
        .find_map(|(peer_id, mapped_player)| (*mapped_player == player_id).then_some(*peer_id))
}

fn reset_to_idle(auction: &mut AuctionState) {
    tracing::info!(
        target: "server::game",
        from = ?auction.phase,
        to = ?AuctionPhase::Idle,
        card_id = ?auction.card_id,
        final_price = auction.current_price,
        leader = ?auction.current_leader,
        "reset_to_idle: state transition"
    );
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
                target: "server::game",
                card_id = ?card_id,
                rarity = ?rarity,
                "Auction draw fixture returned a card with non-auction rarity"
            );
            config.auction_floor_rare
        }
        None => {
            tracing::error!(
                target: "server::game",
                card_id = ?card_id,
                "Auction draw fixture returned a card missing from CardCatalog"
            );
            config.auction_floor_rare
        }
    }
}
