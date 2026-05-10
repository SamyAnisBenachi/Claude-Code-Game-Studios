use std::collections::{HashMap, HashSet};

use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::pool::{DistributeError, PlayerPool, PlayerPools};
use server::feature::acquisition::{
    process_purchase_card, process_purchase_card_with_pool, purchase_card_source,
    purchase_network_events_for_result, PlayerHands, PlayerShopState, PurchaseAttemptResult,
    PurchasePool, ShopPhase, ShopStates,
};
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::CardSource;
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn card(id: u32, cost: u32, copies: i32) -> CardData {
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
        pool_copies_override: Some(copies),
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

fn shop_state(phase: ShopPhase, slots: [Option<CardId>; 3]) -> PlayerShopState {
    PlayerShopState {
        phase,
        displayed_this_draft: slots
            .iter()
            .filter_map(|slot| *slot)
            .collect::<HashSet<_>>(),
        current_slots: slots,
        refresh_count_this_draft: 0,
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

fn gold(economies: &PlayerEconomies, player: PlayerId) -> u32 {
    economies.0.get(&player).expect("economy exists").gold
}

struct ExhaustingPurchasePool {
    distribute_calls: u32,
}

impl PurchasePool for ExhaustingPurchasePool {
    fn is_available(&self, _card_id: CardId) -> bool {
        true
    }

    fn distribute(&mut self, _card_id: CardId) -> Result<(), DistributeError> {
        self.distribute_calls = self.distribute_calls.saturating_add(1);
        Err(DistributeError::Exhausted)
    }
}

#[test]
fn ca13_unavailable_slot_rejects_purchase_and_leaves_dead_slot() {
    test_helpers::init_test_tracing();
    let player = PlayerId(1);
    let card_id = CardId(42);
    let catalog = catalog_with(vec![card(42, 2, 1)]);
    let mut pools = pools_for(player, &catalog);
    pools
        .pools
        .get_mut(&player)
        .expect("pool exists")
        .distribute(card_id)
        .expect("fixture should exhaust the only copy");
    let mut shops = ShopStates {
        players: HashMap::from([(
            player,
            shop_state(ShopPhase::ShopActive, [Some(card_id), None, None]),
        )]),
    };
    let mut hands = PlayerHands::default();
    let mut economies = PlayerEconomies(HashMap::from([(player, economy(5))]));

    let (result, update) = process_purchase_card(
        &mut shops,
        &mut hands,
        &mut economies,
        &mut pools,
        &catalog,
        player,
        card_id,
    );

    assert_eq!(result, PurchaseAttemptResult::CardUnavailable);
    assert!(update.is_none());
    assert_eq!(gold(&economies, player), 5);
    assert_eq!(hands.hand_len(player), 0);
    assert_eq!(
        shops
            .players
            .get(&player)
            .expect("shop exists")
            .current_slots[0],
        Some(card_id)
    );
}

#[test]
fn ca14_successful_purchase_spends_distributes_and_removes_slot() {
    test_helpers::init_test_tracing();
    let player = PlayerId(1);
    let card_id = CardId(42);
    let catalog = catalog_with(vec![card(42, 3, 1)]);
    let mut pools = pools_for(player, &catalog);
    let mut shops = ShopStates {
        players: HashMap::from([(
            player,
            shop_state(
                ShopPhase::ShopActive,
                [Some(CardId(10)), Some(card_id), Some(CardId(11))],
            ),
        )]),
    };
    let mut hands = PlayerHands::default();
    let mut economies = PlayerEconomies(HashMap::from([(player, economy(5))]));
    let source = purchase_card_source(&shops, player, card_id);

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
    assert_eq!(hands.hands.get(&player), Some(&vec![card_id]));
    assert_eq!(gold(&economies, player), 2);
    assert_eq!(
        pools
            .pools
            .get(&player)
            .expect("pool exists")
            .copies_remaining(card_id),
        0
    );
    assert_eq!(
        shops
            .players
            .get(&player)
            .expect("shop exists")
            .current_slots,
        [Some(CardId(10)), None, Some(CardId(11))]
    );
    assert_eq!(
        update
            .clone()
            .expect("successful shop purchase emits slots")
            .slots,
        vec![Some(CardId(10)), None, Some(CardId(11))]
    );

    let events = purchase_network_events_for_result(
        result, player, card_id, source, &economies, update, None,
    )
    .expect("successful shop purchase should emit network events");
    assert_eq!(events.card_acquired.message.card_id, card_id);
    assert_eq!(
        events.card_acquired.message.source,
        CardSource::ShopPurchase
    );
    assert_eq!(events.gold_update.player, player);
    assert_eq!(events.gold_update.gold, 2);
    assert_eq!(
        events
            .shop_slots
            .expect("shop purchase should include slot update")
            .message
            .slots,
        vec![Some(CardId(10)), None, Some(CardId(11))]
    );
}

#[test]
fn playable_002_draft_initial_purchase_emits_card_acquired_and_gold_without_slots() {
    test_helpers::init_test_tracing();
    let player = PlayerId(1);
    let card_id = CardId(42);
    let catalog = catalog_with(vec![card(42, 3, 1)]);
    let mut pools = pools_for(player, &catalog);
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
    let source = purchase_card_source(&shops, player, card_id);

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
    let events = purchase_network_events_for_result(
        result, player, card_id, source, &economies, update, None,
    )
    .expect("successful initial draft purchase should emit network events");
    assert_eq!(events.card_acquired.message.card_id, card_id);
    assert_eq!(
        events.card_acquired.message.source,
        CardSource::DraftInitial
    );
    assert_eq!(events.gold_update.gold, 2);
    assert!(events.shop_slots.is_none());
}

#[test]
fn playable_002_rejected_purchase_emits_no_acquisition_or_gold_event() {
    test_helpers::init_test_tracing();
    let player = PlayerId(1);
    let card_id = CardId(42);
    let catalog = catalog_with(vec![card(42, 2, 1)]);
    let mut pools = pools_for(player, &catalog);
    let mut shops = ShopStates {
        players: HashMap::from([(
            player,
            shop_state(ShopPhase::Inactive, [Some(card_id), None, None]),
        )]),
    };
    let mut hands = PlayerHands::default();
    let mut economies = PlayerEconomies(HashMap::from([(player, economy(5))]));
    let source = purchase_card_source(&shops, player, card_id);

    let (result, update) = process_purchase_card(
        &mut shops,
        &mut hands,
        &mut economies,
        &mut pools,
        &catalog,
        player,
        card_id,
    );

    assert_eq!(result, PurchaseAttemptResult::DiscardedWrongPhase);
    assert!(purchase_network_events_for_result(
        result, player, card_id, source, &economies, update, None,
    )
    .is_none());
    assert_eq!(gold(&economies, player), 5);
    assert_eq!(hands.hand_len(player), 0);
}

#[test]
fn ca18_distribute_failure_refunds_gold_and_leaves_slot() {
    test_helpers::init_test_tracing();
    let player = PlayerId(1);
    let card_id = CardId(42);
    let catalog = catalog_with(vec![card(42, 2, 1)]);
    let mut shops = ShopStates {
        players: HashMap::from([(
            player,
            shop_state(ShopPhase::ShopActive, [Some(card_id), None, None]),
        )]),
    };
    let mut hands = PlayerHands::default();
    let mut economies = PlayerEconomies(HashMap::from([(player, economy(5))]));
    let mut pool = ExhaustingPurchasePool {
        distribute_calls: 0,
    };

    let (result, update) = process_purchase_card_with_pool(
        &mut shops,
        &mut hands,
        &mut economies,
        &mut pool,
        &catalog,
        player,
        card_id,
    );

    assert_eq!(result, PurchaseAttemptResult::DistributeExhausted);
    assert!(update.is_none());
    assert_eq!(pool.distribute_calls, 1);
    assert_eq!(gold(&economies, player), 5);
    assert_eq!(hands.hand_len(player), 0);
    assert_eq!(
        shops
            .players
            .get(&player)
            .expect("shop exists")
            .current_slots[0],
        Some(card_id)
    );
}

#[test]
fn ca20_wrong_phase_discards_stale_purchase_without_mutation() {
    test_helpers::init_test_tracing();
    let player = PlayerId(1);
    let card_id = CardId(42);
    let catalog = catalog_with(vec![card(42, 2, 1)]);
    let mut pools = pools_for(player, &catalog);
    let mut shops = ShopStates {
        players: HashMap::from([(
            player,
            shop_state(ShopPhase::Inactive, [Some(card_id), None, None]),
        )]),
    };
    let mut hands = PlayerHands::default();
    let mut economies = PlayerEconomies(HashMap::from([(player, economy(5))]));

    let (result, update) = process_purchase_card(
        &mut shops,
        &mut hands,
        &mut economies,
        &mut pools,
        &catalog,
        player,
        card_id,
    );

    assert_eq!(result, PurchaseAttemptResult::DiscardedWrongPhase);
    assert!(update.is_none());
    assert_eq!(gold(&economies, player), 5);
    assert_eq!(hands.hand_len(player), 0);
    assert_eq!(
        shops
            .players
            .get(&player)
            .expect("shop exists")
            .current_slots[0],
        Some(card_id)
    );
    assert_eq!(
        pools
            .pools
            .get(&player)
            .expect("pool exists")
            .copies_remaining(card_id),
        1
    );
}
