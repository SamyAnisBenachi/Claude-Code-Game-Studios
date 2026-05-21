# PROMPT 1586 — QA-SNAPSHOT-RESOLUTION-PHASE-FIELDS-FOLLOWUP

- Source-of-truth base: `origin/main@9be8827fbd22b2a49d973ba585b5d210fdc8a903`
- Branch: `work/qa-snapshot-resolution-phase-fields-1586`
- Worktree: `D:/Tmp/wt-1586`
- Owned scope: `client/src/presentation/qa_snapshot.rs`, a focused integration
  test under `tests/integration/qa_snapshot/`, and the matching `[[test]]`
  registration in `client/Cargo.toml`.

## Goal

Close the resolution-phase observability gap surfaced by PROMPT 1478 / PROMPT
1533: QA snapshot JSON had no fields describing the in-flight resolution
script, so forensic consumers could not reconstruct theoretical visible
resolution state (per-lane damage, objective hp deltas, gold awards, unit
deaths/removals) alongside the captured screenshot.

This is a pure data-side projection. Render and playback behaviour stay
exactly the same — `apply_resolution_replay_group_system` remains the sole
writer of visual mutations.

## Closed-this-prompt

### `extras.resolution_phase` JSON block (new)

A new `ResolutionPhaseSnapshot` field is added under `ExtrasSnapshot`,
populated by a new `build_resolution_phase_snapshot` pure projection:

| Field | Type | Source |
|---|---|---|
| `active` | `bool` | `true` when `PendingResolutionScript` is loaded, `AnimQueue` carries resolution groups, `BoardRenderState == ResolutionExecuting`, or `PendingObjectiveDestroyedEvents` is non-empty. |
| `round` | `Option<u32>` | `S2CResolutionEvent::round` from the pending script. `None` once the queue takes over (script round is not stored on `AnimGroup`). |
| `events_source` | `Option<String>` | `"pending_resolution_script"` pre-consume, `"anim_queue"` post-consume, `None` otherwise. |
| `event_count` | `usize` | Total `TaggedEvent` count across all observable groups. |
| `group_count` | `usize` | `AnimQueue::groups.len()` post-consume, or distinct `sub_step` count pre-consume. |
| `anim_queue_current_index` | `Option<usize>` | `AnimQueue::current_index` while playback is running. |
| `last_emitted_group_index` | `Option<usize>` | `ResolutionReplayProgress::last_emitted_group_index()` (PROMPT 1521 marker). |
| `event_summary` | `ResolutionEventSummary` | One `usize` per `ResolutionEvent` variant — phase-resolution event summary. |
| `per_lane_objective` | `Vec<LaneObjectiveResolutionSnapshot>` | Per-`(target_player_id, lane)` aggregate: `damage_total`, `hp_after` (last `objective_hp_after`), `destroyed`, `was_fake`. |
| `gold_awards` | `Vec<GoldAwardSnapshot>` | Every `GoldAwarded` event surfaced as `{player, amount, reason}` (prism/gold delta forensic trail). |
| `unit_deaths` | `Vec<UnitRemovalSnapshot>` | Every `UnitDied` event with `killer_id`. |
| `unit_removals` | `Vec<UnitRemovalSnapshot>` | Every `UnitRemoved` event (no killer attribution). |
| `pending_objective_destroyed_count` | `usize` | `PendingObjectiveDestroyedEvents::len()` — count of objectives staged for the post-resolution destroyed reveal animation. |
| `game_over` | `bool` | `true` when the script contains a `GameOver` event. |
| `game_over_reason` | `Option<String>` | `GameOverReason` Debug string when `game_over == true`. |

### Resource wiring

A new `ExtrasResolutionInputs` `SystemParam` groups the four read-only
resource accesses:

- `Option<Res<PendingResolutionScript>>`
- `Option<Res<ResolutionReplayProgress>>`
- `Option<Res<AnimQueue>>`
- `Option<Res<PendingObjectiveDestroyedEvents>>`

It is added as a single field on `ExtrasInputs`, taking the 16-field
`SystemParam` ceiling from 15 → 16. (Tested by `cargo check -p client`
succeeding without a SystemParam-overflow E0277 — Bevy's compile-time
ceiling.) Every resource is `Option`-wrapped so missing resources during
lobby / pre-handshake / between-rounds capture project to a defaulted
`ResolutionPhaseSnapshot` rather than panicking.

### Source-precedence contract

`AnimQueue` takes precedence over `PendingResolutionScript` for the
`events_*` aggregates. In practice they cannot both carry events
simultaneously — `consume_pending_resolution_script_system` takes the
script out of `PendingResolutionScript` before flattening into
`AnimGroup`s — but the explicit precedence documents the contract for
future audits. Round number is only derivable from the pending script,
so it becomes `None` once playback starts (acceptable: `last_emitted_group_index` +
`anim_queue_current_index` already carry the live playback marker).

### Shape stability

All sub-fields default to inert values outside the resolution window
(`active=false`, empty vectors, `None` markers, zeroed `event_summary`
counts). The defaulted snapshot test
(`test_default_snapshot_emits_resolution_phase_keys_with_inert_defaults`)
asserts every documented key is present in the JSON shape even on a
defaulted (no-resources) capture.

## Behavioral effects

- Snapshot JSON shape is **schema-additive**. Existing fields keep their
  contracts unchanged.
- New fields are pure read-only projections — no protocol mutation, no
  network traffic, no gameplay state mutation, no rendering work.
- The new `ExtrasResolutionInputs` `SystemParam` adds three new
  `Option<Res<_>>` reads at the qa_snapshot system's call site (the four
  resource types are already registered as `init_resource` by their
  owning plugins).

## Test evidence

New test file: `tests/integration/qa_snapshot/resolution_phase_field_coverage_test.rs`.

Registered as `[[test]] qa_snapshot_resolution_phase_field_coverage_test`
in `client/Cargo.toml`.

| Test | Asserts |
|---|---|
| `test_default_snapshot_emits_resolution_phase_keys_with_inert_defaults` | Every documented key under `extras.resolution_phase` is present on a defaulted snapshot, with inert values (`active=false`, zeroed counts, empty vectors, null markers). Locks the JSON shape. |
| `test_resolution_phase_snapshot_aggregates_pending_script_events` | A hand-built 9-event `S2CResolutionEvent` aggregates correctly: per-variant `event_summary`, per-`(player, lane)` objective damage sum + hp_after + destroyed flag, gold award projection, unit death + unit removal projection, GameOver propagation. |
| `test_resolution_phase_snapshot_inert_without_pending_or_queue` | Builder returns inert snapshot when no resource is provided. |

Run output:

```
running 3 tests
test test_resolution_phase_snapshot_inert_without_pending_or_queue ... ok
test test_resolution_phase_snapshot_aggregates_pending_script_events ... ok
test test_default_snapshot_emits_resolution_phase_keys_with_inert_defaults ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Sibling regression check:
`cargo test --test qa_snapshot_placement_auction_state_field_coverage_test`
remains green (15 passed; 0 failed).

Validation level (per prompt rules): path-allowlist + `git diff --check` +
focused snapshot test. Broader `cargo test` deferred to VERIFY lane.

## Files changed

- `client/src/presentation/qa_snapshot.rs`
  - Imports: `AnimQueue`, `AnimQueueEvent`, `PendingObjectiveDestroyedEvents`,
    `PendingResolutionScript`, `ResolutionReplayProgress`.
  - New structs: `ResolutionPhaseSnapshot`, `ResolutionEventSummary`,
    `LaneObjectiveResolutionSnapshot`, `GoldAwardSnapshot`,
    `UnitRemovalSnapshot`.
  - `ExtrasSnapshot.resolution_phase` field added.
  - `ExtrasInputs.resolution` field added.
  - New `ExtrasResolutionInputs` `SystemParam`.
  - `ExtrasInputs::snapshot_with_warnings` wires
    `build_resolution_phase_snapshot` into the snapshot construction.
  - New `build_resolution_phase_snapshot` pure projection.
- `tests/integration/qa_snapshot/resolution_phase_field_coverage_test.rs` — new.
- `client/Cargo.toml` — `[[test]]` registration for the new integration
  test binary.

## Out of scope (filed as follow-up notes)

| Gap | Reason / proposed follow-up |
|---|---|
| Live anim-queue round | `AnimGroup` does not carry the `S2CResolutionEvent::round` after `consume_pending_resolution_script_system` flattens the script. To surface it during playback, either (a) propagate the round into `AnimGroup` (UI write scope), or (b) cache it on a small client-side resource captured at consume time. Filed as a separate follow-up. |
| Resolution recovery signal | `PendingResolutionScript::recovery_requested` and `ResolutionRevealWait::recovery_requested` are private (no accessor). Surfacing the recovery flag requires a small public accessor on those resources; out of this prompt's owned scope. |
| Per-card-art aspect-ratio diagnostics (Q-06) | Still requires UI-write-scope marker insertion — same status as PROMPT 1533. |

## Path-allowlist review

| File | Allowed? |
|---|---|
| `client/src/presentation/qa_snapshot.rs` | ✅ owned scope |
| `tests/integration/qa_snapshot/resolution_phase_field_coverage_test.rs` | ✅ owned scope |
| `client/Cargo.toml` | ✅ test-registration only — `[[test]]` block follows the existing PROMPT 1186 / 1229 pattern (`qa_snapshot_layout_field_coverage_test`, `qa_snapshot_placement_auction_state_field_coverage_test`). Strictly necessary to make the new integration test discoverable. |

No edits to forbidden surfaces (`client/src/ui/hand/mod.rs`,
`client/src/presentation/board_rendering.rs`, `client/src/ui/shop_auction/mod.rs`,
`server/**`, `shared/**`, `production/**`, CI files, unrelated Cargo files).

1586: QA-SNAPSHOT-RESOLUTION-PHASE-FIELDS-FOLLOWUP: SHIPPED
