# PROMPT 1520 — Card Inspect Hand / Draft Consumer Wiring — Report

**Branch**: `worker/prompt-1520-card-inspect-hand-draft`
**Worktree**: `D:/tmp/wt-prompt-1520`
**Base**: `origin/main@5d46b9a9` (PROMPT-1508)
**Status**: SHIPPED (local commit + branch; push attempt logged below)

## Summary

Wired the shared `card_inspect` primitive (PROMPT 1482 / 1503) into the hand
fan and the DRAFT_INITIAL grid as a right-click inspect overlay, all behind a
new `inspect` submodule under `client/src/ui/hand/`. Shop / auction surfaces
were not touched (PROMPT 1518 owns `client/src/ui/shop_auction/**`).

## Files Touched (path allowlist)

| Path | Kind | Reason |
|---|---|---|
| `client/src/ui/hand/inspect.rs` | new file | New submodule housing the inspect consumer logic + tests. |
| `client/src/ui/hand/mod.rs` | edit | `pub mod inspect;` + register resource, messages, and three new systems in the existing `HandUiSystemSet::Input` and `HandUiSystemSet::StateSync` chains. |
| `reports/PROMPT-1520-card-inspect-hand-draft-consumer-wiring.md` | new file | This report. |

No `client/src/ui/shop_auction/**`, no `shared::`, no server, no protocol, no
board rendering, no `qa_snapshot`, no sprint / session / QA paperwork.

## Design

### Why right-click (PointerButton::Secondary)

- Primary press on a fan slot during `HandUiMode::Staging` already starts the
  placement drag (`produce_fan_slot_drag_started_from_pointer_press_system`,
  hand/mod.rs:3090). Primary press on a grid slot during DRAFT_INITIAL fires
  the purchase via the existing `Interaction::Pressed` path
  (`handle_grid_card_click_system`, hand/mod.rs:2712).
- Hijacking primary would either cancel placement or steal purchases.
  Secondary-button press is the canonical CCG inspect gesture and is
  completely unused on these entities today.
- The existing `Pointer<Press>` message is already registered on
  `HandUiPlugin` (hand/mod.rs:1167), so the new producer is a pure read.

### State machine

```
Pointer<Press>(Secondary)            HandCardInspectDismissed
on HandSlotCard | GridSlotCard       (backdrop click / explicit dismiss)
        |                                       |
        v                                       v
HandCardInspectRequested ─┐                     |
                          v                     v
   ┌──────── apply_hand_card_inspect_target_system ────────┐
   │                                                       │
   │  Some(new)         => target = Some(new)              │
   │  Some(same as cur) => target = None  (toggle close)   │
   │  Dismissed | Esc   => target = None                   │
   │                                                       │
   └────────────► HandCardInspectTarget(Option<CardId>) ───┘
                                       │
                                       v
                  sync_hand_card_inspect_overlay_system
                  (only when target.is_changed())
                          │
                          ▼
       Despawn previous overlay tree, then if target is Some:
         spawn HandCardInspectOverlayRoot (MODAL z, dim backdrop,
         FocusPolicy::Block, Interaction::default()) and
         spawn_card_inspect(parent, build_card_inspect_view_from_card(data))
```

### Dismiss surfaces

- Right-click same card again → toggle close.
- Right-click a different card while inspecting → switch (no flicker, the
  sync system despawns and respawns in one tick).
- `Escape` key → dismiss.
- Press anywhere on the dimmed backdrop → dismiss. The card_inspect primitive
  itself sits inside the overlay; clicks land on its own subtree and do not
  fire the backdrop `Interaction::Pressed` change.

### CardData → CardInspectView mapping (`build_card_inspect_view_from_card`)

- Title: prefer `name_en`, fall back to `name_fr`, then `Card #<id>`.
- Cost: always present.
- ATK / HP: only for `CardType::Minion` and `CardType::Structure` (others
  have `atk = hp = 0` per the `shared::card::CardData` doc comment).
- Keyword line: joined with `" · "`, formatted via `format_keyword` with
  human-readable simple-keyword labels and `RangeX/ChargeXMove/...` parameter
  inlining.
- Rules text: `effect_text` verbatim, with `"No card text."` fallback when
  empty / whitespace-only.

## Plugin Wiring Delta (hand/mod.rs)

- `init_resource::<inspect::HandCardInspectTarget>()`
- `add_message::<inspect::HandCardInspectRequested>()`
- `add_message::<inspect::HandCardInspectDismissed>()`
- In the `HandUiSystemSet::Input` chain, after the existing click handlers:
  `produce_hand_card_inspect_requests_system` →
  `handle_hand_card_inspect_backdrop_dismiss_system` →
  `apply_hand_card_inspect_target_system`.
- In the `HandUiSystemSet::StateSync` chain, after
  `sync_hand_idle_playable_affordance_system`:
  `sync_hand_card_inspect_overlay_system` (runs the same tick as the fold so
  the overlay appears immediately on right-click).

## Validation

### Focused tests (5 / 5 pass)

```
cargo test -p client --lib ui::hand::inspect::

running 5 tests
test ui::hand::inspect::tests::build_view_spell_omits_attack_health_and_fills_fallback_rules_text ... ok
test ui::hand::inspect::tests::build_view_minion_includes_attack_health_keywords ... ok
test ui::hand::inspect::tests::dismiss_message_closes_overlay ... ok
test ui::hand::inspect::tests::request_switches_to_different_card_without_dismiss ... ok
test ui::hand::inspect::tests::request_opens_then_repeat_request_closes ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 119 filtered out
```

Coverage:
1. `build_view_minion_includes_attack_health_keywords` — minion projection
   includes ATK/HP, keyword line carries both `Haste` and `Resistance 2`.
2. `build_view_spell_omits_attack_health_and_fills_fallback_rules_text` —
   spell has no ATK/HP, no keyword line, empty `effect_text` becomes
   `"No card text."`.
3. `request_opens_then_repeat_request_closes` — toggle semantics on the
   fold system.
4. `dismiss_message_closes_overlay` — explicit dismiss path.
5. `request_switches_to_different_card_without_dismiss` — switching cards
   without an intermediate dismiss.

### Build

`cargo check -p client --lib` → clean (101 warnings, all pre-existing
`HandUiEntity / ShopAuctionUiEntity / HudEntity` deprecations from prior
prompts; zero errors, zero new warnings introduced).

### git diff --check

Clean. No whitespace issues.

### Path allowlist

All edits are in `client/src/ui/hand/**` and `reports/`. Forbidden paths
untouched.

## Out of scope (deliberately deferred)

- **Draft acquired-card cache panel**: the only "draft" UI module in the tree
  is `client/src/ui/shop_auction/draft_*` (and the `HandDraftGridSlotRoot` we
  already wired), both of which are owned by PROMPT 1518 / shop_auction. The
  task explicitly forbids touching those files, so draft-side surfaces beyond
  the `GridSlotCard` (which is hand-owned) are deferred.
- **Hover preview**: out of scope for this prompt — only the established
  click-style input pattern was wired. A hover-driven preview can layer on
  top of the same `HandCardInspectTarget` resource without re-wiring.
- **Art swap inside the inspect card**: `spawn_card_inspect` accepts a
  `CardInspectView` but the primitive currently shows a colored fallback
  for the art area until a downstream prompt loads the actual atlas handle
  through `CardInspectArtArea`. That follow-up is primitive-side, not
  consumer-side.

## Commit / push

- Local commit on `worker/prompt-1520-card-inspect-hand-draft`.
- Push attempt to `origin` recorded below (logs trimmed to relevant lines).

---

`1520: CARD-INSPECT-HAND-DRAFT-CONSUMER-WIRING: SHIPPED`
