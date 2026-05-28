use std::collections::HashMap;
use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::{prelude::*, time::Virtual};
use bevy_tweening::{TweenAnim, TweeningPlugin};
use client::asset_wiring::{
    CardDisplayArtAsset, CardDisplayArtFallback, CardDisplayArtFallbackReason,
};
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    compute_fan_slot_layout, FanLayoutMetrics, FanSlotIndex, GridSlotCard, GridSlotCardName,
    GridSlotIndex, GridSlotManaCost, GridSlotState, HandCardCatalog, HandContents,
    HandFullNotification, HandGridCardClicked, HandSlotCard, HandUiCardAcquiredReceived,
    HandUiDraftOfferingReceived, HandUiEntities, HandUiOutboundMessages, HandUiPlugin,
    HandUiTimingConfig, NotificationTimer, PendingPurchaseTimer,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const EPSILON: f32 = 0.001;

#[test]
fn hu_07_draft_offering_populates_nine_visible_grid_slots() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_draft_initial();
    let initial_ids = card_ids(1, 9);

    send_offering(&mut app, initial_ids.clone());

    assert_eq!(visible_grid_slot_count(&mut app), 9);
    for (index, card_id) in initial_ids.into_iter().enumerate() {
        let slot = grid_slot(&mut app, index);
        assert_eq!(
            app.world().get::<Visibility>(slot),
            Some(&Visibility::Visible)
        );
        assert_eq!(
            app.world().get::<GridSlotCard>(slot),
            Some(&GridSlotCard(card_id))
        );
        assert_eq!(
            app.world().get::<GridSlotCardName>(slot),
            Some(&GridSlotCardName(format!("Card {}", card_id.0)))
        );
        assert_eq!(
            app.world().get::<GridSlotManaCost>(slot),
            Some(&GridSlotManaCost(card_cost(card_id)))
        );
        assert_eq!(
            app.world().get::<GridSlotState>(slot),
            Some(&GridSlotState::Available)
        );
    }

    send_offering(&mut app, card_ids(10, 9));
    let slot_zero = grid_slot(&mut app, 0);
    assert_eq!(
        app.world().get::<GridSlotCard>(slot_zero),
        Some(&GridSlotCard(CardId(10)))
    );
    assert_eq!(
        app.world().get::<GridSlotCardName>(slot_zero),
        Some(&GridSlotCardName("Card 10".to_string()))
    );
}

#[test]
fn hu_asset_loop_draft_and_fan_slots_resolve_card_display_art_or_fallback() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_draft_initial();

    send_offering(&mut app, vec![CardId(1)]);

    let known_art_slot = grid_slot(&mut app, 0);
    assert_eq!(
        app.world().get::<CardDisplayArtAsset>(known_art_slot),
        Some(&CardDisplayArtAsset {
            path: "art/cards/display/card_iop_knight_001_art_display.png".to_string()
        })
    );

    app.world_mut()
        .write_message(HandUiCardAcquiredReceived { card_id: CardId(1) });
    run_update(&mut app);

    let known_fan_slot = fan_slot(&mut app, 0);
    assert_eq!(
        app.world().get::<CardDisplayArtAsset>(known_fan_slot),
        Some(&CardDisplayArtAsset {
            path: "art/cards/display/card_iop_knight_001_art_display.png".to_string()
        })
    );

    // CardId(999) is intentionally absent from the test catalog (1..=18);
    // the catalog miss feeds None into apply_card_display_art on the fan
    // slot, producing the MissingDisplayAsset fallback. The grid slot
    // path warns-and-hides on a catalog miss, so the fan path is the
    // only place this fallback can land in hand UI.
    app.world_mut().write_message(HandUiCardAcquiredReceived {
        card_id: CardId(999),
    });
    run_update(&mut app);

    let missing_art_fan_slot = fan_slot(&mut app, 1);
    assert_eq!(
        app.world()
            .get::<CardDisplayArtFallback>(missing_art_fan_slot),
        Some(&CardDisplayArtFallback {
            reason: CardDisplayArtFallbackReason::MissingDisplayAsset
        })
    );
}

#[test]
fn hu_08_confirmed_purchase_hides_grid_slot_and_animates_card_to_fan() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_draft_initial();
    send_offering(&mut app, card_ids(1, 9));

    let slot_three = grid_slot(&mut app, 3);
    click_grid_slot(&mut app, slot_three);
    assert_eq!(
        app.world().get::<GridSlotState>(slot_three),
        Some(&GridSlotState::Pending)
    );

    app.world_mut()
        .write_message(HandUiCardAcquiredReceived { card_id: CardId(4) });
    run_update(&mut app);

    assert_eq!(
        app.world().get::<Visibility>(slot_three),
        Some(&Visibility::Hidden)
    );
    assert!(app.world().get::<GridSlotCard>(slot_three).is_none());

    let fan_slot = fan_slot(&mut app, 0);
    assert_eq!(
        app.world().get::<Visibility>(fan_slot),
        Some(&Visibility::Visible)
    );
    assert_eq!(
        app.world().get::<HandSlotCard>(fan_slot),
        Some(&HandSlotCard(CardId(4)))
    );
    assert!(
        app.world().get::<TweenAnim>(fan_slot).is_some(),
        "confirmed purchase should attach a Transform tween"
    );

    run_for(&mut app, Duration::from_millis(280));
    let expected = compute_fan_slot_layout(0, 1, qa_metrics()).expect("fan layout should exist");
    let transform = app
        .world()
        .get::<Transform>(fan_slot)
        .expect("fan slot should have Transform");
    assert_approx(transform.translation.x, expected.card_x);
    assert_approx(transform.translation.y, expected.card_y);
}

#[test]
fn hu_09_and_hu_10c_hand_full_locks_all_remaining_visible_grid_slots() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_draft_initial();
    set_hand_to_nine_cards(&mut app);
    send_offering(&mut app, card_ids(1, 9));

    let pending_slot = grid_slot(&mut app, 8);
    click_grid_slot(&mut app, pending_slot);
    assert_eq!(
        app.world().get::<GridSlotState>(pending_slot),
        Some(&GridSlotState::Pending)
    );

    app.world_mut()
        .write_message(HandUiCardAcquiredReceived { card_id: CardId(1) });
    run_update(&mut app);

    assert_eq!(app.world().resource::<HandContents>().cards.len(), 10);
    for slot in visible_grid_slots(&mut app) {
        assert_eq!(
            app.world().get::<GridSlotState>(slot),
            Some(&GridSlotState::HandFullLocked)
        );
        assert!(app.world().get::<PendingPurchaseTimer>(slot).is_none());
    }
    assert_eq!(
        app.world().get::<GridSlotState>(pending_slot),
        Some(&GridSlotState::HandFullLocked)
    );

    let purchases_before = app
        .world()
        .resource::<HandUiOutboundMessages>()
        .purchase_cards
        .len();
    click_grid_slot(&mut app, pending_slot);
    assert_eq!(
        app.world()
            .resource::<HandUiOutboundMessages>()
            .purchase_cards
            .len(),
        purchases_before,
        "locked grid slots must suppress purchase sends"
    );
}

#[test]
fn hu_10_pending_purchase_times_out_without_deducting_gold_and_can_retry() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_draft_initial();
    send_offering(&mut app, card_ids(1, 9));

    let slot_two = grid_slot(&mut app, 2);
    click_grid_slot(&mut app, slot_two);
    assert_eq!(
        app.world().get::<GridSlotState>(slot_two),
        Some(&GridSlotState::Pending)
    );
    assert_eq!(
        app.world()
            .resource::<HandUiOutboundMessages>()
            .purchase_cards
            .len(),
        1
    );

    run_for(&mut app, Duration::from_millis(3_001));

    assert_eq!(app.world().resource::<PlayerEconomyView>().gold, 5);
    assert_eq!(
        app.world().get::<GridSlotState>(slot_two),
        Some(&GridSlotState::Available)
    );
    assert!(app.world().get::<PendingPurchaseTimer>(slot_two).is_none());

    click_grid_slot(&mut app, slot_two);
    assert_eq!(
        app.world()
            .resource::<HandUiOutboundMessages>()
            .purchase_cards
            .len(),
        2,
        "slot should accept a fresh retry after timeout"
    );
}

#[test]
fn hu_30_hand_full_notification_uses_prepooled_entity_and_timer_lifecycle() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_draft_initial();
    set_hand_to_nine_cards(&mut app);
    send_offering(&mut app, card_ids(1, 9));

    app.world_mut()
        .write_message(HandUiCardAcquiredReceived { card_id: CardId(1) });
    run_update(&mut app);

    let notification = app
        .world()
        .resource::<HandUiEntities>()
        .hand_full_notification;
    assert_eq!(
        app.world().get::<Visibility>(notification),
        Some(&Visibility::Visible)
    );
    assert_eq!(
        app.world().get::<NotificationTimer>(notification),
        Some(&NotificationTimer {
            remaining_ms: 2_000
        })
    );
    assert!(app
        .world()
        .get::<HandFullNotification>(notification)
        .is_some());

    run_for(&mut app, Duration::from_millis(2_001));

    assert_eq!(
        app.world().get::<Visibility>(notification),
        Some(&Visibility::Hidden)
    );
    assert!(app.world().get::<NotificationTimer>(notification).is_none());
}

fn app_with_hand_ui_in_draft_initial() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(TweeningPlugin);
    app.add_plugins(HandUiPlugin);
    app.insert_resource(HandCardCatalog {
        cards: test_catalog(1..=18),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 5,
        reserve_mana: 0,
        initialized: true,
        ..default()
    });
    app.insert_resource(HandUiTimingConfig {
        card_draw_animation_ms: 280,
        purchase_timeout_ms: 3_000,
        hand_full_notification_duration_ms: 2_000,
    });
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_phase(&mut app, RoundPhase::DraftInitial);
    run_update(&mut app);
    app
}

fn run_update(app: &mut App) {
    run_for(app, Duration::ZERO);
}

fn run_for(app: &mut App, duration: Duration) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(duration);
    app.update();
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn set_hand_to_nine_cards(app: &mut App) {
    app.world_mut().resource_mut::<HandContents>().cards =
        (100..109).map(CardId).collect::<Vec<_>>();
    run_update(app);
}

fn send_offering(app: &mut App, card_ids: Vec<CardId>) {
    app.world_mut()
        .write_message(HandUiDraftOfferingReceived { card_ids });
    run_update(app);
}

fn click_grid_slot(app: &mut App, slot: Entity) {
    app.world_mut()
        .write_message(HandGridCardClicked { card: slot });
    run_update(app);
}

fn card_ids(start: u32, count: u32) -> Vec<CardId> {
    (start..start + count).map(CardId).collect()
}

fn card_cost(card_id: CardId) -> u32 {
    card_id.0 % 5 + 1
}

fn test_catalog(ids: impl IntoIterator<Item = u32>) -> HashMap<CardId, CardData> {
    ids.into_iter()
        .map(|id| {
            let card = test_card(id);
            (card.id, card)
        })
        .collect()
}

fn test_card(id: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: card_cost(CardId(id)),
        atk: 1,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: if id == 1 {
            "iop_knight_001".to_string()
        } else {
            format!("test_{id}")
        },
        pool_copies_override: None,
    }
}

fn grid_slot(app: &mut App, index: usize) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &GridSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 as usize == index).then_some(entity))
        .expect("grid slot should exist")
}

fn fan_slot(app: &mut App, index: usize) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 as usize == index).then_some(entity))
        .expect("fan slot should exist")
}

fn visible_grid_slot_count(app: &mut App) -> usize {
    visible_grid_slots(app).len()
}

fn visible_grid_slots(app: &mut App) -> Vec<Entity> {
    let mut query = app
        .world_mut()
        .query::<(Entity, &Visibility, &GridSlotIndex)>();
    query
        .iter(app.world())
        .filter_map(|(entity, visibility, _)| {
            (*visibility == Visibility::Visible).then_some(entity)
        })
        .collect()
}

fn qa_metrics() -> FanLayoutMetrics {
    // LOCAL-to-fan_root coords per PROMPT 671: fan_base_y is `HAND_FAN_STRIP_HEIGHT_PX
    // - fan_base_margin_px` = 260 - 150 = 110 at the default 800x600 viewport used
    // here (MinimalPlugins, no Window → HandFanViewport stays at default). Compared
    // against `transform.translation.y` written by `apply_fan_layout_system`.
    // PROMPT 1854 (STAGE3-D): fan_base_y updated 160→110 to match the new default
    // fan_base_margin_px = 150 (was 100).
    FanLayoutMetrics {
        fan_center_x: 400.0,
        fan_base_y: 110.0,
        fan_half_spread: 280.0,
        arc_height: 10.0,
        max_rotation_deg: 10.0,
    }
}

fn assert_approx(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < EPSILON,
        "expected {actual} to be within {EPSILON} of {expected}"
    );
}
