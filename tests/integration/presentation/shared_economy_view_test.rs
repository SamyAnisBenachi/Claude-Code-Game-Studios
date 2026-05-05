use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::{
    apply_snapshot_to_player_economy_view, PlayerEconomyView, PlayerEconomyViewUpdateSource,
};
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    FanSlotIndex, HandCardCatalog, HandContents, HandUiPlacementDropResolved, HandUiPlugin,
    PendingPlacements, ReserveStripAction, ReserveStripButton,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{
    BoardSnapshot, PlayTarget, PlayerSnapshot, RoundPhase, S2CGameSnapshot, S2CGoldUpdate,
};
use shared::session::PlayerId;

#[test]
fn test_gold_update_updates_shared_player_economy_view() {
    let mut view = PlayerEconomyView::default();

    view.apply_gold_update(&S2CGoldUpdate {
        gold: 4,
        current_mana: 3,
        reserve_mana: 2,
        mana_cap: 10,
    });

    assert_eq!(view.gold, 4);
    assert_eq!(view.current_mana, 3);
    assert_eq!(view.reserve_mana, 2);
    assert_eq!(view.mana_cap, 10);
    assert!(view.initialized);
    assert_eq!(
        view.last_update_source,
        Some(PlayerEconomyViewUpdateSource::GoldUpdate)
    );
}

#[test]
fn test_snapshot_initializes_local_player_economy_view() {
    let mut view = PlayerEconomyView::default();
    let snapshot = snapshot(
        player_snapshot(player(1), 9, 6, 4, 11),
        player_snapshot(player(2), 20, 1, 0, 10),
    );

    assert!(apply_snapshot_to_player_economy_view(&snapshot, &mut view));

    assert_eq!(view.gold, 9);
    assert_eq!(view.current_mana, 6);
    assert_eq!(view.reserve_mana, 4);
    assert_eq!(view.mana_cap, 11);
    assert!(view.initialized);
    assert_eq!(
        view.last_update_source,
        Some(PlayerEconomyViewUpdateSource::Snapshot)
    );
}

#[test]
fn test_reserve_strip_input_does_not_mutate_player_economy_view() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(10), 5)]));
    set_hand(&mut app, [CardId(10)]);
    stage_card(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );

    let before = app.world().resource::<PlayerEconomyView>().clone();
    click_reserve_button(&mut app, 0, ReserveStripAction::Increment);

    assert_eq!(app.world().resource::<PlayerEconomyView>(), &before);
    assert_eq!(reserve_amount(&app, CardId(10)), 1);
}

fn app_with_hand_ui_in_placement(catalog: HashMap<CardId, CardData>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(HandUiPlugin);
    app.insert_resource(HandCardCatalog { cards: catalog });
    app.world_mut()
        .resource_mut::<PlayerEconomyView>()
        .apply_gold_update(&S2CGoldUpdate {
            gold: 5,
            current_mana: 2,
            reserve_mana: 3,
            mana_cap: 10,
        });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = RoundPhase::Placement;
    app.update();
    app
}

fn snapshot(own: PlayerSnapshot, opponent: PlayerSnapshot) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: own.player_id,
        round_number: 4,
        phase: RoundPhase::Placement,
        timer_remaining_ms: None,
        players: vec![own, opponent],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn player_snapshot(
    player_id: PlayerId,
    gold: u32,
    current_mana: u32,
    reserve_mana: u32,
    mana_cap: u8,
) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id: ClassId::Iop,
        gold,
        reserved_gold: 0,
        current_mana,
        reserve_mana,
        spawn_range_cells: 1,
        mana_cap,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: Vec::new(),
        opponent_objectives: Vec::new(),
    }
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

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
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

fn reserve_amount(app: &App, card_id: CardId) -> u32 {
    app.world()
        .resource::<PendingPlacements>()
        .placements
        .iter()
        .find(|placement| placement.card_id == card_id)
        .expect("card should be staged")
        .reserve_amount
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
