# PROMPT 1533 — QA-SNAPSHOT-OBSERVABILITY-FIELDS-FOLLOWUP

- Source-of-truth base: `origin/main@5358aed1a6075aca621936fd14f561be8fb854d3`
- Branch: `prompt-1533-qa-snapshot-observability-fields-followup`
- Worktree: `D:/tmp/wt-1533-qasnap-fields`
- Owned scope: `client/src/presentation/qa_snapshot.rs`, direct tests under
  `tests/integration/qa_snapshot/`, and this report.

## Goal

Close the highest-value observability gaps from PROMPT 1478 that are still
feasible without protocol changes:
1. Explicit placement ACK / rejection lifecycle fields if locally observable.
2. Pointer / focus / hit-test provenance fields if still incomplete.
3. Semantic rendered-label roles for important UI labels.
4. Card-art / image diagnostic metadata where useful for Krosmaga / dev-proxy
   analysis.

For each item, if it cannot be reconstructed from local client state, document
the exact missing authoritative state and propose a separate
protocol / server prompt.

## Closed-this-prompt

### 1. Placement ACK lifecycle fields (locally observable)

`PlacementLifecycleSnapshot` (the existing PROMPT 1486 block) gained two
diagnostic fields plus a refined `state` derivation:

| Field | Type | Derivation |
|---|---|---|
| `accepted` (existing, now populated) | `Option<bool>` | `Some(true)` when `submitted == Some(true) && staged_count == Some(0) && last_rejection_reason.is_none() && !placement_drag_active && !ghost_unstage_active`. Otherwise `None`. Tri-state contract preserved — rejection is still surfaced via the dedicated `rejected` field, so legacy tooling that ignored `accepted` continues to work. |
| `accepted_source` (new) | `Option<String>` | Stable token explaining how `accepted` was inferred. Today: `"local_clearance_heuristic"`. Reserved namespace for a future authoritative server signal (see proposal below). |
| `awaiting_ack` (new) | `Option<bool>` | `true` between `submitted==true` and an observable ACK / rejection. Mirrors the post-submit window where the local client has fired `SubmitPlacements` but neither cleared the staged buffer nor seen a `Correction(*)` disclosure. |
| `state` (existing, extended) | `Option<String>` | New terminal token `"accepted"` added between `"submitted"` and the existing `"rejected"`. |

The heuristic is intentionally conservative (`accepted` only flips to
`Some(true)` when every observable post-ACK condition holds). Auditors who
need higher confidence can cross-reference `accepted_source` and combine
with the per-frame `board.units` populated count.

#### Authoritative gap

The server presently does not emit a client-visible per-placement
`PlacementAccepted{placement_id}` confirmation distinct from the regular
S2C state delta. A separate prompt should propose adding either:

- `S2CPlacementAck { round, accepted_ids: Vec<u32>, rejected: Vec<(u32, RejectReason)> }`
  emitted between the placement submit and the resolution snapshot, or
- An explicit `PlacementLifecycle` resource on the client populated by the
  existing resolution snapshot, with the snapshot writer reading the
  resource instead of inferring from `submitted + staged_count`.

Either path would let `accepted_source` advance from
`local_clearance_heuristic` to `server_ack`. Filed as a follow-up proposal;
no protocol mutation is in scope here.

### 2. Pointer / focus / hit-test provenance

`InputDiagnosticsSnapshot.hovered_entity` was previously hard-coded to
`None` (PROMPT 1486 deferred it because hover tracking lived only inside
`LayoutInputs::interactions`). PROMPT 1533 lifts that signal by projecting
the already-collected `LayoutSnapshot.button_affordances`: when any UI
`Button` carries `Interaction::Hovered`, its stringified entity id flows
into `extras.input.hovered_entity` and its `Name` (when present) is
emitted as `extras.input.focused_semantic_target` as `button:<name>`
(without overriding the auction bid-keyboard-focus signal, which keeps
priority).

This requires no new ECS query and no extra `SystemParam` fields — the
projection runs inside `build_snapshot_with_extras_and_layout` against
the existing `LayoutSnapshot.button_affordances` vector.

#### Authoritative gap

Bevy 0.18 `Interaction` has no `Disabled` variant, so the existing Q-07
limitation around `affordance_state.disabled` remains. Picking-system
hover for non-UI entities (board cells, draggable cards) is already
surfaced through `last_hit_test_source` + `target_cell` and was not
expanded in this prompt to avoid touching the picking pipeline.

### 3. Semantic rendered-label roles

`UiTextMarkerSnapshot` gained an `Option<String>` `role` field populated by
the new `classify_ui_text_marker_role` mapping. The classifier maps the
`Name` component of important rendered labels onto stable role tokens
(`hud.gold_counter`, `hud.phase_timer`, `placement.submit_label`,
`auction.bid_input`, `lobby.confirm_label`, …) plus a small set of
prefix-based families (`hand.card_label`, `shop.slot_price_label`,
`auction.bid_button_label`).

Audits and dev-proxy scripts can grep on `role` rather than chasing raw
`Name` strings whose casing/format drifts every UI refactor. The
classifier lives in `qa_snapshot.rs` so adding new role mappings does not
require touching the UI module write scope.

The `Q-05 limitations` line in `LayoutSnapshot.limitations` was updated to
record this partial closure and explicitly note that per-glyph clipping
remains a UI-write-scope item.

### 4. Card-art / image diagnostic metadata

Not closeable in-scope. The Q-06 limitation
(`image.<marker>.aspect_ratio_src / aspect_ratio_rendered`) requires
per-image markers on every card-art `ImageNode` plus an `Assets<Image>`
read for the source dimensions, neither of which can be added without
editing `client/src/ui/**` — forbidden by the PROMPT 1533 owned scope.

#### Proposed follow-up prompt

> Add a `CardArtDiagnostic` marker component (or extend the existing card
> slot markers under `client/src/ui/hand`, `client/src/ui/shop_auction`,
> and the Krosmaga card-inspect primitive landed in PROMPT 1482) plus a
> small `card_art` block under `LayoutSnapshot` carrying
> `Vec<CardArtSnapshot { marker, src_size: (u32,u32), rendered_size:
> (f32,f32), aspect_ratio_src, aspect_ratio_rendered, asset_handle:
> Option<String>, missing_image: bool }>`. Owned scope must include the UI
> modules so the marker can be inserted at spawn sites. Dependency:
> requires an `Assets<Image>` read inside the snapshot system, so the
> 16-field `SystemParam` ceiling needs re-checking when wiring it in.

The Q-06 limitation string was updated to point at this report.

## Behavioral effects

- Snapshot JSON shape is **schema-additive**. Existing fields keep their
  contracts (`placement_lifecycle.accepted` is still `null` in every
  scenario the legacy test fixtures construct, including the rejection
  fixture in `placement_auction_state_field_coverage_test.rs`).
- New fields default to `null` outside the placement / hover windows that
  populate them, keeping JSON shape stable across phases.
- No gameplay paths or server protocol code touched. No new dependencies.

## Test evidence

Edits under `tests/integration/qa_snapshot/placement_auction_state_field_coverage_test.rs`:

- `test_snapshot_extras_include_pointer_lifecycle_and_debug_grid_blocks`
  extended to assert `awaiting_ack == false` and `accepted_source == null`
  in the existing rejection scenario.
- New test `test_snapshot_placement_lifecycle_accepted_from_local_clearance`
  exercises the local-clearance ACK heuristic (submitted + staged drained
  + no rejection → `accepted=true`, `state="accepted"`,
  `accepted_source="local_clearance_heuristic"`, `awaiting_ack=false`).

Validation level (per prompt rules): path-allowlist + `git diff --check` +
focused snapshot tests. Broader `cargo test` not run (and a pre-existing
`hud_phase_transitions_test` E0063 in another worker's in-flight branch
would block such a run anyway; not owned by this prompt).

## Files changed

- `client/src/presentation/qa_snapshot.rs`
  - `PlacementLifecycleSnapshot` — added `awaiting_ack`, `accepted_source`.
  - `build_placement_lifecycle_snapshot` — new derivation logic + state
    token "accepted".
  - `UiTextMarkerSnapshot` — added `role` field.
  - `build_ui_text_marker_snapshots` — populates `role` via classifier.
  - `classify_ui_text_marker_role` — new pub fn (name → semantic role
    token).
  - `InputDiagnosticsSnapshot.hovered_entity` — now populated from
    `LayoutSnapshot.button_affordances` instead of hard-coded None.
  - `build_input_diagnostics_snapshot` — extra `hovered_entity` parameter.
  - `build_snapshot_with_extras_and_layout` — projects hovered button +
    label as `focused_semantic_target` fallback.
  - `LayoutSnapshot.limitations` — Q-05 + Q-06 strings updated to reflect
    partial closure + Q-06 protocol-proposal pointer.

- `tests/integration/qa_snapshot/placement_auction_state_field_coverage_test.rs`
  - Extended rejection-scenario assertions with `awaiting_ack` /
    `accepted_source`.
  - Added `test_snapshot_placement_lifecycle_accepted_from_local_clearance`.

## Out of scope (filed as proposals)

| Gap | Proposed follow-up |
|---|---|
| Authoritative placement ACK signal | Server prompt adding `S2CPlacementAck` or a `PlacementLifecycle` resource; lets `accepted_source` advance to `server_ack`. |
| Per-card-art aspect-ratio diagnostics (Q-06) | UI prompt adding `CardArtDiagnostic` markers + `LayoutSnapshot.card_art` block. |
| Per-glyph clipping diagnostics (Q-05) | UI prompt adding text-clip markers + dedicated glyph layout probes. |
| Picking-pipeline non-UI hover | Picking-pipeline prompt projecting non-UI hover entities (board cells, draggable cards) through a dedicated resource. |

1533: QA-SNAPSHOT-OBSERVABILITY-FIELDS-FOLLOWUP: SHIPPED
