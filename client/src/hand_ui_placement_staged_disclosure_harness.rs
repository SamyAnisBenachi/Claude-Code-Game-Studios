use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::hand::{
    BoardCellHighlighted, BoardSpawnEdge, FanSlotIndex, FanSlotState, GhostPlacementChanged,
    HandCardCatalog, HandContents, HandSubmitButton, HandSubmitButtonClicked,
    HandSubmitInteractionState, HandUiEntities, HandUiOutboundMessages, HandUiPlacementCursorMoved,
    HandUiPlacementDragStarted, HandUiPlacementDropResolved, HandUiPlugin, PendingPlacements,
    PlacementBoardView, PlacementDisclosureGuidance, PlacementDisclosureState,
    PlacementDisclosureStep, PlacementTargetKind, ReserveStripAction, ReserveStripButton,
    ReserveStripValueText, SubmitValidationError,
};
use client::ui::shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT};
use serde::Serialize;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlayTarget, RoundPhase};
use shared::session::PlayerId;

const CAPTURE_CARD_ID: CardId = CardId(60);
const LOCAL_PLAYER_ID: PlayerId = PlayerId(1);

fn main() {
    let report = build_capture_report();
    publish_report(&report);
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureReport {
    story: &'static str,
    qa_condition: &'static str,
    engine: &'static str,
    viewport: CaptureViewport,
    input_method: &'static str,
    steps: Vec<StepCapture>,
}

#[derive(Debug, Serialize)]
struct CaptureViewport {
    width: u32,
    height: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StepCapture {
    id: &'static str,
    title: &'static str,
    guidance: String,
    disclosure: String,
    submit_text: String,
    submit_state: String,
    timer_visible: bool,
    reserve_strip_visible: bool,
    reserve_text: String,
    fan_slot_state: String,
    drag_visible: bool,
    highlighted_cells: Vec<CellCapture>,
    ghost_message_count: usize,
    submission_count: usize,
    submit_error: Option<String>,
    checkmark_visible: bool,
    later_controls_hidden: bool,
    card_class: &'static str,
    pending_target: Option<CellCapture>,
}

#[derive(Debug, Serialize)]
struct CellCapture {
    lane: u8,
    cell: u8,
}

fn build_capture_report() -> CaptureReport {
    let mut app = app_with_hand_ui_in_placement();
    let mut steps = Vec::new();

    steps.push(capture_step(&mut app, "placement-entry", "PLACEMENT Entry"));

    start_drag(&mut app);
    steps.push(capture_step(&mut app, "card-selected", "Card Selected"));

    move_cursor_to_cell(&mut app, 1, 1);
    steps.push(capture_step(
        &mut app,
        "lane-cell-target-guidance",
        "Lane/Cell Target Guidance",
    ));

    app.update();
    steps.push(capture_step(
        &mut app,
        "valid-target-highlight",
        "Valid Target Highlight",
    ));

    drop_on_cell(&mut app, 1, 1);
    steps.push(capture_step(&mut app, "valid-stage", "Valid Stage"));

    click_reserve_button(&mut app, ReserveStripAction::Increment);
    steps.push(capture_step(
        &mut app,
        "reserve-current-split-adjustment",
        "Reserve/Current Split Adjustment",
    ));

    click_submit(&mut app);
    steps.push(capture_step(&mut app, "invalid-submit", "Invalid Submit"));

    click_reserve_button(&mut app, ReserveStripAction::Increment);
    click_reserve_button(&mut app, ReserveStripAction::Increment);
    click_submit(&mut app);
    steps.push(capture_step(
        &mut app,
        "correction-successful-submit",
        "Correction And Successful Submit",
    ));

    CaptureReport {
        story: "HAND-UI-014",
        qa_condition: "QA-COND-0005",
        engine: "Bevy 0.18 WASM HandUiPlugin harness",
        viewport: CaptureViewport {
            width: 1366,
            height: 768,
        },
        input_method: "deterministic mouse drag and click sequence",
        steps,
    }
}

fn app_with_hand_ui_in_placement() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(HandUiPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(BoardLayout {
        board_origin: Vec2::ZERO,
        cell_width: 64.0,
        lane_height: 80.0,
    });
    app.insert_resource(HandCardCatalog {
        cards: HashMap::from([(CAPTURE_CARD_ID, capture_card())]),
    });
    app.insert_resource(PlacementBoardView {
        local_player_id: LOCAL_PLAYER_ID,
        opponent_player_id: PlayerId(2),
        spawn_edge: BoardSpawnEdge::LowCells,
        spawn_range_cells: 2,
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    spawn_board_cells(&mut app);
    set_economy(&mut app, 0, 3);
    {
        let mut phase = app.world_mut().resource_mut::<CurrentClientPhase>();
        phase.phase = RoundPhase::Placement;
        phase.round = 1;
    }
    {
        let mut phase_view = app.world_mut().resource_mut::<ClientPhaseView>();
        phase_view.phase = RoundPhase::Placement;
        phase_view.round_number = 1;
        phase_view.timer_duration_ms = 10_000;
    }
    app.update();

    app.world_mut().resource_mut::<HandContents>().cards = vec![CAPTURE_CARD_ID];
    app.update();
    app
}

fn capture_card() -> CardData {
    CardData {
        id: CAPTURE_CARD_ID,
        name_fr: "Avant-garde Iop".to_string(),
        name_en: "Iop Vanguard".to_string(),
        class: ClassId::Iop,
        family: Some("HAND-UI-014".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: 3,
        atk: 2,
        hp: 3,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: "hand_ui_014_iop_vanguard".to_string(),
        pool_copies_override: None,
    }
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

fn set_economy(app: &mut App, current_mana: u32, reserve_mana: u32) {
    let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
    economy.current_mana = current_mana;
    economy.reserve_mana = reserve_mana;
    economy.mana_cap = 3;
    economy.initialized = true;
}

fn start_drag(app: &mut App) {
    let card = fan_slot(app, 0);
    app.world_mut().write_message(HandUiPlacementDragStarted {
        card,
        owner_id: LOCAL_PLAYER_ID,
    });
    app.update();
}

fn move_cursor_to_cell(app: &mut App, lane: u8, cell: u8) {
    let world_position = app
        .world()
        .resource::<BoardLayout>()
        .cell_to_world(lane, cell);
    // PROMPT 1210 — harness exercises the board-cell drop path, which reads
    // `cursor_world_position`; no screen-space sibling needed.
    app.world_mut().write_message(HandUiPlacementCursorMoved {
        world_position: Some(world_position),
        screen_position: None,
    });
    app.update();
}

fn drop_on_cell(app: &mut App, lane: u8, cell: u8) {
    let card = fan_slot(app, 0);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card,
        owner_id: LOCAL_PLAYER_ID,
        target: Some(PlayTarget::BoardCell { lane, cell }),
    });
    app.update();
}

fn click_reserve_button(app: &mut App, action: ReserveStripAction) {
    let button = reserve_button(app, 0, action);
    *app.world_mut()
        .get_mut::<Interaction>(button)
        .expect("reserve button should have Interaction") = Interaction::Pressed;
    app.update();
    *app.world_mut()
        .get_mut::<Interaction>(button)
        .expect("reserve button should have Interaction") = Interaction::None;
    app.update();
}

fn click_submit(app: &mut App) {
    let button = submit_button(app);
    app.world_mut()
        .write_message(HandSubmitButtonClicked { button });
    app.update();
}

fn capture_step(app: &mut App, id: &'static str, title: &'static str) -> StepCapture {
    let entities = *app.world().resource::<HandUiEntities>();
    let drag_visible = visibility(app, entities.drag_sprite);
    let fan_slot_state = fan_slot_state(app, 0);
    let reserve_strip_visible = visibility(app, entities.reserve_strips[0]);

    StepCapture {
        id,
        title,
        guidance: guidance_text(app),
        disclosure: disclosure_text(app),
        submit_text: submit_text(app),
        submit_state: submit_state(app),
        timer_visible: visibility(app, entities.timer),
        reserve_strip_visible,
        reserve_text: reserve_text(app, 0),
        fan_slot_state: fan_slot_state.clone(),
        drag_visible,
        highlighted_cells: highlighted_lane_cells(app),
        ghost_message_count: ghost_message_count(app),
        submission_count: submission_count(app),
        submit_error: submit_error(app).map(|error| format!("{error:?}")),
        checkmark_visible: visibility(app, entities.submitted_checkmark),
        later_controls_hidden: !reserve_strip_visible,
        card_class: card_class(&fan_slot_state, drag_visible),
        pending_target: pending_target(app),
    }
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

fn disclosure_text(app: &App) -> String {
    match app.world().resource::<PlacementDisclosureState>().step {
        PlacementDisclosureStep::Hidden => "Hidden".to_string(),
        PlacementDisclosureStep::CardSelection => "CardSelection".to_string(),
        PlacementDisclosureStep::TargetSelection { target_kind } => {
            format!("TargetSelection({})", target_kind_text(target_kind))
        }
        PlacementDisclosureStep::StagedCard => "StagedCard".to_string(),
        PlacementDisclosureStep::Correction { error } => format!("Correction({error:?})"),
        PlacementDisclosureStep::Submitted => "Submitted".to_string(),
    }
}

fn target_kind_text(target_kind: PlacementTargetKind) -> &'static str {
    match target_kind {
        PlacementTargetKind::Minion => "Minion",
        PlacementTargetKind::TargetObj => "TargetObj",
        PlacementTargetKind::LaneWide => "LaneWide",
        PlacementTargetKind::TargetUnit => "TargetUnit",
        PlacementTargetKind::Instant => "Instant",
    }
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

fn submit_state(app: &mut App) -> String {
    let submit = submit_button(app);
    app.world()
        .get::<HandSubmitInteractionState>(submit)
        .map(|state| format!("{state:?}"))
        .unwrap_or_else(|| "Missing".to_string())
}

fn reserve_text(app: &mut App, slot_index: u8) -> String {
    let mut query = app.world_mut().query::<(&ReserveStripValueText, &Text)>();
    query
        .iter(app.world())
        .find_map(|(value_slot, text)| (value_slot.0 == slot_index).then_some(text.0.clone()))
        .expect("reserve strip value text should exist")
}

fn highlighted_lane_cells(app: &mut App) -> Vec<CellCapture> {
    let mut query = app
        .world_mut()
        .query_filtered::<&LaneCell, With<BoardCellHighlighted>>();
    let mut cells = query
        .iter(app.world())
        .map(|lane_cell| CellCapture {
            lane: lane_cell.lane,
            cell: lane_cell.cell,
        })
        .collect::<Vec<_>>();
    cells.sort_by_key(|cell| (cell.lane, cell.cell));
    cells
}

fn ghost_message_count(app: &App) -> usize {
    let messages = app.world().resource::<Messages<GhostPlacementChanged>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).count()
}

fn submission_count(app: &App) -> usize {
    app.world()
        .resource::<HandUiOutboundMessages>()
        .submit_placements
        .len()
}

fn submit_error(app: &mut App) -> Option<SubmitValidationError> {
    let submit = submit_button(app);
    app.world().get::<SubmitValidationError>(submit).copied()
}

fn fan_slot_state(app: &mut App, index: u8) -> String {
    let slot = fan_slot(app, index);
    app.world()
        .get::<FanSlotState>(slot)
        .map(|state| format!("{state:?}"))
        .unwrap_or_else(|| "Missing".to_string())
}

fn pending_target(app: &App) -> Option<CellCapture> {
    app.world()
        .resource::<PendingPlacements>()
        .placements
        .first()
        .and_then(|placement| match placement.target {
            PlayTarget::BoardCell { lane, cell } => Some(CellCapture { lane, cell }),
            _ => None,
        })
}

fn visibility(app: &App, entity: Entity) -> bool {
    app.world()
        .get::<Visibility>(entity)
        .map(|visibility| *visibility == Visibility::Visible)
        .unwrap_or(false)
}

fn card_class(fan_slot_state: &str, drag_visible: bool) -> &'static str {
    if fan_slot_state == "Ghost" {
        "ghost"
    } else if drag_visible {
        "selected"
    } else {
        ""
    }
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

fn submit_button(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<HandSubmitButton>>();
    query
        .single(app.world())
        .expect("submit button should exist")
}

#[cfg(target_arch = "wasm32")]
fn publish_report(report: &CaptureReport) {
    use wasm_bindgen::{JsCast, JsValue};

    let json = serde_json::to_string(report).expect("capture report should serialize");
    let window = web_sys::window().expect("browser window should exist");
    let callback = js_sys::Reflect::get(
        window.as_ref(),
        &JsValue::from_str("handUiPlacementStagedDisclosureReport"),
    )
    .ok()
    .and_then(|value| value.dyn_into::<js_sys::Function>().ok())
    .expect("handUiPlacementStagedDisclosureReport callback should exist");
    callback
        .call1(&JsValue::NULL, &JsValue::from_str(&json))
        .expect("HAND-UI-014 report callback should accept JSON");
}

#[cfg(not(target_arch = "wasm32"))]
fn publish_report(report: &CaptureReport) {
    println!(
        "{}",
        serde_json::to_string_pretty(report).expect("capture report should serialize")
    );
}
