use std::collections::HashMap;

use bevy::prelude::*;
use lightyear::prelude::{
    MessageReceiver, NetworkTarget, PeerId, RemoteId, Server, ServerMultiMessageSender,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use shared::card::{CardId, ClassId};
use shared::protocol::{
    C2SPurchaseCard, C2SRefreshShop, ReliableChannel, S2CDraftOffering, S2CShopSlots,
};
use shared::session::PlayerId;

use crate::core::economy::{api as economy_api, PlayerEconomies};
use crate::core::pool::{PlayerPool, PlayerPools};
use crate::core::session::PlayerConnectionMap;
use crate::core::session::PlayerSessions;
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
    CardMissingFromCatalog,
    PlayerEconomyMissing,
    InsufficientGold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshAttemptResult {
    AcceptedOutOfScope,
    DiscardedWrongPhase,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShopSlotType {
    Class,
    Neutral,
}

#[allow(clippy::too_many_arguments)]
pub fn card_acquisition_tick_system(
    mut shop_states: ResMut<ShopStates>,
    mut hands: ResMut<PlayerHands>,
    mut economies: Option<ResMut<PlayerEconomies>>,
    pools: Option<Res<PlayerPools>>,
    sessions: Option<Res<PlayerSessions>>,
    mut server_rng: Option<ResMut<ServerRng>>,
    catalog: Option<Res<CardCatalog>>,
    config: Option<Res<GameConfig>>,
    connections: Option<Res<PlayerConnectionMap>>,
    mut shop_refreshes: MessageReader<ShopRefreshTriggered>,
    mut refresh_receivers: Query<(&RemoteId, &mut MessageReceiver<C2SRefreshShop>)>,
    mut purchase_receivers: Query<(&RemoteId, &mut MessageReceiver<C2SPurchaseCard>)>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
) {
    let server = server.single().ok();

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

            if let (Some(dispatch), Some(server), Some(sender)) =
                (dispatch.as_ref(), server, sender.as_mut())
            {
                send_draft_offering(sender, server, dispatch);
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

            if let (Some(dispatch), Some(server), Some(sender)) =
                (dispatch.as_ref(), server, sender.as_mut())
            {
                send_shop_slots(sender, server, dispatch);
            }
            continue;
        }

        apply_shop_refresh_trigger(&mut shop_states, *refresh);
    }

    let connections = connections.as_deref();
    for (remote, mut receiver) in refresh_receivers.iter_mut() {
        for _message in receiver.receive() {
            let Some(player_id) = resolve_player(remote, connections) else {
                continue;
            };
            let _ = process_refresh_shop_request(&mut shop_states, player_id);
        }
    }

    let catalog = catalog.as_deref();
    for (remote, mut receiver) in purchase_receivers.iter_mut() {
        for message in receiver.receive() {
            let Some(player_id) = resolve_player(remote, connections) else {
                continue;
            };
            let (Some(catalog), Some(economies)) = (catalog, economies.as_mut()) else {
                continue;
            };
            let _ = process_purchase_card(
                &mut shop_states,
                &mut hands,
                &mut *economies,
                catalog,
                player_id,
                message.card_id,
            );
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

pub fn process_purchase_card(
    shop_states: &mut ShopStates,
    hands: &mut PlayerHands,
    economies: &mut PlayerEconomies,
    catalog: &CardCatalog,
    player_id: PlayerId,
    card_id: CardId,
) -> PurchaseAttemptResult {
    let Some(player_shop) = shop_states.players.get(&player_id) else {
        return PurchaseAttemptResult::DiscardedWrongPhase;
    };

    if !matches!(
        player_shop.phase,
        ShopPhase::DraftInitial | ShopPhase::ShopActive
    ) {
        return PurchaseAttemptResult::DiscardedWrongPhase;
    }

    if hands.hand_len(player_id) >= MAX_HAND_SIZE {
        return PurchaseAttemptResult::HandFull;
    }

    if !player_shop.displays_card(card_id) {
        return PurchaseAttemptResult::CardNotDisplayed;
    }

    let Some(card) = catalog.cards.get(&card_id) else {
        return PurchaseAttemptResult::CardMissingFromCatalog;
    };

    let Some(economy) = economies.0.get_mut(&player_id) else {
        return PurchaseAttemptResult::PlayerEconomyMissing;
    };

    if economy_api::spend_gold(economy, card.cost).is_err() {
        return PurchaseAttemptResult::InsufficientGold;
    }

    hands.push_card(player_id, card_id);
    PurchaseAttemptResult::Purchased
}

fn reset_for_new_draft(player_state: &mut super::state::PlayerShopState) {
    player_state.displayed_this_draft.clear();
    player_state.current_slots = [None; SHOP_SLOT_COUNT];
    player_state.refresh_count_this_draft = 0;
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
        return;
    };

    let _ = sender.send::<S2CDraftOffering, ReliableChannel>(
        &dispatch.message,
        server,
        &NetworkTarget::Single(peer_id),
    );
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
