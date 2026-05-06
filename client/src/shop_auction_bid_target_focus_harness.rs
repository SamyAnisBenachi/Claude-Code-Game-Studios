use std::collections::HashMap;

use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform, UiScale, UiSystems};
use bevy::window::{PresentMode, Window, WindowPlugin};
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::hud::{HudGoldBroadcastMessage, HudPlayerIds};
use client::ui::shop_auction::{
    AuctionBidButton, AuctionBidButtonState, AuctionBidFocusState, AuctionBidKeyboardFocus,
    AuctionBidStatusText, AuctionBidTargetBounds, ShopAuctionAuctionCardReceived,
    ShopAuctionAuctionState, ShopAuctionCardCatalog, ShopAuctionDraftHandView,
    ShopAuctionLocalGoldView, ShopAuctionUiEntities, ShopAuctionUiOutboundMessages,
    ShopAuctionUiPlugin, ShopAuctionUiSystemSet,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{RoundPhase, S2CGoldBroadcast};
use shared::session::PlayerId;

const LOCAL_PLAYER: PlayerId = PlayerId(1);
const OPPONENT_PLAYER: PlayerId = PlayerId(2);
const HARNESS_CARD: CardId = CardId(1);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "SAU-011 Auction Bid Target Focus Harness".to_string(),
            resolution: (1366, 768).into(),
            present_mode: PresentMode::AutoVsync,
            fit_canvas_to_parent: true,
            canvas: Some("#bevy".to_string()),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(UiScale(1.0));
    app.insert_resource(Sau011HarnessState::for_browser());
    app.add_systems(Startup, enter_harness_session_system);
    app.add_systems(
        Update,
        seed_fixture_system
            .before(ShopAuctionUiSystemSet::PhaseTransition)
            .run_if(in_state(ClientState::InSession)),
    );
    app.add_systems(
        Update,
        (
            drive_scenario_input_system
                .after(ShopAuctionUiSystemSet::MessageDrain)
                .before(ShopAuctionUiSystemSet::Input),
            clear_scenario_input_system.after(ShopAuctionUiSystemSet::Input),
            apply_scenario_state_system
                .after(ShopAuctionUiSystemSet::Input)
                .before(ShopAuctionUiSystemSet::StateSync),
        )
            .run_if(in_state(ClientState::InSession)),
    );
    app.add_systems(
        PostUpdate,
        publish_report_system.after(UiSystems::PostLayout),
    );
    app.run();
}

#[derive(Resource, Debug)]
struct Sau011HarnessState {
    scenario: Sau011Scenario,
    seeded: bool,
    tabs_sent: u8,
    clicked_bid: bool,
    clear_tab: bool,
    clear_pointer: Option<Entity>,
    ready_frames: u8,
    published: bool,
}

impl Sau011HarnessState {
    fn for_browser() -> Self {
        Self {
            scenario: selected_scenario(),
            seeded: false,
            tabs_sent: 0,
            clicked_bid: false,
            clear_tab: false,
            clear_pointer: None,
            ready_frames: 0,
            published: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sau011Scenario {
    Affordable,
    FocusPlus1,
    FocusPlus3,
    FocusPlus5,
    Unaffordable,
    Bidding,
    Leading,
}

impl Sau011Scenario {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Affordable => "affordable",
            Self::FocusPlus1 => "focus-plus-1",
            Self::FocusPlus3 => "focus-plus-3",
            Self::FocusPlus5 => "focus-plus-5",
            Self::Unaffordable => "unaffordable",
            Self::Bidding => "bidding",
            Self::Leading => "leading",
        }
    }

    const fn required_tab_presses(self) -> u8 {
        match self {
            Self::FocusPlus1 => 1,
            Self::FocusPlus3 => 2,
            Self::FocusPlus5 => 3,
            Self::Unaffordable => 2,
            _ => 0,
        }
    }

    const fn starting_price(self) -> u32 {
        match self {
            Self::Unaffordable => 0,
            _ => 4,
        }
    }

    const fn gold(self) -> u32 {
        match self {
            Self::Unaffordable => 2,
            _ => 20,
        }
    }
}

fn enter_harness_session_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    commands.spawn((Name::new("SAU-011 Harness Camera"), Camera2d));
    commands.insert_resource(HudPlayerIds {
        local_id: LOCAL_PLAYER,
        opponent_id: OPPONENT_PLAYER,
    });
    next_state.set(ClientState::InSession);
}

fn seed_fixture_system(
    mut state: ResMut<Sau011HarnessState>,
    mut catalog: ResMut<ShopAuctionCardCatalog>,
    mut economy: ResMut<PlayerEconomyView>,
    mut local_gold: ResMut<ShopAuctionLocalGoldView>,
    mut hand_view: ResMut<ShopAuctionDraftHandView>,
    mut current: ResMut<CurrentClientPhase>,
    mut phase_view: ResMut<ClientPhaseView>,
    mut auction_cards: MessageWriter<ShopAuctionAuctionCardReceived>,
    mut gold_broadcasts: MessageWriter<HudGoldBroadcastMessage>,
) {
    if state.seeded {
        return;
    }

    catalog.cards = HashMap::from([(HARNESS_CARD, test_card())]);
    let gold = state.scenario.gold();
    *economy = PlayerEconomyView {
        gold,
        initialized: true,
        ..default()
    };
    *local_gold = ShopAuctionLocalGoldView {
        player_id: Some(LOCAL_PLAYER),
        gold,
        reserved_gold: 0,
        initialized: true,
    };
    *hand_view = ShopAuctionDraftHandView { hand_size: 0 };
    current.phase = RoundPhase::DraftAuction;
    current.round = 3;
    phase_view.phase = RoundPhase::DraftAuction;
    phase_view.round_number = 3;
    phase_view.timer_duration_ms = 20_000;

    auction_cards.write(ShopAuctionAuctionCardReceived {
        card_id: HARNESS_CARD,
        starting_price: state.scenario.starting_price(),
    });
    gold_broadcasts.write(HudGoldBroadcastMessage(S2CGoldBroadcast {
        player_id: LOCAL_PLAYER,
        gold,
        reserved_gold: 0,
    }));

    state.seeded = true;
}

fn drive_scenario_input_system(
    mut state: ResMut<Sau011HarnessState>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut interactions: Query<(&AuctionBidButton, &mut Interaction)>,
    focus_states: Query<&AuctionBidFocusState, With<AuctionBidButton>>,
) {
    let Some(entities) = entities else {
        return;
    };

    if state.tabs_sent < state.scenario.required_tab_presses() {
        if !keyboard_focus_ready(state.scenario, &entities, &focus_states) {
            return;
        }
        keyboard.press(KeyCode::Tab);
        state.tabs_sent += 1;
        state.clear_tab = true;
        return;
    }

    if state.scenario == Sau011Scenario::Bidding && !state.clicked_bid {
        for button_entity in entities.auction_bid_buttons {
            let Ok((button, mut interaction)) = interactions.get_mut(button_entity) else {
                continue;
            };
            if button.increment == 3 {
                *interaction = Interaction::Pressed;
                state.clicked_bid = true;
                state.clear_pointer = Some(button_entity);
                break;
            }
        }
    }
}

fn clear_scenario_input_system(
    mut state: ResMut<Sau011HarnessState>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut interactions: Query<&mut Interaction>,
) {
    if state.clear_tab {
        keyboard.release(KeyCode::Tab);
        keyboard.clear();
        state.clear_tab = false;
    }

    if let Some(button) = state.clear_pointer.take() {
        if let Ok(mut interaction) = interactions.get_mut(button) {
            *interaction = Interaction::None;
        }
    }
}

fn apply_scenario_state_system(
    state: Res<Sau011HarnessState>,
    local_gold: Res<ShopAuctionLocalGoldView>,
    mut auction_state: ResMut<ShopAuctionAuctionState>,
) {
    if state.scenario == Sau011Scenario::Leading {
        auction_state.current_leader = local_gold.player_id;
    }
}

fn publish_report_system(
    mut state: ResMut<Sau011HarnessState>,
    windows: Query<&Window>,
    ui_scale: Res<UiScale>,
    entities: Option<Res<ShopAuctionUiEntities>>,
    outbound: Option<Res<ShopAuctionUiOutboundMessages>>,
    keyboard_focus: Option<Res<AuctionBidKeyboardFocus>>,
    bid_status_texts: Query<&Text, With<AuctionBidStatusText>>,
    buttons: Query<(
        &AuctionBidButton,
        &AuctionBidButtonState,
        &AuctionBidTargetBounds,
        &AuctionBidFocusState,
        &ComputedNode,
        &UiGlobalTransform,
        &Visibility,
        &Text,
        &Node,
    )>,
) {
    if state.published || !state.seeded {
        return;
    }

    let Some(entities) = entities else {
        return;
    };

    let report = collect_report(
        state.scenario,
        &windows,
        ui_scale.0,
        &entities,
        outbound.as_deref(),
        keyboard_focus.as_deref(),
        &bid_status_texts,
        &buttons,
    );
    if !scenario_ready(state.scenario, &report, state.tabs_sent, state.clicked_bid) {
        state.ready_frames = 0;
        return;
    }

    state.ready_frames = state.ready_frames.saturating_add(1);
    if state.ready_frames < 2 {
        return;
    }

    let json = report.to_json();
    info!("SAU-011 auction evidence {json}");
    publish_report_to_browser(&json);
    state.published = true;
}

#[derive(Debug)]
struct HarnessReport {
    scenario: Sau011Scenario,
    viewport_width: f32,
    viewport_height: f32,
    ui_scale: f32,
    buttons: Vec<ButtonReport>,
    focused_increment: Option<u32>,
    bid_status_text: String,
    outbound_place_bids: usize,
}

impl HarnessReport {
    fn to_json(&self) -> String {
        let button_json = self
            .buttons
            .iter()
            .map(ButtonReport::to_json)
            .collect::<Vec<_>>()
            .join(",");
        let focused_increment = self
            .focused_increment
            .map(|increment| increment.to_string())
            .unwrap_or_else(|| "null".to_string());

        format!(
            concat!(
                "{{",
                "\"ready_for_capture\":true,",
                "\"fixture\":\"sau_011_auction_bid_target_focus\",",
                "\"scenario\":\"{}\",",
                "\"viewport\":{{\"width\":{},\"height\":{}}},",
                "\"ui_scale\":{},",
                "\"buttons\":[{}],",
                "\"target_bounds\":[{}],",
                "\"focus_bounds\":[{}],",
                "\"focused_increment\":{},",
                "\"bid_status_text\":\"{}\",",
                "\"outbound_place_bids\":{},",
                "\"verdict\":{{",
                "\"target_bounds_44px\":{},",
                "\"focus_bounds_44px\":{},",
                "\"affordable_labels\":{},",
                "\"unaffordable_keyboard_skip\":{},",
                "\"bidding_feedback\":{},",
                "\"leading_replacement_no_focusable_bid\":{}",
                "}}",
                "}}"
            ),
            self.scenario.as_str(),
            format_number(self.viewport_width),
            format_number(self.viewport_height),
            format_number(self.ui_scale),
            button_json,
            self.bounds_json(),
            self.focus_bounds_json(),
            focused_increment,
            json_escape(&self.bid_status_text),
            self.outbound_place_bids,
            json_bool(self.target_bounds_pass()),
            json_bool(self.focus_bounds_pass()),
            json_bool(self.affordable_labels_pass()),
            json_bool(self.unaffordable_keyboard_skip_pass()),
            json_bool(self.bidding_feedback_pass()),
            json_bool(self.leading_replacement_pass())
        )
    }

    fn bounds_json(&self) -> String {
        self.buttons
            .iter()
            .map(|button| {
                format!(
                    "{{\"increment\":{},\"width_css_px\":{},\"height_css_px\":{},\"component_width_px\":{},\"component_height_px\":{},\"meets_44px\":{}}}",
                    button.increment,
                    format_number(button.width_css_px),
                    format_number(button.height_css_px),
                    format_number(button.component_width_px),
                    format_number(button.component_height_px),
                    json_bool(button.meets_44px())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    fn focus_bounds_json(&self) -> String {
        self.buttons
            .iter()
            .map(|button| {
                format!(
                    "{{\"increment\":{},\"focusable\":{},\"focused\":{},\"focus_ring_visible\":{},\"focus_ring_width_px\":{},\"width_css_px\":{},\"height_css_px\":{},\"meets_44px\":{}}}",
                    button.increment,
                    json_bool(button.focusable),
                    json_bool(button.focused),
                    json_bool(button.focus_ring_visible),
                    format_number(button.focus_ring_width_px),
                    format_number(button.width_css_px),
                    format_number(button.height_css_px),
                    json_bool(button.meets_44px())
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    fn target_bounds_pass(&self) -> bool {
        self.buttons.iter().all(ButtonReport::meets_44px)
    }

    fn focus_bounds_pass(&self) -> bool {
        self.buttons
            .iter()
            .filter(|button| button.focused)
            .all(ButtonReport::meets_44px)
    }

    fn affordable_labels_pass(&self) -> bool {
        self.buttons
            .iter()
            .map(|button| button.text.as_str())
            .eq(["5g\n(+1)", "7g\n(+3)", "9g\n(+5)"])
    }

    fn unaffordable_keyboard_skip_pass(&self) -> bool {
        self.scenario == Sau011Scenario::Unaffordable
            && self.focused_increment == Some(1)
            && self.buttons.iter().any(|button| {
                button.increment == 1
                    && button.focused
                    && button.focusable
                    && button.state == AuctionBidButtonState::Enabled
            })
            && self.buttons.iter().all(|button| {
                button.increment == 1
                    || (!button.focusable && button.state == AuctionBidButtonState::Unaffordable)
            })
    }

    fn bidding_feedback_pass(&self) -> bool {
        self.scenario == Sau011Scenario::Bidding
            && self.outbound_place_bids == 1
            && self.buttons.iter().any(|button| {
                button.increment == 3
                    && button.state == AuctionBidButtonState::InFlight
                    && button.text == "BIDDING..."
            })
            && self.buttons.iter().all(|button| {
                button.increment == 3 || button.state == AuctionBidButtonState::GenericDisabled
            })
    }

    fn leading_replacement_pass(&self) -> bool {
        self.scenario == Sau011Scenario::Leading
            && self.bid_status_text == "YOU ARE LEADING"
            && self.focused_increment.is_none()
            && self
                .buttons
                .iter()
                .all(|button| !button.visible && !button.focusable)
    }

    fn focused_button_has_ring(&self, increment: u32) -> bool {
        self.buttons.iter().any(|button| {
            button.increment == increment
                && button.focused
                && button.focusable
                && button.focus_ring_visible
                && button.focus_ring_width_px >= 2.0
                && button.border_left_px >= 2.0
                && button.border_right_px >= 2.0
        })
    }
}

#[derive(Debug)]
struct ButtonReport {
    increment: u32,
    state: AuctionBidButtonState,
    visible: bool,
    text: String,
    focusable: bool,
    focused: bool,
    focus_ring_visible: bool,
    focus_ring_width_px: f32,
    order: u8,
    component_width_px: f32,
    component_height_px: f32,
    width_css_px: f32,
    height_css_px: f32,
    center_x_css_px: f32,
    center_y_css_px: f32,
    border_left_px: f32,
    border_right_px: f32,
    border_top_px: f32,
    border_bottom_px: f32,
}

impl ButtonReport {
    fn to_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"increment\":{},",
                "\"state\":\"{:?}\",",
                "\"visible\":{},",
                "\"text\":\"{}\",",
                "\"focusable\":{},",
                "\"focused\":{},",
                "\"focus_ring_visible\":{},",
                "\"focus_ring_width_px\":{},",
                "\"order\":{},",
                "\"width_css_px\":{},",
                "\"height_css_px\":{},",
                "\"center_css_px\":{{\"x\":{},\"y\":{}}},",
                "\"border_px\":{{\"left\":{},\"right\":{},\"top\":{},\"bottom\":{}}}",
                "}}"
            ),
            self.increment,
            self.state,
            json_bool(self.visible),
            json_escape(&self.text),
            json_bool(self.focusable),
            json_bool(self.focused),
            json_bool(self.focus_ring_visible),
            format_number(self.focus_ring_width_px),
            self.order,
            format_number(self.width_css_px),
            format_number(self.height_css_px),
            format_number(self.center_x_css_px),
            format_number(self.center_y_css_px),
            format_number(self.border_left_px),
            format_number(self.border_right_px),
            format_number(self.border_top_px),
            format_number(self.border_bottom_px)
        )
    }

    fn meets_44px(&self) -> bool {
        self.width_css_px >= 44.0 && self.height_css_px >= 44.0
    }
}

fn keyboard_focus_ready(
    scenario: Sau011Scenario,
    entities: &ShopAuctionUiEntities,
    focus_states: &Query<&AuctionBidFocusState, With<AuctionBidButton>>,
) -> bool {
    let focusable_count = entities
        .auction_bid_buttons
        .iter()
        .filter_map(|button| focus_states.get(*button).ok())
        .filter(|focus| focus.focusable)
        .count();

    match scenario {
        Sau011Scenario::Unaffordable => focusable_count == 1,
        Sau011Scenario::FocusPlus1 | Sau011Scenario::FocusPlus3 | Sau011Scenario::FocusPlus5 => {
            focusable_count == 3
        }
        _ => true,
    }
}

fn collect_report(
    scenario: Sau011Scenario,
    windows: &Query<&Window>,
    ui_scale: f32,
    entities: &ShopAuctionUiEntities,
    outbound: Option<&ShopAuctionUiOutboundMessages>,
    keyboard_focus: Option<&AuctionBidKeyboardFocus>,
    bid_status_texts: &Query<&Text, With<AuctionBidStatusText>>,
    buttons: &Query<(
        &AuctionBidButton,
        &AuctionBidButtonState,
        &AuctionBidTargetBounds,
        &AuctionBidFocusState,
        &ComputedNode,
        &UiGlobalTransform,
        &Visibility,
        &Text,
        &Node,
    )>,
) -> HarnessReport {
    let window = windows.iter().next();
    let mut reports = entities
        .auction_bid_buttons
        .iter()
        .filter_map(|button_entity| {
            let (
                button,
                state,
                target_bounds,
                focus_state,
                computed_node,
                transform,
                visibility,
                text,
                node,
            ) = buttons.get(*button_entity).ok()?;
            let css_width = computed_node.size.x * computed_node.inverse_scale_factor;
            let css_height = computed_node.size.y * computed_node.inverse_scale_factor;
            let center =
                transform.transform_point2(Vec2::ZERO) * computed_node.inverse_scale_factor;
            Some(ButtonReport {
                increment: button.increment,
                state: *state,
                visible: *visibility == Visibility::Visible,
                text: text.0.clone(),
                focusable: focus_state.focusable,
                focused: focus_state.focused,
                focus_ring_visible: focus_state.focus_ring_visible,
                focus_ring_width_px: focus_state.focus_ring_width_px,
                order: focus_state.order,
                component_width_px: target_bounds.width_px,
                component_height_px: target_bounds.height_px,
                width_css_px: css_width,
                height_css_px: css_height,
                center_x_css_px: center.x,
                center_y_css_px: center.y,
                border_left_px: px_value(node.border.left),
                border_right_px: px_value(node.border.right),
                border_top_px: px_value(node.border.top),
                border_bottom_px: px_value(node.border.bottom),
            })
        })
        .collect::<Vec<_>>();
    reports.sort_by_key(|button| button.order);

    let focused_increment = reports
        .iter()
        .find(|button| button.focused)
        .map(|button| button.increment)
        .or_else(|| {
            keyboard_focus.and_then(|focus| {
                focus.focused_button.and_then(|focused| {
                    entities
                        .auction_bid_buttons
                        .iter()
                        .find_map(|button_entity| {
                            if *button_entity == focused {
                                buttons
                                    .get(*button_entity)
                                    .ok()
                                    .map(|(button, ..)| button.increment)
                            } else {
                                None
                            }
                        })
                })
            })
        });
    let bid_status_text = bid_status_texts
        .get(entities.auction_bid_status_text)
        .map(|text| text.0.clone())
        .unwrap_or_default();

    HarnessReport {
        scenario,
        viewport_width: window.map_or(0.0, Window::width),
        viewport_height: window.map_or(0.0, Window::height),
        ui_scale,
        buttons: reports,
        focused_increment,
        bid_status_text,
        outbound_place_bids: outbound.map_or(0, |outbound| outbound.place_bids.len()),
    }
}

fn scenario_ready(
    scenario: Sau011Scenario,
    report: &HarnessReport,
    tabs_sent: u8,
    clicked_bid: bool,
) -> bool {
    if report.buttons.len() != 3 || !report.target_bounds_pass() {
        return false;
    }

    match scenario {
        Sau011Scenario::Affordable => {
            report.affordable_labels_pass()
                && report
                    .buttons
                    .iter()
                    .all(|button| button.state == AuctionBidButtonState::Enabled && button.visible)
        }
        Sau011Scenario::FocusPlus1 => {
            tabs_sent >= 1
                && report.focused_increment == Some(1)
                && report.focus_bounds_pass()
                && report.focused_button_has_ring(1)
        }
        Sau011Scenario::FocusPlus3 => {
            tabs_sent >= 2
                && report.focused_increment == Some(3)
                && report.focus_bounds_pass()
                && report.focused_button_has_ring(3)
        }
        Sau011Scenario::FocusPlus5 => {
            tabs_sent >= 3
                && report.focused_increment == Some(5)
                && report.focus_bounds_pass()
                && report.focused_button_has_ring(5)
        }
        Sau011Scenario::Unaffordable => tabs_sent >= 2 && report.unaffordable_keyboard_skip_pass(),
        Sau011Scenario::Bidding => clicked_bid && report.bidding_feedback_pass(),
        Sau011Scenario::Leading => report.leading_replacement_pass(),
    }
}

fn px_value(value: Val) -> f32 {
    match value {
        Val::Px(px) => px,
        _ => 0.0,
    }
}

fn json_bool(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other => escaped.push(other),
        }
    }
    escaped
}

fn format_number(value: f32) -> String {
    format!("{value:.1}")
}

fn test_card() -> CardData {
    CardData {
        id: HARNESS_CARD,
        name_fr: "Carte SAU-011".to_string(),
        name_en: "SAU-011 Test Card".to_string(),
        class: ClassId::Iop,
        family: Some("Harness".to_string()),
        rarity: Rarity::Rare,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: 4,
        atk: 1,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: "sau_011_test_card".to_string(),
        pool_copies_override: None,
    }
}

#[cfg(target_arch = "wasm32")]
fn selected_scenario() -> Sau011Scenario {
    use wasm_bindgen::JsValue;

    let search = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("location"))
        .ok()
        .and_then(|location| js_sys::Reflect::get(&location, &JsValue::from_str("search")).ok())
        .and_then(|search| search.as_string())
        .unwrap_or_default();

    if search.contains("scenario=focus-plus-1") {
        Sau011Scenario::FocusPlus1
    } else if search.contains("scenario=focus-plus-3") {
        Sau011Scenario::FocusPlus3
    } else if search.contains("scenario=focus-plus-5") {
        Sau011Scenario::FocusPlus5
    } else if search.contains("scenario=unaffordable") {
        Sau011Scenario::Unaffordable
    } else if search.contains("scenario=bidding") {
        Sau011Scenario::Bidding
    } else if search.contains("scenario=leading") {
        Sau011Scenario::Leading
    } else {
        Sau011Scenario::Affordable
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn selected_scenario() -> Sau011Scenario {
    match std::env::var("SAU011_SCENARIO").as_deref() {
        Ok("focus-plus-1") => Sau011Scenario::FocusPlus1,
        Ok("focus-plus-3") => Sau011Scenario::FocusPlus3,
        Ok("focus-plus-5") => Sau011Scenario::FocusPlus5,
        Ok("unaffordable") => Sau011Scenario::Unaffordable,
        Ok("bidding") => Sau011Scenario::Bidding,
        Ok("leading") => Sau011Scenario::Leading,
        _ => Sau011Scenario::Affordable,
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
        &JsValue::from_str("sau011AuctionEvidenceReady"),
    ) {
        if let Some(function) = callback.dyn_ref::<js_sys::Function>() {
            let _ = function.call1(window.as_ref(), &payload);
        }
    }
    let _ = js_sys::Reflect::set(
        window.as_ref(),
        &JsValue::from_str("__sau011AuctionEvidenceReady"),
        &JsValue::from_str("ready"),
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn publish_report_to_browser(_json: &str) {}
