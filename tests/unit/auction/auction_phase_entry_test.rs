use std::collections::HashMap;

use bevy::prelude::*;
use server::core::economy::PlayerEconomies;
use server::core::rsm::{AbortAuction, AuctionPhaseEntered, BroadcastPhaseChanged};
use server::feature::auction::{
    auction_tick_system, AuctionCardDrawFixture, AuctionPhase, AuctionState, S2CAuctionCard,
};
use server::foundation::config::{CardCatalog, GameConfig};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::session::PlayerId;

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
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

fn app_for_card(card: CardData) -> App {
    let card_id = card.id;
    let mut app = App::new();
    app.add_message::<AuctionPhaseEntered>()
        .add_message::<AbortAuction>()
        .add_message::<S2CAuctionCard>()
        .add_message::<BroadcastPhaseChanged>()
        .insert_resource(AuctionState::default())
        .insert_resource(PlayerEconomies::default())
        .insert_resource(AuctionCardDrawFixture::with_card(card_id))
        .insert_resource(catalog_with(card))
        .insert_resource(GameConfig(shared::config::GameConfig::default()))
        .add_systems(Update, auction_tick_system);
    app
}

fn send_phase_entered(app: &mut App) {
    app.world_mut()
        .resource_mut::<Messages<AuctionPhaseEntered>>()
        .write(AuctionPhaseEntered { round: 3 });
}

#[test]
fn phase_entry_initialises_rare_auction_state() {
    let mut app = app_for_card(make_card(10, Rarity::Rare));
    send_phase_entered(&mut app);

    app.update();

    let state = app.world().resource::<AuctionState>();
    assert_eq!(state.phase, AuctionPhase::LiveBidding);
    assert_eq!(state.card_id, Some(CardId(10)));
    assert_eq!(state.starting_price, 3);
    assert_eq!(state.current_price, 3);
    assert_eq!(state.current_leader, None);
    assert_eq!(state.timer_remaining_ms, 20_000);
}

#[test]
fn phase_entry_uses_rarity_starting_floor() {
    for (rarity, expected_price) in [(Rarity::Rare, 3), (Rarity::Epic, 4), (Rarity::Legendary, 5)] {
        let mut app = app_for_card(make_card(expected_price, rarity));
        send_phase_entered(&mut app);

        app.update();

        let state = app.world().resource::<AuctionState>();
        assert_eq!(state.phase, AuctionPhase::LiveBidding);
        assert_eq!(state.current_price, expected_price);
        assert_eq!(state.starting_price, expected_price);
    }
}

#[test]
fn auction_card_is_queued_without_phase_changed_message() {
    let mut app = app_for_card(make_card(42, Rarity::Rare));
    send_phase_entered(&mut app);

    app.update();

    let auction_cards = read_messages::<S2CAuctionCard>(&app);
    assert_eq!(
        auction_cards,
        vec![S2CAuctionCard {
            card_id: CardId(42),
            starting_price: 3,
        }]
    );
    assert!(read_messages::<BroadcastPhaseChanged>(&app).is_empty());
}

#[test]
fn duplicate_phase_entry_in_non_idle_state_is_rejected() {
    for phase in [
        AuctionPhase::Selecting,
        AuctionPhase::LiveBidding,
        AuctionPhase::Resolving,
    ] {
        let mut app = app_for_card(make_card(7, Rarity::Epic));
        *app.world_mut().resource_mut::<AuctionState>() = AuctionState {
            phase,
            card_id: Some(CardId(5)),
            starting_price: 4,
            current_price: 4,
            current_leader: Some(PlayerId(1)),
            timer_remaining_ms: 8_000,
        };
        send_phase_entered(&mut app);

        app.update();

        let state = app.world().resource::<AuctionState>();
        assert_eq!(state.phase, phase);
        assert_eq!(state.card_id, Some(CardId(5)));
        assert_eq!(state.starting_price, 4);
        assert_eq!(state.current_price, 4);
        assert_eq!(state.current_leader, Some(PlayerId(1)));
        assert_eq!(state.timer_remaining_ms, 8_000);
        assert!(read_messages::<S2CAuctionCard>(&app).is_empty());
    }
}
