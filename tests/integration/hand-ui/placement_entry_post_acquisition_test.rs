use std::collections::HashMap;
use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::{prelude::*, time::Virtual};
use bevy_tweening::TweeningPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    FanSlotIndex, HandCardCatalog, HandContents, HandSlotCard, HandUiCardAcquiredReceived,
    HandUiPlugin, HandUiTimingConfig, ReserveStripButton, ReserveStripForFanSlot,
    ReserveStripValueText, HAND_FAN_SLOT_COUNT,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

const ACQUIRED_CARD_COUNT: usize = 3;
const FIRST_ACQUIRED_CARD_ID: u32 = 50;

// Regression for the Verdict 2 child-visibility leak: PROMPT 623 traced the
// "RESERVE 0 CURRENT 0" text and the `[-] / [+]` buttons leaking onto the fan
// at PLACEMENT entry to `Visibility::Visible` being baked into the reserve
// strip children at spawn time. The parent strip is `Visibility::Hidden`
// until a card is staged, but children with `Visibility::Visible` ignore
// the parent's hidden state and render anyway. Children must declare
// `Visibility::Inherited` so the parent's Hidden propagates. AC-27
// (HU-27 free-card strip hidden) and AC-13(d) (un-stage hides the strip)
// both rely on this propagation.
#[test]
fn reserve_strip_children_inherit_hidden_after_three_acquisitions_at_placement_entry() {
    let mut app = app_with_hand_ui_in_draft_initial();

    for offset in 0..ACQUIRED_CARD_COUNT {
        let card_id = CardId(FIRST_ACQUIRED_CARD_ID + offset as u32);
        app.world_mut()
            .write_message(HandUiCardAcquiredReceived { card_id });
        run_update(&mut app);
    }

    assert_eq!(
        app.world().resource::<HandContents>().cards.len(),
        ACQUIRED_CARD_COUNT,
        "HandContents must hold the {ACQUIRED_CARD_COUNT} acquired cards",
    );

    set_phase(&mut app, RoundPhase::Placement);
    run_update(&mut app);

    for index in 0..ACQUIRED_CARD_COUNT {
        let slot = fan_slot(&mut app, index as u8);
        assert_eq!(
            app.world().get::<Visibility>(slot).copied(),
            Some(Visibility::Visible),
            "fan slot {index} must be Visible after acquisition + PLACEMENT entry",
        );
        let expected_card = CardId(FIRST_ACQUIRED_CARD_ID + index as u32);
        assert_eq!(
            app.world().get::<HandSlotCard>(slot),
            Some(&HandSlotCard(expected_card)),
            "fan slot {index} must carry HandSlotCard for the acquired card",
        );
    }

    for slot_index in 0..HAND_FAN_SLOT_COUNT as u8 {
        let strip = reserve_strip_parent(&mut app, slot_index);
        assert_eq!(
            app.world().get::<Visibility>(strip).copied(),
            Some(Visibility::Hidden),
            "reserve strip parent {slot_index} must be Hidden at PLACEMENT entry (no card staged yet)",
        );
    }

    let value_texts = collect_value_text_visibilities(&mut app);
    assert_eq!(
        value_texts.len(),
        HAND_FAN_SLOT_COUNT,
        "exactly {HAND_FAN_SLOT_COUNT} ReserveStripValueText children must exist (one per fan slot)",
    );
    for (slot_index, visibility) in value_texts {
        assert_eq!(
            visibility,
            Visibility::Inherited,
            "ReserveStripValueText for slot {slot_index} must declare Visibility::Inherited so the parent's Hidden state propagates",
        );
    }

    let buttons = collect_button_visibilities(&mut app);
    assert_eq!(
        buttons.len(),
        HAND_FAN_SLOT_COUNT * 2,
        "exactly {} ReserveStripButton children must exist ({} slots x 2 buttons)",
        HAND_FAN_SLOT_COUNT * 2,
        HAND_FAN_SLOT_COUNT,
    );
    for (slot_index, visibility) in buttons {
        assert_eq!(
            visibility,
            Visibility::Inherited,
            "ReserveStripButton for slot {slot_index} must declare Visibility::Inherited so the parent's Hidden state propagates",
        );
    }
}

fn app_with_hand_ui_in_draft_initial() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(TweeningPlugin);
    app.add_plugins(HandUiPlugin);
    app.insert_resource(HandCardCatalog {
        cards: test_catalog((FIRST_ACQUIRED_CARD_ID)..(FIRST_ACQUIRED_CARD_ID + 32)),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 5,
        reserve_mana: 0,
        initialized: true,
        ..default()
    });
    app.insert_resource(HandUiTimingConfig {
        card_draw_animation_ms: 280,
        purchase_timeout_ms: 3_000,
        hand_full_notification_duration_ms: 2_000,
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
    set_phase(&mut app, RoundPhase::DraftInitial);
    run_update(&mut app);
    app
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn fan_slot(app: &mut App, slot_index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, idx)| (idx.0 == slot_index).then_some(entity))
        .expect("fan slot must exist")
}

fn reserve_strip_parent(app: &mut App, slot_index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &ReserveStripForFanSlot)>();
    query
        .iter(app.world())
        .find_map(|(entity, marker)| (marker.0 == slot_index).then_some(entity))
        .expect("reserve strip parent must exist")
}

fn collect_value_text_visibilities(app: &mut App) -> Vec<(u8, Visibility)> {
    let mut query = app
        .world_mut()
        .query::<(&ReserveStripValueText, &Visibility)>();
    let mut out: Vec<(u8, Visibility)> = query
        .iter(app.world())
        .map(|(marker, visibility)| (marker.0, *visibility))
        .collect();
    out.sort_by_key(|(slot, _)| *slot);
    out
}

fn collect_button_visibilities(app: &mut App) -> Vec<(u8, Visibility)> {
    let mut query = app
        .world_mut()
        .query::<(&ReserveStripButton, &Visibility)>();
    let mut out: Vec<(u8, Visibility)> = query
        .iter(app.world())
        .map(|(marker, visibility)| (marker.slot_index, *visibility))
        .collect();
    out.sort_by_key(|(slot, _)| *slot);
    out
}

fn test_catalog(ids: impl IntoIterator<Item = u32>) -> HashMap<CardId, CardData> {
    ids.into_iter()
        .map(|id| {
            let card = test_card(id);
            (card.id, card)
        })
        .collect()
}

fn test_card(id: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: 1,
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
