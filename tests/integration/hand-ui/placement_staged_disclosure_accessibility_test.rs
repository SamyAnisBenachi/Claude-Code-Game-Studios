use std::collections::{BTreeMap, BTreeSet, HashMap};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::card_animations::HandDragSprite;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::{
    hand::{
        BoardCellHighlighted, BoardSpawnEdge, FanPlateHighlighted, FanSlotIndex, FanSlotState,
        GhostPlacementChanged, HandCardCatalog, HandContents, HandSubmitButton,
        HandSubmitButtonClicked, HandSubmitInteractionState, HandUiOutboundMessages,
        HandUiPlacementCursorMoved, HandUiPlacementDragEnded, HandUiPlacementDragStarted,
        HandUiPlacementDropResolved, HandUiPlugin, PendingPlacements, PlacementBoardView,
        PlacementDisclosureGuidance, PlacementDisclosureState, PlacementDisclosureStep,
        ReserveStripAction, ReserveStripButton, ReserveStripForFanSlot, ReserveStripValueText,
        SubmitValidationError, TimerSubmittedCheckmark,
    },
    shared::{BoardLayout, LaneCell, BOARD_LANE_COUNT},
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[test]
fn a11y_st_14_entry_exposes_only_card_selection_stage() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([
        (CardId(10), CardType::Minion, 2),
        (CardId(11), CardType::Order, 1),
    ]));
    set_hand(&mut app, [CardId(10), CardId(11)]);

    assert_eq!(
        disclosure_step(&app),
        PlacementDisclosureStep::CardSelection
    );
    assert_eq!(guidance_text(&mut app), "Select a card");
    assert_eq!(visibility_of(&mut app, submit_button), Visibility::Visible);
    assert_eq!(visibility_of(&mut app, timer), Visibility::Visible);
    assert_eq!(visibility_of(&mut app, drag_sprite), Visibility::Hidden);
    assert_eq!(count_with::<BoardCellHighlighted>(&mut app), 0);
    assert_eq!(count_with::<FanPlateHighlighted>(&mut app), 0);
    assert_all_reserve_strips_hidden(&mut app);
}

#[test]
fn a11y_st_14_minion_selection_discloses_lane_cell_before_split_controls() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(20), CardType::Minion, 3)]));
    let board_cells = spawn_board_cells(&mut app);
    app.world_mut()
        .resource_mut::<PlacementBoardView>()
        .spawn_range_cells = 2;
    set_hand(&mut app, [CardId(20)]);

    start_drag(&mut app, 0, PlayerId(1));
    let slot = fan_slot(&mut app, 0);
    let strip = reserve_strip(&mut app, 0);

    assert_eq!(
        disclosure_step(&app),
        PlacementDisclosureStep::TargetSelection {
            target_kind: client::ui::hand::PlacementTargetKind::Minion,
        }
    );
    assert_eq!(guidance_text(&mut app), "Choose a lane and cell");
    assert_eq!(visibility_of(&mut app, drag_sprite), Visibility::Visible);
    assert_eq!(
        app.world().get::<FanSlotState>(slot),
        Some(&FanSlotState::Active)
    );
    assert_eq!(
        highlighted_lane_cells(&mut app),
        (1..=BOARD_LANE_COUNT)
            .flat_map(|lane| [(lane, 1), (lane, 2)])
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        app.world().get::<Visibility>(strip),
        Some(&Visibility::Hidden)
    );

    assert_eq!(board_cells.len(), usize::from(BOARD_LANE_COUNT) * 8);
}

#[test]
fn a11y_st_14_invalid_drop_recovers_to_card_selection_without_staging() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(30), CardType::Minion, 2)]));
    spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(30)]);
    start_drag(&mut app, 0, PlayerId(1));

    let slot = fan_slot(&mut app, 0);
    let strip = reserve_strip(&mut app, 0);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id: PlayerId(1),
        target: None,
    });
    app.update();

    assert_eq!(
        disclosure_step(&app),
        PlacementDisclosureStep::CardSelection
    );
    assert_eq!(guidance_text(&mut app), "Select a card");
    assert_eq!(visibility_of(&mut app, drag_sprite), Visibility::Hidden);
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
    assert_eq!(
        app.world().get::<Visibility>(strip),
        Some(&Visibility::Hidden)
    );
}

#[test]
fn a11y_st_14_valid_stage_reveals_staged_guidance_and_split_text() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(40), CardType::Minion, 3)]));
    spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(40)]);
    start_drag(&mut app, 0, PlayerId(1));

    let slot = fan_slot(&mut app, 0);
    let strip = reserve_strip(&mut app, 0);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id: PlayerId(1),
        target: Some(PlayTarget::BoardCell { lane: 1, cell: 1 }),
    });
    app.update();

    assert_eq!(disclosure_step(&app), PlacementDisclosureStep::StagedCard);
    assert_eq!(guidance_text(&mut app), "Review staged card and mana split");
    assert_eq!(
        ghost_messages(&app),
        vec![GhostPlacementChanged {
            target: Some(PlayTarget::BoardCell { lane: 1, cell: 1 }),
            card_id: Some(CardId(40)),
        }]
    );
    assert_eq!(
        app.world().get::<FanSlotState>(slot),
        Some(&FanSlotState::Ghost)
    );
    assert_eq!(submit_text(&mut app), "Submit (1 cards)");
    assert_eq!(
        app.world().get::<Visibility>(strip),
        Some(&Visibility::Visible)
    );
    assert_eq!(reserve_text(&mut app, 0), "Reserve 0 Current 3");
}

#[test]
fn a11y_st_14_instant_stage_uses_same_staged_disclosure_without_board_highlights() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(50), CardType::Order, 1)]));
    spawn_board_cells(&mut app);
    set_hand(&mut app, [CardId(50)]);

    start_drag(&mut app, 0, PlayerId(1));
    let fan_plate = fan_plate(&mut app);
    assert_eq!(
        app.world().get::<FanPlateHighlighted>(fan_plate),
        Some(&FanPlateHighlighted)
    );
    assert_eq!(count_with::<BoardCellHighlighted>(&mut app), 0);

    move_cursor(&mut app, Vec2::new(400.0, 500.0));
    end_drag(&mut app);

    assert_eq!(disclosure_step(&app), PlacementDisclosureStep::StagedCard);
    assert_eq!(guidance_text(&mut app), "Review staged card and mana split");
    assert_eq!(count_with::<BoardCellHighlighted>(&mut app), 0);
    assert_eq!(count_with::<FanPlateHighlighted>(&mut app), 0);
    assert_eq!(
        app.world().resource::<PendingPlacements>().placements[0].target,
        PlayTarget::Instant
    );
}

#[test]
fn a11y_st_14_submit_correction_keeps_player_in_disclosure_flow() {
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(60), CardType::Minion, 2)]));
    set_economy(&mut app, 0, 2);
    set_hand(&mut app, [CardId(60)]);
    stage_card(
        &mut app,
        0,
        PlayerId(1),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );

    click_submit(&mut app);

    assert_eq!(
        disclosure_step(&app),
        PlacementDisclosureStep::Correction {
            error: SubmitValidationError::ManaOverdrawn,
        }
    );
    assert_eq!(guidance_text(&mut app), "Adjust reserve/current mana");
    assert_eq!(submission_count(&app), 0);
    let submit = submit_button(&mut app);
    assert_eq!(
        app.world().get::<HandSubmitInteractionState>(submit),
        Some(&HandSubmitInteractionState::Active)
    );

    click_reserve_button(&mut app, 0, ReserveStripAction::Increment);
    click_reserve_button(&mut app, 0, ReserveStripAction::Increment);
    assert_eq!(reserve_text(&mut app, 0), "Reserve 2 Current 0");
    assert_eq!(submit_error(&mut app), None);

    click_submit(&mut app);

    assert_eq!(disclosure_step(&app), PlacementDisclosureStep::Submitted);
    assert_eq!(guidance_text(&mut app), "Placement submitted");
    assert_eq!(submission_count(&app), 1);
    let submit = submit_button(&mut app);
    assert_eq!(
        app.world().get::<HandSubmitInteractionState>(submit),
        Some(&HandSubmitInteractionState::Inactive)
    );
    let checkmark = submitted_checkmark(&mut app);
    assert_eq!(
        app.world().get::<Visibility>(checkmark),
        Some(&Visibility::Visible)
    );
}

fn app_with_hand_ui_in_placement(catalog: HashMap<CardId, CardData>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(HandUiPlugin);
    app.insert_resource(BoardLayout {
        board_origin: Vec2::ZERO,
        cell_width: 64.0,
        lane_height: 80.0,
    });
    app.insert_resource(HandCardCatalog { cards: catalog });
    app.insert_resource(PlacementBoardView {
        local_player_id: PlayerId(1),
        opponent_player_id: PlayerId(2),
        spawn_edge: BoardSpawnEdge::LowCells,
        spawn_range_cells: 1,
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_phase(&mut app, RoundPhase::Placement);
    app.update();
    app
}

fn test_catalog<const N: usize>(
    entries: [(CardId, CardType, u32); N],
) -> HashMap<CardId, CardData> {
    entries
        .into_iter()
        .map(|(card_id, card_type, cost)| (card_id, test_card(card_id, card_type, cost)))
        .collect()
}

fn test_card(card_id: CardId, card_type: CardType, cost: u32) -> CardData {
    CardData {
        id: card_id,
        name_fr: format!("Carte {}", card_id.0),
        name_en: format!("Card {}", card_id.0),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type,
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

fn set_economy(app: &mut App, current_mana: u32, reserve_mana: u32) {
    let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
    economy.current_mana = current_mana;
    economy.reserve_mana = reserve_mana;
}

fn spawn_board_cells(app: &mut App) -> BTreeMap<(u8, u8), Entity> {
    (1..=BOARD_LANE_COUNT)
        .flat_map(|lane| (1..=8).map(move |cell| (lane, cell)))
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

fn stage_card(app: &mut App, slot_index: u8, owner_id: PlayerId, target: PlayTarget) {
    let slot = fan_slot(app, slot_index);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id,
        target: Some(target),
    });
    app.update();
}

fn click_submit(app: &mut App) {
    let button = submit_button(app);
    app.world_mut()
        .write_message(HandSubmitButtonClicked { button });
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

fn disclosure_step(app: &App) -> PlacementDisclosureStep {
    app.world().resource::<PlacementDisclosureState>().step
}

fn guidance_text(app: &mut App) -> String {
    let mut query = app
        .world_mut()
        .query_filtered::<&Text, With<PlacementDisclosureGuidance>>();
    query
        .single(app.world())
        .expect("placement disclosure guidance should exist")
        .0
        .clone()
}

fn submit_text(app: &mut App) -> String {
    let mut query = app
        .world_mut()
        .query_filtered::<&Text, With<HandSubmitButton>>();
    query
        .single(app.world())
        .expect("submit button should exist")
        .0
        .clone()
}

fn reserve_text(app: &mut App, slot_index: u8) -> String {
    let mut query = app.world_mut().query::<(&ReserveStripValueText, &Text)>();
    query
        .iter(app.world())
        .find_map(|(value_slot, text)| (value_slot.0 == slot_index).then_some(text.0.clone()))
        .expect("reserve strip value text should exist")
}

fn highlighted_lane_cells(app: &mut App) -> BTreeSet<(u8, u8)> {
    let mut query = app
        .world_mut()
        .query_filtered::<&LaneCell, With<BoardCellHighlighted>>();
    query
        .iter(app.world())
        .map(|lane_cell| (lane_cell.lane, lane_cell.cell))
        .collect()
}

fn ghost_messages(app: &App) -> Vec<GhostPlacementChanged> {
    let messages = app.world().resource::<Messages<GhostPlacementChanged>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn submission_count(app: &App) -> usize {
    app.world()
        .resource::<HandUiOutboundMessages>()
        .submit_placements
        .len()
}

fn submit_error(app: &mut App) -> Option<SubmitValidationError> {
    let button = submit_button(app);
    app.world().get::<SubmitValidationError>(button).copied()
}

fn visibility_of(app: &mut App, query_fn: fn(&mut App) -> Entity) -> Visibility {
    let entity = query_fn(app);
    *app.world()
        .get::<Visibility>(entity)
        .expect("entity should have visibility")
}

fn assert_all_reserve_strips_hidden(app: &mut App) {
    let mut query = app
        .world_mut()
        .query_filtered::<&Visibility, With<ReserveStripForFanSlot>>();
    assert!(query
        .iter(app.world())
        .all(|visibility| *visibility == Visibility::Hidden));
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

fn submit_button(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<HandSubmitButton>>();
    query
        .single(app.world())
        .expect("submit button should exist")
}

fn timer(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<client::ui::hand::HandTimer>>();
    query.single(app.world()).expect("timer should exist")
}

fn drag_sprite(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<HandDragSprite>>();
    query.single(app.world()).expect("drag sprite should exist")
}

fn fan_plate(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<client::ui::hand::FanPlateDropZone>>();
    query.single(app.world()).expect("fan plate should exist")
}

fn submitted_checkmark(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<TimerSubmittedCheckmark>>();
    query
        .single(app.world())
        .expect("submitted checkmark should exist")
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}
