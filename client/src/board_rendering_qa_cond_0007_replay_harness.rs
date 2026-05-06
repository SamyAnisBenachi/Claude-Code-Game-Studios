use std::time::Duration;

use bevy::ecs::message::MessageCursor;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::card_animations::{
    AnimQueue, AnimQueueEvent, AnimationTimingConfig, CardAnimationsPlugin, PendingPhaseChange,
};
use client::presentation::board_rendering::{
    resolution_anim_groups_from_script, BoardLocalPlayer, BoardRenderState, BoardRenderingPlugin,
    PendingResolutionScript, PlayerTeamMap, ResolutionRevealWait, SnapshotRecoveryRequested,
    SpawnHighlightState,
};
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::shared::LaneCell;
use serde::Serialize;
use shared::keyword::KeywordKind;
use shared::protocol::{
    GameOverReason, GoldReason, ResolutionEvent, RoundPhase, S2CPhaseChanged, S2CResolutionEvent,
    TaggedEvent,
};
use shared::session::PlayerId;

const LOCAL_PLAYER_ID: PlayerId = PlayerId(1);
const OPPONENT_PLAYER_ID: PlayerId = PlayerId(2);
const CAPTURE_ROUND: u32 = 8;

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
    source_commits: SourceCommits,
    replay_script: ReplayScriptReport,
    steps: Vec<StepCapture>,
    recovery: RecoveryCapture,
    verdict: VerdictReport,
}

#[derive(Debug, Serialize)]
struct CaptureViewport {
    width: u32,
    height: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceCommits {
    implementation: &'static str,
    story_done: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReplayScriptReport {
    round: u32,
    event_count: usize,
    group_count: usize,
    sub_steps: Vec<SubStepReport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SubStepReport {
    sub_step: u8,
    trigger_indexes: Vec<u32>,
    event_labels: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StepCapture {
    id: &'static str,
    title: &'static str,
    expected: &'static str,
    actual: String,
    elapsed_ms: u64,
    board_render_state: String,
    current_phase: String,
    phase_view: String,
    pending_phase: Option<String>,
    queue_group_count: usize,
    queue_current_index: Option<usize>,
    current_sub_step: Option<u8>,
    queue_empty: bool,
    phase_jump_blocked: bool,
    spawn_highlight_cells: Vec<String>,
    readable_result: &'static str,
}

impl StepCapture {
    fn summarize(&self) -> String {
        format!(
            "state={}, current_phase={}, phase_view={}, pending_phase={}, queue_empty={}, current_sub_step={}, phase_jump_blocked={}, spawn_highlights={}",
            self.board_render_state,
            self.current_phase,
            self.phase_view,
            self.pending_phase.as_deref().unwrap_or("none"),
            self.queue_empty,
            self.current_sub_step
                .map(|sub_step| sub_step.to_string())
                .unwrap_or_else(|| "none".to_string()),
            self.phase_jump_blocked,
            if self.spawn_highlight_cells.is_empty() {
                "none".to_string()
            } else {
                self.spawn_highlight_cells.join(", ")
            }
        )
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RecoveryCapture {
    id: &'static str,
    title: &'static str,
    expected: &'static str,
    actual: String,
    request_count: usize,
    duplicate_request_count_after_second_update: usize,
    reasons: Vec<String>,
    pending_script_present: bool,
    queue_empty: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VerdictReport {
    ready_for_capture: bool,
    result_progression_pass: bool,
    phase_buffering_pass: bool,
    recovery_pass: bool,
    playable_client_manual_qa_claimed: bool,
}

fn build_capture_report() -> CaptureReport {
    let script = replay_script();
    let replay_script_report = ReplayScriptReport::from_script(&script);
    let mut app = app_in_session();
    seed_replay(&mut app, script);

    let mut steps = Vec::new();
    steps.push(capture_step(
        &mut app,
        "replay-start",
        "Replay Queue Loaded",
        "Resolution replay starts in ResolutionExecuting with ordered sub-step 1 active and DraftShop buffered.",
        0,
        "Sub-step 1 is visibly identified as active, with later result groups queued.",
    ));

    run_for(&mut app, Duration::from_millis(599));
    steps.push(capture_step(
        &mut app,
        "replay-mid-first-sub-step",
        "First Result Still Holding Phase",
        "At 599ms the client is still in Resolution, the queue is not drained, and DraftShop has not applied.",
        599,
        "The current result remains readable before the first 600ms group boundary.",
    ));

    run_for(&mut app, Duration::from_millis(1));
    run_for(&mut app, Duration::from_millis(150));
    steps.push(capture_step(
        &mut app,
        "replay-second-sub-step",
        "Second Result Advances After Pause",
        "After the inter-step pause, the active result advances to sub-step 2 while DraftShop remains buffered.",
        750,
        "The timeline advances one result group at a time after the queue pause.",
    ));

    run_for(&mut app, Duration::from_millis(600));
    run_for(&mut app, Duration::from_millis(150));
    run_for(&mut app, Duration::from_millis(599));
    steps.push(capture_step(
        &mut app,
        "replay-final-sub-step-buffered",
        "Final Result Before Drain",
        "During the final sub-step, the current phase is still Resolution and DraftShop remains pending.",
        2_099,
        "The final result is still readable before the next phase is released.",
    ));

    run_for(&mut app, Duration::from_millis(1));
    steps.push(capture_step(
        &mut app,
        "replay-drained-next-phase",
        "Replay Drained To Next Phase",
        "After the last group drains, the queue is empty and the buffered DraftShop phase applies.",
        2_100,
        "The after state shows a clean phase transition only after replay drain.",
    ));

    let recovery = capture_recovery();
    let phase_buffering_pass = steps.iter().any(|step| {
        step.id == "replay-final-sub-step-buffered"
            && step.phase_jump_blocked
            && step.current_phase == "Resolution"
            && step.pending_phase.as_deref() == Some("DraftShop")
    }) && steps.iter().any(|step| {
        step.id == "replay-drained-next-phase"
            && step.queue_empty
            && step.current_phase == "DraftShop"
            && step.pending_phase.is_none()
    });
    let result_progression_pass = steps
        .iter()
        .filter_map(|step| step.current_sub_step)
        .collect::<Vec<_>>()
        == vec![1, 1, 2, 3];
    let recovery_pass = recovery.request_count == 1
        && recovery.duplicate_request_count_after_second_update == 0
        && !recovery.pending_script_present
        && recovery.queue_empty;

    CaptureReport {
        qa_condition: "QA-COND-0007",
        scope: "Resolution replay readability using BR-006 queue and phase-buffering infrastructure",
        engine: "Bevy 0.18 WASM board rendering replay QA harness",
        viewport: CaptureViewport {
            width: 1366,
            height: 768,
        },
        input_method: "deterministic ECS replay script against BoardRenderingPlugin, CardAnimationsPlugin, AnimQueue, PendingPhaseChange, and SnapshotRecoveryRequested",
        source_commits: SourceCommits {
            implementation: "8caa1a0195fd817b1ce632877db2174a357e8162",
            story_done: "484bef101c16bc7931456b2c0f72676279d7a536",
        },
        replay_script: replay_script_report,
        steps,
        recovery,
        verdict: VerdictReport {
            ready_for_capture: result_progression_pass && phase_buffering_pass && recovery_pass,
            result_progression_pass,
            phase_buffering_pass,
            recovery_pass,
            playable_client_manual_qa_claimed: false,
        },
    }
}

impl ReplayScriptReport {
    fn from_script(script: &S2CResolutionEvent) -> Self {
        let groups = resolution_anim_groups_from_script(script, AnimationTimingConfig::default())
            .expect("QA replay script should be valid");
        let sub_steps = groups
            .iter()
            .map(|group| SubStepReport {
                sub_step: group.sub_step,
                trigger_indexes: group
                    .events
                    .iter()
                    .filter_map(replay_trigger_index)
                    .collect(),
                event_labels: group.events.iter().filter_map(replay_event_label).collect(),
            })
            .collect::<Vec<_>>();

        Self {
            round: script.round,
            event_count: script.events.len(),
            group_count: groups.len(),
            sub_steps,
        }
    }
}

fn app_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins((CardAnimationsPlugin, BoardRenderingPlugin));
    app.init_resource::<ClientPhaseView>();
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    app.update();
    app
}

fn seed_replay(app: &mut App, script: S2CResolutionEvent) {
    *app.world_mut().resource_mut::<CurrentClientPhase>() = CurrentClientPhase {
        phase: RoundPhase::Resolution,
        round: CAPTURE_ROUND,
    };
    *app.world_mut().resource_mut::<ClientPhaseView>() = ClientPhaseView {
        phase: RoundPhase::Resolution,
        round_number: CAPTURE_ROUND,
        timer_duration_ms: 60_000,
    };
    *app.world_mut().resource_mut::<BoardRenderState>() = BoardRenderState::ResolutionReveal;
    app.world_mut().resource_mut::<BoardLocalPlayer>().player_id = Some(LOCAL_PLAYER_ID);
    app.world_mut()
        .resource_mut::<PlayerTeamMap>()
        .insert(LOCAL_PLAYER_ID, 0);
    app.world_mut()
        .resource_mut::<PlayerTeamMap>()
        .insert(OPPONENT_PLAYER_ID, 1);
    app.world_mut()
        .resource_mut::<PendingResolutionScript>()
        .set(script);
    app.world_mut()
        .resource_mut::<ResolutionRevealWait>()
        .start();
    app.world_mut()
        .resource_mut::<PendingPhaseChange>()
        .set(phase_changed(RoundPhase::DraftShop));
    run_for(app, Duration::ZERO);
}

fn capture_step(
    app: &mut App,
    id: &'static str,
    title: &'static str,
    expected: &'static str,
    elapsed_ms: u64,
    readable_result: &'static str,
) -> StepCapture {
    let board_render_state = format!("{:?}", app.world().resource::<BoardRenderState>());
    let current_phase = phase_name(app.world().resource::<CurrentClientPhase>().phase);
    let phase_view = phase_name(app.world().resource::<ClientPhaseView>().phase);
    let pending_phase = app
        .world()
        .resource::<PendingPhaseChange>()
        .phase()
        .map(phase_name);
    let queue = app.world().resource::<AnimQueue>();
    let queue_group_count = queue.groups.len();
    let queue_current_index = (!queue.groups.is_empty()).then_some(queue.current_index);
    let current_sub_step = queue
        .groups
        .get(queue.current_index)
        .map(|group| group.sub_step);
    let queue_empty = queue.groups.is_empty();
    let phase_jump_blocked = !queue_empty
        && current_phase == "Resolution"
        && pending_phase.as_deref() == Some("DraftShop");
    let spawn_highlight_cells = spawn_highlight_cells(app);

    let mut capture = StepCapture {
        id,
        title,
        expected,
        actual: String::new(),
        elapsed_ms,
        board_render_state,
        current_phase,
        phase_view,
        pending_phase,
        queue_group_count,
        queue_current_index,
        current_sub_step,
        queue_empty,
        phase_jump_blocked,
        spawn_highlight_cells,
        readable_result,
    };
    capture.actual = capture.summarize();
    capture
}

fn capture_recovery() -> RecoveryCapture {
    let mut app = app_in_session();
    let mut cursor = drained_cursor::<SnapshotRecoveryRequested>(&app);

    *app.world_mut().resource_mut::<BoardRenderState>() = BoardRenderState::ResolutionReveal;
    app.world_mut()
        .resource_mut::<PendingResolutionScript>()
        .set(invalid_replay_script());
    app.world_mut()
        .resource_mut::<ResolutionRevealWait>()
        .start();

    run_for(&mut app, Duration::ZERO);
    let requests = messages_since(&app, &mut cursor);
    run_for(&mut app, Duration::ZERO);
    let duplicate_requests = messages_since(&app, &mut cursor);
    let pending_script_present = app.world().resource::<PendingResolutionScript>().is_some();
    let queue_empty = app.world().resource::<AnimQueue>().groups.is_empty();
    let reasons = requests
        .iter()
        .map(|request| format!("{:?}", request.reason))
        .collect::<Vec<_>>();
    let actual = format!(
        "requests={}, duplicate_requests_after_second_update={}, reasons={}, pending_script_present={}, queue_empty={}",
        requests.len(),
        duplicate_requests.len(),
        if reasons.is_empty() {
            "none".to_string()
        } else {
            reasons.join(", ")
        },
        pending_script_present,
        queue_empty
    );

    RecoveryCapture {
        id: "recovery-snapshot-requested",
        title: "Out-Of-Range Replay Requests Snapshot",
        expected: "An invalid replay sub-step is rejected, clears pending replay state, resets the queue, and requests exactly one authoritative snapshot.",
        actual,
        request_count: requests.len(),
        duplicate_request_count_after_second_update: duplicate_requests.len(),
        reasons,
        pending_script_present,
        queue_empty,
    }
}

fn run_for(app: &mut App, duration: Duration) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(duration);
    app.update();
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
}

fn replay_script() -> S2CResolutionEvent {
    S2CResolutionEvent {
        round: CAPTURE_ROUND,
        events: vec![
            tagged(1, 10, ResolutionEvent::SubStepBegin),
            tagged(
                1,
                11,
                ResolutionEvent::UnitMoved {
                    unit_id: 1_001,
                    lane: 2,
                    from_cell: 2,
                    to_cell: 3,
                },
            ),
            tagged(
                2,
                20,
                ResolutionEvent::CombatDamage {
                    attacker_id: 1_001,
                    defender_id: 2_001,
                    damage_amount: 2,
                    defender_hp_after: 3,
                    was_blocked_by_shield: false,
                },
            ),
            tagged(
                2,
                21,
                ResolutionEvent::KeywordTriggered {
                    unit_id: 1_001,
                    keyword: KeywordKind::Bodyguard,
                },
            ),
            tagged(
                3,
                30,
                ResolutionEvent::ObjectiveDamage {
                    attacker_id: Some(1_001),
                    target_player_id: OPPONENT_PLAYER_ID,
                    lane: 2,
                    damage_amount: 1,
                    objective_hp_after: 4,
                },
            ),
            tagged(
                3,
                31,
                ResolutionEvent::GoldAwarded {
                    player: LOCAL_PLAYER_ID,
                    amount: 1,
                    reason: GoldReason::Kill,
                },
            ),
            tagged(
                3,
                32,
                ResolutionEvent::SpawnRangeChanged {
                    player_id: LOCAL_PLAYER_ID,
                    new_spawn_range_cells: 3,
                },
            ),
        ],
    }
}

fn invalid_replay_script() -> S2CResolutionEvent {
    S2CResolutionEvent {
        round: CAPTURE_ROUND,
        events: vec![tagged(
            7,
            99,
            ResolutionEvent::GameOver {
                loser: Some(OPPONENT_PLAYER_ID),
                reason: GameOverReason::ObjectivesDestroyed,
            },
        )],
    }
}

fn tagged(sub_step: u8, trigger_index: u32, event: ResolutionEvent) -> TaggedEvent {
    TaggedEvent {
        sub_step,
        trigger_index,
        event,
    }
}

fn phase_changed(phase: RoundPhase) -> S2CPhaseChanged {
    S2CPhaseChanged {
        phase,
        round_number: CAPTURE_ROUND,
        timer_duration_ms: 30_000,
    }
}

fn replay_trigger_index(event: &AnimQueueEvent) -> Option<u32> {
    match event {
        AnimQueueEvent::ResolutionReplay { event } => Some(event.trigger_index),
        AnimQueueEvent::TransformTween { .. } => None,
    }
}

fn replay_event_label(event: &AnimQueueEvent) -> Option<String> {
    match event {
        AnimQueueEvent::ResolutionReplay { event } => Some(event_label(&event.event)),
        AnimQueueEvent::TransformTween { .. } => None,
    }
}

fn event_label(event: &ResolutionEvent) -> String {
    match event {
        ResolutionEvent::SubStepBegin => "SubStepBegin".to_string(),
        ResolutionEvent::UnitPlaced {
            unit_id,
            lane,
            cell,
            ..
        } => format!("UnitPlaced unit {unit_id} lane {lane} cell {cell}"),
        ResolutionEvent::UnitMoved {
            unit_id,
            lane,
            from_cell,
            to_cell,
        } => format!("UnitMoved unit {unit_id} lane {lane} {from_cell}->{to_cell}"),
        ResolutionEvent::UnitChangedLane {
            unit_id,
            from_lane,
            to_lane,
        } => format!("UnitChangedLane unit {unit_id} lane {from_lane}->{to_lane}"),
        ResolutionEvent::CombatDamage {
            attacker_id,
            defender_id,
            damage_amount,
            defender_hp_after,
            was_blocked_by_shield,
        } => format!(
            "CombatDamage attacker {attacker_id} defender {defender_id} -{damage_amount} hp_after {defender_hp_after} shield_blocked {was_blocked_by_shield}"
        ),
        ResolutionEvent::UnitRemoved {
            unit_id,
            lane,
            cell,
        } => format!("UnitRemoved unit {unit_id} lane {lane} cell {cell}"),
        ResolutionEvent::KeywordTriggered { unit_id, keyword } => {
            format!("KeywordTriggered unit {unit_id} {keyword:?}")
        }
        ResolutionEvent::GoldAwarded {
            player,
            amount,
            reason,
        } => format!("GoldAwarded player {} +{} {reason:?}", player.0, amount),
        ResolutionEvent::ObjectiveDamage {
            target_player_id,
            lane,
            damage_amount,
            objective_hp_after,
            ..
        } => format!(
            "ObjectiveDamage player {} lane {} -{} hp_after {}",
            target_player_id.0, lane, damage_amount, objective_hp_after
        ),
        ResolutionEvent::UnitDied {
            unit_id,
            lane,
            cell,
            ..
        } => format!("UnitDied unit {unit_id} lane {lane} cell {cell}"),
        ResolutionEvent::TrapTriggered {
            trap_id,
            triggering_unit_id,
            lane,
            cell,
        } => format!(
            "TrapTriggered trap {trap_id} trigger {triggering_unit_id} lane {lane} cell {cell}"
        ),
        ResolutionEvent::ObjectiveDestroyed {
            target_player_id,
            lane,
            was_fake,
        } => format!(
            "ObjectiveDestroyed player {} lane {} fake {}",
            target_player_id.0, lane, was_fake
        ),
        ResolutionEvent::SpawnRangeChanged {
            player_id,
            new_spawn_range_cells,
        } => format!(
            "SpawnRangeChanged player {} range {}",
            player_id.0, new_spawn_range_cells
        ),
        ResolutionEvent::GameOver { loser, reason } => format!(
            "GameOver loser {} {reason:?}",
            loser
                .map(|player| player.0.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
    }
}

fn spawn_highlight_cells(app: &mut App) -> Vec<String> {
    let mut query = app.world_mut().query::<(&LaneCell, &SpawnHighlightState)>();
    let mut cells = query
        .iter(app.world())
        .filter_map(|(cell, state)| match state {
            SpawnHighlightState::ValidSpawn { player_id } => {
                Some(format!("P{} L{}C{}", player_id.0, cell.lane, cell.cell))
            }
            SpawnHighlightState::Inactive => None,
        })
        .collect::<Vec<_>>();
    cells.sort();
    cells
}

fn phase_name(phase: RoundPhase) -> String {
    format!("{phase:?}")
}

fn drained_cursor<M: Message + Clone>(app: &App) -> MessageCursor<M> {
    let messages = app.world().resource::<Messages<M>>();
    let mut cursor = messages.get_cursor();
    let _ = cursor.read(messages).count();
    cursor
}

fn messages_since<M: Message + Clone>(app: &App, cursor: &mut MessageCursor<M>) -> Vec<M> {
    let messages = app.world().resource::<Messages<M>>();
    cursor.read(messages).cloned().collect()
}

#[cfg(target_arch = "wasm32")]
fn publish_report(report: &CaptureReport) {
    use wasm_bindgen::{JsCast, JsValue};

    let json = serde_json::to_string(report).expect("capture report should serialize");
    let window = web_sys::window().expect("browser window should exist");
    let callback = js_sys::Reflect::get(
        window.as_ref(),
        &JsValue::from_str("qaCond0007ReplayReport"),
    )
    .ok()
    .and_then(|value| value.dyn_into::<js_sys::Function>().ok())
    .expect("qaCond0007ReplayReport callback should exist");
    callback
        .call1(&JsValue::NULL, &JsValue::from_str(&json))
        .expect("QA-COND-0007 replay report callback should accept JSON");
}

#[cfg(not(target_arch = "wasm32"))]
fn publish_report(report: &CaptureReport) {
    println!(
        "{}",
        serde_json::to_string_pretty(report).expect("capture report should serialize")
    );
}
