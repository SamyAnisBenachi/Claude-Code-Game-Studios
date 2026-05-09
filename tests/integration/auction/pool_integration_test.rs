use std::collections::HashMap;

use bevy::prelude::*;
use server::core::economy::{EconomyPlugin, PlayerEconomies, PlayerEconomy};
use server::core::pool::CardPoolPlugin;
use server::core::rsm::{AbortAuction, AuctionPhaseEntered, AuctionSettled, GameOverEmitted};
use server::feature::acquisition::PlayerHands;
use server::feature::auction::{
    AuctionPhase, AuctionPlugin, AuctionPool, AuctionState, S2CAuctionCard,
};
use server::foundation::config::{CardCatalog, GameConfig};
use server::foundation::rng::ServerRng;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn economy(gold: u32, reserved_gold: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold,
        current_mana: 0,
        reserve_mana: 0,
        mana_cap: 10,
        reserved_gold,
    }
}

fn make_card(id: u32, rarity: Rarity, copies: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Neutral,
        family: Some("AuctionFixture".to_string()),
        rarity,
        card_type: CardType::Minion,
        unit_type: UnitType::Neutral,
        cost: 1,
        atk: 1,
        hp: 1,
        mp: 1,
        ar: 0,
        keywords: vec![],
        effect_text: String::new(),
        art_id: format!("auction_fixture_{id}"),
        pool_copies_override: Some(copies as i32),
    }
}

fn catalog(cards: Vec<CardData>) -> CardCatalog {
    CardCatalog {
        cards: cards.into_iter().map(|card| (card.id, card)).collect(),
    }
}

fn config() -> GameConfig {
    GameConfig(shared::config::GameConfig {
        legendary_pool_entry_round: 6,
        ..shared::config::GameConfig::default()
    })
}

fn auction_fixture(catalog: CardCatalog) -> App {
    let config = config();
    let mut app = App::new();
    app.add_plugins((AuctionPlugin, CardPoolPlugin, EconomyPlugin))
        .add_message::<AuctionPhaseEntered>()
        .add_message::<AbortAuction>()
        .add_message::<GameOverEmitted>()
        .insert_resource(AuctionPool::from_catalog(&catalog, &config))
        .insert_resource(catalog)
        .insert_resource(config)
        .insert_resource(ServerRng::from_seed(7))
        .insert_resource(PlayerHands::default())
        .insert_resource(PlayerEconomies(HashMap::from([
            (player(1), economy(20, 0)),
            (player(2), economy(20, 0)),
        ])));
    app
}

fn enter_auction(app: &mut App, round: u32) {
    app.world_mut()
        .resource_mut::<Messages<AuctionPhaseEntered>>()
        .write(AuctionPhaseEntered { round });
    app.update();
}

fn expire_current_auction(app: &mut App) {
    app.world_mut()
        .resource_mut::<AuctionState>()
        .timer_remaining_ms = 0;
    app.update();
}

fn copies_remaining(app: &App, card_id: CardId) -> u32 {
    app.world()
        .resource::<AuctionPool>()
        .copies_remaining(card_id)
}

fn rare_only_app() -> App {
    auction_fixture(catalog(vec![make_card(10, Rarity::Rare, 3)]))
}

#[test]
fn test_pool_distribution_is_permanent_after_auction_win() {
    let winner = player(1);
    let mut app = rare_only_app();

    enter_auction(&mut app, 3);
    assert_eq!(copies_remaining(&app, CardId(10)), 2);

    {
        let mut state = app.world_mut().resource_mut::<AuctionState>();
        state.current_leader = Some(winner);
        state.current_price = 4;
        state.timer_remaining_ms = 0;
    }
    app.world_mut()
        .resource_mut::<PlayerEconomies>()
        .0
        .get_mut(&winner)
        .expect("winner economy exists")
        .reserved_gold = 4;

    app.update();

    assert_eq!(copies_remaining(&app, CardId(10)), 2);
    assert_eq!(
        app.world().resource::<AuctionState>().phase,
        AuctionPhase::Idle
    );
    assert_eq!(read_messages::<AuctionSettled>(&app).len(), 1);
}

#[test]
fn test_pool_distribution_is_permanent_after_no_bid_outcome() {
    let mut app = rare_only_app();

    enter_auction(&mut app, 3);
    assert_eq!(copies_remaining(&app, CardId(10)), 2);

    expire_current_auction(&mut app);

    assert_eq!(copies_remaining(&app, CardId(10)), 2);
    let settled = read_messages::<AuctionSettled>(&app);
    assert_eq!(settled.len(), 1);
    assert_eq!(settled[0].winner, None);
    assert_eq!(settled[0].final_price, 0);
}

#[test]
fn test_pool_distribution_is_permanent_after_abort_auction() {
    let mut app = rare_only_app();

    enter_auction(&mut app, 3);
    assert_eq!(copies_remaining(&app, CardId(10)), 2);

    app.world_mut()
        .resource_mut::<Messages<AbortAuction>>()
        .write(AbortAuction);
    app.update();

    assert_eq!(copies_remaining(&app, CardId(10)), 2);
    assert_eq!(
        app.world().resource::<AuctionState>().phase,
        AuctionPhase::Idle
    );
    assert!(read_messages::<AuctionSettled>(&app).is_empty());
}

#[test]
fn test_empty_eligible_pool_triggers_immediate_no_card_outcome() {
    let rare_id = CardId(10);
    let legendary_id = CardId(99);
    let mut app = auction_fixture(catalog(vec![
        make_card(rare_id.0, Rarity::Rare, 1),
        make_card(legendary_id.0, Rarity::Legendary, 2),
    ]));
    app.world_mut()
        .resource_mut::<AuctionPool>()
        .pool
        .distribute(rare_id)
        .expect("rare copy can be exhausted for fixture");

    enter_auction(&mut app, 3);

    let settled = read_messages::<AuctionSettled>(&app);
    assert_eq!(settled.len(), 1);
    assert_eq!(settled[0].winner, None);
    assert_eq!(settled[0].final_price, 0);
    assert_eq!(settled[0].card_id, CardId(0));
    assert_eq!(
        app.world().resource::<AuctionState>().phase,
        AuctionPhase::Idle
    );
    assert!(read_messages::<S2CAuctionCard>(&app).is_empty());
    assert_eq!(copies_remaining(&app, legendary_id), 2);
}

#[test]
fn test_round_before_legendary_entry_draws_only_rare_card() {
    let rare_id = CardId(10);
    let legendary_id = CardId(99);
    let mut app = auction_fixture(catalog(vec![
        make_card(rare_id.0, Rarity::Rare, 3),
        make_card(legendary_id.0, Rarity::Legendary, 2),
    ]));

    enter_auction(&mut app, 3);

    let state = app.world().resource::<AuctionState>();
    assert_eq!(state.phase, AuctionPhase::LiveBidding);
    assert_eq!(state.card_id, Some(rare_id));
    let drawn = app
        .world()
        .resource::<CardCatalog>()
        .cards
        .get(&state.card_id.expect("auction card selected"))
        .expect("selected card exists");
    assert_ne!(drawn.rarity, Rarity::Legendary);
    assert_eq!(copies_remaining(&app, rare_id), 2);
    assert_eq!(copies_remaining(&app, legendary_id), 2);
}
