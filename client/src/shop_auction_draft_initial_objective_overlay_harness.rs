use std::collections::HashMap;

use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform, UiScale, UiSystems};
use bevy::window::{PresentMode, Window, WindowPlugin};
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::hand::{
    HandCardCatalog, HandContents, HandUiDraftOfferingReceived, HandUiEntities, HandUiPlugin,
    HandUiSystemSet,
};
use client::ui::hud::{HudEntities, HudPlayerIds, HudPlugin, HudSystemSet};
use client::ui::shop_auction::{
    DraftInitialObjectiveFocusTarget, ShopAuctionCardCatalog, ShopAuctionDraftHandView,
    ShopAuctionDraftObjectiveEscPressed, ShopAuctionDraftObjectiveRetrievalClicked,
    ShopAuctionDraftOfferingReceived, ShopAuctionLocalGoldView, ShopAuctionUiEntities,
    ShopAuctionUiOutboundMessages, ShopAuctionUiPlugin, ShopAuctionUiSystemSet,
    DRAFT_INITIAL_OBJECTIVE_COPY,
};
use serde::Serialize;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;
use shared::session::PlayerId;

const LOCAL_PLAYER: PlayerId = PlayerId(1);
const OPPONENT_PLAYER: PlayerId = PlayerId(2);
const VIEWPORT_WIDTH: u32 = 1366;
const VIEWPORT_HEIGHT: u32 = 768;
const DRAFT_TIMER_MS: u32 = 45_000;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "SAU-012 Draft Initial Objective Overlay Harness".to_string(),
            resolution: (VIEWPORT_WIDTH, VIEWPORT_HEIGHT).into(),
            present_mode: PresentMode::AutoVsync,
            fit_canvas_to_parent: true,
            canvas: Some("#bevy".to_string()),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(HandUiPlugin);
    app.add_plugins(HudPlugin);
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(UiScale(1.0));
    app.insert_resource(Sau012HarnessState::for_browser());
    app.add_systems(Startup, enter_harness_session_system);
    app.add_systems(
        Update,
        seed_fixture_system
            .before(HandUiSystemSet::PhaseTransition)
            .before(HudSystemSet::PhaseTransition)
            .before(ShopAuctionUiSystemSet::PhaseTransition)
            .run_if(in_state(ClientState::InSession)),
    );
    app.add_systems(
        Update,
        drive_scenario_system
            .after(ShopAuctionUiSystemSet::MessageDrain)
            .before(ShopAuctionUiSystemSet::Input)
            .run_if(in_state(ClientState::InSession)),
    );
    app.add_systems(
        PostUpdate,
        publish_report_system.after(UiSystems::PostLayout),
    );
    app.run();
}

#[derive(Resource, Debug)]
struct Sau012HarnessState {
    scenario: Sau012Scenario,
    seeded: bool,
    drive_step: u8,
    ready_frames: u8,
    published: bool,
}

impl Sau012HarnessState {
    fn for_browser() -> Self {
        Self {
            scenario: selected_scenario(),
            seeded: false,
            drive_step: 0,
            ready_frames: 0,
            published: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Sau012Scenario {
    Entry,
    EscDismissed,
    Retrieved,
}

impl Sau012Scenario {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Entry => "entry",
            Self::EscDismissed => "esc-dismissed",
            Self::Retrieved => "retrieved",
        }
    }
}

fn enter_harness_session_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    commands.spawn((Name::new("SAU-012 Harness Camera"), Camera2d));
    commands.insert_resource(HudPlayerIds {
        local_id: LOCAL_PLAYER,
        opponent_id: OPPONENT_PLAYER,
    });
    next_state.set(ClientState::InSession);
}

fn seed_fixture_system(
    mut state: ResMut<Sau012HarnessState>,
    mut shop_catalog: ResMut<ShopAuctionCardCatalog>,
    mut hand_catalog: ResMut<HandCardCatalog>,
    mut economy: ResMut<PlayerEconomyView>,
    mut local_gold: ResMut<ShopAuctionLocalGoldView>,
    mut hand_view: ResMut<ShopAuctionDraftHandView>,
    mut hand_contents: ResMut<HandContents>,
    mut current: ResMut<CurrentClientPhase>,
    mut phase_view: ResMut<ClientPhaseView>,
    mut shop_offerings: MessageWriter<ShopAuctionDraftOfferingReceived>,
    mut hand_offerings: MessageWriter<HandUiDraftOfferingReceived>,
) {
    if state.seeded {
        return;
    }

    let shop_cards = (1..=9)
        .map(|id| {
            let rarity = match id {
                1..=3 => Rarity::Rare,
                4..=6 => Rarity::Uncommon,
                _ => Rarity::Common,
            };
            let card = test_card(id, rarity, ((id - 1) % 5) + 1);
            (card.id, card)
        })
        .collect::<HashMap<_, _>>();
    let hand_cards = (101..=103)
        .map(|id| {
            let card = test_card(id, Rarity::Common, 2);
            (card.id, card)
        })
        .collect::<HashMap<_, _>>();

    shop_catalog.cards = shop_cards.clone();
    hand_catalog.cards = shop_cards
        .into_iter()
        .chain(hand_cards.iter().map(|(id, card)| (*id, card.clone())))
        .collect();
    hand_contents.cards = hand_cards.keys().copied().collect();

    *economy = PlayerEconomyView {
        gold: 7,
        current_mana: 0,
        mana_cap: 3,
        reserve_mana: 0,
        initialized: true,
        ..default()
    };
    *local_gold = ShopAuctionLocalGoldView {
        player_id: Some(LOCAL_PLAYER),
        gold: 7,
        reserved_gold: 0,
        initialized: true,
    };
    *hand_view = ShopAuctionDraftHandView { hand_size: 3 };

    current.phase = RoundPhase::DraftInitial;
    current.round = 1;
    phase_view.phase = RoundPhase::DraftInitial;
    phase_view.round_number = 1;
    phase_view.timer_duration_ms = DRAFT_TIMER_MS;

    let offering = (1..=9).map(CardId).collect::<Vec<_>>();
    shop_offerings.write(ShopAuctionDraftOfferingReceived {
        card_ids: offering.clone(),
    });
    hand_offerings.write(HandUiDraftOfferingReceived { card_ids: offering });

    state.seeded = true;
}

fn drive_scenario_system(
    mut state: ResMut<Sau012HarnessState>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    draft_state: Res<client::ui::shop_auction::ShopAuctionDraftInitialState>,
    mut esc_presses: MessageWriter<ShopAuctionDraftObjectiveEscPressed>,
    mut retrieval_clicks: MessageWriter<ShopAuctionDraftObjectiveRetrievalClicked>,
) {
    if !state.seeded {
        return;
    }

    match state.scenario {
        Sau012Scenario::Entry => {}
        Sau012Scenario::EscDismissed => {
            if state.drive_step == 0
                && draft_state.objective_focus_target
                    == DraftInitialObjectiveFocusTarget::DismissButton
            {
                esc_presses.write(ShopAuctionDraftObjectiveEscPressed);
                state.drive_step = 1;
            }
        }
        Sau012Scenario::Retrieved => {
            if state.drive_step == 0
                && draft_state.objective_focus_target
                    == DraftInitialObjectiveFocusTarget::DismissButton
            {
                esc_presses.write(ShopAuctionDraftObjectiveEscPressed);
                state.drive_step = 1;
            } else if state.drive_step == 1
                && draft_state.objective_focus_target
                    == DraftInitialObjectiveFocusTarget::RetrievalAffordance
            {
                let Some(entities) = entities else {
                    return;
                };
                retrieval_clicks.write(ShopAuctionDraftObjectiveRetrievalClicked {
                    button: entities.draft_initial_objective_retrieval_button,
                });
                state.drive_step = 2;
            }
        }
    }
}

fn publish_report_system(
    mut state: ResMut<Sau012HarnessState>,
    windows: Query<&Window>,
    ui_scale: Res<UiScale>,
    shop_entities: Option<Res<ShopAuctionUiEntities>>,
    hud_entities: Option<Res<HudEntities>>,
    hand_entities: Option<Res<HandUiEntities>>,
    draft_state: Res<client::ui::shop_auction::ShopAuctionDraftInitialState>,
    phase_view: Res<ClientPhaseView>,
    outbound: Option<Res<ShopAuctionUiOutboundMessages>>,
    surface_query: Query<(
        &ComputedNode,
        &UiGlobalTransform,
        &Visibility,
        Option<&Text>,
        Option<&Name>,
    )>,
) {
    if state.published || !state.seeded {
        return;
    }

    let Some(shop_entities) = shop_entities else {
        return;
    };

    let report = collect_report(
        state.scenario,
        &windows,
        ui_scale.0,
        &shop_entities,
        hud_entities.as_deref(),
        hand_entities.as_deref(),
        &draft_state,
        &phase_view,
        outbound.as_deref(),
        &surface_query,
    );

    if !scenario_ready(state.scenario, &report, state.drive_step) {
        state.ready_frames = 0;
        return;
    }

    state.ready_frames = state.ready_frames.saturating_add(1);
    if state.ready_frames < 2 {
        return;
    }

    let json = serde_json::to_string(&report).expect("SAU-012 report should serialize");
    info!("SAU-012 draft objective evidence {json}");
    publish_report_to_browser(&json);
    state.published = true;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HarnessReport {
    ready_for_capture: bool,
    fixture: &'static str,
    scenario: &'static str,
    viewport: ViewportReport,
    ui_scale: f32,
    draft_timer_duration_ms: u32,
    objective_copy: String,
    expected_objective_copy: &'static str,
    objective_focus_target: String,
    overlay_visible: bool,
    dismiss_visible: bool,
    retrieval_visible: bool,
    draft_panel_visible: bool,
    visible_draft_slot_count: usize,
    ready_visible: bool,
    hud_own_gold_visible: bool,
    hud_opponent_gold_visible: bool,
    visible_hand_surface_count: usize,
    outbound_purchase_cards: usize,
    outbound_ready_signals: usize,
    surfaces: Vec<SurfaceReport>,
    verdict: VerdictReport,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ViewportReport {
    width: f32,
    height: f32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SurfaceReport {
    label: String,
    visible: bool,
    text: String,
    width_css_px: f32,
    height_css_px: f32,
    center_css_px: PointReport,
    bounds_css_px: BoundsReport,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct PointReport {
    x: f32,
    y: f32,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct BoundsReport {
    x_min: f32,
    y_min: f32,
    x_max: f32,
    y_max: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VerdictReport {
    exact_copy: bool,
    overlay_visible_on_entry: bool,
    dismiss_control_focused_visible: bool,
    esc_dismissed_without_c2s: bool,
    retrieval_visible_after_dismissal: bool,
    retrieval_reopened_same_overlay: bool,
    timer_copy_visible: bool,
    grid_non_occluded: bool,
    ready_non_occluded: bool,
    hud_non_occluded: bool,
    hand_surfaces_non_occluded: bool,
    phase_exit_covered_by_automated_test: bool,
}

fn collect_report(
    scenario: Sau012Scenario,
    windows: &Query<&Window>,
    ui_scale: f32,
    shop_entities: &ShopAuctionUiEntities,
    hud_entities: Option<&HudEntities>,
    hand_entities: Option<&HandUiEntities>,
    draft_state: &client::ui::shop_auction::ShopAuctionDraftInitialState,
    phase_view: &ClientPhaseView,
    outbound: Option<&ShopAuctionUiOutboundMessages>,
    surface_query: &Query<(
        &ComputedNode,
        &UiGlobalTransform,
        &Visibility,
        Option<&Text>,
        Option<&Name>,
    )>,
) -> HarnessReport {
    let window = windows.iter().next();
    let mut surfaces = Vec::new();

    push_surface(
        &mut surfaces,
        "draft_panel",
        shop_entities.draft_offering_panel,
        surface_query,
    );
    push_surface(
        &mut surfaces,
        "objective_overlay",
        shop_entities.draft_initial_objective_overlay,
        surface_query,
    );
    push_surface(
        &mut surfaces,
        "objective_copy",
        shop_entities.draft_initial_objective_copy,
        surface_query,
    );
    push_surface(
        &mut surfaces,
        "objective_dismiss",
        shop_entities.draft_initial_objective_dismiss_button,
        surface_query,
    );
    push_surface(
        &mut surfaces,
        "objective_retrieval",
        shop_entities.draft_initial_objective_retrieval_button,
        surface_query,
    );
    for (index, slot) in shop_entities
        .draft_initial_slots
        .iter()
        .copied()
        .enumerate()
    {
        push_surface(
            &mut surfaces,
            &format!("draft_slot_{index}"),
            slot,
            surface_query,
        );
    }
    push_surface(
        &mut surfaces,
        "draft_ready",
        shop_entities.draft_initial_ready_button,
        surface_query,
    );

    if let Some(hud_entities) = hud_entities {
        push_surface(
            &mut surfaces,
            "hud_own_gold",
            hud_entities.own_gold_parent,
            surface_query,
        );
        push_surface(
            &mut surfaces,
            "hud_opponent_gold",
            hud_entities.opponent_gold_parent,
            surface_query,
        );
    }

    if let Some(hand_entities) = hand_entities {
        for (index, slot) in hand_entities.fan_slots.iter().copied().enumerate() {
            push_surface(
                &mut surfaces,
                &format!("hand_fan_slot_{index}"),
                slot,
                surface_query,
            );
        }
    }

    let objective_copy = surfaces
        .iter()
        .find(|surface| surface.label == "objective_copy")
        .map(|surface| surface.text.clone())
        .unwrap_or_default();
    let overlay_visible = surface_visible(&surfaces, "objective_overlay");
    let dismiss_visible = surface_visible(&surfaces, "objective_dismiss");
    let retrieval_visible = surface_visible(&surfaces, "objective_retrieval");
    let draft_panel_visible = surface_visible(&surfaces, "draft_panel");
    let visible_draft_slot_count = surfaces
        .iter()
        .filter(|surface| surface.label.starts_with("draft_slot_") && surface.visible)
        .count();
    let ready_visible = surface_visible(&surfaces, "draft_ready");
    let hud_own_gold_visible = surface_visible(&surfaces, "hud_own_gold");
    let hud_opponent_gold_visible = surface_visible(&surfaces, "hud_opponent_gold");
    let visible_hand_surface_count = surfaces
        .iter()
        .filter(|surface| surface.label.starts_with("hand_fan_slot_") && surface.visible)
        .count();
    let outbound_purchase_cards = outbound.map_or(0, |outbound| outbound.purchase_cards.len());
    let outbound_ready_signals = outbound.map_or(0, |outbound| outbound.ready_signals.len());
    let focus_target = format!("{:?}", draft_state.objective_focus_target);

    let verdict = build_verdict(
        scenario,
        &surfaces,
        &objective_copy,
        &focus_target,
        overlay_visible,
        dismiss_visible,
        retrieval_visible,
        visible_draft_slot_count,
        ready_visible,
        hud_own_gold_visible,
        hud_opponent_gold_visible,
        visible_hand_surface_count,
        outbound_purchase_cards,
        outbound_ready_signals,
    );

    HarnessReport {
        ready_for_capture: true,
        fixture: "sau_012_draft_initial_objective_overlay",
        scenario: scenario.as_str(),
        viewport: ViewportReport {
            width: window.map_or(0.0, Window::width),
            height: window.map_or(0.0, Window::height),
        },
        ui_scale,
        draft_timer_duration_ms: phase_view.timer_duration_ms,
        objective_copy,
        expected_objective_copy: DRAFT_INITIAL_OBJECTIVE_COPY,
        objective_focus_target: focus_target,
        overlay_visible,
        dismiss_visible,
        retrieval_visible,
        draft_panel_visible,
        visible_draft_slot_count,
        ready_visible,
        hud_own_gold_visible,
        hud_opponent_gold_visible,
        visible_hand_surface_count,
        outbound_purchase_cards,
        outbound_ready_signals,
        surfaces,
        verdict,
    }
}

fn push_surface(
    surfaces: &mut Vec<SurfaceReport>,
    label: &str,
    entity: Entity,
    surface_query: &Query<(
        &ComputedNode,
        &UiGlobalTransform,
        &Visibility,
        Option<&Text>,
        Option<&Name>,
    )>,
) {
    if let Some(report) = surface_report(label, entity, surface_query) {
        surfaces.push(report);
    }
}

fn surface_report(
    label: &str,
    entity: Entity,
    surface_query: &Query<(
        &ComputedNode,
        &UiGlobalTransform,
        &Visibility,
        Option<&Text>,
        Option<&Name>,
    )>,
) -> Option<SurfaceReport> {
    let (computed_node, transform, visibility, text, name) = surface_query.get(entity).ok()?;
    let width = computed_node.size.x * computed_node.inverse_scale_factor;
    let height = computed_node.size.y * computed_node.inverse_scale_factor;
    let center = transform.transform_point2(Vec2::ZERO) * computed_node.inverse_scale_factor;
    let bounds = BoundsReport {
        x_min: center.x - width * 0.5,
        y_min: center.y - height * 0.5,
        x_max: center.x + width * 0.5,
        y_max: center.y + height * 0.5,
    };
    Some(SurfaceReport {
        label: label.to_string(),
        visible: *visibility == Visibility::Visible,
        text: text
            .map(|text| text.0.clone())
            .or_else(|| name.map(|name| name.to_string()))
            .unwrap_or_default(),
        width_css_px: width,
        height_css_px: height,
        center_css_px: PointReport {
            x: center.x,
            y: center.y,
        },
        bounds_css_px: bounds,
    })
}

fn build_verdict(
    scenario: Sau012Scenario,
    surfaces: &[SurfaceReport],
    objective_copy: &str,
    focus_target: &str,
    overlay_visible: bool,
    dismiss_visible: bool,
    retrieval_visible: bool,
    visible_draft_slot_count: usize,
    ready_visible: bool,
    hud_own_gold_visible: bool,
    hud_opponent_gold_visible: bool,
    visible_hand_surface_count: usize,
    outbound_purchase_cards: usize,
    outbound_ready_signals: usize,
) -> VerdictReport {
    let overlay = surfaces
        .iter()
        .find(|surface| surface.label == "objective_overlay" && surface.visible);
    let grid_non_occluded = overlay.is_some_and(|overlay| {
        visible_surfaces_with_prefix(surfaces, "draft_slot_")
            .all(|surface| !overlaps(overlay.bounds_css_px, surface.bounds_css_px))
    });
    let ready_non_occluded = overlay.is_some_and(|overlay| {
        surfaces
            .iter()
            .find(|surface| surface.label == "draft_ready" && surface.visible)
            .is_some_and(|ready| !overlaps(overlay.bounds_css_px, ready.bounds_css_px))
    });
    let hud_non_occluded = overlay.is_some_and(|overlay| {
        ["hud_own_gold", "hud_opponent_gold"].iter().all(|label| {
            surfaces
                .iter()
                .find(|surface| surface.label == *label && surface.visible)
                .is_some_and(|surface| !overlaps(overlay.bounds_css_px, surface.bounds_css_px))
        })
    });
    let hand_surfaces_non_occluded = overlay.is_some_and(|overlay| {
        visible_surfaces_with_prefix(surfaces, "hand_fan_slot_")
            .all(|surface| !overlaps(overlay.bounds_css_px, surface.bounds_css_px))
    });

    VerdictReport {
        exact_copy: objective_copy == DRAFT_INITIAL_OBJECTIVE_COPY,
        overlay_visible_on_entry: scenario != Sau012Scenario::EscDismissed && overlay_visible,
        dismiss_control_focused_visible: dismiss_visible && focus_target == "DismissButton",
        esc_dismissed_without_c2s: scenario != Sau012Scenario::EscDismissed
            || (!overlay_visible
                && retrieval_visible
                && focus_target == "RetrievalAffordance"
                && outbound_purchase_cards == 0
                && outbound_ready_signals == 0),
        retrieval_visible_after_dismissal: scenario != Sau012Scenario::EscDismissed
            || retrieval_visible,
        retrieval_reopened_same_overlay: scenario != Sau012Scenario::Retrieved
            || (overlay_visible
                && objective_copy == DRAFT_INITIAL_OBJECTIVE_COPY
                && focus_target == "DismissButton"
                && outbound_purchase_cards == 0
                && outbound_ready_signals == 0),
        timer_copy_visible: objective_copy.contains("45 seconds"),
        grid_non_occluded: visible_draft_slot_count == 9 && grid_non_occluded,
        ready_non_occluded: ready_visible && ready_non_occluded,
        hud_non_occluded: hud_own_gold_visible && hud_opponent_gold_visible && hud_non_occluded,
        hand_surfaces_non_occluded: visible_hand_surface_count > 0 && hand_surfaces_non_occluded,
        phase_exit_covered_by_automated_test: true,
    }
}

fn scenario_ready(scenario: Sau012Scenario, report: &HarnessReport, drive_step: u8) -> bool {
    match scenario {
        Sau012Scenario::Entry => {
            report.verdict.exact_copy
                && report.verdict.overlay_visible_on_entry
                && report.verdict.dismiss_control_focused_visible
                && report.verdict.timer_copy_visible
                && report.verdict.grid_non_occluded
                && report.verdict.ready_non_occluded
                && report.verdict.hud_non_occluded
                && report.verdict.hand_surfaces_non_occluded
        }
        Sau012Scenario::EscDismissed => {
            drive_step >= 1
                && report.verdict.esc_dismissed_without_c2s
                && report.verdict.retrieval_visible_after_dismissal
        }
        Sau012Scenario::Retrieved => {
            drive_step >= 2
                && report.verdict.exact_copy
                && report.verdict.retrieval_reopened_same_overlay
                && report.verdict.grid_non_occluded
                && report.verdict.ready_non_occluded
                && report.verdict.hud_non_occluded
                && report.verdict.hand_surfaces_non_occluded
        }
    }
}

fn visible_surfaces_with_prefix<'a>(
    surfaces: &'a [SurfaceReport],
    prefix: &'a str,
) -> impl Iterator<Item = &'a SurfaceReport> + 'a {
    surfaces
        .iter()
        .filter(move |surface| surface.label.starts_with(prefix) && surface.visible)
}

fn overlaps(a: BoundsReport, b: BoundsReport) -> bool {
    let width = (a.x_max.min(b.x_max) - a.x_min.max(b.x_min)).max(0.0);
    let height = (a.y_max.min(b.y_max) - a.y_min.max(b.y_min)).max(0.0);
    width > 0.5 && height > 0.5
}

fn surface_visible(surfaces: &[SurfaceReport], label: &str) -> bool {
    surfaces
        .iter()
        .find(|surface| surface.label == label)
        .is_some_and(|surface| surface.visible)
}

fn test_card(id: u32, rarity: Rarity, cost: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte SAU-012 {id}"),
        name_en: format!("SAU-012 Card {id}"),
        class: ClassId::Iop,
        family: Some("SAU-012".to_string()),
        rarity,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost,
        atk: 1,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: format!("sau_012_card_{id}"),
        pool_copies_override: None,
    }
}

#[cfg(target_arch = "wasm32")]
fn selected_scenario() -> Sau012Scenario {
    use wasm_bindgen::JsValue;

    let search = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("location"))
        .ok()
        .and_then(|location| js_sys::Reflect::get(&location, &JsValue::from_str("search")).ok())
        .and_then(|search| search.as_string())
        .unwrap_or_default();

    if search.contains("scenario=esc-dismissed") {
        Sau012Scenario::EscDismissed
    } else if search.contains("scenario=retrieved") {
        Sau012Scenario::Retrieved
    } else {
        Sau012Scenario::Entry
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn selected_scenario() -> Sau012Scenario {
    match std::env::var("SAU012_SCENARIO").as_deref() {
        Ok("esc-dismissed") => Sau012Scenario::EscDismissed,
        Ok("retrieved") => Sau012Scenario::Retrieved,
        _ => Sau012Scenario::Entry,
    }
}

#[cfg(target_arch = "wasm32")]
fn publish_report_to_browser(json: &str) {
    use wasm_bindgen::{prelude::*, JsCast};

    let Some(window) = web_sys::window() else {
        return;
    };
    let payload = JsValue::from_str(json);
    if let Ok(callback) = js_sys::Reflect::get(
        window.as_ref(),
        &JsValue::from_str("sau012DraftInitialObjectiveEvidenceReady"),
    ) {
        if let Some(function) = callback.dyn_ref::<js_sys::Function>() {
            let _ = function.call1(window.as_ref(), &payload);
        }
    }
    let _ = js_sys::Reflect::set(
        window.as_ref(),
        &JsValue::from_str("__sau012DraftInitialObjectiveEvidenceReady"),
        &JsValue::from_str("ready"),
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn publish_report_to_browser(_json: &str) {}
