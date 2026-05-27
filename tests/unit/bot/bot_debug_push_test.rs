//! Standalone unit tests for BotDebugPushPlugin (PROMPT 1628).
//!
//! Complements the inline `#[cfg(test)]` block in
//! `server/src/feature/bot/debug_push.rs`.  That block validates the heavy
//! assembly paths (ordering, tail-cap, class fallback, hand/economy mapping).
//! These tests focus on the orthogonal surface:
//!
//! - Full env-var matrix (1 / 0 / unset / empty / garbage / whitespace).
//! - Config default values match the published constants.
//! - `BotDebugPushState` default + sequence-wrapping arithmetic.
//! - `decision_kind_label` for every variant including payload-carrying ones.
//! - `decision_detail` None-path for every fieldless variant.
//! - `decision_detail` string formatting for every payload variant.
//! - `assemble_debug_bot_state_push` with zero bots (no panic, empty payload).
//! - Tail-cap at exact boundary (16 entries → no trim).
//! - Tail-cap below cap (3 entries → all returned).

use std::collections::HashSet;

use server::feature::bot::{
    assemble_debug_bot_state_push, decision_detail, decision_kind_label,
    BotDebugPushConfig, BotDebugPushState, BotDecisionEntry, BotDecisionKind, BotDecisionLog,
    BotPlayers, BotState, DEBUG_BOT_DECISION_TAIL_CAP, DEFAULT_BOT_DEBUG_PUSH_INTERVAL_MS,
};
use shared::card::{CardId, ClassId};
use shared::protocol::{CardSource, RoundPhase as WireRoundPhase};
use shared::session::PlayerId;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_entry(player: PlayerId, kind: BotDecisionKind) -> BotDecisionEntry {
    BotDecisionEntry {
        round_number: 1,
        phase: WireRoundPhase::DraftAuction,
        bot_player_id: player,
        decision: kind,
        timestamp_ms: 0,
        legal_action_count: None,
        seed: 1,
        seed_word_counter: 0,
    }
}

fn bot_with_id(id: u64) -> (PlayerId, BotPlayers) {
    let pid = PlayerId(id);
    let mut bots = BotPlayers::default();
    bots.insert(BotState::new(pid, 0));
    (pid, bots)
}

// ---------------------------------------------------------------------------
// Env-var matrix
// ---------------------------------------------------------------------------

#[test]
fn test_env_var_config_explicit_one_enables_regardless_of_default() {
    // "1" must override even when dev_default = false.
    let cfg = BotDebugPushConfig::from_env_values(Some("1"), false);
    assert!(cfg.enabled);
}

#[test]
fn test_env_var_config_explicit_zero_disables_regardless_of_default() {
    // "0" must override even when dev_default = true.
    let cfg = BotDebugPushConfig::from_env_values(Some("0"), true);
    assert!(!cfg.enabled);
}

#[test]
fn test_env_var_config_unset_falls_back_to_dev_default_true() {
    let cfg = BotDebugPushConfig::from_env_values(None, true);
    assert!(cfg.enabled);
}

#[test]
fn test_env_var_config_unset_falls_back_to_dev_default_false() {
    let cfg = BotDebugPushConfig::from_env_values(None, false);
    assert!(!cfg.enabled);
}

#[test]
fn test_env_var_config_empty_string_treats_as_unset() {
    // "" → falls back to dev_default.
    assert!(BotDebugPushConfig::from_env_values(Some(""), true).enabled);
    assert!(!BotDebugPushConfig::from_env_values(Some(""), false).enabled);
}

#[test]
fn test_env_var_config_garbage_value_disables() {
    // Any unrecognised value is treated as disabled.
    for garbage in &["garbage", "2", "true", "yes", "on", "YES", "TRUE", "10", "-1"] {
        let cfg = BotDebugPushConfig::from_env_values(Some(garbage), true);
        assert!(
            !cfg.enabled,
            "expected disabled for env value {:?} with dev_default=true",
            garbage
        );
    }
}

#[test]
fn test_env_var_config_whitespace_only_treats_as_unset() {
    // Whitespace-only string is trimmed to "" → falls back to dev_default.
    assert!(BotDebugPushConfig::from_env_values(Some("   "), true).enabled);
    assert!(!BotDebugPushConfig::from_env_values(Some("\t"), false).enabled);
}

// ---------------------------------------------------------------------------
// Config default values
// ---------------------------------------------------------------------------

#[test]
fn test_config_default_values_match_published_constants() {
    let cfg = BotDebugPushConfig::default();
    assert!(!cfg.enabled, "default must be disabled");
    assert_eq!(
        cfg.interval_ms, DEFAULT_BOT_DEBUG_PUSH_INTERVAL_MS,
        "interval_ms must equal DEFAULT_BOT_DEBUG_PUSH_INTERVAL_MS"
    );
    assert_eq!(
        cfg.tail_cap, DEBUG_BOT_DECISION_TAIL_CAP,
        "tail_cap must equal DEBUG_BOT_DECISION_TAIL_CAP"
    );
}

#[test]
fn test_config_from_env_values_preserves_interval_and_tail_cap() {
    // from_env_values only sets `enabled`; interval and cap must stay at defaults.
    let cfg = BotDebugPushConfig::from_env_values(Some("1"), false);
    assert_eq!(cfg.interval_ms, DEFAULT_BOT_DEBUG_PUSH_INTERVAL_MS);
    assert_eq!(cfg.tail_cap, DEBUG_BOT_DECISION_TAIL_CAP);
}

// ---------------------------------------------------------------------------
// BotDebugPushState arithmetic
// ---------------------------------------------------------------------------

#[test]
fn test_debug_push_state_default_is_zero() {
    let s = BotDebugPushState::default();
    assert_eq!(s.next_push_ms, 0, "next_push_ms must be 0 (fire on first tick)");
    assert_eq!(s.sequence, 0);
}

#[test]
fn test_debug_push_state_sequence_wraps_on_overflow() {
    let s = BotDebugPushState {
        sequence: u64::MAX,
        next_push_ms: 0,
    };
    let next_seq = s.sequence.wrapping_add(1);
    assert_eq!(next_seq, 0, "sequence must wrap to 0 on u64::MAX + 1");
}

#[test]
fn test_debug_push_state_next_push_saturating_add() {
    // saturating_add is the production expression; verify it doesn't panic near u64::MAX.
    let now: u64 = u64::MAX - 100;
    let result = now.saturating_add(DEFAULT_BOT_DEBUG_PUSH_INTERVAL_MS);
    assert_eq!(result, u64::MAX);
}

// ---------------------------------------------------------------------------
// decision_kind_label — all 12 variants
// ---------------------------------------------------------------------------

#[test]
fn test_decision_kind_label_all_variants() {
    let cases: &[(BotDecisionKind, &str)] = &[
        (BotDecisionKind::ClassChosen { class_id: ClassId::Cra }, "class_chosen"),
        (BotDecisionKind::ClassConfirmed, "class_confirmed"),
        (
            BotDecisionKind::Purchased {
                card_id: CardId(1),
                source: CardSource::ShopPurchase,
                gold_after: 0,
            },
            "purchased",
        ),
        (BotDecisionKind::Refreshed { gold_after: 5 }, "refreshed"),
        (BotDecisionKind::PurchaseSkipped { reason: "no_gold" }, "purchase_skipped"),
        (BotDecisionKind::DraftReady, "draft_ready"),
        (
            BotDecisionKind::AuctionBid {
                card_id: CardId(2),
                amount: 1,
                valuation: 2,
            },
            "auction_bid",
        ),
        (BotDecisionKind::AuctionPass { reason: "low_val" }, "auction_pass"),
        (BotDecisionKind::PlacementSubmitted { placements_len: 3, coords: vec![] }, "placement_submitted"),
        (BotDecisionKind::PlacementSkipped { reason: "no_units" }, "placement_skipped"),
        (BotDecisionKind::EmptyPlacementFailsafe, "empty_placement_failsafe"),
        (BotDecisionKind::ResultAcknowledged, "result_acknowledged"),
    ];
    for (kind, expected) in cases {
        assert_eq!(
            decision_kind_label(kind),
            *expected,
            "label mismatch for {:?}",
            kind
        );
    }
}

// ---------------------------------------------------------------------------
// decision_detail — None-returning fieldless variants
// ---------------------------------------------------------------------------

#[test]
fn test_decision_detail_none_for_all_fieldless_variants() {
    let fieldless = &[
        BotDecisionKind::ClassConfirmed,
        BotDecisionKind::DraftReady,
        BotDecisionKind::EmptyPlacementFailsafe,
        BotDecisionKind::ResultAcknowledged,
    ];
    for kind in fieldless {
        assert_eq!(
            decision_detail(kind),
            None,
            "expected None detail for {:?}",
            kind
        );
    }
}

// ---------------------------------------------------------------------------
// decision_detail — payload-carrying variants
// ---------------------------------------------------------------------------

#[test]
fn test_decision_detail_class_chosen() {
    let d = decision_detail(&BotDecisionKind::ClassChosen { class_id: ClassId::Sacrier });
    let s = d.expect("ClassChosen must have detail");
    assert!(s.contains("Sacrier"), "detail must mention class: {s}");
}

#[test]
fn test_decision_detail_purchased_format() {
    let d = decision_detail(&BotDecisionKind::Purchased {
        card_id: CardId(77),
        source: CardSource::AuctionWon,
        gold_after: 8,
    });
    let s = d.expect("Purchased must have detail");
    assert!(s.contains("77"), "card id missing: {s}");
    assert!(s.contains("gold_after=8"), "gold_after missing: {s}");
}

#[test]
fn test_decision_detail_refreshed_format() {
    let d = decision_detail(&BotDecisionKind::Refreshed { gold_after: 3 });
    assert_eq!(d, Some("gold_after=3".to_string()));
}

#[test]
fn test_decision_detail_purchase_skipped_format() {
    let d = decision_detail(&BotDecisionKind::PurchaseSkipped { reason: "not_worth" });
    assert_eq!(d, Some("reason=not_worth".to_string()));
}

#[test]
fn test_decision_detail_auction_bid_format() {
    let d = decision_detail(&BotDecisionKind::AuctionBid {
        card_id: CardId(42),
        amount: 4,
        valuation: 5,
    });
    assert_eq!(d, Some("card=42 amt=4 val=5".to_string()));
}

#[test]
fn test_decision_detail_auction_pass_format() {
    let d = decision_detail(&BotDecisionKind::AuctionPass { reason: "below_threshold" });
    assert_eq!(d, Some("reason=below_threshold".to_string()));
}

#[test]
fn test_decision_detail_placement_submitted_format() {
    let d = decision_detail(&BotDecisionKind::PlacementSubmitted { placements_len: 5, coords: vec![] });
    assert_eq!(d, Some("placements_len=5 coords=[]".to_string()));
}

#[test]
fn test_decision_detail_placement_skipped_format() {
    let d = decision_detail(&BotDecisionKind::PlacementSkipped { reason: "empty_hand" });
    assert_eq!(d, Some("reason=empty_hand".to_string()));
}

// ---------------------------------------------------------------------------
// assemble — empty BotPlayers (no bots) → empty payload, no panic
// ---------------------------------------------------------------------------

#[test]
fn test_assemble_empty_bot_players_yields_empty_payload() {
    let bots = BotPlayers::default(); // no bots
    let log = BotDecisionLog::default();
    let submitted = HashSet::new();

    let push = assemble_debug_bot_state_push(
        &bots,
        &log,
        None,
        None,
        None,
        &submitted,
        0,
        DEBUG_BOT_DECISION_TAIL_CAP,
    );

    assert!(push.bots.is_empty(), "zero bots must yield empty bots vec");
    assert_eq!(push.decision_log_total, 0);
    assert_eq!(push.assembled_at_ms, 0);
}

// ---------------------------------------------------------------------------
// Tail-cap boundary tests
// ---------------------------------------------------------------------------

#[test]
fn test_tail_cap_exact_boundary_does_not_trim() {
    let (bot_id, bots) = bot_with_id(10);
    let mut log = BotDecisionLog::default();

    // Push exactly DEBUG_BOT_DECISION_TAIL_CAP entries (16).
    for i in 0..DEBUG_BOT_DECISION_TAIL_CAP as u8 {
        log.push(make_entry(
            bot_id,
            BotDecisionKind::PlacementSubmitted { placements_len: i, coords: vec![] },
        ));
    }
    let submitted = HashSet::new();
    let push = assemble_debug_bot_state_push(
        &bots,
        &log,
        None,
        None,
        None,
        &submitted,
        0,
        DEBUG_BOT_DECISION_TAIL_CAP,
    );

    assert_eq!(
        push.bots[0].decision_tail.len(),
        DEBUG_BOT_DECISION_TAIL_CAP,
        "tail must hold all {} entries when exactly at cap",
        DEBUG_BOT_DECISION_TAIL_CAP
    );
}

#[test]
fn test_tail_cap_below_cap_returns_all_entries() {
    let (bot_id, bots) = bot_with_id(11);
    let mut log = BotDecisionLog::default();

    for _ in 0..3 {
        log.push(make_entry(bot_id, BotDecisionKind::DraftReady));
    }
    let submitted = HashSet::new();
    let push = assemble_debug_bot_state_push(
        &bots,
        &log,
        None,
        None,
        None,
        &submitted,
        0,
        DEBUG_BOT_DECISION_TAIL_CAP,
    );

    assert_eq!(push.bots[0].decision_tail.len(), 3, "all 3 entries must be returned when below cap");
}
