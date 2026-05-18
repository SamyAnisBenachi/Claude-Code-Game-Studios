//! PROMPT 1116 — S17-UI-BID-BUTTON-PHASE-RACE-001 integration coverage.
//!
//! Asserts SOURCE-1077-10 remediation at the entity level:
//!
//! - AC1: bid-button spawn-state `Text` is the meaningful pending label
//!   `"Loading…"` (the constant `AUCTION_BID_BUTTON_LOADING_LABEL`),
//!   not the previous `Text::new("")`.
//! - AC2: after `S2CAuctionCard` arrives the text is the numeric
//!   `BidButtonLabel` per TR-SAU-002 ("`{total}g\n(+{increment})`").
//! - AC3 / AC5: while `AuctionBidButtonState::HiddenLeading` is the
//!   active state, the bid row is `Visibility::Hidden` AND its
//!   `ImageNode.image` is `Handle::<Image>::default()` so the
//!   baked-`?` `ui_bid_button_disabled.png` chrome is not loaded onto
//!   the entity.
//! - AC4: in the phase-entry race window (DraftAuction entered, no
//!   `S2CAuctionCard` drained yet) the bid row is either
//!   `Visibility::Hidden` OR carries the `"Loading…"` text — the
//!   misleading numeric `BidButtonLabel` is not surfaced.
//! - AC7: the `auction_bid_chrome_state` mapping for `Normal` (Enabled)
//!   and `Disabled` (every other non-`HiddenLeading` state) is
//!   preserved end-to-end; only `HiddenLeading` got a new override.
//!
//! The story prohibits real-art replacement of
//! `assets/art/ui/auction/ui_bid_button_disabled.png` (PAW-TD-*-a
//! accept-risk); the PNG itself remains the canonical `Disabled`
//! chrome and these tests prove the entity does not advertise it
//! during the race / leading windows.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::asset_wiring::{bid_button_asset, BidButtonChromeState};
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::hud::HudGoldBroadcastMessage;
use client::ui::shop_auction::{
    AuctionBidButtonState, ShopAuctionAuctionCardReceived, ShopAuctionAuctionState,
    ShopAuctionCardCatalog, ShopAuctionLocalGoldView, ShopAuctionUiEntities, ShopAuctionUiPlugin,
    AUCTION_BID_BUTTON_LOADING_LABEL,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{RoundPhase, S2CGoldBroadcast};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const LOCAL_PLAYER: PlayerId = PlayerId(1);

/// AC1 — at spawn (immediately after `OnEnter(ClientState::InSession)`)
/// every bid-button `Text` component is the meaningful pending label,
/// not the empty string. Previously `Text::new("")` left a zero-content
/// component on the entity, which together with the baked-`?` chrome
/// was the SOURCE-1077-10 race surface.
#[test]
fn s17_phase_race_ac1_spawn_state_text_is_loading_label() {
    test_helpers::init_test_tracing();
    let app = app_in_session_no_phase();

    let texts = bid_button_texts(&app);
    for (index, text) in texts.iter().enumerate() {
        assert_eq!(
            text, AUCTION_BID_BUTTON_LOADING_LABEL,
            "bid button {index} spawn-state text must be the pending label {AUCTION_BID_BUTTON_LOADING_LABEL:?}, got {text:?}"
        );
        assert!(
            !text.is_empty(),
            "bid button {index} spawn-state text must not be empty (SOURCE-1077-10)"
        );
        assert!(
            !text.contains('?'),
            "bid button {index} spawn-state text must not contain '?' (PAW-TD-*-a), got {text:?}"
        );
    }
}

/// AC4 — in the phase-entry race window (DraftAuction phase entered,
/// `S2CAuctionCard` not yet drained) the bid row's entity contract
/// must be either `Visibility::Hidden` OR `Loading…` + non-`?` chrome.
/// The misleading numeric `BidButtonLabel` ("0g\n(+1)") must not be
/// surfaced.
#[test]
fn s17_phase_race_ac4_draft_auction_without_card_keeps_loading_or_hidden() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session_no_phase();

    set_phase(&mut app, RoundPhase::DraftAuction, 20_000);

    let texts = bid_button_texts(&app);
    let visibilities = bid_button_visibilities(&app);
    for (index, text) in texts.iter().enumerate() {
        let hidden = visibilities[index] == Visibility::Hidden;
        let loading = text == AUCTION_BID_BUTTON_LOADING_LABEL;
        assert!(
            hidden || loading,
            "bid button {index} during phase-entry race must be Visibility::Hidden OR carry the {AUCTION_BID_BUTTON_LOADING_LABEL:?} text, got visibility={:?} text={text:?}",
            visibilities[index]
        );
        assert!(
            !text.contains('?'),
            "bid button {index} phase-race text must not contain '?' glyph, got {text:?}"
        );
        // The auction state machine has `card_id == None` here, so the
        // misleading numeric label is the regression surface.
        assert_ne!(
            text.as_str(),
            "0g\n(+1)",
            "bid button {index} must not surface numeric BidButtonLabel before S2CAuctionCard arrives"
        );
    }

    // The chrome handle must not be the baked-`?` `Disabled` PNG while
    // the entity is visible (per AC4: "the `Disabled` chrome MUST NOT
    // be applied to the bid-button entity in the phase-entry race
    // window"). Visibility::Hidden entities are allowed to carry the
    // Disabled handle — option (b) of AC4.
    let images = bid_button_image_handles(&app);
    let disabled_path = bid_button_asset(BidButtonChromeState::Disabled);
    for (index, image) in images.iter().enumerate() {
        if visibilities[index] == Visibility::Hidden {
            continue;
        }
        let path = image
            .path()
            .map(|p| p.path().to_string_lossy().into_owned())
            .unwrap_or_default();
        assert_ne!(
            path, disabled_path,
            "bid button {index} must not load the baked-`?` Disabled chrome while visible during phase-race"
        );
    }
}

/// AC2 — once the server delivers `S2CAuctionCard` the entity contract
/// flips from the pending label to the numeric `BidButtonLabel`. The
/// numeric form matches TR-SAU-002 ("{total}g\n(+{increment})").
#[test]
fn s17_phase_race_ac2_text_updates_to_numeric_on_auction_card_arrival() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session_no_phase();

    set_phase(&mut app, RoundPhase::DraftAuction, 20_000);
    // Sanity: still in the race window — text must be the pending
    // label (or the row hidden) before the card arrives.
    let pre_texts = bid_button_texts(&app);
    let pre_visibilities = bid_button_visibilities(&app);
    for (index, text) in pre_texts.iter().enumerate() {
        let hidden = pre_visibilities[index] == Visibility::Hidden;
        assert!(
            hidden || text == AUCTION_BID_BUTTON_LOADING_LABEL,
            "pre-card bid button {index} must be hidden or Loading…, got {text:?}"
        );
    }

    // Drain S2CAuctionCard → `card_id.is_some()` → numeric labels.
    send_auction_card(&mut app, CardId(1), 0, 20_000);
    write_local_gold_broadcast(&mut app, 5, 0);

    let post_texts = bid_button_texts(&app);
    assert_eq!(
        post_texts,
        ["1g\n(+1)".to_string(), "3g\n(+3)".to_string(), "5g\n(+5)".to_string()],
        "after S2CAuctionCard drain, bid-button texts must be the numeric BidButtonLabel per TR-SAU-002"
    );
    for (index, text) in post_texts.iter().enumerate() {
        assert_ne!(
            text, AUCTION_BID_BUTTON_LOADING_LABEL,
            "bid button {index} must drop the pending label after S2CAuctionCard arrives"
        );
        assert!(
            !text.contains('?'),
            "bid button {index} post-card text must not contain '?' glyph, got {text:?}"
        );
    }
}

/// AC3 + AC5 — while the local player leads the auction the bid-button
/// state is `HiddenLeading`. The row's `Visibility` is `Hidden` AND
/// the `ImageNode.image` is `Handle::<Image>::default()` so the
/// baked-`?` `Disabled` chrome is not loaded onto the entity.
#[test]
fn s17_phase_race_ac3_ac5_hidden_leading_clears_chrome_and_hides_row() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(0, 20_000);
    write_local_gold_broadcast(&mut app, 5, 0);

    app.world_mut()
        .resource_mut::<ShopAuctionAuctionState>()
        .current_leader = Some(LOCAL_PLAYER);
    run_update(&mut app);

    let states = bid_button_states(&app);
    assert_eq!(
        states,
        [
            AuctionBidButtonState::HiddenLeading,
            AuctionBidButtonState::HiddenLeading,
            AuctionBidButtonState::HiddenLeading,
        ],
        "current_leader == local must drive every bid button into HiddenLeading"
    );

    let visibilities = bid_button_visibilities(&app);
    assert_eq!(
        visibilities,
        [Visibility::Hidden, Visibility::Hidden, Visibility::Hidden],
        "HiddenLeading bid row must be Visibility::Hidden (AC3 option a)"
    );

    let images = bid_button_image_handles(&app);
    for (index, image) in images.iter().enumerate() {
        assert_eq!(
            image.id(),
            Handle::<Image>::default().id(),
            "bid button {index} ImageNode.image must be Handle::<Image>::default() during HiddenLeading (AC3 option b / AC5: no baked-`?` chrome on the entity)"
        );
    }
}

/// AC7 — `auction_bid_chrome_state` mapping for `Normal` and
/// `Disabled` is preserved end-to-end. Only `HiddenLeading` gained a
/// new override. The mapper is a private fn so we exercise it through
/// the entity contract: Enabled rows load the `Normal` PNG path,
/// non-`HiddenLeading` disabled rows (Unaffordable / HandFullLocked /
/// LocallyExpired / GenericDisabled) load the `Disabled` PNG path.
#[test]
fn s17_phase_race_ac7_chrome_mapping_preserved_for_enabled_and_disabled_states() {
    test_helpers::init_test_tracing();
    let mut app = app_in_active_auction(0, 20_000);
    write_local_gold_broadcast(&mut app, 2, 0);

    // free_gold = 2 → +1 Enabled, +3 / +5 Unaffordable.
    assert_eq!(
        bid_button_states(&app),
        [
            AuctionBidButtonState::Enabled,
            AuctionBidButtonState::Unaffordable,
            AuctionBidButtonState::Unaffordable,
        ]
    );

    let images = bid_button_image_handles(&app);
    let normal_path = bid_button_asset(BidButtonChromeState::Normal);
    let disabled_path = bid_button_asset(BidButtonChromeState::Disabled);
    let paths: Vec<String> = images
        .iter()
        .map(|image| {
            image
                .path()
                .map(|p| p.path().to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .collect();

    assert_eq!(
        paths[0], normal_path,
        "Enabled bid button must load the Normal chrome PNG"
    );
    assert_eq!(
        paths[1], disabled_path,
        "Unaffordable bid button must load the Disabled chrome PNG (mapping preserved)"
    );
    assert_eq!(
        paths[2], disabled_path,
        "Unaffordable bid button must load the Disabled chrome PNG (mapping preserved)"
    );
}

// ──────────────────────────────────────────────────────────────────────
// fixtures (parallel of the sibling `auction_bid_buttons_test.rs`
// harness — the standalone test bin can not share helpers across files
// without an extra `#[path]` include, and the surface area we need here
// is small enough to keep local).
// ──────────────────────────────────────────────────────────────────────

fn app_in_session_no_phase() -> App {
    let mut app = base_app();
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    app
}

fn app_in_active_auction(starting_price: u32, timer_duration_ms: u32) -> App {
    let mut app = app_in_session_no_phase();
    set_phase(&mut app, RoundPhase::DraftAuction, timer_duration_ms);
    send_auction_card(&mut app, CardId(1), starting_price, timer_duration_ms);
    app
}

fn base_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(ShopAuctionCardCatalog {
        cards: HashMap::from([(CardId(1), test_card(1, Rarity::Rare, 4))]),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 10,
        initialized: true,
        ..default()
    });
    app.insert_resource(ShopAuctionLocalGoldView {
        player_id: Some(LOCAL_PLAYER),
        gold: 10,
        reserved_gold: 0,
        initialized: true,
    });
    app
}

fn set_phase(app: &mut App, phase: RoundPhase, timer_duration_ms: u32) {
    let round = app.world().resource::<CurrentClientPhase>().round + 1;
    {
        let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
        current.phase = phase;
        current.round = round;
    }
    {
        let mut phase_view = app.world_mut().resource_mut::<ClientPhaseView>();
        phase_view.phase = phase;
        phase_view.round_number = round;
        phase_view.timer_duration_ms = timer_duration_ms;
    }
    run_update(app);
}

fn send_auction_card(app: &mut App, card_id: CardId, starting_price: u32, timer_duration_ms: u32) {
    app.world_mut()
        .write_message(ShopAuctionAuctionCardReceived {
            card_id,
            starting_price,
            timer_duration_ms,
        });
    run_update(app);
}

fn write_local_gold_broadcast(app: &mut App, gold: u32, reserved_gold: u32) {
    app.world_mut()
        .write_message(HudGoldBroadcastMessage(S2CGoldBroadcast {
            player_id: LOCAL_PLAYER,
            gold,
            reserved_gold,
        }));
    run_update(app);
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(std::time::Duration::ZERO);
    app.update();
}

fn bid_button_states(app: &App) -> [AuctionBidButtonState; 3] {
    let buttons = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons;
    buttons.map(|button| {
        *app.world()
            .get::<AuctionBidButtonState>(button)
            .expect("bid button should have a state")
    })
}

fn bid_button_texts(app: &App) -> [String; 3] {
    let buttons = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons;
    buttons.map(|button| {
        app.world()
            .get::<Text>(button)
            .expect("bid button should have text")
            .0
            .clone()
    })
}

fn bid_button_visibilities(app: &App) -> [Visibility; 3] {
    let buttons = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons;
    buttons.map(|button| {
        *app.world()
            .get::<Visibility>(button)
            .expect("bid button should have visibility")
    })
}

fn bid_button_image_handles(app: &App) -> [Handle<Image>; 3] {
    let buttons = app
        .world()
        .resource::<ShopAuctionUiEntities>()
        .auction_bid_buttons;
    buttons.map(|button| {
        app.world()
            .get::<ImageNode>(button)
            .expect("bid button should have ImageNode")
            .image
            .clone()
    })
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
