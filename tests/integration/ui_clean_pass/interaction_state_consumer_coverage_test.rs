//! S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001 (Sprint 18 story-025 /
//! PROMPT 1729) — consumer-coverage structural guard.
//!
//! Verifies the Wave-2 P1 button overlay-tint migration across three
//! consumer files: `client/src/ui/lobby.rs`, `client/src/ui/shop_auction/
//! mod.rs`, and `client/src/ui/hand/mod.rs`.
//!
//! ## What is checked
//!
//! - **AC7**: every P1 button spawn site carries `CursorIcon::System` in its
//!   spawn tuple (structural regression guard against accidental removal).
//! - **AC8 (reach-through)**: `interaction_states` token module imported in
//!   all three consumer files (`HOVER_BG_TINT_ALPHA` present).
//! - **AC9**: this file (the test itself).
//! - **AC10 regression guard**: the Wave-2 spawn sites do not introduce new
//!   bare `Color::srgb(` / `Color::srgba(` literals inside their button
//!   spawn tuples (all base colors must be named constants or helper calls).
//!
//! ## What is NOT checked
//!
//! - Runtime BackgroundColor values — those require a full ECS world and are
//!   covered by manual smoke testing per the story's Advisory evidence tier.
//! - The 7-state grandfathered lobby Confirm CTA bands — see AC1 carve-out.
//! - Any file outside the Wave-2 owned set (hud, settings, etc.).

use std::fs;
use std::path::{Path, PathBuf};

fn client_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("client crate has parent dir")
        .join("client")
        .join("src")
}

fn read_source(rel: &str) -> String {
    let path = client_src_root().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {rel}: {err}"))
}

// ─── helpers ─────────────────────────────────────────────────────────────

/// Returns the spawn tuple text for each `.spawn((` block that contains the
/// `needle` component marker.  Naive paren-depth scanner; sufficient for the
/// structural checks below.
fn spawn_tuples_containing(source: &str, needle: &str) -> Vec<String> {
    let mut results = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].contains(".spawn((") {
            // collect until paren depth returns to 0
            let start = i;
            let mut depth: i32 = 0;
            let mut buf = String::new();
            while i < lines.len() {
                let line = lines[i];
                for ch in line.chars() {
                    match ch {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                }
                buf.push_str(line);
                buf.push('\n');
                i += 1;
                if depth <= 0 && i > start + 1 {
                    break;
                }
            }
            if buf.contains(needle) {
                results.push(buf);
            }
        } else {
            i += 1;
        }
    }
    results
}

// ─── AC7 – CursorIcon::System on every P1 button spawn ──────────────────

#[test]
fn ac7_lobby_create_room_button_has_cursor_icon() {
    let src = read_source("ui/lobby.rs");
    let tuples = spawn_tuples_containing(&src, "LobbyCreateRoomButton");
    assert!(
        !tuples.is_empty(),
        "LobbyCreateRoomButton spawn tuple not found in lobby.rs"
    );
    for t in &tuples {
        assert!(
            t.contains("CursorIcon::System"),
            "LobbyCreateRoomButton spawn is missing CursorIcon::System\n{t}"
        );
    }
}

#[test]
fn ac7_lobby_join_room_button_has_cursor_icon() {
    let src = read_source("ui/lobby.rs");
    let tuples = spawn_tuples_containing(&src, "LobbyJoinRoomButton");
    assert!(
        !tuples.is_empty(),
        "LobbyJoinRoomButton spawn tuple not found in lobby.rs"
    );
    for t in &tuples {
        assert!(
            t.contains("CursorIcon::System"),
            "LobbyJoinRoomButton spawn is missing CursorIcon::System\n{t}"
        );
    }
}

#[test]
fn ac7_lobby_confirm_button_has_cursor_icon() {
    let src = read_source("ui/lobby.rs");
    let tuples = spawn_tuples_containing(&src, "LobbyConfirmButton");
    assert!(
        !tuples.is_empty(),
        "LobbyConfirmButton spawn tuple not found in lobby.rs"
    );
    for t in &tuples {
        assert!(
            t.contains("CursorIcon::System"),
            "LobbyConfirmButton spawn is missing CursorIcon::System\n{t}"
        );
    }
}

#[test]
fn ac7_shop_ready_button_has_cursor_icon() {
    let src = read_source("ui/shop_auction/mod.rs");
    let tuples = spawn_tuples_containing(&src, "ShopReadyButton");
    assert!(
        !tuples.is_empty(),
        "ShopReadyButton spawn tuple not found in shop_auction/mod.rs"
    );
    for t in &tuples {
        assert!(
            t.contains("CursorIcon::System"),
            "ShopReadyButton spawn is missing CursorIcon::System\n{t}"
        );
    }
}

#[test]
fn ac7_shop_refresh_button_has_cursor_icon() {
    let src = read_source("ui/shop_auction/mod.rs");
    let tuples = spawn_tuples_containing(&src, "ShopRefreshButton");
    assert!(
        !tuples.is_empty(),
        "ShopRefreshButton spawn tuple not found"
    );
    for t in &tuples {
        assert!(
            t.contains("CursorIcon::System"),
            "ShopRefreshButton spawn is missing CursorIcon::System\n{t}"
        );
    }
}

#[test]
fn ac7_auction_pass_button_has_cursor_icon() {
    let src = read_source("ui/shop_auction/mod.rs");
    let tuples = spawn_tuples_containing(&src, "AuctionPassButton");
    assert!(
        !tuples.is_empty(),
        "AuctionPassButton spawn tuple not found"
    );
    for t in &tuples {
        assert!(
            t.contains("CursorIcon::System"),
            "AuctionPassButton spawn is missing CursorIcon::System\n{t}"
        );
    }
}

#[test]
fn ac7_auction_bid_buttons_have_cursor_icon() {
    let src = read_source("ui/shop_auction/mod.rs");
    let tuples = spawn_tuples_containing(&src, "AuctionBidButton");
    assert!(
        !tuples.is_empty(),
        "AuctionBidButton spawn tuple not found"
    );
    for t in &tuples {
        assert!(
            t.contains("CursorIcon::System"),
            "AuctionBidButton spawn is missing CursorIcon::System\n{t}"
        );
    }
}

#[test]
fn ac7_draft_initial_ready_button_has_cursor_icon() {
    let src = read_source("ui/shop_auction/mod.rs");
    let tuples = spawn_tuples_containing(&src, "DraftInitialReadyButton");
    assert!(
        !tuples.is_empty(),
        "DraftInitialReadyButton spawn tuple not found"
    );
    for t in &tuples {
        assert!(
            t.contains("CursorIcon::System"),
            "DraftInitialReadyButton spawn is missing CursorIcon::System\n{t}"
        );
    }
}

#[test]
fn ac7_draft_initial_objective_dismiss_button_has_cursor_icon() {
    let src = read_source("ui/shop_auction/mod.rs");
    let tuples = spawn_tuples_containing(&src, "DraftInitialObjectiveDismissButton");
    assert!(
        !tuples.is_empty(),
        "DraftInitialObjectiveDismissButton spawn tuple not found"
    );
    for t in &tuples {
        assert!(
            t.contains("CursorIcon::System"),
            "DraftInitialObjectiveDismissButton spawn is missing CursorIcon::System\n{t}"
        );
    }
}

#[test]
fn ac7_draft_initial_objective_retrieval_button_has_cursor_icon() {
    let src = read_source("ui/shop_auction/mod.rs");
    let tuples = spawn_tuples_containing(&src, "DraftInitialObjectiveRetrievalButton");
    assert!(
        !tuples.is_empty(),
        "DraftInitialObjectiveRetrievalButton spawn tuple not found"
    );
    for t in &tuples {
        assert!(
            t.contains("CursorIcon::System"),
            "DraftInitialObjectiveRetrievalButton spawn is missing CursorIcon::System\n{t}"
        );
    }
}

#[test]
fn ac7_hand_submit_button_has_cursor_icon() {
    let src = read_source("ui/hand/mod.rs");
    let tuples = spawn_tuples_containing(&src, "HandSubmitButton");
    assert!(
        !tuples.is_empty(),
        "HandSubmitButton spawn tuple not found in hand/mod.rs"
    );
    for t in &tuples {
        assert!(
            t.contains("CursorIcon::System"),
            "HandSubmitButton spawn is missing CursorIcon::System\n{t}"
        );
    }
}

// ─── AC8 reach-through – interaction_states tokens imported in consumers ─

#[test]
fn ac8_lobby_imports_hover_bg_tint_alpha() {
    let src = read_source("ui/lobby.rs");
    assert!(
        src.contains("HOVER_BG_TINT_ALPHA"),
        "lobby.rs must import HOVER_BG_TINT_ALPHA from interaction_states"
    );
}

#[test]
fn ac8_shop_auction_imports_hover_bg_tint_alpha() {
    let src = read_source("ui/shop_auction/mod.rs");
    assert!(
        src.contains("HOVER_BG_TINT_ALPHA"),
        "shop_auction/mod.rs must import HOVER_BG_TINT_ALPHA from interaction_states"
    );
}

#[test]
fn ac8_hand_imports_hover_bg_tint_alpha() {
    let src = read_source("ui/hand/mod.rs");
    assert!(
        src.contains("HOVER_BG_TINT_ALPHA"),
        "hand/mod.rs must import HOVER_BG_TINT_ALPHA from interaction_states"
    );
}

// ─── overlay systems registered in plugins ───────────────────────────────

#[test]
fn overlay_systems_registered_in_lobby_plugin() {
    let src = read_source("ui/lobby.rs");
    assert!(
        src.contains("lobby_create_join_interaction_overlay_system"),
        "LobbyUiPlugin must register lobby_create_join_interaction_overlay_system"
    );
}

#[test]
fn overlay_systems_registered_in_shop_auction_plugin() {
    let src = read_source("ui/shop_auction/mod.rs");
    assert!(
        src.contains("shop_auction_primary_button_interaction_overlay_system"),
        "ShopAuctionUiPlugin must register shop_auction_primary_button_interaction_overlay_system"
    );
    assert!(
        src.contains("auction_bid_button_interaction_overlay_system"),
        "ShopAuctionUiPlugin must register auction_bid_button_interaction_overlay_system"
    );
}

#[test]
fn overlay_system_registered_in_hand_plugin() {
    let src = read_source("ui/hand/mod.rs");
    assert!(
        src.contains("hand_submit_button_interaction_overlay_system"),
        "HandUiPlugin must register hand_submit_button_interaction_overlay_system"
    );
}

// ─── AC10 – no new inline RGB literals at Wave-2 spawn sites ─────────────

#[test]
fn ac10_lobby_create_join_spawn_uses_named_constants() {
    let src = read_source("ui/lobby.rs");
    for marker in &["LobbyCreateRoomButton", "LobbyJoinRoomButton"] {
        let tuples = spawn_tuples_containing(&src, marker);
        for t in &tuples {
            assert!(
                !t.contains("Color::srgba(0."),
                "Spawn site for {marker} must not contain inline Color::srgba literal — use named constants\n{t}"
            );
        }
    }
}

#[test]
fn ac10_auction_pass_button_spawn_uses_named_constants() {
    let src = read_source("ui/shop_auction/mod.rs");
    let tuples = spawn_tuples_containing(&src, "AuctionPassButton");
    for t in &tuples {
        assert!(
            !t.contains("Color::srgba(0.12"),
            "AuctionPassButton spawn must use AUCTION_PASS_BUTTON_BG constant, not inline literal\n{t}"
        );
    }
}
