use std::collections::HashMap;

use bevy::prelude::*;
use server::core::economy::{PlayerEconomies, PlayerEconomy, S2CGoldBroadcast};
use server::core::rsm::{AbortAuction, AuctionPhaseEntered, AuctionSettled};
use server::feature::acquisition::PlayerHands;
use server::feature::auction::{
    auction_tick_system, AuctionCardDrawFixture, AuctionPhase, AuctionState, S2CAuctionCard,
};
use server::foundation::config::{CardCatalog, GameConfig};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

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

fn make_card(id: u32, rarity: Rarity) -> CardData {
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
        pool_copies_override: Some(1),
    }
}

fn catalog_with(card: CardData) -> CardCatalog {
    CardCatalog {
        cards: HashMap::from([(card.id, card)]),
    }
}

fn app_with_settling_auction(first: PlayerId, second: PlayerId) -> App {
    let mut app = App::new();
    app.add_message::<AuctionPhaseEntered>()
        .add_message::<AbortAuction>()
        .add_message::<AuctionSettled>()
        .add_message::<S2CAuctionCard>()
        .add_message::<S2CGoldBroadcast>()
        .insert_resource(AuctionState {
            phase: AuctionPhase::LiveBidding,
            card_id: Some(CardId(4)),
            starting_price: 3,
            current_price: 5,
            current_leader: Some(second),
            timer_remaining_ms: 0,
        })
        .insert_resource(PlayerEconomies(HashMap::from([
            (first, economy(10, 0)),
            (second, economy(10, 5)),
        ])))
        .insert_resource(PlayerHands::default())
        .insert_resource(AuctionCardDrawFixture::with_card(CardId(8)))
        .insert_resource(catalog_with(make_card(8, Rarity::Rare)))
        .insert_resource(GameConfig(shared::config::GameConfig::default()))
        .add_systems(Update, auction_tick_system);
    app
}

#[test]
fn test_next_auction_entry_starts_with_zero_reserved_gold_for_all_players() {
    test_helpers::init_test_tracing();
    let first = player(1);
    let second = player(2);
    let mut app = app_with_settling_auction(first, second);

    app.update();

    let settled = read_messages::<AuctionSettled>(&app);
    assert_eq!(settled.len(), 1);
    assert_eq!(settled[0].winner, Some(second));
    assert_eq!(settled[0].final_price, 5);
    assert_eq!(settled[0].card_id, CardId(4));
    assert_eq!(
        app.world().resource::<AuctionState>().phase,
        AuctionPhase::Idle
    );
    assert!(app
        .world()
        .resource::<PlayerEconomies>()
        .0
        .values()
        .all(|economy| economy.reserved_gold == 0));

    app.world_mut()
        .resource_mut::<Messages<AuctionPhaseEntered>>()
        .write(AuctionPhaseEntered { round: 6 });
    app.update();

    let state = app.world().resource::<AuctionState>();
    assert_eq!(state.phase, AuctionPhase::LiveBidding);
    assert_eq!(state.card_id, Some(CardId(8)));
    assert!(app
        .world()
        .resource::<PlayerEconomies>()
        .0
        .values()
        .all(|economy| economy.reserved_gold == 0));
}
