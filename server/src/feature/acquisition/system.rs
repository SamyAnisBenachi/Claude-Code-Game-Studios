use bevy::prelude::*;
use lightyear::prelude::{MessageReceiver, RemoteId};
use shared::card::CardId;
use shared::protocol::{C2SPurchaseCard, C2SRefreshShop};
use shared::session::PlayerId;

use crate::core::economy::{api as economy_api, PlayerEconomies};
use crate::core::session::PlayerConnectionMap;
use crate::foundation::config::CardCatalog;

use super::hands::{PlayerHands, MAX_HAND_SIZE};
use super::messages::{ShopRefreshTrigger, ShopRefreshTriggered};
use super::state::{ShopPhase, ShopStates, SHOP_SLOT_COUNT};

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

pub fn card_acquisition_tick_system(
    mut shop_states: ResMut<ShopStates>,
    mut hands: ResMut<PlayerHands>,
    mut economies: Option<ResMut<PlayerEconomies>>,
    catalog: Option<Res<CardCatalog>>,
    connections: Option<Res<PlayerConnectionMap>>,
    mut shop_refreshes: MessageReader<ShopRefreshTriggered>,
    mut refresh_receivers: Query<(&RemoteId, &mut MessageReceiver<C2SRefreshShop>)>,
    mut purchase_receivers: Query<(&RemoteId, &mut MessageReceiver<C2SPurchaseCard>)>,
) {
    for refresh in shop_refreshes.read() {
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
