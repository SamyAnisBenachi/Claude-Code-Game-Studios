//! PROMPT 1138 — Lobby class-portrait + confirm-area repair regression
//! (S18-UI-LOBBY-CLASS-PORTRAIT-CONFIRM-REPAIR).
//!
//! Guards the four AUDIT-1129 lobby repairs:
//!
//! - **AC1 (UI-1129-07)** — every class picker cell composites a
//!   class-distinct emblem `ImageNode` overlay
//!   (`LobbyClassPickerEmblem`) on top of the portrait. The emblem
//!   handle resolves to the `class_type_icon_asset(class_id)` path so
//!   the picker can read class identity at a glance even when the
//!   canonical lobby portrait slot still hosts a generic placeholder.
//! - **AC2 (UI-1129-08)** — each slot-panel image owns an inline
//!   `Text` label child (`LobbyOwnSlotLabel` / `LobbyOpponentSlotLabel`)
//!   so the two slot chips read as informative status ("You · {class}
//!   · slot N" / "Opp · waiting") instead of an unidentified pair of
//!   blue card placeholders.
//! - **AC3** — `lobby_class_picker_cell_colors` paints the locked
//!   class cell with the same green palette as
//!   `LobbyConfirmButtonStyleState::Confirmed`, so the confirmed-class
//!   visual state is unambiguous and ties the picker cell and the
//!   confirm CTA into one decision.
//! - **AC4 (UI-1129-13)** — `lobby_status_copy` no longer contains
//!   pipe `|` delimiters; bullet `·` separators are used; `Players:
//!   N/M` substring is preserved; the two-line bound is preserved.
//!
//! Friend-game scope only. This bin does NOT advance `QA-COND-0005`
//! Standard-tier accessibility, `QA-COND-0006` playtest validation, or
//! `PAW-TD-*-a` placeholder-art accept-risk.

use bevy::asset::AssetPlugin;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::asset_wiring::class_type_icon_asset;
use client::state::ClientState;
use client::ui::lobby::{
    lobby_opponent_slot_label_text, lobby_own_slot_label_text, lobby_status_copy,
    LobbyClassPickerCell, LobbyClassPickerEmblem, LobbyInputState, LobbyOpponentSlotLabel,
    LobbyOpponentSlotPanel, LobbyOwnSlotLabel, LobbyOwnSlotPanel, LobbyUiPlugin, LobbyViewState,
};
use shared::card::ClassId;
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn spawn_lobby_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(LobbyUiPlugin);
    app.update();
    app.update();
    app
}

const ALL_CLASS_IDS: [ClassId; 7] = [
    ClassId::Iop,
    ClassId::Cra,
    ClassId::Sacrier,
    ClassId::Xelor,
    ClassId::Ecaflip,
    ClassId::Sadida,
    ClassId::Neutral,
];

// ── AC1: class-distinct emblem overlay present on every picker cell. ───────

#[test]
fn ac1_every_picker_cell_owns_a_class_distinct_emblem_image_node() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let mut query = world.query::<(&LobbyClassPickerEmblem, &ImageNode)>();
    let emblems: Vec<(ClassId, Handle<Image>)> = query
        .iter(world)
        .map(|(e, img)| (e.class_id, img.image.clone()))
        .collect();

    assert_eq!(
        emblems.len(),
        ALL_CLASS_IDS.len(),
        "AC1: expected {} class emblem entities (one per ClassId), got {}",
        ALL_CLASS_IDS.len(),
        emblems.len()
    );

    for class_id in ALL_CLASS_IDS {
        let handle = emblems
            .iter()
            .find_map(|(c, h)| (*c == class_id).then(|| h.clone()))
            .unwrap_or_else(|| panic!("AC1: missing emblem entity for {:?}", class_id));
        assert_ne!(
            handle,
            Handle::<Image>::default(),
            "AC1: emblem ImageNode for {:?} must be a non-default handle",
            class_id
        );
    }
}

#[test]
fn ac1_emblem_handle_matches_class_type_icon_asset_path() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    let expected: Vec<(ClassId, Handle<Image>)> = {
        let world = app.world();
        let asset_server = world.resource::<AssetServer>().clone();
        ALL_CLASS_IDS
            .iter()
            .map(|c| (*c, asset_server.load(class_type_icon_asset(*c))))
            .collect()
    };

    let world = app.world_mut();
    let mut query = world.query::<(&LobbyClassPickerEmblem, &ImageNode)>();
    let actual: Vec<(ClassId, Handle<Image>)> = query
        .iter(world)
        .map(|(e, img)| (e.class_id, img.image.clone()))
        .collect();

    for (class_id, expected_handle) in &expected {
        let found = actual
            .iter()
            .any(|(c, h)| c == class_id && h == expected_handle);
        assert!(
            found,
            "AC1: emblem for {:?} must bind asset_server.load(class_type_icon_asset({:?}))",
            class_id, class_id
        );
    }
}

#[test]
fn ac1_emblem_is_descendant_of_its_class_cell() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let mut cell_entities: Vec<(Entity, ClassId)> = world
        .query::<(Entity, &LobbyClassPickerCell)>()
        .iter(world)
        .map(|(e, c)| (e, c.class_id))
        .collect();
    cell_entities.sort_by_key(|(_, c)| format!("{c:?}"));

    let mut emblem_entities: Vec<(Entity, ClassId)> = world
        .query::<(Entity, &LobbyClassPickerEmblem)>()
        .iter(world)
        .map(|(e, c)| (e, c.class_id))
        .collect();
    emblem_entities.sort_by_key(|(_, c)| format!("{c:?}"));

    assert_eq!(
        cell_entities.len(),
        emblem_entities.len(),
        "AC1: cell count must equal emblem count"
    );

    // The emblem must be reachable by descending the cell's children
    // tree (cell -> portrait -> emblem). Confirm at least one path.
    for (cell_entity, class_id) in &cell_entities {
        let mut found_emblem_under_cell = false;
        let mut stack = vec![*cell_entity];
        while let Some(node) = stack.pop() {
            if let Some(emblem) = world.entity(node).get::<LobbyClassPickerEmblem>() {
                if emblem.class_id == *class_id {
                    found_emblem_under_cell = true;
                    break;
                }
            }
            if let Some(children) = world.entity(node).get::<Children>() {
                stack.extend(children.iter());
            }
        }
        assert!(
            found_emblem_under_cell,
            "AC1: emblem for {:?} must live inside the matching picker cell's subtree",
            class_id
        );
    }
}

// ── AC2: slot panels carry inline text labels. ─────────────────────────────

#[test]
fn ac2_own_slot_panel_owns_a_text_label_child() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let own_panel = world
        .query_filtered::<Entity, With<LobbyOwnSlotPanel>>()
        .iter(world)
        .next()
        .expect("AC2: a LobbyOwnSlotPanel must exist after lobby spawn");

    let mut found_label = false;
    if let Some(children) = world.entity(own_panel).get::<Children>() {
        for child in children.iter() {
            if world.entity(child).get::<LobbyOwnSlotLabel>().is_some()
                && world.entity(child).get::<Text>().is_some()
            {
                found_label = true;
                break;
            }
        }
    }

    assert!(
        found_label,
        "AC2: LobbyOwnSlotPanel must own a direct child carrying both \
         LobbyOwnSlotLabel and Text (AUDIT-1129-08 grouped slot label)"
    );
}

#[test]
fn ac2_opponent_slot_panel_owns_a_text_label_child() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let opp_panel = world
        .query_filtered::<Entity, With<LobbyOpponentSlotPanel>>()
        .iter(world)
        .next()
        .expect("AC2: a LobbyOpponentSlotPanel must exist after lobby spawn");

    let mut found_label = false;
    if let Some(children) = world.entity(opp_panel).get::<Children>() {
        for child in children.iter() {
            if world
                .entity(child)
                .get::<LobbyOpponentSlotLabel>()
                .is_some()
                && world.entity(child).get::<Text>().is_some()
            {
                found_label = true;
                break;
            }
        }
    }

    assert!(
        found_label,
        "AC2: LobbyOpponentSlotPanel must own a direct child carrying both \
         LobbyOpponentSlotLabel and Text (AUDIT-1129-08 grouped slot label)"
    );
}

#[test]
fn ac2_own_slot_label_text_announces_class_and_slot() {
    test_helpers::init_test_tracing();
    let mut lobby = LobbyViewState::default();
    let mut input = LobbyInputState::default();
    input.selected_class = ClassId::Sacrier;
    input.requested_slot = 2;

    let pre_confirm = lobby_own_slot_label_text(&lobby, &input);
    assert!(
        pre_confirm.contains("You"),
        "AC2: pre-confirm own-slot label must include 'You' (got {pre_confirm:?})"
    );
    assert!(
        pre_confirm.contains("Sacrier"),
        "AC2: pre-confirm own-slot label must include selected class name (got {pre_confirm:?})"
    );
    assert!(
        pre_confirm.contains("slot 2"),
        "AC2: pre-confirm own-slot label must include 'slot N' (got {pre_confirm:?})"
    );
    assert!(
        !pre_confirm.contains('*'),
        "AC2: pre-confirm own-slot label must not carry the confirmed '*' marker yet \
         (got {pre_confirm:?})"
    );

    lobby.locked_class = Some(ClassId::Sacrier);
    let post_confirm = lobby_own_slot_label_text(&lobby, &input);
    assert!(
        post_confirm.contains('*'),
        "AC2: post-confirm own-slot label must carry the confirmed '*' marker \
         (got {post_confirm:?})"
    );
    assert_ne!(
        pre_confirm, post_confirm,
        "AC2: confirmed-state label must differ from pre-confirm label"
    );
}

#[test]
fn ac2_opponent_slot_label_text_distinguishes_waiting_vs_revealed() {
    test_helpers::init_test_tracing();
    let mut lobby = LobbyViewState::default();

    let waiting = lobby_opponent_slot_label_text(&lobby);
    assert!(
        waiting.contains("Opp"),
        "AC2: waiting opponent-slot label must include 'Opp' (got {waiting:?})"
    );
    assert!(
        waiting.contains("waiting"),
        "AC2: waiting opponent-slot label must announce the wait state explicitly \
         (got {waiting:?})"
    );

    // Reveal an opponent class. local_player_id stays unset so the
    // opponent resolver picks the first revealed pair.
    lobby.revealed_classes = vec![(PlayerId(42), ClassId::Xelor)];
    let revealed = lobby_opponent_slot_label_text(&lobby);
    assert!(
        revealed.contains("Xelor"),
        "AC2: post-reveal opponent-slot label must include the revealed class \
         (got {revealed:?})"
    );
    assert!(
        !revealed.contains("waiting"),
        "AC2: post-reveal opponent-slot label must not still say 'waiting' \
         (got {revealed:?})"
    );
}

// ── AC3: locked class picker cell uses the Confirmed green palette. ────────

#[test]
fn ac3_locked_class_picker_cell_paints_confirmed_green() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    // Pre-lock baseline: Iop is selected by default, locked_class None.
    let iop_unlocked_border = {
        let world = app.world_mut();
        let mut q = world.query::<(&LobbyClassPickerCell, &BorderColor)>();
        q.iter(world)
            .find(|(c, _)| c.class_id == ClassId::Iop)
            .map(|(_, b)| *b)
            .expect("AC3: Iop cell must exist")
    };

    // Simulate a server-acknowledged class lock and let
    // refresh_lobby_ui_system repaint.
    {
        let mut lobby = app.world_mut().resource_mut::<LobbyViewState>();
        lobby.locked_class = Some(ClassId::Iop);
    }
    app.update();
    app.update();

    let iop_locked_border = {
        let world = app.world_mut();
        let mut q = world.query::<(&LobbyClassPickerCell, &BorderColor)>();
        q.iter(world)
            .find(|(c, _)| c.class_id == ClassId::Iop)
            .map(|(_, b)| *b)
            .expect("AC3: Iop cell must still exist after lock")
    };

    assert_ne!(
        iop_unlocked_border.top, iop_locked_border.top,
        "AC3: Iop cell border must repaint when the class is server-locked"
    );

    // Confirm the green palette matches LobbyConfirmButtonStyleState::Confirmed.
    let confirmed_border = Color::srgb(0.40, 0.84, 0.50).to_srgba();
    let actual = iop_locked_border.top.to_srgba();
    let close = |a: f32, b: f32| (a - b).abs() < 1e-4;
    assert!(
        close(actual.red, confirmed_border.red)
            && close(actual.green, confirmed_border.green)
            && close(actual.blue, confirmed_border.blue),
        "AC3: locked-class cell border must mirror the Confirmed-state green \
         (expected ~{confirmed_border:?}, got {actual:?})"
    );
}

// ── AC4: status banner copy is grouped and pipe-free. ──────────────────────

#[test]
fn ac4_status_copy_drops_pipe_delimiters() {
    test_helpers::init_test_tracing();
    let lobby = LobbyViewState::default();
    let input = LobbyInputState::default();
    let copy = lobby_status_copy(&lobby, &input);

    assert!(
        !copy.contains('|'),
        "AC4: lobby_status_copy must not contain pipe delimiters anymore \
         (AUDIT-1129-13 terminal-log feel). Got: {copy:?}"
    );
}

#[test]
fn ac4_status_copy_uses_bullet_separator() {
    test_helpers::init_test_tracing();
    let lobby = LobbyViewState::default();
    let input = LobbyInputState::default();
    let copy = lobby_status_copy(&lobby, &input);

    assert!(
        copy.contains('·'),
        "AC4: lobby_status_copy must use bullet `·` separators between status \
         groups for visual grouping. Got: {copy:?}"
    );
}

#[test]
fn ac4_status_copy_preserves_players_substring_and_two_line_bound() {
    test_helpers::init_test_tracing();
    let lobby = LobbyViewState::default();
    let input = LobbyInputState::default();
    let copy = lobby_status_copy(&lobby, &input);

    assert!(
        copy.contains("Players: "),
        "AC4: lobby_status_copy must preserve the `Players: ` substring \
         (depended on by lobby_entry_test::class_confirmations_are_server_confirmed \
         and PROMPT 985 reachability). Got: {copy:?}"
    );

    let line_count = copy.matches('\n').count() + 1;
    assert!(
        line_count <= 2,
        "AC4: lobby_status_copy must still render in at most 2 lines so the \
         confirm CTA stays reachable at the minimum 1366×768 viewport \
         (PROMPT 985). Got {line_count} lines: {copy:?}"
    );
}

// ── AC5 (PROMPT 1178): slot panels read as muted status chips, not buttons.

#[test]
fn ac5_own_slot_panel_image_node_is_tinted_to_read_as_status_chip() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let panel = world
        .query_filtered::<Entity, With<LobbyOwnSlotPanel>>()
        .iter(world)
        .next()
        .expect("AC5: LobbyOwnSlotPanel must exist");
    let image = world
        .entity(panel)
        .get::<ImageNode>()
        .expect("AC5: LobbyOwnSlotPanel must carry ImageNode");
    let color = image.color.to_srgba();
    // White (1, 1, 1, 1) is the bevy_ui default — would render the
    // panel asset at full button-like saturation. PROMPT 1178 tints
    // the asset down so the chip reads as informational status. The
    // exact RGBA chosen is in `lobby_slot_chip_image_node`; this test
    // just guards against a regression back to white.
    let is_pure_white = (color.red - 1.0).abs() < 1e-4
        && (color.green - 1.0).abs() < 1e-4
        && (color.blue - 1.0).abs() < 1e-4
        && (color.alpha - 1.0).abs() < 1e-4;
    assert!(
        !is_pure_white,
        "AC5: LobbyOwnSlotPanel.ImageNode.color must be tinted (not \
         pure white) so the chip reads as status, not a primary \
         button. Got {color:?}"
    );
}

#[test]
fn ac5_opponent_slot_panel_image_node_is_tinted_to_read_as_status_chip() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let panel = world
        .query_filtered::<Entity, With<LobbyOpponentSlotPanel>>()
        .iter(world)
        .next()
        .expect("AC5: LobbyOpponentSlotPanel must exist");
    let image = world
        .entity(panel)
        .get::<ImageNode>()
        .expect("AC5: LobbyOpponentSlotPanel must carry ImageNode");
    let color = image.color.to_srgba();
    let is_pure_white = (color.red - 1.0).abs() < 1e-4
        && (color.green - 1.0).abs() < 1e-4
        && (color.blue - 1.0).abs() < 1e-4
        && (color.alpha - 1.0).abs() < 1e-4;
    assert!(
        !is_pure_white,
        "AC5: LobbyOpponentSlotPanel.ImageNode.color must be tinted (not \
         pure white) so the chip reads as status, not a primary \
         button. Got {color:?}"
    );
}

#[test]
fn ac5_slot_panels_carry_no_button_marker() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();
    let world = app.world_mut();

    let own = world
        .query_filtered::<Entity, With<LobbyOwnSlotPanel>>()
        .iter(world)
        .next()
        .expect("AC5: LobbyOwnSlotPanel must exist");
    assert!(
        world.entity(own).get::<Button>().is_none(),
        "AC5: LobbyOwnSlotPanel must NOT carry a Button marker — it is \
         a status chip, not a primary action"
    );
    let opp = world
        .query_filtered::<Entity, With<LobbyOpponentSlotPanel>>()
        .iter(world)
        .next()
        .expect("AC5: LobbyOpponentSlotPanel must exist");
    assert!(
        world.entity(opp).get::<Button>().is_none(),
        "AC5: LobbyOpponentSlotPanel must NOT carry a Button marker — it \
         is a status chip, not a primary action"
    );
}

// ── AC6 (PROMPT 1178): own-slot label prefers authoritative slot. ─────────

#[test]
fn ac6_own_slot_label_prefers_authoritative_lobby_slots_over_input_default() {
    use shared::protocol::SessionSlot;
    test_helpers::init_test_tracing();
    // Server confirmed the local player is in slot 2; the
    // `LobbyInputState::default()` `requested_slot = 1` is now stale.
    let lobby = LobbyViewState {
        local_player_id: Some(PlayerId(7)),
        slots: vec![
            SessionSlot {
                slot: 2,
                team: 1,
                player_id: Some(PlayerId(7)),
                class_id: None,
                class_confirmed: false,
                is_bot: false,
            },
            SessionSlot {
                slot: 1,
                team: 0,
                player_id: Some(PlayerId(99)),
                class_id: None,
                class_confirmed: false,
                is_bot: false,
            },
        ],
        ..Default::default()
    };
    let input = LobbyInputState::default(); // requested_slot = 1.
    let label = lobby_own_slot_label_text(&lobby, &input);
    assert!(
        label.contains("slot 2"),
        "AC6: own-slot label must prefer the authoritative `lobby.slots` \
         seat (slot 2) over the stale `input.requested_slot` default \
         (slot 1). Got: {label:?}"
    );
    assert!(
        !label.contains("slot 1"),
        "AC6: own-slot label must NOT show the stale `input.requested_slot` \
         (slot 1) when the server-confirmed slot differs. Got: {label:?}"
    );
}

#[test]
fn ac6_own_slot_label_falls_back_to_input_requested_slot_pre_join() {
    test_helpers::init_test_tracing();
    // Pre-join: `lobby.slots` is empty, `local_player_id` is None.
    let lobby = LobbyViewState::default();
    let mut input = LobbyInputState::default();
    input.requested_slot = 3;
    let label = lobby_own_slot_label_text(&lobby, &input);
    assert!(
        label.contains("slot 3"),
        "AC6: pre-join own-slot label must fall back to \
         `input.requested_slot` (slot 3) when `lobby.slots` is empty. \
         Got: {label:?}"
    );
}

#[test]
fn ac4_status_copy_drops_redundant_status_and_join_prefixes() {
    test_helpers::init_test_tracing();
    let lobby = LobbyViewState::default();
    let input = LobbyInputState::default();
    let copy = lobby_status_copy(&lobby, &input);

    // The legacy format opened with "Status: ..." and "Join: ...".
    // After the AUDIT-1129-13 regrouping those redundant prefixes are
    // dropped — context is implied by line position.
    assert!(
        !copy.contains("Status: "),
        "AC4: lobby_status_copy must drop the legacy `Status: ` prefix. Got: {copy:?}"
    );
    assert!(
        !copy.contains("Join: "),
        "AC4: lobby_status_copy must drop the legacy `Join: ` prefix. Got: {copy:?}"
    );
}
