//! Integration tests for PROMPT 1586 (QA-SNAPSHOT-RESOLUTION-PHASE-FIELDS-FOLLOWUP).
//!
//! Locks the new `extras.resolution_phase` JSON block against the
//! [`client::presentation::qa_snapshot`] schema. The block aggregates per-lane
//! objective damage, gold awards, and unit removal/death events from the
//! locally observable resolution script so forensic snapshot analysis can
//! reconstruct theoretical visible resolution state alongside the captured
//! screenshot.
//!
//! Assertions:
//!  - Schema keys are present even on a defaulted snapshot.
//!  - Defaulted snapshot reports `active = false` + zeroed counts + empty
//!    vectors so the JSON shape stays stable outside the resolution window.
//!  - The pure `build_resolution_phase_snapshot` projection aggregates damage,
//!    gold, deaths, removals, and destroyed flags from a hand-built
//!    `S2CResolutionEvent` script.

use client::presentation::qa_snapshot::{
    build_resolution_phase_snapshot, build_snapshot, ScreenshotInfo, UiCounts,
    QA_SCREENSHOT_FILENAME, QA_SCREENSHOT_FORMAT, SCREENSHOT_STATUS_PENDING,
};
use client::presentation::board_rendering::PendingResolutionScript;

use shared::protocol::{GameOverReason, GoldReason, ResolutionEvent, S2CResolutionEvent, TaggedEvent};
use shared::session::PlayerId;

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
// Schema-presence + defaulted shape.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_default_snapshot_emits_resolution_phase_keys_with_inert_defaults() {
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
    let json = serde_json::to_value(&snapshot).expect("snapshot serialises");
    let block = &json["extras"]["resolution_phase"];
    assert!(block.is_object(), "extras.resolution_phase must be an object");

    for key in [
        "active",
        "round",
        "events_source",
        "event_count",
        "group_count",
        "anim_queue_current_index",
        "last_emitted_group_index",
        "event_summary",
        "per_lane_objective",
        "gold_awards",
        "unit_deaths",
        "unit_removals",
        "pending_objective_destroyed_count",
        "game_over",
        "game_over_reason",
    ] {
        assert!(
            block.get(key).is_some(),
            "extras.resolution_phase.{key} key must be present"
        );
    }

    assert_eq!(block["active"], false, "defaulted snapshot must mark phase inert");
    assert!(block["round"].is_null());
    assert!(block["events_source"].is_null());
    assert_eq!(block["event_count"], 0);
    assert_eq!(block["group_count"], 0);
    assert!(block["anim_queue_current_index"].is_null());
    assert!(block["last_emitted_group_index"].is_null());
    assert_eq!(block["per_lane_objective"], serde_json::json!([]));
    assert_eq!(block["gold_awards"], serde_json::json!([]));
    assert_eq!(block["unit_deaths"], serde_json::json!([]));
    assert_eq!(block["unit_removals"], serde_json::json!([]));
    assert_eq!(block["pending_objective_destroyed_count"], 0);
    assert_eq!(block["game_over"], false);
    assert!(block["game_over_reason"].is_null());

    let summary = &block["event_summary"];
    for key in [
        "sub_step_begin",
        "unit_placed",
        "unit_moved",
        "unit_changed_lane",
        "combat_damage",
        "unit_removed",
        "keyword_triggered",
        "gold_awarded",
        "objective_damage",
        "unit_died",
        "trap_triggered",
        "objective_destroyed",
        "spawn_range_changed",
        "game_over",
    ] {
        assert_eq!(
            summary[key], 0,
            "defaulted event_summary.{key} must be zero"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────
// build_resolution_phase_snapshot — aggregates events from a pending script.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_resolution_phase_snapshot_aggregates_pending_script_events() {
    let local = PlayerId(11);
    let opponent = PlayerId(22);
    let events = vec![
        TaggedEvent {
            sub_step: 0,
            trigger_index: 0,
            event: ResolutionEvent::SubStepBegin,
        },
        TaggedEvent {
            sub_step: 0,
            trigger_index: 1,
            event: ResolutionEvent::CombatDamage {
                attacker_id: 100,
                defender_id: 101,
                damage_amount: 2,
                defender_hp_after: 1,
                was_blocked_by_shield: false,
            },
        },
        TaggedEvent {
            sub_step: 1,
            trigger_index: 0,
            event: ResolutionEvent::ObjectiveDamage {
                attacker_id: Some(100),
                target_player_id: opponent,
                lane: 1,
                damage_amount: 3,
                objective_hp_after: 27,
            },
        },
        TaggedEvent {
            sub_step: 1,
            trigger_index: 1,
            event: ResolutionEvent::ObjectiveDamage {
                attacker_id: Some(102),
                target_player_id: opponent,
                lane: 1,
                damage_amount: 4,
                objective_hp_after: 23,
            },
        },
        TaggedEvent {
            sub_step: 2,
            trigger_index: 0,
            event: ResolutionEvent::UnitDied {
                unit_id: 101,
                lane: 1,
                cell: 5,
                killer_id: Some(100),
            },
        },
        TaggedEvent {
            sub_step: 2,
            trigger_index: 1,
            event: ResolutionEvent::UnitRemoved {
                unit_id: 110,
                lane: 2,
                cell: 3,
            },
        },
        TaggedEvent {
            sub_step: 2,
            trigger_index: 2,
            event: ResolutionEvent::GoldAwarded {
                player: local,
                amount: 2,
                reason: GoldReason::Kill,
            },
        },
        TaggedEvent {
            sub_step: 3,
            trigger_index: 0,
            event: ResolutionEvent::ObjectiveDestroyed {
                target_player_id: opponent,
                lane: 1,
                was_fake: false,
            },
        },
        TaggedEvent {
            sub_step: 3,
            trigger_index: 1,
            event: ResolutionEvent::GameOver {
                loser: Some(opponent),
                reason: GameOverReason::ObjectivesDestroyed,
            },
        },
    ];
    let script = S2CResolutionEvent {
        round: 7,
        events,
    };
    let mut pending = PendingResolutionScript::default();
    pending.set(script);

    let snapshot =
        build_resolution_phase_snapshot(Some(&pending), None, None, None, None);

    // Phase is active and sourced from the pending script.
    assert!(snapshot.active);
    assert_eq!(snapshot.round, Some(7));
    assert_eq!(
        snapshot.events_source.as_deref(),
        Some("pending_resolution_script")
    );
    // 9 tagged events across 4 distinct sub_step groups (0..=3).
    assert_eq!(snapshot.event_count, 9);
    assert_eq!(snapshot.group_count, 4);
    // No live anim queue yet → playback markers absent.
    assert!(snapshot.anim_queue_current_index.is_none());
    assert!(snapshot.last_emitted_group_index.is_none());

    // Per-variant summary.
    let summary = &snapshot.event_summary;
    assert_eq!(summary.sub_step_begin, 1);
    assert_eq!(summary.combat_damage, 1);
    assert_eq!(summary.objective_damage, 2);
    assert_eq!(summary.unit_died, 1);
    assert_eq!(summary.unit_removed, 1);
    assert_eq!(summary.gold_awarded, 1);
    assert_eq!(summary.objective_destroyed, 1);
    assert_eq!(summary.game_over, 1);
    // Variants not present remain zero.
    assert_eq!(summary.unit_placed, 0);
    assert_eq!(summary.unit_moved, 0);
    assert_eq!(summary.unit_changed_lane, 0);
    assert_eq!(summary.keyword_triggered, 0);
    assert_eq!(summary.trap_triggered, 0);
    assert_eq!(summary.spawn_range_changed, 0);

    // Per-lane objective: one entry for (opponent, lane 1) with summed damage
    // and the final hp_after, marked destroyed by the ObjectiveDestroyed event.
    assert_eq!(snapshot.per_lane_objective.len(), 1);
    let lane = &snapshot.per_lane_objective[0];
    assert_eq!(lane.lane, 1);
    assert_eq!(lane.target_player_id, format!("{:?}", opponent));
    assert_eq!(lane.damage_total, 7);
    assert_eq!(lane.hp_after, Some(23));
    assert!(lane.destroyed);
    assert_eq!(lane.was_fake, Some(false));

    // Gold + deaths + removals: each surface their source events.
    assert_eq!(snapshot.gold_awards.len(), 1);
    let gold = &snapshot.gold_awards[0];
    assert_eq!(gold.player, format!("{:?}", local));
    assert_eq!(gold.amount, 2);
    assert_eq!(gold.reason, format!("{:?}", GoldReason::Kill));

    assert_eq!(snapshot.unit_deaths.len(), 1);
    let death = &snapshot.unit_deaths[0];
    assert_eq!(death.unit_id, 101);
    assert_eq!(death.lane, 1);
    assert_eq!(death.cell, 5);
    assert_eq!(death.killer_id, Some(100));

    assert_eq!(snapshot.unit_removals.len(), 1);
    let removal = &snapshot.unit_removals[0];
    assert_eq!(removal.unit_id, 110);
    assert_eq!(removal.lane, 2);
    assert_eq!(removal.cell, 3);
    assert!(removal.killer_id.is_none());

    // GameOver propagated to top-level flag + reason.
    assert!(snapshot.game_over);
    assert_eq!(
        snapshot.game_over_reason.as_deref(),
        Some(format!("{:?}", GameOverReason::ObjectivesDestroyed)).as_deref()
    );

    // No PendingObjectiveDestroyedEvents resource → count stays zero.
    assert_eq!(snapshot.pending_objective_destroyed_count, 0);
}

#[test]
fn test_resolution_phase_snapshot_inert_without_pending_or_queue() {
    let snapshot = build_resolution_phase_snapshot(None, None, None, None, None);
    assert!(!snapshot.active);
    assert!(snapshot.round.is_none());
    assert!(snapshot.events_source.is_none());
    assert_eq!(snapshot.event_count, 0);
    assert_eq!(snapshot.group_count, 0);
    assert!(snapshot.per_lane_objective.is_empty());
    assert!(snapshot.gold_awards.is_empty());
    assert!(snapshot.unit_deaths.is_empty());
    assert!(snapshot.unit_removals.is_empty());
    assert!(!snapshot.game_over);
}
