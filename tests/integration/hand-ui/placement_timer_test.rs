use std::collections::HashMap;
use std::time::Duration;

use bevy::ecs::message::MessageCursor;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::card_animations::HandDragSprite;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::{
    hand::{
        FanSlotIndex, FanSlotState, HandCardCatalog, HandContents, HandSubmitButton,
        HandSubmitButtonClicked, HandUiOutboundMessages, HandUiPlacementDragStarted,
        HandUiPlacementDropResolved, HandUiPlugin, PlacementTimer, TimerState,
        TimerSubmittedCheckmark, TimerUrgencyAudio,
    },
    shared::BoardLayout,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[test]
fn hu_15_grace_window_expiry_submits_staged_cards_and_cancels_drag() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([
        (CardId(10), CardType::Minion),
        (CardId(11), CardType::Minion),
        (CardId(12), CardType::Minion),
    ]));
    set_hand(&mut app, [CardId(10), CardId(11), CardId(12)]);
    stage_card(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );
    stage_card(
        &mut app,
        1,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 2 },
    );
    start_drag(&mut app, 2, PlayerId(7));
    app.world_mut()
        .resource_mut::<PlacementTimer>()
        .remaining_ms = 0;

    run_for(&mut app, Duration::from_millis(201));

    let submissions = submissions(&app);
    assert_eq!(submissions.len(), 1);
    assert_eq!(submissions[0].placements.len(), 2);
    assert!(submissions[0]
        .placements
        .iter()
        .all(|placement| placement.card_id != CardId(12)));
    assert_drag_visibility(&mut app, Visibility::Hidden);
    let slot = fan_slot(&mut app, 2);
    assert_eq!(
        app.world().get::<FanSlotState>(slot),
        Some(&FanSlotState::Active)
    );
}

#[test]
fn hu_15b_valid_drop_during_grace_submits_extended_staged_set() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([
        (CardId(20), CardType::Minion),
        (CardId(21), CardType::Minion),
        (CardId(22), CardType::Minion),
    ]));
    set_hand(&mut app, [CardId(20), CardId(21), CardId(22)]);
    stage_card(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );
    stage_card(
        &mut app,
        1,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 2 },
    );
    start_drag(&mut app, 2, PlayerId(7));
    app.world_mut()
        .resource_mut::<PlacementTimer>()
        .remaining_ms = 0;
    run_update(&mut app);

    let slot = fan_slot(&mut app, 2);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id: PlayerId(7),
        target: Some(PlayTarget::BoardCell { lane: 1, cell: 3 }),
    });
    run_update(&mut app);

    let submissions = submissions(&app);
    assert_eq!(submissions.len(), 1);
    assert_eq!(submissions[0].placements.len(), 3);
    assert!(submissions[0]
        .placements
        .iter()
        .any(|placement| placement.card_id == CardId(22)
            && placement.target == PlayTarget::BoardCell { lane: 1, cell: 3 }));
}

#[test]
fn hu_22_urgency_threshold_sets_state_and_writes_audio_once() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(30), CardType::Minion)]));
    let timer_entity = timer_entity(&mut app);
    {
        let mut timer = app.world_mut().resource_mut::<PlacementTimer>();
        timer.remaining_ms = 5_001;
        timer.urgency_fired = false;
    }

    run_for(&mut app, Duration::from_millis(2));

    assert_eq!(
        app.world().get::<TimerState>(timer_entity),
        Some(&TimerState::Urgent)
    );
    assert!(app.world().resource::<PlacementTimer>().urgency_fired);
    let mut cursor = drained_urgency_cursor(&app);
    assert_eq!(all_urgency_messages(&app).len(), 1);

    run_for(&mut app, Duration::from_millis(500));

    assert!(urgency_messages_since(&app, &mut cursor).is_empty());
}

#[test]
fn hu_23_submit_checkmark_visible_while_timer_keeps_running() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(40), CardType::Minion)]));
    app.world_mut()
        .resource_mut::<PlacementTimer>()
        .remaining_ms = 7_000;

    click_submit_for(&mut app, Duration::from_millis(16));

    assert_eq!(submissions(&app).len(), 1);
    let checkmark = submitted_checkmark(&mut app);
    assert_eq!(
        app.world().get::<Visibility>(checkmark),
        Some(&Visibility::Visible)
    );
    assert!(app.world().resource::<PlacementTimer>().remaining_ms < 7_000);

    run_for(&mut app, Duration::from_millis(7_000));

    assert_eq!(
        submissions(&app).len(),
        1,
        "timer expiry after manual submit must not send a duplicate placement submission"
    );
}

#[test]
fn hu_24_placement_timer_uses_server_phase_duration() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(50), CardType::Minion)]));

    set_phase_with_round(&mut app, RoundPhase::DraftShop, 7);
    run_update(&mut app);
    app.world_mut().resource_mut::<ClientPhaseView>().phase = RoundPhase::Placement;
    app.world_mut()
        .resource_mut::<ClientPhaseView>()
        .round_number = 7;
    app.world_mut()
        .resource_mut::<ClientPhaseView>()
        .timer_duration_ms = 30_000;
    set_phase_with_round(&mut app, RoundPhase::Placement, 7);
    run_update(&mut app);

    assert_eq!(
        app.world().resource::<PlacementTimer>().remaining_ms,
        30_000
    );
}

fn app_with_hand_ui_in_placement(catalog: HashMap<CardId, CardData>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(HandUiPlugin);
    app.insert_resource(BoardLayout {
        board_origin: Vec2::ZERO,
        cell_width: 64.0,
        lane_height: 80.0,
    });
    app.insert_resource(HandCardCatalog { cards: catalog });
    {
        let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
        economy.current_mana = 10;
        economy.reserve_mana = 10;
    }
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_phase(&mut app, RoundPhase::Placement);
    run_update(&mut app);
    app
}

fn test_catalog<const N: usize>(entries: [(CardId, CardType); N]) -> HashMap<CardId, CardData> {
    entries
        .into_iter()
        .map(|(card_id, card_type)| (card_id, test_card(card_id, card_type)))
        .collect()
}

fn test_card(card_id: CardId, card_type: CardType) -> CardData {
    CardData {
        id: card_id,
        name_fr: format!("Carte {}", card_id.0),
        name_en: format!("Card {}", card_id.0),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type,
        unit_type: UnitType::Blade,
        cost: 1,
        atk: 1,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: format!("test_{}", card_id.0),
        pool_copies_override: None,
    }
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn set_phase_with_round(app: &mut App, phase: RoundPhase, round: u32) {
    let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
    current.phase = phase;
    current.round = round;
}

fn set_hand<const N: usize>(app: &mut App, cards: [CardId; N]) {
    app.world_mut().resource_mut::<HandContents>().cards = cards.to_vec();
    run_update(app);
}

fn stage_card(app: &mut App, slot_index: u8, owner_id: PlayerId, target: PlayTarget) {
    let slot = fan_slot(app, slot_index);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id,
        target: Some(target),
    });
    run_update(app);
}

fn start_drag(app: &mut App, slot_index: u8, owner_id: PlayerId) {
    let slot = fan_slot(app, slot_index);
    app.world_mut().write_message(HandUiPlacementDragStarted {
        card: slot,
        owner_id,
    });
    run_update(app);
}

fn click_submit_for(app: &mut App, duration: Duration) {
    let button = submit_button(app);
    app.world_mut()
        .write_message(HandSubmitButtonClicked { button });
    run_for(app, duration);
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

fn submissions(app: &App) -> Vec<shared::protocol::C2SSubmitPlacement> {
    app.world()
        .resource::<HandUiOutboundMessages>()
        .submit_placements
        .clone()
}

fn all_urgency_messages(app: &App) -> Vec<TimerUrgencyAudio> {
    let messages = app.world().resource::<Messages<TimerUrgencyAudio>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).copied().collect()
}

fn drained_urgency_cursor(app: &App) -> MessageCursor<TimerUrgencyAudio> {
    let messages = app.world().resource::<Messages<TimerUrgencyAudio>>();
    let mut cursor = messages.get_cursor();
    let _ = cursor.read(messages).count();
    cursor
}

fn urgency_messages_since(
    app: &App,
    cursor: &mut MessageCursor<TimerUrgencyAudio>,
) -> Vec<TimerUrgencyAudio> {
    let messages = app.world().resource::<Messages<TimerUrgencyAudio>>();
    cursor.read(messages).copied().collect()
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}

fn submit_button(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<HandSubmitButton>>();
    query
        .single(app.world())
        .expect("submit button should exist")
}

fn timer_entity(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<client::ui::hand::HandTimer>>();
    query.single(app.world()).expect("timer should exist")
}

fn submitted_checkmark(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<TimerSubmittedCheckmark>>();
    query
        .single(app.world())
        .expect("submitted checkmark should exist")
}

fn drag_sprite(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<HandDragSprite>>();
    query.single(app.world()).expect("drag sprite should exist")
}

fn assert_drag_visibility(app: &mut App, expected: Visibility) {
    let drag = drag_sprite(app);
    assert_eq!(app.world().get::<Visibility>(drag), Some(&expected));
}
