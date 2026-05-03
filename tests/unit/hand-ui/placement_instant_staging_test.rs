use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::card_animations::HandDragSprite;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::{
    hand::{
        BoardCellHighlighted, FanPlateHighlighted, FanSlotIndex, FanSlotState,
        GhostPlacementChanged, HandCardCatalog, HandContents, HandUiPlacementCursorMoved,
        HandUiPlacementDragEnded, HandUiPlacementDragStarted, HandUiPlugin, PendingPlacements,
    },
    shared::{LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT},
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[test]
fn hu_18_instant_drag_highlights_fan_plate_and_clears_board_cells() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(70), CardType::Order)]));
    let board_cells = spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(1), CardId(2), CardId(70)]);
    app.update();
    app.world_mut()
        .entity_mut(board_cells[&(1, 1)])
        .insert(BoardCellHighlighted);

    start_drag(&mut app, 2, PlayerId(1));

    assert_drag_visibility(&mut app, Visibility::Visible);
    let fan_plate = fan_plate(&mut app);
    assert!(app.world().get::<FanPlateHighlighted>(fan_plate).is_some());
    assert_eq!(
        count_with::<BoardCellHighlighted>(&mut app),
        0,
        "Instant drags must not leave any board cell highlights"
    );
}

#[test]
fn hu_19_instant_drop_on_plate_stages_instant_and_updates_submit_count() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(80), CardType::Order)]));
    set_hand(&mut app, [CardId(1), CardId(2), CardId(80)]);
    app.update();

    start_drag(&mut app, 2, PlayerId(7));
    move_cursor(&mut app, Vec2::new(400.0, 500.0));
    end_drag(&mut app);

    assert_eq!(
        ghost_messages(&app),
        vec![GhostPlacementChanged {
            target: Some(PlayTarget::Instant),
            card_id: Some(CardId(80)),
        }]
    );

    let slot = fan_slot(&mut app, 2);
    assert_eq!(
        app.world().get::<FanSlotState>(slot),
        Some(&FanSlotState::Ghost)
    );
    assert_eq!(submit_text(&mut app), "Submit (1 cards)");
    let fan_plate = fan_plate(&mut app);
    assert!(app.world().get::<FanPlateHighlighted>(fan_plate).is_none());

    let pending = &app.world().resource::<PendingPlacements>().placements;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].card_id, CardId(80));
    assert_eq!(pending[0].owner_id, PlayerId(7));
    assert_eq!(pending[0].target, PlayTarget::Instant);
}

#[test]
fn hu_19_instant_drop_outside_plate_returns_active_without_ghost_message() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(90), CardType::Order)]));
    set_hand(&mut app, [CardId(90)]);
    app.update();

    start_drag(&mut app, 0, PlayerId(7));
    move_cursor(&mut app, Vec2::new(400.0, 100.0));
    end_drag(&mut app);

    let slot = fan_slot(&mut app, 0);
    assert_eq!(
        app.world().get::<FanSlotState>(slot),
        Some(&FanSlotState::Active)
    );
    assert!(ghost_messages(&app).is_empty());
    assert!(app
        .world()
        .resource::<PendingPlacements>()
        .placements
        .is_empty());
    assert_drag_visibility(&mut app, Visibility::Hidden);
    let fan_plate = fan_plate(&mut app);
    assert!(app.world().get::<FanPlateHighlighted>(fan_plate).is_none());
}

fn app_with_hand_ui_in_placement(catalog: HashMap<CardId, CardData>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(HandUiPlugin);
    app.insert_resource(HandCardCatalog { cards: catalog });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_phase(&mut app, RoundPhase::Placement);
    app.update();
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
        unit_type: UnitType::Neutral,
        cost: 1,
        atk: 0,
        hp: 0,
        mp: 0,
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
}

fn spawn_board_cells(app: &mut App) -> BTreeMap<(u8, u8), Entity> {
    (1..=BOARD_LANE_COUNT)
        .flat_map(|lane| (1..=BOARD_CELL_COUNT).map(move |cell| (lane, cell)))
        .map(|(lane, cell)| {
            let entity = app.world_mut().spawn(LaneCell { lane, cell }).id();
            ((lane, cell), entity)
        })
        .collect()
}

fn start_drag(app: &mut App, slot_index: u8, owner_id: PlayerId) {
    let slot = fan_slot(app, slot_index);
    app.world_mut().write_message(HandUiPlacementDragStarted {
        card: slot,
        owner_id,
    });
    app.update();
}

fn move_cursor(app: &mut App, position: Vec2) {
    app.world_mut().write_message(HandUiPlacementCursorMoved {
        world_position: Some(position),
    });
    app.update();
}

fn end_drag(app: &mut App) {
    app.world_mut().write_message(HandUiPlacementDragEnded);
    app.update();
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}

fn fan_plate(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<client::ui::hand::FanPlateDropZone>>();
    query.single(app.world()).expect("fan plate should exist")
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

fn submit_text(app: &mut App) -> String {
    let mut query = app
        .world_mut()
        .query_filtered::<&Text, With<client::ui::hand::HandSubmitButton>>();
    query
        .single(app.world())
        .expect("submit text should exist")
        .0
        .clone()
}

fn ghost_messages(app: &App) -> Vec<GhostPlacementChanged> {
    let messages = app.world().resource::<Messages<GhostPlacementChanged>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}
