use std::collections::HashMap;

use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::pool::{PlayerPool, PlayerPools};
use server::core::session::{PlayerSessionData, PlayerSessions};
use server::feature::acquisition::{
    apply_shop_refresh_trigger, manual_refresh_cost, process_manual_refresh_shop_request,
    PlayerShopState, RefreshAttemptResult, ShopPhase, ShopRefreshTrigger, ShopRefreshTriggered,
    ShopStates,
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
        pool_copies_override: Some(3),
    }
}

fn catalog() -> CardCatalog {
    CardCatalog {
        cards: (1..=20)
            .map(|id| card(id, ClassId::Iop, None))
            .chain((100..=119).map(|id| card(id, ClassId::Neutral, Some("Neutral"))))
            .map(|card| (card.id, card))
            .collect(),
    }
}

fn config(refresh_base_cost: u32, refresh_cap: u32) -> GameConfig {
    GameConfig(shared::config::GameConfig {
        refresh_base_cost,
        refresh_cap,
        ..shared::config::GameConfig::default()
    })
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

struct RefreshFixture {
    player: PlayerId,
    shops: ShopStates,
    economies: PlayerEconomies,
    pools: PlayerPools,
    sessions: PlayerSessions,
    catalog: CardCatalog,
    config: GameConfig,
    rng: ServerRng,
}

impl RefreshFixture {
    fn new(gold: u32, refresh_count: u32) -> Self {
        Self::with_config(gold, refresh_count, 1, 1)
    }

    fn with_config(
        gold: u32,
        refresh_count: u32,
        refresh_base_cost: u32,
        refresh_cap: u32,
    ) -> Self {
        let player = PlayerId(1);
        let catalog = catalog();
        let config = config(refresh_base_cost, refresh_cap);
        let pools = PlayerPools {
            pools: HashMap::from([(player, PlayerPool::initialize(&catalog.cards, &config.0))]),
        };
        let sessions = PlayerSessions {
            players: HashMap::from([(
                player,
                PlayerSessionData {
                    class: ClassId::Iop,
                    class_locked: true,
                },
            )]),
        };
        let shops = ShopStates {
            players: HashMap::from([(
                player,
                PlayerShopState {
                    phase: ShopPhase::ShopActive,
                    displayed_this_draft: Default::default(),
                    current_slots: [None, None, None],
                    refresh_count_this_draft: refresh_count,
                },
            )]),
        };
        let economies = PlayerEconomies(HashMap::from([(player, economy(gold))]));

        Self {
            player,
            shops,
            economies,
            pools,
            sessions,
            catalog,
            config,
            rng: ServerRng::new(),
        }
    }

    fn process_refresh(&mut self) -> RefreshAttemptResult {
        let (result, message) = process_manual_refresh_shop_request(
            &mut self.shops,
            &mut self.economies,
            &self.pools,
            &self.sessions,
            &self.catalog,
            &self.config,
            &mut self.rng,
            self.player,
        );
        if result == RefreshAttemptResult::Refreshed {
            assert!(message.is_some());
        } else {
            assert!(message.is_none());
        }
        result
    }

    fn gold(&self) -> u32 {
        self.economies
            .0
            .get(&self.player)
            .expect("economy exists")
            .gold
    }

    fn refresh_count(&self) -> u32 {
        self.shops
            .players
            .get(&self.player)
            .expect("shop state exists")
            .refresh_count_this_draft
    }
}

#[test]
fn ca8_first_refresh_costs_base_and_increments_counter() {
    let mut fixture = RefreshFixture::new(1, 0);

    let result = fixture.process_refresh();

    assert_eq!(result, RefreshAttemptResult::Refreshed);
    assert_eq!(fixture.gold(), 0);
    assert_eq!(fixture.refresh_count(), 1);
}

#[test]
fn ca9_second_refresh_costs_base_plus_one() {
    let mut fixture = RefreshFixture::new(5, 1);

    let result = fixture.process_refresh();

    assert_eq!(result, RefreshAttemptResult::Refreshed);
    assert_eq!(fixture.gold(), 3);
    assert_eq!(fixture.refresh_count(), 2);
}

#[test]
fn ca10_refresh_cost_caps_at_config_cap() {
    let mut fixture = RefreshFixture::new(5, 5);

    let result = fixture.process_refresh();

    assert_eq!(result, RefreshAttemptResult::Refreshed);
    assert_eq!(fixture.gold(), 3);
    assert_eq!(fixture.refresh_count(), 6);
    assert_eq!(manual_refresh_cost(&fixture.config, u32::MAX), 2);
}

#[test]
fn ca11_insufficient_gold_leaves_gold_and_counter_unchanged() {
    let mut fixture = RefreshFixture::new(1, 1);
    let before_shop = fixture
        .shops
        .players
        .get(&fixture.player)
        .expect("shop state exists")
        .clone();

    let result = fixture.process_refresh();

    assert_eq!(result, RefreshAttemptResult::InsufficientGold);
    assert_eq!(fixture.gold(), 1);
    assert_eq!(fixture.refresh_count(), 1);
    assert_eq!(
        fixture
            .shops
            .players
            .get(&fixture.player)
            .expect("shop state exists"),
        &before_shop
    );
}

#[test]
fn ca15_shop_open_resets_counter_before_next_manual_refresh() {
    let mut fixture = RefreshFixture::new(5, 3);

    apply_shop_refresh_trigger(
        &mut fixture.shops,
        ShopRefreshTriggered {
            player_id: fixture.player,
            trigger: ShopRefreshTrigger::ShopOpen,
        },
    );
    assert_eq!(fixture.refresh_count(), 0);

    let result = fixture.process_refresh();

    assert_eq!(result, RefreshAttemptResult::Refreshed);
    assert_eq!(fixture.gold(), 4);
    assert_eq!(fixture.refresh_count(), 1);
}

#[test]
fn ca22_auction_entry_and_unlock_reset_counter_before_next_manual_refresh() {
    let mut fixture = RefreshFixture::new(5, 2);

    apply_shop_refresh_trigger(
        &mut fixture.shops,
        ShopRefreshTriggered {
            player_id: fixture.player,
            trigger: ShopRefreshTrigger::AuctionLock,
        },
    );
    assert_eq!(fixture.refresh_count(), 0);
    assert_eq!(
        fixture.shops.phase_for(fixture.player),
        ShopPhase::AuctionLock
    );

    apply_shop_refresh_trigger(
        &mut fixture.shops,
        ShopRefreshTriggered {
            player_id: fixture.player,
            trigger: ShopRefreshTrigger::ShopUnlock,
        },
    );
    assert_eq!(fixture.refresh_count(), 0);
    assert_eq!(
        fixture.shops.phase_for(fixture.player),
        ShopPhase::ShopActive
    );

    let result = fixture.process_refresh();

    assert_eq!(result, RefreshAttemptResult::Refreshed);
    assert_eq!(fixture.gold(), 4);
    assert_eq!(fixture.refresh_count(), 1);
}
