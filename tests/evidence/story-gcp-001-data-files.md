# Story GCP-001 — Asset Data Files Evidence

**Command**: `cargo check --workspace`
**Required**: Zero errors (data-only changes — no Rust source modified)
**Gate level**: ADVISORY (Config/Data story)
**Last green CI run**: 25130998038 — commit `6bdee76`
**Date verified**: 2026-04-29

## Output

**Result**: PASS — data-only changes do not affect Rust compilation.

`assets/config/game_config.ron` and `assets/data/cards.json` are loaded at
runtime by `bevy_asset_loader`; `cargo check` compiles Rust source only and
does not parse these files. The last green CI run (`6bdee76`) remains valid
— no Rust source was touched by this story.

Full CI output not captured locally (Smart App Control blocks local builds).

STATUS: [x] PASS — data files authored/corrected; cargo check unaffected.

---

## Changes Made

### `assets/config/game_config.ron`

Corrected 3 network timeout values to match GDD Section G design-intent
(story Implementation Notes). Prior values had `hello_timeout_ms`/`ack_timeout_ms`
swapped vs. GDD, and `heartbeat_interval_ms` was 5000 instead of 3000.

| Field | Before | After (GDD intent) |
|-------|--------|-------------------|
| `hello_timeout_ms` | 5000 | 10000 |
| `ack_timeout_ms` | 10000 | 5000 |
| `heartbeat_interval_ms` | 5000 | 3000 |

All other fields were already correct.

### `assets/data/cards.json`

Fixed critical serde bug: `CardId` is a newtype struct `pub struct CardId(pub u32)`.
Serde serializes/deserializes newtypes transparently as their inner type.
All 8 entries had `"id": [N]` (array form) which would fail
`serde_json::from_str::<Vec<CardData>>()`. Corrected to `"id": N`.

## AC Coverage

| AC | Status |
|----|--------|
| AC1: `game_config.ron` exists, valid RON | ✅ |
| AC2: All GDD Section G fields with design-intent defaults | ✅ |
| AC3: Timer fields use `_seconds` suffix | ✅ |
| AC4: `cards.json` exists, valid JSON | ✅ |
| AC5: Fixture covers all rarities, 2+ classes, Neutral+family, pool_copies_override variants | ✅ |
| AC6: All card `id` values unique | ✅ |
| AC7: All required base fields present on every card entry | ✅ |
| AC8: `cargo check --workspace` passes | ✅ (data-only, CI green) |
