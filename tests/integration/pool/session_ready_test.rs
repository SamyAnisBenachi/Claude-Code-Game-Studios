use std::collections::HashMap;

use bevy::prelude::*;
use server::core::pool::{
    CardPoolPlugin, InitialDraftOffering, ManualRefreshCount, PlayerPool, PlayerPools, ShopSlots,
};
use server::core::rsm::{DraftStarted, GameOverEmitted, ShopRefreshTrigger, ShopRefreshTriggered};
use server::core::session::{PlayerSessionData, PlayerSessions, SessionConfig};
use server::feature::acquisition::{CardAcquisitionPlugin, ShopPhase, ShopStates};
use server::foundation::config::{CardCatalog, GameConfig};
use server::foundation::rng::ServerRng;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{DraftPhase, GameMode, GameOverReason, PlacementTimerMultiplier};
use shared::session::PlayerId;

fn card(id: u32, class: ClassId) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class,
        family: Some("Test".to_string()),
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
        pool_copies_override: Some(2),
    }
}

fn catalog_with(cards: impl IntoIterator<Item = CardData>) -> CardCatalog {
    CardCatalog {
        cards: cards.into_iter().map(|card| (card.id, card)).collect(),
    }
}

fn test_catalog() -> CardCatalog {
    catalog_with(
        (1..=12)
            .map(|id| card(id, ClassId::Iop))
            .chain((100..=105).map(|id| card(id, ClassId::Neutral))),
    )
}

fn session_config(players: &[PlayerId]) -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: players.len() as u8,
        team_map: players
            .iter()
            .enumerate()
            .map(|(index, player)| (*player, index as u8))
            .collect(),
        class_map: players
            .iter()
            .map(|player| (*player, ClassId::Iop))
            .collect(),
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
    }
}

fn player_sessions(players: &[PlayerId]) -> PlayerSessions {
    PlayerSessions {
        players: players
            .iter()
            .map(|player| {
                (
                    *player,
                    PlayerSessionData {
                        class: ClassId::Iop,
                        class_locked: true,
                    },
                )
            })
            .collect(),
    }
}

fn app_with_card_pool(catalog: CardCatalog, players: &[PlayerId]) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CardPoolPlugin));
    app.insert_resource(catalog);
    app.insert_resource(GameConfig(shared::config::GameConfig::default()));
    app.insert_resource(session_config(players));
    app
}

#[test]
fn test_draft_initial_initializes_player_pools() {
    let players = [PlayerId(1), PlayerId(2)];
    let stale_player = PlayerId(99);
    let catalog = test_catalog();
    let stale_catalog = catalog_with([card(999, ClassId::Cra)]);
    let mut app = app_with_card_pool(catalog.clone(), &players);
    app.world_mut().insert_resource(PlayerPools {
        pools: HashMap::from([(
            stale_player,
            PlayerPool::initialize(&stale_catalog.cards, &shared::config::GameConfig::default()),
        )]),
    });

    app.world_mut()
        .resource_mut::<Messages<DraftStarted>>()
        .write(DraftStarted {
            round: 1,
            phase: DraftPhase::Initial,
        });
    app.update();

    let pools = app.world().resource::<PlayerPools>();
    assert!(!pools.pools.contains_key(&stale_player));
    assert_eq!(pools.pools.len(), players.len());
    for player in players {
        let pool = pools
            .pools
            .get(&player)
            .expect("DraftInitial should initialize each session player");
        assert_eq!(pool.copies_remaining.len(), catalog.cards.len());
        assert!(pool.copies_remaining.values().all(|copies| *copies >= 1));
    }
}

#[test]
fn test_non_initial_draft_does_not_reinitialize_pools() {
    let player = PlayerId(1);
    let catalog = test_catalog();
    let mut app = app_with_card_pool(catalog.clone(), &[player]);
    app.world_mut().insert_resource(PlayerPools {
        pools: HashMap::from([(
            player,
            PlayerPool::initialize(&catalog.cards, &shared::config::GameConfig::default()),
        )]),
    });
    let before_len = app.world().resource::<PlayerPools>().pools[&player]
        .copies_remaining
        .len();

    app.world_mut()
        .resource_mut::<Messages<DraftStarted>>()
        .write(DraftStarted {
            round: 2,
            phase: DraftPhase::Shop,
        });
    app.update();

    let pools = app.world().resource::<PlayerPools>();
    assert_eq!(pools.pools.len(), 1);
    assert_eq!(pools.pools[&player].copies_remaining.len(), before_len);
}

#[test]
fn test_pool_init_precedes_card_acquisition_refresh() {
    let player = PlayerId(1);
    let catalog = test_catalog();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CardPoolPlugin, CardAcquisitionPlugin));
    app.insert_resource(catalog);
    app.insert_resource(GameConfig(shared::config::GameConfig::default()));
    app.insert_resource(session_config(&[player]));
    app.insert_resource(player_sessions(&[player]));
    app.insert_resource(ServerRng::from_seed(7));

    app.world_mut()
        .resource_mut::<Messages<DraftStarted>>()
        .write(DraftStarted {
            round: 1,
            phase: DraftPhase::Initial,
        });
    app.world_mut()
        .resource_mut::<Messages<ShopRefreshTriggered>>()
        .write(ShopRefreshTriggered {
            player_id: player,
            trigger: ShopRefreshTrigger::DraftInitial,
        });
    app.update();

    let pools = app.world().resource::<PlayerPools>();
    assert!(pools.pools.contains_key(&player));

    let shops = app.world().resource::<ShopStates>();
    let shop = shops
        .players
        .get(&player)
        .expect("Card Acquisition should process DraftInitial refresh");
    assert_eq!(shop.phase, ShopPhase::DraftInitial);
    assert_eq!(shop.displayed_this_draft.len(), 9);
}

#[test]
fn test_game_over_clears_pool_session_resources() {
    let player = PlayerId(1);
    let catalog = test_catalog();
    let mut app = app_with_card_pool(catalog.clone(), &[player]);
    app.world_mut().insert_resource(PlayerPools {
        pools: HashMap::from([(
            player,
            PlayerPool::initialize(&catalog.cards, &shared::config::GameConfig::default()),
        )]),
    });
    app.world_mut().insert_resource(ShopSlots(HashMap::from([(
        player,
        vec![CardId(1), CardId(2)],
    )])));
    app.world_mut()
        .insert_resource(InitialDraftOffering(HashMap::from([(
            player,
            vec![CardId(3)],
        )])));
    app.world_mut()
        .insert_resource(ManualRefreshCount(HashMap::from([(player, 2)])));

    app.world_mut()
        .resource_mut::<Messages<GameOverEmitted>>()
        .write(GameOverEmitted {
            reason: GameOverReason::Draw,
            loser: None,
            round: 3,
        });
    app.update();

    assert!(app.world().resource::<PlayerPools>().pools.is_empty());
    assert!(app.world().resource::<ShopSlots>().0.is_empty());
    assert!(app.world().resource::<InitialDraftOffering>().0.is_empty());
    assert!(app.world().resource::<ManualRefreshCount>().0.is_empty());
}

#[test]
fn test_card_pool_plugin_registers_cleanly() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CardPoolPlugin));
    app.update();

    assert!(app.world().contains_resource::<PlayerPools>());
    assert!(app.world().contains_resource::<ShopSlots>());
    assert!(app.world().contains_resource::<InitialDraftOffering>());
    assert!(app.world().contains_resource::<ManualRefreshCount>());
    assert!(app.world().contains_resource::<Messages<DraftStarted>>());
    assert!(app.world().contains_resource::<Messages<GameOverEmitted>>());
}
