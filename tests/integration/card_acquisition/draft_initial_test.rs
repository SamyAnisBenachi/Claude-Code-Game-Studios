use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use lightyear::prelude::PeerId;
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::pool::{PlayerPool, PlayerPools};
use server::core::session::{PlayerConnectionMap, PlayerSessionData, PlayerSessions};
use server::feature::acquisition::{
    build_draft_initial_offering, prepare_draft_offering_dispatch, process_purchase_card,
    process_refresh_shop_request, CardAcquisitionPlugin, PlayerHands, PlayerShopState,
    PurchaseAttemptResult, RefreshAttemptResult, ShopPhase, ShopRefreshTrigger,
    ShopRefreshTriggered, ShopStates, DRAFT_INITIAL_OFFERING_COUNT,
};
use server::foundation::config::CardCatalog;
use server::foundation::rng::ServerRng;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::session::PlayerId;

fn card(id: u32, class: ClassId, cost: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class,
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
        pool_copies_override: Some(2),
    }
}

fn catalog_with(cards: Vec<CardData>) -> CardCatalog {
    CardCatalog {
        cards: cards.into_iter().map(|card| (card.id, card)).collect(),
    }
}

fn draft_catalog() -> CardCatalog {
    catalog_with(
        (1..=10)
            .map(|id| card(id, ClassId::Iop, 1))
            .chain((100..=105).map(|id| card(id, ClassId::Neutral, 1)))
            .chain((200..=205).map(|id| card(id, ClassId::Cra, 1)))
            .collect(),
    )
}

fn sessions_for(player: PlayerId, class: ClassId) -> PlayerSessions {
    PlayerSessions {
        players: HashMap::from([(
            player,
            PlayerSessionData {
                class,
                class_locked: true,
            },
        )]),
    }
}

fn pools_for(player: PlayerId, catalog: &CardCatalog) -> PlayerPools {
    PlayerPools {
        pools: HashMap::from([(
            player,
            PlayerPool::initialize(&catalog.cards, &shared::config::GameConfig::default()),
        )]),
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

#[test]
fn test_draft_initial_trigger_populates_nine_distinct_offering() {
    let player = PlayerId(1);
    let catalog = draft_catalog();
    let pools = pools_for(player, &catalog);
    let sessions = sessions_for(player, ClassId::Iop);
    let mut shops = ShopStates::default();

    let offering =
        build_draft_initial_offering(&mut shops, &pools, &sessions, &catalog, player, 42)
            .expect("player session and pool should exist");
    let distinct = offering.card_ids.iter().copied().collect::<HashSet<_>>();

    assert_eq!(
        offering.card_ids.len(),
        DRAFT_INITIAL_OFFERING_COUNT as usize
    );
    assert_eq!(distinct.len(), DRAFT_INITIAL_OFFERING_COUNT as usize);

    let shop = shops
        .players
        .get(&player)
        .expect("DraftInitial should create player shop state");
    assert_eq!(shop.phase, ShopPhase::DraftInitial);
    assert_eq!(
        shop.displayed_this_draft.len(),
        DRAFT_INITIAL_OFFERING_COUNT as usize
    );
    for card_id in &offering.card_ids {
        assert!(shop.displayed_this_draft.contains(card_id));
        let card = catalog
            .cards
            .get(card_id)
            .expect("offered card must exist in catalog");
        assert!(matches!(card.class, ClassId::Iop | ClassId::Neutral));
    }
}

#[test]
fn test_draft_initial_app_system_consumes_one_rng_seed_and_updates_state() {
    let player = PlayerId(1);
    let catalog = draft_catalog();
    let mut app = App::new();
    app.add_plugins(CardAcquisitionPlugin);
    app.insert_resource(catalog.clone());
    app.insert_resource(pools_for(player, &catalog));
    app.insert_resource(sessions_for(player, ClassId::Iop));
    app.insert_resource(ServerRng::new());

    app.world_mut()
        .resource_mut::<Messages<ShopRefreshTriggered>>()
        .write(ShopRefreshTriggered {
            player_id: player,
            trigger: ShopRefreshTrigger::DraftInitial,
        });

    app.update();

    let shops = app.world().resource::<ShopStates>();
    let shop = shops
        .players
        .get(&player)
        .expect("DraftInitial should create player shop state");
    assert_eq!(shop.phase, ShopPhase::DraftInitial);
    assert_eq!(
        shop.displayed_this_draft.len(),
        DRAFT_INITIAL_OFFERING_COUNT as usize
    );
    assert_eq!(app.world().resource::<ServerRng>().audit_log().len(), 2);
}

#[test]
fn test_draft_initial_purchase_carries_unspent_gold() {
    let player = PlayerId(1);
    let card_id = CardId(42);
    let mut shops = ShopStates {
        players: HashMap::from([(
            player,
            PlayerShopState {
                phase: ShopPhase::DraftInitial,
                displayed_this_draft: HashSet::from([card_id]),
                current_slots: [None, None, None],
                refresh_count_this_draft: 0,
            },
        )]),
    };
    let mut hands = PlayerHands::default();
    let mut economies = PlayerEconomies(HashMap::from([(player, economy(5))]));
    let catalog = catalog_with(vec![card(42, ClassId::Iop, 3)]);
    let mut pools = pools_for(player, &catalog);

    let (result, update) = process_purchase_card(
        &mut shops,
        &mut hands,
        &mut economies,
        &mut pools,
        &catalog,
        player,
        card_id,
    );

    assert_eq!(result, PurchaseAttemptResult::Purchased);
    assert!(update.is_none());
    assert_eq!(hands.hand_len(player), 1);
    assert_eq!(economies.0.get(&player).expect("economy exists").gold, 2);
}

#[test]
fn test_refresh_request_in_draft_initial_is_silently_discarded() {
    let player = PlayerId(1);
    let card_id = CardId(42);
    let initial_shop = PlayerShopState {
        phase: ShopPhase::DraftInitial,
        displayed_this_draft: HashSet::from([card_id]),
        current_slots: [None, None, None],
        refresh_count_this_draft: 3,
    };
    let mut shops = ShopStates {
        players: HashMap::from([(player, initial_shop.clone())]),
    };

    let result = process_refresh_shop_request(&mut shops, player);

    assert_eq!(result, RefreshAttemptResult::DiscardedWrongPhase);
    assert_eq!(
        shops
            .players
            .get(&player)
            .expect("player shop state exists"),
        &initial_shop
    );
}

#[test]
fn test_draft_initial_dispatch_targets_only_mapped_player() {
    let player = PlayerId(1);
    let opponent = PlayerId(2);
    let target_peer = PeerId::Netcode(11);
    let opponent_peer = PeerId::Netcode(12);
    let connections = PlayerConnectionMap(HashMap::from([
        (target_peer, player),
        (opponent_peer, opponent),
    ]));
    let message = shared::protocol::S2CDraftOffering {
        card_ids: vec![CardId(1), CardId(2), CardId(3)],
    };

    let dispatch = prepare_draft_offering_dispatch(player, message, Some(&connections));

    assert_eq!(dispatch.player_id, player);
    assert_eq!(dispatch.peer_id, Some(target_peer));
    assert_ne!(dispatch.peer_id, Some(opponent_peer));
}
