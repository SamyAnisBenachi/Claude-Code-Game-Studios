use bevy::prelude::*;
use lightyear::prelude::{
    MessageReceiver, NetworkTarget, PeerId, RemoteId, Server, ServerMultiMessageSender,
};
use shared::card::CardId;
use shared::protocol::{C2SPurchaseCard, C2SRefreshShop, ReliableChannel, S2CDraftOffering};
use shared::session::PlayerId;

use crate::core::economy::{api as economy_api, PlayerEconomies};
use crate::core::pool::PlayerPools;
use crate::core::session::PlayerConnectionMap;
use crate::core::session::PlayerSessions;
use crate::foundation::config::CardCatalog;
use crate::foundation::rng::ServerRng;

use super::hands::{PlayerHands, MAX_HAND_SIZE};
use super::messages::{ShopRefreshTrigger, ShopRefreshTriggered};
use super::state::{ShopPhase, ShopStates, SHOP_SLOT_COUNT};

pub const DRAFT_INITIAL_OFFERING_COUNT: u8 = 9;

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

#[allow(clippy::too_many_arguments)]
pub fn card_acquisition_tick_system(
    mut shop_states: ResMut<ShopStates>,
    mut hands: ResMut<PlayerHands>,
    mut economies: Option<ResMut<PlayerEconomies>>,
    pools: Option<Res<PlayerPools>>,
    sessions: Option<Res<PlayerSessions>>,
    mut server_rng: Option<ResMut<ServerRng>>,
    catalog: Option<Res<CardCatalog>>,
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
