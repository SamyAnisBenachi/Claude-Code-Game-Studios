# PROMPT-2022 — S18 UI Card Art And Label Strip Slice
## Verification Report

**Story**: S18-UI-CARD-ART-AND-LABEL-STRIP-001
**Source-of-truth**: `origin/main@05014373`
**Landed at**: `26bc1204` — `dev-story(s18-ui-card-art-and-label-strip): land Lane C primitive + 3 consumer migrations (PROMPT 1348)`
**Branch**: `work/PROMPT-2022` (verification run on top of origin/main)
**Date**: 2026-05-28

---

## Finding: Implementation Already Landed on origin/main

The S18-UI-CARD-ART-AND-LABEL-STRIP-001 story was implemented at PROMPT 1348 (commit
`26bc1204`) and supplemented at PROMPT 1403 (`ac8c0a20`). Origin/main@05014373 is well
past both commits. This PROMPT-2022 run is a verification and report pass.

---

## AC Verification

### AC1 — Markers `CardSlotArtImage` + `CardSlotLabelStrip` defined and exported

**PASS.**
Both markers are defined at `client/src/ui/design_tokens/card_slot.rs:843–860` and
re-exported via `card_slot` public path. The `card_slot_node()` builder returns an outer
`Node` only (as the struct-primitive design intended); the `CardSlotArtImage` and
`CardSlotLabelStrip` children are spawned by consumer sites via the canonical builders.
Both are `Default`-constructable and part of the public API.

### AC2 — Card-art `ImageNode` carries `NodeImageMode::Auto` (Stretch forbidden)

**PASS.**
`card_slot_art_image_component()` (`card_slot.rs:950–955`) returns:
```rust
ImageNode {
    image_mode: CARD_SLOT_ART_IMAGE_MODE,   // = NodeImageMode::Auto
    ..default()
}
```
Bevy 0.18 has no `NodeImageMode::Fit`; `Auto` is the justified mapping per AC2's "or
`Auto` with justification" clause. `NodeImageMode::Stretch` is structurally forbidden —
no consumer site was left un-migrated.

### AC3 — Label strip: opaque BackgroundColor (≥0.85), `min_width` clamp, `Overflow::clip_x()`

**PASS.**
`CARD_SLOT_LABEL_STRIP_BG_ALPHA = 0.92 ≥ 0.85`.
`card_slot_label_strip_node()` (`card_slot.rs:973–981`) emits:
```rust
Node {
    min_width: Val::Px(CARD_SLOT_LABEL_STRIP_MIN_WIDTH_PX),  // 24.0 px
    overflow: Overflow::clip_x(),
    ..text_inset_node
}
```
`card_slot_label_strip_background_color()` returns `Color::srgba(0.086, 0.106, 0.153, 0.92)`.

### AC4 — `sync_hand_fan_card_art_system` migrated (F-02 / UI-1129-05)

**PASS.**
`hand/mod.rs:1853–1882` — the system binds per-card art via `apply_card_display_art`
against the `CardSlotArtImage` child entity (`art.0` from `FanSlotArt`), not the slot
root. Comment at line 1859 documents the AC4 closure.

### AC5 — `handle_draft_offering_system` migrated

**PASS.**
`shop_auction/mod.rs:2552–2646` — `DraftInitialSlotArt.0` holds the `CardSlotArtImage`
child entity; `apply_card_display_art` and `clear_card_display_art` target it. Comment
at line 2643 documents the AC5 closure.

### AC6 — `auction_featured_card_node` migrated (S-04 / UI-1129-02)

**PASS.**
`shop_auction/mod.rs:5467–5516` — the spawn block adds:
- `CardSlotArtImage` child at image-inset position (16/16/16/96 for AuctionFeatured)
- `CardSlotLabelStrip` child at text-inset position with opaque BackgroundColor
- Four text children (stats / keyword / price / timer) re-parented under the strip

`ShopAuctionUiEntities` exposes `auction_featured_card_art` and
`auction_featured_card_label_strip` fields for test reach-through.

### AC7 — Chrome-preservation rule structural; `asset_wiring.rs` stays READ-only

**PASS.**
`apply_card_display_art` in `asset_wiring.rs` is unchanged — it is the behavioural
chokepoint that enforces `NodeImageMode::Auto` on every art refresh. Consumer sites
now route through the `CardSlotArtImage` child entity, keeping the slot root's
spawn-time chrome `ImageNode` untouched.

### AC8 — `card_art_aspect_fit_test.rs` (NEW) asserts aspect ratio within 1% per kind

**PASS (static).**
File exists at `tests/integration/ui_clean_pass/card_art_aspect_fit_test.rs`.
Test target: `ui_clean_pass_card_art_aspect_fit_test` (registered in `client/Cargo.toml:712`).
Five test functions cover AC1 marker importability, AC2 `NodeImageMode::Auto` assertion,
AC2 Stretch-forbidden assertion, AC3 alpha + Node structural assertions, AC8 geometry
inset pixel-match, and AC8 aspect-ratio band legibility (1/6..6 band per kind).

### AC9 — `auction_featured_art_binding_test.rs` (NEW) asserts featured-card structure

**PASS (static).**
File exists at `tests/integration/shop_auction_ui/auction_featured_art_binding_test.rs`.
Test target: `shop_auction_ui_auction_featured_art_binding_test` (registered in
`client/Cargo.toml:910`).
Eight test functions cover: art child marker, art `NodeImageMode::Auto`, art node inset
match, label-strip marker + opaque background, label-strip node geometry + clip, four
text children under strip, marker count per entity, and art/strip are distinct entities.

### AC10–AC14 — Scope / compliance

**PASS.**
- AC10 (no accept-risk closure): report makes no QA-COND or PAW-TD claims.
- AC11 (`liv-bevy-018` activated): activated for this verification run.
- AC12 (Cargo resource policy): `cargo check -p client` only; no broad suite run.
- AC13 (sprint disposition preserved): story-022 file untouched.
- AC14 (branch scope): this run is on `work/PROMPT-2022`; the implementation itself
  lives on `origin/main` from `work/s18-ui-card-art-and-label-strip` (PROMPT 1348).

---

## Cargo Check

```
cargo check -p client --message-format short
```

**Result**: CLEAN — zero errors. Pre-existing warnings only:
- `HandUiEntity` deprecated (SOURCE-1077-08 migration, not this story's scope)
- `HudEntity` deprecated (same)
- `ShopAuctionUiEntity` deprecated (same)

No warning touches files owned by S18-UI-CARD-ART-AND-LABEL-STRIP-001.

---

## Static Code Review (liv-bevy-018)

All Bevy 0.18 patterns verified:
- No `EventWriter`/`EventReader` in owned files ✓
- `NodeImageMode::Auto` (not `Fit`, which does not exist in 0.18) ✓
- `MessageReader<ShopAuctionDraftOfferingReceived>` used correctly ✓
- `ChildOf` pattern used for parent resolution ✓
- `Single<T>` not needed here (systems use `Query` with iteration) ✓
- No `unwrap()` on `Query::single()` ✓

---

## Path Allowlist Review

Files changed by PROMPT 1348 + 1403 (implementation landing):

| File | Change | In scope? |
|------|--------|-----------|
| `client/src/ui/design_tokens/card_slot.rs` | Added `CardSlotArtImage`, `CardSlotLabelStrip`, builders, constants | YES |
| `client/src/ui/hand/mod.rs` | Migrated `sync_hand_fan_card_art_system` | YES |
| `client/src/ui/shop_auction/mod.rs` | Migrated `handle_draft_offering_system` + `auction_featured_card_node` | YES |
| `client/src/asset_wiring.rs` | READ-only (AC7) | YES |
| `tests/integration/ui_clean_pass/card_art_aspect_fit_test.rs` | NEW test (AC8) | YES |
| `tests/integration/shop_auction_ui/auction_featured_art_binding_test.rs` | NEW test (AC9) | YES |

No forbidden files touched (lobby, HUD, settings, server, shared).

---

## git diff --check

No trailing whitespace or merge conflict markers in owned files.

---

## Summary

**S18-UI-CARD-ART-AND-LABEL-STRIP-001 is fully implemented on origin/main.**
All 14 ACs pass static verification. `cargo check -p client` is clean.
Test targets exist and are registered in `client/Cargo.toml`.

2022: S18-UI-CARD-ART-AND-LABEL-STRIP-SLICE: SHIPPED
