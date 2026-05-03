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
        cards: (1..=24)
            .map(|id| card(id, ClassId::Iop, None))
            .chain((100..=123).map(|id| card(id, ClassId::Neutral, Some("Neutral"))))
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
            rng: ServerRng::from_seed(7),
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

        match result {
            RefreshAttemptResult::Refreshed => {
                let message = message.expect("successful refresh should return S2CShopSlots");
                let shop = self
                    .shops
                    .players
                    .get(&self.player)
                    .expect("shop state exists after refresh");
                assert_eq!(message.slots, shop.current_slots.to_vec());
                assert_eq!(message.slots.len(), 3);
                assert!(message.slots.iter().any(Option::is_some));
            }
            _ => assert!(message.is_none()),
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

    fn shop_state(&self) -> PlayerShopState {
        self.shops
            .players
            .get(&self.player)
            .expect("shop state exists")
            .clone()
    }
}

#[test]
fn test_first_manual_refresh_costs_base_gold() {
    let mut fixture = RefreshFixture::new(10, 0);

    let result = fixture.process_refresh();

    assert_eq!(result, RefreshAttemptResult::Refreshed);
    assert_eq!(fixture.gold(), 9);
    assert_eq!(fixture.refresh_count(), 1);
}

#[test]
fn test_second_manual_refresh_cost_escalated() {
    let mut fixture = RefreshFixture::new(10, 1);

    let result = fixture.process_refresh();

    assert_eq!(result, RefreshAttemptResult::Refreshed);
    assert_eq!(fixture.gold(), 8);
    assert_eq!(fixture.refresh_count(), 2);
}

#[test]
fn test_refresh_cap_limits_cost() {
    let mut fixture = RefreshFixture::new(10, 5);

    let result = fixture.process_refresh();

    assert_eq!(result, RefreshAttemptResult::Refreshed);
    assert_eq!(fixture.gold(), 8);
    assert_eq!(fixture.refresh_count(), 6);
    assert_eq!(manual_refresh_cost(&fixture.config, u32::MAX), 2);
}

#[test]
fn test_insufficient_gold_no_refresh() {
    let mut fixture = RefreshFixture::new(1, 1);
    let before_shop = fixture.shop_state();

    let result = fixture.process_refresh();

    assert_eq!(result, RefreshAttemptResult::InsufficientGold);
    assert_eq!(fixture.gold(), 1);
    assert_eq!(fixture.refresh_count(), 1);
    assert_eq!(fixture.shop_state(), before_shop);
}

#[test]
fn test_wrong_phase_discards_refresh() {
    let mut fixture = RefreshFixture::new(10, 1);
    fixture
        .shops
        .players
        .get_mut(&fixture.player)
        .expect("shop state exists")
        .phase = ShopPhase::AuctionLock;
    let before_shop = fixture.shop_state();

    let result = fixture.process_refresh();

    assert_eq!(result, RefreshAttemptResult::DiscardedWrongPhase);
    assert_eq!(fixture.gold(), 10);
    assert_eq!(fixture.refresh_count(), 1);
    assert_eq!(fixture.shop_state(), before_shop);
}

#[test]
fn test_draw_failure_refunds_gold() {
    let mut fixture = RefreshFixture::new(10, 0);
    fixture.sessions.players.clear();
    let before_shop = fixture.shop_state();

    let result = fixture.process_refresh();

    assert_eq!(result, RefreshAttemptResult::DrawUnavailable);
    assert_eq!(fixture.gold(), 10);
    assert_eq!(fixture.refresh_count(), 0);
    assert_eq!(fixture.shop_state(), before_shop);
}

#[test]
fn test_draft_entry_resets_refresh_count() {
    for trigger in [
        ShopRefreshTrigger::DraftInitial,
        ShopRefreshTrigger::AuctionLock,
        ShopRefreshTrigger::ShopOpen,
        ShopRefreshTrigger::ShopUnlock,
    ] {
        let mut fixture = RefreshFixture::new(10, 3);

        apply_shop_refresh_trigger(
            &mut fixture.shops,
            ShopRefreshTriggered {
                player_id: fixture.player,
                trigger,
            },
        );

        assert_eq!(fixture.refresh_count(), 0, "trigger: {trigger:?}");
    }
}
