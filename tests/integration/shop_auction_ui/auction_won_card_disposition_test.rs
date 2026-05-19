// PROMPT 1347 / S18-AUCTION-WON-CARD-DISPOSITION-001 (AC14 + AC15)
//
// Client-side integration coverage for the auction-won card disposition
// UI lifecycle. Drives `ShopAuctionUiPlugin` through a real Bevy 0.18
// `App` and asserts:
//
// AC4 — affordance banner appears at PLACEMENT entry for the winner.
// AC5 — newly-acquired marker attaches to the matching hand fan slot.
// AC6 — banner + marker reference the same `card_id`.
// AC7 — loser-side settlement toast names the opponent + price.
// AC9 — both banner + marker clear at PLACEMENT-end on the no-op path.
// AC14 — drag-stage (PendingPlacements entry) clears banner + marker.
// AC15 — opponent-settle toast text contains the "Opponent won … for {N}g"
//        copy.

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::card_animations::CardAnimationsPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::hand::{FanSlotIndex, HandSlotCard, PendingPlacements, PlacementTimer};
use client::ui::shop_auction::{
    AuctionWonAffordanceBanner, AuctionWonAffordanceText, AuctionWonHandMarker, AuctionWonPending,
    ShopAuctionAuctionCardReceived, ShopAuctionCardCatalog, ShopAuctionLocalGoldView,
    ShopAuctionSettledReceived, ShopAuctionSettlementState, ShopAuctionUiEntities,
    ShopAuctionUiPlugin,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlacedCardSubmit, PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const LOCAL_PLAYER: PlayerId = PlayerId(1);
const OPPONENT_PLAYER: PlayerId = PlayerId(2);
const AUCTION_WON_CARD_ID: CardId = CardId(107);

// ============================================================================
// Fixtures
// ============================================================================

fn test_card(id: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: Some("AuctionWonFixture".to_string()),
        rarity: Rarity::Rare,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: 3,
        atk: 1,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: format!("auction_won_fixture_{id}"),
        pool_copies_override: None,
    }
}

fn app_in_active_auction(card_id: CardId, starting_price: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(CardAnimationsPlugin);
    app.add_plugins(ShopAuctionUiPlugin);
    // PROMPT 1347 — the hand-side state that `update_auction_won_pending_system`
    // reads is normally provided by `HandUiPlugin`. We register the two
    // resources directly so the test does not need the full hand UI tree.
    app.init_resource::<PendingPlacements>();
    app.insert_resource(PlacementTimer::default());
    app.insert_resource(ShopAuctionCardCatalog {
        cards: HashMap::from([(card_id, test_card(card_id.0))]),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 20,
        initialized: true,
        ..default()
    });
    app.insert_resource(ShopAuctionLocalGoldView {
        player_id: Some(LOCAL_PLAYER),
        gold: 20,
        reserved_gold: 0,
        initialized: true,
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    set_phase(&mut app, RoundPhase::DraftAuction, 20_000);
    app.world_mut()
        .write_message(ShopAuctionAuctionCardReceived {
            card_id,
            starting_price,
            timer_duration_ms: 20_000,
        });
    run_update(&mut app);
    app
}

fn spawn_fan_slot(app: &mut App, slot_index: u8, card_id: CardId) -> Entity {
    // Bare-minimum stand-in for the hand fan slot. The production
    // `spawn_hand_ui` system spawns slots with extra chrome children; the
    // AC5 marker query only requires `FanSlotIndex` + `HandSlotCard`.
    app.world_mut()
        .spawn((
            Name::new(format!("Test Fan Slot {slot_index}")),
            FanSlotIndex(slot_index),
            HandSlotCard(card_id),
            Node::default(),
            Visibility::Visible,
        ))
        .id()
}

fn set_phase(app: &mut App, phase: RoundPhase, timer_duration_ms: u32) {
    let round = app.world().resource::<CurrentClientPhase>().round + 1;
    {
        let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
        current.phase = phase;
        current.round = round;
    }
    {
        let mut phase_view = app.world_mut().resource_mut::<ClientPhaseView>();
        phase_view.phase = phase;
        phase_view.round_number = round;
        phase_view.timer_duration_ms = timer_duration_ms;
    }
    run_update(app);
}

fn write_settled(app: &mut App, winner: Option<PlayerId>, amount: u32) {
    app.world_mut()
        .write_message(ShopAuctionSettledReceived { winner, amount });
    run_update(app);
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}

fn run_for(app: &mut App, duration: Duration) {
    let mut remaining = duration;
    while remaining > Duration::ZERO {
        let step = remaining.min(Duration::from_millis(50));
        *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
            TimeUpdateStrategy::ManualDuration(step);
        app.update();
        remaining = remaining.saturating_sub(step);
    }
}

fn count_banners(app: &mut App) -> usize {
    let mut q = app.world_mut().query::<&AuctionWonAffordanceBanner>();
    q.iter(app.world()).count()
}

fn count_hand_markers(app: &mut App) -> usize {
    let mut q = app.world_mut().query::<&AuctionWonHandMarker>();
    q.iter(app.world()).count()
}

fn first_marker_parent(app: &mut App) -> Option<Entity> {
    let mut q = app
        .world_mut()
        .query::<(&AuctionWonHandMarker, &ChildOf)>();
    q.iter(app.world()).next().map(|(_, child_of)| child_of.parent())
}

fn first_banner_text(app: &mut App) -> Option<String> {
    let mut q = app
        .world_mut()
        .query::<(&AuctionWonAffordanceText, &Text)>();
    q.iter(app.world())
        .next()
        .map(|(_, text)| text.0.clone())
}

// ============================================================================
// AC4 + AC5 + AC6 — banner + marker spawn at PLACEMENT entry on winner.
// ============================================================================

#[test]
fn ac4_ac5_winner_banner_and_marker_spawn_at_placement_entry() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(AUCTION_WON_CARD_ID, 4);
    let _slot = spawn_fan_slot(&mut app, 0, AUCTION_WON_CARD_ID);

    write_settled(&mut app, Some(LOCAL_PLAYER), 4);

    // Pending state is armed at settle (in DraftAuction), but the AC4
    // banner only renders during PLACEMENT.
    assert_eq!(
        count_banners(&mut app),
        0,
        "AC4: banner must NOT exist in DRAFT_AUCTION phase even after armed pending"
    );
    let pending = app.world().resource::<AuctionWonPending>();
    assert!(
        pending.state.is_some(),
        "AuctionWonPending armed on LocalWinner settle (precondition for AC4 / AC5)"
    );

    // Transition to PLACEMENT — AC4 banner + AC5 marker spawn this frame.
    set_phase(&mut app, RoundPhase::Placement, 12_000);

    assert_eq!(count_banners(&mut app), 1, "AC4: banner spawns at PLACEMENT entry");
    assert_eq!(
        count_hand_markers(&mut app),
        1,
        "AC5: hand-fan marker spawns at PLACEMENT entry"
    );

    // AC6 — banner text references the won card name; marker is child of
    // the fan slot whose HandSlotCard matches the won card_id.
    let banner_text = first_banner_text(&mut app).expect("AC4 banner has text");
    assert!(
        banner_text.contains(&test_card(AUCTION_WON_CARD_ID.0).name_en),
        "AC4 / AC6: banner text must name the won card (got {:?})",
        banner_text
    );
    let marker_parent = first_marker_parent(&mut app).expect("AC5 marker has parent");
    let parent_card = app
        .world()
        .get::<HandSlotCard>(marker_parent)
        .expect("AC5 marker parent has HandSlotCard");
    assert_eq!(
        parent_card.0, AUCTION_WON_CARD_ID,
        "AC6: marker parent fan slot holds the same card_id as the won card"
    );
}

// ============================================================================
// AC14 — drag-stage clears banner + marker (AuctionWonPending.staged_yet).
// ============================================================================

#[test]
fn ac14_staging_won_card_clears_banner_and_marker() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(AUCTION_WON_CARD_ID, 4);
    let _slot = spawn_fan_slot(&mut app, 0, AUCTION_WON_CARD_ID);
    write_settled(&mut app, Some(LOCAL_PLAYER), 4);
    set_phase(&mut app, RoundPhase::Placement, 12_000);

    assert_eq!(count_banners(&mut app), 1);
    assert_eq!(count_hand_markers(&mut app), 1);

    // Simulate drag-drop staging by appending the won card to
    // PendingPlacements. `update_auction_won_pending_system` reads this
    // resource and flips `staged_yet`; the sync systems then despawn
    // the banner + marker.
    app.world_mut()
        .resource_mut::<PendingPlacements>()
        .placements
        .push(PlacedCardSubmit {
            card_id: AUCTION_WON_CARD_ID,
            target: PlayTarget::BoardCell { lane: 1, cell: 1 },
            current_mana_spend: 1,
            reserve_mana_spend: 0,
        });

    run_update(&mut app);

    assert_eq!(
        count_banners(&mut app),
        0,
        "AC14: banner despawns on stage (AuctionWonPending.staged_yet=true)"
    );
    assert_eq!(
        count_hand_markers(&mut app),
        0,
        "AC14: hand marker despawns on stage"
    );

    // Pending state is still present (block stays in snapshot until submit
    // or phase exit per AC11) — AC4/AC5 affordance is gated by !staged_yet.
    let pending = app.world().resource::<AuctionWonPending>();
    assert!(
        pending.state.is_some_and(|s| s.staged_yet),
        "AC14 / AC11: AuctionWonPending stays armed with staged_yet=true after stage"
    );
}

// ============================================================================
// AC9 — banner + marker clear at phase exit on the no-op path.
// ============================================================================

#[test]
fn ac9_banner_and_marker_clear_at_phase_exit_no_op_path() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(AUCTION_WON_CARD_ID, 4);
    let _slot = spawn_fan_slot(&mut app, 0, AUCTION_WON_CARD_ID);
    write_settled(&mut app, Some(LOCAL_PLAYER), 4);
    set_phase(&mut app, RoundPhase::Placement, 12_000);

    assert_eq!(count_banners(&mut app), 1, "precondition: banner spawned");
    assert_eq!(count_hand_markers(&mut app), 1, "precondition: marker spawned");

    // No stage, no submit — phase advances to RESOLUTION.
    set_phase(&mut app, RoundPhase::Resolution, 5_000);

    assert_eq!(
        count_banners(&mut app),
        0,
        "AC9: banner cleared at phase exit on no-op path"
    );
    assert_eq!(
        count_hand_markers(&mut app),
        0,
        "AC9: marker cleared at phase exit on no-op path"
    );
    let pending = app.world().resource::<AuctionWonPending>();
    assert!(
        pending.state.is_none(),
        "AC9: AuctionWonPending cleared on phase change to Resolution"
    );
}

// ============================================================================
// AC9 (re-staging) — marker does NOT re-appear in a later PLACEMENT after
// the disposition has been cleared.
// ============================================================================

#[test]
fn ac9_marker_does_not_reappear_after_clear() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(AUCTION_WON_CARD_ID, 4);
    let _slot = spawn_fan_slot(&mut app, 0, AUCTION_WON_CARD_ID);
    write_settled(&mut app, Some(LOCAL_PLAYER), 4);
    set_phase(&mut app, RoundPhase::Placement, 12_000);
    set_phase(&mut app, RoundPhase::Resolution, 5_000);
    // Now the AuctionWonPending is Idle. The same card is still in the
    // fan slot, but a later PLACEMENT must NOT re-spawn the marker.
    set_phase(&mut app, RoundPhase::Placement, 10_000);

    assert_eq!(
        count_banners(&mut app),
        0,
        "AC9: banner one-shot — does not re-appear after clear"
    );
    assert_eq!(
        count_hand_markers(&mut app),
        0,
        "AC9: marker one-shot — does not re-appear after clear"
    );
}

// ============================================================================
// AC15 / AC7 — opponent-settle toast text names the opponent + price.
// ============================================================================

#[test]
fn ac15_opponent_settled_toast_text_includes_price() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(AUCTION_WON_CARD_ID, 4);

    write_settled(&mut app, Some(OPPONENT_PLAYER), 8);

    // The existing settlement_overlay surface owns the loser-side toast.
    // Read its text via the ShopAuctionUiEntities handle.
    let entities = app.world().resource::<ShopAuctionUiEntities>();
    let overlay_text = app
        .world()
        .get::<Text>(entities.settlement_overlay_text)
        .expect("settlement overlay text exists")
        .0
        .clone();
    assert!(
        overlay_text.contains("Opponent") && overlay_text.contains("8"),
        "AC7 / AC15: opponent-settled toast must name opponent + price (got {:?})",
        overlay_text
    );

    // Sanity-check: AuctionWonPending stayed Idle (loser never arms it).
    let pending = app.world().resource::<AuctionWonPending>();
    assert!(
        pending.state.is_none(),
        "AC15: loser client must NOT arm AuctionWonPending"
    );

    // Sanity-check: settlement state captured the opponent outcome.
    let settlement = app.world().resource::<ShopAuctionSettlementState>();
    use client::ui::shop_auction::ShopAuctionSettlementOutcome;
    assert_eq!(
        settlement.outcome,
        Some(ShopAuctionSettlementOutcome::OpponentWinner),
    );
    assert_eq!(settlement.amount, 8);

    // Suppress unused-time warning for `run_for`.
    let _ = run_for;
}
