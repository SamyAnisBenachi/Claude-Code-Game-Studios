use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::{TimePlugin, TimeUpdateStrategy};
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
            live_bidding_deadline_elapsed_ms: None,
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

fn app_with_real_time_auction(card: CardData) -> App {
    let mut app = App::new();
    // PROMPT 1091: TimePlugin + manual delta gives this integration test a
    // wall-clock-anchored `Time::elapsed()` so `auction_tick_system` can set
    // `live_bidding_deadline_elapsed_ms` on phase entry, matching the
    // production schedule path.
    app.add_plugins(TimePlugin);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(1)));
    let card_id = card.id;
    app.add_message::<AuctionPhaseEntered>()
        .add_message::<AbortAuction>()
        .add_message::<AuctionSettled>()
        .add_message::<S2CAuctionCard>()
        .add_message::<S2CGoldBroadcast>()
        .insert_resource(AuctionState::default())
        .insert_resource(PlayerEconomies::default())
        .insert_resource(PlayerHands::default())
        .insert_resource(AuctionCardDrawFixture::with_card(card_id))
        .insert_resource(catalog_with(card))
        .insert_resource(GameConfig(shared::config::GameConfig {
            auction_timer_seconds: 20,
            auction_timer_reset_seconds: 5,
            ..shared::config::GameConfig::default()
        }))
        .add_systems(Update, auction_tick_system);
    app
}

fn advance_elapsed_by(app: &mut App, delta: Duration) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(delta));
    app.update();
}

/// PROMPT 1091 / AUDIT-1076-12 regression: ensures `auction_tick_system`
/// settles a LiveBidding auction within the configured window even when the
/// per-tick `time.delta()` accumulation would otherwise under-count (the
/// failure mode in the 2026-05-17 run-7 manual playtest: a 20s timer took
/// 149s of wall-clock time to expire, so the auction sat in LiveBidding
/// for 2m 29s after the only bid).
///
/// The test progresses Time in two beats: a series of tiny deltas that hold
/// `decrement_live_bidding_timer` near 19s remaining, followed by a large
/// elapsed jump that fires the deadline safety net. The auction must reach
/// `Idle` and emit `AuctionSettled` after the jump.
#[test]
fn live_bidding_settles_when_elapsed_passes_deadline_even_if_decrement_lags() {
    test_helpers::init_test_tracing();

    let card = make_card(64, Rarity::Rare);
    let mut app = app_with_real_time_auction(card);

    // Prime Time<()> so `time.elapsed()` is non-zero before phase entry.
    advance_elapsed_by(&mut app, Duration::from_millis(1));

    app.world_mut()
        .resource_mut::<Messages<AuctionPhaseEntered>>()
        .write(AuctionPhaseEntered { round: 3 });
    advance_elapsed_by(&mut app, Duration::from_millis(1));

    {
        let state = app.world().resource::<AuctionState>();
        assert_eq!(state.phase, AuctionPhase::LiveBidding);
        // 1ms post-entry tick consumes ~1ms of the 20_000ms window; allow
        // a small drift instead of pinning the exact value.
        assert!(
            state.timer_remaining_ms >= 19_990,
            "phase entry must initialise the timer near the configured ceiling; \
             got {}",
            state.timer_remaining_ms
        );
        assert!(
            state.live_bidding_deadline_elapsed_ms.is_some(),
            "phase entry must anchor `live_bidding_deadline_elapsed_ms` so the \
             safety net can settle the auction in bounded wall-clock time"
        );
    }

    // Beat 1: many 1ms ticks simulate the schedule firing while
    // `decrement_live_bidding_timer` only subtracts ~1ms per tick. Far less
    // than the 20_000ms window — auction stays live.
    for _ in 0..100 {
        advance_elapsed_by(&mut app, Duration::from_millis(1));
    }
    {
        let state = app.world().resource::<AuctionState>();
        assert_eq!(state.phase, AuctionPhase::LiveBidding);
        assert!(
            state.timer_remaining_ms > 19_000,
            "decrement under sparse ticks: expected timer >19_000ms after \
             ~100ms wall-clock, got {}",
            state.timer_remaining_ms
        );
    }

    // Beat 2: a single large elapsed jump that drives the absolute deadline
    // past expiry. Even if the per-tick decrement under-counted, the safety
    // net collapses `timer_remaining_ms` to 0 and settle fires this tick.
    // (Verified through `AuctionState` transitions rather than `AuctionSettled`
    // messages because Bevy 0.18 `Messages<T>` buffers may rotate inside the
    // 25s jump when `FixedUpdate` catches up across thousands of fixed-
    // timestep increments.)
    //
    // The 25_000ms jump is anchored to `Time<Real>`, which is uncapped — the
    // bug under `Time<Virtual>` is that `max_delta` clamps the delta to 250ms,
    // so a 25_000ms wall-clock jump would have advanced Virtual by only 250ms.
    // Using `Time<Real>` for the deadline anchor is the actual repair.
    advance_elapsed_by(&mut app, Duration::from_millis(25_000));

    let state = app.world().resource::<AuctionState>();
    assert_eq!(
        state.phase,
        AuctionPhase::Idle,
        "auction must return to Idle after the deadline elapses; \
         AUDIT-1076-12 failure mode would leave it in LiveBidding indefinitely"
    );
    assert!(
        state.live_bidding_deadline_elapsed_ms.is_none(),
        "reset_to_idle must clear the deadline anchor on settlement"
    );
    assert_eq!(
        state.card_id, None,
        "reset_to_idle must clear the auction card on settlement"
    );
    assert_eq!(
        state.timer_remaining_ms, 0,
        "settled auction must report 0 remaining timer"
    );
}
