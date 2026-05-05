use std::fmt::Write as _;

use bevy::asset::RenderAssetUsages;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::card::{CardId, ClassId};
use shared::protocol::{
    BoardSnapshot, ObjectiveSnapshot, PlayerSnapshot, RoundPhase, S2CGameSnapshot,
    UnitBoardLocation, UnitBoardState, UnitStatsSnapshot,
};
use shared::session::PlayerId;

use crate::card_animations::BoardRebuildRequested;
use crate::presentation::board_rendering::{
    BoardCellNode, BoardRenderSet, BoardSnapshotEntity, BoardUnit, CardAtlas, GhostUnit,
    HpBarBackground, HpBarFill, LaneGhostWash, SpawnHighlightState, StandingObjective, StatusIcon,
    StatusOverflowBadge,
};
use crate::state::{ClientGameSnapshotMessage, ClientState};
use crate::ui::shared::{LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT};

pub const BOARD_RENDERING_BASELINE_SEED: &str = "board-rendering-baseline-v1";
pub const BOARD_RENDERING_BASELINE_SCREENSHOT_PATH: &str =
    "production/qa/evidence/captures/board-rendering-baseline-1920x1080.png";
pub const BOARD_RENDERING_BASELINE_TRACE_PATH: &str =
    "production/qa/evidence/captures/board-rendering-baseline-timing.json";
pub const BOARD_RENDERING_BASELINE_VIEWPORT_WIDTH: u32 = 1920;
pub const BOARD_RENDERING_BASELINE_VIEWPORT_HEIGHT: u32 = 1080;

const SAMPLE_WARMUP_FRAMES: u32 = 30;
const SAMPLE_FRAME_COUNT: usize = 120;
const UNIT_CARD: CardId = CardId(10);
const UNIT_CARD_FRAME: usize = 7;
const UNIT_CARD_MAX_HP: u8 = 5;
const ATLAS_FRAME_SIZE: u32 = 16;
const ATLAS_COLUMNS: u32 = 8;
const TOTAL_FRAME_BUDGET_MS: f64 = 16.67;
const STEADY_STATE_PRESENTATION_BUDGET_MS: f64 = 1.0;
const PHASE_BOUNDARY_REBUILD_SPIKE_BUDGET_MS: f64 = 3.0;

pub struct BoardRenderingPerfHarnessPlugin;

impl Plugin for BoardRenderingPerfHarnessPlugin {
    fn build(&self, app: &mut App) {
        add_board_rendering_perf_harness(app);
    }
}

pub struct BoardWasmPerfHarnessPlugin;

impl Plugin for BoardWasmPerfHarnessPlugin {
    fn build(&self, app: &mut App) {
        add_board_rendering_perf_harness(app);
    }
}

fn add_board_rendering_perf_harness(app: &mut App) {
    app.init_resource::<BoardRenderingPerfHarnessState>()
        .init_resource::<Assets<Image>>()
        .init_resource::<Assets<TextureAtlasLayout>>()
        .init_resource::<BoardWasmPerfHarnessReport>()
        .add_systems(Startup, enter_harness_session_system)
        .add_systems(
            Update,
            seed_baseline_fixture_system
                .before(BoardRenderSet::ReadMessages)
                .run_if(in_state(ClientState::InSession)),
        )
        .add_systems(
            Update,
            begin_board_timing_sample_system
                .before(BoardRenderSet::ReadMessages)
                .run_if(in_state(ClientState::InSession)),
        )
        .add_systems(
            Update,
            clear_baseline_spawn_highlights_system
                .after(BoardRenderSet::UpdateHpBars)
                .run_if(in_state(ClientState::InSession)),
        )
        .add_systems(
            Update,
            finish_board_timing_sample_system
                .after(clear_baseline_spawn_highlights_system)
                .run_if(in_state(ClientState::InSession)),
        );
}

#[derive(Resource, Debug)]
struct BoardRenderingPerfHarnessState {
    seeded: bool,
    stable_frames: u32,
    sample_start: Option<Instant>,
    steady_state_samples: Vec<BoardWasmPerfTimingSample>,
    rebuild_spike_ms: Option<f64>,
    published: bool,
}

impl Default for BoardRenderingPerfHarnessState {
    fn default() -> Self {
        Self {
            seeded: false,
            stable_frames: 0,
            sample_start: None,
            steady_state_samples: Vec::with_capacity(SAMPLE_FRAME_COUNT),
            rebuild_spike_ms: None,
            published: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoardWasmPerfTimingSample {
    pub total_frame_ms: f64,
    pub presentation_ms: f64,
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct BoardWasmPerfHarnessReport {
    pub seed: &'static str,
    pub fixture_counts: BoardWasmPerfFixtureCounts,
    pub ready_for_capture: bool,
    pub total_frame_budget: PerfBudgetStatus,
    pub steady_state_presentation_budget: PerfBudgetStatus,
    pub phase_boundary_rebuild_spike_budget: PerfBudgetStatus,
    pub total_frame_avg_ms: Option<f64>,
    pub total_frame_max_ms: Option<f64>,
    pub presentation_avg_ms: Option<f64>,
    pub presentation_max_ms: Option<f64>,
    pub rebuild_spike_ms: Option<f64>,
    pub raw_total_frame_ms: Vec<f64>,
    pub raw_presentation_ms: Vec<f64>,
    pub screenshot_path: &'static str,
    pub trace_path: &'static str,
}

impl Default for BoardWasmPerfHarnessReport {
    fn default() -> Self {
        Self {
            seed: BOARD_RENDERING_BASELINE_SEED,
            fixture_counts: BoardWasmPerfFixtureCounts::default(),
            ready_for_capture: false,
            total_frame_budget: PerfBudgetStatus::NotSampled,
            steady_state_presentation_budget: PerfBudgetStatus::NotSampled,
            phase_boundary_rebuild_spike_budget: PerfBudgetStatus::NotSampled,
            total_frame_avg_ms: None,
            total_frame_max_ms: None,
            presentation_avg_ms: None,
            presentation_max_ms: None,
            rebuild_spike_ms: None,
            raw_total_frame_ms: Vec::new(),
            raw_presentation_ms: Vec::new(),
            screenshot_path: BOARD_RENDERING_BASELINE_SCREENSHOT_PATH,
            trace_path: BOARD_RENDERING_BASELINE_TRACE_PATH,
        }
    }
}

impl BoardWasmPerfHarnessReport {
    pub fn to_json(&self) -> String {
        let mut json = String::new();
        let _ = write!(
            json,
            "{{\"seed\":\"{}\",\"viewport\":{{\"width\":{},\"height\":{}}},",
            self.seed,
            BOARD_RENDERING_BASELINE_VIEWPORT_WIDTH,
            BOARD_RENDERING_BASELINE_VIEWPORT_HEIGHT
        );
        let _ = write!(
            json,
            "\"fixture_counts\":{{\"lanes\":{},\"board_cells\":{},\"visible_units\":{},\"objectives\":{},\"units_with_hp_bars\":{},\"post_reveal_ready_units\":{},\"status_icons\":{},\"spawn_range_highlights\":{},\"ghost_units\":{},\"lane_ghost_washes\":{}}},",
            self.fixture_counts.lanes,
            self.fixture_counts.board_cells,
            self.fixture_counts.visible_units,
            self.fixture_counts.objectives,
            self.fixture_counts.units_with_hp_bars,
            self.fixture_counts.post_reveal_ready_units,
            self.fixture_counts.status_icons,
            self.fixture_counts.spawn_range_highlights,
            self.fixture_counts.ghost_units,
            self.fixture_counts.lane_ghost_washes
        );
        let _ = write!(
            json,
            "\"budgets_ms\":{{\"total_frame\":{},\"steady_state_presentation\":{},\"phase_boundary_rebuild_spike\":{}}},",
            TOTAL_FRAME_BUDGET_MS,
            STEADY_STATE_PRESENTATION_BUDGET_MS,
            PHASE_BOUNDARY_REBUILD_SPIKE_BUDGET_MS
        );
        let _ = write!(
            json,
            "\"ready_for_capture\":{},\"status\":{{\"total_frame\":\"{}\",\"steady_state_presentation\":\"{}\",\"phase_boundary_rebuild_spike\":\"{}\"}},",
            self.ready_for_capture,
            self.total_frame_budget.as_str(),
            self.steady_state_presentation_budget.as_str(),
            self.phase_boundary_rebuild_spike_budget.as_str()
        );
        let _ = write!(
            json,
            "\"summary_ms\":{{\"total_frame_avg\":{},\"total_frame_max\":{},\"presentation_avg\":{},\"presentation_max\":{},\"rebuild_spike\":{}}},",
            optional_ms(self.total_frame_avg_ms),
            optional_ms(self.total_frame_max_ms),
            optional_ms(self.presentation_avg_ms),
            optional_ms(self.presentation_max_ms),
            optional_ms(self.rebuild_spike_ms)
        );
        let _ = write!(
            json,
            "\"artifacts\":{{\"screenshot\":\"{}\",\"trace\":\"{}\"}},",
            self.screenshot_path, self.trace_path
        );
        json.push_str("\"raw_total_frame_ms\":");
        push_json_number_array(&mut json, &self.raw_total_frame_ms);
        json.push_str(",\"raw_presentation_ms\":");
        push_json_number_array(&mut json, &self.raw_presentation_ms);
        json.push('}');
        json
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfBudgetStatus {
    NotSampled,
    Pass,
    Fail,
}

impl PerfBudgetStatus {
    fn from_max(value: Option<f64>, budget_ms: f64, inclusive: bool) -> Self {
        match value {
            Some(value) if inclusive && value <= budget_ms => Self::Pass,
            Some(value) if !inclusive && value < budget_ms => Self::Pass,
            Some(_) => Self::Fail,
            None => Self::NotSampled,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::NotSampled => "not_sampled",
            Self::Pass => "pass",
            Self::Fail => "fail",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardWasmPerfFixtureCounts {
    pub lanes: u8,
    pub board_cells: usize,
    pub visible_units: usize,
    pub objectives: usize,
    pub units_with_hp_bars: usize,
    pub post_reveal_ready_units: usize,
    pub status_icons: usize,
    pub spawn_range_highlights: usize,
    pub ghost_units: usize,
    pub lane_ghost_washes: usize,
}

impl Default for BoardWasmPerfFixtureCounts {
    fn default() -> Self {
        Self {
            lanes: BOARD_LANE_COUNT,
            board_cells: 0,
            visible_units: 0,
            objectives: 0,
            units_with_hp_bars: 0,
            post_reveal_ready_units: 0,
            status_icons: 0,
            spawn_range_highlights: 0,
            ghost_units: 0,
            lane_ghost_washes: 0,
        }
    }
}

impl BoardWasmPerfFixtureCounts {
    pub fn expected() -> Self {
        Self {
            lanes: BOARD_LANE_COUNT,
            board_cells: usize::from(BOARD_LANE_COUNT) * usize::from(BOARD_CELL_COUNT),
            visible_units: 20,
            objectives: 10,
            units_with_hp_bars: 20,
            post_reveal_ready_units: 20,
            status_icons: 0,
            spawn_range_highlights: 0,
            ghost_units: 0,
            lane_ghost_washes: 0,
        }
    }

    pub fn matches_expected(&self) -> bool {
        self == &Self::expected()
    }
}

fn enter_harness_session_system(mut next_state: ResMut<NextState<ClientState>>) {
    next_state.set(ClientState::InSession);
}

fn seed_baseline_fixture_system(
    mut state: ResMut<BoardRenderingPerfHarnessState>,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut card_atlas: Option<ResMut<CardAtlas>>,
    mut snapshots: MessageWriter<ClientGameSnapshotMessage>,
) {
    if state.seeded {
        return;
    }

    let Some(card_atlas) = card_atlas.as_deref_mut() else {
        return;
    };

    *card_atlas = baseline_card_atlas(&mut images, &mut layouts);
    snapshots.write(ClientGameSnapshotMessage(baseline_fixture_snapshot()));
    state.seeded = true;
}

fn begin_board_timing_sample_system(mut state: ResMut<BoardRenderingPerfHarnessState>) {
    state.sample_start = Some(Instant::now());
}

fn clear_baseline_spawn_highlights_system(
    mut cells: Query<(&mut SpawnHighlightState, &mut Sprite), With<BoardCellNode>>,
) {
    for (mut highlight_state, mut sprite) in &mut cells {
        *highlight_state = SpawnHighlightState::Inactive;
        sprite.color = SpawnHighlightState::Inactive.tint();
    }
}

#[allow(clippy::too_many_arguments)]
fn finish_board_timing_sample_system(
    time: Res<Time<Real>>,
    mut state: ResMut<BoardRenderingPerfHarnessState>,
    mut report: ResMut<BoardWasmPerfHarnessReport>,
    mut rebuilds: MessageReader<BoardRebuildRequested>,
    cells: Query<(&LaneCell, &SpawnHighlightState), With<BoardCellNode>>,
    units: Query<
        (&Transform, &Sprite, Option<&Children>),
        (With<BoardUnit>, With<BoardSnapshotEntity>),
    >,
    objectives: Query<Entity, With<StandingObjective>>,
    hp_backgrounds: Query<Entity, With<HpBarBackground>>,
    hp_fills: Query<Entity, With<HpBarFill>>,
    status_icons: Query<Entity, Or<(With<StatusIcon>, With<StatusOverflowBadge>)>>,
    ghost_units: Query<Entity, With<GhostUnit>>,
    lane_ghost_washes: Query<Entity, With<LaneGhostWash>>,
) {
    let presentation_ms = state
        .sample_start
        .take()
        .map(|start| round_ms(start.elapsed().as_secs_f64() * 1_000.0))
        .unwrap_or(0.0);

    if rebuilds.read().next().is_some() {
        state.rebuild_spike_ms = Some(presentation_ms);
    }

    let counts = collect_fixture_counts(
        &cells,
        &units,
        &objectives,
        &hp_backgrounds,
        &hp_fills,
        &status_icons,
        &ghost_units,
        &lane_ghost_washes,
    );
    report.fixture_counts = counts.clone();
    report.ready_for_capture = counts.matches_expected();

    if !counts.matches_expected() {
        state.stable_frames = 0;
        update_report_from_samples(&mut report, &state);
        return;
    }

    state.stable_frames = state.stable_frames.saturating_add(1);
    if state.stable_frames > SAMPLE_WARMUP_FRAMES
        && state.steady_state_samples.len() < SAMPLE_FRAME_COUNT
    {
        state.steady_state_samples.push(BoardWasmPerfTimingSample {
            total_frame_ms: round_ms(time.delta_secs_f64() * 1_000.0),
            presentation_ms,
        });
    }

    update_report_from_samples(&mut report, &state);

    if !state.published
        && report.ready_for_capture
        && state.steady_state_samples.len() >= SAMPLE_FRAME_COUNT
        && state.rebuild_spike_ms.is_some()
    {
        publish_report(&report);
        state.published = true;
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_fixture_counts(
    cells: &Query<(&LaneCell, &SpawnHighlightState), With<BoardCellNode>>,
    units: &Query<
        (&Transform, &Sprite, Option<&Children>),
        (With<BoardUnit>, With<BoardSnapshotEntity>),
    >,
    objectives: &Query<Entity, With<StandingObjective>>,
    hp_backgrounds: &Query<Entity, With<HpBarBackground>>,
    hp_fills: &Query<Entity, With<HpBarFill>>,
    status_icons: &Query<Entity, Or<(With<StatusIcon>, With<StatusOverflowBadge>)>>,
    ghost_units: &Query<Entity, With<GhostUnit>>,
    lane_ghost_washes: &Query<Entity, With<LaneGhostWash>>,
) -> BoardWasmPerfFixtureCounts {
    let units_with_hp_bars = units
        .iter()
        .filter(|(_transform, _sprite, children)| {
            let Some(children) = children else {
                return false;
            };
            let has_background = children
                .iter()
                .any(|child| hp_backgrounds.get(child).is_ok());
            let has_fill = children.iter().any(|child| hp_fills.get(child).is_ok());
            has_background && has_fill
        })
        .count();
    let post_reveal_ready_units = units
        .iter()
        .filter(|(transform, sprite, _children)| {
            transform.scale == Vec3::ONE && sprite.color == Color::srgba(1.0, 1.0, 1.0, 1.0)
        })
        .count();
    let mut lane_seen = [false; BOARD_LANE_COUNT as usize];
    let mut board_cells = 0usize;
    let mut spawn_range_highlights = 0usize;
    for (lane_cell, highlight_state) in cells {
        board_cells += 1;
        if (1..=BOARD_LANE_COUNT).contains(&lane_cell.lane) {
            lane_seen[usize::from(lane_cell.lane - 1)] = true;
        }
        if *highlight_state != SpawnHighlightState::Inactive {
            spawn_range_highlights += 1;
        }
    }

    BoardWasmPerfFixtureCounts {
        lanes: lane_seen.iter().filter(|seen| **seen).count() as u8,
        board_cells,
        visible_units: units.iter().count(),
        objectives: objectives.iter().count(),
        units_with_hp_bars,
        post_reveal_ready_units,
        status_icons: status_icons.iter().count(),
        spawn_range_highlights,
        ghost_units: ghost_units.iter().count(),
        lane_ghost_washes: lane_ghost_washes.iter().count(),
    }
}

fn update_report_from_samples(
    report: &mut BoardWasmPerfHarnessReport,
    state: &BoardRenderingPerfHarnessState,
) {
    report.raw_total_frame_ms = state
        .steady_state_samples
        .iter()
        .map(|sample| sample.total_frame_ms)
        .collect();
    report.raw_presentation_ms = state
        .steady_state_samples
        .iter()
        .map(|sample| sample.presentation_ms)
        .collect();
    report.total_frame_avg_ms = average(&report.raw_total_frame_ms);
    report.total_frame_max_ms = max(&report.raw_total_frame_ms);
    report.presentation_avg_ms = average(&report.raw_presentation_ms);
    report.presentation_max_ms = max(&report.raw_presentation_ms);
    report.rebuild_spike_ms = state.rebuild_spike_ms.map(round_ms);
    report.total_frame_budget =
        PerfBudgetStatus::from_max(report.total_frame_max_ms, TOTAL_FRAME_BUDGET_MS, true);
    report.steady_state_presentation_budget = PerfBudgetStatus::from_max(
        report.presentation_max_ms,
        STEADY_STATE_PRESENTATION_BUDGET_MS,
        false,
    );
    report.phase_boundary_rebuild_spike_budget = PerfBudgetStatus::from_max(
        report.rebuild_spike_ms,
        PHASE_BOUNDARY_REBUILD_SPIKE_BUDGET_MS,
        false,
    );
}

pub fn baseline_fixture_snapshot() -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: player(1),
        round_number: 4,
        phase: RoundPhase::Resolution,
        timer_remaining_ms: Some(20_000),
        players: vec![player_snapshot(player(1)), player_snapshot(player(2))],
        board: BoardSnapshot {
            units: baseline_units(),
            ..default()
        },
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn baseline_units() -> Vec<UnitBoardState> {
    let mut units = Vec::with_capacity(20);
    let mut unit_id = 1_000;

    for lane in 1..=BOARD_LANE_COUNT {
        for (owner_id, cell, hp) in [
            (player(1), 2, 5),
            (player(2), 3, 4),
            (player(1), 6, 3),
            (player(2), 7, 2),
        ] {
            units.push(UnitBoardState {
                unit_id,
                owner_id,
                location: UnitBoardLocation::BoardCell { lane, cell },
                card_id: Some(UNIT_CARD),
                stats: Some(UnitStatsSnapshot {
                    hp,
                    atk: 2,
                    mp: 3,
                    ar: 0,
                }),
                source_class: None,
            });
            unit_id += 1;
        }
    }

    units
}

fn player_snapshot(player_id: PlayerId) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id: ClassId::Iop,
        gold: 0,
        reserved_gold: 0,
        current_mana: 0,
        reserve_mana: 0,
        spawn_range_cells: 1,
        mana_cap: 1,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: objectives([true, false, true, false, true]),
        opponent_objectives: Vec::new(),
    }
}

fn objectives(real_flags: [bool; 5]) -> Vec<ObjectiveSnapshot> {
    real_flags
        .into_iter()
        .enumerate()
        .map(|(index, is_real)| ObjectiveSnapshot {
            lane: index as u8 + 1,
            hp: 5,
            is_real,
            is_destroyed: false,
        })
        .collect()
}

fn baseline_card_atlas(
    images: &mut Assets<Image>,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> CardAtlas {
    let unit_image = images.add(solid_atlas_image([255, 255, 255, 255]));
    let board_elements_image = images.add(solid_atlas_image([255, 190, 68, 255]));
    let unit_layout = layouts.add(atlas_layout());
    let board_elements_layout = layouts.add(atlas_layout());

    CardAtlas {
        image: unit_image,
        layout: unit_layout,
        board_elements_image,
        board_elements_layout,
        unit_frames: default(),
    }
    .with_unit_frame(UNIT_CARD, UNIT_CARD_FRAME, UNIT_CARD_MAX_HP)
}

fn atlas_layout() -> TextureAtlasLayout {
    TextureAtlasLayout::from_grid(UVec2::splat(ATLAS_FRAME_SIZE), ATLAS_COLUMNS, 1, None, None)
}

fn solid_atlas_image(color: [u8; 4]) -> Image {
    Image::new_fill(
        Extent3d {
            width: ATLAS_FRAME_SIZE * ATLAS_COLUMNS,
            height: ATLAS_FRAME_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &color,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn average(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(round_ms(values.iter().sum::<f64>() / values.len() as f64))
    }
}

fn max(values: &[f64]) -> Option<f64> {
    values.iter().copied().reduce(f64::max).map(round_ms)
}

fn round_ms(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

fn optional_ms(value: Option<f64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}

fn push_json_number_array(json: &mut String, values: &[f64]) {
    json.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        let _ = write!(json, "{value}");
    }
    json.push(']');
}

fn publish_report(report: &BoardWasmPerfHarnessReport) {
    let json = report.to_json();
    info!("BOARD-012 harness result {json}");
    publish_report_to_browser(&json);
}

#[cfg(target_arch = "wasm32")]
fn publish_report_to_browser(json: &str) {
    use wasm_bindgen::prelude::*;

    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(callback) =
        js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("boardWasmPerfReport"))
    else {
        return;
    };
    if let Some(callback) = callback.dyn_ref::<js_sys::Function>() {
        let _ = callback.call1(window.as_ref(), &JsValue::from_str(json));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn publish_report_to_browser(json: &str) {
    let _ = json;
}
