//! PROMPT 1228 / HUNT-1201-12 — HUD mana label paints a projected
//! `current_mana - card.cost` / `reserve_mana - card.cost` readout while
//! an affordable placement drag is in flight, and reverts to the
//! authoritative numbers on drag end / cancel.
//!
//! The system under test is [`client::ui::hud::sync_mana_text_system`].
//! The preview path is display-only: it never mutates
//! [`client::presentation::PlayerEconomyView`], so the authoritative
//! mana display returns automatically once the drag clears (no explicit
//! reset event needed). ADR-002 binding preserved — server authority on
//! mana is untouched by this affordance.

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{ActivePlacementDrag, HandCardCatalog, PlacementTargetKind};
use client::ui::hud::{HudEntities, HudPlugin};
use shared::card::{
    CardCatalog, CardData, CardId, CardType, ClassId, Keyword, Rarity, UnitType,
};
use shared::protocol::RoundPhase;
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const TEST_CARD_ID: u32 = 4_242;
const TEST_CARD_COST: u32 = 4;

/// AC1 — While an affordable placement drag is active, the HUD mana label
/// reads the projected post-spend value, and the reserve label reads the
/// post-spend reserve. Current pool (5) covers the cost (4) → reserve
/// untouched. The card stays unchanged on the server, so the underlying
/// `PlayerEconomyView` does NOT mutate.
#[test]
fn mana_preview_shows_projected_value_during_active_drag() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    seed_hand_catalog(&mut app);
    set_phase(&mut app, RoundPhase::Placement);
    apply_economy(&mut app, /* current */ 5, /* reserve */ 2, /* cap */ 8);
    settle_hud_tween(&mut app);

    // Baseline — no drag yet; mana label reads authoritative "5 / 8" and the
    // reserve micro-readout reads "+2 reserve".
    let entities = hud_entities(&app);
    assert_eq!(text(&app, entities.mana_label), "5 / 8");
    assert_eq!(text(&app, entities.reserve_label), "+2 reserve");
    assert_eq!(
        app.world().resource::<PlayerEconomyView>().current_mana,
        5,
        "authoritative current_mana must not be touched by the preview path",
    );

    // Activate a synthetic drag — card_id + target_kind together signal
    // `is_active()` per the public fields contract of `ActivePlacementDrag`.
    start_drag(&mut app, CardId(TEST_CARD_ID), PlacementTargetKind::Minion);
    settle_hud_tween(&mut app);

    let mana_text_during_drag = text(&app, entities.mana_label);
    let reserve_text_during_drag = text(&app, entities.reserve_label);
    assert_eq!(
        mana_text_during_drag, "1 / 8",
        "mana label must read the projected current - cost (5 - 4 = 1) during an affordable drag",
    );
    assert_eq!(
        reserve_text_during_drag, "+2 reserve",
        "reserve label must remain at the authoritative reserve (cost paid entirely from current) during the affordable drag",
    );
    assert_eq!(
        app.world().resource::<PlayerEconomyView>().current_mana,
        5,
        "preview must not mutate PlayerEconomyView — server authority on mana is preserved",
    );
    assert_eq!(
        app.world().resource::<PlayerEconomyView>().reserve_mana,
        2,
        "preview must not mutate PlayerEconomyView.reserve_mana",
    );
}

/// AC2 — When the placement drag ends (cleared back to default), the next
/// frame's HUD mana label reverts to the authoritative readout. This is the
/// "preview is transient" guarantee — no explicit reset wire needed.
#[test]
fn mana_preview_reverts_when_drag_clears() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    seed_hand_catalog(&mut app);
    set_phase(&mut app, RoundPhase::Placement);
    apply_economy(&mut app, 5, 2, 8);
    settle_hud_tween(&mut app);

    let entities = hud_entities(&app);
    start_drag(&mut app, CardId(TEST_CARD_ID), PlacementTargetKind::Minion);
    settle_hud_tween(&mut app);
    assert_eq!(
        text(&app, entities.mana_label),
        "1 / 8",
        "sanity: preview should be active before we clear the drag",
    );

    // Drag end / cancel resets `ActivePlacementDrag` to default.
    clear_drag(&mut app);
    settle_hud_tween(&mut app);

    assert_eq!(
        text(&app, entities.mana_label),
        "5 / 8",
        "HUD mana label must revert to the authoritative current/cap after drag clears",
    );
    assert_eq!(
        text(&app, entities.reserve_label),
        "+2 reserve",
        "HUD reserve label must continue to reflect authoritative reserve after drag clears",
    );
}

/// AC3 — When the dragged card is unaffordable (`current + reserve < cost`),
/// the preview path stays silent and the HUD continues to display the
/// authoritative readout. Matches the affordability rule used by the
/// per-slot disabled-overlay system in `hand/drag_state_visuals.rs`.
#[test]
fn mana_preview_does_not_paint_for_unaffordable_drag() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    seed_hand_catalog(&mut app);
    set_phase(&mut app, RoundPhase::Placement);
    // Cost 4, but local has only current=1 + reserve=2 = 3 → unaffordable.
    apply_economy(&mut app, 1, 2, 8);
    settle_hud_tween(&mut app);

    let entities = hud_entities(&app);
    start_drag(&mut app, CardId(TEST_CARD_ID), PlacementTargetKind::Minion);
    settle_hud_tween(&mut app);

    assert_eq!(
        text(&app, entities.mana_label),
        "1 / 8",
        "unaffordable drag must NOT preview — HUD continues authoritative readout",
    );
    assert_eq!(
        text(&app, entities.reserve_label),
        "+2 reserve",
        "unaffordable drag must NOT modify reserve readout",
    );
}

/// AC4 — A drag whose cost spills into reserve (current=2, cost=4, reserve=3)
/// previews `(0, 1)` per the canonical spend order documented at
/// `client/src/presentation/shared/economy_view.rs::project_mana_after_spend`
/// — current first, then reserve.
#[test]
fn mana_preview_spills_into_reserve_when_current_insufficient() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    seed_hand_catalog(&mut app);
    set_phase(&mut app, RoundPhase::Placement);
    apply_economy(&mut app, 2, 3, 8);
    settle_hud_tween(&mut app);

    let entities = hud_entities(&app);
    start_drag(&mut app, CardId(TEST_CARD_ID), PlacementTargetKind::Minion);
    settle_hud_tween(&mut app);

    assert_eq!(
        text(&app, entities.mana_label),
        "0 / 8",
        "preview must subtract from current first (2 - 2 = 0)",
    );
    assert_eq!(
        text(&app, entities.reserve_label),
        "+1 reserve",
        "remaining cost (4 - 2 = 2) must spill into reserve (3 - 2 = 1)",
    );
}

// ── Harness ────────────────────────────────────────────────────────────────

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HudPlugin);
    // ActivePlacementDrag is normally provided by `HandUiPlugin`; provide a
    // bare default for the preview path's `Option<Res<…>>` lookup.
    app.init_resource::<ActivePlacementDrag>();
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn text(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .unwrap_or_else(|| panic!("{entity:?} should have Text"))
        .0
        .clone()
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
    app.update();
}

fn apply_economy(app: &mut App, current_mana: u32, reserve_mana: u32, mana_cap: u8) {
    *app.world_mut().resource_mut::<PlayerEconomyView>() = PlayerEconomyView {
        gold: 10,
        current_mana,
        reserve_mana,
        mana_cap,
        initialized: true,
        last_update_source: None,
    };
}

fn start_drag(app: &mut App, card_id: CardId, target_kind: PlacementTargetKind) {
    let drag_card = app.world_mut().spawn_empty().id();
    let mut drag = app.world_mut().resource_mut::<ActivePlacementDrag>();
    drag.card = Some(drag_card);
    drag.card_id = Some(card_id);
    drag.owner_id = Some(PlayerId(1));
    drag.target_kind = Some(target_kind);
}

fn clear_drag(app: &mut App) {
    *app.world_mut().resource_mut::<ActivePlacementDrag>() = ActivePlacementDrag::default();
}

fn seed_hand_catalog(app: &mut App) {
    app.insert_resource(HandCardCatalog {
        cards: catalog_with_test_card(),
    });
}

fn catalog_with_test_card() -> CardCatalog {
    let mut catalog: CardCatalog = std::collections::HashMap::new();
    catalog.insert(
        CardId(TEST_CARD_ID),
        CardData {
            id: CardId(TEST_CARD_ID),
            name_fr: "Aperçu Mana".to_string(),
            name_en: "Mana Preview".to_string(),
            class: ClassId::Iop,
            family: None,
            rarity: Rarity::Common,
            card_type: CardType::Minion,
            unit_type: UnitType::Blade,
            cost: TEST_CARD_COST,
            atk: 2,
            hp: 3,
            mp: 1,
            ar: 0,
            keywords: Vec::<Keyword>::new(),
            effect_text: String::new(),
            art_id: format!("test_{TEST_CARD_ID}"),
            pool_copies_override: None,
        },
    );
    catalog
}

/// Run the app for long enough that any in-flight HUD numeric tween
/// (`hud_tween_duration_ms` defaults to 300 ms) has fully settled before we
/// read the rendered label text.
fn settle_hud_tween(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::from_millis(1_000));
    app.update();
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}
