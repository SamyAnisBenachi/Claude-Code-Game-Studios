# Sprint 17 -- S17-UI-CARD-SLOT-INSET-WIRING-001 Evidence

> **Story**: `production/epics/ui-clean-pass/story-018-card-slot-inset-wiring.md`
> **Source audit**: SOURCE-1077-06 (P1) -- `reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md`
> **PROMPT**: 1102 (`/dev-story` worker)
> **Worker branch**: `work/s17-card-slot-inset-wiring`
> **Worker worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s17-card-slot-inset-wiring`
> **Source-of-truth at activation**: `origin/main@ff47075` (PROMPT 1100 Sprint 17 QA plan tip)
> **Story type**: Logic (per `.claude/docs/coding-standards.md` "Test Evidence by Story Type" matrix -- BLOCKING gate satisfied by automated unit + integration tests)
> **Skills active**: `liv-bevy-018` (every `.rs` edit). `liv-bevy-lightyear` NOT used.

---

## Shape extension strategy

Per the story's "Worker MUST: choose the shape extension strategy" clause,
this worker selected **option (b): add two new sibling builder functions**:

- `card_slot_image_inset_node(kind: CardSlotKind) -> (Node, GlobalZIndex)`
- `card_slot_text_inset_node(kind: CardSlotKind) -> (Node, GlobalZIndex)`

### Why not extend `card_slot_node` directly?

Extending `card_slot_node` to return a multi-node struct or tuple would
break the existing Sprint 16 PROMPT 1067 shop-slot Phase 1 migration
call site (`client/src/ui/shop_auction/mod.rs::shop_slot_node`) which
spawns the returned `Node` directly. The story's AC5 binds existing
PROMPT 1067/1074 shop-slot behaviour to remain green; the net-additive
sibling-builder shape is the minimal change that satisfies AC5 + AC6
simultaneously.

### Why a `(Node, GlobalZIndex)` tuple?

The story's AC1 / AC2 require the new builders to "thread the `z_layer`
via `GlobalZIndex`". `GlobalZIndex` is a separate Bevy 0.18 component
(not a field on `Node`), so the builder returns a `(Node, GlobalZIndex)`
bundle that consumers can pass directly to `Commands::spawn` /
`ChildBuilder::spawn` -- the per-surface migration siblings
(`S17-UI-CARD-SLOT-MIGRATION-*` Backlog family) reduce to a thin
`parent.spawn(card_slot_image_inset_node(kind))` re-author instead of
re-authoring child-positioning arithmetic per consumer site (the
defect-class prevention target stated in the story `Problem Class /
Prevention Target` section).

### Why no `padding` field?

`card_slot_geometry(kind)` does NOT currently expose a padding rectangle
(only `image_inset_px`, `text_inset_px`, `hit_target_inset_px`). Per
the story's AC4 wording -- "IF the catalog does not currently expose
padding, this AC is satisfied trivially by a doc-comment noting the
catalog does not expose padding and the new builder emits no `padding`
field." -- this worker added a doc-comment to `card_slot_node` and
emits no `Node.padding` field. A future revision that promotes padding
into `CardSlotGeometry` is out of scope (AC8 forbids retuning the
geometry catalog).

---

## Files changed

| Path | Diff |
|------|------|
| `client/src/ui/design_tokens/card_slot.rs` | +204 lines (two new public builders + doc comment on `card_slot_node` + three new `#[cfg(test)] mod tests` entries). No retune of `card_slot_geometry` constants (AC8). `card_slot_node` body unchanged (AC5). |
| `tests/integration/ui_clean_pass/card_slot_primitive_test.rs` | +243 / -3 lines (8 new `s17_*` integration tests + import additions). Existing AC1..AC8 assertions unchanged (AC5). |

Neither `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`,
`docs/architecture/adr-*.md`, `production/sprint-status.yaml`,
`production/sprints/*`, `production/stage.txt`,
`production/session-state/*`, `production/qa/qa-plan-*.md`,
`production/qa/smoke-*.md`, `production/qa/team-qa-*.md`,
`production/gate-checks/*`, `server/`, `shared/`, or any consumer
surface (`client/src/ui/hand/`, `client/src/ui/shop_auction/*`,
`client/src/presentation/board_rendering.rs`) was touched (AC7, AC10,
AC12, AC13 + prompt-level owned-files contract).

---

## Acceptance Criteria evidence

### AC1 -- Primitive exposes per-kind image inset

`card_slot_image_inset_node(kind)` returns `(Node, GlobalZIndex)`:

- `Node.position_type == PositionType::Absolute`
- `Node.{left, right, top, bottom}` set from
  `card_slot_geometry(kind).image_inset_px`
- `GlobalZIndex` set from `card_slot_geometry(kind).z_layer`

Asserted by inline test
`ui::design_tokens::card_slot::tests::s17_image_inset_node_matches_geometry_per_kind`
and integration test
`s17_inset_image_node_edges_match_geometry_per_kind` + 
`s17_inset_image_node_position_type_absolute_per_kind`.

### AC2 -- Primitive exposes per-kind text inset

`card_slot_text_inset_node(kind)` returns `(Node, GlobalZIndex)`:

- `Node.position_type == PositionType::Absolute`
- `Node.{left, right, top, bottom}` set from
  `card_slot_geometry(kind).text_inset_px`
- `GlobalZIndex` set from `card_slot_geometry(kind).z_layer`

Asserted by inline test
`ui::design_tokens::card_slot::tests::s17_text_inset_node_matches_geometry_per_kind`
and integration test
`s17_inset_text_node_edges_match_geometry_per_kind` + 
`s17_inset_text_node_position_type_absolute_per_kind`.

### AC3 -- `GlobalZIndex` wired from geometry

Both builders emit `GlobalZIndex(geometry.z_layer.0)` for every
`CardSlotKind` variant; values come from the same `card_slot_geometry`
source. Asserted by inline test
`s17_inset_builders_thread_global_z_index_from_geometry_per_kind` and
integration test
`s17_inset_image_and_text_builders_thread_global_z_index_per_kind`.

The variant-by-variant `z_layer` constants match the existing geometry
catalog: `UI_BASE (300)` for `HandFan` / `DraftGrid` / `ShopSlot` /
`AuctionFeatured`; `UI_OVERLAY (400)` for `BoardStagedGhost`.

### AC4 -- Padding wired from geometry (if exposed by the catalog)

`card_slot_geometry(kind)` does NOT expose a padding rectangle.
Per AC4 fallback clause: doc-comment on `card_slot_node` notes the
catalog does not expose padding; new builders emit no `Node.padding`
field. AC satisfied trivially.

### AC5 -- Existing PROMPT 1067 shop-slot Phase 1 migration remains green

`cargo test -p client --test ui_clean_pass_card_slot_primitive_test`
output (full 27 tests):

```
running 27 tests
... [existing AC1..AC8 tests] ...
test ac5_phase_1_shop_slot_node_outer_geometry_matches_primitive ... ok
test ac7_card_slot_node_width_height_match_geometry_for_every_kind ... ok
test ac7_card_slot_node_width_height_match_geometry_for_shop_slot ... ok
...
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured;
```

Every Sprint 16 story 009 closed assertion remains PASS.

### AC6 -- New tests assert inset / z-index wiring

Eight new `s17_*` tests in
`tests/integration/ui_clean_pass/card_slot_primitive_test.rs` plus
three new `s17_*` tests in the `#[cfg(test)] mod tests` block of
`client/src/ui/design_tokens/card_slot.rs`:

- (a) per-variant image-inset Node `left/right/top/bottom` precise
  pixel match: `s17_inset_image_node_edges_match_geometry_per_kind`
  + inline `s17_image_inset_node_matches_geometry_per_kind`.
- (b) per-variant text-inset Node `left/right/top/bottom` precise
  pixel match: `s17_inset_text_node_edges_match_geometry_per_kind`
  + inline `s17_text_inset_node_matches_geometry_per_kind`.
- (c) per-variant `GlobalZIndex` equals
  `card_slot_geometry(kind).z_layer`:
  `s17_inset_image_and_text_builders_thread_global_z_index_per_kind`
  + inline
  `s17_inset_builders_thread_global_z_index_from_geometry_per_kind`.
- (d) variant set coverage:
  `s17_inset_builders_cover_every_card_slot_kind_variant` asserts
  `ALL_CARD_SLOT_KINDS.len() == 5` and iterates every variant.

Additional defensive guards (catch a future regression of
SOURCE-1077-06's defect class):

- `s17_inset_image_node_carries_no_inline_size_overrides_per_kind`:
  Node.width / Node.height MUST remain `Val::Auto` so a future
  revision cannot silently re-introduce inline `Val::Px(N)` width /
  height overrides.
- `s17_inset_text_node_carries_no_inline_size_overrides_per_kind`:
  the text-counterpart guard.
- `s17_inset_builders_dimensions_resolve_to_positive_interior_per_kind`:
  asserts the inset rectangles, when subtracted from the outer
  rectangle, leave a strictly-positive interior for every kind. A
  primitive whose inset over-constrains the outer rectangle would
  fail here.

### AC7 -- No consumer surface migrated

`git diff origin/main..HEAD` for `client/src/ui/hand/`,
`client/src/ui/shop_auction/auction_*`,
`client/src/ui/shop_auction/draft_*`,
`client/src/presentation/board_rendering.rs`:

```text
(no output)
```

Zero changes under any consumer surface.

### AC8 -- No `card_slot_geometry` constant change

`git diff origin/main..HEAD client/src/ui/design_tokens/card_slot.rs`
shows the body of `card_slot_geometry(kind)` (lines 502-567 of the
pre-change file) UNCHANGED. The diff adds new builder functions
(`card_slot_image_inset_node`, `card_slot_text_inset_node`) AFTER the
existing `card_slot_node` builder and adds three new `#[cfg(test)]`
tests. The 14 named per-kind constants
(`CARD_SLOT_HAND_FAN_*`, `CARD_SLOT_DRAFT_GRID_*`,
`CARD_SLOT_SHOP_SLOT_*`, `CARD_SLOT_AUCTION_FEATURED_*`,
`CARD_SLOT_BOARD_GHOST_*`) are UNCHANGED.

### AC9 -- ADR-021 schedule preserved

`cargo check -p client` succeeded under the Cargo resource policy.
No `App::add_systems` introduced; the primitive is a pure builder
function. ADR-021's presentation-layer schedule is untouched.

### AC10 -- No protocol or server change

`git diff origin/main..HEAD server/ shared/ tests/integration/server/`:

```text
(no output)
```

Zero changes under server, shared, or any server-side test bin.

### AC11 -- No accept-risk closure claimed

This evidence document explicitly does NOT claim closure of:

- `S8-QA-001-W1` (OPEN preserved)
- `QA-COND-0005` Standard-tier accessibility (UNCHANGED -- hit-target
  rectangles are not advanced to >=44 px)
- `QA-COND-0006` playtest validation (UNCHANGED)
- `PAW-TD-*-a` placeholder-art accept-risk (UNCHANGED -- no asset
  edit)
- Per-surface migration of HAND / DRAFT-GRID / AUCTION-FEATURED /
  BOARD-GHOST (those four remain Sprint 17+ Backlog under the family
  `S17-UI-CARD-SLOT-MIGRATION-*`)
- Closure of any AUDIT-1076-* finding
- Closure of any SOURCE-1077-* finding outside SOURCE-1077-06
- Closure of any of the 24 PROMPT 1022 audit findings

### AC12 -- Sprint 17 disposition preserved

`git diff origin/main..HEAD production/sprint-status.yaml
production/sprints/ production/stage.txt
production/session-state/ production/qa/qa-plan-*.md
production/qa/smoke-*.md production/qa/team-qa-*.md
production/gate-checks/ docs/architecture/adr-*.md`:

```text
(no output)
```

None of the above are modified.

### AC13 -- Worker branch scope contained

Branch: `work/s17-card-slot-inset-wiring`. Pushed worker branch only;
`main` NOT pushed. Files changed are scoped to:

- `client/src/ui/design_tokens/card_slot.rs`
- `tests/integration/ui_clean_pass/card_slot_primitive_test.rs`
- `production/qa/evidence/sprint-17-card-slot-inset-wiring/evidence.md`
  (this file -- evidence doc, owned per prompt)

No optional `docs/ux/global-ui-design-spec.md` amendment authored
(scope kept tight; the existing §12 reference is already clear about
the primitive's child-positioning contract).

### AC14 -- Cargo resource policy applied for every Cargo command

Both `cargo check -p client` and `cargo test -p client --test
ui_clean_pass_card_slot_primitive_test` were invoked under:

```text
CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

D: free space at worker start: ~761.8 GB (well above the >=50 GB
preflight floor). No stale-child cleanup required.

---

## Cargo command output

### cargo check -p client

```text
    Checking client v0.1.0 (D:\_DEV\claude-code-game-studios-worktrees\s17-card-slot-inset-wiring\client)
    Finished `dev` profile [optimized] target(s) in 6.41s
```

Zero warnings, zero errors on touched file.

### cargo test -p client --test ui_clean_pass_card_slot_primitive_test

```text
running 27 tests
test ac1_image_and_text_accessors_match_geometry_struct ... ok
test ac7_card_slot_node_width_height_match_geometry_for_every_kind ... ok
test ac2_aspect_ratio_preserved_across_canonical_viewports ... ok
test ac2_each_kind_aspect_ratio_falls_in_declared_band ... ok
test ac2_each_kind_outer_dimensions_strictly_positive_and_finite ... ok
test ac4_image_and_text_containment_at_1366x768_and_1024x600_sentinel ... ok
test ac4_image_and_text_insets_fit_inside_outer_rectangle_per_kind ... ok
test ac4_image_and_text_rectangles_are_disjoint_per_kind ... ok
test ac3_card_slot_module_doc_comments_reference_interaction_state_families ... ok
test ac1_all_five_card_slot_kinds_are_importable_from_public_path ... ok
test s17_inset_builders_dimensions_resolve_to_positive_interior_per_kind ... ok
test ac7_card_slot_node_width_height_match_geometry_for_shop_slot ... ok
test ac6_spec_amendment_introduces_section_twelve_card_slot_primitive ... ok
test ac7_hit_target_is_superset_of_or_equal_to_visual_outer_rectangle ... ok
test ac8_card_slot_module_does_not_advance_friend_game_scope_guards ... ok
test ac1_module_body_does_not_introduce_naked_val_px_numeric_literal ... ok
test s17_inset_builders_cover_every_card_slot_kind_variant ... ok
test ac3_interaction_state_token_families_importable_from_published_path ... ok
test s17_inset_image_node_carries_no_inline_size_overrides_per_kind ... ok
test s17_inset_image_node_edges_match_geometry_per_kind ... ok
test s17_inset_image_node_position_type_absolute_per_kind ... ok
test s17_inset_text_node_carries_no_inline_size_overrides_per_kind ... ok
test s17_inset_text_node_position_type_absolute_per_kind ... ok
test s17_inset_image_and_text_builders_thread_global_z_index_per_kind ... ok
test ac7_each_kind_resolves_to_distinct_outer_size_and_z_layer_triple ... ok
test s17_inset_text_node_edges_match_geometry_per_kind ... ok
test ac5_phase_1_shop_slot_node_outer_geometry_matches_primitive ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Sprint 17 net-new tests (10 total): 8 in the integration bin + 2 in
the inline `mod tests` (the inline 9-test count includes 6 pre-
existing). Per-variant coverage across the 5 `CardSlotKind` variants
amounts to 50 per-variant assertions across the new tests.

### cargo test -p client --lib card_slot (inline `mod tests`)

```text
running 9 tests
test ui::design_tokens::card_slot::tests::ac1_all_accessors_return_geometry_consistent_values ... ok
test ui::design_tokens::card_slot::tests::ac2_each_kinds_aspect_ratio_falls_in_declared_band ... ok
test ui::design_tokens::card_slot::tests::ac2_each_kinds_outer_dimensions_are_strictly_positive_and_finite ... ok
test ui::design_tokens::card_slot::tests::all_card_slot_kinds_enumerates_every_variant ... ok
test ui::design_tokens::card_slot::tests::s17_image_inset_node_matches_geometry_per_kind ... ok
test ui::design_tokens::card_slot::tests::s17_text_inset_node_matches_geometry_per_kind ... ok
test ui::design_tokens::card_slot::tests::ac4_image_and_text_insets_fit_inside_outer_rectangle_per_kind ... ok
test ui::design_tokens::card_slot::tests::ac7_card_slot_node_width_height_match_geometry_for_shop_slot ... ok
test ui::design_tokens::card_slot::tests::s17_inset_builders_thread_global_z_index_from_geometry_per_kind ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out; finished in 0.00s
```

### git diff --check

```text
(no output)
```

No whitespace errors.

---

## Conditions carried forward (UNCHANGED)

- Sprint 16 disposition `closed-with-conditions` (UNCHANGED).
- Sprint 17 stage `Polish` (UNCHANGED).
- PROMPT 761 Polish->Release gate-check `FAIL` preserved.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` + `QA-COND-0006` accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved.
- `TQ-S12-C1..C7` preserved verbatim.
- Sprint 15 / 14 / 13 / 12 / 11 / 10 dispositions preserved
  unchanged.
- HUD timer row `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-
  blocked carry preserved; NOT closed by this row.
- 24 PROMPT 1022 audit findings preserved as report-only; NOT closed
  by this row.

---

`1102: S17-UI-CARD-SLOT-INSET-WIRING-001: PASS`
