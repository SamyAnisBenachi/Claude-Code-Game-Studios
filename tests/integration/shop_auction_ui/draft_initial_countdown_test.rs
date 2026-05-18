// PROMPT 1230 — S18-DRAFT-INITIAL-NUMERIC-COUNTDOWN-001
//
// Locks the in-modal numeric countdown for the keep-9 DraftInitial picker:
// the modal-local `DraftInitialCountdownLabel` must paint a live remaining-
// seconds readout while DraftInitial is active *and* the canonical
// `PhaseTimerState` resource is counting down, must update text as the
// elapsed millis advance, and must hide outside the DraftInitial phase so
// the label cannot leak into the shop / auction / placement / resolution
// surfaces that share the same `ShopAuctionUiPlugin`.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hud::PhaseTimerState;
use client::ui::shop_auction::{
    DraftInitialCountdownLabel, ShopAuctionCardCatalog, ShopAuctionDraftOfferingReceived,
    ShopAuctionUiEntities, ShopAuctionUiPlugin,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn sau_018_countdown_is_hidden_before_draft_initial_activates() {
    test_helpers::init_test_tracing();
    let app = app_in_session();

    // Pre-DraftInitial (and pre-offering) state: the label must be Hidden
    // and its text must be the empty string. A stale "45s" before phase
    // entry would mislead the player.
    let label = countdown_label_entity(&app);
    assert_eq!(
        app.world().get::<Visibility>(label),
        Some(&Visibility::Hidden),
        "countdown label must spawn Hidden",
    );
    assert_eq!(label_text(&app, label), "");
}

#[test]
fn sau_018_countdown_visible_with_seconds_text_when_draft_initial_active() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app();
    let label = countdown_label_entity(&app);

    set_phase_timer(&mut app, 45_000, 0, true);
    run_update(&mut app);

    assert_eq!(
        app.world().get::<Visibility>(label),
        Some(&Visibility::Visible),
        "countdown label must become Visible once DraftInitial is active \
         with an active phase timer",
    );
    assert_eq!(
        label_text(&app, label),
        "45s",
        "fresh 45_000 ms budget must render as the literal '45s' the player \
         expects from the objective copy",
    );
}

#[test]
fn sau_018_countdown_text_updates_as_elapsed_advances() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app();
    let label = countdown_label_entity(&app);

    set_phase_timer(&mut app, 45_000, 0, true);
    run_update(&mut app);
    assert_eq!(label_text(&app, label), "45s");

    // 15s elapsed → 30s remaining
    set_phase_timer(&mut app, 45_000, 15_000, true);
    run_update(&mut app);
    assert_eq!(label_text(&app, label), "30s");

    // Mid-second elapsed → round-up so the player never sees "0s" while
    // time is still on the clock (mirrors the HUD countdown convention).
    set_phase_timer(&mut app, 45_000, 44_500, true);
    run_update(&mut app);
    assert_eq!(label_text(&app, label), "1s");

    // Fully elapsed → "0s" before the phase actually changes.
    set_phase_timer(&mut app, 45_000, 45_000, true);
    run_update(&mut app);
    assert_eq!(label_text(&app, label), "0s");
}

#[test]
fn sau_018_countdown_hidden_when_phase_leaves_draft_initial() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app();
    let label = countdown_label_entity(&app);
    set_phase_timer(&mut app, 45_000, 10_000, true);
    run_update(&mut app);
    assert_eq!(
        app.world().get::<Visibility>(label),
        Some(&Visibility::Visible),
    );

    // Leaving DraftInitial — even if the phase timer is still active for
    // the next phase — must Hide the label and clear its text so it
    // cannot leak into shop / auction / placement / resolution.
    for next_phase in [
        RoundPhase::DraftShop,
        RoundPhase::DraftAuction,
        RoundPhase::Placement,
        RoundPhase::Resolution,
        RoundPhase::GameOver,
    ] {
        set_phase(&mut app, next_phase);
        // Keep the canonical timer "active" deliberately so we prove the
        // gate is the DraftInitial mode, not the timer state alone.
        set_phase_timer(&mut app, 30_000, 5_000, true);
        run_update(&mut app);

        assert_eq!(
            app.world().get::<Visibility>(label),
            Some(&Visibility::Hidden),
            "countdown label must hide outside DraftInitial (was {next_phase:?})",
        );
        assert_eq!(
            label_text(&app, label),
            "",
            "countdown label text must clear outside DraftInitial (was {next_phase:?})",
        );
    }
}

#[test]
fn sau_018_countdown_hidden_when_phase_timer_inactive() {
    test_helpers::init_test_tracing();
    let mut app = active_draft_app();
    let label = countdown_label_entity(&app);

    // Active DraftInitial but the canonical timer never started (e.g.
    // `S2CPhaseChanged` lost or zero duration): label stays hidden so the
    // player cannot see a meaningless "0s" stuck on the modal.
    set_phase_timer(&mut app, 0, 0, false);
    run_update(&mut app);
    assert_eq!(
        app.world().get::<Visibility>(label),
        Some(&Visibility::Hidden),
    );
    assert_eq!(label_text(&app, label), "");
}

fn active_draft_app() -> App {
    let mut app = app_in_session();
    set_phase(&mut app, RoundPhase::DraftInitial);
    send_offering(&mut app, card_ids(1, 9));
    app
}

fn app_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    insert_catalog(&mut app);
    app.insert_resource(PlayerEconomyView {
        gold: 5,
        initialized: true,
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

fn set_phase_timer(app: &mut App, duration_ms: u32, elapsed_ms: u32, active: bool) {
    let mut timer = app.world_mut().resource_mut::<PhaseTimerState>();
    timer.duration_ms = duration_ms;
    timer.elapsed_ms = elapsed_ms;
    timer.active = active;
}

fn send_offering(app: &mut App, card_ids: Vec<CardId>) {
    app.world_mut()
        .write_message(ShopAuctionDraftOfferingReceived { card_ids });
    run_update(app);
}

fn run_update(app: &mut App) {
    app.update();
}

fn card_ids(start: u32, count: u32) -> Vec<CardId> {
    (start..start + count).map(CardId).collect()
}

fn countdown_label_entity(app: &App) -> Entity {
    let entities = app.world().resource::<ShopAuctionUiEntities>();
    let label = entities.draft_initial_countdown_label;
    assert!(
        app.world()
            .get::<DraftInitialCountdownLabel>(label)
            .is_some(),
        "draft_initial_countdown_label entity must carry the marker component",
    );
    label
}

fn label_text(app: &App, label: Entity) -> String {
    app.world()
        .get::<Text>(label)
        .map(|text| text.0.clone())
        .unwrap_or_default()
}
