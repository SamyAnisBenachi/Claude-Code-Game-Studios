# S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001 — AC13 Limitation Notes

> **Story**: S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001 (ui-clean-pass story 023)
> **AC13 requirement**: No invented values; evidence note lists every null-emitted field + missing-query file:line.
> **Authored**: 2026-05-28 by PROMPT 2019 (paperwork-only /story-done).
> **Implementation source**: PROMPT 1186 (commit `d75db1af`) — primary Q-01..Q-10 landing.
>   PROMPT 1533 (commit `03342873`) — Q-05 partial closure (semantic `role` token on text markers).

---

## Emitted Fields (non-null on active scene)

All Q-* fields below are emitted on `LayoutSnapshot` when the corresponding
ECS data is present. `null` is the schema-stable absence signal; no field is
ever omitted entirely.

| Q# | JSON path | Emitted | Notes |
|---|---|---|---|
| Q-01 | `layout.viewport.{width_px,height_px,ui_scale,window_scale_factor}` | Always | Sourced from `PrimaryWindow` + `WindowResolution`. |
| Q-02 | `layout.surfaces[].bounds = {x,y,w,h}` | When marker spawned | `SurfaceBoundsRect` in logical px from `ComputedNode` + `GlobalTransform`. `null` when marker entity absent. |
| Q-03 | `layout.surfaces[].overflow_clipped` | When marker spawned | `true` when `ComputedNode::content_size` exceeds `ComputedNode::size`. `null` when marker absent. |
| Q-04 | `layout.surfaces[].children_count` | When marker spawned | Direct child count via `Children` component. `null` when marker absent. |
| Q-07 | `layout.button_affordances[].{entity,name,interaction}` | Per button entity | `interaction` is `default / hover / pressed` (Bevy 0.18 `Interaction` enum values). |
| Q-08 | `layout.surfaces[].z_layer_resolved` | When `GlobalZIndex` present | `null` when no `GlobalZIndex` component on marker entity. |
| Q-09 | `layout.collisions.placement_action_panel_overlaps` | Always | List of surface names whose bounds intersect `placement_action_panel`. Empty list `[]` when no collision. |
| Q-10 | `layout.collisions.{shop_panel_bottom_edge_y,hand_bar_top_edge_y,shop_panel_vs_hand_bar_overlap_px}` | When bounds present | `null` per sub-field when the named surface is absent or has no bounds. |

---

## Null-Emitted / Best-Effort Fields (documented gaps per AC13)

### Q-05 — Text fit + clipped_chars

**Status**: Best-effort partial coverage (Q-05 partially closed by PROMPT 1533).

**What is emitted** (`layout.ui_text_markers[]`): `text`, `Name`, `bounds`,
`font_px`, `clipped`, `overflow_px`, and a stable semantic `role` token
derived from the entity Name. These fields are populated from Bevy UI layout
data when present.

**What is NOT emitted**: Per-glyph clipping count (`clipped_chars` from the
original Q-05 spec). This requires dedicated text-layout inspection beyond
what `ComputedNode` exposes; Bevy 0.18 does not provide per-glyph clipping
metadata through the standard query API.

**Missing query file:line**:
- `client/src/presentation/qa_snapshot.rs:3585–3590` — limitation entry
  (best-effort note, per-glyph clipping gap)
- Root cause: `ComputedNode` / `TextLayout` do not expose per-glyph overflow
  in Bevy 0.18; computing it would require a dedicated `TextMarker` component
  on each text entity (outside owned write scope for PROMPT 1186).

### Q-06 — Image aspect ratios

**Status**: Not emitted. Null-emitted (no `image.*` key in JSON).

**What is emitted**: Nothing. `aspect_ratio_src` and `aspect_ratio_rendered`
are absent from the snapshot.

**Why null**: Computing per-marker image aspect ratios requires:
1. A per-image-marker component (e.g., `CardArtDiagnostic`) to associate
   an `AssetId<Image>` with a UI node.
2. An `Assets<Image>` system-param read in `write_qa_snapshot_system`.
3. Writing to `client/src/ui/*` surface files to attach the marker component.
   Steps 1 and 3 are **forbidden write scope** for S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001.

**Missing query file:line**:
- `client/src/presentation/qa_snapshot.rs:3591–3597` — limitation entry
  (not computable, forbidden write scope, proposed follow-on via
  `CardArtDiagnostic` component per PROMPT 1533 report)

### Q-07 — Button affordance_state.disabled

**Status**: Best-effort partial coverage.

**What is emitted**: `interaction` field with values `default / hover / pressed`.

**What is NOT emitted**: `disabled` state. Bevy 0.18 `Interaction` enum has
no `Disabled` variant. Button disabled state would require a separate
application-level component (e.g., `ButtonDisabled` marker) outside the owned
write scope.

**Missing query file:line**:
- `client/src/presentation/qa_snapshot.rs:3598–3601` — limitation entry
  (no Disabled variant in Bevy 0.18 Interaction enum)

---

## Test Evidence

- `tests/integration/qa_snapshot/layout_field_coverage_test.rs` — 14 tests
  (PROMPT 1186 `d75db1af`). Schema-focused: asserts every Q-* key is present
  in JSON (null or value, never missing), exercises `build_layout_collisions`
  pure helper with synthetic surface bounds, locks canonical surface-name set.
  Tests pass: `cargo test -p client --test qa_snapshot_layout_field_coverage_test`
  → 14/14 PASS (verified at PROMPT 1186 commit).

---

## Non-claims

- Q-05 per-glyph `clipped_chars` is a known gap; not claimed as closed.
- Q-06 image aspect ratios are not implemented; not claimed as closed.
- 24 PROMPT 1022 QA snapshot audit findings are preserved as report-only;
  this evidence file does NOT claim to close any of them.
- No accept-risk closure; limitations are honestly documented above.
