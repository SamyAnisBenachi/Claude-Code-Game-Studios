use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    DraftInitialObjectiveFocusTarget, DraftInitialObjectivePanelClickTarget, DraftInitialSlotCard,
    ShopAuctionCardCatalog, ShopAuctionDraftObjectiveDismissClicked,
    ShopAuctionDraftObjectiveEnterPressed, ShopAuctionDraftObjectiveEscPressed,
    ShopAuctionDraftObjectivePanelClicked, ShopAuctionDraftObjectiveRetrievalClicked,
    ShopAuctionDraftOfferingReceived, ShopAuctionDraftReadyButtonClicked,
    ShopAuctionDraftSlotClicked, ShopAuctionUiEntities, ShopAuctionUiOutboundMessages,
    ShopAuctionUiPlugin, DRAFT_INITIAL_OBJECTIVE_COPY, SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn sau_012_overlay_waits_for_phase_and_offering_before_showing_exact_copy() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(5, true);
    send_offering(&mut app, card_ids(1, 9));

    assert_eq!(draft_panel_visibility(&app), Some(&Visibility::Hidden));
    assert_eq!(
        objective_overlay_visibility(&app),
        Some(&Visibility::Hidden),
        "offering alone must not activate the objective overlay"
    );

    set_phase(&mut app, RoundPhase::DraftInitial);
    run_update(&mut app);

    assert_eq!(draft_panel_visibility(&app), Some(&Visibility::Visible));
    assert_eq!(
        objective_overlay_visibility(&app),
        Some(&Visibility::Visible)
    );
    assert_eq!(
        objective_copy_text(&app),
        Some(DRAFT_INITIAL_OBJECTIVE_COPY)
    );
    assert_eq!(
        objective_retrieval_visibility(&app),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        app.world()
            .resource::<client::ui::shop_auction::ShopAuctionDraftInitialState>()
            .objective_focus_target,
        DraftInitialObjectiveFocusTarget::DismissButton
    );
}

#[test]
fn sau_012_overlay_dismiss_button_hides_overlay_without_hiding_draft_controls() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app(5, true);

    click_objective_dismiss(&mut app);

    assert_eq!(
        objective_overlay_visibility(&app),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        objective_retrieval_visibility(&app),
        Some(&Visibility::Visible)
    );
    assert_eq!(visible_slot_count(&app), 9);
    assert_eq!(draft_ready_visibility(&app), Some(&Visibility::Visible));
    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .purchase_cards
            .len(),
        0
    );
    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .ready_signals
            .len(),
        0
    );
    assert_eq!(
        app.world()
            .resource::<client::ui::shop_auction::ShopAuctionDraftInitialState>()
            .objective_focus_target,
        DraftInitialObjectiveFocusTarget::RetrievalAffordance
    );
}

#[test]
fn sau_012_escape_dismisses_and_enter_retrieves_with_deterministic_focus() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app(5, true);

    app.world_mut()
        .write_message(ShopAuctionDraftObjectiveEscPressed);
    run_update(&mut app);

    assert_eq!(
        objective_overlay_visibility(&app),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        objective_retrieval_visibility(&app),
        Some(&Visibility::Visible)
    );
    assert_eq!(
        app.world()
            .resource::<client::ui::shop_auction::ShopAuctionDraftInitialState>()
            .objective_focus_target,
        DraftInitialObjectiveFocusTarget::RetrievalAffordance
    );

    app.world_mut()
        .write_message(ShopAuctionDraftObjectiveEnterPressed);
    run_update(&mut app);

    assert_eq!(
        objective_overlay_visibility(&app),
        Some(&Visibility::Visible)
    );
    assert_eq!(
        objective_copy_text(&app),
        Some(DRAFT_INITIAL_OBJECTIVE_COPY)
    );
    assert_eq!(
        app.world()
            .resource::<client::ui::shop_auction::ShopAuctionDraftInitialState>()
            .objective_focus_target,
        DraftInitialObjectiveFocusTarget::DismissButton
    );
}

#[test]
fn sau_012_outside_panel_click_dismisses_without_c2s_messages() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app(5, true);

    app.world_mut()
        .write_message(ShopAuctionDraftObjectivePanelClicked {
            target: DraftInitialObjectivePanelClickTarget::NonActionablePanel,
        });
    run_update(&mut app);

    assert_eq!(
        objective_overlay_visibility(&app),
        Some(&Visibility::Hidden)
    );
    let outbound = app.world().resource::<ShopAuctionUiOutboundMessages>();
    assert!(outbound.purchase_cards.is_empty());
    assert!(outbound.ready_signals.is_empty());
}

#[test]
fn sau_012_guarded_controls_do_not_count_as_outside_dismiss() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app(5, true);

    app.world_mut()
        .write_message(ShopAuctionDraftObjectivePanelClicked {
            target: DraftInitialObjectivePanelClickTarget::Timer,
        });
    run_update(&mut app);
    assert_eq!(
        objective_overlay_visibility(&app),
        Some(&Visibility::Visible)
    );

    let slot = draft_slot(&app, 0);
    app.world_mut()
        .write_message(ShopAuctionDraftObjectivePanelClicked {
            target: DraftInitialObjectivePanelClickTarget::CardSlot(slot),
        });
    click_slot(&mut app, slot);
    assert_eq!(
        objective_overlay_visibility(&app),
        Some(&Visibility::Visible)
    );
    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .purchase_cards
            .len(),
        1
    );

    let mut app = active_draft_app(5, true);
    let ready = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .draft_initial_ready_button;
    app.world_mut()
        .write_message(ShopAuctionDraftObjectivePanelClicked {
            target: DraftInitialObjectivePanelClickTarget::ReadyButton,
        });
    click_ready(&mut app, ready);
    assert_eq!(
        objective_overlay_visibility(&app),
        Some(&Visibility::Visible)
    );
    assert_eq!(
        app.world()
            .resource::<ShopAuctionUiOutboundMessages>()
            .ready_signals
            .len(),
        1
    );
}

#[test]
fn sau_012_retrieval_reopens_same_overlay_and_never_emits_c2s() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app(5, true);
    click_objective_dismiss(&mut app);

    click_objective_retrieval(&mut app);

    assert_eq!(
        objective_overlay_visibility(&app),
        Some(&Visibility::Visible)
    );
    assert_eq!(
        objective_copy_text(&app),
        Some(DRAFT_INITIAL_OBJECTIVE_COPY)
    );
    assert_eq!(
        objective_retrieval_visibility(&app),
        Some(&Visibility::Hidden)
    );
    let outbound = app.world().resource::<ShopAuctionUiOutboundMessages>();
    assert!(outbound.purchase_cards.is_empty());
    assert!(outbound.ready_signals.is_empty());
}

#[test]
fn sau_012_overlay_and_retrieval_hide_on_placement_phase_exit() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app(5, true);
    click_objective_dismiss(&mut app);
    assert_eq!(
        objective_retrieval_visibility(&app),
        Some(&Visibility::Visible)
    );

    set_phase(&mut app, RoundPhase::Placement);
    run_update(&mut app);

    assert_eq!(draft_panel_visibility(&app), Some(&Visibility::Hidden));
    assert_eq!(
        objective_overlay_visibility(&app),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        objective_retrieval_visibility(&app),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        app.world()
            .resource::<client::ui::shop_auction::ShopAuctionDraftInitialState>()
            .objective_focus_target,
        DraftInitialObjectiveFocusTarget::None
    );
}

#[test]
fn sau_012_dismiss_and_retrieval_controls_are_button_interaction_targets() {
    test_helpers::init_test_tracing();
    let app = active_draft_app(5, true);
    let entities = *app.world().resource::<ShopAuctionUiEntities>();

    assert!(app
        .world()
        .get::<Button>(entities.draft_initial_objective_dismiss_button)
        .is_some());
    assert!(app
        .world()
        .get::<Interaction>(entities.draft_initial_objective_dismiss_button)
        .is_some());
    assert!(app
        .world()
        .get::<Button>(entities.draft_initial_objective_retrieval_button)
        .is_some());
    assert!(app
        .world()
        .get::<Interaction>(entities.draft_initial_objective_retrieval_button)
        .is_some());
}

fn active_draft_app(gold: u32, economy_initialized: bool) -> App {
    let mut app = app_in_session(gold, economy_initialized);
    set_phase(&mut app, RoundPhase::DraftInitial);
    send_offering(&mut app, card_ids(1, 9));
    app
}

fn app_in_session(gold: u32, economy_initialized: bool) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    insert_catalog(&mut app);
    app.insert_resource(PlayerEconomyView {
        gold,
        initialized: economy_initialized,
        ..default()
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    app
}

fn insert_catalog(app: &mut App) {
    app.insert_resource(ShopAuctionCardCatalog {
        cards: (1..=9)
            .map(|id| {
                let card = test_card(id, Rarity::Common, (id - 1) % 5 + 1);
                (card.id, card)
            })
            .collect::<HashMap<_, _>>(),
    });
}

fn test_card(id: u32, rarity: Rarity, cost: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
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
        art_id: format!("test_{id}"),
        pool_copies_override: None,
    }
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn send_offering(app: &mut App, card_ids: Vec<CardId>) {
    app.world_mut()
        .write_message(ShopAuctionDraftOfferingReceived { card_ids });
    run_update(app);
}

fn click_slot(app: &mut App, slot: Entity) {
    app.world_mut()
        .write_message(ShopAuctionDraftSlotClicked { slot });
    run_update(app);
}

fn click_ready(app: &mut App, button: Entity) {
    app.world_mut()
        .write_message(ShopAuctionDraftReadyButtonClicked { button });
    run_update(app);
}

fn click_objective_dismiss(app: &mut App) {
    let button = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .draft_initial_objective_dismiss_button;
    app.world_mut()
        .write_message(ShopAuctionDraftObjectiveDismissClicked { button });
    run_update(app);
}

fn click_objective_retrieval(app: &mut App) {
    let button = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .draft_initial_objective_retrieval_button;
    app.world_mut()
        .write_message(ShopAuctionDraftObjectiveRetrievalClicked { button });
    run_update(app);
}

fn run_update(app: &mut App) {
    app.update();
}

fn card_ids(start: u32, count: u32) -> Vec<CardId> {
    (start..start + count).map(CardId).collect()
}

fn draft_panel_visibility(app: &App) -> Option<&Visibility> {
    app.world().get::<Visibility>(
        app.world()
            .resource::<ShopAuctionUiEntities>()
            .draft_offering_panel,
    )
}

fn objective_overlay_visibility(app: &App) -> Option<&Visibility> {
    app.world().get::<Visibility>(
        app.world()
            .resource::<ShopAuctionUiEntities>()
            .draft_initial_objective_overlay,
    )
}

fn objective_retrieval_visibility(app: &App) -> Option<&Visibility> {
    app.world().get::<Visibility>(
        app.world()
            .resource::<ShopAuctionUiEntities>()
            .draft_initial_objective_retrieval_button,
    )
}

fn draft_ready_visibility(app: &App) -> Option<&Visibility> {
    app.world().get::<Visibility>(
        app.world()
            .resource::<ShopAuctionUiEntities>()
            .draft_initial_ready_button,
    )
}

fn objective_copy_text(app: &App) -> Option<&str> {
    app.world()
        .get::<Text>(
            app.world()
                .resource::<ShopAuctionUiEntities>()
                .draft_initial_objective_copy,
        )
        .map(|text| text.0.as_str())
}

fn draft_slots(app: &App) -> [Entity; SHOP_AUCTION_UI_DRAFT_INITIAL_SLOT_COUNT] {
    app.world()
        .resource::<ShopAuctionUiEntities>()
        .draft_initial_slots
}

fn draft_slot(app: &App, index: usize) -> Entity {
    draft_slots(app)[index]
}

fn visible_slot_count(app: &App) -> usize {
    draft_slots(app)
        .iter()
        .filter(|slot| app.world().get::<Visibility>(**slot) == Some(&Visibility::Visible))
        .count()
}

fn _slot_card(app: &App, slot: Entity) -> CardId {
    app.world()
        .get::<DraftInitialSlotCard>(slot)
        .expect("slot should have a card")
        .0
}
