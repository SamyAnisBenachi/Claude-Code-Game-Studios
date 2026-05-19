//! Integration tests for PROMPT 1229 (S18-QA-SNAPSHOT-PLACEMENT-AUCTION-STATE-001 /
//! B-1203-X-03). Locks the top-level `placement_state`, `auction_state`, and
//! `current_phase.timer_remaining_ms` JSON fields against the
//! [`client::presentation::qa_snapshot`] schema so PROMPT 1203 audits can
//! correlate QA screenshots with the local placement intent + auction state
//! without source archaeology under `extras.*`.
//!
//! Assertions:
//!  - Every documented top-level key is present in the JSON shape (null /
//!    default allowed; missing keys are not).
//!  - `available = false` + nested nulls when the source resources are
//!    absent (lobby / pre-handshake / outside the placement/auction phase).
//!  - Populated fields surface meaningful values when the extras bag carries
//!    placement or auction state (no fixture spawn — pure projection of the
//!    `ExtrasSnapshot` shape that `ExtrasInputs::snapshot` already produces
//!    in production).

use client::presentation::qa_snapshot::{
    build_auction_state_snapshot, build_placement_state_snapshot, build_snapshot,
    build_snapshot_with_extras, AuctionPanelSnapshot, AuctionStateSnapshot, AuctionTimerSnapshot,
    DragSnapshot, ExtrasSnapshot, HandSnapshot, LocalGoldViewSnapshot, PhaseTimerSnapshot,
    PlacementStateSnapshot, PlacementTimerSnapshot, PlayerResourcesSnapshot, ScreenshotInfo,
    ShopAuctionExtrasSnapshot, TimersSnapshot, UiCounts, QA_SCREENSHOT_FILENAME,
    QA_SCREENSHOT_FORMAT, SCREENSHOT_STATUS_PENDING,
};

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn placeholder_screenshot(requested_at_ms: u128) -> ScreenshotInfo {
    ScreenshotInfo {
        relative_path: QA_SCREENSHOT_FILENAME.to_string(),
        absolute_path: format!("/abs/{QA_SCREENSHOT_FILENAME}"),
        format: QA_SCREENSHOT_FORMAT.to_string(),
        requested_at_ms,
        status: SCREENSHOT_STATUS_PENDING.to_string(),
        captured_at_ms: None,
        error: None,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Schema-presence: every documented key present even on a defaulted snapshot.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_build_snapshot_emits_top_level_placement_and_auction_state_keys() {
    // Arrange / Act — defaulted snapshot (no extras).
    let snapshot = build_snapshot(
        0,
        0,
        placeholder_screenshot(0),
        None,
        None,
        None,
        None,
        None,
        UiCounts::default(),
    );

    // Act — serialise to JSON so we can grep for every documented field key.
    let json = serde_json::to_value(&snapshot).expect("snapshot serialises");

    // Assert — top-level keys present.
    assert!(
        json.get("placement_state").is_some(),
        "placement_state top-level key must be present"
    );
    assert!(
        json.get("snapshot_utc_iso").is_some(),
        "snapshot_utc_iso top-level key must be present"
    );
    assert!(
        json.get("evidence_layers").is_some(),
        "evidence_layers top-level key must be present"
    );
    assert!(
        json.get("ui_text_markers").is_some(),
        "ui_text_markers top-level key must be present"
    );
    assert!(
        json.get("auction_state").is_some(),
        "auction_state top-level key must be present"
    );

    // current_phase.timer_remaining_ms present (null permitted).
    let current_phase = json
        .get("current_phase")
        .expect("current_phase block present");
    assert!(
        current_phase.get("timer_remaining_ms").is_some(),
        "current_phase.timer_remaining_ms key must be present"
    );

    // placement_state schema keys.
    let placement = json.get("placement_state").unwrap();
    for key in [
        "available",
        "staged_count",
        "can_submit",
        "submitted",
        "drag_active",
        "drag_card_id",
        "drag_target_kind",
        "disclosure_step",
        "submit_disabled_reason",
        "invalid_pending_indices",
        "pending_placement_source",
        "last_rejection_state",
    ] {
        assert!(
            placement.get(key).is_some(),
            "placement_state.{key} key must be present"
        );
    }

    // auction_state schema keys.
    let auction = json.get("auction_state").unwrap();
    for key in [
        "available",
        "panel_state",
        "card_id",
        "starting_price",
        "current_price",
        "current_leader",
        "timer_duration_ms",
        "timer_remaining_ms",
        "local_in_flight_bid_amount",
        "local_gold",
        "local_player_id",
        "leader_is_local",
        "leader_label_text",
        "price_label_text",
        "timer_label_text",
    ] {
        assert!(
            auction.get(key).is_some(),
            "auction_state.{key} key must be present"
        );
    }
}

#[test]
fn test_default_snapshot_marks_placement_and_auction_state_unavailable() {
    let snapshot = build_snapshot(
        0,
        0,
        placeholder_screenshot(0),
        None,
        None,
        None,
        None,
        None,
        UiCounts::default(),
    );

    assert_eq!(snapshot.placement_state.available, false);
    assert!(snapshot.placement_state.staged_count.is_none());
    assert!(snapshot.placement_state.can_submit.is_none());
    assert!(snapshot.placement_state.submitted.is_none());
    assert!(snapshot.placement_state.drag_active.is_none());
    assert!(snapshot.placement_state.drag_card_id.is_none());
    assert!(snapshot.placement_state.drag_target_kind.is_none());
    assert!(snapshot.placement_state.disclosure_step.is_none());
    assert!(snapshot.placement_state.submit_disabled_reason.is_none());
    assert!(snapshot.placement_state.invalid_pending_indices.is_empty());
    assert!(snapshot.placement_state.pending_placement_source.is_none());
    assert!(snapshot.placement_state.last_rejection_state.is_none());

    assert_eq!(snapshot.auction_state.available, false);
    assert!(snapshot.auction_state.panel_state.is_none());
    assert!(snapshot.auction_state.card_id.is_none());
    assert!(snapshot.auction_state.starting_price.is_none());
    assert!(snapshot.auction_state.current_price.is_none());
    assert!(snapshot.auction_state.current_leader.is_none());
    assert!(snapshot.auction_state.timer_duration_ms.is_none());
    assert!(snapshot.auction_state.timer_remaining_ms.is_none());
    assert!(snapshot.auction_state.local_in_flight_bid_amount.is_none());
    assert!(snapshot.auction_state.local_gold.is_none());
    assert!(snapshot.auction_state.local_player_id.is_none());
    assert!(snapshot.auction_state.leader_is_local.is_none());
    assert!(snapshot.auction_state.leader_label_text.is_none());
    assert!(snapshot.auction_state.price_label_text.is_none());
    assert!(snapshot.auction_state.timer_label_text.is_none());

    assert!(snapshot.current_phase.timer_remaining_ms.is_none());
}

// ─────────────────────────────────────────────────────────────────────────
// current_phase.timer_remaining_ms is lifted from extras.timers.phase_timer.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_current_phase_timer_remaining_ms_lifted_from_extras_phase_timer() {
    let extras = ExtrasSnapshot {
        timers: TimersSnapshot {
            sampled_at_unix_ms: None,
            sampled_at_utc_iso: None,
            phase_timer: Some(PhaseTimerSnapshot {
                phase_started_elapsed_ms: Some(2_000),
                phase_duration_ms: 30_000,
                duration_ms: 30_000,
                elapsed_ms: 12_500,
                computed_remaining_ms: 17_500,
                remaining_ms: 17_500,
                display_text: "18s".to_string(),
                timer_source: "hud_phase_timer_state".to_string(),
                active: true,
            }),
            placement_timer: None,
            auction_timer: None,
            shop_timer: None,
        },
        ..ExtrasSnapshot::default()
    };

    let snapshot = build_snapshot_with_extras(
        0,
        0,
        placeholder_screenshot(0),
        None,
        None,
        None,
        None,
        None,
        UiCounts::default(),
        extras,
    );

    assert_eq!(
        snapshot.current_phase.timer_remaining_ms,
        Some(17_500),
        "current_phase.timer_remaining_ms must be lifted from extras.timers.phase_timer.remaining_ms"
    );

    let phase_timer = snapshot
        .extras
        .timers
        .phase_timer
        .expect("phase timer snapshot must stay populated");
    assert_eq!(phase_timer.phase_duration_ms, 30_000);
    assert_eq!(phase_timer.computed_remaining_ms, 17_500);
    assert_eq!(phase_timer.duration_ms, 30_000);
    assert_eq!(phase_timer.remaining_ms, 17_500);
    assert_eq!(phase_timer.display_text, "18s");
    assert_eq!(phase_timer.timer_source, "hud_phase_timer_state");
}

// ─────────────────────────────────────────────────────────────────────────
// PlacementStateSnapshot — pure projection of extras.hand + extras.drag +
// extras.timers.placement_timer.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_placement_state_lifts_staged_count_submitted_and_can_submit() {
    let extras = ExtrasSnapshot {
        timers: TimersSnapshot {
            placement_timer: Some(PlacementTimerSnapshot {
                remaining_ms: 4_200,
                urgency_fired: false,
                in_grace_window: false,
                grace_remaining_ms: 0,
                submitted: false,
            }),
            ..TimersSnapshot::default()
        },
        hand: Some(HandSnapshot {
            mode: Some("Staging".to_string()),
            disclosure_step: Some("StagedCard".to_string()),
            hand_count: 3,
            cards: Vec::new(),
            pending_placements: Vec::new(),
            staged_count: 2,
        }),
        drag: DragSnapshot {
            placement_drag_active: true,
            placement_drag_card_id: Some(57),
            placement_drag_target_kind: Some("Minion".to_string()),
            ..DragSnapshot::default()
        },
        ..ExtrasSnapshot::default()
    };

    let state = build_placement_state_snapshot(&extras);
    assert!(state.available);
    assert_eq!(state.staged_count, Some(2));
    assert_eq!(state.submitted, Some(false));
    // 2 staged + not submitted -> can submit.
    assert_eq!(state.can_submit, Some(true));
    assert_eq!(state.drag_active, Some(true));
    assert_eq!(state.drag_card_id, Some(57));
    assert_eq!(state.drag_target_kind.as_deref(), Some("Minion"));
    assert_eq!(state.disclosure_step.as_deref(), Some("StagedCard"));
    assert_eq!(state.submit_disabled_reason, None);
    assert_eq!(
        state.pending_placement_source.as_deref(),
        Some("cursor_drop")
    );
}

#[test]
fn test_placement_state_can_submit_false_when_already_submitted() {
    let extras = ExtrasSnapshot {
        timers: TimersSnapshot {
            placement_timer: Some(PlacementTimerSnapshot {
                remaining_ms: 0,
                urgency_fired: true,
                in_grace_window: false,
                grace_remaining_ms: 0,
                submitted: true,
            }),
            ..TimersSnapshot::default()
        },
        hand: Some(HandSnapshot {
            mode: Some("Submitted".to_string()),
            disclosure_step: Some("Submitted".to_string()),
            hand_count: 1,
            cards: Vec::new(),
            pending_placements: Vec::new(),
            staged_count: 1,
        }),
        ..ExtrasSnapshot::default()
    };

    let state = build_placement_state_snapshot(&extras);
    assert!(state.available);
    assert_eq!(state.submitted, Some(true));
    // 1 staged + already submitted -> cannot submit.
    assert_eq!(state.can_submit, Some(false));
    assert_eq!(
        state.submit_disabled_reason.as_deref(),
        Some("already_submitted")
    );
}

#[test]
fn test_placement_state_can_submit_false_when_zero_staged() {
    let extras = ExtrasSnapshot {
        timers: TimersSnapshot {
            placement_timer: Some(PlacementTimerSnapshot {
                remaining_ms: 9_000,
                urgency_fired: false,
                in_grace_window: false,
                grace_remaining_ms: 0,
                submitted: false,
            }),
            ..TimersSnapshot::default()
        },
        hand: Some(HandSnapshot::default()),
        ..ExtrasSnapshot::default()
    };

    let state = build_placement_state_snapshot(&extras);
    assert!(state.available);
    assert_eq!(state.staged_count, Some(0));
    assert_eq!(state.can_submit, Some(false));
    assert_eq!(
        state.submit_disabled_reason.as_deref(),
        Some("no_staged_placements")
    );
}

#[test]
fn test_placement_state_defaults_when_no_hand_or_timer() {
    let extras = ExtrasSnapshot::default();
    let state = build_placement_state_snapshot(&extras);
    assert!(!state.available);
    assert!(state.staged_count.is_none());
    assert!(state.can_submit.is_none());
    assert!(state.submitted.is_none());
    assert!(state.drag_active.is_none());
    assert!(state.drag_target_kind.is_none());
    assert!(state.disclosure_step.is_none());
    assert!(state.submit_disabled_reason.is_none());
    assert!(state.invalid_pending_indices.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────
// AuctionStateSnapshot — pure projection of extras.shop_auction.auction +
// extras.resources.local_gold_view.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_auction_state_lifts_card_price_leader_timer_and_local_gold() {
    let extras = ExtrasSnapshot {
        shop_auction: Some(ShopAuctionExtrasSnapshot {
            auction: Some(AuctionPanelSnapshot {
                panel_state: "Active".to_string(),
                card_id: Some(123),
                starting_price: 4,
                current_price: 9,
                current_leader: Some("PlayerId(2)".to_string()),
                timer_duration_ms: 12_000,
                timer_remaining_ms: 4_800,
                preparing_elapsed_ms: 0,
                locally_expired_elapsed_ms: 0,
                in_flight_bid_amount: Some(11),
                pending_bid_accepted: false,
                pending_gold_broadcast_seen: false,
                opponent_bid_gate_satisfied: true,
                waiting_for_local_gold_after_opponent_bid: false,
            }),
            ..ShopAuctionExtrasSnapshot::default()
        }),
        resources: Some(PlayerResourcesSnapshot {
            initialized: true,
            last_update_source: Some("GoldUpdate".to_string()),
            gold: 18,
            current_mana: 0,
            reserve_mana: 0,
            mana_cap: 6,
            local_gold_view: Some(LocalGoldViewSnapshot {
                initialized: true,
                gold: 18,
                reserved_gold: 11,
                free_gold: 7,
            }),
        }),
        ..ExtrasSnapshot::default()
    };

    let state = build_auction_state_snapshot(&extras);
    assert!(state.available);
    assert_eq!(state.panel_state.as_deref(), Some("Active"));
    assert_eq!(state.card_id, Some(123));
    assert_eq!(state.starting_price, Some(4));
    assert_eq!(state.current_price, Some(9));
    assert_eq!(state.current_leader.as_deref(), Some("PlayerId(2)"));
    assert_eq!(state.timer_duration_ms, Some(12_000));
    assert_eq!(state.timer_remaining_ms, Some(4_800));
    assert_eq!(state.local_in_flight_bid_amount, Some(11));
    let gold = state.local_gold.expect("local_gold projected");
    assert_eq!(gold.gold, 18);
    assert_eq!(gold.reserved_gold, 11);
    assert_eq!(gold.free_gold, 7);
    assert!(gold.view_initialized);
}

#[test]
fn test_auction_state_defaults_when_no_auction_resource() {
    let extras = ExtrasSnapshot::default();
    let state = build_auction_state_snapshot(&extras);
    assert!(!state.available);
    assert!(state.panel_state.is_none());
    assert!(state.card_id.is_none());
    assert!(state.current_price.is_none());
    assert!(state.current_leader.is_none());
    assert!(state.timer_remaining_ms.is_none());
    assert!(state.local_gold.is_none());
}

#[test]
fn test_auction_state_marks_unavailable_when_only_resources_present() {
    // Resources present but no shop_auction.auction → audit must see
    // available=false rather than a half-populated auction block.
    let extras = ExtrasSnapshot {
        resources: Some(PlayerResourcesSnapshot {
            initialized: true,
            last_update_source: Some("GoldUpdate".to_string()),
            gold: 10,
            current_mana: 0,
            reserve_mana: 0,
            mana_cap: 6,
            local_gold_view: Some(LocalGoldViewSnapshot {
                initialized: true,
                gold: 10,
                reserved_gold: 0,
                free_gold: 10,
            }),
        }),
        ..ExtrasSnapshot::default()
    };

    let state = build_auction_state_snapshot(&extras);
    assert!(!state.available);
    assert!(
        state.local_gold.is_none(),
        "local_gold must NOT surface under auction_state when no auction resource is present"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// Defaults serialise to stable JSON values (no `Option` skipping).
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_default_placement_state_serialises_to_stable_null_shape() {
    let state = PlacementStateSnapshot::default();
    let json = serde_json::to_value(&state).unwrap();
    assert_eq!(json["available"], false);
    assert!(json["staged_count"].is_null());
    assert!(json["can_submit"].is_null());
    assert!(json["submitted"].is_null());
    assert!(json["drag_active"].is_null());
    assert!(json["drag_card_id"].is_null());
    assert!(json["drag_target_kind"].is_null());
    assert!(json["disclosure_step"].is_null());
}

#[test]
fn test_default_auction_state_serialises_to_stable_null_shape() {
    let state = AuctionStateSnapshot::default();
    let json = serde_json::to_value(&state).unwrap();
    assert_eq!(json["available"], false);
    assert!(json["panel_state"].is_null());
    assert!(json["card_id"].is_null());
    assert!(json["starting_price"].is_null());
    assert!(json["current_price"].is_null());
    assert!(json["current_leader"].is_null());
    assert!(json["timer_duration_ms"].is_null());
    assert!(json["timer_remaining_ms"].is_null());
    assert!(json["local_in_flight_bid_amount"].is_null());
    assert!(json["local_gold"].is_null());
    assert!(json["local_player_id"].is_null());
    assert!(json["leader_is_local"].is_null());
    assert!(json["leader_label_text"].is_null());
}

// ─────────────────────────────────────────────────────────────────────────
// Auction timer doc cross-check — auction_state.timer_remaining_ms agrees
// with extras.timers.auction_timer.remaining_ms when both blocks are
// populated.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_auction_state_timer_remaining_ms_agrees_with_extras_auction_timer() {
    let auction_panel = AuctionPanelSnapshot {
        panel_state: "Active".to_string(),
        card_id: Some(7),
        starting_price: 3,
        current_price: 3,
        current_leader: None,
        timer_duration_ms: 15_000,
        timer_remaining_ms: 9_750,
        preparing_elapsed_ms: 0,
        locally_expired_elapsed_ms: 0,
        in_flight_bid_amount: None,
        pending_bid_accepted: false,
        pending_gold_broadcast_seen: false,
        opponent_bid_gate_satisfied: true,
        waiting_for_local_gold_after_opponent_bid: false,
    };
    let extras = ExtrasSnapshot {
        timers: TimersSnapshot {
            auction_timer: Some(AuctionTimerSnapshot {
                panel_state: "Active".to_string(),
                duration_ms: auction_panel.timer_duration_ms,
                remaining_ms: auction_panel.timer_remaining_ms,
                preparing_elapsed_ms: auction_panel.preparing_elapsed_ms,
                locally_expired_elapsed_ms: auction_panel.locally_expired_elapsed_ms,
            }),
            ..TimersSnapshot::default()
        },
        shop_auction: Some(ShopAuctionExtrasSnapshot {
            auction: Some(auction_panel),
            ..ShopAuctionExtrasSnapshot::default()
        }),
        ..ExtrasSnapshot::default()
    };
    let state = build_auction_state_snapshot(&extras);
    assert_eq!(state.timer_remaining_ms, Some(9_750));
    let extras_remaining = extras.timers.auction_timer.as_ref().map(|t| t.remaining_ms);
    assert_eq!(
        state.timer_remaining_ms, extras_remaining,
        "top-level auction_state.timer_remaining_ms must agree with extras.timers.auction_timer.remaining_ms"
    );
}
