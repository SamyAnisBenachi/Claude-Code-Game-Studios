use std::collections::HashMap;

use bevy::prelude::*;
use lightyear::prelude::{
    MessageReceiver, NetworkTarget, PeerId, RemoteId, Server, ServerMultiMessageSender,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use shared::card::{CardId, ClassId};
use shared::protocol::{
    C2SPurchaseCard, C2SRefreshShop, CardSource, ReliableChannel, S2CCardAcquired,
    S2CDraftOffering, S2CGoldUpdate as ProtocolGoldUpdate, S2CShopSlots,
};
use shared::session::PlayerId;

use crate::core::economy::{
    api as economy_api, PlayerEconomies, S2CGoldUpdate as EconomyGoldUpdate,
};
use crate::core::pool::{DistributeError, PlayerPool, PlayerPools};
use crate::core::session::{
    defer_unicast_for_reconnect, DeferredMessage, PlayerConnectionMap, PlayerSessions,
    ReconnectTracker,
};
use crate::foundation::config::{CardCatalog, GameConfig};
use crate::foundation::rng::ServerRng;

use super::hands::{PlayerHands, MAX_HAND_SIZE};
use super::messages::{ShopRefreshTrigger, ShopRefreshTriggered};
use super::state::{PlayerShopState, ShopPhase, ShopStates, SHOP_SLOT_COUNT};

pub const DRAFT_INITIAL_OFFERING_COUNT: u8 = 9;
pub const SHOP_DEDUP_RETRY_LIMIT: usize = 20;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardAcquisitionSet {
    Tick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PurchaseAttemptResult {
    Purchased,
    DiscardedWrongPhase,
    HandFull,
    CardNotDisplayed,
    CardUnavailable,
    CardMissingFromCatalog,
    PlayerEconomyMissing,
    PlayerPoolMissing,
    InsufficientGold,
    DistributeExhausted,
    DistributeUnknownCard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshAttemptResult {
    Refreshed,
    AcceptedOutOfScope,
    DiscardedWrongPhase,
    PlayerEconomyMissing,
    InsufficientGold,
    DrawUnavailable,
}

#[derive(Clone, Debug)]
pub struct DraftOfferingDispatch {
    pub player_id: PlayerId,
    pub peer_id: Option<PeerId>,
    pub message: S2CDraftOffering,
}

#[derive(Clone, Debug)]
pub struct ShopSlotsDispatch {
    pub player_id: PlayerId,
    pub peer_id: Option<PeerId>,
    pub message: S2CShopSlots,
}

#[derive(Clone, Debug)]
pub struct CardAcquiredDispatch {
    pub player_id: PlayerId,
    pub peer_id: Option<PeerId>,
    pub message: S2CCardAcquired,
}

#[derive(Clone, Debug)]
pub struct PurchaseNetworkEvents {
    pub player_id: PlayerId,
    pub card_acquired: CardAcquiredDispatch,
    pub shop_slots: Option<ShopSlotsDispatch>,
    pub gold_update: EconomyGoldUpdate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShopSlotType {
    Class,
    Neutral,
}

pub trait PurchasePool {
    fn is_available(&self, card_id: CardId) -> bool;
    fn distribute(&mut self, card_id: CardId) -> Result<(), DistributeError>;
}

impl PurchasePool for PlayerPool {
    fn is_available(&self, card_id: CardId) -> bool {
        PlayerPool::is_available(self, card_id)
    }

    fn distribute(&mut self, card_id: CardId) -> Result<(), DistributeError> {
        PlayerPool::distribute(self, card_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PurchaseDisplay {
    DraftInitialOffering,
    ShopSlot(usize),
}

#[allow(clippy::too_many_arguments)]
pub fn card_acquisition_tick_system(
    mut shop_states: ResMut<ShopStates>,
    mut hands: ResMut<PlayerHands>,
    mut economies: Option<ResMut<PlayerEconomies>>,
    mut pools: Option<ResMut<PlayerPools>>,
    sessions: Option<Res<PlayerSessions>>,
    mut server_rng: Option<ResMut<ServerRng>>,
    catalog: Option<Res<CardCatalog>>,
    config: Option<Res<GameConfig>>,
    connections: Option<Res<PlayerConnectionMap>>,
    mut reconnect_tracker: Option<ResMut<ReconnectTracker>>,
    mut shop_refreshes: MessageReader<ShopRefreshTriggered>,
    mut c2s_receivers: ParamSet<(
        Query<(&RemoteId, &mut MessageReceiver<C2SRefreshShop>)>,
        Query<(&RemoteId, &mut MessageReceiver<C2SPurchaseCard>)>,
    )>,
    mut gold_updates: MessageWriter<EconomyGoldUpdate>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
    mut pending_draft_offerings: Local<Vec<DraftOfferingDispatch>>,
) {
    let server = server.single().ok();

    // Retry any draft offerings that could not be sent in a previous tick because
    // ServerMultiMessageSender or the Server resource was not yet initialized.
    if !pending_draft_offerings.is_empty() {
        match (&server, sender.as_mut()) {
            (None, _) => {
                tracing::warn!(
                    count = pending_draft_offerings.len(),
                    "card_acquisition: server resource not yet available \
                     — retaining {} pending draft offering(s) for next tick",
                    pending_draft_offerings.len()
                );
            }
            (_, None) => {
                tracing::warn!(
                    count = pending_draft_offerings.len(),
                    "card_acquisition: ServerMultiMessageSender not yet initialized \
                     — retaining {} pending draft offering(s) for next tick",
                    pending_draft_offerings.len()
                );
            }
            (Some(server), Some(sender)) => {
                let mut sent_count: usize = 0;
                // drain_filter is unstable; retain-then-send pattern keeps allocations minimal.
                let to_send = std::mem::take(&mut *pending_draft_offerings);
                for dispatch in to_send {
                    if !defer_draft_offering(reconnect_tracker.as_deref_mut(), &dispatch) {
                        send_draft_offering(sender, server, &dispatch);
                        sent_count += 1;
                    }
                }
                if sent_count > 0 {
                    tracing::info!(
                        "card_acquisition: broadcast S2CDraftOffering to {} client(s) \
                         (retry from pending queue)",
                        sent_count
                    );
                }
            }
        }
    }

    for refresh in shop_refreshes.read() {
        if refresh.trigger == ShopRefreshTrigger::DraftInitial {
            let dispatch = match (
                pools.as_deref(),
                sessions.as_deref(),
                catalog.as_deref(),
                server_rng.as_deref_mut(),
            ) {
                (Some(pools), Some(sessions), Some(catalog), Some(server_rng)) => {
                    let seed = server_rng.draw_initial_draft(rng_player_id(refresh.player_id));
                    build_draft_initial_offering(
                        &mut shop_states,
                        pools,
                        sessions,
                        catalog,
                        refresh.player_id,
                        seed,
                    )
                    .map(|message| {
                        prepare_draft_offering_dispatch(
                            refresh.player_id,
                            message,
                            connections.as_deref(),
                        )
                    })
                }
                _ => {
                    apply_shop_refresh_trigger(&mut shop_states, *refresh);
                    None
                }
            };

            if let Some(dispatch) = dispatch {
                if !defer_draft_offering(reconnect_tracker.as_deref_mut(), &dispatch) {
                    match (&server, sender.as_mut()) {
                        (None, _) => {
                            tracing::warn!(
                                player_id = dispatch.player_id.0,
                                "card_acquisition: server resource not yet available \
                                 — queuing S2CDraftOffering for retry next tick"
                            );
                            pending_draft_offerings.push(dispatch);
                        }
                        (_, None) => {
                            tracing::warn!(
                                player_id = dispatch.player_id.0,
                                "card_acquisition: ServerMultiMessageSender not yet initialized \
                                 — queuing S2CDraftOffering for retry next tick"
                            );
                            pending_draft_offerings.push(dispatch);
                        }
                        (Some(server), Some(sender)) => {
                            send_draft_offering(sender, server, &dispatch);
                            tracing::info!(
                                player_id = dispatch.player_id.0,
                                "card_acquisition: broadcast S2CDraftOffering to 1 client"
                            );
                        }
                    }
                }
            }
            continue;
        }

        if matches!(
            refresh.trigger,
            ShopRefreshTrigger::AuctionLock | ShopRefreshTrigger::ShopOpen
        ) {
            let dispatch = match (
                pools.as_deref(),
                sessions.as_deref(),
                catalog.as_deref(),
                config.as_deref(),
                server_rng.as_deref_mut(),
            ) {
                (Some(pools), Some(sessions), Some(catalog), Some(config), Some(server_rng)) => {
                    build_auto_shop_slots(
                        &mut shop_states,
                        pools,
                        sessions,
                        catalog,
                        config,
                        server_rng,
                        refresh.player_id,
                        refresh.trigger,
                    )
                    .map(|message| {
                        prepare_shop_slots_dispatch(
                            refresh.player_id,
                            message,
                            connections.as_deref(),
                        )
                    })
                }
                _ => {
                    apply_shop_refresh_trigger(&mut shop_states, *refresh);
                    None
                }
            };

            if let Some(dispatch) = dispatch.as_ref() {
                if !defer_shop_slots(reconnect_tracker.as_deref_mut(), dispatch) {
                    if let (Some(server), Some(sender)) = (server, sender.as_mut()) {
                        send_shop_slots(sender, server, dispatch);
                    }
                }
            }
            continue;
        }

        apply_shop_refresh_trigger(&mut shop_states, *refresh);
    }

    let connections = connections.as_deref();
    for (remote, mut receiver) in c2s_receivers.p0().iter_mut() {
        for _message in receiver.receive() {
            let Some(player_id) = resolve_player(remote, connections) else {
                continue;
            };

            let dispatch = match (
                economies.as_mut(),
                pools.as_deref(),
                sessions.as_deref(),
                catalog.as_deref(),
                config.as_deref(),
                server_rng.as_deref_mut(),
            ) {
                (
                    Some(economies),
                    Some(pools),
                    Some(sessions),
                    Some(catalog),
                    Some(config),
                    Some(server_rng),
                ) => {
                    let (result, message) = process_manual_refresh_shop_request(
                        &mut shop_states,
                        &mut *economies,
                        pools,
                        sessions,
                        catalog,
                        config,
                        server_rng,
                        player_id,
                    );
                    if result == RefreshAttemptResult::Refreshed {
                        emit_economy_update_for_player(
                            &*economies,
                            player_id,
                            reconnect_tracker.as_deref_mut(),
                            &mut gold_updates,
                        );
                    }
                    message
                        .map(|message| prepare_shop_slots_dispatch(player_id, message, connections))
                }
                _ => {
                    let _ = process_refresh_shop_request(&mut shop_states, player_id);
                    None
                }
            };

            if let Some(dispatch) = dispatch.as_ref() {
                if !defer_shop_slots(reconnect_tracker.as_deref_mut(), dispatch) {
                    if let (Some(server), Some(sender)) = (server, sender.as_mut()) {
                        send_shop_slots(sender, server, dispatch);
                    }
                }
            }
        }
    }

    let catalog = catalog.as_deref();
    for (remote, mut receiver) in c2s_receivers.p1().iter_mut() {
        for message in receiver.receive() {
            let Some(player_id) = resolve_player(remote, connections) else {
                continue;
            };
            let (Some(catalog), Some(economies), Some(pools)) =
                (catalog, economies.as_mut(), pools.as_deref_mut())
            else {
                continue;
            };
            let source = purchase_card_source(&shop_states, player_id, message.card_id);
            let (result, update) = process_purchase_card(
                &mut shop_states,
                &mut hands,
                &mut *economies,
                pools,
                catalog,
                player_id,
                message.card_id,
            );

            if let Some(events) = purchase_network_events_for_result(
                result,
                player_id,
                message.card_id,
                source,
                &*economies,
                update,
                connections,
            ) {
                dispatch_purchase_network_events(
                    events,
                    reconnect_tracker.as_deref_mut(),
                    server,
                    sender.as_mut(),
                    &mut gold_updates,
                );
            }
        }
    }
}

pub fn build_draft_initial_offering(
    shop_states: &mut ShopStates,
    pools: &PlayerPools,
    sessions: &PlayerSessions,
    catalog: &CardCatalog,
    player_id: PlayerId,
    seed: u64,
) -> Option<S2CDraftOffering> {
    let player_state = shop_states.player_state_mut(player_id);
    reset_for_new_draft(player_state);
    player_state.phase = ShopPhase::DraftInitial;

    let player_class = sessions.players.get(&player_id)?.class;
    let pool = pools.pools.get(&player_id)?;
    let card_ids = pool.draw_initial_draft(
        &catalog.cards,
        player_class,
        DRAFT_INITIAL_OFFERING_COUNT,
        seed,
    );

    player_state
        .displayed_this_draft
        .extend(card_ids.iter().copied());

    Some(S2CDraftOffering { card_ids })
}

pub fn prepare_draft_offering_dispatch(
    player_id: PlayerId,
    message: S2CDraftOffering,
    connections: Option<&PlayerConnectionMap>,
) -> DraftOfferingDispatch {
    DraftOfferingDispatch {
        player_id,
        peer_id: connections.and_then(|connections| peer_for_player(&connections.0, player_id)),
        message,
    }
}

pub fn build_auto_shop_slots(
    shop_states: &mut ShopStates,
    pools: &PlayerPools,
    sessions: &PlayerSessions,
    catalog: &CardCatalog,
    config: &GameConfig,
    server_rng: &mut ServerRng,
    player_id: PlayerId,
    trigger: ShopRefreshTrigger,
) -> Option<S2CShopSlots> {
    let phase = match trigger {
        ShopRefreshTrigger::AuctionLock => ShopPhase::AuctionLock,
        ShopRefreshTrigger::ShopOpen => ShopPhase::ShopActive,
        _ => return None,
    };

    let player_class = sessions.players.get(&player_id)?.class;
    let pool = pools.pools.get(&player_id)?;
    let family_index = neutral_family_index(catalog);

    let player_state = shop_states.player_state_mut(player_id);
    reset_for_new_draft(player_state);
    player_state.phase = phase;

    Some(draw_shop_slots_into_state(
        player_state,
        pool,
        catalog,
        &family_index,
        player_class,
        config,
        server_rng,
        player_id,
    ))
}

pub fn build_manual_shop_slots(
    shop_states: &mut ShopStates,
    pools: &PlayerPools,
    sessions: &PlayerSessions,
    catalog: &CardCatalog,
    config: &GameConfig,
    server_rng: &mut ServerRng,
    player_id: PlayerId,
) -> Option<S2CShopSlots> {
    if shop_states.phase_for(player_id) != ShopPhase::ShopActive {
        return None;
    }

    let player_class = sessions.players.get(&player_id)?.class;
    let pool = pools.pools.get(&player_id)?;
    let family_index = neutral_family_index(catalog);
    let player_state = shop_states.players.get_mut(&player_id)?;

    Some(draw_shop_slots_into_state(
        player_state,
        pool,
        catalog,
        &family_index,
        player_class,
        config,
        server_rng,
        player_id,
    ))
}

pub fn prepare_shop_slots_dispatch(
    player_id: PlayerId,
    message: S2CShopSlots,
    connections: Option<&PlayerConnectionMap>,
) -> ShopSlotsDispatch {
    ShopSlotsDispatch {
        player_id,
        peer_id: connections.and_then(|connections| peer_for_player(&connections.0, player_id)),
        message,
    }
}

pub fn prepare_card_acquired_dispatch(
    player_id: PlayerId,
    message: S2CCardAcquired,
    connections: Option<&PlayerConnectionMap>,
) -> CardAcquiredDispatch {
    CardAcquiredDispatch {
        player_id,
        peer_id: connections.and_then(|connections| peer_for_player(&connections.0, player_id)),
        message,
    }
}

pub fn apply_shop_refresh_trigger(shop_states: &mut ShopStates, trigger: ShopRefreshTriggered) {
    let player_state = shop_states.player_state_mut(trigger.player_id);

    match trigger.trigger {
        ShopRefreshTrigger::DraftInitial => {
            reset_for_new_draft(player_state);
            player_state.phase = ShopPhase::DraftInitial;
        }
        ShopRefreshTrigger::AuctionLock => {
            reset_for_new_draft(player_state);
            player_state.phase = ShopPhase::AuctionLock;
        }
        ShopRefreshTrigger::ShopOpen => {
            reset_for_new_draft(player_state);
            player_state.phase = ShopPhase::ShopActive;
        }
        ShopRefreshTrigger::ShopUnlock => {
            player_state.phase = ShopPhase::ShopActive;
            player_state.refresh_count_this_draft = 0;
        }
    }
}

pub fn process_refresh_shop_request(
    shop_states: &mut ShopStates,
    player_id: PlayerId,
) -> RefreshAttemptResult {
    if shop_states.phase_for(player_id) == ShopPhase::ShopActive {
        return RefreshAttemptResult::AcceptedOutOfScope;
    }

    RefreshAttemptResult::DiscardedWrongPhase
}

#[allow(clippy::too_many_arguments)]
pub fn process_manual_refresh_shop_request(
    shop_states: &mut ShopStates,
    economies: &mut PlayerEconomies,
    pools: &PlayerPools,
    sessions: &PlayerSessions,
    catalog: &CardCatalog,
    config: &GameConfig,
    server_rng: &mut ServerRng,
    player_id: PlayerId,
) -> (RefreshAttemptResult, Option<S2CShopSlots>) {
    let refresh_cost = match shop_states.players.get(&player_id) {
        Some(shop) if shop.phase == ShopPhase::ShopActive => {
            manual_refresh_cost(config, shop.refresh_count_this_draft)
        }
        _ => return (RefreshAttemptResult::DiscardedWrongPhase, None),
    };

    let Some(economy) = economies.0.get_mut(&player_id) else {
        return (RefreshAttemptResult::PlayerEconomyMissing, None);
    };

    if economy_api::spend_gold(economy, refresh_cost).is_err() {
        return (RefreshAttemptResult::InsufficientGold, None);
    }

    let Some(message) = build_manual_shop_slots(
        shop_states,
        pools,
        sessions,
        catalog,
        config,
        server_rng,
        player_id,
    ) else {
        let Some(economy) = economies.0.get_mut(&player_id) else {
            return (RefreshAttemptResult::DrawUnavailable, None);
        };
        economy_api::refund_gold(economy, refresh_cost);
        return (RefreshAttemptResult::DrawUnavailable, None);
    };

    if let Some(shop) = shop_states.players.get_mut(&player_id) {
        shop.refresh_count_this_draft = shop.refresh_count_this_draft.saturating_add(1);
    }

    (RefreshAttemptResult::Refreshed, Some(message))
}

pub fn manual_refresh_cost(config: &GameConfig, refresh_count_this_draft: u32) -> u32 {
    config
        .refresh_base_cost
        .saturating_add(refresh_count_this_draft.min(config.refresh_cap))
}

pub fn process_purchase_card(
    shop_states: &mut ShopStates,
    hands: &mut PlayerHands,
    economies: &mut PlayerEconomies,
    pools: &mut PlayerPools,
    catalog: &CardCatalog,
    player_id: PlayerId,
    card_id: CardId,
) -> (PurchaseAttemptResult, Option<S2CShopSlots>) {
    let Some(pool) = pools.pools.get_mut(&player_id) else {
        return (PurchaseAttemptResult::PlayerPoolMissing, None);
    };

    process_purchase_card_with_pool(
        shop_states,
        hands,
        economies,
        pool,
        catalog,
        player_id,
        card_id,
    )
}

pub fn purchase_card_source(
    shop_states: &ShopStates,
    player_id: PlayerId,
    card_id: CardId,
) -> Option<CardSource> {
    let player_shop = shop_states.players.get(&player_id)?;
    match purchase_display(player_shop, card_id)? {
        PurchaseDisplay::DraftInitialOffering => Some(CardSource::DraftInitial),
        PurchaseDisplay::ShopSlot(_) => Some(CardSource::ShopPurchase),
    }
}

pub fn purchase_network_events_for_result(
    result: PurchaseAttemptResult,
    player_id: PlayerId,
    card_id: CardId,
    source: Option<CardSource>,
    economies: &PlayerEconomies,
    slots_update: Option<S2CShopSlots>,
    connections: Option<&PlayerConnectionMap>,
) -> Option<PurchaseNetworkEvents> {
    if result != PurchaseAttemptResult::Purchased {
        return None;
    }

    let source = source?;
    let gold_update = economy_gold_update_for_player(economies, player_id)?;
    let card_acquired =
        prepare_card_acquired_dispatch(player_id, S2CCardAcquired { card_id, source }, connections);
    let shop_slots =
        slots_update.map(|message| prepare_shop_slots_dispatch(player_id, message, connections));

    Some(PurchaseNetworkEvents {
        player_id,
        card_acquired,
        shop_slots,
        gold_update,
    })
}

pub fn process_purchase_card_with_pool(
    shop_states: &mut ShopStates,
    hands: &mut PlayerHands,
    economies: &mut PlayerEconomies,
    pool: &mut impl PurchasePool,
    catalog: &CardCatalog,
    player_id: PlayerId,
    card_id: CardId,
) -> (PurchaseAttemptResult, Option<S2CShopSlots>) {
    let Some(player_shop) = shop_states.players.get(&player_id) else {
        return (PurchaseAttemptResult::DiscardedWrongPhase, None);
    };

    if !matches!(
        player_shop.phase,
        ShopPhase::DraftInitial | ShopPhase::ShopActive
    ) {
        return (PurchaseAttemptResult::DiscardedWrongPhase, None);
    }

    if hands.hand_len(player_id) >= MAX_HAND_SIZE {
        return (PurchaseAttemptResult::HandFull, None);
    }

    let Some(display) = purchase_display(player_shop, card_id) else {
        return (PurchaseAttemptResult::CardNotDisplayed, None);
    };

    if !pool.is_available(card_id) {
        return (PurchaseAttemptResult::CardUnavailable, None);
    }

    let Some(card) = catalog.cards.get(&card_id) else {
        return (PurchaseAttemptResult::CardMissingFromCatalog, None);
    };

    let Some(economy) = economies.0.get_mut(&player_id) else {
        return (PurchaseAttemptResult::PlayerEconomyMissing, None);
    };

    if economy_api::spend_gold(economy, card.cost).is_err() {
        return (PurchaseAttemptResult::InsufficientGold, None);
    }

    match pool.distribute(card_id) {
        Ok(()) => {
            hands.push_card(player_id, card_id);
            let slots_update = remove_purchased_display(shop_states, player_id, card_id, display);
            (PurchaseAttemptResult::Purchased, slots_update)
        }
        Err(error) => {
            economy_api::refund_gold(economy, card.cost);
            tracing::error!(
                player_id = player_id.0,
                card_id = card_id.0,
                ?error,
                "card acquisition purchase distribution failed; gold refunded"
            );
            let result = match error {
                DistributeError::Exhausted => PurchaseAttemptResult::DistributeExhausted,
                DistributeError::UnknownCard => PurchaseAttemptResult::DistributeUnknownCard,
            };
            (result, None)
        }
    }
}

fn reset_for_new_draft(player_state: &mut super::state::PlayerShopState) {
    player_state.displayed_this_draft.clear();
    player_state.current_slots = [None; SHOP_SLOT_COUNT];
    player_state.refresh_count_this_draft = 0;
}

fn purchase_display(player_shop: &PlayerShopState, card_id: CardId) -> Option<PurchaseDisplay> {
    match player_shop.phase {
        ShopPhase::DraftInitial => player_shop
            .displayed_this_draft
            .contains(&card_id)
            .then_some(PurchaseDisplay::DraftInitialOffering),
        ShopPhase::ShopActive => player_shop
            .current_slots
            .iter()
            .position(|slot| slot.is_some_and(|displayed| displayed == card_id))
            .map(PurchaseDisplay::ShopSlot),
        ShopPhase::Inactive | ShopPhase::AuctionLock => None,
    }
}

fn remove_purchased_display(
    shop_states: &mut ShopStates,
    player_id: PlayerId,
    card_id: CardId,
    display: PurchaseDisplay,
) -> Option<S2CShopSlots> {
    let player_shop = shop_states.players.get_mut(&player_id)?;

    match display {
        PurchaseDisplay::DraftInitialOffering => {
            player_shop.displayed_this_draft.remove(&card_id);
            None
        }
        PurchaseDisplay::ShopSlot(slot_index) => {
            if slot_index < player_shop.current_slots.len() {
                player_shop.current_slots[slot_index] = None;
            }
            Some(S2CShopSlots {
                slots: player_shop.current_slots.to_vec(),
            })
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_shop_slots_into_state(
    player_state: &mut PlayerShopState,
    pool: &PlayerPool,
    catalog: &CardCatalog,
    family_index: &HashMap<String, Vec<CardId>>,
    player_class: ClassId,
    config: &GameConfig,
    server_rng: &mut ServerRng,
    player_id: PlayerId,
) -> S2CShopSlots {
    let mut slots = [None; SHOP_SLOT_COUNT];
    let rng_player = rng_player_id(player_id);

    for (slot_index, slot) in slots.iter_mut().enumerate() {
        let split_seed = server_rng.draw_shop_slot(rng_player, slot_index as u8);
        let slot_type = slot_type_from_seed(split_seed);
        let drawn = draw_unique_slot_card(
            slot_type,
            pool,
            catalog,
            family_index,
            player_class,
            config,
            server_rng,
            rng_player,
            slot_index as u8,
            player_state,
        );

        if let Some(card_id) = drawn {
            player_state.displayed_this_draft.insert(card_id);
            *slot = Some(card_id);
        }
    }

    player_state.current_slots = slots;
    S2CShopSlots {
        slots: slots.to_vec(),
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_unique_slot_card(
    slot_type: ShopSlotType,
    pool: &PlayerPool,
    catalog: &CardCatalog,
    family_index: &HashMap<String, Vec<CardId>>,
    player_class: ClassId,
    config: &GameConfig,
    server_rng: &mut ServerRng,
    rng_player: u32,
    slot_index: u8,
    player_state: &PlayerShopState,
) -> Option<CardId> {
    let eligible =
        eligible_cards_for_slot_type(pool, catalog, family_index, player_class, slot_type);
    if eligible.is_empty() {
        if slot_type == ShopSlotType::Class {
            return draw_unique_slot_card(
                ShopSlotType::Neutral,
                pool,
                catalog,
                family_index,
                player_class,
                config,
                server_rng,
                rng_player,
                slot_index,
                player_state,
            );
        }
        return None;
    }

    let displayed_eligible_count = eligible
        .iter()
        .filter(|card_id| player_state.displayed_this_draft.contains(card_id))
        .count();
    if displayed_eligible_count >= eligible.len() {
        return None;
    }

    for _attempt in 0..=SHOP_DEDUP_RETRY_LIMIT {
        let candidate = draw_slot_candidate(
            slot_type,
            pool,
            catalog,
            family_index,
            player_class,
            config,
            server_rng,
            rng_player,
            slot_index,
        )?;

        if !player_state.displayed_this_draft.contains(&candidate) {
            return Some(candidate);
        }
    }

    None
}

#[allow(clippy::too_many_arguments)]
fn draw_slot_candidate(
    slot_type: ShopSlotType,
    pool: &PlayerPool,
    catalog: &CardCatalog,
    family_index: &HashMap<String, Vec<CardId>>,
    player_class: ClassId,
    config: &GameConfig,
    server_rng: &mut ServerRng,
    rng_player: u32,
    slot_index: u8,
) -> Option<CardId> {
    match slot_type {
        ShopSlotType::Class => {
            let class_seed = server_rng.draw_shop_slot(rng_player, slot_index);
            pool.draw_class_card(&catalog.cards, player_class, class_seed, config)
                .or_else(|| {
                    draw_slot_candidate(
                        ShopSlotType::Neutral,
                        pool,
                        catalog,
                        family_index,
                        player_class,
                        config,
                        server_rng,
                        rng_player,
                        slot_index,
                    )
                })
        }
        ShopSlotType::Neutral => {
            let family_seed = server_rng.draw_shop_slot(rng_player, slot_index);
            let family =
                pool.draw_neutral_family(&catalog.cards, family_index, family_seed, config)?;
            let card_seed = server_rng.draw_shop_slot(rng_player, slot_index);
            pool.draw_family_card(&family, &catalog.cards, family_index, card_seed)
        }
    }
}

fn eligible_cards_for_slot_type(
    pool: &PlayerPool,
    catalog: &CardCatalog,
    family_index: &HashMap<String, Vec<CardId>>,
    player_class: ClassId,
    slot_type: ShopSlotType,
) -> Vec<CardId> {
    let mut eligible: Vec<CardId> = match slot_type {
        ShopSlotType::Class => catalog
            .cards
            .iter()
            .filter(|(card_id, card)| {
                card.class == player_class && pool.copies_remaining(**card_id) > 0
            })
            .map(|(card_id, _)| *card_id)
            .collect(),
        ShopSlotType::Neutral => family_index
            .values()
            .flat_map(|card_ids| card_ids.iter().copied())
            .filter(|card_id| pool.copies_remaining(*card_id) > 0)
            .collect(),
    };
    eligible.sort_by_key(|card_id| card_id.0);
    eligible.dedup();
    eligible
}

fn neutral_family_index(catalog: &CardCatalog) -> HashMap<String, Vec<CardId>> {
    let mut families: HashMap<String, Vec<CardId>> = HashMap::new();
    for (card_id, card) in &catalog.cards {
        if card.class != ClassId::Neutral {
            continue;
        }
        let Some(family) = &card.family else {
            continue;
        };
        families.entry(family.clone()).or_default().push(*card_id);
    }
    for card_ids in families.values_mut() {
        card_ids.sort_by_key(|card_id| card_id.0);
    }
    families
}

fn slot_type_from_seed(seed: u64) -> ShopSlotType {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    if rng.gen_range(0..2) == 0 {
        ShopSlotType::Class
    } else {
        ShopSlotType::Neutral
    }
}

fn resolve_player(
    remote: &RemoteId,
    connections: Option<&PlayerConnectionMap>,
) -> Option<PlayerId> {
    connections?.0.get(&remote.0).copied()
}

fn send_draft_offering(
    sender: &mut ServerMultiMessageSender,
    server: &Server,
    dispatch: &DraftOfferingDispatch,
) {
    let Some(peer_id) = dispatch.peer_id else {
        tracing::warn!(
            player_id = dispatch.player_id.0,
            "send_draft_offering: peer_id is None — S2CDraftOffering not sent"
        );
        return;
    };

    let _ = sender.send::<S2CDraftOffering, ReliableChannel>(
        &dispatch.message,
        server,
        &NetworkTarget::Single(peer_id),
    );
}

pub fn defer_draft_offering(
    tracker: Option<&mut ReconnectTracker>,
    dispatch: &DraftOfferingDispatch,
) -> bool {
    let deferred = defer_unicast_for_reconnect(
        tracker,
        dispatch.player_id,
        DeferredMessage::DraftOffering(dispatch.message.clone()),
    );
    if deferred {
        tracing::debug!(
            player_id = dispatch.player_id.0,
            "defer_draft_offering: S2CDraftOffering queued for reconnecting player"
        );
    }
    deferred
}

fn send_shop_slots(
    sender: &mut ServerMultiMessageSender,
    server: &Server,
    dispatch: &ShopSlotsDispatch,
) {
    let Some(peer_id) = dispatch.peer_id else {
        return;
    };

    let _ = sender.send::<S2CShopSlots, ReliableChannel>(
        &dispatch.message,
        server,
        &NetworkTarget::Single(peer_id),
    );
}

fn dispatch_purchase_network_events(
    events: PurchaseNetworkEvents,
    mut tracker: Option<&mut ReconnectTracker>,
    server: Option<&Server>,
    mut sender: Option<&mut ServerMultiMessageSender>,
    gold_updates: &mut MessageWriter<EconomyGoldUpdate>,
) {
    if !defer_card_acquired(tracker.as_deref_mut(), &events.card_acquired) {
        if let (Some(server), Some(sender)) = (server, sender.as_deref_mut()) {
            send_card_acquired(sender, server, &events.card_acquired);
        }
    }

    emit_economy_update(
        events.player_id,
        events.gold_update,
        tracker.as_deref_mut(),
        gold_updates,
    );

    if let Some(dispatch) = events.shop_slots.as_ref() {
        if !defer_shop_slots(tracker.as_deref_mut(), dispatch) {
            if let (Some(server), Some(sender)) = (server, sender.as_deref_mut()) {
                send_shop_slots(sender, server, dispatch);
            }
        }
    }
}

fn emit_economy_update_for_player(
    economies: &PlayerEconomies,
    player_id: PlayerId,
    tracker: Option<&mut ReconnectTracker>,
    gold_updates: &mut MessageWriter<EconomyGoldUpdate>,
) {
    let Some(gold_update) = economy_gold_update_for_player(economies, player_id) else {
        return;
    };

    emit_economy_update(player_id, gold_update, tracker, gold_updates);
}

fn emit_economy_update(
    player_id: PlayerId,
    gold_update: EconomyGoldUpdate,
    mut tracker: Option<&mut ReconnectTracker>,
    gold_updates: &mut MessageWriter<EconomyGoldUpdate>,
) {
    if !defer_gold_update(tracker.as_deref_mut(), player_id, &gold_update) {
        gold_updates.write(gold_update);
    }
}

pub fn economy_gold_update_for_player(
    economies: &PlayerEconomies,
    player_id: PlayerId,
) -> Option<EconomyGoldUpdate> {
    let economy = economies.0.get(&player_id)?;
    Some(EconomyGoldUpdate {
        player: player_id,
        gold: economy.gold,
        current_mana: economy.current_mana,
        reserve_mana: economy.reserve_mana,
        mana_cap: economy.mana_cap,
    })
}

fn protocol_gold_update(message: &EconomyGoldUpdate) -> ProtocolGoldUpdate {
    ProtocolGoldUpdate {
        gold: message.gold,
        current_mana: message.current_mana,
        reserve_mana: message.reserve_mana,
        mana_cap: message.mana_cap.min(u32::from(u8::MAX)) as u8,
    }
}

fn send_card_acquired(
    sender: &mut ServerMultiMessageSender,
    server: &Server,
    dispatch: &CardAcquiredDispatch,
) {
    let Some(peer_id) = dispatch.peer_id else {
        return;
    };

    let _ = sender.send::<S2CCardAcquired, ReliableChannel>(
        &dispatch.message,
        server,
        &NetworkTarget::Single(peer_id),
    );
}

pub fn defer_card_acquired(
    tracker: Option<&mut ReconnectTracker>,
    dispatch: &CardAcquiredDispatch,
) -> bool {
    defer_unicast_for_reconnect(
        tracker,
        dispatch.player_id,
        DeferredMessage::CardAcquired {
            card_id: dispatch.message.card_id,
            source: dispatch.message.source,
        },
    )
}

fn defer_gold_update(
    tracker: Option<&mut ReconnectTracker>,
    player_id: PlayerId,
    message: &EconomyGoldUpdate,
) -> bool {
    defer_unicast_for_reconnect(
        tracker,
        player_id,
        DeferredMessage::GoldUpdate(protocol_gold_update(message)),
    )
}

pub fn defer_shop_slots(
    tracker: Option<&mut ReconnectTracker>,
    dispatch: &ShopSlotsDispatch,
) -> bool {
    defer_unicast_for_reconnect(
        tracker,
        dispatch.player_id,
        DeferredMessage::ShopSlots(dispatch.message.clone()),
    )
}

fn peer_for_player(
    connections: &std::collections::HashMap<PeerId, PlayerId>,
    player_id: PlayerId,
) -> Option<PeerId> {
    connections
        .iter()
        .find_map(|(peer_id, mapped_player)| (*mapped_player == player_id).then_some(*peer_id))
}

fn rng_player_id(player_id: PlayerId) -> u32 {
    match u32::try_from(player_id.0) {
        Ok(id) => id,
        Err(_) => u32::MAX,
    }
}
