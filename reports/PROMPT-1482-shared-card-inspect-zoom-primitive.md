PROMPT 1482 -- SHARED-CARD-INSPECT-ZOOM-PRIMITIVE

Status: COMPLETE

Files:
- `client/src/ui/card_inspect.rs`
- `client/src/ui/mod.rs`

API surface:
- Added `client::ui::card_inspect` as a shared, consumer-unwired primitive module.
- Added `CardInspectView` data model for title, cost, attack, health, keyword, and rules text.
- Added `CardInspectEntities` return struct for future consumer update wiring.
- Added `spawn_card_inspect(parent, view)` helper that builds the bounded shell, fallback art area, cost badge, text strip, keyword/rules text, and stat badges.
- Added node/component helper functions: `card_inspect_root_node`, `card_inspect_art_node`, `card_inspect_text_strip_node`, `card_inspect_badge_node`, `card_inspect_title_node`, `card_inspect_rules_node`, `card_inspect_stats_row_node`, `card_inspect_art_image_node`, `card_inspect_art_image_mode`, and `card_inspect_fits_1280x720`.

Deterministic markers:
- `CardInspectRoot`
- `CardInspectArtArea`
- `CardInspectTextStrip`
- `CardInspectTitleText`
- `CardInspectRulesText`
- `CardInspectKeywordText`
- `CardInspectCostBadge`
- `CardInspectAttackBadge`
- `CardInspectHealthBadge`

Layout / responsiveness:
- Root shell is bounded to `320x520` with `max_width: 92%` and `max_height: 92%`.
- The art area, text strip, title, rules text, and badges use explicit stable dimensions or bounded overflow.
- Text uses existing `text_fit` policies: single-line clipped title/keyword/badges and word-boundary rules text.
- Art uses existing card-slot image-mode policy (`NodeImageMode::Auto`) and no external assets.

Validation:
- Ran `cargo check -p client` successfully.
- The check emitted existing deprecation warnings for broad HUD/hand/shop marker types; no new compile errors.
- Did not run broad workspace tests per prompt validation policy.

Follow-up consumer lanes:
- Hand hover/inspect lane can spawn `CardInspectRoot` under a hand-owned overlay parent and update marked text entities from `HandCardCatalog`.
- Shop/draft/auction hover lane can reuse the same markers and `CardInspectView` without changing the primitive.
- A future visual polish lane can attach real card art through the existing asset-wiring helper against the `CardInspectArtArea` entity while preserving `NodeImageMode::Auto`.

Branch:
- Branch: `work/shared-card-inspect-zoom-primitive-1482`

1482: SHARED-CARD-INSPECT-ZOOM-PRIMITIVE: COMPLETE
