use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::feature::acquisition::{
    apply_shop_refresh_trigger, process_purchase_card, process_refresh_shop_request,
    CardAcquisitionPlugin, PlayerHands, PlayerShopState, PurchaseAttemptResult,
    RefreshAttemptResult, ShopPhase, ShopRefreshTrigger, ShopRefreshTriggered, ShopStates,
};
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::session::PlayerId;

fn card(id: u32, cost: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost,
        atk: 1,
        hp: 1,
        mp: 1,
        ar: 0,
        keywords: vec![],
        effect_text: String::new(),
        art_id: format!("test_{id}"),
        pool_copies_override: Some(1),
    }
}

fn catalog_with(cards: Vec<CardData>) -> CardCatalog {
    CardCatalog {
        cards: cards.into_iter().map(|card| (card.id, card)).collect(),
    }
}

fn economy(gold: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold,
        current_mana: 0,
        reserve_mana: 0,
        mana_cap: 10,
        reserved_gold: 0,
    }
}

fn shop_state(phase: ShopPhase, displayed: CardId) -> PlayerShopState {
    PlayerShopState {
        phase,
        displayed_this_draft: HashSet::from([displayed]),
        current_slots: [Some(displayed), None, None],
        refresh_count_this_draft: 2,
    }
}

fn hand_with_len(len: u32) -> Vec<CardId> {
    (1..=len).map(CardId).collect()
}

#[test]
fn plugin_registers_state_resources_and_refresh_message() {
    let mut app = App::new();
    app.add_plugins(CardAcquisitionPlugin);
    app.finish();
    app.cleanup();

    assert!(app.world().contains_resource::<ShopStates>());
    assert!(app.world().contains_resource::<PlayerHands>());
    assert!(app
        .world()
        .contains_resource::<Messages<ShopRefreshTriggered>>());
}

#[test]
fn shop_refresh_trigger_enters_shop_active_from_message_bus() {
    let player = PlayerId(1);
    let mut app = App::new();
    app.add_plugins(CardAcquisitionPlugin);

    app.world_mut()
        .resource_mut::<Messages<ShopRefreshTriggered>>()
        .write(ShopRefreshTriggered {
            player_id: player,
            trigger: ShopRefreshTrigger::ShopOpen,
        });

    app.update();

    let shops = app.world().resource::<ShopStates>();
    assert_eq!(shops.phase_for(player), ShopPhase::ShopActive);
}

#[test]
fn shop_unlock_preserves_displayed_cards_and_slots() {
    let player = PlayerId(1);
    let card_id = CardId(42);
    let mut shops = ShopStates::default();
    shops.players.insert(
        player,
        PlayerShopState {
            phase: ShopPhase::AuctionLock,
            displayed_this_draft: HashSet::from([card_id]),
            current_slots: [Some(card_id), None, None],
            refresh_count_this_draft: 5,
        },
    );

    apply_shop_refresh_trigger(
        &mut shops,
        ShopRefreshTriggered {
            player_id: player,
            trigger: ShopRefreshTrigger::ShopUnlock,
        },
    );

    let state = shops
        .players
        .get(&player)
        .expect("player shop state exists");
    assert_eq!(state.phase, ShopPhase::ShopActive);
    assert_eq!(state.current_slots, [Some(card_id), None, None]);
    assert!(state.displayed_this_draft.contains(&card_id));
    assert_eq!(state.refresh_count_this_draft, 0);
}

#[test]
fn purchase_at_nine_cards_adds_tenth_card_and_spends_gold() {
    let player = PlayerId(1);
    let card_id = CardId(42);
    let mut shops = ShopStates::default();
    shops
        .players
        .insert(player, shop_state(ShopPhase::ShopActive, card_id));
    let mut hands = PlayerHands {
        hands: HashMap::from([(player, hand_with_len(9))]),
    };
    let mut economies = PlayerEconomies(HashMap::from([(player, economy(5))]));
    let catalog = catalog_with(vec![card(42, 2)]);

    let result = process_purchase_card(
        &mut shops,
        &mut hands,
        &mut economies,
        &catalog,
        player,
        card_id,
    );

    assert_eq!(result, PurchaseAttemptResult::Purchased);
    assert_eq!(hands.hand_len(player), 10);
    assert_eq!(
        hands.hands.get(&player).expect("hand exists").last(),
        Some(&card_id)
    );
    assert_eq!(economies.0.get(&player).expect("economy exists").gold, 3);
}

#[test]
fn purchase_at_hand_cap_is_rejected_without_gold_or_slot_change() {
    let player = PlayerId(1);
    let card_id = CardId(42);
    let mut shops = ShopStates::default();
    shops
        .players
        .insert(player, shop_state(ShopPhase::ShopActive, card_id));
    let mut hands = PlayerHands {
        hands: HashMap::from([(player, hand_with_len(10))]),
    };
    let mut economies = PlayerEconomies(HashMap::from([(player, economy(5))]));
    let catalog = catalog_with(vec![card(42, 2)]);

    let result = process_purchase_card(
        &mut shops,
        &mut hands,
        &mut economies,
        &catalog,
        player,
        card_id,
    );

    assert_eq!(result, PurchaseAttemptResult::HandFull);
    assert_eq!(hands.hand_len(player), 10);
    assert_eq!(economies.0.get(&player).expect("economy exists").gold, 5);
    assert_eq!(
        shops
            .players
            .get(&player)
            .expect("player shop state exists")
            .current_slots[0],
        Some(card_id)
    );
}

#[test]
fn auction_lock_discards_purchase_and_refresh_without_state_changes() {
    let player = PlayerId(1);
    let card_id = CardId(42);
    let initial_shop = shop_state(ShopPhase::AuctionLock, card_id);
    let mut shops = ShopStates::default();
    shops.players.insert(player, initial_shop.clone());
    let mut hands = PlayerHands {
        hands: HashMap::from([(player, hand_with_len(9))]),
    };
    let mut economies = PlayerEconomies(HashMap::from([(player, economy(5))]));
    let catalog = catalog_with(vec![card(42, 2)]);

    let purchase_result = process_purchase_card(
        &mut shops,
        &mut hands,
        &mut economies,
        &catalog,
        player,
        card_id,
    );
    let refresh_result = process_refresh_shop_request(&mut shops, player);

    assert_eq!(purchase_result, PurchaseAttemptResult::DiscardedWrongPhase);
    assert_eq!(refresh_result, RefreshAttemptResult::DiscardedWrongPhase);
    assert_eq!(
        shops
            .players
            .get(&player)
            .expect("player shop state exists"),
        &initial_shop
    );
    assert_eq!(hands.hand_len(player), 9);
    assert_eq!(economies.0.get(&player).expect("economy exists").gold, 5);
}
