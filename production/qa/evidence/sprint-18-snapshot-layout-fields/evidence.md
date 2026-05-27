# AC13 Evidence — S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001
# QA Snapshot Layout-Debug Field Enrichment (Q-01..Q-10)

Authored: 2026-05-27 by PROMPT 1714 (story-done paperwork).
Implementation: PROMPT 1186 commit `d75db1afd1d7bb7c7881826a0b4942491120a239`.
Closure source-of-truth: `origin/main@3a4f772113c0a49fbdf37df7bbac51bfd99359ac`.

---

## Null-Emitted Fields and Documented Gaps

Three of the ten Q-* fields have partial or null-emit paths. Per AC13, all three are
documented here with the file:line reference to the in-source limitation record.

### Q-05 — `text.<marker>.fits / clipped_chars`

**Status**: Best-effort, not null — text metrics (text content, Name, bounds, font_px,
overflow_px, and a stable semantic `role` token) are emitted when Bevy UI layout data
exists. Per-glyph clipping detection requires dedicated per-marker UI components that
are outside this story's write scope.

**Source limitation record**:
`client/src/presentation/qa_snapshot.rs:3585–3590`
```
"Q-05 text marker diagnostics are best-effort: text, Name, bounds, font_px, clipped,
overflow_px, and (PROMPT 1533) a stable semantic `role` token derived from Name are
emitted when Bevy UI layout data exists; per-glyph clipping still requires dedicated
UI markers."
```

**Invented values**: None. `null` emitted for `clipped_chars` when per-glyph data
unavailable.

---

### Q-06 — `image.<marker>.aspect_ratio_src / aspect_ratio_rendered`

**Status**: Not computable. Both fields emit `null`.

**Reason**: Computing `aspect_ratio_src` requires reading `Assets<Image>` keyed by the
image handle on each entity. Computing `aspect_ratio_rendered` requires per-image-marker
components (e.g. `CardArtDiagnostic`). Adding such components requires touching
`client/src/ui/*`, which is outside the write scope for this story
(see story-023 § Control Manifest Rules — Forbidden).

**Source limitation record**:
`client/src/presentation/qa_snapshot.rs:3591–3598`
```
"Q-06 image.<marker>.aspect_ratio_src / aspect_ratio_rendered: not computable without
per-image-marker components and an Assets<Image> read; adding markers requires touching
client/src/ui/* (forbidden write scope for this story). See PROMPT 1533 report for
proposed CardArtDiagnostic component + dedicated card-art audit prompt."
```

**Invented values**: None. Both fields emit `null`.

---

### Q-07 — `button.<marker>.affordance_state` (disabled variant)

**Status**: Partial — hover / pressed / normal emitted; `disabled` never emitted.

**Reason**: Bevy 0.18's `Interaction` enum has three variants: `None`, `Hovered`,
`Pressed`. There is no `Disabled` variant; the engine does not encode disabled state
in `Interaction`.

**Source limitation record**:
`client/src/presentation/qa_snapshot.rs:3598–3604`
```
"Q-07 button.<marker>.affordance_state.disabled: Bevy 0.18 Interaction enum has no
Disabled variant; only default / hover / pressed are emitted."
```

**Invented values**: None. `disabled` key omitted from `affordance_state` object; no
fabricated boolean.

---

## Fields With Full Emission (Q-01..Q-04, Q-08..Q-10)

| Field | Status | Notes |
|-------|--------|-------|
| Q-01 `viewport.width_px / height_px` | FULL | `PrimaryWindow` query. `null` only when no primary window entity. |
| Q-02 `surface.<name>.bounds` | FULL | `ComputedNode` + `GlobalTransform` per surface root marker. |
| Q-03 `surface.<name>.overflow_clipped` | FULL | `ComputedNode::overflow` — `true` when clip active. |
| Q-04 `surface.<name>.children_count` | FULL | `Children` component count. |
| Q-08 `panel.<name>.z_layer_resolved` | FULL | `GlobalTransform` z component, rounded to i32. |
| Q-09 `placement_action_panel.collisions` | FULL | Bounds-intersection across all surface roots. |
| Q-10 `shop_panel.bottom_edge_y / hand_bar.top_edge_y` | FULL | Derived from Q-02 bounds. |

---

## Test Coverage

`tests/integration/qa_snapshot/layout_field_coverage_test.rs` (NEW, 682 lines, 14 `#[test]`
declarations) exercises: viewport presence, per-surface structure, children_count,
overflow_clipped, z_layer_resolved, button_affordances, collisions, shop/hand y-edges,
text marker presence, limitations vector non-empty, and null-emit paths for Q-06.

All 14 tests passed at commit `d75db1afd1d7bb7c7881826a0b4942491120a239`
(PROMPT 1186 worker run evidence, per `reports/PROMPT-1186-S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001.md`).
