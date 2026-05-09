use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use server::core::pool::{PlayerPool, PlayerPools};
use server::core::session::{PlayerSessionData, PlayerSessions};
use server::feature::acquisition::{
    build_auto_shop_slots, build_manual_shop_slots, CardAcquisitionPlugin, PlayerShopState,
    ShopPhase, ShopRefreshTrigger, ShopRefreshTriggered, ShopStates,
};
use server::foundation::config::{CardCatalog, GameConfig};
use server::foundation::rng::ServerRng;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::session::PlayerId;

fn card(id: u32, class: ClassId, family: Option<&str>) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class,
        family: family.map(str::to_string),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: 1,
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

fn rich_catalog() -> CardCatalog {
    catalog_with(
        (1..=30)
            .map(|id| card(id, ClassId::Iop, None))
            .chain((100..=129).map(|id| card(id, ClassId::Neutral, Some(&format!("Family{id}")))))
            .collect(),
    )
}

fn small_exhaustible_catalog() -> CardCatalog {
    catalog_with(vec![
        card(1, ClassId::Iop, None),
        card(2, ClassId::Iop, None),
        card(100, ClassId::Neutral, Some("NeutralA")),
        card(101, ClassId::Neutral, Some("NeutralB")),
    ])
}

fn sessions_for(player: PlayerId) -> PlayerSessions {
    PlayerSessions {
        players: HashMap::from([(
            player,
            PlayerSessionData {
                class: ClassId::Iop,
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

fn config() -> GameConfig {
    GameConfig(shared::config::GameConfig::default())
}

fn non_empty_slots(slots: &[Option<CardId>]) -> Vec<CardId> {
    slots.iter().filter_map(|slot| *slot).collect()
}

#[test]
fn test_auto_refresh_populates_dedup_set_and_distinct_slots() {
    let player = PlayerId(1);
    let catalog = rich_catalog();
    let pools = pools_for(player, &catalog);
    let sessions = sessions_for(player);
    let config = config();
    let mut rng = ServerRng::new();
    let mut shops = ShopStates::default();
    let before_displayed: HashSet<CardId> = HashSet::new();

    let message = build_auto_shop_slots(
        &mut shops,
        &pools,
        &sessions,
        &catalog,
        &config,
        &mut rng,
        player,
        ShopRefreshTrigger::ShopOpen,
    )
    .expect("shop slots should build with pool/session/catalog context");

    let shop = shops
        .players
        .get(&player)
        .expect("auto-refresh creates player shop state");
    let non_empty = non_empty_slots(&message.slots);
    let distinct = non_empty.iter().copied().collect::<HashSet<_>>();

    assert_eq!(shop.phase, ShopPhase::ShopActive);
    assert_eq!(message.slots, shop.current_slots.to_vec());
    assert_eq!(distinct.len(), non_empty.len());
    for card_id in non_empty {
        assert!(!before_displayed.contains(&card_id));
        assert!(shop.displayed_this_draft.contains(&card_id));
    }
}

#[test]
fn test_shop_open_message_reader_draws_slots_in_app_system() {
    let player = PlayerId(1);
    let catalog = rich_catalog();
    let mut app = App::new();
    app.add_plugins(CardAcquisitionPlugin);
    app.add_message::<ShopRefreshTriggered>();
    app.insert_resource(catalog.clone());
    app.insert_resource(pools_for(player, &catalog));
    app.insert_resource(sessions_for(player));
    app.insert_resource(config());
    app.insert_resource(ServerRng::new());

    app.world_mut()
        .resource_mut::<Messages<ShopRefreshTriggered>>()
        .write(ShopRefreshTriggered {
            player_id: player,
            trigger: ShopRefreshTrigger::ShopOpen,
        });

    app.update();

    let shops = app.world().resource::<ShopStates>();
    let shop = shops
        .players
        .get(&player)
        .expect("ShopOpen should create player shop state");
    let non_empty = non_empty_slots(&shop.current_slots);
    let distinct = non_empty.iter().copied().collect::<HashSet<_>>();

    assert_eq!(shop.phase, ShopPhase::ShopActive);
    assert_eq!(distinct.len(), non_empty.len());
    for card_id in non_empty {
        assert!(shop.displayed_this_draft.contains(&card_id));
    }
}

#[test]
fn test_k_greater_equal_n_short_circuits_to_empty_slots_without_retry_seeds() {
    let player = PlayerId(1);
    let catalog = small_exhaustible_catalog();
    let pools = pools_for(player, &catalog);
    let sessions = sessions_for(player);
    let config = config();
    let all_eligible = catalog.cards.keys().copied().collect::<HashSet<_>>();
    let mut rng = ServerRng::new();
    let mut shops = ShopStates {
        players: HashMap::from([(
            player,
            PlayerShopState {
                phase: ShopPhase::ShopActive,
                displayed_this_draft: all_eligible.clone(),
                current_slots: [Some(CardId(1)), Some(CardId(100)), None],
                refresh_count_this_draft: 0,
            },
        )]),
    };

    let message = build_manual_shop_slots(
        &mut shops, &pools, &sessions, &catalog, &config, &mut rng, player,
    )
    .expect("manual refresh draw should run in ShopActive");

    let shop = shops.players.get(&player).expect("shop state exists");
    assert_eq!(message.slots, vec![None, None, None]);
    assert_eq!(shop.current_slots, [None, None, None]);
    assert_eq!(shop.displayed_this_draft, all_eligible);
    assert_eq!(
        rng.audit_log().len(),
        4,
        "sentinel plus one Phase 1 split seed per slot; no retry or draw seeds"
    );
}

#[test]
fn test_manual_refresh_does_not_repeat_prior_auto_refresh_cards() {
    let player = PlayerId(1);
    let catalog = rich_catalog();
    let pools = pools_for(player, &catalog);
    let sessions = sessions_for(player);
    let config = config();
    let mut rng = ServerRng::new();
    let mut shops = ShopStates::default();

    let auto = build_auto_shop_slots(
        &mut shops,
        &pools,
        &sessions,
        &catalog,
        &config,
        &mut rng,
        player,
        ShopRefreshTrigger::ShopOpen,
    )
    .expect("auto-refresh should draw slots");
    let prior = non_empty_slots(&auto.slots)
        .into_iter()
        .collect::<HashSet<_>>();

    let manual = build_manual_shop_slots(
        &mut shops, &pools, &sessions, &catalog, &config, &mut rng, player,
    )
    .expect("manual refresh should draw slots");
    let refreshed = non_empty_slots(&manual.slots);

    assert!(refreshed.iter().all(|card_id| !prior.contains(card_id)));
    let shop = shops.players.get(&player).expect("shop state exists");
    for card_id in prior.iter().chain(refreshed.iter()) {
        assert!(shop.displayed_this_draft.contains(card_id));
    }
}

#[test]
fn test_n_zero_assigns_empty_slots_without_draw_or_retry_seeds() {
    let player = PlayerId(1);
    let catalog = catalog_with(Vec::new());
    let pools = pools_for(player, &catalog);
    let sessions = sessions_for(player);
    let config = config();
    let mut rng = ServerRng::new();
    let mut shops = ShopStates::default();

    let message = build_auto_shop_slots(
        &mut shops,
        &pools,
        &sessions,
        &catalog,
        &config,
        &mut rng,
        player,
        ShopRefreshTrigger::ShopOpen,
    )
    .expect("empty pool still produces an empty slot message");

    let shop = shops
        .players
        .get(&player)
        .expect("empty draw still creates player shop state");
    assert_eq!(message.slots, vec![None, None, None]);
    assert_eq!(shop.current_slots, [None, None, None]);
    assert!(shop.displayed_this_draft.is_empty());
    assert_eq!(
        rng.audit_log().len(),
        4,
        "sentinel plus one Phase 1 split seed per slot; N=0 prevents draw/retry seeds"
    );
}
