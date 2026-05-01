use std::collections::HashMap;

use bevy::prelude::*;
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::rsm::{AbortAuction, AuctionPhaseEntered, AuctionSettled};
use server::feature::auction::{auction_tick_system, AuctionPhase, AuctionState, S2CAuctionCard};
use server::foundation::config::{CardCatalog, GameConfig};
use shared::card::CardId;
use shared::session::PlayerId;

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

fn app_with_state(state: AuctionState, economies: PlayerEconomies) -> App {
    let mut app = App::new();
    app.add_message::<AuctionPhaseEntered>()
        .add_message::<AbortAuction>()
        .add_message::<AuctionSettled>()
        .add_message::<S2CAuctionCard>()
        .insert_resource(state)
        .insert_resource(economies)
        .insert_resource(CardCatalog {
            cards: HashMap::new(),
        })
        .insert_resource(GameConfig(shared::config::GameConfig::default()))
        .add_systems(Update, auction_tick_system);
    app
}

fn send_abort(app: &mut App) {
    app.world_mut()
        .resource_mut::<Messages<AbortAuction>>()
        .write(AbortAuction);
}

#[test]
fn abort_in_live_bidding_releases_reservation_and_returns_idle() {
    let leader = PlayerId(1);
    let mut economies = PlayerEconomies::default();
    economies.0.insert(leader, economy(10, 5));

    let mut app = app_with_state(
        AuctionState {
            phase: AuctionPhase::LiveBidding,
            card_id: Some(CardId(2)),
            starting_price: 3,
            current_price: 6,
            current_leader: Some(leader),
            timer_remaining_ms: 5_000,
        },
        economies,
    );
    send_abort(&mut app);

    app.update();

    let state = app.world().resource::<AuctionState>();
    assert_eq!(state.phase, AuctionPhase::Idle);
    assert_eq!(state.card_id, None);
    assert_eq!(state.starting_price, 0);
    assert_eq!(state.current_price, 0);
    assert_eq!(state.current_leader, None);
    assert_eq!(state.timer_remaining_ms, 0);

    let economies = app.world().resource::<PlayerEconomies>();
    assert_eq!(
        economies
            .0
            .get(&leader)
            .expect("leader exists")
            .reserved_gold,
        0
    );
    assert!(read_messages::<AuctionSettled>(&app).is_empty());
}

#[test]
fn abort_in_live_bidding_without_leader_returns_idle_without_release() {
    let player = PlayerId(1);
    let mut economies = PlayerEconomies::default();
    economies.0.insert(player, economy(10, 0));

    let mut app = app_with_state(
        AuctionState {
            phase: AuctionPhase::LiveBidding,
            card_id: Some(CardId(2)),
            starting_price: 3,
            current_price: 3,
            current_leader: None,
            timer_remaining_ms: 5_000,
        },
        economies,
    );
    send_abort(&mut app);

    app.update();

    let state = app.world().resource::<AuctionState>();
    assert_eq!(state.phase, AuctionPhase::Idle);

    let economies = app.world().resource::<PlayerEconomies>();
    assert_eq!(
        economies
            .0
            .get(&player)
            .expect("player exists")
            .reserved_gold,
        0
    );
    assert!(read_messages::<AuctionSettled>(&app).is_empty());
}

#[test]
fn abort_in_selecting_returns_idle_without_settlement() {
    let player = PlayerId(1);
    let mut economies = PlayerEconomies::default();
    economies.0.insert(player, economy(10, 0));

    let mut app = app_with_state(
        AuctionState {
            phase: AuctionPhase::Selecting,
            card_id: None,
            starting_price: 0,
            current_price: 0,
            current_leader: None,
            timer_remaining_ms: 0,
        },
        economies,
    );
    send_abort(&mut app);

    app.update();

    assert_eq!(
        app.world().resource::<AuctionState>().phase,
        AuctionPhase::Idle
    );
    let economies = app.world().resource::<PlayerEconomies>();
    assert!(economies
        .0
        .values()
        .all(|economy| economy.reserved_gold == 0));
    assert!(read_messages::<AuctionSettled>(&app).is_empty());
}

#[test]
#[ignore = "pending AUC-006 resolution settlement implementation"]
fn abort_in_resolving_is_uninterruptible_and_settlement_completes() {
    let _ = AuctionState {
        phase: AuctionPhase::Resolving,
        card_id: Some(CardId(9)),
        starting_price: 5,
        current_price: 7,
        current_leader: Some(PlayerId(2)),
        timer_remaining_ms: 0,
    };
}
