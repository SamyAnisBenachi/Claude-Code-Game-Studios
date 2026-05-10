use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    FanSlotIndex, HandCardCatalog, HandContents, HandUiPlacementDropResolved, HandUiPlugin,
    PendingPlacements, ReserveStripAction, ReserveStripButton, ReserveStripButtonDisabled,
    ReserveStripForFanSlot, ReserveStripValueText,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[test]
fn hu_25_plus_increments_to_ceiling_then_disabled_clicks_do_nothing() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(10), 5)]), 3);
    set_hand(&mut app, [CardId(10)]);
    stage_card(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );

    click_reserve_button(&mut app, 0, ReserveStripAction::Increment);
    assert_eq!(reserve_amount(&app, CardId(10)), 1);
    assert!(!button_disabled(&mut app, 0, ReserveStripAction::Increment));

    click_reserve_button(&mut app, 0, ReserveStripAction::Increment);
    assert_eq!(reserve_amount(&app, CardId(10)), 2);
    assert!(!button_disabled(&mut app, 0, ReserveStripAction::Increment));

    click_reserve_button(&mut app, 0, ReserveStripAction::Increment);
    assert_eq!(reserve_amount(&app, CardId(10)), 3);
    assert!(button_disabled(&mut app, 0, ReserveStripAction::Increment));
    assert_eq!(reserve_text(&mut app, 0), "Reserve 3 Current 2");

    click_reserve_button(&mut app, 0, ReserveStripAction::Increment);
    assert_eq!(reserve_amount(&app, CardId(10)), 3);
    assert!(button_disabled(&mut app, 0, ReserveStripAction::Increment));
}

#[test]
fn hu_26_other_staged_cards_reduce_ceiling_without_auto_decrement() {
    let mut app =
        app_with_hand_ui_in_placement(test_catalog([(CardId(20), 5), (CardId(21), 2)]), 3);
    set_hand(&mut app, [CardId(20), CardId(21)]);
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
    set_reserve_amount(&mut app, CardId(20), 2);

    click_reserve_button(&mut app, 1, ReserveStripAction::Increment);

    assert_eq!(reserve_amount(&app, CardId(21)), 1);
    assert!(button_disabled(&mut app, 1, ReserveStripAction::Increment));
    assert_eq!(reserve_amount(&app, CardId(20)), 2);

    click_reserve_button(&mut app, 0, ReserveStripAction::Decrement);

    assert_eq!(reserve_amount(&app, CardId(20)), 1);
    assert!(!button_disabled(&mut app, 1, ReserveStripAction::Increment));

    click_reserve_button(&mut app, 1, ReserveStripAction::Increment);

    assert_eq!(reserve_amount(&app, CardId(21)), 2);
    assert_eq!(reserve_amount(&app, CardId(20)), 1);
    assert!(button_disabled(&mut app, 1, ReserveStripAction::Increment));
}

#[test]
fn hu_27_free_card_reserve_strip_stays_hidden_and_ignores_clicks() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(30), 0)]), 3);
    set_hand(&mut app, [CardId(30)]);
    stage_card(&mut app, 0, PlayerId(7), PlayTarget::Instant);

    let strip = reserve_strip(&mut app, 0);
    assert_eq!(
        app.world().get::<Visibility>(strip),
        Some(&Visibility::Hidden)
    );
    assert_eq!(reserve_amount(&app, CardId(30)), 0);

    click_reserve_button(&mut app, 0, ReserveStripAction::Increment);

    assert_eq!(
        app.world().get::<Visibility>(strip),
        Some(&Visibility::Hidden)
    );
    assert_eq!(reserve_amount(&app, CardId(30)), 0);
}

fn app_with_hand_ui_in_placement(catalog: HashMap<CardId, CardData>, reserve_mana: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(HandUiPlugin);
    app.insert_resource(HandCardCatalog { cards: catalog });
    app.world_mut()
        .resource_mut::<PlayerEconomyView>()
        .reserve_mana = reserve_mana;
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_phase(&mut app, RoundPhase::Placement);
    app.update();
    app
}

fn test_catalog<const N: usize>(entries: [(CardId, u32); N]) -> HashMap<CardId, CardData> {
    entries
        .into_iter()
        .map(|(card_id, cost)| (card_id, test_card(card_id, cost)))
        .collect()
}

fn test_card(card_id: CardId, cost: u32) -> CardData {
    CardData {
        id: card_id,
        name_fr: format!("Carte {}", card_id.0),
        name_en: format!("Card {}", card_id.0),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost,
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

fn set_hand<const N: usize>(app: &mut App, cards: [CardId; N]) {
    app.world_mut().resource_mut::<HandContents>().cards = cards.to_vec();
    app.update();
}

fn stage_card(app: &mut App, slot_index: u8, owner_id: PlayerId, target: PlayTarget) {
    let slot = fan_slot(app, slot_index);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id,
        target: Some(target),
    });
    app.update();
}

fn click_reserve_button(app: &mut App, slot_index: u8, action: ReserveStripAction) {
    let button = reserve_button(app, slot_index, action);
    *app.world_mut()
        .get_mut::<Interaction>(button)
        .expect("reserve button should have Interaction") = Interaction::Pressed;
    app.update();
    *app.world_mut()
        .get_mut::<Interaction>(button)
        .expect("reserve button should have Interaction") = Interaction::None;
    app.update();
}

fn set_reserve_amount(app: &mut App, card_id: CardId, reserve_amount: u32) {
    let mut pending = app.world_mut().resource_mut::<PendingPlacements>();
    let placement = pending
        .placements
        .iter_mut()
        .find(|placement| placement.card_id == card_id)
        .expect("card should be staged");
    placement.reserve_mana_spend = reserve_amount;
    drop(pending);
    app.update();
}

fn reserve_amount(app: &App, card_id: CardId) -> u32 {
    app.world()
        .resource::<PendingPlacements>()
        .placements
        .iter()
        .find(|placement| placement.card_id == card_id)
        .expect("card should be staged")
        .reserve_mana_spend
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}

fn reserve_strip(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &ReserveStripForFanSlot)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("reserve strip should exist")
}

fn reserve_button(app: &mut App, slot_index: u8, action: ReserveStripAction) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &ReserveStripButton)>();
    query
        .iter(app.world())
        .find_map(|(entity, button)| {
            (button.slot_index == slot_index && button.action == action).then_some(entity)
        })
        .expect("reserve button should exist")
}

fn button_disabled(app: &mut App, slot_index: u8, action: ReserveStripAction) -> bool {
    let button = reserve_button(app, slot_index, action);
    app.world()
        .get::<ReserveStripButtonDisabled>(button)
        .is_some()
}

fn reserve_text(app: &mut App, slot_index: u8) -> String {
    let mut query = app.world_mut().query::<(&ReserveStripValueText, &Text)>();
    query
        .iter(app.world())
        .find_map(|(value_slot, text)| (value_slot.0 == slot_index).then_some(text.0.clone()))
        .expect("reserve strip value text should exist")
}
