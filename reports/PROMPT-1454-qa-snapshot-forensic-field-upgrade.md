# PROMPT 1454 -- QA Snapshot Forensic Field Upgrade

Final relay line: `1454: QA-SNAPSHOT-FORENSIC-FIELD-UPGRADE: DONE`

## Summary

Upgraded QA snapshot observability from fresh `origin/main@86e50e831befde7e0a4978c93b40556c1383fd77`, after PROMPT 1452 landed. The schema remains additive/backward-compatible and preserves PROMPT 1452 HUD countdown fields:

- `phase_started_elapsed_ms`
- `phase_duration_ms`
- `computed_remaining_ms`
- `display_text`
- `timer_source`
- compatibility fields `duration_ms`, `remaining_ms`, `elapsed_ms`, `active`

## Changed Files

- `client/src/presentation/qa_snapshot.rs`
- `tests/integration/qa_snapshot/layout_field_coverage_test.rs`
- `tests/integration/qa_snapshot/placement_auction_state_field_coverage_test.rs`
- `tests/integration/qa_snapshot/qa_snapshot_overlay_test.rs`

## Added Observability

- Top-level `snapshot_utc_iso`.
- Top-level `evidence_layers` documenting the four evidence layers and explicitly preventing consumers from treating `snapshot.json` as complete game truth.
- Top-level `ui_text_markers[]` from Bevy UI `Text` entities with text, entity/name marker, bounds when computable, font size, visibility, clipped, and overflow pixels.
- `extras.timers.sampled_at_unix_ms` and `sampled_at_utc_iso`.
- `extras.shop_auction.shop.placeholder_visible` and `slots[]` with slot index, entity, card id, name, cost, atk/hp, state, button state, placeholder visibility, and visibility.
- `placement_state.submit_disabled_reason`, `invalid_pending_indices`, `pending_placement_source`, and `last_rejection_state`.
- `extras.input` pointer/drag diagnostics from available drag resources and active board targeting.
- `extras.board.rendered_unit_count`, `visible_rendered_unit_count`, and per-unit lane/cell, visible flag, world position, and source.
- `auction_state.local_player_id`, `leader_is_local`, projected leader label, price label, and timer label.
- `extras.connection_lost` with overlay visibility, disconnected player id, grace remaining, blocking-input flag, and lifecycle reason when available.

## Constraints Honored

- Did not touch `client/src/ui/hand/mod.rs`.
- Did not touch `client/src/ui/shop_auction/mod.rs`.
- Did not touch production sprint/session paperwork.
- Did not run full workspace tests.
- Kept all changes additive; existing fields remain present.

## Verification

Cargo policy applied:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

Commands:

- `cargo test -p client --test qa_snapshot_overlay_test --test qa_snapshot_placement_auction_state_field_coverage_test --test qa_snapshot_layout_field_coverage_test` -- passed.
- `git diff --check` -- passed.

Targeted results:

- `qa_snapshot_layout_field_coverage_test`: 14 passed.
- `qa_snapshot_overlay_test`: 27 passed.
- `qa_snapshot_placement_auction_state_field_coverage_test`: 13 passed.

Notes:

- The first sandboxed Cargo attempt was blocked by access denied on `D:\_DEV\cargo-target\ccgs-msvc\debug\.cargo-lock`; rerun with approved escalation passed.
- Existing deprecation warnings for broad UI markers remain.

## Branch State

- Worktree: `D:\_DEV\Work\Claude-Code-Game-Studios\.codex-worktrees\prompt-1454`
- Branch: `work/qa-snapshot-forensic-field-upgrade-1454`
- Base: `origin/main@86e50e831befde7e0a4978c93b40556c1383fd77`

1454: QA-SNAPSHOT-FORENSIC-FIELD-UPGRADE: DONE
