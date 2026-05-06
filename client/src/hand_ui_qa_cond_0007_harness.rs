use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::hand::{
    BoardSpawnEdge, FanSlotIndex, FanSlotState, HandCardCatalog, HandContents, HandSubmitButton,
    HandSubmitButtonClicked, HandSubmitInteractionState, HandUiEntities, HandUiOutboundMessages,
    HandUiPlacementDropResolved, HandUiPlugin, PendingPlacements, PlacementBoardView,
    PlacementDisclosureState, PlacementDisclosureStep, PlacementTargetKind, PlacementTimer,
    ReserveStripAction, ReserveStripButton, ReserveStripButtonDisabled, ReserveStripValueText,
    SubmitValidationError, TimerState, TimerUrgencyAudio,
};
use client::ui::shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT};
use serde::Serialize;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlayTarget, RoundPhase};
use shared::session::PlayerId;

const CAPTURE_CARD_ID: CardId = CardId(70);
const LOCAL_PLAYER_ID: PlayerId = PlayerId(1);

fn main() {
    let report = build_capture_report();
    publish_report(&report);
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureReport {
    qa_condition: &'static str,
    scope: &'static str,
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
    story_scope: &'static str,
    expected: &'static str,
    actual: String,
    timer_text: String,
    timer_state: String,
    timer_remaining_ms: u32,
    urgency_fired: bool,
    urgency_audio_count: usize,
    guidance: String,
    disclosure: String,
    submit_text: String,
    submit_state: String,
    submit_error: Option<String>,
    submission_count: usize,
    checkmark_visible: bool,
    reserve_strip_visible: bool,
    reserve_text: String,
    reserve_minus_visible: bool,
    reserve_plus_visible: bool,
    reserve_minus_disabled: bool,
    reserve_plus_disabled: bool,
    fan_slot_state: String,
    pending_count: usize,
    current_mana: u32,
    reserve_mana: u32,
}

impl StepCapture {
    fn summarize(&self) -> String {
        format!(
            "timer={} ({}, {}ms), urgency_fired={}, audio={}, submit='{}'/{}, error={}, checkmark={}, reserve='{}' visible={}, plus_disabled={}, submissions={}",
            self.timer_text,
            self.timer_state,
            self.timer_remaining_ms,
            self.urgency_fired,
            self.urgency_audio_count,
            self.submit_text,
            self.submit_state,
            self.submit_error.as_deref().unwrap_or("none"),
            self.checkmark_visible,
            self.reserve_text,
            self.reserve_strip_visible,
            self.reserve_plus_disabled,
            self.submission_count
        )
    }
}

fn build_capture_report() -> CaptureReport {
    let mut steps = Vec::new();

    let mut normal_timer = app_with_hand_ui_in_placement(3, 3, 3);
    steps.push(capture_step(
        &mut normal_timer,
        "normal-placement-timer",
        "Normal Placement Timer",
        "HAND-UI-009",
        "Timer is visible before the urgency threshold in Normal state with a whole-second label.",
    ));

    let mut urgent_timer = app_with_hand_ui_in_placement(3, 3, 3);
    {
        let mut timer = urgent_timer.world_mut().resource_mut::<PlacementTimer>();
        timer.remaining_ms = 5_001;
        timer.urgency_fired = false;
    }
    run_for(&mut urgent_timer, Duration::from_millis(2));
    steps.push(capture_step(
        &mut urgent_timer,
        "urgent-timer-leq-5s",
        "Urgent Timer At <=5s",
        "HAND-UI-009",
        "Crossing from above 5s to <=5s sets TimerState::Urgent and emits exactly one urgency audio message.",
    ));

    let mut submitted_checkmark = app_with_hand_ui_in_placement(10, 10, 3);
    submitted_checkmark
        .world_mut()
        .resource_mut::<PlacementTimer>()
        .remaining_ms = 7_000;
    click_submit_for(&mut submitted_checkmark, Duration::from_millis(16));
    steps.push(capture_step(
        &mut submitted_checkmark,
        "submitted-checkmark",
        "Submitted Checkmark",
        "HAND-UI-009",
        "Successful submit keeps the timer visible/running and shows the submitted checkmark.",
    ));

    let mut reserve_affordance = app_with_hand_ui_in_placement(3, 3, 3);
    stage_card(&mut reserve_affordance);
    steps.push(capture_step(
        &mut reserve_affordance,
        "reserve-strip-plus-minus",
        "Reserve Strip With +/-",
        "HAND-UI-011",
        "A staged cost card shows reserve/current split text plus decrement and increment controls.",
    ));

    click_reserve_button(&mut reserve_affordance, ReserveStripAction::Increment);
    click_reserve_button(&mut reserve_affordance, ReserveStripAction::Increment);
    click_reserve_button(&mut reserve_affordance, ReserveStripAction::Increment);
    steps.push(capture_step(
        &mut reserve_affordance,
        "disabled-reserve-ceiling",
        "Disabled Reserve Ceiling State",
        "HAND-UI-011",
        "At the reserve ceiling, the increment control is disabled and further increments are blocked.",
    ));

    let mut invalid_submit = app_with_hand_ui_in_placement(0, 3, 3);
    stage_card(&mut invalid_submit);
    click_reserve_button(&mut invalid_submit, ReserveStripAction::Increment);
    click_submit(&mut invalid_submit);
    steps.push(capture_step(
        &mut invalid_submit,
        "invalid-submit-inline-correction",
        "Invalid Submit Inline Correction",
        "HAND-UI-010/HAND-UI-014",
        "Current/reserve overdraw blocks submit, keeps Submit active, attaches an inline correction state, and sends nothing.",
    ));

    click_reserve_button(&mut invalid_submit, ReserveStripAction::Increment);
    click_reserve_button(&mut invalid_submit, ReserveStripAction::Increment);
    click_submit(&mut invalid_submit);
    steps.push(capture_step(
        &mut invalid_submit,
        "corrected-successful-submit",
        "Corrected Successful Submit",
        "HAND-UI-010/HAND-UI-014",
        "After correcting the split, submit clears the error, sends exactly once, disables Submit, and shows the checkmark.",
    ));

    CaptureReport {
        qa_condition: "QA-COND-0007",
        scope: "Hand UI Stories 009, 010, 011, 014",
        engine: "Bevy 0.18 WASM HandUiPlugin QA harness",
        viewport: CaptureViewport {
            width: 1366,
            height: 768,
        },
        input_method: "deterministic ECS placement, reserve, timer, and submit sequence",
        steps,
    }
}

fn app_with_hand_ui_in_placement(current_mana: u32, reserve_mana: u32, card_cost: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(HandUiPlugin);
    app.insert_resource(BoardLayout {
        board_origin: Vec2::ZERO,
        cell_width: 64.0,
        lane_height: 80.0,
    });
    app.insert_resource(HandCardCatalog {
        cards: HashMap::from([(CAPTURE_CARD_ID, capture_card(card_cost))]),
    });
    app.insert_resource(PlacementBoardView {
        local_player_id: LOCAL_PLAYER_ID,
        opponent_player_id: PlayerId(2),
        spawn_edge: BoardSpawnEdge::LowCells,
        spawn_range_cells: 2,
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

    spawn_board_cells(&mut app);
    set_economy(&mut app, current_mana, reserve_mana);
    {
        let mut phase_view = app.world_mut().resource_mut::<ClientPhaseView>();
        phase_view.phase = RoundPhase::Placement;
        phase_view.round_number = 1;
        phase_view.timer_duration_ms = 10_000;
    }
    {
        let mut phase = app.world_mut().resource_mut::<CurrentClientPhase>();
        phase.phase = RoundPhase::Placement;
        phase.round = 1;
    }
    run_update(&mut app);
    app.world_mut().resource_mut::<HandContents>().cards = vec![CAPTURE_CARD_ID];
    run_update(&mut app);
    app
}

fn capture_card(cost: u32) -> CardData {
    CardData {
        id: CAPTURE_CARD_ID,
        name_fr: "Avant-garde Iop".to_string(),
        name_en: "Iop Vanguard".to_string(),
        class: ClassId::Iop,
        family: Some("QA-COND-0007".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost,
        atk: 2,
        hp: 3,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: "qa_cond_0007_iop_vanguard".to_string(),
        pool_copies_override: None,
    }
}

fn spawn_board_cells(app: &mut App) {
    for lane in 1..=BOARD_LANE_COUNT {
        for cell in 1..=BOARD_CELL_COUNT {
            app.world_mut().spawn(LaneCell { lane, cell });
        }
    }
}

fn set_economy(app: &mut App, current_mana: u32, reserve_mana: u32) {
    let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
    economy.current_mana = current_mana;
    economy.reserve_mana = reserve_mana;
    economy.mana_cap = u8::try_from(current_mana.max(reserve_mana)).unwrap_or(u8::MAX);
    economy.initialized = true;
}

fn stage_card(app: &mut App) {
    let card = fan_slot(app, 0);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card,
        owner_id: LOCAL_PLAYER_ID,
        target: Some(PlayTarget::BoardCell { lane: 1, cell: 1 }),
    });
    run_update(app);
}

fn click_reserve_button(app: &mut App, action: ReserveStripAction) {
    let button = reserve_button(app, 0, action);
    *app.world_mut()
        .get_mut::<Interaction>(button)
        .expect("reserve button should have Interaction") = Interaction::Pressed;
    run_update(app);
    *app.world_mut()
        .get_mut::<Interaction>(button)
        .expect("reserve button should have Interaction") = Interaction::None;
    run_update(app);
}

fn click_submit(app: &mut App) {
    let button = submit_button(app);
    app.world_mut()
        .write_message(HandSubmitButtonClicked { button });
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

fn capture_step(
    app: &mut App,
    id: &'static str,
    title: &'static str,
    story_scope: &'static str,
    expected: &'static str,
) -> StepCapture {
    let entities = *app.world().resource::<HandUiEntities>();
    let timer = *app.world().resource::<PlacementTimer>();
    let economy = app.world().resource::<PlayerEconomyView>().clone();
    let mut step = StepCapture {
        id,
        title,
        story_scope,
        expected,
        actual: String::new(),
        timer_text: text_for(app, entities.timer),
        timer_state: timer_state(app, entities.timer),
        timer_remaining_ms: timer.remaining_ms,
        urgency_fired: timer.urgency_fired,
        urgency_audio_count: urgency_audio_count(app),
        guidance: text_for(app, entities.placement_disclosure_guidance),
        disclosure: disclosure_text(app),
        submit_text: text_for(app, entities.submit_button),
        submit_state: submit_state(app, entities.submit_button),
        submit_error: submit_error(app, entities.submit_button),
        submission_count: app
            .world()
            .resource::<HandUiOutboundMessages>()
            .submit_placements
            .len(),
        checkmark_visible: visibility(app, entities.submitted_checkmark),
        reserve_strip_visible: visibility(app, entities.reserve_strips[0]),
        reserve_text: reserve_text(app, 0),
        reserve_minus_visible: reserve_button_visible(app, 0, ReserveStripAction::Decrement),
        reserve_plus_visible: reserve_button_visible(app, 0, ReserveStripAction::Increment),
        reserve_minus_disabled: reserve_button_disabled(app, 0, ReserveStripAction::Decrement),
        reserve_plus_disabled: reserve_button_disabled(app, 0, ReserveStripAction::Increment),
        fan_slot_state: fan_slot_state(app, 0),
        pending_count: app.world().resource::<PendingPlacements>().placements.len(),
        current_mana: economy.current_mana,
        reserve_mana: economy.reserve_mana,
    };
    step.actual = step.summarize();
    step
}

fn text_for(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .map(|text| text.0.clone())
        .unwrap_or_else(|| "missing".to_string())
}

fn timer_state(app: &App, entity: Entity) -> String {
    app.world()
        .get::<TimerState>(entity)
        .map(|state| format!("{state:?}"))
        .unwrap_or_else(|| "Missing".to_string())
}

fn submit_state(app: &App, entity: Entity) -> String {
    app.world()
        .get::<HandSubmitInteractionState>(entity)
        .map(|state| format!("{state:?}"))
        .unwrap_or_else(|| "Missing".to_string())
}

fn submit_error(app: &App, entity: Entity) -> Option<String> {
    app.world()
        .get::<SubmitValidationError>(entity)
        .map(|error| format!("{error:?}"))
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

fn urgency_audio_count(app: &App) -> usize {
    let messages = app.world().resource::<Messages<TimerUrgencyAudio>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).count()
}

fn visibility(app: &App, entity: Entity) -> bool {
    app.world()
        .get::<Visibility>(entity)
        .map(|visibility| *visibility == Visibility::Visible)
        .unwrap_or(false)
}

fn reserve_text(app: &mut App, slot_index: u8) -> String {
    let mut query = app.world_mut().query::<(&ReserveStripValueText, &Text)>();
    query
        .iter(app.world())
        .find_map(|(value_slot, text)| (value_slot.0 == slot_index).then_some(text.0.clone()))
        .unwrap_or_else(|| "missing".to_string())
}

fn reserve_button_visible(app: &mut App, slot_index: u8, action: ReserveStripAction) -> bool {
    let button = reserve_button(app, slot_index, action);
    visibility(app, button)
}

fn reserve_button_disabled(app: &mut App, slot_index: u8, action: ReserveStripAction) -> bool {
    let button = reserve_button(app, slot_index, action);
    app.world()
        .get::<ReserveStripButtonDisabled>(button)
        .is_some()
}

fn fan_slot_state(app: &mut App, index: u8) -> String {
    let slot = fan_slot(app, index);
    app.world()
        .get::<FanSlotState>(slot)
        .map(|state| format!("{state:?}"))
        .unwrap_or_else(|| "Missing".to_string())
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
        &JsValue::from_str("qaCond0007HandUiReport"),
    )
    .ok()
    .and_then(|value| value.dyn_into::<js_sys::Function>().ok())
    .expect("qaCond0007HandUiReport callback should exist");
    callback
        .call1(&JsValue::NULL, &JsValue::from_str(&json))
        .expect("QA-COND-0007 report callback should accept JSON");
}

#[cfg(not(target_arch = "wasm32"))]
fn publish_report(report: &CaptureReport) {
    println!(
        "{}",
        serde_json::to_string_pretty(report).expect("capture report should serialize")
    );
}
