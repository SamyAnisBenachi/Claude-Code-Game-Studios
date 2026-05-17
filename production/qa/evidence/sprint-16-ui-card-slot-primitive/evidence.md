# Sprint 16 / Story 009 — `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` Evidence

> **Authored by**: PROMPT 1067 (`/dev-story` for story 009)
> **Worker branch**: `work/s16-card-slot-primitive`
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s16-card-slot-primitive`
> **Source-of-truth at worker activation**: `origin/main@7a78b257cdcd8b76f439f7264b31648b3ae1261c`
> (PROMPT 1066 `qa(s16): author Sprint 16 QA plan (PROMPT 1066)`)

This evidence dir captures the Phase 1 deliverable for the Sprint 16
UI Card-Slot Primitive row. The default Sprint 16 scope is **primitive
module + spec amendment + shop slot Phase 1 migration + viewport-
invariant integration test + evidence** per the story file. Per-surface
migration of the hand / draft / auction-featured / board staged-ghost
surfaces is **OUT OF SCOPE** for this row and is owned by the Sprint
16+ `S16-UI-CARD-SLOT-MIGRATION-*` follow-on family.

---

## AC1..AC8 doc-review checklist

| AC | Verdict | Evidence |
|----|---------|----------|
| **AC1 — Authoritative primitive module + token usage** | `[x]` | `client/src/ui/design_tokens/card_slot.rs` exists; declared from `client/src/ui/design_tokens/mod.rs`; exports the `CardSlotKind` enum (5 variants), `CardSlotGeometry` struct, and named accessor functions (`card_slot_geometry`, `card_slot_node`, `card_slot_image_inset`, `card_slot_text_inset`, `card_slot_hit_target`). Every numeric value is a named `const`; the AC1 grep guard in `card_slot_primitive_test.rs::ac1_module_body_does_not_introduce_naked_val_px_numeric_literal` enforces "no inline `Val::Px(<digit>)` literal" at the public-API boundary. Doc comments on every published item name the consumer surface and forward-reference spec §12. |
| **AC2 — No nested cards, no layout shift, stable aspect ratio** | `[x]` | Inline `tests` block in `card_slot.rs` and integration tests `ac2_each_kind_outer_dimensions_strictly_positive_and_finite`, `ac2_each_kind_aspect_ratio_falls_in_declared_band`, `ac2_aspect_ratio_preserved_across_canonical_viewports`, `ac7_card_slot_node_width_height_match_geometry_for_every_kind` cover the four sub-clauses. The `card_slot_node` builder is leaf-only — it never instantiates `card_slot_node` for another kind. The slot is pixel-fixed; aspect ratio is identical across `CANONICAL_VIEWPORTS`. |
| **AC3 — Hover / focus / pressed / disabled state mapping via existing interaction primitives** | `[x]` | Doc comments on each `CardSlotKind` variant cite all four families (`HOVER_*`, `FOCUS_*`, `PRESSED_*`, `DISABLED_*`) from `client/src/ui/design_tokens/interaction_states.rs`. Integration test `ac3_interaction_state_token_families_importable_from_published_path` proves the four families are importable from the published path; `ac3_card_slot_module_doc_comments_reference_interaction_state_families` proves the cross-reference is in the doc comments. Per-surface migration of interaction visuals is OUT OF SCOPE (deferred to `S16-UI-INTERACTION-STATE-MIGRATION-*`). |
| **AC4 — Image / text containment at 1366 × 768 and a smaller viewport** | `[x]` | Integration tests `ac4_image_and_text_insets_fit_inside_outer_rectangle_per_kind`, `ac4_image_and_text_rectangles_are_disjoint_per_kind`, `ac4_image_and_text_containment_at_1366x768_and_1024x600_sentinel` cover containment, disjointness, and viewport-iteration. The 1024 × 600 sentinel is the worker-chosen smaller-than-canonical viewport per AC4. |
| **AC5 — Per-surface migration boundaries split into phases** | `[x]` | Only the `shop_slot_node` helper in `client/src/ui/shop_auction/mod.rs` is migrated. `git-diff-stat-disjoint-surfaces.txt` confirms (a) `client/src/ui/hand/mod.rs` UNCHANGED, (b) `client/src/presentation/` UNCHANGED, (c) only the `shop_slot_node` body inside `client/src/ui/shop_auction/mod.rs` changed (no edit to `auction_featured_card_node` or `AUCTION_FEATURED_CARD_*` constants). Integration test `ac5_phase_1_shop_slot_node_outer_geometry_matches_primitive` asserts the helper now calls `card_slot_node(CardSlotKind::ShopSlot)` and that the naked `Val::Px(136.0)` / `Val::Px(78.0)` literals are gone from the helper body. |
| **AC6 — Visual evidence / screenshot harness expectations** | `[~]` partial — paperwork present; QA snapshot bundles deferred to human operator | This evidence directory contains the doc-review checklist (this file), the cargo test pass log (`cargo-test-card-slot-primitive.log`), the cargo check pass log (`cargo-check-client.log`), the disjoint-surface git diff (`git-diff-stat-disjoint-surfaces.txt`), the spec heading scan (`spec-heading-scan.txt`), and the spec adoption matrix diff excerpt (`spec-adoption-matrix-diff.md`). QA snapshot bundles at `1366 × 768` and `1920 × 1080` are placeholder instructions only — capture is **deferred to the human operator** because PROMPT 1067 has no playable-client runtime available. Integration test `ac6_spec_amendment_introduces_section_twelve_card_slot_primitive` asserts the §12 heading is present in `docs/ux/global-ui-design-spec.md`. |
| **AC7 — Tests expected, including viewport-invariant / layout-contract test** | `[x]` | Integration test bin `tests/integration/ui_clean_pass/card_slot_primitive_test.rs` registered in `client/Cargo.toml` as `ui_clean_pass_card_slot_primitive_test`, mirroring the Sprint 15 story 008 pattern. All AC7 sub-assertions covered by tests prefixed `ac1_..ac8_` (see test bin source). |
| **AC8 — Non-claims (no gameplay / no server / no release / no final-art)** | `[x]` | `git-diff-stat-disjoint-surfaces.txt` shows `server/src/` and `shared/src/` UNCHANGED. No claim of release readiness / release-candidate readiness / Standard-tier accessibility / final-art completion / `S8-QA-001-W1` closure / `Polish->Release` retry / stage advance. Friend-game scope guards (`QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`) preserved verbatim in card_slot.rs doc comments — integration test `ac8_card_slot_module_does_not_advance_friend_game_scope_guards` enforces. |

---

## QA snapshot bundle instructions (deferred — human operator)

Per AC6 and the Sprint 16 QA plan §"S12-TD-UI-CARD-SLOT-PRIMITIVE-001",
a manual playable-client run is required to capture the migrated shop
panel at `1366 × 768` and `1920 × 1080`. PROMPT 1067 is a worker
without playable-client runtime access; capture is deferred to a
human operator via the `S15-QA-SNAPSHOT-DEFAULT-DEV` flow:

1. Launch the playable client with `CCGS_QA_SNAPSHOT=1` set in the
   environment (default in dev builds per PROMPT 1021 / 1023).
2. Resize the window to `1366 × 768`, advance to the shop phase, and
   press `F9` to trigger an in-game snapshot. The snapshot bundle
   lands under the existing QA snapshot output path.
3. Move the bundle into `qa-snapshot-1366x768/` under this evidence
   directory.
4. Repeat at `1920 × 1080` and move the bundle into
   `qa-snapshot-1920x1080/`.
5. (Optional) Repeat at `1024 × 600` if the launcher supports the
   viewport; if not, document the limitation in this file.
6. Verify the migrated shop panel composes via the new primitive:
   each shop slot well measures `136 × 78 px` (unchanged outer
   geometry, now sourced from `card_slot_geometry(CardSlotKind::ShopSlot)`).

No visual regression is expected — the primitive ratifies the existing
per-surface literal verbatim.

---

## Non-claims (recap)

This evidence dir does **not** claim any of the following:

- Public release readiness, release-candidate readiness, or full-game
  completion.
- Stage advance from `Polish` to `Release`.
- `Polish->Release` gate-check retry (PROMPT 761 FAIL preserved).
- Two-client GAME_OVER closure (`S8-QA-001-W1`).
- Standard-tier accessibility completion (`QA-COND-0005`).
- Playtest / fun-hypothesis validation (`QA-COND-0006`).
- Final-art / asset-production completion (`PAW-TD-*-a`).
- Hand / draft / auction-featured / board staged-ghost migrations
  (owned by the Sprint 16+ `S16-UI-CARD-SLOT-MIGRATION-*` family).
- Per-surface interaction-state migration (owned by the Sprint 16+
  `S16-UI-INTERACTION-STATE-MIGRATION-*` family).
- Sprint 14 / Sprint 15 row reopen.
